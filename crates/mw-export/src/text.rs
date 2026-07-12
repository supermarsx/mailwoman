//! Plain-text export and the shared header block (plan §3 e3).
//!
//! A TXT export is a small header block (From/To/Cc/Subject/Date) followed by a
//! blank line and the decoded plain-text body. When a message carries only HTML
//! we fall back to the Markdown rendering, which is readable as plain text.

use mail_parser::{Address, Message, MessageParser};

use crate::html2md::html_to_markdown;
use crate::{ExportError, Result};

/// Export a single message to plain text.
pub fn to_txt(raw: &[u8]) -> Result<Vec<u8>> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or_else(|| ExportError::Parse("not a recognisable RFC5322 message".into()))?;

    let mut out = headers_block(&message);
    out.push('\n');
    out.push_str(&plain_body(&message));
    while out.ends_with(['\n', ' ', '\t', '\r']) {
        out.pop();
    }
    out.push('\n');
    Ok(out.into_bytes())
}

/// The decoded plain-text body: the first `text/plain` part, else the first
/// `text/html` part rendered to Markdown, else empty.
pub fn plain_body(message: &Message<'_>) -> String {
    if let Some(text) = message.body_text(0) {
        return text.into_owned();
    }
    if let Some(html) = message.body_html(0) {
        return html_to_markdown(&html);
    }
    String::new()
}

/// A `Key: value` header block. Missing headers are omitted.
pub fn headers_block(message: &Message<'_>) -> String {
    let mut lines = Vec::new();
    push_addr(&mut lines, "From", message.from());
    push_addr(&mut lines, "To", message.to());
    push_addr(&mut lines, "Cc", message.cc());
    if let Some(subject) = message.subject() {
        lines.push(format!("Subject: {subject}"));
    }
    if let Some(date) = message.date() {
        lines.push(format!("Date: {}", date.to_rfc822()));
    }
    let mut out = lines.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    out
}

fn push_addr(lines: &mut Vec<String>, label: &str, addr: Option<&Address<'_>>) {
    if let Some(formatted) = format_addresses(addr) {
        lines.push(format!("{label}: {formatted}"));
    }
}

/// Format an address list as `Name <email>, other@example.com`.
pub fn format_addresses(addr: Option<&Address<'_>>) -> Option<String> {
    let parts: Vec<String> = addr?
        .iter()
        .filter_map(|a| {
            let email = a.address()?;
            Some(match a.name() {
                Some(name) if !name.trim().is_empty() => format!("{} <{email}>", name.trim()),
                _ => email.to_string(),
            })
        })
        .collect();
    (!parts.is_empty()).then(|| parts.join(", "))
}
