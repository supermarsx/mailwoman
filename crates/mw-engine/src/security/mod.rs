//! The engine's crypto & security surface (plan §0, §1.4, §2.1/§2.2): the
//! Mailwoman-native keyring / verdict / DLP / sender-control / mail-rule method
//! families, dispatched over the same `handle_jmap` envelope the mail + PIM
//! surfaces use (result references, per-account state, cookie auth, the WS/SSE
//! push channel) but under new capability URNs `urn:mailwoman:crypto` +
//! `urn:mailwoman:security` (added to `session_json`).
//!
//! Mirrors the V3 `pim/` module pattern (plan §1.4). e0 froze the [`types`]
//! (§2.1, re-exported from `mw-crypto`) + the [`dispatch`] method arms + the
//! `SECURITY_FAMILIES` prefix list + the DLP hook call-site; e6 fills the family
//! bodies:
//! - [`verdict`] — `SecurityVerdict/get` (mail-auth DKIM/SPF/DMARC/ARC +
//!   Received-chain + signature/cert + attachment risk + anomalies).
//! - [`keyring`] — `CryptoKey/get|set|query|changes|lookup|setTrust` (own +
//!   harvested/contact keys; opaque `encryptedPrivateBackup` — never plaintext
//!   private; WKD/VKS/harvest lookup).
//! - [`dlp`] — `Dlp/getRules|scan` + the outbound `evaluate` hook at
//!   `submit_email` (config rules, detectors, redacted audit, `dlpBlocked`).
//! - [`sender_controls`] — `SenderControl/set` (block/silence/ignore/report →
//!   real `MailRule`/Sieve mechanisms, ARF).
//!
//! **Private keys are CLIENT-ONLY** (plan §1.2, risk #4): the server holds only
//! public keys/certs + opaque client-encrypted backups. Decryption + private
//! signing happen in the browser WASM worker, never here.

pub mod dispatch;
pub mod dlp;
pub mod keyring;
pub mod sender_controls;
pub mod types;
pub mod verdict;

use serde_json::{Value, json};

/// Build a standard `*/get` response envelope (`{accountId,state,list,notFound}`),
/// matching the mail + PIM surfaces byte-for-byte. Used by the e6 family fills;
/// scaffolded here so the shared shape lives with the module.
#[allow(dead_code)]
pub(crate) fn get_response(
    account_id: &str,
    state: &str,
    list: Vec<Value>,
    not_found: Vec<Value>,
) -> Value {
    json!({
        "accountId": account_id,
        "state": state,
        "list": list,
        "notFound": not_found,
    })
}

/// A JMAP method-level failure result (`serverFail`). Used by the e6 family fills.
#[allow(dead_code)]
pub(crate) fn server_fail(msg: impl std::fmt::Display) -> Value {
    json!({ "type": "serverFail", "description": msg.to_string() })
}
