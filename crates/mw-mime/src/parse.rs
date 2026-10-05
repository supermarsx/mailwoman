//! Raw RFC822 bytes → [`mw_jmap::Email`] + [`ParsedEnvelope`] (plan §2.2).
//!
//! The mapping targets exactly the `Email` properties the V0 web UI reads back
//! from `Email/get`: `subject`, address lists, `sentAt`/`receivedAt`, `preview`,
//! `hasAttachment`, `size`, `textBody`/`htmlBody` part metadata and the decoded
//! `bodyValues`, plus the threading properties `messageId`, `inReplyTo` and
//! `references` a client needs to write the headers of a reply. Ids, `mailboxIds`, `keywords` and `threadId` are the engine's
//! job — they are left at their defaults here.

use std::collections::HashMap;

use mail_parser::{
    Address, DateTime, HeaderValue, Message, MessageParser, MessagePart, MimeHeaders, PartType,
};
use mw_jmap::{Email, EmailAddress, EmailBodyPart, EmailBodyValue};

use crate::MimeError;

/// Preview length in characters (JMAP `preview` is a short text snippet).
const PREVIEW_LEN: usize = 256;

/// The result of [`parse`]: the JMAP object plus the threading envelope.
#[derive(Debug, Clone)]
pub struct Parsed {
    /// The mapped JMAP email (engine fills id/mailboxIds/keywords/threadId).
    pub email: Email,
    /// Threading headers the engine's JWZ pass consumes.
    pub envelope: ParsedEnvelope,
    /// What decides whether, and to whom, a read receipt may be sent.
    pub receipt: ReceiptHeaders,
}

/// The headers of a received message that bear on a read receipt (RFC 8098
/// §2.1): who asked for one, where the message says it came from, and whether
/// it is list or bulk mail.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReceiptHeaders {
    /// The mailbox in `Disposition-Notification-To`, without display name or
    /// angle brackets. `None` when the header is absent, appears more than
    /// once, names more than one address or a group, or names an address that
    /// fails [`validate_mailbox`](crate::validate_mailbox) — a request this
    /// crate would refuse to answer is reported as no request.
    pub disposition_notification_to: Option<String>,
    /// The first `Return-Path` header, angle brackets stripped. The null path
    /// `<>` is `Some("")`; `None` when the header is absent or is not one
    /// path without whitespace or control characters.
    pub return_path: Option<String>,
    /// Whether the message carries `List-Id`, or `Precedence` with the value
    /// `bulk`, `list` or `junk`.
    pub from_list: bool,
}

/// Threading-relevant headers extracted alongside the [`Email`].
///
/// The engine keys JWZ threading off these (`message_id` as the node identity,
/// `in_reply_to` + `references` as parent links) — see plan §1.7.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedEnvelope {
    /// `Message-ID` (angle brackets stripped), if present.
    pub message_id: Option<String>,
    /// `In-Reply-To` (first id, angle brackets stripped), if present.
    pub in_reply_to: Option<String>,
    /// `References` chain in order (angle brackets stripped).
    pub references: Vec<String>,
}

/// Parse raw RFC822 bytes into a JMAP [`Email`] and its [`ParsedEnvelope`].
///
/// Runs inside the render jail over untrusted bytes. Never panics; returns
/// [`MimeError::Parse`] only when `mail-parser` cannot recognise a message at
/// all (in practice it is extremely lenient, so most malformed input still maps
/// to a best-effort partial `Email`).
pub fn parse(raw: &[u8]) -> Result<Parsed, MimeError> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or_else(|| MimeError::Parse("input is not a recognisable RFC5322 message".into()))?;
    Ok(Parsed {
        email: map_email(&message, raw),
        envelope: map_envelope(&message),
        receipt: map_receipt(&message),
    })
}

/// Decode `bytes` using the charset `label` (via `encoding_rs`), falling back to
/// UTF-8 with lossy replacement for an unknown or absent label.
///
/// `mail-parser` already decodes `text/*` parts, but this is the shared path for
/// any part that arrives as raw bytes with an explicit charset (plan §0).
#[must_use]
pub fn decode_charset(bytes: &[u8], label: Option<&str>) -> String {
    let encoding = label
        .and_then(|l| encoding_rs::Encoding::for_label(l.trim().as_bytes()))
        .unwrap_or(encoding_rs::UTF_8);
    encoding.decode(bytes).0.into_owned()
}

