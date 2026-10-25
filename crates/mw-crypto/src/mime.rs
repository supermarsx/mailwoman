//! MIME structures for signed and encrypted mail (t32-e37 PGP/MIME, t32-e41
//! S/MIME), kept in `mw-crypto` so the browser worker and the server read them the
//! same way.
//!
//! - RFC 3156 (PGP/MIME): `multipart/encrypted; protocol="application/pgp-encrypted"`
//!   and `multipart/signed; protocol="application/pgp-signature"`.
//! - RFC 8551 (S/MIME 4.0): `multipart/signed; protocol="application/pkcs7-signature"`
//!   and `application/pkcs7-mime; smime-type=...`.
//! - RFC 9788 (header protection): protected header fields on the cryptographic
//!   payload, `hp="cipher"` / `hp="clear"`, `HP-Outer`, and the legacy display
//!   element; the older `protected-headers="v1"` marking is read too.
//! - Inline PGP (armored blocks inside a text body) is read, never written.
//!
//! A signed part is verified over its **exact bytes** as they appear between the
//! boundaries (RFC 3156 §5, RFC 8551 §3.5), with line endings made CRLF. The parser
//! below therefore slices the raw message rather than re-serializing a parsed tree.
//! It is deliberately small: headers, `Content-Type` parameters, multipart
//! boundaries, base64 and quoted-printable — what locating and checking crypto parts
//! needs, not a general mail parser.

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::error::{CryptoError, Result};

/// Nesting deeper than this is refused (a crafted message cannot recurse forever).
const MAX_DEPTH: usize = 16;

/// A MIME entity: one header block and a body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MimeEntity {
    /// Header fields in order, unfolded, names as written.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl MimeEntity {
    /// Serialize as `Name: value\r\n...\r\n\r\nbody`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for (name, value) in &self.headers {
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(b": ");
            out.extend_from_slice(value.as_bytes());
            out.extend_from_slice(b"\r\n");
        }
        out.extend_from_slice(b"\r\n");
        out.extend_from_slice(&self.body);
        out
    }

    /// The value of the first header `name` (case-insensitive).
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

/// A parsed `Content-Type`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentType {
    /// `type/subtype`, lower-cased.
    pub mime: String,
    /// Parameters, names lower-cased, values unquoted.
    pub params: Vec<(String, String)>,
}

impl ContentType {
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// Parse a `Content-Type` value (RFC 2045 §5.1).
    pub fn parse(value: &str) -> ContentType {
        let mut parts = split_params(value);
        let mime = if parts.is_empty() {
            String::new()
        } else {
            parts.remove(0).trim().to_ascii_lowercase()
        };
        let params = parts
            .into_iter()
            .filter_map(|p| {
                let (n, v) = p.split_once('=')?;
                Some((n.trim().to_ascii_lowercase(), unquote(v.trim())))
            })
            .collect();
        ContentType { mime, params }
    }
}

