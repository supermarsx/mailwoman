//! The minimum-TLS-version floor (`mw_server::tls::set_min_tls` /
//! `apply_min_tls`), driven over real loopback sockets.
//!
//! Inbound: a real [`TlsListener`] served by `axum::serve`, and rustls clients
//! pinned to one protocol version each. Outbound: a rustls **server** pinned to
//! TLS 1.2, reached through the public entry points of the connector crates that
//! `apply_min_tls` sets (`mw_pop3`, `mw_egress`). Each case asserts first that the
//! TLS 1.2 peer gets through with the floor at 1.2, then that it is refused once
//! the floor is 1.3, then that lowering the floor lets it through again.
//!
//! The per-connector proof that a full, certificate-verified handshake succeeds
//! at 1.2 and is refused at 1.3 lives beside each connector (`mw-imap`,
//! `mw-smtp`, `mw-sieve` `src/tls.rs`; `mw-pop3` `src/conn.rs`; `mw-egress`
//! `src/proxy/stream.rs`). Here the outbound peer's certificate is not trusted by
//! the webpki roots the public entry points use, so "got through" is observed as
//! the handshake reaching certificate verification, which happens only after the
//! protocol version has been agreed.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::routing::get;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme, SupportedProtocolVersion};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;

use mw_server::tls::{self, MinTls};
use mw_server::{TlsConfig, TlsListener};

/// The floor is process-wide; the tests in this binary take turns.
static FLOOR: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

static ONLY_TLS12: &[&SupportedProtocolVersion] = &[&rustls::version::TLS12];
static ONLY_TLS13: &[&SupportedProtocolVersion] = &[&rustls::version::TLS13];

/// What every refusal caused by the floor starts with, on every connector.
const REFUSED: &str = "the minimum TLS version is set to 1.3";

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/tls")
        .join(name)
}

/// A client verifier that trusts any certificate — the inbound cases assert on
/// the protocol version the listener accepts, not on a chain of trust.
#[derive(Debug)]
struct TrustAny(Arc<CryptoProvider>);

impl ServerCertVerifier for TrustAny {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

/// Handshake with the listener at `addr` as a client that speaks only `versions`.
async fn client_handshake(
    addr: SocketAddr,
    versions: &'static [&'static SupportedProtocolVersion],
) -> std::io::Result<TlsStream<TcpStream>> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_protocol_versions(versions)
        .unwrap()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(TrustAny(provider)))
        .with_no_client_auth();
    let tcp = TcpStream::connect(addr).await?;
    let name = ServerName::try_from("localhost").unwrap();
    let mut tls = TlsConnector::from(Arc::new(config))
        .connect(name, tcp)
        .await?;
    // A TLS 1.3 client finishes its side before the server has answered, so a
    // refusal can arrive after `connect` returns. One request/response round trip
    // makes the outcome the same for both versions.
    healthz(&mut tls, false).await?;
    Ok(tls)
}

/// `GET /healthz` over an established session; the body must be `ok`.
async fn healthz(tls: &mut TlsStream<TcpStream>, close: bool) -> std::io::Result<()> {
    let connection = if close { "close" } else { "keep-alive" };
    tls.write_all(
        format!("GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: {connection}\r\n\r\n")
            .as_bytes(),
    )
    .await?;
    let mut seen = Vec::new();
    let mut buf = [0u8; 1024];
    loop {
        let n = tls.read(&mut buf).await?;
        if n == 0 {
            return Err(std::io::Error::other(format!(
                "connection closed before the response: {:?}",
                String::from_utf8_lossy(&seen)
            )));
        }
        seen.extend_from_slice(&buf[..n]);
        let text = String::from_utf8_lossy(&seen);
        if text.starts_with("HTTP/1.1 200") && text.ends_with("\r\n\r\nok") {
            return Ok(());
        }
    }
}

#[tokio::test]
async fn the_listener_refuses_new_tls12_clients_once_the_floor_is_13() {
    let _turn = FLOOR.lock().await;
    tls::set_min_tls(MinTls::V12);

    let app = Router::new().route("/healthz", get(|| async { "ok" }));
    let (listener, _resolver) = TlsListener::bind(
        "127.0.0.1:0",
        &TlsConfig::External {
            cert: fixture("cert1.pem"),
            key: fixture("key1.pem"),
        },
    )
    .await
    .unwrap();
    let addr = {
        use axum::serve::Listener;
        listener.local_addr().unwrap()
    };
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Precondition: with the floor at 1.2 a client that speaks only TLS 1.2 is
    // served, and so is one that speaks only 1.3.
    assert_eq!(tls::min_tls(), MinTls::V12);
    let mut established = client_handshake(addr, ONLY_TLS12)
        .await
        .expect("a TLS 1.2 client is served at the default floor");
    assert_eq!(
        established.get_ref().1.protocol_version(),
        Some(rustls::ProtocolVersion::TLSv1_2)
    );
    client_handshake(addr, ONLY_TLS13)
        .await
        .expect("a TLS 1.3 client is served at the default floor");

    // Raise the floor: no rebind, same listener.
    tls::set_min_tls(MinTls::V13);
    let refused = client_handshake(addr, ONLY_TLS12).await;
    assert!(
        refused.is_err(),
        "a new TLS 1.2 connection must be refused under a 1.3 floor"
    );
    let tls13 = client_handshake(addr, ONLY_TLS13)
        .await
        .expect("a TLS 1.3 client is still served under a 1.3 floor");
    assert_eq!(
        tls13.get_ref().1.protocol_version(),
        Some(rustls::ProtocolVersion::TLSv1_3)
    );
    // The TLS 1.2 session that was already established is not dropped.
    healthz(&mut established, true)
        .await
        .expect("an established session survives the floor being raised");

    // Lower it again: TLS 1.2 clients are served again.
    tls::set_min_tls(MinTls::V12);
    client_handshake(addr, ONLY_TLS12)
        .await
        .expect("lowering the floor restores TLS 1.2 clients");
}

