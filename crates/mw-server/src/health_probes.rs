//! Health probes behind the admin dashboard's Health panel and `/metrics` (t103).
//!
//! Every component reports the same shape:
//!
//! ```json
//! {"status":"ok|warn|fail|disabled","reason":"…","metrics":{…},"thresholds":{…},
//!  "enable":"how to turn it on (disabled only)","probeUs":123}
//! ```
//!
//! `disabled` means this deployment turned the component off (proxy mode, no
//! plugin enabled) and `enable` says how to turn it on. A probe that does not
//! answer in [`PROBE_TIMEOUT`] is `fail` with that reason; nothing is left as
//! "not reported".
//!
//! Cost: each probe is a handful of indexed reads or in-process counters. The
//! render ping spawns the worker once with a `{"ping":true}` frame and no mail.
//! The submission reachability check is a TLS/banner handshake with no
//! credentials, at most [`REACH_TARGETS`] endpoints, refreshed at most once a
//! minute in the background. The whole result is cached for [`CACHE_TTL`] per
//! server, so reloading the dashboard does not re-probe.

use crate::AppState;
use crate::plugins::PluginRegistry;
use serde_json::{Map, Value, json};
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// The longest any single probe may take before it is reported as failed.
pub(crate) const PROBE_TIMEOUT: Duration = Duration::from_millis(1500);
/// How long a computed result is reused.
pub(crate) const CACHE_TTL: Duration = Duration::from_secs(5);
/// How long a submission reachability result is reused.
const REACH_TTL: Duration = Duration::from_secs(60);
/// Submission endpoints checked at most.
const REACH_TARGETS: usize = 4;
/// History samples kept (one per [`HISTORY_STEP`] at most).
const HISTORY_LEN: usize = 120;
const HISTORY_STEP: Duration = Duration::from_secs(30);
/// The `/metrics` refresher's interval.
const METRICS_INTERVAL: Duration = Duration::from_secs(60);

// ── thresholds ───────────────────────────────────────────────────────────────

const DB_PING_WARN_MS: f64 = 50.0;
const DB_PING_FAIL_MS: f64 = 500.0;
const OUTBOX_OLDEST_WARN_SECS: i64 = 600;
const OUTBOX_OLDEST_FAIL_SECS: i64 = 3600;
const SEARCH_PENDING_WARN: u64 = 500;
const RENDER_PING_WARN_MS: f64 = 1000.0;
const SYNC_ERRORING_FAIL_RATIO: f64 = 0.5;

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn component(status: &str, reason: impl Into<String>, metrics: Value, thresholds: Value) -> Value {
    json!({"status":status,"reason":reason.into(),"metrics":metrics,"thresholds":thresholds})
}

fn disabled(reason: &str, enable: &str) -> Value {
    json!({"status":"disabled","reason":reason,"enable":enable,"metrics":{},"thresholds":{}})
}

/// Run `probe` under [`PROBE_TIMEOUT`] and stamp how long it took.
async fn timed(name: &str, probe: impl Future<Output = Value>) -> Value {
    let start = Instant::now();
    let mut value = match tokio::time::timeout(PROBE_TIMEOUT, probe).await {
        Ok(v) => v,
        Err(_) => component(
            "fail",
            format!(
                "The {name} probe did not answer within {} ms.",
                PROBE_TIMEOUT.as_millis()
            ),
            json!({}),
            json!({"timeoutMs":PROBE_TIMEOUT.as_millis() as u64}),
        ),
    };
    value["probeUs"] = json!(start.elapsed().as_micros() as u64);
    value
}

// ── render worker telemetry ──────────────────────────────────────────────────

/// How one render job ended.
#[derive(Clone, Copy)]
pub(crate) enum RenderOutcome {
    Ok,
    Failed,
    Timeout,
    /// Refused because every preview worker slot was busy.
    Busy,
}

struct RenderStats {
    in_flight: AtomicU64,
    ok_total: AtomicU64,
    failed_total: AtomicU64,
    timeout_total: AtomicU64,
    busy_total: AtomicU64,
    recent: Mutex<VecDeque<(Instant, u8)>>,
}

static RENDER: RenderStats = RenderStats {
    in_flight: AtomicU64::new(0),
    ok_total: AtomicU64::new(0),
    failed_total: AtomicU64::new(0),
    timeout_total: AtomicU64::new(0),
    busy_total: AtomicU64::new(0),
    recent: Mutex::new(VecDeque::new()),
};

const HOUR: Duration = Duration::from_secs(3600);

fn render_record(outcome: RenderOutcome) {
    let (counter, code) = match outcome {
        RenderOutcome::Ok => (&RENDER.ok_total, 0),
        RenderOutcome::Failed => (&RENDER.failed_total, 1),
        RenderOutcome::Timeout => (&RENDER.timeout_total, 2),
        RenderOutcome::Busy => (&RENDER.busy_total, 3),
    };
    counter.fetch_add(1, Ordering::Relaxed);
    if code == 0 {
        return;
    }
    let now = Instant::now();
    let mut recent = RENDER.recent.lock().unwrap_or_else(|e| e.into_inner());
    recent.push_back((now, code));
    while recent
        .front()
        .is_some_and(|(t, _)| now.duration_since(*t) > HOUR)
        || recent.len() > 4096
    {
        recent.pop_front();
    }
}

