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
//!
//! # Minimum TLS version
//! [`set_min_tls`] sets the lowest TLS version this listener accepts, and
//! [`apply_min_tls`] sets it together with the five outbound TLS connectors.
//! rustls 0.23 speaks TLS 1.2 and 1.3 only, so the floor is a two-value choice
//! ([`MinTls`]); unset, it is 1.2, which is the configuration the listener had
//! before the floor existed.

use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
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
// Minimum TLS version
// ---------------------------------------------------------------------------

/// The lowest TLS version a connection may negotiate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinTls {
    /// TLS 1.2 or 1.3 (the default; rustls offers nothing older).
    V12,
    /// TLS 1.3 only. A peer limited to TLS 1.2 is refused at the handshake.
    V13,
}

impl MinTls {
    /// Parse the stored or configured form: `"1.2"` or `"1.3"`, surrounding
    /// whitespace ignored. Anything else is `None` — including `"1.0"` and
    /// `"1.1"`, which rustls cannot speak.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "1.2" => Some(MinTls::V12),
            "1.3" => Some(MinTls::V13),
            _ => None,
        }
    }

    /// The form [`MinTls::parse`] reads back.
    pub fn as_str(self) -> &'static str {
        match self {
            MinTls::V12 => "1.2",
            MinTls::V13 => "1.3",
        }
    }
}

static LISTENER_MIN_TLS: AtomicU8 = AtomicU8::new(0);

/// Set the floor for the inbound HTTPS listener only (see [`apply_min_tls`] for
/// the outbound connectors as well). Every [`TlsListener`] in the process reads
/// it at the start of each handshake, so it applies to the next connection
/// without rebinding. Connections that have already completed a handshake are
/// not dropped and keep the version they negotiated.
///
/// In ACME mode the `tls-alpn-01` validation handshake is served inside
/// `tokio-rustls-acme` with its own configuration and is not subject to the floor.
pub fn set_min_tls(v: MinTls) {
    LISTENER_MIN_TLS.store(matches!(v, MinTls::V13) as u8, Ordering::SeqCst);
}

/// The listener floor currently in force.
pub fn min_tls() -> MinTls {
    if LISTENER_MIN_TLS.load(Ordering::SeqCst) == 0 {
        MinTls::V12
    } else {
        MinTls::V13
    }
}

/// Set the floor on the inbound listener and on every outbound TLS connector
/// that has one, for connections opened from now on:
///
/// * the HTTPS listener ([`set_min_tls`]);
/// * IMAP, implicit TLS and STARTTLS (`mw_imap::set_min_tls`);
/// * SMTP submission, implicit TLS and STARTTLS (`mw_smtp::set_min_tls`);
/// * POP3, implicit TLS and STLS (`mw_pop3::conn::set_min_tls`);
/// * ManageSieve, implicit TLS and STARTTLS (`mw_sieve::set_min_tls`);
/// * the origin TLS inside an egress proxy tunnel
///   (`mw_egress::proxy::stream::set_min_tls`).
///
/// Not covered by any floor in this tree: the `reqwest` HTTP clients, and the
/// TLS inside `ldap3` (LDAP), `fred` (Redis) and `sqlx` (Postgres).
///
/// Raising the floor to 1.3 makes every upstream that only speaks TLS 1.2
/// unreachable through the connectors above; their errors then contain "the
/// minimum TLS version is set to 1.3". Passing [`MinTls::V12`] undoes it.
pub fn apply_min_tls(v: MinTls) {
    set_min_tls(v);
    mw_imap::set_min_tls(match v {
        MinTls::V12 => mw_imap::MinTls::V12,
        MinTls::V13 => mw_imap::MinTls::V13,
    });
    mw_smtp::set_min_tls(match v {
        MinTls::V12 => mw_smtp::MinTls::V12,
        MinTls::V13 => mw_smtp::MinTls::V13,
    });
    mw_pop3::conn::set_min_tls(match v {
        MinTls::V12 => mw_pop3::conn::MinTls::V12,
        MinTls::V13 => mw_pop3::conn::MinTls::V13,
    });
    mw_sieve::set_min_tls(match v {
        MinTls::V12 => mw_sieve::MinTls::V12,
        MinTls::V13 => mw_sieve::MinTls::V13,
    });
    mw_egress::proxy::stream::set_min_tls(match v {
        MinTls::V12 => mw_egress::proxy::stream::MinTls::V12,
        MinTls::V13 => mw_egress::proxy::stream::MinTls::V13,
    });
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
    External,
    Acme(AcmeAcceptor),
}

