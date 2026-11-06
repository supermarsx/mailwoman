//! Login second-factor + session-management routes (t16 e3, SPEC §7.4/§19).
//!
//! This module wires the pure `mw-mfa` primitives (WebAuthn RP verify, RFC 6238
//! TOTP, Argon2id recovery codes) and the sealed `mw-store` 0015 tables into the
//! HTTP surface, and inserts the second-factor gate into the login flow.
//!
//! ## Login gate (DQ2 — no downgrade)
//! [`gate_login`] runs AFTER the password/credential check in both the proxy
//! ([`crate::login`]) and engine ([`crate::engine_login`]) branches, BEFORE any
//! session is created. Its policy:
//!   * a user with ANY confirmed factor (TOTP or a passkey) MUST clear a second
//!     factor — there is no password-only path back (no silent downgrade);
//!   * an admin `twofa_policy` (global or per-domain) can REQUIRE a factor; a
//!     required-but-unenrolled user is forced into enrolment before the session is
//!     issued;
//!   * recovery codes are the break-glass path;
//!   * an opt-in user with no factor and no policy proceeds exactly as before.
//!
//! ## Single-use challenge
//! The WebAuthn challenge and the validated credentials live in a server-side
//! [`PendingLogin`] keyed by an unguessable one-shot token. A successful assertion
//! REMOVES the entry, so the challenge cannot be replayed; entries also expire.
//! `mw-mfa` compares the challenge constant-time.
//!
//! ## What is NOT here
//! The enrolment/verification/recovery *browser UI* is `apps/web` (t16 e15); this
//! module is the server contract it drives. WebAuthn origin/rp-id are derived from
//! the configured public origin and trusted-proxy boundary; explicit
//! `MW_WEBAUTHN_ORIGIN`/`MW_WEBAUTHN_RP_ID` overrides remain supported.

#[path = "passkey_policy.rs"]
pub(crate) mod passkey_policy;

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine as _;
use rand::RngCore;
use serde::Deserialize;
use serde_json::json;

use crate::admin_ctx::AdminCtx;
use mw_mfa::totp::TotpParams;
use mw_mfa::webauthn::{
    AssertionRequest, RegistrationRequest, UserVerification, verify_assertion_with_backup,
    verify_registration,
};
use mw_mfa::{recovery, totp};
use mw_store::{Credentials, TwofaPolicyRow, WebauthnCredentialRow};

use crate::{AppState, account_gate};

/// How long a pending login (awaiting its second factor / forced enrolment) or a
/// registration challenge stays valid.
const PENDING_TTL: Duration = Duration::from_secs(300);
/// Max second-factor attempts against one pending login before it is invalidated
/// (brute-force cap; a 6-digit TOTP with a ±1 window admits 3 live codes).
#[cfg(test)]
const MAX_ATTEMPTS: u32 = 5;
/// Bytes of entropy in a pending-login / challenge token.
const TOKEN_BYTES: usize = 32;
/// WebAuthn user-verification policy (DQ2: "preferred").
const UV: UserVerification = UserVerification::Preferred;
/// The `otpauth://` issuer label shown in authenticator apps.
const TOTP_ISSUER: &str = "Mailwoman";

// ─────────────────────────────────────────────────────────────────────────────
// In-memory pending state
// ─────────────────────────────────────────────────────────────────────────────

/// What the pending login is waiting for.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PendingKind {
    /// The user has a confirmed factor and must present it.
    Verify,
    /// Policy requires a factor the user has not enrolled — force enrolment first.
    Enroll,
}

/// A credential-validated login held server-side until its second factor (or forced
/// enrolment) completes. Holds exactly the arguments [`complete_login`] needs, so
/// no credential is re-validated on the second step.
#[derive(Clone)]
struct PendingLogin {
    kind: PendingKind,
    args: SessionArgs,
    device: mw_store::SessionDevice,
    max_attempts: u32,
    /// Raw single-use WebAuthn challenge for this attempt.
    challenge: Vec<u8>,
    origin: String,
    rp_id: String,
    attempts: u32,
    expires_at: Instant,
}

/// A registration challenge issued to an authenticated (or forced-enrolling) user
/// for a passkey `navigator.credentials.create` ceremony.
#[derive(Clone)]
struct RegChallenge {
    discoverable: bool,
    challenge: Vec<u8>,
    origin: String,
    rp_id: String,
    expires_at: Instant,
}

/// The server-side second-factor state carried in [`AppState`]. Two one-shot,
/// TTL-swept maps: pending logins (keyed by an opaque token) and passkey
/// registration challenges (keyed by account id).
pub(crate) struct TwofaState {
    pending: Mutex<HashMap<String, PendingLogin>>,
    reg: Mutex<HashMap<String, RegChallenge>>,
    passwordless: Mutex<HashMap<String, RegChallenge>>,
}

impl TwofaState {
    pub(crate) fn new() -> Self {
        TwofaState {
            pending: Mutex::new(HashMap::new()),
            reg: Mutex::new(HashMap::new()),
            passwordless: Mutex::new(HashMap::new()),
        }
    }

    fn put_pending(&self, token: String, entry: PendingLogin) {
        let mut map = self.pending.lock().expect("twofa pending poisoned");
        map.retain(|_, e| e.expires_at > Instant::now());
        map.insert(token, entry);
    }

    /// Redeem a pending login exactly once (removes it) — used on SUCCESS so the
    /// challenge and credentials cannot be reused.
    fn take_pending(&self, token: &str) -> Option<PendingLogin> {
        let mut map = self.pending.lock().expect("twofa pending poisoned");
        let entry = map.remove(token)?;
        (entry.expires_at > Instant::now()).then_some(entry)
    }

    pub(crate) fn discard_link_pending(&self, token: &str) {
        self.take_pending(token);
    }
    pub(crate) fn native_link_pending(&self, token: &str) -> bool {
        self.peek_pending(token)
            .is_some_and(|p| p.args.jmap_url == "engine" && p.args.api_url == "engine")
    }

    /// A clone of a live pending login without consuming it — used to READ the
    /// challenge/args before a verify that may fail (so TOTP can be retried).
    fn peek_pending(&self, token: &str) -> Option<PendingLogin> {
        let mut map = self.pending.lock().expect("twofa pending poisoned");
        let entry = map.get(token)?;
        if entry.expires_at <= Instant::now() {
            map.remove(token);
            return None;
        }
        Some(entry.clone())
    }

    /// Record a failed attempt; returns `true` and drops the entry once the cap is
    /// exceeded (so a subsequent lookup fails).
    fn note_failure(&self, token: &str) -> bool {
        let mut map = self.pending.lock().expect("twofa pending poisoned");
        if let Some(e) = map.get_mut(token) {
            e.attempts += 1;
            if e.attempts >= e.max_attempts {
                map.remove(token);
                return true;
            }
        }
        false
    }

    fn put_reg(&self, account_id: String, entry: RegChallenge) {
        let mut map = self.reg.lock().expect("twofa reg poisoned");
        map.retain(|_, e| e.expires_at > Instant::now());
        map.insert(account_id, entry);
    }

    fn take_reg(&self, account_id: &str) -> Option<RegChallenge> {
        let mut map = self.reg.lock().expect("twofa reg poisoned");
        let entry = map.remove(account_id)?;
        (entry.expires_at > Instant::now()).then_some(entry)
    }
}

