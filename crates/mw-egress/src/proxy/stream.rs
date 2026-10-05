//! The tunnel byte pipe, and the origin TLS that runs **inside** it.
//!
//! # Why TLS is ours
//! Once `CONNECT`/SOCKS5 has opened the tunnel, the proxy is a byte pipe. We then
//! run [`tokio_rustls`] over it with `ServerName::DnsName(origin_host)` — the
//! **origin's** name, the same Mozilla root set and the same `ring` provider a
//! direct fetch uses. The proxy therefore sees ciphertext, and a proxy that
//! substitutes its own peer produces a **certificate failure**, not a silent
//! substitution.
//!
//! There is no `danger_accept_invalid_certs`, no custom verifier and no way to
//! inject a root store: [`connector`] takes no arguments precisely so no call site
//! and no configuration flag can weaken it. A change that adds a parameter here is
//! the change to reject in review. (The one function in this file that takes a
//! root store, `build_config`, is private; outside this file's tests its only
//! caller passes the webpki roots.)
//!
//! # Minimum TLS version
//! The one setting this module has is a process-wide floor ([`set_min_tls`] /
//! [`min_tls`]). rustls 0.23 speaks TLS 1.2 and 1.3 only, so the floor is one of
//! two versions rustls supports: it can make the client stricter (1.3 only) and
//! cannot disable or loosen certificate verification. It defaults to
//! [`MinTls::V12`], which is the configuration this module built before the floor
//! existed. It is read for each tunnel, so a change applies to the next one.

use std::io;
use std::pin::Pin;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll};

use rustls::ClientConfig;
use rustls_pki_types::ServerName;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use super::ProxyRefusal;

/// The lowest TLS version the origin TLS inside a proxy tunnel may negotiate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinTls {
    /// TLS 1.2 or 1.3 (the default; rustls offers nothing older).
    V12,
    /// TLS 1.3 only. A server limited to TLS 1.2 is refused at the handshake.
    V13,
}

static MIN_TLS: AtomicU8 = AtomicU8::new(0);

/// Set the floor for every tunnel wrapped in TLS from now on. Tunnels already
/// established are not touched.
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
/// inside [`ProxyRefusal::OriginTls`].
pub const MIN_TLS_REFUSED: &str = "the minimum TLS version is set to 1.3";

/// Describe a handshake error, naming the floor when the floor explains it. Under a 1.3 floor the
/// client offers TLS 1.3 only; a peer without it answers with a
/// `protocol_version` alert, or with a ServerHello rustls rejects as
/// incompatible. Every other error, and every error under the 1.2 floor, is
/// described by its own text.
fn explain(e: io::Error, floor: MinTls) -> String {
    if floor != MinTls::V13 {
        return e.to_string();
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
        return e.to_string();
    }
    format!("{MIN_TLS_REFUSED} and the server did not offer it ({e})")
}

/// A client configuration for `roots` at `floor`: explicit `ring` provider, no
/// client certificate, rustls's own verifier. `MinTls::V12` is the rustls safe
/// default (1.2 and 1.3). Private — see the module comment.
fn build_config(roots: rustls::RootCertStore, floor: MinTls) -> Arc<ClientConfig> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = ClientConfig::builder_with_provider(provider);
    let builder = match floor {
        MinTls::V12 => builder
            .with_safe_default_protocol_versions()
            .expect("ring provider supports the safe default protocol versions"),
        MinTls::V13 => builder
            .with_protocol_versions(&[&rustls::version::TLS13])
            .expect("ring provider supports TLS 1.3"),
    };
    Arc::new(builder.with_root_certificates(roots).with_no_client_auth())
}

/// The shared client configuration for `floor`: Mozilla webpki roots, explicit
/// `ring` provider. Mirrors `mw-imap`/`mw-sieve`/`mw-smtp` so the tunnelled fetch
/// and every other TLS client in the tree trust the same set. One configuration
/// is built per floor and kept, so changing the floor never serves a
/// configuration built for the other one.
fn client_config(floor: MinTls) -> Arc<ClientConfig> {
    static V12: OnceLock<Arc<ClientConfig>> = OnceLock::new();
    static V13: OnceLock<Arc<ClientConfig>> = OnceLock::new();
    let cell = match floor {
        MinTls::V12 => &V12,
        MinTls::V13 => &V13,
    };
    cell.get_or_init(|| {
        let mut roots = rustls::RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        build_config(roots, floor)
    })
    .clone()
}

/// A [`TlsConnector`] over [`client_config`] at the floor in force. Takes no
/// arguments: verification is not configurable from any call site.
pub fn connector() -> TlsConnector {
    TlsConnector::from(client_config(min_tls()))
}

