//! [`ComposeRequest`] → raw RFC822 bytes (plan §0, `mail-builder`).
//!
//! Produces the bytes `mw-smtp` submits (MAIL/RCPT/DATA) and the engine
//! `APPEND`s to Sent/Drafts. `mail-builder` fills in `Date`, `MIME-Version` and
//! a `Message-ID` when one is not supplied, and picks `text/plain`,
//! `multipart/alternative`, etc. based on which bodies are present.

use mail_builder::MessageBuilder;
use mail_builder::headers::address::Address as BuilderAddress;
use mail_builder::headers::content_type::ContentType;
use mail_builder::headers::raw::Raw;
use mail_builder::mime::{BodyPart, MimePart};
use mw_jmap::EmailAddress;

use crate::MimeError;
use crate::check::{
    file_name_text, mailbox_problem, no_control, random_token, refuse, validate_content_id,
};

/// A single binary attachment to emit on the composed message.
///
/// Bytes are the already-decoded part contents (the engine resolves these from
/// an existing stored message/part via `Engine::fetch_blob`); `mail-builder`
/// applies the transfer-encoding when writing.
#[derive(Debug, Clone, Default)]
pub struct Attachment {
    /// Suggested `Content-Disposition` filename.
    pub filename: String,
    /// MIME `Content-Type` (e.g. `application/pdf`).
    pub content_type: String,
    /// Raw (decoded) attachment bytes.
    pub bytes: Vec<u8>,
}

/// A request to compose an outgoing message (draft or submission).
///
/// Addresses reuse the frozen [`mw_jmap::EmailAddress`] shape. For a reply, set
/// `in_reply_to` to the parent's `Message-ID` and `references` to the parent's
/// `References` chain plus that `Message-ID`.
#[derive(Debug, Clone, Default)]
pub struct ComposeRequest {
    /// The `From` author (required for a valid submission).
    pub from: Option<EmailAddress>,
    /// `To` recipients.
    pub to: Vec<EmailAddress>,
    /// `Cc` recipients.
    pub cc: Vec<EmailAddress>,
    /// `Bcc` recipients (present in the composed bytes; the submitter decides
    /// whether to strip them before DATA).
    pub bcc: Vec<EmailAddress>,
    /// `Reply-To` addresses.
    pub reply_to: Vec<EmailAddress>,
    /// `Subject`.
    pub subject: Option<String>,
    /// Plain-text body.
    pub text_body: Option<String>,
    /// HTML body (paired with `text_body` produces `multipart/alternative`).
    pub html_body: Option<String>,
    /// Explicit `Message-ID` (brackets optional); auto-generated when `None`.
    pub message_id: Option<String>,
    /// `In-Reply-To` for replies.
    pub in_reply_to: Option<String>,
    /// `References` chain for replies.
    pub references: Vec<String>,
    /// Extra raw headers (verbatim), e.g. `User-Agent`.
    pub headers: Vec<(String, String)>,
    /// Binary attachments (forward / attach-from-mail). Empty ⇒ body-only
    /// output is byte-unchanged; non-empty ⇒ the message becomes multipart.
    pub attachments: Vec<Attachment>,
}

/// A part shown inside the HTML body (an image the body refers to by
/// `cid:<cid>`), as opposed to a file in the attachment list.
#[derive(Debug, Clone, Default)]
pub struct InlinePart {
    /// The `Content-ID`, without angle brackets. The HTML body refers to the
    /// part as `cid:<this value>`. Must pass
    /// [`validate_content_id`](crate::validate_content_id).
    pub cid: String,
    /// MIME `Content-Type` as `type/subtype` (e.g. `image/png`), no parameters.
    pub content_type: String,
    /// Optional file name, written as the `filename` parameter of
    /// `Content-Disposition: inline`. Control characters are dropped.
    pub filename: Option<String>,
    /// Raw (decoded) bytes.
    pub bytes: Vec<u8>,
}

/// What [`build_with`] can put in a message beyond a [`ComposeRequest`].
#[derive(Debug, Clone, Default)]
pub struct ComposeExtras {
    /// Parts the HTML body refers to by `cid:`. Non-empty ⇒ the HTML body and
    /// these parts are wrapped in `multipart/related`.
    pub inline_parts: Vec<InlinePart>,
    /// The mailbox a read receipt is requested for (RFC 8098
    /// `Disposition-Notification-To`). `None` ⇒ no request.
    pub receipt_to: Option<String>,
}

