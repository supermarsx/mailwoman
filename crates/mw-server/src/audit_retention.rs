//! Audit-log retention routes and the `audit_prune` job (t39 L5; t32-ext §14.3).
//!
//! The policy is the `audit_retention` table ([`mw_store::audit_identity::retention`]):
//! one row per category, `*` the default. With no row nothing is pruned.
//!
//! - `GET /admin/audit/retention` (`audit.read`, unscoped): the rows, bounds,
//!   last prune and the prune task.
//! - `PUT /admin/audit/retention/{category}` `{days, legalHold}` and `DELETE`
//!   (`settings.write` + `audit.read`, both unscoped). The first saved policy
//!   with a number of days creates the daily `audit_prune` task (03:15 UTC),
//!   so the policy is applied without a second step. A task the
//!   administrator disabled stays disabled.
//! - `POST /admin/audit/retention/prune`: prune now.
//!
//! Every change and every prune that removed rows is itself an audit entry. The
//! selection is by time and category only, so a prune never depends on, or
//! reports, which accounts exist.

use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::AdminRegistrar,
    capabilities::Capability,
    jobs::{JobCtx, JobOutput, Schedule},
};
use axum::{
    Json,
    extract::{Path, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
};
use mw_store::audit_identity::retention::{
    AuditPruneReport, AuditRetentionRow, CATEGORIES, MAX_DAYS, MIN_DAYS, valid_days,
};
use serde::Deserialize;
use serde_json::{Value, json};

/// The job kind.
pub(crate) const JOB_KIND: &str = "audit_prune";

/// The schedule the prune task is created with.
pub(crate) fn default_schedule() -> Schedule {
    Schedule::Daily {
        daily_at: "03:15".into(),
        tz: "UTC".into(),
    }
}

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/audit/retention",
        Capability::AuditRead,
        get_retention,
    );
    r.admin_endpoint(
        Method::POST,
        "/audit/retention/prune",
        Capability::SettingsWrite,
        prune_now,
    );
    r.admin_endpoint(
        Method::PUT,
        "/audit/retention/{category}",
        Capability::SettingsWrite,
        put_retention,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/audit/retention/{category}",
        Capability::SettingsWrite,
        delete_retention,
    );
}

fn require_writer(ctx: &AdminCtx) -> Result<(), AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    ctx.require_unscoped(Capability::AuditRead)
}

fn known_category(category: &str) -> Result<(), AdminApiError> {
    if CATEGORIES.contains(&category) {
        Ok(())
    } else {
        Err(AdminApiError::not_found("unknown audit category"))
    }
}

async fn snapshot(state: &AppState) -> Result<Value, AdminApiError> {
    let rows = state.store.audit_retention().await?;
    let last = state.store.last_audit_prune().await?;
    let task = state
        .store
        .list_scheduled_tasks()
        .await?
        .into_iter()
        .find(|t| t.kind == JOB_KIND)
        .map(|t| json!({"id":t.id,"enabled":t.enabled,"nextRunAt":t.next_run_at,"lastRunAt":t.last_run_at,"lastStatus":t.last_status}));
    Ok(json!({
        "categories": CATEGORIES,
        "rows": rows,
        "bounds": {"minDays": MIN_DAYS, "maxDays": MAX_DAYS},
        "lastPrune": last,
        "task": task,
        "defaultSchedule": default_schedule().to_json(),
    }))
}

fn no_store(value: Value) -> Response {
    (
        [(axum::http::header::CACHE_CONTROL, "no-store")],
        Json(value),
    )
        .into_response()
}

async fn get_retention(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::AuditRead)?;
    Ok(no_store(snapshot(&state).await?))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RetentionInput {
    days: Option<i64>,
    #[serde(default)]
    legal_hold: bool,
}

