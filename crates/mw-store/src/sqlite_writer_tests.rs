// t50hp-e19: the SQLite group-commit writer.
use super::*;
use crate::StoreError;
use crate::backend::{Backend, q, write_job};
use crate::sqlite_live::LiveSqlite;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous};
use std::future::{Future, poll_fn};
use std::path::Path;
use std::task::Poll;

fn limits() -> Limits {
    Limits::default()
}
/// Batches that only end when the queue is empty or full: lets a test hold
/// the first job and know every queued job joins its batch.
fn wide() -> Limits {
    Limits {
        max_time: Duration::from_secs(30),
        ..Limits::default()
    }
}

async fn file_pool(path: &Path, connections: u32) -> SqlitePool {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5))
        .foreign_keys(true);
    SqlitePoolOptions::new()
        .max_connections(connections)
        .connect_with(options)
        .await
        .unwrap()
}

struct Fixture {
    _dir: tempfile::TempDir,
    backend: Backend,
    writer: Arc<GroupWriter>,
    /// A separate pool on the same file: a reader that is not the writer.
    reader: SqlitePool,
}

async fn fixture(limits: Limits) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("writer.db");
    let pool = file_pool(&path, 4).await;
    sqlx::raw_sql(
        "CREATE TABLE t(id INTEGER PRIMARY KEY, v TEXT NOT NULL);
         CREATE TABLE seq(n INTEGER PRIMARY KEY AUTOINCREMENT, who INTEGER NOT NULL, seen INTEGER NOT NULL);
         CREATE TABLE p(id INTEGER PRIMARY KEY);
         CREATE TABLE c(id INTEGER PRIMARY KEY, p INTEGER NOT NULL REFERENCES p(id) DEFERRABLE INITIALLY DEFERRED);",
    )
    .execute(&pool)
    .await
    .unwrap();
    let writer = GroupWriter::with_limits(limits);
    let backend = Backend::Sqlite(LiveSqlite::unmanaged_with_writer(pool, writer.clone()));
    let reader = file_pool(&path, 2).await;
    Fixture {
        _dir: dir,
        backend,
        writer,
        reader,
    }
}

async fn insert(backend: &Backend, lane: Lane, id: i64) -> Result<(), StoreError> {
    write_job!(backend, lane, |tx| async {
        q("INSERT INTO t(id, v) VALUES(?1, 'x')")
            .bind(id)
            .execute_tx(tx)
            .await?;
        Ok::<_, StoreError>(())
    })
}

async fn ids(f: &Fixture) -> Vec<i64> {
    sqlx::query_scalar("SELECT id FROM t ORDER BY id")
        .fetch_all(&f.reader)
        .await
        .unwrap()
}

