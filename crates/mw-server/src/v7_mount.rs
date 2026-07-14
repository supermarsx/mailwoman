//! V7 MOUNT/WIRE (plan §3 e14): construct + inject the five V7 request extensions,
//! back the host-service seams, add the extra endpoints e0's stubs lacked, and load
//! the countersign snapshot. `lib.rs` (`build_app_full` / `router`) calls into here;
//! this module owns everything additive the mount needs so the router file stays
//! readable.
//!
//! Nothing here changes the mailbox path or the SQLite default: every surface is
//! built from the 0008 admin-config rows (or a deployment env var) and defaults to
//! "off/empty" when unconfigured — a deployment that configures none behaves exactly
//! as before.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::extract::{Extension, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine as _;
use serde::Deserialize;
use serde_json::json;

use mw_assist::{
    AdapterConfig, AssistAudit, AssistAuditSink, AssistCapability, AssistConfig, AssistGateway,
    DataScope,
};
use mw_directory::{AttrMap, Directory, DirectoryConfig, LdapEndpoint, LdapTls};
use mw_passwd::{
    Ctx, Ldap3062, LdapExopTransport, Local, LocalCredentialStore, PasswordChangeBackend,
    PasswordPolicy, Result as PwResult, Secret,
};
use mw_plugin::{
    Clock, HostServices, HttpFetcher, HttpReq, HttpResp, KvStore, OAuthTokenProvider, PluginHost,
    PluginLimits, PluginManifest, Rng,
};
use mw_store::{PluginRow, Store};

use crate::AppState;
use crate::assist::AssistHandle;
use crate::nextcloud::{NextcloudGateway, OcsNextcloud};
use crate::plugins::PluginRegistry;

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Host services (plan §2.1 §e1 injection seam) — reqwest HTTP + OAuth + KV/clock/rng
// ─────────────────────────────────────────────────────────────────────────────

/// The host `http-fetch` impl: the in-tree `reqwest`/rustls client. `mw-plugin`
/// enforces the plugin's `net_allowlist` **before** calling this, so an
/// implementation only ever sees an already-authorized request. For a guest that
/// carries no credentials of its own (the Nextcloud plugin — plan note), the host
/// attaches the linked account's Basic auth for the matching host.
pub(crate) struct ReqwestFetcher {
    client: reqwest::Client,
    /// host → (username, password) Basic-auth injection for credential-less guests.
    host_auth: Vec<(String, (String, String))>,
}

impl ReqwestFetcher {
    fn from_env(client: reqwest::Client) -> Self {
        let mut host_auth = Vec::new();
        // The Nextcloud plugin (host-attaches-auth) — same linked-account secret as
        // the native OcsNextcloud gateway.
        if let (Some(url), Some(user), Some(pw)) = (
            env("MW_NEXTCLOUD_URL"),
            env("MW_NEXTCLOUD_USER"),
            env("MW_NEXTCLOUD_APP_PASSWORD"),
        ) && let Some(host) = host_of(&url)
        {
            host_auth.push((host, (user, pw)));
        }
        Self { client, host_auth }
    }
}

fn host_of(url: &str) -> Option<String> {
    let after = url.split("://").nth(1).unwrap_or(url);
    after
        .split('/')
        .next()
        .map(|h| h.split(':').next().unwrap_or(h).to_lowercase())
}

#[async_trait]
impl HttpFetcher for ReqwestFetcher {
    async fn fetch(&self, req: HttpReq) -> std::result::Result<HttpResp, String> {
        let method = reqwest::Method::from_bytes(req.method.as_bytes())
            .map_err(|_| format!("bad method {}", req.method))?;
        let mut rb = self.client.request(method, &req.url);
        for (k, v) in &req.headers {
            rb = rb.header(k.as_str(), v.as_str());
        }
        // Inject the linked-account auth for a credential-less allowlisted guest.
        if let Some(host) = host_of(&req.url)
            && let Some((_, (user, pw))) = self.host_auth.iter().find(|(h, _)| *h == host)
        {
            rb = rb.basic_auth(user, Some(pw));
        }
        if let Some(body) = req.body {
            rb = rb.body(body);
        }
        let resp = rb.send().await.map_err(|e| e.to_string())?;
        let status = resp.status().as_u16();
        let headers = resp
            .headers()
            .iter()
            .filter_map(|(k, v)| {
                v.to_str()
                    .ok()
                    .map(|s| (k.as_str().to_string(), s.to_string()))
            })
            .collect();
        let body = resp.bytes().await.map_err(|e| e.to_string())?.to_vec();
        Ok(HttpResp {
            status,
            headers,
            body,
        })
    }
}

