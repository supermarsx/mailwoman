//! Mailwoman server (SPEC §4/§7, plan §2): session auth with an opaque
//! cookie, a same-origin JMAP proxy that injects upstream Basic auth, an
//! `/api/sanitize` endpoint that runs untrusted HTML through the disposable
//! `mw-render` child process (the §7.5 boundary), and the embedded SPA.

// `clippy::result_large_err` is a size heuristic, not a correctness lint. Under
// rustc/clippy 1.98.1 it fires on 18 helpers that return
// `Result<_, axum::response::Response>` (the early-return-a-response idiom; clippy
// measures the `Err` at 128+ bytes). Boxing the error would change those
// signatures across this crate while other work is editing it, for no behaviour
// change, so the refactor is deferred rather than done inside a toolchain pin.
// Only this one lint is allowed, and only for this crate.
#![allow(clippy::result_large_err)]
#![recursion_limit = "256"]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::anyhow;
use axum::body::{Body, Bytes};
use axum::extract::{Extension, OriginalUri, Path as UrlPath, Query, Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, Uri, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine as _;
use serde::Deserialize;
use serde_json::{Value, json};

use mw_directory::BindOutcome;
use mw_engine::Engine;
use mw_jmap::JmapClient;
use mw_store::{Credentials, NativeSessionRow, ServerKey, Store};

pub mod arf;
pub mod dlp;
pub mod engine_mode;
mod attachment_preview;
mod attachment_policy;
mod notification_policy;
mod source_bans;
mod delivery_status;
mod branding_routes;
mod analytics_adapter;
mod analytics_config;
mod analytics_routes;
mod appearance_customization;
mod analytics_runtime;
mod login_failure;
pub mod fonts;
pub mod hardening;
pub mod http_metrics;
mod request_history;
pub mod inbound_http;
pub mod knobs;
mod config_routes;
pub mod holidays;
pub mod push;
pub mod push_relay;
pub mod sharing;
pub mod tls;
pub mod watermark;
pub mod wkd;
// V6 additive route modules (plan §1.8): the admin console, OAuth authorization
// server, `/mcp` router, observability/metrics, outbound/inbound webhooks, the
// scoped `/api/v1` REST surface, and structured error reporting. All are
// implemented and mounted into `router()` below.
pub mod admin;
pub mod errors;
pub mod mcp;
pub mod oauth;
pub mod observability;
pub mod telemetry_config;
mod telemetry_routes;
pub mod rest;
pub mod webhooks;
// V7 additive route modules (plan §3): the GAL directory, password self-service,
// the Assist hook, the plugin registry, and the Nextcloud bridge. All are
// implemented and mounted into the V7 route group in `router()` below.
pub mod assist;
pub mod directory;
pub mod nextcloud;
mod nextcloud_accounts;
mod packages;
mod packages_model;
mod nextcloud_config;
mod nextcloud_gateway;
mod nextcloud_transport;
mod nextcloud_pim;
mod nextcloud_pim_transport;
mod nextcloud_pim_scheduler;
mod jmap_limits;
mod nextcloud_xml;
pub mod passwd;
mod primary_identity;
mod oidc_provider;
pub mod plugins;
// V7 MOUNT/WIRE (plan §3 e14): builds + injects the five V7 extensions, backs the
// host-service seams, and the extra endpoints. Owned by e14.
pub mod v7_mount;
// V8 SSO (26.9, t9-e3): the `/api/sso/*` login routes over the frozen `mw-sso`
// `SsoLogin` trait + the `/admin/sso` config CRUD. Additive; the password `/api/login`
// path is byte-unchanged. Mounted by `router()` below.
pub mod admin_sso;
pub mod sso;
// t10 (26.10) deferred-tail additive route modules (plan §3): the UI plugin
// surface and masked-alias routes. The OAuth Dynamic Client Registration handlers
// live in `oauth.rs`. All are implemented and mounted into the t10 route group in
// `router()` below.
pub mod masked;
pub mod ui_plugins;
// t14 (26.14) MOUNT/WIRE (plan §Wave-B E-mount): the admin-session-gated JWZ
// backfill endpoint + the server-metadata admin account passthrough helper the
// `/jmap/api` handler calls. Additive; fail-closed.
pub mod admin_maintenance;
pub mod jobs;
// t16 (26.16, e3): the login second-factor gate + enrolment/verification/recovery
// and session-management routes over `mw-mfa` + the 0015 store tables. The
// `gate_login` hook is called from both login branches below; `twofa_router()` is
// merged into `router()`.
pub mod twofa_routes;
pub(crate) mod audit;
pub(crate) mod alerts;
pub(crate) mod step_up;
mod security_center;
mod signin_cooldown;
mod request_protection;
mod signin_guard;
mod signin_geography;
mod deployment_security;
mod signin_challenge;
mod server_text;
// t16 (26.16, e10 chain link 3) MOUNT: the anonymizing image proxy + remote-image
// grants (e6, S7/S8), the account-preferences router (e18, W12/W13/W15), and the
// mailbox-import + webcal sync driver (e10, W18). e10 owns the `mod` lines + the
// merges in `router()`; the module bodies are owned by e6/e18/e10 respectively.
pub mod image_proxy;
pub mod import_routes;
mod account_drafts;
pub mod prefs_routes;
// 26.20 t32-e10b (f01): the settings model C — the registry (keys, types,
// defaults, scopes, environment precedence, the limit ceiling) and its routes.
pub mod settings_registry;
pub mod settings_routes;
mod features;
mod feature_jmap;
mod diagnostics;
mod admin_dashboard;
mod audit_projection;
mod auth_logs;
mod hosting_connections;
mod ssh_tunnels;
mod hosting_operations;
mod hosting_transport;
pub mod application_logs;
mod user_router;
// t16 (26.16, e10) A9: a runtime ManageSieve caller — compile the account's rules to
// a Sieve script and upload+activate it on the user's ManageSieve server.
pub mod sieve_sync;
// t29-e8: opt-in open tracking — token mint/bind/count and the unauthenticated
// `/t/{token}.gif` image, which records a time and nothing else.
mod open_tracking;
// V6 MOUNT (t6-e11): store adapters backing the frozen Batch-B persistence seams
// over the real 0007 tables.
mod stores_v6;
// t29-e6: the admin security policy applied at start, after a change and every
// 30 s (`admin_apply`), and the client-enforced part served at `/api/policy`.
mod admin_apply;
mod client_policy;
// t27-e3 (OH-1): the reader for the admin panel's per-account `disabled` and
// `force_password_change` flags, consulted on every auth path.
mod account_gate;
// V6 scoped-API-key enforcement middleware (t6-e11b): the `Send` guard that lets a
// scoped `mwk_…` key authorize `/api/v1/*` REST + adds IP-allowlist/rate-limit to
// `/mcp`. Keeps the cookie path unchanged.
mod scope_mw;
// t32-h9: the server, never the client, picks the mail upstream for a sign-in.
mod upstream_resolve;
pub(crate) mod domain_config;
mod domain_security;
mod domain_selection;
mod domain_auth;
mod username_format;
mod domain_detect;
pub mod domain_cli;
mod domain_upgrade;
pub mod domain_transport;
pub mod domain_probe;
mod integration_directory;
mod client_config;
mod remote_policy;
mod domain_routes;
// t32-h10: the answer for a request nothing else handled (a page for a browser
// navigation, `{error, code}` JSON otherwise), plus the return-path and
// static-path checks.
mod error_pages;
// t32-e24a: admin RBAC — the capability catalogue, the signed-in admin's context
// (`AdminCtx`, `AdminApiError`, `If-Match`), and the registry every admin route is
// mounted through (`admin_router::admin_endpoint`).
mod admin_accounts;
mod admin_passkeys;
pub mod admin_ctx;
mod admin_roles;
pub mod admin_router;
pub mod capabilities;

pub use engine_mode::ServerMode;
pub use hardening::HardeningConfig;
pub use push::PushHandle;
pub use push_relay::NativeAuthConfig;
pub use tls::{ReloadableResolver, TlsConfig, TlsListener};
pub use watermark::WatermarkConfig;

use hardening::SessionGuard;
// t20-e1's CIDR allowlist + raw-peer helpers. Reused rather than reimplemented so
// there is exactly one prefix matcher in the crate (trusted proxies here,
// `MW_HEADER_AUTH_TRUSTED_IPS` below).
use scope_mw::proxy::{CidrSet, peer_ip};

/// Cookie carrying the opaque session token.
const COOKIE_NAME: &str = "mw_session";

/// Strict CSP for the SPA shell (SPEC §7.4, t16 S10 tightening). The message body
/// is rendered in a separate sandboxed `<iframe srcdoc>` with its own restrictions.
///
/// `'wasm-unsafe-eval'` permits WebAssembly compilation/instantiation ONLY (not
/// JS `eval()`) — required for the client-side crypto worker (mw-crypto +
/// mw-sanitize wasm, plan §1.1/§1.3). It does not weaken the XSS posture the way
/// `'unsafe-eval'` would; untrusted message bodies render under the far stricter
/// per-message [`MESSAGE_CSP`] in a sandboxed iframe, unaffected by this.
///
/// # t16 S10 (e10 apply-site — chain link 3), completed in 26.17 (t17-e5)
/// This is now **exactly** [`image_proxy::SHELL_CSP_TIGHTENED`]. The two S10 changes
/// e6 delivered were (a) drop style `'unsafe-inline'` and (b) enforce Trusted Types;
/// both are shipped.
///   * **(a) shipped (26.16).** `style-src 'self'` is verified safe for the *shell*:
///     every inline style in the SPA is Solid object-form (`style={{…}}`) which Solid
///     applies through the CSSOM (`el.style.setProperty`), NOT a `style="…"`
///     attribute — CSSOM mutations are not subject to `style-src`. All real CSS is
///     bundled to a `'self'` `<link>` stylesheet (Vite extracts it in the production
///     build); the shell injects no runtime `<style>` element. So the `'unsafe-inline'`
///     style source e6 dropped is genuinely unused by the shell.
///   * **(b) shipped (26.17).** `require-trusted-types-for 'script'` blocks any
///     `Element.innerHTML =`/`iframe.srcdoc =` string assignment that does not go
///     through a Trusted Types policy. Solid's template instantiation
///     (`createElement("template")` + `.innerHTML = …`) does exactly that, so the SPA
///     entrypoint (`apps/web/src/main.tsx`) now registers a **default** Trusted Types
///     policy before `render()`. The policy passes app-controlled HTML through (every
///     sink is a Solid compile-time template, module-generated QR SVG, `sanitizeNoteHtml`
///     output, a detached ProseMirror parse container, or the plugin host's own iframe
///     srcdoc) and exposes only `createHTML`, so dynamic script/script-URL sinks stay
///     fail-closed. ProseMirror's scoped `ProseMirrorClipboard` policy reuses this
///     default. With the policy in place the directive is safe to enforce.
const CSP: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; \
     style-src 'self'; img-src 'self' blob: data:; font-src 'self'; manifest-src 'self'; \
     connect-src 'self' blob:; frame-src 'self'; worker-src 'self' blob:; \
     base-uri 'none'; form-action 'none'; require-trusted-types-for 'script'";

/// Compile-time use of e6's constant so the shell CSP tie does not leave
/// `SHELL_CSP_TIGHTENED` an unused (`dead_code`) item. A `#[cfg(test)]` assertion
/// (`csp_equals_e6_tightened`) verifies the shipped [`CSP`] is exactly that constant,
/// so the tie can't silently drift.
const _: &str = image_proxy::SHELL_CSP_TIGHTENED;

/// The raw seal-master-key bytes behind a [`ServerKey`] (for the C6 PQC wrap).
/// `ServerKey` exposes only its hex form, so decode it back to the 32 raw bytes;
/// the hex is well-formed by construction (`to_hex`), so a bad nibble is treated
/// as `0` rather than panicking.
fn seal_key_bytes(key: &ServerKey) -> Vec<u8> {
    let hex = key.to_hex();
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let hi = (pair[0] as char).to_digit(16).unwrap_or(0);
            let lo = (pair[1] as char).to_digit(16).unwrap_or(0);
            ((hi << 4) | lo) as u8
        })
        .collect()
}

/// A locked-down CSP returned alongside sanitized message HTML so the web app
/// can apply it to the per-message iframe (§7.4). Additive: the SPA shell keeps
/// [`CSP`]; this only constrains untrusted message bodies further.
const MESSAGE_CSP: &str = "default-src 'none'; img-src blob: data:; \
     style-src 'unsafe-inline'; font-src 'self'; media-src blob: data:; \
     base-uri 'none'; form-action 'none'";

/// Cookie carrying the readable double-submit CSRF token.
const CSRF_COOKIE: &str = hardening::CSRF_COOKIE;

/// Embedded production assets. The folder must exist at compile time; a
/// committed `.gitkeep` guarantees that before the web build runs. In dev,
/// `MW_WEB_DIR` (see [`AppConfig::web_dir`]) serves from disk instead.
#[derive(rust_embed::RustEmbed)]
#[folder = "../../apps/web/dist"]
struct WebAssets;

/// Runtime configuration (populated from env by the CLI, or by tests).
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// SQLite database path (file created if missing).
    pub db_path: String,
    /// Hex-encoded 32-byte server key; `None` generates an ephemeral one.
    pub server_key_hex: Option<String>,
    /// Serve static assets from this directory instead of the embedded set.
    pub web_dir: Option<PathBuf>,
    /// Add `Secure` to the session cookie (enable behind TLS).
    ///
    /// t20 (26.19, e3): a **floor**, not the final answer. `Secure` is now also
    /// derived from the request's effective scheme, so a TLS deployment that
    /// leaves this `false` still gets it (see [`ExternalBase`]). Setting it here
    /// can only raise the result; `MW_COOKIE_SECURE` overrides both ways.
    pub cookie_secure: bool,
    /// Proxy a JMAP upstream (V0 default) or drive IMAP/POP3 via `mw-engine`.
    pub mode: ServerMode,
    /// Web-hardening knobs (§7.4).
    pub hardening: HardeningConfig,
    /// V4 crypto/security endpoint config: WKD publishing, ARF relay, DLP config,
    /// watermark overlay (plan §3 e7).
    pub security: SecurityConfig,
}

/// Config for the V4 crypto/security endpoints (plan §3 e7), env-sourced in prod
/// (see [`SecurityConfig::from_env`]) and set explicitly by tests. All fields
/// default to "feature off" so a deployment that configures none behaves exactly
/// as before.
#[derive(Debug, Clone, Default)]
pub struct SecurityConfig {
    /// Directory of published PUBLIC keys served over WKD (env: `MW_WKD_DIR`).
    pub wkd_dir: Option<PathBuf>,
    /// Abuse address ARF reports are addressed to (env: `MW_ABUSE_ADDRESS`).
    pub abuse_address: Option<String>,
    /// Spool directory ARF reports are written to for relay (env: `MW_ABUSE_SPOOL`).
    pub abuse_spool: Option<PathBuf>,
    /// DLP rules source — inline JSON or a file path (env: `MW_DLP_RULES`).
    pub dlp_rules: Option<String>,
    /// Web watermark honesty-overlay config (§7.6).
    pub watermark: WatermarkConfig,
    /// Proxy mode: the JMAP upstream origins a login may name (env:
    /// `MW_JMAP_UPSTREAMS`, comma-separated, e.g. `https://jmap.example.org`).
    ///
    /// * `None` (unset) — any upstream whose every connection target passes
    ///   `mw_egress::ip_allowed`: public addresses work; loopback, RFC1918,
    ///   link-local (including `169.254.169.254`), CGNAT, ULA and the other
    ///   blocked ranges are refused before a connection is made (t24 decision D3).
    /// * `Some(list)` — only an upstream whose origin (scheme, host, port) is on
    ///   the list, and for those the address floor does not apply: naming an
    ///   origin is how an operator permits an internal JMAP server. The connection
    ///   is still pinned to the address resolved at check time. An empty or wholly
    ///   unparseable list refuses every upstream.
    ///
    /// Either way the session's `apiUrl`/`downloadUrl`/`uploadUrl` must share the
    /// upstream's origin; see `upstream_client`.
    pub jmap_upstreams: Option<Vec<String>>,
    /// The upstream every sign-in uses until per-domain configuration exists
    /// (env: `MW_UPSTREAM_DEFAULT`, t32-h9): a JMAP URL in proxy mode
    /// (`https://jmap.example.org`), an `imap(s)://host:port` or
    /// `pop3(s)://host:port` URL in engine mode. Its origin counts as operator
    /// named, like an `MW_JMAP_UPSTREAMS` entry. See `upstream_resolve`.
    pub upstream_default: Option<String>,
}

impl SecurityConfig {
    /// Populate from the environment (used by the `serve` CLI path).
    pub fn from_env() -> Self {
        let path = |k: &str| {
            std::env::var(k)
                .ok()
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
        };
        let string = |k: &str| std::env::var(k).ok().filter(|s| !s.is_empty());
        let flag = |k: &str| {
            std::env::var(k)
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false)
        };
        Self {
            wkd_dir: path("MW_WKD_DIR"),
            abuse_address: string("MW_ABUSE_ADDRESS"),
            abuse_spool: path("MW_ABUSE_SPOOL"),
            dlp_rules: string("MW_DLP_RULES"),
            watermark: WatermarkConfig {
                enabled: flag("MW_WATERMARK"),
                opacity: string("MW_WATERMARK_OPACITY")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0.08),
            },
            jmap_upstreams: string("MW_JMAP_UPSTREAMS").map(|v| parse_jmap_upstreams(&v)),
            upstream_default: string("MW_UPSTREAM_DEFAULT").map(|v| v.trim().to_string()),
        }
    }
}

/// Split `MW_JMAP_UPSTREAMS` into entries, warning about any that cannot name an
/// origin. Bad entries are dropped rather than failing startup; they can never
/// match, so a list of only bad entries refuses every upstream (fail closed).
fn parse_jmap_upstreams(raw: &str) -> Vec<String> {
    let entries: Vec<String> = raw
        .split(',')
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .map(str::to_string)
        .collect();
    for entry in &entries {
        if upstream_list_origin(entry).is_none() {
            tracing::warn!(
                "MW_JMAP_UPSTREAMS: {entry:?} is not an http(s) origin and will never match"
            );
        }
    }
    if !entries.iter().any(|e| upstream_list_origin(e).is_some()) {
        tracing::warn!(
            "MW_JMAP_UPSTREAMS is set but names no usable origin: every proxy login will be refused"
        );
    }
    entries
}

// ---------------------------------------------------------------------------
// External identity — the canonical public origin (t20 B6/B8)
// ---------------------------------------------------------------------------

/// Where this deployment's **public** scheme and host come from.
///
/// mailwoman is normally fronted by a reverse proxy that terminates TLS, so the
/// scheme the browser used is not the scheme of the connection the server sees.
/// Getting that wrong is what let a TLS deployment that forgot `MW_COOKIE_SECURE`
/// ship session cookies without `Secure`, over a connection the server believed
/// was plaintext.
///
/// Resolution order, most authoritative first:
///
/// 1. **`MW_PUBLIC_URL`** — the canonical external base, e.g.
///    `https://mail.example.org`. This is the recommended setting: it is read
///    from the environment, so no request header can move it.
/// 2. **`X-Forwarded-Proto` / `X-Forwarded-Host`, but only when the connecting
///    peer is inside `MW_TRUSTED_PROXIES`.** An untrusted peer's forwarded
///    headers are ignored outright — otherwise any client could assert
///    `X-Forwarded-Proto: https` and change the server's idea of its own origin.
///    Which *element* of a multi-valued header is read matters as much as which
///    peer may send one: see [`nearest_forwarded_token`].
/// 3. **The `Host` header, scheme `http`** — the pre-existing behaviour, kept so
///    a deployment that sets none of these behaves exactly as it did before.
#[derive(Debug, Clone, Default)]
pub(crate) struct ExternalBase {
    /// Scheme from `MW_PUBLIC_URL` (`true` = https).
    public_https: Option<bool>,
    /// Authority (`host[:port]`) from `MW_PUBLIC_URL`. Read by
    /// [`ExternalBase::host`], whose consumers land in a later wave.
    #[allow(dead_code)]
    public_host: Option<String>,
    /// `MW_TRUSTED_PROXIES`, parsed. Empty ⇒ no forwarded header is ever trusted.
    trusted_proxies: CidrSet,
    /// Whether this process is itself the TLS endpoint (ACME or an external
    /// cert). Header-independent, like [`ExternalBase::public_https`], and the
    /// floor under the forwarded answer rather than a competing opinion.
    self_terminated_tls: bool,
    /// Explicit `MW_COOKIE_SECURE`; `None` ⇒ derive from the effective scheme.
    cookie_secure_override: Option<bool>,
    /// Prebuilt `Strict-Transport-Security` value; `None` ⇒ never emitted.
    hsts: Option<HeaderValue>,
}

impl ExternalBase {
    pub(crate) fn from_env() -> Self {
        let var = |k: &str| {
            std::env::var(k)
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|s| !s.is_empty())
        };
        let (public_https, public_host) = match var("MW_PUBLIC_URL") {
            Some(raw) => match split_public_url(&raw) {
                Some((https, host)) => (Some(https), Some(host)),
                None => {
                    tracing::warn!(
                        "MW_PUBLIC_URL is not an absolute http(s) URL, ignoring it: {raw}"
                    );
                    (None, None)
                }
            },
            None => (None, None),
        };
        Self {
            public_https,
            public_host,
            trusted_proxies: CidrSet::parse(&var("MW_TRUSTED_PROXIES").unwrap_or_default()),
            self_terminated_tls: tls_terminated_here(
                var("MW_ACME").as_deref(),
                var("MW_TLS_CERT").as_deref(),
                var("MW_TLS_KEY").as_deref(),
            ),
            cookie_secure_override: var("MW_COOKIE_SECURE").and_then(|v| parse_env_bool(&v)),
            hsts: build_hsts(),
        }
    }

    /// Whether an `X-Forwarded-*` header on this request may be believed: only
    /// when the operator has declared trusted proxies AND the peer we are
    /// actually talking to is one of them.
    ///
    /// Deliberately keyed on `MW_TRUSTED_PROXIES` alone, not on
    /// `MW_FORWARDED_MODE`: that setting selects which *client-IP* header is
    /// read (`X-Forwarded-For` vs `Forwarded`), which is a different question
    /// from whether this peer is a proxy at all.
    fn forwarded_trusted(&self, ext: &axum::http::Extensions) -> bool {
        if self.trusted_proxies.is_empty() {
            return false;
        }
        match peer_ip(ext) {
            // No peer address available (in-process test transport, or a serve
            // path that installed no `ConnectInfo`) ⇒ nothing to match ⇒ no trust.
            None => false,
            Some(ip) => self.trusted_proxies.contains(ip),
        }
    }

    /// Whether `MW_PUBLIC_URL` states an https base. Header-independent, so it is
    /// the one scheme fact available outside a request.
    pub(crate) fn declares_https(&self) -> bool {
        self.public_https.unwrap_or(false)
    }

    /// The scheme the *client* used to reach us.
    ///
    /// Three sources, in decreasing authority, composed rather than competing:
    ///
    /// 1. `MW_PUBLIC_URL` — stated by the operator, so nothing can move it.
    /// 2. A trusted proxy's `X-Forwarded-Proto`. When a proxy is in front, only
    ///    the proxy knows what the *browser* used, so its answer outranks our own
    ///    transport in both directions: a trusted `http` means the client hop was
    ///    plaintext even if the proxy re-encrypted to us.
    /// 3. **Our own listener**, when this process terminates TLS itself. No
    ///    header is involved and none can suppress it — an L4 balancer doing TLS
    ///    passthrough cannot add one, which is exactly the deployment the
    ///    built-in ACME client exists for and the one this used to get wrong.
    pub(crate) fn is_https(&self, headers: &HeaderMap, ext: &axum::http::Extensions) -> bool {
        if let Some(https) = self.public_https {
            return https;
        }
        if self.forwarded_trusted(ext)
            && let Some(proto) = nearest_forwarded_token(headers, "x-forwarded-proto")
        {
            return proto.eq_ignore_ascii_case("https");
        }
        self.self_terminated_tls
    }

    /// The host the client addressed us by, for building absolute external URLs.
    ///
    /// NOT YET CONSUMED IN THIS TAG. The two places that build host-derived
    /// absolute URLs — the OAuth DCR issuer (`oauth.rs`, `header::HOST` with a
    /// hostname-guessed scheme) and the WebAuthn RP origin
    /// (`twofa_routes::derive_rp`) — live in files owned by other lanes this
    /// wave, so adopting this there is a follow-up. Unit-tested below so the
    /// resolution order is pinned before anyone depends on it.
    #[allow(dead_code)]
    pub(crate) fn host(&self, headers: &HeaderMap, ext: &axum::http::Extensions) -> Option<String> {
        if let Some(h) = &self.public_host {
            return Some(h.clone());
        }
        if self.forwarded_trusted(ext)
            && let Some(h) = nearest_forwarded_token(headers, "x-forwarded-host")
        {
            return Some(h);
        }
        headers
            .get(header::HOST)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
    }

    /// The canonical external base URL (`scheme://host`), when a host is known.
    /// See [`ExternalBase::host`] for why nothing consumes this yet.
    #[allow(dead_code)]
    pub(crate) fn base_url(
        &self,
        headers: &HeaderMap,
        ext: &axum::http::Extensions,
    ) -> Option<String> {
        let scheme = if self.is_https(headers, ext) {
            "https"
        } else {
            "http"
        };
        self.host(headers, ext).map(|h| format!("{scheme}://{h}"))
    }

    /// Whether cookies set on this response must carry `Secure`.
    ///
    /// `configured` is the deployment's explicit [`AppConfig::cookie_secure`]; it
    /// can only ever raise the answer, never lower it. `MW_COOKIE_SECURE` remains
    /// an absolute override in both directions so an operator can pin the old
    /// behaviour (and so the dev/test harnesses stay on plain http).
    pub(crate) fn cookie_secure(
        &self,
        configured: bool,
        headers: &HeaderMap,
        ext: &axum::http::Extensions,
    ) -> bool {
        match self.cookie_secure_override {
            Some(forced) => forced,
            None => configured || self.is_https(headers, ext),
        }
    }

    /// The `Strict-Transport-Security` value to emit, if any. Only ever sent over
    /// an effective-https request: sending it over plaintext is meaningless (an
    /// attacker who can rewrite the response can strip it) and pinning a host to
    /// https from an http response would lock out a deployment that has no TLS.
    pub(crate) fn hsts(
        &self,
        headers: &HeaderMap,
        ext: &axum::http::Extensions,
    ) -> Option<&HeaderValue> {
        if self.is_https(headers, ext) {
            self.hsts.as_ref()
        } else {
            None
        }
    }
}

/// Split `MW_PUBLIC_URL` into `(is_https, authority)`, rejecting anything that is
/// not an absolute http(s) URL. Any path (`https://host/mail`) is dropped — the
/// sub-path is `MW_BASE_PATH`'s job, not this one's.
fn split_public_url(raw: &str) -> Option<(bool, String)> {
    let (scheme, rest) = raw.split_once("://")?;
    let https = match scheme.to_ascii_lowercase().as_str() {
        "https" => true,
        "http" => false,
        _ => return None,
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest).trim();
    if authority.is_empty() {
        return None;
    }
    Some((https, authority.to_ascii_lowercase()))
}

/// Does this process terminate TLS itself?
///
/// Mirrors `main.rs`'s transport decision (`serve`: ACME > external cert >
/// plaintext) from the environment, because `build_app` — and so
/// [`ExternalBase::from_env`] — runs *before* that decision is made and has no
/// other way to see it. Split from the environment read so the precedence is
/// directly testable without racing every other test in the binary.
///
/// **Known limit, and the reason this is not the whole fix.** `main.rs` accepts
/// the same three settings as command-line flags (`--acme`, `--tls-cert`,
/// `--tls-key`); clap reads the environment only as their fallback. A deployment
/// that passes them as flags therefore still looks plaintext here. Every
/// documented deployment shape is env-driven (compose, helm, systemd
/// `EnvironmentFile`), so this covers them — but the complete fix is for
/// `main.rs` to hand the decision it already makes to `build_app` instead of it
/// being re-derived here, and `main.rs` is outside this lane's locks.
fn tls_terminated_here(acme: Option<&str>, cert: Option<&str>, key: Option<&str>) -> bool {
    let present = |v: Option<&str>| v.is_some_and(|s| !s.trim().is_empty());
    // A cert without its key (or the reverse) is not a TLS listener: `serve`
    // falls through to plaintext on a half-configured pair, and so must this.
    present(acme) || (present(cert) && present(key))
}

/// `Strict-Transport-Security` from the environment. Defaults to one year, no
/// `includeSubDomains`, no `preload`.
///
/// `includeSubDomains` is **opt-in** on purpose: it applies to every host under
/// the domain, including ones this deployment does not serve, so switching it on
/// by default could take unrelated sibling hosts offline. `preload` is likewise
/// opt-in — it is effectively irreversible once a domain is on the browser list.
/// `MW_HSTS=off` disables the header entirely.
fn build_hsts() -> Option<HeaderValue> {
    let disabled = std::env::var("MW_HSTS")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "off" || v == "0" || v == "false"
        })
        .unwrap_or(false);
    if disabled {
        return None;
    }
    let max_age: u64 = std::env::var("MW_HSTS_MAX_AGE")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(31_536_000);
    let mut value = format!("max-age={max_age}");
    if env_flag("MW_HSTS_INCLUDE_SUBDOMAINS") {
        value.push_str("; includeSubDomains");
    }
    if env_flag("MW_HSTS_PRELOAD") {
        value.push_str("; preload");
    }
    HeaderValue::from_str(&value).ok()
}

/// Parse an explicit boolean env var. Unrecognised values yield `None` (treated
/// as "not set") rather than silently meaning `false`.
fn parse_env_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// The forwarded-header token written by the **nearest** hop: the last
/// comma-separated element of the last header line, trimmed, lowercased and
/// non-empty.
///
/// Same walk direction as the client-IP model in [`crate::scope_mw::proxy`], and
/// for the same reason. A proxy that *appends* to `X-Forwarded-Proto` /
/// `X-Forwarded-Host` instead of replacing it (Envoy's shape, and any chained
/// tier where an outer proxy passes the client's value through) leaves the
/// client's own value on the **left**. Reading leftmost would therefore use a
/// value the client chose, even though [`ExternalBase::forwarded_trusted`] has
/// already confirmed the peer is a declared proxy: the peer gate decides whether
/// to look at the header at all, and says nothing about which element inside it
/// the proxy wrote.
///
/// Two things `proxy::resolve` does have no analogue here, both forced by the
/// header's shape rather than chosen:
///
///   * these values are not addresses, so an element cannot be tested for
///     membership of `MW_TRUSTED_PROXIES` and the "skip hops that are themselves
///     trusted proxies" step cannot be reproduced. Through several trusted tiers
///     the last element is the scheme/host of the *innermost* hop, which
///     under-reports https rather than over-reporting it — no `Secure`, no HSTS,
///     the same safe direction as the RFC 7239 `proto=` gap (F1);
///   * an unusable element (empty, or a header line that is not text) yields
///     `None` rather than reaching further left, mirroring how `resolve` stops at
///     the last verified address on an `unknown` hop instead of stepping over it.
fn nearest_forwarded_token(headers: &HeaderMap, name: &str) -> Option<String> {
    let mut nearest = None;
    for value in headers.get_all(name) {
        // Unreadable as text ⇒ we cannot see what this hop wrote, and falling back
        // to an earlier line would reach past it to something further left.
        nearest = Some(value.to_str().ok()?);
    }
    let token = nearest?.rsplit(',').next()?.trim();
    (!token.is_empty()).then(|| token.to_ascii_lowercase())
}

// ---------------------------------------------------------------------------
// Sub-path hosting (t20 B4) — `MW_BASE_PATH`
// ---------------------------------------------------------------------------

