//! Generation-aware SQLite admission. Raw connections never outlive their read lease.
//!
//! An exclusive lease stops new work, drains queries/transactions, and owns pool
//! replacement. This is an application admission gate, not cross-process fencing.
use futures_util::{TryStreamExt, future::BoxFuture, stream::BoxStream};
use sqlx::{
    Describe, Either, Error, Execute, Executor, Sqlite, SqliteConnection, SqlitePool, Transaction,
    pool::PoolConnection,
    sqlite::{SqliteConnectOptions, SqliteQueryResult, SqliteRow, SqliteStatement, SqliteTypeInfo},
};
use std::{
    collections::HashMap,
    fmt,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, Mutex, Weak, OnceLock},
};
use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

#[derive(Clone)]
pub(crate) struct LiveSqlite {
    inner: Arc<Inner>,
}
struct Inner {
    generation: Arc<RwLock<Generation>>,
    logical_path: PathBuf,
    files: Option<Arc<crate::generation_files::GenerationFiles>>,
    encrypted: bool,
    rebuild: Arc<tokio::sync::Mutex<()>>,
    cache_policy: OnceLock<Option<u32>>,
    cache_registry: Arc<crate::sqlite_page_cache::Registry>,
}
struct Generation {
    pool: SqlitePool,
    cache_kib: Option<u32>,
}
impl Drop for Inner {
    fn drop(&mut self) {
        let Some(files) = self.files.clone() else {
            return;
        };
        // PoolConnection return and SQLite worker shutdown can finish after Drop.
        // Keep cooperative process fencing until an explicit pool drain completes,
        // including when the application's Tokio runtime has already stopped.
        let cleanup = Arc::new((self.generation.clone(), files));
        let owned = cleanup.clone();
        let started = std::thread::Builder::new()
            .name("mw-sqlite-close".into())
            .spawn(move || {
                match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime.block_on(async {
                        let generation = owned.0.write().await;
                        generation.pool.close().await;
                    }),
                    // A failure to drain must not silently release the lifecycle fence.
                    Err(_) => std::mem::forget(owned),
                }
            });
        if started.is_err() {
            std::mem::forget(cleanup);
        }
    }
}
impl fmt::Debug for LiveSqlite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LiveSqlite")
            .field("logical_path", &self.inner.logical_path)
            .finish_non_exhaustive()
    }
}
impl From<SqlitePool> for LiveSqlite {
    fn from(pool: SqlitePool) -> Self {
        Self::unmanaged(pool)
    }
}
impl LiveSqlite {
    pub(crate) fn select_page_cache_policy(&self,requested:Option<u32>)->Option<u32>{
        *self.inner.cache_policy.get_or_init(||requested)
    }
    pub(crate) fn page_cache_registry(&self)->Arc<crate::sqlite_page_cache::Registry>{self.inner.cache_registry.clone()}
    /// Private/offline/test pools still use admission guards, without a filesystem owner lease.
    pub(crate) fn unmanaged(pool: SqlitePool) -> Self {
        let logical_path = pool.connect_options().get_filename().to_path_buf();
        Self {
            inner: Arc::new(Inner {
                generation: Arc::new(RwLock::new(Generation { pool, cache_kib:None })),
                logical_path,
                files: None,
                encrypted: false,
                rebuild: Arc::new(tokio::sync::Mutex::new(())),
                cache_policy: OnceLock::new(),
                cache_registry: Arc::new(Default::default()),
            }),
        }
    }
    /// Full rebuilds can change implicit rowids without ordinary DML triggers.
    /// Conversion and every live full-VACUUM entrypoint share this exclusion gate.
    pub(crate) fn try_rebuild_guard(&self) -> Result<tokio::sync::OwnedMutexGuard<()>, Error> {
        self.inner.rebuild.clone().try_lock_owned().map_err(|_| {
            Error::Protocol(
                "A SQLite conversion or full rebuild is already running; retry after it completes"
                    .into(),
            )
        })
    }
    pub(crate) async fn full_vacuum(&self) -> Result<(), Error> {
        let guard = self.try_rebuild_guard()?;
        let live = self.clone();
        // A cancelled request must not release rebuild exclusion while SQLx's
        // worker is still executing the uncancellable SQLite VACUUM statement.
        tokio::spawn(async move {
            let _guard = guard;
            (&live).execute("VACUUM").await?;
            Ok::<_, Error>(())
        })
        .await
        .map_err(|error| Error::Protocol(format!("SQLite rebuild task failed: {error}")))?
    }
    pub(crate) fn is_encrypted(&self) -> bool {
        self.inner.encrypted
    }
    pub(crate) fn is_managed(&self) -> bool {
        self.inner.files.is_some()
    }
    pub(crate) fn files(&self) -> Option<Arc<crate::generation_files::GenerationFiles>> {
        self.inner.files.clone()
    }
    /// Serialize first opens/reopens of one canonical logical database within this process.
    pub(crate) async fn opening(path: &Path) -> Result<Opening, Error> {
        let logical =
            crate::generation_files::canonical_logical_path(path).map_err(generation_error)?;
        let gate = {
            let mut registry = REGISTRY.lock().unwrap_or_else(|p| p.into_inner());
            if registry.len() > 256 {
                registry.retain(|_, entry| {
                    entry.gate.strong_count() > 0
                        || entry.live.strong_count() > 0
                        || entry.generation.strong_count() > 0
                });
            }
            let entry = registry.entry(logical.clone()).or_default();
            match entry.gate.upgrade() {
                Some(gate) => gate,
                None => {
                    let gate = Arc::new(tokio::sync::Mutex::new(()));
                    entry.gate = Arc::downgrade(&gate);
                    gate
                }
            }
        };
        let guard = gate.lock_owned().await;
        let (existing, generation) = {
            let registry = REGISTRY.lock().unwrap_or_else(|p| p.into_inner());
            let entry = registry.get(&logical).expect("opening registry entry");
            (
                entry.live.upgrade().map(|inner| Self { inner }),
                entry.generation.upgrade(),
            )
        };
        if existing.is_none() {
            if let Some(generation) = generation {
                // Last-owner Drop drains workers asynchronously. Shared process
                // leases alone do not exclude a new same-process header probe:
                // closing its std File could release the old workers' POSIX
                // record locks. Complete that generation's drain before allowing
                // any new pool/probe, without holding the registry mutex.
                let closing = generation.write().await;
                closing.pool.close().await;
            }
        }
        let files = match existing.as_ref().and_then(Self::files) {
            Some(files) => files,
            None => Arc::new(
                crate::generation_files::GenerationFiles::shared(&logical)
                    .map_err(generation_error)?,
            ),
        };
        Ok(Opening {
            logical,
            files,
            existing,
            _guard: guard,
        })
    }
    pub(crate) fn logical_path(&self) -> &Path {
        &self.inner.logical_path
    }
    pub(crate) async fn pin(&self) -> PinnedPool {
        PinnedPool {
            generation: self.inner.generation.clone().read_owned().await,
            _owner: self.inner.clone(),
        }
    }
    pub(crate) async fn acquire(&self) -> Result<GuardedConnection, Error> {
        let pin = self.pin().await;
        let connection = acquire_pinned(&pin).await?;
        Ok(GuardedConnection { connection, pin })
    }
    pub(crate) async fn begin(&self) -> Result<LiveTransaction, Error> {
        let pin = self.pin().await;
        let connection = acquire_pinned(&pin).await?;
        // SQLx custom-BEGIN completion contains an await before its rollback guard
        // exists. An owning task covers both that transition and the admission lease.
        tokio::spawn(async move {
            let transaction =
                Transaction::begin(connection, Some("BEGIN IMMEDIATE".into())).await?;
            Ok(LiveTransaction {
                transaction: Some(transaction),
                pin,
            })
        })
        .await
        .map_err(|error| Error::Protocol(format!("SQLite transaction task failed: {error}")))?
    }
    pub(crate) async fn exclusive(&self) -> ExclusiveSqlite {
        ExclusiveSqlite {
            generation: self.inner.generation.clone().write_owned().await,
            _owner: self.inner.clone(),
        }
    }
    pub(crate) async fn close(&self) {
        self.exclusive().await.close_old().await;
    }
}

