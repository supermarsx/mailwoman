//! t32-e37 OpenPGP core: key profiles and inspection, recipient-aware encryption,
//! signature states, key management, Autocrypt, WKD, attached keys.

use mw_crypto::pgp::autocrypt::{self, PeerState};
use mw_crypto::pgp::manage::{self, RevocationReason, SubkeyPurpose};
use mw_crypto::pgp::{self, GenerateOptions, KeyProfile};
use mw_crypto::verdict;

const PW: &str = "pw-e37";

fn fixture(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/crypto/pgp")
        .join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(fixture(rel)).expect(rel)
}

fn now() -> u64 {
    mw_crypto::clock::now_secs()
}

fn v6(uid: &str) -> pgp::GeneratedKey {
    pgp::generate_key(uid, PW).expect("keygen v6")
}

fn compatible(uid: &str) -> pgp::GeneratedKey {
    pgp::generate_key_with(
        uid,
        PW,
        &GenerateOptions {
            profile: KeyProfile::Compatible,
            expires_in_secs: None,
        },
    )
    .expect("keygen compatible")
}

// ── profiles and inspection ──────────────────────────────────────────────────

#[test]
fn v6_profile_inspects_as_v6_with_aead() {
    let k = v6("Ann <Ann@Example.com>");
    let info = pgp::inspect_key(&k.public_key_armored).expect("inspect");
    assert_eq!(info.version, 6);
    assert_eq!(info.algorithm, "ed25519");
    assert_eq!(info.state, "valid");
    assert!(info.seipd_v2);
    assert!(info.can_encrypt && info.can_sign);
    assert!(!info.has_secret);
    assert_eq!(info.user_ids.len(), 1);
    assert_eq!(info.user_ids[0].address.as_deref(), Some("ann@example.com"));
    assert!(info.user_ids[0].primary);
    assert_eq!(info.subkeys.len(), 1);
    assert_eq!(info.subkeys[0].algorithm, "x25519");
    assert!(
        info.subkeys[0]
            .flags
            .contains(&"encrypt-communications".to_string())
    );
    assert!(info.expires_at.is_none());

    let secret = pgp::inspect_key(&k.encrypted_private_bundle).expect("inspect secret");
    assert!(secret.has_secret);
    assert_eq!(secret.fingerprint, info.fingerprint);
}

#[test]
fn compatible_profile_is_v4_eddsa_without_aead() {
    let k = compatible("Cat <cat@example.com>");
    let info = pgp::inspect_key(&k.public_key_armored).expect("inspect");
    assert_eq!(info.version, 4);
    assert_eq!(info.algorithm, "eddsa-legacy");
    assert_eq!(info.subkeys[0].algorithm, "ecdh-cv25519");
    assert!(!info.seipd_v2);
    assert_eq!(k.fingerprint.len(), 40);
}

#[test]
fn expiry_at_generation_and_after() {
    let k = pgp::generate_key_with(
        "Exp <exp@example.com>",
        PW,
        &GenerateOptions {
            profile: KeyProfile::V6,
            expires_in_secs: Some(3600),
        },
    )
    .expect("keygen");
    let info = pgp::inspect_key(&k.public_key_armored).expect("inspect");
    assert_eq!(info.state, "valid");
    let exp = mw_crypto::clock::parse_rfc3339(info.expires_at.as_deref().expect("expiry"))
        .expect("rfc3339");
    assert!(exp > now() && exp <= now() + 3600);
    assert!(info.subkeys[0].expires_at.is_some());

    let later = pgp::inspect_key_at(&k.public_key_armored, exp + 1).expect("inspect later");
    assert_eq!(later.state, "expired");
    assert!(!later.can_encrypt);
    let r = pgp::encrypt_at(
        b"x",
        std::slice::from_ref(&k.public_key_armored),
        None,
        exp + 1,
    );
    assert!(r.is_err());
}

