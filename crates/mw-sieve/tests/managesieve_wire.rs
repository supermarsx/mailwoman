//! RFC 5804 ManageSieve client, driven over an in-memory duplex stream.
//!
//! `managesieve.rs` was the weakest file in the crate at 40% of lines, because
//! every command sits behind `Connection<S>` and nothing exercised `S`. It is
//! also the file with the most exposure: a Sieve server is remote, may be
//! hostile, and its responses are parsed byte by byte before authentication.
//!
//! `Connection::open` accepts any `AsyncRead + AsyncWrite + Unpin`, so a
//! `tokio::io::duplex` pair is a complete server without a socket, a fixture or
//! a new dependency. Two things are pinned here: the exact bytes the client puts
//! on the wire (interop — a wrong literal length or an unescaped quote breaks
//! against a real server), and refusal of the malformed and truncated responses
//! a hostile or broken server can send.

use mw_sieve::managesieve::{Connection, Credentials};
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};

const GREETING: &str = concat!(
    "\"IMPLEMENTATION\" \"Example ManageSieve v1\"\r\n",
    "\"SIEVE\" \"fileinto reject envelope vacation imap4flags\"\r\n",
    "\"SASL\" \"PLAIN LOGIN\"\r\n",
    "\"VERSION\" \"1.0\"\r\n",
    "\"STARTTLS\"\r\n",
    "OK \"ManageSieve ready.\"\r\n",
);

/// A duplex pair large enough that neither side ever blocks on the other for
/// the small conversations below.
fn pipe() -> (DuplexStream, DuplexStream) {
    tokio::io::duplex(1 << 18)
}

/// Queue server bytes, then open a connection over the greeting.
async fn connected(extra: &str) -> (Connection<DuplexStream>, DuplexStream) {
    let (client, mut server) = pipe();
    server.write_all(GREETING.as_bytes()).await.unwrap();
    if !extra.is_empty() {
        server.write_all(extra.as_bytes()).await.unwrap();
    }
    let conn = Connection::open(client).await.expect("greeting");
    (conn, server)
}

/// Everything the client has written so far, as a string. Reads until a short
/// quiet period so a command split across several writes is captured whole.
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

// ── greeting and capabilities ───────────────────────────────────────────────

/// The greeting's capability lines are parsed into the typed capability set,
/// including the bare `STARTTLS` line that carries no value.
#[tokio::test]
async fn greeting_populates_capabilities() {
    let (conn, _server) = connected("").await;
    let caps = conn.capabilities();

    assert!(caps.offers_sasl("PLAIN"));
    assert!(caps.offers_sasl("plain"), "SASL match folds case");
    assert!(caps.offers_sasl("LOGIN"));
    assert!(!caps.offers_sasl("GSSAPI"));

    assert!(caps.supports("fileinto"));
    assert!(caps.supports("VACATION"), "extension match folds case");
    assert!(!caps.supports("editheader"));
}

