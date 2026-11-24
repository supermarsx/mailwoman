use super::*;
use mw_store::{
    ServerKey,
    request_protection_model::{Action, Mode, Reason, WindowPolicy},
};
use serde_json::{Value, json};

fn policy() -> Policy {
    Policy {
        rate_mode: Mode::Enforce,
        adaptive_mode: Mode::Enforce,
        rate_limit: 3,
        rate_window_secs: 60,
        short: WindowPolicy { seconds: 60, minimum_samples: 2, absolute_anomalies: 2, ratio_percent: 50 },
        long: WindowPolicy { seconds: 3600, minimum_samples: 2, absolute_anomalies: 2, ratio_percent: 50 },
        normal_cap_per_bucket: 1000,
        signals: 63,
        throttle_secs: 3,
        block_secs: 300,
        decay_secs: 3600,
        max_stage: 2,
    }
}
fn config(flush_ms: u64, max_pending: usize) -> AccountingConfig {
    AccountingConfig {
        mode: AccountingMode::WriteBehind,
        flush_every: Duration::from_millis(flush_ms),
        max_pending,
    }
}
const NONE: ProtectionSource<'static> = ProtectionSource { country: None, ip: None };

/// Committed rows and audit events, without the random revision and event ids.
async fn inventory(store: &Store, now: i64) -> (Vec<Value>, Vec<Value>) {
    let mut rows: Vec<Value> = store
        .request_protection_rows(now, None, 250)
        .await
        .unwrap()
        .into_iter()
        .map(|row| {
            json!({"key":row.subject_key,"kind":row.subject_kind,"updated":row.updated_at,
                "expires":row.expires_at,"snapshot":row.snapshot})
        })
        .collect();
    rows.sort_by_key(Value::to_string);
    let mut events: Vec<Value> = store
        .request_protection_events(now, None, 250)
        .await
        .unwrap()
        .into_iter()
        .map(|event| {
            json!({"key":event.subject_key,"at":event.observed_at,"reason":event.reason,
                "action":event.action,"enforced":event.enforced,"retry":event.retry_after_secs,
                "snapshot":event.snapshot,"policy":event.policy})
        })
        .collect();
    events.sort_by_key(Value::to_string);
    (rows, events)
}
async fn rate_count(store: &Store, kind: &str, value: &str, now: i64) -> Option<u32> {
    let key = store.request_protection_subject(kind, value).unwrap();
    store
        .request_protection_load(&key, kind, now)
        .await
        .unwrap()
        .map(|row| row.state.snapshot().rate_count)
}
/// A plain SQLite connection to the same file, for fault injection.
async fn raw(path: &std::path::Path) -> sqlx::SqliteConnection {
    use sqlx::Connection;
    sqlx::SqliteConnection::connect(&format!("sqlite://{}", path.display()))
        .await
        .unwrap()
}
async fn exec(connection: &mut sqlx::SqliteConnection, sql: &str) {
    use sqlx::Executor;
    connection.execute(sql).await.unwrap();
}
const FAIL_UPDATES: &str = "CREATE TRIGGER wb_fail BEFORE UPDATE ON request_protection_subject BEGIN SELECT RAISE(ABORT,'injected'); END";

#[test]
fn switches_default_to_synchronous_and_reject_invalid_values() {
    assert_eq!(AccountingConfig::parse(None, None, None).unwrap(), AccountingConfig::default());
    assert_eq!(AccountingConfig::parse(Some(""), Some(" "), None).unwrap(), AccountingConfig::default());
    assert_eq!(AccountingConfig::default().mode, AccountingMode::Synchronous);
    let parsed = AccountingConfig::parse(Some("write-behind"), Some("50"), Some("65536")).unwrap();
    assert_eq!(parsed, config(50, 65536));
    for (mode, flush, max, name) in [
        (Some("writebehind"), None, None, MODE_VAR),
        (Some("Synchronous"), None, None, MODE_VAR),
        (None, Some("49"), None, FLUSH_VAR),
        (None, Some("5001"), None, FLUSH_VAR),
        (None, Some("1e3"), None, FLUSH_VAR),
        (None, None, Some("63"), MAX_PENDING_VAR),
        (None, None, Some("65537"), MAX_PENDING_VAR),
    ] {
        let error = AccountingConfig::parse(mode, flush, max).unwrap_err();
        assert!(error.starts_with(name), "{error}");
    }
}

#[tokio::test]
async fn postgres_stays_synchronous_and_sqlite_enables_write_behind() {
    assert_eq!(AccountingMode::WriteBehind.effective(false), AccountingMode::Synchronous);
    assert_eq!(AccountingMode::WriteBehind.effective(true), AccountingMode::WriteBehind);
    assert_eq!(AccountingMode::Synchronous.effective(true), AccountingMode::Synchronous);
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let synchronous = super::super::Runtime::start_with(store.clone(), AccountingConfig::default());
    assert!(synchronous.write_behind().is_none() && synchronous.accounting_status().is_none());
    let deferred = super::super::Runtime::start_with(store, config(250, 4096));
    assert!(deferred.write_behind().is_some());
    assert!(
        deferred
            .flush_all(tokio::time::Instant::now() + Duration::from_secs(1))
            .await
            .is_ok()
    );
}

#[test]
fn storage_failure_rule_matches_the_synchronous_path() {
    let mut p = policy();
    let refused = Decision { action: Action::Throttle, reason: Some(Reason::RequestRate), enforced: true, retry_after_secs: 5 };
    let status = |result: Result<(), crate::admin_ctx::AdminApiError>| result.err().map(|error| error.status);
    use axum::http::StatusCode;
    assert_eq!(status(super::super::storage_failure_answer(true, &p, Decision::default())), Some(StatusCode::SERVICE_UNAVAILABLE));
    assert_eq!(status(super::super::storage_failure_answer(false, &p, Decision::default())), None);
    assert_eq!(status(super::super::storage_failure_answer(false, &p, refused)), Some(StatusCode::TOO_MANY_REQUESTS));
    p.rate_mode = Mode::Observe;
    p.adaptive_mode = Mode::Observe;
    assert_eq!(status(super::super::storage_failure_answer(true, &p, Decision::default())), None, "closed applies only to an enforcing mode");
}

/// Differential test: identical ordered observations through the transactional
/// path and through write-behind give identical decisions, and after a flush the
/// same committed rows and audit events.
#[tokio::test]
async fn write_behind_decisions_and_rows_equal_the_synchronous_path() {
    let signals = [
        None, None, None, Some(Signal::Normal), Some(Signal::Normal), Some(Signal::Malformed),
        Some(Signal::CredentialRejected), Some(Signal::ProbePath), Some(Signal::BotUserAgent),
        Some(Signal::Protocol), Some(Signal::Ignored),
    ];
    let mut variants = vec![policy()];
    let mut v = policy(); v.rate_limit = 1; variants.push(v);
    let mut v = policy(); v.max_stage = 1; v.signals = 5; variants.push(v);
    let mut v = policy(); v.adaptive_mode = Mode::Observe; v.rate_limit = 6; variants.push(v);
    let mut v = policy(); v.rate_mode = Mode::Off; variants.push(v);
    for seed in [0x9e37_79b9_7f4a_7c15_u64, 7, 0xdead_beef] {
        let key = ServerKey::generate();
        let serial = Store::open_in_memory(key.clone()).await.unwrap();
        let deferred_store = Store::open_in_memory(key).await.unwrap();
        let deferred = WriteBehind::new(deferred_store.clone(), config(5000, 4096));
        let mut seed = seed;
        let mut next = |n: u64| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed % n
        };
        let mut now = 10_000_i64;
        let (mut refused, mut through) = (0, 0);
        for step in 0..500 {
            now += next(30) as i64;
            if next(60) == 0 {
                now += 4000;
            }
            let kind: &'static str = ["client", "account", "ua-automation"][next(3) as usize];
            let value = if kind == "account" {
                ["a", "b"][next(2) as usize].to_owned()
            } else {
                ["192.0.2.1", "192.0.2.2", "2001:db8::1"][next(3) as usize].to_owned()
            };
            let p = &variants[next(variants.len() as u64) as usize];
            let signal = signals[next(signals.len() as u64) as usize];
            let source = ProtectionSource { country: Some("PT"), ip: None };
            let before = deferred.status().write_throughs;
            let expected = serial
                .request_protection_with_source(kind, &value, p, now, signal, source)
                .await
                .unwrap();
            let answer = deferred.record(kind, &value, p, now, signal, source).await;
            assert!(!answer.storage_failed, "step {step}");
            assert_eq!(answer.decision, expected.decision, "step {step}: {kind} {value} {signal:?}");
            refused += usize::from(answer.decision.enforced);
            through += usize::from(deferred.status().write_throughs > before);
            if next(25) == 0 {
                deferred.flush_pending().await.unwrap();
            }
        }
        deferred.flush_pending().await.unwrap();
        assert_eq!(deferred.status().pending, 0);
        assert!(refused > 10 && through > 5, "refused {refused}, write-throughs {through}");
        assert!(deferred.status().batches > through as u64 / 2);
        assert_eq!(inventory(&serial, now).await, inventory(&deferred_store, now).await);
    }
}

