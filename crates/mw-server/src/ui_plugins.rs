//! TypeScript UI-plugin registry admin + client routes (t10 plan §3 e11, SPEC §22.2).
//! SCAFFOLD stubs returning 501, declared in `lib.rs` but **NOT mounted** — `router()`
//! is byte-unchanged so behaviour is identical. Filled by e11 (registry + capability
//! gating + admin approval + ed25519 signature verify over the 0010 `ui_plugins` /
//! `ui_plugin_grants` tables) and MOUNTED by e13.
//!
//! Route shape (frozen here so e10 web / e11 server agree):
//!   * `GET  /admin/ui-plugins`                 — list registered UI plugins (admin).
//!   * `POST /admin/ui-plugins`                 — upload/register a plugin bundle (admin).
//!   * `POST /admin/ui-plugins/{id}/approve`    — approve + enable (admin).
//!   * `POST /admin/ui-plugins/{id}/grant`      — grant a declared capability (admin).
//!   * `POST /admin/ui-plugins/{id}/disable`    — disable (admin).
//!   * `GET  /api/ui-plugins`                    — the approved+enabled tier the SPA loads.
//!
//! Deny-by-default: `/api/ui-plugins` returns only approved+enabled plugins; a
//! capability grant is intersected with the manifest's declared capabilities.
#![allow(dead_code)]

use axum::Router;
use axum::extract::Path as UrlPath;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};

use crate::AppState;

/// A clean 501 for an unfilled UI-plugin route (e11 replaces the bodies).
fn not_implemented(what: &str) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        axum::Json(serde_json::json!({
            "error": format!("{what} is not implemented in this build")
        })),
    )
        .into_response()
}

/// e13 merges this into `router()` once e11 fills the handlers. Unmounted today.
pub(crate) fn ui_plugins_router() -> Router<AppState> {
    Router::new()
        .route("/admin/ui-plugins", get(list_admin).post(register))
        .route("/admin/ui-plugins/{id}/approve", post(approve))
        .route("/admin/ui-plugins/{id}/grant", post(grant))
        .route("/admin/ui-plugins/{id}/disable", post(disable))
        .route("/api/ui-plugins", get(list_public))
}

async fn list_admin() -> Response {
    not_implemented("UI-plugin admin listing")
}

async fn register() -> Response {
    not_implemented("UI-plugin registration")
}

async fn approve(UrlPath(_id): UrlPath<String>) -> Response {
    not_implemented("UI-plugin approval")
}

async fn grant(UrlPath(_id): UrlPath<String>) -> Response {
    not_implemented("UI-plugin capability grant")
}

async fn disable(UrlPath(_id): UrlPath<String>) -> Response {
    not_implemented("UI-plugin disable")
}

async fn list_public() -> Response {
    not_implemented("UI-plugin tier listing")
}
