//! Image proxy latency: how long a proxied image takes, where the time goes, and
//! how each request ended.
//!
//! Per request the proxy measures the total time from the request arriving to the
//! response being ready, plus four phases: waiting for capacity (the queue), the
//! cache lookup, the upstream fetch and the decode / re-encode. Each finished
//! request lands in a fixed log-scale histogram, so nothing per request is kept:
//! no URL, host, account or message, only bucket counts.
//!
//! Layout (the "HdrHistogram-like" part): values are microseconds. 0..7 each have
//! a bucket; above that every power of two is split into 8 equal sub-buckets, up
//! to 2^26 us (about 67 s; longer values share the last bucket and the exact
//! maximum is tracked separately). A reported percentile is the middle of its
//! bucket, so it is within 1/16 (6.25 %) of the true value. 192 buckets per
//! histogram.
//!
//! Time slices: one set of histograms per minute ([`SLICE_SECS`]), kept for 24 h
//! ([`KEEP_SECS`]) in a ring, next to (and consistent with) the worker history
//! in `remote_image_history.rs`: in this process only, empty after a restart,
//! not shared between replicas. A finished slice is stored sparsely (only the
//! non-empty buckets); the worst case is 1440 slices x 12 histograms x 192
//! buckets, a few MB, and a real instance sits far below it.
//!
//! A range is answered by merging bucket counts across its slices, so median,
//! p95, p99, max and count come from the merged buckets, never from averaging
//! per-slice percentiles.

use std::cell::Cell;
use std::collections::VecDeque;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use axum::response::Response;
use serde_json::{Value, json};

pub(crate) const SLICE_SECS: u64 = 60;
pub(crate) const KEEP_SECS: u64 = 24 * 60 * 60;
pub(crate) const CAPACITY: usize = (KEEP_SECS / SLICE_SECS) as usize;
/// The most points of the over-time series.
pub(crate) const MAX_POINTS: usize = 120;

// Health verdict thresholds (constants, not settings). They are returned with
// the payload so the console shows the numbers it judged by.
/// Fewer finished requests than this in the range: no verdict.
pub(crate) const MIN_REQUESTS: u64 = 10;
/// A fetched image (cache miss) whose p95 reaches this is "Slow".
pub(crate) const SLOW_MISS_P95_MS: f64 = 3000.0;
/// ... and this is "Degraded".
pub(crate) const DEGRADED_MISS_P95_MS: f64 = 8000.0;
/// A cache hit whose p95 reaches this is "Slow" (hits should be near instant).
pub(crate) const SLOW_HIT_P95_MS: f64 = 250.0;
/// Share of requests that ended busy, timed out or failed upstream: "Slow" ...
pub(crate) const SLOW_FAILURE_PCT: f64 = 2.0;
/// ... and "Degraded".
pub(crate) const DEGRADED_FAILURE_PCT: f64 = 10.0;

// ── histogram ───────────────────────────────────────────────────────────────────

const SUB_BITS: u32 = 3;
const SUB: u64 = 1 << SUB_BITS;
const MAX_EXP: u32 = 26;
/// Longest value with its own bucket range; larger values share the last bucket.
const MAX_VALUE: u64 = (1 << MAX_EXP) - 1;
pub(crate) const BUCKETS: usize = (SUB + (MAX_EXP - SUB_BITS) as u64 * SUB) as usize;
/// The relative error bound of a reported percentile.
pub(crate) const RELATIVE_ERROR: f64 = 1.0 / 16.0;

fn bucket_of(us: u64) -> usize {
    if us < SUB {
        return us as usize;
    }
    let us = us.min(MAX_VALUE);
    let exp = 63 - us.leading_zeros();
    let sub = (us >> (exp - SUB_BITS)) & (SUB - 1);
    (SUB + u64::from(exp - SUB_BITS) * SUB + sub) as usize
}

/// `(lowest value, width)` of a bucket.
fn bucket_range(index: usize) -> (u64, u64) {
    let i = index as u64;
    if i < SUB {
        return (i, 1);
    }
    let k = i - SUB;
    let exp = (k / SUB) as u32 + SUB_BITS;
    let sub = k % SUB;
    let width = 1u64 << (exp - SUB_BITS);
    ((SUB + sub) * width, width)
}

