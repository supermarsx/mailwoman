//! 0033 settings model C repository (26.20 t32-e10b, worker f01): dual-backend
//! `Store` methods over `setting_policy` and `setting_value`, and the resolution
//! chain that turns their rows into one effective value.
//!
//! # Tables
//! * `setting_policy` — one row per key with a global value (`NULL` = the
//!   built-in default), whether users may override it, and whether the global
//!   value is locked.
//! * `setting_value` — one row per `(scope_kind, scope_id, key)` for the
//!   narrower scopes: `domain`, `profile`, `group`, `admin_user` (a value an admin
//!   set for one user) and `user` (the user's own choice). `locked` on a row locks
//!   that scope for every narrower one.
//!
//! The store does not know what a key means or which values are valid: values are
//! JSON text validated by `mw-server`'s settings registry before every write.
//!
//! # Resolution (t32-admin §2.4, L6)
//! Broad to narrow: global < domain < profile < group < admin per-user < user.
//! The narrowest value that is set wins, with two exceptions:
//! * a lock at any scope wins over every narrower scope (the broadest lock wins);
//! * the user's own value counts only while the policy allows user overrides.
//!
//! With nothing set, the floor applies: the built-in default, or an environment
//! fallback in its place. An environment pin is passed in by the caller and wins
//! over everything. [`resolve`] is a pure function so the registry, the admin
//! simulator and the tests share one implementation.
//!
//! Profiles and groups are resolved by [`Store::setting_profile_and_group_ids`],
//! using live profile assignments and group membership.
//!
//! Timestamps are unix seconds and booleans are 0/1 (0033 wire contract).

use crate::backend::{Row, q};
use crate::{Store, StoreError};

/// Shared IDNA domain normalization for login configuration and policy lookup.
/// Reject addresses, URLs, ports and IP literals instead of reinterpreting them.
pub fn normalize_policy_domain(input: &str) -> Result<String, StoreError> {
    let input = input.trim().trim_end_matches('.');
    let invalid = || StoreError::Corrupt("invalid policy domain name".into());
    if input.is_empty() || input.contains(['/', '\\', '@', ':', '?', '#', '%']) {
        return Err(invalid());
    }
    let host = url::Host::parse(input).map_err(|_| invalid())?;
    let url::Host::Domain(host) = host else {
        return Err(invalid());
    };
    if host.len() > 253
        || !host.split('.').all(|part| {
            !part.is_empty()
                && part.len() <= 63
                && !part.starts_with('-')
                && !part.ends_with('-')
                && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
    {
        return Err(invalid());
    }
    Ok(host.to_ascii_lowercase())
}

/// A scope narrower than global (`setting_value.scope_kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SettingScope {
    Domain,
    Profile,
    Group,
    AdminUser,
    User,
}

impl SettingScope {
    /// Every scope, broad to narrow.
    pub const ALL: [SettingScope; 5] = [
        SettingScope::Domain,
        SettingScope::Profile,
        SettingScope::Group,
        SettingScope::AdminUser,
        SettingScope::User,
    ];

    /// The `scope_kind` column value.
    pub fn as_str(self) -> &'static str {
        match self {
            SettingScope::Domain => "domain",
            SettingScope::Profile => "profile",
            SettingScope::Group => "group",
            SettingScope::AdminUser => "admin_user",
            SettingScope::User => "user",
        }
    }

    /// Parse a `scope_kind` column value.
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == s)
    }
}

/// One `setting_policy` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingPolicyRow {
    pub key: String,
    /// JSON text, or `None` for "use the built-in default".
    pub global_value: Option<String>,
    pub allow_user_override: bool,
    pub global_locked: bool,
    pub updated_by: String,
    /// Unix seconds.
    pub updated_at: i64,
}

/// One `setting_value` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingValueRow {
    pub scope: SettingScope,
    pub scope_id: String,
    pub key: String,
    /// JSON text.
    pub value: String,
    pub locked: bool,
    pub updated_by: String,
    /// Unix seconds.
    pub updated_at: i64,
}

/// How many `setting_value` rows one key has at one scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingValueCount {
    pub key: String,
    pub scope: SettingScope,
    pub count: i64,
}

fn policy_from_row(r: &Row) -> SettingPolicyRow {
    SettingPolicyRow {
        key: r.get_string("key"),
        global_value: r.get_opt_string("global_value"),
        allow_user_override: r.get_i64("allow_user_override") != 0,
        global_locked: r.get_i64("global_locked") != 0,
        updated_by: r.get_string("updated_by"),
        updated_at: r.get_i64("updated_at"),
    }
}

fn value_from_row(r: &Row) -> Result<SettingValueRow, StoreError> {
    let kind = r.get_string("scope_kind");
    let scope = SettingScope::parse(&kind)
        .ok_or_else(|| StoreError::Corrupt(format!("setting_value.scope_kind {kind:?}")))?;
    Ok(SettingValueRow {
        scope,
        scope_id: r.get_string("scope_id"),
        key: r.get_string("key"),
        value: r.get_string("value"),
        locked: r.get_i64("locked") != 0,
        updated_by: r.get_string("updated_by"),
        updated_at: r.get_i64("updated_at"),
    })
}