#[tokio::test]
async fn over_limit_is_refused_in_process_and_the_transition_is_durable_first() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let deferred = WriteBehind::new(store.clone(), config(5000, 4096));
    let p = policy();
    for _ in 0..3 {
        assert!(!deferred.record("account", "a", &p, 1000, None, NONE).await.decision.enforced);
    }
    // Only the row-creating first charge was written; two are queued.
    assert_eq!(deferred.status().pending, 2);
    assert_eq!(rate_count(&store, "account", "a", 1000).await, Some(1));
    let refusal = deferred.record("account", "a", &p, 1000, None, NONE).await;
    assert!(refusal.decision.enforced && !refusal.storage_failed);
    // The refusing decision was returned only after its transition committed.
    assert_eq!(rate_count(&store, "account", "a", 1000).await, Some(4));
    let events = store.request_protection_events(1000, None, 10).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].reason, "request-rate");
    let writes = deferred.status().write_throughs;
    // Further refusals need no flush and no new transition.
    for _ in 0..5 {
        assert!(deferred.record("account", "a", &p, 1000, None, NONE).await.decision.enforced);
    }
    assert_eq!(deferred.status().write_throughs, writes);
    assert_eq!(deferred.status().pending, 5);
}

#[tokio::test]
async fn crash_loses_at_most_the_queued_ordinary_observations_and_restart_rehydrates() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let mut p = policy();
    p.rate_limit = 100;
    let first = WriteBehind::new(store.clone(), config(5000, 4096));
    // A transition (probe block) is written through before its decision returns.
    let probe = first.record("client", "probe-path:192.0.2.9", &p, 1000, Some(Signal::ProbePath), NONE).await;
    assert!(probe.decision.enforced);
    for _ in 0..11 {
        first.record("account", "a", &p, 1000, None, NONE).await;
    }
    let queued = first.status().pending;
    assert_eq!(queued, 10, "the first charge created the row synchronously");
    drop(first); // Crash: nothing flushed.
    assert_eq!(rate_count(&store, "account", "a", 1000).await, Some(1));
    assert_eq!(11 - 1, queued, "lost observations equal the queued ones");
    let events = store.request_protection_events(1000, None, 10).await.unwrap();
    assert!(events.iter().any(|event| event.reason == "probe-path"), "transitions survive");
    // Restart: state comes back from the store.
    let restarted = WriteBehind::new(store.clone(), config(5000, 4096));
    let blocked = restarted.probe_decision("client", "probe-path:192.0.2.9", &p, 1001).await;
    assert!(blocked.decision.enforced && !blocked.storage_failed);
    p.rate_limit = 3;
    let budget = WriteBehind::new(store.clone(), config(5000, 4096));
    for _ in 0..3 {
        budget.record("account", "b", &p, 2000, None, NONE).await;
    }
    budget
        .flush_all(tokio::time::Instant::now() + Duration::from_secs(5))
        .await
        .unwrap();
    drop(budget);
    let rehydrated = WriteBehind::new(store.clone(), config(5000, 4096));
    assert!(rehydrated.record("account", "b", &p, 2000, None, NONE).await.decision.enforced);
    assert_eq!(rehydrated.status().loads, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_requests_for_one_cold_subject_are_counted_exactly_once() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    // Create the row first so the cold load races, not the row creation.
    store.request_protection("account", "busy", &{ let mut p = policy(); p.rate_limit = 1000; p }, 1000, None).await.unwrap();
    let deferred = Arc::new(WriteBehind::new(store.clone(), config(5000, 4096)));
    let mut p = policy();
    p.rate_limit = 1000;
    let tasks: Vec<_> = (0..64)
        .map(|_| {
            let deferred = deferred.clone();
            let p = p.clone();
            tokio::spawn(async move { deferred.record("account", "busy", &p, 1000, None, NONE).await })
        })
        .collect();
    for task in tasks {
        assert!(!task.await.unwrap().storage_failed);
    }
    deferred.flush_pending().await.unwrap();
    assert_eq!(rate_count(&store, "account", "busy", 1000).await, Some(65));
    let key = store.request_protection_subject("account", "busy").unwrap();
    let cached = lock(deferred.shard(&key)).get(&key).unwrap().state.snapshot().rate_count;
    assert_eq!(cached, 65);
}

#[tokio::test]
async fn background_flusher_commits_within_the_interval() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let deferred = WriteBehind::start(store.clone(), config(50, 4096));
    let mut p = policy();
    p.rate_limit = 100;
    for _ in 0..4 {
        deferred.record("account", "a", &p, 1000, None, NONE).await;
    }
    assert_eq!(deferred.status().pending, 3);
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert_eq!(deferred.status().pending, 0);
    assert_eq!(rate_count(&store, "account", "a", 1000).await, Some(4));
}

#[tokio::test]
async fn flush_failure_degrades_after_two_intervals_and_recovers_without_loss() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wb.db");
    let store = Store::open(path.to_str().unwrap(), ServerKey::generate()).await.unwrap();
    let deferred = WriteBehind::new(store.clone(), config(50, 4096));
    let mut p = policy();
    p.rate_limit = 100;
    deferred.record("account", "a", &p, 1000, None, NONE).await;
    let mut connection = raw(&path).await;
    exec(&mut connection, FAIL_UPDATES).await;
    assert!(!deferred.record("account", "a", &p, 1000, None, NONE).await.storage_failed);
    assert!(deferred.flush_pending().await.is_err());
    let status = deferred.status();
    assert!(status.degraded && status.pending == 1 && status.flush_failures == 1);
    // Within 2 x FLUSH_MS the local decision is used.
    assert!(!deferred.record("account", "a", &p, 1000, None, NONE).await.storage_failed);
    tokio::time::sleep(Duration::from_millis(150)).await;
    let late = deferred.record("account", "a", &p, 1000, None, NONE).await;
    assert!(late.storage_failed, "degraded beyond 2 x FLUSH_MS is a storage failure");
    assert!(!late.decision.enforced);
    // A transition cannot be written through while storage fails.
    let probe = deferred.record("account", "a", &p, 1000, Some(Signal::ProbePath), NONE).await;
    assert!(probe.storage_failed && probe.decision.enforced, "local refusal is kept");
    assert_eq!(deferred.status().write_through_failures, 1);
    exec(&mut connection, "DROP TRIGGER wb_fail").await;
    deferred.flush_pending().await.unwrap();
    assert!(!deferred.status().degraded);
    assert_eq!(rate_count(&store, "account", "a", 1000).await, Some(4), "retained observations committed");
    let events = store.request_protection_events(1000, None, 10).await.unwrap();
    assert!(events.iter().any(|event| event.reason == "probe-path"));
}

