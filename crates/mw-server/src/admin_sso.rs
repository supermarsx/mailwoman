//! Admin SSO-config CRUD (t9-e3, plan §2 D4, SPEC §18.3/§19). Admin-session-gated
//! management of the 0009 `sso_config` backends, mirroring the `directory`/`plugins`
//! admin surfaces.
//!
//! `/admin/sso` — registered in `admin_router` (`sso.read` to list, `sso.write` to
//! change); without an admin session, or with the panel disabled, every route is `401`.
//!   * `GET /admin/sso?scope=` — list the backends in a scope (default `deployment`); the sealed secret is NEVER returned, only a `hasSecret` flag.
//!   * `POST /admin/sso` — create/update a backend; the secret is SEALED at the store (`put_sso_config`), and omitting it on an update preserves the existing sealed secret.
//!   * `DELETE /admin/sso/{id}` — delete a backend (idempotent).
//!
//! The store column scopes list-by-scope (the 0009 API is `list_sso_config(scope)`),
//! so the admin surface lists one scope at a time; the web panel iterates the domains
//! it manages.
#![allow(dead_code)]

use axum::Json;
use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;

use mw_sso::{ClaimMap, SsoConfig};

use crate::AppState;
use crate::admin_ctx::AdminCtx;
use crate::admin_router::AdminRegistrar;
use crate::capabilities::Capability;

/// The `/admin/sso` routes (registered by `admin_router`).
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/sso", Capability::SsoRead, list);
    r.admin_endpoint(Method::POST, "/sso", Capability::SsoWrite, upsert);
    r.admin_endpoint(Method::DELETE, "/sso/{id}", Capability::SsoWrite, remove);
}

// ── handlers ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ScopeQuery {
    /// The scope to list (`deployment` | `domain:<d>`). Defaults to `deployment`.
    #[serde(default)]
    scope: Option<String>,
}

/// `GET /admin/sso?scope=` — list the backends in a scope. Secrets are never
/// returned; a `hasSecret` flag says whether one is sealed.
async fn list(
    State(state): State<AppState>,
    _ctx: AdminCtx,
    Query(q): Query<ScopeQuery>,
) -> Response {
    let scope = q.scope.as_deref().unwrap_or("deployment");
    let rows = match state.store.list_sso_config(scope).await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("sso_config list failed: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response();
        }
    };
    let backends: Vec<_> = rows
        .into_iter()
        .map(|r| {
            // Re-parse the config so a client always sees the canonical shape; a
            // malformed row is surfaced as null config rather than dropped silently.
            let config = serde_json::from_str::<SsoConfig>(&r.config_json).ok();
            let claim_map = serde_json::from_str::<ClaimMap>(&r.claim_map_json).ok();
            json!({
                "id": r.id,
                "kind": r.kind,
                "displayName": r.display_name,
                "scope": r.scope,
                "enabled": r.enabled,
                "config": config,
                "claimMap": claim_map,
                "hasSecret": r.secret.is_some(),
                "createdAt": r.created_at,
                "updatedAt": r.updated_at,
            })
        })
        .collect();
    Json(json!({ "backends": backends })).into_response()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpsertReq {
    /// Stable id, e.g. `"corp-oidc"`.
    id: String,
    /// Admin-facing label.
    display_name: String,
    /// `deployment` | `domain:<d>` (defaults to `deployment`).
    #[serde(default)]
    scope: Option<String>,
    /// Whether the backend is live/advertised.
    #[serde(default)]
    enabled: bool,
    /// Kind-specific config (validated against [`SsoConfig`]; secrets NOT here).
    config: SsoConfig,
    /// Claim/attribute mapping.
    #[serde(default)]
    claim_map: ClaimMap,
    /// The OIDC client secret / SAML SP private key (PEM). Sealed at the store.
    /// Omit on an update to preserve the existing sealed secret.
    #[serde(default)]
    secret: Option<String>,
}

/// `POST /admin/sso` — create or update a backend. Validates the config, seals the
/// secret at the store, and preserves an existing secret when none is supplied.
async fn upsert(
    State(state): State<AppState>,
    _ctx: AdminCtx,
    Json(body): Json<UpsertReq>,
) -> Response {
    // `kind` is derived from the (validated) config so the column can never disagree.
    let kind = body.config.kind().as_db().to_string();
    let scope = body.scope.unwrap_or_else(|| "deployment".to_string());
    let config_json = match serde_json::to_string(&body.config) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("invalid config: {e}") })),
            )
                .into_response();
        }
    };
    let claim_map_json =
        serde_json::to_string(&body.claim_map).unwrap_or_else(|_| "{}".to_string());

    // Secret handling: a supplied secret is sealed on write; when omitted on an
    // update, carry the existing sealed secret forward (renaming/enabling a backend
    // must not silently wipe its secret).
    let secret = match body.secret {
        Some(s) => Some(s.into_bytes()),
        None => match state.store.get_sso_config(&body.id).await {
            Ok(Some(existing)) => existing.secret,
            _ => None,
        },
    };

    let now = crate::push_relay::now_rfc3339();
    let row = mw_store::SsoConfigRow {
        id: body.id.clone(),
        kind,
        display_name: body.display_name,
        scope,
        enabled: body.enabled,
        config_json,
        secret,
        claim_map_json,
        // `created_at` is preserved on conflict by the store's upsert; supply `now`
        // for the insert case.
        created_at: now.clone(),
        updated_at: now,
    };
    match state.store.put_sso_config(&row).await {
        Ok(()) => Json(json!({ "ok": true, "id": body.id })).into_response(),
        Err(e) => {
            tracing::error!("sso_config upsert failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response()
        }
    }
}

/// `DELETE /admin/sso/{id}` — delete a backend (idempotent).
async fn remove(
    State(state): State<AppState>,
    _ctx: AdminCtx,
    UrlPath(id): UrlPath<String>,
) -> Response {
    match state.store.delete_sso_config(&id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("sso_config delete failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response()
        }
    }
}
