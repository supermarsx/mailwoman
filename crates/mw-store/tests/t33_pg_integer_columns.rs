//! t33: integer results computed in SQL decode as `i64` on Postgres too.
//!
//! On Postgres a `CASE ... THEN 0 ELSE 1 END` is an `INT4`, and `Row::get_i64`
//! panics on it (sqlx refuses `INT4` for `i64`); SQLite has one integer type and
//! never shows it. The admin dashboard's domain list (`configured`) and the
//! request-protection prune check (`n`) both did this, so on a Postgres
//! deployment the admin dashboard answered 500 (a handler panic) and the
//! periodic prune panicked. Found by the `v6` Playwright project (proxy mode on
//! postgres:16, t33-e10). The Postgres leg runs when `DATABASE_URL_PG` or
//! `MW_TEST_PG` is set (the `store-dual-backend` CI job sets both).

use mw_store::{ServerKey, Store};

#[allow(dead_code)]
#[path = "../../mw-server/tests/common/gate.rs"]
mod gate;

#[allow(dead_code)]
#[path = "../src/test_db.rs"]
mod test_db;

fn pg_dsn() -> Option<String> {
    std::env::var("DATABASE_URL_PG")
        .ok()
        .or_else(|| std::env::var("MW_TEST_PG").ok())
        .filter(|s| !s.trim().is_empty())
}

async fn exercise(store: &Store) {
    let now = 1_800_000_000;
    let snapshot = store
        .dashboard_snapshot(now, true)
        .await
        .expect("dashboard snapshot");
    let domain = snapshot
        .domains
        .iter()
        .find(|d| d.name == "t33-int.test")
        .expect("the seeded domain is listed");
    assert!(domain.configured, "an upstream is configured for it");
    store
        .prune_request_protection(now)
        .await
        .expect("request-protection prune");
}

#[tokio::test]
async fn computed_integer_columns_decode_on_both_backends() {
    let key = || ServerKey::from_bytes(&[9u8; 32]).unwrap();

    let path = test_db::unique_file_path("mw-store-t33-int", "store.sqlite");
    let path = path.to_str().unwrap().to_string();
    let sqlite = Store::open(&path, key()).await.unwrap();
    let raw = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{path}"))
        .await
        .unwrap();
    sqlx::query("INSERT INTO domains(name,upstream_json) VALUES('t33-int.test','{\"jmap\":\"https://jmap.t33-int.test\"}')")
        .execute(&raw)
        .await
        .unwrap();
    raw.close().await;
    exercise(&sqlite).await;

    let Some(dsn) = pg_dsn() else {
        gate::skip(
            "[mw-store] t33 integer columns: DATABASE_URL_PG and MW_TEST_PG unset — the \
             Postgres decode is not driven.",
        );
        return;
    };
    let pg = Store::open_postgres(&dsn, key()).await.unwrap();
    let raw = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    sqlx::query("DELETE FROM domains WHERE name='t33-int.test'")
        .execute(&raw)
        .await
        .unwrap();
    sqlx::query("INSERT INTO domains(name,upstream_json) VALUES('t33-int.test','{\"jmap\":\"https://jmap.t33-int.test\"}')")
        .execute(&raw)
        .await
        .unwrap();
    exercise(&pg).await;
    sqlx::query("DELETE FROM domains WHERE name='t33-int.test'")
        .execute(&raw)
        .await
        .unwrap();
}
