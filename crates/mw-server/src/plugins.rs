//! Engine-plugin registry admin routes (SPEC §22): register, approve, enable,
//! disable, grant, allow-unsigned, settings and a classifier test, over the 0008
//! `plugins` / `plugin_grants` tables.
//!
//! Every route is **admin-session-gated** (the `mw_admin_session` cookie, same
//! domain as `/admin/*`; `401` when the admin panel is disabled) and every change
//! writes an audit row. The store is the registry: each handler reads and writes
//! the 0008 rows and answers with the plugin as [`plugin_view`] reports it. What
//! runs is decided in `v7_mount` from those rows:
//!
//!   * **Trust.** A first-party id is trusted by the SHA-256 compiled into the
//!     server and is registered by id alone; its manifest is compiled in too. Any
//!     other id needs an administrator-pinned digest for the component file on disk
//!     before it can be registered, and the stored allow-unsigned flag before an
//!     unsigned one can be enabled (`v7_mount::TrustPolicy`).
//!   * **Grants.** A load runs with the stored grant rows ∩ the manifest ∩ the
//!     provenance filter (`v7_mount::effective_capabilities`). No grant row, no
//!     capability. `grant` refuses a capability the manifest does not declare.
//!   * **Effect.** A spam classifier is loaded, replaced and unloaded by
//!     `v7_mount::sync_spam_classifier`, which every handler here calls after its
//!     change. An account backend is loaded at start-up only; a hook the server
//!     never calls is never loaded. [`plugin_view`] reports `loaded`,
//!     `restartRequired` and `notLoadedReason` accordingly.
//!
//! The in-process `mw_plugin::PluginHost` registry (`PluginHost::register` /
//! `approve` / `enable`) is kept in step where its API allows; nothing reads it. It
//! has no way to remove or replace an entry, so it can hold an id that was
//! uninstalled.

use std::sync::{Arc, Mutex};

