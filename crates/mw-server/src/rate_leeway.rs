//! Leeway for legitimate clients (t106): queueing gates, one knob set, and the
//! counters and recent-refusal list behind the request-protection page.
//!
//! # Why this exists
//! Ordinary use of the web app produced `429` from three places that were never
//! meant to refuse a signed-in reader:
//!
//! * `jmap_limits` held **one process-wide** semaphore of four JMAP requests and
//!   answered `429 maxConcurrentRequests` the instant it was full. Every signed-in
//!   user shared those four slots, so a second tab, a send racing a folder load or
//!   simply a second user was enough.
//! * `contact_photo` and `attachment_preview` used the same `try_acquire`-or-429
//!   shape on a two-slot decoder pool, which a list of avatars exceeds at once.
//! * The cookie-consent and client-timing endpoints kept budgets sized for one
//!   visit (20 per address; six beacons per session), which one office address or
//!   one busy session exceeds.
//!
//! # What changed
//! * Concurrency gates are **per account** (a signed-in account is attributable and
//!   never shares a budget with a stranger or a colleague behind the same address)
//!   with a high process ceiling behind them, and they **queue** for up to
//!   `ratelimit.queueMs` before they refuse. A request that fits after a short
//!   wait is simply a little slower.
//! * A refusal is always `429` or `503` with `Retry-After` and the
//!   `RateLimit-Limit`/`-Remaining`/`-Reset` fields, so a client can pace itself.
//! * Every gate counts allowed, queued and rejected requests per class, and every
//!   `429` is listed (class, reason, key type, time; never an address, an account
//!   or a path) on the request-protection page.
//!
//! # What is not widened
//! Sign-in, step-up, password, recovery and linked-mailbox renewal limiters, the
//! anonymous per-address budget of request protection, the user-agent budgets and
//! the active adaptive blocks are untouched. A `429` is never an adaptive signal
//! (`request_protection::Context::observe` records only typed anomalies), so a
//! client that was queued out never escalates the penalty against itself.
//!
//! State is process-local and per replica, and resets on restart, like the other
//! in-process limiters.

use crate::settings_registry;
use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::Response,
};
use mw_store::{Store, settings_policy::SettingSubject};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, SemaphorePermit};

pub(crate) const QUEUE_MS: &str = "ratelimit.queueMs";
pub(crate) const JMAP_PER_ACCOUNT: &str = "ratelimit.jmap.perAccountConcurrent";
pub(crate) const JMAP_GLOBAL: &str = "ratelimit.jmap.globalConcurrent";
pub(crate) const SESSION_BURST: &str = "ratelimit.session.burst";
pub(crate) const SESSION_PER_MINUTE: &str = "ratelimit.session.perMinute";
pub(crate) const ANON_BURST: &str = "ratelimit.anon.burst";
pub(crate) const ANON_PER_MINUTE: &str = "ratelimit.anon.perMinute";
pub(crate) const TRUSTED_PERCENT: &str = "ratelimit.trustedMultiplier";

/// The effective values for one domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Knobs {
    /// Longest a request waits for a slot before it is refused (ms).
    pub queue_ms: u64,
    /// JMAP requests one account may run at once.
    pub jmap_per_account: usize,
    /// JMAP requests the process runs at once, all accounts together.
    pub jmap_global: usize,
    /// Reports one session may send in a row (client timing).
    pub session_burst: u32,
    /// Reports per minute a session regains.
    pub session_per_minute: u32,
    /// Requests one address may make to a cookie-consent route in a row.
    pub anon_burst: u32,
    /// Requests per minute that address regains.
    pub anon_per_minute: u32,
    /// Account request allowance, percent, for a signed-in request from a source
    /// with a successful sign-in on record.
    pub trusted_percent: u32,
}

impl Default for Knobs {
    fn default() -> Self {
        Self {
            queue_ms: 3_000,
            jmap_per_account: 8,
            jmap_global: 256,
            session_burst: 60,
            session_per_minute: 30,
            anon_burst: 60,
            anon_per_minute: 30,
            trusted_percent: 150,
        }
    }
}