/// The deploy prefix this process serves under, read from `MW_BASE_PATH` at boot.
///
/// mailwoman may be hosted at the origin root (`/`) or under a prefix (`/mail`).
/// That is a **runtime** decision, never a build-time one: the same single binary
/// and the same container image must serve either. The SPA is therefore built
/// with a relative asset base (`apps/web/vite.config.ts`, `base: './'`) so every
/// chunk, worker and wasm module resolves against the URL of the chunk importing
/// it, and the prefix reaches the app at serve time instead of at build time.
///
/// Everything else follows from this one value:
///   * [`mount_under_base_path`] nests the whole router under the prefix, so
///     every route in this file matches under `/mail/…` unchanged (and a proxy
///     that strips the prefix itself still works — see there);
///   * `index.html` gains a `<script src="{base}/__mw_base.js">` publishing
///     `window.__MW_BASE__`, the value `apps/web/src/api/basePath.ts` reads to
///     prefix its `fetch`/`WebSocket`/`EventSource` targets and the SW scope;
///   * the JMAP session object's URLs and every `Set-Cookie` `Path` are prefixed.
///
/// Unset (the default) ⇒ `""` ⇒ every path below is byte-identical to before.
fn base_path_from_env() -> String {
    normalize_base_path(&std::env::var("MW_BASE_PATH").unwrap_or_default())
}

/// Canonicalize an operator-supplied prefix to `""` (origin root) or a
/// leading-slash, no-trailing-slash prefix such as `/mail`.
///
/// Accepts the same spellings as the SPA's `normalizeBase()` — `mail`, `/mail`,
/// `/mail/`, `/`, `` — and is deliberately **stricter** about everything else:
/// this side is the authority that injects the value into the page and into
/// `Set-Cookie`, so anything that is not a plain path prefix (a query, a
/// fragment, `..`, an empty segment, a scheme, whitespace) is refused outright
/// rather than normalized into something surprising. A refused value logs and
/// falls back to the origin root, which is the pre-sub-path behaviour.
fn normalize_base_path(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    let body = trimmed.strip_prefix('/').unwrap_or(trimmed);
    if body.is_empty() {
        return String::new();
    }
    let plain = body.split('/').all(|seg| {
        !seg.is_empty()
            && seg != "."
            && seg != ".."
            && seg
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~'))
    });
    if !plain {
        tracing::warn!("MW_BASE_PATH is not a plain path prefix, ignoring it: {raw}");
        return String::new();
    }
    format!("/{body}")
}

/// How a request path relates to the configured deploy prefix.
#[derive(Debug, PartialEq, Eq)]
enum BaseMatch<'a> {
    /// Exactly the prefix with no trailing slash (`/mail`). The shell must be
    /// reached at `/mail/`, or its relative asset URLs resolve against `/`.
    Exact,
    /// Under the prefix; the remainder always starts with `/`.
    Under(&'a str),
    /// Not under the prefix — passed through untouched.
    Outside,
}

/// Match a request path against `base`, on a segment boundary only: with
/// `base = /mail`, `/mailbox` is [`BaseMatch::Outside`], not a prefix hit.
fn match_base_path<'a>(path: &'a str, base: &str) -> BaseMatch<'a> {
    if base.is_empty() {
        return BaseMatch::Outside;
    }
    let Some(rest) = path.strip_prefix(base) else {
        return BaseMatch::Outside;
    };
    if rest.is_empty() {
        BaseMatch::Exact
    } else if rest.starts_with('/') {
        BaseMatch::Under(rest)
    } else {
        BaseMatch::Outside
    }
}

/// Serve the whole application under the deploy prefix as well as at the root
/// (t20 B4).
///
/// **Why nesting and not a URI-rewriting middleware.** The obvious shape — a
/// `Router::layer` that strips the prefix from `req.uri()` — silently does not
/// work: `Router::layer` wraps each *matched route's* service, so routing has
/// already happened by the time it runs. Only the fallback ever sees the
/// rewritten URI, which means static assets appear to work while every `/api/*`
/// and `/jmap/*` request falls through to the SPA fallback and is answered with
/// `index.html` under a `200`. (Observed against a running server under
/// `MW_BASE_PATH=/mail`, not deduced.) axum's own answer is to wrap the router in
/// a `tower::Layer` before serving, but that changes the type this crate returns
/// from [`build_app`]. Nesting does the strip at routing time, which is where it
/// belongs, and needs nothing new.
///
/// The router is ALSO merged at the root, so both reverse-proxy idioms work:
/// nginx `proxy_pass http://app;` (prefix preserved — stripped by the nest) and
/// `proxy_pass http://app/;` (prefix already stripped upstream — served by the
/// root copy). That is also what keeps `/healthz` reachable for a container
/// health check and `/.well-known/*` reachable at the origin root, where WKD and
/// JMAP autodiscovery have to live whatever the prefix is.
///
/// The bare prefix (`/mail`, no trailing slash) is 308-redirected to `/mail/` in
/// [`static_handler`] — see there for why the trailing slash is load-bearing.
fn mount_under_base_path(app: Router, base: &str) -> Router {
    if base.is_empty() {
        return app;
    }
    Router::new().nest(base, app.clone()).merge(app)
}

#[derive(Clone)]
pub(crate) struct AppState {
    started_at: std::time::SystemTime,
    started: std::time::Instant,
    config_snapshot: config_routes::BootConfigSnapshot,
    request_history: Arc<request_history::Recorder>,
    store: Store,
    pub(crate) scheduler: Arc<jobs::Scheduler>,
    render_bin: Option<PathBuf>,
    /// Whether a render kernel jail is expected on this deployment (DQ4/S6, via
    /// [`mw_sandbox::jail_expected`]). When set, the in-process render fallbacks
    /// (HTML sanitize / `.oft` parse) fail closed instead of running hostile input in
    /// this trusted process without the jail.
    render_jail_required: bool,
    web_dir: Option<PathBuf>,
    /// The deployment's explicit cookie-`Secure` setting. A floor only — the
    /// effective value per response comes from
    /// [`ExternalBase::cookie_secure`], which raises it when the request
    /// actually arrived over https.
    cookie_secure: bool,
    /// t20 (26.19, e3): how the public scheme/host is resolved (`MW_PUBLIC_URL`,
    /// then trusted `X-Forwarded-*`, then `Host`), plus the HSTS posture that
    /// follows from it.
    pub(crate) external: Arc<ExternalBase>,
    /// t20 (26.19, e13): the deploy prefix (`MW_BASE_PATH`), resolved once at
    /// boot. `""` = origin root. See [`base_path_from_env`].
    base_path: Arc<str>,
    /// Present only in engine mode; drives IMAP/POP3 behind the JMAP surface.
    engine: Option<Arc<Engine>>,
    /// Realtime push fan-out feeding `/jmap/ws` + `/jmap/eventsource`.
    pub(crate) push: PushHandle,
    /// In-process session idle/absolute-timeout tracking.
    sessions: Arc<SessionGuard>,
    signin_cooldown: Arc<signin_cooldown::Cooldown>,
    request_protection: Arc<request_protection::Runtime>,
    signin_challenges: Arc<signin_challenge::IssuanceLimiter>,
    hardening: HardeningConfig,
    /// V4 crypto/security endpoint config (plan §3 e7).
    security: SecurityConfig,
    /// V5 native-client CORS/origin allowlist (plan §2.2). Empty = OFF (default) →
    /// browser deployments emit no `Access-Control-*` headers.
    native_auth: NativeAuthConfig,
    /// V6 MOUNT (plan §3 e11): the mounted OAuth AS, admin domain logic, and the
    /// store [`ServerKey`] backing the admin/oauth/mcp/zero-access surfaces.
    pub(crate) v6: Arc<V6State>,
    /// t16 (26.16, e3): server-side login second-factor state (one-shot pending
    /// logins + passkey registration challenges). Inert until a user enrols a factor
    /// or an admin sets a require-2FA policy — the default login path is unchanged.
    pub(crate) twofa: Arc<twofa_routes::TwofaState>,
    pub(crate) step_up: Arc<step_up::StepUpState>,
    pub(crate) analytics: Arc<analytics_runtime::Runtime>,
    /// t16 (26.16, e3, A1): the GAL/LDAP directory handle, so the login path can add
    /// an LDAP-bind pre-check (§18.3) when `MW_LDAP_BIND_AUTH` is set. Inert (never
    /// consulted) unless that env flag is on — the default login path is unchanged.
    pub(crate) directory: directory::DirectoryHandle,
}

/// V6 mount-time configuration (plan §3 e11). All fields default to "off" so a
/// deployment that configures none behaves exactly as before. Sourced from the
/// environment by [`V6Config::from_env`] (the `serve` path), or injected directly
/// by tests via [`build_app_full`].
#[derive(Debug, Clone)]
pub struct V6Config {
    /// Whether the `/admin/*` panel surface is enabled (`MW_ADMIN_ENABLED`, default
    /// on). When off, admin routes return `401` (the panel is unreachable).
    pub admin_enabled: bool,
    /// Admin operator username (`MW_ADMIN_USER`). `None` → admin login always fails.
    pub admin_username: Option<String>,
    /// Admin operator password (`MW_ADMIN_PASSWORD`). Compared constant-time.
    pub admin_password: Option<String>,
    /// Redis/Valkey URL for the layered cache (`MW_REDIS_URL`). `None` → memory +
    /// store only (never authoritative).
    pub redis_url: Option<String>,
}

impl Default for V6Config {
    fn default() -> Self {
        Self {
            admin_enabled: true,
            admin_username: None,
            admin_password: None,
            redis_url: None,
        }
    }
}

impl V6Config {
    /// Populate from the environment (the `serve` CLI path).
    pub fn from_env() -> Self {
        let s = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        Self {
            admin_enabled: std::env::var("MW_ADMIN_ENABLED")
                .map(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
                .unwrap_or(true),
            admin_username: s("MW_ADMIN_USER"),
            admin_password: s("MW_ADMIN_PASSWORD"),
            redis_url: s("MW_REDIS_URL"),
        }
    }
}

/// The mounted V6 surface handles carried in [`AppState`].
pub(crate) struct V6State {
    pub(crate) auth: Arc<mw_oauth::AuthServer<stores_v6::OAuthStoreAdapter>>,
    pub(crate) admin: mw_admin::Admin,
    pub(crate) admin_enabled: bool,
    pub(crate) admin_username: Option<String>,
    pub(crate) admin_password: Option<String>,
    /// Ephemeral device-pairing relay (zero-access §9.1): `pairingId → envelopeB64`.
    /// The server relays ciphertext only; it never sees a plaintext key.
    pub(crate) pairing: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

/// Build the fully-wired axum application from configuration.
pub async fn build_app(config: AppConfig) -> anyhow::Result<Router> {
    Ok(build_app_with_push(config).await?.0)
}

/// Build an [`mw_admin::Admin`] backed by the real 0007 store tables — the backing
/// for the `mailwoman admin` CLI (plan §3 e11, GitOps-friendly). Keeps the
/// store-adapter type private to this crate.
pub async fn build_admin(
    db_path: &str,
    server_key_hex: Option<&str>,
) -> anyhow::Result<mw_admin::Admin> {
    let key = match server_key_hex {
        Some(h) => ServerKey::from_hex(h).map_err(|_| anyhow!("MW_SERVER_KEY is not valid hex"))?,
        None => ServerKey::generate(),
    };
    let store = Store::open(db_path, key).await?;
    Ok(admin_apply::admin_for(
        store,
        None,
        admin_apply::env_overlay()?,
    ))
}

/// Like [`build_app`] but also returns the [`PushHandle`] feeding the realtime
/// channel. In engine mode the handle mirrors the `Engine` broadcast; tests use
/// the returned handle to inject synthetic `StateChange`s and prove the WS/SSE
/// wire path without a live engine. V6 config is sourced from the environment.
pub async fn build_app_with_push(config: AppConfig) -> anyhow::Result<(Router, PushHandle)> {
    build_app_full(config, V6Config::from_env()).await
}

/// Build the fully-wired app with an explicit [`V6Config`] (tests inject admin
/// credentials / redis without touching process env).
pub async fn build_app_full(
    config: AppConfig,
    v6config: V6Config,
) -> anyhow::Result<(Router, PushHandle)> {
    build_app_inner(config, v6config, sso::SsoProviderSource::Store).await
}

/// Test seam (t9-e3 `sso_e2e`): build the app with mock [`mw_sso::SsoLogin`]
/// providers injected instead of the ones built from the 0009 `sso_config` rows, so
/// the `/api/sso/*` routes can be driven end-to-end without a live IdP. Everything
/// else — the store, sessions, `finish_login` — is the real path.
#[doc(hidden)]
pub async fn build_app_with_sso_mock(
    config: AppConfig,
    v6config: V6Config,
    providers: Vec<(String, sso::SsoEntry)>,
) -> anyhow::Result<(Router, PushHandle)> {
    let map: std::collections::HashMap<String, sso::SsoEntry> = providers.into_iter().collect();
    build_app_inner(
        config,
        v6config,
        sso::SsoProviderSource::Mock(Arc::new(map)),
    )
    .await
}

async fn build_app_inner(
    config: AppConfig,
    v6config: V6Config,
    sso_source: sso::SsoProviderSource,
) -> anyhow::Result<(Router, PushHandle)> {
    // Refuse the `MW_ADMIN_*` variables nothing applies, check the ones the security
    // policy overlay applies, and set the admin cookie name (t29-e6).
    webhooks::reject_legacy_inbound_secret()?;
    let config_snapshot = config_routes::BootConfigSnapshot::from_config(&config);
    let admin_overlay = admin_apply::env_overlay()?;
    let key = match &config.server_key_hex {
        Some(h) => ServerKey::from_hex(h).map_err(|_| anyhow!("MW_SERVER_KEY is not valid hex"))?,
        None => {
            let k = ServerKey::generate();
            tracing::warn!(
                "MW_SERVER_KEY not set; generated an ephemeral key (set it to persist sessions across restarts)"
            );
            k
        }
    };
    let store = Store::open(&config.db_path, key.clone()).await?;
    // Attachment upload store (t15): when `MW_UPLOAD_DIR` is configured, seal + write
    // uploaded blobs to that filesystem backend. Unset → the store keeps its default
    // fail-closed backend (an upload attempt returns an error, never a plaintext blob).
    let store = match upload_dir_from_env() {
        Some(dir) => {
            tracing::info!("upload store: filesystem backend at {}", dir.display());
            store.with_upload_backend(Arc::new(mw_store::FsUploadBackend::new(dir)))
        }
        None => store,
    };
    let render_bin = locate_render_bin();
    // DQ4/S6: is a render kernel jail expected here? On Linux (or when
    // `MW_RENDER_JAIL=require`), the hostile-input render must run in the jailed
    // child — the in-process fallbacks then fail closed rather than parse untrusted
    // input in this trusted process. On non-Linux this is the documented degraded
    // mode (process-split only) and the fallbacks are permitted.
    let render_jail_required = mw_sandbox::jail_expected();
    match &render_bin {
        Some(p) => tracing::info!(
            "render worker: {} (kernel jail {})",
            p.display(),
            if render_jail_required {
                "required — SPEC §7.5"
            } else {
                "degraded/not required on this platform"
            }
        ),
        None if render_jail_required => tracing::error!(
            "mw-render binary not found but a kernel jail is required (SPEC §7.5): \
             /api/sanitize and /api/import/oft will FAIL CLOSED (no in-process fallback). \
             Deploy the mw-render worker or set MW_RENDER_JAIL=off to run degraded."
        ),
        None => tracing::warn!(
            "mw-render binary not found; /api/sanitize will sanitize in-process (degraded, no jail)"
        ),
    }
    domain_upgrade::prepare(&store, &config.security, matches!(config.mode, ServerMode::Engine)).await?;
    let engine = match config.mode {
        ServerMode::Engine => {
            tracing::info!("engine mode: driving IMAP/POP3 accounts behind the JMAP surface");
            Some(engine_mode::build_engine(&store, &config.db_path))
        }
        ServerMode::Proxy => None,
    };
    // Realtime push: engine mode bridges the engine broadcast into the channel.
    let push = PushHandle::new();
    if let Some(engine) = &engine {
        tokio::spawn(push::bridge_engine(engine.subscribe(), push.clone()));
        nextcloud_pim_scheduler::spawn(engine, store.clone());
    }

    // ── C6 (t16 26.16): PQC-hybrid wrap of the mw-store seal key at rest ────────
    // Crypto-agility groundwork (plan §1.7/§2.4): on first boot wrap the seal master
    // key under the hybrid X25519+ML-KEM-768 primitive and persist the wrapped blob
    // (`store_key_material`). Idempotent — skipped once material exists — and
    // best-effort (a failure logs, never blocks boot, like `ensure_vapid`). This is
    // NOT a user-facing security claim (ml-kem is unaudited, plan §6#8); the fresh
    // recipient secret is groundwork only — a real deploy safeguards it in an
    // HSM/env master. Engine mode only: the helper lives on `Engine` and writes
    // through the store it wraps.
    if let Some(engine) = &engine {
        match store.get_store_key_material().await {
            Ok(Some(_)) => {} // already wrapped — nothing to do.
            Ok(None) => match engine.pqc_wrap_store_seal(&seal_key_bytes(&key)).await {
                Ok(_recipient) => tracing::info!(
                    "store seal key wrapped for crypto-agility (PQC groundwork; recipient \
                     secret ephemeral — provision an HSM/env master to enable unwrap)"
                ),
                Err(e) => tracing::warn!("PQC store-key wrap failed (non-fatal): {e}"),
            },
            Err(e) => tracing::warn!("store_key_material lookup failed (skipping PQC wrap): {e}"),
        }
    }

    // V5 (plan §2.3): ensure a VAPID keypair exists (generated on first boot, its
    // private key sealed at rest) so `GET /api/push/vapid` and the push dispatcher
    // always have a key. Non-fatal on error — the endpoints degrade to 500/skip.
    if let Err(e) = push_relay::ensure_vapid(&store).await {
        tracing::error!("VAPID keypair init failed: {e}");
    }
    // V5 push dispatcher: a SECOND consumer of the `StateChange` broadcast that
    // sends opaque wakes (WebPush/UnifiedPush/APNs — NO message content) to
    // registered subscriptions. Additive; a no-op when nothing is subscribed. It
    // drains a dedicated relay channel so it never inflates the WS/SSE subscriber
    // count the realtime path reports.
    tokio::spawn(push_relay::run_dispatcher(
        store.clone(),
        push.subscribe_relay(),
    ));

    // ── V6 MOUNT (plan §3 e11) ───────────────────────────────────────────────
    // OAuth 2.1 AS + scoped API keys, backed by the 0007 tables.
    let auth = Arc::new(mw_oauth::AuthServer::new(
        stores_v6::OAuthStoreAdapter::new(store.clone()),
    ));
    // Admin domain logic (audit log + provisioning) over the 0007 tables, with the
    // stored security policy applied to this process and re-read every 30 s.
    let (admin, applied_policy) =
        admin_apply::boot(store.clone(), engine.clone(), admin_overlay).await?;
    tokio::spawn(admin_apply::refresh_loop(admin.clone(), applied_policy));

    // Outbound webhooks: a second consumer of the StateChange broadcast, backed by
    // the sealed-secret 0007 `webhooks` table (unseal via the store key).
    let webhook_registry: Arc<dyn webhooks::WebhookRegistry> = Arc::new(
        stores_v6::WebhookRegistryAdapter::new(store.clone(), key.clone()),
    );
    // Each delivery builds its own client, pinned to the address the egress gate
    // checked for that endpoint (t32-h5); see `webhooks::deliver`.
    webhooks::spawn(webhook_registry, engine.as_ref(), push.subscribe_relay());

    // Persisted opt-in telemetry, with explicit deployment environment overrides.
    let obs = observability::ObservabilityConfig::from_store(&store).await?;
    observability::init_metrics(&obs);
    if let Some(guard) = observability::init_otlp(&obs)? {
        observability::retain_otlp(guard);
    }
    telemetry_config::record_active(obs.telemetry.clone());
    errors::set_error_config(errors::ErrorConfig::from_env());
    // A5 (t16 e10, e9 handoff): inject the engine-backed inbound-webhook action sink,
    // mirroring the outbound `WebhookRegistryAdapter` wiring above. Until this is set
    // the inbound endpoint authenticates but returns `202 {dispatched:false}`. Engine
    // mode only (the sink evaluates stored rules / applies JMAP mutations through the
    // engine); proxy mode has no engine, so inbound stays a no-op there.
    if let Some(engine) = &engine {
        webhooks::set_inbound_dispatcher(Some(Arc::new(EngineWebhookActionSink {
            engine: engine.clone(),
        })));
    }

    // Engine wiring (plan §3 e10): attach the layered cache, the zero-access
    // posture source (0007 `zeroaccess_accounts`), and the audit/webhook feed.
    // Inert in proxy mode (no engine) — the default path is byte-unchanged.
    if let Some(engine) = &engine {
        let cache = mw_cache::Cache::connect(
            mw_cache::CacheConfig {
                matrix: mw_cache::ScopeMatrix::spec_defaults(),
                redis_url: v6config.redis_url.clone(),
                memory_capacity: 10_000,
            },
            Some(store.clone()),
        )
        .await;
        let posture = stores_v6::StorePostureSource::load(&store).await;
        let feed = Arc::new(stores_v6::AdminAuditFeed::new(admin.clone()));
        engine.attach_v6(
            mw_engine::V6Hooks::new()
                .with_cache(cache)
                .with_posture_source(posture)
                .with_feed(feed),
        );
    }

    // ── V7 MOUNT (plan §3 e14) ────────────────────────────────────────────────
    // Build the five injected extensions from the 0008 admin-config rows (all
    // "off/empty" when unconfigured → the non-V7 path is byte-unchanged).
    let directory = v7_mount::build_directory(&store).await;
    // A misconfigured `MW_PASSWD_*` selection stops start-up with the error that
    // names it, before anything is bound.
    let passwd_backend = v7_mount::try_build_passwd_backend(&store)
        .map_err(|e| anyhow!("password backend configuration: {e}"))?;
    let (assist, assist_granted) = v7_mount::build_assist(&store).await;
    let plugin_host = v7_mount::build_plugin_host(&store).await;
    let nextcloud = v7_mount::build_nextcloud();

    // Engine wiring (plan §3 e8): attach the GAL directory + the Assist hook so the
    // recipient resolver + assist/JMAP surface reach them through the engine. Load
    // approved plugin/bridge account backends. Inert in proxy mode (no engine).
    if let Some(engine) = &engine {
        // Register plugin/bridge account backends FIRST (this populates the engine's
        // plugin-backing map, which `attach_v7` preserves) and collect the per-account
        // bridge PIM capability source, gated on each bridge's honest `supports-*`
        // (Graph = all six, EWS = calendar+tasks, Gmail = none). t10-e13.
        let (n, bridge_caps) = v7_mount::load_plugin_backends(engine, &plugin_host, &store).await;
        // The §10.8 spam-classification hook: the plugin host's classifier seat,
        // always handed over. It is filled by the first approved+enabled
        // `spam-action` plugin (rspamd/SpamAssassin); an empty seat answers
        // `Unknown`, which ingest treats like `Ham`.
        let spam_hook = v7_mount::build_spam_hook(&plugin_host, &store).await;

        let mut hooks = mw_engine::V7Hooks::new()
            .with_directory(directory.clone())
            .with_assist(Arc::new(v7_mount::AssistHookAdapter::from_gateway(
                &assist,
                &assist_granted,
            )));
        if let Some(caps) = bridge_caps {
            hooks = hooks.with_bridge_caps(caps);
        }
        if let Some(spam) = spam_hook {
            hooks = hooks.with_spam(spam);
        }
        engine.attach_v7(hooks);
        if n > 0 {
            tracing::info!("registered {n} plugin/bridge account backend(s)");
        }
        // The dispatcher, the search-index reconciliation and the connection of
        // stored accounts, all in the background. Here because they read the
        // posture source and the bridge registrations attached above.
        engine_mode::start_background(engine, config.security.clone());
    }

    // The `/mcp` Streamable-HTTP router over the REAL engine. The countersign
    // resolver (`mcp::StoreCountersign`) reads the admin `unattended_send` flag from
    // the 0007 `api_keys` row on each call, so an approval or a withdrawal applies
    // to the next send without a restart.
    //
    // **`/mcp` exists in engine mode only, and its absence in proxy mode is
    // deliberate** (t24-e14). This comment used to promise "a no-op mount in proxy
    // mode — tools return an engine error but `tools/list` still works"; the code
    // never did that, and could not: `build_mcp_router` takes an `Arc<Engine>` and
    // every one of the frozen ten tools is a call into it, so a proxy-mode mount
    // would have to fabricate an engine. A stub that answers `initialize` and
    // enumerates ten tools it cannot execute is a worse answer than no endpoint: an
    // MCP client discovers a tool surface, binds to it, and fails at call time. The
    // honest answer to "is MCP available here" is the transport-level 404 an absent
    // route gives — which `static_handler` now actually returns, instead of the SPA
    // shell it used to hand back with a 200 (see [`wants_spa_shell`]).
    // `apps/web/e2e/mcp.spec.ts:21` was already written to skip loudly on that 404.
    let mcp_router = engine.as_ref().map(|engine| {
        let audit = stores_v6::AdminOAuthAudit::new(admin.clone());
        mcp::build_mcp_router(engine.clone(), auth.clone(), audit)
    });

    let v6 = Arc::new(V6State {
        auth,
        admin,
        admin_enabled: v6config.admin_enabled,
        admin_username: v6config.admin_username,
        admin_password: v6config.admin_password,
        pairing: std::sync::Mutex::new(std::collections::HashMap::new()),
    });

    // t20 (26.19, e3): resolve the public-origin posture once at boot.
    let external = Arc::new(ExternalBase::from_env());
    // A deployment that declares `MW_PUBLIC_URL=https://…` has stated its public
    // scheme, so the static flag is raised to match. This is what reaches the
    // consumers that only see the flag and not the request — notably the WebAuthn
    // RP origin (`twofa_routes::derive_rp`), which previously took its scheme from
    // the cookie setting and therefore claimed `http://` on every TLS deployment
    // that had not set `MW_COOKIE_SECURE`. Only ever raises, never lowers.
    //
    // MIGRATION NOTE (t20 R7): setting `MW_PUBLIC_URL` to an https base changes the
    // WebAuthn RP origin from `http://host` to `https://host`. Passkeys registered
    // under the old origin stop validating. `MW_PUBLIC_URL` is new, so this can only
    // fire when an operator opts in; `MW_WEBAUTHN_ORIGIN` still overrides both.
    let cookie_secure = config.cookie_secure || external.declares_https();
    // t20 (26.19, e13): the sub-path prefix, resolved once here so nothing on the
    // request path re-reads the environment.
    let base_path = base_path_from_env();
    if !base_path.is_empty() {
        tracing::info!("serving under sub-path {base_path} (MW_BASE_PATH)");
    }
    let request_protection = request_protection::Runtime::start(store.clone());
    let state = AppState {
        started_at: std::time::SystemTime::now(),
        started: std::time::Instant::now(),
        config_snapshot,
        request_history: request_history::Recorder::start(store.clone()),
        store,
        scheduler: jobs::new_scheduler(),
        render_bin,
        render_jail_required,
        web_dir: config.web_dir,
        cookie_secure,
        external,
        base_path: Arc::from(base_path.as_str()),
        engine,
        push: push.clone(),
        sessions: Arc::new(SessionGuard::new()),
        signin_cooldown: Arc::new(signin_cooldown::Cooldown::default()),
        request_protection,
        signin_challenges: Arc::new(signin_challenge::IssuanceLimiter::default()),
        hardening: config.hardening,
        security: config.security,
        native_auth: NativeAuthConfig::from_env(),
        v6,
        twofa: Arc::new(twofa_routes::TwofaState::new()),
        step_up: Arc::new(step_up::StepUpState::new()),
        analytics: Arc::new(analytics_runtime::Runtime::default()),
        directory: directory.clone(),
    };
    jobs::start(state.clone());
    let v7 = V7Extensions {
        directory,
        passwd: passwd_backend,
        assist,
        plugins: plugin_host,
        nextcloud,
    };
    // V8 SSO (t9-e3): the shared one-shot pending-flow store (replay/CSRF binding)
    // plus the provider source (store-built in prod; mock in the unit gate).
    let sso_pending = Arc::new(sso::PendingFlows::new(sso::PENDING_TTL));
    Ok((router(state, mcp_router, v7, sso_source, sso_pending), push))
}

/// The five V7 request extensions e14 injects into the mounted route factories
/// (plan §3 e9/e14). Built from the 0008 admin-config rows in [`build_app_full`].
pub(crate) struct V7Extensions {
    directory: directory::DirectoryHandle,
    passwd: passwd::PasswdBackend,
    assist: assist::AssistHandle,
    plugins: plugins::PluginRegistry,
    nextcloud: nextcloud::NextcloudHandle,
}

fn router(
    state: AppState,
    mcp_router: Option<Router>,
    v7: V7Extensions,
    sso_source: sso::SsoProviderSource,
    sso_pending: Arc<sso::PendingFlows>,
) -> Router {
    // V6 additive surfaces (plan §3 e11), merged before the fallback + guard layers
    // so they ride the same security-headers / CSRF-origin / CORS middleware. The
    // normal mailbox routes above are byte-unchanged.
    // Scoped-API-key enforcement (t6-e11b): a `mwk_…` key on `/api/v1/*` is resolved
    // to its `Scope` and enforced (scope + IP + rate-limit + expiry); no key present
    // → the cookie/native path passes through unchanged. `route_layer` scopes the
    // guard to the REST routes only (admin/oauth keep their own auth).
    let rest = rest::rest_router().route_layer(middleware::from_fn_with_state(
        state.clone(),
        scope_mw::rest_scope_guard,
    ));

    // t32-e24b: the admin routes registered in `admin_router` that need a V7 extension (the
    // plugin registry, the Assist gateway) get the same handles.
    let admin_routes = admin_router::router(&state)
        .layer(Extension(v7.plugins.clone()))
        .layer(Extension(v7.assist.clone()));

    // ── V7 MOUNT (plan §3 e14) ────────────────────────────────────────────────
    // The five e9 route factories + the extra e14 endpoints, each layered with the
    // request extension it needs (`Extension<X>` — mirrors e9's mount contract). A
    // missing extension ⇒ the handler 500s, so every one is injected here.
    let v7_routes = directory::directory_router()
        .merge(passwd::passwd_router())
        .merge(assist::assist_router())
        .merge(nextcloud::nextcloud_router())
        .merge(v7_mount::extra_v7_router())
        .layer(Extension(v7.directory))
        .layer(Extension(v7.passwd))
        .layer(Extension(v7.assist))
        .layer(Extension(v7.plugins))
        .layer(Extension(v7.nextcloud));

    // ── V8 SSO MOUNT (t9-e3) ──────────────────────────────────────────────────
    // The `/api/sso/*` login routes + the `/admin/sso` config CRUD, layered with the
    // injected provider source + shared pending-flow store. Additive; the password
    // `/api/login` path is untouched.
    let sso_routes = sso::sso_router()
        .layer(Extension(sso_source))
        .layer(Extension(sso_pending));

    // ── t10 (26.10) MOUNT (plan §3 e13) ───────────────────────────────────────
    // OAuth DCR (RFC 7591) — additive, admin/policy-gated, stays 403 until an admin
    // writes an `enabled` `oauth_dcr` policy row (never on by default). The UI-plugin
    // registry (`/admin/ui-plugins/*` + `/api/ui-plugins/*`) and masked-email
    // (`/api/masked/*`) surfaces are fail-soft: with nothing approved/created they
    // return empty, so the mailbox path is byte-unchanged.
    let t10_routes = oauth::dcr_router()
        .merge(ui_plugins::ui_plugins_router())
        .merge(masked::masked_router());

    // t32-e24a: every admin route registered with a capability.
    let mut v6 = admin_routes
        .merge(oauth::routes())
        .merge(oidc_provider::routes())
        .merge(rest)
        .merge(v7_routes)
        .merge(sso_routes)
        .merge(t10_routes)
        .route("/metrics", get(observability::metrics))
        .route("/errors", post(errors::report_error))
        .merge(webhooks::router());
    if let Some(mcp) = mcp_router {
        // The MCP router is a self-contained `Router<()>` (its own `Arc<McpServer>`
        // state); mount it as a service so it need not share `AppState`. The e11b
        // guard in front adds IP-allowlist + per-key rate-limit + expiry for a
        // presented key (the per-tool scope/countersign check stays inline).
        let mcp = mcp.route_layer(middleware::from_fn_with_state(
            state.clone(),
            scope_mw::mcp_scope_guard,
        ));
        v6 = v6.nest_service("/mcp", mcp);
    }

    let app = Router::new()
        .route("/api/public-config", get(client_config::public_config))
        .route("/api/signin/challenge", get(signin_challenge::issue))
        .merge(v6)
        // t16 (26.16, e3): 2FA login-step / enrolment / recovery + session management.
        // `/api/login/2fa*` are pre-auth (pending-token) and CSRF-exempt like
        // `/api/login`; the `/api/account/*` routes are session-authed.
        .merge(twofa_routes::twofa_router())
        .merge(step_up::user_router())
        .merge(security_center::router())
        // t16 (26.16, e10 chain link 3): the additive route modules e6/e18/e10 deliver.
        // All session-authed and ride the same CSRF/security-headers middleware below.
        //   * image_proxy (e6, S7/S8): the anonymizing `/api/image-proxy` + the
        //     remote-image display grants. The proxy fetch is additionally SSRF-gated.
        //   * prefs_routes (e18, W12/W13/W15): `/api/account/*` signatures / notifications
        //     / saved-searches / identities persistence.
        //   * import_routes (e10, W18): mbox/EML/Maildir import + the webcal sync driver
        //     (its GET reuses e6's SSRF-safe fetcher).
        .merge(image_proxy::image_proxy_router())
        .merge(prefs_routes::prefs_router())
        .merge(features::user_router())
        .merge(client_policy::router())
        .merge(import_routes::import_router())
        .merge(account_drafts::router())
        .merge(sieve_sync::sieve_sync_router())
        .merge(open_tracking::router())
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/me", get(me))
        .route("/api/session/rotate", post(rotate_session))
        // Unauthenticated + CSRF-exempt + attacker-chosen domain: the SSRF gate
        // bounds where discovery may fetch, this bounds how often (t22-e9).
        .route(
            "/api/discover",
            post(discover).layer(axum::middleware::from_fn(
                scope_mw::discover_ratelimit::guard,
            )),
        )
        .route("/api/sanitize", post(sanitize))
        .route("/api/import/oft", post(import_oft))
        .merge(attachment_preview::router())
        .route("/api/attachments/policy", get(attachment_policy::get))
        .merge(delivery_status::router())
        .merge(branding_routes::router())
        .merge(analytics_routes::router())
        .merge(appearance_customization::router())
        .merge(nextcloud_accounts::router())
        .merge(nextcloud_pim::router())
        .merge(packages::router())
        .route("/jmap/session", get(jmap_session))
        .route("/jmap/api", post(jmap_api).route_layer(middleware::from_fn_with_state(state.clone(), scope_mw::jmap_sieve_scope_guard)))
        .route(
            "/jmap/download/{accountId}/{blobId}/{name}",
            get(jmap_download),
        )
        .route("/jmap/upload/{accountId}", post(jmap_upload))
        .route("/api/export/{stableId}", get(export_message))
        // ── V3 PIM endpoints: Mailwoman-native calendar/address-book sharing
        // (ACL-checked serving of a collection to another principal) + the
        // holiday-feed. Handlers are live. ──
        .route("/dav/calendars/{accountId}/{calendarId}", get(caldav_share))
        .route(
            "/dav/addressbooks/{accountId}/{addressBookId}",
            get(carddav_share),
        )
        .route("/api/holidays", get(holiday_regions))
        .route("/api/holidays/{region}", get(holiday_feed))
        // ── V4 crypto/security endpoints: WKD publishing (serve own public
        // keys), ARF report submission (abuse-address relay), and DLP config
        // load. Handlers are live. ──
        .route("/.well-known/openpgpkey/hu/{hash}", get(wkd_lookup))
        .route("/.well-known/openpgpkey/policy", get(wkd_policy))
        .route(
            "/.well-known/openpgpkey/{domain}/hu/{hash}",
            get(wkd_lookup_advanced),
        )
        .route("/.well-known/openpgpkey/{domain}/policy", get(wkd_policy))
        .route("/api/security/report", post(arf_report))
        .route("/api/security/dlp/config", get(dlp_config))
        .route("/api/security/watermark", get(watermark_config))
        .route("/jmap/ws", get(push::jmap_ws))
        .route("/jmap/eventsource", get(push::jmap_eventsource))
        // ── V5 push relay: VAPID key serving, subscription storage, and the
        // opaque-wake dispatcher. Handlers are live, cookie-authed like every
        // other endpoint (never falls through to the SPA index.html). The
        // additive native bearer-auth mode + CORS gate are OFF by default
        // (browser cookie/same-origin path UNCHANGED). ──
        .route("/api/push/vapid", get(push_relay::push_vapid))
        .route("/api/push/permission", get(push_relay::push_permission))
        .route("/api/push/subscribe", post(push_relay::push_subscribe))
        .route("/api/push/unsubscribe", post(push_relay::push_unsubscribe))
        .route("/healthz", get(|| async { "ok" }))
        // t32-h10: a mounted path with the wrong method answers `{error, code}`
        // like every other error; axum still adds the `Allow` header.
        .method_not_allowed_fallback(|| async {
            error_pages::json(error_pages::ErrorKind::MethodNotAllowed)
        })
        .fallback(static_handler)
        // Innermost: reject cross-origin / missing-CSRF writes before handlers.
        .layer(middleware::from_fn_with_state(
            state.clone(),
            state_change_guard,
        ))
        .layer(middleware::from_fn_with_state(state.clone(), signin_geography::session_guard))
        .layer(middleware::from_fn_with_state(state.clone(), source_bans::guard))
        .layer(middleware::from_fn_with_state(state.clone(), request_protection::guard))
        // V5 (plan §2.2; t31 D10): CORS/preflight for the native shell origins
        // (the Tauri origins plus `MW_NATIVE_ORIGINS`). Any other origin passes
        // through untouched, so the browser path emits no `Access-Control-*` headers.
        .layer(middleware::from_fn_with_state(state.clone(), native_cors))
        // t32-h10: a handler panic becomes a plain 500 (inside the security
        // headers) instead of a dropped connection; its message is only logged.
        .layer(middleware::from_fn_with_state(
            state.base_path.clone(),
            error_pages::catch_panic,
        ))
        // Outermost: security headers on every response (incl. guard rejections).
        .layer(middleware::from_fn_with_state(
            state.clone(),
            security_headers,
        ))
        .layer(middleware::from_fn_with_state(state.clone(), http_metrics::layer))
        .with_state(state.clone());
    // t20 B4: also serve the whole thing under `MW_BASE_PATH`. No-op when unset.
    mount_under_base_path(app, &state.base_path)
}

// ---------------------------------------------------------------------------
// A5 — engine-backed inbound-webhook action sink (t16 e10, e9 handoff)
// ---------------------------------------------------------------------------

/// Drives an authenticated inbound webhook's mapped action against the real engine
/// (the parallel of the outbound [`stores_v6::WebhookRegistryAdapter`]). Installed at
/// boot in engine mode via [`webhooks::set_inbound_dispatcher`].
///
///   * [`InboundAction::RunRules`] re-evaluates the account's stored GUI/Sieve rules
///     ([`Engine::get_rules`] + [`mw_sieve::evaluate_all`]) against the supplied
///     envelope and reports how many actions fired. It does not mutate a stored
///     message — a `run_rules` webhook carries an external envelope, not a message id.
///   * [`InboundAction::Sieve`] applies one Sieve action to a target `message_id`
///     through [`Engine::handle_jmap`] (`Email/set`): `Tag`/`Mark` add a keyword;
///     `Move` resolves the destination mailbox by name/role (via `Mailbox/get`) and
///     re-homes the message. Actions with no local single-message effect
///     (`Copy`/`Forward`/`ReplyTemplate`/`Notify`/`Stop`) are recognized as a no-op
///     (`fired: 0`), mirroring the engine's own ingest-time rule application.
struct EngineWebhookActionSink {
    engine: Arc<Engine>,
}

#[async_trait::async_trait]
impl webhooks::WebhookActionSink for EngineWebhookActionSink {
    async fn fire(
        &self,
        account: &str,
        action: &webhooks::InboundAction,
    ) -> Result<webhooks::WebhookDispatch, String> {
        match action {
            webhooks::InboundAction::RunRules { envelope } => {
                let rules = self
                    .engine
                    .get_rules(account)
                    .await
                    .map_err(|e| e.to_string())?;
                let parsed = mw_sieve::ParsedEnvelope {
                    from: envelope.from.clone(),
                    to: envelope.to.clone(),
                    subject: envelope.subject.clone(),
                    // The inbound envelope is header-shaped only (§21.1) — no body /
                    // size / keyword facts are carried, so body-based tests simply
                    // don't match, which is the honest evaluation of what we know.
                    ..Default::default()
                };
                let fired = mw_sieve::evaluate_all(&rules, &parsed).len() as u64;
                Ok(webhooks::WebhookDispatch { fired })
            }
            webhooks::InboundAction::Sieve { action, message_id } => {
                let message_id = message_id
                    .as_deref()
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| "sieve webhook action requires a messageId".to_string())?;
                let action: mw_sieve::Action = serde_json::from_value(action.clone())
                    .map_err(|e| format!("unrecognized sieve action: {e}"))?;
                self.apply_sieve_action(account, message_id, &action).await
            }
        }
    }
}

impl EngineWebhookActionSink {
    /// Apply one Sieve [`Action`](mw_sieve::Action) to `message_id` via the engine's
    /// JMAP surface. Returns the fired count (`1` when a mutation was issued).
    async fn apply_sieve_action(
        &self,
        account: &str,
        message_id: &str,
        action: &mw_sieve::Action,
    ) -> Result<webhooks::WebhookDispatch, String> {
        let patch = match action {
            mw_sieve::Action::Tag { keyword } | mw_sieve::Action::Mark { keyword } => {
                json!({ format!("keywords/{keyword}"): true })
            }
            mw_sieve::Action::Move { mailbox } => {
                let Some(dest) = self.resolve_mailbox(account, mailbox).await? else {
                    return Err(format!("no mailbox matches '{mailbox}'"));
                };
                json!({ "mailboxIds": { dest: true } })
            }
            // No single-message local effect (mirrors the ingest-time rule handler).
            _ => return Ok(webhooks::WebhookDispatch { fired: 0 }),
        };
        let request = json!({
            "methodCalls": [[
                "Email/set",
                { "accountId": account, "update": { message_id: patch } },
                "wh0"
            ]]
        });
        let resp = self.engine.handle_jmap(account, &request).await;
        if jmap_updated(&resp, message_id) {
            Ok(webhooks::WebhookDispatch { fired: 1 })
        } else {
            Err(format!(
                "engine did not apply the action to {message_id}: {}",
                jmap_first_error(&resp).unwrap_or_else(|| "unknown".into())
            ))
        }
    }

