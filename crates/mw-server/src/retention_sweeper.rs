//! Retention sweeper: one periodic job that deletes rows older than the
//! configured limits from every store that holds personal or behavioural data and
//! has no retention of its own, reduces old IP addresses, and reports what it did.
//!
//! * The limits are the global `retention.*` settings. A limit of 0 keeps rows
//!   (the expired-credentials grace of 0 removes them on expiry).
//! * Deletes are batched (`retention.sweeper.batchRows` rows, at most
//!   `retention.sweeper.maxBatches` statements per store per run), idempotent, and
//!   run through the same SQL on SQLite and Postgres.
//! * Rows still needed are never touched: unsent, retrying, failed and uncertain
//!   submissions, running jobs, and rows of accounts under a legal hold
//!   (`dsr.legalHold.subjects`).
//! * Stores that already purge themselves (the audit log, authentication history,
//!   request metrics, protection history, ...) stay with their own code and
//!   settings; they are listed here so the console shows one picture.
//! * The scheduler runs the job about once a day, with a random spread so several
//!   servers do not sweep together. "Run now" and the dry run use the same code.
//!
//! Routes: `GET /admin/retention` (snapshot) and `POST /admin/retention/run`
//! (`{"dryRun": true}` counts, otherwise starts a run).

use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::AdminRegistrar,
    capabilities::Capability,
    jobs::{JobCtx, JobOutput, Schedule},
    observability::log_files,
    settings_registry,
};
use axum::{
    Json,
    extract::State,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
};
use mw_store::{
    Store,
    jobs::retention::{self as store_retention, IpMode},
};
use serde::Deserialize;
use serde_json::{Value, json};

/// The job kind.
pub(crate) const JOB_KIND: &str = "retention_sweep";
const STATUS_KEY: &str = "retention.status";
const NEXT_KEY: &str = "retention.next";
const SALT_KEY: &str = "retention.ipSalt";
const DAY: i64 = 86_400;

/// Offered when an administrator schedules the kind by hand; the built-in loop
/// below already runs it daily.
pub(crate) fn default_schedule() -> Schedule {
    Schedule::Daily {
        daily_at: "03:40".into(),
        tz: "UTC".into(),
    }
}

/// Who removes a store's rows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Owner {
    /// This sweeper, with the entry's setting.
    Sweeper,
    /// Code that existed before the sweeper; the entry only reports it.
    Existing,
}

/// How an entry's setting value is read.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Unit {
    /// Days of age; 0 keeps the rows.
    Age,
    /// Days after expiry; 0 removes at expiry.
    Grace,
    /// Days of age before the stored IP is reduced; 0 keeps the address.
    Ip,
}

/// One store in the console.
pub(crate) struct Entry {
    pub id: &'static str,
    pub owner: Owner,
    pub unit: Unit,
    /// The setting holding its limit.
    pub key: Option<&'static str>,
    /// A limit fixed in code, when there is no setting.
    pub fixed_days: Option<i64>,
    /// The sweeper target (Sweeper entries without an `ip_table`).
    pub target: Option<&'static str>,
    /// The table whose size the console shows.
    pub table: Option<&'static str>,
    /// Table whose `ip` column the entry reduces.
    pub ip_table: Option<&'static str>,
}

const fn sweep(id: &'static str, key: &'static str, unit: Unit, table: &'static str) -> Entry {
    Entry {
        id,
        owner: Owner::Sweeper,
        unit,
        key: Some(key),
        fixed_days: None,
        target: Some(id),
        table: Some(table),
        ip_table: None,
    }
}

const fn existing(
    id: &'static str,
    key: Option<&'static str>,
    fixed_days: Option<i64>,
    table: Option<&'static str>,
) -> Entry {
    Entry {
        id,
        owner: Owner::Existing,
        unit: Unit::Age,
        key,
        fixed_days,
        target: None,
        table,
        ip_table: None,
    }
}

const GRACE: &str = "retention.expiredCredentials.graceDays";
const SESSION: &str = "retention.sessions.idleDays";

