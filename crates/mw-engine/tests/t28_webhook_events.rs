//! 26.20 t28-e18 (f34) — the engine says what kind of change it made, for outbound
//! webhooks.
//!
//! Before 26.20 a webhook could only be told "a state token moved"
//! (`StateChange`): one subscribed to new mail was called for every flag change,
//! move and delete as well. The engine now emits a typed [`EngineEvent`] at the
//! mutation that causes it. These tests attach a recording sink and drive the
//! engine through `resync` and `handle_jmap`, and check each kind is emitted by
//! the change it names and not by the others.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, Flag, MailboxDelta, MailboxRole, MessageRef,
    MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor, WatchHandle,
};
use mw_engine::change::{EngineEvent, EventKind, EventSink};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

const UIDVALIDITY: u32 = 100;

type ScriptMsg = (u32, Vec<u8>, Vec<Flag>);

/// An IMAP account whose INBOX the test can add to and expunge from between
/// resyncs.
struct ScriptedBackend {
    messages: Mutex<HashMap<String, Vec<ScriptMsg>>>,
    removed: Mutex<Vec<u32>>,
}

fn msg(i: u32) -> Vec<u8> {
    format!(
        "Message-ID: <ev-{i}@x>\r\nFrom: alice@example.org\r\nTo: me@example.org\r\n\
         Subject: Event {i}\r\nDate: 01 Jul 2026 09:00:00 +0000\r\n\r\nbody {i}\r\n"
    )
    .into_bytes()
}

impl ScriptedBackend {
    fn with_inbox(uids: &[u32]) -> Self {
        let mut messages = HashMap::new();
        messages.insert(
            "INBOX".to_string(),
            uids.iter().map(|u| (*u, msg(*u), Vec::new())).collect(),
        );
        messages.insert("Archive".to_string(), Vec::new());
        Self {
            messages: Mutex::new(messages),
            removed: Mutex::new(Vec::new()),
        }
    }

    fn deliver(&self, uid: u32) {
        self.messages
            .lock()
            .unwrap()
            .get_mut("INBOX")
            .unwrap()
            .push((uid, msg(uid), Vec::new()));
    }

    fn expunge(&self, uid: u32) {
        self.messages
            .lock()
            .unwrap()
            .get_mut("INBOX")
            .unwrap()
            .retain(|(u, ..)| *u != uid);
        self.removed.lock().unwrap().push(uid);
    }
}