use axum::extract::rejection::JsonRejection;
use axum::extract::{Extension, Path as UrlPath, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use mw_plugin::{Capability, PluginHost, PluginLimits, PluginManifest};
use mw_store::PluginRow;

use crate::AppState;
use crate::v7_mount::{self, PluginRole, TrustPolicy};

/// The shared, lockable plugin host e14 injects. `pub` so the boot-load integration
/// test (`tests/v7_boot_load.rs`) can hand `load_plugin_backends` a fixture-backed
/// host — it is a transparent alias for `Arc<Mutex<mw_plugin::PluginHost>>`.
pub type PluginRegistry = Arc<Mutex<PluginHost>>;

const ADMIN_COOKIE: &str = "mw_admin_session";

/// The KV key both spam classifiers read their daemon address from.
const ENDPOINT_KEY: &str = "endpoint";

/// The message `POST /admin/plugins/{id}/test` hands the classifier.
const TEST_MESSAGE: &[u8] = b"From: sender@example.org\r\n\
To: recipient@example.org\r\n\
Subject: Mailwoman classifier test\r\n\
Message-ID: <classifier-test@mailwoman.invalid>\r\n\
\r\n\
This message was sent by the Mailwoman admin panel to test the spam classifier.\r\n";

pub(crate) fn plugins_router() -> Router<AppState> {
    Router::new()
        .route("/admin/plugins", get(list).post(register))
        .route("/admin/plugins/{id}/approve", post(approve))
        .route("/admin/plugins/{id}/enable", post(enable))
        .route("/admin/plugins/{id}/disable", post(disable))
        .route("/admin/plugins/{id}/grant", post(grant))
        .route("/admin/plugins/{id}/allow-unsigned", post(allow_unsigned))
        .route("/admin/plugins/{id}/settings", post(settings))
        .route("/admin/plugins/{id}/test", post(test_classifier))
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

// ── responses ────────────────────────────────────────────────────────────────

fn refuse(status: StatusCode, code: &str, message: impl Into<String>) -> Response {
    (
        status,
        Json(json!({ "error": message.into(), "code": code })),
    )
        .into_response()
}

fn bad_request(message: impl Into<String>) -> Response {
    refuse(StatusCode::BAD_REQUEST, "bad-request", message)
}

fn unknown_plugin(id: &str) -> Response {
    refuse(
        StatusCode::NOT_FOUND,
        "unknown-plugin",
        format!("unknown plugin '{id}'"),
    )
}

fn store_error(what: &str, e: &dyn std::fmt::Display) -> Response {
    tracing::error!("plugin registry {what} failed: {e}");
    refuse(
        StatusCode::INTERNAL_SERVER_ERROR,
        "store-error",
        "store error",
    )
}

/// Read the registry row for `id`, or the response to answer with.
async fn row_of(state: &AppState, id: &str) -> Result<PluginRow, Response> {
    match state.store.get_plugin(id).await {
        Ok(Some(row)) => Ok(row),
        Ok(None) => Err(unknown_plugin(id)),
        Err(e) => Err(store_error("read", &e)),
    }
}

/// One registered plugin as the admin API reports it. Every key is always present.
///
/// - `capabilities`: what the manifest declares. `granted`: what a deployment-wide
///   instance would run with (stored grants ∩ manifest ∩ provenance).
/// - `signed`: the manifest carries a signature. `allowUnsigned`: the stored flag,
///   always `false` for a first-party id, whose trust is the digest pin.
/// - `enabled` is the stored setting. `loaded` is whether an instance is running in
///   this process, with `loadedCapabilities`; when it is not, `restartRequired` or
///   `notLoadedReason` says why.
/// - `endpoint`: the daemon address a spam classifier is configured with, `null`
///   when unset or when the plugin is not a spam classifier.
pub(crate) async fn plugin_view(state: &AppState, reg: &PluginRegistry, row: &PluginRow) -> Value {
    let manifest = v7_mount::manifest_of(row);
    let trust = TrustPolicy::of(&state.store, &row.id).await;
    let (granted, _) = v7_mount::effective_capabilities(&state.store, &manifest, None).await;
    let status = v7_mount::plugin_status(&state.store, reg, row).await;
    let role = PluginRole::of(&manifest);
    let endpoint = if takes_endpoint(&manifest) {
        state
            .store
            .plugin_kv_get(&row.id, "", ENDPOINT_KEY)
            .await
            .ok()
            .flatten()
            .and_then(|bytes| String::from_utf8(bytes).ok())
    } else {
        None
    };
    json!({
        "id": row.id,
        "name": row.name,
        "version": row.version,
        "firstParty": trust == TrustPolicy::FirstPartyDigestPin,
        "trust": trust.wire(),
        "signed": manifest.signature.is_some(),
        "allowUnsigned": trust == TrustPolicy::AdminPinnedDigest { allow_unsigned: true },
        "approved": row.approved_by.is_some(),
        "approvedBy": row.approved_by,
        "enabled": row.enabled,
        "role": match role {
            PluginRole::Bridge => "account-backend",
            PluginRole::Spam => "spam-classifier",
            PluginRole::Other => "none",
        },
        "capabilities": manifest.capabilities,
        "granted": granted,
        "netAllowlist": manifest.net_allowlist,
        "limits": {
            "memoryMb": manifest.limits.memory_mb,
            "deadlineMs": manifest.limits.deadline_ms,
            "fuel": manifest.limits.fuel,
        },
        "endpoint": endpoint,
        "loaded": status.loaded,
        "loadedCapabilities": status.loaded_capabilities,
        "restartRequired": status.restart_required,
        "notLoadedReason": status.not_loaded.map(v7_mount::NotLoaded::wire),
    })
}

/// Whether the plugin reads a daemon address from its scoped KV: a spam classifier
/// that declares `store-kv-scoped`.
fn takes_endpoint(manifest: &PluginManifest) -> bool {
    PluginRole::of(manifest) == PluginRole::Spam
        && manifest.capabilities.contains(&Capability::StoreKvScoped)
}

/// Apply a registry change to what is running, then answer with the plugin's view.
async fn applied(state: &AppState, reg: &PluginRegistry, id: &str, status: StatusCode) -> Response {
    v7_mount::sync_spam_classifier(reg, &state.store).await;
    match row_of(state, id).await {
        Ok(row) => (
            status,
            Json(json!({ "plugin": plugin_view(state, reg, &row).await })),
        )
            .into_response(),
        Err(resp) => resp,
    }
}

/// Append an audit row for a registry change. `change` names it; `detail` carries
/// ids, capability names and hosts only.
async fn audit(state: &AppState, admin: &str, id: &str, change: &str, mut detail: Value) {
    if let Some(map) = detail.as_object_mut() {
        map.insert("change".into(), json!(change));
    }
    v7_mount::append_plugin_audit(
        &state.store,
        admin,
        mw_admin::ActorKind::Admin,
        mw_admin::AuditKind::SecurityPolicyChanged,
        id,
        detail,
    )
    .await;
}

// ── request validation (unit-tested) ─────────────────────────────────────────

/// A plugin id: 1 to 64 of `a-z`, `0-9` and `-`, starting with a letter or digit.
/// It is also the component's file name, so nothing that could leave the plugin
/// directory is accepted.
fn valid_plugin_id(id: &str) -> bool {
    let ok_char = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-';
    (1..=64).contains(&id.len())
        && id.chars().all(ok_char)
        && id.chars().next().is_some_and(|c| c != '-')
}

/// Parse capability names against the closed set (`mw_plugin::Capability`). An
/// unknown or repeated name is an error naming it.
fn parse_capabilities(names: &[String]) -> Result<Vec<Capability>, String> {
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let cap: Capability = serde_json::from_value(Value::String(name.clone()))
            .map_err(|_| format!("unknown capability '{name}'"))?;
        if out.contains(&cap) {
            return Err(format!("capability '{name}' is listed twice"));
        }
        out.push(cap);
    }
    Ok(out)
}

/// The most hosts one plugin's `net_allowlist` may list.
const MAX_NET_ALLOWLIST: usize = 32;

/// Check a `net_allowlist`: each entry is a host name, an IPv4 address, or a
/// `*.suffix` wildcard over a name with at least two labels. No scheme, port, path,
/// IPv6 literal or bare `*`: `mw-plugin` compares entries with the URL's host only,
/// so anything else would never match or would match more than it reads as.
fn check_net_allowlist(hosts: &[String]) -> Result<Vec<String>, String> {
    if hosts.len() > MAX_NET_ALLOWLIST {
        return Err(format!(
            "netAllowlist lists more than {MAX_NET_ALLOWLIST} hosts"
        ));
    }
    let mut out = Vec::with_capacity(hosts.len());
    for raw in hosts {
        let entry = raw.trim().to_ascii_lowercase();
        let name = entry.strip_prefix("*.").unwrap_or(&entry);
        let wildcard = name.len() != entry.len();
        let label_ok = |l: &str| {
            !l.is_empty()
                && l.len() <= 63
                && l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                && !l.starts_with('-')
                && !l.ends_with('-')
        };
        let labels: Vec<&str> = name.split('.').collect();
        let numeric = labels.iter().all(|l| l.chars().all(|c| c.is_ascii_digit()));
        let ok = name.len() <= 253
            && labels.iter().all(|l| label_ok(l))
            && if wildcard {
                labels.len() >= 2 && !numeric
            } else {
                !numeric || name.parse::<std::net::Ipv4Addr>().is_ok()
            };
        if !ok {
            return Err(format!(
                "netAllowlist entry '{raw}' is not a host name, an IPv4 address or a *.suffix wildcard"
            ));
        }
        if !out.contains(&entry) {
            out.push(entry);
        }
    }
    Ok(out)
}

/// The ceilings a third-party manifest's limits must stay within.
const MAX_MEMORY_MB: u32 = 1024;
const MAX_DEADLINE_MS: u64 = 120_000;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LimitsReq {
    memory_mb: u32,
    deadline_ms: u64,
    #[serde(default)]
    fuel: Option<u64>,
}

/// The body of `POST /admin/plugins`.
///
/// A first-party id is registered by `id`, with `netAllowlist` optionally replacing
/// the compiled-in default hosts; every other key is refused, because that
/// manifest is compiled in. Any other id is a third-party plugin and gives its whole
/// manifest: `name`, `version` and `capabilities` are required.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RegisterReq {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    capabilities: Option<Vec<String>>,
    #[serde(default)]
    net_allowlist: Option<Vec<String>>,
    #[serde(default)]
    limits: Option<LimitsReq>,
    /// Hex-encoded detached Ed25519 signature over the component bytes.
    #[serde(default)]
    signature: Option<String>,
}

