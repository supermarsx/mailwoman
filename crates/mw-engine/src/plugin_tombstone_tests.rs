//! Replayed bridge tombstones cannot manufacture message history.
use super::*;
use crate::account::MailSubmitter;
use crate::backend::*;
use async_trait::async_trait;
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey};
struct Removed;
#[async_trait]
impl AccountBackend for Removed {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(vec![])
    }
    async fn sync_mailbox(&self, _: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        Ok(MailboxDelta {
            added: vec![],
            flag_changes: vec![],
            removed: {
                let mut refs = ["live", "live", "moved", "foreign", "missing"]
                    .map(|raw| MessageRef::Plugin { raw: raw.into() })
                    .to_vec();
                refs.push(MessageRef::Pop3 {
                    uidl: "missing-pop".into(),
                });
                refs.push(MessageRef::Imap {
                    mailbox: RawMailboxRef {
                        name: "INBOX".into(),
                        uidvalidity: 1,
                    },
                    uidvalidity: 1,
                    uid: 2,
                });
                refs
            },
            next_cursor: c.clone(),
        })
    }
    async fn fetch_raw(&self, _: &[MessageRef]) -> Result<Vec<RawMessage>> {
        panic!("tombstone must not fetch")
    }
    async fn store_flags(&self, _: &[MessageRef], _: &[Flag], _: &[Flag]) -> Result<()> {
        panic!("tombstone must not write upstream")
    }
    async fn move_messages(&self, _: &[MessageRef], _: &RawMailboxRef) -> Result<MoveOutcome> {
        panic!("tombstone must not move upstream")
    }
    async fn append(&self, _: &RawMailboxRef, _: &[u8], _: &[Flag]) -> Result<MessageRef> {
        panic!("tombstone must not append")
    }
    async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported("fixture".into()))
    }
}
struct NoSend;
#[async_trait]
impl MailSubmitter for NoSend {
    async fn submit(&self, _: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> {
        panic!("tombstone must not send")
    }
}
async fn account(store: &Store, name: &str) -> String {
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "fixture.invalid",
                port: 993,
                tls: "implicit",
                username: name,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: name.into(),
                password: "fixture".into(),
            },
        )
        .await
        .unwrap()
}
async fn mailbox(store: &Store, account: &str, name: &str) -> String {
    store
        .upsert_mailbox(&MailboxUpsert {
            account_id: account,
            name,
            role: None,
            uidvalidity: 1,
            uidnext: 2,
            highestmodseq: 0,
            total: 1,
            unread: 0,
            parent_id: None,
        })
        .await
        .unwrap()
}
async fn seed(store: &Store, account: &str, mailbox: &str, uid: u32) -> String {
    store
        .upsert_message(&MessageUpsert {
            account_id: account,
            mailbox_id: mailbox,
            uid,
            uidvalidity: 1,
            message_id: None,
            thread_id: None,
            internaldate: None,
            size: 1,
            flags_json: r#"["Seen"]"#,
            envelope: Some(b"{}"),
            blob_ref: None,
        })
        .await
        .unwrap()
}
#[tokio::test]
async fn duplicate_and_replayed_plugin_removals_skip_absent_foreign_and_moved_sources() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let owner = account(&store, "owner@fixture.invalid").await;
    let other = account(&store, "other@fixture.invalid").await;
    let inbox = mailbox(&store, &owner, "INBOX").await;
    let archive = mailbox(&store, &owner, "Archive").await;
    let foreign_box = mailbox(&store, &other, "INBOX").await;
    let id = seed(&store, &owner, &inbox, 1).await;
    let moved = seed(&store, &owner, &archive, 1).await;
    let foreign = seed(&store, &other, &foreign_box, 1).await;
    let rekeyed = seed(&store, &owner, &inbox, 2).await;
    store
        .relocate_message(&rekeyed, &inbox, 3, 1)
        .await
        .unwrap();
    for (raw, id) in [
        ("live", id.as_str()),
        ("moved", moved.as_str()),
        ("foreign", foreign.as_str()),
        ("missing", "no-longer-cached"),
        ("missing-pop", "no-longer-cached-pop"),
    ] {
        store.record_uidl(&owner, raw, id).await.unwrap();
    }
    let engine = Engine::new(store);
    engine.register_backend(
        owner.clone(),
        AccountRuntime::new(Arc::new(Removed), Arc::new(NoSend), "owner@fixture.invalid"),
    );
    let runtime = engine.runtime(&owner).unwrap();
    let remote = RawMailbox {
        mailbox_ref: RawMailboxRef {
            name: "INBOX".into(),
            uidvalidity: 1,
        },
        role: MailboxRole::Inbox,
        parent: None,
        uidnext: 2,
        highestmodseq: 0,
        total: 1,
        unread: 0,
    };
    for _ in 0..2 {
        engine
            .sync_one(&owner, &runtime, &remote, &inbox, false)
            .await
            .unwrap();
        assert_eq!(
            engine.store().current_state(&owner, "Email").await.unwrap(),
            1
        );
        assert_eq!(
            engine
                .store()
                .current_state(&owner, "Mailbox")
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            engine
                .store()
                .changes_since(&owner, "Email", 0)
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            engine
                .store()
                .stable_id_for_uidl(&owner, "missing-pop")
                .await
                .unwrap()
                .is_none(),
            "missing-row retry must finish POP3 bookkeeping without phantom history"
        );
        assert!(engine.store().get_message(&id).await.is_err());
        assert!(engine.store().get_message(&moved).await.is_ok());
        assert!(
            engine.store().get_message(&rekeyed).await.is_ok(),
            "stale UID tombstone must preserve rekeyed message"
        );
        assert!(engine.store().get_message(&foreign).await.is_ok());
        assert_eq!(
            engine
                .store()
                .stable_id_for_uidl(&owner, "live")
                .await
                .unwrap(),
            Some(id.clone()),
            "durable plugin identity remains available for later ingest"
        );
    }
    engine.store().close().await;
}
