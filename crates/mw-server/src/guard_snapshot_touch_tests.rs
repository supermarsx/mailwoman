//! t50hp-e22 (W-1): the session touch no longer selects the request path.
//!
//! Over the real router and Store, SQLite and (with `MW_TEST_PG`) PostgreSQL:
//! a session whose `last_seen` is 20–110 s old keeps its published inputs, its
//! touch is recorded in memory and written within the flush interval; a session
//! whose deadline is inside the 120 s clock margin takes the original path;
//! sign-out, revoke, disable and password change refuse the next request on a
//! touch-due session, in the same AppState at once and in another one within
//! the mirror bound; the shutdown drain writes what is pending.
use super::pg_tests::{
    BOUND, Fixture, Node, SHORT, Sent, pg_dsn, pg_schema, policy, run, send, sqlite_db,
};
use axum::http::StatusCode;
use serde_json::json;
use std::time::{Duration, Instant};

/// Set the row's `last_seen` on a connection of its own.
async fn set_last_seen(f: &Fixture, at: &str) {
    set_column(f, "last_seen", at).await;
}
/// A raw clock write may move a clock backwards, which no writer does; a
/// fence cached before it would then carry a NEWER value than the row. Forget
/// the cached fences of both AppStates so the next request reads the row.
fn forget_cached_fences(f: &Fixture) {
    let environment = crate::request_inputs::Environment::capture().unwrap();
    for node in [&f.a, &f.b] {
        if let Some(entry) = node
            .state
            .native_publication
            .as_ref()
            .unwrap()
            .request_inputs(&f.session, &environment)
        {
            entry.guard_fence.clear();
        }
    }
}
async fn set_column(f: &Fixture, column: &str, at: &str) {
    forget_cached_fences(f);
    let sql = format!(
        "UPDATE sessions SET {column}='{at}' WHERE id='{}'",
        f.session
    );
    if f.url.starts_with("postgres") {
        assert_eq!(f.pg(&sql).await, 1);
        return;
    }
    let path = std::path::PathBuf::from(&f.url);
    let key = mw_store::encryption::global()
        .unwrap()
        .key_for_open(&path)
        .unwrap()
        .cloned();
    let options = mw_store::encryption::connect_options(&path, key.as_ref(), false);
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();
    assert_eq!(
        sqlx::query(&sql)
            .execute(&pool)
            .await
            .unwrap()
            .rows_affected(),
        1
    );
    pool.close().await;
}
fn ago(secs: i64) -> String {
    (chrono::Utc::now() - chrono::Duration::seconds(secs)).to_rfc3339()
}
/// The row's `last_seen` as another Store (no buffer) reads it.
async fn row_last_seen(f: &Fixture) -> Option<String> {
    f.writer
        .get_primary_session_with_activity(&f.session)
        .await
        .ok()
        .and_then(|(_, seen)| seen)
}
/// The state after a client was away longer than the throttle: this process
/// holds no recent touch for the session (its hint aged out) and the row is
/// `idle_secs` old.
async fn away(f: &Fixture, node: &Node, idle_secs: i64) -> String {
    let store = &node.state.store;
    store.disable_session_touch_buffer().await.unwrap();
    assert!(crate::session_persistence::enable_touch_buffer(store));
    let old = ago(idle_secs);
    set_last_seen(f, &old).await;
    old
}
/// Published inputs carried the request: a frame was installed (or, bounded,
/// a certified fence reused) and the original session reader never ran.
fn on_published_inputs(reply: &Sent) -> bool {
    reply.status == StatusCode::OK
        && (reply.inputs.frames_ready == 1 || reply.snap.reused > 0)
        && reply.inputs.legacy_session_reads == 0
}

