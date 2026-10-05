//! 26.20 t28-e12: a submission that waits for a person, the release that sends
//! it, and what happens to the message once it has been sent.
//!
//! Before this change every `pending` submission was due at a time, so there was
//! nothing an MCP `mail.send` could create that would wait for the mailbox owner
//! (t26 audit OH-5); an `EmailSubmission/set` update read only
//! `undoStatus: "canceled"`, so the Outbox "Send now" was reported as `updated`
//! and did nothing; and `onSuccessUpdateEmail` was not read at all.
//!
//! These tests drive the real engine — `handle_jmap`, `submit_held`, the
//! dispatcher pass — over an in-process backend and a submitter that counts
//! every message it is handed. The count is the property: how many times the
//! recipients would have received the message.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MailboxRole,
    MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor,
    WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

const UIDVALIDITY: u32 = 100;

/// An IMAP-shaped backend with the three folders a send touches. It records the
/// folder and flags of every APPEND.
#[derive(Default)]
struct FakeBackend {
    appended: Mutex<Vec<(String, Vec<Flag>)>>,
}

impl FakeBackend {
    fn appended_to(&self, folder: &str) -> Vec<Vec<Flag>> {
        self.appended
            .lock()
            .unwrap()
            .iter()
            .filter(|(name, _)| name == folder)
            .map(|(_, flags)| flags.clone())
            .collect()
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

    async fn append(
        &self,
        mbox: &RawMailboxRef,
        _raw: &[u8],
        flags: &[Flag],
    ) -> Result<MessageRef> {
        let mut appended = self.appended.lock().unwrap();
        appended.push((mbox.name.clone(), flags.to_vec()));
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
    /// The connection fails before anything is accepted.
    TransportError,
    /// Delivered, but only once the test lets go: SMTP is in flight until then.
    DeliverWhenLetGo,
}

struct CountingSubmitter {
    mode: Mutex<Smtp>,
    /// Messages handed to SMTP, whatever the outcome.
    calls: AtomicUsize,
    /// Messages SMTP accepted — deliveries.
    delivered: AtomicUsize,
    let_go: tokio::sync::Notify,
}

impl CountingSubmitter {
    fn new(mode: Smtp) -> Arc<Self> {
        Arc::new(Self {
            mode: Mutex::new(mode),
            calls: AtomicUsize::new(0),
            delivered: AtomicUsize::new(0),
            let_go: tokio::sync::Notify::new(),
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
            Smtp::TransportError => Err(EngineError::Transport("connection refused".into())),
            Smtp::Deliver | Smtp::DeliverWhenLetGo => {
                if mode == Smtp::DeliverWhenLetGo {
                    self.let_go.notified().await;
                }
                self.delivered.fetch_add(1, Ordering::SeqCst);
                Ok(SubmissionResult {
                    accepted: msg.rcpt_to,
                    rejected: Vec::new(),
                })
            }
        }
    }
}

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
    backend: Arc<FakeBackend>,
    submitter: Arc<CountingSubmitter>,
}

async fn add_account(
    engine: &Arc<Engine>,
    username: &str,
    backend: Arc<FakeBackend>,
    submitter: Arc<CountingSubmitter>,
) -> String {
    let account_id = engine
        .store()
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: username.into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            backend as Arc<dyn AccountBackend>,
            submitter as Arc<dyn MailSubmitter>,
            username,
        ),
    );
    engine.resync(&account_id).await.unwrap();
    account_id
}

async fn setup(smtp: Smtp) -> Harness {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let engine = Arc::new(Engine::new(store));
    let backend = Arc::new(FakeBackend::default());
    let submitter = CountingSubmitter::new(smtp);
    let account_id = add_account(
        &engine,
        "me@example.org",
        backend.clone(),
        submitter.clone(),
    )
    .await;
    Harness {
        engine,
        account_id,
        backend,
        submitter,
    }
}

async fn jmap_as(h: &Harness, account_id: &str, calls: Value) -> Value {
    h.engine
        .handle_jmap(account_id, &json!({ "methodCalls": calls }))
        .await
}

async fn jmap(h: &Harness, calls: Value) -> Value {
    jmap_as(h, &h.account_id, calls).await
}

/// The arguments of the first response with this call id.
fn result<'a>(resp: &'a Value, call_id: &str) -> &'a Value {
    resp["methodResponses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r[2] == call_id)
        .map(|r| &r[1])
        .unwrap_or(&Value::Null)
}