async fn acquire_pinned(pin: &PinnedPool) -> Result<PoolConnection<Sqlite>, Error> {
    let pool = pin.pool();
    let result = crate::pool_telemetry::acquire(pool).await;
    if matches!(&result, Err(Error::PoolTimedOut)) {
        // No SQL, account, path or credentials. PoolTimedOut also covers new
        // connection setup, not just demand exceeding the connection ceiling.
        eprintln!(
            "[mw-store] SQLite acquisition timed out after generation admission: open={}, idle={}, maximum={}, closed={}; deadline includes connection setup and validation",
            pool.size(),
            pool.num_idle(),
            pool.options().get_max_connections(),
            pool.is_closed()
        );
    }
    result
}

#[derive(Default)]
struct RegistryEntry {
    gate: Weak<tokio::sync::Mutex<()>>,
    live: Weak<Inner>,
    generation: Weak<RwLock<Generation>>,
}
static REGISTRY: LazyLock<Mutex<HashMap<PathBuf, RegistryEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
fn generation_error(error: crate::generation_files::GenerationError) -> Error {
    Error::Protocol(error.to_string())
}
pub(crate) struct Opening {
    logical: PathBuf,
    pub(crate) files: Arc<crate::generation_files::GenerationFiles>,
    pub(crate) existing: Option<LiveSqlite>,
    _guard: tokio::sync::OwnedMutexGuard<()>,
}
impl Opening {
    pub(crate) fn active_path(&self) -> Result<PathBuf, Error> {
        Ok(self.files.resolve_active().map_err(generation_error)?.path)
    }
    pub(crate) fn register(&self, pool: SqlitePool, encrypted: bool) -> LiveSqlite {
        let live = LiveSqlite {
            inner: Arc::new(Inner {
                generation: Arc::new(RwLock::new(Generation { pool, cache_kib:None })),
                logical_path: self.logical.clone(),
                files: Some(self.files.clone()),
                encrypted,
                rebuild: Arc::new(tokio::sync::Mutex::new(())),
                cache_policy: OnceLock::new(),
                cache_registry: Arc::new(Default::default()),
            }),
        };
        let mut registry = REGISTRY.lock().unwrap_or_else(|p| p.into_inner());
        let entry = registry.entry(self.logical.clone()).or_default();
        entry.live = Arc::downgrade(&live.inner);
        entry.generation = Arc::downgrade(&live.inner.generation);
        live
    }
}

