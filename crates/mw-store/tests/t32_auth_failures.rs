use mw_store::{ServerKey, Store, auth_failures::FailurePolicy};
#[path = "../src/test_db.rs"]
mod test_db;
async fn exercise(store: Store) {
    let source_path = test_db::unique_file_path("auth-failure-copy", "source.sqlite");
    let source = Store::open(source_path.to_str().unwrap(), ServerKey::generate())
        .await
        .unwrap();
    let copied = source
        .record_auth_source_failure(
            "admin",
            "198.51.100.8".parse().unwrap(),
            50,
            FailurePolicy {
                threshold: 1,
                window_secs: 60,
                lock_secs: 120,
            },
        )
        .await
        .unwrap()
        .unwrap();
    store
        .migrate_from_sqlite(source_path.to_str().unwrap())
        .await
        .unwrap();
    let restored = store
        .auth_source_lock("admin", "198.51.100.8".parse().unwrap(), 51)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(restored.revision, copied.revision);
    assert_eq!(restored.locked_until, copied.locked_until);

    let ip = "192.0.2.21".parse().unwrap();
    let mapped = "::ffff:192.0.2.21".parse().unwrap();
    let p = FailurePolicy {
        threshold: 3,
        window_secs: 60,
        lock_secs: 120,
    };
    assert!(
        store
            .record_auth_source_failure("admin", ip, 100, p)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .record_auth_source_failure("admin", mapped, 110, p)
            .await
            .unwrap()
            .is_none()
    );
    let lock = store
        .record_auth_source_failure("admin", ip, 120, p)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(lock.locked_until, 240);
    assert_eq!(lock.failures, 3);
    assert!(
        store
            .auth_source_lock("webmail", ip, 121)
            .await
            .unwrap()
            .is_none()
    );
    let late = store
        .record_auth_source_failure("admin", ip, 200, p)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(late.locked_until, 240);
    assert_eq!(late.revision, lock.revision);
    assert!(
        store
            .auth_source_lock("admin", ip, 239)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        store
            .auth_source_lock("admin", ip, 240)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .record_auth_source_failure("admin", ip, 240, p)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        !store
            .clear_auth_source_failures("admin", ip, Some(&lock.revision))
            .await
            .unwrap()
    );
    assert!(
        store
            .clear_auth_source_failures("admin", ip, None)
            .await
            .unwrap()
    );
    // Exact rolling-window boundary discards only expired attempts.
    assert!(
        store
            .record_auth_source_failure("admin", ip, 300, p)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .record_auth_source_failure("admin", ip, 310, p)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .record_auth_source_failure("admin", ip, 360, p)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .record_auth_source_failure("admin", ip, 361, p)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        store
            .clear_auth_source_failures("admin", ip, None)
            .await
            .unwrap()
    );
    // Independent sources share no allowance; concurrent threshold crossing has one expiry.
    let p = FailurePolicy {
        threshold: 5,
        window_secs: 60,
        lock_secs: 120,
    };
    let mut jobs = Vec::new();
    for _ in 0..12 {
        let s = store.clone();
        jobs.push(tokio::spawn(async move {
            s.record_auth_source_failure("webmail", ip, 500, p)
                .await
                .unwrap()
        }));
    }
    let mut locks = Vec::new();
    for job in jobs {
        if let Some(lock) = job.await.unwrap() {
            locks.push(lock)
        }
    }
    assert_eq!(locks.len(), 8);
    assert!(
        locks
            .iter()
            .all(|v| v.locked_until == 620 && v.revision == locks[0].revision)
    );
    assert!(
        store
            .auth_source_lock("webmail", "192.0.2.22".parse().unwrap(), 501)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .clear_auth_source_failures("webmail", ip, Some(&locks[0].revision))
            .await
            .unwrap()
    );
    assert!(
        store
            .auth_source_lock("webmail", ip, 501)
            .await
            .unwrap()
            .is_none()
    );
}
#[tokio::test]
async fn durable_failure_windows_and_concurrency_sqlite() {
    exercise(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}

#[tokio::test]
async fn durable_failure_windows_and_concurrency_postgres() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        return;
    };
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    let name = format!("mw_auth_failure_{:032x}", rand::random::<u128>());
    sqlx::query(&format!("CREATE DATABASE {name}"))
        .execute(&admin)
        .await
        .unwrap();
    let mut url = url::Url::parse(&dsn).unwrap();
    url.set_path(&format!("/{name}"));
    exercise(
        Store::open(url.as_str(), ServerKey::generate())
            .await
            .unwrap(),
    )
    .await;
    sqlx::query(&format!("DROP DATABASE {name} WITH (FORCE)"))
        .execute(&admin)
        .await
        .unwrap();
}
