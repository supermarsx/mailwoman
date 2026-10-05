//! User/domain/quota provisioning helpers + per-user feature flags + the
//! integrations status model (plan §2.5, §19).

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

/// A change to one field of an account's [`UserFeatureFlags`], for a writer that
/// means that field and no other (`AdminBackend::update_flags`). `disabled` has no
/// variant: it is set by saving the whole record, which also revokes sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagUpdate {
    ZeroAccess(bool),
    ForcePasswordChange(bool),
    RemoteCacheWipe(bool),
}

impl FlagUpdate {
    /// Set this update's field in `flags`, leaving the others as they are.
    pub fn apply(self, flags: &mut UserFeatureFlags) {
        match self {
            Self::ZeroAccess(on) => flags.zero_access = on,
            Self::ForcePasswordChange(on) => flags.force_password_change = on,
            Self::RemoteCacheWipe(on) => flags.remote_cache_wipe = on,
        }
    }
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

/// What the admin surface can say about one integration.
///
/// `Active` is for the surfaces this crate's own routes serve. `Configured` /
/// `NotConfigured` describe an integration whose configuration lives outside this
/// crate (LDAP rows in the store, the Nextcloud environment variables); only the
/// running server can tell which, so the default is `Unknown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationStatus {
    /// Served by the admin surface itself.
    Active,
    /// The deployment has configuration for it.
    Configured,
    /// The deployment has no configuration for it.
    NotConfigured,
    /// Not determined by whoever produced this value.
    Unknown,
}

impl IntegrationStatus {
    /// The wire / CLI spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            IntegrationStatus::Active => "active",
            IntegrationStatus::Configured => "configured",
            IntegrationStatus::NotConfigured => "not-configured",
            IntegrationStatus::Unknown => "unknown",
        }
    }
}

/// The integrations surface (§19 integrations): webhooks + MCP/API-key oversight,
/// and the configuration state of the LDAP directory and the Nextcloud bridge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct IntegrationsConfig {
    /// The webhook oversight list (`GET /admin/webhooks`).
    pub webhooks: IntegrationStatus,
    /// MCP + API-key oversight (list/revoke via the admin surface).
    pub api_key_oversight: IntegrationStatus,
    /// LDAP/GAL directory. Configured through `directory_config` rows, which this
    /// crate does not read.
    pub ldap: IntegrationStatus,
    /// Nextcloud bridge. Configured through `MW_NEXTCLOUD_*`, which this crate does
    /// not read.
    pub nextcloud: IntegrationStatus,
}

impl Default for IntegrationsConfig {
    /// What can be said without asking the running server: the two oversight lists
    /// exist, and nothing is known about LDAP or Nextcloud.
    fn default() -> Self {
        Self {
            webhooks: IntegrationStatus::Active,
            api_key_oversight: IntegrationStatus::Active,
            ldap: IntegrationStatus::Unknown,
            nextcloud: IntegrationStatus::Unknown,
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
    fn integrations_default_claims_nothing_about_ldap_or_nextcloud() {
        let i = IntegrationsConfig::default();
        assert_eq!(i.webhooks, IntegrationStatus::Active);
        assert_eq!(i.ldap, IntegrationStatus::Unknown);
        assert_eq!(i.nextcloud, IntegrationStatus::Unknown);
    }

    #[test]
    fn integration_status_spellings() {
        assert_eq!(IntegrationStatus::Active.as_str(), "active");
        assert_eq!(IntegrationStatus::Configured.as_str(), "configured");
        assert_eq!(IntegrationStatus::NotConfigured.as_str(), "not-configured");
        assert_eq!(IntegrationStatus::Unknown.as_str(), "unknown");
    }
}