async fn create_draft(h: &Harness) -> String {
    let resp = jmap(
        h,
        json!([["Email/set", { "create": { "draft": {
            "from": [{ "email": "me@example.org" }],
            "to": [{ "email": "friend@example.org" }],
            "subject": "Invoice", "bodyValues": { "1": { "value": "please pay once" } },
            "textBody": [{ "partId": "1", "type": "text/plain" }]
        } } }, "c1"]]),
    )
    .await;
    result(&resp, "c1")["created"]["draft"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("draft created: {resp}"))
        .to_string()
}

/// `EmailSubmission/set` with these arguments; returns the whole response.
async fn submission_set(h: &Harness, args: Value) -> Value {
    jmap(h, json!([["EmailSubmission/set", args, "s"]])).await
}

/// Create one submission (creation id `send`) and return its `created` entry.
async fn create_submission(h: &Harness, spec: Value) -> Value {
    let resp = submission_set(h, json!({ "create": { "send": spec } })).await;
    let created = result(&resp, "s")["created"]["send"].clone();
    assert!(created["id"].is_string(), "submission created: {resp}");
    created
}

async fn submission(h: &Harness, sub_id: &str) -> Value {
    let g = jmap(
        h,
        json!([["EmailSubmission/get", { "ids": [sub_id] }, "g"]]),
    )
    .await;
    result(&g, "g")["list"][0].clone()
}

/// The Email, or `Null` when it does not exist.
async fn email(h: &Harness, id: &str) -> Value {
    let g = jmap(
        h,
        json!([["Email/get", { "ids": [id], "properties": ["id", "keywords", "mailboxIds"] }, "g"]]),
    )
    .await;
    result(&g, "g")["list"][0].clone()
}

