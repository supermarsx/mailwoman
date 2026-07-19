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
use std::path::{Path, PathBuf};
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
use sha2::{Digest, Sha256};

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
    BasicCredentialProvider, BasicCredentials, Clock, Grant, HostServices, HttpFetcher, HttpReq,
    HttpResp, KvStore, OAuthTokenProvider, PluginHandle, PluginHost, PluginLimits, PluginManifest,
    Rng,
};
use mw_store::{PluginKvLimits, PluginRow, Store};

use crate::AppState;
use crate::assist::AssistHandle;
use crate::nextcloud::{NextcloudGateway, OcsNextcloud};
use crate::plugins::PluginRegistry;

// The third-party allowlist admin API (approve/revoke/list-pending/uninstall). Declared
// as a CHILD module of `v7_mount` (via `#[path]`) rather than a top-level `mod` in
// `lib.rs`: `lib.rs` is owned by another executor this wave and must not be
// concurrently edited, and `extra_v7_router()` (below, owned here) already merges into
// the mounted router — so these routes reach the app without any `lib.rs` change. The
// file lives at `crates/mw-server/src/admin_plugins.rs`.
#[path = "admin_plugins.rs"]
mod admin_plugins;

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

/// The host `basic-credentials` provider for password-based bridges (on-prem EWS
/// Basic/NTLMv2, t12 §2). OAuth bearers cannot serve NTLMv2 (which derives NTOWFv2
/// from the cleartext password), so the host holds the secret SEALED at rest in the
/// 0011 `ews_account_cred` rows and unseals it only to answer one gated
/// `basic-credentials` import for the bound account. The guest never persists it.
///
/// Fail-closed: an account with no stored row, or a disabled row, returns an auth
/// error (that account simply fails to authenticate) — never a panic.
pub(crate) struct StoreEwsCredProvider {
    store: Store,
}

#[async_trait]
impl BasicCredentialProvider for StoreEwsCredProvider {
    async fn credentials(&self, account: &str) -> std::result::Result<BasicCredentials, String> {
        match self.store.get_ews_account_cred(account).await {
            Ok(Some(c)) if c.enabled => Ok(BasicCredentials {
                user: c.user,
                domain: c.domain,
                password: c.password,
                workstation: c.workstation,
                endpoint: c.endpoint,
            }),
            // Absent or disabled row ⇒ no usable credential ⇒ auth fails cleanly.
            Ok(_) => Err(format!(
                "no enabled EWS credentials stored for account '{account}'"
            )),
            Err(e) => Err(format!("EWS credential store error: {e}")),
        }
    }
}

/// The persistent, sealed, quota-bounded plugin KV backing `store:kv-scoped` (plan
/// §e5 / PQ1–PQ6). Replaces the former non-persistent `HostKv` stub (get→None,
/// put→no-op) with the store-backed 0013 `plugin_kv` methods.
///
/// The `(plugin_id, account_id)` namespace is derived HOST-side by `mw-plugin` from
/// the bound plugin instance and passed in here — never from a guest argument — so a
/// guest can only reach its own namespace. Values are sealed at rest by the store, and
/// per-namespace quotas are enforced at put; an over-quota put returns `Err`, which
/// `mw-plugin` surfaces to the guest as a visible (trapping) failure.
struct StorePluginKv {
    store: Store,
    limits: PluginKvLimits,
}

#[async_trait]
impl KvStore for StorePluginKv {
    async fn get(&self, plugin_id: &str, account_id: &str, key: &str) -> Option<Vec<u8>> {
        // A read error (corrupt/unopenable seal) is treated as absent rather than
        // surfaced — the guest sees `None`, never host internals.
        self.store
            .plugin_kv_get(plugin_id, account_id, key)
            .await
            .ok()
            .flatten()
    }

    async fn put(
        &self,
        plugin_id: &str,
        account_id: &str,
        key: &str,
        value: Vec<u8>,
    ) -> std::result::Result<(), String> {
        self.store
            .plugin_kv_set(plugin_id, account_id, key, &value, &self.limits)
            .await
            .map_err(|e| e.to_string())
    }

    async fn delete(&self, plugin_id: &str, account_id: &str, key: &str) {
        let _ = self
            .store
            .plugin_kv_delete(plugin_id, account_id, key)
            .await;
    }

    async fn list(&self, plugin_id: &str, account_id: &str) -> Vec<String> {
        self.store
            .plugin_kv_list(plugin_id, account_id)
            .await
            .unwrap_or_default()
    }
}

