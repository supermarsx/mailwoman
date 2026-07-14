//! Scoped-API-key enforcement middleware (SPEC §20.1/§2.4, t6-e11b).
//!
//! e11 mounted the V6 routes but left `/api/v1/*` **cookie-only** and per-key
//! IP-allowlist / rate-limit unenforced, because `mw_oauth::require_scope` borrows a
//! `&dyn AuditSink` across an await and so is not `Send` — it cannot sit in an axum
//! handler. This module closes that gap with a **`Send`** guard built on the
//! additive [`mw_oauth::AuthServer::require_scope_send`] (owned `Arc` sink, no
//! borrowed trait object across an await).
//!
//! Two mounts. [`rest_scope_guard`] in front of `/api/v1/*` resolves a presented
//! scoped key (`mwk_…` via `Authorization: Bearer` or the `x-api-key` header) to its
//! [`Scope`] and enforces it — valid+unrevoked, expiry, IP-allowlist, per-key
//! rate-limit, and `Scope::allows(required)` for the route (GET→read, mail vs PIM
//! surface, the account selector). The cookie/native path is untouched: no key
//! present → pass straight through to the existing `authed` cookie session; a key
//! **plus** a cookie downscopes that browser session to the key's grant.
//!
//! [`mcp_scope_guard`] in front of `/mcp` adds IP-allowlist + per-key rate-limit +
//! expiry (the parts `mw_mcp::OAuthAuthorizer` deliberately leaves out); the inline
//! per-tool `Scope::allows` + countersign check e11 wired stays in place. An
//! unauthenticated MCP call (`initialize`/`tools/list`) passes through.
//!
//! Source IP is taken from the trusted reverse-proxy header (`X-Forwarded-For`
//! first hop, or `Forwarded: for=`), falling back to `ConnectInfo<SocketAddr>` when
//! the serve path wired it — the standard "TLS terminates at a front proxy" model.

use std::net::{IpAddr, SocketAddr};

use axum::Json;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use mw_oauth::{OAuthError, RequestContext, Scope, ScopeSelector};

use crate::AppState;
use crate::stores_v6::AdminOAuthAudit;

/// The `x-api-key` header carrying a scoped key for clients that keep
/// `Authorization` for something else.
const KEY_HEADER: &str = "x-api-key";

/// Where in the request the scoped key was presented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyLocation {
    /// `x-api-key: mwk_…` — leaves `Authorization` free for a cookie downscope.
    Header,
    /// `Authorization: Bearer mwk_…` — a headless key-only client.
    Bearer,
}

/// A scoped key extracted from the request.
struct PresentedKey {
    token: String,
    location: KeyLocation,
}

/// How a `/api/v1` handler should obtain its account [`mw_store::Session`] after the
/// guard authorized the request. Inserted as a request extension by
/// [`rest_scope_guard`]; consumed by [`rest_session`].
#[derive(Debug, Clone)]
pub(crate) enum RestSessionSource {
    /// No scoped key (or a key downscoping a cookie session): use the existing
    /// cookie/native `authed` path — byte-identical to the pre-e11b behaviour.
    Cookie,
    /// A key-only request authorized for `account_id`: synthesize a session (the
    /// key is the account authority; upstream data reads use the engine account).
    Key { account_id: String },
}

/// Resolve the account session for a `/api/v1` request that passed [`rest_scope_guard`].
pub(crate) async fn rest_session(
    state: &AppState,
    headers: &HeaderMap,
    source: RestSessionSource,
) -> Result<mw_store::Session, Response> {
    match source {
        RestSessionSource::Cookie => crate::authed(state, headers).await,
        RestSessionSource::Key { account_id } => Ok(mw_store::Session {
            id: String::new(),
            account_id: account_id.clone(),
            username: account_id,
            jmap_url: String::new(),
            api_url: String::new(),
            credentials: mw_store::Credentials {
                username: String::new(),
                password: String::new(),
            },
        }),
    }
}

