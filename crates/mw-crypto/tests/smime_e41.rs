//! t32-e41 S/MIME core: chain validation and the state vocabulary, revocation
//! (OCSP, CRL), encryption with key transport and key agreement under GCM or CBC,
//! key types, PKCS#12, and the bundled Mozilla root set.

use mw_crypto::pkix::path::{self, Policy, Purpose};
use mw_crypto::pkix::revocation::{self, RevocationResult};
use mw_crypto::pkix::{name, sig};
use mw_crypto::smime_cms::{self, Cipher, Recipient, SignOptions, VerifyOptions};
use mw_crypto::smime_keys::KeyType;
use mw_crypto::smime_trust::{self, TrustConfig};
use mw_crypto::{smime, verdict};

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

fn now() -> u64 {
    mw_crypto::clock::now_secs()
}

/// Trust the fixture root only (no bundled roots).
fn root_trust() -> TrustConfig {
    TrustConfig {
        user: Vec::new(),
        admin: vec![bytes("root.pem")],
        mozilla: false,
    }
}

fn opts(sender: Option<&str>) -> VerifyOptions {
    VerifyOptions {
        content: Some(bytes("content.txt")),
        trust: root_trust(),
        sender: sender.map(str::to_string),
        now: now(),
        ..VerifyOptions::default()
    }
}

/// Every certificate in the chain checked good.
fn all_good(v: &smime_cms::SmimeVerification) -> Vec<RevocationResult> {
    v.revocation_queries
        .iter()
        .map(|q| RevocationResult {
            cert_fingerprint: q.cert_fingerprint.clone(),
            status: "good".into(),
            source: "ocsp".into(),
            this_update: None,
            next_update: None,
            revoked_at: None,
            reason: None,
        })
        .collect()
}

fn verify(sig_file: &str, o: &VerifyOptions) -> smime_cms::SmimeVerification {
    smime_cms::verify(&bytes(sig_file), o).expect("verify")
}

// ── chain and states ─────────────────────────────────────────────────────────

#[test]
fn valid_chain_to_an_admin_root() {
    let first = verify("alice-detached.p7s", &opts(Some("Alice@Example.com")));
    // Revocation not checked yet: valid otherwise (soft-fail default).
    assert_eq!(first.verification.state, verdict::REVOCATION_UNCHECKED);
    assert_eq!(first.verification.signature.status, "verified");
    assert_eq!(first.anchor.as_deref(), Some("admin"));
    assert_eq!(first.chain.len(), 3, "alice, intermediate, root");
    assert_eq!(first.revocation_queries.len(), 2);
    let q = &first.revocation_queries[0];
    assert_eq!(q.ocsp_urls, vec!["http://127.0.0.1:40080/ocsp".to_string()]);
    assert_eq!(
        q.crl_urls,
        vec!["http://127.0.0.1:40080/crl.der".to_string()]
    );
    assert!(
        first.capabilities.iter().any(|c| c == "aes256-cbc"),
        "{:?}",
        first.capabilities
    );
    assert_eq!(
        first.verification.signature.chain_status.as_deref(),
        Some("trusted")
    );
    assert_eq!(
        first.verification.signer_addresses,
        vec!["alice@example.com".to_string()]
    );
    assert!(first.verification.signed_at.is_some());

    let mut o = opts(Some("alice@example.com"));
    o.revocation = all_good(&first);
    let checked = verify("alice-detached.p7s", &o);
    assert_eq!(checked.verification.state, verdict::VALID);
    assert_eq!(
        checked.verification.signature.revocation_status.as_deref(),
        Some("good")
    );

    let mut hard = opts(Some("alice@example.com"));
    hard.hard_fail = true;
    let hard = verify("alice-detached.p7s", &hard);
    assert_eq!(hard.verification.state, verdict::REVOCATION_UNCHECKED);
    assert_eq!(hard.verification.detail.as_deref(), Some("hard-fail"));
    assert_eq!(hard.verification.signature.status, "unverified-key");
}

