//! TLS setup for implicit-TLS (465) and post-`STARTTLS` (587) upgrade.
//!
//! Uses `tokio-rustls` with the `ring` provider (plan §1: default-features off,
//! `ring` + `tls12`) and the compiled-in Mozilla root set (`webpki-roots`), so
//! no system trust-store dependency and no OpenSSL. Not exercised by the mock
//! unit tests (those run in cleartext); the handshake tests below run it against a
//! loopback TLS peer, and the env-gated live test against a real server.
//!
//! # Minimum TLS version
//! The crate holds one process-wide floor ([`set_min_tls`] / [`min_tls`]). rustls
//! 0.23 speaks TLS 1.2 and 1.3 only, so the floor is a two-value choice. It
//! defaults to [`MinTls::V12`], which builds exactly the configuration this crate
//! built before the floor existed. The floor is read each time a connection is
//! wrapped, so a change applies to the next submission; a session already open
//! keeps the version it negotiated.

use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use rustls_pki_types::ServerName;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::{ClientConfig, ProtocolVersion, RootCertStore};

use crate::SmtpError;
use crate::sasl;

/// The lowest TLS version an SMTP submission connection may negotiate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinTls {
    /// TLS 1.2 or 1.3 (the default; rustls offers nothing older).
    V12,
    /// TLS 1.3 only. A server limited to TLS 1.2 is refused at the handshake.
    V13,
}

static MIN_TLS: AtomicU8 = AtomicU8::new(0);

/// Set the floor for every SMTP TLS connection opened from now on (implicit TLS
/// and STARTTLS alike). Connections already established are not touched.
pub fn set_min_tls(v: MinTls) {
    MIN_TLS.store(matches!(v, MinTls::V13) as u8, Ordering::SeqCst);
}

/// The floor currently in force.
pub fn min_tls() -> MinTls {
    if MIN_TLS.load(Ordering::SeqCst) == 0 {
        MinTls::V12
    } else {
        MinTls::V13
    }
}

/// The start of the error text produced when a handshake fails because the peer
/// did not offer TLS 1.3 while the floor is [`MinTls::V13`]. It reaches the caller
/// inside `SmtpError::Transport("tls handshake: …")`.
pub const MIN_TLS_REFUSED: &str = "the minimum TLS version is set to 1.3";

/// Rewrite a handshake error that the floor explains. Under a 1.3 floor the
/// client offers TLS 1.3 only; a peer without it answers with a
/// `protocol_version` alert, or with a ServerHello rustls rejects as
/// incompatible. Every other error, and every error under the 1.2 floor, is
/// returned unchanged.
fn explain(e: io::Error, floor: MinTls) -> io::Error {
    if floor != MinTls::V13 {
        return e;
    }
    let caused_by_floor = matches!(
        e.get_ref()
            .and_then(|inner| inner.downcast_ref::<tokio_rustls::rustls::Error>()),
        Some(
            tokio_rustls::rustls::Error::AlertReceived(
                tokio_rustls::rustls::AlertDescription::ProtocolVersion
            ) | tokio_rustls::rustls::Error::PeerIncompatible(_)
        )
    );
    if !caused_by_floor {
        return e;
    }
    io::Error::new(
        e.kind(),
        format!("{MIN_TLS_REFUSED} and the server did not offer it ({e})"),
    )
}

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
/// `webpki-roots`, using `host` as the SNI / certificate name, at the floor
/// currently in force ([`min_tls`]). After the handshake the negotiated channel
/// binding for SCRAM-SHA-256-PLUS is computed: the RFC 9266 `tls-exporter` on
/// TLS 1.3, or the RFC 5929 `tls-server-end-point` (leaf-certificate hash) on
/// TLS 1.2. Under a [`MinTls::V13`] floor only the first can occur.
pub(crate) async fn connect(tcp: TcpStream, host: &str) -> Result<TlsUpgrade, SmtpError> {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    connect_with_roots(tcp, host, roots).await
}

