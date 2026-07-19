//! Bridge OAuth token acquisition (26.16 B1). The host-side OAuth **client** the
//! `oauth-token` import (`host_state.rs:53` `OAuthTokenProvider`) is backed by: it
//! mints and refreshes short-lived access tokens for a bridge account against the
//! provider's token endpoint (`login.microsoftonline.com` for Microsoft Graph,
//! `oauth2.googleapis.com` for Gmail), caching the minted pair in the 0018
//! `bridge_oauth_tokens` table (sealed at rest) so a refresh isn't re-run on every
//! guest call.
//!
//! The jailed guest never contacts the token endpoint and never sees a long-lived
//! secret: it calls `oauth-token` with its (empty) bound-account handle, the host
//! resolves that to the local account id, and this module answers with a valid access
//! token — refreshing transparently when the cached one is near expiry (§2.1/§6.5).
//!
//! Three flows are implemented over one host `reqwest`(rustls) transport:
//!   * **device-code** (`begin_device_authorization` + `poll_device_token`) — the
//!     headless enrolment path (no browser redirect on the server);
//!   * **authorization-code** (`exchange_authorization_code`) — the redirect path;
//!   * **refresh** (`refresh`) — the live path [`acquire_access_token`] drives on a
//!     near-expiry cached token.
//!
//! Enrolment (device-/auth-code, which mint the FIRST refresh token) is admin-driven
//! and out of band of a guest's `oauth-token` call; [`acquire_access_token`] only ever
//! returns a cached-or-refreshed token and never blocks on user interaction. The
//! transport is abstracted behind [`FormPoster`] so the flows unit-test against a mock
//! IdP with no network (live-tenant proof is LPB — see the e2e lane).
//!
//! Config (provider, client id, tenant, scopes) is read from the account's
//! `bridge_accounts.oauth_ref` (0008), which holds only NON-secret values; the client
//! secret (for a confidential app registration) comes from the deployment env, and the
//! refresh token — the one long-lived secret — lives only sealed in 0018.

// Enrolment flows (device-/auth-code) and their response types are provided host-side
// for the admin enrolment endpoint/CLI that mints the first refresh token; only
// `refresh` has an in-crate caller today ([`acquire_access_token`]). Mirrors the
// `oauth.rs` `dcr` module's "provided but not yet mounted" allowance.
#![allow(dead_code)]

use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use mw_store::{BridgeOauthTokenRow, Store};

/// Refresh proactively when the cached access token is within this window of expiry, so
/// a token handed to a guest always has usable headroom for its request.
const EXPIRY_SKEW: Duration = Duration::from_secs(120);

/// The OAuth provider a bridge authenticates against. Determines the default endpoints
/// and scope set; the bridge id maps to one when `oauth_ref` doesn't state it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// Microsoft identity platform (Entra ID) — Graph bridge.
    Microsoft,
    /// Google identity platform — Gmail bridge.
    Google,
}

impl Provider {
    /// Parse a provider from an explicit `oauth_ref` value (case-insensitive), accepting
    /// the common aliases.
    fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "microsoft" | "ms" | "entra" | "azure" | "graph" => Some(Self::Microsoft),
            "google" | "gmail" | "workspace" => Some(Self::Google),
            _ => None,
        }
    }

    /// The default provider for a first-party bridge id (used when `oauth_ref` omits it).
    fn from_bridge_id(bridge_id: &str) -> Option<Self> {
        match bridge_id {
            "bridge-graph" => Some(Self::Microsoft),
            "bridge-gmail" => Some(Self::Google),
            _ => None,
        }
    }

    /// The default delegated scopes when `oauth_ref` lists none. `offline_access`
    /// (Microsoft) / `access_type=offline` handling (Google) is what yields a refresh
    /// token; the caller adds Google's `access_type` separately.
    fn default_scopes(self) -> Vec<String> {
        match self {
            Self::Microsoft => vec![
                "https://graph.microsoft.com/.default".to_string(),
                "offline_access".to_string(),
            ],
            Self::Google => vec!["https://www.googleapis.com/auth/gmail.modify".to_string()],
        }
    }
}