/// The credential-validated arguments a completed login needs, carried through the
/// second-factor round-trip unchanged (so the password is checked exactly once).
#[derive(Clone)]
pub(crate) struct SessionArgs {
    pub authentication_method: &'static str,
    pub primary_identity: crate::primary_identity::VerifiedLoginIdentity,
    pub account_id: String,
    pub username: String,
    /// The `jmap_url`/`api_url` [`Store::create_session`](mw_store::Store) persists
    /// (proxy mode) or the literal `"engine"` sentinel (engine mode).
    pub jmap_url: String,
    pub api_url: String,
    pub creds: Credentials,
    pub client_type: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Login gate (called from lib.rs, both branches)
// ─────────────────────────────────────────────────────────────────────────────

/// Insert the second-factor gate after credential validation. Either completes the
/// login (no factor needed) or returns a `twofaRequired` body carrying a one-shot
/// pending token; NO session is issued until the factor clears.
///
/// An administratively disabled account is refused here, before any factor is
/// looked up, so it is never issued a challenge or a pending token. Every caller
/// has already validated the credentials, so the refusal costs what a successful
/// login costs up to this point, and its body is the wrong-password body.
///
/// The two names of this login are recorded against the account first
/// (`account_gate::remember_login`), so a flag under either reaches the account's
/// keys and tokens, and a flag under a name used at an earlier login reaches this
/// one.
pub(crate) async fn gate_login(
    state: &AppState,
    headers: &HeaderMap,
    ext: &axum::http::Extensions,
    args: SessionArgs,
) -> Response {
    let device = crate::security_center::device(headers, ext, args.client_type.as_deref());
    let policy = match crate::security_center::policy(state, &args.account_id, &args.username).await
    {
        Ok(p) => p,
        Err(r) => return r,
    };
    let store = &state.store;

    if let Some(response) = domain_signin_gate(state, &args, &device).await {
        return response;
    }

    if let Err(e) = account_gate::remember_login(
        store,
        &args.account_id,
        &args.username,
        &args.creds.username,
    )
    .await
    {
        return account_gate::unavailable("login", e);
    }
    match login_gate(state, &args).await {
        Ok(gate) if gate.disabled => {
            // Engine mode connected the account to validate the password; a
            // disabled account must not be left syncing.
            account_gate::stop_engine_work(state, &args.account_id);
            crate::security_center::record_failure(
                state,
                Some(&args.account_id),
                &args.username,
                &device,
                args.authentication_method,
                "account-disabled",
            )
            .await;
            return disabled_response(&args.username);
        }
        Ok(_) => {}
        Err(e) => return account_gate::unavailable("login", e),
    }

    let totp_enrolled = match store.get_totp_secret(&args.account_id).await {
        Ok(Some(t)) => t.confirmed,
        Ok(None) => false,
        Err(e) => return server_error("read totp", e),
    };
    let passkeys = match store.list_webauthn_credentials(&args.account_id).await {
        Ok(v) => v,
        Err(e) => return server_error("list passkeys", e),
    };
    let recovery_available = match store.list_unused_recovery_codes(&args.account_id).await {
        Ok(v) => !v.is_empty(),
        Err(e) => return server_error("list recovery", e),
    };
    let enrolled = totp_enrolled || !passkeys.is_empty();

    let required = policy.required
        || match policy_requires(state, &args.account_id, &args.username).await {
            Ok(r) => r,
            Err(e) => return server_error("read 2fa policy", e),
        };

    if !policy.passkeys_enabled && !passkeys.is_empty() && !totp_enrolled {
        return twofa_unauthorized();
    }
    if enrolled {
        let session = mw_store::Session {
            id: String::new(),
            account_id: args.account_id.clone(),
            username: args.username.clone(),
            jmap_url: args.jmap_url.clone(),
            api_url: args.api_url.clone(),
            credentials: args.creds.clone(),
        };
        match crate::security_center::remembered_login(state, headers, &session).await {
            Ok(true) => {
                return complete_login(state, &args, &device, json!({}), Stage::Remembered).await;
            }
            Ok(false) => {}
            Err(r) => return r,
        }
    }
    // Opt-in user with no factor and no policy requiring one → unchanged path.
    if !enrolled && !required {
        return complete_login(state, &args, &device, json!({}), Stage::Password).await;
    }

    let Some((origin, rp_id)) = crate::step_up::webauthn_rp(&state.external, headers, ext) else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Configure a valid public WebAuthn origin.",
        )
            .into_response();
    };
    let token = new_token();

    if enrolled {
        let challenge = random_challenge();
        state.twofa.put_pending(
            token.clone(),
            PendingLogin {
                kind: PendingKind::Verify,
                args,
                device,
                max_attempts: policy.max_attempts,
                challenge: challenge.clone(),
                origin,
                rp_id: rp_id.clone(),
                attempts: 0,
                expires_at: Instant::now() + Duration::from_secs(policy.pending_secs),
            },
        );
        let mut factors = Vec::new();
        if totp_enrolled {
            factors.push("totp");
        }
        if policy.passkeys_enabled && !passkeys.is_empty() {
            factors.push("webauthn");
        }
        if recovery_available {
            factors.push("recovery");
        }
        let cred_ids: Vec<String> = passkeys.iter().map(|c| c.credential_id.clone()).collect();
        Json(json!({
            "twofaRequired": true,
            "pendingToken": token,
            "factors": factors,
            "webauthn": {
                "challenge": b64(&challenge),
                "credentialIds": cred_ids,
                "rpId": rp_id,
                "userVerification": policy.passkey.assertion_uv_hint(),
            },
        }))
        .into_response()
    } else {
        // Required by policy but nothing enrolled: force enrolment on this login.
        state.twofa.put_pending(
            token.clone(),
            PendingLogin {
                kind: PendingKind::Enroll,
                args,
                device,
                max_attempts: policy.max_attempts,
                challenge: Vec::new(),
                origin,
                rp_id,
                attempts: 0,
                expires_at: Instant::now() + Duration::from_secs(policy.pending_secs),
            },
        );
        Json(json!({
            "twofaRequired": true,
            "enrollmentRequired": true,
            "pendingToken": token,
            "factors": if policy.passkeys_enabled { vec!["totp", "webauthn"] } else { vec!["totp"] },
        }))
        .into_response()
    }
}

/// Whether a second factor is REQUIRED for `username` by admin policy — the global
/// scope or the user's domain.
async fn policy_requires(
    state: &AppState,
    account_id: &str,
    username: &str,
) -> Result<bool, mw_store::StoreError> {
    if let Some(p) = state.store.get_twofa_policy("global", "").await?
        && p.require_2fa
    {
        return Ok(true);
    }
    if let Some(domain) = username.rsplit('@').next().filter(|d| *d != username)
        && let Some(p) = state.store.get_twofa_policy("domain", domain).await?
        && p.require_2fa
    {
        return Ok(true);
    }
    let subject = mw_store::settings_policy::SettingSubject {
        personal_owner: None,
        domain: username.rsplit_once('@').map_or("", |(_, d)| d),
        account_id,
        address: username,
    };
    if crate::settings_registry::effective_for(&state.store, "twofa.required", &subject)
        .await?
        .is_some_and(|v| v.value.as_bool() == Some(true))
    {
        return Ok(true);
    }
    Ok(false)
}

/// t48: would a normal sign-in of this account ask for a second factor? True
/// when one is enrolled or a policy requires one. A linked mailbox for which
/// this holds is never renewed from a stored password alone.
pub(crate) async fn factor_required(
    state: &AppState,
    account_id: &str,
    username: &str,
) -> Result<bool, mw_store::StoreError> {
    let store = &state.store;
    if store
        .get_totp_secret(account_id)
        .await?
        .is_some_and(|t| t.confirmed)
        || !store
            .list_webauthn_credentials(account_id)
            .await?
            .is_empty()
    {
        return Ok(true);
    }
    let required = crate::security_center::policy(state, account_id, username)
        .await
        .map_or(true, |p| p.required);
    Ok(required || policy_requires(state, account_id, username).await?)
}

/// The refusal for a disabled account whose credentials were valid: the same
/// status and body as a wrong password (`crate::unauthorized`), so the response
/// does not tell the caller that the account exists and is disabled.
fn disabled_response(username: &str) -> Response {
    tracing::info!("login refused: account {username} is disabled");
    crate::unauthorized()
}

/// The account flags for the login `args` describes.
async fn login_gate(
    state: &AppState,
    args: &SessionArgs,
) -> Result<account_gate::AccountGate, mw_store::StoreError> {
    account_gate::for_login(
        &state.store,
        &args.account_id,
        &args.username,
        &args.creds.username,
    )
    .await
}

/// Which step of a login is being completed, which decides what a refusal looks
/// like: the wrong-password answer after the password step, the wrong-factor
/// answer after a second-factor or forced-enrolment step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Password,
    SecondFactor,
    Remembered,
    Passwordless,
}

/// Whether the account of a pending login has been disabled since the password
/// step. The second-factor handler and the two enrolment-completing handlers ask
/// before they look at what was presented and answer [`twofa_unauthorized`] if so,
/// so a correct factor and a wrong one get the same answer and neither is
/// consumed. `Err` is the response for flags that cannot be read.
async fn pending_disabled(state: &AppState, pending: &PendingLogin) -> Result<bool, Response> {
    match login_gate(state, &pending.args).await {
        Ok(gate) => {
            if gate.disabled {
                tracing::info!(
                    "second-factor step refused: account {} is disabled",
                    pending.args.username
                );
                account_gate::stop_engine_work(state, &pending.args.account_id);
                crate::security_center::record_failure(
                    state,
                    Some(&pending.args.account_id),
                    &pending.args.username,
                    &pending.device,
                    pending.args.authentication_method,
                    "account-disabled",
                )
                .await;
            }
            Ok(gate.disabled)
        }
        Err(e) => Err(account_gate::unavailable("login", e)),
    }
}

/// Create the session and finish the login, merging `extra` (e.g. one-time recovery
/// codes) into the success body.
///
/// The account's flags are read again here: the second-factor and forced-enrolment
/// handlers reach this with a [`PendingLogin`] minted up to [`PENDING_TTL`] earlier,
/// and an account disabled in between must not get a session. An account held for
/// a password change does get one, with `passwordChangeRequired` in the body.
async fn complete_login(
    state: &AppState,
    args: &SessionArgs,
    device: &mw_store::SessionDevice,
    mut extra: serde_json::Value,
    stage: Stage,
) -> Response {
    if let Some(response) = domain_signin_gate(state, args, device).await {
        return response;
    }
    let gate = match login_gate(state, args).await {
        Ok(gate) => gate,
        Err(e) => return account_gate::unavailable("login", e),
    };
    if gate.disabled {
        account_gate::stop_engine_work(state, &args.account_id);
        crate::security_center::record_failure(
            state,
            Some(&args.account_id),
            &args.username,
            device,
            args.authentication_method,
            "account-disabled",
        )
        .await;
        return match stage {
            Stage::Password => disabled_response(&args.username),
            Stage::SecondFactor | Stage::Remembered | Stage::Passwordless => {
                tracing::info!("login refused: account {} is disabled", args.username);
                twofa_unauthorized()
            }
        };
    }
    if gate.password_change_required
        && let Some(obj) = extra.as_object_mut()
    {
        obj.insert("passwordChangeRequired".into(), json!(true));
    }
    let id = match state
        .store
        .create_session(
            &args.account_id,
            &args.username,
            &args.jmap_url,
            &args.api_url,
            &args.creds,
        )
        .await
    {
        Ok(id) => id,
        Err(e) => return server_error("persist session", e),
    };
    let method = if args.authentication_method == "header-auth" {
        "header-auth"
    } else {
        match stage {
            Stage::Password => "password",
            Stage::SecondFactor => "password+factor",
            Stage::Remembered => "remembered-device",
            Stage::Passwordless => "passkey",
        }
    };
    if let Err(error) = state.store.set_session_authentication(&id, method).await {
        let _ = state.store.delete_session(&id).await;
        return server_error("authentication method", error);
    }
    if let Err(error) = args
        .primary_identity
        .bind(state, &id, &args.account_id)
        .await
    {
        let _ = state.store.delete_session(&id).await;
        return server_error("bind primary identity", error);
    }
    if let Err(response) = crate::security_center::record_login(
        state,
        &id,
        &args.account_id,
        &args.username,
        device,
        method,
    )
    .await
    {
        let _ = state.store.delete_session(&id).await;
        return response;
    }
    state.sessions.begin(&id);
    crate::finish_login_ext(
        state,
        &id,
        &args.account_id,
        &args.username,
        args.client_type.as_deref(),
        extra,
    )
    .await
}

async fn domain_signin_gate(
    state: &AppState,
    args: &SessionArgs,
    device: &mw_store::SessionDevice,
) -> Option<Response> {
    match crate::domain_config::signin_allowed(&state.store, &args.creds.username).await {
        Ok(true) => None,
        Ok(false) => {
            crate::security_center::record_failure(
                state,
                Some(&args.account_id),
                &args.username,
                device,
                args.authentication_method,
                "domain-denied",
            )
            .await;
            account_gate::stop_engine_work(state, &args.account_id);
            Some(crate::unauthorized())
        }
        Err(error) => Some(account_gate::unavailable("domain sign-in policy", error)),
    }
}

