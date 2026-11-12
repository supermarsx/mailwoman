//! Retention classes and the archive tier of the long class.
//!
//! * **Classes.** Every store the sweeper reports belongs to one class:
//!   short (operational: sessions, tokens, rate limits, jobs, metrics),
//!   medium (sign-in and security history, request protection) or long
//!   (security audit, administrator actions, requests, consent, tombstones).
//!   `retention.class.*.days` are ceilings: with `retention.class.enforce` on
//!   a store's own limit is clamped to its class, and 0 ("keep") counts as the
//!   ceiling. With it off the per-store limits stand. Long must not be below
//!   medium and medium must not be below short; a setting that breaks this is
//!   raised to the class below it and reported as a warning.
//! * **Archive.** With `retention.archive.enabled`, the stores that have an
//!   archive table (see `mw_store::retention_archive`) move rows older than
//!   their own limit into sealed files instead of deleting them. The
//!   `retention_archive` job verifies the files, removes files older than the
//!   long class, and records the outcome. `retention.archive.legalHold` stops
//!   the removal.
//! * Routes under `/admin/retention`: `classes`, `archive`, `archive/verify`,
//!   `archive/search`, `archive/restore`, `archive/export`.

use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::AdminRegistrar,
    auth_logs::sweeper::{self, ENTRIES, Owner, Unit},
    capabilities::Capability,
    jobs::{JobCtx, JobOutput, Schedule},
};
use axum::{
    Json,
    extract::{Query, State},
    http::{Method, header},
    response::{IntoResponse, Response},
};
use mw_store::{
    Store,
    jobs::retention as store_retention,
    retention_archive::{self as archive},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;

pub(crate) const JOB_KIND: &str = "retention_archive";
const STATUS_KEY: &str = "retention.archive.status";
const DAY: i64 = 86_400;
/// Age buckets of the distribution cards, in days.
pub(crate) const EDGES: [i64; 4] = [30, 90, 365, 1095];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Class {
    Short,
    Medium,
    Long,
}

impl Class {
    pub const ALL: [Class; 3] = [Class::Short, Class::Medium, Class::Long];
    pub fn id(self) -> &'static str {
        match self {
            Class::Short => "short",
            Class::Medium => "medium",
            Class::Long => "long",
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Class::Short => "retention.class.short.days",
            Class::Medium => "retention.class.medium.days",
            Class::Long => "retention.class.long.days",
        }
    }
}

/// The class of a store listed by the sweeper.
pub(crate) fn class_of(id: &str) -> Option<Class> {
    Some(match id {
        "passwordChangeAudit" | "ssoLoginAudit" | "dlpAudit" | "assistAudit" | "auditLog" | "dsrRequests" | "operationalIncidents" => Class::Long,
        "authHistory" | "authHistoryIp" | "auditLogIp" | "openTracking" | "securityNotices" | "usageCounters"
        | "accountGeo" | "protectionEvents" | "protectionSubjects" | "sessionWatch" => Class::Medium,
        "complianceMail" | "alertEvents" | "jobRuns" | "outbox" | "sessions" | "nativeSessions" | "rememberedDevices"
        | "signinChallenges" | "oidcTokens" | "oidcCodes" | "oauthTokens" | "webhookNonces" | "sourceFailures"
        | "imageHandles" | "requestMetrics" | "clientTiming" | "metrics" | "clientErrors" | "diagnostics" | "uploads" | "logFiles" => Class::Short,
        _ => return None,
    })
}

/// The class ceilings in days; 0 is "forever" and only the long class can have it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct ClassDays {
    pub short: i64,
    pub medium: i64,
    pub long: i64,
}

impl ClassDays {
    pub fn of(&self, c: Class) -> i64 {
        match c {
            Class::Short => self.short,
            Class::Medium => self.medium,
            Class::Long => self.long,
        }
    }

    /// The values as used: medium not below short, long (unless 0) not below medium.
    pub fn normalized(self) -> Self {
        let short = self.short.max(1);
        let medium = self.medium.max(short);
        let long = if self.long == 0 { 0 } else { self.long.max(medium) };
        Self { short, medium, long }
    }

    pub fn warnings(&self) -> Vec<&'static str> {
        let mut w = Vec::new();
        if self.medium < self.short {
            w.push("mediumBelowShort");
        }
        if self.long != 0 && self.long < self.medium {
            w.push("longBelowMedium");
        }
        if self.long == 0 {
            w.push("longForever");
        }
        w
    }
}

