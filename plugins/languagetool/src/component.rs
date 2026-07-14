//! The `wasm32-wasip2` guest component (t7-e13). Compiled ONLY for wasm (the module
//! is `#[cfg(target_arch = "wasm32")]`-gated by `lib.rs`) so the host build never
//! sees the wasm-import extern blocks wit-bindgen emits.
//!
//! Only `dlp-detect::detect` is a real hook; every other export in the frozen
//! `plugin` world is a trivial stub — a plugin implements the whole world but the
//! host only ever CALLS the capability-granted hooks, and LanguageTool is granted
//! only `dlp-detector` + `net`.

wit_bindgen::generate!({
    world: "plugin",
    path: "../../crates/mw-plugin/wit",
});

use exports::mailwoman::plugin::account_backend as ab;
use exports::mailwoman::plugin::addrbook_source as addr;
use exports::mailwoman::plugin::autoconfig_source as autoc;
use exports::mailwoman::plugin::dlp_detect as dlp;
use exports::mailwoman::plugin::message_pipeline as pipe;
use exports::mailwoman::plugin::spam_action as spam;

use mailwoman::plugin::host;
use mailwoman::plugin::types::{
    BackendCaps, ChangeEvent, Flag, Mailbox, MailboxDelta, MailboxRef, MessageRef, PluginError,
    RawMessage, SyncCursor,
};

struct Component;

// ── The real hook: grammar/spelling check over host-mediated HTTP ──────────────

impl dlp::Guest for Component {
    /// Treat `body` as the draft text (UTF-8), POST it to the LanguageTool
    /// `/v2/check` endpoint via the host `http-fetch` (the host enforces the
    /// `net_allowlist` — an out-of-allowlist host comes back as `capability-denied`,
    /// which we propagate), and return one suggestion string per reported match.
    fn detect(body: Vec<u8>) -> Result<Vec<String>, PluginError> {
        let text = String::from_utf8_lossy(&body);
        // application/x-www-form-urlencoded: `text=<draft>&language=auto`.
        let form = format!("text={}&language=auto", form_encode(&text));

        let req = host::HttpRequest {
            method: "POST".into(),
            url: format!(
                "https://{}{}",
                crate::DEFAULT_ENDPOINT_HOST,
                crate::CHECK_PATH
            ),
            headers: vec![
                (
                    "content-type".into(),
                    "application/x-www-form-urlencoded".into(),
                ),
                ("accept".into(), "application/json".into()),
            ],
            body: Some(form.into_bytes()),
        };

        // `?` propagates the host's `capability-denied` (net not granted / host outside
        // the allowlist) straight back to the caller — the plugin never reaches the
        // network on its own.
        let resp = host::http_fetch(&req)?;
        if resp.status >= 400 {
            return Err(PluginError::Transport(format!(
                "languagetool endpoint returned {}",
                resp.status
            )));
        }
        parse_suggestions(&resp.body)
    }
}

/// Parse a LanguageTool `/v2/check` JSON response into human-readable suggestion
/// strings: `"<message> [→ <replacement>]"` for each match.
fn parse_suggestions(body: &[u8]) -> Result<Vec<String>, PluginError> {
    let v: serde_json::Value = serde_json::from_slice(body)
        .map_err(|e| PluginError::Protocol(format!("malformed LanguageTool JSON: {e}")))?;
    let mut out = Vec::new();
    if let Some(matches) = v.get("matches").and_then(|m| m.as_array()) {
        for m in matches {
            let message = m
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .trim();
            if message.is_empty() {
                continue;
            }
            let replacement = m
                .get("replacements")
                .and_then(serde_json::Value::as_array)
                .and_then(|r| r.first())
                .and_then(|r| r.get("value"))
                .and_then(serde_json::Value::as_str);
            match replacement {
                Some(rep) if !rep.is_empty() => out.push(format!("{message} → {rep}")),
                _ => out.push(message.to_string()),
            }
        }
    }
    Ok(out)
}

/// Minimal `application/x-www-form-urlencoded` percent-encoder (no dependency): keep
/// the RFC 3986 unreserved set, percent-encode every other byte. Deliberately
/// conservative — correctness over compactness for a grammar payload.
fn form_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

// ── Trivial stubs for the rest of the world (never granted to LanguageTool) ─────

impl ab::Guest for Component {
    fn capabilities() -> Result<BackendCaps, PluginError> {
        Err(PluginError::Unsupported("account-backend".into()))
    }
    fn list_mailboxes() -> Result<Vec<Mailbox>, PluginError> {
        Err(PluginError::Unsupported("account-backend".into()))
    }
    fn sync_mailbox(_mbox: MailboxRef, _cursor: SyncCursor) -> Result<MailboxDelta, PluginError> {
        Err(PluginError::Unsupported("account-backend".into()))
    }
    fn fetch_raw(_refs: Vec<MessageRef>) -> Result<Vec<RawMessage>, PluginError> {
        Err(PluginError::Unsupported("account-backend".into()))
    }
    fn store_flags(
        _refs: Vec<MessageRef>,
        _add: Vec<Flag>,
        _remove: Vec<Flag>,
    ) -> Result<(), PluginError> {
        Err(PluginError::Unsupported("account-backend".into()))
    }
    fn move_messages(_refs: Vec<MessageRef>, _to: MailboxRef) -> Result<(), PluginError> {
        Err(PluginError::Unsupported("account-backend".into()))
    }
    fn submit(
        _mbox: MailboxRef,
        _raw: Vec<u8>,
        _flags: Vec<Flag>,
    ) -> Result<MessageRef, PluginError> {
        Err(PluginError::Unsupported("account-backend".into()))
    }
    fn poll_changes() -> Result<Vec<ChangeEvent>, PluginError> {
        Err(PluginError::Unsupported("account-backend".into()))
    }
}

impl pipe::Guest for Component {
    fn message_in(raw: Vec<u8>) -> Result<Vec<u8>, PluginError> {
        Ok(raw)
    }
    fn message_out(raw: Vec<u8>) -> Result<Vec<u8>, PluginError> {
        Ok(raw)
    }
}

impl addr::Guest for Component {
    fn search(_query: String) -> Result<Vec<String>, PluginError> {
        Err(PluginError::Unsupported("addrbook-source".into()))
    }
}

impl autoc::Guest for Component {
    fn lookup(_email: String) -> Result<Option<String>, PluginError> {
        Ok(None)
    }
}

impl spam::Guest for Component {
    fn classify(_raw: Vec<u8>) -> Result<String, PluginError> {
        Err(PluginError::Unsupported("spam-action".into()))
    }
}

export!(Component);
