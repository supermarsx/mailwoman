use super::config::{Config, KEY};
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson, check_if_match, etag_of, with_etag},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/security/inbound-allowlist",
        Capability::SecurityRead,
        read,
    );
    r.admin_endpoint(
        Method::PUT,
        "/security/inbound-allowlist",
        Capability::SecurityWrite,
        save,
    );
}
async fn read(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SecurityRead)?;
    let mut snapshot = state.inbound_allowlist.inner.lock().await;
    let raw = state
        .store
        .get_setting(KEY)
        .await
        .map_err(AdminApiError::internal)?;
    let config = Config::parse(raw.as_deref())
        .map_err(|_| AdminApiError::internal("invalid inbound policy"))?;
    snapshot.replace(config.clone());
    state.inbound_allowlist.refresh(&mut snapshot, &config);
    Ok(with_etag(
        (
            [("cache-control", "private, no-store")],
            Json(json!({"config":config,"status":snapshot.status()})),
        ),
        &etag_of(&json!(config)),
    ))
}
async fn save(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(config): AdminJson<Config>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SecurityWrite)?;
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::BanManage).await?;
    config
        .validate()
        .map_err(|message| AdminApiError::invalid("invalidInboundPolicy", message, "/config"))?;
    if !headers.contains_key("if-match") {
        return Err(AdminApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "ifMatchRequired",
            "Reload the policy before saving.",
        ));
    }
    let mut snapshot = state.inbound_allowlist.inner.lock().await;
    let raw = state
        .store
        .get_setting(KEY)
        .await
        .map_err(AdminApiError::internal)?;
    let previous = Config::parse(raw.as_deref())
        .map_err(|_| AdminApiError::internal("invalid inbound policy"))?;
    check_if_match(&headers, &etag_of(&json!(previous)), None, None)?;
    let serialized = serde_json::to_string(&config).map_err(AdminApiError::internal)?;
    if !state
        .store
        .compare_and_set_setting(KEY, raw.as_deref(), &serialized)
        .await
        .map_err(AdminApiError::internal)?
    {
        return Err(AdminApiError::stale(None, None));
    }
    snapshot.replace(config.clone());
    state.inbound_allowlist.refresh(&mut snapshot, &config);
    let response = with_etag(
        (
            [("cache-control", "private, no-store")],
            Json(json!({"config":config,"status":snapshot.status()})),
        ),
        &etag_of(&json!(config)),
    )
    .into_response();
    drop(snapshot);
    crate::admin_accounts::audit(&state, &ctx, "inbound-allowlist-configured", "inbound-allowlist:global", None,
        Some(json!({"adminEnabled":config.admin.enabled,"webmailEnabled":config.webmail.enabled,"hostCount":config.hosts().len()}))).await;
    Ok(response)
}
