//! SAML XML-DSig acceptance (t9-e2). Proves the hand-rolled exc-C14N + XML-DSig
//! validator end-to-end with **real cryptography**: assertions are signed with the
//! RustCrypto signing keys (RSA-SHA256 over the exclusive-C14N form) and verified
//! against a **real OpenSSL-generated X.509 certificate** (the `alice` RSA-2048 cert
//! recorded for the mw-crypto interop suite), plus an ECDSA-SHA256 path.
//!
//! Coverage: a valid signature verifies (positive); and each defence rejects —
//! unsigned, tampered digest, tampered SignatureValue, wrong audience, expired,
//! and replayed / wrong `InResponseTo`.
//!
//! NB: the validator both-signs-and-verifies here, so this proves the pipeline is
//! internally exact and the crypto/cert path is real. Byte-exact interop against a
//! foreign IdP's own canonicalizer (a live Keycloak) is the remaining step, deferred
//! to the e6 live-Keycloak gate — see .orchestration/logs/t9-e2.md.

use std::collections::BTreeSet;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use chrono::{Duration, Utc};
use mw_sso::saml::acs::{self, AcsContext};
use mw_sso::saml::c14n;
use mw_sso::{ClaimMap, SsoError};

const DS: &str = "http://www.w3.org/2000/09/xmldsig#";
const EXC: &str = "http://www.w3.org/2001/10/xml-exc-c14n#";
const ENV: &str = "http://www.w3.org/2000/09/xmldsig#enveloped-signature";
const SHA256D: &str = "http://www.w3.org/2001/04/xmlenc#sha256";
const RSA256: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";
const ECDSA256: &str = "http://www.w3.org/2001/04/xmldsig-more#ecdsa-sha256";

const SP_ENTITY_ID: &str = "https://mail.example/sp";
const ACS_URL: &str = "https://mail.example/acs";
const NAME_ID: &str = "user@acme.test";

fn fixture(rel: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/saml")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture {rel}: {e}"))
}

fn idp_cert_pem() -> String {
    fixture("idp-signing.crt.pem")
}

// ── Signing helpers (a stand-in IdP) ─────────────────────────────────────────

fn digest_of(xml: &str, id: &str) -> String {
    let root = c14n::parse_document(xml).expect("parse for digest");
    let el = root.find_by_id(id).expect("element by id");
    let canon = c14n::canonicalize(el, &BTreeSet::new(), None);
    B64.encode(sha256(canon.as_bytes()))
}

fn sha256(data: &[u8]) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    Sha256::digest(data).to_vec()
}

fn signed_info_xml(ref_id: &str, digest: &str, sig_alg: &str) -> String {
    format!(
        "<ds:SignedInfo xmlns:ds=\"{DS}\">\
         <ds:CanonicalizationMethod Algorithm=\"{EXC}\"/>\
         <ds:SignatureMethod Algorithm=\"{sig_alg}\"/>\
         <ds:Reference URI=\"#{ref_id}\">\
         <ds:Transforms><ds:Transform Algorithm=\"{ENV}\"/><ds:Transform Algorithm=\"{EXC}\"/></ds:Transforms>\
         <ds:DigestMethod Algorithm=\"{SHA256D}\"/><ds:DigestValue>{digest}</ds:DigestValue>\
         </ds:Reference></ds:SignedInfo>"
    )
}

fn canon_signed_info(si_xml: &str) -> Vec<u8> {
    // Canonicalize SignedInfo exactly as the verifier does (apex, empty rendered set).
    let root = c14n::parse_document(si_xml).expect("parse SignedInfo");
    c14n::canonicalize(&root, &BTreeSet::new(), None).into_bytes()
}

fn rsa_sign(message: &[u8]) -> String {
    use rsa::RsaPrivateKey;
    use rsa::pkcs1v15;
    use rsa::pkcs8::DecodePrivateKey;
    use sha2::Sha256;
    use signature::{SignatureEncoding, Signer};

    let key = RsaPrivateKey::from_pkcs8_pem(&fixture("idp-signing.key.pem")).expect("rsa key");
    let sk = pkcs1v15::SigningKey::<Sha256>::new(key);
    let sig = sk.sign(message);
    B64.encode(sig.to_bytes())
}

