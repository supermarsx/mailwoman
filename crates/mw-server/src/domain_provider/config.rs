//! Administrator-selected EWS authority. No credential or destination comes from the client.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum EwsAuth {
    #[default]
    Basic,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct EwsConfig {
    pub endpoint_url: String,
    #[serde(default)]
    pub authentication: EwsAuth,
    #[serde(default)]
    pub ca_pem: Option<String>,
}

impl EwsConfig {
    pub fn endpoint(&self) -> Result<reqwest::Url, String> {
        if self.endpoint_url.len() > 2048 || self.endpoint_url.chars().any(char::is_control) {
            return Err("invalid EWS endpoint".into());
        }
        let url = reqwest::Url::parse(&self.endpoint_url).map_err(|_| "invalid EWS endpoint")?;
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(
                "EWS requires an exact HTTPS endpoint without credentials, query or fragment"
                    .into(),
            );
        }
        Ok(url)
    }

    pub fn roots(&self) -> Result<Vec<Vec<u8>>, String> {
        use rustls_pki_types::{CertificateDer, pem::PemObject};
        let Some(pem) = &self.ca_pem else {
            return Ok(Vec::new());
        };
        if pem.is_empty() || pem.len() > 65536 || pem.contains("PRIVATE KEY") {
            return Err("provide public EWS CA certificates of at most 64 KiB".into());
        }
        let certificates = CertificateDer::pem_slice_iter(pem.as_bytes())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "invalid EWS CA bundle")?;
        if certificates.is_empty() || certificates.len() > 8 {
            return Err("provide between one and eight EWS CA certificates".into());
        }
        let mut roots = rustls::RootCertStore::empty();
        for certificate in &certificates {
            roots
                .add(certificate.clone())
                .map_err(|_| "invalid EWS CA certificate")?;
        }
        Ok(certificates
            .into_iter()
            .map(|certificate| certificate.as_ref().to_vec())
            .collect())
    }

    pub fn validate(&self) -> Result<(), String> {
        self.endpoint()?;
        self.roots()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_https_authority_and_explicit_authentication_are_required() {
        let mut config = EwsConfig {
            endpoint_url: "https://exchange.example/EWS/Exchange.asmx".into(),
            authentication: EwsAuth::Basic,
            ca_pem: None,
        };
        assert!(config.validate().is_ok());
        for endpoint in [
            "http://exchange.example/EWS/Exchange.asmx",
            "https://user:secret@exchange.example/EWS",
            "https://exchange.example/EWS?tenant=other",
            "https://exchange.example/EWS#fragment",
            "https://exchange.example/\nEWS",
        ] {
            config.endpoint_url = endpoint.into();
            assert!(config.validate().is_err());
        }
        config.endpoint_url = "https://exchange.example/EWS/Exchange.asmx".into();
        assert!(
            serde_json::from_value::<EwsConfig>(
                serde_json::json!({"endpointUrl":config.endpoint_url,"authentication":"ntlm"})
            )
            .is_err()
        );
        config.ca_pem = Some("not a certificate".into());
        assert!(config.validate().is_err());
    }
}
