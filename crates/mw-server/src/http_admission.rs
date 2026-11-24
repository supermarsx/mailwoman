//! Request admission for the built-in listener: the connection pool, the request
//! pool with its bounded FIFO wait, the event-stream, probe and admin pools, the
//! counters, the optional adaptive limit, and the live console values.
//!
//! * Request pool: `requests` permits. A request that finds none waits in FIFO
//!   order (tokio's semaphore is fair: a freed permit goes to the oldest waiter, and
//!   a new arrival cannot take it first) for up to `queue_wait`, with at most
//!   `queue_max` waiting; past either it gets 503 + `Retry-After: 1`. A client that
//!   disconnects while waiting drops its wait, which frees its queue slot and its
//!   place in the semaphore's queue.
//! * Event streams (`/jmap/eventsource`): their own pool, the size of the
//!   connection limit, no wait.
//! * Health probes (`/healthz`, `/readyz`): their own pool of [`PROBE_PERMITS`], no
//!   wait, so a full request pool or queue never delays them.
//! * Admin console (paths under the admin path): a request takes a request permit
//!   when one is free, else one of [`ADMIN_PERMITS`] reserved slots, and waits like
//!   any request only when both are taken. So a console save of the smallest
//!   values can still be undone from the console.
//! * WebSocket upgrades take a request permit only until the `101` response; the
//!   socket afterwards holds its connection permit and nothing in this module.
//!
//! Console values (`server.http.*`) are re-read when the settings revision
//! changes; the revision is checked at most once a second, prompted by arriving
//! requests (no timer). Sizes change without cutting off work in progress
//! (`live::Resizable`).
use super::adaptive::Limiter;
use super::limits::HttpLimits;
use super::live::{self, Console, Live, Resizable, Sources};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Concurrent health and readiness probes.
pub const PROBE_PERMITS: usize = 8;
/// Admin-console requests admitted beyond a full request pool.
pub const ADMIN_PERMITS: usize = 8;
/// Upper bounds (ms) of the wait-time buckets; the last bucket is everything above.
pub const WAIT_BUCKETS_MS: [u64; 8] = [10, 25, 50, 100, 250, 500, 1000, 2000];
/// Least time between two settings-revision checks.
pub const REFRESH_EVERY: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pool {
    Requests,
    Streams,
    Probes,
}

/// Which pool a path uses, and whether its in-server time feeds the adaptive
/// limiter. Not sampled: transfers whose time follows their size or a remote party
/// rather than server load (downloads, uploads, exports, imports, previews, the
/// image proxy, DAV feeds, mail refresh), upgrades and streams, and probes. Whether a
/// request is an admin-console request is decided only when it matters (a full
/// request pool), against the admin path in force then ([`is_admin_path`]).
pub fn classify(path: &str) -> (Pool, bool) {
    if path.ends_with("/jmap/eventsource") {
        return (Pool::Streams, false);
    }
    if path == "/healthz" || path == "/readyz" {
        return (Pool::Probes, false);
    }
    const UNSAMPLED: [&str; 11] = [
        "/jmap/download/",
        "/jmap/upload/",
        "/jmap/ws",
        "/jmap/actions",
        "/api/export/",
        "/api/import/",
        "/api/attachments/preview",
        "/api/image-proxy",
        "/api/mail/refresh",
        "/dav/",
        "/.well-known/caldav",
    ];
    let unsampled = UNSAMPLED.iter().any(|prefix| path.starts_with(prefix))
        || (path.starts_with("/api/privacy/requests/") && path.ends_with("/download"));
    (Pool::Requests, !unsampled)
}

/// `path` is the admin path itself or below it (`/admin`, `/admin/…`), never a
/// path that merely shares its prefix (`/administrator`).
pub fn is_admin_path(path: &str, admin: &str) -> bool {
    !admin.is_empty()
        && path.starts_with(admin)
        && matches!(path.as_bytes().get(admin.len()), None | Some(b'/'))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The wait deadline passed (or waiting is off).
    Deadline,
    /// The wait queue was full.
    QueueFull,
    /// The event-stream pool was full.
    Streams,
    /// The probe pool was full.
    Probes,
}