fn signature_block(si_xml: &str, sig_value: &str) -> String {
    format!(
        "<ds:Signature xmlns:ds=\"{DS}\">{si_xml}<ds:SignatureValue>{sig_value}</ds:SignatureValue>\
         <ds:KeyInfo><ds:X509Data><ds:X509Certificate>ignored</ds:X509Certificate></ds:X509Data></ds:KeyInfo>\
         </ds:Signature>"
    )
}

// ── Response builder ─────────────────────────────────────────────────────────

struct Params {
    audience: String,
    in_response_to: String,
    not_before_offset_min: i64,
    not_on_or_after_offset_min: i64,
    assertion_id: String,
    sign: bool,
    tamper_digest: bool,
    tamper_sig: bool,
    sig_alg: &'static str,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            audience: SP_ENTITY_ID.into(),
            in_response_to: "_req-abc".into(),
            not_before_offset_min: -5,
            not_on_or_after_offset_min: 5,
            assertion_id: "_assert-1".into(),
            sign: true,
            tamper_digest: false,
            tamper_sig: false,
            sig_alg: RSA256,
        }
    }
}

fn assertion_body(p: &Params) -> String {
    let now = Utc::now();
    let iso = |d: Duration| (now + d).format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let nb = iso(Duration::minutes(p.not_before_offset_min));
    let noa = iso(Duration::minutes(p.not_on_or_after_offset_min));
    format!(
        "<saml:Assertion xmlns:saml=\"urn:oasis:names:tc:SAML:2.0:assertion\" \
         ID=\"{aid}\" Version=\"2.0\" IssueInstant=\"{ii}\">\
         <saml:Issuer>https://idp.example/entity</saml:Issuer>\
         <saml:Subject>\
         <saml:NameID Format=\"urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress\">{nameid}</saml:NameID>\
         <saml:SubjectConfirmation Method=\"urn:oasis:names:tc:SAML:2.0:cm:bearer\">\
         <saml:SubjectConfirmationData InResponseTo=\"{irt}\" Recipient=\"{acs}\" NotOnOrAfter=\"{noa}\"/>\
         </saml:SubjectConfirmation></saml:Subject>\
         <saml:Conditions NotBefore=\"{nb}\" NotOnOrAfter=\"{noa}\">\
         <saml:AudienceRestriction><saml:Audience>{aud}</saml:Audience></saml:AudienceRestriction>\
         </saml:Conditions>\
         <saml:AttributeStatement>\
         <saml:Attribute Name=\"email\"><saml:AttributeValue>{nameid}</saml:AttributeValue></saml:Attribute>\
         <saml:Attribute Name=\"displayName\"><saml:AttributeValue>Test User</saml:AttributeValue></saml:Attribute>\
         <saml:Attribute Name=\"groups\"><saml:AttributeValue>staff</saml:AttributeValue>\
         <saml:AttributeValue>eng</saml:AttributeValue></saml:Attribute>\
         </saml:AttributeStatement></saml:Assertion>",
        aid = p.assertion_id,
        ii = iso(Duration::zero()),
        nameid = NAME_ID,
        irt = p.in_response_to,
        acs = ACS_URL,
        noa = noa,
        nb = nb,
        aud = p.audience,
    )
}

