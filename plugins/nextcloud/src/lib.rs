#![forbid(unsafe_code)]
#![allow(dead_code)]
//! `nextcloud` — Nextcloud attach/save/share-link plugin (plan §8, SPEC §18.4).
//! SCAFFOLD stub (e0): inert on the host target; the `wasm32-wasip2` component
//! (OCS/WebDAV share-link creation via host `http-fetch`) is built by e15.

/// The manifest plugin id.
pub const PLUGIN_ID: &str = "nextcloud";

#[must_use]
pub fn plugin_id() -> &'static str {
    PLUGIN_ID
}
