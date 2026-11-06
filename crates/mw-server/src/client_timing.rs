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
/// Per-person feature switches (admin default and lock, user opt-out).
pub(crate) const FOLDER_KEY: &str = "observability.clientTiming.folderLoads";
pub(crate) const MESSAGE_KEY: &str = "observability.clientTiming.messageLoads";
const BODY_CAP: usize = 1536;
const MAX_METRICS_PER_BEACON: usize = 16;
/// Interaction beacons (folder and message loads) are batched by the client every
/// few seconds, so their bucket refills faster than the page-load one.
const INTERACTION_BURST: f64 = 12.0;
const INTERACTION_REFILL_PER_SEC: f64 = 1.0 / 5.0;
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
    interaction_limiter: Mutex<KeyedRateLimiter>,
    /// The `(burst, per minute)` both limiters were last built for.
    tuned: Mutex<Option<(u32, u32)>>,
}

/// Which feature set a metric belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    PageLoad,
    Folder,
    Message,
}
pub(crate) fn kind_of(metric: &str) -> Kind {
    match metric {
        "folderOpen" | "folderLoadCold" | "folderLoadCached" => Kind::Folder,
        "messageLoadCold" | "messageLoadCached" => Kind::Message,
        _ => Kind::PageLoad,
    }
}

