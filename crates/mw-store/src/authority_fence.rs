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
    primary_revision:Option<crate::settings_policy::SettingsReadRevision>,
    publication_schema:Option<String>,
    mail_data:Option<crate::mail_data_publication::MailDataFence>,
}
impl fmt::Debug for NativeAuthorityFence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeAuthorityFence")
            .field("scopes", &self.versions.stamps.len())
            .finish_non_exhaustive()
    }
}
impl NativeAuthorityFence {
    /// The settings namespace number from this SAME source-row observation.
    /// Store mints the typed value; a public numeric value cannot manufacture
    /// it. This is request DATA, not authentication, a lease or a cached allow.
    pub fn primary_policy_revision(&self,store:&Store)->Result<Option<&crate::settings_policy::SettingsReadRevision>,StoreError>{
        if self.versions.store!=store.cache_scope(){return Err(invalid("foreign primary policy fence"))}
        Ok(self.primary_revision.as_ref())
    }
    /// Optional DATA proof read in the SAME final statement as this authority
    /// source. A standalone preparation snapshot cannot populate this field.
    pub fn mail_data(&self)->Option<&crate::mail_data_publication::MailDataFence> {self.mail_data.as_ref()}
    pub fn publication_schema(&self)->Option<&str> {self.publication_schema.as_deref()}
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
            && self.publication_schema==other.publication_schema && versions)
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
        let source_fields=format!("{},b.primary_policy_revision",fields.iter().map(|(name,_)|format!("b.{name}")).collect::<Vec<_>>().join(","));
        let absent_fields=format!("{},CAST(NULL AS BIGINT)",fields.iter().map(|_|"CAST(NULL AS TEXT)").collect::<Vec<_>>().join(","));
        // Keep malformed SQLite dynamic values out of the result before
        // transfer; missing/invalid is unsupported, never an invented zero.
        let primary_revision=if pg{"CASE WHEN r.n>=0 THEN r.n ELSE CAST(NULL AS BIGINT) END"}else{"CASE WHEN typeof(r.n)='integer' AND r.n>=0 THEN r.n ELSE CAST(NULL AS BIGINT) END"};
        let domains=if pg{"SELECT value AS name FROM json_array_elements_text(?2::json)"}else{"SELECT value AS name FROM json_each(?2)"};
        let subjects=if pg{"SELECT value AS name FROM json_array_elements_text(?3::json)"}else{"SELECT value AS name FROM json_each(?3)"};
        format!("SELECT * FROM (WITH domains_in AS ({domains}),subjects_in AS ({subjects}),base AS (SELECT {bounded},{primary_revision} AS primary_policy_revision FROM sessions s LEFT JOIN settings_revision r ON r.id=1 JOIN accounts a ON a.id=s.account_id JOIN session_principal sp ON sp.session_id=s.id AND sp.primary_account_id=s.account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding ib ON ib.id=sp.binding_id AND ib.principal_id=p.id WHERE s.id=?1 AND s.linked_internal=0 AND s.jmap_url='engine' AND s.api_url='engine' AND a.kind IN ('imap','pop3') AND COALESCE(a.backend_kind,'')<>'ews' AND p.state='active' AND ib.retired_at IS NULL) SELECT 'source' AS tag,CAST(NULL AS TEXT) AS scope_kind,CAST(NULL AS TEXT) AS scope_id,CAST(NULL AS TEXT) AS incarnation,CAST(NULL AS BIGINT) AS version,{source_fields} FROM base b UNION ALL SELECT 'epoch',e.scope_kind,e.scope_id,e.incarnation,e.version,{absent_fields} FROM base b JOIN authority_epoch e ON (e.scope_kind IN ('policy','routing','directory','protection') AND e.scope_id='') OR (e.scope_kind='account' AND e.scope_id=b.account_id) OR (e.scope_kind='session' AND e.scope_id=b.session_id) OR (e.scope_kind='principal' AND e.scope_id=b.principal_id) OR (e.scope_kind='domain' AND e.scope_id IN(SELECT name FROM domains_in)) OR (e.scope_kind='subject' AND e.scope_id IN(SELECT name FROM subjects_in))) fenced ORDER BY tag,scope_kind,scope_id")
    }));
    &all[usize::from(dialect == Dialect::Postgres)]
}
impl Store {
    /// Settle telemetry only, using an opaque current native source. Neither a
    /// successful update nor zero affected rows authorizes a request. Another
    /// legitimate request may already have touched the row; the caller MUST
    /// obtain its final fresh source/clock fence after this operation.
    /// This never changes activity_at, deadlines or a missing session.
    pub async fn settle_native_last_seen(
        &self, source: &NativeAuthorityFence, min_age_secs: i64,
        idle_secs:u64, absolute_secs:u64,
    ) -> Result<bool, StoreError> {
        if source.versions.store != self.cache_scope() {
            return Err(invalid("foreign native session telemetry source"));
        }
        if min_age_secs <= 0 || !self.uses_sqlite() {
            return Err(invalid("unsupported native telemetry settlement"));
        }
        let now=chrono::Utc::now();
        if !self.session_touch_may_be_due_at(source.last_seen(), min_age_secs, now) {
            return Ok(false);
        }
        let age=chrono::Duration::try_seconds(min_age_secs).ok_or_else(||invalid("invalid native telemetry age"))?;
        let idle=i64::try_from(idle_secs).map_err(|_|invalid("invalid native telemetry idle"))?;
        let absolute=i64::try_from(absolute_secs).map_err(|_|invalid("invalid native telemetry absolute"))?;
        let stamps=source.versions.stamps.values().map(|s|
            (s.scope.kind.as_str(), &s.scope.id, &s.incarnation, s.version)
        ).collect::<Vec<_>>();
        let epochs=serde_json::to_string(&stamps)
            .map_err(|_|invalid("invalid native telemetry source"))?;
        // The write transaction pins the generation; no transaction escapes
        // across connector waits, a response, or an index publication.
        let mut tx=self.backend.begin().await?;
        if source.versions.generation != tx.generation_path() {
            tx.rollback().await?; return Ok(false);
        }
        // Re-sample after acquiring the write transaction, never using a time
        // captured before a writer wait. Clock policy is DATA resolved at the
        // exact source epochs; the UPDATE also fences those epochs and clocks.
        let now=chrono::Utc::now();
        let valid_clock=|value:&str,secs:i64|->bool {
            let Some(time)=chrono::DateTime::parse_from_rfc3339(value).ok().map(|v|v.with_timezone(&chrono::Utc)) else{return false;};
            time<=now && (secs==0 || chrono::Duration::try_seconds(secs).and_then(|age|time.checked_add_signed(age)).is_some_and(|end|now<end))
        };
        if idle==0 || !valid_clock(&source.activity,idle) || !valid_clock(&source.created,absolute) {
            tx.rollback().await?;return Ok(false);
        }
        let cutoff=now.checked_sub_signed(age).ok_or_else(||invalid("invalid native telemetry cutoff"))?;
        let changed=q("UPDATE sessions SET last_seen=?1 WHERE id=?2 AND account_id=?3 AND created_at=?4 AND COALESCE(activity_at,created_at)=?9 AND julianday(created_at)<=julianday('now') AND julianday(COALESCE(activity_at,created_at))<=julianday('now') AND (?10=0 OR julianday(created_at)>julianday('now')-(?10/86400.0)) AND julianday(COALESCE(activity_at,created_at))>julianday('now')-(?11/86400.0) AND last_seen<?5 AND linked_internal=0 AND jmap_url='engine' AND api_url='engine' AND EXISTS(SELECT 1 FROM accounts a JOIN session_principal sp ON sp.primary_account_id=a.id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding ib ON ib.id=sp.binding_id AND ib.principal_id=p.id WHERE a.id=sessions.account_id AND a.kind IN ('imap','pop3') AND COALESCE(a.backend_kind,'')<>'ews' AND sp.session_id=sessions.id AND sp.principal_id=?6 AND sp.binding_id=?7 AND p.state='active' AND ib.retired_at IS NULL) AND NOT EXISTS(SELECT 1 FROM json_each(?8) j WHERE NOT EXISTS(SELECT 1 FROM authority_epoch e WHERE e.scope_kind=json_extract(j.value,'$[0]') AND e.scope_id=json_extract(j.value,'$[1]') AND e.incarnation=json_extract(j.value,'$[2]') AND e.version=json_extract(j.value,'$[3]')))")
            .bind(now.to_rfc3339()).bind(&source.session).bind(&source.account)
            .bind(&source.created).bind(cutoff.to_rfc3339()).bind(&source.principal)
            .bind(&source.binding).bind(epochs).bind(&source.activity).bind(absolute).bind(idle).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(changed==1)
    }
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
        self.native_authority_fence_impl(cookie,known_domains,known_subjects,false,None).await
    }
    pub async fn native_publication_fence(&self,cookie:&str,known_domains:&[String],known_subjects:&[String])->Result<Option<NativeAuthorityFence>,StoreError> {
        self.native_authority_fence_impl(cookie,known_domains,known_subjects,true,None).await
    }
    /// Descriptor IDs shape DATA dependencies only. The source account still
    /// comes from the live cookie/session joins; a mismatched descriptor cannot
    /// select another account's DATA or manufacture an authenticated session.
    pub async fn native_publication_fence_with_mail_data(&self,cookie:&str,known_domains:&[String],known_subjects:&[String],descriptor:&crate::mail_data_publication::MailDataDescriptor)->Result<Option<NativeAuthorityFence>,StoreError> {
        self.check_mail_descriptor(descriptor)?;
        self.native_authority_fence_impl(cookie,known_domains,known_subjects,true,Some(descriptor)).await
    }
    async fn native_authority_fence_impl(&self,cookie:&str,known_domains:&[String],known_subjects:&[String],publication:bool,descriptor:Option<&crate::mail_data_publication::MailDataDescriptor>)->Result<Option<NativeAuthorityFence>,StoreError> {
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
        let sql=if descriptor.is_some() {publication_mail_sql(self.backend.dialect())}else if publication {publication_sql(self.backend.dialect())}else{fence_sql(self.backend.dialect())};
        let query = q(sql)
            .bind(cookie)
            .bind(input)
            .bind(subjects_input);
        let rows=if let Some(descriptor)=descriptor {
            query.bind(descriptor.input_json()).bind(descriptor.account()).fetch_all(&self.backend).await?
        }else {query.fetch_all(&self.backend).await?};
        if generation != self.authority_generation().await {
            return Err(StoreError::SourceChanged);
        }
        if rows.len() > MAX_SCOPES + 1 {
            return Err(corrupt("native authority fence row bound exceeded"));
        }
        let publication_schema=if publication {
            let Some(first)=rows.first() else {return Ok(None);};
            let Some(stamp)=first.try_get_opt_string("publication_schema")? else {return Ok(None);};
            if stamp.len()>128 {return Ok(None);}
            Some(stamp)
        }else{None};
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
        let mail_data=if let Some(descriptor)=descriptor {
            self.mail_data_from_joined(descriptor,generation.clone(),source.try_get_opt_string("mail_data")?.as_deref())?
        }else {None};
        let primary_revision=match source.get_json("primary_policy_revision") {
            serde_json::Value::Null=>None,
            serde_json::Value::Number(number)=>{
                let n=number.as_i64().filter(|n|*n>=0).ok_or_else(||corrupt("invalid primary policy source revision"))?;
                Some(self.settings_revision_from_snapshot(Some(n)))
            },
            _=>return Err(corrupt("invalid primary policy source revision")),
        };
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
            primary_revision,
            publication_schema,
            mail_data,
        }))
    }
}

