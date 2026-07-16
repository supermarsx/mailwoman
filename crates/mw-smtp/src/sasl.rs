//! SASL credential encoders + a client-side SCRAM-SHA-256 state machine
//! (plan §3 e4 / t12 §2, SPEC §6.1).
//!
//! Deliberately self-contained: the plan blesses duplicating this handful of
//! helpers per protocol rather than standing up a shared SASL crate. `PLAIN`,
//! `LOGIN` and `XOAUTH2` are plain encoders; `OAUTHBEARER` (RFC 7628) is one
//! more encoder; `SCRAM-SHA-256` (RFC 5802 / RFC 7677) is a small three-message
//! exchange, so it needs a struct to carry state between the client-first and
//! client-final steps. The SHA-256/HMAC primitives come from the in-tree
//! `sha2`/`hmac` crates; PBKDF2 is the trivial single-block HMAC iteration
//! (dkLen == hLen == 32), so no `pbkdf2` dependency is pulled.

use base64::prelude::*;
use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

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

/// `OAUTHBEARER` initial response (RFC 7628 §3.1): base64 of
/// `n,a=<user>,^Aauth=Bearer <token>^A^A`. The `host`/`port` key-value pairs are
/// optional and omitted here (they are advisory; servers key off the bearer).
pub(crate) fn oauthbearer(user: &str, token: &str) -> String {
    BASE64_STANDARD.encode(format!("n,a={user},\x01auth=Bearer {token}\x01\x01"))
}

// ---- SCRAM-SHA-256 (RFC 5802 / RFC 7677) client -------------------------

fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(msg);
    mac.finalize().into_bytes().into()
}

fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().into()
}

/// PBKDF2-HMAC-SHA-256 for the single output block SCRAM needs (`dkLen == hLen
/// == 32`), i.e. `U1 = HMAC(pw, salt || INT32BE(1))` folded over `i` rounds.
fn pbkdf2_sha256(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    let mut msg = Vec::with_capacity(salt.len() + 4);
    msg.extend_from_slice(salt);
    msg.extend_from_slice(&1u32.to_be_bytes());
    let mut u = hmac_sha256(password, &msg);
    let mut out = u;
    for _ in 1..iterations {
        u = hmac_sha256(password, &u);
        for (o, x) in out.iter_mut().zip(u.iter()) {
            *o ^= *x;
        }
    }
    out
}

/// SCRAM name escaping (RFC 5802 §5.1): in `n=`/authzid a literal `,` becomes
/// `=2C` and `=` becomes `=3D` so the comma-delimited grammar stays parseable.
fn scram_escape(s: &str) -> String {
    s.replace('=', "=3D").replace(',', "=2C")
}

/// An unpredictable, comma/equals-free client nonce (hex over OS-seeded
/// entropy). `RandomState::new()` draws fresh keys from the platform RNG on each
/// call; hex keeps the value inside the SCRAM `printable` set with no escaping.
pub(crate) fn client_nonce() -> String {
    use std::hash::{BuildHasher, Hasher, RandomState};
    let mut out = String::with_capacity(32);
    while out.len() < 32 {
        let h = RandomState::new().build_hasher().finish();
        out.push_str(&format!("{h:016x}"));
    }
    out.truncate(32);
    out
}

/// Client half of a `SCRAM-SHA-256` (non channel-bound) exchange.
///
/// Constructed with [`ScramSha256::new`], which also yields the
/// client-first-message. Feed the server-first-message to [`client_final`] to
/// get the client-final-message (with proof), then check the server-final with
/// [`verify`].
pub(crate) struct ScramSha256 {
    password: Vec<u8>,
    /// gs2 header bytes, base64'd into the `c=` attribute. For the non-PLUS
    /// mechanism this is `n,,` ⇒ `biws`.
    gs2_header: String,
    client_nonce: String,
    client_first_bare: String,
    server_signature: [u8; 32],
}

impl ScramSha256 {
    /// Build the client state and the client-first-message it must send first.
    pub(crate) fn new(username: &str, password: &str, client_nonce: &str) -> (Self, String) {
        let gs2_header = "n,,".to_string();
        let client_first_bare = format!("n={},r={}", scram_escape(username), client_nonce);
        let client_first = format!("{gs2_header}{client_first_bare}");
        (
            Self {
                password: password.as_bytes().to_vec(),
                gs2_header,
                client_nonce: client_nonce.to_string(),
                client_first_bare,
                server_signature: [0u8; 32],
            },
            client_first,
        )
    }