/// What the guard answers for a write-behind admission (request_protection.rs
/// `Context::admit`): the decision, or the storage-failure policy.
fn guard_status(answer: Answer, fail_closed: bool, p: &Policy) -> Option<axum::http::StatusCode> {
    use axum::http::StatusCode;
    if !answer.storage_failed {
        return answer.decision.enforced.then_some(StatusCode::TOO_MANY_REQUESTS);
    }
    super::super::storage_failure_answer(fail_closed, p, answer.decision).err().map(|error| error.status)
}

/// R5 with failing storage: at MAX_PENDING observations are still recorded and
/// written through; the write-throughs fail, so the storage-failure policy
/// applies (503 when closed, the local decision when open: known abuse stays
/// refused). Only beyond 2 x MAX_PENDING is an observation not recorded. After
/// recovery every recorded observation is committed.
#[tokio::test]
async fn full_queue_with_failing_storage_applies_the_policy_and_keeps_counting_up_to_the_hard_cap() {
    use axum::http::StatusCode;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("full.db");
    let store = Store::open(path.to_str().unwrap(), ServerKey::generate()).await.unwrap();
    let deferred = WriteBehind::new(store.clone(), config(5000, 64));
    let mut p = policy();
    p.rate_limit = 1000;
    let blocked = deferred.record("client", "192.0.2.5", &p, 1000, Some(Signal::ProbePath), NONE).await;
    assert!(blocked.decision.enforced && !blocked.storage_failed);
    deferred.record("account", "a", &p, 1000, None, NONE).await; // row created while healthy
    let mut connection = raw(&path).await;
    exec(&mut connection, FAIL_UPDATES).await;
    for _ in 0..63 {
        assert!(!deferred.record("account", "a", &p, 1000, None, NONE).await.storage_failed);
    }
    assert_eq!(deferred.status().pending, 63);
    deferred.record("account", "a", &p, 1000, None, NONE).await;
    assert_eq!(deferred.status().pending, 64, "full");
    let mut recorded = 65;
    for _ in 0..64 {
        let answer = deferred.record("account", "a", &p, 1000, None, NONE).await;
        recorded += 1;
        assert!(answer.storage_failed, "the write-through failed");
        assert_eq!(guard_status(answer, true, &p), Some(StatusCode::SERVICE_UNAVAILABLE));
        assert_eq!(guard_status(answer, false, &p), None);
    }
    assert_eq!(deferred.status().pending, 128, "recorded, not dropped");
    assert_eq!(deferred.status().dropped, 0);
    // The hard cap: not recorded; known abuse is still refused when open.
    let refused = deferred.record("client", "192.0.2.5", &p, 1001, None, NONE).await;
    assert!(refused.storage_failed && refused.decision.enforced);
    assert_eq!(guard_status(refused, false, &p), Some(StatusCode::TOO_MANY_REQUESTS));
    assert_eq!(guard_status(refused, true, &p), Some(StatusCode::SERVICE_UNAVAILABLE));
    assert_eq!(deferred.status().dropped, 1);
    exec(&mut connection, "DROP TRIGGER wb_fail").await;
    deferred.flush_pending().await.unwrap();
    assert_eq!(rate_count(&store, "account", "a", 1000).await, Some(recorded));
}

#[tokio::test]
async fn external_row_changes_are_reconciled_by_one_flush() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wb.db");
    let key = ServerKey::generate();
    let store = Store::open(path.to_str().unwrap(), key.clone()).await.unwrap();
    let other = Store::open(path.to_str().unwrap(), key).await.unwrap();
    let deferred = WriteBehind::new(store.clone(), config(5000, 4096));
    let mut p = policy();
    p.rate_limit = 10;
    deferred.record("account", "a", &p, 1000, None, NONE).await;
    deferred.record("account", "a", &p, 1000, None, NONE).await;
    for _ in 0..7 {
        other.request_protection("account", "a", &p, 1000, None).await.unwrap();
    }
    deferred.flush_pending().await.unwrap();
    assert_eq!(rate_count(&store, "account", "a", 1000).await, Some(9));
    // The cache now holds the committed count, not this process's two.
    assert!(!deferred.record("account", "a", &p, 1000, None, NONE).await.decision.enforced);
    assert!(deferred.record("account", "a", &p, 1000, None, NONE).await.decision.enforced);
    assert_eq!(deferred.status().loads, 1, "reconciled by the flush, not by a re-read");
}

#[tokio::test]
async fn admin_clear_flushes_first_and_resets_the_cached_subject() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let deferred = WriteBehind::new(store.clone(), config(5000, 4096));
    let p = policy();
    assert!(deferred.record("client", "192.0.2.7", &p, 1000, Some(Signal::ProbePath), NONE).await.decision.enforced);
    assert!(deferred.record("client", "192.0.2.7", &p, 1001, None, NONE).await.decision.enforced);
    assert_eq!(deferred.status().pending, 1);
    let key = store.request_protection_subject("client", "192.0.2.7").unwrap();
    // A revision read before the pending observation committed is stale, as an
    // intervening request makes it in the synchronous path.
    let before = store.request_protection_rows(1001, None, 10).await.unwrap()[0].revision.clone();
    assert!(!deferred.clear(&key, &before, 1001).await.unwrap());
    assert_eq!(deferred.status().pending, 0);
    let current = store.request_protection_rows(1001, None, 10).await.unwrap()[0].revision.clone();
    assert!(deferred.clear(&key, &current, 1002).await.unwrap());
    assert!(!deferred.record("client", "192.0.2.7", &p, 1002, None, NONE).await.decision.enforced);
    deferred.invalidate();
    assert_eq!(deferred.status().cache_size, 0);
}

#[tokio::test]
async fn idle_cache_is_bounded_and_rereads_after_one_interval() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let deferred = WriteBehind::new(store.clone(), config(50, 4096));
    let p = policy();
    for i in 0..(SUBJECT_CAPACITY as usize + 500) {
        let ip = format!("10.{}.{}.{}", i >> 16, (i >> 8) & 255, i & 255);
        deferred.probe_decision("client", &format!("probe-path:{ip}"), &p, 1000).await;
    }
    let status = deferred.status();
    assert!(status.cache_size <= SUBJECT_CAPACITY as usize, "{status:?}");
    assert!(status.evictions > 0);
    // Absent subjects create no rows.
    assert!(store.request_protection_rows(1000, None, 10).await.unwrap().is_empty());
    let loads = deferred.status().loads;
    deferred.probe_decision("client", "probe-path:10.0.0.1", &p, 1000).await;
    tokio::time::sleep(Duration::from_millis(80)).await;
    deferred.probe_decision("client", "probe-path:10.0.0.1", &p, 1000).await;
    assert!(deferred.status().loads >= loads + 1);
}

#[tokio::test]
async fn fixture_snapshot_reports_counts_only_and_zero_when_synchronous() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    // Synchronous mode registers nothing; any earlier instance is gone.
    let synchronous = super::super::Runtime::start_with(store.clone(), AccountingConfig::default());
    let idle = fixture_snapshot();
    assert_eq!(idle["accounting"], "synchronous");
    assert_eq!(idle["supported"], true);
    for field in ["pending", "flushed", "batches", "writeThroughs", "writeThroughFailures",
        "flushFailures", "rejected", "dropped", "loads", "storageFailures"] {
        assert_eq!(idle[field], 0, "{field}");
    }
    assert_eq!(idle["degraded"], false);
    drop(synchronous);
    let runtime = super::super::Runtime::start_with(store.clone(), config(5000, 4096));
    let accounting = runtime.write_behind().unwrap();
    let mut p = policy();
    p.rate_limit = 100;
    for _ in 0..5 {
        accounting.record("account", "witness", &p, 1000, None, NONE).await;
    }
    let busy = fixture_snapshot();
    assert_eq!(busy["accounting"], "write-behind");
    assert_eq!(busy["pending"], 4);
    assert_eq!(busy["writeThroughs"], 1);
    accounting.flush_pending().await.unwrap();
    let flushed = fixture_snapshot();
    assert_eq!(flushed["pending"], 0);
    assert_eq!(flushed["flushed"], 5);
    let text = flushed.to_string();
    let key = store.request_protection_subject("account", "witness").unwrap();
    assert!(!text.contains(&key) && !text.contains("witness"), "counts only: {text}");
    drop(runtime);
    assert_eq!(fixture_snapshot()["accounting"], "synchronous");
}

