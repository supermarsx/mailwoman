//! Observability surface (SPEC §21, plan §3 e9). SCAFFOLD (t6-e0): stub handler
//! returning `501`, declared as a `mod` in `lib.rs` but NOT mounted. e9 fills
//! `tracing` per-subsystem hot-reload (SIGHUP/admin), OTLP traces+metrics export,
//! and the auth-gated Prometheus `/metrics` endpoint; e11 mounts them. The
//! `/errors` scrubber tunnel is its sibling module [`crate::errors`]. No mail
//! body/subject/address enters any log (typed-wrapper assertion, §21.1).

use axum::http::StatusCode;

/// Auth-gated Prometheus `/metrics`. STUB: `501` until e9 fills + e11 mounts.
pub async fn metrics_stub() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
