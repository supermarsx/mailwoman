//! 26.20 t24-e7 (B3, t23 E4-07): one send is one send.
//!
//! Before this change a submission became `final` only after the whole of
//! `submit_email` returned, and that included filing the Sent copy. SMTP had
//! already accepted the message by then, but a failed upstream APPEND (a tagged
//! `NO`, e.g. `[OVERQUOTA]`) propagated as an error, left the row `pending`, and
//! the dispatcher handed the same draft to SMTP again on the next 500 ms scan —
//! indefinitely, and across restarts. The inline (no hold) path had the mirror
//! image: it marked a delivered message `canceled`.
//!
//! These tests drive the real engine over an in-process backend whose Sent
//! APPEND can be told to fail or to hang, and a submitter that counts every
//! message it is handed. The count is the property: the number of times the
//! recipients would have received the message.

#[path = "../../mw-store/src/test_db.rs"]
mod test_db;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MailboxRole,
    MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor,
    WatchHandle,
};
use mw_engine::dispatcher::MAX_SUBMISSION_ATTEMPTS;
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store, SubmissionRow};

const UIDVALIDITY: u32 = 100;

/// How the fake backend answers an APPEND to the Sent folder. APPENDs to any
/// other folder (the draft) always succeed.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SentAppend {
    Accept,
    /// A tagged `NO`, which `mw-imap` surfaces as `EngineError::Protocol`.
    RefuseNo,
    /// Never returns: stands in for the process dying mid-filing.
    Hang,
}

struct FakeBackend {
    sent_append: SentAppend,
    /// Signalled when a Sent APPEND starts, so a test can act inside the window.
    sent_append_entered: Arc<tokio::sync::Notify>,
    appended: Mutex<Vec<String>>,
}

impl FakeBackend {
    fn new(sent_append: SentAppend) -> Self {
        Self {
            sent_append,
            sent_append_entered: Arc::new(tokio::sync::Notify::new()),
            appended: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl AccountBackend for FakeBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: true,
            special_use: true,
            ..BackendCaps::default()
        })
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok([
            ("INBOX", MailboxRole::Inbox),
            ("Sent", MailboxRole::Sent),
            ("Drafts", MailboxRole::Drafts),
        ]
        .into_iter()
        .map(|(name, role)| RawMailbox {
            mailbox_ref: RawMailboxRef {
                name: name.to_string(),
                uidvalidity: UIDVALIDITY,
            },
            role,
            parent: None,
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
        })
        .collect())
    }

    async fn sync_mailbox(
        &self,
        _mbox: &RawMailboxRef,
        _cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        Ok(MailboxDelta {
            added: Vec::new(),
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: UIDVALIDITY,
                uidnext: 1,
            },
        })
    }

    async fn fetch_raw(&self, _refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        Ok(Vec::new())
    }

    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _rm: &[Flag]) -> Result<()> {
        Ok(())
    }

    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("move".into()))
    }

    async fn append(&self, mbox: &RawMailboxRef, _raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
        if mbox.name == "Sent" {
            self.sent_append_entered.notify_one();
            match self.sent_append {
                SentAppend::Accept => {}
                SentAppend::RefuseNo => {
                    return Err(EngineError::Protocol(
                        "NO [OVERQUOTA] mailbox is over quota".into(),
                    ));
                }
                SentAppend::Hang => std::future::pending::<()>().await,
            }
        }
        let mut appended = self.appended.lock().unwrap();
        appended.push(mbox.name.clone());
        Ok(MessageRef::Imap {
            mailbox: mbox.clone(),
            uidvalidity: UIDVALIDITY,
            uid: appended.len() as u32,
        })
    }

    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

/// What the fake SMTP server does with each message it is handed.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Smtp {
    /// Every recipient accepted, DATA accepted: the message is delivered.
    Deliver,
    /// Every `RCPT TO` refused: nothing is delivered, and retrying cannot help.
    RejectAllRecipients,
    /// The connection fails before anything is accepted.
    TransportError,
}

struct CountingSubmitter {
    mode: Mutex<Smtp>,
    /// Messages handed to SMTP, whatever the outcome.
    calls: AtomicUsize,
    /// Messages SMTP accepted — deliveries.
    delivered: AtomicUsize,
}

impl CountingSubmitter {
    fn new(mode: Smtp) -> Arc<Self> {
        Arc::new(Self {
            mode: Mutex::new(mode),
            calls: AtomicUsize::new(0),
            delivered: AtomicUsize::new(0),
        })
    }

