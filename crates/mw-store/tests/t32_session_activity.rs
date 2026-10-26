use mw_store::{Credentials, ServerKey, Store, identity_principal::VerifiedPrimary};
#[path = "../src/test_db.rs"]
mod test_db;
async fn exercise(store: Store, key: ServerKey) {
    let source_path = test_db::unique_file_path("activity-copy", "source.sqlite");
    let source = Store::open(source_path.to_str().unwrap(), key)
        .await
        .unwrap();
    let old = source
        .create_session(
            "alice",
            "alice@example.test",
            "engine",
            "engine",
            &Credentials {
                username: "alice".into(),
                password: "fixture".into(),
            },
        )
        .await
        .unwrap();
    source
        .set_session_authentication(&old, "password")
        .await
        .unwrap();
    let principal = source
        .bind_verified_primary(
            &old,
            VerifiedPrimary {
                account_id: "alice",
                kind: "mailbox",
                authority: "fixture",
                subject: "alice",
            },
            123,
        )
        .await
        .unwrap()
        .unwrap();
    source
        .record_session_activity(&old, "2026-01-02T00:00:00Z")
        .await
        .unwrap();
    let clocks = source.session_activity_clock(&old).await.unwrap().unwrap();
    store
        .migrate_from_sqlite(source_path.to_str().unwrap())
        .await
        .unwrap();
    assert_eq!(
        store.session_activity_clock(&old).await.unwrap().unwrap(),
        clocks
    );
    store.touch_session(&old).await.unwrap();
    assert_eq!(
        store.session_activity_clock(&old).await.unwrap().unwrap(),
        clocks
    );
    let new = store
        .create_session(
            "alice",
            "alice@example.test",
            "engine",
            "engine",
            &Credentials {
                username: "alice".into(),
                password: "fixture".into(),
            },
        )
        .await
        .unwrap();
    assert!(store.rotate_session_security(&old, &new).await.unwrap());
    assert_eq!(
        store.session_activity_clock(&new).await.unwrap().unwrap(),
        clocks
    );
    assert_eq!(
        store.primary_principal(&new).await.unwrap(),
        Some(principal)
    );
    assert!(
        !store
            .record_session_activity(&old, "2026-01-03T00:00:00Z")
            .await
            .unwrap()
    );
    store.delete_session(&new).await.unwrap();
    assert!(
        !store
            .record_session_activity(&new, "2026-01-03T00:00:00Z")
            .await
            .unwrap()
    );
}
#[tokio::test]
async fn activity_copy_rotation_and_principal_clock_sqlite() {
    let key = ServerKey::generate();
    exercise(Store::open_in_memory(key.clone()).await.unwrap(), key).await;
}
#[tokio::test]
async fn activity_copy_rotation_and_principal_clock_postgres() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        return;
    };
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    let name = format!("mw_activity_{:032x}", rand::random::<u128>());
    sqlx::query(&format!("CREATE DATABASE {name}"))
        .execute(&admin)
        .await
        .unwrap();
    let mut url = url::Url::parse(&dsn).unwrap();
    url.set_path(&format!("/{name}"));
    let key = ServerKey::generate();
    exercise(Store::open(url.as_str(), key.clone()).await.unwrap(), key).await;
    sqlx::query(&format!("DROP DATABASE {name} WITH (FORCE)"))
        .execute(&admin)
        .await
        .unwrap();
}
