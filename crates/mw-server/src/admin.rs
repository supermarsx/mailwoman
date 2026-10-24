//! Admin-panel HTTP surface (SPEC §19, plan §2.5, §3 e11 MOUNT).
//!
//! A SEPARATE session domain: `/admin/*` runs under its own cookie, named
//! `mw_admin_session` unless `MW_ADMIN_SESSION_COOKIE` says otherwise
//! ([`mw_admin::config::session_cookie_name`]; `Path=/`, because admin routes also
//! live under `/api/admin/*`), distinct from the mailbox `mw_session`. Login validates the operator credentials
//! (`MW_ADMIN_USER`/`MW_ADMIN_PASSWORD`) and mints an admin session stored (hashed)
//! in `admin_sessions`. Every action drives [`mw_admin::Admin`] (backed by the 0007
//! tables via [`crate::stores_v6::AdminBackendAdapter`]), which writes the
//! append-only audit log. `MW_ADMIN_ENABLED=false` makes every route return `401`
//! (the panel is unreachable) — the CLI keeps working.
//!
//! **Session lifetime (0030).** A session ends after 30 minutes without a request
//! and never lasts more than 12 hours; both are enforced in
//! `Store::get_admin_session`, which every admin gate in this crate calls. The
//! cookie carries the 12-hour figure as `Max-Age`, so a browser drops it at the
//! same moment the server would refuse it.
//!
//! **Security policy (t29-e6).** `/admin/security-policy` reads and writes the
//! whole policy; every field is applied (`crate::admin_apply`, `mw_admin::policy`).
//! A field an `MW_ADMIN_*` variable pins is listed in `pinnedByEnv`, and a PUT that
//! changes it is refused with 409 naming the variable.
//!
//! **What these routes do not offer (26.20, t28-e8).** A stored value that nothing
//! in the workspace applies is not presented as a setting:
//! * `/admin/domains/{name}` registers a name. A domain's stored upstream JSON,
//!   allowlist and blocklist are neither returned nor accepted.
//! * `/admin/observability` and `/admin/appearance` still answer, for a later
//!   release that gives them a reader; the panel shows neither form.
//!
//! JSON is camelCase to satisfy the typed web client in `state/slices/admin.ts`.

use axum::Json;
use axum::Router;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use mw_admin::config::admin_route;
use mw_admin::{
    ActorKind, AdminError, CapturePolicy, Domain as AdminDomain, EffectiveSecurityPolicy, MinTls,
    Quota as AdminQuota, SecurityFloor, SecurityPolicy, SecurityPolicyWrite,
};

use crate::AppState;

/// The `/admin/*` router (merged by e11's `router()`).
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(&admin_route("/login"), post(login))
        .route(&admin_route("/logout"), post(logout))
        .route(&admin_route("/session"), get(session))
        .route(&admin_route("/domains"), get(list_domains))
        .route(
            &admin_route("/domains/{name}"),
            put(save_domain).delete(delete_domain),
        )
        .route(&admin_route("/users"), get(list_users).post(provision_user))
        .route(&admin_route("/users/{account_id}/quota"), put(set_quota))
        .route(&admin_route("/users/{account_id}/flags"), put(set_flags))
        .route(
            &admin_route("/users/{account_id}/zero-access"),
            post(toggle_zero_access),
        )
        .route(
            &admin_route("/users/{account_id}/revoke-sessions"),
            post(revoke_sessions),
        )
        .route(
            &admin_route("/security-policy"),
            get(get_policy).put(set_policy),
        )
        .route(&admin_route("/integrations"), get(get_integrations))
        .route(&admin_route("/webhooks"), get(list_webhooks))
        .route(&admin_route("/api-keys"), get(list_api_keys))
        .route(&admin_route("/api-keys/{id}/revoke"), post(revoke_api_key))
        .route(&admin_route("/observability"), get(get_obs).put(set_obs))
        .route(&admin_route("/audit"), get(list_audit))
        .route(&admin_route("/audit/export"), get(export_audit))
        .route(&admin_route("/bans"), get(list_bans).post(add_ban))
        .route(&admin_route("/bans/{ip}"), delete(remove_ban))
        .route(
            &admin_route("/appearance"),
            get(get_appearance).put(set_appearance),
        )
}

// ─── admin session helpers ────────────────────────────────────────────────────

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn admin_cookie(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    let prefix = format!("{}=", mw_admin::config::session_cookie_name());
    for part in raw.split(';') {
        if let Some(v) = part.trim().strip_prefix(&prefix)
            && !v.is_empty()
        {
            return Some(v.to_string());
        }
    }
    None
}

/// The admin cookie. `Max-Age` is the session's absolute cap: the idle deadline is
/// shorter and moves, and only the server can know it, so the cookie carries the
/// one figure that is fixed at login.
fn set_admin_cookie(token: &str, secure: bool) -> String {
    let mut c = format!(
        "{}={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}",
        mw_admin::config::session_cookie_name(),
        mw_store::Store::ADMIN_SESSION_MAX_SECS
    );
    if secure {
        c.push_str("; Secure");
    }
    c
}

