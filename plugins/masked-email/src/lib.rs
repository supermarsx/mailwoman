//! `masked-email` — the masked-email plugin (t10 §3 e7, SPEC §28.4).
//!
//! A thin `message-pipeline` (message-out) component: on send it rewrites the
//! outgoing message's sender to the per-target masked alias the composer selected,
//! so the recipient only ever sees the alias. The alias lifecycle itself
//! (generate/enable/disable/delete + target binding) is server-side — the 0010
//! `masked_email` table via `mw-store`, surfaced by `mw-server/src/masked.rs` — so
//! this component only performs the on-send envelope/header rewrite.
//!
//! This is the **t10-e0 scaffold**: `message-out` is an identity passthrough and the
//! rest of the frozen `plugin` world is stubbed (the PIM/parity interfaces advertise
//! `false`). e7 fills the alias rewrite.

/// The manifest plugin id.
pub const PLUGIN_ID: &str = "masked-email";

#[must_use]
pub fn plugin_id() -> &'static str {
    PLUGIN_ID
}

#[cfg(target_arch = "wasm32")]
mod component;
