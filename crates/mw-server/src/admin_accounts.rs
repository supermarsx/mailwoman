//! Admin accounts: sign-in, the admin's own profile, and the accounts API
//! (t32-ext §M, schedule §13.5 e24c).
//!
//! **Sign-in** (`POST /admin/login`): an account in `admin_account` with a password
//! hash signs in with that password (Argon2id). The environment credential
//! (`MW_ADMIN_USER` / `MW_ADMIN_PASSWORD`) signs in while `MW_ADMIN_ENV_LOGIN` is not
//! `off`, unless a database account with that username has a password of its own.
//! The first environment sign-in on a server with no admin accounts creates the
//! account `MW_ADMIN_USER` (`source = env-bootstrap`, hashed password) holding
//! Super admin. A disabled account is refused like a wrong password. When the
//! account has a TOTP factor the answer is `{twofaRequired, pendingToken,
//! factors:["totp"]}` with no cookie, and `POST /admin/login/totp {pendingToken,
//! code}` finishes it: `200 {username}` + cookie, `401` wrong code, `410` unknown or
//! expired token (wire contract, schedule §10).
//! Enrolled passkeys also require a factor after password verification; the
//! separate `admin_passkeys` module handles UV-required passwordless ceremonies
//! and password-backed recovery, without sharing mailbox credentials.
//!
//! **Own profile**: `GET /admin/me`, `GET/PUT /admin/me/preferences` (the console's
//! appearance, at most 4 KiB, stored as setting `admin_pref:<account>`),
//! `POST /admin/me/totp` + `POST /admin/me/totp/confirm` + `DELETE /admin/me/totp`,
//! `PUT /admin/me/password`.
//!
//! **Accounts API**: `GET/POST /admin/admins`, `GET/PUT/DELETE /admin/admins/{id}`,
//! `POST /admin/admins/{id}/reset-password`, `POST/DELETE /admin/admins/{id}/totp`,
//! `PUT /admin/admins/{id}/roles`, `PUT /admin/admins/{id}/overrides`,
//! `GET /admin/admin-sessions`, `DELETE /admin/admin-sessions/{id}`.
//! Every write is audited with the actor's roles and the capability it used.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use mw_mfa::totp::{self, TotpParams};
use mw_store::{AdminAccountRow, AdminRoleGrant};

use crate::AppState;
use crate::admin_ctx::{AdminApiError, AdminCtx, AdminJson, check_if_match, etag_of, with_etag};
use crate::admin_roles;
use crate::admin_router::{Access, AdminRegistrar};
use crate::capabilities::{Capability, SUPER_ADMIN};

/// How long a password-checked sign-in waits for its TOTP code.
const PENDING_TTL: Duration = Duration::from_secs(300);
/// Wrong codes allowed per pending sign-in.
const MAX_ATTEMPTS: u32 = 5;
/// How long an enrolment secret waits for its first code.
const ENROL_TTL: Duration = Duration::from_secs(600);
/// The largest stored console preference document.
const PREFS_MAX_BYTES: usize = 4096;
const MIN_PASSWORD_CHARS: usize = 12;
const MAX_PASSWORD_BYTES: usize = 1024;
const TOTP_ISSUER: &str = "Mailwoman admin";

/// The accounts routes.
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::POST, "/login", Access::Public, login);
    r.admin_endpoint(Method::POST, "/login/totp", Access::Public, login_totp);
    r.admin_endpoint(Method::POST, "/logout", Access::Public, logout);
    r.admin_endpoint(Method::GET, "/session", Access::SignedIn, session);
    r.admin_endpoint(Method::GET, "/me", Access::SignedIn, me);
    r.admin_endpoint(Method::GET, "/me/preferences", Access::SignedIn, get_prefs);
    r.admin_endpoint(Method::PUT, "/me/preferences", Access::SignedIn, put_prefs);
    r.admin_endpoint(Method::POST, "/me/totp", Access::SignedIn, me_totp_start);
    r.admin_endpoint(
        Method::POST,
        "/me/totp/confirm",
        Access::SignedIn,
        me_totp_confirm,
    );
    r.admin_endpoint(Method::DELETE, "/me/totp", Access::SignedIn, me_totp_delete);
    r.admin_endpoint(Method::PUT, "/me/password", Access::SignedIn, me_password);
    r.admin_endpoint(Method::GET, "/admins", Capability::AdminsRead, list_admins);
    r.admin_endpoint(
        Method::POST,
        "/admins",
        Capability::AdminsManage,
        create_admin,
    );
    r.admin_endpoint(
        Method::GET,
        "/admins/{id}",
        Capability::AdminsRead,
        get_admin,
    );
    r.admin_endpoint(
        Method::PUT,
        "/admins/{id}",
        Capability::AdminsManage,
        update_admin,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/admins/{id}",
        Capability::AdminsManage,
        delete_admin,
    );
    r.admin_endpoint(
        Method::POST,
        "/admins/{id}/reset-password",
        Capability::AdminsManage,
        reset_password,
    );
    r.admin_endpoint(
        Method::POST,
        "/admins/{id}/totp",
        Capability::AdminsManage,
        admin_totp_set,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/admins/{id}/totp",
        Capability::AdminsManage,
        admin_totp_delete,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/admins/{id}/passkeys",
        Capability::AdminsManage,
        admin_passkeys_delete,
    );
    r.admin_endpoint(
        Method::PUT,
        "/admins/{id}/roles",
        Capability::AdminsManage,
        set_roles,
    );
    r.admin_endpoint(
        Method::PUT,
        "/admins/{id}/overrides",
        Capability::AdminsManage,
        set_overrides,
    );
    r.admin_endpoint(
        Method::GET,
        "/admin-sessions",
        Capability::AdminsRead,
        list_sessions,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/admin-sessions/{id}",
        Capability::AdminsManage,
        revoke_session,
    );
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

// ─── environment credential ─────────────────────────────────────────────────────

/// Whether the environment credential may sign in (`MW_ADMIN_ENV_LOGIN` unset or
/// anything but `off`).
pub(crate) fn env_login_enabled() -> bool {
    !std::env::var("MW_ADMIN_ENV_LOGIN")
        .map(|v| v.trim().eq_ignore_ascii_case("off"))
        .unwrap_or(false)
}

/// Is `username` the environment admin?
pub(crate) fn is_env_admin(state: &AppState, username: &str) -> bool {
    state.v6.admin_username.as_deref() == Some(username)
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |d, (x, y)| d | (x ^ y)) == 0
}

fn env_credential_matches(state: &AppState, username: &str, password: &str) -> bool {
    let (Some(user), Some(pass)) = (&state.v6.admin_username, &state.v6.admin_password) else {
        return false;
    };
    // Both compared, whatever the first says.
    let u = ct_eq(username.as_bytes(), user.as_bytes());
    let p = ct_eq(password.as_bytes(), pass.as_bytes());
    u & p
}

