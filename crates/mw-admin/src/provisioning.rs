//! User/domain/quota provisioning helpers + per-user feature flags + the
//! integrations config model (plan §2.5, §19).

use serde::{Deserialize, Serialize};

use crate::{AdminError, Quota};

/// Per-user feature flags (§2.5 users section): the zero-access toggle,
/// force-password-change, remote-cache-wipe, plus an account-disable switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UserFeatureFlags {
    /// Zero-access encrypted-at-rest storage enabled for this account (§9).
    pub zero_access: bool,
    /// Force a password change on next login.
    pub force_password_change: bool,
    /// One-shot remote cache wipe requested (cleared once honored by the engine).
    pub remote_cache_wipe: bool,
    /// Account administratively disabled (login refused).
    pub disabled: bool,
}

impl Quota {
    /// An unlimited quota (a non-positive limit means "no limit").
    pub const UNLIMITED: Quota = Quota {
        bytes_limit: 0,
        msg_limit: 0,
    };

    /// Whether `bytes_used`/`msg_used` are within this quota. A non-positive
    /// limit is treated as unlimited.
    pub fn allows(&self, bytes_used: i64, msg_used: i64) -> bool {
        let bytes_ok = self.bytes_limit <= 0 || bytes_used <= self.bytes_limit;
        let msg_ok = self.msg_limit <= 0 || msg_used <= self.msg_limit;
        bytes_ok && msg_ok
    }

    /// Enforce the quota, returning [`AdminError::QuotaExceeded`] when over.
    pub fn enforce(&self, bytes_used: i64, msg_used: i64) -> Result<(), AdminError> {
        if self.allows(bytes_used, msg_used) {
            Ok(())
        } else {
            Err(AdminError::QuotaExceeded)
        }
    }
}

/// Deferred-integration status (plan §2.5: LDAP/Nextcloud entries are INERT until
/// V7 — the panel shows the config surface but no live glue).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationStatus {
    /// Live and configurable now.
    Active,
    /// Config surface present but not yet wired (LDAP/Nextcloud → V7).
    Deferred,
}

/// The integrations surface (§19 integrations): live webhooks + MCP/API-key
/// oversight, plus the inert LDAP/Nextcloud entries (deferred to V7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct IntegrationsConfig {
    /// Webhooks feature availability (the concrete delivery lives in `mw-server`,
    /// e9). Active in V6.
    pub webhooks: IntegrationStatus,
    /// MCP + API-key oversight (list/revoke via the admin surface). Active in V6.
    pub api_key_oversight: IntegrationStatus,
    /// LDAP/GAL directory — deferred to V7 (config surface only).
    pub ldap: IntegrationStatus,
    /// Nextcloud bridge — deferred to V7 (config surface only).
    pub nextcloud: IntegrationStatus,
}

impl Default for IntegrationsConfig {
    fn default() -> Self {
        Self {
            webhooks: IntegrationStatus::Active,
            api_key_oversight: IntegrationStatus::Active,
            ldap: IntegrationStatus::Deferred,
            nextcloud: IntegrationStatus::Deferred,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quota_within_and_over() {
        let q = Quota {
            bytes_limit: 1000,
            msg_limit: 10,
        };
        assert!(q.allows(500, 5));
        assert!(q.allows(1000, 10)); // at the limit is allowed
        assert!(!q.allows(1001, 5));
        assert!(!q.allows(500, 11));
        assert!(q.enforce(999, 9).is_ok());
        assert!(matches!(q.enforce(2000, 1), Err(AdminError::QuotaExceeded)));
    }

    #[test]
    fn unlimited_quota_never_exceeds() {
        assert!(Quota::UNLIMITED.allows(i64::MAX, i64::MAX));
        assert!(Quota::UNLIMITED.enforce(i64::MAX, i64::MAX).is_ok());
    }

    #[test]
    fn flags_default_all_off() {
        let f = UserFeatureFlags::default();
        assert!(!f.zero_access && !f.force_password_change && !f.remote_cache_wipe && !f.disabled);
    }

    #[test]
    fn integrations_defer_ldap_and_nextcloud() {
        let i = IntegrationsConfig::default();
        assert_eq!(i.webhooks, IntegrationStatus::Active);
        assert_eq!(i.ldap, IntegrationStatus::Deferred);
        assert_eq!(i.nextcloud, IntegrationStatus::Deferred);
    }
}
