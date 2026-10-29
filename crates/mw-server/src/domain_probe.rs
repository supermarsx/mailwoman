//! Saved-domain diagnostics. No request-selected hosts, credentials or mail are sent.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability as C,
    domain_config::{Endpoint, TlsMode, UpstreamConfig},
    domain_transport::GuardedDialer,
};
use axum::{
    Json,
    extract::State,
    http::{Method, StatusCode},
};
use mw_net::{BoxStream, Dialer};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::POST, "/diag/upstream", C::DiagnosticsRun, upstream);
    r.admin_endpoint(Method::POST, "/diag/dns", C::DiagnosticsRun, dns);
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    domain: String,
}
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Probe {
    protocol: String,
    host: String,
    port: u16,
    tls: String,
    reachable: bool,
    /// Whole milliseconds, truncated (kept for older clients).
    latency_ms: u64,
    /// The same time in microseconds; a loopback or LAN handshake is often under 1 ms.
    latency_us: u64,
    tls_version: Option<String>,
    cert_not_after: Option<String>,
    cert_days_left: Option<i64>,
    greeting: Option<String>,
    capabilities: Vec<String>,
    error: Option<String>,
    selected_reason: Option<String>,
}
impl Probe {
    fn set_latency(&mut self, elapsed: Duration) {
        self.latency_ms = elapsed.as_millis().min(u64::MAX as u128) as u64;
        self.latency_us = elapsed.as_micros().min(u64::MAX as u128) as u64;
    }
}
type Windows = HashMap<(String, &'static str), (Instant, u8)>;
static LIMITS: OnceLock<Mutex<Windows>> = OnceLock::new();
type Reports = HashMap<String, (Instant, Value)>;
static CACHE: OnceLock<Mutex<Reports>> = OnceLock::new();
fn limit(session: &str, kind: &'static str) -> Result<(), AdminApiError> {
    let mut limits = LIMITS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    limits.retain(|_, (at, _)| at.elapsed() < Duration::from_secs(60));
    let key = (session.to_string(), kind);
    if limits.len() >= 4096 && !limits.contains_key(&key) {
        return Err(rate_limited());
    }
    let (_, count) = limits.entry(key).or_insert((Instant::now(), 0));
    if *count >= 6 {
        return Err(rate_limited());
    }
    *count += 1;
    Ok(())
}
fn rate_limited() -> AdminApiError {
    AdminApiError::new(
        StatusCode::TOO_MANY_REQUESTS,
        "rate_limited",
        "Run at most six checks per minute.",
    )
    .with("retryAfterSecs", 60)
}
/// The last ten minutes of probe results, for the admin dashboard warnings.
pub fn cached_reports() -> Vec<Value> {
    let mut cache = CACHE
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    cache.retain(|_, (at, _)| at.elapsed() < Duration::from_secs(600));
    cache.values().map(|(_, value)| value.clone()).collect()
}
async fn saved(
    state: &AppState,
    ctx: &AdminCtx,
    input: Input,
    kind: &'static str,
) -> Result<(String, UpstreamConfig), AdminApiError> {
    ctx.require_unscoped(C::DomainsUpstreamWrite)?;
    let domain = crate::domain_config::normalize_domain(&input.domain)
        .map_err(|e| AdminApiError::invalid("invalidDomain", e, "/domain"))?;
    let row = state
        .store
        .get_domain_config(&domain)
        .await?
        .ok_or_else(|| AdminApiError::not_found("Save the domain before checking it."))?;
    limit(&ctx.session_hash, kind)?;
    let config = UpstreamConfig::parse(&row.upstream_json).map_err(|_| {
        AdminApiError::conflict(
            "invalidUpstream",
            "The saved upstream configuration is invalid.",
        )
    })?;
    Ok((domain, config))
}
async fn upstream(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Json(input): Json<Input>,
) -> Result<Json<Value>, AdminApiError> {
    let (domain, config) = saved(&state, &ctx, input, "upstream").await?;
    let plan = crate::image_proxy::load_plan(&state.store)
        .await
        .map_err(|_| {
            AdminApiError::conflict("probeUnavailable", "Egress configuration is unavailable.")
        })?;
    let mut probes = Vec::new();
    for (protocol, endpoint) in [
        ("imap", &config.imap),
        ("pop3", &config.pop3),
        ("smtp", &config.smtp),
        ("sieve", &config.sieve),
    ] {
        if let Some(endpoint) = endpoint {
            probes.push(probe_mail(protocol, endpoint, &domain, &plan).await);
        }
    }
    if let Some(jmap) = &config.jmap {
        probes.push(probe_http("jmap", &jmap.session_url, &domain, &plan).await);
    }
    if let Some(dav) = &config.dav {
        for (protocol, url) in [("caldav", &dav.caldav_url), ("carddav", &dav.carddav_url)] {
            if let Some(url) = url {
                probes.push(probe_http(protocol, url, &domain, &plan).await);
            }
        }
    }
    let body = json!({"domain":domain,"selection":config.selection,"selectionOrder":["preferred","priority","id"],"results":probes,"ranAt":chrono::Utc::now().to_rfc3339(),"credentialsSent":false});
    let mut cache = CACHE
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    cache.retain(|_, (at, _)| at.elapsed() < Duration::from_secs(600));
    if cache.len() >= 256
        && let Some(oldest) = cache
            .iter()
            .min_by_key(|(_, (at, _))| *at)
            .map(|(key, _)| key.clone())
    {
        cache.remove(&oldest);
    }
    cache.insert(domain, (Instant::now(), body.clone()));
    Ok(Json(body))
}
async fn dns(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Json(input): Json<Input>,
) -> Result<Json<Value>, AdminApiError> {
    let (domain, _) = saved(&state, &ctx, input, "dns").await?;
    let report = tokio::time::timeout(
        Duration::from_secs(30),
        mw_autoconfig::diagnostics::diagnose(&domain),
    )
    .await
    .map_err(|_| AdminApiError::conflict("dnsTimeout", "DNS checks timed out."))?
    .map_err(|_| AdminApiError::conflict("dnsUnavailable", "DNS resolver is unavailable."))?;
    Ok(Json(
        serde_json::to_value(report).expect("DNS report serializes"),
    ))
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
async fn line(reader: &mut BufReader<BoxStream>) -> io::Result<String> {
    let mut bytes = Vec::new();
    loop {
        let b = reader.read_u8().await?;
        if b == b'\n' {
            break;
        }
        if bytes.len() >= 4096 {
            return Err(invalid("upstream reply exceeded the line limit"));
        }
        bytes.push(b);
    }
    let text = String::from_utf8_lossy(&bytes);
    Ok(text
        .trim_end_matches('\r')
        .chars()
        .filter(|c| !c.is_control() || *c == '\t')
        .collect())
}
async fn reply(
    reader: &mut BufReader<BoxStream>,
    protocol: &str,
    expected: &str,
) -> io::Result<Vec<String>> {
    let mut lines = Vec::new();
    for _ in 0..64 {
        let text = line(reader).await?;
        let upper = text.to_ascii_uppercase();
        let done = match protocol {
            "smtp" => {
                if !upper.starts_with(expected) {
                    return Err(invalid("SMTP rejected the probe command"));
                }
                text.as_bytes().get(3) == Some(&b' ')
            }
            "imap" => {
                if upper.split_whitespace().next() == Some(expected) {
                    if upper.split_whitespace().nth(1) != Some("OK") {
                        return Err(invalid("IMAP rejected the probe command"));
                    }
                    true
                } else {
                    false
                }
            }
            "sieve" => {
                if upper.starts_with("NO") || upper.starts_with("BYE") {
                    return Err(invalid("Sieve rejected the probe command"));
                }
                upper.split_whitespace().next() == Some("OK")
            }
            _ => true,
        };
        lines.push(text);
        if done {
            return Ok(lines);
        }
    }
    Err(invalid("upstream reply exceeded the line count limit"))
}
async fn command(reader: &mut BufReader<BoxStream>, text: &[u8]) -> io::Result<()> {
    reader.get_mut().write_all(text).await?;
    reader.get_mut().flush().await
}
async fn wrap(
    stream: BoxStream,
    protocol: &str,
    endpoint: &Endpoint,
    result: &mut Probe,
) -> io::Result<BoxStream> {
    let options = endpoint
        .security
        .as_ref()
        .map(|s| s.tls_options())
        .transpose()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
        .unwrap_or_default();
    let host = options.server_name.as_deref().unwrap_or(&endpoint.host);
    let name = rustls_pki_types::ServerName::try_from(host.trim_matches(['[', ']']).to_owned())
        .map_err(|_| invalid("invalid TLS server name"))?;
    let tls = crate::domain_security::connector(protocol, &options)?
        .connect(name, stream)
        .await?;
    let (_, session) = tls.get_ref();
    result.tls_version = session.protocol_version().map(|v| format!("{v:?}"));
    if let Some(leaf) = session.peer_certificates().and_then(|certs| certs.first())
        && let Ok(cert) = mw_crypto::smime_cms::parse_certificate(leaf.as_ref())
    {
        let expires = cert
            .tbs_certificate()
            .validity()
            .not_after
            .to_unix_duration()
            .as_secs();
        result.cert_not_after =
            chrono::DateTime::from_timestamp(expires as i64, 0).map(|dt| dt.to_rfc3339());
        result.cert_days_left = Some((expires as i64 - chrono::Utc::now().timestamp()) / 86400);
    }
    Ok(Box::new(tls))
}
async fn handshake(
    protocol: &str,
    endpoint: &Endpoint,
    stream: BoxStream,
    result: &mut Probe,
) -> io::Result<()> {
    let stream = if endpoint.tls == TlsMode::Implicit {
        wrap(stream, protocol, endpoint, result).await?
    } else {
        stream
    };
    let mut reader = BufReader::new(stream);
    let greeting = if protocol == "smtp" {
        reply(&mut reader, "smtp", "220").await?
    } else if protocol == "sieve" {
        reply(&mut reader, "sieve", "").await?
    } else {
        vec![line(&mut reader).await?]
    };
    if protocol == "imap" && !greeting[0].to_ascii_uppercase().starts_with("* OK") {
        return Err(invalid("not an IMAP greeting"));
    }
    if protocol == "pop3" && !greeting[0].starts_with("+OK") {
        return Err(invalid("not a POP3 greeting"));
    }
    result.greeting = Some(greeting.join(" ").chars().take(512).collect());
    if endpoint.tls == TlsMode::Starttls {
        match protocol {
            "smtp" => {
                command(&mut reader, b"EHLO mailwoman.invalid\r\n").await?;
                let capabilities = reply(&mut reader, "smtp", "250").await?;
                if !capabilities.iter().any(|c| {
                    c.get(4..).is_some_and(|c| {
                        c.split_whitespace()
                            .next()
                            .is_some_and(|s| s.eq_ignore_ascii_case("STARTTLS"))
                    })
                }) {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "SMTP does not advertise STARTTLS",
                    ));
                }
                command(&mut reader, b"STARTTLS\r\n").await?;
                reply(&mut reader, "smtp", "220").await?;
            }
            "imap" => {
                command(&mut reader, b"a0 CAPABILITY\r\n").await?;
                let caps = reply(&mut reader, "imap", "A0").await?;
                if !caps
                    .iter()
                    .flat_map(|c| c.split_whitespace())
                    .any(|c| c.eq_ignore_ascii_case("STARTTLS"))
                {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "IMAP does not advertise STARTTLS",
                    ));
                }
                command(&mut reader, b"a1 STARTTLS\r\n").await?;
                reply(&mut reader, "imap", "A1").await?;
            }
            "pop3" => {
                command(&mut reader, b"STLS\r\n").await?;
                if !line(&mut reader).await?.starts_with("+OK") {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "POP3 refused STLS",
                    ));
                }
            }
            "sieve" => {
                if !greeting
                    .iter()
                    .any(|c| c.trim_matches('"').eq_ignore_ascii_case("STARTTLS"))
                {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "Sieve does not advertise STARTTLS",
                    ));
                }
                command(&mut reader, b"STARTTLS\r\n").await?;
                reply(&mut reader, "sieve", "").await?;
            }
            _ => return Err(invalid("unsupported mail protocol")),
        }
        if !reader.buffer().is_empty() {
            return Err(invalid("unexpected bytes before TLS handshake"));
        }
        reader = BufReader::new(wrap(reader.into_inner(), protocol, endpoint, result).await?);
        if protocol == "sieve" {
            result.capabilities = reply(&mut reader, "sieve", "").await?;
            return Ok(());
        }
    }
    result.capabilities = match protocol {
        "smtp" => {
            command(&mut reader, b"EHLO mailwoman.invalid\r\n").await?;
            reply(&mut reader, "smtp", "250").await?
        }
        "imap" => {
            command(&mut reader, b"a2 CAPABILITY\r\n").await?;
            reply(&mut reader, "imap", "A2").await?
        }
        "pop3" => {
            command(&mut reader, b"CAPA\r\n").await?;
            let first = line(&mut reader).await?;
            if first.starts_with("-ERR") {
                vec!["CAPA not supported".into()]
            } else {
                if !first.starts_with("+OK") {
                    return Err(invalid("invalid POP3 capability reply"));
                }
                let mut capabilities = Vec::new();
                let mut complete = false;
                for _ in 0..64 {
                    let line = line(&mut reader).await?;
                    if line == "." {
                        complete = true;
                        break;
                    }
                    capabilities.push(line);
                }
                if !complete {
                    return Err(invalid("POP3 capability reply exceeded limit"));
                }
                capabilities
            }
        }
        "sieve" => greeting,
        _ => return Err(invalid("unsupported mail protocol")),
    };
    Ok(())
}
async fn probe_mail(
    protocol: &str,
    endpoint: &Endpoint,
    domain: &str,
    plan: &mw_egress::plan::EgressPlan,
) -> Probe {
    let start = Instant::now();
    let mut result = Probe {
        protocol: protocol.into(),
        host: endpoint.host.clone(),
        port: endpoint.port,
        tls: match endpoint.tls {
            TlsMode::Implicit => "implicit",
            TlsMode::Starttls => "starttls",
            TlsMode::None => "none",
        }
        .into(),
        ..Default::default()
    };
    let checked = async {
        let resolved =
            crate::domain_security::resolve_endpoint(protocol, endpoint, domain, plan).await?;
        let mut checked = check_transport(protocol, &resolved.endpoint, domain, plan).await?;
        checked.selected_reason = Some(resolved.selected_reason.into());
        Ok::<_, io::Error>(checked)
    }
    .await;
    match checked {
        Ok(checked) => result = checked,
        Err(error) => result.error = Some(error.to_string()),
    }
    result.capabilities = result
        .capabilities
        .into_iter()
        .take(32)
        .map(|line| line.chars().take(256).collect())
        .collect();
    result.set_latency(start.elapsed());
    result
}