#[test]
fn fixture_keys_inspect_with_their_states() {
    let revoked = pgp::inspect_key(&read("e37/revoked-public.asc")).expect("revoked");
    assert_eq!(revoked.state, "revoked");
    let rev = revoked.revocation.expect("revocation");
    assert!(rev.hard);
    let expired = pgp::inspect_key(&read("e37/expired-public.asc")).expect("expired");
    assert_eq!(expired.state, "expired");
    assert!(expired.created_at.starts_with("2020-01-01"));
    let rsa = pgp::inspect_key(&read("e37/rsa-signsub-public.asc")).expect("rsa");
    assert_eq!(rsa.algorithm, "rsa-3072");
    assert_eq!(rsa.flags, vec!["certify".to_string()]);
    assert!(rsa.can_sign && rsa.can_encrypt);
    assert_eq!(rsa.subkeys.len(), 2);
}

// ── encryption ───────────────────────────────────────────────────────────────

#[test]
fn seipd_version_follows_the_recipients() {
    let a = v6("A <a@example.com>");
    let b = compatible("B <b@example.com>");

    let only_v6 =
        pgp::encrypt_with(b"v6", std::slice::from_ref(&a.public_key_armored), None).unwrap();
    assert_eq!(only_v6.seipd_version, 2);
    let mixed = pgp::encrypt_with(
        b"mixed",
        &[a.public_key_armored.clone(), b.public_key_armored.clone()],
        Some((&a.encrypted_private_bundle, PW)),
    )
    .unwrap();
    assert_eq!(mixed.seipd_version, 1);

    // Both recipients read the mixed message; the signature checks out.
    for k in [&a, &b] {
        let out = pgp::decrypt_with(
            mixed.armored.as_bytes(),
            &k.encrypted_private_bundle,
            PW,
            &[&a.public_key_armored],
        )
        .expect("decrypt");
        assert_eq!(out.plaintext, b"mixed");
        assert!(!out.aead);
        assert_eq!(out.verification.state, verdict::VALID);
        assert_eq!(out.signature.status, "verified");
    }
    let out = pgp::decrypt(
        only_v6.armored.as_bytes(),
        &a.encrypted_private_bundle,
        PW,
        None,
    )
    .unwrap();
    assert!(out.aead);
    assert_eq!(out.verification.state, verdict::NOT_SIGNED);
}

#[test]
fn signature_from_unknown_key_is_unknown_key() {
    let a = v6("A <a@example.com>");
    let stranger = v6("S <s@example.com>");
    let ct = pgp::encrypt(
        b"hi",
        std::slice::from_ref(&a.public_key_armored),
        Some((&stranger.encrypted_private_bundle, PW)),
    )
    .unwrap();
    let out = pgp::decrypt_with(
        ct.as_bytes(),
        &a.encrypted_private_bundle,
        PW,
        &[&a.public_key_armored],
    )
    .unwrap();
    assert_eq!(out.verification.state, verdict::UNKNOWN_KEY);
    assert_eq!(out.signature.status, "unverified-key");
}

#[test]
fn rotated_sender_keys_verify_with_either_key() {
    // A sender with two valid keys for one address (old + rotated). The reader
    // passes both; mail signed by either one verifies.
    let old = v6("A <a@example.com>");
    let new = v6("A <a@example.com>");
    let reader = v6("R <r@example.com>");
    let candidates = [
        old.public_key_armored.as_str(),
        new.public_key_armored.as_str(),
    ];
    for signer in [&old, &new] {
        let ct = pgp::encrypt(
            b"rotated",
            std::slice::from_ref(&reader.public_key_armored),
            Some((&signer.encrypted_private_bundle, PW)),
        )
        .unwrap();
        let out = pgp::decrypt_with(
            ct.as_bytes(),
            &reader.encrypted_private_bundle,
            PW,
            &candidates,
        )
        .unwrap();
        assert_eq!(out.plaintext, b"rotated");
        assert_eq!(out.verification.state, verdict::VALID);
        assert_eq!(out.signature.status, "verified");
    }
    // With only the old key, mail from the new key is "signer key not verified".
    let ct = pgp::encrypt(
        b"rotated",
        std::slice::from_ref(&reader.public_key_armored),
        Some((&new.encrypted_private_bundle, PW)),
    )
    .unwrap();
    let out = pgp::decrypt_with(
        ct.as_bytes(),
        &reader.encrypted_private_bundle,
        PW,
        &[&old.public_key_armored],
    )
    .unwrap();
    assert_eq!(out.signature.status, "unverified-key");
}

// ── detached and cleartext verification states ───────────────────────────────