async fn idle_session_keeps_published_inputs(url: String) {
    let f = Fixture::new(url).await;
    let store = f.b.state.store.clone();
    assert!(
        store.session_touch_buffered(),
        "the server enables the touch buffer"
    );
    f.ready(&f.b, SHORT).await;
    for idle in [20, 60, 110] {
        let old = away(&f, &f.b, idle).await;
        let reply = send(&f.b, f.me(), SHORT).await;
        assert!(
            on_published_inputs(&reply),
            "idle {idle} s: {:?} {:?} {}",
            reply.inputs,
            reply.snap,
            reply.body
        );
        let held = store
            .buffered_last_seen(&f.session)
            .expect("the touch was recorded in memory");
        assert!(held > old, "idle {idle} s");
        let recorded = Instant::now();
        // Not written by the request; written by the flush within its interval.
        let limit = crate::session_persistence::TOUCH_FLUSH_EVERY + Duration::from_millis(1500);
        tokio::time::timeout(limit, async {
            while row_last_seen(&f).await.is_none_or(|seen| seen <= old) {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        })
        .await
        .unwrap_or_else(|_| panic!("idle {idle} s: last_seen not flushed within {limit:?}"));
        assert!(recorded.elapsed() <= limit);
        assert_eq!(row_last_seen(&f).await.as_deref(), Some(held.as_str()));
    }
    // Control: without the buffer a due touch selects the original path.
    store.disable_session_touch_buffer().await.unwrap();
    set_last_seen(&f, &ago(60)).await;
    let original = send(&f.b, f.me(), SHORT).await;
    assert_eq!(original.status, StatusCode::OK);
    assert!(
        original.inputs.legacy_session_reads > 0,
        "unbuffered: original reader ({:?})",
        original.inputs
    );
    assert!(
        row_last_seen(&f).await.unwrap() > ago(5),
        "unbuffered: written by the request"
    );
    assert!(crate::session_persistence::enable_touch_buffer(&store));
}

#[test]
fn sqlite_idle_session_keeps_published_inputs_and_flushes_last_seen() {
    run(|| async { idle_session_keeps_published_inputs(sqlite_db()).await });
}
#[test]
fn postgres_idle_session_keeps_published_inputs_and_flushes_last_seen() {
    let Some(dsn) = pg_dsn() else { return };
    run(move || async move { idle_session_keeps_published_inputs(pg_schema(&dsn).await).await });
}

/// Inside the clock margin the original path decides, touch due or not.
async fn clock_margin_takes_the_original_path(url: String) {
    let f = Fixture::new(url).await;
    policy(&f.writer, "session.absoluteSecs", json!(3600)).await;
    f.ready(&f.b, SHORT).await;
    // 60 s before the absolute deadline: inside the 120 s margin.
    set_column(&f, "created_at", &ago(3600 - 60)).await;
    for due in [true, false] {
        if due {
            away(&f, &f.b, 60).await;
        }
        let reply = send(&f.b, f.me(), SHORT).await;
        assert_eq!(
            reply.status,
            StatusCode::OK,
            "still before the deadline: {}",
            reply.body
        );
        assert_eq!(
            (reply.inputs.frames_ready, reply.snap.reused),
            (0, 0),
            "due={due}: {:?} {:?}",
            reply.inputs,
            reply.snap
        );
        assert!(
            reply.inputs.legacy_session_reads > 0,
            "due={due}: original reader"
        );
    }
    // Outside the margin again: published inputs.
    set_column(&f, "created_at", &ago(60)).await;
    f.ready(&f.b, SHORT).await;
    away(&f, &f.b, 60).await;
    assert!(on_published_inputs(&send(&f.b, f.me(), SHORT).await));
}

#[test]
fn sqlite_clock_margin_takes_the_original_path() {
    run(|| async { clock_margin_takes_the_original_path(sqlite_db()).await });
}
#[test]
fn postgres_clock_margin_takes_the_original_path() {
    let Some(dsn) = pg_dsn() else { return };
    run(move || async move { clock_margin_takes_the_original_path(pg_schema(&dsn).await).await });
}

/// The next request on a touch-due session after a revocation committed in
/// the same AppState (`same`: zero) or in another one (within the bound).
async fn revocation_on_a_touch_due_session(url: String, case: &'static str, same: bool) {
    let f = Fixture::new(url).await;
    f.ready(&f.a, SHORT).await;
    f.ready(&f.b, SHORT).await;
    let mirror = f.mirror(&f.b).await;
    away(&f, &f.b, 60).await;
    let due = send(&f.b, f.me(), SHORT).await;
    assert!(
        on_published_inputs(&due),
        "{case}: touch-due request used published inputs: {:?}",
        due.inputs
    );
    // The next request is touch-due again.
    away(&f, &f.b, 60).await;
    let actor = if same { &f.b } else { &f.a };
    let expedites = mirror.counters().expedites;
    match case {
        "sign-out" => {
            let out = send(actor, f.logout(), SHORT).await;
            assert!(
                out.status.is_success(),
                "{case}: {} {}",
                out.status,
                out.body
            );
        }
        "session-revoke" => {
            actor.state.store.delete_session(&f.session).await.unwrap();
            super::expedite(&actor.state);
        }
        // The console's write request expedites through the request layer.
        "account-disable" => {
            crate::account_gate::update_flags(&actor.state.store, &f.account, |flags| {
                flags.disabled = true
            })
            .await
            .unwrap();
            super::expedite(&actor.state);
        }
        "password-change" => {
            crate::account_gate::update_flags(&actor.state.store, &f.account, |flags| {
                flags.force_password_change = true
            })
            .await
            .unwrap();
            super::expedite(&actor.state);
        }
        _ => unreachable!(),
    }
    if !same {
        tokio::time::sleep(BOUND).await;
        assert_eq!(
            mirror.counters().expedites,
            expedites,
            "{case}: B was never expedited"
        );
    }
    let after = send(&f.b, f.me(), SHORT).await;
    assert_eq!(after.snap.reused, 0, "{case} same={same}: no reuse");
    match case {
        "sign-out" | "session-revoke" | "account-disable" => {
            assert_eq!(
                after.status,
                StatusCode::UNAUTHORIZED,
                "{case} same={same}: {}",
                after.body
            )
        }
        "password-change" => {
            assert_eq!(after.status, StatusCode::OK, "{case}: {}", after.body);
            assert_eq!(
                after.body["passwordChangeRequired"], true,
                "{case} same={same}: {}",
                after.body
            );
        }
        _ => unreachable!(),
    }
    // A buffered touch never recreates a deleted session.
    f.b.state.store.flush_session_touches().await.unwrap();
    if case != "password-change" && case != "account-disable" {
        assert!(
            f.writer
                .get_primary_session_with_activity(&f.session)
                .await
                .is_err(),
            "{case}: still deleted"
        );
    }
}

const REVOCATIONS: [&str; 4] = [
    "sign-out",
    "session-revoke",
    "account-disable",
    "password-change",
];

#[test]
fn sqlite_revocation_refuses_the_next_request_on_a_touch_due_session() {
    for case in REVOCATIONS {
        for same in [true, false] {
            run(
                move || async move { revocation_on_a_touch_due_session(sqlite_db(), case, same).await },
            );
        }
    }
}
#[test]
fn postgres_revocation_refuses_the_next_request_on_a_touch_due_session() {
    let Some(dsn) = pg_dsn() else { return };
    for case in REVOCATIONS {
        for same in [true, false] {
            let dsn = dsn.clone();
            run(move || async move {
                revocation_on_a_touch_due_session(pg_schema(&dsn).await, case, same).await
            });
        }
    }
}

/// The shutdown drain writes the buffered touches.
async fn shutdown_drain_flushes(url: String) {
    let f = Fixture::new(url).await;
    f.ready(&f.b, SHORT).await;
    let old = away(&f, &f.b, 60).await;
    assert!(on_published_inputs(&send(&f.b, f.me(), SHORT).await));
    let state = &f.b.state;
    assert_eq!(state.store.session_touch_buffer_stats().unwrap().pending, 1);
    let report = crate::t34_modes::Held::new(
        state.store.clone(),
        None,
        state.request_protection.clone(),
        state.request_history.clone(),
        state.client_timing.clone(),
    )
    .drain(Duration::from_secs(1))
    .await;
    assert!(report.accounting_flushed);
    assert_eq!(state.store.session_touch_buffer_stats().unwrap().pending, 0);
    assert!(
        row_last_seen(&f).await.unwrap() > old,
        "written by the drain"
    );
}

#[test]
fn sqlite_shutdown_drain_flushes_buffered_touches() {
    run(|| async { shutdown_drain_flushes(sqlite_db()).await });
}
#[test]
fn postgres_shutdown_drain_flushes_buffered_touches() {
    let Some(dsn) = pg_dsn() else { return };
    run(move || async move { shutdown_drain_flushes(pg_schema(&dsn).await).await });
}
