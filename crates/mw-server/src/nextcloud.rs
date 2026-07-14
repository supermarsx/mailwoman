//! Nextcloud routes (plan §3 e9/e14, SPEC §18.4). SCAFFOLD stub (e0).
//!
//! `/api/nextcloud/*` — attach-from / save-to / large-attachment share-link
//! creation via OCS/WebDAV. Filled by e9; MOUNTED by e14. Returns 501 here.
#![allow(dead_code)]

use axum::Router;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;

use crate::AppState;

/// e14 merges this into `router()`. Returns 501 until e9 fills it.
pub(crate) fn nextcloud_router() -> Router<AppState> {
    Router::new()
        .route("/api/nextcloud/attach", post(not_implemented))
        .route("/api/nextcloud/save", post(not_implemented))
        .route("/api/nextcloud/share-link", post(not_implemented))
}

async fn not_implemented() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        "nextcloud not yet implemented (t7 e9/e14)",
    )
}
