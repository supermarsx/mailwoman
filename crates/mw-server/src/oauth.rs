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

// ─── OAuth Dynamic Client Registration (RFC 7591 / RFC 7592) — t10 e8 ─────────
//
// Admin/policy-gated, **DEFAULT DISABLED**. The policy lives in the 0010 `oauth_dcr`
// singleton; a DCR-issued client is an ordinary 0007 `oauth_clients` row plus a
// 0010 `oauth_client_meta` side row (RFC-7591 extras + the HASH of the per-client
// registration-access-token). No edit to 0007, the `Scope` model, or the existing
// AS behaviour. The registration logic lives in `mw_oauth::dcr`; these handlers are
// the thin HTTP glue + the policy/initial-access-token gate.
//
//   * `POST   /oauth/register`      — register a client (RFC 7591). Disabled ⇒ 403.
//   * `GET    /oauth/register/{id}` — read client config   (registration-access-token).
//   * `PUT    /oauth/register/{id}` — update client config (registration-access-token).
//   * `DELETE /oauth/register/{id}` — delete the client    (registration-access-token).
//
// MOUNT NOTE for t10-e13: these routes are provided as [`dcr_router`]; wire it into
// `build_app`'s router (e.g. `.merge(oauth::dcr_router())`). It is intentionally NOT
// merged into [`routes()`] so it stays a single explicit mount point. The inner
// `#![allow(dead_code)]` keeps the chain warning-clean while unmounted (e0's stub
// convention); mounting it (e13) marks the whole chain live.
// Re-exported for t10-e13's mount; unused in-crate until then.
#[allow(unused_imports)]
pub(crate) use dcr::dcr_router;

mod dcr {
    #![allow(dead_code)]
    use super::*;

    pub(crate) fn dcr_router() -> Router<AppState> {
        Router::new()
            .route("/oauth/register", post(dcr_register))
            .route(
                "/oauth/register/{id}",
                get(dcr_read).put(dcr_update).delete(dcr_delete),
            )
            // Admin enable/read surface (parity with `/admin/sso`, `/admin/ui-plugins`).
            // Admin-session-gated; the ONLY in-band way to toggle DCR (still default off).
            .route("/admin/oauth-dcr", get(dcr_admin_get).put(dcr_admin_put))
    }

    /// Env var carrying the shared initial-access-token, checked when the policy sets
    /// `require_initial_access_token`. Unset ⇒ every registration is refused (401) while
    /// the requirement is on (fail-closed).
    const DCR_INITIAL_ACCESS_TOKEN_ENV: &str = "MW_DCR_INITIAL_ACCESS_TOKEN";

