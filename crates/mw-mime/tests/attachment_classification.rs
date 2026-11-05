//! Which parts reach the reader's attachment list (`attachments` / `hasAttachment`).

use mw_mime::parse;

fn msg(part_headers: &str) -> Vec<u8> {
    format!(
        "From: a@example.org\r\nTo: b@example.org\r\nSubject: t\r\nMIME-Version: 1.0\r\n\
Content-Type: multipart/mixed; boundary=B\r\n\r\n--B\r\nContent-Type: text/plain\r\n\r\nhi\r\n\
--B\r\n{part_headers}\r\n\r\nQUJD\r\n--B--\r\n"
    )
    .into_bytes()
}

fn names(raw: &[u8]) -> (bool, Vec<Option<String>>) {
    let e = parse(raw).unwrap().email;
    (e.has_attachment, e.attachments.iter().map(|a| a.name.clone()).collect())
}

#[test]
fn inline_disposition_with_filename_is_an_attachment() {
    let (has, n) = names(&msg(
        "Content-Type: application/pdf; name=\"a.pdf\"\r\nContent-Disposition: inline; filename=\"a.pdf\"\r\nContent-Transfer-Encoding: base64",
    ));
    assert!(has);
    assert_eq!(n, vec![Some("a.pdf".to_string())]);
}

#[test]
fn explicit_attachment_with_content_id_is_an_attachment() {
    let (has, n) = names(&msg(
        "Content-Type: application/pdf; name=\"b.pdf\"\r\nContent-Disposition: attachment; filename=\"b.pdf\"\r\nContent-ID: <x@y>\r\nContent-Transfer-Encoding: base64",
    ));
    assert!(has);
    assert_eq!(n.len(), 1);
}

#[test]
fn cid_inline_image_stays_a_body_resource() {
    let (has, n) = names(&msg(
        "Content-Type: image/png; name=\"l.png\"\r\nContent-Disposition: inline; filename=\"l.png\"\r\nContent-ID: <l@y>\r\nContent-Transfer-Encoding: base64",
    ));
    assert!(!has);
    assert!(n.is_empty());
}

#[test]
fn nameless_inline_part_without_cid_is_not_listed() {
    let (has, n) = names(&msg(
        "Content-Type: image/png\r\nContent-Disposition: inline\r\nContent-Transfer-Encoding: base64",
    ));
    assert!(!has);
    assert!(n.is_empty());
}
