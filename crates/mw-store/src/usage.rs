//! Per-person usage counters for enforced limits (`usage_counter`, migration 0033).
//!
//! One row per address, metric and period. The `day` column holds the period
//! number in the metric's own unit: a metric whose name ends in `.hour` counts
//! clock hours since the Unix epoch, every other metric counts UTC days.
//!
//! [`Store::usage_reserve`] is one conditional upsert, so two server instances
//! (or two `Store` handles) sharing one database can never admit more than the
//! limit together: the row is only changed when the new total stays within it.
//! A caller that reserves and then fails gives the amount back with
//! [`Store::usage_release`].
use crate::{Store, StoreError, q};

/// EmailSubmission/set creates, per clock hour (`limit.send_per_hour`).
pub const SEND_HOUR: &str = "send.hour";
/// EmailSubmission/set creates, per UTC day (`limit.send_per_day`).
pub const SEND_DAY: &str = "send.day";
/// Assistant calls, per UTC day (`limit.ai_calls_per_day`).
pub const ASSIST_CALLS: &str = "assist.calls";

/// Whether `metric` is counted per clock hour (its name ends in `.hour`).
pub fn is_hourly(metric: &str) -> bool {
    metric.ends_with(".hour")
}

/// The period number of `metric` at `now_unix` seconds: the epoch hour for an
/// hourly metric, else the UTC day.
pub fn period_of(metric: &str, now_unix: i64) -> i64 {
    now_unix.div_euclid(if is_hourly(metric) { 3600 } else { 86_400 })
}

fn valid(address: &str, metric: &str) -> Result<(), StoreError> {
    if address.is_empty() || address.len() > 512 || metric.is_empty() || metric.len() > 64 {
        return Err(StoreError::Invalid("usage counter address or metric out of bounds"));
    }
    Ok(())
}