/// Every store the console lists. `docs/admin/retention.md` has the same rows.
pub(crate) const ENTRIES: &[Entry] = &[
    sweep("passwordChangeAudit", "retention.passwordChangeAudit.days", Unit::Age, "password_change_audit"),
    sweep("ssoLoginAudit", "retention.ssoLoginAudit.days", Unit::Age, "sso_login_audit"),
    sweep("dlpAudit", "retention.dlpAudit.days", Unit::Age, "dlp_audit"),
    sweep("assistAudit", "retention.assistAudit.days", Unit::Age, "assist_audit"),
    sweep("alertEvents", "retention.alertEvents.days", Unit::Age, "alert_event"),
    sweep("jobRuns", "retention.jobRuns.days", Unit::Age, "job_run"),
    sweep("outbox", "retention.outbox.days", Unit::Age, "submissions"),
    sweep("sessions", SESSION, Unit::Age, "sessions"),
    sweep("nativeSessions", SESSION, Unit::Age, "native_sessions"),
    sweep("rememberedDevices", GRACE, Unit::Grace, "remembered_device"),
    sweep("signinChallenges", GRACE, Unit::Grace, "signin_challenge"),
    sweep("oidcTokens", GRACE, Unit::Grace, "oidc_token"),
    sweep("oidcCodes", GRACE, Unit::Grace, "oidc_code"),
    sweep("oauthTokens", GRACE, Unit::Grace, "oauth_tokens"),
    sweep("webhookNonces", GRACE, Unit::Grace, "webhook_nonce"),
    sweep("sourceFailures", GRACE, Unit::Grace, "auth_source_failure"),
    Entry {
        id: "imageHandles",
        owner: Owner::Sweeper,
        unit: Unit::Grace,
        key: Some("retention.imageHandles.graceDays"),
        fixed_days: None,
        target: Some("imageHandles"),
        table: None,
        ip_table: None,
    },
    Entry {
        id: "authHistoryIp",
        owner: Owner::Sweeper,
        unit: Unit::Ip,
        key: Some("retention.ipAddresses.days"),
        fixed_days: None,
        target: None,
        table: Some("signin_event"),
        ip_table: Some("signin_event"),
    },
    Entry {
        id: "auditLogIp",
        owner: Owner::Sweeper,
        unit: Unit::Ip,
        key: Some("retention.ipAddresses.days"),
        fixed_days: None,
        target: None,
        table: Some("audit_log"),
        ip_table: Some("audit_log"),
    },
    existing("auditLog", None, None, Some("audit_log")),
    existing("authHistory", Some("logs.auth.retentionDays"), None, Some("signin_event")),
    existing("requestMetrics", Some("metrics.rollupDays"), None, Some("request_metrics_bucket")),
    existing("clientTiming", Some("metrics.rollupDays"), None, Some("client_timing_bucket")),
    existing("metrics", Some("retention.metrics.dayBucketsDays"), None, Some("metrics_bucket")),
    existing("clientErrors", Some("retention.clientErrors.eventDays"), None, Some("client_error_event")),
    existing("protectionEvents", None, Some(30), Some("request_protection_event")),
    existing("protectionSubjects", None, None, Some("request_protection_subject")),
    existing("sessionWatch", Some("security.theft.retentionDays"), None, None),
    existing("operationalIncidents",Some("incidents.retention.days"),None,None),
    existing("complianceMail",Some("retention.complianceMail.days"),None,None),
    existing("dsrRequests", Some("dsr.retention.completedDays"), None, Some("dsr_request")),
    existing("openTracking", Some("privacy.openTracking.retentionDays"), None, Some("open_events")),
    existing("diagnostics", Some("diagnostics.retentionDays"), None, Some("diag_run")),
    existing("securityNotices", Some("security.alerts.noticeRetentionDays"), None, None),
    existing("usageCounters", Some("usage.retentionDays"), None, Some("usage_counter")),
    existing("accountGeo", Some("security.signin.accountGeoHistoryDays"), None, None),
    existing("uploads", None, Some(1), Some("uploaded_blobs")),
    existing("logFiles", Some("logging.file.maxAgeDays"), None, None),
];

/// The age in days a setting value stands for, or `None` to keep the rows.
pub(crate) fn threshold_days(unit: Unit, value: i64) -> Option<i64> {
    match unit {
        Unit::Grace => Some(value.max(0)),
        Unit::Age | Unit::Ip => (value > 0).then_some(value),
    }
}

// ─── settings ──────────────────────────────────────────────────────────────────

pub(crate) async fn num(store: &Store, key: &str) -> i64 {
    let default = settings_registry::lookup(key)
        .and_then(|d| d.default_value().as_i64())
        .unwrap_or(0);
    match settings_registry::effective_for(store, key, &Default::default()).await {
        Ok(Some(v)) => v.value.as_i64().unwrap_or(default),
        _ => default,
    }
}

