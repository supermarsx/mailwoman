//! Durable, bounded failed-credential counters; no account or credential data.
use crate::{Store, StoreError, backend::q};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
const CAPACITY: i64 = 10_000;
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
    pub ip: String,
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
    /// A full inventory cannot silently grant an untracked source a new allowance.
    pub async fn auth_source_lock(
        &self,
        surface: &str,
        ip: IpAddr,
        now: i64,
    ) -> Result<Option<AuthSourceLock>, StoreError> {
        scope(surface)?;
        let ip = ip.to_canonical().to_string();
        let rows=q("SELECT locked_until,attempts_json,revision FROM auth_source_failure WHERE surface=?1 AND source_ip=?2 AND expires_at>?3").bind(surface).bind(&ip).bind(now).fetch_all(&self.backend).await?;
        if let Some(row) = rows.first() {
            let until = row.get_i64("locked_until");
            if until <= now {
                return Ok(None);
            }
            let attempts: Vec<i64> = serde_json::from_str(&row.get_string("attempts_json"))
                .map_err(|_| StoreError::Corrupt("invalid authentication failure state".into()))?;
            return Ok(Some(AuthSourceLock {
                surface: surface.into(),
                ip,
                locked_until: until,
                failures: attempts.len(),
                revision: row.get_string("revision"),
            }));
        }
        let count = q("SELECT COUNT(*) AS total FROM auth_source_failure WHERE expires_at>?1")
            .bind(now)
            .fetch_all(&self.backend)
            .await?[0]
            .get_i64("total");
        if count >= CAPACITY {
            return Err(StoreError::Corrupt(
                "authentication failure state capacity reached".into(),
            ));
        }
        Ok(None)
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
        let ip = ip.to_canonical().to_string();
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('auth-source-failure:lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='auth-source-failure:lock'")
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM auth_source_failure WHERE expires_at<=?1")
            .bind(now)
            .execute_tx(&mut tx)
            .await?;
        let rows=q("SELECT locked_until,attempts_json,revision FROM auth_source_failure WHERE surface=?1 AND source_ip=?2").bind(surface).bind(&ip).fetch_all_tx(&mut tx).await?;
        let mut attempts: Vec<i64> = Vec::new();
        if let Some(row) = rows.first() {
            attempts = serde_json::from_str(&row.get_string("attempts_json"))
                .map_err(|_| StoreError::Corrupt("invalid authentication failure state".into()))?;
            if attempts.len() > 100 {
                return Err(StoreError::Corrupt(
                    "oversized authentication failure state".into(),
                ));
            }
            let until = row.get_i64("locked_until");
            if until > now {
                let lock = AuthSourceLock {
                    surface: surface.into(),
                    ip,
                    locked_until: until,
                    failures: attempts.len(),
                    revision: row.get_string("revision"),
                };
                tx.commit().await?;
                return Ok(Some(lock));
            }
            if until != 0 {
                attempts.clear();
            }
        } else {
            let count = q("SELECT COUNT(*) AS total FROM auth_source_failure")
                .fetch_all_tx(&mut tx)
                .await?[0]
                .get_i64("total");
            if count >= CAPACITY {
                return Err(StoreError::Corrupt(
                    "authentication failure state capacity reached".into(),
                ));
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
            .map(|row| {
                let attempts: Vec<i64> = serde_json::from_str(&row.get_string("attempts_json"))
                    .map_err(|_| {
                        StoreError::Corrupt("invalid authentication failure state".into())
                    })?;
                Ok(AuthSourceLock {
                    surface: row.get_string("surface"),
                    ip: row.get_string("source_ip"),
                    locked_until: row.get_i64("locked_until"),
                    failures: attempts.len(),
                    revision: row.get_string("revision"),
                })
            })
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
        let ip = ip.to_canonical().to_string();
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO settings(key,value) VALUES('auth-source-failure:lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(&mut tx).await?;
        q("UPDATE settings SET value=value WHERE key='auth-source-failure:lock'")
            .execute_tx(&mut tx)
            .await?;
        let removed = if let Some(revision) = expected_revision {
            q("DELETE FROM auth_source_failure WHERE surface=?1 AND source_ip=?2 AND revision=?3")
                .bind(surface)
                .bind(&ip)
                .bind(revision)
                .execute_tx(&mut tx)
                .await?
        } else {
            q("DELETE FROM auth_source_failure WHERE surface=?1 AND source_ip=?2")
                .bind(surface)
                .bind(&ip)
                .execute_tx(&mut tx)
                .await?
        };
        tx.commit().await?;
        Ok(removed == 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn capacity_does_not_evict_live_locks_or_grant_new_allowances() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        q("WITH RECURSIVE seq(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM seq WHERE n<10000) INSERT INTO auth_source_failure(surface,source_ip,attempts_json,locked_until,expires_at,revision) SELECT 'admin',CAST(n AS TEXT),'[100]',200,200,'fixture' FROM seq").execute(&store.backend).await.unwrap();
        let ip = "192.0.2.44".parse().unwrap();
        assert!(store.auth_source_lock("admin", ip, 101).await.is_err());
        let policy = FailurePolicy {
            threshold: 1,
            window_secs: 60,
            lock_secs: 120,
        };
        assert!(
            store
                .record_auth_source_failure("admin", ip, 101, policy)
                .await
                .is_err()
        );
        let count = q("SELECT COUNT(*) AS total FROM auth_source_failure")
            .fetch_all(&store.backend)
            .await
            .unwrap()[0]
            .get_i64("total");
        assert_eq!(count, 10000);
        assert!(
            store
                .record_auth_source_failure("admin", ip, 200, policy)
                .await
                .unwrap()
                .is_some()
        );
        let count = q("SELECT COUNT(*) AS total FROM auth_source_failure")
            .fetch_all(&store.backend)
            .await
            .unwrap()[0]
            .get_i64("total");
        assert_eq!(count, 1);
    }
}
