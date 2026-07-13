//! OAuth 2.1 authorization-server + scoped-API-key HTTP surface (SPEC §20.1, plan
//! §2.3, §3 e11 mount). SCAFFOLD (t6-e0): stub handlers returning `501`, declared
//! as a `mod` in `lib.rs` but NOT mounted. e11 mounts `/oauth/authorize` (code +
//! PKCE-S256 + resource), `/oauth/token`, `/oauth/introspect`, `/oauth/revoke`
//! against `mw-oauth`, and wires the `require_scope` enforcement middleware
//! (resolve key/token → `Scope`, check IP + rate-limit + expiry, emit an audit row).

use axum::http::StatusCode;

/// `/oauth/*` endpoints (authorize/token/introspect/revoke). STUB: `501` until
/// e11 mounts them against the `mw-oauth` authorization server.
pub async fn oauth_stub() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
