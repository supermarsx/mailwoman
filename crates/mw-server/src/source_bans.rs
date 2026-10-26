//! Deployment IP bans use the same trusted-proxy authority as sign-in limits.
use axum::{Json, extract::{Request, State}, middleware::Next, http::{StatusCode, header}, response::{IntoResponse, Response}};
use crate::AppState;

pub(crate) async fn guard(State(state): State<AppState>, request: Request, next: Next) -> Response {
    // Liveness/readiness remain usable by deployment probes, even if their
    // transport address is banned. There is no blanket loopback exemption.
    if matches!(request.uri().path(), "/healthz" | "/readyz") {
        return next.run(request).await;
    }
    let Some(ip) = crate::scope_mw::proxy::client_ip(request.headers(), request.extensions()) else {
        // In-process synthetic requests have no peer; never trust a supplied
        // forwarding header as a substitute. Production installs ConnectInfo.
        return next.run(request).await;
    };
    let status = match state.v6.admin.is_banned(&ip.to_string()).await {
        Ok(false) => return next.run(request).await,
        Ok(true) => StatusCode::FORBIDDEN,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    };
    let mut response = (status, Json(serde_json::json!({"code":"sourceUnavailable","error":"Request unavailable."}))).into_response();
    response.headers_mut().insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response
}
