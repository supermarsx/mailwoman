//! Typed cold-source absence seeding. This is DATA setup, not admission.
use super::*;
use crate::native_admission_snapshot::NativeAdmissionSnapshot;

pub struct NativeAuthoritySeed {
    expected: AuthorityVersions,
    subjects: Vec<AuthorityScope>,
    source: crate::native_admission_snapshot::NativeAuthoritySeedSource,
    schema: Option<String>,
}
impl fmt::Debug for NativeAuthoritySeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeAuthoritySeed")
            .field("subjects", &self.subjects.len())
            .finish_non_exhaustive()
    }
}
impl NativeAuthoritySeed {
    pub fn subjects(&self) -> Vec<String> {
        self.subjects.iter().map(|s| s.id.clone()).collect()
    }
}
impl Store {
    /// Mint only from an opaque Store snapshot and current-parent fence. Alias
    /// reads happen before the seed transaction, but the seed CAS includes BOTH
    /// global routing and policy plus all supplied domain/source dependencies.
    /// A change between derivation and INSERT therefore rejects rather than
    /// certifying the old canonical address as current absence.
    pub async fn prepare_native_authority_seed(
        &self,
        source: &NativeAdmissionSnapshot,
        before: &NativeAuthorityFence,
    ) -> Result<Option<NativeAuthoritySeed>, StoreError> {
        if before.versions().store != self.cache_scope() {
            return Err(invalid("foreign native seed source"));
        }
        let Some(session) = source.session(self)? else {
            return Ok(None);
        };
        let Some(principal) = source.principal()? else {
            return Ok(None);
        };
        let Some(account) = source.account()? else {
            return Ok(None);
        };
        if session.session.id != before.session_id()
            || session.session.account_id != before.account_id()
            || account.id != before.account_id()
            || principal.principal_id != before.principal_id()
            || principal.binding_id != before.binding_id()
            || source.native_subjects(self)?.is_none()
        {
            return Ok(None);
        }
        if before
            .versions()
            .stamps
            .keys()
            .any(|s| s.kind == AuthorityKind::Subject)
        {
            return Ok(None);
        }
        for global in [
            AuthorityKind::Policy,
            AuthorityKind::Routing,
            AuthorityKind::Directory,
            AuthorityKind::Protection,
        ] {
            if !before
                .versions()
                .stamps
                .contains_key(&AuthorityScope::new(global, "")?)
            {
                return Ok(None);
            }
        }
        let mut subjects = vec![AuthorityScope::new(AuthorityKind::Subject, &account.id)?];
        for raw in [
            &session.session.username,
            &session.session.credentials.username,
            &account.username,
        ] {
            if raw.len() > 1024 || !raw.is_ascii() || raw.bytes().any(|c| c < 32 || c == 127) {
                return Ok(None);
            }
            let canonical = self.publication_policy_address(raw).await?;
            if canonical.len() > 1024 || !canonical.contains('@') {
                return Ok(None);
            }
            let scope = AuthorityScope::new(AuthorityKind::Subject, &canonical)?;
            if !subjects.contains(&scope) {
                subjects.push(scope);
            }
        }
        if subjects.len() > 8 {
            return Ok(None);
        }
        let Some(source) = source.authority_seed_source(self)? else {
            return Ok(None);
        };
        Ok(Some(NativeAuthoritySeed {
            expected: before.versions().clone(),
            subjects,
            source,
            schema: before.publication_schema().map(str::to_owned),
        }))
    }
    /// True only when this exact cold-source vector still exists inside the
    /// seed transaction. No in-flight SQL transaction spans worker preparation.
    pub async fn seed_native_authority_subjects(
        &self,
        seed: &NativeAuthoritySeed,
    ) -> Result<bool, StoreError> {
        if seed.expected.store != self.cache_scope() {
            return Err(invalid("foreign native seed"));
        }
        let requested = seed.expected.stamps.keys().cloned().collect::<Vec<_>>();
        let mut tx = self.backend.begin().await?;
        if tx.generation_path() != seed.expected.generation {
            return Ok(false);
        }
        // Parent -> session -> epoch -> existing anchor lock order prevents a
        // new FK/epoch GC cycle against account/session erasure. All metadata
        // locks are confined to this small DB transaction, never a build/fsync.
        if self.backend.dialect() == Dialect::Postgres {
            let account = q("SELECT id FROM accounts WHERE id=?1 FOR KEY SHARE")
                .bind(&seed.source.account)
                .fetch_all_tx(&mut tx)
                .await?;
            if account.len() != 1 {
                return Ok(false);
            }
            let session = q("SELECT id FROM sessions WHERE id=?1 AND account_id=?2 FOR KEY SHARE")
                .bind(&seed.source.session)
                .bind(&seed.source.account)
                .fetch_all_tx(&mut tx)
                .await?;
            if session.len() != 1 {
                return Ok(false);
            }
        }
        // PG row locks serialize authority version checks against source-trigger
        // epoch updates until absence rows have been installed. SQLite obtains
        // its writer reservation BEFORE observing the expected vector.
        if self.backend.dialect() == Dialect::Sqlite {
            q("UPDATE authority_epoch SET version=version+1 WHERE scope_kind='policy' AND 1=0")
                .execute_tx(&mut tx)
                .await?;
        }
        if let Some(expected) = &seed.schema {
            let row = q(super::schema::stamp_sql(self.backend.dialect()))
                .fetch_all_tx(&mut tx)
                .await?;
            if row
                .first()
                .map(|r| r.try_get_opt_string("publication_schema"))
                .transpose()?
                .flatten()
                .as_ref()
                != Some(expected)
            {
                return Ok(false);
            }
        }
        let encoded = inputs(&requested)?;
        let sql = seed_read_sql(self.backend.dialect());
        let rows = q(sql).bind(encoded).fetch_all_tx(&mut tx).await?;
        let mut stamps = BTreeMap::new();
        for row in rows {
            let value = stamp(&row)?;
            stamps.insert(value.scope.clone(), value);
        }
        if stamps != seed.expected.stamps {
            return Ok(false);
        }
        let source = &seed.source;
        let matches=q("SELECT COUNT(*) AS n FROM sessions s JOIN accounts a ON a.id=s.account_id JOIN session_principal sp ON sp.session_id=s.id AND sp.primary_account_id=a.id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding ib ON ib.id=sp.binding_id AND ib.principal_id=p.id WHERE s.id=?1 AND a.id=?2 AND s.username=?3 AND a.username=?4 AND s.sealed_creds=?5 AND sp.principal_id=?6 AND sp.binding_id=?7 AND s.linked_internal=0 AND s.jmap_url='engine' AND s.api_url='engine' AND p.state='active' AND ib.retired_at IS NULL")
            .bind(&source.session).bind(&source.account).bind(&source.username).bind(&source.account_username).bind(&source.sealed).bind(&source.principal).bind(&source.binding).fetch_all_tx(&mut tx).await?;
        if matches.first().map(|r| r.get_i64("n")) != Some(1) {
            return Ok(false);
        }
        if self.backend.dialect() == Dialect::Postgres {
            q("SELECT subject_id FROM authority_subject_anchor WHERE session_id=?1 ORDER BY subject_id FOR UPDATE").bind(&seed.source.session).fetch_all_tx(&mut tx).await?;
        }
        let encoded = inputs(&seed.subjects)?;
        let input = requested_sql(self.backend.dialect());
        let subjects = serde_json::to_string(
            &seed
                .subjects
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
        )
        .map_err(|_| invalid("anchor encoding failed"))?;
        let account_incarnation = &seed
            .expected
            .stamps
            .get(&AuthorityScope::new(
                AuthorityKind::Account,
                &seed.source.account,
            )?)
            .ok_or_else(|| invalid("missing anchor account source"))?
            .incarnation;
        let session_incarnation = &seed
            .expected
            .stamps
            .get(&AuthorityScope::new(
                AuthorityKind::Session,
                &seed.source.session,
            )?)
            .ok_or_else(|| invalid("missing anchor session source"))?
            .incarnation;
        let sql = anchor_sql(self.backend.dialect());
        q(sql.0)
            .bind(&seed.source.session)
            .bind(&seed.source.account)
            .bind(&subjects)
            .bind(account_incarnation)
            .bind(session_incarnation)
            .execute_tx(&mut tx)
            .await?;
        q(input).bind(encoded).execute_tx(&mut tx).await?;
        q(sql.1)
            .bind(&seed.source.session)
            .bind(&seed.source.account)
            .bind(&subjects)
            .bind(account_incarnation)
            .bind(session_incarnation)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(true)
    }
}
fn seed_read_sql(dialect: Dialect) -> &'static str {
    static SQL: OnceLock<[String; 2]> = OnceLock::new();
    &SQL.get_or_init(||std::array::from_fn(|pg| {
        let d=if pg==1 {Dialect::Postgres}else{Dialect::Sqlite};
        format!("WITH {} SELECT e.scope_kind,e.scope_id,e.incarnation,e.version FROM requested r JOIN authority_epoch e ON e.scope_kind=r.scope_kind AND e.scope_id=r.scope_id ORDER BY e.scope_kind,e.scope_id{}",requested(d),if pg==1 {" FOR UPDATE OF e"}else{""})
    }))[usize::from(dialect==Dialect::Postgres)]
}
fn requested_sql(dialect: Dialect) -> &'static str {
    static SQL: OnceLock<[String; 2]> = OnceLock::new();
    &SQL.get_or_init(||std::array::from_fn(|pg| {
        let d=if pg==1 {Dialect::Postgres}else{Dialect::Sqlite};
        format!("WITH {} INSERT INTO authority_epoch(scope_kind,scope_id) SELECT r.scope_kind,r.scope_id FROM requested r WHERE r.scope_kind='subject' ON CONFLICT(scope_kind,scope_id) DO NOTHING",requested(d))
    }))[usize::from(dialect==Dialect::Postgres)]
}