fn clear_admin_cookie(secure: bool) -> String {
    let mut c = format!(
        "{}=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0",
        mw_admin::config::session_cookie_name()
    );
    if secure {
        c.push_str("; Secure");
    }
    c
}

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "admin authentication required" })),
    )
        .into_response()
}

/// Resolve the authenticated admin username, or a `401` response. Also enforces the
/// `admin.enabled` gate (disabled → every route is `401`). An expired session is a
/// `401` like an unknown one: `get_admin_session` answers `None` for both.
async fn admin_authed(state: &AppState, headers: &HeaderMap) -> Result<String, Response> {
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

fn err500(e: impl std::fmt::Display) -> Response {
    tracing::warn!("admin error: {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": "admin operation failed" })),
    )
        .into_response()
}

// ─── login monitor ────────────────────────────────────────────────────────────

/// Feed one admin-login failure to the login monitor and **emit its fail2ban
/// line**.
///
/// Emitting is the point. `mw_admin::banlist` exists to hand an operator's
/// fail2ban jail a line it can match, and the auto-ban this records is a bookkeeping
/// entry — nothing in the request path consults [`mw_admin::Admin::is_banned`], so
/// the jail is the enforcement. Until 26.19 the caller built that line and dropped
/// it on the floor (`let _ = …`), so a jail pointed at Mailwoman's log matched
/// nothing at all.
///
/// The line carries its own RFC 3339 timestamp in addition to the subscriber's.
/// That is harmless: [`mw_admin::FAIL2BAN_FAILREGEX`] anchors on the
/// `mailwoman[auth]:` token rather than on position, and fail2ban strips the
/// leading date before applying the filter.
async fn record_admin_login_failure(
    state: &AppState,
    username: &str,
    source: Option<std::net::IpAddr>,
) {
    if source.is_none() {
        warn_no_source_once();
    }
    match state.v6.admin.record_login_failure(username, source).await {
        Ok(outcome) => {
            if let Some(line) = outcome.log_line {
                // Deliberately the raw line, unstructured: a jail reads it verbatim.
                tracing::warn!("{line}");
            }
            // `banned` implies a resolved source — nothing is banned without one.
            if outcome.banned
                && let Some(ip) = source
            {
                tracing::warn!(
                    "admin login: {ip} crossed the failure threshold and was added to the ban list"
                );
            }
        }
        Err(e) => tracing::warn!("admin login-failure not recorded: {e}"),
    }
}

/// Say once that the monitor has nothing to work with.
///
/// No source address means the serve path installed no `ConnectInfo` — an
/// operator condition, not something a client can cause. The failure is still
/// audited, but it is not counted and no jail line is emitted, so an operator who
/// believed brute-force protection was running needs to hear that it is not.
/// Once per process: this fires on a login path an attacker can drive, and a
/// per-request warning would be a log-flood lever.
fn warn_no_source_once() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        tracing::warn!(
            "admin login failure has no client address (no ConnectInfo on the request): \
             the login monitor cannot count it and no fail2ban line is emitted"
        );
    });
}

// ─── session / login / logout ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct LoginReq {
    username: String,
    password: String,
}

