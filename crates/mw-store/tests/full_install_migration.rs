//! Isolated live PostgreSQL gate. Never uses the database named in the DSN as
//! the destination: creates fresh uniquely named databases in a disposable server.
use mw_store::{ServerKey, Store};
use sqlx::Row;

#[tokio::test]
#[ignore = "requires MW_STACK_TEST_PG pointing at a disposable PostgreSQL server"]
async fn complete_installation_preserves_configuration_and_verifies_before_commit() {
    let base = std::env::var("MW_STACK_TEST_PG").expect("MW_STACK_TEST_PG is required");
    let root = sqlx::PgPool::connect(&base).await.unwrap();
    let id = uuid::Uuid::new_v4().simple().to_string();
    let database = format!("mw_stack_{id}");
    let reject_database = format!("mw_stack_reject_{id}");
    for name in [&database, &reject_database] {
        sqlx::query(&format!("CREATE DATABASE {name}"))
            .execute(&root)
            .await
            .unwrap();
    }
    let target = |name: &str| format!("{}/{}", base.rsplit_once('/').unwrap().0, name);
    let key = || ServerKey::from_bytes(&[71; 32]).unwrap();
    let path = std::env::temp_dir().join(format!("mw-complete-move-{id}.db"));
    let source = Store::open(path.to_str().unwrap(), key()).await.unwrap();
    source
        .set_setting("complete-move-fixture", "kept verbatim")
        .await
        .unwrap();
    let sqlite = sqlx::SqlitePool::connect_with(mw_store::encryption::raw_options(
        path.to_str().unwrap(),
        false,
    ))
    .await
    .unwrap();
    sqlx::query("INSERT INTO admin_users(id,username,password_hash,created_at) VALUES('admin1','preserved-admin','opaque-hash','2026-11-15T00:00:00Z')")
        .execute(&sqlite).await.unwrap();
    let opaque = vec![0_u8, 255, 17, 0, 128];
    sqlx::query("INSERT INTO accounts(id,kind,host,port,tls,username,sealed_creds) VALUES('account1','imap','mail.example.test',993,'tls','user@example.test',?1)")
        .bind(&opaque).execute(&sqlite).await.unwrap();
    sqlx::query("INSERT INTO crypto_keys(id,account_id,kind,created_at,public_key,encrypted_private_backup) VALUES('key1','account1','pgp','2026-11-15T00:00:00Z','Unicode Δ and line breaks',?1)")
        .bind(&opaque).execute(&sqlite).await.unwrap();
    sqlx::query("INSERT INTO domains(name,upstream_json) VALUES('example.test','{}')")
        .execute(&sqlite)
        .await
        .unwrap();
    sqlx::query("UPDATE settings_revision SET n=42")
        .execute(&sqlite)
        .await
        .unwrap();
    let destination = Store::open(&target(&database), key()).await.unwrap();
    let report = destination
        .migrate_complete_from_sqlite(path.to_str().unwrap())
        .await
        .unwrap();
    let expected: i64 = sqlx::query_scalar("SELECT count(*) FROM sqlite_master WHERE type='table' AND substr(name,1,7) <> 'sqlite_' AND name <> '_sqlx_migrations'")
        .fetch_one(&sqlite).await.unwrap();
    assert_eq!(
        report.tables.len() as i64,
        expected,
        "every application table accounted for"
    );
    assert_eq!(
        destination
            .get_setting("complete-move-fixture")
            .await
            .unwrap()
            .as_deref(),
        Some("kept verbatim")
    );
    let pg = sqlx::PgPool::connect(&target(&database)).await.unwrap();
    let row = sqlx::query("SELECT username,password_hash FROM admin_users WHERE id='admin1'")
        .fetch_one(&pg)
        .await
        .unwrap();
    assert_eq!(row.get::<String, _>("username"), "preserved-admin");
    assert_eq!(row.get::<String, _>("password_hash"), "opaque-hash");
    let copied_blob: Vec<u8> =
        sqlx::query_scalar("SELECT encrypted_private_backup FROM crypto_keys WHERE id='key1'")
            .fetch_one(&pg)
            .await
            .unwrap();
    assert_eq!(copied_blob, opaque);
    let expires: Option<String> =
        sqlx::query_scalar("SELECT expires_at FROM crypto_keys WHERE id='key1'")
            .fetch_one(&pg)
            .await
            .unwrap();
    assert!(expires.is_none());
    let domain: String =
        sqlx::query_scalar("SELECT upstream_json FROM domains WHERE name='example.test'")
            .fetch_one(&pg)
            .await
            .unwrap();
    assert_eq!(domain, "{}");
    let revision: i64 = sqlx::query_scalar("SELECT n FROM settings_revision")
        .fetch_one(&pg)
        .await
        .unwrap();
    assert_eq!(revision, 42);
    assert!(
        destination
            .migrate_complete_from_sqlite(path.to_str().unwrap())
            .await
            .is_err(),
        "nonempty destination refused"
    );
    // A late column mismatch must roll back the earlier admin table inserts.
    sqlx::query("ALTER TABLE settings ADD COLUMN unexpected_column TEXT")
        .execute(&sqlite)
        .await
        .unwrap();
    let rejected = Store::open(&target(&reject_database), key()).await.unwrap();
    assert!(
        rejected
            .migrate_complete_from_sqlite(path.to_str().unwrap())
            .await
            .is_err()
    );
    let reject_pg = sqlx::PgPool::connect(&target(&reject_database))
        .await
        .unwrap();
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM admin_users")
        .fetch_one(&reject_pg)
        .await
        .unwrap();
    assert_eq!(
        count, 0,
        "transaction rolled back configuration copied before the mismatch"
    );
    let original: i64 = sqlx::query_scalar("SELECT count(*) FROM admin_users")
        .fetch_one(&sqlite)
        .await
        .unwrap();
    assert_eq!(original, 1, "source is preserved");
    sqlite.close().await;
    pg.close().await;
    reject_pg.close().await;
    drop(source);
    drop(destination);
    drop(rejected);
    root.close().await;
    let _ = std::fs::remove_file(&path);
}
