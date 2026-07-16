//! t12-e-e2e-backend — SMTP extensions live-E2E (audit #9): DSN / SMTPUTF8 /
//! CHUNKING(BDAT) / REQUIRETLS.
//!
//! Drives mw-smtp's `Submitter::submit_with` (the new `SubmitOptions` path, lib.rs)
//! over a REAL TCP socket against an in-process ESMTP conformance server (a pure-Rust
//! sidecar in the same spirit as the SPAMC→HTTP relay — stock tokio, no C/openssl).
//! The server speaks the protocol, advertises the extensions, and CAPTURES the wire
//! so the assertions are made on what a real server actually received:
//!   * SMTPUTF8 auto-negotiated for a non-ASCII envelope recipient (RFC 6531),
//!   * DSN RET / ENVID on MAIL and NOTIFY / ORCPT on RCPT (RFC 3461),
//!   * BDAT/CHUNKING length-framed body, no dot-stuffing (RFC 3030),
//!   * REQUIRETLS fails CLOSED when the server does not advertise it (RFC 8689).
//!
//! These run in the DEFAULT `cargo test` gate (the sidecar is in-process): the wire
//! is real, so a regression in the submission emission is caught deterministically.

use std::sync::{Arc, Mutex};

use base64::Engine as _;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use mw_smtp::{
    Credentials, Dsn, DsnNotify, DsnRet, Outgoing, Security, SubmitConfig, SubmitOptions, Submitter,
};

const B64: base64::engine::general_purpose::GeneralPurpose =
    base64::engine::general_purpose::STANDARD;

/// Minimal CRLF line + exact-byte reader over the accepted socket.
struct Wire {
    sock: TcpStream,
    buf: Vec<u8>,
}
impl Wire {
    fn new(sock: TcpStream) -> Self {
        Self {
            sock,
            buf: Vec::new(),
        }
    }
    async fn send(&mut self, bytes: &[u8]) {
        self.sock.write_all(bytes).await.unwrap();
    }
    /// Read one CRLF-terminated line (without the CRLF).
    async fn line(&mut self) -> String {
        loop {
            if let Some(pos) = self.buf.windows(2).position(|w| w == b"\r\n") {
                let line = self.buf.drain(..pos).collect::<Vec<_>>();
                self.buf.drain(..2); // drop CRLF
                return String::from_utf8_lossy(&line).into_owned();
            }
            let mut tmp = [0u8; 1024];
            let n = self.sock.read(&mut tmp).await.unwrap();
            if n == 0 {
                return String::from_utf8_lossy(&std::mem::take(&mut self.buf)).into_owned();
            }
            self.buf.extend_from_slice(&tmp[..n]);
        }
    }
    /// Read exactly `n` raw bytes (BDAT body framing).
    async fn exact(&mut self, n: usize) -> Vec<u8> {
        while self.buf.len() < n {
            let mut tmp = [0u8; 4096];
            let r = self.sock.read(&mut tmp).await.unwrap();
            if r == 0 {
                break;
            }
            self.buf.extend_from_slice(&tmp[..r]);
        }
        self.buf.drain(..n.min(self.buf.len())).collect()
    }
}

/// The full extended-submission ESMTP server. Returns the captured MAIL/RCPT/BDAT
/// lines + body once the client disconnects.
async fn run_full_server(listener: TcpListener, captured: Arc<Mutex<Vec<String>>>) {
    let (sock, _) = listener.accept().await.unwrap();
    let mut w = Wire::new(sock);
    w.send(b"220 esmtp.mailwoman.test ESMTP\r\n").await;

    let ehlo = w.line().await;
    assert!(ehlo.starts_with("EHLO"), "expected EHLO, got {ehlo:?}");
    w.send(
        b"250-esmtp.mailwoman.test\r\n\
          250-SIZE 104857600\r\n\
          250-8BITMIME\r\n\
          250-SMTPUTF8\r\n\
          250-DSN\r\n\
          250-CHUNKING\r\n\
          250 AUTH SCRAM-SHA-256\r\n",
    )
    .await;

    // SCRAM: echo the client nonce in a well-formed server-first; accept the proof
    // (the RFC 7677 math is pinned by the mw-smtp vector test).
    let auth = w.line().await;
    let ir = auth
        .strip_prefix("AUTH SCRAM-SHA-256 ")
        .expect("AUTH SCRAM line");
    let client_first = String::from_utf8(B64.decode(ir).unwrap()).unwrap();
    let client_nonce = client_first
        .rsplit(',')
        .next()
        .and_then(|f| f.strip_prefix("r="))
        .expect("client nonce")
        .to_string();
    let server_first = format!("r={client_nonce}SRVNONCE,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096");
    w.send(format!("334 {}\r\n", B64.encode(&server_first)).as_bytes())
        .await;
    let _client_final = w.line().await;
    w.send(b"235 2.7.0 Authentication successful\r\n").await;

    // MAIL FROM (capture), RCPT TO (capture), BDAT (capture + body).
    let mail = w.line().await;
    captured.lock().unwrap().push(mail);
    w.send(b"250 2.1.0 OK\r\n").await;

    let rcpt = w.line().await;
    captured.lock().unwrap().push(rcpt);
    w.send(b"250 2.1.5 OK\r\n").await;

    let bdat = w.line().await;
    let n: usize = bdat
        .strip_prefix("BDAT ")
        .and_then(|r| r.strip_suffix(" LAST"))
        .and_then(|r| r.parse().ok())
        .unwrap_or_else(|| panic!("expected BDAT <n> LAST, got {bdat:?}"));
    captured.lock().unwrap().push(bdat);
    let body = w.exact(n).await;
    captured.lock().unwrap().push(format!(
        "BODYLEN={} DOTLINE={}",
        body.len(),
        body.windows(3).any(|x| x == b"\r\n.")
    ));
    w.send(b"250 2.0.0 OK: queued\r\n").await;

    let quit = w.line().await;
    assert_eq!(quit, "QUIT");
    w.send(b"221 2.0.0 Bye\r\n").await;
}

