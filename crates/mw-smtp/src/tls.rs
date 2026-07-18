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
use tokio_rustls::rustls::{ClientConfig, ProtocolVersion, RootCertStore};

use crate::SmtpError;
use crate::sasl;

/// A TLS session plus its SCRAM-SHA-256-PLUS channel binding: the SASL cb-name
/// paired with the raw binding bytes. On **TLS 1.3** this is the RFC 9266
/// `tls-exporter` binding (`export_keying_material`); on **TLS 1.2** it is the
/// RFC 5929 `tls-server-end-point` (leaf-certificate hash). The binding is
/// `None` when it could not be computed (e.g. a TLS-1.2 server presented no
/// parseable leaf certificate), in which case SCRAM-`PLUS` is skipped and plain
/// SCRAM over the TLS channel is used instead.
pub(crate) struct TlsUpgrade {
    pub stream: TlsStream<TcpStream>,
    pub channel_binding: Option<(&'static str, Vec<u8>)>,
}

/// Wrap an established TCP stream in a TLS session validated against
/// `webpki-roots`, using `host` as the SNI / certificate name. After the
/// handshake the negotiated channel binding for SCRAM-SHA-256-PLUS is computed:
/// the RFC 9266 `tls-exporter` on TLS 1.3, or the RFC 5929
/// `tls-server-end-point` (leaf-certificate hash) on TLS 1.2.
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

    // Compute the SCRAM-SHA-256-PLUS channel binding, preferring the RFC 9266
    // `tls-exporter` on TLS 1.3 and falling back to the RFC 5929
    // `tls-server-end-point` (leaf-certificate hash) on TLS 1.2. `tls-exporter`
    // is derived from the connection's exported keying material — independent of
    // the certificate — and is the type real servers (Dovecot 2.4.x) implement
    // over TLS 1.3.
    let conn = stream.get_ref().1;
    let channel_binding = if conn.protocol_version() == Some(ProtocolVersion::TLSv1_3) {
        // RFC 9266 §3: label "EXPORTER-Channel-Binding", empty context, 32-byte
        // output. A failure to export → `None` (plain SCRAM is used instead).
        let mut material = [0u8; 32];
        match conn.export_keying_material(&mut material[..], b"EXPORTER-Channel-Binding", Some(&[]))
        {
            Ok(_) => Some(("tls-exporter", material.to_vec())),
            Err(_) => None,
        }
    } else {
        // TLS 1.2 (or an unknown version): hash the leaf certificate. Absent or
        // unparseable → `None`.
        conn.peer_certificates()
            .and_then(|chain| chain.first())
            .and_then(|leaf| sasl::tls_server_end_point(leaf.as_ref()))
            .map(|bytes| ("tls-server-end-point", bytes))
    };

    Ok(TlsUpgrade {
        stream,
        channel_binding,
    })
}
