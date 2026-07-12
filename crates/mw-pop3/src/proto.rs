//! Pure POP3 wire parsing (RFC 1939 + CAPA/RFC 2449).
//!
//! Everything here is I/O-free and total: given arbitrary bytes it returns a
//! `Result`/`Option` and never panics. That is what makes the module the
//! `cargo-fuzz` surface ([`fuzz_response_lines`]) — the transport layer
//! ([`crate::conn`]) reads framed lines off the socket and hands them here.

use mw_engine::backend::EngineError;

/// A parsed POP3 status indicator (RFC 1939 §3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// `+OK` with the trailing human text (may be empty).
    Ok(String),
    /// `-ERR` with the trailing human text (may be empty).
    Err(String),
}

impl Status {
    /// The trailing message text, regardless of polarity.
    pub fn message(&self) -> &str {
        match self {
            Status::Ok(m) | Status::Err(m) => m,
        }
    }
}

/// Capabilities advertised by a `CAPA` response (RFC 2449 §3).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapaInfo {
    /// SASL mechanisms from the `SASL` capability line, upper-cased.
    pub sasl: Vec<String>,
    /// `STLS` advertised (RFC 2595) — opportunistic TLS is available.
    pub stls: bool,
    /// `TOP` advertised.
    pub top: bool,
    /// `UIDL` advertised.
    pub uidl: bool,
    /// `PIPELINING` advertised.
    pub pipelining: bool,
}

impl CapaInfo {
    fn sasl_has(&self, mech: &str) -> bool {
        self.sasl.iter().any(|m| m == mech)
    }

    /// SASL PLAIN offered.
    pub fn sasl_plain(&self) -> bool {
        self.sasl_has("PLAIN")
    }

    /// SASL LOGIN offered.
    pub fn sasl_login(&self) -> bool {
        self.sasl_has("LOGIN")
    }

    /// SASL XOAUTH2 offered.
    pub fn sasl_xoauth2(&self) -> bool {
        self.sasl_has("XOAUTH2")
    }
}

fn protocol(msg: impl Into<String>) -> EngineError {
    EngineError::Protocol(msg.into())
}

/// Strip a single trailing `\r\n` / `\n` from a raw line.
pub fn trim_eol(line: &[u8]) -> &[u8] {
    let mut end = line.len();
    if end > 0 && line[end - 1] == b'\n' {
        end -= 1;
    }
    if end > 0 && line[end - 1] == b'\r' {
        end -= 1;
    }
    &line[..end]
}

/// Parse a status line into `+OK` / `-ERR` and its trailing text.
///
/// Non-UTF-8 bytes in the human-readable tail are replaced lossily; the
/// indicator itself must be exactly `+OK` or `-ERR` (RFC 1939 §3).
pub fn parse_status(line: &[u8]) -> Result<Status, EngineError> {
    let line = trim_eol(line);
    if let Some(rest) = line.strip_prefix(b"+OK") {
        Ok(Status::Ok(status_tail(rest)))
    } else if let Some(rest) = line.strip_prefix(b"-ERR") {
        Ok(Status::Err(status_tail(rest)))
    } else {
        Err(protocol(format!(
            "expected +OK/-ERR status, got {:?}",
            String::from_utf8_lossy(&line[..line.len().min(40)])
        )))
    }
}

fn status_tail(rest: &[u8]) -> String {
    let rest = rest.strip_prefix(b" ").unwrap_or(rest);
    String::from_utf8_lossy(rest).into_owned()
}

/// Parse the count/size pair from a `+OK <count> <size>` `STAT` reply tail.
pub fn parse_stat(tail: &str) -> Result<(u64, u64), EngineError> {
    let mut it = tail.split_ascii_whitespace();
    let count = it
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| protocol("STAT missing message count"))?;
    let size = it
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| protocol("STAT missing octet total"))?;
    Ok((count, size))
}

/// Parse one `LIST`/`UIDL` body line: `<msg-number> <value>` (RFC 1939 §7/§11).
///
/// Returns `None` for blank lines so callers can skip them without erroring.
pub fn parse_listing_line(line: &str) -> Option<(u32, String)> {
    let line = line.trim_end_matches(['\r', '\n']);
    if line.is_empty() {
        return None;
    }
    let mut it = line.splitn(2, ' ');
    let num = it.next()?.parse::<u32>().ok()?;
    let value = it.next()?.trim();
    if value.is_empty() {
        return None;
    }
    Some((num, value.to_string()))
}

/// Parse a full multi-line `UIDL` body into `(msg-number, uidl)` pairs.
pub fn parse_uidl_body(body: &[u8]) -> Vec<(u32, String)> {
    parse_listing_body(body)
}

/// Parse a full multi-line `LIST` body into `(msg-number, octet-size)` pairs.
pub fn parse_list_body(body: &[u8]) -> Vec<(u32, String)> {
    parse_listing_body(body)
}

fn parse_listing_body(body: &[u8]) -> Vec<(u32, String)> {
    String::from_utf8_lossy(body)
        .lines()
        .filter_map(parse_listing_line)
        .collect()
}

