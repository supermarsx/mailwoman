//! Fresh final-admission inputs. No admission result survives this invocation.
use crate::{
    Store, StoreError,
    backend::{Dialect, Row},
    q,
    settings_policy::{SettingPolicyRow, SettingsReadRevision},
};
use std::collections::HashMap;

pub const MAX_FINAL_GATE_KEYS: usize = 256;
pub const MAX_FINAL_GATE_PARAMETER_BYTES: usize = 128 * 1024;

#[derive(Debug)]
pub struct FinalGateDomain {
    pub canonical_domain: String,
    pub enabled: Option<bool>,
    pub second_alias: bool,
    /// Incompatible SQLite storage classes require the unchanged legacy reader.
    pub legacy_read_required: bool,
}

#[cfg(test)]
#[path = "final_gate_snapshot_tests.rs"]
mod tests;
/// Deferred section decoding preserves policy/domain/directory/flags ordering.
#[derive(Debug)]
pub struct FinalGateSnapshot {
    pub revision: SettingsReadRevision,
    pub global_policy: Result<Option<SettingPolicyRow>, StoreError>,
    pub domains: HashMap<String, Result<FinalGateDomain, StoreError>>,
    pub held_addresses: Vec<Result<String, StoreError>>,
    pub exact_flags: HashMap<String, Result<String, StoreError>>,
}

fn text(row: &Row, col: &str) -> Result<String, StoreError> {
    row.try_get_opt_string(col)?
        .ok_or_else(|| StoreError::Corrupt(format!("missing final gate {col}")))
}
fn number(row: &Row, col: &str) -> Result<Option<i64>, StoreError> {
    use sqlx::Row as _;
    Ok(match row {
        Row::Sqlite(row) => row.try_get(col)?,
        Row::Postgres(row) => row.try_get(col)?,
    })
}
fn policy(row: &Row) -> Result<Option<SettingPolicyRow>, StoreError> {
    let Some(key) = row.try_get_opt_string("policy_key")? else {
        return Ok(None);
    };
    Ok(Some(SettingPolicyRow {
        key,
        global_value: row.try_get_opt_string("global_value")?,
        allow_user_override: number(row, "allow_override")?
            .ok_or_else(|| StoreError::Corrupt("missing policy override".into()))?
            != 0,
        global_locked: number(row, "global_locked")?
            .ok_or_else(|| StoreError::Corrupt("missing policy lock".into()))?
            != 0,
        updated_by: text(row, "updated_by")?,
        updated_at: number(row, "updated_at")?
            .ok_or_else(|| StoreError::Corrupt("missing policy timestamp".into()))?,
    }))
}

// Every section is independently bounded. Final ORDER BY preserves the original
// database collation/order of held addresses (including early-match behavior).
// SQLite eligibility checks storage classes, then at most 64 KiB of combined
// configuration text for printable ASCII and embedded NUL. Oversized or any
// international/control text uses the original decoder; it is never denied by
// eligibility. This scans bounded DB bytes, but transfers/parses no config JSON.
const SQLITE: &str = "SELECT * FROM (WITH domains_in AS (SELECT CAST(value AS TEXT) AS name FROM json_each(?1)), keys_in AS (SELECT CAST(value AS TEXT) AS name FROM json_each(?2))
 SELECT 'policy' AS tag,CAST(NULL AS TEXT) AS input,CAST(NULL AS TEXT) AS canonical,CAST(NULL AS BIGINT) AS enabled,CAST(NULL AS TEXT) AS second_alias,r.n AS revision,p.key AS policy_key,p.global_value,p.allow_user_override AS allow_override,p.global_locked AS global_locked,p.updated_by,p.updated_at,CAST(NULL AS TEXT) AS setting_key,CAST(NULL AS TEXT) AS setting_value,CAST(NULL AS TEXT) AS held_address,CAST(NULL AS BIGINT) AS domain_shape
 FROM (SELECT 1) seed LEFT JOIN settings_revision r ON r.id=1 LEFT JOIN setting_policy p ON p.key='signin.domainMode'
 UNION ALL SELECT 'domain',i.name,COALESCE(a.domain,i.name),CAST(d.enabled AS BIGINT),a2.alias,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,
 CASE WHEN d.name IS NULL THEN 1
 WHEN NOT (typeof(d.name)='text' AND typeof(d.upstream_json)='text' AND typeof(d.allowlist)='text' AND typeof(d.blocklist)='text' AND typeof(d.settings_json)='text' AND typeof(d.enabled)='integer' AND typeof(d.preferred)='integer' AND typeof(d.updated_at) IN ('integer','null')) THEN 0
 WHEN octet_length(d.name)+octet_length(d.upstream_json)+octet_length(d.allowlist)+octet_length(d.blocklist)+octet_length(d.settings_json)>65536 THEN 0
 WHEN instr(CAST(d.name AS BLOB),X'00') OR instr(CAST(d.upstream_json AS BLOB),X'00') OR instr(CAST(d.allowlist AS BLOB),X'00') OR instr(CAST(d.blocklist AS BLOB),X'00') OR instr(CAST(d.settings_json AS BLOB),X'00') THEN 0
 WHEN d.name GLOB '*[^ -~]*' OR d.upstream_json GLOB '*[^ -~]*' OR d.allowlist GLOB '*[^ -~]*' OR d.blocklist GLOB '*[^ -~]*' OR d.settings_json GLOB '*[^ -~]*' THEN 0 ELSE 1 END
 FROM domains_in i LEFT JOIN domain_alias a ON a.alias=i.name LEFT JOIN domains d ON d.name=COALESCE(a.domain,i.name) LEFT JOIN domain_alias a2 ON a2.alias=COALESCE(a.domain,i.name)
 UNION ALL SELECT 'hold',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,h.address,NULL FROM (SELECT address FROM mail_user WHERE status IN ('disabled','deleting','deleted') ORDER BY address LIMIT 5001) h
 UNION ALL SELECT 'flag',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,s.key,s.value,NULL,NULL FROM settings s WHERE s.key IN (SELECT name FROM keys_in)
 ) final_gate ORDER BY tag,held_address";