pub(crate) fn render_busy() {
    render_record(RenderOutcome::Busy);
}

struct InFlight;
impl InFlight {
    fn start() -> Self {
        RENDER.in_flight.fetch_add(1, Ordering::Relaxed);
        InFlight
    }
}
impl Drop for InFlight {
    fn drop(&mut self) {
        RENDER.in_flight.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Count one render job: in flight while it runs, then by outcome.
pub(crate) async fn track_render<T>(
    job: impl Future<Output = anyhow::Result<T>>,
) -> anyhow::Result<T> {
    let _in_flight = InFlight::start();
    let result = job.await;
    render_record(if result.is_ok() {
        RenderOutcome::Ok
    } else {
        RenderOutcome::Failed
    });
    result
}

/// [`track_render`] under a deadline; a job that overruns counts as a timeout.
pub(crate) async fn track_render_timeout<T>(
    limit: Duration,
    job: impl Future<Output = anyhow::Result<T>>,
) -> Result<anyhow::Result<T>, tokio::time::error::Elapsed> {
    let _in_flight = InFlight::start();
    let result = tokio::time::timeout(limit, job).await;
    render_record(match &result {
        Ok(Ok(_)) => RenderOutcome::Ok,
        Ok(Err(_)) => RenderOutcome::Failed,
        Err(_) => RenderOutcome::Timeout,
    });
    result
}

fn render_recent() -> (u64, u64, u64) {
    let now = Instant::now();
    let mut recent = RENDER.recent.lock().unwrap_or_else(|e| e.into_inner());
    while recent
        .front()
        .is_some_and(|(t, _)| now.duration_since(*t) > HOUR)
    {
        recent.pop_front();
    }
    let count = |c| recent.iter().filter(|(_, k)| *k == c).count() as u64;
    (count(1), count(2), count(3))
}

/// Spawn the worker with a ping frame and time the round trip.
async fn render_ping(bin: &std::path::Path) -> anyhow::Result<(Duration, String, Option<String>)> {
    use std::process::Stdio;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let start = Instant::now();
    let mut child = tokio::process::Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("no child stdin"))?;
    stdin.write_all(b"{\"ping\":true}\n").await?;
    stdin.shutdown().await?;
    drop(stdin);
    let mut out = String::new();
    let mut err = String::new();
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("no child stdout"))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("no child stderr"))?;
    let mut stdout = (&mut stdout).take(64 * 1024);
    let mut stderr = (&mut stderr).take(16 * 1024);
    let (a, b) = tokio::join!(
        stdout.read_to_string(&mut out),
        stderr.read_to_string(&mut err)
    );
    a?;
    b?;
    let status = child.wait().await?;
    let elapsed = start.elapsed();
    if !status.success() {
        let why = err
            .lines()
            .last()
            .unwrap_or("no message")
            .trim()
            .to_string();
        anyhow::bail!("the worker exited with {status}: {why}");
    }
    let value: Value = serde_json::from_str(out.trim_end())?;
    let version = value["version"]
        .as_str()
        .ok_or_else(|| {
            anyhow::anyhow!("the worker answered without a version (built before t103)")
        })?
        .to_string();
    let degraded = err
        .lines()
        .find_map(|l| l.strip_prefix("mw-render: jail degraded: "))
        .map(str::to_string);
    Ok((elapsed, version, degraded))
}

async fn probe_render(state: &AppState) -> Value {
    let (failed, timeouts, busy) = render_recent();
    let mut metrics = json!({
        "inFlight": RENDER.in_flight.load(Ordering::Relaxed),
        "failuresLastHour": failed,
        "timeoutsLastHour": timeouts,
        "busyRefusalsLastHour": busy,
        "jobsOkTotal": RENDER.ok_total.load(Ordering::Relaxed),
        "jobsFailedTotal": RENDER.failed_total.load(Ordering::Relaxed),
        "jobsTimedOutTotal": RENDER.timeout_total.load(Ordering::Relaxed),
        "jailRequired": state.render_jail_required,
    });
    let thresholds =
        json!({"pingWarnMs":RENDER_PING_WARN_MS,"pingTimeoutMs":PROBE_TIMEOUT.as_millis() as u64});
    let Some(bin) = &state.render_bin else {
        let (status, reason) = if state.render_jail_required {
            (
                "fail",
                "No mw-render worker binary was found and a kernel jail is required here, so message sanitizing, attachment previews and .oft import are refused. Install mw-render beside the server binary or set MW_RENDER_BIN.",
            )
        } else {
            (
                "warn",
                "No mw-render worker binary was found. Message HTML is sanitized in the server process without isolation; attachment previews and .oft import are off. Install mw-render beside the server binary or set MW_RENDER_BIN.",
            )
        };
        metrics["workerConfigured"] = json!(false);
        return component(status, reason, metrics, thresholds);
    };
    metrics["workerConfigured"] = json!(true);
    metrics["queueDepth"] = metrics["inFlight"].clone();
    match tokio::time::timeout(PROBE_TIMEOUT - Duration::from_millis(100), render_ping(bin)).await {
        Ok(Ok((elapsed, version, degraded))) => {
            let ms = elapsed.as_secs_f64() * 1000.0;
            metrics["pingMs"] = json!(ms);
            metrics["version"] = json!(version);
            metrics["jailDegraded"] = json!(degraded);
            let server = env!("CARGO_PKG_VERSION");
            let (status, reason) = if version != server {
                (
                    "warn",
                    format!(
                        "The worker is version {version}; this server is {server}. Install the matching mw-render."
                    ),
                )
            } else if failed + timeouts > 0 {
                (
                    "warn",
                    format!(
                        "The worker answers, but {failed} render job(s) failed and {timeouts} timed out in the last hour."
                    ),
                )
            } else if ms > RENDER_PING_WARN_MS {
                ("warn", "The worker answers slowly.".to_string())
            } else if let Some(why) = &degraded {
                (
                    "ok",
                    format!("The worker answers. Kernel jail degraded on this platform: {why}."),
                )
            } else {
                ("ok", "The worker answers.".to_string())
            };
            component(status, reason, metrics, thresholds)
        }
        Ok(Err(e)) => component(
            "fail",
            format!("The worker did not answer a ping: {e}"),
            metrics,
            thresholds,
        ),
        Err(_) => component(
            "fail",
            "The worker did not answer a ping in time.",
            metrics,
            thresholds,
        ),
    }
}

