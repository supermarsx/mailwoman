//! Mailwoman server (SPEC §4/§7, plan §2): session auth with an opaque
//! cookie, a same-origin JMAP proxy that injects upstream Basic auth, an
//! `/api/sanitize` endpoint that runs untrusted HTML through the disposable
//! `mw-render` child process (the §7.5 boundary), and the embedded SPA.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::anyhow;
use axum::body::{Body, Bytes};
use axum::extract::{Extension, Path as UrlPath, Query, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, Uri, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine as _;
use rand::RngCore;
use serde::Deserialize;
use serde_json::json;

use mw_engine::Engine;
use mw_jmap::JmapClient;
use mw_store::{Credentials, NativeSessionRow, ServerKey, Store};

pub mod arf;
pub mod dlp;
pub mod engine_mode;
pub mod fonts;
pub mod hardening;
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
pub mod rest;
pub mod webhooks;
// V7 additive route modules (plan §3): the GAL directory, password self-service,
// the Assist hook, the plugin registry, and the Nextcloud bridge. All are
// implemented and mounted into the V7 route group in `router()` below.
pub mod assist;
pub mod directory;
pub mod nextcloud;
pub mod passwd;
pub mod plugins;
// V7 MOUNT/WIRE (plan §3 e14): builds + injects the five V7 extensions, backs the
// host-service seams, the extra endpoints, and the countersign snapshot. Owned by e14.
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
// V6 MOUNT (t6-e11): store adapters backing the frozen Batch-B persistence seams
// over the real 0007 tables.
mod stores_v6;
// V6 scoped-API-key enforcement middleware (t6-e11b): the `Send` guard that lets a
// scoped `mwk_…` key authorize `/api/v1/*` REST + adds IP-allowlist/rate-limit to
// `/mcp`. Keeps the cookie path unchanged.
mod scope_mw;

pub use engine_mode::ServerMode;
pub use hardening::HardeningConfig;
pub use push::PushHandle;
pub use push_relay::NativeAuthConfig;
pub use tls::{ReloadableResolver, TlsConfig, TlsListener};
pub use watermark::WatermarkConfig;

use hardening::SessionGuard;

/// Cookie carrying the opaque session token.
const COOKIE_NAME: &str = "mw_session";

/// Strict CSP for the SPA shell (SPEC §7.4). The message body is rendered in
/// a separate sandboxed `<iframe srcdoc>` with its own restrictions.
// `'wasm-unsafe-eval'` permits WebAssembly compilation/instantiation ONLY (not
// JS `eval()`) — required for the client-side crypto worker (mw-crypto +
// mw-sanitize wasm, plan §1.1/§1.3). It does not weaken the XSS posture the way
// `'unsafe-eval'` would; untrusted message bodies render under the far stricter
// per-message [`MESSAGE_CSP`] in a sandboxed iframe, unaffected by this.
const CSP: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; \
     style-src 'self' 'unsafe-inline'; img-src 'self' blob: data:; font-src 'self'; \
     connect-src 'self' blob:; frame-src 'self'; worker-src 'self' blob:; \
     base-uri 'none'; form-action 'none'";

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
        }
    }
}

