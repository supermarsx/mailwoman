use super::*;
use crate::{Credentials, NewAccount, ServerKey, StoredMeta};

async fn joined_seed(store: &Store, label: &str) -> (String, String) {
    let username = format!("{}-{label}@join.invalid", crate::seal::random_token());
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "join.invalid",
                port: 993,
                tls: "implicit",
                username: &username,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: username.clone(),
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
    let blob = store
        .put_body(&account, b"authoritative body never selected by metadata")
        .await
        .unwrap();
    let id = store
        .upsert_message(&MessageUpsert {
            account_id: &account,
            mailbox_id: &mailbox,
            uid: 1,
            uidvalidity: 1,
            message_id: None,
            thread_id: None,
            internaldate: None,
            size: 44,
            flags_json: "[]",
            envelope: Some(b"sealed envelope"),
            blob_ref: Some(&blob),
        })
        .await
        .unwrap();
    store
        .upsert_message_meta(
            &id,
            &StoredMeta {
                pinned: true,
                snoozed_until: Some("2030-01-01T00:00:00Z".into()),
                follow_up_at: Some("2031-01-01T00:00:00Z".into()),
            },
        )
        .await
        .unwrap();
    // Corrupt BOTH owned and foreign ciphertext. Successful metadata projection
    // proves neither payload is opened as an incidental consequence of the join.
    q("UPDATE messages SET envelope_json=?1 WHERE stable_id=?2")
        .bind(vec![0u8; 12])
        .bind(&id)
        .execute(&store.backend)
        .await
        .unwrap();
    q("UPDATE bodies SET sealed_bytes=?1 WHERE blob_ref=?2")
        .bind(vec![0u8; 12])
        .bind(blob)
        .execute(&store.backend)
        .await
        .unwrap();
    (account, id)
}
async fn joined_contract(store: &Store) {
    let (owner, own) = joined_seed(store, "owner").await;
    let (foreign, other) = joined_seed(store, "foreign").await;
    let ids = vec![own.clone(), other.clone(), "missing".into(), own.clone()];
    let messages = store.get_account_messages(&owner, &ids).await.unwrap();
    let metas = store.get_account_message_metas(&owner, &ids).await.unwrap();
    let expected = messages
        .into_iter()
        .zip(metas)
        .map(|(m, meta)| m.map(|m| (m, meta)))
        .collect::<Vec<_>>();
    let (result, queries) =
        crate::pool_telemetry::measure_queries(store.get_account_messages_with_metas(&owner, &ids))
            .await;
    assert_eq!(result.unwrap(), expected);
    assert_eq!(queries.read.calls, 1, "one actual SELECT, not per-ID calls");
    assert_eq!(
        queries.other.calls, 0,
        "WITH classified Other must not hide I/O"
    );
    assert_eq!(queries.write.calls, 0);
    assert!(expected[1].is_none() && expected[2].is_none());
    assert_eq!(expected[0], expected[3]);
    assert!(expected[0].as_ref().unwrap().1.as_ref().unwrap().pinned);
    assert!(store.get_account_envelope(&owner, &own).await.is_err());
    assert!(
        store
            .get_account_body(
                &owner,
                expected[0].as_ref().unwrap().0.blob_ref.as_ref().unwrap()
            )
            .await
            .is_err()
    );
    for count in [5, 50] {
        let asked = vec![own.clone(); count];
        let (rows, cost) = crate::pool_telemetry::measure_queries(
            store.get_account_messages_with_metas(&owner, &asked),
        )
        .await;
        assert_eq!(rows.unwrap(), vec![expected[0].clone(); count]);
        assert_eq!(cost.read.calls, 1);
        assert_eq!(cost.other.calls + cost.write.calls, 0);
    }
    store
        .upsert_message_meta(&own, &StoredMeta::default())
        .await
        .unwrap();
    assert_eq!(
        store
            .get_account_messages_with_metas(&owner, &[own.clone()])
            .await
            .unwrap()[0]
            .as_ref()
            .unwrap()
            .1,
        Some(StoredMeta::default())
    );
    q("DELETE FROM message_meta WHERE stable_id=?1")
        .bind(&own)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        store
            .get_account_messages_with_metas(&owner, &[own.clone()])
            .await
            .unwrap()[0]
            .as_ref()
            .unwrap()
            .1
            .is_none()
    );
    let (empty, cost) =
        crate::pool_telemetry::measure_queries(store.get_account_messages_with_metas(&owner, &[]))
            .await;
    assert!(empty.unwrap().is_empty());
    assert_eq!(cost.read.calls + cost.other.calls + cost.write.calls, 0);
    let (too_many, cost) = crate::pool_telemetry::measure_queries(
        store.get_account_messages_with_metas(&owner, &vec![own; 5001]),
    )
    .await;
    assert!(matches!(too_many, Err(StoreError::Invalid(_))));
    assert_eq!(cost.read.calls + cost.other.calls + cost.write.calls, 0);
    for account in [owner, foreign] {
        q("DELETE FROM accounts WHERE id=?1")
            .bind(account)
            .execute(&store.backend)
            .await
            .unwrap();
    }
}
#[tokio::test]
async fn sqlite_owned_messages_and_metadata_share_one_fresh_payload_free_read() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    joined_contract(&store).await;
    store.close().await;
}
#[tokio::test]
async fn postgres_owned_messages_and_metadata_share_one_fresh_payload_free_read() {
    use sqlx::Connection;
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        eprintln!("SKIP: MW_TEST_PG absent");
        return;
    };
    let schema = format!("owned_message_meta_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&mut admin)
        .await
        .unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    let store = Store::open(scoped.as_str(), ServerKey::generate())
        .await
        .unwrap();
    joined_contract(&store).await;
    store.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&mut admin)
        .await
        .unwrap();
}
