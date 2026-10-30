//! Real, bounded server checks and persisted redacted history.
#[path = "diagnostics_jobs.rs"]
mod jobs;
#[path = "diagnostics_posture.rs"]
mod posture;
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json,
    extract::{Query, State},
    http::{Extensions, HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use mw_store::diagnostics::DiagnosticRun;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    jobs::register(r);
    r.admin_endpoint(Method::POST, "/diag/run", Capability::DiagnosticsRun, run);
    r.admin_endpoint(
        Method::GET,
        "/diag/runs",
        Capability::DiagnosticsRead,
        history,
    );
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckResult {
    id: &'static str,
    status: &'static str,
    /// Whole milliseconds, truncated (kept for older clients).
    duration_ms: u64,
    /// The same time in microseconds, so a sub-millisecond check is not shown as 0.
    duration_us: u64,
    summary: std::borrow::Cow<'static, str>,
    fix_id: &'static str,
}

async fn check(
    id: &'static str,
    summary: &'static str,
    action: impl std::future::Future<Output = bool>,
) -> CheckResult {
    let start = Instant::now();
    let status = match tokio::time::timeout(Duration::from_secs(5), action).await {
        Ok(true) => "pass",
        _ => "fail",
    };
    CheckResult::timed(id, status, start.elapsed(), summary.into())
}

impl CheckResult {
    /// A result whose duration is `elapsed`, in whole ms and in µs.
    pub(crate) fn timed(
        id: &'static str,
        status: &'static str,
        elapsed: Duration,
        summary: std::borrow::Cow<'static, str>,
    ) -> Self {
        CheckResult {
            id,
            status,
            duration_ms: elapsed.as_millis().min(u64::MAX as u128) as u64,
            duration_us: elapsed.as_micros().min(u64::MAX as u128) as u64,
            summary,
            fix_id: id,
        }
    }
}

/// Server-suite runs in flight, per admin session (t104). A second identical
/// request while one runs (a double click, a second tab) waits for that run and
/// gets its report, marked `deduplicated`, instead of spending the budget or
/// probing twice.
type Inflight = HashMap<String, tokio::sync::watch::Receiver<Option<serde_json::Value>>>;
static RUNNING: OnceLock<Mutex<Inflight>> = OnceLock::new();
fn running() -> std::sync::MutexGuard<'static, Inflight> {
    RUNNING
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}
/// Removes the session's in-flight entry however the leading run ends.
struct Leader(String);
impl Drop for Leader {
    fn drop(&mut self) {
        running().remove(&self.0);
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunInput {
    suite: String,
}
async fn run(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    ext: Extensions,
    AdminJson(input): AdminJson<RunInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::DiagnosticsRun)?;
    if input.suite != "server" {
        return Err(AdminApiError::invalid(
            "unsupported_suite",
            "Choose the server suite; other diagnostic suites are not available yet.",
            "/suite",
        ));
    }
    let follower = running().get(&ctx.session_hash).cloned();
    if let Some(mut rx) = follower
        && let Ok(value) = rx.wait_for(Option::is_some).await
        && let Some(mut value) = (*value).clone()
    {
        value["deduplicated"] = json!(true);
        return Ok(private(value));
    }
    crate::request_budget::diagnostics(&state, &ctx).await?;
    let (tx, rx) = tokio::sync::watch::channel(None);
    running().insert(ctx.session_hash.clone(), rx);
    let _leader = Leader(ctx.session_hash.clone());
    let value = server_suite(&state, ctx, &headers, &ext).await?;
    let _ = tx.send(Some(value.clone()));
    Ok(private(value))
}
fn private(value: serde_json::Value) -> Response {
    ([(header::CACHE_CONTROL, "private, no-store")], Json(value)).into_response()
}
async fn server_suite(
    state: &AppState,
    ctx: AdminCtx,
    headers: &HeaderMap,
    ext: &Extensions,
) -> Result<serde_json::Value, AdminApiError> {
    let id = uuid::Uuid::new_v4().to_string();
    let started = chrono::Utc::now().timestamp();
    // Audit before running even if a failing database later prevents history.
    let audit_saved = state
        .v6
        .admin
        .audit(mw_admin::AuditLogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            ts: chrono::Utc::now().to_rfc3339(),
            actor: ctx.username.clone(),
            actor_kind: mw_admin::ActorKind::Admin,
            action: "diagnostics.run".into(),
            target: Some(id.clone()),
            detail_json: json!({"suite":"server"}).to_string(),
            ip: None,
        })
        .await
        .is_ok();
    let mut results = vec![
        check("database-connectivity", "Database accepts a query", async {
            state.store.diagnostic_ping().await.is_ok()
        })
        .await,
        check(
            "database-write",
            "Scratch write succeeds and is rolled back",
            async { state.store.diagnostic_write_probe(&id).await.is_ok() },
        )
        .await,
        check(
            "database-migrations",
            "Applied migration count matches this server build",
            async {
                state
                    .store
                    .diagnostic_migrations()
                    .await
                    .is_ok_and(|(a, e)| a == e as i64)
            },
        )
        .await,
        check(
            "server-key-roundtrip",
            "Server key seals and opens a synthetic sample",
            async { state.store.diagnostic_seal_probe().is_ok() },
        )
        .await,
        check(
            "egress-private-policy",
            "Default egress policy refuses loopback and private addresses",
            async {
                ["127.0.0.1", "10.0.0.1", "::1", "fc00::1"]
                    .iter()
                    .all(|s| s.parse().is_ok_and(|ip| !mw_egress::ip_allowed(&ip)))
            },
        )
        .await,
    ];
    let upstream = ctx
        .require_unscoped(Capability::DomainsUpstreamWrite)
        .is_ok();
    let (network, security) = tokio::join!(
        posture::network(state, upstream),
        posture::security(state, headers, ext)
    );
    results.extend(network);
    results.extend(security);
    let status = if results.iter().any(|r| r.status == "fail") {
        "fail"
    } else if results.iter().any(|r| r.status == "warn") {
        "warn"
    } else {
        "pass"
    };
    let run = DiagnosticRun {
        id,
        suite: "server".into(),
        started_at: started,
        finished_at: chrono::Utc::now().timestamp(),
        status: status.into(),
        results: serde_json::to_value(results).map_err(AdminApiError::internal)?,
        triggered_by: ctx.username,
    };
    let history_saved = state.store.save_diagnostic_run(&run).await.is_ok();
    let retention_applied = prune_history(&state.store).await.is_ok();
    Ok(
        json!({"run":run,"historySaved":history_saved,"auditSaved":audit_saved,"retentionApplied":retention_applied}),
    )
}

