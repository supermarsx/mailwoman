//! t32-e41 behaviour checks that use only the API that existed before e41, so the
//! same file runs against the old code (where each test fails) and the new code.
//! Fixtures: fixtures/crypto/smime/e41/ (scripts/smime/gen-fixtures.sh, OpenSSL 3.5).

use mw_crypto::{native, smime};

fn fixture(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/crypto/smime/e41")
        .join(rel)
}

fn bytes(rel: &str) -> Vec<u8> {
    std::fs::read(fixture(rel)).expect(rel)
}

fn text(rel: &str) -> String {
    std::fs::read_to_string(fixture(rel)).expect(rel)
}

/// D-15: a detached (`multipart/signed`) signature is checked over the signed
/// part, not over the empty encapsulated content.
#[test]
fn detached_signature_verifies_over_the_supplied_content() {
    let v = native::verify_signature(
        "smime",
        &bytes("content.txt"),
        &bytes("alice-detached.p7s"),
        "",
    );
    assert_eq!(v.status, "verified", "{v:?}");
    let tampered =
        native::verify_signature("smime", b"other content", &bytes("alice-detached.p7s"), "");
    assert_eq!(tampered.status, "invalid", "{tampered:?}");
}

/// ECDSA P-384 and Ed25519 signers (RFC 8419) verify.
#[test]
fn p384_and_ed25519_signatures_verify() {
    for name in ["p384", "ed"] {
        let v = native::verify_signature(
            "smime",
            &bytes("content.txt"),
            &bytes(&format!("{name}-detached.p7s")),
            "",
        );
        assert_eq!(v.status, "verified", "{name}: {v:?}");
    }
}

/// RSASSA-PSS signatures verify.
#[test]
fn pss_signature_verifies() {
    let v = native::verify_signature("smime", &bytes("content.txt"), &bytes("alice-pss.p7s"), "");
    assert_eq!(v.status, "verified", "{v:?}");
}

/// The verdict names the signer's real key algorithm, not a constant.
#[test]
fn verdict_names_the_real_algorithm() {
    let v = smime::verify(&bytes("alice-opaque.p7m")).expect("verify");
    assert_eq!(v.algorithm.as_deref(), Some("rsa-2048"), "{v:?}");
    assert!(v.key_expires_at.is_some(), "{v:?}");
}

/// RSA-OAEP key transport is read.
#[test]
fn oaep_enveloped_data_decrypts() {
    let pt = smime::decrypt(&bytes("to-alice-oaep.p7m"), &text("alice.key"), "").expect("decrypt");
    assert_eq!(pt, bytes("content.txt"));
}

/// ECDH key agreement (RFC 5753) recipients are read, P-256 and P-384.
#[test]
fn key_agreement_enveloped_data_decrypts() {
    for name in ["ecc", "p384"] {
        let pt = smime::decrypt(
            &bytes(&format!("to-{name}.p7m")),
            &text(&format!("{name}.key")),
            "",
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(pt, bytes("content.txt"));
    }
}

/// Encrypting to an elliptic-curve certificate works (and opens again).
#[test]
fn encrypt_to_ec_certificate() {
    let der = smime::encrypt(b"to ecc", &[text("ecc.pem")]).expect("encrypt");
    let pt = smime::decrypt(&der, &text("ecc.key"), "").expect("decrypt");
    assert_eq!(pt, b"to ecc");
}

/// A PKCS#12 file with an ECDSA key imports.
#[test]
fn ec_pkcs12_imports() {
    let r = smime::import_pkcs12(&bytes("ecc.p12"), "test").expect("import");
    assert_eq!(r.algorithm, "ecdsa-p256");
}
