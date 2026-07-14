//! TypeScript UI-plugin registry admin + client routes (t10 plan §3 e11, SPEC §22.2).
//!
//! Server side of the sandboxed TS UI-plugin tier. Mirrors the WASM-tier security
//! posture in [`crate::plugins`]: **admin-session-gated** CRUD, **signed-registry**
//! verification (Ed25519 detached signature over the bundle bytes), **deny-by-default**
//! capability grants, and a per-call capability **broker** for the web sandbox.
//!
//! Persistence is the 0010 `ui_plugins` / `ui_plugin_grants` tables via the e0-provided
//! `mw_store` repo methods (`put/get/list_ui_plugins`, `set_ui_plugin_enabled`,
//! `delete_ui_plugin`, `put/list/delete_ui_plugin_grant`). No secret is stored: the
//! `signature` BLOB is a public detached signature.
//!
//! ## Routes (provided here; MOUNTED by e13 into `router()`)
//!
//! Admin (gated on the `mw_admin_session` cookie + `admin.enabled`):
//!   * `GET    /admin/ui-plugins`               — list every registered UI plugin.
//!   * `POST   /admin/ui-plugins`               — upload/register a plugin bundle+manifest.
//!   * `POST   /admin/ui-plugins/{id}/approve`  — approve + enable.
//!   * `POST   /admin/ui-plugins/{id}/enable`   — enable an already-approved plugin.
//!   * `POST   /admin/ui-plugins/{id}/grant`    — grant one declared capability.
//!   * `POST   /admin/ui-plugins/{id}/disable`  — disable.
//!   * `DELETE /admin/ui-plugins/{id}`          — delete (cascades grants).
//!
//! Web host (the SPA sandbox tier):
//!   * `GET    /api/ui-plugins`                 — approved+enabled registrations + banner.
//!   * `POST   /api/ui-plugins/{id}/rpc`        — the capability broker (net/store RPC).
//!
//! ## Signed-registry (mirrors §7.5 / [`crate::plugins`])
//! A signed plugin's detached Ed25519 signature is verified over the exact bundle bytes
//! against the deployment trust root (`MW_UI_PLUGIN_TRUST_KEYS`, comma-separated hex
//! 32-byte public keys; empty ⇒ every signed load fails closed). An **unsigned** plugin
//! may register only under an explicit admin `allowUnsigned`, which flags a
//! persistent-banner signal (`bannerSignal` / `unsignedBanner`) so the tier is never
//! silently unsigned. Verification reuses `mw_plugin::TrustRoot`.
//!
//! ## Deny-by-default broker
//! `/api/ui-plugins/{id}/rpc` gates every guest→host call: the plugin must be
//! approved+enabled, the requested `cap` must be granted (intersected with the manifest's
//! declared caps at grant time), and the `method` must be in that capability's method
//! allowlist ([`cap_methods`], mirroring the web `CAP_METHOD_ALLOWLIST`). `net:host-allowlist`
//! egress is additionally checked against the grant's host allowlist; `store:kv-scoped`
//! is a per-plugin scoped key/value.
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use base64::Engine as _;
use serde::Deserialize;
use serde_json::{Value, json};

use mw_plugin::{SignatureStatus, TrustRoot};
use mw_store::{Store, StoreError, UiPluginGrantRow, UiPluginRow};

use crate::AppState;

const ADMIN_COOKIE: &str = "mw_admin_session";

/// The frozen RPC envelope protocol version (mirrors web `RPC_PROTOCOL_VERSION`).
const RPC_PROTOCOL_VERSION: u32 = 1;

/// e13 merges this into `router()`. Unmounted today (`router()` byte-unchanged).
pub(crate) fn ui_plugins_router() -> Router<AppState> {
    Router::new()
        .route("/admin/ui-plugins", get(list_admin).post(register))
        .route("/admin/ui-plugins/{id}/approve", post(approve))
        .route("/admin/ui-plugins/{id}/enable", post(enable))
        .route("/admin/ui-plugins/{id}/grant", post(grant))
        .route("/admin/ui-plugins/{id}/disable", post(disable))
        .route("/admin/ui-plugins/{id}", delete(remove))
        .route("/api/ui-plugins", get(list_public))
        .route("/api/ui-plugins/{id}/rpc", post(broker_rpc))
}

