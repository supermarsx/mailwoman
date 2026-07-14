//! `spam-spamassassin` — the SpamAssassin spam-classification bridge plugin
//! (t10 §3 e6, SPEC §10.8).
//!
//! A thin `spam-action` component: `classify(raw)` frames the message in the
//! **SpamAssassin `spamd` SPAMC/1.5** protocol and exchanges it with a `spamd` endpoint
//! via the host `http-fetch` byte transport — **host-mediated, under a net allowlist** —
//! then maps the `Spam: True/False ; score / threshold` reply onto the spam WIT
//! `classify -> result<string>` contract. The SPAMC/SPAMD codec is hand-rolled pure Rust
//! (NO C `spamc` client, no `-sys`), keeping the permissive license floor intact.
//!
//! ## Transport note (for e13/e14)
//! `spamd` speaks a line-based protocol over a raw TCP port (default 783), NOT HTTP; the
//! jail's only egress is the host `http-fetch`. This guest therefore emits the exact
//! SPAMC request frame as the fetch body and parses the SPAMD frame from the response
//! body — the host fetcher (or a thin TCP relay in e14's docker) carries the bytes to
//! `spamd:783`. The pure codec below is transport-agnostic and fully host-unit-tested.
//!
//! ## Verdict contract + fail-soft posture
//! Identical envelope to `spam-rspamd`: `classify` returns `Ok(<json>)` with
//! `{"verdict","action","score","threshold","symbols","source":"spamassassin"}`,
//! `verdict` ∈ [`VERDICT_HAM`]/[`VERDICT_SPAM`]/[`VERDICT_UNKNOWN`]. An unreachable
//! daemon, a non-`EX_OK` code, an over-sized body, or a garbled frame all resolve to
//! [`VERDICT_UNKNOWN`] — never a panic, never a hard block. All parsing is bounded
//! ([`MAX_RESPONSE_BYTES`]) and hostile-input safe.

// Forbid unsafe in our own (host + pure) code; the `wasm32` guest pulls in wit-bindgen's
// generated ABI glue, which necessarily uses `unsafe`, so the lint is scoped to the
// non-wasm build (where all first-party logic is exercised + tested).
#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

/// The manifest plugin id.
pub const PLUGIN_ID: &str = "spam-spamassassin";

/// Default `spamd` endpoint (`host:port`). Overridable per deployment via the `endpoint`
/// KV config key (see [`component`]); the host must be in the plugin `net_allowlist`.
pub const DEFAULT_ENDPOINT: &str = "spamassassin:783";

/// The SPAMC protocol version this guest speaks.
pub const SPAMC_VERSION: &str = "1.5";

/// Verdict: not spam (`Spam: False`).
pub const VERDICT_HAM: &str = "ham";
/// Verdict: spam (`Spam: True`).
pub const VERDICT_SPAM: &str = "spam";
/// Verdict: undetermined (fail-soft). NEVER a hard block.
pub const VERDICT_UNKNOWN: &str = "unknown";

/// Upper bound on a daemon response we will parse (hostile-input guard).
pub const MAX_RESPONSE_BYTES: usize = 1 << 20; // 1 MiB

/// Cap on the number of symbols carried in a verdict (bounded output).
pub const MAX_SYMBOLS: usize = 64;

#[must_use]
pub fn plugin_id() -> &'static str {
    PLUGIN_ID
}

/// Build a SPAMC/1.5 request frame for `command` (`CHECK` | `SYMBOLS` | `REPORT`) over
/// `message`. The `SYMBOLS` command returns the verdict header plus a symbol list body.
#[must_use]
pub fn build_spamc_request(command: &str, message: &[u8]) -> Vec<u8> {
    let head = format!(
        "{command} SPAMC/{SPAMC_VERSION}\r\nContent-length: {}\r\nUser: mailwoman\r\n\r\n",
        message.len()
    );
    let mut out = head.into_bytes();
    out.extend_from_slice(message);
    out
}

