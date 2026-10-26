//! Engine integration tests driving the JMAP surface over a deterministic,
//! in-process [`FakeBackend`] — no live server (plan §3 e6 acceptance). This
//! mirrors, at the engine layer, the two Playwright flows: list mailboxes/read
//! a MIME-parsed message, thread a reply chain, round-trip a keyword, move a
//! message, and compose → "send" → appears in Sent.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
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
    snapshot: AtomicBool,
    uidvalidity: AtomicU32,
    state: Mutex<Scripted>,
    store_flags_calls: AtomicUsize,
    move_calls: AtomicUsize,
    append_calls: AtomicUsize,
    import_supported: AtomicBool,
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
            snapshot: AtomicBool::new(false),
            uidvalidity: AtomicU32::new(UIDVALIDITY),
            state: Mutex::new(Scripted {
                mailboxes,
                messages,
            }),
            store_flags_calls: AtomicUsize::new(0),
            move_calls: AtomicUsize::new(0),
            append_calls: AtomicUsize::new(0),
            import_supported: AtomicBool::new(true),
        }
    }
}

#[async_trait]
impl AccountBackend for FakeBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: self.import_supported.load(Ordering::SeqCst),
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
                        uidvalidity: self.uidvalidity.load(Ordering::SeqCst),
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
        if self.snapshot.load(Ordering::SeqCst) {
            let known_uids = msgs.iter().map(|(uid, ..)| *uid).collect();
            return Ok(MailboxDelta {
                added: Vec::new(),
                flag_changes: msgs
                    .iter()
                    .map(|(uid, _, flags, _)| {
                        (
                            MessageRef::Imap {
                                mailbox: mbox.clone(),
                                uidvalidity: self.uidvalidity.load(Ordering::SeqCst),
                                uid: *uid,
                            },
                            flags.clone(),
                        )
                    })
                    .collect(),
                removed: Vec::new(),
                next_cursor: SyncCursor::ImapSnapshot {
                    uidvalidity: self.uidvalidity.load(Ordering::SeqCst),
                    uidnext: 4,
                    known_uids,
                    highestmodseq: None,
                },
            });
        }
        let added: Vec<MessageRef> = msgs
            .iter()
            .filter(|(uid, ..)| *uid >= uidnext_from)
            .map(|(uid, ..)| MessageRef::Imap {
                mailbox: mbox.clone(),
                uidvalidity: self.uidvalidity.load(Ordering::SeqCst),
                uid: *uid,
            })
            .collect();
        let max_uid = msgs.iter().map(|(u, ..)| *u).max().unwrap_or(0);
        Ok(MailboxDelta {
            added,
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: self.uidvalidity.load(Ordering::SeqCst),
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
            uidvalidity: self.uidvalidity.load(Ordering::SeqCst),
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
            uidvalidity: self.uidvalidity.load(Ordering::SeqCst),
            uid,
        })
    }

    fn supports_dated_import(&self) -> bool {
        self.import_supported.load(Ordering::SeqCst)
    }

    async fn append_with_date(
        &self,
        mbox: &RawMailboxRef,
        raw: &[u8],
        flags: &[Flag],
        received_at: &str,
    ) -> Result<MessageRef> {
        let reference = self.append(mbox, raw, flags).await?;
        let MessageRef::Imap { uid, .. } = reference else {
            unreachable!()
        };
        let mut state = self.state.lock().unwrap();
        state
            .messages
            .get_mut(&mbox.name)
            .unwrap()
            .iter_mut()
            .find(|message| message.0 == uid)
            .unwrap()
            .3 = received_at.to_string();
        Ok(reference)
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
    last_raw: Mutex<Vec<u8>>,
    last_sender: Mutex<String>,
}

impl FakeSubmitter {
    fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
            last_rcpt: Mutex::new(Vec::new()),
            last_raw: Mutex::new(Vec::new()),
            last_sender: Mutex::new(String::new()),
        }
    }
}

#[async_trait]
impl MailSubmitter for FakeSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        *self.last_rcpt.lock().unwrap() = msg.rcpt_to.clone();
        *self.last_raw.lock().unwrap() = msg.raw.clone();
        *self.last_sender.lock().unwrap() = msg.mail_from.clone();
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
    reject_reconnect: bool,
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
        if n > 0 && self.reject_reconnect {
            return Err(mw_engine::backend::EngineError::Auth("rejected".into()));
        }
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
        reject_reconnect: false,
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

