//! t32-e37 behaviour checks that use only the API that existed before e37, so the
//! same file runs against the old code (where each test fails) and the new code.
//!
//! Fixtures: fixtures/crypto/pgp/e37/ (scripts/pgp/gen-fixtures.sh, GnuPG 2.4) and
//! the older fixtures/crypto/pgp/gnupg-*.asc Bob key.

use ::pgp::composed::Deserializable;
use mw_crypto::pgp;

fn fixture(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/crypto/pgp")
        .join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(fixture(rel)).expect(rel)
}

fn read_bytes(rel: &str) -> Vec<u8> {
    std::fs::read(fixture(rel)).expect(rel)
}

/// A signature made by a signing subkey (GnuPG's default for an RSA key with a
/// certify-only primary) verifies against the certificate.
#[test]
fn signing_subkey_signature_verifies() {
    let v = pgp::verify_detached(
        &read_bytes("e37/rsa-signsub-data.txt"),
        &read("e37/rsa-signsub-data.sig"),
        &read("e37/rsa-signsub-public.asc"),
    )
    .expect("verify");
    assert_eq!(v.status, "verified", "{v:?}");
}

/// The verdict names the key's real algorithm and creation time, not a constant.
#[test]
fn verdict_reports_real_algorithm_and_creation() {
    let v = pgp::verify_detached(
        &read_bytes("e37/rsa-signsub-data.txt"),
        &read("e37/rsa-signsub-data.sig"),
        &read("e37/rsa-signsub-public.asc"),
    )
    .expect("verify");
    assert_eq!(v.algorithm.as_deref(), Some("rsa-3072"), "{v:?}");
    assert!(v.key_created_at.is_some(), "{v:?}");
}

/// A signature by a revoked key is not "verified".
#[test]
fn revoked_key_signature_is_not_verified() {
    let v = pgp::verify_detached(
        &read_bytes("e37/revoked-data.txt"),
        &read("e37/revoked-data.sig"),
        &read("e37/revoked-public.asc"),
    )
    .expect("verify");
    assert_ne!(v.status, "verified", "{v:?}");
    assert_eq!(v.revocation_status.as_deref(), Some("revoked"), "{v:?}");
}

/// A SHA-1 data signature is refused (RFC 9580 §9.5).
#[test]
fn sha1_signature_is_refused() {
    let v = pgp::verify_detached(
        &read_bytes("e37/sha1-data.txt"),
        &read("e37/sha1-data.sig"),
        &read("gnupg-public.asc"),
    )
    .expect("verify");
    assert_eq!(v.status, "invalid", "{v:?}");
}

/// Nothing is encrypted to a revoked key.
#[test]
fn encrypt_to_revoked_key_is_refused() {
    let r = pgp::encrypt(b"x", &[read("e37/revoked-public.asc")], None);
    assert!(r.is_err(), "encrypted to a revoked key");
}

/// Nothing is encrypted to an expired key.
#[test]
fn encrypt_to_expired_key_is_refused() {
    let r = pgp::encrypt(b"x", &[read("e37/expired-public.asc")], None);
    assert!(r.is_err(), "encrypted to an expired key");
}

/// The SEIPD packet version in an armored message.
fn seipd_versions(armored: &str) -> Vec<usize> {
    use ::pgp::packet::{Packet, PacketParser};
    use std::io::Read;
    let mut d = ::pgp::armor::Dearmor::new(armored.as_bytes());
    let mut bytes = Vec::new();
    d.read_to_end(&mut bytes).expect("dearmor");
    PacketParser::new(&bytes[..])
        .flatten()
        .filter_map(|p| match p {
            Packet::SymEncryptedProtectedData(d) => Some(d.version()),
            _ => None,
        })
        .collect()
}

/// A message to a v4 key without SEIPDv2 support (GnuPG's) uses SEIPDv1, which
/// GnuPG 2.4 can read; SEIPDv2 would be unreadable for it.
#[test]
fn encrypt_to_gnupg_v4_key_uses_seipd_v1() {
    let ct = pgp::encrypt(b"to gnupg", &[read("gnupg-public.asc")], None).expect("encrypt");
    assert_eq!(seipd_versions(&ct), vec![1]);
}

/// A message to v6 keys keeps SEIPDv2.
#[test]
fn encrypt_to_v6_key_uses_seipd_v2() {
    let k = pgp::generate_key("V6 <v6@example.com>", "pw").expect("keygen");
    let ct = pgp::encrypt(b"to v6", &[k.public_key_armored], None).expect("encrypt");
    assert_eq!(seipd_versions(&ct), vec![2]);
}

/// A generated v6 key advertises SEIPDv2 in its Features (RFC 9580 §5.2.3.32).
#[test]
fn generated_key_advertises_seipd_v2() {
    let k = pgp::generate_key("F <f@example.com>", "pw").expect("keygen");
    let (cert, _) =
        ::pgp::composed::SignedPublicKey::from_string(&k.public_key_armored).expect("parse");
    let advertised = cert
        .details
        .direct_signatures
        .iter()
        .chain(cert.details.users.iter().flat_map(|u| u.signatures.iter()))
        .filter_map(|s| s.features())
        .any(|f| f.seipd_v2());
    assert!(advertised, "no SEIPDv2 feature on the generated key");
}

/// An imported key reports its own creation and expiry times.
#[test]
fn parse_key_reads_creation_and_expiry() {
    let (key, _) = pgp::parse_key(&read("e37/expired-public.asc"), vec![]).expect("parse");
    assert!(key.created_at.starts_with("2020-01-01"), "{key:?}");
    assert!(
        key.expires_at
            .as_deref()
            .is_some_and(|e| e.starts_with("2020-12-31")),
        "{key:?}"
    );
}