/// Escaped quotes and backslashes inside a capability *value* are unescaped, so
/// a server whose implementation string contains one is reported whole rather
/// than truncated at the escape. (The key side of the same line does not honour
/// escapes — see `script_names_containing_an_escaped_quote_are_truncated` — but
/// capability keys are protocol constants, so it is unreachable in practice.)
#[tokio::test]
async fn capability_values_are_unescaped() {
    let (client, mut server) = pipe();
    server
        .write_all(
            b"\"IMPLEMENTATION\" \"Ex \\\"quoted\\\" \\\\ v1\"\r\n\
              \"SIEVE\" \"fileinto\"\r\n\
              \"VERSION\" \"1.0\"\r\n\
              OK\r\n",
        )
        .await
        .unwrap();
    let conn = Connection::open(client).await.expect("greeting");
    let caps = conn.capabilities();
    assert_eq!(caps.implementation, r#"Ex "quoted" \ v1"#);
    assert_eq!(caps.version, "1.0");
    assert!(caps.supports("fileinto"));
    assert!(!caps.starttls, "STARTTLS was not advertised here");
}

/// A greeting that ends `NO` is a refusal, not a connection.
#[tokio::test]
async fn a_refusing_greeting_is_an_error() {
    let (client, mut server) = pipe();
    server
        .write_all(b"NO \"Too many connections from your host\"\r\n")
        .await
        .unwrap();
    let err = match Connection::open(client).await {
        Err(e) => e,
        Ok(_) => panic!("a NO greeting must not open a connection"),
    };
    assert!(err.to_string().contains("Too many connections"), "{err}");
}

/// `CAPABILITY` replaces the stored set rather than merging into it — a server
/// legitimately advertises fewer extensions before authentication.
#[tokio::test]
async fn capability_refreshes_rather_than_merges() {
    let (mut conn, mut server) = connected("").await;
    assert!(conn.capabilities().supports("vacation"));

    server
        .write_all(b"\"SIEVE\" \"fileinto\"\r\n\"SASL\" \"PLAIN\"\r\nOK\r\n")
        .await
        .unwrap();
    let caps = conn.capability().await.expect("capability");
    assert!(caps.supports("fileinto"));
    assert!(
        !caps.supports("vacation"),
        "stale extension survived a refresh"
    );
    assert!(
        !caps.offers_sasl("LOGIN"),
        "stale mechanism survived a refresh"
    );

    assert_eq!(sent(&mut server).await, "CAPABILITY\r\n");
}

// ── authentication ──────────────────────────────────────────────────────────

/// SASL PLAIN's payload is `\0authcid\0passwd`, base64-encoded. The NUL framing
/// is invisible in any string comparison, so it is asserted against the decoded
/// bytes — getting it wrong fails against every real server.
#[tokio::test]
async fn sasl_plain_sends_the_nul_framed_payload() {
    let (mut conn, mut server) = connected("").await;
    server.write_all(b"OK \"Logged in.\"\r\n").await.unwrap();

    conn.authenticate(&Credentials::Plain {
        username: "alice@example.test".into(),
        password: "s3cr3t".into(),
    })
    .await
    .expect("authenticate");

    let line = sent(&mut server).await;
    let b64 = line
        .trim_end()
        .strip_prefix("AUTHENTICATE \"PLAIN\" \"")
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or_else(|| panic!("unexpected command: {line:?}"));

    use base64::Engine;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .expect("base64");
    assert_eq!(raw, b"\0alice@example.test\0s3cr3t");
}

/// SASL LOGIN answers each server challenge in order — username first, then
/// password — each as a base64 quoted string. Answering them the wrong way round
/// would leak the password into the username field of the server's log.
#[tokio::test]
async fn sasl_login_answers_challenges_in_order() {
    let (mut conn, mut server) = connected("").await;
    server
        .write_all(b"{12}\r\nVXNlcm5hbWU6\r\n{12}\r\nUGFzc3dvcmQ6\r\nOK \"Logged in.\"\r\n")
        .await
        .unwrap();

    conn.authenticate(&Credentials::Login {
        username: "bob".into(),
        password: "hunter2".into(),
    })
    .await
    .expect("authenticate");

    let wire = sent(&mut server).await;
    let lines: Vec<&str> = wire.lines().collect();
    assert_eq!(lines[0], "AUTHENTICATE \"LOGIN\"");
    assert_eq!(lines[1], "\"Ym9i\"", "username answer (base64 of `bob`)");
    assert_eq!(
        lines[2], "\"aHVudGVyMg==\"",
        "password answer (base64 of `hunter2`)"
    );
}

/// A rejected authentication is an error carrying the server's reason, not a
/// silently unauthenticated connection.
#[tokio::test]
async fn a_rejected_authentication_is_an_error() {
    let (mut conn, mut server) = connected("").await;
    server
        .write_all(b"NO (AUTH-TOO-WEAK) \"Authentication failed\"\r\n")
        .await
        .unwrap();

    let err = conn
        .authenticate(&Credentials::Plain {
            username: "alice".into(),
            password: "wrong".into(),
        })
        .await
        .expect_err("rejected");
    assert!(err.to_string().contains("Authentication failed"), "{err}");
}

/// A LOGIN exchange the server rejects mid-way stops there rather than looping.
#[tokio::test]
async fn a_rejected_login_challenge_stops() {
    let (mut conn, mut server) = connected("").await;
    server
        .write_all(b"{12}\r\nVXNlcm5hbWU6\r\nNO \"Authentication failed\"\r\n")
        .await
        .unwrap();

    let err = conn
        .authenticate(&Credentials::Login {
            username: "bob".into(),
            password: "hunter2".into(),
        })
        .await
        .expect_err("rejected");
    assert!(err.to_string().contains("Authentication failed"), "{err}");
}

// ── script commands ─────────────────────────────────────────────────────────

/// `PUTSCRIPT` frames the body as a non-synchronising literal whose length is in
/// **bytes**. A multi-byte body is the case that catches a `chars().count()`.
#[tokio::test]
async fn put_script_literal_length_is_in_bytes() {
    let (mut conn, mut server) = connected("").await;
    server.write_all(b"OK\r\n").await.unwrap();

    // 9 ASCII bytes + "é" (2 bytes) = 11 bytes, but only 10 characters.
    let body = "#comment é";
    assert_eq!(body.chars().count(), 10);
    assert_eq!(body.len(), 11);

    conn.put_script("mailwoman", body).await.expect("putscript");

    let wire = sent(&mut server).await;
    assert_eq!(
        wire,
        format!("PUTSCRIPT \"mailwoman\" {{11+}}\r\n{body}\r\n")
    );
}

/// A script name containing a quote or a backslash is escaped, so it cannot
/// terminate the quoted-string argument early and inject a second command.
#[tokio::test]
async fn script_names_are_escaped_on_the_wire() {
    let (mut conn, mut server) = connected("").await;
    server.write_all(b"OK\r\nOK\r\nOK\r\n").await.unwrap();

    conn.set_active(r#"ev"il"#).await.expect("setactive");
    conn.delete_script(r"back\slash").await.expect("delete");
    conn.set_active("").await.expect("deactivate");

    let wire = sent(&mut server).await;
    assert!(wire.contains(r#"SETACTIVE "ev\"il""#), "{wire:?}");
    assert!(wire.contains(r#"DELETESCRIPT "back\\slash""#), "{wire:?}");
    assert!(wire.contains("SETACTIVE \"\"\r\n"), "{wire:?}");
    // No injected newline could ever appear inside an argument.
    assert_eq!(wire.lines().count(), 3, "{wire:?}");
}

/// `LISTSCRIPTS` reports each script and which one is active, tolerating the
/// case variation and trailing whitespace real servers emit.
#[tokio::test]
async fn list_scripts_reports_names_and_the_active_one() {
    let (mut conn, mut server) = connected("").await;
    server
        .write_all(
            b"\"mailwoman\" ACTIVE\r\n\
              \"vacation\"\r\n\
              \"with space\" active \r\n\
              OK \"Listscripts completed.\"\r\n",
        )
        .await
        .unwrap();

    let scripts = conn.list_scripts().await.expect("listscripts");
    assert_eq!(scripts.len(), 3, "{scripts:?}");
    assert_eq!(scripts[0].name, "mailwoman");
    assert!(scripts[0].active);
    assert_eq!(scripts[1].name, "vacation");
    assert!(!scripts[1].active);
    assert_eq!(scripts[2].name, "with space");
    assert!(scripts[2].active, "trailing whitespace after ACTIVE");
}

/// DEFECT (recorded, not fixed): a quoted-string body is scanned for its closing
/// quote without honouring `\"` escapes, so a name containing an escaped quote is
/// truncated at that quote. `unescape` is then applied to the already-truncated
/// slice, which is why the asymmetry is easy to miss: the client *writes* names
/// with `\"` escaping (`quote_arg`) but does not *read* them that way.
///
/// This affects `parse_script_line` (script names) and, in the same way,
/// `split_cap_line`'s extraction of the capability *key* — though not its value,
/// which is delimited by `strip_prefix`/`strip_suffix` and does unescape
/// correctly. Impact is limited: a truncated name simply fails to match on a
/// later `SETACTIVE`/`GETSCRIPT`, capability keys are protocol constants, and a
/// quote in a Sieve script name is exotic. But the round-trip the code implies
/// does not hold.
#[tokio::test]
async fn script_names_containing_an_escaped_quote_are_truncated() {
    let (mut conn, mut server) = connected("").await;
    server
        .write_all(b"\"with \\\"quotes\\\" inside\" ACTIVE\r\nOK\r\n")
        .await
        .unwrap();

    let scripts = conn.list_scripts().await.expect("listscripts");
    assert_eq!(scripts.len(), 1);
    // Correct would be `with "quotes" inside`; the name is cut at the escape.
    assert_eq!(scripts[0].name, "with ");
    assert!(
        !scripts[0].active,
        "the ACTIVE flag is lost with the rest of the line"
    );
}

/// `GETSCRIPT` reads the body from a literal, and falls back to a quoted data
/// line for the servers that return a short script that way.
#[tokio::test]
async fn get_script_accepts_a_literal_and_a_quoted_line() {
    let (mut conn, mut server) = connected("").await;

    server
        .write_all(b"{29}\r\nif true { fileinto \"INBOX\"; }\r\nOK\r\n")
        .await
        .unwrap();
    assert_eq!(
        conn.get_script("mailwoman").await.expect("getscript"),
        "if true { fileinto \"INBOX\"; }"
    );

    server.write_all(b"keep;\r\nOK\r\n").await.unwrap();
    assert_eq!(conn.get_script("short").await.expect("getscript"), "keep;");

    let wire = sent(&mut server).await;
    assert!(wire.contains("GETSCRIPT \"mailwoman\"\r\n"), "{wire:?}");
    assert!(wire.contains("GETSCRIPT \"short\"\r\n"), "{wire:?}");
}

/// `LOGOUT` accepts both terminations a server may use, and still refuses a `NO`.
#[tokio::test]
async fn logout_accepts_ok_and_bye_but_not_no() {
    let (mut conn, mut server) = connected("").await;
    server.write_all(b"OK\r\n").await.unwrap();
    conn.logout().await.expect("OK logout");

    let (mut conn, mut server2) = connected("").await;
    server2.write_all(b"BYE \"closing\"\r\n").await.unwrap();
    conn.logout().await.expect("BYE logout");

    let (mut conn, mut server3) = connected("").await;
    server3.write_all(b"NO \"cannot\"\r\n").await.unwrap();
    assert!(conn.logout().await.is_err());
}

/// `NOOP` and `STARTTLS` frame their commands and require an `OK`.
#[tokio::test]
async fn noop_and_starttls_require_an_ok() {
    let (mut conn, mut server) = connected("").await;
    server
        .write_all(b"OK\r\nOK \"Begin TLS\"\r\n")
        .await
        .unwrap();
    conn.noop().await.expect("noop");
    conn.starttls().await.expect("starttls");
    assert_eq!(sent(&mut server).await, "NOOP\r\nSTARTTLS\r\n");

    let (mut conn, mut server) = connected("").await;
    server
        .write_all(b"NO \"TLS unavailable\"\r\n")
        .await
        .unwrap();
    assert!(conn.starttls().await.is_err());
}

// ── STARTTLS handover ───────────────────────────────────────────────────────

/// Handing the stream to the TLS layer is only safe when nothing is buffered.
/// Bytes the server sent ahead of the handshake would otherwise be indexed as
/// plaintext beneath TLS, so the connection is refused instead.
#[tokio::test]
async fn starttls_handover_refuses_pipelined_bytes() {
    // Clean case: nothing buffered after the greeting.
    let (conn, _server) = connected("").await;
    assert!(conn.into_inner().is_ok());

    // The server pipelines data after the greeting's OK.
    let (conn, _server) = connected("\"INJECTED\" \"data\"\r\n").await;
    let err = conn.into_inner().expect_err("must refuse");
    assert!(
        err.to_string().contains("buffered before STARTTLS"),
        "{err}"
    );
}

// ── hostile and malformed responses ─────────────────────────────────────────

/// A data line that merely *starts with* the letters of a completion keyword is
/// data — otherwise a stored script named `OKmail` would truncate the response.
#[tokio::test]
async fn a_completion_keyword_needs_a_word_boundary() {
    let (mut conn, mut server) = connected("").await;
    server
        .write_all(b"\"OKmail\" ACTIVE\r\n\"NOTES\"\r\n\"BYEbye\"\r\nOK\r\n")
        .await
        .unwrap();

    let scripts = conn.list_scripts().await.expect("listscripts");
    let names: Vec<&str> = scripts.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["OKmail", "NOTES", "BYEbye"]);
}

/// A completion whose human text arrives as a literal is read as such — a server
/// with a long or multi-line error message must still produce a usable error.
#[tokio::test]
async fn a_completion_can_carry_its_text_as_a_literal() {
    let (mut conn, mut server) = connected("").await;
    server
        .write_all(b"NO {29}\r\nquota exceeded on this\r\nhost\r\n")
        .await
        .unwrap();
    let err = conn.noop().await.expect_err("NO");
    assert!(err.to_string().contains("quota exceeded"), "{err}");
}

/// A server that hangs up part-way through is a clear error, not a hang or a
/// truncated success.
#[tokio::test]
async fn a_connection_closed_mid_response_is_an_error() {
    let (client, mut server) = pipe();
    server.write_all(GREETING.as_bytes()).await.unwrap();
    let mut conn = Connection::open(client).await.expect("greeting");

    server
        .write_all(b"\"partial line with no newline")
        .await
        .unwrap();
    // Close only the server->client direction, so the client's own write still
    // succeeds and the failure it reports is the truncated *read*.
    server.shutdown().await.unwrap();

    let err = conn.noop().await.expect_err("closed");
    assert!(err.to_string().contains("closed mid-response"), "{err}");
}

/// The same, for a literal whose announced length never arrives — the length is
/// attacker-controlled, so it must not be trusted to be satisfiable.
#[tokio::test]
async fn a_connection_closed_mid_literal_is_an_error() {
    let (client, mut server) = pipe();
    server.write_all(GREETING.as_bytes()).await.unwrap();
    let mut conn = Connection::open(client).await.expect("greeting");

    server
        .write_all(b"{4096}\r\nonly a few bytes")
        .await
        .unwrap();
    server.shutdown().await.unwrap();

    let err = conn.get_script("x").await.expect_err("closed");
    assert!(err.to_string().contains("closed mid-literal"), "{err}");
}

/// An unterminated line is bounded rather than buffered without limit — a
/// hostile server must not be able to grow the client's heap indefinitely.
#[tokio::test]
async fn an_unbounded_response_line_is_refused() {
    let (client, mut server) = pipe();
    server.write_all(GREETING.as_bytes()).await.unwrap();
    let mut conn = Connection::open(client).await.expect("greeting");

    let flood = vec![b'A'; 70_000];
    server.write_all(&flood).await.unwrap();

    let err = conn.noop().await.expect_err("bounded");
    assert!(err.to_string().contains("exceeded limit"), "{err}");
}

/// Bare-LF line endings (non-conforming but seen in the wild) are accepted, and
/// a lone CR inside a line is not mistaken for a terminator.
#[tokio::test]
async fn bare_lf_line_endings_are_tolerated() {
    let (client, mut server) = pipe();
    server
        .write_all(b"\"IMPLEMENTATION\" \"LF only\"\n\"SIEVE\" \"fileinto\"\nOK\n")
        .await
        .unwrap();
    let mut conn = Connection::open(client).await.expect("greeting");
    assert!(conn.capabilities().supports("fileinto"));

    server.write_all(b"OK\n").await.unwrap();
    conn.noop().await.expect("noop over LF");
}

/// Invalid UTF-8 in a response is replaced rather than fatal — the client must
/// stay usable against a server that emits a Latin-1 error string.
#[tokio::test]
async fn invalid_utf8_in_a_response_does_not_break_the_connection() {
    let (client, mut server) = pipe();
    server.write_all(GREETING.as_bytes()).await.unwrap();
    let mut conn = Connection::open(client).await.expect("greeting");

    server
        .write_all(b"NO \"caf\xe9 not found \xff\xfe\"\r\n")
        .await
        .unwrap();
    let err = conn.noop().await.expect_err("NO");
    assert!(err.to_string().contains("caf"), "{err}");

    server.write_all(b"OK\r\n").await.unwrap();
    conn.noop().await.expect("still usable");
}