/// Split a header value on `;` outside quoted strings.
fn split_params(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for c in value.chars() {
        if escaped {
            cur.push(c);
            escaped = false;
            continue;
        }
        match c {
            '\\' if quoted => {
                cur.push(c);
                escaped = true;
            }
            '"' => {
                quoted = !quoted;
                cur.push(c);
            }
            ';' if !quoted => out.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() || out.is_empty() {
        out.push(cur);
    }
    out
}

fn unquote(v: &str) -> String {
    if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
        let inner = &v[1..v.len() - 1];
        let mut out = String::with_capacity(inner.len());
        let mut escaped = false;
        for c in inner.chars() {
            if escaped {
                out.push(c);
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else {
                out.push(c);
            }
        }
        out
    } else {
        v.to_string()
    }
}

fn quote(v: &str) -> String {
    format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Where the header block ends: the index of the body start and of the separator.
fn header_end(raw: &[u8]) -> (usize, usize) {
    // A body may start right away when the entity has no headers.
    if raw.starts_with(b"\r\n") {
        return (0, 2);
    }
    if raw.starts_with(b"\n") {
        return (0, 1);
    }
    let crlf = find(raw, b"\r\n\r\n").map(|i| (i, i + 4));
    let lf = find(raw, b"\n\n").map(|i| (i, i + 2));
    match (crlf, lf) {
        (Some(a), Some(b)) => {
            if a.0 <= b.0 {
                a
            } else {
                b
            }
        }
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => (raw.len(), raw.len()),
    }
}

/// Parse a raw entity into headers and body (the body is the exact remaining bytes).
pub fn parse_entity(raw: &[u8]) -> MimeEntity {
    let (end, body_start) = header_end(raw);
    let block = String::from_utf8_lossy(&raw[..end]);
    MimeEntity {
        headers: parse_header_block(&block),
        body: raw[body_start.min(raw.len())..].to_vec(),
    }
}

/// Unfold and split an RFC 5322 header block into `(name, value)` pairs.
pub fn parse_header_block(block: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in block.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.is_empty() {
            continue;
        }
        if line.starts_with([' ', '\t']) {
            if let Some((_, v)) = out.last_mut() {
                v.push(' ');
                v.push_str(line.trim());
            }
            continue;
        }
        if let Some((n, v)) = line.split_once(':') {
            out.push((n.trim().to_string(), v.trim().to_string()));
        }
    }
    out
}

/// Every value of header `name` (case-insensitive) in a header block.
pub fn header_values(block: &str, name: &str) -> Vec<String> {
    parse_header_block(block)
        .into_iter()
        .filter(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, v)| v)
        .collect()
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

/// Split a multipart body into its parts' exact bytes (RFC 2046 §5.1.1): the line
/// break before each delimiter belongs to the delimiter, not to the part. Returns
/// `None` when the closing delimiter is missing.
pub fn split_multipart<'a>(body: &'a [u8], boundary: &str) -> Option<Vec<&'a [u8]>> {
    let delim = format!("--{boundary}");
    let d = delim.as_bytes();
    // Positions of delimiter lines: at the very start or right after a line break.
    let mut marks: Vec<(usize, usize, bool)> = Vec::new(); // (line start, content start, closing)
    let mut i = 0;
    while let Some(off) = find(&body[i..], d) {
        let pos = i + off;
        let at_line_start = pos == 0 || body[pos - 1] == b'\n';
        if at_line_start {
            let after = pos + d.len();
            let closing = body[after..].starts_with(b"--");
            // The delimiter line may carry trailing whitespace (transport padding).
            let mut eol = after + if closing { 2 } else { 0 };
            while eol < body.len() && (body[eol] == b' ' || body[eol] == b'\t') {
                eol += 1;
            }
            let content = if body[eol..].starts_with(b"\r\n") {
                eol + 2
            } else if body[eol..].starts_with(b"\n") {
                eol + 1
            } else {
                eol
            };
            // The line break before the delimiter belongs to it.
            let line_start = if pos >= 2 && &body[pos - 2..pos] == b"\r\n" {
                pos - 2
            } else if pos >= 1 && body[pos - 1] == b'\n' {
                pos - 1
            } else {
                pos
            };
            marks.push((line_start, content, closing));
            if closing {
                break;
            }
            i = content.max(pos + 1);
        } else {
            i = pos + 1;
        }
    }
    if !marks.last().is_some_and(|m| m.2) {
        return None;
    }
    let mut parts = Vec::new();
    for w in marks.windows(2) {
        let start = w[0].1;
        let end = w[1].0.max(start);
        parts.push(&body[start..end]);
    }
    Some(parts)
}

/// Line endings made CRLF (bare LF and bare CR both become CRLF), the canonical
/// form a signature covers (RFC 3156 §5, RFC 8551 §3.1.1).
pub fn canonicalize(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + data.len() / 16);
    let mut i = 0;
    while i < data.len() {
        match data[i] {
            b'\r' => {
                out.extend_from_slice(b"\r\n");
                if data.get(i + 1) == Some(&b'\n') {
                    i += 1;
                }
            }
            b'\n' => out.extend_from_slice(b"\r\n"),
            b => out.push(b),
        }
        i += 1;
    }
    out
}

/// Quoted-printable encode text for a signed part (RFC 2045 §6.7): lines stay at or
/// under 76 characters, trailing whitespace and a leading `From ` or `-` are
/// encoded so no transport rewrites what the signature covers (RFC 3156 §3).
pub fn quoted_printable(text: &str) -> String {
    let mut out = String::new();
    let normalized = String::from_utf8_lossy(&canonicalize(text.as_bytes())).into_owned();
    let lines: Vec<&str> = normalized.split("\r\n").collect();
    for (idx, line) in lines.iter().enumerate() {
        let bytes = line.as_bytes();
        let mut encoded: Vec<String> = Vec::with_capacity(bytes.len());
        for (j, &b) in bytes.iter().enumerate() {
            let last = j + 1 == bytes.len();
            let needs = b == b'='
                || !(b == b' ' || b == b'\t' || (33..=126).contains(&b))
                || ((b == b' ' || b == b'\t') && last)
                || (j == 0 && (b == b'-' || (b == b'F' && line.starts_with("From "))))
                || (j == 0 && b == b'.');
            if needs {
                encoded.push(format!("={b:02X}"));
            } else {
                encoded.push((b as char).to_string());
            }
        }
        // Soft line breaks at 75 characters, never inside an `=XX` triplet.
        let mut width = 0;
        for tok in encoded {
            if width + tok.len() > 75 {
                out.push_str("=\r\n");
                width = 0;
            }
            width += tok.len();
            out.push_str(&tok);
        }
        if idx + 1 < lines.len() {
            out.push_str("\r\n");
        }
    }
    out
}

