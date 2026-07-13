//! OAuth 2.1 Authorization Server (SPEC §20.1, plan §2.3).
//!
//! Authorization-code grant with **mandatory PKCE S256** and **mandatory RFC 8707
//! resource indicators**, opaque access + refresh tokens (SHA-256 hashed at rest),
//! token introspection (RFC 7662) and revocation (RFC 7009), over an
//! admin-approved client registry.
//!
//! This module is transport-agnostic: it exposes typed request/response structs;
//! `mw-server` (e11) maps them onto the `/oauth/*` HTTP endpoints.

use chrono::{DateTime, Duration, Utc};

use crate::enforce::RateLimiter;
use crate::store::OAuthStore;
use crate::util::{b64url, random_bytes, sha256_hex};
use crate::{OAuthError, OAuthToken, Scope, TokenKind};

/// Lifetimes for issued artifacts.
#[derive(Debug, Clone)]
pub struct AuthServerConfig {
    pub auth_code_ttl: Duration,
    pub access_ttl: Duration,
    pub refresh_ttl: Duration,
}

impl Default for AuthServerConfig {
    fn default() -> Self {
        Self {
            auth_code_ttl: Duration::minutes(10),
            access_ttl: Duration::hours(1),
            refresh_ttl: Duration::days(30),
        }
    }
}

/// The OAuth 2.1 AS + the API-key/OAuth enforcement core, over a pluggable store.
pub struct AuthServer<S: OAuthStore> {
    pub(crate) store: S,
    pub(crate) config: AuthServerConfig,
    pub(crate) rate: RateLimiter,
}

/// `/oauth/authorize` request (post-consent). `account_id` (the resource owner) is
/// supplied separately by the consent handler, not by the untrusted client.
#[derive(Debug, Clone)]
pub struct AuthorizeRequest {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: Scope,
    pub state: Option<String>,
    /// PKCE challenge (base64url SHA-256 of the verifier).
    pub code_challenge: String,
    /// Must be `S256` — `plain` is rejected.
    pub code_challenge_method: String,
    /// RFC 8707 resource indicator (mandatory).
    pub resource: String,
}

/// Result of a successful authorization: the code to hand back via `redirect_uri`.
#[derive(Debug, Clone)]
pub struct AuthorizeResponse {
    pub code: String,
    pub state: Option<String>,
    pub redirect_uri: String,
}

/// `/oauth/token` request — the two supported grants.
#[derive(Debug, Clone)]
pub enum TokenRequest {
    AuthorizationCode {
        code: String,
        redirect_uri: String,
        client_id: String,
        code_verifier: String,
        /// RFC 8707 resource — must match the authorization request.
        resource: String,
    },
    RefreshToken {
        refresh_token: String,
        client_id: String,
        /// Optional narrowing; if present must equal the token's bound resource.
        resource: Option<String>,
    },
}

/// `/oauth/token` success response.
#[derive(Debug, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: i64,
    pub scope: Scope,
    pub resource: Option<String>,
}

/// `/oauth/introspect` (RFC 7662) response.
#[derive(Debug, Clone, Default)]
pub struct Introspection {
    pub active: bool,
    pub scope: Option<Scope>,
    pub resource: Option<String>,
    pub client_id: Option<String>,
    pub account_id: Option<String>,
    pub kind: Option<TokenKind>,
    pub expires_at: Option<String>,
}

/// True if an RFC 3339 timestamp is in the past (malformed → treated as expired).
pub(crate) fn is_expired(rfc3339: &str) -> bool {
    match DateTime::parse_from_rfc3339(rfc3339) {
        Ok(dt) => dt.with_timezone(&Utc) <= Utc::now(),
        Err(_) => true,
    }
}

impl<S: OAuthStore> AuthServer<S> {
    pub fn new(store: S) -> Self {
        Self::with_config(store, AuthServerConfig::default())
    }

    pub fn with_config(store: S, config: AuthServerConfig) -> Self {
        Self {
            store,
            config,
            rate: RateLimiter::new(),
        }
    }

    pub fn store(&self) -> &S {
        &self.store
    }

    /// Handle a consented authorization request → mint a one-time auth code.
    ///
    /// Enforces: `response_type=code`, **PKCE S256 present**, **resource present**,
    /// client registered + approved, `redirect_uri` registered.
    pub async fn authorize(
        &self,
        req: &AuthorizeRequest,
        account_id: &str,
    ) -> Result<AuthorizeResponse, OAuthError> {
        if req.response_type != "code" {
            return Err(OAuthError::InvalidGrant);
        }
        // Mandatory PKCE S256 — no downgrade to `plain`, no omission.
        if req.code_challenge_method != "S256" || req.code_challenge.is_empty() {
            return Err(OAuthError::PkceFailed);
        }
        // Mandatory RFC 8707 resource indicator.
        if req.resource.is_empty() {
            return Err(OAuthError::InvalidScope);
        }
        let client = self
            .store
            .get_client(&req.client_id)
            .await?
            .ok_or(OAuthError::InvalidClient)?;
        if !client.redirect_uris.iter().any(|u| u == &req.redirect_uri) {
            return Err(OAuthError::InvalidClient);
        }

        let code = b64url(&random_bytes::<32>());
        let now = Utc::now();
        let token = OAuthToken {
            token_hash: sha256_hex(&code),
            client_id: req.client_id.clone(),
            account_id: account_id.to_string(),
            scope: req.scope.clone(),
            resource: Some(req.resource.clone()),
            kind: TokenKind::AuthCode,
            expires_at: (now + self.config.auth_code_ttl).to_rfc3339(),
            created_at: now.to_rfc3339(),
            revoked_at: None,
            pkce_challenge: Some(req.code_challenge.clone()),
        };
        self.store.put_token(token).await?;
        Ok(AuthorizeResponse {
            code,
            state: req.state.clone(),
            redirect_uri: req.redirect_uri.clone(),
        })
    }