async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    // t20 B3: the login monitor needs the address the attempt came from. It lives
    // in the request extensions as `ConnectInfo`, installed by the serve path.
    ext: axum::http::Extensions,
    Json(body): Json<LoginReq>,
) -> Response {
    if !state.v6.admin_enabled {
        return unauthorized();
    }
    // The one seam for "who is this". `proxy::client_ip` is the peer address,
    // refined by a forwarded header only when `MW_FORWARDED_MODE` selects one AND
    // the peer is a configured proxy. Reading `X-Forwarded-For` here instead would
    // reintroduce t20 B1 in a place where it is worse than the bug it replaces: a
    // ban key anyone can choose lets an attacker put a colleague's address — or a
    // shared corporate egress IP — in the ban list on demand.
    let source = crate::scope_mw::proxy::client_ip(&headers, &ext);
    let (Some(user), Some(pass)) = (&state.v6.admin_username, &state.v6.admin_password) else {
        return unauthorized();
    };
    if !ct_eq(body.username.as_bytes(), user.as_bytes())
        || !ct_eq(body.password.as_bytes(), pass.as_bytes())
    {
        record_admin_login_failure(&state, &body.username, source).await;
        return unauthorized();
    }
    let token = crate::push_relay::hash_token(&format!(
        "{}:{}",
        mw_store::ServerKey::generate().to_hex(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    let hash = crate::push_relay::hash_token(&token);
    if let Err(e) = state
        .store
        .put_admin_session(&hash, &body.username, &chrono::Utc::now().to_rfc3339())
        .await
    {
        return err500(e);
    }
    if let Err(e) = state
        .v6
        .admin
        .record_login_success(&body.username, source)
        .await
    {
        tracing::warn!("admin login-success not recorded: {e}");
    }
    // A new double-submit token with every login, as `/api/login` issues
    // (`authenticated_response`). The panel and the mailbox app share the one
    // `mw_csrf` cookie and each reads it when it sends a request, so replacing it
    // here does not strand an open mailbox tab.
    let csrf_token = crate::new_csrf_token();
    let mut resp = Json(AdminSessionDto {
        username: body.username,
        csrf_token: csrf_token.clone(),
    })
    .into_response();
    // The admin cookie stays the first `Set-Cookie`.
    resp.headers_mut().append(
        header::SET_COOKIE,
        set_admin_cookie(&token, state.cookie_secure)
            .parse()
            .unwrap(),
    );
    resp.headers_mut().append(
        header::SET_COOKIE,
        crate::csrf_cookie(&csrf_token, state.cookie_secure),
    );
    resp
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(token) = admin_cookie(&headers) {
        let hash = crate::push_relay::hash_token(&token);
        let _ = state.store.delete_admin_session(&hash).await;
    }
    let mut resp = StatusCode::NO_CONTENT.into_response();
    resp.headers_mut().append(
        header::SET_COOKIE,
        clear_admin_cookie(state.cookie_secure).parse().unwrap(),
    );
    resp
}

/// `GET /admin/session` — who is signed in. Also makes sure the panel holds a
/// CSRF token without replacing one it has (as `GET /api/me` does): a session
/// that has no `mw_csrf` cookie beside it gets one here, which is where the web
/// client asks after a write is refused for its token.
async fn session(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let username = match admin_authed(&state, &headers).await {
        Ok(username) => username,
        Err(resp) => return resp,
    };
    let existing = crate::hardening::cookie(&headers, crate::hardening::CSRF_COOKIE);
    let csrf_token = existing.clone().unwrap_or_else(crate::new_csrf_token);
    let mut resp = Json(AdminSessionDto {
        username,
        csrf_token: csrf_token.clone(),
    })
    .into_response();
    if existing.is_none() {
        resp.headers_mut().append(
            header::SET_COOKIE,
            crate::csrf_cookie(&csrf_token, state.cookie_secure),
        );
    }
    resp
}

// ─── domains ──────────────────────────────────────────────────────────────────

async fn list_domains(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(r) = admin_authed(&state, &headers).await {
        return r;
    }
    match state.v6.admin.list_domains().await {
        Ok(list) => Json(list.into_iter().map(DomainDto::from).collect::<Vec<_>>()).into_response(),
        Err(e) => err500(e),
    }
}

/// `PUT /admin/domains/{name}` — register a domain name. The name comes from the
/// path; no body is read. An existing domain is left as it is.
async fn save_domain(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(name): UrlPath<String>,
) -> Response {
    let Ok(actor) = admin_authed(&state, &headers).await else {
        return unauthorized();
    };
    let name = name.trim();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "a domain name is required" })),
        )
            .into_response();
    }
    match state.v6.admin.create_domain(&actor, name).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(e),
    }
}

async fn delete_domain(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(name): UrlPath<String>,
) -> Response {
    let Ok(actor) = admin_authed(&state, &headers).await else {
        return unauthorized();
    };
    match state.v6.admin.delete_domain(&actor, &name).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(e),
    }
}

// ─── users ────────────────────────────────────────────────────────────────────

async fn list_users(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(r) = admin_authed(&state, &headers).await {
        return r;
    }
    let users = match state.store.list_admin_users().await {
        Ok(u) => u,
        Err(e) => return err500(e),
    };
    let mut out = Vec::new();
    for u in users {
        let account_id = u.username;
        let (username, domain) = match account_id.rsplit_once('@') {
            Some((a, b)) => (a.to_string(), b.to_string()),
            None => (account_id.clone(), String::new()),
        };
        let quota = state
            .v6
            .admin
            .get_quota(&account_id)
            .await
            .ok()
            .flatten()
            .map(QuotaDto::from);
        let flags = state
            .v6
            .admin
            .get_feature_flags(&account_id)
            .await
            .map(FlagsDto::from)
            .unwrap_or_default();
        out.push(UserSummaryDto {
            account_id,
            username,
            domain,
            quota,
            flags,
        });
    }
    Json(out).into_response()
}

async fn provision_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ProvisionReq>,
) -> Response {
    let Ok(actor) = admin_authed(&state, &headers).await else {
        return unauthorized();
    };
    match state
        .v6
        .admin
        .provision_user(&actor, &body.domain, &body.username, body.quota.into())
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(e),
    }
}

async fn set_quota(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(account_id): UrlPath<String>,
    Json(body): Json<QuotaDto>,
) -> Response {
    let Ok(actor) = admin_authed(&state, &headers).await else {
        return unauthorized();
    };
    match state
        .v6
        .admin
        .set_quota(&actor, &account_id, body.into())
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(e),
    }
}

async fn set_flags(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(account_id): UrlPath<String>,
    Json(body): Json<FlagsDto>,
) -> Response {
    let Ok(actor) = admin_authed(&state, &headers).await else {
        return unauthorized();
    };
    match state
        .v6
        .admin
        .set_feature_flags(&actor, &account_id, body.into())
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(e),
    }
}

#[derive(Deserialize)]
struct ToggleReq {
    on: bool,
}

