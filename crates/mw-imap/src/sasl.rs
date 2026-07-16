//! SASL mechanisms for `mw-imap`.
//!
//! Two shapes live here:
//!
//! - **Single-frame builders** — PLAIN (RFC 4616), LOGIN (two-step), XOAUTH2 and
//!   OAUTHBEARER (RFC 7628) bearer-token frames. These build only the client
//!   payloads; the continuation choreography lives in
//!   [`crate::connection::Connection::authenticate`].
//! - **[`SaslClient`] state machines** — the interactive challenge/response
//!   mechanisms, currently SCRAM-SHA-256 and SCRAM-SHA-256-PLUS (RFC 5802 /
//!   RFC 7677). [`crate::connection::Connection::authenticate_sasl`] drives one
//!   step per server continuation.
//!
//! ## PBKDF2 / channel binding
//! SCRAM's salted password uses PBKDF2-HMAC-SHA-256, derived here from the
//! in-tree `hmac` + `sha2` primitives (single output block, since `dkLen ==
//! hLen == 32`) — no separate `pbkdf2` dependency. The `-PLUS` channel binding
//! is `tls-server-end-point` = SHA-256 of the server's leaf certificate. Per
//! RFC 5929 the endpoint hash should track the certificate's own signature
//! digest; we use SHA-256, which is exact for SHA-256/-signed certificates (the
//! overwhelming majority) — SHA-384/-512-signed leaves are a known gap, so the
//! plain SCRAM-SHA-256 mechanism stays the always-interoperable default.
//!
//! SASLprep (RFC 4013) normalisation of the username/password is not applied;
//! this is a no-op for ASCII credentials (the common case and the RFC 7677
//! test vector).

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use base64::engine::general_purpose::STANDARD_NO_PAD as B64_NOPAD;
use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

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

/// SASL OAUTHBEARER (RFC 7628): the `gs2-header`-prefixed bearer frame
/// `base64( "n,a=" user ",\x01host=" host "\x01port=" port "\x01auth=Bearer "
/// token "\x01\x01" )`. `host`/`port` are advisory per the RFC; including them
/// matches the servers that validate them.
pub fn oauthbearer(username: &str, access_token: &str, host: &str, port: u16) -> String {
    let raw = format!(
        "n,a={username},\x01host={host}\x01port={port}\x01auth=Bearer {access_token}\x01\x01"
    );
    B64.encode(raw.as_bytes())
}

// --- interactive SASL: the challenge/response driver contract ---------------

/// A client-side SASL exchange driven one step per server continuation.
///
/// The driver ([`crate::connection::Connection::authenticate_sasl`]) calls
/// [`SaslClient::step`] with each decoded server challenge and writes the
/// base64 of the returned bytes back. The **first** call receives an empty
/// challenge (the server's initial bare `+` continuation), mirroring the
/// no-`SASL-IR` choreography the simple mechanisms already use.
pub trait SaslClient {
    /// Produce the next raw (un-encoded) client response for `challenge`.
    ///
    /// An `Err` message aborts the exchange (the driver sends `*` and surfaces
    /// an authentication error).
    fn step(&mut self, challenge: &[u8]) -> Result<Vec<u8>, String>;
}

// --- SCRAM-SHA-256 / -PLUS (RFC 5802, RFC 7677) -----------------------------

/// One HMAC-SHA-256 over `msg` keyed by `key`.
fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(msg);
    let mut out = [0u8; 32];
    out.copy_from_slice(&mac.finalize().into_bytes());
    out
}

/// One SHA-256 digest of `data`.
fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(data);
    let mut out = [0u8; 32];
    out.copy_from_slice(&h.finalize());
    out
}

/// PBKDF2-HMAC-SHA-256 for a single output block (`dkLen == hLen == 32`, so the
/// block index is always 1) — the shape SCRAM's `SaltedPassword` needs.
fn pbkdf2_hmac_sha256(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    let mut salted = salt.to_vec();
    salted.extend_from_slice(&1u32.to_be_bytes()); // INT(1)
    let mut u = hmac_sha256(password, &salted);
    let mut out = u;
    for _ in 1..iterations.max(1) {
        u = hmac_sha256(password, &u);
        for (o, x) in out.iter_mut().zip(u.iter()) {
            *o ^= *x;
        }
    }
    out
}

/// The `tls-server-end-point` channel binding (RFC 5929): SHA-256 of the
/// server's leaf certificate DER.
pub fn tls_server_end_point(leaf_cert_der: &[u8]) -> Vec<u8> {
    sha256(leaf_cert_der).to_vec()
}

/// Escape a SCRAM `username` (`,` → `=2C`, `=` → `=3D`; RFC 5802 §5.1).
fn escape_scram(username: &str) -> String {
    username.replace('=', "=3D").replace(',', "=2C")
}

