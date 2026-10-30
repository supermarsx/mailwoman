//! Optional browser-client entry gate. It grants no mailbox/admin authority.
//! Protocol integrations and operator recovery retain their own authentication.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson, etag_of, with_etag},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Form, Json, Router,
    extract::{DefaultBodyLimit, Request, State},
    http::{Extensions, HeaderMap, Method, StatusCode, header},
    middleware::Next,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use mw_store::{AuthSurface, Store};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub(crate) const KEY: &str = "security.clientAccess.v1";
const COOKIE: &str = "mw_client_access";
const PENDING: &str = "mw_client_access_pending";
const UNLOCK: &str = "/api/client-access/unlock";
const PURPOSE: &str = "client-interface-access-v1";
const GATE_HEADER: &str = "x-mailwoman-client-gate";
static HASH_WORK: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(2);
#[derive(Clone)]
pub(crate) struct GatePageResponse;
pub(crate) const PAGE_CSP: &str = "default-src 'none'; style-src 'self'; img-src 'self'; base-uri 'none'; form-action 'self'; frame-ancestors 'none'";

// Deliberately no Debug: neither the stored hash nor entry slug belongs in logs.
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Config {
    psk_enabled: bool,
    slug_enabled: bool,
    slug: Option<String>,
    psk_hash: Option<String>,
    grant_hours: u16,
    revision: String,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            psk_enabled: false,
            slug_enabled: false,
            slug: None,
            psk_hash: None,
            grant_hours: 24,
            revision: "disabled-v1".into(),
        }
    }
}
impl Config {
    fn enabled(&self) -> bool {
        self.psk_enabled || self.slug_enabled
    }
    fn parse(raw: Option<&str>) -> anyhow::Result<Self> {
        let Some(raw) = raw else {
            return Ok(Self::default());
        };
        anyhow::ensure!(raw.len() <= 8192, "Invalid client-access configuration");
        let cfg: Self = serde_json::from_str(raw)?;
        anyhow::ensure!(
            (1..=168).contains(&cfg.grant_hours) && cfg.revision.len() <= 64,
            "Invalid client-access configuration"
        );
        anyhow::ensure!(
            !cfg.psk_enabled || cfg.psk_hash.is_some(),
            "Client access key is not configured"
        );
        anyhow::ensure!(
            !cfg.slug_enabled || cfg.slug.is_some(),
            "Client entry slug is not configured"
        );
        if let Some(slug) = &cfg.slug {
            validate_slug(slug).map_err(anyhow::Error::msg)?;
        }
        Ok(cfg)
    }
}
async fn load(store: &Store) -> anyhow::Result<(Option<String>, Config)> {
    let raw = store.get_setting(KEY).await?;
    let cfg = Config::parse(raw.as_deref())?;
    Ok((raw, cfg))
}
fn validate_slug(slug: &str) -> Result<(), &'static str> {
    if !(4..=64).contains(&slug.len())
        || !slug
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        || !slug.as_bytes()[0].is_ascii_alphanumeric()
        || !slug.as_bytes()[slug.len() - 1].is_ascii_alphanumeric()
    {
        return Err(
            "Use 4–64 lowercase letters, digits or hyphens, beginning and ending with a letter or digit.",
        );
    }
    if [
        "admin", "api", "jmap", "dav", "mcp", "oauth", "oidc", "assets", "static", "health",
        "healthz", "ready", "readyz", "metrics", "mail", "login", "logout", "settings", "calendar",
        "contacts", "tasks", "notes", "files", "search", "compose", "inbox", "outbox", "people",
        "security", "webhooks", "hooks", "errors",
    ]
    .contains(&slug)
    {
        return Err("This entry slug is reserved for an application route.");
    }
    Ok(())
}
fn conflicts(slug: &str, path: &str) -> bool {
    path.trim_matches('/').split('/').next() == Some(slug)
}
/// Reverse check, also used by console-path writes and startup resolution.
pub(crate) async fn check_console_path(
    store: &Store,
    path: &str,
) -> anyhow::Result<Option<String>> {
    let (raw, cfg) = load(store).await?;
    anyhow::ensure!(
        !cfg.slug_enabled || !cfg.slug.as_deref().is_some_and(|s| conflicts(s, path)),
        "Console path conflicts with the enabled client entry slug"
    );
    Ok(raw)
}
fn view(state: &AppState, cfg: &Config) -> Value {
    json!({"pskEnabled":cfg.psk_enabled,"slugEnabled":cfg.slug_enabled,"slug":cfg.slug,"pskConfigured":cfg.psk_hash.is_some(),"grantHours":cfg.grant_hours,"clientEntryPath":if cfg.slug_enabled {format!("{}/{}",state.base_path,cfg.slug.as_deref().unwrap_or_default())}else{format!("{}/",state.base_path)}})
}
fn tag(cfg: &Config) -> String {
    etag_of(&json!({"revision":cfg.revision}))
}
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/client-access",
        Capability::SettingsRead,
        read,
    );
    r.admin_endpoint(
        Method::PUT,
        "/client-access",
        Capability::SettingsWrite,
        write,
    );
}
pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/assets/client-access.css", get(styles))
        .route(UNLOCK, post(unlock).layer(DefaultBodyLimit::max(1024)))
}
async fn styles() -> Response {
    ([(header::CONTENT_TYPE,"text/css; charset=utf-8"),(header::CACHE_CONTROL,"public, max-age=3600")],
    "html{color-scheme:light dark;font:16px system-ui}body{margin:0;min-height:100vh;display:grid;place-items:center}main{width:min(28rem,calc(100% - 3rem));padding:2rem;box-sizing:border-box;border:1px solid #8886;border-radius:1rem}h1{font-size:1.5rem}label,input,button{display:block;box-sizing:border-box;width:100%;margin-top:1rem}input,button{font:inherit;padding:.8rem;border:1px solid #888;border-radius:.5rem}button{cursor:pointer}p{line-height:1.5}").into_response()
}
async fn read(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsRead)?;
    let (_, cfg) = load(&state.store).await.map_err(AdminApiError::internal)?;
    Ok(with_etag(
        (
            [(header::CACHE_CONTROL, "private, no-store")],
            Json(view(&state, &cfg)),
        ),
        &tag(&cfg),
    ))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Update {
    psk_enabled: bool,
    slug_enabled: bool,
    slug: Option<String>,
    grant_hours: u16,
    #[serde(default)]
    psk: Option<String>,
    #[serde(default)]
    clear_psk: bool,
}
async fn write(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(body): AdminJson<Update>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsWrite)?;
    let (raw, old) = load(&state.store).await.map_err(AdminApiError::internal)?;
    let Some(sent) = headers.get(header::IF_MATCH).and_then(|h| h.to_str().ok()) else {
        return Err(AdminApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "ifMatchRequired",
            "Read the current client-access policy before saving",
        ));
    };
    if sent != tag(&old) {
        return Err(AdminApiError::stale(None, None));
    }
    if !(1..=168).contains(&body.grant_hours) {
        return Err(AdminApiError::invalid(
            "invalidGrantHours",
            "Use 1–168 hours",
            "/grantHours",
        ));
    }
    let pending_console = state
        .store
        .get_setting(crate::console_path::STORED_KEY)
        .await
        .map_err(AdminApiError::internal)?;
    if let Some(slug) = &body.slug {
        validate_slug(slug).map_err(|e| AdminApiError::invalid("invalidEntrySlug", e, "/slug"))?;
        if conflicts(slug, &mw_admin::config::admin_path())
            || pending_console
                .as_deref()
                .is_some_and(|p| conflicts(slug, p))
            || conflicts(slug, &state.base_path)
        {
            return Err(AdminApiError::invalid(
                "entrySlugCollision",
                "This entry slug conflicts with the console or deployment path",
                "/slug",
            ));
        }
    }
    if body.slug_enabled && body.slug.is_none() {
        return Err(AdminApiError::invalid(
            "entrySlugRequired",
            "Configure an entry slug before enabling it",
            "/slug",
        ));
    }
    let supplied = body.psk.filter(|s| !s.is_empty());
    if supplied
        .as_ref()
        .is_some_and(|s| !(16..=256).contains(&s.len()) || s.trim().is_empty())
    {
        return Err(AdminApiError::invalid(
            "invalidAccessKey",
            "Use an access key of 16–256 UTF-8 bytes; spaces are preserved",
            "/psk",
        ));
    }
    if body.clear_psk && (body.psk_enabled || supplied.is_some()) {
        return Err(AdminApiError::invalid(
            "invalidKeyClear",
            "Disable key protection before clearing its stored key",
            "/clearPsk",
        ));
    }
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::BanManage).await?;
    let key_changed = supplied.is_some() || body.clear_psk;
    let hash = if let Some(secret) = supplied {
        let permit = HASH_WORK.try_acquire().map_err(|_| {
            AdminApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "keyWorkBusy",
                "Try again shortly",
            )
        })?;
        Some(
            tokio::task::spawn_blocking(move || {
                let _permit = permit;
                mw_admin::password::hash_password(&secret, &mw_admin::SecurityPolicy::default())
            })
            .await
            .map_err(AdminApiError::internal)?
            .map_err(AdminApiError::internal)?,
        )
    } else if body.clear_psk {
        None
    } else {
        old.psk_hash.clone()
    };
    if body.psk_enabled && hash.is_none() {
        return Err(AdminApiError::invalid(
            "accessKeyRequired",
            "Configure a key before enabling key protection",
            "/psk",
        ));
    }
    let next = Config {
        psk_enabled: body.psk_enabled,
        slug_enabled: body.slug_enabled,
        slug: body.slug,
        psk_hash: hash,
        grant_hours: body.grant_hours,
        revision: uuid::Uuid::new_v4().to_string(),
    };
    let encoded = serde_json::to_string(&next).map_err(AdminApiError::internal)?;
    if !state
        .store
        .compare_entry_setting(KEY, raw.as_deref(), &encoded, pending_console.as_deref())
        .await
        .map_err(AdminApiError::internal)?
    {
        return Err(AdminApiError::stale(None, None));
    }
    crate::audit::security_event(&state,crate::audit::SecurityEvent::admin(&ctx.username,"client-access-policy-changed").target("setting:client-access").detail(json!({"pskEnabled":next.psk_enabled,"slugEnabled":next.slug_enabled,"keyChanged":key_changed,"slugChanged":next.slug!=old.slug,"grantHours":next.grant_hours}))).await;
    Ok(with_etag(
        (
            [(header::CACHE_CONTROL, "private, no-store")],
            Json(view(&state, &next)),
        ),
        &tag(&next),
    ))
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Proof {
    revision: String,
    expires: i64,
    grant: bool,
    nonce: String,
}
fn signed(store: &Store, proof: &Proof) -> String {
    let data = URL_SAFE_NO_PAD.encode(serde_json::to_vec(proof).expect("proof serializes"));
    let mac = crate::webhooks::sign(
        store.signin_challenge_secret(PURPOSE).as_bytes(),
        data.as_bytes(),
    );
    format!("{data}.{}", mac.trim_start_matches("sha256="))
}
fn verified(store: &Store, cfg: &Config, token: &str, grant: bool) -> Option<Proof> {
    if token.len() > 1024 {
        return None;
    }
    let (data, mac) = token.split_once('.')?;
    if !crate::webhooks::verify_signature(
        store.signin_challenge_secret(PURPOSE).as_bytes(),
        data.as_bytes(),
        &format!("sha256={mac}"),
    ) {
        return None;
    }
    let proof: Proof = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(data).ok()?).ok()?;
    let now = chrono::Utc::now().timestamp();
    (proof.revision == cfg.revision
        && proof.grant == grant
        && proof.expires > now
        && proof.expires <= now + 7 * 86400
        && proof.nonce.len() == 36)
        .then_some(proof)
}
fn cookie_proof(state: &AppState, cfg: &Config, headers: &HeaderMap, grant: bool) -> Option<Proof> {
    let raw = crate::hardening::cookie(headers, if grant { COOKIE } else { PENDING })?;
    verified(&state.store, cfg, &raw, grant)
}
fn new_proof(cfg: &Config, grant: bool) -> Proof {
    Proof {
        revision: cfg.revision.clone(),
        expires: chrono::Utc::now().timestamp()
            + if grant {
                i64::from(cfg.grant_hours) * 3600
            } else {
                600
            },
        grant,
        nonce: uuid::Uuid::new_v4().to_string(),
    }
}
fn set_proof(state: &AppState, response: &mut Response, proof: &Proof, secure: bool) {
    let name = if proof.grant { COOKIE } else { PENDING };
    let path = format!("{}/", state.base_path);
    let max_age = proof
        .expires
        .saturating_sub(chrono::Utc::now().timestamp())
        .max(0);
    let value = format!(
        "{name}={}; Path={path}; Max-Age={max_age}; HttpOnly; SameSite=Strict{}",
        signed(&state.store, proof),
        if secure { "; Secure" } else { "" }
    );
    response.headers_mut().append(
        header::SET_COOKIE,
        value.parse().expect("bounded proof cookie"),
    );
}
fn mark(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(GATE_HEADER, "enabled".parse().unwrap());
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "private, no-store".parse().unwrap());
    response
        .headers_mut()
        .insert(header::REFERRER_POLICY, "no-referrer".parse().unwrap());
    response
}
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn prompt(state: &AppState, proof: &Proof, error: Option<&str>, status: StatusCode) -> Response {
    let action = escape(&format!("{}{UNLOCK}", state.base_path));
    let stylesheet = escape(&format!("{}/assets/client-access.css", state.base_path));
    let notice = error
        .map(|s| format!("<p role=\"alert\">{}</p>", escape(s)))
        .unwrap_or_default();
    let nonce = escape(&proof.nonce);
    let html = format!(
        r#"<!doctype html><html lang="en"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="robots" content="noindex,nofollow"><title>Client access</title>
<link rel="stylesheet" href="{stylesheet}"></head><body><main><h1>Client access</h1>
<p>Enter the access key provided by your administrator to continue.</p>{notice}
<form method="post" action="{action}"><input type="hidden" name="nonce" value="{nonce}">
<label for="access-key">Access key</label><input id="access-key" name="psk" type="password" autocomplete="current-password" maxlength="256" required autofocus>
<button type="submit">Continue</button></form></main></body></html>"#
    );
    let mut response = mark((status, Html(html)).into_response());
    response.extensions_mut().insert(GatePageResponse);
    response
}
fn redirect(state: &AppState) -> Response {
    (
        StatusCode::SEE_OTHER,
        [
            (header::LOCATION, format!("{}/", state.base_path)),
            (header::CACHE_CONTROL, "private, no-store".into()),
        ],
    )
        .into_response()
}
fn required(status: StatusCode) -> Response {
    mark((status,Json(json!({"code":"clientAccessRequired","error":"Client access is required. Open the configured client entry address."}))).into_response())
}
fn exempt(path: &str) -> bool {
    // Opaque sandbox frames and external recipients have no browser gate cookie.
    // These exact resource shapes retain their signed-capability validation;
    // minting/signing and policy endpoints are deliberately NOT exempt.
    if ["/api/rc/i/", "/api/rc/p/", "/api/rc/f/", "/api/rc/d/"]
        .iter()
        .any(|prefix| {
            path.strip_prefix(prefix)
                .is_some_and(|token| !token.is_empty() && !token.contains('/'))
        })
        || path
            .strip_prefix("/t/")
            .is_some_and(|file| file.ends_with(".gif") && !file.contains('/'))
    {
        return true;
    }
    let admin = mw_admin::config::admin_path();
    if path == admin
        || path
            .strip_prefix(&admin)
            .is_some_and(|p| p.starts_with('/'))
    {
        return true;
    }
    if [
        "/dav",
        "/mcp",
        "/oauth",
        "/oidc",
        "/.well-known",
        "/assets",
        "/static",
    ]
    .iter()
    .any(|prefix| {
        path == *prefix
            || path
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with('/'))
    }) {
        return true;
    }
    matches!(
        path,
        "/health"
            | "/healthz"
            | "/ready"
            | "/readyz"
            | "/metrics"
            | "/api/public-config"
            | "/api/rc/frame"
            | "/api/webhooks/inbound"
            | "/errors"
            | "/api/logout"
            | "/api/sso/logout"
            | "/favicon.ico"
            | "/manifest.webmanifest"
            | "/manifest.json"
            | "/sw.js"
            | "/service-worker.js"
    ) || crate::is_sso_idp_callback(path)
}
pub(crate) async fn guard(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    if exempt(&path) || path == UNLOCK {
        return next.run(request).await;
    }
    let cfg = match load(&state.store).await {
        Ok((_, cfg)) => cfg,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    if !cfg.enabled() {
        return next.run(request).await;
    }
    let document = matches!(*request.method(), Method::GET | Method::HEAD)
        && (path == "/"
            || path == "/index.html"
            || crate::wants_spa_shell(request.method(), request.headers(), &path));
    let entry = cfg.slug_enabled
        && matches!(*request.method(), Method::GET | Method::HEAD)
        && cfg
            .slug
            .as_deref()
            .is_some_and(|s| path == format!("/{s}") || path == format!("/{s}/"));
    if cookie_proof(&state, &cfg, request.headers(), true).is_some() {
        if entry {
            return mark(redirect(&state));
        }
        let response = next.run(request).await;
        return if document { mark(response) } else { response };
    }
    let secure =
        state
            .external
            .cookie_secure(state.cookie_secure, request.headers(), request.extensions());
    if entry {
        let proof = new_proof(&cfg, !cfg.psk_enabled);
        let mut response = if cfg.psk_enabled {
            prompt(&state, &proof, None, StatusCode::OK)
        } else {
            mark(redirect(&state))
        };
        set_proof(&state, &mut response, &proof, secure);
        return response;
    }
    let pending = cookie_proof(&state, &cfg, request.headers(), false);
    if cfg.slug_enabled && pending.is_none() {
        return if document {
            mark((StatusCode::NOT_FOUND, "Not found").into_response())
        } else {
            required(StatusCode::FORBIDDEN)
        };
    }
    if document && cfg.psk_enabled {
        let proof = pending.unwrap_or_else(|| new_proof(&cfg, false));
        let mut response = prompt(&state, &proof, None, StatusCode::OK);
        set_proof(&state, &mut response, &proof, secure);
        return response;
    }
    required(StatusCode::FORBIDDEN)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Unlock {
    psk: String,
    nonce: String,
}
fn retry_prompt(state: &AppState, proof: &Proof, response: Response) -> Response {
    let status = response.status();
    if !matches!(
        status,
        StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE
    ) {
        return response;
    }
    let mut page = prompt(
        state,
        proof,
        Some("Please wait before trying again. Client access is temporarily unavailable."),
        status,
    );
    if let Some(value) = response.headers().get(header::RETRY_AFTER) {
        page.headers_mut()
            .insert(header::RETRY_AFTER, value.clone());
    }
    page
}
async fn unlock(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: Extensions,
    uri: axum::http::Uri,
    Form(body): Form<Unlock>,
) -> Response {
    // Unlike generic API writes, this form never grants an Origin exemption to
    // a claimed bearer/native client. It issues a browser authority cookie.
    if !crate::hardening::origin_ok(&headers, uri.authority().map(|a| a.as_str())) {
        return (StatusCode::FORBIDDEN, "Cross-origin request rejected").into_response();
    }
    let cfg = match load(&state.store).await {
        Ok((_, cfg)) => cfg,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    if !cfg.enabled() {
        return redirect(&state);
    }
    // The ordinary Origin/Fetch Metadata guard runs before this handler. The
    // independent form proof also works when an expired grant leaves auth cookies.
    let Some(proof) = cookie_proof(&state, &cfg, &headers, false) else {
        return required(StatusCode::FORBIDDEN);
    };
    let secret = state.store.signin_challenge_secret(PURPOSE);
    if body.nonce.len() != proof.nonce.len()
        || !crate::webhooks::verify_signature(
            secret.as_bytes(),
            body.nonce.as_bytes(),
            &crate::webhooks::sign(secret.as_bytes(), proof.nonce.as_bytes()),
        )
    {
        return required(StatusCode::FORBIDDEN);
    }
    let source = crate::scope_mw::proxy::client_ip(&headers, &ext);
    if let Err(error) = crate::signin_guard::check(&state, AuthSurface::Webmail, source).await {
        return retry_prompt(&state, &proof, error.into_response());
    }
    let admission = match crate::signin_cooldown::check(&state, &headers, &ext).await {
        Ok(a) => a,
        Err(response) => return retry_prompt(&state, &proof, response),
    };
    let Some(hash) = cfg.psk_hash.clone().filter(|_| cfg.psk_enabled) else {
        return required(StatusCode::FORBIDDEN);
    };
    let valid = if !(16..=256).contains(&body.psk.len()) {
        Ok(Ok(false))
    } else {
        let permit = match HASH_WORK.try_acquire() {
            Ok(p) => p,
            Err(_) => {
                return retry_prompt(
                    &state,
                    &proof,
                    (StatusCode::TOO_MANY_REQUESTS, [(header::RETRY_AFTER, "1")]).into_response(),
                );
            }
        };
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            mw_admin::password::verify_password(&body.psk, &hash)
        })
        .await
    };
    match valid {
        Ok(Ok(true)) => {
            if !load(&state.store)
                .await
                .is_ok_and(|(_, latest)| latest.revision == cfg.revision)
            {
                return required(StatusCode::FORBIDDEN);
            }
            admission.succeeded(&state);
            let grant = new_proof(&cfg, true);
            let mut response = mark(redirect(&state));
            let secure = state
                .external
                .cookie_secure(state.cookie_secure, &headers, &ext);
            set_proof(&state, &mut response, &grant, secure);
            response
        }
        Ok(Ok(false)) => {
            crate::request_protection::observe(
                mw_store::request_protection_model::Signal::CredentialRejected,
            );
            if let Some(ip) = source {
                let policy = match crate::signin_guard::policy(&state, AuthSurface::Webmail).await {
                    Ok(p) => p,
                    Err(error) => return retry_prompt(&state, &proof, error.into_response()),
                };
                if state
                    .store
                    .record_auth_source_failure(
                        AuthSurface::Webmail.as_str(),
                        ip,
                        chrono::Utc::now().timestamp(),
                        policy,
                    )
                    .await
                    .is_err()
                {
                    return retry_prompt(
                        &state,
                        &proof,
                        StatusCode::SERVICE_UNAVAILABLE.into_response(),
                    );
                }
            }
            prompt(
                &state,
                &proof,
                Some("The access key was not accepted."),
                StatusCode::FORBIDDEN,
            )
        }
        _ => retry_prompt(
            &state,
            &proof,
            StatusCode::SERVICE_UNAVAILABLE.into_response(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn grants_are_key_revision_stage_and_expiry_bound() {
        let key = mw_store::ServerKey::generate();
        let first = Store::open_in_memory(key.clone()).await.unwrap();
        let restarted = Store::open_in_memory(key).await.unwrap();
        let other = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let mut cfg = Config::default();
        cfg.revision = "rev1".into();
        let mut proof = new_proof(&cfg, true);
        let token = signed(&first, &proof);
        assert!(verified(&restarted, &cfg, &token, true).is_some());
        assert!(verified(&other, &cfg, &token, true).is_none());
        assert!(verified(&first, &cfg, &token, false).is_none());
        assert!(verified(&first, &cfg, &format!("{token}x"), true).is_none());
        cfg.revision = "rev2".into();
        assert!(verified(&first, &cfg, &token, true).is_none());
        cfg.revision = "rev1".into();
        proof.expires = chrono::Utc::now().timestamp() - 1;
        assert!(verified(&first, &cfg, &signed(&first, &proof), true).is_none());
    }
    #[test]
    fn exemptions_are_explicit_capabilities_not_authentication_header_claims() {
        for path in [
            "/api/rc/i/capability",
            "/api/rc/p/capability",
            "/api/rc/f/capability",
            "/api/rc/d/capability",
            "/api/rc/frame",
            "/t/token.gif",
            "/dav/calendars/a/b",
            "/mcp",
            "/oauth/token",
        ] {
            assert!(exempt(path), "{path}");
        }
        for path in [
            "/api/login",
            "/jmap/session",
            "/api/rc/sign",
            "/api/rc/stats",
            "/api/rc/i/token/extra",
            "/administrator",
            "/arbitrary/page",
            "/api/blob/file.png",
        ] {
            assert!(!exempt(path), "{path}");
        }
        for slug in [
            "admin",
            "errors",
            "api",
            "bad/path",
            "bad?query",
            "x",
            "-private",
            "private-",
            "UpperCase",
        ] {
            assert!(validate_slug(slug).is_err(), "{slug}");
        }
        assert!(validate_slug("private-entry-2").is_ok());
    }
}
