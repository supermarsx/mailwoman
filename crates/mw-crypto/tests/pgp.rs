//! OpenPGP acceptance (plan §3 e1): v6 keygen, encrypt→decrypt, sign→verify,
//! protected-subject round-trip, Autocrypt header + backup, and **GnuPG interop**
//! (decrypt+verify a message produced by a real `gpg`, recorded in fixtures/crypto).

use mw_crypto::pgp;

fn fixture(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/crypto")
        .join(rel)
}

#[test]
fn keygen_v6_and_roundtrip_signed() {
    let k = pgp::generate_key("Alice <alice@example.com>", "pw123").expect("keygen");
    assert!(k.public_key_armored.contains("BEGIN PGP PUBLIC KEY"));
    assert!(k.encrypted_private_bundle.contains("BEGIN PGP PRIVATE KEY"));
    assert_eq!(k.fingerprint.len(), 64); // v6 fingerprint = 32 bytes hex

    let ct = pgp::encrypt(
        b"hello secret",
        std::slice::from_ref(&k.public_key_armored),
        Some((&k.encrypted_private_bundle, "pw123")),
    )
    .expect("encrypt");
    assert!(ct.contains("BEGIN PGP MESSAGE"));

    let out = pgp::decrypt(
        ct.as_bytes(),
        &k.encrypted_private_bundle,
        "pw123",
        Some(&k.public_key_armored),
    )
    .expect("decrypt");
    assert_eq!(out.plaintext, b"hello secret");
    assert_eq!(out.signature.status, "verified");
}

#[test]
fn wrong_passphrase_fails() {
    let k = pgp::generate_key("Bob <bob@example.com>", "correct").expect("keygen");
    let ct =
        pgp::encrypt(b"x", std::slice::from_ref(&k.public_key_armored), None).expect("encrypt");
    assert!(pgp::decrypt(ct.as_bytes(), &k.encrypted_private_bundle, "wrong", None).is_err());
}

#[test]
fn detached_sign_verify_and_tamper() {
    let k = pgp::generate_key("C <c@example.com>", "pw").expect("keygen");
    let sig = pgp::sign_detached(b"data to sign", &k.encrypted_private_bundle, "pw").expect("sign");
    assert_eq!(
        pgp::verify_detached(b"data to sign", &sig, &k.public_key_armored)
            .unwrap()
            .status,
        "verified"
    );
    assert_eq!(
        pgp::verify_detached(b"tampered", &sig, &k.public_key_armored)
            .unwrap()
            .status,
        "invalid"
    );
}

#[test]
fn clear_sign_roundtrip_body_preserved_and_verifies() {
    let k = pgp::generate_key("F <f@example.com>", "pw").expect("keygen");
    let body = "Hello, clear-signed world\n- a dash line\nlast";
    let armored = pgp::clear_sign(body, &k.encrypted_private_bundle, "pw").expect("clear_sign");

    // A real cleartext-signature framework: the body stays readable inline and the
    // signature block is present (NOT a bare detached signature that drops the body).
    assert!(armored.contains("-----BEGIN PGP SIGNED MESSAGE-----"));
    assert!(armored.contains("-----BEGIN PGP SIGNATURE-----"));
    assert!(armored.contains("-----END PGP SIGNATURE-----"));

    // The signature verifies against the signer's key AND the recovered cleartext is
    // exactly the original body.
    let (verdict, text) =
        pgp::verify_clear_signed(&armored, &k.public_key_armored).expect("verify clear-signed");
    assert_eq!(verdict.status, "verified");
    // The recovered content is the original body (de-dash-escaped), CRLF-normalized
    // as the cleartext-signature framework canonicalizes it — the body is NOT lost.
    assert_eq!(text, "Hello, clear-signed world\r\n- a dash line\r\nlast");
    // The armored transport form dash-escapes the leading-dash line ("- - a dash…").
    assert!(armored.contains("- - a dash line"));
}

#[test]
fn protected_subject_roundtrip() {
    let k = pgp::generate_key("D <d@example.com>", "pw").expect("keygen");
    let wrapped = pgp::wrap_protected_headers("Secret Subject", b"body text");
    let ct =
        pgp::encrypt(&wrapped, std::slice::from_ref(&k.public_key_armored), None).expect("encrypt");
    let out =
        pgp::decrypt(ct.as_bytes(), &k.encrypted_private_bundle, "pw", None).expect("decrypt");
    assert_eq!(out.subject.as_deref(), Some("Secret Subject"));
    assert_eq!(out.plaintext, b"body text");
}

