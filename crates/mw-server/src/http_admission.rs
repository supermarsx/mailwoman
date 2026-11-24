//! Request admission for the built-in listener: the request pool with its bounded
//! FIFO wait, the event-stream pool, the probe pool and the counters.
//!
//! * Request pool: `requests` permits. A request that finds none waits in FIFO
//!   order (tokio's semaphore is fair: a freed permit goes to the oldest waiter, and
//!   a new arrival cannot take it first) for up to `queue_wait`, with at most
//!   `queue_max` waiting; past either it gets 503 + `Retry-After: 1`. A client that
//!   disconnects while waiting drops its wait, which frees its queue slot and its
//!   place in the semaphore's queue.
//! * Event streams (`/jmap/eventsource`): their own pool, no wait (unchanged).
//! * Health probes (`/healthz`, `/readyz`): their own pool of [`PROBE_PERMITS`], no
//!   wait, so a full request pool or queue never delays them.
//! * WebSocket upgrades take a request permit only until the `101` response; the
//!   socket afterwards holds its connection permit and nothing in this module.
use super::limits::HttpLimits;
use serde_json::{Value, json};
use std::{
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Concurrent health and readiness probes.
pub const PROBE_PERMITS: usize = 8;
/// Upper bounds (ms) of the wait-time buckets; the last bucket is everything above.
pub const WAIT_BUCKETS_MS: [u64; 8] = [10, 25, 50, 100, 250, 500, 1000, 2000];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pool {
    Requests,
    Streams,
    Probes,
}

/// Which pool a path uses.
pub fn classify(path: &str) -> Pool {
    if path.ends_with("/jmap/eventsource") {
        Pool::Streams
    } else if path == "/healthz" || path == "/readyz" {
        Pool::Probes
    } else {
        Pool::Requests
    }
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
    wait_buckets: [AtomicU64; WAIT_BUCKETS_MS.len() + 1],
    refused_deadline: AtomicU64,
    refused_queue_full: AtomicU64,
    refused_streams: AtomicU64,
    refused_probes: AtomicU64,
    connections_refused: AtomicU64,
}

pub struct Admission {
    requests: Arc<Semaphore>,
    streams: Arc<Semaphore>,
    probes: Arc<Semaphore>,
    queue_wait: Duration,
    queue_max: usize,
    waiting: AtomicUsize,
    inflight: AtomicUsize,
    counters: Counters,
}

/// A held admission; the slot frees when this drops (with the response body).
pub struct Lease {
    permit: Option<OwnedSemaphorePermit>,
    admission: Arc<Admission>,
    pool: Pool,
}

fn metric(outcome: &'static str) {
    metrics::counter!("mw_http_admission_total", "outcome" => outcome).increment(1);
}

impl Admission {
    pub fn new(limits: &HttpLimits) -> Arc<Self> {
        Arc::new(Self {
            requests: Arc::new(Semaphore::new(limits.requests)),
            // A push stream holds its permit for as long as the client stays
            // connected; counted with ordinary requests, a few hundred open tabs
            // would leave no capacity for the requests those tabs make, so streams
            // have their own pool, as large as the connection limit.
            streams: Arc::new(Semaphore::new(limits.connections)),
            probes: Arc::new(Semaphore::new(PROBE_PERMITS)),
            queue_wait: limits.queue_wait,
            queue_max: limits.queue_max,
            waiting: AtomicUsize::new(0),
            inflight: AtomicUsize::new(0),
            counters: Counters::default(),
        })
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

    /// Admit a request to `pool`, waiting if the request pool is full.
    pub async fn admit(self: &Arc<Self>, pool: Pool) -> Result<Lease, Refusal> {
        let lease = |permit| Lease {
            permit: Some(permit),
            admission: self.clone(),
            pool,
        };
        let semaphore = match pool {
            Pool::Streams => &self.streams,
            Pool::Probes => &self.probes,
            Pool::Requests => &self.requests,
        };
        if let Ok(permit) = semaphore.clone().try_acquire_owned() {
            if pool != Pool::Requests {
                return Ok(lease(permit));
            }
            self.counters.immediate.fetch_add(1, Ordering::Relaxed);
            metric("immediate");
            self.inflight.fetch_add(1, Ordering::Relaxed);
            return Ok(lease(permit));
        }
        match pool {
            Pool::Streams => {
                self.counters
                    .refused_streams
                    .fetch_add(1, Ordering::Relaxed);
                metric("refused_streams");
                return Err(Refusal::Streams);
            }
            Pool::Probes => {
                self.counters.refused_probes.fetch_add(1, Ordering::Relaxed);
                metric("refused_probes");
                return Err(Refusal::Probes);
            }
            Pool::Requests => {}
        }
        if self.queue_wait.is_zero() {
            return Err(self.refuse_deadline());
        }
        if self.waiting.fetch_add(1, Ordering::AcqRel) >= self.queue_max {
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
        match tokio::time::timeout(self.queue_wait, self.requests.clone().acquire_owned()).await {
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
                self.inflight.fetch_add(1, Ordering::Relaxed);
                Ok(lease(permit))
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

    /// Effective counters, for diagnostics.
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
        json!({
            "admittedImmediately": load(&c.immediate),
            "admittedAfterWait": load(&c.after_wait),
            "waitBuckets": buckets,
            "refusedOnDeadline": load(&c.refused_deadline),
            "refusedOnFullQueue": load(&c.refused_queue_full),
            "refusedStreams": load(&c.refused_streams),
            "refusedProbes": load(&c.refused_probes),
            "connectionsRefused": load(&c.connections_refused),
            "waitingNow": self.waiting(),
            "inFlight": self.inflight.load(Ordering::Relaxed),
        })
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        if self.pool != Pool::Requests {
            return;
        }
        self.admission.inflight.fetch_sub(1, Ordering::Relaxed);
        drop(self.permit.take());
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
    let slot = CURRENT.lock().ok()?;
    let (limits, admission) = slot.as_ref()?;
    let admission = admission.upgrade()?;
    Some((*limits, admission.snapshot()))
}

#[cfg(test)]
#[path = "http_admission_tests.rs"]
mod tests;
