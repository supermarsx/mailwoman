use super::*;
use axum::{
    Router,
    extract::{Path, State},
    http::Extensions,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::Value;

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/accounts", get(list))
        .route("/api/accounts/link", post(link))
        .route("/api/accounts/challenges/{id}/confirm", post(confirm))
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
    if let Some((_, source)) = state
        .store
        .linked_source(&parent.id, &row.id)
        .await
        .map_err(unavailable)?
    {
        value["address"] = json!(source.username);
        let live = match crate::security_center::enforce_session_clock(state, &source).await {
            Ok(()) => true,
            Err(response) if response.status() == StatusCode::UNAUTHORIZED => false,
            Err(response) => return Err(response),
        };
        if live {
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
    Ok(([(axum::http::header::CACHE_CONTROL,"private, no-store")],Json(json!({
        "primary":{"accountId":parent.account_id,"address":parent.username,"storageAuthority":state.store.session_storage_authority(&parent.id).await.map_err(unavailable)?},
        "accounts":accounts,"policy":{"enabled":enabled(&state,&parent).await?,"maxAccounts":maximum(&state).await?,
        "supportedAuthentication":["password","totp","recovery","passkeySecondFactor"],"supportedBackends":["imap","pop3","ews"],
        "reason":if supported(&state,&parent).await?{Value::Null}else{json!("linkedBackendUnsupported")},"childRealtime":false}
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
    finish(&state, &parent, &display, response, None).await
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
    finish(&state, &parent, &display, response, Some(&id)).await
}

async fn finish(
    state: &AppState,
    parent: &mw_store::Session,
    display: &str,
    response: Response,
    challenge: Option<&str>,
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
    let result = attach(state, parent, token, display).await;
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
    Ok((
        StatusCode::CREATED,
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        Json(json!({"account":entry(state,parent,&row).await?})),
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
    Ok(StatusCode::NO_CONTENT.into_response())
}