#[derive(Default)]
struct Counters {
    immediate: AtomicU64,
    after_wait: AtomicU64,
    admin_reserve: AtomicU64,
    wait_buckets: [AtomicU64; WAIT_BUCKETS_MS.len() + 1],
    refused_deadline: AtomicU64,
    refused_queue_full: AtomicU64,
    refused_streams: AtomicU64,
    refused_probes: AtomicU64,
    connections_refused: AtomicU64,
}

pub struct Admission {
    limits: HttpLimits,
    connections: Arc<Resizable>,
    requests: Arc<Resizable>,
    streams: Arc<Resizable>,
    probes: Arc<Semaphore>,
    admin: Arc<Semaphore>,
    /// Tests fix the admin path; otherwise it is read when needed.
    admin_path: Option<String>,
    queue_wait_ms: AtomicU64,
    queue_max: AtomicUsize,
    http2_streams: AtomicU32,
    waiting: AtomicUsize,
    inflight: AtomicUsize,
    counters: Counters,
    /// The values in force and where they came from.
    live: Mutex<(Live, Sources, Console)>,
    /// The adaptive limiter while the adaptive mode is on.
    adaptive: Mutex<Option<Limiter>>,
    adaptive_on: AtomicBool,
    store: OnceLock<mw_store::Store>,
    started: Instant,
    next_check_ms: AtomicU64,
    checking: AtomicBool,
    /// The settings revision the console values were read at (`i64::MIN` = never).
    revision: Mutex<Option<Option<i64>>>,
}

/// A held admission; the slot frees when this drops (with the response body).
pub struct Lease {
    permit: Option<OwnedSemaphorePermit>,
    /// An event-stream slot.
    _stream: Option<live::Held>,
    admission: Arc<Admission>,
    /// The permit came from the resizable request pool.
    from_requests: bool,
    admitted: Instant,
    inflight: usize,
    sampled: bool,
}

/// A held connection slot.
pub type ConnectionLease = live::Held;

fn metric(outcome: &'static str) {
    metrics::counter!("mw_http_admission_total", "outcome" => outcome).increment(1);
}

impl Admission {
    pub fn new(limits: &HttpLimits) -> Arc<Self> {
        Self::build(limits, None)
    }

    /// With a fixed admin path (tests).
    pub fn with_admin_path(limits: &HttpLimits, admin_path: String) -> Arc<Self> {
        Self::build(limits, Some(admin_path))
    }

    fn build(limits: &HttpLimits, admin_path: Option<String>) -> Arc<Self> {
        let (live, sources) = live::resolve(limits, &Console::default());
        let admission = Arc::new(Self {
            limits: *limits,
            connections: Resizable::new(live.connections),
            requests: Resizable::new(live.requests),
            // A push stream holds its permit for as long as the client stays
            // connected; counted with ordinary requests, a few hundred open tabs
            // would leave no capacity for the requests those tabs make, so streams
            // have their own pool, as large as the connection limit.
            streams: Resizable::new(live.connections),
            probes: Arc::new(Semaphore::new(PROBE_PERMITS)),
            admin: Arc::new(Semaphore::new(ADMIN_PERMITS)),
            admin_path,
            queue_wait_ms: AtomicU64::new(live.queue_wait_ms),
            queue_max: AtomicUsize::new(live.queue_max),
            http2_streams: AtomicU32::new(live.http2_streams),
            waiting: AtomicUsize::new(0),
            inflight: AtomicUsize::new(0),
            counters: Counters::default(),
            live: Mutex::new((live, sources, Console::default())),
            adaptive: Mutex::new(None),
            adaptive_on: AtomicBool::new(false),
            store: OnceLock::new(),
            started: Instant::now(),
            next_check_ms: AtomicU64::new(0),
            checking: AtomicBool::new(false),
            revision: Mutex::new(None),
        });
        admission.apply(Console::default());
        admission
    }

    /// The store whose `server.http.*` keys this listener follows.
    pub fn attach_store(&self, store: mw_store::Store) {
        let _ = self.store.set(store);
    }

