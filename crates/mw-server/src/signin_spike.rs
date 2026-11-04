//! Password-spray protection, type 2: an instance-wide spike in failed sign-ins.
//!
//! Distributed spraying (many addresses, each trying a few names) never trips
//! the per-source counters. This module watches the *whole instance* against
//! what it normally looks like and, when the instance is far above that, turns
//! on a temporary **heightened mode**.
//!
//! # What is measured
//! Failed sign-ins are bucketed into five-minute buckets. Per bucket:
//! 0. failed sign-ins, 1. distinct failing sources, 2. distinct failing
//! usernames, 3. failure ratio (failures over failures plus successes),
//! 4. the distributed-spray signature: sources that each failed with at least
//! three different usernames in the bucket.
//! Only credential rejections count; successes feed the ratio.
//!
//! # Baseline
//! A slow EWMA per metric, kept twice: globally and per hour-of-week slot (168),
//! so a quiet night and a busy Monday morning each have their own normal. The
//! effective baseline is the larger of slot and global until a slot has seen
//! three weekly visits, then the slot value (never below a quarter of the global
//! one). `security.spike.adaptHours` is the global time constant (default one
//! week). Each update is clamped to `1.5x baseline + half the floor`, so even a
//! 3.9x burst that stays under the trigger moves the baseline only slightly.
//!
//! # A spike
//! A signal counts when it is at least `multiplierPercent` (default 4x) of its
//! baseline **and** at least its absolute floor (20 failed sign-ins per minute,
//! 10 sources, 30 usernames per bucket; a ratio signal also needs the failure
//! floor and at least 60%). A spike needs `signalsRequired` (default 2) signals;
//! the distributed signature counts as two. Below the floors nothing can ever
//! trigger, so a small instance, a sign-up wave or a launch (whose successes do
//! not count, and whose failures stay below the floors) is not treated as an
//! attack. Gradual growth is absorbed because the baseline follows it.
//!
//! # No training on an attack
//! A bucket in which a spike was detected, and every bucket while heightened
//! mode is on, never updates the baseline. Before `warmupHours` (default 48) of
//! observed buckets there is no enforcement at all.
//!
//! # Heightened mode
//! Lasts `durationSecs`, extended while the signals stay above *half* their
//! trigger level (hysteresis), then expires by itself. It never locks anyone
//! out: anonymous sign-ins from *unknown* sources need the proof-of-work
//! challenge (`forceChallenge`) and wait `paceFactorPercent` of the usual
//! cooldown. Authenticated users and known sources (a recent successful
//! sign-in) are unaffected. An audit event (`signin-spike-heightened`) is the
//! administrator alert.
//!
//! # Persistence and privacy
//! Baselines, the mode's expiry and the last 20 spikes are stored as one small
//! JSON document (`spike:baseline:v1` in the settings table, about 30 KB, fixed
//! size), written at most once per bucket, so a restart keeps the baseline. The
//! current bucket's source and username sets are in memory only, bounded, and
//! hold 64-bit keyed hashes.
use crate::{
    AppState, sec_cfg,
    trust_tier::{self, Tier},
};
use mw_store::SigninEvent;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    sync::{Mutex, OnceLock},
};

pub(crate) const PREFIX: &str = "security.spike.";
pub(crate) const STORE_KEY: &str = "spike:baseline:v1";
pub(crate) const BUCKET_SECS: i64 = 300;
const SLOTS: usize = 168;
const METRICS: usize = 5;
const NAMES: [&str; METRICS] = [
    "failures",
    "sources",
    "usernames",
    "failureRatio",
    "distributedSources",
];
const SOURCE_CAP: usize = 4096;
const NAME_CAP: usize = 8192;
/// Distinct usernames from one source in a bucket that make it a "sprayer".
const SPRAYER_NAMES: usize = 3;
/// Sprayer sources needed before the distributed signature can count.
const MIN_SPRAYERS: f64 = 3.0;
const MIN_RATIO: f64 = 0.6;
const MAX_FILLED_GAP_BUCKETS: i64 = 12;
const HISTORY: usize = 20;
/// Slot visits (one per week per bucket) before a slot overrides the global value.
const MATURE_SLOT: u32 = 36;