// ---------------------------------------------------------------------------
// t50hp-e8: ownership rule, set-based flush, and the "limits still limit" tests.
// ---------------------------------------------------------------------------

use mw_store::pool_telemetry::{QuerySnapshot, measure_queries};

fn file_store_path(dir: &tempfile::TempDir) -> String {
    dir.path().join("owned.db").to_string_lossy().into_owned()
}
fn statements(sql: &QuerySnapshot) -> (u64, u64, u64) {
    (sql.read.calls, sql.write.calls, sql.other.calls)
}
/// Wait (bounded) until the flusher has asserted ownership or given it up.
async fn settle_ownership(accounting: &WriteBehind, owned: bool) {
    for _ in 0..300 {
        if accounting.status().owned == owned {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("ownership did not become {owned}: {:?}", accounting.status());
}

/// Acceptance 1 and 7 (unit level): with ownership, a warm subject is answered
/// with no SQL at a real client's pace (longer than a flush interval between
/// requests), and the request past the budget is throttled at once, its
/// transition written through first. Without ownership the same request re-reads.
#[tokio::test]
async fn owned_subject_is_answered_without_sql_and_throttled_on_the_next_request() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&file_store_path(&dir), ServerKey::generate()).await.unwrap();
    let accounting = WriteBehind::start(store.clone(), config(50, 4096));
    settle_ownership(&accounting, true).await;
    let p = policy(); // rate limit 3
    assert!(!accounting.record("account", "paced", &p, 1000, None, NONE).await.decision.enforced);
    for _ in 0..2 {
        // Idle for more than one flush interval: the flusher commits and the
        // entry is synced, then the next request arrives.
        tokio::time::sleep(Duration::from_millis(120)).await;
        let (answer, sql) = measure_queries(accounting.record("account", "paced", &p, 1000, None, NONE)).await;
        assert!(!answer.decision.enforced && !answer.storage_failed);
        assert_eq!(statements(&sql), (0, 0, 0), "warm owned subject: no SQL ({sql:?})");
    }
    tokio::time::sleep(Duration::from_millis(120)).await;
    let (refused, sql) = measure_queries(accounting.record("account", "paced", &p, 1000, None, NONE)).await;
    assert!(refused.decision.enforced && !refused.storage_failed, "throttled on the first request past the budget");
    assert!(sql.write.calls > 0, "the transition is written through before the answer");
    assert_eq!(rate_count(&store, "account", "paced", 1000).await, Some(4));
    let events = store.request_protection_events(1000, None, 10).await.unwrap();
    assert!(events.iter().any(|event| event.reason == "request-rate" && event.enforced));
    // Still refused from memory afterwards.
    let (again, sql) = measure_queries(accounting.record("account", "paced", &p, 1000, None, NONE)).await;
    assert!(again.decision.enforced);
    assert_eq!(statements(&sql), (0, 0, 0), "{sql:?}");
    assert_eq!(accounting.status().loads, 1, "one cold load in total");

    // The same pace without ownership: every request after an interval re-reads.
    let other_dir = tempfile::tempdir().unwrap();
    let other = Store::open(&file_store_path(&other_dir), ServerKey::generate()).await.unwrap();
    let unowned = WriteBehind::new(other.clone(), config(50, 4096)).without_ownership();
    unowned.record("account", "paced", &p, 1000, None, NONE).await;
    unowned.flush_pending().await.unwrap();
    tokio::time::sleep(Duration::from_millis(120)).await;
    let (_, sql) = measure_queries(unowned.record("account", "paced", &p, 1000, None, NONE)).await;
    assert_eq!(sql.read.calls, 1, "re-read rule: {sql:?}");
}

/// Acceptance 2: a probe-path block refuses the next request from that source
/// with 0 SQL reads, and an administrative unblock admits the next one.
#[tokio::test]
async fn probe_block_refuses_from_memory_and_admin_unblock_admits_the_next_request() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&file_store_path(&dir), ServerKey::generate()).await.unwrap();
    let accounting = WriteBehind::start(store.clone(), config(50, 4096));
    settle_ownership(&accounting, true).await;
    let p = policy();
    let subject = "probe-path:192.0.2.50";
    let hit = accounting.record("client", subject, &p, 1000, Some(Signal::ProbePath), NONE).await;
    assert!(hit.decision.enforced && hit.decision.action == Action::Block);
    tokio::time::sleep(Duration::from_millis(120)).await;
    for now in [1001, 1100, 1299] {
        let (answer, sql) = measure_queries(accounting.probe_decision("client", subject, &p, now)).await;
        assert!(answer.decision.enforced && !answer.storage_failed, "{now}");
        assert_eq!(statements(&sql), (0, 0, 0), "probe decision from the cached entry ({sql:?})");
    }
    // The block ends by its own window, still from memory.
    let (expired, sql) = measure_queries(accounting.probe_decision("client", subject, &p, 1300)).await;
    assert!(!expired.decision.enforced);
    assert_eq!(statements(&sql), (0, 0, 0));
    // Administrative unblock (the console's clear, by revision).
    let key = store.request_protection_subject("client", subject).unwrap();
    let revision = store.request_protection_rows(1001, None, 10).await.unwrap()
        .into_iter().find(|row| row.subject_key == key).unwrap().revision;
    assert!(accounting.clear(&key, &revision, 1002).await.unwrap());
    let next = accounting.probe_decision("client", subject, &p, 1002).await;
    assert!(!next.decision.enforced && !next.storage_failed, "admitted right after the unblock");
}

/// A protection writer outside this instance (a synchronous process, a second
/// write-behind process, a tool using the store functions) joins the witness
/// before its write: the owner gives up ownership first and marks its entries
/// for re-reading, so the block that writer records refuses the owner's next
/// request (see also `review_r4_…`, where the entry is fresher than an interval).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_external_writer_ends_ownership_before_its_block_is_written() {
    let dir = tempfile::tempdir().unwrap();
    let path = file_store_path(&dir);
    let key = ServerKey::generate();
    let store = Store::open(&path, key.clone()).await.unwrap();
    let accounting = WriteBehind::start(store.clone(), config(50, 4096));
    settle_ownership(&accounting, true).await;
    let mut p = policy();
    p.rate_limit = 1000;
    assert!(!accounting.record("client", "192.0.2.60", &p, 1000, None, NONE).await.decision.enforced);
    tokio::time::sleep(Duration::from_millis(120)).await;
    assert!(accounting.status().owned);
    // Another "process": its own Store handle on the same file, synchronous path.
    let other = Store::open(&path, key).await.unwrap();
    let blocked = other
        .request_protection("client", "192.0.2.60", &p, 1001, Some(Signal::ProbePath))
        .await
        .unwrap();
    assert!(blocked.decision.enforced);
    let status = accounting.status();
    assert!(!status.owned && status.ownership_lost >= 1, "{status:?}");
    let next = accounting.record("client", "192.0.2.60", &p, 1001, None, NONE).await;
    assert!(next.decision.enforced, "the external block is seen by the next request");
}

/// The witness lost at runtime (lock file replaced): ownership ends at the next
/// tick and the re-read rule applies until the instance joins again.
#[tokio::test]
async fn a_lost_witness_falls_back_to_the_reread_rule() {
    let dir = tempfile::tempdir().unwrap();
    let path = file_store_path(&dir);
    let store = Store::open(&path, ServerKey::generate()).await.unwrap();
    let accounting = WriteBehind::start(store.clone(), config(50, 4096));
    settle_ownership(&accounting, true).await;
    let lock = format!("{path}.mw-request-protection.lock");
    if std::fs::remove_file(&lock).is_err() {
        eprintln!("[write_behind] the held lock file cannot be removed on this platform; replacement not exercised");
        return;
    }
    if std::fs::write(&lock, b"").is_err() {
        eprintln!("[write_behind] a removed lock file cannot be re-created yet on this platform; not exercised");
        return;
    }
    settle_ownership(&accounting, false).await;
    // It joins again with the new file and, alone, regains ownership.
    settle_ownership(&accounting, true).await;
    let status = accounting.status();
    assert!(status.ownership_lost >= 1 && status.ownership_gained >= 2, "{status:?}");
}

