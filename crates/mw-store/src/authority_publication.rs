//! Transactionally dirtied, versioned authority DATA. No allow or Session cache.
//! Claims are rebuild/notification hints, not proof that every replica installed
//! RAM state. Requests need a NEW active-parent fence at their late boundary.
use crate::{
    Store, StoreError,
    backend::{Backend, Dialect, Row},
    q,
};
use std::{collections::BTreeMap, fmt, path::PathBuf, sync::OnceLock};

const MAX_SCOPES: usize = 32;
const MAX_INPUT_BYTES: usize = 64 * 1024;
const MAX_LEASE: i64 = 60;
const MAX_CLAIMS: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AuthorityKind {
    Policy,
    Routing,
    Directory,
    Protection,
    Domain,
    Account,
    Principal,
    Session,
    Watch,
    Subject,
}
impl AuthorityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Policy => "policy",
            Self::Routing => "routing",
            Self::Directory => "directory",
            Self::Protection => "protection",
            Self::Domain => "domain",
            Self::Account => "account",
            Self::Principal => "principal",
            Self::Session => "session",
            Self::Watch => "watch",
            Self::Subject => "subject",
        }
    }
    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "policy" => Self::Policy,
            "routing" => Self::Routing,
            "directory" => Self::Directory,
            "protection" => Self::Protection,
            "domain" => Self::Domain,
            "account" => Self::Account,
            "principal" => Self::Principal,
            "session" => Self::Session,
            "watch" => Self::Watch,
            "subject" => Self::Subject,
            _ => return None,
        })
    }
    fn global(self) -> bool {
        matches!(
            self,
            Self::Policy | Self::Routing | Self::Directory | Self::Protection
        )
    }
}
impl fmt::Debug for AuthorityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthorityScope {
    kind: AuthorityKind,
    id: String,
}
impl AuthorityScope {
    pub fn new(kind: AuthorityKind, id: &str) -> Result<Self, StoreError> {
        if id.len() > 4096
            || !id.bytes().all(|byte| (b' '..=b'~').contains(&byte))
            || kind.global() != id.is_empty()
        {
            return Err(invalid("unsupported authority scope"));
        }
        Ok(Self {
            kind,
            id: id.into(),
        })
    }
    pub fn kind(&self) -> AuthorityKind {
        self.kind
    }
    pub fn id(&self) -> &str {
        &self.id
    }
}
impl fmt::Debug for AuthorityScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthorityScope")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct AuthorityStamp {
    scope: AuthorityScope,
    incarnation: String,
    version: i64,
}
impl AuthorityStamp {
    pub fn scope(&self) -> &AuthorityScope {
        &self.scope
    }
    pub fn version(&self) -> i64 {
        self.version
    }
    pub fn same_source(&self, other: &Self) -> bool {
        self == other
    }
}
impl fmt::Debug for AuthorityStamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthorityStamp")
            .field("kind", &self.scope.kind)
            .field("version", &self.version)
            .finish_non_exhaustive()
    }
}
#[derive(Clone)]
pub struct AuthorityVersions {
    store: uuid::Uuid,
    generation: Option<PathBuf>,
    stamps: BTreeMap<AuthorityScope, AuthorityStamp>,
}
impl AuthorityVersions {
    pub fn stamps(&self) -> impl Iterator<Item = &AuthorityStamp> {
        self.stamps.values()
    }
    pub fn matches(&self, other: &Self) -> Result<bool, StoreError> {
        if self.store != other.store {
            return Err(corrupt("authority observations belong to different Stores"));
        }
        Ok(self.generation == other.generation && self.stamps == other.stamps)
    }
}
impl fmt::Debug for AuthorityVersions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthorityVersions")
            .field("scopes", &self.stamps.len())
            .finish_non_exhaustive()
    }
}
pub struct AuthorityClaim {
    store: uuid::Uuid,
    generation: Option<PathBuf>,
    stamp: AuthorityStamp,
    token: String,
}
impl AuthorityClaim {
    pub fn stamp(&self) -> &AuthorityStamp {
        &self.stamp
    }
}
impl fmt::Debug for AuthorityClaim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthorityClaim")
            .field("stamp", &self.stamp)
            .finish_non_exhaustive()
    }
}
fn invalid(message: &'static str) -> StoreError {
    StoreError::Invalid(message)
}
fn corrupt(message: &'static str) -> StoreError {
    StoreError::Corrupt(message.into())
}
fn text(row: &Row, column: &str) -> Result<String, StoreError> {
    row.try_get_opt_string(column)?
        .ok_or_else(|| corrupt("authority field is missing"))
}
fn number(row: &Row, column: &str) -> Result<Option<i64>, StoreError> {
    use sqlx::Row as _;
    Ok(match row {
        Row::Sqlite(row) => row.try_get(column)?,
        Row::Postgres(row) => row.try_get(column)?,
    })
}
fn stamp(row: &Row) -> Result<AuthorityStamp, StoreError> {
    let kind = AuthorityKind::parse(&text(row, "scope_kind")?)
        .ok_or_else(|| corrupt("unknown authority scope"))?;
    let scope = AuthorityScope::new(kind, &text(row, "scope_id")?)?;
    let incarnation = text(row, "incarnation")?;
    let version = number(row, "version")?
        .filter(|version| *version > 0)
        .ok_or_else(|| corrupt("invalid authority version"))?;
    if incarnation.len() != 32 || !incarnation.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(corrupt("invalid authority incarnation"));
    }
    Ok(AuthorityStamp {
        scope,
        incarnation,
        version,
    })
}
fn inputs(scopes: &[AuthorityScope]) -> Result<String, StoreError> {
    if scopes.len() > MAX_SCOPES
        || scopes
            .iter()
            .map(|scope| scope.id.len() + 64)
            .sum::<usize>()
            > MAX_INPUT_BYTES
    {
        return Err(invalid("authority scope batch exceeds bound"));
    }
    let value = scopes
        .iter()
        .map(|scope| serde_json::json!({"scope_kind":scope.kind.as_str(),"scope_id":scope.id}))
        .collect::<Vec<_>>();
    let encoded =
        serde_json::to_string(&value).map_err(|_| corrupt("authority scope encoding failed"))?;
    if encoded.len() > MAX_INPUT_BYTES {
        return Err(invalid("authority encoded scope batch exceeds bound"));
    }
    Ok(encoded)
}
fn requested(dialect: Dialect) -> &'static str {
    match dialect {
        Dialect::Sqlite => {
            "requested AS (SELECT json_extract(value,'$.scope_kind') AS scope_kind,json_extract(value,'$.scope_id') AS scope_id FROM json_each(?1))"
        }
        Dialect::Postgres => {
            "requested AS (SELECT value->>'scope_kind' AS scope_kind,value->>'scope_id' AS scope_id FROM json_array_elements(?1::json))"
        }
    }
}
const EXISTS: &str = "((r.scope_kind IN ('policy','routing','directory','protection') AND r.scope_id='') OR (r.scope_kind='domain' AND EXISTS(SELECT 1 FROM domains x WHERE x.name=r.scope_id)) OR (r.scope_kind='account' AND EXISTS(SELECT 1 FROM accounts x WHERE x.id=r.scope_id)) OR (r.scope_kind='principal' AND EXISTS(SELECT 1 FROM identity_principal x WHERE x.id=r.scope_id)) OR (r.scope_kind='session' AND EXISTS(SELECT 1 FROM sessions x WHERE x.id=r.scope_id)) OR (r.scope_kind='subject' AND (EXISTS(SELECT 1 FROM accounts x WHERE x.id=r.scope_id) OR EXISTS(SELECT 1 FROM setting_value x WHERE x.scope_kind IN ('user','admin_user') AND x.scope_id=r.scope_id) OR EXISTS(SELECT 1 FROM personal_setting_value x WHERE x.owner_subject=r.scope_id) OR EXISTS(SELECT 1 FROM user_group_member x WHERE x.account_id=r.scope_id) OR EXISTS(SELECT 1 FROM feature_profile_assignment x WHERE x.scope_kind='user' AND x.scope_id=r.scope_id))) OR (r.scope_kind='watch' AND EXISTS(SELECT 1 FROM settings x WHERE x.key='sess-sec:v1:'||r.scope_id OR x.key='sess-rot:v1:'||r.scope_id)))";
fn query(kind: usize, dialect: Dialect) -> &'static str {
    static SQL: OnceLock<[[String; 2]; 3]> = OnceLock::new();
    let all=SQL.get_or_init(||std::array::from_fn(|kind|std::array::from_fn(|backend|{
        let dialect=if backend==0{Dialect::Sqlite}else{Dialect::Postgres};let input=requested(dialect);
        match kind {
            0=>format!("SELECT e.scope_kind,e.scope_id,e.incarnation,e.version FROM (WITH {input} SELECT DISTINCT r.scope_kind,r.scope_id FROM requested r) r JOIN authority_epoch e ON e.scope_kind=r.scope_kind AND e.scope_id=r.scope_id ORDER BY e.scope_kind,e.scope_id"),
            1=>format!("WITH {input} INSERT INTO authority_epoch(scope_kind,scope_id) SELECT DISTINCT r.scope_kind,r.scope_id FROM requested r WHERE {EXISTS} ON CONFLICT(scope_kind,scope_id) DO NOTHING"),
            _=>format!("WITH {input} INSERT INTO authority_publication_outbox(scope_kind,scope_id,incarnation,target_version) SELECT e.scope_kind,e.scope_id,e.incarnation,e.version FROM requested r JOIN authority_epoch e ON e.scope_kind=r.scope_kind AND e.scope_id=r.scope_id WHERE {EXISTS} ON CONFLICT(scope_kind,scope_id) DO NOTHING"),
        }
    })));
    &all[kind][usize::from(dialect == Dialect::Postgres)]
}
impl Store {
    async fn authority_generation(&self) -> Option<PathBuf> {
        match &self.backend {
            Backend::Sqlite(live) => Some(
                live.pin()
                    .await
                    .active_options()
                    .get_filename()
                    .to_path_buf(),
            ),
            Backend::Postgres(_) => None,
        }
    }
    /// Background reconciliation data ONLY. Arbitrary scope lookup is not a
    /// current credential/parent proof; request admission uses native fence.
    pub async fn authority_versions(
        &self,
        scopes: &[AuthorityScope],
    ) -> Result<AuthorityVersions, StoreError> {
        let input = inputs(scopes)?;
        let generation = self.authority_generation().await;
        let rows = q(query(0, self.backend.dialect()))
            .bind(input)
            .fetch_all(&self.backend)
            .await?;
        let mut stamps = BTreeMap::new();
        for row in rows {
            let stamp = stamp(&row)?;
            stamps.insert(stamp.scope.clone(), stamp);
        }
        if generation != self.authority_generation().await {
            return Err(StoreError::SourceChanged);
        }
        Ok(AuthorityVersions {
            store: self.cache_scope(),
            generation,
            stamps,
        })
    }
    /// Cold/background action, never a write on the warm admission path. Existing
    /// or concurrently dirtied epochs keep their incarnation and version.
    pub async fn seed_authority_scopes(&self, scopes: &[AuthorityScope]) -> Result<(), StoreError> {
        let input = inputs(scopes)?;
        let mut tx = self.backend.begin().await?;
        q(query(1, self.backend.dialect()))
            .bind(&input)
            .execute_tx(&mut tx)
            .await?;
        q(query(2, self.backend.dialect()))
            .bind(&input)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn claim_authority_publications(
        &self,
        limit: usize,
        now: i64,
        lease_seconds: i64,
    ) -> Result<Vec<AuthorityClaim>, StoreError> {
        if limit == 0 || limit > MAX_CLAIMS || !(1..=MAX_LEASE).contains(&lease_seconds) {
            return Err(invalid("authority claim bound exceeded"));
        }
        let until = now
            .checked_add(lease_seconds)
            .ok_or_else(|| invalid("authority claim clock overflow"))?;
        let maximum = now
            .checked_add(MAX_LEASE)
            .ok_or_else(|| invalid("authority claim clock overflow"))?;
        let mut tx = self.backend.begin().await?;
        let generation = tx.generation_path();
        let sql = match self.backend.dialect() {
            Dialect::Sqlite => {
                "SELECT scope_kind,scope_id,incarnation,target_version AS version FROM authority_publication_outbox WHERE notified_version<target_version AND (claim_token IS NULL OR claim_until IS NULL OR claim_until<=?1 OR claim_until>?2) ORDER BY scope_kind,scope_id LIMIT ?3"
            }
            Dialect::Postgres => {
                "SELECT scope_kind,scope_id,incarnation,target_version AS version FROM authority_publication_outbox WHERE notified_version<target_version AND (claim_token IS NULL OR claim_until IS NULL OR claim_until<=?1 OR claim_until>?2) ORDER BY scope_kind,scope_id LIMIT ?3 FOR UPDATE SKIP LOCKED"
            }
        };
        let rows = q(sql)
            .bind(now)
            .bind(maximum)
            .bind(limit as i64)
            .fetch_all_tx(&mut tx)
            .await?;
        let mut claims = Vec::with_capacity(rows.len());
        for row in rows {
            let stamp = stamp(&row)?;
            let token = crate::seal::random_token();
            let updated=q("UPDATE authority_publication_outbox SET claim_token=?1,claim_until=?2 WHERE scope_kind=?3 AND scope_id=?4 AND incarnation=?5 AND target_version=?6 AND notified_version<target_version AND (claim_token IS NULL OR claim_until IS NULL OR claim_until<=?7 OR claim_until>?8)")
                .bind(&token).bind(until).bind(stamp.scope.kind.as_str()).bind(stamp.scope.id.as_str()).bind(stamp.incarnation.as_str()).bind(stamp.version).bind(now).bind(maximum).execute_tx(&mut tx).await?;
            if updated == 1 {
                claims.push(AuthorityClaim {
                    store: self.cache_scope(),
                    generation: generation.clone(),
                    stamp,
                    token,
                });
            }
        }
        tx.commit().await?;
        Ok(claims)
    }
    /// CAS of this exact target and incarnation. Global acknowledgement never
    /// means all replicas installed their process-local immutable components.
    pub async fn acknowledge_authority_publication(
        &self,
        claim: &AuthorityClaim,
    ) -> Result<bool, StoreError> {
        if claim.store != self.cache_scope() {
            return Err(corrupt("authority claim belongs to another Store"));
        }
        let mut tx = self.backend.begin().await?;
        if claim.generation != tx.generation_path() {
            tx.rollback().await?;
            return Ok(false);
        }
        let changed=q("UPDATE authority_publication_outbox SET notified_version=?1,claim_token=NULL,claim_until=NULL WHERE scope_kind=?2 AND scope_id=?3 AND incarnation=?4 AND target_version=?1 AND claim_token=?5")
            .bind(claim.stamp.version).bind(claim.stamp.scope.kind.as_str()).bind(claim.stamp.scope.id.as_str()).bind(claim.stamp.incarnation.as_str()).bind(claim.token.as_str()).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(changed == 1)
    }
    /// Remove only acknowledged absent-source scopes, at most128 per call.
    /// Source deletion tombstones need not live forever: a future incarnation
    /// cannot match an old in-memory stamp even if its counter starts at1.
    pub async fn cleanup_obsolete_authority_scopes(&self, limit: usize) -> Result<u64, StoreError> {
        if limit == 0 || limit > 128 {
            return Err(invalid("authority cleanup bound exceeded"));
        }
        static SELECT: OnceLock<[String; 2]> = OnceLock::new();
        static DELETE: OnceLock<String> = OnceLock::new();
        let condition = format!(
            "r.scope_kind NOT IN ('policy','routing','directory','protection') AND NOT {EXISTS} AND o.notified_version=o.target_version AND o.claim_token IS NULL AND o.incarnation=r.incarnation AND o.target_version=r.version"
        );
        let sql=&SELECT.get_or_init(||std::array::from_fn(|backend|format!("SELECT r.scope_kind,r.scope_id,r.incarnation,r.version FROM authority_epoch r JOIN authority_publication_outbox o ON o.scope_kind=r.scope_kind AND o.scope_id=r.scope_id WHERE {condition} ORDER BY r.scope_kind,r.scope_id LIMIT ?1{}",if backend==1{" FOR UPDATE OF r,o SKIP LOCKED"}else{""})))[usize::from(self.backend.dialect()==Dialect::Postgres)];
        let delete=DELETE.get_or_init(||format!("DELETE FROM authority_epoch AS r WHERE r.scope_kind=?1 AND r.scope_id=?2 AND r.incarnation=?3 AND r.version=?4 AND NOT {EXISTS} AND EXISTS(SELECT 1 FROM authority_publication_outbox o WHERE o.scope_kind=r.scope_kind AND o.scope_id=r.scope_id AND o.incarnation=r.incarnation AND o.target_version=r.version AND o.notified_version=o.target_version AND o.claim_token IS NULL)"));
        let mut tx = self.backend.begin().await?;
        let rows = q(sql).bind(limit as i64).fetch_all_tx(&mut tx).await?;
        let mut count = 0;
        for row in rows {
            let stamp = stamp(&row)?;
            count += q(delete)
                .bind(stamp.scope.kind.as_str())
                .bind(stamp.scope.id.as_str())
                .bind(stamp.incarnation)
                .bind(stamp.version)
                .execute_tx(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(count)
    }
}
#[cfg(test)]
#[path = "authority_publication_tests.rs"]
mod tests;

#[path = "authority_fence.rs"]
mod fence;
pub use fence::NativeAuthorityFence;
