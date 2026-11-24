//! Listener limits that change while the server runs: the admin-console keys
//! (`server.http.*`), how they combine with the environment and the derived
//! values, and the resizable semaphore that applies a new size without touching
//! work in progress.
//!
//! Precedence for each value: a set environment variable, then the console value,
//! then Auto (derived at start-up from the resources, see `resources`). A console
//! value is held to the safety bounds the start-up readings give: connections to
//! the descriptor bound (F − R) / 2 (at least 1024), requests to the memory bound
//! M / 4 MiB (256 to 16384). Each server process applies the values on its own.
use super::limits::{HttpLimits, QUEUE_MAX_CEILING, QUEUE_WAIT_MAX_MS, Source};
use super::resources::{
    self, CONNECTIONS_CEILING, CONNECTIONS_FLOOR, REQUESTS_CEILING, REQUESTS_FLOOR,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tokio::sync::{AcquireError, OwnedSemaphorePermit, Semaphore};

pub const MAX_CONNECTIONS: &str = "server.http.maxConnections";
pub const MAX_REQUESTS: &str = "server.http.maxRequests";
pub const HTTP2_MAX_STREAMS: &str = "server.http.http2MaxStreams";
pub const REQUEST_QUEUE_MS: &str = "server.http.requestQueueMs";
pub const REQUEST_QUEUE_MAX: &str = "server.http.requestQueueMax";
pub const ADAPTIVE: &str = "server.http.adaptiveConcurrency";
pub const ADAPTIVE_FLOOR: &str = "server.http.adaptiveFloor";
pub const RAISE_OPEN_FILES: &str = "server.http.raiseOpenFileLimit";
/// Every `server.http.*` key this module reads.
pub const KEYS: [&str; 8] = [
    MAX_CONNECTIONS,
    MAX_REQUESTS,
    HTTP2_MAX_STREAMS,
    REQUEST_QUEUE_MS,
    REQUEST_QUEUE_MAX,
    ADAPTIVE,
    ADAPTIVE_FLOOR,
    RAISE_OPEN_FILES,
];

/// The console values; `None` = Auto (or the default).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Console {
    pub connections: Option<u64>,
    pub requests: Option<u64>,
    pub http2_streams: Option<u64>,
    pub queue_wait_ms: Option<u64>,
    pub queue_max: Option<u64>,
    pub adaptive: Option<bool>,
    pub adaptive_floor: Option<u64>,
    pub raise_open_files: Option<bool>,
}

impl Console {
    /// From `(key, value)` pairs (registry rows); `null` or a missing key is Auto.
    pub fn from_pairs<'a>(rows: impl IntoIterator<Item = (&'a str, &'a Value)>) -> Self {
        let mut out = Self::default();
        for (key, value) in rows {
            let n = value.as_u64();
            let b = value.as_bool();
            match key {
                MAX_CONNECTIONS => out.connections = n,
                MAX_REQUESTS => out.requests = n,
                HTTP2_MAX_STREAMS => out.http2_streams = n,
                REQUEST_QUEUE_MS => out.queue_wait_ms = n,
                REQUEST_QUEUE_MAX => out.queue_max = n,
                ADAPTIVE => out.adaptive = b,
                ADAPTIVE_FLOOR => out.adaptive_floor = n,
                RAISE_OPEN_FILES => out.raise_open_files = b,
                _ => {}
            }
        }
        out
    }
}

/// The values in force.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Live {
    pub connections: usize,
    pub requests: usize,
    pub http2_streams: u32,
    pub queue_wait_ms: u64,
    pub queue_max: usize,
    pub adaptive: bool,
    pub adaptive_floor: usize,
}

/// Where each [`Live`] value came from, and whether a safety bound cut a console
/// value down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sources {
    pub connections: Source,
    pub requests: Source,
    pub http2_streams: Source,
    pub queue_wait_ms: Source,
    pub queue_max: Source,
    pub adaptive: Source,
    pub adaptive_floor: Source,
    pub connections_capped: bool,
    pub requests_capped: bool,
}

/// The most connections a console value may set: the descriptor bound when the
/// open-file limit is known (never below the 1024 floor), else the ceiling.
pub fn connections_cap(limits: &HttpLimits) -> usize {
    limits
        .derivation
        .auto
        .and_then(|a| a.fd_bound)
        .map_or(CONNECTIONS_CEILING, |bound| {
            (bound.min(CONNECTIONS_CEILING as u64) as usize).max(CONNECTIONS_FLOOR)
        })
}