fn map_email(message: &Message<'_>, raw: &[u8]) -> Email {
    let mut body_values = HashMap::new();
    let text_body = collect_body(message, &message.text_body, &mut body_values);
    let html_body = collect_body(message, &message.html_body, &mut body_values);

    let sent_at = message.date().and_then(datetime_rfc3339);
    let received_at = received_at(message).or_else(|| sent_at.clone());

    Email {
        subject: message.subject().map(str::to_string),
        from: map_addresses(message.from()),
        to: map_addresses(message.to()),
        cc: map_addresses(message.cc()),
        bcc: map_addresses(message.bcc()),
        reply_to: map_addresses(message.reply_to()),
        message_id: message_ids_header(message, raw, "Message-ID"),
        in_reply_to: message_ids_header(message, raw, "In-Reply-To"),
        references: message_ids_header(message, raw, "References"),
        sent_at,
        received_at,
        preview: message
            .body_preview(PREVIEW_LEN)
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty()),
        has_attachment: has_attachment(message),
        size: raw.len() as u64,
        body_values,
        text_body,
        html_body,
        attachments: collect_attachments(message),
        ..Email::default()
    }
}

/// The RFC 8621 §4.1.2.3 `MessageIds` form of the header `name`: the ids it
/// lists, in order, without their angle brackets.
///
/// Read from the header's raw text, not from `mail-parser`'s decoded value,
/// which returns whatever text the header holds when it is not a list of ids.
/// When the header appears more than once the last instance is the one read
/// (RFC 8621 §4.1.3). `None` when the header is absent, is not UTF-8, or
/// [`message_ids`] refuses it.
fn message_ids_header(message: &Message<'_>, raw: &[u8], name: &str) -> Option<Vec<String>> {
    let header = message
        .headers()
        .iter()
        .rfind(|h| h.name.as_str().eq_ignore_ascii_case(name))?;
    let value = raw.get(header.offset_start as usize..header.offset_end as usize)?;
    message_ids(std::str::from_utf8(value).ok()?)
}

/// The ids in a header value that is nothing but `<id>`s.
///
/// Between ids: spaces, tabs, line folds, comments (nested, with `\` quoting
/// the next character) and commas, which some senders write in `References`.
/// An id is the text between `<` and `>`; it has to be non-empty and hold no
/// whitespace, no control character and no `<`, so an id folded across two
/// lines is refused, and so is the obsolete quoted form with a space in it.
/// An id is not required to contain `@`: ids without one are common in real
/// mail, and a reply has to be able to name them.
///
/// Anything else — text outside brackets and comments, an unclosed `<` or
/// comment, a value with no id in it — makes the whole value `None`. One bad
/// id does not leave the others standing: a partial `References` chain reads
/// as a different thread, not as a shorter one.
fn message_ids(value: &str) -> Option<Vec<String>> {
    let mut ids = Vec::new();
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        match c {
            ' ' | '\t' | '\r' | '\n' | ',' => {}
            '(' => {
                let mut depth = 1;
                while depth > 0 {
                    match chars.next()? {
                        '\\' => {
                            chars.next()?;
                        }
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                }
            }
            '<' => {
                let mut id = String::new();
                loop {
                    match chars.next()? {
                        '>' => break,
                        c if c == '<' || c.is_whitespace() || c.is_control() => return None,
                        c => id.push(c),
                    }
                }
                if id.is_empty() {
                    return None;
                }
                ids.push(id);
            }
            _ => return None,
        }
    }
    (!ids.is_empty()).then_some(ids)
}

/// Build [`EmailBodyPart`] metadata for the message's non-inline attachment
/// parts. `blobId` is left `None` — the engine, which owns the message's stable
/// id, fills it (`<stableId>.<partId>`) so [`crate::part_blob`] can resolve a
/// download back to these exact bytes.
fn collect_attachments(message: &Message<'_>) -> Vec<EmailBodyPart> {
    let mut parts = Vec::new();
    for &id in &message.attachments {
        let Some(part) = message.part(id) else {
            continue;
        };
        // Inline/cid resources belong to the rendered body, not the file list.
        if is_inline_resource(part) {
            continue;
        }
        let (mime_type, charset) = content_type_of(part);
        parts.push(EmailBodyPart {
            part_id: Some(id.to_string()),
            blob_id: None,
            size: part.len() as u64,
            r#type: Some(mime_type),
            charset,
            name: part.attachment_name().map(str::to_string),
            cid: part.content_id().map(strip_angle),
            disposition: disposition_of(part),
        });
    }
    parts
}

/// A single MIME part's decoded content, addressed by the `partId` [`parse`]
/// emits — the download unit behind a `<stableId>.<partId>` blob id (plan §2.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartBlob {
    /// Lowercased `type/subtype` (e.g. `application/pdf`).
    pub content_type: String,
    /// The declared attachment filename, when the part carries one.
    pub filename: Option<String>,
    /// Decoded content bytes (Content-Transfer-Encoding already undone).
    pub bytes: Vec<u8>,
}

