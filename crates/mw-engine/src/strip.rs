//! Rewrite a message without its attachment parts (the retention "strip
//! attachments" action).
//!
//! The rewrite is a byte splice, not a re-serialisation: every header and every
//! part that is kept stays byte for byte as it was, so `Message-ID`, `Date`,
//! `References`, DKIM-covered headers on kept parts and the multipart structure
//! survive. Each removed attachment part is replaced by a small `text/plain`
//! part that says what was there, and one `X-Mailwoman-Stripped` header per
//! removed part is added to the top-level headers. That header is also the
//! idempotence marker: a message that carries it is never processed again.
//!
//! Messages whose bytes are covered by a signature or a ciphertext (S/MIME,
//! PGP/MIME, inline PGP) are never rewritten: the result would be corrupt.

use std::collections::HashMap;

/// Header recording a removed part; its presence marks a message as stripped.
pub const MARKER_HEADER: &str = "X-Mailwoman-Stripped";
/// Keyword the retention job puts on a stripped copy so it can be skipped
/// without fetching the message.
pub const MARKER_KEYWORD: &str = "mw-stripped";

const MAX_RAW: usize = 128 * 1024 * 1024;
const MAX_DEPTH: usize = 16;
const MAX_PARTS: usize = 2000;

#[derive(Debug, Clone)]
pub struct StripOptions {
    /// Only attachments whose decoded size is at least this many bytes go.
    pub min_bytes: u64,
    /// Keep inline images (those shown inside the body).
    pub keep_inline_images: bool,
    /// `YYYY-MM-DD`, written into the placeholder and the marker header.
    pub date: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Removed {
    /// Position in the MIME tree, e.g. `2` or `1.3`.
    pub path: String,
    pub name: String,
    pub content_type: String,
    /// Decoded size in bytes.
    pub bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// A signature covers the bytes (S/MIME, PGP/MIME).
    Signed,
    /// The content is encrypted, or inline PGP is present.
    Encrypted,
    AlreadyStripped,
    Unparsable,
    TooLarge,
}

impl SkipReason {
    pub fn as_str(self) -> &'static str {
        match self {
            SkipReason::Signed => "signed",
            SkipReason::Encrypted => "encrypted",
            SkipReason::AlreadyStripped => "alreadyStripped",
            SkipReason::Unparsable => "unparsable",
            SkipReason::TooLarge => "tooLarge",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StripOutcome {
    /// Nothing qualified; the message stays as it is.
    Unchanged,
    /// Left alone on purpose.
    Skipped(SkipReason),
    Stripped { raw: Vec<u8>, removed: Vec<Removed> },
}

/// Rebuild `raw` without the attachments `opts` selects.
pub fn strip_attachments(raw: &[u8], opts: &StripOptions) -> StripOutcome {
    if raw.len() > MAX_RAW {
        return StripOutcome::Skipped(SkipReason::TooLarge);
    }
    let top = split_headers(raw);
    if top.headers.iter().any(|(n, _)| n == "x-mailwoman-stripped") {
        return StripOutcome::Skipped(SkipReason::AlreadyStripped);
    }
    let mut ctx = Ctx {
        opts,
        eol: if raw.windows(2).any(|w| w == b"\r\n") { "\r\n" } else { "\n" },
        removed: Vec::new(),
        parts: 0,
    };
    match walk(raw, "", 0, &mut ctx) {
        Err(reason) => StripOutcome::Skipped(reason),
        Ok(None) => StripOutcome::Unchanged,
        Ok(Some(mut out)) => {
            let at = split_headers(&out).blank;
            let mut extra = String::new();
            for r in &ctx.removed {
                extra.push_str(&format!(
                    "{MARKER_HEADER}: {}; part={}; type={}; size={}; name={}{}",
                    opts.date,
                    r.path,
                    r.content_type,
                    r.bytes,
                    pct(&r.name),
                    ctx.eol
                ));
            }
            out.splice(at..at, extra.into_bytes());
            StripOutcome::Stripped { raw: out, removed: ctx.removed }
        }
    }
}

struct Ctx<'a> {
    opts: &'a StripOptions,
    eol: &'static str,
    removed: Vec<Removed>,
    parts: usize,
}

struct Head {
    headers: Vec<(String, String)>,
    /// Offset of the blank line that ends the headers (or the buffer length).
    blank: usize,
    /// Offset of the first body byte.
    body: usize,
}

fn split_headers(buf: &[u8]) -> Head {
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut pos = 0;
    while pos < buf.len() {
        let end = buf[pos..].iter().position(|b| *b == b'\n').map_or(buf.len(), |i| pos + i);
        let next = (end + 1).min(buf.len());
        let mut line = &buf[pos..end];
        if line.last() == Some(&b'\r') {
            line = &line[..line.len() - 1];
        }
        if line.is_empty() {
            return Head { headers, blank: pos, body: next };
        }
        if line[0] == b' ' || line[0] == b'\t' {
            if let Some(last) = headers.last_mut() {
                last.1.push(' ');
                last.1.push_str(String::from_utf8_lossy(line).trim());
            }
        } else if let Some(colon) = line.iter().position(|b| *b == b':') {
            let name = String::from_utf8_lossy(&line[..colon]).trim().to_ascii_lowercase();
            let value = String::from_utf8_lossy(&line[colon + 1..]).trim().to_string();
            headers.push((name, value));
        }
        pos = next;
    }
    Head { headers, blank: buf.len(), body: buf.len() }
}

fn header<'a>(h: &'a Head, name: &str) -> Option<&'a str> {
    h.headers.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
}

