//! OIDC login backend (t9-e1).
//!
//! Implements [`SsoLogin`] for [`OidcProvider`] over the `openidconnect` crate,
//! wired to the in-tree rustls `reqwest` async client (NOT openidconnect's default
//! native-tls client — see the crate `Cargo.toml` VET note). The flow is:
//!
//! * **discovery** — `.well-known/openid-configuration` + JWKS fetch, via
//!   [`ProviderMetadataWithLogout::discover_async`] so the `end_session_endpoint`
//!   (RP-initiated logout) is learned too.
//! * [`begin`](SsoLogin::begin) — authorization-code + **mandatory PKCE** + nonce +
//!   an opaque `state`; the PKCE verifier + nonce are returned in the
//!   [`PendingState`](crate::state::PendingState) the server persists under the
//!   opaque state token (replay/CSRF binding lives server-side).
//! * [`complete`](SsoLogin::complete) — exchange the code, **validate the ID token**
//!   (JWKS signature, `iss`/`aud`/`exp`, and the nonce from the resolved pending
//!   state), fetch userinfo, and map claims → [`SsoIdentity`] via the config
//!   [`ClaimMap`].
//! * [`logout`](SsoLogin::logout) — RP-initiated logout URL from the discovered
//!   `end_session_endpoint`, if the IdP advertises one (cached from discovery).
//!
//! Every failure returns a [`SsoError`] variant — all of which map to a uniform 401
//! at the route, so a caller never learns which check failed.

use std::sync::{Arc, OnceLock};

use openidconnect::core::{CoreAuthenticationFlow, CoreClient};
use openidconnect::http::{Request as HttpRequest, header};
use openidconnect::{
    AccessToken, AsyncHttpClient, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    EndSessionUrl, EndpointMaybeSet, EndpointNotSet, EndpointSet, IssuerUrl, LogoutHint,
    LogoutRequest, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier,
    ProviderMetadataWithLogout, RedirectUrl, Scope, TokenResponse,
};
use serde_json::Value;

/// The `CoreClient` in the endpoint type-state produced by
/// [`CoreClient::from_provider_metadata`] + `set_redirect_uri`: the authorization
/// endpoint is set, the token + userinfo endpoints are "maybe set" (present iff the
/// discovery doc advertised them).
type ConfiguredClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

use crate::state::PendingState;
use crate::{
    BeginRedirect, ClaimMap, Metadata, OidcConfig, Redirect, SsoBackend, SsoCallback, SsoConfig,
    SsoError, SsoIdentity, SsoLogin, state,
};

/// An OIDC relying-party backend implementing [`SsoLogin`].
///
/// Construction validates the config into the `openidconnect` types (issuer URL);
/// discovery + the network-bound flow happen per call in [`begin`](SsoLogin::begin)
/// / [`complete`](SsoLogin::complete). The `end_session_endpoint` learned during a
/// discovery is cached so the synchronous [`logout`](SsoLogin::logout) can build an
/// RP-initiated logout URL.
#[derive(Debug, Clone)]
pub struct OidcProvider {
    backend: SsoBackend,
    /// The `end_session_endpoint` cached from the last successful discovery, so the
    /// synchronous `logout()` can build a redirect without a network round-trip.
    end_session: Arc<OnceLock<String>>,
}

impl OidcProvider {
    /// Build a provider from a resolved [`SsoBackend`]. Errors if the backend is not
    /// OIDC or the issuer URL is malformed (the cheap validation without the
    /// network; discovery happens on the first login).
    pub fn new(backend: SsoBackend) -> Result<Self, SsoError> {
        let cfg = Self::oidc_config(&backend)?;
        // Validate the issuer eagerly so a misconfigured backend fails at build,
        // not mid-login. This also links `openidconnect` into the vetted tree.
        let _issuer = issuer_url(cfg)?;
        Ok(OidcProvider {
            backend,
            end_session: Arc::new(OnceLock::new()),
        })
    }

    /// The resolved backend.
    pub fn backend(&self) -> &SsoBackend {
        &self.backend
    }