/// The resolved provider endpoints for one bridge account.
#[derive(Debug, Clone)]
struct Endpoints {
    token: String,
    device_authorization: String,
    authorize: String,
}

impl Endpoints {
    fn resolve(provider: Provider, tenant: &str, cfg: &BridgeOauthConfig) -> Self {
        let (token, device, authorize) = match provider {
            Provider::Microsoft => {
                let t = if tenant.is_empty() { "common" } else { tenant };
                (
                    format!("https://login.microsoftonline.com/{t}/oauth2/v2.0/token"),
                    format!("https://login.microsoftonline.com/{t}/oauth2/v2.0/devicecode"),
                    format!("https://login.microsoftonline.com/{t}/oauth2/v2.0/authorize"),
                )
            }
            Provider::Google => (
                "https://oauth2.googleapis.com/token".to_string(),
                "https://oauth2.googleapis.com/device/code".to_string(),
                "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            ),
        };
        Self {
            // Endpoint overrides let a sovereign cloud (or a unit-test mock IdP) redirect
            // the flows without a code change; absent ⇒ the public provider endpoint.
            token: cfg.token_endpoint.clone().unwrap_or(token),
            device_authorization: cfg.device_authorization_endpoint.clone().unwrap_or(device),
            authorize: cfg.authorize_endpoint.clone().unwrap_or(authorize),
        }
    }
}

/// The NON-secret OAuth configuration carried in `bridge_accounts.oauth_ref` (0008),
/// parsed as JSON. It references the deployment's own app registration (client id +
/// tenant) — never a secret; the client secret comes from the env and the refresh token
/// lives sealed in 0018.
#[derive(Debug, Clone, Deserialize)]
struct BridgeOauthConfig {
    /// `"microsoft"` / `"google"` (+ aliases). Omitted ⇒ inferred from the bridge id.
    #[serde(default)]
    provider: Option<String>,
    /// The application (client) id of the admin's own app registration.
    client_id: String,
    /// Microsoft tenant id/domain (or `common`/`organizations`/`consumers`). Ignored
    /// for Google.
    #[serde(default)]
    tenant: Option<String>,
    /// Requested delegated scopes. Empty ⇒ the provider default set.
    #[serde(default)]
    scopes: Vec<String>,
    /// Endpoint overrides (sovereign clouds / tests). Absent ⇒ the public endpoints.
    #[serde(default)]
    token_endpoint: Option<String>,
    #[serde(default)]
    device_authorization_endpoint: Option<String>,
    #[serde(default)]
    authorize_endpoint: Option<String>,
}

/// A token-endpoint error, distinguishing the transient device-flow polling states
/// (`authorization_pending` / `slow_down`) from a hard failure so the poller can
/// keep going.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OauthError {
    /// The HTTP request itself failed (connect/TLS/timeout).
    Transport(String),
    /// The provider returned an OAuth error object (`{error, error_description}`).
    Provider { error: String, description: String },
    /// A response that could not be parsed as an OAuth token/error body.
    Malformed(String),
    /// Misconfiguration (missing/invalid `oauth_ref`, unknown provider).
    Config(String),
}

impl std::fmt::Display for OauthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(e) => write!(f, "oauth transport error: {e}"),
            Self::Provider { error, description } => {
                if description.is_empty() {
                    write!(f, "oauth provider error: {error}")
                } else {
                    write!(f, "oauth provider error: {error} ({description})")
                }
            }
            Self::Malformed(e) => write!(f, "malformed oauth response: {e}"),
            Self::Config(e) => write!(f, "bridge oauth config error: {e}"),
        }
    }
}

impl std::error::Error for OauthError {}

/// A form POST transport (the token/device endpoints are all
/// `application/x-www-form-urlencoded` POSTs). Abstracted so the flows unit-test against
/// a mock IdP; production is [`ReqwestPoster`] over the host `reqwest`(rustls) client.
#[async_trait]
pub trait FormPoster: Send + Sync {
    async fn post_form(
        &self,
        url: &str,
        form: &[(&str, &str)],
    ) -> std::result::Result<FormResponse, OauthError>;
}

