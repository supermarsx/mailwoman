#![forbid(unsafe_code)]
// SCAFFOLD (t6-e0): stub crate — the frozen §2.5 admin contract names exist so
// e7 (web) and e11 (mount) compile against them; e5 owns the real implementation.
#![allow(dead_code, clippy::unused_async)]
//! Admin-panel domain logic for Mailwoman V6 (SPEC §19, plan §2.5).
//!
//! **Frozen contract (§2.5):** the `/admin/*` surface runs under a SEPARATE
//! session domain (`mw_admin_session` cookie or a separate port; passkey-capable)
//! and mirrors §19: [`Domain`]s, users (provision/quota/session-revoke/feature-
//! flags incl. the zero-access toggle/force-change/remote-cache-wipe), the
//! [`SecurityPolicy`], integrations (webhooks + MCP/API-key oversight; LDAP/
//! Nextcloud entries INERT/deferred), observability (log-level/OTLP DSN/audit-
//! viewer+export/login-monitor+ban-list), and appearance. Every endpoint has a
//! `mailwoman admin <noun> <verb>` CLI equivalent + TOML/env binding;
//! `admin.enabled=false` unmounts the panel. **All admin actions write the
//! append-only [`AuditLogEntry`].**
//!
//! e5 fills the bodies (currently `unimplemented!()`), persists via the `mw-store`
//! 0007 tables (`admin_users`/`admin_sessions`/`domains`/`quotas`/`audit_log`/
//! `cache_scope`), and authors the clap subcommand tree. e0 leaves the
//! `mailwoman admin` clap STUB in `mw-server`'s `main.rs`.

use serde::{Deserialize, Serialize};

/// A managed mail domain (`domains` table, 0007).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub name: String,
    /// Upstream routing config (JSON blob per §19).
    pub upstream_json: String,
    pub allowlist: Vec<String>,
    pub blocklist: Vec<String>,
}

/// A per-account quota (`quotas` table, 0007).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Quota {
    pub bytes_limit: i64,
    pub msg_limit: i64,
}

/// The security-policy model (§19 security-policy section): min-TLS, 2FA
/// requirement, Argon2 params, DLP rules, the max-security floor, capture policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub min_tls: String,
    pub require_2fa: bool,
    pub argon2_m_cost: u32,
    pub argon2_t_cost: u32,
    pub argon2_p_cost: u32,
    pub dlp_rules_json: String,
    pub max_security_floor: bool,
    pub capture_policy: String,
}

/// Who/what performed an audited action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActorKind {
    Admin,
    User,
    ApiKey,
    System,
}

/// An append-only audit-log record (`audit_log` table, 0007). There is NO update
/// or delete path — the writer only appends (§2.5 invariant, asserted by e5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    /// RFC 3339 timestamp.
    pub ts: String,
    pub actor: String,
    pub actor_kind: ActorKind,
    pub action: String,
    pub target: Option<String>,
    /// Structured detail (JSON).
    pub detail_json: String,
    pub ip: Option<String>,
}

/// A banned source (login-monitor / ban-list, fail2ban-compatible). e5 emits the
/// fail2ban log line format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanEntry {
    pub ip: String,
    pub reason: String,
    pub banned_at: String,
    pub expires_at: Option<String>,
}

/// The admin-configurable cache scope matrix row (`cache_scope` table, 0007;
/// mirrors `mw-cache`'s per-class policy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheScopeRow {
    pub class: String,
    /// Layers (JSON array of `memory|redis|store`).
    pub layers_json: String,
    pub ttl_secs: i64,
}

/// Observability configuration (§19 observability section).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub log_level: String,
    pub otlp_dsn: Option<String>,
    /// Whether the auth-gated Prometheus `/metrics` endpoint is enabled.
    pub metrics_enabled: bool,
    /// Sentry DSN — VET-BEFORE-ENABLE, off by default (plan §5, R6).
    pub sentry_dsn: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AdminError {
    #[error("not found")]
    NotFound,
    #[error("quota exceeded")]
    QuotaExceeded,
    #[error("admin panel disabled")]
    Disabled,
    #[error("store error: {0}")]
    Store(String),
    #[error("config error: {0}")]
    Config(String),
}

/// The admin domain-logic façade. STUB: e5 replaces this with the real handle
/// (a `mw-store` handle + config) and fills the provisioning/audit/ban methods.
#[derive(Clone, Default)]
pub struct Admin {
    _private: (),
}

impl Admin {
    /// Append an audit record. The ONLY audit-log mutation path (append-only).
    pub async fn audit(&self, _entry: AuditLogEntry) -> Result<(), AdminError> {
        unimplemented!("mw-admin::Admin::audit — filled by t6-e5")
    }

    /// Provision (or update) a user under `domain` with `quota`.
    pub async fn provision_user(
        &self,
        _domain: &str,
        _username: &str,
        _quota: Quota,
    ) -> Result<(), AdminError> {
        unimplemented!("mw-admin::Admin::provision_user — filled by t6-e5")
    }
}
