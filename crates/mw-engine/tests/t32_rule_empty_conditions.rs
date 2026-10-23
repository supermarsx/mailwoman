//! t32-h2 (t32-ext.md §2 D-3): a `MailRule` whose conditions all drop when it
//! is converted to a Sieve rule (today: a rule whose only condition is
//! `thread`) used to be stored with no conditions, and a rule with no
//! conditions matches every message — so it moved or tagged all new inbox
//! mail. `MailRule/set` now refuses such a create or update with
//! `invalidProperties` and never stores an empty-condition rule from a
//! request that had conditions.
//!
//! Driven through `Engine::handle_jmap` with `MailRule/set`, the method the
//! web client's rules slice calls, over a scripted backend whose inbox holds
//! three messages that the first sync delivers (and so runs rules on).

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, Flag, MailboxDelta, MailboxRole, MessageRef,
    MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor, WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

const UIDVALIDITY: u32 = 100;

// ── harness ──────────────────────────────────────────────────────────────────

struct InboxBackend {
    messages: HashMap<String, Vec<(u32, Vec<u8>)>>,
    next_moved_uid: AtomicU32,
}

impl InboxBackend {
    fn new() -> Self {
        let mut messages = HashMap::new();
        messages.insert(
            "INBOX".to_string(),
            vec![
                (1, msg("invoice", "alice", "Invoice 2026")),
                (2, msg("lunch", "bob", "Lunch")),
                (3, msg("report", "carol", "Weekly Report")),
            ],
        );
        Self {
            messages,
            next_moved_uid: AtomicU32::new(9001),
        }
    }
}

fn msg(id: &str, from: &str, subject: &str) -> Vec<u8> {
    format!(
        "Message-ID: <{id}@x>\r\nFrom: {from}@example.org\r\nTo: me@example.org\r\n\
         Subject: {subject}\r\nDate: Wed, 01 Jul 2026 09:00:00 +0000\r\n\r\nbody\r\n"
    )
    .into_bytes()
}