#[test]
fn every_key_type_and_padding_verifies() {
    for (file, alg) in [
        ("ecc-detached.p7s", "ecdsa-p256"),
        ("p384-detached.p7s", "ecdsa-p384"),
        ("ed-detached.p7s", "ed25519"),
        ("alice-pss.p7s", "rsa-2048"),
    ] {
        let first = verify(file, &opts(None));
        let mut o = opts(None);
        o.revocation = all_good(&first);
        let v = verify(file, &o);
        assert_eq!(
            v.verification.state,
            verdict::VALID,
            "{file}: {:?}",
            v.verification
        );
        assert_eq!(
            v.verification.signature.algorithm.as_deref(),
            Some(alg),
            "{file}"
        );
    }
}

#[test]
fn opaque_signature_returns_its_content() {
    let v = smime_cms::verify(
        &bytes("alice-opaque.p7m"),
        &VerifyOptions {
            trust: root_trust(),
            ..VerifyOptions::default()
        },
    )
    .unwrap();
    assert_eq!(v.content.as_deref(), Some(&bytes("content.txt")[..]));
    assert_eq!(v.verification.signature.status, "verified");
}

#[test]
fn bad_untrusted_expired_mismatch_and_eku_states() {
    let mut tampered = opts(None);
    tampered.content = Some(b"changed".to_vec());
    let v = verify("alice-detached.p7s", &tampered);
    assert_eq!(v.verification.state, verdict::BAD_SIGNATURE);
    assert_eq!(v.verification.detail.as_deref(), Some("tampered"));

    let mut untrusted = opts(None);
    untrusted.trust = TrustConfig::default();
    let v = verify("alice-detached.p7s", &untrusted);
    assert_eq!(v.verification.state, verdict::UNTRUSTED);
    assert_eq!(v.verification.detail.as_deref(), Some("no-anchor"));
    assert_eq!(
        v.verification.signature.chain_status.as_deref(),
        Some("untrusted")
    );

    let v = verify("old-detached.p7s", &opts(None));
    assert_eq!(v.verification.state, verdict::KEY_EXPIRED);
    assert_eq!(
        v.verification.signature.chain_status.as_deref(),
        Some("expired")
    );

    let v = verify("carol-detached.p7s", &opts(Some("mallory@example.com")));
    assert_eq!(v.verification.state, verdict::ADDRESS_MISMATCH);

    let v = verify("noeku-detached.p7s", &opts(None));
    assert_eq!(v.verification.state, verdict::UNTRUSTED);
    assert_eq!(v.verification.detail.as_deref(), Some("eku"));
}

#[test]
fn a_user_trusted_leaf_is_its_own_anchor() {
    let mut o = opts(None);
    o.trust = TrustConfig {
        user: vec![bytes("carol.pem")],
        admin: Vec::new(),
        mozilla: false,
    };
    let v = verify("carol-detached.p7s", &o);
    assert_eq!(v.anchor.as_deref(), Some("user"));
    assert_ne!(v.verification.state, verdict::UNTRUSTED);
}

// ── revocation ───────────────────────────────────────────────────────────────

#[test]
fn ocsp_and_crl_answers() {
    let alice = smime_cms::parse_certificate(&bytes("alice.pem")).unwrap();
    let revoked = smime_cms::parse_certificate(&bytes("revoked.pem")).unwrap();
    let inter = smime_cms::parse_certificate(&bytes("inter.pem")).unwrap();
    let qa = revocation::query_for(&alice, &inter);
    let qr = revocation::query_for(&revoked, &inter);

    let good = revocation::check_ocsp_response(&bytes("ocsp-good.der"), &qa, now()).unwrap();
    assert_eq!(good.status, "good");
    assert_eq!(good.cert_fingerprint, qa.cert_fingerprint);
    assert!(good.next_update.is_some());
    let bad = revocation::check_ocsp_response(&bytes("ocsp-revoked.der"), &qr, now()).unwrap();
    assert_eq!(bad.status, "revoked");
    assert!(bad.revoked_at.is_some());
    // A response about another certificate is refused.
    assert!(revocation::check_ocsp_response(&bytes("ocsp-good.der"), &qr, now()).is_err());

    assert_eq!(
        revocation::check_crl(&bytes("crl.der"), &qa, now())
            .unwrap()
            .status,
        "good"
    );
    let r = revocation::check_crl(&bytes("crl.der"), &qr, now()).unwrap();
    assert_eq!(r.status, "revoked");
    assert_eq!(r.reason.as_deref(), Some("KeyCompromise"));
    // A CRL past its nextUpdate answers unknown.
    let far = now() + 20 * 365 * 24 * 3600;
    assert_eq!(
        revocation::check_crl(&bytes("crl.der"), &qa, far)
            .unwrap()
            .status,
        "unknown"
    );
    // A modified CRL does not verify.
    let mut forged = bytes("crl.der");
    let n = forged.len();
    forged[n - 10] ^= 0x01;
    assert!(revocation::check_crl(&forged, &qa, now()).is_err());

    // The OCSP request names the certificate (openssl reads it back in the live test).
    let req = revocation::ocsp_request(&qa).unwrap();
    assert_eq!(req[0], 0x30);

    // A revoked certificate in the chain gives the revoked state.
    let mut o = opts(None);
    o.revocation = vec![bad];
    let v = verify("revoked-detached.p7s", &o);
    assert_eq!(v.verification.state, verdict::REVOKED);
    assert_eq!(v.verification.signature.status, "invalid");
}

