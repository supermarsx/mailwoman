//! t14-E-e2e — HEADLINE: SCRAM-SHA-256-PLUS login COMPLETES via `tls-exporter`
//! (RFC 9266) against a REAL Dovecot 2.4.4 over TLS 1.3.
//!
//! This is the exact acceptance 26.13 could NOT reach: Dovecot 2.4.x advertises
//! `AUTH=SCRAM-SHA-256-PLUS` but implements only `tls-unique` / `tls-exporter`
//! channel binding — NOT `tls-server-end-point` (RFC 5929), which 26.13's client
//! used. 26.14 adds the `tls-exporter` binding (E1 IMAP, E2 SMTP, E3 POP3): on a
//! TLS 1.3 connection the client exports 32 bytes of keying material with label
//! `"EXPORTER-Channel-Binding"` and an empty context, and drives the shipped
//! `ScramSha256` with cb-name `tls-exporter`.
//!
//! What this leg proves LIVE (unit-green ≠ wired):
//!   * a REAL TLS 1.3 handshake is negotiated (asserted via `protocol_version()`),
//!   * `export_keying_material([0u8;32], b"EXPORTER-Channel-Binding", Some(&[]))`
//!     yields the 32-byte RFC 9266 binding,
//!   * the shipped `ScramSha256` (`mw-imap` and `mw-pop3`) builds a channel-bound
//!     `-PLUS` exchange against it, and
//!   * the REAL server VALIDATES the channel-bound client proof and returns
//!     **login OK** — for BOTH IMAP and POP3.
//!
//! SMTP disposition: there is no channel-binding-capable submission server in this
//! environment and `mw_smtp::sasl` is crate-private (unreachable from an
//! integration test), so SMTP `-PLUS` stays unit/mock-proven (E2, 23 tests) — the
//! same honest disposition t13 documented. The IMAP + POP3 legs here prove the
//! `tls-exporter` binding round-trips end-to-end against a real server; the SMTP
//! client shares the identical per-crate design (coordinator kept the three
//! cb-name/selection semantics identical).
//!
//! ## Why a test-local TLS stream
//! The shipped IMAP/POP3 TLS clients trust only the compiled-in Mozilla webpki
//! roots, so a self-signed CI cert cannot handshake through the public connect
//! API (correct client policy, orthogonal to channel binding). This test opens its
//! own tokio-rustls stream with a test-only no-verify verifier (handshake
//! SIGNATURE still verified via the ring provider — only the trust anchor is
//! skipped), pins the negotiated version to TLS 1.3, exports the keying material,
//! and drives the SHIPPED `ScramSha256`.
//!
//! ## Running
//!   scripts/dovecot-t13/gen-certs.sh
//!   docker compose -f docker-compose.ci.yml up -d --wait dovecot-t13
//!   MW_T14_TLS_LIVE=1 cargo test -p mw-server --test t14_scram_plus_login -- --nocapture

use std::sync::Arc;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;

use mw_imap::sasl::SaslClient;

const IMAPS_SHA256: u16 = 3993; // dovecot-t13, RSA/SHA-256 leaf, implicit TLS
const POP3S_SHA256: u16 = 3995; // dovecot-t13, RSA/SHA-256 leaf, implicit TLS
const USER: &str = "testuser";
const PASS: &str = "testpass";

// RFC 9266: the tls-exporter binding is 32 bytes exported under this label with an
// empty context. These MUST match the shipped client (mw-imap/connection.rs,
// mw-pop3/conn.rs) exactly — the point of the live gate is that the server accepts
// the very bytes the client computes.
const EXPORTER_LABEL: &[u8] = b"EXPORTER-Channel-Binding";

fn live() -> bool {
    std::env::var("MW_T14_TLS_LIVE").ok().as_deref() == Some("1")
}
fn host() -> String {
    std::env::var("MW_T14_HOST").unwrap_or_else(|_| "127.0.0.1".into())
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

/// Open a TLS 1.3 stream to `port`, returning the stream and the negotiated
/// tls-exporter channel-binding bytes (RFC 9266). Pins TLS 1.3 so the exporter
/// path is exercised (fails loudly if the server won't do 1.3).
async fn tls13_connect(port: u16) -> Option<(TlsStream<TcpStream>, Vec<u8>)> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("TLS 1.3 supported by the ring provider")
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
                "\n[t14 SCRAM-PLUS SKIP] dovecot-t13 unreachable at {h}:{port} ({e}). Bring it up: \
                 scripts/dovecot-t13/gen-certs.sh ; docker compose -f docker-compose.ci.yml up -d \
                 --wait dovecot-t13 ; MW_T14_TLS_LIVE=1 cargo test -p mw-server \
                 --test t14_scram_plus_login.\n"
            );
            return None;
        }
    };
    let stream = connector
        .connect(server_name, tcp)
        .await
        .expect("TLS 1.3 handshake");
    let binding = {
        let (_, conn) = stream.get_ref();
        assert_eq!(
            conn.protocol_version(),
            Some(rustls::ProtocolVersion::TLSv1_3),
            "the -PLUS login gate must run over TLS 1.3 (the tls-exporter path)"
        );
        conn.export_keying_material([0u8; 32], EXPORTER_LABEL, Some(&[]))
            .expect("TLS 1.3 exporter keying material")
            .to_vec()
    };
    assert_eq!(
        binding.len(),
        32,
        "RFC 9266 tls-exporter binding is 32 bytes"
    );
    Some((stream, binding))
}

