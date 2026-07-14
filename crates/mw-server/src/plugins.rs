//! Plugin-registry admin routes (plan §3 e9/e14, SPEC §22). Filled by e9; MOUNTED by
//! e14 (`router()` byte-unchanged until then).
//!
//! `/admin/plugins/*` — list / approve / enable / disable / capability-grant over the
//! `mw-plugin` host registry. **Admin-session-gated** (the `mw_admin_session` cookie,
//! same domain as `/admin/*`); when the admin panel is disabled every route is `401`.
//!
//! Two security rules are enforced here at the route level:
//!   * **Unsigned refusal** — enabling an unsigned component requires an explicit
//!     `allowUnsigned` flag ([`unsigned_allowed`]); otherwise `403`. (The signature is
//!     re-verified against the component bytes at load time by `mw-plugin`; this route
//!     gates the *policy* so an unsigned plugin can never be enabled by accident.)
//!   * **Deny-by-default grants** — a capability grant is intersected with the
//!     manifest's *declared* capabilities ([`effective_grant`]); a capability the
//!     manifest never declared can never be granted.
//!
//! ## Injection (e14)
//! The `mw-plugin` [`mw_plugin::PluginHost`] (its registry seeded from the 0008
//! `plugins` rows) is injected as a shared, lockable request extension
//! ([`PluginRegistry`]). Persisting an approve/enable/grant back to 0008
//! `plugins`/`plugin_grants` is e14's wiring; this route drives the in-process host.
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use axum::extract::{Extension, Path as UrlPath, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use mw_plugin::{Capability, PluginError, PluginHost};

use crate::AppState;

/// The shared, lockable plugin host e14 injects.
pub(crate) type PluginRegistry = Arc<Mutex<PluginHost>>;

const ADMIN_COOKIE: &str = "mw_admin_session";

/// e14 merges this into `router()` and layers on the injected [`PluginRegistry`].
pub(crate) fn plugins_router() -> Router<AppState> {
    Router::new()
        .route("/admin/plugins", get(list))
        .route("/admin/plugins/{id}/approve", post(approve))
        .route("/admin/plugins/{id}/enable", post(enable))
        .route("/admin/plugins/{id}/disable", post(disable))
        .route("/admin/plugins/{id}/grant", post(grant))
}

// ── admin session gate (mirrors admin.rs; a user-facing mailbox session must not
// reach the plugin registry) ──────────────────────────────────────────────────

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "admin authentication required" })),
    )
        .into_response()
}

fn admin_cookie(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in raw.split(';') {
        if let Some(v) = part.trim().strip_prefix(&format!("{ADMIN_COOKIE}="))
            && !v.is_empty()
        {
            return Some(v.to_string());
        }
    }
    None
}

/// Resolve the authenticated admin id, or a `401`. Enforces the `admin.enabled` gate.
async fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<String, Response> {
    if !state.v6.admin_enabled {
        return Err(unauthorized());
    }
    let token = admin_cookie(headers).ok_or_else(unauthorized)?;
    let hash = crate::push_relay::hash_token(&token);
    match state.store.get_admin_session(&hash).await {
        Ok(Some(admin_id)) => Ok(admin_id),
        _ => Err(unauthorized()),
    }
}

// ── handlers ───────────────────────────────────────────────────────────────────

/// `GET /admin/plugins` — the registry (id/name/version/enabled/approved/signed +
/// declared capabilities). Any unsigned enabled plugin carries `signed=false` so the
/// UI can raise the persistent banner (§7.5).
async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    let host = reg.lock().expect("plugin registry lock");
    let rows: Vec<_> = host
        .list()
        .iter()
        .map(|e| {
            json!({
                "id": e.manifest.id,
                "name": e.manifest.name,
                "version": e.manifest.version,
                "enabled": e.enabled,
                "approved": e.approved_by.is_some(),
                "signed": e.manifest.signature.is_some(),
                "capabilities": e.manifest.capabilities,
                "netAllowlist": e.manifest.net_allowlist,
            })
        })
        .collect();
    Json(json!({ "plugins": rows })).into_response()
}

/// `POST /admin/plugins/{id}/approve` — admin-approve a registered plugin.
async fn approve(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let admin = match require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    // Mutate the in-process host, then (guard dropped) persist to 0008 (e14).
    let result = {
        let mut host = reg.lock().expect("plugin registry lock");
        host.approve(&id, &admin)
    };
    match result {
        Ok(()) => {
            let _ = state.store.set_plugin_approved(&id, &admin).await;
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => plugin_error(&e),
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnableQuery {
    /// Permit enabling an UNSIGNED component (⇒ the UI shows a persistent banner).
    #[serde(default)]
    allow_unsigned: bool,
}

/// `POST /admin/plugins/{id}/enable?allowUnsigned=` — enable an approved plugin. An
/// unsigned component is refused unless `allowUnsigned` is set (signed-registry policy).
async fn enable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
    UrlPath(id): UrlPath<String>,
    Query(q): Query<EnableQuery>,
) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    let allow_unsigned = q.allow_unsigned;
    let result = {
        let mut host = reg.lock().expect("plugin registry lock");

        // Signed-registry policy: refuse an unsigned plugin without the explicit flag.
        let signed = host
            .list()
            .iter()
            .find(|e| e.manifest.id == id)
            .map(|e| e.manifest.signature.is_some());
        match signed {
            None => {
                return plugin_error(&PluginError::Manifest(format!("unknown plugin '{id}'")));
            }
            Some(is_signed) if !unsigned_allowed(is_signed, allow_unsigned) => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "unsigned plugin requires allowUnsigned",
                        "signed": false,
                    })),
                )
                    .into_response();
            }
            _ => {}
        }
        host.enable(&id)
    };

    match result {
        Ok(()) => {
            let _ = state.store.set_plugin_enabled(&id, true).await;
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => plugin_error(&e),
    }
}