#[test]
fn detached_states() {
    let k = v6("Sig <sig@example.com>");
    let sig = pgp::sign_detached(b"data", &k.encrypted_private_bundle, PW).unwrap();

    let good = pgp::verify_detached_with(b"data", &sig, &[&k.public_key_armored]).unwrap();
    assert_eq!(good.state, verdict::VALID);
    assert_eq!(
        good.signer_fingerprint.as_deref(),
        Some(k.fingerprint.as_str())
    );
    assert_eq!(good.signer_addresses, vec!["sig@example.com".to_string()]);
    assert_eq!(good.signature.algorithm.as_deref(), Some("ed25519"));
    assert!(good.signed_at.is_some());

    let bad = pgp::verify_detached_with(b"datA", &sig, &[&k.public_key_armored]).unwrap();
    assert_eq!(bad.state, verdict::BAD_SIGNATURE);
    assert_eq!(bad.detail.as_deref(), Some("tampered"));

    let other = v6("O <o@example.com>");
    let unknown = pgp::verify_detached_with(b"data", &sig, &[&other.public_key_armored]).unwrap();
    assert_eq!(unknown.state, verdict::UNKNOWN_KEY);
}

#[test]
fn signature_made_while_valid_stays_valid_after_expiry() {
    let data = std::fs::read(fixture("e37/expired-data.txt")).unwrap();
    let v = pgp::verify_detached_with(
        &data,
        &read("e37/expired-data.sig"),
        &[&read("e37/expired-public.asc")],
    )
    .unwrap();
    assert_eq!(v.state, verdict::VALID, "{v:?}");
    assert!(v.signed_at.as_deref().unwrap().starts_with("2020-06-01"));
    assert!(v.signature.key_expires_at.is_some());
}

#[test]
fn signature_made_after_expiry_is_key_expired() {
    use ::pgp::composed::{Deserializable, DetachedSignature, SignedSecretKey};
    // A key that expires one second after creation, then a signature made with
    // rPGP directly (our own signer refuses an expired key, see below).
    let k = pgp::generate_key_with(
        "Short <short@example.com>",
        PW,
        &GenerateOptions {
            profile: KeyProfile::V6,
            expires_in_secs: Some(1),
        },
    )
    .unwrap();
    std::thread::sleep(std::time::Duration::from_secs(3));
    assert!(pgp::sign_detached(b"late", &k.encrypted_private_bundle, PW).is_err());

    let (ssk, _) = SignedSecretKey::from_string(&k.encrypted_private_bundle).unwrap();
    let sig = DetachedSignature::sign_binary_data(
        rand::rngs::OsRng,
        &ssk.primary_key,
        &PW.into(),
        ::pgp::crypto::hash::HashAlgorithm::Sha256,
        &b"late"[..],
    )
    .unwrap()
    .to_armored_string(Default::default())
    .unwrap();
    let v = pgp::verify_detached_with(b"late", &sig, &[&k.public_key_armored]).unwrap();
    assert_eq!(v.state, verdict::KEY_EXPIRED, "{v:?}");
    assert_eq!(v.signature.status, "unverified-key");
}

#[test]
fn cleartext_states() {
    let k = v6("Clear <clear@example.com>");
    let signed = pgp::clear_sign("hello\nworld\n", &k.encrypted_private_bundle, PW).unwrap();
    let (v, text) = pgp::verify_cleartext(&signed, &[&k.public_key_armored]).unwrap();
    assert_eq!(v.state, verdict::VALID);
    assert!(text.contains("hello"));
    let tampered = signed.replace("world", "w0rld");
    let (v, _) = pgp::verify_cleartext(&tampered, &[&k.public_key_armored]).unwrap();
    assert_eq!(v.state, verdict::BAD_SIGNATURE);
}

// ── key management ───────────────────────────────────────────────────────────

