//! Public build facts, with no deployment paths, configuration or credentials.
use crate::AppState;
use axum::{
    Router,
    http::header,
    response::{IntoResponse, Response},
    routing::get,
};
const METADATA: &str = include_str!(concat!(env!("OUT_DIR"), "/product-security.json"));
const SBOM: &str = include_str!(concat!(env!("OUT_DIR"), "/software-bom.cdx.json"));
const POLICY: &str = include_str!("../../../SECURITY.md");
pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/product-security", get(metadata))
        .route("/api/product-security/sbom", get(sbom))
        .route("/api/product-security/policy", get(policy))
}
async fn metadata() -> Response {
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        METADATA,
    )
        .into_response()
}
async fn sbom() -> Response {
    (
        [
            (header::CONTENT_TYPE, "application/vnd.cyclonedx+json"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=mailwoman-workspace.cdx.json",
            ),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        SBOM,
    )
        .into_response()
}
async fn policy() -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=SECURITY.md",
            ),
        ],
        POLICY,
    )
        .into_response()
}