/// Decode a body by its `Content-Transfer-Encoding` (base64, quoted-printable, or
/// identity for 7bit / 8bit / binary).
pub fn decode_body(cte: Option<&str>, body: &[u8]) -> Vec<u8> {
    match cte.map(|c| c.trim().to_ascii_lowercase()).as_deref() {
        Some("base64") => {
            let clean: Vec<u8> = body
                .iter()
                .copied()
                .filter(|b| !b.is_ascii_whitespace())
                .collect();
            base64::engine::general_purpose::STANDARD
                .decode(&clean)
                .unwrap_or_default()
        }
        Some("quoted-printable") => decode_qp(body),
        _ => body.to_vec(),
    }
}

fn decode_qp(body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len());
    let mut i = 0;
    while i < body.len() {
        if body[i] == b'=' {
            if body[i + 1..].starts_with(b"\r\n") {
                i += 3;
                continue;
            }
            if body[i + 1..].starts_with(b"\n") {
                i += 2;
                continue;
            }
            if i + 2 < body.len()
                && let Ok(v) =
                    u8::from_str_radix(std::str::from_utf8(&body[i + 1..i + 3]).unwrap_or(""), 16)
            {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(body[i]);
        i += 1;
    }
    out
}

fn base64_lines(der: &[u8]) -> Vec<u8> {
    let b64 = base64::engine::general_purpose::STANDARD.encode(der);
    let mut out = Vec::with_capacity(b64.len() + b64.len() / 38);
    for chunk in b64.as_bytes().chunks(76) {
        out.extend_from_slice(chunk);
        out.extend_from_slice(b"\r\n");
    }
    out
}

/// A fresh, unguessable MIME boundary.
pub fn new_boundary() -> String {
    let mut raw = [0u8; 18];
    crate::rng::fill_random(&mut raw);
    format!(
        "mw-{}",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw)
    )
}

// ── building ─────────────────────────────────────────────────────────────────

/// RFC 3156 §4: `multipart/encrypted` around an armored PGP message.
pub fn pgp_encrypted_entity(armored_ciphertext: &str) -> MimeEntity {
    let b = new_boundary();
    let body = format!(
        "This is an OpenPGP/MIME encrypted message (RFC 4880 and 3156)\r\n\
         --{b}\r\n\
         Content-Type: application/pgp-encrypted\r\n\
         Content-Description: PGP/MIME version identification\r\n\
         \r\n\
         Version: 1\r\n\
         \r\n\
         --{b}\r\n\
         Content-Type: application/octet-stream; name=\"encrypted.asc\"\r\n\
         Content-Description: OpenPGP encrypted message\r\n\
         Content-Disposition: inline; filename=\"encrypted.asc\"\r\n\
         \r\n\
         {}\r\n\
         --{b}--\r\n",
        String::from_utf8_lossy(&canonicalize(armored_ciphertext.trim_end().as_bytes()))
    );
    MimeEntity {
        headers: vec![
            ("MIME-Version".into(), "1.0".into()),
            (
                "Content-Type".into(),
                format!(
                    "multipart/encrypted; protocol=\"application/pgp-encrypted\"; boundary={}",
                    quote(&b)
                ),
            ),
        ],
        body: body.into_bytes(),
    }
}

/// RFC 3156 §5: `multipart/signed` around `signed_part` (a complete MIME entity,
/// already in the canonical form that was signed) and its armored detached
/// signature. `micalg` is e.g. `pgp-sha512` (see [`pgp_micalg`]).
pub fn pgp_signed_entity(signed_part: &[u8], armored_signature: &str, micalg: &str) -> MimeEntity {
    let sig_part = format!(
        "Content-Type: application/pgp-signature; name=\"signature.asc\"\r\n\
         Content-Description: OpenPGP digital signature\r\n\
         Content-Disposition: attachment; filename=\"signature.asc\"\r\n\
         \r\n\
         {}\r\n",
        String::from_utf8_lossy(&canonicalize(armored_signature.trim_end().as_bytes()))
    );
    signed_multipart(
        signed_part,
        sig_part.as_bytes(),
        "application/pgp-signature",
        micalg,
    )
}

