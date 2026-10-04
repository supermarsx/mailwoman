//! Fixture-transcript submission tests (plan §3 e4 acceptance).
//!
//! Each test loads a server transcript from `fixtures/smtp/`, replays it through
//! the in-crate mock socket, and drives the real [`mw_smtp::Submitter`] over a
//! cleartext connection — exercising EHLO capability parse, the three SASL
//! mechanisms, MAIL/RCPT/DATA with per-recipient outcomes, and dot-stuffing.

mod common;

use base64::prelude::*;
use mw_smtp::{
    Credentials, Dsn, DsnNotify, DsnRet, Outgoing, Security, SmtpError, SubmitConfig,
    SubmitOptions, Submitter,
};

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

// ── envelope-address validation (26.20 t27-e2, SEC-2) ───────────────────────

/// Server replies for one unauthenticated session with one recipient. No
/// extension is advertised, so `MAIL FROM` and `RCPT TO` carry no parameters.
fn one_recipient_script() -> Vec<String> {
    [
        "220 mock ESMTP\r\n",
        "250 mock\r\n",
        "250 2.1.0 OK\r\n",
        "250 2.1.5 OK\r\n",
        "354 go ahead\r\n",
        "250 2.0.0 queued\r\n",
        "221 bye\r\n",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// The complete client side of the session [`one_recipient_script`] answers.
fn benign_transcript() -> Vec<String> {
    [
        "EHLO client.test",
        "MAIL FROM:<sender@example.com>",
        "RCPT TO:<good@example.com>",
        "DATA",
        "Subject: hi",
        "",
        "body",
        ".",
        "QUIT",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn benign() -> Outgoing {
    Outgoing {
        mail_from: SENDER.into(),
        rcpt_to: vec!["good@example.com".into()],
        raw: b"Subject: hi\r\n\r\nbody\r\n".to_vec(),
    }
}

/// Every option that adds text to `MAIL FROM` / `RCPT TO` or changes the body
/// framing: DSN `RET`/`ENVID`/`NOTIFY`/`ORCPT` and `BDAT`.
fn every_extension() -> SubmitOptions {
    SubmitOptions {
        dsn: Some(Dsn {
            ret: Some(DsnRet::Hdrs),
            envid: Some("abc123".into()),
            notify: vec![DsnNotify::Success, DsnNotify::Failure],
            orcpt: true,
        }),
        require_tls: false,
        use_chunking: true,
    }
}

/// Addresses that end the path, the command, or both.
const HOSTILE: &[&str] = &[
    "x@example.com>\r\nRCPT TO:<victim@example.com",
    "x@example.com\r\nRSET",
    "x@example.com\nRSET",
    "x@example.com\rRSET",
    "x@example.\0com",
    "x @example.com",
    "x@example.com> NOTIFY=NEVER",
    "a@b>",
    "",
];

/// Submit `hostile` (expected to be refused), then a benign message, to the
/// same single-connection mock, and require that the server saw exactly the
/// benign session and nothing else.
///
/// The benign submission is the positive control: it proves this harness does
/// record what the client writes. It is also what makes "nothing was sent"
/// checkable without a race — the mock accepts one connection and replays one
/// script, so had the hostile submission connected, it would have consumed
/// both and the benign one could not complete.
async fn assert_refused_with_nothing_on_the_wire(hostile: Outgoing, opts: SubmitOptions) {
    let mock = common::start(one_recipient_script()).await;
    let sub = submitter(mock.addr, Credentials::None);
    let what = format!("{:?} -> {:?}", hostile.mail_from, hostile.rcpt_to);

    // Bounded, so that a client which does connect and then waits on a server
    // it has desynchronised fails this test instead of hanging it.
    let refused = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        sub.submit_with(hostile, opts),
    )
    .await;
    match refused {
        Ok(Err(SmtpError::InvalidAddress(_))) => {}
        other => panic!("{what}: expected InvalidAddress, got {other:?}"),
    }

    let ok = sub
        .submit(benign())
        .await
        .unwrap_or_else(|e| panic!("{what}: the benign control was not delivered: {e}"));
    assert_eq!(ok.accepted, vec!["good@example.com".to_string()], "{what}");

    let connections = mock.connections();
    let sent = mock.captured();
    mock.join().await;
    assert_eq!(connections, 1, "{what}: only the benign message connects");
    assert_eq!(sent, benign_transcript(), "{what}");
}

#[tokio::test]
async fn the_benign_control_is_delivered_and_recorded() {
    let mock = common::start(one_recipient_script()).await;
    let sub = submitter(mock.addr, Credentials::None);
    let ok = sub.submit(benign()).await.expect("submission");
    assert_eq!(ok.accepted, vec!["good@example.com".to_string()]);
    let sent = mock.captured();
    mock.join().await;
    assert_eq!(sent, benign_transcript());
}

#[tokio::test]
async fn a_hostile_recipient_is_refused_before_connecting() {
    for bad in HOSTILE {
        let msg = Outgoing {
            rcpt_to: vec![bad.to_string()],
            ..benign()
        };
        assert_refused_with_nothing_on_the_wire(msg.clone(), SubmitOptions::default()).await;
        assert_refused_with_nothing_on_the_wire(msg, every_extension()).await;
    }
}

/// One bad recipient fails the whole message: the good recipients listed
/// before and after it are not sent to either.
#[tokio::test]
async fn one_hostile_recipient_among_good_ones_sends_to_nobody() {
    let msg = Outgoing {
        rcpt_to: vec![
            "first@example.com".into(),
            "x@example.com>\r\nRCPT TO:<victim@example.com".into(),
            "last@example.com".into(),
        ],
        ..benign()
    };
    assert_refused_with_nothing_on_the_wire(msg, SubmitOptions::default()).await;
}

#[tokio::test]
async fn a_hostile_sender_is_refused_before_connecting() {
    // The empty string is the null reverse-path and is valid for a sender, so
    // it is not in this list (see `the_null_reverse_path_is_still_sent`).
    for bad in HOSTILE.iter().filter(|b| !b.is_empty()) {
        let msg = Outgoing {
            mail_from: bad.to_string(),
            ..benign()
        };
        assert_refused_with_nothing_on_the_wire(msg.clone(), SubmitOptions::default()).await;
        assert_refused_with_nothing_on_the_wire(msg, every_extension()).await;
    }
}

#[tokio::test]
async fn the_null_reverse_path_is_still_sent() {
    let mock = common::start(one_recipient_script()).await;
    let sub = submitter(mock.addr, Credentials::None);
    sub.submit(Outgoing {
        mail_from: String::new(),
        ..benign()
    })
    .await
    .expect("a bounce-style message with MAIL FROM:<> is deliverable");
    let sent = mock.captured();
    mock.join().await;
    assert!(sent.iter().any(|l| l == "MAIL FROM:<>"), "{sent:?}");
}

/// The `EHLO` name is operator configuration, but it is interpolated into a
/// command the same way.
#[tokio::test]
async fn a_control_character_in_the_ehlo_name_is_refused_before_connecting() {
    let mock = common::start(one_recipient_script()).await;
    let bad = Submitter::new(SubmitConfig {
        host: mock.addr.ip().to_string(),
        port: mock.addr.port(),
        security: Security::Plaintext,
        credentials: Credentials::None,
        ehlo_name: "client.test\r\nMAIL FROM:<forged@example.com>".to_string(),
    });
    let err = bad.submit(benign()).await.expect_err("refused");
    assert!(matches!(err, SmtpError::Protocol(_)), "{err:?}");

    let sub = submitter(mock.addr, Credentials::None);
    sub.submit(benign()).await.expect("benign control");
    let connections = mock.connections();
    let sent = mock.captured();
    mock.join().await;
    assert_eq!(connections, 1);
    assert_eq!(sent, benign_transcript());
}