/// A raw form-POST response (status + body). Kept transport-agnostic so a mock can
/// stand in for the provider verbatim.
#[derive(Debug, Clone)]
pub struct FormResponse {
    pub status: u16,
    pub body: String,
}

/// The production [`FormPoster`]: the in-tree `reqwest`(rustls) client (no native-tls /
/// openssl). Reused across every bridge account.
pub struct ReqwestPoster {
    client: reqwest::Client,
}

impl ReqwestPoster {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl FormPoster for ReqwestPoster {
    async fn post_form(
        &self,
        url: &str,
        form: &[(&str, &str)],
    ) -> std::result::Result<FormResponse, OauthError> {
        let resp = self
            .client
            .post(url)
            .form(form)
            .send()
            .await
            .map_err(|e| OauthError::Transport(e.to_string()))?;
        let status = resp.status().as_u16();
        let body = resp
            .text()
            .await
            .map_err(|e| OauthError::Transport(e.to_string()))?;
        Ok(FormResponse { status, body })
    }
}

/// A minted/refreshed token set from a provider token response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenResponse {
    pub access_token: String,
    /// Absent when the provider did not issue/rotate one (Google often omits it on a
    /// refresh — the caller retains the prior refresh token).
    pub refresh_token: Option<String>,
    /// Lifetime of the access token, in seconds.
    pub expires_in: i64,
    /// The granted scope string (space-delimited), when returned.
    pub scope: String,
}

/// The wire shape of a provider token response (`access_token` grant success).
#[derive(Debug, Deserialize)]
struct TokenWire {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    #[serde(default)]
    scope: String,
    // Error object (present on a non-2xx or a device-flow pending body).
    error: Option<String>,
    #[serde(default)]
    error_description: String,
}

/// A device-authorization response (RFC 8628 §3.2).
#[derive(Debug, Clone)]
pub struct DeviceAuthorization {
    pub device_code: String,
    pub user_code: String,
    /// The URL the user visits to enter the code (`verification_uri`, or Google's
    /// `verification_url`).
    pub verification_uri: String,
    pub expires_in: i64,
    /// Minimum seconds between polls (defaults to 5 when the provider omits it).
    pub interval: i64,
}

#[derive(Debug, Deserialize)]
struct DeviceAuthWire {
    device_code: Option<String>,
    user_code: Option<String>,
    verification_uri: Option<String>,
    // Google spells it `verification_url`.
    verification_url: Option<String>,
    expires_in: Option<i64>,
    interval: Option<i64>,
    error: Option<String>,
    #[serde(default)]
    error_description: String,
}

/// The outcome of one device-code poll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevicePoll {
    /// The user hasn't completed authorization yet — keep polling at `interval`.
    Pending,
    /// The provider asked us to back off — increase the poll interval.
    SlowDown,
    /// Authorization completed; the token set was issued.
    Complete(TokenResponse),
}

/// The bridge OAuth client for ONE account: its resolved provider, credentials, and
/// endpoints, over a shared [`FormPoster`].
pub struct OauthClient<'a> {
    poster: &'a dyn FormPoster,
    provider: Provider,
    client_id: String,
    client_secret: Option<String>,
    scopes: Vec<String>,
    endpoints: Endpoints,
}

impl<'a> OauthClient<'a> {
    /// The space-delimited requested-scope string.
    fn scope_str(&self) -> String {
        self.scopes.join(" ")
    }