/// Parse a `CAPA` multi-line body (RFC 2449 §3).
pub fn parse_capa(body: &[u8]) -> CapaInfo {
    let text = String::from_utf8_lossy(body);
    let mut info = CapaInfo::default();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_ascii_whitespace();
        let Some(name) = parts.next() else { continue };
        match name.to_ascii_uppercase().as_str() {
            "SASL" => {
                info.sasl = parts.map(|m| m.to_ascii_uppercase()).collect();
            }
            "STLS" => info.stls = true,
            "TOP" => info.top = true,
            "UIDL" => info.uidl = true,
            "PIPELINING" => info.pipelining = true,
            _ => {}
        }
    }
    info
}

/// Reverse RFC 1939 §3 byte-stuffing on a collected multi-line body.
///
/// The caller passes the body *without* the terminating `.` line; every line
/// that began with `.` on the wire had a second `.` prepended, which this
/// removes. CRLF framing between lines is preserved so `RETR` yields faithful
/// RFC822 bytes.
pub fn dot_unstuff(body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len());
    let mut at_line_start = true;
    let mut i = 0;
    while i < body.len() {
        let b = body[i];
        if at_line_start && b == b'.' {
            // Drop exactly one stuffing dot; keep any following bytes verbatim.
            i += 1;
            at_line_start = false;
            continue;
        }
        out.push(b);
        at_line_start = b == b'\n';
        i += 1;
    }
    out
}

/// Fuzz + smoke entry point: run every parser over untrusted server bytes.
///
/// Exercised both by the `cargo-fuzz` target (`fuzz/fuzz_targets/response.rs`)
/// and by an in-crate smoke test, so the parsing surface is checked for panics
/// even where `cargo-fuzz` (nightly/Linux) is unavailable.
pub fn fuzz_response_lines(data: &[u8]) {
    // Treat the first line as a status line, the remainder as a multi-line body.
    let (first, rest) = match data.iter().position(|&b| b == b'\n') {
        Some(idx) => (&data[..idx], &data[idx + 1..]),
        None => (data, &b""[..]),
    };
    let _ = parse_status(first);
    if let Ok(Status::Ok(tail)) = parse_status(first) {
        let _ = parse_stat(&tail);
    }
    let _ = parse_capa(rest);
    let _ = parse_uidl_body(rest);
    let _ = parse_list_body(rest);
    let _ = dot_unstuff(rest);
    let _ = dot_unstuff(data);
    for line in String::from_utf8_lossy(rest).lines() {
        let _ = parse_listing_line(line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_ok_and_err() {
        assert_eq!(
            parse_status(b"+OK message follows\r\n").unwrap(),
            Status::Ok("message follows".into())
        );
        assert_eq!(
            parse_status(b"-ERR no such message\r\n").unwrap(),
            Status::Err("no such message".into())
        );
        assert_eq!(parse_status(b"+OK\r\n").unwrap(), Status::Ok(String::new()));
        assert!(parse_status(b"garbage\r\n").is_err());
    }

    #[test]
    fn stat_pair() {
        assert_eq!(parse_stat("2 320").unwrap(), (2, 320));
        assert!(parse_stat("2").is_err());
        assert!(parse_stat("x y").is_err());
    }

    #[test]
    fn capa_parse() {
        let body = b"TOP\r\nUIDL\r\nPIPELINING\r\nSTLS\r\nSASL PLAIN LOGIN XOAUTH2\r\n";
        let caps = parse_capa(body);
        assert!(caps.top && caps.uidl && caps.pipelining && caps.stls);
        assert!(caps.sasl_plain() && caps.sasl_login() && caps.sasl_xoauth2());
        assert!(!parse_capa(b"TOP\r\n").sasl_plain());
    }

    #[test]
    fn uidl_body() {
        let body = b"1 whqtswO00WBw418f9t5JxYwZ\r\n2 QhdPYR:00WBw1Ph7x7\r\n\r\n";
        let pairs = parse_uidl_body(body);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], (1, "whqtswO00WBw418f9t5JxYwZ".into()));
        assert_eq!(pairs[1].0, 2);
    }

    #[test]
    fn unstuff_removes_leading_dot() {
        // A body line ".." on the wire is a single "." in the message.
        let wire = b"Line one\r\n..hidden\r\nLine three\r\n";
        assert_eq!(dot_unstuff(wire), b"Line one\r\n.hidden\r\nLine three\r\n");
        // A real leading dot in headers like ".signature" survives one strip.
        assert_eq!(dot_unstuff(b".foo\r\n"), b"foo\r\n");
    }

    #[test]
    fn fuzz_entry_never_panics_on_junk() {
        for sample in [
            &b""[..],
            &b"\n"[..],
            &b"+OK 1 2\r\n1 a\r\n.\r\n"[..],
            &b"-ERR\r\n\xff\xfe\x00"[..],
            &b".\r\n..\r\n...\r\n"[..],
            &b"+OK\r\nSASL\r\n \r\n1\r\n1 \r\n"[..],
        ] {
            fuzz_response_lines(sample);
        }
    }
}
