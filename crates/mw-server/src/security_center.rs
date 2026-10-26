//! Account security history, remembered devices, and persisted session policy.
use crate::AppState;
use axum::extract::{Query, State};
use axum::http::{Extensions, HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use mw_store::{RememberedDevice, SessionDevice, SigninEvent};
use rand::RngCore;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub(crate) const REMEMBER_COOKIE: &str = "mw_remembered_device";
#[derive(Debug, Clone)]
pub(crate) struct Policy {
    pub max_sessions: usize,
    pub idle_secs: u64,
    pub absolute_secs: u64,
    pub remember_days: u64,
    pub passkeys_enabled: bool,
    pub required: bool,
    pub pending_secs: u64,
    pub max_attempts: u32,
}
fn error(what: &str, e: impl std::fmt::Display) -> Response {
    tracing::error!("security centre {what}: {e}");
    (StatusCode::SERVICE_UNAVAILABLE,Json(json!({"code":"securityUnavailable","error":"Security information is temporarily unavailable."}))).into_response()
}
pub(crate) async fn policy(
    state: &AppState,
    account: &str,
    address: &str,
) -> Result<Policy, Response> {
    let domain = address.rsplit_once('@').map_or("", |(_, d)| d);
    let subject = mw_store::settings_policy::SettingSubject {
        domain,
        account_id: account,
        address,
    };
    async fn value(
        state: &AppState,
        key: &str,
        subject: &mw_store::settings_policy::SettingSubject<'_>,
        fallback: Value,
    ) -> Result<Value, Response> {
        crate::settings_registry::effective_for(&state.store, key, subject)
            .await
            .map(|v| v.map_or(fallback, |v| v.value))
            .map_err(|e| error("policy", e))
    }
    Ok(Policy {
        max_sessions: value(state, "session.maxConcurrent", &subject, json!(10))
            .await?
            .as_u64()
            .unwrap_or(10)
            .clamp(1, 100) as usize,
        idle_secs: value(state, "session.idleSecs", &subject, json!(1800))
            .await?
            .as_u64()
            .unwrap_or(1800)
            .min(state.hardening.idle_timeout.as_secs()),
        absolute_secs: value(
            state,
            "session.absoluteSecs",
            &subject,
            json!(crate::hardening::MAILBOX_ABSOLUTE_CAP_SECS),
        )
        .await?
        .as_u64()
        .unwrap_or(crate::hardening::MAILBOX_ABSOLUTE_CAP_SECS)
        .min(state.hardening.absolute_timeout.as_secs())
        .min(crate::hardening::MAILBOX_ABSOLUTE_CAP_SECS),
        remember_days: value(state, "twofa.rememberDays", &subject, json!(0))
            .await?
            .as_u64()
            .unwrap_or(0)
            .min(90),
        passkeys_enabled: value(state, "twofa.passkeysEnabled", &subject, json!(true))
            .await?
            .as_bool()
            .unwrap_or(false),
        required: value(state, "twofa.required", &subject, json!(false))
            .await?
            .as_bool()
            .unwrap_or(true),
        pending_secs: value(state, "twofa.pendingTtlSecs", &subject, json!(300))
            .await?
            .as_u64()
            .unwrap_or(300)
            .clamp(60, 900),
        max_attempts: value(state, "twofa.maxAttempts", &subject, json!(5))
            .await?
            .as_u64()
            .unwrap_or(5)
            .clamp(3, 10) as u32,
    })
}
fn bounded(value: &str, max: usize) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .take(max)
        .collect()
}
pub(crate) fn device(
    headers: &HeaderMap,
    ext: &Extensions,
    client_type: Option<&str>,
) -> SessionDevice {
    let read = |key: &str, max| {
        bounded(
            headers
                .get(key)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default(),
            max,
        )
    };
    SessionDevice {
        user_agent: read("user-agent", 512),
        ip: crate::scope_mw::proxy::client_ip(headers, ext)
            .map(|v| v.to_string())
            .unwrap_or_default(),
        client_type: bounded(client_type.unwrap_or("browser"), 32),
        device_label: String::new(),
        client_version: read("x-mailwoman-client", 128),
    }
}
pub(crate) async fn record_login(
    state: &AppState,
    id: &str,
    account: &str,
    address: &str,
    device: &SessionDevice,
    method: &str,
) -> Result<(), Response> {
    let canonical = state
        .store
        .canonical_policy_address(address)
        .await
        .map_err(|e| error("directory identity", e))?;
    // Alias resolution selects policy and transport, not the user's identity.
    let normalized = address.trim().to_lowercase();
    let identity = if let Some((local, domain)) = normalized.rsplit_once('@') {
        format!(
            "{local}@{}",
            crate::domain_config::normalize_domain(domain)
                .map_err(|e| error("directory identity", e))?
        )
    } else {
        normalized
    };
    let (local, domain) = identity.rsplit_once('@').unwrap_or((&identity, ""));
    let registered = state
        .store
        .register_successful_signin_with_policy(
            mw_store::user_directory::SuccessfulSignin {
                account_id: account,
                address: &identity,
                domain,
                local_part: local,
                mode: if method == "sso" {
                    "sso"
                } else if method == "header" {
                    "header"
                } else if state.engine.is_some() {
                    "engine"
                } else {
                    "proxy"
                },
                now: chrono::Utc::now().timestamp(),
            },
            &canonical,
        )
        .await
        .map_err(|e| error("directory registration", e))?;
    if !registered {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error":"account unavailable"})),
        )
            .into_response());
    }
    let policy = policy(state, account, address).await?;
    state
        .store
        .set_session_authentication(id, method)
        .await
        .map_err(|e| error("authentication method", e))?;
    let revoked = state
        .store
        .record_session_device(id, account, device, policy.max_sessions)
        .await
        .map_err(|e| error("session metadata", e))?;
    for old in &revoked {
        state.sessions.forget(old);
        state.step_up.forget_user(old);
    }
    if !revoked.is_empty() {
        crate::audit::security_event(
            state,
            crate::audit::SecurityEvent::user(account, "session-cap-revoked")
                .detail(json!({"count":revoked.len()})),
        )
        .await;
    }
    let country = device
        .ip
        .parse()
        .ok()
        .and_then(mw_engine::security::country_for_ip);
    let previous = state
        .store
        .signin_history(account, i64::MAX, Some("success"), 500)
        .await
        .map_err(|e| error("history", e))?;
    let new_device = !device.user_agent.is_empty()
        && !previous
            .iter()
            .any(|event| event.user_agent == device.user_agent);
    let new_country = country.as_ref().is_some_and(|country| {
        !previous
            .iter()
            .any(|event| event.geo_country.as_ref() == Some(country))
    });
    let event = SigninEvent {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: Some(account.into()),
        username_attempted: address.into(),
        ts: chrono::Utc::now().timestamp(),
        result: "success".into(),
        method: method.into(),
        ip: device.ip.clone(),
        user_agent: device.user_agent.clone(),
        geo_country: country,
        reason: String::new(),
    };
    crate::auth_logs::record(
        state,
        &event,
        mw_store::AuthSurface::Webmail,
        Some(if state.engine.is_some() {
            "engine"
        } else {
            "jmap"
        }),
    )
    .await;
    if new_device {
        crate::alerts::emit(
            state,
            account,
            crate::alerts::AlertKind::NewDevice,
            json!({}),
        )
        .await;
    }
    if new_country {
        crate::alerts::emit(
            state,
            account,
            crate::alerts::AlertKind::NewCountry,
            json!({}),
        )
        .await;
    }
    Ok(())
}
/// Persisted activity is separate from last_seen: polling cannot extend idle life.
/// Neither restarts nor token rotation reset the original sign-in clock.
pub(crate) async fn enforce_session(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<(), Response> {
    match state.store.linked_session_kind(&session.id).await.map_err(|e| error("session authority",e))? {
        Some(0) => {},
        Some(2) => crate::linked_accounts::enforce_context(state,session).await?,
        _ => return Err(crate::unauthorized()),
    }
    enforce_session_clock(state,session).await
}
pub(crate) async fn enforce_session_clock(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<(), Response> {
    let policy = policy(state, &session.account_id, &session.username).await?;
    let Some((created, activity)) = state
        .store
        .session_activity_clock(&session.id)
        .await
        .map_err(|e| error("session clock", e))?
    else {
        return Err(crate::unauthorized());
    };
    if session_deadline_passed(
        &created,
        &activity,
        chrono::Utc::now(),
        policy.idle_secs,
        policy.absolute_secs,
    ) {
        let _ = state.store.delete_session(&session.id).await;
        let _ = state
            .store
            .delete_native_session(&crate::push_relay::hash_token(&session.id))
            .await;
        state.sessions.forget(&session.id);
        state.step_up.forget_user(&session.id);
        return Err(crate::session_expired());
    }
    Ok(())
}
pub(crate) fn session_deadline_passed(
    created: &str,
    activity: &str,
    now: chrono::DateTime<chrono::Utc>,
    idle_secs: u64,
    absolute_secs: u64,
) -> bool {
    let elapsed = |value: &str| {
        chrono::DateTime::parse_from_rfc3339(value)
            .ok()
            .map(|time| {
                (now - time.with_timezone(&chrono::Utc))
                    .num_seconds()
                    .max(0) as u64
            })
    };
    elapsed(created)
        .is_none_or(|age| age >= absolute_secs.min(crate::hardening::MAILBOX_ABSOLUTE_CAP_SECS))
        || elapsed(activity).is_none_or(|age| age >= idle_secs)
}

/// Explicit foreground interaction only; normal authenticated requests update
/// request telemetry but never this idle clock. No account or timestamp input.
async fn renew_activity(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    match crate::linked_accounts::record_activity(&state, &session, &chrono::Utc::now().to_rfc3339())
        .await
    {
        Ok(true) => (
            [(header::CACHE_CONTROL, "private, no-store")],
            Json(json!({"ok":true})),
        )
            .into_response(),
        Ok(false) => crate::unauthorized(),
        Err(error) => self::error("session activity", error),
    }
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/account/session/activity", post(renew_activity))
        .route("/api/account/security/history", get(history))
        .route("/api/account/security/activity", get(activity))
        .route(
            "/api/account/security/remembered-devices",
            get(remembered).post(remember),
        )
        .route(
            "/api/account/security/remembered-devices/revoke",
            post(revoke_remembered),
        )
}
#[derive(Default, Deserialize)]
struct HistoryQuery {
    before: Option<i64>,
    result: Option<String>,
    limit: Option<u32>,
    export: Option<bool>,
}
async fn history(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HistoryQuery>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if query.export == Some(true)
        && let Err(r) =
            crate::step_up::require_user(&state, &session, crate::step_up::StepUpAction::DataExport)
                .await
    {
        return r;
    }
    if query
        .result
        .as_deref()
        .is_some_and(|r| !["success", "failure", "blocked"].contains(&r))
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    match state
        .store
        .signin_history(
            &session.account_id,
            query.before.unwrap_or(i64::MAX),
            query.result.as_deref(),
            query.limit.unwrap_or(100),
        )
        .await
    {
        Ok(events) => Json(json!({"events":events})).into_response(),
        Err(e) => error("history", e),
    }
}
async fn activity(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    match state.store.account_activity(&session.account_id,100).await {
        Ok(events)=>Json(json!({"events":events.into_iter().map(|e|json!({"id":e.id,"timestamp":e.ts,"actor":e.actor,"action":e.action,"target":e.target,"detail":serde_json::from_str::<Value>(&e.detail_json).unwrap_or(Value::Null)})).collect::<Vec<_>>()})).into_response(),Err(e)=>error("activity",e),
    }
}
async fn remembered(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    match state
        .store
        .remembered_devices(&session.account_id, chrono::Utc::now().timestamp())
        .await
    {
        Ok(devices) => Json(json!({"devices":devices})).into_response(),
        Err(e) => error("remembered devices", e),
    }
}
#[derive(Deserialize)]
struct RememberRequest {
    label: String,
}
async fn remember(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: Extensions,
    Json(body): Json<RememberRequest>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if let Err(r) = crate::step_up::require_user(
        &state,
        &session,
        crate::step_up::StepUpAction::TwoFactorDisable,
    )
    .await
    {
        return r;
    }
    let policy = match policy(&state, &session.account_id, &session.username).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    if policy.remember_days == 0 {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"code":"rememberedDevicesDisabled"})),
        )
            .into_response();
    }
    let label = bounded(body.label.trim(), 128);
    if label.is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let fingerprint = match crate::step_up::factor_fingerprint(&state, &session).await {
        Ok(v) => v,
        Err(e) => return error("factors", e),
    };
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let hash = remember_hash(&token, &fingerprint);
    let now = chrono::Utc::now().timestamp();
    let ttl = policy.remember_days * 86400;
    let row = RememberedDevice {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: session.account_id.clone(),
        label,
        created_at: now,
        expires_at: now + ttl as i64,
        last_used_at: now,
    };
    if let Err(e) = state.store.put_remembered_device(&row, &hash).await {
        return error("remember device", e);
    }
    let secure = state
        .external
        .cookie_secure(state.cookie_secure, &headers, &ext);
    let cookie = format!(
        "{REMEMBER_COOKIE}={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age={ttl}{}",
        if secure { "; Secure" } else { "" }
    );
    let mut response = Json(json!({"device":row})).into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        cookie.parse().expect("generated cookie"),
    );
    response
}
fn remember_hash(token: &str, fingerprint: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(fingerprint);
    hash.update(token.as_bytes());
    hash.finalize().iter().map(|b| format!("{b:02x}")).collect()
}
pub(crate) async fn remembered_login(
    state: &AppState,
    headers: &HeaderMap,
    session: &mw_store::Session,
) -> Result<bool, Response> {
    if policy(state, &session.account_id, &session.username)
        .await?
        .remember_days
        == 0
    {
        return Ok(false);
    }
    let Some(token) = crate::hardening::cookie(headers, REMEMBER_COOKIE) else {
        return Ok(false);
    };
    if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Ok(false);
    }
    let fingerprint = crate::step_up::factor_fingerprint(state, session)
        .await
        .map_err(|e| error("factors", e))?;
    state
        .store
        .use_remembered_device(
            &session.account_id,
            &remember_hash(&token, &fingerprint),
            chrono::Utc::now().timestamp(),
        )
        .await
        .map_err(|e| error("remembered device", e))
}
#[derive(Deserialize)]
struct RevokeRemembered {
    id: String,
}
async fn revoke_remembered(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RevokeRemembered>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    match state
        .store
        .revoke_remembered_device(&session.account_id, &body.id)
        .await
    {
        Ok(true) => Json(json!({"ok":true})).into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => error("revoke device", e),
    }
}

