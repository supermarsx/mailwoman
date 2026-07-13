//! WASM boundary for `mw-sanitize` (plan §1.3 / risk #5).
//!
//! Exposes the existing [`crate::sanitize_email_html`] policy to the browser crypto
//! Web Worker (`apps/web/src/crypto/worker.entry.ts`) as `sanitizeEmailHtml`. After a
//! message is DECRYPTED client-side (mw-crypto wasm), its HTML is sanitized HERE,
//! in-worker, before it enters the sandboxed message-body iframe — so decrypted
//! end-to-end-encrypted plaintext is NEVER round-tripped to the server sanitizer
//! (that would defeat E2EE / pre-stage the V6 zero-access break — plan §1.3).
//!
//! Gated on `cfg(target_arch = "wasm32")` so the native engine consumers
//! (mw-render / mw-export / mw-server) never link wasm-bindgen; the sanitize policy
//! itself is target-agnostic (`ammonia` is pure-Rust + wasm-compatible).

use wasm_bindgen::prelude::*;

/// `sanitizeEmailHtml(html)` → sanitized HTML. Applies the SAME allowlist policy as
/// the server-side [`crate::sanitize_email_html`] (scripts/styles/event-handlers/
/// remote images stripped), run CLIENT-SIDE on decrypted E2EE HTML (plan §1.3).
#[wasm_bindgen(js_name = sanitizeEmailHtml)]
pub fn sanitize_email_html_wasm(html: &str) -> String {
    crate::sanitize_email_html(html)
}
