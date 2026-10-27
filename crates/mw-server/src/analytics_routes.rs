use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson, check_if_match, etag_of, with_etag},
    admin_router::AdminRegistrar,
    analytics_adapter::Event,
    analytics_config::{self, Update},
    analytics_runtime,
    capabilities::Capability,
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::json;

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/analytics/status", get(user_status))
        .route(
            "/api/analytics/events",
            post(event).layer(DefaultBodyLimit::max(1024)),
        )
}
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/analytics",
        Capability::IntegrationsRead,
        read,
    );
    r.admin_endpoint(
        Method::PUT,
        "/analytics",
        Capability::IntegrationsWrite,
        save,
    );
    r.admin_endpoint(
        Method::POST,
        "/analytics/allowlist",
        Capability::EgressWrite,
        allow,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/analytics/allowlist/{id}",
        Capability::EgressWrite,
        remove_allow,
    );
}
async fn user_status(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    let storage_owner =
        match crate::prefs_routes::personal_storage::owner(&state, &session, &headers, false).await
        {
            Ok(owner) => owner,
            Err(response) => return response,
        };
    let enabled = analytics_config::load(&state.store)
        .await
        .is_ok_and(|config| config.enabled);
    ([("cache-control", "private, no-store")], Json(json!({"enabled":enabled,"consent":analytics_runtime::consent(&state.store,&storage_owner).await,"privacySignal":analytics_runtime::privacy_signal(&headers),"mode":"event-counts"}))).into_response()
}
async fn event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(event): Json<Event>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    let storage_owner =
        match crate::prefs_routes::personal_storage::owner(&state, &session, &headers, false).await
        {
            Ok(owner) => owner,
            Err(response) => return response,
        };
    if analytics_runtime::privacy_signal(&headers)
        || !analytics_runtime::consent(&state.store, &storage_owner).await
    {
        return StatusCode::NO_CONTENT.into_response();
    }
    let config = match analytics_config::load(&state.store).await {
        Ok(config) if config.enabled => config,
        _ => return StatusCode::NO_CONTENT.into_response(),
    };
    let identity = match state
        .store
        .security_account_address(&session.account_id)
        .await
    {
        Ok(Some(identity)) => identity,
        _ => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    if let Some((_, domain)) = identity.rsplit_once('@')
        && state.store.canonical_policy_domain(domain).await.is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    if state
        .analytics
        .enqueue(state.clone(), session, storage_owner, config, event.screen)
    {
        StatusCode::ACCEPTED
    } else {
        StatusCode::TOO_MANY_REQUESTS
    }
    .into_response()
}
async fn view(state: &AppState, ctx: &AdminCtx) -> Result<Response, AdminApiError> {
    let config = analytics_config::load(&state.store)
        .await
        .map_err(|_| AdminApiError::internal("analytics config"))?;
    let rows = if ctx.has_unscoped(Capability::EgressRead) {
        state
            .store
            .list_egress_internal_allow()
            .await
            .map_err(AdminApiError::internal)?
    } else {
        Vec::new()
    };
    let allowlist: Vec<_> = rows
        .into_iter()
        .filter(|row| row.traffic_class == "analytics")
        .map(|row| json!({"id":row.id,"destination":row.destination}))
        .collect();
    let tag = etag_of(&json!(config));
    Ok(with_etag(
        (
            [("cache-control", "private, no-store")],
            Json(
                json!({"config":config.public(),"status":state.analytics.status(),"allowlist":allowlist}),
            ),
        ),
        &tag,
    ))
}
async fn read(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsRead)?;
    view(&state, &ctx).await
}
async fn save(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(update): AdminJson<Update>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsWrite)?;
    crate::step_up::require_admin(
        &state,
        &ctx,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    let raw = state
        .store
        .get_setting(analytics_config::KEY)
        .await
        .map_err(AdminApiError::internal)?;
    let previous = analytics_config::load(&state.store)
        .await
        .map_err(|_| AdminApiError::internal("analytics config"))?;
    if !headers.contains_key("if-match") {
        return Err(AdminApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "ifMatchRequired",
            "Reload the configuration before saving.",
        ));
    }
    check_if_match(&headers, &etag_of(&json!(previous)), None, None)?;
    let config = update
        .prepare(&previous, &state.store)
        .map_err(|message| AdminApiError::invalid("invalidAnalytics", message, "/config"))?;
    let serialized = serde_json::to_string(&config).expect("analytics config");
    if !state
        .store
        .compare_and_set_setting(analytics_config::KEY, raw.as_deref(), &serialized)
        .await
        .map_err(AdminApiError::internal)?
    {
        return Err(AdminApiError::stale(None, None));
    }
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "analytics-configured",
        "analytics:global",
        None,
        Some(json!({"enabled":config.enabled,"provider":config.provider})),
    )
    .await;
    view(&state, &ctx).await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Allow {
    destination: String,
}
async fn allow(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(body): AdminJson<Allow>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::EgressWrite)?;
    crate::step_up::require_admin(
        &state,
        &ctx,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    let destination = mw_egress::AllowDestination::parse(&body.destination)
        .map_err(|message| AdminApiError::invalid("invalidDestination", message, "/destination"))?;
    let id = uuid::Uuid::new_v4().to_string();
    state
        .store
        .add_egress_internal_allow(&mw_store::EgressAllowRow {
            id: id.clone(),
            traffic_class: "analytics".into(),
            destination: destination.to_string(),
            note: "Analytics collector".into(),
            created_by: ctx.username.clone(),
            created_at: chrono::Utc::now().timestamp(),
        })
        .await
        .map_err(AdminApiError::internal)?;
    crate::image_proxy::refresh_plan(&state.store)
        .await
        .map_err(|_| AdminApiError::internal("egress reload"))?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "analytics-destination-allowed",
        &format!("analytics-allow:{id}"),
        None,
        Some(json!({"destination":destination.to_string()})),
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}
async fn remove_allow(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::EgressWrite)?;
    crate::step_up::require_admin(
        &state,
        &ctx,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    let rows = state
        .store
        .list_egress_internal_allow()
        .await
        .map_err(AdminApiError::internal)?;
    if !rows
        .iter()
        .any(|row| row.id == id && row.traffic_class == "analytics")
    {
        return Err(AdminApiError::not_found("Analytics destination not found."));
    }
    state
        .store
        .delete_egress_internal_allow(&id)
        .await
        .map_err(AdminApiError::internal)?;
    crate::image_proxy::refresh_plan(&state.store)
        .await
        .map_err(|_| AdminApiError::internal("egress reload"))?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "analytics-destination-removed",
        &format!("analytics-allow:{id}"),
        None,
        None,
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}
