//! 26.20 t28-e17 — folder management and permanent removal, driven through
//! `Engine::handle_jmap` against a backend that keeps a server-side folder
//! namespace.
//!
//! What a JMAP response says is not the evidence here: every case reads the
//! backend's own namespace (what an IMAP `LIST` would show) or its message
//! store (what a fresh `FETCH` would find), and states that precondition before
//! the call. The live counterpart is `mw-server/tests/t28_folders_live.rs`.

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
        for name in ["INBOX", "Trash"] {
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

fn role_for(name: &str) -> MailboxRole {
    match name {
        "INBOX" => MailboxRole::Inbox,
        "Trash" => MailboxRole::Trash,
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

    // t65: a stand-in for modified UTF-7, so the cases can tell the typed
    // name from the server spelling: each non-ASCII char is `&<hex>-`.
    fn encode_mailbox_name(&self, name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_ascii() {
                    c.to_string()
                } else {
                    format!("&{:x}-", c as u32)
                }
            })
            .collect()
    }

    fn decode_mailbox_name(&self, wire: &str) -> String {
        let mut out = String::new();
        let mut rest = wire;
        while let Some(at) = rest.find('&') {
            out.push_str(&rest[..at]);
            let tail = &rest[at + 1..];
            match tail.split_once('-').and_then(|(hex, after)| {
                u32::from_str_radix(hex, 16)
                    .ok()
                    .and_then(char::from_u32)
                    .map(|c| (c, after))
            }) {
                Some((c, after)) => {
                    out.push(c);
                    rest = after;
                }
                None => {
                    out.push('&');
                    rest = tail;
                }
            }
        }
        out.push_str(rest);
        out
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

async fn setup_with(manages: bool, can_expunge: bool) -> Harness {
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
    let backend = Arc::new(FolderBackend::new(manages, can_expunge));
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

async fn mailbox_set(h: &Harness, args: Value) -> Value {
    let resp = jmap(h, json!([["Mailbox/set", args, "s"]])).await;
    result(&resp, "s").clone()
}

async fn email_set(h: &Harness, args: Value) -> Value {
    let resp = jmap(h, json!([["Email/set", args, "s"]])).await;
    result(&resp, "s").clone()
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

async fn email_state(h: &Harness) -> String {
    let resp = jmap(h, json!([["Email/get", { "ids": [] }, "g"]])).await;
    result(&resp, "g")["state"].as_str().unwrap().to_string()
}

// ---------------------------------------------------------------------------
// Cases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_rename_and_delete_reach_the_server() {
    let h = setup().await;
    assert_eq!(h.backend.names(), ["INBOX", "Trash"], "precondition");

    // A parent and a child created in one call, the child naming the parent
    // by its creation id.
    let set = mailbox_set(
        &h,
        json!({ "create": {
            "p": { "name": "Receipts" },
            "c": { "name": "2026", "parentId": "#p" },
        } }),
    )
    .await;
    let parent = set["created"]["p"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("{set}"))
        .to_string();
    let child = set["created"]["c"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("{set}"))
        .to_string();
    assert_eq!(set["created"]["c"]["parentId"], json!(parent));
    assert_eq!(set["created"]["p"]["isSubscribed"], true);
    assert_ne!(set["oldState"], set["newState"], "{set}");
    assert_eq!(
        h.backend.names(),
        ["INBOX", "Receipts", "Receipts.2026", "Trash"]
    );
    assert!(h.backend.subscribed().contains("Receipts.2026"));

    // Mailbox/get shows the hierarchy: the child's leaf name under its parent.
    let all = mailboxes(&h).await;
    assert_eq!(all[&child]["parentId"], json!(parent));
    assert_eq!(all[&child]["name"], "2026");
    assert_eq!(all[&parent]["parentId"], Value::Null);
    assert_eq!(all[&parent]["myRights"]["mayDelete"], true);

    // Rename the parent: the server renames the child with it, and both keep
    // their ids.
    let set = mailbox_set(&h, json!({ "update": { &parent: { "name": "Bills" } } })).await;
    assert!(
        set["updated"].as_object().unwrap().contains_key(&parent),
        "{set}"
    );
    assert_eq!(h.backend.names(), ["Bills", "Bills.2026", "INBOX", "Trash"]);
    // The subscription followed the rename.
    assert!(
        h.backend.subscribed().contains("Bills.2026"),
        "{:?}",
        h.backend.subscribed()
    );
    assert!(!h.backend.subscribed().contains("Receipts.2026"));
    let all = mailboxes(&h).await;
    assert_eq!(all[&parent]["name"], "Bills");
    assert_eq!(all[&child]["parentId"], json!(parent));
    // A resync against the renamed server makes no second row.
    h.engine.resync(&h.account_id).await.unwrap();
    assert_eq!(mailboxes(&h).await.len(), 4);

    // Move the child to the top level.
    let set = mailbox_set(&h, json!({ "update": { &child: { "parentId": null } } })).await;
    assert!(
        set["updated"].as_object().unwrap().contains_key(&child),
        "{set}"
    );
    assert_eq!(h.backend.names(), ["2026", "Bills", "INBOX", "Trash"]);
    // Back under its parent.
    let set = mailbox_set(&h, json!({ "update": { &child: { "parentId": &parent } } })).await;
    assert!(
        set["updated"].as_object().unwrap().contains_key(&child),
        "{set}"
    );
    assert_eq!(h.backend.names(), ["Bills", "Bills.2026", "INBOX", "Trash"]);

    // The parent cannot go while it has a child.
    let set = mailbox_set(&h, json!({ "destroy": [&parent] })).await;
    assert_eq!(
        set["notDestroyed"][&parent]["type"], "mailboxHasChild",
        "{set}"
    );
    assert!(h.backend.names().contains(&"Bills".to_string()));

    // Both in one call: the child is destroyed first.
    let set = mailbox_set(&h, json!({ "destroy": [&parent, &child] })).await;
    assert_eq!(set["destroyed"].as_array().unwrap().len(), 2, "{set}");
    assert_eq!(h.backend.names(), ["INBOX", "Trash"]);
    let all = mailboxes(&h).await;
    assert!(!all.contains_key(&parent) && !all.contains_key(&child));
}

#[tokio::test]
async fn a_folder_with_mail_is_kept_unless_the_request_says_to_remove_it() {
    let h = setup().await;
    let inbox = mailbox_id(&h, "inbox").await;
    let set = mailbox_set(&h, json!({ "create": { "k": { "name": "Keep" } } })).await;
    let keep = set["created"]["k"]["id"].as_str().unwrap().to_string();

    // Move one message in with the RFC 8621 patch-path form (t29 A4).
    let ids = email_ids(&h, &inbox).await;
    assert_eq!(ids.len(), 2, "precondition: two messages in the inbox");
    let moved = &ids[0];
    let set = email_set(
        &h,
        json!({ "update": { moved: {
            format!("mailboxIds/{keep}"): true,
            format!("mailboxIds/{inbox}"): null,
        } } }),
    )
    .await;
    assert!(
        set["updated"].as_object().unwrap().contains_key(moved),
        "{set}"
    );
    assert_eq!(h.backend.count("Keep"), 1, "the move reached the server");
    assert_eq!(email_ids(&h, &keep).await, std::slice::from_ref(moved));

    // Refused without the flag; the folder and its message stay on the server.
    let set = mailbox_set(&h, json!({ "destroy": [&keep] })).await;
    assert_eq!(
        set["notDestroyed"][&keep]["type"], "mailboxHasEmail",
        "{set}"
    );
    assert_eq!(h.backend.count("Keep"), 1);

    // With it, the folder and the message go.
    let before = email_state(&h).await;
    let set = mailbox_set(
        &h,
        json!({ "destroy": [&keep], "onDestroyRemoveEmails": true }),
    )
    .await;
    assert_eq!(set["destroyed"], json!([keep]), "{set}");
    assert!(!h.backend.names().contains(&"Keep".to_string()));
    let changes = jmap(
        &h,
        json!([["Email/changes", { "sinceState": before }, "c"]]),
    )
    .await;
    assert_eq!(
        result(&changes, "c")["destroyed"],
        json!([moved]),
        "{changes}"
    );
    let got = jmap(&h, json!([["Email/get", { "ids": [moved] }, "g"]])).await;
    assert_eq!(result(&got, "g")["notFound"], json!([moved]));
}

#[tokio::test]
async fn special_use_folders_and_bad_names_are_refused_without_touching_the_server() {
    let h = setup().await;
    let inbox = mailbox_id(&h, "inbox").await;
    let trash = mailbox_id(&h, "trash").await;
    let names = h.backend.names();

    let set = mailbox_set(&h, json!({ "destroy": [&inbox, &trash] })).await;
    assert_eq!(set["notDestroyed"][&inbox]["type"], "forbidden", "{set}");
    assert_eq!(set["notDestroyed"][&trash]["type"], "forbidden", "{set}");

    let set = mailbox_set(&h, json!({ "update": { &trash: { "name": "Bin" } } })).await;
    assert_eq!(set["notUpdated"][&trash]["type"], "forbidden", "{set}");

    let refused: [(Value, &str, &str); 6] = [
        (
            json!({ "name": "Junk2", "role": "junk" }),
            "invalidProperties",
            "role",
        ),
        (
            json!({ "name": "Trash" }),
            "alreadyExists",
            "sibling name taken",
        ),
        (
            json!({ "name": "a.b" }),
            "invalidProperties",
            "delimiter in name",
        ),
        (
            json!({ "name": "Caf\u{7}e" }),
            "invalidProperties",
            "control character in name",
        ),
        (
            json!({ "name": "X", "parentId": "nope" }),
            "invalidProperties",
            "unknown parent",
        ),
        (
            json!({ "name": "X", "totalEmails": 3 }),
            "invalidProperties",
            "server-set property",
        ),
    ];
    for (spec, kind, context) in refused {
        let set = mailbox_set(&h, json!({ "create": { "x": spec } })).await;
        assert_eq!(set["notCreated"]["x"]["type"], kind, "{context}: {set}");
        assert_eq!(set["oldState"], set["newState"], "{context}: {set}");
    }
    assert_eq!(h.backend.names(), names, "no refusal reached the server");

    // The accepted form beside them.
    let set = mailbox_set(&h, json!({ "create": { "x": { "name": "Junk2" } } })).await;
    assert!(set["created"]["x"]["id"].is_string(), "{set}");
}

#[tokio::test]
async fn destroy_removes_the_message_from_the_server() {
    let h = setup().await;
    let inbox = mailbox_id(&h, "inbox").await;
    let ids = email_ids(&h, &inbox).await;
    assert_eq!(h.backend.count("INBOX"), 2, "precondition");
    let gone = &ids[0];
    let before = email_state(&h).await;

    let set = email_set(&h, json!({ "destroy": [gone, "no-such-id"] })).await;
    assert_eq!(set["destroyed"], json!([gone]), "{set}");
    assert_eq!(
        set["notDestroyed"]["no-such-id"]["type"], "notFound",
        "{set}"
    );
    assert_eq!(h.backend.count("INBOX"), 1, "the server no longer has it");
    assert_eq!(h.backend.expunges().len(), 1);
    assert_eq!(
        h.backend.count("Trash"),
        0,
        "destroy is not a move to Trash"
    );

    assert_eq!(email_ids(&h, &inbox).await, [ids[1].clone()]);
    let changes = jmap(
        &h,
        json!([["Email/changes", { "sinceState": before }, "c"]]),
    )
    .await;
    assert_eq!(
        result(&changes, "c")["destroyed"],
        json!([gone]),
        "{changes}"
    );

    // An id named by both update and destroy is not updated.
    let other = &ids[1];
    let set = email_set(
        &h,
        json!({ "update": { other: { "keywords/$seen": true } }, "destroy": [other] }),
    )
    .await;
    assert_eq!(set["notUpdated"][other]["type"], "willDestroy", "{set}");
    assert_eq!(set["destroyed"], json!([other]), "{set}");
    assert_eq!(h.backend.count("INBOX"), 0);
}

#[tokio::test]
async fn a_server_that_cannot_expunge_one_message_gets_it_moved_to_trash() {
    let h = setup_with(true, false).await;
    let inbox = mailbox_id(&h, "inbox").await;
    let trash = mailbox_id(&h, "trash").await;
    let ids = email_ids(&h, &inbox).await;
    let id = &ids[0];

    let set = email_set(&h, json!({ "destroy": [id] })).await;
    assert_eq!(set["destroyed"], json!([]), "it still exists: {set}");
    assert_eq!(set["notDestroyed"][id]["type"], "forbidden", "{set}");
    assert_eq!(
        set["notDestroyed"][id]["mailwomanMovedTo"],
        json!(trash),
        "{set}"
    );
    assert_eq!(h.backend.count("Trash"), 1);
    assert_eq!(h.backend.count("INBOX"), 1);
    assert_eq!(email_ids(&h, &trash).await, std::slice::from_ref(id));

    // Already in Trash: nothing more can be done, and nothing moves.
    let set = email_set(&h, json!({ "destroy": [id] })).await;
    assert_eq!(set["notDestroyed"][id]["type"], "forbidden", "{set}");
    assert!(
        set["notDestroyed"][id].get("mailwomanMovedTo").is_none(),
        "{set}"
    );
    assert_eq!(h.backend.count("Trash"), 1);
}

#[tokio::test]
async fn an_account_without_folder_verbs_says_so() {
    let imap = setup().await;
    let pop = setup_with(false, true).await;

    let session = imap
        .engine
        .session_json_for(&imap.account_id, "me@example.org");
    let caps =
        &session["accounts"][&imap.account_id]["accountCapabilities"]["urn:ietf:params:jmap:mail"];
    assert_eq!(caps["mailwomanMailboxSet"], true, "precondition: {session}");
    assert_eq!(caps["mayCreateTopLevelMailbox"], true);

    let session = pop
        .engine
        .session_json_for(&pop.account_id, "me@example.org");
    let caps =
        &session["accounts"][&pop.account_id]["accountCapabilities"]["urn:ietf:params:jmap:mail"];
    assert_eq!(caps["mailwomanMailboxSet"], false, "{session}");
    assert_eq!(caps["mayCreateTopLevelMailbox"], false);

    let set = mailbox_set(&pop, json!({ "create": { "x": { "name": "New" } } })).await;
    assert_eq!(set["type"], "accountNotSupportedByMethod", "{set}");
    assert_eq!(pop.backend.names(), ["INBOX", "Trash"]);
    let all = mailboxes(&pop).await;
    assert!(
        all.values()
            .all(|m| m["myRights"]["mayCreateChild"] == false)
    );
}

#[tokio::test]
async fn delete_checks_the_server_not_only_the_cache() {
    let h = setup().await;
    let set = mailbox_set(&h, json!({ "create": { "s": { "name": "Stale" } } })).await;
    let stale = set["created"]["s"]["id"].as_str().unwrap().to_string();

    // Another client files a message and makes a subfolder; no sync runs.
    h.backend
        .server
        .lock()
        .unwrap()
        .deliver("Stale", message(7));
    assert!(
        email_ids(&h, &stale).await.is_empty(),
        "precondition: the cache has not seen it"
    );
    let set = mailbox_set(&h, json!({ "destroy": [&stale] })).await;
    assert_eq!(
        set["notDestroyed"][&stale]["type"], "mailboxHasEmail",
        "{set}"
    );
    assert_eq!(h.backend.count("Stale"), 1);

    h.backend.server.lock().unwrap().add_folder("Stale.Sub");
    let set = mailbox_set(
        &h,
        json!({ "destroy": [&stale], "onDestroyRemoveEmails": true }),
    )
    .await;
    assert_eq!(
        set["notDestroyed"][&stale]["type"], "mailboxHasChild",
        "{set}"
    );
    assert!(h.backend.names().contains(&"Stale".to_string()));
}

// ---------------------------------------------------------------------------
// t65 — Rename from the folder menu
// ---------------------------------------------------------------------------

/// A rename to a name outside ASCII reaches the server in the backend's
/// spelling, for the folder and its child; the browser reads the typed name
/// back, and the ids, the nesting and the subscription stay with the folder.
#[tokio::test]
async fn t65_a_non_ascii_rename_is_sent_in_the_server_spelling_and_read_back_as_typed() {
    let h = setup().await;
    let set = mailbox_set(
        &h,
        json!({ "create": { "p": { "name": "Work" }, "c": { "name": "Q3", "parentId": "#p" } } }),
    )
    .await;
    let parent = set["created"]["p"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("{set}"))
        .to_string();
    let child = set["created"]["c"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("{set}"))
        .to_string();

    let set = mailbox_set(
        &h,
        json!({ "update": { &parent: { "name": "Relatórios" } } }),
    )
    .await;
    assert!(
        set["updated"].as_object().unwrap().contains_key(&parent),
        "{set}"
    );
    assert_eq!(
        h.backend.names(),
        ["INBOX", "Relat&f3-rios", "Relat&f3-rios.Q3", "Trash"],
        "the server holds the encoded spelling, the child moved with it"
    );
    assert!(h.backend.subscribed().contains("Relat&f3-rios.Q3"));
    let all = mailboxes(&h).await;
    assert_eq!(all[&parent]["name"], "Relatórios");
    assert_eq!(all[&parent]["mailwomanUpstreamName"], "Relat&f3-rios");
    assert_eq!(all[&child]["name"], "Q3");
    assert_eq!(all[&child]["parentId"], json!(parent));

    // A resync against the renamed server keeps the same rows.
    h.engine.resync(&h.account_id).await.unwrap();
    let all = mailboxes(&h).await;
    assert_eq!(all.len(), 4, "{all:?}");
    assert_eq!(all[&parent]["name"], "Relatórios");

    // The same typed name for a sibling is a duplicate in the server spelling.
    let set = mailbox_set(&h, json!({ "create": { "x": { "name": "Relatórios" } } })).await;
    assert_eq!(set["notCreated"]["x"]["type"], "alreadyExists", "{set}");
}

/// Rename refusals: a sibling's name, the delimiter, an empty name, a role
/// folder. None of them reaches the server.
#[tokio::test]
async fn t65_rename_refuses_a_sibling_name_a_role_folder_and_the_delimiter() {
    let h = setup().await;
    let set = mailbox_set(
        &h,
        json!({ "create": { "a": { "name": "Alpha" }, "b": { "name": "Beta" } } }),
    )
    .await;
    let beta = set["created"]["b"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("{set}"))
        .to_string();
    let inbox = mailbox_id(&h, "inbox").await;
    let trash = mailbox_id(&h, "trash").await;
    let names = h.backend.names();

    let set = mailbox_set(&h, json!({ "update": { &beta: { "name": "Alpha" } } })).await;
    assert_eq!(set["notUpdated"][&beta]["type"], "alreadyExists", "{set}");
    let set = mailbox_set(&h, json!({ "update": { &beta: { "name": "Al.pha" } } })).await;
    assert_eq!(
        set["notUpdated"][&beta]["type"], "invalidProperties",
        "{set}"
    );
    let set = mailbox_set(&h, json!({ "update": { &beta: { "name": "" } } })).await;
    assert_eq!(
        set["notUpdated"][&beta]["type"], "invalidProperties",
        "{set}"
    );
    for role in [&inbox, &trash] {
        let set = mailbox_set(&h, json!({ "update": { role: { "name": "Renamed" } } })).await;
        assert_eq!(set["notUpdated"][role]["type"], "forbidden", "{set}");
    }
    let all = mailboxes(&h).await;
    assert_eq!(all[&inbox]["myRights"]["mayRename"], false);
    assert_eq!(all[&trash]["myRights"]["mayRename"], false);
    assert_eq!(all[&beta]["myRights"]["mayRename"], true);
    assert_eq!(h.backend.names(), names, "no refusal reached the server");

    // A case-only change of the folder's own name is not a duplicate.
    let set = mailbox_set(&h, json!({ "update": { &beta: { "name": "BETA" } } })).await;
    assert!(
        set["updated"].as_object().unwrap().contains_key(&beta),
        "{set}"
    );
}

/// A folder id of one account is unknown to another account on the same
/// engine: the rename is `notFound` and neither server changes.
#[tokio::test]
async fn t65_rename_only_reaches_the_accounts_own_folders() {
    let owner = setup().await;
    let set = mailbox_set(&owner, json!({ "create": { "o": { "name": "Mine" } } })).await;
    let id = set["created"]["o"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("{set}"))
        .to_string();

    let other_id = owner
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
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    let other_backend = Arc::new(FolderBackend::new(true, true));
    owner.engine.register_backend(
        other_id.clone(),
        AccountRuntime::new(
            other_backend.clone() as Arc<dyn AccountBackend>,
            Arc::new(NoSubmitter) as Arc<dyn MailSubmitter>,
            "other@example.org",
        ),
    );
    owner.engine.resync(&other_id).await.unwrap();
    let other = Harness {
        engine: owner.engine.clone(),
        account_id: other_id,
        backend: other_backend,
    };

    let set = mailbox_set(&other, json!({ "update": { &id: { "name": "Taken" } } })).await;
    assert_eq!(set["notUpdated"][&id]["type"], "notFound", "{set}");
    assert!(owner.backend.names().contains(&"Mine".to_string()));
    assert!(!owner.backend.names().contains(&"Taken".to_string()));
    assert!(!other.backend.names().contains(&"Taken".to_string()));
}