async fn toggle_zero_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(account_id): UrlPath<String>,
    Json(body): Json<ToggleReq>,
) -> Response {
    let Ok(actor) = admin_authed(&state, &headers).await else {
        return unauthorized();
    };
    match state
        .v6
        .admin
        .toggle_zero_access(&actor, &account_id, body.on)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(e),
    }
}

async fn revoke_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(account_id): UrlPath<String>,
) -> Response {
    let Ok(actor) = admin_authed(&state, &headers).await else {
        return unauthorized();
    };
    match state.v6.admin.revoke_sessions(&actor, &account_id).await {
        Ok(count) => Json(json!({ "count": count })).into_response(),
        Err(e) => err500(e),
    }
}

// ─── security policy ──────────────────────────────────────────────────────────

async fn get_policy(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(r) = admin_authed(&state, &headers).await {
        return r;
    }
    match state.v6.admin.effective_security_policy().await {
        Ok(p) => Json(policy_body(&state, &p)).into_response(),
        Err(e) => err500(e),
    }
}

/// `PUT /admin/security-policy` — write the whole policy ([`SecurityPolicyPut`]),
/// apply it to this process at once (`admin_apply::on_policy_change`), audit it,
/// and answer with the policy now in force (the GET body).
///
/// * A body that is not the full shape, names an unknown field, or has a value
///   outside a field's set (`minTls: "1.1"`) → 422 with the reason.
/// * An Argon2 cost out of range, or `dlpRulesJson` that is not a valid rule list
///   → 422 with the reason; nothing is stored.
/// * A change to a field an environment variable pins → 409 naming the variable.
async fn set_policy(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<SecurityPolicyPut>, JsonRejection>,
) -> Response {
    let Ok(actor) = admin_authed(&state, &headers).await else {
        return unauthorized();
    };
    let body = match body {
        Ok(Json(b)) => b,
        Err(rej) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "error": rej.body_text() })),
            )
                .into_response();
        }
    };
    let write = SecurityPolicyWrite {
        policy: SecurityPolicy {
            min_tls: body.min_tls,
            argon2_m_cost: body.argon2_m_cost,
            argon2_t_cost: body.argon2_t_cost,
            argon2_p_cost: body.argon2_p_cost,
            dlp_rules_json: body.dlp_rules_json,
            max_security_floor: body.max_security_floor,
            capture_policy: body.capture_policy,
        },
        require_2fa: body.require2fa,
    };
    match state.v6.admin.write_security_policy(&actor, write).await {
        Ok((old, new)) => {
            crate::admin_apply::on_policy_change(&old, &new);
            Json(policy_body(&state, &new)).into_response()
        }
        Err(AdminError::Pinned { field, var }) => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": format!("{field} is set by {var} and cannot be changed here"),
                "field": field,
                "variable": var,
            })),
        )
            .into_response(),
        Err(AdminError::Invalid(msg)) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "error": msg })),
        )
            .into_response(),
        Err(e) => err500(e),
    }
}

/// The GET body: every field of the policy in force, the `MW_DLP_RULES` rules
/// (read-only here: `envDlpRules`), and `pinnedByEnv`, the wire names of the
/// fields an environment variable pins.
fn policy_body(state: &AppState, p: &EffectiveSecurityPolicy) -> Value {
    let env_rules = match state.security.dlp_rules.as_deref() {
        Some(source) => crate::dlp::load_rules(source).unwrap_or_else(|e| {
            tracing::warn!("MW_DLP_RULES failed to load: {e}");
            Vec::new()
        }),
        None => Vec::new(),
    };
    json!({
        "minTls": p.policy.min_tls.as_str(),
        "require2fa": p.require_2fa,
        "argon2MCost": p.policy.argon2_m_cost,
        "argon2TCost": p.policy.argon2_t_cost,
        "argon2PCost": p.policy.argon2_p_cost,
        "capturePolicy": p.policy.capture_policy.as_str(),
        "maxSecurityFloor": p.policy.max_security_floor.map(SecurityFloor::as_str),
        "dlpRulesJson": p.policy.dlp_rules_json,
        "envDlpRules": env_rules,
        "pinnedByEnv": p.pinned_by_env.iter().map(|f| f.field).collect::<Vec<_>>(),
    })
}

// ─── integrations / oversight ─────────────────────────────────────────────────

/// Whether this process built a Nextcloud gateway at start.
///
/// The gateway handle lives in a request extension on the `/api/nextcloud/*`
/// routes, which this handler cannot see, so the answer is recomputed with the same
/// function the mount used (`v7_mount::build_nextcloud`, which reads the three
/// `MW_NEXTCLOUD_*` variables). Computed once: the mount ran it once too, and a
/// process's environment does not change under it.
fn nextcloud_configured() -> bool {
    static CONFIGURED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *CONFIGURED.get_or_init(|| crate::v7_mount::build_nextcloud().is_some())
}