#[test]
fn revoke_then_verify_and_encrypt() {
    let k = v6("Rev <rev@example.com>");
    let sig = pgp::sign_detached(b"before", &k.encrypted_private_bundle, PW).unwrap();
    let revoked = manage::revoke(
        &k.encrypted_private_bundle,
        PW,
        RevocationReason::Compromised,
        "lost laptop",
    )
    .unwrap();
    let info = pgp::inspect_key(&revoked.public_key_armored).unwrap();
    assert_eq!(info.state, "revoked");
    let r = info.revocation.unwrap();
    assert_eq!(r.reason, "compromised");
    assert_eq!(r.message, "lost laptop");
    assert!(r.hard);

    let v = pgp::verify_detached_with(b"before", &sig, &[&revoked.public_key_armored]).unwrap();
    assert_eq!(v.state, verdict::REVOKED);
    assert!(
        pgp::encrypt(
            b"x",
            std::slice::from_ref(&revoked.public_key_armored),
            None
        )
        .is_err()
    );
}

#[test]
fn soft_revocation_keeps_older_signatures() {
    let k = v6("Soft <soft@example.com>");
    let sig = pgp::sign_detached(b"old", &k.encrypted_private_bundle, PW).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));
    let revoked = manage::revoke(
        &k.encrypted_private_bundle,
        PW,
        RevocationReason::Superseded,
        "",
    )
    .unwrap();
    let v = pgp::verify_detached_with(b"old", &sig, &[&revoked.public_key_armored]).unwrap();
    assert_eq!(v.state, verdict::VALID, "{v:?}");
    assert_eq!(
        pgp::inspect_key(&revoked.public_key_armored).unwrap().state,
        "revoked"
    );
}

#[test]
fn revocation_certificate_applies_to_the_public_key() {
    let k = compatible("Cert <cert@example.com>");
    let cert = manage::revocation_certificate(
        &k.encrypted_private_bundle,
        PW,
        RevocationReason::Retired,
        "",
    )
    .unwrap();
    assert!(cert.contains("BEGIN PGP PUBLIC KEY BLOCK"));
    let revoked = manage::apply_revocation_certificate(&k.public_key_armored, &cert).unwrap();
    assert_eq!(pgp::inspect_key(&revoked).unwrap().state, "revoked");

    let other = compatible("Other <other@example.com>");
    assert!(manage::apply_revocation_certificate(&other.public_key_armored, &cert).is_err());
}

#[test]
fn set_expiry_extends_and_removes() {
    for k in [v6("E6 <e6@example.com>"), compatible("E4 <e4@example.com>")] {
        let at = now() + 30 * 24 * 3600;
        let m = manage::set_expiry(&k.encrypted_private_bundle, PW, Some(at)).unwrap();
        let info = pgp::inspect_key(&m.public_key_armored).unwrap();
        assert_eq!(
            mw_crypto::clock::parse_rfc3339(info.expires_at.as_deref().unwrap()),
            Some(at)
        );
        assert_eq!(
            mw_crypto::clock::parse_rfc3339(info.subkeys[0].expires_at.as_deref().unwrap()),
            Some(at)
        );
        // The renewed key still encrypts, decrypts and signs.
        let ct = pgp::encrypt(b"after", std::slice::from_ref(&m.public_key_armored), None).unwrap();
        let out = pgp::decrypt(ct.as_bytes(), &m.encrypted_private_bundle, PW, None).unwrap();
        assert_eq!(out.plaintext, b"after");

        let never = manage::set_expiry(&m.encrypted_private_bundle, PW, None).unwrap();
        let info = pgp::inspect_key(&never.public_key_armored).unwrap();
        assert!(info.expires_at.is_none(), "{info:?}");
        assert!(manage::set_expiry(&k.encrypted_private_bundle, "wrong", Some(at)).is_err());
    }
}

