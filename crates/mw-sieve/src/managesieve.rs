//! ManageSieve (RFC 5804) client — upload/list/activate/fetch scripts on servers
//! that advertise it (plan §0.6, §3 e2).
//!
//! The protocol logic lives on [`Connection`], generic over any async byte
//! stream, so the transcript tests drive it over an in-memory pipe while
//! production uses [`crate::transport::SieveStream`]. ManageSieve is a
//! line-oriented text protocol: a response is zero or more data lines/literals
//! followed by an `OK`/`NO`/`BYE` completion; script bodies travel as `{n+}`
//! non-synchronizing literals.
//!
//! Implemented commands: `CAPABILITY`, `AUTHENTICATE` (SASL PLAIN + LOGIN),
//! `STARTTLS` (framing only — the transport does the upgrade), `PUTSCRIPT`,
//! `LISTSCRIPTS`, `SETACTIVE`, `GETSCRIPT`, `DELETESCRIPT`, `NOOP`, `LOGOUT`.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{Result, SieveError};

/// Hard cap on one CRLF-terminated line, bounding memory against a hostile or
/// broken server that never sends a newline.
const MAX_LINE: usize = 65_536;

/// Server capabilities advertised in the greeting / `CAPABILITY` response.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Capabilities {
    /// `"IMPLEMENTATION"` — server name/version string.
    pub implementation: String,
    /// `"VERSION"` — ManageSieve protocol version.
    pub version: String,
    /// `"SASL"` — offered SASL mechanisms, upper-cased.
    pub sasl: Vec<String>,
    /// `"SIEVE"` — Sieve extensions the server supports.
    pub sieve: Vec<String>,
    /// `"STARTTLS"` advertised.
    pub starttls: bool,
    /// `"NOTIFY"` — notification methods (e.g. `mailto`).
    pub notify: Vec<String>,
}

impl Capabilities {
    /// Whether a SASL mechanism (case-insensitive) was advertised.
    pub fn offers_sasl(&self, mech: &str) -> bool {
        self.sasl.iter().any(|m| m.eq_ignore_ascii_case(mech))
    }

    /// Whether a Sieve extension (case-insensitive) is supported.
    pub fn supports(&self, ext: &str) -> bool {
        self.sieve.iter().any(|e| e.eq_ignore_ascii_case(ext))
    }

    fn absorb(&mut self, key: &str, value: &str) {
        let words = || value.split_whitespace().map(|s| s.to_string());
        match key.to_ascii_uppercase().as_str() {
            "IMPLEMENTATION" => self.implementation = value.to_string(),
            "VERSION" => self.version = value.to_string(),
            "SASL" => {
                self.sasl = value
                    .split_whitespace()
                    .map(|s| s.to_ascii_uppercase())
                    .collect()
            }
            "SIEVE" => self.sieve = words().collect(),
            "NOTIFY" => self.notify = words().collect(),
            "STARTTLS" => self.starttls = true,
            _ => {}
        }
    }
}

/// One entry from `LISTSCRIPTS`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptInfo {
    pub name: String,
    pub active: bool,
}

/// SASL credentials for `AUTHENTICATE` (a tiny local duplicate of the mw-imap/
/// mw-smtp approach, per plan §3 e2).
#[derive(Debug, Clone)]
pub enum Credentials {
    Plain { username: String, password: String },
    Login { username: String, password: String },
}

/// The completion status of a ManageSieve response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Ok,
    No,
    Bye,
}

/// A parsed response: any data payload plus the completion line.
#[derive(Debug, Clone)]
struct Response {
    /// Text data lines (LISTSCRIPTS / capability lines).
    lines: Vec<String>,
    /// Literal blobs (GETSCRIPT body).
    literals: Vec<Vec<u8>>,
    status: Status,
    /// Optional human-readable text on the completion line.
    text: Option<String>,
}

impl Response {
    fn into_ok(self) -> Result<Self> {
        match self.status {
            Status::Ok => Ok(self),
            Status::No => Err(SieveError::ManageSieve(format!(
                "server said NO: {}",
                self.text.clone().unwrap_or_default()
            ))),
            Status::Bye => Err(SieveError::ManageSieve(format!(
                "server said BYE: {}",
                self.text.clone().unwrap_or_default()
            ))),
        }
    }
}