async fn value(store: &Store, key: &str) -> Value {
    match settings_registry::effective_for(store, key, &Default::default()).await {
        Ok(Some(v)) => v.value,
        _ => settings_registry::lookup(key)
            .map(|d| d.default_value())
            .unwrap_or(Value::Null),
    }
}

async fn text(store: &Store, key: &str) -> String {
    value(store, key).await.as_str().unwrap_or_default().to_string()
}

/// A text setting, for the archive and class code that shares this reader.
pub(crate) async fn text_value(store: &Store, key: &str) -> String {
    text(store, key).await
}

/// A boolean setting, for the archive and class code that shares this reader.
pub(crate) async fn num_bool(store: &Store, key: &str) -> bool {
    flag(store, key).await
}

async fn flag(store: &Store, key: &str) -> bool {
    value(store, key).await.as_bool().unwrap_or(false)
}

// ─── the sweep ─────────────────────────────────────────────────────────────────

/// What one store's sweep did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoreReport {
    pub id: &'static str,
    pub rows: u64,
    pub more: bool,
    pub skipped: Option<&'static str>,
    pub error: Option<String>,
    /// The rows went to the archive tier instead of being deleted.
    pub archived: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Report {
    pub dry_run: bool,
    pub stores: Vec<StoreReport>,
    pub held_accounts: usize,
    pub vacuumed: bool,
}

impl Report {
    pub fn total(&self) -> u64 {
        self.stores.iter().map(|s| s.rows).sum()
    }
    pub fn json(&self) -> Value {
        json!({
            "dryRun": self.dry_run,
            "total": self.total(),
            "heldAccounts": self.held_accounts,
            "vacuumed": self.vacuumed,
            "stores": self.stores.iter().map(|s| json!({
                "id": s.id, "rows": s.rows, "more": s.more,
                "skipped": s.skipped, "error": s.error, "archived": s.archived,
            })).collect::<Vec<_>>(),
        })
    }
}

async fn ip_salt(store: &Store) -> String {
    if let Ok(Some(s)) = store.get_setting(SALT_KEY).await {
        return s;
    }
    let s = uuid::Uuid::new_v4().to_string();
    let _ = store.set_setting(SALT_KEY, &s).await;
    store
        .get_setting(SALT_KEY)
        .await
        .ok()
        .flatten()
        .unwrap_or(s)
}

/// Sweep every Sweeper entry as of `now`. With `dry_run` nothing is deleted and
/// each store reports the rows it would remove.
pub(crate) async fn run_store(store: &Store, now: i64, dry_run: bool) -> Report {
    let batch = num(store, "retention.sweeper.batchRows").await.max(1);
    let max_batches = num(store, "retention.sweeper.maxBatches").await.clamp(1, 1000) as u32;
    let subjects: Vec<String> = value(store, "dsr.legalHold.subjects")
        .await
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    let held = if subjects.is_empty() {
        Vec::new()
    } else {
        store.retention_held_accounts(&subjects).await.unwrap_or_default()
    };
    let mode = match text(store, "retention.ipAddresses.mode").await.as_str() {
        "hash" => IpMode::Hash,
        _ => IpMode::Truncate,
    };
    let (classes, enforce) = crate::retention_tiers::class_days(store).await;
    let archive = crate::retention_tiers::archive_cfg(store).await;
    let mut report = Report {
        dry_run,
        held_accounts: held.len(),
        ..Report::default()
    };
    for e in ENTRIES.iter().filter(|e| e.owner == Owner::Sweeper) {
        let Some(key) = e.key else { continue };
        let own = num(store, key).await;
        let class = crate::retention_tiers::class_of(e.id);
        let days = threshold_days(e.unit, crate::retention_tiers::clamp(e.unit, class, own, &classes, enforce));
        let mut r = StoreReport { id: e.id, rows: 0, more: false, skipped: None, error: None, archived: false };
        let Some(days) = days else {
            r.skipped = Some("keep");
            report.stores.push(r);
            continue;
        };
        let cutoff = now - days * DAY;
        let result = if let Some(table) = e.ip_table {
            let salt = ip_salt(store).await;
            store
                .retention_reduce_ips(table, cutoff, mode, &salt, batch, max_batches, dry_run)
                .await
                .map(|x| (x.rows, x.more))
        } else if let (true, Some(a)) = (archive.enabled, mw_store::retention_archive::archive_table(e.id)) {
            r.archived = true;
            store
                .archive_move(
                    a,
                    cutoff,
                    archive.batch_rows,
                    archive.max_segments,
                    &archive.dir,
                    archive.key_gen,
                    now,
                    &held,
                    dry_run,
                )
                .await
                .map(|x| (x.rows, x.more))
        } else if let Some(t) = e.target.and_then(store_retention::target) {
            store
                .retention_sweep(t, cutoff, batch, max_batches, dry_run, &held)
                .await
                .map(|x| (x.rows, x.more))
        } else {
            continue;
        };
        match result {
            Ok((rows, more)) => {
                r.rows = rows;
                r.more = more;
            }
            Err(err) => r.error = Some(err.to_string()),
        }
        report.stores.push(r);
    }
    // Persisted statistics fold minutes into hours and hours into days, then prune
    // days; a dry run reports nothing because the fold has no row count to preview.
    let mut r = StoreReport { id: "metrics", rows: 0, more: false, skipped: None, error: None, archived: false };
    if dry_run {
        r.skipped = Some("dryRun");
    } else {
        match crate::metrics_persist::rollup(store, now).await {
            Ok(rows) => r.rows = rows,
            Err(err) => r.error = Some(err.to_string()),
        }
    }
    report.stores.push(r);
    // Browser error reports: events past eventDays, groups past groupDays.
    let mut r = StoreReport { id: "clientErrors", rows: 0, more: false, skipped: None, error: None, archived: false };
    match crate::client_errors::prune(store, now, dry_run).await {
        Ok(rows) => r.rows = rows,
        Err(err) => r.error = Some(err.to_string()),
    }
    if dry_run {
        r.skipped = Some("dryRun");
    }
    report.stores.push(r);
    report
}

