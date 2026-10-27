//! Byte-preserving removal of top-level mail-client identification headers.
use std::{borrow::Cow, ops::Range};

const MAX_HEADERS: usize = 128 * 1024;
const TARGETS: [&[u8]; 3] = [b"x-mailer", b"user-agent", b"x-mimeole"];
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HeaderPrivacyError {
    #[error("outgoing header privacy: invalid or oversized top-level header block")]
    InvalidHeaders,
    #[error("outgoing header privacy conflicts with pre-signed DKIM/ARC headers")]
    SignedHeaders,
}
struct Field<'a> {
    name: &'a [u8],
    range: Range<usize>,
    value: Vec<u8>,
}
fn trim(value: &[u8]) -> &[u8] {
    value.trim_ascii()
}
fn target(name: &[u8]) -> bool {
    TARGETS
        .iter()
        .any(|candidate| name.eq_ignore_ascii_case(candidate))
}
fn signed_target(value: &[u8], removed: &[&[u8]]) -> Result<bool, HeaderPrivacyError> {
    let mut headers = None;
    for attribute in value.split(|byte| *byte == b';') {
        let Some(equal) = attribute.iter().position(|byte| *byte == b'=') else {
            continue;
        };
        if trim(&attribute[..equal]).eq_ignore_ascii_case(b"h") {
            if headers.is_some() {
                return Err(HeaderPrivacyError::SignedHeaders);
            }
            headers = Some(trim(&attribute[equal + 1..]));
        }
    }
    let headers = headers
        .filter(|value| !value.is_empty())
        .ok_or(HeaderPrivacyError::SignedHeaders)?;
    let mut covered = false;
    for name in headers.split(|byte| *byte == b':') {
        let name = trim(name);
        if name.is_empty()
            || !name
                .iter()
                .all(|byte| byte.is_ascii_graphic() && *byte != b':')
        {
            return Err(HeaderPrivacyError::SignedHeaders);
        }
        covered |= removed
            .iter()
            .any(|removed| name.eq_ignore_ascii_case(removed));
    }
    Ok(covered)
}

