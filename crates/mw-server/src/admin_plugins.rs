//! Third-party plugin allowlist admin API (26.15 t15 e6, TQ6 + PQ6 uninstall).
//!
//! The admin surface for the ONLY security-core loosening in 26.15: an operator drops a
//! `<id>.wasm` into `MW_THIRDPARTY_PLUGIN_DIR`, reviews the digest this endpoint computes
//! over the exact on-disk bytes, and approves that byte-exact SHA-256 into the 0014
//! allowlist so `resolve_component` will load it — and nothing else.
//!
//! Every route is **admin-session-gated** (`super::require_admin`, the same
//! `mw_admin_session` cookie as `/admin/*`) and **audited**. It is a CHILD module of
//! `v7_mount` (declared there via `#[path]`) and is merged into the already-mounted
//! `extra_v7_router()`, so it needs no `lib.rs` mount edit.
//!
//! Routes (all under the existing `/admin/plugins` surface):
//!   * `GET  /admin/plugins/allowlist` — the present-on-disk third-party components with
//!     their computed digest, joined against the stored pins (so the admin approves the
//!     EXACT digest shown).
//!   * `POST /admin/plugins/allowlist` — approve an exact `(pluginId, digestHex)` pin
//!     (rejects a first-party-colliding id and a malformed digest).
//!   * `POST /admin/plugins/allowlist/{plugin_id}/{digest_hex}/revoke` — revoke a pin AND
//!     disable the plugin (effective next load).
//!   * `POST /admin/plugins/{id}/uninstall` — remove the plugin: purge its KV namespace,
//!     delete its allowlist rows, and disable it.

use axum::Router;
use axum::extract::{Extension, Json, Path as UrlPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::AppState;
use crate::plugins::PluginRegistry;

/// The allowlist admin routes. Merged into `v7_mount::extra_v7_router()`.
pub(crate) fn allowlist_router() -> Router<AppState> {
    Router::new()
        .route(
            "/admin/plugins/allowlist",
            get(list_allowlist).post(approve_digest),
        )
        .route(
            "/admin/plugins/allowlist/{plugin_id}/{digest_hex}/revoke",
            post(revoke_digest),
        )
        .route("/admin/plugins/{id}/uninstall", post(uninstall_plugin))
}

/// Scan `MW_THIRDPARTY_PLUGIN_DIR` for `<id>.wasm` files and compute each one's SHA-256.
/// Returns `(plugin_id, computed_digest_hex)` for every present component. A missing/
/// unreadable dir yields an empty list (third-party loading simply off).
fn scan_present_components() -> Vec<(String, String)> {
    let Some(dir) = super::thirdparty_plugin_dir() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for e in entries.flatten() {
        let path = e.path();
        if path.extension().and_then(|s| s.to_str()) != Some("wasm") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let d: [u8; 32] = Sha256::digest(&bytes).into();
        out.push((stem.to_string(), super::hex32(&d)));
    }
    out
}

/// `GET /admin/plugins/allowlist` — the review surface. `present` lists every third-party
/// component on disk with the digest the admin should approve (and whether it is already
/// an active pin / a first-party id, which cannot be third-party-approved). `pins` is the
/// full stored allowlist (including revoked rows) for oversight.
async fn list_allowlist(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(resp) = super::require_admin(&state, &headers).await {
        return resp;
    }
    let pins = match state.store.list_plugin_allowlist().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("allowlist read failed: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "store error").into_response();
        }
    };
    let first_party = super::first_party_ids();
    let present: Vec<_> = scan_present_components()
        .into_iter()
        .map(|(plugin_id, computed_digest)| {
            let is_first_party = first_party.iter().any(|id| *id == plugin_id);
            let approved = pins
                .iter()
                .any(|p| p.plugin_id == plugin_id && p.digest_hex == computed_digest && !p.revoked);
            json!({
                "pluginId": plugin_id,
                "computedDigest": computed_digest,
                "firstParty": is_first_party,
                "approved": approved,
            })
        })
        .collect();
    let pins: Vec<_> = pins
        .iter()
        .map(|p| {
            json!({
                "pluginId": p.plugin_id,
                "digestHex": p.digest_hex,
                "name": p.name,
                "version": p.version,
                "source": p.source,
                "note": p.note,
                "approvedBy": p.approved_by,
                "approvedAt": p.approved_at,
                "revoked": p.revoked,
            })
        })
        .collect();
    Json(json!({ "present": present, "pins": pins })).into_response()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApproveReq {
    plugin_id: String,
    digest_hex: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    note: Option<String>,
}

/// `POST /admin/plugins/allowlist` — approve (pin) an exact `(pluginId, digestHex)`. The
/// store refuses — with a `400`, writing nothing — a `pluginId` that collides with a
/// first-party id (anti-spoof, TQ2) or a non-canonical digest. Audited on success.
async fn approve_digest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ApproveReq>,
) -> Response {
    let admin = match super::require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let digest_hex = body.digest_hex.trim().to_ascii_lowercase();
    let row = mw_store::new_allowlist_pin(
        body.plugin_id.trim(),
        &digest_hex,
        &admin,
        body.name.clone(),
        body.version.clone(),
        body.source.clone(),
        body.note.clone(),
    );
    match state
        .store
        .put_plugin_allowlist(&row, &super::first_party_ids())
        .await
    {
        Ok(()) => {
            super::append_plugin_audit(
                &state.store,
                &admin,
                mw_admin::ActorKind::Admin,
                mw_admin::AuditKind::PluginAllowlistApproved,
                row.plugin_id.trim(),
                json!({ "digest": digest_hex }),
            )
            .await;
            Json(json!({
                "approved": true,
                "pluginId": row.plugin_id,
                "digestHex": digest_hex,
            }))
            .into_response()
        }
        Err(mw_store::PluginAllowlistError::FirstPartyCollision(id)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "plugin id collides with a first-party component; \
                          the first-party pin always takes precedence and cannot be spoofed",
                "pluginId": id,
            })),
        )
            .into_response(),
        Err(mw_store::PluginAllowlistError::MalformedDigest(d)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "digest must be exactly 64 lowercase hex characters",
                "digestHex": d,
            })),
        )
            .into_response(),
        Err(mw_store::PluginAllowlistError::Store(e)) => {
            tracing::error!("allowlist approve failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "store error").into_response()
        }
    }
}

