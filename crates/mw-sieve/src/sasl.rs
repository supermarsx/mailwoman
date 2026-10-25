//! SASL mechanisms for the ManageSieve client: PLAIN (RFC 4616), LOGIN,
//! XOAUTH2, OAUTHBEARER (RFC 7628) and SCRAM-SHA-256 (RFC 5802 / RFC 7677).
//!
//! The same mechanisms live in `mw-imap` and `mw-pop3`, but `mw-sieve` cannot
//! depend on either (both depend on `mw-engine`, which depends on this crate),
//! so this is a local copy of the client halves, as the POP3 one is. PBKDF2 is
//! the single-block HMAC iteration (`dkLen == hLen == 32`) over the in-tree
//! `hmac` + `sha2` crates. The `-PLUS` channel-binding variant is not offered:
//! the transport does not expose the TLS session to this layer.
//!
//! SASLprep (RFC 4013) is not applied; it is a no-op for ASCII credentials.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

/// Highest PBKDF2 iteration count accepted from a server. The count is chosen
/// by the server and each round costs the client an HMAC, so a hostile server
/// could otherwise pin a worker for minutes. Real deployments use 4096-10000.
pub const MAX_SCRAM_ITERATIONS: u32 = 600_000;

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
        scram_escape(username)
    )
    .into_bytes()
}

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

/// RFC 5802 §5.1: `=` → `=3D`, `,` → `=2C` in names.
fn scram_escape(s: &str) -> String {
    s.replace('=', "=3D").replace(',', "=2C")
}

/// A 32-character hex nonce from OS-seeded hasher keys (the same source the
/// POP3 client uses); hex needs no SCRAM escaping.
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

/// Client half of SCRAM-SHA-256: [`ScramSha256::new`] gives the client-first
/// message, [`client_final`](Self::client_final) answers the server-first, and
/// [`verify`](Self::verify) checks the server's signature, which proves the
/// server knows the password too.
pub struct ScramSha256 {
    password: Vec<u8>,
    client_nonce: String,
    client_first_bare: String,
    server_signature: Option<[u8; 32]>,
}

impl ScramSha256 {
    pub fn new(username: &str, password: &str, client_nonce: &str) -> (Self, String) {
        let client_first_bare = format!("n={},r={client_nonce}", scram_escape(username));
        let client_first = format!("n,,{client_first_bare}");
        (
            ScramSha256 {
                password: password.as_bytes().to_vec(),
                client_nonce: client_nonce.to_string(),
                client_first_bare,
                server_signature: None,
            },
            client_first,
        )
    }

    pub fn client_final(&mut self, server_first: &str) -> Result<String, String> {
        let (mut nonce, mut salt_b64, mut iter_s) = (None, None, None);
        for field in server_first.split(',') {
            let (k, v) = field
                .split_once('=')
                .ok_or_else(|| "malformed SCRAM server-first message".to_string())?;
            match k {
                "r" => nonce = Some(v),
                "s" => salt_b64 = Some(v),
                "i" => iter_s = Some(v),
                "m" => return Err("unsupported mandatory SCRAM extension".into()),
                "e" => return Err(format!("server refused SCRAM: {v}")),
                _ => {}
            }
        }
        let server_nonce = nonce.ok_or("SCRAM server-first has no nonce")?;
        let salt_b64 = salt_b64.ok_or("SCRAM server-first has no salt")?;
        let iterations: u32 = iter_s
            .ok_or("SCRAM server-first has no iteration count")?
            .parse()
            .map_err(|_| "SCRAM iteration count is not a number".to_string())?;
        if iterations == 0 || iterations > MAX_SCRAM_ITERATIONS {
            return Err(format!(
                "SCRAM iteration count {iterations} is outside 1..={MAX_SCRAM_ITERATIONS}"
            ));
        }
        if server_nonce.len() <= self.client_nonce.len()
            || !server_nonce.starts_with(&self.client_nonce)
        {
            return Err("SCRAM server nonce does not extend the client nonce".into());
        }
        let salt = B64
            .decode(salt_b64)
            .map_err(|_| "SCRAM salt is not base64".to_string())?;

        let salted = pbkdf2_sha256(&self.password, &salt, iterations);
        let client_key = hmac_sha256(&salted, b"Client Key");
        let stored_key = sha256(&client_key);
        let client_final_no_proof = format!("c=biws,r={server_nonce}");
        let auth_message = format!(
            "{},{server_first},{client_final_no_proof}",
            self.client_first_bare
        );
        let client_signature = hmac_sha256(&stored_key, auth_message.as_bytes());
        let mut proof = client_key;
        for (p, s) in proof.iter_mut().zip(client_signature.iter()) {
            *p ^= *s;
        }
        let server_key = hmac_sha256(&salted, b"Server Key");
        self.server_signature = Some(hmac_sha256(&server_key, auth_message.as_bytes()));
        Ok(format!("{client_final_no_proof},p={}", B64.encode(proof)))
    }

