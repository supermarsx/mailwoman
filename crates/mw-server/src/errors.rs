//! Server-side browser-error scrubber tunnel (SPEC §21, plan §3 e9). SCAFFOLD
//! (t6-e0): stub handler returning `501`, declared as a `mod` in `lib.rs` but NOT
//! mounted. e9 fills `/errors` — it scrubs mail content/addresses out of
//! browser-reported errors before forwarding, so the CSP can stay
//! `connect-src 'self'` (no third-party error sink); e11 mounts it.

use axum::http::StatusCode;

/// `/errors` — accept + scrub + forward browser error reports. STUB: `501` until
/// e9 fills + e11 mounts.
pub async fn errors_stub() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
