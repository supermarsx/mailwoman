//! Unit tests for `mw-mfa`: RFC 6238 TOTP vectors, recovery-code round-trips +
//! single-use, and hand-built WebAuthn ES256/EdDSA registration + assertion
//! ceremonies with the full matrix of negative paths.

use base64::Engine;
use sha2::{Digest, Sha256};

use crate::cose::{CoseAlg, CoseKey};
use crate::recovery::{RecoveryCode, consume, generate_codes, hash_code, verify_code};
use crate::totp::{
    TotpParams, base32_decode, base32_encode, generate_secret, totp_at, totp_verify,
};
use crate::webauthn::{
    AssertionRequest, RegistrationRequest, UserVerification, verify_assertion, verify_registration,
};

// ── TOTP: RFC 6238 Appendix B test vectors (HMAC-SHA1) ───────────────────────
// Seed is the ASCII string "12345678901234567890" (20 bytes); codes are 8 digits.

const RFC_SEED: &[u8] = b"12345678901234567890";

fn rfc_params() -> TotpParams {
    TotpParams {
        step: 30,
        digits: 8,
        skew: 0,
    }
}

#[test]
fn totp_rfc6238_vectors() {
    let cases: &[(u64, &str)] = &[
        (59, "94287082"),
        (1_111_111_109, "07081804"),
        (1_111_111_111, "14050471"),
        (1_234_567_890, "89005924"),
        (2_000_000_000, "69279037"),
        (20_000_000_000, "65353130"),
    ];
    for (t, expected) in cases {
        assert_eq!(&totp_at(RFC_SEED, *t, &rfc_params()), expected, "t={t}");
    }
}

#[test]
fn totp_six_digit_is_low_order_of_eight() {
    // 6-digit code is the 8-digit value mod 10^6.
    let p6 = TotpParams {
        step: 30,
        digits: 6,
        skew: 0,
    };
    assert_eq!(totp_at(RFC_SEED, 59, &p6), "287082");
}

#[test]
fn totp_verify_accepts_window_and_rejects_others() {
    let p = TotpParams {
        step: 30,
        digits: 6,
        skew: 1,
    };
    let now = 1_234_567_890u64;
    let code = totp_at(RFC_SEED, now, &p);
    // Current step verifies, returning the matched step counter (now / step).
    assert_eq!(totp_verify(RFC_SEED, &code, now, &p), Some(now / p.step));
    // Previous and next steps verify within ±1, reporting the step whose code it is.
    assert_eq!(
        totp_verify(RFC_SEED, &code, now.saturating_sub(30), &p),
        Some(now / p.step)
    );
    assert_eq!(
        totp_verify(RFC_SEED, &code, now + 30, &p),
        Some(now / p.step)
    );
    // Two steps away is rejected.
    assert_eq!(totp_verify(RFC_SEED, &code, now + 60, &p), None);
    // A wrong code is rejected.
    assert_eq!(totp_verify(RFC_SEED, "000000", now, &p), None);
    // Malformed inputs are rejected without panicking.
    assert_eq!(totp_verify(RFC_SEED, "12ab56", now, &p), None);
    assert_eq!(totp_verify(RFC_SEED, "1234567", now, &p), None);
}

#[test]
fn base32_roundtrip_and_known_vector() {
    // RFC 4648 test vector: "foobar" → "MZXW6YTBOI".
    assert_eq!(base32_encode(b"foobar"), "MZXW6YTBOI");
    assert_eq!(base32_decode("MZXW6YTBOI").unwrap(), b"foobar");
    let secret = generate_secret();
    assert_eq!(base32_decode(&base32_encode(&secret)).unwrap(), secret);
    // Lowercase + spaces are tolerated by the decoder.
    assert_eq!(base32_decode("mzxw 6ytb oi").unwrap(), b"foobar");
    assert!(base32_decode("0189").is_none());
}

// ── Recovery codes ───────────────────────────────────────────────────────────

#[test]
fn recovery_codes_hash_roundtrip_and_reject_wrong() {
    let codes = generate_codes(10);
    assert_eq!(codes.len(), 10);
    for c in &codes {
        // Format is `xxxxx-xxxxx` from the unambiguous alphabet.
        assert_eq!(c.len(), 11);
        assert_eq!(c.as_bytes()[5], b'-');
        let h = hash_code(c);
        assert!(verify_code(c, &h));
        // Formatting-insensitive: dashes/case do not affect verification.
        assert!(verify_code(&c.to_lowercase().replace('-', ""), &h));
        assert!(!verify_code("WRONG-CODE0", &h));
    }
    // Codes are distinct.
    let mut sorted = codes.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), codes.len());
}

