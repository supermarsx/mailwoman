//! Cold guard INPUT publication prerequisites, never authorization or routing.
//! No transaction/connection survives extraction. Independent writers require
//! fresh SQL fences; outbox delivery is not a freshness or ownership proof.
use super::*;

const SQLITE_COVERAGE: &str = include_str!("../migrations/0114_guard_input_publication.sql");
const PG_COVERAGE: &str = include_str!("../migrations_pg/0114_guard_input_publication.sql");
const MAX_GUARD_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum GuardInputSection {
    InboundAllowlist,
    ClientAccess,
    Maintenance,
    SourceBans,
}
impl GuardInputSection {
    fn from_key(key: &str) -> Option<Self> {
        Some(match key {
            "security.inbound-allowlist.v1" => Self::InboundAllowlist,
            "security.clientAccess.v1" => Self::ClientAccess,
            "maintenance.mode.v1" => Self::Maintenance,
            "v6:admin:bans" => Self::SourceBans,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy)]
enum SectionError {
    Unsupported,
}

/// Opaque same-Store/generation/current-parent source plus deferred exact-key
/// sections. No credentials, cached Session, source IP, or admission predicate.
pub struct NativeGuardInputSnapshot {
    fence: NativeAuthorityFence,
    sections: BTreeMap<GuardInputSection, Result<Option<String>, SectionError>>,
}
impl fmt::Debug for NativeGuardInputSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeGuardInputSnapshot")
            .field("sections", &self.sections.len())
            .finish_non_exhaustive()
    }
}
impl NativeGuardInputSnapshot {
    /// Store identity binding only, never an authority/admission decision.
    pub fn belongs_to(&self,store:&Store)->bool {self.fence.versions().store==store.cache_scope()}
    /// Logical raw section bytes only; no privacy-bearing source value escapes.
    pub fn logical_section_bytes(&self)->usize {self.sections.values().filter_map(|v|v.as_ref().ok().and_then(|v|v.as_ref())).map(String::len).sum()}
    pub fn source(&self) -> &NativeAuthorityFence {
        &self.fence
    }
    pub fn same_source(
        &self,
        store: &Store,
        current: &NativeAuthorityFence,
    ) -> Result<bool, StoreError> {
        if self.fence.versions().store != store.cache_scope() {
            return Err(invalid("foreign guard input source"));
        }
        self.fence.same_published_source(current)
    }
    /// Decode at the owning guard's semantic position; a bad later section
    /// must not become an earlier middleware's refusal or a default value.
    pub fn raw<'a>(
        &'a self,
        store: &Store,
        section: GuardInputSection,
    ) -> Result<Option<&'a str>, StoreError> {
        if self.fence.versions().store != store.cache_scope() {
            return Err(invalid("foreign guard input source"));
        }
        match self.sections.get(&section) {
            Some(Ok(value)) => Ok(value.as_deref()),
            Some(Err(_)) => Err(invalid("unsupported bounded guard input section")),
            None => Err(invalid("missing guard input section")),
        }
    }
}

impl Store {
    /// Cold-only required111/112/114 definitions check. 114 must never be
    /// integrated/deployed ahead of separately owned migration113.
    pub async fn validate_guard_input_publication_schema(
        &self,
        source: &NativeAuthorityFence,
    ) -> Result<bool, StoreError> {
        self.validate_native_publication_schema_with_extra(source, SQLITE_COVERAGE, PG_COVERAGE)
            .await
    }

