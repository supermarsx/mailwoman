//! Explicit transport security policy; selection must finish before credentials are sent.
use crate::domain_config::{Endpoint, TlsMode};
use serde::{Deserialize, Serialize};
use std::{io, sync::Arc};

pub(crate) struct ResolvedEndpoint {
    pub endpoint: Endpoint,
    pub tls_options: mw_net::TlsOptions,
    pub selected_reason: &'static str,
}

/// Only a protocol explicitly refusing an unadvertised upgrade, or an implicit
/// TLS handshake receiving a non-TLS record, can try the next allowed method.
/// Certificate, hostname, network and timeout failures are terminal.
fn retryable_method_error(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::Unsupported
        || matches!(
            error
                .get_ref()
                .and_then(|e| e.downcast_ref::<rustls::Error>()),
            Some(rustls::Error::InvalidMessage(
                rustls::InvalidMessage::InvalidContentType
            ))
        )
}

pub(crate) async fn resolve_endpoint(
    protocol: &str,
    endpoint: &Endpoint,
    domain: &str,
    plan: &mw_egress::plan::EgressPlan,
) -> io::Result<ResolvedEndpoint> {
    let Some(security) = &endpoint.security else {
        return Ok(ResolvedEndpoint {
            endpoint: endpoint.clone(),
            tls_options: mw_net::TlsOptions::default(),
            selected_reason: "legacy-fixed",
        });
    };
    let options = security
        .tls_options()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let mut selected = endpoint.clone();
    if security.mode == SecurityMode::Fixed {
        selected.tls = security.allowed[0];
        return Ok(ResolvedEndpoint {
            endpoint: selected,
            tls_options: options,
            selected_reason: "configured-fixed",
        });
    }
    let mut last = io::Error::new(
        io::ErrorKind::Unsupported,
        "no allowed connection security method is available",
    );
    for method in &security.allowed {
        selected.tls = *method;
        match crate::domain_probe::check_transport(protocol, &selected, domain, plan).await {
            Ok(_) => {
                return Ok(ResolvedEndpoint {
                    endpoint: selected,
                    tls_options: options,
                    selected_reason: "first-supported-in-priority-order",
                });
            }
            Err(error) if retryable_method_error(&error) => last = error,
            Err(error) => return Err(error),
        }
    }
    Err(last)
}

