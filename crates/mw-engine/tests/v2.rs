//! V2 engine acceptance tests (plan §3 e9) over a deterministic in-process
//! [`FakeBackend`]: full-text search operators, real state + `Email/changes`,
//! the undo-send queue (cancel-before-window + dispatcher fire-after-window),
//! snooze resurface, stable-id-preserving move with index/meta re-key, rule
//! move at ingest, identities, and the realtime broadcast.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, Flag, MailboxDelta, MailboxRole, MessageRef,
    MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor, WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_sieve::{Action, Condition, MatchOp, Rule, StringTest};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store, SubmissionRow};

const UIDVALIDITY: u32 = 100;

type ScriptMsg = (u32, Vec<u8>, Vec<Flag>, String);

struct Scripted {
    mailboxes: Vec<(String, MailboxRole)>,
    messages: HashMap<String, Vec<ScriptMsg>>,
}

struct FakeBackend {
    state: Mutex<Scripted>,
}

impl FakeBackend {
    fn new() -> Self {
        let mailboxes = vec![
            ("INBOX".to_string(), MailboxRole::Inbox),
            ("Archive".to_string(), MailboxRole::Archive),
            ("Sent".to_string(), MailboxRole::Sent),
            ("Drafts".to_string(), MailboxRole::Drafts),
        ];
        let mut messages: HashMap<String, Vec<ScriptMsg>> = HashMap::new();
        messages.insert(
            "INBOX".to_string(),
            vec![
                (1, invoice_msg(), vec![], "2026-07-01T09:00:00Z".into()),
                (2, lunch_msg(), vec![], "2026-07-02T09:00:00Z".into()),
                (3, report_msg(), vec![], "2026-07-03T09:00:00Z".into()),
            ],
        );
        Self {
            state: Mutex::new(Scripted {
                mailboxes,
                messages,
            }),
        }
    }
}

#[async_trait]
impl AccountBackend for FakeBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: true,
            r#move: true,
            special_use: true,
            ..BackendCaps::default()
        })
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        let st = self.state.lock().unwrap();
        Ok(st
            .mailboxes
            .iter()
            .map(|(name, role)| {
                let total = st.messages.get(name).map(|m| m.len()).unwrap_or(0) as u32;
                RawMailbox {
                    mailbox_ref: RawMailboxRef {
                        name: name.clone(),
                        uidvalidity: UIDVALIDITY,
                    },
                    role: *role,
                    parent: None,
                    uidnext: total + 1,
                    highestmodseq: 0,
                    total,
                    unread: total,
                }
            })
            .collect())
    }

    async fn sync_mailbox(
        &self,
        mbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        let uidnext_from = match cursor {
            SyncCursor::UidWindow { uidnext, .. } => *uidnext,
            _ => 1,
        };
        let st = self.state.lock().unwrap();
        let msgs = st.messages.get(&mbox.name).cloned().unwrap_or_default();
        let added: Vec<MessageRef> = msgs
            .iter()
            .filter(|(uid, ..)| *uid >= uidnext_from)
            .map(|(uid, ..)| MessageRef::Imap {
                mailbox: mbox.clone(),
                uidvalidity: UIDVALIDITY,
                uid: *uid,
            })
            .collect();
        let max_uid = msgs.iter().map(|(u, ..)| *u).max().unwrap_or(0);
        Ok(MailboxDelta {
            added,
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: UIDVALIDITY,
                uidnext: max_uid + 1,
            },
        })
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        let st = self.state.lock().unwrap();
        let mut out = Vec::new();
        for r in refs {
            let MessageRef::Imap { mailbox, uid, .. } = r else {
                continue;
            };
            if let Some(msgs) = st.messages.get(&mailbox.name)
                && let Some((_, raw, flags, internaldate)) = msgs.iter().find(|(u, ..)| u == uid)
            {
                out.push(RawMessage {
                    message_ref: r.clone(),
                    raw: raw.clone(),
                    flags: flags.clone(),
                    internaldate: Some(internaldate.clone()),
                });
            }
        }
        Ok(out)
    }

    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _rm: &[Flag]) -> Result<()> {
        Ok(())
    }

    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Ok(MoveOutcome::Uidplus {
            uidvalidity: UIDVALIDITY,
            uids: vec![9001],
        })
    }

    async fn append(&self, mbox: &RawMailboxRef, raw: &[u8], flags: &[Flag]) -> Result<MessageRef> {
        let mut st = self.state.lock().unwrap();
        let entry = st.messages.entry(mbox.name.clone()).or_default();
        let uid = entry.iter().map(|(u, ..)| *u).max().unwrap_or(0) + 1;
        entry.push((
            uid,
            raw.to_vec(),
            flags.to_vec(),
            "2026-07-10T09:00:00Z".into(),
        ));
        Ok(MessageRef::Imap {
            mailbox: mbox.clone(),
            uidvalidity: UIDVALIDITY,
            uid,
        })
    }

    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