    fn oidc_config(backend: &SsoBackend) -> Result<&OidcConfig, SsoError> {
        match &backend.config {
            SsoConfig::Oidc(c) => Ok(c),
            SsoConfig::Saml(_) => Err(SsoError::Config(
                "OidcProvider built from a SAML backend".into(),
            )),
        }
    }

    fn cfg(&self) -> &OidcConfig {
        match &self.backend.config {
            SsoConfig::Oidc(c) => c,
            // `new` rejects the wrong kind, so this is unreachable in practice.
            SsoConfig::Saml(_) => unreachable!("OidcProvider always holds an OIDC config"),
        }
    }

    fn client_secret(&self) -> Option<ClientSecret> {
        self.backend
            .secret
            .as_ref()
            .and_then(|b| String::from_utf8(b.clone()).ok())
            .map(ClientSecret::new)
    }

    /// Build the in-tree rustls `reqwest` async client. Redirects are disabled to
    /// avoid SSRF via the IdP endpoints (per the `openidconnect` guidance).
    fn http_client() -> Result<openidconnect::reqwest::Client, SsoError> {
        openidconnect::reqwest::ClientBuilder::new()
            .redirect(openidconnect::reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| SsoError::Upstream(format!("http client build: {e}")))
    }

    /// Discover the provider metadata (`.well-known/openid-configuration` + JWKS),
    /// caching the `end_session_endpoint` for `logout()`. Generic over the async
    /// HTTP client so tests can inject recorded fixtures.
    async fn discover<C>(&self, http: &C) -> Result<ProviderMetadataWithLogout, SsoError>
    where
        C: for<'a> AsyncHttpClient<'a> + Sync,
    {
        let issuer = issuer_url(self.cfg())?;
        let meta = ProviderMetadataWithLogout::discover_async(issuer, http)
            .await
            .map_err(|e| SsoError::Discovery(e.to_string()))?;
        if let Some(url) = meta.additional_metadata().end_session_endpoint.as_ref() {
            let _ = self.end_session.set(url.url().to_string());
        }
        Ok(meta)
    }

    /// Build the `openidconnect` client from discovered metadata + the config.
    fn build_client(&self, meta: ProviderMetadataWithLogout) -> Result<ConfiguredClient, SsoError> {
        let cfg = self.cfg();
        let redirect = RedirectUrl::new(cfg.redirect_url.clone())
            .map_err(|e| SsoError::Config(format!("bad redirect_url: {e}")))?;
        Ok(CoreClient::from_provider_metadata(
            meta,
            ClientId::new(cfg.client_id.clone()),
            self.client_secret(),
        )
        .set_redirect_uri(redirect))
    }

    /// The generic (fixture-injectable) core of [`begin`](SsoLogin::begin).
    async fn begin_impl<C>(
        &self,
        relay_state: Option<String>,
        http: &C,
    ) -> Result<BeginRedirect, SsoError>
    where
        C: for<'a> AsyncHttpClient<'a> + Sync,
    {
        let meta = self.discover(http).await?;
        let client = self.build_client(meta)?;

        // The opaque `state` doubles as the PendingStore correlator (the server
        // resolves it back to the PendingState on callback).
        let state_token = state::new_state_token();
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let st = state_token.clone();
        let mut req = client.authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            move || CsrfToken::new(st),
            Nonce::new_random,
        );
        // `openidconnect` always adds the `openid` scope; add the rest from config.
        for scope in &self.cfg().scopes {
            if scope != "openid" {
                req = req.add_scope(Scope::new(scope.clone()));
            }
        }
        let (url, _csrf, nonce) = req.set_pkce_challenge(pkce_challenge).url();