impl Store {
    pub async fn cleanup_native_authority_anchors(&self, limit: usize) -> Result<u64, StoreError> {
        if limit == 0 || limit > 128 {
            return Err(invalid("anchor cleanup bound"));
        }
        q("DELETE FROM authority_subject_anchor WHERE (session_id,subject_id) IN(SELECT h.session_id,h.subject_id FROM authority_subject_anchor h LEFT JOIN accounts a ON a.id=h.account_id LEFT JOIN sessions s ON s.id=h.session_id AND s.account_id=h.account_id LEFT JOIN authority_epoch ae ON ae.scope_kind='account' AND ae.scope_id=h.account_id AND ae.incarnation=h.account_incarnation LEFT JOIN authority_epoch se ON se.scope_kind='session' AND se.scope_id=h.session_id AND se.incarnation=h.session_incarnation WHERE a.id IS NULL OR s.id IS NULL OR ae.scope_id IS NULL OR se.scope_id IS NULL ORDER BY h.session_id,h.subject_id LIMIT ?1)").bind(limit as i64).execute(&self.backend).await.map_err(Into::into)
    }
}

fn anchor_sql(dialect: Dialect) -> (&'static str, &'static str) {
    match dialect {
        Dialect::Sqlite => (
            "DELETE FROM authority_subject_anchor WHERE session_id=?1 AND (account_id<>?2 OR account_incarnation<>?4 OR session_incarnation<>?5 OR subject_id NOT IN(SELECT value FROM json_each(?3)))",
            "INSERT INTO authority_subject_anchor(session_id,account_id,subject_id,account_incarnation,session_incarnation) SELECT ?1,?2,value,?4,?5 FROM json_each(?3) WHERE 1=1 ON CONFLICT(session_id,subject_id) DO NOTHING",
        ),
        Dialect::Postgres => (
            "DELETE FROM authority_subject_anchor WHERE session_id=?1 AND (account_id<>?2 OR account_incarnation<>?4 OR session_incarnation<>?5 OR subject_id NOT IN(SELECT value FROM json_array_elements_text(?3::json)))",
            "INSERT INTO authority_subject_anchor(session_id,account_id,subject_id,account_incarnation,session_incarnation) SELECT ?1,?2,value,?4,?5 FROM json_array_elements_text(?3::json) ON CONFLICT(session_id,subject_id) DO NOTHING",
        ),
    }
}