    /// COLD extraction: before fence + bounded exact-source SELECT + after
    /// fence, with additional cold schema-definition queries. This is not a
    /// one-query request fast path. No transaction is held across these awaits.
    pub async fn native_guard_input_source_snapshot(
        &self,
        cookie: &str,
        domains: &[String],
        subjects: &[String],
    ) -> Result<Option<NativeGuardInputSnapshot>, StoreError> {
        let Some(before) = self
            .native_publication_fence(cookie, domains, subjects)
            .await?
        else {
            return Ok(None);
        };
        if !self
            .validate_guard_input_publication_schema(&before)
            .await?
        {
            return Ok(None);
        }
        let rows = q(guard_source_sql(self.backend.dialect()))
            .bind(cookie)
            .bind(before.account_id())
            .bind(before.principal_id())
            .bind(before.binding_id())
            .fetch_all(&self.backend)
            .await?;
        if rows.len() != 4 {
            return Ok(None);
        }
        let mut sections = BTreeMap::new();
        for row in rows {
            let section = GuardInputSection::from_key(&text(&row, "section_key")?)
                .ok_or_else(|| corrupt("unknown guard source section"))?;
            let value = if row.get_i64("invalid") == 0 {
                row.try_get_opt_string("value")
                    .map_err(|_| SectionError::Unsupported)
            } else {
                Err(SectionError::Unsupported)
            };
            if sections.insert(section, value).is_some() {
                return Err(corrupt("duplicate guard source section"));
            }
        }
        #[cfg(test)]
        testing::wait().await;
        let Some(after) = self
            .native_publication_fence(cookie, domains, subjects)
            .await?
        else {
            return Ok(None);
        };
        if !before.same_published_source(&after)? {
            return Err(StoreError::SourceChanged);
        }
        Ok(Some(NativeGuardInputSnapshot {
            fence: after,
            sections,
        }))
    }
}

fn guard_source_sql(dialect: Dialect) -> &'static str {
    static SQL: OnceLock<[String; 2]> = OnceLock::new();
    &SQL.get_or_init(|| std::array::from_fn(|pg| {
        let typed = if pg == 1 { "TRUE" } else { "typeof(value)='text'" };
        let valid = format!("(SELECT ok FROM budget)=1 AND (present IS NULL OR ({typed} AND value IS NOT NULL))");
        format!("WITH wanted(section_key) AS (SELECT 'security.inbound-allowlist.v1' UNION ALL SELECT 'security.clientAccess.v1' UNION ALL SELECT 'maintenance.mode.v1' UNION ALL SELECT 'v6:admin:bans'),owned AS (SELECT s.id FROM sessions s JOIN accounts a ON a.id=s.account_id JOIN session_principal sp ON sp.session_id=s.id AND sp.primary_account_id=a.id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding ib ON ib.id=sp.binding_id AND ib.principal_id=p.id WHERE s.id=?1 AND a.id=?2 AND p.id=?3 AND ib.id=?4 AND s.linked_internal=0 AND s.jmap_url='engine' AND s.api_url='engine' AND p.state='active' AND ib.retired_at IS NULL),candidate AS (SELECT w.section_key,s.key AS present,s.value FROM wanted w LEFT JOIN settings s ON s.key=w.section_key),budget AS (SELECT CASE WHEN COALESCE(SUM(COALESCE(octet_length(value),0)),0)<={MAX_GUARD_BYTES} THEN 1 ELSE 0 END AS ok FROM candidate) SELECT section_key,CAST(CASE WHEN {valid} THEN 0 ELSE 1 END AS BIGINT) AS invalid,CASE WHEN {valid} THEN value ELSE CAST(NULL AS TEXT) END AS value FROM candidate WHERE EXISTS(SELECT 1 FROM owned) ORDER BY section_key LIMIT 5")
    }))[usize::from(dialect == Dialect::Postgres)]
}

#[cfg(test)]
pub(super) fn guard_source_sql_for_test(dialect: Dialect) -> &'static str {
    guard_source_sql(dialect)
}

#[cfg(test)]
pub(super) mod testing {
    use std::sync::Arc;
    pub struct Pause {
        pub entered: tokio::sync::Notify,
        pub release: tokio::sync::Notify,
    }
    tokio::task_local! {pub static PAUSE:Arc<Pause>;}
    pub(super) async fn wait() {
        if let Ok(pause) = PAUSE.try_with(Clone::clone) {
            pause.entered.notify_one();
            pause.release.notified().await;
        }
    }
}
