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

// ── line ends in DATA and SMTPUTF8 (26.20 t27-f2, SEC-2 residuals) ──────────

/// A server that keeps every byte the client sends, as sent.
///
/// It reads the way RFC 5321 says to: a command ends at CRLF, and the message
/// ends at `<CRLF>.<CRLF>` and nowhere else. `EHLO` is answered with
/// `extensions`; everything else is accepted. The task's result is the whole
/// client side of the connection.
async fn recording_server(
    extensions: &'static [&'static str],
) -> (std::net::SocketAddr, tokio::task::JoinHandle<Vec<u8>>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        let mut seen: Vec<u8> = Vec::new();
        // Everything before `done` has been acted on.
        let mut done = 0;
        let mut in_data = false;
        sock.write_all(b"220 recorder ESMTP\r\n").await.unwrap();
        loop {
            let end = if in_data {
                &b"\r\n.\r\n"[..]
            } else {
                &b"\r\n"[..]
            };
            // The CRLF that ended `DATA` also begins `<CRLF>.<CRLF>`.
            let from = if in_data { done - 2 } else { done };
            let found = seen[from..]
                .windows(end.len())
                .position(|w| w == end)
                .map(|p| from + p + end.len());
            let Some(next) = found else {
                let mut tmp = [0u8; 4096];
                match sock.read(&mut tmp).await {
                    Ok(n) if n > 0 => seen.extend_from_slice(&tmp[..n]),
                    _ => return seen,
                }
                continue;
            };
            let unit = seen[done..next].to_vec();
            done = next;
            let reply: String = if in_data {
                in_data = false;
                "250 2.0.0 queued\r\n".into()
            } else if unit.starts_with(b"EHLO") {
                let mut lines = vec!["recorder"];
                lines.extend(extensions);
                let last = lines.len() - 1;
                lines
                    .iter()
                    .enumerate()
                    .map(|(n, l)| format!("250{}{l}\r\n", if n == last { ' ' } else { '-' }))
                    .collect()
            } else if unit == b"DATA\r\n" {
                in_data = true;
                "354 go ahead\r\n".into()
            } else if unit == b"QUIT\r\n" {
                let _ = sock.write_all(b"221 bye\r\n").await;
                return seen;
            } else {
                "250 OK\r\n".into()
            };
            if sock.write_all(reply.as_bytes()).await.is_err() {
                return seen;
            }
        }
    });
    (addr, handle)
}

/// Submit `msg` to a [`recording_server`] and return the outcome with every
/// byte the client wrote.
async fn submit_recorded(
    extensions: &'static [&'static str],
    msg: Outgoing,
) -> (Result<mw_smtp::SubmissionResult, SmtpError>, Vec<u8>) {
    let (addr, server) = recording_server(extensions).await;
    let sub = submitter(addr, Credentials::None);
    let out = tokio::time::timeout(std::time::Duration::from_secs(10), sub.submit(msg))
        .await
        .expect("the submission ends");
    // The client has dropped its socket by now, so the server task ends too.
    (out, server.await.unwrap())
}

const ENVELOPE: &[u8] =
    b"EHLO client.test\r\nMAIL FROM:<sender@example.com>\r\nRCPT TO:<good@example.com>\r\nDATA\r\n";

/// What the client wrote between `DATA` and `QUIT`, once it is established
/// that the rest of the connection is exactly the benign envelope and `QUIT`.
fn data_section(wire: &[u8]) -> &[u8] {
    let shown = String::from_utf8_lossy(wire);
    let rest = wire
        .strip_prefix(ENVELOPE)
        .unwrap_or_else(|| panic!("unexpected commands before the message:\n{shown}"));
    rest.strip_suffix(b"QUIT\r\n")
        .unwrap_or_else(|| panic!("the connection does not end with QUIT:\n{shown}"))
}