/// Acceptance 3: a crash during a flush interval. The runtime is dropped, which
/// stops the flusher, without the shutdown flush (a real kill -9 is not
/// practical in a unit test). After restart the stored count is behind by
/// exactly the queued ordinary observations, which were at most what was made
/// in the last two intervals, and every transition answered to a client is present.
#[test]
fn crash_loses_at_most_one_interval_of_ordinary_observations_and_no_transition() {
    let dir = tempfile::tempdir().unwrap();
    let path = file_store_path(&dir);
    let key = ServerKey::generate();
    let (made, pending, answered_blocks, recent) = {
        let (path, key) = (path.clone(), key.clone());
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
            let result = runtime.block_on(async move {
                let store = Store::open(&path, key).await.unwrap();
                let accounting = WriteBehind::start(store.clone(), config(200, 65536));
                let mut p = policy();
                p.rate_limit = 100_000;
                let started = Instant::now();
                let (mut made, mut stamps, mut blocks) = (0u64, Vec::new(), 0u64);
                while started.elapsed() < Duration::from_millis(900) {
                    accounting.record("account", "crash", &p, 1000, None, NONE).await;
                    made += 1;
                    stamps.push(Instant::now());
                    if made % 50 == 0 {
                        let ip = format!("probe-path:192.0.2.{}", 100 + blocks);
                        let hit = accounting.record("client", &ip, &p, 1000, Some(Signal::ProbePath), NONE).await;
                        assert!(hit.decision.enforced && !hit.storage_failed);
                        blocks += 1;
                    }
                    tokio::time::sleep(Duration::from_millis(2)).await;
                }
                let pending = accounting.status().pending as u64;
                // A probe-path write-through also drains what was queued before it.
                let window = Instant::now() - Duration::from_millis(2 * 200 + 50);
                let recent = stamps.iter().filter(|at| **at >= window).count() as u64;
                (made, pending, blocks, recent)
            });
            drop(runtime); // the crash: the flusher stops, nothing more is flushed
            result
        })
        .join()
        .unwrap()
    };
    let reopened = tokio::runtime::Runtime::new().unwrap();
    reopened.block_on(async move {
        let store = Store::open(&path, key).await.unwrap();
        let stored = u64::from(rate_count(&store, "account", "crash", 1000).await.unwrap());
        assert_eq!(made - stored, pending, "lost exactly the queued observations");
        assert!(pending <= recent, "queued {pending} > made in the last two intervals {recent}");
        let events = store.request_protection_events(1000, None, 250).await.unwrap();
        let recorded = events.iter().filter(|event| event.reason == "probe-path" && event.enforced).count() as u64;
        assert_eq!(recorded, answered_blocks, "every answered transition survives");
        eprintln!("[write_behind] crash: made={made} stored={stored} lost={pending} made-in-last-two-intervals={recent}");
    });
}

/// Acceptance 4: the shutdown flush commits the whole queue.
#[tokio::test]
async fn shutdown_flush_commits_every_queued_observation() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&file_store_path(&dir), ServerKey::generate()).await.unwrap();
    let runtime = super::super::Runtime::start_with(store.clone(), config(5000, 4096));
    let accounting = runtime.write_behind().unwrap();
    let mut p = policy();
    p.rate_limit = 100;
    for _ in 0..25 {
        accounting.record("account", "shutdown", &p, 1000, None, NONE).await;
    }
    assert_eq!(accounting.status().pending, 24);
    runtime.flush_all(tokio::time::Instant::now() + Duration::from_secs(5)).await.unwrap();
    assert_eq!(accounting.status().pending, 0);
    assert_eq!(rate_count(&store, "account", "shutdown", 1000).await, Some(25));
}

/// Acceptance 5: two processes (two Store handles and two accounting instances
/// on one file) never own; each re-reads after an interval, and over-admission
/// stays within (processes - 1) x rate x 2 intervals.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_processes_without_ownership_over_admit_within_the_bound() {
    let dir = tempfile::tempdir().unwrap();
    let path = file_store_path(&dir);
    let key = ServerKey::generate();
    let interval = Duration::from_millis(100);
    let limit = 60u32;
    let mut p = policy();
    p.rate_limit = limit;
    p.rate_window_secs = 3600;
    let stores = [Store::open(&path, key.clone()).await.unwrap(), Store::open(&path, key).await.unwrap()];
    let processes: Vec<_> = stores.iter().map(|store| WriteBehind::start(store.clone(), config(100, 4096))).collect();
    // Each joins at its first check; with two participants neither owns.
    tokio::time::sleep(Duration::from_millis(300)).await;
    for process in &processes {
        settle_ownership(process, false).await;
    }
    let started = Instant::now();
    let tasks: Vec<_> = processes
        .iter()
        .map(|process| {
            let (process, p) = (process.clone(), p.clone());
            tokio::spawn(async move {
                let (mut made, mut admitted) = (0u64, 0u64);
                while started.elapsed() < Duration::from_millis(1500) {
                    let answer = process.record("account", "shared", &p, 1000, None, NONE).await;
                    assert!(!answer.storage_failed);
                    made += 1;
                    admitted += u64::from(!answer.decision.enforced);
                    assert!(!process.status().owned, "never owned with a second participant");
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
                (made, admitted)
            })
        })
        .collect();
    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await.unwrap());
    }
    let elapsed = started.elapsed().as_secs_f64();
    let admitted: u64 = results.iter().map(|(_, admitted)| admitted).sum();
    let rate = results.iter().map(|(made, _)| *made as f64 / elapsed).fold(0.0, f64::max);
    let bound = (rate * 2.0 * interval.as_secs_f64()).ceil() as u64 + 1;
    eprintln!("[write_behind] two processes: admitted={admitted} limit={limit} rate/process={rate:.0}/s bound=+{bound}");
    assert!(admitted >= u64::from(limit), "the limit itself is admitted");
    assert!(admitted <= u64::from(limit) + bound, "over-admission {} > bound {bound}", admitted - u64::from(limit));
}

/// CredentialRejected and Malformed are never deferred.
#[tokio::test]
async fn credential_rejected_and_malformed_are_written_through() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let accounting = WriteBehind::new(store.clone(), config(5000, 4096));
    let mut p = policy();
    p.rate_limit = 100;
    p.short.absolute_anomalies = 100;
    p.long.absolute_anomalies = 100;
    p.short.ratio_percent = 100;
    p.long.ratio_percent = 100;
    accounting.record("client", "192.0.2.70", &p, 1000, None, NONE).await;
    accounting.record("client", "192.0.2.70", &p, 1000, Some(Signal::Normal), NONE).await;
    assert_eq!(accounting.status().pending, 1, "an ordinary completion is queued");
    for signal in [Signal::CredentialRejected, Signal::Malformed] {
        let before = accounting.status().write_throughs;
        accounting.record("client", "192.0.2.70", &p, 1000, Some(signal), NONE).await;
        let status = accounting.status();
        assert_eq!(status.write_throughs, before + 1, "{signal:?}");
        assert_eq!(status.pending, 0, "{signal:?} and everything before it committed");
    }
    let key = store.request_protection_subject("client", "192.0.2.70").unwrap();
    let row = store.request_protection_load(&key, "client", 1000).await.unwrap().unwrap();
    assert_eq!(row.state.snapshot().short.anomalous, 2);
}

/// Memory stays bounded under ownership too (no entry is dropped as "old").
#[tokio::test]
async fn owned_cache_stays_bounded() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let accounting = WriteBehind::new(store.clone(), config(5000, 4096));
    accounting.check_ownership().await;
    assert!(accounting.status().owned, "an in-memory database is private to its pool");
    let p = policy();
    for i in 0..(SUBJECT_CAPACITY as usize + 500) {
        let ip = format!("10.{}.{}.{}", i >> 16, (i >> 8) & 255, i & 255);
        accounting.probe_decision("client", &format!("probe-path:{ip}"), &p, 1000).await;
    }
    let status = accounting.status();
    assert!(status.cache_size <= SUBJECT_CAPACITY as usize && status.evictions > 0, "{status:?}");
}

