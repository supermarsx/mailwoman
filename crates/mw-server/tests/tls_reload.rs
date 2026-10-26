//! End-to-end proof that the external-cert TLS listener terminates TLS and
//! **hot-reloads** the served certificate (plan §3 e10 acceptance; live ACME is
//! manual/nightly). A self-signed pair is staged into temp files, served through
//! a real loopback handshake, then swapped and reloaded — a second handshake must
//! present the new certificate.

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

mod common;
use common::test_db;

use mw_server::{TlsConfig, TlsListener};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/tls")
        .join(name)
}

/// A client verifier that trusts any certificate — the test asserts on the
/// *identity* of the presented cert, not on a chain of trust.
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

/// Complete a TLS handshake to `addr` and return the leaf certificate DER the
/// server presented.
async fn handshake_leaf(addr: std::net::SocketAddr) -> Vec<u8> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .unwrap()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(TrustAny(provider)))
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    let tcp = TcpStream::connect(addr).await.unwrap();
    let name = ServerName::try_from("localhost").unwrap();
    let tls = connector.connect(name, tcp).await.unwrap();
    let (_io, conn) = tls.get_ref();
    conn.peer_certificates().unwrap()[0].as_ref().to_vec()
}

#[tokio::test]
async fn external_cert_listener_serves_and_hot_reloads() {
    // Stage cert1 into temp files.
    let dir = test_db::unique_dir("mw-tls-e2e");
    let cert = dir.join("cert.pem");
    let key = dir.join("key.pem");
    std::fs::copy(fixture("cert1.pem"), &cert).unwrap();
    std::fs::copy(fixture("key1.pem"), &key).unwrap();

    let app = Router::new().route("/healthz", get(|| async { "ok" }));
    let (listener, resolver) = TlsListener::bind(
        "127.0.0.1:0",
        &TlsConfig::External {
            cert: cert.clone(),
            key: key.clone(),
        },
    )
    .await
    .unwrap();
    let resolver = resolver.expect("external mode exposes a reloadable resolver");
    let addr = {
        use axum::serve::Listener;
        listener.local_addr().unwrap()
    };
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // First handshake presents cert1.
    let leaf1 = handshake_leaf(addr).await;
    assert_eq!(leaf1, resolver.current_leaf_der());

    // Swap the files to cert2 and hot-reload (what the SIGHUP handler triggers).
    std::fs::copy(fixture("cert2.pem"), &cert).unwrap();
    std::fs::copy(fixture("key2.pem"), &key).unwrap();
    resolver.reload().unwrap();

    // A fresh handshake now presents cert2 — no listener restart.
    let leaf2 = handshake_leaf(addr).await;
    assert_eq!(leaf2, resolver.current_leaf_der());
    assert_ne!(
        leaf1, leaf2,
        "hot reload must change the served certificate"
    );
}

#[tokio::test]
async fn silent_tls_peer_does_not_block_a_second_valid_handshake() {
    let (listener, _) = TlsListener::bind(
        "127.0.0.1:0",
        &TlsConfig::External {
            cert: fixture("cert1.pem"),
            key: fixture("key1.pem"),
        },
    )
    .await
    .unwrap();
    let addr = axum::serve::Listener::local_addr(&listener).unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, Router::new()).await.unwrap();
    });
    let stalled = TcpStream::connect(addr).await.unwrap();
    // Ensure the first accepted connection has reached the handshake path.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let result =
        tokio::time::timeout(std::time::Duration::from_secs(2), handshake_leaf(addr)).await;
    server.abort();
    drop(stalled);
    assert!(
        result.is_ok(),
        "a silent first TLS peer must not block other handshakes"
    );
}

#[tokio::test]
async fn saturated_tls_handshake_slots_recover_after_deadline() {
    let limits = mw_server::tls::inbound_limits::InboundLimits {
        tls_timeout: std::time::Duration::from_secs(1),
        tls_handshakes: 1,
        ..Default::default()
    };
    let (listener, _) = TlsListener::bind_with_limits(
        "127.0.0.1:0",
        &TlsConfig::External {
            cert: fixture("cert1.pem"),
            key: fixture("key1.pem"),
        },
        limits,
    )
    .await
    .unwrap();
    let addr = axum::serve::Listener::local_addr(&listener).unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, Router::new()).await.unwrap();
    });
    let mut stalled = TcpStream::connect(addr).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let started = std::time::Instant::now();
    let result =
        tokio::time::timeout(std::time::Duration::from_secs(3), handshake_leaf(addr)).await;
    assert!(
        started.elapsed() >= std::time::Duration::from_millis(600),
        "one configured slot must bound concurrent handshakes"
    );
    server.abort();
    assert!(result.is_ok(), "expired handshake must free the sole slot");
    use tokio::io::AsyncReadExt;
    let mut byte = [0];
    assert_eq!(
        tokio::time::timeout(std::time::Duration::from_secs(1), stalled.read(&mut byte))
            .await
            .unwrap()
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn dropping_tls_listener_closes_pending_handshake_sockets() {
    let (listener, _) = TlsListener::bind(
        "127.0.0.1:0",
        &TlsConfig::External {
            cert: fixture("cert1.pem"),
            key: fixture("key1.pem"),
        },
    )
    .await
    .unwrap();
    let addr = axum::serve::Listener::local_addr(&listener).unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, Router::new()).await.unwrap();
    });
    let mut stalled = TcpStream::connect(addr).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    server.abort();
    let _ = server.await;
    use tokio::io::AsyncReadExt;
    let mut byte = [0];
    assert_eq!(
        tokio::time::timeout(std::time::Duration::from_secs(1), stalled.read(&mut byte))
            .await
            .unwrap()
            .unwrap(),
        0
    );
}
