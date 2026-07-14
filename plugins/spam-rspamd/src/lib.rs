//! `spam-rspamd` — the Rspamd spam-trainer plugin (t10 §3 e6, SPEC §10.8).
//!
//! A thin `spam-action` component: `classify(raw)` posts the raw message to an
//! rspamd controller (`/checkv2`) via the host `http-fetch` import — **host-mediated,
//! under a net allowlist** — and returns the action verdict; ham/spam training posts
//! to `/learnham`/`/learnspam`. NO C linkage (rspamd is a network service, not a
//! linked library), keeping the permissive license floor intact.
//!
//! This is the **t10-e0 scaffold**: the guest exports the whole `plugin` world with
//! trivial stubs (the PIM/parity interfaces advertise `false`), and `classify`
//! returns `unsupported` until e6 fills the rspamd protocol. Build shapes mirror
//! `languagetool` (guest behind `#[cfg(target_arch = "wasm32")]`).

/// The manifest plugin id.
pub const PLUGIN_ID: &str = "spam-rspamd";

/// The rspamd controller check path (appended to the configured base URL).
pub const RSPAMD_CHECK_PATH: &str = "/checkv2";

#[must_use]
pub fn plugin_id() -> &'static str {
    PLUGIN_ID
}

#[cfg(target_arch = "wasm32")]
mod component;
