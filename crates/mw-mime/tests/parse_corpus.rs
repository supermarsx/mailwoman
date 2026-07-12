//! Field-parity tests over the MIME torture corpus in `fixtures/mime/`.
//!
//! Each test asserts the `mw_jmap::Email` (and `ParsedEnvelope`) fields that the
//! `Email/get` contract (plan §2.2) must return for a representative shape.

use mw_mime::{decode_charset, parse};

/// Load a fixture from `fixtures/mime/` (relative to this test file).
macro_rules! fixture {
    ($name:literal) => {
        include_bytes!(concat!("../../../fixtures/mime/", $name)).as_slice()
    };
}

fn one_addr(list: &Option<Vec<mw_mime::EmailAddress>>) -> &mw_mime::EmailAddress {
    &list.as_ref().expect("address list present")[0]
}

/// Concatenated decoded text of every body value (order-independent probe).
fn all_body_text(email: &mw_mime::Email) -> String {
    let mut v: Vec<&str> = email
        .body_values
        .values()
        .map(|b| b.value.as_str())
        .collect();
    v.sort_unstable();
    v.join("\n")
}

#[test]
fn simple_single_part() {
    let p = parse(fixture!("simple.eml")).unwrap();
    let e = &p.email;
    assert_eq!(e.subject.as_deref(), Some("Simple greeting"));
    let from = one_addr(&e.from);
    assert_eq!(from.name.as_deref(), Some("Alice Example"));
    assert_eq!(from.email, "alice@example.org");
    assert_eq!(one_addr(&e.to).email, "bob@example.net");
    assert_eq!(e.sent_at.as_deref(), Some("2026-07-12T09:30:00Z"));
    // With no INTERNALDATE / Received header, receivedAt falls back to sentAt.
    assert_eq!(e.received_at, e.sent_at);
    assert!(!e.has_attachment);
    assert_eq!(e.size, fixture!("simple.eml").len() as u64);
    assert_eq!(e.text_body.len(), 1);
    assert_eq!(e.text_body[0].r#type.as_deref(), Some("text/plain"));
    assert!(all_body_text(e).contains("This is a plain text message."));
    assert_eq!(
        p.envelope.message_id.as_deref(),
        Some("simple-1@example.org")
    );
    assert!(p.envelope.in_reply_to.is_none());
    assert!(p.envelope.references.is_empty());
}

#[test]
fn multipart_alternative_with_rfc2047_subject_and_threading() {
    let p = parse(fixture!("alternative.eml")).unwrap();
    let e = &p.email;
    // RFC2047 encoded-word subject is decoded.
    assert_eq!(e.subject.as_deref(), Some("Café meeting"));
    // A comma inside a quoted display name must not split the address.
    assert_eq!(one_addr(&e.from).name.as_deref(), Some("Carol, O'Brien"));
    assert_eq!(e.to.as_ref().unwrap().len(), 2);
    assert_eq!(e.cc.as_ref().unwrap()[0].email, "eve@example.org");
    // Distinct text and html body parts, each with a decoded body value.
    assert_eq!(e.text_body.len(), 1);
    assert_eq!(e.text_body[0].r#type.as_deref(), Some("text/plain"));
    assert_eq!(e.html_body.len(), 1);
    assert_eq!(e.html_body[0].r#type.as_deref(), Some("text/html"));
    assert_ne!(e.text_body[0].part_id, e.html_body[0].part_id);
    // The html part is quoted-printable with a UTF-8 é that must decode.
    let html_id = e.html_body[0].part_id.clone().unwrap();
    assert!(e.body_values[&html_id].value.contains("HTML é body"));
    // Threading headers surface for the engine's JWZ pass.
    assert_eq!(
        p.envelope.in_reply_to.as_deref(),
        Some("simple-1@example.org")
    );
    assert_eq!(
        p.envelope.references,
        vec![
            "root-0@example.org".to_string(),
            "simple-1@example.org".to_string()
        ]
    );
}

#[test]
fn nested_multipart_mixed_flags_attachment() {
    let p = parse(fixture!("nested.eml")).unwrap();
    let e = &p.email;
    assert!(
        e.has_attachment,
        "a text/csv attachment must set hasAttachment"
    );
    assert!(!e.text_body.is_empty() && !e.html_body.is_empty());
    assert_eq!(e.preview.as_deref(), Some("See the attached report."));
    assert!(all_body_text(e).contains("<b>report</b>"));
}

#[test]
fn base64_attachment_sets_has_attachment() {
    let p = parse(fixture!("attachment.eml")).unwrap();
    assert!(p.email.has_attachment);
    assert!(all_body_text(&p.email).contains("Invoice attached."));
}

#[test]
fn inline_cid_is_not_an_attachment() {
    let p = parse(fixture!("inline_cid.eml")).unwrap();
    let e = &p.email;
    // An inline cid image alone must NOT set hasAttachment (JMAP semantics).
    assert!(!e.has_attachment);
    let body = all_body_text(e);
    assert!(
        body.contains("cid:logo@example.org"),
        "cid ref survives QP decode"
    );
}

#[test]
fn quoted_printable_soft_breaks_and_escapes() {
    let p = parse(fixture!("qp.eml")).unwrap();
    let body = all_body_text(&p.email);
    // Soft line break (`=\r\n`) is removed, joining the two physical lines.
    assert!(body.contains("via a soft line break, and here"));
    // `=3D` -> `=`, `=C3=A9` -> `é`.
    assert!(body.contains("an equals sign: = and an accented e: é."));
}

#[test]
fn iso_8859_1_charset_decodes() {
    let p = parse(fixture!("iso8859-1.eml")).unwrap();
    let e = &p.email;
    assert_eq!(e.subject.as_deref(), Some("Señor"));
    assert_eq!(e.text_body[0].charset.as_deref(), Some("ISO-8859-1"));
    assert!(all_body_text(e).contains("Café con leche para el señor."));
}

#[test]
fn shift_jis_charset_decodes() {
    let p = parse(fixture!("shift_jis.eml")).unwrap();
    let e = &p.email;
    assert_eq!(e.subject.as_deref(), Some("こんにちは"));
    assert_eq!(e.text_body[0].charset.as_deref(), Some("Shift_JIS"));
    assert!(all_body_text(e).contains("こんにちは"));
}

#[test]
fn malformed_headers_do_not_panic() {
    // Garbage header block, NUL/high bytes, no Content-Type: must still map.
    let p = parse(fixture!("malformed.eml")).unwrap();
    assert!(p.email.subject.is_none());
    assert!(p.email.from.is_none());
    assert!(all_body_text(&p.email).contains("body with a bare"));
}

#[test]
fn headerless_body_only() {
    let p = parse(fixture!("no_headers.eml")).unwrap();
    assert!(p.email.subject.is_none());
    assert!(p.email.sent_at.is_none());
    assert!(all_body_text(&p.email).contains("Just a body"));
}

#[test]
fn address_group_syntax_is_flattened() {
    let raw = b"From: sender@example.org\r\n\
                To: Team:alice@example.org,bob@example.net;\r\n\
                Subject: group\r\n\r\nbody\r\n";
    let p = parse(raw).unwrap();
    let to = p.email.to.expect("group flattened to a list");
    assert_eq!(to.len(), 2);
    assert_eq!(to[0].email, "alice@example.org");
    assert_eq!(to[1].email, "bob@example.net");
}

#[test]
fn decode_charset_helper() {
    // ISO-8859-1: 0xE9 == é, 0xF1 == ñ.
    assert_eq!(
        decode_charset(&[0x43, 0x61, 0x66, 0xE9], Some("ISO-8859-1")),
        "Café"
    );
    // Shift_JIS: こんにちは.
    let sjis = [0x82, 0xb1, 0x82, 0xf1, 0x82, 0xc9, 0x82, 0xbf, 0x82, 0xcd];
    assert_eq!(decode_charset(&sjis, Some("Shift_JIS")), "こんにちは");
    // Unknown label -> UTF-8 fallback.
    assert_eq!(
        decode_charset("héllo".as_bytes(), Some("no-such-charset")),
        "héllo"
    );
    // Invalid UTF-8 under the fallback is replaced lossily, never panics.
    let lossy = decode_charset(&[0x41, 0xC3, 0x28, 0x42], None);
    assert!(lossy.starts_with('A') && lossy.ends_with('B'));
}

#[test]
fn huge_but_bounded_message_parses() {
    // ~2 MiB body of repeated text — bounded, must parse without blowing up.
    let mut raw = b"From: big@example.org\r\nTo: rx@example.org\r\nSubject: big\r\n\
                    Content-Type: text/plain; charset=utf-8\r\n\r\n"
        .to_vec();
    raw.extend(std::iter::repeat_n(b'A', 2 * 1024 * 1024));
    raw.extend_from_slice(b"\r\n");
    let p = parse(&raw).unwrap();
    assert_eq!(p.email.size, raw.len() as u64);
    assert!(p.email.preview.as_deref().unwrap().starts_with("AAAA"));
}