/// Wait until `n` tickets are submitted (queued or running).
async fn until_in_flight(writer: &GroupWriter, n: u64) {
    for _ in 0..2000 {
        if writer.stats().in_flight >= n {
            return;
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    panic!("in_flight never reached {n}: {:?}", writer.stats());
}

/// Opens once and stays open (a re-run of a held job passes at once).
struct Release(tokio::sync::Semaphore);
impl Release {
    fn notify_one(&self) {
        self.0.add_permits(1 << 20);
    }
}

/// A job that inserts `id`, then holds the batch transaction until released.
fn held(
    f: &Fixture,
    id: i64,
) -> (
    tokio::task::JoinHandle<Result<(), StoreError>>,
    Arc<tokio::sync::Notify>,
    Arc<Release>,
) {
    let release = Arc::new(Release(tokio::sync::Semaphore::new(0)));
    let entered = Arc::new(tokio::sync::Notify::new());
    let backend = f.backend.clone();
    let (r, e) = (release.clone(), entered.clone());
    let task = tokio::spawn(async move {
        write_job!(backend, Lane::Normal, |tx| async {
            q("INSERT INTO t(id, v) VALUES(?1, 'held')")
                .bind(id)
                .execute_tx(tx)
                .await?;
            e.notify_one();
            let _open = r.0.acquire().await;
            Ok::<_, StoreError>(())
        })
    });
    (task, entered, release)
}

fn spawn_insert(f: &Fixture, lane: Lane, id: i64) -> tokio::task::JoinHandle<Result<(), StoreError>> {
    let backend = f.backend.clone();
    tokio::spawn(async move { insert(&backend, lane, id).await })
}

// ---- grouping ---------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn queued_jobs_share_one_commit() {
    let f = fixture(wide()).await;
    let (first, entered, release) = held(&f, 0);
    entered.notified().await;
    let mut tasks = Vec::new();
    for id in 1..64 {
        tasks.push(spawn_insert(&f, Lane::Normal, id));
    }
    until_in_flight(&f.writer, 64).await;
    release.notify_one();
    first.await.unwrap().unwrap();
    for task in tasks {
        task.await.unwrap().unwrap();
    }
    assert_eq!(ids(&f).await, (0..64).collect::<Vec<_>>());
    let stats = f.writer.stats();
    assert_eq!(stats.commits, 1, "{stats:?}");
    assert_eq!(stats.jobs, 64);
    assert_eq!(stats.batch_jobs_max, 64);
    assert_eq!(stats.in_flight, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_jobs_take_far_fewer_commits_than_jobs() {
    let f = fixture(limits()).await;
    let mut tasks = Vec::new();
    for worker in 0..64i64 {
        let backend = f.backend.clone();
        tasks.push(tokio::spawn(async move {
            for round in 0..16i64 {
                insert(&backend, Lane::Normal, worker * 1000 + round).await.unwrap();
            }
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }
    assert_eq!(ids(&f).await.len(), 1024);
    let stats = f.writer.stats();
    assert_eq!(stats.jobs, 1024, "{stats:?}");
    // 64 callers with one outstanding job each: batches form while one runs.
    assert!(stats.commits * 4 <= stats.jobs, "{stats:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_lone_job_commits_at_once() {
    let f = fixture(limits()).await;
    for id in 0..20 {
        let started = Instant::now();
        insert(&f.backend, Lane::Normal, id).await.unwrap();
        assert!(started.elapsed() < Duration::from_secs(1));
    }
    let stats = f.writer.stats();
    assert_eq!((stats.commits, stats.jobs), (20, 20), "{stats:?}");
}

// ---- isolation -------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_failing_job_is_undone_alone_and_only_its_caller_sees_the_error() {
    let f = fixture(wide()).await;
    let (first, entered, release) = held(&f, 1);
    entered.notified().await;
    let before = spawn_insert(&f, Lane::Normal, 2);
    until_in_flight(&f.writer, 2).await;
    // Duplicate key: a constraint error inside the shared batch.
    let duplicate = spawn_insert(&f, Lane::Normal, 2);
    until_in_flight(&f.writer, 3).await;
    let backend = f.backend.clone();
    // Two statements, the second fails: the first must be undone too.
    let partial = tokio::spawn(async move {
        write_job!(backend, Lane::Normal, |tx| async {
            q("INSERT INTO t(id, v) VALUES(50, 'partial')").execute_tx(tx).await?;
            q("INSERT INTO t(id, v) VALUES(1, 'dup')").execute_tx(tx).await?;
            Ok::<_, StoreError>(())
        })
    });
    until_in_flight(&f.writer, 4).await;
    let after = spawn_insert(&f, Lane::Normal, 3);
    until_in_flight(&f.writer, 5).await;
    release.notify_one();
    first.await.unwrap().unwrap();
    before.await.unwrap().unwrap();
    assert!(duplicate.await.unwrap().is_err());
    assert!(partial.await.unwrap().is_err());
    after.await.unwrap().unwrap();
    assert_eq!(ids(&f).await, vec![1, 2, 3]);
    let stats = f.writer.stats();
    assert_eq!((stats.commits, stats.jobs, stats.job_errors), (1, 5, 2), "{stats:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_panicking_job_is_undone_and_the_writer_survives() {
    let f = fixture(wide()).await;
    let (first, entered, release) = held(&f, 1);
    entered.notified().await;
    let backend = f.backend.clone();
    let panicking = tokio::spawn(async move {
        let result: Result<(), StoreError> = write_job!(backend, Lane::Normal, |tx| async {
            q("INSERT INTO t(id, v) VALUES(2, 'panics')").execute_tx(tx).await?;
            if tx.generation_path().is_some() {
                panic!("job panics after a statement");
            }
            Ok::<_, StoreError>(())
        });
        result
    });
    until_in_flight(&f.writer, 2).await;
    let after = spawn_insert(&f, Lane::Normal, 3);
    until_in_flight(&f.writer, 3).await;
    release.notify_one();
    first.await.unwrap().unwrap();
    assert!(panicking.await.unwrap_err().is_panic());
    after.await.unwrap().unwrap();
    // Nothing is poisoned: the next jobs commit normally.
    insert(&f.backend, Lane::Normal, 4).await.unwrap();
    insert(&f.backend, Lane::Priority, 5).await.unwrap();
    assert_eq!(ids(&f).await, vec![1, 3, 4, 5]);
    let stats = f.writer.stats();
    assert_eq!(stats.abandoned, 1, "{stats:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_failed_commit_reruns_every_job_alone() {
    let f = fixture(wide()).await;
    let (first, entered, release) = held(&f, 1);
    entered.notified().await;
    let backend = f.backend.clone();
    let parent = tokio::spawn(async move {
        write_job!(backend, Lane::Normal, |tx| async {
            q("INSERT INTO p(id) VALUES(1)").execute_tx(tx).await?;
            Ok::<_, StoreError>(())
        })
    });
    until_in_flight(&f.writer, 2).await;
    // A deferred foreign key: no statement fails; the shared COMMIT does.
    let backend = f.backend.clone();
    let orphan = tokio::spawn(async move {
        write_job!(backend, Lane::Normal, |tx| async {
            q("INSERT INTO c(id, p) VALUES(1, 99)").execute_tx(tx).await?;
            Ok::<_, StoreError>(())
        })
    });
    until_in_flight(&f.writer, 3).await;
    let after = spawn_insert(&f, Lane::Normal, 2);
    until_in_flight(&f.writer, 4).await;
    release.notify_one();
    first.await.unwrap().unwrap();
    parent.await.unwrap().unwrap();
    assert!(orphan.await.unwrap().is_err(), "its own transaction's commit fails");
    after.await.unwrap().unwrap();
    assert_eq!(ids(&f).await, vec![1, 2]);
    let parents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM p")
        .fetch_one(&f.reader)
        .await
        .unwrap();
    let children: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM c")
        .fetch_one(&f.reader)
        .await
        .unwrap();
    assert_eq!((parents, children), (1, 0));
    let stats = f.writer.stats();
    assert_eq!(stats.batch_failures, 1, "{stats:?}");
}

// ---- crash consistency ------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_writer_stopped_mid_batch_loses_no_acknowledged_job_and_applies_no_unacknowledged_one() {
    let f = fixture(wide()).await;
    insert(&f.backend, Lane::Normal, 1).await.unwrap(); // acknowledged
    // Batch 2: job 10 finished its statements, job 11 holds the transaction,
    // job 12 is queued. None of them is acknowledged when the writer stops.
    let (_, entered0, release0) = held(&f, 9);
    entered0.notified().await;
    let done_before = spawn_insert(&f, Lane::Normal, 10);
    until_in_flight(&f.writer, 2).await;
    let (holding, entered, release) = held(&f, 11);
    until_in_flight(&f.writer, 3).await;
    let queued = spawn_insert(&f, Lane::Normal, 12);
    until_in_flight(&f.writer, 4).await;
    release0.notify_one();
    entered.notified().await;
    f.writer.abort_task();
    tokio::time::sleep(Duration::from_millis(50)).await;
    release.notify_one();
    assert!(holding.await.unwrap().is_err());
    assert!(done_before.await.unwrap().is_err());
    assert!(queued.await.unwrap().is_err());
    // The batch was rolled back: only the acknowledged row exists.
    assert_eq!(ids(&f).await, vec![1]);
    // A new writer task serves the next job.
    insert(&f.backend, Lane::Normal, 13).await.unwrap();
    assert_eq!(ids(&f).await, vec![1, 13]);
}

// ---- order and visibility ------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn jobs_run_in_queue_order_and_see_earlier_jobs_of_their_batch() {
    let f = fixture(wide()).await;
    let (first, entered, release) = held(&f, 0);
    entered.notified().await;
    let mut tasks = Vec::new();
    for who in 1..=16i64 {
        let backend = f.backend.clone();
        tasks.push(tokio::spawn(async move {
            write_job!(backend, Lane::Normal, |tx| async {
                let seen = q("SELECT COUNT(*) FROM seq").fetch_all_tx(tx).await?[0].get_i64_idx(0);
                q("INSERT INTO seq(who, seen) VALUES(?1, ?2)")
                    .bind(who)
                    .bind(seen)
                    .execute_tx(tx)
                    .await?;
                Ok::<_, StoreError>(seen)
            })
        }));
        until_in_flight(&f.writer, 1 + who as u64).await;
    }
    release.notify_one();
    first.await.unwrap().unwrap();
    for (index, task) in tasks.into_iter().enumerate() {
        assert_eq!(task.await.unwrap().unwrap(), index as i64);
    }
    let rows: Vec<(i64, i64)> = sqlx::query_as("SELECT who, seen FROM seq ORDER BY n")
        .fetch_all(&f.reader)
        .await
        .unwrap();
    assert_eq!(rows, (1..=16).map(|who| (who, who - 1)).collect::<Vec<_>>());
    assert_eq!(f.writer.stats().commits, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_answered_job_is_visible_to_another_connection() {
    let f = fixture(limits()).await;
    let mut tasks = Vec::new();
    for worker in 0..32i64 {
        let backend = f.backend.clone();
        let reader = f.reader.clone();
        tasks.push(tokio::spawn(async move {
            for round in 0..8i64 {
                let id = worker * 100 + round;
                insert(&backend, Lane::Normal, id).await.unwrap();
                let found: Option<i64> = sqlx::query_scalar("SELECT id FROM t WHERE id=?1")
                    .bind(id)
                    .fetch_optional(&reader)
                    .await
                    .unwrap();
                assert_eq!(found, Some(id));
            }
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }
}

// ---- bounds ----------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_full_queue_makes_callers_wait_then_refuses_them_as_locked() {
    let f = fixture(Limits {
        queue: 4,
        wait_limit: Duration::from_millis(300),
        ..wide()
    })
    .await;
    let (first, entered, release) = held(&f, 0);
    entered.notified().await;
    let mut queued = Vec::new();
    for id in 1..=4 {
        queued.push(spawn_insert(&f, Lane::Normal, id));
    }
    until_in_flight(&f.writer, 5).await;
    let started = Instant::now();
    let overflow = insert(&f.backend, Lane::Normal, 99).await;
    let waited = started.elapsed();
    match overflow {
        Err(StoreError::Db(sqlx::Error::Database(error))) => {
            assert_eq!(error.code().as_deref(), Some("5"));
            assert_eq!(error.message(), "database is locked");
        }
        other => panic!("expected database is locked, got {other:?}"),
    }
    assert!(
        waited >= Duration::from_millis(290) && waited < Duration::from_secs(3),
        "{waited:?}"
    );
    release.notify_one();
    first.await.unwrap().unwrap();
    // The queued callers waited as long: they are refused too, never half-run.
    for task in queued {
        let _ = task.await.unwrap();
    }
    let stored = ids(&f).await;
    assert!(stored.contains(&0) && !stored.contains(&99), "{stored:?}");
    assert!(f.writer.stats().refusals >= 1);
    // The bound frees itself: a later caller is served.
    insert(&f.backend, Lane::Normal, 100).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_priority_job_waits_for_one_running_job_not_for_the_queue() {
    let f = fixture(limits()).await;
    const SLOW: Duration = Duration::from_millis(5);
    let mut tasks = Vec::new();
    for worker in 0..256i64 {
        let backend = f.backend.clone();
        tasks.push(tokio::spawn(async move {
            write_job!(backend, Lane::Normal, |tx| async {
                q("INSERT INTO t(id, v) VALUES(?1, 'slow')")
                    .bind(10_000 + worker)
                    .execute_tx(tx)
                    .await?;
                tokio::time::sleep(SLOW).await;
                Ok::<_, StoreError>(())
            })
        }));
    }
    until_in_flight(&f.writer, 200).await;
    let mut worst = Duration::ZERO;
    for id in 0..5 {
        let started = Instant::now();
        insert(&f.backend, Lane::Priority, id).await.unwrap();
        worst = worst.max(started.elapsed());
    }
    let pending = f.writer.stats().in_flight;
    // The queue holds about 256 x 5 ms of work; a priority job waits for the
    // running job, runs, and commits.
    assert!(pending >= 100, "the normal queue drained first: {pending}");
    assert!(worst < Duration::from_millis(250), "priority delay {worst:?}");
    for task in tasks {
        task.await.unwrap().unwrap();
    }
    let stats = f.writer.stats();
    assert_eq!(stats.priority_jobs, 5, "{stats:?}");
    eprintln!("priority worst {worst:?}, normal pending then {pending}, stats {stats:?}");
}

// ---- cancellation ------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_caller_that_leaves_before_its_turn_never_starts() {
    let f = fixture(wide()).await;
    let (first, entered, release) = held(&f, 1);
    entered.notified().await;
    let gone = spawn_insert(&f, Lane::Normal, 2);
    until_in_flight(&f.writer, 2).await;
    let after = spawn_insert(&f, Lane::Normal, 3);
    until_in_flight(&f.writer, 3).await;
    gone.abort();
    let _ = gone.await;
    release.notify_one();
    first.await.unwrap().unwrap();
    after.await.unwrap().unwrap();
    assert_eq!(ids(&f).await, vec![1, 3]);
    assert_eq!(f.writer.stats().skipped, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_caller_dropped_while_it_holds_the_transaction_is_undone() {
    let f = fixture(wide()).await;
    let (first, entered, release) = held(&f, 1);
    entered.notified().await;
    let (dropped, dropped_entered, _never) = held(&f, 2);
    until_in_flight(&f.writer, 2).await;
    let after = spawn_insert(&f, Lane::Normal, 3);
    until_in_flight(&f.writer, 3).await;
    release.notify_one();
    // Job 1 is answered only when its batch commits, after job 2 is gone.
    dropped_entered.notified().await; // row 2 inserted, transaction held
    dropped.abort();
    let _ = dropped.await;
    first.await.unwrap().unwrap();
    after.await.unwrap().unwrap();
    assert_eq!(ids(&f).await, vec![1, 3]);
    assert_eq!(f.writer.stats().abandoned, 1);
}

/// Cancel a two-statement job at every await point: both rows or neither.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cancelling_at_any_point_leaves_a_job_whole_or_absent() {
    let f = fixture(limits()).await;
    let mut completed = 0;
    for polls in 1..=48u32 {
        let id = i64::from(polls) * 10;
        let backend = f.backend.clone();
        let mut job = Box::pin(async move {
            write_job!(backend, Lane::Normal, |tx| async {
                q("INSERT INTO t(id, v) VALUES(?1, 'a')")
                    .bind(id)
                    .execute_tx(tx)
                    .await?;
                tokio::task::yield_now().await;
                q("INSERT INTO t(id, v) VALUES(?1, 'b')")
                    .bind(id + 1)
                    .execute_tx(tx)
                    .await?;
                Ok::<_, StoreError>(())
            })
        });
        let mut finished = false;
        for _ in 0..polls {
            if let Poll::Ready(result) = poll_fn(|cx| Poll::Ready(job.as_mut().poll(cx))).await {
                result.unwrap();
                finished = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        drop(job);
        // Let the writer finish whatever it was doing with that job.
        insert(&f.backend, Lane::Normal, id + 5).await.unwrap();
        let present: Vec<i64> = sqlx::query_scalar("SELECT id FROM t WHERE id IN (?1, ?2) ORDER BY id")
            .bind(id)
            .bind(id + 1)
            .fetch_all(&f.reader)
            .await
            .unwrap();
        assert!(
            present.is_empty() || present == vec![id, id + 1],
            "polls {polls}: {present:?}"
        );
        if finished {
            assert_eq!(present, vec![id, id + 1]);
            completed += 1;
        }
    }
    assert!(completed > 0, "no cancellation point reached completion");
    assert_eq!(f.writer.stats().in_flight, 0);
}

// ---- shutdown ----------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn close_answers_every_submitted_job_then_refuses_new_ones() {
    let f = fixture(limits()).await;
    let mut tasks = Vec::new();
    for worker in 0..32i64 {
        let backend = f.backend.clone();
        tasks.push(tokio::spawn(async move {
            write_job!(backend, Lane::Normal, |tx| async {
                q("INSERT INTO t(id, v) VALUES(?1, 'x')")
                    .bind(worker)
                    .execute_tx(tx)
                    .await?;
                tokio::time::sleep(Duration::from_millis(2)).await;
                Ok::<_, StoreError>(())
            })
        }));
    }
    until_in_flight(&f.writer, 16).await;
    let Backend::Sqlite(live) = &f.backend else {
        unreachable!()
    };
    live.close().await;
    for task in tasks {
        task.await.unwrap().unwrap();
    }
    assert_eq!(ids(&f).await.len(), 32);
    assert_eq!(f.writer.stats().in_flight, 0);
    assert!(matches!(
        insert(&f.backend, Lane::Normal, 99).await,
        Err(StoreError::Db(sqlx::Error::PoolClosed))
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn drain_waits_for_submitted_jobs_and_keeps_admitting() {
    let f = fixture(wide()).await;
    let (first, entered, release) = held(&f, 1);
    entered.notified().await;
    let queued = spawn_insert(&f, Lane::Normal, 2);
    until_in_flight(&f.writer, 2).await;
    assert!(
        !f.writer
            .drain(tokio::time::Instant::now() + Duration::from_millis(50))
            .await
    );
    release.notify_one();
    assert!(
        f.writer
            .drain(tokio::time::Instant::now() + Duration::from_secs(5))
            .await
    );
    first.await.unwrap().unwrap();
    queued.await.unwrap().unwrap();
    insert(&f.backend, Lane::Normal, 3).await.unwrap();
    assert_eq!(ids(&f).await, vec![1, 2, 3]);
}

// ---- the converted store paths -------------------------------------------------------

/// A file store whose backend has a group writer (the managed store keeps
/// its own lifecycle; this unmanaged pool shares its file and options).
async fn group_store(dir: &Path, limits: Limits) -> (crate::Store, crate::Store, Arc<GroupWriter>) {
    let managed = crate::Store::open_with_encryption(
        dir.join("mail.db").to_str().unwrap(),
        crate::ServerKey::generate(),
        &crate::encryption::EncryptionConfig::off(),
    )
    .await
    .unwrap();
    let Backend::Sqlite(live) = &managed.backend else {
        unreachable!()
    };
    let options = (*live.pin().await.active_options()).clone();
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await
        .unwrap();
    let writer = GroupWriter::with_limits(limits);
    let mut grouped = managed.clone();
    grouped.backend = Backend::Sqlite(LiveSqlite::unmanaged_with_writer(pool, writer.clone()));
    (managed, grouped, writer)
}

async fn seed_account(s: &crate::Store) -> String {
    let name = format!("writer-{}@example.test", uuid::Uuid::new_v4().simple());
    s.create_account(
        &crate::NewAccount {
            kind: crate::AccountKind::Imap,
            host: "writer.test",
            port: 993,
            tls: "implicit",
            username: &name,
            sync_policy_json: "{}",
        },
        &crate::Credentials {
            username: name.clone(),
            password: "synthetic-only".into(),
        },
    )
    .await
    .unwrap()
}

async fn seed_mailbox(s: &crate::Store, account: &str, name: &str) -> String {
    s.upsert_mailbox(&crate::MailboxUpsert {
        account_id: account,
        name,
        role: None,
        uidvalidity: 100,
        uidnext: 1,
        highestmodseq: 0,
        total: 0,
        unread: 0,
        parent_id: None,
    })
    .await
    .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn grouped_flag_writes_keep_the_projection_and_the_counters_exact() {
    let dir = tempfile::tempdir().unwrap();
    let (managed, s, writer) = group_store(dir.path(), limits()).await;
    let account = seed_account(&s).await;
    let boxes = [
        seed_mailbox(&s, &account, "INBOX").await,
        seed_mailbox(&s, &account, "Archive").await,
    ];
    let mut ids = Vec::new();
    for uid in 1..=240u32 {
        let mailbox = &boxes[(uid % 2) as usize];
        let mid = format!("<m{uid}@writer.test>");
        let flags = if uid % 3 == 0 { r#"["Seen"]"# } else { "[]" };
        ids.push(
            s.upsert_message(&crate::MessageUpsert {
                account_id: &account,
                mailbox_id: mailbox,
                uid,
                uidvalidity: 100,
                message_id: Some(&mid),
                thread_id: None,
                internaldate: Some("2026-11-24T10:00:00Z"),
                size: 100,
                flags_json: flags,
                envelope: None,
                blob_ref: None,
            })
            .await
            .unwrap(),
        );
    }
    let jobs_before = writer.stats().jobs;
    let flag_sets = [
        r#"[]"#,
        r#"["Seen"]"#,
        r#"["Flagged"]"#,
        r#"["Seen","Flagged"]"#,
        r#"[{"Keyword":"$label1"}]"#,
    ];
    let ids = Arc::new(ids);
    let mut tasks = Vec::new();
    for worker in 0..48usize {
        let (s, ids, account) = (s.clone(), ids.clone(), account.clone());
        tasks.push(tokio::spawn(async move {
            let mut state = worker as u64 * 7919 + 1;
            for round in 0..20usize {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let id = &ids[(state >> 33) as usize % ids.len()];
                let flags = flag_sets[(state >> 20) as usize % flag_sets.len()];
                match (worker + round) % 3 {
                    0 => {
                        s.set_flags(id, flags).await.unwrap();
                    }
                    1 => {
                        let other = &ids[(state >> 40) as usize % ids.len()];
                        s.set_flags_batch(&[(id.clone(), flags.to_owned()), (other.clone(), flags.to_owned())])
                            .await
                            .unwrap();
                    }
                    _ => {
                        let current = s.get_message(id).await.unwrap();
                        s.set_flags_if_current(&current, flags).await.unwrap();
                    }
                }
                s.record_changes(&account, "Email", std::slice::from_ref(id), "updated")
                    .await
                    .unwrap();
            }
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }
    let audit = s.audit_message_projection(None).await.unwrap();
    assert!(audit.is_clean(), "drift {} rows, first {:?}", audit.drift, audit.sample);
    for r in q("SELECT b.id, b.total, b.unread,
            (SELECT COUNT(*) FROM messages m WHERE m.mailbox_id = b.id) AS rows_total,
            (SELECT COUNT(*) FROM message_state t WHERE t.mailbox_id = b.id AND t.seen = 0) AS rows_unread
         FROM mailboxes b")
    .fetch_all(&s.backend)
    .await
    .unwrap()
    {
        assert_eq!(
            r.get_i64("unread"),
            r.get_i64("rows_unread"),
            "unread counter of {}",
            r.get_string("id")
        );
    }
    // Change-log states stay one per append, strictly increasing.
    let states: Vec<i64> = q("SELECT DISTINCT state FROM changes WHERE account_id=?1 AND type='Email' ORDER BY state")
        .bind(&account)
        .fetch_all(&s.backend)
        .await
        .unwrap()
        .iter()
        .map(|r| r.get_i64_idx(0))
        .collect();
    assert_eq!(states, (1..=960).collect::<Vec<_>>());
    let stats = writer.stats();
    assert!(stats.jobs - jobs_before >= 960 + 640, "{stats:?}");
    assert!(stats.commits < stats.jobs, "{stats:?}");
    eprintln!("grouped flag writes: {stats:?}");
    let Backend::Sqlite(live) = &s.backend else {
        unreachable!()
    };
    live.close().await;
    managed.close().await;
}

/// The request-protection group goes through the writer (priority lane by
/// default, normal for the timer flush) and decides exactly as the same
/// groups as ordinary transactions do.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn request_protection_groups_run_as_writer_jobs_with_equal_decisions() {
    use crate::request_protection::{ProtectionObservation, ProtectionSource};
    use crate::request_protection_model::{Mode, Policy, Signal, WindowPolicy};
    let policy = Policy {
        rate_mode: Mode::Enforce,
        adaptive_mode: Mode::Enforce,
        rate_limit: 3,
        rate_window_secs: 60,
        short: WindowPolicy {
            seconds: 60,
            minimum_samples: 2,
            absolute_anomalies: 2,
            ratio_percent: 50,
        },
        long: WindowPolicy {
            seconds: 3600,
            minimum_samples: 2,
            absolute_anomalies: 2,
            ratio_percent: 50,
        },
        normal_cap_per_bucket: 1000,
        signals: 63,
        throttle_secs: 3,
        block_secs: 300,
        decay_secs: 3600,
        max_stage: 2,
    };
    let dir = tempfile::tempdir().unwrap();
    let (managed, s, writer) = group_store(dir.path(), limits()).await;
    let serial = crate::Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let signals = [
        None,
        Some(Signal::Normal),
        None,
        Some(Signal::CredentialRejected),
        None,
        None,
        Some(Signal::ProbePath),
        None,
    ];
    let requests: Vec<_> = signals
        .iter()
        .enumerate()
        .map(|(i, signal)| ProtectionObservation {
            kind: "account",
            value: if i % 3 == 0 { "account-b" } else { "account-a" },
            policy: &policy,
            now: 1000 + i as i64,
            signal: *signal,
            source: ProtectionSource {
                country: Some("PT"),
                ip: Some("192.0.2.10"),
            },
        })
        .collect();
    let grouped_participant = s.request_protection_participant().await.unwrap();
    let serial_participant = serial.request_protection_participant().await.unwrap();
    let before = writer.stats();
    let mut grouped = Vec::new();
    let mut plain = Vec::new();
    for (i, chunk) in requests.chunks(2).enumerate() {
        let lane = if i % 2 == 0 { Lane::Normal } else { Lane::Priority };
        grouped.extend(
            s.request_protection_sqlite_batch_in(&grouped_participant, chunk, lane)
                .await
                .unwrap(),
        );
        plain.extend(
            serial
                .request_protection_sqlite_batch_as(&serial_participant, chunk)
                .await
                .unwrap(),
        );
    }
    let decisions = |v: &[(
        crate::request_protection::ProtectionOutcome,
        Option<crate::request_protection::StoredProtection>,
    )]| {
        v.iter()
            .map(|(o, stored)| format!("{:?} {:?}", o.decision, stored.as_ref().map(|s| &s.state)))
            .collect::<Vec<_>>()
    };
    assert_eq!(decisions(&grouped), decisions(&plain));
    let after = writer.stats();
    assert_eq!(after.jobs - before.jobs, 4, "{after:?}");
    assert_eq!(after.priority_jobs - before.priority_jobs, 2, "{after:?}");
    let Backend::Sqlite(live) = &s.backend else {
        unreachable!()
    };
    live.close().await;
    managed.close().await;
}
