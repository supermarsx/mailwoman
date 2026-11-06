//! Persisted performance aggregates (`metrics_bucket`).
//!
//! One row per instance, metric, time bucket and bounded dimension set holds a
//! count, a sum, a minimum, a maximum and a mergeable quantile sketch. The sketch
//! is a fixed log-linear bucket histogram (see [`Sketch`]): merging two sketches
//! adds their bucket counts, so the median and the 95th percentile of a range, or
//! of several instances, are read from the merged counts. A percentile is never
//! stored and never averaged.
//!
//! Resolutions: 60 s buckets, 3600 s buckets and 86400 s buckets. [`Store::rollup_metrics`]
//! folds old minute rows into hour rows and old hour rows into day rows (merging
//! sketches), then prunes day rows past their limit.
//!
//! Nothing here can name a person: the dimensions are fixed role classes (client
//! kind, device class, folder role class, backend type) and the mail domain, and
//! the recorder caps how many distinct dimension sets one metric can have in a
//! bucket.
use crate::{Store, StoreError, q};
use std::collections::BTreeMap;

pub const MINUTE: i64 = 60;
pub const HOUR: i64 = 3600;
pub const DAY: i64 = 86_400;
pub const RESOLUTIONS: [i64; 3] = [MINUTE, HOUR, DAY];
/// Upper bound on rows one read returns.
pub const MAX_READ_ROWS: usize = 250_000;
const MAX_METRIC_LEN: usize = 64;
const MAX_DIM_LEN: usize = 32;
const ROLL_BATCH: usize = 4000;

// ── sketch ──────────────────────────────────────────────────────────────────────

const SUB_BITS: u32 = 3;
const SUB: u64 = 1 << SUB_BITS;
const MAX_EXP: u32 = 26;
const MAX_VALUE: u64 = (1 << MAX_EXP) - 1;
/// Buckets in a sketch.
pub const BUCKETS: usize = (SUB + (MAX_EXP - SUB_BITS) as u64 * SUB) as usize;
/// Relative error bound of a reported quantile above the first eight values.
pub const RELATIVE_ERROR: f64 = 1.0 / 16.0;

/// Bucket holding a value: 0..8 each have their own, every power of two above is
/// split into 8 equal parts, up to 2^26; larger values share the last bucket.
pub fn bucket_of(v: u64) -> usize {
    if v < SUB {
        return v as usize;
    }
    let v = v.min(MAX_VALUE);
    let exp = 63 - v.leading_zeros();
    let sub = (v >> (exp - SUB_BITS)) & (SUB - 1);
    (SUB + u64::from(exp - SUB_BITS) * SUB + sub) as usize
}

/// `(lowest value, width)` of a bucket.
pub fn bucket_range(index: usize) -> (u64, u64) {
    let i = index as u64;
    if i < SUB {
        return (i, 1);
    }
    let k = i - SUB;
    let exp = (k / SUB) as u32 + SUB_BITS;
    let width = 1u64 << (exp - SUB_BITS);
    ((SUB + k % SUB) * width, width)
}

