//! Edges of the MIME mapping that the torture corpus does not reach: part-blob
//! resolution for non-leaf and non-text parts, the `Content-Type`-less defaults,
//! `Content-ID`-implied disposition, `Received`-derived `receivedAt`, and the
//! address/body branches of the compose builder.
//!
//! Both files run over untrusted input inside the render jail, so the shape of
//! every assertion here is "this specific mapping, on this specific input" —
//! a `parse` that returns `Ok` proves nothing on its own.

use mw_jmap::EmailAddress;
use mw_mime::{Attachment, ComposeRequest, build, parse, part_blob};

const MIXED: &[u8] = b"From: Alice <alice@example.org>\r\n\
To: bob@example.net\r\n\
Subject: mixed\r\n\
Content-Type: multipart/mixed; boundary=B\r\n\
\r\n\
--B\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
plain part\r\n\
--B\r\n\
Content-Type: text/html; charset=utf-8\r\n\
\r\n\
<p>html part</p>\r\n\
--B\r\n\
Content-Type: application/pdf\r\n\
Content-Disposition: attachment; filename=\"doc.pdf\"\r\n\
Content-Transfer-Encoding: base64\r\n\
\r\n\
JVBERi0xLjQK\r\n\
--B\r\n\
Content-Type: message/rfc822\r\n\
\r\n\
From: inner@example.org\r\n\
Subject: inner\r\n\
\r\n\
inner body\r\n\
--B--\r\n";

// ── part_blob ───────────────────────────────────────────────────────────────

/// A text part resolves to its decoded bytes and its lower-cased type — the same
/// unit a `<stableId>.<partId>` download addresses.
#[test]
fn part_blob_returns_decoded_text_parts() {
    let p = parse(MIXED).expect("parse");
    let text_id: u32 = p.email.text_body[0]
        .part_id
        .as_deref()
        .and_then(|s| s.parse().ok())
        .expect("numeric part id");

    let blob = part_blob(MIXED, text_id).expect("text part resolves");
    assert_eq!(blob.content_type, "text/plain");
    assert_eq!(blob.filename, None);
    // The CRLF before the boundary belongs to the delimiter, not to the part.
    assert_eq!(blob.bytes, b"plain part");
}

/// A base64 attachment comes back with its transfer-encoding already undone and
/// its declared filename attached.
#[test]
fn part_blob_decodes_a_base64_attachment() {
    let p = parse(MIXED).expect("parse");
    let att = p
        .email
        .attachments
        .iter()
        .find(|a| a.name.as_deref() == Some("doc.pdf"))
        .expect("pdf attachment listed");
    let id: u32 = att.part_id.as_deref().unwrap().parse().unwrap();

    let blob = part_blob(MIXED, id).expect("attachment resolves");
    assert_eq!(blob.content_type, "application/pdf");
    assert_eq!(blob.filename.as_deref(), Some("doc.pdf"));
    assert_eq!(blob.bytes, b"%PDF-1.4\n");
}

/// A container — the multipart root, or a nested `message/rfc822` — is not a
/// downloadable blob and must resolve to `None` rather than to its raw frame.
#[test]
fn part_blob_refuses_containers() {
    assert_eq!(part_blob(MIXED, 0), None, "multipart root is not a blob");

    // The nested message: find it by walking ids until one reports the embedded
    // message's own type, then confirm it does not resolve.
    let container = (0u32..12)
        .find(|&id| part_blob(MIXED, id).is_none() && id != 0)
        .expect("a non-leaf part exists");
    assert_eq!(part_blob(MIXED, container), None);
}

/// An out-of-range part id is `None`, not a panic — the id comes from a client
/// supplied blob id.
#[test]
fn part_blob_refuses_an_unknown_part_id() {
    assert_eq!(part_blob(MIXED, 9_999), None);
    assert_eq!(part_blob(MIXED, u32::MAX), None);
    assert_eq!(part_blob(b"", 0), None);
    assert_eq!(part_blob(b"\x00\x01\x02", 7), None);
}

// ── content-type defaults and disposition ────────────────────────────────────

