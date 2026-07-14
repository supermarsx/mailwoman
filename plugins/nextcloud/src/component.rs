//! The `wasm32-wasip2` guest component (t7-e13). Compiled ONLY for wasm (gated by
//! `lib.rs`) so the host build never sees the wasm-import extern blocks.
//!
//! Only `message-pipeline::message-out` is a real hook; the rest of the frozen
//! `plugin` world is stubbed. Nextcloud is granted only `message-pipeline` + `net`.

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

// ── The real hook: create a Nextcloud public share link over host-mediated HTTP ─

impl pipe::Guest for Component {
    /// `raw` is a JSON share request:
    /// `{ "base_url": "https://cloud.example.com", "path": "/Documents/big.zip",
    ///    "password": "optional", "expiry": "YYYY-MM-DD optional" }`.
    ///
    /// POSTs the Nextcloud OCS create-share API via the host `http-fetch` (the host
    /// enforces the `net_allowlist` on `base_url`'s host and injects the linked
    /// account's auth) and returns the public share URL as UTF-8 bytes.
    fn message_out(raw: Vec<u8>) -> Result<Vec<u8>, PluginError> {
        let req: serde_json::Value = serde_json::from_slice(&raw)
            .map_err(|e| PluginError::Protocol(format!("malformed share request: {e}")))?;

        let base_url = req
            .get("base_url")
            .and_then(serde_json::Value::as_str)
            .map(|s| s.trim_end_matches('/'))
            .ok_or_else(|| PluginError::Unsupported("share request missing base_url".into()))?;
        let path = req
            .get("path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| PluginError::Unsupported("share request missing path".into()))?;

        // application/x-www-form-urlencoded OCS body; shareType 3 = public link.
        let mut form = format!(
            "path={}&shareType={}",
            form_encode(path),
            crate::SHARE_TYPE_PUBLIC_LINK
        );
        if let Some(pw) = req
            .get("password")
            .and_then(serde_json::Value::as_str)
            .filter(|s| !s.is_empty())
        {
            form.push_str(&format!("&password={}", form_encode(pw)));
        }
        if let Some(exp) = req
            .get("expiry")
            .and_then(serde_json::Value::as_str)
            .filter(|s| !s.is_empty())
        {
            form.push_str(&format!("&expireDate={}", form_encode(exp)));
        }

        let http_req = host::HttpRequest {
            method: "POST".into(),
            url: format!("{}{}?format=json", base_url, crate::OCS_SHARES_PATH),
            headers: vec![
                // OCS refuses requests without this header (CSRF guard).
                ("ocs-apirequest".into(), "true".into()),
                (
                    "content-type".into(),
                    "application/x-www-form-urlencoded".into(),
                ),
                ("accept".into(), "application/json".into()),
            ],
            body: Some(form.into_bytes()),
        };

        // `?` propagates the host's `capability-denied` (net not granted / host outside
        // the allowlist) — the guest never reaches the network on its own.
        let resp = host::http_fetch(&http_req)?;
        if resp.status >= 400 {
            return Err(PluginError::Transport(format!(
                "Nextcloud OCS returned HTTP {}",
                resp.status
            )));
        }
        let url = parse_share_url(&resp.body)?;
        Ok(url.into_bytes())
    }

    fn message_in(raw: Vec<u8>) -> Result<Vec<u8>, PluginError> {
        Ok(raw)
    }
}

/// Extract `ocs.data.url` from a Nextcloud OCS JSON response, checking the OCS
/// meta status (OCS returns HTTP 200 even for logical failures — the real code is in
/// `ocs.meta.statuscode`).
fn parse_share_url(body: &[u8]) -> Result<String, PluginError> {
    let v: serde_json::Value = serde_json::from_slice(body)
        .map_err(|e| PluginError::Protocol(format!("malformed OCS JSON: {e}")))?;
    let ocs = v
        .get("ocs")
        .ok_or_else(|| PluginError::Protocol("OCS response missing `ocs`".into()))?;
    if let Some(code) = ocs
        .get("meta")
        .and_then(|m| m.get("statuscode"))
        .and_then(serde_json::Value::as_i64)
    {
        // OCS "ok" is 100 (v1) or 200 (v2).
        if code != 100 && code != 200 {
            let msg = ocs
                .get("meta")
                .and_then(|m| m.get("message"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("share creation failed");
            return Err(PluginError::Protocol(format!("OCS {code}: {msg}")));
        }
    }
    ocs.get("data")
        .and_then(|d| d.get("url"))
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.is_empty())
        .map(std::string::ToString::to_string)
        .ok_or_else(|| PluginError::Protocol("OCS response missing share url".into()))
}

/// Minimal `application/x-www-form-urlencoded` percent-encoder (no dependency).
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

// ── Trivial stubs for the rest of the world (never granted to Nextcloud) ────────

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

impl dlp::Guest for Component {
    fn detect(_body: Vec<u8>) -> Result<Vec<String>, PluginError> {
        Ok(vec![])
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
