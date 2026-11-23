//! Default scheduled tasks for the three retention jobs (t48 part 3):
//! `user_retention`, `retention_archive` and `gc_uploads`.
//!
//! Before t48 nothing created a task for these kinds, so none of them ran by
//! itself. [`ensure_default_tasks`] runs once when the scheduler starts
//! ([`super::start`]) and, for each kind:
//!
//! * claims the marker setting `jobs.seeded.<kind>`. A kind whose marker
//!   exists is left alone, so a task the administrator disabled or deleted is
//!   never created again;
//! * when a task of the kind exists already, keeps it and adds nothing;
//! * otherwise inserts an enabled task with the kind's default schedule.
//!
//! On an installation that already has accounts the task gets
//! `params.armAfter = now + 7 days`. Until then a run started by the scheduler
//! only previews: it changes nothing and records what it would change in the
//! run's output ([`preview_until`]). A fresh installation (no accounts) is
//! armed at once. "Run now" is not affected by `armAfter`.
//!
//! The seeded `gc_uploads` task removes uploads older than
//! [`GC_UPLOADS_HOURS`], longer than the command-line default of 24 hours,
//! because a draft kept only in a browser refers to its uploads and the server
//! cannot see it.

use mw_store::{Store, StoreError, jobs::ScheduledTaskRow};
use serde_json::{Map, Value, json};

use super::JobCtx;

/// The kinds seeded, in order.
pub(crate) const KINDS: [&str; 3] = [
    crate::user_retention::JOB_KIND,
    crate::retention_tiers::JOB_KIND,
    "gc_uploads",
];

/// How long a seeded task only previews on an installation with accounts.
pub(crate) const PREVIEW_SECS: i64 = 7 * 86_400;

/// `olderThanHours` of the seeded `gc_uploads` task (30 days, the job's maximum).
pub(crate) const GC_UPLOADS_HOURS: i64 = 720;

/// The marker setting of `kind`.
pub(crate) fn marker(kind: &str) -> String {
    format!("jobs.seeded.{kind}")
}

/// Create the default task of each kind in [`KINDS`] once (see the module
/// documentation). Returns the kinds a task was created for.
pub(crate) async fn ensure_default_tasks(
    store: &Store,
    now: i64,
) -> Result<Vec<&'static str>, StoreError> {
    let has_accounts = !store.list_accounts().await?.is_empty();
    let mut created = Vec::new();
    for kind in KINDS {
        let Some(schedule) = super::spec(kind).and_then(|s| s.default_schedule) else {
            continue;
        };
        let mut params = Map::new();
        let arm_after = has_accounts.then_some(now + PREVIEW_SECS);
        if let Some(at) = arm_after {
            params.insert("armAfter".into(), json!(at));
        }
        if kind == "gc_uploads" {
            params.insert("olderThanHours".into(), json!(GC_UPLOADS_HOURS));
        }
        let row = ScheduledTaskRow {
            id: uuid::Uuid::new_v4().to_string(),
            kind: kind.to_string(),
            schedule_json: schedule.to_json().to_string(),
            params_json: Value::Object(params).to_string(),
            enabled: true,
            only_in_window: false,
            created_by: "system".into(),
            created_at: now,
            updated_at: now,
            last_run_at: None,
            last_status: None,
            next_run_at: None,
            lease_owner: None,
            lease_until: None,
        };
        let value = json!({"at": now, "taskId": row.id, "armAfter": arm_after}).to_string();
        if store.seed_scheduled_task(&marker(kind), &value, &row).await? {
            tracing::info!(
                kind,
                arm_after,
                "created the default scheduled task for a retention job"
            );
            created.push(kind);
        }
    }
    Ok(created)
}

/// The time a scheduled run of this task starts changing data, when that is
/// still ahead of `now`: the run must then only preview. `None` for "Run now"
/// and for a task without `armAfter` or past it.
pub(crate) fn preview_until(ctx: &JobCtx, now: i64) -> Option<i64> {
    if ctx.triggered_by != "scheduler" {
        return None;
    }
    ctx.params
        .get("armAfter")
        .and_then(Value::as_i64)
        .filter(|at| *at > now)
}