/// A mergeable quantile sketch: bucket counts only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sketch {
    counts: Vec<u64>,
}
impl Default for Sketch {
    fn default() -> Self {
        Self {
            counts: vec![0; BUCKETS],
        }
    }
}
impl Sketch {
    pub fn record(&mut self, v: u64) {
        self.counts[bucket_of(v)] += 1;
    }
    pub fn total(&self) -> u64 {
        self.counts.iter().fold(0u64, |t, c| t.saturating_add(*c))
    }
    pub fn merge(&mut self, other: &Self) {
        for (a, b) in self.counts.iter_mut().zip(&other.counts) {
            *a = a.saturating_add(*b);
        }
    }
    /// The count in bucket `index` (for callers that convert to their own layout).
    pub fn bucket_count(&self, index: usize) -> u64 {
        self.counts.get(index).copied().unwrap_or(0)
    }
    /// Quantile `q` (0..=1): the middle of the bucket holding the rank, never above
    /// `max`. `None` when the sketch is empty.
    pub fn quantile(&self, q: f64, max: u64) -> Option<f64> {
        let n = self.total();
        if n == 0 {
            return None;
        }
        let rank = ((q.clamp(0.0, 1.0) * n as f64).ceil() as u64).clamp(1, n);
        let mut seen = 0u64;
        for (i, c) in self.counts.iter().enumerate() {
            seen += c;
            if seen >= rank {
                let (lo, width) = bucket_range(i);
                let mid = lo as f64 + (width as f64 - 1.0) / 2.0;
                return Some(mid.min(max as f64));
            }
        }
        Some(max as f64)
    }
    /// Non-empty buckets as `(index varint, count varint)` pairs; empty when no
    /// bucket has a count.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for (i, c) in self.counts.iter().enumerate() {
            if *c > 0 {
                put_varint(&mut out, i as u64);
                put_varint(&mut out, *c);
            }
        }
        out
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, StoreError> {
        let mut s = Self::default();
        let mut at = 0;
        while at < bytes.len() {
            let i = get_varint(bytes, &mut at)? as usize;
            let c = get_varint(bytes, &mut at)?;
            if i >= BUCKETS {
                return Err(StoreError::Corrupt("sketch bucket out of range".into()));
            }
            s.counts[i] = s.counts[i].saturating_add(c);
        }
        Ok(s)
    }
}

fn put_varint(out: &mut Vec<u8>, mut v: u64) {
    while v >= 0x80 {
        out.push((v as u8 & 0x7f) | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}
fn get_varint(b: &[u8], at: &mut usize) -> Result<u64, StoreError> {
    let (mut v, mut shift) = (0u64, 0u32);
    loop {
        let byte = *b
            .get(*at)
            .ok_or_else(|| StoreError::Corrupt("truncated sketch".into()))?;
        *at += 1;
        if shift > 63 {
            return Err(StoreError::Corrupt("sketch varint too long".into()));
        }
        v |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(v);
        }
        shift += 7;
    }
}

// ── aggregate and row ───────────────────────────────────────────────────────────

/// What one bucket holds. For a counter metric `count` is the sum of increments and
/// the sketch stays empty; for a value metric `count` is the number of samples.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Agg {
    pub count: u64,
    pub sum: u64,
    pub min: u64,
    pub max: u64,
    pub sketch: Sketch,
}
impl Agg {
    pub fn record(&mut self, v: u64) {
        if self.count == 0 {
            self.min = v;
            self.max = v;
        } else {
            self.min = self.min.min(v);
            self.max = self.max.max(v);
        }
        self.count += 1;
        self.sum = self.sum.saturating_add(v);
        self.sketch.record(v);
    }
    pub fn add_count(&mut self, n: u64) {
        self.count = self.count.saturating_add(n);
        self.sum = self.sum.saturating_add(n);
    }
    pub fn merge(&mut self, other: &Self) {
        if other.count == 0 {
            return;
        }
        if self.count == 0 {
            self.min = other.min;
            self.max = other.max;
        } else {
            self.min = self.min.min(other.min);
            self.max = self.max.max(other.max);
        }
        self.count = self.count.saturating_add(other.count);
        self.sum = self.sum.saturating_add(other.sum);
        self.sketch.merge(&other.sketch);
    }
    pub fn mean(&self) -> Option<f64> {
        (self.count > 0).then(|| self.sum as f64 / self.count as f64)
    }
    pub fn quantile(&self, q: f64) -> Option<f64> {
        self.sketch.quantile(q, self.max)
    }
    /// A value aggregate carries a sketch whose total equals the count; a counter
    /// carries none.
    pub fn valid(&self) -> bool {
        let t = self.sketch.total();
        (t == 0 || t == self.count) && self.min <= self.max
    }
}