pub(crate) fn register_admin(r: &mut impl crate::admin_router::AdminRegistrar) {
    use crate::capabilities::Capability;
    use axum::http::Method;
    r.admin_endpoint(
        Method::GET,
        "/security/posture",
        Capability::SecurityRead,
        admin_posture,
    );
    r.admin_endpoint(
        Method::GET,
        "/sessions",
        Capability::SessionsRead,
        admin_sessions,
    );
    r.admin_endpoint(
        Method::POST,
        "/sessions/revoke",
        Capability::SessionsRevoke,
        admin_revoke,
    );
}

async fn admin_posture(
    State(state): State<AppState>,
    _ctx: crate::admin_ctx::AdminCtx,
) -> Response {
    let cutoff = (chrono::Utc::now() - chrono::Duration::days(90)).to_rfc3339();
    match state.store.security_posture_counts(&cutoff).await {
        Ok(mut value) => {
            value["upstreamAllowlistConfigured"] = json!(
                state.security.jmap_upstreams.is_some()
                    || state.security.upstream_default.is_some()
            );
            value["environmentAdminLoginEnabled"] = json!(
                state.v6.admin_username.is_some()
                    && std::env::var("MW_ADMIN_ENV_LOGIN").as_deref() != Ok("off")
            );
            Json(value).into_response()
        }
        Err(e) => error("security posture", e),
    }
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SessionFilter {
    account_id: Option<String>,
    domain: Option<String>,
    client_type: Option<String>,
    handle: Option<String>,
}
fn matches_session(
    ctx: &crate::admin_ctx::AdminCtx,
    filter: &SessionFilter,
    row: &mw_store::SecuritySession,
) -> bool {
    let domain = row
        .username
        .rsplit_once('@')
        .map_or("", |(_, d)| d)
        .to_ascii_lowercase();
    ctx.require_domain(&domain).is_ok()
        && filter
            .account_id
            .as_ref()
            .is_none_or(|v| v == &row.account_id)
        && filter
            .domain
            .as_ref()
            .is_none_or(|v| v.eq_ignore_ascii_case(&domain))
        && filter
            .client_type
            .as_ref()
            .is_none_or(|v| v == &row.device.client_type)
        && filter.handle.as_ref().is_none_or(|v| v == &row.handle)
}
async fn admin_sessions(
    State(state): State<AppState>,
    ctx: crate::admin_ctx::AdminCtx,
    Query(filter): Query<SessionFilter>,
) -> Response {
    match state
        .store
        .list_security_sessions(filter.account_id.as_deref(), 1000)
        .await
    {
        Ok(rows) => {
            let truncated = rows.len() == 1000;
            let rows: Vec<_> = rows
                .into_iter()
                .filter(|row| matches_session(&ctx, &filter, row))
                .collect();
            Json(json!({"sessions":rows,"truncated":truncated})).into_response()
        }
        Err(e) => error("admin sessions", e),
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RevokeSessions {
    filter: SessionFilter,
    #[serde(default = "preview_default")]
    dry_run: bool,
    /// The exact opaque handles reviewed in the preview. Never selects new sessions.
    #[serde(default)]
    handles: Vec<String>,
}
fn preview_default() -> bool {
    true
}
async fn admin_revoke(
    State(state): State<AppState>,
    ctx: crate::admin_ctx::AdminCtx,
    Json(body): Json<RevokeSessions>,
) -> Response {
    if body.handles.len() > 1000 {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    let rows = match state
        .store
        .list_security_sessions(body.filter.account_id.as_deref(), 1000)
        .await
    {
        Ok(rows) => rows,
        Err(e) => return error("admin sessions", e),
    };
    let truncated = rows.len() == 1000;
    let rows: Vec<_> = rows
        .into_iter()
        .filter(|row| matches_session(&ctx, &body.filter, row))
        .collect();
    if body.dry_run {
        return Json(json!({"dryRun":true,"sessions":rows,"truncated":truncated})).into_response();
    }
    if body.handles.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"code":"previewRequired"})),
        )
            .into_response();
    }
    let mut revoked = 0;
    for row in rows
        .into_iter()
        .filter(|row| body.handles.contains(&row.handle))
    {
        match state
            .store
            .revoke_session_by_handle(&row.account_id, &row.handle)
            .await
        {
            Ok(Some(id)) => {
                state.sessions.forget(&id);
                state.step_up.forget_user(&id);
                revoked += 1;
                crate::audit::security_event(
                    &state,
                    crate::audit::SecurityEvent::admin(&ctx.username, "session-revoked")
                        .target(format!("account:{}", row.account_id))
                        .detail(json!({"handle":row.handle})),
                )
                .await;
            }
            Ok(None) => {}
            Err(e) => return error("revoke session", e),
        }
    }
    Json(json!({"dryRun":false,"revoked":revoked})).into_response()
}