/// The preview line of a run's summary.
pub(crate) fn preview_summary(until: i64, would: u64, noun: &str) -> String {
    let date = chrono::DateTime::from_timestamp(until, 0)
        .map(|d| d.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| until.to_string());
    format!("preview only until {date}: would change {would} {noun}")
}

// ─── console routes ───────────────────────────────────────────────────────────
//
// `GET /admin/retention/jobs` lists the three jobs; `PUT /admin/retention/jobs/{kind}`
// with `{"enabled": bool}` switches one. Switching on a kind that has no task
// creates one the way seeding does (a 7-day preview when there are accounts).

use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json,
    extract::{Path, State},
    http::Method,
    response::{IntoResponse, Response},
};

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/retention/jobs", Capability::SettingsRead, list_route);
    r.admin_endpoint(Method::PUT, "/retention/jobs/{kind}", Capability::SettingsWrite, put_route);
}

fn no_store(v: Value) -> Response {
    ([(axum::http::header::CACHE_CONTROL, "no-store")], Json(v)).into_response()
}

/// The newest task of `kind`.
async fn task_of(store: &Store, kind: &str) -> Result<Option<ScheduledTaskRow>, StoreError> {
    Ok(store
        .list_scheduled_tasks()
        .await?
        .into_iter()
        .rev()
        .find(|t| t.kind == kind))
}

/// One job as the console shows it.
pub(crate) async fn job_json(store: &Store, kind: &str, now: i64) -> Result<Value, StoreError> {
    let task = task_of(store, kind).await?;
    let params: Value = task
        .as_ref()
        .and_then(|t| serde_json::from_str(&t.params_json).ok())
        .unwrap_or_else(|| json!({}));
    let arm_after = params["armAfter"].as_i64();
    let runs = store
        .list_job_runs(&mw_store::jobs::JobRunQuery {
            kind: Some(kind.to_string()),
            limit: 50,
            ..Default::default()
        })
        .await?;
    let detail = |r: &mw_store::jobs::JobRunRow| -> Value {
        r.detail_json
            .as_deref()
            .and_then(|d| serde_json::from_str(d).ok())
            .unwrap_or(Value::Null)
    };
    let last_run = runs.first().map(|r| {
        json!({"at": r.finished_at.unwrap_or(r.started_at), "status": r.status.as_str(),
               "summary": r.summary, "dryRun": detail(r)["dryRun"].as_bool().unwrap_or(false)})
    });
    let last_preview = runs
        .iter()
        .find(|r| detail(r)["dryRun"].as_bool() == Some(true))
        .map(|r| {
            json!({"at": r.finished_at.unwrap_or(r.started_at), "wouldChange": detail(r)["wouldChange"],
                   "summary": r.summary})
        });
    Ok(json!({
        "kind": kind,
        "taskId": task.as_ref().map(|t| t.id.clone()),
        "enabled": task.as_ref().is_some_and(|t| t.enabled),
        "schedule": task.as_ref().and_then(|t| serde_json::from_str::<Value>(&t.schedule_json).ok()),
        "nextRunAt": task.as_ref().and_then(|t| t.next_run_at),
        "armAfter": arm_after,
        "previewOnly": arm_after.is_some_and(|at| at > now),
        "olderThanHours": params["olderThanHours"],
        "lastRun": last_run,
        "lastPreview": last_preview,
    }))
}

async fn list_route(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsRead)?;
    let now = chrono::Utc::now().timestamp();
    let mut jobs = Vec::new();
    for kind in KINDS {
        jobs.push(job_json(&state.store, kind, now).await.map_err(AdminApiError::internal)?);
    }
    Ok(no_store(json!({"jobs": jobs, "now": now})))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PutInput {
    enabled: bool,
}

