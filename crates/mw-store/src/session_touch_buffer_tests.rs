use super::*;
use crate::{Credentials, ServerKey};
use chrono::Duration as Age;

async fn session(store: &Store, name: &str) -> String {
    store
        .create_session(
            "account-touch",
            &format!("{name}@example.test"),
            "https://mail.example.test/session",
            "https://mail.example.test/api",
            &Credentials {
                username: format!("{name}@example.test"),
                password: "secret".into(),
            },
        )
        .await
        .unwrap()
}

async fn row_last_seen(store: &Store, id: &str) -> Option<String> {
    store
        .get_primary_session_with_activity(id)
        .await
        .ok()
        .and_then(|(_, seen)| seen)
}

async fn set_last_seen(store: &Store, id: &str, at: &str) {
    q("UPDATE sessions SET last_seen=?2 WHERE id=?1")
        .bind(id)
        .bind(at)
        .execute(&store.backend)
        .await
        .unwrap();
}

fn config(max_sessions: usize) -> TouchBufferConfig {
    // A long interval: the tests flush explicitly.
    TouchBufferConfig {
        flush_every: Duration::from_secs(3600),
        max_sessions,
        throttle_secs: 15,
    }
}

/// The coalesced touch on one backend: memory instead of an UPDATE, served to
/// the due test, flushed by a max-merge that never recreates or rewinds a row.
async fn coalesces_and_max_merges(store: &Store) {
    assert!(store.enable_session_touch_buffer(config(1024)));
    let id = session(store, &format!("a{}", uuid::Uuid::new_v4().simple())).await;
    let old = (Utc::now() - Age::seconds(60)).to_rfc3339();
    set_last_seen(store, &id, &old).await;
    assert!(store.session_touch_due(&id, Some(&old), 15, Utc::now()));

    // Due: recorded in memory, no statement at all.
    let (_, queries) =
        crate::pool_telemetry::measure_queries(store.touch_session_throttled(&id, 15)).await;
    assert_eq!(
        (queries.write.calls, queries.acquire.calls),
        (0, 0),
        "{queries:?}"
    );
    assert_eq!(
        row_last_seen(store, &id).await.as_deref(),
        Some(old.as_str()),
        "not written yet"
    );
    // The buffered time is what the due test reads in this process.
    assert!(!store.session_touch_due(&id, Some(&old), 15, Utc::now()));
    let held = store.buffered_last_seen(&id).expect("buffered");
    assert!(held > old);
    // A second touch within the throttle is not due against the buffer.
    store.touch_session_throttled(&id, 15).await.unwrap();
    assert_eq!(
        store.buffered_last_seen(&id).as_deref(),
        Some(held.as_str())
    );
    let stats = store.session_touch_buffer_stats().unwrap();
    assert_eq!((stats.recorded, stats.not_due, stats.pending), (1, 1, 1));

    // Flush: one statement, the row gets the buffered time.
    let (changed, queries) =
        crate::pool_telemetry::measure_queries(store.flush_session_touches()).await;
    assert_eq!(changed.unwrap(), 1);
    assert_eq!(queries.write.calls, 1, "one batch statement: {queries:?}");
    assert_eq!(
        row_last_seen(store, &id).await.as_deref(),
        Some(held.as_str())
    );
    // Flushed entries stay as hints until they age past the throttle.
    assert_eq!(
        store.buffered_last_seen(&id).as_deref(),
        Some(held.as_str())
    );
    assert_eq!(store.session_touch_buffer_stats().unwrap().pending, 0);
    assert_eq!(
        store.flush_session_touches().await.unwrap(),
        0,
        "nothing pending"
    );

    // Max-merge: a newer row value (another process) is never rewound.
    let other = session(store, &format!("b{}", uuid::Uuid::new_v4().simple())).await;
    set_last_seen(store, &other, &old).await;
    store.touch_session_throttled(&other, 15).await.unwrap();
    let future = (Utc::now() + Age::seconds(30)).to_rfc3339();
    set_last_seen(store, &other, &future).await;
    // A deleted session is never recreated by the flush.
    let gone = session(store, &format!("c{}", uuid::Uuid::new_v4().simple())).await;
    set_last_seen(store, &gone, &old).await;
    store.touch_session_throttled(&gone, 15).await.unwrap();
    store.delete_session(&gone).await.unwrap();
    assert_eq!(store.flush_session_touches().await.unwrap(), 0);
    assert_eq!(
        row_last_seen(store, &other).await.as_deref(),
        Some(future.as_str())
    );
    assert!(
        store
            .get_primary_session_with_activity(&gone)
            .await
            .is_err(),
        "still deleted"
    );

    // The idle clock is not touched by any of this.
    let clock = store.session_activity_clock(&id).await.unwrap();
    store.touch_session_throttled(&id, 15).await.unwrap();
    store.flush_session_touches().await.unwrap();
    assert_eq!(store.session_activity_clock(&id).await.unwrap(), clock);
    store.disable_session_touch_buffer().await.unwrap();
    assert!(!store.session_touch_buffered());
}

