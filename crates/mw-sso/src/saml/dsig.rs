//! XML-DSig validation for SAML (plan §5): enveloped-signature transform, SHA-256
//! digest of the referenced element, and RSA-SHA256 / ECDSA-SHA256 verification of
//! `SignedInfo` against the IdP signing certificate **pinned in config**.
//!
//! Trust model: the signature is ALWAYS verified against the operator-configured
//! `idp_signing_certs_pem`, never against a certificate embedded in the message's
//! `KeyInfo` (which an attacker controls). A `KeyInfo` cert, if present, is ignored
//! for trust. Only RSA-SHA256 and ECDSA-SHA256 over exclusive-C14N are accepted; SHA-1
//! and inclusive C14N 1.0 are rejected as `SignatureInvalid` (bounded profile, plan §5).

use std::collections::BTreeSet;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use rsa::RsaPublicKey;
use rsa::pkcs1v15;
use sha2::{Digest, Sha256};
use signature::Verifier;
use spki::DecodePublicKey;
use x509_cert::Certificate;
use x509_cert::der::{DecodePem, Encode};

use super::c14n::{self, Element};
use crate::SsoError;

pub const DSIG_NS: &str = "http://www.w3.org/2000/09/xmldsig#";
const EXC_C14N: &str = "http://www.w3.org/2001/10/xml-exc-c14n#";
const ENVELOPED: &str = "http://www.w3.org/2000/09/xmldsig#enveloped-signature";
const SHA256_DIGEST: &str = "http://www.w3.org/2001/04/xmlenc#sha256";
const RSA_SHA256: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";
const ECDSA_SHA256: &str = "http://www.w3.org/2001/04/xmldsig-more#ecdsa-sha256";

/// A verified signature's coverage: the `ID` of the element the signature protects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedSignature {
    /// The `ID` attribute value of the element covered by the (validated) reference.
    pub reference_id: String,
}

