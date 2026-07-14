//! `languagetool` — the no-AI grammar plugin (plan §3 e13, SPEC §10.3/§22).
//!
//! The **smallest real WASM plugin** and the canonical "a real signed plugin loaded
//! in the jail" artifact (e16). It implements the `dlp-detect` guest export: given a
//! draft body it POSTs the text to a LanguageTool `/v2/check` endpoint via the host
//! `http-fetch` import — **host-mediated, under a net allowlist** — and returns the
//! grammar/spelling suggestions. The guest never opens a socket; a target host
//! outside the manifest `net_allowlist` is refused by the host with
//! `capability-denied`, which the plugin propagates.
//!
//! Build shapes (see `Cargo.toml`): the guest bindings live behind
//! `#[cfg(target_arch = "wasm32")]` in [`component`], so the host build is an inert
//! `rlib` exposing only the id constants below while the `wasm32-wasip2` build
//! produces a real component.

/// The manifest plugin id.
pub const PLUGIN_ID: &str = "languagetool";

/// The default LanguageTool endpoint host the component reaches (must appear in the
/// plugin's `net_allowlist` for the host to permit the fetch). A self-hosted
/// LanguageTool deployment allowlists its own host instead (§10.3).
pub const DEFAULT_ENDPOINT_HOST: &str = "api.languagetool.org";

/// The LanguageTool check path the component POSTs the draft to.
pub const CHECK_PATH: &str = "/v2/check";

#[must_use]
pub fn plugin_id() -> &'static str {
    PLUGIN_ID
}

#[cfg(target_arch = "wasm32")]
mod component;
