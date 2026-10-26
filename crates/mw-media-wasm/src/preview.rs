//! Bounded attachment previews. This module executes only in the media guest in
//! production; returned HTML MUST be sanitized by mw-render before display.
use base64::Engine as _;
use mail_parser::{Message, MessageParser, MimeHeaders, PartType};
use serde::Serialize;

const INPUT_MAX: usize = 4 * 1024 * 1024;
const BODY_MAX: usize = 1024 * 1024;
const PART_MAX: usize = 64;
const PROPERTY_MAX: usize = 4096;
const MESSAGE_IID: &[u8; 16] = b"\x07\x03\x02\x00\x00\x00\x00\x00\xc0\x00\x00\x00\x00\x00\x00\x46";

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    format: &'static str,
    subject: Option<String>,
    from: Option<String>,
    to: Option<String>,
    date: Option<String>,
    text: String,
    html: String,
    attachments: Vec<Attachment>,
    warnings: Vec<String>,
    #[serde(skip)]
    attachment_bytes: usize,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Attachment {
    name: String,
    mime: String,
    size: usize,
    data_base64: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_id: Option<String>,
    kind: &'static str,
}
impl Preview {
    fn warn(&mut self, message: &str) {
        if self.warnings.len() < 16 && !self.warnings.iter().any(|w| w == message) {
            self.warnings.push(message.into());
        }
    }
    fn attach(
        &mut self,
        name: Option<&str>,
        mime: Option<&str>,
        cid: Option<&str>,
        data: &[u8],
        kind: &'static str,
    ) -> Result<(), String> {
        self.attachment_bytes = self
            .attachment_bytes
            .checked_add(data.len())
            .ok_or("attachment length overflow")?;
        if self.attachments.len() >= PART_MAX || self.attachment_bytes > INPUT_MAX {
            return Err("attachment output exceeds limits".into());
        }
        let name = safe_name(name.unwrap_or(""), self.attachments.len() + 1);
        let mime = mime
            .filter(|m| {
                m.len() <= 128
                    && m.contains('/')
                    && m.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"/!#$&^_.+-".contains(&b))
            })
            .unwrap_or("application/octet-stream")
            .to_ascii_lowercase();
        self.attachments.push(Attachment {
            name,
            mime,
            size: data.len(),
            data_base64: base64::engine::general_purpose::STANDARD.encode(data),
            content_id: cid.map(meta),
            kind,
        });
        Ok(())
    }
}
fn meta(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control())
        .take(4096)
        .collect()
}
fn safe_name(name: &str, n: usize) -> String {
    let name: String = name
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | ':' | '"' | '<' | '>' | '|' | '?' | '*') {
                '_'
            } else {
                c
            }
        })
        .take(240)
        .collect();
    let name = name.trim().trim_matches('.');
    if name.is_empty() {
        format!("attachment-{n}.bin")
    } else {
        name.into()
    }
}
fn checked_body(value: String) -> Result<String, String> {
    if value.len() > BODY_MAX {
        Err("message body exceeds limit".into())
    } else {
        Ok(value)
    }
}
fn check_message(message: &Message<'_>, depth: usize, parts: &mut usize) -> Result<(), String> {
    *parts = parts
        .checked_add(message.parts.len())
        .ok_or("MIME count overflow")?;
    if depth > 3 || *parts > 256 {
        return Err("MIME nesting or part count exceeds limit".into());
    }
    for part in &message.parts {
        if let PartType::Message(inner) = &part.body {
            check_message(inner, depth + 1, parts)?;
        }
    }
    Ok(())
}
pub fn parse_eml(bytes: &[u8]) -> Result<Preview, String> {
    if bytes.len() > INPUT_MAX {
        return Err("message exceeds size limit".into());
    }
    let message = MessageParser::default()
        .parse(bytes)
        .ok_or("invalid RFC822 message")?;
    if message.parts.first().is_none_or(|p| p.headers.is_empty()) {
        return Err("message has no headers".into());
    }
    check_message(&message, 0, &mut 0)?;
    let addresses = |address: Option<&mail_parser::Address<'_>>| {
        address.map(|a| {
            meta(
                &a.iter()
                    .map(|a| match (a.name.as_deref(), a.address.as_deref()) {
                        (Some(name), Some(email)) => format!("{name} <{email}>"),
                        (_, Some(email)) => email.into(),
                        (Some(name), _) => name.into(),
                        _ => String::new(),
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        })
    };
    let mut out = Preview {
        format: "eml",
        subject: message.subject().map(meta),
        from: addresses(message.from()),
        to: addresses(message.to()),
        date: message.date().map(|d| d.to_rfc3339()),
        ..Preview::default()
    };
    out.text = checked_body(
        message
            .body_text(0)
            .map(|b| b.into_owned())
            .unwrap_or_default(),
    )?;
    out.html = checked_body(
        message
            .body_html(0)
            .map(|b| b.into_owned())
            .unwrap_or_default(),
    )?;
    for part in message.attachments() {
        if part.is_encoding_problem {
            out.warn("An attachment has malformed transfer encoding.");
        }
        let mime = part.content_type().map(|c| {
            format!(
                "{}/{}",
                c.c_type,
                c.c_subtype.as_deref().unwrap_or("octet-stream")
            )
        });
        let kind = if matches!(part.body, PartType::Message(_)) {
            "message"
        } else {
            "file"
        };
        out.attach(
            part.attachment_name(),
            mime.as_deref(),
            part.content_id(),
            part.contents(),
            kind,
        )?;
    }
    Ok(out)
}

struct Read<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Read<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.pos.checked_add(n).ok_or("TNEF length overflow")?;
        let b = self
            .bytes
            .get(self.pos..end)
            .ok_or("truncated TNEF record")?;
        self.pos = end;
        Ok(b)
    }
    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn padded(&mut self, n: usize) -> Result<&'a [u8], String> {
        let b = self.take(n)?;
        self.take((4 - n % 4) % 4)?;
        Ok(b)
    }
    fn left(&self) -> usize {
        self.bytes.len() - self.pos
    }
}
#[derive(Default)]
struct PendingAttachment {
    name: Option<String>,
    long_name: Option<String>,
    mime: Option<String>,
    cid: Option<String>,
    data: Option<Vec<u8>>,
    kind: Option<&'static str>,
}
impl PendingAttachment {
    fn finish(self, out: &mut Preview) -> Result<(), String> {
        if let Some(data) = self.data {
            out.attach(
                self.long_name.as_deref().or(self.name.as_deref()),
                self.mime.as_deref(),
                self.cid.as_deref(),
                &data,
                self.kind.unwrap_or("file"),
            )?;
        } else {
            out.warn("An attachment has no embedded data; external references are not fetched.");
        }
        Ok(())
    }
}
fn decode(bytes: &[u8], unicode: bool, codepage: u32) -> Result<String, String> {
    let text = if unicode {
        if !bytes.len().is_multiple_of(2) {
            return Err("invalid UTF-16 property length".into());
        }
        String::from_utf16_lossy(
            &bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|p| u16::from_le_bytes([p[0], p[1]]))
                .collect::<Vec<_>>(),
        )
    } else {
        let label = match codepage {
            65001 => "utf-8".into(),
            1250..=1258 => format!("windows-{codepage}"),
            932 => "shift_jis".into(),
            936 => "gbk".into(),
            949 => "euc-kr".into(),
            950 => "big5".into(),
            28591 => "windows-1252".into(),
            _ => "windows-1252".into(),
        };
        encoding_rs::Encoding::for_label(label.as_bytes())
            .unwrap_or(encoding_rs::WINDOWS_1252)
            .decode(bytes)
            .0
            .into_owned()
    };
    Ok(text.trim_end_matches('\0').to_string())
}
fn apply_property(
    id: u16,
    ty: u16,
    data: &[u8],
    out: &mut Preview,
    attachment: &mut Option<PendingAttachment>,
    codepage: &mut u32,
) -> Result<(), String> {
    if id == 0x3fde && ty == 3 && data.len() == 4 {
        *codepage = u32::from_le_bytes(data.try_into().unwrap());
    }
    if let Some(a) = attachment {
        match id {
            0x3704 | 0x3707 | 0x370e | 0x3712 if matches!(ty, 0x1e | 0x1f) => {
                let value = decode(data, ty == 0x1f, *codepage)?;
                match id {
                    0x3704 => a.name = Some(value),
                    0x3707 => a.long_name = Some(value),
                    0x370e => a.mime = Some(value),
                    _ => a.cid = Some(value),
                }
            }
            0x3701 if ty == 0x102 => a.data = Some(data.to_vec()),
            0x3701 if ty == 0x0d => {
                if data.len() < 16 {
                    return Err("truncated MAPI object identifier".into());
                }
                if &data[..16] == MESSAGE_IID && data.get(16..20) == Some(b"\x78\x9f\x3e\x22") {
                    a.data = Some(data[16..].to_vec());
                    a.kind = Some("message");
                    a.mime = Some("application/ms-tnef".into());
                } else {
                    a.data = Some(data.to_vec());
                    out.warn("An OLE attachment is available as opaque data only.");
                }
            }
            _ => {}
        }
    } else {
        match id {
            0x0037 | 0x0c1f | 0x0e04 | 0x1000 if matches!(ty, 0x1e | 0x1f) => {
                let value = decode(data, ty == 0x1f, *codepage)?;
                match id {
                    0x0037 => out.subject = Some(meta(&value)),
                    0x0c1f => out.from = Some(meta(&value)),
                    0x0e04 => out.to = Some(meta(&value)),
                    _ => out.text = checked_body(value)?,
                }
            }
            0x1013 if matches!(ty, 0x102 | 0x1e | 0x1f) => {
                out.html = checked_body(decode(data, ty == 0x1f, *codepage)?)?
            }
            0x1009 => out.warn(
                "Compressed RTF body is not rendered; available plain text or HTML is shown.",
            ),
            _ => {}
        }
    }
    Ok(())
}
/// MS-OXTNEF 2.1.3.4: named properties, scalar and multivalue property data.
fn properties(
    bytes: &[u8],
    out: &mut Preview,
    attachment: &mut Option<PendingAttachment>,
    codepage: &mut u32,
) -> Result<(), String> {
    let mut r = Read::new(bytes);
    let count = r.u32()? as usize;
    if count > PROPERTY_MAX {
        return Err("too many MAPI properties".into());
    }
    for _ in 0..count {
        let tag = r.u32()?;
        let ty = tag as u16;
        let id = (tag >> 16) as u16;
        if id >= 0x8000 {
            r.take(16)?;
            match r.u32()? {
                0 => {
                    r.take(4)?;
                }
                1 => {
                    let n = r.u32()? as usize;
                    if n > 8192 {
                        return Err("MAPI property name exceeds limit".into());
                    }
                    r.padded(n)?;
                }
                _ => return Err("invalid named MAPI property kind".into()),
            }
        }
        let base = ty & !0x1000;
        let multi = ty & 0x1000 != 0;
        let variable = matches!(base, 0x1e | 0x1f | 0x102 | 0x0d);
        let values = if multi || variable {
            r.u32()? as usize
        } else {
            1
        };
        if values > PROPERTY_MAX || (!multi && variable && values != 1) {
            return Err("invalid MAPI value count".into());
        }
        for _ in 0..values {
            let data = match base {
                0 | 1 => r.take(0)?,
                2 | 3 | 4 | 0x0a | 0x0b => r.take(4)?,
                5 | 6 | 7 | 0x14 | 0x40 => r.take(8)?,
                0x48 => r.take(16)?,
                0x1e | 0x1f | 0x102 | 0x0d => {
                    let n = r.u32()? as usize;
                    r.padded(n)?
                }
                _ => return Err("unsupported MAPI property type".into()),
            };
            // Named and multivalue metadata cannot masquerade as canonical fields.
            if id < 0x8000 && !multi {
                apply_property(id, base, data, out, attachment, codepage)?;
            }
        }
    }
    if r.left() != 0 {
        return Err("trailing MAPI property data".into());
    }
    Ok(())
}
pub fn parse_tnef(bytes: &[u8]) -> Result<Preview, String> {
    if bytes.len() > INPUT_MAX {
        return Err("TNEF exceeds size limit".into());
    }
    let mut r = Read::new(bytes);
    if r.u32()? != 0x223e9f78 {
        return Err("not a TNEF stream".into());
    }
    r.take(2)?;
    let mut out = Preview {
        format: "tnef",
        ..Preview::default()
    };
    let mut attachment: Option<PendingAttachment> = None;
    let mut codepage = 1252;
    let mut records = 0;
    while r.left() != 0 {
        records += 1;
        if records > PROPERTY_MAX {
            return Err("too many TNEF attributes".into());
        }
        let level = r.take(1)?[0];
        if !matches!(level, 1 | 2) {
            return Err("invalid TNEF attribute level".into());
        }
        let tag = r.u32()?;
        let n = r.u32()? as usize;
        let data = r.take(n)?;
        let checksum = r.u16()?;
        if data.iter().fold(0u16, |sum, b| sum.wrapping_add(*b as u16)) != checksum {
            return Err("TNEF checksum mismatch".into());
        }
        let id = tag as u16;
        match (level, id) {
            (1, 0x9007) => {
                if data.len() < 4 {
                    return Err("invalid TNEF codepage".into());
                }
                codepage = u32::from_le_bytes(data[..4].try_into().unwrap());
            }
            (1, 0x8004) => out.subject = Some(meta(&decode(data, false, codepage)?)),
            (1, 0x800c) => out.text = checked_body(decode(data, false, codepage)?)?,
            (1, 0x9003) => properties(data, &mut out, &mut None, &mut codepage)?,
            (2, 0x9002) => {
                if let Some(a) = attachment.take() {
                    a.finish(&mut out)?;
                }
                if out.attachments.len() >= PART_MAX {
                    return Err("too many TNEF attachments".into());
                }
                attachment = Some(PendingAttachment::default());
            }
            (2, 0x800f) => {
                attachment
                    .as_mut()
                    .ok_or("attachment data before boundary")?
                    .data = Some(data.to_vec())
            }
            (2, 0x8010) => {
                attachment
                    .as_mut()
                    .ok_or("attachment name before boundary")?
                    .name = Some(decode(data, false, codepage)?)
            }
            (2, 0x9005) => {
                if attachment.is_none() {
                    return Err("attachment properties before boundary".into());
                }
                properties(data, &mut out, &mut attachment, &mut codepage)?;
            }
            _ => {}
        }
    }
    if let Some(a) = attachment {
        a.finish(&mut out)?;
    }
    if records == 0 {
        return Err("empty TNEF stream".into());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_mimekit_tnef_corpus() {
        for (name, bytes, expected) in [
            (
                "one-file",
                include_bytes!("../fixtures/tnef/one-file.tnef").as_slice(),
                vec!["AUTHORS"],
            ),
            (
                "two-files",
                include_bytes!("../fixtures/tnef/two-files.tnef").as_slice(),
                vec!["AUTHORS", "README"],
            ),
            (
                "data-before-name",
                include_bytes!("../fixtures/tnef/data-before-name.tnef").as_slice(),
                vec!["AUTOEXEC.BAT", "CONFIG.SYS", "boot.ini"],
            ),
            (
                "unicode-mapi-attr",
                include_bytes!("../fixtures/tnef/unicode-mapi-attr.tnef").as_slice(),
                vec!["example.dat"],
            ),
            (
                "unicode-mapi-attr-name",
                include_bytes!("../fixtures/tnef/unicode-mapi-attr-name.tnef").as_slice(),
                vec![
                    "spaconsole2.cfg",
                    "image001.png",
                    "image002.png",
                    "image003.png",
                ],
            ),
            (
                "multi-value-attribute",
                include_bytes!("../fixtures/tnef/multi-value-attribute.tnef").as_slice(),
                vec!["208225__5_seconds__Voice_Mail.mp3"],
            ),
            (
                "attachments",
                include_bytes!("../fixtures/tnef/attachments.tnef").as_slice(),
                vec!["bookmark.htm", "zappa_av1.jpg"],
            ),
            (
                "MAPI_ATTACH_DATA_OBJ",
                include_bytes!("../fixtures/tnef/MAPI_ATTACH_DATA_OBJ.tnef").as_slice(),
                vec![
                    "VIA_Nytt_1402.doc",
                    "VIA_Nytt_1402.pdf",
                    "VIA_Nytt_14021.htm",
                ],
            ),
        ] {
            let preview = parse_tnef(bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
            let mut names: Vec<_> = preview
                .attachments
                .iter()
                .map(|a| a.name.as_str())
                .collect();
            let mut expected = expected;
            names.sort();
            expected.sort();
            assert_eq!(names, expected, "{name}");
            for attachment in preview.attachments {
                let expected_bytes = std::fs::read(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("fixtures/tnef")
                        .join(name)
                        .join(&attachment.name),
                )
                .unwrap();
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(&attachment.data_base64)
                    .unwrap();
                let text_fixture = matches!(
                    std::path::Path::new(&attachment.name)
                        .extension()
                        .and_then(|e| e.to_str()),
                    Some("htm" | "ini" | "cfg" | "BAT" | "SYS")
                ) || matches!(attachment.name.as_str(), "AUTHORS" | "README");
                let normalize = |bytes: &[u8]| {
                    bytes
                        .iter()
                        .enumerate()
                        .filter_map(|(i, b)| {
                            if *b == b'\r' && bytes.get(i + 1) == Some(&b'\n') {
                                None
                            } else {
                                Some(*b)
                            }
                        })
                        .collect::<Vec<_>>()
                };
                let equal = if text_fixture {
                    normalize(&decoded) == normalize(&expected_bytes)
                } else {
                    decoded == expected_bytes
                };
                assert!(
                    equal,
                    "{name}/{} expected extraction differs ({} vs {} bytes)",
                    attachment.name,
                    decoded.len(),
                    expected_bytes.len()
                );
                assert_eq!(
                    base64::engine::general_purpose::STANDARD
                        .decode(attachment.data_base64)
                        .unwrap()
                        .len(),
                    attachment.size
                );
            }
        }
    }
    fn attribute(level: u8, tag: u32, data: &[u8]) -> Vec<u8> {
        let mut bytes = vec![level];
        bytes.extend(tag.to_le_bytes());
        bytes.extend((data.len() as u32).to_le_bytes());
        bytes.extend(data);
        bytes.extend(
            data.iter()
                .fold(0u16, |a, b| a.wrapping_add(*b as u16))
                .to_le_bytes(),
        );
        bytes
    }
    fn stream(attribute: Vec<u8>) -> Vec<u8> {
        let mut bytes = vec![0x78, 0x9f, 0x3e, 0x22, 0, 0];
        bytes.extend(attribute);
        bytes
    }
    #[test]
    fn mapi_limits_unknown_types_and_nonzero_padding() {
        let excessive = stream(attribute(1, 0x00069003, &u32::MAX.to_le_bytes()));
        assert!(parse_tnef(&excessive).is_err());
        let mut properties = 1u32.to_le_bytes().to_vec();
        properties.extend(0x003700ffu32.to_le_bytes());
        assert!(parse_tnef(&stream(attribute(1, 0x00069003, &properties))).is_err());
        let mut properties = 1u32.to_le_bytes().to_vec();
        properties.extend(0x0037001fu32.to_le_bytes());
        properties.extend(1u32.to_le_bytes());
        properties.extend(6u32.to_le_bytes());
        properties.extend(b"H\0i\0\0\0");
        properties.extend([0xff, 0xff]);
        let p = parse_tnef(&stream(attribute(1, 0x00069003, &properties))).unwrap();
        assert_eq!(p.subject.as_deref(), Some("Hi"));
        properties[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(parse_tnef(&stream(attribute(1, 0x00069003, &properties))).is_err());
    }
    #[test]
    fn attachment_count_and_body_limits_are_enforced() {
        let mut bytes = vec![0x78, 0x9f, 0x3e, 0x22, 0, 0];
        for _ in 0..65 {
            bytes.extend(attribute(2, 0x00069002, &[0; 14]));
            bytes.extend(attribute(2, 0x0006800f, b"x"));
        }
        assert!(parse_tnef(&bytes).is_err());
        assert!(parse_tnef(&stream(attribute(1, 0x0002800c, &vec![b'a'; BODY_MAX + 1]))).is_err());
        assert_eq!(
            safe_name("../../unsafe\\name.txt", 1),
            "_.._unsafe_name.txt"
        );
    }

    #[test]
    fn embedded_message_iid_keeps_message_bytes_and_type() {
        let nested = stream(attribute(1, 0x00018004, b"Nested\0"));
        let mut object = MESSAGE_IID.to_vec();
        object.extend(&nested);
        let mut properties = 1u32.to_le_bytes().to_vec();
        properties.extend(0x3701000du32.to_le_bytes());
        properties.extend(1u32.to_le_bytes());
        properties.extend((object.len() as u32).to_le_bytes());
        properties.extend(&object);
        properties.resize(properties.len() + (4 - object.len() % 4) % 4, 0);
        let mut bytes = stream(attribute(2, 0x00069002, &[0; 14]));
        bytes.extend(attribute(2, 0x00069005, &properties));
        let p = parse_tnef(&bytes).unwrap();
        assert_eq!(p.attachments[0].kind, "message");
        assert_eq!(p.attachments[0].mime, "application/ms-tnef");
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&p.attachments[0].data_base64)
            .unwrap();
        assert_eq!(decoded, nested);
        assert_eq!(
            parse_tnef(&decoded).unwrap().subject.as_deref(),
            Some("Nested")
        );
    }

    #[test]
    fn rejects_truncation_checksums_and_oversize() {
        let bytes = include_bytes!("../fixtures/tnef/one-file.tnef");
        for length in 0..bytes.len() {
            let result = std::panic::catch_unwind(|| parse_tnef(&bytes[..length]));
            assert!(result.is_ok(), "panic at {length}");
        }
        let mut corrupt = bytes.to_vec();
        corrupt[bytes.len() - 1] ^= 1;
        assert!(parse_tnef(&corrupt).is_err());
        assert!(parse_tnef(&vec![0; INPUT_MAX + 1]).is_err());
    }
    #[test]
    fn eml_decodes_headers_bodies_and_nested_message_bytes() {
        let raw=b"From: =?UTF-8?B?Sm9zw6k=?= <jose@example.test>\r\nTo: user@example.test\r\nSubject: =?UTF-8?B?UmVzdW3DqQ==?=\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=b\r\n\r\n--b\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<script>unsafe()</script><p>Hello</p>\r\n--b\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"../file.bin\"\r\nContent-Transfer-Encoding: base64\r\n\r\nAAECAw==\r\n--b\r\nContent-Type: message/rfc822\r\nContent-Disposition: attachment; filename=inner.eml\r\n\r\nFrom: nested@example.test\r\nSubject: Inner\r\n\r\nNested body\r\n--b--\r\n";
        let p = parse_eml(raw).unwrap();
        assert_eq!(p.subject.as_deref(), Some("Resumé"));
        assert!(p.from.unwrap().contains("José"));
        assert!(p.html.contains("<script>"), "sanitization belongs to host");
        assert_eq!(p.attachments.len(), 2);
        assert_eq!(p.attachments[0].name, "_file.bin");
        assert_eq!(p.attachments[0].size, 4);
        assert_eq!(p.attachments[1].kind, "message");
        assert!(base64::engine::general_purpose::STANDARD
            .decode(&p.attachments[1].data_base64)
            .unwrap()
            .starts_with(b"From: nested"));
    }
}
