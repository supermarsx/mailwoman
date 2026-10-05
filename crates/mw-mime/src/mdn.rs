//! Message disposition notifications ("read receipts"), RFC 8098.
//!
//! [`build_mdn`] writes the report a recipient's program sends back to the
//! address in `Disposition-Notification-To`; [`parse_mdn`] reads one that has
//! arrived. Whether a report may be sent at all (the user's choice, the
//! `Return-Path` comparison, list mail — RFC 8098 §2.1) is the caller's
//! decision; [`crate::ReceiptHeaders`] carries what that decision needs.
//!
//! Not implemented: the UTF-8 forms of RFC 6533 (`utf-8;` address type,
//! `message/global-disposition-notification` on build), the optional third
//! part holding the original message or its headers, and the `Reporting-UA`,
//! `MDN-Gateway` and `Error` fields on build.

use mail_builder::MessageBuilder;
use mail_builder::headers::content_type::ContentType;
use mail_builder::headers::raw::Raw;
use mail_builder::mime::{BodyPart, MimePart};
use mail_parser::{MessageParser, MimeHeaders};
use mw_jmap::EmailAddress;

use crate::MimeError;
use crate::build::address_header;
use crate::check::{line_text, mailbox_problem, no_control, random_token, refuse};

/// Longest `Original-Message-ID` written (without angle brackets); with the
/// field name the line stays under RFC 5322's 998-octet limit.
const MAX_MESSAGE_ID_BYTES: usize = 800;

/// How many characters of the original subject are quoted.
const SUBJECT_CHARS: usize = 200;

/// How much of a `message/disposition-notification` part is read.
const MAX_FIELD_BYTES: usize = 64 * 1024;

/// Longest value kept from a parsed field, in characters.
const MAX_VALUE_CHARS: usize = 1000;

/// Most disposition modifiers kept from a parsed report.
const MAX_MODIFIERS: usize = 8;

/// RFC 8098 `action-mode`: whether the disposition was the user's own act.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MdnActionMode {
    /// `manual-action` — the user did it (opened the message, deleted it …).
    Manual,
    /// `automatic-action` — a program did it without the user (a filter).
    Automatic,
}

/// RFC 8098 `sending-mode`: whether the user approved this report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MdnSendingMode {
    /// `MDN-sent-manually` — the user agreed to this report being sent.
    Manual,
    /// `MDN-sent-automatically` — sent under a standing setting.
    Automatic,
}

/// RFC 8098 `disposition-type`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MdnDisposition {
    /// `displayed`. Not a statement that the message was read or understood.
    Displayed,
    /// `deleted`. The recipient may or may not have seen the message.
    Deleted,
    /// `dispatched` — passed on without being displayed.
    Dispatched,
    /// `processed` — handled without being displayed.
    Processed,
    /// Any other type in a received report (RFC 3798's `denied` and `failed`,
    /// or an unknown word), lower-cased. [`build_mdn`] refuses it.
    Other(String),
}

impl MdnDisposition {
    /// The word written in the `Disposition` field.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Displayed => "displayed",
            Self::Deleted => "deleted",
            Self::Dispatched => "dispatched",
            Self::Processed => "processed",
            Self::Other(word) => word,
        }
    }
}

/// What [`build_mdn`] needs to write a report.
#[derive(Debug, Clone)]
pub struct MdnInput {
    /// `From` of the report: the recipient of the original message.
    pub from: EmailAddress,
    /// `To` of the report: the mailbox the original named in
    /// `Disposition-Notification-To`
    /// ([`ReceiptHeaders::disposition_notification_to`](crate::ReceiptHeaders)).
    pub to: String,
    /// `Final-Recipient`: the mailbox the original was delivered to.
    pub final_recipient: String,
    /// `Original-Recipient`, when the original carried the address it was
    /// first sent to and that differs from the final one.
    pub original_recipient: Option<String>,
    /// `Message-ID` of the original (brackets optional). The field is left
    /// out when `None`.
    pub original_message_id: Option<String>,
    /// Subject of the original, quoted in the report's subject and text. Its
    /// control characters become spaces and it is cut to 200 characters.
    pub original_subject: Option<String>,
    /// See [`MdnActionMode`].
    pub action_mode: MdnActionMode,
    /// See [`MdnSendingMode`].
    pub sending_mode: MdnSendingMode,
    /// What happened to the original.
    pub disposition: MdnDisposition,
    /// `Message-ID` of the report itself (brackets optional). When `None`,
    /// `mail-builder` makes one.
    pub message_id: Option<String>,
}

