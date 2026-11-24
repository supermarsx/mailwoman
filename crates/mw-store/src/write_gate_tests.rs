//! Tests of the in-process SQLite write gate (t50hp-e18).
use super::*;
use crate::backend::{Backend, q};
use crate::sqlite_live::LiveSqlite;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use sqlx::{Connection, SqliteConnection};
use std::future::{Future, poll_fn};
use std::path::Path;
use std::sync::Mutex as StdMutex;
use std::task::Poll;

fn options(dir: &Path, busy: Duration) -> SqliteConnectOptions {
    SqliteConnectOptions::new()
        .filename(dir.join("gate.db"))
        .create_if_missing(true)
        .busy_timeout(busy)
}

/// A WAL database with table `w(n)` and a pool of `max` connections, all
/// opened before the test starts (so no connection setup races the test).
async fn file_pool(dir: &Path, max: u32, busy: Duration) -> SqlitePool {
    let mut setup = SqliteConnection::connect_with(
        &options(dir, Duration::from_secs(5)).journal_mode(SqliteJournalMode::Wal),
    )
    .await
    .unwrap();
    sqlx::query("CREATE TABLE IF NOT EXISTS w(n INTEGER NOT NULL)")
        .execute(&mut setup)
        .await
        .unwrap();
    setup.close().await.unwrap();
    let pool = SqlitePoolOptions::new()
        .max_connections(max)
        .acquire_timeout(Duration::from_secs(30))
        .connect_with(options(dir, busy))
        .await
        .unwrap();
    let mut held = Vec::new();
    for _ in 0..max {
        held.push(pool.acquire().await.unwrap());
    }
    drop(held);
    pool
}

fn gated(pool: SqlitePool, gate: &Arc<WriteGate>) -> Backend {
    Backend::Sqlite(LiveSqlite::unmanaged_with_gate(pool, Some(gate.clone())))
}

fn is_locked(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Database(db) => {
            db.code().as_deref().is_some_and(|code| code == "5" || code.parse::<i64>().is_ok_and(|c| c & 0xff == 5))
                && db.message().contains("database is locked")
        }
        _ => false,
    }
}

async fn count(backend: &Backend) -> i64 {
    q("SELECT COUNT(*) FROM w").fetch_scalar_i64(backend).await.unwrap()
}