/// A pinned generation. Keep this alive until every derived private pool is closed.
/// Never clone the raw pool into a task unless that task also owns this pin.
pub(crate) struct PinnedPool {
    generation: OwnedRwLockReadGuard<Generation>,
    // Release admission before the last owner can schedule its final pool drain.
    _owner: Arc<Inner>,
}
impl PinnedPool {
    pub(crate) fn auxiliary_options(&self)->SqliteConnectOptions {
        crate::sqlite_page_cache::auxiliary((*self.active_options()).clone(),self._owner.cache_policy.get().copied().flatten().is_some())
    }
    pub(crate) fn page_cache_policy(&self)->Option<u32>{self._owner.cache_policy.get().copied().flatten()}
    pub(crate) fn page_cache_kib(&self)->Option<u32>{self.generation.cache_kib}
    pub(crate) fn page_cache_registry(&self)->Arc<crate::sqlite_page_cache::Registry>{self._owner.cache_registry.clone()}
    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.generation.pool
    }
    pub(crate) fn active_options(&self) -> Arc<SqliteConnectOptions> {
        self.pool().connect_options()
    }
}

pub(crate) struct ExclusiveSqlite {
    generation: OwnedRwLockWriteGuard<Generation>,
    _owner: Arc<Inner>,
}
impl ExclusiveSqlite {
    pub(crate) fn page_cache_kib(&self)->Option<u32>{self.generation.cache_kib}
    pub(crate) fn replace_with_page_cache(&mut self,pool:SqlitePool,kib:Option<u32>){self.generation.pool=pool;self.generation.cache_kib=kib;}
    #[cfg(test)]
    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.generation.pool
    }
    pub(crate) fn pool_options(&self) -> sqlx::sqlite::SqlitePoolOptions {
        self.generation.pool.options().clone()
    }
    pub(crate) fn connection_options(&self) -> Arc<SqliteConnectOptions> {
        self.generation.pool.connect_options()
    }
    /// The gate is already exclusive, so there are no remaining guarded borrowers.
    pub(crate) async fn close_old(&self) {
        self.generation.pool.close().await;
    }
    /// Publish only a fully opened/validated pool. The filesystem publication is host-owned.
    pub(crate) fn replace(&mut self, pool: SqlitePool) {
        self.generation.pool = pool;
    }
}

pub(crate) struct GuardedConnection {
    // Field drop order is significant: return/close the connection before admitting cutover.
    connection: PoolConnection<Sqlite>,
    #[allow(dead_code)]
    pin: PinnedPool,
}
impl Deref for GuardedConnection {
    type Target = SqliteConnection;
    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}
