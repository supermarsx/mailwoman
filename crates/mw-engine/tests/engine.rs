//! Engine integration tests driving the JMAP surface over a deterministic,
//! in-process [`FakeBackend`] — no live server (plan §3 e6 acceptance). This
//! mirrors, at the engine layer, the two Playwright flows: list mailboxes/read
//! a MIME-parsed message, thread a reply chain, round-trip a keyword, move a
//! message, and compose → "send" → appears in Sent.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeEvent, ChangeSink, Flag, MailboxDelta, MailboxRole,
    MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor,
    WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

const UIDVALIDITY: u32 = 100;

/// One scripted message: `(uid, raw RFC822, flags, INTERNALDATE)`.
type ScriptMsg = (u32, Vec<u8>, Vec<Flag>, String);

// ---- scripted backend ------------------------------------------------------

struct Scripted {
    mailboxes: Vec<(String, MailboxRole)>,
    messages: HashMap<String, Vec<ScriptMsg>>,
}

struct FakeBackend {
    state: Mutex<Scripted>,
    store_flags_calls: AtomicUsize,
    move_calls: AtomicUsize,
    append_calls: AtomicUsize,
}

impl FakeBackend {
    fn new() -> Self {
        Self::with_inbox(vec![
            (1, multipart_msg(), vec![], "2026-07-01T09:00:00Z".into()),
            (2, thread_root_msg(), vec![], "2026-07-02T09:00:00Z".into()),
            (3, thread_reply_msg(), vec![], "2026-07-03T09:00:00Z".into()),
        ])
    }