async fn login_methods(State(state): State<AppState>) -> Response {
    Json(json!({"passwordless":state.engine.is_some()})).into_response()
}
async fn passwordless_begin(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
) -> Response {
    if let Err(error) = crate::signin_guard::check(
        &state,
        mw_store::AuthSurface::Webmail,
        crate::scope_mw::proxy::client_ip(&headers, &ext),
    )
    .await
    {
        return error.into_response();
    }

    if state.engine.is_none() {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({"code":"passwordlessEngineOnly"})),
        )
            .into_response();
    }
    let Some((origin, rp_id)) = crate::step_up::webauthn_rp(&state.external, &headers, &ext) else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let token = new_token();
    let challenge = random_challenge();
    {
        let mut pending = state.twofa.passwordless.lock().expect("passwordless lock");
        pending.retain(|_, value| value.expires_at > Instant::now());
        if pending.len() >= 1024 {
            return StatusCode::TOO_MANY_REQUESTS.into_response();
        }
        pending.insert(
            token.clone(),
            RegChallenge {
                discoverable: true,
                challenge: challenge.clone(),
                origin,
                rp_id: rp_id.clone(),
                expires_at: Instant::now() + PENDING_TTL,
            },
        );
    }
    Json(json!({"pendingToken":token,"challenge":b64(&challenge),"rpId":rp_id,"credentialIds":[],"userVerification":"required"})).into_response()
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PasswordlessFinish {
    pending_token: String,
    credential_id: String,
    client_data_json: String,
    authenticator_data: String,
    signature: String,
    #[serde(default)]
    user_handle: Option<String>,
    #[serde(default)]
    client_type: Option<String>,
}
async fn rejected_passkey(
    state: &AppState,
    device: &mw_store::SessionDevice,
    reason: &str,
) -> Response {
    crate::security_center::record_failure(state, None, "", device, "passkey", reason).await;
    twofa_unauthorized()
}

async fn passwordless_finish(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    Json(body): Json<PasswordlessFinish>,
) -> Response {
    if let Err(error) = crate::signin_guard::check(
        &state,
        mw_store::AuthSurface::Webmail,
        crate::scope_mw::proxy::client_ip(&headers, &ext),
    )
    .await
    {
        return error.into_response();
    }

    let Some(engine) = &state.engine else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({"code":"passwordlessEngineOnly"})),
        )
            .into_response();
    };
    let device = crate::security_center::device(&headers, &ext, body.client_type.as_deref());
    let challenge = state
        .twofa
        .passwordless
        .lock()
        .expect("passwordless lock")
        .remove(&body.pending_token);
    let Some(challenge) = challenge.filter(|value| value.expires_at > Instant::now()) else {
        return rejected_passkey(&state, &device, "challenge-rejected").await;
    };
    let Ok(true) = state.store.passkey_discoverable(&body.credential_id).await else {
        return rejected_passkey(&state, &device, "credential-rejected").await;
    };
    let Ok(Some(credential)) = state
        .store
        .get_webauthn_credential(&body.credential_id)
        .await
    else {
        return rejected_passkey(&state, &device, "credential-rejected").await;
    };
    if !passkey_handle_matches(&credential.account_id, body.user_handle.as_deref(), true) {
        return rejected_passkey(&state, &device, "credential-rejected").await;
    }
    let Ok(Some(previous)) = state.store.webauthn_state(&body.credential_id).await else {
        return rejected_passkey(&state, &device, "credential-rejected").await;
    };
    let (Some(client_data_json), Some(authenticator_data), Some(signature)) = (
        decode_b64(&body.client_data_json),
        decode_b64(&body.authenticator_data),
        decode_b64(&body.signature),
    ) else {
        return rejected_passkey(&state, &device, "credential-rejected").await;
    };
    let request = AssertionRequest {
        challenge: challenge.challenge,
        origin: challenge.origin,
        rp_id: challenge.rp_id,
        client_data_json,
        authenticator_data,
        signature,
        cose_public_key: credential.cose_public_key.clone(),
        stored_sign_count: credential.sign_count.max(0) as u32,
        user_verification: UserVerification::Required,
    };
    let Ok(assertion) = verify_assertion_with_backup(&request, previous.backup_eligible) else {
        return rejected_passkey(&state, &device, "credential-rejected").await;
    };
    let Ok(account) = state.store.get_account(&credential.account_id).await else {
        return rejected_passkey(&state, &device, "credential-rejected").await;
    };
    let credentials = if account.kind == mw_store::AccountKind::Ews {
        if !matches!(
            crate::domain_provider::stored_selection(&state.store, &account.id).await,
            Ok(Some(_))
        ) {
            return rejected_passkey(&state, &device, "credential-rejected").await;
        }
        state
            .store
            .provider_ews_credentials(&account.id)
            .await
            .and_then(|credentials| {
                credentials
                    .map(|credentials| mw_store::Credentials {
                        username: account.username.clone(),
                        password: credentials.password,
                    })
                    .ok_or(mw_store::StoreError::NotFound)
            })
    } else {
        state.store.account_credentials(&account.id).await
    };
    let Ok(creds) = credentials else {
        return rejected_passkey(&state, &device, "credential-rejected").await;
    };
    let policy = match crate::security_center::policy(&state, &account.id, &account.username).await
    {
        Ok(policy) => policy,
        Err(r) => return r,
    };
    if !policy.passkeys_enabled {
        return rejected_passkey(&state, &device, "credential-rejected").await;
    }
    // The signature verified with UV: only now is a policy refusal named.
    let facts = match stored_facts(&state, &credential, &previous).await {
        Ok(f) => f,
        Err(e) => return server_error("passkey policy", e),
    };
    if let Err(refusal) = policy.passkey.assertion_rule(facts.as_stored(&credential)) {
        return policy_refused(
            &state,
            &account.id,
            &account.username,
            &device,
            "passwordless",
            refusal,
        )
        .await;
    }
    let primary_identity =
        match crate::primary_identity::VerifiedLoginIdentity::engine(&state, &account.id).await {
            Ok(identity) => identity,
            Err(error) => return server_error("resolve primary identity", error),
        };
    let args = SessionArgs {
        authentication_method: "passkey",
        primary_identity,
        account_id: account.id.clone(),
        username: account.username,
        jmap_url: "engine".into(),
        api_url: "engine".into(),
        creds,
        client_type: body.client_type,
    };
    if let Some(response) = domain_signin_gate(&state, &args, &device).await {
        return response;
    }
    let gate = match login_gate(&state, &args).await {
        Ok(gate) => gate,
        Err(e) => return server_error("account policy", e),
    };
    if gate.disabled {
        crate::security_center::record_failure(
            &state,
            Some(&args.account_id),
            &args.username,
            &device,
            args.authentication_method,
            "account-disabled",
        )
        .await;
        return twofa_unauthorized();
    }
    let upstream = match crate::upstream_resolve::resolve_upstream(&state, &args.username).await {
        Ok(upstream) => upstream,
        Err(_) => return twofa_unauthorized(),
    };
    let Ok(account) = state.store.get_account(&args.account_id).await else {
        return twofa_unauthorized();
    };
    if !crate::engine_mode::account_matches_upstream(&state.store, &account, upstream.url())
        .await
        .unwrap_or(false)
    {
        return twofa_unauthorized();
    }
    if !engine.is_registered(&args.account_id) {
        let attempt = if account.kind == mw_store::AccountKind::Ews {
            match (
                crate::domain_provider::stored_selection(&state.store, &args.account_id).await,
                ext.get::<crate::plugins::PluginRegistry>(),
            ) {
                (Ok(Some(selected)), Some(registry)) => {
                    crate::domain_provider::login(engine, registry, selected, &args.creds.password)
                        .await
                }
                _ => return twofa_unauthorized(),
            }
        } else {
            crate::engine_mode::engine_login(
                engine,
                upstream.url(),
                &args.creds.username,
                &args.creds.password,
            )
            .await
        };
        match attempt {
            Ok((id, _)) if id == args.account_id => {}
            _ => return twofa_unauthorized(),
        }
    }
    match commit_passkey_assertion(&state, &credential, &previous, &assertion).await {
        Ok(true) => {}
        Ok(false) => return twofa_unauthorized(),
        Err(e) => return server_error("passkey state", e),
    }

    complete_login(&state, &args, &device, json!({}), Stage::Passwordless).await
}

// ─────────────────────────────────────────────────────────────────────────────
// Step 2: POST /api/login/2fa  (pre-auth, pending-token)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VerifyReq {
    pending_token: String,
    method: String,
    /// TOTP or recovery code.
    #[serde(default)]
    code: Option<String>,
    /// WebAuthn assertion fields (base64).
    #[serde(default)]
    credential_id: Option<String>,
    #[serde(default)]
    client_data_json: Option<String>,
    #[serde(default)]
    authenticator_data: Option<String>,
    #[serde(default)]
    signature: Option<String>,
    #[serde(default)]
    user_handle: Option<String>,
}

/// The link wrapper retains and owner-binds the pending token. It cannot change
/// the original native-client login arguments or bypass normal factor checks.
pub(crate) async fn verify_link_factor(
    state: AppState,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    pending: String,
    mut body: serde_json::Value,
) -> Response {
    if state.twofa.peek_pending(&pending).is_none() {
        return (
            StatusCode::GONE,
            Json(json!({"code":"linkedChallengeExpired","error":"linkedChallengeExpired"})),
        )
            .into_response();
    }
    let Some(object) = body.as_object_mut() else {
        return twofa_unauthorized();
    };
    object.insert("pendingToken".into(), serde_json::Value::String(pending));
    let Ok(body) = serde_json::from_value::<VerifyReq>(body) else {
        return twofa_unauthorized();
    };
    verify_2fa(State(state), headers, ext, Json(body)).await
}

