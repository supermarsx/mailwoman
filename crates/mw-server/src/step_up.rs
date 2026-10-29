//! Step-up re-authentication for sensitive actions (t32-ext §14.3 server
//! enforcement, §15 M4; e57 seam).
//!
//! A sensitive endpoint asks for a fresh proof that the person at the keyboard
//! is the account holder, not only someone holding the session: the user
//! confirms with their password plus a second factor when one is enrolled, or a
//! passkey with user verification on its own. A proof lasts [`PROOF_TTL`] (five
//! minutes) and is bound to the session that minted it: another session of the
//! same account, or the same session after sign-out, has no proof.
//!
//! * Mailbox routes take [`RequireStepUp<A>`] as an extractor (it also resolves
//!   the session, so the handler need not call `authed` again), or call
//!   [`require_user`] when only some requests need it (forwarding to an external
//!   address). Admin routes take [`RequireAdminStepUp<A>`] or call
//!   [`require_admin`]. `A` names the action ([`action`]).
//! * Without a live proof the answer is `403 {code:"step_up_required", action,
//!   methods}`; `methods` lists what this session can confirm with
//!   (`password`, `totp`, `passkey`).
//! * Proofs are minted at `POST /api/security/step-up` (mailbox) and
//!   `POST {admin}/step-up` (admin). Five wrong attempts in 15 minutes lock the
//!   session's step-up for the rest of that window (`429 stepUpLocked`). Grants,
//!   refusals and locks are security events in the audit log.
//!
//! Proofs live in this process's memory, like the session timeout clocks
//! ([`crate::hardening::SessionGuard`]); a restart drops them and the user
//! confirms again.

// The extractors and checks are what sensitive endpoints take (M4 grants); until
// every listed endpoint applies them, some of this module has no caller.
#![allow(dead_code)]

use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::{Extensions, HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine as _;
use rand::RngCore;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use mw_mfa::totp::{self, TotpParams};
use mw_mfa::webauthn::{AssertionRequest, UserVerification, verify_assertion_with_backup};

use crate::AppState;
use crate::admin_ctx::{AdminApiError, AdminCtx, AdminJson};
use crate::admin_router::{Access, AdminRegistrar};
use crate::audit::{SecurityEvent, security_event};

/// How long a proof lasts after it is minted.
pub(crate) const PROOF_TTL: Duration = Duration::from_secs(300);
/// Wrong attempts allowed per session within [`FAILURE_WINDOW`].
const MAX_FAILURES: u32 = 5;
const FAILURE_WINDOW: Duration = Duration::from_secs(900);
/// How long a passkey challenge for a step-up stays valid.
const CHALLENGE_TTL: Duration = Duration::from_secs(300);

// ─── actions ────────────────────────────────────────────────────────────────────

/// A sensitive action (t32-ext §14.3 policy list, t32-cfg §4.21).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StepUpAction {
    /// A user exports their own data.
    DataExport,
    /// A user gives someone access to their mailbox.
    DelegationGrant,
    /// A user or admin creates an API key.
    ApiKeyCreate,
    /// An admin changes roles or role grants.
    AdminRoleChange,
    /// A user turns a second factor off.
    TwoFactorDisable,
    /// A rule forwards or redirects mail to an address outside the server.
    ExternalForwarding,
    /// An admin changes branding CSS.
    BrandingCss,
    /// An admin applies an update.
    UpdatesApply,
    /// An admin imports a configuration.
    ConfigImport,
    /// Configure an integration or mutate its remote managed resources.
    IntegrationManage,
    /// An admin changes deployment-wide source IP admission.
    BanManage,
    /// An admin exports a user's data.
    AdminDataExport,
    /// An administrator changes local user access.
    UserManage,
    /// Change the global mailbox admission pause.
    MaintenanceMode,
}

impl StepUpAction {
    pub const ALL: [StepUpAction; 14] = [
        StepUpAction::DataExport,
        StepUpAction::DelegationGrant,
        StepUpAction::ApiKeyCreate,
        StepUpAction::AdminRoleChange,
        StepUpAction::TwoFactorDisable,
        StepUpAction::ExternalForwarding,
        StepUpAction::BrandingCss,
        StepUpAction::UpdatesApply,
        StepUpAction::ConfigImport,
        StepUpAction::IntegrationManage,
        StepUpAction::BanManage,
        StepUpAction::AdminDataExport,
        StepUpAction::UserManage,
        StepUpAction::MaintenanceMode,
    ];

    /// The wire id (`action` in the 403 body and the audit detail).
    pub fn as_str(self) -> &'static str {
        match self {
            StepUpAction::DataExport => "dataExport",
            StepUpAction::DelegationGrant => "delegationGrant",
            StepUpAction::ApiKeyCreate => "apiKeyCreate",
            StepUpAction::AdminRoleChange => "adminRoleChange",
            StepUpAction::TwoFactorDisable => "twoFactorDisable",
            StepUpAction::ExternalForwarding => "externalForwarding",
            StepUpAction::BrandingCss => "brandingCss",
            StepUpAction::UpdatesApply => "updatesApply",
            StepUpAction::ConfigImport => "configImport",
            StepUpAction::IntegrationManage => "integrationManage",
            StepUpAction::BanManage => "banManage",
            StepUpAction::AdminDataExport => "adminDataExport",
            StepUpAction::UserManage => "userManage",
            StepUpAction::MaintenanceMode => "maintenanceMode",
        }
    }
}

/// Names a [`StepUpAction`] at the type level, for the extractors.
pub trait Sensitive: Send + Sync + 'static {
    const ACTION: StepUpAction;
}

/// One marker type per [`StepUpAction`]: `RequireStepUp<action::DataExport>`.
pub mod action {
    use super::{Sensitive, StepUpAction};

    macro_rules! markers {
        ($($name:ident),* $(,)?) => {$(
            #[derive(Debug)]
            pub struct $name;
            impl Sensitive for $name {
                const ACTION: StepUpAction = StepUpAction::$name;
            }
        )*};
    }

