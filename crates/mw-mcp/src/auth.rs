//! Per-call authorization seam.
//!
//! Each tool call resolves its caller's credential to a [`mw_oauth::Scope`] and
//! checks it covers the tool's required fragment. That is `mw-oauth`'s job; this
//! [`Authorizer`] trait is the thin seam so `mw-mcp` can be tested with a
//! deterministic mock while e11 wires the real [`OAuthAuthorizer`].
//!
//! **Enforcement split.** [`OAuthAuthorizer`] performs the parts of enforcement
//! that are needed *per tool* — credential verification (`mw_oauth::verify_api_key`
//! / `AuthServer::introspect`), token/key expiry, and the per-tool capability check
//! (`Scope::allows`) — and emits one audit event per attempt. Source-IP allowlist
//! and per-key rate-limit remain the job of `mw_oauth::AuthServer::require_scope`
//! mounted as `mw-server` middleware **in front of** `/mcp` by e11 (its future is
//! not `Send` because it borrows a `&dyn AuditSink` across an await, so it cannot
//! live inside this axum-served, `Send`-bound path; the middleware layer is the
//! right home for it anyway).
//!
//! [`AuthorizedCall::admin_countersigned`] carries the second half of the
//! send-gate. `mw_oauth::Scope` has no countersign field, so the [`Authorizer`]
//! resolves it: e11 reads it from the `api_keys` row; [`OAuthAuthorizer`] takes a
//! resolver closure keyed on the presented credential (OAuth tokens are never
//! countersigned).

use std::net::IpAddr;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mw_oauth::{AuditEvent, AuditSink, AuthServer, OAuthStore, Scope, verify_api_key};

use crate::McpError;

/// Wire scheme of an `mw-oauth` API key (`mwk_<prefix>.<secret>`).
const KEY_SCHEME: &str = "mwk_";

/// The inbound credential + request context for one tool call.
#[derive(Debug, Clone)]
pub struct Credential<'a> {
    /// `mwk_<prefix>.<secret>` API key or an OAuth 2.1 access token.
    pub token: &'a str,
    /// Source IP for allowlist checks (`None` when the transport did not supply it).
    pub source_ip: Option<IpAddr>,
    /// RFC 8707 resource indicator this request targets.
    pub resource: Option<&'a str>,
}

/// A successfully authorized call: the effective scope + the countersign bit.
#[derive(Debug, Clone)]
pub struct AuthorizedCall {
    pub account_id: String,
    pub scope: Scope,
    /// Whether the key bears the admin countersignature for unattended send.
    pub admin_countersigned: bool,
}

/// Resolves + authorizes a tool call against its required scope.
#[async_trait]
pub trait Authorizer: Send + Sync {
    /// Return the authorized call on success, or [`McpError::ScopeDenied`] if the
    /// credential is invalid, expired, or its scope does not cover `required`.
    async fn authorize(
        &self,
        cred: &Credential<'_>,
        required: &Scope,
    ) -> Result<AuthorizedCall, McpError>;
}

/// Resolver for a key's admin-countersign flag, keyed on the presented credential
/// token. e11 parses the key prefix and reads the `api_keys` row; tests supply a
/// closure. OAuth access tokens are never countersigned.
pub type CountersignResolver = Arc<dyn Fn(&str) -> bool + Send + Sync>;

/// The production [`Authorizer`], over an `mw_oauth::AuthServer` + audit sink.
pub struct OAuthAuthorizer<S: OAuthStore, A: AuditSink + Send + Sync> {
    server: Arc<AuthServer<S>>,
    audit: Arc<A>,
    countersign: CountersignResolver,
}

impl<S: OAuthStore, A: AuditSink + Send + Sync> OAuthAuthorizer<S, A> {
    /// Build over an `AuthServer`, an audit sink, and a countersign resolver.
    pub fn new(
        server: Arc<AuthServer<S>>,
        audit: Arc<A>,
        countersign: CountersignResolver,
    ) -> Self {
        Self {
            server,
            audit,
            countersign,
        }
    }

    /// Convenience: no key is countersigned (unattended send always → Outbox/403).
    pub fn without_countersign(server: Arc<AuthServer<S>>, audit: Arc<A>) -> Self {
        Self::new(server, audit, Arc::new(|_| false))
    }

    fn emit(&self, actor: &str, actor_kind: &'static str, allowed: bool, reason: Option<String>) {
        self.audit.emit(&AuditEvent {
            actor: actor.to_string(),
            actor_kind,
            action: "mcp/tools_call".to_string(),
            allowed,
            reason,
            ip: None,
            ts: Utc::now().to_rfc3339(),
        });
    }
}