// ── database and migrations ──────────────────────────────────────────────────

async fn probe_database(state: &AppState) -> Value {
    let diag = match state.store.store_diagnostics().await {
        Ok(d) => d,
        Err(e) => {
            return component(
                "fail",
                format!("The database did not answer: {e}"),
                json!({}),
                json!({}),
            );
        }
    };
    let info = state.store.database_info().await.ok();
    let in_use = (diag.pool_size as usize).saturating_sub(diag.pool_idle);
    let metrics = json!({
        "pingMs": diag.db_ping_ms,
        "backend": diag.backend,
        "poolSize": diag.pool_size,
        "poolIdle": diag.pool_idle,
        "poolInUse": in_use,
        "dbBytes": diag.db_bytes,
        "walBytes": diag.wal_bytes,
        "serverVersion": info.as_ref().and_then(|i| i.server_version.clone()),
        "name": info.as_ref().and_then(|i| i.name.clone()),
        "host": info.as_ref().and_then(|i| i.host.clone()),
        "port": info.as_ref().and_then(|i| i.port),
        "journalMode": info.as_ref().and_then(|i| i.journal_mode.clone()),
        // The store does not time individual statements, so there is no slow
        // query count to report; the reason travels with the null.
        "slowQueriesLastHour": null,
    });
    let thresholds = json!({"pingWarnMs":DB_PING_WARN_MS,"pingFailMs":DB_PING_FAIL_MS});
    let (status, reason) = if diag.db_ping_ms > DB_PING_FAIL_MS {
        ("fail", "The database answers, but very slowly.")
    } else if diag.db_ping_ms > DB_PING_WARN_MS {
        ("warn", "The database answers slowly.")
    } else {
        ("ok", "The database answers.")
    };
    let mut value = component(status, reason, metrics, thresholds);
    value["notes"] =
        json!({"slowQueriesLastHour":"Not tracked: the store does not time individual queries."});
    value
}

async fn probe_migrations(state: &AppState) -> Value {
    match state.store.migration_detail().await {
        Ok(m) => {
            let (status, reason) = if !m.failed.is_empty() {
                (
                    "fail",
                    format!(
                        "{} migration(s) started but did not complete.",
                        m.failed.len()
                    ),
                )
            } else if !m.pending.is_empty() {
                (
                    "warn",
                    format!(
                        "{} migration(s) in this build are not applied yet.",
                        m.pending.len()
                    ),
                )
            } else if !m.unknown.is_empty() {
                (
                    "warn",
                    format!(
                        "The database has {} migration(s) from a newer build than this one.",
                        m.unknown.len()
                    ),
                )
            } else {
                ("ok", "Up to date.".to_string())
            };
            component(
                status,
                reason,
                serde_json::to_value(&m).unwrap_or_default(),
                json!({}),
            )
        }
        Err(e) => component(
            "fail",
            format!("The migration table could not be read: {e}"),
            json!({}),
            json!({}),
        ),
    }
}

// ── mail sync ────────────────────────────────────────────────────────────────

fn percentile(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted.get(rank).copied()
}

const ENGINE_ENABLE: &str = "Set MW_MODE=engine and restart. In proxy mode this server relays each request to the mail server and keeps no mail, sync state, outbox or search index of its own.";

