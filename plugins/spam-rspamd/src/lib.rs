//! `spam-rspamd` — the Rspamd spam-classification bridge plugin (t10 §3 e6, SPEC §10.8).
//!
//! A thin `spam-action` component: `classify(raw)` POSTs the raw RFC822 message to an
//! **rspamd** scan worker's HTTP `/checkv2` endpoint via the host `http-fetch` import —
//! **host-mediated, under a net allowlist** — and maps the JSON verdict/score/symbols
//! onto the spam WIT `classify -> result<string>` contract. NO C linkage (rspamd is a
//! network service, not a linked library), keeping the permissive license floor intact.
//!
//! ## Verdict contract (what the `classify` string carries)
//! `classify` returns `Ok(<json>)` where `<json>` is a compact, deterministic object:
//! ```json
//! {"verdict":"spam","action":"reject","score":15.00,"threshold":5.00,
//!  "symbols":["BAYES_SPAM"],"source":"rspamd"}
//! ```
//! `verdict` is one of [`VERDICT_HAM`] / [`VERDICT_SPAM`] / [`VERDICT_UNKNOWN`].
//!
//! ## Fail-soft posture (never a panic, never a hard block)
//! An unreachable daemon, a non-2xx status, an over-sized body, or malformed JSON all
//! resolve to an explicit [`VERDICT_UNKNOWN`] verdict — the message is never hard-blocked
//! and the guest never panics. All parsing is bounded ([`MAX_RESPONSE_BYTES`]) and
//! hostile-input safe (daemon-supplied symbol strings are JSON-escaped on the way out).
//!
//! The pure protocol/verdict logic lives here (host- and wasm-buildable, unit-tested on
//! the host); the `wasm32-wasip2` guest in [`component`] is a thin shim that only does
//! the host `http-fetch` and feeds the outcome into these functions.

// Forbid unsafe in our own (host + pure) code; the `wasm32` guest pulls in
// wit-bindgen's generated ABI glue, which necessarily uses `unsafe`, so the lint is
// scoped to the non-wasm build (where all first-party logic is exercised + tested).
#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

/// The manifest plugin id.
pub const PLUGIN_ID: &str = "spam-rspamd";

/// The rspamd scan-worker check path (appended to the configured base URL).
pub const RSPAMD_CHECK_PATH: &str = "/checkv2";

/// Default rspamd endpoint base — the scan worker's HTTP port. Overridable per
/// deployment via the `endpoint` KV config key (see [`component`]); the host `port` must
/// be in the plugin `net_allowlist` regardless.
pub const DEFAULT_ENDPOINT: &str = "http://rspamd:11333";

/// Verdict: the message is not spam (rspamd `no action` / `greylist`, or score below the
/// required threshold).
pub const VERDICT_HAM: &str = "ham";
/// Verdict: the message is spam (rspamd `reject` / `add header` / `rewrite subject` /
/// `soft reject`, or score at/above the required threshold).
pub const VERDICT_SPAM: &str = "spam";
/// Verdict: classification could not be determined (fail-soft). NEVER a hard block.
pub const VERDICT_UNKNOWN: &str = "unknown";

/// Upper bound on a daemon response we will parse (hostile-input guard). A larger body
/// resolves to [`VERDICT_UNKNOWN`] rather than being parsed.
pub const MAX_RESPONSE_BYTES: usize = 1 << 20; // 1 MiB

/// Cap on the number of symbols carried in a verdict (bounded output).
pub const MAX_SYMBOLS: usize = 64;

#[must_use]
pub fn plugin_id() -> &'static str {
    PLUGIN_ID
}

/// Build the full `/checkv2` URL from an endpoint base (trailing slash tolerated).
#[must_use]
pub fn check_url(endpoint: &str) -> String {
    let base = endpoint.trim_end_matches('/');
    format!("{base}{RSPAMD_CHECK_PATH}")
}