/// Build a base64 `SAMLResponse` per `p`, signing the assertion when requested.
fn build_response(p: &Params) -> String {
    let bare = assertion_body(p);
    let assertion = if p.sign {
        let mut digest = digest_of(&bare, &p.assertion_id);
        if p.tamper_digest {
            digest = flip_b64(&digest);
        }
        let si = signed_info_xml(&p.assertion_id, &digest, p.sig_alg);
        let mut sig_value = match p.sig_alg {
            RSA256 => rsa_sign(&canon_signed_info(&si)),
            ECDSA256 => ecdsa_sign(&canon_signed_info(&si)),
            _ => unreachable!(),
        };
        if p.tamper_sig {
            sig_value = flip_b64(&sig_value);
        }
        let sig = signature_block(&si, &sig_value);
        // Enveloped: insert the Signature immediately after the Issuer element.
        bare.replacen("</saml:Issuer>", &format!("</saml:Issuer>{sig}"), 1)
    } else {
        bare
    };

    let now = Utc::now();
    let resp = format!(
        "<samlp:Response xmlns:samlp=\"urn:oasis:names:tc:SAML:2.0:protocol\" \
         ID=\"_resp-1\" Version=\"2.0\" IssueInstant=\"{ii}\" InResponseTo=\"{irt}\" \
         Destination=\"{acs}\">\
         <samlp:Status><samlp:StatusCode Value=\"urn:oasis:names:tc:SAML:2.0:status:Success\"/></samlp:Status>\
         {assertion}</samlp:Response>",
        ii = now.format("%Y-%m-%dT%H:%M:%SZ"),
        irt = p.in_response_to,
        acs = ACS_URL,
    );
    B64.encode(resp)
}

fn ecdsa_sign(message: &[u8]) -> String {
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::signature::Signer;
    // A fixed valid P-256 scalar; deterministic (RFC 6979) signing needs no RNG.
    let sk = SigningKey::from_slice(&[0x11u8; 32]).expect("p256 key");
    let sig: p256::ecdsa::Signature = sk.sign(message);
    B64.encode(sig.to_bytes()) // raw r‖s (64 bytes), the XML-DSig ECDSA form
}

fn ecdsa_public_key_pem() -> String {
    use p256::ecdsa::SigningKey;
    use p256::pkcs8::EncodePublicKey;
    use p256::pkcs8::LineEnding;
    let sk = SigningKey::from_slice(&[0x11u8; 32]).unwrap();
    sk.verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap()
}

fn flip_b64(s: &str) -> String {
    // Corrupt one byte of the decoded value and re-encode (keeps it valid base64).
    let mut bytes = B64.decode(s).expect("b64");
    if let Some(b) = bytes.last_mut() {
        *b ^= 0xff;
    }
    B64.encode(bytes)
}

// ── Test harness ─────────────────────────────────────────────────────────────

fn ctx<'a>(certs: &'a [String], want_signed: bool, irt: Option<&'a str>) -> AcsContext<'a> {
    AcsContext {
        sp_entity_id: SP_ENTITY_ID,
        acs_url: ACS_URL,
        certs_pem: certs,
        want_signed,
        expected_in_response_to: irt,
        claim_map: CLAIM_MAP.get_or_init(ClaimMap::default),
        now: Utc::now(),
    }
}

static CLAIM_MAP: std::sync::OnceLock<ClaimMap> = std::sync::OnceLock::new();

fn run(p: &Params, certs: &[String], irt: Option<&str>) -> Result<mw_sso::SsoIdentity, SsoError> {
    let resp = build_response(p);
    acs::process_response(&resp, &ctx(certs, true, irt)).map(|r| r.identity)
}

// ── Positive ─────────────────────────────────────────────────────────────────

#[test]
fn valid_rsa_signed_response_verifies_against_real_cert() {
    let certs = vec![idp_cert_pem()];
    let id = run(&Params::default(), &certs, Some("_req-abc")).expect("valid response");
    assert_eq!(id.subject, NAME_ID);
    assert_eq!(id.email.as_deref(), Some(NAME_ID));
    assert_eq!(id.display_name.as_deref(), Some("Test User"));
    assert_eq!(id.groups, vec!["staff".to_string(), "eng".to_string()]);
}

#[test]
fn valid_ecdsa_signed_response_verifies() {
    let certs = vec![ecdsa_public_key_pem()];
    let p = Params {
        sig_alg: ECDSA256,
        ..Default::default()
    };
    let id = run(&p, &certs, Some("_req-abc")).expect("valid ecdsa response");
    assert_eq!(id.subject, NAME_ID);
}