    /// Build a backend whose INBOX holds an explicit message set (Sent/Drafts/
    /// Archive stay empty) — used by the operator-search test.
    fn with_inbox(inbox: Vec<ScriptMsg>) -> Self {
        let mailboxes = vec![
            ("INBOX".to_string(), MailboxRole::Inbox),
            ("Sent".to_string(), MailboxRole::Sent),
            ("Drafts".to_string(), MailboxRole::Drafts),
            ("Archive".to_string(), MailboxRole::Archive),
        ];
        let mut messages: HashMap<String, Vec<ScriptMsg>> = HashMap::new();
        messages.insert("INBOX".to_string(), inbox);
        Self {
            state: Mutex::new(Scripted {
                mailboxes,
                messages,
            }),
            store_flags_calls: AtomicUsize::new(0),
            move_calls: AtomicUsize::new(0),
            append_calls: AtomicUsize::new(0),
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

    async fn store_flags(
        &self,
        _refs: &[MessageRef],
        _add: &[Flag],
        _remove: &[Flag],
    ) -> Result<()> {
        self.store_flags_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn move_messages(
        &self,
        _refs: &[MessageRef],
        _to: &RawMailboxRef,
    ) -> Result<MoveOutcome> {
        self.move_calls.fetch_add(1, Ordering::SeqCst);
        Ok(MoveOutcome::Uidplus {
            uidvalidity: UIDVALIDITY,
            uids: vec![999],
        })
    }

    async fn append(&self, mbox: &RawMailboxRef, raw: &[u8], flags: &[Flag]) -> Result<MessageRef> {
        self.append_calls.fetch_add(1, Ordering::SeqCst);
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

// ---- scripted submitter ----------------------------------------------------

struct FakeSubmitter {
    calls: AtomicUsize,
    last_rcpt: Mutex<Vec<String>>,
}

impl FakeSubmitter {
    fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
            last_rcpt: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl MailSubmitter for FakeSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        *self.last_rcpt.lock().unwrap() = msg.rcpt_to.clone();
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

// ---- raw message fixtures --------------------------------------------------

fn multipart_msg() -> Vec<u8> {
    concat!(
        "Message-ID: <multipart@x>\r\n",
        "From: Anna Ng <anna@example.org>\r\n",
        "To: Test User <me@example.org>\r\n",
        "Subject: Multipart hello\r\n",
        "Date: Wed, 01 Jul 2026 09:00:00 +0000\r\n",
        "MIME-Version: 1.0\r\n",
        "Content-Type: multipart/alternative; boundary=\"b0\"\r\n",
        "\r\n",
        "--b0\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "the plain part\r\n",
        "--b0\r\n",
        "Content-Type: text/html; charset=utf-8\r\n",
        "\r\n",
        "<p>the html part</p>\r\n",
        "--b0--\r\n",
    )
    .as_bytes()
    .to_vec()
}

fn thread_root_msg() -> Vec<u8> {
    concat!(
        "Message-ID: <root@x>\r\n",
        "From: Carlos <carlos@example.org>\r\n",
        "To: me@example.org\r\n",
        "Subject: Lunch plans\r\n",
        "Date: Thu, 02 Jul 2026 09:00:00 +0000\r\n",
        "\r\n",
        "Are you free at noon?\r\n",
    )
    .as_bytes()
    .to_vec()
}

fn thread_reply_msg() -> Vec<u8> {
    concat!(
        "Message-ID: <reply@x>\r\n",
        "In-Reply-To: <root@x>\r\n",
        "References: <root@x>\r\n",
        "From: me <me@example.org>\r\n",
        "To: carlos@example.org\r\n",
        "Subject: Re: Lunch plans\r\n",
        "Date: Fri, 03 Jul 2026 09:00:00 +0000\r\n",
        "\r\n",
        "Yes, noon works.\r\n",
    )
    .as_bytes()
    .to_vec()
}

// ---- harness ---------------------------------------------------------------

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
    backend: Arc<FakeBackend>,
    submitter: Arc<FakeSubmitter>,
}

async fn setup() -> Harness {
    setup_with(Arc::new(FakeBackend::new())).await
}

/// Wire an engine + store + registered account over a specific backend.
async fn setup_with(backend: Arc<FakeBackend>) -> Harness {
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
    let submitter = Arc::new(FakeSubmitter::new());
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            backend.clone() as Arc<dyn AccountBackend>,
            submitter.clone() as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    Harness {
        engine,
        account_id,
        backend,
        submitter,
    }
}

/// A minimal RFC822 message with no attachment.
fn plain_msg(message_id: &str, from: &str, subject: &str, body: &str) -> Vec<u8> {
    format!(
        "Message-ID: <{message_id}>\r\n\
         From: {from}\r\n\
         To: me@example.org\r\n\
         Subject: {subject}\r\n\
         Date: Wed, 01 Jul 2026 09:00:00 +0000\r\n\
         \r\n\
         {body}\r\n"
    )
    .into_bytes()
}

/// A multipart/mixed message carrying one real (non-inline) PDF attachment, so
/// `has:attachment` / `filename:` have something to match.
fn attachment_msg(message_id: &str, from: &str, subject: &str) -> Vec<u8> {
    format!(
        "Message-ID: <{message_id}>\r\n\
         From: {from}\r\n\
         To: me@example.org\r\n\
         Subject: {subject}\r\n\
         Date: Wed, 01 Jul 2026 09:00:00 +0000\r\n\
         MIME-Version: 1.0\r\n\
         Content-Type: multipart/mixed; boundary=\"m0\"\r\n\
         \r\n\
         --m0\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         \r\n\
         see attached\r\n\
         --m0\r\n\
         Content-Type: application/pdf; name=\"report.pdf\"\r\n\
         Content-Disposition: attachment; filename=\"report.pdf\"\r\n\
         \r\n\
         %PDF-1.4 pretend\r\n\
         --m0--\r\n"
    )
    .into_bytes()
}

/// Post a JMAP request (a list of `[name, args, callId]` triples).
async fn jmap(h: &Harness, calls: Value) -> Value {
    h.engine
        .handle_jmap(&h.account_id, &json!({ "methodCalls": calls }))
        .await
}

fn method_result<'a>(resp: &'a Value, call_id: &str) -> &'a Value {
    resp["methodResponses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r[2] == call_id)
        .map(|r| &r[1])
        .unwrap_or(&Value::Null)
}

fn mailbox_id(resp: &Value, role: &str) -> String {
    method_result(resp, "mb")["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == role)
        .expect("mailbox with role")["id"]
        .as_str()
        .unwrap()
        .to_string()
}

// ---- tests -----------------------------------------------------------------

#[tokio::test]
async fn mailbox_get_returns_roles() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();

    let resp = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let list = method_result(&resp, "mb")["list"].as_array().unwrap();
    let roles: Vec<&str> = list.iter().filter_map(|m| m["role"].as_str()).collect();
    assert!(roles.contains(&"inbox"), "roles: {roles:?}");
    assert!(roles.contains(&"sent"));
    assert!(roles.contains(&"drafts"));
    // Display names are humanized, not the raw IMAP path.
    let inbox = list.iter().find(|m| m["role"] == "inbox").unwrap();
    assert_eq!(inbox["name"], "Inbox");
    assert_eq!(inbox["totalEmails"], 3);
}

#[tokio::test]
async fn query_then_get_via_result_reference() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let mb = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let inbox = mailbox_id(&mb, "inbox");

    // Email/query -> Email/get chained by a `#ids` result reference, exactly as
    // the web client issues it.
    let resp = jmap(
        &h,
        json!([
            ["Email/query", { "filter": { "inMailbox": inbox } }, "q"],
            ["Email/get", {
                "#ids": { "resultOf": "q", "name": "Email/query", "path": "/ids" }
            }, "g"]
        ]),
    )
    .await;

    let ids = method_result(&resp, "q")["ids"].as_array().unwrap();
    assert_eq!(ids.len(), 3);
    let list = method_result(&resp, "g")["list"].as_array().unwrap();
    assert_eq!(list.len(), 3);

    // The multipart message parsed into both a text and an html body.
    let multipart = list
        .iter()
        .find(|e| e["subject"] == "Multipart hello")
        .expect("multipart message present");
    assert!(!multipart["htmlBody"].as_array().unwrap().is_empty());
    assert!(!multipart["textBody"].as_array().unwrap().is_empty());
    // Its decoded bodyValues carry the actual content.
    let has_html = multipart["bodyValues"]
        .as_object()
        .unwrap()
        .values()
        .any(|v| v["value"].as_str().unwrap_or("").contains("html part"));
    assert!(has_html, "bodyValues: {}", multipart["bodyValues"]);
    // Newest-first ordering (INTERNALDATE desc).
    assert_eq!(list[0]["subject"], "Re: Lunch plans");
}

#[tokio::test]
async fn jwz_groups_the_reply_chain() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let mb = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let inbox = mailbox_id(&mb, "inbox");

    let resp = jmap(
        &h,
        json!([
            ["Email/query", { "filter": { "inMailbox": inbox } }, "q"],
            ["Email/get", {
                "#ids": { "resultOf": "q", "name": "Email/query", "path": "/ids" }
            }, "g"]
        ]),
    )
    .await;
    let list = method_result(&resp, "g")["list"].as_array().unwrap();

    let root = list.iter().find(|e| e["subject"] == "Lunch plans").unwrap();
    let reply = list
        .iter()
        .find(|e| e["subject"] == "Re: Lunch plans")
        .unwrap();
    let multipart = list
        .iter()
        .find(|e| e["subject"] == "Multipart hello")
        .unwrap();

    assert_eq!(root["threadId"], reply["threadId"]);
    assert!(!root["threadId"].is_null());
    assert_ne!(root["threadId"], multipart["threadId"]);
}

#[tokio::test]
async fn keyword_update_round_trips() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let mb = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let inbox = mailbox_id(&mb, "inbox");

    let q = jmap(
        &h,
        json!([["Email/query", { "filter": { "inMailbox": inbox } }, "q"]]),
    )
    .await;
    let first = method_result(&q, "q")["ids"][0]
        .as_str()
        .unwrap()
        .to_string();

    let set = jmap(
        &h,
        json!([["Email/set", { "update": { first.clone(): { "keywords": { "$seen": true } } } }, "s"]]),
    )
    .await;
    assert!(
        method_result(&set, "s")["updated"]
            .as_object()
            .unwrap()
            .contains_key(&first)
    );
    // The backend received the flag change (server-authoritative).
    assert_eq!(h.backend.store_flags_calls.load(Ordering::SeqCst), 1);

    let get = jmap(&h, json!([["Email/get", { "ids": [first] }, "g"]])).await;
    let email = &method_result(&get, "g")["list"][0];
    assert_eq!(email["keywords"]["$seen"], true);
}

#[tokio::test]
async fn mailbox_move_calls_backend_and_relocates() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let mb = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let inbox = mailbox_id(&mb, "inbox");
    let archive = mailbox_id(&mb, "archive");

