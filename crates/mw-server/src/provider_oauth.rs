//! Session-bound first-party provider enrollment. Pending device codes, PKCE
//! verifiers and tokens remain server-side; authenticated JSON replies are no-store.
use crate::v7_mount::oauth_client::{DevicePoll, OauthClient, ReqwestPoster, TokenResponse};
use crate::{AppState, plugins::PluginRegistry};
use axum::extract::{Extension, Path, Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use base64::Engine as _;
use mw_store::{BridgeAccountRow, BridgeOauthTokenRow, Session};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
#[cfg(test)]
#[allow(dead_code)]
#[path = "../../mw-store/src/test_db.rs"]
mod test_db;

#[derive(Clone)]
enum Mode {
    Complete,
    Code {
        verifier: String,
        state: String,
        redirect: String,
    },
    Device {
        code: String,
        interval: u64,
        next: Instant,
    },
}
#[derive(Clone)]
struct Flow {
    busy: bool,
    installed: bool,
    committing: bool,
    account: String,
    session: String,
    authority: String,
    binding: BridgeAccountRow,
    previous: Option<BridgeAccountRow>,
    expires: Instant,
    mode: Mode,
}
fn flows() -> &'static Mutex<HashMap<String, Flow>> {
    static FLOWS: OnceLock<Mutex<HashMap<String, Flow>>> = OnceLock::new();
    FLOWS.get_or_init(Default::default)
}
fn opaque() -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(rand::random::<[u8; 32]>())
}
fn claim_flow(id: &str, account: &str, session: &str, authority: &str) -> Result<Flow, Response> {
    let mut held = flows().lock().unwrap();
    let flow = held
        .get_mut(id)
        .filter(|flow| {
            flow.account == account
                && flow.session == session
                && flow.authority == authority
                && flow.expires > Instant::now()
        })
        .ok_or_else(|| error(StatusCode::BAD_REQUEST, "provider_flow_expired"))?;
    if flow.busy {
        return Err(error(StatusCode::CONFLICT, "provider_busy"));
    }
    flow.busy = true;
    Ok(flow.clone())
}
struct ClaimGuard(String);
impl Drop for ClaimGuard {
    fn drop(&mut self) {
        if let Some(flow) = flows().lock().unwrap().get_mut(&self.0) {
            flow.busy = false;
            flow.committing = false;
        }
    }
}
fn begin_commit(id: &str) -> Result<(), Response> {
    let mut held = flows().lock().unwrap();
    let flow = held
        .get_mut(id)
        .filter(|flow| flow.expires > Instant::now())
        .ok_or_else(|| error(StatusCode::BAD_REQUEST, "provider_flow_expired"))?;
    flow.committing = true;
    Ok(())
}
fn error(status: StatusCode, code: &str) -> Response {
    let mut response = reply(json!({"code":code,"error":code}));
    *response.status_mut() = status;
    response
}
fn reply(value: Value) -> Response {
    let mut reply = Json(value).into_response();
    reply.headers_mut().insert(
        "cache-control",
        axum::http::HeaderValue::from_static("no-store"),
    );
    reply
}

async fn owner(state: &AppState, headers: &HeaderMap) -> Result<(Session, String), Response> {
    if headers.contains_key(crate::linked_accounts::SELECTOR) {
        return Err(error(StatusCode::BAD_REQUEST, "primary_account_required"));
    }
    let session = crate::authed(state, headers).await?;
    let authority = state
        .store
        .session_storage_authority(&session.id)
        .await
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "provider_unavailable"))?
        .ok_or_else(|| error(StatusCode::CONFLICT, "accountChanged"))?;
    if headers
        .get("x-mailwoman-storage-authority")
        .is_some_and(|held| held.to_str().ok() != Some(authority.as_str()))
    {
        return Err(error(StatusCode::CONFLICT, "accountChanged"));
    }
    Ok((session, authority))
}