/// Build the plugin-KV quota ceilings, deployment-configurable via env with the
/// advertised defaults (256 B key, 64 KiB value, 5 MiB total, 1000 keys per namespace).
fn plugin_kv_limits() -> PluginKvLimits {
    let mut l = PluginKvLimits::default();
    if let Some(v) = env("MW_PLUGIN_KV_MAX_KEY_BYTES").and_then(|s| s.parse().ok()) {
        l.max_key_bytes = v;
    }
    if let Some(v) = env("MW_PLUGIN_KV_MAX_VALUE_BYTES").and_then(|s| s.parse().ok()) {
        l.max_value_bytes = v;
    }
    if let Some(v) = env("MW_PLUGIN_KV_MAX_TOTAL_BYTES").and_then(|s| s.parse().ok()) {
        l.max_total_bytes = v;
    }
    if let Some(v) = env("MW_PLUGIN_KV_MAX_KEYS").and_then(|s| s.parse().ok()) {
        l.max_keys = v;
    }
    l
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
/// the allowlist boundary in `mw-plugin`), a deny-by-default OAuth provider, the
/// store-backed EWS per-account basic-credential provider, and scoped KV/clock/rng.
pub(crate) fn host_services(store: &Store) -> HostServices {
    HostServices {
        http: Arc::new(ReqwestFetcher::from_env(reqwest::Client::new())),
        oauth: Arc::new(DeniedOAuthProvider),
        basic_creds: Arc::new(StoreEwsCredProvider {
            store: store.clone(),
        }),
        kv: Arc::new(StorePluginKv {
            store: store.clone(),
            limits: plugin_kv_limits(),
        }),
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
    host.set_services(host_services(store));
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

/// The compiled-in SHA-256 digest pin for each FIRST-PARTY component, keyed by the
/// 0008 `plugins.id` (§7.2, D5). The `.wasm` components no longer ship *inside* the
/// server binary (`include_bytes!`); they ship as external data files loaded at boot
/// (see [`resolve_component`]). This table is the deny-by-default integrity anchor:
/// only bytes that hash to the pinned digest ever enter the wasmtime jail, so a
/// tampered or swapped on-disk component fails closed (logged, never silently
/// loaded). An id absent from this table is not a first-party component.
///
/// GENERATED by `plugins/gen-digests.sh` over `plugins/dist/<id>.wasm`.
/// Rebuild a component ⇒ refresh `plugins/dist/<id>.wasm` ⇒ regenerate this table
/// (the script header documents the full workflow).
const FIRST_PARTY_DIGESTS: &[(&str, [u8; 32])] = &[
    (
        "bridge-graph",
        [
            0xe9, 0x58, 0xaf, 0x2b, 0x11, 0x2b, 0x56, 0x95, 0xf7, 0xe7, 0x05, 0x09, 0xcd, 0x9a,
            0xbb, 0x68, 0xc8, 0xd9, 0xa4, 0xab, 0xfd, 0x9d, 0xe2, 0xc2, 0x82, 0xef, 0x66, 0xe3,
            0x53, 0x65, 0x3c, 0xf9,
        ],
    ),
    (
        "bridge-ews",
        [
            0x54, 0x8b, 0xcd, 0xea, 0x1a, 0x97, 0xd6, 0xda, 0x6a, 0x9c, 0xd1, 0x38, 0xab, 0xd4,
            0x9a, 0x4d, 0x89, 0xcf, 0x20, 0x30, 0x27, 0x84, 0x5b, 0x6d, 0xda, 0x6e, 0x43, 0x84,
            0x86, 0xe9, 0x0c, 0xbf,
        ],
    ),
    (
        "bridge-gmail",
        [
            0x6c, 0xaa, 0x80, 0x85, 0x6f, 0x04, 0x74, 0x2d, 0xa6, 0xb2, 0xd4, 0xde, 0xd3, 0x6a,
            0x4e, 0xc5, 0xa7, 0xf4, 0xa4, 0x86, 0x77, 0xd7, 0xcb, 0x68, 0x1b, 0xbe, 0x0b, 0x83,
            0x86, 0x69, 0xf0, 0xc5,
        ],
    ),
    (
        "languagetool",
        [
            0xaf, 0x64, 0x01, 0x38, 0xa3, 0x6a, 0x24, 0x58, 0x44, 0xe8, 0xb8, 0x79, 0x5e, 0x75,
            0x6b, 0xd0, 0x26, 0x34, 0x2d, 0x0b, 0x62, 0x6f, 0xe1, 0xed, 0x7e, 0x1b, 0x44, 0x78,
            0x6d, 0x16, 0xfb, 0xc1,
        ],
    ),
    (
        "nextcloud",
        [
            0xfb, 0xe4, 0xc8, 0x1e, 0xbe, 0x1d, 0xb0, 0x7e, 0x93, 0xa5, 0xb1, 0xb8, 0x6d, 0xa7,
            0xb8, 0xac, 0x97, 0x59, 0x91, 0xba, 0xcc, 0x40, 0xe2, 0x21, 0x85, 0x95, 0x2e, 0xb6,
            0xcf, 0x97, 0x9f, 0x24,
        ],
    ),
    (
        "spam-rspamd",
        [
            0x23, 0xb4, 0x2c, 0x10, 0xc4, 0xed, 0x67, 0x67, 0xb9, 0x69, 0xf3, 0x2f, 0xb1, 0x7e,
            0x8a, 0xb4, 0xbd, 0xc6, 0x0e, 0xd4, 0xf7, 0x57, 0x67, 0xd3, 0xb1, 0x60, 0x61, 0xc3,
            0x14, 0x3b, 0x17, 0x4a,
        ],
    ),
    (
        "spam-spamassassin",
        [
            0x3c, 0x6d, 0xab, 0xac, 0x3b, 0xc4, 0x47, 0xd8, 0x29, 0xeb, 0xe7, 0x44, 0x70, 0x29,
            0xd4, 0x92, 0x15, 0x4c, 0x9f, 0xb9, 0x49, 0x9b, 0x23, 0xc1, 0xc1, 0xfe, 0x72, 0x7b,
            0xe2, 0xa9, 0x91, 0x3a,
        ],
    ),
];

/// The expected SHA-256 for a first-party component id. `nextcloud-plugin` is an
/// alias for the `nextcloud` component (both 0008 ids map to the same bytes).
fn first_party_digest(plugin_id: &str) -> Option<(&'static str, [u8; 32])> {
    let key = if plugin_id == "nextcloud-plugin" {
        "nextcloud"
    } else {
        plugin_id
    };
    FIRST_PARTY_DIGESTS
        .iter()
        .find(|(id, _)| *id == key)
        .map(|(id, d)| (*id, *d))
}

/// The external plugins directories the first-party `.wasm` components ship in, in
/// resolution order (§7.2, D5). The first candidate that yields a **digest-verified**
/// `<id>.wasm` wins:
///   1. `$MW_PLUGIN_DIR` — the authoritative deployment/Docker/Tauri override.
///   2. `<exe-dir>/plugins` — next to the running binary (Tauri self-contained + any
///      relocatable install that lays the components beside `mailwoman`).
///   3. `/usr/lib/mailwoman/plugins` — the Linux distro/deb/rpm/Flatpak data dir.
///   4. (debug builds only) the in-repo canonical layout `plugins/dist`, so
///      `cargo run`/`cargo test` work with no env set. Compiled OUT of release
///      builds so no build-host path is embedded in the shipped binary.
fn plugin_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(d) = env("MW_PLUGIN_DIR") {
        dirs.push(PathBuf::from(d));
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        dirs.push(parent.join("plugins"));
    }
    dirs.push(PathBuf::from("/usr/lib/mailwoman/plugins"));
    #[cfg(debug_assertions)]
    dirs.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../plugins/dist"));
    dirs
}

fn hex32(b: &[u8; 32]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(64);
    for byte in b {
        let _ = write!(s, "{byte:02x}");
    }
    s
}

/// Resolve a FIRST-PARTY bridge/plugin component from the external plugins dir and
/// VERIFY it against its compiled-in SHA-256 before handing back the bytes (§7.2, D5).
/// The components ship as external data files (stripping their bytes from the server
/// binary); integrity is preserved by the digest pin.
///
/// This is the FROZEN, authoritative path and it is TERMINAL for a first-party id:
/// [`resolve_component`] NEVER consults the third-party allowlist for an id this
/// recognises, even on a miss/tamper here (see the ordering contract on the gate).
///
/// Deny-by-default / fail-closed, and it NEVER panics:
///   - an id with no pinned digest is not first-party ⇒ `None` (the gate then tries the
///     third-party allowlist path);
///   - a missing / unreadable file ⇒ try the next dir, else `None`;
///   - a present-but-tampered file (digest mismatch) ⇒ logged + skipped (only a
///     byte-exact match to the pin ever loads).
///
/// Every skip is `tracing::warn`-logged (never silent).
fn first_party_component(plugin_id: &str) -> Option<Vec<u8>> {
    let (id, expected) = first_party_digest(plugin_id)?;
    let file = format!("{id}.wasm");
    let mut tried = Vec::new();
    for dir in plugin_dirs() {
        let path = dir.join(&file);
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let actual: [u8; 32] = Sha256::digest(&bytes).into();
        if actual == expected {
            tracing::info!(
                "loaded first-party component '{plugin_id}' from {} (digest-verified)",
                path.display()
            );
            return Some(bytes);
        }
        tracing::warn!(
            "component at {} FAILED the digest pin for first-party plugin '{plugin_id}' \
             (expected {}, got {}); skipping (integrity, fail-closed)",
            path.display(),
            hex32(&expected),
            hex32(&actual)
        );
        tried.push(path.display().to_string());
    }
    tracing::warn!(
        "no digest-verified component for first-party plugin '{plugin_id}' (checked: {}); \
         not loaded — set MW_PLUGIN_DIR or install to /usr/lib/mailwoman/plugins",
        if tried.is_empty() {
            "none present".to_string()
        } else {
            tried.join(", ")
        }
    );
    None
}

/// The SEPARATE directory third-party (non-first-party) components load from — NEVER the
/// first-party `plugin_dirs()`, so a third-party file can never shadow a first-party
/// filename or vice versa (TQ2). Unset ⇒ third-party loading is off entirely.
fn thirdparty_plugin_dir() -> Option<PathBuf> {
    env("MW_THIRDPARTY_PLUGIN_DIR").map(PathBuf::from)
}

/// The authoritative first-party id list (including the `nextcloud-plugin` alias). The
/// allowlist approve path uses it to reject a spoofing id (TQ2 anti-spoof); `mw-store`
/// cannot know it on its own because the compiled-in first-party table lives here.
pub(crate) fn first_party_ids() -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = FIRST_PARTY_DIGESTS.iter().map(|(id, _)| *id).collect();
    ids.push("nextcloud-plugin");
    ids
}

/// THE code-load gate (§7.2, D5 + TQ1–TQ5). Ordering here is SECURITY-CRITICAL:
///   1. `first_party_digest(plugin_id)` is consulted FIRST. If it is `Some`, run the
///      frozen first-party verify ([`first_party_component`]) and RETURN its result —
///      the third-party allowlist is NEVER consulted for a first-party id, even on a
///      first-party miss/tamper (`None`, no fall-through). This makes a third-party
///      allowlist row whose id collides with a first-party id UNREACHABLE, so it can
///      never override, shadow, or spoof a first-party identity (TQ2).
///   2. ONLY for a non-first-party id, consult the 0014 admin-pinned-digest allowlist via
///      [`resolve_third_party_component`], which admits bytes ONLY on a byte-exact SHA-256
///      match to a non-revoked admin-approved pin and audits every refusal.
///
/// The digest pin alone is sufficient to admit bytes here (⇒ an `UnsignedAllowed`
/// indication + audit at load). The EXISTING `PluginHost::load` signature path
/// (`signature::decide`) still runs on top: if a `TrustRoot` + manifest signature are
/// configured it ALSO verifies them — a defense-in-depth layer that reuses the vendored
/// `ed25519-dalek`, adding no dependency and never weakening the digest gate.
async fn resolve_component(plugin_id: &str, store: &Store) -> Option<Vec<u8>> {
    if first_party_digest(plugin_id).is_some() {
        // First-party: frozen, authoritative, TERMINAL — no fall-through to the allowlist.
        return first_party_component(plugin_id);
    }
    resolve_third_party_component(plugin_id, store).await
}

/// Resolve a NON-first-party component: admit its bytes IFF their exact SHA-256 is an
/// active (non-revoked) admin pin in the 0014 allowlist for this exact id (TQ1/TQ2/TQ4).
/// Fail-closed + audited on every refuse; NEVER panics. Reads the dir from
/// `MW_THIRDPARTY_PLUGIN_DIR`; the core is [`resolve_third_party_in_dir`].
async fn resolve_third_party_component(plugin_id: &str, store: &Store) -> Option<Vec<u8>> {
    // Never reachable for a first-party id (the gate checks first-party FIRST); assert the
    // invariant defensively so a future refactor can't turn this into a spoof vector.
    debug_assert!(first_party_digest(plugin_id).is_none());
    let Some(dir) = thirdparty_plugin_dir() else {
        tracing::warn!(
            "plugin '{plugin_id}' is not first-party and MW_THIRDPARTY_PLUGIN_DIR is unset; \
             third-party loading is off (deny-by-default)"
        );
        return None;
    };
    resolve_third_party_in_dir(plugin_id, store, &dir).await
}

/// The TOCTOU-safe third-party verify core: read the candidate bytes ONCE into memory,
/// hash THOSE bytes, and return the SAME buffer the caller then loads — there is no second
/// filesystem read between the hash and the load, so a file swapped after the check can
/// never be loaded. Split out so tests can point it at a temp dir without touching the
/// process-global env var.
async fn resolve_third_party_in_dir(plugin_id: &str, store: &Store, dir: &Path) -> Option<Vec<u8>> {
    // The id maps to <dir>/<id>.wasm. ids come from the 0008 registry, but fail closed on
    // any id that could escape the dir (empty / separators / traversal) regardless.
    if plugin_id.is_empty()
        || plugin_id.contains('/')
        || plugin_id.contains('\\')
        || plugin_id.contains("..")
    {
        tracing::warn!("refusing unsafe third-party plugin id '{plugin_id}'");
        audit_plugin_event(
            store,
            mw_admin::AuditKind::PluginLoadRefused,
            plugin_id,
            json!({ "reason": "unsafe-id" }),
        )
        .await;
        return None;
    }
    let path = dir.join(format!("{plugin_id}.wasm"));
    let Ok(bytes) = std::fs::read(&path) else {
        tracing::warn!(
            "no third-party component file for plugin '{plugin_id}' at {}; not loaded",
            path.display()
        );
        audit_plugin_event(
            store,
            mw_admin::AuditKind::PluginLoadRefused,
            plugin_id,
            json!({ "reason": "absent-file" }),
        )
        .await;
        return None;
    };
    // Hash the EXACT in-memory bytes we will return (single read, no re-open).
    let actual: [u8; 32] = Sha256::digest(&bytes).into();
    let actual_hex = hex32(&actual);
    match store
        .is_third_party_digest_approved(plugin_id, &actual_hex)
        .await
    {
        Ok(true) => {
            tracing::info!(
                "loaded third-party component '{plugin_id}' from {} (admin-pinned digest {})",
                path.display(),
                actual_hex
            );
            audit_plugin_event(
                store,
                mw_admin::AuditKind::PluginLoadAdmitted,
                plugin_id,
                json!({ "digest": actual_hex, "signature": "unsigned-allowed" }),
            )
            .await;
            Some(bytes)
        }
        Ok(false) => {
            tracing::warn!(
                "third-party component '{plugin_id}' at {} has digest {} which is NOT an active \
                 admin-approved pin; REFUSED (fail-closed)",
                path.display(),
                actual_hex
            );
            audit_plugin_event(
                store,
                mw_admin::AuditKind::PluginLoadRefused,
                plugin_id,
                json!({ "reason": "digest-not-approved", "digest": actual_hex }),
            )
            .await;
            None
        }
        Err(e) => {
            tracing::error!(
                "allowlist lookup failed for third-party plugin '{plugin_id}': {e}; REFUSED"
            );
            audit_plugin_event(
                store,
                mw_admin::AuditKind::PluginLoadRefused,
                plugin_id,
                json!({ "reason": "allowlist-error" }),
            )
            .await;
            None
        }
    }
}

// ── HIGH_POWER capability provenance gate (TQ4 sub-Q — the user's 26.15 decision) ──────

/// The maintained HIGH_POWER capability set: the account-backend / send-as-user class.
/// Per the user's explicit 26.15 decision these are FIRST-PARTY ONLY. A non-first-party
/// (third-party) plugin can never be granted one, even by admin action —
/// [`provenance_filtered_grant`] strips them at Grant construction, the point a capability
/// becomes runtime-effective, so the refusal cannot be overridden by a persisted
/// `plugin_grants` row. `AccountBackend` IS the "be the account / send as the user" seam
/// (the bridge role, §6.5); a third-party plugin must never hold it. First-party plugins
/// are unaffected. Extend this list if a future capability joins that class.
const HIGH_POWER_CAPABILITIES: &[mw_plugin::Capability] = &[mw_plugin::Capability::AccountBackend];

/// Whether `cap` is in the HIGH_POWER (first-party-only) class.
fn is_high_power(cap: mw_plugin::Capability) -> bool {
    HIGH_POWER_CAPABILITIES.contains(&cap)
}

/// Whether `plugin_id` is a pinned first-party component — the ONLY provenance permitted a
/// HIGH_POWER capability.
fn is_first_party_plugin(plugin_id: &str) -> bool {
    first_party_digest(plugin_id).is_some()
}

/// Filter a requested capability set by provenance. A first-party plugin keeps every
/// capability; a third-party plugin has EVERY HIGH_POWER capability stripped (returned in
/// `refused`). Returns `(kept, refused)`. This is the provenance gate; it runs where the
/// runtime [`Grant`] is built, so a third-party plugin never receives a HIGH_POWER
/// capability at runtime regardless of what an admin persisted.
fn provenance_filtered_grant(
    plugin_id: &str,
    requested: &[mw_plugin::Capability],
) -> (Vec<mw_plugin::Capability>, Vec<mw_plugin::Capability>) {
    if is_first_party_plugin(plugin_id) {
        return (requested.to_vec(), Vec::new());
    }
    let mut kept = Vec::new();
    let mut refused = Vec::new();
    for &c in requested {
        if is_high_power(c) {
            refused.push(c);
        } else {
            kept.push(c);
        }
    }
    (kept, refused)
}

// ── Content-free audit for plugin load / allowlist events ──────────────────────────────

/// Append a content-free audit row for a loader-side plugin event (admit/refuse). The
/// actor is the loader; `detail` MUST carry no mail content — only ids/digests/reasons.
async fn audit_plugin_event(
    store: &Store,
    kind: mw_admin::AuditKind,
    plugin_id: &str,
    detail: serde_json::Value,
) {
    append_plugin_audit(
        store,
        "plugin-loader",
        mw_admin::ActorKind::System,
        kind,
        plugin_id,
        detail,
    )
    .await;
}

/// The shared audit-append used by both the loader and the admin allowlist routes. Reuses
/// `mw-admin`'s [`mw_admin::AuditEvent`] (which mints the id + timestamp and REDACTS the
/// detail) then maps it into the 0007 `audit_log` row. Best-effort: an audit-store error
/// is logged, never propagated — a failed audit must not change a load/deny decision.
pub(crate) async fn append_plugin_audit(
    store: &Store,
    actor: &str,
    actor_kind: mw_admin::ActorKind,
    kind: mw_admin::AuditKind,
    plugin_id: &str,
    detail: serde_json::Value,
) {
    let entry = mw_admin::AuditEvent::new(actor, actor_kind, kind)
        .target(plugin_id)
        .detail(detail)
        .into_entry();
    let row = mw_store::AuditRow {
        id: entry.id,
        ts: entry.ts,
        actor: entry.actor,
        actor_kind: actor_kind_str(entry.actor_kind).to_string(),
        action: entry.action,
        target: entry.target,
        detail_json: entry.detail_json,
        ip: entry.ip,
    };
    if let Err(e) = store.append_audit(&row).await {
        tracing::warn!("plugin audit append failed ({}): {e}", row.action);
    }
}

/// Serialize an [`mw_admin::ActorKind`] to its stable kebab-case string for the audit row
/// (mirrors the mapping in `stores_v6`, kept local to avoid a cross-module private dep).
fn actor_kind_str(k: mw_admin::ActorKind) -> &'static str {
    match k {
        mw_admin::ActorKind::Admin => "admin",
        mw_admin::ActorKind::User => "user",
        mw_admin::ActorKind::ApiKey => "api-key",
        mw_admin::ActorKind::System => "system",
    }
}