    /// Push the optional confidential-client secret onto a form (public PKCE clients
    /// omit it).
    fn with_secret<'f>(&'f self, mut form: Vec<(&'f str, &'f str)>) -> Vec<(&'f str, &'f str)> {
        if let Some(secret) = &self.client_secret {
            form.push(("client_secret", secret.as_str()));
        }
        form
    }

    /// Begin the device-code flow (RFC 8628): request a device + user code the operator
    /// enters at the verification URL.
    pub async fn begin_device_authorization(
        &self,
    ) -> std::result::Result<DeviceAuthorization, OauthError> {
        let scope = self.scope_str();
        let form = self.with_secret(vec![
            ("client_id", self.client_id.as_str()),
            ("scope", scope.as_str()),
        ]);
        let resp = self
            .poster
            .post_form(&self.endpoints.device_authorization, &form)
            .await?;
        let wire: DeviceAuthWire = serde_json::from_str(&resp.body)
            .map_err(|e| OauthError::Malformed(format!("device authorization: {e}")))?;
        if let Some(error) = wire.error {
            return Err(OauthError::Provider {
                error,
                description: wire.error_description,
            });
        }
        Ok(DeviceAuthorization {
            device_code: wire.device_code.ok_or_else(|| {
                OauthError::Malformed("device authorization: no device_code".into())
            })?,
            user_code: wire.user_code.unwrap_or_default(),
            verification_uri: wire
                .verification_uri
                .or(wire.verification_url)
                .unwrap_or_default(),
            expires_in: wire.expires_in.unwrap_or(900),
            interval: wire.interval.unwrap_or(5),
        })
    }

    /// Poll the token endpoint once for a device-code grant. Maps the transient
    /// `authorization_pending` / `slow_down` errors to [`DevicePoll`] states; any other
    /// error is terminal.
    pub async fn poll_device_token(
        &self,
        device_code: &str,
    ) -> std::result::Result<DevicePoll, OauthError> {
        let form = self.with_secret(vec![
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("client_id", self.client_id.as_str()),
            ("device_code", device_code),
        ]);
        let resp = self.poster.post_form(&self.endpoints.token, &form).await?;
        match parse_token_response(&resp)? {
            Ok(token) => Ok(DevicePoll::Complete(token)),
            Err(OauthError::Provider { error, description }) => match error.as_str() {
                "authorization_pending" => Ok(DevicePoll::Pending),
                "slow_down" => Ok(DevicePoll::SlowDown),
                _ => Err(OauthError::Provider { error, description }),
            },
            Err(e) => Err(e),
        }
    }

    /// Exchange an authorization code (redirect flow) for a token set. `code_verifier`
    /// is the PKCE verifier; pass an empty string for a non-PKCE confidential client.
    pub async fn exchange_authorization_code(
        &self,
        code: &str,
        redirect_uri: &str,
        code_verifier: &str,
    ) -> std::result::Result<TokenResponse, OauthError> {
        let mut form = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", self.client_id.as_str()),
        ];
        if !code_verifier.is_empty() {
            form.push(("code_verifier", code_verifier));
        }
        let form = self.with_secret(form);
        let resp = self.poster.post_form(&self.endpoints.token, &form).await?;
        parse_token_response(&resp)?
    }

    /// Refresh an access token from a stored refresh token. Google may not rotate the
    /// refresh token (the response omits it) — the caller keeps the prior one.
    pub async fn refresh(
        &self,
        refresh_token: &str,
    ) -> std::result::Result<TokenResponse, OauthError> {
        let scope = self.scope_str();
        let mut form = vec![
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", self.client_id.as_str()),
        ];
        if !scope.is_empty() {
            form.push(("scope", scope.as_str()));
        }
        let form = self.with_secret(form);
        let resp = self.poster.post_form(&self.endpoints.token, &form).await?;
        parse_token_response(&resp)?
    }
}

/// Parse a token-endpoint response body into a [`TokenResponse`], or the OAuth error it
/// carried. The outer `Result` is a transport/parse failure; the inner is the
/// provider's success-or-error object (so the device poller can classify the error).
fn parse_token_response(
    resp: &FormResponse,
) -> std::result::Result<std::result::Result<TokenResponse, OauthError>, OauthError> {
    let wire: TokenWire = serde_json::from_str(&resp.body).map_err(|e| {
        OauthError::Malformed(format!("token response (status {}): {e}", resp.status))
    })?;
    if let Some(error) = wire.error {
        return Ok(Err(OauthError::Provider {
            error,
            description: wire.error_description,
        }));
    }
    let Some(access_token) = wire.access_token else {
        return Ok(Err(OauthError::Malformed(format!(
            "token response (status {}) had neither access_token nor error",
            resp.status
        ))));
    };
    Ok(Ok(TokenResponse {
        access_token,
        refresh_token: wire.refresh_token,
        // A provider is not required to return `expires_in`; default to one hour.
        expires_in: wire.expires_in.unwrap_or(3600),
        scope: wire.scope,
    }))
}

