//! Actual engine body expiry/refetch and explicit reader-intent renewal.
use async_trait::async_trait;
use mw_engine::{Engine, MailSubmitter, account::AccountRuntime, backend::*};
use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
    body_cache_expiry::{CachedBodyExpiryCursor, CachedBodyExpiryOptions},
};
use serde_json::json;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
struct Remote {
    calls: AtomicUsize,
    store: Store,
    flag_on_fetch: std::sync::Mutex<Option<String>>,
}
const RAW:&[u8]=b"From: sender@example.invalid\r\nTo: reader@example.invalid\r\nSubject: Recoverable\r\n\r\nThe remote body.\r\n";
#[async_trait]
impl AccountBackend for Remote {
    fn supports_cached_body_refetch(&self) -> bool {
        true
    }
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(Vec::new())
    }
    async fn sync_mailbox(&self, _: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        Ok(MailboxDelta {
            added: vec![],
            flag_changes: vec![],
            removed: vec![],
            next_cursor: c.clone(),
        })
    }
    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let changed = { self.flag_on_fetch.lock().unwrap().take() };
        if let Some(id) = changed {
            self.store.set_flags(&id, "[\"\\\\Seen\"]").await?;
        }
        Ok(refs
            .iter()
            .map(|reference| RawMessage {
                message_ref: reference.clone(),
                raw: RAW.to_vec(),
                flags: vec![],
                internaldate: None,
            })
            .collect())
    }
    async fn store_flags(&self, _: &[MessageRef], _: &[Flag], _: &[Flag]) -> Result<()> {
        Err(EngineError::Unsupported("fixture".into()))
    }
    async fn move_messages(&self, _: &[MessageRef], _: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("fixture".into()))
    }
    async fn append(&self, _: &RawMailboxRef, _: &[u8], _: &[Flag]) -> Result<MessageRef> {
        Err(EngineError::Unsupported("fixture".into()))
    }
    async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported("fixture".into()))
    }
}
struct NoSend;
#[async_trait]
impl MailSubmitter for NoSend {
    async fn submit(&self, _: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> {
        Err(EngineError::Unsupported("fixture".into()))
    }
}

#[tokio::test]
async fn explicit_reader_restores_expired_body_and_background_download_does_not_renew() {
    let directory = std::env::temp_dir().join(format!(
        "mw-body-expiry-engine-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("mail.db");
    let store = Store::open_with_encryption(
        path.to_str().unwrap(),
        ServerKey::generate(),
        &mw_store::encryption::EncryptionConfig::off(),
    )
    .await
    .unwrap();
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "example.invalid",
                port: 993,
                tls: "implicit",
                username: "reader@example.invalid",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "reader@example.invalid".into(),
                password: "fixture".into(),
            },
        )
        .await
        .unwrap();
    let mailbox = store
        .upsert_mailbox(&MailboxUpsert {
            account_id: &account,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 1,
            uidnext: 2,
            highestmodseq: 0,
            total: 1,
            unread: 1,
            parent_id: None,
        })
        .await
        .unwrap();
    let id = store
        .upsert_message_with_body(
            &MessageUpsert {
                account_id: &account,
                mailbox_id: &mailbox,
                uid: 1,
                uidvalidity: 1,
                message_id: None,
                thread_id: None,
                internaldate: None,
                size: RAW.len() as u64,
                flags_json: "[]",
                envelope: Some(b"{\"subject\":\"Recoverable\"}"),
                blob_ref: None,
            },
            RAW,
        )
        .await
        .unwrap()
        .0;
    let engine = Engine::new(store.clone());
    let backend = Arc::new(Remote {
        calls: AtomicUsize::new(0),
        store: store.clone(),
        flag_on_fetch: std::sync::Mutex::new(None),
    });
    engine.register_backend(
        account.clone(),
        AccountRuntime::new(backend.clone(), Arc::new(NoSend), "reader@example.invalid"),
    );
    assert_eq!(engine.body_refetchable_accounts(), vec![account.clone()]);
    store
        .register_refetchable_cached_body(&account, &id)
        .await
        .unwrap();
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&path))
        .await
        .unwrap();
    sqlx::query("UPDATE message_body_cache_lease SET last_used_at=1")
        .execute(&pool)
        .await
        .unwrap();
    let prefetch=engine.handle_jmap(&account,&json!({"methodCalls":[["Email/get",{"accountId":account,"ids":[id],"properties":["id","bodyValues"],"fetchHTMLBodyValues":true,"mailwomanBodyUse":false},"p"]]})).await;
    assert_eq!(prefetch["methodResponses"][0][0], "Email/get");
    assert_eq!(
        store
            .cached_body_expiry_stats()
            .await
            .unwrap()
            .oldest_use_at,
        Some(1),
        "background body prefetch must not renew"
    );
    assert_eq!(backend.calls.load(Ordering::SeqCst), 0);
    let report = store
        .expire_cached_bodies(
            CachedBodyExpiryOptions {
                max_run_ms: 2000,
                ..Default::default()
            },
            CachedBodyExpiryCursor::default(),
            engine.body_refetchable_accounts(),
        )
        .await
        .unwrap();
    assert_eq!(report.detached_messages, 1, "{report:?}");
    assert!(store.get_message(&id).await.unwrap().blob_ref.is_none());
    // Speculative/header reads never restore expired content.
    let headers=engine.handle_jmap(&account,&json!({"methodCalls":[["Email/get",{"accountId":account,"ids":[id],"properties":["id","subject"]},"h"]]})).await;
    assert_eq!(headers["methodResponses"][0][0], "Email/get");
    assert_eq!(backend.calls.load(Ordering::SeqCst), 0);
    let prefetch=engine.handle_jmap(&account,&json!({"methodCalls":[["Email/get",{"accountId":account,"ids":[id],"properties":["id","bodyValues"],"fetchHTMLBodyValues":true,"mailwomanBodyUse":false},"p"]]})).await;
    assert_eq!(prefetch["methodResponses"][0][0], "Email/get");
    assert_eq!(backend.calls.load(Ordering::SeqCst), 0);
    *backend.flag_on_fetch.lock().unwrap() = Some(id.clone());
    let standard=engine.handle_jmap(&account,&json!({"methodCalls":[["Email/get",{"accountId":account,"ids":[id],"properties":["id","bodyValues"],"fetchHTMLBodyValues":true},"s"]]})).await;
    assert_eq!(standard["methodResponses"][0][0], "Email/get");
    assert_eq!(
        backend.calls.load(Ordering::SeqCst),
        1,
        "standard body reads must refetch"
    );
    assert!(
        store
            .get_message(&id)
            .await
            .unwrap()
            .flags_json
            .contains("Seen"),
        "a concurrent flag update must survive the bounded body-refill retry"
    );
    sqlx::query("UPDATE message_body_cache_lease SET last_used_at=1")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        store
            .expire_cached_bodies(
                CachedBodyExpiryOptions {
                    max_run_ms: 2000,
                    ..Default::default()
                },
                Default::default(),
                engine.body_refetchable_accounts()
            )
            .await
            .unwrap()
            .detached_messages,
        1
    );
    let opened=engine.handle_jmap(&account,&json!({"methodCalls":[["Email/get",{"accountId":account,"ids":[id],"properties":["id","subject"],"mailwomanBodyUse":true},"r"]]})).await;
    assert_eq!(opened["methodResponses"][0][0], "Email/get");
    assert_eq!(backend.calls.load(Ordering::SeqCst), 2);
    assert!(store.get_message(&id).await.unwrap().blob_ref.is_some());
    assert!(
        store
            .cached_body_expiry_stats()
            .await
            .unwrap()
            .oldest_use_at
            .unwrap()
            > 1
    );
    assert_eq!(
        engine
            .fetch_blob(&account, &id)
            .await
            .unwrap()
            .unwrap()
            .bytes,
        RAW
    );
    assert_eq!(backend.calls.load(Ordering::SeqCst), 2);
    pool.close().await;
    drop(engine);
    store.close().await;
    drop(store);
    std::fs::remove_dir_all(directory).unwrap();
}
