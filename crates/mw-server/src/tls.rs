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

/// PROXY protocol v1/v2 on the HTTP listener. Declared as a `#[path]` child here
/// rather than in `lib.rs` (whose owner is a different lane this wave); the file
/// itself lives at `crates/mw-server/src/proxy_protocol.rs` and a later `lib.rs`
/// change can promote it to a top-level `pub mod proxy_protocol;` unchanged.
#[path = "proxy_protocol.rs"]
pub mod proxy_protocol;

use proxy_protocol::ProxyAcceptor;

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
///
/// The TCP side is a [`ProxyAcceptor`], so when `MW_PROXY_PROTOCOL` is enabled the
/// PROXY header is consumed **before** the TLS handshake — which is the only place
/// it can be, since it precedes the ClientHello on the wire. This is the shape that
/// matters for TLS passthrough behind an L4 balancer, where the app terminates TLS
/// itself (built-in ACME) and would otherwise see the balancer as every client.
pub struct TlsListener {
    tcp: ProxyAcceptor,
    acceptor: Acceptor,
}

impl TlsListener {
    /// Bind `addr` and prepare TLS termination. Returns the listener plus, for
    /// the external-cert mode, the [`ReloadableResolver`] the SIGHUP handler
    /// pokes to hot-reload.
    ///
    /// In ACME mode the domain list is checked first ([`validate_acme_domains`]);
    /// a list that fails is an error and `addr` is never bound.
    pub async fn bind(
        addr: &str,
        tls: &TlsConfig,
    ) -> anyhow::Result<(Self, Option<Arc<ReloadableResolver>>)> {
        // Refuse an ACME domain list that cannot be one before anything is bound:
        // `--acme off` would otherwise start an HTTPS listener and ask Let's
        // Encrypt for a certificate for a host named `off`.
        if let TlsConfig::Acme { domains, .. } = tls {
            validate_acme_domains(domains)?;
        }
        // `ServerConfig::builder()` (here and inside rustls-acme) needs a
        // process-wide crypto provider. Install ring's once; ignore if another
        // component already did.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let tcp = TcpListener::bind(addr)
            .await
            .with_context(|| format!("binding {addr}"))?;
        let tcp = ProxyAcceptor::new(tcp, proxy_protocol::Config::from_env())
            .with_context(|| format!("reading the bound address of {addr}"))?;
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

/// Values people write to mean "no ACME". `--acme` has no such sentinel — it takes
/// domain names, and any value at all switches the listener to HTTPS — so each of
/// these is refused by name instead of being requested as a certificate.
const ACME_NOT_A_DOMAIN: &[&str] = &["off", "no", "none", "false", "disabled", "0"];

/// Check that every `--acme` / `MW_ACME` entry could be a DNS name a public CA
/// would issue for. Called by [`TlsListener::bind`] before the socket is bound.
///
/// An entry is refused when, after trimming and lowercasing, it is empty, is one of
/// the "off" spellings above, has no dot, has a character outside `[a-z0-9.-]`, or
/// has a label that is empty or starts or ends with `-`. This is a plausibility
/// check, not DNS validation: it exists to turn `--acme off` (which the deploy
/// docs once recommended) and `MW_ACME=""` into a startup error rather than an
/// HTTPS listener with no certificate behind what the operator meant to be plain
/// HTTP.
pub fn validate_acme_domains(domains: &[String]) -> anyhow::Result<()> {
    const HOW_TO_DISABLE: &str = "to run without built-in ACME, do not pass --acme and leave \
                                  MW_ACME unset (an empty MW_ACME counts as set)";
    if domains.is_empty() {
        return Err(anyhow!("ACME needs at least one domain; {HOW_TO_DISABLE}"));
    }
    for raw in domains {
        let d = raw.trim().to_ascii_lowercase();
        if d.is_empty() {
            return Err(anyhow!(
                "--acme / MW_ACME has an empty entry; {HOW_TO_DISABLE}"
            ));
        }
        if ACME_NOT_A_DOMAIN.contains(&d.as_str()) {
            return Err(anyhow!(
                "--acme / MW_ACME is set to {raw:?}, which is not a domain name: it would \
                 request a certificate for a host named {raw:?}; {HOW_TO_DISABLE}"
            ));
        }
        let plausible = d.contains('.')
            && d.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-')
            && d.split('.')
                .all(|label| !label.is_empty() && !label.starts_with('-') && !label.ends_with('-'));
        if !plausible {
            return Err(anyhow!(
                "--acme / MW_ACME entry {raw:?} is not a fully-qualified domain name; \
                 {HOW_TO_DISABLE}"
            ));
        }
    }
    Ok(())
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
            // `addr` is the client address the PROXY header named when one was read
            // and believed, and the socket peer otherwise. Transient accept errors
            // are handled inside the acceptor (contract: this method cannot fail).
            let (tcp, addr) = self.tcp.next_conn().await;
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
        ProxyAcceptor::local_addr(&self.tcp)
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

    fn acme(domains: &[&str]) -> Result<(), String> {
        let owned: Vec<String> = domains.iter().map(|d| d.to_string()).collect();
        validate_acme_domains(&owned).map_err(|e| e.to_string())
    }

    #[test]
    fn acme_domains_accepts_hostnames() {
        assert_eq!(acme(&["mail.example.org"]), Ok(()));
        assert_eq!(acme(&["mail.example.org", "Webmail.Example.ORG"]), Ok(()));
        assert_eq!(acme(&[" mail-2.example.org "]), Ok(()));
        assert_eq!(acme(&["xn--mnchen-3ya.example"]), Ok(()));
    }

    #[test]
    fn acme_domains_refuses_off_spellings_and_non_hostnames() {
        for bad in ["off", "OFF", "no", "none", "false", "disabled", "0"] {
            let err = acme(&[bad]).expect_err(bad);
            assert!(
                err.contains(bad) && err.contains("leave MW_ACME unset"),
                "{bad}: the error names the value and says how to disable ACME: {err}"
            );
        }
        for bad in [
            "",
            "   ",
            "localhost",
            "mail_server.example.org",
            "mail.example.org/",
            "https://mail.example.org",
            ".example.org",
            "example.org.",
            "mail..example.org",
            "-mail.example.org",
            "mail-.example.org",
            "*.example.org",
        ] {
            assert!(acme(&[bad]).is_err(), "{bad:?} must be refused");
        }
        // One bad entry refuses the whole list; an empty list is refused too.
        assert!(acme(&["mail.example.org", "off"]).is_err());
        assert!(acme(&["mail.example.org", ""]).is_err());
        assert!(acme(&[]).is_err());
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
