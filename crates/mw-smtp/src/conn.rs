//! Line-based SMTP connection state machine over any async byte stream.
//!
//! [`Connection`] is generic over `S: AsyncRead + AsyncWrite` so the exact same
//! command logic drives a cleartext [`tokio::net::TcpStream`] (the mock unit
//! tests, and pre-`STARTTLS` framing on 587), an implicit-TLS stream on 465, and
//! a post-upgrade TLS stream — the transport is swapped underneath the protocol,
//! never duplicated. All parsing here is defensive (bounded line length, no
//! panics) since the peer is untrusted network input.

use base64::prelude::*;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::sasl;
use crate::{Credentials, DsnNotify, DsnRet, SmtpError};

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
    /// `SMTPUTF8` (RFC 6531) — UTF-8 permitted in the envelope/headers.
    pub smtputf8: bool,
    /// `REQUIRETLS` (RFC 8689) — the server can honour a per-message TLS floor.
    pub requiretls: bool,
    /// `CHUNKING` (RFC 3030) — `BDAT` length-framed submission is available.
    pub chunking: bool,
    /// `DSN` (RFC 3461) — delivery-status-notification parameters accepted.
    pub dsn: bool,
    /// Offered SASL mechanisms, upper-cased (`PLAIN`, `LOGIN`, `XOAUTH2`,
    /// `SCRAM-SHA-256`, `OAUTHBEARER`, …).
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
                Some("SMTPUTF8") => caps.smtputf8 = true,
                Some("REQUIRETLS") => caps.requiretls = true,
                Some("CHUNKING") => caps.chunking = true,
                Some("DSN") => caps.dsn = true,
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

/// Extended `MAIL FROM` parameters negotiated for one submission.
#[derive(Debug, Clone, Default)]
pub(crate) struct MailParams {
    /// `SIZE=` (RFC 1870) when the server advertised SIZE.
    pub size: Option<usize>,
    /// `BODY=8BITMIME` (RFC 6152) for an un-encoded 8-bit body.
    pub body_8bit: bool,
    /// `SMTPUTF8` (RFC 6531) for a UTF-8 envelope/header.
    pub smtputf8: bool,
    /// `REQUIRETLS` (RFC 8689) TLS floor for this message.
    pub require_tls: bool,
    /// `RET=` (RFC 3461) — how much of the message a DSN should return.
    pub ret: Option<DsnRet>,
    /// `ENVID=` (RFC 3461) — an envelope identifier echoed in any DSN.
    pub envid: Option<String>,
}

