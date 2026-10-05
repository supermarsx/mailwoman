//! `build_with`: inline parts (`multipart/related`), the read-receipt request,
//! and the checks that keep a hostile value from adding a header line or a
//! MIME part. Assertions are made on the re-parsed message.

use mail_parser::{Message, MessageParser, MimeHeaders, PartType};
use mw_mime::{
    Attachment, ComposeExtras, ComposeRequest, EmailAddress, InlinePart, build, build_with,
    generate_content_id, html_cid_references, parse, part_blob,
};

const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR-not-a-real-image";

fn addr(name: Option<&str>, email: &str) -> EmailAddress {
    EmailAddress {
        name: name.map(str::to_string),
        email: email.to_string(),
    }
}

fn request(html: Option<&str>) -> ComposeRequest {
    ComposeRequest {
        from: Some(addr(Some("Alice Example"), "alice@example.org")),
        to: vec![addr(None, "bob@example.net")],
        subject: Some("Picture".into()),
        text_body: Some("see the picture".into()),
        html_body: html.map(str::to_string),
        message_id: Some("compose-1@example.org".into()),
        ..ComposeRequest::default()
    }
}

fn image(cid: &str) -> InlinePart {
    InlinePart {
        cid: cid.into(),
        content_type: "image/png".into(),
        filename: Some("picture.png".into()),
        bytes: PNG.to_vec(),
    }
}

/// One way of making a request hostile.
type Change = Box<dyn Fn(&mut ComposeRequest)>;

fn inline(parts: Vec<InlinePart>) -> ComposeExtras {
    ComposeExtras {
        inline_parts: parts,
        ..ComposeExtras::default()
    }
}

fn reparse(raw: &[u8]) -> Message<'_> {
    MessageParser::default()
        .parse(raw)
        .expect("built message parses")
}

/// `type/subtype` of every part, in part order.
fn shape(message: &Message<'_>) -> Vec<String> {
    message
        .parts
        .iter()
        .map(|p| {
            let ct = p.content_type().expect("every built part has a type");
            format!("{}/{}", ct.ctype(), ct.subtype().unwrap_or(""))
        })
        .collect()
}

/// The lines of the top-level header section, unfolded.
fn header_lines(raw: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(raw);
    let head = text.split("\r\n\r\n").next().unwrap_or("");
    let mut lines: Vec<String> = Vec::new();
    for line in head.split("\r\n") {
        match lines.last_mut() {
            Some(last) if line.starts_with([' ', '\t']) => last.push_str(line),
            _ => lines.push(line.to_string()),
        }
    }
    lines
}

/// Header names of every header of every part.
fn all_header_names(message: &Message<'_>) -> Vec<String> {
    message
        .parts
        .iter()
        .flat_map(|p| p.headers.iter().map(|h| h.name.as_str().to_string()))
        .collect()
}

