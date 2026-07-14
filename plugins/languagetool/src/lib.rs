#![forbid(unsafe_code)]
#![allow(dead_code)]
//! `languagetool` — the no-AI grammar plugin (plan §3 e13, SPEC §10.3/§22).
//! The smallest real WASM plugin: the canonical "real component loaded in the jail"
//! proof for e16. SCAFFOLD stub (e0): inert on the host target; the
//! `wasm32-wasip2` component (a message-out/DLP hook calling a LanguageTool HTTP
//! endpoint via host `http-fetch` under a net allowlist) is filled by e13, built e15.

/// The manifest plugin id.
pub const PLUGIN_ID: &str = "languagetool";

#[must_use]
pub fn plugin_id() -> &'static str {
    PLUGIN_ID
}