    /// Resolve a mailbox name or role to its id via `Mailbox/get`.
    async fn resolve_mailbox(&self, account: &str, target: &str) -> Result<Option<String>, String> {
        let request = json!({
            "methodCalls": [["Mailbox/get", { "accountId": account, "ids": null }, "mb0"]]
        });
        let resp = self.engine.handle_jmap(account, &request).await;
        let list = resp
            .get("methodResponses")
            .and_then(Value::as_array)
            .and_then(|r| r.first())
            .and_then(|c| c.as_array())
            .and_then(|c| c.get(1))
            .and_then(|a| a.get("list"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        Ok(list
            .iter()
            .find(|m| {
                let by_name = m.get("name").and_then(Value::as_str);
                let by_role = m.get("role").and_then(Value::as_str);
                by_name.is_some_and(|n| n.eq_ignore_ascii_case(target))
                    || by_role.is_some_and(|r| r.eq_ignore_ascii_case(target))
            })
            .and_then(|m| m.get("id").and_then(Value::as_str))
            .map(str::to_string))
    }
}

/// Whether a JMAP response's first `Email/set` marked `id` as updated.
fn jmap_updated(resp: &Value, id: &str) -> bool {
    resp.get("methodResponses")
        .and_then(Value::as_array)
        .and_then(|r| r.first())
        .and_then(|c| c.as_array())
        .and_then(|c| c.get(1))
        .and_then(|a| a.get("updated"))
        .and_then(|u| u.get(id))
        .is_some()
}

/// The first `notUpdated`/`type` error description in a JMAP response, if any.
fn jmap_first_error(resp: &Value) -> Option<String> {
    let call = resp
        .get("methodResponses")
        .and_then(Value::as_array)?
        .first()?
        .as_array()?
        .get(1)?;
    if let Some(err) = call.get("type").and_then(Value::as_str) {
        return Some(err.to_string());
    }
    let not_updated = call.get("notUpdated")?.as_object()?;
    not_updated
        .values()
        .next()
        .and_then(|v| v.get("description").or_else(|| v.get("type")))
        .and_then(Value::as_str)
        .map(str::to_string)
}

// ---------------------------------------------------------------------------
// Security headers (SPEC §7.4)
// ---------------------------------------------------------------------------

async fn security_headers(State(state): State<AppState>, req: Request, next: Next) -> Response {
    // t20 (26.19, e3): decide the transport-security posture from the REQUEST,
    // before it is consumed, then apply it to the response below.
    let hsts = state
        .external
        .hsts(req.headers(), req.extensions())
        .cloned();
    let cookie_secure =
        state
            .external
            .cookie_secure(state.cookie_secure, req.headers(), req.extensions());

    let mut resp = next.run(req).await;
    // Only fixed public reader/appearance sandboxes may supply an independent policy.
    // A response extension cannot be forged by a request header or upstream.
    let reader_frame = resp.extensions_mut().remove::<remote_policy::ReaderFrameResponse>().is_some();
    let appearance_frame = resp.extensions_mut().remove::<appearance_customization::AppearanceSandboxResponse>().is_some();
    let isolated_frame = reader_frame || appearance_frame;
    let h = resp.headers_mut();
    // A handler's own policy (a download's sandbox, t32-h4) is kept as a second
    // header next to the shell's. The browser enforces both, so it can only
    // narrow what the shell policy allows.
    let own = h.remove("content-security-policy");
    if let (true, Some(policy)) = (isolated_frame, own.as_ref()) {
        h.insert("content-security-policy", policy.clone());
    } else {
        h.insert("content-security-policy", HeaderValue::from_static(CSP));
        if let Some(v) = own {
            h.append("content-security-policy", v);
        }
    }
    h.insert("x-frame-options", HeaderValue::from_static(if isolated_frame { "SAMEORIGIN" } else { "DENY" }));
    h.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    h.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    h.insert(
        "cross-origin-opener-policy",
        HeaderValue::from_static("same-origin"),
    );
    // Additive §7.4 deltas: COEP/CORP/Permissions-Policy. A handler that sets its
    // own `Cross-Origin-Resource-Policy` keeps it: the open-tracking image is loaded
    // by mail programs on other origins, and `same-origin` would make a browser
    // refuse it (t29-e8).
    let own_corp = h.remove("cross-origin-resource-policy");
    hardening::apply_extra_headers(h, state.hardening.coep);
    if isolated_frame {
        // Sandboxing gives the bridge an opaque origin in Firefox. It is a
        // public static document; frame-ancestors and its parent handshake
        // still restrict embedding and the delivery of message HTML.
        h.insert("cross-origin-resource-policy", HeaderValue::from_static("cross-origin"));
        h.insert("cross-origin-embedder-policy", HeaderValue::from_static("credentialless"));
    } else if let Some(v) = own_corp {
        h.insert("cross-origin-resource-policy", v);
    }
    // t20 B6: tell the browser never to speak plaintext to this host again, so a
    // later http navigation cannot be intercepted before the redirect. Emitted
    // only when this request itself was https (see `ExternalBase::hsts`).
    if let Some(v) = hsts {
        h.insert("strict-transport-security", v);
    }
    if cookie_secure {
        mark_cookies_secure(h);
    }
    // t20 B4: re-scope cookies to the deploy prefix. Same reasoning as
    // `mark_cookies_secure` — doing it once, outermost, covers the cookies minted
    // in modules that never see this decision (`admin`, `sso`, `twofa_routes`).
    scope_cookies_to_base(h, &state.base_path);
    resp
}

/// Re-scope every `Set-Cookie` on the response from `Path=/` to the deploy prefix
/// (t20 B4). No-op at the origin root.
///
/// Under `/mail`, a `Path=/` cookie still *works* — it is simply sent to every
/// path on the origin, including a co-hosted app at `/other`. Narrowing it to
/// `Path=/mail` is the scoping the sub-path deployment shape implies. RFC 6265
/// path-matching means `Path=/mail` covers `/mail` and `/mail/…` but not
/// `/mailbox`, so no separate trailing-slash variant is needed.
///
/// Only an explicit `Path=/` is rewritten. Every cookie this server mints sets
/// one; a hypothetical future cookie that omits `Path` would fall back to the
/// browser's default (the request's directory) and is deliberately left alone
/// rather than guessed at here.
fn scope_cookies_to_base(h: &mut HeaderMap, base: &str) {
    if base.is_empty() {
        return;
    }
    let cookies: Vec<HeaderValue> = h.get_all(header::SET_COOKIE).iter().cloned().collect();
    if cookies.is_empty() {
        return;
    }
    let mut changed = false;
    let rescoped: Vec<HeaderValue> = cookies
        .into_iter()
        .map(|v| {
            let Ok(s) = v.to_str() else { return v };
            let mut hit = false;
            let rewritten = s
                .split(';')
                .map(|attr| {
                    let t = attr.trim();
                    match t.split_once('=') {
                        Some((name, value))
                            if name.trim().eq_ignore_ascii_case("path") && value.trim() == "/" =>
                        {
                            hit = true;
                            format!("Path={base}")
                        }
                        _ => t.to_string(),
                    }
                })
                .collect::<Vec<_>>()
                .join("; ");
            if !hit {
                return v;
            }
            match HeaderValue::from_str(&rewritten) {
                Ok(new) => {
                    changed = true;
                    new
                }
                Err(_) => v,
            }
        })
        .collect();
    if !changed {
        return;
    }
    h.remove(header::SET_COOKIE);
    for v in rescoped {
        h.append(header::SET_COOKIE, v);
    }
}

/// Add `Secure` to every `Set-Cookie` on the response that lacks it.
///
/// t20 B6: `Secure` used to be decided at each cookie-minting call site from a
/// static flag that defaulted to `false`, so a deployment terminating TLS at a
/// proxy and not setting `MW_COOKIE_SECURE` handed out session cookies the
/// browser would happily replay over plaintext. Doing it once here, outermost,
/// covers every cookie the server sets — including the ones minted in modules
/// that never see this decision (`sso`, `twofa_routes`, `admin`) — and cannot be
/// forgotten by a future call site.
///
/// Only ever adds the attribute; a cookie that already carries it is untouched.
fn mark_cookies_secure(h: &mut HeaderMap) {
    let cookies: Vec<HeaderValue> = h.get_all(header::SET_COOKIE).iter().cloned().collect();
    if cookies.is_empty() {
        return;
    }
    let mut changed = false;
    let upgraded: Vec<HeaderValue> = cookies
        .into_iter()
        .map(|v| {
            let Ok(s) = v.to_str() else { return v };
            if s.split(';')
                .any(|a| a.trim().eq_ignore_ascii_case("Secure"))
            {
                return v;
            }
            match HeaderValue::from_str(&format!("{s}; Secure")) {
                Ok(new) => {
                    changed = true;
                    new
                }
                Err(_) => v,
            }
        })
        .collect();
    if !changed {
        return;
    }
    h.remove(header::SET_COOKIE);
    for v in upgraded {
        h.append(header::SET_COOKIE, v);
    }
}

/// Reject state-changing requests that fail the Origin/Referer same-site check
/// (always on — effective CSRF defense needing no client change) and, when
/// `csrf_strict` is enabled, the double-submit token check.
async fn state_change_guard(State(state): State<AppState>, req: Request, next: Next) -> Response {
    if hardening::is_state_changing(req.method()) {
        // V8 SSO (t9-e3, plan §9 R3): the OIDC callback / SAML ACS are
        // IdP-redirect/POST landings that carry IdP-signed state (bound one-shot,
        // server-side, in the SSO pending-flow store) — NOT an ambient session — so
        // the cookie-CSRF + same-origin guard does not apply to them (a cross-site
        // form POST from the IdP legitimately has a foreign/absent Origin). Exempt
        // EXACTLY these paths; every other route (incl. `/api/sso/logout`) stays
        // guarded.
        if is_sso_idp_callback(req.uri().path()) || matches!(req.uri().path(), "/oidc/token" | "/oidc/revoke") {
            return next.run(req).await;
        }
        // V5 (plan §2.2): a request whose only authority is its bearer token (the
        // native shells: no cookie, any origin) is not a CSRF vector, so the
        // Origin/double-submit guard is skipped for it. A bearer header beside a
        // session cookie does not qualify: a handler may authorise such a request by
        // the cookie — the admin gates never read `Authorization` — so it is
        // checked like every other cookie request (26.20; before that any bearer
        // header switched both checks off).
        let bearer_only = push_relay::bearer_token(req.headers()).is_some()
            && !hardening::has_auth_cookie(req.headers());
        // t31 (D10): a native shell signing in. The webview sends its fixed Tauri
        // `Origin` (`push_relay::TAURI_ORIGINS`, plus `MW_NATIVE_ORIGINS`) and, before
        // sign-in, has no bearer token, so the Origin check below would refuse it.
        // Excused only on the sign-in routes and only with no session cookie, so a
        // request a cookie authorises is checked exactly as before. A web page
        // cannot send a Tauri origin (a browser sets `Origin` from the page's own
        // scheme and host), so this opens nothing to cross-site requests.
        let native_signin = hardening::native_signin_route(req.uri().path())
            && !hardening::has_auth_cookie(req.headers())
            && req
                .headers()
                .get(header::ORIGIN)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|o| state.native_auth.allows(o));
        if !bearer_only && !native_signin {
            // The URI authority is where HTTP/2 puts `:authority`; hyper does not
            // synthesise a `Host` header for it, and `origin_ok` now refuses a
            // request with neither rather than waving it through (t20 B8).
            let authority = req
                .uri()
                .authority()
                .map(|a| a.as_str())
                .map(str::to_string);
            if !hardening::origin_ok(req.headers(), authority.as_deref()) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "cross-origin request rejected" })),
                )
                    .into_response();
            }
            // Strict mode: a write must repeat the `mw_csrf` cookie in
            // `X-CSRF-Token`. Sign-in routes (no token exists yet) and routes no
            // cookie authorises are not asked for it; the list, with the reason
            // for each entry, is `hardening::csrf_token_exempt`. They have passed
            // the Origin check above like every other write.
            if state.hardening.csrf_strict
                && !hardening::csrf_token_exempt(req.uri().path())
                && !hardening::csrf_request_ok(
                    req.headers(),
                    &state.store,
                    req.uri().path().strip_prefix(&mw_admin::config::admin_path())
                        .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('/')),
                )
            {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "missing or invalid CSRF token" })),
                )
                    .into_response();
            }
        }
    }
    next.run(req).await
}

/// Whether `path` is an SSO IdP-callback landing (`/api/sso/{id}/callback` or
/// `/api/sso/{id}/acs`) — the ONLY paths exempted from the state-change/CSRF guard
/// (they carry IdP-signed state bound server-side, not an ambient session). The
/// session-authed `/api/sso/logout` and every other route stay guarded.
fn is_sso_idp_callback(path: &str) -> bool {
    let Some(rest) = path.strip_prefix("/api/sso/") else {
        return false;
    };
    let mut parts = rest.splitn(2, '/');
    let id = parts.next().unwrap_or("");
    let tail = parts.next().unwrap_or("");
    !id.is_empty() && matches!(tail, "callback" | "acs")
}

/// CORS for the native shells (plan §2.2; t31 D10). A request whose `Origin` is a
/// native shell origin (`push_relay::TAURI_ORIGINS`, plus `MW_NATIVE_ORIGINS`) gets
/// that origin echoed back, and its preflight (`OPTIONS` with
/// `Access-Control-Request-Method`) is answered here. Every other request passes
/// through untouched: a browser origin never gets an `Access-Control-*` header, and
/// a plain `OPTIONS` (CalDAV/CardDAV discovery) reaches the router.
///
/// No `Access-Control-Allow-Credentials`: the shells authenticate with a bearer
/// header and send no cookies, so a cross-origin request can never carry a session
/// cookie with the browser's consent.
async fn native_cors(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let origin = req
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .filter(|o| state.native_auth.allows(o))
        .and_then(|o| HeaderValue::from_str(o).ok());
    let Some(origin) = origin else {
        return next.run(req).await;
    };
    if req.method() == axum::http::Method::OPTIONS
        && req
            .headers()
            .contains_key(header::ACCESS_CONTROL_REQUEST_METHOD)
    {
        let mut resp = StatusCode::NO_CONTENT.into_response();
        add_cors_headers(resp.headers_mut(), origin);
        return resp;
    }
    let mut resp = next.run(req).await;
    add_cors_headers(resp.headers_mut(), origin);
    resp
}

/// Emit the `Access-Control-*` headers for an allowed native shell origin.
fn add_cors_headers(h: &mut HeaderMap, origin: HeaderValue) {
    h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
    h.append(header::VARY, HeaderValue::from_static("Origin"));
    h.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, PUT, PATCH, DELETE, OPTIONS"),
    );
    // `x-mailwoman-client`: the shells send their kind, version and updater state
    // on every request (t32-cfg §4.18); a preflight listing it must be allowed.
    h.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("authorization, content-type, x-csrf-token, x-mailwoman-client, x-mailwoman-account, x-mailwoman-storage-authority, if-match"),
    );
    h.insert(
        header::ACCESS_CONTROL_EXPOSE_HEADERS,
        HeaderValue::from_static("etag"),
    );
    h.insert(
        header::ACCESS_CONTROL_MAX_AGE,
        HeaderValue::from_static("600"),
    );
}

// ---------------------------------------------------------------------------
// Auth endpoints
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginReq {
    username: String,
    password: String,
    #[serde(default)]
    altcha: Option<String>,
    /// V5 native-auth mode (plan §2.2): a native shell sends `"native"` to get a
    /// bearer-token session (a `native_sessions` row, NO cookie) in the response
    /// body instead of the cookie. ADDITIVE — absent for browser logins, so the
    /// cookie/same-origin path is byte-identical.
    #[serde(default)]
    client_type: Option<String>,
    /// Every other field, kept only to refuse the ones that name a server.
    #[serde(flatten)]
    other: std::collections::HashMap<String, serde_json::Value>,
}

/// Login-body fields through which a client could choose the mail server. The
/// server decides the upstream (t32-h9); a body carrying any of them, even
/// empty, is refused rather than having the field ignored.
const CLIENT_SERVER_FIELDS: &[&str] = &[
    "jmapUrl",
    "jmap_url",
    "server",
    "serverUrl",
    "host",
    "port",
    "imapUrl",
    "pop3Url",
    "sessionUrl",
    "upstream",
    "url",
];

impl LoginReq {
    fn names_a_server(&self) -> bool {
        CLIENT_SERVER_FIELDS
            .iter()
            .any(|f| self.other.contains_key(*f))
    }
}

/// 400 for a login body that names a server (t32-h9).
fn client_server_refused() -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": "the server chooses the mail server; a sign-in sends only the address and the password",
            "code": "clientServerNotAllowed",
        })),
    )
        .into_response()
}

/// 404 for an address no configured upstream places (t32-h9). It says nothing
/// about the account: it is decided before any credential is checked.
fn address_not_configured() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": "this address is not set up on this server",
            "code": "addressNotConfigured",
        })),
    )
        .into_response()
}

/// Uniform 401 — never leak which of URL/user/password was wrong (SPEC §7.4).
fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "invalid credentials" })),
    )
        .into_response()
}

