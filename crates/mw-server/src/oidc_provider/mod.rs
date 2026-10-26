//! Mailwoman as an OIDC issuer. Separate from external SSO and mail OAuth.
mod admin;
mod protocol;
use crate::{AppState, admin_ctx::AdminApiError};
pub(crate) use admin::register;
use axum::{
    Json, Router,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use base64::Engine as _;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
pub(crate) fn routes() -> Router<AppState> {
    protocol::routes()
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum ClientAuth {
    None,
    ClientSecretBasic,
    ClientSecretPost,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ClientConfig {
    pub redirect_uris: Vec<String>,
    pub post_logout_redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
    pub auth_method: ClientAuth,
}
fn bad(message: &str) -> AdminApiError {
    AdminApiError::new(
        StatusCode::UNPROCESSABLE_ENTITY,
        "invalidOidcConfiguration",
        message,
    )
}
fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
fn opaque(prefix: &str) -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    format!(
        "{prefix}{}",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    )
}
fn hash(raw: &str) -> String {
    Sha256::digest(raw.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn no_store(value: impl IntoResponse) -> Response {
    let mut response = value.into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "private, no-store".parse().unwrap());
    response
        .headers_mut()
        .insert(header::PRAGMA, "no-cache".parse().unwrap());
    response
}
fn token_error(code: &'static str, status: StatusCode) -> Response {
    let mut response = no_store((status, Json(json!({"error":code}))));
    if status == StatusCode::UNAUTHORIZED {
        let challenge = if code == "invalid_token" {
            "Bearer error=\"invalid_token\""
        } else {
            "Basic realm=\"oidc\""
        };
        response
            .headers_mut()
            .insert(header::WWW_AUTHENTICATE, challenge.parse().unwrap());
    }
    response
}
fn revision(headers: &HeaderMap, current: Option<&str>) -> Result<Option<String>, AdminApiError> {
    let expected = current.unwrap_or("unconfigured");
    let raw = headers
        .get(header::IF_MATCH)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            AdminApiError::new(
                StatusCode::PRECONDITION_REQUIRED,
                "preconditionRequired",
                "Read the current configuration before saving",
            )
        })?;
    if raw != format!("\"{expected}\"") {
        return Err(AdminApiError::stale(None, None));
    }
    Ok(current.map(str::to_owned))
}
fn issuer_url(state: &AppState, raw: &str) -> Result<String, AdminApiError> {
    let url = reqwest::Url::parse(raw).map_err(|_| bad("Issuer must be an HTTPS URL"))?;
    let expected = format!("{}/oidc", state.base_path.trim_end_matches('/'));
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != expected
    {
        return Err(bad(
            "Issuer must use HTTPS and this deployment's /oidc path, without credentials, query or fragment",
        ));
    }
    Ok(url.to_string())
}
fn redirect_valid(raw: &str) -> bool {
    if raw.len() > 2048 {
        return false;
    }
    let Ok(url) = reqwest::Url::parse(raw) else {
        return false;
    };
    let https = url.scheme() == "https";
    let loopback = url.scheme() == "http"
        && url
            .host_str()
            .is_some_and(|host| host == "127.0.0.1" || host == "[::1]" || host == "::1");
    (https || loopback)
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none()
        && !url
            .query_pairs()
            .any(|(key, _)| matches!(key.as_ref(), "code" | "state" | "error"))
}
fn validate_client(config: &ClientConfig) -> Result<(), AdminApiError> {
    if config.redirect_uris.is_empty()
        || config.redirect_uris.len() > 20
        || config.post_logout_redirect_uris.len() > 20
    {
        return Err(bad("Register one to twenty exact redirect URIs"));
    }
    for values in [&config.redirect_uris, &config.post_logout_redirect_uris] {
        let mut seen = std::collections::BTreeSet::new();
        for value in values {
            if !redirect_valid(value) || !seen.insert(value) {
                return Err(bad(
                    "Redirects must be unique HTTPS URLs (or numeric loopback HTTP), without fragments or response parameters",
                ));
            }
        }
    }
    let mut scopes = std::collections::BTreeSet::new();
    if !config.scopes.iter().any(|s| s == "openid")
        || config.scopes.iter().any(|s| {
            !matches!(
                s.as_str(),
                "openid" | "profile" | "email" | "offline_access"
            ) || !scopes.insert(s)
        })
    {
        return Err(bad(
            "Allowed identity scopes are openid, profile, email and offline_access; openid is required",
        ));
    }
    Ok(())
}
async fn provider(
    state: &AppState,
) -> Result<mw_store::oidc_provider::OidcProviderConfig, Response> {
    state
        .store
        .oidc_provider_config()
        .await
        .map_err(|_| token_error("temporarily_unavailable", StatusCode::SERVICE_UNAVAILABLE))?
        .filter(|c| c.enabled)
        .ok_or_else(|| token_error("issuer_disabled", StatusCode::NOT_FOUND))
}
async fn browser_session(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<mw_store::Session, Response> {
    if headers.contains_key(header::AUTHORIZATION) {
        return Err(token_error("invalid_request", StatusCode::BAD_REQUEST));
    }
    let session = crate::authed(state, headers).await?;
    if state
        .store
        .get_native_session(&crate::push_relay::hash_token(&session.id))
        .await
        .map_err(|_| token_error("temporarily_unavailable", StatusCode::SERVICE_UNAVAILABLE))?
        .is_some()
    {
        return Err(token_error("invalid_request", StatusCode::BAD_REQUEST));
    }
    Ok(session)
}
fn redirect_response(uri: &str, values: &[(&str, &str)]) -> Result<String, Response> {
    let mut url = reqwest::Url::parse(uri)
        .map_err(|_| token_error("invalid_request", StatusCode::BAD_REQUEST))?;
    url.query_pairs_mut().extend_pairs(values.iter().copied());
    Ok(url.into())
}
/// Deep consent pages must resolve entry assets at the deployment root, without
/// receiving the administrator console marker.
pub(crate) fn shell(state: &AppState, headers: &HeaderMap) -> Option<Response> {
    let bytes = crate::adapt_asset_to_base(
        "index.html",
        crate::read_asset(state, "index.html")?,
        &state.base_path,
    );
    let html = String::from_utf8(bytes).ok()?;
    let html = html
        .replace("src=\"./", &format!("src=\"{}/", state.base_path))
        .replace("href=\"./", &format!("href=\"{}/", state.base_path));
    Some(crate::asset_response(
        "index.html",
        "text/html",
        html.into_bytes(),
        headers,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn client_redirects_are_exact_and_identity_scopes_only() {
        let mut c = ClientConfig {
            redirect_uris: vec!["https://rp.example/callback".into()],
            post_logout_redirect_uris: vec![],
            scopes: vec!["openid".into()],
            auth_method: ClientAuth::None,
        };
        assert!(validate_client(&c).is_ok());
        for uri in [
            "http://rp.example/callback",
            "https://user:pass@rp.example/callback",
            "https://rp.example/#fragment",
            "https://rp.example/?code=forged",
        ] {
            c.redirect_uris = vec![uri.into()];
            assert!(validate_client(&c).is_err());
        }
        c.redirect_uris = vec!["http://127.0.0.1:4000/callback".into()];
        assert!(validate_client(&c).is_ok());
        c.scopes.push("mail.read".into());
        assert!(validate_client(&c).is_err());
    }
}