// ── encryption ───────────────────────────────────────────────────────────────

#[test]
fn openssl_messages_decrypt() {
    for (file, key) in [
        ("to-alice-cbc.p7m", "alice.key"),
        ("to-alice-oaep.p7m", "alice.key"),
        ("to-alice-gcm.p7m", "alice.key"),
        ("to-ecc.p7m", "ecc.key"),
        ("to-p384.p7m", "p384.key"),
    ] {
        let pt = smime_cms::decrypt(&bytes(file), None, &text(key), "")
            .unwrap_or_else(|e| panic!("{file}: {e}"));
        assert_eq!(pt, bytes("content.txt"), "{file}");
    }
    // Addressed to someone else.
    assert!(smime_cms::decrypt(&bytes("to-ecc.p7m"), None, &text("p384.key"), "").is_err());
}

#[test]
fn round_trips_per_recipient_type_and_cipher() {
    for (cert, key) in [
        ("alice.pem", "alice.key"),
        ("ecc.pem", "ecc.key"),
        ("p384.pem", "p384.key"),
        ("x25519.pem", "x25519.key"),
    ] {
        for cipher in [Cipher::Cbc, Cipher::Gcm] {
            let enc = smime_cms::encrypt(
                b"round trip",
                &[Recipient {
                    cert: bytes(cert),
                    gcm: None,
                }],
                cipher,
            )
            .unwrap_or_else(|e| panic!("{cert} {cipher:?}: {e}"));
            assert_eq!(
                enc.smime_type,
                if cipher == Cipher::Gcm {
                    "authEnveloped-data"
                } else {
                    "enveloped-data"
                }
            );
            let pt = smime_cms::decrypt(&enc.der, Some(&bytes(cert)), &text(key), "").unwrap();
            assert_eq!(pt, b"round trip");
        }
    }
}

#[test]
fn auto_picks_gcm_only_when_every_recipient_reads_it() {
    let r = |gcm| Recipient {
        cert: bytes("alice.pem"),
        gcm,
    };
    let both = smime_cms::encrypt(b"x", &[r(Some(true)), r(Some(true))], Cipher::Auto).unwrap();
    assert_eq!(both.smime_type, "authEnveloped-data");
    let one_unknown = smime_cms::encrypt(b"x", &[r(Some(true)), r(None)], Cipher::Auto).unwrap();
    assert_eq!(one_unknown.smime_type, "enveloped-data");
}

// ── keys, signing, PKCS#12 ───────────────────────────────────────────────────

#[test]
fn generated_keys_sign_and_round_trip_through_pkcs12() {
    for (kind, alg) in [
        (KeyType::EcdsaP256, "ecdsa-p256"),
        (KeyType::EcdsaP384, "ecdsa-p384"),
        (KeyType::Ed25519, "ed25519"),
    ] {
        let g = smime::generate_key(kind, "Gen", "gen@example.com", "pw", now()).unwrap();
        assert_eq!(g.cert.algorithm, alg);
        let sig = smime_cms::sign(
            b"generated",
            g.cert.cert_pem.as_bytes(),
            &g.encrypted_private_bundle,
            "pw",
            &SignOptions::default(),
        )
        .unwrap();
        let mut o = VerifyOptions {
            content: Some(b"generated".to_vec()),
            trust: TrustConfig {
                user: vec![g.cert.cert_pem.clone().into_bytes()],
                admin: Vec::new(),
                mozilla: false,
            },
            sender: Some("gen@example.com".into()),
            ..VerifyOptions::default()
        };
        let v = smime_cms::verify(&sig, &o).unwrap();
        assert_eq!(v.anchor.as_deref(), Some("user"), "{alg}");
        assert_eq!(
            v.verification.signature.status, "verified",
            "{alg}: {:?}",
            v.verification
        );
        o.content = Some(b"generatee".to_vec());
        assert_eq!(
            smime_cms::verify(&sig, &o).unwrap().verification.state,
            verdict::BAD_SIGNATURE
        );

        let p12 =
            smime::export_pkcs12(&g.cert.cert_pem, &g.encrypted_private_bundle, "pw").unwrap();
        let back = smime::import_pkcs12(&p12, "pw").unwrap();
        assert_eq!(back.fingerprint, g.cert.fingerprint);
        assert_eq!(back.algorithm, alg);
    }
}

