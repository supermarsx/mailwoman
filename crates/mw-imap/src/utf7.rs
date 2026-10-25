//! Mailbox names on the wire: modified UTF-7 (RFC 3501 §5.1.3) and the
//! UTF8=ACCEPT form (RFC 6855 / RFC 9051 §6.3.1).
//!
//! A server that has not had `ENABLE UTF8=ACCEPT` (or IMAP4rev2, where UTF-8
//! names are the default) expects and sends non-ASCII mailbox names in
//! modified UTF-7: printable ASCII stands for itself, `&` is written `&-`, and
//! any other run of characters is UTF-16BE in a base64 variant (`,` instead of
//! `/`, no padding) between `&` and `-`. "Entwürfe" is `Entw&APw-rfe`.

use base64::Engine;
use base64::alphabet::Alphabet;
use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};

/// The modified base64 alphabet: standard, with `,` in place of `/`.
const MUTF7_ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+,";

fn engine() -> GeneralPurpose {
    // The alphabet literal is valid by construction.
    let alphabet = Alphabet::new(MUTF7_ALPHABET).unwrap_or(base64::alphabet::STANDARD);
    GeneralPurpose::new(
        &alphabet,
        GeneralPurposeConfig::new()
            .with_encode_padding(false)
            .with_decode_padding_mode(DecodePaddingMode::RequireNone)
            .with_decode_allow_trailing_bits(false),
    )
}

/// Why a modified UTF-7 name could not be decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Utf7Error {
    /// A byte outside printable ASCII appeared unencoded.
    RawByte(u8),
    /// A `&` shift was not closed by `-`.
    Unterminated,
    /// The base64 run is malformed (bad character, padding, stray bits).
    BadBase64,
    /// The decoded UTF-16 is invalid (odd length, unpaired surrogate).
    BadUtf16,
    /// The run encodes only printable ASCII, which must be sent directly, or
    /// two encoded runs follow each other with nothing between them.
    NotCanonical,
}

impl std::fmt::Display for Utf7Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Utf7Error::RawByte(b) => write!(f, "byte 0x{b:02x} must be encoded"),
            Utf7Error::Unterminated => f.write_str("encoded run is not closed by '-'"),
            Utf7Error::BadBase64 => f.write_str("encoded run is not valid modified base64"),
            Utf7Error::BadUtf16 => f.write_str("encoded run is not valid UTF-16"),
            Utf7Error::NotCanonical => f.write_str("encoded run is not in canonical form"),
        }
    }
}

impl std::error::Error for Utf7Error {}

fn is_direct(c: char) -> bool {
    (' '..='~').contains(&c)
}

/// Encode a mailbox name as modified UTF-7.
pub fn encode(name: &str) -> String {
    let engine = engine();
    let mut out = String::with_capacity(name.len() + 8);
    let mut run: Vec<u16> = Vec::new();
    let flush = |run: &mut Vec<u16>, out: &mut String| {
        if run.is_empty() {
            return;
        }
        let bytes: Vec<u8> = run.iter().flat_map(|u| u.to_be_bytes()).collect();
        out.push('&');
        out.push_str(&engine.encode(bytes));
        out.push('-');
        run.clear();
    };
    for c in name.chars() {
        if is_direct(c) {
            flush(&mut run, &mut out);
            if c == '&' {
                out.push_str("&-");
            } else {
                out.push(c);
            }
        } else {
            let mut units = [0u16; 2];
            run.extend_from_slice(c.encode_utf16(&mut units));
        }
    }
    flush(&mut run, &mut out);
    out
}

/// Decode a modified UTF-7 mailbox name, strictly.
pub fn decode(wire: &str) -> Result<String, Utf7Error> {
    let engine = engine();
    let bytes = wire.as_bytes();
    let mut out = String::with_capacity(wire.len());
    let mut i = 0;
    // Position just after the previous encoded run, to catch `&AGE-&AGI-`.
    let mut prev_run_end = None;
    while i < bytes.len() {
        let b = bytes[i];
        if !(0x20..=0x7e).contains(&b) {
            return Err(Utf7Error::RawByte(b));
        }
        if b != b'&' {
            out.push(b as char);
            i += 1;
            continue;
        }
        let start = i + 1;
        let end = bytes[start..]
            .iter()
            .position(|&c| c == b'-')
            .map(|p| start + p)
            .ok_or(Utf7Error::Unterminated)?;
        if end == start {
            out.push('&');
            i = end + 1;
            continue;
        }
        if prev_run_end == Some(i) {
            return Err(Utf7Error::NotCanonical);
        }
        let raw = engine
            .decode(&bytes[start..end])
            .map_err(|_| Utf7Error::BadBase64)?;
        if raw.len() % 2 != 0 {
            return Err(Utf7Error::BadUtf16);
        }
        let units = raw
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&p| u16::from_be_bytes(p));
        for ch in char::decode_utf16(units) {
            let ch = ch.map_err(|_| Utf7Error::BadUtf16)?;
            if is_direct(ch) {
                return Err(Utf7Error::NotCanonical);
            }
            out.push(ch);
        }
        i = end + 1;
        prev_run_end = Some(i);
    }
    Ok(out)
}