#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub enabled: bool,
    pub multiplier: f64,
    pub min_failures_per_min: f64,
    pub min_sources: f64,
    pub min_usernames: f64,
    pub signals_required: u32,
    pub warmup_buckets: u64,
    pub adapt_hours: f64,
    pub duration_secs: i64,
    pub force_challenge: bool,
    pub pace_factor_percent: u32,
}
impl Default for Config {
    fn default() -> Self {
        Self::from_values(&sec_cfg::Values::default())
    }
}
impl Config {
    pub(crate) fn from_values(v: &sec_cfg::Values) -> Self {
        Self {
            enabled: v.flag("security.spike.enabled"),
            multiplier: (v.num("security.spike.multiplierPercent").clamp(150, 2000) as f64) / 100.0,
            min_failures_per_min: v.num("security.spike.minFailuresPerMin").max(1) as f64,
            min_sources: v.num("security.spike.minSources").max(2) as f64,
            min_usernames: v.num("security.spike.minUsernames").max(2) as f64,
            signals_required: v.num("security.spike.signalsRequired").clamp(1, 4) as u32,
            warmup_buckets: v.num("security.spike.warmupHours").clamp(1, 720) * 3600
                / BUCKET_SECS as u64,
            adapt_hours: v.num("security.spike.adaptHours").clamp(6, 2160) as f64,
            duration_secs: (v.num("security.spike.durationSecs") as i64).clamp(60, 86_400),
            force_challenge: v.flag("security.spike.forceChallenge"),
            pace_factor_percent: v.num("security.spike.paceFactorPercent").clamp(100, 1000) as u32,
        }
    }
    fn floors(&self) -> [f64; METRICS] {
        let per_bucket = BUCKET_SECS as f64 / 60.0;
        [
            self.min_failures_per_min * per_bucket,
            self.min_sources,
            self.min_usernames,
            MIN_RATIO,
            MIN_SPRAYERS,
        ]
    }
}

