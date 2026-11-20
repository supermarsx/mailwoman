//! Public build facts, with no deployment paths, configuration or credentials.
use crate::AppState;
use axum::{
    Router,
    extract::State,
    http::header,
    response::{IntoResponse, Response},
    routing::get,
};
pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/product-security", get(metadata))
        .route("/api/product-security/sbom", get(sbom))
        .route("/api/product-security/policy", get(policy))
}
async fn metadata(State(state): State<AppState>) -> Response {
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        state.product_build_info.metadata(),
    )
        .into_response()
}
async fn sbom(State(state): State<AppState>) -> Response {
    (
        [
            (header::CONTENT_TYPE, "application/vnd.cyclonedx+json"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=mailwoman-workspace.cdx.json",
            ),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        state.product_build_info.sbom(),
    )
        .into_response()
}
async fn policy(State(state): State<AppState>) -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=SECURITY.md",
            ),
        ],
        state.product_build_info.policy(),
    )
        .into_response()
}