/// Split at every line end a reader might honour: CRLF, a bare LF, a bare CR.
fn lenient_lines(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut lines = vec![];
    let mut line = vec![];
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\r' | b'\n' => {
                if bytes[i] == b'\r' && bytes.get(i + 1) == Some(&b'\n') {
                    i += 1;
                }
                lines.push(std::mem::take(&mut line));
            }
            b => line.push(b),
        }
        i += 1;
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Undo dot-stuffing on the lines of a `DATA` section and drop the final `.`.
fn unstuffed(mut lines: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    assert_eq!(lines.pop().as_deref(), Some(&b"."[..]), "terminator");
    for l in &mut lines {
        if l.first() == Some(&b'.') {
            l.remove(0);
        }
    }
    lines
}

fn message(body: &[u8]) -> Outgoing {
    let mut raw = b"Subject: hi\r\n\r\n".to_vec();
    raw.extend_from_slice(body);
    Outgoing {
        mail_from: SENDER.into(),
        rcpt_to: vec!["good@example.com".into()],
        raw,
    }
}

/// Precondition for the test after it: a message with CRLF line ends, a Base64
/// part and a quoted-printable part arrives byte for byte, apart from the
/// doubled leading dot.
#[tokio::test]
async fn a_crlf_message_with_base64_and_quoted_printable_parts_arrives_unchanged() {
    // Every byte value, CR, LF and `.` among them: Base64 text is what crosses.
    let binary: Vec<u8> = (0..=255u8).chain(*b"\r.\r\n.\n.").collect();
    let mut b64 = String::new();
    for chunk in BASE64_STANDARD.encode(&binary).as_bytes().chunks(76) {
        b64.push_str(std::str::from_utf8(chunk).unwrap());
        b64.push_str("\r\n");
    }
    let body = format!(
        "--b\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         Content-Transfer-Encoding: quoted-printable\r\n\
         \r\n\
         caf=C3=A9 and a soft line break=\r\n\
         .a line that starts with a dot\r\n\
         .\r\n\
         =2E\r\n\
         trailing space=20\r\n\
         --b\r\n\
         Content-Type: application/octet-stream\r\n\
         Content-Transfer-Encoding: base64\r\n\
         \r\n\
         {b64}\
         --b--\r\n"
    );
    let msg = message(body.as_bytes());
    let raw = String::from_utf8(msg.raw.clone()).unwrap();
    let (out, wire) = submit_recorded(&[], msg).await;
    out.expect("delivered");

    // The two lines that begin with `.` gain one; nothing else differs.
    let data = data_section(&wire);
    let expected = format!("{}.\r\n", raw.replace("\r\n.", "\r\n.."));
    assert_eq!(String::from_utf8_lossy(data), expected);

    // And the Base64 part still decodes to the bytes it was made from.
    let text = String::from_utf8(data.to_vec()).unwrap();
    let part = text.split("base64\r\n\r\n").nth(1).unwrap();
    let encoded: String = part.split("--b--").next().unwrap().split("\r\n").collect();
    assert_eq!(BASE64_STANDARD.decode(encoded).unwrap(), binary);
}