/// A received report, as read by [`parse_mdn`].
///
/// Every string comes from a message someone else wrote. Control characters
/// have been removed and lengths are bounded, but nothing else is promised:
/// `final_recipient` need not be a mailbox, and `original_message_id` need not
/// be the id of any message this account sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MdnReport {
    /// The address in `Final-Recipient`, after the `rfc822;` address type.
    pub final_recipient: Option<String>,
    /// The address in `Original-Recipient`, likewise.
    pub original_recipient: Option<String>,
    /// `Original-Message-ID`, angle brackets stripped.
    pub original_message_id: Option<String>,
    /// `None` when the field has no action mode or an unknown one.
    pub action_mode: Option<MdnActionMode>,
    /// `None` when the field has no sending mode or an unknown one.
    pub sending_mode: Option<MdnSendingMode>,
    /// The disposition type.
    pub disposition: MdnDisposition,
    /// Disposition modifiers (`error`, …), lower-cased; at most eight.
    pub modifiers: Vec<String>,
    /// `Reporting-UA`, as written.
    pub reporting_ua: Option<String>,
}

/// Build an RFC 8098 report: `multipart/report;
/// report-type=disposition-notification` with a human-readable `text/plain`
/// part and a `message/disposition-notification` part carrying
/// `Original-Recipient` (when given), `Final-Recipient`,
/// `Original-Message-ID` (when given) and `Disposition`. No `Reporting-UA` is
/// written, so the report names neither a host nor a program version.
/// A report sent automatically also carries `Auto-Submitted: auto-replied`
/// (RFC 3834).
///
/// Refused, whatever the caller did before:
/// - `from.email`, `to`, `final_recipient` or `original_recipient` that is not
///   one mailbox by [`validate_mailbox`](crate::validate_mailbox)'s rules;
/// - `final_recipient` or `original_recipient` holding a non-ASCII character
///   (the `rfc822` address type is ASCII; RFC 6533 is not implemented);
/// - an `original_message_id` that, with one pair of angle brackets removed,
///   is empty, longer than 800 bytes, or holds anything but printable ASCII
///   other than space, `<` and `>`;
/// - a control character in `message_id`;
/// - [`MdnDisposition::Other`].
///
/// `original_subject` is free text and is never refused: it is reduced to one
/// line. It cannot add a part either, because the boundary is new for every
/// report and not predictable from the original message.
///
/// The envelope is the caller's: RFC 8098 §3 requires the null reverse path.
pub fn build_mdn(input: &MdnInput) -> Result<Vec<u8>, MimeError> {
    for (what, addr) in [
        ("from address", &input.from.email),
        ("to address", &input.to),
    ] {
        if let Some(why) = mailbox_problem(addr) {
            return Err(refuse(what, addr, why));
        }
    }
    let mut fields = String::new();
    if let Some(addr) = &input.original_recipient {
        fields.push_str(&recipient_field("Original-Recipient", addr)?);
    }
    fields.push_str(&recipient_field("Final-Recipient", &input.final_recipient)?);
    if let Some(id) = &input.original_message_id {
        let id = id.trim();
        let id = id
            .strip_prefix('<')
            .and_then(|v| v.strip_suffix('>'))
            .unwrap_or(id);
        let why = if id.is_empty() {
            Some("empty")
        } else if id.len() > MAX_MESSAGE_ID_BYTES {
            Some("longer than 800 bytes")
        } else if !id
            .bytes()
            .all(|b| b.is_ascii_graphic() && b != b'<' && b != b'>')
        {
            Some("not printable ASCII without spaces or angle brackets")
        } else {
            None
        };
        if let Some(why) = why {
            return Err(refuse("original message id", id, why));
        }
        fields.push_str(&format!("Original-Message-ID: <{id}>\r\n"));
    }
    if let MdnDisposition::Other(word) = &input.disposition {
        return Err(refuse("disposition", word, "not a type this crate writes"));
    }
    fields.push_str(&format!(
        "Disposition: {}/{}; {}\r\n",
        match input.action_mode {
            MdnActionMode::Manual => "manual-action",
            MdnActionMode::Automatic => "automatic-action",
        },
        match input.sending_mode {
            MdnSendingMode::Manual => "MDN-sent-manually",
            MdnSendingMode::Automatic => "MDN-sent-automatically",
        },
        input.disposition.as_str(),
    ));

    let subject = input
        .original_subject
        .as_deref()
        .map(|s| line_text(s).trim().chars().take(SUBJECT_CHARS).collect())
        .filter(|s: &String| !s.is_empty());
    let what = input.disposition.as_str();
    let headline = match &subject {
        Some(s) => format!("Return receipt ({what}) - {s}"),
        None => format!("Return receipt ({what})"),
    };
    let mut text = format!(
        "This is a return receipt for the message sent to {}",
        input.final_recipient
    );
    if let Some(s) = &subject {
        text.push_str(&format!(" with the subject \"{s}\""));
    }
    text.push_str(".\r\n\r\n");
    text.push_str(match input.disposition {
        MdnDisposition::Displayed => {
            "The message was displayed. This does not show that it was read or understood."
        }
        MdnDisposition::Deleted => {
            "The message was deleted. The recipient may or may not have seen it."
        }
        MdnDisposition::Dispatched => "The message was passed on without being displayed.",
        MdnDisposition::Processed | MdnDisposition::Other(_) => {
            "The message was processed without being displayed."
        }
    });
    text.push_str("\r\n");

    // `From` and `To` are written by `address_header`, ahead of what the
    // builder writes, so the display name is one phrase whatever it holds.
    let to = EmailAddress {
        name: None,
        email: input.to.clone(),
    };
    let io = |e: std::io::Error| MimeError::Build(e.to_string());
    let mut out = Vec::new();
    address_header(&mut out, "From", std::slice::from_ref(&input.from), false).map_err(io)?;
    address_header(&mut out, "To", std::slice::from_ref(&to), false).map_err(io)?;

    let mut b = MessageBuilder::new().subject(headline);
    if let Some(id) = &input.message_id {
        no_control("message id", id)?;
        b = b.message_id(
            id.trim()
                .trim_start_matches('<')
                .trim_end_matches('>')
                .to_string(),
        );
    }
    if input.sending_mode == MdnSendingMode::Automatic {
        b = b.header("Auto-Submitted", Raw::new("auto-replied"));
    }

    let report = ContentType::new("multipart/report")
        .attribute("report-type", "disposition-notification")
        .attribute("boundary", format!("mw_{}", random_token()));
    // A binary body with an explicit transfer encoding is written as it
    // stands; the fields are ASCII lines ending in CRLF, which is what `7bit`
    // declares.
    let notification = MimePart::new(
        ContentType::new("message/disposition-notification"),
        BodyPart::Binary(fields.into_bytes().into()),
    )
    .transfer_encoding("7bit");
    b.body(MimePart::new(
        report,
        vec![
            MimePart::new("text/plain", BodyPart::Text(text.into())),
            notification,
        ],
    ))
    .write_to(&mut out)
    .map_err(io)?;
    Ok(out)
}