impl Store {
    /// Add `amount` to the counter of `address`/`metric`/`period` if the new
    /// total stays within `limit`. Returns `Some(new_total)` when reserved and
    /// `None` when the reservation would exceed `limit` (nothing is changed).
    /// `limit <= 0` means no limit: the amount is always added.
    ///
    /// One statement: an `INSERT ... ON CONFLICT DO UPDATE ... WHERE` whose
    /// `RETURNING` yields no row when the condition refuses the update.
    pub async fn usage_reserve(
        &self,
        address: &str,
        domain: &str,
        metric: &str,
        period: i64,
        amount: i64,
        limit: i64,
    ) -> Result<Option<i64>, StoreError> {
        valid(address, metric)?;
        if amount <= 0 {
            return Err(StoreError::Invalid("usage reservation must be positive"));
        }
        let limit = if limit <= 0 { i64::MAX } else { limit };
        if amount > limit {
            return Ok(None);
        }
        let row = q("INSERT INTO usage_counter(address,domain,metric,day,value) VALUES(?1,?2,?3,?4,?5) \
             ON CONFLICT(address,metric,day) DO UPDATE SET value=usage_counter.value+excluded.value \
             WHERE usage_counter.value<=?6-excluded.value RETURNING value")
            .bind(address)
            .bind(domain)
            .bind(metric)
            .bind(period)
            .bind(amount)
            .bind(limit)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.map(|row| row.get_i64_idx(0)))
    }

    /// Give back `amount` reserved with [`Store::usage_reserve`]. The counter
    /// never goes below zero. Returns whether a row was changed.
    pub async fn usage_release(
        &self,
        address: &str,
        metric: &str,
        period: i64,
        amount: i64,
    ) -> Result<bool, StoreError> {
        valid(address, metric)?;
        if amount <= 0 {
            return Ok(false);
        }
        let changed = q("UPDATE usage_counter SET value=CASE WHEN value>?4 THEN value-?4 ELSE 0 END \
             WHERE address=?1 AND metric=?2 AND day=?3")
            .bind(address)
            .bind(metric)
            .bind(period)
            .bind(amount)
            .execute(&self.backend)
            .await?;
        Ok(changed > 0)
    }

    /// Sum of the counter of `address`/`metric` over periods `from..=to`.
    pub async fn usage_sum(
        &self,
        address: &str,
        metric: &str,
        from: i64,
        to: i64,
    ) -> Result<i64, StoreError> {
        valid(address, metric)?;
        if to < from {
            return Ok(0);
        }
        Ok(q("SELECT CAST(COALESCE(SUM(value),0) AS BIGINT) FROM usage_counter \
             WHERE address=?1 AND metric=?2 AND day>=?3 AND day<=?4")
            .bind(address)
            .bind(metric)
            .bind(from)
            .bind(to)
            .fetch_scalar_i64(&self.backend)
            .await?)
    }

    /// Delete counter rows whose period ended before `cutoff_unix` seconds:
    /// hourly metrics by epoch hour, every other metric by UTC day. Returns
    /// the number of rows deleted (`usage.retentionDays`).
    pub async fn usage_prune(&self, cutoff_unix: i64) -> Result<u64, StoreError> {
        let hour = cutoff_unix.div_euclid(3600);
        let day = cutoff_unix.div_euclid(86_400);
        Ok(q("DELETE FROM usage_counter WHERE \
             (metric LIKE '%.hour' AND day<?1) OR (metric NOT LIKE '%.hour' AND day<?2)")
            .bind(hour)
            .bind(day)
            .execute(&self.backend)
            .await?)
    }

    /// Bytes this server holds for `account_id` (`limit.storage_bytes`, engine
    /// mode): the sizes of its cached messages plus its uploaded blobs.
    pub async fn usage_storage_bytes(&self, account_id: &str) -> Result<i64, StoreError> {
        Ok(q("SELECT CAST(COALESCE((SELECT SUM(size) FROM messages WHERE account_id=?1),0) \
             + COALESCE((SELECT SUM(size) FROM uploaded_blobs WHERE account_id=?1),0) AS BIGINT)")
            .bind(account_id)
            .fetch_scalar_i64(&self.backend)
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    async fn exercise(a: &Store, b: &Store, address: &str) {
        let period = period_of(SEND_HOUR, 1_700_000_000);
        assert_eq!(period, 1_700_000_000 / 3600);
        assert_eq!(period_of(SEND_DAY, 1_700_000_000), 1_700_000_000 / 86_400);
        // Two handles share one counter; the third of a limit of 2 is refused.
        assert_eq!(a.usage_reserve(address, "example.org", SEND_HOUR, period, 1, 2).await.unwrap(), Some(1));
        assert_eq!(b.usage_reserve(address, "example.org", SEND_HOUR, period, 1, 2).await.unwrap(), Some(2));
        assert_eq!(a.usage_reserve(address, "example.org", SEND_HOUR, period, 1, 2).await.unwrap(), None);
        assert_eq!(b.usage_sum(address, SEND_HOUR, period, period).await.unwrap(), 2);
        // A refused reservation changed nothing; a release makes room again.
        assert!(a.usage_release(address, SEND_HOUR, period, 1).await.unwrap());
        assert_eq!(b.usage_reserve(address, "example.org", SEND_HOUR, period, 1, 2).await.unwrap(), Some(2));
        // An amount over the limit on a fresh row is refused, not inserted.
        assert_eq!(a.usage_reserve(address, "example.org", SEND_DAY, 5, 3, 2).await.unwrap(), None);
        assert_eq!(a.usage_sum(address, SEND_DAY, 0, 10).await.unwrap(), 0);
        // No limit.
        assert_eq!(a.usage_reserve(address, "example.org", SEND_DAY, 5, 3, 0).await.unwrap(), Some(3));
        // Release never goes below zero.
        a.usage_release(address, SEND_DAY, 5, 10).await.unwrap();
        assert_eq!(a.usage_sum(address, SEND_DAY, 5, 5).await.unwrap(), 0);
        // Prune uses each metric's own unit: a day cutoff after the hour row's
        // day deletes it, a day row at the cutoff day is kept.
        a.usage_reserve(address, "example.org", SEND_DAY, period_of(SEND_DAY, 1_700_000_000), 1, 0).await.unwrap();
        let cutoff = (period_of(SEND_DAY, 1_700_000_000)) * 86_400;
        a.usage_prune(cutoff).await.unwrap();
        assert_eq!(a.usage_sum(address, SEND_HOUR, period, period).await.unwrap(), 2);
        assert_eq!(a.usage_sum(address, SEND_DAY, 5, 5).await.unwrap(), 0);
        assert_eq!(a.usage_sum(address, SEND_DAY, 0, i64::MAX).await.unwrap(), 1);
        a.usage_prune(cutoff + 86_400).await.unwrap();
        assert_eq!(a.usage_sum(address, SEND_HOUR, 0, i64::MAX).await.unwrap(), 0);
        assert_eq!(a.usage_sum(address, SEND_DAY, 0, i64::MAX).await.unwrap(), 0);
        assert!(a.usage_reserve("", "x", SEND_DAY, 1, 1, 0).await.is_err());
        assert_eq!(a.usage_storage_bytes("no-such-account").await.unwrap(), 0);
    }

    #[tokio::test]
    async fn sqlite_reserve_release_sum_prune() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("usage.db");
        let dsn = path.to_str().unwrap();
        let key = ServerKey::generate();
        let a = Store::open(dsn, key.clone()).await.unwrap();
        let b = Store::open(dsn, key).await.unwrap();
        exercise(&a, &b, "a@example.org").await;
    }

    #[tokio::test]
    async fn postgres_reserve_release_sum_prune() {
        let Ok(url) = std::env::var("MW_TEST_PG") else {
            eprintln!("SKIP: MW_TEST_PG absent");
            return;
        };
        let key = ServerKey::generate();
        let a = Store::open(&url, key.clone()).await.unwrap();
        let b = Store::open(&url, key).await.unwrap();
        let address = format!("usage-{}@example.org", uuid::Uuid::new_v4());
        exercise(&a, &b, &address).await;
        q("DELETE FROM usage_counter WHERE address=?1").bind(address.as_str()).execute(&a.backend).await.unwrap();
    }
}