/// A TCP listener that terminates TLS before handing streams to axum.
///
/// The TCP side is a [`ProxyAcceptor`], so when `MW_PROXY_PROTOCOL` is enabled the
/// PROXY header is consumed **before** the TLS handshake — which is the only place
/// it can be, since it precedes the ClientHello on the wire. This is the shape that
/// matters for TLS passthrough behind an L4 balancer, where the app terminates TLS
/// itself (built-in ACME) and would otherwise see the balancer as every client.
///
/// The rustls configuration is rebuilt when the floor ([`set_min_tls`]) differs
/// from the one it was built for, which is checked before each handshake.
pub struct TlsListener {
    tcp: ProxyAcceptor,
    acceptor: Acceptor,
    /// Where certificates come from; kept so the configuration can be rebuilt.
    resolver: Arc<dyn ResolvesServerCert>,
    /// The configuration in use and the floor it was built for.
    config: (MinTls, Arc<ServerConfig>),
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
                let floor = min_tls();
                let config = server_config(resolver.clone(), floor);
                Ok((
                    Self {
                        tcp,
                        acceptor: Acceptor::External,
                        resolver: resolver.clone(),
                        config: (floor, Arc::new(config)),
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
                let resolver: Arc<dyn ResolvesServerCert> = state.resolver();
                let floor = min_tls();
                let config = server_config(resolver.clone(), floor);
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
                        acceptor: Acceptor::Acme(acceptor),
                        resolver,
                        config: (floor, Arc::new(config)),
                    },
                    None,
                ))
            }
        }
    }

    /// The configuration for the floor in force, rebuilt if the floor has changed
    /// since the last handshake.
    fn current_config(&mut self) -> Arc<ServerConfig> {
        let floor = min_tls();
        if self.config.0 != floor {
            self.config = (floor, Arc::new(server_config(self.resolver.clone(), floor)));
        }
        self.config.1.clone()
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

/// A rustls server config that resolves certs through `resolver`, offers
/// HTTP/1.1 + HTTP/2 over ALPN, and accepts the protocol versions `floor` allows:
/// the rustls defaults (1.2 and 1.3) for [`MinTls::V12`], 1.3 alone for
/// [`MinTls::V13`].
fn server_config(resolver: Arc<dyn ResolvesServerCert>, floor: MinTls) -> ServerConfig {
    let builder = match floor {
        MinTls::V12 => ServerConfig::builder(),
        MinTls::V13 => ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13]),
    };
    let mut config = builder.with_no_client_auth().with_cert_resolver(resolver);
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    config
}

/// Describe a failed inbound handshake for the log, naming the floor when it is
/// 1.3 and rustls found nothing in common with the client — which is what a
/// client limited to TLS 1.2 produces.
fn describe_handshake_failure(e: &io::Error, floor: MinTls) -> String {
    let incompatible = matches!(
        e.get_ref()
            .and_then(|inner| inner.downcast_ref::<rustls::Error>()),
        Some(rustls::Error::PeerIncompatible(_))
    );
    if floor == MinTls::V13 && incompatible {
        format!("the minimum TLS version is set to 1.3 and the client did not offer it ({e})")
    } else {
        e.to_string()
    }
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
            // Read the floor once per connection: a change made by `set_min_tls`
            // takes effect here, for this and every later handshake.
            let config = self.current_config();
            let floor = self.config.0;
            match &self.acceptor {
                Acceptor::External => {
                    match tokio_rustls::TlsAcceptor::from(config).accept(tcp).await {
                        Ok(tls) => return (tls, addr),
                        Err(e) => tracing::debug!(
                            "tls handshake from {addr} failed: {}",
                            describe_handshake_failure(&e, floor)
                        ),
                    }
                }
                Acceptor::Acme(acceptor) => match acceptor.accept(tcp).await {
                    Ok(Some(start)) => match start.into_stream(config).await {
                        Ok(tls) => return (tls, addr),
                        Err(e) => tracing::debug!(
                            "acme handshake from {addr} failed: {}",
                            describe_handshake_failure(&e, floor)
                        ),
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
    fn min_tls_parses_the_two_versions_rustls_speaks_and_nothing_else() {
        assert_eq!(MinTls::parse("1.2"), Some(MinTls::V12));
        assert_eq!(MinTls::parse(" 1.3\n"), Some(MinTls::V13));
        for bad in ["", "1.0", "1.1", "1.4", "13", "TLSv1.3", "tls1.2", "v1.3"] {
            assert_eq!(MinTls::parse(bad), None, "{bad:?}");
        }
        for v in [MinTls::V12, MinTls::V13] {
            assert_eq!(MinTls::parse(v.as_str()), Some(v));
        }
    }

    #[test]
    fn a_handshake_failure_names_the_floor_only_when_the_floor_explains_it() {
        let incompatible = || {
            io::Error::new(
                io::ErrorKind::InvalidData,
                rustls::Error::PeerIncompatible(rustls::PeerIncompatible::Tls13RequiredForQuic),
            )
        };
        assert!(
            describe_handshake_failure(&incompatible(), MinTls::V13)
                .starts_with("the minimum TLS version is set to 1.3")
        );
        assert_eq!(
            describe_handshake_failure(&incompatible(), MinTls::V12),
            incompatible().to_string()
        );
        let reset = io::Error::new(io::ErrorKind::ConnectionReset, "reset");
        assert_eq!(describe_handshake_failure(&reset, MinTls::V13), "reset");
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
