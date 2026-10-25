//! 26.20 f24 (t28-e17 open item 4): a sent message leaves no copy in Drafts.
//!
//! The engine APPENDs every composed draft to the mail server's Drafts folder
//! (so another mail program sees it) and keeps its own engine-local copy. After
//! the send it filed the local copy into Sent and APPENDed the sent bytes there,
//! but the copy in the server's Drafts folder stayed: the next sync brought it
//! back, and the Drafts folder showed every message the user had sent.
//!
//! The same sync also listed the message twice in Sent: the engine's own copy
//! had no server coordinates, so the APPENDed copy looked like another message.
//!
//! These tests drive the real engine over an in-process IMAP stand-in that keeps
//! what is APPENDed per folder, serves it back to the sync, and records what is
//! expunged. The property is what Drafts and Sent hold after the send and a
//! resync: nothing in Drafts, one message in Sent.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MailboxRole,
    MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor,
    WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store, SubmissionRow};

const UIDVALIDITY: u32 = 100;

/// One message the stand-in server holds.
#[derive(Clone)]
struct Held {
    uid: u32,
    raw: Vec<u8>,
    flags: Vec<Flag>,
}

/// An IMAP server with UIDPLUS in miniature: APPEND answers with the new UID,
/// the sync serves every message at or after the cursor's `uidnext`, and
/// `expunge_messages` removes exactly the named messages.
struct MiniImap {
    folders: Mutex<BTreeMap<String, Vec<Held>>>,
    next_uid: Mutex<u32>,
    /// Every expunge request, as (folder, uid).
    expunged: Mutex<Vec<(String, u32)>>,
    /// Answer `expunge_messages` with `Unsupported` (no UIDPLUS).
    refuse_expunge: bool,
}

impl MiniImap {
    fn new(refuse_expunge: bool) -> Arc<Self> {
        let folders = ["INBOX", "Drafts", "Sent"]
            .into_iter()
            .map(|f| (f.to_string(), Vec::new()))
            .collect();
        Arc::new(Self {
            folders: Mutex::new(folders),
            next_uid: Mutex::new(1),
            expunged: Mutex::new(Vec::new()),
            refuse_expunge,
        })
    }

    fn count(&self, folder: &str) -> usize {
        self.folders.lock().unwrap()[folder].len()
    }

    fn expunged(&self) -> Vec<(String, u32)> {
        self.expunged.lock().unwrap().clone()
    }

    fn uids(&self, folder: &str) -> Vec<u32> {
        self.folders.lock().unwrap()[folder]
            .iter()
            .map(|m| m.uid)
            .collect()
    }
}

fn imap_ref(folder: &str, uid: u32) -> MessageRef {
    MessageRef::Imap {
        mailbox: RawMailboxRef {
            name: folder.to_string(),
            uidvalidity: UIDVALIDITY,
        },
        uidvalidity: UIDVALIDITY,
        uid,
    }
}

#[async_trait]
impl AccountBackend for MiniImap {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: true,
            special_use: true,
            ..BackendCaps::default()
        })
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        let next = *self.next_uid.lock().unwrap();
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
            uidnext: next,
            highestmodseq: 0,
            total: self.count(name) as u32,
            unread: 0,
        })
        .collect())
    }

    async fn sync_mailbox(
        &self,
        mbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        let from = match cursor {
            SyncCursor::UidWindow { uidnext, .. } => *uidnext,
            _ => 1,
        };
        let held = self.folders.lock().unwrap()[&mbox.name].clone();
        let next = *self.next_uid.lock().unwrap();
        Ok(MailboxDelta {
            added: held
                .iter()
                .filter(|m| m.uid >= from)
                .map(|m| imap_ref(&mbox.name, m.uid))
                .collect(),
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: UIDVALIDITY,
                uidnext: next,
            },
        })
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        let folders = self.folders.lock().unwrap();
        Ok(refs
            .iter()
            .filter_map(|r| {
                let MessageRef::Imap { mailbox, uid, .. } = r else {
                    return None;
                };
                folders[&mailbox.name]
                    .iter()
                    .find(|m| m.uid == *uid)
                    .map(|m| RawMessage {
                        message_ref: r.clone(),
                        raw: m.raw.clone(),
                        flags: m.flags.clone(),
                        internaldate: None,
                    })
            })
            .collect())
    }

    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _rm: &[Flag]) -> Result<()> {
        Ok(())
    }

    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("move".into()))
    }

    async fn append(&self, mbox: &RawMailboxRef, raw: &[u8], flags: &[Flag]) -> Result<MessageRef> {
        let uid = {
            let mut next = self.next_uid.lock().unwrap();
            let uid = *next;
            *next += 1;
            uid
        };
        self.folders
            .lock()
            .unwrap()
            .get_mut(&mbox.name)
            .ok_or_else(|| EngineError::Protocol(format!("NO no such mailbox {}", mbox.name)))?
            .push(Held {
                uid,
                raw: raw.to_vec(),
                flags: flags.to_vec(),
            });
        Ok(imap_ref(&mbox.name, uid))
    }

    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }

    async fn expunge_messages(&self, refs: &[MessageRef]) -> Result<()> {
        if self.refuse_expunge {
            return Err(EngineError::Unsupported("expunge_messages".into()));
        }
        let mut folders = self.folders.lock().unwrap();
        for r in refs {
            let MessageRef::Imap { mailbox, uid, .. } = r else {
                continue;
            };
            self.expunged
                .lock()
                .unwrap()
                .push((mailbox.name.clone(), *uid));
            if let Some(held) = folders.get_mut(&mailbox.name) {
                held.retain(|m| m.uid != *uid);
            }
        }
        Ok(())
    }
}

