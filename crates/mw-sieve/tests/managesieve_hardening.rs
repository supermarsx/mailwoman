//! ManageSieve client defences against a hostile server and hostile names
//! (t32 f18). These use only the API that existed before f18, so each one was
//! run against the old client first and failed there.
//!
//! - a script name holding CR/LF must not reach the wire, or it ends the command
//!   line and the rest runs as a second command;
//! - a literal length is chosen by the server and must be bounded before the
//!   client buffers it;
//! - a literal may be followed by more of the same line (`{n}` name, then
//!   ` ACTIVE`);
//! - `Debug` of the credentials must not print the password.

use mw_sieve::managesieve::{Connection, Credentials};
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};

const GREETING: &str = concat!(
    "\"IMPLEMENTATION\" \"Example ManageSieve v1\"\r\n",
    "\"SIEVE\" \"fileinto vacation\"\r\n",
    "\"SASL\" \"PLAIN\"\r\n",
    "\"VERSION\" \"1.0\"\r\n",
    "OK \"ready\"\r\n",
);

async fn connected() -> (Connection<DuplexStream>, DuplexStream) {
    let (client, mut server) = tokio::io::duplex(1 << 18);
    server.write_all(GREETING.as_bytes()).await.unwrap();
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

/// A name with an embedded CRLF would otherwise put a `DELETESCRIPT` on the wire
/// as its own command line (its argument sent as a literal, so no quote needs
/// to survive escaping). The client refuses the name and sends nothing.
#[tokio::test]
async fn a_name_with_crlf_never_reaches_the_wire() {
    let (mut conn, mut server) = connected().await;
    // Answers for every command that might be sent, so a client that does send
    // does not hang and the assertion below sees what it wrote.
    server.write_all(b"OK\r\nOK\r\nOK\r\nOK\r\n").await.unwrap();

    let evil = "x\r\nDELETESCRIPT {7+}\r\nvictim";
    let refused = [
        conn.set_active(evil).await.is_err(),
        conn.get_script(evil).await.is_err(),
        conn.delete_script(evil).await.is_err(),
        conn.put_script(evil, "keep;").await.is_err(),
    ];

    let wire = sent(&mut server).await;
    assert!(
        !wire.contains("\r\nDELETESCRIPT"),
        "an injected command reached the server: {wire:?}"
    );
    assert_eq!(
        refused, [true; 4],
        "set_active, get_script, delete_script, put_script"
    );
    assert_eq!(wire, "", "nothing should be sent for a refused name");
}

/// A literal announced larger than the client will hold is refused at once,
/// before any of it is read; the length is the server's to choose.
#[tokio::test]
async fn an_oversized_literal_is_refused_before_reading_it() {
    let (mut conn, mut server) = connected().await;
    // 4 GiB announced; only a few bytes follow and the stream stays open, so a
    // client that tries to read the literal waits (and the timeout fails it).
    server.write_all(b"{4294967296}\r\nshort").await.unwrap();
    let res = tokio::time::timeout(std::time::Duration::from_secs(5), conn.get_script("x"))
        .await
        .expect("the client must refuse the length, not wait for 4 GiB");
    let err = res.expect_err("oversized literal");
    assert!(err.to_string().contains("limit"), "{err}");
}

/// A script name sent as a literal keeps the ` ACTIVE` that follows it on the
/// same logical line.
#[tokio::test]
async fn a_literal_script_name_keeps_its_active_flag() {
    let (mut conn, mut server) = connected().await;
    server
        .write_all(b"{9}\r\nmy script ACTIVE\r\n\"other\"\r\nOK\r\n")
        .await
        .unwrap();
    let scripts = conn.list_scripts().await.expect("listscripts");
    assert_eq!(scripts.len(), 2, "{scripts:?}");
    assert_eq!(scripts[0].name, "my script");
    assert!(scripts[0].active, "{scripts:?}");
    assert_eq!(scripts[1].name, "other");
    assert!(!scripts[1].active);
}

/// Credentials end up in error contexts and logs through `{:?}`; the password
/// must not.
#[test]
fn credentials_debug_does_not_print_the_password() {
    let plain = Credentials::Plain {
        username: "alice@example.test".into(),
        password: "correct horse battery".into(),
    };
    let login = Credentials::Login {
        username: "bob".into(),
        password: "hunter2-secret".into(),
    };
    let (p, l) = (format!("{plain:?}"), format!("{login:?}"));
    assert!(p.contains("alice@example.test"), "{p}");
    assert!(!p.contains("correct horse battery"), "{p}");
    assert!(!l.contains("hunter2-secret"), "{l}");
}
