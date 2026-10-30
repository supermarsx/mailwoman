//! SMTP next-hop security. Policies apply to the configured relay, never to
//! recipient domains behind that relay. DNSSEC is validated locally, with every
//! DNS wire request and STS HTTPS fetch using the deployment's DnsAuth route.
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};
mod dns;
mod service;
mod sts;
mod tlsa;
pub use service::{PolicyContext, TransportDecision, TransportPolicy};
pub use tlsa::DaneVerifier;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DaneMode {
    #[default]
    Off,
    Opportunistic,
    Required,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StsMode {
    #[default]
    Off,
    Auto,
    Required,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransportPolicyConfig {
    #[serde(default)]
    pub dane: DaneMode,
    #[serde(default)]
    pub mta_sts: StsMode,
    #[serde(default)]
    pub policy_domain: Option<String>,
}
impl TransportPolicyConfig {
    pub fn enabled(&self) -> bool {
        self.dane != DaneMode::Off || self.mta_sts != StsMode::Off
    }
    pub fn validate(&self) -> Result<(), PolicyError> {
        if self.mta_sts != StsMode::Off && self.policy_domain.is_none() {
            return Err(PolicyError::Configuration);
        }
        if let Some(domain) = &self.policy_domain {
            normalize_domain(domain)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StsPolicyMode {
    Enforce,
    Testing,
    None,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StsPolicy {
    pub mode: StsPolicyMode,
    pub mx: Vec<String>,
    pub max_age: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CachedSts {
    pub id: String,
    pub policy: StsPolicy,
    pub fetched_at: u64,
    pub expires_at: u64,
}

/// Store implementations must atomically replace an entry, bound the number of
/// entries, and retain unexpired entries across restart. `key` is a SHA-256 hex
/// digest of the policy domain plus caller's egress authority namespace.
pub trait StsCache: Send + Sync {
    fn load<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<Option<CachedSts>, PolicyError>>;
    fn store<'a>(
        &'a self,
        key: &'a str,
        value: &'a CachedSts,
    ) -> BoxFuture<'a, Result<(), PolicyError>>;
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum PolicyError {
    #[error("invalid SMTP transport policy configuration")]
    Configuration,
    #[error("SMTP DNS policy could not be securely resolved")]
    Dns,
    #[error("SMTP DNSSEC validation failed")]
    Dnssec,
    #[error("required SMTP DANE policy is unavailable")]
    DaneRequired,
    #[error("SMTP TLSA policy could not authenticate the certificate")]
    Tlsa,
    #[error("SMTP MTA-STS policy could not be retrieved or parsed")]
    Sts,
    #[error("SMTP MTA-STS policy does not authorize the next hop")]
    MxMismatch,
    #[error("required SMTP MTA-STS policy is unavailable")]
    StsRequired,
    #[error("SMTP transport policy requires TLS")]
    TlsRequired,
    #[error("SMTP transport policy storage is unavailable")]
    Cache,
    #[error("SMTP transport policy deadline exceeded")]
    Timeout,
}

/// DNS A-labels only; policy names are operator input, not arbitrary URLs.
pub fn normalize_domain(input: &str) -> Result<String, PolicyError> {
    let domain = input.strip_suffix('.').unwrap_or(input);
    if domain.is_empty()
        || domain.len() > 253
        || domain.parse::<std::net::IpAddr>().is_ok()
        || domain.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || label.starts_with('-')
                || label.ends_with('-')
                || !label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
    {
        return Err(PolicyError::Configuration);
    }
    Ok(domain.to_ascii_lowercase())
}