/// A body may hold a bare CR or a bare LF. Whatever it holds, the message is
/// sent with CRLF line ends only, so a server that takes a bare CR or LF for a
/// line end reads the same lines as one that does not, and the only `.` line is
/// the one that ends the message.
#[tokio::test]
async fn no_body_ends_the_message_early_for_a_strict_or_a_lenient_server() {
    let bodies: &[&[u8]] = &[
        // The verifier's reproduction.
        b"hi\r.\rMAIL FROM:<evil@x.test>\rRCPT TO:<victim@x.test>\rDATA\rx",
        b"hi\n.\nMAIL FROM:<evil@x.test>\nRCPT TO:<victim@x.test>\nDATA\nx\n.\n",
        b"hi\r.\nMAIL FROM:<evil@x.test>\r\n",
        b"hi\n.\rMAIL FROM:<evil@x.test>\r\n",
        b"hi\r\n.\rMAIL FROM:<evil@x.test>\r\n",
        b"hi\r.\r\nMAIL FROM:<evil@x.test>\r\n",
        b"hi\r\r\n.\r\r\nQUIT\r\r\n",
        b"hi\n\r.\n\rQUIT\n\r",
        b"lone dot lines\r\n.\r\n.\r\n..\r\nend\r\n",
        b".\r\n",
        b".",
        b"\r.",
        b"\n.",
        b"ends in a lone CR\r",
        b"ends in a lone LF\n",
        b"ends in a dot after a lone CR\r.\r",
        b"\r",
        b"",
    ];
    for body in bodies {
        let shown = String::from_utf8_lossy(body).into_owned();
        let msg = message(body);
        let raw = msg.raw.clone();
        let (out, wire) = submit_recorded(&[], msg).await;
        let out = out.unwrap_or_else(|e| panic!("{shown:?}: {e}"));
        assert_eq!(out.accepted, ["good@example.com"], "{shown:?}");

        // `data_section` has checked that nothing but the envelope precedes
        // the message and nothing but QUIT follows it.
        let data = data_section(&wire);
        let seen = String::from_utf8_lossy(data).into_owned();
        for (i, &b) in data.iter().enumerate() {
            let paired = match b {
                b'\r' => data.get(i + 1) == Some(&b'\n'),
                b'\n' => i > 0 && data[i - 1] == b'\r',
                _ => true,
            };
            assert!(paired, "{shown:?}: a bare CR or LF at byte {i} of {seen:?}");
        }
        let lines = lenient_lines(data);
        let dots = lines.iter().filter(|l| l.as_slice() == b".").count();
        assert_eq!(dots, 1, "{shown:?}: `.` lines in {seen:?}");
        // The content is there as data: the same lines, read leniently.
        assert_eq!(
            unstuffed(lines),
            lenient_lines(&raw),
            "{shown:?}: sent as {seen:?}"
        );
    }
}

const UTF8_ADDRESS: &str = "jörg@example.com";

/// A server that did not offer `SMTPUTF8` is not sent a non-ASCII address, as
/// sender or as recipient, and the message is not sent to anyone.
#[tokio::test]
async fn a_non_ascii_address_is_not_sent_to_a_server_without_smtputf8() {
    let as_sender = Outgoing {
        mail_from: UTF8_ADDRESS.into(),
        ..benign()
    };
    let as_recipient = Outgoing {
        rcpt_to: vec!["good@example.com".into(), UTF8_ADDRESS.into()],
        ..benign()
    };
    for (what, msg) in [("sender", as_sender), ("recipient", as_recipient)] {
        let (out, wire) = submit_recorded(&["8BITMIME", "SIZE 1000000"], msg).await;
        let err = out.expect_err(what).to_string();
        assert!(err.contains("SMTPUTF8"), "{what}: {err}");
        assert!(err.contains(UTF8_ADDRESS), "{what}: {err}");
        let shown = String::from_utf8_lossy(&wire);
        assert!(wire.is_ascii(), "{what}: non-ASCII on the wire:\n{shown}");
        assert!(!shown.contains("SMTPUTF8"), "{what}:\n{shown}");
        assert!(
            !shown.contains("DATA"),
            "{what}: a message was sent:\n{shown}"
        );
    }
}

/// With `SMTPUTF8` offered, the parameter is on `MAIL FROM` and the addresses
/// go out as they are (the domain is not converted to A-labels).
#[tokio::test]
async fn a_non_ascii_address_is_sent_with_the_smtputf8_parameter_when_offered() {
    let as_sender = Outgoing {
        mail_from: UTF8_ADDRESS.into(),
        ..benign()
    };
    let as_recipient = Outgoing {
        rcpt_to: vec!["good@example.com".into(), "jörg@bücher.example".into()],
        ..benign()
    };
    for (what, msg, mail, rcpt) in [
        (
            "sender",
            as_sender,
            "MAIL FROM:<jörg@example.com> SMTPUTF8\r\n",
            "RCPT TO:<good@example.com>\r\n",
        ),
        (
            "recipient",
            as_recipient,
            "MAIL FROM:<sender@example.com> SMTPUTF8\r\n",
            "RCPT TO:<jörg@bücher.example>\r\n",
        ),
    ] {
        let (out, wire) = submit_recorded(&["SMTPUTF8"], msg).await;
        let shown = String::from_utf8_lossy(&wire).into_owned();
        out.unwrap_or_else(|e| panic!("{what}: {e}\n{shown}"));
        assert!(shown.contains(mail), "{what}:\n{shown}");
        assert!(shown.contains(rcpt), "{what}:\n{shown}");
        assert!(shown.ends_with("\r\n.\r\nQUIT\r\n"), "{what}:\n{shown}");
    }
}