    /// Check the server-final message (`v=<signature>` or `e=<error>`).
    pub fn verify(&self, server_final: &str) -> Result<(), String> {
        let expected = self
            .server_signature
            .ok_or("SCRAM server-final arrived before the server-first")?;
        if let Some(e) = server_final.split(',').find_map(|f| f.strip_prefix("e=")) {
            return Err(format!("server refused SCRAM: {e}"));
        }
        let v = server_final
            .split(',')
            .find_map(|f| f.strip_prefix("v="))
            .ok_or("SCRAM server-final has no signature")?;
        let sig = B64
            .decode(v)
            .map_err(|_| "SCRAM server signature is not base64".to_string())?;
        if sig == expected {
            Ok(())
        } else {
            Err(
                "SCRAM server signature does not match: the server does not know the password"
                    .into(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 7677 §3: user `user`, password `pencil`.
    #[test]
    fn scram_sha256_rfc7677_vector() {
        let (mut scram, first) = ScramSha256::new("user", "pencil", "rOprNGfwEbeRWgbNEkqO");
        assert_eq!(first, "n,,n=user,r=rOprNGfwEbeRWgbNEkqO");
        let server_first = "r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096";
        assert_eq!(
            scram.client_final(server_first).unwrap(),
            "c=biws,r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,\
             p=dHzbZapWIk4jUhN+Ute9ytag9zjfMHgsqmmiz7AndVQ="
        );
        scram
            .verify("v=6rriTRBi23WpRR/wtup+mMhUZUn/dB5nLTJRsjl95G4=")
            .unwrap();
        assert!(scram.verify("v=AAAA").is_err());
        assert!(
            scram
                .verify("e=invalid-proof")
                .unwrap_err()
                .contains("invalid-proof")
        );
    }

    #[test]
    fn scram_refuses_hostile_server_first() {
        let fresh = || ScramSha256::new("u", "p", "abc").0;
        assert!(
            fresh().client_final("r=xyz,s=AAAA,i=4096").is_err(),
            "nonce not extended"
        );
        assert!(
            fresh().client_final("r=abc,s=AAAA,i=4096").is_err(),
            "nonce not extended at all"
        );
        assert!(
            fresh().client_final("r=abcd,s=AAAA,i=4000000000").is_err(),
            "iterations capped"
        );
        assert!(fresh().client_final("r=abcd,s=AAAA,i=0").is_err());
        assert!(fresh().client_final("r=abcd,s=!!,i=1").is_err());
        assert!(fresh().client_final("m=ext,r=abcd,s=AAAA,i=1").is_err());
        assert!(fresh().client_final("garbage").is_err());
        assert!(
            fresh().verify("v=AAAA").is_err(),
            "no signature before server-first"
        );
    }

    #[test]
    fn frames() {
        assert_eq!(plain("tim", "tan"), b"\0tim\0tan");
        assert_eq!(xoauth2("u@x", "t"), b"user=u@x\x01auth=Bearer t\x01\x01");
        assert_eq!(
            oauthbearer("u,=x", "t"),
            b"n,a=u=2C=3Dx,\x01auth=Bearer t\x01\x01"
        );
        let n = client_nonce();
        assert_eq!(n.len(), 32);
        assert!(n.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_ne!(n, client_nonce());
    }
}