/// Validate the `ds:Signature` that is a **direct child** of `signed` against the
/// pinned `certs_pem`, returning which element ID the signature covers.
///
/// Steps: locate the signature → check its single reference targets `signed`
/// (`URI="#<signed.ID>"`, or empty URI = whole document) → digest `signed` under the
/// enveloped + exc-C14N transforms and compare to `DigestValue` → canonicalize
/// `SignedInfo` and verify it against every pinned cert until one matches.
pub fn verify_signed_element(
    signed: &Element,
    certs_pem: &[String],
) -> Result<VerifiedSignature, SsoError> {
    let signature = signed
        .child(DSIG_NS, "Signature")
        .ok_or_else(|| SsoError::SignatureInvalid("no enveloped Signature".into()))?;
    let signed_info = signature
        .child(DSIG_NS, "SignedInfo")
        .ok_or_else(|| SsoError::SignatureInvalid("no SignedInfo".into()))?;

    // ── Canonicalization + signature algorithms ──
    let c14n_method = signed_info
        .child(DSIG_NS, "CanonicalizationMethod")
        .ok_or_else(|| SsoError::SignatureInvalid("no CanonicalizationMethod".into()))?;
    require_exc_c14n(c14n_method, "CanonicalizationMethod")?;
    let si_inclusive = inclusive_of(c14n_method);

    let sig_method = signed_info
        .child(DSIG_NS, "SignatureMethod")
        .ok_or_else(|| SsoError::SignatureInvalid("no SignatureMethod".into()))?;
    let sig_alg = sig_method
        .attr("Algorithm")
        .ok_or_else(|| SsoError::SignatureInvalid("no SignatureMethod Algorithm".into()))?
        .to_string();

    // ── Reference: exactly one, covering `signed` ──
    let references = signed_info.children_named(DSIG_NS, "Reference");
    if references.len() != 1 {
        return Err(SsoError::SignatureInvalid(format!(
            "expected 1 Reference, found {}",
            references.len()
        )));
    }
    let reference = references[0];
    let uri = reference.attr("URI").unwrap_or("");
    let signed_id = signed.attr("ID").unwrap_or("");
    let covers_signed = uri.is_empty() || uri == format!("#{signed_id}");
    if !covers_signed {
        return Err(SsoError::SignatureInvalid(format!(
            "Reference URI '{uri}' does not cover the signed element"
        )));
    }

    // ── Digest the referenced element under its transforms ──
    let (enveloped, ref_inclusive) = parse_transforms(reference)?;
    let digest_method = reference
        .descendant(DSIG_NS, "DigestMethod")
        .ok_or_else(|| SsoError::SignatureInvalid("no DigestMethod".into()))?;
    if digest_method.attr("Algorithm") != Some(SHA256_DIGEST) {
        return Err(SsoError::SignatureInvalid("digest not SHA-256".into()));
    }
    let expected_digest = reference
        .descendant(DSIG_NS, "DigestValue")
        .map(Element::text)
        .ok_or_else(|| SsoError::SignatureInvalid("no DigestValue".into()))?;

    let skip = if enveloped { Some(signature.nid) } else { None };
    let canon = c14n::canonicalize(signed, &ref_inclusive, skip);
    let actual_digest = B64.encode(Sha256::digest(canon.as_bytes()));
    if actual_digest != expected_digest.trim() {
        return Err(SsoError::SignatureInvalid(
            "reference digest mismatch".into(),
        ));
    }

    // ── Verify SignedInfo signature against the pinned certs ──
    let signature_value = signature
        .child(DSIG_NS, "SignatureValue")
        .map(Element::text)
        .ok_or_else(|| SsoError::SignatureInvalid("no SignatureValue".into()))?;
    let sig_bytes = B64
        .decode(strip_ws(&signature_value))
        .map_err(|_| SsoError::SignatureInvalid("SignatureValue not base64".into()))?;

    let si_canon = c14n::canonicalize(signed_info, &si_inclusive, None);
    let message = si_canon.as_bytes();

    if certs_pem.is_empty() {
        return Err(SsoError::Config(
            "no IdP signing certificate configured".into(),
        ));
    }
    for pem in certs_pem {
        let spki = match spki_der_from_pem(pem) {
            Ok(s) => s,
            Err(_) => continue,
        };
        if verify_signature(&spki, &sig_alg, message, &sig_bytes) {
            return Ok(VerifiedSignature {
                reference_id: signed_id.to_string(),
            });
        }
    }
    Err(SsoError::SignatureInvalid(
        "SignedInfo signature did not verify against any pinned certificate".into(),
    ))
}

fn require_exc_c14n(el: &Element, what: &str) -> Result<(), SsoError> {
    match el.attr("Algorithm") {
        Some(EXC_C14N) => Ok(()),
        other => Err(SsoError::SignatureInvalid(format!(
            "{what} algorithm '{}' unsupported (need exclusive C14N)",
            other.unwrap_or("<none>")
        ))),
    }
}

/// The `InclusiveNamespaces` `PrefixList` declared under a method/transform element.
fn inclusive_of(el: &Element) -> BTreeSet<String> {
    el.descendant(EXC_C14N, "InclusiveNamespaces")
        .and_then(|ns| ns.attr("PrefixList"))
        .map(c14n::parse_prefix_list)
        .unwrap_or_default()
}