impl DerefMut for GuardedConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.connection
    }
}

pub(crate) struct LiveTransaction {
    // Dropping a transaction queues rollback before the pool lease is released.
    transaction: Option<Transaction<'static, Sqlite>>,
    #[allow(dead_code)]
    pin: PinnedPool,
}
impl Deref for LiveTransaction {
    type Target = SqliteConnection;
    fn deref(&self) -> &Self::Target {
        self.transaction.as_ref().expect("live transaction")
    }
}
impl DerefMut for LiveTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.transaction.as_mut().expect("live transaction")
    }
}
impl LiveTransaction {
    pub(crate) fn generation_path(&self) -> std::path::PathBuf {
        self.pin.active_options().get_filename().to_path_buf()
    }
    pub(crate) async fn commit(mut self) -> Result<(), Error> {
        self.transaction
            .take()
            .expect("live transaction")
            .commit()
            .await
    }
    pub(crate) async fn rollback(mut self) -> Result<(), Error> {
        self.transaction
            .take()
            .expect("live transaction")
            .rollback()
            .await
    }
}

impl<'c> Executor<'c> for &'c LiveSqlite {
    type Database = Sqlite;
    fn fetch_many<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxStream<'e, Result<Either<SqliteQueryResult, SqliteRow>, Error>>
    where
        'c: 'e,
        E: 'q + Execute<'q, Sqlite>,
    {
        Box::pin(async_stream::try_stream! {
            let mut connection=self.acquire().await?;
            let mut rows=(&mut *connection).fetch_many(query);
            while let Some(row)=rows.try_next().await?{yield row;}
        })
    }
    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<SqliteRow>, Error>>
    where
        'c: 'e,
        E: 'q + Execute<'q, Sqlite>,
    {
        Box::pin(async move {
            let mut connection = self.acquire().await?;
            (&mut *connection).fetch_optional(query).await
        })
    }
    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [SqliteTypeInfo],
    ) -> BoxFuture<'e, Result<SqliteStatement<'q>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move {
            let mut connection = self.acquire().await?;
            (&mut *connection).prepare_with(sql, parameters).await
        })
    }
    fn describe<'e, 'q: 'e>(self, sql: &'q str) -> BoxFuture<'e, Result<Describe<Sqlite>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move {
            let mut connection = self.acquire().await?;
            (&mut *connection).describe(sql).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::TryStreamExt;
    use std::time::Duration;
    async fn pool(value: i64) -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE sample(n INTEGER NOT NULL)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO sample(n) VALUES(?1)")
            .bind(value)
            .execute(&pool)
            .await
            .unwrap();
        pool
    }
    #[tokio::test]
    async fn cutover_drains_connections_and_queued_writes_use_new_generation() {
        let live = LiveSqlite::unmanaged(pool(10).await);
        let clone = live.clone();
        let replacement = pool(20).await;
        let mut held = live.acquire().await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT n FROM sample")
                .fetch_one(&mut *held)
                .await
                .unwrap(),
            10
        );
        let swap = tokio::spawn(async move {
            let mut exclusive = clone.exclusive().await;
            exclusive.close_old().await;
            exclusive.replace(replacement);
        });
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(!swap.is_finished());
        let writer = live.clone();
        let queued = tokio::spawn(async move {
            sqlx::query("UPDATE sample SET n=n+1")
                .execute(&writer)
                .await
                .unwrap();
        });
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(!queued.is_finished());
        drop(held);
        tokio::time::timeout(Duration::from_secs(2), swap)
            .await
            .unwrap()
            .unwrap();
        queued.await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT n FROM sample")
                .fetch_one(&live)
                .await
                .unwrap(),
            21
        );
        live.close().await;
    }
    #[tokio::test]
    async fn transaction_holds_generation_until_commit() {
        let live = LiveSqlite::unmanaged(pool(1).await);
        let mut transaction = live.begin().await.unwrap();
        sqlx::query("UPDATE sample SET n=2")
            .execute(&mut *transaction)
            .await
            .unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(25), live.exclusive())
                .await
                .is_err()
        );
        transaction.commit().await.unwrap();
        let exclusive = tokio::time::timeout(Duration::from_secs(1), live.exclusive())
            .await
            .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT n FROM sample")
                .fetch_one(exclusive.pool())
                .await
                .unwrap(),
            2
        );
        exclusive.close_old().await;
    }
    #[tokio::test]
    async fn streaming_executor_keeps_generation_pinned_until_stream_drop() {
        let live = LiveSqlite::unmanaged(pool(1).await);
        let mut rows =
            sqlx::query("SELECT n FROM sample UNION ALL SELECT n+1 FROM sample").fetch(&live);
        assert!(rows.try_next().await.unwrap().is_some());
        assert!(
            tokio::time::timeout(Duration::from_millis(25), live.exclusive())
                .await
                .is_err()
        );
        drop(rows);
        let exclusive = tokio::time::timeout(Duration::from_secs(1), live.exclusive())
            .await
            .unwrap();
        exclusive.close_old().await;
    }
    #[tokio::test]
    async fn dropped_transaction_releases_generation_and_rolls_back() {
        let live = LiveSqlite::unmanaged(pool(1).await);
        let mut transaction = live.begin().await.unwrap();
        sqlx::query("UPDATE sample SET n=9")
            .execute(&mut *transaction)
            .await
            .unwrap();
        drop(transaction);
        let exclusive = tokio::time::timeout(Duration::from_secs(1), live.exclusive())
            .await
            .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT n FROM sample")
                .fetch_one(exclusive.pool())
                .await
                .unwrap(),
            1
        );
        exclusive.close_old().await;
    }
    #[tokio::test]
    async fn pinned_generation_keeps_process_fence_after_last_live_handle_drops() {
        let directory =
            std::env::temp_dir().join(format!("mw-live-fence-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("store.db");
        let opening = LiveSqlite::opening(&path).await.unwrap();
        let options = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true);
        let raw = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE sample(n INTEGER)")
            .execute(&raw)
            .await
            .unwrap();
        let live = opening.register(raw, false);
        drop(opening);
        let pin = live.pin().await;
        drop(live);
        assert!(
            crate::generation_files::GenerationFiles::lock(&path).is_err(),
            "a detached pinned task still owns the process fence"
        );
        drop(pin);
        let mut released = false;
        for _ in 0..100 {
            if let Ok(fence) = crate::generation_files::GenerationFiles::lock(&path) {
                drop(fence);
                released = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(
            released,
            "last-owner cleanup drains the pool before releasing its fence"
        );
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[tokio::test]
    async fn full_vacuum_entrypoints_refuse_held_rebuild_gate_then_resume() {
        let store = crate::Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        let crate::backend::Backend::Sqlite(live) = &store.backend else {
            unreachable!()
        };
        let clone = live.clone();
        let guard = clone.try_rebuild_guard().unwrap();
        assert!(live.try_rebuild_guard().is_err());
        assert!(store.vacuum_database().await.is_err());
        assert!(store.reclaim_note_metadata_residue().await.is_err());
        drop(guard);
        store.vacuum_database().await.unwrap();
        store.reclaim_note_metadata_residue().await.unwrap();
        store.close().await;
    }

    #[tokio::test]
    async fn closed_manager_cannot_reopen_while_conversion_owns_installation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mail.db");
        let key = crate::ServerKey::generate();
        let cfg = crate::encryption::EncryptionConfig::off();
        let store = crate::Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &cfg)
            .await
            .unwrap();
        let crate::backend::Backend::Sqlite(live) = &store.backend else {
            unreachable!()
        };
        let lease = live.files().unwrap().acquire_conversion().unwrap();
        store.close().await;
        assert!(
            crate::Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &cfg)
                .await
                .is_err()
        );
        drop(lease);
        let reopened = crate::Store::open_with_encryption(path.to_str().unwrap(), key, &cfg)
            .await
            .unwrap();
        reopened.close().await;
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn reopening_waits_for_last_owner_async_worker_drain() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mail.db");
        let opening = LiveSqlite::opening(&path).await.unwrap();
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(&path)
                    .create_if_missing(true),
            )
            .await
            .unwrap();
        let live = opening.register(pool.clone(), false);
        drop(opening);
        // Deliberately bypass pin in this fixture to exercise SQLx's outstanding
        // worker drain after the last production manager owner has disappeared.
        let raw = pool.acquire().await.unwrap();
        drop(live);
        let reopen_path = path.clone();
        let reopen = tokio::spawn(async move { LiveSqlite::opening(&reopen_path).await });
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert!(
            !reopen.is_finished(),
            "opening must await old raw worker closure"
        );
        drop(raw);
        let opening = tokio::time::timeout(Duration::from_secs(3), reopen)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(pool.is_closed());
        assert!(opening.existing.is_none());
        drop(opening);
        drop(pool);
    }
}