/// Verify the presented second factor for a pending login and, on success, issue the
/// session. A wrong factor returns a uniform 401 and (except after the attempt cap)
/// keeps the pending token live so the user can retry.
///
/// An account disabled since the password step is treated as a wrong factor
/// whatever was presented: the same 401, the same count against the attempt cap,
/// and the factor is not looked at (no TOTP step or recovery code is spent).
async fn verify_2fa(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    Json(body): Json<VerifyReq>,
) -> Response {
    if let Err(error) = crate::signin_guard::check(
        &state,
        mw_store::AuthSurface::Webmail,
        crate::scope_mw::proxy::client_ip(&headers, &ext),
    )
    .await
    {
        return error.into_response();
    }

    let device = crate::security_center::device(&headers, &ext, None);
    let Some(pending) = state.twofa.peek_pending(&body.pending_token) else {
        crate::security_center::record_failure(
            &state,
            None,
            "",
            &device,
            &body.method,
            "challenge-rejected",
        )
        .await;
        return twofa_unauthorized();
    };
    if pending.kind != PendingKind::Verify {
        crate::security_center::record_failure(
            &state,
            None,
            "",
            &device,
            &body.method,
            "challenge-rejected",
        )
        .await;
        return twofa_unauthorized();
    }
    if body.method == "webauthn" {
        match crate::security_center::policy(
            &state,
            &pending.args.account_id,
            &pending.args.username,
        )
        .await
        {
            Ok(policy) if policy.passkeys_enabled => {}
            Ok(_) => return twofa_unauthorized(),
            Err(r) => return r,
        }
    }
    match pending_disabled(&state, &pending).await {
        Ok(false) => {}
        Ok(true) => {
            state.twofa.note_failure(&body.pending_token);
            return twofa_unauthorized();
        }
        Err(resp) => return resp,
    }
    let account_id = &pending.args.account_id;

    let ok = match body.method.as_str() {
        "totp" => verify_totp_factor(&state, account_id, body.code.as_deref()).await,
        "recovery" => verify_recovery_factor(&state, account_id, body.code.as_deref()).await,
        "webauthn" => match verify_webauthn_factor(&state, &pending, &body).await {
            Ok(Ok(ok)) => Ok(ok),
            Ok(Err(refusal)) => {
                // Signed by an enrolled credential, but not one the policy
                // accepts: named (the caller proved possession), counted.
                state.twofa.note_failure(&body.pending_token);
                return policy_refused(
                    &state,
                    account_id,
                    &pending.args.username,
                    &device,
                    "second-factor",
                    refusal,
                )
                .await;
            }
            Err(e) => Err(e),
        },
        _ => Ok(false),
    };
    match ok {
        Ok(true) => {
            // Redeem the pending login exactly once (burns the challenge).
            match state.twofa.take_pending(&body.pending_token) {
                Some(p) => {
                    complete_login(&state, &p.args, &p.device, json!({}), Stage::SecondFactor).await
                }
                None => twofa_unauthorized(),
            }
        }
        Ok(false) => {
            state.twofa.note_failure(&body.pending_token);
            crate::security_center::record_failure(
                &state,
                Some(account_id),
                &pending.args.username,
                &device,
                &body.method,
                "factor_refused",
            )
            .await;
            twofa_unauthorized()
        }
        Err(e) => server_error("verify 2fa", e),
    }
}

async fn verify_totp_factor(
    state: &AppState,
    account_id: &str,
    code: Option<&str>,
) -> Result<bool, mw_store::StoreError> {
    let Some(code) = code else { return Ok(false) };
    let Some(secret) = state.store.get_totp_secret(account_id).await? else {
        return Ok(false);
    };
    if !secret.confirmed {
        return Ok(false);
    }
    let Some(matched_step) =
        totp::totp_verify(&secret.secret, code, now_unix(), &TotpParams::default())
    else {
        return Ok(false);
    };
    // Replay guard (L1): a valid code stays live for the ±1 window, so the same code
    // could be captured and reused within ~90 s. Remember the last step a login
    // consumed and refuse any step at or below it. `advance_totp_last_step` is a
    // compare-and-swap: it returns `true` (fresh, accept) only if this step strictly
    // exceeds the stored one, and `false` (replay, reject) otherwise — which also
    // makes two logins racing the same code resolve to a single winner at the DB.
    let step = i64::try_from(matched_step).unwrap_or(i64::MAX);
    state.store.advance_totp_last_step(account_id, step).await
}

async fn verify_recovery_factor(
    state: &AppState,
    account_id: &str,
    code: Option<&str>,
) -> Result<bool, mw_store::StoreError> {
    let Some(code) = code else { return Ok(false) };
    for hash in state.store.list_unused_recovery_codes(account_id).await? {
        if recovery::verify_code(code, &hash) {
            // Consume single-use; a lost race (already used) falls through to false.
            return state.store.consume_recovery_code(account_id, &hash).await;
        }
    }
    Ok(false)
}

async fn verify_webauthn_factor(
    state: &AppState,
    pending: &PendingLogin,
    body: &VerifyReq,
) -> Result<Result<bool, passkey_policy::Refusal>, mw_store::StoreError> {
    let (Some(cred_id), Some(cdj), Some(auth_data), Some(sig)) = (
        body.credential_id.as_deref(),
        body.client_data_json.as_deref().and_then(decode_b64),
        body.authenticator_data.as_deref().and_then(decode_b64),
        body.signature.as_deref().and_then(decode_b64),
    ) else {
        return Ok(Ok(false));
    };
    let Some(cred) = state.store.get_webauthn_credential(cred_id).await? else {
        return Ok(Ok(false));
    };
    // The credential MUST belong to the account this login is for.
    if cred.account_id != pending.args.account_id {
        return Ok(Ok(false));
    }
    if !passkey_handle_matches(&cred.account_id, body.user_handle.as_deref(), false) {
        return Ok(Ok(false));
    }
    let Some(previous) = state.store.webauthn_state(cred_id).await? else {
        return Ok(Ok(false));
    };
    let Ok(policy) =
        crate::security_center::policy(state, &pending.args.account_id, &pending.args.username)
            .await
    else {
        return Ok(Ok(false));
    };
    let facts = stored_facts(state, &cred, &previous).await?;
    let req = AssertionRequest {
        challenge: pending.challenge.clone(),
        origin: pending.origin.clone(),
        rp_id: pending.rp_id.clone(),
        client_data_json: cdj,
        authenticator_data: auth_data,
        signature: sig,
        cose_public_key: cred.cose_public_key.clone(),
        stored_sign_count: cred.sign_count.max(0) as u32,
        user_verification: UV,
    };
    let outcome = match verify_assertion_with_backup(&req, previous.backup_eligible) {
        Ok(outcome) => outcome,
        Err(_) => return Ok(Ok(false)),
    };
    if let Err(refusal) = assertion_meets_policy(&policy.passkey, facts.as_stored(&cred), &outcome)
    {
        return Ok(Err(refusal));
    }
    commit_passkey_assertion(state, &cred, &previous, &outcome)
        .await
        .map(Ok)
}

/// Check a signature-verified assertion against the passkey policy: a
/// non-compliant credential under `legacy = refuse`, or a missing UV flag where
/// the rule asks for it.
pub(crate) fn assertion_meets_policy(
    policy: &passkey_policy::PasskeyPolicy,
    cred: passkey_policy::StoredCredential<'_>,
    outcome: &mw_mfa::AssertionOutcome,
) -> Result<(), passkey_policy::Refusal> {
    match policy.assertion_rule(cred)? {
        UserVerification::Required if !outcome.user_verified => {
            Err(passkey_policy::Refusal::UserVerification)
        }
        _ => Ok(()),
    }
}

/// The stored properties of a credential the policy is checked against.
pub(crate) struct StoredFacts {
    discoverable: bool,
    uv_seen: bool,
}

impl StoredFacts {
    pub(crate) fn as_stored<'a>(
        &self,
        cred: &'a WebauthnCredentialRow,
    ) -> passkey_policy::StoredCredential<'a> {
        passkey_policy::StoredCredential {
            transports: &cred.transports,
            discoverable: self.discoverable,
            uv_seen: self.uv_seen,
        }
    }
}

pub(crate) async fn stored_facts(
    state: &AppState,
    cred: &WebauthnCredentialRow,
    previous: &mw_store::WebauthnState,
) -> Result<StoredFacts, mw_store::StoreError> {
    Ok(StoredFacts {
        discoverable: state
            .store
            .passkey_discoverable(&cred.credential_id)
            .await?,
        uv_seen: previous.uv_initialized,
    })
}

/// Answer a passkey-policy refusal: 403 with the code the web app localises,
/// a security event for the account and an audit line.
pub(crate) async fn policy_refused(
    state: &AppState,
    account_id: &str,
    username: &str,
    device: &mw_store::SessionDevice,
    flow: &str,
    refusal: passkey_policy::Refusal,
) -> Response {
    crate::security_center::record_failure(
        state,
        Some(account_id),
        username,
        device,
        "passkey",
        refusal.code(),
    )
    .await;
    policy_audit(state, username, flow, refusal).await;
    (StatusCode::FORBIDDEN, Json(refusal.body())).into_response()
}

/// The audit line of a policy refusal (actor: the user; non-fatal).
pub(crate) async fn policy_audit(
    state: &AppState,
    username: &str,
    flow: &str,
    refusal: passkey_policy::Refusal,
) {
    let entry = mw_admin::AuditLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: username.to_string(),
        actor_kind: mw_admin::ActorKind::User,
        action: "passkey-policy-refused".to_string(),
        target: Some(username.to_string()),
        detail_json: mw_admin::redact_detail(&json!({ "flow": flow, "reason": refusal.code() })),
        ip: None,
    };
    if let Err(e) = state.v6.admin.audit(entry).await {
        tracing::warn!("passkey policy audit append failed: {e}");
    }
}

pub(crate) fn passkey_user_handle(account: &str) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(b"mailwoman.webauthn.user\0");
    hash.update(account.as_bytes());
    hash.finalize().to_vec()
}

pub(crate) fn passkey_handle_matches(account: &str, encoded: Option<&str>, required: bool) -> bool {
    let Some(encoded) = encoded else {
        return !required;
    };
    let Some(handle) = decode_b64(encoded) else {
        return false;
    };
    // Existing credentials used account-id bytes. New registrations use an opaque
    // fixed-size handle (WebAuthn user.id is limited to 64 bytes).
    handle == passkey_user_handle(account) || handle == account.as_bytes()
}

