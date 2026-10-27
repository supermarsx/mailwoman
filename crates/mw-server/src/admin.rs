//! Admin-panel HTTP surface (SPEC §19, plan §2.5, §3 e11 MOUNT).
//!
//! A SEPARATE session domain: `/admin/*` runs under its own cookie, named
//! `mw_admin_session` unless `MW_ADMIN_SESSION_COOKIE` says otherwise
//! ([`mw_admin::config::session_cookie_name`]), distinct from the mailbox
//! `mw_session`. Sign-in and the admin's own session live in
//! [`crate::admin_accounts`]; every route here is registered in
//! [`crate::admin_router`] with the capability it needs, and a domain-scoped admin
//! acts only on users and domains in their domains. Every action drives
//! [`mw_admin::Admin`] (backed by the 0007 tables via
//! [`crate::stores_v6::AdminBackendAdapter`]), which writes the append-only audit
//! log. `MW_ADMIN_ENABLED=false` makes every route return `401` (the panel is
//! unreachable) — the CLI keeps working.
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
//! * Domain configuration lives in `domain_routes`.
//! * `/admin/observability` still answers, for a later release that gives it a
//!   reader; the panel does not show it.
//!
//! **Appearance (26.20, t32 settings model C).** `/admin/appearance` reads and
//! writes the global values of the settings keys `appearance.theme`,
//! `appearance.brandName` and `appearance.accent`, stored in the database and
//! validated by the settings registry; the same values are under
//! `/admin/settings`.
//!
//! JSON is camelCase to satisfy the typed web client in `state/slices/admin.ts`.

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use mw_admin::{
    ActorKind, AdminError, CapturePolicy, EffectiveSecurityPolicy, MinTls, Quota as AdminQuota,
    SecurityFloor, SecurityPolicy, SecurityPolicyWrite,
};

use crate::AppState;
use crate::admin_ctx::AdminCtx;
use crate::admin_router::AdminRegistrar;
use crate::capabilities::Capability;

/// The `/admin/*` routes of this module (registered by `admin_router`). Sign-in,
/// sign-out and the session read live in `admin_accounts`.
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    use Capability as C;
    r.admin_endpoint(Method::GET, "/users", C::UsersRead, list_users);
    r.admin_endpoint(Method::POST, "/users", C::UsersCreate, provision_user);
    r.admin_endpoint(
        Method::PUT,
        "/users/{account_id}/quota",
        C::UsersQuota,
        set_quota,
    );
    r.admin_endpoint(
        Method::PUT,
        "/users/{account_id}/flags",
        C::UsersFlags,
        set_flags,
    );
    r.admin_endpoint(
        Method::POST,
        "/users/{account_id}/zero-access",
        C::UsersFlags,
        toggle_zero_access,
    );
    r.admin_endpoint(
        Method::POST,
        "/users/{account_id}/revoke-sessions",
        C::UsersRevokeSessions,
        revoke_sessions,
    );
    r.admin_endpoint(Method::GET, "/security-policy", C::SecurityRead, get_policy);
    r.admin_endpoint(
        Method::PUT,
        "/security-policy",
        C::SecurityPolicyWrite,
        set_policy,
    );
    r.admin_endpoint(
        Method::GET,
        "/integrations",
        C::IntegrationsRead,
        get_integrations,
    );
    r.admin_endpoint(Method::GET, "/api-keys", C::IntegrationsRead, list_api_keys);
    r.admin_endpoint(
        Method::POST,
        "/api-keys/{id}/revoke",
        C::ApiKeysRevoke,
        revoke_api_key,
    );
    r.admin_endpoint(Method::GET, "/observability", C::ObservabilityRead, get_obs);
    r.admin_endpoint(Method::PUT, "/observability", C::LogsLevel, set_obs);
    r.admin_endpoint(
        Method::GET,
        "/users/{id}/identity",
        C::UsersRead,
        get_user_identity,
    );
    r.admin_endpoint(Method::GET, "/audit", C::AuditRead, list_audit);
    r.admin_endpoint(Method::GET, "/audit/export", C::AuditExport, export_audit);
    r.admin_endpoint(Method::GET, "/bans", C::BansRead, list_bans);
    r.admin_endpoint(Method::POST, "/bans", C::BansWrite, add_ban);
    r.admin_endpoint(Method::DELETE, "/bans/{ip}", C::BansWrite, remove_ban);
    r.admin_endpoint(Method::GET, "/appearance", C::BrandingRead, get_appearance);
    r.admin_endpoint(
        Method::PUT,
        "/appearance",
        C::AppearanceManage,
        set_appearance,
    );
}

