//! MS-OXMSG `.msg` export (plan §3 e5, §1.7, SPEC §10.6).
//!
//! An `.msg` file is a CFB (OLE2 Compound-File-Binary) container carrying MAPI
//! properties. This module writes that container with the [`cfb`] crate and
//! layers our own minimal **MS-OXMSG** property encoder on top. **Scope floor
//! (plan §0.6/§1.7):** faithful body + attachments + headers. Deep write
//! fidelity — embedded OLE objects, custom named properties (`__nameid`) — is
//! explicitly **out** (documented best-effort, §28.8): we emit the standard
//! numbered-property streams only, no named-property map.
//!
//! # Hostile-parse boundary (plan §1.7, SPEC §7.5)
//! The **write** path parses trusted RFC 5322 bytes through `mail-parser` (the
//! same parser the rest of the crate already trusts) and never parses a
//! compound file. The **read** path ([`read_msg`]) parses an *untrusted* CFB
//! container and MUST be treated as hostile input. It is written to never panic
//! (used by the CFB fuzz target), but per §7.5 attacker-supplied `.msg`/`.oft`
//! bytes should still be decoded inside the render jail (`mw-render`), not in a
//! privileged export path. `mw-render` has no CFB job frame today — see the
//! `SEAM(e14/e16)` note below — so until that frame exists, callers feeding
//! untrusted bytes must route them through the jail themselves. Only trusted /
//! test-authored fixtures go through [`read_msg`] in-process.
//!
//! SEAM(e14/e16): add a `Cfb`/`Msg` variant to `mw_render::Job` so untrusted
//! `.msg`/`.oft` import runs in the disposable child (no net, no secrets), and
//! route the web "import .oft template" action through it. The parser here is
//! panic-free and size-limited so it is safe to lift into that child unchanged.

use std::io::{Cursor, Read, Write};

use mail_parser::{Address, Message, MessageParser, MimeHeaders, PartType};

use crate::{ExportError, Result};

/// `IPM.Note` — the message class for an ordinary mail item.
pub(crate) const MESSAGE_CLASS_NOTE: &str = "IPM.Note";

/// Upper bound on an untrusted CFB we will attempt to read (render-jail parser
/// resource limit, mirrors `mw_render::MAX_INPUT_BYTES`).
const MAX_READ_BYTES: usize = 4 * 1024 * 1024;

// --- MAPI property types (MS-OXCDATA §2.11.1) -------------------------------
const PT_LONG: u16 = 0x0003;
const PT_BINARY: u16 = 0x0102;
const PT_UNICODE: u16 = 0x001F;

// --- Property ids we emit (MS-OXPROPS) --------------------------------------
const PID_MESSAGE_CLASS: u16 = 0x001A;
const PID_SUBJECT: u16 = 0x0037;
const PID_CLIENT_SUBMIT_HDRS: u16 = 0x007D; // PidTagTransportMessageHeaders
const PID_SENDER_NAME: u16 = 0x0C1A;
const PID_SENDER_EMAIL: u16 = 0x0C1F;
const PID_DISPLAY_CC: u16 = 0x0E03;
const PID_DISPLAY_TO: u16 = 0x0E04;
const PID_BODY: u16 = 0x1000;
const PID_HTML: u16 = 0x1013; // PidTagHtml (PT_BINARY)
const PID_INTERNET_MESSAGE_ID: u16 = 0x1035;
// Recipient object props.
const PID_DISPLAY_NAME: u16 = 0x3001;
const PID_ADDRTYPE: u16 = 0x3002;
const PID_EMAIL_ADDRESS: u16 = 0x3003;
const PID_RECIPIENT_TYPE: u16 = 0x0C15;
// Attachment object props.
const PID_ATTACH_DATA: u16 = 0x3701; // PidTagAttachDataBinary
const PID_ATTACH_FILENAME: u16 = 0x3704; // short name
const PID_ATTACH_METHOD: u16 = 0x3705;
const PID_ATTACH_LONG_FILENAME: u16 = 0x3707;
const PID_ATTACH_MIME_TAG: u16 = 0x370E;

