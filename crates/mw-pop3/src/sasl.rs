//! SASL initial-response encoders for POP3 `AUTH` (RFC 5034 + mechanism specs).
//!
//! Pure `String`-producing helpers so the exact wire bytes can be asserted in
//! tests. The transport ([`crate::conn`]) frames them into `AUTH` exchanges.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;

/// SASL `PLAIN` (RFC 4616): base64 of `\0authcid\0passwd` (authzid empty).
pub fn plain(username: &str, password: &str) -> String {
    let mut buf = Vec::with_capacity(username.len() + password.len() + 2);
    buf.push(0);
    buf.extend_from_slice(username.as_bytes());
    buf.push(0);
    buf.extend_from_slice(password.as_bytes());
    B64.encode(buf)
}

/// SASL `LOGIN` (non-standard but ubiquitous): the two base64 continuation
/// responses `(username, password)` the server prompts for in order.
pub fn login(username: &str, password: &str) -> (String, String) {
    (B64.encode(username), B64.encode(password))
}

/// SASL `XOAUTH2` (Google/Microsoft): base64 of
/// `user=<addr>^Aauth=Bearer <token>^A^A` where `^A` is 0x01.
pub fn xoauth2(username: &str, access_token: &str) -> String {
    let payload = format!("user={username}\x01auth=Bearer {access_token}\x01\x01");
    B64.encode(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_frame() {
        // RFC 4616 example shape: \0tim\0tanstaaftanstaaf
        let enc = plain("tim", "tanstaaftanstaaf");
        let decoded = B64.decode(enc).unwrap();
        assert_eq!(decoded, b"\0tim\0tanstaaftanstaaf");
    }

    #[test]
    fn login_frames() {
        let (u, p) = login("alice", "secret");
        assert_eq!(B64.decode(u).unwrap(), b"alice");
        assert_eq!(B64.decode(p).unwrap(), b"secret");
    }

    #[test]
    fn xoauth2_frame() {
        let enc = xoauth2("user@example.com", "tok123");
        let decoded = B64.decode(enc).unwrap();
        assert_eq!(
            decoded,
            b"user=user@example.com\x01auth=Bearer tok123\x01\x01"
        );
    }
}
