//! Masked-email routes (t10 plan §3 e7, SPEC §28.4). SCAFFOLD stubs returning 501,
//! declared in `lib.rs` but **NOT mounted** — `router()` is byte-unchanged so
//! behaviour is identical. Filled by e7 (alias generate/enable/disable/delete +
//! target binding over the 0010 `masked_email` table) and MOUNTED by e13.
//!
//! Route shape (frozen here; mailbox-session-authed like the JMAP surface):
//!   * `GET    /api/masked`            — list the session account's aliases.
//!   * `POST   /api/masked`            — generate a new alias (optional target desc).
//!   * `POST   /api/masked/{id}/state` — enable/disable an alias.
//!   * `DELETE /api/masked/{id}`       — delete an alias.
#![allow(dead_code)]

use axum::Router;
use axum::extract::Path as UrlPath;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};

use crate::AppState;

/// A clean 501 for an unfilled masked-email route (e7 replaces the bodies).
fn not_implemented(what: &str) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        axum::Json(serde_json::json!({
            "error": format!("{what} is not implemented in this build")
        })),
    )
        .into_response()
}

/// e13 merges this into `router()` once e7 fills the handlers. Unmounted today.
pub(crate) fn masked_router() -> Router<AppState> {
    Router::new()
        .route("/api/masked", get(list).post(generate))
        .route("/api/masked/{id}/state", post(set_state))
        .route("/api/masked/{id}", axum::routing::delete(delete_alias))
}

async fn list() -> Response {
    not_implemented("masked-email listing")
}

async fn generate() -> Response {
    not_implemented("masked-email generation")
}

async fn set_state(UrlPath(_id): UrlPath<String>) -> Response {
    not_implemented("masked-email state change")
}

async fn delete_alias(UrlPath(_id): UrlPath<String>) -> Response {
    not_implemented("masked-email deletion")
}
