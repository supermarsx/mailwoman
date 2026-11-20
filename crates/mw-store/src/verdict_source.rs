//! Fresh owner/content witnesses for security verdicts; never an admission cache.
use crate::{
    Store, StoreError,
    backend::{Dialect, Tx, q},
};
use std::{collections::HashMap, sync::OnceLock};

// Six finite query shapes per dialect; no caller data enters these strings.
fn fixed_sql(kind: usize, dialect: Dialect, build: impl FnOnce() -> String) -> &'static str {
    static QUERIES: [OnceLock<String>; 12] = [const { OnceLock::new() }; 12];
    QUERIES[kind * 2 + usize::from(dialect == Dialect::Postgres)]
        .get_or_init(build)
        .as_str()
}

const MAX_RAW: usize = 64 * 1024 * 1024;
const MAX_VERDICT: usize = 1024 * 1024;
/// Only Store can mint this witness after an owned bounded raw read.
pub struct VerdictSource {
    scope: uuid::Uuid,
    account: String,
    email: String,
    revision: String,
    blob: String,
}
impl std::fmt::Debug for VerdictSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("VerdictSource([redacted])")
    }
}
impl VerdictSource {
    pub fn email_id(&self) -> &str {
        &self.email
    }
    pub fn matches_account(&self, account: &str) -> bool {
        self.account == account
    }
}
const CURRENT: &str = "m.account_id=?1 AND m.stable_id=?2 AND m.content_revision=?3 AND m.blob_ref=?4 AND (z.account_id IS NULL OR z.enabled=0)";
impl Store {
    pub async fn security_verdict_source(
        &self,
        account: &str,
        email: &str,
    ) -> Result<Option<(VerdictSource, Vec<u8>)>, StoreError> {
        if account.len() > 1024 || email.len() > 1024 {
            return Err(StoreError::Invalid("verdict source key too large"));
        }
        let shape = if self.backend.dialect() == Dialect::Sqlite {
            "typeof(m.content_revision)='text' AND typeof(m.blob_ref)='text' AND typeof(b.sealed_bytes)='blob'"
        } else {
            "TRUE"
        };
        let sql = fixed_sql(0, self.backend.dialect(), || {
            format!(
                "SELECT m.content_revision,m.blob_ref,CASE WHEN octet_length(b.sealed_bytes)<=?3 AND ({shape}) THEN b.sealed_bytes ELSE NULL END AS raw,CAST(CASE WHEN octet_length(b.sealed_bytes)<=?3 AND ({shape}) THEN 0 ELSE 1 END AS BIGINT) AS invalid FROM messages m JOIN accounts a ON a.id=m.account_id JOIN bodies b ON b.account_id=m.account_id AND b.blob_ref=m.blob_ref LEFT JOIN zeroaccess_accounts z ON z.account_id=m.account_id WHERE m.account_id=?1 AND m.stable_id=?2 AND (z.account_id IS NULL OR z.enabled=0)"
            )
        });
        let Some(row) = q(sql)
            .bind(account)
            .bind(email)
            .bind((MAX_RAW + 40) as i64)
            .fetch_optional(&self.backend)
            .await?
        else {
            return Ok(None);
        };
        if row.get_i64("invalid") != 0 {
            return Err(StoreError::Corrupt(
                "invalid or oversized verdict source".into(),
            ));
        }
        let revision = row
            .try_get_opt_string("content_revision")?
            .ok_or_else(|| StoreError::Corrupt("missing verdict source epoch".into()))?;
        let blob = row
            .try_get_opt_string("blob_ref")?
            .ok_or_else(|| StoreError::Corrupt("missing verdict source blob".into()))?;
        if revision.len() > 128 || blob.len() > 1024 {
            return Err(StoreError::Corrupt(
                "invalid verdict source identity".into(),
            ));
        }
        let raw = self.key.open(&row.get_blob("raw"))?;
        if raw.len() > MAX_RAW {
            return Err(StoreError::Corrupt("oversized verdict source".into()));
        }
        Ok(Some((
            VerdictSource {
                scope: self.cache_scope(),
                account: account.into(),
                email: email.into(),
                revision,
                blob,
            },
            raw,
        )))
    }
    pub async fn security_verdict_source_current(
        &self,
        source: &VerdictSource,
    ) -> Result<bool, StoreError> {
        if source.scope != self.cache_scope() {
            return Ok(false);
        }
        let sql = fixed_sql(1, self.backend.dialect(), || {
            format!(
                "SELECT m.stable_id FROM messages m JOIN accounts a ON a.id=m.account_id LEFT JOIN zeroaccess_accounts z ON z.account_id=m.account_id WHERE {CURRENT}"
            )
        });
        Ok(q(sql)
            .bind(&source.account)
            .bind(&source.email)
            .bind(&source.revision)
            .bind(&source.blob)
            .fetch_optional(&self.backend)
            .await?
            .is_some())
    }
    pub async fn security_verdict_cached(
        &self,
        source: &VerdictSource,
        hash: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        if source.scope != self.cache_scope() {
            return Ok(None);
        }
        let shape = if self.backend.dialect() == Dialect::Sqlite {
            "typeof(v.verdict_json)='blob'"
        } else {
            "TRUE"
        };
        let sql = fixed_sql(2, self.backend.dialect(), || {
            format!(
                "SELECT CASE WHEN octet_length(v.verdict_json)<=?6 AND ({shape}) THEN v.verdict_json ELSE NULL END AS verdict_json FROM security_verdicts v JOIN messages m ON m.stable_id=v.email_id AND m.account_id=v.account_id JOIN accounts a ON a.id=m.account_id LEFT JOIN zeroaccess_accounts z ON z.account_id=m.account_id WHERE {CURRENT} AND v.source_revision=m.content_revision AND v.raw_hash=?5"
            )
        });
        let row = q(sql)
            .bind(&source.account)
            .bind(&source.email)
            .bind(&source.revision)
            .bind(&source.blob)
            .bind(hash)
            .bind(MAX_VERDICT as i64)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.and_then(|row| row.get_opt_blob("verdict_json")))
    }
    /// Lock the message before checking the epoch; body mutation triggers lock
    /// the same row, so a later body change cannot resurrect this old cache.
    pub async fn security_verdict_cache_if_current(
        &self,
        source: &VerdictSource,
        hash: &str,
        bytes: &[u8],
    ) -> Result<bool, StoreError> {
        if source.scope != self.cache_scope() || bytes.len() > MAX_VERDICT {
            return Ok(false);
        }
        let mut tx = self.backend.begin().await?;
        crate::header_envelope::lock_parent(&mut tx, &source.account).await?;
        let lock = if matches!(tx, Tx::Postgres(_)) {
            " FOR UPDATE OF m"
        } else {
            ""
        };
        let sql = fixed_sql(3, self.backend.dialect(), || {
            format!(
                "SELECT m.stable_id FROM messages m JOIN accounts a ON a.id=m.account_id LEFT JOIN zeroaccess_accounts z ON z.account_id=m.account_id WHERE {CURRENT}{lock}"
            )
        });
        if q(sql)
            .bind(&source.account)
            .bind(&source.email)
            .bind(&source.revision)
            .bind(&source.blob)
            .fetch_all_tx(&mut tx)
            .await?
            .is_empty()
        {
            tx.commit().await?;
            return Ok(false);
        }
        q("INSERT INTO security_verdicts(email_id,account_id,raw_hash,verdict_json,computed_at,source_revision) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(email_id) DO UPDATE SET account_id=excluded.account_id,raw_hash=excluded.raw_hash,verdict_json=excluded.verdict_json,computed_at=excluded.computed_at,source_revision=excluded.source_revision")
            .bind(&source.email).bind(&source.account).bind(hash).bind(bytes).bind(chrono::Utc::now().to_rfc3339()).bind(&source.revision).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(true)
    }
    /// Batch summaries are joined to current ownership/content and optionally
    /// the exact message rows used for the response, never an old bare-ID cache.
    pub async fn security_verdicts_for_rows(
        &self,
        account: &str,
        rows: &[(String, String)],
    ) -> Result<HashMap<String, Vec<u8>>, StoreError> {
        self.current_verdicts(account, rows, true).await
    }
    pub(crate) async fn current_verdicts(
        &self,
        account: &str,
        rows: &[(String, String)],
        exact: bool,
    ) -> Result<HashMap<String, Vec<u8>>, StoreError> {
        if rows.len() > 4096
            || account.len() > 1024
            || rows
                .iter()
                .any(|(id, rev)| id.len() > 1024 || rev.len() > 128)
        {
            return Err(StoreError::Invalid("verdict batch limit exceeded"));
        }
        if rows.is_empty() {
            return Ok(HashMap::new());
        }
        let input = match self.backend.dialect() {
            Dialect::Sqlite => {
                "SELECT json_extract(value,'$[0]') AS id,json_extract(value,'$[1]') AS revision FROM json_each(?2)"
            }
            Dialect::Postgres => {
                "SELECT value->>0 AS id,value->>1 AS revision FROM json_array_elements(?2::json)"
            }
        };
        let kind = 4 + usize::from(exact);
        let exact = if exact {
            "AND m.cache_revision=r.revision"
        } else {
            ""
        };
        let shape = if self.backend.dialect() == Dialect::Sqlite {
            "typeof(v.verdict_json)='blob'"
        } else {
            "TRUE"
        };
        // Materialize only eligible IDs and metadata lengths before the
        // aggregate guard; no verdict BLOB is selected into this CTE.
        let sql = fixed_sql(kind, self.backend.dialect(), || {
            format!(
                "WITH requested AS ({input}), eligible AS (SELECT DISTINCT m.stable_id,octet_length(v.verdict_json) AS bytes FROM requested r JOIN messages m ON m.stable_id=r.id AND m.account_id=?1 JOIN accounts a ON a.id=m.account_id JOIN security_verdicts v ON v.email_id=m.stable_id AND v.account_id=m.account_id AND v.source_revision=m.content_revision LEFT JOIN zeroaccess_accounts z ON z.account_id=m.account_id WHERE (z.account_id IS NULL OR z.enabled=0) {exact}), budget AS (SELECT CASE WHEN COALESCE(SUM(bytes),0)<=?4 THEN 1 ELSE 0 END AS ok FROM eligible) SELECT '' AS stable_id,NULL AS verdict_json,CAST(1 AS BIGINT) AS oversized FROM budget WHERE ok=0 UNION ALL SELECT e.stable_id,CASE WHEN e.bytes<=?3 AND ({shape}) THEN v.verdict_json ELSE NULL END,CAST(0 AS BIGINT) FROM budget CROSS JOIN eligible e JOIN security_verdicts v ON v.email_id=e.stable_id AND v.account_id=?1 WHERE budget.ok=1"
            )
        });
        let found = q(sql)
            .bind(account)
            .bind(serde_json::to_string(rows).expect("bounded string tuples"))
            .bind(MAX_VERDICT as i64)
            .bind((8 * 1024 * 1024) as i64)
            .fetch_all(&self.backend)
            .await?;
        let mut output = HashMap::new();
        for row in found {
            if row.get_i64("oversized") != 0 {
                return Err(StoreError::Corrupt(
                    "verdict batch byte budget exceeded".into(),
                ));
            }
            if let Some(bytes) = row.get_opt_blob("verdict_json") {
                output.insert(row.get_string("stable_id"), bytes);
            }
        }
        Ok(output)
    }
}
#[cfg(test)]
#[path = "verdict_source_tests.rs"]
mod tests;
