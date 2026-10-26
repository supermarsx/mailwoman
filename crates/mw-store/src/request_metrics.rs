//! Durable, idempotent aggregate snapshots. No request/user content is stored.
use crate::{Store, StoreError, q};
use serde::{Deserialize, Serialize};

mod histogram;
pub use histogram::BOUNDS_MS as PRECISE_LATENCY_BOUNDS_MS;

pub const LATENCY_BOUNDS_MS: [u64; 13] =
    [1, 2, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000];
pub const MAX_HISTORY_ROWS: usize = 20_000;
pub const MAX_HISTORY_JSON_BYTES: i64 = 16 * 1024 * 1024;
const MAX_BUCKET_JSON_BYTES: i64 = 128 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Counts {
    pub requests: u64,
    pub errors5xx: u64,
    pub histogram: [u64; 14],
    pub max_ms: u64,
    #[serde(default, rename = "histogramVersion")]
    histogram_version: u8,
    #[serde(
        default,
        rename = "fineHistogram",
        skip_serializing_if = "Option::is_none"
    )]
    fine_histogram: Option<histogram::Histogram>,
}
impl Default for Counts {
    fn default() -> Self {
        Self {
            requests: 0,
            errors5xx: 0,
            histogram: [0; 14],
            max_ms: 0,
            histogram_version: 2,
            fine_histogram: Some(histogram::Histogram::default()),
        }
    }
}
impl Counts {
    pub fn record(&mut self, status: u16, ms: u64) {
        if self.requests == 0 {
            self.histogram_version = 2;
            self.fine_histogram = Some(histogram::Histogram::default());
        }
        self.requests = self.requests.saturating_add(1);
        self.errors5xx = self.errors5xx.saturating_add(u64::from(status >= 500));
        let bucket = LATENCY_BOUNDS_MS
            .iter()
            .position(|bound| ms <= *bound)
            .unwrap_or(13);
        self.histogram[bucket] = self.histogram[bucket].saturating_add(1);
        self.max_ms = self.max_ms.max(ms);
        if let Some(fine) = &mut self.fine_histogram {
            fine.record(ms);
        }
    }
    pub fn merge(&mut self, other: &Self) {
        if self.requests == 0 {
            self.histogram_version = other.histogram_version;
            self.fine_histogram = other.fine_histogram.clone();
        } else if other.requests > 0 {
            match (&mut self.fine_histogram, &other.fine_histogram) {
                (Some(ours), Some(theirs)) => ours.merge(theirs),
                _ => {
                    self.fine_histogram = None;
                    self.histogram_version = 1;
                }
            }
        }
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
        let rank = self
            .requests
            .saturating_mul(percent.clamp(1, 100))
            .div_ceil(100);
        if let Some(fine) = &self.fine_histogram {
            return fine.percentile(rank, self.max_ms).map(|(value, _)| value);
        }
        let mut seen = 0_u64;
        for (index, count) in self.histogram.iter().enumerate() {
            seen = seen.saturating_add(*count);
            if seen >= rank {
                return Some(LATENCY_BOUNDS_MS.get(index).copied().unwrap_or(self.max_ms));
            }
        }
        None
    }
    pub fn legacy_estimate(&self) -> bool {
        self.requests > 0 && self.fine_histogram.is_none()
    }
    pub fn percentile_resolution_ms(&self, percent: u64) -> Option<u64> {
        let rank = self
            .requests
            .saturating_mul(percent.clamp(1, 100))
            .div_ceil(100);
        self.fine_histogram
            .as_ref()?
            .percentile(rank, self.max_ms)?
            .1
    }
    fn valid(&self) -> bool {
        self.errors5xx <= self.requests
            && self
                .histogram
                .iter()
                .try_fold(0_u64, |sum, n| sum.checked_add(*n))
                == Some(self.requests)
            && match (&self.fine_histogram, self.histogram_version) {
                (Some(fine), 2) => {
                    let mut projected = [0_u64; 14];
                    for (index, count) in fine.entries() {
                        let upper = if index == histogram::OVERFLOW_INDEX {
                            self.max_ms
                        } else {
                            histogram::upper_bound(index)
                        };
                        let legacy = LATENCY_BOUNDS_MS
                            .iter()
                            .position(|bound| upper <= *bound)
                            .unwrap_or(13);
                        projected[legacy] = projected[legacy].saturating_add(count);
                    }
                    fine.total() == Some(self.requests)
                        && fine.compatible_max(self.max_ms)
                        && projected == self.histogram
                }
                (None, 0 | 1) => true,
                _ => false,
            }
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
                || !bucket.counts.valid()
            {
                return Err(StoreError::Corrupt("invalid metrics bucket".into()));
            }
            let data = serde_json::to_string(&bucket.counts)
                .map_err(|e| StoreError::Corrupt(e.to_string()))?;
            if data.len() as i64 > MAX_BUCKET_JSON_BYTES {
                return Err(StoreError::Corrupt(
                    "metrics bucket exceeds byte budget".into(),
                ));
            }
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
        // Suppress oversized payloads inside the same SQL snapshot, before transfer.
        macro_rules! history_query {
            ($bytes:literal) => {concat!(
                "SELECT bucket_seconds,bucket_start,observed_from,observed_to,CAST(",$bytes," AS BIGINT) AS document_bytes,CAST(SUM(",$bytes,
                ") OVER () AS BIGINT) AS total_bytes,CASE WHEN ",$bytes,"<=?4 AND SUM(",$bytes,
                ") OVER ()<=?5 THEN counts_json ELSE NULL END AS counts_json FROM (SELECT bucket_seconds,bucket_start,observed_from,observed_to,counts_json,instance_id FROM request_metrics_bucket WHERE bucket_seconds=?1 AND bucket_start>=?2 AND bucket_start<?3 ORDER BY bucket_start,instance_id LIMIT 20001) bounded ORDER BY bucket_start,instance_id"
            )};
        }
        let sql = match self.backend.dialect() {
            crate::backend::Dialect::Sqlite => history_query!("LENGTH(CAST(counts_json AS BLOB))"),
            crate::backend::Dialect::Postgres => history_query!("OCTET_LENGTH(counts_json)"),
        };
        let rows = q(sql)
            .bind(seconds)
            .bind(from)
            .bind(to)
            .bind(MAX_BUCKET_JSON_BYTES)
            .bind(MAX_HISTORY_JSON_BYTES)
            .fetch_all(&self.backend)
            .await?;
        if rows.len() > MAX_HISTORY_ROWS {
            return Err(StoreError::Corrupt(
                "metrics range exceeds bounded history scan; choose a shorter range".into(),
            ));
        }
        rows.into_iter()
            .map(|row| {
                if row.get_i64("document_bytes") > MAX_BUCKET_JSON_BYTES
                    || row.get_i64("total_bytes") > MAX_HISTORY_JSON_BYTES
                {
                    return Err(StoreError::Corrupt(
                        "metrics history exceeds decoded byte budget; choose a shorter range"
                            .into(),
                    ));
                }
                let counts: Counts = serde_json::from_str(&row.get_string("counts_json"))
                    .map_err(|e| StoreError::Corrupt(e.to_string()))?;
                if !counts.valid() {
                    return Err(StoreError::Corrupt(
                        "invalid stored metrics histogram".into(),
                    ));
                }
                Ok(Bucket {
                    seconds: row.get_i64("bucket_seconds"),
                    start: row.get_i64("bucket_start"),
                    observed_from: row.get_i64("observed_from"),
                    observed_to: row.get_i64("observed_to"),
                    counts,
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
        assert_eq!(rows[0].counts.percentile(95), Some(400));
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
        byte_budget_checks(&store).await;
    }
    async fn byte_budget_checks(store: &Store) {
        let start = 3600;
        let oversized = " ".repeat(MAX_BUCKET_JSON_BYTES as usize + 1);
        q("INSERT INTO request_metrics_bucket(instance_id,bucket_seconds,bucket_start,observed_from,observed_to,counts_json) VALUES('oversized-fixture',60,?1,?1,?2,?3)").bind(start).bind(start+60).bind(oversized).execute(&store.backend).await.unwrap();
        assert!(
            store
                .request_metrics_history(start, start + 60, 60)
                .await
                .unwrap_err()
                .to_string()
                .contains("byte budget")
        );
        q("DELETE FROM request_metrics_bucket WHERE instance_id='oversized-fixture'")
            .execute(&store.backend)
            .await
            .unwrap();
        let mut document = serde_json::to_value(Counts::default()).unwrap();
        document["padding"] = serde_json::json!("é".repeat(60000));
        let document = document.to_string();
        assert!(document.len() < MAX_BUCKET_JSON_BYTES as usize);
        let mut tx = store.backend.begin().await.unwrap();
        for index in 0..140 {
            q("INSERT INTO request_metrics_bucket(instance_id,bucket_seconds,bucket_start,observed_from,observed_to,counts_json) VALUES(?1,60,?2,?2,?3,?4)").bind(format!("budget-fixture-{index}")).bind(start).bind(start+60).bind(&document).execute_tx(&mut tx).await.unwrap();
        }
        tx.commit().await.unwrap();
        assert!(
            store
                .request_metrics_history(start, start + 60, 60)
                .await
                .unwrap_err()
                .to_string()
                .contains("byte budget")
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

#[cfg(test)]
mod precision_tests {
    use super::*;
    #[test]
    fn exact_integer_milliseconds_roundtrip_and_merge_without_old_bucket_edges() {
        let mut first = Counts::default();
        let mut second = Counts::default();
        for _ in 0..50 {
            first.record(200, 37);
            second.record(200, 83);
        }
        first.merge(&second);
        assert_eq!(first.percentile(50), Some(37));
        assert_eq!(first.percentile(95), Some(83));
        assert_eq!(first.percentile_resolution_ms(95), Some(1));
        assert!(!first.legacy_estimate());
        let json = serde_json::to_string(&first).unwrap();
        assert!(json.len() < 512);
        let decoded: Counts = serde_json::from_str(&json).unwrap();
        assert_eq!(first, decoded);
        assert!(decoded.valid());
        let mut aggregate = Counts::default();
        aggregate.merge(&decoded);
        assert_eq!(aggregate.percentile(95), Some(83));
    }
    #[test]
    fn legacy_and_mixed_rows_remain_explicit_coarse_estimates() {
        let legacy:Counts=serde_json::from_value(serde_json::json!({"requests":2,"errors5xx":0,"histogram":[0,0,0,0,0,1,1,0,0,0,0,0,0,0],"max_ms":83})).unwrap();
        assert!(legacy.valid());
        assert!(legacy.legacy_estimate());
        assert_eq!(legacy.percentile(50), Some(50));
        assert_eq!(legacy.percentile(95), Some(100));
        assert_eq!(legacy.percentile_resolution_ms(95), None);
        let mut mixed = Counts::default();
        mixed.record(200, 37);
        mixed.merge(&legacy);
        assert!(mixed.legacy_estimate());
        assert_eq!(mixed.percentile(95), Some(100));
        assert!(
            serde_json::from_str::<Counts>(&serde_json::to_string(&mixed).unwrap())
                .unwrap()
                .valid()
        );
    }
    #[test]
    fn precision_boundaries_and_overflow_keep_finite_memory_and_honest_resolution() {
        for (sample, expected, resolution) in [
            (0, 0, Some(1)),
            (1000, 1000, Some(1)),
            (1001, 1010, Some(10)),
            (10001, 10100, Some(100)),
            (60001, 61000, Some(1000)),
            (900001, 900001, None),
        ] {
            let mut counts = Counts::default();
            counts.record(200, sample);
            assert_eq!(counts.percentile(95), Some(expected));
            assert_eq!(counts.percentile_resolution_ms(95), resolution);
        }
        let mut full = Counts::default();
        for sample in PRECISE_LATENCY_BOUNDS_MS {
            full.record(200, sample);
        }
        full.record(200, u64::MAX);
        let value = serde_json::to_value(&full).unwrap();
        assert_eq!(
            value["fineHistogram"].as_array().unwrap().len(),
            histogram::BIN_COUNT
        );
        assert!(value.to_string().len() < MAX_BUCKET_JSON_BYTES as usize);
    }
    #[test]
    fn forged_or_unbounded_histogram_documents_are_refused() {
        let mut counts = Counts::default();
        counts.record(200, 37);
        for entries in [
            serde_json::json!([[37, 1], [37, 1]]),
            serde_json::json!([[65535, 1]]),
            serde_json::json!([[37, 0]]),
        ] {
            let mut value = serde_json::to_value(&counts).unwrap();
            value["fineHistogram"] = entries;
            assert!(serde_json::from_value::<Counts>(value).is_err());
        }
        let mut value = serde_json::to_value(&counts).unwrap();
        value["fineHistogram"] = serde_json::json!([[37, 2]]);
        assert!(!serde_json::from_value::<Counts>(value).unwrap().valid());
    }
}