/// The manifest a registration asks for, or the refusal message. Checks everything
/// that needs no store and no file.
fn manifest_from(req: RegisterReq) -> Result<PluginManifest, String> {
    let id = req.id.trim().to_string();
    if !valid_plugin_id(&id) {
        return Err(
            "id must be 1 to 64 characters of a-z, 0-9 and '-', not starting with '-'".into(),
        );
    }
    let net_allowlist = req
        .net_allowlist
        .as_deref()
        .map(check_net_allowlist)
        .transpose()?;

    if v7_mount::is_first_party_plugin(&id) {
        let Some(mut manifest) = v7_mount::first_party_manifest(&id) else {
            return Err(format!(
                "'{id}' is an alias of a first-party component; register the component's own id"
            ));
        };
        if req.name.is_some()
            || req.version.is_some()
            || req.capabilities.is_some()
            || req.limits.is_some()
            || req.signature.is_some()
        {
            return Err(format!(
                "'{id}' is a first-party component: its manifest is compiled into the server \
                 and only netAllowlist may be given"
            ));
        }
        if let Some(hosts) = net_allowlist {
            manifest.net_allowlist = hosts;
        }
        return Ok(manifest);
    }

    let required = |value: Option<String>, what: &str| {
        value
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty() && v.len() <= 128)
            .ok_or_else(|| {
                format!("{what} is required for a third-party plugin (1 to 128 characters)")
            })
    };
    let name = required(req.name, "name")?;
    let version = required(req.version, "version")?;
    let capabilities = parse_capabilities(
        &req.capabilities
            .ok_or("capabilities is required for a third-party plugin")?,
    )?;
    let (_, refused) = v7_mount::provenance_filtered_grant(&id, &capabilities);
    if let Some(cap) = refused.first() {
        return Err(format!(
            "capability '{}' is available to first-party components only",
            v7_mount::capability_name(*cap)
        ));
    }
    let limits = match req.limits {
        None => PluginLimits::default(),
        Some(l) => {
            if !(1..=MAX_MEMORY_MB).contains(&l.memory_mb) {
                return Err(format!("limits.memoryMb must be 1 to {MAX_MEMORY_MB}"));
            }
            if !(1..=MAX_DEADLINE_MS).contains(&l.deadline_ms) {
                return Err(format!("limits.deadlineMs must be 1 to {MAX_DEADLINE_MS}"));
            }
            PluginLimits {
                memory_mb: l.memory_mb,
                deadline_ms: l.deadline_ms,
                fuel: l.fuel,
            }
        }
    };
    let signature = match req.signature {
        None => None,
        Some(sig) => {
            let sig = sig.trim().to_ascii_lowercase();
            if sig.len() != 128 || !sig.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(
                    "signature must be 128 hex characters (a 64-byte Ed25519 signature)".into(),
                );
            }
            Some(sig)
        }
    };
    Ok(PluginManifest {
        id,
        name,
        version,
        signature,
        capabilities,
        net_allowlist: net_allowlist.unwrap_or_default(),
        limits,
    })
}

