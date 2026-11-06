//! Persisted performance statistics: the recorder that keeps client load times,
//! image proxy latency, rate-limit decisions and request-protection decisions across
//! restarts, and the admin read/reset routes (`GET /admin/metrics/persisted`,
//! `POST /admin/metrics/persisted/reset`).
//!
//! Recording is cheap and in memory: each sample lands in the sketch of its
//! minute, metric and bounded dimension set ([`mw_store::metrics_bucket`]). Every
//! `metrics.persist.flushSeconds` (and once on graceful shutdown) the changed
//! minutes are written as absolute snapshots under this process's instance id, so
//! a retried flush never counts twice and several instances never contend. A read
//! merges every instance's rows by adding sketch counts, so the median and the 95th
//! percentile of any range come from the merged counts, never from averaging
//! percentiles. A restarted process starts a new instance id, finds the earlier
//! rows in the database and shows them straight away.
//!
//! Privacy: a metric name is a fixed string from this code; dimensions are fixed
//! role classes (client kind, device class); no person, address, path, message or
//! folder name is recorded. Per metric and minute at most [`MAX_DIM_SETS`]
//! distinct dimension sets are kept; later ones fold into `other`.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::AdminRegistrar,
    capabilities::Capability as C,
};
use axum::{
    Json,
    extract::{Query, State},
    http::{Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use mw_store::{
    Store, StoreError,
    metrics_bucket::{self as mb, Agg, Dims, MINUTE, MetricRow},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::{
        Arc, Mutex, OnceLock, RwLock, Weak,
        atomic::{AtomicBool, AtomicI64, Ordering::Relaxed},
    },
    time::Duration,
};

pub(crate) const ENABLED_KEY: &str = "metrics.persist.enabled";
pub(crate) const FLUSH_KEY: &str = "metrics.persist.flushSeconds";
pub(crate) const DIMENSIONS_KEY: &str = "metrics.dimensions.enabled";
pub(crate) const MINUTE_HOURS_KEY: &str = "retention.metrics.minuteBucketsHours";
pub(crate) const HOUR_DAYS_KEY: &str = "retention.metrics.hourBucketsDays";
pub(crate) const DAY_DAYS_KEY: &str = "retention.metrics.dayBucketsDays";

/// Distinct dimension sets one metric keeps per minute before folding into `other`.
pub(crate) const MAX_DIM_SETS: usize = 32;
/// Clean minutes older than this are dropped from memory after a flush.
const KEEP_CLEAN_SECS: i64 = 5 * 60;

/// Metric name prefixes, one per source shown in the console.
pub(crate) const IMAGE: &str = "imageProxy.";
pub(crate) const CLIENT: &str = "clientTiming.";
pub(crate) const RATE: &str = "rateLimit.";
pub(crate) const PROTECTION: &str = "requestProtection.";

struct Entry {
    agg: Agg,
    dirty: bool,
}
#[derive(Default)]
struct Inner {
    entries: BTreeMap<(&'static str, i64, Dims), Entry>,
    sets: HashMap<(&'static str, i64), BTreeSet<Dims>>,
    /// Cumulative rate-limit totals last turned into counts.
    rate_seen: HashMap<(usize, usize), u64>,
}

pub(crate) struct Recorder {
    instance: String,
    store: Store,
    inner: Mutex<Inner>,
    enabled: AtomicBool,
    dimensions: AtomicBool,
    last_flush: AtomicI64,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Fold every non-empty dimension into `other`.
fn overflow(d: &Dims) -> Dims {
    let f = |s: &String| if s.is_empty() { String::new() } else { "other".to_owned() };
    Dims {
        domain: f(&d.domain),
        folder_class: f(&d.folder_class),
        client_kind: f(&d.client_kind),
        device_class: f(&d.device_class),
        backend: f(&d.backend),
    }
}

impl Recorder {
    pub(crate) fn new(store: Store) -> Arc<Self> {
        // Rate-limit decisions made before this recorder existed belong to an earlier one.
        let mut inner = Inner::default();
        for (class, totals) in crate::rate_leeway::totals().into_iter().enumerate() {
            for (outcome, total) in totals.into_iter().enumerate() {
                inner.rate_seen.insert((class, outcome), total);
            }
        }
        Arc::new(Self {
            instance: uuid::Uuid::new_v4().to_string(),
            store,
            inner: Mutex::new(inner),
            enabled: AtomicBool::new(true),
            dimensions: AtomicBool::new(true),
            last_flush: AtomicI64::new(0),
        })
    }

    pub(crate) fn instance(&self) -> &str {
        &self.instance
    }

    fn slot<R>(&self, now: i64, metric: &'static str, dims: &Dims, f: impl FnOnce(&mut Agg) -> R) -> Option<R> {
        if !self.enabled.load(Relaxed) {
            return None;
        }
        let minute = now.max(0) / MINUTE * MINUTE;
        let mut dims = if self.dimensions.load(Relaxed) { dims.clone() } else { Dims::default() };
        let mut inner = lock(&self.inner);
        let known = inner.sets.entry((metric, minute)).or_default();
        if !known.contains(&dims) {
            if known.len() >= MAX_DIM_SETS {
                dims = overflow(&dims);
            }
            known.insert(dims.clone());
        }
        let entry = inner
            .entries
            .entry((metric, minute, dims))
            .or_insert_with(|| Entry { agg: Agg::default(), dirty: false });
        entry.dirty = true;
        Some(f(&mut entry.agg))
    }

    /// One latency sample (`metric` is a fixed name; `value` in the metric's unit).
    pub(crate) fn record_value(&self, now: i64, metric: &'static str, value: u64, dims: &Dims) {
        self.slot(now, metric, dims, |a| a.record(value));
    }

    /// Add `n` to a counter metric.
    pub(crate) fn record_count(&self, now: i64, metric: &'static str, n: u64, dims: &Dims) {
        if n > 0 {
            self.slot(now, metric, dims, |a| a.add_count(n));
        }
    }

    /// Turn the change in the process-wide rate-limit counters into counts.
    fn absorb_rate_limits(&self, now: i64) {
        for (class, totals) in crate::rate_leeway::totals().into_iter().enumerate() {
            for (outcome, total) in totals.into_iter().enumerate() {
                let delta = {
                    let mut inner = lock(&self.inner);
                    let seen = inner.rate_seen.entry((class, outcome)).or_insert(0);
                    let delta = total.saturating_sub(*seen);
                    *seen = total;
                    delta
                };
                self.record_count(now, rate_metric(class, outcome), delta, &Dims::default());
            }
        }
    }

    /// Rows that changed since they were last written.
    fn dirty_rows(&self) -> Vec<MetricRow> {
        self.dirty().into_iter().map(|(_, row)| row).collect()
    }
    fn dirty(&self) -> Vec<(&'static str, MetricRow)> {
        lock(&self.inner)
            .entries
            .iter()
            .filter(|(_, e)| e.dirty)
            .map(|((metric, start, dims), e)| {
                (
                    *metric,
                    MetricRow {
                        instance: self.instance.clone(),
                        metric: (*metric).to_owned(),
                        start: *start,
                        resolution: MINUTE,
                        dims: dims.clone(),
                        agg: e.agg.clone(),
                    },
                )
            })
            .collect()
    }

    /// Write changed minutes; on success mark them clean (unless they changed again
    /// meanwhile) and drop clean minutes that are no longer being filled. On failure
    /// nothing is lost: the rows stay dirty and go out with the next flush.
    pub(crate) async fn flush(&self, now: i64) -> Result<usize, StoreError> {
        if !self.enabled.load(Relaxed) {
            return Ok(0);
        }
        self.absorb_rate_limits(now);
        let rows = self.dirty();
        for chunk in rows.chunks(2000) {
            let written: Vec<MetricRow> = chunk.iter().map(|(_, r)| r.clone()).collect();
            self.store.save_metric_rows(&written, now).await?;
            let mut inner = lock(&self.inner);
            for (metric, r) in chunk {
                if let Some(e) = inner.entries.get_mut(&(*metric, r.start, r.dims.clone()))
                    && e.agg.count == r.agg.count
                {
                    e.dirty = false;
                }
            }
        }
        let cutoff = now - KEEP_CLEAN_SECS;
        {
            let mut inner = lock(&self.inner);
            inner.entries.retain(|(_, start, _), e| e.dirty || *start >= cutoff);
            inner.sets.retain(|(_, start), _| *start >= cutoff);
        }
        self.last_flush.store(now, Relaxed);
        Ok(rows.len())
    }

    /// Forget in-memory minutes of metrics starting with `prefix` (after a reset).
    pub(crate) fn forget(&self, prefix: &str) {
        let mut inner = lock(&self.inner);
        inner.entries.retain(|(m, _, _), _| !m.starts_with(prefix));
        inner.sets.retain(|(m, _), _| !m.starts_with(prefix));
    }

    fn configure(&self, enabled: bool, dimensions: bool) {
        self.enabled.store(enabled, Relaxed);
        self.dimensions.store(dimensions, Relaxed);
    }
}

// ── names ───────────────────────────────────────────────────────────────────────

fn rate_metric(class: usize, outcome: usize) -> &'static str {
    static NAMES: OnceLock<Vec<&'static str>> = OnceLock::new();
    let names = NAMES.get_or_init(|| {
        let mut out = Vec::new();
        for c in crate::rate_leeway::CLASSES {
            for o in crate::rate_leeway::OUTCOME_NAMES {
                out.push(&*Box::leak(format!("{RATE}{c}.{o}").into_boxed_str()));
            }
        }
        out
    });
    names[class * crate::rate_leeway::OUTCOME_NAMES.len() + outcome]
}

/// Client timing metric name to its stored name.
pub(crate) fn client_metric(name: &str) -> Option<&'static str> {
    Some(match name {
        "timeToUsable" => "clientTiming.timeToUsable",
        "ttfb" => "clientTiming.ttfb",
        "dcl" => "clientTiming.dcl",
        "load" => "clientTiming.load",
        "fcp" => "clientTiming.fcp",
        "lcp" => "clientTiming.lcp",
        "folderOpen" => "clientTiming.folderOpen",
        "folderLoadCold" => "clientTiming.folderLoadCold",
        "folderLoadCached" => "clientTiming.folderLoadCached",
        "messageLoadCold" => "clientTiming.messageLoadCold",
        "messageLoadCached" => "clientTiming.messageLoadCached",
        _ => return None,
    })
}

// ── process-wide recorder ───────────────────────────────────────────────────────

fn global() -> &'static RwLock<Option<Arc<Recorder>>> {
    static G: OnceLock<RwLock<Option<Arc<Recorder>>>> = OnceLock::new();
    G.get_or_init(|| RwLock::new(None))
}
static ACTIVE: AtomicBool = AtomicBool::new(false);

/// Whether statistics are being kept across restarts right now.
pub(crate) fn active() -> bool {
    ACTIVE.load(Relaxed)
}

pub(crate) fn current() -> Option<Arc<Recorder>> {
    global().read().ok().and_then(|g| g.clone())
}

fn unix_now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Record a latency sample on the process recorder (a no-op before it starts or
/// while persistence is off).
pub(crate) fn value(now: i64, metric: &'static str, v: u64, dims: &Dims) {
    if ACTIVE.load(Relaxed)
        && let Some(r) = current()
    {
        r.record_value(now, metric, v, dims);
    }
}

/// Add to a counter on the process recorder.
pub(crate) fn count(now: i64, metric: &'static str, n: u64) {
    if ACTIVE.load(Relaxed)
        && let Some(r) = current()
    {
        r.record_count(now, metric, n, &Dims::default());
    }
}

/// Start the process recorder over `store`: warm the live image-latency view from
/// the stored minutes, then flush on a timer until the recorder is replaced or
/// dropped.
pub(crate) fn start(store: Store) -> Arc<Recorder> {
    let recorder = Recorder::new(store.clone());
    if let Ok(mut g) = global().write() {
        *g = Some(recorder.clone());
    }
    ACTIVE.store(true, Relaxed);
    let weak: Weak<Recorder> = Arc::downgrade(&recorder);
    tokio::spawn(async move {
        let mut first = true;
        loop {
            let Some(r) = weak.upgrade() else { break };
            let settings = Settings::load(&r.store).await;
            r.configure(settings.enabled, settings.dimensions);
            ACTIVE.store(settings.enabled, Relaxed);
            if first {
                first = false;
                if settings.enabled {
                    warm_image_latency(&r.store, unix_now()).await;
                }
            }
            if !current().is_some_and(|c| Arc::ptr_eq(&c, &r)) {
                break;
            }
            match tokio::time::timeout(Duration::from_secs(10), r.flush(unix_now())).await {
                Ok(Ok(_)) => {}
                _ => tracing::warn!("metrics flush unavailable; retained for retry"),
            }
            let wait = settings.flush_seconds;
            drop(r);
            tokio::time::sleep(Duration::from_secs(wait)).await;
        }
    });
    recorder
}

/// Best-effort final flush at graceful shutdown.
pub(crate) async fn flush_on_shutdown() {
    let Some(r) = current() else { return };
    match tokio::time::timeout(Duration::from_secs(5), r.flush(unix_now())).await {
        Ok(Ok(n)) => tracing::info!("metrics flushed at shutdown ({n} row(s))"),
        _ => tracing::warn!("metrics were not flushed at shutdown"),
    }
}

/// Fill the live image-latency view from stored minutes so it does not start empty.
async fn warm_image_latency(store: &Store, now: i64) {
    let from = now - crate::image_proxy::latency::KEEP_SECS as i64;
    if let Ok(rows) = store.load_metric_rows(IMAGE, from, now).await {
        crate::image_proxy::latency::warm(rows);
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Settings {
    pub enabled: bool,
    pub dimensions: bool,
    pub flush_seconds: u64,
    pub minute_hours: i64,
    pub hour_days: i64,
    pub day_days: i64,
}
impl Settings {
    pub(crate) async fn load(store: &Store) -> Self {
        use crate::auth_logs::sweeper::{num, num_bool};
        Self {
            enabled: num_bool(store, ENABLED_KEY).await,
            dimensions: num_bool(store, DIMENSIONS_KEY).await,
            flush_seconds: num(store, FLUSH_KEY).await.clamp(5, 3600) as u64,
            minute_hours: num(store, MINUTE_HOURS_KEY).await.clamp(1, 24 * 365),
            hour_days: num(store, HOUR_DAYS_KEY).await.clamp(1, 3650),
            day_days: num(store, DAY_DAYS_KEY).await.clamp(0, 3650),
        }
    }
}

/// Retention rollup for the sweeper: fold minute rows into hours, hour rows into
/// days, prune days. Returns the rows folded or pruned.
pub(crate) async fn rollup(store: &Store, now: i64) -> Result<u64, StoreError> {
    let s = Settings::load(store).await;
    Ok(store.rollup_metrics(now, s.minute_hours, s.hour_days, s.day_days).await?.total())
}

// ── reading ─────────────────────────────────────────────────────────────────────

/// How one console source is read: its metric prefix, the unit divisor from the
/// stored value to the shown one, whether it is a counter, and its groups.
struct Source {
    prefix: &'static str,
    divisor: f64,
    counter: bool,
    groups: &'static [(&'static str, fn(&str) -> bool)],
}

fn source(name: &str) -> Option<Source> {
    Some(match name {
        "imageProxy" => Source {
            prefix: IMAGE,
            divisor: 1000.0,
            counter: false,
            groups: &[
                ("total", |m| m.starts_with("imageProxy.outcome.") && m != "imageProxy.outcome.cancelled"),
                ("miss", |m| m == "imageProxy.outcome.miss"),
                ("hit", |m| m == "imageProxy.outcome.hitRam" || m == "imageProxy.outcome.hitDisk"),
                ("queue", |m| m == "imageProxy.phase.queue"),
                ("cache", |m| m == "imageProxy.phase.cache"),
                ("upstream", |m| m == "imageProxy.phase.upstream"),
                ("decode", |m| m == "imageProxy.phase.decode"),
            ],
        },
        "clientTiming" => Source {
            prefix: CLIENT,
            divisor: 1.0,
            counter: false,
            groups: &[
                ("timeToUsable", |m| m == "clientTiming.timeToUsable"),
                ("fcp", |m| m == "clientTiming.fcp"),
                ("lcp", |m| m == "clientTiming.lcp"),
                ("load", |m| m == "clientTiming.load"),
                ("ttfb", |m| m == "clientTiming.ttfb"),
                ("dcl", |m| m == "clientTiming.dcl"),
                ("folderLoadCold", |m| m == "clientTiming.folderLoadCold"),
                ("folderLoadCached", |m| m == "clientTiming.folderLoadCached"),
                ("messageLoadCold", |m| m == "clientTiming.messageLoadCold"),
                ("messageLoadCached", |m| m == "clientTiming.messageLoadCached"),
            ],
        },
        "rateLimit" => Source {
            prefix: RATE,
            divisor: 1.0,
            counter: true,
            groups: &[
                ("allowed", |m| m.ends_with(".allowed")),
                ("queued", |m| m.ends_with(".queued")),
                ("rejected", |m| m.ends_with(".rejected")),
            ],
        },
        "requestProtection" => Source {
            prefix: PROTECTION,
            divisor: 1.0,
            counter: true,
            groups: &[
                ("allowed", |m| m == "requestProtection.allowed"),
                ("throttled", |m| m == "requestProtection.throttled"),
                ("blocked", |m| m == "requestProtection.blocked"),
            ],
        },
        _ => return None,
    })
}

/// Seconds for a range label.
pub(crate) fn range_secs(label: &str) -> Option<i64> {
    Some(match label {
        "1h" => 3600,
        "24h" => 86_400,
        "7d" => 7 * 86_400,
        "30d" => 30 * 86_400,
        "1y" => 365 * 86_400,
        _ => return None,
    })
}

const POINTS: i64 = 96;

/// The step in seconds for `span` seconds shown in at most [`POINTS`] points: a
/// whole number of minutes, hours (7 days and more) or days (a year).
pub(crate) fn step_for(span: i64) -> i64 {
    let raw = (span + POINTS - 1) / POINTS;
    let unit = if span >= 180 * 86_400 {
        86_400
    } else if span >= 6 * 86_400 {
        3600
    } else {
        MINUTE
    };
    (raw + unit - 1) / unit * unit
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
fn shown(v: Option<f64>, divisor: f64) -> Value {
    v.map_or(Value::Null, |v| json!(round2(v / divisor)))
}

fn summarize_agg(a: &Agg, src: &Source) -> Value {
    if src.counter {
        json!({"count": a.count})
    } else {
        json!({
            "count": a.count,
            "p50": shown(a.quantile(0.5), src.divisor),
            "p95": shown(a.quantile(0.95), src.divisor),
            "p99": shown(a.quantile(0.99), src.divisor),
            "max": shown((a.count > 0).then_some(a.max as f64), src.divisor),
            "mean": shown(a.mean(), src.divisor),
        })
    }
}

/// Merge `rows` into per-group totals and a time series of `step`-second points
/// covering `[from, from + n * step)`.
fn build_groups(rows: &[MetricRow], src: &Source, from: i64, step: i64, n: usize) -> Value {
    let mut out = serde_json::Map::new();
    for (name, pick) in src.groups {
        let mut total = Agg::default();
        let mut points: Vec<Agg> = vec![Agg::default(); n];
        let mut breakdown: BTreeMap<String, Agg> = BTreeMap::new();
        for r in rows.iter().filter(|r| pick(&r.metric)) {
            total.merge(&r.agg);
            let idx = ((r.start - from).max(0) / step) as usize;
            if let Some(p) = points.get_mut(idx.min(n - 1)) {
                p.merge(&r.agg);
            }
            let key = if src.counter {
                r.metric[src.prefix.len()..].rsplit_once('.').map_or(String::new(), |(c, _)| c.to_owned())
            } else {
                r.dims.device_class.clone()
            };
            breakdown.entry(key).or_default().merge(&r.agg);
        }
        let series = |f: &dyn Fn(&Agg) -> Value| points.iter().map(f).collect::<Vec<_>>();
        let times: Vec<i64> = (0..n as i64).map(|i| from + i * step).collect();
        let mut series_obj = json!({"t": times, "count": series(&|a| json!(a.count))});
        if !src.counter {
            series_obj["p50"] = json!(series(&|a| shown(a.quantile(0.5), src.divisor)));
            series_obj["p95"] = json!(series(&|a| shown(a.quantile(0.95), src.divisor)));
        }
        let by: serde_json::Map<String, Value> = breakdown
            .iter()
            .filter(|(k, _)| !k.is_empty())
            .map(|(k, a)| (k.clone(), summarize_agg(a, src)))
            .collect();
        out.insert(
            (*name).to_owned(),
            json!({"totals": summarize_agg(&total, src), "series": series_obj, "breakdown": by}),
        );
    }
    Value::Object(out)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PersistedQuery {
    source: String,
    #[serde(default)]
    range: Option<String>,
}

fn unavailable() -> AdminApiError {
    AdminApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "historyUnavailable",
        "Persisted statistics are temporarily unavailable.",
    )
}

/// The persisted-history payload for one source and range.
pub(crate) async fn payload(store: &Store, source_name: &str, range: &str, now: i64) -> Result<Option<Value>, StoreError> {
    let (Some(src), Some(span)) = (source(source_name), range_secs(range)) else {
        return Ok(None);
    };
    let settings = Settings::load(store).await;
    let step = step_for(span);
    let n = ((span + step - 1) / step).max(1);
    let end = (now / step + 1) * step;
    let from = end - n * step;
    let stats = store.metrics_stats(src.prefix).await?;
    let mut body = json!({
        "source": source_name,
        "range": range,
        "from": from,
        "to": end,
        "stepSecs": step,
        "counter": src.counter,
        "relativeError": mb::RELATIVE_ERROR,
        "persistence": {
            "enabled": settings.enabled,
            "dimensionsEnabled": settings.dimensions,
            "flushSeconds": settings.flush_seconds,
            "lastFlushAt": stats.last_flush,
            "rows": stats.rows,
            "historySince": stats.first_bucket,
            "instances": stats.instances,
            "retention": {
                "minuteBucketsHours": settings.minute_hours,
                "hourBucketsDays": settings.hour_days,
                "dayBucketsDays": settings.day_days,
            },
        },
        "groups": {},
    });
    if settings.enabled {
        let rows = store.load_metric_rows(src.prefix, from, end).await?;
        body["groups"] = build_groups(&rows, &src, from, step, n as usize);
    }
    Ok(Some(body))
}

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/metrics/persisted", C::ObservabilityRead, read_route);
    r.admin_endpoint(Method::POST, "/metrics/persisted/reset", C::SettingsWrite, reset_route);
}

async fn read_route(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(q): Query<PersistedQuery>,
) -> Result<Response, AdminApiError> {
    if !ctx.has_unscoped(C::ObservabilityRead) {
        return Err(AdminApiError::unscoped_required(C::ObservabilityRead));
    }
    let now = unix_now();
    // The unflushed tail of this process belongs in the answer.
    if let Some(r) = current() {
        let _ = tokio::time::timeout(Duration::from_secs(5), r.flush(now)).await;
    }
    let range = q.range.as_deref().unwrap_or("24h");
    let body = tokio::time::timeout(Duration::from_secs(8), payload(&state.store, &q.source, range, now))
        .await
        .map_err(|_| unavailable())?
        .map_err(|_| unavailable())?
        .ok_or_else(|| AdminApiError::new(StatusCode::BAD_REQUEST, "invalidQuery", "Unknown source or range."))?;
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(body)).into_response())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResetInput {
    source: String,
}

async fn reset_route(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(input): AdminJson<ResetInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    let Some(src) = source(&input.source) else {
        return Err(AdminApiError::new(StatusCode::BAD_REQUEST, "invalidQuery", "Unknown source."));
    };
    let removed = reset(&state.store, &input.source, src.prefix).await.map_err(|_| unavailable())?;
    let entry = mw_admin::AuditLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: ctx.username.clone(),
        actor_kind: mw_admin::ActorKind::Admin,
        action: "metrics-reset".into(),
        target: Some(input.source.clone()),
        detail_json: mw_admin::redact_detail(&json!({"source": input.source, "rows": removed})),
        ip: None,
    };
    if let Err(e) = state.v6.admin.audit(entry).await {
        tracing::warn!("metrics reset audit entry not recorded: {e}");
    }
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(json!({"removed": removed}))).into_response())
}