    /// Extract a `Bearer <token>` value from the `Authorization` header.
    fn bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
        let raw = headers
            .get(axum::http::header::AUTHORIZATION)?
            .to_str()
            .ok()?;
        let token = raw
            .strip_prefix("Bearer ")
            .or_else(|| raw.strip_prefix("bearer "))?;
        let token = token.trim();
        (!token.is_empty()).then(|| token.to_string())
    }

    /// Constant-time string comparison (initial-access-token check).
    fn ct_eq_str(a: &str, b: &str) -> bool {
        let (a, b) = (a.as_bytes(), b.as_bytes());
        if a.len() != b.len() {
            return false;
        }
        let mut diff = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            diff |= x ^ y;
        }
        diff == 0
    }

    /// Derive the absolute issuer base URL for `registration_client_uri` from the request
    /// `Host` header. Loopback hosts default to `http`, everything else to `https`.
    fn issuer_base_url(headers: &axum::http::HeaderMap) -> String {
        let host = headers
            .get(axum::http::header::HOST)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("localhost");
        let scheme = if host.starts_with("127.0.0.1")
            || host.starts_with("localhost")
            || host.starts_with("[::1]")
        {
            "http"
        } else {
            "https"
        };
        format!("{scheme}://{host}")
    }

    /// Load + gate the DCR policy: `Ok(policy)` when enabled, else a ready `403`/`500`.
    async fn load_enabled_policy(
        state: &AppState,
        headers: &axum::http::HeaderMap,
    ) -> Result<mw_oauth::DcrPolicy, Response> {
        let row = match state.store.get_oauth_dcr_policy().await {
            Ok(Some(row)) => row,
            Ok(None) => return Err(dcr_disabled()),
            Err(e) => return Err(server_error(e)),
        };
        if !row.enabled {
            return Err(dcr_disabled());
        }
        let allowed_redirect_host_suffixes: Vec<String> =
            serde_json::from_str(&row.allowed_redirect_host_suffixes_json).unwrap_or_default();
        let default_scope: Scope = serde_json::from_str(&row.default_scope_json)
            .unwrap_or_else(|_| mw_oauth::dcr::no_scope());
        Ok(mw_oauth::DcrPolicy {
            enabled: row.enabled,
            require_initial_access_token: row.require_initial_access_token,
            allowed_redirect_host_suffixes,
            default_scope,
            issuer_base_url: issuer_base_url(headers),
        })
    }

    fn dcr_disabled() -> Response {
        (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "access_denied",
                "error_description": "dynamic client registration is disabled",
            })),
        )
            .into_response()
    }

    /// Map a [`mw_oauth::DcrError`] onto its RFC-7591 HTTP response.
    fn dcr_error_response(e: &mw_oauth::DcrError) -> Response {
        use mw_oauth::DcrError as E;
        match e {
        E::Disabled => dcr_disabled(),
        E::InitialAccessTokenRequired => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid_token", "error_description": e.to_string() })),
        )
            .into_response(),
        E::InvalidRedirectUri(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_redirect_uri", "error_description": e.to_string() })),
        )
            .into_response(),
        E::InvalidClientMetadata(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_client_metadata", "error_description": e.to_string() })),
        )
            .into_response(),
        E::Store(_) => server_error(e),
    }
    }

    /// POST /oauth/register — RFC 7591 registration (policy-gated, optional IAT gate).
    async fn dcr_register(
        State(state): State<AppState>,
        headers: axum::http::HeaderMap,
        Json(request): Json<mw_oauth::ClientRegistrationRequest>,
    ) -> Response {
        let policy = match load_enabled_policy(&state, &headers).await {
            Ok(p) => p,
            Err(resp) => return resp,
        };
        // Optional initial-access-token gate (fail-closed).
        if policy.require_initial_access_token {
            let expected = std::env::var(DCR_INITIAL_ACCESS_TOKEN_ENV)
                .ok()
                .filter(|v| !v.is_empty());
            let ok = match (expected, bearer_token(&headers)) {
                (Some(exp), Some(got)) => ct_eq_str(&exp, &got),
                _ => false,
            };
            if !ok {
                return dcr_error_response(&mw_oauth::DcrError::InitialAccessTokenRequired);
            }
        }

        let resp = match mw_oauth::dcr::register(state.v6.auth.store(), request, policy).await {
            Ok(r) => r,
            Err(e) => return dcr_error_response(&e),
        };

        // Persist the RFC-7591 extras + the registration-access-token HASH (0010 side row).
        let meta = mw_store::OAuthClientMetaRow {
            client_id: resp.client_id.clone(),
            registration_access_token_hash: Some(resp.registration_access_token_hash.clone()),
            software_id: resp.software_id.clone(),
            software_version: resp.software_version.clone(),
            contacts_json: serde_json::to_string(&resp.contacts).unwrap_or_else(|_| "[]".into()),
            created_via: mw_oauth::dcr::DCR_CREATED_VIA.to_string(),
            created_at: resp.created_at.clone(),
        };
        if let Err(e) = state.store.put_oauth_client_meta(&meta).await {
            return server_error(e);
        }

        (StatusCode::CREATED, Json(resp)).into_response()
    }

    /// Authenticate an RFC-7592 client-configuration request via the per-client
    /// registration-access-token. Returns the client's meta row, or a ready `401`.
    async fn authenticate_dcr_client(
        state: &AppState,
        headers: &axum::http::HeaderMap,
        client_id: &str,
    ) -> Result<mw_store::OAuthClientMetaRow, Response> {
        let unauthorized = || {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "invalid_token" })),
            )
                .into_response()
        };
        let meta = match state.store.get_oauth_client_meta(client_id).await {
            Ok(Some(m)) => m,
            Ok(None) => return Err(unauthorized()),
            Err(e) => return Err(server_error(e)),
        };
        let Some(hash) = meta.registration_access_token_hash.as_deref() else {
            return Err(unauthorized());
        };
        let Some(presented) = bearer_token(headers) else {
            return Err(unauthorized());
        };
        if !mw_oauth::verify_registration_access_token(&presented, hash) {
            return Err(unauthorized());
        }
        Ok(meta)
    }

    /// Rebuild an RFC-7592 read/update response from the stored 0007 client + 0010 meta.
    /// grant/response/auth-method/scope are the AS-fixed values (this server only issues
    /// public PKCE `authorization_code` clients under the current policy default scope).
    async fn dcr_client_response(
        state: &AppState,
        headers: &axum::http::HeaderMap,
        client_id: &str,
        meta: &mw_store::OAuthClientMetaRow,
    ) -> Result<mw_oauth::ClientRegistrationResponse, Response> {
        let client = match state.v6.auth.store().get_client(client_id).await {
            Ok(Some(c)) => c,
            Ok(None) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "invalid_client_id" })),
                )
                    .into_response());
            }
            Err(e) => return Err(server_error(e)),
        };
        let contacts: Vec<String> = serde_json::from_str(&meta.contacts_json).unwrap_or_default();
        let request = mw_oauth::ClientRegistrationRequest {
            redirect_uris: client.redirect_uris.clone(),
            client_name: Some(client.name.clone()).filter(|s| !s.is_empty()),
            software_id: meta.software_id.clone(),
            software_version: meta.software_version.clone(),
            contacts,
            ..Default::default()
        };
        // Validate against the CURRENT policy to render grant/scope; on a disabled policy
        // fall back to the AS-fixed defaults so an existing client is still readable.
        let base = issuer_base_url(headers);
        let metadata = match load_enabled_policy(state, headers).await {
            Ok(policy) => mw_oauth::dcr::validate_metadata(&request, &policy)
                .unwrap_or_else(|_| default_metadata(&request)),
            Err(_) => default_metadata(&request),
        };
        Ok(mw_oauth::dcr::build_response(
            client_id,
            &request,
            &metadata,
            mw_oauth::dcr::registration_client_uri(&base, client_id),
            0,
            None,
            String::new(),
            client.created_at,
        ))
    }

    /// The AS-fixed metadata defaults (public PKCE `authorization_code`/`code` client).
    fn default_metadata(
        request: &mw_oauth::ClientRegistrationRequest,
    ) -> mw_oauth::dcr::ValidatedMetadata {
        mw_oauth::dcr::ValidatedMetadata {
            redirect_uris: request.redirect_uris.clone(),
            grant_types: vec!["authorization_code".into()],
            response_types: vec!["code".into()],
            token_endpoint_auth_method: "none".into(),
            granted_scope: mw_oauth::dcr::no_scope(),
            scope_display: String::new(),
        }
    }

    /// GET /oauth/register/{id} — read client config (RFC 7592).
    async fn dcr_read(
        State(state): State<AppState>,
        headers: axum::http::HeaderMap,
        UrlPath(id): UrlPath<String>,
    ) -> Response {
        let meta = match authenticate_dcr_client(&state, &headers, &id).await {
            Ok(m) => m,
            Err(resp) => return resp,
        };
        match dcr_client_response(&state, &headers, &id, &meta).await {
            Ok(resp) => Json(resp).into_response(),
            Err(resp) => resp,
        }
    }

    /// PUT /oauth/register/{id} — replace client config (RFC 7592).
    async fn dcr_update(
        State(state): State<AppState>,
        headers: axum::http::HeaderMap,
        UrlPath(id): UrlPath<String>,
        Json(request): Json<mw_oauth::ClientRegistrationRequest>,
    ) -> Response {
        let meta = match authenticate_dcr_client(&state, &headers, &id).await {
            Ok(m) => m,
            Err(resp) => return resp,
        };
        let policy = match load_enabled_policy(&state, &headers).await {
            Ok(p) => p,
            Err(resp) => return resp,
        };
        let metadata = match mw_oauth::dcr::validate_metadata(&request, &policy) {
            Ok(m) => m,
            Err(e) => return dcr_error_response(&e),
        };

        // Preserve the client's original created_at + approval sentinel.
        let existing = match state.v6.auth.store().get_client(&id).await {
            Ok(Some(c)) => c,
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "invalid_client_id" })),
                )
                    .into_response();
            }
            Err(e) => return server_error(e),
        };
        let updated = mw_oauth::OAuthClient {
            client_id: id.clone(),
            name: request.client_name.clone().unwrap_or_default(),
            redirect_uris: metadata.redirect_uris.clone(),
            approved_by: existing.approved_by,
            created_at: existing.created_at.clone(),
        };
        if let Err(e) = state.v6.auth.store().put_client(updated).await {
            return server_error(e);
        }

        // Update the RFC-7591 extras (the registration-access-token hash is unchanged).
        let new_meta = mw_store::OAuthClientMetaRow {
            client_id: id.clone(),
            registration_access_token_hash: meta.registration_access_token_hash.clone(),
            software_id: request.software_id.clone(),
            software_version: request.software_version.clone(),
            contacts_json: serde_json::to_string(&request.contacts).unwrap_or_else(|_| "[]".into()),
            created_via: meta.created_via.clone(),
            created_at: meta.created_at.clone(),
        };
        if let Err(e) = state.store.put_oauth_client_meta(&new_meta).await {
            return server_error(e);
        }

        let base = issuer_base_url(&headers);
        let resp = mw_oauth::dcr::build_response(
            &id,
            &request,
            &metadata,
            mw_oauth::dcr::registration_client_uri(&base, &id),
            0,
            None,
            String::new(),
            existing.created_at,
        );
        Json(resp).into_response()
    }

    /// DELETE /oauth/register/{id} — deprovision the client (RFC 7592).
    async fn dcr_delete(
        State(state): State<AppState>,
        headers: axum::http::HeaderMap,
        UrlPath(id): UrlPath<String>,
    ) -> Response {
        if let Err(resp) = authenticate_dcr_client(&state, &headers, &id).await {
            return resp;
        }
        if let Err(e) = state.store.delete_oauth_client_meta(&id).await {
            return server_error(e);
        }
        if let Err(e) = state.store.delete_oauth_client(&id).await {
            return server_error(e);
        }
        StatusCode::NO_CONTENT.into_response()
    }

    // ─── Admin enable route (RFC 7591 §DCR policy toggle) ─────────────────────
    //
    // Parity with `/admin/sso` (admin_sso.rs) + `/admin/ui-plugins` (ui_plugins.rs):
    // the `mw_admin_session` cookie + `admin.enabled` gate, fail-closed. This is the
    // in-band admin equivalent of writing the `oauth_dcr` policy row via config/CLI —
    // DCR stays DEFAULT-DISABLED and only an explicit admin `enabled: true` turns it on.
    //
    //   * `GET /admin/oauth-dcr` — read the current policy (no secrets to elide).
    //   * `PUT /admin/oauth-dcr` — upsert enabled + the redirect-host-suffix allowlist
    //     + the default granted scope (+ the optional initial-access-token requirement).

    /// The admin-session cookie, same name/gate as `admin_sso.rs` / `ui_plugins.rs`.
    const ADMIN_COOKIE: &str = "mw_admin_session";

    fn admin_unauthorized() -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "admin authentication required" })),
        )
            .into_response()
    }

    /// Extract the `mw_admin_session` cookie value (mirrors admin_sso.rs).
    fn admin_cookie(headers: &axum::http::HeaderMap) -> Option<String> {
        let raw = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
        for part in raw.split(';') {
            if let Some(v) = part.trim().strip_prefix(&format!("{ADMIN_COOKIE}="))
                && !v.is_empty()
            {
                return Some(v.to_string());
            }
        }
        None
    }

    /// Resolve the authenticated admin id, or a `401`. Enforces the `admin.enabled`
    /// gate (disabled panel ⇒ every admin route is `401`). Mirrors admin_sso.rs.
    async fn require_admin(
        state: &AppState,
        headers: &axum::http::HeaderMap,
    ) -> Result<String, Response> {
        if !state.v6.admin_enabled {
            return Err(admin_unauthorized());
        }
        let token = admin_cookie(headers).ok_or_else(admin_unauthorized)?;
        let hash = crate::push_relay::hash_token(&token);
        match state.store.get_admin_session(&hash).await {
            Ok(Some(admin_id)) => Ok(admin_id),
            _ => Err(admin_unauthorized()),
        }
    }

    /// Parse a stored JSON array (redirect-host suffixes) back to a `Value` array.
    fn parse_suffixes(s: &str) -> Value {
        serde_json::from_str::<Value>(s)
            .ok()
            .filter(Value::is_array)
            .unwrap_or_else(|| json!([]))
    }

    /// Parse the stored default-scope JSON back to a `Value` (null when malformed).
    fn parse_scope_value(s: &str) -> Value {
        serde_json::from_str::<Value>(s).unwrap_or(Value::Null)
    }

    /// Render a policy row (or the default-disabled shape) as the admin JSON view. The
    /// DCR policy holds no secrets, so every field is returned as-is.
    fn policy_view(row: Option<&mw_store::OAuthDcrPolicyRow>) -> Value {
        match row {
            Some(r) => json!({
                "enabled": r.enabled,
                "requireInitialAccessToken": r.require_initial_access_token,
                "allowedRedirectHostSuffixes": parse_suffixes(&r.allowed_redirect_host_suffixes_json),
                "defaultScope": parse_scope_value(&r.default_scope_json),
                "updatedAt": r.updated_at,
            }),
            None => json!({
                "enabled": false,
                "requireInitialAccessToken": false,
                "allowedRedirectHostSuffixes": [],
                "defaultScope": Value::Null,
                "updatedAt": Value::Null,
            }),
        }
    }

    /// `GET /admin/oauth-dcr` — read the current DCR policy (admin-gated). An absent
    /// policy row renders the default-DISABLED shape.
    async fn dcr_admin_get(
        State(state): State<AppState>,
        headers: axum::http::HeaderMap,
    ) -> Response {
        if let Err(resp) = require_admin(&state, &headers).await {
            return resp;
        }
        match state.store.get_oauth_dcr_policy().await {
            Ok(row) => Json(policy_view(row.as_ref())).into_response(),
            Err(e) => server_error(e),
        }
    }

    /// `PUT /admin/oauth-dcr` body: the whole policy. `enabled` is required; the rest
    /// default (empty allowlist, no default scope, no initial-access-token requirement).
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AdminDcrReq {
        /// Master switch: `false` (the default posture) keeps `/oauth/register` at 403.
        enabled: bool,
        /// Require the shared `MW_DCR_INITIAL_ACCESS_TOKEN` bearer on registration.
        #[serde(default)]
        require_initial_access_token: bool,
        /// The redirect-host-suffix allowlist (empty ⇒ every redirect is denied).
        #[serde(default)]
        allowed_redirect_host_suffixes: Vec<String>,
        /// The scope every DCR-issued client is granted (advisory request scope is
        /// never escalated beyond this). Omitted ⇒ the no-privilege default scope.
        /// Stored opaquely (the DCR core re-parses it to a [`Scope`] at register time,
        /// falling back to the no-privilege scope if it is not a full scope object),
        /// so this is deliberately permissive rather than a strict typed [`Scope`].
        #[serde(default)]
        default_scope: Option<Value>,
    }

    /// `PUT /admin/oauth-dcr` — upsert the DCR policy (admin-gated). Enabling DCR is an
    /// explicit admin action; this does not change the default-disabled posture.
    async fn dcr_admin_put(
        State(state): State<AppState>,
        headers: axum::http::HeaderMap,
        Json(body): Json<AdminDcrReq>,
    ) -> Response {
        if let Err(resp) = require_admin(&state, &headers).await {
            return resp;
        }
        let allowed_redirect_host_suffixes_json =
            serde_json::to_string(&body.allowed_redirect_host_suffixes)
                .unwrap_or_else(|_| "[]".to_string());
        let default_scope_json = match &body.default_scope {
            Some(scope) => serde_json::to_string(scope).unwrap_or_else(|_| "{}".to_string()),
            None => serde_json::to_string(&mw_oauth::dcr::no_scope())
                .unwrap_or_else(|_| "{}".to_string()),
        };
        let row = mw_store::OAuthDcrPolicyRow {
            enabled: body.enabled,
            require_initial_access_token: body.require_initial_access_token,
            allowed_redirect_host_suffixes_json,
            default_scope_json,
            updated_at: crate::push_relay::now_rfc3339(),
        };
        match state.store.put_oauth_dcr_policy(&row).await {
            Ok(()) => Json(policy_view(Some(&row))).into_response(),
            Err(e) => server_error(e),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use axum::http::{HeaderMap, HeaderValue, header};

        #[test]
        fn bearer_token_parses_case_insensitively() {
            let mut h = HeaderMap::new();
            h.insert(
                header::AUTHORIZATION,
                HeaderValue::from_static("Bearer tok-123"),
            );
            assert_eq!(bearer_token(&h).as_deref(), Some("tok-123"));
            h.insert(
                header::AUTHORIZATION,
                HeaderValue::from_static("bearer  tok-123 "),
            );
            assert_eq!(bearer_token(&h).as_deref(), Some("tok-123"));
            h.insert(header::AUTHORIZATION, HeaderValue::from_static("Basic abc"));
            assert!(bearer_token(&h).is_none());
            assert!(bearer_token(&HeaderMap::new()).is_none());
        }

        #[test]
        fn ct_eq_str_matches_only_equal() {
            assert!(ct_eq_str("secret", "secret"));
            assert!(!ct_eq_str("secret", "secreT"));
            assert!(!ct_eq_str("secret", "secret2"));
            assert!(!ct_eq_str("", "x"));
        }

        #[test]
        fn issuer_base_url_scheme_by_host() {
            let mut h = HeaderMap::new();
            h.insert(header::HOST, HeaderValue::from_static("mail.example.com"));
            assert_eq!(issuer_base_url(&h), "https://mail.example.com");
            h.insert(header::HOST, HeaderValue::from_static("127.0.0.1:8080"));
            assert_eq!(issuer_base_url(&h), "http://127.0.0.1:8080");
            h.insert(header::HOST, HeaderValue::from_static("localhost:3000"));
            assert_eq!(issuer_base_url(&h), "http://localhost:3000");
        }

        #[test]
        fn disabled_maps_to_403() {
            assert_eq!(dcr_disabled().status(), StatusCode::FORBIDDEN);
            assert_eq!(
                dcr_error_response(&mw_oauth::DcrError::Disabled).status(),
                StatusCode::FORBIDDEN
            );
        }

        #[test]
        fn dcr_error_status_mapping() {
            use mw_oauth::DcrError as E;
            assert_eq!(
                dcr_error_response(&E::InitialAccessTokenRequired).status(),
                StatusCode::UNAUTHORIZED
            );
            assert_eq!(
                dcr_error_response(&E::InvalidRedirectUri("x".into())).status(),
                StatusCode::BAD_REQUEST
            );
            assert_eq!(
                dcr_error_response(&E::InvalidClientMetadata("x".into())).status(),
                StatusCode::BAD_REQUEST
            );
            assert_eq!(
                dcr_error_response(&E::Store("x".into())).status(),
                StatusCode::INTERNAL_SERVER_ERROR
            );
        }

        // ── admin enable route (`GET/PUT /admin/oauth-dcr`) ───────────────────────
        //
        // Drive the REAL mounted router (`build_app_full` on a live socket, reqwest
        // over the wire) end-to-end: fail-closed without an admin session, then an
        // admin PUT enable flips `POST /oauth/register` 403→201, GET reflects it, and
        // an admin PUT disable returns it to 403.

        use std::net::SocketAddr;

        const TEST_KEY_HEX: &str =
            "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        const ADMIN_TOKEN: &str = "t11-e1-admin-token";

        fn unique_suffix() -> String {
            static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            format!(
                "{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            )
        }

        /// Boot the full app on a live socket + seed an admin session in the shared
        /// store. Returns `(base_url, admin_store)`; the admin cookie is [`ADMIN_TOKEN`].
        async fn spawn_with_admin() -> (String, mw_store::Store) {
            let tag = unique_suffix();
            let db = std::env::temp_dir()
                .join(format!("mw-t11e1-{tag}.db"))
                .to_string_lossy()
                .into_owned();
            let web_dir = std::env::temp_dir().join(format!("mw-t11e1-web-{tag}"));
            std::fs::create_dir_all(&web_dir).unwrap();
            std::fs::write(
                web_dir.join("index.html"),
                "<!doctype html><title>MW</title><div id=app>MW</div>",
            )
            .unwrap();

            let config = crate::AppConfig {
                db_path: db.clone(),
                server_key_hex: Some(TEST_KEY_HEX.into()),
                web_dir: Some(web_dir),
                cookie_secure: false,
                mode: crate::ServerMode::Proxy,
                hardening: crate::HardeningConfig::default(),
                security: crate::SecurityConfig::default(),
            };
            let v6 = crate::V6Config {
                admin_enabled: true,
                admin_username: Some("root".into()),
                admin_password: Some("hunter2".into()),
                redis_url: None,
            };
            let app = crate::build_app_full(config, v6)
                .await
                .expect("server boots")
                .0;

            // Seed an admin session directly (the login flow is out of scope here).
            let store =
                mw_store::Store::open(&db, mw_store::ServerKey::from_hex(TEST_KEY_HEX).unwrap())
                    .await
                    .expect("open store");
            let hash = crate::push_relay::hash_token(ADMIN_TOKEN);
            store
                .put_admin_session(&hash, "root", &crate::push_relay::now_rfc3339())
                .await
                .expect("seed admin session");

            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr: SocketAddr = listener.local_addr().unwrap();
            tokio::spawn(async move {
                axum::serve(listener, app).await.unwrap();
            });
            (format!("http://{addr}"), store)
        }

        fn admin_cookie_header() -> String {
            format!("{ADMIN_COOKIE}={ADMIN_TOKEN}")
        }

        #[tokio::test]
        async fn admin_route_is_fail_closed_without_admin_session() {
            let (base, _store) = spawn_with_admin().await;
            let c = reqwest::Client::new();

            // GET without any admin cookie → 401.
            let r = c
                .get(format!("{base}/admin/oauth-dcr"))
                .send()
                .await
                .unwrap();
            assert_eq!(r.status(), 401, "GET requires an admin session");

            // PUT without any admin cookie → 401 (and it must NOT enable DCR).
            let r = c
                .put(format!("{base}/admin/oauth-dcr"))
                .json(&json!({ "enabled": true }))
                .send()
                .await
                .unwrap();
            assert_eq!(r.status(), 401, "PUT requires an admin session");

            // A bogus (non-session) admin cookie is rejected too.
            let r = c
                .get(format!("{base}/admin/oauth-dcr"))
                .header(
                    reqwest::header::COOKIE,
                    format!("{ADMIN_COOKIE}=not-a-session"),
                )
                .send()
                .await
                .unwrap();
            assert_eq!(r.status(), 401, "an unknown admin token is rejected");

            // The fail-closed PUT never turned DCR on: register is still 403.
            let reg = c
                .post(format!("{base}/oauth/register"))
                .json(&json!({ "redirect_uris": ["https://apps.vogue-homes.com/cb"] }))
                .send()
                .await
                .unwrap();
            assert_eq!(reg.status(), 403, "DCR stays disabled after a rejected PUT");
        }

        #[tokio::test]
        async fn admin_put_enables_then_disables_dcr_registration() {
            let (base, _store) = spawn_with_admin().await;
            let c = reqwest::Client::new();
            let redirect = "https://apps.vogue-homes.com/cb";

            // ── Default-disabled: register is 403 before any admin action. ──────────
            let before = c
                .post(format!("{base}/oauth/register"))
                .json(&json!({ "redirect_uris": [redirect] }))
                .send()
                .await
                .unwrap();
            assert_eq!(before.status(), 403, "DCR is default-disabled");

            // ── Admin PUT enables + sets the allowlist → 200. ──────────────────────
            let put = c
                .put(format!("{base}/admin/oauth-dcr"))
                .header(reqwest::header::COOKIE, admin_cookie_header())
                .json(&json!({
                    "enabled": true,
                    "allowedRedirectHostSuffixes": ["vogue-homes.com"],
                    "defaultScope": { "read": true },
                }))
                .send()
                .await
                .unwrap();
            assert_eq!(put.status(), 200, "admin enable succeeds");
            let put_body: Value = put.json().await.unwrap();
            assert_eq!(put_body["enabled"], true);

            // ── Admin GET reflects the persisted policy. ───────────────────────────
            let get = c
                .get(format!("{base}/admin/oauth-dcr"))
                .header(reqwest::header::COOKIE, admin_cookie_header())
                .send()
                .await
                .unwrap();
            assert_eq!(get.status(), 200);
            let got: Value = get.json().await.unwrap();
            assert_eq!(got["enabled"], true, "GET reflects the enable");
            assert_eq!(got["allowedRedirectHostSuffixes"][0], "vogue-homes.com");
            assert_eq!(got["defaultScope"]["read"], true);

            // ── Register now transitions 403 → 201 (an allowlisted redirect). ──────
            let created = c
                .post(format!("{base}/oauth/register"))
                .json(&json!({ "redirect_uris": [redirect], "client_name": "t11 client" }))
                .send()
                .await
                .unwrap();
            assert_eq!(created.status(), 201, "enabling DCR admits registration");

            // ── Admin PUT disable → register returns to 403. ───────────────────────
            let off = c
                .put(format!("{base}/admin/oauth-dcr"))
                .header(reqwest::header::COOKIE, admin_cookie_header())
                .json(&json!({ "enabled": false }))
                .send()
                .await
                .unwrap();
            assert_eq!(off.status(), 200, "admin disable succeeds");

            let after = c
                .post(format!("{base}/oauth/register"))
                .json(&json!({ "redirect_uris": [redirect] }))
                .send()
                .await
                .unwrap();
            assert_eq!(after.status(), 403, "disabling DCR returns register to 403");
        }
    }
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