    /// Apply console values: resize the pools, set the queue and stream values,
    /// switch the adaptive limiter. Work in progress is never cut off.
    pub fn apply(&self, console: Console) {
        let (live, sources) = live::resolve(&self.limits, &console);
        self.queue_wait_ms
            .store(live.queue_wait_ms, Ordering::Relaxed);
        self.queue_max.store(live.queue_max, Ordering::Relaxed);
        self.http2_streams
            .store(live.http2_streams, Ordering::Relaxed);
        self.connections.set(live.connections);
        self.streams.set(live.connections);
        let mut adaptive = self.adaptive.lock().unwrap_or_else(|p| p.into_inner());
        if live.adaptive {
            match adaptive.as_mut() {
                Some(limiter) => limiter.set_bounds(live.adaptive_floor, live.requests),
                None => *adaptive = Some(Limiter::new(live.adaptive_floor, live.requests)),
            }
        } else {
            *adaptive = None;
        }
        self.adaptive_on.store(live.adaptive, Ordering::Relaxed);
        let target = adaptive
            .as_ref()
            .map_or(live.requests, |limiter| limiter.limit().min(live.requests));
        self.requests.set(target);
        drop(adaptive);
        *self.live.lock().unwrap_or_else(|p| p.into_inner()) = (live, sources, console);
    }

    /// Re-read the console values now (start-up), whatever the revision.
    pub async fn refresh_now(&self) {
        let Some(store) = self.store.get() else {
            return;
        };
        if let Some((revision, console)) = live::read_console(store).await {
            // The revision also moves for unrelated settings: resize and log only
            // when one of these keys' values changed.
            let before = self.live.lock().unwrap_or_else(|p| p.into_inner()).2;
            if console != before {
                self.apply(console);
                let (live, sources) = self.live();
                tracing::info!(
                    "HTTP limits changed from the admin console: {}",
                    live::describe(&live, &sources)
                );
            }
            *self.revision.lock().unwrap_or_else(|p| p.into_inner()) = Some(revision);
        }
    }

    /// Check the settings revision at most once per [`REFRESH_EVERY`], in a
    /// spawned task, and re-read the keys only when it changed.
    fn maybe_refresh(self: &Arc<Self>) {
        if self.store.get().is_none() {
            return;
        }
        let now = self.started.elapsed().as_millis() as u64;
        let next = self.next_check_ms.load(Ordering::Relaxed);
        if now < next || self.checking.swap(true, Ordering::AcqRel) {
            return;
        }
        self.next_check_ms
            .store(now + REFRESH_EVERY.as_millis() as u64, Ordering::Relaxed);
        let me = self.clone();
        tokio::spawn(async move {
            if let Some(store) = me.store.get() {
                let current = live::revision(store).await;
                let seen = *me.revision.lock().unwrap_or_else(|p| p.into_inner());
                // An unknown revision (None) is re-read every check.
                if current.is_some() && (seen != current || current == Some(None)) {
                    me.refresh_now().await;
                }
            }
            me.checking.store(false, Ordering::Release);
        });
    }

    /// HTTP/2 streams per connection for a connection accepted now.
    pub fn http2_streams(&self) -> u32 {
        self.http2_streams.load(Ordering::Relaxed)
    }

    /// A connection slot, or `None` when the connection limit is reached.
    pub fn try_connection(&self) -> Option<ConnectionLease> {
        match self.connections.try_acquire() {
            Some(permit) => Some(live::Held::new(permit, self.connections.clone())),
            None => {
                self.counters
                    .connections_refused
                    .fetch_add(1, Ordering::Relaxed);
                metric("connection_refused");
                None
            }
        }
    }

    /// A connection was refused because the connection limit was reached.
    pub fn connection_refused(&self) {
        self.counters
            .connections_refused
            .fetch_add(1, Ordering::Relaxed);
        metric("connection_refused");
    }

    /// Requests waiting for a slot now.
    pub fn waiting(&self) -> usize {
        self.waiting.load(Ordering::Relaxed)
    }

    /// The admin path in force now (`mw_admin::config::admin_path`, which the
    /// server sets at start from its configuration).
    pub fn admin_path(&self) -> String {
        self.admin_path
            .clone()
            .unwrap_or_else(mw_admin::config::admin_path)
    }