/// A ManageSieve connection with a small read buffer for line framing.
pub struct Connection<S> {
    stream: S,
    rbuf: Vec<u8>,
    caps: Capabilities,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Connection<S> {
    /// Wrap an established stream and read the server greeting (a capability
    /// listing terminated by `OK`), populating [`capabilities`](Self::capabilities).
    pub async fn open(stream: S) -> Result<Self> {
        let mut conn = Connection {
            stream,
            rbuf: Vec::with_capacity(2048),
            caps: Capabilities::default(),
        };
        let resp = conn.read_response().await?;
        conn.caps = parse_capabilities(&resp.lines);
        resp.into_ok()?;
        Ok(conn)
    }

    /// The capabilities from the greeting (refreshed by [`capability`](Self::capability)).
    pub fn capabilities(&self) -> &Capabilities {
        &self.caps
    }

    /// Recover the inner stream for a `STARTTLS` transport upgrade. Any buffered
    /// unread bytes before the handshake are a protocol violation.
    pub fn into_inner(self) -> Result<S> {
        if self.rbuf.is_empty() {
            Ok(self.stream)
        } else {
            Err(SieveError::ManageSieve(
                "unexpected data buffered before STARTTLS handshake".into(),
            ))
        }
    }

    async fn read_line(&mut self) -> Result<String> {
        loop {
            if let Some(pos) = self.rbuf.iter().position(|&b| b == b'\n') {
                let mut line: Vec<u8> = self.rbuf.drain(..=pos).collect();
                line.pop();
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                return Ok(String::from_utf8_lossy(&line).into_owned());
            }
            if self.rbuf.len() > MAX_LINE {
                return Err(SieveError::ManageSieve(
                    "response line exceeded limit".into(),
                ));
            }
            let mut tmp = [0u8; 1024];
            let n = self.stream.read(&mut tmp).await?;
            if n == 0 {
                return Err(SieveError::ManageSieve(
                    "connection closed mid-response".into(),
                ));
            }
            self.rbuf.extend_from_slice(&tmp[..n]);
        }
    }

    /// Read exactly `len` payload bytes (a literal), then swallow a trailing CRLF.
    async fn read_literal(&mut self, len: usize) -> Result<Vec<u8>> {
        while self.rbuf.len() < len {
            let mut tmp = [0u8; 4096];
            let n = self.stream.read(&mut tmp).await?;
            if n == 0 {
                return Err(SieveError::ManageSieve(
                    "connection closed mid-literal".into(),
                ));
            }
            self.rbuf.extend_from_slice(&tmp[..n]);
        }
        let blob: Vec<u8> = self.rbuf.drain(..len).collect();
        // A literal is followed by CRLF before the next line; consume it if present.
        if self.rbuf.first() == Some(&b'\r') {
            self.rbuf.remove(0);
        }
        if self.rbuf.first() == Some(&b'\n') {
            self.rbuf.remove(0);
        }
        Ok(blob)
    }

    /// Read a full response: data lines / literals until an `OK`/`NO`/`BYE`.
    async fn read_response(&mut self) -> Result<Response> {
        let mut lines = Vec::new();
        let mut literals = Vec::new();
        loop {
            let line = self.read_line().await?;
            if let Some(status) = final_status(&line) {
                let (text, want_literal) = parse_completion_tail(&line);
                let text = if let Some(len) = want_literal {
                    let blob = self.read_literal(len).await?;
                    Some(String::from_utf8_lossy(&blob).into_owned())
                } else {
                    text
                };
                return Ok(Response {
                    lines,
                    literals,
                    status,
                    text,
                });
            }
            // A data line that is (or ends with) a `{n}` literal marker.
            if let Some(len) = literal_marker_len(&line) {
                let blob = self.read_literal(len).await?;
                literals.push(blob);
            } else {
                lines.push(line);
            }
        }
    }

    async fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
        self.stream.write_all(bytes).await?;
        self.stream.flush().await?;
        Ok(())
    }

    async fn command(&mut self, line: &str) -> Result<Response> {
        tracing::trace!(target: "mw_sieve::wire", "C: {}", line.trim_end());
        self.write_all(line.as_bytes()).await?;
        self.read_response().await
    }