/// Extract one MIME part's decoded bytes by its `mail-parser` part index.
///
/// Returns `None` when `part_id` names no leaf part or names a container
/// (multipart / nested message), neither of which is a downloadable blob. Runs
/// over untrusted bytes inside the render jail and never panics.
#[must_use]
pub fn part_blob(raw: &[u8], part_id: u32) -> Option<PartBlob> {
    let message = MessageParser::default().parse(raw)?;
    let part = message.part(part_id)?;
    let (content_type, _charset) = content_type_of(part);
    let bytes = match &part.body {
        PartType::Text(t) | PartType::Html(t) => t.as_bytes().to_vec(),
        PartType::Binary(b) | PartType::InlineBinary(b) => b.as_ref().to_vec(),
        PartType::Message(_) | PartType::Multipart(_) => return None,
    };
    Some(PartBlob {
        content_type,
        filename: part.attachment_name().map(str::to_string),
        bytes,
    })
}

fn map_envelope(message: &Message<'_>) -> ParsedEnvelope {
    ParsedEnvelope {
        message_id: message.message_id().map(strip_angle),
        in_reply_to: header_ids(message.in_reply_to()).into_iter().next(),
        references: header_ids(message.references()),
    }
}

fn map_receipt(message: &Message<'_>) -> ReceiptHeaders {
    let mut requests = Vec::new();
    let mut return_path = None;
    let mut from_list = false;
    for (name, value) in message.headers_raw() {
        if name.eq_ignore_ascii_case("Disposition-Notification-To") {
            requests.push(value);
        } else if name.eq_ignore_ascii_case("Return-Path") {
            return_path.get_or_insert(value);
        } else if name.eq_ignore_ascii_case("List-Id") {
            from_list = true;
        } else if name.eq_ignore_ascii_case("Precedence") {
            let value = value.trim();
            from_list |= ["bulk", "list", "junk"]
                .iter()
                .any(|p| value.eq_ignore_ascii_case(p));
        }
    }
    ReceiptHeaders {
        disposition_notification_to: match requests.as_slice() {
            [one] => single_mailbox(one),
            _ => None,
        },
        return_path: return_path.and_then(path_of),
        from_list,
    }
}

/// The one mailbox a header value names, or `None`.
///
/// Accepts `local@domain`, `<local@domain>` and either with a display name
/// (quoted or not) and comments. Quoted strings and comments are skipped, so a
/// comma inside a display name does not count as a second address; a comma,
/// semicolon or colon outside them (an address list or a group) yields `None`,
/// as does a second `<…>`, text after it, or an address that fails
/// [`validate_mailbox`](crate::validate_mailbox).
fn single_mailbox(value: &str) -> Option<String> {
    let mut bare = String::new();
    let mut angle: Option<String> = None;
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => loop {
                match chars.next()? {
                    '\\' => {
                        chars.next()?;
                    }
                    '"' => break,
                    _ => {}
                }
            },
            '(' => {
                let mut depth = 1;
                while depth > 0 {
                    match chars.next()? {
                        '\\' => {
                            chars.next()?;
                        }
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                }
            }
            '<' => {
                if angle.is_some() {
                    return None;
                }
                let mut inner = String::new();
                loop {
                    match chars.next()? {
                        '>' => break,
                        c => inner.push(c),
                    }
                }
                angle = Some(inner);
                bare.clear();
            }
            ',' | ';' | ':' | '>' | ')' => return None,
            c if angle.is_some() && !c.is_whitespace() => return None,
            c => bare.push(c),
        }
    }
    let addr = match angle {
        Some(inner) => inner,
        None => bare,
    };
    let addr = addr.trim();
    crate::check::mailbox_problem(addr)
        .is_none()
        .then(|| addr.to_string())
}

/// A `Return-Path` value as a bare path: `<a@b>` → `a@b`, `<>` → the empty
/// string. `None` for a value holding whitespace or a control character once
/// the brackets are off.
fn path_of(value: &str) -> Option<String> {
    let value = value.trim();
    let path = value
        .strip_prefix('<')
        .and_then(|v| v.strip_suffix('>'))
        .unwrap_or(value);
    (!path.chars().any(|c| c.is_control() || c.is_whitespace())).then(|| path.to_string())
}