const RECIPIENT_TYPE_TO: u32 = 1;
const RECIPIENT_TYPE_CC: u32 = 2;
const ATTACH_BY_VALUE: u32 = 1;

/// One MAPI property destined for a `__properties_version1.0` stream, plus the
/// side-stream it may need (`__substg1.0_*`).
struct Prop {
    id: u16,
    ptype: u16,
    /// The 8-byte value union (size for variable-length, inline value otherwise).
    value: [u8; 8],
    /// For variable-length props, the `__substg1.0_*` stream bytes.
    substg: Option<Vec<u8>>,
}

/// A collection of MAPI properties for one storage (top level, a recipient, or
/// an attachment).
#[derive(Default)]
struct PropSet {
    props: Vec<Prop>,
}

impl PropSet {
    fn long(&mut self, id: u16, v: u32) {
        let mut value = [0u8; 8];
        value[..4].copy_from_slice(&v.to_le_bytes());
        self.props.push(Prop {
            id,
            ptype: PT_LONG,
            value,
            substg: None,
        });
    }

    /// A `PT_UNICODE` string: stored UTF-16LE in a side-stream; the property
    /// entry records the byte length including the 2-byte terminator.
    fn unicode(&mut self, id: u16, s: &str) {
        let utf16: Vec<u8> = s.encode_utf16().flat_map(u16::to_le_bytes).collect();
        let mut value = [0u8; 8];
        value[..4].copy_from_slice(&((utf16.len() as u32) + 2).to_le_bytes());
        self.props.push(Prop {
            id,
            ptype: PT_UNICODE,
            value,
            substg: Some(utf16),
        });
    }

    fn binary(&mut self, id: u16, bytes: Vec<u8>) {
        let mut value = [0u8; 8];
        value[..4].copy_from_slice(&(bytes.len() as u32).to_le_bytes());
        self.props.push(Prop {
            id,
            ptype: PT_BINARY,
            value,
            substg: Some(bytes),
        });
    }

    /// Serialise the fixed part of the `__properties_version1.0` stream (the
    /// 16-byte entries). `header` is the storage-specific prefix (32 bytes for
    /// the top level, 8 for sub-objects).
    fn properties_stream(&self, header: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(header.len() + self.props.len() * 16);
        out.extend_from_slice(header);
        for p in &self.props {
            out.extend_from_slice(&p.ptype.to_le_bytes());
            out.extend_from_slice(&p.id.to_le_bytes());
            out.extend_from_slice(&0x0000_0006u32.to_le_bytes()); // READABLE|WRITABLE
            out.extend_from_slice(&p.value);
        }
        out
    }
}

/// Export one RFC 5322 message to `.msg` bytes (message class `IPM.Note`).
pub fn to_msg(raw: &[u8]) -> Result<Vec<u8>> {
    write_cfb_message(raw, MESSAGE_CLASS_NOTE)
}

