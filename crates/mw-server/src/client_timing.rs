//! Browser load-time aggregates for signed-in sessions (admin "Client load times").
//!
//! The web app sends one small beacon per page load, after it is signed in, with
//! a few integer millisecond timings and a coarse device class
//! (`POST /api/client-timing`). The server adds each value to a fixed log-scale
//! histogram per hour, metric and device class ([`mw_store::client_timing`]) and
//! forgets everything else: no session, account, address, path, user agent or
//! timestamp is kept, and there is no per-session row. The session id is used
//! only as an in-memory rate-limit key and is never written anywhere.
//!
//! `GET /admin/metrics/client-timing` merges the hourly histograms over a period
//! and returns count, mean, p50/p75/p95/p99 per metric and device class plus a
//! p50/p95 series. Percentiles are withheld below `observability.clientTiming.
//! minSamples` samples. Accuracy is one histogram bin (about 22% worst case).
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    capabilities::Capability as C,
    scope_mw::rate_limit::KeyedRateLimiter,
};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use mw_store::{
    Store,
    client_timing::{DEVICES, Hist, MAX_MS, METRICS, TimingBucket},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub(crate) const ENABLED_KEY: &str = "observability.clientTiming.enabled";
pub(crate) const SAMPLE_KEY: &str = "observability.clientTiming.sampleRatePercent";
pub(crate) const MIN_SAMPLES_KEY: &str = "observability.clientTiming.minSamples";
const BODY_CAP: usize = 1024;
const MAX_METRICS_PER_BEACON: usize = 8;
/// A page load sends one beacon (plus one for a folder open); this leaves room
/// for a few reloads and still stops a client flooding the aggregate.
const BEACON_BURST: f64 = 6.0;
const BEACON_REFILL_PER_SEC: f64 = 1.0 / 30.0;
const POLICY_TTL: Duration = Duration::from_secs(10);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Policy {
    pub enabled: bool,
    pub sample_rate_percent: i64,
    pub min_samples: i64,
    pub retention_days: i64,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_rate_percent: 100,
            min_samples: 20,
            retention_days: 30,
        }
    }
}
impl Policy {
    async fn load(store: &Store) -> Self {
        async fn read(store: &Store, key: &str) -> Option<Value> {
            let row = store.get_setting_policy(key).await.ok()??;
            serde_json::from_str(&row.global_value?).ok()
        }
        let d = Self::default();
        let int = |v: Option<Value>, min, max, default| {
            v.and_then(|v| v.as_i64())
                .filter(|n| (min..=max).contains(n))
                .unwrap_or(default)
        };
        Self {
            enabled: read(store, ENABLED_KEY)
                .await
                .and_then(|v| v.as_bool())
                .unwrap_or(d.enabled),
            sample_rate_percent: int(read(store, SAMPLE_KEY).await, 1, 100, d.sample_rate_percent),
            min_samples: int(read(store, MIN_SAMPLES_KEY).await, 1, 100_000, d.min_samples),
            retention_days: int(read(store, "metrics.rollupDays").await, 1, 400, d.retention_days),
        }
    }
}