/// `POST /admin/plugins/{id}/disable`.
async fn disable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
    UrlPath(id): UrlPath<String>,
) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    let result = {
        let mut host = reg.lock().expect("plugin registry lock");
        host.disable(&id)
    };
    match result {
        Ok(()) => {
            let _ = state.store.set_plugin_enabled(&id, false).await;
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => plugin_error(&e),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GrantReq {
    #[serde(default)]
    account_id: Option<String>,
    capabilities: Vec<Capability>,
}

/// `POST /admin/plugins/{id}/grant` — grant a subset of the plugin's capabilities.
/// Deny-by-default: any requested capability the manifest did NOT declare is dropped
/// (returned in `denied`) and never granted. e14 persists the effective grant to the
/// 0008 `plugin_grants` table.
async fn grant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
    UrlPath(id): UrlPath<String>,
    Json(body): Json<GrantReq>,
) -> Response {
    let admin = match require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let computed = {
        let host = reg.lock().expect("plugin registry lock");
        let Some(entry) = host.list().iter().find(|e| e.manifest.id == id) else {
            return plugin_error(&PluginError::Manifest(format!("unknown plugin '{id}'")));
        };
        effective_grant(&body.capabilities, &entry.manifest.capabilities)
    };
    let (granted, denied) = computed;
    // Persist each effective (deny-by-default) grant to 0008 `plugin_grants` (e14).
    let account_id = body.account_id.clone().unwrap_or_default();
    for cap in &granted {
        let capability = serde_json::to_value(cap)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        let _ = state
            .store
            .put_plugin_grant(&mw_store::PluginGrantRow {
                plugin_id: id.clone(),
                account_id: account_id.clone(),
                capability,
                granted_by: admin.clone(),
                created_at: chrono::Utc::now().to_rfc3339(),
            })
            .await;
    }
    Json(json!({
        "pluginId": id,
        "accountId": body.account_id,
        "granted": granted,
        "denied": denied,
    }))
    .into_response()
}

// ── pure policy helpers (unit-tested) ────────────────────────────────────────

/// Whether a plugin may be enabled given whether it is signed + the admin flag: a
/// signed plugin always may; an unsigned plugin only with `allow_unsigned`.
fn unsigned_allowed(signed: bool, allow_unsigned: bool) -> bool {
    signed || allow_unsigned
}

/// Intersect requested capabilities with those the manifest declared. Returns
/// `(granted, denied)` — deny-by-default: a capability not declared is denied.
fn effective_grant(
    requested: &[Capability],
    declared: &[Capability],
) -> (Vec<Capability>, Vec<Capability>) {
    let mut granted = Vec::new();
    let mut denied = Vec::new();
    for c in requested {
        if declared.contains(c) {
            granted.push(*c);
        } else {
            denied.push(*c);
        }
    }
    (granted, denied)
}

/// Map a [`PluginError`] to an HTTP response.
fn plugin_error(e: &PluginError) -> Response {
    let (code, msg) = match e {
        PluginError::Manifest(m) => (StatusCode::NOT_FOUND, m.clone()),
        PluginError::CapabilityDenied(m) => (StatusCode::BAD_REQUEST, m.clone()),
        PluginError::SignatureInvalid(m) => (StatusCode::FORBIDDEN, m.clone()),
        _ => (StatusCode::BAD_REQUEST, e.to_string()),
    };
    (code, Json(json!({ "error": msg }))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_plugin::{Grant, PluginManifest};

    fn manifest(id: &str, signed: bool, caps: Vec<Capability>) -> PluginManifest {
        PluginManifest {
            id: id.into(),
            name: id.into(),
            version: "1".into(),
            signature: signed.then(|| "deadbeef".to_string()),
            capabilities: caps,
            net_allowlist: vec![],
            limits: Default::default(),
        }
    }

    #[test]
    fn unsigned_is_refused_without_the_flag() {
        assert!(unsigned_allowed(true, false), "signed always allowed");
        assert!(
            !unsigned_allowed(false, false),
            "unsigned refused by default"
        );
        assert!(
            unsigned_allowed(false, true),
            "unsigned allowed with the flag"
        );
    }

    #[test]
    fn grant_is_deny_by_default() {
        // Manifest declares only AccountBackend; a request for Net is denied.
        let declared = vec![Capability::AccountBackend];
        let requested = vec![Capability::AccountBackend, Capability::Net];
        let (granted, denied) = effective_grant(&requested, &declared);
        assert_eq!(granted, vec![Capability::AccountBackend]);
        assert_eq!(denied, vec![Capability::Net]);
    }

    #[test]
    fn approve_then_enable_registry_flow() {
        // The host itself refuses to enable before approval, and an unsigned entry is
        // gated by the route policy above (unsigned_allowed).
        let mut host = PluginHost::new();
        host.register(manifest("lt", false, vec![]));
        assert!(host.enable("lt").is_err(), "must approve first");
        host.approve("lt", "admin@x").unwrap();
        host.enable("lt").unwrap();
        assert!(host.list()[0].enabled);
        // Unused import guard: Grant is part of the mount contract e14 uses.
        let _ = Grant {
            plugin_id: "lt".into(),
            capabilities: vec![],
            granted_by: "admin@x".into(),
            allow_unsigned: true,
        };
    }
}