/// Build the CFB container. Shared by [`to_msg`] and the OFT template writer
/// (`oft::to_oft`) which passes a template message class.
pub(crate) fn write_cfb_message(raw: &[u8], message_class: &str) -> Result<Vec<u8>> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or_else(|| ExportError::Parse("not a recognisable RFC5322 message".into()))?;

    let mut top = PropSet::default();
    top.unicode(PID_MESSAGE_CLASS, message_class);

    if let Some(subject) = message.subject() {
        top.unicode(PID_SUBJECT, subject);
    }
    // Faithful headers: the verbatim RFC 5322 header block round-trips every
    // field we do not model as a first-class property.
    top.unicode(PID_CLIENT_SUBMIT_HDRS, &raw_headers(raw));

    if let Some(sender) = message.from().and_then(|a| a.first()) {
        if let Some(name) = sender.name().filter(|n| !n.trim().is_empty()) {
            top.unicode(PID_SENDER_NAME, name.trim());
        }
        if let Some(email) = sender.address() {
            top.unicode(PID_SENDER_EMAIL, email);
        }
    }
    if let Some(display_to) = format_recipients(message.to()) {
        top.unicode(PID_DISPLAY_TO, &display_to);
    }
    if let Some(display_cc) = format_recipients(message.cc()) {
        top.unicode(PID_DISPLAY_CC, &display_cc);
    }
    if let Some(id) = message.message_id() {
        top.unicode(PID_INTERNET_MESSAGE_ID, id);
    }

    if let Some(text) = message.body_text(0) {
        top.unicode(PID_BODY, &text);
    }
    if let Some(html) = message.body_html(0) {
        top.binary(PID_HTML, html.as_bytes().to_vec());
    }

    let recipients = collect_recipients(&message);
    let attachments = collect_attachments(&message);

    // --- write the compound file ---
    let mut comp = cfb::CompoundFile::create(Cursor::new(Vec::new()))
        .map_err(|e| ExportError::Render(format!("cfb create: {e}")))?;

    // Top-level property stream header (MS-OXMSG §2.4.1.1): 8 reserved, next
    // recipient id, next attachment id, recipient count, attachment count, 8
    // reserved.
    let mut header = Vec::with_capacity(32);
    header.extend_from_slice(&[0u8; 8]);
    header.extend_from_slice(&(recipients.len() as u32).to_le_bytes());
    header.extend_from_slice(&(attachments.len() as u32).to_le_bytes());
    header.extend_from_slice(&(recipients.len() as u32).to_le_bytes());
    header.extend_from_slice(&(attachments.len() as u32).to_le_bytes());
    header.extend_from_slice(&[0u8; 8]);

    write_propset(&mut comp, "", &top, &header)?;

    for (i, recip) in recipients.iter().enumerate() {
        let dir = format!("/__recip_version1.0_#{i:08X}");
        comp.create_storage(&dir)
            .map_err(|e| ExportError::Render(format!("cfb storage: {e}")))?;
        write_propset(&mut comp, &dir, recip, &[0u8; 8])?;
    }
    for (i, att) in attachments.iter().enumerate() {
        let dir = format!("/__attach_version1.0_#{i:08X}");
        comp.create_storage(&dir)
            .map_err(|e| ExportError::Render(format!("cfb storage: {e}")))?;
        write_propset(&mut comp, &dir, att, &[0u8; 8])?;
    }

    comp.flush()
        .map_err(|e| ExportError::Render(format!("cfb flush: {e}")))?;
    Ok(comp.into_inner().into_inner())
}

/// Write a `PropSet`'s `__properties_version1.0` stream and every `__substg1.0_*`
/// side-stream under the storage at `dir` (`""` = root).
fn write_propset(
    comp: &mut cfb::CompoundFile<Cursor<Vec<u8>>>,
    dir: &str,
    set: &PropSet,
    header: &[u8],
) -> Result<()> {
    for p in &set.props {
        if let Some(bytes) = &p.substg {
            let name = format!("{dir}/__substg1.0_{:04X}{:04X}", p.id, p.ptype);
            let mut stream = comp
                .create_stream(&name)
                .map_err(|e| ExportError::Render(format!("cfb stream: {e}")))?;
            stream
                .write_all(bytes)
                .map_err(|e| ExportError::Render(format!("cfb write: {e}")))?;
        }
    }
    let mut stream = comp
        .create_stream(format!("{dir}/__properties_version1.0"))
        .map_err(|e| ExportError::Render(format!("cfb stream: {e}")))?;
    stream
        .write_all(&set.properties_stream(header))
        .map_err(|e| ExportError::Render(format!("cfb write: {e}")))?;
    Ok(())
}

fn collect_recipients(message: &Message<'_>) -> Vec<PropSet> {
    let mut out = Vec::new();
    push_recipients(&mut out, message.to(), RECIPIENT_TYPE_TO);
    push_recipients(&mut out, message.cc(), RECIPIENT_TYPE_CC);
    out
}

