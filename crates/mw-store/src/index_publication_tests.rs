use super::*;
use crate::{AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount};

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
fn request(account: &str, ids: &[&str]) -> IndexPublicationRequest {
    IndexPublicationRequest {
        account_id: account.into(),
        stable_ids: ids.iter().map(|s| (*s).into()).collect(),
    }
}
async fn contract(store: Store) {
    let (a, b, id) = fixture(&store).await;
    let requests = [
        request(&a, &[&id, "missing", &id]),
        request(&b, &[&id]),
        request("missing-account", &[]),
    ];
    let (before, count) =
        crate::pool_telemetry::measure_queries(store.index_publication_snapshot(&requests, 100))
            .await;
    let before = before.unwrap();
    // WITH is conservatively classified Other because a CTE can contain writes.
    assert_eq!(count.read.calls + count.other.calls, 1);
    assert!(before.account(&a).unwrap().exists);
    assert!(!before.account("missing-account").unwrap().exists);
    let current = before.message(&a, &id).unwrap();
    assert_eq!(
        current.message.as_ref(),
        store.get_account_messages(&a, &[id.clone()]).await.unwrap()[0].as_ref()
    );
    assert_eq!(
        current.meta.as_ref(),
        store
            .get_account_message_metas(&a, &[id.clone()])
            .await
            .unwrap()[0]
            .as_ref()
    );
    assert!(current.envelope_bytes.is_some_and(|n| n > 0));
    assert!(current.raw_body_bytes.is_some_and(|n| n > 57));
    let metadata = store
        .index_publication_metadata_snapshot(&requests, 100)
        .await
        .unwrap();
    assert_eq!(metadata.account(&a), before.account(&a));
    let mut comparable = current.clone();
    comparable.raw_body_bytes = None;
    assert_eq!(metadata.message(&a, &id), Some(&comparable));
    assert!(
        metadata
            .matches(
                &store
                    .index_publication_metadata_snapshot(&requests, 100)
                    .await
                    .unwrap()
            )
            .unwrap()
    );
    let expected = current.message.as_ref().unwrap();
    assert_eq!(
        store
            .index_publication_envelope(expected, 1024)
            .await
            .unwrap()
            .as_deref(),
        Some(b"{}".as_slice())
    );
    assert!(store.index_publication_envelope(expected, 1).await.is_err());
    assert!(
        store
            .index_publication_envelope(expected, 64 * 1024 * 1024 + 1)
            .await
            .is_err()
    );
    let source = store
        .index_publication_source(expected, 1024)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(source.envelope.as_deref(), Some(b"{}".as_slice()));
    assert!(
        source
            .raw
            .as_ref()
            .unwrap()
            .starts_with(b"From: a@example.test")
    );
    assert!(!format!("{source:?}").contains("From:"));
    assert!(store.index_publication_source(expected, 1).await.is_err());
    assert!(
        store
            .index_publication_source(expected, 64 * 1024 * 1024 + 1)
            .await
            .is_err()
    );
    let mut foreign_expected = expected.clone();
    foreign_expected.account_id = b.clone();
    assert!(
        store
            .index_publication_envelope(&foreign_expected, 1024)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .index_publication_source(&foreign_expected, 1024)
            .await
            .unwrap()
            .is_none()
    );
    let foreign = before.message(&b, &id).unwrap();
    assert_eq!(foreign.current_owner.as_deref(), Some(a.as_str()));
    assert!(
        foreign.message.is_none()
            && foreign.meta.is_none()
            && foreign.raw_body_bytes.is_none()
            && foreign.envelope_bytes.is_none()
    );
    assert!(
        before
            .message(&a, "missing")
            .unwrap()
            .current_owner
            .is_none()
    );
    assert!(
        before
            .matches(
                &store
                    .index_publication_snapshot(&requests, 100)
                    .await
                    .unwrap()
            )
            .unwrap()
    );
    store
        .upsert_zeroaccess(&crate::ZeroAccessRow {
            account_id: a.clone(),
            enabled: true,
            wrapped_root_key: vec![1],
            kdf_params_json: "{}".into(),
            recovery_wrapped: None,
            paired_devices_json: "[]".into(),
        })
        .await
        .unwrap();
    let zero = store
        .index_publication_snapshot(&requests, 100)
        .await
        .unwrap();
    assert!(!before.matches(&zero).unwrap());
    assert!(zero.account(&a).unwrap().zero_access);
    store.set_zeroaccess_enabled(&a, false).await.unwrap();
    q("INSERT INTO account_restriction(id,account_id,address,domain,scopes_json,mail_delivery,reason,source,created_by,created_at,expires_at) VALUES('publication-restriction',?1,'a@example.test','example.test','[\"indexing\"]','keep','fixture','fixture','fixture',1,200)")
        .bind(&a).execute(&store.backend).await.unwrap();
    let restricted = store
        .index_publication_snapshot(&requests, 100)
        .await
        .unwrap();
    assert!(restricted.account(&a).unwrap().indexing_restricted);
    assert_eq!(
        restricted.account(&a).unwrap().indexing_restricted,
        store
            .restriction_active(&a, 100)
            .await
            .unwrap()
            .unwrap()
            .covers("indexing")
    );
    assert!(!before.matches(&restricted).unwrap());
    let expired = store
        .index_publication_snapshot(&requests, 201)
        .await
        .unwrap();
    assert!(!expired.account(&a).unwrap().indexing_restricted);
    assert!(!restricted.matches(&expired).unwrap());
    // Raw SQL bypasses process-local invalidation and must still be observed.
    q("UPDATE account_restriction SET scopes_json='malformed' WHERE id='publication-restriction'")
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        store
            .index_publication_snapshot(&requests, 100)
            .await
            .is_err()
    );
    q("DELETE FROM account_restriction WHERE id='publication-restriction'")
        .execute(&store.backend)
        .await
        .unwrap();
    // The SQL itself emits only a small overflow row before JSON construction;
    // the Rust caller does not download a giant scalar merely to reject it.
    q("UPDATE messages SET flags_json=?1 WHERE stable_id=?2")
        .bind("x".repeat(256 * 1024 + 1))
        .bind(&id)
        .execute(&store.backend)
        .await
        .unwrap();
    let encoded = serde_json::json!([{"account":a,"id":id}]).to_string();
    let rows = q(publication_sql(store.backend.dialect()))
        .bind(encoded)
        .fetch_all(&store.backend)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get_string("tag"), "overflow");
    assert_eq!(rows[0].get_string("payload"), "{}");
    assert!(
        store
            .index_publication_snapshot(&requests, 100)
            .await
            .is_err()
    );
    q("UPDATE messages SET flags_json='[]' WHERE stable_id=?1")
        .bind(&id)
        .execute(&store.backend)
        .await
        .unwrap();
    q("INSERT INTO account_restriction(id,account_id,address,domain,scopes_json,mail_delivery,reason,source,created_by,created_at) VALUES('oversized-restriction',?1,'a@example.test','example.test',?2,'keep','fixture','fixture','fixture',1)")
        .bind(&a).bind("x".repeat(64*1024+1)).execute(&store.backend).await.unwrap();
    let encoded = serde_json::json!([{"account":a,"id":id}]).to_string();
    let rows = q(publication_sql(store.backend.dialect()))
        .bind(encoded)
        .fetch_all(&store.backend)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get_string("tag"), "overflow");
    assert_eq!(rows[0].get_string("payload"), "{}");
    q("DELETE FROM account_restriction WHERE id='oversized-restriction'")
        .execute(&store.backend)
        .await
        .unwrap();
    // Individually legal scalars can exceed the aggregate working-set budget.
    // Only the constant overflow signal may cross the SQL/Rust boundary.
    let legal_scalar = "x".repeat(256 * 1024);
    let mut large_ids = Vec::new();
    for n in 0..33 {
        let sid = format!("publication-budget-{n}");
        q("INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,message_id,thread_id,internaldate,size,flags_json,blob_ref,envelope_json) SELECT ?1,account_id,mailbox_id,?2,uidvalidity,message_id,thread_id,internaldate,size,?3,blob_ref,envelope_json FROM messages WHERE stable_id=?4")
            .bind(&sid).bind(i64::from(n+100)).bind(&legal_scalar).bind(&id).execute(&store.backend).await.unwrap();
        large_ids.push(sid);
    }
    let encoded = serde_json::to_string(
        &large_ids
            .iter()
            .map(|sid| serde_json::json!({"account":a,"id":sid}))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let rows = q(publication_sql(store.backend.dialect()))
        .bind(encoded)
        .fetch_all(&store.backend)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get_string("tag"), "overflow");
    assert_eq!(rows[0].get_string("payload"), "{}");
    // Metadata has no message cache_revision trigger: compare it independently.
    let pre_meta = store
        .index_publication_snapshot(&requests, 100)
        .await
        .unwrap();
    store
        .upsert_message_meta(
            &id,
            &StoredMeta {
                pinned: true,
                snoozed_until: None,
                follow_up_at: None,
            },
        )
        .await
        .unwrap();
    let meta = store
        .index_publication_snapshot(&requests, 100)
        .await
        .unwrap();
    assert!(!before.matches(&meta).unwrap());
    assert!(meta.message(&a, &id).unwrap().meta.as_ref().unwrap().pinned);
    assert_eq!(
        pre_meta
            .message(&a, &id)
            .unwrap()
            .message
            .as_ref()
            .unwrap()
            .cache_revision,
        meta.message(&a, &id)
            .unwrap()
            .message
            .as_ref()
            .unwrap()
            .cache_revision
    );
    assert!(!pre_meta.matches(&meta).unwrap());
    q("UPDATE messages SET flags_json='[\"\\Seen\"]' WHERE stable_id=?1")
        .bind(&id)
        .execute(&store.backend)
        .await
        .unwrap();
    let flags = store
        .index_publication_snapshot(&requests, 100)
        .await
        .unwrap();
    assert!(!meta.matches(&flags).unwrap());
    assert!(
        !metadata
            .matches(
                &store
                    .index_publication_metadata_snapshot(&requests, 100)
                    .await
                    .unwrap()
            )
            .unwrap()
    );
    assert!(
        store
            .index_publication_envelope(expected, 1024)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .index_publication_source(expected, 1024)
            .await
            .unwrap()
            .is_none(),
        "changed revision rejects old reconstruction"
    );
    // Raw reassignments cannot reveal the formerly owned document to old owner.
    q("UPDATE messages SET account_id=?1 WHERE stable_id=?2")
        .bind(&b)
        .bind(&id)
        .execute(&store.backend)
        .await
        .unwrap();
    let moved = store
        .index_publication_snapshot(&requests, 100)
        .await
        .unwrap();
    assert!(!flags.matches(&moved).unwrap());
    let prior = flags.message(&a, &id).unwrap().message.as_ref().unwrap();
    assert!(
        store
            .index_publication_source(prior, 1024)
            .await
            .unwrap()
            .is_none(),
        "reassignment rejects prior owned body reader"
    );
    assert!(moved.message(&a, &id).unwrap().message.is_none());
    assert_eq!(
        moved.message(&a, &id).unwrap().current_owner.as_deref(),
        Some(b.as_str())
    );
    assert!(
        moved.message(&b, &id).unwrap().raw_body_bytes.is_none(),
        "body belongs to previous account"
    );
    q("DELETE FROM messages WHERE stable_id=?1")
        .bind(&id)
        .execute(&store.backend)
        .await
        .unwrap();
    let erased = store
        .index_publication_snapshot(&requests, 100)
        .await
        .unwrap();
    assert!(!moved.matches(&erased).unwrap());
    assert!(erased.message(&a, &id).unwrap().current_owner.is_none());
    store.close().await;
}
#[tokio::test]
async fn publication_snapshot_sqlite_source_owner_meta_and_erasure() {
    contract(
        Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap(),
    )
    .await;
}
#[tokio::test]
async fn publication_snapshot_postgres_source_owner_meta_and_erasure() {
    use sqlx::Connection;
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        eprintln!("SKIP: MW_TEST_PG absent");
        return;
    };
    let schema = format!("publication_snapshot_{}", uuid::Uuid::new_v4().simple());
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
        Store::open(scoped.as_str(), crate::ServerKey::generate())
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
async fn publication_snapshot_scope_limits_and_closed_database_fail_closed() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let other = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let empty = store.index_publication_snapshot(&[], 100).await.unwrap();
    assert!(
        empty
            .matches(&other.index_publication_snapshot(&[], 100).await.unwrap())
            .is_err()
    );
    let oversized = vec![request("account", &[]); MAX_PUBLICATION_ACCOUNTS + 1];
    let (result, count) =
        crate::pool_telemetry::measure_queries(store.index_publication_snapshot(&oversized, 100))
            .await;
    assert!(result.is_err());
    assert_eq!(count.read.calls + count.other.calls, 0);
    store.close().await;
    assert!(
        store
            .index_publication_snapshot(&[request("account", &[])], 100)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn publication_snapshot_sqlite_malformed_posture_fails_closed() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let (a, _, id) = fixture(&store).await;
    store
        .upsert_zeroaccess(&crate::ZeroAccessRow {
            account_id: a.clone(),
            enabled: false,
            wrapped_root_key: vec![1],
            kdf_params_json: "{}".into(),
            recovery_wrapped: None,
            paired_devices_json: "[]".into(),
        })
        .await
        .unwrap();
    q("UPDATE zeroaccess_accounts SET enabled='broken' WHERE account_id=?1")
        .bind(&a)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        store
            .index_publication_snapshot(&[request(&a, &[&id])], 100)
            .await
            .is_err()
    );
}