/// A histogram of microsecond values.
#[derive(Clone, Debug)]
pub(crate) struct Hist {
    b: Vec<u64>,
    n: u64,
    sum: u64,
    max: u64,
}

impl Default for Hist {
    fn default() -> Self {
        Self {
            b: vec![0; BUCKETS],
            n: 0,
            sum: 0,
            max: 0,
        }
    }
}

impl Hist {
    pub(crate) fn record(&mut self, us: u64) {
        self.b[bucket_of(us)] += 1;
        self.n += 1;
        self.sum = self.sum.saturating_add(us);
        self.max = self.max.max(us);
    }

    pub(crate) fn count(&self) -> u64 {
        self.n
    }

    pub(crate) fn max_us(&self) -> u64 {
        self.max
    }

    pub(crate) fn mean_us(&self) -> Option<f64> {
        (self.n > 0).then(|| self.sum as f64 / self.n as f64)
    }

    /// Add another histogram's counts (merging slices).
    pub(crate) fn merge(&mut self, other: &Hist) {
        for (a, b) in self.b.iter_mut().zip(&other.b) {
            *a += b;
        }
        self.n += other.n;
        self.sum = self.sum.saturating_add(other.sum);
        self.max = self.max.max(other.max);
    }

    fn merge_sparse(&mut self, other: &Sparse) {
        for &(i, c) in &other.b {
            self.b[i as usize] += u64::from(c);
        }
        self.n += other.n;
        self.sum = self.sum.saturating_add(other.sum);
        self.max = self.max.max(other.max);
    }

    /// The `q` quantile (0..=1) in microseconds: the middle of the bucket holding
    /// the rank, never above the exact maximum. `None` when empty.
    pub(crate) fn quantile_us(&self, q: f64) -> Option<f64> {
        if self.n == 0 {
            return None;
        }
        let rank = ((q.clamp(0.0, 1.0) * self.n as f64).ceil() as u64).clamp(1, self.n);
        let mut seen = 0;
        for (i, &c) in self.b.iter().enumerate() {
            seen += c;
            if seen >= rank {
                let (lo, width) = bucket_range(i);
                let mid = lo as f64 + (width as f64 - 1.0) / 2.0;
                return Some(mid.min(self.max as f64));
            }
        }
        Some(self.max as f64)
    }

    fn sparse(&self) -> Sparse {
        Sparse {
            n: self.n,
            sum: self.sum,
            max: self.max,
            b: self
                .b
                .iter()
                .enumerate()
                .filter(|(_, c)| **c > 0)
                .map(|(i, c)| (i as u8, (*c).min(u64::from(u32::MAX)) as u32))
                .collect(),
        }
    }
}

/// A finished slice's histogram: only the non-empty buckets.
#[derive(Clone, Debug, Default)]
struct Sparse {
    n: u64,
    sum: u64,
    max: u64,
    b: Vec<(u8, u32)>,
}

// ── classes ─────────────────────────────────────────────────────────────────────

/// How a request ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    HitRam,
    HitDisk,
    /// Fetched upstream, re-encoded and answered.
    Miss,
    /// Upstream or decode ran out of time (`504`).
    Timeout,
    /// Upstream failed or answered something unusable (`502`, other `5xx`).
    UpstreamError,
    /// Refused or not an image (`4xx` other than busy): not a speed signal.
    Rejected,
    /// No capacity in time: `429` or `503`.
    Busy,
    /// The client went away before the response was ready.
    Cancelled,
}

const OUTCOMES: [Outcome; 8] = [
    Outcome::HitRam,
    Outcome::HitDisk,
    Outcome::Miss,
    Outcome::Timeout,
    Outcome::UpstreamError,
    Outcome::Rejected,
    Outcome::Busy,
    Outcome::Cancelled,
];

