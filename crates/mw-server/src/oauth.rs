//! OAuth 2.1 AS + scoped API keys + zero-access user surface (SPEC §20.1/§9,
//! plan §3 e11 MOUNT). Cookie-authed (the resource-owner session) for the
//! consent/keys/zero-access endpoints; the token/introspect/revoke endpoints are
//! client-called (no cookie). Backed by [`mw_oauth::AuthServer`] over the 0007
//! `api_keys`/`oauth_*` tables and the `zeroaccess_accounts` table.

use axum::Json;
use axum::Router;
use axum::extract::{Path as UrlPath, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use base64::Engine as _;
use serde::Deserialize;
use serde_json::{Value, json};

use mw_oauth::{AuthorizeRequest, OAuthStore, Scope, ScopeSelector, TokenRequest, mint_api_key};

use crate::AppState;

/// The user-facing V6 auth + zero-access routes (merged by e11's `router()`).
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/oauth/consent", post(consent))
        .route("/oauth/decision", post(decision))
        .route("/oauth/token", post(token))
        .route("/oauth/introspect", post(introspect))
        .route("/oauth/revoke", post(revoke))
        .route("/api/keys", get(list_keys).post(create_key))
        .route("/api/keys/{prefix}/revoke", post(revoke_key))
        .route("/api/zeroaccess", get(zeroaccess_status))
        .route("/api/zeroaccess/enable", post(zeroaccess_enable))
        .route("/api/zeroaccess/disable", post(zeroaccess_disable))
        .route("/api/zeroaccess/pair/offer", post(pair_offer))
        .route("/api/zeroaccess/pair/envelope", post(pair_envelope))
        .route("/api/zeroaccess/pair/envelope/{id}", get(pair_envelope_get))
}

fn bad_request(msg: &str) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response()
}

fn server_error(e: impl std::fmt::Display) -> Response {
    tracing::warn!("oauth/keys error: {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": "operation failed" })),
    )
        .into_response()
}

// ─── OAuth authorize params (camelCase from the web consent flow) ─────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthorizeParams {
    #[serde(default = "default_response_type")]
    response_type: String,
    client_id: String,
    redirect_uri: String,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    code_challenge: String,
    #[serde(default = "default_challenge_method")]
    code_challenge_method: String,
    #[serde(default)]
    resource: String,
    /// The requested scope in `mw-oauth` wire form (snake_case `Scope`).
    #[serde(default)]
    scope: Option<Scope>,
}

fn default_response_type() -> String {
    "code".to_string()
}
fn default_challenge_method() -> String {
    "S256".to_string()
}

impl AuthorizeParams {
    fn scope_or_default(&self, account_id: &str) -> Scope {
        self.scope
            .clone()
            .unwrap_or_else(|| Scope::read_only(account_id))
    }
}

// ─── /oauth/consent — validate + display ──────────────────────────────────────

async fn consent(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(params): Json<AuthorizeParams>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let client = state
        .v6
        .auth
        .store()
        .store()
        .get_oauth_client(&params.client_id)
        .await;
    let (client_name, approved) = match client {
        Ok(Some(c)) => (c.name, true),
        _ => (params.client_id.clone(), false),
    };
    let requested = params.scope_or_default(&session.account_id);
    Json(json!({
        "clientId": params.client_id,
        "clientName": client_name,
        "approved": approved,
        "redirectUri": params.redirect_uri,
        "resource": params.resource,
        "requestedScope": requested,
    }))
    .into_response()
}

// ─── /oauth/decision — grant/deny ─────────────────────────────────────────────

#[derive(Deserialize)]
struct DecisionReq {
    approve: bool,
    params: AuthorizeParams,
}

async fn decision(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<DecisionReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let p = &body.params;
    if !body.approve {
        let mut url = format!("{}?error=access_denied", p.redirect_uri);
        if let Some(s) = &p.state {
            url.push_str(&format!("&state={}", urlencode(s)));
        }
        return Json(json!({ "redirectUri": url })).into_response();
    }
    let req = AuthorizeRequest {
        response_type: p.response_type.clone(),
        client_id: p.client_id.clone(),
        redirect_uri: p.redirect_uri.clone(),
        scope: p.scope_or_default(&session.account_id),
        state: p.state.clone(),
        code_challenge: p.code_challenge.clone(),
        code_challenge_method: p.code_challenge_method.clone(),
        resource: p.resource.clone(),
    };
    match state.v6.auth.authorize(&req, &session.account_id).await {
        Ok(res) => {
            let mut url = format!("{}?code={}", res.redirect_uri, urlencode(&res.code));
            if let Some(s) = res.state {
                url.push_str(&format!("&state={}", urlencode(&s)));
            }
            Json(json!({ "redirectUri": url })).into_response()
        }
        Err(e) => bad_request(&format!("authorization denied: {e}")),
    }
}