/// Create the environment admin's account (Super admin) when the server has no
/// admin account yet.
async fn bootstrap(state: &AppState, username: &str, password: &str) -> Result<(), AdminApiError> {
    admin_roles::ensure_builtin_roles(state).await?;
    if state.store.count_admin_accounts().await? > 0 {
        return Ok(());
    }
    let id = uuid::Uuid::new_v4().to_string();
    let t = now();
    let row = AdminAccountRow {
        id: id.clone(),
        username: username.to_string(),
        display_name: username.to_string(),
        password_hash: Some(hash_password(state, password).await?),
        disabled: false,
        source: "env-bootstrap".into(),
        created_at: t,
        updated_at: t,
        last_login_at: None,
    };
    state
        .store
        .insert_admin_account_with_roles(
            &row,
            &[AdminRoleGrant {
                role_id: SUPER_ADMIN.into(),
                domains: None,
            }],
            true,
        )
        .await?;
    tracing::info!(
        "admin: created admin account {username} (Super admin) from the environment credential"
    );
    Ok(())
}

// ─── passwords ──────────────────────────────────────────────────────────────────

async fn hash_password(state: &AppState, password: &str) -> Result<String, AdminApiError> {
    let policy = state
        .v6
        .admin
        .get_security_policy()
        .await
        .map_err(AdminApiError::internal)?;
    let password = password.to_string();
    tokio::task::spawn_blocking(move || mw_admin::password::hash_password(&password, &policy))
        .await
        .map_err(AdminApiError::internal)?
        .map_err(AdminApiError::internal)
}

async fn verify_password(password: &str, phc: &str) -> bool {
    let (password, phc) = (password.to_string(), phc.to_string());
    tokio::task::spawn_blocking(move || {
        mw_admin::password::verify_password(&password, &phc).unwrap_or(false)
    })
    .await
    .unwrap_or(false)
}

fn check_new_password(password: &str, field: &str) -> Result<(), AdminApiError> {
    if password.chars().count() < MIN_PASSWORD_CHARS || password.len() > MAX_PASSWORD_BYTES {
        return Err(AdminApiError::invalid(
            "weakPassword",
            format!("A password has at least {MIN_PASSWORD_CHARS} characters."),
            field,
        ));
    }
    Ok(())
}

// ─── session cookie and CSRF ────────────────────────────────────────────────────

/// The admin cookie. `Max-Age` is the session's absolute cap: the idle deadline is
/// shorter and moves, and only the server can know it, so the cookie carries the
/// one figure that is fixed at login.
///
/// `Path` is the console's path under the deploy prefix (t76), so the browser
/// sends the cookie only to the console and its API, never to the mail app.
fn set_admin_cookie(token: &str, secure: bool, absolute: i64, base: &str) -> String {
    let mut c = format!(
        "{}={token}; HttpOnly; SameSite=Strict; Path={}; Max-Age={}",
        mw_admin::config::session_cookie_name(),
        admin_cookie_path(base),
        absolute
    );
    if secure {
        c.push_str("; Secure");
    }
    c
}

/// The admin cookie's `Path`: `{base}{console}`.
pub(crate) fn admin_cookie_path(base: &str) -> String {
    format!("{base}{}", mw_admin::config::admin_path())
}

/// Clear the admin cookie at `path`.
fn clear_admin_cookie_at(secure: bool, path: &str) -> String {
    let mut c = format!(
        "{}=; HttpOnly; SameSite=Strict; Path={path}; Max-Age=0",
        mw_admin::config::session_cookie_name()
    );
    if secure {
        c.push_str("; Secure");
    }
    c
}

fn admin_cookie(headers: &HeaderMap) -> Option<String> {
    crate::hardening::cookie(headers, &mw_admin::config::session_cookie_name())
}

/// The answer of `POST /admin/login`, `/admin/login/totp` and `GET /admin/session`.
/// `csrfToken` is the value of the `mw_csrf` cookie, for a client that cannot read
/// cookies.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminSessionDto {
    username: String,
    csrf_token: String,
}

pub(crate) async fn session_timeouts(state: &AppState) -> Result<(i64, i64), AdminApiError> {
    let subject = mw_store::settings_policy::SettingSubject::default();
    let mut values = [1800, 43200];
    for (index, key) in ["admin.session.idleSecs", "admin.session.absoluteSecs"]
        .iter()
        .enumerate()
    {
        if let Some(value) =
            crate::settings_registry::effective_for(&state.store, key, &subject).await?
        {
            values[index] = value.value.as_i64().unwrap_or(values[index]);
        }
    }
    Ok((values[0].clamp(300, 7200), values[1].clamp(3600, 86400)))
}

/// Start an admin session for `username` and answer with it.
pub(crate) async fn start_session(state: &AppState, username: &str) -> Response {
    start_session_for_factor(state, username, None).await
}