#[test]
fn add_and_revoke_subkeys() {
    let k = v6("Sub <sub@example.com>");
    let first_enc = pgp::inspect_key(&k.public_key_armored).unwrap().subkeys[0]
        .fingerprint
        .clone();

    let with_enc = manage::add_subkey(
        &k.encrypted_private_bundle,
        PW,
        SubkeyPurpose::Encrypt,
        None,
    )
    .unwrap();
    let with_sign = manage::add_subkey(
        &with_enc.encrypted_private_bundle,
        PW,
        SubkeyPurpose::Sign,
        None,
    )
    .unwrap();
    let info = pgp::inspect_key(&with_sign.public_key_armored).unwrap();
    assert_eq!(info.subkeys.len(), 3);
    assert!(info.subkeys.iter().all(|s| s.state == "valid"), "{info:?}");
    assert!(info.subkeys[2].flags.contains(&"sign".to_string()));

    let revoked = manage::revoke_subkey(
        &with_sign.encrypted_private_bundle,
        PW,
        &first_enc,
        RevocationReason::Retired,
        "",
    )
    .unwrap();
    let info = pgp::inspect_key(&revoked.public_key_armored).unwrap();
    let first = info
        .subkeys
        .iter()
        .find(|s| s.fingerprint == first_enc)
        .unwrap();
    assert_eq!(first.state, "revoked");

    // New mail goes to the remaining encryption subkey and still decrypts.
    let ct = pgp::encrypt(
        b"rotated",
        std::slice::from_ref(&revoked.public_key_armored),
        None,
    )
    .unwrap();
    let out = pgp::decrypt(ct.as_bytes(), &revoked.encrypted_private_bundle, PW, None).unwrap();
    assert_eq!(out.plaintext, b"rotated");
}

#[test]
fn signing_subkey_of_our_own_key_signs_and_verifies() {
    let k = compatible("Signer <signer@example.com>");
    let m = manage::add_subkey(&k.encrypted_private_bundle, PW, SubkeyPurpose::Sign, None).unwrap();
    // The primary still signs (it is flagged for signing); a signature by it
    // verifies, and so does one checked against the updated public key.
    let sig = pgp::sign_detached(b"d", &m.encrypted_private_bundle, PW).unwrap();
    let v = pgp::verify_detached_with(b"d", &sig, &[&m.public_key_armored]).unwrap();
    assert_eq!(v.state, verdict::VALID);
}

/// Someone binds another person's signing subkey to their own certificate (they
/// can sign the binding, but cannot make the subkey's back-signature). Signatures
/// by that subkey must not count as theirs.
#[test]
fn borrowed_signing_subkey_without_back_signature_is_refused() {
    use ::pgp::composed::{Deserializable, SignedPublicKey, SignedPublicSubKey, SignedSecretKey};
    use ::pgp::packet::{KeyFlags, SignatureConfig, SignatureType, Subpacket, SubpacketData};
    use ::pgp::types::KeyDetails;

    let victim = compatible("Victim <victim@example.com>");
    let victim = manage::add_subkey(
        &victim.encrypted_private_bundle,
        PW,
        SubkeyPurpose::Sign,
        None,
    )
    .unwrap();
    let (victim_pub, _) = SignedPublicKey::from_string(&victim.public_key_armored).unwrap();
    let stolen = victim_pub.public_subkeys.last().unwrap().key.clone();

    // A signature by the victim's signing subkey.
    let (victim_ssk, _) = SignedSecretKey::from_string(&victim.encrypted_private_bundle).unwrap();
    let sub_secret = &victim_ssk.secret_subkeys.last().unwrap().key;
    let sig = ::pgp::composed::DetachedSignature::sign_binary_data(
        rand::rngs::OsRng,
        sub_secret,
        &PW.into(),
        ::pgp::crypto::hash::HashAlgorithm::Sha256,
        &b"by the subkey"[..],
    )
    .unwrap()
    .to_armored_string(Default::default())
    .unwrap();
    // It verifies for the victim.
    let ok =
        pgp::verify_detached_with(b"by the subkey", &sig, &[&victim.public_key_armored]).unwrap();
    assert_eq!(ok.state, verdict::VALID);

    // The attacker binds the stolen subkey with a valid binding but no back-sig.
    let attacker = compatible("Mallory <mallory@example.com>");
    let (att_ssk, _) = SignedSecretKey::from_string(&attacker.encrypted_private_bundle).unwrap();
    let mut flags = KeyFlags::default();
    flags.set_sign(true);
    let mut config = SignatureConfig::from_key(
        rand::rngs::OsRng,
        &att_ssk.primary_key,
        SignatureType::SubkeyBinding,
    )
    .unwrap();
    config.hashed_subpackets = vec![
        Subpacket::regular(SubpacketData::SignatureCreationTime(
            ::pgp::types::Timestamp::from_secs(u32::try_from(now()).unwrap()),
        ))
        .unwrap(),
        Subpacket::regular(SubpacketData::IssuerFingerprint(
            att_ssk.primary_key.fingerprint(),
        ))
        .unwrap(),
        Subpacket::regular(SubpacketData::KeyFlags(flags)).unwrap(),
    ];
    let binding = config
        .sign_subkey_binding(
            &att_ssk.primary_key,
            att_ssk.primary_key.public_key(),
            &PW.into(),
            &stolen,
        )
        .unwrap();
    let mut att_pub = att_ssk.to_public_key();
    att_pub
        .public_subkeys
        .push(SignedPublicSubKey::new(stolen, vec![binding]));
    let att_armored = att_pub.to_armored_string(Default::default()).unwrap();

    let v = pgp::verify_detached_with(b"by the subkey", &sig, &[&att_armored]).unwrap();
    assert_ne!(v.state, verdict::VALID, "{v:?}");
    assert_eq!(v.state, verdict::BAD_SIGNATURE);
    assert_eq!(v.detail.as_deref(), Some("unbound-key"));
}

