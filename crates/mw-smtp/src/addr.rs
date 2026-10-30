//! Envelope-address validation.
//!
//! `MAIL FROM:<…>` and `RCPT TO:<…>` are built by string interpolation
//! (`conn.rs`), so whatever is accepted here is written to the SMTP control
//! channel byte for byte. These checks reject; they never repair. An address
//! that fails is not sent in a trimmed or escaped form.
//!
//! RFC 5321/6531 paths are validated before interpolation. Domain validation
//! uses IDNA only to check DNS labels; supplied mailbox bytes are preserved.

use crate::SmtpError;
const MAX_ADDR_BYTES: usize = 254; // 256-octet SMTP path, including < and >.
const ECHO_CHARS: usize = 48;

pub fn validate_mailbox(addr: &str) -> Result<(), SmtpError> {
    mailbox(addr).map_err(|why| refuse(addr, why))
}

fn mailbox(addr: &str) -> Result<(), &'static str> {
    if addr.len() > MAX_ADDR_BYTES {
        return Err("longer than 254 bytes");
    }
    if addr.chars().any(char::is_control) {
        return Err("contains a control character");
    }
    let (local, domain) = addr.rsplit_once('@').ok_or("no @")?;
    validate_local(local)?;
    if domain.is_empty() || domain.chars().any(char::is_whitespace) {
        return Err("invalid domain");
    }
    if let Some(literal) = domain.strip_prefix('[').and_then(|d| d.strip_suffix(']')) {
        let valid = if let Some(ip) = literal.strip_prefix("IPv6:") {
            ip.parse::<std::net::Ipv6Addr>().is_ok()
        } else {
            literal.parse::<std::net::Ipv4Addr>().is_ok()
        };
        return if valid {
            Ok(())
        } else {
            Err("invalid address literal")
        };
    }
    // URL host parsing also accepts IPv4 shorthand and percent escaping. SMTP
    // DNS names do not; check source label syntax before IDNA validation.
    if domain
        .chars()
        .any(|c| c.is_ascii() && !(c.is_ascii_alphanumeric() || c == '-' || c == '.'))
    {
        return Err("invalid domain character");
    }
    let host = url::Host::parse(domain).map_err(|_| "invalid IDNA domain")?;
    let ascii = host.to_string();
    if ascii.len() > 253
        || ascii.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || label.starts_with('-')
                || label.ends_with('-')
                || !label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
    {
        return Err("invalid DNS label");
    }
    // IDNA must not turn an invalid empty source label into a valid one.
    if domain.split('.').any(str::is_empty) {
        return Err("empty DNS label");
    }
    Ok(())
}

fn validate_local(local: &str) -> Result<(), &'static str> {
    if local.is_empty() || local.len() > 64 {
        return Err("local part must be 1..64 octets");
    }
    if let Some(inner) = local.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        let mut chars = inner.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                let escaped = chars.next().ok_or("incomplete quoted escape")?;
                if !((' '..='~').contains(&escaped)) {
                    return Err("invalid quoted escape");
                }
            } else if c == '"' || c.is_control() || (c.is_whitespace() && c != ' ') {
                return Err("invalid quoted local part");
            }
        }
        return Ok(());
    }
    if local.split('.').any(str::is_empty) {
        return Err("empty local atom");
    }
    if local.chars().any(|c| {
        if c.is_ascii() {
            !(c.is_ascii_alphanumeric() || "!#$%&'*+-/=?^_`{|}~.".contains(c))
        } else {
            c.is_control() || c.is_whitespace()
        }
    }) {
        return Err("invalid local atom");
    }
    Ok(())
}

/// Null senders and legacy bare login names remain supported. Fully qualified
/// senders use the same syntax and limits as recipients.
pub fn validate_reverse_path(addr: &str) -> Result<(), SmtpError> {
    if addr.is_empty() {
        return Ok(());
    }
    if addr.contains('@') {
        return validate_mailbox(addr);
    }
    validate_local(addr).map_err(|why| refuse(addr, why))
}

