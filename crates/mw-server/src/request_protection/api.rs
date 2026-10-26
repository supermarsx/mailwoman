use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
fn private(body: serde_json::Value) -> Response {
    ([(header::CACHE_CONTROL, "private, no-store")], Json(body)).into_response()
}
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/bans/protection",
        Capability::BansRead,
        status,
    );
    r.admin_endpoint(
        Method::GET,
        "/bans/protection/subjects",
        Capability::BansRead,
        subjects,
    );
    r.admin_endpoint(
        Method::GET,
        "/bans/protection/events",
        Capability::BansRead,
        events,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/bans/protection/subjects/{key}",
        Capability::BansWrite,
        clear,
    );
}
async fn status(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::BansRead)?;
    let config = state
        .request_protection
        .cache
        .get(&state.store)
        .await
        .map_err(|_| super::unavailable())?;
    Ok(private(
        json!({"policy":config.summary(),"subjectCapacity":mw_store::request_protection::SUBJECT_CAPACITY,"eventCapacity":mw_store::request_protection::EVENT_CAPACITY,"eventRetentionSecs":mw_store::request_protection::EVENT_RETENTION_SECS,"propagationSecs":30,"coverage":"router-requests-verified-session-key-and-signed-content","preRouterHeadersObserved":false}),
    ))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Page {
    after: Option<String>,
    limit: Option<usize>,
}
fn page(query: &Page) -> Result<usize, AdminApiError> {
    let limit = query.limit.unwrap_or(50);
    if !(1..=100).contains(&limit) || query.after.as_ref().is_some_and(|value| value.len() > 160) {
        return Err(AdminApiError::invalid(
            "invalidPage",
            "Invalid page limit or cursor",
            "/limit",
        ));
    }
    Ok(limit)
}
fn key_valid(key: &str) -> bool {
    key.len() == 64
        && key
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}
async fn subjects(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(query): Query<Page>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::BansRead)?;
    let limit = page(&query)?;
    if query.after.as_ref().is_some_and(|key| !key_valid(key)) {
        return Err(AdminApiError::invalid(
            "invalidCursor",
            "Invalid subject cursor",
            "/after",
        ));
    }
    let rows = state
        .store
        .request_protection_rows(
            chrono::Utc::now().timestamp(),
            query.after.as_deref(),
            limit,
        )
        .await
        .map_err(|_| super::unavailable())?;
    let cursor = if rows.len() == limit {
        rows.last().map(|row| row.subject_key.clone())
    } else {
        None
    };
    Ok(private(json!({"subjects":rows,"cursor":cursor})))
}
async fn events(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(query): Query<Page>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::BansRead)?;
    let limit = page(&query)?;
    let cursor = query
        .after
        .as_deref()
        .map(|raw| {
            let (at, id) = raw.split_once(':').ok_or(())?;
            let at = at.parse::<i64>().map_err(|_| ())?;
            if at < 0 || id.len() != 32 || !id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(());
            }
            Ok((at, id))
        })
        .transpose()
        .map_err(|()| {
            AdminApiError::invalid("invalidCursor", "Invalid history cursor", "/after")
        })?;
    let rows = state
        .store
        .request_protection_events(chrono::Utc::now().timestamp(), cursor, limit)
        .await
        .map_err(|_| super::unavailable())?;
    let cursor = if rows.len() == limit {
        rows.last()
            .map(|row| format!("{}:{}", row.observed_at, row.id))
    } else {
        None
    };
    Ok(private(json!({"events":rows,"cursor":cursor})))
}
async fn clear(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(key): Path<String>,
    headers: HeaderMap,
) -> Result<StatusCode, AdminApiError> {
    ctx.require_unscoped(Capability::BansWrite)?;
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::BanManage).await?;
    if !key_valid(&key) {
        return Err(AdminApiError::invalid(
            "invalidSubject",
            "Invalid subject fingerprint",
            "/key",
        ));
    }
    let revision = headers
        .get(header::IF_MATCH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix('"'))
        .and_then(|value| value.strip_suffix('"'))
        .filter(|value| value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| {
            AdminApiError::new(
                StatusCode::PRECONDITION_REQUIRED,
                "ifMatchRequired",
                "Refresh the subject before clearing its state",
            )
        })?;
    if !state
        .store
        .clear_request_protection(&key, revision, chrono::Utc::now().timestamp())
        .await
        .map_err(|_| super::unavailable())?
    {
        return Err(AdminApiError::stale(None, None));
    }
    crate::audit::security_event(
        &state,
        crate::audit::SecurityEvent::admin(&ctx.username, "request-protection-cleared")
            .detail(json!({"subjectFingerprint":key})),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}
