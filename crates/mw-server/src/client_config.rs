//! Minimal, uncached configuration exposed before mailbox sign-in.
use crate::{AppState, domain_config, settings_registry};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, header},
    response::{IntoResponse, Response},
};
use serde_json::json;

pub(crate) async fn public_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
) -> Response {
    let host = state.external.host(&headers, &ext);
    let identity = match domain_config::canonical(&state.store, "hint", host.as_deref()).await {
        Ok(identity) => identity,
        Err(_) => return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let subject = mw_store::settings_policy::SettingSubject {
        domain: &identity.domain,
        address: "",
        account_id: "",
    };
    let values = match settings_registry::effective_all(&state.store, &subject, |def| {
        matches!(
            def.key,
            "appearance.brandName" | "appearance.theme" | "appearance.accent" | "login.notice" | "login.showDomainHint" | "signin.retryCooldownSecs"
        )
    })
    .await
    {
        Ok(values) => values,
        Err(_) => return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let value = |key: &str| {
        values
            .iter()
            .find(|v| v.key == key)
            .map(|v| v.value.clone())
            .unwrap_or(serde_json::Value::Null)
    };
    let hint = if value("login.showDomainHint") != json!(true) || identity.domain.is_empty() {
        None
    } else {
        match state.store.domain_signin_state(&identity.domain).await {
            Ok(mw_store::DomainSigninState::Enabled) => Some(identity.domain),
            Ok(_) => None,
            Err(_) => return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
        }
    };
    let branding = match state.store.branding_snapshot().await {
        Ok(branding) => crate::branding_routes::image_urls(&branding.images),
        Err(_) => return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let mut response = Json(json!({
        "retryCooldownSecs":value("signin.retryCooldownSecs"),
        "brandName":value("appearance.brandName"), "theme":value("appearance.theme"),
        "accent":value("appearance.accent"), "loginNotice":value("login.notice"),
        "usernameDomainHint":hint, "showDomainHint":value("login.showDomainHint"),
        "logoUrl":branding["logoUrl"], "faviconUrl":branding["faviconUrl"],
        "loginBackgroundUrl":branding["loginBackgroundUrl"],
    }))
    .into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
}