impl Outcome {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Outcome::HitRam => "hitRam",
            Outcome::HitDisk => "hitDisk",
            Outcome::Miss => "miss",
            Outcome::Timeout => "timeout",
            Outcome::UpstreamError => "upstreamError",
            Outcome::Rejected => "rejected",
            Outcome::Busy => "busy",
            Outcome::Cancelled => "cancelled",
        }
    }

    fn index(self) -> usize {
        OUTCOMES.iter().position(|o| *o == self).unwrap_or(0)
    }

    fn is_hit(self) -> bool {
        matches!(self, Outcome::HitRam | Outcome::HitDisk)
    }

    /// The outcome a response status stands for when no cache hit was seen.
    pub(crate) fn of_status(status: u16) -> Self {
        match status {
            200..=399 => Outcome::Miss,
            429 | 503 => Outcome::Busy,
            408 | 504 => Outcome::Timeout,
            500..=599 => Outcome::UpstreamError,
            _ => Outcome::Rejected,
        }
    }
}

/// A phase of serving a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Waiting for a slot, a decoder, the rate limit or another fetch of the same image.
    Queue,
    /// The cache lookup (RAM, then disk).
    Cache,
    /// The upstream fetch.
    Upstream,
    /// Decode and re-encode in the media jail.
    Decode,
}

const PHASES: [Phase; 4] = [Phase::Queue, Phase::Cache, Phase::Upstream, Phase::Decode];

impl Phase {
    fn name(self) -> &'static str {
        match self {
            Phase::Queue => "queue",
            Phase::Cache => "cache",
            Phase::Upstream => "upstream",
            Phase::Decode => "decode",
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// One finished request.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Sample {
    pub outcome: Outcome,
    pub total_us: u64,
    /// Time per phase; `None` when the request never entered the phase.
    pub phase_us: [Option<u64>; 4],
}

// ── slices ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default)]
struct Set {
    outcomes: Vec<Hist>,
    phases: Vec<Hist>,
}

impl Set {
    fn new() -> Self {
        Self {
            outcomes: vec![Hist::default(); OUTCOMES.len()],
            phases: vec![Hist::default(); PHASES.len()],
        }
    }

    fn hit(&self) -> Hist {
        let mut h = self.outcomes[Outcome::HitRam.index()].clone();
        h.merge(&self.outcomes[Outcome::HitDisk.index()]);
        h
    }

    /// Every finished request except the cancelled ones (cut short by the client).
    fn finished(&self) -> Hist {
        let mut h = Hist::default();
        for o in OUTCOMES {
            if o != Outcome::Cancelled {
                h.merge(&self.outcomes[o.index()]);
            }
        }
        h
    }

    fn count(&self, o: Outcome) -> u64 {
        self.outcomes[o.index()].count()
    }
}

#[derive(Debug)]
struct Sealed {
    t: u64,
    outcomes: Vec<Sparse>,
    phases: Vec<Sparse>,
}

#[derive(Debug)]
struct Live {
    t: u64,
    set: Set,
}

/// The slice ring plus the slice being filled.
#[derive(Debug)]
pub(crate) struct Recorder {
    cap: usize,
    live: Option<Live>,
    ring: VecDeque<Sealed>,
}

impl Recorder {
    pub(crate) fn new(cap: usize) -> Self {
        Self {
            cap: cap.max(2),
            live: None,
            ring: VecDeque::new(),
        }
    }

    fn seal(&mut self) {
        if let Some(live) = self.live.take() {
            if self.ring.len() == self.cap {
                self.ring.pop_front();
            }
            self.ring.push_back(Sealed {
                t: live.t,
                outcomes: live.set.outcomes.iter().map(Hist::sparse).collect(),
                phases: live.set.phases.iter().map(Hist::sparse).collect(),
            });
        }
    }

    /// Count a finished request at `now` (unix seconds). A clock step back puts it
    /// in the current slice rather than reordering the ring.
    pub(crate) fn record(&mut self, now: u64, s: &Sample) {
        let t = now / SLICE_SECS * SLICE_SECS;
        if self.live.as_ref().is_none_or(|l| t > l.t) {
            self.seal();
            self.live = Some(Live { t, set: Set::new() });
        }
        let Some(live) = self.live.as_mut() else { return };
        live.set.outcomes[s.outcome.index()].record(s.total_us);
        // Phases explain where a request that was not a cache hit spent its time.
        if !s.outcome.is_hit() && s.outcome != Outcome::Cancelled {
            for p in PHASES {
                if let Some(us) = s.phase_us[p.index()] {
                    live.set.phases[p.index()].record(us);
                }
            }
        }
    }