/// The most requests a console value may set: the memory bound when memory is
/// known (256 to 16384), else the ceiling.
pub fn requests_cap(limits: &HttpLimits) -> usize {
    limits
        .derivation
        .auto
        .and_then(|a| a.memory_request_bound)
        .map_or(REQUESTS_CEILING, |bound| {
            (bound.min(REQUESTS_CEILING as u64) as usize).max(REQUESTS_FLOOR)
        })
}

/// Combine the start-up limits (environment and Auto) with the console values.
pub fn resolve(limits: &HttpLimits, console: &Console) -> (Live, Sources) {
    let d = &limits.derivation;
    let pick =
        |env: Source, start: usize, console: Option<u64>, cap: usize| -> (usize, Source, bool) {
            match (env, console) {
                (Source::Explicit, _) => (start, Source::Explicit, false),
                (_, Some(v)) => {
                    let v = v as usize;
                    (v.min(cap), Source::Console, v > cap)
                }
                (source, None) => (start, source, false),
            }
        };
    let (connections, connections_source, connections_capped) = pick(
        d.connections,
        limits.connections,
        console.connections,
        connections_cap(limits),
    );
    let (requests, requests_source, requests_capped) = pick(
        d.requests,
        limits.requests,
        console.requests,
        requests_cap(limits),
    );
    // Unset streams and queue follow the request limit in force.
    let follows = requests_source == Source::Console;
    let (http2_streams, streams_source) = match (d.http2_streams, console.http2_streams) {
        (Source::Explicit, _) => (limits.http2_streams, Source::Explicit),
        (_, Some(v)) => ((v as u32).clamp(1, 128), Source::Console),
        (source, None) if follows || source == Source::Auto => {
            (resources::streams_for(requests), Source::Auto)
        }
        (source, None) => (limits.http2_streams, source),
    };
    let (queue_wait_ms, queue_wait_source) = match (d.queue_wait, console.queue_wait_ms) {
        (Source::Explicit, _) => (limits.queue_wait.as_millis() as u64, Source::Explicit),
        (_, Some(v)) => (v.min(QUEUE_WAIT_MAX_MS), Source::Console),
        (source, None) => (limits.queue_wait.as_millis() as u64, source),
    };
    let (queue_max, queue_source) = match (d.queue_max, console.queue_max) {
        (Source::Explicit, _) => (limits.queue_max, Source::Explicit),
        (_, Some(v)) => ((v as usize).clamp(1, QUEUE_MAX_CEILING), Source::Console),
        (source, None) if follows || source == Source::Auto => {
            (requests.clamp(1, QUEUE_MAX_CEILING), Source::Auto)
        }
        (source, None) => (limits.queue_max, source),
    };
    let (adaptive, adaptive_source) = match (d.adaptive, console.adaptive) {
        (Source::Explicit, _) => (limits.adaptive, Source::Explicit),
        (_, Some(v)) => (v, Source::Console),
        (source, None) => (limits.adaptive, source),
    };
    let (adaptive_floor, floor_source) = match console.adaptive_floor {
        Some(v) => ((v as usize).clamp(1, requests.max(1)), Source::Console),
        None => (
            resources::adaptive_floor(d.resources.cpus, requests),
            Source::Auto,
        ),
    };
    (
        Live {
            connections,
            requests,
            http2_streams,
            queue_wait_ms,
            queue_max,
            adaptive,
            adaptive_floor,
        },
        Sources {
            connections: connections_source,
            requests: requests_source,
            http2_streams: streams_source,
            queue_wait_ms: queue_wait_source,
            queue_max: queue_source,
            adaptive: adaptive_source,
            adaptive_floor: floor_source,
            connections_capped,
            requests_capped,
        },
    )
}

/// The values in force with their sources, for diagnostics and the console.
pub fn describe(live: &Live, sources: &Sources) -> Value {
    json!({
        "connections": {"value": live.connections, "source": sources.connections.as_str(), "capped": sources.connections_capped},
        "requests": {"value": live.requests, "source": sources.requests.as_str(), "capped": sources.requests_capped},
        "http2Streams": {"value": live.http2_streams, "source": sources.http2_streams.as_str()},
        "queueWaitMs": {"value": live.queue_wait_ms, "source": sources.queue_wait_ms.as_str()},
        "queueMax": {"value": live.queue_max, "source": sources.queue_max.as_str()},
        "adaptive": {"value": live.adaptive, "source": sources.adaptive.as_str()},
        "adaptiveFloor": {"value": live.adaptive_floor, "source": sources.adaptive_floor.as_str()},
    })
}