#[derive(Debug, PartialEq, Eq)]
enum ScramState {
    Initial,
    ClientFirstSent,
    ClientFinalSent,
    Done,
}

/// A SCRAM-SHA-256 (RFC 7677) client. When constructed with channel-binding
/// data it speaks the `-PLUS` variant (`p=tls-server-end-point`); otherwise the
/// plain variant (`n,,`).
pub struct ScramSha256 {
    authcid: String,
    password: String,
    /// gs2-header, e.g. `n,,` (plain) or `p=tls-server-end-point,,` (PLUS).
    gs2_header: String,
    /// Raw channel-binding data appended after the gs2-header (empty = plain).
    cbind_data: Vec<u8>,
    client_nonce: String,
    client_first_bare: String,
    server_signature: Vec<u8>,
    state: ScramState,
}

impl ScramSha256 {
    /// A SCRAM client with a fresh random nonce. `channel_binding` selects the
    /// `-PLUS` variant when `Some` (the `tls-server-end-point` bytes).
    pub fn new(username: &str, password: &str, channel_binding: Option<Vec<u8>>) -> Self {
        let mut nonce = [0u8; 24];
        getrandom::fill(&mut nonce).expect("system CSPRNG unavailable");
        Self::with_nonce(
            username,
            password,
            channel_binding,
            &B64_NOPAD.encode(nonce),
        )
    }

    /// Construct with a caller-supplied `client_nonce` (deterministic tests).
    pub fn with_nonce(
        username: &str,
        password: &str,
        channel_binding: Option<Vec<u8>>,
        client_nonce: &str,
    ) -> Self {
        let (gs2_header, cbind_data) = match channel_binding {
            Some(cb) => ("p=tls-server-end-point,,".to_string(), cb),
            None => ("n,,".to_string(), Vec::new()),
        };
        ScramSha256 {
            authcid: escape_scram(username),
            password: password.to_string(),
            gs2_header,
            cbind_data,
            client_nonce: client_nonce.to_string(),
            client_first_bare: String::new(),
            server_signature: Vec::new(),
            state: ScramState::Initial,
        }
    }
}

/// Split a `server-first-message` into `(server_nonce, salt_b64, iterations)`.
fn parse_server_first(msg: &str) -> Result<(String, String, u32), String> {
    let (mut nonce, mut salt, mut iters) = (None, None, None);
    for tok in msg.split(',') {
        if let Some(v) = tok.strip_prefix("r=") {
            nonce = Some(v.to_string());
        } else if let Some(v) = tok.strip_prefix("s=") {
            salt = Some(v.to_string());
        } else if let Some(v) = tok.strip_prefix("i=") {
            iters = Some(
                v.parse::<u32>()
                    .map_err(|_| "invalid i= iteration count".to_string())?,
            );
        }
    }
    Ok((
        nonce.ok_or("server-first missing r=")?,
        salt.ok_or("server-first missing s=")?,
        iters.ok_or("server-first missing i=")?,
    ))
}

impl SaslClient for ScramSha256 {
    fn step(&mut self, challenge: &[u8]) -> Result<Vec<u8>, String> {
        match self.state {
            ScramState::Initial => {
                // The server's initial bare continuation; emit client-first.
                self.client_first_bare = format!("n={},r={}", self.authcid, self.client_nonce);
                self.state = ScramState::ClientFirstSent;
                Ok(format!("{}{}", self.gs2_header, self.client_first_bare).into_bytes())
            }
            ScramState::ClientFirstSent => {
                let server_first =
                    std::str::from_utf8(challenge).map_err(|_| "server-first not UTF-8")?;
                let (server_nonce, salt_b64, iters) = parse_server_first(server_first)?;
                if !server_nonce.starts_with(&self.client_nonce) {
                    return Err("server nonce does not extend the client nonce".into());
                }
                let salt = B64
                    .decode(salt_b64.as_bytes())
                    .map_err(|e| format!("invalid SCRAM salt: {e}"))?;
                let salted = pbkdf2_hmac_sha256(self.password.as_bytes(), &salt, iters);
                let client_key = hmac_sha256(&salted, b"Client Key");
                let stored_key = sha256(&client_key);
                let server_key = hmac_sha256(&salted, b"Server Key");

                let mut cbind_input = self.gs2_header.clone().into_bytes();
                cbind_input.extend_from_slice(&self.cbind_data);
                let client_final_bare =
                    format!("c={},r={}", B64.encode(&cbind_input), server_nonce);
                let auth_message = format!(
                    "{},{},{}",
                    self.client_first_bare, server_first, client_final_bare
                );

                let client_sig = hmac_sha256(&stored_key, auth_message.as_bytes());
                let mut proof = client_key;
                for (p, s) in proof.iter_mut().zip(client_sig.iter()) {
                    *p ^= *s;
                }
                self.server_signature = hmac_sha256(&server_key, auth_message.as_bytes()).to_vec();
                self.state = ScramState::ClientFinalSent;
                Ok(format!("{client_final_bare},p={}", B64.encode(proof)).into_bytes())
            }
            ScramState::ClientFinalSent => {
                let server_final =
                    std::str::from_utf8(challenge).map_err(|_| "server-final not UTF-8")?;
                if let Some(err) = server_final.trim().strip_prefix("e=") {
                    return Err(format!("server rejected SCRAM: {err}"));
                }
                let v = server_final
                    .trim()
                    .strip_prefix("v=")
                    .ok_or("server-final missing v= signature")?;
                let sig = B64
                    .decode(v.as_bytes())
                    .map_err(|e| format!("invalid server signature: {e}"))?;
                if sig != self.server_signature {
                    return Err("server signature verification failed".into());
                }
                self.state = ScramState::Done;
                Ok(Vec::new())
            }
            ScramState::Done => Ok(Vec::new()),
        }
    }
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

