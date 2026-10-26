//! Global, reversible admission pause. It never deletes sessions or cancels work.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson, etag_of, with_etag},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json,
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
const KEY: &str = "maintenance.mode.v1";
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Config {
    pub enabled: bool,
    pub message: String,
    pub retry_after_secs: u32,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            message: String::new(),
            retry_after_secs: 300,
        }
    }
}
impl Config {
    fn validate(&self) -> Result<(), &'static str> {
        if self.message.chars().count() > 1000 || self.message.chars().any(char::is_control) {
            return Err("Use at most 1000 plain-text characters without control characters.");
        }
        if !(1..=86400).contains(&self.retry_after_secs) {
            return Err("Retry time must be between 1 and 86400 seconds.");
        }
        Ok(())
    }
    fn parse(raw: Option<&str>) -> Result<Self, &'static str> {
        let Some(raw) = raw else {
            return Ok(Self::default());
        };
        if raw.len() > 8192 {
            return Err("Invalid maintenance configuration.");
        }
        let value: Self =
            serde_json::from_str(raw).map_err(|_| "Invalid maintenance configuration.")?;
        value.validate()?;
        Ok(value)
    }
}
pub(crate) async fn config(state: &AppState) -> Result<Config, AdminApiError> {
    let raw = state
        .store
        .get_setting(KEY)
        .await
        .map_err(AdminApiError::internal)?;
    Config::parse(raw.as_deref()).map_err(AdminApiError::internal)
}
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/maintenance/mode",
        Capability::MaintenanceRead,
        read,
    );
    r.admin_endpoint(
        Method::PUT,
        "/maintenance/mode",
        Capability::MaintenanceWrite,
        save,
    );
}
fn management_snapshot(raw: Option<&str>) -> (Config, bool, String) {
    match Config::parse(raw) {
        Ok(value) => { let tag = etag_of(&json!(value)); (value, false, tag) }
        Err(_) => (Config { enabled: true, ..Config::default() }, true,
            etag_of(&json!({"invalidMaintenanceConfig":raw}))),
    }
}
async fn read(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::MaintenanceRead)?;
    let raw = state.store.get_setting(KEY).await.map_err(AdminApiError::internal)?;
    let (value, invalid, tag) = management_snapshot(raw.as_deref());
    let mut body = json!(value);
    body["configurationInvalid"] = json!(invalid);
    Ok(with_etag(
        ([("cache-control", "private, no-store")], Json(body)),
        &tag,
    ))
}
async fn save(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(value): AdminJson<Config>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::MaintenanceWrite)?;
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::MaintenanceMode)
        .await?;
    value
        .validate()
        .map_err(|message| AdminApiError::invalid("invalidMaintenanceMode", message, "/"))?;
    let tag = headers
        .get("if-match")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            AdminApiError::new(
                StatusCode::PRECONDITION_REQUIRED,
                "ifMatchRequired",
                "Reload maintenance settings before saving.",
            )
        })?;
    if tag == "*" {
        return Err(AdminApiError::stale(None, None));
    }
    let raw = state
        .store
        .get_setting(KEY)
        .await
        .map_err(AdminApiError::internal)?;
    let (previous, invalid, previous_tag) = management_snapshot(raw.as_deref());
    if tag != previous_tag {
        return Err(AdminApiError::stale(None, None));
    }
    let encoded = serde_json::to_string(&value).map_err(AdminApiError::internal)?;
    if !state
        .store
        .compare_and_set_setting(KEY, raw.as_deref(), &encoded)
        .await
        .map_err(AdminApiError::internal)?
    {
        return Err(AdminApiError::stale(None, None));
    }
    crate::admin_accounts::audit(&state,&ctx,"maintenance-mode-configured","maintenance:global",None,Some(json!({"wasEnabled":if invalid {None}else{Some(previous.enabled)},"repairedInvalidConfiguration":invalid,"enabled":value.enabled,"retryAfterSecs":value.retry_after_secs}))).await;
    Ok(with_etag(
        ([("cache-control", "private, no-store")], Json(&value)),
        &etag_of(&json!(value)),
    ))
}
fn under(path: &str, base: &str) -> bool {
    path == base
        || path
            .strip_prefix(base)
            .is_some_and(|tail| tail.starts_with('/'))
}
fn governed(path: &str, method: &Method, admin: &str) -> bool {
    if under(path, admin) {
        return false;
    }
    if (*method == Method::GET || *method == Method::HEAD)
        && (matches!(path, "/api/public-config" | "/api/signin/challenge")
            || path
                .strip_prefix("/api/assets/")
                .is_some_and(|id| !id.is_empty() && !id.contains('/')))
    {
        return false;
    }
    if *method == Method::POST && matches!(path, "/api/logout" | "/oidc/logout" | "/oauth/revoke") {
        return false;
    }
    [
        "/api", "/jmap", "/dav", "/rc", "/sso", "/oidc", "/oauth", "/mcp",
    ]
    .iter()
    .any(|base| under(path, base))
}
fn refusal(value: Option<&Config>) -> Response {
    let retry = value.map_or(300, |v| v.retry_after_secs);
    let message = value.filter(|v| !v.message.is_empty()).map_or(
        "Mail service is temporarily unavailable for maintenance.",
        |v| v.message.as_str(),
    );
    (
        StatusCode::SERVICE_UNAVAILABLE,
        [
            ("cache-control", "no-store"),
            ("retry-after", &retry.to_string()),
        ],
        Json(json!({"code":"maintenance","message":message,"retryAfterSecs":retry})),
    )
        .into_response()
}
pub(crate) async fn guard(State(state): State<AppState>, request: Request, next: Next) -> Response {
    // Router middleware sees the application-relative URI after MW_BASE_PATH nesting.
    if !governed(
        request.uri().path(),
        request.method(),
        &mw_admin::config::admin_path(),
    ) {
        return next.run(request).await;
    }
    match config(&state).await {
        Ok(value) if !value.enabled => next.run(request).await,
        Ok(value) => refusal(Some(&value)),
        Err(_) => refusal(None),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn admission_classifies_exact_recovery_and_custom_admin_routes() {
        for path in [
            "/api/login",
            "/api/me",
            "/jmap/api",
            "/dav/calendar",
            "/mcp",
            "/sso/callback",
            "/api/rc/p/token",
            "/api/accounts",
        ] {
            assert!(governed(path, &Method::POST, "/ops"), "{path}");
        }
        for path in [
            "/ops",
            "/ops/login",
            "/ops/maintenance/mode",
            "/healthz",
            "/assets/app.js",
            "/workspace-popup.html",
            "/api/public-config",
            "/api/signin/challenge",
            "/api/assets/logo",
        ] {
            assert!(!governed(path, &Method::GET, "/ops"), "{path}");
        }
        assert!(governed("/api/public-config", &Method::POST, "/ops"));
        assert!(!governed("/api/logout", &Method::POST, "/ops"));
        assert!(governed("/api/logout/more", &Method::POST, "/ops"));
        assert!(governed("/api/assets/logo/more", &Method::GET, "/ops"));
    }
    #[test]
    fn defaults_and_message_bounds_are_explicit() {
        assert!(!Config::parse(None).unwrap().enabled);
        let mut value = Config { message: "<b>Planned work</b>".into(), ..Config::default() };
        assert!(value.validate().is_ok());
        value.message = "bad\r\nheader".into();
        assert!(value.validate().is_err());
        value.message.clear();
        value.retry_after_secs = 0;
        assert!(value.validate().is_err());
        assert!(Config::parse(Some("{}")).is_err());
    }
}