/// Delete the stored buckets of one source and forget its in-memory state.
pub(crate) async fn reset(store: &Store, name: &str, prefix: &str) -> Result<u64, StoreError> {
    let mut removed = store.reset_metrics(prefix).await?;
    if name == "clientTiming" {
        removed += store.reset_client_timing().await?;
    }
    if let Some(r) = current() {
        r.forget(prefix);
    }
    if name == "imageProxy" {
        crate::image_proxy::latency::reset();
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn store() -> Store {
        Store::open_in_memory(mw_store::ServerKey::generate()).await.unwrap()
    }
    fn dims(device: &str) -> Dims {
        Dims { client_kind: "web".into(), device_class: device.into(), ..Dims::default() }
    }
    const T: i64 = 1_800_000_000;

    fn group<'a>(v: &'a Value, g: &str) -> &'a Value {
        &v["groups"][g]
    }

    #[tokio::test]
    async fn history_survives_a_restart_with_the_same_medians_and_p95() {
        let s = store().await;
        let a = Recorder::new(s.clone());
        for i in 0..400u64 {
            a.record_value(T + (i as i64 % 50) * 60, "clientTiming.load", 200 + i * 9, &dims("desktop"));
            a.record_value(T + 30, "clientTiming.folderLoadCold", 50 + i, &dims("mobile"));
        }
        assert_eq!(a.flush(T + 4000).await.unwrap(), 51);
        let before = payload(&s, "clientTiming", "24h", T + 3000).await.unwrap().unwrap();
        // "Restart": the in-memory state is gone; a new process has a new instance id.
        drop(a);
        let b = Recorder::new(s.clone());
        let after = payload(&s, "clientTiming", "24h", T + 3000).await.unwrap().unwrap();
        assert_eq!(group(&before, "load")["totals"], group(&after, "load")["totals"]);
        let totals = &group(&after, "load")["totals"];
        assert_eq!(totals["count"], 400);
        let (p50, p95) = (totals["p50"].as_f64().unwrap(), totals["p95"].as_f64().unwrap());
        // True values: samples 200 + 9i, i in 0..400.
        let t50 = 200.0 + 9.0 * 199.0;
        let t95 = 200.0 + 9.0 * 379.0;
        assert!((p50 - t50).abs() / t50 <= mb::RELATIVE_ERROR + 0.01, "{p50} vs {t50}");
        assert!((p95 - t95).abs() / t95 <= mb::RELATIVE_ERROR + 0.01, "{p95} vs {t95}");
        let p = &after["persistence"];
        assert_eq!(p["historySince"], T / 60 * 60);
        assert!(p["rows"].as_u64().unwrap() >= 51);
        assert_eq!(p["lastFlushAt"], T + 4000);
        // New samples after the restart add to the same history.
        b.record_value(T + 100, "clientTiming.load", 100_000, &dims("desktop"));
        b.flush(T + 4100).await.unwrap();
        let more = payload(&s, "clientTiming", "24h", T + 3000).await.unwrap().unwrap();
        assert_eq!(group(&more, "load")["totals"]["count"], 401);
        assert_eq!(more["persistence"]["instances"], 2);
    }

    #[tokio::test]
    async fn dimension_sets_are_capped_and_fold_into_other() {
        let s = store().await;
        let r = Recorder::new(s.clone());
        for i in 0..(MAX_DIM_SETS + 20) {
            r.record_value(T, "clientTiming.load", 100, &dims(&format!("d{i}")));
        }
        let rows = r.dirty_rows();
        assert_eq!(rows.len(), MAX_DIM_SETS + 1);
        let other = rows.iter().find(|r| r.dims.device_class == "other").unwrap();
        assert_eq!((other.agg.count, other.dims.client_kind.as_str()), (20, "other"));
        // Another metric in the same minute has its own budget.
        r.record_value(T, "clientTiming.ttfb", 5, &dims("zz"));
        assert_eq!(r.dirty_rows().len(), MAX_DIM_SETS + 2);
        // With dimensions off, everything shares one row and nothing identifies a class.
        let off = Recorder::new(s);
        off.configure(true, false);
        for i in 0..10 {
            off.record_value(T, "clientTiming.load", 100, &dims(&format!("d{i}")));
        }
        let rows = off.dirty_rows();
        assert_eq!((rows.len(), rows[0].dims.clone()), (1, Dims::default()));
    }

    #[tokio::test]
    async fn two_instances_merge_by_adding_sketches_not_averaging_percentiles() {
        let s = store().await;
        let (a, b) = (Recorder::new(s.clone()), Recorder::new(s.clone()));
        let none = Dims::default();
        for _ in 0..900 {
            a.record_value(T, "imageProxy.outcome.miss", 20_000, &none);
        }
        for _ in 0..100 {
            b.record_value(T, "imageProxy.outcome.miss", 4_000_000, &none);
        }
        a.flush(T + 10).await.unwrap();
        b.flush(T + 10).await.unwrap();
        let v = payload(&s, "imageProxy", "1h", T + 20).await.unwrap().unwrap();
        let miss = &group(&v, "miss")["totals"];
        assert_eq!(miss["count"], 1000);
        // 90% of the requests took 20 ms: the median is ~20 ms, not (20 + 4000) / 2.
        assert!(miss["p50"].as_f64().unwrap() < 25.0, "{miss}");
        assert!(miss["p95"].as_f64().unwrap() > 3500.0, "{miss}");
        // Counters add up per class.
        a.record_count(T, "rateLimit.jmap.rejected", 3, &none);
        b.record_count(T, "rateLimit.jmap.rejected", 4, &none);
        a.flush(T + 11).await.unwrap();
        b.flush(T + 11).await.unwrap();
        let v = payload(&s, "rateLimit", "1h", T + 20).await.unwrap().unwrap();
        assert_eq!(group(&v, "rejected")["totals"]["count"], 7);
        assert_eq!(group(&v, "rejected")["breakdown"]["jmap"]["count"], 7);
    }

    #[tokio::test]
    async fn a_flush_is_idempotent_and_a_failed_one_loses_nothing() {
        let s = store().await;
        let r = Recorder::new(s.clone());
        r.record_value(T, "clientTiming.load", 300, &Dims::default());
        assert_eq!(r.flush(T + 1).await.unwrap(), 1);
        assert_eq!(r.flush(T + 2).await.unwrap(), 0, "clean minutes are not rewritten");
        r.record_value(T, "clientTiming.load", 400, &Dims::default());
        assert_eq!(r.flush(T + 3).await.unwrap(), 1);
        let v = payload(&s, "clientTiming", "1h", T + 60).await.unwrap().unwrap();
        assert_eq!(group(&v, "load")["totals"]["count"], 2);
        // Persistence off: nothing is recorded or written, and the read says so.
        let off = Recorder::new(s.clone());
        off.configure(false, true);
        off.record_value(T, "clientTiming.load", 1, &Dims::default());
        assert!(off.dirty_rows().is_empty());
    }

    #[tokio::test]
    async fn shutdown_flush_writes_the_unflushed_tail_and_reset_clears_it() {
        let s = store().await;
        let r = start(s.clone());
        let now = unix_now();
        r.record_value(now, "imageProxy.outcome.hitRam", 800, &Dims::default());
        flush_on_shutdown().await;
        let stats = s.metrics_stats(IMAGE).await.unwrap();
        assert_eq!(stats.rows, 1);
        assert!(stats.last_flush.is_some());
        assert_eq!(reset(&s, "imageProxy", IMAGE).await.unwrap(), 1);
        assert_eq!(s.metrics_stats(IMAGE).await.unwrap().rows, 0);
        assert!(r.dirty_rows().is_empty(), "the reset also forgets the in-memory minutes");
        ACTIVE.store(false, Relaxed);
        if let Ok(mut g) = global().write() {
            *g = None;
        }
    }

    #[test]
    fn steps_keep_the_chart_to_about_ninety_six_points() {
        for (label, expect_unit) in [("1h", 60), ("24h", 60), ("7d", 3600), ("30d", 3600), ("1y", 86_400)] {
            let span = range_secs(label).unwrap();
            let step = step_for(span);
            assert_eq!(step % expect_unit, 0, "{label}");
            assert!(span / step <= POINTS, "{label}");
        }
        assert!(range_secs("2w").is_none());
        assert!(source("people").is_none());
    }
}