    markers!(
        DataExport,
        DelegationGrant,
        ApiKeyCreate,
        AdminRoleChange,
        TwoFactorDisable,
        ExternalForwarding,
        BrandingCss,
        UpdatesApply,
        ConfigImport,
        IntegrationManage,
        BanManage,
        AdminDataExport,
    );
}

// ─── proof state ────────────────────────────────────────────────────────────────

/// Whose session a proof belongs to: the hash of a mailbox session id, or of an
/// admin session token.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Holder {
    User(String),
    Admin(String),
}

impl Holder {
    fn user(session_id: &str) -> Self {
        Holder::User(crate::push_relay::hash_token(session_id))
    }
}

#[derive(Clone, Copy, Debug)]
struct Failures {
    count: u32,
    since: Instant,
}

#[derive(Clone, Debug)]
struct Challenge {
    bytes: Vec<u8>,
    origin: String,
    rp_id: String,
    expires_at: Instant,
}

#[derive(Default)]
struct Inner {
    proofs: HashMap<Holder, Instant>,
    factor_fingerprints: HashMap<Holder, Vec<u8>>,
    failures: HashMap<Holder, Failures>,
    challenges: HashMap<Holder, Challenge>,
}

/// The live proofs, failure counts and passkey challenges (in [`AppState`]).
#[derive(Default)]
pub(crate) struct StepUpState {
    inner: Mutex<Inner>,
}

impl StepUpState {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().expect("step-up state lock")
    }

    /// Mint a proof for `h` at `now`; clears its failure count.
    fn grant_at(&self, h: &Holder, now: Instant) {
        let mut inner = self.lock();
        inner.proofs.retain(|_, exp| *exp > now);
        let live: std::collections::HashSet<_> = inner.proofs.keys().cloned().collect();
        inner
            .factor_fingerprints
            .retain(|holder, _| live.contains(holder));
        inner.proofs.insert(h.clone(), now + PROOF_TTL);
        inner.failures.remove(h);
    }

    /// Time left on `h`'s proof at `now`, if it has a live one.
    fn remaining_at(&self, h: &Holder, now: Instant) -> Option<Duration> {
        let inner = self.lock();
        inner
            .proofs
            .get(h)
            .and_then(|exp| exp.checked_duration_since(now))
            .filter(|d| !d.is_zero())
    }

    /// How long `h` stays locked out at `now`, if it is.
    fn locked_at(&self, h: &Holder, now: Instant) -> Option<Duration> {
        let inner = self.lock();
        let f = inner.failures.get(h)?;
        let end = f.since + FAILURE_WINDOW;
        (f.count >= MAX_FAILURES && end > now).then(|| end - now)
    }

    /// Count a wrong attempt; returns the attempts left in the window (0 = now
    /// locked).
    fn note_failure_at(&self, h: &Holder, now: Instant) -> u32 {
        let mut inner = self.lock();
        inner.failures.retain(|_, f| f.since + FAILURE_WINDOW > now);
        let f = inner.failures.entry(h.clone()).or_insert(Failures {
            count: 0,
            since: now,
        });
        f.count += 1;
        MAX_FAILURES.saturating_sub(f.count)
    }

    fn put_challenge(&self, h: &Holder, c: Challenge) {
        let now = Instant::now();
        let mut inner = self.lock();
        inner.challenges.retain(|_, c| c.expires_at > now);
        inner.challenges.insert(h.clone(), c);
    }

    /// Take `h`'s passkey challenge (single use).
    fn take_challenge(&self, h: &Holder) -> Option<Challenge> {
        let c = self.lock().challenges.remove(h)?;
        (c.expires_at > Instant::now()).then_some(c)
    }

    /// Drop every proof of a mailbox session (sign-out, revoke).
    pub(crate) fn forget_user(&self, session_id: &str) {
        let h = Holder::user(session_id);
        let mut inner = self.lock();
        inner.proofs.remove(&h);
        inner.factor_fingerprints.remove(&h);
        inner.challenges.remove(&h);
    }
}

// ─── checks ─────────────────────────────────────────────────────────────────────