/// Map an rspamd `/checkv2` outcome (HTTP status + body) to a verdict string.
///
/// Fail-soft: a non-2xx status, an over-sized body, or malformed/unexpected JSON all
/// resolve to [`VERDICT_UNKNOWN`] — never a panic, never a hard block.
#[must_use]
pub fn classify_rspamd(status: u16, body: &[u8]) -> String {
    if status >= 400 {
        return unknown_verdict(&format!("rspamd http status {status}"));
    }
    if body.len() > MAX_RESPONSE_BYTES {
        return unknown_verdict("rspamd response exceeds parse bound");
    }
    let v: serde_json::Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return unknown_verdict("malformed rspamd json"),
    };

    let action = v
        .get("action")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let score = finite(v.get("score").and_then(serde_json::Value::as_f64));
    // rspamd emits `required_score`; tolerate the hyphenated spelling too.
    let threshold = v
        .get("required_score")
        .or_else(|| v.get("required-score"))
        .and_then(serde_json::Value::as_f64)
        .filter(|f| f.is_finite());
    let symbols = extract_symbols(&v);

    let verdict = rspamd_verdict(&action, score, threshold);
    verdict_json(
        verdict,
        &action,
        score,
        threshold.unwrap_or(0.0),
        &symbols,
        "",
    )
}

/// The explicit fail-soft verdict (unreachable daemon, denied net, transport error).
#[must_use]
pub fn unknown_verdict(note: &str) -> String {
    verdict_json(VERDICT_UNKNOWN, "", 0.0, 0.0, &[], note)
}

/// Decide ham/spam from the rspamd action, falling back to score-vs-threshold when the
/// action is absent/unrecognized. A well-formed response with no decidable signal is
/// treated as ham (fail-open) rather than blocking.
fn rspamd_verdict(action: &str, score: f64, threshold: Option<f64>) -> &'static str {
    match action {
        "reject" | "add header" | "add_header" | "rewrite subject" | "rewrite_subject"
        | "soft reject" | "soft_reject" => VERDICT_SPAM,
        "no action" | "no_action" | "greylist" => VERDICT_HAM,
        _ => match threshold {
            Some(t) if score >= t => VERDICT_SPAM,
            Some(_) => VERDICT_HAM,
            None => VERDICT_HAM,
        },
    }
}

/// Collect symbol names from an rspamd `symbols` object (keyed by symbol name), sorted
/// for determinism and bounded to [`MAX_SYMBOLS`].
fn extract_symbols(v: &serde_json::Value) -> Vec<String> {
    let mut out: Vec<String> = match v.get("symbols").and_then(serde_json::Value::as_object) {
        Some(map) => map.keys().cloned().collect(),
        None => Vec::new(),
    };
    out.sort();
    out.truncate(MAX_SYMBOLS);
    out
}

fn finite(x: Option<f64>) -> f64 {
    match x {
        Some(f) if f.is_finite() => f,
        _ => 0.0,
    }
}