#[derive(Deserialize)]
struct HistoryInput {
    cursor: Option<String>,
}
async fn prune_history(store: &mw_store::Store) -> Result<(), mw_store::StoreError> {
    let subject = mw_store::settings_policy::SettingSubject {
        personal_owner: None,
        account_id: "",
        domain: "",
        address: "",
    };
    let days =
        crate::settings_registry::effective_for(store, "diagnostics.retentionDays", &subject)
            .await?
            .and_then(|v| v.value.as_i64())
            .unwrap_or(30)
            .clamp(1, 365);
    store
        .prune_diagnostic_runs(chrono::Utc::now().timestamp() - days * 86400)
        .await
}
async fn history(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(input): Query<HistoryInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::DiagnosticsRead)?;
    prune_history(&state.store).await?;
    let cursor = input
        .cursor
        .as_deref()
        .map(|raw| {
            let (time, id) = raw.split_once(':').ok_or_else(|| {
                AdminApiError::invalid("invalid_cursor", "Invalid history cursor.", "/cursor")
            })?;
            let time = time.parse::<i64>().map_err(|_| {
                AdminApiError::invalid("invalid_cursor", "Invalid history cursor.", "/cursor")
            })?;
            if id.len() > 64 {
                return Err(AdminApiError::invalid(
                    "invalid_cursor",
                    "Invalid history cursor.",
                    "/cursor",
                ));
            }
            Ok((time, id))
        })
        .transpose()?;
    let mut runs = state.store.diagnostic_runs(cursor, 51).await?;
    let more = runs.len() > 50;
    runs.truncate(50);
    let next = if more {
        runs.last().map(|r| format!("{}:{}", r.started_at, r.id))
    } else {
        None
    };
    Ok((
        [(header::CACHE_CONTROL, "private, no-store")],
        Json(json!({"runs":runs,"nextCursor":next})),
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn check_reports_injected_failure_instead_of_success() {
        assert_eq!(check("probe", "Probe", async { true }).await.status, "pass");
        assert_eq!(
            check("probe", "Probe", async { false }).await.status,
            "fail"
        );
    }
}