/// The bounded dimensions of a row. An empty string means "not applicable".
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dims {
    pub domain: String,
    pub folder_class: String,
    pub client_kind: String,
    pub device_class: String,
    pub backend: String,
}
impl Dims {
    fn valid(&self) -> bool {
        [
            &self.domain,
            &self.folder_class,
            &self.client_kind,
            &self.device_class,
            &self.backend,
        ]
        .iter()
        .all(|d| d.len() <= MAX_DIM_LEN && d.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetricRow {
    pub instance: String,
    pub metric: String,
    pub start: i64,
    pub resolution: i64,
    pub dims: Dims,
    pub agg: Agg,
}
impl MetricRow {
    fn check(&self) -> Result<(), StoreError> {
        let bad = |why: &str| Err(StoreError::Corrupt(format!("invalid metric row: {why}")));
        if self.instance.is_empty() || self.instance.len() > 64 {
            return bad("instance");
        }
        if self.metric.is_empty()
            || self.metric.len() > MAX_METRIC_LEN
            || !self.metric.bytes().all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b))
        {
            return bad("metric");
        }
        if !RESOLUTIONS.contains(&self.resolution) || self.start < 0 || self.start % self.resolution != 0 {
            return bad("bucket");
        }
        if !self.dims.valid() {
            return bad("dimension");
        }
        if !self.agg.valid() {
            return bad("aggregate");
        }
        Ok(())
    }
}

/// What a rollup did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RollupReport {
    pub minutes_folded: u64,
    pub hours_folded: u64,
    pub days_pruned: u64,
}
impl RollupReport {
    pub fn total(&self) -> u64 {
        self.minutes_folded + self.hours_folded + self.days_pruned
    }
}

/// How much is stored.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MetricsStats {
    pub rows: u64,
    pub first_bucket: Option<i64>,
    pub last_flush: Option<i64>,
    pub instances: u64,
}

const COLUMNS: &str = "instance_id,metric,bucket_start,resolution_seconds,dim_domain,dim_folder_class,dim_client_kind,dim_device_class,dim_backend,sample_count,sum_value,min_value,max_value,sketch";

fn to_row(r: &crate::backend::Row) -> Result<MetricRow, StoreError> {
    let agg = Agg {
        count: r.get_i64("sample_count").max(0) as u64,
        sum: r.get_i64("sum_value").max(0) as u64,
        min: r.get_i64("min_value").max(0) as u64,
        max: r.get_i64("max_value").max(0) as u64,
        sketch: Sketch::decode(&r.get_blob("sketch"))?,
    };
    if !agg.valid() {
        return Err(StoreError::Corrupt("invalid stored metric aggregate".into()));
    }
    Ok(MetricRow {
        instance: r.get_string("instance_id"),
        metric: r.get_string("metric"),
        start: r.get_i64("bucket_start"),
        resolution: r.get_i64("resolution_seconds"),
        dims: Dims {
            domain: r.get_string("dim_domain"),
            folder_class: r.get_string("dim_folder_class"),
            client_kind: r.get_string("dim_client_kind"),
            device_class: r.get_string("dim_device_class"),
            backend: r.get_string("dim_backend"),
        },
        agg,
    })
}

fn check_prefix(prefix: &str) -> Result<String, StoreError> {
    if prefix.len() > MAX_METRIC_LEN || !prefix.bytes().all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b)) {
        return Err(StoreError::Corrupt("invalid metric prefix".into()));
    }
    Ok(format!("{prefix}%"))
}