fn push_recipients(out: &mut Vec<PropSet>, addr: Option<&Address<'_>>, kind: u32) {
    let Some(addr) = addr else { return };
    for a in addr.iter() {
        let Some(email) = a.address().filter(|s| !s.is_empty()) else {
            continue;
        };
        let mut set = PropSet::default();
        let display = a
            .name()
            .filter(|n| !n.trim().is_empty())
            .map(str::trim)
            .unwrap_or(email);
        set.unicode(PID_DISPLAY_NAME, display);
        set.unicode(PID_EMAIL_ADDRESS, email);
        set.unicode(PID_ADDRTYPE, "SMTP");
        set.long(PID_RECIPIENT_TYPE, kind);
        out.push(set);
    }
}

fn collect_attachments(message: &Message<'_>) -> Vec<PropSet> {
    let mut out = Vec::new();
    for part in message.attachments() {
        let bytes = match &part.body {
            PartType::Binary(b) | PartType::InlineBinary(b) => b.as_ref().to_vec(),
            PartType::Text(t) | PartType::Html(t) => t.as_bytes().to_vec(),
            PartType::Message(_) | PartType::Multipart(_) => continue,
        };
        let name = part.attachment_name().unwrap_or("attachment").to_string();
        let mut set = PropSet::default();
        set.long(PID_ATTACH_METHOD, ATTACH_BY_VALUE);
        set.unicode(PID_ATTACH_LONG_FILENAME, &name);
        set.unicode(PID_ATTACH_FILENAME, &name);
        set.unicode(PID_DISPLAY_NAME, &name);
        if let Some(ct) = part.content_type() {
            let mime = match ct.subtype() {
                Some(sub) => format!("{}/{}", ct.ctype(), sub),
                None => ct.ctype().to_string(),
            };
            set.unicode(PID_ATTACH_MIME_TAG, &mime);
        }
        set.binary(PID_ATTACH_DATA, bytes);
        out.push(set);
    }
    out
}

/// A `Name <email>, other@example.com` display list for `PidTagDisplayTo/Cc`.
fn format_recipients(addr: Option<&Address<'_>>) -> Option<String> {
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
    (!parts.is_empty()).then(|| parts.join("; "))
}

/// The verbatim RFC 5322 header block (everything before the empty line that
/// separates headers from the body), with the trailing blank line dropped.
fn raw_headers(raw: &[u8]) -> String {
    let end = find_header_end(raw).unwrap_or(raw.len());
    String::from_utf8_lossy(&raw[..end]).trim_end().to_string()
}

fn find_header_end(raw: &[u8]) -> Option<usize> {
    // CRLFCRLF or LFLF.
    raw.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 2)
        .or_else(|| raw.windows(2).position(|w| w == b"\n\n").map(|i| i + 1))
}

// ===========================================================================
// Reader — HOSTILE INPUT (see the module-level jail-boundary note). Panic-free;
// backs the round-trip tests and the CFB fuzz target.
// ===========================================================================

/// What [`read_msg`] recovers from a `.msg`/`.oft` container: enough to prove
/// the export floor (body + attachments + headers) round-trips.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ParsedMsg {
    pub message_class: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    /// The verbatim transport headers (`PidTagTransportMessageHeaders`).
    pub headers: Option<String>,
    pub display_to: Option<String>,
    /// `(filename, bytes)` for each attachment.
    pub attachments: Vec<(String, Vec<u8>)>,
}

