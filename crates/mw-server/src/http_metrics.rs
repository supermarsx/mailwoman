//! Request metrics by route class (t32-admin §2.3, f08/e13; mini-slot
//! `t32-http-metrics`, handed to f09/e78 for per-route-template labels).
//!
//! [`layer`] is a middleware mounted once around the whole app (`lib.rs`). For
//! every response it records
//!
//! * `mw_http_requests_total{class, status}` (status = `2xx`/`3xx`/`4xx`/`5xx`/`1xx`)
//!   and `mw_http_request_duration_seconds{class}` on the Prometheus recorder
//!   (`/metrics`);
//! * counters since the process started, per class ([`summary`]);
//! * a ring of the last 60 one-minute buckets, per class: requests, 4xx, 5xx and a
//!   fixed-bucket latency histogram from which p50/p95 are read ([`traffic_series`]).
//!
//! Everything here is per process: a deployment with several replicas has one ring
//! per replica, and the API says so (`perProcess: true`).
//!
//! **Labels.** The class is one of seven fixed words ([`RouteClass`]); no path,
//! address, account, query text or token ever becomes a label.
//!
//! **Percentiles** are the upper bound of the latency bucket that holds the rank
//! (buckets in [`BUCKETS_MS`]), so a p95 of `250` means "at most 250 ms". A minute
//! with no requests reports `0`.

use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use serde::Serialize;

/// The route classes (t32-admin §2.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RouteClass {
    Login,
    Twofa,
    Jmap,
    Upload,
    Admin,
    Api,
    Static,
}

impl RouteClass {
    /// Every class, in a fixed order (the ring's index order).
    pub const ALL: [RouteClass; 7] = [
        RouteClass::Login,
        RouteClass::Twofa,
        RouteClass::Jmap,
        RouteClass::Upload,
        RouteClass::Admin,
        RouteClass::Api,
        RouteClass::Static,
    ];

    /// The wire and label form.
    pub const fn as_str(self) -> &'static str {
        match self {
            RouteClass::Login => "login",
            RouteClass::Twofa => "twofa",
            RouteClass::Jmap => "jmap",
            RouteClass::Upload => "upload",
            RouteClass::Admin => "admin",
            RouteClass::Api => "api",
            RouteClass::Static => "static",
        }
    }

    const fn index(self) -> usize {
        self as usize
    }
}

