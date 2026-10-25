//! The ManageSieve commands and SASL mechanisms added in t32 f18, driven over an
//! in-memory duplex: CHECKSCRIPT, HAVESPACE, RENAMESCRIPT (and its fallback for
//! servers older than RFC 5804 1.0), response codes on refusals, PUTSCRIPT
//! warnings, OAUTHBEARER / XOAUTH2, SASL cancellation and the response-size
//! bound. The exact client bytes are asserted, because a wrong literal length
//! or a missing quote fails against every real server.

use mw_sieve::SieveError;
use mw_sieve::managesieve::{Checked, Connection, Credentials, MAX_RESPONSE};
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};

const GREETING_V1: &str = concat!(
    "\"IMPLEMENTATION\" \"Example ManageSieve v1\"\r\n",
    "\"SIEVE\" \"fileinto vacation\"\r\n",
    "\"SASL\" \"PLAIN OAUTHBEARER XOAUTH2\"\r\n",
    "\"VERSION\" \"1.0\"\r\n",
    "OK \"ready\"\r\n",
);

/// A pre-RFC-5804 server: no VERSION capability, so no RENAMESCRIPT.
const GREETING_OLD: &str = concat!(
    "\"IMPLEMENTATION\" \"Old timsieved\"\r\n",
    "\"SIEVE\" \"fileinto\"\r\n",
    "\"SASL\" \"PLAIN\"\r\n",
    "OK\r\n",
);

async fn connected(greeting: &str) -> (Connection<DuplexStream>, DuplexStream) {
    let (client, mut server) = tokio::io::duplex(1 << 18);
    server.write_all(greeting.as_bytes()).await.unwrap();
    let conn = Connection::open(client).await.expect("greeting");
    (conn, server)
}

