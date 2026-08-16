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
//! the change to reject in review.

use std::io;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll};

use rustls::ClientConfig;
use rustls_pki_types::ServerName;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use super::ProxyRefusal;

/// The shared client configuration: Mozilla webpki roots, explicit `ring` provider,
/// safe default protocol versions. Mirrors `mw-imap`/`mw-sieve`/`mw-smtp` so the
/// tunnelled fetch and every other TLS client in the tree trust the same set.
fn client_config() -> Arc<ClientConfig> {
    static CONFIG: OnceLock<Arc<ClientConfig>> = OnceLock::new();
    CONFIG
        .get_or_init(|| {
            let mut roots = rustls::RootCertStore::empty();
            roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            let provider = Arc::new(rustls::crypto::ring::default_provider());
            let config = ClientConfig::builder_with_provider(provider)
                .with_safe_default_protocol_versions()
                .expect("ring provider supports the safe default protocol versions")
                .with_root_certificates(roots)
                .with_no_client_auth();
            Arc::new(config)
        })
        .clone()
}

/// A [`TlsConnector`] over [`client_config`]. Takes no arguments: verification is
/// not configurable from any call site.
pub fn connector() -> TlsConnector {
    TlsConnector::from(client_config())
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
pub async fn wrap_tls(tcp: TcpStream, origin_host: &str) -> Result<TunnelStream, ProxyRefusal> {
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
    let stream = connector()
        .connect(server_name, tcp)
        .await
        // The rustls error text names the failing check (`InvalidCertificate(...)`,
        // `NotValidForName`, …) and contains no request data, so it is safe to keep
        // for the audit trail.
        .map_err(|e| ProxyRefusal::OriginTls(e.to_string()))?;
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
        let a = client_config();
        let b = client_config();
        assert!(Arc::ptr_eq(&a, &b), "the config is built once");
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
}