// ── Negative ─────────────────────────────────────────────────────────────────

#[test]
fn unsigned_response_is_rejected_when_signature_required() {
    let certs = vec![idp_cert_pem()];
    let p = Params {
        sign: false,
        ..Default::default()
    };
    assert!(matches!(
        run(&p, &certs, Some("_req-abc")),
        Err(SsoError::SignatureInvalid(_))
    ));
}

#[test]
fn tampered_digest_is_rejected() {
    let certs = vec![idp_cert_pem()];
    let p = Params {
        tamper_digest: true,
        ..Default::default()
    };
    // A changed DigestValue breaks the SignedInfo signature (RSA) first.
    assert!(matches!(
        run(&p, &certs, Some("_req-abc")),
        Err(SsoError::SignatureInvalid(_))
    ));
}

#[test]
fn tampered_signature_value_is_rejected() {
    let certs = vec![idp_cert_pem()];
    let p = Params {
        tamper_sig: true,
        ..Default::default()
    };
    assert!(matches!(
        run(&p, &certs, Some("_req-abc")),
        Err(SsoError::SignatureInvalid(_))
    ));
}

#[test]
fn tampered_assertion_body_breaks_reference_digest() {
    // Sign a valid assertion, then mutate a signed attribute value in the response.
    let certs = vec![idp_cert_pem()];
    let resp_b64 = build_response(&Params::default());
    let mut xml = String::from_utf8(B64.decode(&resp_b64).unwrap()).unwrap();
    xml = xml.replace("Test User", "Attacker");
    let tampered = B64.encode(xml);
    let err = acs::process_response(&tampered, &ctx(&certs, true, Some("_req-abc"))).unwrap_err();
    assert!(matches!(err, SsoError::SignatureInvalid(_)));
}

#[test]
fn wrong_audience_is_rejected() {
    let certs = vec![idp_cert_pem()];
    let p = Params {
        audience: "https://evil.example/sp".into(),
        ..Default::default()
    };
    // Re-signed with the wrong audience baked in, so the signature is valid but the
    // AudienceRestriction fails.
    assert!(matches!(
        run(&p, &certs, Some("_req-abc")),
        Err(SsoError::AudienceMismatch)
    ));
}

#[test]
fn expired_assertion_is_rejected() {
    let certs = vec![idp_cert_pem()];
    let p = Params {
        not_before_offset_min: -60,
        not_on_or_after_offset_min: -30, // window closed 30 min ago
        ..Default::default()
    };
    assert!(matches!(
        run(&p, &certs, Some("_req-abc")),
        Err(SsoError::Expired)
    ));
}

#[test]
fn wrong_in_response_to_is_replay() {
    let certs = vec![idp_cert_pem()];
    // Server expected a different pending request id than the one the assertion echoes.
    assert!(matches!(
        run(&Params::default(), &certs, Some("_different-req")),
        Err(SsoError::Replay)
    ));
}

#[test]
fn no_pinned_certificate_fails_closed() {
    let certs: Vec<String> = vec![];
    let err = run(&Params::default(), &certs, Some("_req-abc")).unwrap_err();
    // Uniform-401 family; config error (no trust anchor) rather than a silent pass.
    assert_eq!(err.login_status(), 401);
    assert!(matches!(
        err,
        SsoError::Config(_) | SsoError::SignatureInvalid(_)
    ));
}

#[test]
fn every_rejection_is_uniform_401() {
    let certs = vec![idp_cert_pem()];
    for p in [
        Params {
            tamper_sig: true,
            ..Default::default()
        },
        Params {
            audience: "x".into(),
            ..Default::default()
        },
        Params {
            not_on_or_after_offset_min: -30,
            ..Default::default()
        },
    ] {
        if let Err(e) = run(&p, &certs, Some("_req-abc")) {
            assert_eq!(e.login_status(), 401);
        } else {
            panic!("expected rejection");
        }
    }
}