#[derive(Clone)]
pub(crate) struct AppState {
    store: Store,
    render_bin: Option<PathBuf>,
    web_dir: Option<PathBuf>,
    cookie_secure: bool,
    /// Present only in engine mode; drives IMAP/POP3 behind the JMAP surface.
    engine: Option<Arc<Engine>>,
    /// Realtime push fan-out feeding `/jmap/ws` + `/jmap/eventsource`.
    pub(crate) push: PushHandle,
    /// In-process session idle/absolute-timeout tracking.
    sessions: Arc<SessionGuard>,
    hardening: HardeningConfig,
    /// V4 crypto/security endpoint config (plan §3 e7).
    security: SecurityConfig,
    /// V5 native-client CORS/origin allowlist (plan §2.2). Empty = OFF (default) →
    /// browser deployments emit no `Access-Control-*` headers.
    native_auth: NativeAuthConfig,
    /// V6 MOUNT (plan §3 e11): the mounted OAuth AS, admin domain logic, and the
    /// store [`ServerKey`] backing the admin/oauth/mcp/zero-access surfaces.
    pub(crate) v6: Arc<V6State>,
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
    Ok(mw_admin::Admin::new(
        Arc::new(stores_v6::AdminBackendAdapter::new(store)),
        mw_admin::AdminConfig::default(),
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
    let key = match &config.server_key_hex {
        Some(h) => ServerKey::from_hex(h).map_err(|_| anyhow!("MW_SERVER_KEY is not valid hex"))?,
        None => {
            let k = ServerKey::generate();
            tracing::warn!(
                "MW_SERVER_KEY not set; generated an ephemeral key {} (set it to persist sessions across restarts)",
                k.to_hex()
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
    match &render_bin {
        Some(p) => tracing::info!("render worker: {}", p.display()),
        None => {
            tracing::warn!("mw-render binary not found; /api/sanitize will sanitize in-process")
        }
    }
    let engine = match config.mode {
        ServerMode::Engine => {
            tracing::info!("engine mode: driving IMAP/POP3 accounts behind the JMAP surface");
            Some(Arc::new(Engine::new(store.clone())))
        }
        ServerMode::Proxy => None,
    };
    // Realtime push: engine mode bridges the engine broadcast into the channel.
    let push = PushHandle::new();
    if let Some(engine) = &engine {
        tokio::spawn(push::bridge_engine(engine.subscribe(), push.clone()));
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
        reqwest::Client::new(),
    ));

    // ── V6 MOUNT (plan §3 e11) ───────────────────────────────────────────────
    // OAuth 2.1 AS + scoped API keys, backed by the 0007 tables.
    let auth = Arc::new(mw_oauth::AuthServer::new(
        stores_v6::OAuthStoreAdapter::new(store.clone()),
    ));
    // Admin domain logic (audit log + provisioning) over the 0007 tables.
    let admin = mw_admin::Admin::new(
        Arc::new(stores_v6::AdminBackendAdapter::new(store.clone())),
        mw_admin::AdminConfig::default(),
    );
    if !v6config.admin_enabled {
        let _ = admin.set_enabled("system", false).await;
    }

    // Outbound webhooks: a second consumer of the StateChange broadcast, backed by
    // the sealed-secret 0007 `webhooks` table (unseal via the store key).
    let webhook_registry: Arc<dyn webhooks::WebhookRegistry> = Arc::new(
        stores_v6::WebhookRegistryAdapter::new(store.clone(), key.clone()),
    );
    tokio::spawn(webhooks::run_webhook_dispatcher(
        webhook_registry,
        push.subscribe_relay(),
        reqwest::Client::new(),
    ));

    // Observability (OTLP/metrics/errors/inbound-webhook secret) — all off unless
    // the operator configures the corresponding env var.
    let obs = observability::ObservabilityConfig::from_env();
    observability::init_metrics(&obs);
    if let Ok(Some(guard)) = observability::init_otlp(&obs) {
        // Keep the exporter alive for the process lifetime.
        std::mem::forget(guard);
    }
    errors::set_error_config(errors::ErrorConfig::from_env());
    webhooks::set_inbound_secret(
        std::env::var("MW_WEBHOOK_INBOUND_SECRET")
            .ok()
            .filter(|s| !s.is_empty())
            .map(String::into_bytes),
    );

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
        let posture = Arc::new(stores_v6::StorePostureSource::load(&store).await);
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
    let passwd_backend = v7_mount::build_passwd_backend(&store);
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
        // The §10.8 spam-classification hook from the first approved+enabled
        // `spam-action` plugin (rspamd/SpamAssassin); `None` ⇒ ingest byte-unchanged.
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
    }

    // The `/mcp` Streamable-HTTP router over the REAL engine (a no-op mount in
    // proxy mode — tools return an engine error but `tools/list` still works). The
    // countersign resolver now reads the REAL admin `unattended_send` flag from the
    // 0007 `api_keys` table (folded V6 follow-up b) — no longer an empty stub.
    let countersigned = v7_mount::load_countersigned_prefixes(&store).await;
    let mcp_router = engine.as_ref().map(|engine| {
        let audit = stores_v6::AdminOAuthAudit::new(admin.clone());
        mcp::build_mcp_router(engine.clone(), auth.clone(), audit, countersigned.clone())
    });

    let v6 = Arc::new(V6State {
        auth,
        admin,
        admin_enabled: v6config.admin_enabled,
        admin_username: v6config.admin_username,
        admin_password: v6config.admin_password,
        pairing: std::sync::Mutex::new(std::collections::HashMap::new()),
    });

    let state = AppState {
        store,
        render_bin,
        web_dir: config.web_dir,
        cookie_secure: config.cookie_secure,
        engine,
        push: push.clone(),
        sessions: Arc::new(SessionGuard::new()),
        hardening: config.hardening,
        security: config.security,
        native_auth: NativeAuthConfig::from_env(),
        v6,
    };
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

    // ── V7 MOUNT (plan §3 e14) ────────────────────────────────────────────────
    // The five e9 route factories + the extra e14 endpoints, each layered with the
    // request extension it needs (`Extension<X>` — mirrors e9's mount contract). A
    // missing extension ⇒ the handler 500s, so every one is injected here.
    let v7_routes = directory::directory_router()
        .merge(passwd::passwd_router())
        .merge(assist::assist_router())
        .merge(plugins::plugins_router())
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
        .merge(admin_sso::admin_sso_router())
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

    let mut v6 = admin::routes()
        .merge(oauth::routes())
        .merge(rest)
        .merge(v7_routes)
        .merge(sso_routes)
        .merge(t10_routes)
        // t14 (26.14): the admin-gated JWZ backfill endpoint (§Wave-B E-mount).
        .merge(admin_maintenance::admin_maintenance_router())
        .route("/metrics", get(observability::metrics))
        .route("/errors", post(errors::report_error))
        .route("/api/webhooks/inbound", post(webhooks::inbound_webhook));
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

    Router::new()
        .merge(v6)
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/me", get(me))
        .route("/api/session/rotate", post(rotate_session))
        .route("/api/discover", post(discover))
        .route("/api/sanitize", post(sanitize))
        .route("/api/import/oft", post(import_oft))
        .route("/jmap/session", get(jmap_session))
        .route("/jmap/api", post(jmap_api))
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
        .route("/api/push/subscribe", post(push_relay::push_subscribe))
        .route("/api/push/unsubscribe", post(push_relay::push_unsubscribe))
        .route("/healthz", get(|| async { "ok" }))
        .fallback(static_handler)
        // Innermost: reject cross-origin / missing-CSRF writes before handlers.
        .layer(middleware::from_fn_with_state(
            state.clone(),
            state_change_guard,
        ))
        // V5 (plan §2.2): config-gated CORS/preflight for native shell origins. OFF
        // by default (`MW_NATIVE_ORIGINS` empty) → passes through untouched, so the
        // browser path emits no `Access-Control-*` headers.
        .layer(middleware::from_fn_with_state(state.clone(), native_cors))
        // Outermost: security headers on every response (incl. guard rejections).
        .layer(middleware::from_fn_with_state(
            state.clone(),
            security_headers,
        ))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Security headers (SPEC §7.4)
// ---------------------------------------------------------------------------

async fn security_headers(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let mut resp = next.run(req).await;
    let h = resp.headers_mut();
    h.insert("content-security-policy", HeaderValue::from_static(CSP));
    h.insert("x-frame-options", HeaderValue::from_static("DENY"));
    h.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    h.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    h.insert(
        "cross-origin-opener-policy",
        HeaderValue::from_static("same-origin"),
    );
    // Additive §7.4 deltas: COEP/CORP/Permissions-Policy.
    hardening::apply_extra_headers(h, state.hardening.coep);
    resp
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
        if is_sso_idp_callback(req.uri().path()) {
            return next.run(req).await;
        }
        // V5 (plan §2.2): a native bearer request carries no ambient cookie
        // authority (origin-agnostic, no cookie) → it is not a CSRF vector, so the
        // cookie-only Origin/double-submit guard is skipped for it. Cookie/browser
        // requests are handled byte-identically below.
        if push_relay::bearer_token(req.headers()).is_none() {
            if !hardening::origin_ok(req.headers()) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "cross-origin request rejected" })),
                )
                    .into_response();
            }
            // Pre-auth bootstrap routes have no prior token to double-submit; they
            // are covered by the Origin check + SameSite=Strict cookie instead.
            let csrf_exempt = matches!(req.uri().path(), "/api/login" | "/api/discover");
            if state.hardening.csrf_strict
                && !csrf_exempt
                && !hardening::csrf_double_submit_ok(req.headers())
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

/// V5 config-gated CORS (plan §2.2). OFF by default (`native_auth` empty): passes
/// through untouched so browser deployments see NO `Access-Control-*` headers. When
/// enabled, an allowed `Origin` is echoed back (bearer auth carries no cookies, so
/// no credentials mode is needed) and preflight `OPTIONS` is answered directly.
async fn native_cors(State(state): State<AppState>, req: Request, next: Next) -> Response {
    if !state.native_auth.is_enabled() {
        return next.run(req).await;
    }
    let origin = req
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let allowed = origin
        .as_deref()
        .map(|o| state.native_auth.allows(o))
        .unwrap_or(false);
    if req.method() == axum::http::Method::OPTIONS {
        let mut resp = StatusCode::NO_CONTENT.into_response();
        if allowed {
            add_cors_headers(resp.headers_mut(), origin.as_deref());
        }
        return resp;
    }
    let mut resp = next.run(req).await;
    if allowed {
        add_cors_headers(resp.headers_mut(), origin.as_deref());
    }
    resp
}

/// Emit the `Access-Control-*` headers for an allowed native shell origin.
fn add_cors_headers(h: &mut HeaderMap, origin: Option<&str>) {
    if let Some(value) = origin.and_then(|o| HeaderValue::from_str(o).ok()) {
        h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, value);
    }
    h.insert(header::VARY, HeaderValue::from_static("Origin"));
    h.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, OPTIONS"),
    );
    h.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("authorization, content-type, x-csrf-token"),
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
    jmap_url: String,
    username: String,
    password: String,
    /// V5 native-auth mode (plan §2.2): a native shell sends `"native"` to get a
    /// bearer-token session (a `native_sessions` row, NO cookie) in the response
    /// body instead of the cookie. ADDITIVE — absent for browser logins, so the
    /// cookie/same-origin path is byte-identical.
    #[serde(default)]
    client_type: Option<String>,
}