#[test]
fn change_passphrase() {
    let k = v6("Pass <pass@example.com>");
    let changed = manage::change_passphrase(&k.encrypted_private_bundle, PW, "new-pass").unwrap();
    let ct = pgp::encrypt(b"p", std::slice::from_ref(&k.public_key_armored), None).unwrap();
    assert!(pgp::decrypt(ct.as_bytes(), &changed, PW, None).is_err());
    assert_eq!(
        pgp::decrypt(ct.as_bytes(), &changed, "new-pass", None)
            .unwrap()
            .plaintext,
        b"p"
    );
    assert!(manage::change_passphrase(&k.encrypted_private_bundle, "wrong", "x").is_err());
    assert!(manage::change_passphrase(&k.encrypted_private_bundle, PW, "").is_err());
    // Still locked: no secret component is serialized in the clear.
    assert!(pgp::sign_detached(b"z", &changed, "").is_err());
}

#[test]
fn backup_restore_round_trip() {
    let k = v6("Bak <bak@example.com>");
    let code = pgp::generate_setup_code();
    let payload = pgp::autocrypt_setup_message_full(&k.encrypted_private_bundle, &code).unwrap();
    let restored = pgp::decrypt_autocrypt_setup_message(&payload, &code).unwrap();
    let ct = pgp::encrypt(b"b", std::slice::from_ref(&k.public_key_armored), None).unwrap();
    assert_eq!(
        pgp::decrypt(ct.as_bytes(), &restored, PW, None)
            .unwrap()
            .plaintext,
        b"b"
    );
    assert!(pgp::decrypt_autocrypt_setup_message(&payload, "0000-0000").is_err());
}

// ── Autocrypt ────────────────────────────────────────────────────────────────

#[test]
fn autocrypt_header_round_trip_with_minimal_key() {
    let k = v6("Auto <auto@example.com>");
    // A second subkey must not travel in the header.
    let m = manage::add_subkey(
        &k.encrypted_private_bundle,
        PW,
        SubkeyPurpose::Encrypt,
        None,
    )
    .unwrap();
    let header = pgp::autocrypt_header(&m.public_key_armored, "Auto@Example.com", true).unwrap();
    assert!(header.starts_with("addr=Auto@Example.com; prefer-encrypt=mutual; keydata="));
    let parsed = autocrypt::parse_autocrypt(&header).unwrap();
    assert_eq!(parsed.addr, "auto@example.com");
    assert!(parsed.prefer_encrypt);
    assert!(parsed.key.autocrypt);
    let info = pgp::inspect_key(parsed.key.public_key_armored.as_deref().unwrap()).unwrap();
    assert_eq!(info.subkeys.len(), 1);
    assert_eq!(info.fingerprint, k.fingerprint);
    // The reduced key is still usable for encryption.
    let ct = pgp::encrypt(b"ac", &[parsed.key.public_key_armored.unwrap()], None).unwrap();
    assert_eq!(
        pgp::decrypt(ct.as_bytes(), &m.encrypted_private_bundle, PW, None)
            .unwrap()
            .plaintext,
        b"ac"
    );
}