/// Switch one job on or off. Returns the job as [`job_json`] shows it.
pub(crate) async fn set_enabled(
    store: &Store,
    kind: &str,
    enabled: bool,
    by: &str,
    now: i64,
) -> Result<bool, StoreError> {
    match task_of(store, kind).await? {
        Some(mut t) => {
            if t.enabled == enabled {
                return Ok(false);
            }
            t.enabled = enabled;
            t.updated_at = now;
            // The scheduler gives a re-enabled task its next run.
            t.next_run_at = None;
            store.update_scheduled_task(&t).await
        }
        None if !enabled => Ok(false),
        None => {
            let Some(schedule) = super::spec(kind).and_then(|s| s.default_schedule) else {
                return Ok(false);
            };
            let mut params = Map::new();
            if !store.list_accounts().await?.is_empty() {
                params.insert("armAfter".into(), json!(now + PREVIEW_SECS));
            }
            if kind == "gc_uploads" {
                params.insert("olderThanHours".into(), json!(GC_UPLOADS_HOURS));
            }
            store
                .insert_scheduled_task(&ScheduledTaskRow {
                    id: uuid::Uuid::new_v4().to_string(),
                    kind: kind.to_string(),
                    schedule_json: schedule.to_json().to_string(),
                    params_json: Value::Object(params).to_string(),
                    enabled: true,
                    only_in_window: false,
                    created_by: by.to_string(),
                    created_at: now,
                    updated_at: now,
                    last_run_at: None,
                    last_status: None,
                    next_run_at: None,
                    lease_owner: None,
                    lease_until: None,
                })
                .await?;
            // Seeding must not add a second task later.
            let _ = store.set_setting(&marker(kind), &json!({"at": now, "by": "console"}).to_string()).await;
            Ok(true)
        }
    }
}

async fn put_route(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(kind): Path<String>,
    AdminJson(input): AdminJson<PutInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    let Some(kind) = KINDS.into_iter().find(|k| *k == kind) else {
        return Err(AdminApiError::not_found("No such retention job."));
    };
    let now = chrono::Utc::now().timestamp();
    let changed = set_enabled(&state.store, kind, input.enabled, &ctx.username, now)
        .await
        .map_err(AdminApiError::internal)?;
    if changed {
        let entry = mw_admin::AuditLogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            ts: chrono::Utc::now().to_rfc3339(),
            actor: ctx.username.clone(),
            actor_kind: mw_admin::ActorKind::Admin,
            action: if input.enabled { "retention-job-enabled" } else { "retention-job-disabled" }.into(),
            target: Some(format!("job:{kind}")),
            detail_json: mw_admin::redact_detail(&json!({"kind": kind, "enabled": input.enabled})),
            ip: None,
        };
        if let Err(e) = state.v6.admin.audit(entry).await {
            tracing::warn!("retention job audit entry not recorded: {e}");
        }
    }
    let job = job_json(&state.store, kind, now).await.map_err(AdminApiError::internal)?;
    Ok(no_store(job))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(by: &str, params: Value) -> JobCtx {
        JobCtx {
            kind: "gc_uploads".into(),
            run_id: "r".into(),
            task_id: Some("t".into()),
            params,
            triggered_by: by.into(),
            state: None,
        }
    }

    #[test]
    fn only_scheduled_runs_before_arm_time_preview() {
        let p = json!({"armAfter": 1000});
        assert_eq!(preview_until(&ctx("scheduler", p.clone()), 999), Some(1000));
        assert_eq!(preview_until(&ctx("scheduler", p.clone()), 1000), None);
        assert_eq!(preview_until(&ctx("admin@example.org", p), 10), None);
        assert_eq!(preview_until(&ctx("scheduler", json!({})), 10), None);
    }

    #[test]
    fn preview_summary_names_the_date() {
        assert_eq!(
            preview_summary(86_400, 3, "upload(s)"),
            "preview only until 1970-01-02 00:00 UTC: would change 3 upload(s)"
        );
    }
}