    /// Slices overlapping `[now - range_secs, now]`, merged per `step` seconds, and
    /// everything merged.
    fn merged(&self, now: u64, range_secs: u64, step: u64) -> (Vec<(u64, Set)>, Set) {
        let from = now.saturating_sub(range_secs);
        let mut points: Vec<(u64, Set)> = Vec::new();
        let mut all = Set::new();
        let mut fold = |t: u64, add: &dyn Fn(&mut Set)| {
            if t + SLICE_SECS <= from || t > now {
                return;
            }
            let b = t / step;
            if points.last().is_none_or(|(pb, _)| *pb != b) {
                points.push((b, Set::new()));
            }
            if let Some((_, set)) = points.last_mut() {
                add(set);
            }
            add(&mut all);
        };
        for s in &self.ring {
            fold(s.t, &|set| {
                for (h, sp) in set.outcomes.iter_mut().zip(&s.outcomes) {
                    h.merge_sparse(sp);
                }
                for (h, sp) in set.phases.iter_mut().zip(&s.phases) {
                    h.merge_sparse(sp);
                }
            });
        }
        if let Some(l) = &self.live {
            fold(l.t, &|set| {
                for (h, o) in set.outcomes.iter_mut().zip(&l.set.outcomes) {
                    h.merge(o);
                }
                for (h, o) in set.phases.iter_mut().zip(&l.set.phases) {
                    h.merge(o);
                }
            });
        }
        (points, all)
    }

