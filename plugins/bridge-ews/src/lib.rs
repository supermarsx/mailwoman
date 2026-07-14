#![forbid(unsafe_code)]
#![allow(dead_code)]
//! `bridge-ews` — on-prem Exchange EWS account-backend bridge (plan §3 e11, §6.5).
//! **CUT-PROOF** (must land). SCAFFOLD stub (e0): inert on the host target; the
//! `wasm32-wasip2` SOAP component (sync/send/calendar/free-busy/GAL/OOF/recall/
//! voting; Basic + pure-Rust NTLM; Kerberos is a documented gap, R2) is e11/e15.

/// The manifest plugin id.
pub const PLUGIN_ID: &str = "bridge-ews";

#[must_use]
pub fn plugin_id() -> &'static str {
    PLUGIN_ID
}
