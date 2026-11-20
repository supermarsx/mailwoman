//! Opt-in MAIN database foreground-pool cache targets. Never a hard RSS limit.
//! No mail-request hook or SQL is used to collect the owned-handle diagnostics.
use serde::Serialize;
use sqlx::{SqliteConnection, SqlitePool, sqlite::SqliteConnectOptions};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

pub const AUXILIARY_KIB: u32 = 2000;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageCachePlan {
    pub aggregate_mib: Option<u32>,
    pub planned_maximum_connections: u32,
    pub per_connection_kib: Option<u32>,
    pub planned_foreground_bytes: Option<u64>,
    pub auxiliary_connection_kib: Option<u32>,
}
impl PageCachePlan {
    pub fn new(aggregate_mib: Option<u32>, maximum: u32) -> Result<Self, &'static str> {
        if !(1..=32).contains(&maximum) || aggregate_mib.is_some_and(|n| !(16..=1024).contains(&n))
        {
            return Err("SQLite page-cache target exceeds supported bounds");
        }
        let per_connection_kib = aggregate_mib.map(|n| n * 1024 / maximum);
        Ok(Self {
            aggregate_mib,
            planned_maximum_connections: maximum,
            per_connection_kib,
            planned_foreground_bytes: per_connection_kib
                .map(|n| u64::from(n) * 1024 * u64::from(maximum)),
            auxiliary_connection_kib: aggregate_mib.map(|_| AUXILIARY_KIB),
        })
    }
    pub(crate) fn apply(self, options: SqliteConnectOptions) -> SqliteConnectOptions {
        match self.per_connection_kib {
            Some(kib) => options.pragma("cache_size", format!("-{kib}")),
            None => options,
        }
    }
}
pub(crate) fn auxiliary(options: SqliteConnectOptions, custom: bool) -> SqliteConnectOptions {
    if custom {
        options.pragma("cache_size", format!("-{AUXILIARY_KIB}"))
    } else {
        options
    }
}