/// The console values from the settings store at its current revision; `None`
/// when they cannot be read (the previous values stay).
pub async fn read_console(store: &mw_store::Store) -> Option<(Option<i64>, Console)> {
    let revision = store.settings_read_revision().await.ok()?;
    let subject = mw_store::settings_policy::SettingSubject::default();
    let rows = crate::settings_registry::effective_all_at_revision(
        store,
        &subject,
        |def| KEYS.contains(&def.key),
        &revision,
    )
    .await
    .ok()?;
    Some((
        revision.number(),
        Console::from_pairs(rows.iter().map(|row| (row.key.as_str(), &row.value))),
    ))
}

/// The settings revision now, without reading any key.
pub async fn revision(store: &mw_store::Store) -> Option<Option<i64>> {
    store
        .settings_read_revision()
        .await
        .ok()
        .map(|r| r.number())
}

/// The console's open-file choice, read at start.
pub async fn stored_raise(store: &mw_store::Store) -> Option<bool> {
    read_console(store)
        .await
        .and_then(|(_, c)| c.raise_open_files)
}

// ---- resizable semaphore ---------------------------------------------------

/// A semaphore whose size can change while permits are out. Growing adds
/// permits at once; shrinking takes idle permits at once and the rest as they
/// come back (`debt`), so nothing in progress is cut off, and the number of
/// permits never exceeds the size last set.
pub struct Resizable {
    sem: Arc<Semaphore>,
    size: Mutex<Size>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Size {
    /// Permits in existence (idle plus held).
    existing: usize,
    /// Held permits to destroy when they come back.
    debt: usize,
}

impl Resizable {
    pub fn new(size: usize) -> Arc<Self> {
        Arc::new(Self {
            sem: Arc::new(Semaphore::new(size)),
            size: Mutex::new(Size {
                existing: size,
                debt: 0,
            }),
        })
    }
    fn lock(&self) -> std::sync::MutexGuard<'_, Size> {
        self.size.lock().unwrap_or_else(|p| p.into_inner())
    }
    /// The size last set.
    pub fn size(&self) -> usize {
        let size = self.lock();
        size.existing - size.debt
    }
    pub fn available(&self) -> usize {
        self.sem.available_permits()
    }
    pub fn set(&self, target: usize) {
        let mut size = self.lock();
        let planned = size.existing - size.debt;
        if target < planned {
            size.debt += planned - target;
            while size.debt > 0 {
                let Ok(permit) = self.sem.try_acquire() else {
                    break;
                };
                permit.forget();
                size.debt -= 1;
                size.existing -= 1;
            }
        } else if target > planned {
            let mut give = target - planned;
            let cancel = give.min(size.debt);
            size.debt -= cancel;
            give -= cancel;
            size.existing += give;
            self.sem.add_permits(give);
        }
    }
    pub fn try_acquire(&self) -> Option<OwnedSemaphorePermit> {
        self.sem.clone().try_acquire_owned().ok()
    }
    pub async fn acquire(&self) -> Result<OwnedSemaphorePermit, AcquireError> {
        self.sem.clone().acquire_owned().await
    }
    /// Return a permit: destroyed while the pool is shrinking, else released.
    pub fn release(&self, permit: OwnedSemaphorePermit) {
        let mut size = self.lock();
        if size.debt > 0 {
            size.debt -= 1;
            size.existing -= 1;
            permit.forget();
        } else {
            drop(size);
            drop(permit);
        }
    }
}

/// A held permit of a [`Resizable`] pool, returned through it on drop.
pub struct Held {
    permit: Option<OwnedSemaphorePermit>,
    pool: Arc<Resizable>,
}
impl Held {
    pub fn new(permit: OwnedSemaphorePermit, pool: Arc<Resizable>) -> Self {
        Self {
            permit: Some(permit),
            pool,
        }
    }
}
impl Drop for Held {
    fn drop(&mut self) {
        if let Some(permit) = self.permit.take() {
            self.pool.release(permit);
        }
    }
}

#[cfg(test)]
#[path = "http_live_tests.rs"]
mod tests;