impl Store {
    /// Absolute snapshots: a row is replaced only by one with more samples, so a
    /// retry after an uncertain commit never counts twice.
    pub async fn save_metric_rows(&self, rows: &[MetricRow], now: i64) -> Result<(), StoreError> {
        if rows.len() > 5000 {
            return Err(StoreError::Corrupt("too many metric rows in one flush".into()));
        }
        for r in rows {
            r.check()?;
        }
        let mut tx = self.backend.begin().await?;
        for r in rows {
            upsert(&mut tx, r, now, false).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Rows overlapping `[from, to)` whose metric starts with `prefix`.
    pub async fn load_metric_rows(&self, prefix: &str, from: i64, to: i64) -> Result<Vec<MetricRow>, StoreError> {
        if to <= from {
            return Ok(Vec::new());
        }
        let pattern = check_prefix(prefix)?;
        let rows = q("SELECT instance_id,metric,bucket_start,resolution_seconds,dim_domain,dim_folder_class,dim_client_kind,dim_device_class,dim_backend,sample_count,sum_value,min_value,max_value,sketch FROM metrics_bucket WHERE metric LIKE ?1 AND bucket_start<?3 AND bucket_start+resolution_seconds>?2 ORDER BY bucket_start,metric,instance_id LIMIT 250001")
            .bind(pattern)
            .bind(from)
            .bind(to)
            .fetch_all(&self.backend)
            .await?;
        if rows.len() > MAX_READ_ROWS {
            return Err(StoreError::Corrupt("metric range exceeds the bounded scan; choose a shorter range".into()));
        }
        rows.iter().map(to_row).collect()
    }

    pub async fn metrics_stats(&self, prefix: &str) -> Result<MetricsStats, StoreError> {
        let pattern = check_prefix(prefix)?;
        let rows = q("SELECT COUNT(*) AS n,MIN(bucket_start) AS first_start,MAX(flushed_at) AS last_flush,COUNT(DISTINCT instance_id) AS instances FROM metrics_bucket WHERE metric LIKE ?1")
            .bind(pattern)
            .fetch_all(&self.backend)
            .await?;
        let Some(r) = rows.first() else {
            return Ok(MetricsStats::default());
        };
        Ok(MetricsStats {
            rows: r.get_i64("n").max(0) as u64,
            first_bucket: r.get_opt_i64("first_start"),
            last_flush: r.get_opt_i64("last_flush"),
            instances: r.get_i64("instances").max(0) as u64,
        })
    }

    /// Remove every stored bucket of metrics starting with `prefix`.
    pub async fn reset_metrics(&self, prefix: &str) -> Result<u64, StoreError> {
        let pattern = check_prefix(prefix)?;
        Ok(q("DELETE FROM metrics_bucket WHERE metric LIKE ?1")
            .bind(pattern)
            .execute(&self.backend)
            .await?)
    }

    /// Fold minute rows older than `minute_hours` into hour rows and hour rows older
    /// than `hour_days` into day rows, merging sketches; delete day rows older than
    /// `day_days` (0 keeps them). Idempotent; each batch is one transaction.
    pub async fn rollup_metrics(
        &self,
        now: i64,
        minute_hours: i64,
        hour_days: i64,
        day_days: i64,
    ) -> Result<RollupReport, StoreError> {
        let mut report = RollupReport::default();
        let minute_cut = now.saturating_sub(minute_hours.max(1) * HOUR) / HOUR * HOUR;
        let hour_cut = now.saturating_sub(hour_days.max(1) * DAY) / DAY * DAY;
        report.minutes_folded = self.fold(MINUTE, HOUR, minute_cut, now).await?;
        report.hours_folded = self.fold(HOUR, DAY, hour_cut, now).await?;
        if day_days > 0 {
            let cut = now.saturating_sub(day_days * DAY);
            report.days_pruned = q("DELETE FROM metrics_bucket WHERE resolution_seconds=?1 AND bucket_start+resolution_seconds<=?2")
                .bind(DAY)
                .bind(cut)
                .execute(&self.backend)
                .await?;
        }
        Ok(report)
    }

    async fn fold(&self, from_res: i64, to_res: i64, cutoff: i64, now: i64) -> Result<u64, StoreError> {
        let mut folded = 0u64;
        loop {
            let mut tx = self.backend.begin().await?;
            let found = q("SELECT instance_id,metric,bucket_start,resolution_seconds,dim_domain,dim_folder_class,dim_client_kind,dim_device_class,dim_backend,sample_count,sum_value,min_value,max_value,sketch FROM metrics_bucket WHERE resolution_seconds=?1 AND bucket_start<?2 ORDER BY bucket_start LIMIT 4000")
                .bind(from_res)
                .bind(cutoff)
                .fetch_all_tx(&mut tx)
                .await?;
            if found.is_empty() {
                drop(tx);
                return Ok(folded);
            }
            let rows: Vec<MetricRow> = found.iter().map(to_row).collect::<Result<_, _>>()?;
            let mut groups: BTreeMap<(String, String, i64, Dims), Agg> = BTreeMap::new();
            for r in &rows {
                groups
                    .entry((r.instance.clone(), r.metric.clone(), r.start / to_res * to_res, r.dims.clone()))
                    .or_default()
                    .merge(&r.agg);
            }
            for ((instance, metric, start, dims), agg) in groups {
                let target = MetricRow { instance, metric, start, resolution: to_res, dims, agg };
                upsert(&mut tx, &target, now, true).await?;
            }
            for r in &rows {
                q("DELETE FROM metrics_bucket WHERE instance_id=?1 AND metric=?2 AND bucket_start=?3 AND resolution_seconds=?4 AND dim_domain=?5 AND dim_folder_class=?6 AND dim_client_kind=?7 AND dim_device_class=?8 AND dim_backend=?9")
                    .bind(r.instance.as_str())
                    .bind(r.metric.as_str())
                    .bind(r.start)
                    .bind(r.resolution)
                    .bind(r.dims.domain.as_str())
                    .bind(r.dims.folder_class.as_str())
                    .bind(r.dims.client_kind.as_str())
                    .bind(r.dims.device_class.as_str())
                    .bind(r.dims.backend.as_str())
                    .execute_tx(&mut tx)
                    .await?;
            }
            folded += rows.len() as u64;
            tx.commit().await?;
            if rows.len() < ROLL_BATCH {
                return Ok(folded);
            }
        }
    }
}

/// Insert or update one row. `merge` adds to an existing row (rollup); otherwise the
/// row is an absolute snapshot that only replaces a smaller one.
async fn upsert(tx: &mut crate::backend::Tx, r: &MetricRow, now: i64, merge: bool) -> Result<(), StoreError> {
    let mut agg = r.agg.clone();
    if merge {
        let existing = q("SELECT instance_id,metric,bucket_start,resolution_seconds,dim_domain,dim_folder_class,dim_client_kind,dim_device_class,dim_backend,sample_count,sum_value,min_value,max_value,sketch FROM metrics_bucket WHERE instance_id=?1 AND metric=?2 AND bucket_start=?3 AND resolution_seconds=?4 AND dim_domain=?5 AND dim_folder_class=?6 AND dim_client_kind=?7 AND dim_device_class=?8 AND dim_backend=?9")
            .bind(r.instance.as_str())
            .bind(r.metric.as_str())
            .bind(r.start)
            .bind(r.resolution)
            .bind(r.dims.domain.as_str())
            .bind(r.dims.folder_class.as_str())
            .bind(r.dims.client_kind.as_str())
            .bind(r.dims.device_class.as_str())
            .bind(r.dims.backend.as_str())
            .fetch_all_tx(tx)
            .await?;
        if let Some(row) = existing.first() {
            agg.merge(&to_row(row)?.agg);
        }
    }
    let sql: &'static str = if merge {
        "INSERT INTO metrics_bucket(instance_id,metric,bucket_start,resolution_seconds,dim_domain,dim_folder_class,dim_client_kind,dim_device_class,dim_backend,sample_count,sum_value,min_value,max_value,sketch,flushed_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15) ON CONFLICT(instance_id,metric,bucket_start,resolution_seconds,dim_domain,dim_folder_class,dim_client_kind,dim_device_class,dim_backend) DO UPDATE SET sample_count=excluded.sample_count,sum_value=excluded.sum_value,min_value=excluded.min_value,max_value=excluded.max_value,sketch=excluded.sketch,flushed_at=excluded.flushed_at"
    } else {
        "INSERT INTO metrics_bucket(instance_id,metric,bucket_start,resolution_seconds,dim_domain,dim_folder_class,dim_client_kind,dim_device_class,dim_backend,sample_count,sum_value,min_value,max_value,sketch,flushed_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15) ON CONFLICT(instance_id,metric,bucket_start,resolution_seconds,dim_domain,dim_folder_class,dim_client_kind,dim_device_class,dim_backend) DO UPDATE SET sample_count=excluded.sample_count,sum_value=excluded.sum_value,min_value=excluded.min_value,max_value=excluded.max_value,sketch=excluded.sketch,flushed_at=excluded.flushed_at WHERE excluded.sample_count>metrics_bucket.sample_count"
    };
    q(sql)
        .bind(r.instance.as_str())
        .bind(r.metric.as_str())
        .bind(r.start)
        .bind(r.resolution)
        .bind(r.dims.domain.as_str())
        .bind(r.dims.folder_class.as_str())
        .bind(r.dims.client_kind.as_str())
        .bind(r.dims.device_class.as_str())
        .bind(r.dims.backend.as_str())
        .bind(agg.count.min(i64::MAX as u64) as i64)
        .bind(agg.sum.min(i64::MAX as u64) as i64)
        .bind(agg.min.min(i64::MAX as u64) as i64)
        .bind(agg.max.min(i64::MAX as u64) as i64)
        .bind(agg.sketch.encode())
        .bind(now)
        .execute_tx(tx)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sketch_of(values: impl IntoIterator<Item = u64>) -> (Agg, Vec<u64>) {
        let mut a = Agg::default();
        let mut all = Vec::new();
        for v in values {
            a.record(v);
            all.push(v);
        }
        (a, all)
    }
    fn truth(mut v: Vec<u64>, q: f64) -> f64 {
        v.sort_unstable();
        v[((q * v.len() as f64).ceil() as usize).clamp(1, v.len()) - 1] as f64
    }

    #[test]
    fn buckets_are_ordered_and_cover_their_values() {
        let mut last = 0;
        for v in (0..200_000u64).chain([MAX_VALUE, MAX_VALUE + 5]) {
            let b = bucket_of(v);
            assert!(b >= last && b < BUCKETS);
            last = b;
            if v <= MAX_VALUE {
                let (lo, w) = bucket_range(b);
                assert!(v >= lo && v < lo + w, "{v} in {b}");
            }
        }
    }

    #[test]
    fn sketch_round_trips_and_stays_compact() {
        let (a, _) = sketch_of((1..5000).map(|i| i * 37 % 90_000));
        let bytes = a.sketch.encode();
        assert!(bytes.len() < 2 * BUCKETS * 3);
        assert_eq!(Sketch::decode(&bytes).unwrap(), a.sketch);
        assert!(Sketch::decode(&[0x80]).is_err());
        assert!(Sketch::decode(&[0xff, 0x7f, 1]).is_err());
        assert_eq!(Sketch::default().encode(), Vec::<u8>::new());
    }

    #[test]
    fn merging_two_buckets_equals_the_sketch_of_the_union() {
        let (mut a, va) = sketch_of((0..3000).map(|i| 20 + i * 7 % 4000));
        let (b, vb) = sketch_of((0..2000).map(|i| 300 + i * 13 % 90_000));
        let (union, vu) = sketch_of(va.iter().chain(&vb).copied());
        a.merge(&b);
        assert_eq!(a, union);
        for q in [0.5, 0.95, 0.99] {
            let got = a.quantile(q).unwrap();
            let t = truth(vu.clone(), q);
            assert!((got - t).abs() / t <= RELATIVE_ERROR + 0.01, "q{q}: {got} vs {t}");
        }
        // Percentiles are never averaged: the median of the union is not the mean of medians.
        let mid = (sketch_of(va.clone()).0.quantile(0.5).unwrap() + sketch_of(vb.clone()).0.quantile(0.5).unwrap()) / 2.0;
        assert!((a.quantile(0.5).unwrap() - mid).abs() > 1.0);
    }

    #[test]
    fn an_empty_aggregate_has_no_quantile_and_merges_as_identity() {
        let mut a = Agg::default();
        assert!(a.quantile(0.5).is_none() && a.mean().is_none());
        let (b, _) = sketch_of([5, 900]);
        a.merge(&b);
        assert_eq!((a.count, a.min, a.max, a.sum), (2, 5, 900, 905));
        a.merge(&Agg::default());
        assert_eq!(a, b);
    }

    fn row(instance: &str, metric: &str, start: i64, resolution: i64, values: &[u64]) -> MetricRow {
        let mut agg = Agg::default();
        for v in values {
            agg.record(*v);
        }
        MetricRow {
            instance: instance.into(),
            metric: metric.into(),
            start,
            resolution,
            dims: Dims { client_kind: "web".into(), device_class: "desktop".into(), ..Dims::default() },
            agg,
        }
    }

    async fn store() -> Store {
        Store::open_in_memory(crate::ServerKey::generate()).await.unwrap()
    }

    #[tokio::test]
    async fn rows_round_trip_are_idempotent_and_validated() {
        let s = store().await;
        let r = row("a", "imageProxy.outcome.miss", 600, MINUTE, &[100, 200, 4000]);
        for _ in 0..2 {
            s.save_metric_rows(std::slice::from_ref(&r), 700).await.unwrap();
        }
        let got = s.load_metric_rows("imageProxy.", 0, 1000).await.unwrap();
        assert_eq!(got, vec![r.clone()]);
        // A smaller snapshot never replaces a larger one.
        let smaller = row("a", "imageProxy.outcome.miss", 600, MINUTE, &[100]);
        s.save_metric_rows(&[smaller], 710).await.unwrap();
        assert_eq!(s.load_metric_rows("imageProxy.", 0, 1000).await.unwrap()[0].agg.count, 3);
        assert!(s.load_metric_rows("clientTiming.", 0, 1000).await.unwrap().is_empty());
        for bad in [
            MetricRow { metric: "bad metric".into(), ..r.clone() },
            MetricRow { start: 601, ..r.clone() },
            MetricRow { resolution: 5, ..r.clone() },
            MetricRow { dims: Dims { domain: "a b".into(), ..Dims::default() }, ..r.clone() },
            MetricRow { instance: String::new(), ..r.clone() },
        ] {
            assert!(s.save_metric_rows(&[bad], 0).await.is_err());
        }
        let stats = s.metrics_stats("imageProxy.").await.unwrap();
        assert_eq!((stats.rows, stats.first_bucket, stats.last_flush, stats.instances), (1, Some(600), Some(710), 1));
        assert_eq!(s.reset_metrics("imageProxy.").await.unwrap(), 1);
        assert_eq!(s.metrics_stats("imageProxy.").await.unwrap().rows, 0);
    }

    #[tokio::test]
    async fn rollup_folds_minutes_into_hours_hours_into_days_and_prunes_days() {
        let s = store().await;
        let day0 = 100 * DAY;
        // Two instances, 30 minute rows each within one hour of day 0.
        let mut expect = Agg::default();
        for inst in ["a", "b"] {
            let rows: Vec<MetricRow> = (0..30)
                .map(|m| row(inst, "clientTiming.load", day0 + 5 * HOUR + m * MINUTE, MINUTE, &[100 + m as u64, 5000]))
                .collect();
            for r in &rows {
                expect.merge(&r.agg);
            }
            s.save_metric_rows(&rows, day0).await.unwrap();
        }
        // A recent minute row stays.
        s.save_metric_rows(&[row("a", "clientTiming.load", day0 + 4 * DAY, MINUTE, &[7])], day0).await.unwrap();
        let now = day0 + 4 * DAY + 10 * MINUTE;
        let r1 = s.rollup_metrics(now, 48, 400, 400).await.unwrap();
        assert_eq!(r1.minutes_folded, 60);
        let rows = s.load_metric_rows("clientTiming.", 0, now + DAY).await.unwrap();
        assert_eq!(rows.iter().filter(|r| r.resolution == HOUR).count(), 2, "one hour row per instance");
        assert_eq!(rows.iter().filter(|r| r.resolution == MINUTE).count(), 1);
        let mut merged = Agg::default();
        for r in rows.iter().filter(|r| r.resolution == HOUR) {
            merged.merge(&r.agg);
        }
        assert_eq!(merged, expect, "hour rows hold exactly the merged minute sketches");
        // Hours older than hour_days become one day row per instance; running again changes nothing.
        let r2 = s.rollup_metrics(now, 48, 2, 400).await.unwrap();
        assert_eq!(r2.hours_folded, 2);
        assert_eq!(s.rollup_metrics(now, 48, 2, 400).await.unwrap().hours_folded, 0);
        let rows = s.load_metric_rows("clientTiming.", 0, now + DAY).await.unwrap();
        assert_eq!(rows.iter().filter(|r| r.resolution == DAY).count(), 2);
        // Day rows past dayBucketsDays go; 0 keeps them.
        assert_eq!(s.rollup_metrics(now, 48, 2, 0).await.unwrap().days_pruned, 0);
        assert_eq!(s.rollup_metrics(now, 48, 2, 3).await.unwrap().days_pruned, 2);
        let left = s.load_metric_rows("clientTiming.", 0, now + DAY).await.unwrap();
        assert_eq!(left.len(), 1);
    }
}
