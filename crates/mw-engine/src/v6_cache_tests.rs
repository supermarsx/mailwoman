use super::*;
use mw_cache::{CacheLayer, ClassPolicy, ScopeMatrix};
use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
};

fn fixture_id() -> String {
    format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

fn cache() -> Cache {
    let mut matrix = ScopeMatrix::spec_defaults();
    matrix.apply_override(ClassPolicy {
        class: CacheClass::MessageBodies,
        layers: vec![CacheLayer::Memory],
        ttl_secs: 300,
    });
    Cache::in_memory(matrix)
}
async fn account(store: &Store) -> (String, String) {
    let id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "example.org",
                port: 993,
                tls: "implicit",
                username: "cache@example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "cache@example.org".into(),
                password: "fixture".into(),
            },
        )
        .await
        .unwrap();
    let mailbox = store
        .upsert_mailbox(&MailboxUpsert {
            account_id: &id,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 1,
            uidnext: 2,
            highestmodseq: 1,
            total: 1,
            unread: 1,
            parent_id: None,
        })
        .await
        .unwrap();
    (id, mailbox)
}
fn message<'a>(account: &'a str, mailbox: &'a str, envelope: &'a [u8]) -> MessageUpsert<'a> {
    MessageUpsert {
        account_id: account,
        mailbox_id: mailbox,
        uid: 1,
        uidvalidity: 1,
        message_id: Some("coherence@example.org"),
        thread_id: None,
        internaldate: None,
        size: 1,
        flags_json: "[]",
        envelope: Some(envelope),
        blob_ref: None,
    }
}
async fn read(engine: &Engine, id: &str) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
    let row = engine
        .store()
        .get_messages(&[id.to_owned()])
        .await
        .unwrap()
        .remove(0)
        .unwrap();
    (
        engine.cached_envelope_for_message(&row).await.unwrap(),
        engine.cached_body_for_message(&row).await.unwrap(),
    )
}

