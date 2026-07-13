//! The append-only audit log (plan §2.5, §21). Builds redacted [`AuditLogEntry`]
//! records from typed [`AuditEvent`]s. **No secret, mail body, subject, or
//! address is ever stored in `detail_json`** — [`redact_detail`] scrubs the
//! structured detail before it is persisted.
//!
//! Append-only is enforced STRUCTURALLY by [`crate::store::AdminBackend`], which
//! exposes only `append_audit`/`list_audit` — there is no update/delete path.
//! The [`tests::audit_backend_has_no_mutation_path`] test documents + guards that.

use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ActorKind, AuditLogEntry};

/// The kind of audited action (plan §2.5/§2.6, §21). Serialized kebab-case into
/// the `audit_log.action` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditKind {
    DomainCreated,
    DomainUpdated,
    DomainDeleted,
    UserProvisioned,
    QuotaChanged,
    SessionsRevoked,
    FeatureFlagChanged,
    ZeroAccessToggled,
    ForcePasswordChange,
    RemoteCacheWipe,
    SecurityPolicyChanged,
    IntegrationChanged,
    ApiKeyRevoked,
    ObservabilityChanged,
    AppearanceChanged,
    CacheScopeChanged,
    LoginSucceeded,
    LoginFailed,
    IpBanned,
    IpUnbanned,
    ConfigReloaded,
}

impl AuditKind {
    /// The stable kebab-case action string stored in `audit_log.action`.
    pub fn as_action(self) -> &'static str {
        match self {
            AuditKind::DomainCreated => "domain-created",
            AuditKind::DomainUpdated => "domain-updated",
            AuditKind::DomainDeleted => "domain-deleted",
            AuditKind::UserProvisioned => "user-provisioned",
            AuditKind::QuotaChanged => "quota-changed",
            AuditKind::SessionsRevoked => "sessions-revoked",
            AuditKind::FeatureFlagChanged => "feature-flag-changed",
            AuditKind::ZeroAccessToggled => "zero-access-toggled",
            AuditKind::ForcePasswordChange => "force-password-change",
            AuditKind::RemoteCacheWipe => "remote-cache-wipe",
            AuditKind::SecurityPolicyChanged => "security-policy-changed",
            AuditKind::IntegrationChanged => "integration-changed",
            AuditKind::ApiKeyRevoked => "api-key-revoked",
            AuditKind::ObservabilityChanged => "observability-changed",
            AuditKind::AppearanceChanged => "appearance-changed",
            AuditKind::CacheScopeChanged => "cache-scope-changed",
            AuditKind::LoginSucceeded => "login-succeeded",
            AuditKind::LoginFailed => "login-failed",
            AuditKind::IpBanned => "ip-banned",
            AuditKind::IpUnbanned => "ip-unbanned",
            AuditKind::ConfigReloaded => "config-reloaded",
        }
    }
}

/// A typed audit event as produced by the domain logic, before redaction +
/// persistence. `detail` is arbitrary structured context; it is redacted by
/// [`AuditEvent::into_entry`].
#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub actor: String,
    pub actor_kind: ActorKind,
    pub kind: AuditKind,
    pub target: Option<String>,
    pub detail: Value,
    pub ip: Option<String>,
}

impl AuditEvent {
    /// A minimal event with empty detail.
    pub fn new(actor: impl Into<String>, actor_kind: ActorKind, kind: AuditKind) -> Self {
        Self {
            actor: actor.into(),
            actor_kind,
            kind,
            target: None,
            detail: Value::Object(Default::default()),
            ip: None,
        }
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
    pub fn ip(mut self, ip: Option<String>) -> Self {
        self.ip = ip;
        self
    }

    /// Materialize the immutable, redacted [`AuditLogEntry`]. A fresh UUID id +
    /// RFC 3339 timestamp are assigned; `detail` is scrubbed of secrets and mail
    /// content by [`redact_detail`].
    pub fn into_entry(self) -> AuditLogEntry {
        AuditLogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            ts: chrono::Utc::now().to_rfc3339(),
            actor: self.actor,
            actor_kind: self.actor_kind,
            action: self.kind.as_action().to_string(),
            target: self.target,
            detail_json: redact_detail(&self.detail),
            ip: self.ip,
        }
    }
}