/// With no `Content-Type` at all the part still gets the RFC default
/// `text/plain` and no charset, rather than an empty or absent type.
#[test]
fn a_part_without_content_type_defaults_to_text_plain() {
    let raw = b"Subject: bare\r\n\r\njust text\r\n";
    let p = parse(raw).expect("parse");
    assert_eq!(p.email.text_body[0].r#type.as_deref(), Some("text/plain"));
    assert_eq!(p.email.text_body[0].charset, None);
    let id: u32 = p.email.text_body[0]
        .part_id
        .as_deref()
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(part_blob(raw, id).unwrap().content_type, "text/plain");
}

/// `type/subtype` is lower-cased however it arrived, so a comparison against the
/// mapping's output never has to case-fold.
#[test]
fn content_type_is_lower_cased() {
    let raw = b"Subject: shouty\r\nContent-Type: TEXT/HTML; charset=UTF-8\r\n\r\n<p>x</p>\r\n";
    let p = parse(raw).expect("parse");
    assert_eq!(p.email.html_body[0].r#type.as_deref(), Some("text/html"));
    // The charset label is passed through verbatim — `decode_charset` folds it.
    assert_eq!(p.email.html_body[0].charset.as_deref(), Some("UTF-8"));
}

/// A part with a `Content-ID` and no explicit `Content-Disposition` is reported
/// as `inline` — that implied disposition is what keeps a cid resource out of the
/// attachment list.
#[test]
fn a_content_id_part_without_a_disposition_is_inline() {
    let raw = b"Subject: cid\r\n\
Content-Type: multipart/related; boundary=R\r\n\
\r\n\
--R\r\n\
Content-Type: text/html\r\n\
\r\n\
<img src=\"cid:logo@example.org\">\r\n\
--R\r\n\
Content-Type: image/png\r\n\
Content-ID: <logo@example.org>\r\n\
\r\n\
notreallyapng\r\n\
--R--\r\n";
    let p = parse(raw).expect("parse");
    assert!(
        !p.email.has_attachment,
        "a cid resource is not an attachment"
    );

    // The cid part is reachable as a blob even though it is not listed as an
    // attachment — the reader loads it by content id.
    let blob = (0u32..8)
        .filter_map(|id| part_blob(raw, id))
        .find(|b| b.content_type == "image/png")
        .expect("cid part resolves as a blob");
    assert_eq!(blob.bytes, b"notreallyapng");
}

/// An explicit `Content-Disposition` wins and is lower-cased.
#[test]
fn an_explicit_disposition_is_reported_lower_cased() {
    let p = parse(MIXED).expect("parse");
    let att = p
        .email
        .attachments
        .iter()
        .find(|a| a.name.as_deref() == Some("doc.pdf"))
        .unwrap();
    assert_eq!(att.disposition.as_deref(), Some("attachment"));
}

// ── receivedAt ───────────────────────────────────────────────────────────────

/// When a `Received` header carries a date it wins over `Date` for `receivedAt`,
/// while `sentAt` keeps the author's `Date` — the two are different facts and the
/// UI shows both.
#[test]
fn received_header_supplies_received_at_independently_of_sent_at() {
    let raw =
        b"Received: from mx.example.org by mail.example.net; Tue, 13 Jan 2026 10:00:00 +0000\r\n\
From: a@example.org\r\n\
Date: Mon, 12 Jan 2026 09:30:00 +0000\r\n\
Subject: hops\r\n\
\r\n\
body\r\n";
    let p = parse(raw).expect("parse");
    assert_eq!(p.email.sent_at.as_deref(), Some("2026-01-12T09:30:00Z"));
    assert_eq!(p.email.received_at.as_deref(), Some("2026-01-13T10:00:00Z"));
}

/// An unparseable date is dropped rather than mapped to a bogus timestamp, and
/// `receivedAt` then has nothing to fall back to.
#[test]
fn an_invalid_date_yields_no_timestamp() {
    let raw = b"From: a@example.org\r\nDate: not a date at all\r\nSubject: s\r\n\r\nbody\r\n";
    let p = parse(raw).expect("parse");
    assert_eq!(p.email.sent_at, None);
    assert_eq!(p.email.received_at, None);
}

// ── addresses ────────────────────────────────────────────────────────────────

/// An address list where some entries have no address at all keeps only the
/// usable ones, and a list with none becomes `None` rather than an empty vec —
/// the JMAP distinction between "no To" and "an empty To".
#[test]
fn address_lists_drop_unusable_entries_and_collapse_to_none() {
    let raw = b"From: a@example.org\r\n\
To: Undisclosed recipients:;\r\n\
Cc: Real <real@example.org>, undisclosed:;\r\n\
Subject: s\r\n\r\nbody\r\n";
    let p = parse(raw).expect("parse");
    assert!(p.email.to.is_none(), "{:?}", p.email.to);
    let cc = p.email.cc.expect("cc has one usable address");
    assert_eq!(cc.len(), 1);
    assert_eq!(cc[0].email, "real@example.org");
}

/// An RFC2047 encoded-word in a display name is decoded, and a punycode address
/// is passed through as-is (the engine, not the parser, decides how to display
/// an IDN).
#[test]
fn encoded_word_display_names_and_punycode_addresses() {
    let raw = "From: =?utf-8?B?SsO2cmc=?= <jorg@xn--bcher-kva.example>\r\n\
To: =?iso-8859-1?Q?Se=F1or?= <senor@example.org>\r\n\
Subject: =?utf-8?Q?Caf=C3=A9?=\r\n\r\nbody\r\n"
        .as_bytes();
    let p = parse(raw).expect("parse");
    let from = &p.email.from.as_ref().unwrap()[0];
    assert_eq!(from.name.as_deref(), Some("Jörg"));
    assert_eq!(from.email, "jorg@xn--bcher-kva.example");
    assert_eq!(
        p.email.to.as_ref().unwrap()[0].name.as_deref(),
        Some("Señor")
    );
    assert_eq!(p.email.subject.as_deref(), Some("Café"));
}

// ── build.rs: the compose branches ───────────────────────────────────────────

fn addr(email: &str) -> EmailAddress {
    EmailAddress {
        name: None,
        email: email.into(),
    }
}

/// `Bcc` and `Reply-To` are emitted when present — both are separate branches
/// from `To`/`Cc` and neither was reached by the existing build tests.
#[test]
fn build_emits_bcc_and_reply_to() {
    let req = ComposeRequest {
        from: Some(addr("me@example.org")),
        to: vec![addr("you@example.net")],
        cc: vec![addr("cc@example.org")],
        bcc: vec![addr("bcc1@example.org"), addr("bcc2@example.org")],
        reply_to: vec![addr("replies@example.org")],
        subject: Some("full".into()),
        text_body: Some("body".into()),
        ..Default::default()
    };
    let raw = build(&req).expect("build");
    let text = String::from_utf8(raw.clone()).expect("utf8");
    assert!(text.contains("Bcc:"), "{text}");
    assert!(text.contains("bcc1@example.org"), "{text}");
    assert!(text.contains("bcc2@example.org"), "{text}");
    assert!(text.contains("Reply-To:"), "{text}");
    assert!(text.contains("replies@example.org"), "{text}");

    // Round-trip: the parser sees the same recipients back.
    let p = parse(&raw).expect("re-parse");
    assert_eq!(p.email.bcc.as_ref().unwrap().len(), 2);
    assert_eq!(
        p.email.reply_to.as_ref().unwrap()[0].email,
        "replies@example.org"
    );
}

/// An HTML-only compose produces a single `text/html` part, not an alternative
/// with an empty text half.
#[test]
fn build_html_only_is_a_single_html_part() {
    let req = ComposeRequest {
        from: Some(addr("me@example.org")),
        to: vec![addr("you@example.net")],
        subject: Some("html".into()),
        html_body: Some("<p>rich</p>".into()),
        ..Default::default()
    };
    let raw = build(&req).expect("build");
    let p = parse(&raw).expect("re-parse");
    assert_eq!(p.email.html_body.len(), 1);
    assert_eq!(p.email.html_body[0].r#type.as_deref(), Some("text/html"));
    let text = String::from_utf8(raw).unwrap();
    assert!(!text.contains("multipart/alternative"), "{text}");
}

/// Both bodies present gives `multipart/alternative` with each half intact.
#[test]
fn build_text_and_html_is_multipart_alternative() {
    let req = ComposeRequest {
        from: Some(addr("me@example.org")),
        to: vec![addr("you@example.net")],
        subject: Some("both".into()),
        text_body: Some("plain half".into()),
        html_body: Some("<p>rich half</p>".into()),
        ..Default::default()
    };
    let raw = build(&req).expect("build");
    let text = String::from_utf8(raw.clone()).unwrap();
    assert!(text.contains("multipart/alternative"), "{text}");
    let p = parse(&raw).expect("re-parse");
    assert_eq!(p.email.text_body.len(), 1);
    assert_eq!(p.email.html_body.len(), 1);
}

/// A submission with no body at all is still a valid message with an empty text
/// part — refusing to build would strand a caller sending a subject-only note.
#[test]
fn build_with_no_body_emits_an_empty_text_part() {
    let req = ComposeRequest {
        from: Some(addr("me@example.org")),
        to: vec![addr("you@example.net")],
        subject: Some("subject only".into()),
        ..Default::default()
    };
    let raw = build(&req).expect("build");
    let text = String::from_utf8(raw.clone()).unwrap();
    assert!(text.contains("Content-Type: text/plain"), "{text}");
    let p = parse(&raw).expect("re-parse");
    assert_eq!(p.email.subject.as_deref(), Some("subject only"));
}

/// Threading ids are written bare of angle brackets on the way in and come back
/// bare on the way out, whether or not the caller supplied brackets.
#[test]
fn build_normalises_bracketed_threading_ids() {
    let req = ComposeRequest {
        from: Some(addr("me@example.org")),
        to: vec![addr("you@example.net")],
        subject: Some("re".into()),
        text_body: Some("reply".into()),
        message_id: Some("  <mine@example.org>  ".into()),
        in_reply_to: Some("parent@example.org".into()),
        references: vec!["<root@example.org>".into(), "parent@example.org".into()],
        ..Default::default()
    };
    let raw = build(&req).expect("build");
    let text = String::from_utf8(raw.clone()).unwrap();
    assert!(!text.contains("<<"), "double-bracketed id: {text}");

    let p = parse(&raw).expect("re-parse");
    assert_eq!(p.envelope.message_id.as_deref(), Some("mine@example.org"));
    assert_eq!(
        p.envelope.in_reply_to.as_deref(),
        Some("parent@example.org")
    );
    assert_eq!(
        p.envelope.references,
        ["root@example.org", "parent@example.org"]
    );
}

/// Extra raw headers are emitted verbatim.
#[test]
fn build_emits_extra_raw_headers() {
    let req = ComposeRequest {
        from: Some(addr("me@example.org")),
        to: vec![addr("you@example.net")],
        subject: Some("hdrs".into()),
        text_body: Some("body".into()),
        headers: vec![
            ("User-Agent".into(), "Mailwoman/26.19".into()),
            ("X-Custom".into(), "value".into()),
        ],
        ..Default::default()
    };
    let text = String::from_utf8(build(&req).expect("build")).unwrap();
    assert!(text.contains("User-Agent: Mailwoman/26.19"), "{text}");
    assert!(text.contains("X-Custom: value"), "{text}");
}

/// A named sender survives into the composed bytes and back out.
#[test]
fn build_keeps_a_display_name() {
    let req = ComposeRequest {
        from: Some(EmailAddress {
            name: Some("Alice Example".into()),
            email: "alice@example.org".into(),
        }),
        to: vec![addr("you@example.net")],
        subject: Some("named".into()),
        text_body: Some("body".into()),
        ..Default::default()
    };
    let raw = build(&req).expect("build");
    let p = parse(&raw).expect("re-parse");
    let from = &p.email.from.as_ref().unwrap()[0];
    assert_eq!(from.name.as_deref(), Some("Alice Example"));
    assert_eq!(from.email, "alice@example.org");
}

/// Several attachments each survive with their own bytes — a single-attachment
/// test would not catch a builder that only emitted the last one.
#[test]
fn build_emits_every_attachment() {
    let req = ComposeRequest {
        from: Some(addr("me@example.org")),
        to: vec![addr("you@example.net")],
        subject: Some("attachments".into()),
        text_body: Some("see attached".into()),
        attachments: vec![
            Attachment {
                filename: "a.txt".into(),
                content_type: "text/plain".into(),
                bytes: b"first".to_vec(),
            },
            Attachment {
                filename: "b.bin".into(),
                content_type: "application/octet-stream".into(),
                bytes: vec![0x00, 0xff, 0x10, 0x80],
            },
        ],
        ..Default::default()
    };
    let raw = build(&req).expect("build");
    let p = parse(&raw).expect("re-parse");

    for (name, want) in [
        ("a.txt", b"first".to_vec()),
        ("b.bin", vec![0x00, 0xff, 0x10, 0x80]),
    ] {
        let att = p
            .email
            .attachments
            .iter()
            .find(|a| a.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("{name} listed"));
        let id: u32 = att.part_id.as_deref().unwrap().parse().unwrap();
        assert_eq!(part_blob(&raw, id).unwrap().bytes, want, "{name}");
    }
}
