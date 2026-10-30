//! Checks on values that are written into a header or a report field.
//!
//! `mail-builder` writes several values byte for byte (an address between `<`
//! and `>`, a message id, a content type, a raw header, and an unencoded
//! subject), and the MDN fields in [`crate::mdn`] are assembled by this crate.
//! A CR, LF or NUL in any of them would start a new header line, so the checks
//! here run in this crate whatever the caller has already done. They reject;
//! the two `*_text` helpers, used only for free text, replace instead.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::MimeError;

/// RFC 5321 path maximum excludes the surrounding angle brackets.
const MAX_ADDR_BYTES: usize = 254;

/// Longest `Content-ID` accepted (without the angle brackets).
const MAX_CID_BYTES: usize = 255;

/// How much of a refused value is echoed (debug-escaped) in an error.
const ECHO_CHARS: usize = 48;

/// Check one mailbox (`local@domain`) before it is written into a header or an
/// MDN field.
///
/// SMTPUTF8 dot-atoms and quoted local parts use the same byte-preserving
/// grammar as the SMTP envelope validator.
pub fn validate_mailbox(addr: &str) -> Result<(), MimeError> {
    mailbox_problem(addr).map_or(Ok(()), |why| Err(refuse("address", addr, why)))
}

/// Why `addr` is not an acceptable mailbox, or `None` when it is.
pub(crate) fn mailbox_problem(addr: &str) -> Option<&'static str> {
    mailbox(addr).err()
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

/// Decode RFC 6533 unitext only when it yields a complete safe mailbox.
/// Malformed address claims remain verbatim, as RFC 6533 requires; decoded
/// controls can never be introduced into a report by this operation.
pub(crate) fn decode_utf8_address(value: &str) -> String {
    // Native UTF-8 address syntax has precedence. In particular a quoted
    // local part may legitimately contain the literal bytes `\x{...}`.
    if mailbox_problem(value).is_none() {
        return value.to_owned();
    }
    let literal = |text: &str| {
        text.chars()
            .all(|c| !c.is_ascii() || (('!'..='~').contains(&c) && !matches!(c, '\\' | '+' | '=')))
    };
    let mut output = String::new();
    let mut rest = value;
    while let Some(at) = rest.find("\\x{") {
        if !literal(&rest[..at]) {
            return value.to_owned();
        }
        output.push_str(&rest[..at]);
        let escaped = &rest[at + 3..];
        let Some(end) = escaped.find('}') else {
            return value.to_owned();
        };
        let hex = &escaped[..end];
        if !(2..=6).contains(&hex.len())
            || !hex.bytes().all(|b| b.is_ascii_hexdigit())
            || (hex.len() > 2 && hex.starts_with('0'))
        {
            return value.to_owned();
        }
        let Some(c) = u32::from_str_radix(hex, 16).ok().and_then(char::from_u32) else {
            return value.to_owned();
        };
        if (c.is_ascii() && !matches!(c, ' ' | '+' | '=' | '\\'))
            || c.is_control()
            || matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        {
            return value.to_owned();
        }
        output.push(c);
        rest = &escaped[end + 1..];
    }
    if !literal(rest) {
        return value.to_owned();
    }
    output.push_str(rest);
    if mailbox_problem(&output).is_none() {
        output
    } else {
        value.to_owned()
    }
}

/// Check a `Content-ID` (given without angle brackets) before it is written as
/// `Content-ID: <cid>`.
///
/// Accepted: 1 to 255 bytes of RFC 5322 `atext`, `.` and `@`, with at most one
/// `@`. That admits the ids other mail programs produce (`image001.png@01D…`,
/// `ii_abc123`) as well as [`generate_content_id`]'s. Everything else is
/// refused, which covers control characters, whitespace, `<`, `>`, quotes and
/// non-ASCII.
pub fn validate_content_id(cid: &str) -> Result<(), MimeError> {
    let why = if cid.is_empty() {
        "empty"
    } else if cid.len() > MAX_CID_BYTES {
        "longer than 255 bytes"
    } else if !cid.bytes().all(is_cid_byte) {
        "contains a character outside atext, '.' and '@'"
    } else if cid.bytes().filter(|&b| b == b'@').count() > 1 {
        "more than one @"
    } else {
        return Ok(());
    };
    Err(refuse("content id", cid, why))
}

/// A new `Content-ID` (without angle brackets) for an inline part:
/// 32 hexadecimal digits, `@`, and the reserved name `inline.invalid`.
///
/// The digits come from [`random_token`]; see there for what they are and are
/// not.
#[must_use]
pub fn generate_content_id() -> String {
    format!("{}@inline.invalid", random_token())
}

/// RFC 5322 `atext`, plus `.` and `@`.
fn is_cid_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b".@!#$%&'*+-/=?^_`{|}~".contains(&b)
}

/// 32 hexadecimal digits that differ on every call and that a correspondent
/// cannot predict from other messages.
///
/// They are two outputs of the standard library's SipHash under
/// [`RandomState`] keys, which the standard library seeds from the operating
/// system once per thread and changes on every `RandomState::new()`. This
/// crate has no random-number dependency; the value is used for MIME
/// boundaries and content ids, where it has to be unique and not guessable by
/// whoever wrote the text placed between the boundaries. It is not key
/// material.
pub(crate) fn random_token() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    let half = || {
        let mut h = RandomState::new().build_hasher();
        h.write_u128(nanos);
        h.write_u64(count);
        h.finish()
    };
    format!("{:016x}{:016x}", half(), half())
}