#[test]
fn publication_debug_never_prints_identity_source_or_generation() {
    let request = request("private-account", &["private-message"]);
    let message = IndexPublicationMessage {
        current_owner: Some("private-account".into()),
        message: None,
        meta: Some(StoredMeta {
            pinned: true,
            snoozed_until: Some("private-meta".into()),
            follow_up_at: None,
        }),
        envelope_bytes: None,
        raw_body_bytes: None,
        content_blob_ref: None,
    };
    let snapshot = IndexPublicationSnapshot {
        scope: uuid::Uuid::new_v4(),
        generation: Some("private-generation.db".into()),
        accounts: BTreeMap::new(),
        messages: BTreeMap::from([(
            ("private-account".into(), "private-message".into()),
            message.clone(),
        )]),
    };
    for output in [
        format!("{request:?}"),
        format!("{message:?}"),
        format!("{snapshot:?}"),
    ] {
        assert!(!output.contains("private-"));
    }
}

#[tokio::test]
async fn publication_snapshot_expired_content_identity_is_source_bound() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let (a, _, id) = fixture(&store).await;
    let requests = [request(&a, &[&id])];
    let before = store
        .index_publication_snapshot(&requests, 100)
        .await
        .unwrap();
    let blob = before
        .message(&a, &id)
        .unwrap()
        .message
        .as_ref()
        .unwrap()
        .blob_ref
        .clone()
        .unwrap();
    q("INSERT INTO message_body_cache_lease(stable_id,account_id,blob_ref,uidvalidity,uid,last_used_at) VALUES(?1,?2,?3,1,1,1) ON CONFLICT(stable_id) DO UPDATE SET blob_ref=excluded.blob_ref")
        .bind(&id).bind(&a).bind(&blob).execute(&store.backend).await.unwrap();
    q("UPDATE messages SET blob_ref=NULL WHERE stable_id=?1")
        .bind(&id)
        .execute(&store.backend)
        .await
        .unwrap();
    let expired = store
        .index_publication_snapshot(&requests, 100)
        .await
        .unwrap();
    let source = expired.message(&a, &id).unwrap();
    assert!(source.message.as_ref().unwrap().blob_ref.is_none());
    assert_eq!(source.content_blob_ref.as_deref(), Some(blob.as_str()));
    q("UPDATE message_body_cache_lease SET uid=99 WHERE stable_id=?1")
        .bind(&id)
        .execute(&store.backend)
        .await
        .unwrap();
    let changed = store
        .index_publication_snapshot(&requests, 100)
        .await
        .unwrap();
    assert!(changed.message(&a, &id).unwrap().content_blob_ref.is_none());
    assert!(!expired.matches(&changed).unwrap());
}