async fn probe_sync(state: &AppState) -> Value {
    let Some(engine) = &state.engine else {
        return disabled(
            "Proxy mode: this server does not synchronize mailboxes.",
            ENGINE_ENABLE,
        );
    };
    let snap = engine.sync_health();
    let kinds: HashMap<String, &'static str> = state
        .store
        .list_accounts()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|a| {
            (
                a.id,
                match a.kind {
                    mw_store::AccountKind::Imap => "imap",
                    mw_store::AccountKind::Pop3 => "pop3",
                    mw_store::AccountKind::Ews => "ews",
                },
            )
        })
        .collect();
    let mut by_kind: std::collections::BTreeMap<&str, (u64, u64, u64)> = Default::default();
    let (mut syncing, mut idle, mut erroring) = (0u64, 0u64, 0u64);
    let mut ages = Vec::new();
    let mut never = 0u64;
    let mut not_watching = 0u64;
    let mut errors = Vec::new();
    for a in &snap.accounts {
        match a.state() {
            "syncing" => syncing += 1,
            "erroring" => erroring += 1,
            _ => idle += 1,
        }
        let kind = if a.plugin_backed {
            "bridge"
        } else {
            kinds.get(&a.account_id).copied().unwrap_or("other")
        };
        let entry = by_kind.entry(kind).or_default();
        entry.0 += 1;
        if a.watching {
            entry.1 += 1;
        } else {
            not_watching += 1;
        }
        if a.syncing {
            entry.2 += 1;
        }
        match a.last_ok_age_secs {
            Some(age) => ages.push(age),
            None => never += 1,
        }
        if let Some(e) = &a.last_error
            && errors.len() < 5
        {
            errors.push(json!({"kind":kind,"error":e,"ageSecs":a.last_error_age_secs,"consecutiveFailures":a.consecutive_failures}));
        }
    }
    ages.sort_by(|a, b| a.total_cmp(b));
    let total = snap.accounts.len() as u64;
    let metrics = json!({
        "accounts": total,
        "syncing": syncing,
        "idle": idle,
        "erroring": erroring,
        "notWatching": not_watching,
        "neverSynced": never,
        "lastSyncAgeP50Secs": percentile(&ages, 0.5),
        "lastSyncAgeMaxSecs": ages.last().copied(),
        "queueDepth": snap.resyncs_running,
        "errorsLastHour": snap.sync_errors_last_hour,
        "byKind": by_kind.iter().map(|(k, (n, w, s))| (k.to_string(), json!({"accounts":n,"watchConnections":w,"syncing":s}))).collect::<Map<_, _>>(),
        "recentErrors": errors,
    });
    let thresholds = json!({"erroringFailRatio":SYNC_ERRORING_FAIL_RATIO});
    let (status, reason) = if total == 0 {
        (
            "ok",
            "No mailbox is signed in right now; mailboxes register when someone signs in."
                .to_string(),
        )
    } else if erroring as f64 / total as f64 >= SYNC_ERRORING_FAIL_RATIO {
        (
            "fail",
            format!("{erroring} of {total} mailbox(es) are failing to sync."),
        )
    } else if erroring > 0 {
        (
            "warn",
            format!("{erroring} of {total} mailbox(es) are failing to sync."),
        )
    } else if not_watching > 0 {
        (
            "warn",
            format!(
                "{not_watching} mailbox(es) have no change watch, so new mail arrives only on the next sync."
            ),
        )
    } else {
        (
            "ok",
            format!("{total} mailbox(es) registered; none failing."),
        )
    };
    component(status, reason, metrics, thresholds)
}

// ── outbox ───────────────────────────────────────────────────────────────────

struct Reach {
    checked: Instant,
    value: Value,
}

fn submission_backends(row: &mw_store::DomainConfigRow) -> Vec<crate::domain_selection::Backend> {
    use crate::domain_selection::{Backend, BackendConfig, Protocol, ServiceRole};
    if !row.enabled {
        return Vec::new();
    }
    let Ok(value) = serde_json::from_str::<Value>(&row.upstream_json) else {
        return Vec::new();
    };
    if value.get("version").is_some() {
        return serde_json::from_value::<BackendConfig>(value)
            .ok()
            .and_then(|c| c.selected(ServiceRole::Submission).cloned())
            .into_iter()
            .collect();
    }
    crate::domain_config::UpstreamConfig::parse(&row.upstream_json)
        .ok()
        .and_then(|c| c.smtp)
        .map(|endpoint| Backend {
            id: "smtp".into(),
            role: ServiceRole::Submission,
            protocol: Protocol::Smtp,
            enabled: true,
            preferred: false,
            priority: 100,
            endpoint: Some(endpoint),
            session_url: None,
            ews: None,
            username: Default::default(),
        })
        .into_iter()
        .collect()
}