pub(crate) async fn record_failure(
    state: &AppState,
    account: Option<&str>,
    address: &str,
    device: &SessionDevice,
    method: &str,
    reason: &str,
) {
    record_failure_backend(state, account, address, device, method, reason, None).await;
}

pub(crate) async fn record_failure_backend(
    state: &AppState,
    account: Option<&str>,
    address: &str,
    device: &SessionDevice,
    method: &str,
    reason: &str,
    backend: Option<&str>,
) {
    if let Some(line) = webmail_fail2ban_line(address, &device.ip, reason) {
        tracing::warn!("{line}");
    }
    let known = if account.is_none() {
        state
            .store
            .security_account_for_address(address.trim())
            .await
            .ok()
            .flatten()
    } else {
        None
    };
    let account = account.or(known.as_deref());
    let event = SigninEvent {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: account.map(str::to_owned),
        username_attempted: bounded(address, 320),
        ts: chrono::Utc::now().timestamp(),
        result: "failure".into(),
        method: bounded(method, 32),
        ip: device.ip.clone(),
        user_agent: device.user_agent.clone(),
        geo_country: device
            .ip
            .parse()
            .ok()
            .and_then(mw_engine::security::country_for_ip),
        reason: bounded(reason, 64),
    };
    crate::auth_logs::record(state, &event, mw_store::AuthSurface::Webmail, backend).await;
}

