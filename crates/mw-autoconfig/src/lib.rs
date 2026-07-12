#![forbid(unsafe_code)]
//! `mw-autoconfig` — server discovery for a login email (plan §0/§2.4):
//! RFC 6186 SRV → Thunderbird autoconfig XML → MS Autodiscover v2 → offline
//! provider DB → manual. Returns an [`AccountCandidate`] the login flow feeds
//! into account creation.
//!
//! Scaffolder note (e0): [`discover`] is a compiling stub with a `todo!()`
//! body; the fallback ladder and recorded-XML-fixture tests are added by e5.

use serde::{Deserialize, Serialize};

/// Transport security for a discovered server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TlsMode {
    /// Implicit TLS from connect (IMAPS 993 / POP3S 995 / SMTPS 465).
    Implicit,
    /// Cleartext then STARTTLS upgrade (143 / 110 / 587).
    StartTls,
    /// No transport security (discouraged; manual only).
    None,
}

/// Authentication method a discovered server expects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthMethod {
    /// Password over a SASL PLAIN/LOGIN mechanism.
    Password,
    /// OAuth2 bearer via SASL XOAUTH2 (Gmail/Outlook).
    OAuth2,
}

/// Which rung of the discovery ladder produced the candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiscoverySource {
    /// RFC 6186 DNS SRV records.
    Srv,
    /// Thunderbird autoconfig XML (ISPDB / provider-hosted).
    ThunderbirdAutoconfig,
    /// Microsoft Autodiscover v2.
    Autodiscover,
    /// Offline bundled provider database.
    ProviderDb,
    /// User-entered manual configuration.
    Manual,
}

/// One discovered server endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerSpec {
    pub host: String,
    pub port: u16,
    pub tls: TlsMode,
}

/// The discovery result the login flow consumes (plan §2.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountCandidate {
    pub imap: ServerSpec,
    pub pop3: Option<ServerSpec>,
    pub smtp: ServerSpec,
    pub auth: AuthMethod,
    pub source: DiscoverySource,
}

/// Errors from discovery.
#[derive(Debug, thiserror::Error)]
pub enum DiscoverError {
    /// The email address was not a valid `local@domain`.
    #[error("invalid email address: {0}")]
    InvalidEmail(String),
    /// Every rung of the ladder failed; the UI falls back to manual entry.
    #[error("no configuration discovered for {0}")]
    NotFound(String),
    /// A network/lookup error while probing a discovery source.
    #[error("discovery lookup error: {0}")]
    Lookup(String),
}

/// Discover server configuration for `email`, walking the fallback ladder
/// (SRV → Thunderbird autoconfig → Autodiscover v2 → provider DB → manual).
pub async fn discover(_email: &str) -> Result<AccountCandidate, DiscoverError> {
    todo!("e5: SRV -> TB-autoconfig -> Autodiscover -> provider-DB ladder")
}