/// Run, record the status, count metrics, compact when a lot was removed and
/// leave an audit entry.
pub(crate) async fn run(state: &AppState, dry_run: bool, by: &str) -> Report {
    let started = chrono::Utc::now().timestamp();
    let mut report = run_store(&state.store, started, dry_run).await;
    if dry_run {
        return report;
    }
    crate::scrub_admin::housekeeping(state).await;
    let total = report.total();
    let threshold = num(&state.store, "retention.sweeper.vacuumAfterRows").await;
    if threshold > 0 && total >= threshold as u64 {
        report.vacuumed = state.store.vacuum_database().await.is_ok();
    }
    for s in &report.stores {
        metrics::counter!("mw_retention_rows_deleted_total", "store" => s.id).increment(s.rows);
        if s.error.is_some() {
            metrics::counter!("mw_retention_errors_total", "store" => s.id).increment(1);
        }
    }
    let finished = chrono::Utc::now().timestamp();
    metrics::gauge!("mw_retention_last_run_timestamp_seconds").set(finished as f64);
    record_status(&state.store, &report, started, finished).await;
    if total > 0 {
        audit(state, by, &report).await;
    }
    report
}

async fn record_status(store: &Store, report: &Report, started: i64, finished: i64) {
    let mut status: Value = store
        .get_setting(STATUS_KEY)
        .await
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({"stores": {}}));
    let stores = status["stores"].as_object().cloned().unwrap_or_default();
    let mut stores = stores;
    for s in &report.stores {
        let prev = stores
            .get(s.id)
            .and_then(|v| v["purged"].as_u64())
            .unwrap_or(0);
        stores.insert(
            s.id.to_string(),
            json!({
                "at": finished,
                "rows": s.rows,
                "purged": prev + s.rows,
                "more": s.more,
                "skipped": s.skipped,
                "error": s.error,
            }),
        );
    }
    status = json!({
        "startedAt": started,
        "finishedAt": finished,
        "total": report.total(),
        "vacuumed": report.vacuumed,
        "heldAccounts": report.held_accounts,
        "stores": stores,
    });
    let _ = store.set_setting(STATUS_KEY, &status.to_string()).await;
}

async fn audit(state: &AppState, by: &str, report: &Report) {
    let (actor, kind) = if by == "scheduler" {
        ("system".to_string(), mw_admin::ActorKind::System)
    } else {
        (by.to_string(), mw_admin::ActorKind::Admin)
    };
    let entry = mw_admin::AuditLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor,
        actor_kind: kind,
        action: "retention-swept".into(),
        target: Some("retention".into()),
        detail_json: mw_admin::redact_detail(&report.json()),
        ip: None,
    };
    if let Err(e) = state.v6.admin.audit(entry).await {
        tracing::warn!("retention audit entry not recorded: {e}");
    }
}