impl Knobs {
    fn from_rows(rows: &[settings_registry::Effective]) -> Self {
        let mut out = Self::default();
        let num = |key: &str| {
            rows.iter()
                .find(|row| row.key == key)
                .and_then(|row| row.value.as_u64())
        };
        if let Some(v) = num(QUEUE_MS) {
            out.queue_ms = v;
        }
        if let Some(v) = num(JMAP_PER_ACCOUNT) {
            out.jmap_per_account = (v as usize).max(1);
        }
        if let Some(v) = num(JMAP_GLOBAL) {
            out.jmap_global = (v as usize).max(1);
        }
        if let Some(v) = num(SESSION_BURST) {
            out.session_burst = u32::try_from(v).unwrap_or(u32::MAX).max(1);
        }
        if let Some(v) = num(SESSION_PER_MINUTE) {
            out.session_per_minute = u32::try_from(v).unwrap_or(u32::MAX).max(1);
        }
        if let Some(v) = num(ANON_BURST) {
            out.anon_burst = u32::try_from(v).unwrap_or(u32::MAX).max(1);
        }
        if let Some(v) = num(ANON_PER_MINUTE) {
            out.anon_per_minute = u32::try_from(v).unwrap_or(u32::MAX).max(1);
        }
        if let Some(v) = num(TRUSTED_PERCENT) {
            out.trusted_percent = u32::try_from(v).unwrap_or(100).clamp(100, 1000);
        }
        out
    }

    pub(crate) fn queue(&self) -> Duration {
        Duration::from_millis(self.queue_ms)
    }
}

const KNOB_TTL: Duration = Duration::from_secs(15);
const KNOB_DOMAINS: usize = 256;

fn knob_cache() -> &'static Mutex<HashMap<String, (Instant, Knobs)>> {
    static CACHE: OnceLock<Mutex<HashMap<String, (Instant, Knobs)>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// The knobs for `domain` (`""` is the global value), cached for a few seconds. A
/// store error keeps the defaults rather than meaning "unlimited".
pub(crate) async fn knobs(store: &Store, domain: &str) -> Knobs {
    let domain = domain.to_ascii_lowercase();
    if let Some((at, value)) = knob_cache()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&domain)
        && at.elapsed() < KNOB_TTL
    {
        return *value;
    }
    let subject = SettingSubject {
        domain: &domain,
        ..Default::default()
    };
    let value = match settings_registry::effective_all(store, &subject, |def| {
        def.key.starts_with("ratelimit.")
    })
    .await
    {
        Ok(rows) => Knobs::from_rows(&rows),
        Err(_) => Knobs::default(),
    };
    let mut cache = knob_cache().lock().unwrap_or_else(|e| e.into_inner());
    if cache.len() >= KNOB_DOMAINS {
        cache.clear();
    }
    cache.insert(domain, (Instant::now(), value));
    value
}

/// Forget the cached values (tests and tools; they otherwise expire in seconds).
pub(crate) fn forget_knobs() {
    knob_cache().lock().unwrap_or_else(|e| e.into_inner()).clear();
}

/// The domain of a session's username (`user@domain`), or `""`.
pub(crate) fn domain_of(username: &str) -> &str {
    username.rsplit_once('@').map_or("", |(_, domain)| domain)
}

// ─── counters and the recent list ──────────────────────────────────────────────

/// The classes the page reports, in display order.
pub(crate) const CLASSES: [&str; 11] = [
    "jmap",
    "jmapUpload",
    "contactPhoto",
    "attachmentPreview",
    "clientTiming",
    "cookieConsent",
    "imageProxy",
    "sanitize",
    "admin",
    "api",
    "other",
];

#[derive(Default)]
struct Counters {
    allowed: AtomicU64,
    queued: AtomicU64,
    rejected: AtomicU64,
}

fn counters() -> &'static [Counters; CLASSES.len()] {
    static C: OnceLock<[Counters; CLASSES.len()]> = OnceLock::new();
    C.get_or_init(|| std::array::from_fn(|_| Counters::default()))
}

