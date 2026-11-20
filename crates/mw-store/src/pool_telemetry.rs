//! Payload-free operation/acquisition counters. No SQL text, binds, IDs or
//! credentials are retained. Query duration includes acquisition wait; timings
//! overlap and must not be added together. Direct SQLx outside q() is excluded.
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
}
impl QuerySnapshot {
    pub fn merge(&mut self, other: &Self) {
        self.read.merge(&other.read);
        self.write.merge(&other.write);
        self.other.merge(&other.other);
        self.acquire.merge(&other.acquire);
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
/// Process-wide aggregate across configured stores. This is not per-tenant data.
pub fn process_snapshot() -> QuerySnapshot {
    PROCESS.snapshot()
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
            #[cfg(feature = "query-shape-diagnostics")]
            shape:None,
        }
    }
    pub(crate) fn success(&mut self) {
        self.success = true;
    }
}
impl Drop for QueryTimer {
    fn drop(&mut self) {
        let micros = self.started.elapsed().as_micros().min(u64::MAX as u128) as u64;
        #[cfg(feature = "query-shape-diagnostics")]
        if let Some(shape)=self.shape.take(){shape.finish(self.success,micros);}
        PROCESS.operation(self.class).add(micros, self.success);
        if let Some(counters) = &self.request {
            counters.operation(self.class).add(micros, self.success);
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
        assert!(!json.contains("sqlite"));
        drop(held);
        live.close().await;
    }
}