/// `POST /admin/plugins/allowlist/{plugin_id}/{digest_hex}/revoke` — revoke a pin AND
/// disable the plugin so it will not reload (TQ6; effective on the next load, since
/// `resolve_component` reads the allowlist fresh each load). Audited.
async fn revoke_digest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
    UrlPath((plugin_id, digest_hex)): UrlPath<(String, String)>,
) -> Response {
    let admin = match super::require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let revoked = match state
        .store
        .revoke_plugin_allowlist(&plugin_id, &digest_hex)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("allowlist revoke failed: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "store error").into_response();
        }
    };
    // Disable the plugin so a still-running instance is not re-enabled on the next boot
    // (a hot-unload of a live instance is out of scope — matches enable/disable semantics).
    let _ = state.store.set_plugin_enabled(&plugin_id, false).await;
    {
        let mut host = reg.lock().expect("plugin registry lock");
        let _ = host.disable(&plugin_id);
    }
    super::append_plugin_audit(
        &state.store,
        &admin,
        mw_admin::ActorKind::Admin,
        mw_admin::AuditKind::PluginAllowlistRevoked,
        &plugin_id,
        json!({ "digest": digest_hex, "revoked": revoked }),
    )
    .await;
    Json(json!({ "revoked": revoked, "disabled": true })).into_response()
}

/// `POST /admin/plugins/{id}/uninstall` — remove a plugin entirely: purge its KV
/// namespace (all accounts, PQ6), delete its allowlist rows, and disable it. Wires the
/// previously-caller-less `Store::plugin_kv_purge`. Audited.
async fn uninstall_plugin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let admin = match super::require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let kv_purged = state.store.plugin_kv_purge(&id).await.unwrap_or_else(|e| {
        tracing::error!("plugin kv purge failed for '{id}': {e}");
        0
    });
    let pins_removed = state
        .store
        .delete_plugin_allowlist(&id)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("allowlist delete failed for '{id}': {e}");
            0
        });
    let _ = state.store.set_plugin_enabled(&id, false).await;
    {
        let mut host = reg.lock().expect("plugin registry lock");
        let _ = host.disable(&id);
    }
    super::append_plugin_audit(
        &state.store,
        &admin,
        mw_admin::ActorKind::Admin,
        mw_admin::AuditKind::PluginUninstalled,
        &id,
        json!({ "kvRowsPurged": kv_purged, "allowlistRowsRemoved": pins_removed }),
    )
    .await;
    Json(json!({
        "uninstalled": true,
        "kvRowsPurged": kv_purged,
        "allowlistRowsRemoved": pins_removed,
    }))
    .into_response()
}

#[cfg(test)]
mod tests {
    /// The allowlist routes share the `/admin/plugins/*` prefix with the sibling plugin
    /// routers. matchit registers routes eagerly, so a path conflict (e.g. the static
    /// `allowlist` segment vs the `{id}` param) would PANIC here — this builds the exact
    /// production merge (`plugins_router` + `extra_v7_router`, which already folds in
    /// `allowlist_router`) to prove the routers coexist.
    #[test]
    fn allowlist_routes_merge_without_conflict() {
        let _router: axum::Router<crate::AppState> =
            crate::plugins::plugins_router().merge(super::super::extra_v7_router());
    }
}