fn class_index(class: &str) -> usize {
    CLASSES
        .iter()
        .position(|c| *c == class)
        .unwrap_or(CLASSES.len() - 1)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Outcome names, in the order of [`totals`].
pub(crate) const OUTCOME_NAMES: [&str; 3] = ["allowed", "queued", "rejected"];

/// Decisions since the process started, per class (in [`CLASSES`] order) as
/// `[allowed, queued, rejected]`.
pub(crate) fn totals() -> Vec<[u64; 3]> {
    counters()
        .iter()
        .map(|c| {
            [
                c.allowed.load(Ordering::Relaxed),
                c.queued.load(Ordering::Relaxed),
                c.rejected.load(Ordering::Relaxed),
            ]
        })
        .collect()
}

pub(crate) enum Outcome {
    Allowed,
    Queued,
    Rejected,
}

pub(crate) fn note(class: &str, outcome: Outcome) {
    let c = &counters()[class_index(class)];
    let counter = match outcome {
        Outcome::Allowed => &c.allowed,
        Outcome::Queued => &c.queued,
        Outcome::Rejected => &c.rejected,
    };
    counter.fetch_add(1, Ordering::Relaxed);
}

/// What a limiter says about its own refusal. Put on the response as an
/// extension so [`observe`] can add the headers and list it.
#[derive(Clone, Debug)]
pub(crate) struct Refused {
    pub class: &'static str,
    pub reason: &'static str,
    /// `account`, `session`, `address` or `process`; never the key itself.
    pub key_type: &'static str,
    /// The allowance that was exhausted, for `RateLimit-Limit`.
    pub limit: Option<u32>,
    pub retry_after_secs: u64,
}

/// Tag `response` as a refusal by a limiter that counted itself.
pub(crate) fn tag(mut response: Response, refused: Refused) -> Response {
    response.extensions_mut().insert(refused);
    response
}

/// How many recent refusals are kept.
const RECENT_CAPACITY: usize = 100;

#[derive(Clone, Debug)]
struct Recent {
    at: i64,
    status: u16,
    class: &'static str,
    reason: &'static str,
    key_type: &'static str,
    retry_after_secs: u64,
}

fn recent() -> &'static Mutex<VecDeque<Recent>> {
    static R: OnceLock<Mutex<VecDeque<Recent>>> = OnceLock::new();
    R.get_or_init(Default::default)
}

fn push_recent(entry: Recent) {
    let mut list = recent().lock().unwrap_or_else(|e| e.into_inner());
    if list.len() >= RECENT_CAPACITY {
        list.pop_front();
    }
    list.push_back(entry);
}

/// Reports clients sent about their own retries (client timing beacon).
#[derive(Default)]
struct ClientReport {
    reports: AtomicU64,
    seen: AtomicU64,
    retries: AtomicU64,
    exhausted: AtomicU64,
}

fn client_report() -> &'static ClientReport {
    static R: OnceLock<ClientReport> = OnceLock::new();
    R.get_or_init(Default::default)
}

/// Add one client's report of how many `429`s it saw, how many requests it sent
/// again and how many gave up. Each is clamped so one report cannot dominate.
pub(crate) fn note_client(seen: u32, retries: u32, exhausted: u32) {
    let r = client_report();
    r.reports.fetch_add(1, Ordering::Relaxed);
    r.seen.fetch_add(u64::from(seen.min(1000)), Ordering::Relaxed);
    r.retries
        .fetch_add(u64::from(retries.min(1000)), Ordering::Relaxed);
    r.exhausted
        .fetch_add(u64::from(exhausted.min(1000)), Ordering::Relaxed);
}

/// The request-protection page's view: counters per class, the recent list and
/// the clients' own report.
pub(crate) fn snapshot() -> Value {
    let classes: Vec<Value> = CLASSES
        .iter()
        .zip(counters().iter())
        .map(|(name, c)| {
            json!({
                "class": name,
                "allowed": c.allowed.load(Ordering::Relaxed),
                "queued": c.queued.load(Ordering::Relaxed),
                "rejected": c.rejected.load(Ordering::Relaxed),
            })
        })
        .collect();
    let list: Vec<Value> = recent()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .rev()
        .map(|r| {
            json!({
                "at": r.at,
                "status": r.status,
                "class": r.class,
                "reason": r.reason,
                "keyType": r.key_type,
                "retryAfterSecs": r.retry_after_secs,
            })
        })
        .collect();
    let c = client_report();
    json!({
        "classes": classes,
        "recent": list,
        "recentCapacity": RECENT_CAPACITY,
        "clients": {
            "reports": c.reports.load(Ordering::Relaxed),
            "rateLimited": c.seen.load(Ordering::Relaxed),
            "retries": c.retries.load(Ordering::Relaxed),
            "gaveUp": c.exhausted.load(Ordering::Relaxed),
        },
    })
}

// ─── the queueing gate ─────────────────────────────────────────────────────────

/// Slots held for one request; both are released on drop.
pub(crate) struct Held {
    _key: OwnedSemaphorePermit,
    _global: OwnedSemaphorePermit,
}

/// Why a gate refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Busy {
    /// The account already runs its allowance and none finished in time.
    Account { limit: usize },
    /// The process is at its ceiling and nothing finished in time.
    Process { limit: usize },
}

