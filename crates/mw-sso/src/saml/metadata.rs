//! SP metadata publication (plan §5): the `EntityDescriptor` an IdP consumes to learn
//! our entity ID, ACS endpoint (HTTP-POST binding), supported NameID format, and SLO
//! endpoint. Served at `GET /api/sso/{id}/metadata` via [`crate::SsoLogin::metadata`].

const MD_NS: &str = "urn:oasis:names:tc:SAML:2.0:metadata";
const BINDING_POST: &str = "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST";
const BINDING_REDIRECT: &str = "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect";

/// Build the SP `EntityDescriptor` XML.
pub fn sp_metadata_xml(
    sp_entity_id: &str,
    acs_url: &str,
    nameid_format: &str,
    slo_url: Option<&str>,
) -> String {
    let name_id_format = if nameid_format.is_empty() {
        String::new()
    } else {
        format!(
            "<md:NameIDFormat>{}</md:NameIDFormat>",
            esc_text(nameid_format)
        )
    };
    let slo = slo_url
        .map(|u| {
            format!(
                "<md:SingleLogoutService Binding=\"{BINDING_REDIRECT}\" Location=\"{}\"/>",
                esc_attr(u)
            )
        })
        .unwrap_or_default();
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <md:EntityDescriptor xmlns:md=\"{md}\" entityID=\"{eid}\">\
         <md:SPSSODescriptor AuthnRequestsSigned=\"false\" WantAssertionsSigned=\"true\" \
         protocolSupportEnumeration=\"urn:oasis:names:tc:SAML:2.0:protocol\">\
         {slo}{nameid}\
         <md:AssertionConsumerService Binding=\"{BINDING_POST}\" Location=\"{acs}\" \
         index=\"0\" isDefault=\"true\"/>\
         </md:SPSSODescriptor></md:EntityDescriptor>",
        md = MD_NS,
        eid = esc_attr(sp_entity_id),
        slo = slo,
        nameid = name_id_format,
        acs = esc_attr(acs_url),
    )
}

fn esc_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('"', "&quot;")
}

fn esc_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_has_entity_and_acs() {
        let md = sp_metadata_xml(
            "https://mail.example/sp",
            "https://mail.example/acs",
            "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent",
            Some("https://mail.example/slo"),
        );
        assert!(md.contains("entityID=\"https://mail.example/sp\""));
        assert!(md.contains("Location=\"https://mail.example/acs\""));
        assert!(md.contains("SingleLogoutService"));
        assert!(md.contains("NameIDFormat"));
        // Well-formed.
        assert!(super::super::c14n::parse_document(md.split("?>").nth(1).unwrap()).is_ok());
    }
}
