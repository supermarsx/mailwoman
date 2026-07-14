//! Real-Keycloak interop regression (t9-fix-c14n).
//!
//! Ground truth: `scripts/keycloak/keycloak-saml-response.sample.xml` is a byte-exact
//! `SAMLResponse` produced and RSA-SHA256-signed by a real Keycloak 26.0 (assertion-signed,
//! exc-C14N + enveloped-signature, the assertion apex declaring a **default** `xmlns=`).
//! This is the exact message the e6 live gate rejected with
//! `SignatureInvalid("SignedInfo signature did not verify against any pinned certificate")`.
//!
//! Root cause (see .orchestration/logs/t9-fix-c14n.md): NOT the exc-C14N — the hand-rolled
//! canonicalizer reproduces Keycloak's canonical bytes exactly (the reference digest matches
//! and the RSA signature verifies). The failure was that the pinned IdP signing certificate,
//! as delivered by Keycloak's `/protocol/saml/descriptor` (and by real-world admin-pasted
//! metadata), carries its base64 DER on a **single unwrapped line**, which the strict
//! RFC 7468 PEM decoder rejected → the trust anchor never loaded → "no pinned cert verified".
//!
//! This test pins the certificate in the real-world single-line form (the case that failed)
//! and asserts the hand-rolled validator now VERIFIES Keycloak's real signature.

use std::collections::BTreeSet;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use mw_sso::saml::c14n;
use mw_sso::saml::dsig;

const DSIG_NS: &str = "http://www.w3.org/2000/09/xmldsig#";
const SAML_ASSERTION_NS: &str = "urn:oasis:names:tc:SAML:2.0:assertion";

fn fixture_xml() -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/keycloak/keycloak-saml-response.sample.xml");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("keycloak fixture: {e}"))
}

/// Wrap the assertion's embedded signing-cert base64 as a SINGLE unwrapped PEM line —
/// exactly how Keycloak's SAML descriptor and typical admin-pasted metadata deliver it.
fn single_line_cert_pem(assertion: &c14n::Element) -> String {
    let b64 = assertion
        .descendant(DSIG_NS, "X509Certificate")
        .map(|e| e.text())
        .expect("fixture has an X509Certificate");
    let b64: String = b64.split_whitespace().collect();
    format!("-----BEGIN CERTIFICATE-----\n{b64}\n-----END CERTIFICATE-----\n")
}

#[test]
fn real_keycloak_assertion_signature_verifies() {
    let xml = fixture_xml();
    let root = c14n::parse_document(&xml).expect("parse SAMLResponse");
    let assertion = root
        .child(SAML_ASSERTION_NS, "Assertion")
        .expect("Response carries a saml:Assertion");

    // The exact real-world PEM shape (single line) that regressed the e6 live gate.
    let pem = single_line_cert_pem(assertion);

    let verified = dsig::verify_signed_element(assertion, &[pem])
        .expect("real Keycloak RSA-SHA256 assertion signature must verify");
    assert_eq!(
        verified.reference_id,
        assertion.attr("ID").unwrap(),
        "the verified reference covers the assertion"
    );
}

#[test]
fn exc_c14n_of_assertion_is_byte_exact_with_keycloak_digest() {
    // Independent of the cert path: prove our exc-C14N reproduces Keycloak's canonical
    // bytes by matching the DigestValue Keycloak itself computed over the assertion.
    let xml = fixture_xml();
    let root = c14n::parse_document(&xml).expect("parse");
    let assertion = root
        .child(SAML_ASSERTION_NS, "Assertion")
        .expect("assertion");
    let signature = assertion.child(DSIG_NS, "Signature").expect("signature");

    let canon = c14n::canonicalize(assertion, &BTreeSet::new(), Some(signature.nid));
    let ours = B64.encode({
        use sha2::{Digest, Sha256};
        Sha256::digest(canon.as_bytes())
    });

    let keycloak = signature
        .descendant(DSIG_NS, "DigestValue")
        .map(|e| e.text())
        .expect("DigestValue");
    assert_eq!(
        ours, keycloak,
        "our exc-C14N of the default-namespace-apex assertion matches Keycloak's DigestValue"
    );
}