/// Map a raw SPAMD response frame to a verdict string.
///
/// Fail-soft: an empty/over-sized/garbled frame, a non-`SPAMD/` status line, a non-zero
/// response code, or a missing `Spam:` header all resolve to [`VERDICT_UNKNOWN`] — never
/// a panic, never a hard block.
#[must_use]
pub fn classify_spamc(response: &[u8]) -> String {
    if response.is_empty() {
        return unknown_verdict("empty spamd response");
    }
    if response.len() > MAX_RESPONSE_BYTES {
        return unknown_verdict("spamd response exceeds parse bound");
    }

    let (head, body) = split_headers(response);
    let head_str = String::from_utf8_lossy(head);
    let mut lines = head_str.split('\n').map(|l| l.trim_end_matches('\r'));

    // Status line: `SPAMD/1.1 0 EX_OK` — the code is the 2nd whitespace token; 0 = ok.
    match lines.next() {
        Some(s) if s.starts_with("SPAMD/") => {
            let code = s.split_whitespace().nth(1).unwrap_or("");
            if code != "0" {
                return unknown_verdict(&format!("spamd response code '{code}'"));
            }
        }
        _ => return unknown_verdict("not a SPAMD response"),
    }

    // `Spam: True ; 15.0 / 5.0`
    let mut is_spam: Option<bool> = None;
    let mut score = 0.0;
    let mut threshold = 0.0;
    for line in lines {
        if let Some(rest) = strip_prefix_ci(line, "spam:") {
            let (verdict_part, nums) = match rest.split_once(';') {
                Some((v, n)) => (v.trim(), n.trim()),
                None => (rest.trim(), ""),
            };
            is_spam = Some(
                verdict_part.eq_ignore_ascii_case("true")
                    || verdict_part.eq_ignore_ascii_case("yes"),
            );
            if let Some((s, t)) = nums.split_once('/') {
                score = parse_f64(s);
                threshold = parse_f64(t);
            }
        }
    }

    let Some(is_spam) = is_spam else {
        return unknown_verdict("no Spam header in spamd response");
    };
    let verdict = if is_spam { VERDICT_SPAM } else { VERDICT_HAM };
    let symbols = parse_symbols(body);
    verdict_json(verdict, score, threshold, &symbols, "")
}

/// The explicit fail-soft verdict (unreachable daemon, denied net, transport error).
#[must_use]
pub fn unknown_verdict(note: &str) -> String {
    verdict_json(VERDICT_UNKNOWN, 0.0, 0.0, &[], note)
}

/// Split a response into its header block and body at the first blank line (`\r\n\r\n`
/// or `\n\n`). No delimiter ⇒ everything is header, body empty.
fn split_headers(resp: &[u8]) -> (&[u8], &[u8]) {
    if let Some(i) = find(resp, b"\r\n\r\n") {
        (&resp[..i], &resp[i + 4..])
    } else if let Some(i) = find(resp, b"\n\n") {
        (&resp[..i], &resp[i + 2..])
    } else {
        (resp, &[])
    }
}

/// First index of `needle` in `hay` (bounded, no allocation).
fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

/// Case-insensitive prefix strip, returning the remainder after the prefix.
fn strip_prefix_ci<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let trimmed = line.trim_start();
    if trimmed.len() >= prefix.len() && trimmed[..prefix.len()].eq_ignore_ascii_case(prefix) {
        Some(&trimmed[prefix.len()..])
    } else {
        None
    }
}

fn parse_f64(s: &str) -> f64 {
    match s.trim().parse::<f64>() {
        Ok(f) if f.is_finite() => f,
        _ => 0.0,
    }
}

