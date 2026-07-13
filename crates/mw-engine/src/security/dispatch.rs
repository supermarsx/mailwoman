//! Crypto/security method dispatch (frozen §2.2). Every family rides the existing
//! JMAP envelope; [`Engine::dispatch_security`] is reached from `handle_jmap`'s
//! dispatch for any `CryptoKey/*`, `SecurityVerdict/*`, `SenderControl/*`,
//! `MailRule/*`, or `Dlp/*` method (mirrors the V3 `pim/dispatch.rs`).
//!
//! The set of method names here IS the §2.2 contract the web client and the
//! parallel builders compile against — do not add/rename without a coordinator
//! re-broadcast. e0 froze the arms (bodies `todo!()`); e6 fills the handlers.

use serde_json::{Value, json};

use crate::account::AccountRuntime;
use crate::change::ChangeType;
use crate::engine::Engine;

/// The Mailwoman crypto/security method families (frozen §2.2). A method name
/// whose family prefix is one of these routes to [`Engine::dispatch_security`].
pub const SECURITY_FAMILIES: &[&str] = &[
    "CryptoKey/",
    "SecurityVerdict/",
    "SenderControl/",
    "MailRule/",
    "Dlp/",
];

/// Whether `method` belongs to a Mailwoman crypto/security family (§2.2).
pub fn is_security_method(method: &str) -> bool {
    SECURITY_FAMILIES.iter().any(|fam| method.starts_with(fam))
}

impl Engine {
    /// Dispatch one resolved crypto/security method call (frozen §2.2). Reached
    /// from `handle_jmap` for any security-family method. `rt` is the connected
    /// account's runtime (submitter for ARF, backend for cert harvest).
    pub(crate) async fn dispatch_security(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        name: &str,
        args: &Value,
    ) -> Value {
        match name {
            // ── Keyring (§2.2 — public keys + opaque backup only) ──
            "CryptoKey/get" => self.crypto_key_get(account_id, args).await,
            "CryptoKey/set" => self.crypto_key_set(account_id, args).await,
            "CryptoKey/query" => self.crypto_key_query(account_id, args).await,
            "CryptoKey/changes" => {
                self.security_type_changes(account_id, ChangeType::CryptoKey, args)
                    .await
            }
            "CryptoKey/lookup" => self.crypto_key_lookup(account_id, rt, args).await,
            "CryptoKey/setTrust" => self.crypto_key_set_trust(account_id, args).await,
            // ── Verdicts (§2.2 — server-side, all public) ──
            "SecurityVerdict/get" => self.security_verdict_get(account_id, rt, args).await,
            // ── Sender controls (§2.2) ──
            "SenderControl/set" => self.sender_control_set(account_id, rt, args).await,
            // ── Mail rules over rules.rs (§2.2) ──
            "MailRule/get" => self.mail_rule_get(account_id, args).await,
            "MailRule/set" => self.mail_rule_set(account_id, rt, args).await,
            "MailRule/changes" => {
                self.security_type_changes(account_id, ChangeType::MailRule, args)
                    .await
            }
            // ── DLP (§2.2 — enforcement is inline on EmailSubmission/set) ──
            "Dlp/getRules" => self.dlp_get_rules(account_id, args).await,
            "Dlp/scan" => self.dlp_scan(account_id, args).await,
            other => json!({
                "type": "unknownMethod",
                "description": format!("engine does not implement security method {other}")
            }),
        }
    }

    /// The generic crypto/security `*/changes` handler (frozen §2.1) for
    /// `CryptoKey`/`MailRule`, sourced from the `crypto_changes` log (e6). e0
    /// returns an empty diff envelope so a polling client never panics; e6 wires
    /// it to the real state tokens.
    pub(crate) async fn security_type_changes(
        &self,
        account_id: &str,
        _kind: ChangeType,
        args: &Value,
    ) -> Value {
        let since = args
            .get("sinceState")
            .and_then(Value::as_str)
            .unwrap_or("0");
        json!({
            "accountId": account_id,
            "oldState": since,
            "newState": since,
            "created": [],
            "updated": [],
            "destroyed": [],
            "hasMoreChanges": false,
        })
    }
}
