//! Line-based SMTP connection state machine over any async byte stream.
//!
//! [`Connection`] is generic over `S: AsyncRead + AsyncWrite` so the exact same
//! command logic drives a cleartext [`tokio::net::TcpStream`] (the mock unit
//! tests, and pre-`STARTTLS` framing on 587), an implicit-TLS stream on 465, and
//! a post-upgrade TLS stream — the transport is swapped underneath the protocol,
//! never duplicated. All parsing here is defensive (bounded line length, no
//! panics) since the peer is untrusted network input.

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::sasl;
use crate::{Credentials, SmtpError};

/// Hard cap on a single CRLF-terminated reply line, to bound memory against a
/// hostile or broken server that never sends a newline.
const MAX_LINE: usize = 65_536;

/// A parsed SMTP reply: the numeric status plus the text of each line.
#[derive(Debug, Clone)]
pub(crate) struct Reply {
    pub code: u16,
    pub lines: Vec<String>,
}

impl Reply {
    /// A 2xx reply means the command succeeded.
    fn is_positive(&self) -> bool {
        (200..300).contains(&self.code)
    }

    /// Flatten the reply text for use in an error message.
    fn text(&self) -> String {
        self.lines.join(" ")
    }
}

/// EHLO-advertised capabilities relevant to V1 submission (plan §3 e4).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Capabilities {
    /// `SIZE` advertised; the value is the server's max message size (`0` when
    /// advertised without a limit). Presence drives the `SIZE=` MAIL parameter.
    pub size: Option<u64>,
    /// `8BITMIME` — allows an un-encoded 8-bit body (`BODY=8BITMIME`).
    pub eightbitmime: bool,
    /// `STARTTLS` offered (only meaningful on the cleartext 587 probe).
    pub starttls: bool,
    /// `PIPELINING` offered (parsed for completeness; V1 does not pipeline).
    pub pipelining: bool,
    /// Offered SASL mechanisms, upper-cased (`PLAIN`, `LOGIN`, `XOAUTH2`, …).
    pub auth: Vec<String>,
}

impl Capabilities {
    /// Parse the capability lines of an EHLO reply — i.e. every line *after*
    /// the leading greeting line. Unknown lines are ignored.
    pub(crate) fn parse(cap_lines: &[String]) -> Self {
        let mut caps = Capabilities::default();
        for line in cap_lines {
            // Split on space *or* `=` so both `AUTH PLAIN` and the legacy
            // `AUTH=PLAIN` forms yield the mechanism tokens.
            let upper = line.to_ascii_uppercase();
            let mut parts = upper.split([' ', '=']).filter(|s| !s.is_empty());
            match parts.next() {
                Some("SIZE") => {
                    caps.size = Some(parts.next().and_then(|n| n.parse().ok()).unwrap_or(0));
                }
                Some("8BITMIME") => caps.eightbitmime = true,
                Some("STARTTLS") => caps.starttls = true,
                Some("PIPELINING") => caps.pipelining = true,
                Some("AUTH") => caps.auth = parts.map(String::from).collect(),
                _ => {}
            }
        }
        caps
    }

    /// Whether a given mechanism name (case-insensitive) was advertised.
    fn offers(&self, mech: &str) -> bool {
        self.auth.iter().any(|m| m.eq_ignore_ascii_case(mech))
    }
}

/// Outcome of a single `RCPT TO` — recipients are independent, so a rejection
/// is recorded rather than aborting the whole submission (plan §3 e4).
pub(crate) enum RcptOutcome {
    Accepted,
    Rejected { reason: String },
}

/// An SMTP connection with a small read buffer for line framing.
pub(crate) struct Connection<S> {
    stream: S,
    rbuf: Vec<u8>,
}