/// The built message with what legitimately differs between two builds
/// (boundaries and the date) replaced by fixed text.
fn normalised(raw: &[u8]) -> String {
    let mut text = String::from_utf8(raw.to_vec()).expect("utf8");
    while let Some(at) = text.find("boundary=\"") {
        let start = at + "boundary=\"".len();
        let end = start + text[start..].find('"').expect("closing quote");
        let boundary = text[start..end].to_string();
        text = text
            .replace(&format!("boundary=\"{boundary}\""), "boundary=B")
            .replace(&boundary, "B");
    }
    text.lines()
        .filter(|l| !l.starts_with("Date: "))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn an_inline_part_round_trips_as_a_cid_resource() {
    let cid = generate_content_id();
    let html = format!("<p>look <img src=\"cid:{cid}\" alt=\"\"></p>");
    let raw = build_with(&request(Some(&html)), &inline(vec![image(&cid)])).expect("build");

    let message = reparse(&raw);
    assert_eq!(
        shape(&message),
        [
            "multipart/alternative",
            "text/plain",
            "multipart/related",
            "text/html",
            "image/png"
        ]
    );
    let related = message.parts[2].content_type().expect("related type");
    assert_eq!(related.attribute("type"), Some("text/html"));

    // The reference in the HTML resolves to exactly one part with that id.
    let html_part = message.parts[3].text_contents().expect("html text");
    let refs = html_cid_references(html_part);
    assert_eq!(refs, std::slice::from_ref(&cid));
    let hits: Vec<_> = message
        .parts
        .iter()
        .filter(|p| p.content_id() == Some(cid.as_str()))
        .collect();
    assert_eq!(hits.len(), 1);
    let part = hits[0];
    assert!(
        part.content_disposition()
            .is_some_and(mail_parser::ContentType::is_inline)
    );
    assert_eq!(part.attachment_name(), Some("picture.png"));
    assert_eq!(part.contents(), PNG);

    // The JMAP mapping treats it as part of the body, not as a file.
    let parsed = parse(&raw).expect("parse");
    assert!(!parsed.email.has_attachment);
    assert!(parsed.email.attachments.is_empty());
    assert_eq!(parsed.email.html_body.len(), 1);
    assert_eq!(part_blob(&raw, 4).expect("image blob").bytes, PNG);
}

#[test]
fn inline_parts_sit_in_related_inside_mixed_beside_ordinary_attachments() {
    let mut req = request(Some("<img src=\"cid:one@example.org\"><img src='cid:two'>"));
    req.attachments.push(Attachment {
        filename: "invoice.pdf".into(),
        content_type: "application/pdf".into(),
        bytes: b"%PDF-1.4\n".to_vec(),
    });
    let extras = inline(vec![image("one@example.org"), image("two")]);
    let raw = build_with(&req, &extras).expect("build");

    let message = reparse(&raw);
    assert_eq!(
        shape(&message),
        [
            "multipart/mixed",
            "multipart/alternative",
            "text/plain",
            "multipart/related",
            "text/html",
            "image/png",
            "image/png",
            "application/pdf"
        ]
    );
    let PartType::Multipart(top) = &message.parts[0].body else {
        panic!("root is not multipart");
    };
    assert_eq!(top.len(), 2, "mixed holds the body and the one attachment");

    let parsed = parse(&raw).expect("parse");
    assert!(parsed.email.has_attachment);
    let names: Vec<_> = parsed
        .email
        .attachments
        .iter()
        .map(|p| p.name.clone())
        .collect();
    assert_eq!(names, [Some("invoice.pdf".to_string())]);
}

#[test]
fn html_without_a_text_body_is_related_at_the_top() {
    let mut req = request(Some("<img src=\"cid:only\">"));
    req.text_body = None;
    let raw = build_with(&req, &inline(vec![image("only")])).expect("build");
    assert_eq!(
        shape(&reparse(&raw)),
        ["multipart/related", "text/html", "image/png"]
    );
}

#[test]
fn inline_parts_without_an_html_body_are_refused() {
    let err = build_with(&request(None), &inline(vec![image("only")])).unwrap_err();
    assert!(err.to_string().contains("HTML body"), "{err}");
}

#[test]
fn empty_extras_build_what_build_builds() {
    let plain = request(None);
    let both = request(Some("<p>hi</p>"));
    let mut attached = request(Some("<p>hi</p>"));
    attached.attachments.push(Attachment {
        filename: "a.txt".into(),
        content_type: "text/plain".into(),
        bytes: b"abc".to_vec(),
    });
    for req in [plain, both, attached] {
        let old = build(&req).expect("build");
        let new = build_with(&req, &ComposeExtras::default()).expect("build_with");
        assert_eq!(normalised(&old), normalised(&new));
    }
}

#[test]
fn boundaries_differ_between_builds_and_are_32_hex_digits() {
    let req = request(Some("<img src=\"cid:only\">"));
    let boundaries = |raw: &[u8]| -> Vec<String> {
        reparse(raw)
            .parts
            .iter()
            .filter_map(|p| p.content_type()?.attribute("boundary").map(str::to_string))
            .collect()
    };
    let first = boundaries(&build_with(&req, &inline(vec![image("only")])).expect("build"));
    let second = boundaries(&build_with(&req, &inline(vec![image("only")])).expect("build"));
    assert_eq!(first.len(), 2);
    for b in first.iter().chain(&second) {
        let digits = b.strip_prefix("mw_").expect("prefix");
        assert_eq!(digits.len(), 32, "{b}");
        assert!(digits.bytes().all(|c| c.is_ascii_hexdigit()), "{b}");
    }
    let mut all: Vec<_> = first.iter().chain(&second).collect();
    all.sort();
    all.dedup();
    assert_eq!(all.len(), 4, "a boundary repeated");
}

#[test]
fn a_hostile_content_id_is_refused() {
    let req = request(Some("<img src=\"cid:ok@example.org\">"));
    // Precondition: a benign id builds and comes back as written.
    let raw = build_with(&req, &inline(vec![image("ok@example.org")])).expect("benign id builds");
    assert_eq!(reparse(&raw).parts[4].content_id(), Some("ok@example.org"));

    for cid in [
        "x@example.org>\r\nBcc: victim@example.test\r\nX: <y",
        "x\r\n\r\n--boundary\r\nContent-Type: text/html\r\n\r\n<b>",
        "x\ny",
        "x\ry",
        "x\0y",
        "x y",
        "x\ty",
        "<x@example.org>",
        "x>",
        "",
    ] {
        let err = build_with(&req, &inline(vec![image(cid)]))
            .expect_err(&format!("{cid:?} was accepted"));
        assert!(
            !err.to_string().chars().any(char::is_control),
            "error echoes a raw control character: {err:?}"
        );
    }
    assert!(build_with(&req, &inline(vec![image(&"c".repeat(256))])).is_err());
}

#[test]
fn two_inline_parts_cannot_share_a_content_id() {
    let req = request(Some("<img src=\"cid:same\">"));
    let err = build_with(&req, &inline(vec![image("same"), image("same")])).unwrap_err();
    assert!(err.to_string().contains("two inline parts"), "{err}");
}

#[test]
fn a_hostile_inline_content_type_is_refused() {
    let req = request(Some("<img src=\"cid:only\">"));
    for content_type in [
        "image/png\r\nX-Injected: 1",
        "image/png\r\n\r\n<script>",
        "image/png; boundary=x",
        "image/png\0",
        "image /png",
        "multipart/mixed",
        "message/rfc822",
        "png",
        "/png",
        "image/",
        "",
    ] {
        let mut part = image("only");
        part.content_type = content_type.into();
        assert!(
            build_with(&req, &inline(vec![part])).is_err(),
            "{content_type:?} was accepted"
        );
    }
}

#[test]
fn a_hostile_file_name_adds_no_header_and_no_part() {
    let req = {
        let mut req = request(Some("<img src=\"cid:only\">"));
        req.attachments.push(Attachment {
            filename: "report.pdf".into(),
            content_type: "application/pdf".into(),
            bytes: b"%PDF".to_vec(),
        });
        req
    };
    let benign = build_with(&req, &inline(vec![image("only")])).expect("benign");
    let benign = reparse(&benign);
    assert_eq!(benign.parts[5].attachment_name(), Some("picture.png"));
    assert_eq!(benign.parts[6].attachment_name(), Some("report.pdf"));

    let hostile = "a.png\"\r\nX-Injected: 1\r\nBcc: victim@example.test\r\n\r\n--x\r\n\0";
    let mut part = image("only");
    part.filename = Some(hostile.into());
    let mut req = req;
    req.attachments[0].filename = hostile.into();
    let raw = build_with(&req, &inline(vec![part])).expect("a file name is cleaned, not refused");

    let message = reparse(&raw);
    assert_eq!(shape(&message), shape(&benign), "the part tree changed");
    let names = all_header_names(&message);
    assert!(!names.iter().any(|n| n.eq_ignore_ascii_case("X-Injected")));
    assert!(!names.iter().any(|n| n.eq_ignore_ascii_case("Bcc")));
    assert_eq!(names.len(), all_header_names(&benign).len());
    assert!(message.bcc().is_none());
    for index in [5, 6] {
        let name = message.parts[index].attachment_name().expect("file name");
        assert!(!name.chars().any(char::is_control), "{name:?}");
        assert!(name.starts_with("a.png\""), "{name:?}");
    }
}

#[test]
fn a_receipt_request_is_one_header_and_reads_back() {
    let req = request(None);
    // Precondition: no request, no header.
    let without = build_with(&req, &ComposeExtras::default()).expect("build");
    assert!(!header_lines(&without).iter().any(|l| {
        l.to_ascii_lowercase()
            .starts_with("disposition-notification-to")
    }));
    assert_eq!(
        parse(&without)
            .expect("parse")
            .receipt
            .disposition_notification_to,
        None
    );

    let extras = ComposeExtras {
        receipt_to: Some("alice@example.org".into()),
        ..ComposeExtras::default()
    };
    let with = build_with(&req, &extras).expect("build");
    let lines: Vec<_> = header_lines(&with)
        .into_iter()
        .filter(|l| {
            l.to_ascii_lowercase()
                .starts_with("disposition-notification-to")
        })
        .collect();
    assert_eq!(lines, ["Disposition-Notification-To: <alice@example.org>"]);
    assert_eq!(
        parse(&with)
            .expect("parse")
            .receipt
            .disposition_notification_to
            .as_deref(),
        Some("alice@example.org")
    );
}

#[test]
fn a_hostile_receipt_address_is_refused() {
    let req = request(None);
    let benign = ComposeExtras {
        receipt_to: Some("alice@example.org".into()),
        ..ComposeExtras::default()
    };
    let benign_names = all_header_names(&reparse(&build_with(&req, &benign).expect("benign")));

    for addr in [
        "a@b\r\nBcc: x@y",
        "a@b>\r\nBcc: <x@y",
        "a@b\nX-Injected: 1",
        "a@b\r",
        "a@b\0",
        "a@b, x@y",
        "a@b,x@y",
        "a@b> <x@y",
        "Alice <a@b>",
        "a@b ",
        "a",
        "",
    ] {
        let extras = ComposeExtras {
            receipt_to: Some(addr.into()),
            ..ComposeExtras::default()
        };
        assert!(build_with(&req, &extras).is_err(), "{addr:?} was accepted");
    }
    // Nothing above reached the builder, so there is no second shape to
    // compare; the benign message has exactly one request header and no Bcc.
    assert_eq!(
        benign_names
            .iter()
            .filter(|n| n.eq_ignore_ascii_case("Disposition-Notification-To"))
            .count(),
        1
    );
    assert!(!benign_names.iter().any(|n| n.eq_ignore_ascii_case("Bcc")));
}

#[test]
fn the_receipt_header_cannot_be_written_through_raw_headers() {
    let mut req = request(None);
    req.headers.push(("User-Agent".into(), "Mailwoman".into()));
    build_with(&req, &ComposeExtras::default()).expect("an ordinary raw header builds");

    for name in ["Disposition-Notification-To", "disposition-notification-to"] {
        let mut req = request(None);
        req.headers.push((name.into(), "<x@y>".into()));
        assert!(build_with(&req, &ComposeExtras::default()).is_err());
    }
}

#[test]
fn build_with_refuses_control_characters_in_values_written_verbatim() {
    let ok = request(Some("<p>hi</p>"));
    build_with(&ok, &ComposeExtras::default()).expect("the unmodified request builds");

    let inject = "x\r\nBcc: victim@example.test";
    let cases: Vec<(&str, Change)> = vec![
        ("subject", Box::new(|r| r.subject = Some(inject.into()))),
        ("subject NUL", Box::new(|r| r.subject = Some("a\0b".into()))),
        (
            "message id",
            Box::new(|r| r.message_id = Some(inject.into())),
        ),
        (
            "in-reply-to",
            Box::new(|r| r.in_reply_to = Some(inject.into())),
        ),
        (
            "references",
            Box::new(|r| r.references = vec![inject.into()]),
        ),
        (
            "from",
            Box::new(|r| r.from = Some(addr(None, "a@b>\r\nBcc: <v@example.test"))),
        ),
        ("to", Box::new(|r| r.to = vec![addr(None, "a@b\nBcc: v@c")])),
        ("cc", Box::new(|r| r.cc = vec![addr(None, "a@b> <v@c")])),
        ("bcc", Box::new(|r| r.bcc = vec![addr(None, "a@b, v@c")])),
        (
            "reply-to",
            Box::new(|r| r.reply_to = vec![addr(None, "a b@c")]),
        ),
        (
            "header value",
            Box::new(|r| r.headers = vec![("X-A".into(), inject.into())]),
        ),
        (
            "header name",
            Box::new(|r| r.headers = vec![("X-A: 1\r\nBcc".into(), "v@c".into())]),
        ),
        (
            "header name colon",
            Box::new(|r| r.headers = vec![("Bcc: v@c\r\nX".into(), "1".into())]),
        ),
        (
            "attachment type",
            Box::new(|r| {
                r.attachments = vec![Attachment {
                    filename: "a".into(),
                    content_type: "text/plain\r\nX-Injected: 1".into(),
                    bytes: Vec::new(),
                }];
            }),
        ),
    ];
    for (what, change) in cases {
        let mut req = ok.clone();
        change(&mut req);
        let err =
            build_with(&req, &ComposeExtras::default()).expect_err(&format!("{what}: accepted"));
        assert!(
            !err.to_string().chars().any(char::is_control),
            "{what}: {err:?}"
        );
    }
}

#[test]
fn several_in_reply_to_ids_joined_by_the_engine_still_build() {
    // `mw-engine` joins a list of ids with `> <` so the header reads
    // `<a> <b>`; the check on ids must not refuse that.
    let mut req = request(None);
    req.in_reply_to = Some("a@example.org> <b@example.org".into());
    let raw = build_with(&req, &ComposeExtras::default()).expect("build");
    assert!(
        header_lines(&raw).contains(&"In-Reply-To: <a@example.org> <b@example.org>".to_string())
    );
}

#[test]
fn a_hostile_display_name_adds_no_header() {
    let mut req = request(None);
    req.from = Some(addr(
        Some("Alice\r\nBcc: victim@example.test\r\nX-Injected: 1"),
        "alice@example.org",
    ));
    let raw = build_with(&req, &ComposeExtras::default()).expect("a display name is encoded");
    let message = reparse(&raw);
    assert!(message.bcc().is_none());
    assert!(
        !all_header_names(&message)
            .iter()
            .any(|n| n.eq_ignore_ascii_case("X-Injected"))
    );
    assert_eq!(
        message
            .from()
            .and_then(|a| a.first())
            .and_then(|a| a.address()),
        Some("alice@example.org")
    );
}

fn receipt_of(headers: &str) -> mw_mime::ReceiptHeaders {
    let raw = format!("From: a@example.org\r\n{headers}Subject: s\r\n\r\nbody\r\n");
    parse(raw.as_bytes()).expect("parse").receipt
}

#[test]
fn the_request_address_is_read_only_when_it_is_one_acceptable_mailbox() {
    for (value, expected) in [
        ("alice@example.org", Some("alice@example.org")),
        ("<alice@example.org>", Some("alice@example.org")),
        (
            "Alice Example <alice@example.org>",
            Some("alice@example.org"),
        ),
        (
            "\"Example, Alice\" <alice@example.org>",
            Some("alice@example.org"),
        ),
        (
            "\"a \\\" <x@y>\" <alice@example.org>",
            Some("alice@example.org"),
        ),
        (
            "alice@example.org (Alice (home))",
            Some("alice@example.org"),
        ),
        (
            "=?utf-8?Q?Al=C3=AFce?= <alice@example.org>",
            Some("alice@example.org"),
        ),
        ("Alice\r\n <alice@example.org>", Some("alice@example.org")),
        ("alice@example.org, bob@example.net", None),
        ("<alice@example.org>, <bob@example.net>", None),
        ("<alice@example.org> <bob@example.net>", None),
        ("friends: alice@example.org;", None),
        ("<alice@example.org> trailing", None),
        ("\"a b\"@example.org", None),
        ("<alice@example.org", None),
        ("alice", None),
        ("<>", None),
        ("", None),
        ("\"unterminated <alice@example.org>", None),
        ("(unterminated <alice@example.org>", None),
    ] {
        let got = receipt_of(&format!("Disposition-Notification-To: {value}\r\n"));
        assert_eq!(
            got.disposition_notification_to.as_deref(),
            expected,
            "{value:?}"
        );
    }
}

#[test]
fn two_request_headers_are_no_request() {
    let one = receipt_of("Disposition-Notification-To: alice@example.org\r\n");
    assert_eq!(
        one.disposition_notification_to.as_deref(),
        Some("alice@example.org")
    );
    let two = receipt_of(
        "Disposition-Notification-To: alice@example.org\r\n\
         disposition-notification-to: mallory@example.test\r\n",
    );
    assert_eq!(two.disposition_notification_to, None);
}

#[test]
fn return_path_and_list_headers_are_reported() {
    let none = receipt_of("");
    assert_eq!(none.return_path, None);
    assert!(!none.from_list);

    assert_eq!(
        receipt_of("Return-Path: <alice@example.org>\r\n")
            .return_path
            .as_deref(),
        Some("alice@example.org")
    );
    assert_eq!(
        receipt_of("Return-Path: <>\r\n").return_path.as_deref(),
        Some("")
    );
    // The first header is the one the final delivery added.
    assert_eq!(
        receipt_of("Return-Path: <last@example.org>\r\nReturn-Path: <first@example.org>\r\n")
            .return_path
            .as_deref(),
        Some("last@example.org")
    );
    assert_eq!(
        receipt_of("Return-Path: <a b@example.org>\r\n").return_path,
        None
    );

    assert!(receipt_of("List-Id: Announcements <announce.example.org>\r\n").from_list);
    assert!(receipt_of("list-id: <x.example.org>\r\n").from_list);
    for value in ["bulk", "list", "junk", " Bulk "] {
        assert!(
            receipt_of(&format!("Precedence: {value}\r\n")).from_list,
            "{value}"
        );
    }
    for value in ["first-class", "special-delivery", "bulky"] {
        assert!(
            !receipt_of(&format!("Precedence: {value}\r\n")).from_list,
            "{value}"
        );
    }
}

#[test]
fn cid_references_are_found_in_tags_only() {
    assert_eq!(
        html_cid_references(
            "<p>about cid:in-text</p>\
             <img src=\"cid:a@example.org\">\
             <IMG SRC='CID:b'>\
             <img src=cid:c>\
             <td background=\"cid:d\" style=\"background:url(cid:e)\">\
             <img src=\"cid:a@example.org\">\
             <img src=\"cid:f%40example.org\">\
             <img alt=\"1 > 0\" src=\"cid:g\">\
             <img src=\"acid:h\">\
             <img src=\"cid:\">"
        ),
        ["a@example.org", "b", "c", "d", "e", "f@example.org", "g"]
    );
    assert!(html_cid_references("").is_empty());
    assert!(html_cid_references("cid:x <").is_empty());
    assert_eq!(html_cid_references("<cid:%zz%4"), ["%zz%4"]);
    assert_eq!(html_cid_references("<a b=\"cid:é\u{2028}x"), ["é\u{2028}x"]);
}