#[derive(Clone, Copy, Default, Serialize, Deserialize, Debug)]
struct Ewma {
    v: f64,
    n: u32,
}
impl Ewma {
    fn update(&mut self, obs: f64, alpha: f64) {
        // A running mean for the first samples, then the exponential average.
        let a = alpha.max(1.0 / (f64::from(self.n) + 1.0));
        self.v += a * (obs - self.v);
        self.n = self.n.saturating_add(1);
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct SpikeRecord {
    pub at: i64,
    pub signals: Vec<String>,
    pub observed: Vec<f64>,
    pub baseline: Vec<f64>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct Persisted {
    version: u8,
    global: [Ewma; METRICS],
    slots: Vec<[Ewma; METRICS]>,
    observed_buckets: u64,
    last_bucket: i64,
    heightened_until: i64,
    history: Vec<SpikeRecord>,
}
impl Default for Persisted {
    fn default() -> Self {
        Self {
            version: 1,
            global: Default::default(),
            slots: vec![Default::default(); SLOTS],
            observed_buckets: 0,
            last_bucket: 0,
            heightened_until: 0,
            history: Vec::new(),
        }
    }
}
impl Persisted {
    fn sane(self) -> Option<Self> {
        (self.version == 1
            && self.slots.len() == SLOTS
            && self.history.len() <= HISTORY * 2
            && self.global.iter().chain(self.slots.iter().flatten()).all(|e| e.v.is_finite() && e.v >= 0.0))
        .then_some(self)
    }
}

#[derive(Default)]
struct Bucket {
    start: i64,
    failures: u32,
    successes: u32,
    sources: HashSet<u64>,
    names: HashSet<u64>,
    per_source: HashMap<u64, Vec<u64>>,
    spiked: bool,
}
impl Bucket {
    fn observed(&self) -> [f64; METRICS] {
        let total = f64::from(self.failures) + f64::from(self.successes);
        let sprayers = self
            .per_source
            .values()
            .filter(|n| n.len() >= SPRAYER_NAMES)
            .count();
        [
            f64::from(self.failures),
            self.sources.len() as f64,
            self.names.len() as f64,
            if total > 0.0 { f64::from(self.failures) / total } else { 0.0 },
            sprayers as f64,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Transition {
    Entered { signals: Vec<&'static str> },
}

pub(crate) struct Detector {
    p: Persisted,
    bucket: Bucket,
    dirty: bool,
}

fn slot_of(ts: i64) -> usize {
    ((ts / 3600).rem_euclid(SLOTS as i64)) as usize
}

impl Detector {
    pub(crate) fn new(p: Option<Persisted>) -> Self {
        Self {
            p: p.and_then(Persisted::sane).unwrap_or_default(),
            bucket: Bucket::default(),
            dirty: false,
        }
    }
    pub(crate) fn persisted(&self) -> &Persisted {
        &self.p
    }
    pub(crate) fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }
    pub(crate) fn heightened(&self, now: i64) -> bool {
        self.p.heightened_until > now
    }
    pub(crate) fn warmed_up(&self, cfg: &Config) -> bool {
        self.p.observed_buckets >= cfg.warmup_buckets
    }

    /// The baseline in force at `ts`.
    pub(crate) fn baseline(&self, ts: i64) -> [f64; METRICS] {
        let slot = &self.p.slots[slot_of(ts)];
        let mut out = [0.0; METRICS];
        for m in 0..METRICS {
            let (s, g) = (slot[m], self.p.global[m]);
            out[m] = if s.n >= MATURE_SLOT {
                s.v.max(0.25 * g.v)
            } else {
                s.v.max(g.v)
            };
        }
        out
    }

    fn alpha(cfg: &Config) -> f64 {
        1.0 - (-(BUCKET_SECS as f64) / (cfg.adapt_hours * 3600.0)).exp()
    }

    fn close_bucket(&mut self, cfg: &Config) {
        let b = std::mem::take(&mut self.bucket);
        if b.start == 0 {
            return;
        }
        self.p.observed_buckets = self.p.observed_buckets.saturating_add(1);
        self.p.last_bucket = b.start;
        self.dirty = true;
        // A spike bucket, or any bucket while heightened mode was on, never trains.
        if b.spiked || self.p.heightened_until > b.start {
            return;
        }
        let obs = b.observed();
        let base = self.baseline(b.start);
        let floors = cfg.floors();
        let a = Self::alpha(cfg);
        let slot = slot_of(b.start);
        for m in 0..METRICS {
            let cap = base[m] * 1.5 + floors[m] * 0.5;
            let clamped = obs[m].min(cap);
            self.p.global[m].update(clamped, a);
            self.p.slots[slot][m].update(clamped, (a * SLOTS as f64).min(0.5));
        }
    }

    /// Advance to the bucket containing `now`, closing finished ones.
    fn roll(&mut self, now: i64, cfg: &Config) {
        let start = now - now.rem_euclid(BUCKET_SECS);
        if self.bucket.start == start {
            return;
        }
        if self.bucket.start != 0 {
            self.close_bucket(cfg);
            // Idle buckets in between are real zeros, but a long gap is a stop or a
            // quiet spell: only the first hour is filled, the rest is skipped.
            let mut t = self.p.last_bucket + BUCKET_SECS;
            let mut filled = 0;
            while t < start && filled < MAX_FILLED_GAP_BUCKETS {
                self.bucket = Bucket { start: t, ..Default::default() };
                self.close_bucket(cfg);
                t += BUCKET_SECS;
                filled += 1;
            }
        } else if self.p.last_bucket != 0 {
            // First event after a restart: a short outage is filled, a long one skipped.
            let mut t = self.p.last_bucket + BUCKET_SECS;
            let mut filled = 0;
            while t < start && filled < MAX_FILLED_GAP_BUCKETS {
                self.bucket = Bucket { start: t, ..Default::default() };
                self.close_bucket(cfg);
                t += BUCKET_SECS;
                filled += 1;
            }
        }
        self.bucket = Bucket { start, ..Default::default() };
    }

    pub(crate) fn record_success(&mut self, now: i64, cfg: &Config) {
        self.roll(now, cfg);
        self.bucket.successes = self.bucket.successes.saturating_add(1);
    }

    /// Record a failed credential. Returns a transition when heightened mode starts.
    pub(crate) fn record_failure(
        &mut self,
        now: i64,
        source: u64,
        name: u64,
        cfg: &Config,
    ) -> Option<Transition> {
        self.roll(now, cfg);
        let b = &mut self.bucket;
        b.failures = b.failures.saturating_add(1);
        if b.sources.len() < SOURCE_CAP {
            b.sources.insert(source);
        }
        if b.names.len() < NAME_CAP {
            b.names.insert(name);
        }
        if b.per_source.len() < SOURCE_CAP || b.per_source.contains_key(&source) {
            let names = b.per_source.entry(source).or_default();
            if names.len() < SPRAYER_NAMES && !names.contains(&name) {
                names.push(name);
            }
        }
        if !cfg.enabled || !self.warmed_up(cfg) {
            return None;
        }
        let was = self.heightened(now);
        let factor = if was { 0.5 } else { 1.0 };
        let (score, signals) = self.signals(cfg, now, factor);
        if score < cfg.signals_required {
            return None;
        }
        self.bucket.spiked = true;
        self.p.heightened_until = now + cfg.duration_secs;
        self.dirty = true;
        if was {
            return None;
        }
        let record = SpikeRecord {
            at: now,
            signals: signals.iter().map(|s| (*s).to_owned()).collect(),
            observed: self.bucket.observed().to_vec(),
            baseline: self.baseline(now).to_vec(),
        };
        self.p.history.push(record);
        if self.p.history.len() > HISTORY {
            self.p.history.remove(0);
        }
        Some(Transition::Entered { signals })
    }

    /// Weighted signals over their baseline and floor, at `factor` of the
    /// trigger level (1.0 to enter, 0.5 to stay in).
    fn signals(&self, cfg: &Config, now: i64, factor: f64) -> (u32, Vec<&'static str>) {
        let obs = self.bucket.observed();
        let base = self.baseline(now);
        let floors = cfg.floors();
        // Nothing counts below the minimum failed sign-ins per minute.
        if obs[0] < floors[0] * factor {
            return (0, Vec::new());
        }
        let mut score = 0;
        let mut names = Vec::new();
        for m in 0..METRICS {
            let over = if m == 3 {
                // Ratio: needs the failure floor too, and is capped so a baseline
                // near 100% can still trigger.
                obs[0] >= floors[0] * factor
                    && obs[3] >= (base[3] * cfg.multiplier).clamp(floors[3], 0.9) * factor.max(0.8)
            } else {
                obs[m] >= floors[m] * factor
                    && obs[m] >= base[m].max(1.0) * cfg.multiplier * factor
            };
            if over {
                score += if m == 4 { 2 } else { 1 };
                names.push(NAMES[m]);
            }
        }
        (score, names)
    }

    pub(crate) fn status(&self, cfg: &Config, now: i64) -> serde_json::Value {
        let base = self.baseline(now);
        let obs = self.bucket.observed();
        let metric = |i: usize| {
            serde_json::json!({
                "metric": NAMES[i],
                "baseline": (base[i] * 100.0).round() / 100.0,
                "current": (obs[i] * 100.0).round() / 100.0,
                "floor": cfg.floors()[i],
            })
        };
        serde_json::json!({
            "enabled": cfg.enabled,
            "warmedUp": self.warmed_up(cfg),
            "observedBuckets": self.p.observed_buckets,
            "warmupBuckets": cfg.warmup_buckets,
            "bucketSecs": BUCKET_SECS,
            "heightened": self.heightened(now),
            "heightenedUntil": (self.p.heightened_until > now).then_some(self.p.heightened_until),
            "metrics": (0..METRICS).map(metric).collect::<Vec<_>>(),
            "recent": self.p.history.iter().rev().take(10).collect::<Vec<_>>(),
        })
    }
}

// ─── process-wide instance, persistence ─────────────────────────────────────

fn detector() -> &'static Mutex<Detector> {
    static D: OnceLock<Mutex<Detector>> = OnceLock::new();
    D.get_or_init(|| Mutex::new(Detector::new(None)))
}
fn lock() -> std::sync::MutexGuard<'static, Detector> {
    detector().lock().unwrap_or_else(|p| p.into_inner())
}

pub(crate) async fn load(store: &mw_store::Store) -> Option<Persisted> {
    let raw = store.get_setting(STORE_KEY).await.ok().flatten()?;
    serde_json::from_str::<Persisted>(&raw).ok()?.sane()
}
pub(crate) async fn save(store: &mw_store::Store, p: &Persisted) {
    if let Ok(json) = serde_json::to_string(p)
        && let Err(error) = store.set_setting(STORE_KEY, &json).await
    {
        tracing::warn!("spike baseline could not be stored: {error}");
    }
}

static LOADED: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();
async fn ensure_loaded(store: &mw_store::Store) {
    LOADED
        .get_or_init(|| async {
            if let Some(p) = load(store).await {
                *lock() = Detector::new(Some(p));
            }
        })
        .await;
}

pub(crate) async fn config(state: &AppState) -> Config {
    Config::from_values(&*sec_cfg::values(&state.store, PREFIX).await)
}

/// What heightened mode asks of an anonymous sign-in from a source of `tier`.
pub(crate) struct Pressure {
    pub challenge: bool,
    /// Percentage of the usual cooldown to wait.
    pub pace_percent: Option<u32>,
}
/// Is heightened mode on right now? Read only.
pub(crate) fn is_heightened() -> bool {
    lock().heightened(chrono::Utc::now().timestamp())
}
pub(crate) fn pressure(tier: Tier) -> Pressure {
    let cfg = Config::from_values(&sec_cfg::snapshot(PREFIX));
    if tier != Tier::Unknown
        || !cfg.enabled
        || !lock().heightened(chrono::Utc::now().timestamp())
    {
        return Pressure { challenge: false, pace_percent: None };
    }
    Pressure {
        challenge: cfg.force_challenge,
        pace_percent: (cfg.pace_factor_percent > 100).then_some(cfg.pace_factor_percent),
    }
}

/// Called for every authentication event (`signin_guard::observe`).
pub(crate) async fn observe(state: &AppState, event: &SigninEvent) {
    let credential_failure = event.result == "failure"
        && matches!(event.reason.as_str(), "auth-rejected" | "credential-rejected");
    if event.result != "success" && !credential_failure {
        return;
    }
    let Ok(ip) = event.ip.parse::<IpAddr>() else {
        return;
    };
    let cfg = config(state).await;
    ensure_loaded(&state.store).await;
    let now = chrono::Utc::now().timestamp();
    if event.result == "success" {
        lock().record_success(now, &cfg);
        persist_if_dirty(state).await;
        return;
    }
    let secret = state.store.signin_challenge_secret("spike");
    let key = |domain: &str, v: &str| {
        u64::from_le_bytes(sec_cfg::keyed_hash(&secret, domain, v)[..8].try_into().unwrap())
    };
    let source = key("src", &trust_tier::source_key(ip).to_string());
    let name = key("user", &event.username_attempted.trim().to_lowercase());
    let transition = lock().record_failure(now, source, name, &cfg);
    if let Some(Transition::Entered { signals }) = transition {
        let status = lock().status(&cfg, now);
        tracing::warn!("sign-in spike: heightened mode on ({})", signals.join(", "));
        crate::audit::security_event(
            state,
            crate::audit::SecurityEvent::system("signin-spike-heightened").detail(serde_json::json!({
                "signals": signals,
                "durationSecs": cfg.duration_secs,
                "metrics": status["metrics"],
            })),
        )
        .await;
    }
    persist_if_dirty(state).await;
}

async fn persist_if_dirty(state: &AppState) {
    let snapshot = {
        let mut d = lock();
        d.take_dirty().then(|| d.persisted().clone())
    };
    if let Some(p) = snapshot {
        save(&state.store, &p).await;
    }
}

pub(crate) async fn status(state: &AppState) -> serde_json::Value {
    let cfg = config(state).await;
    ensure_loaded(&state.store).await;
    lock().status(&cfg, chrono::Utc::now().timestamp())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        let mut c = Config::default();
        c.warmup_buckets = 12; // one hour in these tests
        c
    }
    const T0: i64 = 1_700_000_100;
    fn start(t: i64) -> i64 {
        t - t.rem_euclid(BUCKET_SECS)
    }

    /// Feed one bucket of `fails` failures from `sources` sources, `names_each`
    /// usernames per source, and `ok` successes. Returns the transition if any.
    fn bucket(
        d: &mut Detector,
        c: &Config,
        t: i64,
        fails: u32,
        sources: u64,
        names_each: u64,
        ok: u32,
    ) -> Option<Transition> {
        let mut out = None;
        for _ in 0..ok {
            d.record_success(t, c);
        }
        for i in 0..fails {
            let s = u64::from(i) % sources.max(1);
            let n = s * 1000 + (u64::from(i) / sources.max(1)) % names_each.max(1);
            if let Some(tr) = d.record_failure(t + i64::from(i % 200), s + 1, n + 1, c) {
                out = Some(tr);
            }
        }
        out
    }

    fn learn(d: &mut Detector, c: &Config, from: i64, buckets: i64, fails: u32) -> i64 {
        let mut t = start(from);
        for _ in 0..buckets {
            // A normal level: a few failing sources, mostly one name each, many successes.
            assert!(bucket(d, c, t, fails, 4, 1, 40).is_none());
            t += BUCKET_SECS;
        }
        t
    }

    #[test]
    fn warm_up_never_enforces() {
        let c = cfg();
        let mut d = Detector::new(None);
        let t = start(T0);
        // A flood in the very first bucket: no enforcement while warming up.
        assert!(bucket(&mut d, &c, t, 5000, 800, 6, 0).is_none());
        assert!(!d.heightened(t + 10));
        assert!(!d.warmed_up(&c));
    }

    #[test]
    fn floors_keep_small_instances_quiet() {
        let c = cfg();
        let mut d = Detector::new(None);
        let t = learn(&mut d, &c, T0, 30, 2);
        assert!(d.warmed_up(&c));
        // 16x the baseline of 2/bucket, but 32 failures in five minutes is under
        // 20 per minute: nothing may trigger.
        assert!(bucket(&mut d, &c, t, 32, 6, 1, 5).is_none());
        assert!(!d.heightened(t + 100));
    }

    #[test]
    fn a_real_spike_enters_heightened_mode_and_a_distributed_signature_alone_does() {
        let c = cfg();
        let mut d = Detector::new(None);
        let t = learn(&mut d, &c, T0, 30, 5);
        // Many sources, three or more different names each.
        let tr = bucket(&mut d, &c, t, 900, 300, 4, 0);
        assert!(matches!(tr, Some(Transition::Entered { .. })), "{tr:?}");
        assert!(d.heightened(t + 250));
        // The signature itself (sources that each tried three or more names) is
        // detected and weighs two.
        let (score, names) = d.signals(&c, t + 250, 1.0);
        assert!(names.contains(&"distributedSources") && score >= 3, "{names:?} {score}");
        // A pure volume spike from few sources and names, with many successes
        // alongside, is a single signal and does not trigger.
        let mut d2 = Detector::new(None);
        let t2 = learn(&mut d2, &c, T0, 30, 5);
        assert!(bucket(&mut d2, &c, t2, 400, 3, 1, 2000).is_none());
        assert!(!d2.heightened(t2 + 250));
    }

    #[test]
    fn growth_ramp_is_absorbed_and_never_trips() {
        let c = Config { warmup_buckets: 12, ..cfg() };
        let mut d = Detector::new(None);
        let mut t = start(T0);
        // Ten weeks of 5-minute buckets, failures per bucket growing 3x overall
        // (new accounts, a launch) with proportional sources and names.
        let buckets = 10 * 7 * 288;
        for i in 0..buckets {
            let grow = 1.0 + 2.0 * f64::from(i) / f64::from(buckets);
            let fails = (60.0 * grow) as u32;
            let sources = (12.0 * grow) as u64;
            let ok = (300.0 * grow) as u32;
            let tr = bucket(&mut d, &c, t, fails, sources, 2, ok / 10);
            assert!(tr.is_none(), "growth tripped at bucket {i}");
            t += BUCKET_SECS;
        }
        assert!(!d.heightened(t));
    }

    #[test]
    fn a_spike_does_not_train_the_baseline() {
        let c = cfg();
        let mut d = Detector::new(None);
        let t = learn(&mut d, &c, T0, 40, 5);
        let before = d.baseline(t)[0];
        // A sustained attack for six hours.
        let mut t = t;
        let mut entered = false;
        for _ in 0..72 {
            entered |= bucket(&mut d, &c, t, 1500, 400, 5, 10).is_some();
            t += BUCKET_SECS;
        }
        assert!(entered);
        assert!(d.heightened(t - 10), "still heightened while the attack continues");
        // Close the last attack bucket by moving on, then compare.
        d.record_success(t + BUCKET_SECS, &c);
        let after = d.baseline(t)[0];
        assert!(after < before * 1.2 + 1.0, "baseline moved from {before} to {after}");
    }

    #[test]
    fn heightened_mode_expires_with_hysteresis() {
        let c = Config { duration_secs: 600, ..cfg() };
        let mut d = Detector::new(None);
        let t = learn(&mut d, &c, T0, 30, 5);
        assert!(bucket(&mut d, &c, t, 900, 300, 4, 0).is_some());
        let until = d.persisted().heightened_until;
        assert!(until >= t + 600 && until <= t + BUCKET_SECS + 600);
        // Still hot (above half the trigger): the expiry is pushed out.
        let t2 = t + BUCKET_SECS;
        bucket(&mut d, &c, t2, 600, 200, 4, 0);
        assert!(d.persisted().heightened_until > until);
        // Quiet again: no more extension, then it expires by itself.
        let t3 = t2 + BUCKET_SECS;
        bucket(&mut d, &c, t3, 5, 4, 1, 40);
        let end = d.persisted().heightened_until;
        assert!(d.heightened(end - 1));
        assert!(!d.heightened(end));
    }

    #[test]
    fn restart_keeps_baseline_and_mode() {
        let c = cfg();
        let mut d = Detector::new(None);
        let t = learn(&mut d, &c, T0, 30, 5);
        assert!(bucket(&mut d, &c, t, 900, 300, 4, 0).is_some());
        d.record_success(t + BUCKET_SECS, &c);
        let json = serde_json::to_string(d.persisted()).unwrap();
        assert!(json.len() < 64 * 1024, "bounded document, got {}", json.len());
        let revived = Detector::new(serde_json::from_str::<Persisted>(&json).ok());
        assert!(revived.warmed_up(&c));
        assert!(revived.heightened(t + 100));
        assert_eq!(revived.baseline(t), d.baseline(t));
        // Garbage is refused, not trusted.
        let mut bad = d.persisted().clone();
        bad.global[0].v = f64::NAN;
        assert!(Detector::new(Some(bad)).persisted().observed_buckets == 0);
    }

    #[tokio::test]
    async fn persistence_round_trips_through_the_store() {
        let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        assert!(load(&store).await.is_none());
        let c = cfg();
        let mut d = Detector::new(None);
        learn(&mut d, &c, T0, 20, 5);
        d.record_success(T0 + 100_000, &c);
        save(&store, d.persisted()).await;
        let back = load(&store).await.unwrap();
        assert_eq!(back.observed_buckets, d.persisted().observed_buckets);
    }

    #[test]
    fn tier_pressure_never_touches_known_or_authenticated() {
        // `pressure` reads process state; with heightened mode off it asks nothing.
        for tier in [Tier::Known, Tier::Authenticated, Tier::Unknown] {
            let p = pressure(tier);
            if tier != Tier::Unknown {
                assert!(!p.challenge && p.pace_percent.is_none());
            }
        }
    }

    #[test]
    fn a_long_gap_is_skipped_not_filled_with_zeros() {
        let c = cfg();
        let mut d = Detector::new(None);
        let t = learn(&mut d, &c, T0, 30, 5);
        let observed = d.persisted().observed_buckets;
        // Two days of silence (a stop), then an event.
        d.record_success(t + 2 * 86_400, &c);
        assert!(d.persisted().observed_buckets <= observed + 1 + MAX_FILLED_GAP_BUCKETS as u64);
        assert!(d.baseline(t)[0] > 0.5, "the baseline is not zeroed by the stop");
    }
}