impl<S> Connection<S> {
    pub(crate) fn new(stream: S) -> Self {
        Self {
            stream,
            rbuf: Vec::with_capacity(1024),
        }
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> Connection<S> {
    /// Read one CRLF-terminated line, returned without its trailing CR/LF.
    async fn read_line(&mut self) -> Result<String, SmtpError> {
        loop {
            if let Some(pos) = self.rbuf.iter().position(|&b| b == b'\n') {
                let mut line: Vec<u8> = self.rbuf.drain(..=pos).collect();
                line.pop(); // drop '\n'
                if line.last() == Some(&b'\r') {
                    line.pop(); // drop '\r'
                }
                return Ok(String::from_utf8_lossy(&line).into_owned());
            }
            if self.rbuf.len() > MAX_LINE {
                return Err(SmtpError::Protocol("reply line exceeded limit".into()));
            }
            let mut tmp = [0u8; 1024];
            let n = self.stream.read(&mut tmp).await?;
            if n == 0 {
                return Err(SmtpError::Protocol("connection closed mid-reply".into()));
            }
            self.rbuf.extend_from_slice(&tmp[..n]);
        }
    }

    /// Read a full (possibly multi-line) SMTP reply. Continuation lines use a
    /// `-` after the 3-digit code; the final line uses a space (or nothing).
    pub(crate) async fn read_reply(&mut self) -> Result<Reply, SmtpError> {
        let mut lines = Vec::new();
        loop {
            let line = self.read_line().await?;
            if line.len() < 3 {
                return Err(SmtpError::Protocol(format!("short reply line {line:?}")));
            }
            let code: u16 = line[..3]
                .parse()
                .map_err(|_| SmtpError::Protocol(format!("bad reply code {line:?}")))?;
            let sep = line.as_bytes().get(3).copied();
            lines.push(line.get(4..).unwrap_or("").to_string());
            if sep != Some(b'-') {
                // space or end-of-line ⇒ final line
                return Ok(Reply { code, lines });
            }
        }
    }

    async fn write_all(&mut self, bytes: &[u8]) -> Result<(), SmtpError> {
        self.stream.write_all(bytes).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// Send a command line and read its reply.
    async fn command(&mut self, line: &str) -> Result<Reply, SmtpError> {
        self.write_all(line.as_bytes()).await?;
        self.read_reply().await
    }

    /// Consume the opening `220` service-ready greeting.
    pub(crate) async fn read_greeting(&mut self) -> Result<(), SmtpError> {
        let reply = self.read_reply().await?;
        if reply.code == 220 {
            Ok(())
        } else {
            Err(SmtpError::Protocol(format!(
                "unexpected greeting {} {}",
                reply.code,
                reply.text()
            )))
        }
    }

    /// Send `EHLO` and parse the advertised capabilities.
    pub(crate) async fn ehlo(&mut self, name: &str) -> Result<Capabilities, SmtpError> {
        let reply = self.command(&format!("EHLO {name}\r\n")).await?;
        if reply.code != 250 {
            return Err(SmtpError::Protocol(format!(
                "EHLO rejected: {} {}",
                reply.code,
                reply.text()
            )));
        }
        // Line 0 is the greeting domain; capabilities follow.
        Ok(Capabilities::parse(reply.lines.get(1..).unwrap_or(&[])))
    }

    /// Issue `STARTTLS` and require the `220` that permits the handshake.
    pub(crate) async fn starttls(&mut self) -> Result<(), SmtpError> {
        let reply = self.command("STARTTLS\r\n").await?;
        if reply.code == 220 {
            Ok(())
        } else {
            Err(SmtpError::Protocol(format!(
                "STARTTLS rejected: {} {}",
                reply.code,
                reply.text()
            )))
        }
    }

    /// Perform SASL authentication per the configured credentials.
    pub(crate) async fn authenticate(
        &mut self,
        creds: &Credentials,
        caps: &Capabilities,
    ) -> Result<(), SmtpError> {
        let require = |mech: &str| -> Result<(), SmtpError> {
            if caps.auth.is_empty() || caps.offers(mech) {
                Ok(()) // absent AUTH line ⇒ try anyway (some servers omit it)
            } else {
                Err(SmtpError::Auth(format!(
                    "server does not offer {mech} (offers: {})",
                    caps.auth.join(", ")
                )))
            }
        };

        match creds {
            Credentials::None => Ok(()),
            Credentials::Plain { user, pass } => {
                require("PLAIN")?;
                let reply = self
                    .command(&format!("AUTH PLAIN {}\r\n", sasl::plain(user, pass)))
                    .await?;
                Self::expect_auth_ok(reply)
            }
            Credentials::Login { user, pass } => {
                require("LOGIN")?;
                let r = self.command("AUTH LOGIN\r\n").await?;
                if r.code != 334 {
                    return Err(SmtpError::Auth(format!(
                        "AUTH LOGIN: {} {}",
                        r.code,
                        r.text()
                    )));
                }
                let r = self.command(&format!("{}\r\n", sasl::b64(user))).await?;
                if r.code != 334 {
                    return Err(SmtpError::Auth(format!(
                        "LOGIN username step: {} {}",
                        r.code,
                        r.text()
                    )));
                }
                let r = self.command(&format!("{}\r\n", sasl::b64(pass))).await?;
                Self::expect_auth_ok(r)
            }
            Credentials::XOAuth2 { user, token } => {
                require("XOAUTH2")?;
                let r = self
                    .command(&format!("AUTH XOAUTH2 {}\r\n", sasl::xoauth2(user, token)))
                    .await?;
                match r.code {
                    235 => Ok(()),
                    // Failure path (RFC): server sends a `334 <base64 error>`
                    // challenge; the client acknowledges with an empty line and
                    // the server then returns the real failure code.
                    334 => {
                        let r2 = self.command("\r\n").await?;
                        Err(SmtpError::Auth(format!(
                            "XOAUTH2 rejected: {} {}",
                            r2.code,
                            r2.text()
                        )))
                    }
                    _ => Err(SmtpError::Auth(format!(
                        "XOAUTH2 rejected: {} {}",
                        r.code,
                        r.text()
                    ))),
                }
            }
        }
    }

    fn expect_auth_ok(reply: Reply) -> Result<(), SmtpError> {
        if reply.code == 235 {
            Ok(())
        } else {
            Err(SmtpError::Auth(format!(
                "authentication failed: {} {}",
                reply.code,
                reply.text()
            )))
        }
    }

    /// `MAIL FROM:<from>` with optional `SIZE=` (when the server advertised
    /// SIZE) and `BODY=8BITMIME` (when it advertised 8BITMIME).
    pub(crate) async fn mail_from(
        &mut self,
        from: &str,
        size: Option<usize>,
        body_8bit: bool,
    ) -> Result<(), SmtpError> {
        let mut cmd = format!("MAIL FROM:<{from}>");
        if let Some(sz) = size {
            cmd.push_str(&format!(" SIZE={sz}"));
        }
        if body_8bit {
            cmd.push_str(" BODY=8BITMIME");
        }
        cmd.push_str("\r\n");
        let reply = self.command(&cmd).await?;
        if reply.is_positive() {
            Ok(())
        } else {
            Err(SmtpError::Protocol(format!(
                "MAIL FROM rejected: {} {}",
                reply.code,
                reply.text()
            )))
        }
    }

    /// `RCPT TO:<addr>` — a rejection is returned as data, not an error, so the
    /// caller can record per-recipient outcomes and still deliver to the rest.
    pub(crate) async fn rcpt_to(&mut self, addr: &str) -> Result<RcptOutcome, SmtpError> {
        let reply = self.command(&format!("RCPT TO:<{addr}>\r\n")).await?;
        if reply.is_positive() {
            Ok(RcptOutcome::Accepted)
        } else {
            Ok(RcptOutcome::Rejected {
                reason: format!("{} {}", reply.code, reply.text()),
            })
        }
    }

    /// `DATA` → dot-stuffed body → `.` terminator. Requires the `354`
    /// intermediate reply and a `2xx` acceptance of the queued message.
    pub(crate) async fn data(&mut self, raw: &[u8]) -> Result<(), SmtpError> {
        let reply = self.command("DATA\r\n").await?;
        if reply.code != 354 {
            return Err(SmtpError::Protocol(format!(
                "DATA rejected: {} {}",
                reply.code,
                reply.text()
            )));
        }
        let mut payload = Vec::with_capacity(raw.len() + 32);
        dot_stuff(raw, &mut payload);
        if !payload.ends_with(b"\r\n") {
            payload.extend_from_slice(b"\r\n");
        }
        payload.extend_from_slice(b".\r\n");
        self.write_all(&payload).await?;

        let reply = self.read_reply().await?;
        if reply.is_positive() {
            Ok(())
        } else {
            Err(SmtpError::Protocol(format!(
                "message rejected after DATA: {} {}",
                reply.code,
                reply.text()
            )))
        }
    }

    /// Best-effort `QUIT`; submission has already succeeded, so a failure here
    /// is not propagated.
    pub(crate) async fn quit(&mut self) {
        let _ = self.command("QUIT\r\n").await;
    }

    /// Recover the underlying stream for a `STARTTLS` upgrade. Any buffered but
    /// unread bytes are a protocol violation (a server injecting data before the
    /// TLS handshake), so we refuse rather than carry them across the boundary.
    pub(crate) fn into_inner(self) -> Result<S, SmtpError> {
        if self.rbuf.is_empty() {
            Ok(self.stream)
        } else {
            Err(SmtpError::Protocol(
                "unexpected data buffered before STARTTLS handshake".into(),
            ))
        }
    }
}

/// SMTP dot-stuffing (RFC 5321 §4.5.2): any line beginning with `.` gets an
/// extra leading `.` so it cannot be mistaken for the `<CRLF>.<CRLF>` end-of-
/// data marker. Handles both CRLF and bare-LF line endings in the source.
pub(crate) fn dot_stuff(raw: &[u8], out: &mut Vec<u8>) {
    let mut at_line_start = true;
    for &b in raw {
        if at_line_start && b == b'.' {
            out.push(b'.');
        }
        out.push(b);
        at_line_start = b == b'\n';
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_gmail_style_capabilities() {
        let lines: Vec<String> = [
            "SIZE 35882577",
            "8BITMIME",
            "STARTTLS",
            "ENHANCEDSTATUSCODES",
            "PIPELINING",
            "CHUNKING",
            "SMTPUTF8",
            "AUTH LOGIN PLAIN XOAUTH2 PLAIN-CLIENTTOKEN OAUTHBEARER",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let caps = Capabilities::parse(&lines);
        assert_eq!(caps.size, Some(35_882_577));
        assert!(caps.eightbitmime);
        assert!(caps.starttls);
        assert!(caps.pipelining);
        assert!(caps.offers("plain"));
        assert!(caps.offers("LOGIN"));
        assert!(caps.offers("xoauth2"));
    }

    #[test]
    fn parse_legacy_auth_equals_form() {
        let lines = vec!["AUTH=PLAIN LOGIN".to_string()];
        let caps = Capabilities::parse(&lines);
        assert!(caps.offers("PLAIN"));
        assert!(caps.offers("LOGIN"));
    }

    #[test]
    fn size_without_value_is_zero() {
        let caps = Capabilities::parse(&["SIZE".to_string()]);
        assert_eq!(caps.size, Some(0));
    }

    #[test]
    fn dot_stuffing_escapes_leading_dots() {
        let raw = b"Subject: hi\r\n\r\n.\r\n..oops\r\nnormal\r\n";
        let mut out = Vec::new();
        dot_stuff(raw, &mut out);
        assert_eq!(
            out,
            b"Subject: hi\r\n\r\n..\r\n...oops\r\nnormal\r\n".to_vec()
        );
    }

    #[test]
    fn dot_stuffing_handles_bare_lf() {
        let mut out = Vec::new();
        dot_stuff(b".x\n.y", &mut out);
        assert_eq!(out, b"..x\n..y".to_vec());
    }

    // --- STARTTLS framing over a cleartext mock socket -------------------
    //
    // A real TLS handshake can't complete against a cleartext mock, so this
    // drives the 587 pre-upgrade sequence — greeting, EHLO (must advertise
    // STARTTLS), the `STARTTLS` command, its `220`, and the clean handoff of the
    // inner stream — up to the point the transport would be swapped for TLS.

    use tokio::io::AsyncWriteExt;
    use tokio::net::{TcpListener, TcpStream};

    async fn read_one(sock: &mut TcpStream) -> String {
        use tokio::io::AsyncReadExt;
        let mut buf = Vec::new();
        let mut tmp = [0u8; 256];
        loop {
            if let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let mut line: Vec<u8> = buf.drain(..=pos).collect();
                line.pop();
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                return String::from_utf8_lossy(&line).into_owned();
            }
            let n = sock.read(&mut tmp).await.unwrap();
            if n == 0 {
                return String::from_utf8_lossy(&buf).into_owned();
            }
            buf.extend_from_slice(&tmp[..n]);
        }
    }

    #[tokio::test]
    async fn starttls_branch_framing() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            sock.write_all(b"220 mock ESMTP\r\n").await.unwrap();

            let ehlo = read_one(&mut sock).await;
            assert_eq!(ehlo, "EHLO client.test");
            sock.write_all(b"250-mock\r\n250-STARTTLS\r\n250 8BITMIME\r\n")
                .await
                .unwrap();

            let cmd = read_one(&mut sock).await;
            assert_eq!(cmd, "STARTTLS");
            sock.write_all(b"220 2.0.0 Ready to start TLS\r\n")
                .await
                .unwrap();
        });

        let tcp = TcpStream::connect(addr).await.unwrap();
        let mut conn = Connection::new(tcp);
        conn.read_greeting().await.unwrap();
        let caps = conn.ehlo("client.test").await.unwrap();
        assert!(caps.starttls, "server advertised STARTTLS");

        conn.starttls().await.expect("STARTTLS accepted with 220");

        // Nothing was injected before the handshake, so the inner stream is
        // recovered cleanly for the (real) TLS upgrade.
        assert!(conn.into_inner().is_ok());
        server.await.unwrap();
    }
}