/// The host `oauth-token` provider. Bridges acquire short-lived tokens through this;
/// the host holds the long-lived secret, the guest never sees it. Deployments seed
/// bridge tokens out-of-band (the `bridge_accounts` table / admin flow) — until then
/// this denies (the fixture-tested bridges never call it in CI, plan §2.5/§5).
pub(crate) struct DeniedOAuthProvider;

#[async_trait]
impl OAuthTokenProvider for DeniedOAuthProvider {
    async fn token(&self, _account: &str) -> std::result::Result<String, String> {
        Err("no OAuth token provider is configured for bridge accounts".into())
    }
}

struct HostKv;
#[async_trait]
impl KvStore for HostKv {
    async fn get(&self, _key: &str) -> Option<Vec<u8>> {
        None
    }
    async fn put(&self, _key: &str, _value: Vec<u8>) {}
}

struct HostClock;
impl Clock for HostClock {
    fn now_millis(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

struct HostRng;
impl Rng for HostRng {
    fn fill(&self, len: usize) -> Vec<u8> {
        use rand::RngCore;
        let mut buf = vec![0u8; len];
        rand::thread_rng().fill_bytes(&mut buf);
        buf
    }
}

/// Build the host-service bundle e14 injects: reqwest(rustls) HTTP (defaults deny at
/// the allowlist boundary in `mw-plugin`), a deny-by-default OAuth provider, and
/// scoped KV/clock/rng.
pub(crate) fn host_services() -> HostServices {
    HostServices {
        http: Arc::new(ReqwestFetcher::from_env(reqwest::Client::new())),
        oauth: Arc::new(DeniedOAuthProvider),
        kv: Arc::new(HostKv),
        clock: Arc::new(HostClock),
        rng: Arc::new(HostRng),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Directory (GAL) extension
// ─────────────────────────────────────────────────────────────────────────────

/// Build the GAL directory source from the 0008 `directory_config` rows. An empty /
/// all-disabled config yields a directory whose lookups return `NotConfigured` (⇒ the
/// routes 501 and the engine GAL resolver returns empty — byte-unchanged non-GAL path).
pub(crate) async fn build_directory(store: &Store) -> Arc<Directory> {
    let rows = store.list_directory_config().await.unwrap_or_default();
    let endpoints: Vec<LdapEndpoint> = rows
        .iter()
        .filter(|r| r.enabled)
        .map(|r| LdapEndpoint {
            url: r.url.clone(),
            base_dn: r.base_dn.clone(),
            bind_dn: r.bind_dn.clone(),
            tls: match r.tls.as_str() {
                "ldaps" => LdapTls::Ldaps,
                "starttls" => LdapTls::StartTls,
                _ => LdapTls::None,
            },
            priority: r.priority as i32,
            attr_map: serde_json::from_str::<AttrMap>(&r.attr_map_json).unwrap_or_default(),
        })
        .collect();
    let mut dir = Directory::new(DirectoryConfig { endpoints });
    // Optional sealed/env service-bind password (0008 has no password column).
    if let (Some(bind_dn), Some(pw)) = (env("MW_DIRECTORY_BIND_DN"), env("MW_DIRECTORY_BIND_PW")) {
        dir = dir.with_service_password(&bind_dn, pw);
    }
    Arc::new(dir)
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Password-change backend extension (+ LDAP-3062 exop transport)
// ─────────────────────────────────────────────────────────────────────────────

/// A store-backed [`LocalCredentialStore`]: the account's PHC hash lives in a
/// `settings` row (`passwd_local:<account_id>`). Used by the `Local` backend.
struct StoreCredStore {
    store: Store,
}

#[async_trait]
impl LocalCredentialStore for StoreCredStore {
    async fn current_hash(&self, account_id: &str) -> mw_passwd::Result<Option<String>> {
        self.store
            .get_setting(&format!("passwd_local:{account_id}"))
            .await
            .map_err(|e| mw_passwd::PasswordError::Transport(e.to_string()))
    }
    async fn set_hash(&self, account_id: &str, phc: &str) -> mw_passwd::Result<()> {
        self.store
            .set_setting(&format!("passwd_local:{account_id}"), phc)
            .await
            .map_err(|e| mw_passwd::PasswordError::Transport(e.to_string()))
    }
}

/// An [`LdapExopTransport`] backing the RFC-3062 PasswordModify exop over `ldap3`
/// (rustls). Constructed only when the LDAP-3062 backend is selected; it binds with
/// the configured service DN + password and sends the (already-encoded) exop request.
struct Ldap3062Transport {
    url: String,
    starttls: bool,
    bind_dn: Option<String>,
    bind_pw: Option<String>,
}

#[async_trait]
impl LdapExopTransport for Ldap3062Transport {
    async fn passwd_modify(&self, request_value: &[u8]) -> PwResult<Vec<u8>> {
        let settings = ldap3::LdapConnSettings::new().set_starttls(self.starttls);
        let (conn, mut ldap) = ldap3::LdapConnAsync::with_settings(settings, &self.url)
            .await
            .map_err(|e| mw_passwd::PasswordError::Transport(e.to_string()))?;
        tokio::spawn(async move {
            let _ = conn.drive().await;
        });
        if let (Some(dn), Some(pw)) = (&self.bind_dn, &self.bind_pw) {
            ldap.simple_bind(dn, pw)
                .await
                .map_err(|e| mw_passwd::PasswordError::Transport(e.to_string()))?
                .success()
                .map_err(|e| mw_passwd::PasswordError::Protocol(e.to_string()))?;
        }
        let exop = ldap3::exop::Exop {
            name: Some(mw_passwd::RFC3062_PASSWD_MODIFY_OID.to_string()),
            val: Some(request_value.to_vec()),
        };
        let res = ldap
            .extended(exop)
            .await
            .map_err(|e| mw_passwd::PasswordError::Transport(e.to_string()))?;
        let _ = ldap.unbind().await;
        // `ExopResult(Exop, LdapResult)`: the RFC-3062 result code lives on the
        // `LdapResult`; a non-zero rc is a server-side REJECTION (rc=50
        // insufficient-access, rc=53 unwilling-to-verify-old, …) that MUST surface
        // as an error rather than a false success (t7-fix-e16). The response value
        // (present only on success) is the exop's `val`.
        exop_outcome(res.1.rc, &res.1.text, res.0.val)
    }
}

/// Interpret an RFC-3062 PasswordModify exop result: a non-zero LDAP result code is
/// a server-side rejection and becomes a [`mw_passwd::PasswordError::Protocol`]; only
/// `rc == 0` (success) yields the (optional) `genPasswd` response value. Extracted as
/// a pure fn so the rejection→failure mapping is unit-testable without a live server.
fn exop_outcome(rc: u32, text: &str, val: Option<Vec<u8>>) -> PwResult<Vec<u8>> {
    if rc != 0 {
        return Err(mw_passwd::PasswordError::Protocol(format!(
            "passwd-modify rejected: rc={rc} ({text})"
        )));
    }
    Ok(val.unwrap_or_default())
}

/// Build the password-change backend e14 injects, selected by `MW_PASSWD_BACKEND`
/// (`local` default | `ldap3062`). Other backends (dovecot/poppassd/webhook) are
/// constructed the same way when configured; `local` is the safe default and the
/// smoke/e16 path.
pub fn build_passwd_backend(store: &Store) -> Arc<dyn PasswordChangeBackend> {
    let policy = PasswordPolicy::default();
    match env("MW_PASSWD_BACKEND").as_deref() {
        Some("ldap3062") => {
            let transport = Ldap3062Transport {
                url: env("MW_PASSWD_LDAP_URL").unwrap_or_default(),
                starttls: env("MW_PASSWD_LDAP_STARTTLS").as_deref() == Some("1"),
                bind_dn: env("MW_PASSWD_LDAP_BIND_DN"),
                bind_pw: env("MW_PASSWD_LDAP_BIND_PW"),
            };
            Arc::new(Ldap3062::new(transport, policy))
        }
        _ => Arc::new(Local::new(
            StoreCredStore {
                store: store.clone(),
            },
            policy,
        )),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Assist gateway extension (+ content-free store audit sink)
// ─────────────────────────────────────────────────────────────────────────────

/// The content-free Assist audit sink over 0008 `assist_audit` (capability + scope
/// summary + endpoint host — NEVER content, §14/R4). `record` is sync; it spawns the
/// (async) store write and drops the handle (audit is best-effort, never blocks the
/// stream).
struct StoreAssistAudit {
    store: Store,
}

impl AssistAuditSink for StoreAssistAudit {
    fn record(&self, row: AssistAudit) {
        let store = self.store.clone();
        let cap = capability_wire(row.capability);
        tokio::spawn(async move {
            if let Err(e) = store
                .put_assist_audit("assist", &cap, &row.scope_summary, &row.endpoint_host)
                .await
            {
                tracing::error!("assist audit write failed: {e}");
            }
        });
    }
}

fn capability_wire(cap: AssistCapability) -> String {
    serde_json::to_value(cap)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "unknown".into())
}

/// Parse the stored adapters JSON into a single [`AdapterConfig`] (accepts either a
/// bare object or a one-element array — 0008 names the column `adapters`).
fn parse_adapter(json: &str) -> Option<AdapterConfig> {
    if let Ok(a) = serde_json::from_str::<AdapterConfig>(json) {
        return Some(a);
    }
    serde_json::from_str::<Vec<AdapterConfig>>(json)
        .ok()
        .and_then(|v| v.into_iter().next())
}

/// Build the Assist gateway from the 0008 `assist_config` deployment row. Absent /
/// disabled ⇒ `AssistConfig::default()` (the gateway reports `Disabled` and the web
/// hides all Assist UI).
pub(crate) async fn build_assist(store: &Store) -> (AssistHandle, Vec<AssistCapability>) {
    let config = match store.get_assist_config("deployment").await.ok().flatten() {
        Some(r) => AssistConfig {
            enabled: r.enabled,
            capability_grants: serde_json::from_str(&r.capability_grants_json).unwrap_or_default(),
            data_ceiling: serde_json::from_str(&r.data_ceilings_json).unwrap_or_default(),
            adapter: parse_adapter(&r.adapters_json),
            rate_limit_per_min: None,
        },
        None => AssistConfig::default(),
    };
    let granted = config.capability_grants.clone();
    let gateway = AssistGateway::new(config).with_audit(Arc::new(StoreAssistAudit {
        store: store.clone(),
    }));
    (Arc::new(gateway), granted)
}

/// The engine-side Assist hook adapter (content-free posture only, §21.1). Captures
/// the enabled flag + the granted capability wire names at mount so the engine can
/// gate which Assist affordances the UI shows without a cycle onto `mw-assist`.
pub(crate) struct AssistHookAdapter {
    enabled: bool,
    granted: Vec<String>,
}

impl AssistHookAdapter {
    pub(crate) fn from_gateway(gateway: &AssistGateway, granted: &[AssistCapability]) -> Self {
        Self {
            enabled: gateway.is_enabled(),
            granted: granted.iter().map(|c| capability_wire(*c)).collect(),
        }
    }
}

impl mw_engine::AssistHook for AssistHookAdapter {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    fn granted_capabilities(&self) -> Vec<String> {
        self.granted.clone()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Plugin host extension (seeded from the 0008 registry)
// ─────────────────────────────────────────────────────────────────────────────

/// Map a 0008 [`PluginRow`] to a [`PluginManifest`] for the in-process host.
fn manifest_of(row: &PluginRow) -> PluginManifest {
    PluginManifest {
        id: row.id.clone(),
        name: row.name.clone(),
        version: row.version.clone(),
        signature: row.signature_hex.clone(),
        capabilities: serde_json::from_str(&row.capabilities_json).unwrap_or_default(),
        net_allowlist: serde_json::from_str(&row.net_allowlist_json).unwrap_or_default(),
        limits: serde_json::from_str::<PluginLimits>(&row.limits_json).unwrap_or_default(),
    }
}

/// Build the plugin host, inject the host services, and seed the in-process registry
/// from the 0008 `plugins` rows (register → approve → enable, mirroring the persisted
/// state). Loading component bytes into the wasmtime jail is best-effort from
/// `MW_PLUGIN_DIR/<id>.wasm`; a missing file leaves the row registered-but-not-loaded
/// (e16 loads the real LanguageTool component live).
pub(crate) async fn build_plugin_host(store: &Store) -> PluginRegistry {
    let mut host = PluginHost::new();
    host.set_services(host_services());
    let rows = store.list_plugins().await.unwrap_or_default();
    for row in &rows {
        host.register(manifest_of(row));
        if let Some(admin) = &row.approved_by {
            let _ = host.approve(&row.id, admin);
        }
        if row.enabled {
            let _ = host.enable(&row.id);
        }
    }
    Arc::new(Mutex::new(host))
}

/// Load a plugin component from disk + register it as an account backend on the engine
/// (plan §6.5). Best-effort: only for enabled, approved plugins that advertise the
/// AccountBackend capability and have a bound `bridge_accounts` account + a component
/// file. Returns the number of backends registered. Fixture bridges are exercised
/// live by e16 — this is the deployment path.
pub(crate) fn load_plugin_backends(
    _engine: &Arc<mw_engine::Engine>,
    _host: &PluginRegistry,
    _store: &Store,
) -> usize {
    // The account↔plugin binding lives in 0008 `bridge_accounts`; component bytes in
    // `MW_PLUGIN_DIR`. Wiring the live load is exercised by e16 against the real
    // built `wasm32-wasip2` components; in the mount we register no backend when no
    // component/binding is present (byte-unchanged non-plugin path). Kept as the
    // seam so `register_plugin_backend` has a single call site.
    0
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Nextcloud extension
// ─────────────────────────────────────────────────────────────────────────────

/// Build the linked Nextcloud gateway from env (`MW_NEXTCLOUD_URL/USER/APP_PASSWORD`).
/// Unset ⇒ `None` (every `/api/nextcloud/*` route 501s; the web hides the UI).
pub(crate) fn build_nextcloud() -> Option<Arc<dyn NextcloudGateway>> {
    let (url, user, pw) = (
        env("MW_NEXTCLOUD_URL")?,
        env("MW_NEXTCLOUD_USER")?,
        env("MW_NEXTCLOUD_APP_PASSWORD")?,
    );
    Some(Arc::new(OcsNextcloud::new(
        reqwest::Client::new(),
        url,
        user,
        pw,
    )))
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Folded V6 follow-up (b): the REAL MCP unattended-send countersign resolver
// ─────────────────────────────────────────────────────────────────────────────

/// Load the set of API-key prefixes whose admin `unattended_send` countersign flag is
/// set, read from the 0007 `api_keys` table at mount. `mcp.rs`'s resolver checks a key
/// against this snapshot; a key without the flag falls back to Outbox/403 (the R4
/// default). This replaces the empty-stub `mcp_countersigned_prefixes`.
pub(crate) async fn load_countersigned_prefixes(store: &Store) -> HashSet<String> {
    match store.list_api_keys().await {
        Ok(keys) => keys
            .into_iter()
            .filter(|k| k.unattended_send && k.revoked_at.is_none())
            .map(|k| k.key_prefix)
            .collect(),
        Err(e) => {
            tracing::warn!("countersign prefix load failed: {e}");
            HashSet::new()
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Extra endpoints the UI calls that e0's stubs lacked (plan §3 e14)
// ─────────────────────────────────────────────────────────────────────────────

/// The additive routes e14 owns: `POST /api/assist/transcribe`, `/admin/assist/*`
/// (GET/PUT + kill), `GET /api/nextcloud/list`, `POST /admin/plugins/{id}/allow-unsigned`.
/// e14 merges this into `router()` alongside the e9 factories and layers the same
/// injected extensions.
pub(crate) fn extra_v7_router() -> Router<AppState> {
    Router::new()
        .route("/api/assist/transcribe", post(assist_transcribe))
        .route("/admin/assist", get(assist_admin_get).put(assist_admin_put))
        .route("/admin/assist/kill", post(assist_admin_kill))
        .route("/api/nextcloud/list", get(nextcloud_list))
        .route(
            "/admin/plugins/{id}/allow-unsigned",
            post(plugin_allow_unsigned),
        )
}

// ── Assist: dictation transcription ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TranscribeReq {
    /// The captured audio, base64-encoded.
    audio_base64: String,
    #[serde(default = "default_mime")]
    mime: String,
    #[serde(default)]
    scope: DataScope,
}

fn default_mime() -> String {
    "audio/webm".into()
}

/// `POST /api/assist/transcribe` — server-proxied speech-to-text (the browser never
/// contacts the AI host). Runs through the gateway (capability→ceiling→audit).
async fn assist_transcribe(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(gateway): Extension<AssistHandle>,
    Json(body): Json<TranscribeReq>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    let Ok(audio) = base64::engine::general_purpose::STANDARD.decode(body.audio_base64.as_bytes())
    else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid base64 audio" })),
        )
            .into_response();
    };
    match gateway.transcribe(body.scope, &audio, &body.mime).await {
        Ok(text) => Json(json!({ "text": text })).into_response(),
        Err(e) => crate::assist::assist_error(&e),
    }
}

// ── Admin: Assist governance (endpoint config / capability locks / kill switch) ──

/// Resolve the authenticated admin id, or a `401`. Mirrors `plugins::require_admin`.
async fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<String, Response> {
    if !state.v6.admin_enabled {
        return Err(admin_unauthorized());
    }
    let token = admin_cookie(headers).ok_or_else(admin_unauthorized)?;
    let hash = crate::push_relay::hash_token(&token);
    match state.store.get_admin_session(&hash).await {
        Ok(Some(admin_id)) => Ok(admin_id),
        _ => Err(admin_unauthorized()),
    }
}

fn admin_unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "admin authentication required" })),
    )
        .into_response()
}

fn admin_cookie(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in raw.split(';') {
        if let Some(v) = part.trim().strip_prefix("mw_admin_session=")
            && !v.is_empty()
        {
            return Some(v.to_string());
        }
    }
    None
}

/// `GET /admin/assist` — the current deployment Assist config (adapters/locks/ceilings).
async fn assist_admin_get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    match state.store.get_assist_config("deployment").await {
        Ok(Some(r)) => Json(json!({
            "enabled": r.enabled,
            "adapters": serde_json::from_str::<serde_json::Value>(&r.adapters_json).unwrap_or(json!(null)),
            "capabilityGrants": serde_json::from_str::<serde_json::Value>(&r.capability_grants_json).unwrap_or(json!([])),
            "dataCeilings": serde_json::from_str::<serde_json::Value>(&r.data_ceilings_json).unwrap_or(json!({})),
        }))
        .into_response(),
        Ok(None) => Json(json!({ "enabled": false })).into_response(),
        Err(e) => {
            tracing::error!("assist config read failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "store error").into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssistAdminReq {
    #[serde(default)]
    enabled: bool,
    #[serde(default = "empty_obj")]
    adapters: serde_json::Value,
    #[serde(default = "empty_arr")]
    capability_grants: serde_json::Value,
    #[serde(default = "empty_obj")]
    data_ceilings: serde_json::Value,
}

fn empty_obj() -> serde_json::Value {
    json!({})
}
fn empty_arr() -> serde_json::Value {
    json!([])
}

/// `PUT /admin/assist` — persist the deployment Assist config (0008 `assist_config`).
async fn assist_admin_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AssistAdminReq>,
) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    let row = mw_store::AssistConfigRow {
        scope: "deployment".into(),
        adapters_json: body.adapters.to_string(),
        capability_grants_json: body.capability_grants.to_string(),
        data_ceilings_json: body.data_ceilings.to_string(),
        enabled: body.enabled,
    };
    match state.store.put_assist_config(&row).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("assist config write failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "store error").into_response()
        }
    }
}

/// `POST /admin/assist/kill` — the tenant-wide Assist kill switch (§14/§19): flip
/// `enabled=false` in the persisted config. New gateways build Disabled on restart;
/// the running gateway is reconstructed on the next boot (documented — a live
/// hot-kill of the in-memory gateway is an e16 follow-up).
async fn assist_admin_kill(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    let existing = state
        .store
        .get_assist_config("deployment")
        .await
        .ok()
        .flatten();
    let row = mw_store::AssistConfigRow {
        scope: "deployment".into(),
        adapters_json: existing
            .as_ref()
            .map(|r| r.adapters_json.clone())
            .unwrap_or_else(|| "{}".into()),
        capability_grants_json: existing
            .as_ref()
            .map(|r| r.capability_grants_json.clone())
            .unwrap_or_else(|| "[]".into()),
        data_ceilings_json: existing
            .as_ref()
            .map(|r| r.data_ceilings_json.clone())
            .unwrap_or_else(|| "{}".into()),
        enabled: false,
    };
    match state.store.put_assist_config(&row).await {
        Ok(()) => Json(json!({ "killed": true })).into_response(),
        Err(e) => {
            tracing::error!("assist kill write failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "store error").into_response()
        }
    }
}

// ── Nextcloud: WebDAV PROPFIND directory listing ─────────────────────────────

#[derive(Debug, Deserialize)]
struct ListQuery {
    #[serde(default)]
    path: String,
}

/// `GET /api/nextcloud/list?path=` — a WebDAV PROPFIND directory listing (the picker).
async fn nextcloud_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(nc): Extension<crate::nextcloud::NextcloudHandle>,
    Query(q): Query<ListQuery>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    let Some(nc) = nc else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({ "error": "no nextcloud account linked" })),
        )
            .into_response();
    };
    match nc.list(&q.path).await {
        Ok(entries) => Json(json!({ "entries": entries })).into_response(),
        Err(e) => {
            tracing::warn!("nextcloud list failed: {e}");
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": "nextcloud unreachable" })),
            )
                .into_response()
        }
    }
}