/// Handshake (banner + TLS, no credentials) with up to [`REACH_TARGETS`]
/// distinct submission endpoints.
async fn check_reach(state: AppState) -> Value {
    let rows = state
        .store
        .list_domain_configs_limited(129)
        .await
        .unwrap_or_default();
    let mut seen = std::collections::HashSet::new();
    let mut targets = Vec::new();
    let mut configured = 0usize;
    for row in &rows {
        for backend in submission_backends(row) {
            let key = serde_json::to_string(&backend.endpoint).unwrap_or_default()
                + backend.session_url.as_deref().unwrap_or("");
            if seen.insert(key) {
                configured += 1;
                if targets.len() < REACH_TARGETS {
                    targets.push((row.name.clone(), backend));
                }
            }
        }
    }
    if targets.is_empty() {
        return json!({"configured":0,"checked":0,"reachable":0,"unreachable":0,"notCheckable":0});
    }
    let Ok(plan) = crate::image_proxy::load_plan(&state.store).await else {
        return json!({"configured":configured,"checked":0,"error":"The outbound connection policy could not be loaded."});
    };
    let checks = targets.iter().map(|(domain, backend)| {
        let plan = plan.clone();
        async move {
            let start = Instant::now();
            let detail = tokio::time::timeout(
                Duration::from_millis(1200),
                crate::domain_probe::diagnostic_backend(domain, backend, &plan),
            )
            .await
            .unwrap_or_else(|_| json!({"reachable":false,"selectedReason":"timeout"}));
            (start.elapsed(), detail)
        }
    });
    let results = futures_util::future::join_all(checks).await;
    let (mut reachable, mut unreachable, mut not_checkable) = (0, 0, 0);
    let mut slowest: f64 = 0.0;
    for (elapsed, detail) in &results {
        if detail["selectedReason"] == "authenticated-provider-probe-required" {
            not_checkable += 1;
        } else if detail["reachable"] == true {
            reachable += 1;
            slowest = slowest.max(elapsed.as_secs_f64() * 1000.0);
        } else {
            unreachable += 1;
        }
    }
    json!({"configured":configured,"checked":results.len(),"reachable":reachable,"unreachable":unreachable,
        "notCheckable":not_checkable,"slowestHandshakeMs":(reachable>0).then_some(slowest),"credentialsSent":false})
}

fn reach_cache() -> &'static Mutex<HashMap<usize, Reach>> {
    static CACHE: OnceLock<Mutex<HashMap<usize, Reach>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

fn reach_refreshing() -> &'static Mutex<std::collections::HashSet<usize>> {
    static SET: OnceLock<Mutex<std::collections::HashSet<usize>>> = OnceLock::new();
    SET.get_or_init(Default::default)
}

/// The last reachability result, refreshed in the background when older than
/// [`REACH_TTL`]. Waits for the first one only, within the probe budget.
async fn reachability(state: &AppState) -> Option<(Value, f64)> {
    let key = server_key(state);
    let cached = reach_cache()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&key)
        .map(|r| (r.value.clone(), r.checked.elapsed()));
    let stale = cached.as_ref().is_none_or(|(_, age)| *age > REACH_TTL);
    if stale
        && reach_refreshing()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(key)
    {
        let state = state.clone();
        let task = tokio::spawn(async move {
            let value = check_reach(state).await;
            reach_cache()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(
                    key,
                    Reach {
                        checked: Instant::now(),
                        value,
                    },
                );
            reach_refreshing()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&key);
        });
        if cached.is_none() {
            let _ = tokio::time::timeout(Duration::from_millis(1300), task).await;
        }
    }
    reach_cache()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&key)
        .map(|r| (r.value.clone(), r.checked.elapsed().as_secs_f64()))
}

async fn probe_outbox(state: &AppState) -> Value {
    let Some(engine) = &state.engine else {
        return disabled(
            "Proxy mode: messages are handed to the mail server at once, so there is no outbox here.",
            ENGINE_ENABLE,
        );
    };
    let counts = match state.store.outbox_counts(unix_now()).await {
        Ok(c) => c,
        Err(e) => {
            return component(
                "fail",
                format!("The outbox could not be read: {e}"),
                json!({}),
                json!({}),
            );
        }
    };
    let snap = engine.sync_health();
    let reach = reachability(state).await;
    let mut metrics = serde_json::to_value(&counts).unwrap_or_default();
    metrics["sendsLastHour"] = serde_json::to_value(&snap.sends_last_hour).unwrap_or_default();
    metrics["lastSend"] = serde_json::to_value(&snap.last_send).unwrap_or_default();
    metrics["submission"] = reach
        .as_ref()
        .map(|(v, age)| {
            let mut v = v.clone();
            v["ageSecs"] = json!(age);
            v
        })
        .unwrap_or(Value::Null);
    let thresholds = json!({"oldestDueWarnSecs":OUTBOX_OLDEST_WARN_SECS,"oldestDueFailSecs":OUTBOX_OLDEST_FAIL_SECS});
    let oldest = counts.oldest_due_age_secs.unwrap_or(0);
    let unreachable = reach.as_ref().map(|(v, _)| {
        (
            v["unreachable"].as_u64().unwrap_or(0),
            v["reachable"].as_u64().unwrap_or(0),
        )
    });
    let (status, reason) = if oldest > OUTBOX_OLDEST_FAIL_SECS {
        (
            "fail",
            "A message has been waiting to send for over an hour.".to_string(),
        )
    } else if let Some((bad, 0)) = unreachable
        && bad > 0
    {
        (
            "fail",
            "No submission server answered a connection check.".to_string(),
        )
    } else if oldest > OUTBOX_OLDEST_WARN_SECS {
        (
            "warn",
            "A message has been waiting to send for over 10 minutes.".to_string(),
        )
    } else if counts.deferred > 0
        || snap.sends_last_hour.failed > 0
        || snap.sends_last_hour.uncertain > 0
    {
        (
            "warn",
            format!(
                "{} message(s) are waiting to retry; {} send(s) failed in the last hour.",
                counts.deferred,
                snap.sends_last_hour.failed + snap.sends_last_hour.uncertain
            ),
        )
    } else if let Some((bad, _)) = unreachable
        && bad > 0
    {
        (
            "warn",
            format!("{bad} submission server(s) did not answer a connection check."),
        )
    } else {
        ("ok", "Nothing is stuck.".to_string())
    };
    component(status, reason, metrics, thresholds)
}

