//! One bounded HTTPS panel request over the explicitly selected direct/SSH dialer.
use crate::hosting_connections::{PanelConfig, PanelProvider};
pub(crate) use crate::hosting_operations::{PanelError, PanelOperation, PanelResult};
use base64::Engine as _;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Request, header};
use hyper_util::rt::TokioIo;
use mw_net::SharedDialer;
use rustls_pki_types::{CertificateDer, ServerName, pem::PemObject};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

const RESPONSE_LIMIT: usize = 2 * 1024 * 1024;
struct ConnectionTask(tokio::task::JoinHandle<()>);
impl Drop for ConnectionTask {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub(crate) async fn perform(
    config: &PanelConfig,
    secret: &str,
    operation: &PanelOperation,
    dialer: SharedDialer,
) -> Result<PanelResult, PanelError> {
    if !config.enabled {
        return Err(PanelError::new("connection-disabled"));
    }
    let mut url =
        reqwest::Url::parse(&config.base_url).map_err(|_| PanelError::new("invalid-endpoint"))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(PanelError::new("invalid-endpoint"));
    }
    url.set_path(match config.provider {
        PanelProvider::Virtualmin => "/virtual-server/remote.cgi",
        PanelProvider::Hestia => "/api/",
    });
    let form = crate::hosting_operations::form(config, secret, operation)?;
    let mut encoded = reqwest::Url::parse("https://form.invalid/").expect("constant URL");
    encoded
        .query_pairs_mut()
        .extend_pairs(form.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    let body = encoded.query().unwrap_or_default().as_bytes().to_vec();
    if body.len() > 64 * 1024 {
        return Err(PanelError::new("request-too-large"));
    }
    let host = url
        .host_str()
        .expect("validated host")
        .trim_matches(['[', ']']);
    let port = url
        .port_or_known_default()
        .ok_or_else(|| PanelError::new("invalid-endpoint"))?;
    let server_name =
        ServerName::try_from(config.tls.server_name.as_deref().unwrap_or(host).to_owned())
            .map_err(|_| PanelError::new("invalid-tls-name"))?;
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    if let Some(pem) = &config.tls.ca_pem {
        if pem.len() > 65536 || pem.contains("PRIVATE KEY") {
            return Err(PanelError::new("invalid-ca-bundle"));
        }
        let mut count = 0;
        for cert in CertificateDer::pem_slice_iter(pem.as_bytes()) {
            roots
                .add(cert.map_err(|_| PanelError::new("invalid-ca-bundle"))?)
                .map_err(|_| PanelError::new("invalid-ca-bundle"))?;
            count += 1;
        }
        if count == 0 {
            return Err(PanelError::new("invalid-ca-bundle"));
        }
    }
    let versions = match config.tls.min_version.as_str() {
        "1.2" => rustls::DEFAULT_VERSIONS,
        "1.3" => &[&rustls::version::TLS13],
        _ => return Err(PanelError::new("invalid-tls-version")),
    };
    let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_protocol_versions(versions)
    .map_err(|_| PanelError::new("invalid-tls-version"))?
    .with_root_certificates(roots)
    .with_no_client_auth();
    let host_header = match url.port() {
        Some(port) => format!("{}:{port}", url.host_str().expect("validated host")),
        None => url.host_str().expect("validated host").to_owned(),
    };
    let mut request = Request::builder()
        .method("POST")
        .uri(url.path())
        .header(header::HOST, host_header)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::ACCEPT, "application/json, text/plain")
        .header(header::ACCEPT_ENCODING, "identity")
        .header(header::USER_AGENT, "Mailwoman panel management")
        .header(header::CONNECTION, "close");
    if config.provider == PanelProvider::Virtualmin {
        if config.username.is_empty()
            || config.username.contains(':')
            || config.username.chars().any(char::is_control)
        {
            return Err(PanelError::new("invalid-username"));
        }
        let credentials = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{secret}", config.username));
        request = request.header(header::AUTHORIZATION, format!("Basic {credentials}"));
    }
    let request = request
        .body(Full::new(Bytes::from(body)))
        .map_err(|_| PanelError::new("invalid-request"))?;
    let dispatched = AtomicBool::new(false);
    let exchange = async {
        let stream = dialer
            .dial(host, port)
            .await
            .map_err(|_| PanelError::new("connect-failed"))?;
        let stream = tokio_rustls::TlsConnector::from(Arc::new(tls))
            .connect(server_name, stream)
            .await
            .map_err(|_| PanelError::new("tls-failed"))?;
        let (mut sender, connection) = hyper::client::conn::http1::Builder::new()
            .max_buf_size(32768)
            .handshake(TokioIo::new(stream))
            .await
            .map_err(|_| PanelError::new("http-failed"))?;
        let _pump = ConnectionTask(tokio::spawn(async move {
            let _ = connection.await;
        }));
        dispatched.store(true, Ordering::Relaxed);
        let response = sender
            .send_request(request)
            .await
            .map_err(|_| PanelError::new("http-failed"))?;
        let status = response.status();
        if !status.is_success() {
            if matches!(status.as_u16(), 401 | 403) {
                dispatched.store(false, Ordering::Relaxed);
                return Err(PanelError::new("panel-auth-denied"));
            }
            return Err(PanelError::new(if status.is_redirection() {
                "redirect-refused"
            } else {
                "panel-http-failed"
            }));
        }
        let mut incoming = response.into_body();
        let mut bytes = Vec::new();
        while let Some(frame) = incoming.frame().await {
            let frame = frame.map_err(|_| PanelError::new("response-interrupted"))?;
            if let Some(chunk) = frame.data_ref() {
                if bytes.len().saturating_add(chunk.len()) > RESPONSE_LIMIT {
                    return Err(PanelError::new("response-too-large"));
                }
                bytes.extend_from_slice(chunk);
            }
        }
        let result = crate::hosting_operations::parse_response(config.provider, operation, &bytes);
        if result
            .as_ref()
            .is_err_and(|e| e.code == "panel-command-rejected")
        {
            dispatched.store(false, Ordering::Relaxed);
        }
        result
    };
    match tokio::time::timeout(Duration::from_secs(30), exchange).await {
        Ok(Ok(result)) => Ok(result),
        result => {
            let mut error = match result {
                Ok(Err(e)) => e,
                _ => PanelError::new("timeout"),
            };
            error.result_uncertain = operation.is_mutation() && dispatched.load(Ordering::Relaxed);
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hosting_connections::{PanelConnection, PanelTls};
    use rustls_pki_types::PrivateKeyDer;
    use std::sync::Mutex;
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
    const CERT: &str = include_str!("../tests/fixtures/tls/localhost-cert.pem");
    async fn peer(
        status: &'static str,
        body: &'static str,
    ) -> (u16, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
        let cert = CertificateDer::from_pem_slice(CERT.as_bytes()).unwrap();
        let key = PrivateKeyDer::from_pem_slice(include_bytes!(
            "../tests/fixtures/tls/localhost-key.pem"
        ))
        .unwrap();
        let config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
        let records = Arc::new(Mutex::new(Vec::new()));
        let observed = records.clone();
        let task = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let acceptor = acceptor.clone();
                let observed = observed.clone();
                tokio::spawn(async move {
                    let Ok(tls) = acceptor.accept(stream).await else {
                        return;
                    };
                    let mut reader = tokio::io::BufReader::new(tls);
                    let mut head = String::new();
                    let mut length = 0;
                    loop {
                        let mut line = String::new();
                        if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                            return;
                        }
                        if line == "\r\n" {
                            break;
                        }
                        if let Some(value) = line.to_lowercase().strip_prefix("content-length:") {
                            length = value.trim().parse::<usize>().unwrap();
                        }
                        head.push_str(&line);
                    }
                    let mut payload = vec![0; length];
                    reader.read_exact(&mut payload).await.unwrap();
                    head.push_str(&String::from_utf8(payload).unwrap());
                    observed.lock().unwrap().push(head);
                    let response = format!(
                        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nContent-Type: application/json\r\nLocation: https://other.example/\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = reader.get_mut().write_all(response.as_bytes()).await;
                });
            }
        });
        (port, records, task)
    }
    fn config(port: u16, provider: PanelProvider) -> PanelConfig {
        PanelConfig {
            id: "fixture".into(),
            name: "Fixture".into(),
            provider,
            base_url: format!("https://127.0.0.1:{port}"),
            enabled: true,
            username: "admin".into(),
            owner: "owner".into(),
            access_key: "access".into(),
            allow_private: true,
            connection: PanelConnection::Direct,
            tls: PanelTls {
                server_name: Some("localhost".into()),
                ca_pem: Some(CERT.into()),
                min_version: "1.2".into(),
            },
            allow_domain_lifecycle: false,
        }
    }
    fn dialer(port: u16) -> SharedDialer {
        crate::domain_transport::GuardedDialer::shared("127.0.0.1", port, true)
    }
    #[tokio::test]
    async fn https_name_and_private_ca_are_verified_before_credentials_and_post_is_scoped() {
        let (port, records, task) = peer(
            "200 OK",
            r#"{"status":"success","data":[{"name":"example.org","values":{}}]}"#,
        )
        .await;
        let mut cfg = config(port, PanelProvider::Virtualmin);
        cfg.tls.server_name = Some("wrong.example".into());
        assert_eq!(
            perform(
                &cfg,
                "synthetic-secret",
                &PanelOperation::ListDomains,
                dialer(port)
            )
            .await
            .err()
            .unwrap()
            .code,
            "tls-failed"
        );
        cfg.tls.server_name = Some("localhost".into());
        cfg.tls.ca_pem = None;
        assert!(
            perform(
                &cfg,
                "synthetic-secret",
                &PanelOperation::ListDomains,
                dialer(port)
            )
            .await
            .is_err()
        );
        assert!(records.lock().unwrap().is_empty());
        cfg.tls.ca_pem = Some(CERT.into());
        let result = perform(
            &cfg,
            "synthetic-secret",
            &PanelOperation::ListDomains,
            dialer(port),
        )
        .await
        .unwrap();
        assert_eq!(result.domains[0].name, "example.org");
        let recorded = records.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        assert!(recorded[0].starts_with("POST /virtual-server/remote.cgi HTTP/1.1\r\n"));
        assert!(recorded[0].contains("program=list-domains"));
        assert!(!recorded[0].contains("synthetic-secret"));
        task.abort();
    }
    #[tokio::test]
    async fn hestia_mutation_uses_post_form_and_redirect_never_retries() {
        let op = PanelOperation::CreateMailbox {
            domain: "example.org".into(),
            account: "alice".into(),
            password: "a&b=secret".into(),
            quota_mi_b: Some(50),
            display_name: None,
        };
        let (port, records, task) = peer("200 OK", "0").await;
        assert!(
            perform(
                &config(port, PanelProvider::Hestia),
                "secret",
                &op,
                dialer(port)
            )
            .await
            .unwrap()
            .changed
        );
        let recorded = records.lock().unwrap()[0].clone();
        assert!(recorded.starts_with("POST /api/ HTTP/1.1\r\n"));
        assert!(
            recorded.contains("hash=access%3Asecret") && recorded.contains("arg4=a%26b%3Dsecret")
        );
        task.abort();
        let (port, records, task) = peer("302 Found", "").await;
        let error = perform(
            &config(port, PanelProvider::Hestia),
            "secret",
            &op,
            dialer(port),
        )
        .await
        .err()
        .unwrap();
        assert_eq!(error.code, "redirect-refused");
        assert!(error.result_uncertain);
        assert_eq!(records.lock().unwrap().len(), 1);
        task.abort();
    }

    /// The `Authorization` value and the form body of one recorded request.
    fn auth_and_body(recorded: &str) -> (String, String) {
        let (head, body) = recorded.split_once("\r\n\r\n").unwrap_or((recorded, ""));
        let auth = head
            .lines()
            .find_map(|l| {
                l.split_once(':')
                    .filter(|(k, _)| k.eq_ignore_ascii_case("authorization"))
            })
            .map(|(_, v)| v.trim().to_owned())
            .unwrap_or_default();
        // `peer` appends the payload straight after the head's last CRLF.
        let body = if body.is_empty() {
            recorded.rsplit("\r\n").next().unwrap_or_default()
        } else {
            body
        };
        (auth, body.to_owned())
    }

    /// Virtualmin Remote API (https://www.virtualmin.com/docs/development/remote-api/)
    /// against a local HTTPS fixture. The bodies in `tests/fixtures/virtualmin/` are
    /// Virtualmin's own `json-lib.pl` converter applied to `list-*` multiline text
    /// (see that directory's README); no live Virtualmin is reached here.
    #[tokio::test]
    async fn virtualmin_fixture_requests_and_parsed_inventory() {
        const BASIC: &str = "Basic YWRtaW46c3ludGhldGljLXNlY3JldA=="; // admin:synthetic-secret
        let cases: [(&str, PanelOperation, &str); 3] = [
            (
                include_str!("../tests/fixtures/virtualmin/list-domains.json"),
                PanelOperation::ListDomains,
                "json=1&with-feature=mail&program=list-domains",
            ),
            (
                include_str!("../tests/fixtures/virtualmin/list-users.json"),
                PanelOperation::ListMailboxes {
                    domain: "example.org".into(),
                },
                "json=1&program=list-users&domain=example.org",
            ),
            (
                include_str!("../tests/fixtures/virtualmin/create-user.json"),
                PanelOperation::CreateMailbox {
                    domain: "example.org".into(),
                    account: "alice".into(),
                    password: "p&ss word".into(),
                    quota_mi_b: Some(50),
                    display_name: Some("Alice Example".into()),
                },
                "json=1&user=alice&pass=p%26ss+word&quota=51200&real=Alice+Example&program=create-user&domain=example.org",
            ),
        ];
        let mut results = Vec::new();
        for (body, op, form) in cases {
            let (port, records, task) = peer("200 OK", body).await;
            let result = perform(
                &config(port, PanelProvider::Virtualmin),
                "synthetic-secret",
                &op,
                dialer(port),
            )
            .await
            .unwrap();
            let recorded = records.lock().unwrap()[0].clone();
            assert!(recorded.starts_with("POST /virtual-server/remote.cgi HTTP/1.1\r\n"));
            let (auth, sent) = auth_and_body(&recorded);
            assert_eq!(auth, BASIC);
            assert_eq!(sent, form);
            task.abort();
            results.push(result);
        }

        let domains = &results[0].domains;
        assert_eq!(domains.len(), 2);
        assert_eq!(
            (
                domains[0].name.as_str(),
                domains[0].owner.as_deref(),
                domains[0].enabled
            ),
            ("example.org", Some("exampleorg"), Some(true))
        );
        assert_eq!(
            (domains[1].name.as_str(), domains[1].enabled),
            ("old.example.org", Some(false))
        );

        let mailboxes = &results[1].mailboxes;
        assert_eq!(
            mailboxes.len(),
            2,
            "the user without an address is left out"
        );
        assert_eq!(mailboxes[0].account, "alice@example.org");
        assert_eq!(mailboxes[0].address, "alice@example.org");
        assert_eq!(mailboxes[0].enabled, Some(true));
        assert_eq!(mailboxes[0].quota_mi_b, Some(50));
        assert_eq!(
            mailboxes[0].aliases,
            ["sales@example.org", "info@example.org"]
        );
        assert_eq!(mailboxes[1].address, "bob@example.org");
        assert_eq!(mailboxes[1].enabled, Some(false));

        assert!(results[2].changed);
        for result in &results {
            let shown = serde_json::to_string(result).unwrap();
            for leak in ["secret", "$6$", "/home", "Maildir"] {
                assert!(!shown.contains(leak), "{leak} in {shown}");
            }
        }

        let (port, records, task) = peer(
            "200 OK",
            include_str!("../tests/fixtures/virtualmin/create-user-failure.json"),
        )
        .await;
        let error = perform(
            &config(port, PanelProvider::Virtualmin),
            "synthetic-secret",
            &PanelOperation::DeleteMailbox {
                domain: "example.org".into(),
                account: "alice".into(),
            },
            dialer(port),
        )
        .await
        .err()
        .unwrap();
        assert_eq!(error.code, "panel-command-rejected");
        assert!(!error.result_uncertain, "a refusal is a known outcome");
        assert_eq!(records.lock().unwrap().len(), 1);
        task.abort();
    }
}
