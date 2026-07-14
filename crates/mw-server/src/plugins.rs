//! Plugin-registry admin routes (plan §3 e9/e14, SPEC §22). SCAFFOLD stub (e0).
//!
//! `/admin/plugins/*` — list / approve / enable / disable / capability-grant +
//! the `allow_unsigned` banner policy over the 0008 `plugins`/`plugin_grants`
//! tables and the `mw-plugin` host. Filled by e9; MOUNTED by e14 (`router()` is
//! byte-unchanged until then). Every handler returns 501 here.
#![allow(dead_code)]

use axum::Router;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};

use crate::AppState;

/// e14 merges this into `router()`. Returns 501 until e9 fills it.
pub(crate) fn plugins_router() -> Router<AppState> {
    Router::new()
        .route("/admin/plugins", get(not_implemented))
        .route("/admin/plugins/{id}/approve", post(not_implemented))
        .route("/admin/plugins/{id}/enable", post(not_implemented))
        .route("/admin/plugins/{id}/disable", post(not_implemented))
        .route("/admin/plugins/{id}/grant", post(not_implemented))
}

async fn not_implemented() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        "plugin registry not yet implemented (t7 e9/e14)",
    )
}
