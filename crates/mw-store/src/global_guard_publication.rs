//! Fresh session-independent GLOBAL configuration DATA, never authentication.
//! Cold extraction validates installed definitions; hot fences use one bounded
//! statement and never seed epochs. No transaction survives either operation.
use super::*;
use crate::settings_policy::{CanonicalSettingSubject, SettingsResolutionInputs};

const SQLITE_EXTRA: &str = include_str!("../migrations/0114_guard_input_publication.sql");
const PG_EXTRA: &str = include_str!("../migrations_pg/0114_guard_input_publication.sql");
const RAW_BYTES: usize = 1024 * 1024;

#[derive(Clone)]
pub struct GlobalGuardConfigFence {
    versions: AuthorityVersions,
    schema: String,
}
impl fmt::Debug for GlobalGuardConfigFence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GlobalGuardConfigFence").finish_non_exhaustive()
    }
}
impl GlobalGuardConfigFence {
    /// Equality of immutable DATA sources, not an allow, clock or source lease.
    pub fn matches(&self, fresh: &Self) -> Result<bool, StoreError> {
        Ok(self.versions.matches(&fresh.versions)? && self.schema == fresh.schema)
    }
}

pub struct GlobalGuardConfigSource {
    fence: GlobalGuardConfigFence,
    settings: SettingsResolutionInputs,
    sections: BTreeMap<GuardInputSection, Option<String>>,
}
impl fmt::Debug for GlobalGuardConfigSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GlobalGuardConfigSource").field("sections", &self.sections.len()).finish_non_exhaustive()
    }
}
impl GlobalGuardConfigSource {
    pub fn fence(&self) -> &GlobalGuardConfigFence { &self.fence }
    /// Parse/validate at the owning guard's original semantic boundary. Values
    /// can contain secrets; no Debug/Serialize implementation exposes them.
    pub fn raw(&self, store: &Store, section: GuardInputSection) -> Result<Option<&str>, StoreError> {
        if self.fence.versions.store != store.cache_scope() { return Err(invalid("foreign global guard source")); }
        self.sections.get(&section).map(|value| value.as_deref()).ok_or_else(|| invalid("missing global guard section"))
    }
    /// Keep a cloned fence and parse raw sections before moving these inputs
    /// into the existing resolver. No settings memo/Session is certified here.
    pub fn into_settings(self) -> SettingsResolutionInputs { self.settings }
}

fn fence_sql(dialect: Dialect) -> &'static str {
    static SQL: OnceLock<[String; 2]> = OnceLock::new();
    &SQL.get_or_init(|| std::array::from_fn(|pg| {
        let dialect = if pg == 1 { Dialect::Postgres } else { Dialect::Sqlite };
        let valid_incarnation = if pg == 1 { "e.incarnation ~ '^[0-9a-f]{32}$'" } else {
            "typeof(e.incarnation)='text' AND length(e.incarnation)=32 AND e.incarnation NOT GLOB '*[^0-9a-f]*'"
        };
        let valid_version = if pg == 1 { "e.version>0" } else { "typeof(e.version)='integer' AND e.version>0" };
        format!("WITH wanted(scope_kind) AS (SELECT 'policy' UNION ALL SELECT 'routing' UNION ALL SELECT 'protection') SELECT w.scope_kind,'' AS scope_id,CASE WHEN {valid_incarnation} THEN e.incarnation ELSE CAST(NULL AS TEXT) END AS incarnation,CAST(CASE WHEN {valid_version} THEN e.version ELSE NULL END AS BIGINT) AS version,{} AS publication_schema FROM wanted w LEFT JOIN authority_epoch e ON e.scope_kind=w.scope_kind AND e.scope_id='' ORDER BY w.scope_kind LIMIT 4", super::schema::stamp_expression(dialect))
    }))[usize::from(dialect == Dialect::Postgres)]
}

fn raw_sql(dialect: Dialect) -> &'static str {
    static SQL: OnceLock<[String; 2]> = OnceLock::new();
    &SQL.get_or_init(|| std::array::from_fn(|pg| {
        let typed = if pg == 1 { "TRUE" } else { "typeof(value)='text'" };
        let valid = format!("(SELECT ok FROM budget)=1 AND (present IS NULL OR ({typed} AND value IS NOT NULL))");
        format!("WITH wanted(section_key) AS (SELECT 'security.inbound-allowlist.v1' UNION ALL SELECT 'security.clientAccess.v1' UNION ALL SELECT 'maintenance.mode.v1' UNION ALL SELECT 'v6:admin:bans'),candidate AS (SELECT w.section_key,s.key AS present,s.value FROM wanted w LEFT JOIN settings s ON s.key=w.section_key),budget AS (SELECT CASE WHEN COALESCE(SUM(COALESCE(octet_length(value),0)),0)<={RAW_BYTES} THEN 1 ELSE 0 END AS ok FROM candidate) SELECT section_key,CAST(CASE WHEN {valid} THEN 0 ELSE 1 END AS BIGINT) AS invalid,CASE WHEN {valid} THEN value ELSE CAST(NULL AS TEXT) END AS value FROM candidate ORDER BY section_key LIMIT 5")
    }))[usize::from(dialect == Dialect::Postgres)]
}