#[async_trait]
impl AccountBackend for ScriptedBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: true,
            r#move: true,
            special_use: true,
            ..BackendCaps::default()
        })
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        let st = self.messages.lock().unwrap();
        Ok([
            ("INBOX", MailboxRole::Inbox),
            ("Archive", MailboxRole::Archive),
        ]
        .into_iter()
        .map(|(name, role)| {
            let total = st.get(name).map(|m| m.len()).unwrap_or(0) as u32;
            let uidnext = st
                .get(name)
                .and_then(|m| m.iter().map(|(u, ..)| *u).max())
                .unwrap_or(0)
                + 1;
            RawMailbox {
                mailbox_ref: RawMailboxRef {
                    name: name.to_string(),
                    uidvalidity: UIDVALIDITY,
                },
                role,
                parent: None,
                uidnext,
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
        let st = self.messages.lock().unwrap();
        let msgs = st.get(&mbox.name).cloned().unwrap_or_default();
        let r = |uid: u32| MessageRef::Imap {
            mailbox: mbox.clone(),
            uidvalidity: UIDVALIDITY,
            uid,
        };
        let added = msgs
            .iter()
            .filter(|(uid, ..)| *uid >= uidnext_from)
            .map(|(uid, ..)| r(*uid))
            .collect();
        let removed = if mbox.name == "INBOX" {
            std::mem::take(&mut *self.removed.lock().unwrap())
                .into_iter()
                .map(r)
                .collect()
        } else {
            Vec::new()
        };
        let max_uid = msgs
            .iter()
            .map(|(u, ..)| *u)
            .max()
            .unwrap_or(uidnext_from.saturating_sub(1));
        Ok(MailboxDelta {
            added,
            flag_changes: Vec::new(),
            removed,
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: UIDVALIDITY,
                uidnext: max_uid.max(uidnext_from.saturating_sub(1)) + 1,
            },
        })
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        let st = self.messages.lock().unwrap();
        let mut out = Vec::new();
        for r in refs {
            let MessageRef::Imap { mailbox, uid, .. } = r else {
                continue;
            };
            if let Some(msgs) = st.get(&mailbox.name)
                && let Some((_, raw, flags)) = msgs.iter().find(|(u, ..)| u == uid)
            {
                out.push(RawMessage {
                    message_ref: r.clone(),
                    raw: raw.clone(),
                    flags: flags.clone(),
                    internaldate: Some("2026-07-01T09:00:00Z".into()),
                });
            }
        }
        Ok(out)
    }

    async fn store_flags(&self, _r: &[MessageRef], _add: &[Flag], _rm: &[Flag]) -> Result<()> {
        Ok(())
    }

    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Ok(MoveOutcome::Uidplus {
            uidvalidity: UIDVALIDITY,
            uids: vec![9001],
        })
    }

    async fn append(&self, mbox: &RawMailboxRef, raw: &[u8], flags: &[Flag]) -> Result<MessageRef> {
        let mut st = self.messages.lock().unwrap();
        let entry = st.entry(mbox.name.clone()).or_default();
        let uid = entry.iter().map(|(u, ..)| *u).max().unwrap_or(0) + 1;
        entry.push((uid, raw.to_vec(), flags.to_vec()));
        Ok(MessageRef::Imap {
            mailbox: mbox.clone(),
            uidvalidity: UIDVALIDITY,
            uid,
        })
    }

    async fn expunge_messages(&self, refs: &[MessageRef]) -> Result<()> {
        let mut st = self.messages.lock().unwrap();
        for r in refs {
            if let MessageRef::Imap { mailbox, uid, .. } = r
                && let Some(msgs) = st.get_mut(&mailbox.name)
            {
                msgs.retain(|(u, ..)| u != uid);
            }
        }
        Ok(())
    }

    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

struct NullSubmitter;

#[async_trait]
impl MailSubmitter for NullSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

#[derive(Default)]
struct Recorder(Mutex<Vec<EngineEvent>>);

impl EventSink for Recorder {
    fn emit(&self, event: EngineEvent) {
        self.0.lock().unwrap().push(event);
    }
}

impl Recorder {
    fn take(&self) -> Vec<(EventKind, Vec<String>)> {
        std::mem::take(&mut *self.0.lock().unwrap())
            .into_iter()
            .map(|e| (e.kind, e.ids))
            .collect()
    }
}

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
    backend: Arc<ScriptedBackend>,
    events: Arc<Recorder>,
}

async fn setup(seed: &[u32]) -> Harness {
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
    let backend = Arc::new(ScriptedBackend::with_inbox(seed));
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            backend.clone() as Arc<dyn AccountBackend>,
            Arc::new(NullSubmitter) as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    let events = Arc::new(Recorder::default());
    engine.attach_event_sink(events.clone());
    Harness {
        engine,
        account_id,
        backend,
        events,
    }
}

async fn jmap(h: &Harness, calls: Value) -> Value {
    h.engine
        .handle_jmap(&h.account_id, &json!({ "methodCalls": calls }))
        .await
}