pub(crate) async fn class_days(store: &Store) -> (ClassDays, bool) {
    let raw = ClassDays {
        short: sweeper::num(store, Class::Short.key()).await,
        medium: sweeper::num(store, Class::Medium.key()).await,
        long: sweeper::num(store, Class::Long.key()).await,
    };
    let enforce = sweeper::num_bool(store, "retention.class.enforce").await;
    (raw, enforce)
}

/// A store's own limit after the class ceiling. `value` is the setting (0 keeps).
pub(crate) fn clamp(unit: Unit, class: Option<Class>, value: i64, classes: &ClassDays, enforce: bool) -> i64 {
    let Some(class) = class else { return value };
    if !enforce || unit == Unit::Grace {
        return value;
    }
    let ceiling = classes.normalized().of(class);
    if ceiling == 0 {
        return value;
    }
    if value == 0 { ceiling } else { value.min(ceiling) }
}

// ─── archive configuration ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct ArchiveCfg {
    pub enabled: bool,
    pub dir: PathBuf,
    pub dir_valid: bool,
    pub batch_rows: i64,
    pub max_segments: u32,
    pub key_gen: i64,
    pub verify_days: i64,
    pub legal_hold: bool,
}

pub(crate) async fn archive_cfg(store: &Store) -> ArchiveCfg {
    let dir = sweeper::text_value(store, "retention.archive.directory").await;
    let dir_valid = archive::directory_is_valid(&dir);
    ArchiveCfg {
        enabled: sweeper::num_bool(store, "retention.archive.enabled").await && dir_valid,
        dir: PathBuf::from(if dir_valid { dir } else { "archive".into() }),
        dir_valid,
        batch_rows: sweeper::num(store, "retention.archive.batchRows").await.max(50),
        max_segments: sweeper::num(store, "retention.archive.maxSegments").await.clamp(1, 1000) as u32,
        key_gen: sweeper::num(store, "retention.archive.keyGeneration").await.max(1),
        verify_days: sweeper::num(store, "retention.archive.verifyDays").await.max(1),
        legal_hold: sweeper::num_bool(store, "retention.archive.legalHold").await,
    }
}

// ─── what applies to a person ──────────────────────────────────────────────────

/// Stores that hold a person's data, for the privacy dashboard.
const PERSONAL: &[&str] = &[
    "sessions", "rememberedDevices", "outbox", "authHistory", "authHistoryIp", "passwordChangeAudit", "dlpAudit",
    "accountGeo", "securityNotices", "usageCounters", "openTracking", "dsrRequests", "auditLog",
];

/// The system-held retention that applies to one account: per store, its class,
/// the days after the class ceiling (null keeps), and whether the archive tier
/// holds it afterwards.
pub(crate) async fn effective_system(store: &Store, _account_id: &str, _username: &str) -> Vec<Value> {
    let (raw, enforce) = class_days(store).await;
    let cfg = archive_cfg(store).await;
    let mut out = Vec::new();
    for e in ENTRIES.iter().filter(|e| PERSONAL.contains(&e.id)) {
        let class = class_of(e.id);
        let days = match (e.key, e.fixed_days) {
            (Some(k), _) => Some(clamp(e.unit, class, sweeper::num(store, k).await, &raw, enforce)),
            (None, d) => d,
        };
        let archived = cfg.enabled && archive::archive_table(e.id).is_some();
        let total = if archived && class == Some(Class::Long) {
            Some(raw.normalized().long)
        } else {
            None
        };
        out.push(json!({
            "id": e.id,
            "class": class.map(Class::id),
            "days": days,
            "keptForever": days == Some(0),
            "archived": archived,
            "archiveDays": total,
            "source": if e.owner == Owner::Sweeper { "sweeper" } else { "existing" },
        }));
    }
    out
}

// ─── classes view ──────────────────────────────────────────────────────────────