fn webmail_fail2ban_line(address: &str, ip: &str, reason: &str) -> Option<String> {
    if !crate::signin_guard::credential_rejection(reason) {
        return None;
    }
    let ip: std::net::IpAddr = ip.parse().ok()?;
    Some(mw_admin::fail2ban_line(chrono::Utc::now(), address, ip))
}
#[cfg(test)]
mod auth_log_failure_tests {
    use super::{session_deadline_passed, webmail_fail2ban_line};
    #[test]
    fn mailbox_activity_has_fixed_absolute_cap_and_separate_idle_clock() {
        let start = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let created = start.to_rfc3339();
        let cap = crate::hardening::MAILBOX_ABSOLUTE_CAP_SECS;
        for day in [1, 7, 14] {
            let active = start + chrono::Duration::days(day);
            assert!(!session_deadline_passed(
                &created,
                &active.to_rfc3339(),
                active,
                1800,
                cap
            ));
        }
        let deadline = start + chrono::Duration::seconds(cap as i64);
        assert!(session_deadline_passed(
            &created,
            &deadline.to_rfc3339(),
            deadline,
            1800,
            cap * 2
        ));
        assert!(session_deadline_passed(
            &created,
            &created,
            start + chrono::Duration::seconds(1800),
            1800,
            cap
        ));
        let short = start + chrono::Duration::hours(12);
        assert!(session_deadline_passed(
            &created,
            &short.to_rfc3339(),
            short,
            1800,
            43200
        ));
        assert!(session_deadline_passed(
            "invalid", &created, start, 1800, cap
        ));
    }

