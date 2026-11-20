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
const RAW:&[u8]=b"From: sender@example.invalid\r\nTo: reader@example.invalid\r\nSubject: Recoverable\r\nMIME-Version: 1.0\r\nContent-Type: multipart/alternative; boundary=recover\r\n\r\n--recover\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nThe remote body.\r\n--recover\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<p>The remote body.</p>\r\n--recover--\r\n";
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
    let mut original = serde_json::to_value(mw_mime::parse(RAW).unwrap().email).unwrap();
    original["_mailwomanAttachmentSchema"] = json!(1);
    original["futureMetadata"] = json!({"retain":true});
    let encoded = serde_json::to_vec(&original).unwrap();
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
                envelope: Some(&encoded),
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
    let other = store.create_account(&NewAccount {
        kind: AccountKind::Imap, host: "example.invalid", port: 993,
        tls: "implicit", username: "other@example.invalid", sync_policy_json: "{}",
    }, &Credentials { username: "other@example.invalid".into(), password: "fixture".into() }).await.unwrap();
    engine.register_backend(other.clone(), AccountRuntime::new(
        backend.clone(), Arc::new(NoSend), "other@example.invalid",
    ));
    let foreign = engine.handle_jmap(&other, &json!({"methodCalls":[["Email/get",{
        "accountId":other,"ids":[id,id],"properties":["id","bodyValues","mailwomanMentions"],
        "fetchAllBodyValues":true,"mailwomanBodyUse":true
    },"foreign"]]})).await;
    let denied = &foreign["methodResponses"][0][1];
    assert_eq!(denied["list"], json!([]));
    assert_eq!(denied["notFound"], json!([id,id]));
    assert_eq!(store.cached_body_expiry_stats().await.unwrap().oldest_use_at, Some(1),
        "a foreign explicit reader must not renew the aged body lease");
    assert_eq!(backend.calls.load(Ordering::SeqCst), 0);
    let text_part = original["textBody"][0]["partId"].as_str().unwrap();
    let text_blob = format!("{id}.{text_part}");
    assert_eq!(engine.fetch_blob_with_body_use(&account, &id, false).await.unwrap().unwrap().bytes, RAW);
    assert!(!engine.fetch_blob_with_body_use(&account, &text_blob, false).await.unwrap().unwrap().bytes.is_empty());
    assert!(engine.fetch_blob_with_body_use(&other, &text_blob, false).await.unwrap().is_none());
    assert_eq!(backend.calls.load(Ordering::SeqCst), 0);
    assert_eq!(store.cached_body_expiry_stats().await.unwrap().oldest_use_at, Some(1),
        "background whole-message/part downloads must preserve aged inactivity");
    assert_eq!(engine.fetch_blob(&account, &id).await.unwrap().unwrap().bytes, RAW);
    assert!(store.cached_body_expiry_stats().await.unwrap().oldest_use_at.unwrap() > 1,
        "ordinary downloads retain their default genuine-use renewal");
    sqlx::query("UPDATE message_body_cache_lease SET last_used_at=1")
        .execute(&pool).await.unwrap();
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
    for fetch_values in [None, Some(false)] {
        let mut args = json!({"accountId":account,"ids":[id],"properties":["id","htmlBody","textBody","bodyValues"]});
        if let Some(value) = fetch_values {
            args["fetchHTMLBodyValues"] = json!(value);
            args["fetchTextBodyValues"] = json!(value);
            args["fetchAllBodyValues"] = json!(value);
        }
        let metadata = engine
            .handle_jmap(
                &account,
                &json!({"methodCalls":[["Email/get",args,"metadata"]]}),
            )
            .await;
        assert_eq!(metadata["methodResponses"][0][0], "Email/get");
        assert_eq!(
            store
                .cached_body_expiry_stats()
                .await
                .unwrap()
                .oldest_use_at,
            Some(1),
            "body-part descriptor polling must not renew an aged cache lease"
        );
        assert_eq!(backend.calls.load(Ordering::SeqCst), 0);
    }
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
    let expired: serde_json::Value = serde_json::from_slice(
        &store
            .get_account_envelope(&account, &id)
            .await
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert!(
        expired.get("bodyValues").is_none(),
        "expiry must remove the actual sealed payload"
    );
    for name in ["textBody", "htmlBody", "futureMetadata"] {
        assert_eq!(expired[name], original[name], "expiry must preserve {name}");
    }
    // Speculative/header reads never restore expired content.
    let headers=engine.handle_jmap(&account,&json!({"methodCalls":[["Email/get",{"accountId":account,"ids":[id],"properties":["id","subject"]},"h"]]})).await;
    assert_eq!(headers["methodResponses"][0][0], "Email/get");
    assert_eq!(backend.calls.load(Ordering::SeqCst), 0);
    for fetch_values in [None, Some(false)] {
        let mut args = json!({"accountId":account,"ids":[id],"properties":["id","htmlBody","textBody","bodyValues"]});
        if let Some(value) = fetch_values {
            args["fetchHTMLBodyValues"] = json!(value);
            args["fetchTextBodyValues"] = json!(value);
            args["fetchAllBodyValues"] = json!(value);
        }
        let metadata = engine
            .handle_jmap(
                &account,
                &json!({"methodCalls":[["Email/get",args,"metadata"]]}),
            )
            .await;
        assert_eq!(metadata["methodResponses"][0][0], "Email/get");
        assert!(
            store.get_message(&id).await.unwrap().blob_ref.is_none(),
            "metadata polling must not refill an expired body"
        );
        assert_eq!(
            store
                .cached_body_expiry_stats()
                .await
                .unwrap()
                .oldest_use_at,
            Some(1)
        );
        assert_eq!(backend.calls.load(Ordering::SeqCst), 0);
    }
    let prefetch=engine.handle_jmap(&account,&json!({"methodCalls":[["Email/get",{"accountId":account,"ids":[id],"properties":["id","bodyValues"],"fetchHTMLBodyValues":true,"mailwomanBodyUse":false},"p"]]})).await;
    assert_eq!(prefetch["methodResponses"][0][0], "Email/get");
    assert_eq!(backend.calls.load(Ordering::SeqCst), 0);
    let still_expired: serde_json::Value = serde_json::from_slice(
        &store
            .get_account_envelope(&account, &id)
            .await
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert!(
        still_expired.get("bodyValues").is_none(),
        "prefetch must not hydrate the sealed payload"
    );
    *backend.flag_on_fetch.lock().unwrap() = Some(id.clone());
    let standard=engine.handle_jmap(&account,&json!({"methodCalls":[["Email/get",{"accountId":account,"ids":[id],"properties":["id","bodyValues"],"fetchHTMLBodyValues":true},"s"]]})).await;
    assert_eq!(standard["methodResponses"][0][0], "Email/get");
    assert_eq!(
        standard["methodResponses"][0][1]["list"][0]["bodyValues"], original["bodyValues"],
        "the actual standard reader response must contain restored text and HTML"
    );
    sqlx::query("UPDATE message_body_cache_lease SET last_used_at=1")
        .execute(&pool).await.unwrap();
    let only_id = engine.handle_jmap(&account, &json!({"methodCalls":[["Email/get",{
        "accountId":account,"ids":[id],"properties":["id"],"mailwomanBodyUse":true
    },"body-use"]]})).await;
    assert_eq!(only_id["methodResponses"][0][1]["list"], json!([{"id":id}]));
    assert!(store.cached_body_expiry_stats().await.unwrap().oldest_use_at.unwrap() > 1,
        "id-only reader intent must still renew the body lease");
    let restored: serde_json::Value = serde_json::from_slice(
        &store
            .get_account_envelope(&account, &id)
            .await
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    for name in ["textBody", "htmlBody", "futureMetadata", "bodyValues"] {
        assert_eq!(
            restored[name], original[name],
            "reader hydration must restore/preserve {name}"
        );
    }
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
    assert!(!engine.fetch_blob_with_body_use(&account, &text_blob, false).await.unwrap().unwrap().bytes.is_empty());
    assert_eq!(backend.calls.load(Ordering::SeqCst), 2);
    assert_eq!(store.cached_body_expiry_stats().await.unwrap().oldest_use_at, Some(1),
        "background download refill must not reset the existing source lease");
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