// ─── /oauth/token ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
struct TokenReq {
    grant_type: String,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    redirect_uri: Option<String>,
    #[serde(default)]
    client_id: Option<String>,
    #[serde(default)]
    code_verifier: Option<String>,
    #[serde(default)]
    resource: Option<String>,
    #[serde(default)]
    refresh_token: Option<String>,
}

async fn token(State(state): State<AppState>, Json(body): Json<TokenReq>) -> Response {
    let req = match body.grant_type.as_str() {
        "authorization_code" => TokenRequest::AuthorizationCode {
            code: body.code.unwrap_or_default(),
            redirect_uri: body.redirect_uri.unwrap_or_default(),
            client_id: body.client_id.unwrap_or_default(),
            code_verifier: body.code_verifier.unwrap_or_default(),
            resource: body.resource.unwrap_or_default(),
        },
        "refresh_token" => TokenRequest::RefreshToken {
            refresh_token: body.refresh_token.unwrap_or_default(),
            client_id: body.client_id.unwrap_or_default(),
            resource: body.resource,
        },
        other => return bad_request(&format!("unsupported grant_type: {other}")),
    };
    match state.v6.auth.token(&req).await {
        Ok(t) => Json(json!({
            "access_token": t.access_token,
            "refresh_token": t.refresh_token,
            "token_type": t.token_type,
            "expires_in": t.expires_in,
            "scope": t.scope,
            "resource": t.resource,
        }))
        .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_grant", "detail": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct TokenBody {
    token: String,
}

async fn introspect(State(state): State<AppState>, Json(body): Json<TokenBody>) -> Response {
    match state.v6.auth.introspect(&body.token).await {
        Ok(i) => Json(json!({
            "active": i.active,
            "scope": i.scope,
            "resource": i.resource,
            "client_id": i.client_id,
            "account_id": i.account_id,
            "exp": i.expires_at,
        }))
        .into_response(),
        Err(e) => server_error(e),
    }
}

async fn revoke(State(state): State<AppState>, Json(body): Json<TokenBody>) -> Response {
    match state.v6.auth.revoke(&body.token).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => server_error(e),
    }
}

// ─── scoped API keys ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateKeyReq {
    #[serde(default)]
    label: String,
    #[serde(default)]
    account_id: Option<String>,
    scope: Scope,
}

async fn create_key(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateKeyReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    // The key is always scoped to the requesting session's own account.
    let account_id = body
        .account_id
        .unwrap_or_else(|| session.account_id.clone());
    if account_id != session.account_id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "account mismatch" })),
        )
            .into_response();
    }
    let minted = mint_api_key(&account_id, body.scope.clone());
    if let Err(e) = state
        .v6
        .auth
        .store()
        .put_api_key(minted.record.clone())
        .await
    {
        return server_error(e);
    }
    let record = key_record_camel(&minted.record, &body.label, false);
    Json(json!({ "displayToken": minted.display_token, "record": record })).into_response()
}

async fn list_keys(State(state): State<AppState>, headers: axum::http::HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let rows = match state.store.list_api_keys().await {
        Ok(r) => r,
        Err(e) => return server_error(e),
    };
    let out: Vec<Value> = rows
        .into_iter()
        .filter(|r| r.account_id == session.account_id && r.revoked_at.is_none())
        .filter_map(|r| {
            let scope: Scope = serde_json::from_str(&r.scopes_json).ok()?;
            let key = mw_oauth::ApiKey {
                prefix: r.key_prefix,
                hash: r.key_hash,
                account_id: r.account_id,
                scope,
                created_at: r.created_at,
                last_used_at: r.last_used_at,
                revoked_at: r.revoked_at,
            };
            Some(key_record_camel(&key, "", r.unattended_send))
        })
        .collect();
    Json(out).into_response()
}

async fn revoke_key(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    UrlPath(prefix): UrlPath<String>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    match state.v6.auth.store().revoke_api_key(&prefix).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(e) => server_error(e),
    }
}

/// Build the camelCase `ApiKeyRecord` the web `apikeys/types.ts` expects.
fn key_record_camel(key: &mw_oauth::ApiKey, label: &str, unattended_approved: bool) -> Value {
    json!({
        "prefix": key.prefix,
        "label": label,
        "accountId": key.account_id,
        "scope": scope_camel(&key.scope),
        "createdAt": key.created_at,
        "lastUsedAt": key.last_used_at,
        "revokedAt": key.revoked_at,
        "unattendedSendApproved": unattended_approved,
    })
}

