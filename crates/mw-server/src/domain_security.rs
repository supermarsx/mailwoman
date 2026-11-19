//! Explicit transport security policy; selection must finish before credentials are sent.
use crate::domain_config::Endpoint;
#[cfg(test)]
use crate::domain_config::TlsMode;
pub(crate) use mw_domain_config::domain_security::SecurityMode;
#[cfg(test)]
use mw_domain_config::domain_security::ConnectionSecurity;
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