    fn lease(
        self: &Arc<Self>,
        permit: OwnedSemaphorePermit,
        from_requests: bool,
        sampled: bool,
    ) -> Lease {
        let inflight = if from_requests {
            self.inflight.fetch_add(1, Ordering::Relaxed) + 1
        } else {
            0
        };
        Lease {
            permit: Some(permit),
            _stream: None,
            admission: self.clone(),
            from_requests,
            admitted: Instant::now(),
            inflight,
            sampled: sampled && from_requests,
        }
    }

    /// Admit a request for `path` to `pool`, waiting if the request pool is full.
    pub async fn admit(
        self: &Arc<Self>,
        pool: Pool,
        sampled: bool,
        path: &str,
    ) -> Result<Lease, Refusal> {
        self.maybe_refresh();
        match pool {
            Pool::Streams => {
                return match self.streams.try_acquire() {
                    Some(permit) => Ok(Lease {
                        permit: None,
                        _stream: Some(live::Held::new(permit, self.streams.clone())),
                        admission: self.clone(),
                        from_requests: false,
                        admitted: Instant::now(),
                        inflight: 0,
                        sampled: false,
                    }),
                    None => {
                        self.counters
                            .refused_streams
                            .fetch_add(1, Ordering::Relaxed);
                        metric("refused_streams");
                        Err(Refusal::Streams)
                    }
                };
            }
            Pool::Probes => {
                return match self.probes.clone().try_acquire_owned() {
                    Ok(permit) => Ok(self.lease(permit, false, false)),
                    Err(_) => {
                        self.counters.refused_probes.fetch_add(1, Ordering::Relaxed);
                        metric("refused_probes");
                        Err(Refusal::Probes)
                    }
                };
            }
            Pool::Requests => {}
        }
        if let Some(permit) = self.requests.try_acquire() {
            self.counters.immediate.fetch_add(1, Ordering::Relaxed);
            metric("immediate");
            return Ok(self.lease(permit, true, sampled));
        }
        if is_admin_path(path, &self.admin_path())
            && let Ok(permit) = self.admin.clone().try_acquire_owned()
        {
            self.counters.admin_reserve.fetch_add(1, Ordering::Relaxed);
            metric("admin_reserve");
            return Ok(self.lease(permit, false, false));
        }
        let queue_wait = Duration::from_millis(self.queue_wait_ms.load(Ordering::Relaxed));
        if queue_wait.is_zero() {
            return Err(self.refuse_deadline());
        }
        if self.waiting.fetch_add(1, Ordering::AcqRel) >= self.queue_max.load(Ordering::Relaxed) {
            self.waiting.fetch_sub(1, Ordering::AcqRel);
            self.counters
                .refused_queue_full
                .fetch_add(1, Ordering::Relaxed);
            metric("refused_queue_full");
            return Err(Refusal::QueueFull);
        }
        // Dropped on every exit, including cancellation of this future.
        struct Waiting<'a>(&'a AtomicUsize);
        impl Drop for Waiting<'_> {
            fn drop(&mut self) {
                self.0.fetch_sub(1, Ordering::AcqRel);
            }
        }
        let _slot = Waiting(&self.waiting);
        let started = Instant::now();
        match tokio::time::timeout(queue_wait, self.requests.acquire()).await {
            Ok(Ok(permit)) => {
                let waited = started.elapsed();
                self.counters.after_wait.fetch_add(1, Ordering::Relaxed);
                metric("after_wait");
                let ms = waited.as_millis() as u64;
                let bucket = WAIT_BUCKETS_MS
                    .iter()
                    .position(|&top| ms <= top)
                    .unwrap_or(WAIT_BUCKETS_MS.len());
                self.counters.wait_buckets[bucket].fetch_add(1, Ordering::Relaxed);
                Ok(self.lease(permit, true, sampled))
            }
            // A closed semaphore cannot happen (it is never closed); treat as refused.
            Ok(Err(_)) | Err(_) => Err(self.refuse_deadline()),
        }
    }

    fn refuse_deadline(&self) -> Refusal {
        self.counters
            .refused_deadline
            .fetch_add(1, Ordering::Relaxed);
        metric("refused_deadline");
        Refusal::Deadline
    }

    /// The values in force and their sources.
    pub fn live(&self) -> (Live, Sources) {
        let guard = self.live.lock().unwrap_or_else(|p| p.into_inner());
        (guard.0, guard.1)
    }

    /// Effective values, counters and adaptive state, for diagnostics.
    pub fn snapshot(&self) -> Value {
        let c = &self.counters;
        let load = |n: &AtomicU64| n.load(Ordering::Relaxed);
        let mut buckets = serde_json::Map::new();
        for (i, top) in WAIT_BUCKETS_MS.iter().enumerate() {
            buckets.insert(format!("le{top}ms"), json!(load(&c.wait_buckets[i])));
        }
        buckets.insert(
            "over2000ms".into(),
            json!(load(&c.wait_buckets[WAIT_BUCKETS_MS.len()])),
        );
        let adaptive = self
            .adaptive
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
            .map(|limiter| {
                json!({
                    "limit": limiter.limit(),
                    "floor": limiter.floor(),
                    "ceiling": limiter.ceiling(),
                    "baselineUs": limiter.baseline().map(|d| d.as_micros() as u64),
                    "decreases": limiter.decreases(),
                    "increases": limiter.increases(),
                })
            });
        let (live, sources) = self.live();
        json!({
            "effective": live::describe(&live, &sources),
            "safetyBounds": {
                "connections": live::connections_cap(&self.limits),
                "requests": live::requests_cap(&self.limits),
            },
            "inputs": self.limits.inputs(),
            "warnings": self.limits.warnings(),
            "admittedImmediately": load(&c.immediate),
            "admittedAfterWait": load(&c.after_wait),
            "admittedFromAdminReserve": load(&c.admin_reserve),
            "waitBuckets": buckets,
            "refusedOnDeadline": load(&c.refused_deadline),
            "refusedOnFullQueue": load(&c.refused_queue_full),
            "refusedStreams": load(&c.refused_streams),
            "refusedProbes": load(&c.refused_probes),
            "connectionsRefused": load(&c.connections_refused),
            "waitingNow": self.waiting(),
            "inFlight": self.inflight.load(Ordering::Relaxed),
            "requestPermits": self.requests.size(),
            "connectionPermits": self.connections.size(),
            "adaptive": adaptive,
        })
    }

    /// The adaptive limit now, when the limiter is on.
    pub fn adaptive_limit(&self) -> Option<usize> {
        self.adaptive
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
            .map(Limiter::limit)
    }

    #[cfg(test)]
    pub(crate) fn request_pool(&self) -> &Resizable {
        &self.requests
    }
}

