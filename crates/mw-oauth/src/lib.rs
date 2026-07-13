#![forbid(unsafe_code)]
//! Scoped API keys + OAuth 2.1 authorization server for Mailwoman V6 (SPEC
//! §20.1, plan §2.3).
//!
//! **Contract (§2.3):**
//! - [`Scope`] — the typed capability set: `{read,send,delete} × {account,folder}
//!   × {mail,pim}` + IP allowlist + expiry + rate-limit + `mcp_tools` +
//!   `unattended_send`. [`Scope::allows`] is the grant/deny matrix (no escalation).
//! - API key wire format `mwk_<prefix>.<secret>` (256-bit secret); stored
//!   Argon2id-hashed; shown once; prefix-indexed for lookup ([`mint_api_key`],
//!   [`verify_api_key`]).
//! - OAuth 2.1 [`AuthServer`]: `/oauth/authorize` (code + **mandatory PKCE S256** +
//!   **mandatory RFC 8707 resource**), `/oauth/token` (auth-code + refresh),
//!   `/oauth/introspect`, `/oauth/revoke`; admin-approved [`OAuthClient`] registry.
//! - Enforcement core [`AuthServer::require_scope`] (mounted behind axum middleware
//!   by e11): resolves a key/token → [`Scope`], checks expiry + IP allowlist +
//!   rate-limit + resource binding + capability, and emits an [`AuditEvent`].
//!
//! Opaque tokens (SHA-256 hashed in the store) are the default — no JWT dependency.
//! MCP keys ARE API keys (`mcp_tools` grants). Persistence is abstracted behind
//! [`OAuthStore`] (e11 backs it with the `mw-store` 0007 tables); [`InMemoryOAuthStore`]
//! is the in-crate reference impl.

mod enforce;
mod keys;
mod oauth;
mod pkce;
mod store;
mod util;

pub use enforce::{
    AuditEvent, AuditSink, CollectingAudit, CredentialKind, Granted, NoopAudit, RequestContext,
};
pub use oauth::{
    AuthServer, AuthServerConfig, AuthorizeRequest, AuthorizeResponse, Introspection, TokenRequest,
    TokenResponse,
};
pub use pkce::challenge_s256;
pub use store::{InMemoryOAuthStore, OAuthStore};

use serde::{Deserialize, Serialize};

/// Selects a set of accounts or folders a scope applies to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScopeSelector {
    /// `*` — every account/folder.
    All,
    /// An explicit id subset.
    Subset(Vec<String>),
}

/// The typed capability set carried by an API key or OAuth token (§2.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    pub read: bool,
    pub send: bool,
    pub delete: bool,
    /// Which accounts this scope covers.
    pub accounts: ScopeSelector,
    /// Which folders this scope covers (within `accounts`).
    pub folders: ScopeSelector,
    /// Mail surface granted.
    pub mail: bool,
    /// PIM (calendar/tasks/notes/contacts) surface granted.
    pub pim: bool,
    /// CIDR/IP allowlist (empty = any source IP).
    pub ip_allowlist: Vec<String>,
    /// RFC 3339 expiry (None = no expiry).
    pub expires_at: Option<String>,
    /// Per-key request rate limit (requests/min; None = unlimited).
    pub rate_limit: Option<u32>,
    /// MCP tool ids grantable to this key (see `mw-mcp`).
    pub mcp_tools: Vec<String>,
    /// Whether `mail.send` may bypass the Outbox human-in-the-loop gate. Requires
    /// the key's admin-countersign flag; see `mw-mcp` send gating (§2.4).
    pub unattended_send: bool,
}

impl Scope {
    /// A read-only, single-account, mail-only scope (the safest default).
    pub fn read_only(account_id: impl Into<String>) -> Self {
        Self {
            read: true,
            send: false,
            delete: false,
            accounts: ScopeSelector::Subset(vec![account_id.into()]),
            folders: ScopeSelector::All,
            mail: true,
            pim: false,
            ip_allowlist: Vec::new(),
            expires_at: None,
            rate_limit: None,
            mcp_tools: Vec::new(),
            unattended_send: false,
        }
    }

