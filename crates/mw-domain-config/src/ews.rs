//! Administrator-selected EWS authority. No credential or destination comes from the client.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EwsAuth {
    #[default]
    Basic,
    Ntlm,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EwsConfig {
    pub endpoint_url: String,
    #[serde(default)]
    pub authentication: EwsAuth,
    #[serde(default)]
    pub ca_pem: Option<String>,
    #[serde(default)]
    pub ntlm_domain: String,
    #[serde(default)]
    pub workstation: String,
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
        let identifier = |value: &str, limit: usize| {
            value.len() <= limit
                && value
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        };
        if !identifier(&self.ntlm_domain, 128)
            || !identifier(&self.workstation, 64)
            || (self.authentication == EwsAuth::Ntlm && self.ntlm_domain.is_empty())
        {
            return Err("NTLM requires a valid domain; workstation is optional".into());
        }
        Ok(())
    }
    pub fn credential_identity(&self, username: &str) -> Result<(String, String, String), String> {
        self.validate()?;
        if self.authentication == EwsAuth::Basic {
            return Ok((username.into(), String::new(), String::new()));
        }
        let user = if let Some((domain, user)) = username.split_once('\\') {
            if !domain.eq_ignore_ascii_case(&self.ntlm_domain) {
                return Err("NTLM login domain does not match configured domain".into());
            }
            user
        } else {
            username.split_once('@').map_or(username, |(user, _)| user)
        };
        if user.is_empty() || user.len() > 320 || user.chars().any(char::is_control) {
            return Err("invalid NTLM username".into());
        }
        Ok((
            user.into(),
            self.ntlm_domain.clone(),
            self.workstation.clone(),
        ))
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
            ntlm_domain: String::new(),
            workstation: String::new(),
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
        let ntlm=serde_json::from_value::<EwsConfig>(serde_json::json!({"endpointUrl":config.endpoint_url,"authentication":"ntlm","ntlmDomain":"CORP","workstation":"MWCLIENT"})).unwrap();
        assert_eq!(
            ntlm.credential_identity("alice@example.test").unwrap(),
            ("alice".into(), "CORP".into(), "MWCLIENT".into())
        );
        assert!(ntlm.credential_identity("FOREIGN\\alice").is_err());
        assert_eq!(ntlm.credential_identity("CORP\\alice").unwrap().0, "alice");
        let mut invalid = ntlm.clone();
        invalid.ntlm_domain.clear();
        assert!(invalid.validate().is_err());
        config.ca_pem = Some("not a certificate".into());
        assert!(config.validate().is_err());
    }
}
