#![forbid(unsafe_code)]
//! Admin-panel domain logic for Mailwoman V6 (SPEC §19, plan §2.5).
//!
//! **Frozen contract (§2.5):** the `/admin/*` surface runs under a SEPARATE
//! session domain (`mw_admin_session` cookie or a separate port; passkey-capable)
//! and mirrors §19: [`Domain`]s, users (provision/quota/session-revoke/feature-
//! flags incl. the zero-access toggle/force-change/remote-cache-wipe), the
//! [`SecurityPolicy`], integrations (webhooks + MCP/API-key oversight; LDAP/
//! Nextcloud entries INERT/deferred), observability (log-level/OTLP DSN/audit-
//! viewer+export/login-monitor+ban-list), and appearance. Every endpoint has a
//! `mailwoman admin <noun> <verb>` CLI equivalent (see [`cli`]) + TOML/env
//! binding (see [`config`]); `admin.enabled = false` unmounts the panel. **All
//! admin actions write the append-only [`AuditLogEntry`].**
//!
//! **Persistence** is abstracted behind [`store::AdminBackend`] so the domain
//! logic is testable in isolation and does not hard-couple to the in-flight
//! `mw-store` 0007 tables; e11 supplies a `mw-store`-backed adapter at mount
//! time. [`store::InMemoryBackend`] backs `Admin::default()` and the tests.
//!
//! **Append-only audit invariant:** the audit surface exposes only
//! append + read — there is no update/delete path (see [`store`] + the
//! `audit::tests::audit_backend_has_no_mutation_path` test).

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

pub mod audit;
pub mod banlist;
pub mod cli;
pub mod config;
pub mod password;
pub mod provisioning;
pub mod store;

pub use audit::{AuditEvent, AuditKind, export_jsonl, redact_detail};
pub use banlist::{FAIL2BAN_FAILREGEX, LoginMonitor, LoginVerdict, fail2ban_line};
pub use config::{AdminConfig, Appearance};
pub use provisioning::{IntegrationStatus, IntegrationsConfig, UserFeatureFlags};
pub use store::{AdminBackend, InMemoryBackend, UserRecord};

/// A managed mail domain (`domains` table, 0007).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Domain {
    pub name: String,
    /// Upstream routing config (JSON blob per §19).
    pub upstream_json: String,
    pub allowlist: Vec<String>,
    pub blocklist: Vec<String>,
}

/// A per-account quota (`quotas` table, 0007). A non-positive limit means "no
/// limit" (see [`Quota::allows`] in `provisioning`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quota {
    pub bytes_limit: i64,
    pub msg_limit: i64,
}

/// The security-policy model (§19 security-policy section): min-TLS, 2FA
/// requirement, Argon2 params, DLP rules, the max-security floor, capture policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
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

impl Default for SecurityPolicy {
    /// OWASP-recommended Argon2id parameters (m = 19 MiB, t = 2, p = 1) and a
    /// TLS 1.2 floor; 2FA optional; DLP empty; capture off.
    fn default() -> Self {
        Self {
            min_tls: "1.2".to_string(),
            require_2fa: false,
            argon2_m_cost: 19_456,
            argon2_t_cost: 2,
            argon2_p_cost: 1,
            dlp_rules_json: "[]".to_string(),
            max_security_floor: false,
            capture_policy: "off".to_string(),
        }
    }
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
/// or delete path — the writer only appends (§2.5 invariant, asserted in
/// `audit::tests`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    /// RFC 3339 timestamp.
    pub ts: String,
    pub actor: String,
    pub actor_kind: ActorKind,
    pub action: String,
    pub target: Option<String>,
    /// Structured detail (JSON), REDACTED of secrets + mail content (§21.1).
    pub detail_json: String,
    pub ip: Option<String>,
}

/// A banned source (login-monitor / ban-list, fail2ban-compatible; see
/// [`banlist`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BanEntry {
    pub ip: String,
    pub reason: String,
    pub banned_at: String,
    pub expires_at: Option<String>,
}

