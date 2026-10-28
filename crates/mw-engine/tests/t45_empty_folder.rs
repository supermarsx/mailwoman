//! t45 — `Mailbox/empty`: empty any folder in bounded batches, driven through
//! `Engine::handle_jmap` against a backend that keeps a server-side folder
//! namespace (the harness of `t28_folders.rs`).
//!
//! Every case reads the backend's own message store (what a fresh `FETCH`
//! would find) and the backend calls it received, not only the JMAP answer.
//! The live IMAP counterpart is `mw-server/tests/t45_empty_folder_live.rs`.

use std::collections::{BTreeMap, BTreeSet};
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
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

const DELIM: char = '.';

// ---------------------------------------------------------------------------
// A server with folders
// ---------------------------------------------------------------------------

#[derive(Default, Clone)]
struct Folder {
    uidvalidity: u32,
    next_uid: u32,
    /// uid → raw message.
    messages: BTreeMap<u32, Vec<u8>>,
}

#[derive(Default)]
struct Server {
    folders: BTreeMap<String, Folder>,
    subscribed: BTreeSet<String>,
    next_uidvalidity: u32,
    /// Every `expunge_messages` call: (mailbox, uids).
    expunges: Vec<(String, Vec<u32>)>,
    /// Every `move_messages` call: (from, to, how many).
    moves: Vec<(String, String, usize)>,
}

impl Server {
    fn add_folder(&mut self, name: &str) {
        self.next_uidvalidity += 1;
        self.folders.insert(
            name.to_string(),
            Folder {
                uidvalidity: self.next_uidvalidity,
                next_uid: 1,
                messages: BTreeMap::new(),
            },
        );
    }

    fn deliver(&mut self, folder: &str, raw: Vec<u8>) -> u32 {
        let f = self.folders.get_mut(folder).expect("folder exists");
        let uid = f.next_uid;
        f.next_uid += 1;
        f.messages.insert(uid, raw);
        uid
    }
}

struct FolderBackend {
    server: Mutex<Server>,
    /// `false` behaves like a POP3 or bridge backend: no folder verbs.
    manages: bool,
    /// `false` behaves like an IMAP server that cannot expunge one message.
    can_expunge: bool,
}

impl FolderBackend {
    fn new(manages: bool, can_expunge: bool) -> Self {
        let mut server = Server::default();
        for name in ["INBOX", "Trash", "Junk", "Work", "Work.Child"] {
            server.add_folder(name);
        }
        server.deliver("INBOX", message(0));
        server.deliver("INBOX", message(1));
        Self {
            server: Mutex::new(server),
            manages,
            can_expunge,
        }
    }

    /// The server's folder names, as a fresh `LIST` would show them.
    fn names(&self) -> Vec<String> {
        self.server
            .lock()
            .unwrap()
            .folders
            .keys()
            .cloned()
            .collect()
    }

    /// How many messages the server holds in `folder`.
    fn count(&self, folder: &str) -> usize {
        self.server
            .lock()
            .unwrap()
            .folders
            .get(folder)
            .map_or(0, |f| f.messages.len())
    }

    fn moves(&self) -> Vec<(String, String, usize)> {
        self.server.lock().unwrap().moves.clone()
    }

    fn deliver_many(&self, folder: &str, n: usize) {
        let mut st = self.server.lock().unwrap();
        for i in 0..n {
            st.deliver(folder, numbered(folder, i));
        }
    }

    fn subscribed(&self) -> BTreeSet<String> {
        self.server.lock().unwrap().subscribed.clone()
    }

    fn expunges(&self) -> Vec<(String, Vec<u32>)> {
        self.server.lock().unwrap().expunges.clone()
    }

    fn refuse(&self, verb: &str) -> Result<()> {
        if self.manages {
            Ok(())
        } else {
            Err(EngineError::Unsupported(verb.into()))
        }
    }
}