/// The send seam for a bridge-backed account. A bridge's outbound mail flows through
/// its component's native API — the frozen `account-backend` `submit` export, which
/// each bridge maps to its provider send (Graph `sendMail`, Gmail `messages/send`, EWS
/// `CreateItem`+`SendItem`), NOT SMTP. `EmailSubmission/set` reaches this through the
/// engine's `MailSubmitter` seam: the engine composes the draft MIME and calls
/// [`MailSubmitter::submit`], which we route to the plugin backend's `submit` export
/// (the adapter maps `AccountBackend::append` → WIT `submit` → the guest → the
/// provider's send API, through the jail). A submit failure surfaces as an
/// `EngineError` (never a silent drop); the provider files the message into its own
/// Sent folder on send, so the engine skips the upstream Sent APPEND for plugin
/// accounts (see `submit_email`).
struct BridgeSubmitter {
    /// The same plugin/bridge account backend the engine syncs over; its `submit`
    /// (append) export is the provider send path.
    backend: Arc<dyn mw_engine::backend::AccountBackend>,
    bridge_id: String,
}

#[async_trait]
impl mw_engine::account::MailSubmitter for BridgeSubmitter {
    async fn submit(
        &self,
        msg: mw_smtp::Outgoing,
    ) -> mw_engine::backend::Result<mw_smtp::SubmissionResult> {
        // Route to the bridge's `submit` export via the frozen `AccountBackend::append`
        // seam (adapter → WIT `submit` → provider send). The mailbox ref is a neutral
        // placeholder — a bridge send ignores it beyond a synthetic return ref.
        let placeholder = mw_engine::backend::RawMailboxRef {
            name: "Sent".to_string(),
            uidvalidity: 0,
        };
        if let Err(e) = self.backend.append(&placeholder, &msg.raw, &[]).await {
            // Surface the failure (never silently drop). Content-free: bridge id + the
            // coarse backend error only.
            tracing::warn!("bridge '{}' send failed: {e}", self.bridge_id);
            return Err(e);
        }
        // The provider transmits to every recipient atomically and reports fatal
        // failures as the error above; report the envelope recipients accepted.
        Ok(mw_smtp::SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5b. Bridge PIM capability source (plan §2.5/§6.5, t10-e13) — gated on honest supports-*
// ─────────────────────────────────────────────────────────────────────────────

/// The bridge-native PIM/parity trait objects bound for ONE account, each present
/// only when the bridge's honest per-interface `supports-*()` is true (e1's rule:
/// bind a slot iff the accessor is `Some` AND the honest support == true — the coarse
/// legacy `account-backend capabilities()` is NOT consulted). Every unbound slot ⇒ the
/// engine's byte-unchanged standards fallback. Expected live shape: Graph = all six,
/// EWS = calendar+tasks, Gmail = none.
#[derive(Default, Clone)]
pub(crate) struct BridgePimSlots {
    caps: mw_engine::BridgeCaps,
    calendar: Option<Arc<dyn mw_engine::BridgeCalendar>>,
    tasks: Option<Arc<dyn mw_engine::BridgeTasks>>,
    reactions: Option<Arc<dyn mw_engine::BridgeReactions>>,
    voting: Option<Arc<dyn mw_engine::BridgeVoting>>,
    recall: Option<Arc<dyn mw_engine::BridgeRecall>>,
    focused: Option<Arc<dyn mw_engine::BridgeFocusedSync>>,
}

impl BridgePimSlots {
    /// Whether ANY PIM slot is bound (i.e. the account routes some PIM to the bridge).
    fn is_bound(&self) -> bool {
        self.calendar.is_some()
            || self.tasks.is_some()
            || self.reactions.is_some()
            || self.voting.is_some()
            || self.recall.is_some()
            || self.focused.is_some()
    }

    /// A content-free one-line summary of the bound interfaces (for the boot log).
    fn summary(&self) -> String {
        let mut on = Vec::new();
        if self.calendar.is_some() {
            on.push("calendar");
        }
        if self.tasks.is_some() {
            on.push("tasks");
        }
        if self.reactions.is_some() {
            on.push("reactions");
        }
        if self.voting.is_some() {
            on.push("voting");
        }
        if self.recall.is_some() {
            on.push("recall");
        }
        if self.focused.is_some() {
            on.push("focused-sync");
        }
        if on.is_empty() {
            "none (standards fallback)".to_string()
        } else {
            on.join("+")
        }
    }
}

/// Probe a loaded bridge handle's HONEST per-interface support (through the jail) and
/// bind each PIM slot only when both the accessor is present AND the matching
/// `supports-*()` is true. Fail-soft: a probe error ⇒ "no support" ⇒ the engine keeps
/// its standards fallback (never a hard failure at boot).
pub(crate) async fn probe_bridge_pim(handle: &PluginHandle) -> BridgePimSlots {
    // Honest per-interface support, crossing the jail once each.
    let parity = handle.bridge_parity_caps().await.unwrap_or_default();
    let cal_ok = handle.bridge_supports_calendar().await.unwrap_or(false);
    let tasks_ok = handle.bridge_supports_tasks().await.unwrap_or(false);

    // `as_bridge_*` returns Some only when the interface is present AND account-backend
    // is granted; combined with the honest support flag this binds a slot iff both hold.
    let calendar = if cal_ok {
        handle.as_bridge_calendar()
    } else {
        None
    };
    let tasks = if tasks_ok {
        handle.as_bridge_tasks()
    } else {
        None
    };
    let reactions = if parity.reactions {
        handle.as_bridge_reactions()
    } else {
        None
    };
    let voting = if parity.voting {
        handle.as_bridge_voting()
    } else {
        None
    };
    let recall = if parity.recall {
        handle.as_bridge_recall()
    } else {
        None
    };
    let focused = if parity.focused_sync {
        handle.as_bridge_focused_sync()
    } else {
        None
    };

    // Report caps that reflect what actually bound (never overclaim).
    let caps = mw_engine::BridgeCaps {
        reactions: reactions.is_some(),
        voting: voting.is_some(),
        recall: recall.is_some(),
        focused_sync: focused.is_some(),
    };
    BridgePimSlots {
        caps,
        calendar,
        tasks,
        reactions,
        voting,
        recall,
        focused,
    }
}

/// The `BridgeCapabilitySource` e13 attaches: a per-account map of the precomputed
/// (boot-probed) PIM slots. A non-bridge account (absent from the map) yields `None`
/// for every accessor ⇒ the engine's byte-unchanged standards fallback.
pub(crate) struct BridgePimSource {
    accounts: std::collections::HashMap<String, BridgePimSlots>,
}

impl mw_engine::BridgeCapabilitySource for BridgePimSource {
    fn caps(&self, account_id: &str) -> mw_engine::BridgeCaps {
        self.accounts
            .get(account_id)
            .map(|s| s.caps)
            .unwrap_or_default()
    }
    fn reactions(&self, account_id: &str) -> Option<Arc<dyn mw_engine::BridgeReactions>> {
        self.accounts
            .get(account_id)
            .and_then(|s| s.reactions.clone())
    }
    fn voting(&self, account_id: &str) -> Option<Arc<dyn mw_engine::BridgeVoting>> {
        self.accounts.get(account_id).and_then(|s| s.voting.clone())
    }
    fn recall(&self, account_id: &str) -> Option<Arc<dyn mw_engine::BridgeRecall>> {
        self.accounts.get(account_id).and_then(|s| s.recall.clone())
    }
    fn focused_sync(&self, account_id: &str) -> Option<Arc<dyn mw_engine::BridgeFocusedSync>> {
        self.accounts
            .get(account_id)
            .and_then(|s| s.focused.clone())
    }
    fn calendar(&self, account_id: &str) -> Option<Arc<dyn mw_engine::BridgeCalendar>> {
        self.accounts
            .get(account_id)
            .and_then(|s| s.calendar.clone())
    }
    fn tasks(&self, account_id: &str) -> Option<Arc<dyn mw_engine::BridgeTasks>> {
        self.accounts.get(account_id).and_then(|s| s.tasks.clone())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5c. Spam-classification hook (plan §10.8, t10-e13) — jailed spam-action plugin
// ─────────────────────────────────────────────────────────────────────────────

/// Wraps a loaded `spam-action` plugin handle as the engine's [`mw_engine::SpamHook`].
/// The verdict envelope (`{"verdict":"spam"|"ham"|"unknown",…}`) is parsed here; a
/// classify error / malformed body ⇒ `Unknown` (fail-soft, never a hard block).
struct SpamPluginHook {
    handle: PluginHandle,
    plugin_id: String,
}

#[async_trait]
impl mw_engine::SpamHook for SpamPluginHook {
    async fn classify(&self, raw: &[u8]) -> mw_engine::SpamVerdict {
        match self.handle.call_spam_action(raw.to_vec()).await {
            Ok(json) => parse_spam_verdict(&json),
            Err(e) => {
                tracing::debug!(
                    "spam plugin '{}' classify failed (fail-soft unknown): {e}",
                    self.plugin_id
                );
                mw_engine::SpamVerdict::Unknown
            }
        }
    }
}

/// Parse the plugin's verdict envelope to the engine verdict. Anything that is not an
/// explicit `"spam"`/`"ham"` (missing field, malformed JSON, `"unknown"`) ⇒ `Unknown`.
fn parse_spam_verdict(json: &str) -> mw_engine::SpamVerdict {
    let verdict = serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .and_then(|v| {
            v.get("verdict")
                .and_then(|x| x.as_str())
                .map(str::to_string)
        });
    match verdict.as_deref() {
        Some("spam") => mw_engine::SpamVerdict::Spam,
        Some("ham") => mw_engine::SpamVerdict::Ham,
        _ => mw_engine::SpamVerdict::Unknown,
    }
}

/// Build the spam-classification hook from the 0008 registry: the FIRST approved +
/// enabled plugin that declares the `spam-action` capability AND resolves to a
/// digest-verified first-party component. `None` ⇒ no classifier configured (ingest is
/// byte-unchanged). Deny-by-default: an unapproved/disabled/unpinned plugin loads nothing.
pub(crate) async fn build_spam_hook(
    host: &PluginRegistry,
    store: &Store,
) -> Option<Arc<dyn mw_engine::SpamHook>> {
    let plugins = store.list_plugins().await.unwrap_or_default();
    let row = plugins.iter().find(|p| {
        p.approved_by.is_some()
            && p.enabled
            && serde_json::from_str::<Vec<mw_plugin::Capability>>(&p.capabilities_json)
                .map(|caps| caps.contains(&mw_plugin::Capability::SpamAction))
                .unwrap_or(false)
    })?;
    let bytes = resolve_component(&row.id, store).await?;
    let manifest = manifest_of(row);
    // Provenance gate: a third-party spam plugin keeps its `spam-action` (and other
    // non-HIGH_POWER) caps, but any HIGH_POWER cap it declared is stripped here — never
    // granted to a non-first-party plugin (the user's 26.15 decision).
    let (granted_caps, refused_caps) = provenance_filtered_grant(&row.id, &manifest.capabilities);
    if !refused_caps.is_empty() {
        tracing::warn!(
            "third-party plugin '{}' refused HIGH_POWER capability(ies) {:?} (first-party only)",
            row.id,
            refused_caps
        );
        audit_plugin_event(
            store,
            mw_admin::AuditKind::PluginLoadRefused,
            &row.id,
            json!({ "reason": "high-power-cap-refused", "caps": format!("{refused_caps:?}") }),
        )
        .await;
    }
    let grant = Grant {
        plugin_id: row.id.clone(),
        capabilities: granted_caps,
        granted_by: row.approved_by.clone().unwrap_or_default(),
        allow_unsigned: true,
    };
    let handle = {
        let host = host.lock().expect("plugin registry lock");
        match host.load(&bytes, &manifest, &grant) {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("spam plugin '{}' load failed: {e}", row.id);
                return None;
            }
        }
    };
    tracing::info!(
        "spam classifier plugin '{}' loaded (§10.8 delivery-filter hook)",
        row.id
    );
    Some(Arc::new(SpamPluginHook {
        handle,
        plugin_id: row.id.clone(),
    }))
}

/// Boot-load the approved bridge/plugin account backends from the 0008 registry
/// (plan §6.5). For every `bridge_accounts` binding whose bound plugin is an
/// **approved + enabled** `plugins` row *and* resolves to a digest-verified
/// first-party component, this obtains the component bytes from the external plugins
/// dir ([`resolve_component`]), `PluginHost::load`s them under the plugin's manifest +
/// a boot grant (host services already injected via [`build_plugin_host`]), takes
/// `as_account_backend()`, and registers it on the engine via `register_plugin_backend`
/// — after which the account is served by the SAME sync/JMAP dispatch as an IMAP
/// account. Additionally probes each loaded bridge's HONEST per-interface PIM support
/// and, when advertised, binds its calendar/tasks/reactions/voting/recall/focused-sync
/// trait objects into a per-account [`BridgePimSource`] (returned for e13 to attach via
/// [`mw_engine::V7Hooks::with_bridge_caps`]). Returns `(loaded_count, bridge_pim_source)`
/// — the source is `None` when no account bound any PIM interface.
///
/// Deny-by-default: an unbound, unapproved, disabled, or third-party (unpinned) plugin
/// loads nothing, a component whose on-disk bytes fail the digest pin fails closed, and
/// an account with no binding is byte-unchanged from the non-plugin path. Every skip is
/// logged (never silent).
pub async fn load_plugin_backends(
    engine: &Arc<mw_engine::Engine>,
    host: &PluginRegistry,
    store: &Store,
) -> (usize, Option<Arc<dyn mw_engine::BridgeCapabilitySource>>) {
    let bindings = match store.list_bridge_accounts().await {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("bridge_accounts read failed: {e}");
            return (0, None);
        }
    };
    if bindings.is_empty() {
        return (0, None);
    }
    let plugins = store.list_plugins().await.unwrap_or_default();
    let mut loaded = 0usize;
    let mut pim_slots: std::collections::HashMap<String, BridgePimSlots> =
        std::collections::HashMap::new();

    for b in &bindings {
        // The bound plugin must be a known, approved, ENABLED registry row.
        let Some(row) = plugins.iter().find(|p| p.id == b.bridge_id) else {
            tracing::warn!(
                "bridge account {} bound to unknown plugin '{}'; not loaded",
                b.account_id,
                b.bridge_id
            );
            continue;
        };
        if row.approved_by.is_none() || !row.enabled {
            tracing::warn!(
                "bridge plugin '{}' is not approved+enabled; account {} not loaded",
                b.bridge_id,
                b.account_id
            );
            continue;
        }
        // Deny-by-default code load. `resolve_component` admits bytes ONLY if they are a
        // digest-verified FIRST-PARTY component (frozen compiled-in pin, checked first and
        // terminally) OR a non-first-party component whose exact SHA-256 is an active
        // admin-approved pin in the 0014 allowlist (TQ1/TQ2/TQ4). A missing/tampered/
        // unapproved/revoked component fails closed (and audits).
        let Some(bytes) = resolve_component(&b.bridge_id, store).await else {
            tracing::warn!(
                "no digest-verified or admin-pinned component for plugin '{}'; \
                 account {} not loaded",
                b.bridge_id,
                b.account_id
            );
            continue;
        };

        let mut manifest = manifest_of(row);
        // EWS (password-auth bridge): the account's real Exchange host is provisioned
        // per-account in the sealed 0011 `ews_account_cred` row, not in the committed
        // fixture `plugin.toml` allowlist. Mirror its `endpoint_host` into the manifest
        // `net_allowlist` at mount so the jailed guest's host-mediated `http-fetch` to
        // the account's endpoint is admitted through the gate (deny-by-default holds for
        // every other host). Absent/disabled row ⇒ no rewrite (the guest then has no
        // reachable endpoint and fails auth via the credential provider above).
        if let Ok(Some(cred)) = store.get_ews_account_cred(&b.account_id).await
            && !cred.endpoint_host.is_empty()
            && !manifest
                .net_allowlist
                .iter()
                .any(|h| h.eq_ignore_ascii_case(&cred.endpoint_host))
        {
            manifest.net_allowlist.push(cred.endpoint_host);
        }
        // Provenance gate (the user's 26.15 decision): a first-party plugin keeps every
        // declared capability; a THIRD-PARTY plugin has every HIGH_POWER
        // (account-backend / send-as-user class) capability stripped here — the point the
        // runtime grant is built — so it can never act as an account backend even if an
        // admin persisted such a grant. A third-party bridge thus stripped will fail
        // `as_account_backend()` below and not load as a backend (fail-closed, as intended).
        let (granted_caps, refused_caps) =
            provenance_filtered_grant(&row.id, &manifest.capabilities);
        if !refused_caps.is_empty() {
            tracing::warn!(
                "third-party plugin '{}' refused HIGH_POWER capability(ies) {:?} for account {} \
                 (first-party only)",
                b.bridge_id,
                refused_caps,
                b.account_id
            );
            audit_plugin_event(
                store,
                mw_admin::AuditKind::PluginLoadRefused,
                &row.id,
                json!({ "reason": "high-power-cap-refused", "caps": format!("{refused_caps:?}") }),
            )
            .await;
        }
        let grant = Grant {
            plugin_id: row.id.clone(),
            capabilities: granted_caps,
            granted_by: row.approved_by.clone().unwrap_or_default(),
            // A digest-verified first-party component is trusted by virtue of matching
            // the compiled-in SHA-256 pin; a third-party component is trusted by an
            // admin-approved allowlist pin. The boot host carries an empty trust root,
            // which can't verify a detached signature, so the boot grant allows unsigned
            // — surfacing the persistent unsigned banner + audit until a signing trust
            // root is configured. Deny-by-default still holds: it took an approved+enabled
            // row + a binding + a passing digest pin (first-party OR admin allowlist) to
            // reach here, and HIGH_POWER caps are already stripped for third-party above.
            allow_unsigned: true,
        };

        let handle = {
            let host = host.lock().expect("plugin registry lock");
            // Bind THIS account to the instance so the guest's per-account host imports
            // (`basic-credentials`/`oauth-token`) — which pass an EMPTY handle per the
            // "one instance backs one account" contract — resolve to this account's
            // sealed creds host-side (fixes the EWS empty-handle bug; latent OAuth too).
            match host.load_for_account(&bytes, &manifest, &grant, &b.account_id) {
                Ok(h) => h,
                Err(e) => {
                    tracing::error!("plugin '{}' load failed: {e}", b.bridge_id);
                    continue;
                }
            }
        };
        let Some(backend) = handle.as_account_backend() else {
            tracing::warn!(
                "plugin '{}' does not advertise the account-backend capability; \
                 account {} not loaded",
                b.bridge_id,
                b.account_id
            );
            continue;
        };

        // The account's own identity (From/MAIL FROM); fall back to the account id.
        let identity = store
            .get_account(&b.account_id)
            .await
            .map(|a| a.username)
            .unwrap_or_else(|_| b.account_id.clone());
        let runtime = mw_engine::account::AccountRuntime::new(
            backend.clone(),
            Arc::new(BridgeSubmitter {
                backend,
                bridge_id: b.bridge_id.clone(),
            }) as Arc<dyn mw_engine::account::MailSubmitter>,
            identity,
        );
        engine.register_plugin_backend(b.account_id.clone(), b.bridge_id.clone(), runtime);
        loaded += 1;
        tracing::info!(
            "boot-loaded bridge '{}' backing account {}",
            b.bridge_id,
            b.account_id
        );

        // Probe the bridge's honest PIM support and bind the advertised slots for this
        // account (calendar/tasks via `supports-*`; parity via `bridge_parity_caps`).
        // A bridge that supports none (e.g. Gmail) binds nothing → standards fallback.
        let slots = probe_bridge_pim(&handle).await;
        tracing::info!(
            "bridge '{}' PIM binding for account {}: {}",
            b.bridge_id,
            b.account_id,
            slots.summary()
        );
        if slots.is_bound() {
            pim_slots.insert(b.account_id.clone(), slots);
        }
    }
    let source: Option<Arc<dyn mw_engine::BridgeCapabilitySource>> = if pim_slots.is_empty() {
        None
    } else {
        Some(Arc::new(BridgePimSource {
            accounts: pim_slots,
        }))
    };
    (loaded, source)
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
        // The third-party allowlist admin API (approve/revoke/list-pending/uninstall),
        // admin-session-gated + audited. Registered on this already-mounted router so no
        // `lib.rs` mount edit is needed this wave.
        .merge(admin_plugins::allowlist_router())
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

    // ── Externalized first-party component resolver + digest pin (t9-e5, §7.2) ──

    /// The five 0008 first-party ids all have a pinned digest, the `nextcloud-plugin`
    /// alias maps to the `nextcloud` bytes, and an unknown id is not first-party.
    #[test]
    fn first_party_digest_table_maps_every_id_and_the_alias() {
        for id in [
            "bridge-graph",
            "bridge-ews",
            "bridge-gmail",
            "languagetool",
            "nextcloud",
            "spam-rspamd",
            "spam-spamassassin",
        ] {
            assert!(first_party_digest(id).is_some(), "'{id}' is pinned");
        }
        let (canon, _) = first_party_digest("nextcloud-plugin").expect("alias resolves");
        assert_eq!(canon, "nextcloud", "the alias maps to the nextcloud bytes");
        assert!(
            first_party_digest("totally-unknown").is_none(),
            "an unpinned id is not first-party (deny-by-default)"
        );
    }

    /// The shipped canonical layout `plugins/dist/<id>.wasm` byte-matches the pinned
    /// digests — resolved via the debug-build in-repo fallback with NO env set. This
    /// is the anti-drift guard: rebuilding a component without regenerating the digest
    /// table (or a stale table) trips here. An unknown id fails closed to `None`.
    #[test]
    fn resolve_component_verifies_the_shipped_layout() {
        for id in [
            "bridge-graph",
            "bridge-ews",
            "bridge-gmail",
            "languagetool",
            "nextcloud",
            "spam-rspamd",
            "spam-spamassassin",
        ] {
            let bytes =
                first_party_component(id).unwrap_or_else(|| panic!("'{id}' resolves + verifies"));
            assert!(!bytes.is_empty(), "'{id}' has bytes");
            let (_, expected) = first_party_digest(id).unwrap();
            let actual: [u8; 32] = Sha256::digest(&bytes).into();
            assert_eq!(actual, expected, "'{id}' bytes match its pinned digest");
        }
        assert!(
            first_party_component("totally-unknown").is_none(),
            "an unpinned id never loads via the first-party path"
        );
    }

    /// A present-but-tampered component fails the digest pin (fail-closed): the pure
    /// decision — only a byte-exact match to the pin counts as loadable.
    #[test]
    fn digest_pin_rejects_tampered_bytes() {
        let (_, expected) = first_party_digest("languagetool").unwrap();
        let mut good = first_party_component("languagetool").expect("shipped bytes");
        let good_digest: [u8; 32] = Sha256::digest(&good).into();
        assert_eq!(good_digest, expected);
        // Flip one byte ⇒ the digest no longer matches the pin ⇒ would be skipped.
        good[0] ^= 0xff;
        let tampered: [u8; 32] = Sha256::digest(&good).into();
        assert_ne!(tampered, expected, "a tampered component fails the pin");
    }

    // ── Spam verdict parsing (§10.8, fail-soft) ──────────────────────────────────

    #[test]
    fn spam_verdict_parses_envelope_and_fails_soft() {
        assert_eq!(
            parse_spam_verdict(r#"{"verdict":"spam","score":15.2,"source":"rspamd"}"#),
            mw_engine::SpamVerdict::Spam
        );
        assert_eq!(
            parse_spam_verdict(r#"{"verdict":"ham"}"#),
            mw_engine::SpamVerdict::Ham
        );
        assert_eq!(
            parse_spam_verdict(r#"{"verdict":"unknown","note":"unreachable"}"#),
            mw_engine::SpamVerdict::Unknown
        );
        // Malformed / missing field ⇒ fail-soft Unknown (NEVER Spam — no hard block).
        assert_eq!(
            parse_spam_verdict("not json"),
            mw_engine::SpamVerdict::Unknown
        );
        assert_eq!(parse_spam_verdict("{}"), mw_engine::SpamVerdict::Unknown);
    }

    // ── Gated bridge-PIM binding shape (plan §6.5, e13 acceptance) ───────────────

    /// Load a digest-verified first-party bridge component through the real jail and
    /// probe its honest PIM binding shape (account-backend granted so `as_bridge_*`
    /// can bind).
    async fn probe_bridge_shape(id: &str) -> BridgePimSlots {
        let bytes = first_party_component(id).unwrap_or_else(|| panic!("'{id}' resolves"));
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let mut host = PluginHost::new();
        host.set_services(host_services(&store));
        let manifest = PluginManifest {
            id: id.to_string(),
            name: id.to_string(),
            version: "0".into(),
            signature: None,
            capabilities: vec![mw_plugin::Capability::AccountBackend],
            net_allowlist: Vec::new(),
            limits: PluginLimits::default(),
        };
        let grant = Grant {
            plugin_id: id.to_string(),
            capabilities: vec![mw_plugin::Capability::AccountBackend],
            granted_by: "test".into(),
            allow_unsigned: true,
        };
        let handle = host.load(&bytes, &manifest, &grant).expect("load bridge");
        probe_bridge_pim(&handle).await
    }

    /// The gated PIM binding is driven by each bridge's HONEST `supports-*` (never the
    /// coarse legacy account-backend caps): Graph binds all six, EWS calendar+tasks
    /// only, Gmail none (pure standards fallback).
    #[tokio::test]
    async fn bridge_pim_binding_shape_respects_honest_supports() {
        let graph = probe_bridge_shape("bridge-graph").await;
        assert!(
            graph.calendar.is_some() && graph.tasks.is_some(),
            "graph binds calendar + tasks"
        );
        assert!(
            graph.reactions.is_some()
                && graph.voting.is_some()
                && graph.recall.is_some()
                && graph.focused.is_some(),
            "graph binds all four parity interfaces"
        );
        assert_eq!(
            graph.caps,
            mw_engine::BridgeCaps {
                reactions: true,
                voting: true,
                recall: true,
                focused_sync: true,
            },
            "graph advertises all parity caps"
        );

        let ews = probe_bridge_shape("bridge-ews").await;
        assert!(
            ews.calendar.is_some() && ews.tasks.is_some(),
            "ews binds calendar + tasks"
        );
        assert!(
            ews.reactions.is_none()
                && ews.voting.is_none()
                && ews.recall.is_none()
                && ews.focused.is_none(),
            "ews parity stays on the standards fallback (EWS's legacy caps overclaim; \
             the honest supports-* are false)"
        );

        let gmail = probe_bridge_shape("bridge-gmail").await;
        assert!(
            !gmail.is_bound(),
            "gmail binds NO PIM interface (pure standards fallback)"
        );
    }

    // ── Third-party allowlist load gate (TQ1–TQ5) + HIGH_POWER provenance (26.15) ────

    /// A throwaway temp dir for a fake third-party `.wasm` (the gate only hashes bytes;
    /// wasm validity is the loader's concern, not `resolve_component`'s).
    fn temp_dir() -> PathBuf {
        let d = std::env::temp_dir().join(format!("mw-t15-e6-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        let d: [u8; 32] = Sha256::digest(bytes).into();
        hex32(&d)
    }

    /// A non-approved third-party component is REFUSED; approving its EXACT digest admits
    /// exactly those bytes; a revoked pin refuses on the next load; a tampered byte
    /// (digest mismatch) is refused. Every refuse path writes an audit row.
    #[tokio::test]
    async fn third_party_load_gate_positive_and_negatives() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let dir = temp_dir();
        let id = "acme-thirdparty";
        let bytes = b"\x00asm-not-really-but-hashable".to_vec();
        std::fs::write(dir.join(format!("{id}.wasm")), &bytes).unwrap();
        let digest = sha256_hex(&bytes);

        // (a) No approval row ⇒ REFUSED.
        assert!(
            resolve_third_party_in_dir(id, &store, &dir).await.is_none(),
            "a non-approved third-party component must not load"
        );

        // Approve the EXACT digest ⇒ admits exactly those bytes.
        store
            .put_plugin_allowlist(
                &mw_store::new_allowlist_pin(id, &digest, "admin@x", None, None, None, None),
                &first_party_ids(),
            )
            .await
            .unwrap();
        assert_eq!(
            resolve_third_party_in_dir(id, &store, &dir)
                .await
                .as_deref(),
            Some(&bytes[..]),
            "an approved exact-digest component loads its exact bytes"
        );

        // (c) Tamper one byte on disk ⇒ digest mismatch ⇒ REFUSED (the approved pin is
        // for the ORIGINAL bytes).
        let mut tampered = bytes.clone();
        tampered[0] ^= 0xff;
        std::fs::write(dir.join(format!("{id}.wasm")), &tampered).unwrap();
        assert!(
            resolve_third_party_in_dir(id, &store, &dir).await.is_none(),
            "a tampered third-party component (digest mismatch) must not load"
        );
        // Restore the good bytes, then (b) REVOKE ⇒ refused on the next load.
        std::fs::write(dir.join(format!("{id}.wasm")), &bytes).unwrap();
        assert!(store.revoke_plugin_allowlist(id, &digest).await.unwrap());
        assert!(
            resolve_third_party_in_dir(id, &store, &dir).await.is_none(),
            "a revoked pin must refuse on the next load"
        );

        // Every refuse (and the admit) wrote an audit row (no mail content).
        let audit = store.list_audit(50).await.unwrap();
        assert!(
            audit.iter().any(|r| r.action == "plugin-load-refused"),
            "a refusal is audited"
        );
        assert!(
            audit.iter().any(|r| r.action == "plugin-load-admitted"),
            "the admit is audited"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// TQ2 no-spoof: for a FIRST-PARTY id the gate NEVER consults the allowlist. Even if a
    /// (hypothetical) allowlist row existed for a first-party id, first-party resolution is
    /// unchanged; and approve-time refuses such a row outright.
    #[tokio::test]
    async fn first_party_id_is_terminal_and_never_consults_allowlist() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        // The gate resolves a first-party id via the frozen pin with an EMPTY allowlist.
        let via_gate = resolve_component("languagetool", &store)
            .await
            .expect("first-party still resolves through the gate");
        let via_first_party = first_party_component("languagetool").unwrap();
        assert_eq!(
            via_gate, via_first_party,
            "gate uses the frozen first-party pin"
        );

        // Approving a first-party id into the allowlist is refused (anti-spoof), so a
        // colliding row can never even be created.
        let digest = sha256_hex(b"attacker-supplied");
        let err = store
            .put_plugin_allowlist(
                &mw_store::new_allowlist_pin(
                    "languagetool",
                    &digest,
                    "admin@x",
                    None,
                    None,
                    None,
                    None,
                ),
                &first_party_ids(),
            )
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            mw_store::PluginAllowlistError::FirstPartyCollision(_)
        ));

        // A non-first-party id with no dir/approval ⇒ None (deny-by-default).
        assert!(resolve_component("totally-unknown", &store).await.is_none());
    }

    /// HIGH_POWER provenance gate: a first-party plugin keeps every capability; a
    /// third-party plugin has AccountBackend (and any HIGH_POWER cap) stripped, even
    /// though an admin "requested" it — while non-HIGH_POWER caps survive.
    #[test]
    fn high_power_caps_are_first_party_only() {
        use mw_plugin::Capability::{AccountBackend, Net, SpamAction, StoreKvScoped};

        // First-party: nothing stripped.
        let (kept, refused) =
            provenance_filtered_grant("bridge-graph", &[AccountBackend, Net, SpamAction]);
        assert_eq!(kept, vec![AccountBackend, Net, SpamAction]);
        assert!(
            refused.is_empty(),
            "a first-party plugin keeps HIGH_POWER caps"
        );

        // Third-party: AccountBackend refused, the rest kept — an admin cannot override it.
        let (kept, refused) = provenance_filtered_grant(
            "acme-thirdparty",
            &[AccountBackend, Net, SpamAction, StoreKvScoped],
        );
        assert_eq!(kept, vec![Net, SpamAction, StoreKvScoped]);
        assert_eq!(
            refused,
            vec![AccountBackend],
            "a third-party plugin can never be granted a HIGH_POWER cap"
        );
        assert!(is_high_power(AccountBackend));
        assert!(!is_high_power(SpamAction));
    }
}