/// Uniform 401 — never leak which of URL/user/password was wrong (SPEC §7.4).
fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "invalid credentials" })),
    )
        .into_response()
}

async fn login(State(state): State<AppState>, Json(body): Json<LoginReq>) -> Response {
    if let Some(engine) = &state.engine {
        return engine_login(&state, engine, body).await;
    }
    let client = match JmapClient::new(&body.username, &body.password) {
        Ok(c) => c,
        Err(_) => return unauthorized(),
    };
    // Validate credentials by fetching the upstream Session server-side.
    let session = match client.session(&body.jmap_url).await {
        Ok(s) => s,
        Err(_) => return unauthorized(),
    };
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
    let api_url = resolve_api_url(&body.jmap_url, &session.api_url);
    let creds = Credentials {
        username: body.username.clone(),
        password: body.password.clone(),
    };
    let id = match state
        .store
        .create_session(&account_id, &username, &body.jmap_url, &api_url, &creds)
        .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("failed to persist session: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response();
        }
    };
    state.sessions.begin(&id);
    finish_login(
        &state,
        &id,
        &account_id,
        &username,
        body.client_type.as_deref(),
    )
    .await
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
        return Json(json!({
            "ok": true,
            "accountId": account_id,
            "username": username,
            "token": id,
        }))
        .into_response();
    }
    authenticated_response(
        state,
        id,
        json!({
            "ok": true,
            "accountId": account_id,
            "username": username,
        }),
    )
}