// ── handlers ─────────────────────────────────────────────────────────────────

/// `GET /admin/plugins` — `{ "plugins": [ … ] }`, each entry a [`plugin_view`], in
/// id order.
async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    let rows = match state.store.list_plugins().await {
        Ok(rows) => rows,
        Err(e) => return store_error("list", &e),
    };
    let mut plugins = Vec::with_capacity(rows.len());
    for row in &rows {
        plugins.push(plugin_view(&state, &reg, row).await);
    }
    Json(json!({ "plugins": plugins })).into_response()
}

/// `POST /admin/plugins` — register a plugin ([`RegisterReq`]). The new row is
/// unapproved, disabled and holds no grant. Answers `201 { "plugin": … }`.
///
/// Refused, with nothing written:
/// - `400` — a malformed body, an unknown or repeated capability, a capability
///   reserved to first-party components, a manifest key given for a first-party id,
///   an unusable `netAllowlist` entry, limits or signature;
/// - `409 already-registered` — the id has a row (uninstall it first);
/// - `409 component-unavailable` — a first-party id with no component file matching
///   the compiled-in digest;
/// - `403 digest-not-approved` — any other id whose component file is missing from
///   the third-party directory or whose digest is not an active allowlist pin.
async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
    body: Result<Json<RegisterReq>, JsonRejection>,
) -> Response {
    let admin = match require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let manifest = match body {
        Ok(Json(req)) => match manifest_from(req) {
            Ok(m) => m,
            Err(message) => return bad_request(message),
        },
        Err(rejection) => return bad_request(rejection.body_text()),
    };
    let id = manifest.id.clone();
    match state.store.get_plugin(&id).await {
        Ok(None) => {}
        Ok(Some(_)) => {
            return refuse(
                StatusCode::CONFLICT,
                "already-registered",
                format!(
                    "plugin '{id}' is already registered; uninstall it before registering it again"
                ),
            );
        }
        Err(e) => return store_error("read", &e),
    }
    // The component must pass the same gate a load passes, now.
    let first_party = v7_mount::is_first_party_plugin(&id);
    if v7_mount::resolve_component(&id, &state.store)
        .await
        .is_none()
    {
        return if first_party {
            refuse(
                StatusCode::CONFLICT,
                "component-unavailable",
                format!(
                    "no component file for '{id}' matches the digest compiled into this server"
                ),
            )
        } else {
            refuse(
                StatusCode::FORBIDDEN,
                "digest-not-approved",
                format!(
                    "no component file for '{id}' with an approved digest was found in the \
                     third-party plugin directory"
                ),
            )
        };
    }
    let text = |v: serde_json::Result<String>, empty: &str| v.unwrap_or_else(|_| empty.into());
    let row = PluginRow {
        id: id.clone(),
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        signature_hex: manifest.signature.clone(),
        approved_by: None,
        enabled: false,
        capabilities_json: text(serde_json::to_string(&manifest.capabilities), "[]"),
        net_allowlist_json: text(serde_json::to_string(&manifest.net_allowlist), "[]"),
        limits_json: text(serde_json::to_string(&manifest.limits), "{}"),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    if let Err(e) = state.store.put_plugin(&row).await {
        return store_error("write", &e);
    }
    reg.lock()
        .expect("plugin registry lock")
        .register(manifest.clone());
    audit(
        &state,
        &admin,
        &id,
        "plugin-registered",
        json!({
            "firstParty": first_party,
            "version": manifest.version,
            "capabilities": manifest.capabilities,
            "netAllowlist": manifest.net_allowlist,
            "signed": manifest.signature.is_some(),
        }),
    )
    .await;
    applied(&state, &reg, &id, StatusCode::CREATED).await
}

/// `POST /admin/plugins/{id}/approve` — record the administrator's approval.
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
    if let Err(resp) = row_of(&state, &id).await {
        return resp;
    }
    if let Err(e) = state.store.set_plugin_approved(&id, &admin).await {
        return store_error("approve", &e);
    }
    let _ = reg
        .lock()
        .expect("plugin registry lock")
        .approve(&id, &admin);
    audit(&state, &admin, &id, "plugin-approved", json!({})).await;
    applied(&state, &reg, &id, StatusCode::OK).await
}