#[test]
fn recovery_codes_are_single_use() {
    let plain = generate_codes(3);
    let mut stored: Vec<RecoveryCode> = plain
        .iter()
        .map(|c| RecoveryCode {
            hash: hash_code(c),
            used: false,
        })
        .collect();

    // First use of a code succeeds.
    assert!(consume(&mut stored, &plain[1]));
    // The same code cannot be reused.
    assert!(!consume(&mut stored, &plain[1]));
    // A different, still-unused code works.
    assert!(consume(&mut stored, &plain[0]));
    // An unknown code never matches.
    assert!(!consume(&mut stored, "ZZZZZ-ZZZZZ"));
    // Exactly one code remains unused.
    assert_eq!(stored.iter().filter(|c| !c.used).count(), 1);
}

// ── WebAuthn: CBOR/authData builders for hand-built ceremonies ───────────────

const RP_ID: &str = "example.com";
const ORIGIN: &str = "https://example.com";

fn cbor_bstr_header(len: usize) -> Vec<u8> {
    if len < 24 {
        vec![0x40 | len as u8]
    } else if len < 256 {
        vec![0x58, len as u8]
    } else {
        vec![0x59, (len >> 8) as u8, (len & 0xff) as u8]
    }
}

/// COSE_Key for an ES256 (EC2/P-256) public key.
fn cose_es256(x: &[u8; 32], y: &[u8; 32]) -> Vec<u8> {
    let mut v = vec![0xa5]; // map(5)
    v.extend_from_slice(&[0x01, 0x02]); // kty(1) = EC2(2)
    v.extend_from_slice(&[0x03, 0x26]); // alg(3) = ES256(-7)
    v.extend_from_slice(&[0x20, 0x01]); // crv(-1) = P-256(1)
    v.extend_from_slice(&[0x21, 0x58, 0x20]); // x(-2) = bstr(32)
    v.extend_from_slice(x);
    v.extend_from_slice(&[0x22, 0x58, 0x20]); // y(-3) = bstr(32)
    v.extend_from_slice(y);
    v
}

/// COSE_Key for an EdDSA (OKP/Ed25519) public key.
fn cose_eddsa(x: &[u8; 32]) -> Vec<u8> {
    let mut v = vec![0xa4]; // map(4)
    v.extend_from_slice(&[0x01, 0x01]); // kty(1) = OKP(1)
    v.extend_from_slice(&[0x03, 0x27]); // alg(3) = EdDSA(-8)
    v.extend_from_slice(&[0x20, 0x06]); // crv(-1) = Ed25519(6)
    v.extend_from_slice(&[0x21, 0x58, 0x20]); // x(-2) = bstr(32)
    v.extend_from_slice(x);
    v
}

fn attested_cred(aaguid: &[u8; 16], cred_id: &[u8], cose: &[u8]) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(aaguid);
    v.extend_from_slice(&(cred_id.len() as u16).to_be_bytes());
    v.extend_from_slice(cred_id);
    v.extend_from_slice(cose);
    v
}

fn auth_data(rp_id: &str, flags: u8, sign_count: u32, tail: &[u8]) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(&Sha256::digest(rp_id.as_bytes()));
    v.push(flags);
    v.extend_from_slice(&sign_count.to_be_bytes());
    v.extend_from_slice(tail);
    v
}

fn attestation_object_none(auth_data: &[u8]) -> Vec<u8> {
    let mut v = vec![0xa3]; // map(3)
    v.extend_from_slice(&[0x63, b'f', b'm', b't']); // "fmt"
    v.extend_from_slice(&[0x64, b'n', b'o', b'n', b'e']); // "none"
    v.push(0x67);
    v.extend_from_slice(b"attStmt");
    v.push(0xa0); // {}
    v.push(0x68);
    v.extend_from_slice(b"authData");
    v.extend_from_slice(&cbor_bstr_header(auth_data.len()));
    v.extend_from_slice(auth_data);
    v
}