    fn set_mode(&self, mode: Smtp) {
        *self.mode.lock().unwrap() = mode;
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn delivered(&self) -> usize {
        self.delivered.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl MailSubmitter for CountingSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mode = *self.mode.lock().unwrap();
        match mode {
            Smtp::Deliver => {
                self.delivered.fetch_add(1, Ordering::SeqCst);
                Ok(SubmissionResult {
                    accepted: msg.rcpt_to,
                    rejected: Vec::new(),
                })
            }
            Smtp::RejectAllRecipients => Ok(SubmissionResult {
                accepted: Vec::new(),
                rejected: msg
                    .rcpt_to
                    .into_iter()
                    .map(|r| (r, "550 5.1.1 no such user".to_string()))
                    .collect(),
            }),
            Smtp::TransportError => Err(EngineError::Transport("connection refused".into())),
        }
    }
}

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
    submitter: Arc<CountingSubmitter>,
}

async fn create_account(store: &Store) -> String {
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: "me@example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "me@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap()
}

async fn connect(
    store: Store,
    account_id: &str,
    backend: Arc<FakeBackend>,
    submitter: Arc<CountingSubmitter>,
) -> Harness {
    let engine = Arc::new(Engine::new(store));
    engine.register_backend(
        account_id.to_string(),
        AccountRuntime::new(
            backend as Arc<dyn AccountBackend>,
            submitter.clone() as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    engine.resync(account_id).await.unwrap();
    Harness {
        engine,
        account_id: account_id.to_string(),
        submitter,
    }
}

async fn setup(sent_append: SentAppend, smtp: Smtp) -> Harness {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = create_account(&store).await;
    connect(
        store,
        &account_id,
        Arc::new(FakeBackend::new(sent_append)),
        CountingSubmitter::new(smtp),
    )
    .await
}

async fn jmap(h: &Harness, calls: Value) -> Value {
    h.engine
        .handle_jmap(&h.account_id, &json!({ "methodCalls": calls }))
        .await
}

fn result<'a>(resp: &'a Value, call_id: &str) -> &'a Value {
    resp["methodResponses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r[2] == call_id)
        .map(|r| &r[1])
        .unwrap_or(&Value::Null)
}

fn draft_create() -> Value {
    json!(["Email/set", { "create": { "draft": {
        "from": [{ "email": "me@example.org" }],
        "to": [{ "email": "friend@example.org" }],
        "subject": "Invoice", "bodyValues": { "1": { "value": "please pay once" } },
        "textBody": [{ "partId": "1", "type": "text/plain" }]
    } } }, "c1"])
}

async fn create_draft(h: &Harness) -> String {
    let resp = jmap(h, json!([draft_create()])).await;
    result(&resp, "c1")["created"]["draft"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("draft created: {resp}"))
        .to_string()
}

/// Enqueue a submission whose undo window has already elapsed, so the next
/// dispatcher pass is what sends it — the web client's default path (it sends
/// with a 10 s hold).
async fn enqueue_due(h: &Harness, sub_id: &str, draft: &str) {
    h.engine
        .store()
        .insert_submission(&SubmissionRow {
            id: sub_id.into(),
            account_id: h.account_id.clone(),
            email_id: draft.into(),
            identity_id: None,
            send_at: None,
            undo_status: "pending".into(),
            hold_seconds: 10,
            created_at: "2000-01-01T00:00:00Z".into(),
        })
        .await
        .unwrap();
}

async fn submission(h: &Harness, sub_id: &str) -> Value {
    let g = jmap(
        h,
        json!([["EmailSubmission/get", { "ids": [sub_id] }, "g"]]),
    )
    .await;
    result(&g, "g")["list"][0].clone()
}

async fn draft_exists(h: &Harness, draft: &str) -> bool {
    let g = jmap(h, json!([["Email/get", { "ids": [draft] }, "g"]])).await;
    !result(&g, "g")["list"].as_array().unwrap().is_empty()
}

/// Enough dispatcher passes to have re-sent many times over under the old code,
/// which re-sent on every pass.
const TICKS: usize = 10;

/// THE regression for B3. SMTP accepts the message, the Sent APPEND gets a
/// tagged `NO`, and the dispatcher keeps running. The message must have been
/// delivered once, the submission must be `final`, and the draft must be gone —
/// a draft left behind is what the old retry found and sent again.
#[tokio::test]
async fn a_refused_sent_append_does_not_send_the_message_again() {
    let h = setup(SentAppend::RefuseNo, Smtp::Deliver).await;
    let draft = create_draft(&h).await;
    enqueue_due(&h, "sub-b3", &draft).await;

    for _ in 0..TICKS {
        h.engine.dispatch_tick().await.unwrap();
    }

    assert_eq!(
        h.submitter.delivered(),
        1,
        "one user action, one delivery: a failed Sent copy must never re-send"
    );
    assert_eq!(h.submitter.calls(), 1);
    assert_eq!(submission(&h, "sub-b3").await["undoStatus"], "final");
    assert!(
        !draft_exists(&h, &draft).await,
        "the draft is removed even though filing failed"
    );
}

/// The inline path (no hold). SMTP delivered, so the create succeeds and reports
/// `final` — not `canceled` with an error that invites the user to send again.
#[tokio::test]
async fn an_inline_send_with_a_refused_sent_append_reports_what_smtp_did() {
    let h = setup(SentAppend::RefuseNo, Smtp::Deliver).await;
    let resp = jmap(
        &h,
        json!([
            draft_create(),
            ["EmailSubmission/set", { "create": { "s1": {
                "emailId": "#draft", "mailwomanHoldSeconds": 0
            } } }, "c2"]
        ]),
    )
    .await;
    let set = result(&resp, "c2");
    assert!(
        set["notCreated"].get("s1").is_none(),
        "a delivered message is not reported as a failed send: {resp}"
    );
    assert_eq!(set["created"]["s1"]["undoStatus"], "final", "{resp}");
    let sub_id = set["created"]["s1"]["id"].as_str().unwrap().to_string();
    assert_eq!(submission(&h, &sub_id).await["undoStatus"], "final");

    for _ in 0..TICKS {
        h.engine.dispatch_tick().await.unwrap();
    }
    assert_eq!(h.submitter.delivered(), 1);
}

/// A restart inside the window between SMTP accepting the message and the Sent
/// copy being filed. The dispatch is cut off mid-APPEND (its task is aborted, as a
/// killed process would be) and a new engine is opened over the same database
/// file. The new engine must not send the message again.
#[tokio::test]
async fn a_restart_between_acceptance_and_filing_does_not_send_again() {
    let db = test_db::unique_db_path("t24-e7-restart");
    let db = db.to_str().unwrap().to_string();
    let submitter = CountingSubmitter::new(Smtp::Deliver);

    let account_id = {
        let store = Store::open(&db, ServerKey::from_bytes(&[7u8; 32]).unwrap())
            .await
            .unwrap();
        let account_id = create_account(&store).await;
        let backend = Arc::new(FakeBackend::new(SentAppend::Hang));
        let entered = backend.sent_append_entered.clone();
        let h = connect(store, &account_id, backend, submitter.clone()).await;
        let draft = create_draft(&h).await;
        enqueue_due(&h, "sub-restart", &draft).await;

        let engine = h.engine.clone();
        let dispatch = tokio::spawn(async move { engine.dispatch_tick().await });
        entered.notified().await;
        assert_eq!(
            submitter.delivered(),
            1,
            "SMTP accepted before filing began"
        );
        dispatch.abort();
        let _ = dispatch.await;
        account_id
    };

    let store = Store::open(&db, ServerKey::from_bytes(&[7u8; 32]).unwrap())
        .await
        .unwrap();
    let h = connect(
        store,
        &account_id,
        Arc::new(FakeBackend::new(SentAppend::Accept)),
        submitter.clone(),
    )
    .await;
    for _ in 0..TICKS {
        h.engine.dispatch_tick().await.unwrap();
    }

    assert_eq!(
        submitter.delivered(),
        1,
        "the restarted engine must not send a message SMTP already accepted"
    );
    assert_eq!(submission(&h, "sub-restart").await["undoStatus"], "final");
}

// ---- after the fix: what the user sees, and bounded retries -------------------

fn next_attempt_at(sub: &Value) -> DateTime<Utc> {
    let s = sub["mailwomanNextAttemptAt"]
        .as_str()
        .unwrap_or_else(|| panic!("a retry is scheduled: {sub}"));
    DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
}

/// When filing fails the message still counts as sent, and the Outbox says what
/// went wrong: `final`, not failed, with the filing error on `mailwomanLastError`.
/// The local copy is still filed into Sent so the user can see what went out.
#[tokio::test]
async fn a_filing_failure_is_reported_on_a_final_submission() {
    let h = setup(SentAppend::RefuseNo, Smtp::Deliver).await;
    let draft = create_draft(&h).await;
    enqueue_due(&h, "sub-note", &draft).await;
    h.engine.dispatch_tick().await.unwrap();

    let sub = submission(&h, "sub-note").await;
    assert_eq!(sub["undoStatus"], "final");
    assert_eq!(sub["mailwomanFailed"], false);
    assert_eq!(sub["mailwomanAttempts"], 0, "no attempt failed to send");
    let note = sub["mailwomanLastError"].as_str().unwrap_or_default();
    assert!(
        note.starts_with("sent, but filing the copy into Sent failed")
            && note.contains("OVERQUOTA"),
        "the filing error is kept for the Outbox: {sub}"
    );
    assert!(sub["mailwomanNextAttemptAt"].is_null());

    let mb = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let sent = result(&mb, "mb")["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "sent")
        .unwrap()["id"]
        .clone();
    let q = jmap(
        &h,
        json!([["Email/query", { "filter": { "inMailbox": sent } }, "q"]]),
    )
    .await;
    assert_eq!(
        result(&q, "q")["ids"].as_array().unwrap().len(),
        1,
        "the local Sent copy is filed even though the upstream APPEND was refused"
    );
}

/// Every recipient refused: nothing was delivered and retrying cannot change
/// that, so the submission fails on the first attempt and is never tried again.
#[tokio::test]
async fn every_recipient_refused_fails_at_once_and_is_not_retried() {
    let h = setup(SentAppend::Accept, Smtp::RejectAllRecipients).await;
    let draft = create_draft(&h).await;
    enqueue_due(&h, "sub-550", &draft).await;

    let t0 = Utc::now();
    h.engine.dispatch_tick_at(t0).await.unwrap();
    let sub = submission(&h, "sub-550").await;
    assert_eq!(
        sub["undoStatus"], "canceled",
        "RFC 8621 has no failed state"
    );
    assert_eq!(sub["mailwomanFailed"], true);
    assert_eq!(sub["mailwomanAttempts"], 1);
    assert!(
        sub["mailwomanLastError"]
            .as_str()
            .unwrap_or_default()
            .contains("all recipients rejected"),
        "{sub}"
    );

    for day in 0..TICKS as i64 {
        h.engine
            .dispatch_tick_at(t0 + chrono::Duration::days(day))
            .await
            .unwrap();
    }
    assert_eq!(
        h.submitter.calls(),
        1,
        "a terminal submission is never retried"
    );
    assert_eq!(h.submitter.delivered(), 0);
    assert!(
        draft_exists(&h, &draft).await,
        "an unsent draft is kept so the user can fix it"
    );
    // A failed submission cannot be canceled either.
    let cancel = jmap(
        &h,
        json!([["EmailSubmission/set", { "update": { "sub-550": { "undoStatus": "canceled" } } }, "u"]]),
    )
    .await;
    assert!(result(&cancel, "u")["notUpdated"].get("sub-550").is_some());
}

/// A transient failure (the SMTP connection is refused): each attempt is counted,
/// the next one waits out the backoff however many scans happen meanwhile, and
/// after the cap the submission fails and is never sent.
#[tokio::test]
async fn a_transient_failure_backs_off_and_stops_at_the_cap() {
    let h = setup(SentAppend::Accept, Smtp::TransportError).await;
    let draft = create_draft(&h).await;
    enqueue_due(&h, "sub-retry", &draft).await;

    let t0 = Utc::now();
    h.engine.dispatch_tick_at(t0).await.unwrap();
    assert_eq!(h.submitter.calls(), 1);
    let sub = submission(&h, "sub-retry").await;
    assert_eq!(sub["undoStatus"], "pending");
    assert_eq!(sub["mailwomanAttempts"], 1);
    let next = next_attempt_at(&sub);
    assert!(
        next - t0 >= chrono::Duration::seconds(29),
        "the retry delay is not the 500 ms scan interval: {sub}"
    );

    // Scans inside the backoff window do not dial SMTP.
    for _ in 0..TICKS {
        h.engine.dispatch_tick_at(t0).await.unwrap();
    }
    h.engine
        .dispatch_tick_at(next - chrono::Duration::seconds(1))
        .await
        .unwrap();
    assert_eq!(h.submitter.calls(), 1, "backoff honoured");

    let mut at = next;
    for attempt in 2..=MAX_SUBMISSION_ATTEMPTS {
        h.engine.dispatch_tick_at(at).await.unwrap();
        assert_eq!(h.submitter.calls(), attempt as usize);
        let sub = submission(&h, "sub-retry").await;
        assert_eq!(sub["mailwomanAttempts"], attempt);
        if attempt < MAX_SUBMISSION_ATTEMPTS {
            assert_eq!(sub["undoStatus"], "pending");
            let n = next_attempt_at(&sub);
            assert!(n > at, "each retry waits again");
            at = n;
        } else {
            assert_eq!(sub["undoStatus"], "canceled");
            assert_eq!(sub["mailwomanFailed"], true);
            assert!(sub["mailwomanNextAttemptAt"].is_null());
        }
    }

    for day in 1..=TICKS as i64 {
        h.engine
            .dispatch_tick_at(at + chrono::Duration::days(day))
            .await
            .unwrap();
    }
    assert_eq!(
        h.submitter.calls(),
        MAX_SUBMISSION_ATTEMPTS as usize,
        "no send after the terminal failure"
    );
    assert_eq!(h.submitter.delivered(), 0);
}

/// A retry that goes through is delivered once and ends `final`, with the earlier
/// error cleared.
#[tokio::test]
async fn a_retry_that_succeeds_is_sent_once() {
    let h = setup(SentAppend::Accept, Smtp::TransportError).await;
    let draft = create_draft(&h).await;
    enqueue_due(&h, "sub-later", &draft).await;

    let t0 = Utc::now();
    h.engine.dispatch_tick_at(t0).await.unwrap();
    let next = next_attempt_at(&submission(&h, "sub-later").await);

    h.submitter.set_mode(Smtp::Deliver);
    for i in 0..TICKS as i64 {
        h.engine
            .dispatch_tick_at(next + chrono::Duration::minutes(i))
            .await
            .unwrap();
    }
    assert_eq!(h.submitter.delivered(), 1);
    assert_eq!(h.submitter.calls(), 2);
    let sub = submission(&h, "sub-later").await;
    assert_eq!(sub["undoStatus"], "final");
    assert!(sub["mailwomanLastError"].is_null(), "{sub}");
    assert!(sub["mailwomanNextAttemptAt"].is_null(), "{sub}");
    assert!(!draft_exists(&h, &draft).await);
}

/// The inline path fails without retrying on its own: the caller was told
/// synchronously, and a background retry on top of the user's own resend is two
/// copies.
#[tokio::test]
async fn an_inline_send_that_fails_is_not_retried_in_the_background() {
    let h = setup(SentAppend::Accept, Smtp::TransportError).await;
    let resp = jmap(
        &h,
        json!([
            draft_create(),
            ["EmailSubmission/set", { "create": { "s1": {
                "emailId": "#draft", "mailwomanHoldSeconds": 0
            } } }, "c2"]
        ]),
    )
    .await;
    assert!(
        result(&resp, "c2")["notCreated"].get("s1").is_some(),
        "an undelivered inline send is reported as not created: {resp}"
    );
    let list = jmap(&h, json!([["EmailSubmission/get", {}, "g"]])).await;
    let sub = result(&list, "g")["list"][0].clone();
    assert_eq!(sub["undoStatus"], "canceled");
    assert_eq!(sub["mailwomanFailed"], true);

    h.submitter.set_mode(Smtp::Deliver);
    let t0 = Utc::now();
    for day in 0..TICKS as i64 {
        h.engine
            .dispatch_tick_at(t0 + chrono::Duration::days(day))
            .await
            .unwrap();
    }
    assert_eq!(h.submitter.calls(), 1);
    assert_eq!(h.submitter.delivered(), 0);
}

/// Once a submission is `final`, a cancel is refused and the row stays `final`.
#[tokio::test]
async fn a_cancel_cannot_overwrite_a_sent_submission() {
    let h = setup(SentAppend::Accept, Smtp::Deliver).await;
    let draft = create_draft(&h).await;
    enqueue_due(&h, "sub-done", &draft).await;
    h.engine.dispatch_tick().await.unwrap();

    let cancel = jmap(
        &h,
        json!([["EmailSubmission/set", { "update": { "sub-done": { "undoStatus": "canceled" } } }, "u"]]),
    )
    .await;
    assert!(
        result(&cancel, "u")["notUpdated"].get("sub-done").is_some(),
        "{cancel}"
    );
    assert_eq!(submission(&h, "sub-done").await["undoStatus"], "final");
    assert_eq!(h.submitter.delivered(), 1);
}
