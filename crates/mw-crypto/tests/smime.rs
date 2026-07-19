//! S/MIME acceptance (plan §3 e1): sign→verify, encrypt→decrypt, PKCS#12 import,
//! cert harvest, and **interop** — verify an openssl-`cms`-signed message and decrypt
//! an openssl-`cms`-encrypted (Outlook-style RSA + AES-256-CBC) message, recorded in
//! fixtures/crypto/smime.

use base64::Engine;
use mw_crypto::smime;

fn fixture(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/crypto/smime")
        .join(rel)
}

fn read(rel: &str) -> Vec<u8> {
    std::fs::read(fixture(rel)).unwrap_or_else(|e| panic!("fixture {rel}: {e}"))
}
fn read_str(rel: &str) -> String {
    std::fs::read_to_string(fixture(rel)).unwrap_or_else(|e| panic!("fixture {rel}: {e}"))
}

/// The .p12 → an S/MIME `encryptedPrivateBundle` + cert PEM we can operate with.
fn import() -> smime::Pkcs12Import {
    smime::import_pkcs12(&read("alice.p12"), "test").expect("pkcs12 import")
}

#[test]
fn pkcs12_import() {
    let imp = import();
    assert!(imp.cert_pem.contains("BEGIN CERTIFICATE"));
    assert!(
        imp.encrypted_private_bundle
            .contains("ENCRYPTED PRIVATE KEY")
    );
    assert_eq!(imp.fingerprint.len(), 64); // SHA-256 hex
}

#[test]
fn sign_then_verify() {
    let imp = import();
    let cert = read_str("alice.crt.pem");
    let b64 =
        smime::sign(b"hello smime", &cert, &imp.encrypted_private_bundle, "test").expect("sign");
    let der = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .unwrap();
    assert_eq!(smime::verify(&der).unwrap().status, "verified");
}

#[test]
fn encrypt_then_decrypt() {
    let imp = import();
    let cert = read_str("alice.crt.pem");
    let ct = smime::encrypt(b"round trip body", &[cert]).expect("encrypt");
    let pt = smime::decrypt(&ct, &imp.encrypted_private_bundle, "test").expect("decrypt");
    assert_eq!(pt, b"round trip body");
}

/// C4: AES-256-GCM AuthEnvelopedData round-trips (RSA key transport + GCM content).
#[test]
fn authenveloped_gcm_encrypt_then_decrypt() {
    let imp = import();
    let cert = read_str("alice.crt.pem");
    let ct = smime::encrypt_authenveloped(b"aead round trip", &[cert]).expect("gcm encrypt");
    let pt = smime::decrypt_authenveloped(&ct, &imp.encrypted_private_bundle, "test")
        .expect("gcm decrypt");
    assert_eq!(pt, b"aead round trip");
}

/// C4: a tampered GCM ciphertext fails the AEAD tag check (does not silently decrypt).
#[test]
fn authenveloped_gcm_tamper_rejected() {
    let imp = import();
    let cert = read_str("alice.crt.pem");
    let mut ct = smime::encrypt_authenveloped(b"integrity matters", &[cert]).expect("gcm encrypt");
    // Flip a byte late in the DER (inside the encrypted content / tag region).
    let n = ct.len();
    ct[n - 20] ^= 0x01;
    assert!(smime::decrypt_authenveloped(&ct, &imp.encrypted_private_bundle, "test").is_err());
}

/// Interop: verify a message signed by openssl `cms -sign`.
#[test]
fn openssl_signed_interop() {
    let signed = read("signed.der");
    assert_eq!(smime::verify(&signed).unwrap().status, "verified");
}

/// Interop: decrypt an Outlook-style enveloped message (openssl `cms -encrypt`,
/// RSA key transport + AES-256-CBC content).
#[test]
fn openssl_enveloped_interop() {
    let imp = import();
    let env = read("enveloped.der");
    let pt = smime::decrypt(&env, &imp.encrypted_private_bundle, "test").expect("decrypt");
    assert_eq!(pt, b"Encrypted S/MIME body for interop.");
}

#[test]
fn harvest_certs_from_signed() {
    let harvested = smime::harvest_certs(&read("signed.der")).expect("harvest");
    assert_eq!(harvested.len(), 1);
    assert_eq!(harvested[0].kind, "smime");
    assert!(harvested[0].cert_pem.is_some());
}