async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    // t20 B2: the header-auth path needs the connected peer's address, which
    // lives in the request extensions as `ConnectInfo`.
    ext: axum::http::Extensions,
    Json(mut body): Json<LoginReq>,
) -> Response {
    if body.names_a_server() {
        return client_server_refused();
    }
    if let Err(error)=signin_guard::check(&state,mw_store::AuthSurface::Webmail,scope_mw::proxy::client_ip(&headers,&ext)).await { return error.into_response(); }
    if let Some(response) = signin_cooldown::check(&state, &headers, &ext).await {
        return response;
    }
    if let Some(response) = signin_challenge::verify(&state, signin_challenge::Surface::Mailbox, &headers, &ext, body.altcha.as_deref()).await {
        return response;
    }
    // A2 (header-auth trusted proxy): if the deployment trusts an upstream that has
    // already authenticated the user and asserts it via `X-Remote-User`, mint the
    // session from that header without a password. OFF unless configured, and
    // restricted to `MW_HEADER_AUTH_TRUSTED_IPS` peers.
    if let Some(resp) = header_auth_login(&state, &headers, &ext, &body).await {
        return resp;
    }
    let host = state.external.host(&headers, &ext);
    match domain_config::canonical(&state.store, &body.username, host.as_deref()).await {
        Ok(identity) => body.username = identity.login_name,
        Err(_) => {
            security_center::record_failure(&state,None,&body.username,&security_center::device(&headers,&ext,body.client_type.as_deref()),"password","not-configured").await;
            return address_not_configured();
        },
    }
    // A1 (LDAP-bind login backend, §18.3): an env-gated pre-check against the
    // directory. Off by default → the mail backend remains the sole authority.
    if let Err(reason) = directory_bind_gate(&state, &body.username, &body.password).await {
        security_center::record_failure_backend(&state,None,&body.username,&security_center::device(&headers,&ext,body.client_type.as_deref()),"password",reason,Some("ldap")).await;
        return unauthorized();
    }
    let upstream = match upstream_resolve::resolve_upstream(&state, &body.username).await {
        Ok(u) => u,
        Err(upstream_resolve::ResolveError::NotConfigured) => {
            security_center::record_failure(&state,None,&body.username,&security_center::device(&headers,&ext,body.client_type.as_deref()),"password","not-configured").await;
            return address_not_configured();
        },
    };
    if let Some(engine) = &state.engine {
        return engine_login(&state, &headers, &ext, engine, body, upstream.url()).await;
    }
    let jmap_url = upstream.url().to_string();
    let creds = Credentials {
        username: body.username.clone(),
        password: body.password.clone(),
    };
    // Validate credentials by fetching the upstream Session server-side — under the
    // upstream policy, so a refused upstream is never contacted (t24 B2). A refusal
    // answers with the same 401 as a wrong password.
    let attempt = tokio::time::timeout(std::time::Duration::from_secs(25), upstream_session(&state.security, &jmap_url, &creds)).await;
    let attempt = match attempt {
        Ok(result) => result,
        Err(_) => {
            security_center::record_failure_backend(&state,None,&body.username,
                &security_center::device(&headers,&ext,body.client_type.as_deref()),"password","timeout",Some("jmap")).await;
            return StatusCode::GATEWAY_TIMEOUT.into_response();
        }
    };
    let session = match attempt {
        Ok(s) => s,
        Err(e) => {
            if let UpstreamSessionError::Refused(r) = &e {
                let _ = r;
                tracing::warn!("proxy login refused by upstream policy");
            }
            let reason = match &e {
                UpstreamSessionError::Refused(_) => "upstream-policy",
                UpstreamSessionError::Jmap(mw_jmap::JmapError::Status(status)) if status.as_u16()==401 => "auth-rejected",
                UpstreamSessionError::Jmap(mw_jmap::JmapError::Status(status)) if status.as_u16()==403 => "upstream-policy",
                UpstreamSessionError::Jmap(mw_jmap::JmapError::Http(error)) if error.is_timeout() => "timeout",
                _ => "backend-failure",
            };
            security_center::record_failure_backend(&state,None,&body.username,
                &security_center::device(&headers,&ext,body.client_type.as_deref()),"password",reason,Some("jmap")).await;
            if reason == "timeout" { return StatusCode::GATEWAY_TIMEOUT.into_response(); }
            return unauthorized();
        }
    };
    // An upstream may only name URLs on itself.
    if session_urls_on_origin(&jmap_url, &session).is_err() {
        tracing::warn!("proxy login refused by upstream origin policy");
        security_center::record_failure_backend(&state,None,&body.username,&security_center::device(&headers,&ext,body.client_type.as_deref()),"password","upstream-policy",Some("jmap")).await;
        return unauthorized();
    }
    let account_id = session
        .primary_mail_account()
        .unwrap_or_default()
        .to_string();
    let username = if session.username.is_empty() {
        body.username.clone()
    } else {
        session.username.clone()
    };
    // Resolve the (possibly relative) upstream apiUrl to an absolute URL so
    // server-side proxying can reach it regardless of the browser origin.
    let api_url = resolve_api_url(&jmap_url, &session.api_url);
    let primary_identity = match primary_identity::VerifiedLoginIdentity::proxy(&jmap_url, &account_id) {
        Ok(identity) => identity,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    // Credentials validated — hand off to the 2FA gate (DQ2). It either completes the
    // login (no factor enrolled/required) or returns a `twofaRequired` challenge and
    // withholds the session until the second factor clears.
    twofa_routes::gate_login(
        &state,
        &headers,
        &ext,
        twofa_routes::SessionArgs {
            authentication_method: "password",
            primary_identity,
            account_id,
            username,
            jmap_url,
            api_url,
            creds,
            client_type: body.client_type.clone(),
        },
    )
    .await
}

/// Truthy env flag (`"1"`/`"true"`, case-insensitive). Used by the default-off
/// alt-auth toggles (A1 LDAP-bind, A2 header-auth).
fn env_flag(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// A2 (trusted-proxy header auth, §18.3). When `MW_HEADER_AUTH` is set, a reverse
/// proxy that has ALREADY authenticated the user asserts the identity in a trusted
/// header (default `X-Remote-User`, override `MW_HEADER_AUTH_HEADER`). If that header
/// is present and non-empty, the session is minted for that user WITHOUT a password —
/// but still through the 2FA gate (DQ2: no downgrade). Returns `None` (fall through to
/// the password path) when the feature is off or the header is absent.
///
/// SECURITY: this trusts the asserted header ABSOLUTELY, so it is safe ONLY behind a
/// proxy that STRIPS any client-supplied copy of the header. It is OFF by default.
/// In this mode the account identity IS the asserted remote user, so 2FA enrolment
/// keys off that same identity.
///
/// t20 B2 — the header is now additionally restricted to peers listed in
/// `MW_HEADER_AUTH_TRUSTED_IPS`, which SPEC §18.3 has always promised ("explicitly
/// enabled + IP-restricted") and the code did not enforce. Without it, anyone able
/// to reach the app port directly could send `X-Remote-User: <anyone>` and be issued
/// that user's session; "the port is firewalled to the proxy" was the only thing
/// standing in the way, and that is a deployment assumption, not a control.
async fn header_auth_login(
    state: &AppState,
    headers: &HeaderMap,
    ext: &axum::http::Extensions,
    body: &LoginReq,
) -> Option<Response> {
    if !env_flag("MW_HEADER_AUTH") {
        return None;
    }
    let header_name = std::env::var("MW_HEADER_AUTH_HEADER")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "X-Remote-User".to_string());
    let username = headers
        .get(header_name.as_str())
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())?
        .to_string();
    // An identity IS being asserted. Only a peer the operator listed may do that.
    // Refuse rather than falling through to the password path: falling through
    // would answer an identity assertion with a credential prompt, which is a
    // confusing (and probe-friendly) response to what is an attack when it
    // reaches here. A deployment fronted as documented never takes this branch.
    if !header_auth_peer_allowed(ext) {
        tracing::warn!(
            "header auth refused: peer {} is not in MW_HEADER_AUTH_TRUSTED_IPS",
            peer_ip(ext)
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| "<unknown>".into())
        );
        return Some(unauthorized());
    }
    let primary_identity = primary_identity::VerifiedLoginIdentity::header(&username);
    let host = state.external.host(headers, ext);
    let username = match domain_config::canonical(&state.store, &username, host.as_deref()).await {
        Ok(identity) => identity.login_name,
        Err(_) => return Some(unauthorized()),
    };
    match domain_config::signin_allowed(&state.store, &username).await {
        Ok(true) => {}
        Ok(false) => return Some(unauthorized()),
        Err(error) => return Some(account_gate::unavailable("header-auth domain policy", error)),
    }
    // Downstream mail access uses a configured service password (empty if unset — the
    // proxy/network is expected to authorize the upstream in this deployment shape).
    let password = std::env::var("MW_HEADER_AUTH_PASSWORD").unwrap_or_default();
    let creds = Credentials {
        username: username.clone(),
        password,
    };
    // The upstream comes from the server's configuration only (t32-h9):
    // `MW_HEADER_AUTH_JMAP_URL`, else the resolver.
    let mut account_id = username.clone();
    let (jmap_url, api_url) = if let Some(engine) = &state.engine {
        let upstream=match upstream_resolve::resolve_upstream(state,&username).await {
            Ok(upstream)=>upstream,
            Err(_)=>return Some(address_not_configured()),
        };
        match engine_mode::engine_login(engine,upstream.url(),&username,&creds.password).await {
            Ok((id,_))=>account_id=id,
            Err(_)=>return Some(unauthorized()),
        }
        ("engine".to_string(), "engine".to_string())
    } else {
        let url = match std::env::var("MW_HEADER_AUTH_JMAP_URL")
            .ok()
            .filter(|s| !s.is_empty())
        {
            Some(url) => url,
            None => match upstream_resolve::resolve_upstream(state, &username).await {
                Ok(u) => u.url().to_string(),
                Err(upstream_resolve::ResolveError::NotConfigured) => {
                    return Some(address_not_configured());
                }
            },
        };
        (url.clone(), url)
    };
    Some(
        twofa_routes::gate_login(
            state,
            headers,
            ext,
            twofa_routes::SessionArgs {
                authentication_method: "header-auth",
                primary_identity,
                account_id,
                username,
                jmap_url,
                api_url,
                creds,
                client_type: body.client_type.clone(),
            },
        )
        .await,
    )
}

/// May this peer assert an identity via the header-auth header (t20 B2)?
///
/// Fails closed on every uncertainty:
///   * `MW_HEADER_AUTH_TRUSTED_IPS` unset or listing nothing parseable ⇒ no,
///     so enabling `MW_HEADER_AUTH` without an allowlist authenticates nobody
///     rather than everybody;
///   * no peer address available (the serve path installed no `ConnectInfo`) ⇒
///     no, because there is then nothing to check the assertion against.
///
/// Deliberately a **separate** list from `MW_TRUSTED_PROXIES`: a proxy trusted to
/// report a client's IP is not thereby trusted to declare who the client *is*.
/// The check is on the connected peer, not on any forwarded header — an
/// identity assertion is only as good as the hop that made it.
///
/// **That separation is real only while `MW_PROXY_PROTOCOL=off`.** With `accept`
/// or `require` the peer address is itself a declared value:
/// `proxy_protocol::negotiate` returns the address named in the PROXY header, and
/// that address becomes the `ConnectInfo` this gate reads. A member of
/// `MW_TRUSTED_PROXIES` can therefore name any peer address it likes — including
/// one inside `MW_HEADER_AUTH_TRUSTED_IPS` — and then assert an identity for a
/// password-less session. That grants nothing new to a proxy list naming the
/// balancer's own addresses, since such membership is already total authority
/// over the client-IP model; it matters when the list is written as a subnet (a
/// pod or VPC range) covering hosts other than the balancer, because every host
/// in that range can then assert an identity. With both features on, list
/// addresses rather than ranges.
fn header_auth_peer_allowed(ext: &axum::http::Extensions) -> bool {
    let raw = std::env::var("MW_HEADER_AUTH_TRUSTED_IPS").unwrap_or_default();
    header_auth_peer_allowed_with(&raw, peer_ip(ext))
}

/// The decision itself, split from the environment read so it is directly
/// testable — this is a fail-closed security gate and the process-global env
/// makes the wrapper untestable without racing every other test in the binary.
fn header_auth_peer_allowed_with(raw_list: &str, peer: Option<std::net::IpAddr>) -> bool {
    let allowed = CidrSet::parse(raw_list);
    if allowed.is_empty() {
        // Once per process: a misconfigured deployment behind a proxy that always
        // sets the header would otherwise log this on every single request.
        static WARNED: std::sync::Once = std::sync::Once::new();
        WARNED.call_once(|| {
            tracing::warn!(
                "MW_HEADER_AUTH is on but MW_HEADER_AUTH_TRUSTED_IPS lists no usable \
                 address; header auth is refusing every request"
            );
        });
        return false;
    }
    match peer {
        Some(ip) => allowed.contains(ip),
        None => false,
    }
}

/// A1 (LDAP-bind login backend, §18.3). When `MW_LDAP_BIND_AUTH` is set, authenticate
/// the presented password against the configured directory via the directory
/// handle's `bind_auth` BEFORE the mail-backend credential check; a denied
/// bind (or, fail-closed, a bind error while enabled) is the uniform 401. Off by
/// default → `Ok(())`, leaving the mail backend as the sole authority.
async fn directory_bind_gate(state: &AppState, user: &str, pass: &str) -> Result<(), &'static str> {
    if !env_flag("MW_LDAP_BIND_AUTH") {
        return Ok(());
    }
    directory_bind_result(state.directory.bind_auth(user, pass).await)
}

fn directory_bind_result(result: mw_directory::Result<BindOutcome>) -> Result<(), &'static str> {
    match result {
        Ok(BindOutcome::Ok { .. }) => Ok(()),
        Ok(BindOutcome::Denied) => Err("auth-rejected"),
        Err(_) => Err("directory-unavailable"),
    }
}

#[cfg(test)]
mod directory_login_classification_tests {
    #[test]
    fn outages_never_count_as_password_rejections() {
        use mw_directory::{BindOutcome, DirectoryError};
        assert_eq!(super::directory_bind_result(Ok(BindOutcome::Denied)), Err("auth-rejected"));
        for error in [DirectoryError::NotConfigured, DirectoryError::Transport("secret-bearing diagnostic".into()), DirectoryError::Auth("service bind error".into()), DirectoryError::Protocol("unavailable".into())] {
            let reason=super::directory_bind_result(Err(error)).unwrap_err();
            assert_eq!(reason,"directory-unavailable");
            assert!(!crate::signin_guard::credential_rejection(reason));
        }
    }
}

/// Complete a login: for a browser client (`client_type` absent) set the session +
/// CSRF cookies exactly as before; for a native client (`client_type == "native"`,
/// plan §2.2) record a `native_sessions` marker (keyed by the token HASH), set NO
/// cookie, and return the bearer token in the JSON body. The bearer token IS the
/// opaque session id — proxying reads its `sessions` row like any other.
pub(crate) async fn finish_login(
    state: &AppState,
    id: &str,
    account_id: &str,
    username: &str,
    client_type: Option<&str>,
) -> Response {
    finish_login_ext(state, id, account_id, username, client_type, json!({})).await
}

/// Like [`finish_login`] but merges the object `extra` into the success body. The
/// 2FA forced-enrolment completion (t16 e3) uses it to hand the one-time recovery
/// codes back alongside the freshly-issued session, without a second round-trip.
pub(crate) async fn finish_login_ext(
    state: &AppState,
    id: &str,
    account_id: &str,
    username: &str,
    client_type: Option<&str>,
    extra: serde_json::Value,
) -> Response {
    let storage_authority = match state.store.session_storage_authority(id).await {
        Ok(value) => value,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    if client_type == Some("native") {
        let now = push_relay::now_rfc3339();
        let row = NativeSessionRow {
            token_hash: push_relay::hash_token(id),
            account_id: account_id.to_string(),
            client_type: "native".to_string(),
            created_at: now.clone(),
            last_seen: now,
            rotated_from: None,
        };
        if let Err(e) = state.store.create_native_session(&row).await {
            tracing::error!("failed to persist native session: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response();
        }
        // No cookie, no double-submit CSRF token — the bearer token is the credential.
        let mut body = json!({
            "ok": true,
            "accountId": account_id,
            "username": username,
            "token": id,
        });
        merge_body(&mut body, extra);
        if let Some(authority) = storage_authority { body["storageAuthority"] = json!(authority); }
        return Json(body).into_response();
    }
    let mut body = json!({
        "ok": true,
        "accountId": account_id,
        "username": username,
    });
    merge_body(&mut body, extra);
    if let Some(authority) = storage_authority { body["storageAuthority"] = json!(authority); }
    authenticated_response(state, id, body)
}

/// Shallow-merge the fields of `extra` (if an object) into `base` (if an object).
fn merge_body(base: &mut serde_json::Value, extra: serde_json::Value) {
    if let (Some(b), serde_json::Value::Object(e)) = (base.as_object_mut(), extra) {
        for (k, v) in e {
            b.insert(k, v);
        }
    }
}

/// Engine-mode login against `server_url`, the `imap(s)://`/`pop3(s)://` server
/// the resolver picked. Connects the account, then issues the same session cookie
/// the proxy path uses so the browser flow is identical.
async fn engine_login(
    state: &AppState,
    headers: &HeaderMap,
    ext: &axum::http::Extensions,
    engine: &Arc<Engine>,
    body: LoginReq,
    server_url: &str,
) -> Response {
    let (account_id, username) =
        match engine_mode::engine_login(engine, server_url, &body.username, &body.password).await {
            Ok(pair) => pair,
            Err(error) => {
                tracing::warn!("engine login failed");
                security_center::record_failure_backend(state,None,&body.username,
                    &security_center::device(headers,ext,body.client_type.as_deref()),"password",error.reason.code(),Some("engine")).await;
                if error.reason == login_failure::LoginFailureReason::Timeout { return StatusCode::GATEWAY_TIMEOUT.into_response(); }
                return unauthorized();
            }
        };
    let primary_identity = match primary_identity::VerifiedLoginIdentity::engine(state, &account_id).await {
        Ok(identity) => identity,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let creds = Credentials {
        username: body.username.clone(),
        password: body.password.clone(),
    };
    // Credentials validated — same 2FA gate as the proxy branch (DQ2). The session
    // persists the `"engine"` sentinel for jmap/api URLs.
    twofa_routes::gate_login(
        state,
        headers,
        ext,
        twofa_routes::SessionArgs {
            authentication_method: "password",
            primary_identity,
            account_id,
            username,
            jmap_url: "engine".to_string(),
            api_url: "engine".to_string(),
            creds,
            client_type: body.client_type.clone(),
        },
    )
    .await
}

/// Build a login/rotate success response: set the session cookie, mint a fresh
/// double-submit CSRF token (cookie + body), and return the JSON payload.
fn authenticated_response(state: &AppState, id: &str, mut body: serde_json::Value) -> Response {
    let token = state.store.session_csrf_token(id);
    if let Some(obj) = body.as_object_mut() {
        obj.insert("csrfToken".into(), json!(token));
    }
    let mut resp = Json(body).into_response();
    let h = resp.headers_mut();
    h.append(header::SET_COOKIE, session_cookie(id, state.cookie_secure));
    h.append(header::SET_COOKIE, csrf_cookie(&token, state.cookie_secure));
    resp
}

/// `POST /api/discover` is gone (t32-h9; t32-admin plan DA7). It used to look an
/// address's domain up (well-known JMAP, SRV, autoconfig, ISPDB, autodiscover) and
/// hand the servers it found to the browser, which then sent one back as the
/// login's `jmapUrl`. The server now picks the upstream itself
/// (`upstream_resolve`) and a login naming one is refused, so the route answers
/// 410 without reading the body or looking anything up. Its rate limiter stays in
/// front of it and is now inert.
async fn discover() -> Response {
    (
        StatusCode::GONE,
        Json(json!({
            "error": "server lookup was removed; sign in with the address and the password",
            "code": "discoverRemoved",
        })),
    )
        .into_response()
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    // A native shell signs out with its bearer token (t31): end that session and
    // its `native_sessions` marker, or the token keeps working until it times out.
    // Only a token that is a native session is acted on, as in `authed_native`.
    if let Some(token) = push_relay::bearer_token(&headers) {
        let hash = push_relay::hash_token(&token);
        if let Ok(Some(_)) = state.store.get_native_session(&hash).await {
            let _ = state.store.delete_session(&token).await;
            let _ = state.store.delete_native_session(&hash).await;
            state.sessions.forget(&token);
        }
    }
    if let Some(id) = cookie_value(&headers) {
        let _ = state.store.delete_session(&id).await;
        state.sessions.forget(&id);
    }
    let mut resp = StatusCode::NO_CONTENT.into_response();
    resp.headers_mut()
        .append(header::SET_COOKIE, clear_cookie(state.cookie_secure));
    resp
}

async fn me(State(state): State<AppState>, headers: HeaderMap) -> Response {
    // Reachable by an account held for a password change: this is where the
    // client learns it is held.
    let (session, gate) = match authed_with_gate(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let mut body = json!({
        "username": session.username,
        "accountId": session.account_id,
    });
    match state.store.session_storage_authority(&session.id).await {
        Ok(Some(authority)) => body["storageAuthority"] = json!(authority),
        Ok(None) => {}
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
    if gate.password_change_required
        && let Some(obj) = body.as_object_mut()
    {
        obj.insert("passwordChangeRequired".into(), json!(true));
    }
    // Bind the readable token to this session, replacing stale tokens after login.
    let existing = hardening::cookie(&headers, CSRF_COOKIE);
    let token = state.store.session_csrf_token(&session.id);
    if let Some(obj) = body.as_object_mut() {
        obj.insert("csrfToken".into(), json!(token));
    }
    let mut resp = Json(body).into_response();
    if existing.as_deref() != Some(token.as_str()) {
        resp.headers_mut()
            .append(header::SET_COOKIE, csrf_cookie(&token, state.cookie_secure));
    }
    resp
}

/// `POST /api/session/rotate` — issue a new session id (and CSRF token) for the
/// current credentials and invalidate the old id. Rotation caps how long any one
/// identifier is valid without forcing re-login (§7.4). Origin-checked like all
/// state-changing routes.
async fn rotate_session(State(state): State<AppState>, headers: HeaderMap) -> Response {
    // This endpoint rotates a browser cookie, never a different bearer principal.
    if headers.contains_key(header::AUTHORIZATION) {
        return unauthorized();
    }
    let old_id = match cookie_value(&headers) {
        Some(id) => id,
        None => return unauthorized(),
    };
    let session = match authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let new_id = match state
        .store
        .create_session(
            &session.account_id,
            &session.username,
            &session.jmap_url,
            &session.api_url,
            &session.credentials,
        )
        .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("failed to rotate session: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response();
        }
    };
    match state.store.rotate_session_security(&old_id,&new_id).await {
        Ok(true)=>{}
        result=>{
            let _=state.store.delete_session(&new_id).await;
            return match result {
                Err(error)=>account_gate::unavailable("session rotation",error),
                _=>unauthorized(),
            };
        }
    }
    state.step_up.forget_user(&old_id);
    state.sessions.rotate(&old_id, &new_id);
    authenticated_response(&state, &new_id, json!({ "ok": true }))
}

// ---------------------------------------------------------------------------
// JMAP proxy
// ---------------------------------------------------------------------------

async fn jmap_session(State(state): State<AppState>, headers: HeaderMap, ext: axum::http::Extensions) -> Response {
    let scoped = match scope_mw::sieve_key_session(&state,&headers,&ext,None,None).await {Ok(s)=>s,Err(r)=>return r};
    let scripts_only=scoped.is_some();
    let session = match scoped {Some(s)=>s,None=>match authed(&state,&headers).await {Ok(s)=>s,Err(r)=>return r}};
    if let Some(engine) = &state.engine {
        if let Err(e) = engine_mode::ensure_account(engine, &session.account_id).await {
            tracing::warn!("engine account not available: {e}");
            return upstream_error();
        }
        let dsn = !scripts_only && engine.dsn_offered(&session.account_id).await.unwrap_or(false);
        let mut document=engine.session_json_for(&session.account_id,&session.username);
        if !dsn { document["accounts"][&session.account_id]["accountCapabilities"]["urn:ietf:params:jmap:submission"]["submissionExtensions"]=json!({}); }
        if scripts_only {
            document["capabilities"].as_object_mut().unwrap().retain(|key,_|key=="urn:ietf:params:jmap:core");
            document["primaryAccounts"]=json!({});
            if let Some(accounts)=document["accounts"].as_object_mut(){accounts.retain(|key,_|key==&session.account_id);}
            document["accounts"][&session.account_id]["accountCapabilities"]=json!({});
        }
        if let Ok(context)=sieve_sync::context(&state,&session,None).await {
            let capability=mw_engine::sieve_jmap::CAPABILITY;
            document["capabilities"][capability]=json!({"implementation":"Mailwoman"});
            document["accounts"][&session.account_id]["accountCapabilities"][capability]=context.account_capability();
            document["primaryAccounts"][capability]=session.account_id.clone().into();
        }
        return Json(prefix_session_urls(document,&state.base_path)).into_response();
    }    let mut upstream =
        match upstream_session(&state.security, &session.jmap_url, &session.credentials).await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("upstream session fetch failed: {e}");
                return upstream_error();
            }
        };
    // Rewrite every URL so the browser only ever talks to us, never upstream.
    // t20 B4: and under the deploy prefix, so the browser addresses `/mail/jmap/*`.
    let base = &*state.base_path;
    upstream.api_url = format!("{base}/jmap/api");
    upstream.download_url = format!("{base}/jmap/download/{{accountId}}/{{blobId}}/{{name}}");
    upstream.upload_url = format!("{base}/jmap/upload/{{accountId}}");
    upstream.event_source_url = format!("{base}/jmap/eventsource");
    Json(upstream).into_response()
}

/// Prefix the origin-relative URLs a JMAP session object hands the browser with
/// the deploy base (t20 B4).
///
/// Engine mode builds the session in `mw_engine::session_json`, which knows
/// nothing about how this server is hosted; the four RFC 8620 URL fields are the
/// only place the browser learns where to send JMAP traffic, so they are prefixed
/// here rather than there. Absolute URLs (a future upstream-provided value) are
/// left alone.
///
/// The `/dav/*` sharing endpoints need no equivalent: nothing serialises an
/// absolute DAV base to a client — the routes are reached by path, and the
/// prefix is stripped by [`mount_under_base_path`]'s nest before routing.
fn prefix_session_urls(mut session: Value, base: &str) -> Value {
    if base.is_empty() {
        return session;
    }
    for key in ["apiUrl", "downloadUrl", "uploadUrl", "eventSourceUrl"] {
        if let Some(Value::String(url)) = session.get_mut(key)
            && url.starts_with('/')
        {
            *url = format!("{base}{url}");
        }
    }
    session
}

async fn jmap_api(State(state): State<AppState>, source: Option<Extension<scope_mw::RestSessionSource>>, headers: HeaderMap, body: Body) -> Response {
    let body = match axum::body::to_bytes(body, mw_engine::jmap::core::MAX_SIZE_REQUEST).await {
        Ok(body) => body,
        Err(error) => {
            request_protection::body_error(&error);
            return if state.engine.is_some() { jmap_limits::problem(mw_engine::jmap::core::problem("limit", Some("maxSizeRequest")), StatusCode::BAD_REQUEST) } else { StatusCode::PAYLOAD_TOO_LARGE.into_response() };
        },
    };
    let headless_key = matches!(&source, Some(Extension(scope_mw::RestSessionSource::Key { .. })));
    let key_account = match &source {
        Some(Extension(scope_mw::RestSessionSource::Key { account_id })) => Some(account_id.clone()),
        _ => None,
    };
    let authenticated = match source {
        Some(Extension(source)) => scope_mw::rest_session(&state, &headers, source).await,
        None => authed(&state, &headers).await,
    };
    let mut session = match authenticated {
        Ok(s) => s,
        Err(resp) => {
            // t14 (26.14, §Wave-B E-mount): the server-metadata admin editor drives
            // `ServerMetadata/*` + `MailboxRights/*` against an admin-SELECTED account.
            // When the normal mailbox-cookie auth fails, an authenticated admin session
            // may still issue exactly those methods against the selected account's
            // backend (the accountId is carried in each method call). Purely additive —
            // this only runs on the auth-failure path, so the normal JMAP auth is
            // never weakened; a non-admin / non-metadata request falls through to `resp`.
            if let Some(admin_resp) =
                admin_maintenance::try_admin_jmap_passthrough(&state, &headers, &body).await
            {
                return admin_resp;
            }
            return resp;
        }
    };
    if headless_key {
        if let Err(response) = domain_upgrade::refresh_session(&state, &mut session).await {
            return response;
        }
        if key_account.as_deref() != Some(session.account_id.as_str()) {
            return StatusCode::FORBIDDEN.into_response();
        }
        // A credential lookup must never inherit another browser's proof.
        session.id.clear();
    }
    // Authorize the entire batch before connecting an account or forwarding
    // any call. A disabled feature cannot hide behind a permitted first call.
    let request: Value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(_) => {
            request_protection::observe(mw_store::request_protection_model::Signal::Malformed);
            return if state.engine.is_some() { jmap_limits::problem(mw_engine::jmap::core::problem("notJSON", None), StatusCode::BAD_REQUEST) } else { (StatusCode::BAD_REQUEST, "invalid JMAP JSON").into_response() };
        },
    };
    let _native_request_permit = if state.engine.is_some() {
        if let Err(problem) = mw_engine::jmap::core::validate(&request, body.len(), true) {
            return jmap_limits::problem(problem, StatusCode::BAD_REQUEST);
        }
        match jmap_limits::request() { Ok(permit) => Some(permit), Err(response) => return response }
    } else { None };
    if let Err(response) = feature_jmap::admit(&state, &session, &request).await {
        return response;
    }
    if let Some(engine) = &state.engine {
        let echo_only = request["methodCalls"].as_array().is_some_and(|calls| calls.iter().all(|call| call[0] == "Core/echo"));
        if !echo_only && let Err(e) = engine_mode::ensure_account(engine, &session.account_id).await {
            tracing::warn!("engine account not available: {e}");
            return upstream_error();
        }
        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(e) => {
                return (StatusCode::BAD_REQUEST, format!("bad json: {e}")).into_response();
            }
        };
        let has_sieve = request.get("methodCalls").and_then(Value::as_array).is_some_and(|calls|
            calls.iter().any(|call| call.get(0).and_then(Value::as_str).is_some_and(|name|mw_engine::sieve_jmap::is_sieve_method(name) || mw_engine::sieve_jmap::is_managed_method(name))));
        let mut sieve = if has_sieve {
            match sieve_sync::context(&state, &session, Some(&request)).await {
                Ok(context) => Some(context),
                Err(response) => return response,
            }
        } else { None };
        if request.get("methodCalls").and_then(Value::as_array).is_some_and(|calls| calls.iter().any(|call| call.get(0).and_then(Value::as_str).is_some_and(mw_engine::pim::dispatch::is_pim_method))) {
            nextcloud_pim::install(engine, &state.store, &session.account_id);
        }
        let response = engine.handle_jmap_with_sieve(&session.account_id, &request, sieve.as_mut()).await;
        if sieve.as_ref().is_some_and(|ctx| ctx.forwarding_added) {
            alerts::emit(&state, &session.account_id, alerts::AlertKind::ForwardingAdded, json!({})).await;
        }
        return Json(response).into_response();
    }
    let mut deferred_sieve_errors=Vec::new();
    let mut response_order=Vec::new();
    let body=if let Ok(mut request)=serde_json::from_slice::<Value>(&body) {
        let has_sieve=request["methodCalls"].as_array().is_some_and(|calls|calls.iter().any(|c|c[0].as_str().is_some_and(mw_engine::sieve_jmap::is_sieve_method)));
        if has_sieve {
            let upstream=match upstream_session(&state.security,&session.jmap_url,&session.credentials).await {Ok(s)=>s,Err(_)=>return upstream_error()};
            if !upstream.capabilities.contains_key(mw_engine::sieve_jmap::CAPABILITY) {
                if let Some(calls)=request["methodCalls"].as_array_mut() {
                    response_order=calls.iter().map(|c|c[2].clone()).collect();
                    calls.retain(|call| {if call[0].as_str().is_some_and(mw_engine::sieve_jmap::is_sieve_method) {
                        deferred_sieve_errors.push(json!(["error",{"type":"unknownMethod"},call[2]])); false
                    }else{true}});
                    if calls.is_empty(){return Json(json!({"methodResponses":deferred_sieve_errors,"sessionState":upstream.state})).into_response();}
                }
                if let Some(using)=request["using"].as_array_mut(){using.retain(|cap|cap!=mw_engine::sieve_jmap::CAPABILITY);}
            }
        }
        Bytes::from(serde_json::to_vec(&request).unwrap_or_default())
    }else{body};    let client = match upstream_client(
        &state.security,
        &session.jmap_url,
        &session.api_url,
        &session.credentials,
    )
    .await
    {
        Ok(c) => c,
        Err(r) => {
            tracing::warn!("upstream API request refused: {r}");
            return upstream_error();
        }
    };
    match client.request_raw(&session.api_url, body).await {
        Ok((status, bytes)) => {
            let bytes=if status.is_success() && !deferred_sieve_errors.is_empty() {
                match serde_json::from_slice::<Value>(&bytes) {
                    Ok(mut response)=>{
                        if let Some(responses)=response["methodResponses"].as_array_mut(){
                            responses.extend(deferred_sieve_errors);
                            responses.sort_by_key(|r|response_order.iter().position(|id|id==&r[2]).unwrap_or(usize::MAX));
                        }
                        Bytes::from(serde_json::to_vec(&response).unwrap_or_default())
                    },Err(_)=>return upstream_error(),
                }
            }else{bytes};
            let code =
                StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            let mut resp = Response::new(Body::from(bytes));
            *resp.status_mut() = code;
            resp.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
            resp
        }
        Err(e) => {
            tracing::warn!("upstream proxy failed: {e}");
            upstream_error()
        }
    }
}

fn upstream_error() -> Response {
    (
        StatusCode::BAD_GATEWAY,
        Json(json!({ "error": "upstream request failed" })),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Proxy-mode upstream policy (t24 B2)
// ---------------------------------------------------------------------------
//
// The proxy's upstream is request-shaped twice over: an anonymous login names the
// server, and that server's session document names the `apiUrl`, `downloadUrl` and
// `uploadUrl` the relayed legs use. Before t24 nothing checked either, so a login
// against a JMAP server the caller controlled turned `/jmap/download` and
// `/jmap/api` into a read-back of any address the server host could reach.
//
// Every upstream request in proxy mode now goes through [`upstream_client`], which
//   1. requires the target to share the origin of the session's `jmap_url`, so an
//      upstream can only name URLs on itself;
//   2. applies the address policy: the operator allowlist when one is configured
//      (`SecurityConfig::jmap_upstreams`), otherwise `mw_egress::ip_allowed`;
//   3. resolves once and pins the connection to the checked address with
//      `mw_egress::harden_client`, which also turns redirects off and drops any
//      ambient proxy.
// The check is repeated on every leg, not only at login: a stored session outlives
// a configuration change, and an upstream can change its session document.
//
// This is not the egress *route* (`fetch_remote_routed`). A route may be selected
// only by deployment configuration, and the upstream here is request-derived; the
// policy and pinning primitives are what apply.

/// Total time allowed for one upstream request. `harden_client` sets
/// `mw_egress::FETCH_TIMEOUT` (10 s), which suits an image fetch but not a 50 MB
/// upload or a large `Email/get`; before t24 these requests had no limit at all.
const UPSTREAM_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);

/// Why an upstream URL was not used. Logged for the operator; the client only
/// ever sees the uniform `401` (login) or `502` (relayed legs).
#[derive(Debug)]
pub(crate) enum UpstreamRefusal {
    /// Not an absolute http(s) URL.
    Malformed,
    /// Not on the origin the session is bound to.
    CrossOrigin,
    /// An allowlist is configured and does not name this origin.
    NotListed,
    /// `mw_egress` refused it: blocked address, credentials in the URL, or the
    /// name did not resolve.
    Egress(mw_egress::Refusal),
    /// The session fetch redirected more than `mw_egress::MAX_REDIRECTS` times.
    TooManyRedirects,
    /// The pinned HTTP client could not be built.
    Client,
}

impl std::fmt::Display for UpstreamRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed => f.write_str("not an absolute http(s) URL"),
            Self::CrossOrigin => f.write_str("not on the upstream's origin"),
            Self::NotListed => f.write_str("origin not in MW_JMAP_UPSTREAMS"),
            Self::Egress(r) => write!(f, "refused by the egress policy ({r:?})"),
            Self::TooManyRedirects => f.write_str("too many redirects"),
            Self::Client => f.write_str("HTTP client could not be built"),
        }
    }
}