// ── admin session gate (mirrors plugins.rs / admin.rs) ──────────────────────────

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

// ── manifest model (server view of the frozen §2.3 `ui-plugin.json`) ────────────

/// The subset of the frozen web `UiPluginManifest` the server persists + gates on.
/// `signature` is the base64 detached Ed25519 signature over the bundle, `None` when
/// unsigned.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UiManifest {
    id: String,
    name: String,
    version: String,
    #[serde(default)]
    signature: Option<String>,
    #[serde(default)]
    extension_points: Vec<String>,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    csp: String,
}

/// `POST /admin/ui-plugins` body: the manifest, the base64 bundle the signature signs,
/// and the admin `allowUnsigned` policy for unsigned plugins.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterReq {
    manifest: Value,
    #[serde(default)]
    bundle: Option<String>,
    #[serde(default)]
    allow_unsigned: bool,
}

/// `POST /admin/ui-plugins/{id}/grant` body: one capability + its scoped params
/// (e.g. `{ "hosts": ["api.example.com"] }` for `net:host-allowlist`).
#[derive(Debug, Deserialize)]
struct GrantReq {
    capability: String,
    #[serde(default)]
    params: Value,
}

// ── signed-registry verification (reuses mw_plugin::TrustRoot) ───────────────────

/// The deployment trust root for UI plugins: `MW_UI_PLUGIN_TRUST_KEYS` = comma-separated
/// hex-encoded 32-byte Ed25519 public keys. Empty/unset ⇒ every *signed* plugin fails
/// closed (unsigned still needs `allowUnsigned`).
fn ui_trust_root() -> TrustRoot {
    let raw = std::env::var("MW_UI_PLUGIN_TRUST_KEYS").unwrap_or_default();
    let mut keys: Vec<[u8; 32]> = Vec::new();
    for part in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(bytes) = decode_hex(part)
            && let Ok(k) = <[u8; 32]>::try_from(bytes.as_slice())
        {
            keys.push(k);
        }
    }
    TrustRoot::from_public_keys(&keys).unwrap_or_else(|_| TrustRoot::empty())
}

/// Decide whether a plugin may register given its (base64) signature, the bundle bytes,
/// the trust root, and the admin `allow_unsigned` policy. **Fails closed.** Returns the
/// [`SignatureStatus`] plus the raw signature bytes to persist (`None` when unsigned).
fn verify_bundle(
    trust: &TrustRoot,
    bundle: Option<&[u8]>,
    signature_b64: Option<&str>,
    allow_unsigned: bool,
) -> Result<(SignatureStatus, Option<Vec<u8>>), String> {
    match signature_b64 {
        Some(sig_b64) => {
            let raw = base64::engine::general_purpose::STANDARD
                .decode(sig_b64.trim())
                .map_err(|_| "signature is not valid base64".to_string())?;
            if raw.len() != 64 {
                return Err(format!("signature must be 64 bytes, got {}", raw.len()));
            }
            let bytes = bundle
                .ok_or_else(|| "a signed plugin requires its bundle to verify".to_string())?;
            // Reuse the vetted mw_plugin verifier (hex-encoded detached signature).
            trust
                .verify(bytes, &to_hex(&raw))
                .map_err(|e| e.to_string())?;
            Ok((SignatureStatus::Verified, Some(raw)))
        }
        None => {
            if allow_unsigned {
                Ok((SignatureStatus::UnsignedAllowed, None))
            } else {
                Err("unsigned plugin requires allowUnsigned".to_string())
            }
        }
    }
}

// ── deny-by-default capability policy (mirrors web CAP_METHOD_ALLOWLIST) ─────────

/// The per-capability method allowlist. A method not listed for a granted capability is
/// rejected by the broker. `ui:*` render capabilities expose no guest-initiated methods.
fn cap_methods(cap: &str) -> &'static [&'static str] {
    match cap {
        "net:host-allowlist" => &["fetch"],
        "store:kv-scoped" => &["get", "put"],
        _ => &[],
    }
}