/// Whether this person's folder / message load measurement is on: the registry's
/// effective value (admin default and lock, then the person's own choice).
pub(crate) async fn kinds_for(state: &AppState, session: &mw_store::Session) -> (bool, bool) {
    let Ok(identity) = crate::settings_routes::UserSubject::resolved(&state.store, session).await
    else {
        return (false, false);
    };
    let subject = identity.as_subject();
    let on = |key| {
        let store = state.store.clone();
        let subject = subject.clone();
        async move {
            crate::settings_registry::effective_for(&store, key, &subject)
                .await
                .ok()
                .flatten()
                .map(|v| v.value != false)
                .unwrap_or(true)
        }
    };
    (on(FOLDER_KEY).await, on(MESSAGE_KEY).await)
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
            interaction_limiter: Mutex::new(KeyedRateLimiter::new(
                INTERACTION_BURST,
                INTERACTION_REFILL_PER_SEC,
                20_000,
            )),
            tuned: Mutex::new(None),
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
        for (metric, ms) in values {
            if let Some(name) = crate::metrics_persist::client_metric(metric) {
                let dims = mw_store::metrics_bucket::Dims {
                    client_kind: "web".into(),
                    device_class: device.into(),
                    ..Default::default()
                };
                crate::metrics_persist::value(time, name, *ms, &dims);
            }
        }
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
    /// [`Self::allow`] with the configured `ratelimit.session.*` budget: both
    /// limiters are rebuilt when it changes (a change forgets the spent budgets).
    fn allow_with(&self, session: &str, interaction: bool, burst: u32, per_minute: u32) -> bool {
        {
            let mut tuned = self.tuned.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if *tuned != Some((burst, per_minute)) {
                let refill = f64::from(per_minute) / 60.0;
                *self.limiter.lock().unwrap_or_else(std::sync::PoisonError::into_inner) =
                    KeyedRateLimiter::new(f64::from(burst), refill, 20_000);
                *self
                    .interaction_limiter
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) =
                    KeyedRateLimiter::new(f64::from(burst), refill, 20_000);
                *tuned = Some((burst, per_minute));
            }
        }
        self.allow(session, interaction)
    }
    fn allow(&self, session: &str, interaction: bool) -> bool {
        if interaction {
            return self
                .interaction_limiter
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .check(session);
        }
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
    /// Read by `ingest` (see [`RateOnly`]); accepted here so the strict parse allows it.
    #[serde(default)]
    #[allow(dead_code)]
    rate: Option<RateReport>,
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
        // A page load happens once; interactions repeat inside one batched beacon.
        if kind_of(name) == Kind::PageLoad && values.iter().any(|(seen, _)| *seen == name) {
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
    let interaction = parse_kinds(&body).is_some_and(|all_interaction| all_interaction);
    let knobs = crate::rate_leeway::knobs(
        &state.store,
        crate::rate_leeway::domain_of(&session.username),
    )
    .await;
    if !recorder.allow_with(&session.id, interaction, knobs.session_burst, knobs.session_per_minute) {
        crate::rate_leeway::note("clientTiming", crate::rate_leeway::Outcome::Rejected);
        // One token comes back after `60 / perMinute` seconds.
        let wait = (60 / u64::from(knobs.session_per_minute.max(1))).clamp(1, 60);
        return crate::rate_leeway::tag(
            (StatusCode::TOO_MANY_REQUESTS, [(header::RETRY_AFTER, wait.to_string())]).into_response(),
            crate::rate_leeway::Refused {
                class: "clientTiming",
                reason: "sessionBudget",
                key_type: "session",
                limit: Some(knobs.session_burst),
                retry_after_secs: wait,
            },
        );
    }
    crate::rate_leeway::note("clientTiming", crate::rate_leeway::Outcome::Allowed);
    if let Ok(report) = serde_json::from_slice::<RateOnly>(&body)
        && let Some(r) = report.rate
    {
        crate::rate_leeway::note_client(r.seen, r.retries, r.exhausted);
    }
    match parse(&body) {
        Ok(mut accepted) => {
            // Metrics of a feature this person (or the admin) turned off are dropped,
            // not an error: a stale client stays quiet.
            if accepted.values.iter().any(|(m, _)| kind_of(m) != Kind::PageLoad) {
                let (folders, messages) = kinds_for(&state, &session).await;
                accepted.values.retain(|(m, _)| match kind_of(m) {
                    Kind::PageLoad => true,
                    Kind::Folder => folders,
                    Kind::Message => messages,
                });
            }
            if !accepted.values.is_empty() {
                recorder.record_at(now(), accepted.device, &accepted.values);
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

/// What a client says about its own `429`s since its last report.
#[derive(Deserialize)]
struct RateReport {
    #[serde(default)]
    seen: u32,
    #[serde(default)]
    retries: u32,
    #[serde(default)]
    exhausted: u32,
}
#[derive(Deserialize)]
struct RateOnly {
    #[serde(default)]
    rate: Option<RateReport>,
}

/// Whether every sample in a body is an interaction metric (`None`: unparsable;
/// the strict parse below answers 400).
fn parse_kinds(body: &[u8]) -> Option<bool> {
    let beacon: Beacon = serde_json::from_slice(body).ok()?;
    Some(
        !beacon.metrics.is_empty()
            && beacon
                .metrics
                .iter()
                .all(|m| kind_of(&m.name) != Kind::PageLoad),
    )
}

/// What `GET /api/policy` tells a signed-in client.
pub(crate) async fn client_view(state: &AppState, session: &mw_store::Session) -> Value {
    let policy = state.client_timing.policy(&state.store).await;
    let (folder_loads, message_loads) = kinds_for(state, session).await;
    json!({
        "enabled": policy.enabled,
        "sampleRatePercent": policy.sample_rate_percent,
        "folderLoads": policy.enabled && folder_loads,
        "messageLoads": policy.enabled && message_loads,
    })
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

/// The instance-wide default of a feature switch (what a person without an own
/// value gets), for the admin screen's note.
async fn kind_default(state: &AppState, key: &str) -> bool {
    crate::settings_registry::effective_for(&state.store, key, &Default::default())
        .await
        .ok()
        .flatten()
        .map(|v| v.value != false)
        .unwrap_or(true)
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
                "folderLoads": kind_default(&state, FOLDER_KEY).await,
                "messageLoads": kind_default(&state, MESSAGE_KEY).await,
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
            (0..17).map(|_| r#"{"name":"folderLoadCold","ms":1}"#).collect::<Vec<_>>().join(",")
        );
        assert!(parse(many.as_bytes()).is_err());
        assert!(parse(&vec![b' '; BODY_CAP + 1]).is_err());
    }

    #[test]
    fn interaction_metrics_may_repeat_in_one_beacon_but_page_loads_may_not() {
        let ok = parse(
            br#"{"device":"desktop","metrics":[{"name":"folderLoadCold","ms":300},{"name":"folderLoadCached","ms":20},{"name":"messageLoadCold","ms":900},{"name":"messageLoadCold","ms":700},{"name":"messageLoadCached","ms":9999999}]}"#,
        )
        .unwrap();
        assert_eq!(ok.values.len(), 5);
        assert_eq!(ok.values[4], ("messageLoadCached", 120_000));
        assert_eq!(kind_of("folderLoadCold"), Kind::Folder);
        assert_eq!(kind_of("folderOpen"), Kind::Folder);
        assert_eq!(kind_of("messageLoadCached"), Kind::Message);
        assert_eq!(kind_of("fcp"), Kind::PageLoad);
        assert_eq!(parse_kinds(br#"{"metrics":[{"name":"messageLoadCold","ms":1}]}"#), Some(true));
        assert_eq!(parse_kinds(br#"{"metrics":[{"name":"fcp","ms":1},{"name":"messageLoadCold","ms":1}]}"#), Some(false));
        assert_eq!(parse_kinds(b"x"), None);
        // Folder or message identity has nowhere to go.
        for body in [
            &br#"{"metrics":[{"name":"messageLoadCold","ms":1,"id":"m1"}]}"#[..],
            br#"{"metrics":[{"name":"folderLoadCold:inbox","ms":1}]}"#,
            br#"{"folder":"Inbox","metrics":[{"name":"folderLoadCold","ms":1}]}"#,
        ] {
            assert!(parse(body).is_err());
        }
    }

    #[test]
    fn interaction_beacons_have_their_own_larger_rate_limit() {
        let recorder = Recorder {
            instance: String::new(),
            memory: Mutex::new(BTreeMap::new()),
            policy: Mutex::new(None),
            limiter: Mutex::new(KeyedRateLimiter::new(BEACON_BURST, BEACON_REFILL_PER_SEC, 10)),
            interaction_limiter: Mutex::new(KeyedRateLimiter::new(
                INTERACTION_BURST,
                INTERACTION_REFILL_PER_SEC,
                10,
            )),
            tuned: Mutex::new(None),
        };
        assert_eq!((0..40).filter(|_| recorder.allow("s", true)).count(), INTERACTION_BURST as usize);
        // Page-load beacons are unaffected by interaction traffic.
        assert_eq!((0..40).filter(|_| recorder.allow("s", false)).count(), BEACON_BURST as usize);
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
        let allowed = (0..20).filter(|_| recorder.allow("s1", false)).count();
        assert_eq!(allowed, BEACON_BURST as usize);
        assert!(recorder.allow("s2", false));
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
        // Interaction metrics summarize like any other and start empty.
        for m in ["folderLoadCold", "folderLoadCached", "messageLoadCold", "messageLoadCached"] {
            assert_eq!(v[m]["all"]["count"], 0, "{m}");
        }
    }
}