/// Parse a `.msg`/`.oft` CFB container back into its floor properties.
///
/// **Hostile input** — see the module-level jail-boundary note. Never panics on
/// arbitrary bytes (fuzzed); rejects oversized input; ignores malformed streams
/// rather than trusting them.
pub fn read_msg(bytes: &[u8]) -> Result<ParsedMsg> {
    if bytes.len() > MAX_READ_BYTES {
        return Err(ExportError::Parse("msg exceeds size limit".into()));
    }
    let mut comp = cfb::CompoundFile::open(Cursor::new(bytes.to_vec()))
        .map_err(|e| ExportError::Parse(format!("not a CFB container: {e}")))?;

    // Enumerate stream paths first (walk borrows immutably), then read.
    let paths: Vec<String> = comp
        .walk()
        .filter(|e| e.is_stream())
        .map(|e| e.path().to_string_lossy().replace('\\', "/"))
        .collect();

    let mut out = ParsedMsg::default();
    let mut attach: std::collections::BTreeMap<String, (Option<String>, Option<Vec<u8>>)> =
        std::collections::BTreeMap::new();

    for path in &paths {
        let base = path.rsplit('/').next().unwrap_or(path);
        // Top-level string properties.
        match base {
            "__substg1.0_001A001F" => out.message_class = read_unicode(&mut comp, path),
            "__substg1.0_0037001F" => out.subject = read_unicode(&mut comp, path),
            "__substg1.0_1000001F" => out.body = read_unicode(&mut comp, path),
            "__substg1.0_007D001F" => out.headers = read_unicode(&mut comp, path),
            "__substg1.0_0E04001F" => out.display_to = read_unicode(&mut comp, path),
            _ => {}
        }
        // Attachment streams live under `__attach_version1.0_#XXXXXXXX/`.
        if let Some(dir) = path
            .split('/')
            .find(|seg| seg.starts_with("__attach_version1.0_"))
        {
            let entry = attach.entry(dir.to_string()).or_default();
            match base {
                "__substg1.0_3707001F" => entry.0 = read_unicode(&mut comp, path),
                "__substg1.0_37010102" => entry.1 = read_binary(&mut comp, path),
                _ => {}
            }
        }
    }

    for (_, (name, data)) in attach {
        if let Some(bytes) = data {
            out.attachments
                .push((name.unwrap_or_else(|| "attachment".into()), bytes));
        }
    }
    Ok(out)
}

fn read_stream_bytes(comp: &mut cfb::CompoundFile<Cursor<Vec<u8>>>, path: &str) -> Option<Vec<u8>> {
    let mut stream = comp.open_stream(path).ok()?;
    let mut buf = Vec::new();
    // Cap per-stream reads at the whole-file limit so a corrupt length can't
    // drive an unbounded allocation.
    Read::by_ref(&mut stream)
        .take(MAX_READ_BYTES as u64)
        .read_to_end(&mut buf)
        .ok()?;
    Some(buf)
}

fn read_unicode(comp: &mut cfb::CompoundFile<Cursor<Vec<u8>>>, path: &str) -> Option<String> {
    let bytes = read_stream_bytes(comp, path)?;
    let mut units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    // Outlook stores UTF-16 strings NUL-terminated; drop any trailing NULs so a
    // real-world `.msg` reads the same as one we wrote (which omits them).
    while units.last() == Some(&0) {
        units.pop();
    }
    Some(String::from_utf16_lossy(&units))
}

