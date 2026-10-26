//! Durable, idempotent aggregate snapshots. No request/user content is stored.
use crate::{Store, StoreError, q};
use serde::{Deserialize, Serialize};

pub const LATENCY_BOUNDS_MS: [u64; 13] =
    [1, 2, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000];
pub const MAX_HISTORY_ROWS: usize = 20_000;
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Counts {
    pub requests: u64,
    pub errors5xx: u64,
    pub histogram: [u64; 14],
    pub max_ms: u64,
}
impl Counts {
    pub fn record(&mut self, status: u16, ms: u64) {
        self.requests = self.requests.saturating_add(1);
        self.errors5xx = self.errors5xx.saturating_add(u64::from(status >= 500));
        let bucket = LATENCY_BOUNDS_MS
            .iter()
            .position(|bound| ms <= *bound)
            .unwrap_or(13);
        self.histogram[bucket] = self.histogram[bucket].saturating_add(1);
        self.max_ms = self.max_ms.max(ms);
    }
    pub fn merge(&mut self, other: &Self) {
        self.requests = self.requests.saturating_add(other.requests);
        self.errors5xx = self.errors5xx.saturating_add(other.errors5xx);
        for (a, b) in self.histogram.iter_mut().zip(other.histogram) {
            *a = a.saturating_add(b);
        }
        self.max_ms = self.max_ms.max(other.max_ms);
    }
    pub fn percentile(&self, percent: u64) -> Option<u64> {
        if self.requests == 0 {
            return None;
        }
        let rank = self.requests.saturating_mul(percent).div_ceil(100);
        let mut seen = 0_u64;
        for (index, count) in self.histogram.iter().enumerate() {
            seen = seen.saturating_add(*count);
            if seen >= rank {
                return Some(LATENCY_BOUNDS_MS.get(index).copied().unwrap_or(self.max_ms));
            }
        }
        None
    }
}
#[derive(Clone, Debug)]
pub struct Bucket {
    pub seconds: i64,
    pub start: i64,
    pub observed_from: i64,
    pub observed_to: i64,
    pub counts: Counts,
}
impl Store {
    /// Absolute snapshots can be retried after uncertain commit without adding twice.
    pub async fn save_request_metrics(
        &self,
        instance: &str,
        buckets: &[Bucket],
        now: i64,
        retention_days: i64,
    ) -> Result<(), StoreError> {
        if instance.is_empty()
            || instance.len() > 64
            || buckets.len() > 256
            || !(1..=400).contains(&retention_days)
        {
            return Err(StoreError::Corrupt(
                "invalid metrics snapshot bounds".into(),
            ));
        }
        let mut tx = self.backend.begin().await?;
        for bucket in buckets {
            if ![60, 3600].contains(&bucket.seconds)
                || bucket.start < 0
                || bucket.start % bucket.seconds != 0
                || bucket.observed_from < bucket.start
                || bucket.observed_to < bucket.observed_from
                || bucket.observed_to > bucket.start + bucket.seconds
                || bucket.counts.errors5xx > bucket.counts.requests
                || bucket
                    .counts
                    .histogram
                    .iter()
                    .try_fold(0_u64, |sum, n| sum.checked_add(*n))
                    != Some(bucket.counts.requests)
            {
                return Err(StoreError::Corrupt("invalid metrics bucket".into()));
            }
            let data = serde_json::to_string(&bucket.counts)
                .map_err(|e| StoreError::Corrupt(e.to_string()))?;
            q("INSERT INTO request_metrics_bucket(instance_id,bucket_seconds,bucket_start,observed_from,observed_to,counts_json) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(instance_id,bucket_seconds,bucket_start) DO UPDATE SET observed_from=excluded.observed_from,observed_to=excluded.observed_to,counts_json=excluded.counts_json WHERE excluded.observed_to>request_metrics_bucket.observed_to")
                .bind(instance).bind(bucket.seconds).bind(bucket.start).bind(bucket.observed_from).bind(bucket.observed_to).bind(data).execute_tx(&mut tx).await?;
        }
        let cutoff = now.saturating_sub(retention_days * 86400);
        let minute_cutoff = now.saturating_sub(48 * 3600).max(cutoff);
        q("DELETE FROM request_metrics_bucket WHERE (instance_id,bucket_seconds,bucket_start) IN (SELECT instance_id,bucket_seconds,bucket_start FROM request_metrics_bucket WHERE (bucket_seconds=3600 AND bucket_start<=?1-3600) OR (bucket_seconds=60 AND bucket_start<=?2-60) ORDER BY bucket_start LIMIT 1000)").bind(cutoff).bind(minute_cutoff).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }
    /// Minute precision for recent ranges, hourly precision for older ranges.
    /// Fail closed when the bounded raw scan cannot represent all replicas.
    pub async fn request_metrics_history(
        &self,
        from: i64,
        to: i64,
        seconds: i64,
    ) -> Result<Vec<Bucket>, StoreError> {
        if from < 0 || to <= from || to - from > 401 * 86400 || ![60, 3600].contains(&seconds) {
            return Err(StoreError::Corrupt("invalid metrics range".into()));
        }
        let rows = q("SELECT bucket_seconds,bucket_start,observed_from,observed_to,counts_json FROM request_metrics_bucket WHERE bucket_seconds=?1 AND bucket_start>=?2 AND bucket_start<?3 ORDER BY bucket_start,instance_id LIMIT 20001")
            .bind(seconds).bind(from).bind(to).fetch_all(&self.backend).await?;
        if rows.len() > MAX_HISTORY_ROWS {
            return Err(StoreError::Corrupt(
                "metrics range exceeds bounded history scan; choose a shorter range".into(),
            ));
        }
        rows.into_iter()
            .map(|row| {
                Ok(Bucket {
                    seconds: row.get_i64("bucket_seconds"),
                    start: row.get_i64("bucket_start"),
                    observed_from: row.get_i64("observed_from"),
                    observed_to: row.get_i64("observed_to"),
                    counts: serde_json::from_str(&row.get_string("counts_json"))
                        .map_err(|e| StoreError::Corrupt(e.to_string()))?,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    async fn exercise(store: Store) {
        let time = 3600 * 100;
        let mut counts = Counts::default();
        counts.record(200, 2);
        counts.record(503, 400);
        let mut bucket = Bucket {
            seconds: 60,
            start: time,
            observed_from: time,
            observed_to: time + 30,
            counts,
        };
        for _ in 0..2 {
            store
                .save_request_metrics("one", std::slice::from_ref(&bucket), time + 30, 30)
                .await
                .unwrap();
        }
        let rows = store
            .request_metrics_history(time, time + 60, 60)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].counts.requests, 2);
        assert_eq!(rows[0].counts.percentile(95), Some(500));
        // A stale delayed snapshot never replaces a later committed observation.
        bucket.observed_to = time + 30;
        bucket.counts = Counts::default();
        store
            .save_request_metrics("one", std::slice::from_ref(&bucket), time + 30, 30)
            .await
            .unwrap();
        assert_eq!(
            store
                .request_metrics_history(time, time + 60, 60)
                .await
                .unwrap()[0]
                .counts
                .requests,
            2
        );
        bucket.observed_from = time + 30;
        bucket.observed_to = time + 60;
        bucket.counts.record(200, 100);
        store
            .save_request_metrics("restart", &[bucket], time + 60, 30)
            .await
            .unwrap();
        assert_eq!(
            store
                .request_metrics_history(time, time + 60, 60)
                .await
                .unwrap()
                .len(),
            2
        );
        let hour = Bucket {
            seconds: 3600,
            start: time,
            observed_from: time,
            observed_to: time + 3600,
            counts: Counts::default(),
        };
        store
            .save_request_metrics("one", &[hour], time + 3600, 30)
            .await
            .unwrap();
        store
            .save_request_metrics("one", &[], time + 49 * 3600, 30)
            .await
            .unwrap();
        assert!(
            store
                .request_metrics_history(time, time + 60, 60)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            store
                .request_metrics_history(time, time + 3600, 3600)
                .await
                .unwrap()
                .len(),
            1
        );
        store
            .save_request_metrics("one", &[], time + 31 * 86400, 30)
            .await
            .unwrap();
        assert!(
            store
                .request_metrics_history(time, time + 3600, 3600)
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            store
                .request_metrics_history(time, time + 402 * 86400, 3600)
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn durable_metrics_idempotency_restart_and_retention_sqlite() {
        exercise(
            Store::open_in_memory(crate::ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    }
    #[tokio::test]
    async fn durable_metrics_idempotency_restart_and_retention_postgres() {
        let Ok(dsn) = std::env::var("MW_DIRECTORY_TEST_PG_DSN") else {
            eprintln!("SKIP live PostgreSQL: MW_DIRECTORY_TEST_PG_DSN unset");
            return;
        };
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&dsn)
            .await
            .unwrap();
        let schema = format!("metrics_{}", rand::random::<u64>());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&pool)
            .await
            .unwrap();
        let sep = if dsn.contains('?') { '&' } else { '?' };
        let scoped = format!("{dsn}{sep}options=-c%20search_path%3D{schema}");
        let result = tokio::spawn(async move {
            exercise(
                Store::open_postgres(&scoped, crate::ServerKey::generate())
                    .await
                    .unwrap(),
            )
            .await;
        })
        .await;
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&pool)
            .await
            .unwrap();
        result.unwrap();
    }
}