pub(crate) async fn commit_passkey_assertion(
    state: &AppState,
    credential: &WebauthnCredentialRow,
    previous: &mw_store::WebauthnState,
    outcome: &mw_mfa::AssertionOutcome,
) -> Result<bool, mw_store::StoreError> {
    let next = mw_store::WebauthnState {
        backup_eligible: Some(outcome.backup_eligible),
        backup_state: outcome.backup_state,
        uv_initialized: outcome.user_verified,
        revision: previous.revision,
    };
    let accepted = state
        .store
        .advance_webauthn_state(
            credential,
            previous,
            &next,
            i64::from(outcome.new_sign_count),
        )
        .await?;
    if accepted && outcome.counter_anomaly {
        tracing::info!(credential = %credential_handle(&credential.credential_id), "multi-device passkey counter anomaly; preserved monotonic stored counter");
    }
    Ok(accepted)
}

// ─────────────────────────────────────────────────────────────────────────────
// Forced enrolment over a pending token (policy-required, no session yet)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PendingTokenReq {
    pending_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PendingTotpConfirmReq {
    pending_token: String,
    code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PendingPasskeyFinishReq {
    pending_token: String,
    client_data_json: String,
    attestation_object: String,
    #[serde(default)]
    transports: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    authenticator_attachment: Option<String>,
    #[serde(default)]
    resident_key: Option<bool>,
}

/// Begin forced TOTP enrolment for an Enroll-state pending login.
async fn login_enroll_totp_begin(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    Json(body): Json<PendingTokenReq>,
) -> Response {
    if let Err(error) = crate::signin_guard::check(
        &state,
        mw_store::AuthSurface::Webmail,
        crate::scope_mw::proxy::client_ip(&headers, &ext),
    )
    .await
    {
        return error.into_response();
    }

    let Some(p) = enroll_pending(&state, &body.pending_token) else {
        return twofa_unauthorized();
    };
    totp_begin(&state, &p.args.account_id, &p.args.username).await
}

/// Confirm forced TOTP enrolment; on success issue recovery codes AND the session.
async fn login_enroll_totp_confirm(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    Json(body): Json<PendingTotpConfirmReq>,
) -> Response {
    if let Err(error) = crate::signin_guard::check(
        &state,
        mw_store::AuthSurface::Webmail,
        crate::scope_mw::proxy::client_ip(&headers, &ext),
    )
    .await
    {
        return error.into_response();
    }

    let Some(p) = enroll_pending(&state, &body.pending_token) else {
        return twofa_unauthorized();
    };
    // Disabled since the password step: answered as a code or attestation that
    // did not verify, and nothing is enrolled.
    match pending_disabled(&state, &p).await {
        Ok(false) => {}
        Ok(true) => return twofa_unauthorized(),
        Err(resp) => return resp,
    }
    match confirm_totp_and_seed_recovery(&state, &p.args.account_id, &body.code).await {
        Ok(Some(codes)) => {
            state.twofa.take_pending(&body.pending_token);
            complete_login(
                &state,
                &p.args,
                &p.device,
                json!({ "recoveryCodes": codes }),
                Stage::SecondFactor,
            )
            .await
        }
        Ok(None) => twofa_unauthorized(),
        Err(e) => server_error("confirm totp", e),
    }
}

/// Begin forced passkey enrolment (registration challenge) for a pending login.
async fn login_enroll_passkey_begin(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    Json(body): Json<PendingTokenReq>,
) -> Response {
    if let Err(error) = crate::signin_guard::check(
        &state,
        mw_store::AuthSurface::Webmail,
        crate::scope_mw::proxy::client_ip(&headers, &ext),
    )
    .await
    {
        return error.into_response();
    }

    let Some(p) = enroll_pending(&state, &body.pending_token) else {
        return twofa_unauthorized();
    };
    match crate::security_center::policy(&state, &p.args.account_id, &p.args.username).await {
        Ok(policy) if policy.passkeys_enabled => {}
        Ok(_) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({"code":"passkeysDisabled"})),
            )
                .into_response();
        }
        Err(r) => return r,
    }
    passkey_begin(&state, &headers, &ext, &p.args.account_id, &p.args.username).await
}

/// Finish forced passkey enrolment; on success issue recovery codes AND the session.
async fn login_enroll_passkey_finish(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    Json(body): Json<PendingPasskeyFinishReq>,
) -> Response {
    if let Err(error) = crate::signin_guard::check(
        &state,
        mw_store::AuthSurface::Webmail,
        crate::scope_mw::proxy::client_ip(&headers, &ext),
    )
    .await
    {
        return error.into_response();
    }

    let Some(p) = enroll_pending(&state, &body.pending_token) else {
        return twofa_unauthorized();
    };
    // Disabled since the password step: answered as a code or attestation that
    // did not verify, and nothing is enrolled.
    match pending_disabled(&state, &p).await {
        Ok(false) => {}
        Ok(true) => return twofa_unauthorized(),
        Err(resp) => return resp,
    }
    match register_passkey(
        &state,
        &p.args.account_id,
        &p.args.username,
        &body.client_data_json,
        &body.attestation_object,
        &body.transports,
        &body.label,
        body.authenticator_attachment.as_deref(),
        body.resident_key,
    )
    .await
    {
        Ok(Registered::Refused(refusal)) => {
            policy_refused(
                &state,
                &p.args.account_id,
                &p.args.username,
                &p.device,
                "enrolment",
                refusal,
            )
            .await
        }
        Ok(Registered::Saved) => {
            let codes = match seed_recovery_if_empty(&state, &p.args.account_id).await {
                Ok(c) => c,
                Err(e) => return server_error("seed recovery", e),
            };
            state.twofa.take_pending(&body.pending_token);
            complete_login(
                &state,
                &p.args,
                &p.device,
                json!({ "recoveryCodes": codes }),
                Stage::SecondFactor,
            )
            .await
        }
        Ok(Registered::Failed) => twofa_unauthorized(),
        Err(e) => server_error("register passkey", e),
    }
}

/// A live Enroll-state pending login for `token`, or `None`.
///
/// The two "begin" steps do not read the account's flags: they answer with a
/// fresh secret or challenge for any pending enrolment, so an account disabled
/// since the password step looks the same there. It is refused at the "confirm" /
/// "finish" step ([`pending_disabled`]), as a code that did not verify.
fn enroll_pending(state: &AppState, token: &str) -> Option<PendingLogin> {
    let p = state.twofa.peek_pending(token)?;
    (p.kind == PendingKind::Enroll).then_some(p)
}