/// No MIME parsing or reserialization: attachments, nested messages and signed or
/// encrypted bodies remain byte-identical. A signature covering a removed header
/// is refused; unrelated DKIM/ARC signed-header lists are left intact.
pub fn strip_mailer_headers(raw: &[u8]) -> Result<Cow<'_, [u8]>, HeaderPrivacyError> {
    let bounded = &raw[..raw.len().min(MAX_HEADERS)];
    let mut fields: Vec<Field<'_>> = Vec::new();
    let mut offset = 0;
    loop {
        let length = bounded
            .get(offset..)
            .and_then(|remaining| remaining.iter().position(|byte| *byte == b'\n'))
            .ok_or(HeaderPrivacyError::InvalidHeaders)?
            + 1;
        let end = offset + length;
        let line = bounded[offset..end - 1]
            .strip_suffix(b"\r")
            .unwrap_or(&bounded[offset..end - 1]);
        if line.is_empty() {
            break;
        }
        if line.contains(&b'\r') || line.contains(&0) {
            return Err(HeaderPrivacyError::InvalidHeaders);
        }
        if line.starts_with(b" ") || line.starts_with(b"\t") {
            let previous = fields
                .last_mut()
                .ok_or(HeaderPrivacyError::InvalidHeaders)?;
            previous.range.end = end;
            previous.value.push(b' ');
            previous.value.extend_from_slice(trim(line));
        } else {
            let colon = line
                .iter()
                .position(|byte| *byte == b':')
                .ok_or(HeaderPrivacyError::InvalidHeaders)?;
            let name = &line[..colon];
            if name.is_empty()
                || !name
                    .iter()
                    .all(|byte| byte.is_ascii_graphic() && *byte != b':')
            {
                return Err(HeaderPrivacyError::InvalidHeaders);
            }
            fields.push(Field {
                name,
                range: offset..end,
                value: line[colon + 1..].to_vec(),
            });
        }
        offset = end;
    }
    if !fields.iter().any(|field| target(field.name)) {
        return Ok(Cow::Borrowed(raw));
    }
    let removed = fields
        .iter()
        .filter(|field| target(field.name))
        .map(|field| field.name)
        .collect::<Vec<_>>();
    for field in &fields {
        if (field.name.eq_ignore_ascii_case(b"dkim-signature")
            || field.name.eq_ignore_ascii_case(b"arc-message-signature"))
            && signed_target(&field.value, &removed)?
        {
            return Err(HeaderPrivacyError::SignedHeaders);
        }
    }
    let mut clean = Vec::with_capacity(raw.len());
    let mut copied = 0;
    for field in fields.iter().filter(|field| target(field.name)) {
        clean.extend_from_slice(&raw[copied..field.range.start]);
        copied = field.range.end;
    }
    clean.extend_from_slice(&raw[copied..]);
    Ok(Cow::Owned(clean))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn folded_headers_only_and_binary_nested_message_unchanged() {
        let body = b"--part\r\nContent-Type: message/rfc822\r\n\r\nX-Mailer: original\r\nUser-Agent: original\r\n\r\n\x00\xffbody\r\n--part--\r\n";
        let mut raw = b"From: sender@example.test\r\nX-mAiLeR: client\r\n\tversion\r\nUser-Agent: agent\r\nX-MimeOLE: engine\r\nMessage-ID: <keep>\r\nContent-Type: multipart/mixed; boundary=part\r\n\r\n".to_vec();
        raw.extend_from_slice(body);
        let stripped = strip_mailer_headers(&raw).unwrap();
        let mut expected=b"From: sender@example.test\r\nMessage-ID: <keep>\r\nContent-Type: multipart/mixed; boundary=part\r\n\r\n".to_vec();
        expected.extend_from_slice(body);
        assert_eq!(stripped.as_ref(), expected);
    }
    #[test]
    fn signed_and_encrypted_mime_payloads_are_identical() {
        for kind in [
            "multipart/signed",
            "multipart/encrypted",
            "application/pkcs7-mime",
        ] {
            let head = format!("Content-Type: {kind}\r\n");
            let raw =
                format!("X-Mailer: private\r\n{head}\r\nX-Mailer: protected body\r\n--signature--");
            assert_eq!(
                strip_mailer_headers(raw.as_bytes()).unwrap().as_ref(),
                format!("{head}\r\nX-Mailer: protected body\r\n--signature--").as_bytes()
            );
        }
    }
    #[test]
    fn signed_identification_headers_refuse_but_unrelated_signatures_survive() {
        for signature in ["DKIM-Signature", "ARC-Message-Signature"] {
            let raw = format!(
                "{signature}: v=1; h=From:\r\n Subject:X-Mailer; b=signature\r\nX-Mailer: client\r\n\r\nbody"
            );
            assert_eq!(
                strip_mailer_headers(raw.as_bytes()).unwrap_err(),
                HeaderPrivacyError::SignedHeaders
            );
            let raw = format!(
                "{signature}: v=1; h=From:Subject; b=signature\r\nX-Mailer: client\r\n\r\nbody"
            );
            assert_eq!(
                strip_mailer_headers(raw.as_bytes()).unwrap().as_ref(),
                format!("{signature}: v=1; h=From:Subject; b=signature\r\n\r\nbody").as_bytes()
            );
        }
    }
    #[test]
    fn lf_and_duplicate_fields_preserve_other_bytes() {
        assert_eq!(
            strip_mailer_headers(
                b"X-Mailer: a\nSubject: keep\nX-Mailer: b\n continued\n\nUser-Agent: body"
            )
            .unwrap()
            .as_ref(),
            b"Subject: keep\n\nUser-Agent: body"
        );
        assert!(matches!(
            strip_mailer_headers(b"Subject: keep\r\n\r\nbody").unwrap(),
            Cow::Borrowed(_)
        ));
    }
    #[test]
    fn malformed_signature_lists_refuse_but_unrelated_oversigning_survives() {
        for value in [
            "v=1; b=signature",
            "h=; b=signature",
            "h=From; h=Subject; b=signature",
            "h=From::Subject; b=signature",
        ] {
            let raw = format!("DKIM-Signature: {value}\r\nUser-Agent: private\r\n\r\nbody");
            assert_eq!(
                strip_mailer_headers(raw.as_bytes()).unwrap_err(),
                HeaderPrivacyError::SignedHeaders
            );
        }
        let raw =
            b"DKIM-Signature: h=From:X-Mailer; b=signature\r\nUser-Agent: private\r\n\r\nbody";
        assert_eq!(
            strip_mailer_headers(raw).unwrap().as_ref(),
            b"DKIM-Signature: h=From:X-Mailer; b=signature\r\n\r\nbody"
        );
    }
    #[test]
    fn real_mime_builder_output_preserves_attachment_and_standard_headers() {
        let raw = crate::build(&crate::ComposeRequest {
            subject: Some("Keep".into()),
            text_body: Some("Body".into()),
            headers: vec![
                ("X-Mailer".into(), "Private client".into()),
                ("User-Agent".into(), "Private agent".into()),
            ],
            attachments: vec![crate::Attachment {
                filename: "original.eml".into(),
                content_type: "message/rfc822".into(),
                bytes: b"X-Mailer: nested\r\n\r\noriginal".to_vec(),
            }],
            ..Default::default()
        })
        .unwrap();
        let clean = strip_mailer_headers(&raw).unwrap();
        let body = |bytes: &[u8]| {
            bytes
                .windows(4)
                .position(|value| value == b"\r\n\r\n")
                .unwrap()
                + 4
        };
        assert_eq!(&clean[body(&clean)..], &raw[body(&raw)..]);
        let header = String::from_utf8_lossy(&clean[..body(&clean)]).to_ascii_lowercase();
        assert!(!header.contains("x-mailer:") && !header.contains("user-agent:"));
        assert!(header.contains("subject: keep"));
    }
    #[test]
    fn malformed_or_unbounded_headers_refuse() {
        for raw in [
            b" folded\r\n\r\n".as_slice(),
            b"X-Mailer: no separator".as_slice(),
        ] {
            assert_eq!(
                strip_mailer_headers(raw).unwrap_err(),
                HeaderPrivacyError::InvalidHeaders
            );
        }
        assert_eq!(
            strip_mailer_headers(&vec![b'x'; MAX_HEADERS + 1]).unwrap_err(),
            HeaderPrivacyError::InvalidHeaders
        );
    }
}
