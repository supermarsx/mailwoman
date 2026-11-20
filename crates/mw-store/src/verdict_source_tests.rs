use super::*;
use crate::{AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey};
async fn account(store: &Store, name: &str) -> String {
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "fixture.test",
                port: 993,
                tls: "implicit",
                username: name,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: name.into(),
                password: "synthetic".into(),
            },
        )
        .await
        .unwrap()
}
async fn fixture(store: &Store) -> (String, String, String) {
    let a = account(store, "a@example.test").await;
    let b = account(store, "b@example.test").await;
    let mailbox = store
        .upsert_mailbox(&MailboxUpsert {
            account_id: &a,
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
    let body = store
        .put_body(
            &a,
            b"From: a@example.test\r\nSubject: synthetic\r\n\r\nbody",
        )
        .await
        .unwrap();
    let id = store
        .upsert_message(&MessageUpsert {
            account_id: &a,
            mailbox_id: &mailbox,
            uid: 1,
            uidvalidity: 1,
            message_id: Some("<fixture@example.test>"),
            thread_id: None,
            internaldate: None,
            size: 57,
            flags_json: "[]",
            envelope: Some(b"{}"),
            blob_ref: Some(&body),
        })
        .await
        .unwrap();
    (a, b, id)
}

async fn contract(store: Store) {
    let (a, b, id) = fixture(&store).await;
    assert!(
        store
            .security_verdict_source(&b, &id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .security_verdict_source(&a, "missing")
            .await
            .unwrap()
            .is_none()
    );
    let (source, raw) = store
        .security_verdict_source(&a, &id)
        .await
        .unwrap()
        .unwrap();
    assert!(raw.starts_with(b"From:"));
    assert!(!format!("{source:?}").contains(&id));
    assert!(
        store
            .security_verdict_cache_if_current(&source, "raw-hash-not-blob", b"derived")
            .await
            .unwrap()
    );
    assert_eq!(
        store
            .security_verdict_cached(&source, "raw-hash-not-blob")
            .await
            .unwrap()
            .unwrap(),
        b"derived"
    );
    assert!(
        store
            .security_verdict_cached(&source, "wrong-raw-hash")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .get_security_verdicts(&b, &[id.clone()])
            .await
            .unwrap()
            .is_empty()
    );
    q("INSERT INTO message_body_cache_lease(stable_id,account_id,blob_ref,uidvalidity,uid,last_used_at) VALUES(?1,?2,?3,1,1,1) ON CONFLICT(stable_id) DO UPDATE SET last_used_at=1")
        .bind(&id).bind(&a).bind(&source.blob).execute(&store.backend).await.unwrap();
    assert_eq!(store.renew_cached_body_use(&a, &id).await.unwrap(), 1);
    q("UPDATE bodies SET sealed_bytes=sealed_bytes WHERE blob_ref=?1")
        .bind(&source.blob)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        store
            .security_verdict_source_current(&source)
            .await
            .unwrap()
    );
    assert_eq!(
        store
            .security_verdict_cached(&source, "raw-hash-not-blob")
            .await
            .unwrap()
            .unwrap(),
        b"derived"
    );
    let before = store.get_account_messages(&a, &[id.clone()]).await.unwrap()[0]
        .clone()
        .unwrap();
    store.set_flags(&id, "[\"\\Seen\"]").await.unwrap();
    let destination = store
        .upsert_mailbox(&MailboxUpsert {
            account_id: &a,
            name: "Archive",
            role: None,
            uidvalidity: 2,
            uidnext: 2,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        })
        .await
        .unwrap();
    q("UPDATE messages SET mailbox_id=?2,uid=2,uidvalidity=2 WHERE stable_id=?1")
        .bind(&id)
        .bind(&destination)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        store
            .security_verdict_source_current(&source)
            .await
            .unwrap()
    );
    assert_eq!(
        store
            .get_security_verdicts(&a, &[id.clone()])
            .await
            .unwrap()[&id],
        b"derived"
    );
    // A response assembled from old rows cannot adopt a newly read verdict.
    assert!(
        store
            .security_verdicts_for_rows(&a, &[(id.clone(), before.cache_revision)])
            .await
            .unwrap()
            .is_empty()
    );
    let sealed = store
        .key
        .seal(b"From: changed@example.test\r\n\r\nreplaced")
        .unwrap();
    q("UPDATE bodies SET sealed_bytes=?2 WHERE blob_ref=?1")
        .bind(&source.blob)
        .bind(sealed)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        !store
            .security_verdict_source_current(&source)
            .await
            .unwrap()
    );
    assert!(
        !store
            .security_verdict_cache_if_current(&source, "old", b"stale")
            .await
            .unwrap()
    );
    assert!(
        store
            .security_verdict_cached(&source, "raw-hash-not-blob")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .get_security_verdicts(&a, &[id.clone()])
            .await
            .unwrap()
            .is_empty()
    );
    let (next, raw) = store
        .security_verdict_source(&a, &id)
        .await
        .unwrap()
        .unwrap();
    assert!(raw.windows(7).any(|x| x == b"changed"));
    assert!(
        store
            .security_verdict_cache_if_current(&next, "new", b"new")
            .await
            .unwrap()
    );
    // Body deletion/reinsertion with the same opaque blob ID is a new epoch.
    let replacement = store
        .key
        .seal(b"From: restored@example.test\r\n\r\nrestored")
        .unwrap();
    q("DELETE FROM bodies WHERE account_id=?1 AND blob_ref=?2")
        .bind(&a)
        .bind(&next.blob)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        store
            .security_verdict_source(&a, &id)
            .await
            .unwrap()
            .is_none()
    );
    q("INSERT INTO bodies(blob_ref,account_id,sealed_bytes) VALUES(?1,?2,?3)")
        .bind(&next.blob)
        .bind(&a)
        .bind(replacement)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(!store.security_verdict_source_current(&next).await.unwrap());
    let (restored, _) = store
        .security_verdict_source(&a, &id)
        .await
        .unwrap()
        .unwrap();
    assert!(
        store
            .security_verdict_cache_if_current(&restored, "restored", b"restored")
            .await
            .unwrap()
    );
    // Zero-access excludes both existing cache and server-side source inspection.
    store
        .upsert_zeroaccess(&crate::ZeroAccessRow {
            account_id: a.clone(),
            enabled: true,
            wrapped_root_key: vec![],
            kdf_params_json: "{}".into(),
            recovery_wrapped: None,
            paired_devices_json: "[]".into(),
        })
        .await
        .unwrap();
    assert!(
        store
            .security_verdict_source(&a, &id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(!store.security_verdict_source_current(&next).await.unwrap());
    assert!(
        store
            .get_security_verdicts(&a, &[id.clone()])
            .await
            .unwrap()
            .is_empty()
    );
    store.set_zeroaccess_enabled(&a, false).await.unwrap();
    q("DELETE FROM messages WHERE stable_id=?1")
        .bind(&id)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(!store.security_verdict_source_current(&next).await.unwrap());
    assert!(
        store
            .get_security_verdicts(&a, &[id.clone()])
            .await
            .unwrap()
            .is_empty()
    );
    q("INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,size,flags_json,blob_ref) VALUES(?1,?2,?3,4,2,20,'[]',?4)").bind(&id).bind(&a).bind(&destination).bind(&restored.blob).execute(&store.backend).await.unwrap();
    assert!(
        !store
            .security_verdict_source_current(&restored)
            .await
            .unwrap()
    );
    assert!(
        store
            .get_security_verdicts(&a, &[id])
            .await
            .unwrap()
            .is_empty()
    );
    store.close().await;
}
#[tokio::test]
async fn owned_content_epoch_sqlite_contract() {
    contract(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn owned_content_epoch_postgres_contract() {
    use sqlx::Connection;
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        eprintln!("SKIP: MW_TEST_PG absent");
        return;
    };
    let schema = format!("verdict_source_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&mut admin)
        .await
        .unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    contract(
        Store::open(scoped.as_str(), ServerKey::generate())
            .await
            .unwrap(),
    )
    .await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&mut admin)
        .await
        .unwrap();
}
#[tokio::test]
async fn foreign_corrupt_oversized_and_cross_store_proofs_fail_closed() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (a, b, id) = fixture(&store).await;
    let (source, _) = store
        .security_verdict_source(&a, &id)
        .await
        .unwrap()
        .unwrap();
    let other = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    assert!(
        !other
            .security_verdict_source_current(&source)
            .await
            .unwrap()
    );
    assert!(
        !other
            .security_verdict_cache_if_current(&source, "h", b"foreign")
            .await
            .unwrap()
    );
    q("UPDATE bodies SET sealed_bytes=zeroblob(67108909) WHERE blob_ref=?1")
        .bind(&source.blob)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        store
            .security_verdict_source(&b, &id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(store.security_verdict_source(&a, &id).await.is_err());
    q("UPDATE bodies SET sealed_bytes=x'ff' WHERE blob_ref=?1")
        .bind(&source.blob)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(store.security_verdict_source(&a, &id).await.is_err());
}

#[tokio::test]
async fn account_reassignment_and_legacy_cached_foreign_rows_never_authorize_a_source() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (a, b, id) = fixture(&store).await;
    let (source, _) = store
        .security_verdict_source(&a, &id)
        .await
        .unwrap()
        .unwrap();
    store
        .upsert_security_verdict(&crate::SecurityVerdictRow {
            email_id: id.clone(),
            account_id: b.clone(),
            raw_hash: "h".into(),
            verdict_json: b"foreign cached metadata".to_vec(),
            computed_at: "now".into(),
        })
        .await
        .unwrap();
    assert!(
        store
            .get_security_verdicts(&b, &[id.clone()])
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .get_security_verdicts(&a, &[id.clone()])
            .await
            .unwrap()
            .is_empty()
    );
    q("UPDATE messages SET account_id=?2 WHERE stable_id=?1")
        .bind(&id)
        .bind(&b)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        !store
            .security_verdict_source_current(&source)
            .await
            .unwrap()
    );
    assert!(
        !store
            .security_verdict_cache_if_current(&source, "h", b"old")
            .await
            .unwrap()
    );
    assert!(
        store
            .security_verdict_source(&a, &id)
            .await
            .unwrap()
            .is_none()
    );
    // Reassigning a header does not authorize another account's body blob.
    assert!(
        store
            .security_verdict_source(&b, &id)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn aggregate_verdict_payload_is_guarded_before_bulk_transfer() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (a, _, id) = fixture(&store).await;
    let row = store.get_message(&id).await.unwrap();
    let mut ids = Vec::new();
    for n in 0..9 {
        let next = format!("bounded-verdict-{n}");
        q("INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,size,flags_json,blob_ref) VALUES(?1,?2,?3,?4,1,20,'[]',?5)").bind(&next).bind(&a).bind(&row.mailbox_id).bind(n+20i64).bind(row.blob_ref.as_deref()).execute(&store.backend).await.unwrap();
        q("INSERT INTO security_verdicts(email_id,account_id,raw_hash,verdict_json,computed_at,source_revision) SELECT stable_id,account_id,'h',zeroblob(1048576),'now',content_revision FROM messages WHERE stable_id=?1").bind(&next).execute(&store.backend).await.unwrap();
        ids.push(next);
    }
    assert!(matches!(
        store.get_security_verdicts(&a, &ids).await,
        Err(StoreError::Corrupt(_))
    ));
}