/// Does `path` equal `prefix` or continue it with `/`?
fn under(path: &str, prefix: &str) -> bool {
    path.strip_prefix(prefix)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

/// The class of a request path (the deploy prefix already removed by the nest).
pub fn classify(path: &str) -> RouteClass {
    if under(path, "/api/login/2fa") || under(path, "/api/account/2fa") {
        return RouteClass::Twofa;
    }
    if under(path, "/api/login")
        || under(path, "/api/logout")
        || under(path, "/api/discover")
        || under(path, "/api/sso")
        || under(path, "/oauth")
    {
        return RouteClass::Login;
    }
    if under(path, "/jmap/upload") {
        return RouteClass::Upload;
    }
    if under(path, "/jmap") || under(path, "/.well-known/jmap") {
        return RouteClass::Jmap;
    }
    if under(path, &mw_admin::config::admin_path()) {
        return RouteClass::Admin;
    }
    if under(path, "/api")
        || under(path, "/dav")
        || under(path, "/mcp")
        || under(path, "/metrics")
        || under(path, "/healthz")
        || under(path, "/.well-known")
    {
        return RouteClass::Api;
    }
    RouteClass::Static
}

/// Upper bounds (ms) of the latency buckets; the last bucket is unbounded and
/// reports the largest observed latency, preserving an upper bound.
pub const BUCKETS_MS: [u64; 13] = [1, 2, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000];
const NBUCKETS: usize = BUCKETS_MS.len() + 1;
/// Minutes kept in the ring.
pub const WINDOW_MINS: usize = 60;
const NCLASSES: usize = RouteClass::ALL.len();

#[derive(Clone, Copy, Default)]
struct Cell {
    requests: u64,
    errors4xx: u64,
    errors5xx: u64,
    hist: [u64; NBUCKETS],
    max_elapsed_ms: u64,
}

impl Cell {
    fn add(&mut self, status: u16, ms: u64) {
        self.requests += 1;
        self.max_elapsed_ms = self.max_elapsed_ms.max(ms);
        if (400..500).contains(&status) {
            self.errors4xx += 1;
        } else if status >= 500 {
            self.errors5xx += 1;
        }
        let b = BUCKETS_MS
            .iter()
            .position(|&ub| ms <= ub)
            .unwrap_or(NBUCKETS - 1);
        self.hist[b] += 1;
    }

    fn merge(&mut self, o: &Cell) {
        self.requests += o.requests;
        self.max_elapsed_ms = self.max_elapsed_ms.max(o.max_elapsed_ms);
        self.errors4xx += o.errors4xx;
        self.errors5xx += o.errors5xx;
        for (a, b) in self.hist.iter_mut().zip(o.hist.iter()) {
            *a += b;
        }
    }

    /// The bucket bound holding the `q` quantile (0 when empty).
    fn quantile_ms(&self, q: f64) -> u64 {
        let total: u64 = self.hist.iter().sum();
        if total == 0 {
            return 0;
        }
        let rank = ((total as f64) * q).ceil().max(1.0) as u64;
        let mut seen = 0;
        for (i, n) in self.hist.iter().enumerate() {
            seen += n;
            if seen >= rank {
                return BUCKETS_MS.get(i).copied().unwrap_or(self.max_elapsed_ms);
            }
        }
        self.max_elapsed_ms
    }
}

#[derive(Clone, Copy, Default)]
struct Slot {
    /// Unix minute this slot holds; a slot from an older minute reads as empty.
    minute: u64,
    cells: [Cell; NCLASSES],
}

struct Metrics {
    started_at: u64,
    totals: [Cell; NCLASSES],
    ring: [Slot; WINDOW_MINS],
}

static METRICS: Mutex<Option<Metrics>> = Mutex::new(None);

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn with_metrics<R>(f: impl FnOnce(&mut Metrics) -> R) -> R {
    let mut g = METRICS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let m = g.get_or_insert_with(|| Metrics {
        started_at: now_secs(),
        totals: [Cell::default(); NCLASSES],
        ring: [Slot::default(); WINDOW_MINS],
    });
    f(m)
}

/// Record one finished request (also used by tests).
pub fn record(class: RouteClass, status: u16, elapsed_ms: u64) {
    record_at(now_secs(), class, status, elapsed_ms);
}

fn record_at(now: u64, class: RouteClass, status: u16, elapsed_ms: u64) {
    let status_class = match status {
        100..=199 => "1xx",
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        _ => "5xx",
    };
    metrics::counter!("mw_http_requests_total", "class" => class.as_str(), "status" => status_class)
        .increment(1);
    metrics::histogram!("mw_http_request_duration_seconds", "class" => class.as_str())
        .record(elapsed_ms as f64 / 1000.0);
    let minute = now / 60;
    with_metrics(|m| {
        m.totals[class.index()].add(status, elapsed_ms);
        let slot = &mut m.ring[(minute % WINDOW_MINS as u64) as usize];
        if slot.minute != minute {
            *slot = Slot {
                minute,
                cells: [Cell::default(); NCLASSES],
            };
        }
        slot.cells[class.index()].add(status, elapsed_ms);
    });
}

/// The middleware: times the request and records it under its class.
pub async fn layer(req: Request, next: Next) -> Response {
    let class = classify(req.uri().path());
    let start = Instant::now();
    let resp = next.run(req).await;
    let ms = start.elapsed().as_millis().min(u64::MAX as u128) as u64;
    record(class, resp.status().as_u16(), ms);
    resp
}

/// Since-start counters of one class.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClassTotals {
    pub class: RouteClass,
    pub requests: u64,
    pub errors4xx: u64,
    pub errors5xx: u64,
}

/// One minute of one class (or of all classes).
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    pub requests: u64,
    pub errors4xx: u64,
    pub errors5xx: u64,
    pub p50_ms: u64,
    pub p95_ms: u64,
}

impl From<&Cell> for Point {
    fn from(c: &Cell) -> Self {
        Point {
            requests: c.requests,
            errors4xx: c.errors4xx,
            errors5xx: c.errors5xx,
            p50_ms: c.quantile_ms(0.50),
            p95_ms: c.quantile_ms(0.95),
        }
    }
}

/// One minute bucket: all classes together, and each class.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Minute {
    /// Start of the minute, Unix seconds.
    pub t: u64,
    #[serde(flatten)]
    pub all: Point,
    pub by_class: std::collections::BTreeMap<&'static str, Point>,
}

/// `GET /admin/metrics/summary`'s body.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub per_process: bool,
    /// When this process started counting, Unix seconds.
    pub since: u64,
    pub window_mins: usize,
    pub bucket_secs: u64,
    pub latency_buckets_ms: &'static [u64],
    pub totals: Vec<ClassTotals>,
    /// Oldest first; minutes with no traffic are present with zeros.
    pub series: Vec<Minute>,
}

/// The counters and the ring as they are now.
pub fn summary() -> Summary {
    summary_at(now_secs())
}

fn summary_at(now: u64) -> Summary {
    let this_minute = now / 60;
    with_metrics(|m| {
        let totals = RouteClass::ALL
            .iter()
            .map(|c| {
                let t = &m.totals[c.index()];
                ClassTotals {
                    class: *c,
                    requests: t.requests,
                    errors4xx: t.errors4xx,
                    errors5xx: t.errors5xx,
                }
            })
            .collect();
        let mut series = Vec::with_capacity(WINDOW_MINS);
        for back in (0..WINDOW_MINS as u64).rev() {
            let Some(minute) = this_minute.checked_sub(back) else {
                continue;
            };
            let slot = &m.ring[(minute % WINDOW_MINS as u64) as usize];
            let empty = [Cell::default(); NCLASSES];
            let cells = if slot.minute == minute {
                &slot.cells
            } else {
                &empty
            };
            let mut all = Cell::default();
            let mut by_class = std::collections::BTreeMap::new();
            for c in RouteClass::ALL {
                all.merge(&cells[c.index()]);
                by_class.insert(c.as_str(), Point::from(&cells[c.index()]));
            }
            series.push(Minute {
                t: minute * 60,
                all: Point::from(&all),
                by_class,
            });
        }
        Summary {
            per_process: true,
            since: m.started_at,
            window_mins: WINDOW_MINS,
            bucket_secs: 60,
            latency_buckets_ms: &BUCKETS_MS,
            totals,
            series,
        }
    })
}