/// Serialize a [`ComposeRequest`] into raw RFC822 bytes.
///
/// The values are written as given: this function does not check them, and
/// `mail-builder` writes addresses, ids, content types, raw headers and a
/// subject containing CRLF byte for byte. [`build_with`] is the checked form.
pub fn build(req: &ComposeRequest) -> Result<Vec<u8>, MimeError> {
    let mut b = bodies(headers(req), req);

    // Attachments turn the message multipart/mixed; an empty vec leaves the
    // body-only output byte-unchanged (no attachment API is touched).
    for att in &req.attachments {
        b = b.attachment(
            att.content_type.as_str(),
            att.filename.as_str(),
            att.bytes.clone(),
        );
    }

    b.write_to_vec()
        .map_err(|e| MimeError::Build(e.to_string()))
}

/// Serialize a [`ComposeRequest`] plus [`ComposeExtras`] into raw RFC822 bytes,
/// refusing any value that could add a header line or a MIME part.
///
/// Checked here, whatever the caller did before:
/// - every address (`from`, `to`, `cc`, `bcc`, `reply_to`, `receipt_to`) is one
///   mailbox by [`validate_mailbox`](crate::validate_mailbox)'s rules;
/// - `subject`, `message_id`, `in_reply_to`, `references`, attachment content
///   types and the names and values of `headers` hold no control character
///   (which includes CR, LF, NUL and TAB), and a header name is printable
///   ASCII without `:`;
/// - `headers` does not carry `Disposition-Notification-To` (the typed
///   `receipt_to` is the only way to write it, so it appears at most once);
/// - each inline part's `cid` passes
///   [`validate_content_id`](crate::validate_content_id), no `cid` repeats,
///   and its content type is a bare `type/subtype` other than `multipart/*`
///   and `message/*`.
///
/// File names (attachments and inline parts) are not refused; their control
/// characters are dropped. Display names are encoded by `mail-builder`.
///
/// Layout with inline parts: `multipart/related` holding the HTML body and the
/// inline parts (each `Content-ID: <cid>`, `Content-Disposition: inline`). A
/// plain-text body puts that inside `multipart/alternative`; ordinary
/// attachments put the result inside `multipart/mixed`. Inline parts without
/// an HTML body are refused. Whether the HTML refers to each `cid` is not
/// checked here; see [`html_cid_references`].
///
/// With empty extras and acceptable values the output has the same structure
/// as [`build`]'s.
pub fn build_with(req: &ComposeRequest, extras: &ComposeExtras) -> Result<Vec<u8>, MimeError> {
    check_request(req)?;
    check_extras(extras)?;

    let mut b = headers(req);
    if let Some(addr) = &extras.receipt_to {
        b = b.header("Disposition-Notification-To", Raw::new(format!("<{addr}>")));
    }

    if extras.inline_parts.is_empty() {
        b = bodies(b, req);
        for att in &req.attachments {
            b = b.attachment(
                att.content_type.as_str(),
                file_name_text(&att.filename),
                att.bytes.clone(),
            );
        }
    } else {
        let html = req.html_body.as_deref().ok_or_else(|| {
            MimeError::Build("an inline part needs an HTML body that refers to it".into())
        })?;
        let mut related = vec![MimePart::new("text/html", BodyPart::Text(html.into()))];
        related.extend(extras.inline_parts.iter().map(inline_part));
        let related = MimePart::new(multipart("related").attribute("type", "text/html"), related);

        let content = match req.text_body.as_deref() {
            Some(text) => MimePart::new(
                multipart("alternative"),
                vec![
                    MimePart::new("text/plain", BodyPart::Text(text.into())),
                    related,
                ],
            ),
            None => related,
        };

        b = b.body(if req.attachments.is_empty() {
            content
        } else {
            let mut parts = vec![content];
            parts.extend(req.attachments.iter().map(|att| {
                MimePart::new(att.content_type.as_str(), att.bytes.clone())
                    .attachment(file_name_text(&att.filename))
            }));
            MimePart::new(multipart("mixed"), parts)
        });
    }

    b.write_to_vec()
        .map_err(|e| MimeError::Build(e.to_string()))
}

/// The `cid:` references inside the tags of `html`, in order, without
/// duplicates and with percent-escapes decoded (RFC 2392).
///
/// This is a lexical scan, not an HTML parser: it reports a `cid:` URL found
/// between a `<` and its closing `>` (an attribute value such as `src` or an
/// inline `style`), and nothing from text content or from the body of a
/// `<style>` element. The values are returned as written and are not checked.
#[must_use]
pub fn html_cid_references(html: &str) -> Vec<String> {
    let bytes = html.as_bytes();
    let mut out: Vec<String> = Vec::new();
    let mut in_tag = false;
    let mut quote: Option<u8> = None;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if !in_tag {
            in_tag = c == b'<';
            i += 1;
            continue;
        }
        match quote {
            Some(q) if c == q => quote = None,
            None if c == b'"' || c == b'\'' => quote = Some(c),
            None if c == b'>' => in_tag = false,
            _ => {}
        }
        // `in_tag` was set on an earlier byte, so `i >= 1` here.
        let starts_reference = bytes[i..].len() >= 4
            && bytes[i..i + 4].eq_ignore_ascii_case(b"cid:")
            && !bytes[i - 1].is_ascii_alphanumeric();
        if !starts_reference {
            i += 1;
            continue;
        }
        let start = i + 4;
        let end = bytes[start..]
            .iter()
            .position(|b| b.is_ascii_whitespace() || b"\"'<>(),;".contains(b))
            .map_or(bytes.len(), |n| start + n);
        let cid = percent_decode(&bytes[start..end]);
        if !cid.is_empty() && !out.contains(&cid) {
            out.push(cid);
        }
        i = end.max(i + 1);
    }
    out
}