#[derive(Default)]
pub(crate) struct Registry {
    next: AtomicU64,
    epochs: Mutex<BTreeMap<usize, u64>>,
}
impl Registry {
    pub(crate) async fn register(
        &self,
        connection: &mut SqliteConnection,
    ) -> Result<(), sqlx::Error> {
        let mut handle = connection.lock_handle().await?;
        let address = handle.as_raw_handle().as_ptr() as usize;
        let epoch = self
            .next
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| sqlx::Error::Protocol("SQLite diagnostic epoch exhausted".into()))?
            + 1;
        let mut entries = self.epochs.lock().unwrap_or_else(|e| e.into_inner());
        if entries.len() >= 64 && !entries.contains_key(&address) {
            if let Some(old) = entries
                .iter()
                .min_by_key(|(_, epoch)| **epoch)
                .map(|(key, _)| *key)
            {
                entries.remove(&old);
            }
        }
        entries.insert(address, epoch);
        Ok(())
    }
    pub(crate) fn has_connections(&self) -> bool {
        !self
            .epochs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty()
    }
    fn epoch(&self, address: usize) -> Option<u64> {
        self.epochs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&address)
            .copied()
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
    pub connection_epoch: Option<u64>,
    pub cache_used_bytes: Option<u64>,
    pub cache_hits: Option<u64>,
    pub cache_misses: Option<u64>,
    pub cache_writes: Option<u64>,
    pub cache_spills: Option<u64>,
}
/// No pointer, schema, key or identity leaves this helper. SQLite exposes 32-bit
/// counters; negative or failed values are unknown, never clamped to zero. Positive
/// counter wraps cannot be distinguished; consumers must not infer rates.
pub(crate) async fn status(
    connection: &mut SqliteConnection,
    registry: &Registry,
) -> Result<ConnectionStatus, sqlx::Error> {
    // Internal address is only a bounded registry key, never serialized. Drop
    // this guard before acquiring the helper's lock; no nested handle locks.
    let address = {
        let mut handle = connection.lock_handle().await?;
        handle.as_raw_handle().as_ptr() as usize
    };
    let observed = mw_sqlite_status::cache_status(connection).await?;
    Ok(ConnectionStatus {
        connection_epoch: registry.epoch(address),
        cache_used_bytes: observed.used_bytes,
        cache_hits: observed.hits,
        cache_misses: observed.misses,
        cache_writes: observed.writes,
        cache_spills: observed.spills,
    })
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdleSample {
    pub open_before: u32,
    pub open_after: u32,
    pub sampled_connections: usize,
    pub complete_idle_coverage: bool,
    pub deadline_reached: bool,
    pub connections: Vec<ConnectionStatus>,
}
pub(crate) async fn sample_idle(pool: &SqlitePool, registry: Arc<Registry>) -> IdleSample {
    let open_before = pool.size();
    // Cooperative bound: synchronous SQLite status calls / OS scheduling are
    // not preemptible. Never wait for a busy pool lease or create connections.
    let deadline = tokio::time::Instant::now() + Duration::from_millis(25);
    let mut held = Vec::new();
    let mut connections = Vec::new();
    let mut deadline_reached = false;
    // Retain sampled leases until this sweep ends; never count one handle twice.
    // try_acquire never creates a physical connection or waits behind mail work.
    for _ in 0..open_before.min(32) {
        if tokio::time::Instant::now() >= deadline {
            deadline_reached = true;
            break;
        }
        let Some(mut connection) = pool.try_acquire() else {
            break;
        };
        match tokio::time::timeout_at(deadline, status(&mut connection, &registry)).await {
            Ok(Ok(observed)) => connections.push(observed),
            Ok(Err(_)) => {}
            Err(_) => {
                deadline_reached = true;
                held.push(connection);
                break;
            }
        }
        held.push(connection);
    }
    let open_after = pool.size();
    IdleSample {
        sampled_connections: connections.len(),
        complete_idle_coverage: !deadline_reached
            && connections.len() == open_before as usize
            && open_before == open_after,
        open_before,
        open_after,
        deadline_reached,
        connections,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aggregate_target_uses_ceiling_and_driver_default_is_explicit() {
        assert_eq!(
            PageCachePlan::new(None, 8).unwrap().per_connection_kib,
            None
        );
        for maximum in [1, 8, 32] {
            let p = PageCachePlan::new(Some(256), maximum).unwrap();
            assert_eq!(p.per_connection_kib, Some(256 * 1024 / maximum));
            assert!(p.planned_foreground_bytes.unwrap() <= 256 * 1024 * 1024);
            assert_eq!(p.auxiliary_connection_kib, Some(2000));
        }
        assert!(
            PageCachePlan::new(Some(257), 3)
                .unwrap()
                .planned_foreground_bytes
                .unwrap()
                < 257 * 1024 * 1024
        );
        for n in [0, 1, 15, 1025, u32::MAX] {
            assert!(PageCachePlan::new(Some(n), 8).is_err());
        }
        assert!(PageCachePlan::new(None, 0).is_err());
        assert!(PageCachePlan::new(None, 33).is_err());
    }
    #[test]
    fn pool_policy_accepts_legacy_missing_target_and_rejects_invalid_configuration() {
        use crate::pool_policy::PoolPolicy;
        let legacy = PoolPolicy::from_boot_values(Some("{}"), None).unwrap();
        assert_eq!(legacy.sqlite_page_cache_mib, None);
        assert_eq!(
            serde_json::to_value(legacy).unwrap()["sqlitePageCacheMiB"],
            serde_json::Value::Null
        );
        for value in ["0", "15", "1025", "16.5", "\"256\""] {
            let encoded = format!("{{\"sqlitePageCacheMiB\":{value}}}");
            assert!(
                PoolPolicy::from_boot_values(Some(&encoded), None).is_err(),
                "{encoded}"
            );
        }
        assert_eq!(
            PoolPolicy::from_boot_values(None, Some("{\"sqlitePageCacheMiB\":256}"))
                .unwrap()
                .sqlite_page_cache_mib,
            Some(256)
        );
    }
    #[tokio::test]
    async fn null_policy_preserves_existing_driver_options_for_foreground_and_auxiliary() {
        use sqlx::Connection;
        // A deliberately nondefault target proves that null adds no override,
        // including if SQLCipher's compiled/header defaults change in future.
        let original = SqliteConnectOptions::new().pragma("cache_size", "-73");
        for options in [
            PageCachePlan::new(None, 8).unwrap().apply(original.clone()),
            auxiliary(original, false),
        ] {
            let mut c = SqliteConnection::connect_with(&options).await.unwrap();
            assert_eq!(
                crate::database_observability::sqlite_pragma_i64(&mut c, "PRAGMA cache_size")
                    .await
                    .unwrap(),
                -73
            );
            c.close().await.unwrap();
        }
    }
    #[tokio::test]
    async fn owned_real_handle_status_and_retirement_epochs_have_no_diagnostic_sql() {
        use sqlx::Connection;
        let mut c = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        let registry = Registry::default();
        registry.register(&mut c).await.unwrap();
        let before = status(&mut c, &registry).await.unwrap();
        sqlx::query("CREATE TABLE payload(id INTEGER PRIMARY KEY,value TEXT)")
            .execute(&mut c)
            .await
            .unwrap();
        sqlx::query("INSERT INTO payload(value) VALUES('fixture-only')")
            .execute(&mut c)
            .await
            .unwrap();
        for _ in 0..4 {
            let _: String = sqlx::query_scalar("SELECT value FROM payload WHERE id=1")
                .fetch_one(&mut c)
                .await
                .unwrap();
        }
        let (observed, count) =
            crate::pool_telemetry::measure_queries(status(&mut c, &registry)).await;
        let observed = observed.unwrap();
        assert_eq!(count.read.calls + count.write.calls + count.other.calls, 0);
        assert_eq!(before.connection_epoch, observed.connection_epoch);
        assert!(observed.cache_used_bytes.unwrap() > 0);
        assert!(observed.cache_hits.unwrap() > before.cache_hits.unwrap());
        registry.register(&mut c).await.unwrap();
        assert_ne!(
            observed.connection_epoch,
            status(&mut c, &registry).await.unwrap().connection_epoch
        );
        c.close().await.unwrap();
    }
    #[tokio::test]
    async fn idle_sampling_does_not_grow_pool_or_wait_for_busy_mail_handle() {
        let registry = Arc::new(Registry::default());
        let hook = registry.clone();
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .after_connect(move |c, _| {
                let hook = hook.clone();
                Box::pin(async move { hook.register(c).await })
            })
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let mut lease = pool.acquire().await.unwrap();
        let busy = sample_idle(&pool, registry.clone()).await;
        assert_eq!(busy.sampled_connections, 0);
        assert!(!busy.complete_idle_coverage);
        assert_eq!(pool.size(), 1);
        lease.return_to_pool().await;
        let idle = sample_idle(&pool, registry).await;
        assert_eq!(idle.sampled_connections, 1);
        assert!(idle.complete_idle_coverage);
        pool.close().await;
    }
    #[tokio::test]
    async fn small_owned_file_cache_produces_real_misses_and_writes_without_resetting_counters() {
        use sqlx::Connection;
        let dir = tempfile::tempdir().unwrap();
        let options = crate::encryption::connect_options(&dir.path().join("pager.db"), None, true)
            .pragma("cache_size", "-16");
        let mut c = SqliteConnection::connect_with(&options).await.unwrap();
        let registry = Registry::default();
        registry.register(&mut c).await.unwrap();
        let before = status(&mut c, &registry).await.unwrap();
        sqlx::query("CREATE TABLE payload(value BLOB)")
            .execute(&mut c)
            .await
            .unwrap();
        sqlx::query("WITH RECURSIVE seq(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM seq WHERE n<128) INSERT INTO payload(value) SELECT zeroblob(4096) FROM seq").execute(&mut c).await.unwrap();
        for _ in 0..2 {
            let bytes: i64 = sqlx::query_scalar("SELECT sum(length(hex(value))) FROM payload")
                .fetch_one(&mut c)
                .await
                .unwrap();
            assert_eq!(bytes, 128 * 4096 * 2);
        }
        let after = status(&mut c, &registry).await.unwrap();
        assert!(after.cache_misses.unwrap() > before.cache_misses.unwrap());
        assert!(after.cache_writes.unwrap() > before.cache_writes.unwrap());
        assert!(after.cache_spills.is_some());
        c.close().await.unwrap();
    }
    async fn tuned_store() -> (
        tempfile::TempDir,
        crate::Store,
        crate::ServerKey,
        crate::encryption::EncryptionConfig,
    ) {
        assert!(
            std::env::var_os("MW_DB_POOL").is_none(),
            "owned startup test requires MW_DB_POOL unset"
        );
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache.db");
        let key = crate::ServerKey::generate();
        let cfg = crate::encryption::EncryptionConfig::with_key(
            crate::encryption::EncryptionMode::Required,
            crate::encryption::DbKey::generate(),
        );
        let first = crate::Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &cfg)
            .await
            .unwrap();
        let policy = crate::pool_policy::PoolPolicy {
            minimum_connections: 0,
            sqlite_max_connections: Some(2),
            sqlite_page_cache_mib: Some(16),
            ..Default::default()
        };
        first
            .put_setting_policy(&crate::settings_policy::SettingPolicyRow {
                key: "database.pool".into(),
                global_value: Some(serde_json::to_string(&policy).unwrap()),
                allow_user_override: false,
                global_locked: true,
                updated_by: "fixture".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
        first.close().await;
        drop(first);
        let store = crate::Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &cfg)
            .await
            .unwrap();
        (dir, store, key, cfg)
    }
    #[tokio::test]
    async fn keyed_startup_shared_reopen_and_maximum_change_keep_applied_aggregate_and_aux_scope() {
        use sqlx::Connection;
        let (dir, mut store, key, cfg) = tuned_store().await;
        let crate::backend::Backend::Sqlite(live) = store.backend.clone() else {
            unreachable!()
        };
        let pin = live.pin().await;
        let mut held = Vec::new();
        for _ in 0..2 {
            let mut c = pin.pool().acquire().await.unwrap();
            assert_eq!(
                crate::database_observability::sqlite_pragma_i64(&mut c, "PRAGMA cache_size")
                    .await
                    .unwrap(),
                -8192
            );
            held.push(c);
        }
        let mut auxiliary = SqliteConnection::connect_with(&pin.auxiliary_options())
            .await
            .unwrap();
        assert_eq!(
            crate::database_observability::sqlite_pragma_i64(&mut auxiliary, "PRAGMA cache_size")
                .await
                .unwrap(),
            -2000
        );
        auxiliary.close().await.unwrap();
        let wrong = pin.auxiliary_options().pragma(
            "key",
            crate::encryption::key_pragma(&crate::encryption::DbKey::generate()),
        );
        assert!(SqliteConnection::connect_with(&wrong).await.is_err());
        for mut c in held {
            c.return_to_pool().await;
        }
        drop(pin);
        let policy = crate::pool_policy::PoolPolicy {
            minimum_connections: 0,
            sqlite_max_connections: Some(1),
            sqlite_page_cache_mib: Some(32),
            ..Default::default()
        };
        store
            .put_setting_policy(&crate::settings_policy::SettingPolicyRow {
                key: "database.pool".into(),
                global_value: Some(serde_json::to_string(&policy).unwrap()),
                allow_user_override: false,
                global_locked: true,
                updated_by: "fixture".into(),
                updated_at: 2,
            })
            .await
            .unwrap();
        store.configure_pool_at_startup().await.unwrap();
        let report = store.database_pool_snapshot().await;
        assert_eq!(report["maximum"], 1);
        assert_eq!(
            report["sqlitePageCache"]["aggregateMiB"], 16,
            "saved32MiB remains pending a full manager restart"
        );
        assert_eq!(report["sqlitePageCache"]["perConnectionKiB"], 16384);
        assert_eq!(
            report["sqlitePageCache"]["plannedForegroundBytes"],
            16 * 1024 * 1024
        );
        let shared = crate::Store::open_with_encryption(
            dir.path().join("cache.db").to_str().unwrap(),
            key,
            &cfg,
        )
        .await
        .unwrap();
        assert_eq!(
            shared.database_pool_snapshot().await["sqlitePageCache"]["aggregateMiB"],
            16
        );
        // Drop the shared handle without closing the shared manager under store.
        drop(shared);
        store.close().await;
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn online_conversion_reopens_tuned_foreground_not_small_copy_target() {
        let (_dir, store, _key, _cfg) = tuned_store().await;
        // New databases are already INCREMENTAL. Model a genuine legacy NONE
        // database so this exercises the copy, cutover and target reopen paths.
        let crate::backend::Backend::Sqlite(live) = &store.backend else {
            unreachable!()
        };
        {
            let mut c = live.acquire().await.unwrap();
            sqlx::query("PRAGMA auto_vacuum=NONE")
                .execute(&mut *c)
                .await
                .unwrap();
        }
        live.full_vacuum().await.unwrap();
        {
            let mut c = live.acquire().await.unwrap();
            assert_eq!(
                crate::database_observability::sqlite_pragma_i64(&mut c, "PRAGMA auto_vacuum")
                    .await
                    .unwrap(),
                0
            );
        }
        store
            .start_online_conversion(crate::online_maintenance::ConversionOptions {
                pause_ms: 0,
                batch_rows: 128,
                max_cutover_ms: 10000,
                ..Default::default()
            })
            .await
            .unwrap();
        let report = tokio::time::timeout(Duration::from_secs(90), async {
            loop {
                let r = store.online_conversion_status().unwrap();
                if ["completed", "failed", "cancelled", "recovery-required"]
                    .contains(&r.phase.as_str())
                {
                    break r;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(report.phase, "completed", "{:?}", report.error);
        assert!(report.source_generation.is_some());
        assert!(report.target_generation.is_some());
        assert_ne!(report.source_generation, report.target_generation);
        let crate::backend::Backend::Sqlite(live) = &store.backend else {
            unreachable!()
        };
        let pin = live.pin().await;
        let mut c = pin.pool().acquire().await.unwrap();
        assert_eq!(
            crate::database_observability::sqlite_pragma_i64(&mut c, "PRAGMA cache_size")
                .await
                .unwrap(),
            -8192
        );
        drop(c);
        drop(pin);
        assert_eq!(
            store.database_pool_snapshot().await["sqlitePageCache"]["perConnectionKiB"],
            8192
        );
        store.close().await;
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn failed_post_close_conversion_restores_keyed_source_and_foreground_cache() {
        use sqlx::Connection;
        let (_dir, store, _key, _cfg) = tuned_store().await;
        let crate::backend::Backend::Sqlite(live) = &store.backend else {
            unreachable!()
        };
        {
            let mut c = live.acquire().await.unwrap();
            sqlx::query("PRAGMA auto_vacuum=NONE")
                .execute(&mut *c)
                .await
                .unwrap();
        }
        live.full_vacuum().await.unwrap();
        store
            .set_setting("pager-cache-restore-proof", "before-copy")
            .await
            .unwrap();
        let pin = live.pin().await;
        let original_pool = pin.pool().clone();
        let files = live.files().unwrap();
        let original_generation = files.resolve_active().unwrap().id;
        let auxiliary_options = pin.auxiliary_options();
        store
            .start_online_conversion(crate::online_maintenance::ConversionOptions {
                pause_ms: 0,
                batch_rows: 128,
                max_cutover_ms: 10000,
                ..Default::default()
            })
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(90), async {
            loop {
                let report = store.online_conversion_status().unwrap();
                if report.phase == "waiting-for-cutover" {
                    break;
                }
                assert!(
                    !["failed", "completed", "cancelled"].contains(&report.phase.as_str()),
                    "expected actual prepared conversion, got {}",
                    report.phase
                );
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("prepared copy reached owned cutover gate");
        // The generation pin deterministically prevents old-pool closure until
        // this independent source writer is ready. The transaction's Drop
        // queues rollback on panic/cancellation; no new production fault hook.
        let mut external = SqliteConnection::connect_with(&auxiliary_options)
            .await
            .unwrap();
        let blocker = external.begin_with("BEGIN IMMEDIATE").await.unwrap();
        drop(pin);
        // Final source capture cleanup must write AFTER close_old. Holding the
        // SQLite writer lock forces the real error/timeout → restore_source
        // path, rather than the earlier busy-generation admission abort.
        let failure = tokio::time::timeout(Duration::from_secs(40), async {
            loop {
                let report = store.online_conversion_status().unwrap();
                if report.phase == "failed" {
                    break;
                }
                if report.phase == "recovery-required"
                    && report.error.as_deref()
                        == Some("Reopening the selected source; database access remains fenced")
                {
                    break;
                }
                if ["completed", "cancelled"].contains(&report.phase.as_str()) {
                    return Err(report.phase);
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Ok::<_, String>(())
        })
        .await;
        let old_pool_was_closed = original_pool.is_closed();
        // Always release the injected writer before assertions, even if the
        // bounded observation failed. A recovery-required intermediate state
        // is NOT success: the final restored serving pool below is mandatory.
        blocker.rollback().await.unwrap();
        external.close().await.unwrap();
        assert!(
            failure.is_ok_and(|result| result.is_ok()),
            "real post-close failure was not observed"
        );
        assert!(
            old_pool_was_closed,
            "a pre-cutover busy error does not exercise restoration"
        );
        let report = tokio::time::timeout(Duration::from_secs(90), async {
            loop {
                let report = store.online_conversion_status().unwrap();
                if ["failed", "completed", "cancelled"].contains(&report.phase.as_str()) {
                    break report;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("failed conversion automatically restored the original serving pool");
        assert_eq!(report.phase, "failed");
        assert!(
            report
                .error
                .as_deref()
                .is_some_and(|error| !error.contains("Busy operations"))
        );
        assert_eq!(
            report.source_generation.as_deref(),
            Some(original_generation.as_str())
        );
        assert!(report.target_generation.is_some());
        assert!(!report.retained_previous);
        assert_eq!(files.resolve_active().unwrap().id, original_generation);
        let mut c = live.acquire().await.unwrap();
        assert_eq!(
            crate::database_observability::sqlite_pragma_i64(&mut c, "PRAGMA auto_vacuum")
                .await
                .unwrap(),
            0
        );
        assert_eq!(
            crate::database_observability::sqlite_pragma_i64(&mut c, "PRAGMA cache_size")
                .await
                .unwrap(),
            -8192
        );
        drop(c);
        assert_eq!(
            store
                .get_setting("pager-cache-restore-proof")
                .await
                .unwrap()
                .as_deref(),
            Some("before-copy")
        );
        store
            .set_setting("pager-cache-restore-proof", "after-restore")
            .await
            .unwrap();
        assert_eq!(
            store
                .get_setting("pager-cache-restore-proof")
                .await
                .unwrap()
                .as_deref(),
            Some("after-restore")
        );
        let snapshot = store.database_pool_snapshot().await;
        assert_eq!(snapshot["sqlitePageCache"]["aggregateMiB"], 16);
        assert_eq!(snapshot["sqlitePageCache"]["perConnectionKiB"], 8192);
        assert!(!live.pin().await.pool().is_closed());
        store.close().await;
    }
}