/// Acceptance 6: sign-in failure throttling is unaffected by the accounting
/// mode: the same responses, the same source locks, the same protection rows.
#[test]
fn sign_in_failure_throttling_is_the_same_with_and_without_write_behind() {
    let run = |mode: Option<AccountingConfig>| {
        std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                TEST_CONFIG.with(|cell| cell.set(mode));
                tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(sign_in_failures())
            })
            .unwrap()
            .join()
            .unwrap()
    };
    let synchronous = run(None);
    let deferred = run(Some(config(250, 4096)));
    eprintln!("[write_behind] sign-in failures: {synchronous:?}");
    assert_eq!(synchronous, deferred);
    assert!(!synchronous.1.is_empty(), "the source failure lock engaged: {synchronous:?}");
}

/// Wrong-password sign-ins from one address against the proxy-mode router.
/// Returns (statuses, source-lock failure counts, protection rows).
async fn sign_in_failures() -> (Vec<u16>, Vec<(String, usize)>, Vec<Value>) {
    use axum::body::Body;
    use axum::http::Request as HttpRequest;
    use tower::ServiceExt;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let upstream = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, mw_mock_jmap::router()).await.unwrap() });
    let dir = tempfile::tempdir().unwrap();
    let db = file_store_path(&dir);
    let key = ServerKey::generate();
    let store = Store::open(&db, key.clone()).await.unwrap();
    for (name, value) in [
        ("signin.retryCooldownSecs", json!(0)),
        ("security.protection.rateMode", json!("enforce")),
        ("security.protection.adaptiveMode", json!("enforce")),
    ] {
        store.put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: name.into(), global_value: Some(value.to_string()), allow_user_override: false,
            global_locked: false, updated_by: "fixture".into(), updated_at: 1,
        }).await.unwrap();
    }
    let (router, _) = crate::build_app_full(
        crate::AppConfig {
            db_path: db.clone(), server_key_hex: Some(key.to_hex()), web_dir: None, cookie_secure: false,
            mode: crate::ServerMode::Proxy, hardening: Default::default(),
            security: crate::SecurityConfig {
                upstream_default: Some(upstream.clone()), jmap_upstreams: Some(vec![upstream]), ..Default::default()
            },
        },
        crate::V6Config { admin_enabled: false, ..Default::default() },
    ).await.unwrap();
    let mut statuses = Vec::new();
    for _ in 0..12 {
        let mut request = HttpRequest::builder()
            .method("POST").uri("/api/login").header("host", "localhost")
            .header("content-type", "application/json").header("user-agent", "Mozilla/5.0")
            .body(Body::from(json!({"username": mw_mock_jmap::USER, "password": "wrong-password"}).to_string()))
            .unwrap();
        request.extensions_mut().insert(axum::extract::ConnectInfo("127.0.0.1:41000".parse::<std::net::SocketAddr>().unwrap()));
        statuses.push(router.clone().oneshot(request).await.unwrap().status().as_u16());
    }
    // Write-behind: queued completions are flushed within one interval.
    tokio::time::sleep(Duration::from_millis(700)).await;
    let now = chrono::Utc::now().timestamp();
    let locks = store.list_auth_source_locks(now).await.unwrap()
        .into_iter().map(|lock| (lock.surface, lock.failures)).collect();
    let mut rows: Vec<Value> = store.request_protection_rows(now, None, 250).await.unwrap().into_iter()
        .map(|row| json!({"kind": row.subject_kind, "rate": row.snapshot.rate_count,
            "anomalous": row.snapshot.short.anomalous, "stage": row.snapshot.stage}))
        .collect();
    rows.sort_by_key(Value::to_string);
    (statuses, locks, rows)
}

// ---------------------------------------------------------------------------
// t50hp-e15a review fixes.
// ---------------------------------------------------------------------------

/// R4: when ownership ends, an entry synced less than one interval ago is
/// re-read on its next use, so a block written right after the owner's
/// acknowledgement refuses the next request. (Failed before the fix.)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn review_r4_losing_ownership_rereads_recently_synced_entries() {
    let dir = tempfile::tempdir().unwrap();
    let path = file_store_path(&dir);
    let key = ServerKey::generate();
    let store = Store::open(&path, key.clone()).await.unwrap();
    // No flusher: the test runs every ownership check itself.
    let owner = Arc::new(WriteBehind::new(store.clone(), config(5000, 4096)));
    owner.check_ownership().await;
    assert!(owner.status().owned);
    let mut p = policy();
    p.rate_limit = 1000;
    assert!(!owner.record("client", "192.0.2.81", &p, 1000, None, NONE).await.decision.enforced);
    let other = Store::open(&path, key).await.unwrap();
    let writer = tokio::spawn({
        let p = p.clone();
        async move { other.request_protection("client", "192.0.2.81", &p, 1001, Some(Signal::ProbePath)).await }
    });
    tokio::time::sleep(Duration::from_millis(150)).await;
    owner.check_ownership().await; // acknowledges the joiner, ends ownership
    assert!(!owner.status().owned);
    assert!(writer.await.unwrap().unwrap().decision.enforced);
    // The entry was synced well within the 5 s interval.
    let next = owner.record("client", "192.0.2.81", &p, 1001, None, NONE).await;
    assert!(next.decision.enforced, "re-read after losing ownership");
}

/// R5: a full queue does not stop counting. With healthy storage the request
/// is counted (written through) and answered by its decision. (Failed before
/// the fix: the request was answered as a storage failure and not counted.)
#[tokio::test]
async fn review_r5_a_full_queue_still_counts_every_request() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let accounting = WriteBehind::new(store.clone(), config(5000, 64));
    let mut p = policy();
    p.rate_limit = 1000;
    for _ in 0..65 {
        accounting.record("account", "full", &p, 1000, None, NONE).await;
    }
    assert_eq!(accounting.status().pending, 64, "the queue is full");
    for _ in 0..10 {
        let answer = accounting.record("account", "full", &p, 1000, None, NONE).await;
        assert!(!answer.storage_failed && !answer.decision.enforced, "{answer:?}");
    }
    accounting.flush_pending().await.unwrap();
    assert_eq!(accounting.status().dropped, 0);
    assert_eq!(rate_count(&store, "account", "full", 1000).await, Some(75), "every request counted");
}

/// R5 with healthy storage under both storage-failure settings: a full queue
/// never answers 503 and never admits uncounted; the limit holds exactly.
#[tokio::test]
async fn review_r5_full_queue_answers_by_decision_under_both_settings() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let accounting = WriteBehind::new(store.clone(), config(5000, 64));
    let mut p = policy();
    p.rate_limit = 70;
    for _ in 0..65 {
        accounting.record("account", "both", &p, 1000, None, NONE).await;
    }
    let mut statuses = Vec::new();
    for _ in 0..10 {
        let answer = accounting.record("account", "both", &p, 1000, None, NONE).await;
        assert!(!answer.storage_failed);
        let (closed, open) = (guard_status(answer, true, &p), guard_status(answer, false, &p));
        assert_eq!(closed, open, "the setting does not matter with healthy storage");
        statuses.push(closed.map(|status| status.as_u16()));
    }
    // 65 + 5 admitted reach the limit of 70; the next 5 are refused (counted).
    assert_eq!(statuses, [None, None, None, None, None, Some(429), Some(429), Some(429), Some(429), Some(429)]);
    // The first full-queue write-through drained the queue; later ones were queued.
    assert!(accounting.status().backpressure_writes >= 1);
}