/// Map the wire (`snake_case`) [`Scope`] to the web's camelCase `ApiKeyScope`.
fn scope_camel(s: &Scope) -> Value {
    let sel = |sel: &ScopeSelector| match sel {
        ScopeSelector::All => json!({ "kind": "all" }),
        ScopeSelector::Subset(ids) => json!({ "kind": "subset", "ids": ids }),
    };
    json!({
        "read": s.read,
        "send": s.send,
        "delete": s.delete,
        "accounts": sel(&s.accounts),
        "folders": sel(&s.folders),
        "mail": s.mail,
        "pim": s.pim,
        "ipAllowlist": s.ip_allowlist,
        "expiresAt": s.expires_at,
        "rateLimit": s.rate_limit,
        "mcpTools": s.mcp_tools,
        "unattendedSend": s.unattended_send,
    })
}

// ─── zero-access (§9): the server stores wrapped material + relays ciphertext ──

async fn zeroaccess_status(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    match state.store.get_zeroaccess(&session.account_id).await {
        Ok(Some(row)) => {
            let meta: Value = serde_json::from_str(&row.kdf_params_json).unwrap_or(Value::Null);
            let paired: Value =
                serde_json::from_str(&row.paired_devices_json).unwrap_or_else(|_| json!([]));
            Json(json!({
                "enabled": row.enabled,
                "saltB64": meta.get("saltB64"),
                "kdfParams": meta.get("kdfParams"),
                "wrappedDataKeyB64": base64::engine::general_purpose::STANDARD.encode(&row.wrapped_root_key),
                "pairedDevices": paired,
            }))
            .into_response()
        }
        Ok(None) => Json(json!({ "enabled": false, "pairedDevices": [] })).into_response(),
        Err(e) => server_error(e),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnableReq {
    salt_b64: String,
    kdf_params: Value,
    wrapped_data_key_b64: String,
}

async fn zeroaccess_enable(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<EnableReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let wrapped = match base64::engine::general_purpose::STANDARD.decode(&body.wrapped_data_key_b64)
    {
        Ok(b) => b,
        Err(_) => return bad_request("wrappedDataKeyB64 is not valid base64"),
    };
    let meta = json!({ "saltB64": body.salt_b64, "kdfParams": body.kdf_params }).to_string();
    let row = mw_store::ZeroAccessRow {
        account_id: session.account_id.clone(),
        enabled: true,
        wrapped_root_key: wrapped,
        kdf_params_json: meta,
        recovery_wrapped: None,
        paired_devices_json: "[]".to_string(),
    };
    match state.store.upsert_zeroaccess(&row).await {
        Ok(()) => {
            let _ = state
                .v6
                .admin
                .toggle_zero_access("system", &session.account_id, true)
                .await;
            Json(json!({ "ok": true })).into_response()
        }
        Err(e) => server_error(e),
    }
}

async fn zeroaccess_disable(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    match state
        .store
        .set_zeroaccess_enabled(&session.account_id, false)
        .await
    {
        Ok(()) => {
            let _ = state
                .v6
                .admin
                .toggle_zero_access("system", &session.account_id, false)
                .await;
            Json(json!({ "ok": true })).into_response()
        }
        Err(e) => server_error(e),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PairOfferReq {
    /// The offering device's ephemeral public point (base64). Accepted but not
    /// persisted — it travels out-of-band in the scanned QR; the server relays the
    /// sealed envelope only.
    #[serde(default)]
    #[allow(dead_code)]
    public_b64: String,
}

async fn pair_offer(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(_body): Json<PairOfferReq>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    // The server relays ciphertext only; it mints an opaque pairing id and never
    // persists a plaintext key. The scanned QR carries the public point out-of-band.
    let pairing_id = mw_store::ServerKey::generate().to_hex();
    Json(json!({ "pairingId": pairing_id })).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PairEnvelopeReq {
    pairing_id: String,
    envelope_b64: String,
}

async fn pair_envelope(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<PairEnvelopeReq>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    state
        .v6
        .pairing
        .lock()
        .expect("pairing lock")
        .insert(body.pairing_id, body.envelope_b64);
    Json(json!({ "ok": true })).into_response()
}

async fn pair_envelope_get(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    let envelope = state
        .v6
        .pairing
        .lock()
        .expect("pairing lock")
        .get(&id)
        .cloned();
    Json(json!({ "envelopeB64": envelope })).into_response()
}

/// Minimal percent-encoding for redirect query values (unreserved set passes).
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}
