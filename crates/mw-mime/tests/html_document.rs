//! Complete outgoing HTML documents must survive MIME serialization intact.

use mail_parser::{MessageParser, MimeHeaders};
use mw_mime::{
    Attachment, ComposeExtras, ComposeRequest, EmailAddress, InlinePart, build_with, parse,
};

const HTML: &str = concat!(
    "<!DOCTYPE html><html lang=\"pt-PT\"><head><meta charset=\"utf-8\">",
    "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">",
    "<style>body{font-family:Arial,sans-serif;font-size:12pt;color:#202124;}",
    "table.invoice{border-collapse:collapse;}@media(max-width:600px){.invoice{width:100%;}}</style>",
    "</head><body dir=\"ltr\" style=\"margin:0;font-family:Arial,sans-serif;font-size:12pt\">",
    "<p style=\"line-height:1.5\">Olá <strong>João</strong> &amp; team.</p>",
    "<table class=\"invoice\" width=\"480\"><tbody><tr><td style=\"padding:8px;border:1px solid #ddd\">",
    "<img src=\"cid:logo@example.org\" width=\"24\" height=\"24\" alt=\"Logo\">",
    "</td></tr></tbody></table></body></html>",
);

fn request(text_body: Option<&str>) -> ComposeRequest {
    ComposeRequest {
        from: Some(EmailAddress {
            name: Some("Alice".into()),
            email: "alice@example.org".into(),
        }),
        to: vec![EmailAddress {
            name: None,
            email: "bob@example.net".into(),
        }],
        subject: Some("Document formatting".into()),
        html_body: Some(HTML.into()),
        text_body: text_body.map(str::to_owned),
        ..ComposeRequest::default()
    }
}

fn assert_document(raw: &[u8]) {
    let message = MessageParser::default().parse(raw).expect("valid MIME");
    let html_parts: Vec<_> = message
        .parts
        .iter()
        .filter(|part| {
            part.content_type()
                .is_some_and(|ct| ct.ctype() == "text" && ct.subtype() == Some("html"))
        })
        .collect();
    assert_eq!(html_parts.len(), 1);
    // Compare decoded contents, so transfer encoding cannot hide document,
    // stylesheet, body attributes, typography, table, or CID changes.
    assert_eq!(html_parts[0].text_contents(), Some(HTML));

    let mapped = parse(raw).expect("JMAP body mapping");
    assert_eq!(mapped.email.html_body.len(), 1);
    let part_id = mapped.email.html_body[0].part_id.as_ref().expect("part id");
    assert_eq!(mapped.email.body_values[part_id].value, HTML);
}

#[test]
fn complete_html_document_survives_html_only_and_text_alternative() {
    for text in [None, Some("Olá João & team.")] {
        let raw = build_with(&request(text), &ComposeExtras::default()).expect("build document");
        assert_document(&raw);
        let message = MessageParser::default().parse(&raw).expect("valid MIME");
        let text_parts: Vec<_> = message
            .parts
            .iter()
            .filter(|part| {
                part.content_type()
                    .is_some_and(|ct| ct.ctype() == "text" && ct.subtype() == Some("plain"))
            })
            .collect();
        assert_eq!(text_parts.len(), usize::from(text.is_some()));
        if let Some(text) = text {
            assert_eq!(text_parts[0].text_contents(), Some(text));
        }
    }
}

#[test]
fn complete_html_document_survives_related_alternative_and_attachment() {
    let image = b"inline image bytes";
    let attachment = b"attachment bytes";
    let mut req = request(Some("Olá João & team."));
    req.attachments.push(Attachment {
        filename: "invoice.pdf".into(),
        content_type: "application/pdf".into(),
        bytes: attachment.to_vec(),
    });
    let extras = ComposeExtras {
        inline_parts: vec![InlinePart {
            cid: "logo@example.org".into(),
            content_type: "image/png".into(),
            filename: Some("logo.png".into()),
            bytes: image.to_vec(),
        }],
        ..ComposeExtras::default()
    };
    let raw = build_with(&req, &extras).expect("build related document");
    assert_document(&raw);

    let message = MessageParser::default().parse(&raw).expect("valid MIME");
    let types: Vec<_> = message
        .parts
        .iter()
        .map(|part| {
            let ct = part.content_type().expect("MIME type");
            format!("{}/{}", ct.ctype(), ct.subtype().unwrap_or(""))
        })
        .collect();
    assert_eq!(
        types,
        [
            "multipart/mixed",
            "multipart/alternative",
            "text/plain",
            "multipart/related",
            "text/html",
            "image/png",
            "application/pdf",
        ]
    );
    let logo = message
        .parts
        .iter()
        .find(|part| part.content_id() == Some("logo@example.org"))
        .expect("matching CID resource");
    assert_eq!(logo.contents(), image);
    assert_eq!(
        message.parts.last().expect("attachment").contents(),
        attachment
    );
}
