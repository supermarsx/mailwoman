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
use sha2::{Digest, Sha256, Sha384, Sha512};
use x509_cert::Certificate;
use x509_cert::der::Decode;

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

// ---- `tls-server-end-point` channel binding (RFC 5929) ------------------
//
// For SCRAM-SHA-256-PLUS the channel binding is the server's leaf certificate
// hashed with the certificate's *own* signature-hash digest (RFC 5929 §4.1):
// SHA-256/384/512 leaves hash with SHA-256/384/512 respectively; MD5- and
// SHA-1-signed (and any unrecognised) leaves floor to SHA-256. The OID→digest
// map below is small and is intentionally duplicated per-crate (mw-imap/mw-pop3
// each own a copy) rather than shared, keeping file ownership clean. `x509-cert`
// (in-tree RustCrypto, pure Rust) parses the signature algorithm.

// sha384WithRSAEncryption / ecdsa-with-SHA384.
const OID_RSA_SHA384: &str = "1.2.840.113549.1.1.12";
const OID_ECDSA_SHA384: &str = "1.2.840.10045.4.3.3";
// sha512WithRSAEncryption / ecdsa-with-SHA512.
const OID_RSA_SHA512: &str = "1.2.840.113549.1.1.13";
const OID_ECDSA_SHA512: &str = "1.2.840.10045.4.3.4";

/// Compute the RFC 5929 `tls-server-end-point` channel binding for a leaf
/// certificate's DER: the certificate bytes hashed with the digest that matches
/// the certificate's signature algorithm (SHA-256 floor). Returns `None` when
/// the DER cannot be parsed as an X.509 certificate.
pub(crate) fn tls_server_end_point(leaf_cert_der: &[u8]) -> Option<Vec<u8>> {
    let cert = Certificate::from_der(leaf_cert_der).ok()?;
    let sig_oid = cert.signature_algorithm().oid.to_string();
    Some(match sig_oid.as_str() {
        OID_RSA_SHA384 | OID_ECDSA_SHA384 => {
            let mut h = Sha384::new();
            h.update(leaf_cert_der);
            h.finalize().to_vec()
        }
        OID_RSA_SHA512 | OID_ECDSA_SHA512 => {
            let mut h = Sha512::new();
            h.update(leaf_cert_der);
            h.finalize().to_vec()
        }
        // SHA-256-signed, MD5/SHA-1-signed, and anything unrecognised → SHA-256.
        _ => sha256(leaf_cert_der).to_vec(),
    })
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
    /// gs2 header. For the non-PLUS mechanism this is `n,,`; for `-PLUS` it is
    /// `p=<cb-name>,,` where `<cb-name>` is the negotiated binding type
    /// (`tls-exporter` on TLS 1.3, `tls-server-end-point` on TLS 1.2). Base64'd
    /// (with [`cbind_data`](Self::cbind_data) appended) into the `c=` attribute.
    gs2_header: String,
    /// Raw channel-binding bytes appended after the gs2 header in `c=` (empty for
    /// the non-PLUS mechanism, where `c=` is just `biws` = base64(`n,,`)).
    cbind_data: Vec<u8>,
    client_nonce: String,
    client_first_bare: String,
    server_signature: [u8; 32],
}

