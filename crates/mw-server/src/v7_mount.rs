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

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};

use async_trait::async_trait;
use axum::extract::rejection::JsonRejection;
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
    AdapterConfig, AssistAudit, AssistAuditSink, AssistCapability, AssistConfig, AssistError,
    AssistGateway, ChatPayload, ChatStream, DataScope, EndpointAdapter,
};
use mw_directory::{AttrMap, Directory, DirectoryConfig, LdapEndpoint, LdapTls};
use mw_passwd::{
    Ctx, DovecotConfig, DovecotHttp, Ldap3062, LdapExopTransport, Local, LocalCredentialStore,
    PasswordChangeBackend, PasswordPolicy, Poppassd, PoppassdConfig, Result as PwResult, Secret,
    WebhookConfig, WebhookHmac,
};
use mw_plugin::{
    BasicCredentialProvider, BasicCredentials, Capability, Clock, Grant, HostServices, HttpFetcher,
    HttpReq, HttpResp, KvStore, OAuthTokenProvider, PluginHandle, PluginHost, PluginLimits,
    PluginManifest, Rng,
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

// The egress-proxy admin API (26.20 t22-e12), a `#[path]` child for the same reason
// as `admin_plugins` above: it needs `require_admin` and the already-mounted
// `extra_v7_router()`, so it costs no `lib.rs` edit and no second copy of the admin
// gate. File lives at `crates/mw-server/src/egress_admin.rs`.
#[path = "egress_admin.rs"]
mod egress_admin;

// The host-side bridge OAuth client (26.16 B1): device-code / auth-code / refresh flows
// backing the `oauth-token` import. Declared as a CHILD module of `v7_mount` (via
// `#[path]`) for the same reason as `admin_plugins` above — `lib.rs` is owned by another
// executor this wave and must not be concurrently edited, and the provider this module
// backs is injected through [`host_services`] (owned here), so it reaches the running
// host without any `lib.rs` change. The file lives at `crates/mw-server/src/oauth_client.rs`.
#[path = "oauth_client.rs"]
mod oauth_client;

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Host services (plan §2.1 §e1 injection seam) — reqwest HTTP + OAuth + KV/clock/rng
// ─────────────────────────────────────────────────────────────────────────────

/// The host `http-fetch` impl over `reqwest`/rustls. `mw-plugin` checks the `net`
/// capability and the plugin's `net_allowlist` **before** calling this, so the URL's
/// host is one the administrator listed. What is checked here is where that name
/// leads:
///
/// - [`plugin_fetch_addrs`]: http(s) only, no credentials in the URL, and the host
///   is resolved once; the connection is pinned to the addresses of that one answer
///   which the policy allows (a second DNS answer is never used), and refused when
///   there is none;
/// - the address policy is [`mw_egress::on_prem_allowed`]: private ranges are
///   reachable, because rspamd, SpamAssassin, LanguageTool and Nextcloud are usually
///   on them; link-local (which includes the `169.254.169.254` metadata address),
///   unspecified and multicast addresses never are;
/// - loopback is reachable only when the URL names it (`localhost` or a loopback
///   literal), which the administrator must have listed for the gate to pass it. A
///   name that merely resolves to loopback is refused;
/// - a redirect is followed only to the same host, never from https to http, at most
///   [`PLUGIN_FETCH_MAX_REDIRECTS`] times. Any other redirect is returned to the guest
///   as the `3xx` it is; a guest that follows it calls `http-fetch` again and meets
///   the allowlist again.
///
/// No proxy is used and the operator's egress route (`/admin/egress`) is not applied:
/// that route is read by the image proxy and the calendar-subscription fetch only.
///
/// For a guest that carries no credentials of its own (the Nextcloud plugin), the
/// host attaches the linked account's Basic auth for the matching host.
pub(crate) struct ReqwestFetcher {
    /// host → (username, password) Basic-auth injection for credential-less guests.
    host_auth: Vec<(String, (String, String))>,
}

/// How many same-host redirects one plugin `http-fetch` follows.
const PLUGIN_FETCH_MAX_REDIRECTS: usize = 5;

impl ReqwestFetcher {
    fn from_env() -> Self {
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
        Self { host_auth }
    }
}

fn host_of(url: &str) -> Option<String> {
    let after = url.split("://").nth(1).unwrap_or(url);
    after
        .split('/')
        .next()
        .map(|h| h.split(':').next().unwrap_or(h).to_lowercase())
}

/// Whether a URL host names loopback itself: `localhost`, or a loopback address
/// literal (`host` is `Url::host_str`, so an IPv6 literal is in brackets).
fn names_loopback(host: &str) -> bool {
    let bare = host
        .strip_prefix('[')
        .and_then(|h| h.strip_suffix(']'))
        .unwrap_or(host);
    bare.eq_ignore_ascii_case("localhost")
        || bare.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

/// [`mw_egress::on_prem_allowed`] plus loopback, for a URL that [`names_loopback`].
fn on_prem_or_loopback(ip: &IpAddr) -> bool {
    ip.is_loopback() || mw_egress::on_prem_allowed(ip)
}

/// The address policy for one plugin fetch, chosen from the host the URL names.
fn plugin_fetch_policy(host: &str) -> fn(&IpAddr) -> bool {
    if names_loopback(host) {
        on_prem_or_loopback
    } else {
        mw_egress::on_prem_allowed
    }
}

/// Check a plugin fetch URL and resolve its host, once, to the addresses the request
/// may connect to: every address of the answer that [`plugin_fetch_policy`] allows.
/// An IP literal is its own answer and never goes to the resolver. Unlike
/// [`mw_egress::validate_and_resolve_with`], which keeps the first allowed address,
/// this keeps them all, so a dual-stack name whose service listens on one family
/// (`localhost`, most often) still connects.
async fn plugin_fetch_addrs(url: &reqwest::Url) -> Result<Vec<std::net::SocketAddr>, &'static str> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err("only http and https URLs are fetched");
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("credentials in the URL are not allowed");
    }
    let host = url
        .host_str()
        .filter(|h| !h.is_empty())
        .ok_or("the URL has no host")?;
    let port = url.port_or_known_default().ok_or("the URL has no port")?;
    let bare = host
        .strip_prefix('[')
        .and_then(|h| h.strip_suffix(']'))
        .unwrap_or(host);
    let resolved: Vec<std::net::SocketAddr> = match bare.parse::<IpAddr>() {
        Ok(ip) => vec![std::net::SocketAddr::new(ip, port)],
        Err(_) => tokio::net::lookup_host((host, port))
            .await
            .map_err(|_| "the host does not resolve")?
            .collect(),
    };
    let policy = plugin_fetch_policy(host);
    let allowed: Vec<_> = resolved.into_iter().filter(|a| policy(&a.ip())).collect();
    if allowed.is_empty() {
        return Err("the host has no address this server may connect to");
    }
    Ok(allowed)
}

/// Whether a redirect from `from` to `to` stays on the same host without dropping
/// from https to http.
fn same_host_redirect(from: &reqwest::Url, to: &reqwest::Url) -> bool {
    from.host_str() == to.host_str() && (to.scheme() == "https" || from.scheme() == to.scheme())
}