/// Why an upstream session could not be obtained.
#[derive(Debug)]
pub(crate) enum UpstreamSessionError {
    /// A URL was refused before any request was sent to it.
    Refused(UpstreamRefusal),
    /// The request was sent and failed (status, transport, bad JSON).
    Jmap(mw_jmap::JmapError),
}

impl std::fmt::Display for UpstreamSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Refused(r) => write!(f, "upstream refused: {r}"),
            Self::Jmap(e) => write!(f, "{e}"),
        }
    }
}

/// Parse an absolute http(s) URL with a host.
fn http_url(raw: &str) -> Result<reqwest::Url, UpstreamRefusal> {
    let url = reqwest::Url::parse(raw.trim()).map_err(|_| UpstreamRefusal::Malformed)?;
    match url.scheme() {
        "http" | "https" if url.host_str().is_some() => Ok(url),
        _ => Err(UpstreamRefusal::Malformed),
    }
}

/// The origin an `MW_JMAP_UPSTREAMS` entry names, if it names one.
fn upstream_list_origin(entry: &str) -> Option<reqwest::Url> {
    let url = http_url(entry).ok()?;
    (url.username().is_empty() && url.password().is_none()).then_some(url)
}

/// The address predicate that applies to `url`, or the refusal when a configured
/// allowlist does not name its origin.
fn upstream_address_policy(
    security: &SecurityConfig,
    url: &reqwest::Url,
) -> Result<fn(&std::net::IpAddr) -> bool, UpstreamRefusal> {
    // The configured default upstream is named by the operator as surely as a
    // listed origin is (t32-h9).
    if security
        .upstream_default
        .as_deref()
        .and_then(upstream_list_origin)
        .is_some_and(|d| d.origin() == url.origin())
    {
        return Ok(operator_listed_address);
    }
    let Some(list) = &security.jmap_upstreams else {
        return Ok(mw_egress::ip_allowed);
    };
    let listed = list
        .iter()
        .filter_map(|entry| upstream_list_origin(entry))
        .any(|entry| entry.origin() == url.origin());
    if listed {
        Ok(operator_listed_address)
    } else {
        Err(UpstreamRefusal::NotListed)
    }
}

/// The address predicate for an origin the operator named in `MW_JMAP_UPSTREAMS`
/// or `MW_UPSTREAM_DEFAULT`.
/// Naming the origin is the operator's decision to trust wherever it resolves —
/// that is what lets a deployment use an internal JMAP server — so no range is
/// refused. The connection is still pinned to the address resolved at check time.
fn operator_listed_address(_: &std::net::IpAddr) -> bool {
    true
}

/// Check `target` against the upstream policy and return a JMAP client pinned to
/// its resolved address. `bound` is the session's `jmap_url`, whose origin the
/// target must share.
///
/// The returned client is for `target` only: it is pinned to that host's address
/// and follows no redirects.
pub(crate) async fn upstream_client(
    security: &SecurityConfig,
    bound: &str,
    target: &str,
    credentials: &Credentials,
) -> Result<JmapClient, UpstreamRefusal> {
    let bound = http_url(bound)?;
    let url = http_url(target)?;
    if url.origin() != bound.origin() {
        return Err(UpstreamRefusal::CrossOrigin);
    }
    let policy = upstream_address_policy(security, &url)?;
    let pinned = mw_egress::validate_and_resolve_with(url, policy)
        .await
        .map_err(UpstreamRefusal::Egress)?;
    let http = mw_egress::harden_client(reqwest::Client::builder(), &pinned.host, pinned.addr)
        .timeout(UPSTREAM_TIMEOUT)
        .connect_timeout(mw_egress::FETCH_TIMEOUT)
        .build()
        .map_err(|_| UpstreamRefusal::Client)?;
    Ok(JmapClient::with_http(
        http,
        &credentials.username,
        &credentials.password,
    ))
}

/// Fetch the upstream JMAP session for `jmap_url` under the upstream policy.
///
/// RFC 8620 §2.2 allows `/.well-known/jmap` to redirect, and providers do. A
/// redirect is followed only on the same origin, and each hop is checked and
/// pinned again, up to `mw_egress::MAX_REDIRECTS`.
pub(crate) async fn upstream_session(
    security: &SecurityConfig,
    jmap_url: &str,
    credentials: &Credentials,
) -> Result<mw_jmap::Session, UpstreamSessionError> {
    let mut url = mw_jmap::client::session_url(jmap_url)
        .map_err(|_| UpstreamSessionError::Refused(UpstreamRefusal::Malformed))?;
    for _ in 0..=mw_egress::MAX_REDIRECTS {
        let client = upstream_client(security, jmap_url, &url, credentials)
            .await
            .map_err(UpstreamSessionError::Refused)?;
        match client.session_at(&url).await {
            Err(mw_jmap::JmapError::Redirect(location)) => {
                let next = http_url(&url)
                    .and_then(|base| base.join(&location).map_err(|_| UpstreamRefusal::Malformed))
                    .map_err(UpstreamSessionError::Refused)?;
                url = next.to_string();
            }
            other => return other.map_err(UpstreamSessionError::Jmap),
        }
    }
    Err(UpstreamSessionError::Refused(
        UpstreamRefusal::TooManyRedirects,
    ))
}

/// Require the session's relayed URLs to share the upstream's origin. Checked at
/// login so a session naming a foreign URL is never issued; the relayed legs check
/// the URL they actually use again, through [`upstream_client`].
///
/// `eventSourceUrl` is not checked: the proxy never relays it (`/jmap/eventsource`
/// is served locally), so constraining it would refuse upstreams for no gain.
fn session_urls_on_origin(
    jmap_url: &str,
    session: &mw_jmap::Session,
) -> Result<(), UpstreamRefusal> {
    let origin = http_url(jmap_url)?.origin();
    for relayed in [&session.api_url, &session.download_url, &session.upload_url] {
        let url = http_url(&resolve_api_url(jmap_url, relayed))?;
        if url.origin() != origin {
            return Err(UpstreamRefusal::CrossOrigin);
        }
    }
    Ok(())
}

/// The `Content-Type` a download is served with, in both modes: the upstream's or
/// the MIME part's value is reduced to its `type/subtype`, and any type a browser
/// would run or render as a document is replaced with `application/octet-stream`.
/// The response also carries `Content-Disposition: attachment`, `nosniff` and
/// [`DOWNLOAD_CSP`]; this is one layer of three.
fn proxied_download_type(upstream: Option<&str>) -> String {
    const OPAQUE: &str = "application/octet-stream";
    let Some(essence) = upstream
        .and_then(|v| v.split(';').next())
        .map(|v| v.trim().to_ascii_lowercase())
    else {
        return OPAQUE.to_string();
    };
    let token = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"!#$&-^_.+".contains(&b))
    };
    let well_formed = essence
        .split_once('/')
        .is_some_and(|(kind, sub)| token(kind) && token(sub));
    let active = essence.contains("html")
        || essence.contains("xml")
        || essence.contains("script")
        || essence.starts_with("multipart/");
    if well_formed && !active {
        essence
    } else {
        OPAQUE.to_string()
    }
}

// ---------------------------------------------------------------------------
// Blob download + export (plan §2.4 / §10.5, e14)
// ---------------------------------------------------------------------------

/// `GET /jmap/download/{accountId}/{blobId}/{name}` — the JMAP downloadUrl the
/// session rewrites to (RFC 8620 §6.2). Cookie-authed. In engine mode the blobId
/// is resolved locally by [`mw_engine::Engine::fetch_blob`] (whole message
/// `<stableId>` → `message/rfc822`; a part `<stableId>.<partId>` → its decoded
/// bytes). In proxy mode the request is forwarded to the upstream downloadUrl
/// with injected Basic auth and relayed back verbatim.
///
/// Both modes advertise `Accept-Ranges: bytes` and answer a single `Range` with
/// `206` (t20 B9). Neither mode streams: the whole blob is read into memory
/// first and the range is sliced out of it, so peak memory per concurrent
/// download is the blob size regardless of the range asked for.
async fn jmap_download(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<attachment_policy::DownloadQuery>,
    ext: axum::http::Extensions,
    UrlPath((account_id, blob_id, name)): UrlPath<(String, String, String)>,
) -> Response {
    let scoped=match scope_mw::sieve_key_session(&state,&headers,&ext,Some(&account_id),Some(&blob_id)).await {Ok(s)=>s,Err(r)=>return r};
    let session=match scoped {Some(s)=>s,None=>match authed(&state,&headers).await {Ok(s)=>s,Err(r)=>return r}};    // A session may only download blobs from its own account.
    if account_id != session.account_id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "account mismatch" })),
        )
            .into_response();
    }
    let policy = match attachment_policy::effective(&state,&session).await { Ok(p)=>p,Err(r)=>return r };
    if query.preview==Some(true) && !policy.preview_allowed {return attachment_policy::refusal("attachmentPreviewDisabled");}
    if !policy.download_allowed {return attachment_policy::refusal("downloadsDisabled");}
    if let Some(engine) = &state.engine {
        match engine.fetch_blob(&account_id, &blob_id).await {
            Ok(Some(blob)) => {
                if blob_id.starts_with('U') && (!policy.allowed_extensions.is_empty() || !policy.blocked_extensions.is_empty() || !policy.warn_extensions.is_empty()) {return attachment_policy::refusal("attachmentMetadataUnavailable");}
                let metadata=attachment_policy::Metadata {name:blob.filename.clone(),mime:blob.content_type.clone(),size:blob.bytes.len() as u64};
                let decision=match attachment_policy::authorize(&policy,&metadata,&query) {Ok(d)=>d,Err(r)=>return r};
                let filename = if name.is_empty() {
                    &blob.filename
                } else {
                    &name
                };
                // The part's type is the sender's word for it, reduced the same
                // way as a proxied download's (t32-h4).
                let mut response=blob_response(
                    &proxied_download_type(Some(&blob.content_type)),
                    filename,
                    blob.bytes,
                    &headers,
                );
                attachment_policy::response_headers(&mut response,&decision);
                response
            }
            Ok(None) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "blob not found" })),
            )
                .into_response(),
            Err(e) => {
                tracing::warn!("blob fetch failed: {e}");
                upstream_error()
            }
        }
    } else {
        proxy_download(
            &state,
            &session,
            &blob_id,
            &name,
            &headers,
            &policy,
            &query,
        )
        .await
    }
}

/// Proxy a download to the upstream JMAP server: fetch its Session for the real
/// downloadUrl template, substitute the coordinates, GET it with injected auth,
/// and relay status + body back to the browser.
///
/// The upstream's `Content-Type` and `Content-Disposition` are not relayed as
/// received (t24 B2): a full `200` is served through [`blob_response`] with an
/// `attachment` disposition built from the requested name and the type reduced by
/// [`proxied_download_type`]; any other status carries no upstream content headers.
///
/// The body is **buffered, not streamed** — `JmapClient::get_bytes` reads the
/// whole upstream response into memory before this function sees it, so peak
/// memory per concurrent proxied download equals the blob size. (An earlier
/// comment here claimed streaming; it never did.) Because the bytes are already
/// in hand, a client `Range` is served by slicing that buffer, matching the
/// engine path's behaviour; the request itself is always sent upstream whole.
async fn proxy_download(
    state: &AppState,
    session: &mw_store::Session,
    blob_id: &str,
    name: &str,
    req: &HeaderMap,
    policy: &attachment_policy::Policy,
    query: &attachment_policy::DownloadQuery,
) -> Response {
    let security=&state.security;
    let account_id=&session.account_id;
    let metadata=if policy.restricts_content() {match attachment_policy::proxy_metadata(state,session,blob_id,query).await {Ok(parts)=>parts,Err(r)=>return r}} else {Vec::new()};
    for part in &metadata {if let Err(r)=attachment_policy::authorize(policy,part,query) {return r;}}
    let upstream = match upstream_session(security, &session.jmap_url, &session.credentials).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("upstream session for download failed: {e}");
            return upstream_error();
        }
    };
    let url = upstream
        .download_url
        .replace("{accountId}", &percent_encode(account_id))
        .replace("{blobId}", &percent_encode(blob_id))
        .replace("{name}", &percent_encode(name))
        .replace("{type}", "application/octet-stream");
    let abs = resolve_api_url(&session.jmap_url, &url);
    let client =
        match upstream_client(security, &session.jmap_url, &abs, &session.credentials).await {
            Ok(c) => c,
            Err(r) => {
                tracing::warn!("upstream download refused: {r}");
                return upstream_error();
            }
        };
    let result=if policy.max_bytes>0 {client.get_bytes_limited(&abs,policy.max_bytes as usize).await} else {client.get_bytes(&abs).await.map(|(status,kind,_,bytes)|(status,kind,bytes))};
    match result {
        Ok((status, content_type, bytes)) => {
            let code = StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            // Only a full upstream 200 is a representation we may slice and serve as
            // a blob; anything else (redirect, error, or an upstream 206 we did not
            // ask for) is relayed with its status and body only.
            if code == StatusCode::OK {
                for part in &metadata { let actual=attachment_policy::Metadata{name:part.name.clone(),mime:part.mime.clone(),size:bytes.len() as u64}; if let Err(r)=attachment_policy::authorize(policy,&actual,query) {return r;} }
                for part in &metadata { let actual=attachment_policy::Metadata{name:part.name.clone(),mime:content_type.clone().unwrap_or_else(|| "application/octet-stream".into()),size:bytes.len() as u64}; if let Err(r)=attachment_policy::authorize(policy,&actual,query) {return r;} }
                let decision=policy.decide("",content_type.as_deref().unwrap_or("application/octet-stream"),bytes.len() as u64);
                let mut response=blob_response(
                    &proxied_download_type(content_type.as_deref()),
                    name,
                    bytes.to_vec(),
                    req,
                );
                attachment_policy::response_headers(&mut response,&attachment_policy::Decision{download_allowed:true,preview_allowed:policy.preview_allowed,warning:decision.warning,reason:None});
                return response;
            }
            let mut resp = Response::new(Body::from(bytes));
            *resp.status_mut() = code;
            resp
        }
        Err(e) => {
            tracing::warn!("download proxy failed: {e}");
            upstream_error()
        }
    }
}

/// Upload size cap enforced by [`jmap_upload`], matching the `maxSizeUpload` the
/// session advertises (`mw_engine::session_json` / `mw-autoconfig`). A body above
/// this is refused with `413` before any storage or upstream forwarding.
const MAX_UPLOAD_BYTES: usize = 50_000_000;

/// The configured upload directory (`MW_UPLOAD_DIR`), or `None` when unset/empty.
/// The filesystem [`mw_store::FsUploadBackend`] is rooted here; an unset directory
/// leaves the store's fail-closed backend in place.
fn upload_dir_from_env() -> Option<PathBuf> {
    std::env::var_os("MW_UPLOAD_DIR")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}

/// `POST /jmap/upload/{accountId}` — the JMAP upload endpoint the session advertises
/// via `uploadUrl`/`maxSizeUpload` (RFC 8620 §6.1). Cookie-authed and account-scoped
/// (a session may only upload to its own account). The raw request body is read up to
/// `maxSizeUpload` (over-limit → `413`), the `Content-Type` header is taken as the
/// blob type (default `application/octet-stream`), and in engine mode the bytes are
/// sealed + written to the configured upload backend by
/// [`mw_engine::Engine::store_upload`], which mints a `U`-prefixed `blobId`. Returns
/// `200 {accountId, blobId, type, size}`; that blobId then resolves through
/// `fetch_blob` as an outgoing `Email/set` attachment. In proxy mode the upload is
/// forwarded to the upstream uploadUrl ([`proxy_upload`]).
async fn jmap_upload(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    UrlPath(account_id): UrlPath<String>,
    body: Body,
) -> Response {
    let scoped=match scope_mw::sieve_key_session(&state,&headers,&ext,Some(&account_id),None).await {Ok(s)=>s,Err(r)=>return r};
    let scripts_only=scoped.is_some();
    let session=match scoped {Some(s)=>s,None=>match authed(&state,&headers).await {Ok(s)=>s,Err(r)=>return r}};    // A session may only upload to its own account.
    if account_id != session.account_id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "account mismatch" })),
        )
            .into_response();
    }
    let _native_upload_permit = if state.engine.is_some() {
        match jmap_limits::upload() { Ok(permit) => Some(permit), Err(response) => return response }
    } else { None };
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .unwrap_or_else(|| "application/octet-stream".to_string());
    // Read the body with a hard cap at `maxSizeUpload`; a larger body → 413 (before
    // any storage write or upstream forwarding). `Body` is extracted directly so the
    // read is bounded here rather than by the default (smaller) extractor limit, and
    // read through [`read_body_capped`] rather than `axum::body::to_bytes` so an
    // over-limit body is drained instead of closed on — see [`DRAIN_BUDGET_BYTES`].
    let cap=if scripts_only {
        if !content_type.split(';').next().is_some_and(|v|v.trim().eq_ignore_ascii_case("application/sieve")) {return StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response();}
        match sieve_sync::upload_limit(&state,&session).await {Ok(n)=>n,Err(r)=>return r}
    }else{MAX_UPLOAD_BYTES};
    let bytes = match read_body_capped(body, cap).await {
        Ok(b) => b,
        Err(()) => return upload_too_large(),
    };
    if scripts_only && (bytes.is_empty() || std::str::from_utf8(&bytes).is_err()) {return StatusCode::BAD_REQUEST.into_response();}
    let Some(engine) = &state.engine else {
        return proxy_upload(&state.security, &session, &account_id, &content_type, bytes).await;
    };
    match engine
        .store_upload(&account_id, &content_type, &bytes)
        .await
    {
        Ok(blob_id) => (
            StatusCode::OK,
            Json(json!({
                "accountId": account_id,
                "blobId": blob_id,
                "type": content_type,
                "size": bytes.len(),
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::warn!("upload store failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "upload storage failed" })),
            )
                .into_response()
        }
    }
}

/// How many further bytes of an over-limit request body we will read and discard
/// before answering `413`.
///
/// The response is written to a socket the client is **still writing to**. Close
/// on it and the reverse proxy in front sees its own write fail; Apache's
/// `mod_proxy` then abandons the exchange without reading the response already
/// waiting on the socket, and the client gets httpd's `502 text/html` instead of
/// this crate's JSON `413` (t20-e-e2e D6). `t20-e7` measured five `mod_proxy`
/// configurations against exactly this — `proxy-nokeepalive` alone, then with
/// `proxy-sendcl`, with `proxy-sendchunked`, with `ping=5` (100-continue), and
/// with `LimitRequestBody` lowered to this crate's cap — and **all five still
/// produced `502`**. There is no directive meaning "on write failure, read the
/// response anyway", so the only place this is fixable is here. Five of the six
/// proxy cells tolerate the close, which is why this reads as an Apache fault
/// and is not: the JSON-error contract should not depend on how forgiving the
/// proxy is.
///
/// Draining is best-effort and **bounded twice**, because the body is
/// attacker-controlled and this request is already known to be doomed:
///
///   * **Bytes** — one further `MAX_UPLOAD_BYTES`. A client that ignored the
///     advertised `maxSizeUpload` and sent up to twice it is making a mistake
///     worth answering properly; past that it is not. Total read per request
///     stays bounded at twice the cap, which is the same order as the buffer the
///     request was always allowed to allocate.
///   * **Time** — [`DRAIN_DEADLINE`]. Bytes alone are not enough: a body
///     trickled out a byte at a time would hold the connection indefinitely
///     while staying under the byte budget. A genuine overshoot of a few
///     megabytes drains in well under a second, so the deadline only bites on a
///     client that is not really trying to finish.
///
/// Past either bound we stop reading and answer anyway — a `413` that Apache
/// drops is no worse than the `413` it dropped before, and every other proxy
/// relays it.
///
/// **Note for anyone adding another size-capped route**: `import_routes.rs` takes
/// its cap through `DefaultBodyLimit` + the `Bytes` extractor's rejection, which
/// has this same non-draining shape. It was outside this lane's locks and is
/// unfixed.
const DRAIN_BUDGET_BYTES: usize = MAX_UPLOAD_BYTES;

/// Wall-clock bound on the drain described at [`DRAIN_BUDGET_BYTES`].
const DRAIN_DEADLINE: std::time::Duration = std::time::Duration::from_secs(30);

/// Read a request body, capping it at `cap` bytes.
///
/// `Err(())` means "this body cannot be accepted" — over the cap, or a transport
/// failure — which is the same thing `axum::body::to_bytes` reported and what the
/// caller turns into a `413`. The difference is that exceeding the cap does not
/// end the read here: the remainder is drained (see [`DRAIN_BUDGET_BYTES`]) so
/// that by the time the response is written, the client has finished writing.
async fn read_body_capped(body: Body, cap: usize) -> Result<Bytes, ()> {
    use futures_util::StreamExt;

    let mut stream = body.into_data_stream();
    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = stream.next().await {
        // A transport error leaves nothing to drain: the client is already gone,
        // and that is the one case where closing immediately is right.
        let Ok(chunk) = chunk else { return Err(()) };
        if buf.len() + chunk.len() > cap {
            drain_refused_body(stream, chunk.len()).await;
            return Err(());
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(Bytes::from(buf))
}

/// Read and discard what is left of a body already refused, within both bounds.
/// `already` is the size of the chunk that broke the cap, which was discarded
/// too and so counts against the budget.
async fn drain_refused_body(mut stream: axum::body::BodyDataStream, already: usize) {
    use futures_util::StreamExt;

    let drained = tokio::time::timeout(DRAIN_DEADLINE, async {
        let mut seen = already;
        while seen < DRAIN_BUDGET_BYTES {
            match stream.next().await {
                Some(Ok(chunk)) => seen += chunk.len(),
                // Finished, or hung up: either way there is nothing left to read.
                Some(Err(_)) | None => break,
            }
        }
        seen
    })
    .await;
    match drained {
        Ok(seen) => {
            tracing::debug!("drained {seen} bytes of a refused upload before answering 413");
        }
        Err(_) => {
            tracing::debug!(
                "gave up draining a refused upload after {DRAIN_DEADLINE:?}; a proxy that \
                 abandons an exchange on write failure may turn this 413 into its own error"
            );
        }
    }
}

/// The stable JSON `413` body, shared by every size-capped route (t20 B7/B9).
///
/// Shape — treat as frozen; the proxy-conformance suite and any client that
/// wants to report the real limit match on these keys:
///
/// ```json
/// { "error": "payload too large", "limit": "maxSizeUpload", "maxBytes": 50000000 }
/// ```
///
/// `limit` names the constant that was hit (`maxSizeUpload` for `/jmap/upload`,
/// `maxMessageBytes` for the import routes) and `maxBytes` is its value in bytes.
/// Pinning the shape is what makes a refusal by the *app* distinguishable from
/// one by a fronting proxy, which answers with an HTML error page and no JSON —
/// nginx's default `client_max_body_size 1m` being the case operators hit first.
pub(crate) fn payload_too_large(limit: &str, max_bytes: usize) -> Response {
    (
        StatusCode::PAYLOAD_TOO_LARGE,
        Json(json!({
            "error": "payload too large",
            "limit": limit,
            "maxBytes": max_bytes,
        })),
    )
        .into_response()
}

/// `413` for an upload body over `maxSizeUpload`.
fn upload_too_large() -> Response {
    payload_too_large("maxSizeUpload", MAX_UPLOAD_BYTES)
}

/// Forward an upload to the upstream JMAP server (proxy mode): fetch its Session for
/// the real uploadUrl template, substitute the accountId, POST the bytes with injected
/// auth + the client's `Content-Type`, and relay the upstream `{accountId, blobId,
/// type, size}` response back — the symmetric counterpart of [`proxy_download`].
///
/// The upstream's `Content-Type` is not relayed as received (t24 B2): a success is
/// labelled `application/json`, which RFC 8620 §6.1 says it is, and anything else
/// carries no upstream content type.
async fn proxy_upload(
    security: &SecurityConfig,
    session: &mw_store::Session,
    account_id: &str,
    content_type: &str,
    bytes: Bytes,
) -> Response {
    let upstream = match upstream_session(security, &session.jmap_url, &session.credentials).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("upstream session for upload failed: {e}");
            return upstream_error();
        }
    };
    let url = upstream
        .upload_url
        .replace("{accountId}", &percent_encode(account_id));
    let abs = resolve_api_url(&session.jmap_url, &url);
    let client =
        match upstream_client(security, &session.jmap_url, &abs, &session.credentials).await {
            Ok(c) => c,
            Err(r) => {
                tracing::warn!("upstream upload refused: {r}");
                return upstream_error();
            }
        };
    match client.post_bytes(&abs, content_type, bytes).await {
        Ok((status, _upstream_type, body)) => {
            let code = StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let mut resp = Response::new(Body::from(body));
            *resp.status_mut() = code;
            if code.is_success() {
                resp.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
            }
            resp
        }
        Err(e) => {
            tracing::warn!("upload proxy failed: {e}");
            upstream_error()
        }
    }
}

// ---------------------------------------------------------------------------
// V3 PIM endpoints (plan §3 e9): Mailwoman-native calendar / address-book
// sharing (ACL-checked, read-only) + the bundled holiday feed. All cookie-authed
// like every other endpoint; the sharing data path rides the frozen engine PIM
// surface (`handle_jmap`), so it lights up when e8 fills `dispatch_pim`.
// ---------------------------------------------------------------------------

/// A clean `501` for a PIM feature that requires the local engine store (a proxy
/// upstream has no Mailwoman-native collections to serve).
fn requires_engine_mode(feature: &str) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({ "error": format!("{feature} requires engine mode") })),
    )
        .into_response()
}

/// `GET /dav/calendars/{accountId}/{calendarId}` — serve a Mailwoman-native
/// calendar collection to a grantee principal per `calendar_shares` (on-server
/// ACL sharing, §11). The owner reads their own collection; a grantee with
/// `read`/`readWrite` may fetch; everyone else is `403`.
async fn caldav_share(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath((account_id, calendar_id)): UrlPath<(String, String)>,
) -> Response {
    let session = match authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let Some(engine) = &state.engine else {
        return requires_engine_mode("calendar sharing");
    };
    sharing::serve_shared_calendar(
        engine,
        &account_id,
        &calendar_id,
        &session.account_id,
        &session.username,
    )
    .await
}

/// `GET /dav/addressbooks/{accountId}/{addressBookId}` — serve a Mailwoman-native
/// address-book collection (§13). Owner-only in V3 (the frozen model has no
/// address-book share-ACL; see [`sharing`]).
async fn carddav_share(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath((account_id, address_book_id)): UrlPath<(String, String)>,
) -> Response {
    let session = match authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let Some(engine) = &state.engine else {
        return requires_engine_mode("address-book sharing");
    };
    sharing::serve_shared_addressbook(engine, &account_id, &address_book_id, &session.account_id)
        .await
}

/// `GET /api/holidays` — the list of subscribable holiday regions (§11).
async fn holiday_regions(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(resp) = authed(&state, &headers).await {
        return resp;
    }
    holidays::regions_response()
}

/// `GET /api/holidays/{region}` — a bundled holiday pack as ICS (§11).
async fn holiday_feed(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(region): UrlPath<String>,
) -> Response {
    if let Err(resp) = authed(&state, &headers).await {
        return resp;
    }
    holidays::feed_response(&region)
}

/// `GET /api/export/{stableId}?format=eml|mbox|txt|md` — export one message
/// through `mw-export`. Engine-mode only (the raw body lives in the local
/// cache); proxy mode has no server-side store to export from, so it returns
/// `501`. `format` defaults to `eml`.
async fn export_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(stable_id): UrlPath<String>,
    Query(q): Query<ExportQuery>,
) -> Response {
    let session = match user_router::feature_session(&state, &headers, "export").await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    if let Err(response) = step_up::require_user(&state, &session, step_up::StepUpAction::DataExport).await {
        return response;
    }
    let attachment_policy=match attachment_policy::effective(&state,&session).await {Ok(p)=>p,Err(r)=>return r};
    if attachment_policy.restricts_content() {return attachment_policy::refusal("attachmentContainerRestricted");}
    let Some(engine) = &state.engine else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({ "error": "export requires engine mode" })),
        )
            .into_response();
    };
    let Some((format, content_type, ext)) = export_format(q.format.as_deref().unwrap_or("eml"))
    else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "unknown export format" })),
        )
            .into_response();
    };
    // The whole-message blob path yields the raw RFC822 bytes mw-export consumes.
    let raw = match engine.fetch_blob(&session.account_id, &stable_id).await {
        Ok(Some(blob)) => blob.bytes,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "message not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::warn!("export blob fetch failed: {e}");
            return upstream_error();
        }
    };
    match mw_export::export_one(&mw_export::RawEmail::new(raw), format) {
        Ok(out) => blob_response(content_type, &format!("{stable_id}.{ext}"), out, &headers),
        Err(e) => {
            tracing::warn!("export failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "export error").into_response()
        }
    }
}

/// Query string for [`export_message`].
#[derive(Debug, Deserialize)]
struct ExportQuery {
    format: Option<String>,
}

/// Map an export `format` query value to `(Format, content-type, extension)`.
fn export_format(name: &str) -> Option<(mw_export::Format, &'static str, &'static str)> {
    match name {
        "eml" => Some((mw_export::Format::Eml, "message/rfc822", "eml")),
        "mbox" => Some((mw_export::Format::Mbox, "application/mbox", "mbox")),
        "txt" | "text" => Some((mw_export::Format::Txt, "text/plain; charset=utf-8", "txt")),
        "md" | "markdown" => Some((
            mw_export::Format::Markdown,
            "text/markdown; charset=utf-8",
            "md",
        )),
        // t16 W7: the MS-OXMSG `.msg` (CFB) and Office Open XML `.docx` writers
        // (`msg::to_msg` / `docx::to_docx`) exist + are tested — expose them here.
        "msg" => Some((mw_export::Format::Msg, "application/vnd.ms-outlook", "msg")),
        "docx" => Some((
            mw_export::Format::Docx,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "docx",
        )),
        _ => None,
    }
}

/// What a `Range` request header asks for, resolved against the real length.
#[derive(Debug, PartialEq, Eq)]
enum RangeDecision {
    /// Send the whole representation with `200`.
    Full,
    /// Send `bytes[start..=end]` with `206`.
    Partial { start: u64, end: u64 },
    /// Every named range lies outside the representation: `416`.
    Unsatisfiable,
}

/// Resolve a `Range` header against a representation of `total` bytes
/// (RFC 9110 §14.1.2, §14.2, §15.4.17).
///
/// Deliberate, stated limits — each is a permitted behaviour, not an oversight:
/// * only the `bytes` unit is understood; any other unit is ignored (`Full`);
/// * a **malformed or invalid** spec (`last < first`, non-numeric, a bare `-`)
///   is ignored rather than rejected, which is what §14.2 requires of an origin
///   server;
/// * a **multi-range** request is answered with the whole representation. A
///   server MAY ignore `Range`, and `multipart/byteranges` buys nothing for the
///   two uses we have (resume, and a media element seeking), so it is not built.
///
/// A zero-length blob is `Unsatisfiable` for any byte range, since no range can
/// overlap an empty extent.
fn parse_range(req: &HeaderMap, total: u64) -> RangeDecision {
    let Some(raw) = req.get(header::RANGE).and_then(|v| v.to_str().ok()) else {
        return RangeDecision::Full;
    };
    let Some((unit, spec)) = raw.trim().split_once('=') else {
        return RangeDecision::Full;
    };
    if !unit.trim().eq_ignore_ascii_case("bytes") || spec.contains(',') {
        return RangeDecision::Full;
    }
    let Some((first, last)) = spec.trim().split_once('-') else {
        return RangeDecision::Full;
    };
    let last_byte = total.saturating_sub(1);
    let (start, end) = match (first.trim(), last.trim()) {
        // A bare `-` names nothing.
        ("", "") => return RangeDecision::Full,
        // Suffix range: the final N bytes. A representation shorter than the
        // suffix yields the whole of it (§14.1.2); a zero-length suffix names
        // nothing that exists.
        ("", suffix) => {
            let Ok(n) = suffix.parse::<u64>() else {
                return RangeDecision::Full;
            };
            if n == 0 {
                return RangeDecision::Unsatisfiable;
            }
            (total.saturating_sub(n), last_byte)
        }
        // Open-ended: from `start` to the end.
        (first, "") => {
            let Ok(start) = first.parse::<u64>() else {
                return RangeDecision::Full;
            };
            (start, last_byte)
        }
        (first, last) => {
            let (Ok(start), Ok(last)) = (first.parse::<u64>(), last.parse::<u64>()) else {
                return RangeDecision::Full;
            };
            if last < start {
                return RangeDecision::Full;
            }
            (start, last.min(last_byte))
        }
    };
    if total == 0 || start >= total {
        return RangeDecision::Unsatisfiable;
    }
    RangeDecision::Partial { start, end }
}

