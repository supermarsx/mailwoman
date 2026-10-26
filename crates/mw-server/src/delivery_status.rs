//! Bounded parsing for delivery reports retrieved through the authenticated blob proxy.
use crate::AppState;
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use base64::Engine as _;
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, OnceLock};

const MAX_BYTES: usize = 8 * 1024 * 1024;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Input {
    bytes_base64: String,
}

pub(crate) fn router() -> Router<AppState> {
    Router::new().route(
        "/api/mail/delivery-status",
        post(parse).layer(DefaultBodyLimit::max(12 * 1024 * 1024)),
    )
}
async fn parse(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<Input>,
) -> Response {
    if let Err(response) = crate::authed(&state, &headers).await {
        return response;
    }
    if input.bytes_base64.len() > MAX_BYTES.div_ceil(3) * 4 {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    static WORKERS: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
    let Ok(permit) = WORKERS
        .get_or_init(|| Arc::new(tokio::sync::Semaphore::new(2)))
        .clone()
        .try_acquire_owned()
    else {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    };
    let raw = match base64::engine::general_purpose::STANDARD.decode(input.bytes_base64) {
        Ok(raw) if !raw.is_empty() && raw.len() <= MAX_BYTES => raw,
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };
    let task = tokio::task::spawn_blocking(move || {
        // Retained inside the task: HTTP cancellation/timeout cannot release a
        // slot while bounded CPU work is still in progress.
        let _permit = permit;
        mw_engine::jmap::dsn_report_json(&raw)
    });
    let response = match tokio::time::timeout(std::time::Duration::from_secs(5), task).await {
        Ok(Ok(report)) => Json(json!({"report":report})).into_response(),
        _ => StatusCode::UNPROCESSABLE_ENTITY.into_response(),
    };
    (
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        response,
    )
        .into_response()
}