// ─── helpers ──────────────────────────────────────────────────────────────────

fn err500(e: impl std::fmt::Display) -> Response {
    tracing::warn!("admin error: {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": "admin operation failed" })),
    )
        .into_response()
}

// ─── users ────────────────────────────────────────────────────────────────────

/// Presentation-only recovery of a verified native alias. The directory address
/// remains the authority for policy checks and every stored association.
async fn directory_login_address(
    state: &AppState,
    account: &str,
    address: &str,
) -> Result<String, mw_store::StoreError> {
    let Some(native) = state
        .store
        .verified_native_directory_login(account, address)
        .await?
    else {
        return Ok(address.into());
    };
    let normalized = native.trim().to_lowercase();
    let Some((local, domain)) = normalized.rsplit_once('@') else {
        return Ok(address.into());
    };
    let domain =
        crate::domain_config::normalize_domain(domain).map_err(mw_store::StoreError::Corrupt)?;
    let login = format!("{local}@{domain}");
    if login != address && state.store.canonical_policy_address(&login).await? == address {
        Ok(login)
    } else {
        Ok(address.into())
    }
}

async fn list_users(State(state): State<AppState>, ctx: AdminCtx) -> Response {
    let users = match state.store.directory_accounts().await {
        Ok(u) => u,
        Err(e) => return err500(e),
    };
    let scope = ctx.domain_filter(Capability::UsersRead);
    let mut out = Vec::new();
    for u in users {
        let account_id = u.account_id;
        let policy_address = match state.store.canonical_policy_address(&u.address).await {
            Ok(value) => value,
            Err(e) => return err500(e),
        };
        if let Some(scope) = scope
            && !scope.contains(&crate::admin_ctx::domain_of(&policy_address))
        {
            continue;
        }
        let directory_address = u.address;
        let login_address =
            match directory_login_address(&state, &account_id, &directory_address).await {
                Ok(address) => address,
                Err(error) => return err500(error),
            };
        let legacy_binding = login_address != directory_address;
        let (username, domain) = match login_address.rsplit_once('@') {
            Some((a, b)) => (a.to_string(), b.to_string()),
            None => (login_address.clone(), String::new()),
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
            login_address,
            directory_address,
            legacy_binding,
            account_id,
            username,
            domain,
            quota,
            flags,
            last_signin_at: u.last_signin_at,
            status: u.status,
            deleted_at: u.deleted_at,
        });
    }
    Json(out).into_response()
}

async fn user_identity(
    state: &AppState,
    ctx: &AdminCtx,
    id: &str,
) -> Result<mw_store::audit_identity::AuditIdentity, crate::admin_ctx::AdminApiError> {
    let identity = state
        .store
        .audit_identity("user", id)
        .await?
        .ok_or_else(|| {
            crate::admin_ctx::AdminApiError::new(
                StatusCode::NOT_FOUND,
                "userNotFound",
                "User not found.",
            )
        })?;
    let address = state
        .store
        .canonical_policy_address(identity.address.as_deref().unwrap_or_default())
        .await?;
    ctx.require_domain(&crate::admin_ctx::domain_of(&address))?;
    Ok(identity)
}
async fn get_user_identity(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(id): UrlPath<String>,
) -> Response {
    match user_identity(&state, &ctx, &id).await {
        Ok(identity) => {
            let directory_address = identity.address.as_deref().unwrap_or_default();
            let login_address =
                match directory_login_address(&state, &identity.id, directory_address).await {
                    Ok(address) => address,
                    Err(error) => return err500(error),
                };
            (
                [(header::CACHE_CONTROL, "no-store")],
                Json(json!({
                    "accountId":identity.id,"label":identity.label,"address":identity.address,
                    "loginAddress":login_address,"directoryAddress":directory_address,
                    "legacyBinding":login_address != directory_address,
                })),
            )
                .into_response()
        }
        Err(error) => error.into_response(),
    }
}