/// Build [`EmailBodyPart`] metadata for `ids`, accumulating decoded text parts
/// into `body_values` keyed by the string form of the `mail-parser` part id.
fn collect_body(
    message: &Message<'_>,
    ids: &[u32],
    body_values: &mut HashMap<String, EmailBodyValue>,
) -> Vec<EmailBodyPart> {
    let mut parts = Vec::with_capacity(ids.len());
    for &id in ids {
        let Some(part) = message.part(id) else {
            continue;
        };
        let part_id = id.to_string();
        let (mime_type, charset) = content_type_of(part);

        if let Some(value) = decoded_text(part, charset.as_deref()) {
            body_values.insert(
                part_id.clone(),
                EmailBodyValue {
                    value,
                    is_encoding_problem: part.is_encoding_problem,
                    is_truncated: false,
                },
            );
        }

        parts.push(EmailBodyPart {
            part_id: Some(part_id),
            blob_id: None,
            size: part.len() as u64,
            r#type: Some(mime_type),
            charset,
            name: part.attachment_name().map(str::to_string),
            cid: part.content_id().map(strip_angle),
            disposition: disposition_of(part),
        });
    }
    parts
}

/// Decode a text/html body part to a UTF-8 `String`, or `None` for non-text.
fn decoded_text(part: &MessagePart<'_>, charset: Option<&str>) -> Option<String> {
    match &part.body {
        PartType::Text(t) | PartType::Html(t) => Some(t.as_ref().to_string()),
        // Defensive: a part routed into text/html bodies but left as raw bytes
        // (unusual). Decode via the declared charset, UTF-8 lossy fallback.
        PartType::Binary(b) | PartType::InlineBinary(b) => Some(decode_charset(b, charset)),
        PartType::Message(_) | PartType::Multipart(_) => None,
    }
}

/// `(lowercased "type/subtype", charset)` for a part, with sensible defaults.
fn content_type_of(part: &MessagePart<'_>) -> (String, Option<String>) {
    match part.content_type() {
        Some(ct) => {
            let mut mime = ct.ctype().to_ascii_lowercase();
            if let Some(sub) = ct.subtype() {
                mime.push('/');
                mime.push_str(&sub.to_ascii_lowercase());
            }
            (mime, ct.attribute("charset").map(str::to_string))
        }
        None => (default_type(part).to_string(), None),
    }
}

fn default_type(part: &MessagePart<'_>) -> &'static str {
    match &part.body {
        PartType::Text(_) => "text/plain",
        PartType::Html(_) => "text/html",
        PartType::Message(_) => "message/rfc822",
        PartType::Multipart(_) => "multipart/mixed",
        PartType::Binary(_) | PartType::InlineBinary(_) => "application/octet-stream",
    }
}

/// JMAP `disposition`: explicit Content-Disposition type, else `inline` when the
/// part is referenced by a `Content-ID` (a cid resource), else unset.
fn disposition_of(part: &MessagePart<'_>) -> Option<String> {
    if let Some(cd) = part.content_disposition() {
        Some(cd.ctype().to_ascii_lowercase())
    } else if part.content_id().is_some() {
        Some("inline".to_string())
    } else {
        None
    }
}

/// A message "has an attachment" (JMAP sense) when it carries at least one
/// attachment part that is not merely an inline/cid resource.
fn has_attachment(message: &Message<'_>) -> bool {
    message.attachments().any(|p| !is_inline_resource(p))
}

fn is_inline_resource(part: &MessagePart<'_>) -> bool {
    part.content_disposition().is_some_and(|d| d.is_inline()) || part.content_id().is_some()
}

fn map_addresses(addr: Option<&Address<'_>>) -> Option<Vec<EmailAddress>> {
    let list: Vec<EmailAddress> = addr?
        .iter()
        .filter_map(|a| {
            a.address().map(|email| EmailAddress {
                name: a.name().map(str::to_string),
                email: email.to_string(),
            })
        })
        .collect();
    (!list.is_empty()).then_some(list)
}

fn received_at(message: &Message<'_>) -> Option<String> {
    message
        .received()
        .and_then(|r| r.date())
        .filter(DateTime::is_valid)
        .map(|d| d.to_rfc3339())
}

fn datetime_rfc3339(d: &DateTime) -> Option<String> {
    d.is_valid().then(|| d.to_rfc3339())
}

/// Flatten a `Message-ID`-style header into its individual ids (brackets off).
fn header_ids(value: &HeaderValue<'_>) -> Vec<String> {
    match value {
        HeaderValue::Text(t) => vec![strip_angle(t)],
        HeaderValue::TextList(list) => list.iter().map(|c| strip_angle(c)).collect(),
        _ => Vec::new(),
    }
}

fn strip_angle(s: &str) -> String {
    s.trim()
        .trim_start_matches('<')
        .trim_end_matches('>')
        .to_string()
}
