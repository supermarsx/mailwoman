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

/// The scope a consented authorization may carry: `requested`, cut down to
/// `ceiling` when the client has one ([`Scope::intersect`]), and never with
/// `unattended_send`.
///
/// Unattended send is an API-key privilege that an administrator countersigns per
/// key; the consent flow has no such second signature, so it cannot be granted
/// here whatever the request or the ceiling says.
///
/// A request for more than the ceiling is narrowed, not refused (RFC 6749 §3.3
/// lets the server ignore part of a requested scope; the token response always
/// states the scope that was issued).
pub fn consent_scope(requested: &Scope, ceiling: Option<&Scope>) -> Scope {
    let mut scope = match ceiling {
        Some(c) => requested.intersect(c),
        None => requested.clone(),
    };
    scope.unattended_send = false;
    scope
}

/// A stored OAuth token's scope as it is honoured: without `unattended_send`.
/// [`AuthServer::authorize_within`] never stores it; this covers rows written
/// before that was so.
pub(crate) fn token_scope(mut scope: Scope) -> Scope {
    scope.unattended_send = false;
    scope
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

    /// Handle a consented authorization request → mint a one-time auth code, for
    /// a client that has no scope ceiling. See [`Self::authorize_within`].
    pub async fn authorize(
        &self,
        req: &AuthorizeRequest,
        account_id: &str,
    ) -> Result<AuthorizeResponse, OAuthError> {
        self.authorize_within(req, account_id, None).await
    }

    /// Handle a consented authorization request → mint a one-time auth code.
    ///
    /// Enforces: `response_type=code`, **PKCE S256 present**, **resource present**,
    /// client registered + approved, `redirect_uri` registered.
    ///
    /// The code — and every token later issued from it — carries
    /// [`consent_scope`]`(&req.scope, ceiling)`: at most `ceiling`, and never
    /// `unattended_send`. `ceiling` is the most the client may be granted; the
    /// client registry row holds no scope, so the caller supplies it.
    pub async fn authorize_within(
        &self,
        req: &AuthorizeRequest,
        account_id: &str,
        ceiling: Option<&Scope>,
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
            scope: consent_scope(&req.scope, ceiling),
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
    /// Neither grant takes a scope parameter, so the pair can only carry what
    /// `src` does (less `unattended_send`, see [`token_scope`]).
    async fn issue_pair(&self, src: &OAuthToken) -> Result<TokenResponse, OAuthError> {
        let scope = token_scope(src.scope.clone());
        let now = Utc::now();
        let access = b64url(&random_bytes::<32>());
        let refresh = b64url(&random_bytes::<32>());

        let access_row = OAuthToken {
            token_hash: sha256_hex(&access),
            client_id: src.client_id.clone(),
            account_id: src.account_id.clone(),
            scope: scope.clone(),
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
            scope,
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
            scope: Some(token_scope(t.scope)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enforce::{NoopAudit, RequestContext};
    use crate::{InMemoryOAuthStore, ScopeSelector};

    /// Token rows written before the consent flow stopped storing the flag: an
    /// access and a refresh token whose stored scope has `unattended_send`.
    #[tokio::test]
    async fn a_stored_token_carrying_unattended_send_is_not_honoured_for_it() {
        let server = AuthServer::new(InMemoryOAuthStore::new());
        let mut scope = Scope::read_only("acct-9");
        scope.send = true;
        scope.accounts = ScopeSelector::All;
        scope.unattended_send = true;
        let row = |token: &str, kind: TokenKind| OAuthToken {
            token_hash: sha256_hex(token),
            client_id: "client-1".into(),
            account_id: "acct-9".into(),
            scope: scope.clone(),
            resource: None,
            kind,
            expires_at: (Utc::now() + Duration::hours(1)).to_rfc3339(),
            created_at: Utc::now().to_rfc3339(),
            revoked_at: None,
            pkce_challenge: None,
        };
        for (token, kind) in [
            ("old-access", TokenKind::Access),
            ("old-refresh", TokenKind::Refresh),
        ] {
            server.store().put_token(row(token, kind)).await.unwrap();
        }
        let mut expected = scope.clone();
        expected.unattended_send = false;

        let info = server.introspect("old-access").await.unwrap();
        assert!(info.active);
        assert_eq!(info.scope, Some(expected.clone()));

        let ctx = RequestContext {
            credential: "old-access",
            source_ip: None,
            resource: None,
        };
        let granted = server
            .require_scope(&ctx, &expected, &NoopAudit)
            .await
            .unwrap();
        assert_eq!(granted.scope, expected);
        assert!(matches!(
            server.require_scope(&ctx, &scope, &NoopAudit).await,
            Err(OAuthError::InvalidScope)
        ));

        let next = server
            .token(&TokenRequest::RefreshToken {
                refresh_token: "old-refresh".into(),
                client_id: "client-1".into(),
                resource: None,
            })
            .await
            .unwrap();
        assert_eq!(next.scope, expected);
        let info = server.introspect(&next.access_token).await.unwrap();
        assert_eq!(info.scope, Some(expected));
    }
}
