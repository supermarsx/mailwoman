use super::*;
use crate::{AccountKind, Credentials, NewAccount, ServerKey};

async fn account(store: &Store) -> String {
    let username = format!("{}@cursor.invalid", uuid::Uuid::new_v4());
    store
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
        .unwrap()
}
fn options() -> ChangeLogRetentionOptions {
    ChangeLogRetentionOptions {
        max_run_ms: 2000,
        ..Default::default()
    }
}
async fn age(store: &Store, account: &str) {
    q("UPDATE changes SET at='2000-01-01T00:00:00Z' WHERE account_id=?1")
        .bind(account)
        .execute(&store.backend)
        .await
        .unwrap();
}
async fn position(store: &Store) -> (String, String, i64) {
    let row = q("SELECT account_id,type,updated_at FROM change_log_maintenance_cursor WHERE id=1")
        .fetch_one(&store.backend)
        .await
        .unwrap();
    (
        row.get_string("account_id"),
        row.get_string("type"),
        row.get_i64("updated_at"),
    )
}

#[tokio::test]
async fn durable_cursor_schema_enforces_one_nonnull_singleton() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    for sql in [
        "INSERT INTO change_log_maintenance_cursor VALUES(NULL,'','',0)",
        "INSERT INTO change_log_maintenance_cursor VALUES(2,'','',0)",
        "INSERT INTO change_log_maintenance_cursor VALUES(1,'','',0)",
    ] {
        assert!(q(sql).execute(&store.backend).await.is_err());
    }
    assert_eq!(
        q("SELECT COUNT(*) FROM change_log_maintenance_cursor")
            .fetch_scalar_i64(&store.backend)
            .await
            .unwrap(),
        1
    );
    store.close().await;
}
async fn set_position(store: &Store, account: &str, kind: &str, at: i64) {
    q("UPDATE change_log_maintenance_cursor SET account_id=?1,type=?2,updated_at=?3 WHERE id=1")
        .bind(account)
        .bind(kind)
        .bind(at)
        .execute(&store.backend)
        .await
        .unwrap();
}

#[tokio::test]
async fn background_cursor_survives_plaintext_and_encrypted_reopen_after_noop_group() {
    use crate::encryption::{DbKey, EncryptionConfig, EncryptionMode};
    for encrypted in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("cursor.db");
        let key = ServerKey::generate();
        let config = if encrypted {
            EncryptionConfig::with_key(EncryptionMode::Required, DbKey::generate())
        } else {
            EncryptionConfig::off()
        };
        let store = Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &config)
            .await
            .unwrap();
        let mut accounts = [account(&store).await, account(&store).await];
        accounts.sort();
        for account in &accounts {
            store
                .record_change(account, "Email", "one", "updated")
                .await
                .unwrap();
        }
        age(&store, &accounts[1]).await;
        let first = store.expire_change_log_resumable(options()).await.unwrap();
        assert_eq!(first.cursor.account_id, accounts[0]);
        assert_eq!(
            first.removed_rows, 0,
            "a recent first group still reserves progress"
        );
        let previous_instance = store.instance_id();
        store.close().await;
        drop(store);
        let reopened = Store::open_with_encryption(path.to_str().unwrap(), key, &config)
            .await
            .unwrap();
        assert_ne!(reopened.instance_id(), previous_instance);
        let second = reopened
            .expire_change_log_resumable(options())
            .await
            .unwrap();
        assert_eq!(second.cursor.account_id, accounts[1]);
        assert_eq!(
            second.removed_rows, 1,
            "restart must not return to recent first group"
        );
        assert!(matches!(
            reopened.changes_since(&accounts[1], "Email", 0).await,
            Err(StoreError::ChangeHistoryExpired)
        ));
        assert_eq!(
            reopened.current_state(&accounts[1], "Email").await.unwrap(),
            1
        );
        assert_eq!(
            reopened
                .record_change(&accounts[1], "Email", "new", "updated")
                .await
                .unwrap(),
            2
        );
        assert_eq!(
            q("SELECT COUNT(*) FROM change_log_maintenance_cursor")
                .fetch_scalar_i64(&reopened.backend)
                .await
                .unwrap(),
            1
        );
        reopened.close().await;
        drop(reopened);
    }
}

#[tokio::test]
async fn reserved_but_interrupted_group_keeps_history_and_deleted_boundaries_wrap() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let mut accounts = [account(&store).await, account(&store).await];
    accounts.sort();
    store
        .record_change(&accounts[0], "Email", "email", "updated")
        .await
        .unwrap();
    store
        .record_change(&accounts[0], "Mailbox", "box", "updated")
        .await
        .unwrap();
    store
        .record_change(&accounts[1], "Email", "next", "updated")
        .await
        .unwrap();
    let first = reserve_next_group(&store.backend, 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (&first.account_id, first.kind.as_str()),
        (&accounts[0], "Email")
    );
    // Simulate interruption after reservation, before cleanup. No floor/data
    // transaction ran, so old tokens and their complete history remain valid.
    assert_eq!(
        store
            .changes_since(&accounts[0], "Email", 0)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        q("SELECT COUNT(*) FROM change_log_checkpoint")
            .fetch_scalar_i64(&store.backend)
            .await
            .unwrap(),
        0
    );
    q("DELETE FROM changes WHERE account_id=?1 AND type='Email'")
        .bind(&accounts[0])
        .execute(&store.backend)
        .await
        .unwrap();
    let second = reserve_next_group(&store.backend, 2)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (&second.account_id, second.kind.as_str()),
        (&accounts[0], "Mailbox")
    );
    q("DELETE FROM accounts WHERE id=?1")
        .bind(&accounts[0])
        .execute(&store.backend)
        .await
        .unwrap();
    assert_eq!(
        reserve_next_group(&store.backend, 3)
            .await
            .unwrap()
            .unwrap()
            .account_id,
        accounts[1]
    );
    q("DELETE FROM accounts WHERE id=?1")
        .bind(&accounts[1])
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        reserve_next_group(&store.backend, 4)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(position(&store).await, (String::new(), String::new(), 4));
    let new = account(&store).await;
    store
        .record_change(&new, "Email", "new", "created")
        .await
        .unwrap();
    assert_eq!(
        reserve_next_group(&store.backend, 5)
            .await
            .unwrap()
            .unwrap()
            .account_id,
        new
    );
    store.close().await;
}