#[async_trait]
impl HttpFetcher for ReqwestFetcher {
    async fn fetch(&self, req: HttpReq) -> std::result::Result<HttpResp, String> {
        let method = reqwest::Method::from_bytes(req.method.as_bytes())
            .map_err(|_| format!("bad method {}", req.method))?;
        let url = reqwest::Url::parse(&req.url).map_err(|_| "malformed url".to_string())?;
        let addrs = plugin_fetch_addrs(&url)
            .await
            .map_err(|why| format!("refused by the host address policy: {why}"))?;
        let host = url.host_str().unwrap_or_default().to_lowercase();
        // `.no_proxy()`: an ambient `HTTP_PROXY` would resolve the name itself, so the
        // pin below would not be what the request reaches. See `mw_egress::harden_client`.
        let client = reqwest::Client::builder()
            .no_proxy()
            .resolve_to_addrs(&host, &addrs)
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                let stays = attempt
                    .previous()
                    .last()
                    .is_some_and(|from| same_host_redirect(from, attempt.url()));
                if stays && attempt.previous().len() <= PLUGIN_FETCH_MAX_REDIRECTS {
                    attempt.follow()
                } else {
                    attempt.stop()
                }
            }))
            .build()
            .map_err(|e| e.to_string())?;
        let mut rb = client.request(method, url);
        for (k, v) in &req.headers {
            rb = rb.header(k.as_str(), v.as_str());
        }
        // Inject the linked-account auth for a credential-less allowlisted guest.
        if let Some((_, (user, pw))) = self.host_auth.iter().find(|(h, _)| *h == host) {
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

/// The host `oauth-token` provider (26.16 B1). Bridges acquire short-lived tokens
/// through this; the host holds the long-lived secret, the guest never sees it. On a
/// guest call it returns the account's cached 0018 access token when unexpired, else
/// refreshes it (over the host `reqwest`(rustls) client, against the provider's token
/// endpoint) and re-caches the pair sealed at rest — see
/// [`oauth_client::acquire_access_token`]. The first refresh token is minted out of
/// band by the admin device-code / authorization-code enrolment; until an account is
/// enrolled, a guest call fails cleanly (it can't drive an interactive flow).
///
/// Config (provider/client id/tenant/scopes) is the account's NON-secret
/// `bridge_accounts.oauth_ref`; the optional confidential-client secret is read once
/// from the deployment env (`MW_BRIDGE_OAUTH_CLIENT_SECRET`) — a public PKCE client
/// leaves it unset. The refresh token, the one long-lived secret, lives only sealed.
pub(crate) struct StoreOAuthProvider {
    store: Store,
    poster: Arc<oauth_client::ReqwestPoster>,
    /// Deployment-wide confidential-client secret; `None` for a public PKCE client.
    client_secret: Option<String>,
}

#[async_trait]
impl OAuthTokenProvider for StoreOAuthProvider {
    async fn token(&self, account: &str) -> std::result::Result<String, String> {
        oauth_client::acquire_access_token(
            &self.store,
            self.poster.as_ref(),
            self.client_secret.as_deref(),
            account,
        )
        .await
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
pub(crate) fn plugin_kv_limits() -> PluginKvLimits {
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

/// Build the host-service bundle e14 injects: the [`ReqwestFetcher`] (behind the
/// allowlist check in `mw-plugin`), the store-backed bridge OAuth provider (B1 —
/// cached-or-refreshed 0018 tokens over the host reqwest/rustls client), the
/// store-backed EWS per-account basic-credential provider, and scoped KV/clock/rng.
pub(crate) fn host_services(store: &Store) -> HostServices {
    // `.no_proxy()`: the OAuth poster carries bridge refresh tokens, and an ambient
    // `HTTP_PROXY` would see the token exchange. See `mw_egress::harden_client`.
    let oauth_http = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("reqwest client builds");
    HostServices {
        http: Arc::new(ReqwestFetcher::from_env()),
        oauth: Arc::new(StoreOAuthProvider {
            store: store.clone(),
            poster: Arc::new(oauth_client::ReqwestPoster::new(oauth_http)),
            client_secret: env("MW_BRIDGE_OAUTH_CLIENT_SECRET"),
        }),
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

/// The variable that selects the password-change backend.
const PASSWD_BACKEND_ENV: &str = "MW_PASSWD_BACKEND";

/// A password-backend setting that cannot be used. Each variant names the variable
/// so the operator can fix it from the message alone; none carries the value of a
/// backend setting, because two of those variables hold secrets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasswdBackendConfigError {
    /// `MW_PASSWD_BACKEND` is set to something that is not a backend name.
    UnknownBackend(String),
    /// The selected backend needs this variable and it is unset or empty.
    Missing {
        backend: &'static str,
        var: &'static str,
    },
    /// This variable is set but cannot be used; `reason` says why.
    Invalid {
        var: &'static str,
        reason: &'static str,
    },
}

impl std::fmt::Display for PasswdBackendConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownBackend(name) => write!(
                f,
                "{PASSWD_BACKEND_ENV}={name:?} is not a password backend \
                 (expected local, ldap3062, dovecot, poppassd or webhook)"
            ),
            Self::Missing { backend, var } => {
                write!(f, "{PASSWD_BACKEND_ENV}={backend} requires {var}")
            }
            Self::Invalid { var, reason } => write!(f, "{var} {reason}"),
        }
    }
}

impl std::error::Error for PasswdBackendConfigError {}

/// The shortest HMAC secret the webhook backend accepts, in bytes.
const WEBHOOK_SECRET_MIN_BYTES: usize = 16;

/// The poppassd port when `MW_PASSWD_POPPASSD_PORT` is unset (the port poppassd
/// conventionally listens on).
const POPPASSD_DEFAULT_PORT: u16 = 106;

/// Require an `http://` or `https://` URL with a host. A plain-HTTP URL is accepted
/// (doveadm and in-cluster webhooks are commonly reached that way) and logged, since
/// the request body carries the new password.
fn passwd_http_url(
    backend: &'static str,
    var: &'static str,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<String, PasswdBackendConfigError> {
    let url = lookup(var).ok_or(PasswdBackendConfigError::Missing { backend, var })?;
    let is_http = url.starts_with("http://") || url.starts_with("https://");
    if !is_http || host_of(&url).is_none_or(|h| h.is_empty()) {
        return Err(PasswdBackendConfigError::Invalid {
            var,
            reason: "must be an http:// or https:// URL with a host",
        });
    }
    if url.starts_with("http://") {
        tracing::warn!("{var} is plain HTTP: new passwords travel to it unencrypted");
    }
    Ok(url)
}

/// A backend that checks the caller's current password before delegating.
///
/// The doveadm and webhook backends change a password with an administrative
/// credential and never look at the old one (`mw_passwd::DovecotHttp::change` and
/// `WebhookHmac::change` both ignore it). Behind `POST /api/password` that would let
/// anyone holding a session cookie replace the password without knowing it. The
/// login that created a session stored the password it verified upstream as that
/// session's sealed credentials, so the current password is checked against those.
///
/// An account with no stored session cannot be a `POST /api/password` caller (the
/// handler resolved one to get here); that case is the operator's
/// `mailwoman password` command, which runs on the host and is let through.
struct CurrentPasswordCheck<B> {
    inner: B,
    store: Store,
}

#[async_trait]
impl<B: PasswordChangeBackend> PasswordChangeBackend for CurrentPasswordCheck<B> {
    async fn change(
        &self,
        ctx: &Ctx,
        old: Secret,
        new: Secret,
    ) -> PwResult<mw_passwd::PasswordChangeOutcome> {
        let sessions = self
            .store
            .sessions_by_account(&ctx.account_id)
            .await
            .map_err(|e| mw_passwd::PasswordError::Transport(e.to_string()))?;
        // Compared as digests so the comparison time does not depend on how much of
        // the stored password the guess matched.
        let given = Sha256::digest(old.expose().as_bytes());
        let matches =
            |s: &mw_store::Session| Sha256::digest(s.credentials.password.as_bytes()) == given;
        if !sessions.is_empty() && !sessions.iter().any(matches) {
            return Err(mw_passwd::PasswordError::WrongCurrent);
        }
        self.inner.change(ctx, old, new).await
    }

    fn policy(&self) -> PasswordPolicy {
        self.inner.policy()
    }

    fn kind(&self) -> mw_passwd::BackendKind {
        self.inner.kind()
    }
}

/// Build the password-change backend from `lookup`, which reads one setting by its
/// environment-variable name. `MW_PASSWD_BACKEND` selects it:
///
/// | value | backend | settings |
/// |---|---|---|
/// | unset, `local` | Argon2id hash in the store | none |
/// | `ldap3062` | RFC 3062 PasswordModify | `MW_PASSWD_LDAP_URL` (required), `MW_PASSWD_LDAP_STARTTLS=1`, `MW_PASSWD_LDAP_BIND_DN`, `MW_PASSWD_LDAP_BIND_PW` |
/// | `dovecot` | doveadm HTTP API | `MW_PASSWD_DOVECOT_URL`, `MW_PASSWD_DOVECOT_API_KEY` (both required), `MW_PASSWD_DOVECOT_COMMAND` (default `pw`) |
/// | `poppassd` | poppassd line protocol | `MW_PASSWD_POPPASSD_HOST` (required), `MW_PASSWD_POPPASSD_PORT` (default 106) |
/// | `webhook` | HMAC-SHA256-signed POST | `MW_PASSWD_WEBHOOK_URL`, `MW_PASSWD_WEBHOOK_SECRET` (both required; the secret is at least 16 bytes) |
///
/// Any other value, a missing required setting or an unusable one is an error
/// naming the variable. Nothing falls back to `local`: a change reported as done
/// against a local hash that no mail server reads would be a false success.
///
/// # Errors
/// [`PasswdBackendConfigError`] as described above.
pub fn passwd_backend_from(
    store: &Store,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Arc<dyn PasswordChangeBackend>, PasswdBackendConfigError> {
    use PasswdBackendConfigError::{Invalid, Missing, UnknownBackend};
    let policy = PasswordPolicy::default();
    let lookup = |key: &str| lookup(key).filter(|v| !v.is_empty());
    let selected = lookup(PASSWD_BACKEND_ENV);
    match selected.as_deref() {
        None | Some("local") => Ok(Arc::new(Local::new(
            StoreCredStore {
                store: store.clone(),
            },
            policy,
        ))),
        Some("ldap3062") => {
            let transport = Ldap3062Transport {
                url: lookup("MW_PASSWD_LDAP_URL").ok_or(Missing {
                    backend: "ldap3062",
                    var: "MW_PASSWD_LDAP_URL",
                })?,
                starttls: lookup("MW_PASSWD_LDAP_STARTTLS").as_deref() == Some("1"),
                bind_dn: lookup("MW_PASSWD_LDAP_BIND_DN"),
                bind_pw: lookup("MW_PASSWD_LDAP_BIND_PW"),
            };
            Ok(Arc::new(Ldap3062::new(transport, policy)))
        }
        Some("dovecot") => {
            let url = passwd_http_url("dovecot", "MW_PASSWD_DOVECOT_URL", &lookup)?;
            let api_key = lookup("MW_PASSWD_DOVECOT_API_KEY").ok_or(Missing {
                backend: "dovecot",
                var: "MW_PASSWD_DOVECOT_API_KEY",
            })?;
            let mut config = DovecotConfig::new(url, api_key);
            if let Some(command) = lookup("MW_PASSWD_DOVECOT_COMMAND") {
                config.command = command;
            }
            config.policy = policy;
            Ok(Arc::new(CurrentPasswordCheck {
                inner: DovecotHttp::new(config),
                store: store.clone(),
            }))
        }
        Some("poppassd") => {
            let host = lookup("MW_PASSWD_POPPASSD_HOST").ok_or(Missing {
                backend: "poppassd",
                var: "MW_PASSWD_POPPASSD_HOST",
            })?;
            let port = match lookup("MW_PASSWD_POPPASSD_PORT") {
                None => POPPASSD_DEFAULT_PORT,
                Some(raw) => match raw.trim().parse::<u16>() {
                    Ok(p) if p != 0 => p,
                    _ => {
                        return Err(Invalid {
                            var: "MW_PASSWD_POPPASSD_PORT",
                            reason: "must be a TCP port from 1 to 65535",
                        });
                    }
                },
            };
            let mut config = PoppassdConfig::new(host, port);
            config.policy = policy;
            // No `CurrentPasswordCheck`: in the protocol's `pass` step the server
            // verifies the current password itself.
            Ok(Arc::new(Poppassd::new(config)))
        }
        Some("webhook") => {
            let url = passwd_http_url("webhook", "MW_PASSWD_WEBHOOK_URL", &lookup)?;
            let secret = lookup("MW_PASSWD_WEBHOOK_SECRET").ok_or(Missing {
                backend: "webhook",
                var: "MW_PASSWD_WEBHOOK_SECRET",
            })?;
            if secret.len() < WEBHOOK_SECRET_MIN_BYTES {
                return Err(Invalid {
                    var: "MW_PASSWD_WEBHOOK_SECRET",
                    reason: "must be at least 16 bytes",
                });
            }
            let mut config = WebhookConfig::new(url, secret.into_bytes());
            config.policy = policy;
            Ok(Arc::new(CurrentPasswordCheck {
                inner: WebhookHmac::new(config),
                store: store.clone(),
            }))
        }
        Some(other) => Err(UnknownBackend(other.to_string())),
    }
}

/// [`passwd_backend_from`] over the process environment.
///
/// # Errors
/// [`PasswdBackendConfigError`] when the selected backend is misconfigured.
pub fn try_build_passwd_backend(
    store: &Store,
) -> Result<Arc<dyn PasswordChangeBackend>, PasswdBackendConfigError> {
    passwd_backend_from(store, &env)
}

/// The password-change backend the mount injects into `POST /api/password` and the
/// `mailwoman password` command uses, built by [`try_build_passwd_backend`].
///
/// # Panics
/// When the configured backend cannot be built. Both callers run this during
/// start-up, before the server accepts a connection, so the process stops with the
/// [`PasswdBackendConfigError`] message instead of serving a backend other than the
/// one the operator selected.
pub fn build_passwd_backend(store: &Store) -> Arc<dyn PasswordChangeBackend> {
    try_build_passwd_backend(store)
        .unwrap_or_else(|e| panic!("password backend configuration: {e}"))
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
    live: Arc<AssistLive>,
}

impl AssistAuditSink for StoreAssistAudit {
    fn record(&self, row: AssistAudit) {
        // The gateway audits before it dispatches, and a stopped gateway refuses at
        // dispatch ([`StoppableAdapter`]). A row written here would say content
        // reached the endpoint when nothing left.
        if self.live.is_stopped() {
            return;
        }
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

/// The configured embedding model's id, recorded at [`build_assist`] so
/// [`AssistHookAdapter::from_gateway`] can report it to the engine (A8).
///
/// A module static rather than a return value or constructor argument because both
/// would change the `build_assist` / `from_gateway` call sites, and those live in
/// `crates/mw-server/src/lib.rs`, which another executor holds this wave. `mw-server`
/// builds Assist exactly once at mount, before the adapter is constructed, so the
/// ordering is not a race — and reading a stale/empty id is harmless: it only
/// disables the model check, leaving dimensionality as the guard.
static EMBED_MODEL: std::sync::RwLock<String> = std::sync::RwLock::new(String::new());

/// The embedding model id an [`AdapterConfig`] will use, or `""` for an adapter
/// whose model is not nameable (the local-process adapter runs whatever the
/// configured binary runs).
fn embed_model_of(adapter: Option<&AdapterConfig>) -> String {
    match adapter {
        Some(AdapterConfig::OpenAiCompatible { embed_model, .. }) => embed_model.clone(),
        _ => String::new(),
    }
}

/// The environment variable an operator sets to bound Assist egress.
const ASSIST_RATE_LIMIT_ENV: &str = "MW_ASSIST_RATE_LIMIT_PER_MIN";

/// The per-account Assist request budget, in outbound endpoint requests per minute.
///
/// The 0008 `assist_config` table has no column for this, so until a migration adds
/// one the setting is an environment variable — the same shape as every other
/// deployment-level Assist/egress control (`MW_MCP_RESOURCE`, `MW_RENDER_JAIL`). It
/// was previously hardcoded `None` with no operator-reachable setting anywhere, which
/// made it a dead control end to end rather than a default someone had not changed.
///
/// **Sizing.** The unit is one outbound request, because bounding third-party API cost
/// and bulk egress volume is the whole point; a scheme that charged a multi-request
/// pass as a single unit would stop bounding either. So a *cold-cache* semantic search
/// costs up to `1 + mw_engine::search_semantic::LAZY_FILL_MAX` = 33 requests (one for
/// the query, one per document embedded on demand), and converges toward 1 as the
/// embedding cache fills. A chat invocation costs 1. Size the limit accordingly: 60
/// permits roughly two cold semantic searches a minute, per account.
///
/// - unset or empty ⇒ `None`, unlimited — the pre-26.19 behaviour, unchanged.
/// - `0` ⇒ an explicit hard stop: every Assist request is refused. A kill switch that
///   leaves the rest of the deployment running.
/// - unparseable ⇒ `None` with a warning, because failing a server boot over a typo in
///   an optional knob is worse than running without the knob.
fn assist_rate_limit() -> Option<u32> {
    parse_rate_limit(env(ASSIST_RATE_LIMIT_ENV).as_deref())
}

/// The parsing half of [`assist_rate_limit`], split out so it is testable without
/// mutating process environment (`set_var` is `unsafe` in edition 2024, and an env
/// mutation is visible to every other test in the binary).
fn parse_rate_limit(raw: Option<&str>) -> Option<u32> {
    let raw = raw?;
    match raw.trim().parse::<u32>() {
        Ok(n) => Some(n),
        Err(_) => {
            tracing::warn!(
                "{ASSIST_RATE_LIMIT_ENV}={raw:?} is not a non-negative integer; \
                 Assist runs without a rate limit"
            );
            None
        }
    }
}

/// Build the Assist gateway from the 0008 `assist_config` deployment row. Absent /
/// disabled ⇒ `AssistConfig::default()` (the gateway reports `Disabled` and the web
/// hides all Assist UI).
pub(crate) async fn build_assist(store: &Store) -> (AssistHandle, Vec<AssistCapability>) {
    let row = store.get_assist_config("deployment").await.ok().flatten();
    let mut config = stored_assist_config(row.as_ref());
    config.rate_limit_per_min = assist_rate_limit();
    if let Ok(mut w) = EMBED_MODEL.write() {
        *w = embed_model_of(config.adapter.as_ref());
    }
    let granted = config.capability_grants.clone();
    let live = Arc::new(AssistLive {
        stopped: AtomicBool::new(false),
        booted: assist_admin_wire(&config),
    });
    // The gateway's own adapter is replaced by the same adapter behind the stop
    // check, so every dispatch path (chat, embed, transcribe) passes through it.
    let adapter = config.adapter.as_ref().and_then(AdapterConfig::build);
    let mut gateway = AssistGateway::new(config).with_audit(Arc::new(StoreAssistAudit {
        store: store.clone(),
        live: Arc::clone(&live),
    }));
    if let Some(inner) = adapter {
        gateway = gateway.with_adapter(Arc::new(StoppableAdapter {
            inner,
            live: Arc::clone(&live),
        }));
    }
    let gateway = Arc::new(gateway);
    register_assist_live(&gateway, live);
    (gateway, granted)
}

/// The [`AssistConfig`] a stored `assist_config` row describes (`None` ⇒ the
/// default: off, no adapter, no grants, the deny ceiling). A column that does not
/// parse reads as its default. [`build_assist`] and `GET /admin/assist` both read
/// the row through this, so the panel shows what the gateway would be built from.
/// `rate_limit_per_min` is not in the row; the caller sets it.
fn stored_assist_config(row: Option<&mw_store::AssistConfigRow>) -> AssistConfig {
    match row {
        Some(r) => AssistConfig {
            enabled: r.enabled,
            capability_grants: serde_json::from_str(&r.capability_grants_json).unwrap_or_default(),
            data_ceiling: serde_json::from_str(&r.data_ceilings_json).unwrap_or_default(),
            adapter: parse_adapter(&r.adapters_json),
            rate_limit_per_min: None,
        },
        None => AssistConfig::default(),
    }
}

/// What the admin routes need to know about one running gateway.
struct AssistLive {
    /// Set when an administrator turns Assist off. Checked on every dispatch.
    stopped: AtomicBool,
    /// The configuration the gateway was built from, in the admin wire shape.
    booted: serde_json::Value,
}

impl AssistLive {
    fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }
}

/// The [`AssistLive`] of every gateway [`build_assist`] has built and that is still
/// alive, keyed by the gateway itself.
///
/// A side table because the gateway type belongs to `mw-assist` and the handle type
/// to `assist.rs`, and the admin handlers receive only that handle. Keyed per
/// gateway rather than one process-wide flag so that two apps in one process (the
/// integration tests) do not stop each other.
static ASSIST_LIVE: Mutex<Vec<(Weak<AssistGateway>, Arc<AssistLive>)>> = Mutex::new(Vec::new());

fn register_assist_live(gateway: &AssistHandle, live: Arc<AssistLive>) {
    if let Ok(mut table) = ASSIST_LIVE.lock() {
        table.retain(|(g, _)| g.strong_count() > 0);
        table.push((Arc::downgrade(gateway), live));
    }
}

fn assist_live(gateway: &AssistHandle) -> Option<Arc<AssistLive>> {
    let table = ASSIST_LIVE.lock().ok()?;
    table
        .iter()
        .find(|(g, _)| std::ptr::eq(g.as_ptr(), Arc::as_ptr(gateway)))
        .map(|(_, live)| Arc::clone(live))
}

/// Whether this gateway answers Assist requests right now: built enabled with an
/// adapter, and not stopped since.
pub(crate) fn assist_running(gateway: &AssistHandle) -> bool {
    gateway.is_enabled() && !assist_live(gateway).is_some_and(|l| l.is_stopped())
}

/// The configured adapter behind the stop check. A stopped gateway answers
/// [`AssistError::Disabled`] — the same refusal as one built disabled — before
/// anything is sent.
struct StoppableAdapter {
    inner: Arc<dyn EndpointAdapter>,
    live: Arc<AssistLive>,
}

impl StoppableAdapter {
    fn check(&self) -> mw_assist::Result<()> {
        if self.live.is_stopped() {
            Err(AssistError::Disabled)
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl EndpointAdapter for StoppableAdapter {
    async fn chat(&self, payload: &ChatPayload) -> mw_assist::Result<ChatStream> {
        self.check()?;
        self.inner.chat(payload).await
    }
    async fn embed(&self, input: &str) -> mw_assist::Result<Vec<f32>> {
        self.check()?;
        self.inner.embed(input).await
    }
    async fn transcribe(&self, audio: &[u8], mime: &str) -> mw_assist::Result<String> {
        self.check()?;
        self.inner.transcribe(audio, mime).await
    }
    fn host(&self) -> String {
        self.inner.host()
    }
}

/// The engine-side Assist hook adapter (content-free posture only, §21.1). Captures
/// the enabled flag + the granted capability wire names at mount so the engine can
/// gate which Assist affordances the UI shows without a cycle onto `mw-assist`.
pub(crate) struct AssistHookAdapter {
    enabled: bool,
    /// The gateway's stop flag, so the engine stops offering Assist when an
    /// administrator turns it off. `None` for a gateway `build_assist` did not build.
    live: Option<Arc<AssistLive>>,
    granted: Vec<String>,
    /// A8 (26.19): the gateway itself, kept so the hook can hand the engine an
    /// embedding provider. `None` unless Assist is enabled AND `search-semantic` is
    /// granted, which is what keeps semantic re-rank off by default.
    embeddings: Option<Arc<GatewayEmbeddings>>,
}

impl AssistHookAdapter {
    /// Takes the `Arc` (not `&AssistGateway`) so the embedding provider can share
    /// the same gateway — its enforcement pipeline is the point. The `mw-server`
    /// mount site passes `&assist`, which is already an `&AssistHandle`, so this is
    /// signature-compatible with the existing call.
    pub(crate) fn from_gateway(gateway: &AssistHandle, granted: &[AssistCapability]) -> Self {
        let semantic = gateway.is_enabled() && granted.contains(&AssistCapability::SearchSemantic);
        Self {
            enabled: gateway.is_enabled(),
            live: assist_live(gateway),
            granted: granted.iter().map(|c| capability_wire(*c)).collect(),
            embeddings: semantic.then(|| {
                Arc::new(GatewayEmbeddings {
                    gateway: Arc::clone(gateway),
                    model: EMBED_MODEL.read().map(|m| m.clone()).unwrap_or_default(),
                })
            }),
        }
    }
}

impl AssistHookAdapter {
    fn is_stopped(&self) -> bool {
        self.live.as_ref().is_some_and(|l| l.is_stopped())
    }
}

impl mw_engine::AssistHook for AssistHookAdapter {
    fn is_enabled(&self) -> bool {
        self.enabled && !self.is_stopped()
    }
    fn granted_capabilities(&self) -> Vec<String> {
        if self.is_stopped() {
            return Vec::new();
        }
        self.granted.clone()
    }
    fn embedding_provider(&self) -> Option<Arc<dyn mw_engine::EmbeddingProvider>> {
        if self.is_stopped() {
            return None;
        }
        self.embeddings
            .as_ref()
            .map(|e| Arc::clone(e) as Arc<dyn mw_engine::EmbeddingProvider>)
    }
}

/// A8 (26.19): the engine's [`mw_engine::EmbeddingProvider`] over the Assist gateway.
///
/// Deliberately a thin pass-through: every embedding request goes through
/// `AssistGateway::embed`, so it inherits the full §14 pipeline — capability check,
/// data-class ceiling clamp, rate limit, and the content-free audit row naming the
/// capability, the scope and the endpoint host. The engine gets no way to reach an
/// endpoint that the gateway would not have allowed.
pub(crate) struct GatewayEmbeddings {
    gateway: AssistHandle,
    model: String,
}

/// Attachments and E2EE-decrypted content stay excluded from a re-rank embedding
/// regardless of what the deployment ceiling allows, because a search re-rank has no
/// business reading them.
///
/// This constant is the single source for both halves of that claim: the [`DataScope`]
/// the request is dispatched under (which is what the audit row describes) and the
/// [`mw_engine::search_semantic::EmbedScope`] the engine builds the payload from. They
/// were previously independent, and only the first one existed — so the exclusion was
/// recorded in the audit trail and not applied to the request. Keeping them derived
/// from one value is what makes the audit row a description of what actually left.
const RERANK_INCLUDE_ATTACHMENTS: bool = false;

impl GatewayEmbeddings {
    /// The narrowest scope that still names the account for the audit row.
    fn scope(&self, account_id: &str) -> DataScope {
        DataScope {
            accounts: vec![account_id.to_string()],
            folders: Vec::new(),
            include_e2ee: false,
            include_attachments: RERANK_INCLUDE_ATTACHMENTS,
        }
    }
}

#[async_trait::async_trait]
impl mw_engine::EmbeddingProvider for GatewayEmbeddings {
    async fn embed(&self, account_id: &str, text: &str) -> Result<Vec<f32>, String> {
        self.gateway
            .embed(self.scope(account_id), text)
            .await
            .map_err(|e| e.to_string())
    }

    fn model_id(&self) -> String {
        self.model.clone()
    }

    fn content_scope(&self) -> mw_engine::search_semantic::EmbedScope {
        mw_engine::search_semantic::EmbedScope {
            include_attachments: RERANK_INCLUDE_ATTACHMENTS,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Plugin host extension (seeded from the 0008 registry)
// ─────────────────────────────────────────────────────────────────────────────

/// Map a 0008 [`PluginRow`] to a [`PluginManifest`] for the in-process host.
pub(crate) fn manifest_of(row: &PluginRow) -> PluginManifest {
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
            0x97, 0xf2, 0xe9, 0x3b, 0x70, 0x84, 0xfc, 0xc8, 0x51, 0xa7, 0xf4, 0xf4, 0x58, 0xe2,
            0xbf, 0x0f, 0x81, 0x0a, 0x42, 0x96, 0x15, 0x98, 0xfb, 0x71, 0x24, 0x3f, 0xfb, 0x19,
            0xfd, 0x51, 0xc3, 0xcb,
        ],
    ),
    (
        "bridge-ews",
        [
            0x17, 0x7d, 0x3b, 0x66, 0xfc, 0xc1, 0x38, 0xcb, 0x9a, 0xe5, 0x10, 0x38, 0xca, 0xe1,
            0xe2, 0x3d, 0xfe, 0xb3, 0x54, 0x76, 0x39, 0x85, 0xa4, 0xfc, 0x83, 0x04, 0xb7, 0x67,
            0xf8, 0x6d, 0x12, 0xd8,
        ],
    ),
    (
        "bridge-gmail",
        [
            0xc9, 0xfb, 0xbc, 0x80, 0x79, 0x3a, 0xd3, 0x1f, 0xa3, 0xbb, 0x39, 0x03, 0x79, 0x86,
            0x1e, 0x63, 0xb4, 0x37, 0x03, 0x85, 0xac, 0x34, 0x83, 0x3b, 0x64, 0x0c, 0x67, 0xaa,
            0x3d, 0x15, 0x8d, 0x47,
        ],
    ),
    (
        "languagetool",
        [
            0x97, 0x79, 0xd5, 0xc4, 0x38, 0x31, 0x73, 0x5b, 0xcc, 0x4a, 0xcb, 0xab, 0xf0, 0xa9,
            0x7a, 0x4b, 0xd5, 0xbf, 0x0b, 0xc2, 0xe0, 0x2e, 0xdc, 0x78, 0xc2, 0x37, 0x64, 0xad,
            0xa1, 0x37, 0xc9, 0x24,
        ],
    ),
    (
        "nextcloud",
        [
            0x4c, 0xff, 0xbc, 0x6a, 0xe9, 0x17, 0x8e, 0xa2, 0x70, 0x6a, 0xbf, 0x20, 0x2b, 0x97,
            0x7d, 0x43, 0xfe, 0xd3, 0x67, 0x39, 0xe2, 0x72, 0x28, 0xfc, 0xbe, 0x6d, 0x25, 0x70,
            0xc1, 0x7b, 0xf4, 0x08,
        ],
    ),
    (
        "spam-rspamd",
        [
            0x3d, 0xc8, 0x5b, 0x8a, 0x83, 0x53, 0x96, 0xc6, 0xf8, 0xb9, 0xa2, 0xe5, 0xec, 0x3d,
            0x4e, 0xcf, 0x49, 0x86, 0x3f, 0x56, 0xd8, 0x35, 0xd9, 0x2d, 0x0a, 0x0c, 0x0a, 0xb0,
            0xc6, 0x65, 0xa4, 0x66,
        ],
    ),
    (
        "spam-spamassassin",
        [
            0x63, 0xaf, 0x02, 0x89, 0x75, 0x09, 0xf1, 0x7c, 0xff, 0x9d, 0x71, 0xa1, 0x8d, 0xea,
            0xca, 0xc7, 0xa5, 0x6d, 0xf9, 0x66, 0xb7, 0x71, 0x38, 0xb0, 0x21, 0xe5, 0x7e, 0x5b,
            0x66, 0x67, 0xbf, 0x3c,
        ],
    ),
];

/// The manifest of one first-party component, compiled in beside its digest pin.
///
/// A component's bytes do not carry their manifest, so for a first-party id this
/// table is the manifest: an administrator registers the id and cannot declare a
/// capability the component was not built to use. `net_allowlist` is the default;
/// registration may replace it with the deployment's own hosts.
/// `first_party_manifests_match_the_plugin_toml_files` holds the table to
/// `plugins/<id>/plugin.toml`.
struct FirstPartyManifest {
    id: &'static str,
    name: &'static str,
    version: &'static str,
    capabilities: &'static [Capability],
    net_allowlist: &'static [&'static str],
    memory_mb: u32,
    deadline_ms: u64,
}

const FIRST_PARTY_MANIFESTS: &[FirstPartyManifest] = &[
    FirstPartyManifest {
        id: "bridge-graph",
        name: "Microsoft Graph bridge",
        version: "26.8.0",
        capabilities: &[
            Capability::AccountBackend,
            Capability::Net,
            Capability::AddrbookSource,
            Capability::StoreKvScoped,
        ],
        net_allowlist: &["graph.microsoft.com", "login.microsoftonline.com"],
        memory_mb: 64,
        deadline_ms: 15_000,
    },
    FirstPartyManifest {
        id: "bridge-ews",
        name: "Exchange EWS bridge",
        version: "26.8.0",
        capabilities: &[
            Capability::AccountBackend,
            Capability::Net,
            Capability::AddrbookSource,
        ],
        // `plugin.toml` lists the fixture host `ews.example.com`. An Exchange server
        // is per deployment: `load_plugin_backends` adds each bound account's
        // `ews_account_cred.endpoint_host`, and registration may list hosts too.
        net_allowlist: &[],
        memory_mb: 128,
        deadline_ms: 10_000,
    },
    FirstPartyManifest {
        id: "bridge-gmail",
        name: "Gmail API bridge",
        version: "26.8.0",
        capabilities: &[Capability::AccountBackend, Capability::Net],
        net_allowlist: &["gmail.googleapis.com", "oauth2.googleapis.com"],
        memory_mb: 64,
        deadline_ms: 15_000,
    },
    FirstPartyManifest {
        id: "languagetool",
        name: "LanguageTool grammar",
        version: "26.8.0",
        capabilities: &[Capability::DlpDetector, Capability::Net],
        net_allowlist: &["api.languagetool.org"],
        memory_mb: 32,
        deadline_ms: 10_000,
    },
    FirstPartyManifest {
        id: "nextcloud",
        name: "Nextcloud share links",
        version: "26.8.0",
        capabilities: &[Capability::MessagePipeline, Capability::Net],
        net_allowlist: &[],
        memory_mb: 32,
        deadline_ms: 15_000,
    },
    FirstPartyManifest {
        id: "spam-rspamd",
        name: "Rspamd spam classifier",
        version: "26.10.0",
        capabilities: &[
            Capability::SpamAction,
            Capability::Net,
            Capability::StoreKvScoped,
        ],
        net_allowlist: &["rspamd"],
        memory_mb: 32,
        deadline_ms: 10_000,
    },
    FirstPartyManifest {
        id: "spam-spamassassin",
        name: "SpamAssassin spam classifier",
        version: "26.10.0",
        capabilities: &[
            Capability::SpamAction,
            Capability::Net,
            Capability::StoreKvScoped,
        ],
        net_allowlist: &["spamassassin"],
        memory_mb: 32,
        deadline_ms: 10_000,
    },
];

/// The compiled-in manifest for a first-party id, unsigned, with its default
/// `net_allowlist`. `None` for any other id, and for the `nextcloud-plugin` alias:
/// a component is registered under its own id.
pub(crate) fn first_party_manifest(plugin_id: &str) -> Option<PluginManifest> {
    let m = FIRST_PARTY_MANIFESTS.iter().find(|m| m.id == plugin_id)?;
    Some(PluginManifest {
        id: m.id.to_string(),
        name: m.name.to_string(),
        version: m.version.to_string(),
        signature: None,
        capabilities: m.capabilities.to_vec(),
        net_allowlist: m.net_allowlist.iter().map(|h| h.to_string()).collect(),
        limits: PluginLimits {
            memory_mb: m.memory_mb,
            deadline_ms: m.deadline_ms,
            fuel: None,
        },
    })
}

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
    first_party_component_in(plugin_id, &plugin_dirs())
}

/// [`first_party_component`] over an explicit directory list.
fn first_party_component_in(plugin_id: &str, dirs: &[PathBuf]) -> Option<Vec<u8>> {
    let (id, expected) = first_party_digest(plugin_id)?;
    let file = format!("{id}.wasm");
    let mut tried = Vec::new();
    for dir in dirs {
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
/// This gate decides which BYTES may be handed to `PluginHost::load`. Whether a
/// component without a signature may then load is [`TrustPolicy`]'s decision, and
/// `PluginHost::load` verifies a signature the manifest does carry against the host
/// trust root — which is empty in this server, so a signed manifest fails closed.
pub(crate) async fn resolve_component(plugin_id: &str, store: &Store) -> Option<Vec<u8>> {
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
pub(crate) fn is_first_party_plugin(plugin_id: &str) -> bool {
    first_party_digest(plugin_id).is_some()
}

/// Filter a requested capability set by provenance. A first-party plugin keeps every
/// capability; a third-party plugin has EVERY HIGH_POWER capability stripped (returned in
/// `refused`). Returns `(kept, refused)`. This is the provenance gate; it runs where the
/// runtime [`Grant`] is built, so a third-party plugin never receives a HIGH_POWER
/// capability at runtime regardless of what an admin persisted.
pub(crate) fn provenance_filtered_grant(
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

// ── Trust policy, grant computation and the load plan ──────────────────────────────────

/// Why a component's bytes are trusted to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrustPolicy {
    /// A first-party id. [`first_party_component`] returns bytes only when they hash
    /// to the SHA-256 compiled into this binary ([`FIRST_PARTY_DIGESTS`]); that match
    /// is the trust decision. No administrator flag takes part in it and the stored
    /// allow-unsigned flag is not read for these ids.
    FirstPartyDigestPin,
    /// Any other id. Its bytes are admitted only when they hash to a digest an
    /// administrator pinned in the 0014 allowlist
    /// ([`resolve_third_party_component`]); a component whose manifest carries no
    /// signature additionally needs the stored per-plugin allow-unsigned flag
    /// (`Store::plugin_allow_unsigned`).
    AdminPinnedDigest { allow_unsigned: bool },
}

impl TrustPolicy {
    /// The policy for `plugin_id`. A store error reading the flag reads as "not
    /// allowed".
    pub(crate) async fn of(store: &Store, plugin_id: &str) -> Self {
        if is_first_party_plugin(plugin_id) {
            return Self::FirstPartyDigestPin;
        }
        let allow_unsigned = store
            .plugin_allow_unsigned(plugin_id)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("allow-unsigned flag read failed for '{plugin_id}': {e}");
                false
            });
        Self::AdminPinnedDigest { allow_unsigned }
    }

    /// The name of this policy in the admin API.
    pub(crate) fn wire(self) -> &'static str {
        match self {
            Self::FirstPartyDigestPin => "first-party-digest",
            Self::AdminPinnedDigest { .. } => "admin-pinned-digest",
        }
    }

    /// Whether a component whose manifest carries no signature may load.
    ///
    /// This is the value [`plan_load`] passes as `mw_plugin::Grant::allow_unsigned`,
    /// the one switch `PluginHost::load` has for such a component. For a first-party
    /// id it is `true` because of the digest pin, not because anyone allowed it.
    pub(crate) fn admits_unsigned(self) -> bool {
        match self {
            Self::FirstPartyDigestPin => true,
            Self::AdminPinnedDigest { allow_unsigned } => allow_unsigned,
        }
    }
}

/// The kebab-case name of a capability, as stored in `plugin_grants.capability` and
/// used by the admin API.
pub(crate) fn capability_name(cap: Capability) -> String {
    serde_json::to_value(cap)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

/// The capabilities one load of `manifest` runs with, as `(granted, refused)`:
/// the stored `plugin_grants` rows for this scope (`Store::plugin_grants_scoped`:
/// deployment-wide rows, plus `account`'s own when the instance backs an account)
/// ∩ the manifest's declared capabilities, then through [`provenance_filtered_grant`]
/// (`refused` is what that removed). With no grant row the result is empty. A row
/// naming a capability the manifest does not declare contributes nothing, and a
/// store error reads as no grant.
pub(crate) async fn effective_capabilities(
    store: &Store,
    manifest: &PluginManifest,
    account: Option<&str>,
) -> (Vec<Capability>, Vec<Capability>) {
    let stored = store
        .plugin_grants_scoped(&manifest.id, account)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!("plugin_grants read failed for '{}': {e}", manifest.id);
            Vec::new()
        });
    let granted: Vec<Capability> = manifest
        .capabilities
        .iter()
        .copied()
        .filter(|c| stored.contains(&capability_name(*c)))
        .collect();
    provenance_filtered_grant(&manifest.id, &granted)
}

/// Why a registered plugin is not running in this process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NotLoaded {
    /// No administrator has approved it.
    NotApproved,
    /// It is approved but not enabled.
    Disabled,
    /// It carries no signature and its [`TrustPolicy`] does not admit that.
    UnsignedNotAllowed,
    /// No stored grant gives it a capability it can run with.
    NoGrant,
    /// No component file passed the digest gate ([`resolve_component`]).
    ComponentUnavailable,
    /// `PluginHost::load` refused the component.
    LoadFailed,
    /// This server runs in proxy mode: it has no engine, so nothing calls a plugin.
    NoEngine,
    /// It is an account backend and no account is bound to it (`bridge_accounts`).
    NoAccountBinding,
    /// Nothing in this server calls the hooks it implements.
    NoHostCaller,
    /// Another spam classifier holds the one classifier seat.
    Superseded,
}

impl NotLoaded {
    /// The name of this reason in the admin API.
    pub(crate) fn wire(self) -> &'static str {
        match self {
            Self::NotApproved => "not-approved",
            Self::Disabled => "disabled",
            Self::UnsignedNotAllowed => "unsigned-not-allowed",
            Self::NoGrant => "no-grant",
            Self::ComponentUnavailable => "component-unavailable",
            Self::LoadFailed => "load-failed",
            Self::NoEngine => "proxy-mode",
            Self::NoAccountBinding => "no-account-binding",
            Self::NoHostCaller => "no-host-caller",
            Self::Superseded => "another-classifier-active",
        }
    }
}

/// What a plugin is loaded as. Decided from the manifest's declared capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PluginRole {
    /// Declares `account-backend`: loaded per `bridge_accounts` binding by
    /// [`load_plugin_backends`], at start-up; stopped by [`unload_unwanted_bridges`]
    /// after any registry change that no longer permits the loaded instance.
    Bridge,
    /// Declares `spam-action`: loaded into the classifier seat by
    /// [`sync_spam_classifier`], at start-up and after every registry change.
    Spam,
    /// Anything else (`dlp-detector`, `message-pipeline`, `addrbook-source`,
    /// `autoconfig-source` on their own). The server has no caller for these hooks,
    /// so such a plugin is never loaded.
    Other,
}

impl PluginRole {
    pub(crate) fn of(manifest: &PluginManifest) -> Self {
        if manifest.capabilities.contains(&Capability::AccountBackend) {
            Self::Bridge
        } else if manifest.capabilities.contains(&Capability::SpamAction) {
            Self::Spam
        } else {
            Self::Other
        }
    }
}

/// A load that the registry state permits: the manifest and the grant to hand to
/// `PluginHost::load`.
pub(crate) struct LoadPlan {
    manifest: PluginManifest,
    grant: Grant,
    /// HIGH_POWER capabilities a stored grant named and provenance removed.
    refused: Vec<Capability>,
}

/// Decide whether `row` may load for `account` (`None` ⇒ an instance bound to no
/// account) and with what. Reads only the store; no component file is opened.
pub(crate) async fn plan_load(
    store: &Store,
    row: &PluginRow,
    account: Option<&str>,
) -> Result<LoadPlan, NotLoaded> {
    if row.approved_by.is_none() {
        return Err(NotLoaded::NotApproved);
    }
    if !row.enabled {
        return Err(NotLoaded::Disabled);
    }
    let manifest = manifest_of(row);
    let trust = TrustPolicy::of(store, &row.id).await;
    if manifest.signature.is_none() && !trust.admits_unsigned() {
        return Err(NotLoaded::UnsignedNotAllowed);
    }
    let (capabilities, refused) = effective_capabilities(store, &manifest, account).await;
    if capabilities.is_empty() {
        return Err(NotLoaded::NoGrant);
    }
    let grant = Grant {
        plugin_id: row.id.clone(),
        capabilities,
        granted_by: row.approved_by.clone().unwrap_or_default(),
        allow_unsigned: trust.admits_unsigned(),
    };
    Ok(LoadPlan {
        manifest,
        grant,
        refused,
    })
}

/// Carry out a [`LoadPlan`]: pass the component through the digest gate and load it
/// under the plan's grant, bound to `account` when given.
async fn load_planned(
    host: &PluginRegistry,
    store: &Store,
    plan: &LoadPlan,
    account: Option<&str>,
) -> Result<PluginHandle, NotLoaded> {
    let id = &plan.manifest.id;
    if !plan.refused.is_empty() {
        tracing::warn!(
            "third-party plugin '{id}' refused HIGH_POWER capability(ies) {:?} (first-party only)",
            plan.refused
        );
        audit_plugin_event(
            store,
            mw_admin::AuditKind::PluginLoadRefused,
            id,
            json!({ "reason": "high-power-cap-refused", "caps": format!("{:?}", plan.refused) }),
        )
        .await;
    }
    // Deny-by-default code load: first-party bytes must match the compiled-in pin,
    // any other bytes an active admin-approved pin. A missing, tampered, unapproved
    // or revoked component fails closed (and audits).
    let Some(bytes) = resolve_component(id, store).await else {
        return Err(NotLoaded::ComponentUnavailable);
    };
    let host = host.lock().expect("plugin registry lock");
    let loaded = match account {
        // Bind the account to the instance so the guest's per-account host imports
        // (`basic-credentials`/`oauth-token`), which pass an empty handle, resolve to
        // this account's sealed credentials host-side.
        Some(account) => host.load_for_account(&bytes, &plan.manifest, &plan.grant, account),
        None => host.load(&bytes, &plan.manifest, &plan.grant),
    };
    loaded.map_err(|e| {
        tracing::error!("plugin '{id}' load failed: {e}");
        NotLoaded::LoadFailed
    })
}

// ── What is loaded in this process ─────────────────────────────────────────────────────

/// One plugin as loaded: per instance, the capabilities it runs with.
#[derive(Debug, Clone, PartialEq, Eq)]
struct LoadedPlugin {
    role: PluginRole,
    /// Account id (`""` for an instance bound to no account) → that instance's
    /// effective capabilities, as `PluginHandle::granted` reports them.
    instances: BTreeMap<String, Vec<Capability>>,
    /// The registry row's `net_allowlist` when it was loaded.
    net_allowlist: Vec<String>,
}

/// The engine's spam classifier seat. [`build_spam_hook`] hands this to the engine
/// once; [`sync_spam_classifier`] fills, replaces and empties it afterwards, so a
/// registry change reaches ingest without a restart. Empty, it answers `Unknown`,
/// on which ingest does nothing.
pub(crate) struct SpamSeat {
    current: std::sync::RwLock<Option<Arc<SpamPluginHook>>>,
}

impl SpamSeat {
    fn get(&self) -> Option<Arc<SpamPluginHook>> {
        self.current.read().expect("spam seat lock").clone()
    }

    fn set(&self, hook: Option<Arc<SpamPluginHook>>) {
        *self.current.write().expect("spam seat lock") = hook;
    }
}

#[async_trait]
impl mw_engine::SpamHook for SpamSeat {
    async fn classify(&self, raw: &[u8]) -> mw_engine::SpamVerdict {
        match self.get() {
            Some(hook) => hook.classify(raw).await,
            None => mw_engine::SpamVerdict::Unknown,
        }
    }
}

/// What the plugin admin routes need to know about one plugin host: what is loaded
/// in it and the classifier seat the engine holds.
pub(crate) struct PluginRuntime {
    spam: Arc<SpamSeat>,
    /// Set by [`build_spam_hook`] and [`load_plugin_backends`], which the mount
    /// calls in engine mode only. Unset ⇒ proxy mode: nothing would call a plugin,
    /// so none is loaded.
    engine_mode: AtomicBool,
    loaded: Mutex<BTreeMap<String, LoadedPlugin>>,
    /// The reason the last load attempt of a plugin failed after planning.
    failed: Mutex<BTreeMap<String, NotLoaded>>,
    /// The PIM source [`load_plugin_backends`] handed to the engine, if it bound any.
    bridge_pim: Mutex<Option<Arc<BridgePimSource>>>,
}

impl PluginRuntime {
    fn loaded(&self, plugin_id: &str) -> Option<LoadedPlugin> {
        self.loaded
            .lock()
            .expect("plugin runtime lock")
            .get(plugin_id)
            .cloned()
    }

    fn failure(&self, plugin_id: &str) -> Option<NotLoaded> {
        self.failed
            .lock()
            .expect("plugin runtime lock")
            .get(plugin_id)
            .copied()
    }

    fn note_failure(&self, plugin_id: &str, why: NotLoaded) {
        self.failed
            .lock()
            .expect("plugin runtime lock")
            .insert(plugin_id.to_string(), why);
    }

    /// Record a loaded instance of `row` (bound to `account`, `""` for none).
    fn note_loaded(&self, row: &PluginRow, account: &str, handle: &PluginHandle) {
        let mut loaded = self.loaded.lock().expect("plugin runtime lock");
        let entry = loaded.entry(row.id.clone()).or_insert_with(|| {
            let manifest = manifest_of(row);
            LoadedPlugin {
                role: PluginRole::of(&manifest),
                instances: BTreeMap::new(),
                net_allowlist: manifest.net_allowlist,
            }
        });
        entry
            .instances
            .insert(account.to_string(), handle.granted());
        self.failed
            .lock()
            .expect("plugin runtime lock")
            .remove(&row.id);
    }

    fn forget(&self, plugin_id: &str) {
        self.loaded
            .lock()
            .expect("plugin runtime lock")
            .remove(plugin_id);
    }

    /// Drop the record of one instance; the plugin's record goes with its last one.
    fn forget_instance(&self, plugin_id: &str, account: &str) {
        let mut loaded = self.loaded.lock().expect("plugin runtime lock");
        if let Some(entry) = loaded.get_mut(plugin_id) {
            entry.instances.remove(account);
            if entry.instances.is_empty() {
                loaded.remove(plugin_id);
            }
        }
    }
}

/// The [`PluginRuntime`] of every plugin host that is still alive, keyed by the
/// host. A side table for the reason [`ASSIST_LIVE`] is one: the handlers receive
/// only the `PluginRegistry` extension, and two apps in one process (the integration
/// tests) must not share state.
static PLUGIN_RUNTIMES: Mutex<Vec<RuntimeEntry>> = Mutex::new(Vec::new());

/// One [`PLUGIN_RUNTIMES`] entry: a plugin host and its runtime record.
type RuntimeEntry = (Weak<Mutex<PluginHost>>, Arc<PluginRuntime>);

/// The runtime record for `reg`, created on first use.
pub(crate) fn plugin_runtime(reg: &PluginRegistry) -> Arc<PluginRuntime> {
    let mut table = PLUGIN_RUNTIMES.lock().expect("plugin runtime table lock");
    table.retain(|(host, _)| host.strong_count() > 0);
    if let Some((_, runtime)) = table
        .iter()
        .find(|(host, _)| std::ptr::eq(host.as_ptr(), Arc::as_ptr(reg)))
    {
        return Arc::clone(runtime);
    }
    let runtime = Arc::new(PluginRuntime {
        spam: Arc::new(SpamSeat {
            current: std::sync::RwLock::new(None),
        }),
        engine_mode: AtomicBool::new(false),
        loaded: Mutex::new(BTreeMap::new()),
        failed: Mutex::new(BTreeMap::new()),
        bridge_pim: Mutex::new(None),
    });
    table.push((Arc::downgrade(reg), Arc::clone(&runtime)));
    runtime
}

/// [`plan_load`] for a spam classifier: the grant must include `spam-action`.
async fn plan_spam_load(store: &Store, row: &PluginRow) -> Result<LoadPlan, NotLoaded> {
    let plan = plan_load(store, row, None).await?;
    if plan.grant.capabilities.contains(&Capability::SpamAction) {
        Ok(plan)
    } else {
        Err(NotLoaded::NoGrant)
    }
}

/// Make the classifier seat match the registry: the first plugin, in id order, whose
/// role is [`PluginRole::Spam`] and whose [`plan_spam_load`] succeeds and loads, is
/// seated; with none, the seat is emptied. A seated plugin whose plan is unchanged is
/// kept without being loaded again. The admin routes call this after every change,
/// so approve, enable, disable, grant, allow-unsigned, uninstall and a digest
/// revocation all take effect on the next message. Does nothing in proxy mode.
pub(crate) async fn sync_spam_classifier(reg: &PluginRegistry, store: &Store) {
    let runtime = plugin_runtime(reg);
    if !runtime.engine_mode.load(Ordering::SeqCst) {
        return;
    }
    let rows = match store.list_plugins().await {
        Ok(rows) => rows,
        Err(e) => {
            // The registry cannot be read, so nothing is known to be permitted.
            tracing::error!("plugin registry read failed; spam classifier unloaded: {e}");
            Vec::new()
        }
    };
    let seated = runtime.spam.get();
    let mut chosen: Option<String> = None;
    for row in &rows {
        if PluginRole::of(&manifest_of(row)) != PluginRole::Spam {
            continue;
        }
        runtime
            .failed
            .lock()
            .expect("plugin runtime lock")
            .remove(&row.id);
        if chosen.is_some() {
            continue;
        }
        let Ok(plan) = plan_spam_load(store, row).await else {
            continue;
        };
        let planned: BTreeSet<Capability> = plan.grant.capabilities.iter().copied().collect();
        let unchanged = seated.as_ref().is_some_and(|s| s.plugin_id == row.id)
            && runtime.loaded(&row.id).is_some_and(|l| {
                l.net_allowlist == plan.manifest.net_allowlist
                    && l.instances.get("").is_some_and(|caps| {
                        caps.iter().copied().collect::<BTreeSet<_>>() == planned
                    })
            });
        if unchanged {
            chosen = Some(row.id.clone());
            continue;
        }
        match load_planned(reg, store, &plan, None).await {
            Ok(handle) => {
                runtime.forget(&row.id);
                runtime.note_loaded(row, "", &handle);
                runtime.spam.set(Some(Arc::new(SpamPluginHook {
                    handle,
                    plugin_id: row.id.clone(),
                })));
                tracing::info!(
                    "spam classifier plugin '{}' loaded (§10.8 delivery-filter hook)",
                    row.id
                );
                chosen = Some(row.id.clone());
            }
            Err(why) => runtime.note_failure(&row.id, why),
        }
    }
    if chosen.is_none() && seated.is_some() {
        runtime.spam.set(None);
        tracing::info!("spam classifier unloaded: no approved, enabled and granted plugin");
    }
    // Drop the record of any classifier that is no longer the seated one.
    let spam_ids: Vec<&str> = rows
        .iter()
        .filter(|r| PluginRole::of(&manifest_of(r)) == PluginRole::Spam)
        .map(|r| r.id.as_str())
        .collect();
    if let Some(old) = seated
        && chosen.as_deref() != Some(old.plugin_id.as_str())
    {
        runtime.forget(&old.plugin_id);
    }
    for id in spam_ids {
        if chosen.as_deref() != Some(id) {
            runtime.forget(id);
        }
    }
}

/// Run the seated spam classifier on `raw` if it is `plugin_id`, and return the
/// guest's verdict envelope (or the host's error). `None` when `plugin_id` is not the
/// seated classifier. This is the call ingest makes, without the ingest around it.
pub(crate) async fn probe_spam_classifier(
    reg: &PluginRegistry,
    plugin_id: &str,
    raw: Vec<u8>,
) -> Option<Result<String, String>> {
    let hook = plugin_runtime(reg).spam.get()?;
    if hook.plugin_id != plugin_id {
        return None;
    }
    Some(
        hook.handle
            .call_spam_action(raw)
            .await
            .map_err(|e| e.to_string()),
    )
}

/// The bridge instances the registry state asks for: per bound account, the
/// capabilities its instance would run with. An account whose plan fails, or whose
/// grant lacks `account-backend`, is left out; the first such reason is returned too.
async fn wanted_bridge_instances(
    store: &Store,
    row: &PluginRow,
) -> (BTreeMap<String, Vec<Capability>>, usize, Option<NotLoaded>) {
    let bindings = store.list_bridge_accounts().await.unwrap_or_default();
    let mut wanted = BTreeMap::new();
    let mut bound = 0usize;
    let mut refusal = None;
    for b in bindings.iter().filter(|b| b.bridge_id == row.id) {
        bound += 1;
        match plan_load(store, row, Some(&b.account_id)).await {
            Ok(plan)
                if plan
                    .grant
                    .capabilities
                    .contains(&Capability::AccountBackend) =>
            {
                let caps: BTreeSet<Capability> = plan.grant.capabilities.iter().copied().collect();
                wanted.insert(b.account_id.clone(), caps.into_iter().collect());
            }
            Ok(_) => refusal = refusal.or(Some(NotLoaded::NoGrant)),
            Err(why) => refusal = refusal.or(Some(why)),
        }
    }
    (wanted, bound, refusal)
}

/// Stop every loaded bridge instance the registry no longer asks for as it is: its
/// plugin was uninstalled, disabled, lost its digest pin's enablement or its
/// allow-unsigned flag, or the instance's grant or host list changed. The account's
/// backend is unregistered from the engine and its PIM slots are dropped, so nothing
/// routes to the instance any more. Nothing is loaded here: an instance the registry
/// asks for that is not running, including one stopped because its grant changed,
/// starts at the next start-up, which [`plugin_status`] reports.
pub(crate) async fn unload_unwanted_bridges(
    engine: Option<&Arc<mw_engine::Engine>>,
    reg: &PluginRegistry,
    store: &Store,
) {
    let Some(engine) = engine else {
        return;
    };
    let runtime = plugin_runtime(reg);
    let bridges: Vec<(String, LoadedPlugin)> = runtime
        .loaded
        .lock()
        .expect("plugin runtime lock")
        .iter()
        .filter(|(_, l)| l.role == PluginRole::Bridge)
        .map(|(id, l)| (id.clone(), l.clone()))
        .collect();
    if bridges.is_empty() {
        return;
    }
    let rows = match store.list_plugins().await {
        Ok(rows) => rows,
        Err(e) => {
            // The registry cannot be read, so nothing is known to be permitted.
            tracing::error!("plugin registry read failed; loaded bridges unloaded: {e}");
            Vec::new()
        }
    };
    let pim = runtime
        .bridge_pim
        .lock()
        .expect("plugin runtime lock")
        .clone();
    for (id, loaded) in bridges {
        let row = rows.iter().find(|r| r.id == id);
        let wanted = match row {
            Some(row) if manifest_of(row).net_allowlist == loaded.net_allowlist => {
                wanted_bridge_instances(store, row).await.0
            }
            _ => BTreeMap::new(),
        };
        for (account, caps) in &loaded.instances {
            if wanted.get(account) == Some(caps) {
                continue;
            }
            engine.unregister(account);
            if let Some(pim) = &pim {
                pim.drop_account(account);
            }
            runtime.forget_instance(&id, account);
            tracing::info!(
                "bridge '{id}' unloaded for account {account}: the registry no longer permits it as loaded"
            );
        }
    }
}

/// Apply the registry to what is running, as far as it can be applied without a
/// restart: [`sync_spam_classifier`] and [`unload_unwanted_bridges`]. Every admin
/// route that changes the registry calls this before it answers.
pub(crate) async fn sync_plugins(
    engine: Option<&Arc<mw_engine::Engine>>,
    reg: &PluginRegistry,
    store: &Store,
) {
    sync_spam_classifier(reg, store).await;
    unload_unwanted_bridges(engine, reg, store).await;
}

/// What the admin API reports about one registered plugin in this process.
pub(crate) struct PluginStatus {
    /// An instance of it is loaded and something in this server calls it.
    pub(crate) loaded: bool,
    /// The capabilities the loaded instance(s) run with.
    pub(crate) loaded_capabilities: Vec<Capability>,
    /// The registry state differs from what is loaded and only a restart applies it.
    pub(crate) restart_required: bool,
    /// Why it is not loaded, when a restart alone would not load it.
    pub(crate) not_loaded: Option<NotLoaded>,
}

/// [`PluginStatus`] for `row`.
pub(crate) async fn plugin_status(
    store: &Store,
    reg: &PluginRegistry,
    row: &PluginRow,
) -> PluginStatus {
    let runtime = plugin_runtime(reg);
    let manifest = manifest_of(row);
    let loaded = runtime.loaded(&row.id);
    let loaded_capabilities: Vec<Capability> = loaded
        .iter()
        .flat_map(|l| l.instances.values().flatten().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let (restart_required, not_loaded) = match PluginRole::of(&manifest) {
        PluginRole::Other => (false, Some(NotLoaded::NoHostCaller)),
        _ if !runtime.engine_mode.load(Ordering::SeqCst) => (false, Some(NotLoaded::NoEngine)),
        // `sync_spam_classifier` applies every change, so there is nothing a restart
        // would add.
        PluginRole::Spam if loaded.is_some() => (false, None),
        PluginRole::Spam => {
            let why = match plan_spam_load(store, row).await {
                Err(why) => why,
                Ok(_) => runtime.failure(&row.id).unwrap_or(NotLoaded::Superseded),
            };
            (false, Some(why))
        }
        // Bridges are loaded by `load_plugin_backends` at start-up only, and stopped
        // by `unload_unwanted_bridges` at any time.
        PluginRole::Bridge => {
            let (wanted, bound, refusal) = wanted_bridge_instances(store, row).await;
            let failure = runtime.failure(&row.id);
            let differs = match &loaded {
                Some(l) => l.instances != wanted || l.net_allowlist != manifest.net_allowlist,
                None => !wanted.is_empty(),
            };
            let why = if loaded.is_some() {
                None
            } else if bound == 0 {
                // What stands in the way besides the missing binding comes first.
                Some(
                    plan_load(store, row, None)
                        .await
                        .err()
                        .unwrap_or(NotLoaded::NoAccountBinding),
                )
            } else {
                refusal.or(failure)
            };
            (differs && failure.is_none(), why)
        }
    };
    PluginStatus {
        loaded: loaded.is_some(),
        loaded_capabilities,
        restart_required,
        not_loaded,
    }
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
        // The envelope check `mw_smtp::Submitter` runs before it connects. This path
        // never reaches that submitter, so it is run here; a refusal is the error the
        // SMTP path yields through this seam, and the bridge is not called.
        msg.validate()
            .map_err(|e| mw_engine::backend::EngineError::Protocol(e.to_string()))?;
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

#[cfg(test)]
mod bridge_submitter_tests {
    use super::BridgeSubmitter;
    use async_trait::async_trait;
    use mw_engine::account::MailSubmitter;
    use mw_engine::backend::{
        AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MessageRef,
        MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor, WatchHandle,
    };
    use std::sync::{Arc, Mutex};

    /// A bridge backend that records every message handed to its `append` (the
    /// bridge `submit` export); nothing else is called by the submitter.
    #[derive(Default)]
    struct RecordingBridge {
        sent: Mutex<Vec<Vec<u8>>>,
    }

    #[async_trait]
    impl AccountBackend for RecordingBridge {
        async fn capabilities(&self) -> Result<BackendCaps> {
            Ok(BackendCaps::default())
        }
        async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
            Ok(Vec::new())
        }
        async fn sync_mailbox(&self, _: &RawMailboxRef, _: &SyncCursor) -> Result<MailboxDelta> {
            Err(EngineError::Unsupported("recording bridge".into()))
        }
        async fn fetch_raw(&self, _: &[MessageRef]) -> Result<Vec<RawMessage>> {
            Ok(Vec::new())
        }
        async fn store_flags(&self, _: &[MessageRef], _: &[Flag], _: &[Flag]) -> Result<()> {
            Ok(())
        }
        async fn move_messages(&self, _: &[MessageRef], _: &RawMailboxRef) -> Result<MoveOutcome> {
            Err(EngineError::Unsupported("recording bridge".into()))
        }
        async fn append(&self, _: &RawMailboxRef, raw: &[u8], _: &[Flag]) -> Result<MessageRef> {
            self.sent.lock().unwrap().push(raw.to_vec());
            Ok(MessageRef::Pop3 {
                uidl: "sent".into(),
            })
        }
        async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> {
            Err(EngineError::Unsupported("recording bridge".into()))
        }
    }

    fn submitter() -> (Arc<RecordingBridge>, BridgeSubmitter) {
        let bridge = Arc::new(RecordingBridge::default());
        let submitter = BridgeSubmitter {
            backend: bridge.clone(),
            bridge_id: "bridge-test".into(),
        };
        (bridge, submitter)
    }

    fn outgoing(mail_from: &str, rcpt_to: &[&str]) -> mw_smtp::Outgoing {
        mw_smtp::Outgoing {
            mail_from: mail_from.into(),
            rcpt_to: rcpt_to.iter().map(|r| r.to_string()).collect(),
            raw: b"Subject: hi\r\n\r\nbody\r\n".to_vec(),
        }
    }

    #[tokio::test]
    async fn a_valid_envelope_reaches_the_bridge() {
        let (bridge, submitter) = submitter();
        let out = submitter
            .submit(outgoing("alice@example.test", &["bob@example.test"]))
            .await
            .unwrap();
        assert_eq!(out.accepted, vec!["bob@example.test".to_string()]);
        assert_eq!(bridge.sent.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn a_hostile_envelope_address_is_refused_before_the_bridge() {
        let cases: [(&str, &[&str]); 5] = [
            (
                "alice@example.test",
                &["bob@example.test\r\nRCPT TO:<victim@example.test>"],
            ),
            (
                "alice@example.test",
                &[
                    "bob@example.test",
                    "x@example.test\nBcc: victim@example.test",
                ],
            ),
            ("alice@example.test", &["no-at-sign"]),
            ("alice@example.test", &[""]),
            (
                "alice@example.test>\r\nRCPT TO:<victim@example.test",
                &["bob@example.test"],
            ),
        ];
        for (from, to) in cases {
            let (bridge, submitter) = submitter();
            // What the SMTP path yields through the engine seam for the same message.
            let want = match outgoing(from, to).validate() {
                Err(e @ mw_smtp::SmtpError::InvalidAddress(_)) => e.to_string(),
                other => panic!("{from:?} {to:?}: mw-smtp did not refuse it: {other:?}"),
            };
            match submitter.submit(outgoing(from, to)).await {
                Err(EngineError::Protocol(m)) => assert_eq!(m, want),
                other => panic!("{from:?} {to:?}: not refused: {other:?}"),
            }
            assert!(
                bridge.sent.lock().unwrap().is_empty(),
                "{from:?} {to:?}: the message reached the bridge"
            );
        }
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
/// for every accessor ⇒ the engine's byte-unchanged standards fallback. An account is
/// removed when its bridge instance is unloaded ([`unload_unwanted_bridges`]).
pub(crate) struct BridgePimSource {
    accounts: std::sync::RwLock<std::collections::HashMap<String, BridgePimSlots>>,
}

impl BridgePimSource {
    fn slots(&self, account_id: &str) -> Option<BridgePimSlots> {
        self.accounts
            .read()
            .expect("bridge pim lock")
            .get(account_id)
            .cloned()
    }

    fn drop_account(&self, account_id: &str) {
        self.accounts
            .write()
            .expect("bridge pim lock")
            .remove(account_id);
    }
}

impl mw_engine::BridgeCapabilitySource for BridgePimSource {
    fn caps(&self, account_id: &str) -> mw_engine::BridgeCaps {
        self.slots(account_id).map(|s| s.caps).unwrap_or_default()
    }
    fn reactions(&self, account_id: &str) -> Option<Arc<dyn mw_engine::BridgeReactions>> {
        self.slots(account_id).and_then(|s| s.reactions)
    }
    fn voting(&self, account_id: &str) -> Option<Arc<dyn mw_engine::BridgeVoting>> {
        self.slots(account_id).and_then(|s| s.voting)
    }
    fn recall(&self, account_id: &str) -> Option<Arc<dyn mw_engine::BridgeRecall>> {
        self.slots(account_id).and_then(|s| s.recall)
    }
    fn focused_sync(&self, account_id: &str) -> Option<Arc<dyn mw_engine::BridgeFocusedSync>> {
        self.slots(account_id).and_then(|s| s.focused)
    }
    fn calendar(&self, account_id: &str) -> Option<Arc<dyn mw_engine::BridgeCalendar>> {
        self.slots(account_id).and_then(|s| s.calendar)
    }
    fn tasks(&self, account_id: &str) -> Option<Arc<dyn mw_engine::BridgeTasks>> {
        self.slots(account_id).and_then(|s| s.tasks)
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

/// The engine's spam-classification hook: the classifier seat of this plugin host,
/// filled from the registry by [`sync_spam_classifier`] (the first approved, enabled
/// plugin, in id order, with a stored `spam-action` grant and a component that passes
/// the digest gate). Always `Some`: the seat is handed over even when empty so that a
/// plugin registered later is called without a restart. An empty seat answers
/// `Unknown`, which `apply_spam_at_ingest` treats like `Ham`.
pub(crate) async fn build_spam_hook(
    host: &PluginRegistry,
    store: &Store,
) -> Option<Arc<dyn mw_engine::SpamHook>> {
    let runtime = plugin_runtime(host);
    runtime.engine_mode.store(true, Ordering::SeqCst);
    sync_spam_classifier(host, store).await;
    Some(Arc::clone(&runtime.spam) as Arc<dyn mw_engine::SpamHook>)
}

/// Boot-load the approved bridge/plugin account backends from the 0008 registry
/// (plan §6.5). For every `bridge_accounts` binding whose bound plugin passes
/// [`plan_load`] for that account (approved, enabled, trust policy satisfied, at
/// least one stored grant), this passes the component through the digest gate and
/// loads it under the plan's grant ([`load_planned`]; host services already injected
/// via [`build_plugin_host`]), takes `as_account_backend()` — present only when the
/// grant includes `account-backend` — and registers it on the engine via
/// `register_plugin_backend`, after which the account is served by the SAME sync/JMAP
/// dispatch as an IMAP account. Additionally probes each loaded bridge's HONEST per-interface PIM support
/// and, when advertised, binds its calendar/tasks/reactions/voting/recall/focused-sync
/// trait objects into a per-account [`BridgePimSource`] (returned for e13 to attach via
/// [`mw_engine::V7Hooks::with_bridge_caps`]). Returns `(loaded_count, bridge_pim_source)`
/// — the source is `None` when no account bound any PIM interface.
///
/// Deny-by-default: an unbound, unapproved, disabled, ungranted or unpinned plugin
/// loads nothing, a component whose on-disk bytes fail the digest pin fails closed, and
/// an account with no binding is byte-unchanged from the non-plugin path. Every skip is
/// logged (never silent). Runs once, at start-up. A later registry change stops an
/// instance it no longer permits ([`unload_unwanted_bridges`]); one that would start
/// an instance is reported by [`plugin_status`] as needing a restart.
pub async fn load_plugin_backends(
    engine: &Arc<mw_engine::Engine>,
    host: &PluginRegistry,
    store: &Store,
) -> (usize, Option<Arc<dyn mw_engine::BridgeCapabilitySource>>) {
    let runtime = plugin_runtime(host);
    runtime.engine_mode.store(true, Ordering::SeqCst);
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
        let mut plan = match plan_load(store, row, Some(&b.account_id)).await {
            Ok(plan) => plan,
            Err(why) => {
                tracing::warn!(
                    "bridge plugin '{}' not loaded for account {}: {}",
                    b.bridge_id,
                    b.account_id,
                    why.wire()
                );
                continue;
            }
        };
        // EWS (password-auth bridge): the account's real Exchange host is provisioned
        // per-account in the sealed 0011 `ews_account_cred` row, not in the registry
        // row's allowlist. Mirror its `endpoint_host` into this instance's manifest
        // `net_allowlist` so the jailed guest's host-mediated `http-fetch` to the
        // account's endpoint is admitted through the gate (deny-by-default holds for
        // every other host). Absent/disabled row ⇒ no rewrite (the guest then has no
        // reachable endpoint and fails auth via the credential provider above).
        if let Ok(Some(cred)) = store.get_ews_account_cred(&b.account_id).await
            && !cred.endpoint_host.is_empty()
            && !plan
                .manifest
                .net_allowlist
                .iter()
                .any(|h| h.eq_ignore_ascii_case(&cred.endpoint_host))
        {
            plan.manifest.net_allowlist.push(cred.endpoint_host);
        }
        // The grant is `plan_load`'s: stored grants ∩ manifest, with every HIGH_POWER
        // capability removed for a third-party plugin (the user's 26.15 decision), so
        // such a plugin fails `as_account_backend()` below and is not registered.
        let handle = match load_planned(host, store, &plan, Some(&b.account_id)).await {
            Ok(handle) => handle,
            Err(why) => {
                tracing::warn!(
                    "bridge plugin '{}' not loaded for account {}: {}",
                    b.bridge_id,
                    b.account_id,
                    why.wire()
                );
                runtime.note_failure(&row.id, why);
                continue;
            }
        };
        let Some(backend) = handle.as_account_backend() else {
            tracing::warn!(
                "plugin '{}' holds no account-backend grant; account {} not loaded",
                b.bridge_id,
                b.account_id
            );
            runtime.note_failure(&row.id, NotLoaded::NoGrant);
            continue;
        };

        // The account's own identity (From/MAIL FROM); fall back to the account id.
        let identity = store
            .get_account(&b.account_id)
            .await
            .map(|a| a.username)
            .unwrap_or_else(|_| b.account_id.clone());
        let account_runtime = mw_engine::account::AccountRuntime::new(
            backend.clone(),
            Arc::new(BridgeSubmitter {
                backend,
                bridge_id: b.bridge_id.clone(),
            }) as Arc<dyn mw_engine::account::MailSubmitter>,
            identity,
        );
        engine.register_plugin_backend(b.account_id.clone(), b.bridge_id.clone(), account_runtime);
        runtime.note_loaded(row, &b.account_id, &handle);
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
        let source = Arc::new(BridgePimSource {
            accounts: std::sync::RwLock::new(pim_slots),
        });
        // Kept so that unloading an instance also takes its PIM slots away.
        *runtime.bridge_pim.lock().expect("plugin runtime lock") = Some(Arc::clone(&source));
        Some(source)
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
        // `.no_proxy()`: this client sends the Nextcloud app password on every OCS
        // call. See `mw_egress::harden_client`.
        reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("reqwest client builds"),
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
/// (GET/PUT + status + kill), `GET /api/nextcloud/list`, and the allowlist and egress
/// admin routers.
/// e14 merges this into `router()` alongside the e9 factories and layers the same
/// injected extensions.
pub(crate) fn extra_v7_router() -> Router<AppState> {
    Router::new()
        .route("/api/assist/transcribe", post(assist_transcribe))
        .route("/admin/assist", get(assist_admin_get).put(assist_admin_put))
        .route("/admin/assist/status", get(assist_admin_status))
        .route("/admin/assist/kill", post(assist_admin_kill))
        .route("/api/nextcloud/list", get(nextcloud_list))
        // The third-party allowlist admin API (approve/revoke/list-pending/uninstall),
        // admin-session-gated + audited. Registered on this already-mounted router so no
        // `lib.rs` mount edit is needed this wave.
        .merge(admin_plugins::allowlist_router())
        // The egress-proxy admin API (list/put/delete), admin-session-gated + audited.
        // Registered here for the same reason as the allowlist router above.
        .merge(egress_admin::egress_admin_router())
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

// The admin wire shape of the deployment Assist configuration. `GET /admin/assist`
// returns exactly this object and `PUT /admin/assist` accepts exactly this object;
// `apps/web/src/screens/Admin/Assist/service.ts` is its only client.
//
//   {
//     "enabled": bool,
//     "adapter": null | { "kind": "open-ai-compatible" | "anthropic" | "local-process", … },
//     "capabilityGrants": [ "summarize", … ],
//     "dataCeilings": { "accounts": [], "folders": [], "includeE2ee": bool, "includeAttachments": bool }
//   }
//
// Every key is camelCase. The `adapter` and `dataCeilings` objects are
// `mw_assist::AdapterConfig` and `mw_assist::DataScope` with their keys renamed, so
// a field added to either type appears on the wire without an edit here.

/// `snake_case` → `camelCase`, for one object key.
fn snake_to_camel(key: &str) -> String {
    let mut out = String::with_capacity(key.len());
    let mut upper = false;
    for c in key.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// `camelCase` → `snake_case`, for one object key.
fn camel_to_snake(key: &str) -> String {
    let mut out = String::with_capacity(key.len() + 4);
    for c in key.chars() {
        if c.is_ascii_uppercase() {
            out.push('_');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// A serialised `mw-assist` value with its top-level keys renamed to camelCase.
fn to_wire_object<T: serde::Serialize>(value: &T) -> serde_json::Value {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::Object(map)) => serde_json::Value::Object(
            map.into_iter()
                .map(|(k, v)| (snake_to_camel(&k), v))
                .collect(),
        ),
        Ok(other) => other,
        Err(_) => serde_json::Value::Null,
    }
}

/// Parse one camelCase wire object into the `mw-assist` type it renames.
///
/// Refuses a key that is not camelCase and a key the type does not have — the
/// types themselves ignore unknown keys, which is how a client speaking another
/// shape used to have its whole payload dropped and the row overwritten with
/// defaults. With `require_all`, also refuses an object that leaves a key out.
fn from_wire_object<T>(
    what: &str,
    value: &serde_json::Value,
    require_all: bool,
) -> Result<T, String>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let serde_json::Value::Object(wire) = value else {
        return Err(format!("{what} must be an object"));
    };
    let mut snake = serde_json::Map::new();
    for (key, v) in wire {
        let renamed = camel_to_snake(key);
        if snake_to_camel(&renamed) != *key {
            return Err(format!("{what}.{key}: keys are camelCase"));
        }
        snake.insert(renamed, v.clone());
    }
    let parsed: T = serde_json::from_value(serde_json::Value::Object(snake.clone()))
        .map_err(|e| format!("{what}: {e}"))?;
    let canonical = match serde_json::to_value(&parsed) {
        Ok(serde_json::Value::Object(map)) => map,
        _ => return Err(format!("{what} must be an object")),
    };
    if let Some(unknown) = snake.keys().find(|k| !canonical.contains_key(*k)) {
        return Err(format!("{what}.{}: unknown field", snake_to_camel(unknown)));
    }
    if require_all && let Some(missing) = canonical.keys().find(|k| !snake.contains_key(*k)) {
        return Err(format!("{what}.{}: missing field", snake_to_camel(missing)));
    }
    Ok(parsed)
}

/// An [`AssistConfig`] in the admin wire shape. Every key is always present.
fn assist_admin_wire(config: &AssistConfig) -> serde_json::Value {
    json!({
        "enabled": config.enabled,
        "adapter": config.adapter.as_ref().map(to_wire_object),
        "capabilityGrants": config.capability_grants,
        "dataCeilings": to_wire_object(&config.data_ceiling),
    })
}

/// The body of `PUT /admin/assist`. All four keys are required and no other key is
/// accepted, so a request can neither erase a setting by omitting it nor have a
/// setting ignored by misnaming it.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AssistAdminReq {
    enabled: bool,
    /// `null` (no endpoint) or an adapter object.
    adapter: serde_json::Value,
    capability_grants: Vec<AssistCapability>,
    data_ceilings: serde_json::Value,
}

impl AssistAdminReq {
    fn into_config(self) -> Result<AssistConfig, String> {
        let adapter = if self.adapter.is_null() {
            None
        } else {
            let adapter: AdapterConfig = from_wire_object("adapter", &self.adapter, false)?;
            check_adapter(&adapter)?;
            Some(adapter)
        };
        for (i, cap) in self.capability_grants.iter().enumerate() {
            if self.capability_grants[..i].contains(cap) {
                return Err(format!(
                    "capabilityGrants: {} is listed twice",
                    capability_wire(*cap)
                ));
            }
        }
        Ok(AssistConfig {
            enabled: self.enabled,
            capability_grants: self.capability_grants,
            data_ceiling: from_wire_object("dataCeilings", &self.data_ceilings, true)?,
            adapter,
            rate_limit_per_min: None,
        })
    }
}

/// Refuse an adapter the gateway could be built from but could never reach.
fn check_adapter(adapter: &AdapterConfig) -> Result<(), String> {
    match adapter {
        AdapterConfig::OpenAiCompatible { base_url, .. }
        | AdapterConfig::Anthropic { base_url, .. } => {
            let is_http = base_url.starts_with("http://") || base_url.starts_with("https://");
            if !is_http || host_of(base_url).is_none_or(|h| h.is_empty()) {
                return Err(
                    "adapter.baseUrl must be an http:// or https:// URL with a host".into(),
                );
            }
        }
        AdapterConfig::LocalProcess { program, .. } => {
            if program.trim().is_empty() {
                return Err("adapter.program must not be empty".into());
            }
        }
    }
    Ok(())
}

/// The `assist_config` row for a configuration. The columns hold the `mw-assist`
/// types' own serialisation, which is what [`stored_assist_config`] reads back.
fn assist_row(config: &AssistConfig) -> mw_store::AssistConfigRow {
    let text = |v: serde_json::Result<String>, empty: &str| v.unwrap_or_else(|_| empty.into());
    mw_store::AssistConfigRow {
        scope: "deployment".into(),
        adapters_json: text(serde_json::to_string(&config.adapter), "null"),
        capability_grants_json: text(serde_json::to_string(&config.capability_grants), "[]"),
        data_ceilings_json: text(serde_json::to_string(&config.data_ceiling), "{}"),
        enabled: config.enabled,
    }
}

/// What the running gateway is doing, next to what is stored:
///
/// - `enabled`: the stored setting.
/// - `running`: the gateway in this process answers Assist requests.
/// - `endpointHost`: where it sends them, or `null` when it is not running.
/// - `restartPending`: the stored configuration is not the one in effect. Turning
///   Assist off takes effect at once, so it never leaves this set; every other
///   change is read when the gateway is built, at start-up.
fn assist_status(gateway: &AssistHandle, stored: &AssistConfig) -> serde_json::Value {
    let running = assist_running(gateway);
    let restart_pending = match assist_live(gateway) {
        Some(_) if !stored.enabled => false,
        Some(live) => assist_admin_wire(stored) != live.booted,
        // Not a gateway `build_assist` built, so there is nothing to compare with
        // and no stop flag to have applied the stored setting.
        None => stored.enabled || gateway.is_enabled(),
    };
    json!({
        "enabled": stored.enabled,
        "running": running,
        "endpointHost": if running { gateway.endpoint_host() } else { None },
        "restartPending": restart_pending,
    })
}

/// Persist `config`, apply its on/off setting to the running gateway, audit the
/// change, and answer with [`assist_status`].
async fn store_assist_config(
    state: &AppState,
    gateway: &AssistHandle,
    admin: &str,
    config: &AssistConfig,
) -> Response {
    if let Err(e) = state.store.put_assist_config(&assist_row(config)).await {
        tracing::error!("assist config write failed: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "store error").into_response();
    }
    // Off stops the running gateway now; on lifts that stop. Lifting it does not
    // start a gateway that was built disabled — `assist_status` reports that case
    // as `restartPending`.
    if let Some(live) = assist_live(gateway) {
        live.stopped.store(!config.enabled, Ordering::SeqCst);
    }
    append_assist_audit(state, admin, config).await;
    Json(assist_status(gateway, config)).into_response()
}

/// Append an audit row for an Assist configuration change. The detail names the
/// adapter kind and endpoint host, never the API key. Best-effort, as in
/// `egress_admin::append_egress_audit`, whose choice of audit kind this follows.
async fn append_assist_audit(state: &AppState, admin: &str, config: &AssistConfig) {
    let (kind, host) = match &config.adapter {
        Some(AdapterConfig::OpenAiCompatible { base_url, .. }) => {
            ("open-ai-compatible", host_of(base_url))
        }
        Some(AdapterConfig::Anthropic { base_url, .. }) => ("anthropic", host_of(base_url)),
        Some(AdapterConfig::LocalProcess { .. }) => ("local-process", None),
        None => ("none", None),
    };
    let entry = mw_admin::AuditEvent::new(
        admin,
        mw_admin::ActorKind::Admin,
        mw_admin::AuditKind::SecurityPolicyChanged,
    )
    .target("assist")
    .detail(json!({
        "enabled": config.enabled,
        "adapterKind": kind,
        "endpointHost": host,
        "capabilityGrants": config.capability_grants,
        "includeE2ee": config.data_ceiling.include_e2ee,
        "includeAttachments": config.data_ceiling.include_attachments,
        "accounts": config.data_ceiling.accounts.len(),
        "folders": config.data_ceiling.folders.len(),
    }))
    .into_entry();
    let row = mw_store::AuditRow {
        id: entry.id,
        ts: entry.ts,
        actor: entry.actor,
        actor_kind: "admin".to_string(),
        action: entry.action,
        target: entry.target,
        detail_json: entry.detail_json,
        ip: entry.ip,
    };
    if let Err(e) = state.store.append_audit(&row).await {
        tracing::warn!("assist audit append failed ({}): {e}", row.action);
    }
}

/// Read the stored deployment configuration, or the `500` to answer with.
async fn read_assist_config(state: &AppState) -> Result<AssistConfig, Response> {
    match state.store.get_assist_config("deployment").await {
        Ok(row) => Ok(stored_assist_config(row.as_ref())),
        Err(e) => {
            tracing::error!("assist config read failed: {e}");
            Err((StatusCode::INTERNAL_SERVER_ERROR, "store error").into_response())
        }
    }
}

fn bad_request(message: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": message.into() })),
    )
        .into_response()
}

/// `GET /admin/assist` — the stored deployment Assist configuration in the admin
/// wire shape. With no row stored it is the default configuration, every key
/// present.
async fn assist_admin_get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    match read_assist_config(&state).await {
        Ok(config) => Json(assist_admin_wire(&config)).into_response(),
        Err(resp) => resp,
    }
}

/// `GET /admin/assist/status` — [`assist_status`] for the stored configuration.
async fn assist_admin_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(gateway): Extension<AssistHandle>,
) -> Response {
    if let Err(resp) = require_admin(&state, &headers).await {
        return resp;
    }
    match read_assist_config(&state).await {
        Ok(config) => Json(assist_status(&gateway, &config)).into_response(),
        Err(resp) => resp,
    }
}

/// `PUT /admin/assist` — replace the deployment Assist configuration (0008
/// `assist_config`). The body is the admin wire shape, whole; anything else is a
/// `400` naming the problem and nothing is written. Answers with [`assist_status`].
async fn assist_admin_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(gateway): Extension<AssistHandle>,
    body: Result<Json<AssistAdminReq>, JsonRejection>,
) -> Response {
    let admin = match require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let config = match body {
        Ok(Json(req)) => match req.into_config() {
            Ok(c) => c,
            Err(message) => return bad_request(message),
        },
        Err(rejection) => return bad_request(rejection.body_text()),
    };
    store_assist_config(&state, &gateway, &admin, &config).await
}

/// The body of `POST /admin/assist/kill`: `on: true` turns Assist off.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AssistKillReq {
    on: bool,
}

/// `POST /admin/assist/kill` — the tenant-wide Assist kill switch (§14/§19).
///
/// `{"on": true}` stores `enabled = false` and stops the gateway running in this
/// process: from the next request on, every chat, embedding and transcription is
/// refused before anything is sent, and no audit row is written for it.
/// `{"on": false}` stores `enabled = true` and lifts the stop; a gateway that was
/// built disabled stays off until the server restarts, which the [`assist_status`]
/// answer reports as `restartPending`. The rest of the configuration is kept.
async fn assist_admin_kill(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(gateway): Extension<AssistHandle>,
    body: Result<Json<AssistKillReq>, JsonRejection>,
) -> Response {
    let admin = match require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let on = match body {
        Ok(Json(req)) => req.on,
        Err(rejection) => return bad_request(rejection.body_text()),
    };
    let mut config = match read_assist_config(&state).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    config.enabled = !on;
    store_assist_config(&state, &gateway, &admin, &config).await
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

    // ── 26.20 (t28-e14): first-party manifests, trust policy, grant computation ──

    /// The uncommented lines of `plugins/<id>/plugin.toml`.
    fn plugin_toml(id: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../plugins")
            .join(id)
            .join("plugin.toml");
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn toml_value<'a>(body: &'a str, key: &str) -> &'a str {
        let at = body
            .find(&format!("{key} = "))
            .unwrap_or_else(|| panic!("no `{key}` in plugin.toml"));
        let rest = &body[at + key.len() + 3..];
        match rest.strip_prefix('[') {
            Some(list) => &list[..list.find(']').expect("closing bracket")],
            None => rest.lines().next().unwrap_or_default(),
        }
    }

    fn toml_list(body: &str, key: &str) -> Vec<String> {
        toml_value(body, key)
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// The compiled-in manifest table says what each `plugin.toml` says. The one
    /// stated difference is `bridge-ews`, whose file lists a fixture host.
    #[test]
    fn first_party_manifests_match_the_plugin_toml_files() {
        assert_eq!(
            FIRST_PARTY_MANIFESTS
                .iter()
                .map(|m| m.id)
                .collect::<Vec<_>>(),
            FIRST_PARTY_DIGESTS
                .iter()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>(),
            "one manifest per pinned component, same order"
        );
        for m in FIRST_PARTY_MANIFESTS {
            let body = plugin_toml(m.id);
            let text = |key: &str| toml_value(&body, key).trim().trim_matches('"').to_string();
            assert_eq!(text("id"), m.id);
            assert_eq!(text("name"), m.name, "{}: name", m.id);
            assert_eq!(text("version"), m.version, "{}: version", m.id);
            assert_eq!(
                toml_list(&body, "capabilities"),
                m.capabilities
                    .iter()
                    .map(|c| capability_name(*c))
                    .collect::<Vec<_>>(),
                "{}: capabilities",
                m.id
            );
            let hosts = toml_list(&body, "net_allowlist");
            if m.id == "bridge-ews" {
                assert_eq!(hosts, vec!["ews.example.com".to_string()]);
                assert!(m.net_allowlist.is_empty());
            } else {
                assert_eq!(hosts, m.net_allowlist, "{}: net_allowlist", m.id);
            }
            assert_eq!(
                text("memory_mb"),
                m.memory_mb.to_string(),
                "{}: memory_mb",
                m.id
            );
            assert_eq!(
                text("deadline_ms"),
                m.deadline_ms.to_string(),
                "{}: deadline_ms",
                m.id
            );
        }
        assert!(first_party_manifest("nextcloud-plugin").is_none());
        assert!(first_party_manifest("acme").is_none());
    }

    /// A first-party component file whose bytes do not hash to the compiled-in digest
    /// is not returned, wherever it is found; the genuine file next to it is.
    #[test]
    fn a_tampered_first_party_component_is_refused_by_the_digest() {
        let good = first_party_component("spam-rspamd").expect("shipped bytes");
        let tampered_dir = temp_dir();
        let mut tampered = good.clone();
        let last = tampered.len() - 1;
        tampered[last] ^= 0x01;
        std::fs::write(tampered_dir.join("spam-rspamd.wasm"), &tampered).unwrap();
        assert!(
            first_party_component_in("spam-rspamd", std::slice::from_ref(&tampered_dir)).is_none(),
            "a one-bit change must fail the pin"
        );

        let good_dir = temp_dir();
        std::fs::write(good_dir.join("spam-rspamd.wasm"), &good).unwrap();
        assert_eq!(
            first_party_component_in("spam-rspamd", &[tampered_dir.clone(), good_dir.clone()]),
            Some(good),
            "the tampered file is skipped, not loaded, and the verified one is used"
        );
        let _ = std::fs::remove_dir_all(&tampered_dir);
        let _ = std::fs::remove_dir_all(&good_dir);
    }

    fn registry_row(manifest: &PluginManifest) -> PluginRow {
        PluginRow {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            signature_hex: manifest.signature.clone(),
            approved_by: Some("admin".into()),
            enabled: true,
            capabilities_json: serde_json::to_string(&manifest.capabilities).unwrap(),
            net_allowlist_json: serde_json::to_string(&manifest.net_allowlist).unwrap(),
            limits_json: serde_json::to_string(&manifest.limits).unwrap(),
            created_at: "2026-10-05T00:00:00Z".into(),
        }
    }

    async fn grant_caps(store: &Store, id: &str, account: &str, caps: &[Capability]) {
        let names: Vec<String> = caps.iter().map(|c| capability_name(*c)).collect();
        store
            .replace_plugin_grants(id, account, &names, "admin")
            .await
            .unwrap();
    }

    /// The grant a load runs with is the stored rows ∩ the manifest ∩ provenance.
    /// Before 26.20 the loader passed the manifest's whole capability list, so the
    /// first assertion — no row, no capability — is the one that failed.
    #[tokio::test]
    async fn the_effective_grant_is_the_stored_grant_within_the_manifest() {
        use Capability::{AccountBackend, Net, SpamAction, StoreKvScoped};
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let rspamd = first_party_manifest("spam-rspamd").unwrap();
        assert_eq!(rspamd.capabilities, vec![SpamAction, Net, StoreKvScoped]);

        let (granted, _) = effective_capabilities(&store, &rspamd, None).await;
        assert!(
            granted.is_empty(),
            "no grant row means no capability, got {granted:?}"
        );
        assert_eq!(
            plan_load(&store, &registry_row(&rspamd), None).await.err(),
            Some(NotLoaded::NoGrant)
        );

        grant_caps(&store, "spam-rspamd", "", &[SpamAction]).await;
        let (granted, _) = effective_capabilities(&store, &rspamd, None).await;
        assert_eq!(granted, vec![SpamAction], "net was not granted");

        // A row for a capability the manifest does not declare adds nothing.
        grant_caps(&store, "spam-rspamd", "", &[SpamAction, AccountBackend]).await;
        let (granted, _) = effective_capabilities(&store, &rspamd, None).await;
        assert_eq!(granted, vec![SpamAction]);

        // A grant scoped to one account reaches that account's instance only.
        grant_caps(&store, "spam-rspamd", "acct-a", &[Net]).await;
        let (granted, _) = effective_capabilities(&store, &rspamd, None).await;
        assert_eq!(granted, vec![SpamAction]);
        let (granted, _) = effective_capabilities(&store, &rspamd, Some("acct-a")).await;
        assert_eq!(granted, vec![SpamAction, Net]);
        let (granted, _) = effective_capabilities(&store, &rspamd, Some("acct-b")).await;
        assert_eq!(granted, vec![SpamAction]);

        // Provenance: a stored HIGH_POWER grant never reaches a third-party plugin.
        let third = PluginManifest {
            id: "acme".into(),
            name: "Acme".into(),
            version: "1".into(),
            signature: None,
            capabilities: vec![AccountBackend, Net],
            net_allowlist: Vec::new(),
            limits: PluginLimits::default(),
        };
        grant_caps(&store, "acme", "", &[AccountBackend, Net]).await;
        let (granted, refused) = effective_capabilities(&store, &third, None).await;
        assert_eq!(granted, vec![Net]);
        assert_eq!(refused, vec![AccountBackend]);
    }

    /// First-party trust is the digest pin and ignores the stored flag; a third-party
    /// plugin without a signature needs the flag before a load is planned.
    #[tokio::test]
    async fn the_trust_policy_reads_the_stored_flag_for_third_party_ids_only() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        assert_eq!(
            TrustPolicy::of(&store, "spam-rspamd").await,
            TrustPolicy::FirstPartyDigestPin
        );
        assert!(TrustPolicy::FirstPartyDigestPin.admits_unsigned());
        // A flag stored against a first-party id changes nothing.
        store
            .set_plugin_allow_unsigned("spam-rspamd", false)
            .await
            .unwrap();
        assert!(
            TrustPolicy::of(&store, "spam-rspamd")
                .await
                .admits_unsigned()
        );

        let third = PluginManifest {
            id: "acme".into(),
            name: "Acme".into(),
            version: "1".into(),
            signature: None,
            capabilities: vec![Capability::Net],
            net_allowlist: Vec::new(),
            limits: PluginLimits::default(),
        };
        grant_caps(&store, "acme", "", &[Capability::Net]).await;
        let row = registry_row(&third);
        assert_eq!(
            TrustPolicy::of(&store, "acme").await,
            TrustPolicy::AdminPinnedDigest {
                allow_unsigned: false
            }
        );
        assert_eq!(
            plan_load(&store, &row, None).await.err(),
            Some(NotLoaded::UnsignedNotAllowed)
        );
        store.set_plugin_allow_unsigned("acme", true).await.unwrap();
        let plan = plan_load(&store, &row, None).await.expect("planned");
        assert!(plan.grant.allow_unsigned);
        assert_eq!(plan.grant.capabilities, vec![Capability::Net]);

        // Approval and enablement come first.
        let mut unapproved = row.clone();
        unapproved.approved_by = None;
        assert_eq!(
            plan_load(&store, &unapproved, None).await.err(),
            Some(NotLoaded::NotApproved)
        );
        let mut disabled = row.clone();
        disabled.enabled = false;
        assert_eq!(
            plan_load(&store, &disabled, None).await.err(),
            Some(NotLoaded::Disabled)
        );
    }

    /// A loaded bridge is stopped as soon as the registry no longer permits it:
    /// narrowing its grant, then disabling it, each unregister the account's backend
    /// from the engine and drop its PIM slots. Before 26.20 nothing stopped a loaded
    /// instance short of a restart.
    #[tokio::test]
    async fn a_loaded_bridge_is_unloaded_when_the_registry_stops_permitting_it() {
        use Capability::{AccountBackend, AddrbookSource, Net, StoreKvScoped};

        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let graph = first_party_manifest("bridge-graph").unwrap();
        store.put_plugin(&registry_row(&graph)).await.unwrap();
        for account in ["acct-a", "acct-b"] {
            store
                .put_bridge_account(&mw_store::BridgeAccountRow {
                    account_id: account.into(),
                    bridge_id: "bridge-graph".into(),
                    oauth_ref: None,
                    extra_json: "{}".into(),
                })
                .await
                .unwrap();
        }
        let all = [AccountBackend, Net, AddrbookSource, StoreKvScoped];
        grant_caps(&store, "bridge-graph", "", &all).await;

        let reg = build_plugin_host(&store).await;
        let engine = Arc::new(mw_engine::Engine::new(store.clone()));
        let (loaded, pim) = load_plugin_backends(&engine, &reg, &store).await;
        let pim = pim.expect("graph binds PIM slots");
        assert_eq!(loaded, 2);
        let row = store.get_plugin("bridge-graph").await.unwrap().unwrap();
        for account in ["acct-a", "acct-b"] {
            assert!(engine.is_registered(account), "{account} is served");
            assert!(pim.calendar(account).is_some(), "{account} has PIM slots");
        }
        let status = plugin_status(&store, &reg, &row).await;
        assert!(status.loaded && !status.restart_required);

        // Nothing changed: a sync leaves both instances running.
        sync_plugins(Some(&engine), &reg, &store).await;
        assert!(engine.is_registered("acct-a") && engine.is_registered("acct-b"));

        // Narrow the grant for everyone, keep the full one for account A only: B's
        // instance would now run with less than it was loaded with, so it is stopped.
        grant_caps(&store, "bridge-graph", "", &[AccountBackend, Net]).await;
        grant_caps(&store, "bridge-graph", "acct-a", &all).await;
        sync_plugins(Some(&engine), &reg, &store).await;
        assert!(engine.is_registered("acct-a"), "A's grant is unchanged");
        assert!(
            !engine.is_registered("acct-b"),
            "B must not keep its old grant"
        );
        assert!(pim.calendar("acct-a").is_some());
        assert!(pim.calendar("acct-b").is_none(), "B's PIM slots are gone");
        assert!(!pim.caps("acct-b").reactions);
        let status = plugin_status(&store, &reg, &row).await;
        assert!(status.loaded, "A's instance still runs");
        assert!(
            status.restart_required,
            "B's instance with the narrower grant starts at the next start-up"
        );

        // Disable the plugin: the last instance stops too.
        store
            .set_plugin_enabled("bridge-graph", false)
            .await
            .unwrap();
        sync_plugins(Some(&engine), &reg, &store).await;
        assert!(!engine.is_registered("acct-a"));
        assert!(pim.calendar("acct-a").is_none());
        let row = store.get_plugin("bridge-graph").await.unwrap().unwrap();
        let status = plugin_status(&store, &reg, &row).await;
        assert!(!status.loaded && !status.restart_required);
        assert_eq!(status.not_loaded, Some(NotLoaded::Disabled));

        // Enabling it again does not start it: that needs a restart, and says so.
        store
            .set_plugin_enabled("bridge-graph", true)
            .await
            .unwrap();
        sync_plugins(Some(&engine), &reg, &store).await;
        assert!(!engine.is_registered("acct-a"));
        let row = store.get_plugin("bridge-graph").await.unwrap().unwrap();
        let status = plugin_status(&store, &reg, &row).await;
        assert!(!status.loaded && status.restart_required);
        assert_eq!(status.not_loaded, None);
    }

    // ── 26.20 (t28-e14): the plugin `http-fetch` address policy ──────────────────

    #[test]
    fn loopback_is_reachable_only_when_the_url_names_it() {
        let loopback: IpAddr = "127.0.0.1".parse().unwrap();
        let private: IpAddr = "10.1.2.3".parse().unwrap();
        let metadata: IpAddr = "169.254.169.254".parse().unwrap();
        for host in ["localhost", "LOCALHOST", "127.0.0.1", "127.9.9.9", "[::1]"] {
            assert!(names_loopback(host), "{host}");
            assert!(plugin_fetch_policy(host)(&loopback), "{host}");
            assert!(!plugin_fetch_policy(host)(&metadata), "{host}");
        }
        for host in [
            "rspamd",
            "rspamd.internal",
            "10.1.2.3",
            "localhost.example.org",
        ] {
            assert!(!names_loopback(host), "{host}");
            let policy = plugin_fetch_policy(host);
            assert!(!policy(&loopback), "{host} must not reach loopback");
            assert!(policy(&private), "{host} may reach a private address");
            assert!(!policy(&metadata), "{host} must not reach link-local");
        }
    }

    #[test]
    fn only_a_same_host_redirect_is_followed() {
        let url = |s: &str| reqwest::Url::parse(s).unwrap();
        assert!(same_host_redirect(
            &url("http://a.test/x"),
            &url("http://a.test/y")
        ));
        assert!(same_host_redirect(
            &url("http://a.test/x"),
            &url("https://a.test/y")
        ));
        assert!(!same_host_redirect(
            &url("https://a.test/x"),
            &url("http://a.test/y")
        ));
        assert!(!same_host_redirect(
            &url("http://a.test/x"),
            &url("http://b.test/y")
        ));
        assert!(!same_host_redirect(
            &url("http://a.test/x"),
            &url("http://169.254.169.254/latest/meta-data")
        ));
    }

    /// Answer every connection on a fresh loopback port with `response`, counting
    /// the requests that arrive.
    async fn count_requests(response: String) -> (u16, Arc<std::sync::atomic::AtomicUsize>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let hits = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let seen = Arc::clone(&hits);
        tokio::spawn(async move {
            loop {
                let Ok((mut sock, _)) = listener.accept().await else {
                    return;
                };
                seen.fetch_add(1, Ordering::SeqCst);
                let mut buf = [0u8; 2048];
                let _ = sock.read(&mut buf).await;
                let _ = sock.write_all(response.as_bytes()).await;
                let _ = sock.shutdown().await;
            }
        });
        (port, hits)
    }

    fn plugin_get(url: String) -> HttpReq {
        HttpReq {
            method: "GET".into(),
            url,
            headers: Vec::new(),
            body: None,
        }
    }

    /// The fetcher itself, over real sockets: a link-local address and a URL with
    /// credentials are refused before any connection; a redirect to another host is
    /// handed back as the `302` and that host is never contacted. Before 26.20 the
    /// fetcher was a default `reqwest` client, which follows such a redirect.
    #[tokio::test]
    async fn the_plugin_fetcher_applies_the_address_policy_and_keeps_redirects_on_host() {
        let fetcher = ReqwestFetcher {
            host_auth: Vec::new(),
        };

        let err = fetcher
            .fetch(plugin_get("http://169.254.169.254/latest/meta-data".into()))
            .await
            .unwrap_err();
        assert!(err.contains("address policy"), "{err}");
        let err = fetcher
            .fetch(plugin_get("http://user:pw@127.0.0.1:9/".into()))
            .await
            .unwrap_err();
        assert!(err.contains("address policy"), "{err}");

        let ok = "HTTP/1.1 200 OK\r\ncontent-length: 2\r\nconnection: close\r\n\r\nhi".to_string();
        let (other_port, other_hits) = count_requests(ok.clone()).await;
        // `localhost` and `127.0.0.1` are different hosts to the redirect rule.
        let (first_port, first_hits) = count_requests(format!(
            "HTTP/1.1 302 Found\r\nlocation: http://localhost:{other_port}/elsewhere\r\n\
             content-length: 0\r\nconnection: close\r\n\r\n"
        ))
        .await;
        let resp = fetcher
            .fetch(plugin_get(format!("http://127.0.0.1:{first_port}/start")))
            .await
            .expect("the 302 itself is the answer");
        assert_eq!(resp.status, 302);
        assert_eq!(first_hits.load(Ordering::SeqCst), 1);
        assert_eq!(
            other_hits.load(Ordering::SeqCst),
            0,
            "the redirect target on another host must not be contacted"
        );

        // Positive control: the same target is reachable when asked for directly.
        let resp = fetcher
            .fetch(plugin_get(format!(
                "http://localhost:{other_port}/elsewhere"
            )))
            .await
            .expect("a named loopback host is reachable");
        assert_eq!((resp.status, resp.body), (200, b"hi".to_vec()));
        assert_eq!(other_hits.load(Ordering::SeqCst), 1);
    }

    // ── 26.19 (t19-e16): S1(a) payload clamp + S2 operator-reachable rate limit ──

    /// **Fails against the pre-fix code**: `parse_rate_limit` did not exist, because
    /// there was no operator-reachable setting to parse — the limit was hardcoded
    /// `None` at the mount site and the 0008 table has no column for it.
    #[test]
    fn the_rate_limit_setting_parses_every_operator_input() {
        // Unset ⇒ unlimited, which is the pre-26.19 behaviour, unchanged.
        assert_eq!(parse_rate_limit(None), None);
        // A plain value, and one an operator pasted with whitespace.
        assert_eq!(parse_rate_limit(Some("60")), Some(60));
        assert_eq!(parse_rate_limit(Some(" 60 ")), Some(60));
        // Zero is a deliberate hard stop, not "unset" — `check_rate` refuses every
        // request at a limit of 0, which is a kill switch that leaves the rest of the
        // deployment running.
        assert_eq!(parse_rate_limit(Some("0")), Some(0));
        // Junk degrades to unlimited with a warning rather than failing the boot: a
        // typo in an optional knob must not take a mail server down.
        assert_eq!(parse_rate_limit(Some("banana")), None);
        assert_eq!(parse_rate_limit(Some("-5")), None);
        assert_eq!(parse_rate_limit(Some("1.5")), None);
    }

    /// The wiring itself: an operator's configured value has to arrive at the object
    /// that enforces it. **Fails against the pre-fix code**, where `build_assist`
    /// hardcoded `rate_limit_per_min: None` and no configured value could reach the
    /// gateway from anywhere.
    #[tokio::test]
    async fn the_configured_rate_limit_reaches_the_gateway() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        store
            .put_assist_config(&mw_store::AssistConfigRow {
                scope: "deployment".into(),
                adapters_json: serde_json::json!({
                    "kind": "open-ai-compatible",
                    "base_url": "https://endpoint.invalid",
                    "api_key": "k",
                    "chat_model": "c",
                    "embed_model": "e",
                })
                .to_string(),
                capability_grants_json: serde_json::json!(["search-semantic"]).to_string(),
                data_ceilings_json: serde_json::json!({ "accounts": ["acct"] }).to_string(),
                enabled: true,
            })
            .await
            .unwrap();

        // `set_var` is `unsafe` in edition 2024 and process-global; the gate runs with
        // `--test-threads=1`, and the previous value is restored either way.
        let previous = std::env::var(ASSIST_RATE_LIMIT_ENV).ok();
        unsafe { std::env::set_var(ASSIST_RATE_LIMIT_ENV, "7") };
        let (gateway, granted) = build_assist(&store).await;
        match previous {
            Some(v) => unsafe { std::env::set_var(ASSIST_RATE_LIMIT_ENV, v) },
            None => unsafe { std::env::remove_var(ASSIST_RATE_LIMIT_ENV) },
        }

        assert_eq!(
            gateway.rate_limit_per_min(),
            Some(7),
            "the operator's configured limit must reach the gateway that enforces it"
        );
        // Positive control: the rest of the row was read too, so this is a wired
        // gateway rather than a default one that happens to agree.
        assert!(gateway.is_enabled());
        assert_eq!(granted, vec![AssistCapability::SearchSemantic]);
    }

    /// The `GatewayEmbeddings` comment claims attachments stay excluded from a re-rank
    /// "regardless of the deployment ceiling". This pins both halves of that claim to
    /// one value: the [`DataScope`] the request is dispatched under (what the audit row
    /// describes) and the [`mw_engine::search_semantic::EmbedScope`] the engine builds
    /// the payload from.
    ///
    /// **Fails against the pre-fix code**: `content_scope` did not exist, so only the
    /// audit-row half was ever set and the payload carried attachment text anyway.
    #[test]
    fn the_rerank_excludes_attachments_from_the_payload_and_the_audit_row_alike() {
        use mw_engine::EmbeddingProvider;

        let provider = GatewayEmbeddings {
            gateway: Arc::new(mw_assist::AssistGateway::new(AssistConfig::default())),
            model: String::new(),
        };

        let dispatched = provider.scope("acct");
        assert_eq!(dispatched.accounts, vec!["acct".to_string()]);
        assert!(!dispatched.include_e2ee);
        assert!(
            !dispatched.include_attachments,
            "the audit row says attachments were excluded..."
        );
        assert!(
            !provider.content_scope().include_attachments,
            "...and the payload the engine builds must agree with it"
        );
        assert_eq!(
            dispatched.include_attachments,
            provider.content_scope().include_attachments,
            "both halves derive from RERANK_INCLUDE_ATTACHMENTS, so the audit row \
             cannot drift from what actually left"
        );
    }
}