async fn admit(state: &AppState, account: &str, bridge: &str) -> Result<(), Response> {
    if state.engine.is_none() || !matches!(bridge, "bridge-graph" | "bridge-gmail") {
        return Err(error(StatusCode::BAD_REQUEST, "provider_unsupported"));
    }
    crate::features::plugin_policy_admit(&state.store, bridge, Some(account))
        .await
        .map_err(|_| error(StatusCode::FORBIDDEN, "provider_disabled"))?;
    let row = state
        .store
        .get_plugin(bridge)
        .await
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "provider_unavailable"))?
        .filter(|row| row.enabled && row.approved_by.is_some())
        .ok_or_else(|| error(StatusCode::FORBIDDEN, "provider_not_approved"))?;
    crate::v7_mount::plan_load(&state.store, &row, Some(account))
        .await
        .map_err(|_| error(StatusCode::FORBIDDEN, "provider_not_approved"))?;
    let grants = state
        .store
        .plugin_grants_scoped(bridge, Some(account))
        .await
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "provider_unavailable"))?;
    if !["account-backend", "net"]
        .iter()
        .all(|cap| grants.iter().any(|grant| grant == cap))
    {
        return Err(error(StatusCode::FORBIDDEN, "provider_not_approved"));
    }
    // An administrator-selected EWS authority is not a user-selectable bridge.
    if crate::domain_provider::stored_selection(&state.store, account)
        .await
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "provider_unavailable"))?
        .is_some()
    {
        return Err(error(StatusCode::FORBIDDEN, "provider_locked"));
    }
    Ok(())
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/provider/oauth", get(status))
        .route("/api/provider/oauth/start", post(start))
        .route("/api/provider/oauth/complete", post(complete))
        .route("/api/provider/oauth/{flow}", delete(cancel))
        .layer(axum::middleware::map_response(
            |mut response: Response| async move {
                response.headers_mut().insert(
                    "cache-control",
                    axum::http::HeaderValue::from_static("no-store"),
                );
                response
            },
        ))
}

