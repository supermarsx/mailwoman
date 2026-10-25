//! SASL mechanisms for the ManageSieve client: PLAIN (RFC 4616), LOGIN,
//! XOAUTH2 and OAUTHBEARER (RFC 7628).
//!
//! The same mechanisms live in `mw-imap` and `mw-pop3`, but `mw-sieve` cannot
//! depend on either (both depend on `mw-engine`, which depends on this crate),
//! so this is a local copy of the client halves, as the POP3 one is.

/// PLAIN: `\0authcid\0passwd` (authzid empty).
pub fn plain(username: &str, password: &str) -> Vec<u8> {
    let mut raw = Vec::with_capacity(username.len() + password.len() + 2);
    raw.push(0);
    raw.extend_from_slice(username.as_bytes());
    raw.push(0);
    raw.extend_from_slice(password.as_bytes());
    raw
}

/// XOAUTH2: `user=<user>^Aauth=Bearer <token>^A^A`.
pub fn xoauth2(username: &str, token: &str) -> Vec<u8> {
    format!("user={username}\x01auth=Bearer {token}\x01\x01").into_bytes()
}

/// OAUTHBEARER (RFC 7628 §3.1): `n,a=<user>,^Aauth=Bearer <token>^A^A`. The
/// optional `host`/`port` pairs are omitted.
pub fn oauthbearer(username: &str, token: &str) -> Vec<u8> {
    format!(
        "n,a={},\x01auth=Bearer {token}\x01\x01",
        saslname_escape(username)
    )
    .into_bytes()
}

/// RFC 5801 §4 saslname: `=` → `=3D`, `,` → `=2C`.
fn saslname_escape(s: &str) -> String {
    s.replace('=', "=3D").replace(',', "=2C")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames() {
        assert_eq!(plain("tim", "tan"), b"\0tim\0tan");
        assert_eq!(xoauth2("u@x", "t"), b"user=u@x\x01auth=Bearer t\x01\x01");
        assert_eq!(
            oauthbearer("u,=x", "t"),
            b"n,a=u=2C=3Dx,\x01auth=Bearer t\x01\x01"
        );
    }
}