/// Apply [`parse_range`] to an already-materialised body, returning the status,
/// the slice to send and the `Content-Range` value the status requires.
///
/// The slice is a refcounted view of the same allocation, so answering a range
/// costs no extra copy — but note what this does **not** do: the caller has
/// already read the entire blob into memory, so peak resident memory per
/// concurrent download stays equal to the full blob size no matter how small the
/// requested range is. Range support here buys resumability and seeking, not
/// streaming. Real streaming needs a chunked reader through the store and the
/// upstream client, which is a larger change than this.
fn apply_range(bytes: Bytes, req: &HeaderMap) -> (StatusCode, Bytes, Option<String>) {
    let total = bytes.len() as u64;
    match parse_range(req, total) {
        RangeDecision::Full => (StatusCode::OK, bytes, None),
        RangeDecision::Partial { start, end } => (
            StatusCode::PARTIAL_CONTENT,
            bytes.slice(start as usize..=end as usize),
            Some(format!("bytes {start}-{end}/{total}")),
        ),
        // §15.4.17: an empty body plus the unsatisfied-range form, so the client
        // learns the current length and can re-ask.
        RangeDecision::Unsatisfiable => (
            StatusCode::RANGE_NOT_SATISFIABLE,
            Bytes::new(),
            Some(format!("bytes */{total}")),
        ),
    }
}

/// The policy every download carries next to the shell's (t32-h4). A download
/// opened as a page, directly or from the `blob:` URL the client makes of it, is
/// an opaque-origin document that can load nothing, so an HTML or SVG
/// attachment cannot run as this origin even if its type were mislabelled.
const DOWNLOAD_CSP: &str = "sandbox; default-src 'none'";

/// Build a download response with `Content-Type`, `Content-Disposition`
/// (attachment), `Content-Length`, `Accept-Ranges: bytes`, `nosniff` and
/// [`DOWNLOAD_CSP`] set, honouring a single `Range` request with `206` (t20 B9).
///
/// `Accept-Ranges` is advertised unconditionally on this path because every
/// caller holds the complete bytes. See [`apply_range`] for what range support
/// does and does not fix — the read is still buffered.
fn blob_response(content_type: &str, filename: &str, bytes: Vec<u8>, req: &HeaderMap) -> Response {
    let (status, body, content_range) = apply_range(Bytes::from(bytes), req);
    let len = body.len();
    let mut resp = Response::new(Body::from(body));
    *resp.status_mut() = status;
    let h = resp.headers_mut();
    h.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    h.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(DOWNLOAD_CSP),
    );
    h.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    // A 416 describes the mismatch; it carries no representation, so the blob's
    // own type and filename would be misleading on it.
    if status != StatusCode::RANGE_NOT_SATISFIABLE {
        if let Ok(v) = HeaderValue::from_str(content_type) {
            h.insert(header::CONTENT_TYPE, v);
        }
        let disposition = format!("attachment; filename=\"{}\"", sanitize_filename(filename));
        if let Ok(v) = HeaderValue::from_str(&disposition) {
            h.insert(header::CONTENT_DISPOSITION, v);
        }
    }
    if let Some(cr) = content_range.and_then(|v| HeaderValue::from_str(&v).ok()) {
        h.insert(header::CONTENT_RANGE, cr);
    }
    h.insert(header::CONTENT_LENGTH, HeaderValue::from(len));
    resp
}

/// Strip characters that would break a `Content-Disposition` filename token
/// (quotes, control chars, path separators) so the header stays well-formed.
fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '"' | '\\' | '/' | '\r' | '\n') {
                '_'
            } else {
                c
            }
        })
        .collect();
    if cleaned.trim().is_empty() {
        "download".to_string()
    } else {
        cleaned
    }
}

/// Percent-encode a path segment for substitution into an upstream URL template
/// (RFC 3986 unreserved set passes through; everything else becomes `%XX`).
fn percent_encode(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for b in segment.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push('%');
            out.push_str(&format!("{b:02X}"));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// V4 crypto/security endpoints (plan §3 e7): WKD publishing, ARF report
// submission, DLP config load, + the web watermark honesty overlay toggle. Route
// seams reserved by e0; e7 fills them (and forwards CryptoKey/MailRule
// StateChange over push). All return a clean 501 until then rather than falling
// through to the SPA index.html.
// ---------------------------------------------------------------------------

/// `GET /.well-known/openpgpkey/hu/{hash}` — WKD (Web Key Directory) **direct
/// method** (§7.3 / plan §3 e7): serves an own PUBLIC key by z-base-32 hashed
/// local-part. The mail domain is taken from the `Host` header. PUBLIC (no cookie)
/// so external clients can fetch keys — only keys the operator has published in
/// `MW_WKD_DIR` are served.
async fn wkd_lookup(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(hash): UrlPath<String>,
) -> Response {
    match host_domain(&headers) {
        Some(domain) => wkd_serve(&state, &domain, &hash),
        None => wkd_not_found(),
    }
}

/// `GET /.well-known/openpgpkey/{domain}/hu/{hash}` — WKD **advanced method**: the
/// mail domain is explicit in the path (served from the `openpgpkey.<domain>`
/// vhost). PUBLIC, same published-key source as the direct method.
async fn wkd_lookup_advanced(
    State(state): State<AppState>,
    UrlPath((domain, hash)): UrlPath<(String, String)>,
) -> Response {
    wkd_serve(&state, &domain, &hash)
}

/// Serve a published WKD key for `(domain, hash)` as a binary transferable key.
fn wkd_serve(state: &AppState, domain: &str, hash: &str) -> Response {
    if !wkd::valid_domain(domain) || !wkd::valid_hash(hash) {
        return wkd_not_found();
    }
    let Some(dir) = &state.security.wkd_dir else {
        return wkd_not_found();
    };
    match wkd::WkdDirectory::new(dir.clone()).lookup(domain, hash) {
        Some(bytes) => {
            let mut resp = Response::new(Body::from(bytes));
            let h = resp.headers_mut();
            h.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/octet-stream"),
            );
            // WKD clients may fetch cross-origin (openpgpkey.<domain> vhost).
            h.insert(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                HeaderValue::from_static("*"),
            );
            resp
        }
        None => wkd_not_found(),
    }
}

/// WKD 404 (unknown/unpublished key), with the permissive CORS header WKD clients
/// expect on every openpgpkey response.
fn wkd_not_found() -> Response {
    let mut resp = (StatusCode::NOT_FOUND, "not found").into_response();
    resp.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    resp
}

/// `GET /.well-known/openpgpkey[/{domain}]/policy` — the WKD policy file. Its mere
/// existence (empty body, 200) signals a WKD-enabled domain; no flags are set.
async fn wkd_policy() -> Response {
    let mut resp = ([(header::CONTENT_TYPE, "text/plain")], "").into_response();
    resp.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    resp
}

/// The mail domain for the direct WKD method: the `Host` authority with any port
/// stripped and the advanced-method `openpgpkey.` vhost prefix removed.
fn host_domain(headers: &HeaderMap) -> Option<String> {
    let host = headers.get(header::HOST)?.to_str().ok()?;
    let host = host.split(':').next().unwrap_or(host);
    let domain = host.strip_prefix("openpgpkey.").unwrap_or(host);
    Some(domain.to_lowercase())
}

/// `POST /api/security/report {emailId, kind, note?}` — ARF (RFC 5965) abuse
/// report (§7.3 sender-controls / plan §3 e7). Cookie-authed. Builds a valid
/// feedback report wrapping the reported message and relays it to
/// `MW_ABUSE_ADDRESS` (spooled to `MW_ABUSE_SPOOL` when configured). Engine mode
/// only (the reported message's raw bytes come from the local store).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArfReportReq {
    email_id: String,
    kind: String,
    #[serde(default)]
    note: Option<String>,
}

async fn arf_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ArfReportReq>,
) -> Response {
    let session = match authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let Some(abuse) = state.security.abuse_address.clone() else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({ "error": "abuse reporting is not configured (set MW_ABUSE_ADDRESS)" })),
        )
            .into_response();
    };
    let Some(kind) = arf::FeedbackKind::from_token(&body.kind) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "kind must be 'phishing' or 'junk'" })),
        )
            .into_response();
    };
    let Some(engine) = &state.engine else {
        return requires_engine_mode("abuse reporting");
    };
    // The report wraps the ORIGINAL reported message (fetched from the local store).
    let raw = match engine.fetch_blob(&session.account_id, &body.email_id).await {
        Ok(Some(blob)) => blob.bytes,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "reported message not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::warn!("ARF blob fetch failed: {e}");
            return upstream_error();
        }
    };
    let reporting_domain = session.username.rsplit('@').next().unwrap_or_default();
    let report = arf::build_report(
        kind,
        &session.username,
        &abuse,
        reporting_domain,
        &raw,
        body.note.as_deref(),
    );
    // Relay: spool for pickup when configured; otherwise the report is generated
    // and logged (direct SMTP submission via the account MailSubmitter is the
    // engine's job, plan §1.9 — it lands with SenderControl/set in e6).
    let relayed = match &state.security.abuse_spool {
        Some(dir) => match arf::spool(dir, &report) {
            Ok(path) => {
                tracing::info!("ARF report spooled to {}", path.display());
                true
            }
            Err(e) => {
                tracing::error!("ARF spool failed: {e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "failed to spool abuse report" })),
                )
                    .into_response();
            }
        },
        None => {
            tracing::info!(
                "ARF report generated for {abuse} ({} bytes); no spool configured",
                report.len()
            );
            false
        }
    };
    (
        StatusCode::ACCEPTED,
        Json(json!({
            "ok": true,
            "feedbackType": kind.as_str(),
            "abuseAddress": abuse,
            "reportSize": report.len(),
            "relayed": relayed,
        })),
    )
        .into_response()
}

/// `GET /api/security/dlp/config` — DLP config load (§7.6 / plan §3 e7): the
/// active rules, `MW_DLP_RULES` and the admin-stored ones, each marked with its
/// `source` ([`dlp::active_rules`], the same union the engine enforces), so the
/// web can name them pre-send. Cookie-authed.
async fn dlp_config(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(resp) = authed(&state, &headers).await {
        return resp;
    }
    let rules = match dlp::active_rules(&state.store, state.security.dlp_rules.as_deref()).await {
        Ok(rules) => rules,
        Err(e) => {
            tracing::error!("MW_DLP_RULES failed to load: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "DLP rules are misconfigured" })),
            )
                .into_response();
        }
    };
    Json(json!({ "list": rules, "count": rules.len() })).into_response()
}

/// `GET /api/security/watermark` — the honest screen-capture watermark config
/// (§7.6 / plan §3 e7 / risk #13). Cookie-authed; returns the flag + the viewer's
/// identity to tile + the mandatory [`watermark::HONEST_NOTE`]. The overlay is a
/// deterrent, never a protection guarantee.
async fn watermark_config(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    Json(state.security.watermark.payload(&session.username)).into_response()
}

// ---------------------------------------------------------------------------
// Sanitize (via the mw-render child — the SPEC §7.5 boundary)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct SanitizeReq {
    html: String,
}

async fn sanitize(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SanitizeReq>,
) -> Response {
    if let Err(resp) = authed(&state, &headers).await {
        return resp;
    }
    let clean = match &state.render_bin {
        Some(bin) => match run_render_child(bin, &body.html).await {
            Ok(html) => html,
            Err(e) if state.render_jail_required => {
                // S6/DQ4 fail-closed: the jailed child was expected but failed. Do NOT
                // sanitize hostile HTML in this trusted process without the jail.
                tracing::error!(
                    "render child failed ({e}) and a kernel jail is required; \
                     refusing in-process fallback (SPEC §7.5)"
                );
                return render_unavailable();
            }
            Err(e) => {
                tracing::warn!(
                    "render child failed ({e}); degraded in-process sanitize (no jail expected)"
                );
                mw_sanitize::sanitize_email_html(&body.html)
            }
        },
        // No render worker at all: fail closed when a jail was expected, else degrade.
        None if state.render_jail_required => {
            tracing::error!(
                "no render worker and a kernel jail is required; refusing /api/sanitize (SPEC §7.5)"
            );
            return render_unavailable();
        }
        None => mw_sanitize::sanitize_email_html(&body.html),
    };
    // `html` is the existing contract; `csp` is additive — the web app may apply
    // it to the per-message iframe. Existing consumers read only `html`.
    Json(json!({ "html": clean, "csp": MESSAGE_CSP })).into_response()
}

/// `POST /api/import/oft {contentBase64}` — import an untrusted `.oft`/`.msg`
/// template. The hostile CFB parse runs in the disposable `mw-render` child (the
/// §7.5 boundary, plan §3 e14/e5) via a `Cfb` [`mw_render::Job`]; the child returns
/// the sanitized body + subject. Falls back to an in-process parse only when no
/// render worker is present (documented, mirrors [`sanitize`]). Cookie-authed; the
/// composer fills a new draft from the returned fields.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportOftReq {
    content_base64: String,
}

async fn import_oft(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ImportOftReq>,
) -> Response {
    if let Err(resp) = authed(&state, &headers).await {
        return resp;
    }
    let result = match &state.render_bin {
        Some(bin) => run_render_child_cfb(bin, &body.content_base64).await,
        None => {
            // S6/DQ4 + SPEC §7.5: the hostile OLE2/MS-OXMSG parse has NO safe
            // in-process path — the native `from_oft` parser was removed from the
            // runtime path (it survives only as a test-fixture writer). The CFB parse
            // runs only inside the render child's WASM media jail. Never parse a
            // compound file natively in this trusted process — refuse.
            tracing::error!(
                "no render worker; refusing to import .oft/.msg in-process \
                 (the compound-file parse requires the WASM media jail — SPEC §7.5)"
            );
            return render_unavailable();
        }
    };
    match result {
        Ok((html, subject)) => Json(json!({
            "html": html,
            "subject": subject,
            "csp": MESSAGE_CSP,
        }))
        .into_response(),
        Err(e) => {
            tracing::warn!("oft import failed: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "error": "could not import the template" })),
            )
                .into_response()
        }
    }
}

/// The S6/DQ4 fail-closed response: a render kernel jail was expected but the jailed
/// child is unavailable, so we refuse rather than process hostile input unconfined in
/// this trusted process. Plain, no-hype (`503`).
fn render_unavailable() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "error": "the render worker is unavailable; \
                      hostile content is not processed without its sandbox"
        })),
    )
        .into_response()
}

/// Spawn `mw-render` with a `Cfb` job and read back the imported `(html, subject)`.
async fn run_render_child_cfb(
    bin: &Path,
    cfb_base64: &str,
) -> anyhow::Result<(String, Option<String>)> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let mut child = tokio::process::Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("no child stdin"))?;
    let job = serde_json::to_string(&json!({ "cfbBase64": cfb_base64 }))?;
    stdin.write_all(job.as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.shutdown().await?;
    drop(stdin);

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("no child stdout"))?;
    let mut line = String::new();
    BufReader::new(stdout).read_line(&mut line).await?;
    let status = child.wait().await?;
    if !status.success() {
        return Err(anyhow!("render child exited unsuccessfully: {status}"));
    }
    let out: serde_json::Value = serde_json::from_str(line.trim_end())?;
    let html = out
        .get("html")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let subject = out
        .get("subject")
        .and_then(|v| v.as_str())
        .map(String::from);
    Ok((html, subject))
}

/// Spawn `mw-render`, write one job line, read one output line, wait, exit.
async fn run_render_child(bin: &Path, html: &str) -> anyhow::Result<String> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let mut child = tokio::process::Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("no child stdin"))?;
    let job = serde_json::to_string(&json!({ "html": html }))?;
    stdin.write_all(job.as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.shutdown().await?;
    drop(stdin);

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("no child stdout"))?;
    let mut line = String::new();
    BufReader::new(stdout).read_line(&mut line).await?;

    let status = child.wait().await?;
    if !status.success() {
        return Err(anyhow!("render child exited unsuccessfully: {status}"));
    }
    let out: serde_json::Value = serde_json::from_str(line.trim_end())?;
    Ok(out
        .get("html")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string())
}

fn locate_render_bin() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("MW_RENDER_BIN") {
        let pb = PathBuf::from(p);
        if pb.exists() {
            return Some(pb);
        }
    }
    let name = if cfg!(windows) {
        "mw-render.exe"
    } else {
        "mw-render"
    };
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let here = dir.join(name);
    if here.exists() {
        return Some(here);
    }
    // `cargo test` builds the test binary under target/<profile>/deps/, while
    // the worker lands one level up in target/<profile>/.
    let up = dir.parent()?.join(name);
    if up.exists() {
        return Some(up);
    }
    None
}

// ---------------------------------------------------------------------------
// Static assets / SPA fallback
// ---------------------------------------------------------------------------

/// The generated one-line script that publishes the deploy prefix to the SPA
/// (t20 B4). Served as a file rather than inlined into `index.html` because the
/// shell [`CSP`] carries `script-src 'self'` with no `'unsafe-inline'` — an
/// inline `<script>` would be blocked and the SPA would silently fall back to
/// root-absolute paths under a prefix, which is the exact failure this closes.
const BASE_SHIM_PATH: &str = "__mw_base.js";

async fn static_handler(
    State(state): State<AppState>,
    method: Method,
    OriginalUri(original): OriginalUri,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    // t20 B4: the bare prefix must redirect to the trailing-slash form. The SPA is
    // built with a relative asset base, so `./assets/index-*.js` in a document
    // served at `/mail` resolves to `/assets/…` — outside the prefix, and outside
    // what a proxy forwarding only `/mail/*` will even route. `OriginalUri` is the
    // pre-nesting path, so this sees `/mail` where `uri` already reads `/`.
    if let Some(resp) = redirect_to_base_slash(&original, &state.base_path) {
        return resp;
    }
    let base: &str = &state.base_path;
    // The path relative to the prefix. A nested request already reads that way in
    // `uri`; `{base}/` itself reaches the root copy of the router with the prefix
    // still on, so it is taken from `OriginalUri` whenever it is under the prefix.
    let app_path = match match_base_path(original.path(), base) {
        BaseMatch::Under(rest) => rest,
        _ => uri.path(),
    };
    let raw = app_path.trim_start_matches('/');
    let path = if raw.is_empty() { "index.html" } else { raw };
    let error = |kind| error_pages::respond(kind, &method, &headers, base, path);

    // t32-h10: a path that could climb out of the web directory or name something
    // other than a plain file (`..`, `\`, `:`, NUL, encoded or overlong forms of
    // them) is refused before it reaches the filesystem.
    if error_pages::is_unsafe_asset_path(path) {
        return error(error_pages::ErrorKind::BadRequest);
    }
    // The base shim is generated, not bundled: served at every prefix (including
    // none) so an `index.html` cached from a previously-prefixed deployment never
    // hits a 404 here.
    if path == BASE_SHIM_PATH {
        return base_shim_response(base, &headers);
    }
    if path == error_pages::STYLESHEET_PATH {
        return error_pages::stylesheet_response(&headers);
    }
    if let Some(resp) = serve_asset(&state, path, &headers) {
        return resp;
    }
    if let Some(resp) = redirect_page_trailing_slash(&original, app_path, base) {
        return resp;
    }
    // SPA fallback: one of the app's own pages gets index.html; everything else
    // gets the error answer, which is the designed page for a browser navigation
    // and JSON otherwise. See [`wants_spa_shell`]: answering a subresource or a
    // non-GET with `200 text/html` is what made a missing webfont arrive as `<!DO…`
    // and an unmounted `POST /mcp` arrive as HTML a JSON client then choked on;
    // answering an unknown page with the shell was a soft 404 (t32-h10).
    if !is_app_page(path) || !wants_spa_shell(&method, &headers, path) {
        request_protection::admin_route_miss(app_path);
        return error(error_pages::ErrorKind::NotFound);
    }
    let shell = if path == console_page() {
        serve_console_shell(&state, &headers)
    } else if path == "oidc/consent" {
        oidc_provider::shell(&state, &headers)
    } else {
        serve_asset(&state, "index.html", &headers)
    };
    shell.unwrap_or_else(|| error(error_pages::ErrorKind::Unavailable))
}

/// The admin console's page relative to the deploy prefix, without the leading
/// `/`: `admin` unless `MW_ADMIN_PATH` moved it (t32-h15). Read from the one helper,
/// [`mw_admin::config::admin_path`].
fn console_page() -> String {
    mw_admin::config::admin_path()
        .trim_start_matches('/')
        .to_string()
}

/// The pages the SPA itself serves, relative to the deploy prefix. The app routes
/// by URL fragment (`/#/calendar`, `{console}#/users`), so these are the only paths
/// at which the shell is a real page; at any deeper path its relative asset URLs
/// (`./assets/…`) would not even resolve.
fn is_app_page(path: &str) -> bool {
    matches!(path, "index.html" | "oauth/authorize" | "oidc/consent") || path == console_page()
}

/// `308 {prefix}{console}/` → `{prefix}{console}` (the console is `/admin` unless
/// `MW_ADMIN_PATH` moved it), and the same for `/oauth/authorize/`, keeping the
/// query (the browser keeps the fragment, so `/admin/#/users` lands on
/// `/admin#/users`). At the slash form the shell's relative asset URLs resolve one
/// level too deep and the page never starts (t32-h8, t32-h10). `None` otherwise,
/// which includes `/admin/` once the console has moved: that is an unknown path.
///
/// The prefix is put back only when the request came in under it; a request at the
/// origin root of a prefixed deployment is redirected at the root.
fn redirect_page_trailing_slash(original: &Uri, app_path: &str, base: &str) -> Option<Response> {
    let page = app_path.strip_suffix('/')?.trim_start_matches('/');
    if page != "oauth/authorize" && page != "oidc/consent" && page != console_page() {
        return None;
    }
    let prefix = match match_base_path(original.path(), base) {
        BaseMatch::Under(_) => base,
        _ => "",
    };
    let target = match original.query() {
        Some(q) => format!("{prefix}/{page}?{q}"),
        None => format!("{prefix}/{page}"),
    };
    let loc = HeaderValue::from_str(&target).ok()?;
    let mut resp = StatusCode::PERMANENT_REDIRECT.into_response();
    resp.headers_mut().insert(header::LOCATION, loc);
    Some(resp)
}

/// Final path segments that only ever name a **subresource**. A request for one of
/// these is not a client-side route whatever headers it carries, so it is settled
/// before the header signals below — which is what makes a missing
/// `/fonts/inter-400.woff2` a 404 even for a client that sends no useful headers.
///
/// `.html` is deliberately absent: `index.html` is served directly above, and any
/// other `.html` is close enough to a document to be worth the shell.
const SUBRESOURCE_EXTENSIONS: &[&str] = &[
    "avif",
    "css",
    "eot",
    "gif",
    "ico",
    "jpeg",
    "jpg",
    "js",
    "json",
    "map",
    "mjs",
    "otf",
    "png",
    "svg",
    "ttf",
    "wasm",
    "webmanifest",
    "webp",
    "woff",
    "woff2",
    "xml",
];

/// May this unmatched request be answered with `index.html`?
///
/// The SPA fallback exists so a deep link like `/mail/inbox/42` reloads into the
/// app instead of 404ing. Applying it to *every* unmatched path is the defect:
/// a missing subresource then arrives with `200` and an HTML body, which the
/// browser reports as a corrupt asset (`OTS parsing error: invalid sfntVersion:
/// 1008821359` — those four bytes are `<!DO`) and a JSON client reports as
/// `Unexpected token '<'`. Both hide the real answer, which is "that is not here".
///
/// The discriminator, most reliable signal first:
///
/// 1. **Method.** Only `GET`/`HEAD` can be a navigation. A `POST` that reaches the
///    fallback is a call to a route that is not mounted, and must say so.
/// 2. **A subresource extension** on the last path segment — never a client-side
///    route, and the one check that does not depend on the client's headers.
/// 3. **Fetch Metadata**, which every current browser sends on same-origin requests
///    and which no other signal can override: `Sec-Fetch-Dest` names the consumer
///    directly (`document`/`iframe`/`frame`/`embed`/`object` are navigations,
///    `font`/`script`/`style`/`image`/`empty` are not), and `Sec-Fetch-Mode:
///    navigate` says the same thing for the handful of cases that send only it.
/// 4. **`Accept`**, for a client that sends no Fetch Metadata at all (curl, an HTTP
///    library, an old browser): the shell is served only if the client explicitly
///    asked for `text/html` or `application/xhtml+xml`.
///
/// **A client that sends none of these gets a 404**, including one sending `Accept:
/// */*` or no `Accept` at all. That is the deliberate direction to fail in: a real
/// browser navigation always sends both Fetch Metadata and `Accept: text/html`, so
/// the case this forgoes is a hand-written deep-link request from a CLI, where a
/// 404 is visible and diagnosable. The opposite error — a `200` HTML page in place
/// of a font, a chunk or a JSON reply — is silent and corrupts the caller.
fn wants_spa_shell(method: &Method, headers: &HeaderMap, path: &str) -> bool {
    if !matches!(*method, Method::GET | Method::HEAD) {
        return false;
    }
    if has_subresource_extension(path) {
        return false;
    }
    let navigate =
        header_str(headers, "sec-fetch-mode").map(|mode| mode.eq_ignore_ascii_case("navigate"));
    if let Some(dest) = header_str(headers, "sec-fetch-dest") {
        // A navigation a service worker answered with `fetch(event.request)` arrives
        // with `Sec-Fetch-Dest: empty` and `Sec-Fetch-Mode: navigate` (t32-h8).
        return matches!(dest, "document" | "iframe" | "frame" | "embed" | "object")
            || (dest == "empty" && navigate == Some(true));
    }
    if let Some(navigate) = navigate {
        return navigate;
    }
    accepts_html(headers)
}

/// The last path segment's extension, matched case-insensitively against
/// [`SUBRESOURCE_EXTENSIONS`].
fn has_subresource_extension(path: &str) -> bool {
    let last = path.rsplit('/').next().unwrap_or(path);
    let Some((_, ext)) = last.rsplit_once('.') else {
        return false;
    };
    SUBRESOURCE_EXTENSIONS
        .iter()
        .any(|known| ext.eq_ignore_ascii_case(known))
}

/// Does the client explicitly ask for an HTML document? `*/*` does not count — it
/// is what a fetch with no stated preference sends, which is the case this refuses.
fn accepts_html(headers: &HeaderMap) -> bool {
    let Some(accept) = header_str(headers, header::ACCEPT.as_str()) else {
        return false;
    };
    accept.split(',').any(|part| {
        let media = part.split(';').next().unwrap_or_default().trim();
        media.eq_ignore_ascii_case("text/html")
            || media.eq_ignore_ascii_case("application/xhtml+xml")
    })
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name)?.to_str().ok()
}

/// `308 {base}` → `{base}/` (t20 B4). `None` for every other request.
///
/// 308 rather than 301/302 so the method and body survive the redirect, and
/// because serving the shell only at the trailing-slash form is a permanent
/// property of the deployment rather than a routing accident.
fn redirect_to_base_slash(original: &Uri, base: &str) -> Option<Response> {
    if !matches!(match_base_path(original.path(), base), BaseMatch::Exact) {
        return None;
    }
    let target = match original.query() {
        Some(q) => format!("{base}/?{q}"),
        None => format!("{base}/"),
    };
    let loc = HeaderValue::from_str(&target).ok()?;
    let mut resp = StatusCode::PERMANENT_REDIRECT.into_response();
    resp.headers_mut().insert(header::LOCATION, loc);
    Some(resp)
}

/// `GET {base}/__mw_base.js` — publish the prefix as `window.__MW_BASE__`, which
/// `apps/web/src/api/basePath.ts` reads (and canonicalizes) to build every
/// `fetch`/`WebSocket`/`EventSource` target and the service-worker scope.
fn base_shim_response(base: &str, req: &HeaderMap) -> Response {
    asset_response(
        BASE_SHIM_PATH,
        "text/javascript; charset=utf-8",
        base_shim_body(base).into_bytes(),
        req,
    )
}

/// The shim's body. `base` is `""` or a `normalize_base_path`-validated prefix,
/// so it cannot contain a quote, a backslash or `<` — there is nothing to escape
/// and no way to break out of the string literal.
fn base_shim_body(base: &str) -> String {
    format!("window.__MW_BASE__=\"{base}/\";\n")
}

fn serve_asset(state: &AppState, path: &str, req: &HeaderMap) -> Option<Response> {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let bytes = adapt_asset_to_base(path, read_asset(state, path)?, &state.base_path);
    let mut resp = asset_response(path, mime.as_ref(), bytes, req);
    allow_worker_scope(&mut resp, path, &state.base_path);
    Some(resp)
}

/// The bytes of one file of the web build, from `MW_WEB_DIR` when set, else from
/// the bundle embedded at compile time.
fn read_asset(state: &AppState, path: &str) -> Option<Vec<u8>> {
    if let Some(dir) = &state.web_dir {
        // Only plain path components may reach `join`. The old guard,
        // `dir.join(path).starts_with(dir)`, compared components lexically, so
        // `../x` (and `..\x` on Windows) passed it and a raw `GET /../x` read a
        // file outside the web directory (t32-h10).
        if error_pages::is_unsafe_asset_path(path)
            || !Path::new(path)
                .components()
                .all(|c| matches!(c, std::path::Component::Normal(_)))
        {
            return None;
        }
        return std::fs::read(dir.join(path)).ok();
    }
    Some(WebAssets::get(path)?.data.into_owned())
}

/// The marker [`serve_console_shell`] puts in the console page, and only there.
/// `apps/web/src/api/adminPath.ts` reads it; with it, the web app takes its own
/// address as the console's location and builds every admin URL from that.
const CONSOLE_MARKER: &str = r#"<meta name="mw-console" content="admin">"#;

/// The admin console page: `index.html` adapted to the prefix like the mail shell,
/// plus [`CONSOLE_MARKER`], with its `./` file references made prefix-absolute
/// (t32-h15).
///
/// The marker is what keeps `MW_ADMIN_PATH` out of every other response: the web
/// build cannot know the path, and nothing the server sends outside the console
/// names it, so the page must be told it is the console rather than where the
/// console is. A `<meta name>` is inert under the shell [`CSP`]; an inline script
/// carrying the same fact would be blocked.
///
/// The `./` rewrite is for an admin path of more than one segment: at
/// `/mail/ops/console`, `./assets/…` would resolve under `/mail/ops/`. Chunks
/// imported from the entry resolve against the entry's own URL, so the entry and
/// the stylesheet in this document are the only references that need it.
fn serve_console_shell(state: &AppState, req: &HeaderMap) -> Option<Response> {
    let base: &str = &state.base_path;
    let bytes = adapt_asset_to_base("index.html", read_asset(state, "index.html")?, base);
    let html = match String::from_utf8(bytes) {
        Ok(html) => mark_console_shell(&html, base),
        // Not UTF-8 ⇒ not the shell we shipped; there is no console to serve.
        Err(_) => return None,
    };
    Some(asset_response(
        "index.html",
        "text/html",
        html.into_bytes(),
        req,
    ))
}