async fn wait_for(mut done: impl FnMut() -> bool) {
    for _ in 0..2000 {
        if done() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    panic!("condition not reached within 2 s");
}

// ---- switch and classification --------------------------------------------

#[test]
fn mode_parse_defaults_to_gate_and_refuses_unknown_values() {
    assert_eq!(parse_mode(None), Ok(Mode::Gate));
    assert_eq!(parse_mode(Some("gate")), Ok(Mode::Gate));
    assert_eq!(parse_mode(Some(" OFF ")), Ok(Mode::Off));
    assert_eq!(parse_mode(Some("")), Ok(Mode::Gate));
    assert_eq!(parse_mode(Some("  ")), Ok(Mode::Gate));
    assert_eq!(parse_mode(Some("group")), Err(INVALID_MODE.to_string()));
    assert_eq!(parse_mode(Some("on")), Err(INVALID_MODE.to_string()));
}

#[test]
fn only_write_verbs_are_gated() {
    for sql in [
        "INSERT INTO w VALUES(1)",
        "  insert or ignore into w values(1)",
        "UPDATE w SET n=1",
        "DELETE FROM w",
        "REPLACE INTO w VALUES(1)",
        "INSERT INTO w VALUES(1) RETURNING n",
    ] {
        assert!(is_gated_statement(sql), "{sql}");
    }
    for sql in [
        "SELECT 1",
        "WITH x AS (SELECT 1) SELECT * FROM x",
        "VALUES(1)",
        "PRAGMA wal_checkpoint(PASSIVE)",
        "VACUUM",
        "CREATE TABLE t(n)",
        "BEGIN IMMEDIATE",
        "",
    ] {
        assert!(!is_gated_statement(sql), "{sql}");
    }
}

// ---- the gate itself --------------------------------------------------------

#[tokio::test]
async fn waiters_are_served_in_arrival_order() {
    let gate = WriteGate::new();
    let first = gate.acquire(Site::Statement).await.unwrap();
    let order = Arc::new(StdMutex::new(Vec::new()));
    let mut tasks = Vec::new();
    for index in 0..16u64 {
        let gate2 = gate.clone();
        let order2 = order.clone();
        tasks.push(tokio::spawn(async move {
            let permit = gate2.acquire(Site::Statement).await.unwrap();
            order2.lock().unwrap().push(index);
            tokio::task::yield_now().await;
            drop(permit);
        }));
        // The next writer arrives only after this one is queued.
        wait_for(|| gate.stats().queued == index + 1).await;
    }
    drop(first);
    for task in tasks {
        task.await.unwrap();
    }
    assert_eq!(*order.lock().unwrap(), (0..16).collect::<Vec<_>>());
    let stats = gate.stats();
    assert_eq!((stats.granted, stats.waited, stats.queued, stats.queued_max), (17, 16, 0, 16));
    assert_eq!(stats.timeouts + stats.cancelled, 0);
}

#[tokio::test]
async fn cancelled_waiter_leaves_the_queue_and_never_takes_the_permit() {
    let gate = WriteGate::new();
    let held = gate.acquire(Site::Statement).await.unwrap();
    let mut waiting = Box::pin(gate.acquire(Site::Statement));
    assert!(poll_fn(|cx| Poll::Ready(waiting.as_mut().poll(cx).is_pending())).await);
    assert_eq!(gate.stats().queued, 1);
    let gate2 = gate.clone();
    let next = tokio::spawn(async move { gate2.acquire(Site::Statement).await.map(drop) });
    wait_for(|| gate.stats().queued == 2).await;
    drop(waiting);
    assert_eq!(gate.stats().queued, 1);
    assert_eq!(gate.stats().cancelled, 1);
    drop(held);
    tokio::time::timeout(Duration::from_secs(2), next).await.unwrap().unwrap().unwrap();
    let stats = gate.stats();
    assert_eq!((stats.queued, stats.granted), (0, 2));
    // The permit is free again.
    drop(gate.acquire(Site::Statement).await.unwrap());
}

#[tokio::test]
async fn a_waiter_gives_up_after_the_wait_limit_with_the_locked_error() {
    let gate = WriteGate::with_wait_limit(Duration::from_millis(100));
    let held = gate.acquire(Site::Statement).await.unwrap();
    let started = Instant::now();
    let error = gate.acquire(Site::Statement).await.unwrap_err();
    assert!(started.elapsed() >= Duration::from_millis(95));
    assert!(is_locked(&error), "{error:?}");
    assert_eq!(error.to_string(), "error returned from database: database is locked");
    let stats = gate.stats();
    assert_eq!((stats.timeouts, stats.queued, stats.cancelled), (1, 0, 0));
    drop(held);
    drop(gate.acquire(Site::Statement).await.unwrap());
}

#[tokio::test]
async fn a_long_hold_is_counted() {
    let gate = WriteGate::new();
    let permit = gate.acquire(Site::Statement).await.unwrap();
    tokio::time::sleep(LONG_HOLD + Duration::from_millis(50)).await;
    drop(permit);
    let stats = gate.stats();
    assert_eq!(stats.long_holds, 1);
    assert!(stats.hold_micros_max >= micros(LONG_HOLD));
}

// ---- hooks in LiveSqlite / Backend -------------------------------------------

#[tokio::test]
async fn cancelling_a_gated_begin_or_write_at_every_await_releases_the_permit() {
    let dir = tempfile::tempdir().unwrap();
    let gate = WriteGate::with_wait_limit(Duration::from_secs(2));
    let pool = file_pool(dir.path(), 2, Duration::from_secs(5)).await;
    let backend = gated(pool.clone(), &gate);
    for kind in ["begin", "write"] {
        let mut completed = false;
        for polls in 1..=64 {
            let mut future: std::pin::Pin<Box<dyn Future<Output = ()> + Send>> = match kind {
                "begin" => {
                    let backend = backend.clone();
                    Box::pin(async move {
                        let mut tx = backend.begin().await.unwrap();
                        q("INSERT INTO w VALUES(1)").execute_tx(&mut tx).await.unwrap();
                        tx.commit().await.unwrap();
                    })
                }
                _ => {
                    let backend = backend.clone();
                    Box::pin(async move {
                        q("INSERT INTO w VALUES(2)").execute(&backend).await.unwrap();
                    })
                }
            };
            let mut finished = false;
            for _ in 0..polls {
                if poll_fn(|cx| Poll::Ready(future.as_mut().poll(cx).is_ready())).await {
                    finished = true;
                    break;
                }
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            drop(future);
            // The permit is free at once, and the database lock is free once
            // the queued rollback ran (a fresh BEGIN IMMEDIATE waits for it in
            // the busy handler at most).
            let permit = tokio::time::timeout(Duration::from_millis(500), gate.acquire(Site::Statement))
                .await
                .expect("permit released after cancellation")
                .unwrap();
            drop(permit);
            let mut tx = tokio::time::timeout(Duration::from_secs(5), backend.begin())
                .await
                .unwrap()
                .unwrap();
            q("SELECT 1").fetch_all_tx(&mut tx).await.unwrap();
            tx.rollback().await.unwrap();
            if finished {
                completed = true;
                break;
            }
        }
        assert!(completed, "{kind}: the future never completed within 64 polls");
    }
    assert_eq!(gate.stats().queued, 0);
    pool.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn waiting_writers_hold_no_pool_connection_and_reads_do_not_take_the_gate() {
    let dir = tempfile::tempdir().unwrap();
    let gate = WriteGate::new();
    let pool = file_pool(dir.path(), 4, Duration::from_secs(5)).await;
    let backend = gated(pool.clone(), &gate);
    let mut holder = backend.begin().await.unwrap();
    q("INSERT INTO w VALUES(0)").execute_tx(&mut holder).await.unwrap();
    let mut writers = Vec::new();
    for index in 0..16i64 {
        let backend = backend.clone();
        writers.push(tokio::spawn(async move {
            if index % 2 == 0 {
                q("INSERT INTO w VALUES(?1)").bind(index).execute(&backend).await.unwrap();
            } else {
                let mut tx = backend.begin().await.unwrap();
                q("INSERT INTO w VALUES(?1)").bind(index).execute_tx(&mut tx).await.unwrap();
                tx.commit().await.unwrap();
            }
        }));
    }
    wait_for(|| gate.stats().queued == 16).await;
    // 16 writers wait; only the holder's connection is in use.
    assert_eq!(pool.size() - pool.num_idle() as u32, 1);
    let granted = gate.stats().granted;
    // Reads of every shape complete while the queue is full.
    let reads = async {
        assert_eq!(count(&backend).await, 0);
        q("WITH x AS (SELECT n FROM w) SELECT COUNT(*) FROM x").fetch_all(&backend).await.unwrap();
        q("VALUES(1)").fetch_all(&backend).await.unwrap();
        backend
            .fetch_all_unit(vec![q("SELECT COUNT(*) FROM w"), q("SELECT 1")])
            .await
            .unwrap();
        let Backend::Sqlite(live) = &backend else { unreachable!() };
        sqlx::query("PRAGMA user_version").fetch_all(live).await.unwrap();
    };
    tokio::time::timeout(Duration::from_secs(2), reads).await.expect("reads wait for no writer");
    assert_eq!(gate.stats().granted, granted, "reads took no permit");
    // Read connections return to the pool asynchronously.
    wait_for(|| pool.size() - pool.num_idle() as u32 == 1).await;
    holder.commit().await.unwrap();
    for writer in writers {
        writer.await.unwrap();
    }
    assert_eq!(count(&backend).await, 17);
    let stats = gate.stats();
    assert_eq!((stats.granted, stats.waited, stats.timeouts, stats.queued), (17, 16, 0, 0));
    pool.close().await;
}

/// 64 concurrent writers on 8 connections with `busy_timeout=0`: any wait in
/// SQLite's busy handler would surface at once as SQLITE_BUSY. With the gate
/// every write commits; without it the same load fails.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn sixty_four_gated_writers_never_meet_sqlite_busy() {
    async fn run(backend: Backend) -> (usize, usize) {
        let mut tasks = Vec::new();
        for writer in 0..64i64 {
            let backend = backend.clone();
            tasks.push(tokio::spawn(async move {
                let mut failures = 0;
                for round in 0..8i64 {
                    let result = async {
                        if (writer + round) % 3 == 0 {
                            q("INSERT INTO w VALUES(?1)").bind(writer).execute(&backend).await?;
                        } else {
                            let mut tx = backend.begin().await?;
                            q("INSERT INTO w VALUES(?1)").bind(writer).execute_tx(&mut tx).await?;
                            // Hold the write lock across an await, as request
                            // transactions do.
                            tokio::time::sleep(Duration::from_micros(200)).await;
                            q("UPDATE w SET n=n WHERE rowid=last_insert_rowid()").execute_tx(&mut tx).await?;
                            tx.commit().await?;
                        }
                        Ok::<_, sqlx::Error>(())
                    }
                    .await;
                    match result {
                        Ok(()) => {}
                        Err(error) if is_locked(&error) => failures += 1,
                        Err(error) => panic!("unexpected error: {error:?}"),
                    }
                }
                failures
            }));
        }
        let mut failures = 0;
        for task in tasks {
            failures += task.await.unwrap();
        }
        (64 * 8 - failures, failures)
    }
    let dir = tempfile::tempdir().unwrap();
    let gate = WriteGate::new();
    let pool = file_pool(dir.path(), 8, Duration::ZERO).await;
    let (committed, failures) = run(gated(pool.clone(), &gate)).await;
    assert_eq!((committed, failures), (512, 0));
    let backend = gated(pool.clone(), &gate);
    assert_eq!(count(&backend).await, 512);
    let stats = gate.stats();
    assert_eq!((stats.granted, stats.timeouts, stats.queued), (512, 0, 0));
    assert!(stats.waited > 0, "the load contended for the gate");
    pool.close().await;

    // Control: the same load without the gate meets SQLITE_BUSY.
    let dir = tempfile::tempdir().unwrap();
    let pool = file_pool(dir.path(), 8, Duration::ZERO).await;
    let (_, failures) = run(Backend::Sqlite(LiveSqlite::unmanaged_with_gate(pool.clone(), None))).await;
    assert!(failures > 0, "without the gate the writers collide in SQLite");
    pool.close().await;
}

/// Another process (here: a connection outside the pool) holding the write
/// lock is still waited for through `busy_timeout`, and still fails after it.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_lock_held_outside_the_process_still_resolves_through_busy_timeout() {
    let dir = tempfile::tempdir().unwrap();
    let gate = WriteGate::new();
    let pool = file_pool(dir.path(), 2, Duration::from_secs(5)).await;
    let backend = gated(pool.clone(), &gate);
    let mut outside = SqliteConnection::connect_with(&options(dir.path(), Duration::from_secs(5)))
        .await
        .unwrap();
    sqlx::query("BEGIN IMMEDIATE").execute(&mut outside).await.unwrap();
    sqlx::query("INSERT INTO w VALUES(100)").execute(&mut outside).await.unwrap();
    let writer = {
        let backend = backend.clone();
        tokio::spawn(async move {
            let started = Instant::now();
            let mut tx = backend.begin().await.unwrap();
            q("INSERT INTO w VALUES(1)").execute_tx(&mut tx).await.unwrap();
            tx.commit().await.unwrap();
            q("INSERT INTO w VALUES(2)").execute(&backend).await.unwrap();
            started.elapsed()
        })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    sqlx::query("COMMIT").execute(&mut outside).await.unwrap();
    let waited = writer.await.unwrap();
    assert!(waited >= Duration::from_millis(250), "waited {waited:?}");
    assert_eq!(count(&backend).await, 3);
    pool.close().await;

    // A short busy_timeout still fails with the locked error while the gate
    // itself is free.
    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(options(dir.path(), Duration::from_millis(100)))
        .await
        .unwrap();
    let backend = gated(pool.clone(), &gate);
    sqlx::query("BEGIN IMMEDIATE").execute(&mut outside).await.unwrap();
    let error = q("INSERT INTO w VALUES(3)").execute(&backend).await.unwrap_err();
    assert!(is_locked(&error), "{error:?}");
    let error = backend.begin().await.err().expect("BEGIN IMMEDIATE is busy");
    assert!(is_locked(&error), "{error:?}");
    sqlx::query("ROLLBACK").execute(&mut outside).await.unwrap();
    q("INSERT INTO w VALUES(3)").execute(&backend).await.unwrap();
    outside.close().await.unwrap();
    assert_eq!(gate.stats().timeouts, 0);
    pool.close().await;
}

// ---- nested and overlapping write patterns (deadlock safety) -----------------

/// A task that holds a write transaction and writes through the pool (another
/// connection) cannot finish either way: without the gate the second write
/// sleeps in the busy handler until `busy_timeout`; with it, it waits in the
/// gate for the same limit. Both end with the same locked error, after the
/// same bound, and the transaction stays usable.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_write_beside_an_open_transaction_fails_after_the_same_bound_with_and_without_gate() {
    let limit = Duration::from_millis(200);
    for with_gate in [true, false] {
        let dir = tempfile::tempdir().unwrap();
        let gate = WriteGate::with_wait_limit(limit);
        gate.allow_same_task_wait.store(true, Ordering::Relaxed);
        let pool = file_pool(dir.path(), 2, limit).await;
        let backend = Backend::Sqlite(LiveSqlite::unmanaged_with_gate(
            pool.clone(),
            with_gate.then(|| gate.clone()),
        ));
        // Inside a spawned task, as request handlers run (the same-task
        // counter needs a task id; a block_on future has none).
        let task_backend = backend.clone();
        tokio::spawn(async move {
            let backend = task_backend;
            let mut tx = backend.begin().await.unwrap();
            q("INSERT INTO w VALUES(1)").execute_tx(&mut tx).await.unwrap();
            let started = Instant::now();
            let autocommit = q("INSERT INTO w VALUES(2)").execute(&backend).await.unwrap_err();
            let nested_begin = backend.begin().await.err().expect("nested BEGIN cannot get the lock");
            let elapsed = started.elapsed();
            assert!(is_locked(&autocommit), "{with_gate}: {autocommit:?}");
            assert!(is_locked(&nested_begin), "{with_gate}: {nested_begin:?}");
            assert!(elapsed >= limit * 2 - Duration::from_millis(20), "{with_gate}: {elapsed:?}");
            assert!(elapsed < Duration::from_secs(3), "{with_gate}: {elapsed:?}");
            q("INSERT INTO w VALUES(3)").execute_tx(&mut tx).await.unwrap();
            tx.commit().await.unwrap();
        })
        .await
        .unwrap();
        assert_eq!(count(&backend).await, 2);
        if with_gate {
            let stats = gate.stats();
            assert_eq!((stats.same_task_waits, stats.timeouts), (2, 2));
        }
        pool.close().await;
    }
}

/// Two independent transactions joined in ONE task: the second waits in the
/// gate while the first finishes in the other branch, then proceeds. No
/// deadlock, even though both run on the same tokio task.
#[tokio::test]
async fn transactions_joined_in_one_task_both_commit() {
    let dir = tempfile::tempdir().unwrap();
    let gate = WriteGate::new();
    gate.allow_same_task_wait.store(true, Ordering::Relaxed);
    let pool = file_pool(dir.path(), 2, Duration::from_secs(5)).await;
    let backend = gated(pool.clone(), &gate);
    let one = async {
        let mut tx = backend.begin().await?;
        q("INSERT INTO w VALUES(1)").execute_tx(&mut tx).await?;
        tokio::time::sleep(Duration::from_millis(20)).await;
        tx.commit().await
    };
    let two = async {
        tokio::task::yield_now().await;
        q("INSERT INTO w VALUES(2)").execute(&backend).await.map(drop)
    };
    let (one, two) = tokio::time::timeout(Duration::from_secs(4), async { tokio::join!(one, two) })
        .await
        .expect("no deadlock");
    one.unwrap();
    two.unwrap();
    assert_eq!(count(&backend).await, 2);
    assert_eq!(gate.stats().timeouts, 0);
    pool.close().await;
}

/// Store maintenance writes on private connections outside the pool (50 ms
/// busy timeout) and does not take the gate: a queue of waiting writers does
/// not delay it, and it meets a permit holder only through SQLite's lock.
#[tokio::test]
async fn private_maintenance_connections_do_not_take_the_gate() {
    let dir = tempfile::tempdir().unwrap();
    let gate = WriteGate::new();
    let pool = file_pool(dir.path(), 2, Duration::from_secs(5)).await;
    let backend = gated(pool.clone(), &gate);
    let held = gate.acquire(Site::Statement).await.unwrap();
    let mut private = SqliteConnection::connect_with(&options(dir.path(), Duration::from_millis(50)))
        .await
        .unwrap();
    tokio::time::timeout(
        Duration::from_secs(1),
        sqlx::query("DELETE FROM w WHERE n < 0").execute(&mut private),
    )
    .await
    .expect("maintenance does not wait for the gate")
    .unwrap();
    private.close().await.unwrap();
    drop(held);
    q("INSERT INTO w VALUES(1)").execute(&backend).await.unwrap();
    pool.close().await;
}

// ---- configuration --------------------------------------------------------------

#[tokio::test]
async fn managed_file_databases_get_a_gate_and_report_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mail.db");
    let store = crate::Store::open_with_encryption(
        path.to_str().unwrap(),
        crate::ServerKey::generate(),
        &crate::encryption::EncryptionConfig::off(),
    )
    .await
    .unwrap();
    let Backend::Sqlite(live) = &store.backend else { unreachable!() };
    let expected = configured_mode().unwrap();
    assert_eq!(live.write_gate().is_some(), expected == Mode::Gate);
    let before = live.write_gate().map(|gate| gate.stats().granted);
    let (_, request) = crate::pool_telemetry::measure_queries(store.set_setting("gate-probe", "1")).await;
    if expected == Mode::Gate {
        assert!(live.write_gate().unwrap().stats().granted > before.unwrap());
        // The process snapshot (the pool diagnostics' "operations") reports
        // the gate; a per-request snapshot never does.
        let process = serde_json::to_value(crate::pool_telemetry::process_snapshot()).unwrap();
        assert!(process["writeGate"]["granted"].as_u64().unwrap() >= 1, "{process}");
        assert!(process["writeGate"]["queued"].is_u64());
    }
    let request = serde_json::to_value(request).unwrap();
    assert!(request.get("writeGate").is_none(), "{request}");
    assert_eq!(expected.as_str(), if expected == Mode::Gate { "gate" } else { "off" });
    store.close().await;

    // In-memory stores never have a gate (unless the test-only override is on).
    let memory = crate::Store::open_in_memory(crate::ServerKey::generate()).await.unwrap();
    let Backend::Sqlite(live) = &memory.backend else { unreachable!() };
    assert_eq!(
        live.write_gate().is_some(),
        crate::sqlite_live::gate_every_pool_for_tests()
    );
    memory.close().await;
}
