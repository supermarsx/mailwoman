//! Bounded, versioned projection of the headers exposed by `Email/get`.
//!
//! An absent or invalid projection means callers must use the original raw
//! header path. Oversized headers are never silently truncated or marked valid.
use serde_json::{Value, json};
use std::io::{self, Write};

pub(crate) const KEY: &str = "_mailwomanHeaderProjection";
const VERSION: u64 = 1;
const MAX_ENCODED_BYTES: usize = 64 * 1024;
const READABLE: [&str; 11] = [
    "delivered-to",
    "x-original-to",
    "envelope-to",
    "x-envelope-to",
    "x-forwarded-to",
    "resent-to",
    "expires",
    "expiry-date",
    "reply-by",
    "sensitivity",
    "supersedes",
];

/// The complete header block, excluding the first empty line and message body.
fn header_bytes(raw: &[u8]) -> &[u8] {
    let mut offset = 0;
    for line in raw.split(|byte| *byte == b'\n') {
        if line.is_empty() || line == b"\r" {
            return &raw[..offset];
        }
        offset += line.len();
        if offset < raw.len() {
            offset += 1;
        }
    }
    raw
}

/// Raw header pairs in message order, with the legacy unfolding/trim behavior.
/// Decoding only the header block avoids validating or copying binary bodies.
pub(crate) fn pairs(raw: &[u8]) -> Vec<(String, String)> {
    let text = String::from_utf8_lossy(header_bytes(raw));
    let mut pairs: Vec<(String, String)> = Vec::new();
    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.is_empty() {
            break;
        }
        if line.starts_with([' ', '\t']) {
            if let Some((_, value)) = pairs.last_mut() {
                value.push(' ');
                value.push_str(line.trim());
            }
        } else if let Some((name, value)) = line.split_once(':') {
            pairs.push((name.trim().to_ascii_lowercase(), value.trim().to_string()));
        }
    }
    pairs
}

/// Count encoded bytes without cloning or serializing an untrusted object into
/// an arbitrarily large buffer. The serializer stops as soon as the cap is hit.
struct EncodedBudget(usize);
impl Write for EncodedBudget {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let Some(remaining) = self.0.checked_sub(bytes.len()) else {
            return Err(io::Error::other("header projection exceeds byte budget"));
        };
        self.0 = remaining;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn fits(value: &Value) -> bool {
    encoded_size(value).is_some()
}
fn encoded_size<T: serde::Serialize + ?Sized>(value: &T) -> Option<usize> {
    let mut budget = EncodedBudget(MAX_ENCODED_BYTES);
    serde_json::to_writer(&mut budget, value).ok()?;
    Some(MAX_ENCODED_BYTES - budget.0)
}

/// Keep only readable fields while parsing, rather than allocating unknown
/// values/folds and discarding them later. Malformed non-header lines leave the
/// preceding field active, matching `pairs`; valid unknown headers replace it.
fn bounded_pairs(raw: &[u8]) -> Option<Vec<(String, String)>> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut current: Option<usize> = None;
    let mut size = encoded_size(&json!({"version": VERSION, "pairs": []}))?;
    for line in header_bytes(raw).split(|byte| *byte == b'\n') {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if line.is_empty() {
            break;
        }
        if line.starts_with(b" ") || line.starts_with(b"\t") {
            let Some(index) = current else {
                continue;
            };
            if line.len() > MAX_ENCODED_BYTES {
                return None;
            }
            let decoded = String::from_utf8_lossy(line);
            let continuation = decoded.trim();
            // Encoded strings include two quotes. Appending an unfolded line
            // contributes its escaped content and one separating ASCII space.
            let added = encoded_size(continuation)?.checked_sub(2)?.checked_add(1)?;
            size = size.checked_add(added)?;
            if size > MAX_ENCODED_BYTES {
                return None;
            }
            let value: &mut String = &mut pairs[index].1;
            value.push(' ');
            value.push_str(continuation);
        } else if let Some(colon) = line.iter().position(|byte| *byte == b':') {
            if colon > MAX_ENCODED_BYTES {
                return None;
            }
            let name = String::from_utf8_lossy(&line[..colon])
                .trim()
                .to_ascii_lowercase();
            current = None;
            if !READABLE.contains(&name.as_str()) {
                continue;
            }
            let raw_value = &line[colon + 1..];
            if raw_value.len() > MAX_ENCODED_BYTES {
                return None;
            }
            let value = String::from_utf8_lossy(raw_value).trim().to_owned();
            let added =
                encoded_size(&(&name, &value))?.checked_add(usize::from(!pairs.is_empty()))?;
            size = size.checked_add(added)?;
            if size > MAX_ENCODED_BYTES {
                return None;
            }
            current = Some(pairs.len());
            pairs.push((name, value));
        }
    }
    Some(pairs)
}

/// Replace any previous projection with a complete snapshot, if bounded.
/// Other envelope fields remain unchanged even when projection is impossible.
pub(crate) fn stamp(email: &mut Value, raw: &[u8]) {
    let Some(object) = email.as_object_mut() else {
        return;
    };
    object.remove(KEY);
    let Some(pairs) = bounded_pairs(raw) else {
        return;
    };
    let projection = json!({"version": VERSION, "pairs": pairs});
    if fits(&projection) {
        object.insert(KEY.to_owned(), projection);
    }
}