    /// Handle a token request (authorization-code or refresh grant).
    pub async fn token(&self, req: &TokenRequest) -> Result<TokenResponse, OAuthError> {
        match req {
            TokenRequest::AuthorizationCode {
                code,
                client_id,
                code_verifier,
                resource,
                ..
            } => {
                self.grant_auth_code(code, client_id, code_verifier, resource)
                    .await
            }
            TokenRequest::RefreshToken {
                refresh_token,
                client_id,
                resource,
            } => {
                self.grant_refresh(refresh_token, client_id, resource.as_deref())
                    .await
            }
        }
    }

    async fn grant_auth_code(
        &self,
        code: &str,
        client_id: &str,
        code_verifier: &str,
        resource: &str,
    ) -> Result<TokenResponse, OAuthError> {
        let code_hash = sha256_hex(code);
        let auth = self
            .store
            .get_token(&code_hash)
            .await?
            .ok_or(OAuthError::InvalidGrant)?;

        if auth.kind != TokenKind::AuthCode
            || auth.revoked_at.is_some()
            || is_expired(&auth.expires_at)
        {
            return Err(OAuthError::InvalidGrant);
        }
        if auth.client_id != client_id {
            return Err(OAuthError::InvalidClient);
        }
        // RFC 8707 audience binding: the token request's resource must match the
        // one bound at authorization time.
        if auth.resource.as_deref() != Some(resource) {
            return Err(OAuthError::InvalidScope);
        }
        // Mandatory PKCE S256 verification.
        let challenge = auth
            .pkce_challenge
            .as_deref()
            .ok_or(OAuthError::PkceFailed)?;
        if !crate::pkce::verify_s256(code_verifier, challenge) {
            return Err(OAuthError::PkceFailed);
        }
        // Auth codes are single-use — burn it before issuing tokens.
        self.store.revoke_token(&code_hash).await?;

        self.issue_pair(&auth).await
    }

    async fn grant_refresh(
        &self,
        refresh_token: &str,
        client_id: &str,
        resource: Option<&str>,
    ) -> Result<TokenResponse, OAuthError> {
        let refresh_hash = sha256_hex(refresh_token);
        let refresh = self
            .store
            .get_token(&refresh_hash)
            .await?
            .ok_or(OAuthError::InvalidGrant)?;

        if refresh.kind != TokenKind::Refresh
            || refresh.revoked_at.is_some()
            || is_expired(&refresh.expires_at)
        {
            return Err(OAuthError::InvalidGrant);
        }
        if refresh.client_id != client_id {
            return Err(OAuthError::InvalidClient);
        }
        // A narrowing `resource` may not widen or retarget the audience.
        if let Some(r) = resource
            && refresh.resource.as_deref() != Some(r)
        {
            return Err(OAuthError::InvalidScope);
        }
        // Rotate: burn the presented refresh token, mint a fresh pair.
        self.store.revoke_token(&refresh_hash).await?;
        self.issue_pair(&refresh).await
    }

    /// Mint an access + refresh pair inheriting `src`'s scope/resource/identity.
    async fn issue_pair(&self, src: &OAuthToken) -> Result<TokenResponse, OAuthError> {
        let now = Utc::now();
        let access = b64url(&random_bytes::<32>());
        let refresh = b64url(&random_bytes::<32>());

        let access_row = OAuthToken {
            token_hash: sha256_hex(&access),
            client_id: src.client_id.clone(),
            account_id: src.account_id.clone(),
            scope: src.scope.clone(),
            resource: src.resource.clone(),
            kind: TokenKind::Access,
            expires_at: (now + self.config.access_ttl).to_rfc3339(),
            created_at: now.to_rfc3339(),
            revoked_at: None,
            pkce_challenge: None,
        };
        let refresh_row = OAuthToken {
            token_hash: sha256_hex(&refresh),
            kind: TokenKind::Refresh,
            expires_at: (now + self.config.refresh_ttl).to_rfc3339(),
            ..access_row.clone()
        };
        self.store.put_token(access_row.clone()).await?;
        self.store.put_token(refresh_row).await?;

        Ok(TokenResponse {
            access_token: access,
            refresh_token: Some(refresh),
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_ttl.num_seconds(),
            scope: src.scope.clone(),
            resource: src.resource.clone(),
        })
    }

    /// Introspect an access/refresh token (RFC 7662). Auth codes never introspect
    /// as active. Unknown/expired/revoked tokens return `active:false`.
    pub async fn introspect(&self, token: &str) -> Result<Introspection, OAuthError> {
        let hash = sha256_hex(token);
        let Some(t) = self.store.get_token(&hash).await? else {
            return Ok(Introspection::default());
        };
        let active = t.revoked_at.is_none()
            && !is_expired(&t.expires_at)
            && matches!(t.kind, TokenKind::Access | TokenKind::Refresh);
        if !active {
            return Ok(Introspection::default());
        }
        Ok(Introspection {
            active: true,
            scope: Some(t.scope),
            resource: t.resource,
            client_id: Some(t.client_id),
            account_id: Some(t.account_id),
            kind: Some(t.kind),
            expires_at: Some(t.expires_at),
        })
    }

    /// Revoke a token by value (RFC 7009). Idempotent — revoking an unknown token
    /// succeeds silently, as the RFC requires.
    pub async fn revoke(&self, token: &str) -> Result<(), OAuthError> {
        self.store.revoke_token(&sha256_hex(token)).await
    }
}