fn message(i: usize) -> Vec<u8> {
    format!(
        "Message-ID: <folder-{i}@x>\r\n\
         From: alice@example.org\r\n\
         To: me@example.org\r\n\
         Subject: Folder test {i}\r\n\
         Date: 0{} Jul 2026 09:00:00 +0000\r\n\
         \r\n\
         body {i}\r\n",
        i + 1
    )
    .into_bytes()
}

fn numbered(folder: &str, i: usize) -> Vec<u8> {
    format!(
        "Message-ID: <empty-{folder}-{i}@x>\r\n\
         From: alice@example.org\r\n\
         To: me@example.org\r\n\
         Subject: Empty test {folder} {i}\r\n\
         Date: 01 Jul 2026 09:00:00 +0000\r\n\
         \r\n\
         body {i}\r\n"
    )
    .into_bytes()
}

fn role_for(name: &str) -> MailboxRole {
    match name {
        "INBOX" => MailboxRole::Inbox,
        "Trash" => MailboxRole::Trash,
        "Junk" => MailboxRole::Junk,
        _ => MailboxRole::None,
    }
}

#[async_trait]
impl AccountBackend for FolderBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: self.can_expunge,
            r#move: true,
            ..BackendCaps::default()
        })
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        let st = self.server.lock().unwrap();
        Ok(st
            .folders
            .iter()
            .map(|(name, f)| RawMailbox {
                mailbox_ref: RawMailboxRef {
                    name: name.clone(),
                    uidvalidity: f.uidvalidity,
                },
                role: role_for(name),
                parent: name.rsplit_once(DELIM).map(|(p, _)| p.to_string()),
                uidnext: f.next_uid,
                highestmodseq: 0,
                total: f.messages.len() as u32,
                unread: f.messages.len() as u32,
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
        let st = self.server.lock().unwrap();
        let f = st
            .folders
            .get(&mbox.name)
            .ok_or_else(|| EngineError::MailboxNotFound(mbox.name.clone()))?;
        Ok(MailboxDelta {
            added: f
                .messages
                .keys()
                .filter(|uid| **uid >= from)
                .map(|uid| MessageRef::Imap {
                    mailbox: mbox.clone(),
                    uidvalidity: f.uidvalidity,
                    uid: *uid,
                })
                .collect(),
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: f.uidvalidity,
                uidnext: f.next_uid,
            },
        })
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        let st = self.server.lock().unwrap();
        Ok(refs
            .iter()
            .filter_map(|r| {
                let MessageRef::Imap { mailbox, uid, .. } = r else {
                    return None;
                };
                let raw = st.folders.get(&mailbox.name)?.messages.get(uid)?.clone();
                Some(RawMessage {
                    message_ref: r.clone(),
                    raw,
                    flags: Vec::new(),
                    internaldate: Some("2026-07-01T09:00:00Z".into()),
                })
            })
            .collect())
    }

    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _d: &[Flag]) -> Result<()> {
        Ok(())
    }

    async fn move_messages(&self, refs: &[MessageRef], to: &RawMailboxRef) -> Result<MoveOutcome> {
        let mut st = self.server.lock().unwrap();
        if let Some(MessageRef::Imap { mailbox, .. }) = refs.first() {
            let from = mailbox.name.clone();
            st.moves.push((from, to.name.clone(), refs.len()));
        }
        let mut uids = Vec::new();
        for r in refs {
            let MessageRef::Imap { mailbox, uid, .. } = r else {
                continue;
            };
            let raw = st
                .folders
                .get_mut(&mailbox.name)
                .and_then(|f| f.messages.remove(uid))
                .ok_or_else(|| EngineError::Protocol(format!("no uid {uid}")))?;
            uids.push(st.deliver(&to.name, raw));
        }
        let uidvalidity = st.folders[&to.name].uidvalidity;
        Ok(MoveOutcome::Uidplus { uidvalidity, uids })
    }

    async fn append(&self, mbox: &RawMailboxRef, raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
        let mut st = self.server.lock().unwrap();
        let uid = st.deliver(&mbox.name, raw.to_vec());
        Ok(MessageRef::Imap {
            mailbox: mbox.clone(),
            uidvalidity: st.folders[&mbox.name].uidvalidity,
            uid,
        })
    }

    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }

    fn manages_mailboxes(&self) -> bool {
        self.manages
    }

    async fn hierarchy_delimiter(&self) -> Result<Option<char>> {
        self.refuse("hierarchy_delimiter")?;
        Ok(Some(DELIM))
    }

    async fn create_mailbox(&self, name: &str) -> Result<()> {
        self.refuse("create_mailbox")?;
        let mut st = self.server.lock().unwrap();
        if st.folders.contains_key(name) {
            return Err(EngineError::Protocol("ALREADYEXISTS".into()));
        }
        st.add_folder(name);
        Ok(())
    }

    async fn rename_mailbox(&self, from: &RawMailboxRef, to: &str) -> Result<()> {
        self.refuse("rename_mailbox")?;
        let mut st = self.server.lock().unwrap();
        let prefix = format!("{}{DELIM}", from.name);
        let moving: Vec<String> = st
            .folders
            .keys()
            .filter(|n| **n == from.name || n.starts_with(&prefix))
            .cloned()
            .collect();
        if moving.is_empty() {
            return Err(EngineError::MailboxNotFound(from.name.clone()));
        }
        for old in moving {
            let folder = st.folders.remove(&old).unwrap();
            let new = format!("{to}{}", &old[from.name.len()..]);
            st.folders.insert(new, folder);
        }
        Ok(())
    }

    async fn delete_mailbox(&self, mbox: &RawMailboxRef) -> Result<()> {
        self.refuse("delete_mailbox")?;
        let mut st = self.server.lock().unwrap();
        st.folders
            .remove(&mbox.name)
            .map(|_| ())
            .ok_or_else(|| EngineError::MailboxNotFound(mbox.name.clone()))
    }

    async fn subscribe(&self, mbox: &RawMailboxRef, subscribed: bool) -> Result<()> {
        self.refuse("subscribe")?;
        let mut st = self.server.lock().unwrap();
        if subscribed {
            st.subscribed.insert(mbox.name.clone());
        } else {
            st.subscribed.remove(&mbox.name);
        }
        Ok(())
    }

    async fn subscribed_mailboxes(&self) -> Result<Vec<String>> {
        self.refuse("subscribed_mailboxes")?;
        Ok(self.subscribed().into_iter().collect())
    }

    async fn expunge_messages(&self, refs: &[MessageRef]) -> Result<()> {
        if !self.can_expunge {
            return Err(EngineError::Unsupported(
                "no UIDPLUS and other messages are marked deleted".into(),
            ));
        }
        let mut st = self.server.lock().unwrap();
        for r in refs {
            let MessageRef::Imap { mailbox, uid, .. } = r else {
                continue;
            };
            if let Some(f) = st.folders.get_mut(&mailbox.name) {
                f.messages.remove(uid);
            }
            st.expunges.push((mailbox.name.clone(), vec![*uid]));
        }
        st.moves.push(("expunge".into(), String::new(), refs.len()));
        Ok(())
    }
}