/// The `retention_sweep` job.
pub(crate) async fn sweep_job(ctx: JobCtx) -> Result<JobOutput, String> {
    let state = ctx.state()?;
    let report = run(state, false, &ctx.triggered_by).await;
    let failed = report.stores.iter().filter(|s| s.error.is_some()).count();
    if failed > 0 && report.total() == 0 {
        return Err(format!("{failed} store(s) failed"));
    }
    Ok(JobOutput::new(
        format!("{} row(s) removed or reduced", report.total()),
        report.json(),
    ))
}

// ─── schedule and log policy ───────────────────────────────────────────────────

fn jitter_secs(minutes: i64) -> i64 {
    if minutes <= 0 {
        return 0;
    }
    (uuid::Uuid::new_v4().as_u128() % (minutes as u128 * 60 + 1)) as i64
}

/// Read the `logging.*` settings into the live log policy.
pub(crate) async fn refresh_logging(store: &Store) {
    let defaults = log_files::Policy::default();
    let dir = text(store, "logging.file.directory").await;
    let valid_dir = log_files::directory_is_valid(&dir);
    let components: Vec<String> = value(store, "logging.componentLevels")
        .await
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    let mb = |n: i64| (n.max(1) as u64) << 20;
    let policy = log_files::Policy {
        file_enabled: flag(store, "logging.file.enabled").await && valid_dir,
        directory: if valid_dir { dir.into() } else { defaults.directory },
        max_size_bytes: mb(num(store, "logging.file.maxSizeMb").await),
        rotate_secs: num(store, "logging.file.rotateHours").await * 3600,
        max_age_days: num(store, "logging.file.maxAgeDays").await.max(1),
        max_files: num(store, "logging.file.maxFiles").await.max(1) as usize,
        max_total_bytes: mb(num(store, "logging.file.maxTotalMb").await),
        compress: flag(store, "logging.file.compress").await,
        level: text(store, "logging.level").await,
        components,
        addresses: log_files::Redaction::parse(&text(store, "logging.redaction.addresses").await),
        ips: log_files::Redaction::parse(&text(store, "logging.redaction.ips").await),
        tokens: flag(store, "logging.redaction.tokens").await,
    };
    if let Err(e) = log_files::apply_levels(&policy.level, &policy.components) {
        tracing::warn!("log levels not applied: {e}");
    }
    if log_files::policy() != policy {
        log_files::set_policy(policy);
    }
}

async fn due(state: &AppState, now: i64) -> bool {
    if !flag(&state.store, "retention.sweeper.enabled").await {
        return false;
    }
    let jitter = num(&state.store, "retention.sweeper.jitterMinutes").await;
    let next = state
        .store
        .get_setting(NEXT_KEY)
        .await
        .ok()
        .flatten()
        .and_then(|s| s.parse::<i64>().ok());
    match next {
        None => {
            // First start: the first run comes after the spread, not at once.
            let _ = state
                .store
                .set_setting(NEXT_KEY, &(now + jitter_secs(jitter)).to_string())
                .await;
            false
        }
        Some(t) if now >= t => {
            let _ = state
                .store
                .set_setting(NEXT_KEY, &(now + DAY + jitter_secs(jitter)).to_string())
                .await;
            true
        }
        Some(_) => false,
    }
}

/// Start the loop: refresh the log policy every minute, run the sweep when due.
pub(crate) fn start(state: AppState) {
    tokio::spawn(async move {
        loop {
            refresh_logging(&state.store).await;
            let now = chrono::Utc::now().timestamp();
            if due(&state, now).await
                && let Err(e) = crate::jobs::run_now(&state, JOB_KIND, json!({}), "scheduler").await
            {
                tracing::debug!("retention sweep not started: {e:?}");
            }
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });
}

// ─── routes ────────────────────────────────────────────────────────────────────

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/retention", Capability::SettingsRead, snapshot_route);
    r.admin_endpoint(Method::POST, "/retention/run", Capability::SettingsWrite, run_route);
    crate::retention_tiers::register(r);
    crate::user_retention::register(r);
}

fn no_store(value: Value) -> Response {
    (
        [(axum::http::header::CACHE_CONTROL, "no-store")],
        Json(value),
    )
        .into_response()
}

