//! Durable, idempotent aggregate snapshots. No request body, address, identity or
//! query string is stored. Hourly buckets may carry a bounded list of normalised,
//! redacted not-found paths ([`NotFound`]); the server redacts them before they reach
//! this module, and [`Counts::valid`] refuses entries that are not in that form.
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
    /// Version-three sub-ms samples, in integer microseconds. Old rows omit this.
    #[serde(
        default,
        rename = "subMillisecondHistogram",
        skip_serializing_if = "Option::is_none"
    )]
    sub_millisecond: Option<std::collections::BTreeMap<u16, u64>>,
    /// 404 answers in this bucket. `None` for snapshots written before 404s were
    /// tracked: that period is "not tracked", never a measured zero.
    #[serde(default, rename = "notFound", skip_serializing_if = "Option::is_none")]
    pub not_found: Option<NotFound>,
}
/// At most this many not-found paths are kept per hourly bucket, and per answer.
pub const NOT_FOUND_PATHS_PER_BUCKET: usize = 50;
/// Upper bound, in bytes, of a stored normalised path (printable ASCII only).
pub const NOT_FOUND_PATH_MAX_BYTES: usize = 160;
/// 404 counts of one bucket and (hourly tier only) the most frequent normalised
/// `method + path` pairs, kept with the Space-Saving algorithm so a client sending
/// endless distinct paths cannot grow it: an entry's `count` is an upper bound and
/// `count - error` a lower bound of how often that pair was answered 404.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NotFound {
    pub count: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<NotFoundPath>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotFoundPath {
    pub method: String,
    pub path: String,
    pub count: u64,
    #[serde(default)]
    pub error: u64,
}
impl NotFound {
    /// Count one 404; with `capacity > 0` also keep `method path` among the top entries.
    pub fn record(&mut self, method: &str, path: &str, capacity: usize) {
        self.count = self.count.saturating_add(1);
        if capacity == 0 {
            return;
        }
        if let Some(entry) = self
            .paths
            .iter_mut()
            .find(|entry| entry.method == method && entry.path == path)
        {
            entry.count = entry.count.saturating_add(1);
            return;
        }
        if self.paths.len() < capacity {
            self.paths.push(NotFoundPath {
                method: method.into(),
                path: path.into(),
                count: 1,
                error: 0,
            });
            return;
        }
        // Space-Saving: the newcomer replaces the smallest entry and inherits its count as error.
        if let Some(smallest) = self.paths.iter_mut().min_by_key(|entry| entry.count) {
            let floor = smallest.count;
            *smallest = NotFoundPath {
                method: method.into(),
                path: path.into(),
                count: floor.saturating_add(1),
                error: floor,
            };
        }
    }
    fn floor(&self, capacity: usize) -> u64 {
        if self.paths.len() >= capacity {
            self.paths
                .iter()
                .map(|entry| entry.count)
                .min()
                .unwrap_or(0)
        } else {
            0
        }
    }
    /// Merge two summaries (another replica, another hour) and keep the top `capacity`.
    /// A pair absent from a full summary may have been counted there up to that
    /// summary's smallest count, which widens its bounds accordingly.
    pub fn merge(&mut self, other: &Self, capacity: usize) {
        let (ours, theirs) = (self.floor(capacity), other.floor(capacity));
        self.count = self.count.saturating_add(other.count);
        for entry in &mut self.paths {
            if !other
                .paths
                .iter()
                .any(|o| o.method == entry.method && o.path == entry.path)
            {
                entry.count = entry.count.saturating_add(theirs);
                entry.error = entry.error.saturating_add(theirs);
            }
        }
        for o in &other.paths {
            match self
                .paths
                .iter_mut()
                .find(|entry| entry.method == o.method && entry.path == o.path)
            {
                Some(entry) => {
                    entry.count = entry.count.saturating_add(o.count);
                    entry.error = entry.error.saturating_add(o.error);
                }
                None => self.paths.push(NotFoundPath {
                    method: o.method.clone(),
                    path: o.path.clone(),
                    count: o.count.saturating_add(ours),
                    error: o.error.saturating_add(ours),
                }),
            }
        }
        self.paths.sort_by(|a, b| {
            b.count
                .cmp(&a.count)
                .then_with(|| a.method.cmp(&b.method))
                .then_with(|| a.path.cmp(&b.path))
        });
        self.paths.truncate(capacity);
    }
    fn valid(&self, requests: u64) -> bool {
        let printable = |s: &str, max: usize| {
            !s.is_empty() && s.len() <= max && s.bytes().all(|b| (0x21..0x7f).contains(&b))
        };
        self.count <= requests
            && self.paths.len() <= NOT_FOUND_PATHS_PER_BUCKET
            && self.paths.iter().all(|entry| {
                printable(&entry.method, 16)
                    && entry.method.bytes().all(|b| b.is_ascii_uppercase())
                    && printable(&entry.path, NOT_FOUND_PATH_MAX_BYTES)
                    && entry.path.starts_with('/')
                    && !entry.path.contains(['?', '#', '@'])
                    && entry.count > 0
                    && entry.error < entry.count
                    && entry.count <= self.count
            })
    }
}
impl Default for Counts {
    fn default() -> Self {
        Self {
            requests: 0,
            errors5xx: 0,
            histogram: [0; 14],
            max_ms: 0,
            histogram_version: 3,
            fine_histogram: Some(histogram::Histogram::default()),
            sub_millisecond: Some(Default::default()),
            not_found: Some(NotFound::default()),
        }
    }
}
impl Counts {
    /// Count a 404 (after [`Counts::record_us`] for the same request); `capacity` is
    /// how many top paths this tier keeps (0 for minute buckets). An untracked legacy
    /// document stays untracked rather than starting from an invented zero.
    pub fn record_not_found(&mut self, method: &str, path: &str, capacity: usize) {
        if let Some(not_found) = &mut self.not_found {
            not_found.record(method, path, capacity);
        }
    }
    pub fn record(&mut self, status: u16, ms: u64) {
        self.record_sample(status, ms, None);
        // An integer-ms caller cannot establish sub-ms measurement precision.
        self.sub_millisecond = None;
        self.histogram_version = if self.fine_histogram.is_some() { 2 } else { 1 };
    }
    pub fn record_us(&mut self, status: u16, micros: u64) {
        self.record_sample(
            status,
            micros / 1000,
            (micros < 1000).then_some(micros as u16),
        );
    }
    fn record_sample(&mut self, status: u16, ms: u64, sub_ms: Option<u16>) {
        if self.requests == 0 {
            self.histogram_version = 3;
            self.fine_histogram = Some(histogram::Histogram::default());
            self.sub_millisecond = Some(Default::default());
        }
        if let (Some(values), Some(micros)) = (&mut self.sub_millisecond, sub_ms) {
            let count = values.entry(micros).or_default();
            *count = count.saturating_add(1);
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
            self.sub_millisecond = other.sub_millisecond.clone();
        } else if other.requests > 0 {
            match (&mut self.sub_millisecond, &other.sub_millisecond) {
                (Some(ours), Some(theirs)) => {
                    for (&micros, &count) in theirs {
                        let value = ours.entry(micros).or_default();
                        *value = value.saturating_add(count);
                    }
                }
                _ => {
                    self.sub_millisecond = None;
                    self.histogram_version = 2;
                }
            }
            match (&mut self.fine_histogram, &other.fine_histogram) {
                (Some(ours), Some(theirs)) => ours.merge(theirs),
                _ => {
                    self.fine_histogram = None;
                    self.sub_millisecond = None;
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
        // Untracked history on either side leaves the merged 404 count unknown.
        self.not_found = match (self.not_found.take(), &other.not_found) {
            (Some(mut ours), Some(theirs)) => {
                ours.merge(theirs, NOT_FOUND_PATHS_PER_BUCKET);
                Some(ours)
            }
            _ => None,
        };
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
    fn sub_ms_percentile(&self, percent: u64) -> Option<f64> {
        if self.requests == 0 {
            return None;
        }
        let rank = self
            .requests
            .saturating_mul(percent.clamp(1, 100))
            .div_ceil(100);
        let mut seen = 0_u64;
        for (&micros, &count) in self.sub_millisecond.as_ref()? {
            seen = seen.saturating_add(count);
            if seen >= rank {
                return Some(f64::from(micros) / 1000.0);
            }
        }
        None
    }
    /// Milliseconds on the wire; only measured v3 sub-ms bins produce fractions.
    pub fn precise_percentile_ms(&self, percent: u64) -> Option<f64> {
        self.sub_ms_percentile(percent)
            .or_else(|| self.percentile(percent).map(|value| value as f64))
    }
    pub fn precise_percentile_resolution_ms(&self, percent: u64) -> Option<f64> {
        if self.sub_ms_percentile(percent).is_some() {
            Some(0.001)
        } else {
            self.percentile_resolution_ms(percent)
                .map(|value| value as f64)
        }
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
    pub fn valid(&self) -> bool {
        self.not_found
            .as_ref()
            .is_none_or(|not_found| not_found.valid(self.requests))
            && self.errors5xx <= self.requests
            && self
                .histogram
                .iter()
                .try_fold(0_u64, |sum, n| sum.checked_add(*n))
                == Some(self.requests)
            && match (&self.fine_histogram, self.histogram_version) {
                (Some(fine), 2 | 3) => {
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
                    let sub_valid = match (&self.sub_millisecond, self.histogram_version) {
                        (Some(values), 3) => {
                            values.len() <= 1000
                                && values
                                    .iter()
                                    .all(|(&micros, &count)| micros < 1000 && count > 0)
                                && values
                                    .values()
                                    .try_fold(0_u64, |sum, count| sum.checked_add(*count))
                                    == Some(
                                        fine.entries()
                                            .find(|(index, _)| *index == 0)
                                            .map_or(0, |(_, count)| count),
                                    )
                        }
                        (None, 2) => true,
                        _ => false,
                    };
                    sub_valid
                        && fine.total() == Some(self.requests)
                        && fine.compatible_max(self.max_ms)
                        && projected == self.histogram
                }
                (None, 0 | 1) => self.sub_millisecond.is_none(),
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
        let mut fractional = Counts::default();
        for micros in [1, 123, 999, 1000] {
            fractional.record_us(200, micros);
        }
        let sample_time = 720_000_000;
        store
            .save_request_metrics(
                "fractional",
                &[Bucket {
                    seconds: 60,
                    start: sample_time,
                    observed_from: sample_time,
                    observed_to: sample_time + 60,
                    counts: fractional,
                }],
                sample_time + 60,
                30,
            )
            .await
            .unwrap();
        let rows = store
            .request_metrics_history(sample_time, sample_time + 60, 60)
            .await
            .unwrap();
        assert_eq!(rows[0].counts.precise_percentile_ms(50), Some(0.123));
        assert_eq!(
            rows[0].counts.precise_percentile_resolution_ms(50),
            Some(0.001)
        );
        not_found_checks(&store).await;
        byte_budget_checks(&store).await;
    }
    async fn not_found_checks(store: &Store) {
        let start = 800_000_000 / 3600 * 3600;
        let mut minute = Counts::default();
        let mut hour = Counts::default();
        for (status, path) in [(200, "/"), (404, "/wp-login.php"), (404, "/{token}")] {
            minute.record_us(status, 500);
            hour.record_us(status, 500);
            if status == 404 {
                minute.record_not_found("GET", path, 0);
                hour.record_not_found("GET", path, NOT_FOUND_PATHS_PER_BUCKET);
            }
        }
        let legacy: Counts = serde_json::from_value(serde_json::json!({"requests":1,"errors5xx":0,"histogram":[1,0,0,0,0,0,0,0,0,0,0,0,0,0],"max_ms":0})).unwrap();
        let buckets = [
            Bucket {
                seconds: 60,
                start,
                observed_from: start,
                observed_to: start + 60,
                counts: minute,
            },
            Bucket {
                seconds: 3600,
                start,
                observed_from: start,
                observed_to: start + 3600,
                counts: hour,
            },
        ];
        store
            .save_request_metrics("nf", &buckets, start + 3600, 30)
            .await
            .unwrap();
        store
            .save_request_metrics(
                "nf-legacy",
                &[Bucket {
                    seconds: 60,
                    start,
                    observed_from: start,
                    observed_to: start + 60,
                    counts: legacy,
                }],
                start + 3600,
                30,
            )
            .await
            .unwrap();
        let minutes = store
            .request_metrics_history(start, start + 60, 60)
            .await
            .unwrap();
        assert_eq!(minutes.len(), 2);
        let tracked = minutes.iter().find(|b| b.counts.requests == 3).unwrap();
        let nf = tracked.counts.not_found.as_ref().unwrap();
        assert_eq!(nf.count, 2);
        assert!(nf.paths.is_empty(), "minute buckets keep counts only");
        let untracked = minutes.iter().find(|b| b.counts.requests == 1).unwrap();
        assert!(
            untracked.counts.not_found.is_none(),
            "legacy rows stay untracked"
        );
        let hours = store
            .request_metrics_history(start, start + 3600, 3600)
            .await
            .unwrap();
        let nf = hours[0].counts.not_found.as_ref().unwrap();
        assert_eq!(nf.count, 2);
        assert_eq!(nf.paths.len(), 2);
        // A forged document carrying an unredacted query or address is refused.
        let mut forged = hours[0].counts.clone();
        forged.not_found.as_mut().unwrap().paths[0].path = "/x?token=1".into();
        assert!(!forged.valid());
        forged.not_found.as_mut().unwrap().paths[0].path = "/a@b.example".into();
        assert!(!forged.valid());
        // Hourly 404 detail follows the hourly retention.
        store
            .save_request_metrics("nf", &[], start + 31 * 86400, 30)
            .await
            .unwrap();
        assert!(
            store
                .request_metrics_history(start, start + 3600, 3600)
                .await
                .unwrap()
                .is_empty()
        );
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

#[cfg(test)]
mod sub_millisecond_tests {
    use super::*;
    #[test]
    fn microsecond_samples_roundtrip_with_bounded_resolution() {
        let mut counts = Counts::default();
        for micros in [1, 123, 999, 1000] {
            counts.record_us(200, micros);
        }
        assert!(counts.valid());
        for (percent, expected, resolution) in [
            (25, 0.001, 0.001),
            (50, 0.123, 0.001),
            (75, 0.999, 0.001),
            (100, 1.0, 1.0),
        ] {
            assert_eq!(counts.precise_percentile_ms(percent), Some(expected));
            assert_eq!(
                counts.precise_percentile_resolution_ms(percent),
                Some(resolution)
            );
        }
        let encoded = serde_json::to_string(&counts).unwrap();
        assert!(encoded.len() < MAX_BUCKET_JSON_BYTES as usize);
        let decoded: Counts = serde_json::from_str(&encoded).unwrap();
        assert_eq!(counts, decoded);
        assert!(decoded.valid());
        let mut merged = Counts::default();
        merged.merge(&decoded);
        merged.merge(&counts);
        assert_eq!(merged.precise_percentile_ms(50), Some(0.123));
        assert!(merged.valid());
    }
    #[test]
    fn mixing_integer_ms_history_never_invents_sub_ms_measurements() {
        let mut old = Counts::default();
        old.record(200, 0);
        let encoded = serde_json::to_value(&old).unwrap();
        assert_eq!(encoded["histogramVersion"], 2);
        assert!(encoded.get("subMillisecondHistogram").is_none());
        let old: Counts = serde_json::from_value(encoded).unwrap();
        assert!(old.valid());
        let mut precise = Counts::default();
        precise.record_us(200, 123);
        for reverse in [false, true] {
            let mut merged = if reverse {
                old.clone()
            } else {
                precise.clone()
            };
            merged.merge(if reverse { &precise } else { &old });
            assert!(merged.valid());
            assert_eq!(merged.precise_percentile_ms(95), Some(0.0));
            assert_eq!(merged.precise_percentile_resolution_ms(95), Some(1.0));
            assert!(merged.sub_millisecond.is_none());
        }
    }
    #[test]
    fn rejects_inconsistent_or_out_of_range_microsecond_bins() {
        let mut counts = Counts::default();
        counts.record_us(200, 123);
        let mut bad = serde_json::to_value(&counts).unwrap();
        bad["subMillisecondHistogram"] = serde_json::json!({"1000":1});
        assert!(!serde_json::from_value::<Counts>(bad).unwrap().valid());
        counts.sub_millisecond.as_mut().unwrap().insert(124, 1);
        assert!(!counts.valid());
        let mut full = Counts::default();
        for micros in 0..1000 {
            full.record_us(200, micros);
        }
        assert!(full.valid());
        assert!(serde_json::to_string(&full).unwrap().len() < MAX_BUCKET_JSON_BYTES as usize);
    }
}

#[cfg(test)]
mod not_found_tests {
    use super::*;
    #[test]
    fn top_paths_stay_bounded_under_ten_thousand_distinct_paths_and_keep_the_heavy_hitter() {
        let mut counts = Counts::default();
        for index in 0..10_000 {
            counts.record_us(404, 100);
            counts.record_not_found("GET", &format!("/scan/{index}"), NOT_FOUND_PATHS_PER_BUCKET);
            if index % 4 == 0 {
                counts.record_us(404, 100);
                counts.record_not_found("GET", "/wp-login.php", NOT_FOUND_PATHS_PER_BUCKET);
            }
        }
        let nf = counts.not_found.as_ref().unwrap();
        assert_eq!(nf.count, 12_500);
        assert_eq!(nf.paths.len(), NOT_FOUND_PATHS_PER_BUCKET);
        let hitter = nf.paths.iter().find(|p| p.path == "/wp-login.php").unwrap();
        // Space-Saving: the true count (2,500) lies within [count - error, count].
        assert!(hitter.count - hitter.error <= 2_500 && 2_500 <= hitter.count);
        assert!(counts.valid());
        let json = serde_json::to_string(&counts).unwrap();
        assert!(json.len() < MAX_BUCKET_JSON_BYTES as usize / 8);
        // Merging many full summaries still yields at most the cap.
        let mut merged = Counts::default();
        for _ in 0..30 {
            merged.merge(&counts);
        }
        let nf = merged.not_found.as_ref().unwrap();
        assert_eq!(nf.count, 30 * 12_500);
        assert_eq!(nf.paths.len(), NOT_FOUND_PATHS_PER_BUCKET);
        assert_eq!(nf.paths[0].path, "/wp-login.php");
        assert!(merged.valid());
    }
    #[test]
    fn merging_with_untracked_history_reports_unknown_not_zero() {
        let legacy: Counts = serde_json::from_value(serde_json::json!({"requests":1,"errors5xx":0,"histogram":[1,0,0,0,0,0,0,0,0,0,0,0,0,0],"max_ms":0})).unwrap();
        assert!(legacy.not_found.is_none());
        let mut tracked = Counts::default();
        tracked.record_us(404, 10);
        tracked.record_not_found("GET", "/x", 0);
        let mut merged = tracked.clone();
        merged.merge(&legacy);
        assert!(merged.not_found.is_none());
        let mut fresh = Counts::default();
        fresh.merge(&tracked);
        assert_eq!(fresh.not_found.unwrap().count, 1);
        // A legacy document never starts counting from an invented zero.
        let mut old = legacy.clone();
        old.record_not_found("GET", "/x", 0);
        assert!(old.not_found.is_none());
    }
    #[test]
    fn exact_counts_when_below_capacity_and_more_404s_than_requests_is_refused() {
        let mut counts = Counts::default();
        for path in ["/a", "/b", "/a"] {
            counts.record_us(404, 10);
            counts.record_not_found("GET", path, NOT_FOUND_PATHS_PER_BUCKET);
        }
        let nf = counts.not_found.as_ref().unwrap();
        assert_eq!(nf.paths.iter().find(|p| p.path == "/a").unwrap().count, 2);
        assert!(nf.paths.iter().all(|p| p.error == 0));
        let mut value = serde_json::to_value(&counts).unwrap();
        value["notFound"]["count"] = serde_json::json!(99);
        assert!(!serde_json::from_value::<Counts>(value).unwrap().valid());
    }
}
