//! The native (server-side) crypto facade — `mw-crypto/src/native.rs`, the entry
//! point `mw-engine` calls from `SecurityVerdict/get` and `CryptoKey/lookup`.
//!
//! It was the least-covered file in the crate (18% of lines) despite sitting on
//! the hostile-input path: every byte it sees came off the wire in a received
//! message. What matters about it is that a *failure* to verify is surfaced as a
//! renderable 3-state verdict rather than an error or a panic — so these tests
//! are mostly negative paths, and they assert the whole verdict rather than just
//! its `status`, so a mutation that flips one field is caught.

use mw_crypto::{SignatureVerdict, native, pgp};

/// The exact shape the engine must get back when nothing could be verified.
/// Asserting field-by-field (not just `status`) pins `none_verdict` against a
/// silent change to, say, `key_changed`.
fn assert_is_none_verdict(v: &SignatureVerdict, kind: &str) {
    assert_eq!(v.kind, kind);
    assert_eq!(v.status, "none");
    assert_eq!(v.signer_key_id, None);
    assert_eq!(v.algorithm, None);
    assert_eq!(v.key_created_at, None);
    assert_eq!(v.key_expires_at, None);
    assert_eq!(v.chain_status, None);
    assert_eq!(v.revocation_status, None);
    assert!(!v.key_changed);
}

/// A signature blob that is not valid UTF-8 cannot be an armored PGP signature.
/// It must not panic on the `from_utf8` and must not be reported as verified.
#[test]
fn pgp_signature_that_is_not_utf8_is_a_none_verdict() {
    let not_utf8 = [0xff, 0xfe, 0x00, 0x80, 0x41];
    let v = native::verify_signature("pgp", b"payload", &not_utf8, "");
    assert_is_none_verdict(&v, "pgp");
}

/// Well-formed UTF-8 that is not a signature, and an empty signer key, degrade
/// to the same verdict rather than an error.
#[test]
fn pgp_garbage_signature_is_a_none_verdict() {
    for sig in [
        b"".as_slice(),
        b"not a signature",
        b"-----BEGIN PGP SIGNATURE-----\ntruncated",
        b"-----BEGIN PGP SIGNATURE-----\n\nAAAA\n-----END PGP SIGNATURE-----\n",
    ] {
        let v = native::verify_signature("pgp", b"payload", sig, "");
        assert_is_none_verdict(&v, "pgp");
    }
}

/// A syntactically valid signature checked against a key it was not made with
/// must not come back `verified`.
#[test]
fn pgp_signature_against_the_wrong_key_is_not_verified() {
    let signer = pgp::generate_key("Signer <signer@example.test>", "pw").expect("keygen");
    let other = pgp::generate_key("Other <other@example.test>", "pw").expect("keygen");
    let sig = pgp::sign_detached(b"payload", &signer.encrypted_private_bundle, "pw").expect("sign");

    let good = native::verify_signature(
        "pgp",
        b"payload",
        sig.as_bytes(),
        &signer.public_key_armored,
    );
    assert_eq!(good.status, "verified", "control case must verify");
    assert_eq!(good.kind, "pgp");

    let wrong_key =
        native::verify_signature("pgp", b"payload", sig.as_bytes(), &other.public_key_armored);
    assert_ne!(wrong_key.status, "verified", "{wrong_key:?}");

    let wrong_data = native::verify_signature(
        "pgp",
        b"tampered",
        sig.as_bytes(),
        &signer.public_key_armored,
    );
    assert_ne!(wrong_data.status, "verified", "{wrong_data:?}");
}

/// A CMS blob that will not parse is a `none` S/MIME verdict, not an error.
#[test]
fn smime_garbage_cms_is_a_none_verdict() {
    for blob in [b"".as_slice(), b"not DER at all", &[0x30, 0x82, 0xff, 0xff]] {
        let v = native::verify_signature("smime", b"", blob, "");
        assert_is_none_verdict(&v, "smime");
    }
}

