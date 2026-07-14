//! Server-side browser-error scrubber tunnel (SPEC §21, plan §3 e9). Owned by
//! t6-e9; mounted by e11.
//!
//! The web app reports client-side errors to **same-origin** `/errors` (so the CSP
//! stays `connect-src 'self'` — no third-party error sink is contacted by the
//! browser). This handler SCRUBS any mail content — addresses, subjects, bodies,
//! and free-text that looks like an address — out of the report, then optionally
//! forwards the scrubbed payload to an operator-configured sink over the in-tree
//! `reqwest` (server-side, so the browser never talks to it).
//!
//! ## Sentry (VET-BEFORE-ENABLE, plan §5 / §6 R6)
//! A Sentry forward is intentionally NOT linked here: the `sentry` crate is off by
//! default and its dependency tree was not vetted for the rustls/no-openssl floor
//! in this milestone. Instead we ship the scrubber + a generic DSN forward
//! ([`ErrorConfig::forward_url`]); the payload is Sentry-envelope-shaped enough that
//! e12 can point it at a Sentry-compatible relay later without touching the browser
//! path. Enabling the real `sentry` SDK remains a documented deferral.

use std::sync::RwLock;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::Value;

use crate::observability::redact_address;

/// Keys whose string values are mail-derived and MUST be dropped wholesale before a
/// report leaves the process, regardless of nesting depth (case-insensitive).
const SENSITIVE_KEYS: &[&str] = &[
    "subject",
    "body",
    "text",
    "html",
    "snippet",
    "preview",
    "from",
    "to",
    "cc",
    "bcc",
    "sender",
    "recipient",
    "recipients",
    "address",
    "email",
    "displayname",
    "display_name",
    "mailbox",
    "envelope",
    "password",
    "authorization",
    "cookie",
];

/// Operator config for the `/errors` sink. Both fields default to off — with no
/// forward URL the scrubbed report is logged and dropped (still same-origin safe).
#[derive(Debug, Clone, Default)]
pub struct ErrorConfig {
    /// Where to POST the SCRUBBED report server-side (env: `MW_ERROR_FORWARD_URL`).
    /// A DSN-style URL to a Sentry-compatible relay or any collector. `None` → log
    /// only.
    pub forward_url: Option<String>,
}

impl ErrorConfig {
    /// Populate from the environment (wired by e11).
    pub fn from_env() -> Self {
        Self {
            forward_url: std::env::var("MW_ERROR_FORWARD_URL")
                .ok()
                .filter(|v| !v.is_empty()),
        }
    }
}

/// Process-wide `/errors` config. Set by e11 from [`ErrorConfig::from_env`] in
/// `build_app`; read by [`report_error`]. Global (not an `AppState` field) so the
/// module stays self-contained — e11 mounts the route without touching this file.
static ERROR_CONFIG: RwLock<Option<ErrorConfig>> = RwLock::new(None);

/// Install the `/errors` sink config (wired by e11).
pub fn set_error_config(config: ErrorConfig) {
    *ERROR_CONFIG.write().expect("error config lock") = Some(config);
}

/// The configured forward URL, if any.
fn forward_url() -> Option<String> {
    ERROR_CONFIG
        .read()
        .expect("error config lock")
        .as_ref()
        .and_then(|c| c.forward_url.clone())
}

/// Recursively scrub a JSON error report IN PLACE:
///   * any value under a [`SENSITIVE_KEYS`] key becomes `"[redacted]"`;
///   * every remaining string has embedded email addresses rewritten to
///     `[address]` (a stray address in a stack trace or free-text message).
///
/// Numbers/bools/nulls pass through — they carry no mail content. The scrub is the
/// testable core of the tunnel (`/errors` acceptance).
pub fn scrub(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, v) in map.iter_mut() {
                if is_sensitive_key(key) {
                    *v = Value::String("[redacted]".to_string());
                } else {
                    scrub(v);
                }
            }
        }
        Value::Array(items) => {
            for v in items.iter_mut() {
                scrub(v);
            }
        }
        Value::String(s) => {
            if let Some(clean) = scrub_addresses(s) {
                *s = clean;
            }
        }
        _ => {}
    }
}

/// Whether a JSON key names a mail-content field (case-insensitive, ignoring `-`/`_`).
fn is_sensitive_key(key: &str) -> bool {
    let norm: String = key
        .chars()
        .filter(|c| *c != '_' && *c != '-')
        .flat_map(char::to_lowercase)
        .collect();
    SENSITIVE_KEYS
        .iter()
        .any(|k| k.chars().filter(|c| *c != '_').collect::<String>() == norm)
}