/// Decode `%XX` escapes; anything that is not a complete escape is kept as it
/// stands, and bytes that do not form UTF-8 are replaced.
fn percent_decode(raw: &[u8]) -> String {
    let hex = |b: u8| char::from(b).to_digit(16).map(|d| d as u8);
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        let escape = (raw[i] == b'%')
            .then(|| Some(hex(*raw.get(i + 1)?)? << 4 | hex(*raw.get(i + 2)?)?))
            .flatten();
        match escape {
            Some(byte) => {
                out.push(byte);
                i += 3;
            }
            None => {
                out.push(raw[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The header section shared by [`build`] and [`build_with`].
fn headers(req: &ComposeRequest) -> MessageBuilder<'_> {
    let mut b = MessageBuilder::new();

    if let Some(from) = &req.from {
        b = b.from(builder_addr(from));
    }
    if !req.to.is_empty() {
        b = b.to(builder_list(&req.to));
    }
    if !req.cc.is_empty() {
        b = b.cc(builder_list(&req.cc));
    }
    if !req.bcc.is_empty() {
        b = b.bcc(builder_list(&req.bcc));
    }
    if !req.reply_to.is_empty() {
        b = b.reply_to(builder_list(&req.reply_to));
    }
    if let Some(subject) = &req.subject {
        b = b.subject(subject.as_str());
    }
    if let Some(mid) = &req.message_id {
        b = b.message_id(bare_id(mid));
    }
    if let Some(irt) = &req.in_reply_to {
        b = b.in_reply_to(bare_id(irt));
    }
    if !req.references.is_empty() {
        let refs: Vec<String> = req.references.iter().map(|r| bare_id(r)).collect();
        b = b.references(refs);
    }
    for (name, value) in &req.headers {
        b = b.header(name.as_str(), Raw::new(value.as_str()));
    }
    b
}

/// The text and HTML bodies, left to `mail-builder` to arrange.
fn bodies<'x>(b: MessageBuilder<'x>, req: &'x ComposeRequest) -> MessageBuilder<'x> {
    match (&req.text_body, &req.html_body) {
        (Some(text), Some(html)) => b.text_body(text.as_str()).html_body(html.as_str()),
        (Some(text), None) => b.text_body(text.as_str()),
        (None, Some(html)) => b.html_body(html.as_str()),
        // A submission with no body is still valid; emit an empty text part.
        (None, None) => b.text_body(""),
    }
}

/// A `multipart/<subtype>` content type with a boundary from
/// [`random_token`], so that body text cannot contain the delimiter.
fn multipart(subtype: &str) -> ContentType<'static> {
    ContentType::new(format!("multipart/{subtype}"))
        .attribute("boundary", format!("mw_{}", random_token()))
}

fn inline_part(part: &InlinePart) -> MimePart<'_> {
    let disposition = match part.filename.as_deref().map(file_name_text) {
        Some(name) if !name.is_empty() => ContentType::new("inline").attribute("filename", name),
        _ => ContentType::new("inline"),
    };
    MimePart::new(part.content_type.as_str(), part.bytes.as_slice())
        .header("Content-Disposition", disposition)
        .cid(part.cid.as_str())
}

/// The checks [`build_with`] makes on the request itself.
fn check_request(req: &ComposeRequest) -> Result<(), MimeError> {
    let lists = [
        ("to", &req.to),
        ("cc", &req.cc),
        ("bcc", &req.bcc),
        ("reply-to", &req.reply_to),
    ];
    let listed = lists
        .iter()
        .flat_map(|(what, list)| list.iter().map(move |a| (*what, a)));
    for (what, addr) in req.from.iter().map(|a| ("from", a)).chain(listed) {
        if let Some(why) = mailbox_problem(&addr.email) {
            return Err(refuse(&format!("{what} address"), &addr.email, why));
        }
    }
    if let Some(subject) = &req.subject {
        no_control("subject", subject)?;
    }
    if let Some(id) = &req.message_id {
        no_control("message id", id)?;
    }
    if let Some(id) = &req.in_reply_to {
        no_control("in-reply-to", id)?;
    }
    for id in &req.references {
        no_control("references", id)?;
    }
    for (name, value) in &req.headers {
        if name.is_empty() || !name.bytes().all(|b| b.is_ascii_graphic() && b != b':') {
            return Err(refuse("header name", name, "not a header field name"));
        }
        if name.eq_ignore_ascii_case("Disposition-Notification-To") {
            return Err(refuse(
                "header name",
                name,
                "a receipt request is made with `receipt_to`",
            ));
        }
        no_control("header value", value)?;
    }
    for att in &req.attachments {
        no_control("attachment content type", &att.content_type)?;
    }
    Ok(())
}

/// The checks [`build_with`] makes on the extras.
fn check_extras(extras: &ComposeExtras) -> Result<(), MimeError> {
    if let Some(addr) = &extras.receipt_to
        && let Some(why) = mailbox_problem(addr)
    {
        return Err(refuse("receipt address", addr, why));
    }
    for (n, part) in extras.inline_parts.iter().enumerate() {
        validate_content_id(&part.cid)?;
        if extras.inline_parts[..n].iter().any(|p| p.cid == part.cid) {
            return Err(refuse("content id", &part.cid, "used by two inline parts"));
        }
        if !is_leaf_media_type(&part.content_type) {
            return Err(refuse(
                "inline content type",
                &part.content_type,
                "not a type/subtype of a single part",
            ));
        }
    }
    Ok(())
}

/// `type/subtype` in RFC 6838 restricted names, excluding the two composite
/// top-level types.
fn is_leaf_media_type(value: &str) -> bool {
    let name = |s: &str| {
        !s.is_empty()
            && s.len() <= 127
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"!#$&-^_.+".contains(&b))
    };
    value.split_once('/').is_some_and(|(top, sub)| {
        name(top)
            && name(sub)
            && !top.eq_ignore_ascii_case("multipart")
            && !top.eq_ignore_ascii_case("message")
    })
}

fn builder_addr(a: &EmailAddress) -> BuilderAddress<'_> {
    BuilderAddress::new_address(a.name.as_deref(), a.email.as_str())
}

