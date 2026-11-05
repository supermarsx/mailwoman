//! Aggregate browser load-time histograms (`client_timing_bucket`).
//!
//! One row per instance, hour, metric and device class holds a count, a sum and a
//! fixed log-scale histogram. No session, account, address, path or user agent
//! reaches this module, and nothing here can be joined back to a person.
//!
//! Resolution: 48 bins. Bin 0 is `[0, 10)` ms; bins 1..=47 are geometric between
//! 10 ms and 120 000 ms with ratio `12000^(1/47)` (about 1.221). A percentile is
//! found by rank in the merged histogram and interpolated geometrically inside its
//! bin, so it is within one bin of the true value: at most about 22% relative error
//! worst case, typically under 10%. The sum, and so the mean, is exact.
use crate::{Store, StoreError, q};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub const BINS: usize = 48;
pub const MIN_MS: u64 = 10;
pub const MAX_MS: u64 = 120_000;
/// Page-load metrics, then interaction metrics. `folderOpen` is the first
/// release's single folder figure; it is still accepted and shown, no longer sent.
/// The `Cold`/`Cached` pairs split a folder or message load by whether it was
/// served from the browser's own cache (cached or prefetched) or waited on the
/// network. No folder or message identity is part of the key.
pub const METRICS: [&str; 11] = [
    "timeToUsable",
    "ttfb",
    "dcl",
    "load",
    "fcp",
    "lcp",
    "folderOpen",
    "folderLoadCold",
    "folderLoadCached",
    "messageLoadCold",
    "messageLoadCached",
];
pub const DEVICES: [&str; 3] = ["desktop", "mobile", "unknown"];
/// Upper bound on rows returned for one range.
pub const MAX_HISTORY_ROWS: usize = 50_000;

/// `edges()[j]` is `10 * r^j` for j in 0..=46, closed by 120 000 at index 47.
fn edges() -> &'static [f64; BINS] {
    static EDGES: OnceLock<[f64; BINS]> = OnceLock::new();
    EDGES.get_or_init(|| {
        let ratio = (MAX_MS as f64 / MIN_MS as f64).powf(1.0 / (BINS - 1) as f64);
        let mut e = [0.0; BINS];
        for (j, edge) in e.iter_mut().enumerate() {
            *edge = MIN_MS as f64 * ratio.powi(j as i32);
        }
        e[BINS - 1] = MAX_MS as f64;
        e
    })
}

/// Bin index of a sample in milliseconds (clamped to `0..=MAX_MS`).
pub fn bin_of(ms: u64) -> usize {
    let ms = ms.min(MAX_MS) as f64;
    if ms < MIN_MS as f64 {
        return 0;
    }
    // Bin j (1..=47) covers [edges[j-1], edges[j]); the last also holds MAX_MS.
    let e = edges();
    e.partition_point(|edge| *edge <= ms).min(BINS - 1)
}

