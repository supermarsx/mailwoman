//! Administrator-only WebAuthn ceremonies. Every challenge is one-use, bounded,
//! origin-bound and, for enrollment/step-up, bound to the current admin session.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::{Access, AdminRegistrar},
};
use axum::{
    Json,
    extract::State,
    http::{Extensions, HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use mw_mfa::{AssertionRequest, RegistrationRequest, UserVerification, recovery};
use rand::RngCore;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

const TTL: Duration = Duration::from_secs(300);
struct Challenge {
    bytes: Vec<u8>,
    origin: String,
    rp: String,
    account: Option<String>,
    expires: Instant,
}
static CHALLENGES: LazyLock<Mutex<HashMap<String, Challenge>>> = LazyLock::new(Default::default);
fn put(key: String, challenge: Challenge) -> Result<(), AdminApiError> {
    let mut map = CHALLENGES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    map.retain(|_, c| c.expires > Instant::now());
    if map.len() >= 1024 {
        return Err(AdminApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "busy",
            "Too many pending ceremonies.",
        ));
    }
    map.insert(key, challenge);
    Ok(())
}
fn take(key: &str) -> Option<Challenge> {
    CHALLENGES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(key)
        .filter(|c| c.expires > Instant::now())
}
fn random() -> Vec<u8> {
    let mut bytes = vec![0; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}
fn encode(v: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(v)
}
fn decode(v: &str) -> Result<Vec<u8>, AdminApiError> {
    URL_SAFE_NO_PAD
        .decode(v)
        .map_err(|_| AdminApiError::unauthenticated())
}
fn user_handle(id: &str) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update(b"mailwoman.webauthn.admin\0");
    h.update(id);
    h.finalize().to_vec()
}
fn account(ctx: &AdminCtx) -> Result<&str, AdminApiError> {
    if ctx.session_hash.is_empty() {
        return Err(AdminApiError::unauthenticated());
    }
    ctx.account_id
        .as_deref()
        .ok_or_else(AdminApiError::unauthenticated)
}
fn challenge(
    state: &AppState,
    headers: &HeaderMap,
    ext: &Extensions,
    account: Option<String>,
) -> Result<Challenge, AdminApiError> {
    let (origin, rp) =
        crate::step_up::webauthn_rp(&state.external, headers, ext).ok_or_else(|| {
            AdminApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "originUnavailable",
                "Configure a public WebAuthn origin.",
            )
        })?;
    Ok(Challenge {
        bytes: random(),
        origin,
        rp,
        account,
        expires: Instant::now() + TTL,
    })
}
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/me/passkeys", Access::SignedIn, list);
    r.admin_endpoint(
        Method::POST,
        "/me/passkeys/begin",
        Access::SignedIn,
        enroll_begin,
    );
    r.admin_endpoint(
        Method::POST,
        "/me/passkeys/finish",
        Access::SignedIn,
        enroll_finish,
    );
    r.admin_endpoint(
        Method::POST,
        "/me/passkeys/remove",
        Access::SignedIn,
        remove,
    );
    r.admin_endpoint(
        Method::POST,
        "/me/recovery",
        Access::SignedIn,
        rotate_recovery,
    );
    r.admin_endpoint(
        Method::POST,
        "/login/passkey/begin",
        Access::Public,
        login_begin,
    );
    r.admin_endpoint(
        Method::POST,
        "/login/passkey/finish",
        Access::Public,
        login_finish,
    );
    r.admin_endpoint(
        Method::POST,
        "/login/recovery",
        Access::Public,
        login_recovery,
    );
    r.admin_endpoint(
        Method::POST,
        "/step-up/passkey",
        Access::SignedIn,
        stepup_begin,
    );
}
async fn list(State(state): State<AppState>, ctx: AdminCtx) -> Result<Json<Value>, AdminApiError> {
    let keys = state.store.list_admin_passkeys(account(&ctx)?).await?;
    Ok(Json(
        json!({"passkeys":keys.iter().map(|k|json!({"credentialId":k.credential.credential_id,"label":k.credential.label,"createdAt":k.credential.created_at,"backupEligible":k.state.backup_eligible,"backedUp":k.state.backup_state})).collect::<Vec<_>>(),"recoveryRemaining":state.store.admin_recovery_hashes(account(&ctx)?).await?.len()}),
    ))
}
async fn enroll_begin(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    ext: Extensions,
) -> Result<Json<Value>, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::TwoFactorDisable)
        .await?;
    let id = account(&ctx)?;
    let c = challenge(&state, &headers, &ext, Some(id.into()))?;
    let keys = state.store.list_admin_passkeys(id).await?;
    let body = json!({"challenge":encode(&c.bytes),"rpId":c.rp,"userHandle":encode(&user_handle(id)),"userName":ctx.username,"residentKey":"required","userVerification":"required","excludeCredentialIds":keys.iter().map(|k|&k.credential.credential_id).collect::<Vec<_>>()});
    put(format!("register:{}", ctx.session_hash), c)?;
    Ok(Json(body))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Registration {
    client_data_json: String,
    attestation_object: String,
    #[serde(default)]
    transports: String,
    #[serde(default)]
    label: String,
}
async fn enroll_finish(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(body): AdminJson<Registration>,
) -> Result<Json<Value>, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::TwoFactorDisable)
        .await?;
    let id = account(&ctx)?;
    let c = take(&format!("register:{}", ctx.session_hash))
        .filter(|c| c.account.as_deref() == Some(id))
        .ok_or_else(AdminApiError::unauthenticated)?;
    if body.label.len() > 128 || body.transports.len() > 256 {
        return Err(AdminApiError::invalid(
            "tooLong",
            "Passkey label or transports too long.",
            "/label",
        ));
    }
    let key = mw_mfa::verify_registration(&RegistrationRequest {
        challenge: c.bytes,
        origin: c.origin,
        rp_id: c.rp,
        client_data_json: decode(&body.client_data_json)?,
        attestation_object: decode(&body.attestation_object)?,
        user_verification: UserVerification::Required,
    })
    .map_err(|_| AdminApiError::unauthenticated())?;
    let credential = mw_store::AdminPasskey {
        credential: mw_store::WebauthnCredentialRow {
            credential_id: encode(&key.credential_id),
            account_id: id.into(),
            cose_public_key: key.cose_public_key,
            sign_count: i64::from(key.sign_count),
            transports: body.transports,
            label: body.label,
            created_at: String::new(),
        },
        state: mw_store::WebauthnState {
            backup_eligible: Some(key.backup_eligible),
            backup_state: key.backup_state,
            uv_initialized: key.user_verified,
            revision: 0,
        },
    };
    let codes = recovery::generate_codes(recovery::DEFAULT_RECOVERY_CODES);
    let hashes = codes
        .iter()
        .map(|code| recovery::hash_code(code))
        .collect::<Vec<_>>();
    let (inserted, seeded) = state
        .store
        .register_admin_passkey(&credential, &hashes)
        .await?;
    if !inserted {
        return Err(AdminApiError::new(
            StatusCode::CONFLICT,
            "credentialExists",
            "Credential already registered or passkey limit reached.",
        ));
    }
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "admin-passkey-enrolled",
        &format!("admin:{id}"),
        None,
        None,
    )
    .await;
    Ok(Json(
        json!({"ok":true,"recoveryCodes":if seeded {codes} else {vec![]}}),
    ))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Remove {
    credential_id: String,
}
async fn remove(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(body): AdminJson<Remove>,
) -> Result<Json<Value>, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::TwoFactorDisable)
        .await?;
    if !state
        .store
        .delete_admin_passkey(account(&ctx)?, &body.credential_id)
        .await?
    {
        return Err(AdminApiError::new(
            StatusCode::NOT_FOUND,
            "notFound",
            "Passkey not found.",
        ));
    }
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "admin-passkey-removed",
        &format!("admin:{}", account(&ctx)?),
        None,
        None,
    )
    .await;
    Ok(Json(json!({"ok":true})))
}
async fn rotate_recovery(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::TwoFactorDisable)
        .await?;
    if state
        .store
        .list_admin_passkeys(account(&ctx)?)
        .await?
        .is_empty()
        && state.store.get_admin_totp(account(&ctx)?).await?.is_none()
    {
        return Err(AdminApiError::new(
            StatusCode::CONFLICT,
            "noSecondFactor",
            "Enroll a passkey or authenticator first.",
        ));
    }
    let codes = recovery::generate_codes(recovery::DEFAULT_RECOVERY_CODES);
    let hashes = codes
        .iter()
        .map(|c| recovery::hash_code(c))
        .collect::<Vec<_>>();
    state
        .store
        .replace_admin_recovery(account(&ctx)?, &hashes)
        .await?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "admin-recovery-replaced",
        &format!("admin:{}", account(&ctx)?),
        None,
        None,
    )
    .await;
    Ok((
        [(axum::http::header::CACHE_CONTROL, "no-store")],
        Json(json!({"recoveryCodes":codes})),
    )
        .into_response())
}
async fn login_begin(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: Extensions,
) -> Result<Json<Value>, AdminApiError> {
    if !state.v6.admin_enabled {
        return Err(AdminApiError::unauthenticated());
    }
    crate::admin_accounts::check_login_source(
        &state,
        crate::scope_mw::proxy::client_ip(&headers, &ext),
    )
    .await?;
    let c = challenge(&state, &headers, &ext, None)?;
    let token = encode(&random());
    let body = json!({"challenge":encode(&c.bytes),"rpId":c.rp,"credentialIds":[],"userVerification":"required","pendingToken":token});
    put(token, c)?;
    Ok(Json(body))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Assertion {
    pub credential_id: String,
    pub client_data_json: String,
    pub authenticator_data: String,
    pub signature: String,
    #[serde(default)]
    pub user_handle: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginFinish {
    pending_token: String,
    #[serde(flatten)]
    assertion: Assertion,
}
async fn verify(
    state: &AppState,
    c: Challenge,
    proof: &Assertion,
    required_handle: bool,
) -> Result<Option<mw_store::AdminPasskey>, AdminApiError> {
    let Some(key) = state.store.get_admin_passkey(&proof.credential_id).await? else {
        return Ok(None);
    };
    let id = &key.credential.account_id;
    if c.account.as_ref().is_some_and(|expected| expected != id) {
        return Ok(None);
    }
    match proof.user_handle.as_deref() {
        Some(handle) if decode(handle).ok().as_deref() == Some(user_handle(id).as_slice()) => {}
        None if !required_handle => {}
        _ => return Ok(None),
    }
    let result = mw_mfa::verify_assertion_with_backup(
        &AssertionRequest {
            challenge: c.bytes,
            origin: c.origin,
            rp_id: c.rp,
            client_data_json: decode(&proof.client_data_json)?,
            authenticator_data: decode(&proof.authenticator_data)?,
            signature: decode(&proof.signature)?,
            cose_public_key: key.credential.cose_public_key.clone(),
            stored_sign_count: key.credential.sign_count as u32,
            user_verification: UserVerification::Required,
        },
        key.state.backup_eligible,
    );
    let Ok(result) = result else { return Ok(None) };
    if !state
        .store
        .advance_admin_passkey(&key, result.backup_state, i64::from(result.new_sign_count))
        .await?
    {
        return Ok(None);
    }
    Ok(Some(key))
}
async fn login_finish(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: Extensions,
    AdminJson(body): AdminJson<LoginFinish>,
) -> Result<Response, AdminApiError> {
    let source = crate::scope_mw::proxy::client_ip(&headers, &ext);
    let result: Result<(mw_store::AdminAccountRow, mw_store::AdminPasskey), AdminApiError> =
        async {
            if !state.v6.admin_enabled {
                return Err(AdminApiError::unauthenticated());
            }
            crate::admin_accounts::check_login_source(&state, source).await?;
            let c = take(&body.pending_token)
                .filter(|c| c.account.is_none())
                .ok_or_else(AdminApiError::unauthenticated)?;
            let key = verify(&state, c, &body.assertion, true)
                .await?
                .ok_or_else(AdminApiError::unauthenticated)?;
            let account = state
                .store
                .get_admin_account(&key.credential.account_id)
                .await?
                .filter(|a| {
                    !a.disabled
                        && (a.source != "env-bootstrap"
                            || crate::admin_accounts::env_login_enabled())
                })
                .ok_or_else(AdminApiError::unauthenticated)?;
            Ok((account, key))
        }
        .await;
    let (acc, key) = match result {
        Ok(account) => account,
        Err(error) => {
            crate::admin_accounts::record_admin_login_failure(&state, "passkey", source).await;
            crate::auth_logs::record_admin(&state, "", source, "passkey", false).await;
            return Err(error);
        }
    };
    let response = crate::admin_accounts::start_session_for_factor(
        &state,
        &acc.username,
        Some(mw_store::AdminSessionFactor::Passkey(&key)),
    )
    .await;
    if response.status().is_success() {
        crate::admin_accounts::record_login_success(&state, &acc.username, source).await;
    }
    crate::auth_logs::record_admin(
        &state,
        &acc.username,
        source,
        "passkey",
        response.status().is_success(),
    )
    .await;
    Ok(response)
}
async fn stepup_begin(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    ext: Extensions,
) -> Result<Json<Value>, AdminApiError> {
    let id = account(&ctx)?;
    let keys = state.store.list_admin_passkeys(id).await?;
    if keys.is_empty() {
        return Err(AdminApiError::new(
            StatusCode::CONFLICT,
            "noPasskey",
            "No passkey is enrolled.",
        ));
    }
    let c = challenge(&state, &headers, &ext, Some(id.into()))?;
    let body = json!({"challenge":encode(&c.bytes),"rpId":c.rp,"credentialIds":keys.iter().map(|k|&k.credential.credential_id).collect::<Vec<_>>(),"userVerification":"required"});
    put(format!("step:{}", ctx.session_hash), c)?;
    Ok(Json(body))
}
pub(crate) async fn verify_stepup(
    state: &AppState,
    ctx: &AdminCtx,
    proof: &Assertion,
) -> Result<bool, AdminApiError> {
    let Some(c) = take(&format!("step:{}", ctx.session_hash)) else {
        return Ok(false);
    };
    Ok(verify(state, c, proof, false)
        .await?
        .as_ref()
        .map(|key| key.credential.account_id.as_str())
        == Some(account(ctx)?))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecoveryLogin {
    pending_token: String,
    code: String,
}
async fn login_recovery(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: Extensions,
    AdminJson(body): AdminJson<RecoveryLogin>,
) -> Result<Response, AdminApiError> {
    let source = crate::scope_mw::proxy::client_ip(&headers, &ext);
    let pending = async {
        crate::admin_accounts::check_login_source(&state, source).await?;
        crate::admin_accounts::take_password_pending(&state, &body.pending_token).await
    }
    .await;
    let account = match pending {
        Ok(account) => account,
        Err(error) => {
            crate::auth_logs::record_admin(&state, "", source, "password+recovery", false).await;
            return Err(error);
        }
    };
    let hashes = state.store.admin_recovery_hashes(&account.id).await?;
    for hash in hashes {
        if recovery::verify_code(&body.code, &hash)
            && state
                .store
                .consume_admin_recovery(&account.id, &hash)
                .await?
        {
            let response = crate::admin_accounts::start_session_for_factor(
                &state,
                &account.username,
                Some(mw_store::AdminSessionFactor::Recovery {
                    account: &account.id,
                    hash: &hash,
                }),
            )
            .await;
            if response.status().is_success() {
                crate::admin_accounts::record_login_success(&state, &account.username, source)
                    .await;
            }
            crate::auth_logs::record_admin(
                &state,
                &account.username,
                source,
                "password+recovery",
                response.status().is_success(),
            )
            .await;
            return Ok(response);
        }
    }
    crate::admin_accounts::record_admin_login_failure(&state, &account.username, source).await;
    crate::auth_logs::record_admin(
        &state,
        &account.username,
        source,
        "password+recovery",
        false,
    )
    .await;
    Err(AdminApiError::unauthenticated())
}