/// [`connect`] with the trust anchors supplied. Private: the only caller outside
/// this module's tests is [`connect`], which passes the webpki roots.
async fn connect_with_roots(
    tcp: TcpStream,
    host: &str,
    roots: RootCertStore,
) -> Result<TlsUpgrade, SmtpError> {
    let floor = min_tls();
    // Pin the ring provider explicitly rather than relying on a process-global
    // default being installed (the crate may be embedded in a host that never
    // installs one).
    let provider = Arc::new(tokio_rustls::rustls::crypto::ring::default_provider());
    let builder = ClientConfig::builder_with_provider(provider);
    let builder = match floor {
        MinTls::V12 => builder.with_safe_default_protocol_versions(),
        MinTls::V13 => builder.with_protocol_versions(&[&tokio_rustls::rustls::version::TLS13]),
    }
    .map_err(|e| SmtpError::Transport(format!("tls config: {e}")))?;
    let config = builder.with_root_certificates(roots).with_no_client_auth();

    let server_name = ServerName::try_from(host.to_string())
        .map_err(|e| SmtpError::Transport(format!("invalid server name {host:?}: {e}")))?;

    let stream = TlsConnector::from(Arc::new(config))
        .connect(server_name, tcp)
        .await
        .map_err(|e| SmtpError::Transport(format!("tls handshake: {}", explain(e, floor))))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use rustls_pki_types::pem::PemObject;
    use rustls_pki_types::{CertificateDer, PrivateKeyDer};
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio_rustls::rustls;

    /// A self-signed certificate for `localhost` (P-256, valid to 2126), used only
    /// as the test peer's identity and as the single trust anchor of the test
    /// client. Not a secret.
    const CERT_PEM: &str = "-----BEGIN CERTIFICATE-----
MIIBojCCAUigAwIBAgIUXZfggy7a6Ac70pTuOuVMCuNzsNEwCgYIKoZIzj0EAwIw
HDEaMBgGA1UEAwwRbXctdGxzLWZsb29yLXRlc3QwIBcNMjYxMDA1MDAzMDA1WhgP
MjEyNjA5MTEwMDMwMDVaMBwxGjAYBgNVBAMMEW13LXRscy1mbG9vci10ZXN0MFkw
EwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAECjpBfVlabTNR7L8L59lw7vc/NaVVGmKn
1tAX8qOht2J5VmI1K0rbmOUevAO5cYmc52ATirpamxCpmne9BQExZaNmMGQwHQYD
VR0OBBYEFBFUy5FE3DLSfLMO3C8ZIQKQS736MB8GA1UdIwQYMBaAFBFUy5FE3DLS
fLMO3C8ZIQKQS736MBQGA1UdEQQNMAuCCWxvY2FsaG9zdDAMBgNVHRMBAf8EAjAA
MAoGCCqGSM49BAMCA0gAMEUCICMiQGvni0u2s5i7LpPBCkPD5bdvJuRtdareNrwa
QNGgAiEAoA+Jfh/SX0fNzYyti/9S7+BJ+t9hS1MrxabN7QUlVaA=
-----END CERTIFICATE-----
";
    const KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQg9JIIL1eAr3QLu3pR
51s9cIWRVry46AolZDGxmGbZs3OhRANCAAQKOkF9WVptM1Hsvwvn2XDu9z81pVUa
YqfW0Bfyo6G3YnlWYjUrStuY5R68A7lxiZznYBOKulqbEKmad70FATFl
-----END PRIVATE KEY-----
";

    fn test_roots() -> RootCertStore {
        let mut roots = RootCertStore::empty();
        roots
            .add(CertificateDer::from_pem_slice(CERT_PEM.as_bytes()).unwrap())
            .unwrap();
        roots
    }

    /// The protocol versions of a peer that has not been upgraded past TLS 1.2.
    static ONLY_TLS12: &[&rustls::SupportedProtocolVersion] = &[&rustls::version::TLS12];

    /// A loopback TLS server that accepts only the given protocol versions. After
    /// each handshake it sends its own RFC 9266 exporter value (32 bytes; zeros
    /// when the session is not TLS 1.3) so the client's binding can be compared
    /// with what the server derived.
    async fn tls_peer(
        versions: &'static [&'static rustls::SupportedProtocolVersion],
    ) -> SocketAddr {
        let cert = CertificateDer::from_pem_slice(CERT_PEM.as_bytes()).unwrap();
        let key = PrivateKeyDer::from_pem_slice(KEY_PEM.as_bytes()).unwrap();
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = rustls::ServerConfig::builder_with_provider(provider)
            .with_protocol_versions(versions)
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
                        let mut exported = [0u8; 32];
                        let conn = tls.get_ref().1;
                        if conn.protocol_version() == Some(ProtocolVersion::TLSv1_3) {
                            conn.export_keying_material(
                                &mut exported[..],
                                b"EXPORTER-Channel-Binding",
                                Some(&[]),
                            )
                            .unwrap();
                        }
                        let _ = tls.write_all(&exported).await;
                        let mut sink = Vec::new();
                        let _ = tls.read_to_end(&mut sink).await;
                    }
                });
            }
        });
        addr
    }

    /// The channel binding of a session, and the exporter value its server sent.
    type Bound = (Option<(&'static str, Vec<u8>)>, [u8; 32]);

    /// One upgrade through this module's `connect` path at the floor in force.
    async fn upgrade(addr: SocketAddr) -> Result<Bound, SmtpError> {
        let tcp = TcpStream::connect(addr).await.unwrap();
        let mut up = connect_with_roots(tcp, "localhost", test_roots()).await?;
        let mut server_exported = [0u8; 32];
        up.stream.read_exact(&mut server_exported).await.unwrap();
        Ok((up.channel_binding, server_exported))
    }

    /// The floor is process-wide, so every assertion that moves it lives in this
    /// one test.
    #[tokio::test]
    async fn a_tls13_floor_refuses_a_tls12_only_server_and_scram_plus_still_binds() {
        let only12 = tls_peer(ONLY_TLS12).await;
        let both = tls_peer(rustls::ALL_VERSIONS).await;
        let leaf = CertificateDer::from_pem_slice(CERT_PEM.as_bytes()).unwrap();
        let end_point = sasl::tls_server_end_point(leaf.as_ref()).unwrap();

        // Precondition: unset, the floor is 1.2 and the 1.2-only peer is reachable
        // through this harness, binding to the certificate hash.
        assert_eq!(min_tls(), MinTls::V12);
        let (binding, _) = upgrade(only12).await.unwrap();
        assert_eq!(binding, Some(("tls-server-end-point", end_point.clone())));

        set_min_tls(MinTls::V13);
        assert_eq!(min_tls(), MinTls::V13);
        let Err(err) = upgrade(only12).await else {
            panic!("a 1.2-only server must be refused under a 1.3 floor");
        };
        let SmtpError::Transport(text) = &err else {
            panic!("expected a transport error, got {err:?}");
        };
        assert!(
            text.starts_with(&format!("tls handshake: {MIN_TLS_REFUSED}")),
            "the error names the floor: {text}"
        );

        // The floor refuses a version, not every server; and under it the
        // SCRAM-SHA-256-PLUS binding is the exporter value the server derived too.
        let (binding, server_exported) = upgrade(both).await.unwrap();
        assert_ne!(server_exported, [0u8; 32], "the peer negotiated TLS 1.3");
        assert_eq!(binding, Some(("tls-exporter", server_exported.to_vec())));

        // Lowering it again restores the connection.
        set_min_tls(MinTls::V12);
        let (binding, _) = upgrade(only12).await.unwrap();
        assert_eq!(binding, Some(("tls-server-end-point", end_point)));
    }

    #[test]
    fn an_unrelated_handshake_error_is_not_blamed_on_the_floor() {
        let cert_error = || {
            io::Error::new(
                io::ErrorKind::InvalidData,
                rustls::Error::InvalidCertificate(rustls::CertificateError::UnknownIssuer),
            )
        };
        let before = cert_error().to_string();
        assert_eq!(explain(cert_error(), MinTls::V13).to_string(), before);
        // Under the 1.2 floor nothing is rewritten, whatever the cause.
        let alert = io::Error::new(
            io::ErrorKind::InvalidData,
            rustls::Error::AlertReceived(rustls::AlertDescription::ProtocolVersion),
        );
        assert!(
            !explain(alert, MinTls::V12)
                .to_string()
                .starts_with(MIN_TLS_REFUSED)
        );
    }
}