impl Store {
    // No public constructor mints an unvalidated source. This observation is
    // private so a schema stamp alone cannot certify installed definitions.
    async fn observe_global_guard_config(&self) -> Result<Option<GlobalGuardConfigFence>, StoreError> {
        let generation = self.authority_generation().await;
        let rows = q(fence_sql(self.backend.dialect())).fetch_all(&self.backend).await?;
        if generation != self.authority_generation().await { return Err(StoreError::SourceChanged); }
        if rows.len() != 3 { return Ok(None); }
        let mut schema = None;
        let mut stamps = BTreeMap::new();
        for row in rows {
            let Some(value) = row.try_get_opt_string("publication_schema")? else { return Ok(None); };
            if value.len() > 128 || schema.as_ref().is_some_and(|old| old != &value) { return Ok(None); }
            schema = Some(value);
            if row.try_get_opt_string("incarnation")?.is_none() || number(&row, "version")?.is_none() { return Ok(None); }
            let value = stamp(&row)?;
            if stamps.insert(value.scope.clone(), value).is_some() { return Ok(None); }
        }
        Ok(Some(GlobalGuardConfigFence {
            versions: AuthorityVersions { store: self.cache_scope(), generation, stamps },
            schema: schema.ok_or_else(|| corrupt("missing global guard schema"))?,
        }))
    }

    /// COLD strict GLOBAL inputs, independently usable without any account or
    /// session. Missing derived epochs remain unsupported; no seeding occurs.
    /// Definitions and cold sources are bounded before transfer. Concurrent
    /// source changes fail the before/after observation, never install old DATA.
    pub async fn global_guard_config_source(&self) -> Result<Option<GlobalGuardConfigSource>, StoreError> {
        let Some(before) = self.observe_global_guard_config().await? else { return Ok(None); };
        if !self.validate_publication_schema_source(&before.versions, &before.schema, SQLITE_EXTRA, PG_EXTRA).await? { return Ok(None); }
        let settings = self.settings_publication_inputs(&CanonicalSettingSubject {
            personal_owner: None, domain: String::new(), account_id: String::new(), address: String::new(),
        }).await?;
        let rows = q(raw_sql(self.backend.dialect())).fetch_all(&self.backend).await?;
        if rows.len() != 4 { return Ok(None); }
        let mut sections = BTreeMap::new();
        for row in rows {
            if row.get_i64("invalid") != 0 { return Ok(None); }
            let section = match text(&row, "section_key")?.as_str() {
                "security.inbound-allowlist.v1" => GuardInputSection::InboundAllowlist,
                "security.clientAccess.v1" => GuardInputSection::ClientAccess,
                "maintenance.mode.v1" => GuardInputSection::Maintenance,
                "v6:admin:bans" => GuardInputSection::SourceBans,
                _ => return Err(corrupt("unknown global guard section")),
            };
            if sections.insert(section, row.try_get_opt_string("value")?).is_some() { return Ok(None); }
        }
        #[cfg(test)] testing::wait().await;
        let Some(after) = self.global_guard_config_fence(&before).await? else { return Ok(None); };
        if !before.matches(&after)? { return Err(StoreError::SourceChanged); }
        Ok(Some(GlobalGuardConfigSource { fence: after, settings, sections }))
    }

    /// HOT one-statement observation of current GLOBAL DATA versions under a
    /// previously validated schema. It authenticates nobody and grants nothing.
    /// Compare its result with the preparation fence at the owning late seam.
    pub async fn global_guard_config_fence(&self, validated: &GlobalGuardConfigFence) -> Result<Option<GlobalGuardConfigFence>, StoreError> {
        if validated.versions.store != self.cache_scope() { return Err(invalid("foreign global guard fence")); }
        let Some(fresh) = self.observe_global_guard_config().await? else { return Ok(None); };
        if fresh.schema != validated.schema || fresh.versions.generation != validated.versions.generation { return Ok(None); }
        Ok(Some(fresh))
    }
}

#[cfg(test)]
mod testing {
    use std::sync::Arc;
    pub struct Pause { pub entered: tokio::sync::Notify, pub release: tokio::sync::Notify }
    tokio::task_local! { pub static PAUSE: Arc<Pause>; }
    pub async fn wait() { if let Ok(pause) = PAUSE.try_with(Clone::clone) { pause.entered.notify_one(); pause.release.notified().await; } }
}
#[cfg(test)]
#[path = "global_guard_publication_tests.rs"]
mod tests;