// ─────────────────────────────────────────────────────────────────────────────
// Authenticated enrolment / management  (cookie- or bearer-authed)
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/account/2fa` — the caller's factor status.
async fn twofa_status(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let account_id = &session.account_id;
    let totp = matches!(state.store.get_totp_secret(account_id).await, Ok(Some(t)) if t.confirmed);
    let passkeys = match state.store.list_webauthn_credentials(account_id).await {
        Ok(v) => v,
        Err(e) => return server_error("list passkeys", e),
    };
    let recovery_remaining = match state.store.list_unused_recovery_codes(account_id).await {
        Ok(v) => v.len(),
        Err(e) => return server_error("list recovery", e),
    };
    let required = match policy_requires(&state, &session.account_id, &session.username).await {
        Ok(r) => r,
        Err(e) => return server_error("read policy", e),
    };
    let effective_policy = match crate::security_center::policy(
        &state,
        &session.account_id,
        &session.username,
    )
    .await
    {
        Ok(policy) => policy,
        Err(response) => return response,
    };
    let mut passkey_json = Vec::with_capacity(passkeys.len());
    for c in &passkeys {
        let previous = match state.store.webauthn_state(&c.credential_id).await {
            Ok(p) => p.unwrap_or_default(),
            Err(e) => return server_error("passkey state", e),
        };
        let facts = match stored_facts(&state, c, &previous).await {
            Ok(f) => f,
            Err(e) => return server_error("passkey state", e),
        };
        let gaps: Vec<&str> = effective_policy
            .passkey
            .credential_gaps(facts.as_stored(c))
            .iter()
            .map(|g| g.code())
            .collect();
        passkey_json.push(json!({
            "handle": credential_handle(&c.credential_id),
            "label": c.label,
            "createdAt": c.created_at,
            "meetsPolicy": gaps.is_empty(),
            "policyGaps": gaps,
        }));
    }
    Json(json!({
        "totp": totp,
        "passkeys": passkey_json,
        "recoveryRemaining": recovery_remaining,
        "policyRequired": required,
        "passkeysEnabled": effective_policy.passkeys_enabled,
        "passkeyPolicy": effective_policy.passkey.summary(),
    }))
    .into_response()
}

/// `POST /api/account/2fa/totp/begin` — start TOTP enrolment.
async fn guard_factor_enrollment(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<(), Response> {
    crate::step_up::require_known_authentication(state, session).await?;
    let has_factor = state
        .store
        .get_totp_secret(&session.account_id)
        .await
        .map_err(|e| server_error("factor", e))?
        .is_some_and(|t| t.confirmed)
        || !state
            .store
            .list_webauthn_credentials(&session.account_id)
            .await
            .map_err(|e| server_error("factor", e))?
            .is_empty();
    // A first SSO factor must remain enrollable without a pre-existing factor.
    if has_factor
        || crate::step_up::session_password_capable(state, session)
            .await
            .map_err(|e| server_error("authentication method", e))?
    {
        crate::step_up::require_user(
            state,
            session,
            crate::step_up::StepUpAction::TwoFactorDisable,
        )
        .await?;
    }
    Ok(())
}

async fn totp_begin_route(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if let Err(r) = guard_factor_enrollment(&state, &session).await {
        return r;
    }
    totp_begin(&state, &session.account_id, &session.username).await
}

/// `POST /api/account/2fa/totp/confirm` — confirm TOTP enrolment.
async fn totp_confirm_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CodeReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if let Err(r) = guard_factor_enrollment(&state, &session).await {
        return r;
    }
    match confirm_totp_and_seed_recovery(&state, &session.account_id, &body.code).await {
        Ok(Some(codes)) => Json(json!({ "ok": true, "recoveryCodes": codes })).into_response(),
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "code did not verify" })),
        )
            .into_response(),
        Err(e) => server_error("confirm totp", e),
    }
}

/// `POST /api/account/2fa/totp/disable` — remove TOTP (refused if it is the last
/// factor of a policy-required account).
async fn totp_disable_route(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if let Err(response) = crate::step_up::require_user(
        &state,
        &session,
        crate::step_up::StepUpAction::TwoFactorDisable,
    )
    .await
    {
        return response;
    }
    let account_id = &session.account_id;
    let _removal = removal_lock().lock().await;
    // An unconfirmed (pending) secret is not a factor: cancelling it removes
    // nothing the last-factor rule protects.
    let confirmed = match state.store.get_totp_secret(account_id).await {
        Ok(t) => t.is_some_and(|t| t.confirmed),
        Err(e) => return server_error("read totp", e),
    };
    if confirmed {
        let passkeys = match state.store.list_webauthn_credentials(account_id).await {
            Ok(v) => v.len(),
            Err(e) => return server_error("list passkeys", e),
        };
        if let Err(r) = guard_last_factor(
            &state,
            &session.account_id,
            &session.username,
            passkeys, /*remaining factors*/
        )
        .await
        {
            return r;
        }
    }
    if let Err(e) = state.store.delete_totp_secret(account_id).await {
        return server_error("delete totp", e);
    }
    crate::alerts::emit(
        &state,
        account_id,
        crate::alerts::AlertKind::TwoFactorChanged,
        json!({}),
    )
    .await;
    Json(json!({ "ok": true })).into_response()
}

/// `POST /api/account/2fa/passkey/begin` — issue a registration challenge.
async fn passkey_begin_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if let Err(r) = guard_factor_enrollment(&state, &session).await {
        return r;
    }
    match crate::security_center::policy(&state, &session.account_id, &session.username).await {
        Ok(policy) if policy.passkeys_enabled => {}
        Ok(_) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({"code":"passkeysDisabled"})),
            )
                .into_response();
        }
        Err(r) => return r,
    }
    passkey_begin(
        &state,
        &headers,
        &ext,
        &session.account_id,
        &session.username,
    )
    .await
}

/// `POST /api/account/2fa/passkey/finish` — verify + store a registered passkey.
async fn passkey_finish_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    Json(body): Json<PasskeyFinishReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if let Err(r) = guard_factor_enrollment(&state, &session).await {
        return r;
    }
    match crate::security_center::policy(&state, &session.account_id, &session.username).await {
        Ok(policy) if policy.passkeys_enabled => {}
        Ok(_) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({"code":"passkeysDisabled"})),
            )
                .into_response();
        }
        Err(r) => return r,
    }
    match register_passkey(
        &state,
        &session.account_id,
        &session.username,
        &body.client_data_json,
        &body.attestation_object,
        &body.transports,
        &body.label,
        body.authenticator_attachment.as_deref(),
        body.resident_key,
    )
    .await
    {
        Ok(Registered::Refused(refusal)) => {
            let device = crate::security_center::device(&headers, &ext, None);
            policy_refused(
                &state,
                &session.account_id,
                &session.username,
                &device,
                "registration",
                refusal,
            )
            .await
        }
        Ok(Registered::Saved) => {
            let codes = match seed_recovery_if_empty(&state, &session.account_id).await {
                Ok(c) => c,
                Err(e) => return server_error("seed recovery", e),
            };
            Json(json!({ "ok": true, "recoveryCodes": codes })).into_response()
        }
        Ok(Registered::Failed) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "registration did not verify" })),
        )
            .into_response(),
        Err(e) => server_error("register passkey", e),
    }
}

/// `POST /api/account/2fa/passkey/remove` — delete one passkey by its handle
/// (refused if it is the last factor of a policy-required account).
async fn passkey_remove_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<HandleReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if let Err(response) = crate::step_up::require_user(
        &state,
        &session,
        crate::step_up::StepUpAction::TwoFactorDisable,
    )
    .await
    {
        return response;
    }
    let account_id = &session.account_id;
    let _removal = removal_lock().lock().await;
    let passkeys = match state.store.list_webauthn_credentials(account_id).await {
        Ok(v) => v,
        Err(e) => return server_error("list passkeys", e),
    };
    let Some(cred) = passkeys
        .iter()
        .find(|c| credential_handle(&c.credential_id) == body.handle)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "no such passkey" })),
        )
            .into_response();
    };
    // Remaining factors AFTER this removal.
    let totp = matches!(state.store.get_totp_secret(account_id).await, Ok(Some(t)) if t.confirmed);
    let remaining = (passkeys.len() - 1) + usize::from(totp);
    if let Err(r) =
        guard_last_factor(&state, &session.account_id, &session.username, remaining).await
    {
        return r;
    }
    if let Err(e) = state
        .store
        .delete_webauthn_credential(&cred.credential_id)
        .await
    {
        return server_error("delete passkey", e);
    }
    crate::alerts::emit(
        &state,
        account_id,
        crate::alerts::AlertKind::TwoFactorChanged,
        json!({}),
    )
    .await;
    Json(json!({ "ok": true })).into_response()
}

/// `POST /api/account/2fa/recovery/regenerate` — issue a fresh set of recovery
/// codes (invalidating the old set), shown once.
async fn recovery_regenerate_route(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if let Err(response) = crate::step_up::require_user(
        &state,
        &session,
        crate::step_up::StepUpAction::TwoFactorDisable,
    )
    .await
    {
        return response;
    }
    let account_id = &session.account_id;
    let codes = recovery::generate_codes(recovery::DEFAULT_RECOVERY_CODES);
    let hashes: Vec<String> = codes.iter().map(|c| recovery::hash_code(c)).collect();
    if let Err(e) = state.store.clear_recovery_codes(account_id).await {
        return server_error("clear recovery", e);
    }
    if let Err(e) = state.store.add_recovery_codes(account_id, &hashes).await {
        return server_error("add recovery", e);
    }
    crate::alerts::emit(
        &state,
        account_id,
        crate::alerts::AlertKind::TwoFactorChanged,
        json!({}),
    )
    .await;
    Json(json!({ "ok": true, "recoveryCodes": codes })).into_response()
}

// ── session management (S11) ─────────────────────────────────────────────────

/// `GET /api/account/sessions` — the caller's active sessions (metadata only; the
/// current one flagged). The raw session id is never returned.
async fn sessions_list_route(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let current = mw_store::session_handle(&session.id);
    let metas = match state
        .store
        .list_security_sessions(Some(&session.account_id), 100)
        .await
    {
        Ok(m) => m,
        Err(e) => return server_error("list sessions", e),
    };
    let out: Vec<_> = metas
        .iter()
        .map(|m| {
            json!({
                "handle": m.handle,
                "username": m.username,
                "createdAt": m.created_at,
                "lastSeen": m.last_seen,
                "lastActivity": m.last_activity,
                "current": m.handle == current,
                "country": m.device.ip.parse().ok().and_then(mw_engine::security::country_for_ip),
                "userAgent":m.device.user_agent,"ip":m.device.ip,"clientType":m.device.client_type,"clientVersion":m.device.client_version,"deviceLabel":m.device.device_label,
            })
        })
        .collect();
    Json(json!({ "sessions": out })).into_response()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RevokeReq {
    /// A specific session handle to revoke; omit (or set `all`) to revoke every
    /// OTHER session (sign out everywhere else).
    #[serde(default)]
    handle: Option<String>,
    #[serde(default)]
    all: bool,
}

/// `POST /api/account/sessions/revoke` — revoke a specific session by handle, or all
/// but the current one.
async fn sessions_revoke_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RevokeReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if headers
        .get("x-mailwoman-account")
        .is_some_and(|expected| expected.to_str().ok() != Some(session.account_id.as_str()))
    {
        return (StatusCode::CONFLICT,Json(json!({"code":"sessionAccountChanged","error":"Account changed; reload sessions before continuing."}))).into_response();
    }
    let current = mw_store::session_handle(&session.id);
    match body.handle {
        Some(handle) if !body.all => {
            match state
                .store
                .revoke_session_by_handle(&session.account_id, &handle)
                .await
            {
                Ok(Some(id)) => {
                    state.sessions.forget(&id);
                    Json(json!({ "ok": true, "revoked": 1 })).into_response()
                }
                Ok(None) => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "no such session" })),
                )
                    .into_response(),
                Err(e) => server_error("revoke session", e),
            }
        }
        _ => match state
            .store
            .revoke_other_sessions(&session.account_id, &current)
            .await
        {
            Ok(ids) => {
                for id in &ids {
                    state.sessions.forget(id);
                }
                Json(json!({ "ok": true, "revoked": ids.len() })).into_response()
            }
            Err(e) => server_error("revoke sessions", e),
        },
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared enrolment helpers (used by both authed and forced-enrol paths)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CodeReq {
    code: String,
}

#[derive(Debug, Deserialize)]
struct HandleReq {
    handle: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PasskeyFinishReq {
    client_data_json: String,
    attestation_object: String,
    #[serde(default)]
    transports: String,
    #[serde(default)]
    label: String,
    /// `PublicKeyCredential.authenticatorAttachment` as the browser reported it.
    #[serde(default)]
    authenticator_attachment: Option<String>,
    /// `credProps.rk` as the browser reported it.
    #[serde(default)]
    resident_key: Option<bool>,
}

/// Generate + store an UNCONFIRMED TOTP secret and return the base32 secret +
/// `otpauth://` URI for the authenticator app.
async fn totp_begin(state: &AppState, account_id: &str, username: &str) -> Response {
    match state.store.get_totp_secret(account_id).await {
        Ok(Some(secret)) if secret.confirmed => return (StatusCode::CONFLICT,Json(json!({"code":"totpAlreadyEnrolled","error":"Remove the current authenticator before replacing it."}))).into_response(),
        Ok(_) => {}, Err(e) => return server_error("factor",e),
    }
    let secret = totp::generate_secret();
    if let Err(e) = state
        .store
        .put_totp_secret(account_id, &secret, false)
        .await
    {
        return server_error("store totp", e);
    }
    let uri = totp::provisioning_uri(&secret, TOTP_ISSUER, username, &TotpParams::default());
    Json(json!({
        "ok": true,
        "secret": totp::base32_encode(&secret),
        "otpauthUri": uri,
    }))
    .into_response()
}

