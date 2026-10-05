//! rustls client configuration (ring provider + webpki-roots).
//!
//! Built with an explicit `ring` [`CryptoProvider`] rather than relying on a
//! process-global default, so the crate is self-contained and does not depend
//! on install order elsewhere in the workspace.
//!
//! # Minimum TLS version
//! The crate holds one process-wide floor ([`set_min_tls`] / [`min_tls`]). rustls
//! 0.23 speaks TLS 1.2 and 1.3 only, so the floor is a two-value choice. It
//! defaults to [`MinTls::V12`], which builds exactly the configuration this crate
//! built before the floor existed. The floor is read each time a connection is
//! wrapped, so a change applies to the next connection; sessions already open keep
//! the version they negotiated.

use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use rustls::ClientConfig;
use rustls_pki_types::ServerName;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;

use crate::error::{ImapError, ImapResult};

/// The lowest TLS version an outbound IMAP connection may negotiate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinTls {
    /// TLS 1.2 or 1.3 (the default; rustls offers nothing older).
    V12,
    /// TLS 1.3 only. A server limited to TLS 1.2 is refused at the handshake.
    V13,
}

static MIN_TLS: AtomicU8 = AtomicU8::new(0);

/// Set the floor for every IMAP TLS connection opened from now on (implicit TLS
/// and STARTTLS alike). Connections already established are not touched.
///
/// Not callable from another crate yet: this module is private and `lib.rs`
/// does not re-export it, so today only this module's tests call it and the
/// floor of a running server stays at [`MinTls::V12`]. Hence the `allow`.
#[allow(dead_code)]
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
/// inside `ImapError::Tls("tls handshake: …")`.
pub const MIN_TLS_REFUSED: &str = "the minimum TLS version is set to 1.3";

/// A [`TlsConnector`] together with the floor its configuration was built for, so
/// a handshake the floor caused to fail can be reported as such.
pub(crate) struct Connector {
    inner: TlsConnector,
    floor: MinTls,
}

impl Connector {
    /// Run the TLS handshake over `tcp`. Same contract as
    /// [`TlsConnector::connect`], except that a failure attributable to the floor
    /// carries a message starting with [`MIN_TLS_REFUSED`].
    pub(crate) async fn connect(
        &self,
        server_name: ServerName<'static>,
        tcp: TcpStream,
    ) -> io::Result<TlsStream<TcpStream>> {
        self.inner
            .connect(server_name, tcp)
            .await
            .map_err(|e| explain(e, self.floor))
    }
}

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
            .and_then(|inner| inner.downcast_ref::<rustls::Error>()),
        Some(
            rustls::Error::AlertReceived(rustls::AlertDescription::ProtocolVersion)
                | rustls::Error::PeerIncompatible(_)
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

/// Build a [`Connector`] trusting the Mozilla webpki root set, at the floor
/// currently in force.
pub(crate) fn connector() -> ImapResult<Connector> {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    connector_with_roots(roots)
}

/// [`connector`] with the trust anchors supplied. Private: the only caller
/// outside this module's tests is [`connector`], which passes the webpki roots.
fn connector_with_roots(roots: rustls::RootCertStore) -> ImapResult<Connector> {
    let floor = min_tls();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = ClientConfig::builder_with_provider(provider);
    let builder = match floor {
        MinTls::V12 => builder.with_safe_default_protocol_versions(),
        MinTls::V13 => builder.with_protocol_versions(&[&rustls::version::TLS13]),
    }
    .map_err(|e| ImapError::Tls(format!("rustls protocol setup: {e}")))?;
    let config = builder.with_root_certificates(roots).with_no_client_auth();

    Ok(Connector {
        inner: TlsConnector::from(Arc::new(config)),
        floor,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustls_pki_types::pem::PemObject;
    use rustls_pki_types::{CertificateDer, PrivateKeyDer};
    use std::net::SocketAddr;
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpListener;

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

    fn test_roots() -> rustls::RootCertStore {
        let mut roots = rustls::RootCertStore::empty();
        roots
            .add(CertificateDer::from_pem_slice(CERT_PEM.as_bytes()).unwrap())
            .unwrap();
        roots
    }

    /// The protocol versions of a peer that has not been upgraded past TLS 1.2.
    static ONLY_TLS12: &[&rustls::SupportedProtocolVersion] = &[&rustls::version::TLS12];

    /// A loopback TLS server that accepts only the given protocol versions.
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
                        let mut sink = Vec::new();
                        let _ = tls.read_to_end(&mut sink).await;
                    }
                });
            }
        });
        addr
    }

    /// One handshake through this module's connector at the floor in force,
    /// returning the negotiated version.
    async fn handshake(addr: SocketAddr) -> io::Result<rustls::ProtocolVersion> {
        let connector = connector_with_roots(test_roots()).unwrap();
        let tcp = TcpStream::connect(addr).await.unwrap();
        let name = ServerName::try_from("localhost").unwrap();
        let tls = connector.connect(name, tcp).await?;
        Ok(tls.get_ref().1.protocol_version().unwrap())
    }

    /// The floor is process-wide, so every assertion that moves it lives in this
    /// one test.
    #[tokio::test]
    async fn a_tls13_floor_refuses_a_tls12_only_server_and_names_the_reason() {
        let only12 = tls_peer(ONLY_TLS12).await;
        let both = tls_peer(rustls::ALL_VERSIONS).await;

        // Precondition: unset, the floor is 1.2 and the 1.2-only peer is reachable
        // through this harness.
        assert_eq!(min_tls(), MinTls::V12);
        assert_eq!(
            handshake(only12).await.unwrap(),
            rustls::ProtocolVersion::TLSv1_2
        );

        set_min_tls(MinTls::V13);
        assert_eq!(min_tls(), MinTls::V13);
        let err = handshake(only12)
            .await
            .expect_err("a 1.2-only server must be refused under a 1.3 floor");
        assert!(
            err.to_string().starts_with(MIN_TLS_REFUSED),
            "the error names the floor: {err}"
        );
        // The floor refuses a version, not every server.
        assert_eq!(
            handshake(both).await.unwrap(),
            rustls::ProtocolVersion::TLSv1_3
        );

        // Lowering it again restores the connection.
        set_min_tls(MinTls::V12);
        assert_eq!(
            handshake(only12).await.unwrap(),
            rustls::ProtocolVersion::TLSv1_2
        );
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
