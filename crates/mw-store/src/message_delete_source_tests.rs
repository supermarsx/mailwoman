use super::*;
use crate::{AccountKind, Credentials, NewAccount, ServerKey};
async fn fixture(store: &Store) -> (String, String, String) {
    let username = format!("{}@delete.invalid", crate::seal::random_token());
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "fixture.invalid",
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
            unread: 0,
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
            size: 3,
            flags_json: r#"["Seen"]"#,
            envelope: Some(b"{}"),
            blob_ref: None,
        })
        .await
        .unwrap();
    store
        .put_message_embedding(&id, &account, "fixture", &[1.0, 2.0])
        .await
        .unwrap();
    (account, mailbox, id)
}
async fn source_contract(store: &Store) {
    let (account, mailbox, id) = fixture(store).await;
    let snapshot = store.get_message(&id).await.unwrap();
    let mut foreign = snapshot.clone();
    foreign.account_id = "foreign".into();
    assert!(
        !store
            .delete_synced_message_if_source(&foreign)
            .await
            .unwrap()
    );
    let mut moved = snapshot.clone();
    moved.mailbox_id = "different-mailbox".into();
    assert!(!store.delete_synced_message_if_source(&moved).await.unwrap());
    let mut rekeyed = snapshot.clone();
    rekeyed.uid += 1;
    assert!(
        !store
            .delete_synced_message_if_source(&rekeyed)
            .await
            .unwrap()
    );
    let mut validity = snapshot.clone();
    validity.uidvalidity += 1;
    assert!(
        !store
            .delete_synced_message_if_source(&validity)
            .await
            .unwrap()
    );
    assert_eq!(store.count_message_embeddings(&account).await.unwrap(), 1);
    assert_eq!(store.get_mailbox(&mailbox).await.unwrap().total, 1);
    q("UPDATE messages SET uid=10 WHERE stable_id=?1")
        .bind(&id)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        !store
            .delete_synced_message_if_source(&snapshot)
            .await
            .unwrap()
    );
    assert_eq!(store.count_message_embeddings(&account).await.unwrap(), 1);
    q("UPDATE messages SET uid=1 WHERE stable_id=?1")
        .bind(&id)
        .execute(&store.backend)
        .await
        .unwrap();
    store.set_flags(&id, "[]").await.unwrap();
    let blob = store.put_body(&account, b"new body").await.unwrap();
    q("UPDATE messages SET blob_ref=?1 WHERE stable_id=?2")
        .bind(blob)
        .bind(&id)
        .execute(&store.backend)
        .await
        .unwrap();
    let current = store.get_message(&id).await.unwrap();
    assert_ne!(current.cache_revision, snapshot.cache_revision);
    assert_eq!(store.get_mailbox(&mailbox).await.unwrap().unread, 1);
    assert!(
        store
            .delete_synced_message_if_source(&snapshot)
            .await
            .unwrap()
    );
    assert!(
        !store
            .delete_synced_message_if_source(&snapshot)
            .await
            .unwrap()
    );
    assert_eq!(store.count_message_embeddings(&account).await.unwrap(), 0);
    let mailbox = store.get_mailbox(&mailbox).await.unwrap();
    assert_eq!((mailbox.total, mailbox.unread), (0, 0));
}
#[tokio::test]
async fn source_cas_preserves_foreign_moved_rekeyed_rows_but_deletes_current_flags_and_body() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    source_contract(&store).await;
    store.close().await;
}
#[tokio::test]
async fn postgres_source_cas_uses_current_flags_and_preserves_other_sources() {
    let Ok(dsn) = std::env::var("MW_BG_TEST_PG") else {
        eprintln!("SKIP: MW_BG_TEST_PG absent");
        return;
    };
    let store = Store::open_postgres(&dsn, ServerKey::generate())
        .await
        .unwrap();
    source_contract(&store).await;
    store.close().await;
}
#[tokio::test]
async fn counter_failure_rolls_back_source_deletion_and_embedding() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (account, mailbox, id) = fixture(&store).await;
    let snapshot = store.get_message(&id).await.unwrap();
    q("CREATE TRIGGER reject_source_delete_counter BEFORE UPDATE OF total ON mailboxes BEGIN SELECT RAISE(ABORT,'fixture counter failure'); END").execute(&store.backend).await.unwrap();
    assert!(
        store
            .delete_synced_message_if_source(&snapshot)
            .await
            .is_err()
    );
    assert!(store.get_message(&id).await.is_ok());
    assert_eq!(store.count_message_embeddings(&account).await.unwrap(), 1);
    assert_eq!(store.get_mailbox(&mailbox).await.unwrap().total, 1);
    q("DROP TRIGGER reject_source_delete_counter")
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        store
            .delete_synced_message_if_source(&snapshot)
            .await
            .unwrap()
    );
    store.close().await;
}

#[tokio::test]
async fn either_history_failure_rolls_back_delete_embedding_counters_and_both_states() {
    for kind in ["Email", "Mailbox"] {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let (account, mailbox, id) = fixture(&store).await;
        let snapshot = store.get_message(&id).await.unwrap();
        let trigger = match kind {
            "Email" => "CREATE TRIGGER reject_source_delete_history BEFORE INSERT ON changes WHEN NEW.type='Email' BEGIN SELECT RAISE(ABORT,'fixture history failure'); END",
            "Mailbox" => "CREATE TRIGGER reject_source_delete_history BEFORE INSERT ON changes WHEN NEW.type='Mailbox' BEGIN SELECT RAISE(ABORT,'fixture history failure'); END",
            _ => unreachable!(),
        };
        q(trigger).execute(&store.backend).await.unwrap();
        assert!(
            store
                .delete_synced_message_if_source(&snapshot)
                .await
                .is_err()
        );
        assert!(store.get_message(&id).await.is_ok());
        assert_eq!(store.count_message_embeddings(&account).await.unwrap(), 1);
        assert_eq!(store.get_mailbox(&mailbox).await.unwrap().total, 1);
        for kind in ["Email", "Mailbox"] {
            assert_eq!(store.current_state(&account, kind).await.unwrap(), 0);
            assert!(
                store
                    .changes_since(&account, kind, 0)
                    .await
                    .unwrap()
                    .is_empty()
            );
        }
        q("DROP TRIGGER reject_source_delete_history")
            .execute(&store.backend)
            .await
            .unwrap();
        assert!(
            store
                .delete_synced_message_if_source(&snapshot)
                .await
                .unwrap()
        );
        assert!(
            !store
                .delete_synced_message_if_source(&snapshot)
                .await
                .unwrap()
        );
        for kind in ["Email", "Mailbox"] {
            assert_eq!(store.current_state(&account, kind).await.unwrap(), 1);
            assert_eq!(
                store.changes_since(&account, kind, 0).await.unwrap().len(),
                1
            );
        }
        assert_eq!(store.count_message_embeddings(&account).await.unwrap(), 0);
        assert_eq!(store.get_mailbox(&mailbox).await.unwrap().total, 0);
        store.close().await;
    }
}
