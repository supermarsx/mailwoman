use mw_store::{
    ServerKey, Store,
    request_protection_model::{Action, Mode, Policy, Signal, WindowPolicy},
};
#[path = "../src/test_db.rs"]
mod test_db;

#[tokio::test]
async fn postgres_user_agent_upgrade_and_admission_use_isolated_schema() {
    let dsn = std::env::var("DATABASE_URL_PG").or_else(|_| std::env::var("MW_TEST_PG"));
    let Ok(dsn) = dsn else {
        assert!(
            std::env::var_os("MW_REQUIRE_LIVE").is_none(),
            "PostgreSQL is required"
        );
        eprintln!("SKIP PostgreSQL user-agent upgrade: no test DSN");
        return;
    };
    let schema = format!("ua_{}", test_db::unique_tag().replace('-', "_"));
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await
        .unwrap();
    let scoped = format!(
        "{dsn}{}options=-csearch_path%3D{schema}",
        if dsn.contains('?') { "&" } else { "?" }
    );
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&scoped)
        .await
        .unwrap();
    sqlx::raw_sql(include_str!("../migrations_pg/0059_request_protection.sql"))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO request_protection_subject VALUES('old','account','{}',100,999999,'revision')",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO request_protection_event VALUES('event','old','account',100,'short-absolute','block',1,300,'{}')").execute(&pool).await.unwrap();
    let mut transaction = pool.begin().await.unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations_pg/0080_request_protection_user_agent.sql"
    ))
    .execute(&mut *transaction)
    .await
    .unwrap();
    transaction.commit().await.unwrap();
    let old: (i64, String) = sqlx::query_as(
        "SELECT expires_at,revision FROM request_protection_subject WHERE subject_key='old'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(old, (999999, "revision".into()));
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM request_protection_event WHERE id='event'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count.0, 1);
    sqlx::query(
        "INSERT INTO request_protection_subject VALUES('ua','ua-custom','{}',100,999999,'r')",
    )
    .execute(&pool)
    .await
    .unwrap();
    assert!(
        sqlx::query("INSERT INTO request_protection_subject VALUES('bad','invalid','{}',1,2,'r')")
            .execute(&pool)
            .await
            .is_err()
    );
    // Only these test-created tables exist in this unique schema. Start a fresh
    // full store there to exercise the actual persisted admission API too.
    sqlx::raw_sql("DROP TABLE request_protection_event; DROP TABLE request_protection_subject;")
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;
    let store = Store::open(&scoped, ServerKey::generate()).await.unwrap();
    let mut p = policy();
    p.rate_mode = Mode::Enforce;
    p.rate_limit = 1;
    p.max_stage = 1;
    assert!(
        !store
            .request_protection("ua-ai-training", "192.0.2.99", &p, 100, None)
            .await
            .unwrap()
            .decision
            .enforced
    );
    assert!(
        store
            .request_protection("ua-ai-training", "192.0.2.99", &p, 100, None)
            .await
            .unwrap()
            .decision
            .enforced
    );
    drop(store);
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&admin)
        .await
        .unwrap();
    admin.close().await;
}