async fn mailbox_id(h: &Harness, role: &str) -> String {
    let g = jmap(h, json!([["Mailbox/get", { "ids": null }, "m"]])).await;
    result(&g, "m")["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == role)
        .and_then(|m| m["id"].as_str())
        .unwrap_or_else(|| panic!("a {role} mailbox: {g}"))
        .to_string()
}

async fn emails_in(h: &Harness, mailbox: &str) -> Vec<String> {
    let q = jmap(
        h,
        json!([["Email/query", { "filter": { "inMailbox": mailbox } }, "q"]]),
    )
    .await;
    result(&q, "q")["ids"]
        .as_array()
        .unwrap_or_else(|| panic!("Email/query: {q}"))
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

/// Dispatcher passes at a clock ten years on: far past any `sendAt`, any undo
/// window and any retry delay a test here sets.
async fn tick_long_after(h: &Harness) {
    for _ in 0..3 {
        h.engine
            .dispatch_tick_at(Utc::now() + chrono::Duration::days(3650))
            .await
            .unwrap();
    }
}

const RELEASE: fn() -> Value = || json!({ "sendAt": null, "mailwomanHoldSeconds": 0 });

async fn release(h: &Harness, sub_id: &str) -> Value {
    submission_set(h, json!({ "update": { sub_id: RELEASE() } })).await
}

/// Assert `notUpdated[id].type` and that the id is not under `updated`.
fn assert_not_updated(resp: &Value, id: &str, error_type: &str) {
    let set = result(resp, "s");
    assert!(set["updated"].get(id).is_none(), "{id} not updated: {resp}");
    assert_eq!(set["notUpdated"][id]["type"], error_type, "{resp}");
}

// ---- the manual hold -------------------------------------------------------

/// A manual hold is not sent by the create and not sent by the dispatcher at
/// any later time; a release sends it, once.
#[tokio::test]
async fn a_manual_hold_waits_for_a_release_and_is_then_sent_once() {
    let h = setup(Smtp::Deliver).await;
    let draft = create_draft(&h).await;
    assert_eq!(h.submitter.calls(), 0, "precondition: nothing sent yet");

    let created =
        create_submission(&h, json!({ "emailId": draft, "mailwomanHold": "manual" })).await;
    let sub_id = created["id"].as_str().unwrap();
    assert_eq!(created["undoStatus"], "pending");
    assert_eq!(created["mailwomanHold"], "manual");
    assert_eq!(h.submitter.calls(), 0, "the create did not send it");

    tick_long_after(&h).await;
    assert_eq!(
        h.submitter.calls(),
        0,
        "the dispatcher never sends a manual hold"
    );
    let held = submission(&h, sub_id).await;
    assert_eq!(held["undoStatus"], "pending");
    assert_eq!(held["mailwomanHold"], "manual");
    assert_eq!(held["mailwomanOrigin"], Value::Null);
    assert!(!email(&h, &draft).await.is_null(), "the draft is kept");

    let resp = release(&h, sub_id).await;
    assert_eq!(
        result(&resp, "s")["updated"][sub_id]["undoStatus"],
        "final",
        "{resp}"
    );
    assert_eq!(
        result(&resp, "s")["updated"][sub_id]["mailwomanHold"],
        Value::Null
    );
    assert_eq!(h.submitter.calls(), 1, "the release sent it");
    assert_eq!(h.submitter.delivered(), 1);

    tick_long_after(&h).await;
    assert_eq!(h.submitter.calls(), 1, "and nothing sends it again");
    let sent = submission(&h, sub_id).await;
    assert_eq!(sent["undoStatus"], "final");
    assert_eq!(sent["mailwomanHold"], Value::Null);

    // A second release of a sent submission is refused and sends nothing.
    let again = release(&h, sub_id).await;
    assert_not_updated(&again, sub_id, "serverFail");
    assert_eq!(h.submitter.calls(), 1);
}

/// `submit_held` is what the MCP mount calls: it holds, and records who asked.
/// A JMAP client can ask for a hold but cannot write an origin.
#[tokio::test]
async fn submit_held_holds_and_records_its_origin() {
    let h = setup(Smtp::Deliver).await;
    let draft = create_draft(&h).await;

    let sub_id = h
        .engine
        .submit_held(&h.account_id, &draft, "apiKey", "abcd1234")
        .await
        .unwrap();
    tick_long_after(&h).await;
    assert_eq!(h.submitter.calls(), 0, "held: neither sent nor due");
    let held = submission(&h, &sub_id).await;
    assert_eq!(held["undoStatus"], "pending");
    assert_eq!(held["mailwomanHold"], "manual");
    assert_eq!(
        held["mailwomanOrigin"],
        json!({ "kind": "apiKey", "name": "abcd1234" })
    );

    // The same through JMAP, with an origin in the spec: held, origin not taken.
    let other = create_draft(&h).await;
    let created = create_submission(
        &h,
        json!({
            "emailId": other,
            "mailwomanHold": "manual",
            "mailwomanOrigin": { "kind": "apiKey", "name": "forged" },
        }),
    )
    .await;
    let from_client = submission(&h, created["id"].as_str().unwrap()).await;
    assert_eq!(from_client["mailwomanHold"], "manual");
    assert_eq!(from_client["mailwomanOrigin"], Value::Null);

    // The origin survives the release, so a sent row still says where it came from.
    let resp = release(&h, &sub_id).await;
    assert_eq!(
        result(&resp, "s")["updated"][&sub_id]["undoStatus"],
        "final"
    );
    assert_eq!(h.submitter.calls(), 1);
    assert_eq!(
        submission(&h, &sub_id).await["mailwomanOrigin"],
        json!({ "kind": "apiKey", "name": "abcd1234" })
    );
}

/// A hold the engine does not know is refused, not read as "no hold".
#[tokio::test]
async fn an_unknown_hold_is_refused_and_nothing_is_sent() {
    let h = setup(Smtp::Deliver).await;
    let draft = create_draft(&h).await;
    for hold in [json!("review"), json!(true), json!(""), json!(1)] {
        let resp = submission_set(
            &h,
            json!({ "create": { "send": { "emailId": draft, "mailwomanHold": hold } } }),
        )
        .await;
        let set = result(&resp, "s");
        assert_eq!(
            set["notCreated"]["send"]["type"], "invalidProperties",
            "{resp}"
        );
        assert_eq!(
            set["notCreated"]["send"]["properties"],
            json!(["mailwomanHold"])
        );
        assert!(set["created"].get("send").is_none(), "{resp}");
    }
    assert_eq!(h.submitter.calls(), 0);
    // Control: without the property the same draft is sent at once.
    create_submission(&h, json!({ "emailId": draft })).await;
    assert_eq!(h.submitter.calls(), 1);
}

/// "Discard" on a held row is a cancel; a canceled row cannot be released.
#[tokio::test]
async fn a_held_submission_can_be_discarded_and_is_then_never_sent() {
    let h = setup(Smtp::Deliver).await;
    let draft = create_draft(&h).await;
    let created =
        create_submission(&h, json!({ "emailId": draft, "mailwomanHold": "manual" })).await;
    let sub_id = created["id"].as_str().unwrap();

    let cancel = submission_set(
        &h,
        json!({ "update": { sub_id: { "undoStatus": "canceled" } } }),
    )
    .await;
    assert!(
        result(&cancel, "s")["updated"].get(sub_id).is_some(),
        "{cancel}"
    );
    let row = submission(&h, sub_id).await;
    assert_eq!(row["undoStatus"], "canceled");
    assert_eq!(
        row["mailwomanHold"],
        Value::Null,
        "a canceled row is not held"
    );

    let resp = release(&h, sub_id).await;
    assert_not_updated(&resp, sub_id, "serverFail");
    tick_long_after(&h).await;
    assert_eq!(h.submitter.calls(), 0);
    assert!(!email(&h, &draft).await.is_null(), "the draft is kept");
}

// ---- release on the other waiting rows (the Outbox "Send now") -------------

/// The Outbox "Send now" sends `{sendAt: null, mailwomanHoldSeconds: 0}`. On a
/// scheduled row and on a row inside its undo window it sends the message now.
#[tokio::test]
async fn send_now_sends_a_scheduled_and_an_undo_window_submission() {
    let h = setup(Smtp::Deliver).await;

    let later = (Utc::now() + chrono::Duration::days(30)).to_rfc3339();
    let scheduled_draft = create_draft(&h).await;
    let scheduled =
        create_submission(&h, json!({ "emailId": scheduled_draft, "sendAt": later })).await;
    let window_draft = create_draft(&h).await;
    let window = create_submission(
        &h,
        json!({ "emailId": window_draft, "mailwomanHoldSeconds": 3600 }),
    )
    .await;
    h.engine.dispatch_tick().await.unwrap();
    assert_eq!(h.submitter.calls(), 0, "precondition: neither is due yet");

    for (n, created) in [scheduled, window].iter().enumerate() {
        let sub_id = created["id"].as_str().unwrap();
        let resp = release(&h, sub_id).await;
        let changed = &result(&resp, "s")["updated"][sub_id];
        assert_eq!(changed["undoStatus"], "final", "{resp}");
        assert_eq!(changed["sendAt"], Value::Null);
        assert_eq!(changed["mailwomanHoldSeconds"], 0);
        assert_eq!(h.submitter.calls(), n + 1, "sent by the release itself");
        let row = submission(&h, sub_id).await;
        assert_eq!(row["undoStatus"], "final");
        assert_eq!(row["sendAt"], Value::Null);
    }
    tick_long_after(&h).await;
    assert_eq!(h.submitter.calls(), 2, "each was sent once");
}

/// `{mailwomanHold: null}` is the other spelling of a release.
#[tokio::test]
async fn clearing_the_hold_property_releases() {
    let h = setup(Smtp::Deliver).await;
    let draft = create_draft(&h).await;
    let created =
        create_submission(&h, json!({ "emailId": draft, "mailwomanHold": "manual" })).await;
    let sub_id = created["id"].as_str().unwrap();
    let resp = submission_set(
        &h,
        json!({ "update": { sub_id: { "mailwomanHold": null } } }),
    )
    .await;
    assert_eq!(
        result(&resp, "s")["updated"][sub_id]["undoStatus"],
        "final",
        "{resp}"
    );
    assert_eq!(h.submitter.calls(), 1);
}

/// An update that is neither a cancel nor a release is refused by name, and
/// leaves the row as it was. Before 26.20 each of these came back `updated`.
#[tokio::test]
async fn an_update_that_is_neither_a_cancel_nor_a_release_is_refused() {
    let h = setup(Smtp::Deliver).await;
    let draft = create_draft(&h).await;
    let created =
        create_submission(&h, json!({ "emailId": draft, "mailwomanHold": "manual" })).await;
    let sub_id = created["id"].as_str().unwrap();
    let later = (Utc::now() + chrono::Duration::days(1)).to_rfc3339();

    for (patch, error_type) in [
        (json!({}), "invalidPatch"),
        (json!({ "sendAt": later }), "invalidProperties"),
        (json!({ "sendAt": null }), "invalidProperties"),
        (json!({ "mailwomanHoldSeconds": 0 }), "invalidProperties"),
        (
            json!({ "sendAt": null, "mailwomanHoldSeconds": 30 }),
            "invalidProperties",
        ),
        (json!({ "mailwomanHold": "manual" }), "invalidProperties"),
        (json!({ "undoStatus": "final" }), "invalidProperties"),
        (json!({ "undoStatus": "pending" }), "invalidProperties"),
        (
            json!({ "undoStatus": "canceled", "mailwomanHold": null }),
            "invalidProperties",
        ),
        (json!({ "emailId": "other" }), "invalidProperties"),
        (json!({ "mailwomanOrigin": null }), "invalidProperties"),
    ] {
        let resp = submission_set(&h, json!({ "update": { sub_id: patch.clone() } })).await;
        assert_not_updated(&resp, sub_id, error_type);
        let row = submission(&h, sub_id).await;
        assert_eq!(row["undoStatus"], "pending", "after {patch}");
        assert_eq!(row["mailwomanHold"], "manual", "after {patch}");
    }
    tick_long_after(&h).await;
    assert_eq!(h.submitter.calls(), 0);
}

/// A release whose send SMTP does not accept is still a release: the row is no
/// longer held, it carries the failure, and the dispatcher retries it after the
/// backoff exactly as it would any other row.
#[tokio::test]
async fn a_release_smtp_does_not_accept_backs_off_and_is_retried() {
    let h = setup(Smtp::TransportError).await;
    let draft = create_draft(&h).await;
    let created =
        create_submission(&h, json!({ "emailId": draft, "mailwomanHold": "manual" })).await;
    let sub_id = created["id"].as_str().unwrap();

    let resp = release(&h, sub_id).await;
    let changed = &result(&resp, "s")["updated"][sub_id];
    assert_eq!(changed["undoStatus"], "pending", "{resp}");
    assert_eq!(changed["mailwomanHold"], Value::Null);
    assert_eq!(changed["mailwomanAttempts"], 1);
    assert!(
        changed["mailwomanLastError"]
            .as_str()
            .unwrap()
            .contains("refused")
    );
    assert!(changed["mailwomanNextAttemptAt"].is_string());
    assert_eq!((h.submitter.calls(), h.submitter.delivered()), (1, 0));

    // Inside the backoff nothing is attempted.
    h.engine.dispatch_tick().await.unwrap();
    assert_eq!(h.submitter.calls(), 1);

    h.submitter.set_mode(Smtp::Deliver);
    tick_long_after(&h).await;
    assert_eq!((h.submitter.calls(), h.submitter.delivered()), (2, 1));
    assert_eq!(submission(&h, sub_id).await["undoStatus"], "final");
}

/// Another account can neither release nor cancel this account's submission:
/// to it the id does not exist.
#[tokio::test]
async fn another_account_cannot_release_or_cancel_a_submission() {
    let h = setup(Smtp::Deliver).await;
    let other_submitter = CountingSubmitter::new(Smtp::Deliver);
    let other = add_account(
        &h.engine,
        "other@example.org",
        Arc::new(FakeBackend::default()),
        other_submitter.clone(),
    )
    .await;
    let draft = create_draft(&h).await;
    let created =
        create_submission(&h, json!({ "emailId": draft, "mailwomanHold": "manual" })).await;
    let sub_id = created["id"].as_str().unwrap();

    for patch in [RELEASE(), json!({ "undoStatus": "canceled" })] {
        let resp = jmap_as(
            &h,
            &other,
            json!([["EmailSubmission/set", { "update": { sub_id: patch } }, "s"]]),
        )
        .await;
        assert_not_updated(&resp, sub_id, "serverFail");
        let why = result(&resp, "s")["notUpdated"][sub_id]["description"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(why.contains("unknown submission"), "{why}");
    }
    // Nor can it submit this account's draft as its own.
    let resp = jmap_as(
        &h,
        &other,
        json!([["EmailSubmission/set", { "create": { "send": { "emailId": draft } } }, "s"]]),
    )
    .await;
    assert_eq!(
        result(&resp, "s")["notCreated"]["send"]["type"],
        "invalidProperties",
        "{resp}"
    );
    assert_eq!((h.submitter.calls(), other_submitter.calls()), (0, 0));
    let row = submission(&h, sub_id).await;
    assert_eq!(
        (&row["undoStatus"], &row["mailwomanHold"]),
        (&json!("pending"), &json!("manual"))
    );

    // Control: the owner can.
    release(&h, sub_id).await;
    assert_eq!(h.submitter.calls(), 1);
}

// ---- one sender per row ----------------------------------------------------

/// While an inline send is waiting on SMTP its row is `pending` and due. A
/// dispatcher pass in that window must not hand the same message to SMTP.
#[tokio::test]
async fn the_dispatcher_does_not_send_a_row_an_inline_send_is_sending() {
    let h = setup(Smtp::DeliverWhenLetGo).await;
    let draft = create_draft(&h).await;

    let (engine, account_id) = (h.engine.clone(), h.account_id.clone());
    let inline = tokio::spawn(async move {
        engine
            .handle_jmap(
                &account_id,
                &json!({ "methodCalls": [
                    ["EmailSubmission/set", { "create": { "send": { "emailId": draft } } }, "s"]
                ] }),
            )
            .await
    });
    for _ in 0..500 {
        if h.submitter.calls() == 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(h.submitter.calls(), 1, "precondition: SMTP is in flight");

    // A second sender would enter `submit` and wait there with the first.
    let pass = tokio::time::timeout(Duration::from_secs(2), h.engine.dispatch_tick()).await;
    assert!(
        pass.is_ok(),
        "the dispatcher pass did not start a send of its own"
    );
    assert_eq!(h.submitter.calls(), 1, "one message handed to SMTP");

    h.submitter.let_go.notify_waiters();
    let resp = inline.await.unwrap();
    assert_eq!(
        result(&resp, "s")["created"]["send"]["undoStatus"],
        "final",
        "{resp}"
    );
    h.engine.dispatch_tick().await.unwrap();
    assert_eq!((h.submitter.calls(), h.submitter.delivered()), (1, 1));
}

// ---- onSuccessUpdateEmail / onSuccessDestroyEmail (RFC 8621 §7.5) ----------

/// What the web client sends with a submission: file into Sent, clear `$draft`.
fn sent_patch(sent: &str) -> Value {
    json!({ "mailboxIds": { sent: true }, "keywords/$draft": null })
}

async fn assert_is_a_draft_in(h: &Harness, id: &str, drafts: &str, context: &str) {
    let e = email(h, id).await;
    assert_eq!(e["keywords"]["$draft"], true, "{context}: has $draft: {e}");
    assert_eq!(
        e["mailboxIds"],
        json!({ drafts: true }),
        "{context}: in Drafts"
    );
}

async fn assert_is_sent_in(h: &Harness, id: &str, sent: &str, context: &str) {
    let e = email(h, id).await;
    assert!(!e.is_null(), "{context}: the Email keeps its id");
    assert!(
        e["keywords"].get("$draft").is_none(),
        "{context}: no $draft: {e}"
    );
    assert_eq!(e["mailboxIds"], json!({ sent: true }), "{context}: in Sent");
}

/// The send the web client makes: after it the draft — the same Email id — has
/// no `$draft` and is in Sent, the mail server has a copy in Sent, and the
/// response carries the implicit `Email/set`.
#[tokio::test]
async fn on_success_update_email_files_the_sent_draft() {
    let h = setup(Smtp::Deliver).await;
    let (drafts, sent) = (mailbox_id(&h, "drafts").await, mailbox_id(&h, "sent").await);
    let draft = create_draft(&h).await;
    assert_is_a_draft_in(&h, &draft, &drafts, "precondition").await;
    assert!(
        emails_in(&h, &sent).await.is_empty(),
        "precondition: Sent is empty"
    );

    let resp = submission_set(
        &h,
        json!({
            "create": { "send": { "emailId": draft } },
            "onSuccessUpdateEmail": { "#send": sent_patch(&sent) },
        }),
    )
    .await;
    assert_eq!(
        result(&resp, "s")["created"]["send"]["undoStatus"],
        "final",
        "{resp}"
    );
    assert_eq!(h.submitter.delivered(), 1);

    assert_is_sent_in(&h, &draft, &sent, "after the send").await;
    assert_eq!(
        emails_in(&h, &sent).await,
        vec![draft.clone()],
        "one message in Sent"
    );
    assert!(
        emails_in(&h, &drafts).await.is_empty(),
        "none left in Drafts"
    );
    let upstream = h.backend.appended_to("Sent");
    assert_eq!(upstream.len(), 1, "one copy filed on the mail server");
    assert!(!upstream[0].contains(&Flag::Draft), "and not as a draft");

    // The implicit Email/set response follows the method's own, same call id.
    let responses = resp["methodResponses"].as_array().unwrap();
    assert_eq!(responses.len(), 2, "{resp}");
    assert_eq!(responses[1][0], "Email/set");
    assert_eq!(responses[1][2], "s");
    assert!(responses[1][1]["updated"].get(&draft).is_some(), "{resp}");
    assert_ne!(responses[1][1]["oldState"], responses[1][1]["newState"]);
}

/// A send SMTP does not accept leaves the draft exactly as it was.
#[tokio::test]
async fn a_failed_send_leaves_the_draft_untouched() {
    let h = setup(Smtp::TransportError).await;
    let (drafts, sent) = (mailbox_id(&h, "drafts").await, mailbox_id(&h, "sent").await);
    let draft = create_draft(&h).await;
    assert_is_a_draft_in(&h, &draft, &drafts, "precondition").await;

    let resp = submission_set(
        &h,
        json!({
            "create": { "send": { "emailId": draft } },
            "onSuccessUpdateEmail": { "#send": sent_patch(&sent) },
        }),
    )
    .await;
    assert!(
        result(&resp, "s")["notCreated"].get("send").is_some(),
        "{resp}"
    );
    assert_eq!((h.submitter.calls(), h.submitter.delivered()), (1, 0));
    assert_eq!(
        resp["methodResponses"].as_array().unwrap().len(),
        1,
        "no implicit Email/set"
    );

    assert_is_a_draft_in(&h, &draft, &drafts, "after the failed send").await;
    assert!(emails_in(&h, &sent).await.is_empty());
    assert!(h.backend.appended_to("Sent").is_empty());
}

/// For a held submission the patch is applied when the message is sent — at
/// the release — and not when the submission is created.
#[tokio::test]
async fn a_held_submission_is_filed_at_release_not_at_enqueue() {
    let h = setup(Smtp::Deliver).await;
    let (drafts, sent) = (mailbox_id(&h, "drafts").await, mailbox_id(&h, "sent").await);
    let draft = create_draft(&h).await;

    let resp = submission_set(
        &h,
        json!({
            "create": { "send": { "emailId": draft, "mailwomanHold": "manual" } },
            "onSuccessUpdateEmail": { "#send": sent_patch(&sent) },
        }),
    )
    .await;
    let sub_id = result(&resp, "s")["created"]["send"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        resp["methodResponses"].as_array().unwrap().len(),
        1,
        "nothing changed yet"
    );
    tick_long_after(&h).await;
    assert_is_a_draft_in(&h, &draft, &drafts, "while held").await;
    assert_eq!(h.submitter.calls(), 0);

    let released = release(&h, &sub_id).await;
    assert_eq!(h.submitter.delivered(), 1);
    assert_is_sent_in(&h, &draft, &sent, "after the release").await;
    let responses = released["methodResponses"].as_array().unwrap();
    assert_eq!(
        responses.len(),
        2,
        "the release reports the Email change: {released}"
    );
    assert!(responses[1][1]["updated"].get(&draft).is_some());
}

/// For a delayed submission the patch is applied by the dispatcher pass that
/// sends it.
#[tokio::test]
async fn a_delayed_submission_is_filed_when_the_dispatcher_sends_it() {
    let h = setup(Smtp::Deliver).await;
    let (drafts, sent) = (mailbox_id(&h, "drafts").await, mailbox_id(&h, "sent").await);
    let draft = create_draft(&h).await;

    submission_set(
        &h,
        json!({
            "create": { "send": { "emailId": draft, "mailwomanHoldSeconds": 30 } },
            "onSuccessUpdateEmail": { "#send": sent_patch(&sent) },
        }),
    )
    .await;
    h.engine.dispatch_tick().await.unwrap();
    assert_eq!(
        h.submitter.calls(),
        0,
        "precondition: inside the undo window"
    );
    assert_is_a_draft_in(&h, &draft, &drafts, "inside the undo window").await;

    tick_long_after(&h).await;
    assert_eq!(h.submitter.delivered(), 1);
    assert_is_sent_in(&h, &draft, &sent, "after the dispatcher sent it").await;
}

/// `onSuccessDestroyEmail`: the Email is removed after the send and no copy is
/// kept; a failed send removes nothing.
#[tokio::test]
async fn on_success_destroy_email_removes_the_message_only_after_a_send() {
    let h = setup(Smtp::TransportError).await;
    let sent = mailbox_id(&h, "sent").await;
    let draft = create_draft(&h).await;
    let args = json!({
        "create": { "send": { "emailId": draft } },
        "onSuccessDestroyEmail": ["#send"],
    });

    submission_set(&h, args.clone()).await;
    assert_eq!(h.submitter.delivered(), 0);
    assert!(
        !email(&h, &draft).await.is_null(),
        "not sent, so not destroyed"
    );

    h.submitter.set_mode(Smtp::Deliver);
    let resp = submission_set(&h, args).await;
    assert_eq!(h.submitter.delivered(), 1);
    assert!(
        email(&h, &draft).await.is_null(),
        "destroyed after the send"
    );
    assert!(emails_in(&h, &sent).await.is_empty(), "and no copy kept");
    let responses = resp["methodResponses"].as_array().unwrap();
    assert_eq!(responses[1][1]["destroyed"], json!([draft]), "{resp}");
}

/// Without either argument the engine files as it did before: a copy in Sent,
/// the draft removed.
#[tokio::test]
async fn without_on_success_a_copy_is_filed_and_the_draft_removed() {
    let h = setup(Smtp::Deliver).await;
    let sent = mailbox_id(&h, "sent").await;
    let draft = create_draft(&h).await;
    let resp = submission_set(&h, json!({ "create": { "send": { "emailId": draft } } })).await;
    assert_eq!(resp["methodResponses"].as_array().unwrap().len(), 1);
    assert!(email(&h, &draft).await.is_null(), "the draft is removed");
    let in_sent = emails_in(&h, &sent).await;
    assert_eq!(in_sent.len(), 1, "a copy is in Sent");
    assert_ne!(in_sent[0], draft);
}

/// An `onSuccess*` argument that could not be carried out is refused with the
/// create, before anything is sent — not discovered after the message has gone.
#[tokio::test]
async fn an_on_success_instruction_that_cannot_be_applied_refuses_the_create() {
    let h = setup(Smtp::Deliver).await;
    let sent = mailbox_id(&h, "sent").await;
    let draft = create_draft(&h).await;

    for (args, context) in [
        (
            json!({ "onSuccessUpdateEmail": { "#send": { "subject": "changed" } } }),
            "a property Email/set update cannot apply",
        ),
        (
            json!({ "onSuccessUpdateEmail": { "#send": { "mailboxIds": { "no-such-mailbox": true } } } }),
            "a mailbox that does not exist",
        ),
        (
            json!({
                "onSuccessUpdateEmail": { "#send": sent_patch(&sent) },
                "onSuccessDestroyEmail": ["#send"],
            }),
            "both update and destroy",
        ),
    ] {
        let mut args = args;
        args["create"] = json!({ "send": { "emailId": draft } });
        let resp = submission_set(&h, args).await;
        let set = result(&resp, "s");
        assert_eq!(
            set["notCreated"]["send"]["properties"],
            json!(["onSuccessUpdateEmail"]),
            "{context}: {resp}"
        );
        assert!(set["created"].get("send").is_none(), "{context}");
    }
    assert_eq!(h.submitter.calls(), 0, "nothing was sent");
    assert!(!email(&h, &draft).await.is_null());

    // A cancel cannot carry one either: it is applied at send time only.
    let created =
        create_submission(&h, json!({ "emailId": draft, "mailwomanHold": "manual" })).await;
    let sub_id = created["id"].as_str().unwrap();
    let resp = submission_set(
        &h,
        json!({
            "update": { sub_id: { "undoStatus": "canceled" } },
            "onSuccessUpdateEmail": { sub_id: sent_patch(&sent) },
        }),
    )
    .await;
    assert_not_updated(&resp, sub_id, "invalidArguments");
    assert_eq!(submission(&h, sub_id).await["undoStatus"], "pending");
}