/// RFC 8551 §3.5.3: `multipart/signed` around `signed_part` and a DER CMS
/// SignedData without content. `micalg` is e.g. `sha-256` (see [`smime_micalg`]).
pub fn smime_signed_entity(signed_part: &[u8], der_signature: &[u8], micalg: &str) -> MimeEntity {
    let mut sig_part = b"Content-Type: application/pkcs7-signature; name=\"smime.p7s\"\r\n\
         Content-Transfer-Encoding: base64\r\n\
         Content-Disposition: attachment; filename=\"smime.p7s\"\r\n\
         Content-Description: S/MIME Cryptographic Signature\r\n\
         \r\n"
        .to_vec();
    sig_part.extend_from_slice(&base64_lines(der_signature));
    signed_multipart(
        signed_part,
        &sig_part,
        "application/pkcs7-signature",
        micalg,
    )
}

fn signed_multipart(
    signed_part: &[u8],
    sig_part: &[u8],
    protocol: &str,
    micalg: &str,
) -> MimeEntity {
    let b = new_boundary();
    let mut body = Vec::new();
    body.extend_from_slice(b"This is a cryptographically signed message in MIME format.\r\n");
    body.extend_from_slice(format!("--{b}\r\n").as_bytes());
    body.extend_from_slice(signed_part);
    body.extend_from_slice(format!("\r\n--{b}\r\n").as_bytes());
    body.extend_from_slice(sig_part);
    body.extend_from_slice(format!("--{b}--\r\n").as_bytes());
    MimeEntity {
        headers: vec![
            ("MIME-Version".into(), "1.0".into()),
            (
                "Content-Type".into(),
                format!(
                    "multipart/signed; protocol={}; micalg={}; boundary={}",
                    quote(protocol),
                    micalg,
                    quote(&b)
                ),
            ),
        ],
        body,
    }
}

/// RFC 8551 §3.2: an `application/pkcs7-mime` entity holding DER CMS of
/// `smime_type` (`signed-data`, `enveloped-data`, `authEnveloped-data`).
pub fn smime_opaque_entity(der: &[u8], smime_type: &str) -> MimeEntity {
    let name = "smime.p7m";
    MimeEntity {
        headers: vec![
            ("MIME-Version".into(), "1.0".into()),
            (
                "Content-Type".into(),
                format!("application/pkcs7-mime; smime-type={smime_type}; name=\"{name}\""),
            ),
            ("Content-Transfer-Encoding".into(), "base64".into()),
            (
                "Content-Disposition".into(),
                format!("attachment; filename=\"{name}\""),
            ),
        ],
        body: base64_lines(der),
    }
}

/// The RFC 3156 `micalg` value for an armored or binary OpenPGP signature
/// (`pgp-sha256`, `pgp-sha512`, ...).
pub fn pgp_micalg(signature: &[u8]) -> Result<String> {
    use pgp::packet::{Packet, PacketParser};
    use std::io::Read;
    let bytes = if signature.starts_with(b"-----BEGIN") {
        let mut d = pgp::armor::Dearmor::new(signature);
        let mut out = Vec::new();
        d.read_to_end(&mut out)
            .map_err(|e| CryptoError::Parse(e.to_string()))?;
        out
    } else {
        signature.to_vec()
    };
    for p in PacketParser::new(&bytes[..]) {
        if let Ok(Packet::Signature(s)) = p
            && let Some(h) = s.hash_alg()
        {
            use pgp::crypto::hash::HashAlgorithm as H;
            let name = match h {
                H::Sha1 => "sha1",
                H::Sha224 => "sha224",
                H::Sha256 => "sha256",
                H::Sha384 => "sha384",
                H::Sha512 => "sha512",
                H::Sha3_256 => "sha3-256",
                H::Sha3_512 => "sha3-512",
                _ => "sha256",
            };
            return Ok(format!("pgp-{name}"));
        }
    }
    Err(CryptoError::Parse("no signature packet".into()))
}

