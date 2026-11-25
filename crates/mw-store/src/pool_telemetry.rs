//! Payload-free operation/acquisition counters. No SQL text, binds, IDs or
//! credentials are retained. Query duration includes acquisition wait; timings
//! overlap and must not be added together. Direct SQLx outside q() is excluded.
//!
//! Commits are counted per backend: `commits` counts `Tx::commit` (one per
//! store transaction, timed from the COMMIT call to its return, so on SQLite it
//! includes the WAL write and on PostgreSQL the round trip and any synchronous
//! flush); `autocommit_writes` counts write statements (INSERT, UPDATE, DELETE,
//! REPLACE) that ran outside a transaction, each of which SQLite and
//! PostgreSQL commit on their own. Their sum is the number of database commits
//! the counted statements caused. Commits made by direct SQLx calls outside
//! q()/Tx are not seen.
#[cfg(feature = "query-shape-diagnostics")]
#[path = "query_shapes.rs"]
pub mod shapes;

use serde::Serialize;
use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationSnapshot {
    pub calls: u64,
    pub failures: u64,
    pub total_micros: u64,
    pub max_micros: u64,
}
impl OperationSnapshot {
    fn merge(&mut self, other: &Self) {
        self.calls = self.calls.saturating_add(other.calls);
        self.failures = self.failures.saturating_add(other.failures);
        self.total_micros = self.total_micros.saturating_add(other.total_micros);
        self.max_micros = self.max_micros.max(other.max_micros);
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySnapshot {
    pub read: OperationSnapshot,
    pub write: OperationSnapshot,
    pub other: OperationSnapshot,
    pub acquire: OperationSnapshot,
    /// `Tx::commit` calls, per backend, timed.
    pub commits: CommitSnapshot,
    /// Successful write statements executed outside a transaction (each one is
    /// its own commit).
    pub autocommit_writes: u64,
    /// SQLite in-process write gate, summed over the process's managed
    /// databases. Only in [`process_snapshot`], and only once a gate exists
    /// (`MW_SQLITE_WRITER=gate` on a managed SQLite database); never in a
    /// per-request snapshot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_gate: Option<WriteGateStats>,
    /// SQLite group-commit writer (`MW_SQLITE_WRITER=group`), summed over the
    /// process's managed databases; like `write_gate`, only in
    /// [`process_snapshot`] and only once a writer exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sqlite_writer: Option<SqliteWriterStats>,
}
pub use crate::sqlite_live::sqlite_writer::SqliteWriterStats;
pub use crate::sqlite_live::write_gate::WriteGateStats;
/// Commit counters by backend.
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitSnapshot {
    pub sqlite: OperationSnapshot,
    pub postgres: OperationSnapshot,
}
impl CommitSnapshot {
    /// Commit calls on both backends.
    pub fn calls(&self) -> u64 {
        self.sqlite.calls.saturating_add(self.postgres.calls)
    }
}
impl QuerySnapshot {
    pub fn merge(&mut self, other: &Self) {
        self.read.merge(&other.read);
        self.write.merge(&other.write);
        self.other.merge(&other.other);
        self.acquire.merge(&other.acquire);
        self.commits.sqlite.merge(&other.commits.sqlite);
        self.commits.postgres.merge(&other.commits.postgres);
        self.autocommit_writes = self.autocommit_writes.saturating_add(other.autocommit_writes);
    }
    /// Database commits caused by the counted work: transaction commits plus
    /// autocommit write statements.
    pub fn database_commits(&self) -> u64 {
        self.commits.calls().saturating_add(self.autocommit_writes)
    }
}
#[derive(Default)]
struct AtomicOperation {
    calls: AtomicU64,
    failures: AtomicU64,
    total: AtomicU64,
    maximum: AtomicU64,
}
impl AtomicOperation {
    fn add(&self, micros: u64, success: bool) {
        self.calls.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.failures.fetch_add(1, Ordering::Relaxed);
        }
        self.total.fetch_add(micros, Ordering::Relaxed);
        self.maximum.fetch_max(micros, Ordering::Relaxed);
    }
    fn snapshot(&self) -> OperationSnapshot {
        OperationSnapshot {
            calls: self.calls.load(Ordering::Relaxed),
            failures: self.failures.load(Ordering::Relaxed),
            total_micros: self.total.load(Ordering::Relaxed),
            max_micros: self.maximum.load(Ordering::Relaxed),
        }
    }
}
#[derive(Default)]
struct Counters {
    read: AtomicOperation,
    write: AtomicOperation,
    other: AtomicOperation,
    acquire: AtomicOperation,
    commit_sqlite: AtomicOperation,
    commit_postgres: AtomicOperation,
    autocommit_writes: AtomicU64,
}
impl Counters {
    fn operation(&self, class: Class) -> &AtomicOperation {
        match class {
            Class::Read => &self.read,
            Class::Write => &self.write,
            Class::Other => &self.other,
            Class::Acquire => &self.acquire,
        }
    }
    fn snapshot(&self) -> QuerySnapshot {
        QuerySnapshot {
            read: self.read.snapshot(),
            write: self.write.snapshot(),
            other: self.other.snapshot(),
            acquire: self.acquire.snapshot(),
            commits: CommitSnapshot {
                sqlite: self.commit_sqlite.snapshot(),
                postgres: self.commit_postgres.snapshot(),
            },
            autocommit_writes: self.autocommit_writes.load(Ordering::Relaxed),
            write_gate: None,
            sqlite_writer: None,
        }
    }
}
static PROCESS: std::sync::LazyLock<Counters> = std::sync::LazyLock::new(Counters::default);
tokio::task_local! { static REQUEST: Arc<Counters>; }
/// Fresh task-local measurement boundary; never retain it across HTTP requests
/// or WebSocket frames. Spawned child tasks must arrange their own propagation.
pub async fn measure_queries<F: Future>(future: F) -> (F::Output, QuerySnapshot) {
    let counters = Arc::new(Counters::default());
    let output = REQUEST.scope(counters.clone(), future).await;
    (output, counters.snapshot())
}
/// Carry the caller's per-request measurement (if any) into a future that will
/// run on another task (`tokio::spawn`), so its statements still count for the
/// request that caused them.
pub fn in_current_measurement<F: Future>(future: F) -> impl Future<Output = F::Output> {
    match REQUEST.try_with(Arc::clone) {
        Ok(counters) => futures_util::future::Either::Left(REQUEST.scope(counters, future)),
        Err(_) => futures_util::future::Either::Right(future),
    }
}
/// Process-wide aggregate across configured stores. This is not per-tenant data.
pub fn process_snapshot() -> QuerySnapshot {
    let mut snapshot = PROCESS.snapshot();
    snapshot.write_gate = crate::sqlite_live::write_gate::process_stats();
    snapshot.sqlite_writer = crate::sqlite_live::sqlite_writer::process_stats();
    snapshot
}
#[derive(Clone, Copy)]
enum Class {
    Read,
    Write,
    Other,
    Acquire,
}
pub(crate) struct QueryTimer {
    class: Class,
    started: Instant,
    request: Option<Arc<Counters>>,
    success: bool,
    direct: bool,
    #[cfg(feature = "query-shape-diagnostics")]
    shape: Option<shapes::Capture>,
}
impl QueryTimer {
    pub(crate) fn start(sql: &'static str) -> Self {
        let verb = sql.split_ascii_whitespace().next().unwrap_or("");
        let class = if verb.eq_ignore_ascii_case("SELECT") {
            Class::Read
        } else if ["INSERT", "UPDATE", "DELETE", "REPLACE"]
            .iter()
            .any(|candidate| verb.eq_ignore_ascii_case(candidate))
        {
            Class::Write
        } else {
            Class::Other
        };
        #[cfg(not(feature = "query-shape-diagnostics"))]
        { Self::new(class) }
        #[cfg(feature = "query-shape-diagnostics")]
        {
            let mut timer=Self::new(class);
            timer.shape=shapes::capture(sql,class);
            timer
        }
    }
    #[cfg(feature = "query-shape-diagnostics")]
    pub(crate) fn returned_rows(&mut self,rows:u64){if let Some(shape)=&mut self.shape{shape.returned=Some(rows);}}
    #[cfg(feature = "query-shape-diagnostics")]
    pub(crate) fn affected_rows(&mut self,rows:u64){if let Some(shape)=&mut self.shape{shape.affected=Some(rows);}}
    fn new(class: Class) -> Self {
        Self {
            class,
            started: Instant::now(),
            request: REQUEST.try_with(Arc::clone).ok(),
            success: false,
            direct: false,
            #[cfg(feature = "query-shape-diagnostics")]
            shape:None,
        }
    }
    pub(crate) fn success(&mut self) {
        self.success = true;
    }
    /// The statement runs on the backend outside any transaction: a write
    /// is then its own commit and is counted in `autocommit_writes`.
    pub(crate) fn direct(&mut self) {
        self.direct = true;
    }
}
impl Drop for QueryTimer {
    fn drop(&mut self) {
        let micros = self.started.elapsed().as_micros().min(u64::MAX as u128) as u64;
        #[cfg(feature = "query-shape-diagnostics")]
        if let Some(shape)=self.shape.take(){shape.finish(self.success,micros);}
        PROCESS.operation(self.class).add(micros, self.success);
        let autocommit = self.direct && self.success && matches!(self.class, Class::Write);
        if autocommit {
            PROCESS.autocommit_writes.fetch_add(1, Ordering::Relaxed);
        }
        if let Some(counters) = &self.request {
            counters.operation(self.class).add(micros, self.success);
            if autocommit {
                counters.autocommit_writes.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}
/// Which backend a commit ran on.
#[derive(Clone, Copy)]
pub(crate) enum CommitBackend {
    Sqlite,
    Postgres,
}
/// Times one transaction commit; counted on drop (a failed or cancelled
/// commit is a call and a failure).
pub(crate) struct CommitTimer {
    backend: CommitBackend,
    started: Instant,
    request: Option<Arc<Counters>>,
    success: bool,
}
impl CommitTimer {
    pub(crate) fn start(backend: CommitBackend) -> Self {
        Self {
            backend,
            started: Instant::now(),
            request: REQUEST.try_with(Arc::clone).ok(),
            success: false,
        }
    }
    pub(crate) fn success(&mut self) {
        self.success = true;
    }
}
impl Drop for CommitTimer {
    fn drop(&mut self) {
        let micros = self.started.elapsed().as_micros().min(u64::MAX as u128) as u64;
        let add = |counters: &Counters| match self.backend {
            CommitBackend::Sqlite => counters.commit_sqlite.add(micros, self.success),
            CommitBackend::Postgres => counters.commit_postgres.add(micros, self.success),
        };
        add(&PROCESS);
        if let Some(counters) = &self.request {
            add(counters);
        }
    }
}
pub(crate) async fn acquire<DB: sqlx::Database>(
    pool: &sqlx::Pool<DB>,
) -> Result<sqlx::pool::PoolConnection<DB>, sqlx::Error> {
    let mut timer = QueryTimer::new(Class::Acquire);
    let connection = pool.acquire().await?;
    timer.success();
    Ok(connection)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn scoped_counts_track_failures_and_do_not_retain_previous_request() {
        let store = crate::Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        let (_, snapshot) = measure_queries(async {
            crate::backend::q("CREATE TABLE telemetry_probe(id INTEGER)")
                .execute(&store.backend)
                .await
                .unwrap();
            crate::backend::q("INSERT INTO telemetry_probe VALUES(1)")
                .execute(&store.backend)
                .await
                .unwrap();
            crate::backend::q("SELECT COUNT(*) FROM telemetry_probe")
                .fetch_scalar_i64(&store.backend)
                .await
                .unwrap();
            assert!(
                crate::backend::q("SELECT missing_column FROM telemetry_probe")
                    .fetch_all(&store.backend)
                    .await
                    .is_err()
            );
        })
        .await;
        assert_eq!(snapshot.read.calls, 2);
        assert_eq!(snapshot.read.failures, 1);
        assert_eq!(snapshot.write.calls, 1);
        assert_eq!(snapshot.other.calls, 1);
        assert_eq!(snapshot.acquire.calls, 4);
        let (_, fresh) = measure_queries(async {}).await;
        assert_eq!(
            fresh.read.calls + fresh.write.calls + fresh.other.calls + fresh.acquire.calls,
            0
        );
        store.close().await;
    }
    #[tokio::test]
    async fn commits_and_autocommit_writes_are_counted_per_backend() {
        let store = crate::Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        crate::backend::q("CREATE TABLE commit_probe(id INTEGER)")
            .execute(&store.backend)
            .await
            .unwrap();
        let process_before = process_snapshot();
        let (_, snapshot) = measure_queries(async {
            // Two autocommit writes, one autocommit read (not a commit).
            for id in 1..=2i64 {
                crate::backend::q("INSERT INTO commit_probe VALUES(?1)")
                    .bind(id)
                    .execute(&store.backend)
                    .await
                    .unwrap();
            }
            crate::backend::q("SELECT COUNT(*) FROM commit_probe")
                .fetch_scalar_i64(&store.backend)
                .await
                .unwrap();
            // One transaction with two writes: one commit, no autocommit writes.
            let mut tx = store.backend.begin().await.unwrap();
            for id in 3..=4i64 {
                crate::backend::q("INSERT INTO commit_probe VALUES(?1)")
                    .bind(id)
                    .execute_tx(&mut tx)
                    .await
                    .unwrap();
            }
            tx.commit().await.unwrap();
            // A rolled-back transaction is not a commit.
            let mut tx = store.backend.begin().await.unwrap();
            crate::backend::q("INSERT INTO commit_probe VALUES(5)")
                .execute_tx(&mut tx)
                .await
                .unwrap();
            tx.rollback().await.unwrap();
            // A failed autocommit write is not a commit.
            assert!(
                crate::backend::q("INSERT INTO missing_table VALUES(1)")
                    .execute(&store.backend)
                    .await
                    .is_err()
            );
        })
        .await;
        assert_eq!(snapshot.commits.sqlite.calls, 1);
        assert_eq!(snapshot.commits.sqlite.failures, 0);
        assert_eq!(snapshot.commits.postgres.calls, 0);
        assert_eq!(snapshot.autocommit_writes, 2);
        assert_eq!(snapshot.database_commits(), 3);
        assert_eq!(snapshot.write.calls, 6);
        // The process-wide counters move by at least the same amounts.
        let process_after = process_snapshot();
        assert!(process_after.commits.sqlite.calls >= process_before.commits.sqlite.calls + 1);
        assert!(process_after.autocommit_writes >= process_before.autocommit_writes + 2);
        let json = serde_json::to_value(snapshot).unwrap();
        assert_eq!(json["commits"]["sqlite"]["calls"], 1);
        assert_eq!(json["autocommitWrites"], 2);
        let mut merged = snapshot;
        merged.merge(&snapshot);
        assert_eq!(merged.database_commits(), 6);
        store.close().await;
    }
    #[tokio::test]
    async fn acquisition_timeout_is_visible_as_wait_and_failure_without_sql_payload() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .acquire_timeout(std::time::Duration::from_millis(80))
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let backend = crate::backend::Backend::Sqlite(pool.into());
        let crate::backend::Backend::Sqlite(live) = &backend else {
            unreachable!()
        };
        let held = live.acquire().await.unwrap();
        let (result, snapshot) =
            measure_queries(crate::backend::q("SELECT 1").fetch_scalar_i64(&backend)).await;
        assert!(matches!(result, Err(sqlx::Error::PoolTimedOut)));
        assert_eq!(snapshot.read.calls, 1);
        assert_eq!(snapshot.read.failures, 1);
        assert_eq!(snapshot.acquire.calls, 1);
        assert_eq!(snapshot.acquire.failures, 1);
        assert!(snapshot.acquire.total_micros >= 10_000);
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(!json.contains("SELECT"));
        // No connection string (the per-backend commit keys are field names).
        assert!(!json.contains("sqlite:"));
        assert!(!json.contains("memory"));
        drop(held);
        live.close().await;
    }
}