    /// Re-query `CAPABILITY`, refreshing the stored capabilities.
    pub async fn capability(&mut self) -> Result<&Capabilities> {
        let resp = self.command("CAPABILITY\r\n").await?;
        self.caps = parse_capabilities(&resp.lines);
        resp.into_ok()?;
        Ok(&self.caps)
    }

    /// Frame the `STARTTLS` command and require its `OK` (the caller then swaps
    /// the transport to TLS and re-reads capabilities).
    pub async fn starttls(&mut self) -> Result<()> {
        self.command("STARTTLS\r\n").await?.into_ok()?;
        Ok(())
    }

    /// Authenticate via SASL PLAIN or LOGIN.
    pub async fn authenticate(&mut self, creds: &Credentials) -> Result<()> {
        match creds {
            Credentials::Plain { username, password } => {
                let ir = B64.encode(sasl_plain(username, password));
                let resp = self
                    .command(&format!("AUTHENTICATE \"PLAIN\" \"{ir}\"\r\n"))
                    .await?;
                resp.into_ok()?;
                Ok(())
            }
            Credentials::Login { username, password } => {
                self.authenticate_login(username, password).await
            }
        }
    }

    /// SASL LOGIN: server issues a username challenge, then a password challenge,
    /// each answered with a base64 quoted string.
    async fn authenticate_login(&mut self, username: &str, password: &str) -> Result<()> {
        tracing::trace!(target: "mw_sieve::wire", "C: AUTHENTICATE \"LOGIN\"");
        self.write_all(b"AUTHENTICATE \"LOGIN\"\r\n").await?;
        let answers = [B64.encode(username), B64.encode(password)];
        let mut idx = 0usize;
        loop {
            let line = self.read_line().await?;
            if let Some(status) = final_status(&line) {
                let (text, want_literal) = parse_completion_tail(&line);
                let text = if let Some(len) = want_literal {
                    let blob = self.read_literal(len).await?;
                    Some(String::from_utf8_lossy(&blob).into_owned())
                } else {
                    text
                };
                return Response {
                    lines: Vec::new(),
                    literals: Vec::new(),
                    status,
                    text,
                }
                .into_ok()
                .map(|_| ());
            }
            // Otherwise this is a server challenge (a literal or quoted string);
            // consume any literal body and reply with the next answer.
            if let Some(len) = literal_marker_len(&line) {
                let _ = self.read_literal(len).await?;
            }
            let answer = answers.get(idx).cloned().unwrap_or_default();
            idx += 1;
            self.write_all(format!("\"{answer}\"\r\n").as_bytes())
                .await?;
        }
    }

    /// `PUTSCRIPT "name" {n+}` + body — upload/replace a script.
    pub async fn put_script(&mut self, name: &str, body: &str) -> Result<()> {
        let bytes = body.as_bytes();
        let head = format!("PUTSCRIPT \"{}\" {{{}+}}\r\n", quote_arg(name), bytes.len());
        tracing::trace!(target: "mw_sieve::wire", "C: {}", head.trim_end());
        self.write_all(head.as_bytes()).await?;
        self.write_all(bytes).await?;
        self.write_all(b"\r\n").await?;
        self.read_response().await?.into_ok()?;
        Ok(())
    }

    /// `LISTSCRIPTS` — every stored script and whether it is active.
    pub async fn list_scripts(&mut self) -> Result<Vec<ScriptInfo>> {
        let resp = self.command("LISTSCRIPTS\r\n").await?;
        let scripts = resp
            .lines
            .iter()
            .filter_map(|l| parse_script_line(l))
            .collect();
        resp.into_ok()?;
        Ok(scripts)
    }

    /// `SETACTIVE "name"` — activate a script (empty name deactivates all).
    pub async fn set_active(&mut self, name: &str) -> Result<()> {
        self.command(&format!("SETACTIVE \"{}\"\r\n", quote_arg(name)))
            .await?
            .into_ok()?;
        Ok(())
    }