/// Either side of the one decision the tunnel makes: plaintext (only with a route's
/// explicit `allow_plaintext`) or TLS terminated by us against the origin's name.
pub enum TunnelStream {
    /// Plaintext through the tunnel. The proxy can read and rewrite everything —
    /// which is why this needs a per-route opt-in (plan OQ-3).
    Plain(TcpStream),
    /// TLS we terminate ourselves; the proxy sees ciphertext only.
    Tls(Box<tokio_rustls::client::TlsStream<TcpStream>>),
}

impl TunnelStream {
    /// Whether this pipe is protected end to end from the proxy.
    pub fn is_encrypted(&self) -> bool {
        matches!(self, TunnelStream::Tls(_))
    }
}

impl AsyncRead for TunnelStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.get_mut() {
            TunnelStream::Plain(s) => Pin::new(s).poll_read(cx, buf),
            TunnelStream::Tls(s) => Pin::new(s.as_mut()).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TunnelStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            TunnelStream::Plain(s) => Pin::new(s).poll_write(cx, buf),
            TunnelStream::Tls(s) => Pin::new(s.as_mut()).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            TunnelStream::Plain(s) => Pin::new(s).poll_flush(cx),
            TunnelStream::Tls(s) => Pin::new(s.as_mut()).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            TunnelStream::Plain(s) => Pin::new(s).poll_shutdown(cx),
            TunnelStream::Tls(s) => Pin::new(s.as_mut()).poll_shutdown(cx),
        }
    }
}

/// Wrap an open tunnel in TLS verified against `origin_host`.
///
/// `origin_host` is the **hostname from the URL**, never the proxy's and never the
/// IP literal we handed the proxy. Verifying against the literal would trade the
/// SSRF hole for a TLS hole (plan §4.5), so an IP-literal origin is verified as an
/// IP address and a named origin as that name — both against the same roots.
///
/// The handshake runs at the floor in force ([`min_tls`]). An origin refused
/// because it did not offer TLS 1.3 under a [`MinTls::V13`] floor is an
/// `OriginTls` refusal whose text starts with [`MIN_TLS_REFUSED`].
pub async fn wrap_tls(tcp: TcpStream, origin_host: &str) -> Result<TunnelStream, ProxyRefusal> {
    let floor = min_tls();
    handshake(
        TlsConnector::from(client_config(floor)),
        floor,
        tcp,
        origin_host,
    )
    .await
}