/// `/api/v1/*` guard: enforce a presented scoped key, else pass the cookie/native
/// path through untouched. Always inserts a [`RestSessionSource`] extension so the
/// handlers can resolve their session.
pub(crate) async fn rest_scope_guard(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let headers = req.headers().clone();
    let Some(key) = extract_api_key(&headers) else {
        // No scoped key → the browser cookie / native bearer path, unchanged.
        req.extensions_mut().insert(RestSessionSource::Cookie);
        return next.run(req).await;
    };

    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let source_ip = client_ip(&headers, req.extensions());

    // A key in `x-api-key` alongside a cookie downscopes that browser session: the
    // upstream data read uses the cookie account, so the key must cover it. A
    // `Authorization: Bearer mwk_…` key is always key-only (it would otherwise clash
    // with the native-bearer session path).
    let cookie_account = if key.location == KeyLocation::Header {
        match crate::cookie_value(&headers) {
            Some(id) => (state.store.get_session(&id).await.ok()).map(|s| s.account_id),
            None => None,
        }
    } else {
        None
    };

    let required_accounts = match &cookie_account {
        // Downscope: the key must cover the cookie session's account.
        Some(acct) => ScopeSelector::Subset(vec![acct.clone()]),
        // Key-only: the key IS the account authority — impose no account constraint.
        None => ScopeSelector::Subset(vec![]),
    };
    let required = rest_required_scope(&method, &path, required_accounts);

    let audit = AdminOAuthAudit::new(state.v6.admin.clone());
    let ctx = RequestContext {
        credential: &key.token,
        source_ip,
        resource: None,
    };
    match state
        .v6
        .auth
        .require_scope_send(&ctx, &required, &*audit)
        .await
    {
        Ok(granted) => {
            let source = if cookie_account.is_some() {
                RestSessionSource::Cookie
            } else {
                RestSessionSource::Key {
                    account_id: granted.account_id,
                }
            };
            req.extensions_mut().insert(source);
            next.run(req).await
        }
        Err(e) => scope_error_response(&e),
    }
}

/// `/mcp` guard: enforce IP-allowlist + per-key rate-limit + expiry for a presented
/// key (the parts `OAuthAuthorizer` leaves to the middleware); the per-tool scope +
/// countersign check stays inline in `mw_mcp`. Unauthenticated calls pass through.
pub(crate) async fn mcp_scope_guard(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let headers = req.headers().clone();
    let Some(key) = extract_api_key(&headers) else {
        // Unauthenticated MCP (`initialize`/`tools/list`) is unchanged.
        return next.run(req).await;
    };
    let source_ip = client_ip(&headers, req.extensions());
    let audit = AdminOAuthAudit::new(state.v6.admin.clone());
    let ctx = RequestContext {
        credential: &key.token,
        source_ip,
        resource: None,
    };
    // `nothing_required` imposes no capability check here (that is the per-tool
    // authorizer's job); require_scope_send still enforces expiry + IP + rate-limit.
    match state
        .v6
        .auth
        .require_scope_send(&ctx, &nothing_required(), &*audit)
        .await
    {
        Ok(_) => next.run(req).await,
        Err(e) => scope_error_response(&e),
    }
}

/// Extract a presented `mwk_…` key from `x-api-key` (preferred) or a `Bearer`
/// `Authorization`. A non-`mwk_` bearer (a native session token) is NOT a key, so
/// the native-bearer path is left to `authed` untouched.
fn extract_api_key(headers: &HeaderMap) -> Option<PresentedKey> {
    if let Some(v) = headers.get(KEY_HEADER).and_then(|v| v.to_str().ok()) {
        let t = v.trim();
        if t.starts_with("mwk_") {
            return Some(PresentedKey {
                token: t.to_string(),
                location: KeyLocation::Header,
            });
        }
    }
    if let Some(v) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        && let Some(t) = v.strip_prefix("Bearer ").map(str::trim)
        && t.starts_with("mwk_")
    {
        return Some(PresentedKey {
            token: t.to_string(),
            location: KeyLocation::Bearer,
        });
    }
    None
}

