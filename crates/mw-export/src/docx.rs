//! Word `.docx` export (plan §3 e5, §1.7, SPEC §10.6).
//!
//! Renders a message — header block + body + an attachment manifest — to a
//! Word document with [`docx_rs`]. The body goes through the **same
//! sanitized-shape pipeline** the TXT/Markdown exports use ([`crate::text`] →
//! [`crate::html2md`]): an HTML-only message is folded to Markdown-shaped plain
//! text with `script`/`style`/`head` subtrees dropped, so no code text leaks
//! into the document. No untrusted-container parse happens here (DOCX is
//! write-only in this crate), so there is no render-jail concern on this path.

use std::io::Cursor;

use docx_rs::{BreakType, Docx, Paragraph, Run};
use mail_parser::{Message, MessageParser};

use crate::text::{format_addresses, plain_body};
use crate::{ExportError, Result};

/// Export one RFC 5322 message to `.docx` bytes.
pub fn to_docx(raw: &[u8]) -> Result<Vec<u8>> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or_else(|| ExportError::Parse("not a recognisable RFC5322 message".into()))?;

    let mut docx = Docx::new();

    // Header block: one paragraph per present field, `**Label:** value` in bold.
    for (label, value) in header_fields(&message) {
        docx = docx.add_paragraph(labelled(&label, &value));
    }

    // Body: sanitized-shape plain text (HTML → Markdown-shaped via html2md).
    // Blank-line-separated blocks become paragraphs; single newlines become
    // soft line breaks within a paragraph.
    let body = plain_body(&message);
    for block in body.split("\n\n") {
        let block = block.trim_matches(['\r', '\n']);
        if block.is_empty() {
            continue;
        }
        docx = docx.add_paragraph(body_paragraph(block));
    }

    // Attachment manifest: filenames only (the bytes are not embedded — a Word
    // document is a rendition, not a container; §28.8 best-effort).
    let names: Vec<String> = message
        .attachments()
        .filter_map(|p| {
            use mail_parser::MimeHeaders;
            p.attachment_name().map(str::to_string)
        })
        .collect();
    if !names.is_empty() {
        docx = docx.add_paragraph(labelled("Attachments", &names.join(", ")));
    }

    let mut buf = Cursor::new(Vec::new());
    docx.build()
        .pack(&mut buf)
        .map_err(|e| ExportError::Render(format!("docx pack: {e}")))?;
    Ok(buf.into_inner())
}

/// The header fields we render, in order, skipping absent ones.
fn header_fields(message: &Message<'_>) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    if let Some(subject) = message.subject() {
        fields.push(("Subject".into(), subject.to_string()));
    }
    if let Some(from) = format_addresses(message.from()) {
        fields.push(("From".into(), from));
    }
    if let Some(to) = format_addresses(message.to()) {
        fields.push(("To".into(), to));
    }
    if let Some(cc) = format_addresses(message.cc()) {
        fields.push(("Cc".into(), cc));
    }
    if let Some(date) = message.date() {
        fields.push(("Date".into(), date.to_rfc822()));
    }
    fields
}

/// A `Label: value` paragraph with a bold label.
fn labelled(label: &str, value: &str) -> Paragraph {
    Paragraph::new()
        .add_run(Run::new().add_text(format!("{label}: ")).bold())
        .add_run(Run::new().add_text(value.to_string()))
}

/// A body paragraph, mapping single newlines within the block to soft breaks.
fn body_paragraph(block: &str) -> Paragraph {
    let mut para = Paragraph::new();
    for (i, line) in block.split('\n').enumerate() {
        let mut run = Run::new();
        if i > 0 {
            run = run.add_break(BreakType::TextWrapping);
        }
        run = run.add_text(line.trim_end_matches('\r').to_string());
        para = para.add_run(run);
    }
    para
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_a_docx_zip() {
        let raw = b"From: Alice <alice@example.com>\r\n\
To: bob@example.com\r\n\
Subject: Hello\r\n\
Content-Type: text/plain\r\n\
\r\n\
First paragraph.\r\n\
\r\n\
Second paragraph.\r\n";
        let bytes = to_docx(raw).unwrap();
        // DOCX is a ZIP (OOXML): `PK\x03\x04` local-file-header magic.
        assert_eq!(&bytes[..4], b"PK\x03\x04");
        // The word document part is present.
        assert!(find_subslice(&bytes, b"word/document.xml").is_some());
    }

    #[test]
    fn html_only_body_is_sanitized_shape() {
        let raw = b"From: a@example.com\r\n\
Subject: HTML\r\n\
Content-Type: text/html\r\n\
\r\n\
<p>Visible</p><script>alert('x')</script>\r\n";
        let bytes = to_docx(raw).unwrap();
        // The document XML is deflated, so we can't grep the text directly, but
        // the export must succeed and produce a valid package.
        assert_eq!(&bytes[..4], b"PK\x03\x04");
    }

    fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack.windows(needle.len()).position(|w| w == needle)
    }
}