/// The `403 step_up_required` body.
fn required_body(action: StepUpAction, methods: &[&'static str]) -> Value {
    let detail = "Confirm it is you to continue.";
    json!({
        "code": "step_up_required",
        "detail": detail,
        "error": detail,
        "action": action.as_str(),
        "methods": methods,
    })
}

/// `Ok` when `session` holds a live proof; otherwise the `403 step_up_required`
/// answer for `action`.
pub(crate) async fn require_user(
    state: &AppState,
    session: &mw_store::Session,
    action: StepUpAction,
) -> Result<(), Response> {
    // A scoped key has no interactive session and cannot inherit a browser proof.
    if session.id.is_empty() {
        return Err((StatusCode::FORBIDDEN, Json(required_body(action, &[]))).into_response());
    }
    require_known_authentication(state, session).await?;
    if user_remaining(state, session)
        .await
        .map_err(|e| server_error("step-up factors", e))?
        .is_some()
    {
        return Ok(());
    }
    let methods = user_methods(state, session)
        .await
        .map_err(|e| server_error("step-up methods", e))?;
    security_event(
        state,
        SecurityEvent::user(&session.account_id, "step-up-required")
            .detail(json!({"action":action.as_str()})),
    )
    .await;
    Err((
        StatusCode::FORBIDDEN,
        Json(required_body(action, &methods.list())),
    )
        .into_response())
}

// Bind proofs to the actual enrolled key material as well as the session. Labels
// and mutable signature counters deliberately do not invalidate a proof.
pub(crate) async fn factor_fingerprint(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<Vec<u8>, mw_store::StoreError> {
    let mut hash = Sha256::new();
    hash.update(session.account_id.as_bytes());
    hash.update(Sha256::digest(session.credentials.password.as_bytes()));
    if let Some(secret) = state.store.get_totp_secret(&session.account_id).await?
        && secret.confirmed
    {
        hash.update([1]);
        hash.update(Sha256::digest(&secret.secret));
    }
    let mut keys = state
        .store
        .list_webauthn_credentials(&session.account_id)
        .await?;
    keys.sort_by(|a, b| a.credential_id.cmp(&b.credential_id));
    for key in keys {
        hash.update(Sha256::digest(key.credential_id.as_bytes()));
        hash.update(Sha256::digest(&key.cose_public_key));
    }
    Ok(hash.finalize().to_vec())
}
async fn user_remaining(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<Option<Duration>, mw_store::StoreError> {
    let holder = Holder::user(&session.id);
    let Some(left) = state.step_up.remaining_at(&holder, Instant::now()) else {
        return Ok(None);
    };
    let fingerprint = factor_fingerprint(state, session).await?;
    let valid = state.step_up.lock().factor_fingerprints.get(&holder) == Some(&fingerprint);
    if !valid {
        state.step_up.forget_user(&session.id);
    }
    Ok(valid.then_some(left))
}

/// `Ok` when the admin's session holds a live proof; otherwise `403
/// step_up_required` for `action`.
pub(crate) async fn require_admin(
    state: &AppState,
    ctx: &AdminCtx,
    action: StepUpAction,
) -> Result<(), AdminApiError> {
    if admin_remaining(state, ctx).await?.is_some() {
        return Ok(());
    }
    security_event(
        state,
        SecurityEvent::admin(&ctx.username, "step-up-required")
            .detail(json!({"action":action.as_str()})),
    )
    .await;
    let mut e = AdminApiError::new(
        StatusCode::FORBIDDEN,
        "step_up_required",
        "Confirm it is you to continue.",
    )
    .with("action", action.as_str());
    e = e.with("methods", json!(admin_methods(state, ctx).await?));
    Err(e)
}

/// A mailbox session with a live step-up proof for `A`. Rejects with `401` when
/// there is no session (as `authed` does) and `403 step_up_required` without a
/// proof.
pub struct RequireStepUp<A: Sensitive> {
    pub session: mw_store::Session,
    _action: PhantomData<A>,
}

impl<A: Sensitive> FromRequestParts<AppState> for RequireStepUp<A> {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = crate::authed(state, &parts.headers).await?;
        require_user(state, &session, A::ACTION).await?;
        Ok(Self {
            session,
            _action: PhantomData,
        })
    }
}

/// An admin (resolved by the registrar) whose session holds a live step-up proof
/// for `A`. Only on routes mounted through `admin_router`.
pub struct RequireAdminStepUp<A: Sensitive> {
    pub ctx: AdminCtx,
    _action: PhantomData<A>,
}

impl<A: Sensitive> FromRequestParts<AppState> for RequireAdminStepUp<A> {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let ctx = AdminCtx::from_request_parts(parts, state).await?;
        require_admin(state, &ctx, A::ACTION)
            .await
            .map_err(IntoResponse::into_response)?;
        Ok(Self {
            ctx,
            _action: PhantomData,
        })
    }
}

// ─── what a session can confirm with ────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Default)]
struct Methods {
    password: bool,
    totp: bool,
    passkey: bool,
}

impl Methods {
    fn has_factor(self) -> bool {
        self.totp || self.passkey
    }

    fn list(self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.password {
            out.push("password");
        }
        if self.totp {
            out.push("totp");
        }
        if self.passkey {
            out.push("passkey");
        }
        out
    }
}

/// Whether the session was opened with a password the user knows, so the
/// password can confirm them again. Not for single sign-on sessions (no password
/// is stored) or trusted-proxy header sessions (their stored password is the
/// deployment's service password, `MW_HEADER_AUTH_PASSWORD`, not the user's).
pub(crate) fn password_capable(session: &mw_store::Session) -> bool {
    let pw = &session.credentials.password;
    if pw.is_empty() || session.jmap_url == "sso" {
        return false;
    }
    if crate::env_flag("MW_HEADER_AUTH")
        && std::env::var("MW_HEADER_AUTH_PASSWORD").is_ok_and(|svc| ct_eq(&svc, pw))
    {
        return false;
    }
    true
}

pub(crate) async fn require_known_authentication(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<(), Response> {
    let method = state
        .store
        .session_authentication(&session.id)
        .await
        .map_err(|e| server_error("authentication method", e))?;
    if !matches!(
        method.as_str(),
        "password" | "password+factor" | "remembered-device" | "passkey" | "header-auth" | "sso"
    ) {
        return Err((StatusCode::UNAUTHORIZED, Json(json!({"code":"reauthenticationRequired","error":"Sign in again to verify this session."}))).into_response());
    }
    Ok(())
}

pub(crate) async fn session_password_capable(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<bool, mw_store::StoreError> {
    let method = state.store.session_authentication(&session.id).await?;
    Ok(matches!(
        method.as_str(),
        "password" | "password+factor" | "remembered-device" | "passkey"
    ) && password_capable(session))
}

async fn user_methods(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<Methods, mw_store::StoreError> {
    let totp = state
        .store
        .get_totp_secret(&session.account_id)
        .await?
        .is_some_and(|s| s.confirmed);
    let passkey = !state
        .store
        .list_webauthn_credentials(&session.account_id)
        .await?
        .is_empty();
    Ok(Methods {
        password: session_password_capable(state, session).await?,
        totp,
        passkey,
    })
}

struct AdminFactors {
    hash: Option<String>,
    env_password: Option<String>,
    totp: Option<Vec<u8>>,
    account_id: Option<String>,
    fingerprint: Vec<u8>,
    passkeys: bool,
}
async fn admin_factors(
    state: &AppState,
    ctx: &AdminCtx,
) -> Result<Option<AdminFactors>, mw_store::StoreError> {
    let row = state
        .store
        .get_admin_account_by_username(&ctx.username)
        .await?;
    if row
        .as_ref()
        .is_some_and(|r| r.disabled || ctx.account_id.as_ref().is_some_and(|id| id != &r.id))
    {
        return Ok(None);
    }
    let account_id = row.as_ref().map(|r| r.id.clone());
    let hash = row.and_then(|r| r.password_hash);
    let env_password = if hash.is_none() && is_env_admin(state, &ctx.username) {
        state.v6.admin_password.clone()
    } else {
        None
    };
    if hash.is_none() && env_password.as_ref().is_none_or(String::is_empty) {
        return Ok(None);
    }
    let totp = match account_id.as_deref() {
        Some(id) => state.store.get_admin_totp(id).await?,
        None => None,
    };
    let mut digest = Sha256::new();
    digest.update(Sha256::digest(
        hash.as_deref().unwrap_or_default().as_bytes(),
    ));
    digest.update(Sha256::digest(
        env_password.as_deref().unwrap_or_default().as_bytes(),
    ));
    digest.update(Sha256::digest(totp.as_deref().unwrap_or_default()));
    let passkeys = match account_id.as_deref() {
        Some(id) => state.store.list_admin_passkeys(id).await?,
        None => Vec::new(),
    };
    for key in &passkeys {
        digest.update(Sha256::digest(key.credential.credential_id.as_bytes()));
        digest.update(Sha256::digest(&key.credential.cose_public_key));
    }
    Ok(Some(AdminFactors {
        hash,
        env_password,
        totp,
        account_id,
        fingerprint: digest.finalize().to_vec(),
        passkeys: !passkeys.is_empty(),
    }))
}
async fn admin_methods(
    state: &AppState,
    ctx: &AdminCtx,
) -> Result<Vec<&'static str>, mw_store::StoreError> {
    let Some(factors) = admin_factors(state, ctx).await? else {
        return Ok(Vec::new());
    };
    let recovery_available = if (factors.passkeys || factors.totp.is_some())
        && let Some(id) = factors.account_id.as_deref()
    {
        !state.store.admin_recovery_hashes(id).await?.is_empty()
    } else {
        false
    };
    let mut methods = Vec::new();
    if !factors.passkeys || factors.totp.is_some() || recovery_available {
        methods.push("password");
    }
    if factors.totp.is_some() {
        methods.push("totp");
    }
    if factors.passkeys {
        methods.push("passkey");
    }
    if recovery_available {
        methods.push("recovery");
    }
    Ok(methods)
}
async fn admin_remaining(
    state: &AppState,
    ctx: &AdminCtx,
) -> Result<Option<Duration>, mw_store::StoreError> {
    if ctx.session_hash.is_empty() {
        return Ok(None);
    }
    let holder = Holder::Admin(ctx.session_hash.clone());
    let Some(left) = state.step_up.remaining_at(&holder, Instant::now()) else {
        return Ok(None);
    };
    let factors = admin_factors(state, ctx).await?;
    let mut inner = state.step_up.lock();
    if factors.is_some_and(|f| inner.factor_fingerprints.get(&holder) == Some(&f.fingerprint)) {
        return Ok(Some(left));
    }
    inner.proofs.remove(&holder);
    inner.factor_fingerprints.remove(&holder);
    Ok(None)
}

fn is_env_admin(state: &AppState, username: &str) -> bool {
    state
        .v6
        .admin_username
        .as_deref()
        .is_some_and(|u| ct_eq(u, username))
}

/// Equality without an early exit on the first differing byte or on length.
fn ct_eq(a: &str, b: &str) -> bool {
    let (x, y) = (Sha256::digest(a.as_bytes()), Sha256::digest(b.as_bytes()));
    x.iter()
        .zip(y.iter())
        .fold(0u8, |acc, (p, q)| acc | (p ^ q))
        == 0
}

// ─── mailbox routes ─────────────────────────────────────────────────────────────

/// `/api/security/step-up` (merged by `router()` in `lib.rs`).
pub(crate) fn user_router() -> Router<AppState> {
    let r = Router::new()
        .route("/api/security/step-up", get(user_status).post(user_mint))
        .route("/api/security/step-up/passkey", post(user_passkey_begin));
    #[cfg(test)]
    let r = r.route("/api/security/step-up/probe", post(http_tests::user_probe));
    r
}

/// `GET /api/security/step-up` — whether this session holds a proof, for how
/// long, and what it can confirm with.
async fn user_status(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let methods = match user_methods(&state, &session).await {
        Ok(m) => m,
        Err(e) => return server_error("step-up methods", e),
    };
    let left = match user_remaining(&state, &session).await {
        Ok(left) => left,
        Err(e) => return server_error("step-up factors", e),
    };
    Json(json!({
        "active": left.is_some(),
        "expiresIn": left.map_or(0, |d| d.as_secs()),
        "methods": methods.list(),
    }))
    .into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PasskeyAssertion {
    credential_id: String,
    client_data_json: String,
    authenticator_data: String,
    signature: String,
    #[serde(default)]
    user_handle: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserMintReq {
    #[serde(default)]
    password: Option<String>,
    /// A code from the authenticator app.
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    passkey: Option<PasskeyAssertion>,
}

/// `POST /api/security/step-up/passkey` — a single-use challenge for
/// `navigator.credentials.get`, bound to this session.
async fn user_passkey_begin(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: Extensions,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let creds = match state
        .store
        .list_webauthn_credentials(&session.account_id)
        .await
    {
        Ok(c) => c,
        Err(e) => return server_error("step-up passkeys", e),
    };
    if creds.is_empty() {
        return refusal(
            StatusCode::CONFLICT,
            "noPasskey",
            "This account has no passkey.",
        );
    }
    let Some((origin, rp_id)) = webauthn_rp(&state.external, &headers, &ext) else {
        return refusal(
            StatusCode::SERVICE_UNAVAILABLE,
            "stepUpUnavailable",
            "Configure a valid public WebAuthn origin.",
        );
    };
    let mut bytes = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    state.step_up.put_challenge(
        &Holder::user(&session.id),
        Challenge {
            bytes: bytes.clone(),
            origin,
            rp_id: rp_id.clone(),
            expires_at: Instant::now() + CHALLENGE_TTL,
        },
    );
    Json(json!({
        "challenge": b64(&bytes),
        "rpId": rp_id,
        "credentialIds": creds.iter().map(|c| c.credential_id.clone()).collect::<Vec<_>>(),
        "userVerification": crate::security_center::policy(&state, &session.account_id, &session.username)
            .await
            .map_or("preferred", |p| p.passkey.assertion_uv_hint()),
    }))
    .into_response()
}

/// What a submission was checked against.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    Granted(&'static str),
    /// Not enough was sent for this session; nothing was checked.
    Incomplete,
    Failed,
    /// A passkey whose signature verified but which the passkey policy refuses
    /// (t39-e6).
    PolicyRefused(crate::twofa_routes::passkey_policy::Refusal),
}

/// `POST /api/security/step-up` — confirm with `{password, code}` or a passkey
/// assertion (`{passkey}`, with or without the password).
///
/// What counts:
/// * a passkey assertion with the user-verified flag, on its own;
/// * the password, plus a code or a passkey assertion when the account has a
///   second factor;
/// * for a session without a usable password (single sign-on, trusted-proxy
///   header): a code or a passkey assertion.
///
/// A submission that is short of these is refused before anything is checked
/// (`422 stepUpIncomplete`), so the answer never says whether a password alone
/// was right.
async fn user_mint(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: Extensions,
    body: Result<Json<UserMintReq>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if let Err(response) = require_known_authentication(&state, &session).await {
        return response;
    }
    let Ok(Json(body)) = body else {
        return refusal(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalidBody",
            "Send your password, a code, or a passkey.",
        );
    };
    let h = Holder::user(&session.id);
    let ip = crate::scope_mw::proxy::client_ip(&headers, &ext);
    let event = |action: &'static str| {
        SecurityEvent::user(&session.account_id, action)
            .ip(ip)
            .request(&headers)
    };
    if let Some(left) = state.step_up.locked_at(&h, Instant::now()) {
        return locked(left);
    }
    let methods = match user_methods(&state, &session).await {
        Ok(m) => m,
        Err(e) => return server_error("step-up methods", e),
    };
    if !methods.password && !methods.has_factor() {
        return refusal(
            StatusCode::FORBIDDEN,
            "stepUpUnavailable",
            "This session cannot be confirmed: it was not opened with a password and the \
             account has no second factor. Set up an authenticator app or a passkey first.",
        );
    }
    let fingerprint = match factor_fingerprint(&state, &session).await {
        Ok(value) => value,
        Err(e) => return server_error("step-up factors", e),
    };
    let verdict = match check_user(&state, &session, methods, &body).await {
        Ok(v) => v,
        Err(e) => return server_error("step-up check", e),
    };
    match verdict {
        Verdict::Granted(method) => {
            match factor_fingerprint(&state, &session).await {
                Ok(current) if current == fingerprint => {}
                Ok(_) => {
                    return refusal(
                        StatusCode::CONFLICT,
                        "stepUpFactorsChanged",
                        "Your authentication factors changed. Confirm again.",
                    );
                }
                Err(e) => return server_error("step-up factors", e),
            }
            state.step_up.grant_at(&h, Instant::now());
            state
                .step_up
                .lock()
                .factor_fingerprints
                .insert(h.clone(), fingerprint);
            security_event(
                &state,
                event("step-up-granted").detail(json!({ "method": method })),
            )
            .await;
            Json(json!({ "ok": true, "expiresIn": PROOF_TTL.as_secs() })).into_response()
        }
        Verdict::Incomplete => {
            let detail = if methods.password {
                "Send your password and a code from your authenticator app, or use your passkey."
            } else {
                "Send a code from your authenticator app, or use your passkey."
            };
            let mut body = refusal_body("stepUpIncomplete", detail);
            body["methods"] = json!(methods.list());
            (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
        }
        Verdict::Failed => {
            let left = state.step_up.note_failure_at(&h, Instant::now());
            security_event(
                &state,
                event(if left == 0 {
                    "step-up-locked"
                } else {
                    "step-up-refused"
                }),
            )
            .await;
            if left == 0 {
                return locked(FAILURE_WINDOW);
            }
            let mut body = refusal_body("stepUpFailed", "That did not match. Try again.");
            body["attemptsLeft"] = json!(left);
            (StatusCode::FORBIDDEN, Json(body)).into_response()
        }
        Verdict::PolicyRefused(policy_refusal) => {
            let left = state.step_up.note_failure_at(&h, Instant::now());
            security_event(
                &state,
                event("step-up-refused").detail(json!({ "reason": policy_refusal.code() })),
            )
            .await;
            crate::twofa_routes::policy_audit(&state, &session.username, "step-up", policy_refusal)
                .await;
            if left == 0 {
                return locked(FAILURE_WINDOW);
            }
            let mut body = refusal_body(policy_refusal.code(), policy_refusal.message());
            body["attemptsLeft"] = json!(left);
            (StatusCode::FORBIDDEN, Json(body)).into_response()
        }
    }
}

async fn check_user(
    state: &AppState,
    session: &mw_store::Session,
    methods: Methods,
    body: &UserMintReq,
) -> Result<Verdict, mw_store::StoreError> {
    let password = body
        .password
        .as_deref()
        .filter(|p| methods.password && !p.is_empty());
    let code = body
        .code
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty());
    let passkey = body.passkey.as_ref();
    let h = Holder::user(&session.id);

    // What was sent must be enough for this session before anything is checked.
    let enough = match (password, code.is_some(), passkey.is_some()) {
        (None, _, true) => true,
        (Some(_), has_code, has_passkey) => !methods.has_factor() || has_code || has_passkey,
        (None, true, false) => !methods.password,
        (None, false, false) => false,
    };
    if !enough {
        return Ok(Verdict::Incomplete);
    }

    let mut used = Vec::new();
    if let Some(pw) = password {
        if !ct_eq(pw, &session.credentials.password) {
            return Ok(Verdict::Failed);
        }
        used.push("password");
    }
    if let Some(a) = passkey {
        // On its own a passkey must have verified the user (PIN, biometric).
        let uv = if password.is_some() {
            UserVerification::Preferred
        } else {
            UserVerification::Required
        };
        let Some(challenge) = state.step_up.take_challenge(&h) else {
            return Ok(Verdict::Failed);
        };
        match verify_passkey(state, session, &challenge, a, uv).await? {
            Ok(true) => {}
            Ok(false) => return Ok(Verdict::Failed),
            Err(refusal) => return Ok(Verdict::PolicyRefused(refusal)),
        }
        used.push("passkey");
    } else if let Some(code) = code {
        if !methods.totp || !verify_totp(state, &session.account_id, code).await? {
            return Ok(Verdict::Failed);
        }
        used.push("totp");
    }
    Ok(Verdict::Granted(match used.as_slice() {
        ["password"] => "password",
        ["password", "totp"] => "password+totp",
        ["password", "passkey"] => "password+passkey",
        ["passkey"] => "passkey",
        ["totp"] => "totp",
        _ => return Ok(Verdict::Failed),
    }))
}

/// A TOTP code, with the login path's replay guard: a step a sign-in or an
/// earlier step-up used is refused.
async fn verify_totp(
    state: &AppState,
    account_id: &str,
    code: &str,
) -> Result<bool, mw_store::StoreError> {
    let Some(secret) = state.store.get_totp_secret(account_id).await? else {
        return Ok(false);
    };
    if !secret.confirmed {
        return Ok(false);
    }
    let Some(step) = totp::totp_verify(&secret.secret, code, now_unix(), &TotpParams::default())
    else {
        return Ok(false);
    };
    let step = i64::try_from(step).unwrap_or(i64::MAX);
    state.store.advance_totp_last_step(account_id, step).await
}

async fn verify_passkey(
    state: &AppState,
    session: &mw_store::Session,
    challenge: &Challenge,
    a: &PasskeyAssertion,
    uv: UserVerification,
) -> Result<Result<bool, crate::twofa_routes::passkey_policy::Refusal>, mw_store::StoreError> {
    let account_id = session.account_id.as_str();
    let (Some(cdj), Some(auth_data), Some(sig)) = (
        decode_b64(&a.client_data_json),
        decode_b64(&a.authenticator_data),
        decode_b64(&a.signature),
    ) else {
        return Ok(Ok(false));
    };
    let Some(cred) = state
        .store
        .get_webauthn_credential(&a.credential_id)
        .await?
    else {
        return Ok(Ok(false));
    };
    if cred.account_id != account_id {
        return Ok(Ok(false));
    }
    if !crate::twofa_routes::passkey_handle_matches(account_id, a.user_handle.as_deref(), false) {
        return Ok(Ok(false));
    }
    let Some(previous) = state.store.webauthn_state(&a.credential_id).await? else {
        return Ok(Ok(false));
    };
    let req = AssertionRequest {
        challenge: challenge.bytes.clone(),
        origin: challenge.origin.clone(),
        rp_id: challenge.rp_id.clone(),
        client_data_json: cdj,
        authenticator_data: auth_data,
        signature: sig,
        cose_public_key: cred.cose_public_key.clone(),
        stored_sign_count: cred.sign_count.max(0) as u32,
        user_verification: uv,
    };
    let outcome = match verify_assertion_with_backup(&req, previous.backup_eligible) {
        Ok(outcome) => outcome,
        Err(_) => return Ok(Ok(false)),
    };
    // t39-e6: the passkey policy, read only. Checked after the signature, so a
    // refusal is only named to someone holding the credential.
    let Ok(policy) =
        crate::security_center::policy(state, account_id, &session.username).await
    else {
        return Ok(Ok(false));
    };
    let facts = crate::twofa_routes::stored_facts(state, &cred, &previous).await?;
    if let Err(refusal) = crate::twofa_routes::assertion_meets_policy(
        &policy.passkey,
        facts.as_stored(&cred),
        &outcome,
    ) {
        return Ok(Err(refusal));
    }
    crate::twofa_routes::commit_passkey_assertion(state, &cred, &previous, &outcome)
        .await
        .map(Ok)
}

// ─── admin routes ───────────────────────────────────────────────────────────────

/// `GET`/`POST {admin}/step-up` (one call line in `admin_router::endpoints`).
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/step-up", Access::SignedIn, admin_status);
    r.admin_endpoint(Method::POST, "/step-up", Access::SignedIn, admin_mint);
    #[cfg(test)]
    r.admin_endpoint(
        Method::POST,
        "/step-up/probe",
        Access::SignedIn,
        http_tests::admin_probe,
    );
}

async fn admin_status(State(state): State<AppState>, ctx: AdminCtx) -> Response {
    let left = match admin_remaining(&state, &ctx).await {
        Ok(left) => left,
        Err(e) => return server_error("admin proof", e),
    };
    let methods = match admin_methods(&state, &ctx).await {
        Ok(methods) => methods,
        Err(e) => return server_error("admin factors", e),
    };
    Json(json!({
        "active": left.is_some(),
        "expiresIn": left.map_or(0, |d| d.as_secs()),
        "methods": methods,
    }))
    .into_response()
}

#[derive(Deserialize)]
struct AdminMintReq {
    #[serde(default)]
    password: String,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    passkey: Option<crate::admin_passkeys::Assertion>,
    #[serde(default, rename = "recoveryCode")]
    recovery_code: Option<String>,
}

/// `POST {admin}/step-up` — the admin confirms with their password.
async fn admin_mint(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    ext: Extensions,
    AdminJson(body): AdminJson<AdminMintReq>,
) -> Result<Response, AdminApiError> {
    if ctx.session_hash.is_empty() {
        return Err(AdminApiError::unauthenticated());
    }
    let h = Holder::Admin(ctx.session_hash.clone());
    let ip = crate::scope_mw::proxy::client_ip(&headers, &ext);
    let event = |action: &'static str| {
        SecurityEvent::admin(&ctx.username, action)
            .target(format!("admin:{}", ctx.username))
            .ip(ip)
            .request(&headers)
    };
    if let Some(left) = state.step_up.locked_at(&h, Instant::now()) {
        return Err(AdminApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "stepUpLocked",
            "Too many attempts. Try again later.",
        )
        .with("retryAfter", left.as_secs().max(1)));
    }
    let Some(factors) = admin_factors(&state, &ctx).await? else {
        return Err(AdminApiError::new(
            StatusCode::FORBIDDEN,
            "stepUpUnavailable",
            "This administrator has no available confirmation method.",
        ));
    };
    if body.passkey.is_none()
        && body.recovery_code.is_none()
        && (factors.passkeys && factors.totp.is_none()
            || factors.totp.is_some() && body.code.as_deref().is_none_or(|v| v.trim().is_empty()))
    {
        return Err(AdminApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "stepUpIncomplete",
            "Use your passkey, or your password with an authenticator or recovery code.",
        ));
    }
    let mut valid = if let Some(proof) = &body.passkey {
        crate::admin_passkeys::verify_stepup(&state, &ctx, proof).await?
    } else if let Some(hash) = factors.hash.clone() {
        let password = body.password.clone();
        tokio::task::spawn_blocking(move || {
            mw_admin::password::verify_password(&password, &hash).unwrap_or(false)
        })
        .await
        .unwrap_or(false)
    } else {
        factors
            .env_password
            .as_ref()
            .is_some_and(|p| !p.is_empty() && ct_eq(&body.password, p))
    };
    if valid
        && body.passkey.is_none()
        && let Some(code) = &body.recovery_code
    {
        valid = false;
        if let Some(id) = factors.account_id.as_deref() {
            for hash in state.store.admin_recovery_hashes(id).await? {
                if mw_mfa::recovery::verify_code(code, &hash)
                    && state.store.consume_admin_recovery(id, &hash).await?
                {
                    valid = true;
                    break;
                }
            }
        }
    } else if valid
        && body.passkey.is_none()
        && let Some(secret) = &factors.totp
    {
        valid = match totp::totp_verify(
            secret,
            body.code.as_deref().unwrap_or_default().trim(),
            now_unix(),
            &TotpParams::default(),
        ) {
            Some(step) => {
                state
                    .store
                    .advance_admin_totp_step(
                        factors.account_id.as_deref().unwrap_or_default(),
                        i64::try_from(step).unwrap_or(i64::MAX),
                    )
                    .await?
            }
            None => false,
        };
    }
    if valid {
        if admin_factors(&state, &ctx)
            .await?
            .is_none_or(|current| current.fingerprint != factors.fingerprint)
        {
            return Err(AdminApiError::new(
                StatusCode::CONFLICT,
                "stepUpFactorsChanged",
                "Authentication factors changed. Confirm again.",
            ));
        }
        state.step_up.grant_at(&h, Instant::now());
        state
            .step_up
            .lock()
            .factor_fingerprints
            .insert(h.clone(), factors.fingerprint);
        security_event(
            &state,
            event("step-up-granted").detail(json!({ "method": if body.passkey.is_some(){"passkey"}else if body.recovery_code.is_some(){"password+recovery"}else if factors.totp.is_some() { "password+totp" } else { "password" } })),
        )
        .await;
        return Ok(Json(json!({ "ok": true, "expiresIn": PROOF_TTL.as_secs() })).into_response());
    }
    let left = state.step_up.note_failure_at(&h, Instant::now());
    security_event(
        &state,
        event(if left == 0 {
            "step-up-locked"
        } else {
            "step-up-refused"
        }),
    )
    .await;
    if left == 0 {
        return Err(AdminApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "stepUpLocked",
            "Too many attempts. Try again later.",
        )
        .with("retryAfter", FAILURE_WINDOW.as_secs()));
    }
    Err(AdminApiError::new(
        StatusCode::FORBIDDEN,
        "stepUpFailed",
        "That did not match. Try again.",
    )
    .with("attemptsLeft", left))
}