/// Build the error for a refused value. The value is debug-escaped and
/// truncated, so the message itself carries no raw control character.
fn refuse(addr: &str, why: &str) -> SmtpError {
    let head: String = addr.chars().take(ECHO_CHARS).collect();
    let more = if head.len() < addr.len() { "…" } else { "" };
    SmtpError::InvalidAddress(format!("{head:?}{more}: {why}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refused(r: Result<(), SmtpError>) -> String {
        match r {
            Err(SmtpError::InvalidAddress(m)) => m,
            other => panic!("expected InvalidAddress, got {other:?}"),
        }
    }

    #[test]
    fn international_octet_limits_and_dns_syntax_are_enforced() {
        validate_mailbox(&format!("{}@example.org", "é".repeat(32))).unwrap();
        for bad in [
            format!("{}@example.org", "é".repeat(33)),
            "a..b@example.org".into(),
            ".a@example.org".into(),
            "a.@example.org".into(),
            "a@-example.org".into(),
            "a@example-.org".into(),
            "a@example.org.".into(),
            "a@exa%mple.org".into(),
            "a@[127.1]".into(),
            "a@[IPv6:not-an-ip]".into(),
            "a@exam_ple.org".into(),
            "a@a..b".into(),
            format!("a@{}.org", "a".repeat(64)),
            format!(
                "{}@{}.{}.{}",
                "a".repeat(64),
                "b".repeat(63),
                "c".repeat(63),
                "d".repeat(63)
            ),
        ] {
            assert!(validate_mailbox(&bad).is_err(), "accepted {bad:?}");
        }
        for quoted in [
            "\"\"@example.org",
            "\"a<> b,c@d\"@example.org",
            "\"é\\\"x\\\\\"@example.org",
        ] {
            validate_mailbox(quoted).unwrap();
        }
    }

    #[test]
    fn ordinary_mailboxes_pass() {
        for ok in [
            "a@b",
            "bob@example.test",
            "first.last+tag@sub.example.test",
            "møt@example.com",
            "user@[192.0.2.1]",
            "o'brien@example.test",
            "\"a b@c\"@example.test",
            "\"jörg\"@bücher.example",
            "a@[IPv6:2001:db8::1]",
        ] {
            validate_mailbox(ok).unwrap_or_else(|e| panic!("{ok}: {e}"));
            validate_reverse_path(ok).unwrap_or_else(|e| panic!("{ok}: {e}"));
        }
    }

    #[test]
    fn injection_shapes_are_refused_on_both_paths() {
        for bad in [
            "x@example.test>\r\nRCPT TO:<victim@example.test",
            "x@example.test\nDATA",
            "x@example.test\r",
            "x@example.\0test",
            "x@example.test\u{7f}",
            "x@example.test\u{85}",
            "x@example.test\u{2028}",
            "x @example.test",
            "x@example.test ",
            "\tx@example.test",
            "a@b>",
            "<a@b",
            "a@b> SIZE=1",
            "a\\@b@example.test",
            "a@b,c@d",
            "a@b;c@d",
            "a(comment)@b",
            "a@b@c",
        ] {
            refused(validate_mailbox(bad));
            refused(validate_reverse_path(bad));
        }
    }

    #[test]
    fn a_mailbox_needs_both_sides_of_one_at() {
        for bad in ["", "bob", "@example.test", "bob@", "@"] {
            refused(validate_mailbox(bad));
        }
        refused(validate_mailbox(&format!(
            "{}@example.test",
            "a".repeat(320)
        )));
    }

    #[test]
    fn the_reverse_path_may_be_null_or_a_bare_login_name() {
        validate_reverse_path("").unwrap();
        validate_reverse_path("alice").unwrap();
        // …but a bare name is still held to the character rules,
        refused(validate_reverse_path("alice\r\nRSET"));
        refused(validate_reverse_path("alice> SIZE=1"));
        // and a value with an `@` has to be a whole mailbox.
        refused(validate_reverse_path("alice@"));
        refused(validate_reverse_path("@example.test"));
    }

    #[test]
    fn the_error_text_carries_no_raw_control_character_and_is_bounded() {
        let m = refused(validate_mailbox(
            "x@example.test>\r\nRCPT TO:<v@example.test",
        ));
        assert!(!m.chars().any(char::is_control), "{m:?}");
        assert!(m.contains("\\r\\n"), "{m}");

        let long = format!("{}\r\n@example.test", "a".repeat(400));
        let m = refused(validate_mailbox(&long));
        assert!(m.len() < 120, "bounded echo, got {} bytes", m.len());
    }
}
