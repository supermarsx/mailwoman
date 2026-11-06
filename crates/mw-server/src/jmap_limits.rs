//! Concurrency ceilings of the native JMAP endpoints.
//!
//! Each signed-in account may run `ratelimit.jmap.perAccountConcurrent` requests
//! at once (the value the session resource advertises as `maxConcurrentRequests`),
//! and the process `ratelimit.jmap.globalConcurrent` across every account. A
//! request over either **waits** up to `ratelimit.queueMs` for a slot and only
//! then answers `429` with `Retry-After` and the `RateLimit-*` fields (see
//! `rate_leeway`). Before t106 this was one process-wide gate of four slots taken
//! with `try_acquire`, which every signed-in user shared and which refused the
//! fifth request at once.
//!
//! Permits cover upstream connection, execution and response construction.
use crate::{
    AppState,
    rate_leeway::{self, Busy, Gate, Held},
};
use axum::{
    Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use std::sync::OnceLock;

fn requests() -> &'static Gate {
    static G: OnceLock<Gate> = OnceLock::new();
    G.get_or_init(|| Gate::new("jmap"))
}
fn uploads() -> &'static Gate {
    static G: OnceLock<Gate> = OnceLock::new();
    G.get_or_init(|| Gate::new("jmapUpload"))
}
fn exports() -> &'static Gate {
    static G: OnceLock<Gate> = OnceLock::new();
    G.get_or_init(|| Gate::new("jmapExport"))
}
pub(crate) fn problem(mut value: serde_json::Value, status: StatusCode) -> Response {
    value["status"] = serde_json::json!(status.as_u16());
    (
        status,
        [(header::CONTENT_TYPE, "application/problem+json")],
        Json(value),
    )
        .into_response()
}
fn busy(busy: Busy, limit: &str, class: &'static str) -> Response {
    rate_leeway::tag(
        problem(
            mw_engine::jmap::core::problem("limit", Some(limit)),
            StatusCode::TOO_MANY_REQUESTS,
        ),
        busy.refused(class),
    )
}
/// A slot for one JMAP API request of `username`'s account.
pub(crate) async fn request(
    state: &AppState,
    account_id: &str,
    username: &str,
) -> Result<Held, Response> {
    let knobs = rate_leeway::knobs(&state.store, rate_leeway::domain_of(username)).await;
    requests()
        .acquire(account_id, knobs.jmap_per_account, knobs.jmap_global, knobs.queue())
        .await
        .map_err(|b| busy(b, "maxConcurrentRequests", "jmap"))
}
/// A slot for one folder export. An export reads a whole folder and holds its
/// slot until the last byte is sent, so an account runs one at a time (never
/// more than a quarter of its request allowance) and the process a few.
pub(crate) async fn export(
    state: &AppState,
    account_id: &str,
    username: &str,
) -> Result<Held, Response> {
    let knobs = rate_leeway::knobs(&state.store, rate_leeway::domain_of(username)).await;
    exports()
        .acquire(
            account_id,
            (knobs.jmap_per_account / 4).clamp(1, 2),
            (knobs.jmap_global / 4).max(2),
            knobs.queue(),
        )
        .await
        .map_err(|b| busy(b, "maxConcurrentExport", "jmapExport"))
}
/// A slot for one blob upload. Uploads are heavier, so an account keeps half the
/// request allowance, never below the four the session resource advertises.
pub(crate) async fn upload(
    state: &AppState,
    account_id: &str,
    username: &str,
) -> Result<Held, Response> {
    let knobs = rate_leeway::knobs(&state.store, rate_leeway::domain_of(username)).await;
    uploads()
        .acquire(
            account_id,
            (knobs.jmap_per_account / 2).max(4),
            (knobs.jmap_global / 2).max(4),
            knobs.queue(),
        )
        .await
        .map_err(|b| busy(b, "maxConcurrentUpload", "jmapUpload"))
}