/// Decode a name the server sent, keeping the bytes as they are when they are
/// not valid modified UTF-7 (some servers send raw UTF-8 or broken names; the
/// name must still round-trip to the server unchanged). With `utf8` (UTF8=ACCEPT
/// or IMAP4rev2 enabled) names are already UTF-8 and are not decoded.
pub fn decode_lossless(wire: &str, utf8: bool) -> String {
    if utf8 {
        return wire.to_string();
    }
    decode(wire).unwrap_or_else(|_| wire.to_string())
}

/// Why a mailbox name cannot be sent as a quoted string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnquotableName;

impl std::fmt::Display for UnquotableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("mailbox name contains CR, LF or NUL")
    }
}

impl std::error::Error for UnquotableName {}

/// The mailbox name as a command argument: a quoted string in UTF-8 when
/// `utf8` is enabled, else in modified UTF-7.
pub fn mailbox_arg(name: &str, utf8: bool) -> Result<String, UnquotableName> {
    if name.contains(['\r', '\n', '\0']) {
        return Err(UnquotableName);
    }
    let wire = if utf8 { name.to_string() } else { encode(name) };
    let mut out = String::with_capacity(wire.len() + 2);
    out.push('"');
    for ch in wire.chars() {
        if ch == '"' || ch == '\\' {
            out.push('\\');
        }
        out.push(ch);
    }
    out.push('"');
    Ok(out)
}

/// Fuzz entry point: decoding arbitrary input must not panic, and every
/// successful decode must re-encode to the same wire form (the codec is
/// canonical both ways).
pub fn fuzz(data: &[u8]) {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    if let Ok(name) = decode(text) {
        assert_eq!(encode(&name), text, "decode/encode is not canonical");
    }
    assert_eq!(decode(&encode(text)).as_deref(), Ok(text));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc_3501_examples() {
        // RFC 3501 §5.1.3: "~peter/mail/台北/日本語".
        assert_eq!(
            encode("~peter/mail/\u{53f0}\u{5317}/\u{65e5}\u{672c}\u{8a9e}"),
            "~peter/mail/&U,BTFw-/&ZeVnLIqe-"
        );
        assert_eq!(
            decode("~peter/mail/&U,BTFw-/&ZeVnLIqe-").unwrap(),
            "~peter/mail/\u{53f0}\u{5317}/\u{65e5}\u{672c}\u{8a9e}"
        );
    }

    #[test]
    fn common_folder_names() {
        for (name, wire) in [
            ("Entwürfe", "Entw&APw-rfe"),
            ("Gelöschte Elemente", "Gel&APY-schte Elemente"),
            ("Корзина", "&BBoEPgRABDcEOAQ9BDA-"),
            ("送信済み", "&kAFP4W4IMH8-"),
            ("A & B", "A &- B"),
            ("Plain", "Plain"),
            ("", ""),
        ] {
            assert_eq!(encode(name), wire, "encode {name:?}");
            assert_eq!(decode(wire).unwrap(), name, "decode {wire:?}");
        }
    }

    #[test]
    fn astral_plane_uses_surrogate_pairs() {
        let name = "Mail \u{1f4e7}";
        let wire = encode(name);
        assert_eq!(wire, "Mail &2D3c5w-");
        assert_eq!(decode(&wire).unwrap(), name);
    }

    #[test]
    fn strict_decode_rejects_malformed_runs() {
        assert_eq!(decode("Entw&APw"), Err(Utf7Error::Unterminated));
        assert_eq!(decode("a&A/w-"), Err(Utf7Error::BadBase64));
        assert_eq!(decode("a&APw=-"), Err(Utf7Error::BadBase64));
        assert_eq!(decode("a&AP-"), Err(Utf7Error::BadBase64));
        assert_eq!(decode("a&2D0-"), Err(Utf7Error::BadUtf16));
        // "a" (U+0061) must be sent directly, not encoded.
        assert_eq!(decode("&AGE-"), Err(Utf7Error::NotCanonical));
        assert_eq!(decode("&APw-&APw-"), Err(Utf7Error::NotCanonical));
        assert_eq!(decode("t\u{e9}st"), Err(Utf7Error::RawByte(0xc3)));
        assert_eq!(decode("tab\there"), Err(Utf7Error::RawByte(0x09)));
    }

    #[test]
    fn lossless_decode_keeps_what_it_cannot_read() {
        assert_eq!(decode_lossless("Entw&APw-rfe", false), "Entwürfe");
        assert_eq!(decode_lossless("Entwürfe", false), "Entwürfe");
        assert_eq!(decode_lossless("Entw&APw-rfe", true), "Entw&APw-rfe");
    }

    #[test]
    fn mailbox_args() {
        assert_eq!(mailbox_arg("Entwürfe", false).unwrap(), "\"Entw&APw-rfe\"");
        assert_eq!(mailbox_arg("Entwürfe", true).unwrap(), "\"Entwürfe\"");
        assert_eq!(mailbox_arg("a\"b\\c", false).unwrap(), "\"a\\\"b\\\\c\"");
        assert_eq!(mailbox_arg("bad\r\nname", false), Err(UnquotableName));
    }

    #[test]
    fn fuzz_corpus_never_panics() {
        for buf in [
            &b""[..],
            b"&",
            b"&-",
            b"&&-",
            b"&AAA-",
            b"&2D3c5w-",
            b"&3D0-",
            b"\xff",
            b"a&U,BTFw-b",
            "Ü&".as_bytes(),
        ] {
            fuzz(buf);
        }
    }
}
