//! SP-initiated `AuthnRequest` over the HTTP-Redirect binding (plan §5): build the
//! request XML, DEFLATE it (raw, RFC 1951 — [`super::deflate`]), base64, and
//! percent-encode it into the IdP SSO URL as the `SAMLRequest` query parameter,
//! carrying the opaque state token as `RelayState`.
//!
//! The request is unsigned (the near-universal Keycloak/ADFS default accepts unsigned
//! AuthnRequests; signing the redirect query is a documented follow-up). Replay/CSRF
//! is enforced by the server-side one-shot `PendingStore` keyed by the state token +
//! the `InResponseTo` binding checked at the ACS.

const SAMLP_NS: &str = "urn:oasis:names:tc:SAML:2.0:protocol";
const SAML_NS: &str = "urn:oasis:names:tc:SAML:2.0:assertion";
const BINDING_POST: &str = "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST";

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;

use crate::SsoError;

/// Build an `AuthnRequest` XML document.
pub fn build_xml(
    request_id: &str,
    issue_instant: &str,
    sp_entity_id: &str,
    acs_url: &str,
    destination: &str,
    nameid_format: &str,
) -> String {
    let name_id_policy = if nameid_format.is_empty() {
        "<samlp:NameIDPolicy AllowCreate=\"true\"/>".to_string()
    } else {
        format!(
            "<samlp:NameIDPolicy Format=\"{}\" AllowCreate=\"true\"/>",
            xml_attr(nameid_format)
        )
    };
    format!(
        "<samlp:AuthnRequest xmlns:samlp=\"{samlp}\" xmlns:saml=\"{saml}\" \
         ID=\"{id}\" Version=\"2.0\" IssueInstant=\"{instant}\" \
         Destination=\"{dest}\" ProtocolBinding=\"{binding}\" \
         AssertionConsumerServiceURL=\"{acs}\">\
         <saml:Issuer>{issuer}</saml:Issuer>{policy}</samlp:AuthnRequest>",
        samlp = SAMLP_NS,
        saml = SAML_NS,
        id = xml_attr(request_id),
        instant = xml_attr(issue_instant),
        dest = xml_attr(destination),
        binding = BINDING_POST,
        acs = xml_attr(acs_url),
        issuer = xml_text(sp_entity_id),
        policy = name_id_policy,
    )
}

/// Assemble the HTTP-Redirect URL: `<idp_sso_url>?SAMLRequest=<enc>&RelayState=<enc>`.
pub fn redirect_url(
    idp_sso_url: &str,
    authn_xml: &str,
    relay_state: &str,
) -> Result<String, SsoError> {
    let deflated = super::deflate(authn_xml.as_bytes())?;
    let encoded = B64.encode(deflated);
    let sep = if idp_sso_url.contains('?') { '&' } else { '?' };
    Ok(format!(
        "{idp_sso_url}{sep}SAMLRequest={}&RelayState={}",
        percent_encode(&encoded),
        percent_encode(relay_state),
    ))
}

/// Percent-encode a query-parameter value: keep RFC 3986 unreserved chars, encode
/// everything else (so base64 `+ / =` and the state token are transport-safe).
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for &b in s.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push('%');
            out.push(hex_digit(b >> 4));
            out.push(hex_digit(b & 0x0f));
        }
    }
    out
}

fn hex_digit(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        _ => (b'A' + (n - 10)) as char,
    }
}

fn xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('"', "&quot;")
}

fn xml_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authnrequest_is_well_formed_and_carries_key_fields() {
        let xml = build_xml(
            "_req1",
            "2026-07-14T12:00:00Z",
            "https://mail.example/sp",
            "https://mail.example/acs",
            "https://idp.example/sso",
            "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent",
        );
        assert!(xml.contains("ID=\"_req1\""));
        assert!(xml.contains("AssertionConsumerServiceURL=\"https://mail.example/acs\""));
        assert!(xml.contains("<saml:Issuer>https://mail.example/sp</saml:Issuer>"));
        assert!(xml.contains("ProtocolBinding=\"urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST\""));
        // Round-trips through the DOM parser (well-formed).
        assert!(super::super::c14n::parse_document(&xml).is_ok());
    }

    #[test]
    fn redirect_url_deflates_and_encodes() {
        let xml = build_xml(
            "_r",
            "2026-07-14T12:00:00Z",
            "sp",
            "acs",
            "https://idp.example/sso",
            "",
        );
        let url = redirect_url("https://idp.example/sso", &xml, "state-token-123").unwrap();
        assert!(url.starts_with("https://idp.example/sso?SAMLRequest="));
        assert!(url.contains("&RelayState=state-token-123"));
        // The SAMLRequest value must be a decodable DEFLATE+base64 round-trip.
        let q = url.split("SAMLRequest=").nth(1).unwrap();
        let enc = q.split('&').next().unwrap();
        let b64 = percent_decode(enc);
        let deflated = B64.decode(b64).unwrap();
        let back = super::super::inflate(&deflated).unwrap();
        assert_eq!(back, xml.as_bytes());
    }

    fn percent_decode(s: &str) -> Vec<u8> {
        let b = s.as_bytes();
        let mut out = Vec::new();
        let mut i = 0;
        while i < b.len() {
            if b[i] == b'%' {
                let h = (from_hex(b[i + 1]) << 4) | from_hex(b[i + 2]);
                out.push(h);
                i += 3;
            } else {
                out.push(b[i]);
                i += 1;
            }
        }
        out
    }
    fn from_hex(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'A'..=b'F' => c - b'A' + 10,
            b'a'..=b'f' => c - b'a' + 10,
            _ => 0,
        }
    }
}