async fn sent(server: &mut DuplexStream) -> String {
    let mut out = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        match tokio::time::timeout(std::time::Duration::from_millis(50), server.read(&mut buf))
            .await
        {
            Ok(Ok(0)) | Err(_) => break,
            Ok(Ok(n)) => out.extend_from_slice(&buf[..n]),
            Ok(Err(e)) => panic!("server read: {e}"),
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn refusal(err: SieveError) -> mw_sieve::Refusal {
    match err {
        SieveError::Refused(r) => r,
        other => panic!("expected a server refusal, got {other:?}"),
    }
}

// ── CHECKSCRIPT ────────────────────────────────────────────────────────────

#[tokio::test]
async fn checkscript_valid_with_and_without_warnings() {
    let (mut conn, mut server) = connected(GREETING_V1).await;
    server
        .write_all(b"OK\r\nOK (WARNINGS) \"line 1: warning: unused\"\r\n")
        .await
        .unwrap();

    assert_eq!(
        conn.check_script("keep;").await.unwrap(),
        Checked::Valid { warnings: None }
    );
    assert_eq!(
        conn.check_script("stop;").await.unwrap(),
        Checked::Valid {
            warnings: Some("line 1: warning: unused".into())
        }
    );
    assert_eq!(
        sent(&mut server).await,
        "CHECKSCRIPT {5+}\r\nkeep;\r\nCHECKSCRIPT {5+}\r\nstop;\r\n"
    );
}

/// A compile error comes back with the line the server named (Pigeonhole's
/// text, delivered as a literal the way Pigeonhole sends multi-line errors).
#[tokio::test]
async fn checkscript_error_carries_line_positions() {
    let (mut conn, mut server) = connected(GREETING_V1).await;
    let msg = "line 2: error: unknown command 'frob' (only reported once at first occurrence).\r\n\
               error: validation failed.";
    server
        .write_all(format!("NO {{{}}}\r\n{msg}\r\n", msg.len()).as_bytes())
        .await
        .unwrap();
    match conn.check_script("keep;\r\nfrob;\r\n").await.unwrap() {
        Checked::Invalid { problems, text } => {
            assert_eq!(text, msg);
            assert_eq!(problems.len(), 1, "{problems:?}");
            assert_eq!(problems[0].line, Some(2));
            assert!(problems[0].message.contains("unknown command 'frob'"));
        }
        other => panic!("expected Invalid, got {other:?}"),
    }
    // The literal length counts bytes: 14 for the two lines with CRLF.
    assert!(sent(&mut server).await.starts_with("CHECKSCRIPT {14+}\r\n"));
}

#[tokio::test]
async fn checkscript_bye_is_an_error() {
    let (mut conn, mut server) = connected(GREETING_V1).await;
    server
        .write_all(b"BYE \"shutting down\"\r\n")
        .await
        .unwrap();
    let r = refusal(conn.check_script("keep;").await.unwrap_err());
    assert!(r.bye);
    assert_eq!(r.text, "shutting down");
}

// ── HAVESPACE and quota codes ──────────────────────────────────────────────

#[tokio::test]
async fn havespace_ok_and_quota_codes_are_surfaced() {
    let (mut conn, mut server) = connected(GREETING_V1).await;
    server
        .write_all(b"OK\r\nNO (QUOTA/MAXSIZE) \"Script is too large\"\r\nNO (QUOTA/MAXSCRIPTS) \"Too many scripts\"\r\n")
        .await
        .unwrap();

    conn.have_space("mailwoman", 1024).await.expect("fits");
    let r = refusal(conn.have_space("big", 9_999_999).await.unwrap_err());
    let code = r.code.expect("code");
    assert_eq!(code.name, "QUOTA/MAXSIZE");
    assert!(code.is_quota());
    assert_eq!(r.text, "Script is too large");
    let r = refusal(conn.have_space("another", 10).await.unwrap_err());
    assert_eq!(r.code.as_ref().unwrap().name, "QUOTA/MAXSCRIPTS");
    assert!(r.to_string().contains("QUOTA/MAXSCRIPTS"), "{r}");

    assert_eq!(
        sent(&mut server).await,
        "HAVESPACE \"mailwoman\" 1024\r\nHAVESPACE \"big\" 9999999\r\nHAVESPACE \"another\" 10\r\n"
    );
}

#[tokio::test]
async fn putscript_quota_refusal_and_warnings() {
    let (mut conn, mut server) = connected(GREETING_V1).await;
    server
        .write_all(
            b"NO (QUOTA) \"Quota exceeded\"\r\nOK (WARNINGS) \"line 3: deprecated\"\r\nOK\r\n",
        )
        .await
        .unwrap();

    let r = refusal(conn.put_script("s", "keep;").await.unwrap_err());
    assert!(r.code.unwrap().is_quota());
    assert_eq!(
        conn.put_script("s", "keep;").await.unwrap().as_deref(),
        Some("line 3: deprecated")
    );
    assert_eq!(conn.put_script("s", "keep;").await.unwrap(), None);
}

// ── RENAMESCRIPT ───────────────────────────────────────────────────────────

#[tokio::test]
async fn renamescript_on_an_rfc5804_server() {
    let (mut conn, mut server) = connected(GREETING_V1).await;
    server
        .write_all(b"OK\r\nNO (ALREADYEXISTS) \"exists\"\r\n")
        .await
        .unwrap();
    conn.rename_script("old", "new \"one\"").await.unwrap();
    let r = refusal(conn.rename_script("a", "b").await.unwrap_err());
    assert_eq!(r.code.unwrap().name, "ALREADYEXISTS");
    assert_eq!(
        sent(&mut server).await,
        "RENAMESCRIPT \"old\" \"new \\\"one\\\"\"\r\nRENAMESCRIPT \"a\" \"b\"\r\n"
    );
}

/// RFC 5804 §2.11.1: without RENAMESCRIPT, fetch, store under the new name,
/// carry the active flag over, then delete the old one — in that order, so a
/// failure part-way never loses the script.
#[tokio::test]
async fn renamescript_falls_back_on_an_old_server() {
    let (mut conn, mut server) = connected(GREETING_OLD).await;
    server
        .write_all(
            b"\"old\" ACTIVE\r\n\"other\"\r\nOK\r\n\
              {5}\r\nkeep;\r\nOK\r\n\
              OK\r\n\
              OK\r\n\
              OK\r\n",
        )
        .await
        .unwrap();
    conn.rename_script("old", "new").await.unwrap();
    assert_eq!(
        sent(&mut server).await,
        "LISTSCRIPTS\r\nGETSCRIPT \"old\"\r\nPUTSCRIPT \"new\" {5+}\r\nkeep;\r\n\
         SETACTIVE \"new\"\r\nDELETESCRIPT \"old\"\r\n"
    );
}

#[tokio::test]
async fn renamescript_fallback_refuses_an_existing_target() {
    let (mut conn, mut server) = connected(GREETING_OLD).await;
    server
        .write_all(b"\"old\"\r\n\"new\"\r\nOK\r\n")
        .await
        .unwrap();
    let r = refusal(conn.rename_script("old", "new").await.unwrap_err());
    assert_eq!(r.code.unwrap().name, "ALREADYEXISTS");
    assert_eq!(
        sent(&mut server).await,
        "LISTSCRIPTS\r\n",
        "nothing overwritten"
    );
}

// ── SASL ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn oauthbearer_sends_the_rfc7628_frame_and_answers_an_error_with_0x01() {
    let (mut conn, mut server) = connected(GREETING_V1).await;
    // eyJzdGF0dXMiOiJpbnZhbGlkX3Rva2VuIn0= is {"status":"invalid_token"}
    server
        .write_all(b"\"eyJzdGF0dXMiOiJpbnZhbGlkX3Rva2VuIn0=\"\r\nNO \"Authentication failed\"\r\n")
        .await
        .unwrap();
    let err = conn
        .authenticate(&Credentials::OAuthBearer {
            username: "alice@example.test".into(),
            token: "tok".into(),
        })
        .await
        .unwrap_err();
    assert!(err.to_string().contains("Authentication failed"), "{err}");

    let wire = sent(&mut server).await;
    let lines: Vec<&str> = wire.lines().collect();
    use base64::Engine;
    let b64 = lines[0]
        .strip_prefix("AUTHENTICATE \"OAUTHBEARER\" \"")
        .and_then(|s| s.strip_suffix('"'))
        .unwrap();
    assert_eq!(
        base64::engine::general_purpose::STANDARD
            .decode(b64)
            .unwrap(),
        b"n,a=alice@example.test,\x01auth=Bearer tok\x01\x01"
    );
    assert_eq!(lines[1], "\"AQ==\"", "the %x01 reply, base64");
}

#[tokio::test]
async fn xoauth2_succeeds_with_its_initial_response() {
    let (mut conn, mut server) = connected(GREETING_V1).await;
    server.write_all(b"OK \"Logged in\"\r\n").await.unwrap();
    conn.authenticate(&Credentials::XOAuth2 {
        username: "u@example.test".into(),
        token: "t".into(),
    })
    .await
    .unwrap();
    let wire = sent(&mut server).await;
    use base64::Engine;
    let expected = base64::engine::general_purpose::STANDARD
        .encode(b"user=u@example.test\x01auth=Bearer t\x01\x01");
    assert_eq!(wire, format!("AUTHENTICATE \"XOAUTH2\" \"{expected}\"\r\n"));
}

/// A challenge that is not base64 is cancelled with `"*"` (RFC 5804 §2.1)
/// rather than answered.
#[tokio::test]
async fn a_malformed_challenge_is_cancelled() {
    let (mut conn, mut server) = connected(GREETING_V1).await;
    server
        .write_all(b"\"!!not base64!!\"\r\nNO \"Authentication cancelled\"\r\n")
        .await
        .unwrap();
    let err = conn
        .authenticate(&Credentials::Login {
            username: "bob".into(),
            password: "pw".into(),
        })
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("malformed SASL challenge"),
        "{err}"
    );
    let wire = sent(&mut server).await;
    assert_eq!(wire, "AUTHENTICATE \"LOGIN\"\r\n\"*\"\r\n");
}

// ── bounds ─────────────────────────────────────────────────────────────────

/// Many lines, each under the line limit, still cannot grow one response past
/// MAX_RESPONSE.
#[tokio::test]
async fn a_response_is_bounded_in_total() {
    let (client, mut server) = tokio::io::duplex(1 << 18);
    server.write_all(GREETING_V1.as_bytes()).await.unwrap();
    let mut conn = Connection::open(client).await.expect("greeting");

    let writer = tokio::spawn(async move {
        let line = format!("\"{}\"\r\n", "a".repeat(60_000));
        let mut written = 0usize;
        while written <= MAX_RESPONSE + 120_000 {
            if server.write_all(line.as_bytes()).await.is_err() {
                return;
            }
            written += line.len();
        }
        // Keep the stream open: a client that is not bounded waits here.
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        drop(server);
    });
    let res = tokio::time::timeout(std::time::Duration::from_secs(20), conn.list_scripts())
        .await
        .expect("bounded, not waiting");
    let err = res.expect_err("over the response limit");
    assert!(err.to_string().contains("response exceeded"), "{err}");
    writer.abort();
}
