//! `spam-spamassassin` — the SpamAssassin spam-trainer plugin (t10 §3 e6, SPEC §10.8).
//!
//! A thin `spam-action` component: `classify(raw)` checks the message against a
//! SpamAssassin `spamd`/`spamc`-compatible HTTP endpoint via the host `http-fetch`
//! import — **host-mediated, under a net allowlist** — and returns the verdict;
//! ham/spam training feeds `sa-learn` equivalents. NO C linkage (spamd is a network
//! service), keeping the permissive license floor intact.
//!
//! This is the **t10-e0 scaffold**: the guest exports the whole `plugin` world with
//! trivial stubs (the PIM/parity interfaces advertise `false`), and `classify`
//! returns `unsupported` until e6 fills the spamd protocol.

/// The manifest plugin id.
pub const PLUGIN_ID: &str = "spam-spamassassin";

#[must_use]
pub fn plugin_id() -> &'static str {
    PLUGIN_ID
}

#[cfg(target_arch = "wasm32")]
mod component;
