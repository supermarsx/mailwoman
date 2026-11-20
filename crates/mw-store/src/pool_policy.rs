//! Startup connection budgets. SQLx grows/shrinks physical occupancy with demand
//! inside immutable ceilings; this is not a live ceiling-resize controller.
use crate::{
    Store, StoreError,
    backend::{Backend, q},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{
    Connection, Database, Pool, pool::PoolOptions, postgres::PgPoolOptions,
    sqlite::SqlitePoolOptions,
};
use std::time::Duration;

pub const POLICY_KEY: &str = "database.pool";
const SQLITE_IDLE_HEALTH_WINDOW: Duration = Duration::from_secs(1);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct PoolPolicy {
    pub minimum_connections: u32,
    pub sqlite_max_connections: Option<u32>,
    pub postgres_max_connections: Option<u32>,
    pub acquire_timeout_ms: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    pub postgres_connection_budget: u32,
    pub postgres_reserved_connections: u32,
    pub replica_count: u32,
}
impl Default for PoolPolicy {
    fn default() -> Self {
        Self {
            minimum_connections: 1,
            sqlite_max_connections: None,
            postgres_max_connections: None,
            acquire_timeout_ms: 30_000,
            idle_timeout_seconds: 60,
            max_lifetime_seconds: 1800,
            postgres_connection_budget: 100,
            postgres_reserved_connections: 20,
            replica_count: 1,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresCapacity {
    pub maximum: u32,
    pub reserved: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolPlan {
    pub minimum: u32,
    pub maximum: u32,
    pub available_cpus: u32,
    pub cpu_derived: bool,
    pub acquire_timeout_ms: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    pub per_replica_budget: Option<u32>,
    pub budget_constrained: bool,
}
impl PoolPolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.minimum_connections > 1024
            || self
                .sqlite_max_connections
                .is_some_and(|n| !(1..=32).contains(&n))
            || self
                .postgres_max_connections
                .is_some_and(|n| !(1..=1024).contains(&n))
            || !(100..=120_000).contains(&self.acquire_timeout_ms)
            || !(1..=3600).contains(&self.idle_timeout_seconds)
            || !(30..=86400).contains(&self.max_lifetime_seconds)
            || !(1..=1_000_000).contains(&self.postgres_connection_budget)
            || self.postgres_reserved_connections >= self.postgres_connection_budget
            || !(1..=10000).contains(&self.replica_count)
        {
            return Err("database.pool exceeds supported connection/time/budget bounds".into());
        }
        if self
            .sqlite_max_connections
            .is_some_and(|maximum| self.minimum_connections > maximum)
            || self
                .postgres_max_connections
                .is_some_and(|maximum| self.minimum_connections > maximum)
        {
            return Err("minimumConnections exceeds an explicit backend maximum".into());
        }
        let per_replica = (self.postgres_connection_budget - self.postgres_reserved_connections)
            / self.replica_count;
        if per_replica == 0 || self.minimum_connections > per_replica {
            return Err(
                "PostgreSQL configured budget cannot satisfy replica minimumConnections".into(),
            );
        }
        Ok(())
    }
    /// MW_DB_POOL is an atomic object pin, rather than a partial silent overlay.
    pub fn from_boot_values(
        stored: Option<&str>,
        environment: Option<&str>,
    ) -> Result<Self, String> {
        let policy = match environment.or(stored) {
            Some(text) => serde_json::from_str::<Self>(text)
                .map_err(|e| format!("invalid database.pool object: {e}"))?,
            None => Self::default(),
        };
        policy.validate()?;
        Ok(policy)
    }
    pub fn sqlite_plan(&self, cpus: u32) -> Result<PoolPlan, String> {
        self.validate()?;
        let cpus = cpus.max(1);
        let maximum = self
            .sqlite_max_connections
            .unwrap_or(cpus.saturating_mul(2).clamp(2, 8));
        self.plan(
            cpus,
            maximum,
            self.sqlite_max_connections.is_none(),
            None,
            false,
        )
    }
    pub fn postgres_plan(&self, cpus: u32, capacity: PostgresCapacity) -> Result<PoolPlan, String> {
        self.validate()?;
        let cpus = cpus.max(1);
        let budget = self.postgres_connection_budget.min(capacity.maximum);
        let reserve = self.postgres_reserved_connections.max(capacity.reserved);
        let available = budget
            .checked_sub(reserve)
            .ok_or("PostgreSQL reserve exhausts connection budget")?;
        let per_replica = available / self.replica_count;
        if per_replica == 0 {
            return Err(
                "PostgreSQL connection budget cannot allocate one connection per replica".into(),
            );
        }
        let requested = self
            .postgres_max_connections
            .unwrap_or(cpus.saturating_mul(4).clamp(4, 64));
        self.plan(
            cpus,
            requested.min(per_replica),
            self.postgres_max_connections.is_none(),
            Some(per_replica),
            requested > per_replica,
        )
    }
    fn plan(
        &self,
        cpus: u32,
        maximum: u32,
        cpu_derived: bool,
        per_replica_budget: Option<u32>,
        budget_constrained: bool,
    ) -> Result<PoolPlan, String> {
        if self.minimum_connections > maximum {
            return Err("minimumConnections exceeds effective backend/replica ceiling".into());
        }
        Ok(PoolPlan {
            minimum: self.minimum_connections,
            maximum,
            available_cpus: cpus,
            cpu_derived,
            acquire_timeout_ms: self.acquire_timeout_ms,
            idle_timeout_seconds: self.idle_timeout_seconds,
            max_lifetime_seconds: self.max_lifetime_seconds,
            per_replica_budget,
            budget_constrained,
        })
    }
}
impl PoolPlan {
    pub fn sqlite_options(&self) -> SqlitePoolOptions {
        self.options::<sqlx::Sqlite>()
            .test_before_acquire(false)
            .before_acquire(|connection, metadata| {
                Box::pin(async move {
                    // SQLx already pings before returning a connection to its idle
                    // queue, including flushing pending rollbacks. Avoid repeating
                    // that worker round trip for a recently returned SQLite handle.
                    // Actual queries still execute and propagate every error.
                    if metadata.idle_for >= SQLITE_IDLE_HEALTH_WINDOW {
                        connection.ping().await?;
                    }
                    Ok(true)
                })
            })
    }
    pub fn postgres_options(&self) -> PgPoolOptions {
        self.options()
    }
    fn options<DB: Database>(&self) -> PoolOptions<DB> {
        PoolOptions::new()
            .min_connections(self.minimum)
            .max_connections(self.maximum)
            .acquire_timeout(Duration::from_millis(self.acquire_timeout_ms))
            .idle_timeout(Duration::from_secs(self.idle_timeout_seconds))
            .max_lifetime(Duration::from_secs(self.max_lifetime_seconds))
    }
}
fn cpus() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get().min(u32::MAX as usize) as u32)
        .unwrap_or(1)
}
async fn connect_checked<DB: Database>(
    options: PoolOptions<DB>,
    connect: <DB::Connection as sqlx::Connection>::Options,
) -> Result<Pool<DB>, sqlx::Error> {
    // Retain an owned pool even on first-acquire failure so worker shutdown can
    // be confirmed explicitly; never treat a dropped connect future as closed.
    let pool = options.connect_lazy_with(connect);
    match pool.acquire().await {
        Ok(connection) => {
            drop(connection);
            Ok(pool)
        }
        Err(error) => {
            pool.close().await;
            Err(error)
        }
    }
}
impl Store {
    /// Called once before a newly opened Store escapes startup. Policy changes
    /// on already-running Stores are pending a complete process restart.
    pub(crate) async fn configure_pool_at_startup(&mut self) -> Result<(), StoreError> {
        let stored = self
            .get_setting_policy(POLICY_KEY)
            .await?
            .and_then(|row| row.global_value);
        let environment = std::env::var("MW_DB_POOL").ok();
        let policy = PoolPolicy::from_boot_values(stored.as_deref(), environment.as_deref())
            .map_err(StoreError::Corrupt)?;
        match self.backend.clone() {
            Backend::Sqlite(live) => {
                let plan = policy.sqlite_plan(cpus()).map_err(StoreError::Corrupt)?;
                tokio::spawn(async move {
                    let mut exclusive = live.exclusive().await;
                    let previous = exclusive.pool_options();
                    if matches_plan(&previous, &plan) && !previous.get_test_before_acquire() {
                        return Ok::<_, sqlx::Error>(());
                    }
                    let connection = (*exclusive.connection_options()).clone();
                    exclusive.close_old().await;
                    // Install the lazy pool before its first await. If runtime
                    // shutdown cancels startup, the manager's generation drain
                    // still owns every new worker and its filesystem lease.
                    let pool = plan.sqlite_options().connect_lazy_with(connection.clone());
                    exclusive.replace(pool.clone());
                    let opened = pool.acquire().await.map(|connection| drop(connection));
                    if let Err(error) = opened {
                        pool.close().await;
                        let restored = previous.connect_lazy_with(connection);
                        exclusive.replace(restored.clone());
                        if restored.acquire().await.is_err() {
                            restored.close().await;
                        }
                        return Err(error);
                    }
                    Ok(())
                })
                .await
                .map_err(|e| StoreError::Corrupt(format!("pool startup task: {e}")))??;
            }
            Backend::Postgres(previous) => {
                let capacity=q("SELECT current_setting('max_connections') AS maximum, current_setting('superuser_reserved_connections') AS super_reserved, current_setting('reserved_connections',true) AS reserved")
                    .fetch_one(&self.backend).await?;
                let parse = |name: &str| -> Result<u32, StoreError> {
                    capacity
                        .get_opt_string(name)
                        .unwrap_or_else(|| "0".into())
                        .parse::<u32>()
                        .map_err(|_| {
                            StoreError::Corrupt("invalid PostgreSQL connection limits".into())
                        })
                };
                let capacity = PostgresCapacity {
                    maximum: parse("maximum")?,
                    reserved: parse("super_reserved")?.saturating_add(parse("reserved")?),
                };
                let plan = policy
                    .postgres_plan(cpus(), capacity)
                    .map_err(StoreError::Corrupt)?;
                if !matches_plan(previous.options(), &plan) {
                    let connection = (*previous.connect_options()).clone();
                    previous.close().await;
                    self.backend = Backend::Postgres(
                        connect_checked(plan.postgres_options(), connection).await?,
                    );
                }
            }
        }
        Ok(())
    }
    /// Fast occupancy/options and process-wide aggregate counters; no SQL, DSN,
    /// database name, identities or bind values are included.
    pub async fn database_pool_snapshot(&self) -> Value {
        match &self.backend {
            Backend::Sqlite(live) => {
                let pin = live.pin().await;
                snapshot(pin.pool())
            }
            Backend::Postgres(pool) => snapshot(pool),
        }
    }
}
fn matches_plan<DB: Database>(options: &PoolOptions<DB>, plan: &PoolPlan) -> bool {
    options.get_min_connections() == plan.minimum
        && options.get_max_connections() == plan.maximum
        && options.get_acquire_timeout() == Duration::from_millis(plan.acquire_timeout_ms)
        && options.get_idle_timeout() == Some(Duration::from_secs(plan.idle_timeout_seconds))
        && options.get_max_lifetime() == Some(Duration::from_secs(plan.max_lifetime_seconds))
}
fn snapshot<DB: Database>(pool: &Pool<DB>) -> Value {
    let options = pool.options();
    json!({"open":pool.size(),"idle":pool.num_idle(),"busy":pool.size().saturating_sub(pool.num_idle() as u32),
        "minimum":options.get_min_connections(),"maximum":options.get_max_connections(),
        "acquireTimeoutMs":options.get_acquire_timeout().as_millis() as u64,
        "idleTimeoutMs":options.get_idle_timeout().map(|n|n.as_millis() as u64),
        "maxLifetimeMs":options.get_max_lifetime().map(|n|n.as_millis() as u64),"closed":pool.is_closed(),
        "scaling":"SQLx demand growth and idle retirement within startup ceiling",
        "ceilingChangesRequireRestart":true,"dynamicCeilingResize":false,
        "telemetryScope":"process; q() statements plus central SQLite/PostgreSQL acquisitions; direct SQLx statements excluded",
        "operations":crate::pool_telemetry::process_snapshot()})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn sqlite_idle_health_replaces_closed_workers_and_keeps_live_read_errors() {
        let dir = tempfile::tempdir().unwrap();
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(dir.path().join("health.db"))
            .create_if_missing(true);
        let policy = PoolPolicy {
            minimum_connections: 0,
            sqlite_max_connections: Some(1),
            ..Default::default()
        };
        let plan = policy.sqlite_plan(1).unwrap();
        assert!(!plan.sqlite_options().get_test_before_acquire());
        assert!(
            policy
                .postgres_plan(
                    1,
                    PostgresCapacity {
                        maximum: 100,
                        reserved: 3
                    }
                )
                .unwrap()
                .postgres_options()
                .get_test_before_acquire(),
            "PostgreSQL health and transport behavior must remain unchanged"
        );
        let pool = connect_checked(plan.sqlite_options(), options.clone())
            .await
            .unwrap();
        let mut connection = pool.acquire().await.unwrap();
        sqlx::query("CREATE TABLE grants(id INTEGER PRIMARY KEY,allowed INTEGER NOT NULL);INSERT INTO grants VALUES(1,1);CREATE TEMP TABLE worker_identity(id INTEGER)").execute(&mut *connection).await.unwrap();
        connection.close_on_drop();
        drop(connection);
        // An actual worker shutdown must produce a fresh connection, while
        // persistent state remains available and no query is replayed.
        let mut replacement = pool.acquire().await.unwrap();
        assert!(
            sqlx::query("SELECT id FROM temp.worker_identity")
                .fetch_optional(&mut *replacement)
                .await
                .is_err()
        );
        assert!(
            sqlx::query("INSERT INTO grants VALUES(1,1)")
                .execute(&mut *replacement)
                .await
                .is_err()
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM grants")
                .fetch_one(&mut *replacement)
                .await
                .unwrap(),
            1
        );
        replacement.return_to_pool().await;
        drop(replacement);
        let mut writer = sqlx::SqliteConnection::connect_with(&options)
            .await
            .unwrap();
        sqlx::query("UPDATE grants SET allowed=0 WHERE id=1")
            .execute(&mut writer)
            .await
            .unwrap();
        assert!(
            sqlx::query("SELECT id FROM grants WHERE id=1 AND allowed=1")
                .fetch_optional(&pool)
                .await
                .unwrap()
                .is_none(),
            "hot acquisition must not reuse an earlier authorization result"
        );
        // Exercise the cold-idle branch as well as the frequent hot path.
        tokio::time::sleep(SQLITE_IDLE_HEALTH_WINDOW + Duration::from_millis(100)).await;
        assert!(
            sqlx::query("SELECT id FROM grants WHERE id=1 AND allowed=1")
                .fetch_optional(&pool)
                .await
                .unwrap()
                .is_none()
        );
        writer.close().await.unwrap();
        pool.close().await;
    }
    #[tokio::test]
    async fn sqlite_hot_reborrow_still_waits_for_cancelled_transaction_rollback() {
        let dir = tempfile::tempdir().unwrap();
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(dir.path().join("rollback.db"))
            .create_if_missing(true);
        let plan = PoolPolicy {
            minimum_connections: 0,
            sqlite_max_connections: Some(1),
            ..Default::default()
        }
        .sqlite_plan(1)
        .unwrap();
        let pool = connect_checked(plan.sqlite_options(), options)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE pending(id INTEGER PRIMARY KEY)")
            .execute(&pool)
            .await
            .unwrap();
        for id in 1..=3 {
            let mut transaction = pool.begin().await.unwrap();
            sqlx::query("INSERT INTO pending VALUES(?1)")
                .bind(id)
                .execute(&mut *transaction)
                .await
                .unwrap();
            drop(transaction); // queues rollback; SQLx release ping must flush it
            assert_eq!(
                sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM pending")
                    .fetch_one(&pool)
                    .await
                    .unwrap(),
                0
            );
        }
        pool.close().await;
    }
    #[test]
    fn backend_cpu_and_replica_budgets_are_bounded_and_never_zero() {
        let policy = PoolPolicy::default();
        assert_eq!(policy.sqlite_plan(4).unwrap().maximum, 8);
        assert_eq!(policy.sqlite_plan(80).unwrap().maximum, 8);
        let capacity = PostgresCapacity {
            maximum: 100,
            reserved: 3,
        };
        assert_eq!(policy.postgres_plan(4, capacity).unwrap().maximum, 16);
        assert_eq!(policy.postgres_plan(8, capacity).unwrap().maximum, 32);
        let policy = PoolPolicy {
            postgres_max_connections: Some(100),
            replica_count: 4,
            ..policy
        };
        let plan = policy.postgres_plan(80, capacity).unwrap();
        assert_eq!(plan.maximum, 20);
        assert!(plan.budget_constrained);
        assert!(
            PoolPolicy {
                replica_count: 100,
                ..policy.clone()
            }
            .postgres_plan(4, capacity)
            .is_err()
        );
        assert!(
            PoolPolicy {
                minimum_connections: 21,
                ..policy
            }
            .postgres_plan(4, capacity)
            .is_err()
        );
    }
    #[test]
    fn invalid_or_unknown_policy_is_rejected_and_environment_pin_is_atomic() {
        assert!(PoolPolicy::from_boot_values(Some("{\"unknown\":1}"), None).is_err());
        let pinned =
            PoolPolicy::from_boot_values(Some("bad-json"), Some("{\"sqliteMaxConnections\":2}"))
                .unwrap();
        assert_eq!(pinned.sqlite_max_connections, Some(2));
        assert!(
            PoolPolicy {
                acquire_timeout_ms: 0,
                ..PoolPolicy::default()
            }
            .validate()
            .is_err()
        );
        for policy in [
            PoolPolicy {
                minimum_connections: 3,
                sqlite_max_connections: Some(2),
                ..PoolPolicy::default()
            },
            PoolPolicy {
                minimum_connections: 3,
                postgres_max_connections: Some(2),
                ..PoolPolicy::default()
            },
            PoolPolicy {
                replica_count: 100,
                ..PoolPolicy::default()
            },
            PoolPolicy {
                minimum_connections: 21,
                replica_count: 4,
                ..PoolPolicy::default()
            },
        ] {
            assert!(policy.validate().is_err());
        }
    }
    #[tokio::test]
    async fn sqlx_grows_on_demand_and_retires_idle_connections_inside_ceiling() {
        let plan = PoolPolicy {
            minimum_connections: 0,
            sqlite_max_connections: Some(3),
            idle_timeout_seconds: 1,
            ..PoolPolicy::default()
        }
        .sqlite_plan(4)
        .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(dir.path().join("pool.db"))
            .create_if_missing(true);
        let pool = connect_checked(plan.sqlite_options(), options)
            .await
            .unwrap();
        let first = pool.acquire().await.unwrap();
        let second = pool.acquire().await.unwrap();
        let third = pool.acquire().await.unwrap();
        assert_eq!(pool.size(), 3);
        assert!(pool.try_acquire().is_none());
        drop(first);
        drop(second);
        drop(third);
        tokio::time::timeout(Duration::from_secs(6), async {
            while pool.size() > 0 {
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        })
        .await
        .expect("idle pool shrinks without rebuilding or changing its ceiling");
        assert_eq!(pool.options().get_max_connections(), 3);
        pool.close().await;
    }
    #[tokio::test]
    async fn persisted_policy_applies_on_new_open_not_to_live_pool() {
        if std::env::var_os("MW_DB_POOL").is_some() {
            mw_test_gate::skip("pool boot-policy fixture requires MW_DB_POOL unset");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("boot.db");
        let key = crate::ServerKey::generate();
        let config = crate::encryption::EncryptionConfig::off();
        let store = Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &config)
            .await
            .unwrap();
        let initial = store.database_pool_snapshot().await;
        let maximum = if initial["maximum"] == 1 { 2 } else { 1 };
        let policy = PoolPolicy {
            minimum_connections: 0,
            sqlite_max_connections: Some(maximum),
            acquire_timeout_ms: 250,
            idle_timeout_seconds: 2,
            max_lifetime_seconds: 90,
            ..PoolPolicy::default()
        };
        store
            .put_setting_policy(&crate::settings_policy::SettingPolicyRow {
                key: POLICY_KEY.into(),
                global_value: Some(serde_json::to_string(&policy).unwrap()),
                allow_user_override: false,
                global_locked: true,
                updated_by: "fixture".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
        assert_eq!(
            store.database_pool_snapshot().await["maximum"],
            initial["maximum"],
            "saving a startup policy must not falsely claim live SQLx resizing"
        );
        store.close().await;
        let reopened = Store::open_with_encryption(path.to_str().unwrap(), key, &config)
            .await
            .unwrap();
        let actual = reopened.database_pool_snapshot().await;
        assert_eq!(actual["maximum"], maximum);
        assert_eq!(actual["minimum"], 0);
        assert_eq!(actual["acquireTimeoutMs"], 250);
        assert_eq!(actual["idleTimeoutMs"], 2000);
        assert_eq!(actual["maxLifetimeMs"], 90000);
        assert_eq!(actual["dynamicCeilingResize"], false);
        reopened.close().await;
    }
    #[test]
    fn server_reserved_capacity_and_replica_rounding_are_enforced() {
        let policy = PoolPolicy {
            postgres_reserved_connections: 2,
            replica_count: 3,
            ..PoolPolicy::default()
        };
        let plan = policy
            .postgres_plan(
                4,
                PostgresCapacity {
                    maximum: 20,
                    reserved: 5,
                },
            )
            .unwrap();
        assert_eq!(plan.per_replica_budget, Some(5));
        assert_eq!(plan.maximum, 5);
        assert!(
            policy
                .postgres_plan(
                    4,
                    PostgresCapacity {
                        maximum: 5,
                        reserved: 5
                    }
                )
                .is_err()
        );
    }
    #[tokio::test]
    async fn encrypted_single_connection_restart_and_exclusive_replacement_do_not_self_deadlock() {
        if std::env::var_os("MW_DB_POOL").is_some() {
            mw_test_gate::skip("encrypted pool startup fixture requires MW_DB_POOL unset");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("encrypted-one.db");
        let key = crate::ServerKey::generate();
        let config = crate::encryption::EncryptionConfig::with_key(
            crate::encryption::EncryptionMode::Optional,
            crate::encryption::DbKey::generate(),
        );
        let store = Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &config)
            .await
            .unwrap();
        let policy = PoolPolicy {
            minimum_connections: 1,
            sqlite_max_connections: Some(1),
            acquire_timeout_ms: 5000,
            ..Default::default()
        };
        store
            .put_setting_policy(&crate::settings_policy::SettingPolicyRow {
                key: POLICY_KEY.into(),
                global_value: Some(serde_json::to_string(&policy).unwrap()),
                allow_user_override: false,
                global_locked: true,
                updated_by: "fixture".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
        store.close().await;
        drop(store);
        let store = tokio::time::timeout(
            Duration::from_secs(15),
            Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &config),
        )
        .await
        .expect("encrypted one-connection startup must release bootstrap and generation leases")
        .unwrap();
        assert_eq!(store.database_pool_snapshot().await["maximum"], 1);
        assert_eq!(store.database_pool_snapshot().await["minimum"], 1);
        let Backend::Sqlite(live) = &store.backend else {
            unreachable!()
        };
        let mut exclusive = tokio::time::timeout(Duration::from_secs(1), live.exclusive())
            .await
            .expect("startup must release its exclusive lease");
        let options = (*exclusive.connection_options()).clone();
        let sizing = exclusive.pool_options();
        exclusive.close_old().await;
        let replacement = sizing.connect_lazy_with(options);
        exclusive.replace(replacement.clone());
        tokio::time::timeout(Duration::from_secs(5), replacement.acquire())
            .await
            .expect("raw pool acquisition under exclusive gate must not repin")
            .unwrap();
        drop(exclusive);
        assert_eq!(
            q("SELECT COUNT(*) AS n FROM setting_policy WHERE key='database.pool'")
                .fetch_one(&store.backend)
                .await
                .unwrap()
                .get_i64("n"),
            1
        );
        store.close().await;
        let required = crate::encryption::EncryptionConfig {
            mode: crate::encryption::EncryptionMode::Required,
            ..config
        };
        let reopened = Store::open_with_encryption(path.to_str().unwrap(), key, &required)
            .await
            .unwrap();
        reopened.close().await;
    }
}
