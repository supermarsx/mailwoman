//! Outbound webhook dispatch (SPEC §20.2, plan §3 e9). SCAFFOLD (t6-e0): stub
//! handler returning `501`, declared as a `mod` in `lib.rs` but NOT mounted. e9
//! fills HMAC-SHA256-signed outbound webhooks (retry + backoff) as a SECOND
//! consumer of the engine `StateChange` broadcast, plus the inbound webhook-rule
//! action endpoints; e11 mounts them. Secrets are sealed at rest (`webhooks`
//! table, 0007).

use axum::http::StatusCode;

/// Webhook management/delivery endpoints. STUB: `501` until e9 fills + e11 mounts.
pub async fn webhooks_stub() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