/// [`CONSOLE_MARKER`] at the top of `<head>`, and `="./` made `="{base}/`.
fn mark_console_shell(html: &str, base: &str) -> String {
    let mut out = html
        .replace("src=\"./", &format!("src=\"{base}/"))
        .replace("href=\"./", &format!("href=\"{base}/"));
    match out.find("<head>") {
        Some(i) => out.insert_str(i + "<head>".len(), CONSOLE_MARKER),
        None => out.insert_str(0, CONSOLE_MARKER),
    }
    out
}

/// Send `Service-Worker-Allowed` with the service worker (t20 B4).
///
/// The SPA registers `{base}/sw.js` with `scope: {base}/`, which is that script's
/// own default maximum scope, so strictly the header is not required. It is sent
/// anyway so the registration keeps working if the script ever moves (e.g. into
/// the hashed `assets/` directory), where the default scope would no longer cover
/// the shell and the registration would be rejected.
fn allow_worker_scope(resp: &mut Response, path: &str, base: &str) {
    // Root deployments emit exactly the headers they did before B4: `/` is already
    // the default scope there, so the header would say nothing.
    if base.is_empty() || (path != "sw.js" && !path.ends_with("/sw.js")) {
        return;
    }
    if let Ok(v) = HeaderValue::from_str(&format!("{base}/")) {
        resp.headers_mut().insert("service-worker-allowed", v);
    }
}

/// Adapt a bundled asset to the deploy prefix at serve time (t20 B4). Byte-for-byte
/// identity at the origin root, so a root deployment serves exactly what was built.
///
/// Two assets need it; nothing else in `dist/` carries a root-absolute URL (the
/// relative Vite base already handles chunks, workers, wasm and the `public/`
/// `url()` references):
///
///   * **`index.html`** gains the [`BASE_SHIM_PATH`] script tag.
///   * **The bundled CSS** carries eight `url(/fonts/*.woff2)` references. Those
///     stay root-absolute through the build because `/fonts/` is *not* a `public/`
///     directory — the files are produced by `mailwoman fonts pull` and land in
///     the served tree afterwards, so Vite cannot see them and cannot rewrite them
///     the way it rewrote `/themes/`. Rewriting the eight `url()`s here is the
///     fix that does not require the operator's font directory to be knowable at
///     build time. (`@font-face` carries `local()` fallbacks, so the pre-fix
///     failure mode was a silent downgrade to system fonts, not a broken page.)
fn adapt_asset_to_base(path: &str, bytes: Vec<u8>, base: &str) -> Vec<u8> {
    if base.is_empty() {
        return bytes;
    }
    if path == "index.html" || path.ends_with("/index.html") {
        return inject_base_shim(bytes, base);
    }
    if path.ends_with(".css") {
        return prefix_font_urls(bytes, base);
    }
    bytes
}

/// Insert the base-shim script into `<head>`, before the module entry.
///
/// The entry is `<script type="module">`, which is deferred by definition, so a
/// classic script anywhere in the document runs first — but placing it at the top
/// of `<head>` keeps that independent of where the bundler put the entry tag.
/// The `src` is written prefix-absolute (not `./`) because the server knows the
/// prefix and a deep-linked SPA fallback would otherwise resolve `./` against the
/// wrong directory.
fn inject_base_shim(bytes: Vec<u8>, base: &str) -> Vec<u8> {
    let mut html = match String::from_utf8(bytes) {
        Ok(s) => s,
        // Not UTF-8 ⇒ not the shell we shipped; serve it untouched.
        Err(e) => return e.into_bytes(),
    };
    let tag = format!("<script src=\"{base}/{BASE_SHIM_PATH}\"></script>");
    match html.find("<head>") {
        Some(i) => html.insert_str(i + "<head>".len(), &tag),
        None => html.insert_str(0, &tag),
    }
    html.into_bytes()
}

/// Prefix the `url(/fonts/…)` references in bundled CSS with the deploy base.
/// Handles the three quoting forms a CSS `url()` can take; the production build
/// minifies to the unquoted one.
fn prefix_font_urls(bytes: Vec<u8>, base: &str) -> Vec<u8> {
    let Ok(css) = std::str::from_utf8(&bytes) else {
        return bytes;
    };
    if !css.contains("/fonts/") {
        return bytes;
    }
    let mut out = css.to_string();
    for (open, quote) in [
        ("url(/fonts/", ""),
        ("url('/fonts/", "'"),
        ("url(\"/fonts/", "\""),
    ] {
        out = out.replace(open, &format!("url({quote}{base}/fonts/"));
    }
    out.into_bytes()
}

/// Serve one static asset with the caching posture its name earns (t20 B9).
///
/// This previously set `Content-Type` and nothing else, so every asset was
/// re-fetched on every load and no browser, reverse proxy or CDN could cache any
/// of it. The build emits content-hashed filenames (`index-a1b2c3d4.js`), which
/// are immutable by construction — a change produces a different name — so those
/// get a one-year `immutable` lifetime. Everything else, `index.html` above all,
/// gets `no-cache`: it names the hashed assets, so caching it would pin clients
/// to the previous deploy.
///
/// Both kinds carry a strong `ETag`, so a revalidated `no-cache` document answers
/// 304 rather than resending the body.
fn asset_response(path: &str, mime: &str, bytes: Vec<u8>, req: &HeaderMap) -> Response {
    let etag = asset_etag(&bytes);
    let cache_control = if is_content_hashed(path) {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };

    // A conditional request whose validator still matches: 304, no body.
    if etag_matches(req, &etag) {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        let h = resp.headers_mut();
        h.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static(cache_control),
        );
        if let Ok(v) = HeaderValue::from_str(&etag) {
            h.insert(header::ETAG, v);
        }
        return resp;
    }

    let mut resp = Response::new(Body::from(bytes));
    let h = resp.headers_mut();
    if let Ok(v) = HeaderValue::from_str(mime) {
        h.insert(header::CONTENT_TYPE, v);
    }
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(cache_control),
    );
    if let Ok(v) = HeaderValue::from_str(&etag) {
        h.insert(header::ETAG, v);
    }
    resp
}

/// A strong `ETag` over the exact bytes served. Content-derived, so it is right
/// for both the embedded bundle (which has no mtime) and a `MW_WEB_DIR` tree.
fn asset_etag(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    format!(
        "\"{}\"",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest[..16])
    )
}

/// Does `If-None-Match` list this entity tag? `*` matches any existing entity.
fn etag_matches(req: &HeaderMap, etag: &str) -> bool {
    let Some(raw) = req.get(header::IF_NONE_MATCH).and_then(|v| v.to_str().ok()) else {
        return false;
    };
    raw.split(',').any(|candidate| {
        let candidate = candidate.trim();
        // A cache may revalidate a strong tag using the weak form.
        let candidate = candidate.strip_prefix("W/").unwrap_or(candidate);
        candidate == "*" || candidate == etag
    })
}

/// The shortest suffix that can be a bundler hash. Vite's default is eight
/// base64url characters; a shorter trailing segment is a word, not a digest.
const MIN_CONTENT_HASH_LEN: usize = 8;

/// Whether a filename carries a bundler-generated content hash, which is what
/// makes a one-year immutable lifetime safe. Matches the Vite/Rollup default
/// `name-<hash>.ext` shape.
///
/// Two things about a real Vite hash drive this, and the rule that shipped
/// before 26.19 got both wrong — measured against a real `apps/web/dist`,
/// **96 of 242 built assets (39.7%) were served `no-cache`**, including the main
/// bundle:
///
///   * **The alphabet is base64url**, so the hash itself may contain `-` and
///     `_`. Splitting at the *last* `-` therefore does not reliably isolate it:
///     `admin-Bvbu-L2g.js` yields `L2g`, three characters, rejected. Every `-`
///     boundary is tried instead, so one of them lands on the true separator.
///   * **A hash need not contain a digit.** For an eight-character base64url
///     hash the chance of none is ≈ (54/64)^8 ≈ 25%, and `index-BgsGROWZ.js` is
///     that case on a shipped bundle.
///
/// What the old digit test was really doing was separating a digest from a
/// hand-written name, and that is kept — but expressed directly: a candidate
/// made of nothing but lowercase letters and hyphens reads as a name
/// (`mail-settings.css`, `dark-mode-toggle.css`), anything else reads as a
/// digest. Uppercase counts as digest evidence because bundler output is
/// mixed-case while hand-written asset names in this tree are kebab-case; that
/// asymmetry is what admits `nextcloud-CUQHNHRA.js`, a real shipped file with
/// neither a digit nor a lowercase letter in its hash.
///
/// **Residual, stated rather than hidden**: an all-lowercase digit-free hash
/// (≈ (26/64)^8 ≈ 1 in 2 300 assets) is still read as a name. That direction is
/// the safe one — `no-cache` revalidates against the strong `ETag` and answers
/// `304`, so it costs a round trip and never serves stale bytes. The reverse
/// error would pin a hand-written file for a year, which is why the lowercase
/// test is not relaxed further. Against the current `dist`, 242 of 242 assets
/// classify correctly and no non-asset file does.
fn is_content_hashed(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    let Some((stem, _ext)) = name.rsplit_once('.') else {
        return false;
    };
    stem.match_indices('-')
        .any(|(at, _)| looks_like_content_hash(&stem[at + 1..]))
}

/// Is this trailing segment a bundler digest rather than part of a written name?
fn looks_like_content_hash(candidate: &str) -> bool {
    candidate.len() >= MIN_CONTENT_HASH_LEN
        && candidate
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        // Nothing but lowercase letters and hyphens is a written name. A digit,
        // an underscore or any uppercase letter is evidence of a digest.
        && !candidate
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b == b'-')
}

// ---------------------------------------------------------------------------
// Session cookie helpers + auth extraction
// ---------------------------------------------------------------------------

fn session_cookie(id: &str, secure: bool) -> HeaderValue {
    let mut c = format!("{COOKIE_NAME}={id}; HttpOnly; SameSite=Strict; Path=/");
    if secure {
        c.push_str("; Secure");
    }
    HeaderValue::from_str(&c).expect("cookie value is ascii")
}

pub(crate) fn clear_cookie(secure: bool) -> HeaderValue {
    let mut c = format!("{COOKIE_NAME}=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0");
    if secure {
        c.push_str("; Secure");
    }
    HeaderValue::from_str(&c).expect("cookie value is ascii")
}

pub(crate) fn cookie_value(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in raw.split(';') {
        if let Some(v) = part.trim().strip_prefix(&format!("{COOKIE_NAME}="))
            && !v.is_empty()
        {
            return Some(v.to_string());
        }
    }
    None
}

/// Resolve the authenticated session from the cookie or native bearer, or return
/// the refusal. Enforces the idle/absolute session timeouts (§7.4) — an expired
/// session is deleted and rejected — and the account's admin flags
/// ([`account_gate`]): a disabled account's session is deleted and answered 401,
/// and an account held for a password change is answered
/// [`account_gate::password_change_required`].
///
/// Every session-authed handler calls this, so a held account reaches nothing but
/// the handlers that call [`authed_with_gate`] instead.
pub(crate) async fn authed(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<mw_store::Session, Response> {
    let (session, gate) = authed_with_gate(state, headers).await?;
    if gate.password_change_required {
        return Err(account_gate::password_change_required());
    }
    Ok(session)
}

/// [`authed`] for the handlers an account held for a password change must still
/// reach — `GET /api/me`, `GET /api/password/policy`, `POST /api/password` — which
/// get the gate back instead of a refusal. A disabled account is refused here
/// exactly as in [`authed`].
pub(crate) async fn authed_with_gate(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(mw_store::Session, account_gate::AccountGate), Response> {
    let bearer = push_relay::bearer_token(headers);
    let mut session = match &bearer {
        Some(token) => authed_native(state, token).await?,
        None => authed_cookie(state, headers).await?,
    };
    if let Err(response) = domain_upgrade::refresh_session(state, &mut session).await {
        let _ = state.store.delete_session(&session.id).await;
        state.sessions.forget(&session.id);
        account_gate::stop_engine_work(state, &session.account_id);
        return Err(response);
    }
    let gate = account_gate::for_session(&state.store, &session)
        .await
        .map_err(|e| account_gate::unavailable("session", e))?;
    if gate.disabled {
        // The panel deletes a disabled account's sessions when it sets the flag;
        // this covers a session that survived that (another server process, a
        // login that raced the flag) and makes the refusal independent of it.
        tracing::info!(
            "session for disabled account {} refused and deleted",
            session.username
        );
        let _ = state.store.delete_session(&session.id).await;
        if bearer.is_some() {
            let _ = state
                .store
                .delete_native_session(&push_relay::hash_token(&session.id))
                .await;
        }
        state.sessions.forget(&session.id);
        account_gate::stop_engine_work(state, &session.account_id);
        return Err(unauthorized());
    }
    request_protection::session_account(&session).await?;
    Ok((session, gate))
}

/// The cookie half of [`authed_with_gate`]: resolve the session and enforce its
/// timeouts.
async fn authed_cookie(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<mw_store::Session, Response> {
    let id = cookie_value(headers).ok_or_else(unauthorized)?;
    let session = state
        .store
        .get_session(&id)
        .await
        .map_err(|_| unauthorized())?;
    if let Err(reason) = state.sessions.check(&id, &state.hardening) {
        tracing::info!("session {id} expired ({reason:?})");
        let _ = state.store.delete_session(&id).await;
        state.sessions.forget(&id);
        return Err(session_expired());
    }
    security_center::enforce_session(state, &session).await?;
    let _ = state.store.touch_session(&id).await;
    Ok(session)
}

/// Resolve a native bearer session (plan §2.2). The token doubles as the opaque
/// session id; its `native_sessions` row (keyed by the token HASH) marks it
/// bearer-eligible, so a session id learned elsewhere cannot be replayed as a
/// bearer token. Enforces the same idle/absolute timeouts as the cookie path.
async fn authed_native(state: &AppState, token: &str) -> Result<mw_store::Session, Response> {
    let hash = push_relay::hash_token(token);
    match state.store.get_native_session(&hash).await {
        Ok(Some(_)) => {}
        _ => return Err(unauthorized()),
    }
    let session = state
        .store
        .get_session(token)
        .await
        .map_err(|_| unauthorized())?;
    if let Err(reason) = state.sessions.check(token, &state.hardening) {
        tracing::info!("native session expired ({reason:?})");
        let _ = state.store.delete_session(token).await;
        let _ = state.store.delete_native_session(&hash).await;
        state.sessions.forget(token);
        return Err(session_expired());
    }
    security_center::enforce_session(state, &session).await?;
    let _ = state.store.touch_session(token).await;
    Ok(session)
}

/// 401 for an expired session — distinct body so the client re-authenticates.
fn session_expired() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "session expired" })),
    )
        .into_response()
}

/// The readable double-submit CSRF cookie (NOT HttpOnly — the SPA must read it
/// to echo it back; SameSite=Strict keeps it same-origin).
pub(crate) fn csrf_cookie(token: &str, secure: bool) -> HeaderValue {
    let mut c = format!("{CSRF_COOKIE}={token}; SameSite=Strict; Path=/");
    if secure {
        c.push_str("; Secure");
    }
    HeaderValue::from_str(&c).expect("csrf token is url-safe base64")
}

/// Resolve a possibly-relative JMAP `apiUrl` against the origin of the
/// user-supplied server URL, yielding an absolute URL for server-side calls.
fn resolve_api_url(server_url: &str, api_url: &str) -> String {
    if api_url.starts_with("http://") || api_url.starts_with("https://") {
        return api_url.to_string();
    }
    match origin_of(server_url) {
        Some(origin) if api_url.starts_with('/') => format!("{origin}{api_url}"),
        Some(origin) => format!("{origin}/{api_url}"),
        None => api_url.to_string(),
    }
}

/// Extract `scheme://authority` from a URL string.
fn origin_of(url: &str) -> Option<String> {
    let scheme_end = url.find("://")? + 3;
    let rest = &url[scheme_end..];
    let end = rest.find('/').map(|i| scheme_end + i).unwrap_or(url.len());
    Some(url[..end].to_string())
}

#[cfg(test)]
mod external_base_tests {
    use super::*;
    use axum::extract::ConnectInfo;
    use axum::http::Extensions;
    use std::net::SocketAddr;

