use super::*;
use crate::{AccountKind, Credentials, NewAccount, ServerKey, StoredMeta};

async fn seed(store: &Store, label: &str) -> (String, String) {
    let username = format!("{}-{label}@scope.invalid", crate::seal::random_token());
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "mail.invalid",
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
    let id = store
        .upsert_message(&MessageUpsert {
            account_id: &account,
            mailbox_id: &mailbox,
            uid: 1,
            uidvalidity: 1,
            message_id: None,
            thread_id: None,
            internaldate: None,
            size: 0,
            flags_json: "[]",
            envelope: Some(label.as_bytes()),
            blob_ref: None,
        })
        .await
        .unwrap();
    store
        .upsert_message_meta(
            &id,
            &StoredMeta {
                pinned: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    (account, id)
}

async fn contract(store: &Store) {
    let (owner, own_id) = seed(store, "owner").await;
    let (foreign, foreign_id) = seed(store, "FOREIGN SECRET").await;
    let ids = vec![
        own_id.clone(),
        foreign_id.clone(),
        own_id.clone(),
        "missing".into(),
    ];
    let messages = store.get_account_messages(&owner, &ids).await.unwrap();
    assert_eq!(messages[0].as_ref().unwrap().account_id, owner);
    assert!(messages[1].is_none() && messages[3].is_none());
    assert_eq!(messages[0], messages[2]);
    let metas = store.get_account_message_metas(&owner, &ids).await.unwrap();
    assert!(metas[0].as_ref().unwrap().pinned);
    assert!(metas[1].is_none() && metas[3].is_none());
    assert_eq!(metas[0], metas[2]);
    // Poison a foreign seal: successful scoped reads prove SQL excludes it
    // before plaintext decryption, rather than filtering after it was opened.
    q("UPDATE messages SET envelope_json=?1 WHERE stable_id=?2")
        .bind(vec![0u8; 12])
        .bind(&foreign_id)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(store.get_envelopes(&ids).await.is_err());
    assert_eq!(
        store.get_account_envelopes(&owner, &ids).await.unwrap(),
        vec![Some(b"owner".to_vec()), None, Some(b"owner".to_vec()), None]
    );
    let (_, queries) = crate::pool_telemetry::measure_queries(async {
        assert!(
            store
                .get_account_messages(&owner, &[])
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            store
                .get_account_message_metas(&owner, &[])
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            store
                .get_account_envelopes(&owner, &[])
                .await
                .unwrap()
                .is_empty()
        );
    })
    .await;
    assert_eq!(queries.read.calls, 0);
    for account in [owner, foreign] {
        q("DELETE FROM accounts WHERE id=?1")
            .bind(account)
            .execute(&store.backend)
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn sqlite_account_scoped_batches_exclude_foreign_rows_before_decryption() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    contract(&store).await;
    store.close().await;
}

#[tokio::test]
async fn postgres_account_scoped_batches_exclude_foreign_rows_before_decryption() {
    let Ok(dsn) = std::env::var("MW_BG_TEST_PG") else {
        eprintln!("SKIP: MW_BG_TEST_PG absent");
        return;
    };
    let store = Store::open_postgres(&dsn, ServerKey::generate())
        .await
        .unwrap();
    contract(&store).await;
    store.close().await;
}