type Key = (i64, &'static str, &'static str);
pub(crate) struct Recorder {
    instance: String,
    memory: Mutex<BTreeMap<Key, Hist>>,
    policy: Mutex<Option<(Instant, Policy)>>,
    limiter: Mutex<KeyedRateLimiter>,
}
fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
impl Recorder {
    pub(crate) fn start(store: Store) -> Arc<Self> {
        let recorder = Arc::new(Self {
            instance: uuid::Uuid::new_v4().to_string(),
            memory: Mutex::new(BTreeMap::new()),
            policy: Mutex::new(None),
            limiter: Mutex::new(KeyedRateLimiter::new(
                BEACON_BURST,
                BEACON_REFILL_PER_SEC,
                20_000,
            )),
        });
        let weak = Arc::downgrade(&recorder);
        tokio::spawn(async move {
            let mut timer = tokio::time::interval(Duration::from_secs(30));
            timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                timer.tick().await;
                let Some(recorder) = weak.upgrade() else {
                    break;
                };
                let (instance, buckets) = (recorder.instance.clone(), recorder.snapshot(now()));
                drop(recorder);
                if buckets.is_empty() {
                    continue;
                }
                let work = async {
                    let policy = Policy::load(&store).await;
                    store
                        .save_client_timing(&instance, &buckets, now(), policy.retention_days)
                        .await
                };
                match tokio::time::timeout(Duration::from_secs(5), work).await {
                    Ok(Ok(())) => {}
                    _ => tracing::warn!("client timing flush unavailable; retained for retry"),
                }
            }
        });
        recorder
    }
    /// Current and previous hour only: older hours were flushed long ago.
    fn snapshot(&self, time: i64) -> Vec<TimingBucket> {
        let mut memory = self
            .memory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let oldest = time / 3600 * 3600 - 3600;
        memory.retain(|(start, _, _), _| *start >= oldest);
        memory
            .iter()
            .map(|((start, metric, device), hist)| TimingBucket {
                start: *start,
                metric: (*metric).to_owned(),
                device: (*device).to_owned(),
                hist: hist.clone(),
            })
            .collect()
    }
    fn record_at(&self, time: i64, device: &'static str, values: &[(&'static str, u64)]) {
        let mut memory = self
            .memory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (metric, ms) in values {
            memory
                .entry((time / 3600 * 3600, *metric, device))
                .or_default()
                .record(*ms);
        }
    }
    async fn policy(&self, store: &Store) -> Policy {
        if let Some((at, policy)) = *self
            .policy
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            && at.elapsed() < POLICY_TTL
        {
            return policy;
        }
        let policy = Policy::load(store).await;
        *self
            .policy
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some((Instant::now(), policy));
        policy
    }
    fn allow(&self, session: &str) -> bool {
        self.limiter
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .check(session)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Sample {
    name: String,
    ms: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Beacon {
    #[serde(default)]
    device: Option<String>,
    metrics: Vec<Sample>,
}

/// A validated beacon: a static device class and unique known metrics, clamped.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Accepted {
    pub device: &'static str,
    pub values: Vec<(&'static str, u64)>,
}
/// Strict parse of a beacon body. `Err` carries a short reason (never echoed
/// user input).
pub(crate) fn parse(body: &[u8]) -> Result<Accepted, &'static str> {
    if body.len() > BODY_CAP {
        return Err("body too large");
    }
    let beacon: Beacon = serde_json::from_slice(body).map_err(|_| "malformed body")?;
    let device = match beacon.device.as_deref() {
        None => "unknown",
        Some(d) => *DEVICES
            .iter()
            .find(|known| **known == d)
            .ok_or("unknown device class")?,
    };
    if beacon.metrics.is_empty() || beacon.metrics.len() > MAX_METRICS_PER_BEACON {
        return Err("wrong number of metrics");
    }
    let mut values: Vec<(&'static str, u64)> = Vec::new();
    for sample in &beacon.metrics {
        let name = *METRICS
            .iter()
            .find(|known| **known == sample.name)
            .ok_or("unknown metric")?;
        if values.iter().any(|(seen, _)| *seen == name) {
            return Err("duplicate metric");
        }
        values.push((name, sample.ms.min(MAX_MS)));
    }
    Ok(Accepted { device, values })
}

pub(crate) fn router() -> Router<AppState> {
    Router::new().route(
        "/api/client-timing",
        post(ingest).layer(DefaultBodyLimit::max(BODY_CAP * 2)),
    )
}

/// `POST /api/client-timing`: 204 on success (also when collection is off, so a
/// stale client stays quiet), 401 without a session, 400/413 on a bad body, 429
/// when one session sends too many.
async fn ingest(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    let recorder = &state.client_timing;
    if !recorder.policy(&state.store).await.enabled {
        return StatusCode::NO_CONTENT.into_response();
    }
    if body.len() > BODY_CAP {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    if !recorder.allow(&session.id) {
        return (StatusCode::TOO_MANY_REQUESTS, [(header::RETRY_AFTER, "30")]).into_response();
    }
    match parse(&body) {
        Ok(accepted) => {
            recorder.record_at(now(), accepted.device, &accepted.values);
            StatusCode::NO_CONTENT.into_response()
        }
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

/// What `GET /api/policy` tells a signed-in client.
pub(crate) async fn client_view(state: &AppState) -> Value {
    let policy = state.client_timing.policy(&state.store).await;
    json!({"enabled": policy.enabled, "sampleRatePercent": policy.sample_rate_percent})
}

pub(crate) fn register(r: &mut impl crate::admin_router::AdminRegistrar) {
    r.admin_endpoint(
        axum::http::Method::GET,
        "/metrics/client-timing",
        C::ObservabilityRead,
        history,
    );
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Range {
    from: Option<i64>,
    to: Option<i64>,
    max_points: Option<usize>,
}

fn stats(hist: &Hist, min_samples: u64) -> Value {
    let enough = hist.count >= min_samples;
    let p = |n| enough.then(|| hist.percentile_ms(n)).flatten();
    json!({
        "count": hist.count,
        "sufficient": enough,
        "meanMs": hist.mean_ms(),
        "p50Ms": p(50), "p75Ms": p(75), "p95Ms": p(95), "p99Ms": p(99),
    })
}

/// Merge `rows` into per-metric totals, per-device totals and a p50/p95 series.
pub(crate) fn summarize(
    rows: &[TimingBucket],
    from: i64,
    to: i64,
    width: i64,
    min_samples: u64,
) -> Value {
    let mut metrics = serde_json::Map::new();
    for metric in METRICS {
        let mut all = Hist::default();
        let mut devices: BTreeMap<&str, Hist> = BTreeMap::new();
        let mut points: BTreeMap<i64, Hist> = BTreeMap::new();
        for row in rows.iter().filter(|r| r.metric == metric) {
            all.merge(&row.hist);
            devices
                .entry(row.device.as_str())
                .or_default()
                .merge(&row.hist);
            points
                .entry(from + (row.start - from).max(0) / width * width)
                .or_default()
                .merge(&row.hist);
        }
        let mut series = Vec::new();
        let mut t = from;
        while t < to {
            let h = points.remove(&t).unwrap_or_default();
            let enough = h.count >= min_samples;
            series.push(json!({
                "t": t, "count": h.count,
                "p50Ms": enough.then(|| h.percentile_ms(50)).flatten(),
                "p95Ms": enough.then(|| h.percentile_ms(95)).flatten(),
            }));
            t += width;
        }
        let by_device: serde_json::Map<String, Value> = devices
            .iter()
            .map(|(d, h)| ((*d).to_owned(), stats(h, min_samples)))
            .collect();
        metrics.insert(
            metric.to_owned(),
            json!({"all": stats(&all, min_samples), "devices": by_device, "series": series}),
        );
    }
    Value::Object(metrics)
}

pub(crate) async fn history(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(range): Query<Range>,
) -> Result<Response, AdminApiError> {
    if !ctx.has_unscoped(C::ObservabilityRead) {
        return Err(AdminApiError::unscoped_required(C::ObservabilityRead));
    }
    let unavailable = || {
        AdminApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "historyUnavailable",
            "Client load times are temporarily unavailable.",
        )
    };
    let policy = Policy::load(&state.store).await;
    let current = now();
    let to = range.to.unwrap_or(current).min(current + 3600);
    let from = range.from.unwrap_or(to - 86400);
    let span = to - from;
    if from < 0 || span <= 0 || span > policy.retention_days * 86400 {
        return Err(AdminApiError::new(
            StatusCode::BAD_REQUEST,
            "invalidRange",
            "The period is empty or longer than the retention.",
        ));
    }
    let points = range.max_points.unwrap_or(48).clamp(2, 200) as i64;
    let hours_from = from / 3600 * 3600;
    let width = ((span + points - 1) / points + 3599) / 3600 * 3600;
    let rows = tokio::time::timeout(
        Duration::from_secs(5),
        state.store.client_timing_history(hours_from, to),
    )
    .await
    .map_err(|_| unavailable())?
    .map_err(|_| unavailable())?;
    let metrics = summarize(&rows, hours_from, to, width, policy.min_samples as u64);
    Ok((
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({
            "from": hours_from, "to": to, "bucketSeconds": width,
            "metrics": metrics,
            "policy": {
                "enabled": policy.enabled,
                "sampleRatePercent": policy.sample_rate_percent,
                "minSamples": policy.min_samples,
                "retentionDays": policy.retention_days,
            },
            "resolution": {"bins": mw_store::client_timing::BINS, "maxRelativeError": 0.22},
            "scope": "client", "sampled": true,
        })),
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_valid_beacon_is_clamped_and_classified() {
        let ok = parse(
            br#"{"device":"mobile","metrics":[{"name":"timeToUsable","ms":999999},{"name":"ttfb","ms":0}]}"#,
        )
        .unwrap();
        assert_eq!(ok.device, "mobile");
        assert_eq!(ok.values, vec![("timeToUsable", 120_000), ("ttfb", 0)]);
        assert_eq!(parse(br#"{"metrics":[{"name":"fcp","ms":5}]}"#).unwrap().device, "unknown");
    }

    #[test]
    fn invalid_beacons_are_refused() {
        for body in [
            &br#"{"metrics":[{"name":"path","ms":5}]}"#[..],
            br#"{"metrics":[{"name":"fcp","ms":-1}]}"#,
            br#"{"metrics":[{"name":"fcp","ms":1.5}]}"#,
            br#"{"metrics":[{"name":"fcp","ms":1},{"name":"fcp","ms":2}]}"#,
            br#"{"metrics":[]}"#,
            br#"{"device":"tv","metrics":[{"name":"fcp","ms":1}]}"#,
            br#"{"url":"/x","metrics":[{"name":"fcp","ms":1}]}"#,
            br#"{"metrics":[{"name":"fcp","ms":1,"extra":1}]}"#,
            b"not json",
        ] {
            assert!(parse(body).is_err(), "{}", String::from_utf8_lossy(body));
        }
        let many = format!(
            r#"{{"metrics":[{}]}}"#,
            (0..9).map(|_| r#"{"name":"fcp","ms":1}"#).collect::<Vec<_>>().join(",")
        );
        assert!(parse(many.as_bytes()).is_err());
        assert!(parse(&vec![b' '; BODY_CAP + 1]).is_err());
    }

    #[tokio::test]
    async fn recording_snapshotting_and_the_session_rate_limit() {
        let store = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let recorder = Recorder::start(store.clone());
        let t = 3600 * 500 + 7;
        recorder.record_at(t, "desktop", &[("timeToUsable", 800), ("ttfb", 40)]);
        recorder.record_at(t, "desktop", &[("timeToUsable", 1200)]);
        let snap = recorder.snapshot(t);
        assert_eq!(snap.len(), 2);
        let tti = snap.iter().find(|b| b.metric == "timeToUsable").unwrap();
        assert_eq!((tti.start, tti.hist.count, tti.hist.sum_ms), (3600 * 500, 2, 2000));
        store.save_client_timing("i", &snap, t, 30).await.unwrap();
        let rows = store.client_timing_history(0, t + 3600).await.unwrap();
        assert_eq!(rows.len(), 2);
        // An hour two hours old is dropped from memory.
        assert!(recorder.snapshot(t + 3 * 3600).is_empty());
        let allowed = (0..20).filter(|_| recorder.allow("s1")).count();
        assert_eq!(allowed, BEACON_BURST as usize);
        assert!(recorder.allow("s2"));
        let p = Policy::load(&store).await;
        assert_eq!(p, Policy::default());
    }

    #[test]
    fn summary_hides_percentiles_below_the_minimum_and_merges_hours() {
        let mut h = Hist::default();
        for ms in 1..=30 {
            h.record(ms * 100);
        }
        let mut few = Hist::default();
        few.record(500);
        let rows = vec![
            TimingBucket { start: 0, metric: "ttfb".into(), device: "desktop".into(), hist: h.clone() },
            TimingBucket { start: 3600, metric: "ttfb".into(), device: "desktop".into(), hist: h },
            TimingBucket { start: 3600, metric: "load".into(), device: "mobile".into(), hist: few },
        ];
        let v = summarize(&rows, 0, 7200, 3600, 20);
        assert_eq!(v["ttfb"]["all"]["count"], 60);
        assert!(v["ttfb"]["all"]["p95Ms"].as_f64().unwrap() > 2000.0);
        assert_eq!(v["ttfb"]["series"].as_array().unwrap().len(), 2);
        assert_eq!(v["load"]["all"]["sufficient"], false);
        assert!(v["load"]["all"]["p50Ms"].is_null());
        assert_eq!(v["load"]["all"]["meanMs"], 500.0);
        assert_eq!(v["lcp"]["all"]["count"], 0);
    }
}