struct NoSubmitter;

#[async_trait]
impl MailSubmitter for NoSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
    backend: Arc<FolderBackend>,
}

async fn add_account(store: &Store, user: &str) -> String {
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: user,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: user.into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap()
}

async fn setup_with(manages: bool, can_expunge: bool) -> Harness {
    setup_backend(FolderBackend::new(manages, can_expunge)).await
}

async fn setup_backend(backend: FolderBackend) -> Harness {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = add_account(&store, "me@example.org").await;
    let engine = Arc::new(Engine::new(store));
    let backend = Arc::new(backend);
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            backend.clone() as Arc<dyn AccountBackend>,
            Arc::new(NoSubmitter) as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    engine.resync(&account_id).await.unwrap();
    Harness {
        engine,
        account_id,
        backend,
    }
}

async fn setup() -> Harness {
    setup_with(true, true).await
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

/// Every mailbox `Mailbox/get` lists, by id.
async fn mailboxes(h: &Harness) -> BTreeMap<String, Value> {
    let resp = jmap(h, json!([["Mailbox/get", { "properties": null }, "g"]])).await;
    result(&resp, "g")["list"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| (m["id"].as_str().unwrap().to_string(), m.clone()))
        .collect()
}

async fn mailbox_id(h: &Harness, role_or_name: &str) -> String {
    mailboxes(h)
        .await
        .into_iter()
        .find(|(_, m)| m["role"] == role_or_name || m["name"] == role_or_name)
        .unwrap_or_else(|| panic!("no mailbox {role_or_name}"))
        .0
}

async fn email_ids(h: &Harness, mailbox: &str) -> Vec<String> {
    let resp = jmap(
        h,
        json!([["Email/query", { "filter": { "inMailbox": mailbox } }, "q"]]),
    )
    .await;
    result(&resp, "q")["ids"]
        .as_array()
        .unwrap_or_else(|| panic!("Email/query: {resp}"))
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

async fn empty(h: &Harness, args: Value) -> Value {
    let resp = jmap(h, json!([["Mailbox/empty", args, "e"]])).await;
    let call = resp["methodResponses"][0].clone();
    if call[0] == "error" {
        json!({ "error": call[1] })
    } else {
        call[1].clone()
    }
}

/// Preview, then call batch after batch the way the web client does, until
/// the folder is empty or a batch makes no progress. Returns every response.
async fn empty_all(h: &Harness, mailbox: &str, outcome: &str, limit: u64) -> Vec<Value> {
    let preview = empty(
        h,
        json!({ "mailboxId": mailbox, "outcome": outcome, "expectedTotal": 0, "limit": 0 }),
    )
    .await;
    // A preview with a wrong count is refused and names the real one.
    let mut expected = preview["total"]
        .as_u64()
        .or_else(|| preview["error"]["mailwomanTotal"].as_u64())
        .unwrap_or_else(|| panic!("preview: {preview}"));
    let mut out = Vec::new();
    loop {
        let r = empty(
            h,
            json!({ "mailboxId": mailbox, "outcome": outcome, "expectedTotal": expected, "limit": limit }),
        )
        .await;
        let processed = r["processed"]
            .as_u64()
            .unwrap_or_else(|| panic!("batch: {r}"));
        expected = r["remaining"].as_u64().unwrap();
        out.push(r);
        if expected == 0 || processed == 0 {
            return out;
        }
        assert!(out.len() < 50, "runaway");
    }
}

fn empty_capability(session: &Value, account: &str) -> Value {
    session["accounts"][account]["accountCapabilities"]["urn:ietf:params:jmap:mail"]
        ["mailwomanEmptyMailbox"]
        .clone()
}

#[tokio::test]
async fn an_ordinary_folder_moves_to_trash_in_bounded_batches_and_leaves_children() {
    let h = setup().await;
    h.backend.deliver_many("Work", 250);
    h.backend.deliver_many("Work.Child", 3);
    h.engine.resync(&h.account_id).await.unwrap();
    assert_eq!(h.backend.count("Work"), 250, "precondition");
    assert_eq!(h.backend.count("Trash"), 0, "precondition");
    let work = mailbox_id(&h, "Work").await;
    let trash = mailbox_id(&h, "trash").await;

    let responses = empty_all(&h, &work, "trash", 100).await;
    let processed: Vec<u64> = responses
        .iter()
        .map(|r| r["processed"].as_u64().unwrap())
        .collect();
    assert_eq!(processed, [100, 100, 50], "{responses:?}");
    assert_eq!(responses[0]["outcome"], "trash");
    assert_eq!(responses[0]["trashId"], json!(trash));
    assert_eq!(responses[0]["total"], 250);

    // What the server now holds: nothing in Work, all 250 in Trash, the child
    // folder and its messages untouched.
    assert_eq!(h.backend.count("Work"), 0);
    assert_eq!(h.backend.count("Trash"), 250);
    assert_eq!(h.backend.count("Work.Child"), 3);
    assert!(h.backend.names().contains(&"Work.Child".to_string()));
    // One backend move per batch, never more than the batch size.
    let moves: Vec<usize> = h
        .backend
        .moves()
        .into_iter()
        .filter(|(from, to, _)| from == "Work" && to == "Trash")
        .map(|(_, _, n)| n)
        .collect();
    assert_eq!(moves, [100, 100, 50]);
    assert!(
        h.backend.expunges().is_empty(),
        "a move to Trash expunges nothing itself"
    );
    // The cache agrees, so the client sees them in Trash.
    assert!(email_ids(&h, &work).await.is_empty());
    let child = mailbox_id(&h, "Child").await;
    assert_eq!(email_ids(&h, &child).await.len(), 3);
}

#[tokio::test]
async fn trash_and_junk_are_emptied_for_good() {
    let h = setup().await;
    h.backend.deliver_many("Trash", 4);
    h.backend.deliver_many("Junk", 2);
    h.engine.resync(&h.account_id).await.unwrap();
    let trash = mailbox_id(&h, "trash").await;
    let junk = mailbox_id(&h, "junk").await;

    // Asking to "move to Trash" from Trash is the wrong outcome: refused, and
    // nothing changes.
    let wrong = empty(
        &h,
        json!({ "mailboxId": trash, "outcome": "trash", "expectedTotal": 4 }),
    )
    .await;
    assert_eq!(wrong["error"]["type"], "stateMismatch", "{wrong}");
    assert_eq!(wrong["error"]["mailwomanOutcome"], "destroy");
    assert_eq!(h.backend.count("Trash"), 4);

    let r = empty_all(&h, &trash, "destroy", 100).await;
    assert_eq!(r[0]["processed"], 4, "{r:?}");
    assert_eq!(h.backend.count("Trash"), 0);
    let r = empty_all(&h, &junk, "destroy", 100).await;
    assert_eq!(r[0]["processed"], 2, "{r:?}");
    assert_eq!(h.backend.count("Junk"), 0);
    // Inbox untouched.
    assert_eq!(h.backend.count("INBOX"), 2);
    assert_eq!(h.backend.expunges().len(), 6);
}

#[tokio::test]
async fn without_a_trash_folder_emptying_is_permanent_and_must_be_asked_for_as_such() {
    let backend = FolderBackend::new(true, true);
    backend.server.lock().unwrap().folders.remove("Trash");
    let h = setup_backend(backend).await;
    assert!(
        !h.backend.names().contains(&"Trash".to_string()),
        "precondition"
    );
    let inbox = mailbox_id(&h, "inbox").await;
    let wrong = empty(
        &h,
        json!({ "mailboxId": inbox, "outcome": "trash", "expectedTotal": 2 }),
    )
    .await;
    assert_eq!(wrong["error"]["mailwomanOutcome"], "destroy", "{wrong}");
    assert_eq!(h.backend.count("INBOX"), 2);
    let r = empty_all(&h, &inbox, "destroy", 100).await;
    assert_eq!(r[0]["outcome"], "destroy");
    assert_eq!(h.backend.count("INBOX"), 0);
}

#[tokio::test]
async fn a_stale_count_is_refused_before_anything_moves() {
    let h = setup().await;
    h.backend.deliver_many("Work", 40);
    h.engine.resync(&h.account_id).await.unwrap();
    let work = mailbox_id(&h, "Work").await;
    // The dialog said 3; the folder holds 40.
    let stale = empty(
        &h,
        json!({ "mailboxId": work, "outcome": "trash", "expectedTotal": 3 }),
    )
    .await;
    assert_eq!(stale["error"]["type"], "stateMismatch", "{stale}");
    assert_eq!(stale["error"]["mailwomanTotal"], 40);
    assert_eq!(h.backend.count("Work"), 40);
    assert!(h.backend.moves().is_empty());
    // A drift within the allowance (mail arriving while the dialog is open) is
    // accepted.
    let ok = empty(
        &h,
        json!({ "mailboxId": work, "outcome": "trash", "expectedTotal": 39, "limit": 10 }),
    )
    .await;
    assert_eq!(ok["processed"], 10, "{ok}");
    assert_eq!(ok["remaining"], 30);
    // The batch size is bounded.
    let big = empty(
        &h,
        json!({ "mailboxId": work, "outcome": "trash", "expectedTotal": 30, "limit": 5000 }),
    )
    .await;
    assert_eq!(big["error"]["type"], "invalidArguments", "{big}");
    assert_eq!(h.backend.count("Work"), 30);
}

#[tokio::test]
async fn another_accounts_folder_is_refused() {
    let h = setup().await;
    let other_id = add_account(h.engine.store(), "other@example.org").await;
    let other_backend = Arc::new(FolderBackend::new(true, true));
    other_backend.deliver_many("Work", 5);
    h.engine.register_backend(
        other_id.clone(),
        AccountRuntime::new(
            other_backend.clone() as Arc<dyn AccountBackend>,
            Arc::new(NoSubmitter) as Arc<dyn MailSubmitter>,
            "other@example.org",
        ),
    );
    h.engine.resync(&other_id).await.unwrap();
    let other = Harness {
        engine: h.engine.clone(),
        account_id: other_id.clone(),
        backend: other_backend.clone(),
    };
    let foreign_work = mailbox_id(&other, "Work").await;

    // Account 1 names account 2's folder: refused, nothing moved.
    let refused = empty(
        &h,
        json!({ "mailboxId": foreign_work, "outcome": "trash", "expectedTotal": 5 }),
    )
    .await;
    assert_eq!(refused["error"]["type"], "invalidArguments", "{refused}");
    assert_eq!(other_backend.count("Work"), 5);
    assert!(other_backend.moves().is_empty());
    // Naming account 2 as the accountId does not lend its authority either.
    let refused = empty(
        &h,
        json!({ "accountId": other_id, "mailboxId": foreign_work, "outcome": "trash", "expectedTotal": 5 }),
    )
    .await;
    assert_eq!(refused["error"]["type"], "accountNotFound", "{refused}");
    assert_eq!(other_backend.count("Work"), 5);
    // An invented id is not a folder of this account.
    let refused = empty(
        &h,
        json!({ "mailboxId": "no-such", "outcome": "trash", "expectedTotal": 0 }),
    )
    .await;
    assert_eq!(refused["error"]["type"], "invalidArguments");
}

#[tokio::test]
async fn a_backend_without_folders_cannot_empty_and_the_session_says_so() {
    // `manages: false` behaves like POP3.
    let h = setup_with(false, true).await;
    let inbox = mailbox_id(&h, "inbox").await;
    let r = empty(
        &h,
        json!({ "mailboxId": inbox, "outcome": "trash", "expectedTotal": 2 }),
    )
    .await;
    assert_eq!(r["error"]["type"], "accountNotSupportedByMethod", "{r}");
    assert_eq!(h.backend.count("INBOX"), 2);
    let session = h.engine.session_json_for(&h.account_id, "me@example.org");
    let cap = empty_capability(&session, &h.account_id);
    assert_eq!(cap["moveToTrash"], false, "{session}");
    assert_eq!(cap["destroy"], false);

    let h = setup().await;
    let session = h.engine.session_json_for(&h.account_id, "me@example.org");
    let cap = empty_capability(&session, &h.account_id);
    assert_eq!(cap["moveToTrash"], true, "{session}");
    assert_eq!(cap["destroy"], true);
    assert_eq!(cap["maxBatch"], 200);
}

#[tokio::test]
async fn the_preview_changes_nothing() {
    let h = setup().await;
    let inbox = mailbox_id(&h, "inbox").await;
    let p = empty(
        &h,
        json!({ "mailboxId": inbox, "outcome": "trash", "expectedTotal": 2, "limit": 0 }),
    )
    .await;
    assert_eq!(p["total"], 2, "{p}");
    assert_eq!(p["processed"], 0);
    assert_eq!(h.backend.count("INBOX"), 2);
    assert!(h.backend.moves().is_empty());
}