/// Whether a cached token (RFC-3339 `expires_at`) is due for a proactive refresh now.
/// An unparseable timestamp is treated as expired (fail toward a refresh, never serve a
/// possibly-dead token).
fn needs_refresh(expires_at: &str, now: DateTime<Utc>) -> bool {
    match DateTime::parse_from_rfc3339(expires_at) {
        Ok(exp) => now + EXPIRY_SKEW >= exp.with_timezone(&Utc),
        Err(_) => true,
    }
}

/// Build the per-account [`OauthClient`] from the account's `bridge_accounts` binding
/// (0008 `oauth_ref` config + bridge id → provider) and the deployment client secret.
async fn build_client<'a>(
    store: &Store,
    poster: &'a dyn FormPoster,
    client_secret: Option<&str>,
    account: &str,
) -> std::result::Result<OauthClient<'a>, OauthError> {
    // There's a single binding per local account; filter the (small) list rather than
    // add a store method — this file doesn't own `v7_config.rs`.
    let bindings = store
        .list_bridge_accounts()
        .await
        .map_err(|e| OauthError::Config(format!("bridge_accounts read failed: {e}")))?;
    let binding = bindings
        .into_iter()
        .find(|b| b.account_id == account)
        .ok_or_else(|| OauthError::Config(format!("no bridge account binding for '{account}'")))?;

    let raw = binding.oauth_ref.unwrap_or_default();
    if raw.trim().is_empty() {
        return Err(OauthError::Config(format!(
            "bridge account '{account}' has no oauth_ref configured"
        )));
    }
    let cfg: BridgeOauthConfig = serde_json::from_str(&raw).map_err(|e| {
        OauthError::Config(format!(
            "bridge account '{account}' oauth_ref is not a valid OAuth config object: {e}"
        ))
    })?;

    let provider = cfg
        .provider
        .as_deref()
        .and_then(Provider::parse)
        .or_else(|| Provider::from_bridge_id(&binding.bridge_id))
        .ok_or_else(|| {
            OauthError::Config(format!(
                "bridge account '{account}': unknown OAuth provider (set oauth_ref.provider)"
            ))
        })?;

    let tenant = cfg.tenant.clone().unwrap_or_default();
    let endpoints = Endpoints::resolve(provider, &tenant, &cfg);
    let scopes = if cfg.scopes.is_empty() {
        provider.default_scopes()
    } else {
        cfg.scopes.clone()
    };
    Ok(OauthClient {
        poster,
        provider,
        client_id: cfg.client_id,
        client_secret: client_secret.map(str::to_string),
        scopes,
        endpoints,
    })
}