pub(crate) async fn start_session_for_factor(
    state: &AppState,
    username: &str,
    factor: Option<mw_store::AdminSessionFactor<'_>>,
) -> Response {
    let token = crate::push_relay::hash_token(&format!(
        "{}:{}",
        mw_store::ServerKey::generate().to_hex(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    let hash = crate::push_relay::hash_token(&token);
    let (idle, absolute) = match session_timeouts(state).await {
        Ok(limits) => limits,
        Err(e) => return e.into_response(),
    };
    let inserted = match factor {
        Some(factor) => {
            state
                .store
                .put_admin_session_for_factor(&hash, username, now(), (idle, absolute), factor)
                .await
        }
        None => state
            .store
            .put_admin_session_with_timeouts(&hash, username, now(), idle, absolute)
            .await
            .map(|()| true),
    };
    match inserted {
        Ok(true) => {}
        Ok(false) => return AdminApiError::unauthenticated().into_response(),
        Err(e) => return AdminApiError::internal(e).into_response(),
    }
    if let Ok(Some(acc)) = state.store.get_admin_account_by_username(username).await {
        let _ = state.store.touch_admin_login(&acc.id, now()).await;
    }
    // A new token with every sign-in, as `/api/login` issues
    // (`authenticated_response`). The panel and the mailbox app share the one
    // `mw_csrf` cookie and each reads it when it sends a request, so replacing it
    // here does not strand an open mailbox tab.
    let csrf_token = state.store.session_csrf_token(&token);
    let mut resp = Json(AdminSessionDto {
        username: username.to_string(),
        csrf_token: csrf_token.clone(),
    })
    .into_response();
    // The admin cookie stays the first `Set-Cookie`.
    if let Ok(v) = set_admin_cookie(&token, state.cookie_secure, absolute, &state.base_path).parse()
    {
        resp.headers_mut().append(header::SET_COOKIE, v);
    }
    resp.headers_mut().append(
        header::SET_COOKIE,
        crate::csrf_cookie(&csrf_token, state.cookie_secure),
    );
    // t63 R2: this browser is now an admin browser; it gets no analytics tag while it carries the marker.
    if let Ok(v) = crate::analytics_tag::admin_browser_cookie(state.cookie_secure).parse() {
        resp.headers_mut().append(header::SET_COOKIE, v);
    }
    resp
}

// ─── sign-in ────────────────────────────────────────────────────────────────────

struct Pending {
    username: String,
    account_id: String,
    password_hash: Option<String>,
    expires: Instant,
    attempts: u32,
}

static PENDING: LazyLock<Mutex<HashMap<String, Pending>>> = LazyLock::new(Default::default);

/// Enrolment secrets are bound to the initiating administrator session.
struct PendingEnrollment {
    secret: zeroize::Zeroizing<Vec<u8>>,
    session_hash: String,
    expires: Instant,
    attempts: u32,
}
type PendingEnrollments = HashMap<String, PendingEnrollment>;
static ENROLLING: LazyLock<Mutex<PendingEnrollments>> = LazyLock::new(Default::default);

fn lock<T>(m: &'static Mutex<T>) -> std::sync::MutexGuard<'static, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn new_token() -> String {
    let mut bytes = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Deserialize)]
struct LoginReq {
    username: String,
    password: String,
    #[serde(default)]
    altcha: Option<String>,
}

/// What the credentials of a sign-in turn out to be.
enum Checked {
    /// Signed in as this username.
    Ok(String),
    /// Password right; the account's TOTP code is next.
    NeedsTotp {
        username: String,
        account_id: String,
        password_hash: Option<String>,
    },
    Refused,
    Disabled,
}

async fn check_credentials(
    state: &AppState,
    username: &str,
    password: &str,
) -> Result<Checked, AdminApiError> {
    let account = state.store.get_admin_account_by_username(username).await?;
    if let Some(acc) = &account
        && let Some(phc) = &acc.password_hash
    {
        // A database account with its own password: only that password signs in.
        if acc.disabled || (acc.source == "env-bootstrap" && !env_login_enabled()) {
            return Ok(Checked::Disabled);
        }
        if !verify_password(password, phc).await {
            return Ok(Checked::Refused);
        }
        return totp_step(state, acc).await;
    }
    if !env_login_enabled() || !env_credential_matches(state, username, password) {
        return Ok(Checked::Refused);
    }
    bootstrap(state, username, password).await?;
    match state.store.get_admin_account_by_username(username).await? {
        Some(acc) if acc.disabled => Ok(Checked::Disabled),
        Some(acc) => totp_step(state, &acc).await,
        // Accounts exist, none for the environment admin (it was deleted): the
        // credential still signs in while env sign-in is on, holding nothing.
        None => Ok(Checked::Refused),
    }
}

async fn totp_step(state: &AppState, acc: &AdminAccountRow) -> Result<Checked, AdminApiError> {
    if state.store.get_admin_totp(&acc.id).await?.is_some()
        || !state.store.list_admin_passkeys(&acc.id).await?.is_empty()
    {
        Ok(Checked::NeedsTotp {
            username: acc.username.clone(),
            account_id: acc.id.clone(),
            password_hash: acc.password_hash.clone(),
        })
    } else {
        Ok(Checked::Ok(acc.username.clone()))
    }
}

/// `POST /admin/login {username, password}`.
async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    // The login monitor needs the address the attempt came from. It lives in the
    // request extensions as `ConnectInfo`, installed by the serve path.
    ext: axum::http::Extensions,
    Json(body): Json<LoginReq>,
) -> Response {
    if !state.v6.admin_enabled {
        return AdminApiError::unauthenticated().into_response();
    }
    // The one seam for "who is this" (`proxy::client_ip`): the peer address, refined
    // by a forwarded header only from a configured proxy.
    let source = crate::scope_mw::proxy::client_ip(&headers, &ext);
    if let Err(error) = check_login_source(&state, source).await {
        crate::auth_logs::record_admin_reason(
            &state,
            &body.username,
            source,
            "password",
            false,
            "source-blocked",
        )
        .await;
        return error.into_response();
    }
    // Settled below: only an attempt that starts a session gives its charge back
    // (t33 D1); every other outcome keeps it by dropping the admission.
    let admission = match crate::signin_cooldown::check(&state, &headers, &ext).await {
        Ok(admission) => admission,
        Err(response) => return response,
    };
    if let Some(response) = crate::signin_challenge::verify(
        &state,
        crate::signin_challenge::Surface::Admin,
        &headers,
        &ext,
        body.altcha.as_deref(),
    )
    .await
    {
        return response;
    }
    match check_credentials(&state, &body.username, &body.password).await {
        Ok(Checked::Ok(username)) => {
            let response = start_session(&state, &username).await;
            if response.status().is_success() {
                admission.succeeded(&state);
                record_login_success(&state, &username, source).await;
            }
            crate::auth_logs::record_admin_reason(
                &state,
                &username,
                source,
                "password",
                response.status().is_success(),
                if response.status().is_success() {
                    ""
                } else {
                    "session-refused"
                },
            )
            .await;
            response
        }
        Ok(Checked::NeedsTotp {
            username,
            account_id,
            password_hash,
        }) => {
            let mut factors = Vec::new();
            if matches!(state.store.get_admin_totp(&account_id).await, Ok(Some(_))) {
                factors.push("totp");
            }
            if state
                .store
                .list_admin_passkeys(&account_id)
                .await
                .is_ok_and(|v| !v.is_empty())
            {
                factors.push("webauthn");
            }
            if state
                .store
                .admin_recovery_hashes(&account_id)
                .await
                .is_ok_and(|v| !v.is_empty())
            {
                factors.push("recovery");
            }
            let token = new_token();
            let mut pending = lock(&PENDING);
            pending.retain(|_, p| p.expires > Instant::now());
            pending.insert(
                token.clone(),
                Pending {
                    username,
                    account_id,
                    password_hash,
                    expires: Instant::now() + PENDING_TTL,
                    attempts: 0,
                },
            );
            Json(json!({
                "twofaRequired": true,
                "pendingToken": token,
                "factors": factors,
            }))
            .into_response()
        }
        Ok(Checked::Disabled) => {
            crate::auth_logs::record_admin_reason(
                &state,
                &body.username,
                source,
                "password",
                false,
                "account-disabled",
            )
            .await;
            AdminApiError::unauthenticated().into_response()
        }
        Ok(Checked::Refused) => {
            record_admin_login_failure(&state, &body.username, source).await;
            crate::auth_logs::record_admin(&state, &body.username, source, "password", false).await;
            AdminApiError::unauthenticated().into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TotpLoginReq {
    pending_token: String,
    code: String,
}

/// Consume a password-verified challenge for recovery without sharing mailbox tokens.
pub(crate) async fn take_password_pending(
    state: &AppState,
    token: &str,
) -> Result<AdminAccountRow, AdminApiError> {
    if !state.v6.admin_enabled {
        return Err(AdminApiError::unauthenticated());
    }
    let pending = lock(&PENDING)
        .remove(token)
        .filter(|p| p.expires > Instant::now())
        .ok_or_else(AdminApiError::unauthenticated)?;
    state
        .store
        .get_admin_account(&pending.account_id)
        .await?
        .filter(|a| {
            !a.disabled
                && a.password_hash == pending.password_hash
                && (a.source != "env-bootstrap" || env_login_enabled())
        })
        .ok_or_else(AdminApiError::unauthenticated)
}

/// `POST /admin/login/totp {pendingToken, code}`.
async fn login_totp(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    Json(body): Json<TotpLoginReq>,
) -> Response {
    if !state.v6.admin_enabled {
        return AdminApiError::unauthenticated().into_response();
    }
    let source = crate::scope_mw::proxy::client_ip(&headers, &ext);
    if let Err(error) = check_login_source(&state, source).await {
        return error.into_response();
    }
    let found = lock(&PENDING)
        .remove(&body.pending_token)
        .filter(|p| p.expires > Instant::now());
    let Some(mut challenge) = found else {
        return AdminApiError::new(StatusCode::GONE, "expired", "Enter your password again.")
            .into_response();
    };
    let username = challenge.username.clone();
    let account_id = challenge.account_id.clone();
    match state.store.get_admin_account(&account_id).await {
        Ok(Some(account))
            if !account.disabled && account.password_hash == challenge.password_hash => {}
        Ok(_) => return AdminApiError::unauthenticated().into_response(),
        Err(e) => return AdminApiError::internal(e).into_response(),
    }
    match verify_totp(&state, &account_id, &body.code).await {
        Ok(true) => {
            match state.store.get_admin_account(&account_id).await {
                Ok(Some(account))
                    if !account.disabled && account.password_hash == challenge.password_hash => {}
                _ => return AdminApiError::unauthenticated().into_response(),
            }
            lock(&PENDING).remove(&body.pending_token);
            let response = start_session(&state, &username).await;
            if response.status().is_success() {
                record_login_success(&state, &username, source).await;
            }
            crate::auth_logs::record_admin_reason(
                &state,
                &username,
                source,
                "password+totp",
                response.status().is_success(),
                if response.status().is_success() {
                    ""
                } else {
                    "session-refused"
                },
            )
            .await;
            response
        }
        Ok(false) => {
            challenge.attempts += 1;
            if challenge.attempts < MAX_ATTEMPTS && challenge.expires > Instant::now() {
                lock(&PENDING).insert(body.pending_token, challenge);
            }
            record_admin_login_failure(&state, &username, source).await;
            crate::auth_logs::record_admin(&state, &username, source, "password+totp", false).await;
            AdminApiError::new(
                StatusCode::UNAUTHORIZED,
                "wrongCode",
                "That code is not right.",
            )
            .into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// Check `code` against the account's factor and consume its step (a code works
/// once).
async fn verify_totp(
    state: &AppState,
    account_id: &str,
    code: &str,
) -> Result<bool, AdminApiError> {
    let Some(secret) = state.store.get_admin_totp(account_id).await? else {
        return Ok(false);
    };
    let unix = u64::try_from(now()).unwrap_or_default();
    let Some(step) = totp::totp_verify(&secret, code.trim(), unix, &TotpParams::default()) else {
        return Ok(false);
    };
    Ok(state
        .store
        .advance_admin_totp_step(account_id, i64::try_from(step).unwrap_or(i64::MAX))
        .await?)
}

pub(crate) async fn check_login_source(
    state: &AppState,
    source: Option<std::net::IpAddr>,
) -> Result<(), AdminApiError> {
    crate::signin_guard::check(state, mw_store::AuthSurface::Admin, source).await?;
    if let Some(ip) = source {
        match state.v6.admin.is_banned(&ip.to_string()).await {
            Ok(true) => return Err(AdminApiError::unauthenticated()),
            Ok(false) => {}
            Err(error) => return Err(AdminApiError::internal(error)),
        }
    }
    Ok(())
}

pub(crate) async fn record_login_success(
    state: &AppState,
    username: &str,
    source: Option<std::net::IpAddr>,
) {
    if let Err(e) = state.v6.admin.record_login_success(username, source).await {
        tracing::warn!("admin login-success not recorded: {e}");
    }
}

/// Feed one admin-login failure to the login monitor and **emit its fail2ban
/// line**: `mw_admin::banlist` hands an operator's fail2ban jail a line it can
/// match, and the jail is the enforcement. The line carries its own RFC 3339
/// timestamp; [`mw_admin::FAIL2BAN_FAILREGEX`] anchors on the `mailwoman[auth]:`
/// token, not on position.
pub(crate) async fn record_admin_login_failure(
    state: &AppState,
    username: &str,
    source: Option<std::net::IpAddr>,
) {
    if source.is_none() {
        warn_no_source_once();
    }
    match state
        .v6
        .admin
        .record_login_failure_without_monitor(username, source)
        .await
    {
        Ok(outcome) => {
            if let Some(line) = outcome.log_line {
                // Deliberately the raw line, unstructured: a jail reads it verbatim.
                tracing::warn!("{line}");
            }
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

/// Say once that the monitor has nothing to work with: no source address means
/// the serve path installed no `ConnectInfo`. Once per process, because this fires
/// on a path an attacker can drive.
fn warn_no_source_once() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        tracing::warn!(
            "admin login failure has no client address (no ConnectInfo on the request): \
             the login monitor cannot count it and no fail2ban line is emitted"
        );
    });
}

/// `POST /admin/logout`.
async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(token) = admin_cookie(&headers) {
        let hash = crate::push_relay::hash_token(&token);
        let _ = state.store.delete_admin_session(&hash).await;
    }
    let mut resp = StatusCode::NO_CONTENT.into_response();
    // The scoped cookie, then the `Path=/` one a sign-in before t76 set.
    for path in [admin_cookie_path(&state.base_path), "/".to_string()] {
        if let Ok(v) = clear_admin_cookie_at(state.cookie_secure, &path).parse() {
            resp.headers_mut().append(header::SET_COOKIE, v);
        }
    }
    resp
}

/// `GET /admin/session` — who is signed in. Also makes sure the panel holds a
/// CSRF token without replacing one it has (as `GET /api/me` does).
async fn session(State(state): State<AppState>, ctx: AdminCtx, headers: HeaderMap) -> Response {
    let existing = crate::hardening::cookie(&headers, crate::hardening::CSRF_COOKIE);
    let csrf_token = state
        .store
        .session_csrf_token(&admin_cookie(&headers).unwrap_or_default());
    let mut resp = Json(AdminSessionDto {
        username: ctx.username,
        csrf_token: csrf_token.clone(),
    })
    .into_response();
    if existing.as_deref() != Some(csrf_token.as_str()) {
        resp.headers_mut().append(
            header::SET_COOKIE,
            crate::csrf_cookie(&csrf_token, state.cookie_secure),
        );
    }
    resp
}

// ─── audit ──────────────────────────────────────────────────────────────────────

/// Append an audit row for a write by `ctx`: actor, roles, the route's capability,
/// target `kind:id`, and the object before and after (redacted).
pub(crate) async fn audit(
    state: &AppState,
    ctx: &AdminCtx,
    action: &str,
    target: &str,
    before: Option<Value>,
    after: Option<Value>,
) {
    let redact = |v: Option<Value>| v.map(|v| mw_admin::redact_detail(&v));
    let row = mw_store::AuditRow {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: ctx.username.clone(),
        actor_kind: "admin".into(),
        action: action.to_string(),
        target: Some(target.to_string()),
        detail_json: "{}".into(),
        ip: None,
    };
    let roles = ctx.audit_roles();
    let cap = ctx.route_capability().map(Capability::as_str);
    let (before, after) = (redact(before), redact(after));
    if let Err(e) = state
        .store
        .append_admin_audit(
            &row,
            (!roles.is_empty()).then_some(roles.as_str()),
            cap,
            before.as_deref(),
            after.as_deref(),
        )
        .await
    {
        tracing::warn!("admin audit append failed: {e}");
    }
}

// ─── own profile ────────────────────────────────────────────────────────────────

/// `GET /admin/me`.
async fn me(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    let totp_enrolled = match &ctx.account_id {
        Some(id) => state.store.get_admin_totp(id).await?.is_some(),
        None => false,
    };
    let caps: Vec<&str> = ctx.capabilities().iter().map(|c| c.as_str()).collect();
    Ok(Json(json!({
        "username": ctx.username,
        "displayName": ctx.display_name,
        "accountId": ctx.account_id,
        "roles": ctx.roles,
        "capabilities": caps,
        "domainScope": ctx.domain_scope(),
        "totpEnrolled": totp_enrolled,
        "envLogin": env_login_enabled(),
    }))
    .into_response())
}

fn require_account(ctx: &AdminCtx) -> Result<&str, AdminApiError> {
    ctx.account_id.as_deref().ok_or_else(|| {
        AdminApiError::conflict(
            "noAccount",
            "This sign-in has no admin account. Sign in again to create it.",
        )
    })
}

fn prefs_key(ctx: &AdminCtx) -> String {
    format!(
        "admin_pref:{}",
        ctx.account_id.as_deref().unwrap_or(&ctx.username)
    )
}

/// `GET /admin/me/preferences` — the console's own settings (appearance), `{}`
/// until saved.
async fn get_prefs(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    let stored = state.store.get_setting(&prefs_key(&ctx)).await?;
    let v: Value = stored
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));
    let tag = etag_of(&v);
    Ok(with_etag(Json(v), &tag))
}

/// `PUT /admin/me/preferences` — a JSON object of at most 4 KiB.
async fn put_prefs(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(body): AdminJson<Value>,
) -> Result<Response, AdminApiError> {
    if !body.is_object() {
        return Err(AdminApiError::invalid(
            "notAnObject",
            "Preferences are a JSON object.",
            "",
        ));
    }
    if body
        .get("leetspeak")
        .is_some_and(|value| !value.is_boolean())
    {
        return Err(AdminApiError::invalid(
            "invalidValue",
            "Leetspeak must be a boolean.",
            "/leetspeak",
        ));
    }
    let text = body.to_string();
    if text.len() > PREFS_MAX_BYTES {
        return Err(AdminApiError::invalid(
            "tooLarge",
            format!("Preferences are at most {PREFS_MAX_BYTES} bytes."),
            "",
        ));
    }
    let key = prefs_key(&ctx);
    let raw = state.store.get_setting(&key).await?;
    let current: Value = raw
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| json!({}));
    check_if_match(&headers, &etag_of(&current), None, None)?;
    if !state
        .store
        .compare_and_set_setting(&key, raw.as_deref(), &text)
        .await?
    {
        return Err(AdminApiError::stale(None, None));
    }
    let tag = etag_of(&body);
    Ok(with_etag(Json(body), &tag))
}

/// `POST /admin/me/totp` — a new secret for the admin's own factor; it is stored
/// only once a code from it is confirmed.
async fn me_totp_start(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::TwoFactorDisable)
        .await?;
    let id = require_account(&ctx)?;
    if state.store.get_admin_totp(id).await?.is_some() {
        return Err(AdminApiError::conflict(
            "alreadyEnrolled",
            "Remove the existing factor with a valid code before enrolling another.",
        ));
    }
    let secret = totp::generate_secret().to_vec();
    let uri = totp::provisioning_uri(&secret, TOTP_ISSUER, &ctx.username, &TotpParams::default());
    let encoded = totp::base32_encode(&secret);
    let mut map = lock(&ENROLLING);
    map.retain(|_, pending| pending.expires > Instant::now());
    if map.len() >= 1024 && !map.contains_key(id) {
        return Err(AdminApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "enrollmentBusy",
            "Try again shortly.",
        ));
    }
    map.insert(
        id.to_string(),
        PendingEnrollment {
            secret: zeroize::Zeroizing::new(secret),
            session_hash: ctx.session_hash.clone(),
            expires: Instant::now() + ENROL_TTL,
            attempts: 0,
        },
    );
    Ok((
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({ "secret": encoded, "otpauthUri": uri })),
    )
        .into_response())
}

#[derive(Deserialize)]
struct CodeReq {
    code: String,
}

/// `POST /admin/me/totp/confirm {code}` — store the factor started above.
async fn me_totp_confirm(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(body): AdminJson<CodeReq>,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::TwoFactorDisable)
        .await?;
    let id = require_account(&ctx)?.to_string();
    let unix = u64::try_from(now()).unwrap_or_default();
    let (secret, step) = {
        let mut map = lock(&ENROLLING);
        map.retain(|_, pending| pending.expires > Instant::now());
        let pending = map
            .get_mut(&id)
            .filter(|p| p.session_hash == ctx.session_hash)
            .ok_or_else(|| {
                AdminApiError::conflict(
                    "noEnrolment",
                    "Start authenticator setup again in this session.",
                )
            })?;
        let step = totp::totp_verify(
            &pending.secret,
            body.code.trim(),
            unix,
            &TotpParams::default(),
        );
        if let Some(step) = step {
            let secret = zeroize::Zeroizing::new(pending.secret.to_vec());
            map.remove(&id);
            (secret, step)
        } else {
            pending.attempts += 1;
            if pending.attempts >= MAX_ATTEMPTS {
                map.remove(&id);
            }
            return Err(AdminApiError::invalid(
                "wrongCode",
                "That code is not right. After five attempts, start setup again.",
                "/code",
            ));
        }
    };
    let codes = mw_mfa::recovery::generate_codes(mw_mfa::recovery::DEFAULT_RECOVERY_CODES);
    let hashes = codes
        .iter()
        .map(|code| mw_mfa::recovery::hash_code(code))
        .collect::<Vec<_>>();
    let Some(seeded) = state
        .store
        .enroll_admin_totp_with_recovery(
            &id,
            &ctx.session_hash,
            &secret,
            i64::try_from(step).unwrap_or(i64::MAX),
            now(),
            &hashes,
        )
        .await?
    else {
        return Err(AdminApiError::conflict(
            "enrollmentChanged",
            "The administrator account, session, or factor changed. Sign in and start setup again.",
        ));
    };
    invalidate_pending(&id);
    lock(&ENROLLING).remove(&id);
    audit(
        &state,
        &ctx,
        "admin-totp-enrolled",
        &format!("admin:{id}"),
        None,
        None,
    )
    .await;
    Ok((
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({"ok":true,"recoveryCodes":if seeded {codes} else {vec![]}})),
    )
        .into_response())
}

