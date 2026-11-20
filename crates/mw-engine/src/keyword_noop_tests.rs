//! Real Email/set replay behavior: no phantom history, writes or upstream deltas.
// Reuses this real backend fixture; no production query/publication switch.
include!("mixed_query_contract_tests.rs");
use super::*;
use crate::account::MailSubmitter;
use crate::backend::*;
use async_trait::async_trait;
use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Default)]
struct Remote {
    flags: AtomicUsize,
    moves: AtomicUsize,
}
#[async_trait]
impl AccountBackend for Remote {
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
            removed: vec![],
            next_cursor: c.clone(),
        })
    }
    async fn fetch_raw(&self, _: &[MessageRef]) -> Result<Vec<RawMessage>> {
        panic!("keyword patches must not fetch bodies")
    }
    async fn store_flags(&self, _: &[MessageRef], _: &[Flag], _: &[Flag]) -> Result<()> {
        self.flags.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    async fn move_messages(&self, _: &[MessageRef], _: &RawMailboxRef) -> Result<MoveOutcome> {
        self.moves.fetch_add(1, Ordering::SeqCst);
        Ok(MoveOutcome::Uidplus {
            uidvalidity: 2,
            uids: vec![7],
        })
    }
    async fn append(&self, _: &RawMailboxRef, _: &[u8], _: &[Flag]) -> Result<MessageRef> {
        panic!("not composing")
    }
    async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported("fixture".into()))
    }
}
struct NoSend;
#[async_trait]
impl MailSubmitter for NoSend {
    async fn submit(&self, _: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> {
        panic!("not sending")
    }
}
struct Fixture {
    engine: Engine,
    remote: Arc<Remote>,
    runtime: AccountRuntime,
    account: String,
    inbox: String,
    archive: String,
    id: String,
}
impl Fixture {
    async fn new(flags: &str) -> Self {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let account = store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "fixture.invalid",
                    port: 993,
                    tls: "implicit",
                    username: "owner@fixture.invalid",
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: "owner@fixture.invalid".into(),
                    password: "fixture".into(),
                },
            )
            .await
            .unwrap();
        let mut boxes = vec![];
        for (name, role) in [("INBOX", "inbox"), ("Archive", "archive")] {
            boxes.push(
                store
                    .upsert_mailbox(&MailboxUpsert {
                        account_id: &account,
                        name,
                        role: Some(role),
                        uidvalidity: 1,
                        uidnext: 2,
                        highestmodseq: 0,
                        total: 0,
                        unread: 0,
                        parent_id: None,
                    })
                    .await
                    .unwrap(),
            );
        }
        let id = store
            .upsert_message(&MessageUpsert {
                account_id: &account,
                mailbox_id: &boxes[0],
                uid: 1,
                uidvalidity: 1,
                message_id: Some("<keyword@fixture.invalid>"),
                thread_id: None,
                internaldate: None,
                size: 10,
                flags_json: flags,
                envelope: Some(b"{}"),
                blob_ref: None,
            })
            .await
            .unwrap();
        let engine = Engine::new(store);
        engine
            .search()
            .upsert(&mw_search::IndexDoc {
                stable_id: id.clone(),
                account_id: account.clone(),
                mailbox_id: boxes[0].clone(),
                keywords: vec!["$seen".into(), "label".into()],
                ..Default::default()
            })
            .unwrap();
        let remote = Arc::new(Remote::default());
        engine.register_backend(
            account.clone(),
            AccountRuntime::new(remote.clone(), Arc::new(NoSend), "owner@fixture.invalid"),
        );
        let runtime = engine.runtime(&account).unwrap();
        Self {
            engine,
            remote,
            runtime,
            account,
            inbox: boxes.remove(0),
            archive: boxes.remove(0),
            id,
        }
    }
    async fn patch(&self, patch: Value) -> Value {
        let response = self
            .engine
            .email_set(
                &self.account,
                &self.runtime,
                &json!({"update":{self.id.clone():patch}}),
                &mut HashMap::new(),
            )
            .await;
        assert_eq!(response["updated"][&self.id], Value::Null, "{response}");
        assert!(
            response["updated"]
                .as_object()
                .unwrap()
                .contains_key(&self.id),
            "{response}"
        );
        assert!(response.get("notUpdated").is_none(), "{response}");
        response
    }
    async fn snapshot(&self) -> (String, u64, u64, usize) {
        (
            self.engine
                .store()
                .get_message(&self.id)
                .await
                .unwrap()
                .cache_revision,
            self.engine
                .store()
                .current_state(&self.account, "Email")
                .await
                .unwrap(),
            mw_search::commit_count(),
            self.remote.flags.load(Ordering::SeqCst),
        )
    }
}
#[tokio::test]
async fn replayed_keywords_do_not_write_history_or_index_but_real_updates_still_apply() {
    let f = Fixture::new(r#"["Recent",{"Keyword":"label"},"Seen","Seen"]"#).await;
    let before = f.snapshot().await;
    for patch in [
        json!({"keywords/$seen":true}),
        json!({"keywords/$seen":true}),
        json!({"keywords":{"label":true,"$seen":true}}),
        json!({"keywords/missing":null}),
    ] {
        let response = f.patch(patch).await;
        assert_eq!(response["oldState"], response["newState"]);
        assert_eq!(
            f.snapshot().await,
            before,
            "semantic no-op must skip SQL/index/history/upstream writes"
        );
    }
    let unread = f.engine.store().get_mailbox(&f.inbox).await.unwrap().unread;
    let changed = f.patch(json!({"keywords/$seen":null})).await;
    assert_ne!(changed["oldState"], changed["newState"]);
    let after = f.snapshot().await;
    assert_ne!(after.0, before.0);
    assert_eq!(after.1, before.1 + 1);
    assert_eq!(after.3, 1);
    assert_eq!(
        f.engine.store().get_mailbox(&f.inbox).await.unwrap().unread,
        unread + 1
    );
    assert!(
        flags_from_json(
            &f.engine
                .store()
                .get_message(&f.id)
                .await
                .unwrap()
                .flags_json
        )
        .contains(&Flag::Recent)
    );
    let pinned = f.patch(json!({"keywords/$seen":null,"pinned":true})).await;
    assert_ne!(pinned["oldState"], pinned["newState"]);
    assert!(
        f.engine
            .store()
            .get_message_meta(&f.id)
            .await
            .unwrap()
            .unwrap()
            .pinned
    );
    assert_eq!(f.remote.flags.load(Ordering::SeqCst), 1);
    let moved = f
        .patch(json!({"keywords/$seen":null,"mailboxIds":{f.archive.clone():true}}))
        .await;
    assert_ne!(moved["oldState"], moved["newState"]);
    assert_eq!(
        f.engine
            .store()
            .get_message(&f.id)
            .await
            .unwrap()
            .mailbox_id,
        f.archive
    );
    assert_eq!(f.remote.flags.load(Ordering::SeqCst), 1);
    assert_eq!(f.remote.moves.load(Ordering::SeqCst), 1);
    f.engine.store().close().await;
}
#[tokio::test]
async fn malformed_cached_flags_are_repaired_even_when_requested_keywords_are_empty() {
    let f = Fixture::new("not-json").await;
    let before = f.snapshot().await;
    let response = f.patch(json!({"keywords":{}})).await;
    assert_ne!(response["oldState"], response["newState"]);
    let after = f.snapshot().await;
    assert_ne!(after.0, before.0);
    assert_eq!(after.1, before.1 + 1);
    assert_eq!(after.3, 1);
    assert_eq!(
        f.engine
            .store()
            .get_message(&f.id)
            .await
            .unwrap()
            .flags_json,
        "[]"
    );
    f.engine.store().close().await;
}

#[tokio::test]
async fn unchanged_metadata_preserves_state_and_index_while_combined_changes_still_apply() {
    let f = Fixture::new(r#"["Seen",{"Keyword":"label"}]"#).await;
    let original = f.snapshot().await;
    assert!(
        f.engine
            .store()
            .get_message_meta(&f.id)
            .await
            .unwrap()
            .is_none()
    );
    for patch in [
        json!({"pinned":false,"snoozedUntil":null,"followUpAt":null}),
        json!({"pinned":null,"snoozedUntil":42,"followUpAt":false}),
    ] {
        let response = f.patch(patch).await;
        assert_eq!(response["oldState"], response["newState"]);
        assert_eq!(f.snapshot().await, original);
        assert!(
            f.engine
                .store()
                .get_message_meta(&f.id)
                .await
                .unwrap()
                .is_none(),
            "default no-op must not materialize metadata row"
        );
    }
    let snooze = "2026-12-01T10:00:00Z";
    let follow = "2026-12-02T10:00:00Z";
    let response = f
        .patch(json!({"pinned":true,"snoozedUntil":snooze,"followUpAt":follow}))
        .await;
    assert_ne!(response["oldState"], response["newState"]);
    let configured = f.snapshot().await;
    assert_eq!(
        configured.0, original.0,
        "metadata retains the existing message-row revision semantics"
    );
    assert_eq!(configured.1, original.1 + 1);
    assert_eq!(configured.2, original.2 + 1);
    assert_eq!(configured.3, 0);
    for patch in [
        json!({"pinned":true,"snoozedUntil":snooze,"followUpAt":follow}),
        json!({"pinned":true}),
        json!({"snoozedUntil":snooze}),
        json!({"followUpAt":follow}),
        json!({"pinned":"keep existing value"}),
    ] {
        let response = f.patch(patch).await;
        assert_eq!(response["oldState"], response["newState"]);
        assert_eq!(f.snapshot().await, configured);
    }
    // Preserve prior coercion: invalid pin values keep the pin, while non-string
    // snooze/follow-up values clear them rather than introducing validation here.
    let response = f
        .patch(json!({"pinned":"keep","snoozedUntil":42,"followUpAt":false}))
        .await;
    assert_ne!(response["oldState"], response["newState"]);
    assert_eq!(
        f.engine.store().get_message_meta(&f.id).await.unwrap(),
        Some(StoredMeta {
            pinned: true,
            snoozed_until: None,
            follow_up_at: None
        })
    );
    let cleared = f.snapshot().await;
    let response = f.patch(json!({"snoozedUntil":42,"followUpAt":false})).await;
    assert_eq!(response["oldState"], response["newState"]);
    assert_eq!(f.snapshot().await, cleared);
    let response = f
        .patch(json!({"pinned":true,"snoozedUntil":null,"followUpAt":follow}))
        .await;
    assert_ne!(response["oldState"], response["newState"]);
    assert_eq!(
        f.engine
            .store()
            .get_message_meta(&f.id)
            .await
            .unwrap()
            .unwrap()
            .follow_up_at
            .as_deref(),
        Some(follow)
    );
    // No-op metadata must not suppress a real keyword delta.
    let before = f.snapshot().await;
    let response = f
        .patch(json!({"pinned":true,"followUpAt":follow,"keywords/$seen":null}))
        .await;
    assert_ne!(response["oldState"], response["newState"]);
    let after = f.snapshot().await;
    assert_eq!(after.1, before.1 + 1);
    assert_eq!(after.3, 1);
    assert_ne!(after.0, before.0);
    // A real pin change must still apply alongside unchanged keywords/time fields.
    let response = f
        .patch(json!({"pinned":false,"followUpAt":follow,"keywords/$seen":null}))
        .await;
    assert_ne!(response["oldState"], response["newState"]);
    assert!(
        !f.engine
            .store()
            .get_message_meta(&f.id)
            .await
            .unwrap()
            .unwrap()
            .pinned
    );
    assert_eq!(f.remote.flags.load(Ordering::SeqCst), 1);
    let before = f.snapshot().await;
    let response=f.patch(json!({"pinned":false,"snoozedUntil":null,"followUpAt":follow,"mailboxIds":{f.archive.clone():true}})).await;
    assert_ne!(response["oldState"], response["newState"]);
    assert_eq!(
        f.engine
            .store()
            .get_message(&f.id)
            .await
            .unwrap()
            .mailbox_id,
        f.archive
    );
    assert_eq!(
        f.engine
            .store()
            .current_state(&f.account, "Email")
            .await
            .unwrap(),
        before.1 + 1
    );
    assert_eq!(f.remote.flags.load(Ordering::SeqCst), 1);
    assert_eq!(f.remote.moves.load(Ordering::SeqCst), 1);
    f.engine.store().close().await;
}
