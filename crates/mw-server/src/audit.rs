//! Security events in the audit log (t32-ext §14.3 audit v2, §15 M4; e57 seam).
//!
//! [`security_event`] is the one call for a security-relevant event: sign-in
//! outcomes, step-up proofs, 2FA and session changes, delegations, forwarding
//! rules, API keys, exports. The row goes to `audit_log` with
//! `category = "security"`, the request's `X-Request-Id` when it has one, and a
//! detail object scrubbed by [`mw_admin::redact_detail`] (no secret, no address,
//! no mail content). A failed write is logged and never fails the caller: the
//! event already happened.
//!
//! The audit chain (e58) takes over the append path behind this function; callers
//! do not change.

use std::net::IpAddr;

use axum::http::HeaderMap;
use mw_admin::ActorKind;
use serde_json::Value;

use crate::AppState;

/// `audit_log.category` of a security event.
pub(crate) const CATEGORY_SECURITY: &str = "security";

/// One security event, before redaction.
#[derive(Debug, Clone)]
pub(crate) struct SecurityEvent {
    /// Account id, admin username, or `system`.
    pub actor: String,
    pub actor_kind: ActorKind,
    /// Kebab-case action (`step-up-granted`).
    pub action: &'static str,
    /// `kind:id` of what the event is about (`account:<id>`).
    pub target: Option<String>,
    pub detail: Value,
    pub ip: Option<IpAddr>,
    pub request_id: Option<String>,
    pub category: &'static str,
}

impl SecurityEvent {
    fn new(actor: impl Into<String>, actor_kind: ActorKind, action: &'static str) -> Self {
        Self {
            actor: actor.into(),
            actor_kind,
            action,
            target: None,
            detail: Value::Object(Default::default()),
            ip: None,
            request_id: None,
            category: CATEGORY_SECURITY,
        }
    }

    /// An event a mailbox user caused; the target defaults to their account.
    pub fn user(account_id: &str, action: &'static str) -> Self {
        Self::new(account_id, ActorKind::User, action).target(format!("account:{account_id}"))
    }

    /// An event an administrator caused.
    pub fn admin(username: &str, action: &'static str) -> Self {
        Self::new(username, ActorKind::Admin, action)
    }

    /// An event the server raised on its own.
    pub fn system(action: &'static str) -> Self {
        Self::new("system", ActorKind::System, action)
    }

    #[must_use]
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    #[must_use]
    pub fn detail(mut self, detail: Value) -> Self {
        self.detail = detail;
        self
    }

    #[must_use]
    pub fn ip(mut self, ip: Option<IpAddr>) -> Self {
        self.ip = ip;
        self
    }

    /// Take the request id from the request's `X-Request-Id`.
    #[must_use]
    pub fn request(mut self, headers: &HeaderMap) -> Self {
        self.request_id = headers
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|v| !v.is_empty() && v.len() <= 128)
            .map(str::to_string);
        self
    }

    #[must_use]
    pub fn category(mut self, category: &'static str) -> Self {
        self.category = category;
        self
    }
}

fn actor_kind_str(kind: ActorKind) -> &'static str {
    match kind {
        ActorKind::Admin => "admin",
        ActorKind::User => "user",
        ActorKind::ApiKey => "api-key",
        ActorKind::System => "system",
    }
}

/// Append `ev` to the audit log. Returns the row id, or `None` when the write
/// failed (logged).
pub(crate) async fn security_event(state: &AppState, ev: SecurityEvent) -> Option<String> {
    let row = mw_store::AuditRow {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: ev.actor,
        actor_kind: actor_kind_str(ev.actor_kind).to_string(),
        action: ev.action.to_string(),
        target: ev.target,
        detail_json: mw_admin::redact_detail(&ev.detail),
        ip: ev.ip.map(|ip| ip.to_string()),
    };
    match state
        .store
        .append_categorized_audit(&row, ev.category, ev.request_id.as_deref())
        .await
    {
        Ok(()) => Some(row.id),
        Err(e) => {
            tracing::warn!("security event {} not recorded: {e}", row.action);
            None
        }
    }
}