/// `POST /admin/plugins/{id}/enable` — enable an approved plugin. `400
/// not-approved` before approval; `403 unsigned-not-allowed` for an unsigned plugin
/// whose trust policy does not admit that (a third-party plugin without the stored
/// allow-unsigned flag). The answer's `loaded` says whether it is now running.
async fn enable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let admin = match require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let row = match row_of(&state, &id).await {
        Ok(row) => row,
        Err(resp) => return resp,
    };
    if row.approved_by.is_none() {
        return refuse(
            StatusCode::BAD_REQUEST,
            "not-approved",
            "plugin must be approved before it can be enabled",
        );
    }
    let trust = TrustPolicy::of(&state.store, &id).await;
    if row.signature_hex.is_none() && !trust.admits_unsigned() {
        return refuse(
            StatusCode::FORBIDDEN,
            "unsigned-not-allowed",
            "this plugin is unsigned and has not been allowed to run unsigned",
        );
    }
    if let Err(e) = state.store.set_plugin_enabled(&id, true).await {
        return store_error("enable", &e);
    }
    let _ = reg.lock().expect("plugin registry lock").enable(&id);
    audit(
        &state,
        &admin,
        &id,
        "plugin-enabled",
        json!({ "trust": trust.wire() }),
    )
    .await;
    applied(&state, &reg, &id, StatusCode::OK).await
}

/// `POST /admin/plugins/{id}/disable` — disable a plugin. A loaded spam classifier
/// is unloaded before this answers.
async fn disable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let admin = match require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    if let Err(resp) = row_of(&state, &id).await {
        return resp;
    }
    if let Err(e) = state.store.set_plugin_enabled(&id, false).await {
        return store_error("disable", &e);
    }
    let _ = reg.lock().expect("plugin registry lock").disable(&id);
    audit(&state, &admin, &id, "plugin-disabled", json!({})).await;
    applied(&state, &reg, &id, StatusCode::OK).await
}

/// The body of `POST /admin/plugins/{id}/grant`: the complete capability set for
/// one scope. `accountId` absent or `null` ⇒ the deployment-wide scope.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GrantReq {
    #[serde(default)]
    account_id: Option<String>,
    capabilities: Vec<String>,
}