    /// Build a `HeaderMap` from `(name, value)` pairs. **Appends**, matching the
    /// `hdrs` helper in `proxy.rs`, so a repeated name produces the two separate
    /// header lines a chained proxy actually emits rather than collapsing to the
    /// last one.
    fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut m = HeaderMap::new();
        for (k, v) in pairs {
            m.append(
                axum::http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                HeaderValue::from_str(v).unwrap(),
            );
        }
        m
    }

    /// Request extensions as the serve path builds them, with a peer address.
    fn peer(addr: &str) -> Extensions {
        let mut e = Extensions::new();
        e.insert(ConnectInfo(addr.parse::<SocketAddr>().unwrap()));
        e
    }

    /// Extensions with no `ConnectInfo` — what an in-process test transport, or a
    /// serve path that never installed it, produces.
    fn no_peer() -> Extensions {
        Extensions::new()
    }

    fn base(public: Option<&str>, trusted: &str) -> ExternalBase {
        let (public_https, public_host) = match public.and_then(split_public_url) {
            Some((s, h)) => (Some(s), Some(h)),
            None => (None, None),
        };
        ExternalBase {
            public_https,
            public_host,
            trusted_proxies: CidrSet::parse(trusted),
            self_terminated_tls: false,
            cookie_secure_override: None,
            hsts: HeaderValue::from_static("max-age=31536000").into(),
        }
    }

    /// The same, for a process that terminates TLS itself (`MW_ACME`, or
    /// `MW_TLS_CERT` + `MW_TLS_KEY`).
    fn base_serving_tls(public: Option<&str>, trusted: &str) -> ExternalBase {
        ExternalBase {
            self_terminated_tls: true,
            ..base(public, trusted)
        }
    }

    #[test]
    fn public_url_parses_scheme_and_authority() {
        assert_eq!(
            split_public_url("https://mail.example.org"),
            Some((true, "mail.example.org".into()))
        );
        // A path is dropped — sub-path hosting is MW_BASE_PATH's job.
        assert_eq!(
            split_public_url("https://mail.example.org/mail/"),
            Some((true, "mail.example.org".into()))
        );
        assert_eq!(
            split_public_url("http://LOCALHOST:8080"),
            Some((false, "localhost:8080".into()))
        );
        // Anything that is not an absolute http(s) URL is refused, not guessed at.
        assert_eq!(split_public_url("mail.example.org"), None);
        assert_eq!(split_public_url("ftp://mail.example.org"), None);
        assert_eq!(split_public_url("https://"), None);
    }

    #[test]
    fn public_url_is_the_authority_on_scheme() {
        let b = base(Some("https://mail.example.org"), "");
        // Believed with no peer, no headers, and against a contradicting header.
        assert!(b.is_https(&headers(&[]), &no_peer()));
        assert!(b.is_https(&headers(&[("x-forwarded-proto", "http")]), &no_peer()));
        assert!(b.declares_https());
    }

    #[test]
    fn forwarded_proto_is_ignored_from_an_untrusted_peer() {
        // The B6/B1 core: a client that simply asserts https must not be believed.
        let b = base(None, "10.0.0.0/8");
        let h = headers(&[("x-forwarded-proto", "https")]);
        assert!(!b.is_https(&h, &peer("203.0.113.9:5555")));
        // Trusted list configured but no peer address to check against ⇒ no trust.
        assert!(!b.is_https(&h, &no_peer()));
        // No trusted list at all ⇒ the header is inert however it arrived.
        assert!(!base(None, "").is_https(&h, &peer("10.1.2.3:5555")));
    }

    #[test]
    fn forwarded_proto_is_honoured_from_a_trusted_peer() {
        let b = base(None, "10.0.0.0/8, 2001:db8::/32");
        assert!(b.is_https(
            &headers(&[("x-forwarded-proto", "https")]),
            &peer("10.1.2.3:5555")
        ));
        // A trusted proxy reporting plaintext is believed too.
        assert!(!b.is_https(
            &headers(&[("x-forwarded-proto", "http")]),
            &peer("10.1.2.3:5555")
        ));
        // Multi-valued headers have their own test below; the nearest hop's value
        // is the one read, so this list resolves to its LAST element.
        assert!(b.is_https(
            &headers(&[("x-forwarded-proto", "http, https")]),
            &peer("10.1.2.3:5555")
        ));
        // IPv6, and a v4 peer seen through a dual-stack listener as v4-mapped.
        assert!(b.is_https(
            &headers(&[("x-forwarded-proto", "https")]),
            &peer("[2001:db8::1]:5555")
        ));
        assert!(b.is_https(
            &headers(&[("x-forwarded-proto", "https")]),
            &peer("[::ffff:10.1.2.3]:5555")
        ));
    }

    #[test]
    fn host_resolution_prefers_public_url_then_trusted_forwarded_then_host() {
        let h = headers(&[
            ("host", "backend.internal"),
            ("x-forwarded-host", "mail.example.org"),
        ]);
        assert_eq!(
            base(Some("https://canonical.example.org"), "10.0.0.0/8")
                .host(&h, &peer("10.1.2.3:5555"))
                .as_deref(),
            Some("canonical.example.org")
        );
        assert_eq!(
            base(None, "10.0.0.0/8")
                .host(&h, &peer("10.1.2.3:5555"))
                .as_deref(),
            Some("mail.example.org")
        );
        // Untrusted peer: X-Forwarded-Host is ignored, Host stands (today's shape).
        assert_eq!(
            base(None, "10.0.0.0/8")
                .host(&h, &peer("203.0.113.9:5555"))
                .as_deref(),
            Some("backend.internal")
        );
        assert_eq!(
            base(Some("https://canonical.example.org"), "")
                .base_url(&headers(&[]), &no_peer())
                .as_deref(),
            Some("https://canonical.example.org")
        );
    }

    /// A proxy that **appends** to `X-Forwarded-Proto` rather than replacing it
    /// (Envoy's shape, and any chained tier that passes the client's value
    /// through) leaves whatever the client sent on the LEFT. The peer gate says
    /// the header may be looked at; it says nothing about which element inside it
    /// the proxy wrote, so the element read must be the nearest hop's — the last.
    ///
    /// Each direction is asserted, so "the last element is read" is distinguished
    /// from "a multi-valued header is refused outright": a reader that simply
    /// dropped lists would pass the first half of every pair and fail the second.
    #[test]
    fn an_appending_proxy_cannot_have_the_client_choose_the_scheme() {
        let b = base(None, "10.0.0.0/8");
        let p = peer("10.1.2.3:5555");

        // THE ATTACK: the client asserts https, the trusted proxy appends the
        // plaintext it actually saw. Reading leftmost would believe the client.
        assert!(!b.is_https(&headers(&[("x-forwarded-proto", "https, http")]), &p));
        // ... and the mirror, so this is not just "lists are ignored".
        assert!(b.is_https(&headers(&[("x-forwarded-proto", "http, https")]), &p));

        // Repeated header lines are the same list by another spelling: RFC 9110
        // makes them equivalent to one comma-joined value, in order, so the
        // nearest hop's value is the last element of the last line.
        assert!(!b.is_https(
            &headers(&[
                ("x-forwarded-proto", "https"),
                ("x-forwarded-proto", "http"),
            ]),
            &p
        ));
        assert!(b.is_https(
            &headers(&[
                ("x-forwarded-proto", "http"),
                ("x-forwarded-proto", "https"),
            ]),
            &p
        ));
        // Mixed: a list on one line, a further hop on the next.
        assert!(!b.is_https(
            &headers(&[
                ("x-forwarded-proto", "https, https"),
                ("x-forwarded-proto", "http"),
            ]),
            &p
        ));

        // An unusable nearest element is not a licence to reach further left: a
        // trailing comma leaves an empty token, and empty is not `https`.
        assert!(!b.is_https(&headers(&[("x-forwarded-proto", "https,")]), &p));
        assert!(!b.is_https(&headers(&[("x-forwarded-proto", "https, ")]), &p));

        // An untrusted peer is refused the header whichever end it writes.
        let untrusted = peer("203.0.113.9:5555");
        assert!(!b.is_https(
            &headers(&[("x-forwarded-proto", "http, https")]),
            &untrusted
        ));
        assert!(!b.is_https(
            &headers(&[("x-forwarded-proto", "https, http")]),
            &untrusted
        ));
        assert!(!b.is_https(
            &headers(&[
                ("x-forwarded-proto", "http"),
                ("x-forwarded-proto", "https"),
            ]),
            &untrusted
        ));
    }

    /// The same walk direction for `X-Forwarded-Host`, which had no test at all.
    /// Nothing consumes [`ExternalBase::host`] yet, which is exactly why this is
    /// worth pinning now: the value a future DCR issuer or WebAuthn RP origin
    /// would adopt must not be one the client chose.
    #[test]
    fn an_appending_proxy_cannot_have_the_client_choose_the_host() {
        let b = base(None, "10.0.0.0/8");
        let p = peer("10.1.2.3:5555");
        let host = |h: &HeaderMap| b.host(h, &p);

        let appended = headers(&[
            ("host", "backend.internal"),
            ("x-forwarded-host", "evil.example.net, mail.example.org"),
        ]);
        assert_eq!(host(&appended).as_deref(), Some("mail.example.org"));

        // Mirror, so the assertion above is not satisfied by ignoring lists.
        let appended = headers(&[
            ("host", "backend.internal"),
            ("x-forwarded-host", "mail.example.org, evil.example.net"),
        ]);
        assert_eq!(host(&appended).as_deref(), Some("evil.example.net"));

        // Repeated lines, the other spelling of the same chain.
        let lines = headers(&[
            ("host", "backend.internal"),
            ("x-forwarded-host", "evil.example.net"),
            ("x-forwarded-host", "mail.example.org"),
        ]);
        assert_eq!(host(&lines).as_deref(), Some("mail.example.org"));

        // Unusable nearest element ⇒ fall through to `Host`, never leftwards.
        let trailing = headers(&[
            ("host", "backend.internal"),
            ("x-forwarded-host", "mail.example.org,"),
        ]);
        assert_eq!(host(&trailing).as_deref(), Some("backend.internal"));

        // A value that is not text is unreadable, not a reason to read an earlier
        // line: the header contributes nothing and `Host` stands.
        let mut opaque = headers(&[("host", "backend.internal")]);
        opaque.append(
            axum::http::HeaderName::from_static("x-forwarded-host"),
            HeaderValue::from_str("mail.example.org").unwrap(),
        );
        opaque.append(
            axum::http::HeaderName::from_static("x-forwarded-host"),
            HeaderValue::from_bytes(&[0xC3, 0xA9]).unwrap(),
        );
        assert_eq!(host(&opaque).as_deref(), Some("backend.internal"));

        // Untrusted peer: neither element is honoured, `Host` stands.
        let untrusted = headers(&[
            ("host", "backend.internal"),
            ("x-forwarded-host", "evil.example.net, mail.example.org"),
        ]);
        assert_eq!(
            b.host(&untrusted, &peer("203.0.113.9:5555")).as_deref(),
            Some("backend.internal")
        );
    }

    #[test]
    fn cookie_secure_derives_from_scheme_and_the_env_override_wins() {
        let b = base(Some("https://mail.example.org"), "");
        let (h, e) = (headers(&[]), no_peer());
        // Derived: https base ⇒ Secure even though the flag is false. This is the
        // t20 B6 fix — the TLS deployment that forgot MW_COOKIE_SECURE.
        assert!(b.cookie_secure(false, &h, &e));

        // Explicit override pins the answer in both directions.
        let forced_off = ExternalBase {
            cookie_secure_override: Some(false),
            ..b.clone()
        };
        assert!(!forced_off.cookie_secure(true, &h, &e));
        let forced_on = ExternalBase {
            cookie_secure_override: Some(true),
            ..base(None, "")
        };
        assert!(forced_on.cookie_secure(false, &h, &e));

        // Plain http with nothing configured keeps today's behaviour, which is what
        // the ~30 existing test harnesses (cookie_secure: false) depend on.
        assert!(!base(None, "").cookie_secure(false, &h, &e));
        // An explicitly-configured deployment still gets Secure over http.
        assert!(base(None, "").cookie_secure(true, &h, &e));
    }

    #[test]
    fn hsts_is_emitted_only_over_https() {
        let b = base(Some("https://mail.example.org"), "");
        assert!(b.hsts(&headers(&[]), &no_peer()).is_some());
        // Plaintext: no header. It would be unenforceable, and it would pin a host
        // that may have no TLS at all.
        assert!(base(None, "").hsts(&headers(&[]), &no_peer()).is_none());
        // Configured off entirely.
        let off = ExternalBase { hsts: None, ..b };
        assert!(off.hsts(&headers(&[]), &no_peer()).is_none());
    }

    // ── t20 B4: sub-path hosting (`MW_BASE_PATH`) ────────────────────────────

    #[test]
    fn base_path_normalizes_the_spellings_an_operator_writes() {
        // Every accepted spelling lands on the one canonical form, which is also
        // what the SPA's `normalizeBase()` produces from the injected value.
        for raw in ["/mail", "mail", "/mail/", " /mail/ ", "/mail//"] {
            assert_eq!(normalize_base_path(raw), "/mail", "input {raw:?}");
        }
        assert_eq!(normalize_base_path("/a/b"), "/a/b");
        // Root spellings ⇒ "" ⇒ every path stays byte-identical to the pre-B4 code.
        for raw in ["", "   ", "/", "//"] {
            assert_eq!(normalize_base_path(raw), "", "input {raw:?}");
        }
        // Not a plain path prefix ⇒ refused, not normalized into something else.
        // `..` matters most: it would otherwise reach the asset lookup as a
        // traversal component.
        for raw in [
            "/mail/..",
            "/../etc",
            "/mail/./x",
            "/mail?x=1",
            "/mail#frag",
            "https://evil.example/mail",
            "/mail x",
            "//host/mail",
            "/mail\"',",
        ] {
            assert_eq!(normalize_base_path(raw), "", "input {raw:?}");
        }
    }

    #[test]
    fn base_match_only_strips_on_a_segment_boundary() {
        assert_eq!(match_base_path("/mail", "/mail"), BaseMatch::Exact);
        assert_eq!(match_base_path("/mail/", "/mail"), BaseMatch::Under("/"));
        assert_eq!(
            match_base_path("/mail/api/login", "/mail"),
            BaseMatch::Under("/api/login")
        );
        // The trap: a sibling path sharing the prefix's leading characters must
        // NOT be rewritten to `box/…`.
        assert_eq!(match_base_path("/mailbox", "/mail"), BaseMatch::Outside);
        // Already-stripped by the proxy, and the root-only endpoints: forwarded
        // untouched so both proxy idioms and `/healthz` keep working.
        assert_eq!(match_base_path("/api/login", "/mail"), BaseMatch::Outside);
        assert_eq!(match_base_path("/healthz", "/mail"), BaseMatch::Outside);
        // No prefix configured ⇒ nothing ever matches ⇒ the middleware is inert.
        assert_eq!(match_base_path("/mail/api", ""), BaseMatch::Outside);
    }

    #[test]
    fn index_html_gains_the_base_shim_only_under_a_prefix() {
        let shell =
            br#"<!doctype html><html><head><title>Mailwoman</title></head><body></body></html>"#;

        // Root: byte-identical to what was built.
        assert_eq!(
            adapt_asset_to_base("index.html", shell.to_vec(), ""),
            shell.to_vec()
        );

        let out = String::from_utf8(adapt_asset_to_base("index.html", shell.to_vec(), "/mail"))
            .expect("utf-8");
        // Prefix-absolute src, so a deep-linked SPA fallback cannot resolve it
        // against the wrong directory.
        assert!(out.contains(r#"<script src="/mail/__mw_base.js"></script>"#));
        // Injected at the top of <head>, ahead of the deferred module entry.
        assert!(out.starts_with(r#"<!doctype html><html><head><script src="/mail/"#));
        // The shim publishes exactly what `basePath.ts` reads.
        assert_eq!(base_shim_body("/mail"), "window.__MW_BASE__=\"/mail/\";\n");
        assert_eq!(base_shim_body(""), "window.__MW_BASE__=\"/\";\n");
    }

    #[test]
    fn bundled_css_font_urls_are_prefixed() {
        // The production build minifies to the unquoted form; the other two are
        // covered so an unminified build does not regress silently.
        let css = b"@font-face{src:url(/fonts/inter-400.woff2) format('woff2')}\
                    a{background:url('/fonts/x.woff2')}b{background:url(\"/fonts/y.woff2\")}";
        let out = String::from_utf8(adapt_asset_to_base(
            "assets/index-a1b2c3d4.css",
            css.to_vec(),
            "/mail",
        ))
        .expect("utf-8");
        assert!(out.contains("url(/mail/fonts/inter-400.woff2)"));
        assert!(out.contains("url('/mail/fonts/x.woff2')"));
        assert!(out.contains("url(\"/mail/fonts/y.woff2\")"));
        assert!(!out.contains("url(/fonts/"));
        // Root: untouched, so a root deployment serves the built bytes exactly.
        assert_eq!(
            adapt_asset_to_base("assets/index-a1b2c3d4.css", css.to_vec(), ""),
            css.to_vec()
        );
        // A JS chunk is never rewritten — the relative Vite base already handles it.
        let js = b"import('./settings-a1b2c3d4.js');/fonts/nope".to_vec();
        assert_eq!(
            adapt_asset_to_base("assets/index-a1b2c3d4.js", js.clone(), "/mail"),
            js
        );
    }

    #[test]
    fn jmap_session_urls_carry_the_prefix() {
        let session = mw_engine::session_json("acct", "user@example.org");
        let prefixed = prefix_session_urls(session.clone(), "/mail");
        assert_eq!(prefixed["apiUrl"], "/mail/jmap/api");
        assert_eq!(prefixed["eventSourceUrl"], "/mail/jmap/eventsource");
        assert_eq!(
            prefixed["downloadUrl"],
            "/mail/jmap/download/{accountId}/{blobId}/{name}"
        );
        assert_eq!(prefixed["uploadUrl"], "/mail/jmap/upload/{accountId}");
        // Untouched at the root, and non-URL fields are never rewritten.
        assert_eq!(prefix_session_urls(session.clone(), ""), session);
        assert_eq!(prefixed["username"], "user@example.org");
        // An absolute URL (a value that did not originate here) is left alone.
        let abs = prefix_session_urls(json!({ "apiUrl": "https://up.example/jmap" }), "/mail");
        assert_eq!(abs["apiUrl"], "https://up.example/jmap");
    }

    #[test]
    fn cookies_are_rescoped_to_the_deploy_prefix() {
        let mut h = HeaderMap::new();
        h.append(
            header::SET_COOKIE,
            HeaderValue::from_static("mw_session=a; HttpOnly; SameSite=Strict; Path=/"),
        );
        h.append(
            header::SET_COOKIE,
            HeaderValue::from_static("mw_csrf=b; SameSite=Strict; Path=/; Secure"),
        );
        scope_cookies_to_base(&mut h, "/mail");
        let out: Vec<String> = h
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect();
        assert_eq!(
            out,
            vec![
                "mw_session=a; HttpOnly; SameSite=Strict; Path=/mail",
                "mw_csrf=b; SameSite=Strict; Path=/mail; Secure",
            ]
        );
        // Every other attribute survives, including the clearing cookie's Max-Age.
        let mut clearing = HeaderMap::new();
        clearing.append(
            header::SET_COOKIE,
            HeaderValue::from_static("mw_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0"),
        );
        scope_cookies_to_base(&mut clearing, "/mail");
        assert_eq!(
            clearing.get(header::SET_COOKIE).unwrap(),
            "mw_session=; HttpOnly; SameSite=Strict; Path=/mail; Max-Age=0"
        );
        // Root deployment: untouched.
        let mut root = HeaderMap::new();
        root.append(
            header::SET_COOKIE,
            HeaderValue::from_static("mw_session=a; Path=/"),
        );
        scope_cookies_to_base(&mut root, "");
        assert_eq!(
            root.get(header::SET_COOKIE).unwrap(),
            "mw_session=a; Path=/"
        );
    }

    #[test]
    fn env_bool_accepts_both_spellings_and_rejects_junk() {
        assert_eq!(parse_env_bool("1"), Some(true));
        assert_eq!(parse_env_bool(" TRUE "), Some(true));
        assert_eq!(parse_env_bool("off"), Some(false));
        assert_eq!(parse_env_bool("0"), Some(false));
        // Unrecognised ⇒ "not set", not a silent false.
        assert_eq!(parse_env_bool("maybe"), None);
        assert_eq!(parse_env_bool(""), None);
    }

    /// B2, the whole gate. Every uncertainty must refuse: with `MW_HEADER_AUTH=1`
    /// a `true` here mints a session for whoever the header names, with no
    /// password. There is no in-process coverage of this path other than these
    /// assertions — t20-e11's spoof suite drives it through a real listener.
    #[test]
    fn header_auth_peer_gate_fails_closed() {
        let allowed = "10.0.0.0/8, 192.168.1.5";
        let inside = Some("10.1.2.3".parse().unwrap());
        let outside = Some("203.0.113.9".parse().unwrap());

        // The only combination that admits: configured list AND a peer in it.
        assert!(header_auth_peer_allowed_with(allowed, inside));

        // Peer outside the list.
        assert!(!header_auth_peer_allowed_with(allowed, outside));
        // Allowlist unset ⇒ nobody, not everybody. This is the posture that makes
        // enabling MW_HEADER_AUTH without an allowlist safe rather than catastrophic.
        assert!(!header_auth_peer_allowed_with("", inside));
        // Allowlist present but entirely unparseable ⇒ same as unset, not match-all.
        assert!(!header_auth_peer_allowed_with(
            "nonsense, 10.0.0.0/99",
            inside
        ));
        // No peer address at all (serve path installed no ConnectInfo) ⇒ refuse,
        // because there is nothing to check the assertion against.
        assert!(!header_auth_peer_allowed_with(allowed, None));
        assert!(!header_auth_peer_allowed_with("", None));

        // A bare address is a host route: the neighbour is not admitted.
        assert!(header_auth_peer_allowed_with(
            allowed,
            Some("192.168.1.5".parse().unwrap())
        ));
        assert!(!header_auth_peer_allowed_with(
            allowed,
            Some("192.168.1.6".parse().unwrap())
        ));
        // A v4 peer reported as v4-mapped by a dual-stack listener still matches,
        // so the gate cannot be sidestepped by which socket family accepted it.
        assert!(header_auth_peer_allowed_with(
            allowed,
            Some("::ffff:10.1.2.3".parse().unwrap())
        ));
    }

    /// The header-auth allowlist reads as fail-closed: an unset or all-junk list is
    /// indistinguishable from "unconfigured", and `header_auth_peer_allowed` refuses
    /// on that rather than admitting everyone.
    #[test]
    fn cidr_list_drops_malformed_entries_without_widening() {
        assert!(CidrSet::parse("").is_empty());
        assert!(CidrSet::parse("not-an-ip, 10.0.0.0/99").is_empty());
        let list = CidrSet::parse("10.0.0.0/8, junk, 192.168.1.5");
        assert!(!list.is_empty());
        assert!(list.contains("10.9.9.9".parse().unwrap()));
        // A bare address is a host route, not a network.
        assert!(list.contains("192.168.1.5".parse().unwrap()));
        assert!(!list.contains("192.168.1.6".parse().unwrap()));
        // The dropped junk entry did not widen anything.
        assert!(!list.contains("203.0.113.1".parse().unwrap()));
    }

    #[test]
    fn set_cookie_is_upgraded_once_and_only_when_missing() {
        let mut h = HeaderMap::new();
        h.append(
            header::SET_COOKIE,
            HeaderValue::from_static("mw_session=a; HttpOnly; SameSite=Strict; Path=/"),
        );
        h.append(
            header::SET_COOKIE,
            HeaderValue::from_static("mw_csrf=b; SameSite=Strict; Path=/; Secure"),
        );
        mark_cookies_secure(&mut h);

        let out: Vec<&str> = h
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap())
            .collect();
        assert_eq!(out.len(), 2, "cookie count must not change");
        assert!(out[0].ends_with("; Secure"), "missing Secure was added");
        // The one that already had it is untouched — no duplicate attribute.
        assert_eq!(out[1].matches("Secure").count(), 1);

        // Idempotent: a second pass changes nothing.
        let before = out.join("\n");
        mark_cookies_secure(&mut h);
        let after: Vec<&str> = h
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap())
            .collect();
        assert_eq!(before, after.join("\n"));
    }

    #[test]
    fn only_content_hashed_names_earn_an_immutable_lifetime() {
        assert!(is_content_hashed("assets/index-a1b2c3d4.js"));
        assert!(is_content_hashed("assets/Compose-9f8e7d6c5b.css"));
        // index.html names the hashed assets — caching it would pin the deploy.
        assert!(!is_content_hashed("index.html"));
        assert!(!is_content_hashed("sw.js"));
        assert!(!is_content_hashed("themes/dark.svg"));
        // A hyphenated human name is not a hash (no digits, or too short).
        assert!(!is_content_hashed("assets/mail-settings.css"));
        assert!(!is_content_hashed("assets/index-ab12.js"));
    }

    /// **t20-e-e2e D3, measured against a real `apps/web/dist`.** Every fixture
    /// above happens to contain a digit, which is why a rule requiring one
    /// passed its own test while serving 96 of 242 built assets `no-cache`.
    ///
    /// These are **real filenames** from that build, not invented shapes. That
    /// is the point: fixtures drawn from the producer's actual output are what
    /// the digit rule was never made to face.
    #[test]
    fn real_vite_hashes_earn_an_immutable_lifetime() {
        // Digit-free, mixed case. The shipped main bundle, served `no-cache`.
        assert!(is_content_hashed("assets/index-BgsGROWZ.js"));
        assert!(is_content_hashed("assets/assist-CEdGpMgw.js"));
        // The hash's own alphabet is base64url, so it may contain `-` or `_` —
        // splitting at the LAST `-` gave `L2g`, three characters, rejected.
        assert!(is_content_hashed("assets/admin-Bvbu-L2g.js"));
        assert!(is_content_hashed("assets/auth-Bbng-azm.js"));
        assert!(is_content_hashed("assets/contacts-0b-hpZac.js"));
        assert!(is_content_hashed("assets/admin-_JQUlavb.js"));
        assert!(is_content_hashed("assets/admin-CEjRn_ee.js"));
        // Neither a digit nor a lowercase letter. Admitted because hand-written
        // names in this tree are kebab-case, so uppercase is digest evidence.
        assert!(is_content_hashed("assets/nextcloud-CUQHNHRA.js"));

        // The other direction, which the digit rule was really enforcing and
        // which must not be lost: a written name stays revalidated. Multi-word
        // names are included because trying every `-` boundary gives them more
        // chances to be mistaken for a hash than the old single split did.
        assert!(!is_content_hashed("assets/mail-settings.css"));
        assert!(!is_content_hashed("assets/dark-mode-toggle.css"));
        assert!(!is_content_hashed("assets/user-guide-print.css"));
        assert!(!is_content_hashed("assets/pdf-worker-entry.mjs"));
        // Real non-asset files from the same build.
        assert!(!is_content_hashed("pdf.worker.mjs"));
        assert!(!is_content_hashed("themes/grove-grain.svg"));
        assert!(!is_content_hashed("themes/grove-paper.svg"));
        // No extension, or no separator at all.
        assert!(!is_content_hashed("assets/BgsGROWZ"));
        assert!(!is_content_hashed("assets/indexBgsGROWZ.js"));
    }

    /// **t20-e-e2e D6(b).** An over-limit body must be **read to the end** before
    /// the `413` is written, not closed on. Apache's `mod_proxy` abandons the
    /// exchange when its write fails and never reads the response already waiting
    /// on the socket, so the client gets httpd's `502 text/html` instead of the
    /// JSON `413`; `t20-e7` measured five `mod_proxy` configurations and all five
    /// still produced `502`, which is what makes this app-side.
    ///
    /// Observed by counting how many chunks the body was polled for. The old
    /// `axum::body::to_bytes` path stopped at the chunk that broke the cap, so a
    /// counter proves drain-versus-close directly rather than by proxy.
    #[tokio::test]
    async fn an_over_limit_body_is_drained_before_the_413() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        /// A body of `chunks` chunks of `size` bytes that counts what it yields.
        fn counted(chunks: usize, size: usize, seen: Arc<AtomicUsize>) -> Body {
            let stream = futures_util::stream::iter((0..chunks).map(move |_| {
                seen.fetch_add(size, Ordering::SeqCst);
                Ok::<_, std::io::Error>(Bytes::from(vec![b'a'; size]))
            }));
            Body::from_stream(stream)
        }

        // Under the cap: accepted whole, nothing to drain.
        let seen = Arc::new(AtomicUsize::new(0));
        let body = counted(4, 4, Arc::clone(&seen));
        assert_eq!(read_body_capped(body, 16).await.unwrap().len(), 16);
        assert_eq!(seen.load(Ordering::SeqCst), 16, "a body at the cap is read");

        // Over the cap: refused, and the WHOLE body is still consumed. Pre-fix
        // this stopped at 20 — the cap plus the chunk that broke it.
        let seen = Arc::new(AtomicUsize::new(0));
        let body = counted(25, 4, Arc::clone(&seen));
        assert!(read_body_capped(body, 16).await.is_err());
        assert_eq!(
            seen.load(Ordering::SeqCst),
            100,
            "the client must be able to finish writing before the 413 is sent, or a \
             proxy that abandons on write failure replaces it with its own error"
        );

        // The drain is bounded. `DRAIN_BUDGET_BYTES` is one further cap, so a body
        // past twice the cap is cut off rather than followed indefinitely.
        const CHUNK: usize = 64 * 1024;
        let seen = Arc::new(AtomicUsize::new(0));
        let offered = (MAX_UPLOAD_BYTES + DRAIN_BUDGET_BYTES) / CHUNK + 64;
        let body = counted(offered, CHUNK, Arc::clone(&seen));
        assert!(read_body_capped(body, MAX_UPLOAD_BYTES).await.is_err());
        let read = seen.load(Ordering::SeqCst);
        assert!(
            read < offered * CHUNK,
            "an attacker-controlled body must not be followed to its end: read \
             {read} of {} offered",
            offered * CHUNK
        );
        assert!(
            read <= MAX_UPLOAD_BYTES + DRAIN_BUDGET_BYTES + CHUNK,
            "the total read stays within the cap plus one drain budget, got {read}"
        );
    }

    /// **t20-e-e2e D4.** With TLS terminated by this process there is no
    /// `X-Forwarded-Proto` to read — an L4 balancer doing passthrough cannot add
    /// one — so the scheme has to come from the listener. Before this, `is_https`
    /// was `false` on an ACME or passthrough deployment: no HSTS, and session
    /// cookies without `Secure` unless the operator also set `MW_COOKIE_SECURE`.
    #[test]
    fn the_apps_own_tls_listener_is_https_without_any_header() {
        let b = base_serving_tls(None, "");
        let (h, e) = (headers(&[]), no_peer());

        assert!(b.is_https(&h, &e), "the process is the TLS endpoint");
        assert!(b.hsts(&h, &e).is_some(), "HSTS belongs on a real https hop");
        // The half that is worse than the missing header: `Secure` must follow
        // from noticing we serve https, not from MW_COOKIE_SECURE being set.
        assert!(
            b.cookie_secure(false, &h, &e),
            "an ACME/passthrough deployment must not need MW_COOKIE_SECURE to \
             get Secure session cookies"
        );
        assert_eq!(
            b.base_url(&headers(&[("host", "mail.example.org")]), &e)
                .as_deref(),
            Some("https://mail.example.org"),
            "a process serving https must not describe itself as http"
        );

        // A half-configured pair is not a TLS listener; `serve` falls through to
        // plaintext on it and so must this.
        assert!(!tls_terminated_here(None, Some("/certs/s.crt"), None));
        assert!(!tls_terminated_here(None, Some("/certs/s.crt"), Some(" ")));
        assert!(!tls_terminated_here(None, None, Some("/certs/s.key")));
        assert!(tls_terminated_here(None, Some("/c"), Some("/k")));
        assert!(tls_terminated_here(Some("mail.example.org"), None, None));
        assert!(!tls_terminated_here(Some("  "), None, None));
        assert!(!tls_terminated_here(None, None, None));

        // Precedence. A trusted proxy in front knows what the BROWSER used, so
        // its answer outranks our own transport in both directions.
        let proxied = base_serving_tls(None, "10.0.0.0/8");
        let p = peer("10.1.2.3:5555");
        assert!(
            !proxied.is_https(&headers(&[("x-forwarded-proto", "http")]), &p),
            "a trusted proxy reporting a plaintext client hop is believed"
        );
        assert!(proxied.is_https(&headers(&[("x-forwarded-proto", "https")]), &p));
        // An UNTRUSTED peer's header is inert, so it cannot suppress the fact
        // that we ourselves are the TLS endpoint.
        assert!(
            proxied.is_https(
                &headers(&[("x-forwarded-proto", "http")]),
                &peer("203.0.113.9:5555")
            ),
            "an untrusted claim of http must not downgrade our own TLS listener"
        );
        // And M1's walk direction still decides which element is read.
        assert!(
            !proxied.is_https(&headers(&[("x-forwarded-proto", "https, http")]), &p),
            "the nearest hop still wins over the client's leftmost value"
        );

        // MW_PUBLIC_URL stays absolute in both directions.
        assert!(
            !base_serving_tls(Some("http://mail.example.org"), "").is_https(&h, &e),
            "an operator who declares an http public URL is believed"
        );
        // And a plaintext listener is unchanged — no accidental blanket https.
        assert!(!base(None, "").is_https(&h, &e));
    }

    #[test]
    fn if_none_match_honours_star_and_the_weak_form() {
        let etag = asset_etag(b"hello");
        assert!(etag_matches(
            &headers(&[("if-none-match", etag.as_str())]),
            &etag
        ));
        assert!(etag_matches(
            &headers(&[("if-none-match", &format!("W/{etag}"))]),
            &etag
        ));
        assert!(etag_matches(&headers(&[("if-none-match", "*")]), &etag));
        assert!(etag_matches(
            &headers(&[("if-none-match", &format!("\"other\", {etag}"))]),
            &etag
        ));
        assert!(!etag_matches(
            &headers(&[("if-none-match", "\"nope\"")]),
            &etag
        ));
        assert!(!etag_matches(&headers(&[]), &etag));
        // Different bytes ⇒ different tag, so a stale validator does not match.
        assert_ne!(etag, asset_etag(b"hello!"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped shell [`CSP`] is exactly e6's [`image_proxy::SHELL_CSP_TIGHTENED`]
    /// (26.17 re-enabled `require-trusted-types-for 'script'` once the SPA registered a
    /// default Trusted Types policy — see the `CSP` docs). Normalize whitespace since
    /// the constants are written across continuation lines.
    #[test]
    fn csp_equals_e6_tightened() {
        let norm = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        assert_eq!(
            norm(CSP),
            norm(image_proxy::SHELL_CSP_TIGHTENED),
            "shipped shell CSP must equal e6's SHELL_CSP_TIGHTENED"
        );
        // The Trusted-Types enforcement directive is now present (a default TT policy
        // ships in apps/web/src/main.tsx).
        assert!(
            norm(CSP).contains("require-trusted-types-for 'script'"),
            "shell CSP enforces Trusted Types"
        );
    }

    #[test]
    fn export_format_covers_w7_msg_and_docx() {
        assert!(export_format("msg").is_some());
        assert!(export_format("docx").is_some());
        assert!(export_format("eml").is_some());
        assert!(export_format("nope").is_none());
    }

    /// One `Range` header → a [`HeaderMap`] carrying it.
    fn range_req(value: &str) -> HeaderMap {
        let mut m = HeaderMap::new();
        m.insert(header::RANGE, HeaderValue::from_str(value).unwrap());
        m
    }

    #[test]
    fn range_parsing_resolves_the_three_byte_range_forms() {
        // start-end, clamped to the last byte.
        assert_eq!(
            parse_range(&range_req("bytes=2-5"), 10),
            RangeDecision::Partial { start: 2, end: 5 }
        );
        assert_eq!(
            parse_range(&range_req("bytes=2-99"), 10),
            RangeDecision::Partial { start: 2, end: 9 }
        );
        // Open-ended.
        assert_eq!(
            parse_range(&range_req("bytes=4-"), 10),
            RangeDecision::Partial { start: 4, end: 9 }
        );
        // Suffix; a suffix longer than the blob yields the whole blob.
        assert_eq!(
            parse_range(&range_req("bytes=-3"), 10),
            RangeDecision::Partial { start: 7, end: 9 }
        );
        assert_eq!(
            parse_range(&range_req("bytes=-50"), 10),
            RangeDecision::Partial { start: 0, end: 9 }
        );
        // The unit is case-insensitive; whitespace is tolerated.
        assert_eq!(
            parse_range(&range_req("BYTES= 0-0 "), 10),
            RangeDecision::Partial { start: 0, end: 0 }
        );
        // No header at all.
        assert_eq!(parse_range(&HeaderMap::new(), 10), RangeDecision::Full);
    }

    #[test]
    fn malformed_and_multi_ranges_fall_back_to_the_whole_representation() {
        for raw in [
            "bytes=",                      // no spec
            "bytes=-",                     // bare dash
            "bytes=abc-def",               // non-numeric
            "bytes=5-2",                   // last < first: invalid, so ignored
            "items=0-1",                   // unit we do not understand
            "0-1",                         // no unit at all
            "bytes=0-1,5-6",               // multi-range: answered whole, by choice
            "bytes=-1,-2",                 // multi-range suffix form
            "bytes=99999999999999999999-", // overflows u64
        ] {
            assert_eq!(
                parse_range(&range_req(raw), 10),
                RangeDecision::Full,
                "{raw} must be ignored, not rejected"
            );
        }
    }

    #[test]
    fn ranges_outside_the_blob_are_unsatisfiable() {
        assert_eq!(
            parse_range(&range_req("bytes=10-20"), 10),
            RangeDecision::Unsatisfiable
        );
        assert_eq!(
            parse_range(&range_req("bytes=-0"), 10),
            RangeDecision::Unsatisfiable
        );
        // Nothing overlaps an empty extent.
        assert_eq!(
            parse_range(&range_req("bytes=0-0"), 0),
            RangeDecision::Unsatisfiable
        );
        // …but an empty blob with no Range is still a plain 200.
        assert_eq!(parse_range(&HeaderMap::new(), 0), RangeDecision::Full);
    }

    /// Status, headers and body bytes of a [`blob_response`].
    async fn blob_parts(req: &HeaderMap) -> (StatusCode, HeaderMap, Vec<u8>) {
        let resp = blob_response("text/plain", "note.txt", b"0123456789".to_vec(), req);
        let status = resp.status();
        let headers = resp.headers().clone();
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec();
        (status, headers, body)
    }

    #[tokio::test]
    async fn download_advertises_ranges_and_serves_a_single_one() {
        // No Range: full body, but the capability is advertised.
        let (status, h, body) = blob_parts(&HeaderMap::new()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(h.get(header::ACCEPT_RANGES).unwrap(), "bytes");
        assert!(h.get(header::CONTENT_RANGE).is_none());
        assert_eq!(h.get(header::CONTENT_LENGTH).unwrap(), "10");
        assert_eq!(body, b"0123456789");

        // A single range: 206 with the exact slice, length and Content-Range.
        let (status, h, body) = blob_parts(&range_req("bytes=2-5")).await;
        assert_eq!(status, StatusCode::PARTIAL_CONTENT);
        assert_eq!(body, b"2345");
        assert_eq!(h.get(header::CONTENT_RANGE).unwrap(), "bytes 2-5/10");
        assert_eq!(h.get(header::CONTENT_LENGTH).unwrap(), "4");
        assert_eq!(h.get(header::ACCEPT_RANGES).unwrap(), "bytes");
        // A partial response still describes what it is part of.
        assert_eq!(h.get(header::CONTENT_TYPE).unwrap(), "text/plain");
        assert!(
            h.get(header::CONTENT_DISPOSITION)
                .unwrap()
                .to_str()
                .unwrap()
                .contains("note.txt")
        );

        // Resume shape: the tail of a partly-fetched download.
        let (status, h, body) = blob_parts(&range_req("bytes=8-")).await;
        assert_eq!(status, StatusCode::PARTIAL_CONTENT);
        assert_eq!(body, b"89");
        assert_eq!(h.get(header::CONTENT_RANGE).unwrap(), "bytes 8-9/10");

        // Multi-range is answered whole, with a 200 — never a broken 206.
        let (status, h, body) = blob_parts(&range_req("bytes=0-1,4-5")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, b"0123456789");
        assert!(h.get(header::CONTENT_RANGE).is_none());
    }

    #[tokio::test]
    async fn a_range_past_the_end_is_416_with_the_real_length() {
        let (status, h, body) = blob_parts(&range_req("bytes=64-128")).await;
        assert_eq!(status, StatusCode::RANGE_NOT_SATISFIABLE);
        assert_eq!(h.get(header::CONTENT_RANGE).unwrap(), "bytes */10");
        assert_eq!(h.get(header::ACCEPT_RANGES).unwrap(), "bytes");
        assert!(body.is_empty());
        // No representation is being sent, so it is not described as one.
        assert!(h.get(header::CONTENT_DISPOSITION).is_none());
    }

    #[tokio::test]
    async fn the_413_body_is_stable_json_naming_the_limit() {
        let resp = upload_too_large();
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let v: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["error"], "payload too large");
        assert_eq!(v["limit"], "maxSizeUpload");
        assert_eq!(v["maxBytes"], MAX_UPLOAD_BYTES);
        // The advertised session limit and the enforced one are the same number.
        assert_eq!(MAX_UPLOAD_BYTES, 50_000_000);
    }

    #[test]
    fn seal_key_bytes_roundtrips_hex() {
        let key = ServerKey::generate();
        let bytes = seal_key_bytes(&key);
        assert_eq!(bytes.len(), 32, "seal key is 32 raw bytes");
        // Re-encoding the decoded bytes reproduces the hex form.
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, key.to_hex());
    }

    #[test]
    fn resolves_relative_api_url_against_server_origin() {
        assert_eq!(
            resolve_api_url("http://mock:8181/.well-known/jmap", "/jmap"),
            "http://mock:8181/jmap"
        );
        assert_eq!(
            resolve_api_url("http://mock:8181", "jmap"),
            "http://mock:8181/jmap"
        );
        assert_eq!(
            resolve_api_url("http://ignored", "https://real.example/api"),
            "https://real.example/api"
        );
    }

    fn jmap_session_with(api: &str, download: &str, upload: &str) -> mw_jmap::Session {
        serde_json::from_value(json!({
            "apiUrl": api,
            "downloadUrl": download,
            "uploadUrl": upload,
            "eventSourceUrl": "https://push.elsewhere.example/events",
        }))
        .unwrap()
    }

    #[test]
    fn session_urls_must_share_the_upstream_origin() {
        let up = "https://jmap.example.org/.well-known/jmap";
        // Relative and same-origin absolute URLs pass; `eventSourceUrl` is not
        // relayed and is not checked.
        assert!(
            session_urls_on_origin(
                up,
                &jmap_session_with(
                    "/jmap",
                    "https://jmap.example.org/dl/{accountId}/{blobId}/{name}",
                    ""
                )
            )
            .is_ok()
        );
        // A scheme-relative path is joined as a path on the upstream, not a host.
        assert!(session_urls_on_origin(up, &jmap_session_with("//10.0.0.5/api", "", "")).is_ok());
        for foreign in [
            "http://169.254.169.254/latest/meta-data/",
            "http://jmap.example.org/jmap",       // scheme differs
            "https://jmap.example.org:8443/jmap", // port differs
            "https://jmap.example.org@10.0.0.5/jmap",
            "https://{accountId}.example.org/x",
        ] {
            for session in [
                jmap_session_with(foreign, "", ""),
                jmap_session_with("/jmap", foreign, ""),
                jmap_session_with("/jmap", "", foreign),
            ] {
                assert!(
                    session_urls_on_origin(up, &session).is_err(),
                    "{foreign} must be refused"
                );
            }
        }
    }

    #[test]
    fn the_allowlist_matches_origins_and_its_absence_is_the_address_floor() {
        let url = reqwest::Url::parse("https://jmap.example.org/.well-known/jmap").unwrap();
        let floor = upstream_address_policy(&SecurityConfig::default(), &url).unwrap();
        assert!(!floor(&"127.0.0.1".parse().unwrap()));
        assert!(!floor(&"169.254.169.254".parse().unwrap()));
        assert!(!floor(&"10.1.2.3".parse().unwrap()));
        assert!(floor(&"93.184.215.14".parse().unwrap()));

        let listed = |entries: &[&str]| SecurityConfig {
            jmap_upstreams: Some(entries.iter().map(|e| e.to_string()).collect()),
            ..SecurityConfig::default()
        };
        // Named origin (a path in the entry is ignored; the default port is implied):
        // no range refused, because the operator chose it.
        for entry in [
            "https://jmap.example.org",
            " https://jmap.example.org:443/ ",
            "https://JMAP.example.org/.well-known/jmap",
        ] {
            let policy = upstream_address_policy(&listed(&[entry]), &url)
                .unwrap_or_else(|_| panic!("{entry:?} names the origin"));
            assert!(policy(&"10.1.2.3".parse().unwrap()));
        }
        for entry in [
            "http://jmap.example.org",
            "https://jmap.example.org:8443",
            "https://other.example.org",
            "jmap.example.org",
            "https://user:pw@jmap.example.org",
        ] {
            assert!(
                upstream_address_policy(&listed(&[entry]), &url).is_err(),
                "{entry:?} does not name the origin"
            );
        }
        // Set but empty: nothing is permitted.
        assert!(upstream_address_policy(&listed(&[]), &url).is_err());
        assert_eq!(
            parse_jmap_upstreams(" https://a.example, ,http://b.example:8080 "),
            vec!["https://a.example", "http://b.example:8080"]
        );
    }

    #[test]
    fn proxied_download_type_drops_parameters_and_active_types() {
        assert_eq!(proxied_download_type(Some("image/png")), "image/png");
        assert_eq!(
            proxied_download_type(Some("Application/PDF; name=\"x.pdf\"")),
            "application/pdf"
        );
        for active in [
            "text/html",
            "text/html; charset=utf-8",
            "application/xhtml+xml",
            "image/svg+xml",
            "text/xml",
            "application/javascript",
            "text/javascript",
            "multipart/x-mixed-replace; boundary=x",
            "garbage",
            "text/plain\r\nX-Injected: 1",
            "",
        ] {
            assert_eq!(
                proxied_download_type(Some(active)),
                "application/octet-stream",
                "{active:?}"
            );
        }
        assert_eq!(proxied_download_type(None), "application/octet-stream");
    }

    #[test]
    fn origin_extraction() {
        assert_eq!(
            origin_of("http://mock:8181/a/b").as_deref(),
            Some("http://mock:8181")
        );
        assert_eq!(
            origin_of("https://mail.example.org").as_deref(),
            Some("https://mail.example.org")
        );
        assert!(origin_of("not-a-url").is_none());
    }

    #[test]
    fn only_sso_idp_callbacks_are_csrf_exempt() {
        // The IdP-signed landings are exempt…
        assert!(is_sso_idp_callback("/api/sso/corp-oidc/callback"));
        assert!(is_sso_idp_callback("/api/sso/corp-saml/acs"));
        // …but nothing else on the SSO surface (logout/begin/providers stay guarded)…
        assert!(!is_sso_idp_callback("/api/sso/logout"));
        assert!(!is_sso_idp_callback("/api/sso/corp/begin"));
        assert!(!is_sso_idp_callback("/api/sso/corp-oidc/metadata"));
        // …and no id ⇒ no exemption; unrelated paths ⇒ no exemption.
        assert!(!is_sso_idp_callback("/api/sso//callback"));
        assert!(!is_sso_idp_callback("/api/login"));
        assert!(!is_sso_idp_callback("/admin/sso"));
    }

    #[test]
    fn cookie_parsing_picks_the_right_pair() {
        let mut h = HeaderMap::new();
        h.insert(
            header::COOKIE,
            HeaderValue::from_static("other=1; mw_session=abc123; x=2"),
        );
        assert_eq!(cookie_value(&h).as_deref(), Some("abc123"));
    }
}