// ─── helpers ────────────────────────────────────────────────────────────────────

fn refusal_body(code: &str, detail: &str) -> Value {
    json!({ "code": code, "detail": detail, "error": detail })
}

fn refusal(status: StatusCode, code: &str, detail: &str) -> Response {
    (status, Json(refusal_body(code, detail))).into_response()
}

fn locked(left: Duration) -> Response {
    let mut body = refusal_body("stepUpLocked", "Too many attempts. Try again later.");
    body["retryAfter"] = json!(left.as_secs().max(1));
    (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response()
}

fn server_error(what: &str, e: mw_store::StoreError) -> Response {
    tracing::error!("{what} failed: {e}");
    (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response()
}

/// Use the canonical deployment origin and trust forwarded headers only through
/// ExternalBase's configured proxy boundary. Parse authorities rather than split
/// on ':' so IPv6 and explicit ports remain correct.
pub(crate) fn webauthn_rp(
    external: &crate::ExternalBase,
    headers: &HeaderMap,
    ext: &Extensions,
) -> Option<(String, String)> {
    let origin = std::env::var("MW_WEBAUTHN_ORIGIN")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| external.base_url(headers, ext))?;
    let rp = std::env::var("MW_WEBAUTHN_RP_ID")
        .ok()
        .filter(|v| !v.trim().is_empty());
    parse_webauthn_origin(&origin, rp)
}
fn parse_webauthn_origin(origin: &str, configured_rp: Option<String>) -> Option<(String, String)> {
    let url = reqwest::Url::parse(origin).ok()?;
    if !matches!(url.scheme(), "https" | "http")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return None;
    }
    let host = url.host_str()?;
    let rp = configured_rp.unwrap_or_else(|| host.to_string());
    if host != rp && !host.ends_with(&format!(".{rp}")) {
        return None;
    }
    Some((url.origin().ascii_serialization(), rp))
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn decode_b64(s: &str) -> Option<Vec<u8>> {
    use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
    URL_SAFE_NO_PAD
        .decode(s)
        .or_else(|_| URL_SAFE.decode(s))
        .or_else(|_| STANDARD_NO_PAD.decode(s))
        .or_else(|_| STANDARD.decode(s))
        .ok()
}

#[cfg(test)]
#[path = "step_up_tests.rs"]
mod http_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn origin_uses_configured_authority_and_ignores_untrusted_forwarding() {
        let mut external = crate::ExternalBase::from_env();
        external.public_host = Some("mail.example:8443".into());
        external.public_https = Some(true);
        external.trusted_proxies = crate::scope_mw::proxy::CidrSet::parse("");
        external.self_terminated_tls = false;
        let mut headers = HeaderMap::new();
        headers.insert("host", "attacker.example".parse().unwrap());
        headers.insert("x-forwarded-host", "spoof.example".parse().unwrap());
        headers.insert("x-forwarded-proto", "http".parse().unwrap());
        let origin = external.base_url(&headers, &Extensions::new()).unwrap();
        assert_eq!(
            parse_webauthn_origin(&origin, None),
            Some(("https://mail.example:8443".into(), "mail.example".into()))
        );
        external.public_host = None;
        external.public_https = None;
        assert_eq!(
            external.base_url(&headers, &Extensions::new()).as_deref(),
            Some("http://attacker.example")
        );
    }
    #[test]
    fn origin_rejects_credentials_paths_and_unrelated_rp() {
        for origin in [
            "https://user:password@mail.example",
            "https://mail.example/path",
            "https://mail.example/?next=evil",
            "javascript:bad",
        ] {
            assert!(parse_webauthn_origin(origin, None).is_none());
        }
        assert!(
            parse_webauthn_origin("https://mail.example", Some("attacker.example".into()))
                .is_none()
        );
        assert_eq!(
            parse_webauthn_origin("https://mail.example:8443", Some("example".into()))
                .unwrap()
                .0,
            "https://mail.example:8443"
        );
    }

    #[test]
    fn proof_expires_after_five_minutes() {
        let s = StepUpState::new();
        let h = Holder::user("session-a");
        let t0 = Instant::now();
        assert!(s.remaining_at(&h, t0).is_none(), "no proof before minting");
        s.grant_at(&h, t0);
        assert!(s.remaining_at(&h, t0 + Duration::from_secs(299)).is_some());
        assert!(
            s.remaining_at(&h, t0 + PROOF_TTL).is_none(),
            "a proof is gone at five minutes"
        );
    }

    #[test]
    fn proof_is_bound_to_its_session() {
        let s = StepUpState::new();
        let t0 = Instant::now();
        s.grant_at(&Holder::user("session-a"), t0);
        assert!(s.remaining_at(&Holder::user("session-b"), t0).is_none());
        // An admin token with the same hash input is a different holder.
        let hash = crate::push_relay::hash_token("session-a");
        assert!(s.remaining_at(&Holder::Admin(hash), t0).is_none());
        s.forget_user("session-a");
        assert!(s.remaining_at(&Holder::user("session-a"), t0).is_none());
    }

    #[test]
    fn five_failures_lock_for_the_window() {
        let s = StepUpState::new();
        let h = Holder::user("session-a");
        let t0 = Instant::now();
        for left in (0..MAX_FAILURES).rev() {
            assert!(s.locked_at(&h, t0).is_none());
            assert_eq!(s.note_failure_at(&h, t0), left);
        }
        assert!(s.locked_at(&h, t0 + Duration::from_secs(60)).is_some());
        assert!(s.locked_at(&h, t0 + FAILURE_WINDOW).is_none());
        // A success clears the count.
        let h2 = Holder::user("session-b");
        s.note_failure_at(&h2, t0);
        s.grant_at(&h2, t0);
        assert_eq!(s.note_failure_at(&h2, t0), MAX_FAILURES - 1);
    }

    #[test]
    fn challenge_is_single_use() {
        let s = StepUpState::new();
        let h = Holder::user("session-a");
        s.put_challenge(
            &h,
            Challenge {
                bytes: vec![1; 32],
                origin: "http://x".into(),
                rp_id: "x".into(),
                expires_at: Instant::now() + CHALLENGE_TTL,
            },
        );
        assert!(s.take_challenge(&h).is_some());
        assert!(s.take_challenge(&h).is_none());
    }

    #[test]
    fn action_ids_are_distinct() {
        let ids: BTreeSet<_> = StepUpAction::ALL.iter().map(|a| a.as_str()).collect();
        assert_eq!(ids.len(), StepUpAction::ALL.len());
        assert_eq!(
            <action::TwoFactorDisable as Sensitive>::ACTION,
            StepUpAction::TwoFactorDisable
        );
    }

    #[test]
    fn single_sign_on_and_header_sessions_have_no_password() {
        let s = |jmap: &str, pw: &str| mw_store::Session {
            id: "s".into(),
            account_id: "a".into(),
            username: "a".into(),
            jmap_url: jmap.into(),
            api_url: jmap.into(),
            credentials: mw_store::Credentials {
                username: "a".into(),
                password: pw.into(),
            },
        };
        assert!(password_capable(&s("https://jmap.example", "pw")));
        assert!(!password_capable(&s("sso", "")));
        assert!(!password_capable(&s("engine", "")));
    }

    #[test]
    fn ct_eq_compares_whole_values() {
        assert!(ct_eq("hunter2", "hunter2"));
        assert!(!ct_eq("hunter2", "hunter3"));
        assert!(!ct_eq("hunter2", "hunter22"));
        assert!(!ct_eq("", "x"));
    }
}
