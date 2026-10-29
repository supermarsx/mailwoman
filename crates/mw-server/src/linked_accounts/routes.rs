use super::*;
use axum::{
    Router,
    extract::{Path, State},
    http::Extensions,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/accounts", get(list))
        .route("/api/accounts/link", post(link))
        .route("/api/accounts/primary", axum::routing::put(set_primary))
        .route("/api/accounts/layout", axum::routing::put(set_layout))
        .route("/api/accounts/challenges/{id}/confirm", post(confirm))
        .route("/api/accounts/{id}/renew", post(renew))
        .route(
            "/api/accounts/{id}/renew/challenges/{challenge}/confirm",
            post(confirm_renew),
        )
        .route("/api/accounts/{id}/switch", post(switch))
        .route("/api/accounts/{id}", axum::routing::delete(unlink))
}
async fn parent(
    state: &AppState,
    headers: &HeaderMap,
    mutation: bool,
) -> Result<mw_store::Session, Response> {
    if headers.contains_key(SELECTOR) {
        return Err(failure(StatusCode::FORBIDDEN, "primaryContextRequired"));
    }
    let session = crate::authed(state, headers).await?;
    crate::prefs_routes::personal_storage::owner(state, &session, headers, mutation).await?;
    Ok(session)
}
async fn entry(
    state: &AppState,
    parent: &mw_store::Session,
    row: &mw_store::linked_accounts::LinkedAccount,
) -> Result<Value, Response> {
    let mut value = serde_json::to_value(row).map_err(unavailable)?;
    value["status"] = json!("reauthRequired");
    if let Some(login) = sign_in_name(state, parent, row).await {
        value["address"] = json!(login);
    }
    // t48: a lapsed child session is renewed here from the sealed credential.
    let found = match super::credential::live_source(state, parent, &row.id).await {
        Ok(found) => found,
        Err(response) if response.status() == StatusCode::SERVICE_UNAVAILABLE => {
            value["status"] = json!("disabled");
            None
        }
        Err(response) => return Err(response),
    };
    if let Some((_, source)) = found {
        value["address"] = json!(source.username);
        {
            let gate = crate::account_gate::for_session(&state.store, &source)
                .await
                .map_err(unavailable)?;
            let permitted = enabled(state, parent).await?
                && enabled(state, &source).await?
                && !gate.disabled
                && !gate.password_change_required;
            value["status"] = json!(if permitted { "ready" } else { "disabled" });
            if permitted {
                value["storageAuthority"] = json!(
                    state
                        .store
                        .session_storage_authority(&source.id)
                        .await
                        .map_err(unavailable)?
                );
            }
        }
    }
    // t48: whether access renews by itself, and if not why (codes only), read
    // after any renewal attempt above. The secret is never in a response.
    let (renewal, reason) = super::credential::renewal(state, parent, &row.id).await?;
    value["renewal"] = renewal;
    value["renewalReason"] = reason;
    Ok(value)
}
async fn list(State(state): State<AppState>, headers: HeaderMap) -> Result<Response, Response> {
    let parent = parent(&state, &headers, false).await?;
    let rows = state
        .store
        .linked_accounts(&parent.id)
        .await
        .map_err(unavailable)?;
    let mut accounts = Vec::with_capacity(rows.len());
    for row in rows {
        accounts.push(entry(&state, &parent, &row).await?);
    }
    // The stored first mailbox counts only while it is still one of these links
    // and ready to open; otherwise the primary account opens and the client is
    // told once (it clears the mark after telling the user).
    let stored = state
        .store
        .linked_primary(&parent.id)
        .await
        .map_err(unavailable)?;
    let primary_mailbox = stored.as_deref().filter(|id| {
        accounts
            .iter()
            .any(|a| a["id"] == json!(id) && a["status"] == json!("ready"))
    });
    let primary_fallback = stored.is_some() && primary_mailbox.is_none();
    let (store_credentials, store_reason) =
        super::credential::owner_policy(&state, &parent).await?;
    let keys = layout_keys(accounts.iter().filter_map(|a| a["id"].as_str()));
    let layout = layout_view(
        state
            .store
            .linked_layout(&parent.id)
            .await
            .map_err(unavailable)?,
        &keys,
    );
    Ok(([(axum::http::header::CACHE_CONTROL,"private, no-store")],Json(json!({
        "primaryMailbox":primary_mailbox,"primaryFallback":primary_fallback,"layout":layout,
        "primary":{"accountId":parent.account_id,"address":parent.username,"storageAuthority":state.store.session_storage_authority(&parent.id).await.map_err(unavailable)?},
        "accounts":accounts,"policy":{"enabled":enabled(&state,&parent).await?,"maxAccounts":maximum(&state).await?,
        "supportedAuthentication":["password","totp","recovery","passkeySecondFactor"],"supportedBackends":["imap","pop3","ews"],
        "reason":if supported(&state,&parent).await?{Value::Null}else{json!("linkedBackendUnsupported")},"childRealtime":true,
        "storeCredentials":store_credentials,"storeCredentialsReason":store_reason}
    }))).into_response())
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LinkRequest {
    username: String,
    password: String,
    altcha: Option<String>,
    display_name: Option<String>,
}

async fn link(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut ext: Extensions,
    Json(body): Json<LinkRequest>,
) -> Result<Response, Response> {
    let parent = parent(&state, &headers, true).await?;
    require_enabled(&state, &parent).await?;
    crate::step_up::require_user(
        &state,
        &parent,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    if state
        .store
        .linked_accounts(&parent.id)
        .await
        .map_err(unavailable)?
        .len()
        >= maximum(&state).await?
    {
        return Err(failure(StatusCode::CONFLICT, "linkedAccountLimit"));
    }
    let display = body.display_name.unwrap_or_else(|| body.username.clone());
    if display.trim().is_empty()
        || display.chars().count() > 128
        || display.chars().any(char::is_control)
        || body.username.len() > 1024
        || body.password.len() > 16384
    {
        return Err(failure(StatusCode::BAD_REQUEST, "invalidLinkedAccount"));
    }
    // Do not let an edge's header-auth assertion prove the typed child, nor let a
    // primary remembered-device cookie stand in for the child's factor.
    let mut login_headers = headers.clone();
    login_headers.remove(axum::http::header::COOKIE);
    login_headers.remove(axum::http::header::AUTHORIZATION);
    ext.insert(PasswordProof);
    let response = crate::login(
        State(state.clone()),
        login_headers,
        ext,
        Json(crate::LoginReq {
            username: body.username,
            password: body.password,
            altcha: body.altcha,
            client_type: Some("native".into()),
            other: Default::default(),
        }),
    )
    .await;
    finish(&state, &parent, &display, response, None, None).await
}

async fn confirm(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: Extensions,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Response, Response> {
    let parent = parent(&state, &headers, true).await?;
    require_enabled(&state, &parent).await?;
    crate::step_up::require_user(
        &state,
        &parent,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    let Some((pending, display)) = state
        .store
        .linked_challenge(&parent.id, &id, chrono::Utc::now().timestamp())
        .await
        .map_err(unavailable)?
    else {
        return Err(failure(StatusCode::GONE, "linkedChallengeExpired"));
    };
    let mut factor_headers = headers;
    factor_headers.remove(axum::http::header::COOKIE);
    factor_headers.remove(axum::http::header::AUTHORIZATION);
    let response =
        crate::twofa_routes::verify_link_factor(state.clone(), factor_headers, ext, pending, body)
            .await;
    finish(&state, &parent, &display, response, Some(&id), None).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RenewRequest {
    /// Optional: the server knows the sign-in name of every link it can renew
    /// (t70), so only the password is needed. A name typed here is used
    /// instead; the sign-in must still be the mailbox the link was made for.
    username: Option<String>,
    password: String,
    altcha: Option<String>,
}

/// `POST /api/accounts/{id}/renew` — sign the linked mailbox in again and keep
/// the link: same link id, revision, order, nickname and primary mark. The
/// sign-in must be the very mailbox the link was made for. No unlink needed.
async fn renew(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut ext: Extensions,
    Path(id): Path<String>,
    Json(body): Json<RenewRequest>,
) -> Result<Response, Response> {
    let parent = parent(&state, &headers, true).await?;
    require_enabled(&state, &parent).await?;
    let Some(row) = state
        .store
        .linked_accounts(&parent.id)
        .await
        .map_err(unavailable)?
        .into_iter()
        .find(|row| row.id == id)
    else {
        return Err(failure(StatusCode::NOT_FOUND, "linkedAccountNotFound"));
    };
    let typed = body
        .username
        .as_deref()
        .map(str::trim)
        .filter(|typed| !typed.is_empty())
        .map(str::to_string);
    let username = match typed {
        Some(typed) => typed,
        None => match sign_in_name(&state, &parent, &row).await {
            Some(known) => known,
            None => return Err(failure(StatusCode::BAD_REQUEST, "linkedUsernameRequired")),
        },
    };
    if username.len() > 1024 || body.password.is_empty() || body.password.len() > 16384 {
        return Err(failure(StatusCode::BAD_REQUEST, "invalidLinkedAccount"));
    }
    // t70: the normal sign-in limits key on the client address; this one keys
    // on the owner and the link, so one link cannot be guessed at from many
    // addresses either. Each refusal is audited without the password.
    if let Err(retry) = renew_attempt(&parent.account_id, &row.id) {
        renew_failed(&state, &parent, &row.id, "rateLimited").await;
        let mut response = failure(StatusCode::TOO_MANY_REQUESTS, "linkedRenewLimit");
        response.headers_mut().insert(
            axum::http::header::RETRY_AFTER,
            axum::http::HeaderValue::from(retry),
        );
        return Err(response);
    }
    let mut login_headers = headers.clone();
    login_headers.remove(axum::http::header::COOKIE);
    login_headers.remove(axum::http::header::AUTHORIZATION);
    ext.insert(PasswordProof);
    let response = crate::login(
        State(state.clone()),
        login_headers,
        ext,
        Json(crate::LoginReq {
            username,
            password: body.password,
            altcha: body.altcha,
            client_type: Some("native".into()),
            other: Default::default(),
        }),
    )
    .await;
    // t70: a refused password and a server that could not be reached are told
    // apart here (the owner is signed in; the public login keeps one answer).
    if let Some((code, status)) = renew_refusal(&response) {
        renew_failed(&state, &parent, &row.id, code).await;
        return Err(failure(status, code));
    }
    let result = finish(
        &state,
        &parent,
        &row.display_name,
        response,
        None,
        Some(&row.id),
    )
    .await;
    match &result {
        Ok(_) => renew_succeeded(&parent.account_id, &row.id),
        Err(response) => renew_failed(&state, &parent, &row.id, &error_code(response)).await,
    }
    result
}

/// t70: the name a link signs in with when renewed with only its password:
/// the one kept with its sealed credential, or else the sign-in name of the
/// linked account (links made before t48, or whose credential row was
/// removed). Only the owner's own links reach here.
async fn sign_in_name(
    state: &AppState,
    parent: &mw_store::Session,
    row: &mw_store::linked_accounts::LinkedAccount,
) -> Option<String> {
    if let Some(login) = super::credential::login_name(state, parent, &row.id).await {
        return Some(login);
    }
    state
        .store
        .get_account(&row.account_id)
        .await
        .ok()
        .map(|account| account.username)
        .filter(|name| !name.trim().is_empty())
}

/// Renew attempts allowed per owner and link in [`RENEW_WINDOW`] (per process).
const RENEW_ATTEMPTS: u32 = 10;
const RENEW_WINDOW: std::time::Duration = std::time::Duration::from_secs(15 * 60);
type RenewWindows = HashMap<String, (std::time::Instant, u32)>;
static RENEW_LIMIT: std::sync::LazyLock<std::sync::Mutex<RenewWindows>> =
    std::sync::LazyLock::new(Default::default);

fn renew_key(owner: &str, link: &str) -> String {
    format!("{owner}\u{0}{link}")
}
/// Count one attempt; `Err(seconds)` once the window is used up.
fn renew_attempt(owner: &str, link: &str) -> Result<(), u64> {
    let now = std::time::Instant::now();
    let mut windows = RENEW_LIMIT.lock().unwrap_or_else(|e| e.into_inner());
    windows.retain(|_, (start, _)| now.duration_since(*start) < RENEW_WINDOW);
    let entry = windows.entry(renew_key(owner, link)).or_insert((now, 0));
    if entry.1 >= RENEW_ATTEMPTS {
        let left = RENEW_WINDOW.saturating_sub(now.duration_since(entry.0));
        return Err(left.as_secs().max(1));
    }
    entry.1 += 1;
    Ok(())
}
fn renew_succeeded(owner: &str, link: &str) {
    RENEW_LIMIT
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(&renew_key(owner, link));
}

/// A failed sign-in as `(code, status)`: the password was refused, or the
/// mailbox's server could not be reached. Anything else goes through [`finish`].
fn renew_refusal(response: &Response) -> Option<(&'static str, StatusCode)> {
    use crate::login_failure::LoginFailureReason as Reason;
    let reason = response.extensions().get::<Reason>().copied();
    let unreachable = matches!(
        reason,
        Some(
            Reason::Dns
                | Reason::Connect
                | Reason::Timeout
                | Reason::Transport
                | Reason::Tls
                | Reason::Certificate
        )
    );
    if unreachable || response.status() == StatusCode::GATEWAY_TIMEOUT {
        return Some(("linkedServerUnreachable", StatusCode::BAD_GATEWAY));
    }
    (response.status() == StatusCode::UNAUTHORIZED)
        .then_some(("linkedPasswordRejected", StatusCode::UNAUTHORIZED))
}
fn error_code(response: &Response) -> String {
    match response.status() {
        StatusCode::TOO_MANY_REQUESTS => "tooManyAttempts".into(),
        StatusCode::CONFLICT => "conflict".into(),
        status => format!("http{}", status.as_u16()),
    }
}
/// Audit a failed renew: the link and a reason code, never the password or
/// the name typed.
async fn renew_failed(state: &AppState, parent: &mw_store::Session, link: &str, reason: &str) {
    super::credential::audit(
        state,
        &parent.account_id,
        &parent.account_id,
        "linked-access-renew-failed",
        json!({"link":link,"reason":reason}),
    )
    .await;
}

async fn confirm_renew(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: Extensions,
    Path((id, challenge)): Path<(String, String)>,
    Json(body): Json<Value>,
) -> Result<Response, Response> {
    let parent = parent(&state, &headers, true).await?;
    require_enabled(&state, &parent).await?;
    let Some((pending, display)) = state
        .store
        .linked_challenge(&parent.id, &challenge, chrono::Utc::now().timestamp())
        .await
        .map_err(unavailable)?
    else {
        return Err(failure(StatusCode::GONE, "linkedChallengeExpired"));
    };
    let mut factor_headers = headers;
    factor_headers.remove(axum::http::header::COOKIE);
    factor_headers.remove(axum::http::header::AUTHORIZATION);
    let response =
        crate::twofa_routes::verify_link_factor(state.clone(), factor_headers, ext, pending, body)
            .await;
    finish(
        &state,
        &parent,
        &display,
        response,
        Some(&challenge),
        Some(&id),
    )
    .await
}

async fn finish(
    state: &AppState,
    parent: &mw_store::Session,
    display: &str,
    response: Response,
    challenge: Option<&str>,
    renewing: Option<&str>,
) -> Result<Response, Response> {
    let status = response.status();
    let retry = response
        .headers()
        .get(axum::http::header::RETRY_AFTER)
        .cloned();
    let bytes = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .map_err(unavailable)?;
    let mut body: Value = serde_json::from_slice(&bytes).map_err(unavailable)?;
    if !status.is_success() {
        // Only the normal public error DTO escapes. Never forward Set-Cookie.
        let mut response = (status, Json(body)).into_response();
        if let Some(retry) = retry {
            response
                .headers_mut()
                .insert(axum::http::header::RETRY_AFTER, retry);
        }
        return Err(response);
    }
    if body.get("twofaRequired").and_then(Value::as_bool) == Some(true) {
        let pending = body
            .get("pendingToken")
            .and_then(Value::as_str)
            .ok_or_else(|| unavailable("missing factor token"))?;
        if !state.twofa.native_link_pending(pending) {
            state.twofa.discard_link_pending(pending);
            return Err(failure(StatusCode::CONFLICT, "linkedBackendUnsupported"));
        }
        if body.get("enrollmentRequired").and_then(Value::as_bool) == Some(true) {
            state.twofa.discard_link_pending(pending);
            return Err(failure(
                StatusCode::CONFLICT,
                "linkedAccountEnrollmentRequired",
            ));
        }
        let challenge_result = state
            .store
            .create_linked_challenge(&parent.id, pending, display, chrono::Utc::now().timestamp())
            .await;
        let id = match challenge_result {
            Ok(Some(id)) => id,
            Ok(None) => {
                state.twofa.discard_link_pending(pending);
                return Err(failure(
                    StatusCode::TOO_MANY_REQUESTS,
                    "linkedChallengeLimit",
                ));
            }
            Err(error) => {
                state.twofa.discard_link_pending(pending);
                return Err(unavailable(error));
            }
        };
        body.as_object_mut()
            .ok_or_else(|| unavailable("factor response"))?
            .remove("pendingToken");
        body["challengeId"] = json!(id);
        return Ok((
            StatusCode::ACCEPTED,
            [(axum::http::header::CACHE_CONTROL, "private, no-store")],
            Json(body),
        )
            .into_response());
    }
    let token = body
        .get("token")
        .and_then(Value::as_str)
        .ok_or_else(|| unavailable("missing verified session"))?;
    let result = match renewing {
        None => attach(state, parent, token, display).await,
        Some(link) => rebind(state, parent, token, link).await,
    };
    if result.is_err() {
        let _ = state.store.delete_session(token).await;
        let _ = state
            .store
            .delete_native_session(&crate::push_relay::hash_token(token))
            .await;
        state.sessions.forget(token);
    }
    if let Some(id) = challenge {
        let _ = state.store.delete_linked_challenge(&parent.id, id).await;
    }
    result
}
async fn attach(
    state: &AppState,
    parent: &mw_store::Session,
    token: &str,
    display: &str,
) -> Result<Response, Response> {
    crate::security_center::enforce_session(state, parent).await?;
    require_enabled(state, parent).await?;
    let child = state.store.get_session(token).await.map_err(unavailable)?;
    crate::security_center::enforce_session(state, &child).await?;
    let gate = crate::account_gate::for_session(&state.store, &child)
        .await
        .map_err(unavailable)?;
    if gate.disabled {
        return Err(crate::unauthorized());
    }
    if gate.password_change_required {
        return Err(failure(
            StatusCode::CONFLICT,
            "linkedAccountPasswordChangeRequired",
        ));
    }
    require_enabled(state, &child).await?;
    let row = state
        .store
        .attach_verified_link(
            &parent.id,
            token,
            display,
            maximum(state).await?,
            chrono::Utc::now().timestamp(),
        )
        .await
        .map_err(unavailable)?
        .ok_or_else(|| failure(StatusCode::CONFLICT, "linkedAccountConflict"))?;
    state
        .store
        .delete_native_session(&crate::push_relay::hash_token(token))
        .await
        .map_err(unavailable)?;
    // t48: keep the password sealed so access renews by itself, or record why
    // it cannot (second factor, zero-access, policy).
    // The link is committed; a failure to store the credential only means
    // access will need renewing later, so it must not fail (and discard) it.
    if super::credential::remember(state, parent, &row.id, &child)
        .await
        .is_err()
    {
        tracing::warn!("linked credential not stored for a new link");
    }
    // Open realtime streams pick the new link up now (t46).
    links_changed();
    Ok((
        StatusCode::CREATED,
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        Json(json!({"account":entry(state,parent,&row).await?})),
    )
        .into_response())
}

/// t48: the renew counterpart of [`attach`]. The fresh login replaces the
/// link's lapsed source only when it is the same principal, subject and
/// account; anything else is refused and the fresh session discarded.
async fn rebind(
    state: &AppState,
    parent: &mw_store::Session,
    token: &str,
    link: &str,
) -> Result<Response, Response> {
    crate::security_center::enforce_session(state, parent).await?;
    require_enabled(state, parent).await?;
    let child = state.store.get_session(token).await.map_err(unavailable)?;
    crate::security_center::enforce_session(state, &child).await?;
    let gate = crate::account_gate::for_session(&state.store, &child)
        .await
        .map_err(unavailable)?;
    if gate.disabled {
        return Err(crate::unauthorized());
    }
    if gate.password_change_required {
        return Err(failure(
            StatusCode::CONFLICT,
            "linkedAccountPasswordChangeRequired",
        ));
    }
    require_enabled(state, &child).await?;
    if !state
        .store
        .rebind_linked_source(&parent.id, link, token, false)
        .await
        .map_err(unavailable)?
    {
        return Err(failure(StatusCode::CONFLICT, "linkedAccountMismatch"));
    }
    state
        .store
        .delete_native_session(&crate::push_relay::hash_token(token))
        .await
        .map_err(unavailable)?;
    if super::credential::remember(state, parent, link, &child)
        .await
        .is_err()
    {
        tracing::warn!("linked credential not stored after renewing access");
    }
    super::credential::audit(
        state,
        &parent.account_id,
        &parent.account_id,
        "linked-access-renewed",
        json!({"link":link}),
    )
    .await;
    links_changed();
    let row = state
        .store
        .linked_accounts(&parent.id)
        .await
        .map_err(unavailable)?
        .into_iter()
        .find(|row| row.id == link)
        .ok_or_else(|| failure(StatusCode::NOT_FOUND, "linkedAccountNotFound"))?;
    Ok((
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        Json(json!({"account":entry(state,parent,&row).await?})),
    )
        .into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PrimaryRequest {
    link_id: Option<String>,
}

/// `PUT /api/accounts/primary` — which mailbox opens first: one of the owner's
/// ready links, or `null` for the primary account. A preference only; opening
/// the link still needs `/switch` and every check behind it.
async fn set_primary(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PrimaryRequest>,
) -> Result<Response, Response> {
    let parent = parent(&state, &headers, true).await?;
    if let Some(id) = body.link_id.as_deref() {
        require_enabled(&state, &parent).await?;
        let Some(row) = state
            .store
            .linked_accounts(&parent.id)
            .await
            .map_err(unavailable)?
            .into_iter()
            .find(|row| row.id == id)
        else {
            return Err(failure(StatusCode::NOT_FOUND, "linkedAccountNotFound"));
        };
        if entry(&state, &parent, &row).await?["status"] != json!("ready") {
            return Err(failure(StatusCode::CONFLICT, "linkedAccountUnavailable"));
        }
    }
    if !state
        .store
        .set_linked_primary(
            &parent.id,
            body.link_id.as_deref(),
            chrono::Utc::now().timestamp(),
        )
        .await
        .map_err(unavailable)?
    {
        return Err(failure(StatusCode::NOT_FOUND, "linkedAccountNotFound"));
    }
    Ok((
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        Json(json!({"primaryMailbox":body.link_id})),
    )
        .into_response())
}

/// The key of the sign-in account in a mailbox layout; links use their ids.
const MAIN_KEY: &str = "main";
/// Longest nickname, in characters, after trimming.
pub(crate) const NICKNAME_MAX: usize = 64;

fn layout_keys<'a>(links: impl Iterator<Item = &'a str>) -> Vec<String> {
    std::iter::once(MAIN_KEY.to_owned())
        .chain(links.map(str::to_owned))
        .collect()
}

/// A display-only nickname: trimmed, at most [`NICKNAME_MAX`] characters, and
/// free of control, zero-width and bidirectional formatting characters (so it
/// cannot reorder or hide the address shown beside it). `None` = cleared.
pub(crate) fn nickname(raw: &str) -> Result<Option<String>, &'static str> {
    let value = raw.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > NICKNAME_MAX {
        return Err("nicknameTooLong");
    }
    if value.chars().any(|c| {
        c.is_control()
            || matches!(
                c,
                '\u{061C}'
                    | '\u{200B}'..='\u{200F}'
                    | '\u{202A}'..='\u{202E}'
                    | '\u{2060}'..='\u{206F}'
                    | '\u{FEFF}'
                    | '\u{FFF9}'..='\u{FFFB}'
            )
    }) {
        return Err("nicknameInvalid");
    }
    Ok(Some(value.to_owned()))
}

/// Order the current keys by `order`: unknown or repeated ids are dropped,
/// keys it does not name follow in their natural order (a new link lands at
/// the end; the sign-in account, if unnamed, first).
pub(crate) fn normalized_order(order: &[String], keys: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(keys.len());
    for key in order {
        if keys.contains(key) && !out.contains(key) {
            out.push(key.clone());
        }
    }
    if !out.iter().any(|k| k == MAIN_KEY) {
        out.insert(0, MAIN_KEY.to_owned());
    }
    for key in keys {
        if !out.contains(key) {
            out.push(key.clone());
        }
    }
    out
}

/// t86: the icons a mailbox avatar may show. The web app keeps the same list
/// (apps/web/src/api/mailboxAvatar.ts, `AVATAR_ICONS`); anything else is refused.
pub(crate) const AVATAR_ICONS: &[&str] = &[
    "briefcase",
    "office",
    "laptop",
    "team",
    "store",
    "chart",
    "house",
    "heartHome",
    "family",
    "person",
    "smile",
    "star",
    "lock",
    "shield",
    "eyeOff",
    "key",
    "gear",
    "crown",
    "wrench",
    "bolt",
    "inbox",
    "receipt",
    "bell",
    "cart",
    "plane",
    "school",
];
/// Longest avatar letter, in characters (Unicode scalar values), after trimming.
pub(crate) const AVATAR_TEXT_MAX: usize = 2;

/// One mailbox avatar as the client sends it (t86).
#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AvatarRequest {
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) text: Option<String>,
    #[serde(default)]
    pub(crate) icon: Option<String>,
    #[serde(default)]
    pub(crate) fg: Option<String>,
    #[serde(default)]
    pub(crate) bg: Option<String>,
}

/// A character an avatar letter may not hold: controls, whitespace, and format
/// characters (zero-width, bidirectional, soft hyphen, tag characters), so a
/// letter cannot hide or reorder what is shown beside it.
fn refused_avatar_char(c: char) -> bool {
    c.is_control()
        || c.is_whitespace()
        || matches!(
            c,
            '\u{00AD}'
                | '\u{0600}'..='\u{0605}'
                | '\u{061C}'
                | '\u{06DD}'
                | '\u{070F}'
                | '\u{08E2}'
                | '\u{180E}'
                | '\u{200B}'..='\u{200F}'
                | '\u{202A}'..='\u{202E}'
                | '\u{2060}'..='\u{206F}'
                | '\u{FEFF}'
                | '\u{FFF9}'..='\u{FFFB}'
                | '\u{110BD}'
                | '\u{1D173}'..='\u{1D17A}'
                | '\u{E0001}'
                | '\u{E0020}'..='\u{E007F}'
        )
}

/// `#rrggbb` (any case), stored lower case.
fn avatar_colour(raw: Option<&str>) -> Result<Option<String>, &'static str> {
    let Some(raw) = raw else { return Ok(None) };
    let value = raw.trim();
    if value.len() == 7
        && value.starts_with('#')
        && value[1..].bytes().all(|b| b.is_ascii_hexdigit())
    {
        Ok(Some(value.to_ascii_lowercase()))
    } else {
        Err("avatarColorInvalid")
    }
}

/// A validated avatar as stored and sent back, or `None` for the default look
/// (a letter with no text and no colours: nothing to keep).
pub(crate) fn avatar(raw: &AvatarRequest) -> Result<Option<Value>, &'static str> {
    let fg = avatar_colour(raw.fg.as_deref())?;
    let bg = avatar_colour(raw.bg.as_deref())?;
    let mut out = serde_json::Map::new();
    match raw.kind.as_str() {
        "letter" => {
            if raw.icon.is_some() {
                return Err("avatarInvalid");
            }
            let text = raw.text.as_deref().unwrap_or("").trim();
            if text.chars().count() > AVATAR_TEXT_MAX {
                return Err("avatarTextTooLong");
            }
            if text.chars().any(refused_avatar_char) {
                return Err("avatarTextInvalid");
            }
            if text.is_empty() && fg.is_none() && bg.is_none() {
                return Ok(None);
            }
            out.insert("kind".into(), json!("letter"));
            if !text.is_empty() {
                out.insert("text".into(), json!(text));
            }
        }
        "icon" => {
            if raw.text.is_some() {
                return Err("avatarInvalid");
            }
            let icon = raw.icon.as_deref().unwrap_or("");
            if !AVATAR_ICONS.contains(&icon) {
                return Err("avatarIconUnknown");
            }
            out.insert("kind".into(), json!("icon"));
            out.insert("icon".into(), json!(icon));
        }
        _ => return Err("avatarInvalid"),
    }
    if let Some(fg) = fg {
        out.insert("fg".into(), json!(fg));
    }
    if let Some(bg) = bg {
        out.insert("bg".into(), json!(bg));
    }
    Ok(Some(Value::Object(out)))
}

/// Stored avatars, for the current keys only, each validated again (a row
/// from an older or edited database cannot reach the page unchecked).
fn avatars_view(stored: &str, keys: &[String]) -> serde_json::Map<String, Value> {
    let raw: std::collections::BTreeMap<String, Value> =
        serde_json::from_str(stored).unwrap_or_default();
    raw.into_iter()
        .filter(|(k, _)| keys.contains(k))
        .filter_map(|(k, v)| {
            let spec: AvatarRequest = serde_json::from_value(v).ok()?;
            avatar(&spec).ok().flatten().map(|v| (k, v))
        })
        .collect()
}

fn layout_view(
    stored: Option<mw_store::linked_accounts::LinkedLayoutRow>,
    keys: &[String],
) -> Value {
    let stored = stored.unwrap_or_default();
    let order: Vec<String> = serde_json::from_str(&stored.order_json).unwrap_or_default();
    let nicknames: std::collections::BTreeMap<String, String> =
        serde_json::from_str(&stored.nicknames_json).unwrap_or_default();
    let nicknames: serde_json::Map<String, Value> = nicknames
        .into_iter()
        .filter(|(k, _)| keys.contains(k))
        .filter_map(|(k, v)| nickname(&v).ok().flatten().map(|v| (k, json!(v))))
        .collect();
    let avatars = avatars_view(&stored.avatars_json, keys);
    json!({"revision":stored.revision,"order":normalized_order(&order,keys),"nicknames":nicknames,"avatars":avatars})
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LayoutRequest {
    revision: String,
    order: Vec<String>,
    nicknames: std::collections::BTreeMap<String, String>,
    /// t86: absent = keep the stored avatars (a client that predates them).
    #[serde(default)]
    avatars: Option<std::collections::BTreeMap<String, AvatarRequest>>,
}

/// `PUT /api/accounts/layout` — the owner's order, nicknames and avatars for their
/// mailboxes, guarded by the revision they last read (409 when another device
/// saved first). Display only: nothing here is an identity or reaches upstream.
async fn set_layout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<LayoutRequest>,
) -> Result<Response, Response> {
    let parent = parent(&state, &headers, true).await?;
    if body.order.len() > 32
        || body.nicknames.len() > 32
        || body.avatars.as_ref().is_some_and(|a| a.len() > 32)
        || body.revision.len() > 64
    {
        return Err(failure(StatusCode::UNPROCESSABLE_ENTITY, "invalidLayout"));
    }
    let rows = state
        .store
        .linked_accounts(&parent.id)
        .await
        .map_err(unavailable)?;
    let keys = layout_keys(rows.iter().map(|row| row.id.as_str()));
    let order = normalized_order(&body.order, &keys);
    let mut nicknames = std::collections::BTreeMap::new();
    for (key, raw) in &body.nicknames {
        let value =
            nickname(raw).map_err(|code| failure(StatusCode::UNPROCESSABLE_ENTITY, code))?;
        if let (true, Some(value)) = (keys.contains(key), value) {
            nicknames.insert(key.clone(), value);
        }
    }
    let avatars = match &body.avatars {
        None => None,
        Some(raw) => {
            let mut out = serde_json::Map::new();
            for (key, spec) in raw {
                let value =
                    avatar(spec).map_err(|code| failure(StatusCode::UNPROCESSABLE_ENTITY, code))?;
                if let (true, Some(value)) = (keys.contains(key), value) {
                    out.insert(key.clone(), value);
                }
            }
            Some(out)
        }
    };
    let order_json = serde_json::to_string(&order).map_err(unavailable)?;
    let nicknames_json = serde_json::to_string(&nicknames).map_err(unavailable)?;
    let avatars_json = avatars
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(unavailable)?;
    let Some(revision) = state
        .store
        .put_linked_layout(
            &parent.id,
            &body.revision,
            &order_json,
            &nicknames_json,
            avatars_json.as_deref(),
            chrono::Utc::now().timestamp(),
        )
        .await
        .map_err(unavailable)?
    else {
        return Err(failure(StatusCode::CONFLICT, "layoutConflict"));
    };
    // Avatars not sent are the stored ones: read them back for the answer.
    let avatars = match avatars {
        Some(avatars) => avatars,
        None => avatars_view(
            &state
                .store
                .linked_layout(&parent.id)
                .await
                .map_err(unavailable)?
                .unwrap_or_default()
                .avatars_json,
            &keys,
        ),
    };
    Ok((
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        Json(json!({"layout":{"revision":revision,"order":order,"nicknames":nicknames,"avatars":avatars}})),
    )
        .into_response())
}

async fn switch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, Response> {
    let parent = parent(&state, &headers, true).await?;
    let mut selected = headers;
    selected.insert(
        SELECTOR,
        id.parse()
            .map_err(|_| failure(StatusCode::BAD_REQUEST, "invalidLinkedContext"))?,
    );
    let child = resolve(&state, &selected, &parent)
        .await?
        .ok_or_else(crate::unauthorized)?;
    Ok(([(axum::http::header::CACHE_CONTROL,"private, no-store")],Json(json!({"context":{"linkId":id,"accountId":child.account_id,"storageAuthority":state.store.session_storage_authority(&child.id).await.map_err(unavailable)?},"realtimeAvailable":false}))).into_response())
}
async fn unlink(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, Response> {
    let parent = parent(&state, &headers, true).await?;
    crate::step_up::require_user(
        &state,
        &parent,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    let tag = headers
        .get(axum::http::header::IF_MATCH)
        .ok_or_else(|| failure(StatusCode::PRECONDITION_REQUIRED, "revisionRequired"))?
        .to_str()
        .map_err(|_| failure(StatusCode::PRECONDITION_FAILED, "revisionMismatch"))?;
    let revision = tag
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .filter(|v| uuid::Uuid::parse_str(v).is_ok())
        .ok_or_else(|| failure(StatusCode::PRECONDITION_FAILED, "revisionMismatch"))?;
    if !state
        .store
        .unlink_account(&parent.id, &id, revision)
        .await
        .map_err(unavailable)?
    {
        return Err(failure(StatusCode::PRECONDITION_FAILED, "revisionMismatch"));
    }
    // Delivery already stops at the next change (it is re-verified); this
    // drops the link from open streams' sets at once (t46).
    links_changed();
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[test]
    fn nicknames_are_trimmed_capped_and_refuse_control_and_bidi_characters() {
        assert_eq!(nickname("  Work  "), Ok(Some("Work".into())));
        assert_eq!(nickname("   "), Ok(None));
        assert_eq!(
            nickname(&"é".repeat(NICKNAME_MAX)),
            Ok(Some("é".repeat(NICKNAME_MAX)))
        );
        assert_eq!(
            nickname(&"a".repeat(NICKNAME_MAX + 1)),
            Err("nicknameTooLong")
        );
        for bad in [
            "a\u{202E}b",
            "a\u{2066}b",
            "a\u{200F}b",
            "a\nb",
            "a\u{0007}b",
            "a\u{FEFF}b",
        ] {
            assert_eq!(nickname(bad), Err("nicknameInvalid"), "{bad:?}");
        }
        assert_eq!(
            nickname("<b>Personal</b>"),
            Ok(Some("<b>Personal</b>".into())),
            "kept as text; the client renders it as text"
        );
    }

    #[test]
    fn order_drops_unknown_ids_and_appends_new_links() {
        let keys = layout_keys(["a", "b", "c"].into_iter());
        let order = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            normalized_order(&order(&["c", "x", "main", "c"]), &keys),
            order(&["c", "main", "a", "b"])
        );
        assert_eq!(
            normalized_order(&order(&["b"]), &keys),
            order(&["main", "b", "a", "c"])
        );
        assert_eq!(
            normalized_order(&[], &keys),
            order(&["main", "a", "b", "c"])
        );
    }

    fn spec(
        kind: &str,
        text: Option<&str>,
        icon: Option<&str>,
        fg: Option<&str>,
        bg: Option<&str>,
    ) -> AvatarRequest {
        AvatarRequest {
            kind: kind.into(),
            text: text.map(Into::into),
            icon: icon.map(Into::into),
            fg: fg.map(Into::into),
            bg: bg.map(Into::into),
        }
    }

    #[test]
    fn avatar_icons_come_from_the_allowlist_and_colours_are_hex() {
        assert_eq!(
            avatar(&spec(
                "icon",
                None,
                Some("briefcase"),
                Some("#FFFFFF"),
                Some("#1D4ED8")
            )),
            Ok(Some(
                serde_json::json!({"kind":"icon","icon":"briefcase","fg":"#ffffff","bg":"#1d4ed8"})
            ))
        );
        assert_eq!(AVATAR_ICONS.len(), 26);
        for icon in AVATAR_ICONS {
            assert!(
                avatar(&spec("icon", None, Some(icon), None, None)).is_ok(),
                "{icon}"
            );
        }
        for bad in ["", "Briefcase", "rocket", "<svg>", "briefcase "] {
            assert_eq!(
                avatar(&spec("icon", None, Some(bad), None, None)),
                Err("avatarIconUnknown"),
                "{bad:?}"
            );
        }
        assert_eq!(
            avatar(&spec("icon", None, None, None, None)),
            Err("avatarIconUnknown")
        );
        for bad in [
            "#fff",
            "red",
            "#12345g",
            "#1234567",
            "rgb(0,0,0)",
            "#ffffff;x",
            "",
        ] {
            assert_eq!(
                avatar(&spec("letter", Some("A"), None, Some(bad), None)),
                Err("avatarColorInvalid"),
                "fg {bad:?}"
            );
            assert_eq!(
                avatar(&spec("letter", Some("A"), None, None, Some(bad))),
                Err("avatarColorInvalid"),
                "bg {bad:?}"
            );
        }
        assert_eq!(
            avatar(&spec("emoji", None, None, None, None)),
            Err("avatarInvalid")
        );
        assert_eq!(
            avatar(&spec("letter", Some("A"), Some("lock"), None, None)),
            Err("avatarInvalid")
        );
        assert_eq!(
            avatar(&spec("icon", Some("A"), Some("lock"), None, None)),
            Err("avatarInvalid")
        );
    }

    #[test]
    fn avatar_letters_are_one_or_two_visible_characters() {
        assert_eq!(
            avatar(&spec("letter", Some(" W "), None, None, None)),
            Ok(Some(serde_json::json!({"kind":"letter","text":"W"})))
        );
        assert_eq!(
            avatar(&spec(
                "letter",
                Some("Ação".get(0..3).unwrap()),
                None,
                None,
                Some("#000000")
            )),
            Ok(Some(
                serde_json::json!({"kind":"letter","text":"Aç","bg":"#000000"})
            ))
        );
        assert!(
            avatar(&spec("letter", Some("日本"), None, None, None))
                .unwrap()
                .is_some()
        );
        assert!(
            avatar(&spec("letter", Some("\u{1F600}"), None, None, None))
                .unwrap()
                .is_some()
        );
        assert_eq!(
            avatar(&spec("letter", Some("ABC"), None, None, None)),
            Err("avatarTextTooLong")
        );
        for bad in [
            "A\u{202E}",
            "\u{200B}A",
            "\u{2066}A",
            "\u{0007}A",
            "\u{00AD}A",
            "A\u{E0041}",
            "\u{FEFF}",
        ] {
            assert_eq!(
                avatar(&spec("letter", Some(bad), None, None, None)),
                Err("avatarTextInvalid"),
                "{bad:?}"
            );
        }
        // The default look (no letter, no colours) is not stored.
        assert_eq!(avatar(&spec("letter", None, None, None, None)), Ok(None));
        assert_eq!(
            avatar(&spec("letter", Some("  "), None, None, None)),
            Ok(None)
        );
    }

    #[test]
    fn stored_avatars_are_checked_again_and_kept_to_current_keys() {
        let keys = layout_keys(["a"].into_iter());
        let stored = serde_json::json!({
            "main": {"kind":"icon","icon":"house","bg":"#112233"},
            "a": {"kind":"icon","icon":"rocket"},
            "gone": {"kind":"letter","text":"G"},
        })
        .to_string();
        let view = avatars_view(&stored, &keys);
        assert_eq!(
            serde_json::Value::Object(view),
            serde_json::json!({"main":{"kind":"icon","icon":"house","bg":"#112233"}})
        );
        assert!(avatars_view("not json", &keys).is_empty());
    }
}