#[tokio::test]
async fn sqlite_touch_buffer_coalesces_and_max_merges() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    coalesces_and_max_merges(&store).await;
    store.close().await;
}

#[tokio::test]
async fn postgres_touch_buffer_coalesces_and_max_merges() {
    let Ok(url) = std::env::var("MW_TEST_PG") else {
        mw_test_gate::skip("session touch buffer PostgreSQL leg: MW_TEST_PG not configured");
        return;
    };
    let store = Store::open_postgres(&url, ServerKey::generate())
        .await
        .unwrap();
    coalesces_and_max_merges(&store).await;
    store.close().await;
}

/// A managed (file) SQLite database: the flush statement takes the write gate
/// like every other UPDATE, and batches above FLUSH_BATCH split.
#[tokio::test]
async fn sqlite_file_flush_batches_through_the_write_gate() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("touch.db");
    let store = Store::open(path.to_str().unwrap(), ServerKey::generate())
        .await
        .unwrap();
    assert!(store.enable_session_touch_buffer(config(10_000)));
    let old = (Utc::now() - Age::seconds(60)).to_rfc3339();
    let mut ids = Vec::new();
    for n in 0..(FLUSH_BATCH + 7) {
        let id = session(&store, &format!("f{n}")).await;
        set_last_seen(&store, &id, &old).await;
        ids.push(id);
    }
    for id in &ids {
        store.touch_session_throttled(id, 15).await.unwrap();
    }
    let (changed, queries) =
        crate::pool_telemetry::measure_queries(store.flush_session_touches()).await;
    assert_eq!(changed.unwrap(), ids.len() as u64);
    assert_eq!(queries.write.calls, 2, "two batches: {queries:?}");
    for id in &ids {
        assert!(row_last_seen(&store, id).await.unwrap() > old);
    }
    store.disable_session_touch_buffer().await.unwrap();
    store.close().await;
}

/// At the cap a new session's touch is written through, exactly the original
/// UPDATE; flushed hints are dropped first.
#[tokio::test]
async fn at_the_cap_a_new_session_writes_through() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    assert!(store.enable_session_touch_buffer(config(2)));
    let old = (Utc::now() - Age::seconds(60)).to_rfc3339();
    let mut ids = Vec::new();
    for n in 0..3 {
        let id = session(&store, &format!("cap{n}")).await;
        set_last_seen(&store, &id, &old).await;
        ids.push(id);
    }
    store.touch_session_throttled(&ids[0], 15).await.unwrap();
    store.touch_session_throttled(&ids[1], 15).await.unwrap();
    let (_, queries) =
        crate::pool_telemetry::measure_queries(store.touch_session_throttled(&ids[2], 15)).await;
    assert_eq!(queries.write.calls, 1, "written through: {queries:?}");
    assert!(row_last_seen(&store, &ids[2]).await.unwrap() > old);
    assert!(store.buffered_last_seen(&ids[2]).is_none());
    assert_eq!(store.session_touch_buffer_stats().unwrap().write_through, 1);
    // After a flush the two entries are hints; a new session evicts them.
    store.flush_session_touches().await.unwrap();
    let fourth = session(&store, "cap3").await;
    set_last_seen(&store, &fourth, &old).await;
    let (_, queries) =
        crate::pool_telemetry::measure_queries(store.touch_session_throttled(&fourth, 15)).await;
    assert_eq!(
        queries.write.calls, 0,
        "hints evicted, buffered: {queries:?}"
    );
    assert!(store.session_touch_buffer_stats().unwrap().held <= 2);
    store.disable_session_touch_buffer().await.unwrap();
    store.close().await;
}

/// A failed flush keeps its entries; the next flush writes them.
#[tokio::test]
async fn a_failed_flush_keeps_the_touches() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    assert!(store.enable_session_touch_buffer(config(16)));
    let id = session(&store, "fail").await;
    let old = (Utc::now() - Age::seconds(60)).to_rfc3339();
    set_last_seen(&store, &id, &old).await;
    store.touch_session_throttled(&id, 15).await.unwrap();
    q("ALTER TABLE sessions RENAME TO sessions_away")
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(store.flush_session_touches().await.is_err());
    let stats = store.session_touch_buffer_stats().unwrap();
    assert_eq!((stats.pending, stats.flush_failures), (1, 1));
    q("ALTER TABLE sessions_away RENAME TO sessions")
        .execute(&store.backend)
        .await
        .unwrap();
    assert_eq!(store.flush_session_touches().await.unwrap(), 1);
    assert!(row_last_seen(&store, &id).await.unwrap() > old);
    store.disable_session_touch_buffer().await.unwrap();
    store.close().await;
}