#[tokio::test]
async fn independent_replica_caches_follow_shared_store_writes_and_expunge() {
    let dir = std::env::temp_dir().join(format!("mw-cache-replicas-{}", fixture_id()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("mail.db");
    let key = ServerKey::generate();
    let config = mw_store::encryption::EncryptionConfig::off();
    let first_store = Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &config)
        .await
        .unwrap();
    let writer = Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &config)
        .await
        .unwrap();
    let (account, mailbox) = account(&writer).await;
    let (id, _) = writer
        .upsert_message_with_body(&message(&account, &mailbox, b"old envelope"), b"old body")
        .await
        .unwrap();
    let first = Engine::new(first_store);
    let second = Engine::new(
        Store::open_with_encryption(path.to_str().unwrap(), key, &config)
            .await
            .unwrap(),
    );
    let first_cache = cache();
    let second_cache = cache();
    first.attach_v6(V6Hooks::new().with_cache(first_cache.clone()));
    second.attach_v6(V6Hooks::new().with_cache(second_cache.clone()));
    let expected = (Some(b"old envelope".to_vec()), Some(b"old body".to_vec()));
    assert_eq!(read(&first, &id).await, expected);
    assert_eq!(read(&second, &id).await, expected);
    let old_row = writer.get_message(&id).await.unwrap();
    let (same, _) = writer
        .upsert_message_with_body(&message(&account, &mailbox, b"new envelope"), b"new body")
        .await
        .unwrap();
    assert_eq!(id, same);
    let expected = (Some(b"new envelope".to_vec()), Some(b"new body".to_vec()));
    // Neither Cache receives invalidate(), a push event, or a pubsub message.
    assert_eq!(read(&first, &id).await, expected);
    assert_eq!(read(&second, &id).await, expected);
    let row = writer.get_message(&id).await.unwrap();
    assert!(first_cache.memory_contains(
        CacheClass::HeaderWindows,
        &mw_cache::envelope_cache_key(&account, &id, &row.cache_revision)
    ));
    assert_ne!(row.blob_ref, old_row.blob_ref);
    assert!(
        first
            .cached_envelope("another-account", &id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        first
            .cached_body("another-account", &id, row.blob_ref.as_deref().unwrap())
            .await
            .unwrap()
            .is_none()
    );
    writer.delete_message(&id).await.unwrap();
    for engine in [&first, &second] {
        assert!(
            engine
                .cached_envelope(&account, &id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            engine
                .cached_body(&account, &id, row.blob_ref.as_deref().unwrap())
                .await
                .unwrap()
                .is_none()
        );
    }
    writer.close().await;
    drop(first);
    drop(second);
    drop(writer);
    for attempt in 0..100 {
        match std::fs::remove_dir_all(&dir) {
            Ok(()) => break,
            Err(e) if attempt == 99 => panic!("cache replica fixture cleanup: {e}"),
            Err(_) => tokio::time::sleep(std::time::Duration::from_millis(10)).await,
        }
    }
}

#[tokio::test]
async fn cache_hits_fail_closed_on_store_outage_and_validate_after_reopen() {
    let dir = std::env::temp_dir().join(format!("mw-cache-outage-{}", fixture_id()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("mail.db");
    let key = ServerKey::generate();
    let config = mw_store::encryption::EncryptionConfig::off();
    let store = Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &config)
        .await
        .unwrap();
    let (account, mailbox) = account(&store).await;
    let (id, _) = store
        .upsert_message_with_body(&message(&account, &mailbox, b"before"), b"body before")
        .await
        .unwrap();
    let engine = Engine::new(store);
    let cache = cache();
    engine.attach_v6(V6Hooks::new().with_cache(cache.clone()));
    read(&engine, &id).await;
    engine.store().close().await;
    assert!(engine.cached_envelope(&account, &id).await.is_err());
    let reopened = Store::open_with_encryption(path.to_str().unwrap(), key, &config)
        .await
        .unwrap();
    reopened
        .upsert_message_with_body(&message(&account, &mailbox, b"after"), b"body after")
        .await
        .unwrap();
    let restored = Engine::new(reopened);
    restored.attach_v6(V6Hooks::new().with_cache(cache));
    assert_eq!(
        read(&restored, &id).await,
        (Some(b"after".to_vec()), Some(b"body after".to_vec()))
    );
    restored.store().close().await;
    drop(restored);
    drop(engine);
    for attempt in 0..100 {
        match std::fs::remove_dir_all(&dir) {
            Ok(()) => break,
            Err(e) if attempt == 99 => panic!("cache outage fixture cleanup: {e}"),
            Err(_) => tokio::time::sleep(std::time::Duration::from_millis(10)).await,
        }
    }
}

#[tokio::test]
async fn live_zero_access_posture_overrides_stale_ram_once_per_account() {
    struct LiveZero(std::sync::atomic::AtomicUsize);
    impl AccountPostureSource for LiveZero {
        fn posture(&self, _: &str) -> AccountPosture {
            AccountPosture::Standard
        }
        fn posture_live<'a>(
            &'a self,
            _: &'a str,
        ) -> Pin<Box<dyn Future<Output = AccountPosture> + Send + 'a>> {
            Box::pin(async move {
                self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                AccountPosture::ZeroAccess
            })
        }
    }
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (account, mailbox) = account(&store).await;
    let (id, _) = store
        .upsert_message_with_body(
            &message(&account, &mailbox, b"private header"),
            b"private body",
        )
        .await
        .unwrap();
    let row = store.get_message(&id).await.unwrap();
    let engine = Engine::new(store);
    let cache = cache();
    let source = Arc::new(LiveZero(std::sync::atomic::AtomicUsize::new(0)));
    engine.attach_v6(
        V6Hooks::new()
            .with_cache(cache.clone())
            .with_posture_source(source.clone()),
    );
    let proofs = engine
        .mail_cache_postures(&[Some(row.clone()), Some(row.clone())])
        .await;
    assert_eq!(source.0.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(proofs[&account], AccountPosture::ZeroAccess);
    assert!(
        engine
            .cached_envelope_with_posture(&row, proofs[&account])
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        engine
            .cached_body_with_posture(&row, proofs[&account])
            .await
            .unwrap()
            .is_some()
    );
    assert!(!cache.memory_contains(
        CacheClass::HeaderWindows,
        &mw_cache::envelope_cache_key(&account, &id, &row.cache_revision)
    ));
    assert!(!cache.memory_contains(
        CacheClass::MessageBodies,
        &mw_cache::body_cache_key(&account, row.blob_ref.as_deref().unwrap())
    ));
    engine.store().close().await;
}
