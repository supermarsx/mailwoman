//! TLS termination for `mailwoman serve` (plan §1.10, §3 e10): either
//! ACME-managed certificates via `tokio-rustls-acme` (tls-alpn-01, single port)
//! or an external cert/key pair that **hot-reloads on SIGHUP** without dropping
//! the listener.
//!
//! The listener implements axum's [`axum::serve::Listener`] so the existing
//! `axum::serve(listener, app).with_graceful_shutdown(..)` path is reused for
//! both plaintext and TLS. Live ACME needs public DNS + the Let's Encrypt
//! endpoint, so it is exercised manually/nightly (plan §6 risk 10); the
//! external-cert reload path is what the integration tests drive with a
//! self-signed pair.

use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use anyhow::{Context, anyhow};
use base64::Engine as _;
use rustls::ServerConfig;
use rustls::pki_types::{
    CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer, PrivatePkcs8KeyDer, PrivateSec1KeyDer,
};
use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::server::TlsStream;

use tokio_rustls_acme::caches::DirCache;
use tokio_rustls_acme::{AcmeAcceptor, AcmeConfig};

/// How the server should obtain its certificate.
#[derive(Debug, Clone)]
pub enum TlsConfig {
    /// ACME-managed (Let's Encrypt) certificates via tls-alpn-01.
    Acme {
        domains: Vec<String>,
        contact: Option<String>,
        cache_dir: PathBuf,
        /// Use the Let's Encrypt *staging* directory (avoids rate limits).
        staging: bool,
    },
    /// An operator-provided cert/key pair, reloadable on SIGHUP.
    External { cert: PathBuf, key: PathBuf },
}

// ---------------------------------------------------------------------------
// Hot-reloadable certificate resolver
// ---------------------------------------------------------------------------

/// A [`ResolvesServerCert`] that serves a cert/key loaded from disk and can swap
/// them atomically at runtime (SIGHUP → [`ReloadableResolver::reload`]). Reads on
/// the handshake path take only a short read-lock and clone an `Arc`.
#[derive(Debug)]
pub struct ReloadableResolver {
    cert_path: PathBuf,
    key_path: PathBuf,
    current: RwLock<Arc<CertifiedKey>>,
}

impl ReloadableResolver {
    /// Load the initial cert/key pair from disk.
    pub fn load(cert_path: PathBuf, key_path: PathBuf) -> anyhow::Result<Arc<Self>> {
        let ck = load_certified_key(&cert_path, &key_path)?;
        Ok(Arc::new(Self {
            cert_path,
            key_path,
            current: RwLock::new(Arc::new(ck)),
        }))
    }

    /// Re-read the cert/key files and swap them in. On failure the previously
    /// served pair is left untouched so a bad deploy never takes the listener
    /// down.
    pub fn reload(&self) -> anyhow::Result<()> {
        let ck = load_certified_key(&self.cert_path, &self.key_path)?;
        *self.current.write().expect("resolver lock") = Arc::new(ck);
        Ok(())
    }

    /// The DER of the currently served end-entity certificate (tests assert the
    /// reload actually changed what is served).
    pub fn current_leaf_der(&self) -> Vec<u8> {
        self.current.read().expect("resolver lock").cert[0]
            .as_ref()
            .to_vec()
    }
}

impl ResolvesServerCert for ReloadableResolver {
    fn resolve(&self, _hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        Some(self.current.read().expect("resolver lock").clone())
    }
}

/// Parse a cert chain + private key from PEM files and build a verified
/// [`CertifiedKey`] using the ring provider.
fn load_certified_key(cert_path: &PathBuf, key_path: &PathBuf) -> anyhow::Result<CertifiedKey> {
    let certs = load_certs(cert_path)?;
    if certs.is_empty() {
        return Err(anyhow!("no certificates in {}", cert_path.display()));
    }
    let key = load_key(key_path)?;
    let provider = rustls::crypto::ring::default_provider();
    CertifiedKey::from_der(certs, key, &provider).with_context(|| {
        format!(
            "cert/key in {} do not form a usable pair",
            key_path.display()
        )
    })
}