async fn provision_user(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Json(body): Json<ProvisionReq>,
) -> Response {
    let actor = ctx.username.clone();
    if let Err(e) = ctx.require_domain(&body.domain) {
        return e.into_response();
    }
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
    ctx: AdminCtx,
    UrlPath(account_id): UrlPath<String>,
    Json(body): Json<QuotaDto>,
) -> Response {
    if let Err(error) = user_identity(&state, &ctx, &account_id).await {
        return error.into_response();
    }
    let actor = ctx.username.clone();
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
    ctx: AdminCtx,
    UrlPath(account_id): UrlPath<String>,
    Json(body): Json<FlagsDto>,
) -> Response {
    if let Err(error) = user_identity(&state, &ctx, &account_id).await {
        return error.into_response();
    }
    let actor = ctx.username.clone();
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
    ctx: AdminCtx,
    UrlPath(account_id): UrlPath<String>,
    Json(body): Json<ToggleReq>,
) -> Response {
    if let Err(error) = user_identity(&state, &ctx, &account_id).await {
        return error.into_response();
    }
    let actor = ctx.username.clone();
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
    ctx: AdminCtx,
    UrlPath(account_id): UrlPath<String>,
) -> Response {
    if let Err(error) = user_identity(&state, &ctx, &account_id).await {
        return error.into_response();
    }
    let actor = ctx.username.clone();
    match state.v6.admin.revoke_sessions(&actor, &account_id).await {
        Ok(count) => Json(json!({ "count": count })).into_response(),
        Err(e) => err500(e),
    }
}

// ─── security policy ──────────────────────────────────────────────────────────

async fn get_policy(State(state): State<AppState>, _ctx: AdminCtx) -> Response {
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
    ctx: AdminCtx,
    body: Result<Json<SecurityPolicyPut>, JsonRejection>,
) -> Response {
    let actor = ctx.username.clone();
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
async fn get_integrations(State(state): State<AppState>, _ctx: AdminCtx) -> Response {
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

/// `GET /admin/api-keys` — every key row, live and revoked; never a key hash.
///
/// `unattendedSendRequested` is the owner's request in the key's scope;
/// `unattendedSendApproved` is the `api_keys.unattended_send` column, which only
/// `PUT /admin/api-keys/{id}/unattended-send` (`oauth.rs`) writes.
async fn list_api_keys(State(state): State<AppState>, _ctx: AdminCtx) -> Response {
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
    ctx: AdminCtx,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let actor = ctx.username.clone();
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
async fn get_obs(State(state): State<AppState>, _ctx: AdminCtx) -> Response {
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
    ctx: AdminCtx,
    Json(body): Json<ObsDto>,
) -> Response {
    let actor = ctx.username.clone();
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

async fn audit_rows(
    state: &AppState,
    ctx: &AdminCtx,
    cap: Capability,
    limit: usize,
    export: bool,
) -> Result<Vec<Value>, crate::admin_ctx::AdminApiError> {
    let work = async {
        let rows = state
            .v6
            .admin
            .list_audit(limit)
            .await
            .map_err(crate::admin_ctx::AdminApiError::internal)?;
        let mut projection = crate::audit_projection::Projection::new(&state.store, ctx, cap);
        let mut values = Vec::new();
        for row in rows {
            let value = if export {
                serde_json::to_value(&row).map_err(crate::admin_ctx::AdminApiError::internal)?
            } else {
                audit_dto(row.clone())
            };
            if let Some(value) = projection.decorate(&row, value).await? {
                values.push(value);
            }
        }
        Ok(values)
    };
    tokio::time::timeout(std::time::Duration::from_secs(3), work)
        .await
        .map_err(|_| {
            crate::admin_ctx::AdminApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "auditTimeout",
                "Audit query budget exceeded; reduce the limit.",
            )
        })?
}

async fn list_audit(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(q): Query<LimitQuery>,
) -> Response {
    match audit_rows(
        &state,
        &ctx,
        Capability::AuditRead,
        q.limit.unwrap_or(50).min(1000),
        false,
    )
    .await
    {
        Ok(rows) => ([(header::CACHE_CONTROL, "no-store")], Json(rows)).into_response(),
        Err(error) => error.into_response(),
    }
}

async fn export_audit(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(q): Query<LimitQuery>,
) -> Response {
    match audit_rows(
        &state,
        &ctx,
        Capability::AuditExport,
        q.limit.unwrap_or(1000).min(10000),
        true,
    )
    .await
    {
        Ok(rows) => {
            let text = rows
                .into_iter()
                .map(|row| row.to_string() + "\n")
                .collect::<String>();
            (
                [
                    (header::CONTENT_TYPE, "application/x-ndjson"),
                    (header::CACHE_CONTROL, "no-store"),
                ],
                text,
            )
                .into_response()
        }
        Err(error) => error.into_response(),
    }
}

async fn list_bans(State(state): State<AppState>, ctx: AdminCtx) -> Response {
    if let Err(error) = ctx.require_unscoped(Capability::BansRead) {
        return error.into_response();
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
    ctx: AdminCtx,
    Json(body): Json<BanReq>,
) -> Response {
    if let Err(error) = ctx.require_unscoped(Capability::BansWrite) {
        return error.into_response();
    }
    if let Err(error) =
        crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::BanManage).await
    {
        return error.into_response();
    }
    let actor = ctx.username.clone();
    match state
        .v6
        .admin
        .ban_ip(&actor, &body.ip, &body.reason, body.expires_at)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(AdminError::Invalid(reason)) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"code":"invalidBan","error":reason})),
        )
            .into_response(),
        Err(e) => err500(e),
    }
}

