//! t49pg-e3: start-up validation of the guard snapshot on PostgreSQL.
//! The PostgreSQL cases run only with `MW_TEST_PG`, each in its own schema.
use super::*;
use crate::request_inputs::snapshot::pg_tests::{pg_dsn, pg_schema};
use mw_store::ServerKey;
use sqlx::Connection;

const BOUND: Duration = Duration::from_millis(100);
const MIRROR_APP: &str = "mailwoman-guard-mirror";

fn good() -> PostgresSnapshotFacts {
    PostgresSnapshotFacts { server_version_num: 160_000, in_recovery: false, change_log_xid_column: true }
}

/// Connections of the test database whose application_name is the mirror's.
async fn mirror_connections(dsn: &str) -> i64 {
    let mut admin = sqlx::PgConnection::connect(dsn).await.unwrap();
    let count = sqlx::query_scalar("SELECT count(*) FROM pg_stat_activity WHERE datname=current_database() AND application_name=$1")
        .bind(MIRROR_APP)
        .fetch_one(&mut admin)
        .await
        .unwrap();
    admin.close().await.unwrap();
    count
}
async fn all_connections(dsn: &str) -> i64 {
    let mut admin = sqlx::PgConnection::connect(dsn).await.unwrap();
    // The admin connection itself is one of them, always.
    let count = sqlx::query_scalar("SELECT count(*) FROM pg_stat_activity WHERE datname=current_database()")
        .fetch_one(&mut admin)
        .await
        .unwrap();
    admin.close().await.unwrap();
    count
}
async fn exec(url: &str, sql: &str) {
    let mut conn = sqlx::PgConnection::connect(url).await.unwrap();
    sqlx::query(sql).execute(&mut conn).await.unwrap();
    conn.close().await.unwrap();
}
async fn open(url: &str) -> Store {
    Store::open(url, ServerKey::generate()).await.unwrap()
}

#[test]
fn each_failing_fact_has_its_own_message() {
    assert!(check_facts(&good()).is_ok());
    let old = check_facts(&PostgresSnapshotFacts { server_version_num: 120_011, ..good() }).unwrap_err();
    assert!(old.contains("PostgreSQL 13 or later") && old.contains("120011") && old.contains("MW_GUARD_SNAPSHOT=off"), "{old}");
    let standby = check_facts(&PostgresSnapshotFacts { in_recovery: true, ..good() }).unwrap_err();
    assert!(standby.contains("pg_is_in_recovery() is true") && standby.contains("standby or read replica"), "{standby}");
    let column = check_facts(&PostgresSnapshotFacts { change_log_xid_column: false, ..good() }).unwrap_err();
    assert!(column.contains("authority_change_log.xid") && column.contains("0120"), "{column}");
    // 13.0 is the first accepted version.
    assert!(check_facts(&PostgresSnapshotFacts { server_version_num: 130_000, ..good() }).is_ok());
    assert!(check_facts(&PostgresSnapshotFacts { server_version_num: 129_999, ..good() }).is_err());
}

#[test]
fn health_changes_state_once_per_change() {
    let grace = Duration::from_secs(2);
    let t0 = Instant::now();
    let at = |ms: u64| t0 + Duration::from_millis(ms);
    let mut health = Health::default();
    // Fresh: nothing. A short stale spell below the grace period: nothing.
    assert_eq!(health.observe(true, at(0), grace), None);
    assert_eq!(health.observe(false, at(100), grace), None);
    assert_eq!(health.observe(false, at(1_900), grace), None);
    assert_eq!(health.observe(true, at(2_000), grace), None, "recovered before the grace period: no state change");
    // A long one: exactly one Degraded however many observations follow.
    assert_eq!(health.observe(false, at(3_000), grace), None);
    assert_eq!(health.observe(false, at(5_000), grace), Some(Transition::Degraded));
    for ms in (5_100..9_000).step_by(100) {
        assert_eq!(health.observe(false, at(ms), grace), None, "no repeat warning at {ms} ms");
    }
    // Recovery once, then quiet.
    assert_eq!(health.observe(true, at(9_100), grace), Some(Transition::Recovered));
    assert_eq!(health.observe(true, at(9_200), grace), None);
    // A second spell warns again (a new state change).
    assert_eq!(health.observe(false, at(10_000), grace), None);
    assert_eq!(health.observe(false, at(12_000), grace), Some(Transition::Degraded));
}