/// [`wrap_tls`] over a given connector. `floor` is the floor `connector` was built
/// for; it is used only to describe a failure.
async fn handshake(
    connector: TlsConnector,
    floor: MinTls,
    tcp: TcpStream,
    origin_host: &str,
) -> Result<TunnelStream, ProxyRefusal> {
    // `Url::host_str` brackets an IPv6 literal (`[2606:2800::1]`) but `ServerName`
    // wants the bare address. Stripping is safe: a DNS name can never begin with
    // `[`, so this branch is reachable only for a v6 literal.
    let bare = origin_host
        .strip_prefix('[')
        .and_then(|h| h.strip_suffix(']'))
        .unwrap_or(origin_host);
    let server_name = ServerName::try_from(bare.to_string()).map_err(|_| {
        ProxyRefusal::Origin(crate::Refusal::BadRequest(
            "origin host is not a valid TLS server name",
        ))
    })?;
    let stream = connector
        .connect(server_name, tcp)
        .await
        // The rustls error text names the failing check (`InvalidCertificate(...)`,
        // `NotValidForName`, …) and contains no request data, so it is safe to keep
        // for the audit trail.
        .map_err(|e| ProxyRefusal::OriginTls(explain(e, floor)))?;
    Ok(TunnelStream::Tls(Box::new(stream)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_connector_takes_no_configuration() {
        // A compile-time statement of the property: `connector()` is a nullary
        // function, so there is no parameter through which a call site could hand
        // in a permissive verifier or an extra root. If this stops compiling
        // because someone added an argument, that is the review signal.
        let f: fn() -> TlsConnector = connector;
        let _ = f();
    }

    #[test]
    fn client_config_is_shared_and_carries_the_webpki_roots() {
        let a = client_config(MinTls::V12);
        let b = client_config(MinTls::V12);
        assert!(Arc::ptr_eq(&a, &b), "the config is built once per floor");
        assert!(
            !Arc::ptr_eq(&a, &client_config(MinTls::V13)),
            "the two floors never share a configuration"
        );
        // A non-empty root set is what makes an untrusted peer fail. If this were
        // empty, every certificate would fail for the wrong reason and the MITM
        // assertion would pass vacuously.
        assert!(
            !webpki_roots::TLS_SERVER_ROOTS.is_empty(),
            "webpki root set must not be empty"
        );
    }

    #[tokio::test]
    async fn a_host_that_is_not_a_valid_server_name_is_refused_before_dialling() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { while listener.accept().await.is_ok() {} });
        let tcp = TcpStream::connect(addr).await.unwrap();
        let Err(err) = wrap_tls(tcp, "not a host name").await else {
            panic!("a hostname that is not a valid TLS server name must be refused");
        };
        assert!(matches!(err, ProxyRefusal::Origin(_)), "{err:?}");
    }

    /// A loopback TLS origin for `localhost` that accepts only the given protocol
    /// versions, and the root store that trusts exactly its certificate.
    async fn tls_origin(
        versions: &'static [&'static rustls::SupportedProtocolVersion],
    ) -> (std::net::SocketAddr, rustls::RootCertStore) {
        use tokio::io::AsyncReadExt;
        let issued = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
        let cert = issued.cert.der().clone();
        let key =
            rustls_pki_types::PrivateKeyDer::try_from(issued.signing_key.serialize_der()).unwrap();
        let mut roots = rustls::RootCertStore::empty();
        roots.add(cert.clone()).unwrap();
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = rustls::ServerConfig::builder_with_provider(provider)
            .with_protocol_versions(versions)
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .unwrap();
        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
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
        (addr, roots)
    }

    /// The protocol versions of a peer that has not been upgraded past TLS 1.2.
    static ONLY_TLS12: &[&rustls::SupportedProtocolVersion] = &[&rustls::version::TLS12];

    /// One origin handshake through this module's `handshake` at the floor in
    /// force, trusting `roots` in place of the webpki set. Returns the negotiated
    /// version.
    async fn origin_handshake(
        addr: std::net::SocketAddr,
        roots: &rustls::RootCertStore,
    ) -> Result<rustls::ProtocolVersion, ProxyRefusal> {
        let floor = min_tls();
        let connector = TlsConnector::from(build_config(roots.clone(), floor));
        let tcp = TcpStream::connect(addr).await.unwrap();
        match handshake(connector, floor, tcp, "localhost").await? {
            TunnelStream::Tls(tls) => Ok(tls.get_ref().1.protocol_version().unwrap()),
            TunnelStream::Plain(_) => panic!("handshake returns a TLS stream"),
        }
    }

    /// The floor is process-wide, so every assertion that moves it lives in this
    /// one test. The other tests in this crate that reach `wrap_tls` expect a
    /// certificate failure from a peer that speaks TLS 1.3, which the floor does
    /// not change.
    #[tokio::test]
    async fn a_tls13_floor_refuses_a_tls12_only_origin_and_names_the_reason() {
        let (only12, roots12) = tls_origin(ONLY_TLS12).await;
        let (both, roots_both) = tls_origin(rustls::ALL_VERSIONS).await;

        // Precondition: unset, the floor is 1.2 and the 1.2-only origin is
        // reachable through this harness.
        assert_eq!(min_tls(), MinTls::V12);
        assert_eq!(
            origin_handshake(only12, &roots12).await.unwrap(),
            rustls::ProtocolVersion::TLSv1_2
        );

        set_min_tls(MinTls::V13);
        assert_eq!(min_tls(), MinTls::V13);
        match origin_handshake(only12, &roots12).await {
            Err(ProxyRefusal::OriginTls(reason)) => assert!(
                reason.starts_with(MIN_TLS_REFUSED),
                "the refusal names the floor: {reason}"
            ),
            other => panic!("a 1.2-only origin must be refused under a 1.3 floor: {other:?}"),
        }
        // The floor refuses a version, not every origin.
        assert_eq!(
            origin_handshake(both, &roots_both).await.unwrap(),
            rustls::ProtocolVersion::TLSv1_3
        );
        // The floor does not touch verification: an origin whose certificate is
        // not trusted still fails on the certificate, and is not blamed on the
        // floor.
        match origin_handshake(both, &roots12).await {
            Err(ProxyRefusal::OriginTls(reason)) => assert!(
                !reason.starts_with(MIN_TLS_REFUSED) && reason.contains("certificate"),
                "{reason}"
            ),
            other => panic!("an untrusted certificate must be refused: {other:?}"),
        }

        // Lowering it again restores the connection.
        set_min_tls(MinTls::V12);
        assert_eq!(
            origin_handshake(only12, &roots12).await.unwrap(),
            rustls::ProtocolVersion::TLSv1_2
        );
    }
}