        let pending = PendingState::Oidc {
            pkce_verifier: pkce_verifier.into_secret(),
            nonce: nonce.secret().clone(),
            relay_state,
        };
        Ok(BeginRedirect {
            url: url.to_string(),
            state_token,
            pending,
        })
    }

    /// The generic (fixture-injectable) core of [`complete`](SsoLogin::complete).
    async fn complete_impl<C>(
        &self,
        callback: SsoCallback,
        http: &C,
    ) -> Result<SsoIdentity, SsoError>
    where
        C: for<'a> AsyncHttpClient<'a> + Sync,
    {
        // The server resolved the opaque state token → PendingState BEFORE calling
        // us (one-shot; a miss is already `Replay`). We only trust the OIDC variant.
        let (pkce_verifier, nonce) = match &callback.pending {
            PendingState::Oidc {
                pkce_verifier,
                nonce,
                ..
            } => (pkce_verifier.clone(), nonce.clone()),
            PendingState::Saml { .. } => {
                return Err(SsoError::Replay);
            }
        };

        // If the IdP signalled an error on the redirect, fail closed.
        if let Some(err) = callback.param("error") {
            return Err(SsoError::Upstream(format!("idp error: {err}")));
        }
        let code = callback
            .param("code")
            .ok_or_else(|| SsoError::TokenValidation("missing authorization code".into()))?
            .to_string();

        let meta = self.discover(http).await?;
        let client = self.build_client(meta)?;

        // Exchange the code (PKCE-bound). openssl-free: JWS/PKCE are RustCrypto.
        let token_response = client
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|e| SsoError::Config(format!("token endpoint: {e}")))?
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier))
            .request_async(http)
            .await
            .map_err(|e| SsoError::Upstream(format!("code exchange: {e}")))?;

        // Validate the ID token: signature (JWKS), iss/aud/exp, and the nonce we
        // bound at begin() (carried in the resolved PendingState).
        let id_token = token_response
            .id_token()
            .ok_or_else(|| SsoError::TokenValidation("no id_token in token response".into()))?;
        let verifier = client.id_token_verifier();
        let expected_nonce = Nonce::new(nonce);
        let claims = id_token
            .claims(&verifier, &expected_nonce)
            .map_err(map_claims_error)?;

        let subject = claims.subject().to_string();

        // Raw claim view: the verified ID-token standard claims, enriched with a
        // best-effort userinfo fetch (custom claims like `groups` usually land
        // there). NEVER contains tokens or the assertion.
        let mut merged = serde_json::Map::new();
        if let Ok(Value::Object(m)) = serde_json::to_value(claims) {
            merged.extend(m);
        }
        if let Some(ui) = self
            .fetch_userinfo(&client, token_response.access_token(), http)
            .await
        {
            merged.extend(ui);
        }

        Ok(map_identity(subject, &merged, &self.backend.claim_map))
    }

    /// Best-effort raw userinfo fetch through the same async HTTP client, returning
    /// the JSON object (custom claims such as `groups` included). A missing endpoint
    /// or any error yields `None` — the validated ID token is the authoritative
    /// identity, userinfo is only enrichment.
    async fn fetch_userinfo<C>(
        &self,
        client: &ConfiguredClient,
        access_token: &AccessToken,
        http: &C,
    ) -> Option<serde_json::Map<String, Value>>
    where
        C: for<'a> AsyncHttpClient<'a> + Sync,
    {
        let endpoint = client.user_info_url()?.url().to_string();
        let req = HttpRequest::builder()
            .method("GET")
            .uri(endpoint)
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", access_token.secret()),
            )
            .header(header::ACCEPT, "application/json")
            .body(Vec::new())
            .ok()?;
        let resp = http.call(req).await.ok()?;
        match serde_json::from_slice::<Value>(resp.body()) {
            Ok(Value::Object(m)) => Some(m),
            _ => None,
        }
    }
}

/// Map an `openidconnect` claims-verification failure onto the right [`SsoError`]
/// (still a uniform 401 at the route; this only sets the audit reason).
fn map_claims_error(e: openidconnect::ClaimsVerificationError) -> SsoError {
    use openidconnect::ClaimsVerificationError as C;
    match e {
        C::Expired(m) => {
            tracing::debug!(reason = %m, "oidc id-token expired");
            SsoError::Expired
        }
        C::InvalidAudience(m) => {
            tracing::debug!(reason = %m, "oidc id-token audience mismatch");
            SsoError::AudienceMismatch
        }
        C::InvalidNonce(m) => {
            tracing::debug!(reason = %m, "oidc id-token nonce mismatch");
            SsoError::Replay
        }
        C::SignatureVerification(inner) => SsoError::SignatureInvalid(inner.to_string()),
        other => SsoError::TokenValidation(other.to_string()),
    }
}

