use super::*;
use crate::{AccountKind, Credentials, NewAccount, ServerKey};

async fn fixture(store: &Store) -> (String, String) {
    let username = format!("{}@ingest.invalid", crate::seal::random_token());
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
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        })
        .await
        .unwrap();
    (account, mailbox)
}

async fn contract(store: &Store) {
    let (account, mailbox) = fixture(store).await;
    let mut message = MessageUpsert {
        account_id: &account,
        mailbox_id: &mailbox,
        uid: 1,
        uidvalidity: 1,
        message_id: Some("<one@ingest.invalid>"),
        thread_id: None,
        internaldate: Some("2024-01-01T00:00:00Z"),
        size: 3,
        flags_json: r#"[{"Keyword":"label"},"Flagged"]"#,
        envelope: Some(br#"{"subject":"one","bodyValues":{"1":{"value":"abc"}}}"#),
        blob_ref: None,
    };
    let first = store
        .ingest_message_with_body(&message, b"abc")
        .await
        .unwrap();
    assert!(!first.existed && first.changed);
    assert_eq!(store.current_state(&account, "Email").await.unwrap(), 1);
    assert_eq!(store.current_state(&account, "Mailbox").await.unwrap(), 1);
    let revision = store
        .get_message(&first.stable_id)
        .await
        .unwrap()
        .cache_revision;
    for _ in 0..2 {
        let replay = store
            .ingest_message_with_body(&message, b"abc")
            .await
            .unwrap();
        assert!(replay.existed && !replay.changed);
        assert_eq!(replay.stable_id, first.stable_id);
    }
    assert_eq!(
        store
            .get_message(&first.stable_id)
            .await
            .unwrap()
            .cache_revision,
        revision
    );
    assert_eq!(
        store
            .changes_since(&account, "Email", 0)
            .await
            .unwrap()
            .len(),
        1
    );
    message.flags_json = r#"["Flagged",{"Keyword":"label"},"Flagged"]"#;
    assert!(
        !store
            .ingest_message_with_body(&message, b"abc")
            .await
            .unwrap()
            .changed
    );
    message.envelope = Some(br#"{"bodyValues":{"1":{"value":"abc"}},"subject":"one"}"#);
    assert!(
        !store
            .ingest_message_with_body(&message, b"abc")
            .await
            .unwrap()
            .changed
    );
    message.flags_json = r#"["Seen"]"#;
    assert!(
        store
            .ingest_message_with_body(&message, b"abc")
            .await
            .unwrap()
            .changed
    );
    assert_eq!(store.current_state(&account, "Email").await.unwrap(), 2);
    message.envelope =
        Some(br#"{"subject":"changed projection","bodyValues":{"1":{"value":"abc"}}}"#);
    assert!(
        store
            .ingest_message_with_body(&message, b"abc")
            .await
            .unwrap()
            .changed
    );
    assert_eq!(store.current_state(&account, "Email").await.unwrap(), 3);
    assert!(
        store
            .ingest_message_with_body(&message, b"cba")
            .await
            .unwrap()
            .changed
    );
    assert_eq!(store.current_state(&account, "Email").await.unwrap(), 4);
    // Same bytes/flags at changed coordinates must still invalidate readers.
    message.uidvalidity = 2;
    assert!(
        store
            .ingest_message_with_body(&message, b"cba")
            .await
            .unwrap()
            .changed
    );
    assert_eq!(store.current_state(&account, "Email").await.unwrap(), 5);
    assert_eq!(store.current_state(&account, "Mailbox").await.unwrap(), 5);
    assert_eq!(
        store.list_message_ids(&mailbox, 10, 0).await.unwrap().len(),
        1
    );
    // The cached-row counter follows the rows: the re-keyed identity is still
    // one cached message (t50hp-e26; the server's figure is upstream_total).
    assert_eq!(store.get_mailbox(&mailbox).await.unwrap().total, 1);
    store
        .register_refetchable_cached_body(&account, &first.stable_id)
        .await
        .unwrap();
    q("UPDATE message_body_cache_lease SET last_used_at=1 WHERE stable_id=?1")
        .bind(&first.stable_id)
        .execute(&store.backend)
        .await
        .unwrap();
    let expired = store
        .expire_cached_bodies(
            crate::body_cache_expiry::CachedBodyExpiryOptions {
                max_run_ms: 2000,
                ..Default::default()
            },
            Default::default(),
            vec![account.clone()],
        )
        .await
        .unwrap();
    assert_eq!(expired.detached_messages, 1, "{}", expired.status);
    let detached = store.get_message(&first.stable_id).await.unwrap();
    assert!(detached.blob_ref.is_none());
    assert!(
        !store
            .ingest_message_with_body(&message, b"cba")
            .await
            .unwrap()
            .changed
    );
    assert!(
        store
            .get_message(&first.stable_id)
            .await
            .unwrap()
            .blob_ref
            .is_none()
    );
    assert_eq!(store.current_state(&account, "Email").await.unwrap(), 5);
    assert!(
        !String::from_utf8(store.get_envelope(&first.stable_id).await.unwrap().unwrap())
            .unwrap()
            .contains("bodyValues")
    );
    assert_eq!(
        store
            .cached_body_expiry_stats()
            .await
            .unwrap()
            .oldest_use_at,
        Some(1)
    );
    q("DELETE FROM accounts WHERE id=?1")
        .bind(&account)
        .execute(&store.backend)
        .await
        .unwrap();
}

#[tokio::test]
async fn sqlite_identical_ingest_keeps_history_and_revision_stable() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    contract(&store).await;
    store.close().await;
}

#[tokio::test]
async fn postgres_identical_ingest_keeps_history_and_revision_stable() {
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

#[tokio::test]
async fn failed_history_insert_rolls_back_message_body_and_retries_created_once() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (account, mailbox) = fixture(&store).await;
    let mut message = MessageUpsert {
        account_id: &account,
        mailbox_id: &mailbox,
        uid: 1,
        uidvalidity: 1,
        message_id: None,
        thread_id: None,
        internaldate: None,
        size: 3,
        flags_json: "[]",
        envelope: Some(b"{}"),
        blob_ref: None,
    };
    let fail = "CREATE TRIGGER fail_ingest_history BEFORE INSERT ON changes WHEN NEW.type='Mailbox' BEGIN SELECT RAISE(ABORT,'fixture history failure'); END";
    q(fail).execute(&store.backend).await.unwrap();
    assert!(
        store
            .ingest_message_with_body(&message, b"abc")
            .await
            .is_err()
    );
    assert_eq!(store.get_mailbox(&mailbox).await.unwrap().total, 0);
    assert_eq!(
        q("SELECT COUNT(*) FROM bodies")
            .fetch_scalar_i64(&store.backend)
            .await
            .unwrap(),
        0
    );
    assert_eq!(store.current_state(&account, "Email").await.unwrap(), 0);
    q("DROP TRIGGER fail_ingest_history")
        .execute(&store.backend)
        .await
        .unwrap();
    let first = store
        .ingest_message_with_body(&message, b"abc")
        .await
        .unwrap();
    assert!(!first.existed && first.changed);
    assert_eq!(store.current_state(&account, "Email").await.unwrap(), 1);
    q(fail).execute(&store.backend).await.unwrap();
    message.flags_json = r#"["Seen"]"#;
    assert!(
        store
            .ingest_message_with_body(&message, b"abc")
            .await
            .is_err()
    );
    assert_eq!(
        store
            .get_message(&first.stable_id)
            .await
            .unwrap()
            .flags_json,
        "[]"
    );
    assert_eq!(store.current_state(&account, "Email").await.unwrap(), 1);
    q("DROP TRIGGER fail_ingest_history")
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        store
            .ingest_message_with_body(&message, b"abc")
            .await
            .unwrap()
            .changed
    );
    assert!(
        !store
            .ingest_message_with_body(&message, b"abc")
            .await
            .unwrap()
            .changed
    );
    assert_eq!(store.current_state(&account, "Email").await.unwrap(), 2);
    // A damaged old cache projection must not prevent fresh authoritative MIME
    // from repairing it; failed old-ciphertext comparison is not an ingest error.
    q("UPDATE messages SET envelope_json=?1 WHERE stable_id=?2")
        .bind(vec![0u8; 12])
        .bind(&first.stable_id)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        store
            .ingest_message_with_body(&message, b"abc")
            .await
            .unwrap()
            .changed
    );
    assert_eq!(
        store.get_envelope(&first.stable_id).await.unwrap().unwrap(),
        b"{}"
    );
    assert_eq!(store.current_state(&account, "Email").await.unwrap(), 3);
    store.close().await;
}