/// The closed set of capabilities a manifest may declare.
fn is_known_capability(cap: &str) -> bool {
    matches!(
        cap,
        "ui:compose-action"
            | "ui:message-toolbar"
            | "ui:settings-panel"
            | "net:host-allowlist"
            | "store:kv-scoped"
    )
}

/// A structured broker RPC error (never leaks host internals to the guest).
struct RpcErr {
    code: &'static str,
    message: String,
}

impl RpcErr {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// The broker gate: deny-by-default. `Some(err)` ⇒ reject. A capability not in `grants`
/// is `capability-denied`; a method not in the capability's allowlist is `method-denied`.
fn broker_gate(grants: &[UiPluginGrantRow], cap: &str, method: &str) -> Option<RpcErr> {
    if !grants.iter().any(|g| g.capability == cap) {
        return Some(RpcErr::new(
            "capability-denied",
            format!("capability not granted: {cap}"),
        ));
    }
    if !cap_methods(cap).contains(&method) {
        return Some(RpcErr::new(
            "method-denied",
            format!("method not allowed for {cap}: {method}"),
        ));
    }
    None
}

// ── persistence helpers (take `&Store`; unit-tested) ────────────────────────────

/// Persist a verified/allowed registration to 0010 `ui_plugins` (unapproved, disabled).
async fn persist_registration(
    store: &Store,
    manifest_value: &Value,
    manifest: &UiManifest,
    signature: Option<Vec<u8>>,
) -> Result<(), StoreError> {
    store
        .put_ui_plugin(&UiPluginRow {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            manifest_json: serde_json::to_string(manifest_value).unwrap_or_default(),
            signature,
            approved_by: None,
            enabled: false,
            capabilities_json: serde_json::to_string(&manifest.capabilities).unwrap_or_default(),
            extension_points_json: serde_json::to_string(&manifest.extension_points)
                .unwrap_or_default(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .await
}

/// Approve + enable a registered plugin. Returns `false` when the id is unknown.
async fn do_approve(store: &Store, id: &str, admin: &str) -> Result<bool, StoreError> {
    let Some(mut row) = store.get_ui_plugin(id).await? else {
        return Ok(false);
    };
    row.approved_by = Some(admin.to_string());
    row.enabled = true;
    store.put_ui_plugin(&row).await?;
    Ok(true)
}

/// The outcome of a capability-grant attempt.
enum GrantOutcome {
    Unknown,
    Denied,
    Granted,
}

/// Grant one capability — **deny-by-default**: a capability the manifest never declared
/// (or an unknown capability) is refused and never persisted.
async fn do_grant(
    store: &Store,
    id: &str,
    admin: &str,
    cap: &str,
    params: &Value,
) -> Result<GrantOutcome, StoreError> {
    let Some(row) = store.get_ui_plugin(id).await? else {
        return Ok(GrantOutcome::Unknown);
    };
    let declared: Vec<String> = serde_json::from_str(&row.capabilities_json).unwrap_or_default();
    if !is_known_capability(cap) || !declared.iter().any(|c| c == cap) {
        return Ok(GrantOutcome::Denied);
    }
    store
        .put_ui_plugin_grant(&UiPluginGrantRow {
            plugin_id: id.to_string(),
            capability: cap.to_string(),
            params_json: serde_json::to_string(params).unwrap_or_else(|_| "{}".to_string()),
            granted_by: admin.to_string(),
            granted_at: chrono::Utc::now().to_rfc3339(),
        })
        .await?;
    Ok(GrantOutcome::Granted)
}

// ── admin handlers ──────────────────────────────────────────────────────────────

/// `GET /admin/ui-plugins` — every registered plugin (id/name/version/enabled/approved/
/// signed + declared capabilities). `signed=false` ⇒ the UI raises the persistent banner.
async fn list_admin(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    let rows = match state.store.list_ui_plugins().await {
        Ok(r) => r,
        Err(e) => return store_error(&e),
    };
    let plugins: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "name": r.name,
                "version": r.version,
                "enabled": r.enabled,
                "approved": r.approved_by.is_some(),
                "signed": r.signature.is_some(),
                "capabilities": parse_json_array(&r.capabilities_json),
                "extensionPoints": parse_json_array(&r.extension_points_json),
            })
        })
        .collect();
    Json(json!({ "plugins": plugins })).into_response()
}

