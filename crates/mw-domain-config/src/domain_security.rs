//! Validated connection security policies, without network negotiation.
use crate::domain_config::TlsMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SecurityMode {
    Fixed,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConnectionSecurity {
    pub mode: SecurityMode,
    /// Operator order is significant. It is not a request to retry credentials.
    pub allowed: Vec<TlsMode>,
    #[serde(default)]
    pub allow_plaintext: bool,
    #[serde(default)]
    pub server_name: Option<String>,
    #[serde(default)]
    pub ca_pem: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smtp_policy: Option<mw_mail_transport::TransportPolicyConfig>,
}

impl ConnectionSecurity {
    pub fn tls_options(&self) -> Result<mw_net::TlsOptions, String> {
        self.validate()?;
        use rustls_pki_types::{CertificateDer, pem::PemObject};
        let additional_roots = match &self.ca_pem {
            Some(pem) => CertificateDer::pem_slice_iter(pem.as_bytes())
                .map(|c| c.map(|c| c.as_ref().to_vec()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "the CA certificate bundle is invalid")?,
            None => Vec::new(),
        };
        Ok(mw_net::TlsOptions {
            server_name: self.server_name.clone(),
            additional_roots,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if let Some(policy) = &self.smtp_policy {
            policy.validate().map_err(|e| e.to_string())?;
            if policy.enabled()
                && (self.mode != SecurityMode::Fixed
                    || self.allowed.contains(&TlsMode::None)
                    || self.server_name.is_some())
            {
                return Err("SMTP transport policies require a fixed TLS method without a server-name override".into());
            }
        }
        if self.allowed.is_empty() || self.allowed.len() > 3 {
            return Err("choose between one and three connection security methods".into());
        }
        if self.mode == SecurityMode::Fixed && self.allowed.len() != 1 {
            return Err("fixed connection security requires exactly one method".into());
        }
        for (i, method) in self.allowed.iter().enumerate() {
            if self.allowed[..i].contains(method) {
                return Err("connection security priorities must not repeat a method".into());
            }
        }
        if self.allowed.contains(&TlsMode::None) && !self.allow_plaintext {
            return Err("plaintext requires explicit permission".into());
        }
        if let Some(name) = &self.server_name {
            if name.len() > 253 || name.trim() != name {
                return Err("the TLS server name is invalid".into());
            }
            rustls_pki_types::ServerName::try_from(name.clone())
                .map_err(|_| "the TLS server name must be a DNS name or IP address")?;
        }
        if let Some(pem) = &self.ca_pem {
            if pem.is_empty() || pem.len() > 64 * 1024 || pem.contains("PRIVATE KEY") {
                return Err("provide a public CA certificate bundle of at most 64 KiB".into());
            }
            use rustls_pki_types::{CertificateDer, pem::PemObject};
            let certs = CertificateDer::pem_slice_iter(pem.as_bytes())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "the CA certificate bundle is invalid")?;
            if certs.is_empty() {
                return Err("the CA certificate bundle contains no certificates".into());
            }
            let mut roots = rustls::RootCertStore::empty();
            for cert in certs {
                roots
                    .add(cert)
                    .map_err(|_| "the CA certificate is invalid")?;
            }
        }
        Ok(())
    }
}