    /// `GETSCRIPT "name"` — fetch a stored script's source.
    pub async fn get_script(&mut self, name: &str) -> Result<String> {
        let resp = self
            .command(&format!("GETSCRIPT \"{}\"\r\n", quote_arg(name)))
            .await?;
        let body = resp
            .literals
            .first()
            .map(|b| String::from_utf8_lossy(b).into_owned())
            // Some servers return a short script as a quoted line rather than a literal.
            .or_else(|| resp.lines.first().cloned())
            .unwrap_or_default();
        resp.into_ok()?;
        Ok(body)
    }

    /// `DELETESCRIPT "name"`.
    pub async fn delete_script(&mut self, name: &str) -> Result<()> {
        self.command(&format!("DELETESCRIPT \"{}\"\r\n", quote_arg(name)))
            .await?
            .into_ok()?;
        Ok(())
    }

    /// `NOOP` — keepalive.
    pub async fn noop(&mut self) -> Result<()> {
        self.command("NOOP\r\n").await?.into_ok()?;
        Ok(())
    }

    /// `LOGOUT` — best-effort; the server answers `OK`/`BYE` and closes.
    pub async fn logout(&mut self) -> Result<()> {
        let resp = self.command("LOGOUT\r\n").await?;
        // Both OK and BYE are acceptable terminations.
        match resp.status {
            Status::Ok | Status::Bye => Ok(()),
            Status::No => resp.into_ok().map(|_| ()),
        }
    }
}

// --- parsing helpers --------------------------------------------------------

/// SASL PLAIN payload `authzid \0 authcid \0 passwd` (empty authzid).
fn sasl_plain(username: &str, password: &str) -> Vec<u8> {
    let mut raw = Vec::with_capacity(username.len() + password.len() + 2);
    raw.push(0);
    raw.extend_from_slice(username.as_bytes());
    raw.push(0);
    raw.extend_from_slice(password.as_bytes());
    raw
}

/// Escape a command argument for a `"..."` ManageSieve quoted-string.
fn quote_arg(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Classify a line as a completion (`OK`/`NO`/`BYE`) — the keyword must stand
/// alone or be followed by a space (so a script line like `"OKmail"` is data).
fn final_status(line: &str) -> Option<Status> {
    let head = line.split([' ']).next().unwrap_or("");
    match head.to_ascii_uppercase().as_str() {
        "OK" => Some(Status::Ok),
        "NO" => Some(Status::No),
        "BYE" => Some(Status::Bye),
        _ => None,
    }
}

/// Parse the tail of a completion line: optional `(CODE)` and either a trailing
/// quoted `"text"` or a `{n}` literal marker (whose length is returned so the
/// caller reads the literal body as the text).
fn parse_completion_tail(line: &str) -> (Option<String>, Option<usize>) {
    // Drop the status keyword.
    let rest = line.split_once(' ').map_or("", |(_, r)| r).trim();
    if rest.is_empty() {
        return (None, None);
    }
    // Strip a leading parenthesised response code, if any.
    let rest = if let Some(stripped) = rest.strip_prefix('(') {
        stripped.split_once(')').map_or("", |(_, r)| r).trim()
    } else {
        rest
    };
    if rest.is_empty() {
        return (None, None);
    }
    if let Some(len) = literal_marker_len(rest) {
        return (None, Some(len));
    }
    // A quoted string.
    if let Some(inner) = rest.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return (Some(unescape(inner)), None);
    }
    (Some(rest.to_string()), None)
}