impl Lease {
    /// The response headers are ready: feed the in-server time to the limiter.
    pub fn headers_ready(&mut self) {
        if !std::mem::take(&mut self.sampled) || !self.admission.adaptive_on.load(Ordering::Relaxed)
        {
            return;
        }
        let latency = self.admitted.elapsed();
        let admission = &self.admission;
        let mut adaptive = admission.adaptive.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(limiter) = adaptive.as_mut()
            && let Some(target) = limiter.sample(latency, self.inflight)
        {
            let ceiling = admission.live().0.requests;
            admission.requests.set(target.min(ceiling));
        }
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        let Some(permit) = self.permit.take() else {
            return;
        };
        if self.from_requests {
            self.admission.inflight.fetch_sub(1, Ordering::Relaxed);
            self.admission.requests.release(permit);
        }
    }
}

// ---- the running listener, for diagnostics ---------------------------------

static CURRENT: Mutex<Option<(HttpLimits, Weak<Admission>)>> = Mutex::new(None);

/// Record the listener that is serving now (the latest `serve` call wins).
pub fn register(limits: HttpLimits, admission: &Arc<Admission>) {
    if let Ok(mut slot) = CURRENT.lock() {
        *slot = Some((limits, Arc::downgrade(admission)));
    }
}

/// The effective limits and counters of the running listener, if one is serving.
pub fn current() -> Option<(HttpLimits, Value)> {
    let admission = current_admission()?;
    let limits = admission.limits;
    Some((limits, admission.snapshot()))
}

/// The running listener's admission, if one is serving.
pub fn current_admission() -> Option<Arc<Admission>> {
    let slot = CURRENT.lock().ok()?;
    slot.as_ref()?.1.upgrade()
}

#[cfg(test)]
#[path = "http_admission_tests.rs"]
mod tests;