/// `POST /admin/ui-plugins` — upload/register a plugin. Verifies the signed-registry
/// policy; an unsigned plugin registers only with `allowUnsigned` and returns a
/// `bannerSignal` (mirrors the WASM tier's persistent-banner behaviour).
async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RegisterReq>,
) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    let manifest: UiManifest = match serde_json::from_value(body.manifest.clone()) {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("invalid manifest: {e}") })),
            )
                .into_response();
        }
    };
    // Reject any undeclared/unknown capability up front (deny-by-default at the door).
    if let Some(bad) = manifest
        .capabilities
        .iter()
        .find(|c| !is_known_capability(c))
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("unknown capability declared: {bad}") })),
        )
            .into_response();
    }

    let bundle = match body.bundle.as_deref() {
        Some(b64) => match base64::engine::general_purpose::STANDARD.decode(b64.trim()) {
            Ok(bytes) => Some(bytes),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "bundle is not valid base64" })),
                )
                    .into_response();
            }
        },
        None => None,
    };

    let trust = ui_trust_root();
    let (status, signature) = match verify_bundle(
        &trust,
        bundle.as_deref(),
        manifest.signature.as_deref(),
        body.allow_unsigned,
    ) {
        Ok(v) => v,
        Err(msg) => {
            // Unsigned-without-policy + bad/untrusted signature ⇒ refuse (fail closed).
            return (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": msg, "signed": manifest.signature.is_some() })),
            )
                .into_response();
        }
    };

    if let Err(e) = persist_registration(&state.store, &body.manifest, &manifest, signature).await {
        return store_error(&e);
    }

    let unsigned = status == SignatureStatus::UnsignedAllowed;
    (
        StatusCode::CREATED,
        Json(json!({
            "id": manifest.id,
            "signed": !unsigned,
            // Persistent-banner signal: an unsigned plugin was admitted under allowUnsigned.
            "bannerSignal": unsigned,
        })),
    )
        .into_response()
}

/// `POST /admin/ui-plugins/{id}/approve` — approve + enable.
async fn approve(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let admin = match require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    match do_approve(&state.store, &id, &admin).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => not_found(&id),
        Err(e) => store_error(&e),
    }
}

/// `POST /admin/ui-plugins/{id}/enable` — enable an already-approved plugin.
async fn enable(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    match state.store.get_ui_plugin(&id).await {
        Ok(Some(row)) if row.approved_by.is_some() => {
            match state.store.set_ui_plugin_enabled(&id, true).await {
                Ok(()) => StatusCode::NO_CONTENT.into_response(),
                Err(e) => store_error(&e),
            }
        }
        Ok(Some(_)) => (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "plugin must be approved before it can be enabled" })),
        )
            .into_response(),
        Ok(None) => not_found(&id),
        Err(e) => store_error(&e),
    }
}

/// `POST /admin/ui-plugins/{id}/grant` — grant one declared capability (deny-by-default).
async fn grant(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
    Json(body): Json<GrantReq>,
) -> Response {
    let admin = match require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    match do_grant(&state.store, &id, &admin, &body.capability, &body.params).await {
        Ok(GrantOutcome::Granted) => Json(json!({
            "pluginId": id,
            "capability": body.capability,
            "granted": true,
        }))
        .into_response(),
        Ok(GrantOutcome::Denied) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "capability not declared by the manifest (deny-by-default)",
                "capability": body.capability,
                "granted": false,
            })),
        )
            .into_response(),
        Ok(GrantOutcome::Unknown) => not_found(&id),
        Err(e) => store_error(&e),
    }
}