/// One `<name>: rfc822; <addr>` line, or the refusal.
fn recipient_field(name: &str, addr: &str) -> Result<String, MimeError> {
    let why = match mailbox_problem(addr) {
        None if !addr.is_ascii() => Some("not ASCII (RFC 6533 is not implemented)"),
        other => other,
    };
    match why {
        Some(why) => Err(refuse(&name.to_ascii_lowercase(), addr, why)),
        None => Ok(format!("{name}: rfc822; {addr}\r\n")),
    }
}

/// Read a received RFC 8098 report.
///
/// `Some` when the message is `multipart/report` (with `report-type` either
/// absent or `disposition-notification`), has a part of type
/// `message/disposition-notification` or
/// `message/global-disposition-notification`, and that part has a
/// `Disposition` field naming a type. Everything else about the report is
/// optional and read leniently: field names in any case, folded lines, bare LF
/// line ends, a missing address type, a missing mode pair, and any transfer
/// encoding `mail-parser` decodes. A field that appears twice is read from its
/// first occurrence.
///
/// Runs over untrusted bytes and never panics. A report nested in another
/// message (a forwarded receipt) is not reported.
#[must_use]
pub fn parse_mdn(raw: &[u8]) -> Option<MdnReport> {
    let message = MessageParser::default().parse(raw)?;
    let root = message.root_part().content_type()?;
    if !root.ctype().eq_ignore_ascii_case("multipart")
        || !root
            .subtype()
            .is_some_and(|s| s.eq_ignore_ascii_case("report"))
        || root
            .attribute("report-type")
            .is_some_and(|t| !t.trim().eq_ignore_ascii_case("disposition-notification"))
    {
        return None;
    }
    let part = message.parts.iter().find(|p| {
        p.content_type().is_some_and(|ct| {
            ct.ctype().eq_ignore_ascii_case("message")
                && ct.subtype().is_some_and(|s| {
                    s.eq_ignore_ascii_case("disposition-notification")
                        || s.eq_ignore_ascii_case("global-disposition-notification")
                })
        })
    })?;
    let contents = part.contents();
    parse_fields(&String::from_utf8_lossy(
        &contents[..contents.len().min(MAX_FIELD_BYTES)],
    ))
}

