//! V5 push relay + native bearer-auth seams (plan §2.2/§2.3/§3 e0/e5).
//!
//! e0 reserves the surface; e5 fills it. Everything here is ADDITIVE and OFF by
//! default — the browser cookie/same-origin path is byte-identical, and the push
//! endpoints return a clean `501` until e5 wires them (same discipline as the V3
//! PIM / V4 crypto route seams in `lib.rs`).
//!
//! e5's deliverables that slot in here:
//!   * VAPID keygen on first boot + `GET /api/push/vapid` serving the PUBLIC key
//!     (private sealed at rest via `mw_store::Store::store_vapid_keypair`),
//!   * `POST /api/push/subscribe|unsubscribe` over `mw_store::PushSubscriptionRow`,
//!   * the native bearer-auth mode (`/api/login clientType:"native"` → a token +
//!     `native_sessions`; bearer-accept on the authed routes; CSRF-guard skip for
//!     bearer; the config-gated CORS/origin allowlist),
//!   * the push DISPATCHER = a second consumer of the engine `StateChange`
//!     broadcast that sends OPAQUE wakes (no content) respecting quiet-hours/rules.

use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::State};
use serde_json::json;

use crate::AppState;

/// The Tauri shell origins the CORS/origin allowlist opts into when configured
/// (plan §2.2). e5 threads a config-gated allowlist (env `MW_NATIVE_ORIGINS`,
/// default EMPTY → off) into the middleware; until then no CORS headers are emitted
/// and browser deployments see no behavior change.
#[derive(Debug, Clone, Default)]
pub struct NativeAuthConfig {
    /// Allowed shell origins (e.g. `tauri://localhost`, `https://tauri.localhost`).
    /// Empty = the native/CORS mode is OFF (the default).
    pub origins: Vec<String>,
}

impl NativeAuthConfig {
    /// Populate from `MW_NATIVE_ORIGINS` (comma-separated). Absent/empty → off.
    pub fn from_env() -> Self {
        let origins = std::env::var("MW_NATIVE_ORIGINS")
            .ok()
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.split(',')
                    .map(|o| o.trim().to_string())
                    .filter(|o| !o.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        Self { origins }
    }

    /// Whether the native/CORS mode is enabled (any origin configured).
    pub fn is_enabled(&self) -> bool {
        !self.origins.is_empty()
    }
}

/// Extract a `Authorization: Bearer <token>` value, if present. The seam the e5
/// bearer-accept path uses (in addition to the cookie) on the authed routes; bearer
/// requests skip the cookie-only CSRF guard (no ambient authority to protect).
pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = raw
        .strip_prefix("Bearer ")
        .or_else(|| raw.strip_prefix("bearer "))?;
    let token = token.trim();
    (!token.is_empty()).then(|| token.to_string())
}

/// A clean `501` for a push endpoint e5 has not filled yet (never falls through to
/// the SPA `index.html`).
fn not_implemented(feature: &str) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({ "error": format!("{feature} is not implemented in this build") })),
    )
        .into_response()
}

/// `GET /api/push/vapid` → `{ publicKey }` (public-only). e5 serves the persisted
/// VAPID public key so the browser can subscribe in-page. 501 until then.
pub(crate) async fn push_vapid(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    not_implemented("push VAPID key")
}

/// `POST /api/push/subscribe` (body = `PushSubscriptionInfo`) → `{ id, vapidPublicKey }`.
/// e5 stores the subscription (`push_subscriptions`) idempotently. 501 until then.
pub(crate) async fn push_subscribe(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    not_implemented("push subscribe")
}

/// `POST /api/push/unsubscribe {id|endpoint}`. e5 removes the subscription. 501 now.
pub(crate) async fn push_unsubscribe(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    not_implemented("push unsubscribe")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn bearer_token_parses_case_insensitively() {
        let mut h = HeaderMap::new();
        h.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer abc123"),
        );
        assert_eq!(bearer_token(&h).as_deref(), Some("abc123"));
        h.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("bearer  xyz "),
        );
        assert_eq!(bearer_token(&h).as_deref(), Some("xyz"));
    }

    #[test]
    fn bearer_token_absent_without_header() {
        assert!(bearer_token(&HeaderMap::new()).is_none());
    }

    #[test]
    fn native_auth_off_by_default() {
        assert!(!NativeAuthConfig::default().is_enabled());
    }

    #[test]
    fn native_auth_parses_origins() {
        let cfg = NativeAuthConfig {
            origins: vec!["tauri://localhost".into()],
        };
        assert!(cfg.is_enabled());
    }
}
