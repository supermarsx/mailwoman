//! Admin-panel HTTP surface (SPEC §19, plan §2.5, §3 e11 MOUNT).
//!
//! A SEPARATE session domain: `/admin/*` runs under its own `mw_admin_session`
//! cookie (Path=/admin), distinct from the mailbox `mw_session`. Login validates
//! the operator credentials (`MW_ADMIN_USER`/`MW_ADMIN_PASSWORD`) and mints an
//! admin session stored (hashed) in `admin_sessions`. Every action drives
//! [`mw_admin::Admin`] (backed by the 0007 tables via
//! [`crate::stores_v6::AdminBackendAdapter`]), which writes the append-only audit
//! log. `admin.enabled = false` makes every route return `401` (the panel is
//! unreachable) — the CLI + GitOps config keep working.
//!
//! JSON is camelCase to satisfy the typed web client in `state/slices/admin.ts`.

use axum::Json;
use axum::Router;
use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use mw_admin::{ActorKind, Domain as AdminDomain, Quota as AdminQuota};

use crate::AppState;

const ADMIN_COOKIE: &str = "mw_admin_session";

/// The `/admin/*` router (merged by e11's `router()`).
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin/login", post(login))
        .route("/admin/logout", post(logout))
        .route("/admin/session", get(session))
        .route("/admin/domains", get(list_domains))
        .route(
            "/admin/domains/{name}",
            put(save_domain).delete(delete_domain),
        )
        .route("/admin/users", get(list_users).post(provision_user))
        .route("/admin/users/{account_id}/quota", put(set_quota))
        .route("/admin/users/{account_id}/flags", put(set_flags))
        .route(
            "/admin/users/{account_id}/zero-access",
            post(toggle_zero_access),
        )
        .route(
            "/admin/users/{account_id}/revoke-sessions",
            post(revoke_sessions),
        )
        .route("/admin/security-policy", get(get_policy).put(set_policy))
        .route("/admin/integrations", get(get_integrations))
        .route("/admin/webhooks", get(list_webhooks))
        .route("/admin/api-keys", get(list_api_keys))
        .route("/admin/api-keys/{id}/revoke", post(revoke_api_key))
        .route("/admin/observability", get(get_obs).put(set_obs))
        .route("/admin/audit", get(list_audit))
        .route("/admin/audit/export", get(export_audit))
        .route("/admin/bans", get(list_bans).post(add_ban))
        .route("/admin/bans/{ip}", delete(remove_ban))
        .route("/admin/appearance", get(get_appearance).put(set_appearance))
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
    for part in raw.split(';') {
        if let Some(v) = part.trim().strip_prefix(&format!("{ADMIN_COOKIE}="))
            && !v.is_empty()
        {
            return Some(v.to_string());
        }
    }
    None
}

fn set_admin_cookie(token: &str, secure: bool) -> String {
    let mut c = format!("{ADMIN_COOKIE}={token}; HttpOnly; SameSite=Strict; Path=/admin");
    if secure {
        c.push_str("; Secure");
    }
    c
}

fn clear_admin_cookie(secure: bool) -> String {
    let mut c = format!("{ADMIN_COOKIE}=; HttpOnly; SameSite=Strict; Path=/admin; Max-Age=0");
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
/// `admin.enabled` gate (disabled → every route is `401`).
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

// ─── session / login / logout ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct LoginReq {
    username: String,
    password: String,
}

