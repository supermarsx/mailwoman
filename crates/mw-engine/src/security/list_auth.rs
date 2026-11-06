//! The compact authentication summary of a message-list row (`Email/get`
//! property `mailwomanAuth`).
//!
//! The web client shows a sender's brand logo only when the message passed an
//! aligned DMARC check and DKIM passed for a domain of that brand. The full
//! verdict (`SecurityVerdict/get`) needs DNS and is computed when a message is
//! opened; a list must not do that per row. So this reads the verdicts already
//! cached in `security_verdicts` for the page, in one query, and returns only
//! what the check needs. A message with no cached verdict gets `null`, and the
//! client then shows the ordinary initials avatar.

use serde_json::{Value, json};

use crate::engine::Engine;
use crate::backend::Result;
use crate::security::types::SecurityVerdict;

/// The `Email/get` property carrying the summary.
pub const AUTH_PROPERTY: &str = "mailwomanAuth";

/// `{dkim:{result,domain}, dmarc:{result,aligned}}` of a verdict.
pub(crate) fn summary_json(verdict: &SecurityVerdict) -> Value {
    json!({
        "dkim": { "result": verdict.auth.dkim.result, "domain": verdict.auth.dkim.domain },
        "dmarc": { "result": verdict.auth.dmarc.result, "aligned": verdict.auth.dmarc.aligned },
    })
}

impl Engine {
    /// Fill [`AUTH_PROPERTY`] on every assembled email, from the cached verdicts.
    pub(crate) async fn fill_auth(
        &self,
        account_id: &str,
        emails: &mut [Option<Value>],
    ) -> Result<()> {
        let ids: Vec<String> = emails
            .iter()
            .flatten()
            .filter_map(|e| e.get("id").and_then(Value::as_str).map(String::from))
            .collect();
        let cached = self.store().get_security_verdicts(account_id, &ids).await?;
        for email in emails.iter_mut().flatten() {
            let summary = email
                .get("id")
                .and_then(Value::as_str)
                .and_then(|id| cached.get(id))
                .and_then(|bytes| serde_json::from_slice::<SecurityVerdict>(bytes).ok())
                .map(|verdict| summary_json(&verdict))
                .unwrap_or(Value::Null);
            email[AUTH_PROPERTY] = summary;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::types::{ArcVerdict, AuthVerdict, DkimVerdict, DmarcVerdict, SpfVerdict};

    fn verdict(dkim: &str, domain: Option<&str>, dmarc: &str, aligned: bool) -> SecurityVerdict {
        SecurityVerdict {
            email_id: "m1".into(),
            auth: AuthVerdict {
                dkim: DkimVerdict {
                    result: dkim.into(),
                    domain: domain.map(String::from),
                    selector: Some("s1".into()),
                },
                spf: SpfVerdict {
                    result: "pass".into(),
                    domain: domain.map(String::from),
                },
                dmarc: DmarcVerdict {
                    result: dmarc.into(),
                    policy: Some("reject".into()),
                    aligned,
                },
                arc: ArcVerdict {
                    result: "none".into(),
                    chain_length: 0,
                },
            },
            plain_language: String::new(),
            received: Vec::new(),
            signature: None,
            encryption: crate::security::types::EncryptionInfo {
                kind: "none".into(),
                is_encrypted: false,
                decrypts_client_side: false,
            },
            attachments: Vec::new(),
            anomalies: Vec::new(),
        }
    }

    #[test]
    fn summary_carries_only_what_the_brand_check_reads() {
        let pass = summary_json(&verdict("pass", Some("paypal.com"), "pass", true));
        assert_eq!(
            pass,
            json!({
                "dkim": {"result": "pass", "domain": "paypal.com"},
                "dmarc": {"result": "pass", "aligned": true},
            })
        );
        let fail = summary_json(&verdict("fail", None, "fail", false));
        assert_eq!(fail["dkim"]["domain"], Value::Null);
        assert_eq!(fail["dmarc"]["aligned"], json!(false));
        assert!(pass.get("spf").is_none() && pass.get("arc").is_none());
    }
}
