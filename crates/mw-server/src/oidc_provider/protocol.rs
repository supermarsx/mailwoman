use super::*;
use axum::{
    Form,
    extract::{Query, State},
    response::Redirect,
    routing::{get, post},
};
use mw_store::oidc_provider::{
    OidcAuthorization, OidcBrowserFlow, OidcGrant, OidcIssueCode, OidcRedeem, OidcRedeemKind,
    OidcRedemption,
};
use serde::Deserialize;
const FLOW_COOKIE: &str = "mw_oidc_flow";
pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/oidc/.well-known/openid-configuration", get(discovery))
        .route("/oidc/jwks", get(jwks))
        .route("/oidc/authorize", get(authorize))
        .route("/oidc/context", get(context))
        .route("/oidc/decision", post(decision))
        .route("/oidc/token", post(token).options(preflight))
        .route("/oidc/userinfo", get(userinfo).options(preflight))
        .route("/oidc/revoke", post(revoke).options(preflight))
        .route("/oidc/grants", get(grants))
        .route("/oidc/grants/{id}", axum::routing::delete(revoke_grant))
        .route("/oidc/logout", get(logout_context).post(logout))
        .route("/oidc/logout/context", get(logout_review))
        .layer(axum::extract::DefaultBodyLimit::max(32768))
        .layer(axum::middleware::from_fn(protocol_admission))
}
async fn protocol_admission(req: axum::extract::Request, next: axum::middleware::Next) -> Response {
    use std::sync::{Arc, Mutex, OnceLock};
    static RATE: OnceLock<Mutex<crate::scope_mw::rate_limit::KeyedRateLimiter>> = OnceLock::new();
    static WORK: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
    if req.method() != axum::http::Method::POST
        || !matches!(req.uri().path(), "/oidc/token" | "/oidc/revoke")
    {
        return next.run(req).await;
    }
    let Some(source) = crate::scope_mw::proxy::client_ip(req.headers(), req.extensions()) else {
        return unavailable();
    };
    let allowed = RATE
        .get_or_init(|| {
            Mutex::new(crate::scope_mw::rate_limit::KeyedRateLimiter::new(
                30.0, 1.0, 10000,
            ))
        })
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .check(&source.to_canonical().to_string());
    let permit = WORK
        .get_or_init(|| Arc::new(tokio::sync::Semaphore::new(32)))
        .clone()
        .try_acquire_owned();
    if !allowed || permit.is_err() {
        let mut response = token_error("temporarily_unavailable", StatusCode::TOO_MANY_REQUESTS);
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, "1".parse().unwrap());
        return response;
    }
    let _permit = permit.expect("checked admission permit");
    next.run(req).await
}
fn unavailable() -> Response {
    token_error("temporarily_unavailable", StatusCode::SERVICE_UNAVAILABLE)
}
fn invalid() -> Response {
    token_error("invalid_request", StatusCode::BAD_REQUEST)
}
fn public_document(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
    response
}
async fn discovery(State(state): State<AppState>) -> Result<Response, Response> {
    let c = provider(&state).await?;
    Ok(public_document(Json(json!({"issuer":c.issuer,"authorization_endpoint":format!("{}/authorize",c.issuer),"token_endpoint":format!("{}/token",c.issuer),"userinfo_endpoint":format!("{}/userinfo",c.issuer),"jwks_uri":format!("{}/jwks",c.issuer),"revocation_endpoint":format!("{}/revoke",c.issuer),"end_session_endpoint":format!("{}/logout",c.issuer),"response_types_supported":["code"],"response_modes_supported":["query"],"grant_types_supported":["authorization_code","refresh_token"],"subject_types_supported":["public"],"id_token_signing_alg_values_supported":["RS256"],"token_endpoint_auth_methods_supported":["none","client_secret_basic","client_secret_post"],"code_challenge_methods_supported":["S256"],"scopes_supported":["openid","profile","email","offline_access"],"claims_supported":["iss","sub","aud","exp","iat","auth_time","nonce","sid","email","name","at_hash"]})).into_response()))
}
async fn jwks(State(state): State<AppState>) -> Result<Response, Response> {
    let c = provider(&state).await?;
    Ok(public_document(([(header::CACHE_CONTROL,"public, max-age=60")],Json(json!({"keys":state.store.oidc_public_keys(&c.issuer,now()).await.map_err(|_|unavailable())?}))).into_response()))
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Authorization {
    client_id: String,
    redirect_uri: String,
    response_type: String,
    scope: String,
    code_challenge: String,
    code_challenge_method: String,
    state: Option<String>,
    nonce: Option<String>,
    prompt: Option<String>,
    max_age: Option<u64>,
    response_mode: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
struct ValidatedRequest {
    started_at_millis: i64,
    parameters: Authorization,
    client_revision: String,
    provider_revision: String,
}
fn scopes(request: &Authorization) -> Vec<String> {
    request
        .scope
        .split_ascii_whitespace()
        .map(str::to_owned)
        .collect()
}
fn prompts(request: &Authorization) -> Vec<&str> {
    request
        .prompt
        .as_deref()
        .unwrap_or_default()
        .split_ascii_whitespace()
        .collect()
}
async fn validate(
    state: &AppState,
    request: &Authorization,
) -> Result<
    (
        mw_store::oidc_provider::OidcClientRow,
        ClientConfig,
        mw_store::oidc_provider::OidcProviderConfig,
    ),
    Response,
> {
    let provider = provider(state).await?;
    let client = state
        .store
        .oidc_client(&request.client_id)
        .await
        .map_err(|_| unavailable())?
        .filter(|c| c.enabled)
        .ok_or_else(invalid)?;
    let config: ClientConfig =
        serde_json::from_value(client.config.clone()).map_err(|_| unavailable())?;
    if !config.redirect_uris.contains(&request.redirect_uri) {
        return Err(invalid());
    }
    let scope = scopes(request);
    let prompt = prompts(request);
    if request.response_type != "code"
        || request
            .response_mode
            .as_deref()
            .is_some_and(|m| m != "query")
        || request.code_challenge_method != "S256"
        || request.code_challenge.len() != 43
        || !request
            .code_challenge
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        || request.state.as_ref().is_some_and(|v| v.len() > 2048)
        || request.nonce.as_ref().is_some_and(|v| v.len() > 512)
        || !scope.iter().any(|s| s == "openid")
        || scope.iter().any(|s| !config.scopes.contains(s))
        || scope.len() > 4
        || scope
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != scope.len()
        || prompt
            .iter()
            .any(|p| !matches!(*p, "none" | "login" | "consent" | "select_account"))
        || (prompt.contains(&"none") && prompt.len() != 1)
    {
        return Err(invalid());
    }
    Ok((client, config, provider))
}
fn cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| {
            let (name, value) = part.trim().split_once('=')?;
            (name == FLOW_COOKIE
                && value.len() == 43
                && value
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'))
            .then(|| value.to_owned())
        })
}
fn error_redirect(request: &Authorization, code: &str) -> Result<Response, Response> {
    let mut values = vec![("error", code)];
    if let Some(state) = &request.state {
        values.push(("state", state));
    }
    Ok(no_store(Redirect::to(&redirect_response(
        &request.redirect_uri,
        &values,
    )?)))
}
async fn authorize(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    Query(request): Query<Authorization>,
) -> Result<Response, Response> {
    let (client, _, provider) = validate(&state, &request).await?;
    let session = browser_session(&state, &headers).await.ok();
    if prompts(&request).contains(&"none") {
        let Some(session) = session.as_ref() else {
            return error_redirect(&request, "login_required");
        };
        let Some(principal) = state
            .store
            .primary_principal(&session.id)
            .await
            .map_err(|_| unavailable())?
        else {
            return error_redirect(&request, "login_required");
        };
        if request
            .max_age
            .is_some_and(|age| now().saturating_sub(principal.auth_time) as u64 > age)
        {
            return error_redirect(&request, "login_required");
        }
        // This issuer always obtains explicit consent, including offline access.
        return error_redirect(&request, "consent_required");
    }
    let browser = cookie(&headers).unwrap_or_else(|| opaque(""));
    let handle = opaque("flow_");
    let Some(source) = crate::scope_mw::proxy::client_ip(&headers, &ext) else {
        return Err(unavailable());
    };
    let mut source_input = state
        .store
        .signin_challenge_secret("oidc-flow-source")
        .into_bytes();
    source_input.extend_from_slice(source.to_canonical().to_string().as_bytes());
    let source_hash = Sha256::digest(source_input)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let validated = ValidatedRequest {
        started_at_millis: chrono::Utc::now().timestamp_millis(),
        parameters: request,
        client_revision: client.revision,
        provider_revision: provider.revision,
    };
    let flow = OidcBrowserFlow {
        flow_hash: hash(&handle),
        browser_hash: hash(&browser),
        source_hash,
        request: serde_json::to_value(validated).map_err(|_| invalid())?,
        initial_session_id: session.map(|s| s.id),
        created_at: now(),
        expires_at: now() + 600,
    };
    if !state
        .store
        .begin_oidc_browser_flow(&flow, now())
        .await
        .map_err(|_| unavailable())?
    {
        return Err(no_store((
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, "60")],
            Json(json!({"error":"temporarily_unavailable"})),
        )));
    }
    let mut response = no_store(Redirect::to(&format!(
        "{}/oidc/consent?flow={handle}",
        state.base_path
    )));
    let secure = if state.cookie_secure { "; Secure" } else { "" };
    response.headers_mut().append(
        header::SET_COOKIE,
        format!(
            "{FLOW_COOKIE}={browser}; Path={}/oidc; HttpOnly; SameSite=Lax; Max-Age=600{secure}",
            state.base_path
        )
        .parse()
        .map_err(|_| invalid())?,
    );
    Ok(response)
}
#[derive(Deserialize)]
struct FlowQuery {
    flow: String,
}
async fn flow(
    state: &AppState,
    headers: &HeaderMap,
    handle: &str,
) -> Result<(OidcBrowserFlow, ValidatedRequest), Response> {
    if handle.len() > 128 {
        return Err(invalid());
    }
    let browser = cookie(headers).ok_or_else(invalid)?;
    let row = state
        .store
        .oidc_browser_flow(&hash(handle), &hash(&browser), now())
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(|| token_error("expired_request", StatusCode::BAD_REQUEST))?;
    let request: ValidatedRequest =
        serde_json::from_value(row.request.clone()).map_err(|_| invalid())?;
    let (client, _, provider) = validate(state, &request.parameters).await?;
    if client.revision != request.client_revision || provider.revision != request.provider_revision
    {
        return Err(invalid());
    }
    Ok((row, request))
}
async fn context(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<FlowQuery>,
) -> Result<Response, Response> {
    let (flow, request) = flow(&state, &headers, &query.flow).await?;
    let session = match browser_session(&state, &headers).await {
        Ok(s) => s,
        Err(_) => return Ok(no_store(Json(json!({"loginRequired":true})))),
    };
    let principal = state
        .store
        .primary_principal(&session.id)
        .await
        .map_err(|_| unavailable())?;
    let clock = state
        .store
        .session_activity_clock(&session.id)
        .await
        .map_err(|_| unavailable())?;
    let fresh = clock
        .as_ref()
        .and_then(|(created, _)| chrono::DateTime::parse_from_rfc3339(created).ok())
        .is_some_and(|created| created.timestamp_millis() >= request.started_at_millis)
        && flow.initial_session_id.as_deref() != Some(session.id.as_str());
    let reauthenticate = principal.as_ref().is_none_or(|p| {
        !fresh
            && request
                .parameters
                .max_age
                .is_some_and(|age| age == 0 || now().saturating_sub(p.auth_time) as u64 > age)
    }) || (!fresh
        && (prompts(&request.parameters).contains(&"login")
            || prompts(&request.parameters).contains(&"select_account")));
    if reauthenticate {
        return Ok(no_store(Json(
            json!({"loginRequired":true,"reauthenticationRequired":true}),
        )));
    }
    let principal = principal.ok_or_else(invalid)?;
    if !state
        .store
        .begin_oidc_authorization(
            &OidcAuthorization {
                request_hash: flow.flow_hash.clone(),
                session_id: session.id.clone(),
                client_id: request.parameters.client_id.clone(),
                client_revision: request.client_revision.clone(),
                provider_revision: request.provider_revision.clone(),
                request: serde_json::to_value(&request).map_err(|_| invalid())?,
                expires_at: flow.expires_at,
            },
            now(),
        )
        .await
        .map_err(|_| unavailable())?
    {
        return Err(invalid());
    }
    let client = state
        .store
        .get_oauth_client(&request.parameters.client_id)
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(invalid)?;
    Ok(no_store(Json(
        json!({"loginRequired":false,"clientName":client.name,"clientId":client.client_id,"redirectUri":request.parameters.redirect_uri,"scopes":scopes(&request.parameters),"subject":principal.principal_id,"account":session.username,"csrfToken":state.store.session_csrf_token(&session.id)}),
    )))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Decision {
    flow: String,
    approve: bool,
}
async fn decision(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Decision>,
) -> Result<Response, Response> {
    let session = browser_session(&state, &headers).await?;
    if !crate::hardening::csrf_request_ok(&headers, &state.store, false) {
        return Err(token_error("invalid_request", StatusCode::FORBIDDEN));
    }
    let (flow, request) = flow(&state, &headers, &body.flow).await?;
    let stored = state
        .store
        .oidc_authorization(&flow.flow_hash, &session.id, now())
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(invalid)?;
    let principal = state
        .store
        .primary_principal(&session.id)
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(invalid)?;
    if !body.approve {
        if !state
            .store
            .deny_oidc_authorization(&stored.request_hash, &session.id, now())
            .await
            .map_err(|_| unavailable())?
        {
            return Err(invalid());
        }
        let mut values = vec![("error", "access_denied")];
        if let Some(state) = &request.parameters.state {
            values.push(("state", state));
        }
        return Ok(no_store(Json(
            json!({"redirectUri":redirect_response(&request.parameters.redirect_uri,&values)?}),
        )));
    }
    let raw = opaque("oidc_code_");
    let mut claims = json!({});
    let scope = scopes(&request.parameters);
    if scope.iter().any(|s| s == "email") && session.username.contains('@') {
        claims["email"] = json!(session.username);
    }
    // A display label is not a verified legal name; omit the optional name claim.
    let grant = OidcGrant {
        id: opaque("grant_"),
        client_id: request.parameters.client_id.clone(),
        client_revision: request.client_revision,
        provider_revision: request.provider_revision,
        principal_id: principal.principal_id,
        session_id: session.id,
        claims,
        scopes: scope,
        nonce: request.parameters.nonce.clone(),
        auth_time: principal.auth_time,
        created_at: now(),
        expires_at: principal.auth_time.saturating_add(1296000),
    };
    if !state
        .store
        .issue_oidc_code(OidcIssueCode {
            request_hash: &stored.request_hash,
            grant: &grant,
            code_hash: &hash(&raw),
            redirect_uri: &request.parameters.redirect_uri,
            pkce_challenge: &request.parameters.code_challenge,
            now: now(),
        })
        .await
        .map_err(|_| unavailable())?
    {
        return Err(invalid());
    }
    let mut values = vec![("code", raw.as_str())];
    if let Some(state) = &request.parameters.state {
        values.push(("state", state));
    }
    Ok(no_store(Json(
        json!({"redirectUri":redirect_response(&request.parameters.redirect_uri,&values)?}),
    )))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TokenRequest {
    grant_type: String,
    client_id: Option<String>,
    client_secret: Option<String>,
    code: Option<String>,
    redirect_uri: Option<String>,
    code_verifier: Option<String>,
    refresh_token: Option<String>,
    scope: Option<String>,
}
fn secret_matches(expected: &str, raw: &str) -> bool {
    use hmac::{Hmac, KeyInit, Mac};
    let mut expected_mac = Hmac::<Sha256>::new_from_slice(b"mailwoman-oidc-secret-compare-v1")
        .expect("fixed HMAC key");
    expected_mac.update(expected.as_bytes());
    let expected_mac = expected_mac.finalize().into_bytes();
    let mut actual = Hmac::<Sha256>::new_from_slice(b"mailwoman-oidc-secret-compare-v1")
        .expect("fixed HMAC key");
    actual.update(hash(raw).as_bytes());
    actual.verify_slice(expected_mac.as_ref()).is_ok()
}
fn decode_basic_part(raw: &str) -> Result<String, Response> {
    let url = reqwest::Url::parse(&format!("https://decode.invalid/?value={raw}"))
        .map_err(|_| invalid())?;
    let pairs: Vec<_> = url.query_pairs().collect();
    if pairs.len() != 1 || pairs[0].0 != "value" {
        return Err(invalid());
    }
    Ok(pairs[0].1.to_string())
}
fn client_origin(headers: &HeaderMap, config: &ClientConfig) -> Result<Option<String>, Response> {
    let Some(origin) = headers.get(header::ORIGIN) else {
        return Ok(None);
    };
    let origin = origin.to_str().map_err(|_| invalid())?;
    if !config
        .redirect_uris
        .iter()
        .filter_map(|uri| reqwest::Url::parse(uri).ok())
        .any(|uri| uri.origin().ascii_serialization() == origin)
    {
        return Err(token_error("invalid_client", StatusCode::UNAUTHORIZED));
    }
    Ok(Some(origin.into()))
}
fn cors(mut response: Response, origin: Option<&str>) -> Response {
    if let Some(origin) = origin
        && let Ok(value) = origin.parse()
    {
        response
            .headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, value);
        response
            .headers_mut()
            .insert(header::VARY, "Origin".parse().unwrap());
    }
    response
}
async fn authenticate_client(
    state: &AppState,
    headers: &HeaderMap,
    id: Option<&str>,
    secret: Option<&str>,
) -> Result<
    (
        mw_store::oidc_provider::OidcClientRow,
        ClientConfig,
        Option<String>,
    ),
    Response,
> {
    let basic = headers
        .get(header::AUTHORIZATION)
        .map(|value| value.to_str().map_err(|_| invalid()))
        .transpose()?;
    let (client_id, supplied, method) = if let Some(basic) = basic {
        if secret.is_some() {
            return Err(invalid());
        }
        let encoded = basic.strip_prefix("Basic ").ok_or_else(invalid)?;
        if encoded.len() > 4096 {
            return Err(invalid());
        }
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|_| invalid())?;
        let decoded = std::str::from_utf8(&decoded).map_err(|_| invalid())?;
        let (client, secret) = decoded.split_once(':').ok_or_else(invalid)?;
        let client = decode_basic_part(client)?;
        if id.is_some_and(|id| id != client) {
            return Err(invalid());
        }
        (
            client,
            Some(decode_basic_part(secret)?),
            ClientAuth::ClientSecretBasic,
        )
    } else {
        (
            id.ok_or_else(invalid)?.to_owned(),
            secret.map(str::to_owned),
            if secret.is_some() {
                ClientAuth::ClientSecretPost
            } else {
                ClientAuth::None
            },
        )
    };
    if client_id.len() > 256 || supplied.as_ref().is_some_and(|secret| secret.len() > 512) {
        return Err(invalid());
    }
    let client = state
        .store
        .oidc_client(&client_id)
        .await
        .map_err(|_| unavailable())?
        .filter(|c| c.enabled)
        .ok_or_else(|| token_error("invalid_client", StatusCode::UNAUTHORIZED))?;
    let config: ClientConfig =
        serde_json::from_value(client.config.clone()).map_err(|_| unavailable())?;
    if config.auth_method != method
        || (method != ClientAuth::None
            && !client
                .secret_hash
                .as_deref()
                .zip(supplied.as_deref())
                .is_some_and(|(expected, raw)| secret_matches(expected, raw)))
    {
        return Err(token_error("invalid_client", StatusCode::UNAUTHORIZED));
    }
    let origin = client_origin(headers, &config)?;
    Ok((client, config, origin))
}
async fn grant_session(state: &AppState, grant: &OidcGrant) -> Result<mw_store::Session, Response> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::COOKIE,
        format!("{}={}", crate::COOKIE_NAME, grant.session_id)
            .parse()
            .map_err(|_| invalid())?,
    );
    let session = crate::authed(state, &headers)
        .await
        .map_err(|_| token_error("invalid_grant", StatusCode::BAD_REQUEST))?;
    let principal = state
        .store
        .primary_principal(&session.id)
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(|| token_error("invalid_grant", StatusCode::BAD_REQUEST))?;
    if principal.principal_id != grant.principal_id || principal.auth_time != grant.auth_time {
        return Err(token_error("invalid_grant", StatusCode::BAD_REQUEST));
    }
    Ok(session)
}
fn permitted_claims(grant: &OidcGrant) -> Value {
    let mut claims = grant.claims.clone();
    if let Some(object) = claims.as_object_mut() {
        if !grant.scopes.iter().any(|scope| scope == "email") {
            object.remove("email");
            object.remove("email_verified");
        }
        if !grant.scopes.iter().any(|scope| scope == "profile") {
            object.remove("name");
        }
    }
    claims
}
async fn token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(body): Form<TokenRequest>,
) -> Result<Response, Response> {
    let provider = provider(&state).await?;
    let (client, _, origin) = authenticate_client(
        &state,
        &headers,
        body.client_id.as_deref(),
        body.client_secret.as_deref(),
    )
    .await?;
    let requested_scope = body.scope.as_deref().map(|scope| {
        scope
            .split_ascii_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    });
    if body.scope.as_ref().is_some_and(|scope| scope.len() > 256)
        || body.scope.is_some() && body.grant_type != "refresh_token"
    {
        return Err(token_error("invalid_scope", StatusCode::BAD_REQUEST));
    }
    let access = opaque("oidc_access_");
    let refresh = opaque("oidc_refresh_");
    let challenge;
    let presented;
    let kind = match body.grant_type.as_str() {
        "authorization_code" => {
            let code = body
                .code
                .as_deref()
                .filter(|c| c.starts_with("oidc_code_") && c.len() < 128)
                .ok_or_else(invalid)?;
            let verifier = body
                .code_verifier
                .as_deref()
                .filter(|v| {
                    (43..=128).contains(&v.len())
                        && v.bytes().all(|b| {
                            b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~')
                        })
                })
                .ok_or_else(invalid)?;
            challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .encode(Sha256::digest(verifier.as_bytes()));
            presented = hash(code);
            OidcRedeemKind::Code {
                redirect_uri: body.redirect_uri.as_deref().ok_or_else(invalid)?,
                pkce_challenge: &challenge,
            }
        }
        "refresh_token" => {
            presented = hash(
                body.refresh_token
                    .as_deref()
                    .filter(|t| t.starts_with("oidc_refresh_") && t.len() < 128)
                    .ok_or_else(invalid)?,
            );
            OidcRedeemKind::Refresh
        }
        _ => {
            return Err(token_error(
                "unsupported_grant_type",
                StatusCode::BAD_REQUEST,
            ));
        }
    };
    let redeemed = state
        .store
        .redeem_oidc_token(OidcRedeem {
            scope: requested_scope.as_deref(),
            presented_hash: &presented,
            client_id: &client.client_id,
            client_revision: &client.revision,
            kind,
            access_hash: &hash(&access),
            refresh_hash: Some(&hash(&refresh)),
            now: now(),
        })
        .await
        .map_err(|_| unavailable())?;
    if matches!(redeemed, OidcRedemption::InvalidScope) {
        return Ok(cors(
            token_error("invalid_scope", StatusCode::BAD_REQUEST),
            origin.as_deref(),
        ));
    }
    let OidcRedemption::Issued(grant) = redeemed else {
        return Ok(cors(
            token_error("invalid_grant", StatusCode::BAD_REQUEST),
            origin.as_deref(),
        ));
    };
    grant_session(&state, &grant).await?;
    let key = state
        .store
        .oidc_signing_keys(&provider.issuer, now())
        .await
        .map_err(|_| unavailable())?
        .into_iter()
        .find(|key| key.active)
        .ok_or_else(unavailable)?;
    let mut claims = permitted_claims(&grant);
    claims["iss"] = json!(provider.issuer);
    claims["sub"] = json!(grant.principal_id);
    claims["aud"] = json!([client.client_id]);
    claims["iat"] = json!(now());
    claims["exp"] = json!((now() + 300).min(grant.expires_at));
    claims["auth_time"] = json!(grant.auth_time);
    claims["sid"] = json!(grant.id);
    if let Some(nonce) = &grant.nonce {
        claims["nonce"] = json!(nonce);
    }
    let pem = std::str::from_utf8(&key.private_pem).map_err(|_| unavailable())?;
    let id_token =
        mw_sso::issuer::sign_id_token(pem, &key.kid, claims, &access).map_err(|_| unavailable())?;
    let mut response = json!({"access_token":access,"token_type":"Bearer","expires_in":(grant.expires_at-now()).min(300),"scope":grant.scopes.join(" ")});
    if grant.scopes.iter().any(|s| s == "openid") {
        response["id_token"] = json!(id_token);
    }
    if grant.scopes.iter().any(|s| s == "offline_access") {
        response["refresh_token"] = json!(refresh);
    }
    Ok(cors(no_store(Json(response)), origin.as_deref()))
}
async fn userinfo(State(state): State<AppState>, headers: HeaderMap) -> Result<Response, Response> {
    provider(&state).await?;
    let raw = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .filter(|v| v.starts_with("oidc_access_") && v.len() < 128)
        .ok_or_else(|| token_error("invalid_token", StatusCode::UNAUTHORIZED))?;
    let grant = state
        .store
        .oidc_access_grant(&hash(raw), now())
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(|| token_error("invalid_token", StatusCode::UNAUTHORIZED))?;
    grant_session(&state, &grant)
        .await
        .map_err(|_| token_error("invalid_token", StatusCode::UNAUTHORIZED))?;
    let client = state
        .store
        .oidc_client(&grant.client_id)
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(invalid)?;
    let config: ClientConfig = serde_json::from_value(client.config).map_err(|_| unavailable())?;
    let origin = client_origin(&headers, &config)?;
    let mut claims = permitted_claims(&grant);
    claims["sub"] = json!(grant.principal_id);
    Ok(cors(no_store(Json(claims)), origin.as_deref()))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RevokeRequest {
    token: String,
    client_id: Option<String>,
    client_secret: Option<String>,
    token_type_hint: Option<String>,
}
async fn revoke(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(body): Form<RevokeRequest>,
) -> Result<Response, Response> {
    provider(&state).await?;
    let (client, _, origin) = authenticate_client(
        &state,
        &headers,
        body.client_id.as_deref(),
        body.client_secret.as_deref(),
    )
    .await?;
    if body.token.len() > 512
        || body
            .token_type_hint
            .as_deref()
            .is_some_and(|hint| !matches!(hint, "access_token" | "refresh_token"))
    {
        return Err(invalid());
    }
    state
        .store
        .revoke_oidc_token(&hash(&body.token), &client.client_id, now())
        .await
        .map_err(|_| unavailable())?;
    Ok(cors(no_store(StatusCode::OK), origin.as_deref()))
}
async fn grants(State(state): State<AppState>, headers: HeaderMap) -> Result<Response, Response> {
    let session = browser_session(&state, &headers).await?;
    let principal = state
        .store
        .primary_principal(&session.id)
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(invalid)?;
    Ok(no_store(Json(
        json!({"grants":state.store.oidc_consents(&principal.principal_id).await.map_err(|_|unavailable())?, "csrfToken":state.store.session_csrf_token(&session.id)}),
    )))
}
async fn revoke_grant(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Response, Response> {
    let session = browser_session(&state, &headers).await?;
    if !crate::hardening::csrf_request_ok(&headers, &state.store, false) {
        return Err(token_error("invalid_request", StatusCode::FORBIDDEN));
    }
    let principal = state
        .store
        .primary_principal(&session.id)
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(invalid)?;
    state
        .store
        .revoke_oidc_consent(&principal.principal_id, &id, now())
        .await
        .map_err(|_| unavailable())?;
    Ok(no_store(StatusCode::NO_CONTENT))
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LogoutRequest {
    client_id: Option<String>,
    id_token_hint: Option<String>,
    post_logout_redirect_uri: Option<String>,
    state: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct LogoutFlow {
    kind: String,
    session_id: Option<String>,
    subject: Option<String>,
    redirect_uri: Option<String>,
    state: Option<String>,
}
// The unsigned audience is only a bounded client lookup hint. The complete
// token is subsequently verified against that exact client and issuer.
fn hinted_client(token: &str) -> Option<String> {
    if token.len() > 16384 {
        return None;
    }
    let payload = token.split('.').nth(1)?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .ok()?;
    let claims: Value = serde_json::from_slice(&bytes).ok()?;
    let audience = claims.get("aud")?;
    let id = audience.as_str().or_else(|| {
        audience
            .as_array()
            .filter(|a| a.len() == 1)?
            .first()?
            .as_str()
    })?;
    (id.len() <= 256).then(|| id.to_owned())
}
async fn logout_context(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    Query(request): Query<LogoutRequest>,
) -> Result<Response, Response> {
    let provider = provider(&state).await?;
    let session = browser_session(&state, &headers).await.ok();
    let mut subject = None;
    if request.state.as_ref().is_some_and(|v| v.len() > 2048)
        || request
            .id_token_hint
            .as_ref()
            .is_some_and(|v| v.len() > 16384)
    {
        return Err(invalid());
    }
    let mut redirect = None;
    let client_id = request
        .client_id
        .clone()
        .or_else(|| request.id_token_hint.as_deref().and_then(hinted_client));
    if let Some(id) = client_id.as_deref() {
        let client = state
            .store
            .oidc_client(id)
            .await
            .map_err(|_| unavailable())?
            .filter(|c| c.enabled)
            .ok_or_else(invalid)?;
        let config: ClientConfig =
            serde_json::from_value(client.config).map_err(|_| unavailable())?;
        if let Some(hint) = request.id_token_hint.as_deref() {
            let keys = state
                .store
                .oidc_public_keys(&provider.issuer, now())
                .await
                .map_err(|_| unavailable())?;
            let claims = mw_sso::issuer::verify_id_token(
                hint,
                &provider.issuer,
                id,
                json!({"keys":keys}),
                None,
            )
            .map_err(|_| invalid())?;
            subject = Some(
                claims
                    .get("sub")
                    .and_then(Value::as_str)
                    .ok_or_else(invalid)?
                    .to_owned(),
            );
        }
        if let Some(uri) = request.post_logout_redirect_uri.as_ref() {
            if !config.post_logout_redirect_uris.contains(uri) {
                return Err(invalid());
            }
            redirect = Some(uri.clone());
        }
    } else if request.post_logout_redirect_uri.is_some() || request.id_token_hint.is_some() {
        return Err(invalid());
    }
    let browser = cookie(&headers).unwrap_or_else(|| opaque(""));
    let handle = opaque("logout_");
    let source = crate::scope_mw::proxy::client_ip(&headers, &ext).ok_or_else(unavailable)?;
    let source_hash = hash(&format!(
        "{}:{}",
        state.store.signin_challenge_secret("oidc-flow-source"),
        source.to_canonical()
    ));
    let row = OidcBrowserFlow {
        flow_hash: hash(&handle),
        browser_hash: hash(&browser),
        source_hash,
        request: serde_json::to_value(LogoutFlow {
            kind: "logout".into(),
            session_id: session.as_ref().map(|s| s.id.clone()),
            subject,
            redirect_uri: redirect,
            state: request.state,
        })
        .map_err(|_| invalid())?,
        initial_session_id: session.map(|s| s.id),
        created_at: now(),
        expires_at: now() + 600,
    };
    if !state
        .store
        .begin_oidc_browser_flow(&row, now())
        .await
        .map_err(|_| unavailable())?
    {
        return Err(token_error(
            "temporarily_unavailable",
            StatusCode::TOO_MANY_REQUESTS,
        ));
    }
    let mut response = no_store(Redirect::to(&format!(
        "{}/oidc/consent?logout={handle}",
        state.base_path
    )));
    let secure = if state.cookie_secure { "; Secure" } else { "" };
    response.headers_mut().append(
        header::SET_COOKIE,
        format!(
            "{FLOW_COOKIE}={browser}; Path={}/oidc; HttpOnly; SameSite=Lax; Max-Age=600{secure}",
            state.base_path
        )
        .parse()
        .map_err(|_| invalid())?,
    );
    Ok(response)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LogoutDecision {
    flow: String,
    approve: bool,
}
async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<LogoutDecision>,
) -> Result<Response, Response> {
    let session = browser_session(&state, &headers).await?;
    if !crate::hardening::csrf_request_ok(&headers, &state.store, false) {
        return Err(token_error("invalid_request", StatusCode::FORBIDDEN));
    }
    let browser = cookie(&headers).ok_or_else(invalid)?;
    let flow = state
        .store
        .oidc_browser_flow(&hash(&body.flow), &hash(&browser), now())
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(invalid)?;
    let request: LogoutFlow = serde_json::from_value(flow.request).map_err(|_| invalid())?;
    let principal = state
        .store
        .primary_principal(&session.id)
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(invalid)?;
    if request.kind != "logout"
        || request
            .session_id
            .as_deref()
            .is_some_and(|id| id != session.id)
        || request
            .subject
            .as_deref()
            .is_some_and(|subject| subject != principal.principal_id)
    {
        return Err(invalid());
    }
    if !state
        .store
        .consume_oidc_browser_flow(&flow.flow_hash, &flow.browser_hash, now())
        .await
        .map_err(|_| unavailable())?
    {
        return Err(invalid());
    }
    if !body.approve {
        return Ok(no_store(Json(
            json!({"redirectUri":format!("{}/",state.base_path)}),
        )));
    }
    state
        .store
        .delete_session(&session.id)
        .await
        .map_err(|_| unavailable())?;
    state.sessions.forget(&session.id);
    let redirect = match request.redirect_uri {
        Some(uri) => {
            let values = request
                .state
                .as_deref()
                .map(|value| vec![("state", value)])
                .unwrap_or_default();
            redirect_response(&uri, &values)?
        }
        None => format!("{}/", state.base_path),
    };
    let mut response = no_store(Json(json!({"redirectUri":redirect})));
    response
        .headers_mut()
        .append(header::SET_COOKIE, crate::clear_cookie(state.cookie_secure));
    Ok(response)
}
async fn logout_review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<FlowQuery>,
) -> Result<Response, Response> {
    let session = browser_session(&state, &headers).await?;
    let browser = cookie(&headers).ok_or_else(invalid)?;
    let row = state
        .store
        .oidc_browser_flow(&hash(&query.flow), &hash(&browser), now())
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(invalid)?;
    let request: LogoutFlow = serde_json::from_value(row.request).map_err(|_| invalid())?;
    let principal = state
        .store
        .primary_principal(&session.id)
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(invalid)?;
    if request.kind != "logout"
        || request
            .session_id
            .as_deref()
            .is_some_and(|id| id != session.id)
        || request
            .subject
            .as_deref()
            .is_some_and(|subject| subject != principal.principal_id)
    {
        return Err(invalid());
    }
    Ok(no_store(Json(
        json!({"account":session.username,"redirectUri":request.redirect_uri,"csrfToken":state.store.session_csrf_token(&session.id)}),
    )))
}
async fn preflight(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, Response> {
    provider(&state).await?;
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(invalid)?;
    let requested = headers
        .get(header::ACCESS_CONTROL_REQUEST_METHOD)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(invalid)?;
    if !matches!(requested, "GET" | "POST") {
        return Err(invalid());
    }
    if headers
        .get(header::ACCESS_CONTROL_REQUEST_HEADERS)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(',').any(|name| {
                !matches!(
                    name.trim().to_ascii_lowercase().as_str(),
                    "content-type" | "authorization"
                )
            })
        })
    {
        return Err(invalid());
    }
    let mut allowed = false;
    for client in state
        .store
        .oidc_clients()
        .await
        .map_err(|_| unavailable())?
    {
        if client.enabled
            && let Ok(config) = serde_json::from_value::<ClientConfig>(client.config)
            && client_origin(&headers, &config).is_ok()
        {
            allowed = true;
            break;
        }
    }
    if !allowed {
        return Err(token_error("invalid_client", StatusCode::FORBIDDEN));
    }
    let mut response = cors(no_store(StatusCode::NO_CONTENT), Some(origin));
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        "GET, POST, OPTIONS".parse().unwrap(),
    );
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        "authorization, content-type".parse().unwrap(),
    );
    Ok(response)
}