/// A loopback TLS server limited to TLS 1.2, presenting the fixture certificate
/// (which no webpki root vouches for). It speaks no protocol after the
/// handshake; no client in this file gets that far.
async fn tls12_only_upstream() -> SocketAddr {
    let cert = CertificateDer::from_pem_file(fixture("cert1.pem")).unwrap();
    let key = PrivateKeyDer::from_pem_file(fixture("key1.pem")).unwrap();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(ONLY_TLS12)
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .unwrap();
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((tcp, _)) = listener.accept().await {
            let acceptor = acceptor.clone();
            tokio::spawn(async move {
                if let Ok(mut tls) = acceptor.accept(tcp).await {
                    let mut sink = Vec::new();
                    let _ = tls.read_to_end(&mut sink).await;
                }
            });
        }
    });
    addr
}

/// Open a POP3S session to `addr` through the public backend entry point and
/// return the error text (the peer's certificate is never trusted, so there is
/// always one).
async fn pop3_error(addr: SocketAddr) -> String {
    let cfg = mw_pop3::Pop3Config {
        host: "localhost".into(),
        port: addr.port(),
        tls: mw_pop3::TlsMode::Implicit,
        auth: mw_pop3::Pop3Auth::UserPass,
        username: "user".into(),
        secret: "secret".into(),
        leave_policy: mw_pop3::LeavePolicy::default(),
        poll_interval: Duration::from_secs(60),
    };
    match mw_pop3::conn::Pop3Conn::open(&cfg).await {
        Ok(_) => panic!("the test upstream's certificate must not be trusted"),
        Err(e) => e.to_string(),
    }
}

/// Wrap a tunnel to `addr` in origin TLS through the public egress entry point
/// and return the refusal text.
async fn egress_error(addr: SocketAddr) -> String {
    let tcp = TcpStream::connect(addr).await.unwrap();
    match mw_egress::proxy::stream::wrap_tls(tcp, "localhost").await {
        Ok(_) => panic!("the test upstream's certificate must not be trusted"),
        Err(mw_egress::proxy::ProxyRefusal::OriginTls(reason)) => reason,
        Err(other) => panic!("expected an origin TLS refusal, got {other:?}"),
    }
}

#[tokio::test]
async fn apply_min_tls_reaches_the_listener_pop3_and_the_egress_tunnel() {
    let _turn = FLOOR.lock().await;
    tls::apply_min_tls(MinTls::V12);
    let upstream = tls12_only_upstream().await;

    // Precondition: at 1.2 every floor reads 1.2, and the TLS 1.2 upstream is
    // reached as far as certificate verification — the version was agreed and
    // the failure is about the certificate, not about the floor.
    assert_eq!(tls::min_tls(), MinTls::V12);
    assert_eq!(mw_pop3::conn::min_tls(), mw_pop3::conn::MinTls::V12);
    assert_eq!(
        mw_egress::proxy::stream::min_tls(),
        mw_egress::proxy::stream::MinTls::V12
    );
    for before in [pop3_error(upstream).await, egress_error(upstream).await] {
        assert!(
            before.contains("certificate") && !before.contains(REFUSED),
            "at the 1.2 floor the handshake reaches certificate verification: {before}"
        );
    }

    tls::apply_min_tls(MinTls::V13);
    assert_eq!(tls::min_tls(), MinTls::V13);
    assert_eq!(mw_pop3::conn::min_tls(), mw_pop3::conn::MinTls::V13);
    assert_eq!(
        mw_egress::proxy::stream::min_tls(),
        mw_egress::proxy::stream::MinTls::V13
    );
    let pop3 = pop3_error(upstream).await;
    assert!(
        pop3.contains(REFUSED),
        "POP3 names the floor as the reason: {pop3}"
    );
    let egress = egress_error(upstream).await;
    assert!(
        egress.starts_with(REFUSED),
        "the egress tunnel names the floor as the reason: {egress}"
    );

    // Undo: the same upstream is reached as far as its certificate again.
    tls::apply_min_tls(MinTls::V12);
    for after in [pop3_error(upstream).await, egress_error(upstream).await] {
        assert!(
            after.contains("certificate") && !after.contains(REFUSED),
            "lowering the floor restores the connection attempt: {after}"
        );
    }
}
