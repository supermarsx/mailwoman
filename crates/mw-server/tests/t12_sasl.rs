//! t12-e-e2e-backend — IMAP SASL SCRAM-SHA-256 + OAUTHBEARER live-E2E (audit #5).
//!
//! The "unit-green ≠ wired" gate for mw-imap's new SASL client (`crates/mw-imap/
//! src/sasl.rs`, dispatched from `session.rs`). The RFC 7677 SCRAM proof math is
//! pinned by the in-crate vector test; THIS leg proves the whole challenge/response
//! state machine round-trips against a REAL server (Dovecot) that advertises ONLY
//! `AUTH=SCRAM-SHA-256` — so a successful `Session::login` can only have negotiated
//! SCRAM (no silent PLAIN/LOGIN fallback).
//!
//! ## Live leg (gated `MW_IMAP_LIVE=1`, loud-skip otherwise)
//!   docker compose -f docker-compose.ci.yml up -d --wait dovecot-sasl
//!   MW_IMAP_LIVE=1 cargo test -p mw-server --test t12_sasl -- --nocapture
//!
//! The Dovecot at `MW_IMAP_LIVE_HOST` (default 127.0.0.1) : `MW_IMAP_LIVE_PORT`
//! (default 3243) authenticates the seeded testuser/testpass account over SCRAM.
//!
//! ## Always-on leg (default `cargo test`)
//! The OAUTHBEARER (RFC 7628) initial-response frame shape is asserted offline —
//! Dovecot OAUTHBEARER needs an oauth2 introspection backend, so the live token
//! path is covered at the frame level here and end-to-end by the vector tests.

use mw_imap::session::{Credentials, SelectMode, Session};
use mw_imap::transport::TlsMode;

fn live() -> bool {
    std::env::var("MW_IMAP_LIVE").ok().as_deref() == Some("1")
}

fn host() -> String {
    std::env::var("MW_IMAP_LIVE_HOST").unwrap_or_else(|_| "127.0.0.1".into())
}

fn port() -> u16 {
    std::env::var("MW_IMAP_LIVE_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3243)
}

const USER: &str = "testuser";
const PASS: &str = "testpass";

/// Connect + probe capabilities against the live SCRAM-only Dovecot. Returns the
/// session on success, or `None` after a LOUD skip banner (never a silent skip).
async fn connect_probe(scenario: &str) -> Option<Session> {
    match Session::connect(&host(), port(), TlsMode::Plaintext).await {
        Ok(mut s) => {
            if let Err(e) = s.probe_capabilities().await {
                eprintln!("\n[t12 IMAP SKIP] {scenario}: CAPABILITY probe failed ({e}).");
                return None;
            }
            Some(s)
        }
        Err(e) => {
            eprintln!(
                "\n[t12 IMAP SKIP] {scenario}: Dovecot-SASL unreachable at {}:{} ({e}). \
                 Bring it up: docker compose -f docker-compose.ci.yml up -d --wait \
                 dovecot-sasl ; then MW_IMAP_LIVE=1 cargo test -p mw-server --test t12_sasl.\n",
                host(),
                port()
            );
            None
        }
    }
}

/// SCRAM-SHA-256 is advertised and `Session::login` negotiates it against real Dovecot.
#[tokio::test]
async fn imap_scram_sha256_login_live() {
    if !live() {
        eprintln!(
            "\n[t12 IMAP SKIP] MW_IMAP_LIVE!=1 — real Dovecot SCRAM not driven. See module doc.\n"
        );
        return;
    }
    let Some(mut session) = connect_probe("imap_scram_sha256_login_live").await else {
        return;
    };

    // The server offers SCRAM-SHA-256 (and, being SCRAM-only, NOTHING weaker) — so a
    // successful login is a genuine SCRAM negotiation, not a PLAIN fallback.
    let caps = session.backend_caps();
    assert!(
        caps.sasl_scram_sha256,
        "Dovecot-SASL must advertise AUTH=SCRAM-SHA-256 (backend_caps={caps:?})"
    );

    session
        .login(&Credentials::Password {
            username: USER.into(),
            password: PASS.into(),
        })
        .await
        .expect("SCRAM-SHA-256 login must succeed against the real server");

    // Prove the session is genuinely authenticated: SELECT the seeded INBOX.
    let sel = session
        .select("INBOX", SelectMode::Plain)
        .await
        .expect("SELECT INBOX after SCRAM login");
    assert_eq!(
        sel.exists, 4,
        "the seeded INBOX has 4 messages (proves post-SCRAM authenticated access)"
    );
    let _ = session.logout().await;
}

/// A wrong password over SCRAM is rejected as an auth failure (the server-side proof
/// verification fails) — not a hang, not a transport error.
#[tokio::test]
async fn imap_scram_wrong_password_rejected_live() {
    if !live() {
        return;
    }
    let Some(mut session) = connect_probe("imap_scram_wrong_password_rejected_live").await else {
        return;
    };
    let err = session
        .login(&Credentials::Password {
            username: USER.into(),
            password: "not-the-password".into(),
        })
        .await
        .expect_err("a bad SCRAM password must be rejected");
    assert!(
        matches!(err, mw_imap::ImapError::Auth(_)),
        "wrong SCRAM password surfaces as an auth failure, got {err:?}"
    );
}

/// OAUTHBEARER (RFC 7628) initial-response frame shape — always-on (no oauth2 backend
/// needed). The `gs2,a=user,^Aauth=Bearer <tok>^A^A` envelope is base64-verified.
#[tokio::test]
async fn imap_oauthbearer_frame_shape() {
    use base64::Engine as _;
    let frame =
        mw_imap::sasl::oauthbearer("carol@example.com", "vF9dft4qmT", "imap.example.com", 993);
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(frame.trim())
        .expect("OAUTHBEARER frame is base64");
    let text = String::from_utf8(decoded).expect("utf-8");
    assert!(
        text.starts_with("n,a=carol@example.com,"),
        "gs2 + authzid: {text:?}"
    );
    assert!(
        text.contains("\x01auth=Bearer vF9dft4qmT\x01\x01"),
        "RFC 7628 auth field with the bearer token: {text:?}"
    );
}