/// A bounded, credential-free protocol handshake for one already selected method.
pub(crate) async fn check_transport(
    protocol: &str,
    endpoint: &Endpoint,
    domain: &str,
    plan: &mw_egress::plan::EgressPlan,
) -> io::Result<Probe> {
    let mut result = Probe {
        protocol: protocol.into(),
        host: endpoint.host.clone(),
        port: endpoint.port,
        tls: match endpoint.tls {
            TlsMode::Implicit => "implicit",
            TlsMode::Starttls => "starttls",
            TlsMode::None => "none",
        }
        .into(),
        ..Default::default()
    };
    tokio::time::timeout(Duration::from_secs(5), async {
        let stream: BoxStream = if protocol == "sieve" {
            Box::new(
                mw_egress::plan::connect_sieve_with(
                    plan,
                    Some(domain),
                    &endpoint.host,
                    endpoint.port,
                    endpoint.allow_private,
                    endpoint.tls != TlsMode::None,
                )
                .await
                .map_err(|_| invalid("Sieve route or address policy refused the connection"))?,
            )
        } else {
            GuardedDialer::new(&endpoint.host, endpoint.port, endpoint.allow_private)
                .dial(&endpoint.host, endpoint.port)
                .await?
        };
        handshake(protocol, endpoint, stream, &mut result).await
    })
    .await
    .map_err(|_| {
        io::Error::new(
            io::ErrorKind::TimedOut,
            "connection check timed out after five seconds",
        )
    })??;
    result.reachable = true;
    Ok(result)
}
async fn probe_http(
    protocol: &str,
    value: &str,
    domain: &str,
    plan: &mw_egress::plan::EgressPlan,
) -> Probe {
    let start = Instant::now();
    let Ok(url) = reqwest::Url::parse(value) else {
        return Probe {
            protocol: protocol.into(),
            error: Some("Invalid saved URL".into()),
            ..Default::default()
        };
    };
    let mut result = Probe {
        protocol: protocol.into(),
        host: url.host_str().unwrap_or_default().into(),
        port: url.port_or_known_default().unwrap_or(443),
        tls: if url.scheme() == "https" {
            "implicit"
        } else {
            "none"
        }
        .into(),
        ..Default::default()
    };
    let request =
        mw_egress::plan::Request::get(mw_egress::TrafficClass::AdminDetect).for_domain(domain);
    match tokio::time::timeout(
        Duration::from_secs(5),
        mw_egress::plan::send_with(plan, url, &request),
    )
    .await
    {
        Ok(response) => match response.outcome {
            Ok(_) => {
                result.reachable = true;
                result.greeting =
                    Some("Guarded HTTP request succeeded; no credentials sent".into());
            }
            Err(error) => result.error = Some(format!("Guarded HTTP request refused: {error:?}")),
        },
        Err(_) => result.error = Some("HTTP check timed out after five seconds".into()),
    }
    result.set_latency(start.elapsed());
    result
}