/// The RFC 8551 §3.5.3.2 `micalg` value for a digest name (`sha-256` ...).
pub fn smime_micalg(digest: &str) -> String {
    match digest.to_ascii_lowercase().replace('-', "").as_str() {
        "sha384" => "sha-384".into(),
        "sha512" => "sha-512".into(),
        "sha1" => "sha-1".into(),
        _ => "sha-256".into(),
    }
}

// ── reading ──────────────────────────────────────────────────────────────────

/// One cryptographic structure found in a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoPart {
    /// RFC 3156 `multipart/encrypted`: the armored (or binary) PGP message.
    PgpEncrypted { ciphertext: Vec<u8> },
    /// RFC 3156 `multipart/signed`: the signed part's exact bytes (CRLF) and the
    /// signature part's decoded content.
    PgpSigned {
        signed: Vec<u8>,
        signature: Vec<u8>,
        micalg: Option<String>,
    },
    /// RFC 8551 `multipart/signed` (detached CMS).
    SmimeSigned {
        signed: Vec<u8>,
        signature: Vec<u8>,
        micalg: Option<String>,
    },
    /// RFC 8551 `application/pkcs7-mime` (opaque signed, enveloped or
    /// authenveloped). `smime_type` is the parameter as written (lower-cased), or
    /// empty when absent.
    SmimeOpaque { der: Vec<u8>, smime_type: String },
    /// Armored OpenPGP blocks inside a text body.
    InlinePgp { segments: Vec<InlineSegment> },
}

/// A piece of a text body that may hold inline OpenPGP armor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum InlineSegment {
    /// Plain text outside any armor (unsigned, unencrypted).
    Text { text: String },
    /// `-----BEGIN PGP MESSAGE-----` ... (encrypted or signed message).
    Message { armored: String },
    /// `-----BEGIN PGP SIGNED MESSAGE-----` ... (cleartext signature).
    Signed { armored: String },
    /// `-----BEGIN PGP PUBLIC KEY BLOCK-----` ...
    PublicKey { armored: String },
}

/// What [`analyze`] found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis {
    /// The outermost cryptographic structure, if any.
    pub part: Option<CryptoPart>,
    /// The structure covers only part of the message: there are other parts beside
    /// it (for example an unsigned footer or attachment added by a list server),
    /// or inline armor is surrounded by plain text.
    pub partial: bool,
    /// Child indexes from the root to the structure (empty = the root itself).
    pub path: Vec<usize>,
}

/// Locate the cryptographic structure of a raw message (or entity).
pub fn analyze(raw: &[u8]) -> Analysis {
    let mut path = Vec::new();
    match find_crypto(raw, 0, &mut path) {
        Some((part, siblings)) => {
            let partial = siblings
                || matches!(&part, CryptoPart::InlinePgp { segments }
                    if segments.iter().any(|s| matches!(s, InlineSegment::Text { text } if !text.trim().is_empty())));
            Analysis {
                part: Some(part),
                partial,
                path,
            }
        }
        None => Analysis {
            part: None,
            partial: false,
            path: Vec::new(),
        },
    }
}

