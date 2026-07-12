//! Markdown export (plan §3 e3, SPEC §10.5).
//!
//! A bold header block, a `---` rule, then the body: the first `text/html` part
//! converted to Markdown, or the first `text/plain` part verbatim.

use mail_parser::{Message, MessageParser};

use crate::html2md::html_to_markdown;
use crate::text::format_addresses;
use crate::{ExportError, Result};

/// Export a single message to Markdown.
pub fn to_markdown(raw: &[u8]) -> Result<Vec<u8>> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or_else(|| ExportError::Parse("not a recognisable RFC5322 message".into()))?;

    let mut out = header_block(&message);
    let body = body_markdown(&message);
    if !body.is_empty() {
        out.push_str("\n\n");
        out.push_str(&body);
    }
    while out.ends_with(['\n', ' ', '\t', '\r']) {
        out.pop();
    }
    out.push('\n');
    Ok(out.into_bytes())
}

/// The message body as Markdown: HTML converted, else plain text verbatim.
pub fn body_markdown(message: &Message<'_>) -> String {
    if let Some(html) = message.body_html(0) {
        return html_to_markdown(&html);
    }
    message
        .body_text(0)
        .map(|t| t.into_owned())
        .unwrap_or_default()
}

fn header_block(message: &Message<'_>) -> String {
    let mut lines = Vec::new();
    if let Some(subject) = message.subject() {
        lines.push(format!("**Subject:** {subject}"));
    }
    if let Some(from) = format_addresses(message.from()) {
        lines.push(format!("**From:** {from}"));
    }
    if let Some(to) = format_addresses(message.to()) {
        lines.push(format!("**To:** {to}"));
    }
    if let Some(cc) = format_addresses(message.cc()) {
        lines.push(format!("**Cc:** {cc}"));
    }
    if let Some(date) = message.date() {
        lines.push(format!("**Date:** {}", date.to_rfc822()));
    }
    let mut out = lines.join("  \n"); // hard breaks keep the header on one visual block
    if !out.is_empty() {
        out.push_str("\n\n---");
    }
    out
}