async fn read_line(s: &mut TlsStream<TcpStream>) -> String {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        match s.read(&mut byte).await {
            Ok(0) | Err(_) => break,
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

/// Drive `AUTHENTICATE <mech>` via the shipped `SaslClient`; return the final
/// tagged line (or whatever the server sent if it aborts the exchange).
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
            return line; // tagged result
        }
    }
}

// ── HEADLINE (IMAP): tls-exporter -PLUS login COMPLETES vs real Dovecot 2.4.4 ─────
#[tokio::test]
async fn imap_scram_plus_tls_exporter_login_completes_live() {
    if !live() {
        eprintln!("\n[t14 SCRAM-PLUS SKIP] MW_T14_TLS_LIVE!=1 — real CB Dovecot not driven.\n");
        return;
    }
    let Some((mut s, binding)) = tls13_connect(IMAPS_SHA256).await else {
        return;
    };
    let _greeting = read_line(&mut s).await;

    // Drive the SHIPPED SCRAM client with the tls-exporter binding computed from
    // the REAL TLS 1.3 connection.
    let mut client =
        mw_imap::sasl::ScramSha256::new(USER, PASS, Some(("tls-exporter", binding.clone())));
    let result = imap_authenticate(&mut s, "SCRAM-SHA-256-PLUS", &mut client).await;

    // THE acceptance 26.13 could not reach: the server validated the channel-bound
    // proof and completed login.
    assert!(
        result.starts_with("a1 OK") || result.contains(" OK "),
        "SCRAM-SHA-256-PLUS via tls-exporter must COMPLETE login (server-validated \
         channel-bound proof); got {result:?}"
    );
    write_line(&mut s, "a9 LOGOUT").await;
}

// ── HEADLINE (POP3): tls-exporter -PLUS login COMPLETES vs real Dovecot 2.4.4 ─────
#[tokio::test]
async fn pop3_scram_plus_tls_exporter_login_completes_live() {
    if !live() {
        return;
    }
    let Some((mut s, binding)) = tls13_connect(POP3S_SHA256).await else {
        return;
    };
    let greeting = read_line(&mut s).await;
    assert!(greeting.starts_with("+OK"), "POP3S greeting: {greeting:?}");

    // Build the client-first via the SHIPPED POP3 SCRAM client with the exporter
    // binding, then drive the full RFC 5034 AUTH choreography to a +OK.
    let nonce = mw_pop3::sasl::client_nonce();
    let (mut scram, client_first) =
        mw_pop3::sasl::ScramSha256::new(USER, PASS, &nonce, Some(("tls-exporter", binding)));

    write_line(&mut s, "AUTH SCRAM-SHA-256-PLUS").await;
    let prompt = read_line(&mut s).await;
    assert!(prompt.starts_with("+"), "POP3 AUTH prompt: {prompt:?}");
    write_line(&mut s, &B64.encode(client_first.as_bytes())).await;

    // Server-first arrives as a `+ <base64>` continuation.
    let sf_line = read_line(&mut s).await;
    let sf_b64 = sf_line
        .strip_prefix("+ ")
        .or_else(|| sf_line.strip_prefix("+"))
        .unwrap_or_else(|| panic!("expected POP3 server-first continuation; got {sf_line:?}"))
        .trim();
    let server_first =
        String::from_utf8(B64.decode(sf_b64).expect("server-first base64")).expect("utf8");
    let client_final = scram.client_final(&server_first).expect("client-final");
    write_line(&mut s, &B64.encode(client_final.as_bytes())).await;

    // Dovecot delivers the server-final (v=) as a final `+ <base64>` continuation;
    // verify it, ack with an empty line, then require +OK.
    let next = read_line(&mut s).await;
    let final_ok = if let Some(rest) = next.strip_prefix("+ ").or_else(|| next.strip_prefix("+")) {
        let rest = rest.trim();
        if rest.starts_with("OK") {
            // Some servers may fold straight to +OK.
            true
        } else {
            let server_final =
                String::from_utf8(B64.decode(rest).expect("server-final base64")).expect("utf8");
            scram
                .verify(&server_final)
                .expect("server signature must verify (mutual auth)");
            write_line(&mut s, "").await;
            let ok = read_line(&mut s).await;
            ok.starts_with("+OK")
        }
    } else {
        next.starts_with("+OK")
    };
    assert!(
        final_ok,
        "SCRAM-SHA-256-PLUS via tls-exporter must COMPLETE POP3 login (server-validated \
         channel-bound proof)"
    );
    write_line(&mut s, "QUIT").await;
}