    /// Whether this (granted) scope authorizes `required` — the grant/deny matrix.
    ///
    /// A required capability is authorized only if this scope explicitly grants it;
    /// a narrower key can never escalate. `ip_allowlist`/`expires_at`/`rate_limit`
    /// are per-key constraints checked by [`AuthServer::require_scope`], not part of
    /// the capability comparison here.
    pub fn allows(&self, required: &Scope) -> bool {
        // Verb grants: every requested verb must be held.
        if required.read && !self.read {
            return false;
        }
        if required.send && !self.send {
            return false;
        }
        if required.delete && !self.delete {
            return false;
        }
        // Surface grants.
        if required.mail && !self.mail {
            return false;
        }
        if required.pim && !self.pim {
            return false;
        }
        // Account / folder coverage.
        if !selector_covers(&self.accounts, &required.accounts) {
            return false;
        }
        if !selector_covers(&self.folders, &required.folders) {
            return false;
        }
        // MCP tool grants: every requested tool must be granted.
        if !required
            .mcp_tools
            .iter()
            .all(|t| self.mcp_tools.contains(t))
        {
            return false;
        }
        // Unattended send is a strictly additional privilege.
        if required.unattended_send && !self.unattended_send {
            return false;
        }
        true
    }
}

/// Whether `granted` covers every id in `required`. `All` covers anything; a
/// concrete subset can never cover `All` (that would be an escalation).
fn selector_covers(granted: &ScopeSelector, required: &ScopeSelector) -> bool {
    match (granted, required) {
        (ScopeSelector::All, _) => true,
        (ScopeSelector::Subset(_), ScopeSelector::All) => false,
        (ScopeSelector::Subset(g), ScopeSelector::Subset(r)) => r.iter().all(|id| g.contains(id)),
    }
}

/// An opaque scoped API key. The secret is shown once and only the Argon2id hash
/// is stored (`api_keys` table, 0007).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Prefix used for O(1) row lookup (`mwk_<prefix>`).
    pub prefix: String,
    /// Argon2id hash of the secret half.
    pub hash: String,
    pub account_id: String,
    pub scope: Scope,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub revoked_at: Option<String>,
}

/// The plaintext key returned exactly once at mint time (`mwk_<prefix>.<secret>`).
#[derive(Debug, Clone)]
pub struct MintedApiKey {
    pub display_token: String,
    pub record: ApiKey,
}

/// Mint a fresh `mwk_<prefix>.<secret>` API key for `account_id` under `scope`.
///
/// Generates a 256-bit secret, Argon2id-hashes it, and returns the shown-once
/// display token alongside the storable [`ApiKey`] record (which holds only the
/// hash). Persist `record` via an [`OAuthStore`]; surface `display_token` once.
pub fn mint_api_key(account_id: &str, scope: Scope) -> MintedApiKey {
    keys::mint(account_id, scope)
}

/// Verify a presented `mwk_...` token against a stored [`ApiKey`].
///
/// Constant-time on both the prefix and (via Argon2id) the secret; rejects revoked
/// keys and malformed tokens.
pub fn verify_api_key(presented: &str, stored: &ApiKey) -> bool {
    keys::verify(presented, stored)
}

/// OAuth 2.1 token kinds persisted in `oauth_tokens` (0007).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TokenKind {
    AuthCode,
    Access,
    Refresh,
}

/// An admin-approved OAuth 2.1 client (`oauth_clients` table, 0007).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClient {
    pub client_id: String,
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub approved_by: String,
    pub created_at: String,
}

/// An issued OAuth 2.1 token (`oauth_tokens` table, 0007). Only the hash is
/// stored; `resource` carries the RFC 8707 resource indicator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub token_hash: String,
    pub client_id: String,
    pub account_id: String,
    pub scope: Scope,
    pub resource: Option<String>,
    pub kind: TokenKind,
    pub expires_at: String,
    pub created_at: String,
    pub revoked_at: Option<String>,
    /// PKCE S256 challenge (auth-code grants).
    pub pkce_challenge: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("invalid client")]
    InvalidClient,
    #[error("invalid grant")]
    InvalidGrant,
    #[error("invalid scope")]
    InvalidScope,
    #[error("pkce verification failed")]
    PkceFailed,
    #[error("token expired or revoked")]
    Expired,
    #[error("source ip not in allowlist")]
    IpDenied,
    #[error("rate limit exceeded")]
    RateLimited,
    #[error("store error: {0}")]
    Store(String),
}

/// Verify a PKCE `code_verifier` against a stored S256 `challenge`
/// (`BASE64URL-NOPAD(SHA256(verifier)) == challenge`), in constant time.
pub fn verify_pkce_s256(verifier: &str, challenge: &str) -> bool {
    pkce::verify_s256(verifier, challenge)
}