/// `GET /admin/integrations` — what this deployment has configured.
///
/// * `ldap` — `configured` when the store holds at least one enabled
///   `directory_config` row, `not-configured` when it holds none, `unknown` when
///   the store could not be read. The running directory client was built from those
///   rows at start (`v7_mount::build_directory`), and no route writes them, so the
///   two agree unless the table was edited underneath a running server.
/// * `nextcloud` — `configured` when all three `MW_NEXTCLOUD_*` variables were set
///   at start, else `not-configured`.
///
/// Neither value says the remote service is reachable: nothing here connects to it.
async fn get_integrations(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(r) = admin_authed(&state, &headers).await {
        return r;
    }
    use mw_admin::IntegrationStatus;
    let i = state.v6.admin.integrations();
    let ldap = match state.store.list_directory_config().await {
        Ok(rows) if rows.iter().any(|r| r.enabled) => IntegrationStatus::Configured,
        Ok(_) => IntegrationStatus::NotConfigured,
        Err(e) => {
            tracing::warn!("admin integrations: directory config not readable: {e}");
            IntegrationStatus::Unknown
        }
    };
    let nextcloud = if nextcloud_configured() {
        IntegrationStatus::Configured
    } else {
        IntegrationStatus::NotConfigured
    };
    Json(json!({
        "webhooks": i.webhooks.as_str(),
        "apiKeyOversight": i.api_key_oversight.as_str(),
        "ldap": ldap.as_str(),
        "nextcloud": nextcloud.as_str(),
    }))
    .into_response()
}

