//! The admin persistence PORT (plan §2.5, §2.1 tables). `mw-admin` owns the
//! domain logic; the concrete persistence lives behind this trait so the crate
//! is testable in isolation and does not hard-couple to the in-flight `mw-store`
//! 0007 tables (e1). e11 supplies a `mw-store`-backed adapter at mount time;
//! [`InMemoryBackend`] is used here + by the crate's tests.
//!
//! **Append-only audit invariant (§2.5).** This trait exposes ONLY
//! [`AdminBackend::append_audit`] / [`AdminBackend::list_audit`] for the audit
//! log — there is deliberately NO `update_audit`/`delete_audit`/`clear_audit`
//! method. The append-only guarantee is therefore STRUCTURAL: no caller can
//! reach a mutation path because none is defined. `crates/mw-admin/src/audit.rs`
//! asserts this in a test.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;

use crate::{
    AdminError, AuditLogEntry, BanEntry, CacheScopeRow, Domain, ObservabilityConfig, Quota,
    SecurityPolicy, UserFeatureFlags,
};

/// A provisioned mail user (`admin_users`/`quotas`, 0007). `password_hash` is
/// Argon2id (NULL = passkey-only), never plaintext.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRecord {
    pub username: String,
    pub domain: String,
    pub password_hash: Option<String>,
}

/// The persistence port for the admin domain logic. Object-safe (via
/// `async_trait`) so `Admin` can hold `Arc<dyn AdminBackend>`.
///
/// The audit surface is intentionally append + read only — see the module docs.
#[async_trait]
pub trait AdminBackend: Send + Sync {
    // ── Audit log — APPEND-ONLY. No update/delete path exists. ──────────────
    /// Append one immutable audit record. The ONLY audit mutation entry point.
    async fn append_audit(&self, entry: AuditLogEntry) -> Result<(), AdminError>;
    /// Read the most recent `limit` audit records, newest first.
    async fn list_audit(&self, limit: usize) -> Result<Vec<AuditLogEntry>, AdminError>;

    // ── Domains ─────────────────────────────────────────────────────────────
    async fn upsert_domain(&self, domain: Domain) -> Result<(), AdminError>;
    async fn get_domain(&self, name: &str) -> Result<Option<Domain>, AdminError>;
    async fn list_domains(&self) -> Result<Vec<Domain>, AdminError>;
    async fn delete_domain(&self, name: &str) -> Result<(), AdminError>;

    // ── Users / quotas / feature flags ───────────────────────────────────────
    async fn upsert_user(&self, user: UserRecord) -> Result<(), AdminError>;
    async fn get_user(&self, account_id: &str) -> Result<Option<UserRecord>, AdminError>;
    async fn set_quota(&self, account_id: &str, quota: Quota) -> Result<(), AdminError>;
    async fn get_quota(&self, account_id: &str) -> Result<Option<Quota>, AdminError>;
    async fn set_flags(&self, account_id: &str, flags: UserFeatureFlags) -> Result<(), AdminError>;
    async fn get_flags(&self, account_id: &str) -> Result<UserFeatureFlags, AdminError>;
    /// Revoke all sessions for an account; returns the number revoked. The
    /// concrete adapter deletes the account's `sessions`/`native_sessions` rows.
    async fn revoke_sessions(&self, account_id: &str) -> Result<u64, AdminError>;

    // ── Config surfaces ──────────────────────────────────────────────────────
    async fn set_security_policy(&self, policy: SecurityPolicy) -> Result<(), AdminError>;
    async fn get_security_policy(&self) -> Result<Option<SecurityPolicy>, AdminError>;
    async fn set_observability(&self, cfg: ObservabilityConfig) -> Result<(), AdminError>;
    async fn get_observability(&self) -> Result<Option<ObservabilityConfig>, AdminError>;
    async fn upsert_cache_scope(&self, row: CacheScopeRow) -> Result<(), AdminError>;
    async fn list_cache_scope(&self) -> Result<Vec<CacheScopeRow>, AdminError>;

    // ── Login monitor / ban list ─────────────────────────────────────────────
    async fn add_ban(&self, ban: BanEntry) -> Result<(), AdminError>;
    async fn remove_ban(&self, ip: &str) -> Result<(), AdminError>;
    async fn list_bans(&self) -> Result<Vec<BanEntry>, AdminError>;
    async fn is_banned(&self, ip: &str) -> Result<bool, AdminError>;
}

/// In-memory [`AdminBackend`] — the default backend for `Admin::default()` and
/// the whole crate's test-suite. The audit log is a plain append-only `Vec`;
/// no method removes or rewrites an entry.
#[derive(Default)]
pub struct InMemoryBackend {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    audit: Vec<AuditLogEntry>,
    domains: HashMap<String, Domain>,
    users: HashMap<String, UserRecord>,
    quotas: HashMap<String, Quota>,
    flags: HashMap<String, UserFeatureFlags>,
    sessions: HashMap<String, u64>,
    security: Option<SecurityPolicy>,
    observability: Option<ObservabilityConfig>,
    cache_scope: HashMap<String, CacheScopeRow>,
    bans: HashMap<String, BanEntry>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed a session count for an account so `revoke_sessions` has something to
    /// report (tests / harnesses).
    pub fn seed_sessions(&self, account_id: &str, count: u64) {
        self.inner
            .lock()
            .expect("poisoned")
            .sessions
            .insert(account_id.to_string(), count);
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().expect("admin backend mutex poisoned")
    }
}