// ── search index ─────────────────────────────────────────────────────────────

/// Bytes under `dir` and the age of the last commit (Tantivy rewrites
/// `meta.json` on each commit). Bounded at 20 000 entries.
fn index_disk(dir: &std::path::Path) -> (Option<u64>, Option<f64>) {
    let mut total = 0u64;
    let mut stack = vec![dir.to_path_buf()];
    let mut seen = 0usize;
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            seen += 1;
            if seen > 20_000 {
                return (None, None);
            }
            match e.metadata() {
                Ok(m) if m.is_dir() => stack.push(e.path()),
                Ok(m) => total += m.len(),
                Err(_) => {}
            }
        }
    }
    let commit = std::fs::metadata(dir.join("meta.json"))
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .map(|d| d.as_secs_f64());
    (Some(total), commit)
}

async fn probe_search(state: &AppState) -> Value {
    let Some(engine) = &state.engine else {
        return disabled(
            "Proxy mode: mail is not stored on this server, so there is no search index; searches go to the mail server.",
            ENGINE_ENABLE,
        );
    };
    let status = match engine.search_index_status().await {
        Ok(s) => s,
        Err(e) => {
            return component(
                "fail",
                format!("The search index could not be read: {e}"),
                json!({"present":true}),
                json!({}),
            );
        }
    };
    let pending = status.messages.saturating_sub(status.documents);
    let progress = (status.rebuilding && status.rebuild_total > 0)
        .then(|| (status.rebuild_done as f64 / status.rebuild_total as f64 * 100.0).min(100.0));
    let (bytes, commit_age) = match engine.search_index_dir().map(|d| d.to_path_buf()) {
        Some(dir) => tokio::task::spawn_blocking(move || index_disk(&dir))
            .await
            .unwrap_or((None, None)),
        None => (None, None),
    };
    let metrics = json!({
        "present": true,
        "persistent": status.persistent,
        "documents": status.documents,
        "indexableMessages": status.messages,
        "pending": pending,
        "zeroAccessDocuments": status.zero_access_documents,
        "bytes": bytes,
        "lastCommitAgeSecs": commit_age,
        "rebuilding": status.rebuilding,
        "rebuildProgressPct": progress,
        "rebuildDone": status.rebuild_done,
        "rebuildTotal": status.rebuild_total,
    });
    let thresholds = json!({"pendingWarn":SEARCH_PENDING_WARN});
    let (st, reason) = if status.zero_access_documents > 0 {
        (
            "fail",
            format!(
                "The index holds {} document(s) of zero-access accounts; rebuild it under Maintenance.",
                status.zero_access_documents
            ),
        )
    } else if status.rebuilding {
        ("warn", "A rebuild is running.".to_string())
    } else if pending > SEARCH_PENDING_WARN {
        (
            "warn",
            format!("{pending} stored message(s) are not in the index yet."),
        )
    } else if !status.persistent {
        ("ok", "Present, kept in memory: it is rebuilt from the store at every start. Set MW_SEARCH_DIR to a directory to keep it on disk.".to_string())
    } else {
        ("ok", "Present and in step with the store.".to_string())
    };
    component(st, reason, metrics, thresholds)
}

// ── plugins ──────────────────────────────────────────────────────────────────