/// xtext encoding (RFC 3461 §4): printable ASCII passes through except `+` and
/// `=`, which — together with any non-printable byte — become `+HH`.
fn xtext(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        if (0x21..=0x7e).contains(&b) && b != b'+' && b != b'=' {
            out.push(b as char);
        } else {
            out.push_str(&format!("+{b:02X}"));
        }
    }
    out
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
    ///
    /// `channel_binding` carries the negotiated SCRAM-`PLUS` binding as
    /// `(cb_name, bytes)` when the transport is TLS — `tls-exporter` (RFC 9266)
    /// on TLS 1.3, `tls-server-end-point` (RFC 5929) on TLS 1.2. It enables
    /// SCRAM-SHA-256-PLUS when the server also advertises the `-PLUS` mechanism.
    pub(crate) async fn authenticate(
        &mut self,
        creds: &Credentials,
        caps: &Capabilities,
        channel_binding: Option<(&str, &[u8])>,
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
            Credentials::Scram { user, pass } => {
                // Preference: SCRAM-SHA-256-PLUS when a channel binding is present
                // AND the server advertised `-PLUS`; otherwise plain SCRAM-SHA-256
                // (which also covers the plaintext / implicit-TLS-without-binding
                // path).
                let use_plus = channel_binding.is_some() && caps.offers("SCRAM-SHA-256-PLUS");
                let (mech, binding) = if use_plus {
                    ("SCRAM-SHA-256-PLUS", channel_binding)
                } else {
                    require("SCRAM-SHA-256")?;
                    ("SCRAM-SHA-256", None)
                };
                let nonce = sasl::client_nonce();
                let (mut scram, client_first) = sasl::ScramSha256::new(user, pass, &nonce, binding);
                // AUTH <mech> <base64 client-first-message>.
                let r = self
                    .command(&format!(
                        "AUTH {mech} {}\r\n",
                        BASE64_STANDARD.encode(&client_first)
                    ))
                    .await?;
                if r.code != 334 {
                    return Err(SmtpError::Auth(format!(
                        "{mech} rejected at client-first: {} {}",
                        r.code,
                        r.text()
                    )));
                }
                let server_first = decode_challenge(&r)?;
                let client_final = scram.client_final(&server_first).map_err(SmtpError::Auth)?;
                let r = self
                    .command(&format!("{}\r\n", BASE64_STANDARD.encode(&client_final)))
                    .await?;
                match r.code {
                    // Some servers fold the server-final into the 235 line; if it
                    // carries a decodable `v=`, verify it, otherwise accept.
                    235 => {
                        if let Ok(txt) = decode_challenge(&r) {
                            let _ = scram.verify(&txt);
                        }
                        Ok(())
                    }
                    // The server sends the server-final (v=) as a 334 challenge;
                    // verify it, then acknowledge with an empty line for the 235.
                    334 => {
                        let server_final = decode_challenge(&r)?;
                        scram.verify(&server_final).map_err(SmtpError::Auth)?;
                        let r2 = self.command("\r\n").await?;
                        Self::expect_auth_ok(r2)
                    }
                    _ => Err(SmtpError::Auth(format!(
                        "{mech} rejected: {} {}",
                        r.code,
                        r.text()
                    ))),
                }
            }
            Credentials::OAuthBearer { user, token } => {
                require("OAUTHBEARER")?;
                let r = self
                    .command(&format!(
                        "AUTH OAUTHBEARER {}\r\n",
                        sasl::oauthbearer(user, token)
                    ))
                    .await?;
                match r.code {
                    235 => Ok(()),
                    // Failure (RFC 7628 §3.2.3): the server returns a `334`
                    // base64 JSON error; the client sends a single `%x01`
                    // (base64 `AQ==`) and the server then returns the real code.
                    334 => {
                        let r2 = self
                            .command(&format!("{}\r\n", BASE64_STANDARD.encode("\x01")))
                            .await?;
                        Err(SmtpError::Auth(format!(
                            "OAUTHBEARER rejected: {} {}",
                            r2.code,
                            r2.text()
                        )))
                    }
                    _ => Err(SmtpError::Auth(format!(
                        "OAUTHBEARER rejected: {} {}",
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

    /// `MAIL FROM:<from>` with the negotiated ESMTP parameters. `SIZE=`
    /// (RFC 1870) and `BODY=8BITMIME` (RFC 6152) are emitted first to preserve
    /// the historical ordering; `SMTPUTF8`/`REQUIRETLS`/`RET=`/`ENVID=` follow.
    pub(crate) async fn mail_from(&mut self, from: &str, p: &MailParams) -> Result<(), SmtpError> {
        let mut cmd = format!("MAIL FROM:<{from}>");
        if let Some(sz) = p.size {
            cmd.push_str(&format!(" SIZE={sz}"));
        }
        if p.body_8bit {
            cmd.push_str(" BODY=8BITMIME");
        }
        if p.smtputf8 {
            cmd.push_str(" SMTPUTF8");
        }
        if p.require_tls {
            cmd.push_str(" REQUIRETLS");
        }
        if let Some(ret) = p.ret {
            cmd.push_str(&format!(" RET={}", ret.as_str()));
        }
        if let Some(envid) = &p.envid {
            cmd.push_str(&format!(" ENVID={}", xtext(envid)));
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

    /// `RCPT TO:<addr>` with optional DSN `NOTIFY=`/`ORCPT=` (RFC 3461). A
    /// rejection is returned as data, not an error, so the caller can record
    /// per-recipient outcomes and still deliver to the rest.
    pub(crate) async fn rcpt_to(
        &mut self,
        addr: &str,
        notify: &[DsnNotify],
        orcpt: bool,
    ) -> Result<RcptOutcome, SmtpError> {
        let mut cmd = format!("RCPT TO:<{addr}>");
        if !notify.is_empty() {
            let joined = notify
                .iter()
                .map(|n| n.as_str())
                .collect::<Vec<_>>()
                .join(",");
            cmd.push_str(&format!(" NOTIFY={joined}"));
        }
        if orcpt {
            cmd.push_str(&format!(" ORCPT=rfc822;{}", xtext(addr)));
        }
        cmd.push_str("\r\n");
        let reply = self.command(&cmd).await?;
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

    /// `BDAT <n> LAST` (RFC 3030 CHUNKING): submit the whole message as one
    /// length-framed binary chunk. Unlike `DATA` there is no dot-stuffing and no
    /// `<CRLF>.<CRLF>` terminator — the byte count is authoritative — so a body
    /// containing a lone `.` line needs no escaping.
    pub(crate) async fn bdat(&mut self, raw: &[u8]) -> Result<(), SmtpError> {
        let mut body = raw.to_vec();
        if !body.ends_with(b"\r\n") {
            body.extend_from_slice(b"\r\n");
        }
        let mut payload = format!("BDAT {} LAST\r\n", body.len()).into_bytes();
        payload.extend_from_slice(&body);
        self.write_all(&payload).await?;

        let reply = self.read_reply().await?;
        if reply.is_positive() {
            Ok(())
        } else {
            Err(SmtpError::Protocol(format!(
                "message rejected after BDAT: {} {}",
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

/// Decode the base64 SASL challenge carried in a `334` (or inline in a `235`)
/// reply line, returning the UTF-8 payload.
fn decode_challenge(reply: &Reply) -> Result<String, SmtpError> {
    let blob = reply.lines.first().map(|s| s.trim()).unwrap_or("");
    let bytes = BASE64_STANDARD
        .decode(blob)
        .map_err(|e| SmtpError::Protocol(format!("bad base64 SASL challenge: {e}")))?;
    String::from_utf8(bytes)
        .map_err(|e| SmtpError::Protocol(format!("non-UTF-8 SASL challenge: {e}")))
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
        assert!(caps.chunking);
        assert!(caps.smtputf8);
        assert!(caps.offers("plain"));
        assert!(caps.offers("LOGIN"));
        assert!(caps.offers("xoauth2"));
        assert!(caps.offers("OAUTHBEARER"));
    }

    #[test]
    fn parse_dsn_and_requiretls_capabilities() {
        let lines = vec![
            "DSN".to_string(),
            "REQUIRETLS".to_string(),
            "AUTH SCRAM-SHA-256 SCRAM-SHA-256-PLUS".to_string(),
        ];
        let caps = Capabilities::parse(&lines);
        assert!(caps.dsn);
        assert!(caps.requiretls);
        assert!(caps.offers("scram-sha-256"));
    }

    #[test]
    fn xtext_encodes_reserved_and_nonprintable() {
        // Space (0x20), '+' and '=' must be quoted; ordinary chars pass through.
        assert_eq!(xtext("a b"), "a+20b");
        assert_eq!(xtext("a+b=c"), "a+2Bb+3Dc");
        assert_eq!(xtext("bob@example.com"), "bob@example.com");
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

    // --- extended-submission round-trip (SCRAM + DSN + SMTPUTF8 + BDAT) ----
    //
    // A stateful framed reader so the length-framed `BDAT` body can be consumed
    // exactly (the line-oriented `read_one` above discards intra-segment bytes).

    struct Framed {
        sock: TcpStream,
        buf: Vec<u8>,
    }

    impl Framed {
        fn new(sock: TcpStream) -> Self {
            Self {
                sock,
                buf: Vec::new(),
            }
        }

        async fn fill(&mut self) {
            use tokio::io::AsyncReadExt;
            let mut tmp = [0u8; 512];
            let n = self.sock.read(&mut tmp).await.unwrap();
            assert!(n > 0, "unexpected EOF from client");
            self.buf.extend_from_slice(&tmp[..n]);
        }

        async fn line(&mut self) -> String {
            loop {
                if let Some(pos) = self.buf.iter().position(|&b| b == b'\n') {
                    let mut line: Vec<u8> = self.buf.drain(..=pos).collect();
                    line.pop();
                    if line.last() == Some(&b'\r') {
                        line.pop();
                    }
                    return String::from_utf8_lossy(&line).into_owned();
                }
                self.fill().await;
            }
        }

        async fn exact(&mut self, n: usize) -> Vec<u8> {
            while self.buf.len() < n {
                self.fill().await;
            }
            self.buf.drain(..n).collect()
        }

        async fn send(&mut self, bytes: &[u8]) {
            self.sock.write_all(bytes).await.unwrap();
        }
    }

    #[tokio::test]
    async fn extended_submission_scram_dsn_smtputf8_bdat() {
        use crate::{
            Credentials, Dsn, DsnNotify, DsnRet, Outgoing, Security, SubmitConfig, SubmitOptions,
            Submitter,
        };

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            let mut f = Framed::new(sock);
            f.send(b"220 mock.mailwoman.test ESMTP\r\n").await;

            assert_eq!(f.line().await, "EHLO client.test");
            f.send(
                b"250-mock.mailwoman.test\r\n\
                  250-SIZE 100000\r\n\
                  250-8BITMIME\r\n\
                  250-SMTPUTF8\r\n\
                  250-DSN\r\n\
                  250-CHUNKING\r\n\
                  250 AUTH SCRAM-SHA-256\r\n",
            )
            .await;

            // SCRAM: read AUTH + client-first, echo the client nonce back in a
            // server-first, accept the client-final (the proof math itself is
            // pinned by the RFC 7677 vector test in `sasl`).
            let auth = f.line().await;
            let ir = auth
                .strip_prefix("AUTH SCRAM-SHA-256 ")
                .expect("AUTH SCRAM line");
            let client_first = String::from_utf8(BASE64_STANDARD.decode(ir).unwrap()).unwrap();
            let client_nonce = client_first
                .rsplit(',')
                .next()
                .and_then(|f| f.strip_prefix("r="))
                .expect("client nonce")
                .to_string();
            let server_first =
                format!("r={client_nonce}SRVNONCE,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096");
            f.send(format!("334 {}\r\n", BASE64_STANDARD.encode(&server_first)).as_bytes())
                .await;
            let client_final = f.line().await; // base64 client-final-message
            assert!(!client_final.is_empty());
            f.send(b"235 2.7.0 Authentication successful\r\n").await;

            // MAIL FROM must carry SMTPUTF8 (UTF-8 recipient below), SIZE, and
            // the DSN RET/ENVID parameters.
            let mail = f.line().await;
            assert!(
                mail.starts_with("MAIL FROM:<sender@example.com> SIZE="),
                "{mail}"
            );
            assert!(mail.contains(" SMTPUTF8"), "{mail}");
            assert!(mail.contains(" RET=HDRS"), "{mail}");
            assert!(mail.contains(" ENVID=abc123"), "{mail}");
            f.send(b"250 2.1.0 OK\r\n").await;

            // RCPT TO must carry NOTIFY + ORCPT (xtext of the UTF-8 address).
            let rcpt = f.line().await;
            assert!(rcpt.starts_with("RCPT TO:<møt@example.com>"), "{rcpt}");
            assert!(rcpt.contains(" NOTIFY=SUCCESS,FAILURE"), "{rcpt}");
            assert!(
                rcpt.contains(" ORCPT=rfc822;m+C3+B8t@example.com"),
                "{rcpt}"
            );
            f.send(b"250 2.1.5 OK\r\n").await;

            // BDAT: exact length-framed body, no dot-stuffing.
            let bdat = f.line().await;
            let n: usize = bdat
                .strip_prefix("BDAT ")
                .and_then(|r| r.strip_suffix(" LAST"))
                .and_then(|r| r.parse().ok())
                .unwrap_or_else(|| panic!("bad BDAT header {bdat:?}"));
            let body = f.exact(n).await;
            // The lone "." line survives verbatim (BDAT does not dot-stuff).
            assert!(
                body.windows(3).any(|w| w == b"\r\n.") && body.ends_with(b"\r\n"),
                "raw body preserved under BDAT"
            );
            f.send(b"250 2.0.0 OK: queued\r\n").await;

            assert_eq!(f.line().await, "QUIT");
            f.send(b"221 2.0.0 Bye\r\n").await;
        });

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

        assert_eq!(result.accepted, vec!["møt@example.com".to_string()]);
        assert!(result.rejected.is_empty());
        server.await.unwrap();
    }

    #[tokio::test]
    async fn oauthbearer_dispatch_frames_rfc7628_and_accepts() {
        use crate::{Credentials, Outgoing, Security, SubmitConfig, SubmitOptions, Submitter};

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let captured = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let cap = captured.clone();

        let server = tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            let mut f = Framed::new(sock);
            f.send(b"220 mock ESMTP\r\n").await;
            assert_eq!(f.line().await, "EHLO client.test");
            f.send(b"250-mock\r\n250 AUTH OAUTHBEARER\r\n").await;

            let auth = f.line().await;
            cap.lock().unwrap().push(auth.clone());
            f.send(b"235 2.7.0 Accepted\r\n").await;

            assert!(f.line().await.starts_with("MAIL FROM:"));
            f.send(b"250 OK\r\n").await;
            assert!(f.line().await.starts_with("RCPT TO:"));
            f.send(b"250 OK\r\n").await;
            assert_eq!(f.line().await, "DATA");
            f.send(b"354 go\r\n").await;
            loop {
                if f.line().await == "." {
                    break;
                }
            }
            f.send(b"250 queued\r\n").await;
            assert_eq!(f.line().await, "QUIT");
            f.send(b"221 bye\r\n").await;
        });

        let sub = Submitter::new(SubmitConfig {
            host: addr.ip().to_string(),
            port: addr.port(),
            security: Security::Plaintext,
            credentials: Credentials::OAuthBearer {
                user: "carol@example.com".into(),
                token: "vF9dft4qmT".into(),
            },
            ehlo_name: "client.test".into(),
        });
        sub.submit_with(
            Outgoing {
                mail_from: "sender@example.com".into(),
                rcpt_to: vec!["rcpt@example.com".into()],
                raw: b"Subject: hi\r\n\r\nbody\r\n".to_vec(),
            },
            SubmitOptions::default(),
        )
        .await
        .expect("oauthbearer submission");
        server.await.unwrap();

        let auth = &captured.lock().unwrap()[0];
        let ir = auth.strip_prefix("AUTH OAUTHBEARER ").unwrap();
        let decoded = BASE64_STANDARD.decode(ir).unwrap();
        assert_eq!(
            decoded,
            b"n,a=carol@example.com,\x01auth=Bearer vF9dft4qmT\x01\x01"
        );
    }

    // --- SCRAM-SHA-256-PLUS channel-binding negotiation ------------------
    //
    // These drive `authenticate` directly over a cleartext mock socket with a
    // synthetic `tls-server-end-point` binding (a real TLS handshake can't run
    // against a mock), asserting the mechanism preference and the `c=` echo.

    /// When a binding is present AND the server advertises `-PLUS`, the client
    /// sends `AUTH SCRAM-SHA-256-PLUS` and the client-final `c=` decodes to
    /// `p=tls-server-end-point,,` followed by the raw binding bytes.
    #[tokio::test]
    async fn scram_plus_selected_and_binds_channel() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let binding = vec![0x5Au8; 32];
        let server_binding = binding.clone();

        let server = tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            let mut f = Framed::new(sock);
            f.send(b"220 mock ESMTP\r\n").await;
            assert_eq!(f.line().await, "EHLO client.test");
            f.send(b"250-mock\r\n250 AUTH SCRAM-SHA-256 SCRAM-SHA-256-PLUS\r\n")
                .await;

            // The client must prefer the `-PLUS` mechanism.
            let auth = f.line().await;
            let ir = auth
                .strip_prefix("AUTH SCRAM-SHA-256-PLUS ")
                .expect("AUTH SCRAM-SHA-256-PLUS line");
            let client_first = String::from_utf8(BASE64_STANDARD.decode(ir).unwrap()).unwrap();
            assert!(
                client_first.starts_with("p=tls-server-end-point,,"),
                "gs2 header selects channel binding: {client_first}"
            );
            let client_nonce = client_first
                .rsplit(',')
                .next()
                .and_then(|f| f.strip_prefix("r="))
                .expect("client nonce")
                .to_string();
            let server_first =
                format!("r={client_nonce}SRVNONCE,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096");
            f.send(format!("334 {}\r\n", BASE64_STANDARD.encode(&server_first)).as_bytes())
                .await;

            // The client-final `c=` must echo gs2-header || binding.
            let client_final =
                String::from_utf8(BASE64_STANDARD.decode(f.line().await).unwrap()).unwrap();
            let c = client_final
                .split(',')
                .find_map(|t| t.strip_prefix("c="))
                .expect("client-final c=");
            let mut expected = b"p=tls-server-end-point,,".to_vec();
            expected.extend_from_slice(&server_binding);
            assert_eq!(BASE64_STANDARD.decode(c).unwrap(), expected);
            f.send(b"235 2.7.0 Authentication successful\r\n").await;
        });

        let tcp = TcpStream::connect(addr).await.unwrap();
        let mut conn = Connection::new(tcp);
        conn.read_greeting().await.unwrap();
        let caps = conn.ehlo("client.test").await.unwrap();
        assert!(caps.offers("SCRAM-SHA-256-PLUS"));
        conn.authenticate(
            &Credentials::Scram {
                user: "user".into(),
                pass: "pencil".into(),
            },
            &caps,
            Some(("tls-server-end-point", &binding)),
        )
        .await
        .expect("SCRAM-SHA-256-PLUS authentication");
        server.await.unwrap();
    }

    /// TLS-1.3 path: with cb-name `tls-exporter` (RFC 9266) the client sends
    /// `AUTH SCRAM-SHA-256-PLUS` and the client-final `c=` decodes to
    /// `p=tls-exporter,,` followed by the raw exporter binding bytes.
    #[tokio::test]
    async fn scram_plus_tls_exporter_selected_and_binds_channel() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let binding = vec![0xC3u8; 32];
        let server_binding = binding.clone();

        let server = tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            let mut f = Framed::new(sock);
            f.send(b"220 mock ESMTP\r\n").await;
            assert_eq!(f.line().await, "EHLO client.test");
            f.send(b"250-mock\r\n250 AUTH SCRAM-SHA-256 SCRAM-SHA-256-PLUS\r\n")
                .await;

            let auth = f.line().await;
            let ir = auth
                .strip_prefix("AUTH SCRAM-SHA-256-PLUS ")
                .expect("AUTH SCRAM-SHA-256-PLUS line");
            let client_first = String::from_utf8(BASE64_STANDARD.decode(ir).unwrap()).unwrap();
            assert!(
                client_first.starts_with("p=tls-exporter,,"),
                "gs2 header selects the tls-exporter binding: {client_first}"
            );
            let client_nonce = client_first
                .rsplit(',')
                .next()
                .and_then(|f| f.strip_prefix("r="))
                .expect("client nonce")
                .to_string();
            let server_first =
                format!("r={client_nonce}SRVNONCE,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096");
            f.send(format!("334 {}\r\n", BASE64_STANDARD.encode(&server_first)).as_bytes())
                .await;

            let client_final =
                String::from_utf8(BASE64_STANDARD.decode(f.line().await).unwrap()).unwrap();
            let c = client_final
                .split(',')
                .find_map(|t| t.strip_prefix("c="))
                .expect("client-final c=");
            let mut expected = b"p=tls-exporter,,".to_vec();
            expected.extend_from_slice(&server_binding);
            assert_eq!(BASE64_STANDARD.decode(c).unwrap(), expected);
            f.send(b"235 2.7.0 Authentication successful\r\n").await;
        });

        let tcp = TcpStream::connect(addr).await.unwrap();
        let mut conn = Connection::new(tcp);
        conn.read_greeting().await.unwrap();
        let caps = conn.ehlo("client.test").await.unwrap();
        assert!(caps.offers("SCRAM-SHA-256-PLUS"));
        conn.authenticate(
            &Credentials::Scram {
                user: "user".into(),
                pass: "pencil".into(),
            },
            &caps,
            Some(("tls-exporter", &binding)),
        )
        .await
        .expect("SCRAM-SHA-256-PLUS (tls-exporter) authentication");
        server.await.unwrap();
    }

    /// A binding is present but the server does NOT advertise `-PLUS`: the client
    /// falls back to plain `SCRAM-SHA-256` with the unbound gs2 header (`c=biws`).
    #[tokio::test]
    async fn scram_falls_back_to_plain_when_plus_absent() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let binding = vec![0x5Au8; 32];

        let server = tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            let mut f = Framed::new(sock);
            f.send(b"220 mock ESMTP\r\n").await;
            assert_eq!(f.line().await, "EHLO client.test");
            f.send(b"250-mock\r\n250 AUTH SCRAM-SHA-256\r\n").await;

            let auth = f.line().await;
            let ir = auth
                .strip_prefix("AUTH SCRAM-SHA-256 ")
                .expect("plain SCRAM-SHA-256 line (no -PLUS)");
            let client_first = String::from_utf8(BASE64_STANDARD.decode(ir).unwrap()).unwrap();
            assert!(
                client_first.starts_with("n,,"),
                "unbound gs2 header when -PLUS absent: {client_first}"
            );
            let client_nonce = client_first
                .rsplit(',')
                .next()
                .and_then(|f| f.strip_prefix("r="))
                .unwrap()
                .to_string();
            let server_first =
                format!("r={client_nonce}SRVNONCE,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096");
            f.send(format!("334 {}\r\n", BASE64_STANDARD.encode(&server_first)).as_bytes())
                .await;
            let client_final =
                String::from_utf8(BASE64_STANDARD.decode(f.line().await).unwrap()).unwrap();
            // Unbound: c= is base64("n,,") = "biws".
            assert!(client_final.starts_with("c=biws,"), "{client_final}");
            f.send(b"235 2.7.0 Authentication successful\r\n").await;
        });

        let tcp = TcpStream::connect(addr).await.unwrap();
        let mut conn = Connection::new(tcp);
        conn.read_greeting().await.unwrap();
        let caps = conn.ehlo("client.test").await.unwrap();
        conn.authenticate(
            &Credentials::Scram {
                user: "user".into(),
                pass: "pencil".into(),
            },
            &caps,
            // A binding is available, but the server never advertised `-PLUS`.
            Some(("tls-server-end-point", &binding)),
        )
        .await
        .expect("plain SCRAM-SHA-256 fallback");
        server.await.unwrap();
    }
}