/// Parse the `SYMBOLS` body (symbol names separated by commas and/or newlines) into a
/// sorted, deduplicated, bounded list.
fn parse_symbols(body: &[u8]) -> Vec<String> {
    if body.is_empty() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(body);
    let mut out: Vec<String> = text
        .split([',', '\n', '\r'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    out.sort();
    out.dedup();
    out.truncate(MAX_SYMBOLS);
    out
}

/// Hand-build the deterministic verdict JSON (no serialize dep; daemon-supplied strings
/// are escaped so a hostile symbol name cannot break the envelope). The envelope shape
/// matches `spam-rspamd` (`action` is absent for SpamAssassin ⇒ empty string).
fn verdict_json(
    verdict: &str,
    score: f64,
    threshold: f64,
    symbols: &[String],
    note: &str,
) -> String {
    let mut syms = String::from("[");
    for (i, s) in symbols.iter().enumerate() {
        if i > 0 {
            syms.push(',');
        }
        json_str(&mut syms, s);
    }
    syms.push(']');

    let mut out = String::with_capacity(96 + syms.len());
    out.push_str("{\"verdict\":");
    json_str(&mut out, verdict);
    out.push_str(",\"action\":\"\"");
    out.push_str(&format!(",\"score\":{:.2}", num(score)));
    out.push_str(&format!(",\"threshold\":{:.2}", num(threshold)));
    out.push_str(",\"symbols\":");
    out.push_str(&syms);
    out.push_str(",\"source\":\"spamassassin\"");
    if !note.is_empty() {
        out.push_str(",\"note\":");
        json_str(&mut out, note);
    }
    out.push('}');
    out
}

fn num(f: f64) -> f64 {
    if f.is_finite() { f } else { 0.0 }
}

/// Append `s` as a JSON string literal (minimal RFC 8259 escaping).
fn json_str(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(target_arch = "wasm32")]
mod component;

#[cfg(test)]
mod tests {
    use super::*;

    fn field<'a>(json: &'a str, key: &str) -> &'a str {
        let needle = format!("\"{key}\":");
        let start = json.find(&needle).expect("key present") + needle.len();
        let rest = &json[start..];
        if let Some(stripped) = rest.strip_prefix('"') {
            let end = stripped.find('"').unwrap();
            &stripped[..end]
        } else {
            let end = rest.find([',', '}']).unwrap_or(rest.len());
            &rest[..end]
        }
    }

    #[test]
    fn request_frame_is_well_formed() {
        let frame = build_spamc_request("SYMBOLS", b"Subject: hi\r\n\r\nbody");
        let text = String::from_utf8(frame).unwrap();
        assert!(text.starts_with("SYMBOLS SPAMC/1.5\r\n"), "{text}");
        assert!(text.contains("Content-length: 19\r\n"), "{text}");
        assert!(text.contains("User: mailwoman\r\n"));
        assert!(text.ends_with("\r\n\r\nSubject: hi\r\n\r\nbody"));
    }

    #[test]
    fn spam_true_response_is_spam_with_score_and_symbols() {
        let resp = b"SPAMD/1.1 0 EX_OK\r\n\
Content-length: 24\r\n\
Spam: True ; 15.2 / 5.0\r\n\
\r\n\
BAYES_99,FORGED_MUA,HTML";
        let out = classify_spamc(resp);
        assert_eq!(field(&out, "verdict"), VERDICT_SPAM);
        assert_eq!(field(&out, "score"), "15.20");
        assert_eq!(field(&out, "threshold"), "5.00");
        assert_eq!(field(&out, "source"), "spamassassin");
        assert!(
            out.contains(r#""symbols":["BAYES_99","FORGED_MUA","HTML"]"#),
            "{out}"
        );
    }

    #[test]
    fn spam_false_response_is_ham() {
        let resp = b"SPAMD/1.1 0 EX_OK\r\nSpam: False ; 1.1 / 5.0\r\n\r\n";
        let out = classify_spamc(resp);
        assert_eq!(field(&out, "verdict"), VERDICT_HAM);
        assert_eq!(field(&out, "score"), "1.10");
    }

    #[test]
    fn lf_only_framing_is_tolerated() {
        // Some spamd builds / relays emit bare LF.
        let resp = b"SPAMD/1.1 0 EX_OK\nSpam: True ; 8.0 / 5.0\n\nAWL,RAZOR2";
        let out = classify_spamc(resp);
        assert_eq!(field(&out, "verdict"), VERDICT_SPAM);
        assert!(out.contains(r#""symbols":["AWL","RAZOR2"]"#), "{out}");
    }

    #[test]
    fn fail_soft_on_non_ok_code() {
        let resp = b"SPAMD/1.1 76 EX_PROTOCOL\r\n\r\n";
        assert_eq!(field(&classify_spamc(resp), "verdict"), VERDICT_UNKNOWN);
    }

    #[test]
    fn fail_soft_on_missing_spam_header() {
        let resp = b"SPAMD/1.1 0 EX_OK\r\nContent-length: 0\r\n\r\n";
        assert_eq!(field(&classify_spamc(resp), "verdict"), VERDICT_UNKNOWN);
    }

    #[test]
    fn fail_soft_on_garbage_and_empty() {
        for resp in [
            &b""[..],
            b"not a spamd frame at all",
            b"HTTP/1.1 200 OK\r\n\r\n",
            b"\x00\x01\x02\xff\xfe",
        ] {
            assert_eq!(
                field(&classify_spamc(resp), "verdict"),
                VERDICT_UNKNOWN,
                "resp={resp:?}"
            );
        }
    }

    #[test]
    fn fail_soft_on_oversized_response() {
        let big = vec![b' '; MAX_RESPONSE_BYTES + 1];
        assert_eq!(field(&classify_spamc(&big), "verdict"), VERDICT_UNKNOWN);
    }

    #[test]
    fn unknown_verdict_carries_note_and_is_never_spam() {
        let out = unknown_verdict("spamd unreachable: connection refused");
        assert_eq!(field(&out, "verdict"), VERDICT_UNKNOWN);
        assert!(out.contains("\"note\":"));
        assert!(!out.contains(r#""verdict":"spam""#));
    }

    #[test]
    fn envelope_is_valid_json_and_symbols_bounded() {
        // Build a body with more than MAX_SYMBOLS names.
        let mut body = String::new();
        for i in 0..(MAX_SYMBOLS + 30) {
            if i > 0 {
                body.push(',');
            }
            body.push_str(&format!("S{i:03}"));
        }
        let resp = format!("SPAMD/1.1 0 EX_OK\r\nSpam: True ; 9.0 / 5.0\r\n\r\n{body}");
        let out = classify_spamc(resp.as_bytes());
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("valid json envelope");
        assert_eq!(parsed["verdict"], VERDICT_SPAM);
        assert_eq!(parsed["symbols"].as_array().unwrap().len(), MAX_SYMBOLS);
    }
}