/// `POST /admin/ui-plugins/{id}/disable`.
async fn disable(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    match state.store.set_ui_plugin_enabled(&id, false).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => store_error(&e),
    }
}

/// `DELETE /admin/ui-plugins/{id}` — delete a plugin and its grants (idempotent).
async fn remove(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    match state.store.delete_ui_plugin(&id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => store_error(&e),
    }
}

// ── web-host handlers ────────────────────────────────────────────────────────────

/// `GET /api/ui-plugins` — the approved+enabled tier the SPA loads, as
/// `UiPluginRegistration[]` (manifest + grants + flags). `unsignedBanner` lists any
/// approved-but-unsigned plugin ids so the SPA raises the persistent banner.
async fn list_public(State(state): State<AppState>) -> Response {
    let rows = match state.store.list_ui_plugins().await {
        Ok(r) => r,
        Err(e) => return store_error(&e),
    };
    let mut plugins: Vec<Value> = Vec::new();
    let mut banner: Vec<String> = Vec::new();
    for r in &rows {
        if r.approved_by.is_none() || !r.enabled {
            continue; // deny-by-default: only approved+enabled plugins are served.
        }
        let manifest: Value =
            serde_json::from_str(&r.manifest_json).unwrap_or_else(|_| json!({ "id": r.id }));
        let grants: Vec<Value> = match state.store.list_ui_plugin_grants(&r.id).await {
            Ok(gs) => gs
                .iter()
                .map(|g| {
                    json!({
                        "capability": g.capability,
                        "params": parse_json_object(&g.params_json),
                    })
                })
                .collect(),
            Err(e) => return store_error(&e),
        };
        if r.signature.is_none() {
            banner.push(r.id.clone());
        }
        plugins.push(json!({
            "manifest": manifest,
            "grants": grants,
            "enabled": r.enabled,
            "approved": r.approved_by.is_some(),
        }));
    }
    Json(json!({ "plugins": plugins, "unsignedBanner": banner })).into_response()
}

/// A guest→host RPC request over the frozen `{v,id,cap,method,args}` envelope.
#[derive(Debug, Deserialize)]
struct RpcRequest {
    v: u32,
    id: String,
    cap: String,
    method: String,
    #[serde(default)]
    args: Vec<Value>,
}

/// `POST /api/ui-plugins/{id}/rpc` — the capability broker. Deny-by-default: the plugin
/// must be approved+enabled, the `cap` granted, and the `method` allowlisted. Dispatches
/// `net:host-allowlist`/`fetch` (allowlist-checked egress) and `store:kv-scoped`
/// `get`/`put` (per-plugin scoped KV).
async fn broker_rpc(
    State(state): State<AppState>,
    UrlPath(plugin_id): UrlPath<String>,
    Json(req): Json<RpcRequest>,
) -> Response {
    if req.v != RPC_PROTOCOL_VERSION {
        return rpc_err(&req.id, "bad-request", "unsupported RPC protocol version");
    }
    // The plugin must exist + be approved + enabled.
    let plugin = match state.store.get_ui_plugin(&plugin_id).await {
        Ok(Some(p)) if p.approved_by.is_some() && p.enabled => p,
        Ok(_) => {
            return rpc_err(
                &req.id,
                "capability-denied",
                "plugin is not approved and enabled",
            );
        }
        Err(_) => return rpc_err(&req.id, "internal", "registry lookup failed"),
    };
    let _ = &plugin;

    let grants = match state.store.list_ui_plugin_grants(&plugin_id).await {
        Ok(g) => g,
        Err(_) => return rpc_err(&req.id, "internal", "grant lookup failed"),
    };
    if let Some(err) = broker_gate(&grants, &req.cap, &req.method) {
        return rpc_err(&req.id, err.code, err.message);
    }

    match (req.cap.as_str(), req.method.as_str()) {
        ("net:host-allowlist", "fetch") => broker_fetch(&req, &grants).await,
        ("store:kv-scoped", "get") => broker_kv_get(&plugin_id, &req),
        ("store:kv-scoped", "put") => broker_kv_put(&plugin_id, &req),
        _ => rpc_err(
            &req.id,
            "not-implemented",
            "no handler for this capability method",
        ),
    }
}

