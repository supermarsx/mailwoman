//! Bounded in-memory history of the image proxy's worker statistics.
//!
//! `GET /admin/egress/privacy-stats` is a point-in-time read. This module keeps
//! one cumulative-counter sample every [`SAMPLE_SECS`] seconds for [`KEEP_SECS`]
//! seconds in a ring buffer, and answers `?range=15m|1h|6h|24h` with series
//! downsampled to at most [`MAX_POINTS`] points. The history lives in this process
//! only: it is empty after a restart and is not shared between replicas.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::{Mutex, OnceLock};

use serde_json::{Value, json};

pub(crate) const SAMPLE_SECS: u64 = 15;
pub(crate) const KEEP_SECS: u64 = 24 * 60 * 60;
pub(crate) const CAPACITY: usize = (KEEP_SECS / SAMPLE_SECS) as usize;
/// The most points any series of a response carries.
pub(crate) const MAX_POINTS: usize = 120;

// Cumulative counters bumped by the request paths.
static SERVED: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);
static BUSY_429: AtomicU64 = AtomicU64::new(0);
static BUSY_503: AtomicU64 = AtomicU64::new(0);
static WAIT_US: AtomicU64 = AtomicU64::new(0);
static WAITS: AtomicU64 = AtomicU64::new(0);
static LAST_BURST: AtomicU64 = AtomicU64::new(0);
static LAST_PER_MIN: AtomicU64 = AtomicU64::new(0);

/// An image answered to a client, `bytes` long.
pub(crate) fn note_served(bytes: usize) {
    SERVED.fetch_add(1, Relaxed);
    BYTES.fetch_add(bytes as u64, Relaxed);
}

/// A busy answer: `429` (rate limit) or `503` (no slot in time).
pub(crate) fn note_busy(status: u16) {
    match status {
        429 => BUSY_429.fetch_add(1, Relaxed),
        _ => BUSY_503.fetch_add(1, Relaxed),
    };
}

/// A request waited `micros` for a rate-limit token.
pub(crate) fn note_wait(micros: u64) {
    WAIT_US.fetch_add(micros, Relaxed);
    WAITS.fetch_add(1, Relaxed);
}

/// The bucket parameters of the latest rate-limited request.
pub(crate) fn note_rate(burst: usize, per_min: usize) {
    LAST_BURST.store(burst as u64, Relaxed);
    LAST_PER_MIN.store(per_min as u64, Relaxed);
}

/// One reading of every cumulative counter and gauge.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Sample {
    pub t: u64,
    pub served: u64,
    pub bytes: u64,
    pub busy_429: u64,
    pub busy_503: u64,
    pub wait_us: u64,
    pub waits: u64,
    pub trackers: u64,
    pub coalesced: u64,
    pub handle_hits: u64,
    pub handle_misses: u64,
    pub high_priority: u64,
    pub ram_hits: u64,
    pub disk_hits: u64,
    pub misses: u64,
    pub ram_bytes: u64,
    pub ram_entries: u64,
    pub disk_bytes: u64,
    pub disk_entries: u64,
    pub active: u64,
    pub queued: u64,
    pub burst: u64,
    pub per_min: u64,
    pub accounts: u64,
    /// `(class, allowed, refused, failed)`, cumulative.
    pub classes: Vec<(&'static str, u64, u64, u64)>,
}

/// The ring buffer.
#[derive(Debug)]
pub(crate) struct History {
    cap: usize,
    samples: VecDeque<Sample>,
}