async fn put_retention(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(category): Path<String>,
    AdminJson(input): AdminJson<RetentionInput>,
) -> Result<Response, AdminApiError> {
    require_writer(&ctx)?;
    known_category(&category)?;
    if !valid_days(input.days) {
        return Err(AdminApiError::invalid(
            "retentionOutOfBounds",
            format!("days must be empty (keep every entry) or {MIN_DAYS} to {MAX_DAYS}"),
            "/days",
        )
        .with("minDays", MIN_DAYS)
        .with("maxDays", MAX_DAYS));
    }
    let before = state
        .store
        .audit_retention()
        .await?
        .into_iter()
        .find(|r| r.category == category);
    let row = AuditRetentionRow {
        category: category.clone(),
        days: input.days,
        legal_hold: input.legal_hold,
        updated_by: ctx.username.clone(),
        updated_at: chrono::Utc::now().timestamp(),
    };
    if !state.store.put_audit_retention(&row).await? {
        return Err(AdminApiError::invalid(
            "retentionOutOfBounds",
            "audit retention refused",
            "/days",
        ));
    }
    if row.days.is_some() {
        ensure_task(&state, &ctx.username).await?;
    }
    audit(
        &state,
        &ctx.username,
        "audit-retention-changed",
        &category,
        json!({
            "before": before.map(|b| json!({"days": b.days, "legalHold": b.legal_hold})),
            "after": {"days": row.days, "legalHold": row.legal_hold},
        }),
    )
    .await;
    Ok(no_store(snapshot(&state).await?))
}

async fn delete_retention(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(category): Path<String>,
) -> Result<Response, AdminApiError> {
    require_writer(&ctx)?;
    known_category(&category)?;
    if state.store.delete_audit_retention(&category).await? {
        audit(
            &state,
            &ctx.username,
            "audit-retention-reset",
            &category,
            json!({}),
        )
        .await;
    }
    Ok(no_store(snapshot(&state).await?))
}

async fn prune_now(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    require_writer(&ctx)?;
    let report = prune(&state, &ctx.username).await?;
    let mut value = snapshot(&state).await?;
    value["report"] = report_json(&report);
    Ok((StatusCode::OK, no_store(value)).into_response())
}

fn report_json(report: &AuditPruneReport) -> Value {
    let removed: serde_json::Map<String, Value> = report
        .removed
        .iter()
        .map(|(c, n)| (c.clone(), json!(n)))
        .collect();
    json!({"removed": removed, "total": report.total, "checkpointId": report.checkpoint_id})
}

/// Prune as of now and record it in the audit log when anything was removed.
pub(crate) async fn prune(
    state: &AppState,
    triggered_by: &str,
) -> Result<AuditPruneReport, mw_store::StoreError> {
    let report = state
        .store
        .prune_audit_log(chrono::Utc::now().timestamp())
        .await?;
    if report.total > 0 {
        audit(
            state,
            triggered_by,
            "audit-pruned",
            "log",
            report_json(&report),
        )
        .await;
    }
    Ok(report)
}

/// Create the enabled `audit_prune` task when there is none.
async fn ensure_task(state: &AppState, by: &str) -> Result<(), mw_store::StoreError> {
    if state
        .store
        .list_scheduled_tasks()
        .await?
        .iter()
        .any(|t| t.kind == JOB_KIND)
    {
        return Ok(());
    }
    let now = chrono::Utc::now().timestamp();
    state
        .store
        .insert_scheduled_task(&mw_store::jobs::ScheduledTaskRow {
            id: uuid::Uuid::new_v4().to_string(),
            kind: JOB_KIND.into(),
            schedule_json: default_schedule().to_json().to_string(),
            params_json: "{}".into(),
            enabled: true,
            only_in_window: false,
            created_by: by.into(),
            created_at: now,
            updated_at: now,
            last_run_at: None,
            last_status: None,
            next_run_at: None,
            lease_owner: None,
            lease_until: None,
        })
        .await
}

async fn audit(state: &AppState, actor: &str, action: &str, category: &str, detail: Value) {
    let (actor, kind) = if actor == "scheduler" {
        ("system".to_string(), mw_admin::ActorKind::System)
    } else {
        (actor.to_string(), mw_admin::ActorKind::Admin)
    };
    let entry = mw_admin::AuditLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor,
        actor_kind: kind,
        action: action.into(),
        target: Some(format!("audit-retention:{category}")),
        detail_json: mw_admin::redact_detail(&detail),
        ip: None,
    };
    if let Err(e) = state.v6.admin.audit(entry).await {
        tracing::warn!("audit retention entry not recorded: {e}");
    }
}

/// The `audit_prune` job.
pub(crate) async fn prune_job(ctx: JobCtx) -> Result<JobOutput, String> {
    let state = ctx.state()?;
    let report = prune(state, &ctx.triggered_by)
        .await
        .map_err(|e| format!("audit prune failed: {e}"))?;
    Ok(JobOutput::new(
        format!("audit entries removed: {}", report.total),
        report_json(&report),
    ))
}