/// Verify a TOTP code against the stored (pending) secret; on success confirm it and
/// seed recovery codes if the account has none. Returns the codes to display once
/// (empty vec if recovery codes already existed), or `None` if the code was wrong.
async fn confirm_totp_and_seed_recovery(
    state: &AppState,
    account_id: &str,
    code: &str,
) -> Result<Option<Vec<String>>, mw_store::StoreError> {
    let Some(secret) = state.store.get_totp_secret(account_id).await? else {
        return Ok(None);
    };
    // Enrolment confirmation proves possession of the secret. Bind the confirming
    // code to `last_step` exactly as the login path does ([`verify_totp_factor`]),
    // so the code used to enrol cannot be replayed at the first login within its
    // ±1 (~90 s) window. `advance_totp_last_step` is a compare-and-swap; we ignore
    // its bool here (enrolment succeeds regardless — a concurrent login racing the
    // same step is the single winner and the loser is a replay we want refused).
    let Some(matched_step) =
        totp::totp_verify(&secret.secret, code, now_unix(), &TotpParams::default())
    else {
        return Ok(None);
    };
    let step = i64::try_from(matched_step).unwrap_or(i64::MAX);
    state.store.advance_totp_last_step(account_id, step).await?;
    state.store.confirm_totp(account_id).await?;
    let codes = seed_recovery_if_empty(state, account_id).await?;
    Ok(Some(codes))
}

/// Issue + persist a fresh recovery set iff the account currently has none. Returns
/// the plaintext codes to show once (empty when a set already existed).
async fn seed_recovery_if_empty(
    state: &AppState,
    account_id: &str,
) -> Result<Vec<String>, mw_store::StoreError> {
    if !state
        .store
        .list_unused_recovery_codes(account_id)
        .await?
        .is_empty()
    {
        return Ok(Vec::new());
    }
    let codes = recovery::generate_codes(recovery::DEFAULT_RECOVERY_CODES);
    let hashes: Vec<String> = codes.iter().map(|c| recovery::hash_code(c)).collect();
    state.store.add_recovery_codes(account_id, &hashes).await?;
    Ok(codes)
}

/// Issue a passkey registration challenge (stored server-side keyed by account).
async fn passkey_begin(
    state: &AppState,
    headers: &HeaderMap,
    ext: &axum::http::Extensions,
    account_id: &str,
    username: &str,
) -> Response {
    let Some((origin, rp_id)) = crate::step_up::webauthn_rp(&state.external, headers, ext) else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Configure a valid public WebAuthn origin.",
        )
            .into_response();
    };
    let challenge = random_challenge();
    let credentials = match state.store.list_webauthn_credentials(account_id).await {
        Ok(rows) => rows,
        Err(e) => return server_error("list passkeys", e),
    };
    let policy = match crate::security_center::policy(state, account_id, username).await {
        Ok(p) => p.passkey,
        Err(r) => return r,
    };
    state.twofa.put_reg(
        account_id.to_string(),
        RegChallenge {
            discoverable: policy.discoverable(),
            challenge: challenge.clone(),
            origin,
            rp_id: rp_id.clone(),
            expires_at: Instant::now() + PENDING_TTL,
        },
    );
    let selection = policy.registration_options();
    // A stable per-account user handle (WebAuthn user.id), opaque and fixed-size.
    Json(json!({
        "ok": true,
        "challenge": b64(&challenge),
        "rpId": rp_id,
        "residentKey": selection["residentKey"],
        "userHandle": b64(&passkey_user_handle(account_id)),
        "userName": username,
        "userVerification": selection["userVerification"],
        "authenticatorAttachment": selection.get("authenticatorAttachment"),
        "authenticatorSelection": selection,
        "excludeCredentialIds": credentials.iter().map(|c|&c.credential_id).collect::<Vec<_>>(),
    }))
    .into_response()
}

/// What [`register_passkey`] did.
enum Registered {
    Saved,
    /// The challenge was missing/expired or verification failed (4xx).
    Failed,
    /// It verified but does not meet the account's passkey policy.
    Refused(passkey_policy::Refusal),
}

/// Verify a passkey registration ceremony against the stored challenge, check it
/// against the account's passkey policy (t39-e6) and persist the credential.
#[allow(clippy::too_many_arguments)]
async fn register_passkey(
    state: &AppState,
    account_id: &str,
    username: &str,
    client_data_json_b64: &str,
    attestation_object_b64: &str,
    transports: &str,
    label: &str,
    attachment: Option<&str>,
    resident_key: Option<bool>,
) -> Result<Registered, mw_store::StoreError> {
    let Some(reg) = state.twofa.take_reg(account_id) else {
        return Ok(Registered::Failed);
    };
    let (Some(cdj), Some(att)) = (
        decode_b64(client_data_json_b64),
        decode_b64(attestation_object_b64),
    ) else {
        return Ok(Registered::Failed);
    };
    // UV is checked below against the policy, after the signature verified, so
    // the refusal can say why.
    let req = RegistrationRequest {
        challenge: reg.challenge,
        origin: reg.origin,
        rp_id: reg.rp_id,
        attestation_object: att,
        client_data_json: cdj,
        user_verification: UV,
    };
    let cred = match verify_registration(&req) {
        Ok(c) => c,
        Err(_) => return Ok(Registered::Failed),
    };
    let Ok(policy) = crate::security_center::policy(state, account_id, username).await else {
        return Ok(Registered::Failed);
    };
    let mut policy = policy.passkey;
    // A challenge issued under a discoverable rule stays one.
    if reg.discoverable {
        policy.resident_key = passkey_policy::Requirement::Required;
    }
    let report = passkey_policy::ClientReport {
        attachment,
        transports,
        resident_key,
    };
    if let Err(refusal) = policy.check_registration(cred.user_verified, &report) {
        return Ok(Registered::Refused(refusal));
    }
    let credential_id = b64(&cred.credential_id);
    let row = WebauthnCredentialRow {
        credential_id,
        account_id: account_id.to_string(),
        cose_public_key: cred.cose_public_key,
        sign_count: i64::from(cred.sign_count),
        transports: transports.to_string(),
        label: if label.is_empty() {
            "passkey".to_string()
        } else {
            label.to_string()
        },
        created_at: String::new(),
    };
    let initial = mw_store::WebauthnState {
        backup_eligible: Some(cred.backup_eligible),
        backup_state: cred.backup_state,
        uv_initialized: cred.user_verified,
        revision: 0,
    };
    match state
        .store
        .register_webauthn_credential(&row, &initial, policy.discoverable())
        .await
    {
        Ok(()) => Ok(Registered::Saved),
        Err(mw_store::StoreError::Db(error))
            if error
                .as_database_error()
                .is_some_and(|e| e.is_unique_violation()) =>
        {
            Ok(Registered::Failed)
        }
        Err(error) => Err(error),
    }
}

/// Serialises factor removals in this process so two concurrent removals (two
/// passkeys, or a passkey and TOTP) cannot both count the other as the factor
/// that remains. Removals are rare; one process-wide lock is enough.
fn removal_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

/// The last-factor rule (t39-e6, t77). `remaining` = TOTP + passkeys that would
/// survive the removal; recovery codes do not count. When it is 0 the removal
/// is refused (409) if the account's policy requires two-factor sign-in
/// (`lastFactorRequired`) or `twofa.lastFactorRemoval` is `block` (or its
/// t39-e6 alias `requireRecoveryCodes`): `lastFactorBlocked`. Each refusal
/// writes a `passkey-policy-refused` audit entry (flow `last-factor-removal`).
async fn guard_last_factor(
    state: &AppState,
    account_id: &str,
    username: &str,
    remaining: usize,
) -> Result<(), Response> {
    if remaining > 0 {
        return Ok(());
    }
    let required = policy_requires(state, account_id, username)
        .await
        .map_err(|e| server_error("read policy", e))?;
    let policy = crate::security_center::policy(state, account_id, username).await?;
    if let Err(refusal) = policy.passkey.check_last_factor_removal(required) {
        policy_audit(state, username, "last-factor-removal", refusal).await;
        return Err((StatusCode::CONFLICT, Json(refusal.body())).into_response());
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Admin: require-2FA policy (DQ2; consumed by the e16 admin panel)
// ─────────────────────────────────────────────────────────────────────────────

/// The require-2FA policy routes (registered by `admin_router`).
pub(crate) fn register_admin(r: &mut impl crate::admin_router::AdminRegistrar) {
    use crate::capabilities::Capability;
    r.admin_endpoint(
        axum::http::Method::GET,
        "/2fa/policy",
        Capability::SecurityRead,
        policy_list,
    );
    r.admin_endpoint(
        axum::http::Method::POST,
        "/2fa/policy",
        Capability::TwofaPolicyWrite,
        policy_set,
    );
}

/// `GET /admin/2fa/policy` — the current require-2FA policy rows (admin-gated).
async fn policy_list(State(state): State<AppState>, _ctx: AdminCtx) -> Response {
    match state.store.list_twofa_policies().await {
        Ok(rows) => {
            let out: Vec<_> = rows
                .iter()
                .map(|p| {
                    json!({
                        "scopeKind": p.scope_kind,
                        "scopeValue": p.scope_value,
                        "require2fa": p.require_2fa,
                        "updatedBy": p.updated_by,
                        "updatedAt": p.updated_at,
                    })
                })
                .collect();
            Json(json!({ "policies": out })).into_response()
        }
        Err(e) => server_error("list policy", e),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PolicyReq {
    /// "global" or "domain".
    scope_kind: String,
    #[serde(default)]
    scope_value: String,
    require2fa: bool,
}

/// `POST /admin/2fa/policy` — set (upsert) a require-2FA policy for the global or a
/// per-domain scope (admin-gated, audited).
///
/// While `MW_ADMIN_REQUIRE_2FA` is set it pins the global rule (t29-e6,
/// `mw_admin::EnvOverlay`): a write that would change the global rule answers 409
/// naming the variable, as `PUT /admin/security-policy` does for `require2fa`.
/// Writing the pinned value, and any per-domain rule, is accepted.
async fn policy_set(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Json(body): Json<PolicyReq>,
) -> Response {
    let actor = ctx.username.clone();
    if body.scope_kind != "global" && body.scope_kind != "domain" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "scopeKind must be \"global\" or \"domain\"" })),
        )
            .into_response();
    }
    if body.scope_kind == "global"
        && let Some(pinned) = state.v6.admin.overlay().require_2fa
        && pinned != body.require2fa
    {
        let var = mw_admin::EnvOverlay::REQUIRE_2FA_VAR;
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": format!(
                    "the global two-factor rule is set by {var} and cannot be changed here"
                ),
                "variable": var,
            })),
        )
            .into_response();
    }
    // The global scope pins scope_value to ""; a domain scope is normalised lowercase.
    let scope_value = if body.scope_kind == "global" {
        String::new()
    } else {
        body.scope_value.trim().to_ascii_lowercase()
    };
    if body.scope_kind == "domain" && scope_value.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "a domain-scoped policy needs a scopeValue" })),
        )
            .into_response();
    }
    let row = TwofaPolicyRow {
        scope_kind: body.scope_kind.clone(),
        scope_value: scope_value.clone(),
        require_2fa: body.require2fa,
        updated_by: actor.clone(),
        updated_at: String::new(),
    };
    if let Err(e) = state.store.set_twofa_policy(&row).await {
        return server_error("set policy", e);
    }
    audit(
        &state,
        &actor,
        "twofa-policy-set",
        &format!("{}:{scope_value}", body.scope_kind),
        json!({ "require2fa": body.require2fa }),
    )
    .await;
    Json(json!({ "ok": true })).into_response()
}