impl Store {
    /// Resolve only explicit domain aliases for policy. This deliberately never
    /// consults the preferred domain: an existing bare login has no domain.
    pub async fn canonical_policy_domain(&self, domain: &str) -> Result<String, StoreError> {
        if domain.trim().is_empty() {
            return Ok(String::new());
        }
        let mut domain = normalize_policy_domain(domain)?;
        let mut seen = Vec::new();
        while !domain.is_empty() {
            if seen.len() >= 16 || seen.contains(&domain) {
                return Err(StoreError::Corrupt("cyclic domain policy alias".into()));
            }
            seen.push(domain.clone());
            let Some(target) = self.domain_alias_target(&domain).await? else {
                break;
            };
            domain = normalize_policy_domain(&target)?;
        }
        Ok(domain)
    }

    /// Canonical policy address, without changing the credential/login identity.
    /// Bare names remain bare, rather than acquiring the current default domain.
    pub async fn canonical_policy_address(&self, address: &str) -> Result<String, StoreError> {
        let address = address.trim().to_lowercase();
        let Some((local, domain)) = address.rsplit_once('@') else {
            return Ok(address);
        };
        Ok(format!(
            "{local}@{}",
            self.canonical_policy_domain(domain).await?
        ))
    }

    /// Domains belonging to the same configured mail namespace, for internal
    /// forwarding classification. Empty/bare identities have no domain family.
    pub async fn policy_domain_family(&self, domain: &str) -> Result<Vec<String>, StoreError> {
        let canonical = self.canonical_policy_domain(domain).await?;
        if canonical.is_empty() {
            return Ok(Vec::new());
        }
        let mut domains = self.domain_aliases(&canonical).await?;
        domains.push(canonical);
        domains.sort();
        domains.dedup();
        Ok(domains)
    }

    pub async fn canonical_setting_subject(
        &self,
        subject: &SettingSubject<'_>,
    ) -> Result<CanonicalSettingSubject, StoreError> {
        let address = self.canonical_policy_address(subject.address).await?;
        let domain = self.canonical_policy_domain(subject.domain).await?;
        Ok(CanonicalSettingSubject {
            domain,
            account_id: subject.account_id.to_owned(),
            address,
        })
    }