/// Read the fields of a `message/disposition-notification` body.
fn parse_fields(text: &str) -> Option<MdnReport> {
    // Unfold: a line that starts with a space or a tab continues the one
    // before it.
    let mut lines: Vec<String> = Vec::new();
    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        match lines.last_mut() {
            Some(last) if line.starts_with([' ', '\t']) => {
                last.push(' ');
                last.push_str(line.trim());
            }
            _ => lines.push(line.to_string()),
        }
    }
    let field = |name: &str| {
        lines.iter().find_map(|line| {
            let (n, v) = line.split_once(':')?;
            n.trim().eq_ignore_ascii_case(name).then(|| clean(v))
        })
    };

    let disposition = field("Disposition")?;
    let (modes, kind) = match disposition.split_once(';') {
        Some((modes, kind)) => (modes, kind),
        None => ("", disposition.as_str()),
    };
    let (kind, modifiers) = match kind.split_once('/') {
        Some((kind, modifiers)) => (kind, modifiers),
        None => (kind, ""),
    };
    let kind = kind.trim().to_ascii_lowercase();
    if kind.is_empty() {
        return None;
    }
    let (action, sending) = match modes.split_once('/') {
        Some((action, sending)) => (action.trim(), sending.trim()),
        None => (modes.trim(), ""),
    };

    Some(MdnReport {
        final_recipient: field("Final-Recipient").and_then(|v| typed_address(&v)),
        original_recipient: field("Original-Recipient").and_then(|v| typed_address(&v)),
        original_message_id: field("Original-Message-ID")
            .map(|v| {
                v.chars()
                    .filter(|c| !c.is_whitespace() && *c != '<' && *c != '>')
                    .collect::<String>()
            })
            .filter(|v| !v.is_empty()),
        action_mode: if action.eq_ignore_ascii_case("manual-action") {
            Some(MdnActionMode::Manual)
        } else if action.eq_ignore_ascii_case("automatic-action") {
            Some(MdnActionMode::Automatic)
        } else {
            None
        },
        sending_mode: if sending.eq_ignore_ascii_case("MDN-sent-manually") {
            Some(MdnSendingMode::Manual)
        } else if sending.eq_ignore_ascii_case("MDN-sent-automatically") {
            Some(MdnSendingMode::Automatic)
        } else {
            None
        },
        disposition: match kind.as_str() {
            "displayed" => MdnDisposition::Displayed,
            "deleted" => MdnDisposition::Deleted,
            "dispatched" => MdnDisposition::Dispatched,
            "processed" => MdnDisposition::Processed,
            _ => MdnDisposition::Other(kind),
        },
        modifiers: modifiers
            .split(',')
            .map(|m| m.trim().to_ascii_lowercase())
            .filter(|m| !m.is_empty())
            .take(MAX_MODIFIERS)
            .collect(),
        reporting_ua: field("Reporting-UA").filter(|v| !v.is_empty()),
    })
}

/// A field value with control characters removed, trimmed and cut to
/// [`MAX_VALUE_CHARS`].
fn clean(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_VALUE_CHARS)
        .collect::<String>()
        .trim()
        .to_string()
}

/// The address of an `address-type; address` value (or the whole value when
/// there is no `;`), angle brackets stripped. `None` when nothing is left.
fn typed_address(value: &str) -> Option<String> {
    let addr = value.split_once(';').map_or(value, |(_, addr)| addr).trim();
    let addr = addr
        .strip_prefix('<')
        .and_then(|v| v.strip_suffix('>'))
        .unwrap_or(addr)
        .trim();
    (!addr.is_empty()).then(|| addr.to_string())
}