/// Run one validated saved backend without exposing server greetings, URLs,
/// credentials, certificates or raw transport errors in the support report.
pub(crate) async fn diagnostic_backend(
    domain: &str,
    backend: &crate::domain_selection::Backend,
    plan: &mw_egress::plan::EgressPlan,
) -> Value {
    use crate::domain_selection::Protocol;
    if backend.protocol == Protocol::Ews {
        return json!({"protocol":"ews","reachable":false,"tls":false,
            "credentialsSent":false,"selectedReason":"authenticated-provider-probe-required"});
    }
    let protocol = match backend.protocol {
        Protocol::Imap => "imap",
        Protocol::Pop3 => "pop3",
        Protocol::Jmap => "jmap",
        Protocol::Smtp => "smtp",
        Protocol::Sieve => "sieve",
        Protocol::Ews => "ews",
    };
    let probe = if let Some(endpoint) = &backend.endpoint {
        probe_mail(protocol, endpoint, domain, plan).await
    } else if let Some(url) = &backend.session_url {
        probe_http(protocol, url, domain, plan).await
    } else {
        Probe::default()
    };
    json!({"protocol":protocol,"reachable":probe.reachable,"tls":probe.tls,
        "tlsVersion":probe.tls_version,"credentialsSent":false,
        "capabilityCount":probe.capabilities.len(),"selectedReason":probe.selected_reason})
}