async fn classes_snapshot(state: &AppState) -> Value {
    let store = &state.store;
    let (raw, enforce) = class_days(store).await;
    let now = chrono::Utc::now().timestamp();
    let mut classes = Vec::new();
    for c in Class::ALL {
        let mut entries = Vec::new();
        let mut totals = vec![0i64; EDGES.len() + 1];
        for e in ENTRIES.iter().filter(|e| class_of(e.id) == Some(c)) {
            let own = match (e.key, e.fixed_days) {
                (Some(k), _) => Some(sweeper::num(store, k).await),
                (None, d) => d,
            };
            let table_age = match e.target.and_then(store_retention::target) {
                Some(t) => Some((t.table, t.age)),
                None => archive::archive_table(e.id).map(|a| (a.table, a.age)),
            };
            let buckets = match table_age {
                Some((table, age)) => store.retention_age_buckets(table, age, now, &EDGES).await.ok(),
                None => None,
            };
            if let Some(b) = &buckets {
                for (t, n) in totals.iter_mut().zip(b) {
                    *t += n;
                }
            }
            let limit = own.map(|d| clamp(e.unit, Some(c), d, &raw, enforce));
            entries.push(json!({
                "id": e.id,
                "key": e.key,
                "ownDays": own,
                "days": limit,
                "keptForever": limit == Some(0),
                "exceedsClass": own.is_some_and(|d| e.unit != Unit::Grace && (d == 0 || (raw.normalized().of(c) != 0 && d > raw.normalized().of(c)))),
                "archivable": archive::archive_table(e.id).is_some(),
                "ageBuckets": buckets,
            }));
        }
        classes.push(json!({
            "id": c.id(),
            "key": c.key(),
            "days": raw.of(c),
            "effectiveDays": raw.normalized().of(c),
            "forever": raw.normalized().of(c) == 0,
            "entries": entries,
            "ageBuckets": totals,
        }));
    }
    json!({
        "enforce": enforce,
        "enforceKey": "retention.class.enforce",
        "edgesDays": EDGES,
        "warnings": raw.warnings(),
        "classes": classes,
    })
}

// ─── archive status and job ────────────────────────────────────────────────────

async fn archive_snapshot(state: &AppState) -> Value {
    let store = &state.store;
    let cfg = archive_cfg(store).await;
    let totals = store.archive_totals().await.unwrap_or_default();
    let segments = store.archive_segments(None).await.unwrap_or_default();
    let status: Value = store
        .get_setting(STATUS_KEY)
        .await
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null);
    let (raw, _) = class_days(store).await;
    let mut per_store: std::collections::BTreeMap<String, (i64, i64, i64)> = Default::default();
    for s in segments.iter().filter(|s| s.state != "purged") {
        let e = per_store.entry(s.store_id.clone()).or_default();
        e.0 += 1;
        e.1 += s.row_count;
        e.2 += s.bytes;
    }
    let recent: Vec<&archive::Segment> = segments.iter().rev().take(100).collect();
    json!({
        "enabled": cfg.enabled,
        "directory": cfg.dir.to_string_lossy(),
        "directoryValid": cfg.dir_valid,
        "keyGeneration": cfg.key_gen,
        "legalHold": cfg.legal_hold,
        "longDays": raw.normalized().long,
        "longForever": raw.normalized().long == 0,
        "totals": totals,
        "stores": per_store.iter().map(|(k, v)| json!({"id": k, "segments": v.0, "rows": v.1, "bytes": v.2})).collect::<Vec<_>>(),
        "segments": recent,
        "status": status,
        "tables": archive::ARCHIVE_TABLES.iter().map(|t| t.id).collect::<Vec<_>>(),
        "keys": ["retention.archive.enabled", "retention.archive.directory", "retention.archive.batchRows",
                 "retention.archive.maxSegments", "retention.archive.keyGeneration", "retention.archive.verifyDays",
                 "retention.archive.legalHold", "retention.class.short.days", "retention.class.medium.days",
                 "retention.class.long.days", "retention.class.enforce"],
    })
}

pub(crate) fn default_schedule() -> Schedule {
    Schedule::Daily { daily_at: "05:10".into(), tz: "UTC".into() }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct ArchiveRun {
    pub verified: Option<archive::VerifyReport>,
    pub purged: u64,
}

/// Verify (when due or `force`) and purge. Pure store work, so tests call it.
pub(crate) async fn maintain(store: &Store, now: i64, force: bool, dry_run: bool) -> Result<ArchiveRun, String> {
    let cfg = archive_cfg(store).await;
    let (raw, _) = class_days(store).await;
    let mut run = ArchiveRun::default();
    let last: i64 = store
        .get_setting(STATUS_KEY)
        .await
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| v["verifiedAt"].as_i64())
        .unwrap_or(0);
    if !dry_run && (force || last + cfg.verify_days * DAY <= now) {
        run.verified = Some(store.archive_verify_all(None, &cfg.dir, now).await.map_err(|e| e.to_string())?);
    }
    let long = raw.normalized().long;
    if long > 0 && !cfg.legal_hold {
        run.purged = store.archive_purge(now - long * DAY, &cfg.dir, dry_run).await.map_err(|e| e.to_string())?;
    }
    Ok(run)
}