/// The public prefix of an `mwk_` key, for lookup + countersign resolution.
fn key_prefix(token: &str) -> Option<&str> {
    token.strip_prefix(KEY_SCHEME)?.split('.').next()
}

/// Whether an RFC 3339 timestamp is in the past (malformed → treated as expired).
fn is_expired(rfc3339: &str) -> bool {
    match DateTime::parse_from_rfc3339(rfc3339) {
        Ok(dt) => dt.with_timezone(&Utc) <= Utc::now(),
        Err(_) => true,
    }
}

#[async_trait]
impl<S: OAuthStore, A: AuditSink + Send + Sync> Authorizer for OAuthAuthorizer<S, A> {
    async fn authorize(
        &self,
        cred: &Credential<'_>,
        required: &Scope,
    ) -> Result<AuthorizedCall, McpError> {
        // 1. Resolve the credential to (actor, account, scope, bound-resource) via
        //    mw-oauth. `token_resource` is the RFC 8707 audience the token was issued
        //    for (OAuth tokens only; API keys are not resource-bound, so `None`).
        let (actor, account_id, scope, token_resource) = if cred.token.starts_with(KEY_SCHEME) {
            let Some(prefix) = key_prefix(cred.token) else {
                self.emit("unknown", "api-key", false, Some("malformed key".into()));
                return Err(McpError::ScopeDenied);
            };
            let key = match self
                .server
                .store()
                .get_api_key(prefix)
                .await
                .map_err(|_| McpError::ScopeDenied)?
            {
                Some(k) => k,
                None => {
                    self.emit(prefix, "api-key", false, Some("unknown key".into()));
                    return Err(McpError::ScopeDenied);
                }
            };
            if !verify_api_key(cred.token, &key) {
                self.emit(prefix, "api-key", false, Some("invalid credential".into()));
                return Err(McpError::ScopeDenied);
            }
            (key.prefix.clone(), key.account_id, key.scope, None)
        } else {
            // OAuth access token — introspection validates active/kind/expiry.
            let intro = self
                .server
                .introspect(cred.token)
                .await
                .map_err(|_| McpError::ScopeDenied)?;
            if !intro.active {
                self.emit(
                    "unknown",
                    "oauth-token",
                    false,
                    Some("inactive token".into()),
                );
                return Err(McpError::ScopeDenied);
            }
            (
                intro.client_id.clone().unwrap_or_else(|| "unknown".into()),
                intro.account_id.unwrap_or_default(),
                intro.scope.ok_or(McpError::ScopeDenied)?,
                intro.resource,
            )
        };
        let actor_kind: &'static str = if cred.token.starts_with(KEY_SCHEME) {
            "api-key"
        } else {
            "oauth-token"
        };

        // 2. Expiry (API-key scope expiry; OAuth token expiry already covered above).
        if let Some(exp) = &scope.expires_at
            && is_expired(exp)
        {
            self.emit(&actor, actor_kind, false, Some("expired".into()));
            return Err(McpError::ScopeDenied);
        }

        // 3. RFC 8707 resource-indicator (audience) binding. `cred.resource` is this
        //    MCP endpoint's canonical resource identifier, resolved at the `/mcp`
        //    mount. Audience enforcement is ON BY DEFAULT: `MW_MCP_RESOURCE` overrides,
        //    but when it is unset the resource is derived from the deployment's
        //    configured public origin, so `cred.resource` is normally `Some`. A token
        //    bound to a DIFFERENT resource was issued for another audience and must be
        //    rejected here — a wrong-audience token never reaches a tool. API keys
        //    carry no resource binding and so are exempt (consistent with
        //    `mw_oauth::require_scope`). Enforcement is off only when neither an
        //    override nor a public origin is configured (`cred.resource` is `None`).
        if let (Some(bound), Some(want)) = (&token_resource, cred.resource)
            && bound != want
        {
            self.emit(&actor, actor_kind, false, Some("audience mismatch".into()));
            return Err(McpError::ScopeDenied);
        }

        // 4. Per-tool capability — no scope escalation.
        if !scope.allows(required) {
            self.emit(&actor, actor_kind, false, Some("scope denied".into()));
            return Err(McpError::ScopeDenied);
        }

        self.emit(&actor, actor_kind, true, None);
        let admin_countersigned = (self.countersign)(cred.token);
        Ok(AuthorizedCall {
            account_id,
            scope,
            admin_countersigned,
        })
    }
}
