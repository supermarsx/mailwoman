//! Sender-control family (`SenderControl/set`, frozen §2.2). Wires to REAL
//! mechanisms, NOT localStorage (plan §1.9 / risk #14): block → a real Sieve
//! `MailRule` (From is → Move Junk|Stop); silence → a per-sender notify-suppress
//! flag; ignore-conversation → an auto-archive rule scoped to a `threadId`;
//! report-phishing/junk → ARF emit via the account submitter + spam-trainer.
//!
//! e0 skeleton — the frozen arm with a `todo!()` body. e6 fills the mechanisms.

use serde_json::Value;

use crate::account::AccountRuntime;
use crate::engine::Engine;

impl Engine {
    /// `SenderControl/set {emailId|address|threadId, action, abuseReport?}` →
    /// `{updated, mailRuleId?}` — applies the §1.9 real mechanism.
    pub(crate) async fn sender_control_set(
        &self,
        _account_id: &str,
        _rt: &AccountRuntime,
        _args: &Value,
    ) -> Value {
        todo!("e6: block→Sieve MailRule, silence→notify-suppress, ignore→archive, report→ARF")
    }
}