    /// The policy row of `key`, or `None` when nothing was ever stored for it.
    pub async fn get_setting_policy(
        &self,
        key: &str,
    ) -> Result<Option<SettingPolicyRow>, StoreError> {
        let row = q(
            "SELECT key, global_value, allow_user_override, global_locked, updated_by, updated_at
                     FROM setting_policy WHERE key = ?1",
        )
        .bind(key)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(policy_from_row))
    }

    /// Every stored policy row, in key order.
    pub async fn list_setting_policies(&self) -> Result<Vec<SettingPolicyRow>, StoreError> {
        let rows = q(
            "SELECT key, global_value, allow_user_override, global_locked, updated_by, updated_at
                      FROM setting_policy ORDER BY key",
        )
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(policy_from_row).collect())
    }

    /// Insert or replace the policy row of `row.key`.
    pub async fn put_setting_policy(&self, row: &SettingPolicyRow) -> Result<(), StoreError> {
        q("INSERT INTO setting_policy
               (key, global_value, allow_user_override, global_locked, updated_by, updated_at)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6)
           ON CONFLICT(key) DO UPDATE SET
               global_value = excluded.global_value,
               allow_user_override = excluded.allow_user_override,
               global_locked = excluded.global_locked,
               updated_by = excluded.updated_by,
               updated_at = excluded.updated_at")
        .bind(&row.key)
        .bind(row.global_value.clone())
        .bind(i64::from(row.allow_user_override))
        .bind(i64::from(row.global_locked))
        .bind(&row.updated_by)
        .bind(row.updated_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Delete the policy row of `key` (value and flags back to the built-in
    /// defaults). `true` when a row was removed.
    pub async fn delete_setting_policy(&self, key: &str) -> Result<bool, StoreError> {
        let n = q("DELETE FROM setting_policy WHERE key = ?1")
            .bind(key)
            .execute(&self.backend)
            .await?;
        Ok(n > 0)
    }

    /// One scoped value row.
    pub async fn get_setting_value(
        &self,
        scope: SettingScope,
        scope_id: &str,
        key: &str,
    ) -> Result<Option<SettingValueRow>, StoreError> {
        let canonical_id;
        let scope_id = if scope == SettingScope::Domain {
            canonical_id = self.canonical_policy_domain(scope_id).await?;
            canonical_id.as_str()
        } else {
            scope_id
        };
        let row = q(
            "SELECT scope_kind, scope_id, key, value, locked, updated_by, updated_at
                     FROM setting_value WHERE scope_kind = ?1 AND scope_id = ?2 AND key = ?3",
        )
        .bind(scope.as_str())
        .bind(scope_id)
        .bind(key)
        .fetch_optional(&self.backend)
        .await?;
        row.as_ref().map(value_from_row).transpose()
    }

    /// Insert or replace one scoped value row.
    pub async fn put_setting_value(&self, row: &SettingValueRow) -> Result<(), StoreError> {
        let scope_id = if row.scope == SettingScope::Domain {
            self.canonical_policy_domain(&row.scope_id).await?
        } else {
            row.scope_id.clone()
        };
        q("INSERT INTO setting_value
               (scope_kind, scope_id, key, value, locked, updated_by, updated_at)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
           ON CONFLICT(scope_kind, scope_id, key) DO UPDATE SET
               value = excluded.value,
               locked = excluded.locked,
               updated_by = excluded.updated_by,
               updated_at = excluded.updated_at")
        .bind(row.scope.as_str())
        .bind(&scope_id)
        .bind(&row.key)
        .bind(&row.value)
        .bind(i64::from(row.locked))
        .bind(&row.updated_by)
        .bind(row.updated_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Delete one scoped value row. `true` when a row was removed.
    pub async fn delete_setting_value(
        &self,
        scope: SettingScope,
        scope_id: &str,
        key: &str,
    ) -> Result<bool, StoreError> {
        let canonical_id;
        let scope_id = if scope == SettingScope::Domain {
            canonical_id = self.canonical_policy_domain(scope_id).await?;
            canonical_id.as_str()
        } else {
            scope_id
        };
        let n = q("DELETE FROM setting_value WHERE scope_kind = ?1 AND scope_id = ?2 AND key = ?3")
            .bind(scope.as_str())
            .bind(scope_id)
            .bind(key)
            .execute(&self.backend)
            .await?;
        Ok(n > 0)
    }

    /// Delete every value row of one scope id (all keys), e.g. when a user or a
    /// domain is deleted. Returns the number of rows removed.
    pub async fn delete_setting_values_for(
        &self,
        scope: SettingScope,
        scope_id: &str,
    ) -> Result<u64, StoreError> {
        let canonical_id;
        let scope_id = if scope == SettingScope::Domain {
            canonical_id = self.canonical_policy_domain(scope_id).await?;
            canonical_id.as_str()
        } else {
            scope_id
        };
        Ok(
            q("DELETE FROM setting_value WHERE scope_kind = ?1 AND scope_id = ?2")
                .bind(scope.as_str())
                .bind(scope_id)
                .execute(&self.backend)
                .await?,
        )
    }

    /// A page of `key`'s value rows at `scope`, in `scope_id` order, starting after
    /// the `after` cursor (a `scope_id`). At most `limit` rows (1..=500).
    pub async fn list_setting_values(
        &self,
        key: &str,
        scope: SettingScope,
        after: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SettingValueRow>, StoreError> {
        let limit = limit.clamp(1, 500);
        let rows = q(
            "SELECT scope_kind, scope_id, key, value, locked, updated_by, updated_at
                      FROM setting_value
                      WHERE key = ?1 AND scope_kind = ?2 AND scope_id > ?3
                      ORDER BY scope_id LIMIT ?4",
        )
        .bind(key)
        .bind(scope.as_str())
        .bind(after.unwrap_or(""))
        .bind(limit)
        .fetch_all(&self.backend)
        .await?;
        rows.iter().map(value_from_row).collect()
    }

    /// The number of value rows per `(key, scope)`, for the admin overview.
    pub async fn count_setting_values(&self) -> Result<Vec<SettingValueCount>, StoreError> {
        let rows = q("SELECT key, scope_kind, COUNT(*) AS n FROM setting_value
                      GROUP BY key, scope_kind ORDER BY key, scope_kind")
        .fetch_all(&self.backend)
        .await?;
        rows.iter()
            .map(|r| {
                let kind = r.get_string("scope_kind");
                let scope = SettingScope::parse(&kind).ok_or_else(|| {
                    StoreError::Corrupt(format!("setting_value.scope_kind {kind:?}"))
                })?;
                Ok(SettingValueCount {
                    key: r.get_string("key"),
                    scope,
                    count: r.get_i64("n"),
                })
            })
            .collect()
    }

    /// Every value row that can apply to one subject, for all keys at once: the
    /// subject's domain, its profiles and groups, and the per-user rows under
    /// either of its two ids (the account id, and the canonical address that the
    /// `limit.*`/`quota.*` keys use). Pass an empty string for an id the caller
    /// does not have.
    pub async fn setting_values_for_subject(
        &self,
        subject: &SettingSubject<'_>,
    ) -> Result<Vec<SettingValueRow>, StoreError> {
        let canonical = self.canonical_setting_subject(subject).await?;
        let subject = &canonical.as_subject();
        let (profiles, groups) = self
            .setting_profile_and_group_ids(subject.account_id, subject.domain)
            .await?;
        let rows = q(
            "SELECT scope_kind, scope_id, key, value, locked, updated_by, updated_at
                      FROM setting_value
                      WHERE (scope_kind = 'domain' AND scope_id = ?1 AND ?1 <> '')
                         OR (scope_kind IN ('admin_user', 'user')
                             AND ((scope_id = ?2 AND ?2 <> '') OR (scope_id = ?3 AND ?3 <> '')))",
        )
        .bind(subject.domain)
        .bind(subject.account_id)
        .bind(subject.address)
        .fetch_all(&self.backend)
        .await?;
        let mut out = rows
            .iter()
            .map(value_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        for id in &profiles {
            out.extend(self.setting_values_at(SettingScope::Profile, id).await?);
        }
        for id in &groups {
            out.extend(self.setting_values_at(SettingScope::Group, id).await?);
        }
        Ok(out)
    }

    /// Every value row of one scope id.
    pub async fn setting_values_at(
        &self,
        scope: SettingScope,
        scope_id: &str,
    ) -> Result<Vec<SettingValueRow>, StoreError> {
        let canonical_id;
        let scope_id = if scope == SettingScope::Domain {
            canonical_id = self.canonical_policy_domain(scope_id).await?;
            canonical_id.as_str()
        } else {
            scope_id
        };
        let rows = q(
            "SELECT scope_kind, scope_id, key, value, locked, updated_by, updated_at
                      FROM setting_value WHERE scope_kind = ?1 AND scope_id = ?2 ORDER BY key",
        )
        .bind(scope.as_str())
        .bind(scope_id)
        .fetch_all(&self.backend)
        .await?;
        rows.iter().map(value_from_row).collect()
    }

    /// The value an administrator holds `key` at for `subject`, or `None` when the
    /// user may choose (no lock at any scope and user overrides allowed). `floor`
    /// is the key's built-in default as JSON text. For readers outside
    /// `mw-server` that cannot use its registry (the engine's read-receipt
    /// policy, the push relay's notification switch): they keep reading the
    /// user's own stored value unless this answers `Some`. The user's own value
    /// lives outside `setting_value` for those keys, so only admin layers are
    /// read here. No environment pin applies to such keys.
    pub async fn held_setting(
        &self,
        key: &str,
        floor: &str,
        subject: &SettingSubject<'_>,
    ) -> Result<Option<String>, StoreError> {
        let policy = self.get_setting_policy(key).await?;
        let rows = self.setting_values_for_subject(subject).await?;
        let mine: Vec<&SettingValueRow> = rows
            .iter()
            .filter(|r| r.key == key && r.scope != SettingScope::User)
            .collect();
        let r = resolve(&ResolveInput {
            floor,
            floor_is_env: false,
            pin: None,
            policy: policy.as_ref(),
            values: &mine,
        });
        Ok((!r.user_may_override).then_some(r.value))
    }

    /// The feature profiles and user groups that apply to a subject, highest
    /// priority first. Equal priorities prefer user, then group, then domain
    /// assignments, with stable profile-id ties. Groups have stable id order.
    /// Domain-specific groups never apply outside their canonical domain.
    pub async fn setting_profile_and_group_ids(
        &self,
        account_id: &str,
        domain: &str,
    ) -> Result<(Vec<String>, Vec<String>), StoreError> {
        let domain = self.canonical_policy_domain(domain).await?;
        let groups: Vec<String> = q("SELECT g.id FROM user_group g JOIN user_group_member m ON m.group_id=g.id WHERE m.account_id=?1 AND ?1<>'' AND (g.domain='' OR g.domain=?2) ORDER BY g.id")
            .bind(account_id).bind(&domain).fetch_all(&self.backend).await?.iter().map(|r|r.get_string("id")).collect();
        let rows = q("SELECT a.profile_id FROM feature_profile_assignment a WHERE (a.scope_kind='domain' AND a.scope_id=?1 AND ?1<>'') OR (a.scope_kind='user' AND a.scope_id=?2 AND ?2<>'') OR (a.scope_kind='group' AND EXISTS (SELECT 1 FROM user_group_member m JOIN user_group g ON g.id=m.group_id WHERE m.group_id=a.scope_id AND m.account_id=?2 AND ?2<>'' AND (g.domain='' OR g.domain=?1))) ORDER BY a.priority DESC,CASE a.scope_kind WHEN 'user' THEN 0 WHEN 'group' THEN 1 ELSE 2 END,a.profile_id,a.scope_id")
            .bind(&domain).bind(account_id).fetch_all(&self.backend).await?;
        let mut profiles = Vec::new();
        for row in rows {
            let id=row.get_string("profile_id");
            if !profiles.contains(&id) { profiles.push(id); }
        }
        Ok((profiles, groups))
    }
}

/// Who a value is resolved for. Empty strings mean "not known".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SettingSubject<'a> {
    /// The canonical (lower-case) domain.
    pub domain: &'a str,
    /// The account id (`accounts.id`): the per-user scope id of most keys.
    pub account_id: &'a str,
    /// The canonical address: the per-user scope id of `limit.*`/`quota.*`.
    pub address: &'a str,
}

/// Owned policy identity after resolving aliases; upstream usernames and account
/// identifiers are never rewritten in persisted authentication state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSettingSubject {
    pub domain: String,
    pub account_id: String,
    pub address: String,
}

impl CanonicalSettingSubject {
    pub fn as_subject(&self) -> SettingSubject<'_> {
        SettingSubject {
            domain: &self.domain,
            account_id: &self.account_id,
            address: &self.address,
        }
    }
}

/// Where an effective value came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingSource {
    /// The built-in default.
    Default,
    /// An environment variable used as the fallback (nothing stored).
    EnvFallback,
    /// An environment variable that pins the value.
    EnvPin,
    Global,
    Domain,
    Profile,
    Group,
    AdminUser,
    User,
}

impl SettingSource {
    /// The wire name (`GET /api/settings` `source`).
    pub fn as_str(self) -> &'static str {
        match self {
            SettingSource::Default => "default",
            SettingSource::EnvFallback => "envFallback",
            SettingSource::EnvPin => "envPin",
            SettingSource::Global => "global",
            SettingSource::Domain => "domain",
            SettingSource::Profile => "profile",
            SettingSource::Group => "group",
            SettingSource::AdminUser => "admin_user",
            SettingSource::User => "user",
        }
    }

    fn of(scope: SettingScope) -> Self {
        match scope {
            SettingScope::Domain => SettingSource::Domain,
            SettingScope::Profile => SettingSource::Profile,
            SettingScope::Group => SettingSource::Group,
            SettingScope::AdminUser => SettingSource::AdminUser,
            SettingScope::User => SettingSource::User,
        }
    }
}

/// The inputs of one key's resolution.
#[derive(Debug, Clone, Default)]
pub struct ResolveInput<'a> {
    /// The floor: the built-in default, already replaced by an environment
    /// fallback when one is set (`floor_is_env`).
    pub floor: &'a str,
    pub floor_is_env: bool,
    /// An environment pin's value: wins over everything.
    pub pin: Option<&'a str>,
    /// The policy row, if one is stored.
    pub policy: Option<&'a SettingPolicyRow>,
    /// The subject's value rows for this key, any scope, any order. Several
    /// profile or group rows may be present; the first one in the slice wins
    /// within its scope, so pass them highest priority first.
    pub values: &'a [&'a SettingValueRow],
}

/// The result of one key's resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    /// The effective value (JSON text).
    pub value: String,
    pub source: SettingSource,
    /// The scope whose lock decides the value, if a lock does (`Global` for the
    /// global lock, `EnvPin` for a pin).
    pub locked_by: Option<SettingSource>,
    /// Whether the user may store their own value: no pin, no lock at any scope,
    /// and the policy allows user overrides.
    pub user_may_override: bool,
    /// The user's own stored value, even when it does not apply.
    pub user_value: Option<String>,
}