/// The background task flushes within its interval; outside a runtime or with
/// an unusable configuration nothing is buffered.
#[tokio::test]
async fn the_background_flush_runs_on_its_interval() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    assert!(!store.enable_session_touch_buffer(TouchBufferConfig {
        throttle_secs: 0,
        ..config(4)
    }));
    assert!(!store.session_touch_buffered());
    assert!(store.enable_session_touch_buffer(TouchBufferConfig {
        flush_every: Duration::from_millis(100),
        ..config(4)
    }));
    let id = session(&store, "tick").await;
    let old = (Utc::now() - Age::seconds(60)).to_rfc3339();
    set_last_seen(&store, &id, &old).await;
    store.touch_session_throttled(&id, 15).await.unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while row_last_seen(&store, &id).await.unwrap() <= old {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("flushed by the background task");
    store.disable_session_touch_buffer().await.unwrap();
    store.close().await;
}

#[test]
fn outside_a_runtime_nothing_is_buffered() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let store = runtime
        .block_on(Store::open_in_memory(ServerKey::generate()))
        .unwrap();
    assert!(!store.enable_session_touch_buffer(config(4)));
    assert!(!store.session_touch_buffered());
    runtime.block_on(store.close());
}

/// Retention deletes sessions by `last_seen` age (`retention.sessions.idleDays`,
/// whole days, at least 1 when enabled; default 1095). The stored value lags a
/// buffered touch by at most the flush interval plus the throttle (20 s by
/// default, 305 s at the largest throttle), far below one day; and the sweep
/// writes the buffered touches before it compares. A session in active use is
/// therefore never deleted as stale; an idle one still is.
async fn retention_never_prunes_a_session_with_a_newer_buffered_touch(store: &Store) {
    assert!(store.enable_session_touch_buffer(config(1024)));
    let target = crate::jobs::retention::target("sessions").unwrap();
    let day = 86_400;
    let now = Utc::now().timestamp();
    // Both rows stored as 2 days old, against a 1-day threshold.
    let stored = (Utc::now() - Age::days(2)).to_rfc3339();
    let active = session(store, &format!("r{}", uuid::Uuid::new_v4().simple())).await;
    let idle = session(store, &format!("i{}", uuid::Uuid::new_v4().simple())).await;
    set_last_seen(store, &active, &stored).await;
    set_last_seen(store, &idle, &stored).await;
    // The active session's touch is only in the buffer.
    store.touch_session_throttled(&active, 15).await.unwrap();
    assert_eq!(
        row_last_seen(store, &active).await.as_deref(),
        Some(stored.as_str()),
        "row lags"
    );
    let lag_bound = Duration::from_secs(5 + 15);
    assert!(
        lag_bound.as_secs() * 1000 < day as u64,
        "lag is orders of magnitude below one day"
    );
    let counted = store
        .retention_sweep(target, now - day, 100, 1, true, &[])
        .await
        .unwrap();
    // Other rows of a shared test database may be old too: count, at least ours.
    assert!(counted.rows >= 1, "dry run counts the idle session");
    let swept = store
        .retention_sweep(target, now - day, 100, 4, false, &[])
        .await
        .unwrap();
    assert!(swept.rows >= 1, "the idle session is deleted");
    assert!(
        store
            .get_primary_session_with_activity(&active)
            .await
            .is_ok(),
        "active session kept"
    );
    assert!(
        store
            .get_primary_session_with_activity(&idle)
            .await
            .is_err(),
        "idle session deleted"
    );
    store.delete_session(&active).await.unwrap();
    store.disable_session_touch_buffer().await.unwrap();
}

#[tokio::test]
async fn sqlite_retention_never_prunes_a_session_with_a_newer_buffered_touch() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    retention_never_prunes_a_session_with_a_newer_buffered_touch(&store).await;
    store.close().await;
}

#[tokio::test]
async fn postgres_retention_never_prunes_a_session_with_a_newer_buffered_touch() {
    let Ok(url) = std::env::var("MW_TEST_PG") else {
        mw_test_gate::skip("touch buffer retention PostgreSQL leg: MW_TEST_PG not configured");
        return;
    };
    let store = Store::open_postgres(&url, ServerKey::generate())
        .await
        .unwrap();
    retention_never_prunes_a_session_with_a_newer_buffered_touch(&store).await;
    store.close().await;
}
