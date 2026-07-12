//! TLS setup for implicit-TLS (465) and post-`STARTTLS` (587) upgrade.
//!
//! Uses `tokio-rustls` with the `ring` provider (plan §1: default-features off,
//! `ring` + `tls12`) and the compiled-in Mozilla root set (`webpki-roots`), so
//! no system trust-store dependency and no OpenSSL. Not exercised by the mock
//! unit tests (those run in cleartext); covered by the env-gated live test.

use std::sync::Arc;

use rustls_pki_types::ServerName;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};

use crate::SmtpError;

/// Wrap an established TCP stream in a TLS session validated against
/// `webpki-roots`, using `host` as the SNI / certificate name.
pub(crate) async fn connect(tcp: TcpStream, host: &str) -> Result<TlsStream<TcpStream>, SmtpError> {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    // Pin the ring provider explicitly rather than relying on a process-global
    // default being installed (the crate may be embedded in a host that never
    // installs one).
    let provider = Arc::new(tokio_rustls::rustls::crypto::ring::default_provider());
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| SmtpError::Transport(format!("tls config: {e}")))?
        .with_root_certificates(roots)
        .with_no_client_auth();

    let server_name = ServerName::try_from(host.to_string())
        .map_err(|e| SmtpError::Transport(format!("invalid server name {host:?}: {e}")))?;

    TlsConnector::from(Arc::new(config))
        .connect(server_name, tcp)
        .await
        .map_err(|e| SmtpError::Transport(format!("tls handshake: {e}")))
}