/// Register `GET /admin/metrics/summary` (mini-slot `t32-admin-registry`).
pub(crate) fn register(r: &mut impl crate::admin_router::AdminRegistrar) {
    r.admin_endpoint(
        axum::http::Method::GET,
        "/metrics/summary",
        crate::capabilities::Capability::ObservabilityRead,
        summary_route,
    );
}

/// `GET /admin/metrics/summary` — [`summary`] as JSON.
async fn summary_route(_ctx: crate::admin_ctx::AdminCtx) -> impl axum::response::IntoResponse {
    ([(axum::http::header::CACHE_CONTROL, "no-store")], axum::Json(summary()))
}

/// The dashboard's `traffic.series`: `{t, requests, errors5xx, p50Ms, p95Ms}` per
/// minute over all classes, oldest first.
pub fn traffic_series() -> Vec<serde_json::Value> {
    summary()
        .series
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "t": m.t,
                "requests": m.all.requests,
                "errors5xx": m.all.errors5xx,
                "p50Ms": m.all.p50_ms,
                "p95Ms": m.all.p95_ms,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_follow_the_path_on_segment_boundaries() {
        assert_eq!(classify("/api/login"), RouteClass::Login);
        assert_eq!(classify("/api/login2"), RouteClass::Api);
        assert_eq!(classify("/api/login/2fa"), RouteClass::Twofa);
        assert_eq!(classify("/api/account/2fa/totp/begin"), RouteClass::Twofa);
        assert_eq!(classify("/api/sso/x/callback"), RouteClass::Login);
        assert_eq!(classify("/jmap/upload/a1"), RouteClass::Upload);
        assert_eq!(classify("/jmap"), RouteClass::Jmap);
        assert_eq!(classify("/jmap/ws"), RouteClass::Jmap);
        assert_eq!(classify("/admin/users"), RouteClass::Admin);
        assert_eq!(classify("/administrator"), RouteClass::Static);
        assert_eq!(classify("/api/v1/messages"), RouteClass::Api);
        assert_eq!(classify("/healthz"), RouteClass::Api);
        assert_eq!(classify("/assets/app.js"), RouteClass::Static);
        assert_eq!(classify("/"), RouteClass::Static);
    }

    #[test]
    fn quantiles_read_the_bucket_bound() {
        let mut c = Cell::default();
        for _ in 0..90 {
            c.add(200, 3); // bucket <= 5
        }
        for _ in 0..10 {
            c.add(500, 400); // bucket <= 500
        }
        assert_eq!(c.quantile_ms(0.5), 5);
        assert_eq!(c.quantile_ms(0.95), 500);
        assert_eq!(c.errors5xx, 10);
        assert_eq!(Cell::default().quantile_ms(0.95), 0);
        let mut slow = Cell::default();
        slow.add(200, 60_000);
        assert_eq!(slow.quantile_ms(0.5), 60_000);
        c.merge(&slow);
        assert_eq!(c.quantile_ms(1.0), 60_000);
    }

    #[test]
    fn the_ring_keeps_sixty_minutes_and_drops_older_ones() {
        // A fixed clock far from the real one, so other tests' traffic (real time)
        // lands in other minutes of the shared ring.
        let base = 1_000_000_000u64 / 60 * 60 + 17;
        record_at(base, RouteClass::Admin, 200, 4);
        record_at(base, RouteClass::Admin, 503, 30);
        record_at(base + 60, RouteClass::Jmap, 404, 1);
        let s = summary_at(base + 60);
        assert_eq!(s.series.len(), WINDOW_MINS);
        let last = &s.series[WINDOW_MINS - 1];
        let prev = &s.series[WINDOW_MINS - 2];
        assert_eq!(last.t, (base + 60) / 60 * 60);
        assert_eq!(last.all.requests, 1);
        assert_eq!(last.all.errors4xx, 1);
        assert_eq!(last.by_class["jmap"].requests, 1);
        assert_eq!(prev.by_class["admin"].requests, 2);
        assert_eq!(prev.by_class["admin"].errors5xx, 1);
        assert_eq!(prev.all.p95_ms, 50);
        // 60 minutes later the first minute has left the window.
        let later = summary_at(base + 60 * 60);
        assert!(later.series.iter().all(|m| m.t != base / 60 * 60));
        assert_eq!(later.series[0].t, (base + 60) / 60 * 60);
        assert_eq!(later.series[0].all.requests, 1);
    }
}
