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
use crate::sasl;

/// A TLS session plus its `tls-server-end-point` channel binding (RFC 5929),
/// used by SCRAM-SHA-256-PLUS. The binding is `None` when the server presented
/// no leaf certificate or it could not be parsed (SCRAM-`PLUS` is then skipped
/// and plain SCRAM over the TLS channel is used instead).
pub(crate) struct TlsUpgrade {
    pub stream: TlsStream<TcpStream>,
    pub channel_binding: Option<Vec<u8>>,
}

/// Wrap an established TCP stream in a TLS session validated against
/// `webpki-roots`, using `host` as the SNI / certificate name. After the
/// handshake the peer's leaf certificate is hashed into the
/// `tls-server-end-point` channel binding so the caller can negotiate
/// SCRAM-SHA-256-PLUS.
pub(crate) async fn connect(tcp: TcpStream, host: &str) -> Result<TlsUpgrade, SmtpError> {
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

    let stream = TlsConnector::from(Arc::new(config))
        .connect(server_name, tcp)
        .await
        .map_err(|e| SmtpError::Transport(format!("tls handshake: {e}")))?;

    // Extract the leaf certificate (first in the peer chain) and compute the
    // `tls-server-end-point` channel binding for SCRAM-SHA-256-PLUS. Absent or
    // unparseable → `None` (plain SCRAM is used instead).
    let channel_binding = stream
        .get_ref()
        .1
        .peer_certificates()
        .and_then(|chain| chain.first())
        .and_then(|leaf| sasl::tls_server_end_point(leaf.as_ref()));

    Ok(TlsUpgrade {
        stream,
        channel_binding,
    })
}
