//! Cookie consent storage: the consent log, admin classification rules and the
//! names of browser storage the scan found that nothing classifies yet.
//!
//! Only names, counts and category decisions are stored. No cookie or storage
//! value, IP address or user agent reaches this module. A consent row carries the
//! account id only when the person was signed in when they decided.
use crate::{Store, StoreError, q};
use serde::{Deserialize, Serialize};

/// Most unclassified names kept; further new names are dropped, so a page that
/// invents names cannot grow the table.
pub const MAX_DETECTED: i64 = 500;
/// Most consent rows one export or statistics read returns.
pub const MAX_CONSENT_ROWS: usize = 200_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsentRow {
    pub id: String,
    pub account_id: Option<String>,
    pub domain: String,
    pub policy_version: i64,
    pub decision: String,
    pub categories_json: String,
    pub region: String,
    pub signal: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CookieRule {
    pub id: String,
    pub kind: String,
    pub pattern: String,
    pub category: String,
    pub note: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectedStorage {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub first_seen: i64,
    pub last_seen: i64,
    pub seen_count: i64,
    pub status: String,
}

fn consent(row: &crate::backend::Row) -> ConsentRow {
    ConsentRow {
        id: row.get_string("id"),
        account_id: row.get_opt_string("account_id"),
        domain: row.get_string("domain"),
        policy_version: row.get_i64("policy_version"),
        decision: row.get_string("decision"),
        categories_json: row.get_string("categories_json"),
        region: row.get_string("region"),
        signal: row.get_string("signal"),
        created_at: row.get_i64("created_at"),
    }
}

impl Store {
    pub async fn record_cookie_consent(&self, row: &ConsentRow) -> Result<(), StoreError> {
        q("INSERT INTO cookie_consent(id,account_id,domain,policy_version,decision,categories_json,region,signal,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)")
            .bind(row.id.as_str())
            .bind(row.account_id.clone())
            .bind(row.domain.as_str())
            .bind(row.policy_version)
            .bind(row.decision.as_str())
            .bind(row.categories_json.as_str())
            .bind(row.region.as_str())
            .bind(row.signal.as_str())
            .bind(row.created_at)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// The newest consent a signed-in person gave, if any.
    pub async fn latest_cookie_consent(
        &self,
        account_id: &str,
    ) -> Result<Option<ConsentRow>, StoreError> {
        let row = q("SELECT id,account_id,domain,policy_version,decision,categories_json,region,signal,created_at FROM cookie_consent WHERE account_id=?1 ORDER BY created_at DESC, id DESC LIMIT 1")
            .bind(account_id)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.as_ref().map(consent))
    }

    /// Consent rows with `from <= created_at < to`, oldest first, at most
    /// [`MAX_CONSENT_ROWS`] (the second value is true when more existed).
    pub async fn cookie_consents_between(
        &self,
        from: i64,
        to: i64,
    ) -> Result<(Vec<ConsentRow>, bool), StoreError> {
        let rows = q("SELECT id,account_id,domain,policy_version,decision,categories_json,region,signal,created_at FROM cookie_consent WHERE created_at>=?1 AND created_at<?2 ORDER BY created_at,id LIMIT 200001")
            .bind(from)
            .bind(to)
            .fetch_all(&self.backend)
            .await?;
        let truncated = rows.len() > MAX_CONSENT_ROWS;
        Ok((
            rows.iter().take(MAX_CONSENT_ROWS).map(consent).collect(),
            truncated,
        ))
    }

    /// Delete consent rows older than `before`; returns the number removed.
    pub async fn purge_cookie_consents(&self, before: i64) -> Result<u64, StoreError> {
        Ok(q("DELETE FROM cookie_consent WHERE created_at<?1")
            .bind(before)
            .execute(&self.backend)
            .await?)
    }

    /// Domains with consent records, including the empty domain for anonymous defaults.
    pub async fn cookie_consent_domains(&self) -> Result<Vec<String>, StoreError> {
        let rows = q("SELECT DISTINCT domain FROM cookie_consent")
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(|row| row.get_string("domain")).collect())
    }

    pub async fn purge_domain_cookie_consents(
        &self,
        domain: &str,
        before: i64,
    ) -> Result<u64, StoreError> {
        Ok(q("DELETE FROM cookie_consent WHERE domain=?1 AND created_at<?2")
            .bind(domain)
            .bind(before)
            .execute(&self.backend)
            .await?)
    }

    /// Delete every consent row of one account (an erasure request).
    pub async fn delete_cookie_consents_for_account(
        &self,
        account_id: &str,
    ) -> Result<u64, StoreError> {
        Ok(q("DELETE FROM cookie_consent WHERE account_id=?1")
            .bind(account_id)
            .execute(&self.backend)
            .await?)
    }

    pub async fn list_cookie_rules(&self) -> Result<Vec<CookieRule>, StoreError> {
        let rows = q("SELECT id,kind,pattern,category,note,created_at FROM cookie_rule ORDER BY kind,pattern")
            .fetch_all(&self.backend)
            .await?;
        Ok(rows
            .iter()
            .map(|r| CookieRule {
                id: r.get_string("id"),
                kind: r.get_string("kind"),
                pattern: r.get_string("pattern"),
                category: r.get_string("category"),
                note: r.get_string("note"),
                created_at: r.get_i64("created_at"),
            })
            .collect())
    }

    /// Add a rule, or change the category and note of the rule with the same kind
    /// and pattern.
    pub async fn upsert_cookie_rule(&self, rule: &CookieRule) -> Result<(), StoreError> {
        q("INSERT INTO cookie_rule(id,kind,pattern,category,note,created_at) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(kind,pattern) DO UPDATE SET category=excluded.category,note=excluded.note")
            .bind(rule.id.as_str())
            .bind(rule.kind.as_str())
            .bind(rule.pattern.as_str())
            .bind(rule.category.as_str())
            .bind(rule.note.as_str())
            .bind(rule.created_at)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    pub async fn delete_cookie_rule(&self, id: &str) -> Result<bool, StoreError> {
        Ok(q("DELETE FROM cookie_rule WHERE id=?1")
            .bind(id)
            .execute(&self.backend)
            .await?
            > 0)
    }

    /// Note that a browser reported `name`. A new name becomes `unclassified`
    /// unless [`MAX_DETECTED`] names are already kept; a known name only has its
    /// count and last-seen time updated (a reviewed `ignored` status stays).
    pub async fn record_detected_storage(
        &self,
        id: &str,
        kind: &str,
        name: &str,
        now: i64,
    ) -> Result<(), StoreError> {
        let updated = q("UPDATE cookie_detected SET last_seen=?3,seen_count=seen_count+1 WHERE kind=?1 AND name=?2")
            .bind(kind)
            .bind(name)
            .bind(now)
            .execute(&self.backend)
            .await?;
        if updated > 0 {
            return Ok(());
        }
        let held = q("SELECT COUNT(*) AS n FROM cookie_detected")
            .fetch_one(&self.backend)
            .await?
            .get_i64("n");
        if held >= MAX_DETECTED {
            return Ok(());
        }
        q("INSERT INTO cookie_detected(id,kind,name,first_seen,last_seen,seen_count,status) VALUES(?1,?2,?3,?4,?4,1,'unclassified') ON CONFLICT(kind,name) DO NOTHING")
            .bind(id)
            .bind(kind)
            .bind(name)
            .bind(now)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    pub async fn list_detected_storage(&self) -> Result<Vec<DetectedStorage>, StoreError> {
        let rows = q("SELECT id,kind,name,first_seen,last_seen,seen_count,status FROM cookie_detected ORDER BY last_seen DESC, name LIMIT 501")
            .fetch_all(&self.backend)
            .await?;
        Ok(rows
            .iter()
            .map(|r| DetectedStorage {
                id: r.get_string("id"),
                kind: r.get_string("kind"),
                name: r.get_string("name"),
                first_seen: r.get_i64("first_seen"),
                last_seen: r.get_i64("last_seen"),
                seen_count: r.get_i64("seen_count"),
                status: r.get_string("status"),
            })
            .collect())
    }

    /// One detected name by id, so a classification can become a rule.
    pub async fn get_detected_storage(
        &self,
        id: &str,
    ) -> Result<Option<DetectedStorage>, StoreError> {
        Ok(self
            .list_detected_storage()
            .await?
            .into_iter()
            .find(|d| d.id == id))
    }

    pub async fn set_detected_storage_status(
        &self,
        id: &str,
        status: &str,
    ) -> Result<bool, StoreError> {
        Ok(q("UPDATE cookie_detected SET status=?2 WHERE id=?1")
            .bind(id)
            .bind(status)
            .execute(&self.backend)
            .await?
            > 0)
    }

    pub async fn delete_detected_storage(&self, id: &str) -> Result<bool, StoreError> {
        Ok(q("DELETE FROM cookie_detected WHERE id=?1")
            .bind(id)
            .execute(&self.backend)
            .await?
            > 0)
    }
}