/// Derive the client source IP: the trusted reverse-proxy header first
/// (`X-Forwarded-For` first hop, then `Forwarded: for=`), falling back to the
/// direct-connection `ConnectInfo<SocketAddr>` when the serve path supplied it.
fn client_ip(headers: &HeaderMap, ext: &axum::http::Extensions) -> Option<IpAddr> {
    if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok())
        && let Some(first) = xff.split(',').next()
        && let Ok(ip) = first.trim().parse::<IpAddr>()
    {
        return Some(ip);
    }
    if let Some(fwd) = headers.get("forwarded").and_then(|v| v.to_str().ok()) {
        for part in fwd.split(&[';', ','][..]) {
            let part = part.trim();
            if let Some(v) = part
                .strip_prefix("for=")
                .or_else(|| part.strip_prefix("For="))
            {
                let v = v.trim_matches('"');
                // `for="[2001:db8::1]:443"` (bracketed IPv6) or `for=1.2.3.4:5678`.
                let candidate = if let Some(inner) = v.strip_prefix('[') {
                    inner.split(']').next().unwrap_or(inner)
                } else {
                    v.rsplit_once(':').map(|(h, _)| h).unwrap_or(v)
                };
                if let Ok(ip) = candidate.parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }
    ext.get::<ConnectInfo<SocketAddr>>().map(|ci| ci.0.ip())
}

/// The [`Scope`] a `/api/v1` route requires: verb from the method (GET→read,
/// DELETE→delete, else send), surface from the path (mail vs PIM), and the account
/// selector the caller supplies (the cookie account for a downscope, or empty for a
/// key-only request where the key is itself the account authority).
fn rest_required_scope(method: &Method, path: &str, accounts: ScopeSelector) -> Scope {
    let is_pim = ["/calendar", "/contacts", "/tasks", "/notes"]
        .iter()
        .any(|p| path.contains(p));
    let (read, send, delete) = match *method {
        Method::GET | Method::HEAD => (true, false, false),
        Method::DELETE => (false, false, true),
        _ => (false, true, false),
    };
    Scope {
        read,
        send,
        delete,
        accounts,
        // REST list/read is not folder-scoped per request; no folder constraint.
        folders: ScopeSelector::Subset(Vec::new()),
        mail: !is_pim,
        pim: is_pim,
        ip_allowlist: Vec::new(),
        expires_at: None,
        rate_limit: None,
        mcp_tools: Vec::new(),
        unattended_send: false,
    }
}

/// A scope that requires nothing — used by the `/mcp` guard so `require_scope_send`
/// runs only its expiry/IP/rate-limit checks (capability is the per-tool authorizer).
fn nothing_required() -> Scope {
    Scope {
        read: false,
        send: false,
        delete: false,
        accounts: ScopeSelector::Subset(Vec::new()),
        folders: ScopeSelector::Subset(Vec::new()),
        mail: false,
        pim: false,
        ip_allowlist: Vec::new(),
        expires_at: None,
        rate_limit: None,
        mcp_tools: Vec::new(),
        unattended_send: false,
    }
}

/// Map an enforcement failure to its HTTP status (never leak which check failed
/// beyond the coarse class): expiry→401, IP→403, rate→429, scope→403, bad key→401.
fn scope_error_response(e: &OAuthError) -> Response {
    let (code, msg): (StatusCode, &str) = match e {
        OAuthError::Expired => (StatusCode::UNAUTHORIZED, "credential expired or revoked"),
        OAuthError::IpDenied => (StatusCode::FORBIDDEN, "source ip not permitted"),
        OAuthError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate limit exceeded"),
        OAuthError::InvalidScope => (StatusCode::FORBIDDEN, "scope does not permit this request"),
        OAuthError::InvalidGrant => (StatusCode::UNAUTHORIZED, "invalid api key"),
        _ => (StatusCode::UNAUTHORIZED, "unauthorized"),
    };
    (code, Json(json!({ "error": msg }))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hdrs(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.insert(
                axum::http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                v.parse().unwrap(),
            );
        }
        h
    }

    #[test]
    fn extracts_key_from_header_and_bearer() {
        let h = hdrs(&[("x-api-key", "mwk_abc.def")]);
        let k = extract_api_key(&h).unwrap();
        assert_eq!(k.token, "mwk_abc.def");
        assert_eq!(k.location, KeyLocation::Header);

        let h = hdrs(&[("authorization", "Bearer mwk_zzz.yyy")]);
        let k = extract_api_key(&h).unwrap();
        assert_eq!(k.location, KeyLocation::Bearer);

        // A non-mwk bearer (native session token) is not a key.
        let h = hdrs(&[("authorization", "Bearer plain-session-id")]);
        assert!(extract_api_key(&h).is_none());
        assert!(extract_api_key(&HeaderMap::new()).is_none());
    }

    #[test]
    fn client_ip_prefers_forwarded_headers() {
        let ext = axum::http::Extensions::new();
        let h = hdrs(&[("x-forwarded-for", "8.8.8.8, 10.0.0.1")]);
        assert_eq!(
            client_ip(&h, &ext).unwrap(),
            "8.8.8.8".parse::<IpAddr>().unwrap()
        );

        let h = hdrs(&[("forwarded", "for=1.2.3.4:5678;proto=https")]);
        assert_eq!(
            client_ip(&h, &ext).unwrap(),
            "1.2.3.4".parse::<IpAddr>().unwrap()
        );

        let h = hdrs(&[("forwarded", "for=\"[2001:db8::1]:443\"")]);
        assert_eq!(
            client_ip(&h, &ext).unwrap(),
            "2001:db8::1".parse::<IpAddr>().unwrap()
        );

        assert!(client_ip(&HeaderMap::new(), &ext).is_none());
    }

    #[test]
    fn required_scope_maps_verb_and_surface() {
        let s = rest_required_scope(&Method::GET, "/api/v1/messages", ScopeSelector::All);
        assert!(s.read && s.mail && !s.pim && !s.send);
        let s = rest_required_scope(&Method::GET, "/api/v1/calendar/x", ScopeSelector::All);
        assert!(s.read && s.pim && !s.mail);
        let s = rest_required_scope(&Method::DELETE, "/api/v1/messages/1", ScopeSelector::All);
        assert!(s.delete && !s.read);
    }
}
