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
//! the request Host (same-origin deployment) unless `MW_WEBAUTHN_ORIGIN`/
//! `MW_WEBAUTHN_RP_ID` override it (reverse-proxy deployments).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine as _;
use rand::RngCore;
use serde::Deserialize;
use serde_json::json;

use mw_mfa::totp::TotpParams;
use mw_mfa::webauthn::{
    AssertionRequest, RegistrationRequest, UserVerification, verify_assertion, verify_registration,
};
use mw_mfa::{recovery, totp};
use mw_store::{Credentials, TwofaPolicyRow, WebauthnCredentialRow};

use crate::AppState;

/// How long a pending login (awaiting its second factor / forced enrolment) or a
/// registration challenge stays valid.
const PENDING_TTL: Duration = Duration::from_secs(300);
/// Max second-factor attempts against one pending login before it is invalidated
/// (brute-force cap; a 6-digit TOTP with a ±1 window admits 3 live codes).
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
}

impl TwofaState {
    pub(crate) fn new() -> Self {
        TwofaState {
            pending: Mutex::new(HashMap::new()),
            reg: Mutex::new(HashMap::new()),
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
            if e.attempts >= MAX_ATTEMPTS {
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
pub(crate) async fn gate_login(
    state: &AppState,
    headers: &HeaderMap,
    args: SessionArgs,
) -> Response {
    let store = &state.store;

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

    let required = match policy_requires(state, &args.username).await {
        Ok(r) => r,
        Err(e) => return server_error("read 2fa policy", e),
    };

    // Opt-in user with no factor and no policy requiring one → unchanged path.
    if !enrolled && !required {
        return complete_login(state, &args, json!({})).await;
    }

    let (origin, rp_id) = derive_rp(state.cookie_secure, headers);
    let token = new_token();

    if enrolled {
        let challenge = random_challenge();
        state.twofa.put_pending(
            token.clone(),
            PendingLogin {
                kind: PendingKind::Verify,
                args,
                challenge: challenge.clone(),
                origin,
                rp_id: rp_id.clone(),
                attempts: 0,
                expires_at: Instant::now() + PENDING_TTL,
            },
        );
        let mut factors = Vec::new();
        if totp_enrolled {
            factors.push("totp");
        }
        if !passkeys.is_empty() {
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
                "userVerification": "preferred",
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
                challenge: Vec::new(),
                origin,
                rp_id,
                attempts: 0,
                expires_at: Instant::now() + PENDING_TTL,
            },
        );
        Json(json!({
            "twofaRequired": true,
            "enrollmentRequired": true,
            "pendingToken": token,
            "factors": ["totp", "webauthn"],
        }))
        .into_response()
    }
}

/// Whether a second factor is REQUIRED for `username` by admin policy — the global
/// scope or the user's domain.
async fn policy_requires(state: &AppState, username: &str) -> Result<bool, mw_store::StoreError> {
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
    Ok(false)
}

/// Create the session and finish the login, merging `extra` (e.g. one-time recovery
/// codes) into the success body.
async fn complete_login(
    state: &AppState,
    args: &SessionArgs,
    extra: serde_json::Value,
) -> Response {
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
}

/// Verify the presented second factor for a pending login and, on success, issue the
/// session. A wrong factor returns a uniform 401 and (except after the attempt cap)
/// keeps the pending token live so the user can retry.
async fn verify_2fa(State(state): State<AppState>, Json(body): Json<VerifyReq>) -> Response {
    let Some(pending) = state.twofa.peek_pending(&body.pending_token) else {
        return twofa_unauthorized();
    };
    if pending.kind != PendingKind::Verify {
        return twofa_unauthorized();
    }
    let account_id = &pending.args.account_id;

    let ok = match body.method.as_str() {
        "totp" => verify_totp_factor(&state, account_id, body.code.as_deref()).await,
        "recovery" => verify_recovery_factor(&state, account_id, body.code.as_deref()).await,
        "webauthn" => verify_webauthn_factor(&state, &pending, &body).await,
        _ => Ok(false),
    };
    match ok {
        Ok(true) => {
            // Redeem the pending login exactly once (burns the challenge).
            match state.twofa.take_pending(&body.pending_token) {
                Some(p) => complete_login(&state, &p.args, json!({})).await,
                None => twofa_unauthorized(),
            }
        }
        Ok(false) => {
            state.twofa.note_failure(&body.pending_token);
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
) -> Result<bool, mw_store::StoreError> {
    let (Some(cred_id), Some(cdj), Some(auth_data), Some(sig)) = (
        body.credential_id.as_deref(),
        body.client_data_json.as_deref().and_then(decode_b64),
        body.authenticator_data.as_deref().and_then(decode_b64),
        body.signature.as_deref().and_then(decode_b64),
    ) else {
        return Ok(false);
    };
    let Some(cred) = state.store.get_webauthn_credential(cred_id).await? else {
        return Ok(false);
    };
    // The credential MUST belong to the account this login is for.
    if cred.account_id != pending.args.account_id {
        return Ok(false);
    }
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
    match verify_assertion(&req) {
        Ok(outcome) => {
            state
                .store
                .update_webauthn_sign_count(cred_id, i64::from(outcome.new_sign_count))
                .await?;
            Ok(true)
        }
        Err(_) => Ok(false),
    }
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
}

/// Begin forced TOTP enrolment for an Enroll-state pending login.
async fn login_enroll_totp_begin(
    State(state): State<AppState>,
    Json(body): Json<PendingTokenReq>,
) -> Response {
    let Some(p) = enroll_pending(&state, &body.pending_token) else {
        return twofa_unauthorized();
    };
    totp_begin(&state, &p.args.account_id, &p.args.username).await
}

/// Confirm forced TOTP enrolment; on success issue recovery codes AND the session.
async fn login_enroll_totp_confirm(
    State(state): State<AppState>,
    Json(body): Json<PendingTotpConfirmReq>,
) -> Response {
    let Some(p) = enroll_pending(&state, &body.pending_token) else {
        return twofa_unauthorized();
    };
    match confirm_totp_and_seed_recovery(&state, &p.args.account_id, &body.code).await {
        Ok(Some(codes)) => {
            state.twofa.take_pending(&body.pending_token);
            complete_login(&state, &p.args, json!({ "recoveryCodes": codes })).await
        }
        Ok(None) => twofa_unauthorized(),
        Err(e) => server_error("confirm totp", e),
    }
}

/// Begin forced passkey enrolment (registration challenge) for a pending login.
async fn login_enroll_passkey_begin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PendingTokenReq>,
) -> Response {
    let Some(p) = enroll_pending(&state, &body.pending_token) else {
        return twofa_unauthorized();
    };
    passkey_begin(&state, &headers, &p.args.account_id, &p.args.username)
}

/// Finish forced passkey enrolment; on success issue recovery codes AND the session.
async fn login_enroll_passkey_finish(
    State(state): State<AppState>,
    Json(body): Json<PendingPasskeyFinishReq>,
) -> Response {
    let Some(p) = enroll_pending(&state, &body.pending_token) else {
        return twofa_unauthorized();
    };
    match register_passkey(
        &state,
        &p.args.account_id,
        &body.client_data_json,
        &body.attestation_object,
        &body.transports,
        &body.label,
    )
    .await
    {
        Ok(true) => {
            let codes = match seed_recovery_if_empty(&state, &p.args.account_id).await {
                Ok(c) => c,
                Err(e) => return server_error("seed recovery", e),
            };
            state.twofa.take_pending(&body.pending_token);
            complete_login(&state, &p.args, json!({ "recoveryCodes": codes })).await
        }
        Ok(false) => twofa_unauthorized(),
        Err(e) => server_error("register passkey", e),
    }
}

/// A live Enroll-state pending login for `token`, or `None`.
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
    let required = match policy_requires(&state, &session.username).await {
        Ok(r) => r,
        Err(e) => return server_error("read policy", e),
    };
    let passkey_json: Vec<_> = passkeys
        .iter()
        .map(|c| json!({ "handle": credential_handle(&c.credential_id), "label": c.label, "createdAt": c.created_at }))
        .collect();
    Json(json!({
        "totp": totp,
        "passkeys": passkey_json,
        "recoveryRemaining": recovery_remaining,
        "policyRequired": required,
    }))
    .into_response()
}

/// `POST /api/account/2fa/totp/begin` — start TOTP enrolment.
async fn totp_begin_route(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
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
    let account_id = &session.account_id;
    let passkeys = match state.store.list_webauthn_credentials(account_id).await {
        Ok(v) => v.len(),
        Err(e) => return server_error("list passkeys", e),
    };
    if let Err(r) = guard_last_factor(
        &state,
        &session.username,
        passkeys, /*remaining factors*/
    )
    .await
    {
        return r;
    }
    if let Err(e) = state.store.delete_totp_secret(account_id).await {
        return server_error("delete totp", e);
    }
    Json(json!({ "ok": true })).into_response()
}

/// `POST /api/account/2fa/passkey/begin` — issue a registration challenge.
async fn passkey_begin_route(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    passkey_begin(&state, &headers, &session.account_id, &session.username)
}

/// `POST /api/account/2fa/passkey/finish` — verify + store a registered passkey.
async fn passkey_finish_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PasskeyFinishReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    match register_passkey(
        &state,
        &session.account_id,
        &body.client_data_json,
        &body.attestation_object,
        &body.transports,
        &body.label,
    )
    .await
    {
        Ok(true) => {
            let codes = match seed_recovery_if_empty(&state, &session.account_id).await {
                Ok(c) => c,
                Err(e) => return server_error("seed recovery", e),
            };
            Json(json!({ "ok": true, "recoveryCodes": codes })).into_response()
        }
        Ok(false) => (
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
    let account_id = &session.account_id;
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
    if let Err(r) = guard_last_factor(&state, &session.username, remaining).await {
        return r;
    }
    if let Err(e) = state
        .store
        .delete_webauthn_credential(&cred.credential_id)
        .await
    {
        return server_error("delete passkey", e);
    }
    Json(json!({ "ok": true })).into_response()
}

/// `POST /api/account/2fa/recovery/regenerate` — issue a fresh set of recovery
/// codes (invalidating the old set), shown once.
async fn recovery_regenerate_route(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let account_id = &session.account_id;
    let codes = recovery::generate_codes(recovery::DEFAULT_RECOVERY_CODES);
    let hashes: Vec<String> = codes.iter().map(|c| recovery::hash_code(c)).collect();
    if let Err(e) = state.store.clear_recovery_codes(account_id).await {
        return server_error("clear recovery", e);
    }
    if let Err(e) = state.store.add_recovery_codes(account_id, &hashes).await {
        return server_error("add recovery", e);
    }
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
    let metas = match state.store.list_session_meta(&session.account_id).await {
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
                "current": m.handle == current,
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
}

/// Generate + store an UNCONFIRMED TOTP secret and return the base32 secret +
/// `otpauth://` URI for the authenticator app.
async fn totp_begin(state: &AppState, account_id: &str, username: &str) -> Response {
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
fn passkey_begin(
    state: &AppState,
    headers: &HeaderMap,
    account_id: &str,
    username: &str,
) -> Response {
    let (origin, rp_id) = derive_rp(state.cookie_secure, headers);
    let challenge = random_challenge();
    state.twofa.put_reg(
        account_id.to_string(),
        RegChallenge {
            challenge: challenge.clone(),
            origin,
            rp_id: rp_id.clone(),
            expires_at: Instant::now() + PENDING_TTL,
        },
    );
    // A stable per-account user handle (WebAuthn user.id) — the account id bytes.
    Json(json!({
        "ok": true,
        "challenge": b64(&challenge),
        "rpId": rp_id,
        "userHandle": b64(account_id.as_bytes()),
        "userName": username,
        "userVerification": "preferred",
    }))
    .into_response()
}

/// Verify a passkey registration ceremony against the stored challenge and persist
/// the credential. `Ok(false)` = the challenge was missing/expired or verification
/// failed; the caller maps that to a 4xx.
async fn register_passkey(
    state: &AppState,
    account_id: &str,
    client_data_json_b64: &str,
    attestation_object_b64: &str,
    transports: &str,
    label: &str,
) -> Result<bool, mw_store::StoreError> {
    let Some(reg) = state.twofa.take_reg(account_id) else {
        return Ok(false);
    };
    let (Some(cdj), Some(att)) = (
        decode_b64(client_data_json_b64),
        decode_b64(attestation_object_b64),
    ) else {
        return Ok(false);
    };
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
        Err(_) => return Ok(false),
    };
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
    state.store.add_webauthn_credential(&row).await?;
    Ok(true)
}

/// Refuse to remove the LAST factor of an account whose domain/global policy
/// requires 2FA (`remaining` = factor count that would survive the removal). An
/// account with no policy requirement may disable freely.
async fn guard_last_factor(
    state: &AppState,
    username: &str,
    remaining: usize,
) -> Result<(), Response> {
    if remaining == 0 {
        let required = policy_requires(state, username)
            .await
            .map_err(|e| server_error("read policy", e))?;
        if required {
            return Err((
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "your organization requires two-factor authentication; enrol another factor before removing this one",
                })),
            )
                .into_response());
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Admin: require-2FA policy (DQ2; consumed by the e16 admin panel)
// ─────────────────────────────────────────────────────────────────────────────

const ADMIN_COOKIE: &str = "mw_admin_session";

/// The authenticated admin id, or `None` (mirrors `admin_maintenance.rs`). Honours
/// the `admin.enabled` gate.
async fn admin_actor(state: &AppState, headers: &HeaderMap) -> Option<String> {
    if !state.v6.admin_enabled {
        return None;
    }
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    let prefix = format!("{ADMIN_COOKIE}=");
    let token = raw
        .split(';')
        .find_map(|p| p.trim().strip_prefix(&prefix).filter(|v| !v.is_empty()))?;
    let hash = crate::push_relay::hash_token(token);
    state.store.get_admin_session(&hash).await.ok().flatten()
}

fn admin_unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "admin authentication required" })),
    )
        .into_response()
}