    let q = jmap(
        &h,
        json!([["Email/query", { "filter": { "inMailbox": inbox } }, "q"]]),
    )
    .await;
    let id = method_result(&q, "q")["ids"][0]
        .as_str()
        .unwrap()
        .to_string();

    jmap(
        &h,
        json!([["Email/set", { "update": { id: { "mailboxIds": { archive.clone(): true } } } }, "s"]]),
    )
    .await;
    assert_eq!(h.backend.move_calls.load(Ordering::SeqCst), 1);

    // Inbox now has one fewer; Archive has it.
    let after = jmap(
        &h,
        json!([
            ["Email/query", { "filter": { "inMailbox": inbox } }, "qi"],
            ["Email/query", { "filter": { "inMailbox": archive } }, "qa"]
        ]),
    )
    .await;
    assert_eq!(
        method_result(&after, "qi")["ids"].as_array().unwrap().len(),
        2
    );
    assert_eq!(
        method_result(&after, "qa")["ids"].as_array().unwrap().len(),
        1
    );
}

#[tokio::test]
async fn compose_send_appears_in_sent() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();

    // Create a draft, then submit it in the same request (referencing #draft).
    let resp = jmap(
        &h,
        json!([
            ["Email/set", { "create": { "draft": {
                "from": [{ "email": "me@example.org" }],
                "to": [{ "email": "friend@example.org" }],
                "subject": "Hello there",
                "bodyValues": { "1": { "value": "Hi!" } },
                "textBody": [{ "partId": "1", "type": "text/plain" }]
            } } }, "c1"],
            ["EmailSubmission/set", { "create": { "sub1": { "emailId": "#draft" } } }, "c2"]
        ]),
    )
    .await;

    assert!(method_result(&resp, "c1")["created"]["draft"]["id"].is_string());
    assert!(method_result(&resp, "c2")["created"]["sub1"]["id"].is_string());
    // The submitter ran with the recipient from the draft.
    assert_eq!(h.submitter.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        *h.submitter.last_rcpt.lock().unwrap(),
        vec!["friend@example.org".to_string()]
    );
    // The sent copy was appended upstream (Sent).
    assert!(h.backend.append_calls.load(Ordering::SeqCst) >= 1);

    // It appears in Sent on re-query, and the draft is gone from Drafts.
    let mb = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let sent = mailbox_id(&mb, "sent");
    let drafts = mailbox_id(&mb, "drafts");
    let after = jmap(
        &h,
        json!([
            ["Email/query", { "filter": { "inMailbox": sent } }, "qs"],
            ["Email/query", { "filter": { "inMailbox": drafts } }, "qd"]
        ]),
    )
    .await;
    let sent_ids = method_result(&after, "qs")["ids"].as_array().unwrap();
    assert_eq!(sent_ids.len(), 1);
    assert_eq!(
        method_result(&after, "qd")["ids"].as_array().unwrap().len(),
        0
    );

    // And the sent message is readable with its subject.
    let sid = sent_ids[0].as_str().unwrap();
    let get = jmap(&h, json!([["Email/get", { "ids": [sid] }, "g"]])).await;
    assert_eq!(
        method_result(&get, "g")["list"][0]["subject"],
        "Hello there"
    );
}