/// Append an admin audit line (mirrors `admin_maintenance::audit`); non-fatal.
async fn audit(
    state: &AppState,
    actor: &str,
    action: &str,
    target: &str,
    detail: serde_json::Value,
) {
    let entry = mw_admin::AuditLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: actor.to_string(),
        actor_kind: mw_admin::ActorKind::Admin,
        action: action.to_string(),
        target: Some(target.to_string()),
        detail_json: mw_admin::redact_detail(&detail),
        ip: None,
    };
    if let Err(e) = state.v6.admin.audit(entry).await {
        tracing::warn!("twofa policy audit append failed: {e}");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Router
// ─────────────────────────────────────────────────────────────────────────────

/// The 2FA + session-management routes. `lib.rs` merges this into the main router;
/// `/api/login/2fa*` are pre-auth (pending-token) and CSRF-exempt like `/api/login`,
/// the `/api/account/*` routes are session-authed and ride the normal CSRF guard.
pub(crate) fn twofa_router() -> Router<AppState> {
    Router::new()
        // Step 2 of login + forced enrolment (pending-token authority).
        .route("/api/login/2fa", post(verify_2fa))
        .route("/api/login/methods", get(login_methods))
        .route("/api/login/passkey/begin", post(passwordless_begin))
        .route("/api/login/passkey/finish", post(passwordless_finish))
        .route(
            "/api/login/2fa/enroll/totp/begin",
            post(login_enroll_totp_begin),
        )
        .route(
            "/api/login/2fa/enroll/totp/confirm",
            post(login_enroll_totp_confirm),
        )
        .route(
            "/api/login/2fa/enroll/passkey/begin",
            post(login_enroll_passkey_begin),
        )
        .route(
            "/api/login/2fa/enroll/passkey/finish",
            post(login_enroll_passkey_finish),
        )
        // Authenticated enrolment / management.
        .route("/api/account/2fa", get(twofa_status))
        .route("/api/account/2fa/totp/begin", post(totp_begin_route))
        .route("/api/account/2fa/totp/confirm", post(totp_confirm_route))
        .route("/api/account/2fa/totp/disable", post(totp_disable_route))
        .route("/api/account/2fa/passkey/begin", post(passkey_begin_route))
        .route(
            "/api/account/2fa/passkey/finish",
            post(passkey_finish_route),
        )
        .route(
            "/api/account/2fa/passkey/remove",
            post(passkey_remove_route),
        )
        .route(
            "/api/account/2fa/recovery/regenerate",
            post(recovery_regenerate_route),
        )
        // Session management (S11).
        .route("/api/account/sessions", get(sessions_list_route))
        .route("/api/account/sessions/revoke", post(sessions_revoke_route))
}

// ─────────────────────────────────────────────────────────────────────────────
// Small helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Current UNIX time (seconds) for TOTP.
fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A fresh 32-byte single-use WebAuthn challenge.
fn random_challenge() -> Vec<u8> {
    let mut b = vec![0u8; TOKEN_BYTES];
    rand::thread_rng().fill_bytes(&mut b);
    b
}

/// A fresh unguessable pending-login token.
fn new_token() -> String {
    let mut b = [0u8; TOKEN_BYTES];
    rand::thread_rng().fill_bytes(&mut b);
    b64(&b)
}

/// A stable non-secret handle for a stored credential id (already base64url of the
/// raw credential id; truncated for compact display/removal without leaking length).
fn credential_handle(credential_id: &str) -> String {
    use sha2::{Digest, Sha256};
    let d = Sha256::digest(credential_id.as_bytes());
    d[..8].iter().map(|b| format!("{b:02x}")).collect()
}

/// base64url-no-pad encode.
fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Decode base64 from a browser ceremony, tolerating url-safe/standard and
/// padded/unpadded forms (`btoa` yields standard-padded; some clients url-encode).
fn decode_b64(s: &str) -> Option<Vec<u8>> {
    use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
    URL_SAFE_NO_PAD
        .decode(s)
        .or_else(|_| URL_SAFE.decode(s))
        .or_else(|_| STANDARD_NO_PAD.decode(s))
        .or_else(|_| STANDARD.decode(s))
        .ok()
}

/// The uniform second-factor failure: one 401 body for every wrong/absent factor and
/// every unknown/expired pending token, so a caller cannot tell which check failed.
fn twofa_unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "second factor required" })),
    )
        .into_response()
}

/// Log a store error and return an opaque 500 (never leaks the internal error).
fn server_error(what: &str, e: mw_store::StoreError) -> Response {
    tracing::error!("2fa: {what} failed: {e}");
    (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passkey_user_handle_is_account_bound_and_required_for_discovery() {
        let encoded = b64(&passkey_user_handle("alice@example.test"));
        assert_eq!(passkey_user_handle("alice@example.test").len(), 32);
        assert!(passkey_handle_matches(
            "alice@example.test",
            Some(&encoded),
            true
        ));
        assert!(!passkey_handle_matches(
            "bob@example.test",
            Some(&encoded),
            true
        ));
        assert!(!passkey_handle_matches("alice@example.test", None, true));
        assert!(passkey_handle_matches("alice@example.test", None, false));
        assert!(!passkey_handle_matches(
            "alice@example.test",
            Some(""),
            true
        ));
        assert!(!passkey_handle_matches(
            "alice@example.test",
            Some("!invalid"),
            false
        ));
        let legacy = b64(b"alice@example.test");
        assert!(passkey_handle_matches(
            "alice@example.test",
            Some(&legacy),
            true
        ));
        assert!(!passkey_handle_matches(
            "bob@example.test",
            Some(&legacy),
            false
        ));
    }

    fn args() -> SessionArgs {
        SessionArgs {
            authentication_method: "password",
            primary_identity: crate::primary_identity::VerifiedLoginIdentity::proxy(
                "https://fixture.example",
                "a@ex",
            )
            .unwrap(),
            account_id: "a@ex".into(),
            username: "a@ex".into(),
            jmap_url: "engine".into(),
            api_url: "engine".into(),
            creds: Credentials {
                username: "a@ex".into(),
                password: "pw".into(),
            },
            client_type: None,
        }
    }

    fn pending(kind: PendingKind) -> PendingLogin {
        PendingLogin {
            kind,
            args: args(),
            device: mw_store::SessionDevice::default(),
            max_attempts: MAX_ATTEMPTS,
            challenge: vec![1, 2, 3, 4],
            origin: "https://mail.ex".into(),
            rp_id: "mail.ex".into(),
            attempts: 0,
            expires_at: Instant::now() + PENDING_TTL,
        }
    }

    #[test]
    fn pending_is_single_use_on_success() {
        let st = TwofaState::new();
        st.put_pending("tok".into(), pending(PendingKind::Verify));
        // peek does not consume …
        assert!(st.peek_pending("tok").is_some());
        assert!(st.peek_pending("tok").is_some());
        // … take consumes exactly once (the challenge cannot be replayed).
        assert!(st.take_pending("tok").is_some());
        assert!(st.take_pending("tok").is_none());
        assert!(st.peek_pending("tok").is_none());
    }

    #[test]
    fn expired_pending_is_rejected() {
        let st = TwofaState::new();
        let mut p = pending(PendingKind::Verify);
        p.expires_at = Instant::now() - Duration::from_secs(1);
        st.put_pending("old".into(), p);
        assert!(st.peek_pending("old").is_none());
        assert!(st.take_pending("old").is_none());
    }

    #[test]
    fn attempt_cap_invalidates_pending() {
        let st = TwofaState::new();
        st.put_pending("tok".into(), pending(PendingKind::Verify));
        for _ in 0..(MAX_ATTEMPTS - 1) {
            assert!(!st.note_failure("tok"));
            assert!(st.peek_pending("tok").is_some());
        }
        // The capped attempt drops the entry: no further tries are possible.
        assert!(st.note_failure("tok"));
        assert!(st.peek_pending("tok").is_none());
    }

    #[test]
    fn reg_challenge_is_single_use() {
        let st = TwofaState::new();
        st.put_reg(
            "a@ex".into(),
            RegChallenge {
                discoverable: false,
                challenge: vec![9, 9],
                origin: "https://mail.ex".into(),
                rp_id: "mail.ex".into(),
                expires_at: Instant::now() + PENDING_TTL,
            },
        );
        assert!(st.take_reg("a@ex").is_some());
        assert!(st.take_reg("a@ex").is_none());
    }

    #[test]
    fn decode_b64_tolerates_url_safe_and_standard() {
        // 0xFB 0xFF encodes with the url-safe/standard alphabet difference (`-_` vs `+/`).
        let raw = vec![0xfb_u8, 0xff, 0x00, 0x10];
        let std = base64::engine::general_purpose::STANDARD.encode(&raw);
        let url = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&raw);
        assert_eq!(decode_b64(&std).unwrap(), raw);
        assert_eq!(decode_b64(&url).unwrap(), raw);
        assert!(decode_b64("not valid base64!!").is_none());
    }

    #[test]
    fn credential_handle_is_stable_and_bounded() {
        let h = credential_handle("Y3JlZC1hYmM");
        assert_eq!(h.len(), 16);
        assert_eq!(h, credential_handle("Y3JlZC1hYmM"));
        assert_ne!(h, credential_handle("other"));
    }
}
