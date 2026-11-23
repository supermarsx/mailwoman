//! Durable, bounded failed-credential counters; no account or credential data.
//!
//! # Source key
//! A source is an IPv4 address (IPv4-mapped IPv6 included) or an IPv6 /64, the
//! same grouping as sign-in pacing (`signin_cooldown`). An IPv6 /64 is stored as
//! its network address (`2001:db8:1:2::`) so the value still parses as an address
//! for the unlock endpoint; [`AuthSourceLock::network`] names the /64.
//! Rows written before this keying hold a full IPv6 address. They are not
//! migrated: lookups also match the exact address, so such a lock is enforced
//! and can be unlocked until it expires on its own (at most `lockSecs`).
//!
//! # Capacity
//! At most [`CAPACITY`] rows across both surfaces. A new source at capacity
//! evicts the least recently failed record that is not an active lock. Only when
//! every record is an active lock is the new failure left untracked; a check for
//! an untracked source never fails, so saturation cannot deny sign-in (pacing
//! still applies to that source, and every existing lock is still enforced).
use crate::{Store, StoreError, backend::q};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv6Addr};
const CAPACITY: i64 = 10_000;

/// The stored key for `ip` and, when it differs, the pre-/64 legacy key.
fn source_key(ip: IpAddr) -> (String, String) {
    let ip = ip.to_canonical();
    let key = match ip {
        IpAddr::V4(v4) => v4.to_string(),
        IpAddr::V6(v6) => {
            let s = v6.segments();
            Ipv6Addr::new(s[0], s[1], s[2], s[3], 0, 0, 0, 0).to_string()
        }
    };
    (key, ip.to_string())
}
/// Display form of a stored key: the IPv4 address, `<net>/64` for a /64 key, or
/// the full address of a legacy row.
fn network(stored: &str) -> String {
    match stored.parse::<IpAddr>() {
        Ok(IpAddr::V6(v6)) if v6.segments()[4..].iter().all(|s| *s == 0) => {
            format!("{v6}/64")
        }
        _ => stored.to_string(),
    }
}
#[derive(Debug, Clone, Copy)]
pub struct FailurePolicy {
    pub threshold: usize,
    pub window_secs: i64,
    pub lock_secs: i64,
}
impl FailurePolicy {
    fn valid(self) -> bool {
        (1..=100).contains(&self.threshold)
            && (30..=86_400).contains(&self.window_secs)
            && (30..=86_400).contains(&self.lock_secs)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthSourceLock {
    pub surface: String,
    /// The stored source key; parses as an address (see the module docs).
    pub ip: String,
    /// The IPv4 address or the IPv6 `/64` the lock covers.
    pub network: String,
    pub locked_until: i64,
    pub failures: usize,
    pub revision: String,
}
fn scope(surface: &str) -> Result<(), StoreError> {
    if matches!(surface, "admin" | "webmail") {
        Ok(())
    } else {
        Err(StoreError::Corrupt("invalid authentication surface".into()))
    }
}
fn revision() -> String {
    format!("{:032x}", rand::random::<u128>())
}
impl Store {
    /// The active lock for this source, if any. An untracked source is never an
    /// error, even when the table is full (see the module docs).
    pub async fn auth_source_lock(
        &self,
        surface: &str,
        ip: IpAddr,
        now: i64,
    ) -> Result<Option<AuthSourceLock>, StoreError> {
        scope(surface)?;
        let (key, legacy) = source_key(ip);
        let rows=q("SELECT source_ip,locked_until,attempts_json,revision FROM auth_source_failure WHERE surface=?1 AND source_ip IN (?2,?3) AND expires_at>?4 AND locked_until>?4 ORDER BY locked_until DESC LIMIT 1").bind(surface).bind(&key).bind(&legacy).bind(now).fetch_all(&self.backend).await?;
        rows.first().map(|row| lock_from(surface, row)).transpose()
    }
    /// Global write lock serializes threshold transitions across replicas. A locked
    /// source retains its original deadline even if concurrent requests finish late.
    pub async fn record_auth_source_failure(
        &self,
        surface: &str,
        ip: IpAddr,
        now: i64,
        policy: FailurePolicy,
    ) -> Result<Option<AuthSourceLock>, StoreError> {
        scope(surface)?;
        if !policy.valid() {
            return Err(StoreError::Corrupt("invalid failure policy".into()));
        }
        let (ip, legacy) = source_key(ip);
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('auth-source-failure:lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='auth-source-failure:lock'")
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM auth_source_failure WHERE expires_at<=?1")
            .bind(now)
            .execute_tx(&mut tx)
            .await?;
        let rows=q("SELECT source_ip,locked_until,attempts_json,revision FROM auth_source_failure WHERE surface=?1 AND source_ip IN (?2,?3)").bind(surface).bind(&ip).bind(&legacy).fetch_all_tx(&mut tx).await?;
        // An active lock (the /64 row or a legacy exact-address row) keeps its deadline.
        if let Some(row) = rows.iter().find(|r| r.get_i64("locked_until") > now) {
            let lock = lock_from(surface, row)?;
            tx.commit().await?;
            return Ok(Some(lock));
        }
        let mut attempts: Vec<i64> = Vec::new();
        if let Some(row) = rows.iter().find(|r| r.get_string("source_ip") == ip) {
            attempts = parse_attempts(row)?;
            if attempts.len() > 100 {
                return Err(StoreError::Corrupt(
                    "oversized authentication failure state".into(),
                ));
            }
            if row.get_i64("locked_until") != 0 {
                attempts.clear();
            }
        } else {
            // A new record. Under the global write lock above, so racing inserts
            // see each other's rows and the cap holds.
            let count = q("SELECT COUNT(*) AS total FROM auth_source_failure")
                .fetch_all_tx(&mut tx)
                .await?[0]
                .get_i64("total");
            if count >= CAPACITY {
                let excess = count - CAPACITY + 1;
                // Least recently failed first; an active lock is never evicted.
                let victims=q("SELECT surface,source_ip FROM auth_source_failure WHERE locked_until<=?1 ORDER BY expires_at,surface,source_ip LIMIT ?2").bind(now).bind(excess).fetch_all_tx(&mut tx).await?;
                if (victims.len() as i64) < excess {
                    return Err(StoreError::Corrupt(
                        "authentication failure state holds only active locks".into(),
                    ));
                }
                for victim in &victims {
                    q("DELETE FROM auth_source_failure WHERE surface=?1 AND source_ip=?2")
                        .bind(victim.get_string("surface"))
                        .bind(victim.get_string("source_ip"))
                        .execute_tx(&mut tx)
                        .await?;
                }
            }
        }
        attempts.retain(|at| *at > now.saturating_sub(policy.window_secs));
        attempts.push(now);
        let until = if attempts.len() >= policy.threshold {
            now.saturating_add(policy.lock_secs)
        } else {
            0
        };
        let expires = now.saturating_add(policy.window_secs).max(until);
        let rev = revision();
        let json = serde_json::to_string(&attempts)
            .map_err(|_| StoreError::Corrupt("invalid authentication failure state".into()))?;
        q("INSERT INTO auth_source_failure(surface,source_ip,attempts_json,locked_until,expires_at,revision) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(surface,source_ip) DO UPDATE SET attempts_json=excluded.attempts_json,locked_until=excluded.locked_until,expires_at=excluded.expires_at,revision=excluded.revision").bind(surface).bind(&ip).bind(&json).bind(until).bind(expires).bind(&rev).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok((until > now).then_some(AuthSourceLock {
            surface: surface.into(),
            network: network(&ip),
            ip,
            locked_until: until,
            failures: attempts.len(),
            revision: rev,
        }))
    }
    pub async fn list_auth_source_locks(
        &self,
        now: i64,
    ) -> Result<Vec<AuthSourceLock>, StoreError> {
        let rows=q("SELECT surface,source_ip,attempts_json,locked_until,revision FROM auth_source_failure WHERE locked_until>?1 ORDER BY surface,source_ip LIMIT 10000").bind(now).fetch_all(&self.backend).await?;
        rows.iter()
            .map(|row| lock_from(&row.get_string("surface"), row))
            .collect()
    }

    /// Completed authentication or an explicit operator unlock resets only its surface.
    pub async fn clear_auth_source_failures(
        &self,
        surface: &str,
        ip: IpAddr,
        expected_revision: Option<&str>,
    ) -> Result<bool, StoreError> {
        scope(surface)?;
        let (ip, legacy) = source_key(ip);
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('auth-source-failure:lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='auth-source-failure:lock'")
            .execute_tx(&mut tx)
            .await?;
        let removed = if let Some(revision) = expected_revision {
            q("DELETE FROM auth_source_failure WHERE surface=?1 AND source_ip IN (?2,?3) AND revision=?4")
                .bind(surface)
                .bind(&ip)
                .bind(&legacy)
                .bind(revision)
                .execute_tx(&mut tx)
                .await?
        } else {
            q("DELETE FROM auth_source_failure WHERE surface=?1 AND source_ip IN (?2,?3)")
                .bind(surface)
                .bind(&ip)
                .bind(&legacy)
                .execute_tx(&mut tx)
                .await?
        };
        tx.commit().await?;
        Ok(removed >= 1)
    }
}
fn parse_attempts(row: &crate::backend::Row) -> Result<Vec<i64>, StoreError> {
    serde_json::from_str(&row.get_string("attempts_json"))
        .map_err(|_| StoreError::Corrupt("invalid authentication failure state".into()))
}
/// A lock from a row carrying `source_ip,locked_until,attempts_json,revision`.
fn lock_from(surface: &str, row: &crate::backend::Row) -> Result<AuthSourceLock, StoreError> {
    let ip = row.get_string("source_ip");
    Ok(AuthSourceLock {
        surface: surface.into(),
        network: network(&ip),
        ip,
        locked_until: row.get_i64("locked_until"),
        failures: parse_attempts(row)?.len(),
        revision: row.get_string("revision"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;

    const P5: FailurePolicy = FailurePolicy {
        threshold: 5,
        window_secs: 900,
        lock_secs: 900,
    };

    async fn exec(store: &Store, sql: String) {
        q(Box::leak(sql.into_boxed_str()))
            .execute(&store.backend)
            .await
            .unwrap();
    }
    async fn scalar(store: &Store, sql: String) -> i64 {
        q(Box::leak(sql.into_boxed_str()))
            .fetch_all(&store.backend)
            .await
            .unwrap()[0]
            .get_i64("total")
    }
    async fn total(store: &Store) -> i64 {
        scalar(
            store,
            "SELECT COUNT(*) AS total FROM auth_source_failure".into(),
        )
        .await
    }
    async fn present(store: &Store, surface: &str, ip: &str) -> bool {
        scalar(
            store,
            format!(
                "SELECT COUNT(*) AS total FROM auth_source_failure WHERE surface='{surface}' AND source_ip='{ip}'"
            ),
        )
        .await
            == 1
    }
    /// `count` rows for 10.<n/65536>.<n/256%256>.<n%256>, n = 1..=count, with the
    /// given lock deadline and `expires_at = expires_base + n` (so n=1 is oldest).
    async fn fill(
        store: &Store,
        surface: &str,
        prefix: &str,
        count: i64,
        locked_until: i64,
        expires_base: i64,
    ) {
        exec(store, format!(
            "WITH RECURSIVE seq(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM seq WHERE n<{count}) \
             INSERT INTO auth_source_failure(surface,source_ip,attempts_json,locked_until,expires_at,revision) \
             SELECT '{surface}','{prefix}.' || ((n/256)%256) || '.' || (n%256),'[100,101,102,103,104]',{locked_until},{expires_base}+n,'fixture' FROM seq"
        )).await;
    }

    /// 9,990 unlocked rows plus 10 active locks that are older by expiry than every
    /// unlocked row: a new source is admitted, exactly one row (the oldest UNLOCKED)
    /// is evicted, and every lock survives and is still enforced.
    async fn full_unlocked_table_evicts_oldest_unlocked_only(store: Store) {
        fill(&store, "webmail", "10.0", 9_990, 0, 2_000).await;
        fill(&store, "admin", "10.1", 10, 1_500, 1_499).await;
        assert_eq!(total(&store).await, CAPACITY);
        let new_ip: IpAddr = "192.0.2.44".parse().unwrap();
        assert!(
            store
                .auth_source_lock("admin", new_ip, 500)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .record_auth_source_failure("admin", new_ip, 500, P5)
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(total(&store).await, CAPACITY);
        assert!(present(&store, "admin", "192.0.2.44").await);
        assert!(
            !present(&store, "webmail", "10.0.0.1").await,
            "oldest unlocked evicted"
        );
        assert!(
            present(&store, "webmail", "10.0.0.2").await,
            "only one evicted"
        );
        let locks = scalar(
            &store,
            "SELECT COUNT(*) AS total FROM auth_source_failure WHERE locked_until>500".into(),
        )
        .await;
        assert_eq!(locks, 10);
        let lock = store
            .auth_source_lock("admin", "10.1.0.1".parse().unwrap(), 500)
            .await
            .unwrap()
            .expect("existing lock still blocks");
        assert_eq!(lock.locked_until, 1_500);
        // A locked source's own failure keeps its deadline and evicts nothing.
        let again = store
            .record_auth_source_failure("admin", "10.1.0.1".parse().unwrap(), 501, P5)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(again.locked_until, 1_500);
        assert!(present(&store, "webmail", "10.0.0.2").await);
    }

    /// Every record an active lock: a new source's check is not refused (no 503),
    /// its failure is not tracked, and no lock is displaced.
    async fn full_table_of_locks_does_not_refuse_new_sources(store: Store) {
        fill(&store, "webmail", "10.2", CAPACITY, 5_000, 5_000).await;
        let new_ip: IpAddr = "2001:db8:77::1".parse().unwrap();
        assert!(
            store
                .auth_source_lock("webmail", new_ip, 100)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .auth_source_lock("admin", new_ip, 100)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .record_auth_source_failure("webmail", new_ip, 100, P5)
                .await
                .is_err(),
            "tracking is refused only when every record is a lock"
        );
        assert_eq!(total(&store).await, CAPACITY);
        assert!(
            store
                .auth_source_lock("webmail", "10.2.0.1".parse().unwrap(), 100)
                .await
                .unwrap()
                .is_some()
        );
        // When the locks lapse, the source is tracked again.
        let lock = store
            .record_auth_source_failure(
                "webmail",
                new_ip,
                5_000,
                FailurePolicy { threshold: 1, ..P5 },
            )
            .await
            .unwrap();
        assert!(lock.is_some());
    }

    /// Two addresses in one IPv6 /64 are one source; another /64 is not; IPv4 and
    /// IPv4-mapped IPv6 stay keyed by the full address.
    async fn ipv6_slash64_shares_one_record(store: Store) {
        let p = FailurePolicy { threshold: 2, ..P5 };
        let a: IpAddr = "2001:db8:1:2::1".parse().unwrap();
        let b: IpAddr = "2001:db8:1:2:ffff:ffff:ffff:9".parse().unwrap();
        assert!(
            store
                .record_auth_source_failure("admin", a, 100, p)
                .await
                .unwrap()
                .is_none()
        );
        let lock = store
            .record_auth_source_failure("admin", b, 101, p)
            .await
            .unwrap()
            .expect("second failure in the same /64 locks it");
        assert_eq!(lock.ip, "2001:db8:1:2::");
        assert_eq!(lock.network, "2001:db8:1:2::/64");
        assert_eq!(total(&store).await, 1);
        let c: IpAddr = "2001:db8:1:2::abcd".parse().unwrap();
        assert!(
            store
                .auth_source_lock("admin", c, 102)
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            store
                .auth_source_lock("admin", "2001:db8:1:3::1".parse().unwrap(), 102)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .auth_source_lock("webmail", c, 102)
                .await
                .unwrap()
                .is_none()
        );
        // The listed address round-trips through the unlock endpoint's parse.
        let listed = store.list_auth_source_locks(102).await.unwrap();
        assert_eq!(listed.len(), 1);
        let listed_ip: IpAddr = listed[0].ip.parse().unwrap();
        assert!(
            store
                .clear_auth_source_failures("admin", listed_ip, Some(&listed[0].revision))
                .await
                .unwrap()
        );
        assert_eq!(total(&store).await, 0);
        let v4: IpAddr = "192.0.2.5".parse().unwrap();
        let v4b: IpAddr = "192.0.2.6".parse().unwrap();
        store
            .record_auth_source_failure("admin", v4, 200, p)
            .await
            .unwrap();
        assert!(
            store
                .record_auth_source_failure("admin", v4b, 201, p)
                .await
                .unwrap()
                .is_none()
        );
        let mapped: IpAddr = "::ffff:192.0.2.5".parse().unwrap();
        let lock = store
            .record_auth_source_failure("admin", mapped, 202, p)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(lock.network, "192.0.2.5");
    }

    /// Rows written before /64 keying (full IPv6 address) keep being enforced for
    /// that address until they expire, and can still be unlocked.
    async fn legacy_full_address_rows_are_honoured(store: Store) {
        exec(&store, "INSERT INTO auth_source_failure(surface,source_ip,attempts_json,locked_until,expires_at,revision) VALUES('admin','2001:db8:9:9::5','[1,2,3,4,5]',1000,1000,'legacy')".into()).await;
        let old: IpAddr = "2001:db8:9:9::5".parse().unwrap();
        let lock = store
            .auth_source_lock("admin", old, 100)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(lock.revision, "legacy");
        assert_eq!(
            store
                .record_auth_source_failure("admin", old, 101, P5)
                .await
                .unwrap()
                .unwrap()
                .revision,
            "legacy"
        );
        assert!(
            store
                .clear_auth_source_failures("admin", old, Some("legacy"))
                .await
                .unwrap()
        );
        assert!(
            store
                .auth_source_lock("admin", old, 102)
                .await
                .unwrap()
                .is_none()
        );
    }

    /// Concurrent new sources racing on a full table never exceed the cap, are all
    /// tracked, and displace exactly the oldest unlocked rows.
    async fn concurrent_inserts_at_capacity_respect_the_cap(store: Store) {
        fill(&store, "webmail", "10.3", CAPACITY, 0, 2_000).await;
        let mut jobs = Vec::new();
        for i in 0..16 {
            let s = store.clone();
            jobs.push(tokio::spawn(async move {
                let ip: IpAddr = format!("198.51.100.{}", i + 1).parse().unwrap();
                s.auth_source_lock("admin", ip, 500).await.unwrap();
                // A day-long window: each new record outlives every fixture row,
                // so racers must evict fixture rows, never each other.
                let p = FailurePolicy {
                    window_secs: 86_400,
                    ..P5
                };
                s.record_auth_source_failure("admin", ip, 500, p)
                    .await
                    .unwrap()
            }));
        }
        for job in jobs {
            assert!(job.await.unwrap().is_none());
        }
        assert_eq!(total(&store).await, CAPACITY);
        for i in 1..=16 {
            assert!(present(&store, "admin", &format!("198.51.100.{i}")).await);
            assert!(!present(&store, "webmail", &format!("10.3.0.{i}")).await);
        }
        assert!(present(&store, "webmail", "10.3.0.17").await);
    }

    async fn sqlite() -> Store {
        Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn capacity_evicts_oldest_unlocked_sqlite() {
        full_unlocked_table_evicts_oldest_unlocked_only(sqlite().await).await;
    }
    #[tokio::test]
    async fn capacity_of_locks_fails_open_on_check_sqlite() {
        full_table_of_locks_does_not_refuse_new_sources(sqlite().await).await;
    }
    #[tokio::test]
    async fn ipv6_slash64_keying_sqlite() {
        ipv6_slash64_shares_one_record(sqlite().await).await;
    }
    #[tokio::test]
    async fn legacy_rows_sqlite() {
        legacy_full_address_rows_are_honoured(sqlite().await).await;
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_capacity_sqlite() {
        concurrent_inserts_at_capacity_respect_the_cap(sqlite().await).await;
    }

    /// Each scenario on its own fresh PostgreSQL database when `MW_TEST_PG` is set.
    async fn on_pg<F, Fut>(scenario: F)
    where
        F: FnOnce(Store) -> Fut,
        Fut: Future<Output = ()>,
    {
        let Ok(dsn) = std::env::var("MW_TEST_PG") else {
            return;
        };
        let admin = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&dsn)
            .await
            .unwrap();
        let name = format!("mw_auth_cap_{:032x}", rand::random::<u128>());
        sqlx::query(&format!("CREATE DATABASE {name}"))
            .execute(&admin)
            .await
            .unwrap();
        let mut url = url::Url::parse(&dsn).unwrap();
        url.set_path(&format!("/{name}"));
        let store = Store::open(url.as_str(), crate::ServerKey::generate())
            .await
            .unwrap();
        scenario(store.clone()).await;
        drop(store);
        sqlx::query(&format!("DROP DATABASE {name} WITH (FORCE)"))
            .execute(&admin)
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn capacity_evicts_oldest_unlocked_postgres() {
        on_pg(full_unlocked_table_evicts_oldest_unlocked_only).await;
    }
    #[tokio::test]
    async fn capacity_of_locks_fails_open_on_check_postgres() {
        on_pg(full_table_of_locks_does_not_refuse_new_sources).await;
    }
    #[tokio::test]
    async fn ipv6_slash64_keying_postgres() {
        on_pg(ipv6_slash64_shares_one_record).await;
    }
    #[tokio::test]
    async fn legacy_rows_postgres() {
        on_pg(legacy_full_address_rows_are_honoured).await;
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_capacity_postgres() {
        on_pg(concurrent_inserts_at_capacity_respect_the_cap).await;
    }
}
