//! Operational telemetry configuration. Changes apply on the next server start.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson, check_if_match, etag_of, with_etag},
    admin_router::AdminRegistrar,
    capabilities::Capability,
    telemetry_config::{self, TelemetryConfig},
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
        "/telemetry",
        Capability::ObservabilityRead,
        read,
    );
    r.admin_endpoint(
        Method::PUT,
        "/telemetry",
        Capability::IntegrationsWrite,
        save,
    );
}

async fn view(state: &AppState) -> Result<Response, AdminApiError> {
    let config = telemetry_config::load(&state.store)
        .await
        .map_err(AdminApiError::internal)?;
    let (effective, pinned) = config
        .clone()
        .with_environment(|key| std::env::var(key).ok())
        .map_err(|(_, message)| AdminApiError::internal(message))?;
    let tag = etag_of(&json!(config));
    Ok(with_etag(([("cache-control", "private, no-store")], Json(json!({
        "config":config, "effective":effective, "pinned":pinned,
        "active":telemetry_config::active(), "restartRequired":telemetry_config::active().is_some_and(|active| active != effective),
        "metricsTokenConfigured":std::env::var("MW_METRICS_TOKEN").is_ok_and(|token| !token.is_empty())
    }))).into_response(), &tag))
}

async fn read(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::ObservabilityRead)?;
    view(&state).await
}

async fn save(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(config): AdminJson<TelemetryConfig>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsWrite)?;
    crate::step_up::require_admin(
        &state,
        &ctx,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    config
        .validate()
        .map_err(|(path, message)| AdminApiError::invalid("invalidTelemetry", message, path))?;
    // Validate pinned environment values too, before persisting an unusable configuration.
    config
        .clone()
        .with_environment(|key| std::env::var(key).ok())
        .map_err(|(path, message)| AdminApiError::invalid("invalidTelemetry", message, path))?;
    if !headers.contains_key("if-match") {
        return Err(AdminApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "ifMatchRequired",
            "Reload telemetry before saving.",
        ));
    }
    let raw = state
        .store
        .get_setting(telemetry_config::SETTINGS_KEY)
        .await
        .map_err(AdminApiError::internal)?;
    let previous: TelemetryConfig = match &raw {
        Some(value) => serde_json::from_str(value).map_err(AdminApiError::internal)?,
        None => TelemetryConfig::default(),
    };
    check_if_match(&headers, &etag_of(&json!(previous)), None, None)?;
    let serialized = serde_json::to_string(&config).map_err(AdminApiError::internal)?;
    if !state
        .store
        .compare_and_set_setting(telemetry_config::SETTINGS_KEY, raw.as_deref(), &serialized)
        .await
        .map_err(AdminApiError::internal)?
    {
        return Err(AdminApiError::stale(None, None));
    }
    crate::admin_accounts::audit(&state, &ctx, "telemetry-configured", "telemetry:global", None,
        Some(json!({"enabled":config.enabled,"traces":config.traces,"metrics":config.metrics,"restartRequired":true}))).await;
    view(&state).await
}