async fn login(State(state): State<AppState>, Json(body): Json<LoginReq>) -> Response {
    if !state.v6.admin_enabled {
        return unauthorized();
    }
    let (Some(user), Some(pass)) = (&state.v6.admin_username, &state.v6.admin_password) else {
        return unauthorized();
    };
    if !ct_eq(body.username.as_bytes(), user.as_bytes())
        || !ct_eq(body.password.as_bytes(), pass.as_bytes())
    {
        let _ = state
            .v6
            .admin
            .record_login_failure(&body.username, "admin-panel")
            .await;
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
    let _ = state
        .v6
        .admin
        .record_login_success(&body.username, "admin-panel")
        .await;
    let mut resp = Json(AdminSessionDto {
        username: body.username,
    })
    .into_response();
    resp.headers_mut().append(
        header::SET_COOKIE,
        set_admin_cookie(&token, state.cookie_secure)
            .parse()
            .unwrap(),
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

async fn session(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match admin_authed(&state, &headers).await {
        Ok(username) => Json(AdminSessionDto { username }).into_response(),
        Err(resp) => resp,
    }
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

async fn save_domain(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(_name): UrlPath<String>,
    Json(body): Json<DomainDto>,
) -> Response {
    let Ok(actor) = admin_authed(&state, &headers).await else {
        return unauthorized();
    };
    match state.v6.admin.create_domain(&actor, body.into()).await {
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
    match state.v6.admin.get_security_policy().await {
        Ok(p) => Json(SecurityPolicyDto::from(p)).into_response(),
        Err(e) => err500(e),
    }
}

async fn set_policy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SecurityPolicyDto>,
) -> Response {
    let Ok(actor) = admin_authed(&state, &headers).await else {
        return unauthorized();
    };
    match state
        .v6
        .admin
        .set_security_policy(&actor, body.into())
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(e),
    }
}

// ─── integrations / oversight ─────────────────────────────────────────────────

async fn get_integrations(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(r) = admin_authed(&state, &headers).await {
        return r;
    }
    let i = state.v6.admin.integrations();
    let s = |v: mw_admin::IntegrationStatus| match v {
        mw_admin::IntegrationStatus::Active => "active",
        mw_admin::IntegrationStatus::Deferred => "deferred",
    };
    Json(json!({
        "webhooks": s(i.webhooks),
        "apiKeyOversight": s(i.api_key_oversight),
        "ldap": s(i.ldap),
        "nextcloud": s(i.nextcloud),
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

#[derive(Serialize)]
struct AdminSessionDto {
    username: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DomainDto {
    name: String,
    upstream_json: String,
    allowlist: Vec<String>,
    blocklist: Vec<String>,
}
impl From<AdminDomain> for DomainDto {
    fn from(d: AdminDomain) -> Self {
        Self {
            name: d.name,
            upstream_json: d.upstream_json,
            allowlist: d.allowlist,
            blocklist: d.blocklist,
        }
    }
}
impl From<DomainDto> for AdminDomain {
    fn from(d: DomainDto) -> Self {
        Self {
            name: d.name,
            upstream_json: d.upstream_json,
            allowlist: d.allowlist,
            blocklist: d.blocklist,
        }
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SecurityPolicyDto {
    min_tls: String,
    require2fa: bool,
    argon2_m_cost: u32,
    argon2_t_cost: u32,
    argon2_p_cost: u32,
    dlp_rules_json: String,
    max_security_floor: bool,
    capture_policy: String,
}
impl From<mw_admin::SecurityPolicy> for SecurityPolicyDto {
    fn from(p: mw_admin::SecurityPolicy) -> Self {
        Self {
            min_tls: p.min_tls,
            require2fa: p.require_2fa,
            argon2_m_cost: p.argon2_m_cost,
            argon2_t_cost: p.argon2_t_cost,
            argon2_p_cost: p.argon2_p_cost,
            dlp_rules_json: p.dlp_rules_json,
            max_security_floor: p.max_security_floor,
            capture_policy: p.capture_policy,
        }
    }
}
impl From<SecurityPolicyDto> for mw_admin::SecurityPolicy {
    fn from(p: SecurityPolicyDto) -> Self {
        Self {
            min_tls: p.min_tls,
            require_2fa: p.require2fa,
            argon2_m_cost: p.argon2_m_cost,
            argon2_t_cost: p.argon2_t_cost,
            argon2_p_cost: p.argon2_p_cost,
            dlp_rules_json: p.dlp_rules_json,
            max_security_floor: p.max_security_floor,
            capture_policy: p.capture_policy,
        }
    }
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
