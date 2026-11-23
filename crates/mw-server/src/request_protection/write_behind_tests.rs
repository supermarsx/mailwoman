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

#[tokio::test]
async fn full_queue_is_a_storage_failure_that_still_refuses_known_abuse() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let deferred = WriteBehind::new(store.clone(), config(5000, 64));
    let mut p = policy();
    p.rate_limit = 1000;
    let blocked = deferred.record("client", "192.0.2.5", &p, 1000, Some(Signal::ProbePath), NONE).await;
    assert!(blocked.decision.enforced);
    for _ in 0..64 {
        assert!(!deferred.record("account", "a", &p, 1000, None, NONE).await.storage_failed);
    }
    assert_eq!(deferred.status().pending, 63, "first charge was written through");
    deferred.record("account", "a", &p, 1000, None, NONE).await;
    assert_eq!(deferred.status().pending, 64);
    let full = deferred.record("account", "a", &p, 1000, None, NONE).await;
    assert!(full.storage_failed && !full.decision.enforced);
    let refused = deferred.record("client", "192.0.2.5", &p, 1001, None, NONE).await;
    assert!(refused.storage_failed && refused.decision.enforced);
    assert_eq!(deferred.status().dropped, 2);
    deferred.flush_pending().await.unwrap();
    assert_eq!(rate_count(&store, "account", "a", 1000).await, Some(65));
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