/// `(value, parameters)` of a structured header, parameter names lower-case,
/// RFC 2231 continuations and extended values decoded.
fn structured(v: &str) -> (String, HashMap<String, String>) {
    let mut pieces: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut prev_backslash = false;
    for c in v.chars() {
        if c == '"' && !prev_backslash {
            quoted = !quoted;
        }
        if c == ';' && !quoted {
            pieces.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
        prev_backslash = c == '\\' && !prev_backslash;
    }
    pieces.push(cur);
    let main = pieces.remove(0).trim().to_ascii_lowercase();
    // name -> [(index, extended, raw)]
    let mut raw: HashMap<String, Vec<(usize, bool, String)>> = HashMap::new();
    for p in pieces {
        let Some((k, val)) = p.split_once('=') else { continue };
        let k = k.trim().to_ascii_lowercase();
        let val = val.trim();
        let val = if val.len() >= 2 && val.starts_with('"') && val.ends_with('"') {
            val[1..val.len() - 1].replace("\\\"", "\"").replace("\\\\", "\\")
        } else {
            val.to_string()
        };
        let (base, idx, ext) = match k.split_once('*') {
            None => (k.clone(), 0, false),
            Some((b, rest)) => {
                let ext = rest.ends_with('*');
                let digits = rest.trim_end_matches('*');
                (b.to_string(), digits.parse().unwrap_or(0), ext || digits.is_empty())
            }
        };
        raw.entry(base).or_default().push((idx, ext, val));
    }
    let mut out = HashMap::new();
    for (k, mut parts) in raw {
        parts.sort_by_key(|p| p.0);
        let extended = parts.iter().any(|p| p.1);
        let mut joined = String::new();
        let mut charset = String::new();
        for (i, (_, ext, val)) in parts.iter().enumerate() {
            let mut piece = val.as_str();
            if i == 0 && *ext {
                // charset'language'value
                let mut it = piece.splitn(3, '\'');
                if let (Some(cs), Some(_), Some(rest)) = (it.next(), it.next(), it.next()) {
                    charset = cs.to_ascii_lowercase();
                    piece = rest;
                }
            }
            joined.push_str(piece);
        }
        let value = if extended { decode_charset(&pct_decode(&joined), &charset) } else { joined };
        out.insert(k, value);
    }
    (main, out)
}

fn pct_decode(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

fn hex(c: u8) -> Option<u8> {
    (c as char).to_digit(16).map(|d| d as u8)
}

fn decode_charset(bytes: &[u8], charset: &str) -> String {
    match charset {
        "iso-8859-1" | "latin1" | "us-ascii" | "windows-1252" => bytes.iter().map(|b| *b as char).collect(),
        _ => String::from_utf8_lossy(bytes).into_owned(),
    }
}

/// Decode RFC 2047 encoded words in a header value (utf-8 and latin-1 only).
fn decode_words(v: &str) -> String {
    let mut out = String::new();
    let mut rest = v;
    let mut last_was_word = false;
    while let Some(start) = rest.find("=?") {
        let (before, after) = rest.split_at(start);
        let parts: Vec<&str> = after[2..].splitn(4, '?').collect();
        let decoded = (parts.len() == 4)
            .then(|| {
                let tail = parts[3];
                tail.find("?=").map(|e| (parts[0], parts[1], &tail[..e], &tail[e + 2..]))
            })
            .flatten();
        match decoded {
            Some((cs, enc, text, remainder)) => {
                let bytes = if enc.eq_ignore_ascii_case("b") {
                    b64_decode(text)
                } else {
                    q_decode(text)
                };
                if !(last_was_word && before.trim().is_empty()) {
                    out.push_str(before);
                }
                out.push_str(&decode_charset(&bytes, &cs.to_ascii_lowercase()));
                rest = remainder;
                last_was_word = true;
            }
            None => {
                out.push_str(before);
                out.push_str("=?");
                rest = &after[2..];
                last_was_word = false;
            }
        }
    }
    out.push_str(rest);
    out
}

fn q_decode(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'_' => out.push(b' '),
            b'=' if i + 2 < b.len() => {
                if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                    out.push(h * 16 + l);
                    i += 2;
                } else {
                    out.push(b'=');
                }
            }
            c => out.push(c),
        }
        i += 1;
    }
    out
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64_decode(s: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in s.bytes() {
        let Some(v) = B64.iter().position(|x| *x == c) else { continue };
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    out
}

fn b64_encode(data: &[u8], eol: &str) -> String {
    let mut out = String::new();
    let mut line = 0;
    for chunk in data.chunks(3) {
        let n = (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(B64[((n >> (18 - 6 * i)) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
        line += 4;
        if line >= 76 {
            out.push_str(eol);
            line = 0;
        }
    }
    out
}

/// Percent-encode for the marker header so it stays ASCII and one token.
fn pct(s: &str) -> String {
    let mut out = String::new();
    for (i, b) in s.bytes().enumerate() {
        if i >= 120 {
            break;
        }
        if b.is_ascii_alphanumeric() || b"._-".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn decoded_size(h: &Head, body: &[u8]) -> u64 {
    match header(h, "content-transfer-encoding").map(|v| v.to_ascii_lowercase()).as_deref() {
        Some("base64") => {
            let chars = body.iter().filter(|b| !b.is_ascii_whitespace()).count() as u64;
            let pad = body.iter().rev().filter(|b| !b.is_ascii_whitespace()).take_while(|b| **b == b'=').count() as u64;
            (chars / 4 * 3).saturating_sub(pad)
        }
        Some("quoted-printable") => {
            let eq = body.iter().filter(|b| **b == b'=').count() as u64;
            (body.len() as u64).saturating_sub(eq * 2)
        }
        _ => body.len() as u64,
    }
}

fn human(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{bytes} B")
    }
}

fn walk(buf: &[u8], path: &str, depth: usize, ctx: &mut Ctx) -> Result<Option<Vec<u8>>, SkipReason> {
    if depth > MAX_DEPTH {
        return Err(SkipReason::Unparsable);
    }
    ctx.parts += 1;
    if ctx.parts > MAX_PARTS {
        return Err(SkipReason::Unparsable);
    }
    let head = split_headers(buf);
    let (ty, params) = match header(&head, "content-type") {
        Some(v) => structured(v),
        None => ("text/plain".to_string(), HashMap::new()),
    };
    match ty.as_str() {
        "multipart/signed" | "application/pgp-signature" => return Err(SkipReason::Signed),
        "multipart/encrypted" | "application/pgp-encrypted" => return Err(SkipReason::Encrypted),
        "application/pkcs7-mime" | "application/x-pkcs7-mime" => {
            return Err(if params.get("smime-type").is_some_and(|t| t == "signed-data") {
                SkipReason::Signed
            } else {
                SkipReason::Encrypted
            });
        }
        "application/pkcs7-signature" | "application/x-pkcs7-signature" => return Err(SkipReason::Signed),
        _ => {}
    }
    let body = &buf[head.body..];

    if ty.starts_with("multipart/") {
        let boundary = params.get("boundary").filter(|b| !b.is_empty()).ok_or(SkipReason::Unparsable)?;
        let split = split_multipart(body, boundary);
        let mut changed = false;
        let mut out_parts: Vec<Vec<u8>> = Vec::with_capacity(split.parts.len());
        for (i, p) in split.parts.iter().enumerate() {
            let child = if path.is_empty() { format!("{}", i + 1) } else { format!("{path}.{}", i + 1) };
            match walk(p.content, &child, depth + 1, ctx)? {
                Some(new) => {
                    changed = true;
                    out_parts.push(new);
                }
                None => out_parts.push(p.content.to_vec()),
            }
        }
        if !changed {
            return Ok(None);
        }
        let mut out = Vec::with_capacity(buf.len());
        out.extend_from_slice(&buf[..head.body]);
        out.extend_from_slice(split.head);
        for (p, new) in split.parts.iter().zip(out_parts) {
            out.extend_from_slice(p.delimiter);
            out.extend_from_slice(&new);
            out.extend_from_slice(p.eol);
        }
        out.extend_from_slice(split.tail);
        return Ok(Some(out));
    }

    // A leaf.
    if ty == "text/plain" || ty == "text/html" {
        let disp = header(&head, "content-disposition").map(|v| structured(v).0);
        if disp.as_deref() != Some("attachment") {
            if body.windows(15).any(|w| w == b"-----BEGIN PGP ") {
                return Err(SkipReason::Encrypted);
            }
        }
    }
    if path.is_empty() {
        // A message that is itself one attachment: leave it whole.
        return Ok(None);
    }
    let (disp, dparams) = header(&head, "content-disposition").map(structured).unwrap_or_default();
    let name_raw = dparams.get("filename").or_else(|| params.get("name")).cloned();
    let attachment = classify(&ty, &disp, name_raw.is_some(), header(&head, "content-id").is_some(), ctx.opts.keep_inline_images);
    if !attachment {
        return Ok(None);
    }
    let bytes = decoded_size(&head, body);
    if bytes < ctx.opts.min_bytes {
        return Ok(None);
    }
    let name = name_raw.map(|n| decode_words(&n)).unwrap_or_default();
    let shown = if name.is_empty() { "unnamed".to_string() } else { name.clone() };
    let text = format!(
        "Attachment removed by retention rule on {}: {} ({}, {})",
        ctx.opts.date,
        shown,
        ty,
        human(bytes)
    );
    ctx.removed.push(Removed { path: path.to_string(), name: shown, content_type: ty, bytes });
    let eol = ctx.eol;
    let mut out = format!("Content-Type: text/plain; charset=utf-8{eol}");
    if text.is_ascii() {
        out.push_str(&format!("Content-Transfer-Encoding: 7bit{eol}{eol}{text}"));
    } else {
        let enc = b64_encode(text.as_bytes(), eol);
        out.push_str(&format!("Content-Transfer-Encoding: base64{eol}{eol}{}", enc.trim_end()));
    }
    Ok(Some(out.into_bytes()))
}

fn classify(ty: &str, disp: &str, has_name: bool, has_cid: bool, keep_inline: bool) -> bool {
    let att = disp == "attachment";
    if matches!(
        ty,
        "text/calendar" | "message/delivery-status" | "message/disposition-notification" | "message/feedback-report" | "text/rfc822-headers"
    ) && !att
    {
        return false;
    }
    if (ty == "text/plain" || ty == "text/html") && !att {
        return false;
    }
    if ty.starts_with("image/") && !att && (disp == "inline" || has_cid) {
        return !keep_inline;
    }
    if att || has_name || ty == "message/rfc822" {
        return true;
    }
    !ty.starts_with("text/")
}

struct PartSlice<'a> {
    /// The delimiter line including its line ending.
    delimiter: &'a [u8],
    content: &'a [u8],
    /// The line ending that belongs to the next delimiter.
    eol: &'a [u8],
}

struct Split<'a> {
    head: &'a [u8],
    parts: Vec<PartSlice<'a>>,
    /// The closing delimiter and epilogue (empty when the close is missing).
    tail: &'a [u8],
}

fn split_multipart<'a>(body: &'a [u8], boundary: &str) -> Split<'a> {
    let open = format!("--{boundary}");
    let open = open.as_bytes();
    // (line start, line end including eol, is close)
    let mut marks: Vec<(usize, usize, bool)> = Vec::new();
    let mut pos = 0;
    while pos < body.len() {
        let nl = body[pos..].iter().position(|b| *b == b'\n').map_or(body.len(), |i| pos + i);
        let next = (nl + 1).min(body.len());
        let mut line = &body[pos..nl];
        if line.last() == Some(&b'\r') {
            line = &line[..line.len() - 1];
        }
        if line.starts_with(open) {
            let rest = &line[open.len()..];
            let blank = |s: &[u8]| s.iter().all(|b| *b == b' ' || *b == b'\t');
            if blank(rest) {
                marks.push((pos, next, false));
            } else if rest.starts_with(b"--") && blank(&rest[2..]) {
                marks.push((pos, next, true));
                break;
            }
        }
        pos = next;
    }
    let Some(first) = marks.first() else {
        return Split { head: body, parts: Vec::new(), tail: &[] };
    };
    let head = &body[..first.0];
    let mut parts = Vec::new();
    let mut tail: &[u8] = &[];
    for (i, m) in marks.iter().enumerate() {
        if m.2 {
            tail = &body[m.0..];
            break;
        }
        let end = marks.get(i + 1).map_or(body.len(), |n| n.0);
        let mut content_end = end;
        if marks.get(i + 1).is_some() {
            if content_end > m.1 && body[content_end - 1] == b'\n' {
                content_end -= 1;
                if content_end > m.1 && body[content_end - 1] == b'\r' {
                    content_end -= 1;
                }
            }
        }
        parts.push(PartSlice {
            delimiter: &body[m.0..m.1],
            content: &body[m.1..content_end.max(m.1)],
            eol: &body[content_end.max(m.1)..end.max(content_end.max(m.1))],
        });
    }
    Split { head, parts, tail }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(min: u64, keep_inline: bool) -> StripOptions {
        StripOptions { min_bytes: min, keep_inline_images: keep_inline, date: "2026-11-06".into() }
    }

    fn b64(n: usize) -> String {
        let data: Vec<u8> = (0..n).map(|i| (i * 7 % 251) as u8).collect();
        b64_encode(&data, "\r\n")
    }

    fn mixed(att_len: usize) -> Vec<u8> {
        format!(
            "From: a@example.org\r\nTo: b@example.org\r\nSubject: Report\r\nMessage-ID: <m1@example.org>\r\nDate: Mon, 1 Jun 2026 10:00:00 +0000\r\nReferences: <p@example.org>\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"BND\"\r\n\r\nthis is a preamble\r\n--BND\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nSee attached.\r\n--BND\r\nContent-Type: application/pdf; name=\"q3.pdf\"\r\nContent-Disposition: attachment; filename=\"q3.pdf\"\r\nContent-Transfer-Encoding: base64\r\n\r\n{}\r\n--BND--\r\nepilogue\r\n",
            b64(att_len)
        )
        .into_bytes()
    }

    fn stripped(o: StripOutcome) -> (String, Vec<Removed>) {
        match o {
            StripOutcome::Stripped { raw, removed } => (String::from_utf8(raw).unwrap(), removed),
            other => panic!("expected Stripped, got {other:?}"),
        }
    }

    #[test]
    fn mixed_attachment_is_replaced_and_headers_survive() {
        let raw = mixed(50_000);
        let (out, removed) = stripped(strip_attachments(&raw, &opts(1000, true)));
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].name, "q3.pdf");
        assert_eq!(removed[0].path, "2");
        assert!((49_990..=50_000).contains(&removed[0].bytes));
        assert!(out.contains("Message-ID: <m1@example.org>\r\n"));
        assert!(out.contains("References: <p@example.org>\r\n"));
        assert!(out.contains("Date: Mon, 1 Jun 2026 10:00:00 +0000\r\n"));
        assert!(out.contains("X-Mailwoman-Stripped: 2026-11-06; part=2; type=application/pdf;"));
        assert!(out.contains("Attachment removed by retention rule on 2026-11-06: q3.pdf (application/pdf, "));
        assert!(out.contains("this is a preamble\r\n--BND\r\n"));
        assert!(out.contains("See attached.\r\n--BND\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: 7bit"));
        assert!(out.ends_with("--BND--\r\nepilogue\r\n"));
        assert!(!out.contains("application/pdf; name"));
        assert!(out.len() < raw.len() / 10);
        // The result is a valid multipart with two parts and one closing line.
        assert_eq!(out.matches("\r\n--BND\r\n").count(), 2);
        assert_eq!(out.matches("--BND--").count(), 1);
    }

    #[test]
    fn stripping_is_idempotent_and_marker_is_honoured() {
        let raw = mixed(50_000);
        let (out, _) = stripped(strip_attachments(&raw, &opts(1000, true)));
        assert_eq!(strip_attachments(out.as_bytes(), &opts(0, false)), StripOutcome::Skipped(SkipReason::AlreadyStripped));
    }

    #[test]
    fn small_attachment_and_no_attachment_are_noops() {
        assert_eq!(strip_attachments(&mixed(300), &opts(100_000, true)), StripOutcome::Unchanged);
        let plain = b"From: a@example.org\r\nSubject: hi\r\n\r\njust text\r\n";
        assert_eq!(strip_attachments(plain, &opts(0, true)), StripOutcome::Unchanged);
        let html = b"Content-Type: multipart/alternative; boundary=\"A\"\r\n\r\n--A\r\nContent-Type: text/plain\r\n\r\nhi\r\n--A\r\nContent-Type: text/html\r\n\r\n<p>hi</p>\r\n--A--\r\n";
        assert_eq!(strip_attachments(html, &opts(0, true)), StripOutcome::Unchanged);
    }

    fn related() -> Vec<u8> {
        format!(
            "Subject: s\r\nContent-Type: multipart/mixed; boundary=\"OUT\"\r\n\r\n--OUT\r\nContent-Type: multipart/alternative; boundary=\"ALT\"\r\n\r\n--ALT\r\nContent-Type: text/plain\r\n\r\nplain\r\n--ALT\r\nContent-Type: multipart/related; boundary=\"REL\"\r\n\r\n--REL\r\nContent-Type: text/html\r\n\r\n<img src=\"cid:logo\">\r\n--REL\r\nContent-Type: image/png\r\nContent-ID: <logo>\r\nContent-Disposition: inline\r\nContent-Transfer-Encoding: base64\r\n\r\n{}\r\n--REL--\r\n--ALT--\r\n--OUT\r\nContent-Type: image/jpeg; name=\"photo.jpg\"\r\nContent-Disposition: attachment; filename=\"photo.jpg\"\r\nContent-Transfer-Encoding: base64\r\n\r\n{}\r\n--OUT--\r\n",
            b64(9000),
            b64(20000)
        )
        .into_bytes()
    }

    #[test]
    fn inline_images_are_kept_when_configured() {
        let (out, removed) = stripped(strip_attachments(&related(), &opts(100, true)));
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].path, "2");
        assert!(out.contains("Content-ID: <logo>"));
        assert!(out.contains("--REL--\r\n--ALT--\r\n--OUT\r\n"));
        assert!(!out.contains("photo.jpg\"\r\nContent-Disposition"));
    }

    #[test]
    fn inline_images_go_when_not_kept() {
        let (out, removed) = stripped(strip_attachments(&related(), &opts(100, false)));
        assert_eq!(removed.iter().map(|r| r.path.as_str()).collect::<Vec<_>>(), ["1.2.2", "2"]);
        assert!(!out.contains("Content-ID: <logo>"));
        assert!(out.contains("<img src=\"cid:logo\">"));
        assert_eq!(out.matches("X-Mailwoman-Stripped:").count(), 2);
    }

    #[test]
    fn nested_message_rfc822_is_one_attachment() {
        let inner = format!(
            "Subject: inner\r\nContent-Type: multipart/mixed; boundary=\"IN\"\r\n\r\n--IN\r\nContent-Type: text/plain\r\n\r\nx\r\n--IN\r\nContent-Type: application/zip\r\nContent-Disposition: attachment; filename=a.zip\r\n\r\n{}\r\n--IN--\r\n",
            "Z".repeat(5000)
        );
        let raw = format!(
            "Subject: outer\r\nContent-Type: multipart/mixed; boundary=\"OUT\"\r\n\r\n--OUT\r\nContent-Type: text/plain\r\n\r\nfwd\r\n--OUT\r\nContent-Type: message/rfc822\r\nContent-Disposition: attachment; filename=\"fwd.eml\"\r\n\r\n{inner}\r\n--OUT--\r\n"
        );
        let (out, removed) = stripped(strip_attachments(raw.as_bytes(), &opts(1000, true)));
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].content_type, "message/rfc822");
        assert!(!out.contains("a.zip"));
        // The inner boundary is gone entirely.
        assert!(!out.contains("--IN"));
    }

    #[test]
    fn signed_encrypted_and_pgp_messages_are_skipped_untouched() {
        let smime = "Content-Type: multipart/signed; protocol=\"application/pkcs7-signature\"; boundary=\"S\"\r\n\r\n--S\r\nContent-Type: multipart/mixed; boundary=\"M\"\r\n\r\n--M\r\nContent-Type: application/pdf\r\nContent-Disposition: attachment; filename=a.pdf\r\n\r\nAAAA\r\n--M--\r\n--S\r\nContent-Type: application/pkcs7-signature\r\n\r\nSIG\r\n--S--\r\n";
        assert_eq!(strip_attachments(smime.as_bytes(), &opts(0, true)), StripOutcome::Skipped(SkipReason::Signed));
        let enveloped = "Content-Type: application/pkcs7-mime; smime-type=enveloped-data\r\n\r\nxxxx\r\n";
        assert_eq!(strip_attachments(enveloped.as_bytes(), &opts(0, true)), StripOutcome::Skipped(SkipReason::Encrypted));
        let pgpmime = "Content-Type: multipart/encrypted; protocol=\"application/pgp-encrypted\"; boundary=\"P\"\r\n\r\n--P\r\nContent-Type: application/pgp-encrypted\r\n\r\nVersion: 1\r\n--P\r\nContent-Type: application/octet-stream\r\n\r\nxx\r\n--P--\r\n";
        assert_eq!(strip_attachments(pgpmime.as_bytes(), &opts(0, true)), StripOutcome::Skipped(SkipReason::Encrypted));
        let inline_pgp = "Content-Type: multipart/mixed; boundary=\"B\"\r\n\r\n--B\r\nContent-Type: text/plain\r\n\r\n-----BEGIN PGP MESSAGE-----\r\nabc\r\n-----END PGP MESSAGE-----\r\n--B\r\nContent-Type: application/pdf\r\nContent-Disposition: attachment; filename=a.pdf\r\n\r\nAAAA\r\n--B--\r\n";
        assert_eq!(strip_attachments(inline_pgp.as_bytes(), &opts(0, true)), StripOutcome::Skipped(SkipReason::Encrypted));
    }

    #[test]
    fn quoted_printable_lf_only_and_encoded_names() {
        let raw = "Content-Type: multipart/mixed; boundary=B\n\n--B\nContent-Type: text/plain\n\nbody\n--B\nContent-Type: application/octet-stream; name=\"=?utf-8?B?csOpc3Vtw6kucGRm?=\"\nContent-Disposition: attachment; filename*=utf-8''r%C3%A9sum%C3%A9.pdf\nContent-Transfer-Encoding: quoted-printable\n\n=41=42=43 long long long long long long long long\n--B--\n";
        let (out, removed) = stripped(strip_attachments(raw.as_bytes(), &opts(10, true)));
        assert_eq!(removed[0].name, "résumé.pdf");
        assert!(!out.contains('\r'));
        // Non-ASCII placeholder text travels as base64.
        assert!(out.contains("Content-Transfer-Encoding: base64"));
        assert!(out.contains("X-Mailwoman-Stripped: 2026-11-06; part=2; type=application/octet-stream; size=") && out.contains("name=r%C3%A9sum%C3%A9.pdf"));
    }

    #[test]
    fn huge_parts_are_handled_and_oversize_messages_refused() {
        let raw = mixed(6_000_000);
        let (out, removed) = stripped(strip_attachments(&raw, &opts(1_000_000, true)));
        assert!(removed[0].bytes > 5_900_000);
        assert!(out.len() < 2000);
        let big = vec![b'a'; MAX_RAW + 1];
        assert_eq!(strip_attachments(&big, &opts(0, true)), StripOutcome::Skipped(SkipReason::TooLarge));
    }

    #[test]
    fn missing_boundary_is_unparsable_and_calendar_parts_stay() {
        let raw = "Content-Type: multipart/mixed\r\n\r\nx\r\n";
        assert_eq!(strip_attachments(raw.as_bytes(), &opts(0, true)), StripOutcome::Skipped(SkipReason::Unparsable));
        let cal = "Content-Type: multipart/mixed; boundary=\"B\"\r\n\r\n--B\r\nContent-Type: text/plain\r\n\r\nhi\r\n--B\r\nContent-Type: text/calendar; method=REQUEST\r\n\r\nBEGIN:VCALENDAR\r\n--B--\r\n";
        assert_eq!(strip_attachments(cal.as_bytes(), &opts(0, true)), StripOutcome::Unchanged);
    }
}