/// Hand-build the deterministic verdict JSON (no serialize dep; daemon-supplied strings
/// are escaped so a hostile symbol name cannot break the envelope).
fn verdict_json(
    verdict: &str,
    action: &str,
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
    out.push_str(",\"action\":");
    json_str(&mut out, action);
    out.push_str(&format!(",\"score\":{:.2}", num(score)));
    out.push_str(&format!(",\"threshold\":{:.2}", num(threshold)));
    out.push_str(",\"symbols\":");
    out.push_str(&syms);
    out.push_str(",\"source\":\"rspamd\"");
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
        // Tiny extractor for the deterministic envelope (test-only).
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
    fn check_url_appends_path_and_trims_slash() {
        assert_eq!(
            check_url("http://rspamd:11333"),
            "http://rspamd:11333/checkv2"
        );
        assert_eq!(
            check_url("http://rspamd:11333/"),
            "http://rspamd:11333/checkv2"
        );
    }

    #[test]
    fn checkv2_reject_action_is_spam_with_symbols() {
        let body = br#"{
            "action": "reject",
            "score": 15.3,
            "required_score": 5.0,
            "symbols": {
                "FORGED_MUA": {"name":"FORGED_MUA","score":3.1},
                "BAYES_SPAM": {"name":"BAYES_SPAM","score":5.1}
            }
        }"#;
        let out = classify_rspamd(200, body);
        assert_eq!(field(&out, "verdict"), VERDICT_SPAM);
        assert_eq!(field(&out, "action"), "reject");
        assert_eq!(field(&out, "score"), "15.30");
        assert_eq!(field(&out, "threshold"), "5.00");
        assert_eq!(field(&out, "source"), "rspamd");
        // Symbols are sorted deterministically.
        assert!(
            out.contains(r#""symbols":["BAYES_SPAM","FORGED_MUA"]"#),
            "{out}"
        );
    }

    #[test]
    fn checkv2_no_action_is_ham() {
        let body = br#"{"action":"no action","score":0.1,"required_score":5.0,"symbols":{}}"#;
        let out = classify_rspamd(200, body);
        assert_eq!(field(&out, "verdict"), VERDICT_HAM);
        assert_eq!(field(&out, "action"), "no action");
    }

    #[test]
    fn checkv2_unknown_action_falls_back_to_score_vs_threshold() {
        // No recognizable action; score >= threshold ⇒ spam.
        let spam = classify_rspamd(
            200,
            br#"{"action":"custom","score":9.0,"required_score":5.0}"#,
        );
        assert_eq!(field(&spam, "verdict"), VERDICT_SPAM);
        // score < threshold ⇒ ham.
        let ham = classify_rspamd(
            200,
            br#"{"action":"custom","score":1.0,"required_score":5.0}"#,
        );
        assert_eq!(field(&ham, "verdict"), VERDICT_HAM);
    }

    #[test]
    fn fail_soft_on_http_error_status() {
        let out = classify_rspamd(502, b"upstream boom");
        assert_eq!(field(&out, "verdict"), VERDICT_UNKNOWN);
        assert_eq!(field(&out, "source"), "rspamd");
    }

    #[test]
    fn fail_soft_on_garbage_body() {
        for body in [&b"not json at all"[..], b"{", b"", b"\x00\x01\x02\xff"] {
            let out = classify_rspamd(200, body);
            assert_eq!(field(&out, "verdict"), VERDICT_UNKNOWN, "body={body:?}");
        }
    }

    #[test]
    fn fail_soft_on_oversized_body() {
        let big = vec![b' '; MAX_RESPONSE_BYTES + 1];
        let out = classify_rspamd(200, &big);
        assert_eq!(field(&out, "verdict"), VERDICT_UNKNOWN);
    }

    #[test]
    fn unknown_verdict_carries_note_and_is_never_spam() {
        let out = unknown_verdict("rspamd unreachable: connection refused");
        assert_eq!(field(&out, "verdict"), VERDICT_UNKNOWN);
        assert!(out.contains("\"note\":"));
        assert!(!out.contains(r#""verdict":"spam""#));
    }

    #[test]
    fn hostile_symbol_name_is_escaped() {
        // A daemon-supplied symbol containing a quote must not break the envelope.
        let body = br#"{"action":"reject","score":10.0,"required_score":5.0,
            "symbols":{"EVIL\"NAME":{"score":1.0}}}"#;
        let out = classify_rspamd(200, body);
        // Still valid: parses back with serde and the verdict is intact.
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("valid json envelope");
        assert_eq!(parsed["verdict"], VERDICT_SPAM);
        assert_eq!(parsed["symbols"][0], "EVIL\"NAME");
    }

    #[test]
    fn symbol_count_is_bounded() {
        let mut map = String::from("{");
        for i in 0..(MAX_SYMBOLS + 20) {
            if i > 0 {
                map.push(',');
            }
            map.push_str(&format!("\"S{i:03}\":{{\"score\":1.0}}"));
        }
        map.push('}');
        let body = format!(
            "{{\"action\":\"reject\",\"score\":9.0,\"required_score\":5.0,\"symbols\":{map}}}"
        );
        let out = classify_rspamd(200, body.as_bytes());
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["symbols"].as_array().unwrap().len(), MAX_SYMBOLS);
    }
}
