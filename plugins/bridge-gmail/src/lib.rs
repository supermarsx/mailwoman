#![allow(dead_code)]
//! `bridge-gmail` — the Gmail API account-backend bridge (plan §3 e12, SPEC §6.5).
//!
//! A first-party **`wasm32-wasip2` WASM engine plugin** that implements the frozen
//! `mailwoman:plugin` `account-backend` export over the Gmail REST API, giving the
//! engine an account backend indistinguishable from `mw-imap`. It delivers **true
//! label semantics** (Gmail labels ↔ the engine's mailbox/flag model), **history-ID
//! delta sync** (via the additive `SyncCursor::Plugin` cursor), and a **per-user
//! OAuth client** — OAuth tokens are acquired host-side (`oauth-token`) and never
//! enter the guest, and all HTTP is host-mediated (`http-fetch`) under the
//! manifest `net_allowlist` (`gmail.googleapis.com` + `oauth2.googleapis.com`).
//!
//! FIRST scope-cut candidate (§27 ladder) — IMAP+XOAUTH2 covers most Workspace
//! users — but it is a complete, real bridge.
//!
//! ## Layout
//! * [`model`] — transport-neutral value types + the frozen-ABI JSON smuggles.
//! * [`labels`] — Gmail label ↔ role/flag semantics (the whole quirk surface).
//! * [`gmail`] — Gmail REST v1 wire types + URL builders.
//! * [`backend`] — [`backend::GmailBackend`], generic over a [`backend::Transport`];
//!   pure and host-unit-tested.
//! * `component` (only on `target_family = "wasm"`) — the `wit-bindgen` glue: a
//!   `Transport` over the host imports + the `account-backend` `Guest` impls.
//!
//! The host build (`cargo build --workspace`) and the integration test compile only
//! the pure crate above; the wasm component is built by `build.sh` and committed at
//! `fixtures/bridge-gmail.wasm`, which `tests/bridge.rs` loads through `mw-plugin`.

pub mod backend;
pub mod gmail;
pub mod labels;
pub mod model;

#[cfg(target_family = "wasm")]
mod component;

/// The manifest plugin id (matches `plugin.toml`).
pub const PLUGIN_ID: &str = "bridge-gmail";

#[must_use]
pub fn plugin_id() -> &'static str {
    PLUGIN_ID
}
