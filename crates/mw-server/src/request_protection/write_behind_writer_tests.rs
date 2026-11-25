//! t50hp-e19, review t50hp-e15b G1 and G3: write-behind accounting with the
//! SQLite group writer. G3 runs in every mode; G1 needs `MW_SQLITE_WRITER=group`
//! (the store's mode is read once per process) and is skipped otherwise.
use super::*;
use mw_store::{
    ServerKey,
    request_protection_model::{Mode, WindowPolicy},
};
use std::future::{Future, poll_fn};
use std::task::Poll;

fn policy(rate_limit: u32) -> Policy {
    Policy {
        rate_mode: Mode::Enforce,
        adaptive_mode: Mode::Enforce,
        rate_limit,
        rate_window_secs: 3600,
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
fn config() -> AccountingConfig {
    AccountingConfig { mode: AccountingMode::WriteBehind, flush_every: Duration::from_millis(5000), max_pending: 4096 }
}
const NONE: ProtectionSource<'static> = ProtectionSource { country: None, ip: None };

fn group_mode() -> bool {
    mw_store::parse_sqlite_writer_mode(std::env::var("MW_SQLITE_WRITER").ok().as_deref())
        == Ok(mw_store::SqliteWriterMode::Group)
}

/// Rows (state only) and audit events of a store, for a differential check.
async fn inventory(store: &Store, now: i64) -> (Vec<String>, Vec<String>) {
    let mut rows: Vec<String> = store
        .request_protection_rows(now, None, 250)
        .await
        .unwrap()
        .into_iter()
        .map(|row| format!("{} {} {}", row.subject_key, row.subject_kind, serde_json::json!(row.snapshot)))
        .collect();
    rows.sort();
    let mut events: Vec<String> = store
        .request_protection_events(now, None, 250)
        .await
        .unwrap()
        .into_iter()
        .map(|event| format!("{} {} {} {}", event.subject_key, event.observed_at, event.reason, event.action))
        .collect();
    events.sort();
    (rows, events)
}

/// G3: a write-through whose request is cancelled at any await point (also
/// after its group was handed to the group writer) is booked once: the next
/// drain books an orphaned group instead of writing it again. Rows and audit
/// events equal the synchronous path applied to the observations that were
/// recorded.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn g3_a_cancelled_write_through_is_booked_exactly_once() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("g3.db");
    let key = ServerKey::generate();
    let store = Store::open(path.to_str().unwrap(), key.clone()).await.unwrap();
    let reference = Store::open_in_memory(key).await.unwrap();
    let accounting = WriteBehind::new(store.clone(), config());
    let p = policy(1_000_000);
    let mut cancelled = 0;
    for polls in 1..=60u32 {
        let now = 1000 + i64::from(polls);
        let before = accounting.next_seq.load(Ordering::SeqCst);
        // Malformed: a typed write-through (normal lane).
        let mut call = Box::pin(accounting.record("account", "g3", &p, now, Some(Signal::Malformed), NONE));
        let mut finished = false;
        for _ in 0..polls {
            if let Poll::Ready(_) = poll_fn(|cx| Poll::Ready(call.as_mut().poll(cx))).await {
                finished = true;
                break;
            }
            tokio::task::yield_now().await;
        }
        drop(call);
        cancelled += u32::from(!finished);
        if accounting.next_seq.load(Ordering::SeqCst) != before {
            reference
                .request_protection_with_source("account", "g3", &p, now, Some(Signal::Malformed), NONE)
                .await
                .unwrap();
        }
    }
    accounting.flush_pending().await.unwrap();
    assert!(cancelled > 0, "no cancellation point was reached");
    assert_eq!(inventory(&store, 2000).await, inventory(&reference, 2000).await);
    store.close().await;
}

/// G1: a transition write-through is not held behind a timer flush that waits
/// in a congested normal lane: the flush escalates to the priority lane while
/// the transition waits for `flushing`.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn g1_a_transition_is_not_held_behind_a_flush_in_a_congested_normal_lane() {
    if !group_mode() {
        mw_test_gate::skip("G1 write-behind priority bound: needs MW_SQLITE_WRITER=group");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("g1.db");
    let store = Store::open(path.to_str().unwrap(), ServerKey::generate()).await.unwrap();
    let accounting = Arc::new(WriteBehind::new(store.clone(), config()));
    let p = policy(3);
    // Rows exist before the congestion (a first row is a normal-lane write-through).
    accounting.record("account", "g1", &p, 1000, None, NONE).await;
    accounting.record("account", "other", &p, 1000, None, NONE).await;
    // One ordinary observation queued for the timer flush.
    accounting.record("account", "other", &p, 1001, None, NONE).await;
    // Congest the normal lane: change-log appends through the group writer.
    let mut backlog = Vec::new();
    for n in 0..1000u32 {
        let store = store.clone();
        backlog.push(tokio::spawn(async move {
            let _ = store.record_changes("congestion", "Email", &[format!("m{n}")], "updated").await;
        }));
    }
    tokio::time::sleep(Duration::from_millis(20)).await;
    let flusher = accounting.clone();
    let flush = tokio::spawn(async move { flusher.flush_pending().await });
    tokio::time::sleep(Duration::from_millis(20)).await;
    // The 2nd and 3rd observations are under the limit; the 4th transitions.
    accounting.record("account", "g1", &p, 1002, None, NONE).await;
    let started = Instant::now();
    let mut answer = accounting.record("account", "g1", &p, 1003, None, NONE).await;
    if !answer.decision.enforced {
        answer = accounting.record("account", "g1", &p, 1004, None, NONE).await;
    }
    let waited = started.elapsed();
    let still_queued = backlog.iter().filter(|task| !task.is_finished()).count();
    assert!(answer.decision.enforced && !answer.storage_failed, "transition not durable");
    assert!(still_queued >= 200, "the transition waited for the normal backlog: {still_queued} left after {waited:?}");
    eprintln!("G1: transition answered after {waited:?}; {still_queued} congestion jobs still pending");
    flush.await.unwrap().unwrap();
    for task in backlog {
        task.await.unwrap();
    }
    store.close().await;
}