/// Build the [`SsoIdentity`] from the merged claim object and the config
/// [`ClaimMap`]. `subject` is the authoritative (validated) ID-token `sub`.
fn map_identity(
    subject: String,
    merged: &serde_json::Map<String, Value>,
    map: &ClaimMap,
) -> SsoIdentity {
    let email = pick(merged, map.email.as_deref().unwrap_or("email"));
    let display_name = pick(merged, map.display.as_deref().unwrap_or("name"));
    let groups = pick_groups(merged, map.groups.as_deref().unwrap_or("groups"));

    // The content-free scalar claim view for e3's advanced mapping + audit.
    let mut claims = std::collections::BTreeMap::new();
    for (k, v) in merged {
        if let Some(s) = scalar(v) {
            claims.insert(k.clone(), s);
        }
    }

    SsoIdentity {
        subject,
        email,
        display_name,
        groups,
        claims,
    }
}

/// A single scalar claim value as a string, or `None` for arrays/objects/null.
fn scalar(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Pick a scalar claim by name.
fn pick(merged: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    merged.get(key).and_then(scalar)
}

/// Pick a multi-valued group claim: an array of strings, a single string, or a
/// space/comma-separated string.
fn pick_groups(merged: &serde_json::Map<String, Value>, key: &str) -> Vec<String> {
    match merged.get(key) {
        Some(Value::Array(items)) => items.iter().filter_map(scalar).collect(),
        Some(Value::String(s)) => s
            .split([',', ' '])
            .filter(|g| !g.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

/// Parse the configured issuer into an `openidconnect::IssuerUrl` (the discovery
/// anchor). A real, compiling use of the vetted `openidconnect` tree.
pub fn issuer_url(cfg: &OidcConfig) -> Result<IssuerUrl, SsoError> {
    IssuerUrl::new(cfg.issuer_url.clone())
        .map_err(|e| SsoError::Config(format!("bad issuer_url: {e}")))
}

#[async_trait::async_trait]
impl SsoLogin for OidcProvider {
    async fn begin(&self, relay_state: Option<String>) -> Result<BeginRedirect, SsoError> {
        let http = Self::http_client()?;
        self.begin_impl(relay_state, &http).await
    }

    async fn complete(&self, callback: SsoCallback) -> Result<SsoIdentity, SsoError> {
        let http = Self::http_client()?;
        self.complete_impl(callback, &http).await
    }

    fn metadata(&self) -> Option<Metadata> {
        // OIDC advertises via the IdP discovery doc, not SP metadata.
        None
    }

    fn logout(&self, subject: &str) -> Option<Redirect> {
        // RP-initiated logout needs the `end_session_endpoint`, learned during a
        // prior discovery (begin/complete) and cached. Best-effort: no id_token
        // hint is available here, so we pass the subject as a logout hint.
        let endpoint = self.end_session.get()?;
        let url = EndSessionUrl::new(endpoint.clone()).ok()?;
        let req = LogoutRequest::from(url)
            .set_client_id(ClientId::new(self.cfg().client_id.clone()))
            .set_logout_hint(LogoutHint::new(subject.to_string()));
        Some(Redirect {
            url: req.http_get_url().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClaimMap, SsoKind, SsoScope};

    fn oidc_backend(issuer: &str) -> SsoBackend {
        SsoBackend {
            id: "corp-oidc".into(),
            kind: SsoKind::Oidc,
            display_name: "Acme SSO".into(),
            scope: SsoScope::Deployment,
            enabled: true,
            config: SsoConfig::Oidc(OidcConfig {
                issuer_url: issuer.into(),
                client_id: "mailwoman".into(),
                redirect_url: "https://mail.example/api/sso/corp-oidc/callback".into(),
                scopes: vec!["openid".into(), "email".into()],
                ..Default::default()
            }),
            claim_map: ClaimMap::default(),
            secret: Some(b"client-secret".to_vec()),
        }
    }

    #[test]
    fn builds_from_valid_oidc_backend() {
        let p = OidcProvider::new(oidc_backend("https://idp.example")).unwrap();
        assert_eq!(p.backend().id, "corp-oidc");
    }

    #[test]
    fn rejects_bad_issuer_url() {
        assert!(matches!(
            OidcProvider::new(oidc_backend("not a url")),
            Err(SsoError::Config(_))
        ));
    }

    #[test]
    fn rejects_saml_backend() {
        let mut b = oidc_backend("https://idp.example");
        b.config = SsoConfig::Saml(Default::default());
        assert!(matches!(OidcProvider::new(b), Err(SsoError::Config(_))));
    }

    #[test]
    fn logout_none_before_discovery() {
        // No discovery has populated the end_session cache yet.
        let p = OidcProvider::new(oidc_backend("https://idp.example")).unwrap();
        assert!(p.logout("subject").is_none());
    }

    #[test]
    fn logout_builds_rp_initiated_url_when_cached() {
        let p = OidcProvider::new(oidc_backend("https://idp.example")).unwrap();
        p.end_session
            .set("https://idp.example/logout".into())
            .unwrap();
        let r = p.logout("user-123").expect("logout url");
        assert!(r.url.starts_with("https://idp.example/logout"));
        assert!(r.url.contains("client_id=mailwoman"));
    }

    #[test]
    fn map_identity_uses_claim_map_and_defaults() {
        let mut merged = serde_json::Map::new();
        merged.insert("email".into(), Value::String("a@b.test".into()));
        merged.insert("name".into(), Value::String("Ada".into()));
        merged.insert(
            "groups".into(),
            Value::Array(vec![
                Value::String("admins".into()),
                Value::String("users".into()),
            ]),
        );
        let id = map_identity("sub-1".into(), &merged, &ClaimMap::default());
        assert_eq!(id.subject, "sub-1");
        assert_eq!(id.email.as_deref(), Some("a@b.test"));
        assert_eq!(id.display_name.as_deref(), Some("Ada"));
        assert_eq!(id.groups, vec!["admins".to_string(), "users".into()]);
        assert!(id.claims.contains_key("email"));
    }

    #[test]
    fn map_identity_honours_custom_claim_names() {
        let mut merged = serde_json::Map::new();
        merged.insert("mail".into(), Value::String("c@d.test".into()));
        merged.insert("cn".into(), Value::String("Grace".into()));
        merged.insert("roles".into(), Value::String("staff eng".into()));
        let map = ClaimMap {
            email: Some("mail".into()),
            display: Some("cn".into()),
            groups: Some("roles".into()),
            username: None,
        };
        let id = map_identity("sub-2".into(), &merged, &map);
        assert_eq!(id.email.as_deref(), Some("c@d.test"));
        assert_eq!(id.display_name.as_deref(), Some("Grace"));
        assert_eq!(id.groups, vec!["staff".to_string(), "eng".into()]);
    }

    // ── Fixture-driven begin→complete round-trip ────────────────────────────
    //
    // Recorded fixtures (generated offline from a test RSA key — see
    // `tests/fixtures/`): a discovery doc, the matching JWKS, and RS256 ID-token
    // variants. A mock `AsyncHttpClient` serves them so the real openidconnect
    // validation path (discovery, JWKS signature, iss/aud/exp/nonce, userinfo) runs
    // with NO live IdP. Injected via the private `*_impl` seams.

    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake};

    use openidconnect::http::Response;
    use openidconnect::{HttpRequest as OidcHttpRequest, HttpResponse};
    use serde_json::json;

    const DISCOVERY: &str = include_str!("../tests/fixtures/discovery.json");
    const JWKS: &str = include_str!("../tests/fixtures/jwks.json");
    const USERINFO: &str = include_str!("../tests/fixtures/userinfo.json");
    const ID_TOKEN_VALID: &str = include_str!("../tests/fixtures/id_token_valid.txt");
    const ID_TOKEN_TAMPERED: &str = include_str!("../tests/fixtures/id_token_tampered.txt");
    const ID_TOKEN_EXPIRED: &str = include_str!("../tests/fixtures/id_token_expired.txt");
    const ID_TOKEN_WRONG_AUD: &str = include_str!("../tests/fixtures/id_token_wrong_aud.txt");
    const ID_TOKEN_WRONG_NONCE: &str = include_str!("../tests/fixtures/id_token_wrong_nonce.txt");

    const FIXTURE_ISSUER: &str = "https://idp.test";
    const FIXTURE_CLIENT: &str = "mailwoman-rp";
    const BOUND_NONCE: &str = "the-bound-nonce";

    /// Minimal `std::error::Error` for the mock client.
    #[derive(Debug)]
    struct FixtureError(String);
    impl std::fmt::Display for FixtureError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "fixture: {}", self.0)
        }
    }
    impl std::error::Error for FixtureError {}

    /// A mock HTTP client that serves the recorded fixtures by request path. The
    /// `/token` response embeds whichever ID token the test selected.
    struct FixtureClient {
        id_token: String,
    }

    impl<'c> AsyncHttpClient<'c> for FixtureClient {
        type Error = FixtureError;
        type Future = Pin<Box<dyn Future<Output = Result<HttpResponse, FixtureError>> + 'c>>;

        fn call(&'c self, request: OidcHttpRequest) -> Self::Future {
            let path = request.uri().path().to_string();
            let id_token = self.id_token.clone();
            Box::pin(async move {
                let body: String = match path.as_str() {
                    "/.well-known/openid-configuration" => DISCOVERY.to_string(),
                    "/jwks" => JWKS.to_string(),
                    "/userinfo" => USERINFO.to_string(),
                    "/token" => json!({
                        "access_token": "the-access-token",
                        "token_type": "Bearer",
                        "expires_in": 3600,
                        "id_token": id_token,
                    })
                    .to_string(),
                    other => return Err(FixtureError(format!("unexpected path {other}"))),
                };
                Response::builder()
                    .status(200)
                    .header("content-type", "application/json")
                    .body(body.into_bytes())
                    .map_err(|e| FixtureError(e.to_string()))
            })
        }
    }

    /// A no-op waker so we can drive the fixture futures (which never truly pend —
    /// every mock response is immediately ready) without a real async runtime.
    struct NoopWake;
    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    fn block_on<F: Future>(fut: F) -> F::Output {
        let mut fut = std::pin::pin!(fut);
        let waker = Arc::new(NoopWake).into();
        let mut cx = Context::from_waker(&waker);
        loop {
            if let Poll::Ready(v) = fut.as_mut().poll(&mut cx) {
                return v;
            }
        }
    }

    fn fixture_provider() -> OidcProvider {
        let backend = SsoBackend {
            id: "corp-oidc".into(),
            kind: SsoKind::Oidc,
            display_name: "Acme SSO".into(),
            scope: SsoScope::Deployment,
            enabled: true,
            config: SsoConfig::Oidc(OidcConfig {
                issuer_url: FIXTURE_ISSUER.into(),
                client_id: FIXTURE_CLIENT.into(),
                redirect_url: "https://mail.example/api/sso/corp-oidc/callback".into(),
                scopes: vec!["openid".into(), "email".into(), "profile".into()],
                ..Default::default()
            }),
            claim_map: ClaimMap::default(),
            secret: Some(b"top-secret".to_vec()),
        };
        OidcProvider::new(backend).unwrap()
    }

    fn callback(id_nonce: &str) -> SsoCallback {
        let mut params = std::collections::BTreeMap::new();
        params.insert("code".to_string(), "auth-code-xyz".to_string());
        params.insert("state".to_string(), "opaque-state".to_string());
        SsoCallback {
            params,
            pending: PendingState::Oidc {
                pkce_verifier: "the-pkce-verifier-value-1234567890abcdef".into(),
                nonce: id_nonce.into(),
                relay_state: Some("/inbox".into()),
            },
        }
    }

    fn complete_with(id_token: &str, nonce: &str) -> Result<SsoIdentity, SsoError> {
        let p = fixture_provider();
        let http = FixtureClient {
            id_token: id_token.trim().to_string(),
        };
        block_on(p.complete_impl(callback(nonce), &http))
    }

    #[test]
    fn begin_builds_pkce_auth_url_and_pending() {
        let p = fixture_provider();
        let http = FixtureClient {
            id_token: ID_TOKEN_VALID.trim().to_string(),
        };
        let out = block_on(p.begin_impl(Some("/calendar".into()), &http)).unwrap();

        assert!(out.url.starts_with("https://idp.test/authorize"));
        assert!(out.url.contains("code_challenge="));
        assert!(out.url.contains("code_challenge_method=S256"));
        assert!(out.url.contains(&format!("state={}", out.state_token)));
        assert!(out.url.contains("nonce="));
        assert!(out.url.contains("scope=openid"));
        assert_eq!(out.state_token.len(), 64, "256-bit hex correlator");
        match out.pending {
            PendingState::Oidc {
                pkce_verifier,
                nonce,
                relay_state,
            } => {
                assert!(!pkce_verifier.is_empty());
                assert!(!nonce.is_empty());
                assert_eq!(relay_state.as_deref(), Some("/calendar"));
            }
            _ => panic!("expected OIDC pending state"),
        }
        // Discovery during begin() caches the end_session endpoint for logout().
        let logout = p.logout("user-abc-123").expect("rp-initiated logout url");
        assert!(logout.url.starts_with("https://idp.test/logout"));
    }

    #[test]
    fn complete_valid_round_trip_yields_identity() {
        let id = complete_with(ID_TOKEN_VALID, BOUND_NONCE).expect("valid login");
        assert_eq!(id.subject, "user-abc-123");
        assert_eq!(id.email.as_deref(), Some("ada@corp.test"));
        // display_name + groups come from the userinfo enrichment.
        assert_eq!(id.display_name.as_deref(), Some("Ada Lovelace"));
        assert_eq!(id.groups, vec!["admins".to_string(), "staff".into()]);
        // Raw claim view carries scalars, never tokens.
        assert_eq!(
            id.claims.get("sub").map(String::as_str),
            Some("user-abc-123")
        );
        assert!(!id.claims.values().any(|v| v.contains("the-access-token")));
    }

    #[test]
    fn complete_rejects_tampered_signature() {
        let e = complete_with(ID_TOKEN_TAMPERED, BOUND_NONCE).unwrap_err();
        assert!(matches!(e, SsoError::SignatureInvalid(_)), "got {e:?}");
        assert_eq!(e.login_status(), 401);
    }

    #[test]
    fn complete_rejects_wrong_audience() {
        let e = complete_with(ID_TOKEN_WRONG_AUD, BOUND_NONCE).unwrap_err();
        assert!(matches!(e, SsoError::AudienceMismatch), "got {e:?}");
    }

    #[test]
    fn complete_rejects_wrong_nonce_as_replay() {
        // Token was minted with a different nonce than the pending state holds.
        let e = complete_with(ID_TOKEN_WRONG_NONCE, BOUND_NONCE).unwrap_err();
        assert!(matches!(e, SsoError::Replay), "got {e:?}");
    }

    #[test]
    fn complete_rejects_expired_token() {
        let e = complete_with(ID_TOKEN_EXPIRED, BOUND_NONCE).unwrap_err();
        assert!(matches!(e, SsoError::Expired), "got {e:?}");
    }

    #[test]
    fn complete_rejects_saml_pending_as_replay() {
        // A resolved SAML pending state on an OIDC provider ⇒ correlation failure.
        let p = fixture_provider();
        let http = FixtureClient {
            id_token: ID_TOKEN_VALID.trim().to_string(),
        };
        let cb = SsoCallback {
            params: std::collections::BTreeMap::new(),
            pending: PendingState::Saml {
                request_id: "_abc".into(),
                relay_state: None,
            },
        };
        let e = block_on(p.complete_impl(cb, &http)).unwrap_err();
        assert!(matches!(e, SsoError::Replay), "got {e:?}");
    }

    #[test]
    fn complete_rejects_missing_code() {
        let p = fixture_provider();
        let http = FixtureClient {
            id_token: ID_TOKEN_VALID.trim().to_string(),
        };
        let cb = SsoCallback {
            params: std::collections::BTreeMap::new(), // no `code`
            pending: PendingState::Oidc {
                pkce_verifier: "v".into(),
                nonce: BOUND_NONCE.into(),
                relay_state: None,
            },
        };
        let e = block_on(p.complete_impl(cb, &http)).unwrap_err();
        assert!(matches!(e, SsoError::TokenValidation(_)), "got {e:?}");
    }
}