pub(super) fn publication_sql(dialect:Dialect)->&'static str {
    static SQL:OnceLock<[String;2]>=OnceLock::new();
    &SQL.get_or_init(||std::array::from_fn(|pg| {
        let d=if pg==1 {Dialect::Postgres}else{Dialect::Sqlite};
        format!("SELECT f.*,{} AS publication_schema FROM ({}) f ORDER BY f.tag,f.scope_kind,f.scope_id",super::schema::stamp_expression(d),fence_sql(d))
    }))[usize::from(dialect==Dialect::Postgres)]
}

fn publication_mail_sql(dialect:Dialect)->&'static str {
    static SQL:OnceLock<[String;2]>=OnceLock::new();
    &SQL.get_or_init(||std::array::from_fn(|pg| {
        let d=if pg==1 {Dialect::Postgres}else{Dialect::Sqlite};
        let data=crate::mail_data_publication::joined_mail_data_sql(d,"f.account_id","?4");
        format!("SELECT f.*,{} AS publication_schema,CASE WHEN f.tag='source' AND f.account_id=?5 THEN {data} ELSE CAST(NULL AS TEXT) END AS mail_data FROM ({}) f ORDER BY f.tag,f.scope_kind,f.scope_id",super::schema::stamp_expression(d),fence_sql(d))
    }))[usize::from(dialect==Dialect::Postgres)]
}