pub(crate) async fn archive_job(ctx: JobCtx) -> Result<JobOutput, String> {
    let state = ctx.state()?;
    let now = chrono::Utc::now().timestamp();
    let run = maintain(&state.store, now, false, false).await?;
    let mut status: Value = state
        .store
        .get_setting(STATUS_KEY)
        .await
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));
    status["at"] = json!(now);
    status["purged"] = json!(status["purged"].as_u64().unwrap_or(0) + run.purged);
    if let Some(v) = &run.verified {
        status["verifiedAt"] = json!(now);
        status["checked"] = json!(v.checked);
        status["ok"] = json!(v.ok);
        status["failed"] = json!(v.failed.iter().map(|(id, why)| json!({"segment": id, "verdict": why})).collect::<Vec<_>>());
    }
    let _ = state.store.set_setting(STATUS_KEY, &status.to_string()).await;
    if run.purged > 0 || run.verified.as_ref().is_some_and(|v| !v.failed.is_empty()) {
        audit(state, "system", "retention-archive-maintained", json!({
            "purgedSegments": run.purged,
            "failed": run.verified.as_ref().map(|v| v.failed.len()).unwrap_or(0),
        }))
        .await;
    }
    Ok(JobOutput::new(
        format!(
            "{} segment(s) checked, {} failed, {} purged",
            run.verified.as_ref().map(|v| v.checked).unwrap_or(0),
            run.verified.as_ref().map(|v| v.failed.len()).unwrap_or(0),
            run.purged
        ),
        json!({"status": status}),
    ))
}

async fn audit(state: &AppState, actor: &str, action: &str, detail: Value) {
    let system = actor == "system";
    let entry = mw_admin::AuditLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: actor.to_string(),
        actor_kind: if system { mw_admin::ActorKind::System } else { mw_admin::ActorKind::Admin },
        action: action.into(),
        target: Some("retention".into()),
        detail_json: mw_admin::redact_detail(&detail),
        ip: None,
    };
    if let Err(e) = state.v6.admin.audit(entry).await {
        tracing::warn!("retention audit entry not recorded: {e}");
    }
}

// ─── routes ────────────────────────────────────────────────────────────────────

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/retention/classes", Capability::SettingsRead, classes_route);
    r.admin_endpoint(Method::GET, "/retention/archive", Capability::SettingsRead, archive_route);
    r.admin_endpoint(Method::POST, "/retention/archive/verify", Capability::SettingsWrite, verify_route);
    r.admin_endpoint(Method::GET, "/retention/archive/search", Capability::SettingsRead, search_route);
    r.admin_endpoint(Method::POST, "/retention/archive/restore", Capability::SettingsWrite, restore_route);
    r.admin_endpoint(Method::GET, "/retention/archive/export", Capability::SettingsRead, export_route);
}

fn no_store(v: impl IntoResponse) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], v).into_response()
}

fn store_err(e: impl std::fmt::Display) -> AdminApiError {
    AdminApiError::conflict("retentionStoreUnavailable", e.to_string())
}

async fn classes_route(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsRead)?;
    Ok(no_store(Json(classes_snapshot(&state).await)))
}

async fn archive_route(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsRead)?;
    Ok(no_store(Json(archive_snapshot(&state).await)))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VerifyIn {}

