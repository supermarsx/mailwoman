#![forbid(unsafe_code)]
#![allow(dead_code)]
//! `bridge-gmail` — Gmail API account-backend bridge (plan §3 e12, §6.5).
//! FIRST scope-cut candidate (§27 ladder). SCAFFOLD stub (e0): inert on the host
//! target; the `wasm32-wasip2` component (label semantics + history-ID delta sync +
//! per-user OAuth) is filled by e12 and built by e15.

/// The manifest plugin id.
pub const PLUGIN_ID: &str = "bridge-gmail";

#[must_use]
pub fn plugin_id() -> &'static str {
    PLUGIN_ID
}