#[tokio::test]
async fn publication_source_budget_precedes_decrypt_and_does_not_renew_lease() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let (a, _, id) = fixture(&store).await;
    let source = store
        .get_account_messages(&a, &[id.clone()])
        .await
        .unwrap()
        .pop()
        .unwrap()
        .unwrap();
    let blob = source.blob_ref.as_ref().unwrap();
    q("INSERT INTO message_body_cache_lease(stable_id,account_id,blob_ref,uidvalidity,uid,last_used_at) VALUES(?1,?2,?3,1,1,1) ON CONFLICT(stable_id) DO UPDATE SET last_used_at=1").bind(&id).bind(&a).bind(blob).execute(&store.backend).await.unwrap();
    store
        .index_publication_source(&source, 1024)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        q("SELECT last_used_at FROM message_body_cache_lease WHERE stable_id=?1")
            .bind(&id)
            .fetch_one(&store.backend)
            .await
            .unwrap()
            .get_i64("last_used_at"),
        1
    );
    q("UPDATE bodies SET sealed_bytes=?1 WHERE blob_ref=?2")
        .bind(vec![0u8; 4096])
        .bind(blob)
        .execute(&store.backend)
        .await
        .unwrap();
    let error = store
        .index_publication_source(&source, 1024)
        .await
        .unwrap_err();
    assert!(
        matches!(error,StoreError::Corrupt(ref text) if text=="publication source byte budget exceeded")
    );
    assert!(
        store.index_publication_source(&source, 8192).await.is_err(),
        "fitting corrupt sealed source must fail decryption"
    );
}