/// Whether `s` holds a control character (CR, LF, NUL, TAB, DEL, C1 …).
pub(crate) fn has_control(s: &str) -> bool {
    s.chars().any(char::is_control)
}

/// Refuse `value` when it holds a control character.
pub(crate) fn no_control(what: &str, value: &str) -> Result<(), MimeError> {
    if has_control(value) {
        return Err(refuse(what, value, "contains a control character"));
    }
    Ok(())
}

/// Free text for one header line: every control character and every Unicode
/// line or paragraph separator becomes a space, so the result cannot end the
/// line it is written on.
pub(crate) fn line_text(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}') {
                ' '
            } else {
                c
            }
        })
        .collect()
}

/// A file name for a `filename` parameter: control characters are dropped.
pub(crate) fn file_name_text(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).collect()
}

/// Build the error for a refused value. The value is debug-escaped and
/// truncated, so the message itself carries no raw control character.
pub(crate) fn refuse(what: &str, value: &str, why: &str) -> MimeError {
    let head: String = value.chars().take(ECHO_CHARS).collect();
    let more = if head.len() < value.len() { "…" } else { "" };
    MimeError::Build(format!("{what} {head:?}{more}: {why}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // The three vector lists below are the ones in `mw-smtp/src/addr.rs`.

    #[test]
    fn unitext_recipient_decoding_is_bounded_and_never_introduces_controls() {
        let native = r#""é\\x{E9}"@example.org"#;
        assert_eq!(decode_utf8_address(native), native);
        let mixed = r"j\x{F6}rg+tag@example.org";
        assert_eq!(decode_utf8_address(mixed), mixed);
        assert_eq!(
            decode_utf8_address(r"j\x{F6}rg\x{2B}tag@example.org"),
            "jörg+tag@example.org"
        );
        for raw in [
            r"a\x{0A}@example.org",
            r"a\x{D800}@example.org",
            r"a\x{202E}@example.org",
            r"a\x{110000}@example.org",
            r"a\x{GG}@example.org",
            r"a\x{41}@example.org",
            r"a\x{00E9}@example.org",
        ] {
            assert_eq!(decode_utf8_address(raw), raw);
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
        ] {
            validate_mailbox(ok).unwrap_or_else(|e| panic!("{ok}: {e}"));
        }
    }

    #[test]
    fn injection_shapes_are_refused() {
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
            assert!(validate_mailbox(bad).is_err(), "{bad:?} was accepted");
        }
    }

    #[test]
    fn a_mailbox_needs_both_sides_of_one_at() {
        for bad in ["", "bob", "@example.test", "bob@", "@"] {
            assert!(validate_mailbox(bad).is_err(), "{bad:?} was accepted");
        }
        let long = format!("{}@example.test", "a".repeat(320));
        assert!(validate_mailbox(&long).is_err());
    }

    #[test]
    fn the_error_text_carries_no_raw_control_character_and_is_bounded() {
        let m = validate_mailbox("x@example.test>\r\nRCPT TO:<v@example.test")
            .unwrap_err()
            .to_string();
        assert!(!m.chars().any(char::is_control), "{m:?}");
        assert!(m.contains("\\r\\n"), "{m}");

        let long = format!("{}\r\n@example.test", "a".repeat(400));
        let m = validate_mailbox(&long).unwrap_err().to_string();
        assert!(m.len() < 140, "bounded echo, got {} bytes", m.len());
    }

    #[test]
    fn content_ids_from_other_mail_programs_pass() {
        for ok in [
            "image001.png@01D9ABCD.12345678",
            "ii_lxyz0abc1",
            "part1.06090408.01060107@example.com",
            "a",
        ] {
            validate_content_id(ok).unwrap_or_else(|e| panic!("{ok}: {e}"));
        }
    }

    #[test]
    fn hostile_content_ids_are_refused() {
        for bad in [
            "",
            "a\r\nBcc: v@example.test",
            "a\nb",
            "a\0b",
            "a b",
            "a\tb",
            "<a@b>",
            "a>b",
            "a\"b",
            "a@b@c",
            "é@example.test",
            "a\u{2028}b",
        ] {
            assert!(validate_content_id(bad).is_err(), "{bad:?} was accepted");
        }
        assert!(validate_content_id(&"a".repeat(256)).is_err());
        validate_content_id(&"a".repeat(255)).expect("255 bytes is the limit");
    }

    #[test]
    fn generated_content_ids_are_valid_and_do_not_repeat() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..2000 {
            let cid = generate_content_id();
            validate_content_id(&cid).unwrap_or_else(|e| panic!("{cid}: {e}"));
            let (left, right) = cid.split_once('@').expect("addr-spec form");
            assert_eq!(left.len(), 32, "{cid}");
            assert!(left.bytes().all(|b| b.is_ascii_hexdigit()), "{cid}");
            assert_eq!(right, "inline.invalid");
            assert!(seen.insert(cid), "a content id repeated");
        }
    }

    #[test]
    fn line_text_leaves_nothing_that_ends_a_line() {
        let out = line_text("a\r\nBcc: v@example.test\0\u{85}\u{2028}b\tc");
        assert!(!has_control(&out), "{out:?}");
        assert!(!out.contains('\u{2028}'));
        assert_eq!(out, "a  Bcc: v@example.test   b c");
        assert_eq!(file_name_text("a\r\n\0b.png"), "ab.png");
    }
}