pub(crate) async fn diagnostic_http_service(
    domain: &str,
    protocol: &str,
    url: &str,
    plan: &mw_egress::plan::EgressPlan,
) -> Value {
    let probe = probe_http(protocol, url, domain, plan).await;
    json!({"protocol":protocol,"reachable":probe.reachable,"tls":probe.tls,"credentialsSent":false})
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncBufReadExt;
    fn endpoint(tls: TlsMode) -> Endpoint {
        Endpoint {
            host: "localhost".into(),
            port: 1234,
            tls,
            auth: "password".into(),
            allow_private: true,
            security: None,
            authentication: None,
        }
    }
    #[tokio::test]
    async fn probes_capabilities_without_sending_credentials_or_mail() {
        for (protocol, greeting, expected, answer) in [
            (
                "imap",
                "* OK ready\r\n",
                "a2 CAPABILITY\r\n",
                "* CAPABILITY IMAP4rev1 AUTH=PLAIN\r\na2 OK done\r\n",
            ),
            (
                "smtp",
                "220 fixture ESMTP\r\n",
                "EHLO mailwoman.invalid\r\n",
                "250-fixture\r\n250 AUTH PLAIN\r\n",
            ),
            (
                "pop3",
                "+OK ready\r\n",
                "CAPA\r\n",
                "+OK list\r\nUSER\r\nUIDL\r\n.\r\n",
            ),
        ] {
            let (client, server) = tokio::io::duplex(4096);
            let peer = tokio::spawn(async move {
                let mut server = BufReader::new(server);
                server
                    .get_mut()
                    .write_all(greeting.as_bytes())
                    .await
                    .unwrap();
                let mut command = String::new();
                server.read_line(&mut command).await.unwrap();
                assert_eq!(command, expected);
                server.get_mut().write_all(answer.as_bytes()).await.unwrap();
                let mut rest = String::new();
                server.read_to_string(&mut rest).await.unwrap();
                assert!(rest.is_empty(), "no AUTH/LOGIN/MAIL commands");
            });
            let mut result = Probe::default();
            handshake(
                protocol,
                &endpoint(TlsMode::None),
                Box::new(client),
                &mut result,
            )
            .await
            .unwrap();
            assert!(!result.capabilities.is_empty());
            assert!(result.tls_version.is_none());
            peer.await.unwrap();
        }
    }
    #[tokio::test]
    async fn refuses_starttls_downgrade_and_never_attempts_auth() {
        let (client, server) = tokio::io::duplex(4096);
        let peer = tokio::spawn(async move {
            let mut server = BufReader::new(server);
            server
                .get_mut()
                .write_all(b"220 fixture\r\n")
                .await
                .unwrap();
            let mut command = String::new();
            server.read_line(&mut command).await.unwrap();
            assert_eq!(command, "EHLO mailwoman.invalid\r\n");
            server
                .get_mut()
                .write_all(b"250 AUTH PLAIN\r\n")
                .await
                .unwrap();
            let mut rest = String::new();
            server.read_to_string(&mut rest).await.unwrap();
            assert!(rest.is_empty());
        });
        let error = handshake(
            "smtp",
            &endpoint(TlsMode::Starttls),
            Box::new(client),
            &mut Probe::default(),
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("STARTTLS"));
        peer.await.unwrap();
    }
    #[tokio::test]
    async fn refuses_oversized_greetings_and_wrong_imap_tags() {
        let (client, mut server) = tokio::io::duplex(8192);
        server.write_all(&vec![b'A'; 4097]).await.unwrap();
        drop(server);
        assert!(line(&mut BufReader::new(Box::new(client))).await.is_err());
        let (client, mut server) = tokio::io::duplex(128);
        server.write_all(b"a20 OK wrong tag\r\n").await.unwrap();
        drop(server);
        assert!(
            reply(&mut BufReader::new(Box::new(client)), "imap", "A2")
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn implicit_tls_refuses_an_untrusted_certificate() {
        use rustls_pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
        let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/crypto/smime/e41");
        let cert =
            CertificateDer::from_pem_slice(&std::fs::read(fixtures.join("ecc.pem")).unwrap())
                .unwrap();
        let key = PrivateKeyDer::from_pem_slice(&std::fs::read(fixtures.join("ecc.key")).unwrap())
            .unwrap();
        let config = rustls::ServerConfig::builder_with_provider(std::sync::Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .unwrap();
        let (client, server) = tokio::io::duplex(16384);
        let peer = tokio::spawn(async move {
            tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(config))
                .accept(server)
                .await
                .is_err()
        });
        let mut result = Probe::default();
        assert!(
            handshake(
                "imap",
                &endpoint(TlsMode::Implicit),
                Box::new(client),
                &mut result
            )
            .await
            .is_err()
        );
        assert!(result.tls_version.is_none());
        assert!(peer.await.unwrap());
    }
    #[test]
    fn throttles_each_admin_session_and_kind_independently() {
        let session = format!("unit-probe-{}", uuid::Uuid::new_v4());
        for _ in 0..6 {
            limit(&session, "upstream").unwrap();
        }
        assert_eq!(
            limit(&session, "upstream").unwrap_err().status,
            StatusCode::TOO_MANY_REQUESTS
        );
        assert!(limit(&session, "dns").is_ok());
        assert!(limit(&(session + "other"), "upstream").is_ok());
    }
}