/// The admin-configurable cache scope matrix row (`cache_scope` table, 0007;
/// mirrors `mw-cache`'s per-class policy).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheScopeRow {
    pub class: String,
    /// Layers (JSON array of `memory|redis|store`).
    pub layers_json: String,
    pub ttl_secs: i64,
}

/// Observability configuration (§19 observability section).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ObservabilityConfig {
    pub log_level: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otlp_dsn: Option<String>,
    /// Whether the auth-gated Prometheus `/metrics` endpoint is enabled.
    pub metrics_enabled: bool,
    /// Sentry DSN — VET-BEFORE-ENABLE, off by default (plan §5, R6).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentry_dsn: Option<String>,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            otlp_dsn: None,
            metrics_enabled: false,
            sentry_dsn: None,
        }
    }
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

/// The result of recording a login failure through [`Admin::record_login_failure`].
#[derive(Debug, Clone)]
pub struct LoginFailureOutcome {
    /// The monitor's verdict (watched vs ban).
    pub verdict: LoginVerdict,
    /// The fail2ban-compatible log line an operator's jail can parse.
    pub log_line: String,
    /// Whether this failure crossed the threshold and auto-added a ban.
    pub banned: bool,
}

/// The admin domain-logic façade. Holds a persistence [`AdminBackend`], the
/// live [`AdminConfig`], and the in-process [`LoginMonitor`]. Every mutating
/// method writes the append-only audit log.
#[derive(Clone)]
pub struct Admin {
    backend: Arc<dyn AdminBackend>,
    config: Arc<Mutex<AdminConfig>>,
    monitor: Arc<LoginMonitor>,
}

impl Default for Admin {
    fn default() -> Self {
        Self::in_memory()
    }
}

impl Admin {
    /// Construct with a concrete backend + config (e11 wires the `mw-store`
    /// adapter here).
    pub fn new(backend: Arc<dyn AdminBackend>, config: AdminConfig) -> Self {
        Self {
            backend,
            config: Arc::new(Mutex::new(config)),
            monitor: Arc::new(LoginMonitor::with_defaults()),
        }
    }

    /// An in-memory admin (default config) — used by `Default`, tests, and
    /// harnesses.
    pub fn in_memory() -> Self {
        Self::new(Arc::new(InMemoryBackend::new()), AdminConfig::default())
    }

    /// Whether the panel is enabled (`admin.enabled`). e11 unmounts the routes
    /// when this is `false`; the domain logic + CLI keep working (GitOps).
    pub fn panel_enabled(&self) -> bool {
        self.config.lock().expect("config poisoned").enabled
    }

    /// A snapshot of the current config.
    pub fn config(&self) -> AdminConfig {
        self.config.lock().expect("config poisoned").clone()
    }

    /// Access the underlying backend (e.g. for e11's adapter-specific plumbing).
    pub fn backend(&self) -> &Arc<dyn AdminBackend> {
        &self.backend
    }

    // ── Audit ────────────────────────────────────────────────────────────────

    /// Append an audit record. The ONLY audit-log mutation path (append-only).
    pub async fn audit(&self, entry: AuditLogEntry) -> Result<(), AdminError> {
        self.backend.append_audit(entry).await
    }

    /// Build (redact) + append an [`AuditEvent`].
    pub async fn record(&self, event: AuditEvent) -> Result<(), AdminError> {
        self.audit(event.into_entry()).await
    }

    /// Read the most recent `limit` audit records (newest first).
    pub async fn list_audit(&self, limit: usize) -> Result<Vec<AuditLogEntry>, AdminError> {
        self.backend.list_audit(limit).await
    }

    /// Export the most recent `limit` audit records as JSONL (§19 audit export).
    pub async fn export_audit(&self, limit: usize) -> Result<String, AdminError> {
        Ok(export_jsonl(&self.list_audit(limit).await?))
    }