/// An ASCII envelope does not get the parameter, offered or not.
#[tokio::test]
async fn an_ascii_envelope_carries_no_smtputf8_parameter() {
    for extensions in [&["SMTPUTF8"][..], &[][..]] {
        let (out, wire) = submit_recorded(extensions, benign()).await;
        out.expect("delivered");
        let mut expected = ENVELOPE.to_vec();
        expected.extend_from_slice(b"Subject: hi\r\n\r\nbody\r\n.\r\nQUIT\r\n");
        assert_eq!(
            String::from_utf8_lossy(&wire),
            String::from_utf8_lossy(&expected)
        );
    }
}

/// The refusal comes before `MAIL FROM`, wherever the non-ASCII address is:
/// the client says `EHLO`, learns that `SMTPUTF8` is not offered, and leaves.
/// The error is its own variant, so a caller can tell that a retry will not
/// help.
#[tokio::test]
async fn a_message_that_needs_smtputf8_is_refused_before_mail_from() {
    let last_recipient = Outgoing {
        rcpt_to: vec![
            "good@example.com".into(),
            "other@example.com".into(),
            UTF8_ADDRESS.into(),
        ],
        ..benign()
    };
    let sender = Outgoing {
        mail_from: UTF8_ADDRESS.into(),
        ..benign()
    };
    for (what, msg) in [("recipient", last_recipient), ("sender", sender)] {
        let (out, wire) = submit_recorded(&["SIZE 1000000"], msg).await;
        match out {
            Err(SmtpError::SmtpUtf8Required(addr)) => assert!(addr.contains(UTF8_ADDRESS)),
            other => panic!("{what}: expected SmtpUtf8Required, got {other:?}"),
        }
        assert_eq!(
            String::from_utf8_lossy(&wire),
            "EHLO client.test\r\nQUIT\r\n",
            "{what}"
        );
    }
}

/// `SIZE=` declares the message as it is transmitted (RFC 1870 §5): every line
/// end counted as CRLF, without the dots added by stuffing and without the
/// final `.` line.
#[tokio::test]
async fn the_declared_size_is_the_size_of_what_is_transmitted() {
    let bodies: &[&[u8]] = &[
        b"body\r\n",
        b"bare\nline\nfeeds\n",
        b"bare\rcarriage\rreturns",
        b".dots\r\n.\r\n..\r\nand\na\rmix\r\n.",
        b"",
    ];
    for body in bodies {
        let shown = String::from_utf8_lossy(body).into_owned();
        let (out, wire) = submit_recorded(&["SIZE 1000000"], message(body)).await;
        out.unwrap_or_else(|e| panic!("{shown:?}: {e}"));

        let text = String::from_utf8(wire).unwrap();
        let (envelope, data) = text.split_once("\r\nDATA\r\n").expect("a DATA command");
        let declared: usize = envelope
            .lines()
            .find_map(|l| l.strip_prefix("MAIL FROM:<sender@example.com> SIZE="))
            .unwrap_or_else(|| panic!("{shown:?}: no SIZE on MAIL FROM:\n{envelope}"))
            .parse()
            .expect("a number");

        let data = data
            .strip_suffix(".\r\nQUIT\r\n")
            .expect("terminator, QUIT");
        // Every line that begins with `.` carries one dot that is not content.
        let stuffed = data.split("\r\n").filter(|l| l.starts_with('.')).count();
        assert_eq!(declared, data.len() - stuffed, "{shown:?}: sent {data:?}");
    }
}