#[test]
fn pss_and_sha384_signing() {
    let g = smime::generate_with_bits("P", "p@example.com", "pw", now(), 2048).unwrap();
    for (pss, hash) in [
        (true, sig::Hash::Sha256),
        (false, sig::Hash::Sha384),
        (true, sig::Hash::Sha512),
    ] {
        let s = smime_cms::sign(
            b"d",
            g.cert.cert_pem.as_bytes(),
            &g.encrypted_private_bundle,
            "pw",
            &SignOptions {
                pss,
                hash: Some(hash),
                ..SignOptions::default()
            },
        )
        .unwrap();
        let v = smime::verify_detached(&s, Some(b"d")).unwrap();
        assert_eq!(v.status, "verified", "pss={pss} {hash:?}");
    }
}

#[test]
fn openssl_pkcs12_files_with_ec_and_ed25519_keys_import() {
    assert_eq!(
        smime::import_pkcs12(&bytes("ecc.p12"), "test")
            .unwrap()
            .algorithm,
        "ecdsa-p256"
    );
    assert_eq!(
        smime::import_pkcs12(&bytes("ed.p12"), "test")
            .unwrap()
            .algorithm,
        "ed25519"
    );
    assert!(smime::import_pkcs12(&bytes("ecc.p12"), "wrong").is_err());
}

// ── names, paths, roots ──────────────────────────────────────────────────────

#[test]
fn sender_match_rules() {
    let alice = smime_cms::parse_certificate(&bytes("alice.pem")).unwrap();
    assert!(name::matches_sender(&alice, "ALICE@example.COM", true));
    assert!(!name::matches_sender(&alice, "ALICE@example.com", false));
    assert!(!name::matches_sender(&alice, "alice@example.org", true));
    assert!(name::matches_sender(&alice, "<alice@example.com.>", true));
}

#[test]
fn a_missing_intermediate_leaves_no_anchor() {
    let alice = smime_cms::parse_certificate(&bytes("alice.pem")).unwrap();
    let anchors = root_trust().anchors();
    let v = path::validate(
        &alice,
        &[],
        &anchors,
        &Policy {
            purpose: Purpose::Sign,
            at: now(),
            ..Policy::default()
        },
    );
    assert_eq!(v.error, Some(path::PathError::NoAnchor));
    let inter = smime_cms::parse_certificate(&bytes("inter.pem")).unwrap();
    let v = path::validate(
        &alice,
        &[inter],
        &anchors,
        &Policy {
            purpose: Purpose::Encrypt,
            at: now(),
            ..Policy::default()
        },
    );
    assert!(v.error.is_none(), "{:?}", v.error);
    // P-256 leaf: keyAgreement allows encryption, keyEncipherment is not needed.
    let ecc = smime_cms::parse_certificate(&bytes("ecc.pem")).unwrap();
    let inter = smime_cms::parse_certificate(&bytes("inter.pem")).unwrap();
    let v = path::validate(
        &ecc,
        &[inter],
        &anchors,
        &Policy {
            purpose: Purpose::Encrypt,
            at: now(),
            ..Policy::default()
        },
    );
    assert!(v.error.is_none(), "{:?}", v.error);
}