    async fn emit(
        &self,
        actor: &str,
        actor_kind: ActorKind,
        kind: AuditKind,
        target: Option<String>,
        detail: serde_json::Value,
    ) -> Result<(), AdminError> {
        self.record(
            AuditEvent::new(actor, actor_kind, kind)
                .detail(detail)
                .ip(None)
                .target_opt(target),
        )
        .await
    }

    // ── Domains ──────────────────────────────────────────────────────────────

    pub async fn create_domain(&self, actor: &str, domain: Domain) -> Result<(), AdminError> {
        let name = domain.name.clone();
        self.backend.upsert_domain(domain).await?;
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::DomainCreated,
            Some(name),
            serde_json::json!({}),
        )
        .await
    }

    pub async fn list_domains(&self) -> Result<Vec<Domain>, AdminError> {
        self.backend.list_domains().await
    }

    pub async fn get_domain(&self, name: &str) -> Result<Option<Domain>, AdminError> {
        self.backend.get_domain(name).await
    }

    pub async fn delete_domain(&self, actor: &str, name: &str) -> Result<(), AdminError> {
        self.backend.delete_domain(name).await?;
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::DomainDeleted,
            Some(name.to_string()),
            serde_json::json!({}),
        )
        .await
    }

    // ── Users / quotas / feature flags ───────────────────────────────────────

    /// Provision (or update) a user under `domain` with `quota`.
    pub async fn provision_user(
        &self,
        actor: &str,
        domain: &str,
        username: &str,
        quota: Quota,
    ) -> Result<(), AdminError> {
        let account_id = account_id(username, domain);
        self.backend
            .upsert_user(UserRecord {
                username: username.to_string(),
                domain: domain.to_string(),
                password_hash: None,
            })
            .await?;
        self.backend.set_quota(&account_id, quota).await?;
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::UserProvisioned,
            Some(account_id),
            serde_json::json!({ "bytes_limit": quota.bytes_limit, "msg_limit": quota.msg_limit }),
        )
        .await
    }

    pub async fn set_quota(
        &self,
        actor: &str,
        account_id: &str,
        quota: Quota,
    ) -> Result<(), AdminError> {
        self.backend.set_quota(account_id, quota).await?;
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::QuotaChanged,
            Some(account_id.to_string()),
            serde_json::json!({ "bytes_limit": quota.bytes_limit, "msg_limit": quota.msg_limit }),
        )
        .await
    }

    pub async fn get_quota(&self, account_id: &str) -> Result<Option<Quota>, AdminError> {
        self.backend.get_quota(account_id).await
    }

    /// Enforce the account's quota against a reported usage.
    pub async fn check_quota(
        &self,
        account_id: &str,
        bytes_used: i64,
        msg_used: i64,
    ) -> Result<(), AdminError> {
        match self.backend.get_quota(account_id).await? {
            Some(q) => q.enforce(bytes_used, msg_used),
            None => Ok(()),
        }
    }

    pub async fn get_feature_flags(
        &self,
        account_id: &str,
    ) -> Result<UserFeatureFlags, AdminError> {
        self.backend.get_flags(account_id).await
    }

    pub async fn set_feature_flags(
        &self,
        actor: &str,
        account_id: &str,
        flags: UserFeatureFlags,
    ) -> Result<(), AdminError> {
        self.backend.set_flags(account_id, flags).await?;
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::FeatureFlagChanged,
            Some(account_id.to_string()),
            serde_json::json!({
                "zero_access": flags.zero_access,
                "force_password_change": flags.force_password_change,
                "remote_cache_wipe": flags.remote_cache_wipe,
                "disabled": flags.disabled,
            }),
        )
        .await
    }

    /// Toggle the per-account zero-access storage flag (§9).
    pub async fn toggle_zero_access(
        &self,
        actor: &str,
        account_id: &str,
        on: bool,
    ) -> Result<(), AdminError> {
        let mut flags = self.backend.get_flags(account_id).await?;
        flags.zero_access = on;
        self.backend.set_flags(account_id, flags).await?;
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::ZeroAccessToggled,
            Some(account_id.to_string()),
            serde_json::json!({ "enabled": on }),
        )
        .await
    }

    /// Set/clear the force-password-change flag.
    pub async fn force_password_change(
        &self,
        actor: &str,
        account_id: &str,
        on: bool,
    ) -> Result<(), AdminError> {
        let mut flags = self.backend.get_flags(account_id).await?;
        flags.force_password_change = on;
        self.backend.set_flags(account_id, flags).await?;
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::ForcePasswordChange,
            Some(account_id.to_string()),
            serde_json::json!({ "enabled": on }),
        )
        .await
    }

    /// Request a one-shot remote cache wipe for the account (§19 users). The
    /// engine clears the flag once honored.
    pub async fn request_remote_cache_wipe(
        &self,
        actor: &str,
        account_id: &str,
    ) -> Result<(), AdminError> {
        let mut flags = self.backend.get_flags(account_id).await?;
        flags.remote_cache_wipe = true;
        self.backend.set_flags(account_id, flags).await?;
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::RemoteCacheWipe,
            Some(account_id.to_string()),
            serde_json::json!({}),
        )
        .await
    }

    /// Revoke all of an account's sessions; returns the number revoked.
    pub async fn revoke_sessions(&self, actor: &str, account_id: &str) -> Result<u64, AdminError> {
        let n = self.backend.revoke_sessions(account_id).await?;
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::SessionsRevoked,
            Some(account_id.to_string()),
            serde_json::json!({ "count": n }),
        )
        .await?;
        Ok(n)
    }

    // ── Security policy ──────────────────────────────────────────────────────

    pub async fn get_security_policy(&self) -> Result<SecurityPolicy, AdminError> {
        Ok(self
            .backend
            .get_security_policy()
            .await?
            .unwrap_or_else(|| self.config().security))
    }

    pub async fn set_security_policy(
        &self,
        actor: &str,
        policy: SecurityPolicy,
    ) -> Result<(), AdminError> {
        self.backend.set_security_policy(policy.clone()).await?;
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::SecurityPolicyChanged,
            None,
            serde_json::json!({
                "min_tls": policy.min_tls,
                "require_2fa": policy.require_2fa,
                "max_security_floor": policy.max_security_floor,
                "capture_policy": policy.capture_policy,
            }),
        )
        .await
    }

    // ── Observability ────────────────────────────────────────────────────────

    pub async fn get_observability(&self) -> Result<ObservabilityConfig, AdminError> {
        Ok(self
            .backend
            .get_observability()
            .await?
            .unwrap_or_else(|| self.config().observability))
    }

    pub async fn set_observability(
        &self,
        actor: &str,
        cfg: ObservabilityConfig,
    ) -> Result<(), AdminError> {
        self.backend.set_observability(cfg.clone()).await?;
        // detail carries no DSN secret material — DSNs are redacted-by-omission.
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::ObservabilityChanged,
            None,
            serde_json::json!({
                "log_level": cfg.log_level,
                "metrics_enabled": cfg.metrics_enabled,
                "otlp_configured": cfg.otlp_dsn.is_some(),
                "sentry_configured": cfg.sentry_dsn.is_some(),
            }),
        )
        .await
    }

    // ── Cache scope matrix ───────────────────────────────────────────────────

    pub async fn set_cache_scope(&self, actor: &str, row: CacheScopeRow) -> Result<(), AdminError> {
        let class = row.class.clone();
        self.backend.upsert_cache_scope(row).await?;
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::CacheScopeChanged,
            Some(class),
            serde_json::json!({}),
        )
        .await
    }

    pub async fn list_cache_scope(&self) -> Result<Vec<CacheScopeRow>, AdminError> {
        self.backend.list_cache_scope().await
    }

    // ── Appearance / enabled ─────────────────────────────────────────────────

    pub async fn set_appearance(
        &self,
        actor: &str,
        appearance: Appearance,
    ) -> Result<(), AdminError> {
        self.config.lock().expect("config poisoned").appearance = appearance.clone();
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::AppearanceChanged,
            None,
            serde_json::json!({ "theme": appearance.theme, "brand_name": appearance.brand_name }),
        )
        .await
    }

    /// Enable/disable the panel (`admin.enabled`). Audited as a config change.
    pub async fn set_enabled(&self, actor: &str, enabled: bool) -> Result<(), AdminError> {
        self.config.lock().expect("config poisoned").enabled = enabled;
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::ConfigReloaded,
            None,
            serde_json::json!({ "enabled": enabled }),
        )
        .await
    }

    // ── Integrations (oversight) ─────────────────────────────────────────────

    /// The integrations surface (webhooks + API-key oversight live; LDAP/
    /// Nextcloud deferred).
    pub fn integrations(&self) -> IntegrationsConfig {
        IntegrationsConfig::default()
    }

    /// Oversight action: mark an API key revoked (the key store is `mw-oauth`;
    /// e11 wires the real revoke — here we audit the admin oversight action).
    pub async fn revoke_api_key(&self, actor: &str, key_id: &str) -> Result<(), AdminError> {
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::ApiKeyRevoked,
            Some(key_id.to_string()),
            serde_json::json!({}),
        )
        .await
    }

    // ── Login monitor / ban list ─────────────────────────────────────────────

    /// Record a successful login (clears the failure counter + audits).
    pub async fn record_login_success(&self, account: &str, ip: &str) -> Result<(), AdminError> {
        self.monitor.record_success(ip);
        self.emit(
            account,
            ActorKind::User,
            AuditKind::LoginSucceeded,
            Some(account.to_string()),
            serde_json::json!({}),
        )
        .await?;
        Ok(())
    }

    /// Record a failed login. Emits a fail2ban-compatible line, audits the
    /// failure, and auto-bans the source when the threshold is crossed.
    pub async fn record_login_failure(
        &self,
        account: &str,
        ip: &str,
    ) -> Result<LoginFailureOutcome, AdminError> {
        let now = chrono::Utc::now();
        let verdict = self.monitor.record_failure(ip, now);
        let log_line = fail2ban_line(now, account, ip);
        self.emit(
            account,
            ActorKind::User,
            AuditKind::LoginFailed,
            Some(account.to_string()),
            serde_json::json!({}),
        )
        .await?;

        let banned = matches!(verdict, LoginVerdict::Ban { .. });
        if banned {
            let failures = match verdict {
                LoginVerdict::Ban { failures } => failures,
                LoginVerdict::Watched { failures } => failures,
            };
            self.ban_ip(
                "system",
                ip,
                &format!("brute-force: {failures} failures"),
                None,
            )
            .await?;
        }
        Ok(LoginFailureOutcome {
            verdict,
            log_line,
            banned,
        })
    }

    pub async fn ban_ip(
        &self,
        actor: &str,
        ip: &str,
        reason: &str,
        expires_at: Option<String>,
    ) -> Result<(), AdminError> {
        self.backend
            .add_ban(BanEntry {
                ip: ip.to_string(),
                reason: reason.to_string(),
                banned_at: chrono::Utc::now().to_rfc3339(),
                expires_at,
            })
            .await?;
        let kind = if actor == "system" {
            ActorKind::System
        } else {
            ActorKind::Admin
        };
        self.emit(
            actor,
            kind,
            AuditKind::IpBanned,
            Some(ip.to_string()),
            serde_json::json!({ "reason": reason }),
        )
        .await
    }

    pub async fn unban_ip(&self, actor: &str, ip: &str) -> Result<(), AdminError> {
        self.backend.remove_ban(ip).await?;
        self.emit(
            actor,
            ActorKind::Admin,
            AuditKind::IpUnbanned,
            Some(ip.to_string()),
            serde_json::json!({}),
        )
        .await
    }

    pub async fn list_bans(&self) -> Result<Vec<BanEntry>, AdminError> {
        self.backend.list_bans().await
    }

    pub async fn is_banned(&self, ip: &str) -> Result<bool, AdminError> {
        self.backend.is_banned(ip).await
    }
}