const POSTGRES: &str = "SELECT * FROM (WITH domains_in AS (SELECT value AS name FROM json_array_elements_text(?1::json)), keys_in AS (SELECT value AS name FROM json_array_elements_text(?2::json))
 SELECT 'policy' AS tag,CAST(NULL AS TEXT) AS input,CAST(NULL AS TEXT) AS canonical,CAST(NULL AS BIGINT) AS enabled,CAST(NULL AS TEXT) AS second_alias,r.n AS revision,p.key AS policy_key,p.global_value,CAST(p.allow_user_override AS BIGINT) AS allow_override,CAST(p.global_locked AS BIGINT) AS global_locked,p.updated_by,p.updated_at,CAST(NULL AS TEXT) AS setting_key,CAST(NULL AS TEXT) AS setting_value,CAST(NULL AS TEXT) AS held_address,CAST(NULL AS BIGINT) AS domain_shape
 FROM (SELECT 1) seed LEFT JOIN settings_revision r ON r.id=1 LEFT JOIN setting_policy p ON p.key='signin.domainMode'
 UNION ALL SELECT 'domain',i.name,COALESCE(a.domain,i.name),CAST(d.enabled AS BIGINT),a2.alias,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,CAST(1 AS BIGINT)
 FROM domains_in i LEFT JOIN domain_alias a ON a.alias=i.name LEFT JOIN domains d ON d.name=COALESCE(a.domain,i.name) LEFT JOIN domain_alias a2 ON a2.alias=COALESCE(a.domain,i.name)
 UNION ALL SELECT 'hold',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,h.address,NULL FROM (SELECT address FROM mail_user WHERE status IN ('disabled','deleting','deleted') ORDER BY address LIMIT 5001) h
 UNION ALL SELECT 'flag',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,s.key,s.value,NULL,NULL FROM settings s WHERE s.key IN (SELECT name FROM keys_in)
 ) final_gate ORDER BY tag,held_address";

impl Store {
    /// A fresh, coherent input snapshot. Callers must retain the existing late
    /// admission boundary, pure policy resolver and unsupported-case fallback.
    pub async fn final_account_gate_snapshot(
        &self,
        domains: &[String],
        exact_keys: &[String],
    ) -> Result<FinalGateSnapshot, StoreError> {
        if domains.len() > MAX_FINAL_GATE_KEYS || exact_keys.len() > MAX_FINAL_GATE_KEYS {
            return Err(StoreError::Corrupt(
                "final gate input count exceeds bound".into(),
            ));
        }
        let raw_bytes = domains
            .iter()
            .chain(exact_keys)
            .try_fold(0usize, |bytes, value| bytes.checked_add(value.len()));
        if raw_bytes.is_none_or(|bytes| bytes > MAX_FINAL_GATE_PARAMETER_BYTES) {
            return Err(StoreError::Corrupt(
                "final gate input bytes exceed bound".into(),
            ));
        }
        let domains_json = serde_json::to_string(domains)
            .map_err(|_| StoreError::Corrupt("final gate domain encoding".into()))?;
        let keys_json = serde_json::to_string(exact_keys)
            .map_err(|_| StoreError::Corrupt("final gate key encoding".into()))?;
        if domains_json.len().saturating_add(keys_json.len()) > MAX_FINAL_GATE_PARAMETER_BYTES {
            return Err(StoreError::Corrupt(
                "final gate input bytes exceed bound".into(),
            ));
        }
        let sql = match self.backend.dialect() {
            Dialect::Sqlite => SQLITE,
            Dialect::Postgres => POSTGRES,
        };
        let rows = q(sql)
            .bind(domains_json)
            .bind(keys_json)
            .fetch_all(&self.backend)
            .await?;
        let mut revision = None;
        let mut global_policy = None;
        let mut domain_rows = HashMap::new();
        let mut held_addresses = Vec::new();
        let mut exact_flags = HashMap::new();
        for row in rows {
            match text(&row, "tag")?.as_str() {
                "policy" => {
                    revision =
                        Some(self.settings_revision_from_snapshot(number(&row, "revision")?));
                    global_policy = Some(policy(&row));
                }
                "domain" => {
                    let input = text(&row, "input")?;
                    let value = (|| {
                        Ok(FinalGateDomain {
                            canonical_domain: text(&row, "canonical")?,
                            enabled: number(&row, "enabled")?.map(|value| value != 0),
                            second_alias: row.try_get_opt_string("second_alias")?.is_some(),
                            legacy_read_required: number(&row, "domain_shape")? != Some(1),
                        })
                    })();
                    domain_rows.insert(input, value);
                }
                "hold" => held_addresses.push(text(&row, "held_address")),
                "flag" => {
                    exact_flags.insert(text(&row, "setting_key")?, text(&row, "setting_value"));
                }
                _ => {
                    return Err(StoreError::Corrupt(
                        "unknown final gate snapshot section".into(),
                    ));
                }
            }
        }
        Ok(FinalGateSnapshot {
            revision: revision
                .ok_or_else(|| StoreError::Corrupt("missing final gate revision row".into()))?,
            global_policy: global_policy
                .ok_or_else(|| StoreError::Corrupt("missing final gate policy row".into()))?,
            domains: domain_rows,
            held_addresses,
            exact_flags,
        })
    }
}