    /// Consume the server-first-message and produce the client-final-message
    /// (`c=...,r=...,p=<proof>`). Also derives the expected server signature for
    /// [`verify`].
    pub(crate) fn client_final(&mut self, server_first: &str) -> Result<String, String> {
        let (mut nonce, mut salt_b64, mut iter_s) = (None, None, None);
        for field in server_first.split(',') {
            let (k, v) = field
                .split_once('=')
                .ok_or_else(|| format!("malformed server-first field {field:?}"))?;
            match k {
                "r" => nonce = Some(v),
                "s" => salt_b64 = Some(v),
                "i" => iter_s = Some(v),
                // A mandatory extension we do not understand: fail closed.
                "m" => return Err("unsupported mandatory SCRAM extension".into()),
                _ => {}
            }
        }
        let server_nonce = nonce.ok_or("server-first missing r=")?;
        let salt_b64 = salt_b64.ok_or("server-first missing s=")?;
        let iterations: u32 = iter_s
            .ok_or("server-first missing i=")?
            .parse()
            .map_err(|_| "server-first i= is not a number".to_string())?;
        if !server_nonce.starts_with(&self.client_nonce) {
            return Err("server nonce does not extend the client nonce".into());
        }
        let salt = BASE64_STANDARD
            .decode(salt_b64)
            .map_err(|e| format!("bad SCRAM salt: {e}"))?;

        let salted = pbkdf2_sha256(&self.password, &salt, iterations);
        let client_key = hmac_sha256(&salted, b"Client Key");
        let stored_key = sha256(&client_key);

        let channel_binding = BASE64_STANDARD.encode(self.gs2_header.as_bytes());
        let client_final_no_proof = format!("c={channel_binding},r={server_nonce}");
        let auth_message = format!(
            "{},{},{}",
            self.client_first_bare, server_first, client_final_no_proof
        );

        let client_signature = hmac_sha256(&stored_key, auth_message.as_bytes());
        let mut proof = client_key;
        for (p, s) in proof.iter_mut().zip(client_signature.iter()) {
            *p ^= *s;
        }

        let server_key = hmac_sha256(&salted, b"Server Key");
        self.server_signature = hmac_sha256(&server_key, auth_message.as_bytes());

        Ok(format!(
            "{client_final_no_proof},p={}",
            BASE64_STANDARD.encode(proof)
        ))
    }

    /// Verify the server-final-message (`v=<base64 server signature>`) proves the
    /// server also knows the salted password.
    pub(crate) fn verify(&self, server_final: &str) -> Result<(), String> {
        // Server-final may in theory carry extension fields; `v=` is what we need.
        let v = server_final
            .split(',')
            .find_map(|f| f.strip_prefix("v="))
            .ok_or("server-final missing v=")?;
        let sig = BASE64_STANDARD
            .decode(v)
            .map_err(|e| format!("bad server signature: {e}"))?;
        if sig == self.server_signature {
            Ok(())
        } else {
            Err("server signature did not verify".into())
        }
    }
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

    #[test]
    fn oauthbearer_frame_is_rfc7628_shape() {
        let ir = oauthbearer("alice@example.com", "vF9dft4qmT");
        let decoded = BASE64_STANDARD.decode(ir).unwrap();
        assert_eq!(
            decoded,
            b"n,a=alice@example.com,\x01auth=Bearer vF9dft4qmT\x01\x01"
        );
    }

    // RFC 4231 §4.2 HMAC-SHA-256 test case 2 (key "Jefe", data "what do ya want
    // for nothing?"), pinning the primitive the SCRAM proof rides on.
    #[test]
    fn hmac_sha256_rfc4231_case2() {
        let mac = hmac_sha256(b"Jefe", b"what do ya want for nothing?");
        assert_eq!(
            hex(&mac),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    // NIST FIPS-180 SHA-256("abc").
    #[test]
    fn sha256_abc_vector() {
        assert_eq!(
            hex(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    /// The full SCRAM-SHA-256 exchange from RFC 7677 §3 (user `user`, password
    /// `pencil`) — exercises client-first, PBKDF2(i=4096), the client proof, and
    /// server-signature verification against the published vector.
    #[test]
    fn scram_sha256_rfc7677_vector() {
        let client_nonce = "rOprNGfwEbeRWgbNEkqO";
        let (mut scram, client_first) = ScramSha256::new("user", "pencil", client_nonce);
        assert_eq!(client_first, "n,,n=user,r=rOprNGfwEbeRWgbNEkqO");

        let server_first = "r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096";
        let client_final = scram.client_final(server_first).unwrap();
        assert_eq!(
            client_final,
            "c=biws,r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,\
             p=dHzbZapWIk4jUhN+Ute9ytag9zjfMHgsqmmiz7AndVQ="
        );

        scram
            .verify("v=6rriTRBi23WpRR/wtup+mMhUZUn/dB5nLTJRsjl95G4=")
            .expect("server signature verifies against the RFC 7677 vector");
    }

    #[test]
    fn scram_rejects_a_mismatched_server_signature() {
        let (mut scram, _) = ScramSha256::new("user", "pencil", "rOprNGfwEbeRWgbNEkqO");
        let _ = scram
            .client_final(
                "r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096",
            )
            .unwrap();
        assert!(
            scram
                .verify("v=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
                .is_err()
        );
    }

    #[test]
    fn scram_rejects_a_forged_server_nonce() {
        let (mut scram, _) = ScramSha256::new("user", "pencil", "clientnonce");
        // Server nonce must begin with the client nonce.
        let err = scram
            .client_final("r=somethingelse,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096")
            .unwrap_err();
        assert!(err.contains("extend"), "{err}");
    }

    fn hex(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            s.push_str(&format!("{b:02x}"));
        }
        s
    }
}
