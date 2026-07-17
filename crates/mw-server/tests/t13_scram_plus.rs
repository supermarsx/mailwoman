//! t13-E10 — SCRAM-SHA-256-PLUS channel binding live-E2E (26.13 workstream 1).
//!
//! The "unit-green ≠ wired" gate for the `-PLUS` channel-binding work (E1 IMAP, E2
//! SMTP, E3 POP3). What this leg proves LIVE against a REAL Dovecot 2.4.4 over REAL
//! TLS, and what it necessarily gates, is spelled out below — the honest disposition.
//!
//! ## FINDING (empirically proven here): Dovecot does not implement tls-server-end-point
//! Dovecot 2.4.x ADVERTISES `AUTH=SCRAM-SHA-256-PLUS` over TLS, but its channel
//! binding implements only `tls-unique` / `tls-exporter` — NOT `tls-server-end-point`
//! (RFC 5929), the binding type mw-imap/mw-pop3/mw-smtp use. Driving the shipped
//! `-PLUS` client against it makes the server reject the exchange with:
//!   `NO [ALERT] Channel binding failed: Unsupported channel binding type
//!    'tls-server-end-point'`
//! (2.4.1 silently drops the connection at the same point; 2.4.4 returns the clean
//! error above — hence the 2.4.4 pin). This is a SERVER capability gap, NOT a
//! mw-client bug: our client correctly sends the widely-interoperable RFC 5929
//! `tls-server-end-point` binding (the same one PostgreSQL and most SMTP/IMAP
//! SCRAM-PLUS stacks use). It is exactly the version/server-dependent CB risk the
//! plan flagged (§ flag 5). NOTE for the coordinator: whether to ALSO offer
//! `tls-exporter` for Dovecot interop is a product decision, not a release blocker.
//!
//! ## What IS proven LIVE (against the real server + real certs)
//!   * the server genuinely advertises `SCRAM-SHA-256-PLUS`,
//!   * the shipped `tls_server_end_point` computes the RFC 5929 binding from the REAL
//!     leaf cert the server presents — 32 bytes from a SHA-256-signed cert, 64 bytes
//!     from a SHA-512-signed cert — exercising E1/E3's digest selection on real-world
//!     certs (not just the checked-in fixtures), and byte-exact against `sha256(der)`,
//!   * the shipped `-PLUS` client framing is well-formed and reaches the server, which
//!     explicitly rejects only the cbind TYPE.
//!
//! ## What is therefore GATED (unit/mock-proven, not live)
//! The SCRAM-PLUS PROOF acceptance — a server validating the channel-bound client
//! proof and completing login — cannot be exercised because no in-environment server
//! implements `tls-server-end-point`. That path stays proven at the unit/mock level:
//! E1 (IMAP 22 tests), E2 (SMTP mock server negotiating `-PLUS`, 23 tests), E3 (POP3
//! mock framing). SMTP additionally has no CB-capable submission server here and its
//! `sasl` module is crate-private (unreachable from an integration test), so SMTP
//! `-PLUS` is entirely unit/mock-proven — see `.orchestration/logs/t13-E2.md`.
//!
//! ## Why a test-local TLS stream
//! The shipped IMAP/POP3/SMTP TLS clients trust ONLY the compiled-in Mozilla webpki
//! roots (`crates/*/src/tls.rs`) — a self-signed CI cert cannot handshake through the
//! public connect API (correct client policy, orthogonal to CB). This test opens its
//! own tokio-rustls stream with a test-only no-verify verifier (handshake SIGNATURE
//! still verified via the ring provider; only the trust anchor is skipped), extracts
//! the REAL leaf cert, and drives the SHIPPED `tls_server_end_point` + `ScramSha256`.
//!
//! ## Running
//!   scripts/dovecot-t13/gen-certs.sh
//!   docker compose -f docker-compose.ci.yml up -d --wait dovecot-t13 dovecot-t13-sha512
//!   MW_T13_TLS_LIVE=1 cargo test -p mw-server --test t13_scram_plus -- --nocapture

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use sha2::{Digest, Sha256};

use mw_imap::sasl::SaslClient;

const IMAPS_SHA256: u16 = 3993; // dovecot-t13, RSA/SHA-256 leaf
const IMAPS_SHA512: u16 = 4993; // dovecot-t13-sha512, RSA/SHA-512 leaf
const POP3S_SHA256: u16 = 3995; // dovecot-t13, RSA/SHA-256 leaf
const USER: &str = "testuser";
const PASS: &str = "testpass";

fn live() -> bool {
    std::env::var("MW_T13_TLS_LIVE").ok().as_deref() == Some("1")
}
fn host() -> String {
    std::env::var("MW_T13_HOST").unwrap_or_else(|_| "127.0.0.1".into())
}

