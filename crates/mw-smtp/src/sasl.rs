//! SASL credential encoders (plan §3 e4, SPEC §6.1).
//!
//! Deliberately tiny and self-contained: the plan blesses duplicating this
//! handful of lines rather than standing up a shared SASL crate for V1. The
//! same three mechanisms (`PLAIN`, `LOGIN`, `XOAUTH2`) are used by `mw-imap`;
//! the *encoding* is identical, only the command framing differs per protocol.

use base64::prelude::*;

/// Base64 of an arbitrary UTF-8 token (used for the `LOGIN` username/password
/// steps, which are each sent as a standalone base64 line after a `334`).
pub(crate) fn b64(s: &str) -> String {
    BASE64_STANDARD.encode(s.as_bytes())
}

/// `PLAIN` initial response (RFC 4616): base64 of `\0authcid\0passwd`.
///
/// There is no authorization identity, so the first field is empty.
pub(crate) fn plain(user: &str, pass: &str) -> String {
    // NUL-separated; never logged (credentials).
    BASE64_STANDARD.encode(format!("\0{user}\0{pass}"))
}

/// `XOAUTH2` initial response (Google/Microsoft): base64 of
/// `user=<user>^Aauth=Bearer <token>^A^A`, where `^A` is a single 0x01 byte.
pub(crate) fn xoauth2(user: &str, token: &str) -> String {
    BASE64_STANDARD.encode(format!("user={user}\x01auth=Bearer {token}\x01\x01"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_frame_is_nul_separated() {
        let ir = plain("alice@example.com", "s3cret");
        let decoded = BASE64_STANDARD.decode(ir).unwrap();
        assert_eq!(decoded, b"\0alice@example.com\0s3cret");
    }

    #[test]
    fn login_steps_are_plain_base64() {
        assert_eq!(
            BASE64_STANDARD.decode(b64("alice@example.com")).unwrap(),
            b"alice@example.com"
        );
        assert_eq!(BASE64_STANDARD.decode(b64("s3cret")).unwrap(), b"s3cret");
    }

    #[test]
    fn xoauth2_frame_uses_ctrl_a_delimiters() {
        let ir = xoauth2("alice@example.com", "ya29.TOKEN");
        let decoded = BASE64_STANDARD.decode(ir).unwrap();
        assert_eq!(
            decoded,
            b"user=alice@example.com\x01auth=Bearer ya29.TOKEN\x01\x01"
        );
    }
}
