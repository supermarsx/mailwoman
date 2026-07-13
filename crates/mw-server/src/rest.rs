//! REST convenience layer over JMAP (SPEC §20.1, plan §3 e9). SCAFFOLD (t6-e0):
//! stub handler returning `501`, declared as a `mod` in `lib.rs` but NOT mounted.
//! e9 fills the thin `/api/v1/...` REST surface generated over the existing JMAP
//! surface (identical results); e11 mounts it behind the `require_scope`
//! enforcement middleware.

use axum::http::StatusCode;

/// `/api/v1/*` — the REST convenience layer over JMAP. STUB: `501` until e9 fills
/// + e11 mounts.
pub async fn rest_stub() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
