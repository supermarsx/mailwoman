//! t50hp-e26: `mailboxes.unread` / `total` count the cached rows. Before
//! 0122 a sync wrote the upstream STATUS UNSEEN and then applied the same
//! upstream change again as a per-row delta, and a local ingest added a row
//! without a counter change. Each test compares the counter, and what
//! `Mailbox/get` reports, with a brute-force count of the cached rows.
use super::*;
use crate::account::MailSubmitter;
use crate::backend::*;
use async_trait::async_trait;
use mw_store::{AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store};
use std::sync::{Arc, Mutex};

fn inbox_ref() -> RawMailboxRef {
    RawMailboxRef { name: "INBOX".into(), uidvalidity: 1 }
}
fn imap(uid: u32) -> MessageRef {
    MessageRef::Imap { mailbox: inbox_ref(), uidvalidity: 1, uid }
}

#[derive(Default)]
struct Upstream {
    unseen: Mutex<u32>,
    delta: Mutex<Option<MailboxDelta>>,
}
#[async_trait]
impl AccountBackend for Upstream {
    async fn capabilities(&self) -> Result<BackendCaps> { Ok(BackendCaps::default()) }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(vec![RawMailbox {
            mailbox_ref: inbox_ref(),
            role: MailboxRole::Inbox,
            parent: None,
            uidnext: 10,
            highestmodseq: 1,
            total: 3,
            unread: *self.unseen.lock().unwrap(),
        }])
    }
    async fn sync_mailbox(&self, _: &RawMailboxRef, _: &SyncCursor) -> Result<MailboxDelta> {
        Ok(self.delta.lock().unwrap().take().unwrap_or(MailboxDelta {
            added: vec![],
            flag_changes: vec![],
            removed: vec![],
            next_cursor: SyncCursor::Qresync { uidvalidity: 1, highestmodseq: 1 },
        }))
    }
    async fn fetch_raw(&self, _: &[MessageRef]) -> Result<Vec<RawMessage>> { Err(EngineError::Unsupported("fixture".into())) }
    async fn store_flags(&self, _: &[MessageRef], _: &[Flag], _: &[Flag]) -> Result<()> { Ok(()) }
    async fn move_messages(&self, _: &[MessageRef], _: &RawMailboxRef) -> Result<MoveOutcome> { Err(EngineError::Unsupported("fixture".into())) }
    async fn append(&self, _: &RawMailboxRef, _: &[u8], _: &[Flag]) -> Result<MessageRef> { Err(EngineError::Unsupported("fixture".into())) }
    async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> { Err(EngineError::Unsupported("fixture".into())) }
}
struct NoSend;
#[async_trait]
impl MailSubmitter for NoSend {
    async fn submit(&self, _: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> { panic!("not sending") }
}

/// Three cached unread messages and a counter that agrees (3).
async fn fixture() -> (Engine, Arc<Upstream>, String, String) {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account = store
        .create_account(
            &NewAccount { kind: AccountKind::Imap, host: "fixture.invalid", port: 993, tls: "implicit", username: "o@fixture.invalid", sync_policy_json: "{}" },
            &Credentials { username: "o@fixture.invalid".into(), password: "x".into() },
        )
        .await
        .unwrap();
    let inbox = store
        .upsert_mailbox(&MailboxUpsert { account_id: &account, name: "INBOX", role: Some("inbox"), uidvalidity: 1, uidnext: 10, highestmodseq: 1, total: 3, unread: 3, parent_id: None })
        .await
        .unwrap();
    for uid in 1..=3u32 {
        let mid = format!("<m{uid}@fixture.invalid>");
        let date = format!("2026-11-2{uid}T00:00:00Z");
        store
            .upsert_message(&MessageUpsert { account_id: &account, mailbox_id: &inbox, uid, uidvalidity: 1, message_id: Some(&mid), thread_id: None, internaldate: Some(&date), size: 10, flags_json: "[]", envelope: None, blob_ref: None })
            .await
            .unwrap();
    }
    store.save_cursor(&account, &inbox, &serde_json::to_string(&SyncCursor::Qresync { uidvalidity: 1, highestmodseq: 1 }).unwrap()).await.unwrap();
    let up = Arc::new(Upstream { unseen: Mutex::new(3), delta: Mutex::new(None) });
    let engine = Engine::new(store);
    engine.register_backend(account.clone(), AccountRuntime::new(up.clone(), Arc::new(NoSend), "o@fixture.invalid"));
    (engine, up, account, inbox)
}

/// (stored unread, brute-force unread rows); also asserts `Mailbox/get`
/// reports the brute-force unread and total for every folder.
async fn counter_and_rows(e: &Engine, inbox: &str) -> (u32, usize) {
    let account = e.store().get_mailbox(inbox).await.unwrap().account_id;
    let out = e
        .handle_jmap(&account, &serde_json::json!({
            "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            "methodCalls": [["Mailbox/get", {"accountId": account}, "m"]]
        }))
        .await;
    let list = out["methodResponses"][0][1]["list"].as_array().cloned().unwrap_or_default();
    assert!(!list.is_empty(), "Mailbox/get answered: {out}");
    for mailbox in list {
        let id = mailbox["id"].as_str().unwrap();
        let ids = e.store().list_message_ids(id, i64::MAX, 0).await.unwrap();
        let rows: Vec<_> = e.store().get_messages(&ids).await.unwrap().into_iter().flatten().collect();
        let unread = rows.iter().filter(|m| !crate::mapping::flags_from_json(&m.flags_json).contains(&Flag::Seen)).count();
        assert_eq!(mailbox["unreadEmails"].as_u64(), Some(unread as u64), "Mailbox/get unreadEmails for {id}");
        assert_eq!(mailbox["totalEmails"].as_u64(), Some(rows.len() as u64), "Mailbox/get totalEmails for {id}");
    }
    let counter = e.store().get_mailbox(inbox).await.unwrap().unread;
    let ids = e.store().list_message_ids(inbox, i64::MAX, 0).await.unwrap();
    let rows = e
        .store()
        .get_messages(&ids)
        .await
        .unwrap()
        .into_iter()
        .flatten()
        .filter(|m| !crate::mapping::flags_from_json(&m.flags_json).contains(&Flag::Seen))
        .count();
    (counter, rows)
}

#[tokio::test]
async fn upstream_mark_read_applied_by_sync() {
    let (e, up, account, inbox) = fixture().await;
    assert_eq!(counter_and_rows(&e, &inbox).await, (3, 3));
    *up.unseen.lock().unwrap() = 2;
    *up.delta.lock().unwrap() = Some(MailboxDelta { added: vec![], flag_changes: vec![(imap(1), vec![Flag::Seen])], removed: vec![], next_cursor: SyncCursor::Qresync { uidvalidity: 1, highestmodseq: 2 } });
    e.resync(&account).await.unwrap();
    assert_eq!(counter_and_rows(&e, &inbox).await, (2, 2), "counter vs cached unread rows after an upstream mark-read");
}

#[tokio::test]
async fn upstream_expunge_applied_by_sync() {
    let (e, up, account, inbox) = fixture().await;
    *up.unseen.lock().unwrap() = 2;
    *up.delta.lock().unwrap() = Some(MailboxDelta { added: vec![], flag_changes: vec![], removed: vec![imap(1)], next_cursor: SyncCursor::Qresync { uidvalidity: 1, highestmodseq: 2 } });
    e.resync(&account).await.unwrap();
    assert_eq!(counter_and_rows(&e, &inbox).await, (2, 2), "counter vs cached unread rows after an upstream expunge");
}

#[tokio::test]
async fn upstream_mark_unread_applied_by_sync() {
    let (e, up, account, inbox) = fixture().await;
    e.store().set_flags(&e.store().list_message_ids(&inbox, 1, 0).await.unwrap()[0], "[\"Seen\"]").await.unwrap();
    assert_eq!(counter_and_rows(&e, &inbox).await, (2, 2));
    // uid 3 is the newest (listed first); upstream marks it unread again.
    *up.unseen.lock().unwrap() = 3;
    *up.delta.lock().unwrap() = Some(MailboxDelta { added: vec![], flag_changes: vec![(imap(3), vec![])], removed: vec![], next_cursor: SyncCursor::Qresync { uidvalidity: 1, highestmodseq: 2 } });
    e.resync(&account).await.unwrap();
    assert_eq!(counter_and_rows(&e, &inbox).await, (3, 3), "counter vs cached unread rows after an upstream mark-unread");
}

#[tokio::test]
async fn ingest_without_status() {
    let (e, _up, account, inbox) = fixture().await;
    let raw = RawMessage { message_ref: imap(4), raw: b"From: a@fixture.invalid\r\nSubject: new\r\nMessage-ID: <m4@fixture.invalid>\r\n\r\nbody\r\n".to_vec(), flags: vec![], internaldate: Some("2026-11-24T00:00:00Z".into()) };
    e.ingest(&account, &inbox, &raw).await.unwrap();
    assert_eq!(counter_and_rows(&e, &inbox).await, (4, 4), "counter vs cached unread rows after an ingest with no STATUS");
}