async fn verify_route(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(_): AdminJson<VerifyIn>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    let now = chrono::Utc::now().timestamp();
    let run = maintain(&state.store, now, true, false).await.map_err(store_err)?;
    let v = run.verified.unwrap_or_default();
    let mut status: Value = state
        .store
        .get_setting(STATUS_KEY)
        .await
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));
    status["verifiedAt"] = json!(now);
    status["checked"] = json!(v.checked);
    status["ok"] = json!(v.ok);
    status["failed"] = json!(v.failed.iter().map(|(id, why)| json!({"segment": id, "verdict": why})).collect::<Vec<_>>());
    status["purged"] = json!(status["purged"].as_u64().unwrap_or(0) + run.purged);
    let _ = state.store.set_setting(STATUS_KEY, &status.to_string()).await;
    audit(&state, &ctx.username, "retention-archive-verified", json!({"checked": v.checked, "failed": v.failed.len(), "purgedSegments": run.purged})).await;
    Ok(no_store(Json(json!({"checked": v.checked, "ok": v.ok, "failed": v.failed, "purged": run.purged}))))
}

#[derive(Deserialize)]
struct SearchQ {
    store: String,
    key: Option<String>,
    account: Option<String>,
}

async fn search_route(State(state): State<AppState>, ctx: AdminCtx, Query(q): Query<SearchQ>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsRead)?;
    if archive::archive_table(&q.store).is_none() {
        return Err(AdminApiError::not_found("unknown archive store"));
    }
    let cfg = archive_cfg(&state.store).await;
    let hits = state
        .store
        .archive_search(&q.store, q.key.as_deref().filter(|k| !k.is_empty()), q.account.as_deref().filter(|k| !k.is_empty()), 100, &cfg.dir)
        .await
        .map_err(store_err)?;
    audit(&state, &ctx.username, "retention-archive-searched", json!({"store": q.store, "hits": hits.len()})).await;
    Ok(no_store(Json(json!({"hits": hits}))))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RestoreIn {
    segment_id: String,
}

async fn restore_route(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(input): AdminJson<RestoreIn>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    let cfg = archive_cfg(&state.store).await;
    let n = state.store.archive_restore(&input.segment_id, &cfg.dir).await.map_err(store_err)?;
    audit(&state, &ctx.username, "retention-archive-restored", json!({"segment": input.segment_id, "rows": n})).await;
    Ok(no_store(Json(json!({"restored": n}))))
}

#[derive(Deserialize)]
struct ExportQ {
    segment: String,
}