/// R2: the same database opened through another name shares one witness, so
/// two instances never both own: a directory junction (Windows) or symbolic
/// link (Unix), and another spelling of the file name (case on Windows, a
/// file symbolic link on Unix). The store resolves only the parent directory;
/// the witness now resolves the whole path.
#[tokio::test]
async fn review_r2_an_aliased_path_shares_the_witness() {
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("real");
    std::fs::create_dir(&real).unwrap();
    let alias = dir.path().join("alias");
    #[cfg(windows)]
    let linked = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&alias)
        .arg(&real)
        .output()
        .is_ok_and(|output| output.status.success());
    #[cfg(unix)]
    let linked = std::os::unix::fs::symlink(&real, &alias).is_ok();
    #[cfg(not(any(windows, unix)))]
    let linked = false;
    let key = ServerKey::generate();
    let direct = Store::open(real.join("aliased.db").to_str().unwrap(), key.clone()).await.unwrap();
    let mut names = Vec::new();
    if linked {
        names.push(alias.join("aliased.db"));
    } else {
        eprintln!("[write_behind] no directory alias could be created; that case is not exercised");
    }
    #[cfg(windows)]
    names.push(real.join("ALIASED.db"));
    #[cfg(unix)]
    {
        let file_link = real.join("other-name.db");
        if std::os::unix::fs::symlink(real.join("aliased.db"), &file_link).is_ok() {
            names.push(file_link);
        }
    }
    let first = WriteBehind::new(direct, config(250, 4096));
    first.check_ownership().await;
    assert!(first.status().owned);
    for name in names {
        let through = Store::open(name.to_str().unwrap(), key.clone()).await.unwrap();
        let second = WriteBehind::new(through, config(250, 4096));
        second.check_ownership().await;
        first.check_ownership().await;
        assert!(!(first.status().owned && second.status().owned), "{}: both own", name.display());
        assert!(!first.status().owned && !second.status().owned, "{}: two participants, neither owns", name.display());
        drop(second);
        first.check_ownership().await;
        assert!(first.status().owned, "alone again");
    }
}

/// R3: the owner's deadline is counted from before its check, so a stall
/// between the check and storing the deadline cannot extend trust past the
/// joiner's wait. Here the owner stalls longer than the joiner waits: the
/// joiner writes and the owner is not owned while it does, nor after the stall;
/// its next check sees the participant and the block is enforced.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn review_r3_a_stalled_check_cannot_extend_ownership_past_the_joiners_wait() {
    let dir = tempfile::tempdir().unwrap();
    let path = file_store_path(&dir);
    let key = ServerKey::generate();
    let store = Store::open(&path, key.clone()).await.unwrap();
    let owner = Arc::new(WriteBehind::new(store.clone(), config(250, 4096)));
    owner.check_ownership().await;
    assert!(owner.status().owned);
    let mut p = policy();
    p.rate_limit = 1000;
    assert!(!owner.record("client", "192.0.2.82", &p, 1000, None, NONE).await.decision.enforced);
    // Window 500 ms; the joiner waits 500 ms + 2 s. The owner stalls 3.5 s.
    owner.stall_after_check(Some(Duration::from_millis(3500)));
    let stalled = tokio::spawn({
        let owner = owner.clone();
        async move { owner.check_ownership().await }
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let other = Store::open(&path, key).await.unwrap();
    let started = Instant::now();
    let blocked = other.request_protection("client", "192.0.2.82", &p, 1001, Some(Signal::ProbePath)).await.unwrap();
    assert!(blocked.decision.enforced);
    assert!(started.elapsed() >= Duration::from_secs(2), "the joiner waited for an acknowledgement");
    assert!(!owner.owned(), "no trust while the joiner writes");
    stalled.await.unwrap();
    assert!(!owner.owned(), "the stalled check stored an already past deadline");
    owner.stall_after_check(None);
    owner.check_ownership().await;
    let next = owner.record("client", "192.0.2.82", &p, 1001, None, NONE).await;
    assert!(next.decision.enforced, "{:?}", owner.status());
}

fn set_read_only(path: &str, read_only: bool) {
    let mut permissions = std::fs::metadata(path).unwrap().permissions();
    permissions.set_readonly(read_only);
    std::fs::set_permissions(path, permissions).unwrap();
}

/// R1: a synchronous writer that cannot join the witness (here: the lock file is
/// not writable for it) still writes, and the owner must not keep serving the
/// subject from its cache: the write carries the database marker, the owner's
/// next check sees it, and the block is enforced within one flush interval.
/// (Failed before the fix: the block was ignored for the row's lifetime.)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn review_r1_a_writer_that_cannot_join_is_still_seen_by_the_owner() {
    let dir = tempfile::tempdir().unwrap();
    let path = file_store_path(&dir);
    let key = ServerKey::generate();
    let store = Store::open(&path, key.clone()).await.unwrap();
    let accounting = WriteBehind::start(store.clone(), config(50, 4096));
    settle_ownership(&accounting, true).await;
    let mut p = policy();
    p.rate_limit = 1000;
    assert!(!accounting.record("client", "192.0.2.80", &p, 1000, None, NONE).await.decision.enforced);
    tokio::time::sleep(Duration::from_millis(120)).await;
    let canonical = std::fs::canonicalize(&path).unwrap();
    let lock = format!("{}.mw-request-protection.lock", canonical.display());
    set_read_only(&lock, true);
    let other = Store::open(&path, key).await.unwrap();
    let blocked = other
        .request_protection("client", "192.0.2.80", &p, 1001, Some(Signal::ProbePath))
        .await;
    set_read_only(&lock, false);
    assert!(blocked.unwrap().decision.enforced);
    // One flush interval (and a margin) later the owner answers from the row.
    tokio::time::sleep(Duration::from_millis(200)).await;
    let next = accounting.record("client", "192.0.2.80", &p, 1001, None, NONE).await;
    assert!(next.decision.enforced, "the unjoined writer's block must be seen: {:?}", accounting.status());
    assert!(accounting.status().unjoined_writes_seen >= 1);
}

/// R1: the join counters are in the load-fixture snapshot (and the admin status).
#[tokio::test]
async fn review_r1_witness_status_is_reported() {
    let snapshot = fixture_snapshot();
    for field in ["joins", "joinFailures", "transientFailures", "unjoinedWrites"] {
        assert!(snapshot["writerJoins"][field].is_u64(), "{field}: {snapshot}");
    }
}

// Real OS processes: the test binary runs itself as a child with one of the
// roles below (MW_E8_CHILD_ROLE), on a database named by MW_E8_CHILD_DB.

const CHILD_TEST: &str = "request_protection::write_behind::tests::wb_child_process";

fn child(role: &str, db: &str, key: &ServerKey) -> std::process::Child {
    std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env("MW_E8_CHILD_ROLE", role)
        .env("MW_E8_CHILD_DB", db)
        .env("MW_E8_CHILD_KEY", key.to_hex())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap()
}