impl Busy {
    pub(crate) fn refused(self, class: &'static str) -> Refused {
        match self {
            Self::Account { limit } => Refused {
                class,
                reason: "accountConcurrency",
                key_type: "account",
                limit: u32::try_from(limit).ok(),
                retry_after_secs: 1,
            },
            Self::Process { limit } => Refused {
                class,
                reason: "processConcurrency",
                key_type: "process",
                limit: u32::try_from(limit).ok(),
                retry_after_secs: 2,
            },
        }
    }
}

/// A concurrency gate with a slot count per key and one for the process. Both
/// counts are read at each call, so a settings change applies to the next request.
pub(crate) struct Gate {
    class: &'static str,
    per_key: Mutex<(usize, HashMap<String, Arc<Semaphore>>)>,
    global: Mutex<(usize, Arc<Semaphore>)>,
}

/// Keys kept before idle ones are dropped.
const GATE_KEYS: usize = 8_192;

impl Gate {
    pub(crate) fn new(class: &'static str) -> Self {
        Self {
            class,
            per_key: Mutex::new((0, HashMap::new())),
            global: Mutex::new((0, Arc::new(Semaphore::new(0)))),
        }
    }

    fn key_semaphore(&self, key: &str, slots: usize) -> Arc<Semaphore> {
        let mut guard = self.per_key.lock().unwrap_or_else(|e| e.into_inner());
        if guard.0 != slots {
            // A changed allowance starts a fresh table; permits still held are
            // returned to the semaphores they came from.
            *guard = (slots, HashMap::new());
        }
        if guard.1.len() >= GATE_KEYS && !guard.1.contains_key(key) {
            guard.1.retain(|_, sem| Arc::strong_count(sem) > 1);
        }
        guard
            .1
            .entry(key.to_owned())
            .or_insert_with(|| Arc::new(Semaphore::new(slots)))
            .clone()
    }

    fn global_semaphore(&self, slots: usize) -> Arc<Semaphore> {
        let mut guard = self.global.lock().unwrap_or_else(|e| e.into_inner());
        if guard.0 != slots {
            *guard = (slots, Arc::new(Semaphore::new(slots)));
        }
        guard.1.clone()
    }

    /// Take a slot for `key`, waiting at most `wait` for each of the two. An
    /// account is served in arrival order and never waits on another account's
    /// allowance; the process ceiling is taken last, so a waiting request holds
    /// nothing a peer needs.
    pub(crate) async fn acquire(
        &self,
        key: &str,
        per_key: usize,
        global: usize,
        wait: Duration,
    ) -> Result<Held, Busy> {
        let key_sem = self.key_semaphore(key, per_key);
        let global_sem = self.global_semaphore(global);
        let mut queued = false;
        let key_permit = match key_sem.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                queued = true;
                match tokio::time::timeout(wait, key_sem.acquire_owned()).await {
                    Ok(Ok(p)) => p,
                    _ => {
                        note(self.class, Outcome::Rejected);
                        return Err(Busy::Account { limit: per_key });
                    }
                }
            }
        };
        let global_permit = match global_sem.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                queued = true;
                match tokio::time::timeout(wait, global_sem.acquire_owned()).await {
                    Ok(Ok(p)) => p,
                    _ => {
                        note(self.class, Outcome::Rejected);
                        return Err(Busy::Process { limit: global });
                    }
                }
            }
        };
        note(
            self.class,
            if queued {
                Outcome::Queued
            } else {
                Outcome::Allowed
            },
        );
        Ok(Held {
            _key: key_permit,
            _global: global_permit,
        })
    }
}

/// A shared pool (decoders, preview workers) that waits up to `wait` for a slot.
pub(crate) async fn acquire_pool(
    class: &'static str,
    pool: &'static Semaphore,
    wait: Duration,
) -> Option<SemaphorePermit<'static>> {
    if let Ok(permit) = pool.try_acquire() {
        note(class, Outcome::Allowed);
        return Some(permit);
    }
    match tokio::time::timeout(wait, pool.acquire()).await {
        Ok(Ok(permit)) => {
            note(class, Outcome::Queued);
            Some(permit)
        }
        _ => {
            note(class, Outcome::Rejected);
            None
        }
    }
}

/// The `Refused` a pool refusal carries.
pub(crate) fn pool_refusal(class: &'static str, limit: usize) -> Refused {
    Refused {
        class,
        reason: "poolBusy",
        key_type: "process",
        limit: u32::try_from(limit).ok(),
        retry_after_secs: 1,
    }
}

// ─── the response layer ────────────────────────────────────────────────────────