async fn remove_ban(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(ip): UrlPath<String>,
) -> Response {
    if let Err(error) = ctx.require_unscoped(Capability::BansWrite) {
        return error.into_response();
    }
    if let Err(error) =
        crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::BanManage).await
    {
        return error.into_response();
    }
    let actor = ctx.username.clone();
    match state.v6.admin.unban_ip(&actor, &ip).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(AdminError::Invalid(reason)) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"code":"invalidBan","error":reason})),
        )
            .into_response(),
        Err(e) => err500(e),
    }
}

// ─── appearance ───────────────────────────────────────────────────────────────

/// `GET /admin/appearance` — the deployment-default appearance: the stored global
/// values of `appearance.theme`, `appearance.brandName` and `appearance.accent`,
/// or the built-in defaults.
async fn get_appearance(State(state): State<AppState>, _ctx: AdminCtx) -> Response {
    match state.v6.admin.appearance().await {
        Ok(a) => Json(json!({ "theme": a.theme, "brandName": a.brand_name, "accent": a.accent }))
            .into_response(),
        Err(e) => err500(e),
    }
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
    ctx: AdminCtx,
    Json(body): Json<AppearanceDto>,
) -> Response {
    let actor = ctx.username.clone();
    let appearance = mw_admin::Appearance {
        theme: body.theme,
        brand_name: body.brand_name,
        accent: body.accent.filter(|a| !a.is_empty()),
    };
    match state.v6.admin.set_appearance(&actor, appearance).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(AdminError::Invalid(why)) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "code": "invalidValue", "detail": why, "error": why })),
        )
            .into_response(),
        Err(e) => err500(e),
    }
}

// ─── camelCase DTOs (match state/slices/admin.ts) ─────────────────────────────

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
    status: String,
    deleted_at: Option<i64>,
    login_address: String,
    directory_address: String,
    legacy_binding: bool,
    last_signin_at: Option<i64>,
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
        let app = build_app_full(config, v6).await.expect("server boots").0;
        // These cases isolate the failed-credential monitor; initial-attempt
        // pacing has its own real HTTP suite.
        let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", db.display()))
            .await
            .unwrap();
        sqlx::query("INSERT INTO setting_policy(key,global_value,updated_by,updated_at) VALUES('signin.retryCooldownSecs','0','test',0)").execute(&pool).await.unwrap();
        app
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

    async fn correct_password_is_blocked(c: &reqwest::Client, base: &str) {
        let response = c
            .post(format!("{base}/admin/login"))
            .json(&json!({"username": ADMIN_USER, "password": ADMIN_PASS}))
            .send()
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            429,
            "an active credential lock also blocks a correct password"
        );
    }

    async fn automatic_locks(
        _c: &reqwest::Client,
        base: &str,
        cookie: &str,
    ) -> Vec<serde_json::Value> {
        // An already authenticated observer can review the temporary sign-in lock.
        // Manual global source bans have separate middleware acceptance tests.
        let observer = reqwest::Client::builder()
            .no_proxy()
            .local_address(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 2)))
            .build()
            .unwrap();
        let resp = observer
            .get(format!("{base}/admin/bans/automatic"))
            .header(reqwest::header::COOKIE, cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = resp.json().await.unwrap();
        body["locks"].as_array().unwrap().clone()
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
        let cookie = good_login(&c, &base).await;
        for _ in 0..5 {
            bad_login(&c, &base, None).await;
        }
        correct_password_is_blocked(&c, &base).await;
        let list = automatic_locks(&c, &base, &cookie).await;

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
            list[0]["lockedUntil"].as_i64().unwrap() > chrono::Utc::now().timestamp()
                && list[0]["failures"].as_u64() == Some(5),
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
        let cookie = good_login(&c, &base).await;
        for _ in 0..5 {
            bad_login(&c, &base, Some(("x-forwarded-for", "203.0.113.99"))).await;
        }
        correct_password_is_blocked(&c, &base).await;
        let list = automatic_locks(&c, &base, &cookie).await;

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
        assert!(
            automatic_locks(&c, &base, &cookie).await.is_empty(),
            "four is under 5"
        );

        // Four more failures after the success must not trip the threshold either.
        for _ in 0..4 {
            bad_login(&c, &base, None).await;
        }
        let list = automatic_locks(&c, &base, &cookie).await;
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
        let list = automatic_locks(&c, &base, &cookie).await;
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