async fn probe_plugins(state: &AppState, reg: Option<&PluginRegistry>) -> Value {
    let failures = mw_plugin::health::call_failures();
    let calls = json!({
        "limitTripsLastHour": failures.limit_last_hour,
        "trapsLastHour": failures.trap_last_hour,
        "runtimeErrorsLastHour": failures.runtime_last_hour,
        "limitTripsTotal": failures.limit_total,
        "trapsTotal": failures.trap_total,
        "runtimeErrorsTotal": failures.runtime_total,
    });
    if state.engine.is_none() {
        return disabled(
            "Proxy mode: engine plugins (account bridges, spam classifiers) run only in engine mode.",
            ENGINE_ENABLE,
        );
    }
    let Some(reg) = reg else {
        return component(
            "fail",
            "The plugin host is not attached to this route.",
            calls,
            json!({}),
        );
    };
    let start = Instant::now();
    let host_ok = reg.lock().is_ok();
    let ping_ms = start.elapsed().as_secs_f64() * 1000.0;
    let rows = match state.store.list_plugins().await {
        Ok(r) => r,
        Err(e) => {
            return component(
                "fail",
                format!("The plugin registry could not be read: {e}"),
                calls,
                json!({}),
            );
        }
    };
    let mut plugins = Vec::new();
    let (mut loaded, mut not_loaded) = (0u64, 0u64);
    for row in rows.iter().filter(|r| r.enabled).take(64) {
        let manifest = crate::v7_mount::manifest_of(row);
        let st = crate::v7_mount::plugin_status(&state.store, reg, row).await;
        if st.loaded {
            loaded += 1;
        } else {
            not_loaded += 1;
        }
        plugins.push(json!({
            "id": row.id, "name": row.name, "version": row.version, "loaded": st.loaded,
            "notLoadedReason": st.not_loaded.map(crate::v7_mount::NotLoaded::wire),
            "restartRequired": st.restart_required,
            "limits": {"memoryMb": manifest.limits.memory_mb, "deadlineMs": manifest.limits.deadline_ms, "fuel": manifest.limits.fuel},
        }));
    }
    let mut metrics = calls;
    metrics["installed"] = json!(rows.len());
    metrics["enabled"] = json!(plugins.len());
    metrics["loaded"] = json!(loaded);
    metrics["notLoaded"] = json!(not_loaded);
    metrics["hostPingMs"] = json!(ping_ms);
    metrics["plugins"] = json!(plugins);
    if plugins.is_empty() {
        let mut value = disabled(
            if rows.is_empty() {
                "No plugin is installed."
            } else {
                "No installed plugin is enabled."
            },
            "Install or enable a plugin under Plugins.",
        );
        value["metrics"] = metrics;
        return value;
    }
    let (status, reason) = if !host_ok {
        (
            "fail",
            "The plugin host lock is poisoned: a plugin call panicked. Restart the server."
                .to_string(),
        )
    } else if failures.trap_last_hour + failures.limit_last_hour > 0 {
        (
            "warn",
            format!(
                "{} trap(s) and {} limit trip(s) in the last hour.",
                failures.trap_last_hour, failures.limit_last_hour
            ),
        )
    } else if not_loaded > 0 {
        (
            "warn",
            format!("{not_loaded} enabled plugin(s) are not loaded."),
        )
    } else {
        ("ok", format!("{loaded} plugin(s) loaded."))
    };
    component(status, reason, metrics, json!({}))
}

// ── assembly, cache, history ─────────────────────────────────────────────────

fn server_key(state: &AppState) -> usize {
    Arc::as_ptr(&state.request_history) as usize
}

#[derive(Default)]
struct Slot {
    cached: Option<(Instant, Value)>,
    history: VecDeque<Value>,
    last_sample: Option<Instant>,
}

fn slots() -> &'static Mutex<HashMap<usize, Slot>> {
    static SLOTS: OnceLock<Mutex<HashMap<usize, Slot>>> = OnceLock::new();
    SLOTS.get_or_init(Default::default)
}

fn compute_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(Default::default)
}

fn sample(components: &Value, t: i64) -> Value {
    let m = |c: &str, k: &str| components[c]["metrics"][k].clone();
    json!({"t":t,"dbPingMs":m("database","pingMs"),"renderPingMs":m("render","pingMs"),
        "syncErrorsLastHour":m("sync","errorsLastHour"),"outboxQueued":m("outbox","queued"),
        "outboxDeferred":m("outbox","deferred"),"searchPending":m("search","pending")})
}

async fn compute(state: &AppState, reg: Option<&PluginRegistry>) -> Value {
    let (database, migrations, sync, outbox, search, render, plugins) = tokio::join!(
        timed("database", probe_database(state)),
        timed("migrations", probe_migrations(state)),
        timed("mail sync", probe_sync(state)),
        timed("outbox", probe_outbox(state)),
        timed("search index", probe_search(state)),
        timed("render worker", probe_render(state)),
        timed("plugin runtime", probe_plugins(state, reg)),
    );
    json!({"database":database,"migrations":migrations,"sync":sync,"outbox":outbox,
        "search":search,"render":render,"plugins":plugins})
}

/// The Health panel's data: every component, cached for [`CACHE_TTL`].
pub(crate) async fn health(state: &AppState, reg: Option<&PluginRegistry>, fresh: bool) -> Value {
    let key = server_key(state);
    let cached = |slots: &HashMap<usize, Slot>| {
        slots.get(&key).and_then(|s| {
            s.cached
                .as_ref()
                .filter(|(at, _)| at.elapsed() < CACHE_TTL)
                .map(|(at, v)| {
                    (
                        at.elapsed(),
                        v.clone(),
                        s.history.iter().cloned().collect::<Vec<_>>(),
                    )
                })
        })
    };
    let hit = if fresh {
        None
    } else {
        cached(&slots().lock().unwrap_or_else(|e| e.into_inner()))
    };
    let (age, components, history) = match hit {
        Some(hit) => hit,
        None => {
            let _single = compute_lock().lock().await;
            // Another request may have filled it while this one waited.
            let again = if fresh {
                None
            } else {
                cached(&slots().lock().unwrap_or_else(|e| e.into_inner()))
            };
            match again {
                Some(hit) => hit,
                None => {
                    let components = compute(state, reg).await;
                    export_metrics(&components);
                    let mut all = slots().lock().unwrap_or_else(|e| e.into_inner());
                    if all.len() > 64 && !all.contains_key(&key) {
                        all.clear();
                    }
                    let slot = all.entry(key).or_default();
                    slot.cached = Some((Instant::now(), components.clone()));
                    if slot.last_sample.is_none_or(|t| t.elapsed() >= HISTORY_STEP) {
                        slot.history.push_back(sample(&components, unix_now()));
                        while slot.history.len() > HISTORY_LEN {
                            slot.history.pop_front();
                        }
                        slot.last_sample = Some(Instant::now());
                    }
                    (
                        Duration::ZERO,
                        components,
                        slot.history.iter().cloned().collect(),
                    )
                }
            }
        }
    };
    let db = &components["database"]["metrics"];
    let mig = &components["migrations"]["metrics"];
    json!({
        // Kept for older consoles.
        "dbPingMs": db["pingMs"],
        "migration": {"applied": mig["applied"], "expected": mig["expected"]},
        "checkedAt": unix_now() - age.as_secs() as i64,
        "ageSecs": age.as_secs_f64(),
        "ttlSecs": CACHE_TTL.as_secs(),
        "components": components,
        "history": history,
        "historyStepSecs": HISTORY_STEP.as_secs(),
    })
}