#[tokio::test]
async fn resync_is_idempotent() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    h.engine.resync(&h.account_id).await.unwrap();
    let mb = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let inbox = mailbox_id(&mb, "inbox");
    let q = jmap(
        &h,
        json!([["Email/query", { "filter": { "inMailbox": inbox } }, "q"]]),
    )
    .await;
    // No duplicate ingestion across two syncs.
    assert_eq!(method_result(&q, "q")["ids"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn operator_search_via_filter_text() {
    // The web packs the ENTIRE operator string into `Email/query`'s
    // `filter.text` (e.g. `subject:foo`, `from:anna`, `has:attachment`). These
    // must parse into field/attachment predicates — the previous code wrapped
    // the whole string as one literal all-fields term, so every operator query
    // returned nothing.
    let inbox = vec![
        (
            1,
            plain_msg(
                "foo-anna@x",
                "Anna Ng <anna@example.org>",
                "foo report",
                "quarterly numbers",
            ),
            vec![],
            "2026-07-01T09:00:00Z".to_string(),
        ),
        (
            2,
            plain_msg(
                "bar-bob@x",
                "Bob <bob@example.org>",
                "bar update",
                "status note",
            ),
            vec![],
            "2026-07-02T09:00:00Z".to_string(),
        ),
        (
            3,
            attachment_msg("foo-carol@x", "Carol <carol@example.org>", "foo digest"),
            vec![],
            "2026-07-03T09:00:00Z".to_string(),
        ),
    ];
    let h = setup_with(Arc::new(FakeBackend::with_inbox(inbox))).await;
    h.engine.resync(&h.account_id).await.unwrap();

    let count = |resp: &Value| method_result(resp, "q")["ids"].as_array().unwrap().len();
    let query = |text: &str| json!([["Email/query", { "filter": { "text": text } }, "q"]]);

    // subject:foo -> the two "foo …" subjects (messages 1 and 3), not message 2.
    assert_eq!(count(&jmap(&h, query("subject:foo")).await), 2);
    // subject:bar -> only message 2 (the old bug matched literal "subject"/"bar").
    assert_eq!(count(&jmap(&h, query("subject:bar")).await), 1);

    // from:anna resolves to Anna's message specifically (chained get proves it).
    let r = jmap(
        &h,
        json!([
            ["Email/query", { "filter": { "text": "from:anna" } }, "q"],
            ["Email/get", {
                "#ids": { "resultOf": "q", "name": "Email/query", "path": "/ids" }
            }, "g"]
        ]),
    )
    .await;
    let list = method_result(&r, "g")["list"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["from"][0]["email"], "anna@example.org");

    // has:attachment -> only the message carrying the PDF part (message 3).
    assert_eq!(count(&jmap(&h, query("has:attachment")).await), 1);
    // Operators AND together within one text string.
    assert_eq!(
        count(&jmap(&h, query("subject:foo has:attachment")).await),
        1
    );

    // A bare term still narrows across fields (plain full-text is preserved).
    assert_eq!(count(&jmap(&h, query("digest")).await), 1);
    assert_eq!(count(&jmap(&h, query("nonexistentword")).await), 0);
}

/// A backend whose first `watch` connection drops (emits `Disconnected`) so the
/// engine's watch loop must re-establish it. Counts `watch` calls.
struct ReconnectBackend {
    watch_calls: AtomicUsize,
}

#[async_trait]
impl AccountBackend for ReconnectBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(Vec::new()) // empty ⇒ resync trivially succeeds
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
                uidvalidity: 0,
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
        Ok(MoveOutcome::RederiveByMessageId)
    }
    async fn append(&self, mbox: &RawMailboxRef, _raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
        Ok(MessageRef::Imap {
            mailbox: mbox.clone(),
            uidvalidity: 0,
            uid: 1,
        })
    }
    async fn watch(&self, sink: ChangeSink) -> Result<WatchHandle> {
        let n = self.watch_calls.fetch_add(1, Ordering::SeqCst);
        let mbox = RawMailboxRef {
            name: "INBOX".into(),
            uidvalidity: 0,
        };
        let _ = sink.emit(ChangeEvent::MailboxChanged { mailbox: mbox });
        if n == 0 {
            // First connection drops right after signalling activity.
            let _ = sink.emit(ChangeEvent::Disconnected);
        }
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

#[tokio::test]
async fn watch_reconnects_after_a_drop() {
    // A dropped watch connection must self-heal: the loop re-establishes the
    // watch (a fresh backend login) rather than stalling ingestion.
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let engine = Arc::new(Engine::new(store));
    let backend = Arc::new(ReconnectBackend {
        watch_calls: AtomicUsize::new(0),
    });
    engine.register_backend(
        "recon-acct",
        AccountRuntime::new(
            backend.clone() as Arc<dyn AccountBackend>,
            Arc::new(FakeSubmitter::new()) as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );

    engine.start_watch("recon-acct").await.unwrap();

    // The first watch drops; the loop backs off (~1s) then reconnects.
    let reconnected = tokio::time::timeout(std::time::Duration::from_secs(6), async {
        loop {
            if backend.watch_calls.load(Ordering::SeqCst) >= 2 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
    })
    .await;
    assert!(
        reconnected.is_ok(),
        "watch loop did not reconnect after a drop (watch called {} times)",
        backend.watch_calls.load(Ordering::SeqCst)
    );
}
