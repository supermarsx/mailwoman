//! MS-OXMSG `.msg` export (plan §3 e5 + t10-e9, §1.7, SPEC §10.6, §28.8).
//!
//! An `.msg` file is a CFB (OLE2 Compound-File-Binary) container carrying MAPI
//! properties. This module writes that container with the [`cfb`] crate and
//! layers our own **MS-OXMSG** property encoder on top.
//!
//! # Fidelity tiers
//! - **Floor (26.9, plan §0.6/§1.7):** faithful body + attachments + headers via
//!   the standard numbered-property streams.
//! - **Deep write fidelity (26.10, plan §1.6/§3 t10-e9, SPEC §28.8):** custom
//!   **named properties** (the `__nameid` map, MS-OXMSG §2.2.3) and **embedded
//!   messages** (`PidTagAttachMethod = afEmbeddedMessage`, a nested MSG storage).
//!   Both are **additive**: a message that carries no custom `X-*` headers and no
//!   `message/rfc822` part is written byte-for-byte identically to the floor (the
//!   hard regression gate — see `tests/msg_deep_fidelity.rs`). The deep layer only
//!   appends storages/streams when the source actually has those features.
//!
//! Custom named properties are sourced from custom internet headers (`X-*`),
//! mapped to string-named properties in the `PS_INTERNET_HEADERS` namespace, the
//! same mapping Outlook/MS-OXCMAIL §2.5.3 uses. Embedded messages are sourced
//! from MIME `message/rfc822` parts (previously dropped).
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

/// Bound on how deep an embedded-message chain we will follow on the (hostile)
/// read path — a malicious `.msg` could otherwise nest storages without limit.
const MAX_EMBED_DEPTH: usize = 8;

// --- MAPI property types (MS-OXCDATA §2.11.1) -------------------------------
const PT_LONG: u16 = 0x0003;
const PT_BINARY: u16 = 0x0102;
const PT_OBJECT: u16 = 0x000D;
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
const PID_ATTACH_DATA: u16 = 0x3701; // PidTagAttachDataBinary (PT_BINARY) / DataObject (PT_OBJECT)
const PID_ATTACH_FILENAME: u16 = 0x3704; // short name
const PID_ATTACH_METHOD: u16 = 0x3705;
const PID_ATTACH_LONG_FILENAME: u16 = 0x3707;
const PID_ATTACH_MIME_TAG: u16 = 0x370E;

const RECIPIENT_TYPE_TO: u32 = 1;
const RECIPIENT_TYPE_CC: u32 = 2;
const ATTACH_BY_VALUE: u32 = 1;
/// `PidTagAttachMethod = afEmbeddedMessage` (MS-OXPROPS): the attachment data is
/// a nested MSG storage (`__substg1.0_3701000D`), not a binary blob.
const ATTACH_EMBEDDED_MSG: u32 = 5;

