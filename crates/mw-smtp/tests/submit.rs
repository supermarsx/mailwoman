//! Fixture-transcript submission tests (plan §3 e4 acceptance).
//!
//! Each test loads a server transcript from `fixtures/smtp/`, replays it through
//! the in-crate mock socket, and drives the real [`mw_smtp::Submitter`] over a
//! cleartext connection — exercising EHLO capability parse, the three SASL
//! mechanisms, MAIL/RCPT/DATA with per-recipient outcomes, and dot-stuffing.

mod common;

use base64::prelude::*;
use mw_smtp::{Credentials, Outgoing, Security, SubmitConfig, Submitter};

const SENDER: &str = "sender@example.com";

fn fixture(name: &str) -> Vec<String> {
    common::load_script(&format!(
        "{}/../../fixtures/smtp/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
}

fn submitter(addr: std::net::SocketAddr, credentials: Credentials) -> Submitter {
    Submitter::new(SubmitConfig {
        host: addr.ip().to_string(),
        port: addr.port(),
        security: Security::Plaintext,
        credentials,
        ehlo_name: "client.test".to_string(),
    })
}

/// A body whose third line is a lone `.` — must be dot-stuffed to `..` on the
/// wire so it is not read as the end-of-data marker.
fn body_with_dot_line() -> Vec<u8> {
    b"From: sender@example.com\r\n\
      To: good@example.com\r\n\
      Subject: Test\r\n\
      \r\n\
      Hello.\r\n\
      .\r\n\
      After the dot line.\r\n"
        .to_vec()
}

#[tokio::test]
async fn auth_plain_happy_path_with_per_recipient_outcome() {
    let mock = common::start(fixture("auth_plain_session.txt")).await;
    let sub = submitter(
        mock.addr,
        Credentials::Plain {
            user: "alice@example.com".into(),
            pass: "s3cret".into(),
        },
    );

    let result = sub
        .submit(Outgoing {
            mail_from: SENDER.into(),
            rcpt_to: vec!["good@example.com".into(), "bad@example.com".into()],
            raw: body_with_dot_line(),
        })
        .await
        .expect("submission");

    assert_eq!(result.accepted, vec!["good@example.com".to_string()]);
    assert_eq!(result.rejected.len(), 1);
    assert_eq!(result.rejected[0].0, "bad@example.com");
    assert!(
        result.rejected[0].1.contains("550"),
        "reject reason carries the server code: {:?}",
        result.rejected[0].1
    );

    let sent = mock.captured();
    mock.join().await;

    assert!(sent.iter().any(|l| l == "EHLO client.test"));

    // AUTH PLAIN initial-response frame decodes to \0user\0pass.
    let auth = sent
        .iter()
        .find(|l| l.starts_with("AUTH PLAIN "))
        .expect("AUTH PLAIN line");
    let ir = auth.strip_prefix("AUTH PLAIN ").unwrap();
    let decoded = BASE64_STANDARD.decode(ir).unwrap();
    assert_eq!(decoded, b"\0alice@example.com\0s3cret");

    // SIZE was advertised, so MAIL FROM carries it (ASCII body ⇒ no 8BITMIME).
    let mail = sent
        .iter()
        .find(|l| l.starts_with("MAIL FROM:"))
        .expect("MAIL FROM line");
    assert!(
        mail.starts_with("MAIL FROM:<sender@example.com> SIZE="),
        "{mail}"
    );
    assert!(!mail.contains("BODY=8BITMIME"));

    assert!(sent.iter().any(|l| l == "RCPT TO:<good@example.com>"));
    assert!(sent.iter().any(|l| l == "RCPT TO:<bad@example.com>"));
    assert!(sent.iter().any(|l| l == "DATA"));
    // The lone "." line was dot-stuffed to ".." on the wire.
    assert!(sent.iter().any(|l| l == ".."), "dot-stuffed line present");
    assert!(sent.iter().any(|l| l == "QUIT"));
}

#[tokio::test]
async fn auth_login_challenge_response() {
    let mock = common::start(fixture("auth_login_session.txt")).await;
    let sub = submitter(
        mock.addr,
        Credentials::Login {
            user: "bob@example.com".into(),
            pass: "hunter2".into(),
        },
    );

    let result = sub
        .submit(Outgoing {
            mail_from: SENDER.into(),
            rcpt_to: vec!["rcpt@example.com".into()],
            raw: b"Subject: hi\r\n\r\nbody\r\n".to_vec(),
        })
        .await
        .expect("submission");

    assert_eq!(result.accepted, vec!["rcpt@example.com".to_string()]);
    assert!(result.rejected.is_empty());

    let sent = mock.captured();
    mock.join().await;

    assert!(sent.iter().any(|l| l == "AUTH LOGIN"));
    // The two base64 steps carry username then password.
    let user_step = BASE64_STANDARD
        .decode(
            sent.iter()
                .find(|l| **l == BASE64_STANDARD.encode("bob@example.com"))
                .expect("username step"),
        )
        .unwrap();
    assert_eq!(user_step, b"bob@example.com");
    let pass_step = BASE64_STANDARD
        .decode(
            sent.iter()
                .find(|l| **l == BASE64_STANDARD.encode("hunter2"))
                .expect("password step"),
        )
        .unwrap();
    assert_eq!(pass_step, b"hunter2");
}

#[tokio::test]
async fn auth_xoauth2_initial_response_frame() {
    let mock = common::start(fixture("auth_xoauth2_session.txt")).await;
    let sub = submitter(
        mock.addr,
        Credentials::XOAuth2 {
            user: "carol@example.com".into(),
            token: "ya29.A0ARR".into(),
        },
    );

    let result = sub
        .submit(Outgoing {
            mail_from: SENDER.into(),
            rcpt_to: vec!["rcpt@example.com".into()],
            raw: b"Subject: hi\r\n\r\nbody\r\n".to_vec(),
        })
        .await
        .expect("submission");

    assert_eq!(result.accepted, vec!["rcpt@example.com".to_string()]);

    let sent = mock.captured();
    mock.join().await;

    let auth = sent
        .iter()
        .find(|l| l.starts_with("AUTH XOAUTH2 "))
        .expect("AUTH XOAUTH2 line");
    let ir = auth.strip_prefix("AUTH XOAUTH2 ").unwrap();
    let decoded = BASE64_STANDARD.decode(ir).unwrap();
    assert_eq!(
        decoded,
        b"user=carol@example.com\x01auth=Bearer ya29.A0ARR\x01\x01"
    );

    // SIZE was NOT advertised in this transcript ⇒ MAIL FROM has no SIZE param.
    let mail = sent
        .iter()
        .find(|l| l.starts_with("MAIL FROM:"))
        .expect("MAIL FROM line");
    assert_eq!(mail, "MAIL FROM:<sender@example.com>");
}
