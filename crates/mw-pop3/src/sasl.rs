//! SASL initial-response encoders for POP3 `AUTH` (RFC 5034 + mechanism specs),
//! plus a client-side SCRAM-SHA-256 state machine (RFC 5802 / RFC 7677).
//!
//! Pure `String`-producing helpers so the exact wire bytes can be asserted in
//! tests. The transport ([`crate::conn`]) frames them into `AUTH` exchanges.
//! `SCRAM-SHA-256` needs a struct to carry state between the client-first and
//! client-final steps. The SHA-256/HMAC primitives come from the in-tree
//! `sha2`/`hmac` crates; PBKDF2 is the trivial single-block HMAC iteration
//! (dkLen == hLen == 32), so no `pbkdf2` dependency is pulled.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

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

/// SASL `OAUTHBEARER` (RFC 7628 §3.1): base64 of
/// `n,a=<user>,^Aauth=Bearer <token>^A^A`. `host`/`port` key-value pairs are
/// optional and omitted (advisory; servers key off the bearer).
pub fn oauthbearer(username: &str, access_token: &str) -> String {
    B64.encode(format!(
        "n,a={username},\x01auth=Bearer {access_token}\x01\x01"
    ))
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

/// SCRAM name escaping (RFC 5802 §5.1): a literal `,` becomes `=2C` and `=`
/// becomes `=3D` so the comma-delimited grammar stays parseable.
fn scram_escape(s: &str) -> String {
    s.replace('=', "=3D").replace(',', "=2C")
}

/// An unpredictable, comma/equals-free client nonce (hex over OS-seeded
/// entropy). `RandomState::new()` draws fresh keys from the platform RNG on each
/// call; hex keeps the value inside the SCRAM `printable` set with no escaping.
pub fn client_nonce() -> String {
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
/// [`ScramSha256::new`] yields the client-first-message; feed the
/// server-first-message to [`client_final`](ScramSha256::client_final) for the
/// client-final-message, then check the server-final with
/// [`verify`](ScramSha256::verify).
pub struct ScramSha256 {
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
    pub fn new(username: &str, password: &str, client_nonce: &str) -> (Self, String) {
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
    /// (`c=...,r=...,p=<proof>`). Also derives the expected server signature.
    pub fn client_final(&mut self, server_first: &str) -> Result<String, String> {
        let (mut nonce, mut salt_b64, mut iter_s) = (None, None, None);
        for field in server_first.split(',') {
            let (k, v) = field
                .split_once('=')
                .ok_or_else(|| format!("malformed server-first field {field:?}"))?;
            match k {
                "r" => nonce = Some(v),
                "s" => salt_b64 = Some(v),
                "i" => iter_s = Some(v),
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
        let salt = B64
            .decode(salt_b64)
            .map_err(|e| format!("bad SCRAM salt: {e}"))?;

        let salted = pbkdf2_sha256(&self.password, &salt, iterations);
        let client_key = hmac_sha256(&salted, b"Client Key");
        let stored_key = sha256(&client_key);

        let channel_binding = B64.encode(self.gs2_header.as_bytes());
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

        Ok(format!("{client_final_no_proof},p={}", B64.encode(proof)))
    }

    /// Verify the server-final-message (`v=<base64 server signature>`).
    pub fn verify(&self, server_final: &str) -> Result<(), String> {
        let v = server_final
            .split(',')
            .find_map(|f| f.strip_prefix("v="))
            .ok_or("server-final missing v=")?;
        let sig = B64
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

    #[test]
    fn oauthbearer_frame() {
        let enc = oauthbearer("user@example.com", "vF9dft4qmT");
        let decoded = B64.decode(enc).unwrap();
        assert_eq!(
            decoded,
            b"n,a=user@example.com,\x01auth=Bearer vF9dft4qmT\x01\x01"
        );
    }

    // RFC 4231 §4.2 HMAC-SHA-256 case 2, and FIPS-180 SHA-256("abc").
    #[test]
    fn primitive_vectors() {
        assert_eq!(
            hex(&hmac_sha256(b"Jefe", b"what do ya want for nothing?")),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
        assert_eq!(
            hex(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    /// The full SCRAM-SHA-256 exchange from RFC 7677 §3 (user `user`, password
    /// `pencil`) against the published client-proof and server-signature.
    #[test]
    fn scram_sha256_rfc7677_vector() {
        let (mut scram, client_first) = ScramSha256::new("user", "pencil", "rOprNGfwEbeRWgbNEkqO");
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
    fn scram_rejects_forged_nonce_and_signature() {
        let (mut scram, _) = ScramSha256::new("user", "pencil", "clientnonce");
        assert!(
            scram
                .client_final("r=other,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096")
                .unwrap_err()
                .contains("extend")
        );
        let (mut ok, _) = ScramSha256::new("user", "pencil", "rOprNGfwEbeRWgbNEkqO");
        ok.client_final(
            "r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096",
        )
        .unwrap();
        assert!(
            ok.verify("v=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
                .is_err()
        );
    }

    fn hex(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            s.push_str(&format!("{b:02x}"));
        }
        s
    }
}