/// SMTP that accepts every recipient.
struct Accepting;

#[async_trait]
impl MailSubmitter for Accepting {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
    imap: Arc<MiniImap>,
}

async fn setup(refuse_expunge: bool) -> Harness {
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
    let imap = MiniImap::new(refuse_expunge);
    let engine = Arc::new(Engine::new(store));
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            imap.clone() as Arc<dyn AccountBackend>,
            Arc::new(Accepting) as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    engine.resync(&account_id).await.unwrap();
    Harness {
        engine,
        account_id,
        imap,
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

async fn mailbox_id(h: &Harness, role: &str) -> String {
    let resp = jmap(h, json!([["Mailbox/get", {}, "m"]])).await;
    result(&resp, "m")["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == role)
        .and_then(|m| m["id"].as_str())
        .unwrap_or_else(|| panic!("a {role} mailbox: {resp}"))
        .to_string()
}

/// The ids the engine lists in the mailbox with `role`.
async fn listed(h: &Harness, role: &str) -> Vec<String> {
    let mb = mailbox_id(h, role).await;
    let resp = jmap(
        h,
        json!([["Email/query", { "filter": { "inMailbox": mb } }, "q"]]),
    )
    .await;
    result(&resp, "q")["ids"]
        .as_array()
        .unwrap_or_else(|| panic!("Email/query: {resp}"))
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

fn draft_create(drafts: &str, subject: &str) -> Value {
    json!(["Email/set", { "create": { "draft": {
        "mailboxIds": { drafts: true },
        "keywords": { "$draft": true, "$seen": true },
        "from": [{ "email": "me@example.org" }],
        "to": [{ "email": "friend@example.org" }],
        "subject": subject,
        "bodyValues": { "1": { "value": "hello" } },
        "textBody": [{ "partId": "1", "type": "text/plain" }]
    } } }, "c1"])
}

/// How the send files the message afterwards.
enum Filing {
    /// The web client's request: `onSuccessUpdateEmail` moves it into Sent.
    UpdateIntoSent,
    /// No `onSuccess*`: the engine files a copy into Sent.
    Default,
    /// `onSuccessDestroyEmail`: no copy is kept.
    Destroy,
}

/// Compose and send in one request, as the web client does (hold 0).
async fn send(h: &Harness, subject: &str, filing: Filing) -> Value {
    let drafts = mailbox_id(h, "drafts").await;
    let sent = mailbox_id(h, "sent").await;
    let mut set = json!({ "create": { "s1": {
        "emailId": "#draft", "mailwomanHoldSeconds": 0,
        "envelope": { "mailFrom": { "email": "me@example.org" },
                      "rcptTo": [{ "email": "friend@example.org" }] }
    } } });
    match filing {
        Filing::UpdateIntoSent => {
            set["onSuccessUpdateEmail"] =
                json!({ "#s1": { "mailboxIds": { sent: true }, "keywords/$draft": null } });
        }
        Filing::Default => {}
        Filing::Destroy => set["onSuccessDestroyEmail"] = json!(["#s1"]),
    }
    let resp = jmap(
        h,
        json!([
            draft_create(&drafts, subject),
            ["EmailSubmission/set", set, "c2"]
        ]),
    )
    .await;
    assert_eq!(
        result(&resp, "c2")["created"]["s1"]["undoStatus"],
        "final",
        "the send went out: {resp}"
    );
    resp
}

/// Precondition: a composed draft is on the server's Drafts folder, and the
/// engine lists one message there — the stand-in is a faithful enough server
/// for the bug to show.
#[tokio::test]
async fn a_composed_draft_is_appended_to_the_server_drafts_folder() {
    let h = setup(false).await;
    let drafts = mailbox_id(&h, "drafts").await;
    let resp = jmap(&h, json!([draft_create(&drafts, "Kept")])).await;
    assert!(
        result(&resp, "c1")["created"]["draft"]["id"].is_string(),
        "{resp}"
    );
    assert_eq!(h.imap.count("Drafts"), 1);
    assert!(
        h.imap.expunged().is_empty(),
        "nothing is removed before a send"
    );
}

/// THE regression: the web client's send (filed into Sent by
/// `onSuccessUpdateEmail`). After the send and a resync the server's Drafts
/// folder is empty, the engine lists nothing in Drafts, and Sent has the message.
#[tokio::test]
async fn a_send_filed_into_sent_leaves_no_copy_in_drafts() {
    let h = setup(false).await;
    send(&h, "Filed", Filing::UpdateIntoSent).await;
    h.engine.resync(&h.account_id).await.unwrap();
    assert_eq!(
        listed(&h, "drafts").await,
        Vec::<String>::new(),
        "Drafts after resync"
    );
    assert_eq!(listed(&h, "sent").await.len(), 1, "Sent after resync");

    let draft_uid = 1; // the first APPEND of this server
    assert_eq!(h.imap.expunged(), vec![("Drafts".to_string(), draft_uid)]);
    assert_eq!(h.imap.count("Drafts"), 0);
    assert_eq!(h.imap.count("Sent"), 1, "the sent copy is on the server");
}

#[tokio::test]
async fn a_send_with_the_default_filing_leaves_no_copy_in_drafts() {
    let h = setup(false).await;
    send(&h, "Default", Filing::Default).await;
    assert_eq!(h.imap.expunged(), vec![("Drafts".to_string(), 1)]);
    h.engine.resync(&h.account_id).await.unwrap();
    assert_eq!(listed(&h, "drafts").await, Vec::<String>::new());
    assert_eq!(
        listed(&h, "sent").await.len(),
        1,
        "one copy in Sent after resync"
    );
}

#[tokio::test]
async fn a_send_that_keeps_no_copy_leaves_none_in_drafts_either() {
    let h = setup(false).await;
    send(&h, "Destroyed", Filing::Destroy).await;
    assert_eq!(h.imap.expunged(), vec![("Drafts".to_string(), 1)]);
    h.engine.resync(&h.account_id).await.unwrap();
    assert_eq!(listed(&h, "drafts").await, Vec::<String>::new());
}

/// A held send (the web client's 10 s undo window) is sent by the dispatcher;
/// the same cleanup runs there.
#[tokio::test]
async fn a_held_send_sent_by_the_dispatcher_leaves_no_copy_in_drafts() {
    let h = setup(false).await;
    let drafts = mailbox_id(&h, "drafts").await;
    let resp = jmap(&h, json!([draft_create(&drafts, "Held")])).await;
    let draft = result(&resp, "c1")["created"]["draft"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    h.engine
        .store()
        .insert_submission(&SubmissionRow {
            id: "sub-held".into(),
            account_id: h.account_id.clone(),
            email_id: draft,
            identity_id: None,
            send_at: None,
            undo_status: "pending".into(),
            hold_seconds: 10,
            created_at: "2000-01-01T00:00:00Z".into(),
        })
        .await
        .unwrap();
    h.engine.dispatch_tick().await.unwrap();
    assert_eq!(h.imap.expunged(), vec![("Drafts".to_string(), 1)]);
    h.engine.resync(&h.account_id).await.unwrap();
    assert_eq!(listed(&h, "drafts").await, Vec::<String>::new());
}

/// Only the draft of the send goes: another draft in the same folder stays.
#[tokio::test]
async fn another_draft_in_the_folder_is_not_touched() {
    let h = setup(false).await;
    let drafts = mailbox_id(&h, "drafts").await;
    jmap(&h, json!([draft_create(&drafts, "Unsent")])).await;
    send(&h, "Sent one", Filing::UpdateIntoSent).await;
    assert_eq!(
        h.imap.uids("Drafts"),
        vec![1],
        "the unsent draft (uid 1) stays"
    );
    assert_eq!(h.imap.expunged(), vec![("Drafts".to_string(), 2)]);
}

/// A server that cannot expunge one message on its own (no UIDPLUS): the copy
/// stays, as it always did there, and the send is final with no error.
#[tokio::test]
async fn a_server_that_cannot_expunge_keeps_the_copy_and_the_send_stands() {
    let h = setup(true).await;
    let resp = send(&h, "Stays", Filing::UpdateIntoSent).await;
    let sub_id = result(&resp, "c2")["created"]["s1"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(h.imap.count("Drafts"), 1);
    let g = jmap(
        &h,
        json!([["EmailSubmission/get", { "ids": [sub_id] }, "g"]]),
    )
    .await;
    let sub = &result(&g, "g")["list"][0];
    assert_eq!(sub["undoStatus"], "final", "{g}");
    assert_eq!(
        sub["mailwomanLastError"],
        Value::Null,
        "not reported as a failure: {g}"
    );
}