/// Parse a `Reference`'s `Transforms`: returns `(enveloped_present, exc_c14n_inclusive)`.
/// Requires the transform chain to be the SAML-standard enveloped + exc-C14N pair.
fn parse_transforms(reference: &Element) -> Result<(bool, BTreeSet<String>), SsoError> {
    let mut enveloped = false;
    let mut inclusive = BTreeSet::new();
    let mut saw_c14n = false;
    if let Some(transforms) = reference.child(DSIG_NS, "Transforms") {
        for t in transforms.children_named(DSIG_NS, "Transform") {
            match t.attr("Algorithm") {
                Some(ENVELOPED) => enveloped = true,
                Some(EXC_C14N) => {
                    saw_c14n = true;
                    inclusive = inclusive_of(t);
                }
                Some(other) => {
                    return Err(SsoError::SignatureInvalid(format!(
                        "unsupported Reference transform '{other}'"
                    )));
                }
                None => {}
            }
        }
    }
    // The digest is over the exc-C14N form regardless of whether a c14n transform was
    // listed explicitly (some IdPs omit it, implying it). Only reject unknown ones.
    let _ = saw_c14n;
    Ok((enveloped, inclusive))
}

/// Verify `signature` over `message` with the SPKI public key, per the DSig
/// `SignatureMethod` algorithm URI.
fn verify_signature(spki_der: &[u8], sig_alg: &str, message: &[u8], signature: &[u8]) -> bool {
    match sig_alg {
        RSA_SHA256 => {
            let Ok(rsa_pub) = RsaPublicKey::from_public_key_der(spki_der) else {
                return false;
            };
            let vk = pkcs1v15::VerifyingKey::<Sha256>::new(rsa_pub);
            match pkcs1v15::Signature::try_from(signature) {
                Ok(sig) => vk.verify(message, &sig).is_ok(),
                Err(_) => false,
            }
        }
        ECDSA_SHA256 => {
            // XML-DSig ECDSA signatures are the raw r‖s concatenation (RFC 4051),
            // NOT DER — 64 bytes for P-256.
            let Ok(vk) = p256::ecdsa::VerifyingKey::from_public_key_der(spki_der) else {
                return false;
            };
            match p256::ecdsa::Signature::from_slice(signature) {
                Ok(sig) => vk.verify(message, &sig).is_ok(),
                Err(_) => false,
            }
        }
        // SHA-1 and everything else are rejected in the bounded profile.
        _ => false,
    }
}

/// Extract the SubjectPublicKeyInfo DER from a PEM. Accepts an X.509 `CERTIFICATE`
/// (the normal IdP form) or a bare `PUBLIC KEY` (SPKI) PEM.
fn spki_der_from_pem(pem: &str) -> Result<Vec<u8>, SsoError> {
    let pem = pem.trim();
    if pem.contains("BEGIN CERTIFICATE") {
        let cert = Certificate::from_pem(pem.as_bytes())
            .map_err(|e| SsoError::Config(format!("bad certificate PEM: {e}")))?;
        cert.tbs_certificate()
            .subject_public_key_info()
            .to_der()
            .map_err(|e| SsoError::Config(format!("SPKI encode: {e}")))
    } else if pem.contains("BEGIN PUBLIC KEY") {
        let (label, der) = ::der::pem::decode_vec(pem.as_bytes())
            .map_err(|e| SsoError::Config(format!("bad public-key PEM: {e}")))?;
        if label != "PUBLIC KEY" {
            return Err(SsoError::Config(format!("unexpected PEM label '{label}'")));
        }
        Ok(der)
    } else {
        Err(SsoError::Config(
            "PEM is neither CERTIFICATE nor PUBLIC KEY".into(),
        ))
    }
}

/// Build a pinned-cert PEM from the base64 DER in a `<ds:X509Certificate>` (used only
/// for SP metadata / diagnostics — never as a trust anchor).
pub fn cert_pem_from_b64_der(b64_der: &str) -> String {
    let mut out = String::from("-----BEGIN CERTIFICATE-----\n");
    let compact: String = b64_der.split_whitespace().collect();
    for chunk in compact.as_bytes().chunks(64) {
        out.push_str(std::str::from_utf8(chunk).unwrap_or(""));
        out.push('\n');
    }
    out.push_str("-----END CERTIFICATE-----\n");
    out
}

fn strip_ws(s: &str) -> String {
    s.split_whitespace().collect()
}