#[tokio::test]
async fn explicit_manual_cursor_and_disabled_pass_do_not_override_background_position() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account = account(&store).await;
    for kind in ["Email", "Mailbox"] {
        store
            .record_change(&account, kind, "one", "updated")
            .await
            .unwrap();
    }
    age(&store, &account).await;
    reserve_next_group(&store.backend, 1).await.unwrap();
    let saved = position(&store).await;
    let manual = store
        .expire_change_log(
            options(),
            ChangeLogRetentionCursor {
                account_id: account.clone(),
                kind: "Email".into(),
            },
        )
        .await
        .unwrap();
    assert_eq!(manual.cursor.kind, "Mailbox");
    assert_eq!(manual.removed_rows, 1);
    assert_eq!(position(&store).await, saved);
    let disabled = store
        .expire_change_log_resumable(ChangeLogRetentionOptions {
            enabled: false,
            ..options()
        })
        .await
        .unwrap();
    assert_eq!(disabled.status, "disabled");
    assert_eq!(position(&store).await, saved);
    store.close().await;
}

#[tokio::test]
async fn cursor_survives_database_copy_and_resumes_next_type() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("source.db");
    let key = ServerKey::generate();
    let source = Store::open_with_encryption(
        path.to_str().unwrap(),
        key.clone(),
        &crate::encryption::EncryptionConfig::off(),
    )
    .await
    .unwrap();
    let account = account(&source).await;
    for kind in ["Email", "Mailbox"] {
        source
            .record_change(&account, kind, "one", "updated")
            .await
            .unwrap();
    }
    age(&source, &account).await;
    reserve_next_group(&source.backend, 1).await.unwrap();
    let target = Store::open_in_memory(key).await.unwrap();
    let copied = target
        .migrate_from_sqlite(path.to_str().unwrap())
        .await
        .unwrap();
    assert!(
        copied
            .tables
            .iter()
            .any(|(table, rows)| table == "change_log_maintenance_cursor" && *rows == 1)
    );
    assert_eq!(position(&target).await, position(&source).await);
    let next = target.expire_change_log_resumable(options()).await.unwrap();
    assert_eq!(next.cursor.kind, "Mailbox");
    assert_eq!(next.removed_rows, 1);
    target.close().await;
    source.close().await;
}

#[tokio::test]
async fn postgres_reservations_serialize_and_busy_cleanup_does_not_rewind_progress() {
    let Ok(dsn) = std::env::var("MW_BG_TEST_PG") else {
        eprintln!("SKIP: MW_BG_TEST_PG absent");
        return;
    };
    let store = Store::open_postgres(&dsn, ServerKey::generate())
        .await
        .unwrap();
    let saved = position(&store).await;
    let account = account(&store).await;
    for kind in ["Email", "Mailbox"] {
        store
            .record_change(&account, kind, "one", "updated")
            .await
            .unwrap();
    }
    age(&store, &account).await;
    set_position(&store, &account, "", 0).await;
    let (first, second) = tokio::join!(
        reserve_next_group(&store.backend, 1),
        reserve_next_group(&store.backend, 2)
    );
    let first = first.unwrap().unwrap();
    let second = second.unwrap().unwrap();
    assert_eq!(first.account_id, account);
    assert_eq!(second.account_id, account);
    assert_ne!(first.kind, second.kind);
    assert_eq!(
        std::collections::BTreeSet::from([first.kind, second.kind]),
        std::collections::BTreeSet::from(["Email".into(), "Mailbox".into()])
    );
    set_position(&store, &account, "", 0).await;
    let mut locked = store.backend.begin().await.unwrap();
    q("SELECT id FROM accounts WHERE id=?1 FOR UPDATE")
        .bind(&account)
        .execute_tx(&mut locked)
        .await
        .unwrap();
    let busy = store.expire_change_log_resumable(options()).await.unwrap();
    assert_eq!(busy.status, "busy");
    assert_eq!(busy.removed_rows, 0);
    assert_eq!(
        (busy.cursor.account_id.as_str(), busy.cursor.kind.as_str()),
        (account.as_str(), "Email")
    );
    locked.commit().await.unwrap();
    assert_eq!(
        store
            .changes_since(&account, "Email", 0)
            .await
            .unwrap()
            .len(),
        1
    );
    let next = store.expire_change_log_resumable(options()).await.unwrap();
    assert_eq!(
        (next.cursor.account_id.as_str(), next.cursor.kind.as_str()),
        (account.as_str(), "Mailbox")
    );
    assert_eq!(next.removed_rows, 1);
    assert!(matches!(
        store.changes_since(&account, "Mailbox", 0).await,
        Err(StoreError::ChangeHistoryExpired)
    ));
    assert_eq!(
        store
            .record_change(&account, "Mailbox", "new", "updated")
            .await
            .unwrap(),
        2
    );
    q("DELETE FROM accounts WHERE id=?1")
        .bind(&account)
        .execute(&store.backend)
        .await
        .unwrap();
    set_position(&store, &saved.0, &saved.1, saved.2).await;
    store.close().await;
}