// ── Test-only TLS: skip cert-chain trust, keep handshake-signature verification ──
#[derive(Debug)]
struct NoVerify(Arc<rustls::crypto::CryptoProvider>);

impl rustls::client::danger::ServerCertVerifier for NoVerify {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls_pki_types::CertificateDer<'_>,
        _intermediates: &[rustls_pki_types::CertificateDer<'_>],
        _server_name: &rustls_pki_types::ServerName<'_>,
        _ocsp: &[u8],
        _now: rustls_pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls_pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls_pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

/// Open a TLS stream to `port`, returning the stream and the REAL leaf cert DER.
async fn tls_connect(port: u16) -> Option<(TlsStream<TcpStream>, Vec<u8>)> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .unwrap()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerify(provider)))
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    let server_name = rustls_pki_types::ServerName::try_from("localhost").unwrap();
    let h = host();
    let tcp = match TcpStream::connect((h.as_str(), port)).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!(
                "\n[t13 SCRAM-PLUS SKIP] dovecot-t13 unreachable at {h}:{port} ({e}). Bring it up: \
                 scripts/dovecot-t13/gen-certs.sh ; docker compose -f docker-compose.ci.yml up -d \
                 --wait dovecot-t13 dovecot-t13-sha512 ; MW_T13_TLS_LIVE=1 cargo test -p mw-server \
                 --test t13_scram_plus.\n"
            );
            return None;
        }
    };
    let stream = connector
        .connect(server_name, tcp)
        .await
        .expect("TLS handshake");
    let leaf = {
        let (_, conn) = stream.get_ref();
        conn.peer_certificates()
            .and_then(|c| c.first())
            .map(|c| c.as_ref().to_vec())
            .expect("server presented a leaf certificate")
    };
    Some((stream, leaf))
}

async fn read_line(s: &mut TlsStream<TcpStream>) -> String {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        match s.read(&mut byte).await {
            Ok(0) | Err(_) => break, // EOF / reset — the caller inspects what was read
            Ok(_) => {
                buf.push(byte[0]);
                if buf.ends_with(b"\r\n") {
                    break;
                }
            }
        }
    }
    String::from_utf8_lossy(&buf).trim_end().to_string()
}

async fn write_line(s: &mut TlsStream<TcpStream>, line: &str) {
    s.write_all(line.as_bytes()).await.unwrap();
    s.write_all(b"\r\n").await.unwrap();
    s.flush().await.unwrap();
}

/// Drive `AUTHENTICATE <mech>` via the shipped `SaslClient`; return the final tagged
/// line (or whatever the server sent if it aborts the exchange).
async fn imap_authenticate(
    s: &mut TlsStream<TcpStream>,
    mech: &str,
    client: &mut dyn SaslClient,
) -> String {
    write_line(s, &format!("a1 AUTHENTICATE {mech}")).await;
    loop {
        let line = read_line(s).await;
        if let Some(rest) = line.strip_prefix("+ ").or_else(|| line.strip_prefix("+")) {
            let rest = rest.trim();
            let challenge = if rest.is_empty() {
                Vec::new()
            } else {
                B64.decode(rest).unwrap_or_default()
            };
            match client.step(&challenge) {
                Ok(resp) => write_line(s, &B64.encode(&resp)).await,
                Err(e) => return format!("<client-step-error: {e}>"),
            }
        } else {
            return line; // tagged result or empty (connection dropped)
        }
    }
}

// ── IMAP: binding computation from the REAL cert (SHA-256 leaf → 32 bytes) ────────
#[tokio::test]
async fn imap_tls_server_end_point_binding_sha256_cert_live() {
    if !live() {
        eprintln!("\n[t13 SCRAM-PLUS SKIP] MW_T13_TLS_LIVE!=1 — real CB Dovecot not driven.\n");
        return;
    }
    let Some((mut s, leaf)) = tls_connect(IMAPS_SHA256).await else {
        return;
    };
    let binding = mw_imap::sasl::tls_server_end_point(&leaf);
    assert_eq!(
        binding.len(),
        32,
        "SHA-256-signed leaf → 32-byte tls-server-end-point binding"
    );
    assert_eq!(
        binding,
        Sha256::digest(&leaf).to_vec(),
        "binding must be sha256(leaf DER) byte-for-byte"
    );

    let greeting = read_line(&mut s).await;
    assert!(
        greeting.contains("AUTH=SCRAM-SHA-256-PLUS"),
        "server must advertise SCRAM-SHA-256-PLUS over TLS; greeting={greeting:?}"
    );
    write_line(&mut s, "a9 LOGOUT").await;
}

