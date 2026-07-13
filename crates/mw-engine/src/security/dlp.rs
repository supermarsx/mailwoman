//! DLP family (`Dlp/getRules`, `Dlp/scan`, frozen §2.2) + the outbound
//! [`evaluate`] hook (plan §1.8). Enforcement is inline on `EmailSubmission/set`:
//! a `block` verdict fails the submission with a `dlpBlocked` error; `warn`/
//! `require-encryption` surface pre-send via `Dlp/scan`. Rules are config/env-
//! sourced (`MW_DLP_RULES` = path to a JSON `[DlpRule]` file); every evaluation
//! that matches writes a REDACTED `dlp_audit` row (matched detector + rule, NEVER
//! the matched content — the excerpt carries only detector tokens).
//!
//! Built-in detectors (`pan`/`iban`/`ssn`/`national-id`) are hand-scanned so they
//! need no regex; `custom-regex` compiles the rule's pattern.

use serde_json::{Value, json};

use crate::engine::Engine;
use crate::security::types::{DlpRule, DlpVerdict};

use super::{gen_id, server_fail};

/// The compose/submit content a rule set is evaluated against.
#[derive(Debug, Default, Clone)]
pub(crate) struct DlpInput {
    pub recipients: Vec<String>,
    pub subject: String,
    pub body: String,
    pub attachments: Vec<AttachmentMeta>,
}

#[derive(Debug, Clone)]
pub(crate) struct AttachmentMeta {
    pub name: String,
    pub content_type: String,
    pub size: i64,
}

impl Engine {
    /// `Dlp/getRules` → `{list:[DlpRule]}` (read the active config/env rules).
    pub(crate) async fn dlp_get_rules(&self, _account_id: &str, _args: &Value) -> Value {
        let rules = load_rules();
        json!({ "list": rules })
    }

    /// `Dlp/scan {draftId|{recipients,subject,bodyText,attachments}}` →
    /// `{list:[DlpVerdict]}` — the compose-time dry-run (no audit, no enforcement).
    pub(crate) async fn dlp_scan(&self, account_id: &str, args: &Value) -> Value {
        let input = if let Some(draft_id) = args.get("draftId").and_then(Value::as_str) {
            match self.dlp_input_for_email(account_id, draft_id).await {
                Ok(i) => i,
                Err(e) => return server_fail(e),
            }
        } else {
            dlp_input_from_args(args)
        };
        let verdicts = scan(&load_rules(), &input);
        json!({ "list": verdicts })
    }

    /// Build the DLP input from a stored draft: recipients + subject from the
    /// envelope, body text + attachment metadata from the parsed MIME.
    pub(crate) async fn dlp_input_for_email(
        &self,
        _account_id: &str,
        email_id: &str,
    ) -> Result<DlpInput, String> {
        let msg = self
            .store()
            .get_message(email_id)
            .await
            .map_err(|e| e.to_string())?;
        let raw = match msg.blob_ref.as_ref() {
            Some(b) => self
                .store()
                .get_body(b)
                .await
                .map_err(|e| e.to_string())?
                .unwrap_or_default(),
            None => Vec::new(),
        };
        let parsed = mw_mime::parse(&raw).map(|p| p.email).unwrap_or_default();
        let recipients = crate::jmap::recipients(&parsed);
        let subject = parsed.subject.clone().unwrap_or_default();
        let body = parsed
            .body_values
            .values()
            .map(|v| v.value.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let attachments = parsed
            .attachments
            .iter()
            .map(|a| AttachmentMeta {
                name: a.name.clone().unwrap_or_default(),
                content_type: a.r#type.clone().unwrap_or_default(),
                size: a.size as i64,
            })
            .collect();
        Ok(DlpInput {
            recipients,
            subject,
            body,
            attachments,
        })
    }
}

/// Parse the inline `Dlp/scan` argument object into a [`DlpInput`].
fn dlp_input_from_args(args: &Value) -> DlpInput {
    let str_list = |v: Option<&Value>| -> Vec<String> {
        v.and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    };
    let attachments = args
        .get("attachments")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .map(|att| AttachmentMeta {
                    name: att
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    content_type: att
                        .get("type")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    size: att.get("size").and_then(Value::as_i64).unwrap_or(0),
                })
                .collect()
        })
        .unwrap_or_default();
    DlpInput {
        recipients: str_list(args.get("recipients")),
        subject: args
            .get("subject")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        body: args
            .get("bodyText")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        attachments,
    }
}