#[test]
fn autocrypt_header_and_backup() {
    let k = pgp::generate_key("E <e@example.com>", "pw").expect("keygen");
    let header =
        pgp::autocrypt_header(&k.public_key_armored, "e@example.com", true).expect("header");
    assert!(header.contains("addr=e@example.com"));
    assert!(header.contains("prefer-encrypt=mutual"));
    assert!(header.contains("keydata="));

    let parsed = pgp::parse_autocrypt_header(&header).expect("parse header");
    assert_eq!(parsed.fingerprint, k.fingerprint);
    assert!(parsed.autocrypt);

    let asm = pgp::autocrypt_setup_message(&k.encrypted_private_bundle).expect("asm");
    assert!(asm.contains("BEGIN PGP PRIVATE KEY"));
}

/// C3: the full Autocrypt Setup Message wire format round-trips under the setup
/// code, carries the `Passphrase-Format`/`Passphrase-Begin` armor headers, and a
/// wrong code fails to decrypt.
#[test]
fn autocrypt_setup_message_full_roundtrip() {
    let k = pgp::generate_key("G <g@example.com>", "userpass").expect("keygen");
    let code = pgp::generate_setup_code();
    // numeric9x4: 9 groups of 4 digits.
    assert_eq!(code.len(), 9 * 4 + 8);
    assert_eq!(code.split('-').count(), 9);
    assert!(
        code.split('-')
            .all(|g| g.len() == 4 && g.chars().all(|c| c.is_ascii_digit()))
    );

    let payload =
        pgp::autocrypt_setup_message_full(&k.encrypted_private_bundle, &code).expect("asm full");
    assert!(payload.contains("-----BEGIN PGP MESSAGE-----"));
    assert!(payload.contains("Passphrase-Format: numeric9x4"));
    assert!(payload.contains("Passphrase-Begin: "));

    // Recover the inner (still passphrase-locked) bundle with the setup code.
    let recovered = pgp::decrypt_autocrypt_setup_message(&payload, &code).expect("asm decrypt");
    assert!(recovered.contains("BEGIN PGP PRIVATE KEY"));
    // The recovered bundle still unlocks the key with the user passphrase.
    let sig = pgp::sign_detached(b"post-transfer", &recovered, "userpass").expect("sign recovered");
    assert_eq!(
        pgp::verify_detached(b"post-transfer", &sig, &k.public_key_armored)
            .unwrap()
            .status,
        "verified"
    );

    // Wrong setup code cannot decrypt.
    assert!(
        pgp::decrypt_autocrypt_setup_message(
            &payload,
            "0000-0000-0000-0000-0000-0000-0000-0000-0000"
        )
        .is_err()
    );

    // The MIME framing carries the Autocrypt-Setup-Message marker + attachment.
    let mime = pgp::autocrypt_setup_message_mime("g@example.com", &payload);
    assert!(mime.contains("Autocrypt-Setup-Message: v1"));
    assert!(mime.contains(pgp::AUTOCRYPT_SETUP_CONTENT_TYPE));
}

#[test]
fn tofu_eval_transitions() {
    assert_eq!(pgp::tofu_eval(None, "AABB"), ("tofu".into(), false));
    assert_eq!(pgp::tofu_eval(Some("aabb"), "AABB"), ("tofu".into(), false));
    assert_eq!(
        pgp::tofu_eval(Some("AABB"), "CCDD"),
        ("unverified".into(), true)
    );
}

#[test]
fn wkd_url_derivation() {
    let url = pgp::wkd_url("Joe.Doe@Example.ORG", true).unwrap();
    assert!(
        url.starts_with("https://openpgpkey.example.org/.well-known/openpgpkey/example.org/hu/")
    );
    assert!(url.contains("?l=joe.doe"));
    let direct = pgp::wkd_url("Joe.Doe@Example.ORG", false).unwrap();
    assert!(direct.starts_with("https://example.org/.well-known/openpgpkey/hu/"));
}

/// Interop: decrypt + verify a message generated by a real GnuPG (recorded).
#[test]
fn gnupg_interop_decrypt_verify() {
    let secret = std::fs::read_to_string(fixture("pgp/gnupg-secret.asc")).expect("secret fixture");
    let public = std::fs::read_to_string(fixture("pgp/gnupg-public.asc")).expect("public fixture");
    let msg = std::fs::read(fixture("pgp/gnupg-message.asc")).expect("message fixture");

    let out = pgp::decrypt(&msg, &secret, "interop-pass", Some(&public)).expect("decrypt gnupg");
    assert!(String::from_utf8_lossy(&out.plaintext).contains("decrypt me with rPGP"));
    assert_eq!(out.signature.status, "verified");
}