#[test]
fn autocrypt_attribute_rules() {
    let k = v6("Rules <rules@example.com>");
    let header = pgp::autocrypt_header(&k.public_key_armored, "rules@example.com", false).unwrap();
    assert!(!autocrypt::parse_autocrypt(&header).unwrap().prefer_encrypt);
    let with_ignored = format!("_extra=1; {header}");
    assert!(autocrypt::parse_autocrypt(&with_ignored).is_ok());
    let with_critical = format!("future=1; {header}");
    assert!(autocrypt::parse_autocrypt(&with_critical).is_err());
    assert!(autocrypt::parse_autocrypt("addr=a@b.example").is_err());

    let gossip = autocrypt::gossip_header(&k.public_key_armored, "rules@example.com").unwrap();
    assert!(!gossip.contains("prefer-encrypt"));
    let block = format!(
        "Content-Type: text/plain\r\nAutocrypt-Gossip: {gossip}\r\nAutocrypt-Gossip: addr=x@y.example; keydata=!!\r\n"
    );
    let found = autocrypt::gossip_from_headers(&block);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].key.source, "autocrypt-gossip");
}

#[test]
fn autocrypt_recommendation() {
    let day = 24 * 3600;
    let fresh = PeerState {
        last_seen: 100 * day,
        autocrypt_timestamp: Some(100 * day),
        has_autocrypt_key: true,
        has_gossip_key: false,
        prefer_encrypt: true,
    };
    assert_eq!(autocrypt::recommend(None, true), "disable");
    assert_eq!(autocrypt::recommend(Some(&fresh), true), "encrypt");
    assert_eq!(autocrypt::recommend(Some(&fresh), false), "available");
    let stale = PeerState {
        last_seen: 100 * day,
        autocrypt_timestamp: Some(60 * day),
        ..fresh.clone()
    };
    assert_eq!(autocrypt::recommend(Some(&stale), true), "discourage");
    let gossip_only = PeerState {
        has_autocrypt_key: false,
        has_gossip_key: true,
        ..fresh.clone()
    };
    assert_eq!(autocrypt::recommend(Some(&gossip_only), true), "discourage");
    assert_eq!(
        autocrypt::recommend_message(&["encrypt", "available"]),
        "available"
    );
    assert_eq!(
        autocrypt::recommend_message(&["encrypt", "encrypt"]),
        "encrypt"
    );
    assert_eq!(autocrypt::recommend_message(&[]), "disable");
}

// ── WKD and attached keys ────────────────────────────────────────────────────

#[test]
fn wkd_urls_and_key_check() {
    let [adv, direct] = pgp::wkd_urls("Joe.Doe@Example.ORG").unwrap();
    assert!(adv.starts_with("https://openpgpkey.example.org/"));
    assert!(direct.starts_with("https://example.org/"));
    assert!(pgp::wkd_url("no-at-sign", true).is_err());

    let k = compatible("Wkd <wkd@example.com>");
    let (cert, _) =
        <::pgp::composed::SignedPublicKey as ::pgp::composed::Deserializable>::from_string(
            &k.public_key_armored,
        )
        .unwrap();
    let bin = ::pgp::ser::Serialize::to_bytes(&cert).unwrap();
    assert!(pgp::wkd_key_from_bytes("WKD@example.com", &bin).is_ok());
    assert!(pgp::wkd_key_from_bytes("someone@example.com", &bin).is_err());
}

#[test]
fn attached_keys() {
    let k = v6("Att <att@example.com>");
    let keys = pgp::detect_attached_keys(
        "application/pgp-keys; name=\"key.asc\"",
        Some("key.asc"),
        k.public_key_armored.as_bytes(),
    )
    .expect("a key attachment");
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].fingerprint, k.fingerprint);
    assert_eq!(keys[0].source, "attached");
    assert!(!keys[0].has_private);

    let in_text = format!("Here is my key:\n{}", k.public_key_armored);
    assert!(pgp::detect_attached_keys("text/plain", Some("key.asc"), in_text.as_bytes()).is_some());
    assert!(pgp::detect_attached_keys("text/plain", Some("notes.txt"), b"just text").is_none());
    assert!(pgp::detect_attached_keys("image/png", Some("x.png"), b"\x89PNG").is_none());
    // Secret key armor is not offered as an attached public key.
    assert_eq!(
        pgp::detect_attached_keys(
            "application/pgp-keys",
            None,
            k.encrypted_private_bundle.as_bytes()
        )
        .map(|v| v.len()),
        Some(0)
    );
}