/// Engine-mode login: `jmapUrl` is read as an `imap(s)://`/`pop3(s)://` server
/// URL. Connects the account, then issues the same session cookie the proxy path
/// uses so the browser flow is identical.
async fn engine_login(state: &AppState, engine: &Arc<Engine>, body: LoginReq) -> Response {
    let (account_id, username) =
        match engine_mode::engine_login(engine, &body.jmap_url, &body.username, &body.password)
            .await
        {
            Ok(pair) => pair,
            Err(e) => {
                tracing::warn!("engine login failed: {e}");
                return unauthorized();
            }
        };
    let creds = Credentials {
        username: body.username.clone(),
        password: body.password.clone(),
    };
    let id = match state
        .store
        .create_session(&account_id, &username, "engine", "engine", &creds)
        .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("failed to persist engine session: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response();
        }
    };
    state.sessions.begin(&id);
    finish_login(
        state,
        &id,
        &account_id,
        &username,
        body.client_type.as_deref(),
    )
    .await
}

/// Build a login/rotate success response: set the session cookie, mint a fresh
/// double-submit CSRF token (cookie + body), and return the JSON payload.
fn authenticated_response(state: &AppState, id: &str, mut body: serde_json::Value) -> Response {
    let token = new_csrf_token();
    if let Some(obj) = body.as_object_mut() {
        obj.insert("csrfToken".into(), json!(token));
    }
    let mut resp = Json(body).into_response();
    let h = resp.headers_mut();
    h.append(header::SET_COOKIE, session_cookie(id, state.cookie_secure));
    h.append(header::SET_COOKIE, csrf_cookie(&token, state.cookie_secure));
    resp
}