async fn snapshot(state: &AppState) -> Value {
    let store = &state.store;
    let status: Value = store
        .get_setting(STATUS_KEY)
        .await
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));
    let mut stores = Vec::new();
    for e in ENTRIES {
        let days = match (e.key, e.fixed_days) {
            (Some(k), _) => Some(num(store, k).await),
            (None, d) => d,
        };
        let default_days = e
            .key
            .and_then(settings_registry::lookup)
            .and_then(|d| d.default_value().as_i64());
        let footprint = match (e.target.and_then(store_retention::target), e.table) {
            (Some(t), _) => store.retention_footprint(t).await.ok(),
            (None, Some(table)) => store.retention_table_footprint(table).await.ok().flatten(),
            _ => None,
        };
        let last = &status["stores"][e.id];
        stores.push(json!({
            "id": e.id,
            "owner": if e.owner == Owner::Sweeper { "sweeper" } else { "existing" },
            "unit": match e.unit { Unit::Age => "age", Unit::Grace => "grace", Unit::Ip => "ip" },
            "key": e.key,
            "days": days,
            "defaultDays": default_days,
            "fixed": e.key.is_none() && e.fixed_days.is_some(),
            "rows": footprint.map(|f| f.rows),
            "bytes": footprint.and_then(|f| f.bytes),
            "lastSweepAt": last["at"],
            "lastRows": last["rows"],
            "purged": last["purged"],
            "error": last["error"],
        }));
    }
    let next = store
        .get_setting(NEXT_KEY)
        .await
        .ok()
        .flatten()
        .and_then(|s| s.parse::<i64>().ok());
    let running = store
        .running_job_runs()
        .await
        .map(|r| r.iter().any(|j| j.kind == JOB_KIND))
        .unwrap_or(false);
    json!({
        "sweeper": {
            "enabled": flag(store, "retention.sweeper.enabled").await,
            "nextRunAt": next,
            "lastRunAt": status["finishedAt"],
            "lastTotal": status["total"],
            "vacuumed": status["vacuumed"],
            "running": running,
            "keys": {
                "enabled": "retention.sweeper.enabled",
                "batchRows": "retention.sweeper.batchRows",
                "maxBatches": "retention.sweeper.maxBatches",
                "jitterMinutes": "retention.sweeper.jitterMinutes",
                "vacuumAfterRows": "retention.sweeper.vacuumAfterRows",
            },
        },
        "stores": stores,
        "logs": {
            "status": log_files::status(),
            "keys": [
                "logging.file.enabled", "logging.file.directory", "logging.file.maxSizeMb",
                "logging.file.rotateHours", "logging.file.maxAgeDays", "logging.file.maxFiles",
                "logging.file.maxTotalMb", "logging.file.compress", "logging.level",
                "logging.componentLevels", "logging.redaction.addresses",
                "logging.redaction.ips", "logging.redaction.tokens",
            ],
        },
    })
}

async fn snapshot_route(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsRead)?;
    Ok(no_store(snapshot(&state).await))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RunInput {
    #[serde(default)]
    dry_run: bool,
}

