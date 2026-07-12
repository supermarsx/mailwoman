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
    let dir = std::env::temp_dir().join(format!("mw-tls-e2e-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
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
