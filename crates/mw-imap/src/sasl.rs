//! SASL initial-response frame builders (base64-encoded) for the mechanisms
//! `mw-imap` uses: PLAIN (RFC 4616), LOGIN (two-step), and XOAUTH2 (Google/
//! Microsoft bearer-token, `user=…^Aauth=Bearer …^A^A`).
//!
//! These build only the client payloads; the continuation choreography lives in
//! [`crate::connection::Connection::authenticate`].

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;

/// SASL PLAIN: `base64( authzid "\0" authcid "\0" passwd )` with empty authzid.
pub fn plain(username: &str, password: &str) -> String {
    let mut raw = Vec::with_capacity(username.len() + password.len() + 2);
    raw.push(0);
    raw.extend_from_slice(username.as_bytes());
    raw.push(0);
    raw.extend_from_slice(password.as_bytes());
    B64.encode(raw)
}

/// SASL LOGIN: the two successive base64 payloads (username, then password).
pub fn login(username: &str, password: &str) -> [String; 2] {
    [
        B64.encode(username.as_bytes()),
        B64.encode(password.as_bytes()),
    ]
}

/// SASL XOAUTH2: `base64( "user=" user "\x01auth=Bearer " token "\x01\x01" )`.
pub fn xoauth2(username: &str, access_token: &str) -> String {
    let raw = format!("user={username}\x01auth=Bearer {access_token}\x01\x01");
    B64.encode(raw.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_matches_rfc4616_vector() {
        // RFC 4616 §5: authcid "tim", passwd "tanstaaftanstaaf".
        assert_eq!(
            plain("tim", "tanstaaftanstaaf"),
            "AHRpbQB0YW5zdGFhZnRhbnN0YWFm"
        );
    }

    #[test]
    fn login_encodes_each_step() {
        let [u, p] = login("Aladdin", "open sesame");
        assert_eq!(u, "QWxhZGRpbg==");
        assert_eq!(p, "b3BlbiBzZXNhbWU=");
    }

    #[test]
    fn xoauth2_matches_google_documented_vector() {
        // From Google's IMAP XOAUTH2 documentation.
        let frame = xoauth2(
            "someuser@example.com",
            "ya29.vF9dft4qmTc2Nvb3RlckBhdHRhdmlzdGEuY29tCg",
        );
        assert_eq!(
            frame,
            "dXNlcj1zb21ldXNlckBleGFtcGxlLmNvbQFhdXRoPUJlYXJlciB5YTI5LnZGOWRmdDRxbVRjMk52YjNSbGNrQmhkSFJoZG1semRHRXVZMjl0Q2cBAQ=="
        );
    }

    #[test]
    fn xoauth2_frame_decodes_to_control_delimited_payload() {
        let frame = xoauth2("u@x", "TOK");
        let decoded = B64.decode(frame).unwrap();
        assert_eq!(decoded, b"user=u@x\x01auth=Bearer TOK\x01\x01");
    }
}