/// `DELETE /admin/me/totp` — fresh central proof already consumed the required
/// factor (TOTP, passkey, or recovery); asking for the same code again rejects it.
async fn me_totp_delete(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::TwoFactorDisable)
        .await?;
    let id = require_account(&ctx)?.to_string();
    invalidate_pending(&id);
    state.store.delete_admin_totp(&id).await?;
    audit(
        &state,
        &ctx,
        "admin-totp-removed",
        &format!("admin:{id}"),
        None,
        None,
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PasswordChange {
    current_password: String,
    new_password: String,
}

/// `PUT /admin/me/password {currentPassword, newPassword}`.
async fn me_password(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(body): AdminJson<PasswordChange>,
) -> Result<Response, AdminApiError> {
    let id = require_account(&ctx)?.to_string();
    let acc = state
        .store
        .get_admin_account(&id)
        .await?
        .ok_or_else(|| AdminApiError::not_found("No such admin."))?;
    let current_ok = match &acc.password_hash {
        Some(phc) => verify_password(&body.current_password, phc).await,
        None => env_credential_matches(&state, &acc.username, &body.current_password),
    };
    if !current_ok {
        return Err(AdminApiError::invalid(
            "wrongPassword",
            "The current password is not right.",
            "/currentPassword",
        ));
    }
    check_new_password(&body.new_password, "/newPassword")?;
    let phc = hash_password(&state, &body.new_password).await?;
    state.store.set_admin_password(&id, &phc, now()).await?;
    invalidate_pending(&id);
    state.store.delete_admin_sessions_of(&acc.username).await?;
    audit(
        &state,
        &ctx,
        "admin-password-changed",
        &format!("admin:{id}"),
        None,
        None,
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// ─── accounts API ───────────────────────────────────────────────────────────────

async fn account_view(state: &AppState, acc: &AdminAccountRow) -> Result<Value, AdminApiError> {
    let roles: Vec<Value> = state
        .store
        .admin_account_roles(&acc.id)
        .await?
        .into_iter()
        .map(|g| json!({ "roleId": g.role_id, "domains": g.domains }))
        .collect();
    let overrides: Vec<Value> = state
        .store
        .admin_overrides(&acc.id)
        .await?
        .into_iter()
        .map(|(c, e)| json!({ "capability": c, "effect": e }))
        .collect();
    let totp = state.store.get_admin_totp(&acc.id).await?.is_some();
    let (last_superadmin, allow_loss) = state.store.admin_superadmin_protection(&acc.id).await?;
    Ok(json!({
        "id": acc.id,
        "username": acc.username,
        "displayName": acc.display_name,
        "disabled": acc.disabled,
        "source": acc.source,
        "createdAt": acc.created_at,
        "updatedAt": acc.updated_at,
        "lastLoginAt": acc.last_login_at,
        "roles": roles,
        "overrides": overrides,
        "totpEnrolled": totp,
        "lastSuperadmin": last_superadmin,
        "lastSuperadminProtected": last_superadmin && !allow_loss,
        "allowLastSuperadminLoss": allow_loss,
        "passwordConfigured": acc.password_hash.as_ref().is_some_and(|p| !p.is_empty()),
        "passkeyCount": state.store.list_admin_passkeys(&acc.id).await?.len(),
    }))
}

async fn find(state: &AppState, id: &str) -> Result<AdminAccountRow, AdminApiError> {
    state
        .store
        .get_admin_account(id)
        .await?
        .ok_or_else(|| AdminApiError::not_found("No such admin."))
}

/// `GET /admin/admins`.
async fn list_admins(
    State(state): State<AppState>,
    _ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    let mut out = Vec::new();
    for acc in state.store.list_admin_accounts().await? {
        let mut v = account_view(&state, &acc).await?;
        v["etag"] = Value::String(etag_of(&v));
        out.push(v);
    }
    Ok(Json(json!({ "admins": out })).into_response())
}

/// `GET /admin/admins/{id}`.
async fn get_admin(
    State(state): State<AppState>,
    _ctx: AdminCtx,
    UrlPath(id): UrlPath<String>,
) -> Result<Response, AdminApiError> {
    let v = account_view(&state, &find(&state, &id).await?).await?;
    let tag = etag_of(&v);
    Ok(with_etag(Json(v), &tag))
}

async fn require_can_manage(
    state: &AppState,
    actor: &AdminCtx,
    account: &AdminAccountRow,
) -> Result<(), AdminApiError> {
    let assignments = state.store.admin_account_roles(&account.id).await?;
    let roles = state.store.list_admin_roles().await?;
    let overrides = state.store.admin_overrides(&account.id).await?;
    let (grants, _) = admin_roles::effective(&assignments, &roles, &overrides);
    admin_roles::require_can_grant(actor, &grants.keys().copied().collect::<Vec<_>>())
}

fn invalidate_pending(id: &str) {
    lock(&PENDING).retain(|_, p| p.account_id != id);
    lock(&ENROLLING).remove(id);
}

fn last_super_error() -> AdminApiError {
    AdminApiError::conflict(
        "lastSuperAdmin",
        "Give another enabled account Super admin first.",
    )
}

fn valid_username(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._@-".contains(&b))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GrantBody {
    role_id: String,
    #[serde(default)]
    domains: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateAdmin {
    username: String,
    #[serde(default)]
    display_name: Option<String>,
    password: String,
    #[serde(default)]
    roles: Vec<GrantBody>,
}

/// Check role assignments: each role exists, each domain list is non-empty, and
/// the writer may grant every capability involved.
async fn check_grants(
    state: &AppState,
    ctx: &AdminCtx,
    grants: &[GrantBody],
    pointer: &str,
) -> Result<Vec<AdminRoleGrant>, AdminApiError> {
    admin_roles::ensure_builtin_roles(state).await?;
    let roles = state.store.list_admin_roles().await?;
    let mut out = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (i, g) in grants.iter().enumerate() {
        if !seen.insert(&g.role_id) {
            return Err(AdminApiError::invalid(
                "duplicateRole",
                "Assign each role once.",
                format!("{pointer}/{i}/roleId"),
            ));
        }
        if g.domains.as_ref().is_some_and(|domains| {
            domains.iter().any(|domain| {
                let domain = domain.trim();
                domain.is_empty()
                    || domain.len() > 253
                    || domain.split('.').any(|label| {
                        label.is_empty()
                            || label.len() > 63
                            || label.starts_with('-')
                            || label.ends_with('-')
                            || !label
                                .bytes()
                                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                    })
            })
        }) {
            return Err(AdminApiError::invalid(
                "invalidDomain",
                "Enter valid domain names.",
                format!("{pointer}/{i}/domains"),
            ));
        }
        let Some(role) = roles.iter().find(|r| r.id == g.role_id) else {
            return Err(AdminApiError::invalid(
                "unknownRole",
                format!("{} is not a role.", g.role_id),
                format!("{pointer}/{i}/roleId"),
            ));
        };
        let domains = match &g.domains {
            None => None,
            Some(list) if list.is_empty() => {
                return Err(AdminApiError::invalid(
                    "noDomains",
                    "Name at least one domain, or leave the list out for all domains.",
                    format!("{pointer}/{i}/domains"),
                ));
            }
            Some(list) => Some(
                list.iter()
                    .map(|d| d.trim().to_ascii_lowercase())
                    .collect::<Vec<_>>(),
            ),
        };
        admin_roles::require_can_grant(ctx, &admin_roles::role_capabilities(role))?;
        out.push(AdminRoleGrant {
            role_id: g.role_id.clone(),
            domains,
        });
    }
    Ok(out)
}

/// `POST /admin/admins {username, displayName?, password, roles?}` → `201`.
async fn create_admin(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(body): AdminJson<CreateAdmin>,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::AdminRoleChange)
        .await?;
    let username = body.username.trim().to_ascii_lowercase();
    if !valid_username(&username) {
        return Err(AdminApiError::invalid(
            "invalidUsername",
            "A username has 1 to 64 characters: a-z, 0-9, dot, underscore, @ and hyphen.",
            "/username",
        ));
    }
    if state
        .store
        .get_admin_account_by_username(&username)
        .await?
        .is_some()
    {
        return Err(AdminApiError::invalid(
            "usernameTaken",
            format!("An admin named {username} already exists."),
            "/username",
        ));
    }
    check_new_password(&body.password, "/password")?;
    let grants = check_grants(&state, &ctx, &body.roles, "/roles").await?;
    let phc = hash_password(&state, &body.password).await?;
    let t = now();
    let row = AdminAccountRow {
        id: uuid::Uuid::new_v4().to_string(),
        display_name: body
            .display_name
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| username.clone()),
        username,
        password_hash: Some(phc),
        disabled: false,
        source: "db".into(),
        created_at: t,
        updated_at: t,
        last_login_at: None,
    };
    state
        .store
        .insert_admin_account_with_roles(&row, &grants, false)
        .await?;
    let v = account_view(&state, &row).await?;
    audit(
        &state,
        &ctx,
        "admin-created",
        &format!("admin:{}", row.id),
        None,
        Some(v.clone()),
    )
    .await;
    let tag = etag_of(&v);
    Ok(with_etag((StatusCode::CREATED, Json(v)), &tag))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UpdateAdmin {
    display_name: String,
    disabled: bool,
    #[serde(default)]
    dry_run: bool,
}

/// `PUT /admin/admins/{id} {displayName, disabled}`. Disabling ends the account's
/// sessions.
async fn update_admin(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(id): UrlPath<String>,
    headers: HeaderMap,
    AdminJson(body): AdminJson<UpdateAdmin>,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::AdminRoleChange)
        .await?;
    let acc = find(&state, &id).await?;
    require_can_manage(&state, &ctx, &acc).await?;
    let before = account_view(&state, &acc).await?;
    check_if_match(&headers, &etag_of(&before), None, Some(acc.updated_at))?;
    let name = body.display_name.trim();
    if name.is_empty() || name.chars().count() > 128 {
        return Err(AdminApiError::invalid(
            "invalidName",
            "A display name has 1 to 128 characters.",
            "/displayName",
        ));
    }
    if body.disabled && !acc.disabled {
        admin_roles::last_super_admin_guard(&state, &acc).await?;
    }
    if body.dry_run {
        let sessions = if body.disabled && !acc.disabled {
            state
                .store
                .list_admin_sessions(now())
                .await?
                .iter()
                .filter(|s| s.admin_id == acc.username)
                .count()
        } else {
            0
        };
        return Ok(Json(json!({ "impact": { "sessionsEnded": sessions } })).into_response());
    }
    if !state
        .store
        .change_admin_account_guarded(&id, Some(name), body.disabled, None, false, now())
        .await?
    {
        return Err(last_super_error());
    }
    if body.disabled {
        invalidate_pending(&id);
        state.store.delete_admin_sessions_of(&acc.username).await?;
    }
    let after = account_view(&state, &find(&state, &id).await?).await?;
    audit(
        &state,
        &ctx,
        if body.disabled && !acc.disabled {
            "admin-disabled"
        } else {
            "admin-updated"
        },
        &format!("admin:{id}"),
        Some(before),
        Some(after.clone()),
    )
    .await;
    let tag = etag_of(&after);
    Ok(with_etag(Json(after), &tag))
}

/// `DELETE /admin/admins/{id}` — the account and its sessions.
async fn delete_admin(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(id): UrlPath<String>,
    headers: HeaderMap,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::AdminRoleChange)
        .await?;
    let acc = find(&state, &id).await?;
    require_can_manage(&state, &ctx, &acc).await?;
    admin_roles::last_super_admin_guard(&state, &acc).await?;
    let before = account_view(&state, &acc).await?;
    check_if_match(&headers, &etag_of(&before), None, Some(acc.updated_at))?;
    if !state
        .store
        .change_admin_account_guarded(&id, None, false, None, true, now())
        .await?
    {
        return Err(last_super_error());
    }
    state.store.delete_admin_sessions_of(&acc.username).await?;
    audit(
        &state,
        &ctx,
        "admin-deleted",
        &format!("admin:{id}"),
        Some(before),
        None,
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Deserialize)]
struct ResetPassword {
    password: String,
}

/// `POST /admin/admins/{id}/reset-password {password}` — sets the password and
/// ends the account's sessions.
async fn reset_password(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(id): UrlPath<String>,
    AdminJson(body): AdminJson<ResetPassword>,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::AdminRoleChange)
        .await?;
    let acc = find(&state, &id).await?;
    require_can_manage(&state, &ctx, &acc).await?;
    check_new_password(&body.password, "/password")?;
    let phc = hash_password(&state, &body.password).await?;
    state.store.set_admin_password(&id, &phc, now()).await?;
    invalidate_pending(&id);
    state.store.delete_admin_sessions_of(&acc.username).await?;
    state.store.delete_admin_sessions_of(&acc.username).await?;
    audit(
        &state,
        &ctx,
        "admin-password-reset",
        &format!("admin:{id}"),
        None,
        None,
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// `POST /admin/admins/{id}/totp` — give the account a new factor and answer its
/// secret once, for the admin to hand over.
async fn admin_totp_set(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(id): UrlPath<String>,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::TwoFactorDisable)
        .await?;
    let acc = find(&state, &id).await?;
    require_can_manage(&state, &ctx, &acc).await?;
    let secret = totp::generate_secret().to_vec();
    invalidate_pending(&id);
    state.store.put_admin_totp(&id, &secret, now()).await?;
    audit(
        &state,
        &ctx,
        "admin-totp-set",
        &format!("admin:{id}"),
        None,
        None,
    )
    .await;
    let uri = totp::provisioning_uri(&secret, TOTP_ISSUER, &acc.username, &TotpParams::default());
    Ok(Json(json!({ "secret": totp::base32_encode(&secret), "otpauthUri": uri })).into_response())
}

/// `DELETE /admin/admins/{id}/totp` — remove the account's factor.
async fn admin_totp_delete(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(id): UrlPath<String>,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::TwoFactorDisable)
        .await?;
    let acc = find(&state, &id).await?;
    require_can_manage(&state, &ctx, &acc).await?;
    invalidate_pending(&id);
    state.store.delete_admin_totp(&id).await?;
    audit(
        &state,
        &ctx,
        "admin-totp-removed",
        &format!("admin:{id}"),
        None,
        None,
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn admin_passkeys_delete(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(id): UrlPath<String>,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::TwoFactorDisable)
        .await?;
    let acc = find(&state, &id).await?;
    require_can_manage(&state, &ctx, &acc).await?;
    if !state.store.revoke_admin_passkeys(&id).await? {
        return Err(last_super_error());
    }
    invalidate_pending(&id);
    audit(
        &state,
        &ctx,
        "admin-passkeys-revoked",
        &format!("admin:{id}"),
        None,
        None,
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RolesBody {
    roles: Vec<GrantBody>,
    #[serde(default)]
    dry_run: bool,
}

/// `PUT /admin/admins/{id}/roles {roles: [{roleId, domains|null}]}` — replaces the
/// account's roles. The writer must be able to grant the roles taken away as well
/// as the ones given.
async fn set_roles(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(id): UrlPath<String>,
    headers: HeaderMap,
    AdminJson(body): AdminJson<RolesBody>,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::AdminRoleChange)
        .await?;
    let acc = find(&state, &id).await?;
    require_can_manage(&state, &ctx, &acc).await?;
    let before = account_view(&state, &acc).await?;
    check_if_match(&headers, &etag_of(&before), None, Some(acc.updated_at))?;
    let grants = check_grants(&state, &ctx, &body.roles, "/roles").await?;
    let roles = state.store.list_admin_roles().await?;
    for old in state.store.admin_account_roles(&id).await? {
        if let Some(role) = roles.iter().find(|r| r.id == old.role_id) {
            admin_roles::require_can_grant(&ctx, &admin_roles::role_capabilities(role))?;
        }
    }
    let keeps_super = grants
        .iter()
        .any(|g| g.role_id == SUPER_ADMIN && g.domains.is_none());
    if !keeps_super {
        admin_roles::last_super_admin_guard(&state, &acc).await?;
    }
    if body.dry_run {
        return Ok(Json(json!({ "impact": { "roles": grants.len() } })).into_response());
    }
    if !state
        .store
        .change_admin_account_guarded(&id, None, false, Some(&grants), false, now())
        .await?
    {
        return Err(last_super_error());
    }
    let after = account_view(&state, &acc).await?;
    audit(
        &state,
        &ctx,
        "admin-roles-changed",
        &format!("admin:{id}"),
        Some(before),
        Some(after.clone()),
    )
    .await;
    let tag = etag_of(&after);
    Ok(with_etag(Json(after), &tag))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OverrideBody {
    capability: String,
    effect: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverridesBody {
    overrides: Vec<OverrideBody>,
}

/// `PUT /admin/admins/{id}/overrides {overrides: [{capability, effect}]}`.
async fn set_overrides(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(id): UrlPath<String>,
    headers: HeaderMap,
    AdminJson(body): AdminJson<OverridesBody>,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::AdminRoleChange)
        .await?;
    let acc = find(&state, &id).await?;
    require_can_manage(&state, &ctx, &acc).await?;
    let before = account_view(&state, &acc).await?;
    check_if_match(&headers, &etag_of(&before), None, Some(acc.updated_at))?;
    let mut rows = Vec::new();
    let mut caps = Vec::new();
    for (i, o) in body.overrides.iter().enumerate() {
        let Some(cap) = Capability::parse(&o.capability) else {
            return Err(AdminApiError::invalid(
                "unknownCapability",
                format!("{} is not a capability.", o.capability),
                format!("/overrides/{i}/capability"),
            ));
        };
        if o.effect != "allow" && o.effect != "deny" {
            return Err(AdminApiError::invalid(
                "invalidEffect",
                "An override is allow or deny.",
                format!("/overrides/{i}/effect"),
            ));
        }
        caps.push(cap);
        rows.push((o.capability.clone(), o.effect.clone()));
    }
    for (old, _) in state.store.admin_overrides(&id).await? {
        if let Some(cap) = Capability::parse(&old) {
            caps.push(cap);
        }
    }
    admin_roles::require_can_grant(&ctx, &caps)?;
    // A deny can demote a super admin just as effectively as removing its role.
    if rows.iter().any(|(_, effect)| effect == "deny") {
        admin_roles::last_super_admin_guard(&state, &acc).await?;
    }
    if !state.store.set_admin_overrides_guarded(&id, &rows).await? {
        return Err(last_super_error());
    }
    let after = account_view(&state, &acc).await?;
    audit(
        &state,
        &ctx,
        "admin-overrides-changed",
        &format!("admin:{id}"),
        Some(before),
        Some(after.clone()),
    )
    .await;
    let tag = etag_of(&after);
    Ok(with_etag(Json(after), &tag))
}

// ─── admin sessions ─────────────────────────────────────────────────────────────

/// The public id of a session: a prefix of its token hash (never the token).
fn session_id(token_hash: &str) -> String {
    token_hash.chars().take(16).collect()
}

/// `GET /admin/admin-sessions` — every live admin session.
async fn list_sessions(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    let out: Vec<Value> = state
        .store
        .list_admin_sessions(now())
        .await?
        .into_iter()
        .map(|s| {
            json!({
                "id": session_id(&s.token_hash),
                "adminId": s.admin_id,
                "createdAt": s.created_at,
                "idleExpiresAt": s.expires_at,
                "expiresAt": s.absolute_expires_at,
                "current": s.token_hash == ctx.session_hash,
            })
        })
        .collect();
    Ok(Json(json!({ "sessions": out })).into_response())
}

/// `DELETE /admin/admin-sessions/{id}` — end one admin session.
async fn revoke_session(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(id): UrlPath<String>,
) -> Result<Response, AdminApiError> {
    let sessions = state.store.list_admin_sessions(now()).await?;
    let Some(s) = sessions.iter().find(|s| session_id(&s.token_hash) == id) else {
        return Err(AdminApiError::not_found("No such session."));
    };
    state.store.delete_admin_session(&s.token_hash).await?;
    audit(
        &state,
        &ctx,
        "admin-session-revoked",
        &format!("admin-session:{id}"),
        Some(json!({ "adminId": s.admin_id })),
        None,
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usernames() {
        assert!(valid_username("alice"));
        assert!(valid_username("ops.team-1@example.com"));
        assert!(!valid_username(""));
        assert!(!valid_username("Alice"));
        assert!(!valid_username("a b"));
        assert!(!valid_username(&"a".repeat(65)));
    }

    #[test]
    fn passwords() {
        assert!(check_new_password("short", "/password").is_err());
        assert!(check_new_password("twelve chars", "/password").is_ok());
        let e = check_new_password("x", "/newPassword").unwrap_err();
        assert_eq!(e.field.as_deref(), Some("/newPassword"));
    }

    #[test]
    fn session_ids_do_not_carry_the_hash() {
        let h = "0123456789abcdef0123456789abcdef";
        assert_eq!(session_id(h), "0123456789abcdef");
    }
}