#[tokio::test]
async fn legacy_cursor_upgrade_removes_expunges_and_publishes_flag_changes() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let mailbox_result = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let inbox = mailbox_id(&mailbox_result, "inbox");
    let initial = jmap(&h, json!([["Email/query", {"filter":{"inMailbox": inbox}}, "q"], ["Email/get", {"#ids": {"resultOf":"q", "name":"Email/query", "path":"/ids"}}, "before"]])).await;
    let initial = method_result(&initial, "before");
    let state = initial["state"].as_str().unwrap().to_string();
    let changed_id = initial["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|email| email["subject"] == "Lunch plans")
        .unwrap_or_else(|| panic!("initial messages: {initial}"))["id"]
        .as_str()
        .unwrap()
        .to_string();
    let removed_id = initial["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|email| email["subject"] == "Multipart hello")
        .map(|email| email["id"].as_str().unwrap().to_string());
    {
        let mut backend = h.backend.state.lock().unwrap();
        let messages = backend.messages.get_mut("INBOX").unwrap();
        messages.retain(|(uid, ..)| *uid != 1);
        messages.iter_mut().find(|(uid, ..)| *uid == 2).unwrap().2 = vec![Flag::Seen];
    }
    h.backend.snapshot.store(true, Ordering::SeqCst);
    h.engine.resync(&h.account_id).await.unwrap();
    let after = jmap(
        &h,
        json!([["Email/query", {"filter":{"inMailbox": inbox}}, "q"], ["Email/get", {"#ids": {"resultOf":"q", "name":"Email/query", "path":"/ids"}}, "get"], ["Email/changes", {"sinceState": state}, "changes"]]),
    )
    .await;
    let emails = method_result(&after, "get")["list"].as_array().unwrap();
    assert_eq!(emails.len(), 2);
    assert_eq!(
        emails
            .iter()
            .find(|email| email["id"] == changed_id)
            .unwrap()["keywords"]["$seen"],
        true
    );
    let changes = method_result(&after, "changes");
    assert!(
        changes["updated"]
            .as_array()
            .unwrap()
            .contains(&json!(changed_id))
    );
    assert_eq!(changes["destroyed"].as_array().unwrap().len(), 1);
    if let Some(id) = removed_id {
        assert!(
            changes["destroyed"]
                .as_array()
                .unwrap()
                .contains(&json!(id))
        );
    }
    let stable_state = method_result(&after, "get")["state"].clone();
    h.engine.resync(&h.account_id).await.unwrap();
    let again = jmap(&h, json!([["Email/get", {}, "get"]])).await;
    assert_eq!(method_result(&again, "get")["state"], stable_state);
}

#[tokio::test]
async fn active_raw_sieve_runs_at_ingest_and_policy_refusal_preserves_it() {
    let h = setup().await;
    let policy = mw_sieve::policy::Policy {
        raw_edit: true,
        own_domains: vec!["example.org".into()],
        max_script_bytes: Some(65536),
        max_scripts: Some(5),
        ..Default::default()
    };
    let source = r#"require ["fileinto", "imap4flags"]; if header :is "Subject" "Lunch plans" { addflag "\\Seen"; fileinto "Archive"; stop; }"#;
    h.engine
        .put_local_sieve_script(
            &h.account_id,
            "active",
            source,
            true,
            &policy,
            mw_sieve::policy::EditKind::Raw,
        )
        .await
        .unwrap();
    let denied = h
        .engine
        .put_local_sieve_script(
            &h.account_id,
            "active",
            "redirect \"outside@elsewhere.test\";",
            true,
            &policy,
            mw_sieve::policy::EditKind::Raw,
        )
        .await;
    assert!(denied.is_err());
    h.engine.resync(&h.account_id).await.unwrap();
    assert_eq!(h.backend.store_flags_calls.load(Ordering::SeqCst), 1);
    assert_eq!(h.backend.move_calls.load(Ordering::SeqCst), 1);
    let boxes = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let archive = mailbox_id(&boxes, "archive");
    let result = jmap(&h, json!([["Email/query", {"filter":{"inMailbox":archive}}, "q"], ["Email/get", {"#ids":{"resultOf":"q","name":"Email/query","path":"/ids"}}, "get"]])).await;
    let emails = method_result(&result, "get")["list"].as_array().unwrap();
    assert_eq!(emails.len(), 1);
    assert_eq!(emails[0]["keywords"]["$seen"], true);
    assert_eq!(emails[0]["subject"], "Lunch plans");
}

#[tokio::test]
async fn uidvalidity_reset_preserves_mailbox_and_surviving_ids_and_removes_stale_mail() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let boxes = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let inbox = mailbox_id(&boxes, "inbox");
    let initial = jmap(&h, json!([["Email/query", {"filter":{"inMailbox":inbox}}, "q"], ["Email/get", {"#ids":{"resultOf":"q","name":"Email/query","path":"/ids"}}, "get"]])).await;
    let before = method_result(&initial, "get");
    let survivor = before["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|email| email["subject"] == "Lunch plans")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let old_state = before["state"].as_str().unwrap().to_string();
    {
        let mut state = h.backend.state.lock().unwrap();
        let messages = state.messages.get_mut("INBOX").unwrap();
        messages.retain(|(uid, ..)| *uid == 2);
        messages[0].0 = 7;
        messages[0].2 = vec![Flag::Seen];
    }
    // A newly activated rule must not replay on historical survivors of reset.
    h.engine
        .put_local_sieve_script(
            &h.account_id,
            "after-delivery",
            r#"require "imap4flags"; addflag "\\Flagged";"#,
            true,
            &mw_sieve::policy::Policy {
                raw_edit: true,
                ..Default::default()
            },
            mw_sieve::policy::EditKind::Raw,
        )
        .await
        .unwrap();
    h.backend.uidvalidity.store(200, Ordering::SeqCst);
    h.engine.resync(&h.account_id).await.unwrap();
    let boxes_after = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    assert_eq!(mailbox_id(&boxes_after, "inbox"), inbox);
    assert_eq!(
        method_result(&boxes_after, "mb")["list"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    let result = jmap(&h, json!([["Email/query", {"filter":{"inMailbox":inbox}}, "q"], ["Email/get", {"#ids":{"resultOf":"q","name":"Email/query","path":"/ids"}}, "get"], ["Email/changes", {"sinceState":old_state}, "changes"]])).await;
    let emails = method_result(&result, "get")["list"].as_array().unwrap();
    assert_eq!(emails.len(), 1);
    assert_eq!(emails[0]["id"], survivor);
    assert_eq!(emails[0]["keywords"]["$seen"], true);
    assert_eq!(
        method_result(&result, "changes")["destroyed"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let row = h.engine.store().get_message(&survivor).await.unwrap();
    assert_eq!((row.uidvalidity, row.uid), (200, 7));
    assert_eq!(
        h.backend.store_flags_calls.load(Ordering::SeqCst),
        0,
        "historical reset must not replay Sieve"
    );
    h.engine.resync(&h.account_id).await.unwrap();
    assert_eq!(
        h.engine
            .store()
            .list_message_ids(&inbox, 100, 0)
            .await
            .unwrap(),
        vec![survivor]
    );
}

#[tokio::test]
async fn local_vacation_submits_once_to_envelope_sender_and_suppresses_automatic_mail() {
    let cases = [
        ("sender@example.net", "me@example.org", ""),
        ("sender@example.net", "me@example.org", ""),
        ("list@example.net", "me@example.org", "List-Id: <list.example.net>\r\n"),
        ("auto@example.net", "me@example.org", "Auto-Submitted: auto-generated\r\n"),
        ("bulk@example.net", "me@example.org", "Precedence: bulk\r\n"),
        ("", "me@example.org", ""),
        ("me@example.org", "me@example.org", ""),
        ("hidden@example.net", "somebody@example.org", ""),
        ("loop@example.net", "me@example.org", "X-Mailwoman-Loop: vacation\r\n"),
    ];
    let messages = cases.iter().enumerate().map(|(i,(sender,to,extra))| {
        ((i+1) as u32,format!("Message-ID: <vacation-{i}@example.net>\r\nReturn-Path: <{sender}>\r\nFrom: forged@example.net\r\nTo: {to}\r\nSubject: Hello\r\n{extra}\r\nHello").into_bytes(),vec![],"2026-07-01T09:00:00Z".into())
    }).collect();
    let h = setup_with(Arc::new(FakeBackend::with_inbox(messages))).await;
    h.engine.store().put_setting_value(&mw_store::settings_policy::SettingValueRow {
        scope:mw_store::settings_policy::SettingScope::AdminUser, scope_id:h.account_id.clone(),key:"sieve.vacationAllowed".into(),value:"true".into(),locked:true,updated_by:"test".into(),updated_at:0,
    }).await.unwrap();
    h.engine.put_local_sieve_script(&h.account_id,"vacation",r#"require "vacation"; vacation :days 7 :subject "Away" "Back soon";"#,true,
        &mw_sieve::policy::Policy {raw_edit:true,vacation_allowed:true,..Default::default()},mw_sieve::policy::EditKind::Raw).await.unwrap();
    h.engine.resync(&h.account_id).await.unwrap();
    assert_eq!(h.submitter.calls.load(Ordering::SeqCst),1);
    assert_eq!(*h.submitter.last_rcpt.lock().unwrap(),vec!["sender@example.net"]);
    assert_eq!(*h.submitter.last_sender.lock().unwrap(),"");
    let raw = String::from_utf8(h.submitter.last_raw.lock().unwrap().clone()).unwrap();
    assert!(raw.contains("Auto-Submitted: auto-replied"),"{raw}");
    assert!(raw.contains("Back soon"),"{raw}");
    assert!(raw.contains("In-Reply-To:"),"{raw}");
    // A second sync and a UID epoch reset do not replay historical mail.
    h.engine.resync(&h.account_id).await.unwrap();
    h.backend.uidvalidity.store(200,Ordering::SeqCst);
    h.engine.resync(&h.account_id).await.unwrap();
    assert_eq!(h.submitter.calls.load(Ordering::SeqCst),1);
    h.engine.store().put_setting_value(&mw_store::settings_policy::SettingValueRow {
        scope:mw_store::settings_policy::SettingScope::AdminUser, scope_id:h.account_id.clone(),key:"sieve.vacationAllowed".into(),value:"false".into(),locked:true,updated_by:"test".into(),updated_at:1,
    }).await.unwrap();
    h.backend.state.lock().unwrap().messages.get_mut("INBOX").unwrap().push((20,b"Message-ID: <after-revocation@example.net>\r\nReturn-Path: <new@example.net>\r\nFrom: new@example.net\r\nTo: me@example.org\r\n\r\nNew mail".to_vec(),vec![],"2026-07-01T09:00:00Z".into()));
    let _ = h.engine.resync(&h.account_id).await;
    assert_eq!(h.submitter.calls.load(Ordering::SeqCst),1,"revoked vacation authority must apply at delivery");
}

#[tokio::test]
async fn local_redirect_copy_uses_smtp_and_hot_revocation_prevents_later_forwarding() {
    let message = b"Message-ID: <forward-1@example.net>\r\nReturn-Path: <sender@example.net>\r\nFrom: sender@example.net\r\nTo: me@example.org\r\nSubject: Forward\r\n\r\nHello".to_vec();
    let h=setup_with(Arc::new(FakeBackend::with_inbox(vec![(1,message.clone(),vec![],"2026-07-01T09:00:00Z".into())]))).await;
    let setting = |value: &str| mw_store::settings_policy::SettingValueRow {
        scope:mw_store::settings_policy::SettingScope::AdminUser,scope_id:h.account_id.clone(),key:"cap.sieve.forward_external".into(),value:value.into(),locked:true,updated_by:"test".into(),updated_at:0,
    };
    h.engine.store().put_setting_value(&setting("true")).await.unwrap();
    h.engine.put_local_sieve_script(&h.account_id,"forward",r#"require "copy"; redirect :copy "target@external.test";"#,true,
        &mw_sieve::policy::Policy {raw_edit:true,forward_external:true,..Default::default()},mw_sieve::policy::EditKind::Raw).await.unwrap();
    h.engine.resync(&h.account_id).await.unwrap();
    assert_eq!(h.submitter.calls.load(Ordering::SeqCst),1);
    assert_eq!(*h.submitter.last_rcpt.lock().unwrap(),vec!["target@external.test"]);
    assert!(h.submitter.last_raw.lock().unwrap().starts_with(b"X-Mailwoman-Loop: local-sieve\r\n"));
    assert_eq!(h.backend.move_calls.load(Ordering::SeqCst),0);
    h.engine.store().put_setting_value(&setting("false")).await.unwrap();
    h.backend.state.lock().unwrap().messages.get_mut("INBOX").unwrap().push((2,String::from_utf8(message).unwrap().replace("forward-1","forward-2").into_bytes(),vec![],"2026-07-01T09:00:00Z".into()));
    let _=h.engine.resync(&h.account_id).await;
    assert_eq!(h.submitter.calls.load(Ordering::SeqCst),1);
}

#[tokio::test]
async fn watch_authentication_rejection_stops_without_retry() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let engine = Arc::new(Engine::new(store));
    let backend = Arc::new(ReconnectBackend {
        watch_calls: AtomicUsize::new(0),
        reject_reconnect: true,
    });
    engine.register_backend(
        "rejected",
        AccountRuntime::new(
            backend.clone(),
            Arc::new(FakeSubmitter::new()),
            "me@example.org",
        ),
    );
    engine.start_watch("rejected").await.unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while engine.is_registered("rejected") {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    assert_eq!(
        backend.watch_calls.load(Ordering::SeqCst),
        2,
        "initial watch plus exactly one rejected reconnect"
    );
}
#[tokio::test]
async fn obsolete_worker_cannot_reject_a_replacement_runtime() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let engine = Arc::new(Engine::new(store));
    let old: Arc<dyn AccountBackend> = Arc::new(ReconnectBackend {
        watch_calls: AtomicUsize::new(0),
        reject_reconnect: true,
    });
    let replacement: Arc<dyn AccountBackend> = Arc::new(ReconnectBackend {
        watch_calls: AtomicUsize::new(0),
        reject_reconnect: false,
    });
    engine.register_backend(
        "replaced",
        AccountRuntime::new(
            old.clone(),
            Arc::new(FakeSubmitter::new()),
            "me@example.org",
        ),
    );
    engine.register_backend(
        "replaced",
        AccountRuntime::new(
            replacement,
            Arc::new(FakeSubmitter::new()),
            "me@example.org",
        ),
    );
    engine.reject_runtime_credentials("replaced", &old).await;
    assert!(engine.is_registered("replaced"));
}
#[tokio::test]
async fn rejected_revision_stays_blocked_when_durable_marker_cannot_be_written() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let engine = Arc::new(Engine::new(store));
    let backend: Arc<dyn AccountBackend> = Arc::new(ReconnectBackend {
        watch_calls: AtomicUsize::new(0), reject_reconnect: true,
    });
    let mut runtime = AccountRuntime::new(backend.clone(), Arc::new(FakeSubmitter::new()), "me@example.org");
    runtime.credential_revision = Some(vec![1, 2, 3]);
    engine.register_backend("missing-store-row", runtime);
    engine.reject_runtime_credentials("missing-store-row", &backend).await;
    assert!(engine.credential_revision_rejected("missing-store-row", &[1, 2, 3]));
    assert!(!engine.credential_revision_rejected("missing-store-row", &[4, 5, 6]));
    assert!(!engine.is_registered("missing-store-row"));
}

#[tokio::test]
async fn import_persists_upstream_date_flags_without_replaying_arrival_rules() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let boxes = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let inbox = mailbox_id(&boxes, "inbox");
    let query = jmap(
        &h,
        json!([["Email/query", {"filter":{"inMailbox":inbox}}, "q"]]),
    )
    .await;
    let source = method_result(&query, "q")["ids"][0].as_str().unwrap();
    h.engine
        .put_local_sieve_script(
            &h.account_id,
            "discard-import",
            "discard;",
            true,
            &mw_sieve::policy::Policy {
                raw_edit: true,
                ..Default::default()
            },
            mw_sieve::policy::EditKind::Raw,
        )
        .await
        .unwrap();
    let response=jmap(&h,json!([["Email/import",{"emails":{"new":{"blobId":source,"mailboxIds":{inbox.clone():true},"keywords":{"$seen":true},"receivedAt":"2020-02-03T04:05:06Z"}}},"i"]])).await;
    let imported = method_result(&response, "i")["created"]["new"]["id"]
        .as_str()
        .expect("durable import")
        .to_string();
    assert_ne!(imported, source);
    let saved = h.engine.store().get_message(&imported).await.unwrap();
    assert_eq!(saved.uidvalidity, UIDVALIDITY);
    assert!(saved.uid > 0);
    assert_eq!(saved.internaldate.as_deref(), Some("2020-02-03T04:05:06Z"));
    assert_eq!(saved.mailbox_id, inbox);
    let server = h.backend.state.lock().unwrap().messages["INBOX"].clone();
    let appended = server.last().unwrap();
    assert_eq!(appended.3, "2020-02-03T04:05:06Z");
    assert_eq!(appended.2, vec![Flag::Seen]);
    assert_eq!(h.backend.append_calls.load(Ordering::SeqCst), 1);
    assert_eq!(h.submitter.calls.load(Ordering::SeqCst), 0);
    h.backend
        .state
        .lock()
        .unwrap()
        .messages
        .get_mut("INBOX")
        .unwrap()
        .last_mut()
        .unwrap()
        .3 = "03-Feb-2020 04:05:06 +0000".into();
    h.engine.resync(&h.account_id).await.unwrap();
    assert!(
        h.engine.store().get_message(&imported).await.is_ok(),
        "later sync must not run discard for an already imported message"
    );
    assert_eq!(
        h.engine
            .store()
            .get_message(&imported)
            .await
            .unwrap()
            .internaldate
            .as_deref(),
        Some("2020-02-03T04:05:06Z")
    );
}

#[tokio::test]
async fn import_refuses_unsupported_or_invalid_specs_before_upstream_mutation() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let boxes = jmap(&h, json!([["Mailbox/get", {}, "mb"]])).await;
    let inbox = mailbox_id(&boxes, "inbox");
    let archive = mailbox_id(&boxes, "archive");
    let query = jmap(
        &h,
        json!([["Email/query", {"filter":{"inMailbox":inbox}}, "q"]]),
    )
    .await;
    let source = method_result(&query, "q")["ids"][0].as_str().unwrap();
    for spec in [
        json!({"blobId":source,"mailboxIds":{inbox.clone():true,archive:true}}),
        json!({"blobId":source,"mailboxIds":{inbox.clone():false}}),
        json!({"blobId":source,"mailboxIds":{inbox.clone():true},"keywords":{"evil\r\nLOGOUT":true}}),
        json!({"blobId":source,"mailboxIds":{inbox.clone():true},"keywords":{"$seen":false}}),
        json!({"blobId":source,"mailboxIds":{inbox.clone():true},"receivedAt":"not-a-date"}),
    ] {
        let response = jmap(&h, json!([["Email/import",{"emails":{"new":spec}},"i"]])).await;
        assert!(
            method_result(&response, "i")["created"]
                .as_object()
                .unwrap()
                .is_empty(),
            "{response}"
        );
    }
    h.backend.import_supported.store(false, Ordering::SeqCst);
    let response=jmap(&h,json!([["Email/import",{"emails":{"new":{"blobId":source,"mailboxIds":{inbox:true}}}},"i"]])).await;
    assert_eq!(
        method_result(&response, "i")["notCreated"]["new"]["type"],
        "forbidden"
    );
    assert_eq!(h.backend.append_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn dlp_unreadable_policy_never_dispatches_submission() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    h.engine
        .store()
        .set_setting(
            "v6:admin:security_policy",
            r#"{"dlp_rules_json":"broken policy"}"#,
        )
        .await
        .unwrap();
    let response=jmap(&h,json!([
        ["Email/set",{"create":{"draft":{"from":[{"email":"me@example.org"}],"to":[{"email":"friend@example.org"}],"subject":"Policy unavailable","bodyValues":{"1":{"value":"Synthetic message"}},"textBody":[{"partId":"1","type":"text/plain"}]}}},"draft"],
        ["EmailSubmission/set",{"create":{"send":{"emailId":"#draft"}}},"send"]
    ])).await;
    assert_eq!(
        method_result(&response, "send")["notCreated"]["send"]["code"],
        "dlpPolicyUnavailable"
    );
    assert_eq!(h.submitter.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn dlp_scan_cannot_inspect_another_accounts_draft() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let draft=jmap(&h,json!([["Email/set",{"create":{"draft":{"from":[{"email":"me@example.org"}],"to":[{"email":"friend@example.org"}],"subject":"Private fixture","bodyValues":{"1":{"value":"password synthetic-secret"}},"textBody":[{"partId":"1","type":"text/plain"}]}}},"draft"]])).await;
    let id = method_result(&draft, "draft")["created"]["draft"]["id"]
        .as_str()
        .unwrap();
    let other = h
        .engine
        .store()
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: "other@example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "other@example.org".into(),
                password: "fixture".into(),
            },
        )
        .await
        .unwrap();
    h.engine.register_backend(
        other.clone(),
        AccountRuntime::new(
            Arc::new(FakeBackend::new()),
            Arc::new(FakeSubmitter::new()),
            "other@example.org",
        ),
    );
    let response = h
        .engine
        .handle_jmap(
            &other,
            &json!({"methodCalls":[["Dlp/scan",{"draftId":id},"scan"]]}),
        )
        .await;
    assert_eq!(method_result(&response, "scan")["type"], "serverFail");
    assert_eq!(
        method_result(&response, "scan")["code"],
        "dlpInputUnavailable"
    );
    assert!(method_result(&response, "scan").get("list").is_none());
}

fn dlp_test_policy(action: &str, domains: Vec<&str>) -> String {
    json!({"dlp_rules_json":json!([{"id":"fixture","name":"Fixture","enabled":true,"priority":1,"conditions":{"detectors":["pan"],"customRegex":null,"dictionaries":[],"attachmentTypes":[],"maxAttachmentSize":null,"recipientDomains":domains,"recipientDomainMode":"in","classification":null},"action":action,"message":"Restricted"}]).to_string()}).to_string()
}

#[tokio::test]
async fn dlp_policy_changed_during_hold_is_rechecked_before_dispatch() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let queued=jmap(&h,json!([
        ["Email/set",{"create":{"draft":{"from":[{"email":"me@example.org"}],"to":[{"email":"friend@example.org"}],"subject":"Queued","bodyValues":{"1":{"value":"4111 1111 1111 1111"}},"textBody":[{"partId":"1","type":"text/plain"}]}}},"draft"],
        ["EmailSubmission/set",{"create":{"send":{"emailId":"#draft","mailwomanHold":"manual"}}},"send"]
    ])).await;
    let id = method_result(&queued, "send")["created"]["send"]["id"]
        .as_str()
        .unwrap();
    assert_eq!(h.submitter.calls.load(Ordering::SeqCst), 0);
    h.engine
        .store()
        .set_setting(
            "v6:admin:security_policy",
            &dlp_test_policy("block", vec![]),
        )
        .await
        .unwrap();
    let released = jmap(
        &h,
        json!([["EmailSubmission/set",{"update":{id:{"mailwomanHold":null}}},"release"]]),
    )
    .await;
    assert_eq!(h.submitter.calls.load(Ordering::SeqCst), 0, "{released}");
    let query = jmap(&h, json!([["EmailSubmission/get",{"ids":[id]},"get"]])).await;
    assert_eq!(
        method_result(&query, "get")["list"][0]["undoStatus"],
        "canceled",
        "{query}"
    );
}