/// `GET /admin/2fa/policy` — the current require-2FA policy rows (admin-gated).
async fn policy_list(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if admin_actor(&state, &headers).await.is_none() {
        return admin_unauthorized();
    }
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
async fn policy_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PolicyReq>,
) -> Response {
    let Some(actor) = admin_actor(&state, &headers).await else {
        return admin_unauthorized();
    };
    if body.scope_kind != "global" && body.scope_kind != "domain" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "scopeKind must be \"global\" or \"domain\"" })),
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
        // Admin require-2FA policy (DQ2; consumed by the e16 admin panel).
        .route("/admin/2fa/policy", get(policy_list).post(policy_set))
}

// ─────────────────────────────────────────────────────────────────────────────
// Small helpers
// ─────────────────────────────────────────────────────────────────────────────

/// The WebAuthn origin (`scheme://host[:port]`) and rp-id (`host`) for this
/// deployment. `MW_WEBAUTHN_ORIGIN`/`MW_WEBAUTHN_RP_ID` override the Host-derived
/// values (needed when a reverse proxy rewrites Host); otherwise the same-origin
/// deployment's Host header is authoritative.
fn derive_rp(cookie_secure: bool, headers: &HeaderMap) -> (String, String) {
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost")
        .to_string();
    let host_only = host.split(':').next().unwrap_or(&host).to_string();
    let scheme = if cookie_secure { "https" } else { "http" };
    let origin = std::env::var("MW_WEBAUTHN_ORIGIN")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("{scheme}://{host}"));
    let rp_id = std::env::var("MW_WEBAUTHN_RP_ID")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or(host_only);
    (origin, rp_id)
}

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

    fn args() -> SessionArgs {
        SessionArgs {
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
    fn derive_rp_prefers_host_and_scheme() {
        // Env overrides must not leak between tests; assert only the Host-derived path
        // when the override vars are absent.
        if std::env::var("MW_WEBAUTHN_ORIGIN").is_err()
            && std::env::var("MW_WEBAUTHN_RP_ID").is_err()
        {
            let mut h = HeaderMap::new();
            h.insert(header::HOST, "mail.example.com:8443".parse().unwrap());
            let (origin, rp_id) = derive_rp(true, &h);
            assert_eq!(origin, "https://mail.example.com:8443");
            assert_eq!(rp_id, "mail.example.com");

            let (origin_http, _) = derive_rp(false, &h);
            assert_eq!(origin_http, "http://mail.example.com:8443");
        }
    }

    #[test]
    fn credential_handle_is_stable_and_bounded() {
        let h = credential_handle("Y3JlZC1hYmM");
        assert_eq!(h.len(), 16);
        assert_eq!(h, credential_handle("Y3JlZC1hYmM"));
        assert_ne!(h, credential_handle("other"));
    }
}
