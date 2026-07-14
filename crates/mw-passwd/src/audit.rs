//! Content-free password-change audit (0008 `password_change_audit`, plan §2.7).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Which backend performed (or attempted) a change — recorded in the audit row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackendKind {
    Local,
    Ldap3062,
    DovecotHttp,
    Poppassd,
    WebhookHmac,
}

/// Success or failure of a change attempt. On failure the message is the
/// [`PasswordError`](crate::PasswordError) `Display` — which by construction contains
/// **no password material** (secrets never leave [`Secret`](crate::Secret)).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "result", content = "detail")]
pub enum AuditOutcome {
    Success,
    Failure(String),
}

/// A content-free audit event for a password-change attempt.
///
/// It carries the account id, the backend, and the outcome only — never the username's
/// credentials, never the old/new password.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub account_id: String,
    pub backend: BackendKind,
    pub outcome: AuditOutcome,
}

/// Sink for password-change audit events. `mw-server` (e9/e14) backs this with the
/// 0008 `password_change_audit` table; tests use an in-memory capture.
#[async_trait]
pub trait AuditSink: Send + Sync {
    async fn record(&self, event: &AuditEvent);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_is_content_free_and_round_trips() {
        let ev = AuditEvent {
            account_id: "a1".into(),
            backend: BackendKind::Ldap3062,
            outcome: AuditOutcome::Failure("current password rejected".into()),
        };
        let json = serde_json::to_string(&ev).unwrap();
        // Kebab-case backend tag; no secret material could appear here by construction.
        assert!(json.contains("ldap3062"));
        let back: AuditEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }
}