/// Depth-first search for the first crypto structure. Returns it and whether a
/// sibling with content exists anywhere on the way down.
fn find_crypto(raw: &[u8], depth: usize, path: &mut Vec<usize>) -> Option<(CryptoPart, bool)> {
    if depth > MAX_DEPTH {
        return None;
    }
    let entity = parse_entity(raw);
    let ct = ContentType::parse(entity.header("content-type").unwrap_or("text/plain"));
    let cte = entity.header("content-transfer-encoding");
    match ct.mime.as_str() {
        "multipart/encrypted" => {
            let parts = split_multipart(&entity.body, ct.param("boundary")?)?;
            let protocol = ct.param("protocol").unwrap_or("").to_ascii_lowercase();
            if protocol == "application/pgp-encrypted" && parts.len() >= 2 {
                let data = parse_entity(parts[1]);
                let ciphertext = decode_body(data.header("content-transfer-encoding"), &data.body);
                return Some((CryptoPart::PgpEncrypted { ciphertext }, false));
            }
            None
        }
        "multipart/signed" => {
            let parts = split_multipart(&entity.body, ct.param("boundary")?)?;
            if parts.len() < 2 {
                return None;
            }
            let protocol = ct.param("protocol").unwrap_or("").to_ascii_lowercase();
            let sig = parse_entity(parts[1]);
            let signature = decode_body(sig.header("content-transfer-encoding"), &sig.body);
            let signed = canonicalize(parts[0]);
            let micalg = ct.param("micalg").map(str::to_ascii_lowercase);
            match protocol.as_str() {
                "application/pgp-signature" => Some((
                    CryptoPart::PgpSigned {
                        signed,
                        signature,
                        micalg,
                    },
                    false,
                )),
                "application/pkcs7-signature" | "application/x-pkcs7-signature" => Some((
                    CryptoPart::SmimeSigned {
                        signed,
                        signature,
                        micalg,
                    },
                    false,
                )),
                _ => None,
            }
        }
        "application/pkcs7-mime" | "application/x-pkcs7-mime" => {
            let der = decode_body(cte, &entity.body);
            let smime_type = ct.param("smime-type").unwrap_or("").to_ascii_lowercase();
            Some((CryptoPart::SmimeOpaque { der, smime_type }, false))
        }
        "application/pkcs7-signature" | "application/x-pkcs7-signature" => None,
        m if m.starts_with("multipart/") => {
            let parts = split_multipart(&entity.body, ct.param("boundary")?)?;
            let non_empty = parts
                .iter()
                .filter(|p| !parse_entity(p).body.iter().all(u8::is_ascii_whitespace))
                .count();
            for (i, p) in parts.iter().enumerate() {
                path.push(i);
                if let Some((found, siblings)) = find_crypto(p, depth + 1, path) {
                    // `multipart/alternative` offers the same content twice; a crypto
                    // part beside a plain alternative is still the whole message
                    // only if the plain one is empty, so count siblings for all.
                    return Some((found, siblings || non_empty > 1));
                }
                path.pop();
            }
            None
        }
        m if m.starts_with("text/") || m.is_empty() => {
            let text = decode_body(cte, &entity.body);
            let text = String::from_utf8_lossy(&text);
            let segments = inline_segments(&text);
            if segments
                .iter()
                .any(|s| !matches!(s, InlineSegment::Text { .. }))
            {
                Some((CryptoPart::InlinePgp { segments }, false))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Split text into inline OpenPGP armor blocks and the plain text around them.
pub fn inline_segments(text: &str) -> Vec<InlineSegment> {
    const KINDS: [(&str, &str); 3] = [
        (
            "-----BEGIN PGP SIGNED MESSAGE-----",
            "-----END PGP SIGNATURE-----",
        ),
        ("-----BEGIN PGP MESSAGE-----", "-----END PGP MESSAGE-----"),
        (
            "-----BEGIN PGP PUBLIC KEY BLOCK-----",
            "-----END PGP PUBLIC KEY BLOCK-----",
        ),
    ];
    let mut out = Vec::new();
    let mut rest = text;
    loop {
        // The earliest BEGIN line that starts a line.
        let next = KINDS
            .iter()
            .enumerate()
            .filter_map(|(k, (begin, _))| {
                let mut from = 0;
                while let Some(off) = rest[from..].find(begin) {
                    let pos = from + off;
                    if pos == 0 || rest.as_bytes()[pos - 1] == b'\n' {
                        return Some((pos, k));
                    }
                    from = pos + 1;
                }
                None
            })
            .min_by_key(|(pos, _)| *pos);
        let Some((start, k)) = next else {
            if !rest.is_empty() {
                out.push(InlineSegment::Text {
                    text: rest.to_string(),
                });
            }
            break;
        };
        let (_, end_marker) = KINDS[k];
        let Some(end_off) = rest[start..].find(end_marker) else {
            out.push(InlineSegment::Text {
                text: rest.to_string(),
            });
            break;
        };
        let end = start + end_off + end_marker.len();
        if start > 0 {
            out.push(InlineSegment::Text {
                text: rest[..start].to_string(),
            });
        }
        let armored = rest[start..end].to_string();
        out.push(match k {
            0 => InlineSegment::Signed { armored },
            1 => InlineSegment::Message { armored },
            _ => InlineSegment::PublicKey { armored },
        });
        rest = &rest[end..];
        // Keep the line break that ended the armor with the armor.
        if let Some(r) = rest.strip_prefix("\r\n") {
            rest = r;
        } else if let Some(r) = rest.strip_prefix('\n') {
            rest = r;
        }
    }
    out
}

// ── header protection (RFC 9788) ─────────────────────────────────────────────

/// The header fields protected by default (RFC 9788 §2.1 and §4.1).
pub const PROTECTED_FIELDS: [&str; 9] = [
    "Subject",
    "From",
    "To",
    "Cc",
    "Reply-To",
    "Date",
    "Message-ID",
    "In-Reply-To",
    "References",
];

/// What the outer (unprotected) Subject is replaced with on an encrypted message
/// (RFC 9788 §4.2.1).
pub const OBSCURED_SUBJECT: &str = "[...]";

/// Whether the payload will be encrypted (`hp="cipher"`) or only signed
/// (`hp="clear"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Protection {
    Cipher,
    Clear,
}

/// Header protection read back from a payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtectedHeaders {
    /// The protected header fields, in order.
    pub headers: Vec<(String, String)>,
    /// `"cipher"` | `"clear"` (RFC 9788), or `"v1"` (the older
    /// `protected-headers="v1"`, which is also what Mailwoman 26.19 wrote).
    pub scheme: String,
    /// `HP-Outer` fields: what the outer header said for each obscured field.
    pub outer: Vec<(String, String)>,
    /// The payload with the protected fields kept and the legacy display element
    /// removed — what the reader renders.
    pub entity: String,
}

/// Add RFC 9788 header protection to a payload entity (the MIME entity that will
/// be signed and/or encrypted). `headers` are the message's real header fields;
/// `outer` the values the outer header will carry for fields that differ (e.g.
/// `Subject: [...]` when encrypting), written as `HP-Outer`. With
/// `legacy_display` (meant for `Protection::Cipher`), the protected Subject and
/// addresses are also shown in the body for readers without header protection
/// support: prepended to a top-level `text/plain` body, or as a first
/// `text/plain; hp-legacy-display="1"` part otherwise.
pub fn protect_headers(
    payload: &[u8],
    headers: &[(String, String)],
    outer: &[(String, String)],
    protection: Protection,
    legacy_display: bool,
) -> Vec<u8> {
    let mut entity = parse_entity(payload);
    let protected: Vec<(String, String)> = headers
        .iter()
        .filter(|(n, _)| PROTECTED_FIELDS.iter().any(|p| p.eq_ignore_ascii_case(n)))
        .cloned()
        .collect();

    if legacy_display {
        let lines: String = protected
            .iter()
            .filter(|(n, _)| {
                ["subject", "from", "to", "cc"].contains(&n.to_ascii_lowercase().as_str())
            })
            .map(|(n, v)| format!("{n}: {v}\r\n"))
            .collect();
        let ct = ContentType::parse(entity.header("content-type").unwrap_or("text/plain"));
        let cte = entity
            .header("content-transfer-encoding")
            .map(str::to_ascii_lowercase);
        let plain_identity =
            ct.mime == "text/plain" && matches!(cte.as_deref(), None | Some("7bit") | Some("8bit"));
        if plain_identity {
            let mut body = lines.into_bytes();
            body.extend_from_slice(b"\r\n");
            body.extend_from_slice(&entity.body);
            entity.body = body;
            let mut ct_value = entity
                .header("content-type")
                .unwrap_or("text/plain; charset=utf-8")
                .to_string();
            ct_value.push_str("; hp-legacy-display=\"1\"");
            set_header(&mut entity.headers, "Content-Type", &ct_value);
        } else {
            let b = new_boundary();
            let mut body = Vec::new();
            body.extend_from_slice(format!("--{b}\r\n").as_bytes());
            body.extend_from_slice(
                b"Content-Type: text/plain; charset=utf-8; hp-legacy-display=\"1\"\r\n\r\n",
            );
            body.extend_from_slice(lines.as_bytes());
            body.extend_from_slice(format!("\r\n--{b}\r\n").as_bytes());
            body.extend_from_slice(&entity.to_bytes());
            body.extend_from_slice(format!("\r\n--{b}--\r\n").as_bytes());
            entity = MimeEntity {
                headers: vec![(
                    "Content-Type".into(),
                    format!("multipart/mixed; boundary={}", quote(&b)),
                )],
                body,
            };
        }
    }

    // `hp` goes on the payload's Content-Type.
    let ct_value = entity
        .header("content-type")
        .unwrap_or("text/plain; charset=utf-8")
        .to_string();
    let hp = match protection {
        Protection::Cipher => "cipher",
        Protection::Clear => "clear",
    };
    set_header(
        &mut entity.headers,
        "Content-Type",
        &format!("{ct_value}; hp=\"{hp}\""),
    );

    let mut new_headers: Vec<(String, String)> = protected;
    for (n, v) in outer {
        new_headers.push(("HP-Outer".into(), format!("{n}: {v}")));
    }
    new_headers.extend(
        entity
            .headers
            .into_iter()
            .filter(|(n, _)| !PROTECTED_FIELDS.iter().any(|p| p.eq_ignore_ascii_case(n))),
    );
    entity.headers = new_headers;
    entity.to_bytes()
}

fn set_header(headers: &mut Vec<(String, String)>, name: &str, value: &str) {
    if let Some(h) = headers
        .iter_mut()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
    {
        h.1 = value.to_string();
    } else {
        headers.push((name.to_string(), value.to_string()));
    }
}

/// Read header protection from a decrypted or signed payload. `None` when the
/// payload carries none.
pub fn unprotect_headers(payload: &[u8]) -> Option<ProtectedHeaders> {
    let entity = parse_entity(payload);
    let ct = ContentType::parse(entity.header("content-type")?);
    let scheme = match (ct.param("hp"), ct.param("protected-headers")) {
        (Some(hp), _) => hp.to_ascii_lowercase(),
        (None, Some(v)) => v.to_ascii_lowercase(),
        _ => return None,
    };
    let headers: Vec<(String, String)> = entity
        .headers
        .iter()
        .filter(|(n, _)| PROTECTED_FIELDS.iter().any(|p| p.eq_ignore_ascii_case(n)))
        .cloned()
        .collect();
    let outer: Vec<(String, String)> = entity
        .headers
        .iter()
        .filter(|(n, _)| n.eq_ignore_ascii_case("hp-outer"))
        .filter_map(|(_, v)| {
            v.split_once(':')
                .map(|(a, b)| (a.trim().to_string(), b.trim().to_string()))
        })
        .collect();
    let rendered = strip_legacy_display(&entity, &ct, 0);
    Some(ProtectedHeaders {
        headers,
        scheme,
        outer,
        entity: String::from_utf8_lossy(&rendered).into_owned(),
    })
}

/// Remove the legacy display element: a first `hp-legacy-display="1"` part of a
/// `multipart/mixed`, or the header lines at the top of a `text/plain` body so
/// marked.
fn strip_legacy_display(entity: &MimeEntity, ct: &ContentType, depth: usize) -> Vec<u8> {
    if depth > MAX_DEPTH {
        return entity.to_bytes();
    }
    if ct.mime == "text/plain" && ct.param("hp-legacy-display") == Some("1") {
        let body = &entity.body;
        let cut = find(body, b"\r\n\r\n")
            .map(|i| i + 4)
            .or_else(|| find(body, b"\n\n").map(|i| i + 2))
            .unwrap_or(0);
        let mut out = entity.clone();
        out.body = body[cut..].to_vec();
        return out.to_bytes();
    }
    if ct.mime == "multipart/mixed"
        && let Some(b) = ct.param("boundary")
        && let Some(parts) = split_multipart(&entity.body, b)
        && let Some(first) = parts.first()
    {
        let first_entity = parse_entity(first);
        let first_ct = ContentType::parse(first_entity.header("content-type").unwrap_or(""));
        if first_ct.param("hp-legacy-display") == Some("1") {
            let rest: Vec<&[u8]> = parts[1..].to_vec();
            if rest.len() == 1 {
                return rest[0].to_vec();
            }
            let nb = new_boundary();
            let mut body = Vec::new();
            for p in rest {
                body.extend_from_slice(format!("--{nb}\r\n").as_bytes());
                body.extend_from_slice(p);
                body.extend_from_slice(b"\r\n");
            }
            body.extend_from_slice(format!("--{nb}--\r\n").as_bytes());
            let mut out = entity.clone();
            set_header(
                &mut out.headers,
                "Content-Type",
                &format!("multipart/mixed; boundary={}", quote(&nb)),
            );
            out.body = body;
            return out.to_bytes();
        }
    }
    entity.to_bytes()
}

/// Fold a header field for transmission (RFC 5322 §2.2.3): lines of at most 78
/// characters, broken at spaces. A word too long for any line (Autocrypt
/// `keydata`) is split across continuation lines, which Autocrypt allows because
/// whitespace inside `keydata` is ignored.
pub fn fold_header(name: &str, value: &str) -> String {
    let mut out = format!("{name}:");
    let mut width = out.len();
    for word in value.split_whitespace() {
        if width + 1 + word.len() <= 78 {
            out.push(' ');
            out.push_str(word);
            width += 1 + word.len();
        } else if word.len() < 78 || !word.is_ascii() {
            out.push_str("\r\n ");
            out.push_str(word);
            width = 1 + word.len();
        } else {
            for chunk in word.as_bytes().chunks(76) {
                out.push_str("\r\n ");
                out.push_str(std::str::from_utf8(chunk).unwrap_or_default());
                width = 1 + chunk.len();
            }
        }
    }
    out
}
