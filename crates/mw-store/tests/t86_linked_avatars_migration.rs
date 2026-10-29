//! t86 — migration `0075_linked_avatars` over a populated database.
//!
//! A `linked_layout` row written before t86 (order and nicknames only) must
//! come through with its values and an empty avatar map, and the store then
//! reads and writes avatars beside them. Postgres runs when `MW_TEST_PG` names a
//! server the test may create a database on.
//!
//! Run: cargo test -p mw-store --test t86_linked_avatars_migration -- --test-threads=1

use mw_store::{ServerKey, Store};
use sqlx::Row;

#[path = "../src/test_db.rs"]
mod test_db;

fn cut(mut m: sqlx::migrate::Migrator, before: i64) -> sqlx::migrate::Migrator {
    m.migrations.to_mut().retain(|mig| mig.version < before);
    assert!(m.migrations.iter().any(|mig| mig.version == before - 1));
    m
}

const ROW: &str = "INSERT INTO linked_layout(owner_principal_id,order_json,nicknames_json,revision,updated_at) VALUES('owner-1','[\"main\"]','{\"main\":\"Home\"}','rev-1',1)";

#[tokio::test]
async fn sqlite_old_layout_rows_get_an_empty_avatar_map() {
    let path = test_db::unique_file_path("mw-store-t86-upgrade", "u.sqlite");
    let path_str = path.to_string_lossy().to_string();
    let url = format!("sqlite://{path_str}?mode=rwc");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    cut(sqlx::migrate!("./migrations"), 75)
        .run(&pool)
        .await
        .unwrap();
    // The owner row is not the subject here: write the layout row on its own.
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(ROW).execute(&pool).await.unwrap();
    pool.close().await;

    drop(
        Store::open(&path_str, ServerKey::from_bytes(&[7u8; 32]).unwrap())
            .await
            .expect("0075 applies over a populated database"),
    );

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    let row = sqlx::query("SELECT order_json,nicknames_json,revision,avatars_json FROM linked_layout WHERE owner_principal_id='owner-1'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.get::<String, _>("order_json"), "[\"main\"]");
    assert_eq!(
        row.get::<String, _>("nicknames_json"),
        "{\"main\":\"Home\"}"
    );
    assert_eq!(row.get::<String, _>("revision"), "rev-1");
    assert_eq!(row.get::<String, _>("avatars_json"), "{}");
    pool.close().await;
}

#[tokio::test]
async fn postgres_old_layout_rows_get_an_empty_avatar_map() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        return;
    };
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    let name = format!("mw_t86_{:032x}", rand::random::<u128>());
    sqlx::query(&format!("CREATE DATABASE {name}"))
        .execute(&admin)
        .await
        .unwrap();
    let mut url = url::Url::parse(&dsn).unwrap();
    url.set_path(&format!("/{name}"));
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(url.as_str())
        .await
        .unwrap();
    cut(sqlx::migrate!("./migrations_pg"), 75)
        .run(&pool)
        .await
        .unwrap();
    // Skip the owner foreign key for this one row (superuser on a test server).
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET CONSTRAINTS ALL DEFERRED")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("SET LOCAL session_replication_role = replica")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query(ROW).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    pool.close().await;

    drop(
        Store::open(url.as_str(), ServerKey::from_bytes(&[7u8; 32]).unwrap())
            .await
            .expect("0075 applies over a populated Postgres database"),
    );

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(url.as_str())
        .await
        .unwrap();
    let row = sqlx::query("SELECT order_json,nicknames_json,revision,avatars_json FROM linked_layout WHERE owner_principal_id='owner-1'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        row.get::<String, _>("nicknames_json"),
        "{\"main\":\"Home\"}"
    );
    assert_eq!(row.get::<String, _>("revision"), "rev-1");
    assert_eq!(row.get::<String, _>("avatars_json"), "{}");
    pool.close().await;
    sqlx::query(&format!("DROP DATABASE {name} WITH (FORCE)"))
        .execute(&admin)
        .await
        .unwrap();
}