#[async_trait]
impl AdminBackend for InMemoryBackend {
    async fn append_audit(&self, entry: AuditLogEntry) -> Result<(), AdminError> {
        // The sole audit mutation: push. Never rewrite or remove.
        self.lock().audit.push(entry);
        Ok(())
    }

    async fn list_audit(&self, limit: usize) -> Result<Vec<AuditLogEntry>, AdminError> {
        let g = self.lock();
        Ok(g.audit.iter().rev().take(limit).cloned().collect())
    }

    async fn upsert_domain(&self, domain: Domain) -> Result<(), AdminError> {
        self.lock().domains.insert(domain.name.clone(), domain);
        Ok(())
    }

    async fn get_domain(&self, name: &str) -> Result<Option<Domain>, AdminError> {
        Ok(self.lock().domains.get(name).cloned())
    }

    async fn list_domains(&self) -> Result<Vec<Domain>, AdminError> {
        let mut v: Vec<Domain> = self.lock().domains.values().cloned().collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(v)
    }

    async fn delete_domain(&self, name: &str) -> Result<(), AdminError> {
        self.lock()
            .domains
            .remove(name)
            .map(|_| ())
            .ok_or(AdminError::NotFound)
    }

    async fn upsert_user(&self, user: UserRecord) -> Result<(), AdminError> {
        let key = format!("{}@{}", user.username, user.domain);
        self.lock().users.insert(key, user);
        Ok(())
    }

    async fn get_user(&self, account_id: &str) -> Result<Option<UserRecord>, AdminError> {
        Ok(self.lock().users.get(account_id).cloned())
    }

    async fn set_quota(&self, account_id: &str, quota: Quota) -> Result<(), AdminError> {
        self.lock().quotas.insert(account_id.to_string(), quota);
        Ok(())
    }

    async fn get_quota(&self, account_id: &str) -> Result<Option<Quota>, AdminError> {
        Ok(self.lock().quotas.get(account_id).copied())
    }

    async fn set_flags(&self, account_id: &str, flags: UserFeatureFlags) -> Result<(), AdminError> {
        self.lock().flags.insert(account_id.to_string(), flags);
        Ok(())
    }

    async fn get_flags(&self, account_id: &str) -> Result<UserFeatureFlags, AdminError> {
        Ok(self
            .lock()
            .flags
            .get(account_id)
            .copied()
            .unwrap_or_default())
    }

    async fn revoke_sessions(&self, account_id: &str) -> Result<u64, AdminError> {
        Ok(self.lock().sessions.remove(account_id).unwrap_or(0))
    }

    async fn set_security_policy(&self, policy: SecurityPolicy) -> Result<(), AdminError> {
        self.lock().security = Some(policy);
        Ok(())
    }

    async fn get_security_policy(&self) -> Result<Option<SecurityPolicy>, AdminError> {
        Ok(self.lock().security.clone())
    }

    async fn set_observability(&self, cfg: ObservabilityConfig) -> Result<(), AdminError> {
        self.lock().observability = Some(cfg);
        Ok(())
    }

    async fn get_observability(&self) -> Result<Option<ObservabilityConfig>, AdminError> {
        Ok(self.lock().observability.clone())
    }

    async fn upsert_cache_scope(&self, row: CacheScopeRow) -> Result<(), AdminError> {
        self.lock().cache_scope.insert(row.class.clone(), row);
        Ok(())
    }

    async fn list_cache_scope(&self) -> Result<Vec<CacheScopeRow>, AdminError> {
        let mut v: Vec<CacheScopeRow> = self.lock().cache_scope.values().cloned().collect();
        v.sort_by(|a, b| a.class.cmp(&b.class));
        Ok(v)
    }

    async fn add_ban(&self, ban: BanEntry) -> Result<(), AdminError> {
        self.lock().bans.insert(ban.ip.clone(), ban);
        Ok(())
    }

    async fn remove_ban(&self, ip: &str) -> Result<(), AdminError> {
        self.lock().bans.remove(ip);
        Ok(())
    }

    async fn list_bans(&self) -> Result<Vec<BanEntry>, AdminError> {
        let mut v: Vec<BanEntry> = self.lock().bans.values().cloned().collect();
        v.sort_by(|a, b| a.ip.cmp(&b.ip));
        Ok(v)
    }

    async fn is_banned(&self, ip: &str) -> Result<bool, AdminError> {
        Ok(self.lock().bans.contains_key(ip))
    }
}