fn load_certs(path: &PathBuf) -> anyhow::Result<Vec<CertificateDer<'static>>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading certificate {}", path.display()))?;
    Ok(pem_blocks(&text, "CERTIFICATE")
        .into_iter()
        .map(CertificateDer::from)
        .collect())
}

fn load_key(path: &PathBuf) -> anyhow::Result<PrivateKeyDer<'static>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading private key {}", path.display()))?;
    if let Some(der) = pem_blocks(&text, "PRIVATE KEY").into_iter().next() {
        return Ok(PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(der)));
    }
    if let Some(der) = pem_blocks(&text, "EC PRIVATE KEY").into_iter().next() {
        return Ok(PrivateKeyDer::Sec1(PrivateSec1KeyDer::from(der)));
    }
    if let Some(der) = pem_blocks(&text, "RSA PRIVATE KEY").into_iter().next() {
        return Ok(PrivateKeyDer::Pkcs1(PrivatePkcs1KeyDer::from(der)));
    }
    Err(anyhow!("no supported private key in {}", path.display()))
}

/// Decode every `-----BEGIN {tag}----- .. -----END {tag}-----` block to DER.
/// A dependency-free PEM reader (avoids pulling `rustls-pemfile`); tolerant of
/// CRLF and stray whitespace in the base64 body.
fn pem_blocks(pem: &str, tag: &str) -> Vec<Vec<u8>> {
    let begin = format!("-----BEGIN {tag}-----");
    let end = format!("-----END {tag}-----");
    let mut out = Vec::new();
    let mut rest = pem;
    while let Some(b) = rest.find(&begin) {
        let after = &rest[b + begin.len()..];
        let Some(e) = after.find(&end) else { break };
        let body: String = after[..e].chars().filter(|c| !c.is_whitespace()).collect();
        if let Ok(der) = base64::engine::general_purpose::STANDARD.decode(body.as_bytes()) {
            out.push(der);
        }
        rest = &after[e + end.len()..];
    }
    out
}

// ---------------------------------------------------------------------------
// The TLS listener (implements axum's Listener trait)
// ---------------------------------------------------------------------------

enum Acceptor {
    External(tokio_rustls::TlsAcceptor),
    Acme {
        acceptor: AcmeAcceptor,
        config: Arc<ServerConfig>,
    },
}

/// A TCP listener that terminates TLS before handing streams to axum.
pub struct TlsListener {
    tcp: TcpListener,
    acceptor: Acceptor,
}

impl TlsListener {
    /// Bind `addr` and prepare TLS termination. Returns the listener plus, for
    /// the external-cert mode, the [`ReloadableResolver`] the SIGHUP handler
    /// pokes to hot-reload.
    pub async fn bind(
        addr: &str,
        tls: &TlsConfig,
    ) -> anyhow::Result<(Self, Option<Arc<ReloadableResolver>>)> {
        // `ServerConfig::builder()` (here and inside rustls-acme) needs a
        // process-wide crypto provider. Install ring's once; ignore if another
        // component already did.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let tcp = TcpListener::bind(addr)
            .await
            .with_context(|| format!("binding {addr}"))?;
        match tls {
            TlsConfig::External { cert, key } => {
                let resolver = ReloadableResolver::load(cert.clone(), key.clone())?;
                let config = server_config(resolver.clone());
                let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
                Ok((
                    Self {
                        tcp,
                        acceptor: Acceptor::External(acceptor),
                    },
                    Some(resolver),
                ))
            }
            TlsConfig::Acme {
                domains,
                contact,
                cache_dir,
                staging,
            } => {
                let mut cfg = AcmeConfig::new(domains.clone())
                    .cache(DirCache::new(cache_dir.clone()))
                    .directory_lets_encrypt(!staging);
                if let Some(contact) = contact {
                    cfg = cfg.contact_push(format!("mailto:{contact}"));
                }
                let mut state = cfg.state();
                let config = server_config(state.resolver());
                let acceptor = state.acceptor();
                // Drive certificate acquisition/renewal for the process lifetime.
                tokio::spawn(async move {
                    use futures_util::StreamExt;
                    while let Some(event) = state.next().await {
                        match event {
                            Ok(ok) => tracing::info!("acme: {ok:?}"),
                            Err(err) => tracing::error!("acme: {err:?}"),
                        }
                    }
                });
                Ok((
                    Self {
                        tcp,
                        acceptor: Acceptor::Acme {
                            acceptor,
                            config: Arc::new(config),
                        },
                    },
                    None,
                ))
            }
        }
    }
}

