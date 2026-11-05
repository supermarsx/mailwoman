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
    Router::new()
        .route("/api/policy", get(get_policy))
        .route("/api/version", get(get_version))
}

/// The identity of the web app this server serves, for the "new version
/// available" toast. `webBuild` is a hash of the sorted file names under
/// `assets/` (Vite fingerprints them, so any rebuilt bundle changes it) of the
/// directory actually served: `MW_WEB_DIR` when set, else the embedded bundle.
/// Names only are hashed, so a poll never reads the bundle's bytes.
pub(crate) fn web_build_id(web_dir: Option<&std::path::Path>) -> String {
    use sha2::{Digest, Sha256};
    let mut names: Vec<String> = match web_dir {
        Some(dir) => std::fs::read_dir(dir.join("assets"))
            .map(|entries| {
                entries
                    .flatten()
                    .filter_map(|e| e.file_name().to_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        None => crate::WebAssets::iter()
            .filter_map(|p| p.strip_prefix("assets/").map(str::to_string))
            .collect(),
    };
    names.sort();
    let mut hash = Sha256::new();
    for name in &names {
        hash.update(name.as_bytes());
        hash.update([0]);
    }
    hash.finalize()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// `GET /api/version` — session required, `no-store`. Nothing here is served to
/// an unauthenticated caller.
async fn get_version(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(response) = crate::authed(&state, &headers).await {
        return response;
    }
    (
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({
            "version": env!("CARGO_PKG_VERSION"),
            "webBuild": web_build_id(state.web_dir.as_deref()),
        })),
    )
        .into_response()
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
                "clientTiming": crate::client_timing::client_view(&state, &session).await,
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