/// Resolve one key (L6). See the module docs for the order.
pub fn resolve(input: &ResolveInput<'_>) -> Resolved {
    let user_value = input
        .values
        .iter()
        .find(|v| v.scope == SettingScope::User)
        .map(|v| v.value.clone());
    let allow_user = input.policy.is_none_or(|p| p.allow_user_override);

    if let Some(pin) = input.pin {
        return Resolved {
            value: pin.to_string(),
            source: SettingSource::EnvPin,
            locked_by: Some(SettingSource::EnvPin),
            user_may_override: false,
            user_value,
        };
    }

    // The global layer: a stored value, or the floor when `NULL`/absent.
    let (global_value, global_source) = match input.policy.and_then(|p| p.global_value.as_deref()) {
        Some(v) => (v.to_string(), SettingSource::Global),
        None if input.floor_is_env => (input.floor.to_string(), SettingSource::EnvFallback),
        None => (input.floor.to_string(), SettingSource::Default),
    };
    if input.policy.is_some_and(|p| p.global_locked) {
        return Resolved {
            value: global_value,
            source: global_source,
            locked_by: Some(SettingSource::Global),
            user_may_override: false,
            user_value,
        };
    }

    let mut value = global_value;
    let mut source = global_source;
    for scope in SettingScope::ALL {
        if scope == SettingScope::User && !allow_user {
            continue;
        }
        let Some(row) = input.values.iter().find(|v| v.scope == scope) else {
            continue;
        };
        value = row.value.clone();
        source = SettingSource::of(scope);
        if row.locked && scope != SettingScope::User {
            return Resolved {
                value,
                source,
                locked_by: Some(SettingSource::of(scope)),
                user_may_override: false,
                user_value,
            };
        }
    }
    Resolved {
        value,
        source,
        locked_by: None,
        user_may_override: allow_user,
        user_value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    fn policy(global: Option<&str>, allow_user: bool, locked: bool) -> SettingPolicyRow {
        SettingPolicyRow {
            key: "k".into(),
            global_value: global.map(str::to_string),
            allow_user_override: allow_user,
            global_locked: locked,
            updated_by: "root".into(),
            updated_at: 1,
        }
    }

    fn val(scope: SettingScope, value: &str, locked: bool) -> SettingValueRow {
        SettingValueRow {
            scope,
            scope_id: "id".into(),
            key: "k".into(),
            value: value.into(),
            locked,
            updated_by: "root".into(),
            updated_at: 1,
        }
    }

    fn run(
        floor: &str,
        pin: Option<&str>,
        p: Option<&SettingPolicyRow>,
        values: &[SettingValueRow],
    ) -> Resolved {
        let refs: Vec<&SettingValueRow> = values.iter().collect();
        resolve(&ResolveInput {
            floor,
            floor_is_env: false,
            pin,
            policy: p,
            values: &refs,
        })
    }

    #[test]
    fn nothing_stored_is_the_default() {
        let r = run("\"dark\"", None, None, &[]);
        assert_eq!(r.value, "\"dark\"");
        assert_eq!(r.source, SettingSource::Default);
        assert!(r.user_may_override);
        assert_eq!(r.locked_by, None);
    }

    #[test]
    fn env_fallback_replaces_the_default_but_not_a_stored_value() {
        let r = resolve(&ResolveInput {
            floor: "\"env\"",
            floor_is_env: true,
            ..Default::default()
        });
        assert_eq!(r.source, SettingSource::EnvFallback);
        let p = policy(Some("\"stored\""), true, false);
        let r = resolve(&ResolveInput {
            floor: "\"env\"",
            floor_is_env: true,
            policy: Some(&p),
            ..Default::default()
        });
        assert_eq!(
            (r.value.as_str(), r.source),
            ("\"stored\"", SettingSource::Global)
        );
    }

    #[test]
    fn narrowest_value_wins_in_order() {
        let p = policy(Some("1"), true, false);
        // Stored narrowest first; the order in the slice does not matter.
        let all = [
            val(SettingScope::User, "6", false),
            val(SettingScope::AdminUser, "5", false),
            val(SettingScope::Group, "4", false),
            val(SettingScope::Profile, "3", false),
            val(SettingScope::Domain, "2", false),
        ];
        for (from, want) in [(0, "6"), (1, "5"), (2, "4"), (3, "3"), (4, "2"), (5, "1")] {
            assert_eq!(run("0", None, Some(&p), &all[from..]).value, want, "{from}");
        }
        let mut reversed = all.clone();
        reversed.reverse();
        assert_eq!(run("0", None, Some(&p), &reversed).value, "6");
    }

    #[test]
    fn user_value_ignored_without_override_but_reported() {
        let p = policy(Some("1"), false, false);
        let r = run("0", None, Some(&p), &[val(SettingScope::User, "9", false)]);
        assert_eq!(r.value, "1");
        assert!(!r.user_may_override);
        assert_eq!(r.user_value.as_deref(), Some("9"));
    }

    #[test]
    fn global_lock_beats_every_narrower_scope() {
        let p = policy(Some("1"), true, true);
        let r = run(
            "0",
            None,
            Some(&p),
            &[
                val(SettingScope::Domain, "2", true),
                val(SettingScope::User, "6", false),
            ],
        );
        assert_eq!(r.value, "1");
        assert_eq!(r.locked_by, Some(SettingSource::Global));
        assert!(!r.user_may_override);
    }

    #[test]
    fn global_lock_without_value_locks_the_default() {
        let p = policy(None, true, true);
        let r = run(
            "\"dark\"",
            None,
            Some(&p),
            &[val(SettingScope::User, "\"x\"", false)],
        );
        assert_eq!(r.value, "\"dark\"");
        assert_eq!(r.source, SettingSource::Default);
        assert_eq!(r.locked_by, Some(SettingSource::Global));
    }

    #[test]
    fn domain_lock_beats_narrower_scopes_only() {
        let p = policy(Some("1"), true, false);
        let r = run(
            "0",
            None,
            Some(&p),
            &[
                val(SettingScope::Domain, "2", true),
                val(SettingScope::AdminUser, "5", false),
                val(SettingScope::User, "6", false),
            ],
        );
        assert_eq!(r.value, "2");
        assert_eq!(r.locked_by, Some(SettingSource::Domain));
        assert!(!r.user_may_override);
    }

    #[test]
    fn pin_wins_over_everything() {
        let p = policy(Some("1"), true, true);
        let r = run(
            "0",
            Some("7"),
            Some(&p),
            &[val(SettingScope::Domain, "2", true)],
        );
        assert_eq!(r.value, "7");
        assert_eq!(r.source, SettingSource::EnvPin);
        assert!(!r.user_may_override);
    }

    fn pg_dsn() -> Option<String> {
        std::env::var("DATABASE_URL_PG")
            .ok()
            .or_else(|| std::env::var("MW_TEST_PG").ok())
            .filter(|s| !s.trim().is_empty())
    }

    /// The Postgres legs of the store tests, env-gated like the rest of the crate
    /// (`MW_REQUIRE_LIVE` turns a skip into a failure).
    #[tokio::test]
    async fn postgres_legs() {
        let Some(dsn) = pg_dsn() else {
            mw_test_gate::skip(
                "[mw-store] settings_policy Postgres legs (set DATABASE_URL_PG or MW_TEST_PG                  to a live postgres:16 to run them). The SQLite path still asserted.",
            );
            return;
        };
        let s = Store::open_postgres(&dsn, ServerKey::generate())
            .await
            .expect("DATABASE_URL_PG is set but Postgres is not reachable");
        // A shared database: start from empty tables for this module's rows.
        crate::backend::q("DELETE FROM setting_value")
            .execute(&s.backend)
            .await
            .unwrap();
        crate::backend::q("DELETE FROM setting_policy")
            .execute(&s.backend)
            .await
            .unwrap();
        round_trip(&s).await;
        paging(&s).await;
        crate::backend::q("DELETE FROM setting_value")
            .execute(&s.backend)
            .await
            .unwrap();
        subject_rows(&s).await;
    }

    #[tokio::test]
    async fn policy_and_value_rows_round_trip() {
        round_trip(&store().await).await;
    }

    async fn round_trip(s: &Store) {
        assert_eq!(s.get_setting_policy("k").await.unwrap(), None);
        let mut p = policy(Some("\"a\""), false, true);
        s.put_setting_policy(&p).await.unwrap();
        assert_eq!(s.get_setting_policy("k").await.unwrap().as_ref(), Some(&p));
        p.global_value = None;
        p.updated_at = 2;
        s.put_setting_policy(&p).await.unwrap();
        assert_eq!(s.list_setting_policies().await.unwrap(), vec![p]);
        assert!(s.delete_setting_policy("k").await.unwrap());
        assert!(!s.delete_setting_policy("k").await.unwrap());
        assert_eq!(s.get_setting_policy("k").await.unwrap(), None);

        let mut v = val(SettingScope::Domain, "\"d\"", true);
        v.scope_id = "example.org".into();
        s.put_setting_value(&v).await.unwrap();
        let got = s
            .get_setting_value(SettingScope::Domain, "example.org", "k")
            .await
            .unwrap();
        assert_eq!(got.as_ref(), Some(&v));
        v.value = "\"e\"".into();
        s.put_setting_value(&v).await.unwrap();
        assert_eq!(
            s.list_setting_values("k", SettingScope::Domain, None, 10)
                .await
                .unwrap(),
            vec![v.clone()]
        );
        assert!(
            s.delete_setting_value(SettingScope::Domain, "example.org", "k")
                .await
                .unwrap()
        );
        assert!(
            !s.delete_setting_value(SettingScope::Domain, "example.org", "k")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn values_page_by_cursor_and_count_per_scope() {
        paging(&store().await).await;
    }

    async fn paging(s: &Store) {
        for id in ["a", "b", "c"] {
            let mut v = val(SettingScope::User, "1", false);
            v.scope_id = id.into();
            s.put_setting_value(&v).await.unwrap();
        }
        let first = s
            .list_setting_values("k", SettingScope::User, None, 2)
            .await
            .unwrap();
        assert_eq!(
            first
                .iter()
                .map(|v| v.scope_id.as_str())
                .collect::<Vec<_>>(),
            ["a", "b"]
        );
        let rest = s
            .list_setting_values("k", SettingScope::User, Some("b"), 2)
            .await
            .unwrap();
        assert_eq!(rest.len(), 1);
        assert_eq!(
            s.count_setting_values().await.unwrap(),
            vec![SettingValueCount {
                key: "k".into(),
                scope: SettingScope::User,
                count: 3
            }]
        );
        assert_eq!(
            s.delete_setting_values_for(SettingScope::User, "a")
                .await
                .unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn subject_rows_cover_domain_and_both_user_ids_only() {
        subject_rows(&store().await).await;
    }

    #[tokio::test]
    async fn held_setting_answers_only_when_the_user_may_not_choose() {
        let s = store().await;
        let subject = SettingSubject {
            domain: "example.org",
            account_id: "acct-1",
            address: "alice@example.org",
        };
        let key = "mail.readReceipts";
        let held = |s: &Store| {
            let s = s.clone();
            async move { s.held_setting(key, "\"ask\"", &subject).await.unwrap() }
        };
        assert_eq!(held(&s).await, None, "nothing stored: the user chooses");

        let mut p = policy(Some("\"always\""), true, false);
        p.key = key.into();
        s.put_setting_policy(&p).await.unwrap();
        assert_eq!(held(&s).await, None, "a global value without a lock");

        let mut v = val(SettingScope::Domain, "\"never\"", true);
        v.scope_id = "example.org".into();
        v.key = key.into();
        s.put_setting_value(&v).await.unwrap();
        assert_eq!(
            held(&s).await.as_deref(),
            Some("\"never\""),
            "a domain lock"
        );

        p.global_locked = true;
        p.global_value = None;
        s.put_setting_policy(&p).await.unwrap();
        assert_eq!(
            held(&s).await.as_deref(),
            Some("\"ask\""),
            "a global lock without a value holds the default"
        );

        p.global_locked = false;
        p.allow_user_override = false;
        s.put_setting_policy(&p).await.unwrap();
        s.delete_setting_value(SettingScope::Domain, "example.org", key)
            .await
            .unwrap();
        assert_eq!(
            held(&s).await.as_deref(),
            Some("\"ask\""),
            "overrides switched off: the admin layers decide"
        );
    }

    async fn subject_rows(s: &Store) {
        let rows = [
            (SettingScope::Domain, "example.org"),
            (SettingScope::Domain, "other.org"),
            (SettingScope::User, "acct-1"),
            (SettingScope::AdminUser, "alice@example.org"),
            (SettingScope::User, "acct-2"),
        ];
        for (scope, id) in rows {
            let mut v = val(scope, "1", false);
            v.scope_id = id.into();
            s.put_setting_value(&v).await.unwrap();
        }
        let got = s
            .setting_values_for_subject(&SettingSubject {
                domain: "example.org",
                account_id: "acct-1",
                address: "alice@example.org",
            })
            .await
            .unwrap();
        let mut ids: Vec<&str> = got.iter().map(|v| v.scope_id.as_str()).collect();
        ids.sort_unstable();
        assert_eq!(ids, ["acct-1", "alice@example.org", "example.org"]);

        // An unknown id never matches a row stored under the empty string.
        let none = s
            .setting_values_for_subject(&SettingSubject::default())
            .await
            .unwrap();
        assert!(none.is_empty());
    }
    #[tokio::test]
    async fn aliases_inherit_canonical_locks_addresses_and_domain_family_without_remapping_bare_users()
     {
        let s = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        s.put_domain_config(&crate::DomainConfigRow {
            name: "canonical.test".into(),
            upstream_json: "{}".into(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: true,
            settings_json: "{}".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
        s.put_domain_alias("alias.test", "canonical.test")
            .await
            .unwrap();
        s.set_preferred_domain(Some("canonical.test"))
            .await
            .unwrap();
        let row = |scope, id: &str, key: &str, value: &str| SettingValueRow {
            scope,
            scope_id: id.into(),
            key: key.into(),
            value: value.into(),
            locked: true,
            updated_by: "admin".into(),
            updated_at: 1,
        };
        s.put_setting_value(&row(
            SettingScope::Domain,
            "ALIAS.TEST",
            "privacy.remoteContent",
            "\"block\"",
        ))
        .await
        .unwrap();
        let stored = s
            .get_setting_value(
                SettingScope::Domain,
                "canonical.test",
                "privacy.remoteContent",
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            stored.scope_id, "canonical.test",
            "writes through alias target the canonical row"
        );
        s.put_setting_value(&row(
            SettingScope::AdminUser,
            "user@canonical.test",
            "quota.mailbox",
            "42",
        ))
        .await
        .unwrap();
        let subject = SettingSubject {
            domain: "ALIAS.TEST",
            account_id: "Opaque-Account-ID",
            address: "User@Alias.Test",
        };
        let canonical = s.canonical_setting_subject(&subject).await.unwrap();
        assert_eq!(canonical.domain, "canonical.test");
        assert_eq!(canonical.address, "user@canonical.test");
        assert_eq!(canonical.account_id, "Opaque-Account-ID");
        assert_eq!(
            s.setting_values_for_subject(&subject).await.unwrap().len(),
            2
        );
        assert_eq!(
            s.held_setting("privacy.remoteContent", "\"ask\"", &subject)
                .await
                .unwrap(),
            Some("\"block\"".into())
        );
        assert_eq!(
            s.policy_domain_family("alias.test").await.unwrap(),
            vec!["alias.test", "canonical.test"]
        );
        let bare = SettingSubject {
            domain: "",
            account_id: "Opaque-Account-ID",
            address: "User",
        };
        let bare = s.canonical_setting_subject(&bare).await.unwrap();
        assert!(bare.domain.is_empty());
        assert_eq!(bare.address, "user");
        assert!(
            s.held_setting("privacy.remoteContent", "\"ask\"", &bare.as_subject())
                .await
                .unwrap()
                .is_none()
        );
        assert!(s.policy_domain_family("").await.unwrap().is_empty());
    }
    #[tokio::test]
    async fn unicode_aliases_use_idna_policy_rows_and_invalid_domains_fail_closed() {
        let s = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        s.put_domain_config(&crate::DomainConfigRow {
            name: "canonical.test".into(),
            upstream_json: "{}".into(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
        s.put_domain_alias("xn--bcher-kva.test", "canonical.test")
            .await
            .unwrap();
        assert_eq!(
            normalize_policy_domain(" BÜCHER.test. ").unwrap(),
            "xn--bcher-kva.test"
        );
        assert_eq!(
            s.canonical_policy_domain("bücher.test").await.unwrap(),
            "canonical.test"
        );
        assert_eq!(
            s.canonical_policy_address("USER@bücher.test")
                .await
                .unwrap(),
            "user@canonical.test"
        );
        assert_eq!(
            s.canonical_policy_address("BÜCHER").await.unwrap(),
            "bücher"
        );
        for bad in [
            "127.1",
            "127.0.0.1",
            "[::1]",
            "a/b",
            "a@b",
            "a:443",
            "a%2eb",
            "-a.test",
            "a..test",
        ] {
            assert!(normalize_policy_domain(bad).is_err(), "{bad}");
        }
    }
}