/// An unrecognised `kind` is echoed back on the verdict rather than being
/// silently normalised or panicked on — the engine renders whatever it asked
/// about.
#[test]
fn an_unknown_kind_echoes_itself_on_the_verdict() {
    for kind in ["", "PGP", "s/mime", "made-up"] {
        let v = native::verify_signature(kind, b"data", b"sig", "key");
        assert_is_none_verdict(&v, kind);
    }
}

/// Harvesting from bytes that are neither CMS nor an Autocrypt header yields no
/// keys — never a partial or placeholder key that would then enter the keyring.
#[test]
fn harvesting_from_junk_yields_no_keys() {
    for blob in [
        b"".as_slice(),
        b"\x00\x01\x02\x03",
        b"addr=nobody@example.test; keydata=not-base64!!",
        b"-----BEGIN PGP PUBLIC KEY BLOCK-----\ntruncated",
    ] {
        assert!(
            native::harvest_keys(blob).is_empty(),
            "harvested something from {blob:?}"
        );
    }
}

/// The Autocrypt fallback path: a real `Autocrypt:` header value round-trips
/// into exactly one public key carrying the sender's address.
#[test]
fn harvesting_an_autocrypt_header_yields_the_sender_key() {
    let k = pgp::generate_key("Auto <auto@example.test>", "pw").expect("keygen");
    let header = pgp::autocrypt_header(&k.public_key_armored, "auto@example.test", true)
        .expect("autocrypt header");

    let keys = native::harvest_keys(header.as_bytes());
    assert_eq!(keys.len(), 1, "{keys:?}");
    assert_eq!(keys[0].kind, "pgp");
    assert!(!keys[0].is_own, "a harvested key is never an own key");
    assert!(!keys[0].has_private, "no private half may be harvested");
    assert!(keys[0].encrypted_private_backup.is_none());
    assert!(
        keys[0].addresses.iter().any(|a| a == "auto@example.test"),
        "{:?}",
        keys[0].addresses
    );
}

/// PQC store-key wrapping rejects the malformed inputs a corrupted `key_material`
/// row could present, rather than panicking on a slice or silently returning
/// wrong key bytes.
#[test]
fn store_key_wrap_rejects_malformed_material() {
    let kp = native::generate_store_recipient();
    let seal = b"seal-key-material-32-bytes-long!!";

    // A recipient public of the wrong length is refused up front.
    assert!(native::wrap_store_key(seal, &[]).is_err());
    assert!(native::wrap_store_key(seal, &kp.public[..kp.public.len() - 1]).is_err());
    let mut too_long = kp.public.clone();
    too_long.push(0);
    assert!(native::wrap_store_key(seal, &too_long).is_err());

    let wrapped = native::wrap_store_key(seal, &kp.public).expect("wrap");

    // A recipient secret of the wrong length, and a truncated blob, are refused.
    assert!(native::unwrap_store_key(&wrapped, &[]).is_err());
    assert!(native::unwrap_store_key(&wrapped, &kp.secret[..8]).is_err());
    assert!(native::unwrap_store_key(&wrapped[..16], &kp.secret).is_err());
    assert!(native::unwrap_store_key(&[], &kp.secret).is_err());

    // Flipping a bit anywhere in the blob fails the AEAD rather than returning
    // altered key bytes.
    for offset in [0, 40, wrapped.len() - 1] {
        let mut bad = wrapped.clone();
        bad[offset] ^= 0x01;
        assert!(
            native::unwrap_store_key(&bad, &kp.secret).is_err(),
            "tamper at {offset} was accepted"
        );
    }

    assert_eq!(
        native::unwrap_store_key(&wrapped, &kp.secret).unwrap(),
        seal
    );
}

/// Two generated recipients are distinct — a constant-returning keygen would
/// make every store seal wrappable by anyone.
#[test]
fn generated_store_recipients_are_distinct() {
    let a = native::generate_store_recipient();
    let b = native::generate_store_recipient();
    assert_ne!(a.public, b.public);
    assert_ne!(a.secret, b.secret);
    assert_eq!(a.public.len(), b.public.len());
    assert_eq!(a.secret.len(), b.secret.len());
}