async fn list_webhooks(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(r) = admin_authed(&state, &headers).await {
        return r;
    }
    match state.store.list_all_webhooks().await {
        Ok(list) => Json(
            list.into_iter()
                .map(|w| {
                    json!({
                        "id": w.id,
                        "accountId": w.account_id,
                        "url": w.url,
                        "eventFilterJson": w.event_filter_json,
                        "createdAt": w.created_at,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(e) => err500(e),
    }
}

/// `GET /admin/api-keys` — every key row, live and revoked; never a key hash.
///
/// `unattendedSendRequested` is the owner's request in the key's scope;
/// `unattendedSendApproved` is the `api_keys.unattended_send` column, which only
/// `PUT /admin/api-keys/{id}/unattended-send` (`oauth.rs`) writes.
async fn list_api_keys(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(r) = admin_authed(&state, &headers).await {
        return r;
    }
    match state.store.list_api_keys().await {
        Ok(list) => Json(
            list.into_iter()
                .map(|k| {
                    let scope: Value = serde_json::from_str(&k.scopes_json).unwrap_or(Value::Null);
                    json!({
                        "id": k.id,
                        "prefix": k.key_prefix,
                        "accountId": k.account_id,
                        "scopesJson": k.scopes_json,
                        "createdAt": k.created_at,
                        "lastUsedAt": k.last_used_at,
                        "expiresAt": scope.get("expires_at").cloned().unwrap_or(Value::Null),
                        "revokedAt": k.revoked_at,
                        "unattendedSendRequested": scope.get("unattended_send")
                            == Some(&Value::Bool(true)),
                        "unattendedSendApproved": k.unattended_send,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(e) => err500(e),
    }
}

async fn revoke_api_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let Ok(actor) = admin_authed(&state, &headers).await else {
        return unauthorized();
    };
    if let Err(e) = state
        .store
        .revoke_api_key_by_id(&id, &chrono::Utc::now().to_rfc3339())
        .await
    {
        return err500(e);
    }
    let _ = state.v6.admin.revoke_api_key(&actor, &id).await;
    StatusCode::NO_CONTENT.into_response()
}

// ─── observability / audit / bans ─────────────────────────────────────────────

/// `GET /admin/observability` — the stored telemetry record. Stored, not applied:
/// see [`mw_admin::ObservabilityConfig`]. The panel does not call this.
async fn get_obs(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(r) = admin_authed(&state, &headers).await {
        return r;
    }
    match state.v6.admin.get_observability().await {
        Ok(c) => Json(json!({
            "logLevel": c.log_level,
            "otlpDsn": c.otlp_dsn,
            "metricsEnabled": c.metrics_enabled,
            "sentryDsn": c.sentry_dsn,
        }))
        .into_response(),
        Err(e) => err500(e),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ObsDto {
    log_level: String,
    otlp_dsn: Option<String>,
    metrics_enabled: bool,
    sentry_dsn: Option<String>,
}

async fn set_obs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ObsDto>,
) -> Response {
    let Ok(actor) = admin_authed(&state, &headers).await else {
        return unauthorized();
    };
    let cfg = mw_admin::ObservabilityConfig {
        log_level: body.log_level,
        otlp_dsn: body.otlp_dsn,
        metrics_enabled: body.metrics_enabled,
        sentry_dsn: body.sentry_dsn,
    };
    match state.v6.admin.set_observability(&actor, cfg).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(e),
    }
}

#[derive(Deserialize)]
struct LimitQuery {
    limit: Option<usize>,
}

async fn list_audit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<LimitQuery>,
) -> Response {
    if let Err(r) = admin_authed(&state, &headers).await {
        return r;
    }
    let limit = q.limit.unwrap_or(50).min(1000);
    match state.v6.admin.list_audit(limit).await {
        Ok(list) => Json(list.into_iter().map(audit_dto).collect::<Vec<_>>()).into_response(),
        Err(e) => err500(e),
    }
}

async fn export_audit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<LimitQuery>,
) -> Response {
    if let Err(r) = admin_authed(&state, &headers).await {
        return r;
    }
    let limit = q.limit.unwrap_or(1000).min(100_000);
    match state.v6.admin.export_audit(limit).await {
        Ok(text) => ([(header::CONTENT_TYPE, "application/x-ndjson")], text).into_response(),
        Err(e) => err500(e),
    }
}

async fn list_bans(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(r) = admin_authed(&state, &headers).await {
        return r;
    }
    match state.v6.admin.list_bans().await {
        Ok(list) => Json(
            list.into_iter()
                .map(|b| {
                    json!({
                        "ip": b.ip,
                        "reason": b.reason,
                        "bannedAt": b.banned_at,
                        "expiresAt": b.expires_at,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(e) => err500(e),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BanReq {
    ip: String,
    reason: String,
    expires_at: Option<String>,
}

async fn add_ban(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<BanReq>,
) -> Response {
    let Ok(actor) = admin_authed(&state, &headers).await else {
        return unauthorized();
    };
    match state
        .v6
        .admin
        .ban_ip(&actor, &body.ip, &body.reason, body.expires_at)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(e),
    }
}

async fn remove_ban(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(ip): UrlPath<String>,
) -> Response {
    let Ok(actor) = admin_authed(&state, &headers).await else {
        return unauthorized();
    };
    match state.v6.admin.unban_ip(&actor, &ip).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(e),
    }
}

// ─── appearance ───────────────────────────────────────────────────────────────

/// `GET /admin/appearance` — the deployment-default appearance held in this
/// process's memory (see [`mw_admin::Admin::set_appearance`]: it is not persisted).
/// The panel does not call this.
async fn get_appearance(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(r) = admin_authed(&state, &headers).await {
        return r;
    }
    let a = state.v6.admin.config().appearance;
    Json(json!({ "theme": a.theme, "brandName": a.brand_name, "accent": a.accent })).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppearanceDto {
    theme: String,
    brand_name: String,
    accent: Option<String>,
}

async fn set_appearance(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AppearanceDto>,
) -> Response {
    let Ok(actor) = admin_authed(&state, &headers).await else {
        return unauthorized();
    };
    let appearance = mw_admin::Appearance {
        theme: body.theme,
        brand_name: body.brand_name,
        accent: body.accent,
    };
    match state.v6.admin.set_appearance(&actor, appearance).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(e),
    }
}

// ─── camelCase DTOs (match state/slices/admin.ts) ─────────────────────────────

/// The answer of `POST /admin/login` and `GET /admin/session`. `csrfToken` is the
/// value of the `mw_csrf` cookie, for a client that cannot read cookies.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminSessionDto {
    username: String,
    csrf_token: String,
}

/// A domain as the panel sees it: its name. The stored `upstream_json`,
/// `allowlist` and `blocklist` columns are not part of the wire shape — see the
/// module docs.
#[derive(Serialize)]
struct DomainDto {
    name: String,
}
impl From<AdminDomain> for DomainDto {
    fn from(d: AdminDomain) -> Self {
        Self { name: d.name }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct QuotaDto {
    bytes_limit: i64,
    msg_limit: i64,
}
impl From<AdminQuota> for QuotaDto {
    fn from(q: AdminQuota) -> Self {
        Self {
            bytes_limit: q.bytes_limit,
            msg_limit: q.msg_limit,
        }
    }
}
impl From<QuotaDto> for AdminQuota {
    fn from(q: QuotaDto) -> Self {
        Self {
            bytes_limit: q.bytes_limit,
            msg_limit: q.msg_limit,
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct FlagsDto {
    zero_access: bool,
    force_password_change: bool,
    remote_cache_wipe: bool,
    disabled: bool,
}
impl From<mw_admin::UserFeatureFlags> for FlagsDto {
    fn from(f: mw_admin::UserFeatureFlags) -> Self {
        Self {
            zero_access: f.zero_access,
            force_password_change: f.force_password_change,
            remote_cache_wipe: f.remote_cache_wipe,
            disabled: f.disabled,
        }
    }
}
impl From<FlagsDto> for mw_admin::UserFeatureFlags {
    fn from(f: FlagsDto) -> Self {
        Self {
            zero_access: f.zero_access,
            force_password_change: f.force_password_change,
            remote_cache_wipe: f.remote_cache_wipe,
            disabled: f.disabled,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserSummaryDto {
    account_id: String,
    username: String,
    domain: String,
    quota: Option<QuotaDto>,
    flags: FlagsDto,
}

#[derive(Deserialize)]
struct ProvisionReq {
    domain: String,
    username: String,
    quota: QuotaDto,
}

/// The body of `PUT /admin/security-policy`: every field, none optional, nothing
/// else. The enums refuse a value outside their set at deserialisation.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SecurityPolicyPut {
    min_tls: MinTls,
    require2fa: bool,
    argon2_m_cost: u32,
    argon2_t_cost: u32,
    argon2_p_cost: u32,
    capture_policy: CapturePolicy,
    max_security_floor: Option<SecurityFloor>,
    dlp_rules_json: String,
}

fn audit_dto(e: mw_admin::AuditLogEntry) -> Value {
    let kind = match e.actor_kind {
        ActorKind::Admin => "admin",
        ActorKind::User => "user",
        ActorKind::ApiKey => "api-key",
        ActorKind::System => "system",
    };
    json!({
        "id": e.id,
        "ts": e.ts,
        "actor": e.actor,
        "actorKind": kind,
        "action": e.action,
        "target": e.target,
        "detailJson": e.detail_json,
        "ip": e.ip,
    })
}

#[cfg(test)]
mod tests {
    //! The B3 wiring, over a real spawned server.
    //!
    //! What the login monitor is keyed on is only observable end-to-end: the
    //! domain logic in `mw-admin` has always been correct in isolation, and its
    //! unit tests passed throughout the period in which per-IP banning did not
    //! work at all. What was wrong was the value *this* module handed it. So these
    //! drive the real `/admin/login` route and read the real ban list back.
    //!
    //! **The harness installs `into_make_service_with_connect_info`.** Without it
    //! every request arrives with no peer address, the unattributed branch runs,
    //! and a test asserting "a ban was recorded" would fail — or worse, one
    //! asserting a negative would pass for entirely the wrong reason. One case
    //! below deliberately spawns *without* it to pin the `None` behaviour; it uses
    //! a separate spawner so the two can never be confused.

    use std::net::SocketAddr;
    use std::path::PathBuf;

    use serde_json::json;

    use crate::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

    const SERVER_KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
    const ADMIN_USER: &str = "root";
    const ADMIN_PASS: &str = "hunter2";

    fn unique() -> String {
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        format!(
            "{}_{}_{}",
            std::process::id(),
            SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )
    }

    fn web_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mw-t20e6-web-{}", unique()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("index.html"),
            "<!doctype html><title>MW</title><div id=app>MW</div>",
        )
        .unwrap();
        dir
    }

    async fn build() -> axum::Router {
        let db = std::env::temp_dir().join(format!("mw-t20e6-{}.db", unique()));
        let config = AppConfig {
            db_path: db.to_string_lossy().into_owned(),
            server_key_hex: Some(SERVER_KEY_HEX.into()),
            web_dir: Some(web_dir()),
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
        };
        let v6 = V6Config {
            admin_enabled: true,
            admin_username: Some(ADMIN_USER.into()),
            admin_password: Some(ADMIN_PASS.into()),
            redis_url: None,
        };
        build_app_full(config, v6).await.expect("server boots").0
    }

    /// A server whose requests carry a peer address — what `main.rs` serves in
    /// production. Anything asserting on monitor behaviour must use this.
    async fn spawn_with_peer() -> String {
        let app = build().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
            .unwrap();
        });
        format!("http://{addr}")
    }

    /// A server with **no** `ConnectInfo`, for the unattributed case only.
    async fn spawn_without_peer() -> String {
        let app = build().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("http://{addr}")
    }

    async fn bad_login(c: &reqwest::Client, base: &str, extra: Option<(&str, &str)>) {
        let mut req = c
            .post(format!("{base}/admin/login"))
            .json(&json!({ "username": ADMIN_USER, "password": "wrong" }));
        if let Some((k, v)) = extra {
            req = req.header(k, v);
        }
        let resp = req.send().await.unwrap();
        assert_eq!(resp.status(), 401, "a wrong password is rejected");
    }

    /// Log in for real and return the `Cookie` header value.
    async fn good_login(c: &reqwest::Client, base: &str) -> String {
        let resp = c
            .post(format!("{base}/admin/login"))
            .json(&json!({ "username": ADMIN_USER, "password": ADMIN_PASS }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "the right password still works");
        let set_cookie = resp
            .headers()
            .get(reqwest::header::SET_COOKIE)
            .expect("login sets a cookie")
            .to_str()
            .unwrap();
        set_cookie.split(';').next().unwrap().to_string()
    }

    async fn bans(c: &reqwest::Client, base: &str, cookie: &str) -> Vec<serde_json::Value> {
        let resp = c
            .get(format!("{base}/admin/bans"))
            .header(reqwest::header::COOKIE, cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        resp.json().await.unwrap()
    }

    async fn audit(c: &reqwest::Client, base: &str, cookie: &str) -> Vec<serde_json::Value> {
        let resp = c
            .get(format!("{base}/admin/audit?limit=50"))
            .header(reqwest::header::COOKIE, cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        resp.json().await.unwrap()
    }

    /// The headline: the ban is recorded against the address the request actually
    /// came from. Before this lane the ban list held one row reading `admin-panel`
    /// — a subject that is not an address, matches no source, and could never be
    /// unbanned through the by-IP route.
    #[tokio::test]
    async fn failed_admin_logins_ban_the_real_client_address() {
        let base = spawn_with_peer().await;
        let c = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("reqwest client builds");
        for _ in 0..5 {
            bad_login(&c, &base, None).await;
        }
        let cookie = good_login(&c, &base).await;
        let list = bans(&c, &base, &cookie).await;

        assert_eq!(list.len(), 1, "one source, one ban: {list:?}");
        let ip = list[0]["ip"].as_str().unwrap();
        assert_eq!(
            ip, "127.0.0.1",
            "the ban must name the peer address, got {ip:?}"
        );
        // It parses as an address — the property both fail2ban and the
        // unban-by-IP route depend on, and the one `admin-panel` never had.
        assert!(ip.parse::<std::net::IpAddr>().is_ok());
        assert!(
            list[0]["reason"].as_str().unwrap().contains("brute-force"),
            "{list:?}"
        );
    }

    /// The default posture ignores forwarded headers, so a client cannot choose
    /// the ban key. If it could, this fix would be a downgrade rather than a fix:
    /// an attacker could name a colleague's address, or a shared egress IP, and
    /// have it banned on demand.
    #[tokio::test]
    async fn a_forged_forwarded_header_does_not_become_the_ban_key() {
        let base = spawn_with_peer().await;
        let c = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("reqwest client builds");
        for _ in 0..5 {
            bad_login(&c, &base, Some(("x-forwarded-for", "203.0.113.99"))).await;
        }
        let cookie = good_login(&c, &base).await;
        let list = bans(&c, &base, &cookie).await;

        assert_eq!(list.len(), 1, "{list:?}");
        assert_eq!(
            list[0]["ip"].as_str().unwrap(),
            "127.0.0.1",
            "the peer address, not the header's claim: {list:?}"
        );
        assert!(
            !serde_json::to_string(&list)
                .unwrap()
                .contains("203.0.113.99"),
            "the forged address must appear nowhere: {list:?}"
        );
    }

    /// A successful login clears that source's counter, so an operator who
    /// mistypes four times and then gets it right is not left one typo from a ban
    /// for the rest of the window.
    #[tokio::test]
    async fn a_successful_login_clears_the_counter_for_that_source() {
        let base = spawn_with_peer().await;
        let c = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("reqwest client builds");
        for _ in 0..4 {
            bad_login(&c, &base, None).await;
        }
        let cookie = good_login(&c, &base).await;
        assert!(bans(&c, &base, &cookie).await.is_empty(), "four is under 5");

        // Four more failures after the success must not trip the threshold either.
        for _ in 0..4 {
            bad_login(&c, &base, None).await;
        }
        let list = bans(&c, &base, &cookie).await;
        assert!(list.is_empty(), "the success reset the window: {list:?}");
    }

    /// No `ConnectInfo` ⇒ no source ⇒ nothing counted and nothing banned. The
    /// decision under test is that this does **not** fall back to a shared bucket:
    /// a bucket every unattributed request shares is precisely the defect B3
    /// named, and it would let anyone reaching this path write a ban row naming no
    /// real host. Authentication itself is unaffected — it does not depend on the
    /// monitor.
    #[tokio::test]
    async fn without_a_peer_address_nothing_is_counted_or_banned() {
        let base = spawn_without_peer().await;
        let c = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("reqwest client builds");
        for _ in 0..12 {
            bad_login(&c, &base, None).await;
        }
        let cookie = good_login(&c, &base).await;
        let list = bans(&c, &base, &cookie).await;
        assert!(
            list.is_empty(),
            "no address ⇒ no ban row, not a placeholder one: {list:?}"
        );
    }

    /// The failure is still on the audit record even when it could not be counted
    /// — dropping the security event would be a worse trade than dropping the
    /// count.
    #[tokio::test]
    async fn unattributable_failures_are_still_audited() {
        let base = spawn_without_peer().await;
        let c = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("reqwest client builds");
        bad_login(&c, &base, None).await;
        let cookie = good_login(&c, &base).await;

        let entries = audit(&c, &base, &cookie).await;
        let failed: Vec<_> = entries
            .iter()
            .filter(|e| e["action"] == "login-failed")
            .collect();
        assert_eq!(failed.len(), 1, "{entries:?}");
        assert!(failed[0]["ip"].is_null(), "honest empty source: {failed:?}");
    }

    /// With a peer address the audit record carries it — what an incident review
    /// reads to answer "where from", and unconditionally absent before this lane.
    #[tokio::test]
    async fn audited_logins_carry_the_source_address() {
        let base = spawn_with_peer().await;
        let c = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("reqwest client builds");
        bad_login(&c, &base, None).await;
        let cookie = good_login(&c, &base).await;

        let entries = audit(&c, &base, &cookie).await;
        for action in ["login-failed", "login-succeeded"] {
            let e = entries
                .iter()
                .find(|e| e["action"] == action)
                .unwrap_or_else(|| panic!("{action} audited: {entries:?}"));
            assert_eq!(e["ip"].as_str(), Some("127.0.0.1"), "{action}: {e:?}");
        }
    }
}
