//! Admin-panel HTTP surface (SPEC §19, plan §2.5, §3 e5). SCAFFOLD (t6-e0): stub
//! handlers returning `501`, declared as a `mod` in `lib.rs` but NOT mounted —
//! `router()` is byte-unchanged so behaviour is identical. e5 fills these against
//! `mw-admin` (a SEPARATE `mw_admin_session` domain, passkey-capable) and e11
//! mounts them under `/admin/*`. `admin.enabled=false` leaves the panel unmounted.

use axum::http::StatusCode;

/// `/admin/*` — the §19 panel endpoints (domains/users/security-policy/
/// integrations/observability/appearance). Every action writes the append-only
/// audit log. STUB: `501` until e5 fills + e11 mounts.
pub async fn admin_stub() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