/// Rewrite every `local@domain` token in `s` to the redaction marker. Returns
/// `Some(cleaned)` when at least one address was found, else `None` (no allocation
/// on the common no-address path). Deliberately conservative — a hand-rolled scan
/// (no regex dep) that treats any `word@word.tld`-ish run as an address.
fn scrub_addresses(s: &str) -> Option<String> {
    if !s.contains('@') {
        return None;
    }
    let bytes = s.as_bytes();
    let is_atom = |c: u8| {
        c.is_ascii_alphanumeric()
            || matches!(c, b'.' | b'_' | b'%' | b'+' | b'-' | b'=' | b'\'' | b'/')
    };
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    let mut found = false;
    while i < bytes.len() {
        if bytes[i] == b'@' {
            // Walk back over the local part already pushed into `out`.
            let mut local = out.len();
            while local > 0 && is_atom(out.as_bytes()[local - 1]) {
                local -= 1;
            }
            // Walk forward over the domain part.
            let mut j = i + 1;
            while j < bytes.len() && (is_atom(bytes[j]) && bytes[j] != b'@') {
                j += 1;
            }
            let has_local = local < out.len();
            let has_domain = j > i + 1 && s[i + 1..j].contains('.');
            if has_local && has_domain {
                out.truncate(local);
                out.push_str(redact_address(s));
                i = j;
                found = true;
                continue;
            }
        }
        // Push this char (handles multi-byte UTF-8 correctly by copying the char).
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    found.then_some(out)
}

/// `POST /errors` — accept a browser error report, scrub mail content, and forward
/// the scrubbed payload to the configured sink (or log it). Deliberately
/// UNAUTHENTICATED: errors happen on the login screen too, before a session exists;
/// the scrubber guarantees nothing sensitive leaves regardless. Always returns
/// `202` so a reporting failure never cascades into more client errors.
pub async fn report_error(Json(mut report): Json<Value>) -> Response {
    scrub(&mut report);
    match forward_url() {
        Some(url) => {
            let http = reqwest::Client::new();
            match http.post(&url).json(&report).send().await {
                Ok(resp) => tracing::debug!("scrubbed error report forwarded: {}", resp.status()),
                Err(e) => tracing::warn!("error-report forward failed: {e}"),
            }
        }
        None => {
            // Log only the scrubbed shape (top-level keys), never nested content.
            let keys: Vec<&str> = report
                .as_object()
                .map(|m| m.keys().map(String::as_str).collect())
                .unwrap_or_default();
            tracing::info!("browser error report received (scrubbed); fields: {keys:?}");
        }
    }
    StatusCode::ACCEPTED.into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn to_text(v: &Value) -> String {
        serde_json::to_string(v).unwrap()
    }

    #[test]
    fn scrubs_sensitive_keys_wholesale() {
        // The container key ("context") is NOT sensitive, so nested opaque ids
        // survive while the mail-content fields inside it are scrubbed.
        let mut report = json!({
            "message": "render failed",
            "context": {
                "subject": "Q3 acquisition terms — CONFIDENTIAL",
                "from": "ceo@acme.example",
                "body": "The wire instructions are ...",
                "id": "m-42"
            },
            "level": "error"
        });
        scrub(&mut report);
        let text = to_text(&report);
        assert!(!text.contains("acquisition"), "subject leaked: {text}");
        assert!(!text.contains("wire instructions"), "body leaked: {text}");
        assert!(!text.contains("ceo@acme.example"), "address leaked: {text}");
        // Non-sensitive structure + opaque ids survive.
        assert!(text.contains("render failed"));
        assert!(text.contains("m-42"));
        assert_eq!(report["context"]["subject"], "[redacted]");
    }

    #[test]
    fn scrubs_addresses_embedded_in_free_text() {
        let mut report = json!({
            "stack": "at parseFrom (alice@example.com) line 12; cc bob.smith+tag@mail.co.uk here",
            "count": 3
        });
        scrub(&mut report);
        let text = to_text(&report);
        assert!(!text.contains("alice@example.com"), "addr1 leaked: {text}");
        assert!(!text.contains("bob.smith"), "addr2 leaked: {text}");
        assert!(text.contains("[address]"));
        assert!(text.contains("parseFrom"));
        assert_eq!(report["count"], 3);
    }

    #[test]
    fn leaves_address_free_text_untouched() {
        // No `@` → no allocation, no change.
        assert!(scrub_addresses("plain error, no address").is_none());
        // A lone `@` that is not an address is preserved.
        assert!(scrub_addresses("cost @ scale").is_none());
    }

    #[test]
    fn sensitive_key_matching_ignores_case_and_separators() {
        assert!(is_sensitive_key("Subject"));
        assert!(is_sensitive_key("display_name"));
        assert!(is_sensitive_key("displayName"));
        assert!(!is_sensitive_key("id"));
        assert!(!is_sensitive_key("message"));
    }
}
