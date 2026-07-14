//! Assist gateway routes (plan §3 e9/e14, SPEC §14). SCAFFOLD stub (e0).
//!
//! `/api/assist/*` — the gateway HTTP surface (capability-scoped invoke +
//! streaming). The server PROXIES the endpoint so the browser never contacts the AI
//! host (CSP `connect-src 'self'`, mirroring the `/errors` tunnel). Filled by e9;
//! MOUNTED by e14. Returns 501 here.
#![allow(dead_code)]

use axum::Router;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};

use crate::AppState;

/// e14 merges this into `router()`. Returns 501 until e9 fills it.
pub(crate) fn assist_router() -> Router<AppState> {
    Router::new()
        .route("/api/assist/config", get(not_implemented))
        .route("/api/assist/invoke", post(not_implemented))
}

async fn not_implemented() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        "assist not yet implemented (t7 e9/e14)",
    )
}
