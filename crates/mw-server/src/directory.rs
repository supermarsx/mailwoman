//! Directory/GAL routes (plan §3 e9/e14, SPEC §13). SCAFFOLD stub (e0).
//!
//! `/api/directory/*` — GAL search / group-expand / cert-lookup / photo over
//! `mw-directory`. Filled by e9; MOUNTED by e14. Every handler returns 501 here.
#![allow(dead_code)]

use axum::Router;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;

use crate::AppState;

/// e14 merges this into `router()`. Returns 501 until e9 fills it.
pub(crate) fn directory_router() -> Router<AppState> {
    Router::new()
        .route("/api/directory/search", get(not_implemented))
        .route("/api/directory/group/{dn}", get(not_implemented))
        .route("/api/directory/cert", get(not_implemented))
        .route("/api/directory/photo", get(not_implemented))
}

async fn not_implemented() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        "directory not yet implemented (t7 e9/e14)",
    )
}