async fn status(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let (session, _) = match owner(&state, &headers).await {
        Ok(owner) => owner,
        Err(error) => return error,
    };
    let binding = match state.store.get_bridge_account(&session.account_id).await {
        Ok(binding) => binding,
        Err(_) => return error(StatusCode::SERVICE_UNAVAILABLE, "provider_unavailable"),
    };
    let config = binding
        .as_ref()
        .and_then(|binding| binding.oauth_ref.as_deref())
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
        .unwrap_or(json!({}));
    let authorized = state
        .store
        .get_bridge_oauth_token(&session.account_id)
        .await
        .ok()
        .flatten()
        .is_some_and(|token| token.refresh_token.is_some());
    let connected = authorized
        && binding.as_ref().is_some_and(|binding| {
            state.engine.as_ref().is_some_and(|engine| {
                engine.plugin_backend_id(&session.account_id).as_deref()
                    == Some(binding.bridge_id.as_str())
            })
        });
    let supported = if binding
        .as_ref()
        .is_some_and(|binding| binding.bridge_id == "bridge-gmail")
    {
        vec!["authorizationCode"]
    } else {
        vec!["authorizationCode", "deviceCode"]
    };
    reply(
        json!({"bridgeId":binding.as_ref().map(|binding|&binding.bridge_id),"configured":binding.is_some(),"connected":connected,"authorized":authorized,"clientId":config.get("client_id").or(config.get("clientId")),"tenant":config.get("tenant"),"flows":supported}),
    )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Start {
    bridge_id: String,
    client_id: String,
    #[serde(default)]
    tenant: String,
    flow: String,
}
async fn start(State(state): State<AppState>, request: Request) -> Response {
    let headers = request.headers().clone();
    let (session, authority) = match owner(&state, &headers).await {
        Ok(owner) => owner,
        Err(error) => return error,
    };
    let origin = state.external.base_url(&headers, request.extensions());
    let Ok(body) = axum::body::to_bytes(request.into_body(), 16384).await else {
        return error(StatusCode::BAD_REQUEST, "provider_invalid_request");
    };
    let input: Start = match serde_json::from_slice(&body) {
        Ok(input) => input,
        Err(_) => return error(StatusCode::BAD_REQUEST, "provider_invalid_request"),
    };
    if let Err(error) = admit(&state, &session.account_id, &input.bridge_id).await {
        return error;
    }
    if input.client_id.is_empty()
        || input.client_id.len() > 256
        || !input
            .client_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        || input.tenant.len() > 253
        || !input
            .tenant
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.'))
    {
        return error(StatusCode::BAD_REQUEST, "provider_invalid_configuration");
    }
    if state.store.get_account(&session.account_id).await.is_err() {
        return error(StatusCode::CONFLICT, "provider_account_unavailable");
    }
    let previous = match state.store.get_bridge_account(&session.account_id).await {
        Ok(value) => value,
        Err(_) => return error(StatusCode::SERVICE_UNAVAILABLE, "provider_unavailable"),
    };
    if previous
        .as_ref()
        .is_none_or(|binding| binding.bridge_id != input.bridge_id)
    {
        match state
            .store
            .count_account_messages(&session.account_id)
            .await
        {
            Ok(0) => {}
            Ok(_) => return error(StatusCode::CONFLICT, "provider_migration_required"),
            Err(_) => return error(StatusCode::SERVICE_UNAVAILABLE, "provider_unavailable"),
        }
    }
    let binding = BridgeAccountRow {
        account_id: session.account_id.clone(),
        bridge_id: input.bridge_id.clone(),
        oauth_ref: Some(json!({"client_id":input.client_id,"tenant":input.tenant}).to_string()),
        extra_json: "{}".into(),
    };
    let poster = ReqwestPoster::enrollment(state.store.clone(), binding.clone());
    let client = match OauthClient::enrollment(&poster, &binding) {
        Ok(client) => client,
        Err(_) => return error(StatusCode::BAD_REQUEST, "provider_invalid_configuration"),
    };
    let id = opaque();
    let now = Instant::now();
    let (mode, response, ttl) = match input.flow.as_str() {
        "authorizationCode" => {
            let Some(origin) = origin else {
                return error(StatusCode::BAD_REQUEST, "provider_public_url_required");
            };
            let Ok(url) = reqwest::Url::parse(&origin) else {
                return error(StatusCode::BAD_REQUEST, "provider_public_url_required");
            };
            if url.scheme() != "https"
                && !(url.scheme() == "http"
                    && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")))
            {
                return error(StatusCode::BAD_REQUEST, "provider_https_required");
            }
            // Personal Microsoft accounts require a registered reply URL with
            // no query component. Correlation lives in OAuth state + flowId.
            let redirect = format!("{origin}{}/", state.base_path);
            let verifier = opaque();
            let oauth_state = opaque();
            let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .encode(Sha256::digest(verifier.as_bytes()));
            let authorize = match client.authorization_url(&redirect, &oauth_state, &challenge) {
                Ok(url) => url,
                Err(_) => return error(StatusCode::BAD_REQUEST, "provider_invalid_configuration"),
            };
            (
                Mode::Code {
                    verifier,
                    state: oauth_state,
                    redirect: redirect.clone(),
                },
                json!({"flowId":id,"authorizationUrl":authorize,"redirectUri":redirect,"expiresIn":600}),
                600,
            )
        }
        "deviceCode" if input.bridge_id == "bridge-graph" => {
            let device = match client.begin_device_authorization().await {
                Ok(value) => value,
                Err(_) => return error(StatusCode::BAD_GATEWAY, "provider_authorization_failed"),
            };
            let Ok(verification) = reqwest::Url::parse(&device.verification_uri) else {
                return error(StatusCode::BAD_GATEWAY, "provider_authorization_failed");
            };
            if verification.scheme() != "https"
                || !matches!(
                    verification.host_str(),
                    Some("microsoft.com" | "www.microsoft.com" | "login.microsoftonline.com")
                )
            {
                return error(StatusCode::BAD_GATEWAY, "provider_authorization_failed");
            }
            let interval = device.interval.clamp(5, 60) as u64;
            let ttl = device.expires_in.clamp(1, 1800) as u64;
            (
                Mode::Device {
                    code: device.device_code,
                    interval,
                    next: now + Duration::from_secs(interval),
                },
                json!({"flowId":id,"userCode":device.user_code,"verificationUri":verification.as_str(),"interval":interval,"expiresIn":ttl}),
                ttl,
            )
        }
        _ => return error(StatusCode::BAD_REQUEST, "provider_flow_unsupported"),
    };
    let mut held = flows().lock().unwrap();
    held.retain(|_, flow| flow.expires > now || flow.committing);
    if held.len() >= 256 {
        return error(StatusCode::TOO_MANY_REQUESTS, "provider_busy");
    }
    if held.values().any(|flow| {
        flow.account == session.account_id && flow.session == session.id && flow.committing
    }) {
        return error(StatusCode::CONFLICT, "provider_committing");
    }
    held.retain(|_, flow| flow.account != session.account_id || flow.session != session.id);
    held.insert(
        id,
        Flow {
            busy: false,
            installed: false,
            committing: false,
            account: session.account_id,
            session: session.id,
            authority,
            binding,
            previous,
            expires: now + Duration::from_secs(ttl),
            mode,
        },
    );
    reply(response)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Complete {
    flow_id: String,
    #[serde(default)]
    code: String,
    #[serde(default)]
    state: String,
}
async fn complete(
    State(state): State<AppState>,
    Extension(plugins): Extension<PluginRegistry>,
    headers: HeaderMap,
    Json(input): Json<Complete>,
) -> Response {
    let (session, authority) = match owner(&state, &headers).await {
        Ok(owner) => owner,
        Err(error) => return error,
    };
    let mut flow = match claim_flow(&input.flow_id, &session.account_id, &session.id, &authority) {
        Ok(flow) => flow,
        Err(error) => return error,
    };
    let _claim = ClaimGuard(input.flow_id.clone());
    if let Err(error) = admit(&state, &flow.account, &flow.binding.bridge_id).await {
        return error;
    }
    if state.store.get_bridge_account(&flow.account).await.ok() != Some(flow.previous.clone()) {
        return error(StatusCode::CONFLICT, "provider_configuration_changed");
    }
    if matches!(flow.mode, Mode::Complete) {
        let mounted = state.engine.as_ref().is_some_and(|engine| {
            engine.plugin_backend_id(&flow.account).as_deref()
                == Some(flow.binding.bridge_id.as_str())
        });
        let authorized = state
            .store
            .get_bridge_oauth_token(&flow.account)
            .await
            .ok()
            .flatten()
            .is_some_and(|token| token.refresh_token.is_some());
        if mounted && authorized {
            return reply(json!({"status":"connected"}));
        }
    }
    if flow.installed {
        return finish_activation(&state, &plugins, &input.flow_id, flow).await;
    }
    let poster = ReqwestPoster::enrollment(state.store.clone(), flow.binding.clone());
    let client = match OauthClient::enrollment(&poster, &flow.binding) {
        Ok(value) => value,
        Err(_) => return error(StatusCode::BAD_REQUEST, "provider_invalid_configuration"),
    };
    let token = match &mut flow.mode {
        Mode::Complete => return error(StatusCode::BAD_REQUEST, "provider_flow_expired"),
        Mode::Code {
            verifier,
            state: expected,
            redirect,
        } => {
            if input.state != *expected || input.code.is_empty() || input.code.len() > 8192 {
                return error(StatusCode::BAD_REQUEST, "provider_state_invalid");
            }
            match client
                .exchange_authorization_code(&input.code, redirect, verifier)
                .await
            {
                Ok(token) => token,
                Err(_) => return error(StatusCode::BAD_GATEWAY, "provider_authorization_failed"),
            }
        }
        Mode::Device {
            code,
            interval,
            next,
        } => {
            let result = if *next > Instant::now() {
                Ok(DevicePoll::Pending)
            } else {
                *next = Instant::now() + Duration::from_secs(*interval);
                client.poll_device_token(code).await
            };
            match result {
                Ok(DevicePoll::Complete(token)) => token,
                Ok(status) => {
                    if status == DevicePoll::SlowDown {
                        *interval = (*interval + 5).min(60);
                        *next = Instant::now() + Duration::from_secs(*interval);
                    }
                    let response = json!({"status":if status==DevicePoll::SlowDown{"slowDown"}else{"pending"},"interval":*interval});
                    flow.busy = false;
                    let mut held = flows().lock().unwrap();
                    if !held.contains_key(&input.flow_id) {
                        return error(StatusCode::BAD_REQUEST, "provider_flow_expired");
                    }
                    held.insert(input.flow_id, flow);
                    return reply(response);
                }
                Err(_) => return error(StatusCode::BAD_GATEWAY, "provider_authorization_failed"),
            }
        }
    };
    if token
        .refresh_token
        .as_ref()
        .is_none_or(|token| token.is_empty())
    {
        return error(StatusCode::BAD_REQUEST, "provider_offline_consent_required");
    }
    if let Err(response) = verify_identity(&state, &session, &flow.binding, &token).await {
        return response;
    }
    if let Err(error) = admit(&state, &flow.account, &flow.binding.bridge_id).await {
        return error;
    }
    // A revoked or changed session during the provider round-trip cannot install credentials.
    if owner(&state, &headers).await.is_err() {
        return error(StatusCode::UNAUTHORIZED, "provider_session_expired");
    }
    if let Err(response) = begin_commit(&input.flow_id) {
        return response;
    }
    flow.committing = true;
    let credentials = BridgeOauthTokenRow {
        bridge_account_id: flow.account.clone(),
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        expires_at: (chrono::Utc::now()
            + chrono::Duration::seconds(token.expires_in.clamp(1, 86400)))
        .to_rfc3339(),
        scope: token.scope,
        updated_at: String::new(),
    };
    match state
        .store
        .enroll_bridge_oauth(
            &flow.binding,
            flow.previous.as_ref(),
            &credentials,
            Some(&session),
        )
        .await
    {
        Ok(true) => {}
        Ok(false) => return error(StatusCode::CONFLICT, "provider_configuration_changed"),
        Err(_) => return error(StatusCode::SERVICE_UNAVAILABLE, "provider_save_failed"),
    }
    flow.installed = true;
    flow.previous = Some(flow.binding.clone());
    flows()
        .lock()
        .unwrap()
        .insert(input.flow_id.clone(), flow.clone());
    finish_activation(&state, &plugins, &input.flow_id, flow).await
}

async fn finish_activation(
    state: &AppState,
    plugins: &PluginRegistry,
    id: &str,
    mut flow: Flow,
) -> Response {
    // Activation retries have the same cancellation boundary as the original
    // durable install: a canceled flow must never reappear after an await.
    if let Err(response) = begin_commit(id) {
        return response;
    }
    flow.committing = true;
    let engine = state.engine.as_ref().unwrap();
    if crate::v7_mount::activate_oauth(engine, plugins, &flow.account, &flow.binding.bridge_id)
        .await
        .is_err()
    {
        flow.busy = false;
        flows().lock().unwrap().insert(id.into(), flow);
        return error(
            StatusCode::SERVICE_UNAVAILABLE,
            "provider_activation_failed",
        );
    }
    // Keep a session-bound receipt for the original TTL. If the browser loses
    // the success response, retrying must not exchange the same code twice.
    flow.mode = Mode::Complete;
    flow.busy = false;
    flow.committing = false;
    flows().lock().unwrap().insert(id.into(), flow);
    reply(json!({"status":"connected"}))
}

async fn cancel(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let (session, authority) = match owner(&state, &headers).await {
        Ok(owner) => owner,
        Err(error) => return error,
    };
    match cancel_flow(&id, &session.account_id, &session.id, &authority) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => error,
    }
}
fn cancel_flow(id: &str, account: &str, session: &str, authority: &str) -> Result<(), Response> {
    let mut held = flows().lock().unwrap();
    if held.get(id).is_some_and(|flow| {
        flow.account == account && flow.session == session && flow.authority == authority
    }) {
        if held.get(id).is_some_and(|flow| flow.committing) {
            return Err(error(StatusCode::CONFLICT, "provider_committing"));
        }
        held.remove(id);
    }
    Ok(())
}

async fn verify_identity(
    state: &AppState,
    session: &Session,
    binding: &BridgeAccountRow,
    token: &TokenResponse,
) -> Result<(), Response> {
    let url = if binding.bridge_id == "bridge-graph" {
        "https://graph.microsoft.com/v1.0/me?$select=mail,userPrincipalName"
    } else {
        "https://gmail.googleapis.com/gmail/v1/users/me/profile"
    };
    let allowed = state
        .store
        .get_plugin(&binding.bridge_id)
        .await
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "provider_unavailable"))?
        .ok_or_else(|| error(StatusCode::FORBIDDEN, "provider_not_approved"))?;
    let allowed: Vec<String> = serde_json::from_str(&allowed.net_allowlist_json)
        .map_err(|_| error(StatusCode::FORBIDDEN, "provider_disabled"))?;
    let host = reqwest::Url::parse(url)
        .unwrap()
        .host_str()
        .unwrap()
        .to_owned();
    if !allowed
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(&host))
    {
        return Err(error(StatusCode::FORBIDDEN, "provider_disabled"));
    }
    let headers = vec![(
        "Authorization".into(),
        format!("Bearer {}", token.access_token),
    )];
    let metadata = mw_egress::ResponseMetadata::default();
    let plan = crate::image_proxy::load_plan(&state.store)
        .await
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "provider_unavailable"))?;
    let domain = session
        .username
        .rsplit_once('@')
        .map(|(_, domain)| domain)
        .unwrap_or("");
    let operation = mw_egress::plan::Request::plugin(
        mw_egress::TrafficClass::Plugins,
        mw_egress::PluginHttp {
            connection: None,
            additional_roots: &[],
            method: "GET",
            headers: &headers,
        },
        None,
    )
    .map_err(|_| error(StatusCode::BAD_GATEWAY, "provider_identity_failed"))?
    .for_domain(domain)
    .response_limit(65536)
    .capture_response(&metadata);
    let response =
        mw_egress::plan::send_with(&plan, reqwest::Url::parse(url).unwrap(), &operation).await;
    let body = response
        .outcome
        .map_err(|_| error(StatusCode::BAD_GATEWAY, "provider_identity_failed"))?;
    if !metadata
        .snapshot()
        .is_some_and(|head| (200..300).contains(&head.status))
    {
        return Err(error(StatusCode::BAD_GATEWAY, "provider_identity_failed"));
    }
    let profile: Value = serde_json::from_slice(&body)
        .map_err(|_| error(StatusCode::BAD_GATEWAY, "provider_identity_failed"))?;
    let expected = state
        .store
        .canonical_policy_address(&session.username)
        .await
        .map_err(|_| error(StatusCode::BAD_GATEWAY, "provider_identity_failed"))?;
    for property in ["mail", "userPrincipalName", "emailAddress"] {
        if let Some(address) = profile.get(property).and_then(Value::as_str)
            && state
                .store
                .canonical_policy_address(address)
                .await
                .ok()
                .is_some_and(|address| address.eq_ignore_ascii_case(&expected))
        {
            return Ok(());
        }
    }
    Err(error(StatusCode::FORBIDDEN, "provider_identity_mismatch"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pending() -> (String, Flow) {
        (
            opaque(),
            Flow {
                busy: false,
                installed: false,
                committing: false,
                account: "owner".into(),
                session: "session".into(),
                authority: "authority".into(),
                binding: BridgeAccountRow {
                    account_id: "owner".into(),
                    bridge_id: "bridge-graph".into(),
                    oauth_ref: None,
                    extra_json: "{}".into(),
                },
                previous: None,
                expires: Instant::now() + Duration::from_secs(60),
                mode: Mode::Code {
                    verifier: "never-return-pkce".into(),
                    state: "state".into(),
                    redirect: "https://mail.example/".into(),
                },
            },
        )
    }
    #[test]
    fn pending_authorizations_are_owner_session_realm_bound_and_single_flight() {
        let (id, flow) = pending();
        flows().lock().unwrap().insert(id.clone(), flow);
        for (owner, session, authority) in [
            ("other", "session", "authority"),
            ("owner", "other", "authority"),
            ("owner", "session", "other"),
        ] {
            assert_eq!(
                claim_flow(&id, owner, session, authority)
                    .err()
                    .unwrap()
                    .status(),
                StatusCode::BAD_REQUEST
            );
        }
        assert!(claim_flow(&id, "owner", "session", "authority").is_ok());
        assert_eq!(
            claim_flow(&id, "owner", "session", "authority")
                .err()
                .unwrap()
                .status(),
            StatusCode::CONFLICT
        );
        flows().lock().unwrap().remove(&id);
        assert_eq!(
            claim_flow(&id, "owner", "session", "authority")
                .err()
                .unwrap()
                .status(),
            StatusCode::BAD_REQUEST
        );
    }
    #[test]
    fn expired_flows_cannot_be_claimed_and_json_errors_are_not_cached() {
        let (id, mut flow) = pending();
        flow.expires = Instant::now() - Duration::from_secs(1);
        flows().lock().unwrap().insert(id.clone(), flow);
        let error = claim_flow(&id, "owner", "session", "authority")
            .err()
            .unwrap();
        assert_eq!(error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(error.headers()["cache-control"], "no-store");
        flows().lock().unwrap().remove(&id);
        assert_eq!(
            reply(json!({"flowId":"opaque"})).headers()["cache-control"],
            "no-store"
        );
    }

    #[test]
    fn cancellation_has_an_explicit_commit_boundary_and_errors_release_claims() {
        let (id, flow) = pending();
        flows().lock().unwrap().insert(id.clone(), flow);
        assert!(claim_flow(&id, "owner", "session", "authority").is_ok());
        {
            let _guard = ClaimGuard(id.clone());
        }
        assert!(!flows().lock().unwrap()[&id].busy);
        cancel_flow(&id, "other", "session", "authority").unwrap();
        assert!(flows().lock().unwrap().contains_key(&id));
        cancel_flow(&id, "owner", "session", "authority").unwrap();
        assert!(
            begin_commit(&id).is_err(),
            "a canceled grant cannot be installed"
        );
        let (id, flow) = pending();
        flows().lock().unwrap().insert(id.clone(), flow);
        begin_commit(&id).unwrap();
        assert_eq!(
            cancel_flow(&id, "owner", "session", "authority")
                .err()
                .unwrap()
                .status(),
            StatusCode::CONFLICT
        );
        assert!(
            flows().lock().unwrap().contains_key(&id),
            "cannot report canceled once saving has begun"
        );
        flows().lock().unwrap().remove(&id);
    }

    #[tokio::test]
    async fn mounted_enrollment_routes_require_primary_auth_csrf_and_bound_configuration() {
        use mw_store::{AccountKind, Credentials, NewAccount, PluginRow, ServerKey, Store};
        const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        let db = test_db::unique_dir("provider-oauth-http")
            .join("db.sqlite")
            .to_string_lossy()
            .into_owned();
        let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
            .await
            .unwrap();
        let credentials = Credentials {
            username: "owner@example.test".into(),
            password: "fixture-only".into(),
        };
        let account = store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "mail.example.test",
                    port: 993,
                    tls: "implicit",
                    username: &credentials.username,
                    sync_policy_json: "{}",
                },
                &credentials,
            )
            .await
            .unwrap();
        let sid = store
            .create_session(
                &account,
                &credentials.username,
                "engine",
                "engine",
                &credentials,
            )
            .await
            .unwrap();
        store
            .set_session_authentication(&sid, "password")
            .await
            .unwrap();
        store
            .bind_verified_primary(
                &sid,
                mw_store::identity_principal::VerifiedPrimary {
                    account_id: &account,
                    kind: "mailbox",
                    authority: "imap:mail.example.test:993",
                    subject: &account,
                },
                chrono::Utc::now().timestamp(),
            )
            .await
            .unwrap()
            .unwrap();
        let authority = store
            .session_storage_authority(&sid)
            .await
            .unwrap()
            .unwrap();
        let manifest = crate::v7_mount::first_party_manifest("bridge-graph").unwrap();
        store
            .put_plugin(&PluginRow {
                id: manifest.id.clone(),
                name: manifest.name.clone(),
                version: manifest.version.clone(),
                signature_hex: None,
                approved_by: Some("fixture-admin".into()),
                enabled: true,
                capabilities_json: serde_json::to_string(&manifest.capabilities).unwrap(),
                net_allowlist_json: serde_json::to_string(&manifest.net_allowlist).unwrap(),
                limits_json: serde_json::to_string(&manifest.limits).unwrap(),
                created_at: String::new(),
            })
            .await
            .unwrap();
        store
            .set_plugin_allow_unsigned("bridge-graph", true)
            .await
            .unwrap();
        store
            .replace_plugin_grants(
                "bridge-graph",
                "",
                &["net".into(), "account-backend".into()],
                "fixture-admin",
            )
            .await
            .unwrap();
        let app = crate::build_app(crate::AppConfig {
            db_path: db,
            server_key_hex: Some(KEY.into()),
            mode: crate::ServerMode::Engine,
            cookie_secure: false,
            web_dir: None,
            hardening: crate::HardeningConfig {
                csrf_strict: true,
                ..Default::default()
            },
            security: crate::SecurityConfig::default(),
        })
        .await
        .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await
            .unwrap();
        });
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        let csrf = store.session_csrf_token(&sid);
        let cookie = format!("mw_session={sid}; mw_csrf={csrf}");
        let authorized = |method: reqwest::Method, path: &str| {
            client
                .request(method, format!("{base}{path}"))
                .header("cookie", &cookie)
                .header("origin", &base)
                .header("x-csrf-token", &csrf)
                .header("x-mailwoman-account", &account)
                .header("x-mailwoman-storage-authority", &authority)
        };
        let endpoint = "/api/provider/oauth";
        let unauthenticated = client
            .get(format!("{base}{endpoint}"))
            .send()
            .await
            .unwrap();
        assert_eq!(unauthenticated.status(), 401);
        assert_eq!(unauthenticated.headers()["cache-control"], "no-store");
        let status = authorized(reqwest::Method::GET, endpoint)
            .send()
            .await
            .unwrap();
        assert_eq!(status.status(), 200);
        assert_eq!(status.headers()["cache-control"], "no-store");
        let status: Value = status.json().await.unwrap();
        assert_eq!(status["connected"], false);
        let body = json!({"bridgeId":"bridge-graph","clientId":"fixture-app","tenant":"common","flow":"authorizationCode"});
        let rejected = client
            .post(format!("{base}{endpoint}/start"))
            .header("cookie", &cookie)
            .header("origin", &base)
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(
            rejected.status(),
            403,
            "strictCSRF staysenabledforOAuthwrites"
        );
        let child = authorized(reqwest::Method::POST, &format!("{endpoint}/start"))
            .header("x-mailwoman-link", "child")
            .json(&body)
            .send()
            .await
            .unwrap();
        assert!(!child.status().is_success());
        let started = authorized(reqwest::Method::POST, &format!("{endpoint}/start"))
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(started.status(), 200);
        assert_eq!(started.headers()["cache-control"], "no-store");
        let started: Value = started.json().await.unwrap();
        for forbidden in [
            "deviceCode",
            "codeVerifier",
            "accessToken",
            "refreshToken",
            "clientSecret",
        ] {
            assert!(started.get(forbidden).is_none());
        }
        let url = reqwest::Url::parse(started["authorizationUrl"].as_str().unwrap()).unwrap();
        assert_eq!(url.host_str(), Some("login.microsoftonline.com"));
        let redirect = url
            .query_pairs()
            .find(|(key, _)| key == "redirect_uri")
            .unwrap()
            .1
            .into_owned();
        assert!(
            reqwest::Url::parse(&redirect).unwrap().query().is_none(),
            "Microsoft personal accounts require a reply URL without query parameters"
        );
        assert!(redirect.ends_with('/'));
        assert!(
            url.query_pairs()
                .any(|(key, value)| key == "code_challenge_method" && value == "S256")
        );
        let flow = started["flowId"].as_str().unwrap();
        let invalid = authorized(reqwest::Method::POST, &format!("{endpoint}/complete"))
            .json(&json!({"flowId":flow,"code":"never-exchange","state":"wrong"}))
            .send()
            .await
            .unwrap();
        assert_eq!(invalid.status(), 400);
        assert_eq!(
            invalid.json::<Value>().await.unwrap()["code"],
            "provider_state_invalid"
        );
        assert!(
            store
                .get_bridge_oauth_token(&account)
                .await
                .unwrap()
                .is_none()
        );
        store
            .put_bridge_account(&BridgeAccountRow {
                account_id: account.clone(),
                bridge_id: "bridge-gmail".into(),
                oauth_ref: Some(r#"{"client_id":"admin-changed"}"#.into()),
                extra_json: "{}".into(),
            })
            .await
            .unwrap();
        let stale = authorized(reqwest::Method::POST, &format!("{endpoint}/complete"))
            .json(&json!({"flowId":flow,"code":"never-exchange","state":"wrong"}))
            .send()
            .await
            .unwrap();
        assert_eq!(stale.status(), 409);
        assert_eq!(
            stale.json::<Value>().await.unwrap()["code"],
            "provider_configuration_changed"
        );
        assert!(
            store
                .get_bridge_oauth_token(&account)
                .await
                .unwrap()
                .is_none()
        );
        store.delete_session(&sid).await.unwrap();
        assert_eq!(
            authorized(reqwest::Method::GET, endpoint)
                .send()
                .await
                .unwrap()
                .status(),
            401,
            "revoked sessions cannot replay enrollment receipts"
        );
        flows().lock().unwrap().remove(flow);
        server.abort();
    }
}
