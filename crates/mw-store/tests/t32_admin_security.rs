//! Security invariants run on SQLite and, when configured, an isolated Postgres schema.
use mw_store::{AdminAccountRow, AdminRoleGrant, ServerKey, Store};
fn row(id: &str) -> AdminAccountRow {
    AdminAccountRow {
        id: id.into(),
        username: id.into(),
        display_name: id.into(),
        password_hash: Some("test-hash".into()),
        disabled: false,
        source: "db".into(),
        created_at: 1,
        updated_at: 1,
        last_login_at: None,
    }
}
async fn exercise(store: Store) {
    store
        .ensure_builtin_admin_roles(&[("super_admin", "Super admin")])
        .await
        .unwrap();
    let grants = [AdminRoleGrant {
        role_id: "super_admin".into(),
        domains: None,
    }];
    let a = row("first");
    let b = row("second");
    let (a_created, b_created) = tokio::join!(
        store.insert_admin_account_with_roles(&a, &grants, true),
        store.insert_admin_account_with_roles(&b, &grants, true)
    );
    assert_ne!(
        a_created.unwrap(),
        b_created.unwrap(),
        "bootstrap is single-winner"
    );
    assert_eq!(store.count_admin_accounts().await.unwrap(), 1);
    let first = store.list_admin_accounts().await.unwrap()[0].clone();
    assert_eq!(store.admin_account_roles(&first.id).await.unwrap().len(), 1);
    assert!(!store.delete_admin_role("super_admin").await.unwrap());
    assert_eq!(
        store.admin_account_roles(&first.id).await.unwrap().len(),
        1,
        "refusing builtin delete preserves assignments"
    );
    assert!(
        !store
            .set_admin_overrides_guarded(&first.id, &[("admins.manage".into(), "deny".into())])
            .await
            .unwrap()
    );
    let second = row("third");
    store
        .insert_admin_account_with_roles(&second, &grants, false)
        .await
        .unwrap();
    let (a, b) = tokio::join!(
        store.change_admin_account_guarded(&first.id, None, false, Some(&[]), false, 2),
        store.change_admin_account_guarded(&second.id, None, false, Some(&[]), false, 2)
    );
    assert_ne!(
        a.unwrap(),
        b.unwrap(),
        "concurrent demotions retain one super admin"
    );
    let last = if store
        .admin_account_roles(&first.id)
        .await
        .unwrap()
        .is_empty()
    {
        second
    } else {
        first
    };
    assert!(
        !store
            .change_admin_account_guarded(&last.id, Some("disabled"), true, None, false, 3)
            .await
            .unwrap()
    );
    assert!(
        !store
            .change_admin_account_guarded(&last.id, None, false, None, true, 3)
            .await
            .unwrap()
    );
    // The explicit deployment opt-in is read inside the same transaction as
    // the destructive mutation; resetting it restores protection immediately.
    let loss_policy = mw_store::settings_policy::SettingPolicyRow {
        key: "admin.allowLastSuperadminLoss".into(),
        global_value: Some("true".into()),
        allow_user_override: false,
        global_locked: true,
        updated_by: "test".into(),
        updated_at: 4,
    };
    store.put_setting_policy(&loss_policy).await.unwrap();
    assert!(
        store
            .set_admin_overrides_guarded(&last.id, &[("admins.manage".into(), "deny".into())])
            .await
            .unwrap(),
        "explicit opt-in permits losing the final full-superadmin grant"
    );
    store
        .set_admin_overrides_guarded(&last.id, &[])
        .await
        .unwrap();
    store.delete_setting_policy(&loss_policy.key).await.unwrap();
    assert!(
        !store
            .change_admin_account_guarded(&last.id, None, true, None, false, 5)
            .await
            .unwrap(),
        "deleting the opt-in restores the safe default"
    );
    let invalid = row("rollback");
    assert!(
        store
            .insert_admin_account_with_roles(
                &invalid,
                &[AdminRoleGrant {
                    role_id: "missing".into(),
                    domains: None
                }],
                false
            )
            .await
            .is_err()
    );
    assert!(
        store.get_admin_account("rollback").await.unwrap().is_none(),
        "failed grants roll back the account"
    );
    store
        .put_admin_totp(&last.id, b"totp-secret", 3)
        .await
        .unwrap();
    assert_eq!(
        store.get_admin_totp(&last.id).await.unwrap().unwrap(),
        b"totp-secret"
    );
    let (a, b) = tokio::join!(
        store.advance_admin_totp_step(&last.id, 10),
        store.advance_admin_totp_step(&last.id, 10)
    );
    assert_ne!(a.unwrap(), b.unwrap(), "TOTP replay CAS admits one request");
    store
        .put_admin_session_with_timeouts("token", &last.username, 1000, 300, 3600)
        .await
        .unwrap();
    assert_eq!(
        store
            .get_admin_session_with_idle("token", 1299, 300)
            .await
            .unwrap(),
        Some(last.username.clone())
    );
    assert!(
        store
            .get_admin_session_with_idle("token", 1599, 300)
            .await
            .unwrap()
            .is_none(),
        "idle expiry cannot be refreshed"
    );
    store
        .put_admin_session_with_timeouts("absolute", &last.username, 1000, 7200, 3600)
        .await
        .unwrap();
    assert!(
        store
            .get_admin_session_with_idle("absolute", 4600, 7200)
            .await
            .unwrap()
            .is_none()
    );
    store
        .put_admin_session_with_timeouts("revoked", &last.username, 1000, 300, 3600)
        .await
        .unwrap();
    store.delete_admin_session("revoked").await.unwrap();
    assert!(
        store
            .get_admin_session_with_idle("revoked", 1001, 300)
            .await
            .unwrap()
            .is_none()
    );
    let mut passwordless = row("passwordless");
    passwordless.password_hash = None;
    store
        .insert_admin_account_with_roles(&passwordless, &grants, false)
        .await
        .unwrap();
    assert!(
        !store
            .change_admin_account_guarded(&last.id, None, false, Some(&[]), false, 6)
            .await
            .unwrap(),
        "an unusable superadmin is not a replacement"
    );
    for id in ["key-one", "key-two"] {
        let key = mw_store::AdminPasskey {
            credential: mw_store::WebauthnCredentialRow {
                credential_id: id.into(),
                account_id: passwordless.id.clone(),
                cose_public_key: vec![1, 2, 3],
                sign_count: 0,
                transports: "internal".into(),
                label: id.into(),
                created_at: String::new(),
            },
            state: mw_store::WebauthnState {
                backup_eligible: Some(false),
                backup_state: false,
                uv_initialized: true,
                revision: 0,
            },
        };
        assert!(store.register_admin_passkey(&key, &[]).await.unwrap().0);
    }
    assert!(
        store
            .change_admin_account_guarded(&last.id, None, false, Some(&[]), false, 6)
            .await
            .unwrap()
    );
    let (one, two) = tokio::join!(
        store.delete_admin_passkey(&passwordless.id, "key-one"),
        store.delete_admin_passkey(&passwordless.id, "key-two")
    );
    let outcomes = [one.unwrap(), two.unwrap()];
    assert_eq!(outcomes.iter().filter(|r| **r == Some(true)).count(), 1);
    assert_eq!(
        outcomes.iter().filter(|r| r.is_none()).count(),
        1,
        "concurrent removals retain a usable credential"
    );
    assert!(!store.revoke_admin_passkeys(&passwordless.id).await.unwrap());
    store.put_setting_policy(&loss_policy).await.unwrap();
    assert!(
        store.revoke_admin_passkeys(&passwordless.id).await.unwrap(),
        "explicit opt-in permits credential loss"
    );
}
#[tokio::test]
async fn sqlite_admin_security() {
    exercise(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}

#[tokio::test]
async fn postgres_admin_security() {
    let Ok(dsn) = std::env::var("MW_ADMIN_TEST_PG_DSN") else {
        eprintln!("SKIP Postgres: MW_ADMIN_TEST_PG_DSN unset");
        return;
    };
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    let schema = format!("admin_security_{}", rand::random::<u64>());
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&pool)
        .await
        .unwrap();
    let sep = if dsn.contains('?') { '&' } else { '?' };
    let scoped = format!("{dsn}{sep}options=-c%20search_path%3D{schema}");
    let result = tokio::spawn(async move {
        exercise(
            Store::open_postgres(&scoped, ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    })
    .await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&pool)
        .await
        .unwrap();
    result.unwrap();
}