fn read_binary(comp: &mut cfb::CompoundFile<Cursor<Vec<u8>>>, path: &str) -> Option<Vec<u8>> {
    read_stream_bytes(comp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &[u8] = b"From: Alice <alice@example.com>\r\n\
To: Bob <bob@example.com>\r\n\
Cc: Carol <carol@example.com>\r\n\
Subject: Quarterly report\r\n\
Message-ID: <abc123@example.com>\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Hello Bob,\r\nHere is the report.\r\n";

    #[test]
    fn round_trips_body_headers_subject() {
        let bytes = to_msg(SAMPLE).unwrap();
        let parsed = read_msg(&bytes).unwrap();
        assert_eq!(parsed.subject.as_deref(), Some("Quarterly report"));
        assert_eq!(parsed.message_class.as_deref(), Some(MESSAGE_CLASS_NOTE));
        assert!(
            parsed
                .body
                .as_deref()
                .unwrap()
                .contains("Here is the report.")
        );
        let headers = parsed.headers.unwrap();
        assert!(headers.contains("Subject: Quarterly report"));
        assert!(headers.contains("Message-ID: <abc123@example.com>"));
        // The header block never bleeds into the body.
        assert!(!headers.contains("Hello Bob"));
        assert!(parsed.display_to.unwrap().contains("bob@example.com"));
    }

    #[test]
    fn round_trips_attachment() {
        let raw = b"From: a@example.com\r\n\
To: b@example.com\r\n\
Subject: with file\r\n\
Content-Type: multipart/mixed; boundary=X\r\n\
\r\n\
--X\r\n\
Content-Type: text/plain\r\n\
\r\n\
body text\r\n\
--X\r\n\
Content-Type: application/pdf; name=\"doc.pdf\"\r\n\
Content-Disposition: attachment; filename=\"doc.pdf\"\r\n\
Content-Transfer-Encoding: base64\r\n\
\r\n\
JVBERi0xLjQK\r\n\
--X--\r\n";
        let bytes = to_msg(raw).unwrap();
        let parsed = read_msg(&bytes).unwrap();
        assert_eq!(parsed.attachments.len(), 1, "one attachment expected");
        let (name, data) = &parsed.attachments[0];
        assert_eq!(name, "doc.pdf");
        assert_eq!(&data[..5], b"%PDF-");
    }

    #[test]
    fn is_a_valid_cfb_container() {
        let bytes = to_msg(SAMPLE).unwrap();
        // CFB magic (OLE2 signature).
        assert_eq!(
            &bytes[..8],
            &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]
        );
    }

    #[test]
    fn reader_never_panics_on_garbage() {
        // Not a CFB → clean error, no panic.
        assert!(read_msg(b"not a compound file at all").is_err());
        assert!(read_msg(&[]).is_err());
        // Valid magic, truncated body → error, no panic.
        let mut trunc = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
        trunc.extend_from_slice(&[0u8; 24]);
        let _ = read_msg(&trunc);
    }

    #[test]
    fn oversize_input_rejected() {
        let big = vec![0u8; MAX_READ_BYTES + 1];
        assert!(read_msg(&big).is_err());
    }

    /// Interop smoke: hand-build a CFB the way Outlook writes one — UTF-16
    /// strings carry a trailing NUL terminator inside the substg stream, and the
    /// top-level property stream uses the 32-byte header. Our reader must decode
    /// it identically to one we produced ourselves.
    #[test]
    fn reads_outlook_style_fixture() {
        let outlook = build_outlook_style_msg();
        let parsed = read_msg(&outlook).unwrap();
        assert_eq!(parsed.subject.as_deref(), Some("Outlook made this"));
        assert_eq!(parsed.body.as_deref(), Some("Body from Outlook."));
        assert_eq!(parsed.message_class.as_deref(), Some("IPM.Note"));
    }

    /// Build a tiny Outlook-shaped `.msg`: NUL-terminated UTF-16 substg streams +
    /// the 32-byte top-level property header, written directly with `cfb` (not
    /// via our own writer) so the test genuinely exercises the interop path.
    fn build_outlook_style_msg() -> Vec<u8> {
        fn utf16_nul(s: &str) -> Vec<u8> {
            s.encode_utf16()
                .chain(std::iter::once(0))
                .flat_map(u16::to_le_bytes)
                .collect()
        }
        let mut comp = cfb::CompoundFile::create(Cursor::new(Vec::new())).unwrap();
        for (name, text) in [
            ("__substg1.0_001A001F", "IPM.Note"),
            ("__substg1.0_0037001F", "Outlook made this"),
            ("__substg1.0_1000001F", "Body from Outlook."),
        ] {
            let mut s = comp.create_stream(format!("/{name}")).unwrap();
            s.write_all(&utf16_nul(text)).unwrap();
        }
        // 32-byte top-level header, zero counts, then no property entries (a
        // lenient reader keys off the substg streams, not the entry table).
        let mut props = comp.create_stream("/__properties_version1.0").unwrap();
        props.write_all(&[0u8; 32]).unwrap();
        drop(props);
        comp.flush().unwrap();
        comp.into_inner().into_inner()
    }
}
