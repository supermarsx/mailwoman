//! Read-only delivery-status reports (RFC 3464 and RFC 6533).
//! All returned text is an unverified claim made by the message sender.
use mail_parser::{MessageParser, MimeHeaders, PartType};
use std::collections::BTreeMap;

const MAX_RAW: usize = 8 * 1024 * 1024;
const MAX_REPORT: usize = 256 * 1024;
const MAX_RECIPIENTS: usize = 64;
const MAX_VALUE: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DsnRecipient {
    pub final_recipient: String,
    pub address_type: String,
    pub original_recipient: Option<String>,
    pub action: String,
    pub status: String,
    pub diagnostic: Option<String>,
    pub remote_mta: Option<String>,
    pub last_attempt: Option<String>,
    pub retry_until: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DsnReport {
    pub reporting_mta: String,
    pub envelope_id: Option<String>,
    pub arrival_date: Option<String>,
    pub recipients: Vec<DsnRecipient>,
}

/// Only direct report parts of a root multipart/report are interpreted. Reports
/// inside a forwarded message are not evidence about the enclosing message.
#[must_use]
pub fn parse_dsn(raw: &[u8]) -> Option<DsnReport> {
    if raw.len() > MAX_RAW {
        return None;
    }
    let message = MessageParser::default().parse(raw)?;
    let root = message.root_part();
    let ct = root.content_type()?;
    if !ct.ctype().eq_ignore_ascii_case("multipart")
        || !ct.subtype()?.eq_ignore_ascii_case("report")
    {
        return None;
    }
    let kind = ct.attribute("report-type")?;
    if !matches!(
        kind.to_ascii_lowercase().as_str(),
        "delivery-status" | "global-delivery-status"
    ) {
        return None;
    }
    let PartType::Multipart(children) = &root.body else {
        return None;
    };
    let mut reports = children
        .iter()
        .filter_map(|id| message.parts.get(*id as usize))
        .filter(|part| {
            part.content_type().is_some_and(|ct| {
                ct.ctype().eq_ignore_ascii_case("message")
                    && ct
                        .subtype()
                        .is_some_and(|sub| sub.eq_ignore_ascii_case(kind))
            })
        });
    let part = reports.next()?;
    if reports.next().is_some() {
        return None;
    }
    let bytes = part.contents();
    if bytes.len() > MAX_REPORT {
        return None;
    }
    let text = std::str::from_utf8(bytes).ok()?;
    if kind.eq_ignore_ascii_case("delivery-status") && !text.is_ascii() {
        return None;
    }
    parse_fields(text)
}

fn clean(value: &str) -> String {
    value.chars().filter(|c| !c.is_control() && !matches!(*c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')).collect::<String>().trim().to_owned()
}

fn fields(block: &[String]) -> Option<BTreeMap<String, String>> {
    if block.len() > 64 {
        return None;
    }
    let mut fields = BTreeMap::new();
    for line in block {
        let (key, value) = line.split_once(':')?;
        if key.is_empty()
            || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || value.len() > MAX_VALUE
        {
            return None;
        }
        if fields
            .insert(key.to_ascii_lowercase(), clean(value))
            .is_some()
        {
            return None;
        }
    }
    Some(fields)
}

fn typed(value: &str) -> Option<(String, String)> {
    let (kind, value) = value.split_once(';')?;
    let kind = kind.trim();
    let value = value.trim();
    if kind.is_empty()
        || value.is_empty()
        || !kind.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return None;
    }
    Some((kind.to_ascii_lowercase(), value.to_owned()))
}

fn parse_fields(text: &str) -> Option<DsnReport> {
    let mut blocks: Vec<Vec<String>> = vec![Vec::new()];
    for line in text.lines() {
        if line.is_empty() {
            if !blocks.last()?.is_empty() {
                blocks.push(Vec::new());
            }
        } else if line.starts_with([' ', '\t']) {
            let last = blocks.last_mut()?.last_mut()?;
            if last.len() + line.len() > MAX_VALUE {
                return None;
            }
            last.push(' ');
            last.push_str(line.trim());
        } else {
            if line.len() > MAX_VALUE || blocks.last()?.len() >= 64 {
                return None;
            }
            blocks.last_mut()?.push(line.to_owned());
        }
        if blocks.len() > MAX_RECIPIENTS + 2 {
            return None;
        }
    }
    if blocks.last()?.is_empty() {
        blocks.pop();
    }
    if blocks.len() < 2 || blocks.len() > MAX_RECIPIENTS + 1 {
        return None;
    }
    let general = fields(&blocks[0])?;
    let (_, reporting_mta) = typed(general.get("reporting-mta")?)?;
    let mut recipients = Vec::new();
    for block in &blocks[1..] {
        let f = fields(block)?;
        let (address_type, final_recipient) = typed(f.get("final-recipient")?)?;
        let action = f.get("action")?.to_ascii_lowercase();
        if !matches!(
            action.as_str(),
            "failed" | "delayed" | "delivered" | "relayed" | "expanded"
        ) {
            return None;
        }
        let status_field = f.get("status")?;
        let status = match status_field.split_once('(') {
            Some((code, comment)) if comment.ends_with(')') => code.trim().to_owned(),
            Some(_) => return None,
            None => status_field.trim().to_owned(),
        };
        let parts: Vec<_> = status.split('.').collect();
        if parts.len() != 3
            || !matches!(parts[0], "2" | "4" | "5")
            || parts[1..].iter().any(|p| {
                p.is_empty()
                    || p.len() > 3
                    || (p.len() > 1 && p.starts_with('0'))
                    || !p.bytes().all(|b| b.is_ascii_digit())
            })
        {
            return None;
        }
        // RFC 3464 section 2.3.3 explicitly permits failed with a 4.x.x
        // status after retries are abandoned. Never derive action from status.
        let original_recipient = match f.get("original-recipient") {
            Some(value) => Some(typed(value)?.1),
            None => None,
        };
        recipients.push(DsnRecipient {
            final_recipient,
            address_type,
            original_recipient,
            action,
            status,
            diagnostic: f.get("diagnostic-code").cloned(),
            remote_mta: f.get("remote-mta").cloned(),
            last_attempt: f.get("last-attempt-date").cloned(),
            retry_until: f.get("will-retry-until").cloned(),
        });
    }
    Some(DsnReport {
        reporting_mta,
        envelope_id: general.get("original-envelope-id").cloned(),
        arrival_date: general.get("arrival-date").cloned(),
        recipients,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn report(kind: &str, fields: &str) -> Vec<u8> {
        format!("From: mailer-daemon@example.org\r\nContent-Type: multipart/report; report-type={kind}; boundary=b\r\n\r\n--b\r\nContent-Type: text/plain\r\n\r\nDelivery report\r\n--b\r\nContent-Type: message/{kind}\r\n\r\n{fields}\r\n--b--\r\n").into_bytes()
    }
    const FAILED: &str = "Reporting-MTA: dns; mx.example.org\r\nOriginal-Envelope-Id: 123\r\n\r\nFinal-Recipient: rfc822; user@example.org\r\nAction: failed\r\nStatus: 5.1.1\r\nDiagnostic-Code: smtp; 550 user unknown\r\n continued diagnostic";
    #[test]
    fn failed_and_multiple_recipients_are_separate_claims() {
        let r = parse_dsn(&report("delivery-status", FAILED)).unwrap();
        assert_eq!(r.reporting_mta, "mx.example.org");
        assert_eq!(
            r.recipients[0].diagnostic.as_deref(),
            Some("smtp; 550 user unknown continued diagnostic")
        );
        let raw = report(
            "delivery-status",
            &format!(
                "{FAILED}\r\n\r\nFinal-Recipient: rfc822; other@example.org\r\nAction: delayed\r\nStatus: 4.2.0\r\nWill-Retry-Until: Tue, 27 Oct 2026 01:00:00 +0000"
            ),
        );
        let r = parse_dsn(&raw).unwrap();
        assert_eq!(r.recipients.len(), 2);
        assert_eq!(r.recipients[1].action, "delayed");
        assert!(r.recipients[1].retry_until.is_some());
    }
    #[test]
    fn success_actions_do_not_mean_read() {
        for action in ["delivered", "relayed", "expanded"] {
            let text = FAILED
                .replace("Action: failed", &format!("Action: {action}"))
                .replace("5.1.1", "2.0.0");
            assert_eq!(
                parse_dsn(&report("delivery-status", &text))
                    .unwrap()
                    .recipients[0]
                    .action,
                action
            );
        }
    }
    #[test]
    fn utf8_requires_global_type_and_removes_directional_controls() {
        let text = FAILED
            .replace(
                "rfc822; user@example.org",
                "utf-8; utilizador\u{202e}@exemplo.pt",
            )
            .replace("user unknown", "destinatário inválido");
        assert!(parse_dsn(&report("delivery-status", &text)).is_none());
        let r = parse_dsn(&report("global-delivery-status", &text)).unwrap();
        assert_eq!(r.recipients[0].final_recipient, "utilizador@exemplo.pt");
        assert_eq!(r.recipients[0].address_type, "utf-8");
    }
    #[test]
    fn malformed_ambiguous_and_oversized_reports_are_not_interpreted() {
        for text in [
            FAILED.replace("5.1.1", "5.1"),
            FAILED.replace("Action: failed", "Action: unknown"),
            FAILED.replace("Action: failed", "Action: failed\r\nAction: delivered"),
            FAILED.replace("Reporting-MTA:", "Not-Reporting-MTA:"),
            FAILED.replace("Final-Recipient:", "Not-Final-Recipient:"),
        ] {
            assert!(parse_dsn(&report("delivery-status", &text)).is_none());
        }
        assert!(
            parse_dsn(&report(
                "delivery-status",
                &FAILED.replace("user unknown", &"x".repeat(MAX_VALUE + 1))
            ))
            .is_none()
        );
        assert!(parse_dsn(&vec![b'a'; MAX_RAW + 1]).is_none());
        let recipient = "\r\n\r\nFinal-Recipient: rfc822; x@y\r\nAction: failed\r\nStatus: 5.0.0";
        assert!(
            parse_dsn(&report(
                "delivery-status",
                &format!(
                    "Reporting-MTA: dns; mx{}",
                    recipient.repeat(MAX_RECIPIENTS + 1)
                )
            ))
            .is_none()
        );
    }
    #[test]
    fn forwarded_or_wrong_report_types_are_ignored() {
        let raw = report("delivery-status", FAILED);
        let forwarded = format!(
            "Content-Type: message/rfc822\r\n\r\n{}",
            String::from_utf8(raw).unwrap()
        );
        assert!(parse_dsn(forwarded.as_bytes()).is_none());
        assert!(parse_dsn(&report("disposition-notification", FAILED)).is_none());
    }
}

#[cfg(test)]
#[test]
fn failed_can_retain_temporary_failure_status_after_retry_exhaustion() {
    let fields = "Reporting-MTA: dns; mx\n\nFinal-Recipient: rfc822; x@y\nAction: failed\nStatus: 4.4.7 (delivery time expired)";
    let report = parse_fields(fields).unwrap();
    assert_eq!(report.recipients[0].action, "failed");
    assert_eq!(report.recipients[0].status, "4.4.7");
    assert!(
        parse_fields(&fields.replace("4.4.7 (delivery time expired)", "4.4.7 garbage")).is_none()
    );
}