#[test]
fn bundled_mozilla_roots() {
    let info = smime_trust::root_set_info();
    assert!(info.count >= 50, "{info:?}");
    assert_eq!(info.snapshot.len(), 10);
    let roots = smime_trust::mozilla_roots();
    assert_eq!(roots.len(), info.count);
    // Every bundled root is self-signed and parses as a CA certificate.
    for r in roots {
        assert_eq!(
            r.cert.tbs_certificate().issuer(),
            r.cert.tbs_certificate().subject(),
            "{}",
            path::fingerprint(&r.cert)
        );
    }
    // The fixture chain is not trusted by the bundled set.
    let mut o = opts(None);
    o.trust = TrustConfig::bundled();
    assert_eq!(
        verify("alice-detached.p7s", &o).verification.state,
        verdict::UNTRUSTED
    );
}

#[test]
fn claimed_signing_time_cannot_resurrect_an_expired_certificate() {
    // signingTime is chosen by the signer, not a trusted timestamp authority.
    let mut o = opts(None);
    o.now += 20 * 365 * 24 * 3600;
    assert_eq!(
        verify("alice-detached.p7s", &o).verification.state,
        verdict::KEY_EXPIRED
    );
}

#[test]
fn san_without_email_does_not_fall_back_to_subject_email() {
    use der::Encode;
    let cert = smime_cms::parse_certificate(&bytes("alice.pem")).unwrap();
    let mut der = cert.to_der().unwrap();
    // Change only the SAN GeneralName tag from rfc822Name to dNSName. The
    // subject email remains intact; signature validity is irrelevant to extraction.
    let needle = b"\x81\x11alice@example.com";
    let pos = der.windows(needle.len()).position(|w| w == needle).unwrap();
    der[pos] = 0x82;
    let cert = smime_cms::parse_certificate(&der).unwrap();
    assert!(!name::matches_sender(&cert, "alice@example.com", true));
}

#[test]
fn malformed_key_usage_is_not_treated_as_absent() {
    use der::Encode;
    let cert = smime_cms::parse_certificate(&bytes("alice.pem")).unwrap();
    let ext = cert
        .tbs_certificate()
        .extensions()
        .unwrap()
        .iter()
        .find(|e| e.extn_id == mw_crypto::pkix::oid::KEY_USAGE)
        .unwrap();
    let mut der = cert.to_der().unwrap();
    let needle = ext.extn_value.as_bytes();
    let pos = der.windows(needle.len()).position(|w| w == needle).unwrap();
    der[pos] = 0x04; // OCTET STRING instead of the required BIT STRING.
    let cert = smime_cms::parse_certificate(&der).unwrap();
    let result = path::validate(
        &cert,
        &[],
        &[],
        &Policy {
            at: now(),
            ..Policy::default()
        },
    );
    assert_eq!(result.error, Some(path::PathError::Constraint("key-usage")));
}

#[test]
fn encryption_refuses_expired_and_non_email_certificates() {
    for name in ["old.pem", "noeku.pem"] {
        assert!(
            smime_cms::encrypt(
                b"secret",
                &[Recipient {
                    cert: bytes(name),
                    gcm: None
                }],
                Cipher::Cbc
            )
            .is_err(),
            "{name}"
        );
    }
}

#[test]
fn encryption_refuses_noncontributory_x25519_keys() {
    use der::Encode;
    let cert = smime_cms::parse_certificate(&bytes("x25519.pem")).unwrap();
    let needle = cert
        .tbs_certificate()
        .subject_public_key_info()
        .subject_public_key
        .raw_bytes();
    let mut der = cert.to_der().unwrap();
    let pos = der.windows(needle.len()).position(|w| w == needle).unwrap();
    der[pos..pos + needle.len()].fill(0);
    assert!(
        smime_cms::encrypt(
            b"secret",
            &[Recipient {
                cert: der,
                gcm: None
            }],
            Cipher::Gcm
        )
        .is_err()
    );
}

#[test]
fn rsa_outbound_transport_uses_oaep() {
    use der::Decode;
    let out = smime_cms::encrypt(
        b"new message",
        &[Recipient {
            cert: bytes("alice.pem"),
            gcm: None,
        }],
        Cipher::Cbc,
    )
    .unwrap();
    let ci = cms::content_info::ContentInfo::from_der(&out.der).unwrap();
    let envelope: cms::enveloped_data::EnvelopedData = ci.content.decode_as().unwrap();
    let cms::enveloped_data::RecipientInfo::Ktri(info) =
        envelope.recip_infos.0.iter().next().unwrap()
    else {
        panic!("RSA recipient")
    };
    assert_eq!(info.key_enc_alg.oid, mw_crypto::pkix::oid::RSAES_OAEP);
}