impl AuditEvent {
    /// Set the target from an `Option` (internal convenience).
    #[must_use]
    fn target_opt(mut self, target: Option<String>) -> Self {
        self.target = target;
        self
    }
}

/// The canonical account id for a `(username, domain)` pair.
pub fn account_id(username: &str, domain: &str) -> String {
    format!("{username}@{domain}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn provision_writes_user_quota_and_audit() {
        let admin = Admin::in_memory();
        admin
            .provision_user(
                "root",
                "example.com",
                "alice",
                Quota {
                    bytes_limit: 1000,
                    msg_limit: 50,
                },
            )
            .await
            .unwrap();

        let acct = account_id("alice", "example.com");
        assert_eq!(
            admin.get_quota(&acct).await.unwrap(),
            Some(Quota {
                bytes_limit: 1000,
                msg_limit: 50
            })
        );
        let audit = admin.list_audit(10).await.unwrap();
        assert_eq!(audit.len(), 1);
        assert_eq!(audit[0].action, "user-provisioned");
        assert_eq!(audit[0].target.as_deref(), Some(acct.as_str()));
    }

    #[tokio::test]
    async fn quota_enforcement_via_admin() {
        let admin = Admin::in_memory();
        let acct = account_id("bob", "example.com");
        admin
            .set_quota(
                "root",
                &acct,
                Quota {
                    bytes_limit: 100,
                    msg_limit: 2,
                },
            )
            .await
            .unwrap();
        assert!(admin.check_quota(&acct, 50, 1).await.is_ok());
        assert!(matches!(
            admin.check_quota(&acct, 200, 1).await,
            Err(AdminError::QuotaExceeded)
        ));
        // No quota row → unrestricted.
        assert!(
            admin
                .check_quota("nobody@x", i64::MAX, i64::MAX)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn zero_access_toggle_updates_flags_and_audits() {
        let admin = Admin::in_memory();
        let acct = account_id("carol", "example.com");
        admin.toggle_zero_access("root", &acct, true).await.unwrap();
        assert!(admin.get_feature_flags(&acct).await.unwrap().zero_access);
        let audit = admin.list_audit(10).await.unwrap();
        assert_eq!(audit[0].action, "zero-access-toggled");
    }

    #[tokio::test]
    async fn failed_logins_auto_ban_and_log_fail2ban() {
        let admin = Admin::in_memory();
        let mut last = None;
        for _ in 0..5 {
            last = Some(
                admin
                    .record_login_failure("dave", "198.51.100.9")
                    .await
                    .unwrap(),
            );
        }
        let outcome = last.unwrap();
        assert!(outcome.banned, "5th failure should ban");
        assert_eq!(
            banlist::parse_host(&outcome.log_line).as_deref(),
            Some("198.51.100.9")
        );
        assert!(admin.is_banned("198.51.100.9").await.unwrap());
    }

    #[tokio::test]
    async fn audit_detail_is_redacted() {
        let admin = Admin::in_memory();
        admin
            .record(
                AuditEvent::new("root", ActorKind::Admin, AuditKind::ConfigReloaded)
                    .detail(serde_json::json!({ "password": "leak-me", "note": "x@y.com" })),
            )
            .await
            .unwrap();
        let audit = admin.list_audit(1).await.unwrap();
        assert!(!audit[0].detail_json.contains("leak-me"));
        assert!(!audit[0].detail_json.contains("x@y.com"));
    }

    #[tokio::test]
    async fn panel_disable_flag_models_unmount() {
        let admin = Admin::in_memory();
        assert!(admin.panel_enabled());
        admin.set_enabled("root", false).await.unwrap();
        assert!(!admin.panel_enabled());
    }
}