/// Low and high edge of a bin, in milliseconds.
pub fn bin_range(bin: usize) -> (f64, f64) {
    let e = edges();
    if bin == 0 {
        (0.0, MIN_MS as f64)
    } else {
        (e[bin - 1], e[bin.min(BINS - 1)])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hist {
    pub count: u64,
    pub sum_ms: u64,
    pub bins: Vec<u64>,
}
impl Default for Hist {
    fn default() -> Self {
        Self {
            count: 0,
            sum_ms: 0,
            bins: vec![0; BINS],
        }
    }
}
impl Hist {
    pub fn record(&mut self, ms: u64) {
        let ms = ms.min(MAX_MS);
        self.count += 1;
        self.sum_ms += ms;
        self.bins[bin_of(ms)] += 1;
    }
    pub fn merge(&mut self, other: &Self) {
        self.count = self.count.saturating_add(other.count);
        self.sum_ms = self.sum_ms.saturating_add(other.sum_ms);
        for (a, b) in self.bins.iter_mut().zip(&other.bins) {
            *a = a.saturating_add(*b);
        }
    }
    pub fn valid(&self) -> bool {
        self.bins.len() == BINS
            && self.bins.iter().try_fold(0u64, |t, b| t.checked_add(*b)) == Some(self.count)
            && self.sum_ms <= self.count.saturating_mul(MAX_MS)
    }
    pub fn mean_ms(&self) -> Option<f64> {
        (self.count > 0).then(|| self.sum_ms as f64 / self.count as f64)
    }
    /// Percentile `percent` (1..=100) in milliseconds, or `None` when empty.
    pub fn percentile_ms(&self, percent: u32) -> Option<f64> {
        if self.count == 0 || !(1..=100).contains(&percent) {
            return None;
        }
        let rank = (self.count * u64::from(percent)).div_ceil(100).max(1);
        let mut before = 0u64;
        for (bin, n) in self.bins.iter().enumerate() {
            if *n > 0 && before + n >= rank {
                let (lo, hi) = bin_range(bin);
                let frac = (rank - before) as f64 / *n as f64;
                return Some(if bin == 0 {
                    lo + (hi - lo) * frac
                } else {
                    lo * (hi / lo).powf(frac)
                });
            }
            before += n;
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimingBucket {
    pub start: i64,
    pub metric: String,
    pub device: String,
    pub hist: Hist,
}

impl Store {
    /// Absolute snapshots: retrying after an uncertain commit never adds twice.
    pub async fn save_client_timing(
        &self,
        instance: &str,
        buckets: &[TimingBucket],
        now: i64,
        retention_days: i64,
    ) -> Result<(), StoreError> {
        if instance.is_empty()
            || instance.len() > 64
            || buckets.len() > 1024
            || !(1..=400).contains(&retention_days)
        {
            return Err(StoreError::Corrupt("invalid timing snapshot bounds".into()));
        }
        let mut tx = self.backend.begin().await?;
        for b in buckets {
            if b.start < 0
                || b.start % 3600 != 0
                || !METRICS.contains(&b.metric.as_str())
                || !DEVICES.contains(&b.device.as_str())
                || !b.hist.valid()
            {
                return Err(StoreError::Corrupt("invalid timing bucket".into()));
            }
            let bins = serde_json::to_string(&b.hist.bins)
                .map_err(|e| StoreError::Corrupt(e.to_string()))?;
            q("INSERT INTO client_timing_bucket(instance_id,bucket_start,metric,device,sample_count,sum_ms,histogram_json) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(instance_id,bucket_start,metric,device) DO UPDATE SET sample_count=excluded.sample_count,sum_ms=excluded.sum_ms,histogram_json=excluded.histogram_json WHERE excluded.sample_count>client_timing_bucket.sample_count")
                .bind(instance).bind(b.start).bind(b.metric.as_str()).bind(b.device.as_str())
                .bind(b.hist.count as i64).bind(b.hist.sum_ms as i64).bind(bins)
                .execute_tx(&mut tx).await?;
        }
        let cutoff = now.saturating_sub(retention_days * 86400);
        q("DELETE FROM client_timing_bucket WHERE bucket_start<=?1-3600")
            .bind(cutoff)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Every replica's hourly rows with `from <= start < to`.
    pub async fn client_timing_history(
        &self,
        from: i64,
        to: i64,
    ) -> Result<Vec<TimingBucket>, StoreError> {
        if from < 0 || to <= from || to - from > 401 * 86400 {
            return Err(StoreError::Corrupt("invalid timing range".into()));
        }
        let rows = q("SELECT bucket_start,metric,device,sample_count,sum_ms,histogram_json FROM client_timing_bucket WHERE bucket_start>=?1 AND bucket_start<?2 ORDER BY bucket_start,metric,device,instance_id LIMIT 50001")
            .bind(from)
            .bind(to)
            .fetch_all(&self.backend)
            .await?;
        if rows.len() > MAX_HISTORY_ROWS {
            return Err(StoreError::Corrupt(
                "timing range exceeds bounded history scan; choose a shorter range".into(),
            ));
        }
        rows.into_iter()
            .map(|row| {
                let bins: Vec<u64> = serde_json::from_str(&row.get_string("histogram_json"))
                    .map_err(|e| StoreError::Corrupt(e.to_string()))?;
                let hist = Hist {
                    count: row.get_i64("sample_count").max(0) as u64,
                    sum_ms: row.get_i64("sum_ms").max(0) as u64,
                    bins,
                };
                if !hist.valid() {
                    return Err(StoreError::Corrupt(
                        "invalid stored timing histogram".into(),
                    ));
                }
                Ok(TimingBucket {
                    start: row.get_i64("bucket_start"),
                    metric: row.get_string("metric"),
                    device: row.get_string("device"),
                    hist,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bin_placement_is_monotonic_and_clamped() {
        assert_eq!(bin_of(0), 0);
        assert_eq!(bin_of(9), 0);
        assert_eq!(bin_of(10), 1);
        assert_eq!(bin_of(MAX_MS), BINS - 1);
        assert_eq!(bin_of(10_000_000), BINS - 1);
        let mut last = 0;
        for ms in 0..=MAX_MS {
            let b = bin_of(ms);
            assert!(b >= last && b < BINS);
            let (lo, hi) = bin_range(b);
            assert!(
                ms as f64 >= lo - 1e-9 && (ms as f64) <= hi + 1e-9,
                "{ms} in {b}"
            );
            last = b;
        }
    }

    #[test]
    fn percentiles_of_a_known_distribution_stay_within_one_bin() {
        let mut h = Hist::default();
        for ms in 1..=1000u64 {
            h.record(ms * 10);
        }
        assert_eq!(h.count, 1000);
        for (p, truth) in [(50, 5000.0), (75, 7500.0), (95, 9500.0), (99, 9900.0)] {
            let got = h.percentile_ms(p).unwrap();
            assert!((got - truth).abs() / truth < 0.22, "p{p}: {got} vs {truth}");
        }
        assert_eq!(h.mean_ms().unwrap(), 5005.0);
    }

    #[test]
    fn merging_buckets_equals_recording_into_one() {
        let (mut a, mut b, mut all) = (Hist::default(), Hist::default(), Hist::default());
        for ms in (50..3000).step_by(7) {
            a.record(ms);
            all.record(ms);
        }
        for ms in (400..20000).step_by(11) {
            b.record(ms);
            all.record(ms);
        }
        a.merge(&b);
        assert_eq!(a, all);
        assert!(a.valid());
    }

    #[test]
    fn empty_and_single_sample() {
        let mut h = Hist::default();
        assert!(h.percentile_ms(50).is_none());
        assert!(h.mean_ms().is_none());
        h.record(1234);
        let (lo, hi) = bin_range(bin_of(1234));
        let p = h.percentile_ms(50).unwrap();
        assert!(p >= lo && p <= hi);
        assert!(h.percentile_ms(0).is_none() && h.percentile_ms(101).is_none());
    }

    #[test]
    fn forged_documents_are_refused() {
        let mut h = Hist::default();
        h.record(100);
        h.count = 2;
        assert!(!h.valid());
        let mut short = Hist::default();
        short.bins.pop();
        assert!(!short.valid());
    }

    #[tokio::test]
    async fn persistence_round_trip_idempotency_and_retention() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        let mut hist = Hist::default();
        for ms in [100, 200, 3000] {
            hist.record(ms);
        }
        let now = 3600 * 100;
        let bucket = TimingBucket {
            start: now - 3600,
            metric: "timeToUsable".into(),
            device: "desktop".into(),
            hist,
        };
        for _ in 0..2 {
            store
                .save_client_timing("one", std::slice::from_ref(&bucket), now, 30)
                .await
                .unwrap();
        }
        let got = store.client_timing_history(0, now + 3600).await.unwrap();
        assert_eq!(got, vec![bucket.clone()]);
        let old = TimingBucket {
            start: 0,
            ..bucket.clone()
        };
        store
            .save_client_timing("one", &[old], now, 30)
            .await
            .unwrap();
        store
            .save_client_timing("one", &[], now + 31 * 86400, 30)
            .await
            .unwrap();
        assert!(
            store
                .client_timing_history(0, now + 3600)
                .await
                .unwrap()
                .is_empty()
        );
        let bad = TimingBucket {
            metric: "path".into(),
            ..bucket
        };
        assert!(
            store
                .save_client_timing("one", &[bad], now, 30)
                .await
                .is_err()
        );
    }
}