/// Read only a fully validated current projection. Validate all borrowed data
/// and the encoded size before allocating the returned strings.
pub(crate) fn read(email: &Value) -> Option<Vec<(String, String)>> {
    let projection = email.get(KEY)?;
    let object = projection.as_object()?;
    if object.len() != 2 || object.get("version")?.as_u64()? != VERSION {
        return None;
    }
    let entries = object.get("pairs")?.as_array()?;
    if entries.len() > MAX_ENCODED_BYTES / 7 {
        return None;
    }
    let mut bytes = 0usize;
    for entry in entries {
        let pair = entry.as_array()?;
        if pair.len() != 2 {
            return None;
        }
        let name = pair[0].as_str()?;
        let value = pair[1].as_str()?;
        if !READABLE.contains(&name) {
            return None;
        }
        bytes = bytes.checked_add(name.len())?.checked_add(value.len())?;
        if bytes > MAX_ENCODED_BYTES {
            return None;
        }
    }
    if !fits(projection) {
        return None;
    }
    Some(
        entries
            .iter()
            .map(|entry| {
                (
                    entry[0].as_str().unwrap().to_owned(),
                    entry[1].as_str().unwrap().to_owned(),
                )
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_body_is_ignored_and_only_readable_headers_are_snapshotted() {
        let mut raw =
            b"From: ignored@example.org\r\nDelivered-To: owner@example.org\r\n\r\n".to_vec();
        raw.extend(std::iter::repeat_n(0xff, 2 * 1024 * 1024));
        let mut email = json!({"unknown": {"keep": true}, "bodyValues":{"1":{"value":"preserve"}}});
        stamp(&mut email, &raw);
        assert_eq!(
            read(&email),
            Some(vec![("delivered-to".into(), "owner@example.org".into())])
        );
        assert_eq!(pairs(&raw).len(), 2);
        assert_eq!(email["unknown"]["keep"], true);
        assert_eq!(email["bodyValues"]["1"]["value"], "preserve");
    }

    #[test]
    fn empty_or_missing_readable_headers_are_valid_complete_projections() {
        for raw in [b"".as_slice(), b"\r\nbody", b"From: x@example.org\n\nbody"] {
            let mut email = json!({});
            stamp(&mut email, raw);
            assert_eq!(read(&email), Some(vec![]));
        }
        assert_eq!(read(&json!({})), None);
    }

    #[test]
    fn huge_unknown_values_and_folds_are_skipped_without_polluting_allowed_fields() {
        let mut raw = b"Expires: soon\nX-Unknown: ".to_vec();
        raw.extend(std::iter::repeat_n(0xff, 2 * 1024 * 1024));
        raw.extend_from_slice(b"\n\t");
        raw.extend(std::iter::repeat_n(b'x', 2 * 1024 * 1024));
        raw.extend_from_slice(b"\nDelivered-To: owner@example.org\n\nbody");
        let mut email = json!({});
        stamp(&mut email, &raw);
        assert_eq!(
            read(&email),
            Some(vec![
                ("expires".into(), "soon".into()),
                ("delivered-to".into(), "owner@example.org".into())
            ])
        );
    }

    #[test]
    fn duplicate_folding_mixed_newlines_and_malformed_lines_match_legacy() {
        let raw = b" orphan\r\nDelivered-To: first@example.org\r\n\tcontinued\nmalformed\n second\r\nDELIVERED-TO: second@example.org\nExpires: soon\r\n\r\nbody";
        let expected = vec![
            (
                "delivered-to".into(),
                "first@example.org continued second".into(),
            ),
            ("delivered-to".into(), "second@example.org".into()),
            ("expires".into(), "soon".into()),
        ];
        assert_eq!(pairs(raw), expected);
        let mut email = json!({});
        stamp(&mut email, raw);
        assert_eq!(read(&email), Some(expected));
        assert_eq!(
            pairs(b"Expires: bad\xffvalue\r\n\r\n"),
            vec![("expires".into(), "bad\u{fffd}value".into())]
        );
    }

    #[test]
    fn invalid_shapes_versions_names_and_large_payloads_are_rejected() {
        for invalid in [
            json!(null),
            json!([]),
            json!({"version":0,"pairs":[]}),
            json!({"version":2,"pairs":[]}),
            json!({"version":1,"pairs":null}),
            json!({"version":1,"pairs":[["expires",1]]}),
            json!({"version":1,"pairs":[["expires","a","b"]]}),
            json!({"version":1,"pairs":[["Expires","soon"]]}),
            json!({"version":1,"pairs":[["subject","secret"]]}),
            json!({"version":1,"pairs":[],"extra":true}),
            json!({"version":1,"pairs":[["expires","x".repeat(MAX_ENCODED_BYTES)]]}),
            json!({"version":1,"pairs":[["expires","\u{0}".repeat(MAX_ENCODED_BYTES/2)]]}),
        ] {
            let email = json!({KEY: invalid, "unknown": "keep"});
            assert_eq!(read(&email), None);
            assert_eq!(email["unknown"], "keep");
        }
    }

    #[test]
    fn overflow_removes_old_stamp_without_truncating_or_losing_envelope() {
        let mut email = json!({KEY: {"version":1,"pairs":[["expires","old"]]}, "unknown": "keep"});
        let raw = format!("Expires: {}\r\n\r\nbody", "x".repeat(MAX_ENCODED_BYTES));
        stamp(&mut email, raw.as_bytes());
        assert!(email.get(KEY).is_none());
        assert_eq!(email["unknown"], "keep");
        assert_eq!(pairs(raw.as_bytes())[0].1.len(), MAX_ENCODED_BYTES);
        let raw = format!("Expires: {}\n\nbody", "\u{0}".repeat(MAX_ENCODED_BYTES / 2));
        stamp(&mut email, raw.as_bytes());
        assert!(email.get(KEY).is_none());
        stamp(&mut email, b"Expires: fresh\n\n");
        assert_eq!(read(&email), Some(vec![("expires".into(), "fresh".into())]));
    }
}