// ── IMAP: SHA-512-signed leaf → 64-byte binding (the SHA-384/512 digest leg) ──────
#[tokio::test]
async fn imap_tls_server_end_point_binding_sha512_cert_live() {
    if !live() {
        return;
    }
    let Some((mut s, leaf)) = tls_connect(IMAPS_SHA512).await else {
        return;
    };
    let binding = mw_imap::sasl::tls_server_end_point(&leaf);
    // RFC 5929 sig-hash floor: a SHA-512-signed cert yields a 64-byte binding — the
    // digest selection E1 added, exercised here against a REAL server cert.
    assert_eq!(
        binding.len(),
        64,
        "SHA-512-signed leaf → 64-byte tls-server-end-point binding"
    );
    let greeting = read_line(&mut s).await;
    assert!(
        greeting.contains("AUTH=SCRAM-SHA-256-PLUS"),
        "greeting={greeting:?}"
    );
    write_line(&mut s, "a9 LOGOUT").await;
}

// ── IMAP: the shipped -PLUS client reaches the server, which lacks the cbind type ──
#[tokio::test]
async fn imap_scram_plus_dovecot_lacks_tls_server_end_point_live() {
    if !live() {
        return;
    }
    let Some((mut s, leaf)) = tls_connect(IMAPS_SHA256).await else {
        return;
    };
    let binding = mw_imap::sasl::tls_server_end_point(&leaf);
    let _greeting = read_line(&mut s).await;

    let mut client = mw_imap::sasl::ScramSha256::new(USER, PASS, Some(binding));
    let result = imap_authenticate(&mut s, "SCRAM-SHA-256-PLUS", &mut client).await;

    // The client framing reached the server; Dovecot rejects only the cbind TYPE.
    // (Documents the FINDING: -PLUS is advertised but tls-server-end-point is not
    // implemented server-side. If a future server DOES support it this assertion will
    // fail — a prompt to enable the full channel-bound login proof here.)
    let lower = result.to_lowercase();
    assert!(
        (result.contains("NO") || result.contains("BAD"))
            && (lower.contains("channel binding") || lower.contains("tls-server-end-point")),
        "expected Dovecot to reject tls-server-end-point (server CB gap); got {result:?}"
    );
}

// ── POP3: binding computation + the same tls-server-end-point gap ─────────────────
#[tokio::test]
async fn pop3_tls_server_end_point_binding_and_cbind_gap_live() {
    if !live() {
        return;
    }
    let Some((mut s, leaf)) = tls_connect(POP3S_SHA256).await else {
        return;
    };
    let binding = mw_pop3::sasl::tls_server_end_point(&leaf);
    assert_eq!(binding.len(), 32, "SHA-256 leaf → 32-byte POP3 binding");
    assert_eq!(
        binding,
        Sha256::digest(&leaf).to_vec(),
        "POP3 binding must be sha256(leaf DER)"
    );

    let greeting = read_line(&mut s).await;
    assert!(greeting.starts_with("+OK"), "POP3 greeting: {greeting:?}");

    // CAPA advertises SCRAM-SHA-256-PLUS.
    write_line(&mut s, "CAPA").await;
    let mut saw_plus = false;
    loop {
        let l = read_line(&mut s).await;
        if l.contains("SCRAM-SHA-256-PLUS") {
            saw_plus = true;
        }
        if l == "." || l.starts_with("-ERR") || l.is_empty() {
            break;
        }
    }
    assert!(saw_plus, "POP3S must advertise SASL SCRAM-SHA-256-PLUS");

    // Drive the shipped POP3 -PLUS client; the server rejects the cbind type.
    let nonce = format!(
        "t13{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let (_client, client_first) =
        mw_pop3::sasl::ScramSha256::new(USER, PASS, &nonce, Some(binding));
    write_line(&mut s, "AUTH SCRAM-SHA-256-PLUS").await;
    let prompt = read_line(&mut s).await;
    assert!(prompt.starts_with("+"), "POP3 AUTH prompt: {prompt:?}");
    write_line(&mut s, &B64.encode(client_first.as_bytes())).await;

    // Dovecot POP3 responds with -ERR (no tls-server-end-point) rather than a
    // server-first continuation. Either an -ERR or a dropped connection evidences the
    // same server CB gap; the well-formed client-first is what reaches the server.
    let resp = read_line(&mut s).await;
    assert!(
        resp.starts_with("-ERR") || resp.is_empty() || resp.to_lowercase().contains("channel"),
        "POP3 -PLUS should be rejected (server lacks tls-server-end-point); got {resp:?}"
    );
    write_line(&mut s, "QUIT").await;
}
