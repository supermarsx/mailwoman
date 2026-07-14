//! Assertion Consumer Service (ACS): consume the base64 `SAMLResponse` an IdP POSTs
//! (HTTP-POST binding), verify its XML-DSig signature, and validate + extract the
//! assertion (plan §5). This ties [`super::dsig`] (signature) and
//! [`super::assertion`] (conditions/audience/replay + identity) together.
//!
//! The SAML `Response` may carry the signature on the `Response`, the `Assertion`, or
//! both. We verify the signature that covers the assertion we consume: a directly
//! signed `Assertion`, else a signed enclosing `Response`. `EncryptedAssertion` is a
//! documented, config-visible rejection in 26.9 (NOT a silent pass) — decryption is
//! out of the bounded profile.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use chrono::{DateTime, Utc};

use super::assertion::{self, AssertionContext, SAML_ASSERTION_NS, SAML_PROTOCOL_NS};
use super::c14n;
use super::dsig::{self, DSIG_NS};
use crate::{ClaimMap, SsoError, SsoIdentity};

/// Inputs the provider supplies to process an ACS POST.
pub struct AcsContext<'a> {
    /// Our SP entity ID (required assertion audience).
    pub sp_entity_id: &'a str,
    /// Our ACS URL (required `Recipient`, if asserted).
    pub acs_url: &'a str,
    /// Pinned IdP signing certificates (PEM) — the sole trust anchor.
    pub certs_pem: &'a [String],
    /// Require a valid signature covering the assertion.
    pub want_signed: bool,
    /// The pending AuthnRequest `ID` to match against `InResponseTo`.
    pub expected_in_response_to: Option<&'a str>,
    /// IdP attribute → account-field mapping.
    pub claim_map: &'a ClaimMap,
    /// Current time (injected for tests).
    pub now: DateTime<Utc>,
}

/// The outcome of a successful ACS: the identity plus the replay-cache keys.
#[derive(Debug, Clone)]
pub struct AcsResult {
    /// Extracted identity.
    pub identity: SsoIdentity,
    /// Consumed assertion `ID` (for the one-time replay cache).
    pub assertion_id: String,
    /// Tightest assertion expiry (replay-cache TTL).
    pub not_on_or_after: Option<DateTime<Utc>>,
}

/// Process a base64 `SAMLResponse` (HTTP-POST binding value).
pub fn process_response(saml_response_b64: &str, ctx: &AcsContext) -> Result<AcsResult, SsoError> {
    let xml_bytes = B64
        .decode(strip_ws(saml_response_b64))
        .map_err(|_| SsoError::TokenValidation("SAMLResponse not base64".into()))?;
    let xml = std::str::from_utf8(&xml_bytes)
        .map_err(|_| SsoError::TokenValidation("SAMLResponse not UTF-8".into()))?;
    let root = c14n::parse_document(xml)?;
    if root.local != "Response" || root.ns_uri() != SAML_PROTOCOL_NS {
        return Err(SsoError::TokenValidation("not a samlp:Response".into()));
    }

    // Response-level InResponseTo (CSRF/replay binding to our AuthnRequest).
    if let Some(irt) = root.attr("InResponseTo") {
        match ctx.expected_in_response_to {
            Some(expected) if expected == irt => {}
            _ => return Err(SsoError::Replay),
        }
    }

    // Top-level Status must be Success.
    let status_code = root
        .descendant(SAML_PROTOCOL_NS, "StatusCode")
        .and_then(|s| s.attr("Value").map(str::to_string))
        .unwrap_or_default();
    if !status_code.ends_with(":status:Success") {
        return Err(SsoError::TokenValidation(format!(
            "SAML status not Success: {status_code}"
        )));
    }

    // EncryptedAssertion — documented rejection (bounded profile, plan §5).
    if root
        .descendant(SAML_ASSERTION_NS, "EncryptedAssertion")
        .is_some()
    {
        return Err(SsoError::TokenValidation(
            "EncryptedAssertion is not supported in this release".into(),
        ));
    }

    let assertion = root
        .child(SAML_ASSERTION_NS, "Assertion")
        .ok_or_else(|| SsoError::TokenValidation("Response carries no Assertion".into()))?;

    // ── Signature: verify the one that covers the assertion we consume ──
    let mut signature_ok = false;
    if assertion.child(DSIG_NS, "Signature").is_some() {
        dsig::verify_signed_element(assertion, ctx.certs_pem)?;
        signature_ok = true;
    } else if root.child(DSIG_NS, "Signature").is_some() {
        // A signed Response envelopes (and thus vouches for) the assertion.
        dsig::verify_signed_element(&root, ctx.certs_pem)?;
        signature_ok = true;
    }
    if ctx.want_signed && !signature_ok {
        return Err(SsoError::SignatureInvalid(
            "no signature covers the assertion".into(),
        ));
    }

    // ── Conditions / audience / InResponseTo / identity ──
    let validated = assertion::validate_and_extract(
        assertion,
        &AssertionContext {
            sp_entity_id: ctx.sp_entity_id,
            acs_url: ctx.acs_url,
            expected_in_response_to: ctx.expected_in_response_to,
            claim_map: ctx.claim_map,
            now: ctx.now,
        },
    )?;

    Ok(AcsResult {
        identity: validated.identity,
        assertion_id: validated.assertion_id,
        not_on_or_after: validated.not_on_or_after,
    })
}

fn strip_ws(s: &str) -> String {
    s.split_whitespace().collect()
}
