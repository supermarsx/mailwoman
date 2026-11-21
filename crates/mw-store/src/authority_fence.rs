//! Narrow active-parent/clock fence. Domain arguments shape dependencies only;
//! current account/principal/binding references always come from session joins.
//! Supplied dependency names are not authority: a cold builder must prove complete
//! canonical-subject/domain coverage before comparing this data-only fence.
use super::*;

pub struct NativeAuthorityFence {
    versions: AuthorityVersions,
    session: String,
    account: String,
    principal: String,
    binding: String,
    created: String,
    activity: String,
    last_seen: Option<String>,
}
impl fmt::Debug for NativeAuthorityFence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeAuthorityFence")
            .field("scopes", &self.versions.stamps.len())
            .finish_non_exhaustive()
    }
}
impl NativeAuthorityFence {
    pub fn versions(&self) -> &AuthorityVersions {
        &self.versions
    }
    pub fn session_id(&self) -> &str {
        &self.session
    }
    pub fn account_id(&self) -> &str {
        &self.account
    }
    pub fn principal_id(&self) -> &str {
        &self.principal
    }
    pub fn binding_id(&self) -> &str {
        &self.binding
    }
    pub fn clocks(&self) -> (&str, &str) {
        (&self.created, &self.activity)
    }
    pub fn last_seen(&self) -> Option<&str> {
        self.last_seen.as_deref()
    }
    /// Source identity/version equality only. Expiry and all policy/process
    /// decisions still run per request using the NEW fence's current clocks.
    pub fn same_published_source(&self, other: &Self) -> Result<bool, StoreError> {
        let versions = self.versions.matches(&other.versions)?;
        Ok(self.session == other.session
            && self.account == other.account
            && self.principal == other.principal
            && self.binding == other.binding
            && versions)
    }
}
pub(super) fn fence_sql(dialect: Dialect) -> &'static str {
    static SQL: OnceLock<[String; 2]> = OnceLock::new();
    let all=SQL.get_or_init(||std::array::from_fn(|backend|{
        let pg=backend==1;
        let fields=[("session_id","s.id"),("account_id","s.account_id"),("principal_id","sp.principal_id"),("binding_id","sp.binding_id"),("created","s.created_at"),("activity","COALESCE(s.activity_at,s.created_at)"),("last_seen","s.last_seen")];
        let bounded=fields.iter().map(|(name,expression)|{
            let typed=if pg{"TRUE".to_owned()}else{format!("typeof({expression})='text'")};
            format!("CASE WHEN {typed} AND octet_length({expression})<=1024 THEN {expression} ELSE CAST(NULL AS TEXT) END AS {name}")
        }).collect::<Vec<_>>().join(",");
        let source_fields=fields.iter().map(|(name,_)|format!("b.{name}")).collect::<Vec<_>>().join(",");
        let absent_fields=fields.iter().map(|_|"CAST(NULL AS TEXT)").collect::<Vec<_>>().join(",");
        let domains=if pg{"SELECT value AS name FROM json_array_elements_text(?2::json)"}else{"SELECT value AS name FROM json_each(?2)"};
        let subjects=if pg{"SELECT value AS name FROM json_array_elements_text(?3::json)"}else{"SELECT value AS name FROM json_each(?3)"};
        format!("SELECT * FROM (WITH domains_in AS ({domains}),subjects_in AS ({subjects}),base AS (SELECT {bounded} FROM sessions s JOIN accounts a ON a.id=s.account_id JOIN session_principal sp ON sp.session_id=s.id AND sp.primary_account_id=s.account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding ib ON ib.id=sp.binding_id AND ib.principal_id=p.id WHERE s.id=?1 AND s.linked_internal=0 AND s.jmap_url='engine' AND s.api_url='engine' AND a.kind IN ('imap','pop3') AND COALESCE(a.backend_kind,'')<>'ews' AND p.state='active' AND ib.retired_at IS NULL) SELECT 'source' AS tag,CAST(NULL AS TEXT) AS scope_kind,CAST(NULL AS TEXT) AS scope_id,CAST(NULL AS TEXT) AS incarnation,CAST(NULL AS BIGINT) AS version,{source_fields} FROM base b UNION ALL SELECT 'epoch',e.scope_kind,e.scope_id,e.incarnation,e.version,{absent_fields} FROM base b JOIN authority_epoch e ON (e.scope_kind IN ('policy','routing','directory','protection') AND e.scope_id='') OR (e.scope_kind='account' AND e.scope_id=b.account_id) OR (e.scope_kind='session' AND e.scope_id=b.session_id) OR (e.scope_kind='principal' AND e.scope_id=b.principal_id) OR (e.scope_kind='domain' AND e.scope_id IN(SELECT name FROM domains_in)) OR (e.scope_kind='subject' AND e.scope_id IN(SELECT name FROM subjects_in))) fenced ORDER BY tag,scope_kind,scope_id")
    }));
    &all[usize::from(dialect == Dialect::Postgres)]
}
impl Store {
    /// One current statement, NOT an allow decision. Missing/unsupported source
    /// or epoch returns None and requires full original authentication. No
    /// account/principal ID from cached data or client arguments drives a join.
    pub async fn native_authority_fence(
        &self,
        cookie: &str,
        known_domains: &[String],
    ) -> Result<Option<NativeAuthorityFence>, StoreError> {
        self.native_authority_fence_with_subjects(cookie, known_domains, &[])
            .await
    }
    /// Additional exact policy subjects come from a verified cold builder, not
    /// client account selectors. Missing/unsupported dependency epochs cannot
    /// certify absence: they require the complete original SQL path. In this
    /// first stage subjects without a live source cannot be seeded arbitrarily.
    pub async fn native_authority_fence_with_subjects(
        &self,
        cookie: &str,
        known_domains: &[String],
        known_subjects: &[String],
    ) -> Result<Option<NativeAuthorityFence>, StoreError> {
        if cookie.is_empty()
            || cookie.len() > 1024
            || known_domains.len() > 16
            || known_subjects.len() > 8
            || known_domains.len() + known_subjects.len() + 7 > MAX_SCOPES
            || known_subjects.iter().any(|name| name.len() > 1024)
            || known_domains.iter().any(|name| name.len() > 1024)
        {
            return Err(invalid("native authority fence input bound exceeded"));
        }
        let mut domains = Vec::new();
        for name in known_domains {
            let scope = AuthorityScope::new(AuthorityKind::Domain, name)?;
            if !domains.contains(&scope) {
                domains.push(scope);
            }
        }
        let input = serde_json::to_string(
            &domains
                .iter()
                .map(|scope| scope.id.as_str())
                .collect::<Vec<_>>(),
        )
        .map_err(|_| corrupt("native fence encoding failed"))?;
        if input.len() > MAX_INPUT_BYTES {
            return Err(invalid("native authority fence encoding bound exceeded"));
        }
        let mut subjects = Vec::new();
        for name in known_subjects {
            let scope = AuthorityScope::new(AuthorityKind::Subject, name)?;
            if !subjects.contains(&scope) {
                subjects.push(scope);
            }
        }
        let subjects_input =
            serde_json::to_string(&subjects.iter().map(|s| s.id.as_str()).collect::<Vec<_>>())
                .map_err(|_| corrupt("native fence subject encoding failed"))?;
        if input.len() + subjects_input.len() > MAX_INPUT_BYTES {
            return Err(invalid("native authority fence encoding bound exceeded"));
        }
        let generation = self.authority_generation().await;
        let rows = q(fence_sql(self.backend.dialect()))
            .bind(cookie)
            .bind(input)
            .bind(subjects_input)
            .fetch_all(&self.backend)
            .await?;
        if generation != self.authority_generation().await {
            return Err(StoreError::SourceChanged);
        }
        if rows.len() > MAX_SCOPES + 1 {
            return Err(corrupt("native authority fence row bound exceeded"));
        }
        let mut source = None;
        let mut stamps = BTreeMap::new();
        for row in rows {
            match text(&row, "tag")?.as_str() {
                "source" => {
                    if source.replace(row).is_some() {
                        return Err(corrupt("duplicate native authority source"));
                    }
                }
                "epoch" => {
                    let stamp = stamp(&row)?;
                    stamps.insert(stamp.scope.clone(), stamp);
                }
                _ => return Err(corrupt("unknown native authority fence section")),
            }
        }
        let Some(source) = source else {
            return Ok(None);
        };
        // Nulls are the bounded unsupported marker. Do not invent missing IDs
        // or turn corrupted runtime clocks into an authenticated allow.
        let required = [
            "session_id",
            "account_id",
            "principal_id",
            "binding_id",
            "created",
            "activity",
        ];
        let mut values = Vec::new();
        for field in required {
            let Some(value) = source.try_get_opt_string(field)? else {
                return Ok(None);
            };
            values.push(value);
        }
        let [session, account, principal, binding, created, activity]: [String; 6] = values
            .try_into()
            .map_err(|_| corrupt("native authority source shape"))?;
        let mut expected = domains;
        expected.extend(subjects);
        for kind in [
            AuthorityKind::Policy,
            AuthorityKind::Routing,
            AuthorityKind::Directory,
            AuthorityKind::Protection,
        ] {
            expected.push(AuthorityScope::new(kind, "")?);
        }
        expected.extend([
            AuthorityScope::new(AuthorityKind::Session, &session)?,
            AuthorityScope::new(AuthorityKind::Account, &account)?,
            AuthorityScope::new(AuthorityKind::Principal, &principal)?,
        ]);
        expected.sort();
        expected.dedup();
        if stamps.len() != expected.len()
            || expected.iter().any(|scope| !stamps.contains_key(scope))
        {
            return Ok(None);
        }
        Ok(Some(NativeAuthorityFence {
            versions: AuthorityVersions {
                store: self.cache_scope(),
                generation,
                stamps,
            },
            session,
            account,
            principal,
            binding,
            created,
            activity,
            last_seen: source.try_get_opt_string("last_seen")?,
        }))
    }
}
