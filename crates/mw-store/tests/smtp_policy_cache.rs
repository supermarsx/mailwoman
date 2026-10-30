use mw_store::{ServerKey, Store};
#[path = "../src/test_db.rs"]
mod test_db;

fn policy(fetched: u64, expires: u64) -> String {
    serde_json::json!({"fetchedAt":fetched,"expiresAt":expires,"id":fetched.to_string(),
        "policy":{"mode":"enforce","mx":["mail.example.org"],"maxAge":expires-fetched}})
    .to_string()
}

#[tokio::test]
#[ignore = "requires an explicitly supplied PostgreSQL fixture"]
async fn postgres_replicas_preserve_newest_policy() {
    let dsn = std::env::var("MW_TEST_PG").expect("MW_TEST_PG must name the test database");
    let schema = format!("smtp_policy_{}", uuid::Uuid::new_v4().simple());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&pool)
        .await
        .unwrap();
    let scoped = format!(
        "{dsn}{}options=-csearch_path%3D{schema}",
        if dsn.contains('?') { "&" } else { "?" }
    );
    let key = ServerKey::generate();
    let first = Store::open(&scoped, key.clone()).await.unwrap();
    let second = Store::open(&scoped, key.clone()).await.unwrap();
    let digest = "b".repeat(64);
    let old = policy(10, 100);
    let new = policy(20, 500);
    let (a, b) = tokio::join!(
        first.cache_smtp_policy(&digest, &old, 30),
        second.cache_smtp_policy(&digest, &new, 30)
    );
    a.unwrap();
    b.unwrap();
    drop(first);
    drop(second);
    let reopened = Store::open(&scoped, key).await.unwrap();
    assert_eq!(
        reopened
            .get_setting(&format!("smtp.transport.sts.{digest}"))
            .await
            .unwrap(),
        Some(new)
    );
    drop(reopened);
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;
}

#[tokio::test]
async fn restart_preserves_policy_and_older_fetch_cannot_replace_it() {
    let path = test_db::unique_file_path("smtp-sts-cache", "policy.sqlite");
    let key = ServerKey::generate();
    let digest = "a".repeat(64);
    let setting = format!("smtp.transport.sts.{digest}");
    let store = Store::open(path.to_str().unwrap(), key.clone())
        .await
        .unwrap();
    store
        .cache_smtp_policy(&digest, &policy(20, 500), 20)
        .await
        .unwrap();
    drop(store);
    let reopened = Store::open(path.to_str().unwrap(), key).await.unwrap();
    reopened
        .cache_smtp_policy(&digest, &policy(10, 100), 30)
        .await
        .unwrap();
    assert_eq!(
        reopened.get_setting(&setting).await.unwrap(),
        Some(policy(20, 500))
    );
    reopened.cache_smtp_policy(&digest, &policy(20, 900), 30).await.unwrap();
    assert_eq!(reopened.get_setting(&setting).await.unwrap(), Some(policy(20, 500)));
    reopened
        .cache_smtp_policy(&digest, &policy(40, 800), 40)
        .await
        .unwrap();
    assert_eq!(
        reopened.get_setting(&setting).await.unwrap(),
        Some(policy(40, 800))
    );
}

#[tokio::test]
async fn full_cache_refuses_new_policy_without_evicting_live_protection() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    for index in 0..1024 {
        store
            .set_setting(&format!("smtp.transport.sts.{index:064x}"), &policy(1, 100))
            .await
            .unwrap();
    }
    let digest = "f".repeat(64);
    assert!(
        store
            .cache_smtp_policy(&digest, &policy(2, 200), 2)
            .await
            .is_err()
    );
    assert_eq!(
        store
            .settings_with_prefix("smtp.transport.sts.")
            .await
            .unwrap()
            .len(),
        1024
    );
    store
        .cache_smtp_policy(&digest, &policy(101, 200), 101)
        .await
        .unwrap();
    assert_eq!(
        store
            .settings_with_prefix("smtp.transport.sts.")
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(
        store
            .cache_smtp_policy("not-a-key", &policy(101, 200), 101)
            .await
            .is_err()
    );
}