/// Child entry point; a no-op unless started by `child`.
#[test]
fn wb_child_process() {
    let Ok(role) = std::env::var("MW_E8_CHILD_ROLE") else { return };
    let db = std::env::var("MW_E8_CHILD_DB").unwrap();
    let key = ServerKey::from_hex(&std::env::var("MW_E8_CHILD_KEY").unwrap()).unwrap();
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    runtime.block_on(async move {
        use std::io::Write;
        let say = |line: String| {
            let mut out = std::io::stdout().lock();
            let _ = writeln!(out, "E8CHILD {line}");
            let _ = out.flush();
        };
        let store = Store::open(&db, key).await.unwrap();
        match role.as_str() {
            "bound" => {
                let accounting = WriteBehind::start(store, config(100, 4096));
                let mut p = policy();
                p.rate_limit = 60;
                p.rate_window_secs = 3600;
                tokio::time::sleep(Duration::from_millis(400)).await;
                say("READY".into());
                let started = Instant::now();
                let (mut made, mut admitted, mut owned) = (0u64, 0u64, false);
                while started.elapsed() < Duration::from_millis(1500) {
                    let answer = accounting.record("account", "shared", &p, 1000, None, NONE).await;
                    made += 1;
                    admitted += u64::from(!answer.decision.enforced && !answer.storage_failed);
                    owned |= accounting.status().owned;
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
                say(format!("DONE made={made} admitted={admitted} owned={owned}"));
            }
            "kill" => {
                let accounting = WriteBehind::start(store, config(200, 65536));
                let mut p = policy();
                p.rate_limit = 1_000_000;
                let mut made = 0u64;
                loop {
                    accounting.record("account", "killed", &p, 1000, None, NONE).await;
                    made += 1;
                    say(format!("MADE {made}"));
                    if made % 40 == 0 {
                        let ip = format!("probe-path:198.51.100.{}", made / 40);
                        let hit = accounting.record("client", &ip, &p, 1000, Some(Signal::ProbePath), NONE).await;
                        if hit.decision.enforced && !hit.storage_failed {
                            say(format!("BLOCK {ip}"));
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(3)).await;
                }
            }
            other => panic!("unknown role {other}"),
        }
    });
}

/// Two real OS processes on one database: neither owns, and over-admission
/// stays within (processes - 1) x rate x 2 intervals.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_os_processes_never_own_and_stay_within_the_bound() {
    use std::io::BufRead;
    let dir = tempfile::tempdir().unwrap();
    let path = file_store_path(&dir);
    let key = ServerKey::generate();
    let store = Store::open(&path, key.clone()).await.unwrap();
    let accounting = WriteBehind::start(store.clone(), config(100, 4096));
    let mut p = policy();
    p.rate_limit = 60;
    p.rate_window_secs = 3600;
    // Create the row before either process charges it.
    store.request_protection("account", "shared", &p, 1000, None).await.unwrap();
    let mut process = child("bound", &path, &key);
    let stdout = process.stdout.take().unwrap();
    let (lines, receiver) = std::sync::mpsc::channel::<String>();
    std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout).lines().map_while(Result::ok) {
            // libtest may print its own text before ours on the same line.
            if let Some(line) = line.find("E8CHILD ").map(|at| &line[at + 8..]) {
                let _ = lines.send(line.to_owned());
            }
        }
    });
    let ready = tokio::task::spawn_blocking({
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(receiver));
        move || {
            let receiver = receiver.lock().unwrap();
            let ready = receiver.recv_timeout(Duration::from_secs(60)).unwrap();
            assert_eq!(ready, "READY");
            receiver.recv_timeout(Duration::from_secs(60)).unwrap()
        }
    });
    tokio::time::sleep(Duration::from_millis(500)).await;
    let started = Instant::now();
    let (mut made, mut admitted, mut owned) = (0u64, 1u64, false);
    while started.elapsed() < Duration::from_millis(1500) {
        let answer = accounting.record("account", "shared", &p, 1000, None, NONE).await;
        made += 1;
        admitted += u64::from(!answer.decision.enforced && !answer.storage_failed);
        owned |= accounting.status().owned;
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let done = ready.await.unwrap();
    assert!(process.wait().unwrap().success());
    let field = |name: &str| -> String {
        done.split_whitespace().find_map(|part| part.strip_prefix(&format!("{name}=")).map(str::to_owned)).unwrap()
    };
    let (child_made, child_admitted): (u64, u64) = (field("made").parse().unwrap(), field("admitted").parse().unwrap());
    assert_eq!(field("owned"), "false", "the child never owned");
    assert!(!owned, "the parent never owned");
    let total = admitted + child_admitted;
    let rate = (made.max(child_made) as f64) / 1.5;
    let bound = (rate * 2.0 * 0.1).ceil() as u64 + 1;
    eprintln!("[write_behind] two OS processes: admitted={total} limit=60 rate/process={rate:.0}/s bound=+{bound}");
    assert!(total >= 60 && total <= 60 + bound, "admitted {total}, bound 60 + {bound}");
}

/// A real kill (TerminateProcess / SIGKILL) of a child mid-interval: every
/// block it answered is in the database, and the ordinary observations lost
/// are at most what it made in the last two intervals.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_killed_process_loses_no_answered_block_and_at_most_two_intervals() {
    use std::io::BufRead;
    let dir = tempfile::tempdir().unwrap();
    let path = file_store_path(&dir);
    let key = ServerKey::generate();
    Store::open(&path, key.clone()).await.unwrap().close().await;
    let mut process = child("kill", &path, &key);
    let stdout = process.stdout.take().unwrap();
    let (lines, receiver) = std::sync::mpsc::channel::<(Instant, String)>();
    std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout).lines().map_while(Result::ok) {
            // libtest may print its own text before ours on the same line.
            if let Some(line) = line.find("E8CHILD ").map(|at| &line[at + 8..]) {
                let _ = lines.send((Instant::now(), line.to_owned()));
            }
        }
    });
    let (mut blocks, mut made) = (Vec::new(), Vec::new());
    let deadline = Instant::now() + Duration::from_secs(120);
    while blocks.len() < 4 && Instant::now() < deadline {
        let (at, line) = receiver.recv_timeout(Duration::from_secs(60)).unwrap();
        if let Some(ip) = line.strip_prefix("BLOCK ") {
            blocks.push(ip.to_owned());
        } else if let Some(n) = line.strip_prefix("MADE ") {
            made.push((at, n.parse::<u64>().unwrap()));
        }
    }
    assert_eq!(blocks.len(), 4);
    // Mid-interval: some ordinary observations after the last block.
    tokio::time::sleep(Duration::from_millis(130)).await;
    process.kill().unwrap(); // by this child's handle
    let killed = Instant::now();
    process.wait().unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await; // the reader forwards the last lines
    while let Ok((at, line)) = receiver.try_recv() {
        if let Some(n) = line.strip_prefix("MADE ") {
            made.push((at, n.parse::<u64>().unwrap()));
        } else if let Some(ip) = line.strip_prefix("BLOCK ") {
            blocks.push(ip.to_owned());
        }
    }
    let store = Store::open(&path, key).await.unwrap();
    let events = store.request_protection_events(1000, None, 250).await.unwrap();
    for ip in &blocks {
        let subject = store.request_protection_subject("client", ip).unwrap();
        assert!(events.iter().any(|event| event.subject_key == subject && event.reason == "probe-path"), "{ip} answered but missing");
    }
    let last = made.last().unwrap().1;
    let stored = u64::from(rate_count(&store, "account", "killed", 1000).await.unwrap());
    let window = Duration::from_millis(2 * 200 + 150);
    let recent = made.iter().filter(|(at, _)| killed.duration_since(*at) <= window).count() as u64;
    eprintln!("[write_behind] kill: made>={last} stored={stored} lost<={} recent={recent} blocks={}", last.saturating_sub(stored), blocks.len());
    assert!(stored <= last + 1, "nothing counted that was not made");
    assert!(last.saturating_sub(stored) <= recent, "lost {} > made in the last two intervals {recent}", last - stored);
}

/// The ownership check runs on its own timer and does not wait for the flush
/// lock: a flush held for many ownership windows (a long drain, a blocked
/// writer) does not let ownership lapse. Before this, the check ran after the
/// flush in the same loop and took the flush lock, so a slow flush lapsed
/// ownership and the next check marked every cached subject for re-reading.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_slow_flush_does_not_lapse_ownership() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&file_store_path(&dir), ServerKey::generate()).await.unwrap();
    let accounting = WriteBehind::start(store.clone(), config(50, 4096));
    settle_ownership(&accounting, true).await;
    let mut p = policy();
    p.rate_limit = 1000;
    accounting.record("account", "slow", &p, 1000, None, NONE).await;
    let gained = accounting.status().ownership_gained;
    let held = accounting.flushing.lock().await; // a flush that does not finish
    for _ in 0..10 {
        tokio::time::sleep(Duration::from_millis(60)).await; // > one 100 ms window in total
        assert!(accounting.status().owned, "{:?}", accounting.status());
    }
    drop(held);
    let status = accounting.status();
    assert_eq!((status.ownership_gained, status.ownership_lost), (gained, 0), "{status:?}");
    // Still answered from memory.
    tokio::time::sleep(Duration::from_millis(120)).await;
    let (_, sql) = measure_queries(accounting.record("account", "slow", &p, 1000, None, NONE)).await;
    assert_eq!(statements(&sql), (0, 0, 0), "{sql:?}");
}