/// `net:host-allowlist`/`fetch`: allowlist-checked GET egress proxy. The target host must
/// match the grant's `hosts` allowlist (exact or dot-suffix); otherwise `capability-denied`.
async fn broker_fetch(req: &RpcRequest, grants: &[UiPluginGrantRow]) -> Response {
    let url = match req.args.first().and_then(Value::as_str) {
        Some(u) => u,
        None => return rpc_err(&req.id, "bad-request", "fetch requires a url argument"),
    };
    let host = match url_host(url) {
        Some(h) => h,
        None => return rpc_err(&req.id, "bad-request", "could not parse url host"),
    };
    let params = grants
        .iter()
        .find(|g| g.capability == "net:host-allowlist")
        .map(|g| g.params_json.as_str())
        .unwrap_or("{}");
    if !host_allowed(&host, params) {
        return rpc_err(
            &req.id,
            "capability-denied",
            format!("host not in the granted allowlist: {host}"),
        );
    }
    match reqwest::Client::new().get(url).send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            rpc_ok(&req.id, json!({ "status": status, "body": body }))
        }
        Err(_) => rpc_err(&req.id, "internal", "egress request failed"),
    }
}

/// A per-plugin scoped key/value store for the `store:kv-scoped` capability. In-process
/// (there is no 0010 KV table); keyed by `plugin_id \0 key` so plugins cannot read each
/// other's namespaces.
static UI_KV: LazyLock<Mutex<HashMap<String, Value>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn kv_key(plugin_id: &str, key: &str) -> String {
    format!("{plugin_id}\u{0}{key}")
}

fn broker_kv_get(plugin_id: &str, req: &RpcRequest) -> Response {
    let key = match req.args.first().and_then(Value::as_str) {
        Some(k) => k,
        None => return rpc_err(&req.id, "bad-request", "get requires a key argument"),
    };
    let value = UI_KV
        .lock()
        .expect("ui kv lock")
        .get(&kv_key(plugin_id, key))
        .cloned()
        .unwrap_or(Value::Null);
    rpc_ok(&req.id, value)
}

fn broker_kv_put(plugin_id: &str, req: &RpcRequest) -> Response {
    let key = match req.args.first().and_then(Value::as_str) {
        Some(k) => k.to_string(),
        None => return rpc_err(&req.id, "bad-request", "put requires a key argument"),
    };
    let value = req.args.get(1).cloned().unwrap_or(Value::Null);
    UI_KV
        .lock()
        .expect("ui kv lock")
        .insert(kv_key(plugin_id, &key), value);
    rpc_ok(&req.id, json!({ "ok": true }))
}

// ── response + parsing helpers ───────────────────────────────────────────────────

/// A successful broker RPC response: `{v,id,ok}`.
fn rpc_ok(id: &str, ok: Value) -> Response {
    Json(json!({ "v": RPC_PROTOCOL_VERSION, "id": id, "ok": ok })).into_response()
}

/// A broker RPC error response: `{v,id,err:{code,message}}` (never leaks host internals).
fn rpc_err(id: &str, code: &str, message: impl Into<String>) -> Response {
    Json(json!({
        "v": RPC_PROTOCOL_VERSION,
        "id": id,
        "err": { "code": code, "message": message.into() },
    }))
    .into_response()
}

fn not_found(id: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": format!("unknown UI plugin '{id}'") })),
    )
        .into_response()
}

fn store_error(e: &StoreError) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": format!("registry error: {e}") })),
    )
        .into_response()
}

fn parse_json_array(s: &str) -> Value {
    serde_json::from_str::<Value>(s)
        .ok()
        .filter(Value::is_array)
        .unwrap_or_else(|| json!([]))
}

fn parse_json_object(s: &str) -> Value {
    serde_json::from_str::<Value>(s)
        .ok()
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}))
}