    pub(crate) fn query(&self, now: u64, range_secs: u64, max_points: usize, enabled: bool) -> Value {
        let max_points = max_points.max(2) as u64;
        let step = (range_secs / max_points).max(SLICE_SECS) / SLICE_SECS * SLICE_SECS;
        let (points, all) = self.merged(now, range_secs, step.max(SLICE_SECS));
        let step = step.max(SLICE_SECS);

        let mut t = Vec::new();
        let mut cols: std::collections::BTreeMap<&'static str, Vec<Value>> = Default::default();
        for (b, set) in &points {
            t.push(((b + 1) * step).min(now));
            let (fin, hit, miss) = (set.finished(), set.hit(), &set.outcomes[Outcome::Miss.index()]);
            let mut put = |k: &'static str, v: Value| cols.entry(k).or_default().push(v);
            put("count", json!(fin.count()));
            put("p50Ms", ms(fin.quantile_us(0.5)));
            put("p95Ms", ms(fin.quantile_us(0.95)));
            put("p99Ms", ms(fin.quantile_us(0.99)));
            put("maxMs", ms((fin.count() > 0).then(|| fin.max_us() as f64)));
            put("missCount", json!(miss.count()));
            put("missP50Ms", ms(miss.quantile_us(0.5)));
            put("missP95Ms", ms(miss.quantile_us(0.95)));
            put("hitCount", json!(hit.count()));
            put("hitP50Ms", ms(hit.quantile_us(0.5)));
            put("hitP95Ms", ms(hit.quantile_us(0.95)));
            put("busy", json!(set.count(Outcome::Busy)));
            put("timeouts", json!(set.count(Outcome::Timeout)));
            put("upstreamErrors", json!(set.count(Outcome::UpstreamError)));
            put("cancelled", json!(set.count(Outcome::Cancelled)));
        }
        let series: serde_json::Map<String, Value> = cols.into_iter().map(|(k, v)| (k.to_string(), json!(v))).collect();

        let outcomes: serde_json::Map<String, Value> = OUTCOMES
            .iter()
            .map(|o| (o.name().to_string(), summary(&all.outcomes[o.index()])))
            .collect();
        // Where the time goes for requests that were not cache hits: each phase's
        // share of their summed total time; the rest is "other" (routing, grant
        // check, response building).
        let mut non_hit_total = Hist::default();
        for o in OUTCOMES {
            if !o.is_hit() && o != Outcome::Cancelled {
                non_hit_total.merge(&all.outcomes[o.index()]);
            }
        }
        let total_sum = non_hit_total.sum as f64;
        let mut phase_sum = 0.0;
        let phases: serde_json::Map<String, Value> = PHASES
            .iter()
            .map(|p| {
                let h = &all.phases[p.index()];
                phase_sum += h.sum as f64;
                let mut v = summary(h);
                v["sharePct"] = if total_sum > 0.0 {
                    json!(round1((h.sum as f64 / total_sum * 100.0).min(100.0)))
                } else {
                    Value::Null
                };
                (p.name().to_string(), v)
            })
            .collect();
        let other_pct = (total_sum > 0.0).then(|| round1(((total_sum - phase_sum).max(0.0) / total_sum * 100.0).min(100.0)));
        let (verdict, reason) = verdict(&all);
        let finished = all.finished();
        let failures = all.count(Outcome::Busy) + all.count(Outcome::Timeout) + all.count(Outcome::UpstreamError);
        json!({
            "enabled": enabled,
            "scope": "instance",
            "resetsOnRestart": true,
            "sliceSecs": SLICE_SECS,
            "stepSecs": step,
            "retentionSecs": KEEP_SECS,
            "relativeError": RELATIVE_ERROR,
            "rangeSecs": range_secs,
            "t": t,
            "series": series,
            "total": summary(&finished),
            "hit": summary(&all.hit()),
            "miss": summary(&all.outcomes[Outcome::Miss.index()]),
            "outcomes": outcomes,
            "phases": phases,
            "otherPct": other_pct,
            "failures": failures,
            "failurePct": (finished.count() > 0).then(|| round1(failures as f64 / finished.count() as f64 * 100.0)),
            "health": {
                "verdict": verdict,
                "reason": reason,
                "thresholds": {
                    "minRequests": MIN_REQUESTS,
                    "slowMissP95Ms": SLOW_MISS_P95_MS,
                    "degradedMissP95Ms": DEGRADED_MISS_P95_MS,
                    "slowHitP95Ms": SLOW_HIT_P95_MS,
                    "slowFailurePct": SLOW_FAILURE_PCT,
                    "degradedFailurePct": DEGRADED_FAILURE_PCT,
                },
            },
        })
    }
}

fn ms(us: Option<f64>) -> Value {
    us.map_or(Value::Null, |v| json!((v / 10.0).round() / 100.0))
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

fn summary(h: &Hist) -> Value {
    json!({
        "count": h.count(),
        "p50Ms": ms(h.quantile_us(0.5)),
        "p95Ms": ms(h.quantile_us(0.95)),
        "p99Ms": ms(h.quantile_us(0.99)),
        "maxMs": ms((h.count() > 0).then(|| h.max_us() as f64)),
        "meanMs": ms(h.mean_us()),
    })
}

/// `("noData" | "healthy" | "slow" | "degraded", why)` for the merged range.
fn verdict(all: &Set) -> (&'static str, &'static str) {
    let finished = all.finished();
    if finished.count() < MIN_REQUESTS {
        return ("noData", "fewRequests");
    }
    let failures = all.count(Outcome::Busy) + all.count(Outcome::Timeout) + all.count(Outcome::UpstreamError);
    let failure_pct = failures as f64 / finished.count() as f64 * 100.0;
    let miss_p95 = all.outcomes[Outcome::Miss.index()].quantile_us(0.95).map(|v| v / 1000.0);
    let hit_p95 = all.hit().quantile_us(0.95).map(|v| v / 1000.0);
    if failure_pct >= DEGRADED_FAILURE_PCT {
        return ("degraded", "failureRate");
    }
    if miss_p95.is_some_and(|v| v >= DEGRADED_MISS_P95_MS) {
        return ("degraded", "missP95");
    }
    if failure_pct >= SLOW_FAILURE_PCT {
        return ("slow", "failureRate");
    }
    if miss_p95.is_some_and(|v| v >= SLOW_MISS_P95_MS) {
        return ("slow", "missP95");
    }
    if hit_p95.is_some_and(|v| v >= SLOW_HIT_P95_MS) {
        return ("slow", "hitP95");
    }
    ("healthy", "ok")
}

// ── process-wide recorder and per-request trace ─────────────────────────────────

static ENABLED: AtomicBool = AtomicBool::new(true);

/// Follow `privacy.imageLatencyTracking` (read with each request's policy).
pub(crate) fn set_enabled(on: bool) {
    ENABLED.store(on, Relaxed);
}

pub(crate) fn enabled() -> bool {
    ENABLED.load(Relaxed)
}

fn store() -> &'static Mutex<Recorder> {
    static R: OnceLock<Mutex<Recorder>> = OnceLock::new();
    R.get_or_init(|| Mutex::new(Recorder::new(CAPACITY)))
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// The latency payload for a range label (`15m|1h|6h|24h`, `1h` when unknown).
pub(crate) fn payload(range: Option<&str>) -> Value {
    let label = range.filter(|r| super::history::parse_range(r).is_some()).unwrap_or("1h");
    let secs = super::history::parse_range(label).unwrap_or(3600);
    let on = enabled();
    let mut v = store().lock().map_or_else(
        |_| Recorder::new(2).query(unix_now(), secs, MAX_POINTS, on),
        |r| r.query(unix_now(), secs, MAX_POINTS, on),
    );
    v["range"] = json!(label);
    v
}

/// What one request has seen so far.
struct Trace {
    start: Instant,
    engaged: Cell<bool>,
    hit: Cell<Option<Outcome>>,
    phase_us: [Cell<Option<u64>>; 4],
}

tokio::task_local! {
    static TRACE: Trace;
}

/// The request reached the cache or later: it is proxy work worth measuring
/// (a rejected sign-in or a blocked-by-policy answer never gets here).
pub(crate) fn engage() {
    let _ = TRACE.try_with(|t| t.engaged.set(true));
}

/// The cache answered from RAM (`false`) or disk (`true`).
pub(crate) fn note_hit(disk: bool) {
    let _ = TRACE.try_with(|t| t.hit.set(Some(if disk { Outcome::HitDisk } else { Outcome::HitRam })));
}

fn add_phase(phase: Phase, us: u64) {
    let _ = TRACE.try_with(|t| {
        let c = &t.phase_us[phase.index()];
        c.set(Some(c.get().unwrap_or(0).saturating_add(us)));
    });
}

/// Run `fut`, adding its wall time to `phase` of the current request.
pub(crate) async fn timed<T>(phase: Phase, fut: impl Future<Output = T>) -> T {
    let started = Instant::now();
    let out = fut.await;
    add_phase(phase, started.elapsed().as_micros() as u64);
    out
}

fn finish(outcome: Outcome) {
    if !enabled() {
        return;
    }
    let _ = TRACE.try_with(|t| {
        if !t.engaged.get() {
            return;
        }
        let sample = Sample {
            outcome,
            total_us: t.start.elapsed().as_micros() as u64,
            phase_us: [0, 1, 2, 3].map(|i| t.phase_us[i].get()),
        };
        if let Ok(mut r) = store().lock() {
            r.record(unix_now(), &sample);
        }
    });
}

/// Records a cancelled request when the tracked future is dropped unfinished.
struct CancelGuard(bool);

impl Drop for CancelGuard {
    fn drop(&mut self) {
        if !self.0 {
            finish(Outcome::Cancelled);
        }
    }
}

/// Measure one image request from now until its response is ready.
pub(crate) async fn track(fut: impl Future<Output = Response>) -> Response {
    let trace = Trace {
        start: Instant::now(),
        engaged: Cell::new(false),
        hit: Cell::new(None),
        phase_us: Default::default(),
    };
    TRACE
        .scope(trace, async move {
            let mut guard = CancelGuard(false);
            let response = fut.await;
            guard.0 = true;
            let outcome = TRACE
                .try_with(|t| t.hit.get())
                .ok()
                .flatten()
                .filter(|_| response.status().as_u16() < 400)
                .unwrap_or_else(|| Outcome::of_status(response.status().as_u16()));
            finish(outcome);
            response
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[test]
    fn buckets_are_ordered_and_cover_their_values() {
        let mut prev = 0;
        for v in (0..300_000u64).chain([1 << 20, (1 << 26) - 1, 1 << 30]) {
            let i = bucket_of(v);
            assert!(i >= prev && i < BUCKETS, "{v} -> {i}");
            prev = i;
            let (lo, width) = bucket_range(i);
            if v <= MAX_VALUE {
                assert!(v >= lo && v < lo + width, "{v} not in {lo}+{width}");
            }
        }
        assert_eq!(bucket_of(u64::MAX), BUCKETS - 1);
    }

    #[test]
    fn empty_histogram_has_no_percentiles() {
        let h = Hist::default();
        assert_eq!(h.quantile_us(0.5), None);
        assert_eq!(h.mean_us(), None);
        assert_eq!(ms(h.quantile_us(0.95)), Value::Null);
        let r = Recorder::new(10);
        let v = r.query(10_000, 3600, MAX_POINTS, true);
        assert_eq!(v["t"].as_array().unwrap().len(), 0);
        assert_eq!(v["total"]["count"], json!(0));
        assert_eq!(v["health"]["verdict"], json!("noData"));
    }

    #[test]
    fn percentiles_stay_within_the_documented_error() {
        // 1..=100_000 us uniformly: exact quantiles are known.
        let mut h = Hist::default();
        for v in 1..=100_000u64 {
            h.record(v);
        }
        for (q, exact) in [(0.5, 50_000.0), (0.95, 95_000.0), (0.99, 99_000.0)] {
            let got = h.quantile_us(q).unwrap();
            assert!(
                (got - exact).abs() / exact <= RELATIVE_ERROR,
                "q{q}: {got} vs {exact}"
            );
        }
        assert_eq!(h.max_us(), 100_000);
        assert!(h.quantile_us(1.0).unwrap() <= 100_000.0);
        // A single value is reported within the bound too, across the range.
        for v in [1u64, 7, 8, 9, 100, 1234, 56_789, 3_000_000, 60_000_000] {
            let mut one = Hist::default();
            one.record(v);
            let got = one.quantile_us(0.5).unwrap();
            assert!((got - v as f64).abs() / (v as f64).max(1.0) <= RELATIVE_ERROR, "{v}: {got}");
        }
    }

    #[test]
    fn merging_buckets_equals_recording_into_one() {
        let (mut a, mut b, mut whole) = (Hist::default(), Hist::default(), Hist::default());
        for v in 1..=5000u64 {
            let us = v * 37 % 90_000 + 1;
            whole.record(us);
            let target = if v % 2 == 0 { &mut a } else { &mut b };
            target.record(us);
        }
        let mut merged = Hist::default();
        merged.merge(&a.sparse_roundtrip());
        merged.merge(&b);
        assert_eq!(merged.count(), whole.count());
        assert_eq!(merged.max_us(), whole.max_us());
        for q in [0.5, 0.95, 0.99] {
            assert_eq!(merged.quantile_us(q), whole.quantile_us(q));
        }
    }

    impl Hist {
        fn sparse_roundtrip(&self) -> Hist {
            let mut h = Hist::default();
            h.merge_sparse(&self.sparse());
            h
        }
    }

    fn sample(outcome: Outcome, total_ms: u64) -> Sample {
        let mut phase_us = [None; 4];
        if !outcome.is_hit() {
            phase_us = [Some(total_ms * 100), Some(10), Some(total_ms * 600), Some(total_ms * 200)];
        }
        Sample {
            outcome,
            total_us: total_ms * 1000,
            phase_us,
        }
    }

    #[test]
    fn slices_merge_across_a_range_and_the_ring_is_bounded() {
        let mut r = Recorder::new(5);
        let base = 1_000_000 / SLICE_SECS * SLICE_SECS;
        for i in 0..8u64 {
            for _ in 0..20 {
                r.record(base + i * SLICE_SECS, &sample(Outcome::HitRam, 2));
            }
            for k in 1..=10 {
                r.record(base + i * SLICE_SECS + 1, &sample(Outcome::Miss, k * 100));
            }
        }
        assert!(r.ring.len() <= 5, "ring holds at most its capacity");
        let now = base + 7 * SLICE_SECS + 30;
        let v = r.query(now, 600, MAX_POINTS, true);
        // 6 slices survive (5 sealed + the live one); the range sees the ones inside it.
        assert_eq!(v["hit"]["count"], json!(20 * 6));
        assert_eq!(v["miss"]["count"], json!(10 * 6));
        let p95 = v["miss"]["p95Ms"].as_f64().unwrap();
        assert!((p95 - 1000.0).abs() / 1000.0 <= RELATIVE_ERROR, "{p95}");
        let hit = v["hit"]["p50Ms"].as_f64().unwrap();
        assert!((hit - 2.0).abs() <= 0.2, "{hit}");
        // Series: one count per point, aligned with t.
        let n = v["t"].as_array().unwrap().len();
        for (_, s) in v["series"].as_object().unwrap() {
            assert_eq!(s.as_array().unwrap().len(), n);
        }
        // Phase shares: upstream 60 %, decode 20 %, queue 10 % of the miss time.
        assert!((v["phases"]["upstream"]["sharePct"].as_f64().unwrap() - 60.0).abs() < 8.0);
        assert!(v["otherPct"].as_f64().unwrap() < 15.0);
        // A narrow range sees only the newest slices.
        let narrow = r.query(now, 60, MAX_POINTS, true);
        assert_eq!(narrow["hit"]["count"], json!(20 * 2));
    }

    #[test]
    fn health_verdict_follows_the_thresholds() {
        let run = |samples: &[(Outcome, u64, usize)]| {
            let mut r = Recorder::new(10);
            for (o, ms, n) in samples {
                for _ in 0..*n {
                    r.record(120, &sample(*o, *ms));
                }
            }
            r.query(130, 900, MAX_POINTS, true)["health"]["verdict"].clone()
        };
        assert_eq!(run(&[(Outcome::Miss, 100, 5)]), json!("noData"));
        assert_eq!(run(&[(Outcome::Miss, 300, 30), (Outcome::HitRam, 1, 70)]), json!("healthy"));
        assert_eq!(run(&[(Outcome::Miss, 4000, 30), (Outcome::HitRam, 1, 70)]), json!("slow"));
        assert_eq!(run(&[(Outcome::Miss, 9000, 30), (Outcome::HitRam, 1, 70)]), json!("degraded"));
        assert_eq!(run(&[(Outcome::Miss, 300, 90), (Outcome::Busy, 1, 5)]), json!("slow"));
        assert_eq!(run(&[(Outcome::Miss, 300, 80), (Outcome::Timeout, 8000, 20)]), json!("degraded"));
        // Rejected (not an image, policy) and cancelled requests are not speed failures.
        assert_eq!(run(&[(Outcome::Miss, 300, 30), (Outcome::Rejected, 5, 40), (Outcome::Cancelled, 5, 40)]), json!("healthy"));
    }

    #[test]
    fn statuses_map_to_outcomes() {
        assert_eq!(Outcome::of_status(200), Outcome::Miss);
        assert_eq!(Outcome::of_status(429), Outcome::Busy);
        assert_eq!(Outcome::of_status(503), Outcome::Busy);
        assert_eq!(Outcome::of_status(504), Outcome::Timeout);
        assert_eq!(Outcome::of_status(502), Outcome::UpstreamError);
        assert_eq!(Outcome::of_status(415), Outcome::Rejected);
        assert_eq!(Outcome::of_status(403), Outcome::Rejected);
    }

    #[tokio::test]
    async fn untracked_calls_are_no_ops_and_cancellation_is_seen() {
        // Outside a scope nothing panics.
        engage();
        note_hit(true);
        timed(Phase::Queue, async {}).await;

        let before = || {
            let r = store().lock().unwrap();
            let all = r.merged(unix_now(), 120, 60).1;
            (all.count(Outcome::Cancelled), all.count(Outcome::HitDisk), all.count(Outcome::Busy))
        };
        let (c0, d0, b0) = before();
        // Finished with a disk hit.
        track(async {
            engage();
            note_hit(true);
            timed(Phase::Cache, tokio::time::sleep(std::time::Duration::from_millis(2))).await;
            (axum::http::StatusCode::OK, "x").into_response()
        })
        .await;
        // Busy.
        track(async {
            engage();
            (axum::http::StatusCode::SERVICE_UNAVAILABLE, "busy").into_response()
        })
        .await;
        // Not engaged (auth failure): not recorded.
        track(async { (axum::http::StatusCode::UNAUTHORIZED, "no").into_response() }).await;
        // Cancelled after engaging.
        let handle = tokio::spawn(track(async {
            engage();
            std::future::pending::<Response>().await
        }));
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        handle.abort();
        let _ = handle.await;
        let (c1, d1, b1) = before();
        assert_eq!((c1 - c0, d1 - d0, b1 - b0), (1, 1, 1));
    }
}
