//! Bounded cold publication readers. No legacy revision memo is consulted.
use super::*;
use crate::backend::Dialect;
use std::sync::OnceLock;
const ROW_CAP: usize = 4096;
const BYTE_CAP: usize = 2 * 1024 * 1024;
impl Store {
    #[cfg(test)]
    pub(crate) async fn settings_publication_probe_for_test(
        &self,
        s: &CanonicalSettingSubject,
    ) -> Result<(usize, usize), StoreError> {
        let rows = q(publication_sql(self.backend.dialect()))
            .bind(&s.domain)
            .bind(&s.account_id)
            .bind(&s.address)
            .fetch_all(&self.backend)
            .await?;
        Ok((
            rows.len(),
            rows.iter().map(|r| r.retained_data_bytes()).sum(),
        ))
    }
    pub(crate) async fn publication_policy_address(
        &self,
        input: &str,
    ) -> Result<String, StoreError> {
        if input.len() > 1024 || !input.is_ascii() || input.bytes().any(|c| c < 32 || c == 127) {
            return Err(StoreError::Invalid("unsupported publication address"));
        }
        let address = input.trim().to_lowercase();
        let Some((local, raw)) = address.rsplit_once('@') else {
            return Ok(address);
        };
        let mut domain = normalize_policy_domain(raw)?;
        let mut seen = Vec::new();
        while !domain.is_empty() {
            if seen.len() >= 16 || seen.contains(&domain) {
                return Err(StoreError::Invalid("cyclic publication alias"));
            }
            seen.push(domain.clone());
            let sql = if self.backend.dialect() == Dialect::Postgres {
                "SELECT CASE WHEN octet_length(domain)<=1024 THEN domain ELSE NULL END AS target FROM domain_alias WHERE alias=?1"
            } else {
                "SELECT CASE WHEN typeof(domain)='text' AND octet_length(domain)<=1024 THEN domain ELSE NULL END AS target FROM domain_alias WHERE alias=?1"
            };
            let Some(row) = q(sql).bind(&domain).fetch_optional(&self.backend).await? else {
                break;
            };
            let target = row
                .try_get_opt_string("target")?
                .ok_or(StoreError::Invalid("unsupported publication alias"))?;
            if !target.is_ascii() || target.bytes().any(|c| c < 32 || c == 127) {
                return Err(StoreError::Invalid("unsupported publication alias"));
            }
            domain = normalize_policy_domain(&target)?;
        }
        Ok(format!("{local}@{domain}"))
    }
    pub async fn settings_publication_canonical(
        &self,
        subject: &SettingSubject<'_>,
    ) -> Result<CanonicalSettingSubject, StoreError> {
        if subject.personal_owner.is_some()
            || [subject.domain, subject.account_id, subject.address]
                .iter()
                .any(|s| s.len() > 1024 || !s.is_ascii())
        {
            return Err(StoreError::Invalid("unsupported publication subject"));
        }
        let address = self.publication_policy_address(subject.address).await?;
        let domain = if subject.domain.trim().is_empty() {
            String::new()
        } else {
            let name = self
                .publication_policy_address(&format!("publication@{}", subject.domain))
                .await?;
            name.rsplit_once('@')
                .ok_or(StoreError::Invalid("unsupported publication domain"))?
                .1
                .to_owned()
        };
        Ok(CanonicalSettingSubject {
            personal_owner: None,
            domain,
            account_id: subject.account_id.to_owned(),
            address,
        })
    }
    /// Caller supplies a canonical subject derived from the cold source under
    /// routing/policy fences. This is a DATA reader, not an authority token.
    /// Caps and type witnesses are applied inside the same statement BEFORE any
    /// returned policy/value payload; mutation cannot race a separate preflight.
    pub async fn settings_publication_inputs(
        &self,
        subject: &CanonicalSettingSubject,
    ) -> Result<SettingsResolutionInputs, StoreError> {
        if subject.personal_owner.is_some()
            || [&subject.domain, &subject.account_id, &subject.address]
                .iter()
                .any(|s| s.len() > 1024 || !s.is_ascii() || s.bytes().any(|b| b < 32 || b == 127))
        {
            return Err(StoreError::Invalid("unsupported publication subject"));
        }
        let rows = q(publication_sql(self.backend.dialect()))
            .bind(&subject.domain)
            .bind(&subject.account_id)
            .bind(&subject.address)
            .fetch_all(&self.backend)
            .await?;
        if rows.len() > ROW_CAP || rows.iter().any(|r| r.get_i64("invalid") != 0) {
            return Err(StoreError::Invalid("publication settings bound or shape"));
        }
        let mut policies = Vec::new();
        let mut values = Vec::new();
        for row in rows {
            for field in ["tag", "scope_kind", "scope_id", "key", "updated_by"] {
                row.try_get_opt_string(field)?
                    .ok_or(StoreError::Invalid("missing publication text"))?;
            }
            if row.get_string("tag") == "value" && row.try_get_opt_string("value")?.is_none() {
                return Err(StoreError::Invalid("missing publication value"));
            }
            if row.get_string("tag") == "policy" {
                policies.push(SettingPolicyRow {
                    key: row.get_string("key"),
                    global_value: row.try_get_opt_string("value")?,
                    allow_user_override: row.get_i64("allow_user_override") != 0,
                    global_locked: row.get_i64("global_locked") != 0,
                    updated_by: row.get_string("updated_by"),
                    updated_at: row.get_i64("updated_at"),
                });
            } else if row.get_string("tag") == "value" {
                values.push(value_from_row(&row)?);
            }
        }
        Ok(SettingsResolutionInputs {
            canonical: subject.clone(),
            policies,
            values,
        })
    }
}
fn publication_sql(dialect: Dialect) -> &'static str {
    static SQL: OnceLock<[String; 2]> = OnceLock::new();
    &SQL.get_or_init(||std::array::from_fn(|pg| {
        let text=|x:&str,nullable:bool| if pg==1 {"TRUE".to_owned()} else {format!("(typeof({x})='text'{} )",if nullable {format!(" OR {x} IS NULL")}else{String::new()})};
        let integer=|x:&str| if pg==1 {"TRUE".to_owned()}else{format!("typeof({x})='integer'")};
        let group_valid=text("g.id",false); let profile_valid=text("a.profile_id",false);
        let fields=[("key",false),("value",true),("scope_kind",false),("scope_id",false),("updated_by",false)];
        let valid=fields.iter().map(|(f,n)|format!("{} AND COALESCE(octet_length({f}),0)<=1048576",text(f,*n))).chain(["locked","updated_at","allow_user_override","global_locked"].iter().map(|f|integer(f))).collect::<Vec<_>>().join(" AND ");
        let scalar=["key","value","scope_kind","scope_id","updated_by"].iter().map(|f|format!("COALESCE(octet_length({f}),0)")).collect::<Vec<_>>().join("+");
        let output=fields.iter().map(|(f,_)|format!("CASE WHEN x.ok=1 THEN x.{f} ELSE CAST(NULL AS TEXT) END AS {f}")).chain(["locked","updated_at","allow_user_override","global_locked"].iter().map(|f|format!("CASE WHEN x.ok=1 THEN x.{f} ELSE CAST(NULL AS BIGINT) END AS {f}"))).collect::<Vec<_>>().join(",");
        // Candidate bases duplicate rows for account/address at equal scope/key
        // have unspecified old DB order; conservatively fall back, never invent
        // a precedence different from the ordinary reader.
        format!("SELECT x.tag,CAST(CASE WHEN x.ok=1 THEN 0 ELSE 1 END AS BIGINT) AS invalid,{output} FROM (WITH groups_in AS (SELECT CASE WHEN {group_valid} AND octet_length(g.id)<=1024 THEN g.id ELSE NULL END AS id FROM user_group g JOIN user_group_member m ON m.group_id=g.id WHERE m.account_id=?2 AND ?2<>'' AND (g.domain='' OR g.domain=?1) ORDER BY g.id LIMIT 129),assignments AS (SELECT CASE WHEN {profile_valid} AND octet_length(a.profile_id)<=1024 THEN a.profile_id ELSE NULL END AS profile_id,ROW_NUMBER() OVER(ORDER BY a.priority DESC,CASE a.scope_kind WHEN 'user' THEN 0 WHEN 'group' THEN 1 ELSE 2 END,a.profile_id,a.scope_id) AS ordinal FROM feature_profile_assignment a WHERE (a.scope_kind='domain' AND a.scope_id=?1 AND ?1<>'') OR (a.scope_kind='user' AND a.scope_id=?2 AND ?2<>'') OR (a.scope_kind='group' AND a.scope_id IN(SELECT id FROM groups_in)) ORDER BY a.priority DESC,CASE a.scope_kind WHEN 'user' THEN 0 WHEN 'group' THEN 1 ELSE 2 END,a.profile_id,a.scope_id LIMIT 129),profiles_in AS (SELECT profile_id,MIN(ordinal) AS ordinal FROM assignments GROUP BY profile_id),base_values AS (SELECT * FROM setting_value WHERE (scope_kind='domain' AND scope_id=?1 AND ?1<>'') OR (scope_kind IN('admin_user','user') AND ((scope_id=?2 AND ?2<>'') OR (scope_id=?3 AND ?3<>'')))),all_rows AS (SELECT 'check' AS tag,CAST(-1 AS BIGINT) AS ordinal,'' AS scope_kind,'' AS scope_id,'' AS key,CAST(NULL AS TEXT) AS value,CAST(0 AS BIGINT) AS locked,'' AS updated_by,CAST(0 AS BIGINT) AS updated_at,CAST(0 AS BIGINT) AS allow_user_override,CAST(0 AS BIGINT) AS global_locked UNION ALL SELECT 'policy',CAST(0 AS BIGINT),'','',key,global_value,CAST(0 AS BIGINT),updated_by,updated_at,allow_user_override,global_locked FROM setting_policy UNION ALL SELECT 'value',CAST(1 AS BIGINT),scope_kind,scope_id,key,value,locked,updated_by,updated_at,CAST(0 AS BIGINT),CAST(0 AS BIGINT) FROM base_values UNION ALL SELECT 'value',CAST(1000 AS BIGINT)+p.ordinal,v.scope_kind,v.scope_id,v.key,v.value,v.locked,v.updated_by,v.updated_at,CAST(0 AS BIGINT),CAST(0 AS BIGINT) FROM profiles_in p JOIN setting_value v ON v.scope_kind='profile' AND v.scope_id=p.profile_id UNION ALL SELECT 'value',CAST(2000 AS BIGINT)+ROW_NUMBER() OVER(ORDER BY g.id),v.scope_kind,v.scope_id,v.key,v.value,v.locked,v.updated_by,v.updated_at,CAST(0 AS BIGINT),CAST(0 AS BIGINT) FROM groups_in g JOIN setting_value v ON v.scope_kind='group' AND v.scope_id=g.id),bounded AS (SELECT * FROM all_rows ORDER BY ordinal,key LIMIT 4097),budget AS (SELECT CASE WHEN COUNT(*)<={ROW_CAP} AND COALESCE(SUM({scalar}+64),0)<={BYTE_CAP} AND COALESCE(MIN(CASE WHEN {valid} THEN 1 ELSE 0 END),1)=1 AND (SELECT COUNT(*) FROM groups_in)<=128 AND NOT EXISTS(SELECT 1 FROM groups_in WHERE id IS NULL) AND (SELECT COUNT(*) FROM assignments)<=128 AND NOT EXISTS(SELECT 1 FROM assignments WHERE profile_id IS NULL) AND NOT EXISTS(SELECT 1 FROM base_values GROUP BY scope_kind,key HAVING COUNT(*)>1) THEN 1 ELSE 0 END AS ok FROM bounded) SELECT bounded.*,budget.ok FROM bounded CROSS JOIN budget) x ORDER BY x.ordinal,x.key")
    }))[usize::from(dialect==Dialect::Postgres)]
}
