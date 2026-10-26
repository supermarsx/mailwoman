use super::*;
use crate::{
    admin_ctx::{AdminCtx, AdminJson, with_etag},
    admin_router::AdminRegistrar,
    capabilities::Capability,
    step_up::{self, StepUpAction},
};
use axum::{
    extract::{Path, State},
    http::Method,
};
use mw_store::oidc_provider::OidcClientRow;
use std::sync::{Arc, OnceLock};

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/oidc",
        Capability::IntegrationsRead,
        get_config,
    );
    r.admin_endpoint(
        Method::PUT,
        "/oidc",
        Capability::OauthClientsWrite,
        put_config,
    );
    r.admin_endpoint(
        Method::POST,
        "/oidc/keys/rotate",
        Capability::OauthClientsWrite,
        rotate_key,
    );
    r.admin_endpoint(
        Method::GET,
        "/oidc/clients",
        Capability::IntegrationsRead,
        list_clients,
    );
    r.admin_endpoint(
        Method::POST,
        "/oidc/clients",
        Capability::OauthClientsWrite,
        create_client,
    );
    r.admin_endpoint(
        Method::GET,
        "/oidc/clients/{id}",
        Capability::IntegrationsRead,
        get_client,
    );
    r.admin_endpoint(
        Method::PUT,
        "/oidc/clients/{id}",
        Capability::OauthClientsWrite,
        put_client,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/oidc/clients/{id}",
        Capability::OauthClientsWrite,
        delete_client,
    );
}
async fn manage(state: &AppState, ctx: &AdminCtx) -> Result<(), AdminApiError> {
    ctx.require_unscoped(Capability::OauthClientsWrite)?;
    step_up::require_admin(state, ctx, StepUpAction::IntegrationManage).await
}
async fn generate_key() -> Result<(String, String, Value), AdminApiError> {
    static LIMIT: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
    let permit = LIMIT
        .get_or_init(|| Arc::new(tokio::sync::Semaphore::new(1)))
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            AdminApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "oidcKeyBusy",
                "Signing key generation is already running",
            )
        })?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let kid = opaque("key_");
        mw_sso::issuer::generate_rs256_key(&kid).map(|(pem, jwk)| (kid, pem, jwk))
    })
    .await
    .map_err(|_| {
        AdminApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "oidcKeyUnavailable",
            "Signing key generation failed",
        )
    })?
    .map_err(|_| {
        AdminApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "oidcKeyUnavailable",
            "Signing key generation failed",
        )
    })
}
async fn get_config(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsRead)?;
    let config = state.store.oidc_provider_config().await?;
    let mut value = match &config {
        Some(c) => {
            json!({"enabled":c.enabled,"issuer":c.issuer,"revision":c.revision,"updatedAt":c.updated_at,"discoveryUrl":format!("{}/.well-known/openid-configuration",c.issuer)})
        }
        None => json!({"enabled":false,"issuer":"","revision":"unconfigured"}),
    };
    if let Some(config) = &config {
        value["activeKeyId"] = json!(
            state
                .store
                .oidc_signing_keys(&config.issuer, now())
                .await?
                .into_iter()
                .find(|key| key.active)
                .map(|key| key.kid)
        );
    }
    Ok(no_store(with_etag(
        Json(value),
        &format!(
            "\"{}\"",
            config
                .as_ref()
                .map(|c| c.revision.as_str())
                .unwrap_or("unconfigured")
        ),
    )))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProviderWrite {
    issuer: String,
    enabled: bool,
}
async fn put_config(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(body): AdminJson<ProviderWrite>,
) -> Result<Response, AdminApiError> {
    manage(&state, &ctx).await?;
    let issuer = issuer_url(&state, &body.issuer)?;
    let old = state.store.oidc_provider_config().await?;
    let expected = revision(&headers, old.as_ref().map(|c| c.revision.as_str()))?;
    if body.enabled
        && !state
            .store
            .oidc_signing_keys(&issuer, now())
            .await?
            .iter()
            .any(|key| key.active)
    {
        let (kid, pem, jwk) = generate_key().await?;
        state
            .store
            .rotate_oidc_signing_key(&kid, &issuer, pem.as_bytes(), &jwk, now())
            .await?;
    }
    let revision = state
        .store
        .put_oidc_provider_config(&issuer, body.enabled, expected.as_deref(), now())
        .await?
        .ok_or_else(|| AdminApiError::stale(None, None))?;
    Ok(no_store(with_etag(
        Json(json!({"enabled":body.enabled,"issuer":issuer,"revision":revision})),
        &format!("\"{revision}\""),
    )))
}
async fn rotate_key(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
) -> Result<Response, AdminApiError> {
    manage(&state, &ctx).await?;
    let config = state
        .store
        .oidc_provider_config()
        .await?
        .ok_or_else(|| bad("Configure the issuer first"))?;
    let keys = state.store.oidc_signing_keys(&config.issuer, now()).await?;
    let old = keys
        .iter()
        .find(|key| key.active)
        .map(|key| key.kid.as_str());
    revision(&headers, old)?;
    let (kid, pem, jwk) = generate_key().await?;
    // The store compares this exact active key under the identity write lock.
    if !state
        .store
        .rotate_oidc_signing_key_if_current(mw_store::oidc_provider::OidcKeyRotation {
            kid: &kid,
            issuer: &config.issuer,
            private_pem: pem.as_bytes(),
            public_jwk: &jwk,
            expected_kid: old,
            now: now(),
        })
        .await?
    {
        return Err(AdminApiError::stale(None, None));
    }
    Ok(no_store(with_etag(
        Json(json!({"kid":kid,"retainedVerificationSeconds":900})),
        &format!("\"{kid}\""),
    )))
}
async fn summary(state: &AppState, row: &OidcClientRow) -> Result<Value, AdminApiError> {
    let base = state
        .store
        .get_oauth_client(&row.client_id)
        .await?
        .ok_or_else(|| bad("Approved client registry entry is missing"))?;
    Ok(
        json!({"clientId":row.client_id,"name":base.name,"config":row.config,"enabled":row.enabled,"revision":row.revision,"updatedAt":row.updated_at,"secretConfigured":row.secret_hash.is_some()}),
    )
}
async fn list_clients(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsRead)?;
    let mut clients = Vec::new();
    for row in state.store.oidc_clients().await? {
        clients.push(summary(&state, &row).await?);
    }
    Ok(no_store(Json(json!({"clients":clients}))))
}
async fn get_client(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsRead)?;
    let row = state.store.oidc_client(&id).await?.ok_or_else(|| {
        AdminApiError::new(
            StatusCode::NOT_FOUND,
            "notFound",
            "Identity client not found",
        )
    })?;
    Ok(no_store(with_etag(
        Json(summary(&state, &row).await?),
        &format!("\"{}\"", row.revision),
    )))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ClientWrite {
    name: Option<String>,
    config: ClientConfig,
    enabled: bool,
    #[serde(default)]
    secret_action: SecretAction,
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SecretAction {
    #[default]
    Keep,
    Rotate,
    Clear,
}
fn secret(
    config: &ClientConfig,
    action: &SecretAction,
    old: Option<&OidcClientRow>,
) -> Result<(Option<String>, Option<String>), AdminApiError> {
    if config.auth_method == ClientAuth::None {
        return Ok((None, None));
    }
    match action {
        SecretAction::Clear => Err(bad("Confidential clients require a generated secret")),
        SecretAction::Keep if old.is_some() => {
            let hash = old.and_then(|r| r.secret_hash.clone()).ok_or_else(|| {
                bad("Generate a secret when enabling confidential authentication")
            })?;
            Ok((Some(hash), None))
        }
        _ => {
            let raw = opaque("oidc_secret_");
            Ok((Some(hash(&raw)), Some(raw)))
        }
    }
}
async fn create_client(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(body): AdminJson<ClientWrite>,
) -> Result<Response, AdminApiError> {
    manage(&state, &ctx).await?;
    validate_client(&body.config)?;
    let name = body
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty() && name.len() <= 120 && !name.chars().any(char::is_control))
        .ok_or_else(|| bad("Client name is required (up to120 characters)"))?;
    let (secret_hash, raw) = secret(&body.config, &body.secret_action, None)?;
    let row = OidcClientRow {
        client_id: opaque("oidc_client_"),
        config: serde_json::to_value(body.config)
            .map_err(|_| bad("Invalid client configuration"))?,
        enabled: body.enabled,
        revision: String::new(),
        updated_at: now(),
        secret_hash,
    };
    let revision = state
        .store
        .create_oidc_client(&row, name, &ctx.username)
        .await?
        .ok_or_else(|| bad("Identity client capacity reached"))?;
    Ok(no_store((
        StatusCode::CREATED,
        Json(json!({"clientId":row.client_id,"revision":revision,"clientSecret":raw})),
    )))
}
async fn put_client(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    headers: HeaderMap,
    AdminJson(body): AdminJson<ClientWrite>,
) -> Result<Response, AdminApiError> {
    manage(&state, &ctx).await?;
    validate_client(&body.config)?;
    if body.name.as_deref().is_some_and(|name| {
        name.trim().is_empty() || name.len() > 120 || name.chars().any(char::is_control)
    }) {
        return Err(bad("Client name must contain one to120 characters"));
    }
    let old = state
        .store
        .oidc_client(&id)
        .await?
        .ok_or_else(|| bad("Identity client not found"))?;
    let expected = revision(&headers, Some(&old.revision))?;
    let (secret_hash, raw) = secret(&body.config, &body.secret_action, Some(&old))?;
    let row = OidcClientRow {
        client_id: id,
        config: serde_json::to_value(body.config)
            .map_err(|_| bad("Invalid client configuration"))?,
        enabled: body.enabled,
        revision: String::new(),
        updated_at: now(),
        secret_hash,
    };
    let revision = state
        .store
        .put_oidc_client_named(&row, expected.as_deref(), body.name.as_deref())
        .await?
        .ok_or_else(|| AdminApiError::stale(None, None))?;
    Ok(no_store(with_etag(
        Json(json!({"clientId":row.client_id,"revision":revision,"clientSecret":raw})),
        &format!("\"{revision}\""),
    )))
}
async fn delete_client(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AdminApiError> {
    manage(&state, &ctx).await?;
    let old = state
        .store
        .oidc_client(&id)
        .await?
        .ok_or_else(|| bad("Identity client not found"))?;
    revision(&headers, Some(&old.revision))?;
    if !state.store.delete_oidc_client(&id, &old.revision).await? {
        return Err(AdminApiError::stale(None, None));
    }
    Ok(no_store(StatusCode::NO_CONTENT))
}
