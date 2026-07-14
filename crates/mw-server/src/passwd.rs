//! Password-change route (plan §3 e9/e14, SPEC §18.3). SCAFFOLD stub (e0).
//!
//! `POST /api/password` — change via a `mw-passwd` backend, then re-seal upstream
//! creds on success and (for zero-access) signal the client-side re-wrap. Filled by
//! e9; MOUNTED by e14. Returns 501 here.
#![allow(dead_code)]

use axum::Router;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};

use crate::AppState;

/// e14 merges this into `router()`. Returns 501 until e9 fills it.
pub(crate) fn passwd_router() -> Router<AppState> {
    Router::new()
        .route("/api/password", post(not_implemented))
        .route("/api/password/policy", get(not_implemented))
}

async fn not_implemented() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        "password change not yet implemented (t7 e9/e14)",
    )
}