#[tokio::test]
async fn sqlite_and_mode_off_check_nothing() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    validate(&store, Mode::Bounded(BOUND)).await.expect("SQLite bounded: unchanged, accepted");
    validate(&store, Mode::Off).await.unwrap();
    assert!(store.postgres_snapshot_facts().await.unwrap().is_none());
    assert!(STARTED.lock().unwrap().as_ref().is_none_or(|mirror| !mirror.is_live() || mirror.counters().polls == 0));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn postgres_mode_off_runs_no_check_and_opens_no_connection() {
    let Some(dsn) = pg_dsn() else { return };
    let url = pg_schema(&dsn).await;
    let store = open(&url).await;
    // Poison every precondition: with the mode off none of it is looked at.
    exec(&url, "ALTER TABLE authority_change_log DROP COLUMN xid").await;
    let before = all_connections(&dsn).await;
    validate(&store, Mode::Off).await.expect("mode off performs no checks");
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(all_connections(&dsn).await, before, "mode off opens no connection");
    assert_eq!(mirror_connections(&dsn).await, 0);
    store.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn postgres_happy_path_starts_with_one_complete_poll_and_a_named_connection() {
    let Some(dsn) = pg_dsn() else { return };
    let url = pg_schema(&dsn).await;
    let store = open(&url).await;
    let facts = store.postgres_snapshot_facts().await.unwrap().expect("PostgreSQL");
    assert!(facts.server_version_num >= 130_000 && !facts.in_recovery && facts.change_log_xid_column, "{facts:?}");
    let before = mirror_connections(&dsn).await;
    validate(&store, Mode::Bounded(BOUND)).await.expect("a healthy database starts");
    let mirror = STARTED.lock().unwrap().clone().expect("the started mirror is kept");
    let counters = mirror.counters();
    assert!(counters.polls >= 1 && counters.failures == 0 && counters.connects == 1, "{counters:?}");
    assert!(mirror.is_live());
    assert_eq!(mirror_connections(&dsn).await - before, 1, "one extra, named connection");
    assert_eq!(witness()["validated"], true);
    assert_eq!(witness()["degraded"], false);
    // The request path's mirror for this store is the one validated.
    assert!(Arc::ptr_eq(&EpochMirror::for_store(&store, BOUND), &mirror));
    release();
    drop(mirror);
    store.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn postgres_each_failed_precondition_refuses_with_its_message_and_starts_no_mirror() {
    let Some(dsn) = pg_dsn() else { return };
    // Simulated at the check's seam: an old server and a standby.
    let url = pg_schema(&dsn).await;
    let store = open(&url).await;
    let old = validate_with(&store, BOUND, PostgresSnapshotFacts { server_version_num: 120_000, ..good() }).await.unwrap_err();
    assert!(old.contains("PostgreSQL 13 or later"), "{old}");
    let standby = validate_with(&store, BOUND, PostgresSnapshotFacts { in_recovery: true, ..good() }).await.unwrap_err();
    assert!(standby.contains("pg_is_in_recovery() is true"), "{standby}");
    assert_eq!(mirror_connections(&dsn).await, 0, "a refused start leaves no poll connection");
    store.close().await;

    // Real: the xid column removed.
    let url = pg_schema(&dsn).await;
    let store = open(&url).await;
    exec(&url, "ALTER TABLE authority_change_log DROP COLUMN xid").await;
    let error = validate(&store, Mode::Bounded(BOUND)).await.unwrap_err();
    assert!(error.contains("authority_change_log.xid") && error.contains("migrations"), "{error}");
    store.close().await;

    // Real: a required trigger disabled, then dropped.
    let url = pg_schema(&dsn).await;
    let store = open(&url).await;
    exec(&url, "ALTER TABLE authority_epoch DISABLE TRIGGER authority_change_log_epoch").await;
    let error = validate(&store, Mode::Bounded(BOUND)).await.unwrap_err();
    assert!(error.contains("triggers or functions") && error.contains("missing, disabled or changed"), "{error}");
    exec(&url, "ALTER TABLE authority_epoch ENABLE TRIGGER authority_change_log_epoch").await;
    exec(&url, "DROP TRIGGER authority_change_log_revision ON settings_revision").await;
    let error = validate(&store, Mode::Bounded(BOUND)).await.unwrap_err();
    assert!(error.contains("triggers or functions"), "{error}");
    assert_eq!(mirror_connections(&dsn).await, 0, "no poll connection was opened by a failed definition check");
    store.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn postgres_unreachable_poll_connection_refuses_naming_the_poll() {
    let Some(dsn) = pg_dsn() else { return };
    let url = pg_schema(&dsn).await;
    let store = open(&url).await;
    // Every statement the poll connection runs first waits behind this lock:
    // the change log is unreadable, so no complete poll can succeed.
    let mut locker = sqlx::PgConnection::connect(&url).await.unwrap();
    sqlx::query("BEGIN").execute(&mut locker).await.unwrap();
    sqlx::query("LOCK TABLE authority_change_log IN ACCESS EXCLUSIVE MODE").execute(&mut locker).await.unwrap();
    let error = validate(&store, Mode::Bounded(BOUND)).await.unwrap_err();
    assert!(error.contains("poll connection") && error.contains("one complete poll"), "{error}");
    sqlx::query("ROLLBACK").execute(&mut locker).await.unwrap();
    locker.close().await.unwrap();
    store.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn postgres_a_failure_after_start_warns_once_then_recovers_once() {
    let Some(dsn) = pg_dsn() else { return };
    let url = pg_schema(&dsn).await;
    let store = open(&url).await;
    let mirror = EpochMirror::for_store(&store, BOUND);
    assert!(mirror.poll_now().await.is_ok() && mirror.poll_now().await.is_ok());
    let status = Monitor::spawn(&mirror, Duration::from_millis(20), Duration::from_millis(400));
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(mirror.is_fresh() && !status.degraded() && status.degradations() == 0);

    // The change log becomes unreadable: polls fail, reuse stops.
    let mut locker = sqlx::PgConnection::connect(&url).await.unwrap();
    sqlx::query("BEGIN").execute(&mut locker).await.unwrap();
    sqlx::query("LOCK TABLE authority_change_log IN ACCESS EXCLUSIVE MODE").execute(&mut locker).await.unwrap();
    let wait = |what: &'static str, done: Box<dyn Fn() -> bool>| async move {
        tokio::time::timeout(Duration::from_secs(20), async {
            while !done() {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap_or_else(|_| panic!("timed out waiting for {what}"));
    };
    let s = status.clone();
    wait("degraded", Box::new(move || s.degraded())).await;
    assert!(!mirror.is_fresh(), "no reuse while degraded");
    // A long stretch of failing polls and monitor ticks: still one warning.
    tokio::time::sleep(Duration::from_millis(1_500)).await;
    assert_eq!((status.degradations(), status.recoveries()), (1, 0));
    assert!(mirror.counters().failures >= 2, "the mirror kept polling and failing");

    sqlx::query("ROLLBACK").execute(&mut locker).await.unwrap();
    locker.close().await.unwrap();
    let s = status.clone();
    wait("recovered", Box::new(move || !s.degraded())).await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!((status.degradations(), status.recoveries()), (1, 1), "one warning, one recovery");
    assert!(mirror.is_fresh());
    drop(mirror);
    store.close().await;
}
