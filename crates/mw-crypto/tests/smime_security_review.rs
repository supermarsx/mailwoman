//! Independent regression cases for trust pinning, algorithm binding and CRL scope.
use der::{
    Decode, Encode,
    asn1::{BitString, OctetString},
};
use mw_crypto::{
    pkix::{
        oid,
        path::{self, Anchor, AnchorSource, Policy},
        revocation,
        sig::{self, Hash, Use},
    },
    smime_cms, smime_keys,
};
use spki::{AlgorithmIdentifierOwned, SubjectPublicKeyInfoOwned};
use x509_cert::{
    Certificate, ext::Extension, name::Name, serial_number::SerialNumber, time::Validity,
};

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/crypto/smime/e41")
            .join(name),
    )
    .unwrap()
}
fn alice_key() -> smime_keys::PrivateKey {
    smime_keys::unwrap(std::str::from_utf8(&fixture("alice.key")).unwrap(), "").unwrap()
}
fn cert(name: &str) -> Certificate {
    smime_cms::parse_certificate(&fixture(name)).unwrap()
}
#[derive(der::Sequence)]
struct Tbs {
    #[asn1(context_specific = "0", tag_mode = "EXPLICIT")]
    version: u8,
    serial_number: SerialNumber,
    signature: AlgorithmIdentifierOwned,
    issuer: Name,
    validity: Validity,
    subject: Name,
    subject_public_key_info: SubjectPublicKeyInfoOwned,
    #[asn1(context_specific = "3", tag_mode = "EXPLICIT")]
    extensions: Vec<Extension>,
}
#[derive(der::Sequence)]
struct Cert {
    tbs: Tbs,
    algorithm: AlgorithmIdentifierOwned,
    signature: BitString,
}
fn issue(
    template: &Certificate,
    issuer: &Certificate,
    key: &smime_keys::PrivateKey,
    ca: bool,
) -> Certificate {
    let mut tbs = Tbs::from_der(&template.tbs_certificate().to_der().unwrap()).unwrap();
    let algorithm = key.signature_algorithm(Hash::Sha256, false).unwrap();
    tbs.issuer = issuer.tbs_certificate().subject().clone();
    tbs.signature = algorithm.clone();
    if ca {
        tbs.extensions
            .retain(|e| e.extn_id != oid::BASIC_CONSTRAINTS && e.extn_id != oid::KEY_USAGE);
        tbs.extensions.push(Extension {
            extn_id: oid::BASIC_CONSTRAINTS,
            critical: true,
            extn_value: OctetString::new(vec![0x30, 3, 1, 1, 0xff]).unwrap(),
        });
        tbs.extensions.push(Extension {
            extn_id: oid::KEY_USAGE,
            critical: true,
            extn_value: OctetString::new(vec![3, 2, 1, 6]).unwrap(),
        });
    }
    let signature = BitString::from_bytes(
        &key.sign(Hash::Sha256, false, &tbs.to_der().unwrap())
            .unwrap(),
    )
    .unwrap();
    Certificate::from_der(
        &Cert {
            tbs,
            algorithm,
            signature,
        }
        .to_der()
        .unwrap(),
    )
    .unwrap()
}
#[test]
fn trusting_an_email_leaf_does_not_authorize_it_to_issue_other_identities() {
    let alice = cert("alice.pem");
    let key = alice_key();
    let forged = issue(&cert("carol.pem"), &alice, &key, false);
    let anchors = [Anchor {
        cert: alice.clone(),
        source: AnchorSource::User,
        distrust_after: None,
    }];
    let policy = Policy {
        at: mw_crypto::clock::now_secs(),
        ..Policy::default()
    };
    assert!(
        path::validate(&alice, &[], &anchors, &policy)
            .error
            .is_none(),
        "the explicitly pinned leaf remains usable"
    );
    assert!(
        path::validate(&forged, &[], &anchors, &policy)
            .error
            .is_some(),
        "pinning Alice must not let Alice certify Carol"
    );
}
#[test]
fn ed25519_signature_cannot_reinterpret_an_x25519_certificate_key() {
    let key = smime_keys::PrivateKey::generate(smime_keys::KeyType::Ed25519).unwrap();
    let mut spki = key.public_spki().unwrap();
    spki.algorithm.oid = oid::X25519;
    let signature = key.sign(Hash::Sha512, false, b"message").unwrap();
    let algorithm = key.signature_algorithm(Hash::Sha512, false).unwrap();
    assert!(
        sig::verify(
            &spki,
            &algorithm,
            Some(Hash::Sha512),
            b"message",
            &signature,
            Use::Message
        )
        .is_err()
    );
}
#[test]
fn weak_ed25519_identity_key_cannot_verify_arbitrary_messages() {
    let mut identity = vec![0u8; 32];
    identity[0] = 1;
    let spki = SubjectPublicKeyInfoOwned {
        algorithm: AlgorithmIdentifierOwned {
            oid: oid::ED25519,
            parameters: None,
        },
        subject_public_key: BitString::from_bytes(&identity).unwrap(),
    };
    let mut signature = vec![0u8; 64];
    signature[0] = 1;
    assert!(
        sig::verify(
            &spki,
            &spki.algorithm,
            None,
            b"arbitrary forged message",
            &signature,
            Use::Message
        )
        .is_err()
    );
}
fn crl_with(
    extensions: Vec<Extension>,
    stale_no_next: bool,
) -> (Vec<u8>, revocation::RevocationQuery) {
    use x509_cert::{
        certificate::Version,
        crl::{CertificateList, TbsCertList},
        time::Time,
    };
    let key = alice_key();
    let issuer = issue(&cert("alice.pem"), &cert("alice.pem"), &key, true);
    let leaf = issue(&cert("carol.pem"), &issuer, &key, false);
    let now = mw_crypto::clock::now_secs();
    let time = |secs| {
        Time::GeneralTime(
            der::asn1::GeneralizedTime::from_unix_duration(std::time::Duration::from_secs(secs))
                .unwrap(),
        )
    };
    let algorithm = key.signature_algorithm(Hash::Sha256, false).unwrap();
    let tbs_cert_list: TbsCertList = TbsCertList {
        version: Version::V2,
        signature: algorithm.clone(),
        issuer: issuer.tbs_certificate().subject().clone(),
        this_update: time(if stale_no_next {
            now - 365 * 86400
        } else {
            now - 60
        }),
        next_update: if stale_no_next {
            None
        } else {
            Some(time(now + 3600))
        },
        revoked_certificates: None,
        crl_extensions: Some(extensions),
    };
    let signature = BitString::from_bytes(
        &key.sign(Hash::Sha256, false, &tbs_cert_list.to_der().unwrap())
            .unwrap(),
    )
    .unwrap();
    let crl = CertificateList {
        tbs_cert_list,
        signature_algorithm: algorithm,
        signature,
    };
    (crl.to_der().unwrap(), revocation::query_for(&leaf, &issuer))
}
#[test]
fn delta_and_scoped_crls_do_not_claim_omitted_certificates_are_good() {
    let (complete, q) = crl_with(vec![], false);
    assert_eq!(
        revocation::check_crl(&complete, &q, mw_crypto::clock::now_secs())
            .unwrap()
            .status,
        "good"
    );
    for (oid, value) in [
        ("2.5.29.27", vec![2, 1, 1]),
        ("2.5.29.28", vec![0x30, 3, 0x82, 1, 0xff]),
    ] {
        let extension = Extension {
            extn_id: oid.parse().unwrap(),
            critical: true,
            extn_value: OctetString::new(value).unwrap(),
        };
        let (partial, q) = crl_with(vec![extension], false);
        let checked = revocation::check_crl(&partial, &q, mw_crypto::clock::now_secs());
        assert!(
            !checked.is_ok_and(|r| r.status == "good"),
            "partial CRL {oid} is not a complete revocation list"
        );
    }
}
#[test]
fn crl_without_expiry_cannot_be_replayed_forever() {
    let (crl, q) = crl_with(vec![], true);
    assert!(
        !revocation::check_crl(&crl, &q, mw_crypto::clock::now_secs())
            .is_ok_and(|r| r.status == "good")
    );
}
