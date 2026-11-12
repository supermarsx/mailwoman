//! Operator-maintained governance records. Mappings are implementation aids,
//! never certification or an automatic assertion of operating effectiveness.
use serde_json::{Value, json};

pub(crate) const CONTRACT_FIELDS: &[&str] = &[
    "owner",
    "status",
    "processorId",
    "contractType",
    "counterparty",
    "purpose",
    "signedAt",
    "expiresAt",
    "reviewAt",
    "jurisdiction",
    "documentUrl",
    "clauses",
    "notes",
];
pub(crate) const TRANSFER_FIELDS: &[&str] = &[
    "owner",
    "status",
    "direction",
    "sourceCountry",
    "destinationCountry",
    "exporter",
    "importer",
    "processorId",
    "contractId",
    "categories",
    "purpose",
    "legalBasis",
    "mechanism",
    "safeguards",
    "assessmentUrl",
    "reviewAt",
    "notes",
];
pub(crate) const CONTROL_FIELDS: &[&str] = &[
    "owner",
    "status",
    "controlId",
    "mappings",
    "implementation",
    "evidence",
    "testedAt",
    "reviewAt",
    "notes",
];
pub(crate) fn fields(kind: &str) -> Option<&'static [&'static str]> {
    match kind {
        "contract" => Some(CONTRACT_FIELDS),
        "transfer" => Some(TRANSFER_FIELDS),
        "control" => Some(CONTROL_FIELDS),
        _ => None,
    }
}