#[tokio::test]
async fn dlp_checks_actual_smtp_recipients_not_only_mime_to() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    h.engine
        .store()
        .set_setting(
            "v6:admin:security_policy",
            &dlp_test_policy("block", vec!["outside.example"]),
        )
        .await
        .unwrap();
    let response=jmap(&h,json!([
        ["Email/set",{"create":{"draft":{"from":[{"email":"me@example.org"}],"to":[{"email":"friend@example.org"}],"subject":"Envelope override","bodyValues":{"1":{"value":"4111 1111 1111 1111"}},"textBody":[{"partId":"1","type":"text/plain"}]}}},"draft"],
        ["EmailSubmission/set",{"create":{"send":{"emailId":"#draft","envelope":{"mailFrom":{"email":"me@example.org"},"rcptTo":[{"email":"recipient@outside.example"}]}}}},"send"]
    ])).await;
    assert_eq!(h.submitter.calls.load(Ordering::SeqCst), 0, "{response}");
}

#[tokio::test]
async fn dlp_encryption_requirement_refuses_matching_plaintext_submission() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    h.engine
        .store()
        .set_setting(
            "v6:admin:security_policy",
            &dlp_test_policy("require-encryption", vec![]),
        )
        .await
        .unwrap();
    let response=jmap(&h,json!([
        ["Email/set",{"create":{"draft":{"from":[{"email":"me@example.org"}],"to":[{"email":"friend@example.org"}],"subject":"Unencrypted","bodyValues":{"1":{"value":"4111 1111 1111 1111"}},"textBody":[{"partId":"1","type":"text/plain"}]}}},"draft"],
        ["EmailSubmission/set",{"create":{"send":{"emailId":"#draft"}}},"send"]
    ])).await;
    assert_eq!(
        method_result(&response, "send")["notCreated"]["send"]["type"],
        "dlpBlocked",
        "{response}"
    );
    assert_eq!(h.submitter.calls.load(Ordering::SeqCst), 0);
}
