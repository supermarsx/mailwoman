//! [`ComposeRequest`] → raw RFC822 bytes (plan §0, `mail-builder`).
//!
//! Produces the bytes `mw-smtp` submits (MAIL/RCPT/DATA) and the engine
//! `APPEND`s to Sent/Drafts. `mail-builder` fills in `Date`, `MIME-Version` and
//! a `Message-ID` when one is not supplied, and picks `text/plain`,
//! `multipart/alternative`, etc. based on which bodies are present.

use mail_builder::MessageBuilder;
use mail_builder::headers::address::Address as BuilderAddress;
use mail_builder::headers::raw::Raw;
use mw_jmap::EmailAddress;

use crate::MimeError;

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
}

/// Serialize a [`ComposeRequest`] into raw RFC822 bytes.
pub fn build(req: &ComposeRequest) -> Result<Vec<u8>, MimeError> {
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

    b = match (&req.text_body, &req.html_body) {
        (Some(text), Some(html)) => b.text_body(text.as_str()).html_body(html.as_str()),
        (Some(text), None) => b.text_body(text.as_str()),
        (None, Some(html)) => b.html_body(html.as_str()),
        // A submission with no body is still valid; emit an empty text part.
        (None, None) => b.text_body(""),
    };

    b.write_to_vec()
        .map_err(|e| MimeError::Build(e.to_string()))
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