#[tokio::test]
async fn smtp_dsn_smtputf8_bdat_wire_live() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let captured = Arc::new(Mutex::new(Vec::<String>::new()));
    let server = tokio::spawn(run_full_server(listener, captured.clone()));

    let sub = Submitter::new(SubmitConfig {
        host: addr.ip().to_string(),
        port: addr.port(),
        security: Security::Plaintext,
        credentials: Credentials::Scram {
            user: "user".into(),
            pass: "pencil".into(),
        },
        ehlo_name: "client.test".into(),
    });

    let result = sub
        .submit_with(
            Outgoing {
                mail_from: "sender@example.com".into(),
                // Non-ASCII recipient ⇒ SMTPUTF8 must be auto-negotiated.
                rcpt_to: vec!["møt@example.com".into()],
                raw: b"Subject: hi\r\n\r\nHello.\r\n.\r\nafter\r\n".to_vec(),
            },
            SubmitOptions {
                dsn: Some(Dsn {
                    ret: Some(DsnRet::Hdrs),
                    envid: Some("abc123".into()),
                    notify: vec![DsnNotify::Success, DsnNotify::Failure],
                    orcpt: true,
                }),
                require_tls: false,
                use_chunking: true,
            },
        )
        .await
        .expect("extended submission");

    server.await.unwrap();
    assert_eq!(result.accepted, vec!["møt@example.com".to_string()]);
    assert!(result.rejected.is_empty());

    let cap = captured.lock().unwrap();
    let mail = &cap[0];
    assert!(mail.starts_with("MAIL FROM:<sender@example.com>"), "{mail}");
    assert!(
        mail.contains(" SMTPUTF8"),
        "SMTPUTF8 auto-negotiated: {mail}"
    );
    assert!(mail.contains(" RET=HDRS"), "DSN RET: {mail}");
    assert!(mail.contains(" ENVID=abc123"), "DSN ENVID: {mail}");

    let rcpt = &cap[1];
    assert!(rcpt.starts_with("RCPT TO:<møt@example.com>"), "{rcpt}");
    assert!(
        rcpt.contains(" NOTIFY=SUCCESS,FAILURE"),
        "DSN NOTIFY: {rcpt}"
    );
    assert!(rcpt.contains(" ORCPT=rfc822;"), "DSN ORCPT: {rcpt}");

    let bdat = &cap[2];
    assert!(bdat.ends_with(" LAST"), "single-chunk BDAT: {bdat}");
    // The lone "." line is preserved verbatim under BDAT (no dot-stuffing).
    assert!(
        cap[3].contains("DOTLINE=true"),
        "BDAT preserves the raw body: {}",
        cap[3]
    );
}

/// REQUIRETLS fails CLOSED: a server that does NOT advertise REQUIRETLS causes
/// `submit_with(require_tls=true)` to error BEFORE any MAIL FROM — the guarantee is
/// never silently dropped.
#[tokio::test]
async fn smtp_requiretls_fails_closed_when_unadvertised() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let saw_mail = Arc::new(Mutex::new(false));
    let saw = saw_mail.clone();

    let server = tokio::spawn(async move {
        let (sock, _) = listener.accept().await.unwrap();
        let mut w = Wire::new(sock);
        w.send(b"220 esmtp ESMTP\r\n").await;
        let _ehlo = w.line().await;
        // Deliberately NO REQUIRETLS in the advertised set.
        w.send(b"250-esmtp\r\n250 SIZE 104857600\r\n").await;
        // If the client (incorrectly) proceeded, it would send MAIL FROM here.
        let next = w.line().await;
        if next.starts_with("MAIL FROM") {
            *saw.lock().unwrap() = true;
        }
    });

    let sub = Submitter::new(SubmitConfig {
        host: addr.ip().to_string(),
        port: addr.port(),
        security: Security::Plaintext,
        credentials: Credentials::None,
        ehlo_name: "client.test".into(),
    });

    let err = sub
        .submit_with(
            Outgoing {
                mail_from: "sender@example.com".into(),
                rcpt_to: vec!["rcpt@example.com".into()],
                raw: b"Subject: x\r\n\r\nbody\r\n".to_vec(),
            },
            SubmitOptions {
                require_tls: true,
                ..SubmitOptions::default()
            },
        )
        .await
        .expect_err("REQUIRETLS must fail closed when unadvertised");

    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("requiretls"),
        "the error must name REQUIRETLS, got: {err}"
    );
    server.abort();
    let _ = server.await;
    assert!(
        !*saw_mail.lock().unwrap(),
        "the client must NOT send MAIL FROM when REQUIRETLS is unmet"
    );
}