// ── /metrics ────────────────────────────────────────────────────────────────

fn status_code(status: &str) -> f64 {
    match status {
        "ok" => 0.0,
        "warn" => 1.0,
        "fail" => 2.0,
        _ => 3.0,
    }
}

fn snake(key: &str) -> String {
    let mut out = String::new();
    for c in key.chars() {
        if c.is_ascii_uppercase() {
            out.push('_');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Publish every numeric metric as `mw_health_<component>_<metric>` and each
/// status as `mw_health_status{component}` (0 ok, 1 warn, 2 fail, 3 disabled).
/// Nested counts (`byKind`, `sendsLastHour`) become a `key` label.
fn export_metrics(components: &Value) {
    let Some(all) = components.as_object() else {
        return;
    };
    // A gauge set before the recorder exists is dropped; installing is idempotent.
    crate::observability::install_prometheus_recorder();
    for (name, value) in all {
        let status = value["status"].as_str().unwrap_or("fail");
        metrics::gauge!("mw_health_status", "component" => name.clone()).set(status_code(status));
        let Some(fields) = value["metrics"].as_object() else {
            continue;
        };
        for (key, v) in fields {
            let metric = format!("mw_health_{}_{}", snake(name), snake(key));
            match v {
                Value::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        metrics::gauge!(metric).set(f);
                    }
                }
                Value::Bool(b) => metrics::gauge!(metric).set(if *b { 1.0 } else { 0.0 }),
                Value::Object(inner) => {
                    for (label, iv) in inner {
                        if let Some(f) = iv.as_f64() {
                            metrics::gauge!(metric.clone(), "key" => label.clone()).set(f);
                        } else if let Some(obj) = iv.as_object() {
                            for (sub, sv) in obj {
                                if let Some(f) = sv.as_f64() {
                                    metrics::gauge!(format!("{metric}_{}", snake(sub)), "key" => label.clone()).set(f);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Keep `/metrics` current without anyone opening the dashboard: probe once a
/// minute (through the same cache and single flight).
pub(crate) fn spawn_metrics_refresher(state: AppState, reg: PluginRegistry) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(15)).await;
        let mut tick = tokio::time::interval(METRICS_INTERVAL);
        loop {
            tick.tick().await;
            let _ = health(&state, Some(&reg), false).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_pick_the_middle_and_the_top() {
        let v = [1.0, 2.0, 3.0, 10.0];
        assert_eq!(percentile(&v, 0.5), Some(3.0));
        assert_eq!(percentile(&v, 1.0), Some(10.0));
        assert_eq!(percentile(&[], 0.5), None);
    }

    #[test]
    fn metric_names_are_snake_case() {
        assert_eq!(snake("lastSyncAgeP50Secs"), "last_sync_age_p50_secs");
        assert_eq!(snake("sync"), "sync");
    }

    #[tokio::test]
    async fn a_probe_that_hangs_is_reported_as_failed_within_the_budget() {
        let start = Instant::now();
        let v = timed("stuck", std::future::pending::<Value>()).await;
        assert!(start.elapsed() < PROBE_TIMEOUT + Duration::from_millis(500));
        assert_eq!(v["status"], "fail");
        assert!(v["reason"].as_str().unwrap().contains("did not answer"));
        assert!(v["probeUs"].as_u64().unwrap() >= PROBE_TIMEOUT.as_micros() as u64);
    }

    #[tokio::test]
    async fn render_jobs_are_counted_by_outcome() {
        let before = RENDER.failed_total.load(Ordering::Relaxed);
        let _ = track_render(async { Err::<(), _>(anyhow::anyhow!("boom")) }).await;
        assert_eq!(RENDER.failed_total.load(Ordering::Relaxed), before + 1);
        let t = RENDER.timeout_total.load(Ordering::Relaxed);
        let r = track_render_timeout(
            Duration::from_millis(10),
            std::future::pending::<anyhow::Result<()>>(),
        )
        .await;
        assert!(r.is_err());
        assert_eq!(RENDER.timeout_total.load(Ordering::Relaxed), t + 1);
    }
}
