//! Live submission smoke test against a real SMTP server (plan §3 e4, wired by
//! e7's Greenmail container). Env-gated and `#[ignore]` so `cargo test -p
//! mw-smtp` is green with no server present.
//!
//! Enable with a submission address in `GREENMAIL_SMTP` (host:port, cleartext —
//! Greenmail's default SMTP port is 3025) and run:
//!   GREENMAIL_SMTP=127.0.0.1:3025 cargo test -p mw-smtp -- --ignored

use mw_smtp::{Credentials, Outgoing, Security, SubmitConfig, Submitter};

#[tokio::test]
#[ignore = "requires a live SMTP server via GREENMAIL_SMTP"]
async fn submits_to_live_server() {
    let addr =
        std::env::var("GREENMAIL_SMTP").expect("GREENMAIL_SMTP must be set for the live test");
    let (host, port) = addr
        .rsplit_once(':')
        .expect("GREENMAIL_SMTP must be host:port");

    let sub = Submitter::new(SubmitConfig {
        host: host.to_string(),
        port: port.parse().expect("port"),
        // Greenmail's plain SMTP listener is cleartext; e7 also exposes a
        // STARTTLS/implicit-TLS port for the secured paths.
        security: Security::Plaintext,
        credentials: Credentials::None,
        ehlo_name: "mailwoman.test".to_string(),
    });

    let raw = b"From: sender@mailwoman.test\r\n\
        To: rcpt@mailwoman.test\r\n\
        Subject: mw-smtp live smoke\r\n\
        \r\n\
        Delivered by mw-smtp.\r\n"
        .to_vec();

    let result = sub
        .submit(Outgoing {
            mail_from: "sender@mailwoman.test".into(),
            rcpt_to: vec!["rcpt@mailwoman.test".into()],
            raw,
        })
        .await
        .expect("submission to live server");

    assert_eq!(result.accepted, vec!["rcpt@mailwoman.test".to_string()]);
    assert!(result.rejected.is_empty());
}