struct FakeSubmitter {
    calls: AtomicUsize,
}

#[async_trait]
impl MailSubmitter for FakeSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

fn invoice_msg() -> Vec<u8> {
    concat!(
        "Message-ID: <invoice@x>\r\n",
        "From: alice@example.org\r\n",
        "To: me@example.org\r\n",
        "Subject: Invoice 2026\r\n",
        "Date: Wed, 01 Jul 2026 09:00:00 +0000\r\n",
        "MIME-Version: 1.0\r\n",
        "Content-Type: multipart/mixed; boundary=\"b1\"\r\n",
        "\r\n",
        "--b1\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "Please pay the attached invoice\r\n",
        "--b1\r\n",
        "Content-Type: application/pdf; name=\"invoice.pdf\"\r\n",
        "Content-Disposition: attachment; filename=\"invoice.pdf\"\r\n",
        "Content-Transfer-Encoding: base64\r\n",
        "\r\n",
        "JVBERi0xLjQK\r\n",
        "--b1--\r\n",
    )
    .as_bytes()
    .to_vec()
}

fn lunch_msg() -> Vec<u8> {
    concat!(
        "Message-ID: <lunch@x>\r\n",
        "From: bob@example.org\r\n",
        "To: me@example.org\r\n",
        "Subject: Lunch\r\n",
        "Date: Thu, 02 Jul 2026 09:00:00 +0000\r\n",
        "\r\n",
        "noon works for me\r\n",
    )
    .as_bytes()
    .to_vec()
}

fn report_msg() -> Vec<u8> {
    concat!(
        "Message-ID: <report@x>\r\n",
        "From: carol@example.org\r\n",
        "To: me@example.org\r\n",
        "Subject: Weekly Report\r\n",
        "Date: Fri, 03 Jul 2026 09:00:00 +0000\r\n",
        "\r\n",
        "weekly numbers inside\r\n",
    )
    .as_bytes()
    .to_vec()
}

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
    submitter: Arc<FakeSubmitter>,
}