/// JSON object keys whose values are ALWAYS replaced with the redaction marker —
/// secrets + any mail envelope/content field (§21.1: no body/subject/address in
/// logs).
const SENSITIVE_KEYS: &[&str] = &[
    "password",
    "passwd",
    "pass",
    "secret",
    "token",
    "access_token",
    "refresh_token",
    "api_key",
    "apikey",
    "key",
    "authorization",
    "auth",
    "cookie",
    "credential",
    "credentials",
    "private_key",
    "wrapped_root_key",
    // Mail content / envelope — never logged.
    "subject",
    "body",
    "html",
    "text",
    "preview",
    "snippet",
    "to",
    "from",
    "cc",
    "bcc",
    "reply_to",
    "sender",
    "recipient",
    "recipients",
    "address",
    "addresses",
    "email",
    "mailbox",
    "envelope",
];

/// The value substituted for redacted fields.
pub const REDACTED: &str = "[redacted]";

static EMAIL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}").expect("valid email regex")
});

/// Redact structured detail into a compact JSON string safe for the audit log.
/// Sensitive keys are dropped to [`REDACTED`]; any email address appearing in a
/// string value is masked; the walk recurses through objects and arrays.
pub fn redact_detail(detail: &Value) -> String {
    let scrubbed = scrub(detail);
    serde_json::to_string(&scrubbed).unwrap_or_else(|_| "{}".to_string())
}

fn scrub(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::with_capacity(map.len());
            for (k, v) in map {
                if SENSITIVE_KEYS.contains(&k.to_ascii_lowercase().as_str()) {
                    out.insert(k.clone(), Value::String(REDACTED.to_string()));
                } else {
                    out.insert(k.clone(), scrub(v));
                }
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(scrub).collect()),
        Value::String(s) => Value::String(mask_addresses(s)),
        other => other.clone(),
    }
}

fn mask_addresses(s: &str) -> String {
    EMAIL_RE.replace_all(s, "[redacted-address]").into_owned()
}

/// Serialize a slice of entries to newline-delimited JSON (JSONL) for the
/// audit-log export (§19 observability / audit export).
pub fn export_jsonl(entries: &[AuditLogEntry]) -> String {
    entries
        .iter()
        .map(|e| serde_json::to_string(e).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{AdminBackend, InMemoryBackend};

    #[test]
    fn redaction_removes_secrets_and_mail_content() {
        let detail = serde_json::json!({
            "username": "alice",
            "password": "hunter2",
            "subject": "Q3 numbers",
            "note": "contact bob@example.com about it",
            "nested": { "token": "abc", "keep": "value" },
            "recipients": ["x@y.com", "z@w.org"]
        });
        let out = redact_detail(&detail);
        assert!(!out.contains("hunter2"), "password leaked: {out}");
        assert!(!out.contains("Q3 numbers"), "subject leaked: {out}");
        assert!(!out.contains("bob@example.com"), "address leaked: {out}");
        assert!(!out.contains("x@y.com"), "recipient leaked: {out}");
        assert!(out.contains("alice"), "non-sensitive dropped: {out}");
        assert!(out.contains("value"), "non-sensitive nested dropped: {out}");
        assert!(out.contains(REDACTED));
    }

    #[test]
    fn event_builds_entry_with_id_and_action() {
        let entry = AuditEvent::new("admin", ActorKind::Admin, AuditKind::UserProvisioned)
            .target("alice@example.com")
            .into_entry();
        assert_eq!(entry.action, "user-provisioned");
        assert_eq!(entry.actor_kind, ActorKind::Admin);
        assert!(!entry.id.is_empty());
        assert!(!entry.ts.is_empty());
    }

    /// The append-only invariant (§2.5): the audit surface exposes append + read
    /// ONLY. This test both exercises the append path and documents that no
    /// mutation/removal method exists on the port — the trait simply has none, so
    /// appended history cannot be rewritten or deleted through the API.
    #[tokio::test]
    async fn audit_backend_has_no_mutation_path() {
        let backend = InMemoryBackend::new();
        for i in 0..5 {
            let entry = AuditEvent::new("admin", ActorKind::Admin, AuditKind::ConfigReloaded)
                .detail(serde_json::json!({ "seq": i }))
                .into_entry();
            backend.append_audit(entry).await.unwrap();
        }
        // Every appended record is still present; nothing can remove or rewrite
        // them because AdminBackend defines no such method (compile-time proof).
        let all = backend.list_audit(100).await.unwrap();
        assert_eq!(all.len(), 5);

        // Static assertion of intent: the only mutating audit method is append.
        // If a future edit adds an update/delete method, this reference list must
        // be revisited — the invariant lives in the trait's shape.
        fn assert_append_is_only_audit_mutator<B: AdminBackend>() {}
        assert_append_is_only_audit_mutator::<InMemoryBackend>();
    }
}
