#![forbid(unsafe_code)]
#![allow(dead_code)]
//! `bridge-graph` — Microsoft Graph account-backend bridge (plan §3 e10, §6.5).
//! **CUT-PROOF** (must land). SCAFFOLD stub (e0): inert on the host target; the
//! `wasm32-wasip2` component (WIT `account-backend` export + reactions/voting/
//! recall/Focused-Inbox-sync caps, OAuth device/auth-code) is filled by e10 and
//! built by e15.

/// The manifest plugin id (matches the `plugin.toml` `id` e10 authors).
pub const PLUGIN_ID: &str = "bridge-graph";

/// Placeholder entry so the crate compiles as a host lib until e10 fills it.
#[must_use]
pub fn plugin_id() -> &'static str {
    PLUGIN_ID
}