    #[test]
    fn bans_only_known_credential_failures_with_resolved_source() {
        for reason in [
            "timeout",
            "tls",
            "certificate",
            "dns",
            "connect",
            "transport",
            "backend-failure",
            "not-configured",
            "domain-denied",
            "account-disabled",
            "directory-rejected",
        ] {
            assert!(webmail_fail2ban_line("user", "192.0.2.1", reason).is_none());
        }
        assert!(webmail_fail2ban_line("user", "unknown", "auth-rejected").is_none());
        for reason in ["auth-rejected", "credential-rejected", "factor_refused"] {
            let line =
                webmail_fail2ban_line("attacker\nrhost=198.51.100.2", "192.0.2.1", reason).unwrap();
            assert_eq!(line.lines().count(), 1);
            assert!(line.ends_with("rhost=192.0.2.1"));
        }
    }
}

#[cfg(test)]
mod session_scope_tests {
    use super::*;
    use crate::admin_ctx::{AdminCtx, DomainScope};
    use crate::capabilities::Capability;
    #[test]
    fn session_filter_never_expands_domain_grant() {
        let ctx = AdminCtx::with_grants(
            "operator",
            [(
                Capability::SessionsRead,
                DomainScope::Domains(["example.org".into()].into_iter().collect()),
            )]
            .into_iter()
            .collect(),
        )
        .for_route(Some(Capability::SessionsRead));
        let mut row = mw_store::SecuritySession {
            handle: "opaque".into(),
            account_id: "account".into(),
            username: "user@example.org".into(),
            created_at: String::new(),
            last_seen: String::new(),
            last_activity: String::new(),
            device: SessionDevice::default(),
        };
        assert!(matches_session(&ctx, &SessionFilter::default(), &row));
        row.username = "user@other.org".into();
        assert!(!matches_session(&ctx, &SessionFilter::default(), &row));
        assert!(!matches_session(
            &ctx,
            &SessionFilter {
                domain: Some("other.org".into()),
                ..Default::default()
            },
            &row
        ));
    }
}
