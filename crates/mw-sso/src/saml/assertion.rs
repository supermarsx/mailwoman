//! SAML assertion validation + identity extraction (plan §5): `Conditions`
//! (audience = our SP entity ID, `NotBefore`/`NotOnOrAfter` with clock skew),
//! `SubjectConfirmationData` (`Recipient`, `InResponseTo` bound to the pending
//! AuthnRequest — replay/CSRF defence), and `NameID` + attribute extraction mapped
//! into an [`SsoIdentity`] via the [`ClaimMap`].

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};

use super::c14n::Element;
use crate::{ClaimMap, SsoError, SsoIdentity};

pub const SAML_ASSERTION_NS: &str = "urn:oasis:names:tc:SAML:2.0:assertion";
pub const SAML_PROTOCOL_NS: &str = "urn:oasis:names:tc:SAML:2.0:protocol";
const NAMEID_EMAIL: &str = "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress";

/// Permitted clock skew between SP and IdP when checking validity windows.
const CLOCK_SKEW: Duration = Duration::seconds(120);

/// What the caller must supply to validate an assertion.
pub struct AssertionContext<'a> {
    /// Our SP entity ID — the required `AudienceRestriction` audience.
    pub sp_entity_id: &'a str,
    /// Our ACS URL — the required `SubjectConfirmationData` `Recipient` (if present).
    pub acs_url: &'a str,
    /// The pending AuthnRequest `ID` the assertion's `InResponseTo` must echo.
    pub expected_in_response_to: Option<&'a str>,
    /// The IdP claim/attribute → account-field mapping.
    pub claim_map: &'a ClaimMap,
    /// The current time (injected for testability).
    pub now: DateTime<Utc>,
}

/// A validated assertion: the resolved identity plus the fields the caller caches for
/// replay defence (the assertion `ID` and its expiry).
pub struct ValidatedAssertion {
    /// The identity extracted from the assertion.
    pub identity: SsoIdentity,
    /// The assertion `ID`, for the one-time (replay) cache.
    pub assertion_id: String,
    /// The tightest `NotOnOrAfter` seen (assertion condition / confirmation), used as
    /// the replay-cache expiry.
    pub not_on_or_after: Option<DateTime<Utc>>,
}