    #[test]
    fn oauthbearer_frame_matches_rfc7628_shape() {
        let frame = oauthbearer("user@example.com", "TOK", "server.example.com", 143);
        let decoded = B64.decode(frame).unwrap();
        assert_eq!(
            decoded,
            b"n,a=user@example.com,\x01host=server.example.com\x01port=143\x01auth=Bearer TOK\x01\x01"
        );
    }

    #[test]
    fn pbkdf2_single_iteration_equals_prf() {
        // With one iteration PBKDF2 collapses to HMAC(pw, salt || INT(1)).
        let salt = b"salt";
        let mut expected_input = salt.to_vec();
        expected_input.extend_from_slice(&1u32.to_be_bytes());
        assert_eq!(
            pbkdf2_hmac_sha256(b"password", salt, 1),
            hmac_sha256(b"password", &expected_input)
        );
    }

    /// RFC 7677 §3 SCRAM-SHA-256 exchange: user "user", password "pencil".
    #[test]
    fn scram_sha256_matches_rfc7677_vector() {
        let mut client = ScramSha256::with_nonce("user", "pencil", None, "rOprNGfwEbeRWgbNEkqO");

        // Server's initial bare continuation → client-first.
        let client_first = client.step(b"").unwrap();
        assert_eq!(client_first, b"n,,n=user,r=rOprNGfwEbeRWgbNEkqO");

        // Server-first → client-final (proof must match the RFC's value).
        let server_first = "r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096";
        let client_final = client.step(server_first.as_bytes()).unwrap();
        assert_eq!(
            String::from_utf8(client_final).unwrap(),
            "c=biws,r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,\
             p=dHzbZapWIk4jUhN+Ute9ytag9zjfMHgsqmmiz7AndVQ="
        );

        // Server-final signature verifies; the client acknowledges with empty.
        let ack = client
            .step(b"v=6rriTRBi23WpRR/wtup+mMhUZUn/dB5nLTJRsjl95G4=")
            .unwrap();
        assert!(ack.is_empty());
        assert_eq!(client.state, ScramState::Done);
    }

    #[test]
    fn scram_rejects_forged_server_signature() {
        let mut client = ScramSha256::with_nonce("user", "pencil", None, "rOprNGfwEbeRWgbNEkqO");
        client.step(b"").unwrap();
        let server_first = "r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096";
        client.step(server_first.as_bytes()).unwrap();
        // A signature over the wrong bytes must be rejected.
        let err = client
            .step(b"v=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
            .unwrap_err();
        assert!(err.contains("verification failed"), "{err}");
    }

    #[test]
    fn scram_plus_uses_channel_bound_gs2_header() {
        let mut client =
            ScramSha256::with_nonce("user", "pencil", Some(vec![0xAB; 32]), "nonce123");
        let client_first = client.step(b"").unwrap();
        assert_eq!(client_first, b"p=tls-server-end-point,,n=user,r=nonce123");
        // The client-final `c=` echoes gs2-header || cbind-data, base64'd.
        let server_first = "r=nonce123SERVER,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096";
        let client_final =
            String::from_utf8(client.step(server_first.as_bytes()).unwrap()).unwrap();
        let mut cbind = b"p=tls-server-end-point,,".to_vec();
        cbind.extend_from_slice(&[0xAB; 32]);
        assert!(client_final.starts_with(&format!("c={},", B64.encode(&cbind))));
    }

    #[test]
    fn escape_scram_encodes_reserved_chars() {
        assert_eq!(escape_scram("a,b=c"), "a=2Cb=3Dc");
    }
}