pub(crate) fn validate(
    kind: &str,
    fields: &std::collections::BTreeMap<String, String>,
) -> Result<(), &'static str> {
    let value = |key: &str| fields.get(key).map(String::as_str).unwrap_or("").trim();
    let required = |keys: &[&str]| keys.iter().all(|key| !value(key).is_empty());
    for key in ["signedAt", "expiresAt", "reviewAt", "testedAt"] {
        let text = value(key);
        if !text.is_empty()
            && (text.len() != 10 || chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d").is_err())
        {
            return Err("Use YYYY-MM-DD dates.");
        }
    }
    for key in ["documentUrl", "assessmentUrl"] {
        let text = value(key);
        if !text.is_empty() {
            let url = reqwest::Url::parse(text).map_err(|_| "Use an HTTPS evidence URL.")?;
            if url.scheme() != "https"
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
            {
                return Err("Use an HTTPS evidence URL without credentials.");
            }
        }
    }
    match kind {
        "control" => {
            if !matches!(
                value("status"),
                "unassessed" | "planned" | "implemented" | "verified" | "exception"
            ) {
                return Err("Choose a valid control status.");
            }
            if value("status") == "verified" && !required(&["owner", "evidence", "testedAt"]) {
                return Err("Verified controls require an owner, evidence and a test date.");
            }
            if value("status") == "exception" && !required(&["owner", "notes", "reviewAt"]) {
                return Err("Exceptions require an owner, justification and review date.");
            }
        }
        "contract" => {
            if !matches!(
                value("status"),
                "draft" | "active" | "expired" | "terminated"
            ) {
                return Err("Choose a valid contract status.");
            }
            if !matches!(value("contractType"), "dpa" | "msa" | "scc" | "other") {
                return Err("Choose a valid contract type.");
            }
            if value("status") == "active"
                && !required(&["owner", "counterparty", "signedAt", "documentUrl"])
            {
                return Err(
                    "Active contracts require an owner, counterparty, signing date and document link.",
                );
            }
            if !value("expiresAt").is_empty()
                && !value("signedAt").is_empty()
                && value("expiresAt") < value("signedAt")
            {
                return Err("Expiry must not precede signing.");
            }
        }
        "transfer" => {
            if !matches!(value("status"), "draft" | "active" | "suspended" | "closed")
                || !matches!(value("direction"), "internal" | "international")
            {
                return Err("Choose a valid transfer type and status.");
            }
            for key in ["sourceCountry", "destinationCountry"] {
                let text = value(key);
                if !text.is_empty()
                    && (text.len() != 2 || !text.bytes().all(|c| c.is_ascii_uppercase()))
                {
                    return Err("Use two-letter uppercase country codes.");
                }
            }
            if value("status") == "active"
                && !required(&[
                    "owner",
                    "exporter",
                    "importer",
                    "purpose",
                    "categories",
                    "legalBasis",
                    "sourceCountry",
                    "destinationCountry",
                ])
            {
                return Err(
                    "Active transfers require ownership, parties, countries, purpose, data categories and legal basis.",
                );
            }
            if !matches!(
                value("mechanism"),
                "internal" | "adequacy" | "scc" | "bcr" | "derogation" | "other"
            ) {
                return Err("Choose a valid transfer mechanism.");
            }
            if value("direction") == "international" && value("status") == "active" {
                if value("mechanism") == "scc" && !required(&["contractId"]) {
                    return Err("Link the signed contract for an active SCC transfer.");
                }
                if value("mechanism") == "internal" || !required(&["safeguards", "reviewAt"]) {
                    return Err(
                        "Active international transfers require safeguards, a review date and an international transfer mechanism.",
                    );
                }
                if matches!(value("mechanism"), "scc" | "bcr" | "other")
                    && !required(&["assessmentUrl"])
                {
                    return Err("Record the transfer assessment link for this mechanism.");
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Original short control descriptions mapped to references, not standard text.
pub(crate) fn catalog() -> Vec<Value> {
    [
 ("access","Access and privilege reviews","GDPR Art. 32; SOC 2 CC6.1, CC6.2, CC6.3; ISO 27001 A.5.15, A.5.18","docs/admin/security-policy.md"),
 ("authentication","Strong authentication and session controls","GDPR Art. 32; SOC 2 CC6.1; ISO 27001 A.8.5","docs/admin/security-center.md"),
 ("encryption","Encryption and key management","GDPR Art. 32; SOC 2 CC6.1, CC6.7; ISO 27001 A.8.24","docs/admin/encryption.md"),
 ("audit","Audit logging and integrity verification","GDPR Art. 5(2), 32; SOC 2 CC7.2; ISO 27001 A.8.15, A.8.16","crates/mw-store/src/audit_chain.rs"),
 ("retention","Retention and secure disposal","GDPR Art. 5(1)(e), 17; SOC 2 C1.2, P4.2, P4.3; ISO 27001 A.8.10","docs/admin/retention.md"),
 ("requests","Data subject requests and identity verification","GDPR Art. 12–22; SOC 2 P4.3, P5.1, P5.2; ISO 27001 A.5.34","docs/admin/data-requests.md"),
 ("consent","Consent records and withdrawal","GDPR Art. 7; SOC 2 P2.1; ISO 27001 A.5.34","docs/admin/cookies.md"),
 ("processing","Processing inventory and privacy notices","GDPR Art. 13, 14, 30; SOC 2 P1.1; ISO 27001 A.5.34","docs/admin/compliance.md"),
 ("suppliers","Processor contracts and supplier review","GDPR Art. 28; SOC 2 CC9.2, P6.4; ISO 27001 A.5.19, A.5.20","docs/admin/compliance.md"),
 ("transfers","Internal and international data transfers","GDPR Art. 44–49; SOC 2 CC6.7, P6.1, P6.2; ISO 27001 A.5.14","docs/admin/compliance.md"),
 ("incident","Incident response and breach notification","GDPR Art. 33, 34; SOC 2 CC7.3, CC7.4, CC7.5, P6.3; ISO 27001 A.5.24–A.5.28","docs/admin/breach-register.md"),
 ("risk","Risk assessment and treatment","GDPR Art. 24, 35; SOC 2 CC3.1–CC3.4; ISO 27001 6.1","docs/admin/compliance.md"),
 ("recovery","Backup restoration and erasure propagation","GDPR Art. 32; SOC 2 A1.2, A1.3; ISO 27001 A.8.13, A.5.30","docs/admin/encryption.md"),
 ("change","Change approval and vulnerability remediation","GDPR Art. 25, 32; SOC 2 CC8.1; ISO 27001 A.8.8, A.8.32","docs/security/audit-prep/self-baseline.md"),
 ("assurance","Internal audits and management review","GDPR Art. 24; SOC 2 CC4.1, CC4.2; ISO 27001 9.2, 9.3","docs/security/audit-prep/README.md"),
 ].into_iter().map(|(id,name,mappings,source)|json!({"id":format!("builtin:{id}"),"name":name,"kind":"control","builtin":true,"enabled":true,"controlId":id,"mappings":mappings,"implementation":source,"status":"unassessed"})).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verified_needs_evidence() {
        let fields = std::collections::BTreeMap::from([("status".into(), "verified".into())]);
        assert!(validate("control", &fields).is_err());
    }
    #[test]
    fn catalog_never_claims_verification() {
        assert_eq!(catalog().len(), 15);
        assert!(catalog().iter().all(|v| v["status"] == "unassessed"));
    }
}