/// A random, unguessable double-submit CSRF token.
fn new_csrf_token() -> String {
    let mut bytes = [0u8; 24];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// `POST /api/discover {email}` → the autoconfig ladder's candidate server set
/// (plan §2.4). Pre-login, so unauthenticated; additive in both modes.
#[derive(Debug, Deserialize)]
struct DiscoverReq {
    email: String,
}

async fn discover(Json(body): Json<DiscoverReq>) -> Response {
    match mw_autoconfig::discover(&body.email).await {
        Ok(candidate) => Json(candidate).into_response(),
        Err(mw_autoconfig::DiscoverError::InvalidEmail(_)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid email address" })),
        )
            .into_response(),
        Err(mw_autoconfig::DiscoverError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "no configuration discovered" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
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
    let session = match authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let mut body = json!({
        "username": session.username,
        "accountId": session.account_id,
    });
    // Ensure the client holds a CSRF token without rotating one it already has.
    let existing = hardening::cookie(&headers, CSRF_COOKIE);
    let token = existing.clone().unwrap_or_else(new_csrf_token);
    if let Some(obj) = body.as_object_mut() {
        obj.insert("csrfToken".into(), json!(token));
    }
    let mut resp = Json(body).into_response();
    if existing.is_none() {
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
    let _ = state.store.delete_session(&old_id).await;
    state.sessions.rotate(&old_id, &new_id);
    authenticated_response(&state, &new_id, json!({ "ok": true }))
}

// ---------------------------------------------------------------------------
// JMAP proxy
// ---------------------------------------------------------------------------

async fn jmap_session(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    if let Some(engine) = &state.engine {
        if let Err(e) = engine_mode::ensure_account(engine, &session.account_id).await {
            tracing::warn!("engine account not available: {e}");
            return upstream_error();
        }
        return Json(mw_engine::session_json(
            &session.account_id,
            &session.username,
        ))
        .into_response();
    }
    let client = match JmapClient::new(&session.credentials.username, &session.credentials.password)
    {
        Ok(c) => c,
        Err(_) => return upstream_error(),
    };
    let mut upstream = match client.session(&session.jmap_url).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("upstream session fetch failed: {e}");
            return upstream_error();
        }
    };
    // Rewrite every URL so the browser only ever talks to us, never upstream.
    upstream.api_url = "/jmap/api".to_string();
    upstream.download_url = "/jmap/download/{accountId}/{blobId}/{name}".to_string();
    upstream.upload_url = "/jmap/upload/{accountId}".to_string();
    upstream.event_source_url = "/jmap/eventsource".to_string();
    Json(upstream).into_response()
}

async fn jmap_api(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    let session = match authed(&state, &headers).await {
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
    if let Some(engine) = &state.engine {
        if let Err(e) = engine_mode::ensure_account(engine, &session.account_id).await {
            tracing::warn!("engine account not available: {e}");
            return upstream_error();
        }
        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(e) => {
                return (StatusCode::BAD_REQUEST, format!("bad json: {e}")).into_response();
            }
        };
        let response = engine.handle_jmap(&session.account_id, &request).await;
        return Json(response).into_response();
    }
    let client = match JmapClient::new(&session.credentials.username, &session.credentials.password)
    {
        Ok(c) => c,
        Err(_) => return upstream_error(),
    };
    match client.request_raw(&session.api_url, body).await {
        Ok((status, bytes)) => {
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
// Blob download + export (plan §2.4 / §10.5, e14)
// ---------------------------------------------------------------------------

/// `GET /jmap/download/{accountId}/{blobId}/{name}` — the JMAP downloadUrl the
/// session rewrites to (RFC 8620 §6.2). Cookie-authed. In engine mode the blobId
/// is resolved locally by [`mw_engine::Engine::fetch_blob`] (whole message
/// `<stableId>` → `message/rfc822`; a part `<stableId>.<partId>` → its decoded
/// bytes). In proxy mode the request is forwarded to the upstream downloadUrl
/// with injected Basic auth and streamed back verbatim.
async fn jmap_download(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath((account_id, blob_id, name)): UrlPath<(String, String, String)>,
) -> Response {
    let session = match authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    // A session may only download blobs from its own account.
    if account_id != session.account_id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "account mismatch" })),
        )
            .into_response();
    }
    if let Some(engine) = &state.engine {
        match engine.fetch_blob(&account_id, &blob_id).await {
            Ok(Some(blob)) => {
                let filename = if name.is_empty() {
                    &blob.filename
                } else {
                    &name
                };
                blob_response(&blob.content_type, filename, blob.bytes)
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
        proxy_download(&session, &account_id, &blob_id, &name).await
    }
}

/// Proxy a download to the upstream JMAP server: fetch its Session for the real
/// downloadUrl template, substitute the coordinates, GET it with injected auth,
/// and stream status + content headers + body straight back to the browser.
async fn proxy_download(
    session: &mw_store::Session,
    account_id: &str,
    blob_id: &str,
    name: &str,
) -> Response {
    let client = match JmapClient::new(&session.credentials.username, &session.credentials.password)
    {
        Ok(c) => c,
        Err(_) => return upstream_error(),
    };
    let upstream = match client.session(&session.jmap_url).await {
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
    match client.get_bytes(&abs).await {
        Ok((status, content_type, content_disposition, bytes)) => {
            let code = StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let mut resp = Response::new(Body::from(bytes));
            *resp.status_mut() = code;
            let h = resp.headers_mut();
            if let Some(ct) = content_type.and_then(|v| HeaderValue::from_str(&v).ok()) {
                h.insert(header::CONTENT_TYPE, ct);
            }
            if let Some(cd) = content_disposition.and_then(|v| HeaderValue::from_str(&v).ok()) {
                h.insert(header::CONTENT_DISPOSITION, cd);
            }
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
    UrlPath(account_id): UrlPath<String>,
    body: Body,
) -> Response {
    let session = match authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    // A session may only upload to its own account.
    if account_id != session.account_id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "account mismatch" })),
        )
            .into_response();
    }
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .unwrap_or_else(|| "application/octet-stream".to_string());
    // Read the body with a hard cap at `maxSizeUpload`; a larger body → 413 (before
    // any storage write or upstream forwarding). `Body` is extracted directly so the
    // read is bounded here rather than by the default (smaller) extractor limit.
    let bytes = match axum::body::to_bytes(body, MAX_UPLOAD_BYTES).await {
        Ok(b) => b,
        Err(_) => return upload_too_large(),
    };
    let Some(engine) = &state.engine else {
        return proxy_upload(&session, &account_id, &content_type, bytes).await;
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

/// `413` for an upload body over `maxSizeUpload`. Factual, concrete size in the body.
fn upload_too_large() -> Response {
    (
        StatusCode::PAYLOAD_TOO_LARGE,
        Json(json!({
            "error": format!("upload exceeds maxSizeUpload ({MAX_UPLOAD_BYTES} bytes)"),
        })),
    )
        .into_response()
}

/// Forward an upload to the upstream JMAP server (proxy mode): fetch its Session for
/// the real uploadUrl template, substitute the accountId, POST the bytes with injected
/// auth + the client's `Content-Type`, and relay the upstream `{accountId, blobId,
/// type, size}` response (status + content-type + body) straight back — the symmetric
/// counterpart of [`proxy_download`].
async fn proxy_upload(
    session: &mw_store::Session,
    account_id: &str,
    content_type: &str,
    bytes: Bytes,
) -> Response {
    let client = match JmapClient::new(&session.credentials.username, &session.credentials.password)
    {
        Ok(c) => c,
        Err(_) => return upstream_error(),
    };
    let upstream = match client.session(&session.jmap_url).await {
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
    match client.post_bytes(&abs, content_type, bytes).await {
        Ok((status, ct, body)) => {
            let code = StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let mut resp = Response::new(Body::from(body));
            *resp.status_mut() = code;
            if let Some(ct) = ct.and_then(|v| HeaderValue::from_str(&v).ok()) {
                resp.headers_mut().insert(header::CONTENT_TYPE, ct);
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
    let session = match authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
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
        Ok(out) => blob_response(content_type, &format!("{stable_id}.{ext}"), out),
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
        _ => None,
    }
}

/// Build a download response with `Content-Type`, `Content-Disposition`
/// (attachment) and `Content-Length` set.
fn blob_response(content_type: &str, filename: &str, bytes: Vec<u8>) -> Response {
    let len = bytes.len();
    let mut resp = Response::new(Body::from(bytes));
    let h = resp.headers_mut();
    if let Ok(v) = HeaderValue::from_str(content_type) {
        h.insert(header::CONTENT_TYPE, v);
    }
    let disposition = format!("attachment; filename=\"{}\"", sanitize_filename(filename));
    if let Ok(v) = HeaderValue::from_str(&disposition) {
        h.insert(header::CONTENT_DISPOSITION, v);
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

/// `GET /api/security/dlp/config` — DLP config load (§7.6 / plan §3 e7): parses
/// `MW_DLP_RULES` and surfaces the active rules (same frozen `DlpRule` shape the
/// engine enforces) so the web can name them pre-send. Cookie-authed.
async fn dlp_config(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(resp) = authed(&state, &headers).await {
        return resp;
    }
    let rules = match &state.security.dlp_rules {
        Some(source) => match dlp::load_rules(source) {
            Ok(rules) => rules,
            Err(e) => {
                tracing::error!("MW_DLP_RULES failed to load: {e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "DLP rules are misconfigured" })),
                )
                    .into_response();
            }
        },
        None => Vec::new(),
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
            Err(e) => {
                tracing::warn!("render child failed ({e}); falling back to in-process sanitize");
                mw_sanitize::sanitize_email_html(&body.html)
            }
        },
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
            // No render worker: parse in-process (documented fallback, like sanitize).
            tracing::warn!("mw-render binary not found; parsing .oft in-process");
            match base64::engine::general_purpose::STANDARD.decode(body.content_base64.as_bytes()) {
                Ok(bytes) => mw_export::from_oft(&bytes)
                    .map(|p| {
                        (
                            mw_sanitize::sanitize_email_html(&p.body.unwrap_or_default()),
                            p.subject,
                        )
                    })
                    .map_err(|e| anyhow!("{e}")),
                Err(e) => Err(anyhow!("bad base64: {e}")),
            }
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

async fn static_handler(State(state): State<AppState>, uri: Uri) -> Response {
    let raw = uri.path().trim_start_matches('/');
    let path = if raw.is_empty() { "index.html" } else { raw };

    if let Some(resp) = serve_asset(&state, path) {
        return resp;
    }
    // SPA fallback: unknown non-asset routes get index.html.
    serve_asset(&state, "index.html")
        .unwrap_or_else(|| (StatusCode::NOT_FOUND, "not found").into_response())
}

fn serve_asset(state: &AppState, path: &str) -> Option<Response> {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    if let Some(dir) = &state.web_dir {
        let full = dir.join(path);
        // Guard against path traversal escaping the web dir.
        if !full.starts_with(dir) {
            return None;
        }
        let bytes = std::fs::read(&full).ok()?;
        return Some(asset_response(mime.as_ref(), bytes));
    }
    let file = WebAssets::get(path)?;
    Some(asset_response(mime.as_ref(), file.data.into_owned()))
}

fn asset_response(mime: &str, bytes: Vec<u8>) -> Response {
    let mut resp = Response::new(Body::from(bytes));
    if let Ok(v) = HeaderValue::from_str(mime) {
        resp.headers_mut().insert(header::CONTENT_TYPE, v);
    }
    resp
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

/// Resolve the authenticated session from the cookie, or return a 401 response.
/// Also enforces the idle/absolute session timeouts (§7.4): an expired session is
/// deleted and rejected.
pub(crate) async fn authed(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<mw_store::Session, Response> {
    // V5 (plan §2.2): a native client presents `Authorization: Bearer <token>`
    // instead of the cookie. Absent for browsers → the cookie path below is
    // byte-identical.
    if let Some(token) = push_relay::bearer_token(headers) {
        return authed_native(state, &token).await;
    }
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
fn csrf_cookie(token: &str, secure: bool) -> HeaderValue {
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
mod tests {
    use super::*;

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
