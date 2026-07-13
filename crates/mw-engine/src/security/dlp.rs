//! DLP family (`Dlp/getRules`, `Dlp/scan`, frozen §2.2) + the outbound
//! [`evaluate`] hook (plan §1.8). Enforcement is inline on `EmailSubmission/set`:
//! a `block` verdict fails the submission with a `dlpBlocked` error; `warn`/
//! `require-encryption` surface pre-send via `Dlp/scan`. Rules are config/env-
//! sourced (`MW_DLP_RULES`); every evaluation writes a REDACTED `dlp_audit` row
//! (matched detector + rule, NEVER the matched content — via `mw-store` redact).
//!
//! e0 skeleton — the frozen arms with `todo!()` bodies + a NO-OP [`evaluate`]
//! wired into `submit_email` (returns no findings = allow). e6 loads the rules,
//! runs the detectors, writes the audit, and returns the `block` verdict.

use serde_json::Value;

use crate::engine::Engine;
use crate::security::types::DlpVerdict;

impl Engine {
    /// `Dlp/getRules` → `{list:[DlpRule]}` (read the active config/env rules).
    pub(crate) async fn dlp_get_rules(&self, _account_id: &str, _args: &Value) -> Value {
        todo!("e6: load MW_DLP_RULES config → [DlpRule]")
    }

    /// `Dlp/scan {draftId|{recipients,subject,bodyText,attachments}}` →
    /// `{list:[DlpVerdict]}` — the compose-time dry-run.
    pub(crate) async fn dlp_scan(&self, _account_id: &str, _args: &Value) -> Value {
        todo!("e6: run detectors over the draft → [DlpVerdict] (redacted)")
    }
}

/// The outbound DLP evaluation hook (plan §1.8), called at the `submit_email`
/// chokepoint BEFORE `submitter.submit`. Returns the matched verdicts; a caller
/// treats any `verdict.blocked == true` as a hard block (fails
/// `EmailSubmission/set` with `dlpBlocked`).
///
/// e0 wires this as a NO-OP: no rules are loaded, so it returns no findings
/// (= allow) and the send path is byte-for-byte unchanged. e6 loads the config
/// rules, runs the detectors, writes the redacted `dlp_audit` row, and returns
/// a `block` verdict when a rule with `action:"block"` matches.
pub(crate) async fn evaluate(
    _engine: &Engine,
    _account_id: &str,
    _email_id: &str,
) -> Vec<DlpVerdict> {
    // No-op seam (e6 fills). No rules loaded → no findings → allow.
    Vec::new()
}