/// The capabilities a grant request may store, or the refusal message: every name
/// is in the closed set, declared by the manifest, and allowed to this plugin's
/// provenance.
fn grantable(manifest: &PluginManifest, requested: &[String]) -> Result<Vec<Capability>, String> {
    let caps = parse_capabilities(requested)?;
    if let Some(cap) = caps.iter().find(|c| !manifest.capabilities.contains(c)) {
        return Err(format!(
            "capability '{}' is not declared by this plugin's manifest",
            v7_mount::capability_name(*cap)
        ));
    }
    let (_, refused) = v7_mount::provenance_filtered_grant(&manifest.id, &caps);
    if let Some(cap) = refused.first() {
        return Err(format!(
            "capability '{}' is available to first-party components only",
            v7_mount::capability_name(*cap)
        ));
    }
    Ok(caps)
}

/// `POST /admin/plugins/{id}/grant` — set the capabilities granted in one scope
/// ([`GrantReq`]). The stored rows for that scope are replaced, so leaving a
/// capability out revokes it and an empty list revokes them all. `400`, with
/// nothing written, for a name outside the closed set, a capability the manifest
/// does not declare, or one reserved to first-party components.
async fn grant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
    UrlPath(id): UrlPath<String>,
    body: Result<Json<GrantReq>, JsonRejection>,
) -> Response {
    let admin = match require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let req = match body {
        Ok(Json(req)) => req,
        Err(rejection) => return bad_request(rejection.body_text()),
    };
    let row = match row_of(&state, &id).await {
        Ok(row) => row,
        Err(resp) => return resp,
    };
    let caps = match grantable(&v7_mount::manifest_of(&row), &req.capabilities) {
        Ok(caps) => caps,
        Err(message) => return bad_request(message),
    };
    let account_id = req.account_id.unwrap_or_default();
    let names: Vec<String> = caps.iter().map(|c| v7_mount::capability_name(*c)).collect();
    if let Err(e) = state
        .store
        .replace_plugin_grants(&id, &account_id, &names, &admin)
        .await
    {
        return store_error("grant", &e);
    }
    audit(
        &state,
        &admin,
        &id,
        "plugin-granted",
        json!({ "capabilities": names, "accountScoped": !account_id.is_empty() }),
    )
    .await;
    v7_mount::sync_spam_classifier(&reg, &state.store).await;
    let row = match row_of(&state, &id).await {
        Ok(row) => row,
        Err(resp) => return resp,
    };
    Json(json!({
        "pluginId": id,
        "accountId": if account_id.is_empty() { Value::Null } else { json!(account_id) },
        "granted": names,
        "plugin": plugin_view(&state, &reg, &row).await,
    }))
    .into_response()
}

/// The body of `POST /admin/plugins/{id}/allow-unsigned`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AllowUnsignedReq {
    allow: bool,
}

/// `POST /admin/plugins/{id}/allow-unsigned` — set (`{"allow": true}`) or clear the
/// stored flag that lets a third-party plugin without a signature be enabled and
/// loaded. It changes nothing else: the plugin is not enabled by it. Clearing it
/// unloads a running unsigned spam classifier. `400 first-party` for a first-party
/// id, which is trusted by its digest pin and has no such flag.
async fn allow_unsigned(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
    UrlPath(id): UrlPath<String>,
    body: Result<Json<AllowUnsignedReq>, JsonRejection>,
) -> Response {
    let admin = match require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let allow = match body {
        Ok(Json(req)) => req.allow,
        Err(rejection) => return bad_request(rejection.body_text()),
    };
    if let Err(resp) = row_of(&state, &id).await {
        return resp;
    }
    if v7_mount::is_first_party_plugin(&id) {
        return refuse(
            StatusCode::BAD_REQUEST,
            "first-party",
            "a first-party component is trusted by the digest compiled into the server; \
             the allow-unsigned flag does not apply to it",
        );
    }
    if let Err(e) = state.store.set_plugin_allow_unsigned(&id, allow).await {
        return store_error("allow-unsigned", &e);
    }
    audit(
        &state,
        &admin,
        &id,
        "plugin-allow-unsigned",
        json!({ "allow": allow }),
    )
    .await;
    applied(&state, &reg, &id, StatusCode::OK).await
}

/// The body of `POST /admin/plugins/{id}/settings`: `endpoint` is the daemon
/// address, or `null` to go back to the component's built-in default.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsReq {
    endpoint: Option<String>,
}