#[async_trait]
impl AccountBackend for InboxBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: true,
            r#move: true,
            special_use: true,
            ..BackendCaps::default()
        })
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok([
            ("INBOX", MailboxRole::Inbox),
            ("Archive", MailboxRole::Archive),
        ]
        .into_iter()
        .map(|(name, role)| {
            let total = self.messages.get(name).map(|m| m.len()).unwrap_or(0) as u32;
            RawMailbox {
                mailbox_ref: RawMailboxRef {
                    name: name.to_string(),
                    uidvalidity: UIDVALIDITY,
                },
                role,
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
        let msgs = self.messages.get(&mbox.name).cloned().unwrap_or_default();
        let added = msgs
            .iter()
            .filter(|(uid, _)| *uid >= uidnext_from)
            .map(|(uid, _)| MessageRef::Imap {
                mailbox: mbox.clone(),
                uidvalidity: UIDVALIDITY,
                uid: *uid,
            })
            .collect();
        let max_uid = msgs.iter().map(|(u, _)| *u).max().unwrap_or(0);
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
        let mut out = Vec::new();
        for r in refs {
            let MessageRef::Imap { mailbox, uid, .. } = r else {
                continue;
            };
            if let Some((_, raw)) = self
                .messages
                .get(&mailbox.name)
                .and_then(|m| m.iter().find(|(u, _)| u == uid))
            {
                out.push(RawMessage {
                    message_ref: r.clone(),
                    raw: raw.clone(),
                    flags: Vec::new(),
                    internaldate: Some("2026-07-01T09:00:00Z".into()),
                });
            }
        }
        Ok(out)
    }

    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _rm: &[Flag]) -> Result<()> {
        Ok(())
    }

    async fn move_messages(&self, r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        let uids = r
            .iter()
            .map(|_| self.next_moved_uid.fetch_add(1, Ordering::SeqCst))
            .collect();
        Ok(MoveOutcome::Uidplus {
            uidvalidity: UIDVALIDITY,
            uids,
        })
    }

    async fn append(&self, mbox: &RawMailboxRef, _raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
        Ok(MessageRef::Imap {
            mailbox: mbox.clone(),
            uidvalidity: UIDVALIDITY,
            uid: 9002,
        })
    }

    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

struct NoSubmit;

#[async_trait]
impl MailSubmitter for NoSubmit {
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
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(InboxBackend::new()) as Arc<dyn AccountBackend>,
            Arc::new(NoSubmit) as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    Harness { engine, account_id }
}

impl Harness {
    async fn call(&self, method: &str, args: Value) -> Value {
        let req = json!({ "methodCalls": [[method, args, "c0"]] });
        let resp = self.engine.handle_jmap(&self.account_id, &req).await;
        resp["methodResponses"][0][1].clone()
    }

    /// The number of messages in the mailbox with `role`.
    async fn count_in(&self, role: &str) -> usize {
        let mb = self.call("Mailbox/get", json!({})).await;
        let id = mb["list"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["role"] == role)
            .expect("mailbox with role")["id"]
            .clone();
        let q = self
            .call("Email/query", json!({ "filter": { "inMailbox": id } }))
            .await;
        q["ids"].as_array().unwrap().len()
    }
}

/// A rule body as the web client's rules slice sends it.
fn rule(name: &str, conditions: Value) -> Value {
    json!({
        "name": name,
        "matchAll": true,
        "conditions": conditions,
        "actions": [{ "type": "move", "value": "Archive" }],
        "enabled": true,
        "runsAt": "engine"
    })
}

fn thread_only() -> Value {
    json!([{ "type": "thread", "op": "is", "value": "T1" }])
}

fn subject_report() -> Value {
    json!([{ "type": "subject", "op": "contains", "value": "report" }])
}

fn assert_invalid_conditions(set_error: &Value) {
    assert_eq!(set_error["type"], "invalidProperties", "{set_error}");
    assert_eq!(
        set_error["properties"],
        json!(["conditions"]),
        "{set_error}"
    );
}

// ── tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn thread_only_rule_is_refused_and_not_stored() {
    let h = setup().await;
    let set = h
        .call(
            "MailRule/set",
            json!({ "create": { "new": rule("Thread rule", thread_only()) } }),
        )
        .await;

    assert!(set["created"].get("new").is_none(), "{set}");
    assert_invalid_conditions(&set["notCreated"]["new"]);
    let got = h.call("MailRule/get", json!({})).await;
    assert_eq!(got["list"], json!([]), "a refused rule is not stored");
    assert!(h.engine.get_rules(&h.account_id).await.unwrap().is_empty());
}

#[tokio::test]
async fn refused_rule_leaves_inbox_alone_and_a_valid_rule_still_files() {
    let h = setup().await;
    let set = h
        .call(
            "MailRule/set",
            json!({ "create": {
                "thread": rule("Thread rule", thread_only()),
                "reports": rule("Reports", subject_report()),
            } }),
        )
        .await;
    assert_invalid_conditions(&set["notCreated"]["thread"]);
    assert!(set["created"]["reports"]["id"].is_string(), "{set}");

    // The first sync delivers the three inbox messages and runs the rules.
    h.engine.resync(&h.account_id).await.unwrap();

    // Only the Weekly Report moved; the refused rule touched nothing.
    assert_eq!(h.count_in("inbox").await, 2);
    assert_eq!(h.count_in("archive").await, 1);
}

#[tokio::test]
async fn update_that_drops_every_condition_is_refused_and_keeps_the_rule() {
    let h = setup().await;
    let set = h
        .call(
            "MailRule/set",
            json!({ "create": { "new": rule("Reports", subject_report()) } }),
        )
        .await;
    let id = set["created"]["new"]["id"].as_str().unwrap().to_string();

    let upd = h
        .call(
            "MailRule/set",
            json!({ "update": { (id.clone()): rule("Reports", thread_only()) } }),
        )
        .await;
    assert!(upd["updated"].get(&id).is_none(), "{upd}");
    assert_invalid_conditions(&upd["notUpdated"][&id]);

    let got = h.call("MailRule/get", json!({ "ids": [id] })).await;
    assert_eq!(got["list"][0]["conditions"], subject_report());

    // The kept rule still files the report and leaves the rest in the inbox.
    h.engine.resync(&h.account_id).await.unwrap();
    assert_eq!(h.count_in("inbox").await, 2);
    assert_eq!(h.count_in("archive").await, 1);
}
