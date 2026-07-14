// `deny` (not `forbid`) so the ONE place unsafe is unavoidable — the wit-bindgen
// `export!` ABI shim in `guest` — can opt in with a scoped `#[allow(unsafe_code)]`.
// All pure mapping code (the entire host-target lib view) stays unsafe-free.
#![deny(unsafe_code)]
//! `bridge-graph` — the Microsoft Graph account-backend bridge (plan §3 e10, §6.5).
//! **CUT-PROOF.** A first-party `wasm32-wasip2` engine plugin: a plugin backend the
//! engine cannot tell from `mw-imap`, speaking Microsoft Graph (M365 / Outlook.com /
//! Exchange-Online) entirely over the host-mediated `http-fetch` import, with OAuth
//! held/refreshed HOST-side via `oauth-token` (tokens never live in the guest).
//!
//! ## Shape (plan §1.2 — dual view)
//! * **Pure mapping** ([`graph`], [`model`], [`types`], [`mail`], [`contacts`],
//!   [`calendar`], [`todo`], [`caps`]) — target-independent Rust over a [`graph::Transport`]
//!   seam, fully host-unit-tested against recorded fixtures ([`fixtures`]).
//! * **Guest wiring** (`guest`, `cfg(target_arch = "wasm32")`) — wires the pure
//!   mapping to the frozen `mailwoman:plugin` WIT exports (`account-backend`,
//!   `addrbook-source`) over the gated host imports.
//!
//! So `cargo build --workspace` yields an inert host-target lib; `cargo build
//! -p bridge-graph --target wasm32-wasip2` (or `build.sh`) yields the real component.
//!
//! ## Coverage
//! * **Mail** (`account-backend`): folders, delta sync (+ Focused-Inbox `$Focused`/
//!   `$Other` keywords), raw fetch, flag store, move, submit, poll. The delta
//!   `@odata.deltaLink` is the opaque `SyncCursor::Plugin` the engine persists.
//! * **Contacts + GAL** (`addrbook-source`): personal contacts + people + directory.
//! * **Calendar** (calendars incl. shared, event delta, rooms, free/busy) and
//!   **To-Do** (lists + tasks): implemented + fixture-tested, but the frozen WIT world
//!   has NO calendar/task export, so they cross no plugin boundary yet (see the e10
//!   report's WIT-ABI friction note for e11/e12).
//! * **Outlook-parity caps** ([`caps`]): reactions, voting, categories, and the
//!   HONEST message-recall matrix (recall is never reported as guaranteed on Graph).

pub mod calendar;
pub mod caps;
pub mod contacts;
pub mod graph;
pub mod mail;
pub mod model;
pub mod todo;
pub mod types;

#[cfg(not(target_arch = "wasm32"))]
pub mod fixtures;

/// The manifest plugin id (matches `plugin.toml`).
pub const PLUGIN_ID: &str = "bridge-graph";

/// The plugin id, exposed as a function for host-side registry wiring.
#[must_use]
pub fn plugin_id() -> &'static str {
    PLUGIN_ID
}

// The wasm component glue lives in its own module, compiled only for the guest
// target so the host-target lib view (and clippy over it) stays pure Rust. The
// wit-bindgen `export!` shim needs `unsafe` (the C ABI boundary), permitted here.
#[cfg(target_arch = "wasm32")]
#[allow(unsafe_code)]
mod guest;
