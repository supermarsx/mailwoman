#![forbid(unsafe_code)]
// SCAFFOLD (t6-e0): stub crate — the frozen §2.3 API names exist so e4 (MCP) and
// e11 (mount) compile against `Scope`; e3 owns the real implementation.
#![allow(dead_code, clippy::unused_async)]
//! Scoped API keys + OAuth 2.1 authorization server for Mailwoman V6 (SPEC
//! §20.1, plan §2.3).
//!
//! **Frozen contract (§2.3):**
//! - [`Scope`] — the typed capability set: `{read,send,delete} × {account,folder}
//!   × {mail,pim}` + IP allowlist + expiry + rate-limit + `mcp_tools` +
//!   `unattended_send`.
//! - API key wire format `mwk_<prefix>.<secret>`; stored Argon2id-hashed; shown
//!   once; prefix-indexed for lookup.
//! - OAuth 2.1: `/oauth/authorize` (code + PKCE-S256 + resource), `/oauth/token`,
//!   `/oauth/introspect`, `/oauth/revoke`; admin-approved [`OAuthClient`] registry.
//! - Enforcement middleware `require_scope(scope)` lives in `mw-server` (mounted
//!   by e11): resolves a key/token → [`Scope`], checks IP + rate-limit + expiry,
//!   and emits an audit row.
//!
//! Opaque tokens (hashed in the store) are the default — no JWT dependency unless
//! resource-indicator interop forces it. MCP keys ARE API keys (`mcp:*` scopes).
//!
//! e3 fills the bodies (currently `unimplemented!()`) and persists via the
//! `mw-store` 0007 tables.

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

    /// Whether this scope authorizes `required` (STUB: e3 fills the real matrix).
    pub fn allows(&self, _required: &Scope) -> bool {
        unimplemented!("mw-oauth::Scope::allows — filled by t6-e3")
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
/// STUB: e3 generates 32 random bytes, Argon2id-hashes the secret, and returns
/// the shown-once token alongside the storable record.
pub fn mint_api_key(_account_id: &str, _scope: Scope) -> MintedApiKey {
    unimplemented!("mw-oauth::mint_api_key — filled by t6-e3")
}

/// Verify a presented `mwk_...` token against a stored [`ApiKey`] hash.
pub fn verify_api_key(_presented: &str, _stored: &ApiKey) -> bool {
    unimplemented!("mw-oauth::verify_api_key — filled by t6-e3")
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

/// Verify a PKCE `code_verifier` against a stored S256 `challenge`.
/// STUB: e3 implements `BASE64URL(SHA256(verifier)) == challenge`.
pub fn verify_pkce_s256(_verifier: &str, _challenge: &str) -> bool {
    unimplemented!("mw-oauth::verify_pkce_s256 — filled by t6-e3")
}