/// A rustls server config that resolves certs through `resolver` and offers
/// HTTP/1.1 + HTTP/2 over ALPN.
fn server_config(resolver: Arc<dyn ResolvesServerCert>) -> ServerConfig {
    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(resolver);
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    config
}

impl axum::serve::Listener for TlsListener {
    type Io = TlsStream<TcpStream>;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            let (tcp, addr) = match self.tcp.accept().await {
                Ok(pair) => pair,
                Err(e) => {
                    // Transient accept errors: back off briefly and retry
                    // (contract: this method must not return an error).
                    tracing::debug!("tcp accept error: {e}");
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    continue;
                }
            };
            match &self.acceptor {
                Acceptor::External(acc) => match acc.accept(tcp).await {
                    Ok(tls) => return (tls, addr),
                    Err(e) => tracing::debug!("tls handshake from {addr} failed: {e}"),
                },
                Acceptor::Acme { acceptor, config } => match acceptor.accept(tcp).await {
                    Ok(Some(start)) => match start.into_stream(config.clone()).await {
                        Ok(tls) => return (tls, addr),
                        Err(e) => tracing::debug!("acme handshake from {addr} failed: {e}"),
                    },
                    // tls-alpn-01 validation request: served internally, no app stream.
                    Ok(None) => {}
                    Err(e) => tracing::debug!("acme accept from {addr} failed: {e}"),
                },
            }
        }
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.tcp.local_addr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/tls")
            .join(name)
    }

    #[test]
    fn loads_a_self_signed_pair() {
        let ck = load_certified_key(&fixture("cert1.pem"), &fixture("key1.pem")).unwrap();
        assert!(!ck.cert.is_empty());
    }

    #[test]
    fn reload_swaps_the_served_certificate() {
        // Stage cert1 into temp files, then overwrite with cert2 and reload.
        let dir = std::env::temp_dir().join(format!("mw-tls-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cert = dir.join("cert.pem");
        let key = dir.join("key.pem");
        std::fs::copy(fixture("cert1.pem"), &cert).unwrap();
        std::fs::copy(fixture("key1.pem"), &key).unwrap();

        let resolver = ReloadableResolver::load(cert.clone(), key.clone()).unwrap();
        let first = resolver.current_leaf_der();
        assert!(!first.is_empty());

        std::fs::copy(fixture("cert2.pem"), &cert).unwrap();
        std::fs::copy(fixture("key2.pem"), &key).unwrap();
        resolver.reload().unwrap();
        let second = resolver.current_leaf_der();

        assert_ne!(
            first, second,
            "reload must change the served leaf certificate"
        );
        // The end-to-end TLS handshake through this reloaded resolver is proven
        // by the `tls_reload` integration test.
    }

    #[test]
    fn reload_from_a_broken_file_keeps_the_old_cert() {
        let dir = std::env::temp_dir().join(format!("mw-tls-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cert = dir.join("cert.pem");
        let key = dir.join("key.pem");
        std::fs::copy(fixture("cert1.pem"), &cert).unwrap();
        std::fs::copy(fixture("key1.pem"), &key).unwrap();

        let resolver = ReloadableResolver::load(cert.clone(), key.clone()).unwrap();
        let before = resolver.current_leaf_der();

        std::fs::write(&cert, b"not a certificate").unwrap();
        assert!(resolver.reload().is_err());
        assert_eq!(
            resolver.current_leaf_der(),
            before,
            "a failed reload must not disturb the live cert"
        );
    }
}