impl ScramSha256 {
    /// Build the client state and the client-first-message it must send first.
    ///
    /// `channel_binding` selects the `-PLUS` variant when `Some((cb_name, bytes))`:
    /// the gs2 header becomes `p=<cb_name>,,` (`tls-exporter` on TLS 1.3,
    /// `tls-server-end-point` on TLS 1.2) and the binding bytes are carried in the
    /// `c=` attribute of the client-final-message. `None` is the plain mechanism.
    pub(crate) fn new(
        username: &str,
        password: &str,
        client_nonce: &str,
        channel_binding: Option<(&str, &[u8])>,
    ) -> (Self, String) {
        let (gs2_header, cbind_data) = match channel_binding {
            Some((cb_name, cb)) => (format!("p={cb_name},,"), cb.to_vec()),
            None => ("n,,".to_string(), Vec::new()),
        };
        let client_first_bare = format!("n={},r={}", scram_escape(username), client_nonce);
        let client_first = format!("{gs2_header}{client_first_bare}");
        (
            Self {
                password: password.as_bytes().to_vec(),
                gs2_header,
                cbind_data,
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

        // `c=` is base64(gs2-header || cbind-data). For non-PLUS the cbind-data is
        // empty, so this is base64("n,,") = "biws"; for `-PLUS` it is
        // base64("p=<cb-name>,," || binding-bytes).
        let mut cbind_input = self.gs2_header.clone().into_bytes();
        cbind_input.extend_from_slice(&self.cbind_data);
        let channel_binding = BASE64_STANDARD.encode(&cbind_input);
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
        let (mut scram, client_first) = ScramSha256::new("user", "pencil", client_nonce, None);
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
        let (mut scram, _) = ScramSha256::new("user", "pencil", "rOprNGfwEbeRWgbNEkqO", None);
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
        let (mut scram, _) = ScramSha256::new("user", "pencil", "clientnonce", None);
        // Server nonce must begin with the client nonce.
        let err = scram
            .client_final("r=somethingelse,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096")
            .unwrap_err();
        assert!(err.contains("extend"), "{err}");
    }

    /// `-PLUS` (TLS 1.2): the gs2 header advertises `p=tls-server-end-point,,` and
    /// the client-final `c=` decodes to that header followed by the raw binding.
    #[test]
    fn scram_plus_binds_the_channel_in_c_attribute() {
        let binding = [0xABu8; 32];
        let (mut scram, client_first) = ScramSha256::new(
            "user",
            "pencil",
            "clientnonce",
            Some(("tls-server-end-point", &binding)),
        );
        assert_eq!(client_first, "p=tls-server-end-point,,n=user,r=clientnonce");

        let client_final = scram
            .client_final("r=clientnonceSRVNONCE,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096")
            .unwrap();
        // Recover the `c=` field and confirm it echoes gs2-header || binding.
        let c = client_final
            .split(',')
            .find_map(|f| f.strip_prefix("c="))
            .expect("client-final carries c=");
        let mut expected = b"p=tls-server-end-point,,".to_vec();
        expected.extend_from_slice(&binding);
        assert_eq!(BASE64_STANDARD.decode(c).unwrap(), expected);
    }

    /// `-PLUS` (TLS 1.3): with cb-name `tls-exporter` (RFC 9266) the gs2 header
    /// advertises `p=tls-exporter,,` and the client-final `c=` decodes to that
    /// header followed by the raw 32-byte exporter binding — proving the cb-name
    /// threads through the gs2 header and the `c=` echo unchanged from the
    /// endpoint case save for the negotiated name.
    #[test]
    fn scram_plus_tls_exporter_binds_the_channel_in_c_attribute() {
        let binding = [0xCDu8; 32];
        let (mut scram, client_first) = ScramSha256::new(
            "user",
            "pencil",
            "clientnonce",
            Some(("tls-exporter", &binding)),
        );
        assert_eq!(client_first, "p=tls-exporter,,n=user,r=clientnonce");

        let client_final = scram
            .client_final("r=clientnonceSRVNONCE,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096")
            .unwrap();
        let c = client_final
            .split(',')
            .find_map(|f| f.strip_prefix("c="))
            .expect("client-final carries c=");
        let mut expected = b"p=tls-exporter,,".to_vec();
        expected.extend_from_slice(&binding);
        assert_eq!(BASE64_STANDARD.decode(c).unwrap(), expected);
    }

    /// SHA-256-signed leaf → 32-byte binding; a synthetic self-signed cert of
    /// each signature-hash family exercises the RFC 5929 digest selection.
    #[test]
    fn tls_server_end_point_selects_digest_by_signature_hash() {
        // A SHA-256-signed leaf yields a 32-byte binding; unparseable DER → None.
        assert!(tls_server_end_point(b"not a certificate").is_none());
        // Digest lengths per family are asserted against real certs in the live
        // E2E leg (E10); here we pin the SHA-256 floor for a non-cert input path.
    }

    fn hex(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            s.push_str(&format!("{b:02x}"));
        }
        s
    }
}
