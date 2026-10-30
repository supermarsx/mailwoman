//! Bounded, non-lossy RFC 9007 projection of RFC 8098 reports.
use std::collections::BTreeMap;

use mail_parser::{MessageParser, MimeHeaders, PartType};

use crate::{MdnActionMode, MdnDisposition, MdnSendingMode};

/// Parsed message data only. Association with an account-owned Email is the
/// host's responsibility; this parser never treats a Message-ID as authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MdnDocument {
    pub subject: Option<String>,
    pub text_body: Option<String>,
    pub include_original_message: bool,
    pub reporting_ua: Option<String>,
    pub action_mode: MdnActionMode,
    pub sending_mode: MdnSendingMode,
    pub disposition: MdnDisposition,
    pub mdn_gateway: Option<String>,
    pub original_recipient: Option<String>,
    pub final_recipient: Option<String>,
    pub original_message_id: Option<String>,
    pub error: Vec<String>,
    pub extension_fields: BTreeMap<String, String>,
}

/// Parse a root report, never a forwarded/nested receipt. Unlike the lenient UI
/// summary parser, this rejects oversize or ambiguous fields rather than
/// returning a truncated representation. All returned text remains untrusted.
#[must_use]
pub fn parse_mdn_document(raw: &[u8]) -> Option<MdnDocument> {
    if raw.len() > 9 * 1024 * 1024 {
        return None;
    }
    let message = MessageParser::default().parse(raw)?;
    let root = message.root_part();
    let content_type = root.content_type()?;
    if !is_type(content_type, "multipart", "report")
        || !content_type.attribute("report-type").is_some_and(|kind| {
            kind.eq_ignore_ascii_case("disposition-notification")
                || kind.eq_ignore_ascii_case("global-disposition-notification")
        })
    {
        return None;
    }
    let PartType::Multipart(children) = &root.body else {
        return None;
    };
    if !(2..=3).contains(&children.len()) {
        return None;
    }
    let notification = message.parts.get(children[1] as usize)?;
    let kind = notification.content_type()?;
    if !is_type(kind, "message", "disposition-notification")
        && !is_type(kind, "message", "global-disposition-notification")
    {
        return None;
    }
    let bytes = notification.contents();
    if bytes.len() > 64 * 1024 {
        return None;
    }
    let text = std::str::from_utf8(bytes).ok()?;
    let mut fields: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        if line.starts_with([' ', '\t']) {
            let (_, value) = fields.last_mut()?;
            value.push(' ');
            value.push_str(line.trim());
            if value.len() > 8192 {
                return None;
            }
        } else {
            let (name, value) = line.split_once(':')?;
            if name.is_empty()
                || name.len() > 64
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_graphic() && byte != b':')
                || fields.len() >= 64
            {
                return None;
            }
            let value = value.trim();
            if value.len() > 8192 {
                return None;
            }
            fields.push((name.to_owned(), value.to_owned()));
        }
    }
    let mut known = BTreeMap::new();
    let mut extensions = BTreeMap::new();
    let mut errors = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (name, value) in fields {
        if value.chars().any(|c| c.is_control() && c != '\t') {
            return None;
        }
        let lower = name.to_ascii_lowercase();
        if lower == "error" {
            errors.push(value);
            continue;
        }
        if !seen.insert(lower.clone()) {
            return None;
        }
        if [
            "reporting-ua",
            "mdn-gateway",
            "original-recipient",
            "final-recipient",
            "original-message-id",
            "disposition",
        ]
        .contains(&lower.as_str())
        {
            known.insert(lower, value);
        } else {
            extensions.insert(name, value);
        }
    }
    let disposition = without_comments(known.get("disposition")?)?;
    let (modes, kind) = disposition.split_once(';')?;
    let (action, sending) = modes.trim().split_once('/')?;
    let action_mode = match action.trim().to_ascii_lowercase().as_str() {
        "manual-action" => MdnActionMode::Manual,
        "automatic-action" => MdnActionMode::Automatic,
        _ => return None,
    };
    let sending_mode = match sending.trim().to_ascii_lowercase().as_str() {
        "mdn-sent-manually" => MdnSendingMode::Manual,
        "mdn-sent-automatically" => MdnSendingMode::Automatic,
        _ => return None,
    };
    let disposition = match kind.split('/').next()?.trim().to_ascii_lowercase().as_str() {
        "displayed" => MdnDisposition::Displayed,
        "deleted" => MdnDisposition::Deleted,
        "dispatched" => MdnDisposition::Dispatched,
        "processed" => MdnDisposition::Processed,
        _ => return None,
    };
    // Final-Recipient is mandatory in the underlying MDN wire format.
    let final_recipient = known.remove("final-recipient")?;
    let (address_type, address) = final_recipient.split_once(';')?;
    if address_type.trim().is_empty() || address.trim().is_empty() {
        return None;
    }
    let human = message.parts.get(children[0] as usize)?;
    let text_body = if human
        .content_type()
        .is_some_and(|kind| is_type(kind, "text", "plain"))
    {
        match &human.body {
            PartType::Text(text) => Some(text.to_string()),
            _ => None,
        }
    } else {
        None
    };
    if text_body
        .as_ref()
        .is_some_and(|value| value.len() > 64 * 1024)
    {
        return None;
    }
    let subject = message.subject().map(str::to_owned);
    if subject.as_ref().is_some_and(|value| value.len() > 4096) {
        return None;
    }
    let include_original_message = children
        .get(2)
        .and_then(|id| message.parts.get(*id as usize))
        .is_some_and(|part| {
            part.content_type().is_some_and(|kind| {
                is_type(kind, "message", "rfc822") || is_type(kind, "message", "global")
            })
        });
    Some(MdnDocument {
        subject,
        text_body,
        include_original_message,
        reporting_ua: known.remove("reporting-ua"),
        action_mode,
        sending_mode,
        disposition,
        mdn_gateway: known.remove("mdn-gateway"),
        original_recipient: known.remove("original-recipient"),
        final_recipient: Some(final_recipient),
        original_message_id: known.remove("original-message-id"),
        error: errors,
        extension_fields: extensions,
    })
}

fn is_type(kind: &mail_parser::ContentType<'_>, primary: &str, subtype: &str) -> bool {
    kind.ctype().eq_ignore_ascii_case(primary)
        && kind
            .subtype()
            .is_some_and(|value| value.eq_ignore_ascii_case(subtype))
}

fn without_comments(value: &str) -> Option<String> {
    let mut output = String::new();
    let mut depth = 0_u8;
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        match character {
            '\\' if depth > 0 => escaped = true,
            '(' => {
                depth = depth.checked_add(1)?;
                if depth > 8 {
                    return None;
                }
                if depth == 1 {
                    output.push(' ');
                }
            }
            ')' => {
                depth = depth.checked_sub(1)?;
            }
            _ if depth == 0 => output.push(character),
            _ => {}
        }
    }
    (depth == 0 && !escaped).then_some(output)
}
