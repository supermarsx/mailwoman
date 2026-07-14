#![forbid(unsafe_code)]
// SCAFFOLD (t7-e0): frozen §2.2 types + method shapes as INERT stubs; the ldap3
// (rustls) queries, multi-directory priority merge, and GAL cache are filled by e2.
#![allow(dead_code)]
//! `mw-directory` — LDAP/GAL directory (plan §2.2, SPEC §13). **Read-only at 1.0.**
//!
//! GAL search over every recipient field, distribution-group read + expand-before-
//! send, **S/MIME cert lookup** (feeds `mw-crypto`'s cert path, §8.2), photo
//! attributes, paged search, StartTLS/LDAPS, **multiple directories with priority
//! order**, and an offline GAL cache (via `mw-cache::CacheClass::GalDirectory`).
//! LDAP-bind **login** (§18.3) reuses this crate's connection layer.
//!
//! `ldap3` is configured **rustls-only** (`default-features=false,
//! features=["tls-rustls"]`) so **no openssl** enters the tree (deny.toml ban).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Errors surfaced by directory operations (plan §2.2).
#[derive(Debug, thiserror::Error)]
pub enum DirectoryError {
    #[error("ldap protocol error: {0}")]
    Protocol(String),
    #[error("bind/auth failed: {0}")]
    Auth(String),
    #[error("transport/TLS error: {0}")]
    Transport(String),
    #[error("no directory configured")]
    NotConfigured,
}

pub type Result<T> = std::result::Result<T, DirectoryError>;

/// DER-encoded bytes (an X.509 cert / photo blob).
pub type Der = Vec<u8>;

/// TLS mode for an LDAP endpoint (plan §2.2). rustls throughout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LdapTls {
    /// Plain LDAP (no TLS) — dev/on-prem only.
    None,
    /// StartTLS on the LDAP port.
    StartTls,
    /// Implicit LDAPS.
    Ldaps,
}

/// Attribute-name mapping so a deployment's schema maps onto GAL fields.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AttrMap {
    pub display_name: Option<String>,
    pub mail: Option<String>,
    pub member: Option<String>,
    pub user_cert: Option<String>,
    pub photo: Option<String>,
}

/// One LDAP endpoint in a priority-ordered directory list (plan §2.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LdapEndpoint {
    pub url: String,
    pub base_dn: String,
    pub bind_dn: Option<String>,
    pub tls: LdapTls,
    /// Lower = queried first; results merge in priority order.
    pub priority: i32,
    #[serde(default)]
    pub attr_map: AttrMap,
}

/// Directory config: an ordered set of endpoints merged by priority (plan §2.2).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryConfig {
    pub endpoints: Vec<LdapEndpoint>,
}

/// A resolved GAL entry (plan §2.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GalEntry {
    pub dn: String,
    pub display_name: String,
    pub mail: String,
    /// Whether this entry is a distribution group (expandable).
    #[serde(default)]
    pub is_group: bool,
}

/// The outcome of an LDAP-bind authentication (plan §2.2/§18.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindOutcome {
    Ok { dn: String },
    Denied,
}

/// The read-only directory seam (plan §2.2). e2 backs this with `ldap3` over the
/// priority-ordered `DirectoryConfig`, caching via `mw-cache::GalDirectory`.
#[async_trait]
pub trait DirectorySource: Send + Sync {
    /// GAL search across every recipient field; `page` is a 0-based page index.
    async fn search_gal(&self, query: &str, page: u32) -> Result<Vec<GalEntry>>;
    /// Expand a distribution group DN to its members (recursively upstream).
    async fn expand_group(&self, dn: &str) -> Result<Vec<GalEntry>>;
    /// S/MIME certificate lookup for a recipient (feeds mw-crypto §8.2).
    async fn lookup_cert(&self, email: &str) -> Result<Vec<Der>>;
    /// Photo attribute for a recipient.
    async fn lookup_photo(&self, email: &str) -> Result<Option<Der>>;
    /// LDAP-bind authentication (§18.3 login backend).
    async fn bind_auth(&self, user: &str, pass: &str) -> Result<BindOutcome>;
}

/// The concrete multi-directory client (plan §2.2). Inert until e2 fills it.
pub struct Directory {
    config: DirectoryConfig,
}

impl Directory {
    /// Build a directory over a priority-ordered config.
    #[must_use]
    pub fn new(config: DirectoryConfig) -> Self {
        Self { config }
    }

    /// The configured endpoints (priority order).
    #[must_use]
    pub fn config(&self) -> &DirectoryConfig {
        &self.config
    }
}

#[async_trait]
impl DirectorySource for Directory {
    async fn search_gal(&self, _query: &str, _page: u32) -> Result<Vec<GalEntry>> {
        Err(DirectoryError::NotConfigured)
    }
    async fn expand_group(&self, _dn: &str) -> Result<Vec<GalEntry>> {
        Err(DirectoryError::NotConfigured)
    }
    async fn lookup_cert(&self, _email: &str) -> Result<Vec<Der>> {
        Err(DirectoryError::NotConfigured)
    }
    async fn lookup_photo(&self, _email: &str) -> Result<Option<Der>> {
        Err(DirectoryError::NotConfigured)
    }
    async fn bind_auth(&self, _user: &str, _pass: &str) -> Result<BindOutcome> {
        Err(DirectoryError::NotConfigured)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_round_trips() {
        let cfg = DirectoryConfig {
            endpoints: vec![LdapEndpoint {
                url: "ldaps://dc.example.com".into(),
                base_dn: "dc=example,dc=com".into(),
                bind_dn: Some("cn=svc,dc=example,dc=com".into()),
                tls: LdapTls::Ldaps,
                priority: 0,
                attr_map: AttrMap::default(),
            }],
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: DirectoryConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, back);
    }

    #[tokio::test]
    async fn stub_is_not_configured() {
        let d = Directory::new(DirectoryConfig::default());
        assert!(matches!(
            d.search_gal("smith", 0).await,
            Err(DirectoryError::NotConfigured)
        ));
    }
}