/// The route class of a path, for a refusal no limiter described. Never keeps the path.
fn route_class(path: &str) -> &'static str {
    let admin = mw_admin::config::admin_path();
    if path.starts_with("/jmap/") {
        "jmap"
    } else if path == "/api/sanitize" {
        "sanitize"
    } else if path.starts_with("/api/rc/") || path.starts_with("/api/image") {
        "imageProxy"
    } else if path == "/api/client-timing" {
        "clientTiming"
    } else if path.starts_with("/api/cookie") || path.starts_with("/api/consent") {
        "cookieConsent"
    } else if path.strip_prefix(&admin).is_some_and(|s| s.starts_with('/')) {
        "admin"
    } else if path.starts_with("/api/") {
        "api"
    } else {
        "other"
    }
}

/// Outermost: every `429` and every limiter `503` leaves with `Retry-After` and
/// the `RateLimit-*` fields, and is listed on the request-protection page.
pub(crate) async fn observe(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;
    let status = response.status();
    let refused = response.extensions().get::<Refused>().cloned();
    if status != StatusCode::TOO_MANY_REQUESTS && !(status == StatusCode::SERVICE_UNAVAILABLE && refused.is_some()) {
        return response;
    }
    let headers = response.headers_mut();
    let given = headers
        .get(header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse::<u64>().ok());
    let retry = given
        .or(refused.as_ref().map(|r| r.retry_after_secs))
        .unwrap_or(1)
        .clamp(1, 3600);
    if given.is_none() {
        headers.insert(header::RETRY_AFTER, HeaderValue::from(retry));
    }
    if let Some(limit) = refused.as_ref().and_then(|r| r.limit) {
        headers.insert("ratelimit-limit", HeaderValue::from(limit));
        headers.insert("ratelimit-remaining", HeaderValue::from(0u32));
        headers.insert("ratelimit-reset", HeaderValue::from(retry));
    }
    let (class, reason, key_type) = match &refused {
        Some(r) => (r.class, r.reason, r.key_type),
        None => {
            // A limiter that did not describe itself: count it here.
            let class = route_class(&path);
            note(class, Outcome::Rejected);
            (class, "unspecified", "unknown")
        }
    };
    push_recent(Recent {
        at: chrono::Utc::now().timestamp(),
        status: status.as_u16(),
        class,
        reason,
        key_type,
        retry_after_secs: retry,
    });
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn gate_is_per_key_and_queues_before_it_refuses() {
        let gate = Gate::new("jmap");
        let a = gate.acquire("a", 2, 10, Duration::ZERO).await.unwrap();
        let _a2 = gate.acquire("a", 2, 10, Duration::ZERO).await.unwrap();
        // A third for the same key with no wait is refused; another key is not.
        assert_eq!(
            gate.acquire("a", 2, 10, Duration::ZERO).await.err(),
            Some(Busy::Account { limit: 2 })
        );
        let _b = gate.acquire("b", 2, 10, Duration::ZERO).await.unwrap();
        // With a wait, the third is admitted once one finishes.
        let waiting = gate.acquire("a", 2, 10, Duration::from_secs(2));
        let release = async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            drop(a);
        };
        let (got, ()) = tokio::join!(waiting, release);
        assert!(got.is_ok());
    }

    #[tokio::test]
    async fn process_ceiling_refuses_when_every_slot_is_taken() {
        let gate = Gate::new("jmap");
        let _x = gate.acquire("a", 5, 2, Duration::ZERO).await.unwrap();
        let _y = gate.acquire("b", 5, 2, Duration::ZERO).await.unwrap();
        assert_eq!(
            gate.acquire("c", 5, 2, Duration::from_millis(20)).await.err(),
            Some(Busy::Process { limit: 2 })
        );
    }

    #[test]
    fn defaults_match_the_registry_and_domains_are_parsed() {
        assert_eq!(domain_of("alice@Example.org"), "Example.org");
        assert_eq!(domain_of("alice"), "");
        let d = Knobs::default();
        for (key, value) in [
            (QUEUE_MS, d.queue_ms),
            (JMAP_PER_ACCOUNT, d.jmap_per_account as u64),
            (JMAP_GLOBAL, d.jmap_global as u64),
            (SESSION_BURST, u64::from(d.session_burst)),
            (SESSION_PER_MINUTE, u64::from(d.session_per_minute)),
            (ANON_BURST, u64::from(d.anon_burst)),
            (ANON_PER_MINUTE, u64::from(d.anon_per_minute)),
            (TRUSTED_PERCENT, u64::from(d.trusted_percent)),
        ] {
            let def = settings_registry::lookup(key).expect("registered");
            assert_eq!(def.default_value(), json!(value), "{key}");
        }
    }
}
