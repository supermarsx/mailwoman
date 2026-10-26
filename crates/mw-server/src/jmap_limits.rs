//! Conservative process-wide ceilings also bound every individual account.
//! Permits cover upstream connection, execution and response construction.
use axum::{
    Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use tokio::sync::{Semaphore, SemaphorePermit};
static REQUESTS: Semaphore = Semaphore::const_new(4);
static UPLOADS: Semaphore = Semaphore::const_new(4);
pub(crate) fn problem(mut value: serde_json::Value, status: StatusCode) -> Response {
    value["status"] = serde_json::json!(status.as_u16());
    (
        status,
        [(header::CONTENT_TYPE, "application/problem+json")],
        Json(value),
    )
        .into_response()
}
fn acquire(gate: &'static Semaphore, limit: &str) -> Result<SemaphorePermit<'static>, Response> {
    gate.try_acquire().map_err(|_| {
        problem(
            mw_engine::jmap::core::problem("limit", Some(limit)),
            StatusCode::TOO_MANY_REQUESTS,
        )
    })
}
pub(crate) fn request() -> Result<SemaphorePermit<'static>, Response> {
    acquire(&REQUESTS, "maxConcurrentRequests")
}
pub(crate) fn upload() -> Result<SemaphorePermit<'static>, Response> {
    acquire(&UPLOADS, "maxConcurrentUpload")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn permits_enforce_the_advertised_ceiling_and_release_on_cancel() {
        let gate = Box::leak(Box::new(Semaphore::new(4)));
        let permits: Vec<_> = (0..4)
            .map(|_| acquire(gate, "maxConcurrentRequests").unwrap())
            .collect();
        assert_eq!(
            acquire(gate, "maxConcurrentRequests").unwrap_err().status(),
            StatusCode::TOO_MANY_REQUESTS
        );
        drop(permits);
        assert!(acquire(gate, "maxConcurrentRequests").is_ok());
    }
}