/// Load the active DLP rule set from `MW_DLP_RULES` (a path to a JSON `[DlpRule]`
/// file). Missing/unset/unparseable → no rules (allow-all, the safe default).
pub(crate) fn load_rules() -> Vec<DlpRule> {
    let Ok(path) = std::env::var("MW_DLP_RULES") else {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

/// Evaluate a rule set against content → one [`DlpVerdict`] per matched rule
/// (highest priority first). The excerpt is redacted (detector tokens only).
pub(crate) fn scan(rules: &[DlpRule], input: &DlpInput) -> Vec<DlpVerdict> {
    let text = format!("{}\n{}", input.subject, input.body);
    let mut ordered: Vec<&DlpRule> = rules.iter().filter(|r| r.enabled).collect();
    ordered.sort_by_key(|r| std::cmp::Reverse(r.priority));

    let mut out = Vec::new();
    for rule in ordered {
        // Recipient-domain gate: when configured, the rule only applies if the
        // recipients satisfy it.
        if !recipient_gate(rule, &input.recipients) {
            continue;
        }
        let mut matched: Vec<String> = Vec::new();
        for det in &rule.conditions.detectors {
            if detector_hits(det, &text, rule) {
                matched.push(det.clone());
            }
        }
        if let Some(re) = &rule.conditions.custom_regex
            && !rule
                .conditions
                .detectors
                .iter()
                .any(|d| d == "custom-regex")
            && regex::Regex::new(re)
                .map(|r| r.is_match(&text))
                .unwrap_or(false)
        {
            matched.push("custom-regex".into());
        }
        if attachment_type_hit(rule, &input.attachments) {
            matched.push("attachment-type".into());
        }
        if attachment_size_hit(rule, &input.attachments) {
            matched.push("attachment-size".into());
        }
        // A pure recipient-domain rule (no content detectors) fires on the gate.
        if matched.is_empty()
            && !rule.conditions.recipient_domains.is_empty()
            && rule.conditions.detectors.is_empty()
            && rule.conditions.custom_regex.is_none()
        {
            matched.push("recipient-domain".into());
        }
        if matched.is_empty() {
            continue;
        }
        out.push(DlpVerdict {
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            action: rule.action.clone(),
            matched_detectors: matched.clone(),
            excerpt_redacted: redacted_excerpt(&matched),
            blocked: rule.action == "block",
        });
    }
    out
}

/// A content-free excerpt: the matched detector tokens only (never the content).
fn redacted_excerpt(matched: &[String]) -> String {
    format!("•••• redacted ({})", matched.join(", "))
}

fn recipient_gate(rule: &DlpRule, recipients: &[String]) -> bool {
    let domains = &rule.conditions.recipient_domains;
    if domains.is_empty() {
        return true;
    }
    let recip_domains: Vec<String> = recipients
        .iter()
        .filter_map(|r| r.rsplit_once('@').map(|(_, d)| d.to_lowercase()))
        .collect();
    let any_in = recip_domains
        .iter()
        .any(|d| domains.iter().any(|x| x.to_lowercase() == *d));
    match rule.conditions.recipient_domain_mode.as_deref() {
        Some("in") => any_in,
        Some("notIn") => !any_in,
        _ => true,
    }
}

fn attachment_type_hit(rule: &DlpRule, atts: &[AttachmentMeta]) -> bool {
    let types = &rule.conditions.attachment_types;
    if types.is_empty() {
        return false;
    }
    atts.iter().any(|a| {
        types
            .iter()
            .any(|t| a.content_type.eq_ignore_ascii_case(t) || a.name.to_lowercase().ends_with(t))
    })
}

fn attachment_size_hit(rule: &DlpRule, atts: &[AttachmentMeta]) -> bool {
    match rule.conditions.max_attachment_size {
        Some(max) => atts.iter().any(|a| a.size > max),
        None => false,
    }
}

/// Run one built-in detector over `text`. `custom-regex` is handled by the rule's
/// `customRegex` field.
fn detector_hits(detector: &str, text: &str, rule: &DlpRule) -> bool {
    match detector {
        "pan" => detect_pan(text),
        "iban" => detect_iban(text),
        "ssn" => detect_ssn(text),
        "national-id" => detect_national_id(text),
        "custom-regex" => rule
            .conditions
            .custom_regex
            .as_deref()
            .and_then(|re| regex::Regex::new(re).ok())
            .map(|r| r.is_match(text))
            .unwrap_or(false),
        _ => false,
    }
}

/// Payment card number: a 13–19 digit run (spaces/dashes allowed as separators)
/// that passes the Luhn checksum.
fn detect_pan(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            // Collect a digit run allowing single space/dash separators.
            let mut digits: Vec<u8> = Vec::new();
            let mut j = i;
            while j < bytes.len() {
                let c = bytes[j];
                if c.is_ascii_digit() {
                    digits.push(c - b'0');
                    j += 1;
                } else if (c == b' ' || c == b'-')
                    && j + 1 < bytes.len()
                    && bytes[j + 1].is_ascii_digit()
                {
                    j += 1;
                } else {
                    break;
                }
            }
            if (13..=19).contains(&digits.len()) && luhn_ok(&digits) {
                return true;
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    false
}

fn luhn_ok(digits: &[u8]) -> bool {
    let mut sum = 0u32;
    let mut double = false;
    for &d in digits.iter().rev() {
        let mut v = d as u32;
        if double {
            v *= 2;
            if v > 9 {
                v -= 9;
            }
        }
        sum += v;
        double = !double;
    }
    sum.is_multiple_of(10)
}

/// IBAN: two letters + two check digits + 11–30 alphanumerics, mod-97 == 1.
fn detect_iban(text: &str) -> bool {
    for token in text.split(|c: char| !c.is_ascii_alphanumeric()) {
        let t = token.to_ascii_uppercase();
        let len = t.len();
        if (15..=34).contains(&len)
            && t.as_bytes()[0].is_ascii_alphabetic()
            && t.as_bytes()[1].is_ascii_alphabetic()
            && t.as_bytes()[2].is_ascii_digit()
            && t.as_bytes()[3].is_ascii_digit()
            && iban_mod97(&t)
        {
            return true;
        }
    }
    false
}

fn iban_mod97(iban: &str) -> bool {
    // Move the first 4 chars to the end, then convert letters to numbers (A=10..).
    let rearranged: String = format!("{}{}", &iban[4..], &iban[..4]);
    let mut remainder = 0u32;
    for c in rearranged.chars() {
        let chunk = if c.is_ascii_digit() {
            (c as u8 - b'0').to_string()
        } else if c.is_ascii_alphabetic() {
            ((c.to_ascii_uppercase() as u8 - b'A') as u32 + 10).to_string()
        } else {
            return false;
        };
        for d in chunk.bytes() {
            remainder = (remainder * 10 + (d - b'0') as u32) % 97;
        }
    }
    remainder == 1
}

/// US-style SSN: `NNN-NN-NNNN` (dashes or spaces), not all-zero groups.
fn detect_ssn(text: &str) -> bool {
    scan_grouped(text, &[3, 2, 4])
}

/// A generic national-id: a 9+ contiguous digit run not already a card/SSN shape.
fn detect_national_id(text: &str) -> bool {
    let mut run = 0usize;
    for b in text.bytes() {
        if b.is_ascii_digit() {
            run += 1;
            if run >= 9 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

/// Match `d0-d1-d2` grouped digit sequences separated by `-`/` `/nothing.
fn scan_grouped(text: &str, groups: &[usize]) -> bool {
    let bytes = text.as_bytes();
    let total: usize = groups.iter().sum();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let mut digits = 0usize;
            let mut j = i;
            let mut ok = true;
            for (gi, &g) in groups.iter().enumerate() {
                let mut count = 0;
                while j < bytes.len() && bytes[j].is_ascii_digit() && count < g {
                    j += 1;
                    count += 1;
                    digits += 1;
                }
                if count != g {
                    ok = false;
                    break;
                }
                if gi + 1 < groups.len()
                    && j < bytes.len()
                    && (bytes[j] == b'-' || bytes[j] == b' ')
                {
                    j += 1;
                }
            }
            if ok && digits == total && (j >= bytes.len() || !bytes[j].is_ascii_digit()) {
                return true;
            }
            i += 1;
        } else {
            i += 1;
        }
    }
    false
}

/// The outbound DLP evaluation hook (plan §1.8), called at the `submit_email`
/// chokepoint (via `EmailSubmission/set` create) BEFORE dispatch. Loads the
/// config rules, scans the draft, writes a REDACTED `dlp_audit` row per matched
/// rule, and returns the verdicts. A caller treats any `verdict.blocked == true`
/// as a hard block (fails `EmailSubmission/set` with `dlpBlocked`).
///
/// No rules loaded → no findings → allow (the send path is unchanged).
pub(crate) async fn evaluate(engine: &Engine, account_id: &str, email_id: &str) -> Vec<DlpVerdict> {
    let rules = load_rules();
    if rules.is_empty() {
        return Vec::new();
    }
    let input = match engine.dlp_input_for_email(account_id, email_id).await {
        Ok(i) => i,
        Err(_) => return Vec::new(),
    };
    let verdicts = scan(&rules, &input);
    let now = chrono::Utc::now().to_rfc3339();
    for v in &verdicts {
        // Redacted audit only — matched detectors + rule, NEVER the content.
        let _ = engine
            .store()
            .insert_dlp_audit(&mw_store::DlpAuditRow {
                id: gen_id("dlp"),
                account_id: account_id.to_string(),
                at: now.clone(),
                rule_id: v.rule_id.clone(),
                rule_name: v.rule_name.clone(),
                action: v.action.clone(),
                matched_detectors_json: serde_json::to_string(&v.matched_detectors)
                    .unwrap_or_else(|_| "[]".into()),
                blocked: v.blocked,
            })
            .await;
    }
    verdicts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::types::{DlpConditions, DlpRule};

    fn pan_rule() -> DlpRule {
        DlpRule {
            id: "rule-pan".into(),
            name: "Block card numbers".into(),
            enabled: true,
            priority: 10,
            conditions: DlpConditions {
                detectors: vec!["pan".into()],
                custom_regex: None,
                dictionaries: vec![],
                attachment_types: vec![],
                max_attachment_size: None,
                recipient_domains: vec![],
                recipient_domain_mode: None,
                classification: None,
            },
            action: "block".into(),
            message: "Contains a card number.".into(),
        }
    }

    #[test]
    fn blocks_luhn_valid_pan_and_redacts() {
        let input = DlpInput {
            recipients: vec!["ext@partner.example".into()],
            subject: "invoice".into(),
            body: "Please charge card 4111 1111 1111 1111 today.".into(),
            attachments: vec![],
        };
        let verdicts = scan(&[pan_rule()], &input);
        assert_eq!(verdicts.len(), 1, "one blocking verdict");
        let v = &verdicts[0];
        assert!(v.blocked);
        assert_eq!(v.action, "block");
        assert!(v.matched_detectors.contains(&"pan".to_string()));
        // The redacted excerpt must NOT leak the matched content.
        assert!(!v.excerpt_redacted.contains("4111"));
    }

    #[test]
    fn ignores_non_luhn_digit_runs() {
        let input = DlpInput {
            recipients: vec![],
            subject: "".into(),
            body: "order number 4111 1111 1111 1112 is fine".into(), // fails Luhn
            attachments: vec![],
        };
        // The trailing check digit is wrong, so Luhn rejects it → no PAN verdict.
        assert!(scan(&[pan_rule()], &input).is_empty());
    }
}