/// The live `oauth-token` path: return a valid access token for a bridge `account`,
/// refreshing transparently when the cached one is near expiry.
///
/// 1. cached + not near expiry ⇒ return it (no network);
/// 2. cached + near expiry + has a refresh token ⇒ refresh, re-cache (sealed), return;
/// 3. otherwise ⇒ error — the account must first be enrolled (device-/auth-code) to mint
///    a refresh token. Never blocks on user interaction (a guest call can't drive a
///    device flow), so it fails cleanly rather than hanging.
///
/// Returns `Result<String, String>` to match the `OAuthTokenProvider::token` seam.
pub async fn acquire_access_token(
    store: &Store,
    poster: &dyn FormPoster,
    client_secret: Option<&str>,
    account: &str,
) -> std::result::Result<String, String> {
    let cached = store
        .get_bridge_oauth_token(account)
        .await
        .map_err(|e| format!("cached-token read failed: {e}"))?;

    let now = Utc::now();
    if let Some(tok) = &cached
        && !needs_refresh(&tok.expires_at, now)
    {
        return Ok(tok.access_token.clone());
    }

    let Some(refresh_token) = cached.as_ref().and_then(|t| t.refresh_token.clone()) else {
        return Err(format!(
            "no usable OAuth token for bridge account '{account}': run the device-code or \
             authorization-code enrolment to mint a refresh token"
        ));
    };

    let client = build_client(store, poster, client_secret, account)
        .await
        .map_err(|e| e.to_string())?;
    let refreshed = client
        .refresh(&refresh_token)
        .await
        .map_err(|e| e.to_string())?;

    // Google often omits a rotated refresh token on refresh — keep the one we used.
    let new_refresh = refreshed.refresh_token.or(Some(refresh_token));
    let expires_at = (now + Duration::from_secs(refreshed.expires_in.max(0) as u64)).to_rfc3339();
    let scope = if refreshed.scope.is_empty() {
        cached.map(|t| t.scope).unwrap_or_default()
    } else {
        refreshed.scope
    };
    let row = BridgeOauthTokenRow {
        bridge_account_id: account.to_string(),
        access_token: refreshed.access_token.clone(),
        refresh_token: new_refresh,
        expires_at,
        scope,
        updated_at: String::new(),
    };
    store
        .put_bridge_oauth_token(&row)
        .await
        .map_err(|e| format!("cache write failed: {e}"))?;
    Ok(refreshed.access_token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use mw_store::ServerKey;

    /// One recorded POST: the URL and its decoded form fields.
    type RecordedCall = (String, Vec<(String, String)>);

    /// A mock IdP: canned responses served FIFO, recording each POST for assertions.
    #[derive(Default)]
    struct MockIdp {
        calls: Mutex<Vec<RecordedCall>>,
        responses: Mutex<Vec<FormResponse>>,
    }

    impl MockIdp {
        fn with(responses: Vec<FormResponse>) -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                responses: Mutex::new(responses),
            }
        }
        fn json(status: u16, body: serde_json::Value) -> FormResponse {
            FormResponse {
                status,
                body: body.to_string(),
            }
        }
        fn last_form_value(&self, key: &str) -> Option<String> {
            let calls = self.calls.lock().unwrap();
            let (_, form) = calls.last()?;
            form.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
        }
    }

    #[async_trait]
    impl FormPoster for MockIdp {
        async fn post_form(
            &self,
            url: &str,
            form: &[(&str, &str)],
        ) -> std::result::Result<FormResponse, OauthError> {
            self.calls.lock().unwrap().push((
                url.to_string(),
                form.iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            ));
            let mut r = self.responses.lock().unwrap();
            if r.is_empty() {
                return Err(OauthError::Transport(
                    "mock: no canned response left".into(),
                ));
            }
            Ok(r.remove(0))
        }
    }

    /// A poster that must never be called (used to prove the cached-hit path is offline).
    struct PanicPoster;
    #[async_trait]
    impl FormPoster for PanicPoster {
        async fn post_form(
            &self,
            _url: &str,
            _form: &[(&str, &str)],
        ) -> std::result::Result<FormResponse, OauthError> {
            panic!("token endpoint must not be contacted for an unexpired cached token");
        }
    }

    fn client<'a>(poster: &'a dyn FormPoster, token_endpoint: &str) -> OauthClient<'a> {
        OauthClient {
            poster,
            provider: Provider::Microsoft,
            client_id: "app-123".into(),
            client_secret: None,
            scopes: vec!["offline_access".into()],
            endpoints: Endpoints {
                token: token_endpoint.into(),
                device_authorization: format!("{token_endpoint}/devicecode"),
                authorize: format!("{token_endpoint}/authorize"),
            },
        }
    }

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    async fn seed_binding(store: &Store, account: &str, oauth_ref: serde_json::Value) {
        store
            .put_bridge_account(&mw_store::BridgeAccountRow {
                account_id: account.into(),
                bridge_id: "bridge-graph".into(),
                oauth_ref: Some(oauth_ref.to_string()),
                extra_json: "{}".into(),
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn refresh_parses_token_and_forwards_grant() {
        let idp = MockIdp::with(vec![MockIdp::json(
            200,
            serde_json::json!({
                "access_token": "AT-new",
                "refresh_token": "RT-rotated",
                "expires_in": 3600,
                "scope": "Mail.ReadWrite offline_access",
            }),
        )]);
        let c = client(&idp, "https://idp.test/token");
        let tok = c.refresh("RT-old").await.unwrap();
        assert_eq!(tok.access_token, "AT-new");
        assert_eq!(tok.refresh_token.as_deref(), Some("RT-rotated"));
        assert_eq!(tok.expires_in, 3600);
        // The correct grant + the caller's refresh token went on the wire.
        assert_eq!(
            idp.last_form_value("grant_type").as_deref(),
            Some("refresh_token")
        );
        assert_eq!(
            idp.last_form_value("refresh_token").as_deref(),
            Some("RT-old")
        );
    }

    #[tokio::test]
    async fn refresh_surfaces_provider_error() {
        let idp = MockIdp::with(vec![MockIdp::json(
            400,
            serde_json::json!({ "error": "invalid_grant", "error_description": "expired" }),
        )]);
        let c = client(&idp, "https://idp.test/token");
        let err = c.refresh("RT-dead").await.unwrap_err();
        assert_eq!(
            err,
            OauthError::Provider {
                error: "invalid_grant".into(),
                description: "expired".into()
            }
        );
    }

    #[tokio::test]
    async fn device_flow_begins_then_completes_after_pending() {
        let idp = MockIdp::with(vec![
            // begin_device_authorization
            MockIdp::json(
                200,
                serde_json::json!({
                    "device_code": "DC-1",
                    "user_code": "WXYZ-1234",
                    "verification_uri": "https://microsoft.com/devicelogin",
                    "expires_in": 900,
                    "interval": 5,
                }),
            ),
            // first poll — pending
            MockIdp::json(400, serde_json::json!({ "error": "authorization_pending" })),
            // second poll — complete
            MockIdp::json(
                200,
                serde_json::json!({
                    "access_token": "AT-dev",
                    "refresh_token": "RT-dev",
                    "expires_in": 3600,
                }),
            ),
        ]);
        let c = client(&idp, "https://idp.test/token");
        let da = c.begin_device_authorization().await.unwrap();
        assert_eq!(da.device_code, "DC-1");
        assert_eq!(da.user_code, "WXYZ-1234");
        assert_eq!(da.verification_uri, "https://microsoft.com/devicelogin");

        assert_eq!(
            c.poll_device_token("DC-1").await.unwrap(),
            DevicePoll::Pending
        );
        match c.poll_device_token("DC-1").await.unwrap() {
            DevicePoll::Complete(t) => {
                assert_eq!(t.access_token, "AT-dev");
                assert_eq!(t.refresh_token.as_deref(), Some("RT-dev"));
            }
            other => panic!("expected completion, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn device_poll_maps_slow_down() {
        let idp = MockIdp::with(vec![MockIdp::json(
            400,
            serde_json::json!({ "error": "slow_down" }),
        )]);
        let c = client(&idp, "https://idp.test/token");
        assert_eq!(
            c.poll_device_token("DC").await.unwrap(),
            DevicePoll::SlowDown
        );
    }

    #[tokio::test]
    async fn authorization_code_exchange_sends_pkce_verifier() {
        let idp = MockIdp::with(vec![MockIdp::json(
            200,
            serde_json::json!({ "access_token": "AT", "refresh_token": "RT", "expires_in": 3600 }),
        )]);
        let c = client(&idp, "https://idp.test/token");
        let tok = c
            .exchange_authorization_code("CODE", "https://app/cb", "VERIFIER")
            .await
            .unwrap();
        assert_eq!(tok.access_token, "AT");
        assert_eq!(
            idp.last_form_value("grant_type").as_deref(),
            Some("authorization_code")
        );
        assert_eq!(
            idp.last_form_value("code_verifier").as_deref(),
            Some("VERIFIER")
        );
    }

    #[test]
    fn needs_refresh_honors_skew() {
        let now = Utc::now();
        // Expires in 10 minutes ⇒ still fresh.
        let fresh = (now + Duration::from_secs(600)).to_rfc3339();
        assert!(!needs_refresh(&fresh, now));
        // Expires within the skew window ⇒ refresh.
        let soon = (now + Duration::from_secs(30)).to_rfc3339();
        assert!(needs_refresh(&soon, now));
        // Already expired ⇒ refresh; unparseable ⇒ refresh.
        let past = (now - Duration::from_secs(60)).to_rfc3339();
        assert!(needs_refresh(&past, now));
        assert!(needs_refresh("not-a-timestamp", now));
    }

    #[tokio::test]
    async fn acquire_returns_cached_when_fresh_without_network() {
        let s = store().await;
        let account = "alice@corp";
        seed_binding(&s, account, serde_json::json!({ "client_id": "app-123" })).await;
        let expires_at = (Utc::now() + Duration::from_secs(3600)).to_rfc3339();
        s.put_bridge_oauth_token(&BridgeOauthTokenRow {
            bridge_account_id: account.into(),
            access_token: "AT-cached".into(),
            refresh_token: Some("RT".into()),
            expires_at,
            scope: "Mail.Read".into(),
            updated_at: String::new(),
        })
        .await
        .unwrap();

        // PanicPoster proves no token-endpoint call happens on a fresh cache hit.
        let got = acquire_access_token(&s, &PanicPoster, None, account)
            .await
            .unwrap();
        assert_eq!(got, "AT-cached");
    }

    #[tokio::test]
    async fn acquire_refreshes_expired_and_recaches_sealed() {
        let s = store().await;
        let account = "bob@corp";
        // Point the client at the mock IdP via an endpoint override in oauth_ref.
        seed_binding(
            &s,
            account,
            serde_json::json!({
                "provider": "microsoft",
                "client_id": "app-123",
                "token_endpoint": "https://idp.test/token",
            }),
        )
        .await;
        // Cached token already expired, but carries a refresh token.
        let expired = (Utc::now() - Duration::from_secs(60)).to_rfc3339();
        s.put_bridge_oauth_token(&BridgeOauthTokenRow {
            bridge_account_id: account.into(),
            access_token: "AT-old".into(),
            refresh_token: Some("RT-old".into()),
            expires_at: expired,
            scope: "Mail.Read".into(),
            updated_at: String::new(),
        })
        .await
        .unwrap();

        let idp = MockIdp::with(vec![MockIdp::json(
            200,
            serde_json::json!({
                "access_token": "AT-fresh",
                // No rotated refresh token ⇒ the prior one is retained.
                "expires_in": 3600,
                "scope": "Mail.Read",
            }),
        )]);
        let got = acquire_access_token(&s, &idp, None, account).await.unwrap();
        assert_eq!(got, "AT-fresh");
        // The refresh used the stored refresh token.
        assert_eq!(
            idp.last_form_value("refresh_token").as_deref(),
            Some("RT-old")
        );

        // Re-cached through the sealed 0018 writer: new access token, retained refresh
        // token, future expiry. (The byte-level "sealed at rest, not plaintext" proof
        // lives in the mw-store `bridge_tokens` unit test over this same writer; the
        // e2e lane re-verifies non-plaintext bytes on the real filesystem.)
        let cached = s.get_bridge_oauth_token(account).await.unwrap().unwrap();
        assert_eq!(cached.access_token, "AT-fresh");
        assert_eq!(cached.refresh_token.as_deref(), Some("RT-old"));
        assert!(!needs_refresh(&cached.expires_at, Utc::now()));
    }

    #[tokio::test]
    async fn acquire_errors_when_uncached() {
        let s = store().await;
        let account = "carol@corp";
        seed_binding(&s, account, serde_json::json!({ "client_id": "app-123" })).await;
        let err = acquire_access_token(&s, &PanicPoster, None, account)
            .await
            .unwrap_err();
        assert!(err.contains("enrolment"), "unexpected error: {err}");
    }

    #[tokio::test]
    async fn acquire_errors_when_expired_but_no_refresh_token() {
        let s = store().await;
        let account = "dave@corp";
        seed_binding(&s, account, serde_json::json!({ "client_id": "app-123" })).await;
        let expired = (Utc::now() - Duration::from_secs(60)).to_rfc3339();
        s.put_bridge_oauth_token(&BridgeOauthTokenRow {
            bridge_account_id: account.into(),
            access_token: "AT-old".into(),
            refresh_token: None,
            expires_at: expired,
            scope: String::new(),
            updated_at: String::new(),
        })
        .await
        .unwrap();
        let err = acquire_access_token(&s, &PanicPoster, None, account)
            .await
            .unwrap_err();
        assert!(err.contains("enrolment"), "unexpected error: {err}");
    }
}