pub(crate) fn connector(
    protocol: &str,
    options: &mw_net::TlsOptions,
) -> io::Result<tokio_rustls::TlsConnector> {
    let tls13 = match protocol {
        "imap" => mw_imap::min_tls() == mw_imap::MinTls::V13,
        "pop3" => mw_pop3::conn::min_tls() == mw_pop3::conn::MinTls::V13,
        "smtp" => mw_smtp::min_tls() == mw_smtp::MinTls::V13,
        "sieve" => mw_sieve::min_tls() == mw_sieve::MinTls::V13,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unsupported mail protocol",
            ));
        }
    };
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    for cert in &options.additional_roots {
        roots
            .add(rustls_pki_types::CertificateDer::from(cert.clone()))
            .map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "invalid custom trust anchor")
            })?;
    }
    let builder = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ));
    let builder = if tls13 {
        builder.with_protocol_versions(&[&rustls::version::TLS13])
    } else {
        builder.with_safe_default_protocol_versions()
    }
    .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid TLS version policy"))?;
    Ok(tokio_rustls::TlsConnector::from(Arc::new(
        builder.with_root_certificates(roots).with_no_client_auth(),
    )))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SecurityMode {
    Fixed,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ConnectionSecurity {
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

#[cfg(test)]
mod tests {
    #[test]
    fn smtp_transport_policies_are_scoped_to_secure_dns_endpoints() {
        let endpoint = |protocol: &str, host: &str, method: &str, extra: &str| {
            let json = format!(
                r#"{{"{protocol}":{{"host":"{host}","port":465,"tls":"implicit","allowPrivate":true,"security":{{"mode":"fixed","allowed":["{method}"],"allowPlaintext":true,"smtpPolicy":{{"dane":"required","mtaSts":"auto","policyDomain":"relay.example.org"}}{extra}}}}}}}"#
            );
            crate::domain_config::UpstreamConfig::parse(&json)
        };
        assert!(endpoint("smtp", "mail.example.org", "implicit", "").is_ok());
        assert!(endpoint("imap", "mail.example.org", "implicit", "").is_err());
        assert!(endpoint("smtp", "127.0.0.1", "implicit", "").is_err());
        assert!(endpoint("smtp", "mail.example.org", "none", "").is_err());
        assert!(
            endpoint(
                "smtp",
                "mail.example.org",
                "implicit",
                r#", "serverName":"other.example.org""#
            )
            .is_err()
        );
        assert!(crate::domain_config::UpstreamConfig::parse(r#"{"smtp":{"host":"mail.example.org","port":465,"tls":"implicit","security":{"mode":"fixed","allowed":["implicit"],"smtpPolicy":{"mtaSts":"required"}}}}"#).is_err());
    }
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
    fn automatic(port: u16) -> Endpoint {
        Endpoint {
            host: "127.0.0.1".into(),
            port,
            tls: TlsMode::Implicit,
            auth: "password".into(),
            allow_private: true,
            authentication: None,
            security: Some(ConnectionSecurity {
                mode: SecurityMode::Auto,
                allowed: vec![TlsMode::Implicit, TlsMode::Starttls, TlsMode::None],
                allow_plaintext: true,
                server_name: None,
                ca_pem: None,
                smtp_policy: None,
            }),
        }
    }

    #[tokio::test]
    async fn auto_uses_ordered_preauth_negotiation_and_never_sends_credentials() {
        use std::sync::{
            Mutex,
            atomic::{AtomicUsize, Ordering},
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let count = Arc::new(AtomicUsize::new(0));
        let commands = Arc::new(Mutex::new(Vec::<String>::new()));
        let observed = count.clone();
        let recorded = commands.clone();
        let server = tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                observed.fetch_add(1, Ordering::SeqCst);
                let recorded = recorded.clone();
                tokio::spawn(async move {
                    if stream.write_all(b"* OK fixture ready\r\n").await.is_err() {
                        return;
                    }
                    let mut reader = tokio::io::BufReader::new(stream);
                    loop {
                        let mut command = String::new();
                        match reader.read_line(&mut command).await {
                            Ok(0) | Err(_) => break,
                            _ => {}
                        }
                        recorded.lock().unwrap().push(command.clone());
                        let tag = command.split_whitespace().next().unwrap_or("a0");
                        let response = format!("* CAPABILITY IMAP4rev1\r\n{tag} OK complete\r\n");
                        if reader
                            .get_mut()
                            .write_all(response.as_bytes())
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                });
            }
        });
        let plan = mw_egress::plan::EgressPlan::new(vec![], Default::default()).unwrap();
        let selected = resolve_endpoint("imap", &automatic(port), "example.test", &plan)
            .await
            .unwrap();
        assert_eq!(selected.endpoint.tls, TlsMode::None);
        assert_eq!(count.load(Ordering::SeqCst), 3);
        assert!(
            commands
                .lock()
                .unwrap()
                .iter()
                .all(|line| line.contains("CAPABILITY"))
        );
        server.abort();
    }

    #[tokio::test]
    async fn auto_does_not_fall_back_after_certificate_failure() {
        use rustls_pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
        use std::sync::atomic::{AtomicUsize, Ordering};
        let cert =
            CertificateDer::from_pem_slice(include_bytes!("../tests/fixtures/tls/cert1.pem"))
                .unwrap();
        let key = PrivateKeyDer::from_pem_slice(include_bytes!("../tests/fixtures/tls/key1.pem"))
            .unwrap();
        let config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .unwrap();
        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let count = Arc::new(AtomicUsize::new(0));
        let observed = count.clone();
        let server = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                observed.fetch_add(1, Ordering::SeqCst);
                let acceptor = acceptor.clone();
                tokio::spawn(async move {
                    let _ = acceptor.accept(stream).await;
                });
            }
        });
        let plan = mw_egress::plan::EgressPlan::new(vec![], Default::default()).unwrap();
        let result = resolve_endpoint("imap", &automatic(port), "example.test", &plan).await;
        assert!(result.is_err());
        assert_eq!(count.load(Ordering::SeqCst), 1);
        server.abort();
    }

    #[test]
    fn priority_is_explicit_and_plaintext_needs_consent() {
        let mut policy = ConnectionSecurity {
            mode: SecurityMode::Auto,
            allowed: vec![TlsMode::Implicit, TlsMode::Starttls],
            allow_plaintext: false,
            server_name: None,
            ca_pem: None,
            smtp_policy: None,
        };
        assert!(policy.validate().is_ok());
        policy.allowed.push(TlsMode::None);
        assert!(policy.validate().is_err());
        policy.allow_plaintext = true;
        assert!(policy.validate().is_ok());
        policy.mode = SecurityMode::Fixed;
        assert!(policy.validate().is_err());
        policy.allowed = vec![TlsMode::Implicit];
        policy.server_name = Some("https://mail.example.org".into());
        assert!(policy.validate().is_err());
        policy.server_name = Some("mail.example.org".into());
        policy.ca_pem = Some("not a certificate".into());
        assert!(policy.validate().is_err());
    }
}