impl History {
    pub(crate) fn new(cap: usize) -> Self {
        Self {
            cap: cap.max(2),
            samples: VecDeque::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.samples.len()
    }

    /// Append a sample, dropping the oldest at capacity. A sample not newer than
    /// the last one is ignored (a clock step back must not reorder the series).
    pub(crate) fn push(&mut self, s: Sample) {
        if self.samples.back().is_some_and(|last| s.t <= last.t) {
            return;
        }
        if self.samples.len() == self.cap {
            self.samples.pop_front();
        }
        self.samples.push_back(s);
    }

    /// The series for the last `range_secs`, at most `max_points` per series.
    pub(crate) fn query(&self, now: u64, range_secs: u64, max_points: usize) -> Value {
        let max_points = max_points.max(2);
        let from = now.saturating_sub(range_secs);
        // One extra sample before the window anchors the first delta.
        let start = self.samples.partition_point(|s| s.t < from);
        let start = start.saturating_sub(1);
        let window: Vec<&Sample> = self.samples.iter().skip(start).collect();
        let step = (range_secs / max_points as u64).max(SAMPLE_SECS);
        // Group samples into buckets of `step` seconds, keeping the last of each
        // bucket plus the sample before it for the delta.
        let mut ends: Vec<(usize, usize)> = Vec::new(); // (index of previous, index of last)
        let mut bucket: Option<u64> = None;
        for (i, s) in window.iter().enumerate() {
            let b = s.t / step;
            if bucket == Some(b) {
                if let Some(e) = ends.last_mut() {
                    e.1 = i;
                }
            } else {
                bucket = Some(b);
                ends.push((i.saturating_sub(1), i));
            }
        }
        // The first window sample has no predecessor: its deltas are unknown.
        let mut t = Vec::new();
        let mut cols: std::collections::BTreeMap<&'static str, Vec<Option<f64>>> = Default::default();
        let mut class_cols: std::collections::BTreeMap<&'static str, [Vec<Option<f64>>; 3]> = Default::default();
        let mut put = |name: &'static str, v: Option<f64>| cols.entry(name).or_default().push(v);
        for &(prev, last) in &ends {
            let (a, b) = (window[prev], window[last]);
            let span = b.t.saturating_sub(a.t) as f64;
            let has_prev = prev != last && span > 0.0;
            // A counter that went backwards means a reset: no delta for that point.
            let delta = |x: u64, y: u64| (has_prev && y >= x).then(|| (y - x) as f64);
            let rate = |x: u64, y: u64| delta(x, y).map(|d| d / span);
            let pct = |n: Option<f64>, d: Option<f64>| match (n, d) {
                (Some(n), Some(d)) if d > 0.0 => Some((n / d * 1000.0).round() / 10.0),
                _ => None,
            };
            t.push(b.t);
            put("requestsPerSec", rate(a.served, b.served).map(round3));
            put("bytesPerSec", rate(a.bytes, b.bytes).map(f64::round));
            let lookups = delta(a.ram_hits, b.ram_hits)
                .zip(delta(a.disk_hits, b.disk_hits))
                .zip(delta(a.misses, b.misses))
                .map(|((r, d), m)| r + d + m);
            put("ramHitPct", pct(delta(a.ram_hits, b.ram_hits), lookups));
            put("diskHitPct", pct(delta(a.disk_hits, b.disk_hits), lookups));
            put("ramBytes", Some(b.ram_bytes as f64));
            put("diskBytes", Some(b.disk_bytes as f64));
            put("ramEntries", Some(b.ram_entries as f64));
            put("diskEntries", Some(b.disk_entries as f64));
            put("activeFetches", Some(b.active as f64));
            let peak = window[prev..=last].iter().map(|s| s.queued).max().unwrap_or(0);
            put("queueDepth", Some(peak as f64));
            let waits = delta(a.waits, b.waits);
            put(
                "waitMs",
                delta(a.wait_us, b.wait_us)
                    .zip(waits)
                    .map(|(us, n)| if n > 0.0 { us / n / 1000.0 } else { 0.0 })
                    .map(round3),
            );
            put("busy429", delta(a.busy_429, b.busy_429));
            put("busy503", delta(a.busy_503, b.busy_503));
            put("trackersReplaced", delta(a.trackers, b.trackers));
            put("coalescedFetches", delta(a.coalesced, b.coalesced));
            put("highPriorityRequests", delta(a.high_priority, b.high_priority));
            put(
                "handleCacheHitPct",
                pct(
                    delta(a.handle_hits, b.handle_hits),
                    delta(a.handle_hits, b.handle_hits)
                        .zip(delta(a.handle_misses, b.handle_misses))
                        .map(|(h, m)| h + m),
                ),
            );
            put("rateBurst", Some(b.burst as f64));
            put("rateRefillPerMin", Some(b.per_min as f64));
            put("activeAccounts", Some(b.accounts as f64));
            for (name, ..) in &b.classes {
                let entry = class_cols.entry(name).or_default();
                let find = |s: &Sample| s.classes.iter().find(|c| c.0 == *name).copied();
                let (pa, pb) = (find(a), find(b));
                let picks: [fn((&'static str, u64, u64, u64)) -> u64; 3] = [|c| c.1, |c| c.2, |c| c.3];
                for (slot, pick) in picks.into_iter().enumerate() {
                    entry[slot].push(match (pa, pb) {
                        (Some(x), Some(y)) => delta(pick(x), pick(y)),
                        _ => None,
                    });
                }
            }
        }
        let series: serde_json::Map<String, Value> = cols.into_iter().map(|(k, v)| (k.to_string(), json!(v))).collect();
        let classes: serde_json::Map<String, Value> = class_cols
            .into_iter()
            .map(|(k, [a, r, f])| (k.to_string(), json!({"allowed": a, "refused": r, "failed": f})))
            .collect();
        json!({
            "rangeSecs": range_secs,
            "stepSecs": step,
            "sampleSecs": SAMPLE_SECS,
            "retentionSecs": KEEP_SECS,
            "resetsOnRestart": true,
            "samples": self.samples.len(),
            "t": t,
            "series": series,
            "classes": classes,
        })
    }
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

/// `15m|1h|6h|24h` in seconds; anything else is `None`.
pub(crate) fn parse_range(range: &str) -> Option<u64> {
    match range {
        "15m" => Some(15 * 60),
        "1h" => Some(60 * 60),
        "6h" => Some(6 * 60 * 60),
        "24h" => Some(24 * 60 * 60),
        _ => None,
    }
}

fn store() -> &'static Mutex<History> {
    static H: OnceLock<Mutex<History>> = OnceLock::new();
    H.get_or_init(|| Mutex::new(History::new(CAPACITY)))
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Read every counter and gauge now.
fn gather() -> Sample {
    let tier = super::tier::global().stats(super::tier::now_ms());
    let classes = mw_egress::plan::counts()
        .into_iter()
        .map(|(c, n)| (c.as_str(), n.allowed, n.refused, n.failed))
        .collect();
    Sample {
        t: unix_now(),
        served: SERVED.load(Relaxed),
        bytes: BYTES.load(Relaxed),
        busy_429: BUSY_429.load(Relaxed),
        busy_503: BUSY_503.load(Relaxed),
        wait_us: WAIT_US.load(Relaxed),
        waits: WAITS.load(Relaxed),
        trackers: super::remote_content::trackers_replaced(),
        coalesced: super::remote_content::image_flight::coalesced(),
        handle_hits: super::remote_content::handle_cache::counters().0,
        handle_misses: super::remote_content::handle_cache::counters().1,
        high_priority: super::high_priority_requests(),
        ram_hits: tier.counters.ram_hits,
        disk_hits: tier.counters.disk_hits,
        misses: tier.counters.misses,
        ram_bytes: tier.ram_bytes as u64,
        ram_entries: tier.ram_entries as u64,
        disk_bytes: tier.disk_bytes,
        disk_entries: tier.disk_entries as u64,
        active: super::fetch_semaphore().active() as u64,
        queued: super::remote_content::queue_depth() as u64,
        burst: LAST_BURST.load(Relaxed),
        per_min: LAST_PER_MIN.load(Relaxed),
        accounts: super::rate_limiter().lock().map_or(0, |l| l.active_accounts() as u64),
        classes,
    }
}

/// Start the sampler once per process. Needs a Tokio runtime; without one it
/// does nothing and a later call tries again.
pub(crate) fn ensure_sampler() {
    static STARTED: AtomicBool = AtomicBool::new(false);
    if STARTED.load(Relaxed) {
        return;
    }
    let Ok(handle) = tokio::runtime::Handle::try_current() else {
        return;
    };
    if STARTED.swap(true, Relaxed) {
        return;
    }
    handle.spawn(async {
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(SAMPLE_SECS));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tick.tick().await;
            let sample = gather();
            if let Ok(mut h) = store().lock() {
                h.push(sample);
            }
        }
    });
}

/// The history payload for a `range` query value (`1h` when absent or unknown).
pub(crate) fn payload(range: Option<&str>) -> Value {
    let label = range.filter(|r| parse_range(r).is_some()).unwrap_or("1h");
    let secs = parse_range(label).unwrap_or(3600);
    let mut value = store()
        .lock()
        .map_or_else(|_| History::new(2).query(unix_now(), secs, MAX_POINTS), |h| h.query(unix_now(), secs, MAX_POINTS));
    value["range"] = json!(label);
    value["now"] = json!(unix_now());
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(t: u64, n: u64) -> Sample {
        Sample {
            t,
            served: n * 3,
            bytes: n * 3000,
            ram_hits: n * 2,
            misses: n,
            ram_bytes: 100 + n,
            classes: vec![("image", n, 0, 0)],
            ..Sample::default()
        }
    }

    #[test]
    fn ring_buffer_drops_the_oldest_and_ignores_stale_samples() {
        let mut h = History::new(4);
        for i in 1..=6 {
            h.push(sample(i * 15, i));
        }
        assert_eq!(h.len(), 4);
        assert_eq!(h.samples.front().unwrap().t, 45);
        h.push(sample(10, 99));
        assert_eq!(h.len(), 4, "an older sample is ignored");
    }

    #[test]
    fn capacity_covers_a_day() {
        assert_eq!(CAPACITY as u64 * SAMPLE_SECS, KEEP_SECS);
    }

    #[test]
    fn query_is_bounded_and_computes_rates() {
        let mut h = History::new(CAPACITY);
        let base = 1_000_000;
        for i in 0..CAPACITY as u64 {
            h.push(sample(base + i * 15, i));
        }
        let now = base + (CAPACITY as u64 - 1) * 15;
        for (range, secs) in [("15m", 900), ("1h", 3600), ("6h", 21_600), ("24h", 86_400)] {
            assert_eq!(parse_range(range), Some(secs));
            let v = h.query(now, secs, MAX_POINTS);
            let t = v["t"].as_array().unwrap();
            assert!(t.len() <= MAX_POINTS + 1, "{range}: {} points", t.len());
            for (_, s) in v["series"].as_object().unwrap() {
                assert_eq!(s.as_array().unwrap().len(), t.len());
            }
        }
        let v = h.query(now, 900, MAX_POINTS);
        let rps = v["series"]["requestsPerSec"].as_array().unwrap();
        // 3 requests per 1 sample step of 15 s = 0.2/s.
        assert_eq!(rps.last().unwrap().as_f64(), Some(0.2));
        let bps = v["series"]["bytesPerSec"].as_array().unwrap();
        assert_eq!(bps.last().unwrap().as_f64(), Some(200.0));
        // 2 ram hits of 3 lookups per step.
        assert_eq!(v["series"]["ramHitPct"].as_array().unwrap().last().unwrap().as_f64(), Some(66.7));
        assert_eq!(v["resetsOnRestart"], json!(true));
    }

    #[test]
    fn empty_history_answers_empty_series() {
        let v = History::new(10).query(1000, 3600, MAX_POINTS);
        assert_eq!(v["t"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn a_counter_reset_gives_a_gap_not_a_negative_rate() {
        let mut h = History::new(10);
        h.push(sample(15, 10));
        h.push(sample(30, 1));
        let v = h.query(30, 900, 120);
        assert!(v["series"]["requestsPerSec"].as_array().unwrap().last().unwrap().is_null());
    }

    #[test]
    fn unknown_range_falls_back_to_an_hour() {
        let v = payload(Some("nonsense"));
        assert_eq!(v["range"], json!("1h"));
    }
}