#[tokio::test]
async fn forward_key_repair_preserves_active_state_and_rejects_null_keys() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::raw_sql(include_str!("../migrations/0059_request_protection.sql"))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO request_protection_subject VALUES('subject','account','{\"until\":123456}',100,123456,'revision')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO request_protection_event VALUES('event','subject','account',100,'long-absolute','block',1,300,'{}')").execute(&pool).await.unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/0061_request_protection_keys.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    let row: (String, i64, String) = sqlx::query_as("SELECT state_json,expires_at,revision FROM request_protection_subject WHERE subject_key='subject'").fetch_one(&pool).await.unwrap();
    assert_eq!(
        row,
        ("{\"until\":123456}".into(), 123456, "revision".into())
    );
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM request_protection_event WHERE id='event' AND reason='long-absolute'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count.0, 1);
    assert!(
        sqlx::query("INSERT INTO request_protection_subject VALUES(NULL,'account','{}',1,2,'r')")
            .execute(&pool)
            .await
            .is_err()
    );
    assert!(sqlx::query("INSERT INTO request_protection_event VALUES(NULL,'subject','account',100,'long-absolute','block',1,300,'{}')").execute(&pool).await.is_err());
    sqlx::raw_sql(include_str!(
        "../migrations/0080_request_protection_user_agent.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    let preserved:(String,i64,String)=sqlx::query_as("SELECT state_json,expires_at,revision FROM request_protection_subject WHERE subject_key='subject'").fetch_one(&pool).await.unwrap();
    assert_eq!(preserved, row);
    sqlx::query(
        "INSERT INTO request_protection_subject VALUES('ua','ua-ai-training','{}',1,2,'r')",
    )
    .execute(&pool)
    .await
    .unwrap();
    assert!(
        sqlx::query(
            "INSERT INTO request_protection_subject VALUES('bad','arbitrary','{}',1,2,'r')"
        )
        .execute(&pool)
        .await
        .is_err()
    );
    let events: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM request_protection_event WHERE id='event'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(events.0, 1);
}
fn policy() -> Policy {
    Policy {
        rate_mode: Mode::Off,
        adaptive_mode: Mode::Enforce,
        rate_limit: 10,
        rate_window_secs: 60,
        short: WindowPolicy {
            seconds: 60,
            minimum_samples: 100,
            absolute_anomalies: 100,
            ratio_percent: 90,
        },
        long: WindowPolicy {
            seconds: 7 * 86400,
            minimum_samples: 100,
            absolute_anomalies: 3,
            ratio_percent: 50,
        },
        normal_cap_per_bucket: 1000,
        signals: 7,
        throttle_secs: 3,
        block_secs: 300,
        decay_secs: 3600,
        max_stage: 2,
    }
}
async fn exercise(store: Store, replica: Store, key: ServerKey) {
    let source_path = test_db::unique_file_path("request-protection-copy", "source.sqlite");
    let source = Store::open(source_path.to_str().unwrap(), key)
        .await
        .unwrap();
    let p = policy();
    for _ in 0..3 {
        source
            .request_protection(
                "client",
                "192.0.2.8",
                &p,
                100,
                Some(Signal::CredentialRejected),
            )
            .await
            .unwrap();
    }
    let initial = source
        .request_protection_rows(101, None, 100)
        .await
        .unwrap();
    store
        .migrate_from_sqlite(source_path.to_str().unwrap())
        .await
        .unwrap();
    let restored = store.request_protection_rows(101, None, 100).await.unwrap();
    assert_eq!(restored[0].revision, initial[0].revision);
    assert!(
        replica
            .request_protection("client", "::ffff:192.0.2.8", &p, 101, None)
            .await
            .unwrap()
            .decision
            .enforced
    );
    assert_eq!(
        store
            .request_protection_events(101, None, 100)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store
            .request_protection_subject("client", "192.0.2.8")
            .unwrap(),
        store
            .request_protection_subject("client", "::ffff:192.0.2.8")
            .unwrap()
    );
    assert_ne!(
        store
            .request_protection_subject("account", "192.0.2.8")
            .unwrap(),
        store
            .request_protection_subject("client", "192.0.2.8")
            .unwrap()
    );

    let mut off = p.clone();
    off.adaptive_mode = Mode::Off;
    assert!(
        store
            .request_protection("account", "off-account", &off, 101, None)
            .await
            .unwrap()
            .subject_key
            .is_empty()
    );
    assert_eq!(
        store
            .request_protection_rows(101, None, 100)
            .await
            .unwrap()
            .len(),
        1
    );

    let mut rates = p.clone();
    rates.adaptive_mode = Mode::Off;
    rates.rate_mode = Mode::Enforce;
    let mut tasks = tokio::task::JoinSet::new();
    for index in 0..64 {
        let node = if index % 2 == 0 {
            store.clone()
        } else {
            replica.clone()
        };
        let policy = rates.clone();
        tasks.spawn(async move {
            node.request_protection("account", "verified-account", &policy, 1000, None)
                .await
                .unwrap()
                .decision
                .enforced
        });
    }
    let mut allowed = 0;
    while let Some(answer) = tasks.join_next().await {
        if !answer.unwrap() {
            allowed += 1;
        }
    }
    assert_eq!(
        allowed, 10,
        "two independent stores share one atomic budget"
    );
    let reset = replica
        .request_protection("account", "verified-account", &rates, 1020, None)
        .await
        .unwrap();
    assert_eq!(reset.decision.action, Action::Allow);

    for day in [0, 2] {
        assert_eq!(
            store
                .request_protection(
                    "client",
                    "198.51.100.1",
                    &p,
                    2000 + day * 86400,
                    Some(Signal::Malformed)
                )
                .await
                .unwrap()
                .decision
                .action,
            Action::Allow
        );
    }
    let blocked = replica
        .request_protection(
            "client",
            "198.51.100.1",
            &p,
            2000 + 4 * 86400,
            Some(Signal::Malformed),
        )
        .await
        .unwrap();
    assert!(blocked.decision.enforced);
    assert_eq!(blocked.snapshot.short.anomalous, 1);
    assert_eq!(blocked.snapshot.long.anomalous, 3);
    let updated = store
        .request_protection("client", "198.51.100.1", &p, 2000 + 4 * 86400, None)
        .await
        .unwrap();
    assert!(
        !store
            .clear_request_protection(&blocked.subject_key, &blocked.revision, 2000 + 4 * 86400)
            .await
            .unwrap()
    );
    assert!(
        replica
            .clear_request_protection(&updated.subject_key, &updated.revision, 2000 + 4 * 86400)
            .await
            .unwrap()
    );
    let events = store
        .request_protection_events(2000 + 4 * 86400, None, 100)
        .await
        .unwrap();
    let encoded = serde_json::to_string(&events).unwrap();
    assert!(!encoded.contains("198.51.100.1"));
    assert!(!encoded.contains("verified-account"));
    assert!(events.iter().any(|event| event.action == "cleared"));
    let sustained = events
        .iter()
        .find(|event| event.reason == "long-absolute")
        .unwrap();
    assert_eq!(sustained.snapshot.last_signal, Some(Signal::Malformed));
    assert_eq!(sustained.snapshot.long.anomalous, 3);
    let applied = sustained
        .policy
        .as_ref()
        .expect("audit preserves applied thresholds");
    assert_eq!(applied.long.seconds, 7 * 86400);
    assert_eq!(applied.long.absolute_anomalies, 3);
    assert_eq!(applied.throttle_secs, 3);
    assert!(
        store
            .request_protection_rows(10_000_000, None, 100)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .request_protection_events(10_000_000, None, 100)
            .await
            .unwrap()
            .is_empty()
    );
}
#[tokio::test]
async fn durable_request_budgets_sqlite() {
    let path = test_db::unique_file_path("request-protection", "store.sqlite");
    let key = ServerKey::generate();
    let first = Store::open(path.to_str().unwrap(), key.clone())
        .await
        .unwrap();
    let second = Store::open(path.to_str().unwrap(), key.clone())
        .await
        .unwrap();
    exercise(first, second, key).await;
}
#[tokio::test]
async fn durable_request_budgets_postgres() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        return;
    };
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    let name = format!("mw_request_protection_{:032x}", rand::random::<u128>());
    sqlx::query(&format!("CREATE DATABASE {name}"))
        .execute(&admin)
        .await
        .unwrap();
    let mut url = url::Url::parse(&dsn).unwrap();
    url.set_path(&format!("/{name}"));
    let key = ServerKey::generate();
    let first = Store::open(url.as_str(), key.clone()).await.unwrap();
    let second = Store::open(url.as_str(), key.clone()).await.unwrap();
    exercise(first, second, key).await;
    sqlx::query(&format!("DROP DATABASE {name} WITH (FORCE)"))
        .execute(&admin)
        .await
        .unwrap();
}