fn builder_list(list: &[EmailAddress]) -> BuilderAddress<'_> {
    BuilderAddress::new_list(list.iter().map(builder_addr).collect())
}

/// Strip surrounding angle brackets — `mail-builder` re-adds them when writing
/// `Message-ID`/`In-Reply-To`/`References`, so ids must be passed bare.
fn bare_id(id: &str) -> String {
    id.trim()
        .trim_start_matches('<')
        .trim_end_matches('>')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_req() -> ComposeRequest {
        ComposeRequest {
            from: Some(EmailAddress {
                name: None,
                email: "sender@example.test".into(),
            }),
            to: vec![EmailAddress {
                name: None,
                email: "rcpt@example.test".into(),
            }],
            subject: Some("Fwd: hi".into()),
            text_body: Some("see attached".into()),
            message_id: Some("m1@example.test".into()),
            ..Default::default()
        }
    }

    #[test]
    fn body_only_stays_single_part_when_attachments_empty() {
        let raw = build(&base_req()).expect("build");
        let text = String::from_utf8(raw).expect("utf8");
        // An empty attachment vec must not touch the attachment API: the message
        // is a plain single text part, not multipart/mixed.
        assert!(text.contains("Content-Type: text/plain"), "{text}");
        assert!(!text.contains("multipart/mixed"), "{text}");
        assert!(text.contains("see attached"), "{text}");
    }

    #[test]
    fn attachment_yields_multipart_carrying_the_part() {
        let mut req = base_req();
        req.attachments.push(Attachment {
            filename: "invoice.pdf".into(),
            content_type: "application/pdf".into(),
            bytes: b"%PDF-1.4\n".to_vec(),
        });
        let raw = build(&req).expect("build");

        // Re-parse: the built message is multipart and carries the attachment
        // part with its filename, content-type, and exact bytes.
        let parsed = crate::parse(&raw).expect("parse built message");
        let att = parsed
            .email
            .attachments
            .iter()
            .find(|p| p.name.as_deref() == Some("invoice.pdf"))
            .expect("attachment part present");
        assert_eq!(att.r#type.as_deref(), Some("application/pdf"));
        let part_id: u32 = att
            .part_id
            .as_deref()
            .and_then(|s| s.parse().ok())
            .expect("numeric part id");
        let blob = crate::part_blob(&raw, part_id).expect("decode attachment part");
        assert_eq!(blob.content_type, "application/pdf");
        assert_eq!(blob.filename.as_deref(), Some("invoice.pdf"));
        assert_eq!(blob.bytes, b"%PDF-1.4\n");
    }
}