async fn run_route(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(input): AdminJson<RunInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    if input.dry_run {
        let report = run(&state, true, &ctx.username).await;
        return Ok(no_store(report.json()));
    }
    match crate::jobs::run_now(&state, JOB_KIND, json!({}), &ctx.username).await {
        Ok(id) => Ok((StatusCode::ACCEPTED, Json(json!({"runId": id}))).into_response()),
        Err(crate::jobs::RunNowError::AlreadyRunning) => Err(AdminApiError::conflict(
            "retentionAlreadyRunning",
            "a retention sweep is already running",
        )),
        Err(crate::jobs::RunNowError::Store(msg)) => Err(AdminApiError::conflict(
            "retentionStoreUnavailable",
            format!("the job store refused the run: {msg}"),
        )),
        Err(crate::jobs::RunNowError::UnknownKind) => {
            Err(AdminApiError::not_found("retention sweep unavailable"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds_follow_the_unit() {
        assert_eq!(threshold_days(Unit::Age, 0), None);
        assert_eq!(threshold_days(Unit::Age, 30), Some(30));
        assert_eq!(threshold_days(Unit::Grace, 0), Some(0));
        assert_eq!(threshold_days(Unit::Ip, 0), None);
    }

    #[test]
    fn every_entry_names_a_registered_key_and_a_known_target() {
        let mut ids = std::collections::HashSet::new();
        for e in ENTRIES {
            assert!(ids.insert(e.id), "duplicate entry {}", e.id);
            if let Some(k) = e.key {
                let d = settings_registry::lookup(k).unwrap_or_else(|| panic!("{k} registered"));
                if e.owner == Owner::Sweeper {
                    assert!(d.key.starts_with("retention."), "{k}");
                }
            }
            if let Some(t) = e.target {
                assert!(store_retention::target(t).is_some(), "{t}");
            }
            if e.owner == Owner::Sweeper {
                assert!(e.key.is_some(), "{} needs a setting", e.id);
            }
        }
        for t in store_retention::TARGETS {
            assert!(
                ENTRIES.iter().any(|e| e.target == Some(t.id)),
                "target {} has no entry",
                t.id
            );
        }
    }

    #[test]
    fn retention_doc_lists_every_entry_and_key() {
        let doc = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/admin/retention.md"),
        )
        .unwrap();
        for e in ENTRIES {
            assert!(doc.contains(&format!("`{}`", e.id)), "docs/admin/retention.md lacks {}", e.id);
            if let Some(k) = e.key {
                assert!(doc.contains(&format!("`{k}`")), "docs/admin/retention.md lacks {k}");
            }
        }
    }

    #[tokio::test]
    async fn run_store_honours_limits_zero_keeps_and_dry_run_counts() {
        use sqlx::sqlite::SqlitePoolOptions;
        let dir = std::env::temp_dir().join(format!("mw-retention-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("s.sqlite");
        let store = Store::open(path.to_str().unwrap(), mw_store::ServerKey::from_bytes(&[3u8; 32]).unwrap())
            .await
            .unwrap();
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite://{}", path.display()))
            .await
            .unwrap();
        let now = 1_800_000_000i64;
        for (id, fired) in [("old", now - 1100 * DAY), ("new", now - DAY)] {
            sqlx::query("INSERT INTO alert_event(id,rule_id,fired_at,summary,delivered_json) VALUES(?,?,?,?,?)")
                .bind(id).bind("r").bind(fired).bind("s").bind("[]")
                .execute(&pool).await.unwrap();
        }
        let row = |r: &Report, id: &str| r.stores.iter().find(|s| s.id == id).cloned().unwrap();

        let dry = run_store(&store, now, true).await;
        assert_eq!(row(&dry, "alertEvents").rows, 1);
        assert_eq!(row(&dry, "outbox").rows, 0);

        let real = run_store(&store, now, false).await;
        assert_eq!(row(&real, "alertEvents").rows, 1);
        assert!(real.stores.iter().all(|s| s.error.is_none()), "{:?}", real.stores);
        let left: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alert_event").fetch_one(&pool).await.unwrap();
        assert_eq!(left.0, 1);
        assert_eq!(row(&run_store(&store, now, false).await, "alertEvents").rows, 0, "idempotent");

        // 0 keeps every row.
        store
            .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                key: "retention.alertEvents.days".into(),
                global_value: Some("0".into()),
                allow_user_override: false,
                global_locked: false,
                updated_by: "test".into(),
                updated_at: now,
            })
            .await
            .unwrap();
        sqlx::query("INSERT INTO alert_event(id,rule_id,fired_at,summary,delivered_json) VALUES('ancient','r',1,'s','[]')")
            .execute(&pool).await.unwrap();
        let kept = run_store(&store, now, false).await;
        assert_eq!(row(&kept, "alertEvents").skipped, Some("keep"));
        std::fs::remove_dir_all(dir).ok();
    }

    #[tokio::test]
    async fn archive_and_class_ceilings_change_what_the_sweep_does() {
        use sqlx::sqlite::SqlitePoolOptions;
        let dir = std::env::temp_dir().join(format!("mw-retention-arch-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("s.sqlite");
        let store = Store::open(path.to_str().unwrap(), mw_store::ServerKey::from_bytes(&[5u8; 32]).unwrap())
            .await
            .unwrap();
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite://{}", path.display()))
            .await
            .unwrap();
        let set = |key: &'static str, value: String| {
            let store = store.clone();
            async move {
                store
                    .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                        key: key.into(),
                        global_value: Some(value),
                        allow_user_override: false,
                        global_locked: false,
                        updated_by: "test".into(),
                        updated_at: 1,
                    })
                    .await
                    .unwrap();
            }
        };
        let now = 1_800_000_000i64;
        sqlx::query("INSERT INTO assist_audit(id,ts,actor,capability,scope_summary,endpoint_host) VALUES('old','2010-01-01T00:00:00Z','a','c','s','h')")
            .execute(&pool).await.unwrap();
        let add_event = |id: &'static str, fired: i64| {
            let pool = pool.clone();
            async move {
                sqlx::query("INSERT INTO alert_event(id,rule_id,fired_at,summary,delivered_json) VALUES(?,?,?,?,?)")
                    .bind(id).bind("r").bind(fired).bind("s").bind("[]")
                    .execute(&pool).await.unwrap();
            }
        };
        add_event("fresh", now - DAY).await;
        let row = |r: &Report, id: &str| r.stores.iter().find(|s| s.id == id).cloned().unwrap();
        let hot = |table: &'static str| {
            let pool = pool.clone();
            async move {
                sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {table}")).fetch_one(&pool).await.unwrap()
            }
        };

        // Archive off: the old audit row is deleted as before.
        assert_eq!(row(&run_store(&store, now, true).await, "assistAudit").archived, false);

        // Archive on: the same row moves into a sealed file, and a dry run counts without writing.
        let arch = dir.join("arch");
        set("retention.archive.enabled", "true".into()).await;
        set("retention.archive.directory", serde_json::to_string(arch.to_str().unwrap()).unwrap()).await;
        let dry = run_store(&store, now, true).await;
        assert_eq!((row(&dry, "assistAudit").rows, row(&dry, "assistAudit").archived), (1, true));
        assert_eq!(hot("assist_audit").await, 1);
        assert!(!arch.exists(), "a dry run writes no file");
        let real = run_store(&store, now, false).await;
        assert_eq!((row(&real, "assistAudit").rows, row(&real, "assistAudit").error.clone()), (1, None));
        assert_eq!(hot("assist_audit").await, 0);
        let totals = store.archive_totals().await.unwrap();
        assert_eq!((totals.segments, totals.rows), (1, 1));
        assert_eq!(row(&run_store(&store, now, false).await, "assistAudit").rows, 0, "idempotent");
        assert_eq!(store.archive_verify_all(None, &arch, now).await.unwrap().ok, 1);

        // Class ceilings: a store kept for ever (0) is held to its class only when enforced.
        set("retention.alertEvents.days", "0".into()).await;
        assert_eq!(row(&run_store(&store, now, false).await, "alertEvents").skipped, Some("keep"));
        add_event("stale", now - 500 * DAY).await;
        set("retention.class.enforce", "true".into()).await;
        set("retention.class.short.days", "365".into()).await;
        let clamped = run_store(&store, now, false).await;
        assert_eq!(row(&clamped, "alertEvents").rows, 1, "the 500-day-old event goes, the fresh one stays: {:?}", row(&clamped, "alertEvents"));
        assert_eq!(hot("alert_event").await, 1);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn retention_and_logging_ranges_validate() {
        use serde_json::json;
        let ok = |k: &str, v: Value| {
            let d = settings_registry::lookup(k).unwrap_or_else(|| panic!("{k}"));
            settings_registry::validate(d, &v, "/value").is_ok()
        };
        assert!(ok("retention.outbox.days", json!(0)));
        assert!(ok("retention.outbox.days", json!(3650)));
        assert!(!ok("retention.outbox.days", json!(3651)));
        assert!(!ok("retention.outbox.days", json!(-1)));
        assert!(ok("retention.expiredCredentials.graceDays", json!(0)));
        assert!(!ok("retention.sweeper.batchRows", json!(10)));
        assert!(!ok("retention.ipAddresses.mode", json!("erase")));
        assert!(ok("logging.redaction.ips", json!("hashed")));
        assert!(!ok("logging.redaction.ips", json!("partial")));
        assert!(!ok("logging.level", json!("verbose")));
        assert!(!ok("logging.file.maxSizeMb", json!(0)));
        for k in ["retention.sweeper.enabled", "logging.level", "logging.file.directory"] {
            let d = settings_registry::lookup(k).unwrap();
            assert!(!d.scopes.allows(mw_store::settings_policy::SettingScope::Domain), "{k}");
            assert!(!d.scopes.allows(mw_store::settings_policy::SettingScope::User), "{k}");
        }
    }
}
