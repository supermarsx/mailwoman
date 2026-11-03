//! `GET /api/policy` — the part of the admin security policy that the Mailwoman
//! clients enforce themselves, for a signed-in user:
//!
//! ```json
//! { "maxSecurityFloor": null | "full-sanitized" | "sanitized-no-media" | "plain-text",
//!   "capturePolicy": "off" | "protect",
//!   "openTracking": true | false }
//! ```
//!
//! `openTracking` is false when `MW_PUBLIC_URL` is unset or the authenticated
//! sender's effective feature/locked preference forbids tracking. The composer
//! hides its per-message control then.
//!
//! The reader does not offer a rendering mode less restrictive than
//! `maxSecurityFloor`; with `capturePolicy: "protect"` the desktop and Android apps
//! ask the operating system to keep their window out of screenshots (a browser has
//! no such switch). The server cannot check that a client honours either.
//!
//! Session required (cookie or native bearer, as every `/api/*` route);
//! `Cache-Control: no-store`, so a changed policy is read at the next fetch.

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde_json::json;

use crate::AppState;

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/api/policy", get(get_policy))
}

async fn get_policy(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    let open_tracking = match crate::open_tracking::enabled_for(&state, &session).await {
        Ok(allowed) => allowed,
        Err(e) => {
            tracing::error!("GET /api/policy tracking policy: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "tracking policy unavailable",
            )
                .into_response();
        }
    };
    match state.v6.admin.effective_security_policy().await {
        Ok(e) => (
            [(header::CACHE_CONTROL, "no-store")],
            Json(json!({
                "maxSecurityFloor": e.policy.max_security_floor.map(|f| f.as_str()),
                "capturePolicy": e.policy.capture_policy.as_str(),
                "openTracking": open_tracking,
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("GET /api/policy: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CACHE_CONTROL, "no-store")],
                Json(json!({ "error": "the security policy could not be read" })),
            )
                .into_response()
        }
    }
}