async fn setup() -> Harness {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = store
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
        .unwrap();
    let engine = Arc::new(Engine::new(store));
    let submitter = Arc::new(FakeSubmitter {
        calls: AtomicUsize::new(0),
    });
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(FakeBackend::new()) as Arc<dyn AccountBackend>,
            submitter.clone() as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    Harness {
        engine,
        account_id,
        submitter,
    }
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

fn mailbox_id(resp: &Value, role: &str) -> String {
    result(resp, "mb")["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == role)
        .expect("mailbox with role")["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn inbox(h: &Harness) -> String {
    let mb = jmap(h, json!([["Mailbox/get", {}, "mb"]])).await;
    mailbox_id(&mb, "inbox")
}

/// The single id matching a search filter, or a panic if not exactly one.
async fn search_one(h: &Harness, inbox: &str, filter: Value) -> String {
    let mut f = filter;
    f["inMailbox"] = json!(inbox);
    let resp = jmap(h, json!([["Email/query", { "filter": f }, "q"]])).await;
    let ids = result(&resp, "q")["ids"].as_array().unwrap();
    assert_eq!(ids.len(), 1, "expected exactly one hit, got {ids:?}");
    ids[0].as_str().unwrap().to_string()
}

async fn subject_of(h: &Harness, id: &str) -> String {
    let g = jmap(h, json!([["Email/get", { "ids": [id] }, "g"]])).await;
    result(&g, "g")["list"][0]["subject"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

#[tokio::test]
async fn search_operators_return_correct_ids() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let inbox = inbox(&h).await;

    // from:, subject:, body:, has:attachment, filename: each isolate the message.
    let a = search_one(&h, &inbox, json!({ "from": "alice" })).await;
    assert_eq!(subject_of(&h, &a).await, "Invoice 2026");
    assert_eq!(
        subject_of(
            &h,
            &search_one(&h, &inbox, json!({ "subject": "invoice" })).await
        )
        .await,
        "Invoice 2026"
    );
    assert_eq!(
        subject_of(&h, &search_one(&h, &inbox, json!({ "body": "noon" })).await).await,
        "Lunch"
    );
    assert_eq!(
        subject_of(
            &h,
            &search_one(&h, &inbox, json!({ "hasAttachment": true })).await
        )
        .await,
        "Invoice 2026"
    );
    assert_eq!(
        subject_of(
            &h,
            &search_one(&h, &inbox, json!({ "filename": "invoice" })).await
        )
        .await,
        "Invoice 2026"
    );
}

#[tokio::test]
async fn state_advances_and_email_changes_diff_is_correct() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let inbox = inbox(&h).await;

    // Everything since state 0 was created.
    let c0 = jmap(&h, json!([["Email/changes", { "sinceState": "0" }, "c"]])).await;
    assert_eq!(result(&c0, "c")["created"].as_array().unwrap().len(), 3);
    assert!(result(&c0, "c")["destroyed"].as_array().unwrap().is_empty());

    // Capture the post-resync query state, then flag one message read.
    let q = jmap(
        &h,
        json!([["Email/query", { "filter": { "inMailbox": inbox } }, "q"]]),
    )
    .await;
    let s1 = result(&q, "q")["queryState"].as_str().unwrap().to_string();
    let id = result(&q, "q")["ids"][0].as_str().unwrap().to_string();

    jmap(
        &h,
        json!([["Email/set", { "update": { id.clone(): { "keywords": { "$seen": true } } } }, "s"]]),
    )
    .await;

    // Since s1, exactly that id is updated (nothing created/destroyed).
    let c1 = jmap(&h, json!([["Email/changes", { "sinceState": s1 }, "c"]])).await;
    let updated = result(&c1, "c")["updated"].as_array().unwrap();
    assert_eq!(updated, &vec![json!(id)]);
    assert!(result(&c1, "c")["created"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn fetch_blob_serves_whole_message_and_attachment_parts() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let inbox = inbox(&h).await;

    // The invoice message is the one carrying the base64 PDF attachment.
    let id = search_one(&h, &inbox, json!({ "hasAttachment": true })).await;

    // Email/get emits the message blobId (= stableId) and an attachment part
    // whose blobId is scoped as `<stableId>.<partId>`.
    let g = jmap(
        &h,
        json!([[
            "Email/get",
            { "ids": [id.clone()], "properties": ["blobId", "attachments"] },
            "g"
        ]]),
    )
    .await;
    let email = &result(&g, "g")["list"][0];
    assert_eq!(email["blobId"].as_str(), Some(id.as_str()));
    let atts = email["attachments"].as_array().expect("attachments array");
    assert_eq!(atts.len(), 1, "one non-inline attachment: {atts:?}");
    let att_blob = atts[0]["blobId"].as_str().unwrap();
    assert!(
        att_blob.starts_with(&format!("{id}.")),
        "attachment blobId scoped to the message: {att_blob}"
    );
    assert_eq!(atts[0]["name"].as_str(), Some("invoice.pdf"));

    // Downloading the attachment blobId decodes the base64 PDF body.
    let part = h
        .engine
        .fetch_blob(&h.account_id, att_blob)
        .await
        .unwrap()
        .expect("attachment blob resolves");
    assert_eq!(part.content_type, "application/pdf");
    assert_eq!(part.filename, "invoice.pdf");
    assert_eq!(part.bytes, b"%PDF-1.4\n");

    // Downloading the message blobId returns RFC822 that re-parses.
    let whole = h
        .engine
        .fetch_blob(&h.account_id, &id)
        .await
        .unwrap()
        .expect("message blob resolves");
    assert_eq!(whole.content_type, "message/rfc822");
    let reparsed = mw_mime::parse(&whole.bytes).expect("re-parse exported RFC822");
    assert_eq!(reparsed.email.subject.as_deref(), Some("Invoice 2026"));

    // An unknown blobId and a foreign account both resolve to nothing (→ 404).
    assert!(
        h.engine
            .fetch_blob(&h.account_id, "0000deadbeef")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        h.engine
            .fetch_blob("some-other-account", &id)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn undo_send_cancel_before_window() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();

    // Compose a draft, submit with a long hold → stays pending, nothing sent.
    let resp = jmap(
        &h,
        json!([
            ["Email/set", { "create": { "draft": {
                "from": [{ "email": "me@example.org" }],
                "to": [{ "email": "friend@example.org" }],
                "subject": "Later", "bodyValues": { "1": { "value": "hi" } },
                "textBody": [{ "partId": "1", "type": "text/plain" }]
            } } }, "c1"],
            ["EmailSubmission/set", { "create": { "s1": {
                "emailId": "#draft", "mailwomanHoldSeconds": 3600
            } } }, "c2"]
        ]),
    )
    .await;
    let sub_id = result(&resp, "c2")["created"]["s1"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        result(&resp, "c2")["created"]["s1"]["undoStatus"],
        "pending"
    );
    assert_eq!(
        h.submitter.calls.load(Ordering::SeqCst),
        0,
        "hold must defer send"
    );

    // Cancel within the window.
    let cancel = jmap(
        &h,
        json!([["EmailSubmission/set", { "update": { sub_id.clone(): { "undoStatus": "canceled" } } }, "u"]]),
    )
    .await;
    assert!(
        result(&cancel, "u")["updated"]
            .as_object()
            .unwrap()
            .contains_key(&sub_id)
    );

    // Dispatcher runs but must NOT send a canceled submission.
    h.engine.dispatch_tick().await.unwrap();
    assert_eq!(h.submitter.calls.load(Ordering::SeqCst), 0);

    // The Outbox shows it canceled.
    let g = jmap(
        &h,
        json!([["EmailSubmission/get", { "ids": [sub_id] }, "g"]]),
    )
    .await;
    assert_eq!(result(&g, "g")["list"][0]["undoStatus"], "canceled");
}

#[tokio::test]
async fn undo_send_fires_after_window() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let mut rx = h.engine.subscribe();

    // Create a draft, then enqueue a due-in-the-past pending submission directly
    // (bypassing the immediate-send path) so the dispatcher is what fires it.
    let resp = jmap(
        &h,
        json!([["Email/set", { "create": { "draft": {
            "from": [{ "email": "me@example.org" }],
            "to": [{ "email": "friend@example.org" }],
            "subject": "Scheduled", "bodyValues": { "1": { "value": "hi" } },
            "textBody": [{ "partId": "1", "type": "text/plain" }]
        } } }, "c1"]]),
    )
    .await;
    let draft = result(&resp, "c1")["created"]["draft"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    h.engine
        .store()
        .insert_submission(&SubmissionRow {
            id: "sub-due".into(),
            account_id: h.account_id.clone(),
            email_id: draft.clone(),
            identity_id: None,
            send_at: None,
            undo_status: "pending".into(),
            hold_seconds: 0,
            created_at: "2000-01-01T00:00:00Z".into(),
        })
        .await
        .unwrap();

    h.engine.dispatch_tick().await.unwrap();
    assert_eq!(
        h.submitter.calls.load(Ordering::SeqCst),
        1,
        "due submission must fire"
    );

    // Marked final; the draft is gone and the sent copy is in Sent.
    let g = jmap(
        &h,
        json!([["EmailSubmission/get", { "ids": ["sub-due"] }, "g"]]),
    )
    .await;
    assert_eq!(result(&g, "g")["list"][0]["undoStatus"], "final");
    let mb = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let sent = mailbox_id(&mb, "sent");
    let q = jmap(
        &h,
        json!([["Email/query", { "filter": { "inMailbox": sent } }, "q"]]),
    )
    .await;
    assert_eq!(result(&q, "q")["ids"].as_array().unwrap().len(), 1);

    // The realtime broadcast fired for this account.
    let sc = rx.try_recv().expect("a StateChange was broadcast");
    assert_eq!(sc.account_id, h.account_id);
}

#[tokio::test]
async fn snooze_resurfaces_when_due() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let inbox = inbox(&h).await;
    let id = search_one(&h, &inbox, json!({ "from": "bob" })).await;

    // Snooze into the past → the dispatcher should resurface (clear) it.
    jmap(
        &h,
        json!([["Email/set", { "update": { id.clone(): { "snoozedUntil": "2000-01-01T00:00:00Z" } } }, "s"]]),
    )
    .await;
    let before = jmap(&h, json!([["Email/get", { "ids": [id.clone()] }, "g"]])).await;
    assert_eq!(
        result(&before, "g")["list"][0]["snoozedUntil"],
        "2000-01-01T00:00:00Z"
    );

    h.engine.dispatch_tick().await.unwrap();

    let after = jmap(&h, json!([["Email/get", { "ids": [id] }, "g"]])).await;
    assert!(result(&after, "g")["list"][0]["snoozedUntil"].is_null());
}

#[tokio::test]
async fn relocate_preserves_id_and_rekeys_index_and_meta() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let mb = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let inbox = mailbox_id(&mb, "inbox");
    let archive = mailbox_id(&mb, "archive");
    let id = search_one(&h, &inbox, json!({ "from": "alice" })).await;

    // Pin it, then move it to Archive.
    jmap(
        &h,
        json!([["Email/set", { "update": { id.clone(): { "pinned": true } } }, "p"]]),
    )
    .await;
    jmap(
        &h,
        json!([["Email/set", { "update": { id.clone(): { "mailboxIds": { archive.clone(): true } } } }, "m"]]),
    )
    .await;

    // Same id now lives in Archive, gone from Inbox — the id is preserved.
    let after = jmap(
        &h,
        json!([
            ["Email/query", { "filter": { "inMailbox": inbox } }, "qi"],
            ["Email/query", { "filter": { "inMailbox": archive } }, "qa"]
        ]),
    )
    .await;
    let inbox_ids = result(&after, "qi")["ids"].as_array().unwrap();
    let arch_ids = result(&after, "qa")["ids"].as_array().unwrap();
    assert!(!inbox_ids.iter().any(|v| v == &json!(id)));
    assert!(arch_ids.iter().any(|v| v == &json!(id)));

    // The pin (meta) survived the move.
    let g = jmap(&h, json!([["Email/get", { "ids": [id.clone()] }, "g"]])).await;
    assert_eq!(result(&g, "g")["list"][0]["pinned"], true);

    // The search index was re-keyed onto Archive: a from-search scoped to
    // Archive finds it; the same search scoped to Inbox does not.
    let in_arch = search_one(&h, &archive, json!({ "from": "alice" })).await;
    assert_eq!(in_arch, id);
    let inbox_hit = jmap(
        &h,
        json!([["Email/query", { "filter": { "inMailbox": inbox, "from": "alice" } }, "q"]]),
    )
    .await;
    assert!(
        result(&inbox_hit, "q")["ids"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn rule_moves_message_at_ingest() {
    let h = setup().await;
    // Rule: subject contains "Report" → move to Archive. Set before the first
    // sync so the matching seeded message is filed on delivery.
    let rule = Rule {
        id: "r1".into(),
        name: "reports".into(),
        match_all: true,
        conditions: vec![Condition::Subject(StringTest {
            op: MatchOp::Contains,
            value: "report".into(),
        })],
        actions: vec![Action::Move {
            mailbox: "archive".into(),
        }],
        enabled: true,
    };
    h.engine.set_rules(&h.account_id, &[rule]).await.unwrap();
    h.engine.resync(&h.account_id).await.unwrap();

    let mb = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let inbox = mailbox_id(&mb, "inbox");
    let archive = mailbox_id(&mb, "archive");

    // The Report message landed in Archive; the other two stayed in Inbox.
    let q = jmap(
        &h,
        json!([
            ["Email/query", { "filter": { "inMailbox": inbox } }, "qi"],
            ["Email/query", { "filter": { "inMailbox": archive } }, "qa"]
        ]),
    )
    .await;
    assert_eq!(result(&q, "qi")["ids"].as_array().unwrap().len(), 2);
    let arch = result(&q, "qa")["ids"].as_array().unwrap();
    assert_eq!(arch.len(), 1);
    assert_eq!(
        subject_of(&h, arch[0].as_str().unwrap()).await,
        "Weekly Report"
    );
}

#[tokio::test]
async fn identities_expose_allowed_froms() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();

    let resp = jmap(
        &h,
        json!([["Identity/query", {}, "iq"], ["Identity/get", {}, "ig"]]),
    )
    .await;
    let ids = result(&resp, "iq")["ids"].as_array().unwrap();
    assert!(
        !ids.is_empty(),
        "a default identity is seeded from the account"
    );
    let list = result(&resp, "ig")["list"].as_array().unwrap();
    assert!(list.iter().any(|i| i["email"] == "me@example.org"));
}

#[tokio::test]
async fn saved_search_folder_runs_its_filter() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();

    // Register a saved search "attachments" as a folder, then query it.
    h.engine
        .store()
        .upsert_saved_search(&mw_store::SavedSearchRow {
            id: "ss-att".into(),
            user: h.account_id.clone(),
            name: "Attachments".into(),
            query_json: json!({ "hasAttachment": true }).to_string(),
            as_folder: true,
        })
        .await
        .unwrap();

    // It shows up as a virtual mailbox.
    let mb = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    assert!(
        result(&mb, "mb")["list"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["id"] == "ss-att" && m["mailwomanSearchQuery"].is_string())
    );

    // Querying the folder id runs the stored filter (only the invoice matches).
    let q = jmap(
        &h,
        json!([["Email/query", { "filter": { "inMailbox": "ss-att" } }, "q"]]),
    )
    .await;
    let ids = result(&q, "q")["ids"].as_array().unwrap();
    assert_eq!(ids.len(), 1);
    assert_eq!(
        subject_of(&h, ids[0].as_str().unwrap()).await,
        "Invoice 2026"
    );
}