// --- Named-property map (MS-OXMSG §2.2.3) -----------------------------------
/// `PS_INTERNET_HEADERS` `{00020386-0000-0000-C000-000000000046}` — the property
/// set custom internet (`X-*`) headers map into (MS-OXCMAIL §2.5.3), serialised
/// little-endian for `Data1`/`Data2`/`Data3` then big-endian for `Data4`.
const PS_INTERNET_HEADERS: [u8; 16] = [
    0x86, 0x03, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46,
];
/// Named properties dispatch to property ids at or above `0x8000`
/// (MS-OXPROPS §1.3.2); property index `i` → id `0x8000 + i`.
const NAMED_PROP_BASE: u16 = 0x8000;
/// GUID-index of the first GUID in the `__nameid` GUID stream: 0 = none,
/// 1 = `PS_MAPI`, 2 = `PS_PUBLIC_STRINGS`, 3.. = stream index 0.. (MS-OXMSG §2.2.3.1.2).
const GUID_INDEX_STREAM_FIRST: u16 = 3;

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

    /// A `PT_OBJECT` entry: the value lives in a sibling storage (e.g. an
    /// embedded message under `__substg1.0_3701000D`), so no side-stream is
    /// written here — only the property-table entry pointing at the object.
    fn object(&mut self, id: u16) {
        let mut value = [0u8; 8];
        // Size unknown/streamed; 0xFFFFFFFF is the conventional "not stored inline".
        value[..4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        self.props.push(Prop {
            id,
            ptype: PT_OBJECT,
            value,
            substg: None,
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

/// One attachment ready to be written: its property set plus, when it is an
/// embedded `message/rfc822`, the fully-built nested message.
struct BuiltAttachment {
    props: PropSet,
    embedded: Option<Box<BuiltMessage>>,
}

/// A message reduced to the storages/streams that make up its `.msg`
/// representation. Built recursively so an embedded message is just another
/// `BuiltMessage` under an attachment.
struct BuiltMessage {
    top: PropSet,
    /// Ordered custom named-property names (string-named, `PS_INTERNET_HEADERS`).
    /// Their values already live in `top` at ids `NAMED_PROP_BASE + i`.
    named: Vec<String>,
    recipients: Vec<PropSet>,
    attachments: Vec<BuiltAttachment>,
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

    let built = build_message(&message, raw, message_class);

    let mut comp = cfb::CompoundFile::create(Cursor::new(Vec::new()))
        .map_err(|e| ExportError::Render(format!("cfb create: {e}")))?;

    write_built_message(&mut comp, "", &built)?;

    comp.flush()
        .map_err(|e| ExportError::Render(format!("cfb flush: {e}")))?;
    Ok(comp.into_inner().into_inner())
}

/// Reduce a parsed message (top-level or embedded) to its `BuiltMessage`.
///
/// `raw` is the raw byte block the header stream is taken from — the outer
/// RFC 5322 bytes for the top level, the nested part's own bytes for an embedded
/// message.
fn build_message(message: &Message<'_>, raw: &[u8], message_class: &str) -> BuiltMessage {
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

    // Deep fidelity (additive): custom `X-*` headers → string-named properties.
    // Emitting NO named props when there are none keeps the floor byte-identical.
    let named = collect_named_props(message, &mut top);

    let recipients = collect_recipients(message);
    let attachments = collect_attachments(message);

    BuiltMessage {
        top,
        named,
        recipients,
        attachments,
    }
}

/// Write a fully-built message (top-level `dir == ""`, or an embedded message
/// under an attachment) as CFB storages/streams.
///
/// Ordering note (regression gate): when `built.named` is empty and no
/// attachment is embedded, this issues exactly the same `cfb` calls, in the same
/// order, as the 26.9 floor writer — so a floor message is byte-identical.
fn write_built_message(
    comp: &mut cfb::CompoundFile<Cursor<Vec<u8>>>,
    dir: &str,
    built: &BuiltMessage,
) -> Result<()> {
    // Top-level property stream header (MS-OXMSG §2.4.1.1): 8 reserved, next
    // recipient id, next attachment id, recipient count, attachment count, 8
    // reserved.
    let mut header = Vec::with_capacity(32);
    header.extend_from_slice(&[0u8; 8]);
    header.extend_from_slice(&(built.recipients.len() as u32).to_le_bytes());
    header.extend_from_slice(&(built.attachments.len() as u32).to_le_bytes());
    header.extend_from_slice(&(built.recipients.len() as u32).to_le_bytes());
    header.extend_from_slice(&(built.attachments.len() as u32).to_le_bytes());
    header.extend_from_slice(&[0u8; 8]);

    write_propset(comp, dir, &built.top, &header)?;

    // Named-property map (only when custom named props exist → floor unchanged).
    if !built.named.is_empty() {
        write_nameid_map(comp, dir, &built.named)?;
    }

    for (i, recip) in built.recipients.iter().enumerate() {
        let rdir = format!("{dir}/__recip_version1.0_#{i:08X}");
        comp.create_storage(&rdir)
            .map_err(|e| ExportError::Render(format!("cfb storage: {e}")))?;
        write_propset(comp, &rdir, recip, &[0u8; 8])?;
    }
    for (i, att) in built.attachments.iter().enumerate() {
        let adir = format!("{dir}/__attach_version1.0_#{i:08X}");
        comp.create_storage(&adir)
            .map_err(|e| ExportError::Render(format!("cfb storage: {e}")))?;
        write_propset(comp, &adir, &att.props, &[0u8; 8])?;
        if let Some(embedded) = &att.embedded {
            // The embedded message is a full MSG under `__substg1.0_3701000D`
            // (PidTagAttachDataObject, PT_OBJECT), recursively encoded.
            let edir = format!("{adir}/__substg1.0_{PID_ATTACH_DATA:04X}{PT_OBJECT:04X}");
            comp.create_storage(&edir)
                .map_err(|e| ExportError::Render(format!("cfb storage: {e}")))?;
            write_built_message(comp, &edir, embedded)?;
        }
    }
    Ok(())
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

/// Write the `__nameid_version1.0` storage (MS-OXMSG §2.2.3): the GUID, entry,
/// and string streams describing every string-named property under `dir`.
///
/// All names here are string-named in `PS_INTERNET_HEADERS`, so the GUID stream
/// carries that single GUID and every entry references GUID-index 3.
fn write_nameid_map(
    comp: &mut cfb::CompoundFile<Cursor<Vec<u8>>>,
    dir: &str,
    names: &[String],
) -> Result<()> {
    let mut entries = Vec::with_capacity(names.len() * 8);
    let mut strings = Vec::new();
    for (i, name) in names.iter().enumerate() {
        let str_offset = strings.len() as u32;
        // Entry (8 bytes): string offset · (guid-index<<1 | N=1) · property index.
        entries.extend_from_slice(&str_offset.to_le_bytes());
        let kind_and_guid: u16 = (GUID_INDEX_STREAM_FIRST << 1) | 0x0001; // N=1 → string name
        entries.extend_from_slice(&kind_and_guid.to_le_bytes());
        entries.extend_from_slice(&(i as u16).to_le_bytes());
        // String-stream record: 4-byte length + UTF-16LE name, padded to 4 bytes.
        let utf16: Vec<u8> = name.encode_utf16().flat_map(u16::to_le_bytes).collect();
        strings.extend_from_slice(&(utf16.len() as u32).to_le_bytes());
        strings.extend_from_slice(&utf16);
        while strings.len() % 4 != 0 {
            strings.push(0);
        }
    }

    let ndir = format!("{dir}/__nameid_version1.0");
    comp.create_storage(&ndir)
        .map_err(|e| ExportError::Render(format!("cfb storage: {e}")))?;
    for (tag, bytes) in [
        (0x0002u16, PS_INTERNET_HEADERS.to_vec()), // GUID stream
        (0x0003u16, entries),                      // entry stream
        (0x0004u16, strings),                      // string stream
    ] {
        let name = format!("{ndir}/__substg1.0_{tag:04X}{PT_BINARY:04X}");
        let mut stream = comp
            .create_stream(&name)
            .map_err(|e| ExportError::Render(format!("cfb stream: {e}")))?;
        stream
            .write_all(&bytes)
            .map_err(|e| ExportError::Render(format!("cfb write: {e}")))?;
    }
    Ok(())
}

/// Map custom internet headers (`X-*`) to string-named `PT_UNICODE` properties,
/// pushing each value into `top` at id `NAMED_PROP_BASE + i` and returning the
/// ordered names for the `__nameid` map. Returns empty (⇒ no map written, floor
/// byte-unchanged) when the message carries no custom headers.
fn collect_named_props(message: &Message<'_>, top: &mut PropSet) -> Vec<String> {
    let mut names = Vec::new();
    for (name, value) in message.headers_raw() {
        if !name.to_ascii_lowercase().starts_with("x-") {
            continue;
        }
        let idx = names.len();
        let Ok(id) = u16::try_from(NAMED_PROP_BASE as usize + idx) else {
            break; // ran out of the named-property id range; stop (extremely unlikely)
        };
        top.unicode(id, value.trim());
        names.push(name.to_string());
    }
    names
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

fn collect_attachments(message: &Message<'_>) -> Vec<BuiltAttachment> {
    let mut out = Vec::new();
    for part in message.attachments() {
        // Embedded `message/rfc822` → nested MSG storage (deep fidelity, additive).
        if let PartType::Message(nested) = &part.body {
            let name = nested
                .subject()
                .filter(|s| !s.trim().is_empty())
                .map(|s| format!("{}.msg", s.trim()))
                .unwrap_or_else(|| "message.msg".to_string());
            let mut set = PropSet::default();
            set.long(PID_ATTACH_METHOD, ATTACH_EMBEDDED_MSG);
            set.unicode(PID_ATTACH_LONG_FILENAME, &name);
            set.unicode(PID_ATTACH_FILENAME, &name);
            set.unicode(PID_DISPLAY_NAME, &name);
            set.unicode(PID_ATTACH_MIME_TAG, "message/rfc822");
            set.object(PID_ATTACH_DATA);
            let embedded = build_message(nested, nested.raw_message(), MESSAGE_CLASS_NOTE);
            out.push(BuiltAttachment {
                props: set,
                embedded: Some(Box::new(embedded)),
            });
            continue;
        }
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
        out.push(BuiltAttachment {
            props: set,
            embedded: None,
        });
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

/// A custom named property recovered from the `__nameid` map: its name and, for
/// the `PT_UNICODE` values we write, its string value.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NamedProperty {
    pub name: String,
    pub value: String,
}

/// What [`read_msg`] recovers from a `.msg`/`.oft` container: the floor
/// properties (body + attachments + headers) plus the deep-fidelity layer
/// (custom named properties + embedded messages).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ParsedMsg {
    pub message_class: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    /// The verbatim transport headers (`PidTagTransportMessageHeaders`).
    pub headers: Option<String>,
    pub display_to: Option<String>,
    /// `(filename, bytes)` for each by-value attachment.
    pub attachments: Vec<(String, Vec<u8>)>,
    /// Custom named properties recovered via the `__nameid` map (deep fidelity).
    pub named_properties: Vec<NamedProperty>,
    /// Embedded messages (`afEmbeddedMessage`), each parsed recursively.
    pub embedded: Vec<ParsedMsg>,
}

/// Parse a `.msg`/`.oft` CFB container back into its floor + deep properties.
///
/// **Hostile input** — see the module-level jail-boundary note. Never panics on
/// arbitrary bytes (fuzzed); rejects oversized input; ignores malformed streams
/// rather than trusting them; bounds embedded-message recursion.
pub fn read_msg(bytes: &[u8]) -> Result<ParsedMsg> {
    if bytes.len() > MAX_READ_BYTES {
        return Err(ExportError::Parse("msg exceeds size limit".into()));
    }
    let mut comp = cfb::CompoundFile::open(Cursor::new(bytes.to_vec()))
        .map_err(|e| ExportError::Parse(format!("not a CFB container: {e}")))?;

    // Enumerate stream paths first (walk borrows immutably), then read. Paths are
    // normalised to `/`-separated with a leading `/`.
    let paths: Vec<String> = comp
        .walk()
        .filter(|e| e.is_stream())
        .map(|e| {
            let p = e.path().to_string_lossy().replace('\\', "/");
            if p.starts_with('/') {
                p
            } else {
                format!("/{p}")
            }
        })
        .collect();

    Ok(parse_storage(&mut comp, &paths, "", 0))
}

/// Parse the message rooted at storage `prefix` (`""` = container root). Recurses
/// into embedded-message storages up to [`MAX_EMBED_DEPTH`].
fn parse_storage(
    comp: &mut cfb::CompoundFile<Cursor<Vec<u8>>>,
    paths: &[String],
    prefix: &str,
    depth: usize,
) -> ParsedMsg {
    let mut out = ParsedMsg::default();

    // Named-property id → name map from this storage's `__nameid` sub-storage.
    let nameid = read_nameid_map(comp, paths, prefix);

    // Direct-child streams: top-level properties + named-property values.
    for path in paths {
        let Some(base) = direct_child_base(path, prefix) else {
            continue;
        };
        match base {
            "__substg1.0_001A001F" => out.message_class = read_unicode(comp, path),
            "__substg1.0_0037001F" => out.subject = read_unicode(comp, path),
            "__substg1.0_1000001F" => out.body = read_unicode(comp, path),
            "__substg1.0_007D001F" => out.headers = read_unicode(comp, path),
            "__substg1.0_0E04001F" => out.display_to = read_unicode(comp, path),
            _ => {
                if let Some(id) = named_value_id(base)
                    && let (Some(name), Some(value)) =
                        (nameid.get(&id).cloned(), read_unicode(comp, path))
                {
                    out.named_properties.push(NamedProperty { name, value });
                }
            }
        }
    }
    out.named_properties
        .sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.value.cmp(&b.value)));

    // Child storages: attachments (by-value + embedded messages).
    for adir in child_storages(paths, prefix) {
        if !storage_name(&adir, prefix).starts_with("__attach_version1.0_") {
            continue;
        }
        // An embedded message lives under `<attach>/__substg1.0_3701000D`.
        let edir = format!("{adir}/__substg1.0_{PID_ATTACH_DATA:04X}{PT_OBJECT:04X}");
        let is_embedded = paths.iter().any(|p| p.starts_with(&format!("{edir}/")));
        if is_embedded {
            if depth < MAX_EMBED_DEPTH {
                out.embedded
                    .push(parse_storage(comp, paths, &edir, depth + 1));
            }
            continue;
        }
        // By-value attachment: filename + binary data.
        let mut name = None;
        let mut data = None;
        for path in paths {
            if let Some(base) = direct_child_base(path, &adir) {
                match base {
                    "__substg1.0_3707001F" => name = read_unicode(comp, path),
                    "__substg1.0_37010102" => data = read_binary(comp, path),
                    _ => {}
                }
            }
        }
        if let Some(bytes) = data {
            out.attachments
                .push((name.unwrap_or_else(|| "attachment".into()), bytes));
        }
    }
    out
}

/// Read a storage's `__nameid` map into a dispatch-id → name table. Dispatch id
/// `NAMED_PROP_BASE + property-index` matches the id under which the named value
/// stream is stored. Best-effort: malformed maps yield an empty table.
fn read_nameid_map(
    comp: &mut cfb::CompoundFile<Cursor<Vec<u8>>>,
    paths: &[String],
    prefix: &str,
) -> std::collections::BTreeMap<u16, String> {
    let mut map = std::collections::BTreeMap::new();
    let ndir = format!("{prefix}/__nameid_version1.0");
    let entry_path = format!("{ndir}/__substg1.0_00030102");
    let string_path = format!("{ndir}/__substg1.0_00040102");
    if !paths.iter().any(|p| p == &entry_path) {
        return map;
    }
    let Some(entries) = read_binary(comp, &entry_path) else {
        return map;
    };
    let strings = read_binary(comp, &string_path).unwrap_or_default();

    for entry in entries.chunks_exact(8) {
        let str_offset = u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]) as usize;
        let kind_and_guid = u16::from_le_bytes([entry[4], entry[5]]);
        let prop_index = u16::from_le_bytes([entry[6], entry[7]]);
        if kind_and_guid & 0x0001 == 0 {
            continue; // numeric (LID) named prop — we only emit string names
        }
        if let Some(name) = read_string_record(&strings, str_offset)
            && let Some(id) = NAMED_PROP_BASE.checked_add(prop_index)
        {
            map.insert(id, name);
        }
    }
    map
}

/// Read one `<4-byte len><UTF-16LE bytes>` record from the `__nameid` string
/// stream at `offset`. Bounds-checked; returns `None` on any inconsistency.
fn read_string_record(strings: &[u8], offset: usize) -> Option<String> {
    let len_end = offset.checked_add(4)?;
    let len = u32::from_le_bytes(strings.get(offset..len_end)?.try_into().ok()?) as usize;
    let data_end = len_end.checked_add(len)?;
    let bytes = strings.get(len_end..data_end)?;
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    Some(String::from_utf16_lossy(&units))
}

/// The dispatch id for a named-property value stream `__substg1.0_XXXX001F`
/// whose id is at or above [`NAMED_PROP_BASE`]; `None` otherwise.
fn named_value_id(base: &str) -> Option<u16> {
    let rest = base.strip_prefix("__substg1.0_")?;
    if rest.len() != 8 || !rest.ends_with("001F") {
        return None;
    }
    let id = u16::from_str_radix(&rest[..4], 16).ok()?;
    (id >= NAMED_PROP_BASE).then_some(id)
}

/// The final path segment (base name) of a stream that is a *direct* child of
/// storage `prefix` (`""` = root), or `None` if `path` is not a direct child.
fn direct_child_base<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = path.strip_prefix(prefix)?.strip_prefix('/')?;
    (!rest.contains('/')).then_some(rest)
}

/// The set of immediate child-storage paths under `prefix`, derived from the
/// stream paths that live beneath them.
fn child_storages(paths: &[String], prefix: &str) -> Vec<String> {
    let mut dirs: Vec<String> = Vec::new();
    for path in paths {
        let Some(rest) = path.strip_prefix(prefix).and_then(|r| r.strip_prefix('/')) else {
            continue;
        };
        let Some((seg, tail)) = rest.split_once('/') else {
            continue; // a direct-child stream, not a sub-storage
        };
        // `tail` non-empty ⇒ `seg` is a storage. Dedup.
        if tail.is_empty() {
            continue;
        }
        let dir = format!("{prefix}/{seg}");
        if !dirs.contains(&dir) {
            dirs.push(dir);
        }
    }
    dirs
}

/// The name of storage `dir` relative to its parent `prefix`.
fn storage_name(dir: &str, prefix: &str) -> String {
    dir.strip_prefix(prefix)
        .and_then(|r| r.strip_prefix('/'))
        .unwrap_or(dir)
        .to_string()
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
        // Floor message: no deep-fidelity artefacts.
        assert!(parsed.named_properties.is_empty());
        assert!(parsed.embedded.is_empty());
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

    /// Deep fidelity: a custom `X-*` header round-trips as a named property via
    /// the `__nameid` map.
    #[test]
    fn round_trips_named_property() {
        let raw = b"From: a@example.com\r\n\
To: b@example.com\r\n\
Subject: tagged\r\n\
X-Custom-Tag: hello-world-42\r\n\
X-Priority-Label: urgent\r\n\
\r\n\
body\r\n";
        let bytes = to_msg(raw).unwrap();
        let parsed = read_msg(&bytes).unwrap();
        let got: Vec<(&str, &str)> = parsed
            .named_properties
            .iter()
            .map(|p| (p.name.as_str(), p.value.as_str()))
            .collect();
        assert!(
            got.contains(&("X-Custom-Tag", "hello-world-42")),
            "named props were {got:?}"
        );
        assert!(got.contains(&("X-Priority-Label", "urgent")), "got {got:?}");
    }

    /// Deep fidelity: a `message/rfc822` part round-trips as an embedded message.
    #[test]
    fn round_trips_embedded_message() {
        let raw = b"From: outer@example.com\r\n\
To: rcpt@example.com\r\n\
Subject: fwd wrapper\r\n\
Content-Type: multipart/mixed; boundary=B\r\n\
\r\n\
--B\r\n\
Content-Type: text/plain\r\n\
\r\n\
See attached message.\r\n\
--B\r\n\
Content-Type: message/rfc822\r\n\
\r\n\
From: inner@example.com\r\n\
To: dest@example.com\r\n\
Subject: the inner note\r\n\
\r\n\
Inner body content.\r\n\
--B--\r\n";
        let bytes = to_msg(raw).unwrap();
        let parsed = read_msg(&bytes).unwrap();
        assert_eq!(parsed.embedded.len(), 1, "one embedded message expected");
        let inner = &parsed.embedded[0];
        assert_eq!(inner.subject.as_deref(), Some("the inner note"));
        assert!(
            inner
                .body
                .as_deref()
                .unwrap()
                .contains("Inner body content.")
        );
        assert!(
            inner
                .headers
                .as_deref()
                .unwrap()
                .contains("inner@example.com")
        );
        // The embedded message is not surfaced as a by-value attachment.
        assert!(parsed.attachments.is_empty());
    }

    /// Deep fidelity nests: named property + embedded message together, with the
    /// embedded message carrying its own named property (own `__nameid` scope).
    #[test]
    fn named_and_embedded_together() {
        let raw = b"From: outer@example.com\r\n\
Subject: combined\r\n\
X-Outer-Flag: outer-value\r\n\
Content-Type: multipart/mixed; boundary=B\r\n\
\r\n\
--B\r\n\
Content-Type: text/plain\r\n\
\r\n\
top body\r\n\
--B\r\n\
Content-Type: message/rfc822\r\n\
\r\n\
From: inner@example.com\r\n\
Subject: inner\r\n\
X-Inner-Flag: inner-value\r\n\
\r\n\
inner body\r\n\
--B--\r\n";
        let bytes = to_msg(raw).unwrap();
        let parsed = read_msg(&bytes).unwrap();
        assert!(
            parsed
                .named_properties
                .iter()
                .any(|p| p.name == "X-Outer-Flag" && p.value == "outer-value")
        );
        assert_eq!(parsed.embedded.len(), 1);
        assert!(
            parsed.embedded[0]
                .named_properties
                .iter()
                .any(|p| p.name == "X-Inner-Flag" && p.value == "inner-value")
        );
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