/// If `line` ends with a `{n}` / `{n+}` literal marker, return `n`.
fn literal_marker_len(line: &str) -> Option<usize> {
    let trimmed = line.trim_end();
    let open = trimmed.rfind('{')?;
    let close = trimmed.rfind('}')?;
    if close < open || close != trimmed.len() - 1 {
        return None;
    }
    let digits = trimmed[open + 1..close].trim_end_matches('+');
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// Parse one `LISTSCRIPTS` line: `"name"` optionally followed by `ACTIVE`.
fn parse_script_line(line: &str) -> Option<ScriptInfo> {
    let line = line.trim();
    let inner_end = line[1..].find('"')? + 1;
    let name = unescape(&line[1..inner_end]);
    let active = line[inner_end + 1..].trim().eq_ignore_ascii_case("ACTIVE");
    Some(ScriptInfo { name, active })
}

/// Parse capability data lines into [`Capabilities`]. Each line is `"KEY"
/// ["VALUE"]`; `STARTTLS` may appear bare.
fn parse_capabilities(lines: &[String]) -> Capabilities {
    let mut caps = Capabilities::default();
    for line in lines {
        let (key, value) = split_cap_line(line);
        caps.absorb(&key, &value);
    }
    caps
}

/// Split `"KEY" "VALUE"` (or bare `"KEY"`) into key + value, unescaping both.
fn split_cap_line(line: &str) -> (String, String) {
    let line = line.trim();
    if !line.starts_with('"') {
        return (line.to_string(), String::new());
    }
    // First quoted token = key.
    let Some(end) = line[1..].find('"').map(|p| p + 1) else {
        return (line.to_string(), String::new());
    };
    let key = unescape(&line[1..end]);
    let rest = line[end + 1..].trim();
    let value = rest
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .map(unescape)
        .unwrap_or_default();
    (key, value)
}

/// Unescape a ManageSieve quoted-string body (`\\` and `\"`).
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_status_needs_word_boundary() {
        assert_eq!(final_status("OK \"done\""), Some(Status::Ok));
        assert_eq!(final_status("NO (QUOTA) \"full\""), Some(Status::No));
        assert_eq!(final_status("BYE"), Some(Status::Bye));
        assert_eq!(final_status("\"OKmail\""), None);
        assert_eq!(final_status("\"script\" ACTIVE"), None);
    }

    #[test]
    fn parse_completion_tail_variants() {
        assert_eq!(parse_completion_tail("OK"), (None, None));
        assert_eq!(
            parse_completion_tail("OK \"Success.\""),
            (Some("Success.".into()), None)
        );
        assert_eq!(
            parse_completion_tail("NO (QUOTA) \"Quota exceeded\""),
            (Some("Quota exceeded".into()), None)
        );
        assert_eq!(parse_completion_tail("NO {12}"), (None, Some(12)));
    }

    #[test]
    fn literal_marker_len_parses() {
        assert_eq!(literal_marker_len("{42}"), Some(42));
        assert_eq!(literal_marker_len("{42+}"), Some(42));
        assert_eq!(literal_marker_len("PUTSCRIPT \"x\" {7+}"), Some(7));
        assert_eq!(literal_marker_len("nope"), None);
        assert_eq!(literal_marker_len("{}"), None);
        assert_eq!(literal_marker_len("{a}"), None);
    }

    #[test]
    fn script_line_parses_active() {
        assert_eq!(
            parse_script_line("\"vacation\" ACTIVE"),
            Some(ScriptInfo {
                name: "vacation".into(),
                active: true
            })
        );
        assert_eq!(
            parse_script_line("\"backup\""),
            Some(ScriptInfo {
                name: "backup".into(),
                active: false
            })
        );
    }

    #[test]
    fn capability_line_split() {
        assert_eq!(
            split_cap_line("\"SASL\" \"PLAIN LOGIN\""),
            ("SASL".into(), "PLAIN LOGIN".into())
        );
        assert_eq!(
            split_cap_line("\"STARTTLS\""),
            ("STARTTLS".into(), String::new())
        );
    }

    #[test]
    fn capabilities_absorb() {
        let lines = vec![
            "\"IMPLEMENTATION\" \"Dovecot Pigeonhole\"".to_string(),
            "\"SIEVE\" \"fileinto vacation imap4flags\"".to_string(),
            "\"SASL\" \"plain login\"".to_string(),
            "\"STARTTLS\"".to_string(),
            "\"VERSION\" \"1.0\"".to_string(),
        ];
        let caps = parse_capabilities(&lines);
        assert_eq!(caps.implementation, "Dovecot Pigeonhole");
        assert!(caps.supports("fileinto"));
        assert!(caps.offers_sasl("PLAIN"));
        assert!(caps.offers_sasl("login"));
        assert!(caps.starttls);
        assert_eq!(caps.version, "1.0");
    }

    #[test]
    fn unescape_handles_backslash() {
        assert_eq!(unescape(r#"a\"b\\c"#), "a\"b\\c");
    }
}