async fn export_route(State(state): State<AppState>, ctx: AdminCtx, Query(q): Query<ExportQ>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsRead)?;
    let cfg = archive_cfg(&state.store).await;
    let text = state.store.archive_export(&q.segment, &cfg.dir).await.map_err(store_err)?;
    audit(&state, &ctx.username, "retention-archive-exported", json!({"segment": q.segment})).await;
    Ok((
        [(header::CONTENT_TYPE, "application/x-ndjson"), (header::CACHE_CONTROL, "no-store")],
        text,
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_registry;

    #[test]
    fn every_listed_store_has_a_class() {
        for e in ENTRIES {
            assert!(class_of(e.id).is_some(), "{} needs a retention class", e.id);
        }
    }

    #[test]
    fn archive_tables_are_long_class_sweeper_stores() {
        for t in archive::ARCHIVE_TABLES {
            assert_eq!(class_of(t.id), Some(Class::Long), "{}", t.id);
            let e = ENTRIES.iter().find(|e| e.id == t.id).expect("listed");
            assert!(e.owner == Owner::Sweeper, "{}", t.id);
            // The archive reads the same table and age column the sweeper deletes by.
            let target = store_retention::target(t.id).expect("target");
            assert_eq!(target.table, t.table);
            assert_eq!(target.pk, t.pk);
        }
    }

    #[test]
    fn class_ceilings_are_cross_checked_and_clamp_limits() {
        let c = ClassDays { short: 365, medium: 1095, long: 2555 };
        assert!(c.warnings().is_empty());
        let bad = ClassDays { short: 400, medium: 100, long: 50 };
        assert_eq!(bad.warnings(), vec!["mediumBelowShort", "longBelowMedium"]);
        assert_eq!(bad.normalized(), ClassDays { short: 400, medium: 400, long: 400 });
        let forever = ClassDays { short: 365, medium: 1095, long: 0 };
        assert_eq!(forever.warnings(), vec!["longForever"]);
        assert_eq!(forever.normalized().long, 0);

        // Off: unchanged. On: capped, and "keep" becomes the ceiling.
        assert_eq!(clamp(Unit::Age, Some(Class::Short), 0, &c, false), 0);
        assert_eq!(clamp(Unit::Age, Some(Class::Short), 0, &c, true), 365);
        assert_eq!(clamp(Unit::Age, Some(Class::Short), 90, &c, true), 90);
        assert_eq!(clamp(Unit::Age, Some(Class::Short), 900, &c, true), 365);
        assert_eq!(clamp(Unit::Grace, Some(Class::Short), 0, &c, true), 0, "grace 0 removes at expiry");
        assert_eq!(clamp(Unit::Age, Some(Class::Long), 0, &forever, true), 0, "forever long class keeps");
        assert_eq!(clamp(Unit::Age, None, 0, &c, true), 0);
    }

    /// Every key of the user rules, the classes and the archive is registered with
    /// real bounds, documented, labelled in both catalogs and listed in the console.
    #[test]
    fn retention_tier_keys_are_registered_documented_and_labelled() {
        use mw_store::settings_policy::SettingScope;
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
        let docs = read("docs/admin/settings.md");
        let keys_ts = read("apps/web/src/screens/Admin/Settings/keys.ts");
        let ftl = [read("apps/web/locales/en/admin-settings.ftl"), read("apps/web/locales/pt-PT/admin-settings.ftl")];
        let slug = |key: &str| {
            let mut out = String::new();
            let mut prev = ' ';
            for c in key.chars() {
                if c.is_ascii_uppercase() && (prev.is_ascii_lowercase() || prev.is_ascii_digit()) {
                    out.push('-');
                }
                out.push(if c == '.' || c == '_' { '-' } else { c.to_ascii_lowercase() });
                prev = c;
            }
            out
        };
        let mut found = 0;
        for d in settings_registry::REGISTRY {
            if !(d.key.starts_with("retention.user.") || d.key.starts_with("retention.class.") || d.key.starts_with("retention.archive.")) {
                continue;
            }
            found += 1;
            assert!(docs.contains(&format!("<a id=\"{}\"></a>", d.key)), "{} has no docs anchor", d.key);
            assert!(keys_ts.contains(&format!("'{}'", d.key)), "{} is not in keys.ts", d.key);
            for (i, catalog) in ftl.iter().enumerate() {
                for prefix in ["admin-settings-key-", "admin-settings-help-"] {
                    assert!(catalog.contains(&format!("
{prefix}{} = ", slug(d.key))), "{} lacks {prefix} in catalog {i}", d.key);
                }
            }
            if let settings_registry::Kind::Int { min, max } = d.kind {
                let default = d.default_value();
                assert!(settings_registry::validate(d, &default, "/value").is_ok(), "{} default outside bounds", d.key);
                assert!(settings_registry::validate(d, &json!(min - 1), "/value").is_err(), "{}", d.key);
                assert!(settings_registry::validate(d, &json!(max + 1), "/value").is_err(), "{}", d.key);
            }
            // The server's sweep reads these once for the whole server; only the
            // user-rule limits can differ by domain.
            let per_domain = d.scopes.allows(SettingScope::Domain);
            assert_eq!(per_domain, d.key.starts_with("retention.user.") && !matches!(d.key, "retention.user.runForRestricted" | "retention.user.intervalHours" | "retention.user.batchMessages" | "retention.user.maxPerRun" | "retention.user.pauseMs"), "{}", d.key);
            assert!(!d.scopes.allows(SettingScope::User), "{} must not be a per-user setting", d.key);
        }
        assert_eq!(found, 36);
        // The defaults keep the cross-check: long >= medium >= short.
        let n = |k: &str| settings_registry::lookup(k).unwrap().default_value().as_i64().unwrap();
        assert!(n("retention.class.long.days") >= n("retention.class.medium.days"));
        assert!(n("retention.class.medium.days") >= n("retention.class.short.days"));
        // Long may be 0 (forever); short and medium may not.
        assert!(settings_registry::validate(settings_registry::lookup("retention.class.long.days").unwrap(), &json!(0), "/v").is_ok());
        assert!(settings_registry::validate(settings_registry::lookup("retention.class.short.days").unwrap(), &json!(0), "/v").is_err());
        assert!(settings_registry::validate(settings_registry::lookup("retention.class.medium.days").unwrap(), &json!(0), "/v").is_err());
    }

    async fn store() -> (Store, PathBuf) {
        let dir = std::env::temp_dir().join(format!("mw-tier-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let s = Store::open(dir.join("s.sqlite").to_str().unwrap(), mw_store::ServerKey::from_bytes(&[4u8; 32]).unwrap())
            .await
            .unwrap();
        (s, dir)
    }

    async fn set(s: &Store, key: &str, v: &str) {
        s.put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: key.into(),
            global_value: Some(v.into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn effective_system_reports_class_days_and_the_archive_for_personal_stores() {
        let (s, dir) = store().await;
        let find = |v: &[Value], id: &str| v.iter().find(|x| x["id"] == id).cloned().unwrap();
        let before = effective_system(&s, "a", "a@example.test").await;
        let dlp = find(&before, "dlpAudit");
        assert_eq!((dlp["class"].as_str(), dlp["archived"].as_bool(), dlp["days"].as_i64()), (Some("long"), Some(false), Some(365)));
        assert!(before.iter().all(|x| x["id"] != "jobRuns"), "only stores that hold a person's data");

        set(&s, "retention.archive.enabled", "true").await;
        set(&s, "retention.sessions.idleDays", "0").await;
        set(&s, "retention.class.enforce", "true").await;
        let after = effective_system(&s, "a", "a@example.test").await;
        let dlp = find(&after, "dlpAudit");
        assert_eq!((dlp["archived"].as_bool(), dlp["archiveDays"].as_i64()), (Some(true), Some(2555)));
        let sessions = find(&after, "sessions");
        assert_eq!((sessions["class"].as_str(), sessions["days"].as_i64(), sessions["keptForever"].as_bool()), (Some("short"), Some(365), Some(false)), "0 became the class ceiling");
        std::fs::remove_dir_all(dir).ok();
    }

    #[tokio::test]
    async fn maintain_verifies_flags_tampering_and_purges_by_class_age() {
        use sqlx::sqlite::SqlitePoolOptions;
        let (s, dir) = store().await;
        let arch = dir.join("arch");
        set(&s, "retention.archive.directory", &serde_json::to_string(arch.to_str().unwrap()).unwrap()).await;
        set(&s, "retention.archive.enabled", "true").await;
        set(&s, "retention.class.long.days", "3650").await;
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite://{}", dir.join("s.sqlite").display()))
            .await
            .unwrap();
        let now = 1_800_000_000i64;
        for (id, ts) in [("a", "2012-01-01T00:00:00Z"), ("b", "2026-01-01T00:00:00Z")] {
            sqlx::query("INSERT INTO assist_audit(id,ts,actor,capability,scope_summary,endpoint_host) VALUES(?,?,'x','c','s','h')")
                .bind(id)
                .bind(ts)
                .execute(&pool)
                .await
                .unwrap();
        }
        let cfg = archive_cfg(&s).await;
        assert!(cfg.enabled);
        let t = archive::archive_table("assistAudit").unwrap();
        let moved = s.archive_move(t, now, 100, 5, &cfg.dir, cfg.key_gen, now, &[], false).await.unwrap();
        assert_eq!(moved.rows, 2);

        let run = maintain(&s, now, true, false).await.unwrap();
        let v = run.verified.unwrap();
        assert_eq!((v.checked, v.ok), (1, 1));
        // The 2012 row is older than ten years; the segment's newest row is 2026, so it stays.
        assert_eq!(run.purged, 0);

        // Tamper, then the next check names the segment.
        let seg = s.archive_segments(None).await.unwrap().remove(0);
        let f = cfg.dir.join(&seg.location);
        let mut bytes = std::fs::read(&f).unwrap();
        bytes[10] ^= 1;
        std::fs::write(&f, bytes).unwrap();
        let v = maintain(&s, now, true, false).await.unwrap().verified.unwrap();
        assert_eq!(v.failed.len(), 1);

        // Legal hold stops removal; without it the segment goes once past the class age.
        set(&s, "retention.archive.legalHold", "true").await;
        let later = now + 3651 * DAY * 2;
        assert_eq!(maintain(&s, later, false, false).await.unwrap().purged, 0);
        set(&s, "retention.archive.legalHold", "false").await;
        assert_eq!(maintain(&s, later, false, true).await.unwrap().purged, 1, "dry run counts");
        assert_eq!(s.archive_totals().await.unwrap().segments, 1);
        assert_eq!(maintain(&s, later, false, false).await.unwrap().purged, 1);
        assert_eq!(s.archive_totals().await.unwrap().segments, 0);
        std::fs::remove_dir_all(dir).ok();
    }
}
