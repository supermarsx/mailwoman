//! The reader's attachments bar and the Security panel must list the same
//! parts. The panel enumerates `mail_parser`'s `Message::attachments()`; the
//! bar lists `parse().email.attachments`. Every fixture below must agree.

use mail_parser::MessageParser;
use mw_mime::parse;

const B64: &str = "QUJD";

fn mixed(parts: &[&str]) -> Vec<u8> {
    let mut s = String::from(
        "From: a@example.org\r\nTo: b@example.org\r\nSubject: t\r\nMIME-Version: 1.0\r\n\
Content-Type: multipart/mixed; boundary=B\r\n\r\n--B\r\nContent-Type: text/plain\r\n\r\nhi\r\n",
    );
    for p in parts {
        s.push_str(&format!("--B\r\n{p}\r\n\r\n{B64}\r\n"));
    }
    s.push_str("--B--\r\n");
    s.into_bytes()
}

fn assert_parity(label: &str, raw: &[u8], expect: usize) {
    let panel = MessageParser::default().parse(raw).unwrap().attachments().count();
    let bar = parse(raw).unwrap().email.attachments.len();
    assert_eq!(panel, bar, "{label}: Security panel lists {panel}, bar lists {bar}");
    assert_eq!(bar, expect, "{label}: expected {expect} listed parts");
}

#[test]
fn every_part_the_security_panel_lists_reaches_the_bar() {
    let b64 = "Content-Transfer-Encoding: base64";
    let cases: Vec<(&str, Vec<u8>, usize)> = vec![
        ("pdf attachment", mixed(&[&format!("Content-Type: application/pdf; name=\"a.pdf\"\r\nContent-Disposition: attachment; filename=\"a.pdf\"\r\n{b64}")]), 1),
        ("no filename", mixed(&[&format!("Content-Type: application/octet-stream\r\nContent-Disposition: attachment\r\n{b64}")]), 1),
        ("cid image", mixed(&[&format!("Content-Type: image/png; name=\"l.png\"\r\nContent-Disposition: inline; filename=\"l.png\"\r\nContent-ID: <l@y>\r\n{b64}")]), 1),
        ("cid exe without disposition", mixed(&[&format!("Content-Type: application/x-msdownload; name=\"x.exe\"\r\nContent-ID: <x@y>\r\n{b64}")]), 1),
        ("tnef", mixed(&[&format!("Content-Type: application/ms-tnef; name=\"winmail.dat\"\r\nContent-Disposition: attachment; filename=\"winmail.dat\"\r\n{b64}")]), 1),
        ("calendar", mixed(&[&format!("Content-Type: text/calendar; method=REQUEST; name=\"invite.ics\"\r\nContent-Disposition: attachment; filename=\"invite.ics\"\r\n{b64}")]), 1),
        ("smime signature", mixed(&[&format!("Content-Type: application/pkcs7-signature; name=\"smime.p7s\"\r\nContent-Disposition: attachment; filename=\"smime.p7s\"\r\n{b64}")]), 1),
        ("pgp signature", mixed(&[&format!("Content-Type: application/pgp-signature; name=\"signature.asc\"\r\n{b64}")]), 1),
        ("forwarded message", mixed(&["Content-Type: message/rfc822\r\nContent-Disposition: attachment; filename=\"fwd.eml\"\r\n\r\nFrom: c@example.org\r\nSubject: inner\r\n\r\ninner body"]), 1),
        ("two files", mixed(&[
            &format!("Content-Type: application/pdf; name=\"a.pdf\"\r\nContent-Disposition: attachment; filename=\"a.pdf\"\r\n{b64}"),
            &format!("Content-Type: image/png; name=\"b.png\"\r\nContent-Disposition: inline; filename=\"b.png\"\r\n{b64}"),
        ]), 2),
    ];
    for (label, raw, expect) in &cases {
        assert_parity(label, raw, *expect);
    }
}

#[test]
fn attachments_inside_related_and_nested_alternative_are_listed() {
    let raw = "From: a@example.org\r\nTo: b@example.org\r\nSubject: t\r\nMIME-Version: 1.0\r\n\
Content-Type: multipart/mixed; boundary=M\r\n\r\n--M\r\nContent-Type: multipart/alternative; boundary=A\r\n\r\n\
--A\r\nContent-Type: text/plain\r\n\r\nhi\r\n--A\r\nContent-Type: multipart/related; boundary=R\r\n\r\n\
--R\r\nContent-Type: text/html\r\n\r\n<img src=\"cid:l@y\">\r\n\
--R\r\nContent-Type: image/png; name=\"l.png\"\r\nContent-ID: <l@y>\r\nContent-Transfer-Encoding: base64\r\n\r\nQUJD\r\n\
--R\r\nContent-Type: application/pdf; name=\"in-related.pdf\"\r\nContent-Disposition: attachment; filename=\"in-related.pdf\"\r\nContent-Transfer-Encoding: base64\r\n\r\nQUJD\r\n\
--R--\r\n--A--\r\n--M\r\nContent-Type: application/pdf; name=\"outer.pdf\"\r\nContent-Disposition: attachment; filename=\"outer.pdf\"\r\nContent-Transfer-Encoding: base64\r\n\r\nQUJD\r\n--M--\r\n";
    assert_parity("related+mixed", raw.as_bytes(), 3);
    let e = parse(raw.as_bytes()).unwrap().email;
    assert!(e.has_attachment, "real files still raise hasAttachment");
}