/// Validate `assertion` against `ctx` and extract the [`SsoIdentity`]. Any failure is
/// a uniform-401 [`SsoError`]; no assertion content is placed on the wire.
pub fn validate_and_extract(
    assertion: &Element,
    ctx: &AssertionContext,
) -> Result<ValidatedAssertion, SsoError> {
    if assertion.local != "Assertion" || assertion.ns_uri() != SAML_ASSERTION_NS {
        return Err(SsoError::TokenValidation("not a SAML Assertion".into()));
    }
    let assertion_id = assertion
        .attr("ID")
        .ok_or_else(|| SsoError::TokenValidation("assertion has no ID".into()))?
        .to_string();

    let mut tightest_expiry: Option<DateTime<Utc>> = None;

    // ── Subject / NameID + SubjectConfirmationData ──
    let subject = assertion
        .child(SAML_ASSERTION_NS, "Subject")
        .ok_or_else(|| SsoError::TokenValidation("assertion has no Subject".into()))?;
    let name_id_el = subject
        .child(SAML_ASSERTION_NS, "NameID")
        .ok_or_else(|| SsoError::TokenValidation("Subject has no NameID".into()))?;
    let name_id = name_id_el.text();
    if name_id.is_empty() {
        return Err(SsoError::TokenValidation("empty NameID".into()));
    }
    let name_id_format = name_id_el.attr("Format").unwrap_or("").to_string();

    for confirmation in subject.children_named(SAML_ASSERTION_NS, "SubjectConfirmation") {
        if let Some(data) = confirmation.child(SAML_ASSERTION_NS, "SubjectConfirmationData") {
            // Recipient must match our ACS (if asserted).
            if let Some(recipient) = data.attr("Recipient")
                && !ctx.acs_url.is_empty()
                && recipient != ctx.acs_url
            {
                return Err(SsoError::AudienceMismatch);
            }
            // InResponseTo binds the assertion to our AuthnRequest.
            if let Some(irt) = data.attr("InResponseTo") {
                match ctx.expected_in_response_to {
                    Some(expected) if expected == irt => {}
                    _ => return Err(SsoError::Replay),
                }
            }
            if let Some(noa) = data.attr("NotOnOrAfter") {
                let t = parse_time(noa)?;
                if ctx.now >= t + CLOCK_SKEW {
                    return Err(SsoError::Expired);
                }
                tighten(&mut tightest_expiry, t);
            }
        }
    }

    // ── Conditions: validity window + audience ──
    if let Some(conditions) = assertion.child(SAML_ASSERTION_NS, "Conditions") {
        if let Some(nb) = conditions.attr("NotBefore") {
            let t = parse_time(nb)?;
            if ctx.now + CLOCK_SKEW < t {
                return Err(SsoError::Expired);
            }
        }
        if let Some(noa) = conditions.attr("NotOnOrAfter") {
            let t = parse_time(noa)?;
            if ctx.now >= t + CLOCK_SKEW {
                return Err(SsoError::Expired);
            }
            tighten(&mut tightest_expiry, t);
        }
        // Every AudienceRestriction must admit our SP entity ID.
        let restrictions = conditions.children_named(SAML_ASSERTION_NS, "AudienceRestriction");
        for restriction in &restrictions {
            let audiences: Vec<String> = restriction
                .children_named(SAML_ASSERTION_NS, "Audience")
                .into_iter()
                .map(Element::text)
                .collect();
            if !audiences.iter().any(|a| a == ctx.sp_entity_id) {
                return Err(SsoError::AudienceMismatch);
            }
        }
    }

    // ── Attributes ──
    let mut attributes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for statement in assertion.children_named(SAML_ASSERTION_NS, "AttributeStatement") {
        for attr in statement.children_named(SAML_ASSERTION_NS, "Attribute") {
            let name = attr
                .attr("Name")
                .or_else(|| attr.attr("FriendlyName"))
                .unwrap_or("")
                .to_string();
            if name.is_empty() {
                continue;
            }
            let values: Vec<String> = attr
                .children_named(SAML_ASSERTION_NS, "AttributeValue")
                .into_iter()
                .map(Element::text)
                .filter(|v| !v.is_empty())
                .collect();
            attributes.entry(name).or_default().extend(values);
        }
    }

    let identity = map_identity(&name_id, &name_id_format, &attributes, ctx.claim_map);

    Ok(ValidatedAssertion {
        identity,
        assertion_id,
        not_on_or_after: tightest_expiry,
    })
}

fn map_identity(
    name_id: &str,
    name_id_format: &str,
    attrs: &BTreeMap<String, Vec<String>>,
    claim_map: &ClaimMap,
) -> SsoIdentity {
    let first = |name: &str| attrs.get(name).and_then(|v| v.first()).cloned();

    let email = claim_map
        .email
        .as_deref()
        .and_then(first)
        .or_else(|| first("email"))
        .or_else(|| first("urn:oid:0.9.2342.19200300.100.1.3"))
        .or_else(|| (name_id_format == NAMEID_EMAIL).then(|| name_id.to_string()));

    let display_name = claim_map
        .display
        .as_deref()
        .and_then(first)
        .or_else(|| first("displayName"))
        .or_else(|| first("name"))
        .or_else(|| first("urn:oid:2.16.840.1.113730.3.1.241"));

    let groups_attr = claim_map.groups.as_deref().unwrap_or("groups");
    let mut groups = attrs.get(groups_attr).cloned().unwrap_or_default();
    if groups.is_empty()
        && let Some(g) = attrs.get("memberOf")
    {
        groups = g.clone();
    }

    // Content-free single-valued view for downstream mapping + audit (never tokens).
    let mut claims: BTreeMap<String, String> = BTreeMap::new();
    for (k, v) in attrs {
        if let Some(first) = v.first() {
            claims.insert(k.clone(), first.clone());
        }
    }

    SsoIdentity {
        subject: name_id.to_string(),
        email,
        display_name,
        groups,
        claims,
    }
}

fn tighten(slot: &mut Option<DateTime<Utc>>, t: DateTime<Utc>) {
    *slot = Some(match *slot {
        Some(cur) if cur <= t => cur,
        _ => t,
    });
}

fn parse_time(s: &str) -> Result<DateTime<Utc>, SsoError> {
    DateTime::parse_from_rfc3339(s.trim())
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| SsoError::TokenValidation("bad SAML timestamp".into()))
}