/// Extract the lowercase host from a URL without pulling in a URL crate.
fn url_host(url: &str) -> Option<String> {
    let after = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let authority = after.split(['/', '?', '#']).next().unwrap_or("");
    let authority = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let host = authority.split(':').next().unwrap_or(authority);
    if host.is_empty() {
        None
    } else {
        Some(host.to_ascii_lowercase())
    }
}

/// Whether `host` matches the grant's `{ "hosts": [...] }` allowlist (exact or dot-suffix).
fn host_allowed(host: &str, params_json: &str) -> bool {
    let params: Value = serde_json::from_str(params_json).unwrap_or_else(|_| json!({}));
    let Some(hosts) = params.get("hosts").and_then(Value::as_array) else {
        return false;
    };
    hosts
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_ascii_lowercase)
        .any(|allowed| host == allowed || host.ends_with(&format!(".{allowed}")))
}

/// Lowercase hex encode (for `mw_plugin::TrustRoot::verify`, which takes hex).
fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Minimal hex decoder for trust-root keys.
fn decode_hex(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if !s.len().is_multiple_of(2) {
        return None;
    }
    let val = |c: u8| -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    };
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(s.len() / 2);
    for pair in b.chunks(2) {
        out.push((val(pair[0])? << 4) | val(pair[1])?);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::ServerKey;

    fn manifest_json(id: &str, signature: Option<&str>, caps: &[&str]) -> Value {
        json!({
            "id": id,
            "name": id,
            "version": "1.0.0",
            "signature": signature,
            "extensionPoints": ["message-toolbar"],
            "capabilities": caps,
            "csp": "default-src 'none'",
        })
    }

    // ── signed-registry / banner ─────────────────────────────────────────────────

    #[test]
    fn unsigned_requires_allow_unsigned_and_signals_banner() {
        let trust = TrustRoot::empty();
        // Unsigned + no policy ⇒ refused (fail closed).
        assert!(verify_bundle(&trust, Some(b"bundle"), None, false).is_err());
        // Unsigned + allowUnsigned ⇒ admitted, flagged UnsignedAllowed (⇒ banner).
        let (status, sig) = verify_bundle(&trust, Some(b"bundle"), None, true).unwrap();
        assert_eq!(status, SignatureStatus::UnsignedAllowed);
        assert!(sig.is_none());
    }

    #[test]
    fn signed_plugin_fails_closed_against_empty_trust_root() {
        let trust = TrustRoot::empty();
        // A well-formed (base64, 64-byte) but untrusted signature never verifies.
        let sig_b64 = base64::engine::general_purpose::STANDARD.encode([0u8; 64]);
        assert!(verify_bundle(&trust, Some(b"bundle"), Some(&sig_b64), false).is_err());
        // A malformed signature is rejected too.
        assert!(verify_bundle(&trust, Some(b"bundle"), Some("not-base64!!"), false).is_err());
    }

    // ── deny-by-default broker ───────────────────────────────────────────────────

    #[test]
    fn broker_rejects_ungranted_capability() {
        let grants = vec![UiPluginGrantRow {
            plugin_id: "p".into(),
            capability: "net:host-allowlist".into(),
            params_json: r#"{"hosts":["api.example.com"]}"#.into(),
            granted_by: "admin".into(),
            granted_at: "t".into(),
        }];
        // A capability never granted is denied.
        let err = broker_gate(&grants, "store:kv-scoped", "get").unwrap();
        assert_eq!(err.code, "capability-denied");
        // A granted capability but a method outside its allowlist is denied.
        let err = broker_gate(&grants, "net:host-allowlist", "delete").unwrap();
        assert_eq!(err.code, "method-denied");
        // The granted capability + allowlisted method passes the gate.
        assert!(broker_gate(&grants, "net:host-allowlist", "fetch").is_none());
    }

    #[test]
    fn host_allowlist_matches_exact_and_dot_suffix_only() {
        let params = r#"{"hosts":["example.com"]}"#;
        assert!(host_allowed("example.com", params));
        assert!(host_allowed("api.example.com", params));
        assert!(!host_allowed("notexample.com", params));
        assert!(!host_allowed("evil.com", params));
        assert!(!host_allowed("example.com", "{}")); // empty allowlist ⇒ deny.
    }

    #[test]
    fn url_host_strips_scheme_userinfo_port_and_path() {
        assert_eq!(
            url_host("https://user:pw@API.Example.com:8443/x?y"),
            Some("api.example.com".into())
        );
        assert_eq!(url_host("http://host/"), Some("host".into()));
        assert_eq!(url_host(""), None);
    }

    // ── persistence: upload → approve → grant persists (0010 repo) ────────────────

    #[tokio::test]
    async fn upload_approve_grant_persists_and_is_deny_by_default() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let trust = TrustRoot::empty();

        // Upload (unsigned, allowed) → persisted, unapproved + disabled.
        let mv = manifest_json(
            "snooze",
            None,
            &["ui:message-toolbar", "net:host-allowlist"],
        );
        let manifest: UiManifest = serde_json::from_value(mv.clone()).unwrap();
        let (status, sig) = verify_bundle(&trust, Some(b"bundle-bytes"), None, true).unwrap();
        assert_eq!(status, SignatureStatus::UnsignedAllowed);
        persist_registration(&store, &mv, &manifest, sig)
            .await
            .unwrap();
        let row = store.get_ui_plugin("snooze").await.unwrap().unwrap();
        assert!(
            row.approved_by.is_none() && !row.enabled,
            "unapproved on upload"
        );
        assert!(
            row.signature.is_none(),
            "unsigned ⇒ NULL signature (banner)"
        );

        // Approve → approved + enabled.
        assert!(do_approve(&store, "snooze", "admin@x").await.unwrap());
        let row = store.get_ui_plugin("snooze").await.unwrap().unwrap();
        assert_eq!(row.approved_by.as_deref(), Some("admin@x"));
        assert!(row.enabled);

        // Grant a DECLARED capability → persists to ui_plugin_grants.
        let params = json!({ "hosts": ["api.example.com"] });
        assert!(matches!(
            do_grant(&store, "snooze", "admin@x", "net:host-allowlist", &params)
                .await
                .unwrap(),
            GrantOutcome::Granted
        ));
        let grants = store.list_ui_plugin_grants("snooze").await.unwrap();
        assert_eq!(grants.len(), 1);
        assert_eq!(grants[0].capability, "net:host-allowlist");

        // Grant an UNDECLARED capability → deny-by-default, nothing persisted.
        assert!(matches!(
            do_grant(&store, "snooze", "admin@x", "store:kv-scoped", &json!({}))
                .await
                .unwrap(),
            GrantOutcome::Denied
        ));
        assert_eq!(
            store.list_ui_plugin_grants("snooze").await.unwrap().len(),
            1,
            "undeclared grant must not persist"
        );

        // Unknown plugin.
        assert!(matches!(
            do_grant(&store, "nope", "admin@x", "net:host-allowlist", &params)
                .await
                .unwrap(),
            GrantOutcome::Unknown
        ));
    }

    #[tokio::test]
    async fn kv_broker_scopes_per_plugin() {
        let get = RpcRequest {
            v: 1,
            id: "r1".into(),
            cap: "store:kv-scoped".into(),
            method: "get".into(),
            args: vec![json!("k")],
        };
        let put = RpcRequest {
            v: 1,
            id: "r2".into(),
            cap: "store:kv-scoped".into(),
            method: "put".into(),
            args: vec![json!("k"), json!("v-a")],
        };
        broker_kv_put("plugin-a", &put);
        // Plugin B cannot read plugin A's namespace.
        let resp_b = broker_kv_get("plugin-b", &get);
        assert_eq!(resp_b.status(), StatusCode::OK);
        // (namespaced key ensures isolation; direct map check)
        assert!(
            UI_KV
                .lock()
                .unwrap()
                .get(&kv_key("plugin-a", "k"))
                .is_some()
        );
        assert!(
            UI_KV
                .lock()
                .unwrap()
                .get(&kv_key("plugin-b", "k"))
                .is_none()
        );
    }
}