async fn mailbox_id(h: &Harness, role: &str) -> String {
    let r = jmap(h, json!([["Mailbox/get", {}, "m"]])).await;
    r["methodResponses"][0][1]["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == role)
        .unwrap_or_else(|| panic!("no {role} mailbox: {r}"))["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn inbox_ids(h: &Harness) -> Vec<String> {
    let inbox = mailbox_id(h, "inbox").await;
    let r = jmap(
        h,
        json!([["Email/query", { "filter": { "inMailbox": inbox } }, "q"]]),
    )
    .await;
    r["methodResponses"][0][1]["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

fn kinds(events: &[(EventKind, Vec<String>)]) -> Vec<EventKind> {
    events.iter().map(|(k, _)| *k).collect()
}

#[tokio::test]
async fn new_mail_is_message_received_and_nothing_else() {
    let h = setup(&[1, 2]).await;
    h.engine.resync(&h.account_id).await.unwrap();
    let first = h.events.take();
    assert_eq!(
        kinds(&first),
        [EventKind::MessageReceived, EventKind::MessageReceived],
        "the first sync receives both messages: {first:?}"
    );

    // A resync with nothing new emits nothing.
    h.engine.resync(&h.account_id).await.unwrap();
    assert!(h.events.take().is_empty(), "no change, no event");

    // One delivery, one event, naming the new message.
    h.backend.deliver(3);
    h.engine.resync(&h.account_id).await.unwrap();
    let got = h.events.take();
    assert_eq!(kinds(&got), [EventKind::MessageReceived], "{got:?}");
    let ids = inbox_ids(&h).await;
    assert_eq!(ids.len(), 3);
    assert!(
        ids.contains(&got[0].1[0]),
        "the event names the new message"
    );
}

#[tokio::test]
async fn a_flag_change_is_message_flags_not_received() {
    let h = setup(&[1]).await;
    h.engine.resync(&h.account_id).await.unwrap();
    h.events.take();
    let id = inbox_ids(&h).await.remove(0);

    let r = jmap(
        &h,
        json!([["Email/set", { "update": { &id: { "keywords/$seen": true } } }, "s"]]),
    )
    .await;
    assert!(
        r["methodResponses"][0][1]["updated"]
            .as_object()
            .is_some_and(|u| u.contains_key(&id)),
        "setup: the flag was set: {r}"
    );
    assert_eq!(h.events.take(), [(EventKind::MessageFlags, vec![id])]);
}

#[tokio::test]
async fn a_move_is_message_moved_and_a_destroy_is_message_deleted() {
    let h = setup(&[1, 2]).await;
    h.engine.resync(&h.account_id).await.unwrap();
    h.events.take();
    let ids = inbox_ids(&h).await;
    let archive = mailbox_id(&h, "archive").await;

    let r = jmap(
        &h,
        json!([["Email/set", { "update": { &ids[0]: { "mailboxIds": { &archive: true } } } }, "s"]]),
    )
    .await;
    assert!(
        r["methodResponses"][0][1]["updated"]
            .as_object()
            .is_some_and(|u| u.contains_key(&ids[0])),
        "setup: the move was applied: {r}"
    );
    let moved = h.events.take();
    assert_eq!(kinds(&moved), [EventKind::MessageMoved], "{moved:?}");
    assert_eq!(moved[0].1, [ids[0].clone()]);

    let r = jmap(&h, json!([["Email/set", { "destroy": [&ids[1]] }, "d"]])).await;
    assert!(
        r["methodResponses"][0][1]["destroyed"]
            .as_array()
            .is_some_and(|d| d.iter().any(|v| v == &json!(ids[1]))),
        "setup: the destroy was applied: {r}"
    );
    let deleted = h.events.take();
    assert!(
        kinds(&deleted).contains(&EventKind::MessageDeleted)
            && !kinds(&deleted).contains(&EventKind::MessageReceived),
        "{deleted:?}"
    );
}

#[tokio::test]
async fn an_expunge_on_the_server_is_message_deleted() {
    let h = setup(&[1, 2]).await;
    h.engine.resync(&h.account_id).await.unwrap();
    h.events.take();
    h.backend.expunge(2);
    h.engine.resync(&h.account_id).await.unwrap();
    let got = h.events.take();
    assert_eq!(kinds(&got), [EventKind::MessageDeleted], "{got:?}");
}

#[tokio::test]
async fn attach_v6_without_a_sink_keeps_the_sink_attached_before() {
    let h = setup(&[1]).await;
    h.engine.attach_v6(mw_engine::V6Hooks::new());
    h.engine.resync(&h.account_id).await.unwrap();
    assert_eq!(
        kinds(&h.events.take()),
        [EventKind::MessageReceived],
        "the webhook sink survives the later cache/posture attach"
    );
}
