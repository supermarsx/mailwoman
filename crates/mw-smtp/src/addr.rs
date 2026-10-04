//! Envelope-address validation.
//!
//! `MAIL FROM:<…>` and `RCPT TO:<…>` are built by string interpolation
//! (`conn.rs`), so whatever is accepted here is written to the SMTP control
//! channel byte for byte. These checks reject; they never repair. An address
//! that fails is not sent in a trimmed or escaped form.
//!
//! The accepted grammar is narrower than RFC 5321 on purpose: quoted local
//! parts (`"a b"@example.org`) and backslash escapes are refused, because the
//! characters they exist to carry are the ones that end a path or a command.

use crate::SmtpError;

/// RFC 5321 §4.5.3.1: 64-octet local part + `@` + 255-octet domain.
const MAX_ADDR_BYTES: usize = 320;

/// How much of a refused value is echoed (debug-escaped) in the error.
const ECHO_CHARS: usize = 48;

/// Check a forward-path mailbox (an `RCPT TO` recipient).
///
/// Refused: the empty string; more than 320 bytes; any control character
/// (CR, LF, NUL, DEL, C1 …); any whitespace; `<` or `>`; `"` or `\` (so quoted
/// local parts are not supported); `,` `;` `(` `)` (list separators and
/// comments in a header address list); and anything that is not exactly one
/// `@` with a non-empty local part before it and a non-empty domain after it.
/// Non-ASCII is allowed — `SMTPUTF8` is negotiated per message.
pub fn validate_mailbox(addr: &str) -> Result<(), SmtpError> {
    check_chars(addr).map_err(|why| refuse(addr, why))?;
    match addr.split_once('@') {
        Some((local, domain)) if !local.is_empty() && !domain.is_empty() => Ok(()),
        Some(_) => Err(refuse(addr, "empty local part or domain")),
        None => Err(refuse(addr, "no @")),
    }
}

/// Check a reverse-path (the `MAIL FROM` sender).
///
/// The empty string is the null reverse-path `<>` (RFC 5321 §4.5.5) and is
/// accepted. A value with no `@` is also accepted when it passes the character
/// rules of [`validate_mailbox`]: an account's sender is the login name it was
/// connected with, which on some servers is a bare user name, and those
/// servers qualify it themselves. A value containing `@` must be a
/// [`validate_mailbox`] mailbox.
pub fn validate_reverse_path(addr: &str) -> Result<(), SmtpError> {
    if addr.is_empty() {
        return Ok(());
    }
    if addr.contains('@') {
        return validate_mailbox(addr);
    }
    check_chars(addr).map_err(|why| refuse(addr, why))
}

/// The character-level rules shared by both paths. `@` is counted here so that
/// a second one is refused whichever path is being checked.
fn check_chars(addr: &str) -> Result<(), &'static str> {
    if addr.is_empty() {
        return Err("empty");
    }
    if addr.len() > MAX_ADDR_BYTES {
        return Err("longer than 320 bytes");
    }
    let mut ats = 0;
    for c in addr.chars() {
        if c.is_control() {
            return Err("contains a control character");
        }
        if c.is_whitespace() {
            return Err("contains whitespace");
        }
        match c {
            '<' | '>' => return Err("contains an angle bracket"),
            '"' | '\\' => return Err("quoted local parts are not supported"),
            ',' | ';' | '(' | ')' => return Err("contains a list separator or comment"),
            '@' => ats += 1,
            _ => {}
        }
    }
    if ats > 1 {
        return Err("more than one @");
    }
    Ok(())
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
            "\"a b\"@example.test",
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