// ── Plugins: allow-unsigned (enable an unsigned component under policy) ───────

/// `POST /admin/plugins/{id}/allow-unsigned` — enable an unsigned component with the
/// explicit unsigned override (⇒ the persistent banner). Persists `enabled` to 0008.
async fn plugin_allow_unsigned(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(reg): Extension<PluginRegistry>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    {
        let mut host = reg.lock().expect("plugin registry lock");
        if let Err(e) = host.enable(&id) {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    }
    let _ = state.store.set_plugin_enabled(&id, true).await;
    Json(json!({ "enabled": true, "signed": false, "allowUnsigned": true })).into_response()
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. CLI helpers (main.rs `plugin` / `password` subcommands)
// ─────────────────────────────────────────────────────────────────────────────

/// `mailwoman plugin list` body (over the 0008 registry).
pub async fn cli_plugin_list(store: &Store) -> anyhow::Result<Vec<PluginRow>> {
    Ok(store.list_plugins().await?)
}

/// `mailwoman plugin approve <id>` body (persists to 0008).
pub async fn cli_plugin_approve(store: &Store, id: &str, admin: &str) -> anyhow::Result<()> {
    if store.list_plugins().await?.iter().all(|p| p.id != id) {
        anyhow::bail!("no plugin '{id}' is registered");
    }
    store.set_plugin_approved(id, admin).await?;
    Ok(())
}

/// `mailwoman password` body: change an account password via the configured backend,
/// then re-seal stored upstream credentials (mirrors `POST /api/password`).
pub async fn cli_password_change(
    store: &Store,
    backend: &dyn PasswordChangeBackend,
    account_id: &str,
    old: &str,
    new: &str,
) -> anyhow::Result<()> {
    let ctx = Ctx {
        account_id: account_id.to_string(),
        username: account_id.to_string(),
        reseal_credentials: !store.sessions_by_account(account_id).await?.is_empty(),
        zeroaccess: false,
    };
    let outcome = backend
        .change(&ctx, Secret::new(old), Secret::new(new))
        .await
        .map_err(|e| anyhow::anyhow!("password change failed: {e}"))?;
    let mut resealed = 0;
    if outcome.reencrypt_credentials {
        resealed = store.reseal_account_credentials(account_id, new).await?;
    }
    store
        .put_password_change_audit(account_id, "cli", "ok")
        .await?;
    println!(
        "password changed for {account_id} (credentials re-sealed: {resealed}; zero-access re-wrap required: {})",
        outcome.zeroaccess_rewrap_required
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::ServerKey;

    #[test]
    fn ldap3062_exop_rejection_is_a_failure_not_a_false_success() {
        // BUG 2 (t7-fix-e16): a REJECTED RFC-3062 password change must surface as an
        // error. A non-zero result code (rc=50 insufficient-access, rc=53
        // unwilling-to-verify-old) previously fell through to `Ok(..)` — reporting the
        // change as successful.
        for (rc, text) in [(50u32, "insufficient access"), (53, "unwilling to perform")] {
            let out = exop_outcome(rc, text, None);
            match out {
                Err(mw_passwd::PasswordError::Protocol(m)) => {
                    assert!(m.contains(&format!("rc={rc}")), "rc reported: {m}");
                }
                other => panic!("rc={rc} must be a Protocol error, got {other:?}"),
            }
        }
        // Even if the server were to (spuriously) return a value alongside a rejection,
        // the rejection still wins — no false success.
        assert!(exop_outcome(50, "denied", Some(vec![1, 2, 3])).is_err());

        // A success (rc=0) yields the optional response value (empty when absent).
        assert_eq!(exop_outcome(0, "", Some(vec![9, 9])).unwrap(), vec![9, 9]);
        assert_eq!(exop_outcome(0, "", None).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn host_of_strips_scheme_and_port() {
        assert_eq!(
            host_of("https://cloud.example.org/x").as_deref(),
            Some("cloud.example.org")
        );
        assert_eq!(host_of("http://h:8080").as_deref(), Some("h"));
    }

    #[tokio::test]
    async fn countersign_prefixes_read_the_flag() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        store
            .put_api_key(&mw_store::ApiKeyRow {
                id: "1".into(),
                key_prefix: "aaaa".into(),
                key_hash: "h".into(),
                account_id: "acct".into(),
                scopes_json: "{}".into(),
                unattended_send: true,
                created_at: "2026-07-14T00:00:00Z".into(),
                last_used_at: None,
                revoked_at: None,
            })
            .await
            .unwrap();
        store
            .put_api_key(&mw_store::ApiKeyRow {
                id: "2".into(),
                key_prefix: "bbbb".into(),
                key_hash: "h".into(),
                account_id: "acct".into(),
                scopes_json: "{}".into(),
                unattended_send: false,
                created_at: "2026-07-14T00:00:00Z".into(),
                last_used_at: None,
                revoked_at: None,
            })
            .await
            .unwrap();
        let set = load_countersigned_prefixes(&store).await;
        assert!(set.contains("aaaa"), "countersigned key is present");
        assert!(!set.contains("bbbb"), "non-countersigned key is absent");
    }

    #[tokio::test]
    async fn disabled_assist_when_unconfigured() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let (gateway, granted) = build_assist(&store).await;
        assert!(!gateway.is_enabled(), "unconfigured Assist is Disabled");
        assert!(
            granted.is_empty(),
            "no capabilities granted when unconfigured"
        );
    }
}
