//! Privileged authentication history; metadata only, never credentials or upstream errors.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json,
    extract::{Query, State},
    http::Method,
    response::{IntoResponse, Response},
};
use mw_store::{AuthLogLimits, AuthLogQuery, AuthSurface, SigninEvent};
use serde::Deserialize;
use serde_json::json;
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/logs/auth", Capability::LogsRead, history);
}
pub(crate) async fn limits(store: &mw_store::Store) -> AuthLogLimits {
    let subject = mw_store::settings_policy::SettingSubject {
        account_id: "",
        domain: "",
        address: "",
    };
    let days = crate::settings_registry::effective_for(store, "logs.auth.retentionDays", &subject)
        .await
        .ok()
        .flatten()
        .and_then(|v| v.value.as_u64())
        .unwrap_or(7)
        .clamp(1, 90);
    let cap = crate::settings_registry::effective_for(store, "logs.auth.maxEntries", &subject)
        .await
        .ok()
        .flatten()
        .and_then(|v| v.value.as_u64())
        .unwrap_or(50000)
        .clamp(1000, 100000);
    AuthLogLimits {
        retention_days: days as u32,
        max_entries: cap as u32,
    }
}
pub(crate) async fn record(
    state: &AppState,
    event: &SigninEvent,
    surface: AuthSurface,
    backend: Option<&str>,
) {
    if state
        .store
        .record_auth_event(event, surface, backend, limits(&state.store).await)
        .await
        .is_err()
    {
        warn_storage_failure();
    }
}
fn warn_storage_failure() {
    use std::sync::atomic::{AtomicI64, Ordering};
    static LAST: AtomicI64 = AtomicI64::new(0);
    let now = chrono::Utc::now().timestamp();
    let last = LAST.load(Ordering::Relaxed);
    if now.saturating_sub(last) >= 60
        && LAST
            .compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
    {
        tracing::warn!("authentication history storage unavailable");
    }
}
pub(crate) async fn record_admin(
    state: &AppState,
    username: &str,
    source: Option<std::net::IpAddr>,
    method: &str,
    success: bool,
) {
    record_admin_reason(
        state,
        username,
        source,
        method,
        success,
        if success { "" } else { "auth-rejected" },
    )
    .await;
}
pub(crate) async fn record_admin_reason(
    state: &AppState,
    username: &str,
    source: Option<std::net::IpAddr>,
    method: &str,
    success: bool,
    reason: &str,
) {
    let event = SigninEvent {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: None,
        username_attempted: username.into(),
        ts: chrono::Utc::now().timestamp(),
        result: if success { "success" } else { "failure" }.into(),
        method: method.into(),
        ip: source.map(|v| v.to_string()).unwrap_or_default(),
        user_agent: String::new(),
        geo_country: None,
        reason: reason.into(),
    };
    record(state, &event, AuthSurface::Admin, Some("admin-local")).await;
}
#[derive(Deserialize)]
struct Input {
    q: Option<String>,
    result: Option<String>,
    reason: Option<String>,
    backend: Option<String>,
    surface: Option<String>,
    cursor: Option<String>,
    limit: Option<u32>,
}
async fn history(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(input): Query<Input>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::LogsRead)?;
    if input.limit.is_some_and(|n| !(1..=100).contains(&n))
        || input.q.as_ref().is_some_and(|v| v.chars().count() > 128)
        || input.backend.as_ref().is_some_and(|v| v.len() > 128)
        || input.reason.as_ref().is_some_and(|v| v.len() > 64)
        || input
            .surface
            .as_deref()
            .is_some_and(|v| !matches!(v, "admin" | "webmail"))
        || input
            .result
            .as_deref()
            .is_some_and(|v| !matches!(v, "success" | "failure" | "blocked"))
    {
        return Err(AdminApiError::invalid(
            "invalid_filter",
            "Invalid authentication history filter.",
            "/",
        ));
    }
    let retention = limits(&state.store).await;
    state
        .store
        .prune_auth_events(chrono::Utc::now().timestamp(), retention)
        .await?;

    let before = input
        .cursor
        .as_deref()
        .map(|value| {
            let (ts, id) = value.split_once(':').ok_or_else(|| {
                AdminApiError::invalid("invalid_cursor", "Invalid cursor.", "/cursor")
            })?;
            let ts = ts.parse::<i64>().map_err(|_| {
                AdminApiError::invalid("invalid_cursor", "Invalid cursor.", "/cursor")
            })?;
            if id.is_empty() || id.len() > 128 {
                return Err(AdminApiError::invalid(
                    "invalid_cursor",
                    "Invalid cursor.",
                    "/cursor",
                ));
            }
            Ok((ts, id.to_owned()))
        })
        .transpose()?;
    let limit = input.limit.unwrap_or(50).clamp(1, 100);
    let mut events = state
        .store
        .auth_log_events(&AuthLogQuery {
            q: input.q,
            result: input.result,
            reason: input.reason,
            backend: input.backend,
            surface: input.surface,
            before,
            limit: limit + 1,
        })
        .await?;
    let more = events.len() > limit as usize;
    events.truncate(limit as usize);
    let cursor = if more {
        events
            .last()
            .map(|v| format!("{}:{}", v.last_seen, v.event.id))
    } else {
        None
    };
    Ok((
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        Json(json!({"events":events,"cursor":cursor,"retentionDays":retention.retention_days})),
    )
        .into_response())
}
