//! Mailwoman server (SPEC §4/§7, plan §2): session auth with an opaque
//! cookie, a same-origin JMAP proxy that injects upstream Basic auth, an
//! `/api/sanitize` endpoint that runs untrusted HTML through the disposable
//! `mw-render` child process (the §7.5 boundary), and the embedded SPA.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::anyhow;
use axum::body::{Body, Bytes};
use axum::extract::{Request, State};
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
use mw_store::{Credentials, ServerKey, Store};

pub mod engine_mode;
pub mod fonts;
pub mod hardening;
pub mod push;
pub mod tls;

pub use engine_mode::ServerMode;
pub use hardening::HardeningConfig;
pub use push::PushHandle;
pub use tls::{ReloadableResolver, TlsConfig, TlsListener};

use hardening::SessionGuard;

/// Cookie carrying the opaque session token.
const COOKIE_NAME: &str = "mw_session";

/// Strict CSP for the SPA shell (SPEC §7.4). The message body is rendered in
/// a separate sandboxed `<iframe srcdoc>` with its own restrictions.
const CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
     img-src 'self' blob: data:; font-src 'self'; connect-src 'self'; frame-src 'self'; \
     worker-src 'self' blob:; base-uri 'none'; form-action 'none'";

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
}

/// Build the fully-wired axum application from configuration.
pub async fn build_app(config: AppConfig) -> anyhow::Result<Router> {
    Ok(build_app_with_push(config).await?.0)
}

/// Like [`build_app`] but also returns the [`PushHandle`] feeding the realtime
/// channel. In engine mode the handle mirrors the `Engine` broadcast; tests use
/// the returned handle to inject synthetic `StateChange`s and prove the WS/SSE
/// wire path without a live engine.
pub async fn build_app_with_push(config: AppConfig) -> anyhow::Result<(Router, PushHandle)> {
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
    let store = Store::open(&config.db_path, key).await?;
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

    let state = AppState {
        store,
        render_bin,
        web_dir: config.web_dir,
        cookie_secure: config.cookie_secure,
        engine,
        push: push.clone(),
        sessions: Arc::new(SessionGuard::new()),
        hardening: config.hardening,
    };
    Ok((router(state), push))
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/me", get(me))
        .route("/api/session/rotate", post(rotate_session))
        .route("/api/discover", post(discover))
        .route("/api/sanitize", post(sanitize))
        .route("/jmap/session", get(jmap_session))
        .route("/jmap/api", post(jmap_api))
        .route("/jmap/ws", get(push::jmap_ws))
        .route("/jmap/eventsource", get(push::jmap_eventsource))
        .route("/healthz", get(|| async { "ok" }))
        .fallback(static_handler)
        // Innermost: reject cross-origin / missing-CSRF writes before handlers.
        .layer(middleware::from_fn_with_state(
            state.clone(),
            state_change_guard,
        ))
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
    next.run(req).await
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
    authenticated_response(
        &state,
        &id,
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
    authenticated_response(
        state,
        &id,
        json!({
            "ok": true,
            "accountId": account_id,
            "username": username,
        }),
    )
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
        Err(resp) => return resp,
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

fn clear_cookie(secure: bool) -> HeaderValue {
    let mut c = format!("{COOKIE_NAME}=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0");
    if secure {
        c.push_str("; Secure");
    }
    HeaderValue::from_str(&c).expect("cookie value is ascii")
}

fn cookie_value(headers: &HeaderMap) -> Option<String> {
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
    fn cookie_parsing_picks_the_right_pair() {
        let mut h = HeaderMap::new();
        h.insert(
            header::COOKIE,
            HeaderValue::from_static("other=1; mw_session=abc123; x=2"),
        );
        assert_eq!(cookie_value(&h).as_deref(), Some("abc123"));
    }
}
