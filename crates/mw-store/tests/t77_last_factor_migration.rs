//! t77 — migration `0074_last_factor_block` over a populated database.
//!
//! t39-e6 stored `twofa.lastFactorRemoval = "requireRecoveryCodes"` (global
//! policy row and domain/admin-user values). t77 folds that value into `block`.
//! The database is built with the migrator cut off before 0074, rows holding the
//! old value (and other values that must stay as they are) are written with raw
//! SQL, then the file is opened the way the server does and the full migrator
//! brings it forward.
//!
//! Run: cargo test -p mw-store --test t77_last_factor_migration -- --test-threads=1

use mw_store::{ServerKey, Store};
use sqlx::Row;

#[path = "../src/test_db.rs"]
mod test_db;

fn cut(mut m: sqlx::migrate::Migrator, before: i64) -> sqlx::migrate::Migrator {
    m.migrations.to_mut().retain(|mig| mig.version < before);
    assert!(m.migrations.iter().any(|mig| mig.version == before - 1));
    m
}

const SEED: &[&str] = &[
    "INSERT INTO setting_policy (key, global_value, allow_user_override, global_locked, updated_by, updated_at)
     VALUES ('twofa.lastFactorRemoval', '\"requireRecoveryCodes\"', 0, 1, 'seed', 1)",
    "INSERT INTO setting_policy (key, global_value, allow_user_override, global_locked, updated_by, updated_at)
     VALUES ('twofa.passkeyLegacy', '\"requireRecoveryCodes\"', 0, 0, 'seed', 1)",
    "INSERT INTO setting_value (scope_kind, scope_id, key, value, locked, updated_by, updated_at)
     VALUES ('domain', 'a.example', 'twofa.lastFactorRemoval', '\"requireRecoveryCodes\"', 1, 'seed', 1)",
    "INSERT INTO setting_value (scope_kind, scope_id, key, value, locked, updated_by, updated_at)
     VALUES ('domain', 'b.example', 'twofa.lastFactorRemoval', '\"allow\"', 0, 'seed', 1)",
    "INSERT INTO setting_value (scope_kind, scope_id, key, value, locked, updated_by, updated_at)
     VALUES ('admin_user', 'u1', 'twofa.lastFactorRemoval', '\"requireRecoveryCodes\"', 0, 'seed', 1)",
];

#[tokio::test]
async fn require_recovery_codes_rows_become_block() {
    let path = test_db::unique_file_path("mw-store-t77-upgrade", "u.sqlite");
    let path_str = path.to_string_lossy().to_string();
    let url = format!("sqlite://{path_str}?mode=rwc");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    cut(sqlx::migrate!("./migrations"), 74)
        .run(&pool)
        .await
        .unwrap();
    for stmt in SEED {
        sqlx::query(stmt).execute(&pool).await.unwrap();
    }
    pool.close().await;

    drop(
        Store::open(&path_str, ServerKey::from_bytes(&[7u8; 32]).unwrap())
            .await
            .expect("0074 applies over a populated database"),
    );

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    let policy: Vec<(String, String)> = sqlx::query(
        "SELECT key, global_value FROM setting_policy WHERE updated_by = 'seed' ORDER BY key",
    )
    .fetch_all(&pool)
    .await
    .unwrap()
    .into_iter()
    .map(|r| (r.get("key"), r.get("global_value")))
    .collect();
    assert_eq!(
        policy,
        vec![
            ("twofa.lastFactorRemoval".into(), "\"block\"".into()),
            // Another key's value is not touched, whatever it holds.
            (
                "twofa.passkeyLegacy".into(),
                "\"requireRecoveryCodes\"".into()
            ),
        ]
    );
    let values: Vec<(String, String, String, i64)> = sqlx::query(
        "SELECT scope_kind, scope_id, value, locked FROM setting_value
         WHERE key = 'twofa.lastFactorRemoval' ORDER BY scope_kind, scope_id",
    )
    .fetch_all(&pool)
    .await
    .unwrap()
    .into_iter()
    .map(|r| {
        (
            r.get("scope_kind"),
            r.get("scope_id"),
            r.get("value"),
            r.get("locked"),
        )
    })
    .collect();
    assert_eq!(
        values,
        vec![
            ("admin_user".into(), "u1".into(), "\"block\"".into(), 0),
            ("domain".into(), "a.example".into(), "\"block\"".into(), 1),
            ("domain".into(), "b.example".into(), "\"allow\"".into(), 0),
        ]
    );
    pool.close().await;
    let _ = std::fs::remove_file(&path_str);
    let _ = std::fs::remove_file(format!("{path_str}-wal"));
    let _ = std::fs::remove_file(format!("{path_str}-shm"));
}