/// `POST /admin/plugins/{id}/settings` — set where a spam classifier finds its
/// daemon ([`SettingsReq`]). The value is written to the plugin's deployment-wide
/// scoped KV under `endpoint`, which the component reads on every message when it
/// holds the `store-kv-scoped` grant; the host named there must also be in the
/// plugin's `netAllowlist`. `400` for a plugin that takes no endpoint.
async fn settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
    UrlPath(id): UrlPath<String>,
    body: Result<Json<SettingsReq>, JsonRejection>,
) -> Response {
    let admin = match require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let endpoint = match body {
        Ok(Json(req)) => req.endpoint.map(|e| e.trim().to_string()),
        Err(rejection) => return bad_request(rejection.body_text()),
    };
    let row = match row_of(&state, &id).await {
        Ok(row) => row,
        Err(resp) => return resp,
    };
    if !takes_endpoint(&v7_mount::manifest_of(&row)) {
        return bad_request("this plugin has no endpoint setting");
    }
    match &endpoint {
        Some(value) => {
            let usable = !value.is_empty()
                && value.len() <= 255
                && value.chars().all(|c| c.is_ascii_graphic());
            if !usable {
                return bad_request(
                    "endpoint must be 1 to 255 printable ASCII characters, or null",
                );
            }
            if let Err(e) = state
                .store
                .plugin_kv_set(
                    &id,
                    "",
                    ENDPOINT_KEY,
                    value.as_bytes(),
                    &v7_mount::plugin_kv_limits(),
                )
                .await
            {
                return store_error("settings", &e);
            }
        }
        None => {
            if let Err(e) = state.store.plugin_kv_delete(&id, "", ENDPOINT_KEY).await {
                return store_error("settings", &e);
            }
        }
    }
    audit(
        &state,
        &admin,
        &id,
        "plugin-settings",
        json!({ "endpoint": endpoint }),
    )
    .await;
    applied(&state, &reg, &id, StatusCode::OK).await
}