#[test]
fn metadata_publication_sql_has_no_raw_body_relation_or_expression() {
    for dialect in [Dialect::Sqlite, Dialect::Postgres] {
        let sql = publication_sql_variant(dialect, false);
        assert!(!sql.contains("JOIN bodies"));
        assert!(!sql.contains("b.sealed_bytes"));
        assert!(
            publication_sql(dialect).contains("JOIN bodies"),
            "the full snapshot retains its original raw-body witness"
        );
    }
}
#[tokio::test]
async fn metadata_variants_ignore_corrupt_raw_and_envelope_guard_precedes_decrypt() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let (a, _, id) = fixture(&store).await;
    let requests = [request(&a, &[&id])];
    let snapshot = store
        .index_publication_metadata_snapshot(&requests, 100)
        .await
        .unwrap();
    let message = snapshot.message(&a, &id).unwrap().message.as_ref().unwrap();
    let blob = message.blob_ref.as_ref().unwrap();
    q("UPDATE bodies SET sealed_bytes=?1 WHERE blob_ref=?2")
        .bind(vec![0u8; 4096])
        .bind(blob)
        .execute(&store.backend)
        .await
        .unwrap();
    assert_eq!(
        store
            .index_publication_envelope(message, 1024)
            .await
            .unwrap()
            .as_deref(),
        Some(b"{}".as_slice())
    );
    assert!(
        snapshot
            .matches(
                &store
                    .index_publication_metadata_snapshot(&requests, 100)
                    .await
                    .unwrap()
            )
            .unwrap()
    );
    q("UPDATE messages SET envelope_json=?1 WHERE stable_id=?2")
        .bind(vec![0u8; 4096])
        .bind(&id)
        .execute(&store.backend)
        .await
        .unwrap();
    let current = store
        .get_account_messages(&a, &[id.clone()])
        .await
        .unwrap()
        .pop()
        .unwrap()
        .unwrap();
    let error = store
        .index_publication_envelope(&current, 1024)
        .await
        .unwrap_err();
    assert!(
        matches!(error,StoreError::Corrupt(ref text) if text=="publication envelope byte budget exceeded")
    );
    assert!(
        store
            .index_publication_envelope(&current, 8192)
            .await
            .is_err(),
        "fitting malformed envelope fails decrypt rather than silently becoming metadata"
    );
    assert!(
        store
            .index_publication_envelope(message, 8192)
            .await
            .unwrap()
            .is_none(),
        "old revision cannot fetch replacement envelope"
    );
}