fn client_data(ty: &str, challenge: &[u8], origin: &str) -> Vec<u8> {
    let ch = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(challenge);
    format!(r#"{{"type":"{ty}","challenge":"{ch}","origin":"{origin}"}}"#).into_bytes()
}

// ── ES256 signer harness ─────────────────────────────────────────────────────

struct Es256Key {
    signing: p256::ecdsa::SigningKey,
    x: [u8; 32],
    y: [u8; 32],
}

impl Es256Key {
    fn new() -> Self {
        let signing = p256::ecdsa::SigningKey::from_slice(&[0x11u8; 32]).unwrap();
        let ep = signing.verifying_key().to_sec1_point(false);
        let b = ep.as_bytes(); // 0x04 || X(32) || Y(32)
        let mut x = [0u8; 32];
        let mut y = [0u8; 32];
        x.copy_from_slice(&b[1..33]);
        y.copy_from_slice(&b[33..65]);
        Es256Key { signing, x, y }
    }

    /// DER-encoded ES256 signature over `message` (ECDSA hashes with SHA-256).
    fn sign(&self, message: &[u8]) -> Vec<u8> {
        use p256::ecdsa::signature::Signer;
        let sig: p256::ecdsa::Signature = self.signing.sign(message);
        sig.to_der().as_bytes().to_vec()
    }

    fn cose(&self) -> Vec<u8> {
        cose_es256(&self.x, &self.y)
    }
}

// ── EdDSA signer harness ─────────────────────────────────────────────────────

struct EdKey {
    signing: ed25519_dalek::SigningKey,
    x: [u8; 32],
}

impl EdKey {
    fn new() -> Self {
        let signing = ed25519_dalek::SigningKey::from_bytes(&[0x42u8; 32]);
        let x = signing.verifying_key().to_bytes();
        EdKey { signing, x }
    }

    fn sign(&self, message: &[u8]) -> Vec<u8> {
        use ed25519_dalek::Signer;
        self.signing.sign(message).to_bytes().to_vec()
    }

    fn cose(&self) -> Vec<u8> {
        cose_eddsa(&self.x)
    }
}

/// Build the signed message = authenticatorData ‖ SHA256(clientDataJSON).
fn signed_message(auth_data: &[u8], client_data_json: &[u8]) -> Vec<u8> {
    let mut m = auth_data.to_vec();
    m.extend_from_slice(&Sha256::digest(client_data_json));
    m
}

// ── Registration ─────────────────────────────────────────────────────────────

#[test]
fn registration_es256_extracts_credential() {
    let key = Es256Key::new();
    let aaguid = [0u8; 16];
    let cred_id = b"credential-abc";
    let cose = key.cose();
    let att = attested_cred(&aaguid, cred_id, &cose);
    // AT | UP | UV.
    let ad = auth_data(RP_ID, 0x40 | 0x01 | 0x04, 7, &att);
    let attestation_object = attestation_object_none(&ad);
    let client_data_json = client_data("webauthn.create", b"reg-challenge", ORIGIN);

    let req = RegistrationRequest {
        challenge: b"reg-challenge".to_vec(),
        origin: ORIGIN.to_string(),
        rp_id: RP_ID.to_string(),
        attestation_object,
        client_data_json,
        user_verification: UserVerification::Preferred,
    };
    let cred = verify_registration(&req).expect("registration verifies");
    assert_eq!(cred.credential_id, cred_id);
    assert_eq!(cred.sign_count, 7);
    assert_eq!(cred.alg, CoseAlg::Es256);
    // The stored COSE bytes are exactly the key (no trailing bytes swept in).
    assert_eq!(cred.cose_public_key, cose);
}

#[test]
fn registration_measures_cose_exactly_ignoring_extensions() {
    // With the ED (extension-data) flag set, an extensions map trails the COSE key;
    // the stored key must not include it.
    let key = Es256Key::new();
    let cose = key.cose();
    let mut tail = attested_cred(&[0u8; 16], b"cred", &cose);
    tail.push(0xa0); // trailing (empty) extensions map
    let ad = auth_data(RP_ID, 0x40 | 0x80 | 0x01, 1, &tail); // AT | ED | UP
    let req = RegistrationRequest {
        challenge: b"c".to_vec(),
        origin: ORIGIN.to_string(),
        rp_id: RP_ID.to_string(),
        attestation_object: attestation_object_none(&ad),
        client_data_json: client_data("webauthn.create", b"c", ORIGIN),
        user_verification: UserVerification::Preferred,
    };
    let cred = verify_registration(&req).expect("registration verifies");
    assert_eq!(cred.cose_public_key, cose);
}

#[test]
fn registration_rejects_non_none_attestation() {
    // Hand-build an attestationObject with fmt "packed".
    let key = Es256Key::new();
    let ad = auth_data(
        RP_ID,
        0x41,
        0,
        &attested_cred(&[0u8; 16], b"c", &key.cose()),
    );
    let mut obj = vec![0xa2]; // map(2): fmt + authData
    obj.extend_from_slice(&[0x63, b'f', b'm', b't']);
    obj.extend_from_slice(&[0x66]);
    obj.extend_from_slice(b"packed");
    obj.push(0x68);
    obj.extend_from_slice(b"authData");
    obj.extend_from_slice(&cbor_bstr_header(ad.len()));
    obj.extend_from_slice(&ad);

    let req = RegistrationRequest {
        challenge: b"c".to_vec(),
        origin: ORIGIN.to_string(),
        rp_id: RP_ID.to_string(),
        attestation_object: obj,
        client_data_json: client_data("webauthn.create", b"c", ORIGIN),
        user_verification: UserVerification::Preferred,
    };
    assert!(matches!(
        verify_registration(&req),
        Err(crate::MfaError::Attestation(_))
    ));
}

// ── Assertion (the security-critical path) ───────────────────────────────────

fn good_es256_assertion() -> (AssertionRequest, u32) {
    let key = Es256Key::new();
    let ad = auth_data(RP_ID, 0x01 | 0x04, 10, &[]); // UP | UV, count 10
    let cdj = client_data("webauthn.get", b"login-challenge", ORIGIN);
    let signature = key.sign(&signed_message(&ad, &cdj));
    let req = AssertionRequest {
        challenge: b"login-challenge".to_vec(),
        origin: ORIGIN.to_string(),
        rp_id: RP_ID.to_string(),
        client_data_json: cdj,
        authenticator_data: ad,
        signature,
        cose_public_key: key.cose(),
        stored_sign_count: 9,
        user_verification: UserVerification::Preferred,
    };
    (req, 10)
}

#[test]
fn assertion_es256_verifies_and_returns_new_counter() {
    let (req, expected) = good_es256_assertion();
    let out = verify_assertion(&req).expect("assertion verifies");
    assert_eq!(out.new_sign_count, expected);
}

#[test]
fn assertion_eddsa_verifies() {
    let key = EdKey::new();
    let ad = auth_data(RP_ID, 0x01 | 0x04, 3, &[]);
    let cdj = client_data("webauthn.get", b"ed-challenge", ORIGIN);
    let signature = key.sign(&signed_message(&ad, &cdj));
    let req = AssertionRequest {
        challenge: b"ed-challenge".to_vec(),
        origin: ORIGIN.to_string(),
        rp_id: RP_ID.to_string(),
        client_data_json: cdj,
        authenticator_data: ad,
        signature,
        cose_public_key: key.cose(),
        stored_sign_count: 0,
        user_verification: UserVerification::Preferred,
    };
    assert_eq!(verify_assertion(&req).unwrap().new_sign_count, 3);
}

#[test]
fn assertion_rejects_tampered_signature() {
    let (mut req, _) = good_es256_assertion();
    let last = req.signature.len() - 1;
    req.signature[last] ^= 0xff;
    assert!(matches!(
        verify_assertion(&req),
        Err(crate::MfaError::Verify(_))
    ));
}

#[test]
fn assertion_rejects_wrong_challenge() {
    let (mut req, _) = good_es256_assertion();
    req.challenge = b"not-the-challenge".to_vec();
    assert!(matches!(
        verify_assertion(&req),
        Err(crate::MfaError::Challenge)
    ));
}

#[test]
fn assertion_rejects_wrong_origin() {
    let (mut req, _) = good_es256_assertion();
    req.origin = "https://evil.example".to_string();
    assert!(matches!(
        verify_assertion(&req),
        Err(crate::MfaError::Origin)
    ));
}

#[test]
fn assertion_rejects_wrong_rp_id() {
    let (mut req, _) = good_es256_assertion();
    // clientData origin still matches, but the rpIdHash will not.
    req.rp_id = "evil.example".to_string();
    assert!(matches!(
        verify_assertion(&req),
        Err(crate::MfaError::RpIdHash)
    ));
}

#[test]
fn assertion_rejects_missing_user_presence() {
    let key = Es256Key::new();
    let ad = auth_data(RP_ID, 0x00, 1, &[]); // no UP flag
    let cdj = client_data("webauthn.get", b"c", ORIGIN);
    let signature = key.sign(&signed_message(&ad, &cdj));
    let req = AssertionRequest {
        challenge: b"c".to_vec(),
        origin: ORIGIN.to_string(),
        rp_id: RP_ID.to_string(),
        client_data_json: cdj,
        authenticator_data: ad,
        signature,
        cose_public_key: key.cose(),
        stored_sign_count: 0,
        user_verification: UserVerification::Preferred,
    };
    assert!(matches!(
        verify_assertion(&req),
        Err(crate::MfaError::UserPresence)
    ));
}

#[test]
fn assertion_requires_uv_when_policy_requires() {
    let key = Es256Key::new();
    let ad = auth_data(RP_ID, 0x01, 1, &[]); // UP only, no UV
    let cdj = client_data("webauthn.get", b"c", ORIGIN);
    let signature = key.sign(&signed_message(&ad, &cdj));
    let req = AssertionRequest {
        challenge: b"c".to_vec(),
        origin: ORIGIN.to_string(),
        rp_id: RP_ID.to_string(),
        client_data_json: cdj,
        authenticator_data: ad,
        signature,
        cose_public_key: key.cose(),
        stored_sign_count: 0,
        user_verification: UserVerification::Required,
    };
    assert!(matches!(
        verify_assertion(&req),
        Err(crate::MfaError::UserVerification)
    ));
}

#[test]
fn assertion_rejects_counter_regression() {
    // Same-value counter (10 == stored 10) is a regression.
    let key = Es256Key::new();
    let ad = auth_data(RP_ID, 0x01 | 0x04, 10, &[]);
    let cdj = client_data("webauthn.get", b"c", ORIGIN);
    let signature = key.sign(&signed_message(&ad, &cdj));
    let req = AssertionRequest {
        challenge: b"c".to_vec(),
        origin: ORIGIN.to_string(),
        rp_id: RP_ID.to_string(),
        client_data_json: cdj,
        authenticator_data: ad,
        signature,
        cose_public_key: key.cose(),
        stored_sign_count: 10,
        user_verification: UserVerification::Preferred,
    };
    assert!(matches!(
        verify_assertion(&req),
        Err(crate::MfaError::CounterRegression)
    ));
}

#[test]
fn assertion_allows_zero_counter_when_unsupported() {
    // Both counters zero → authenticator has no counter → accepted.
    let key = Es256Key::new();
    let ad = auth_data(RP_ID, 0x01 | 0x04, 0, &[]);
    let cdj = client_data("webauthn.get", b"c", ORIGIN);
    let signature = key.sign(&signed_message(&ad, &cdj));
    let req = AssertionRequest {
        challenge: b"c".to_vec(),
        origin: ORIGIN.to_string(),
        rp_id: RP_ID.to_string(),
        client_data_json: cdj,
        authenticator_data: ad,
        signature,
        cose_public_key: key.cose(),
        stored_sign_count: 0,
        user_verification: UserVerification::Preferred,
    };
    assert_eq!(verify_assertion(&req).unwrap().new_sign_count, 0);
}

#[test]
fn assertion_rejects_cross_key_signature() {
    // A signature made by a different key must not verify against the stored key.
    let (mut req, _) = good_es256_assertion();
    let other = EdKey::new();
    req.cose_public_key = other.cose();
    // Now the stored key is EdDSA but the signature is ES256 DER → rejected.
    assert!(verify_assertion(&req).is_err());
}

// ── COSE decode ──────────────────────────────────────────────────────────────

#[test]
fn cose_decode_es256_and_eddsa() {
    let key = Es256Key::new();
    match CoseKey::from_cbor(&key.cose()).unwrap() {
        CoseKey::Es256 { x, y } => {
            assert_eq!(x, key.x);
            assert_eq!(y, key.y);
        }
        _ => panic!("expected ES256"),
    }
    let ed = EdKey::new();
    match CoseKey::from_cbor(&ed.cose()).unwrap() {
        CoseKey::EdDsa { x } => assert_eq!(x, ed.x),
        _ => panic!("expected EdDSA"),
    }
}

#[test]
fn cose_decode_rejects_unsupported_alg() {
    // kty EC2(2), alg ES512(-36), crv P-256(1) — structurally complete but the
    // algorithm is not one we verify.
    let mut v = vec![0xa3];
    v.extend_from_slice(&[0x01, 0x02]); // kty = 2
    v.extend_from_slice(&[0x03, 0x38, 0x23]); // alg = -36
    v.extend_from_slice(&[0x20, 0x01]); // crv = 1
    assert!(matches!(
        CoseKey::from_cbor(&v),
        Err(crate::MfaError::UnsupportedAlg)
    ));
}