/// `POST /admin/plugins/{id}/test` — hand the loaded spam classifier a fixed test
/// message, through the same call ingest makes, and answer
/// `{ "verdict": "spam" | "ham" | "unknown", "detail": <the component's answer> }`.
/// `409 not-loaded` (with `notLoadedReason`) when `id` is not the loaded
/// classifier; `502 classifier-error` when the call itself failed.
async fn test_classifier(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
    UrlPath(id): UrlPath<String>,
) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    let row = match row_of(&state, &id).await {
        Ok(row) => row,
        Err(resp) => return resp,
    };
    match v7_mount::probe_spam_classifier(&reg, &id, TEST_MESSAGE.to_vec()).await {
        None => {
            let status = v7_mount::plugin_status(&state.store, &reg, &row).await;
            (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "plugin is not the loaded spam classifier",
                    "code": "not-loaded",
                    "notLoadedReason": status.not_loaded.map(v7_mount::NotLoaded::wire),
                })),
            )
                .into_response()
        }
        Some(Err(e)) => refuse(StatusCode::BAD_GATEWAY, "classifier-error", e),
        Some(Ok(envelope)) => {
            let detail: Value =
                serde_json::from_str(&envelope).unwrap_or(Value::String(envelope.clone()));
            let verdict = match detail.get("verdict").and_then(Value::as_str) {
                Some("spam") => "spam",
                Some("ham") => "ham",
                _ => "unknown",
            };
            Json(json!({ "verdict": verdict, "detail": detail })).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn register_req(body: Value) -> Result<PluginManifest, String> {
        manifest_from(serde_json::from_value::<RegisterReq>(body).map_err(|e| e.to_string())?)
    }

    #[test]
    fn capability_names_are_a_closed_set() {
        assert_eq!(
            parse_capabilities(&names(&["spam-action", "net"])).unwrap(),
            vec![Capability::SpamAction, Capability::Net]
        );
        for bad in ["Net", "network", "", "spam_action", "*"] {
            let err = parse_capabilities(&names(&[bad])).unwrap_err();
            assert!(err.contains("unknown capability"), "{bad:?}: {err}");
        }
        assert!(
            parse_capabilities(&names(&["net", "net"]))
                .unwrap_err()
                .contains("twice")
        );
    }

    #[test]
    fn a_first_party_registration_takes_the_compiled_in_manifest() {
        let m = register_req(json!({ "id": "spam-rspamd" })).unwrap();
        assert_eq!(
            m.capabilities,
            vec![
                Capability::SpamAction,
                Capability::Net,
                Capability::StoreKvScoped
            ]
        );
        assert_eq!(m.net_allowlist, vec!["rspamd".to_string()]);
        assert!(m.signature.is_none());

        // The deployment's hosts replace the default ones.
        let m = register_req(json!({ "id": "spam-rspamd", "netAllowlist": ["Scan.Internal"] }))
            .unwrap();
        assert_eq!(m.net_allowlist, vec!["scan.internal".to_string()]);

        // Nothing else about a first-party manifest can be supplied.
        for extra in [
            json!({ "capabilities": ["account-backend"] }),
            json!({ "name": "x" }),
            json!({ "version": "9" }),
            json!({ "limits": { "memoryMb": 512, "deadlineMs": 1000 } }),
            json!({ "signature": "ab".repeat(64) }),
        ] {
            let mut body = json!({ "id": "spam-rspamd" });
            body.as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            let err = register_req(body).unwrap_err();
            assert!(err.contains("first-party component"), "{extra}: {err}");
        }
        assert!(
            register_req(json!({ "id": "nextcloud-plugin" }))
                .unwrap_err()
                .contains("alias")
        );
    }

    #[test]
    fn a_third_party_registration_is_checked_before_anything_is_stored() {
        let base =
            || json!({ "id": "acme", "name": "Acme", "version": "1", "capabilities": ["net"] });
        let with = |key: &str, value: Value| {
            let mut body = base();
            body[key] = value;
            register_req(body)
        };
        let ok = register_req(base()).unwrap();
        assert_eq!(ok.capabilities, vec![Capability::Net]);
        assert!(ok.net_allowlist.is_empty());
        assert_eq!(ok.limits, PluginLimits::default());

        assert!(
            with("capabilities", json!(["net", "root"]))
                .unwrap_err()
                .contains("unknown capability 'root'")
        );
        assert!(
            with("capabilities", json!(["account-backend"]))
                .unwrap_err()
                .contains("first-party components only")
        );
        assert!(with("signature", json!("abc")).is_err());
        assert!(with("limits", json!({ "memoryMb": 0, "deadlineMs": 10 })).is_err());
        assert!(with("limits", json!({ "memoryMb": 64, "deadlineMs": 999_999 })).is_err());
        assert!(with("unknownKey", json!(1)).is_err());
        for id in ["", "-x", "Acme", "a/b", "..", "a.b", "a b"] {
            assert!(with("id", json!(id)).is_err(), "{id:?} accepted");
        }
        let mut no_caps = base();
        no_caps.as_object_mut().unwrap().remove("capabilities");
        assert!(register_req(no_caps).unwrap_err().contains("capabilities"));
    }

    #[test]
    fn net_allowlist_entries_are_hosts() {
        assert_eq!(
            check_net_allowlist(&names(&[
                "rspamd",
                "API.example.org",
                "*.example.org",
                "127.0.0.1",
                "rspamd"
            ]))
            .unwrap(),
            names(&["rspamd", "api.example.org", "*.example.org", "127.0.0.1"])
        );
        for bad in [
            "",
            "*",
            "*.com",
            "*.0.0.1",
            "http://rspamd",
            "rspamd:11333",
            "rspamd/path",
            "[::1]",
            "::1",
            "a b",
            "999.1",
            "-x.example",
            "user@host",
        ] {
            assert!(
                check_net_allowlist(&names(&[bad])).is_err(),
                "{bad:?} accepted"
            );
        }
        let many: Vec<String> = (0..=MAX_NET_ALLOWLIST).map(|i| format!("h{i}")).collect();
        assert!(check_net_allowlist(&many).is_err());
    }

    #[test]
    fn a_grant_cannot_exceed_the_manifest_or_the_provenance() {
        let rspamd = v7_mount::first_party_manifest("spam-rspamd").unwrap();
        assert_eq!(
            grantable(&rspamd, &names(&["spam-action"])).unwrap(),
            vec![Capability::SpamAction]
        );
        assert!(grantable(&rspamd, &[]).unwrap().is_empty());
        assert!(
            grantable(&rspamd, &names(&["spam-action", "account-backend"]))
                .unwrap_err()
                .contains("not declared")
        );
        assert!(grantable(&rspamd, &names(&["everything"])).is_err());

        // A third-party manifest that somehow declares a HIGH_POWER capability still
        // cannot be granted it.
        let third = PluginManifest {
            id: "acme".into(),
            name: "Acme".into(),
            version: "1".into(),
            signature: None,
            capabilities: vec![Capability::AccountBackend, Capability::Net],
            net_allowlist: vec![],
            limits: PluginLimits::default(),
        };
        assert!(
            grantable(&third, &names(&["account-backend"]))
                .unwrap_err()
                .contains("first-party components only")
        );
        assert_eq!(
            grantable(&third, &names(&["net"])).unwrap(),
            vec![Capability::Net]
        );
    }
}
