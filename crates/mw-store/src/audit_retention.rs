//! Audit-log retention (t32-ext §14.3 "Retention"; t39 L5).
//!
//! `audit_retention` holds one row per category: `days` (NULL keeps every entry)
//! and `legal_hold` (1 stops pruning). The row `*` is the default for every
//! category without its own row. With no row at all, nothing is pruned: a
//! deployment that never sets a retention keeps its whole audit log.
//!
//! Categories follow `audit_log.category`: `security` and `alert` match their
//! own rows, `admin` matches rows written without a category (administrator
//! actions) and rows named `admin`, and any other category uses `*` only. A
//! legal hold on `*` stops every prune.
//!
//! [`Store::prune_audit_log`] is the only delete path for `audit_log`. It removes
//! rows older than the cutoff in bounded batches and writes an
//! `audit_checkpoint` (`kind = 'prune'`) naming the last removed `seq`/`hash`, so
//! a chain verifier knows where the retained log starts. Pruning is by time and
//! category only: no account, address or actor takes part in the selection.

use crate::backend::q;
use crate::{Store, StoreError};

/// The default row's category.
pub const DEFAULT_CATEGORY: &str = "*";
/// Every category an administrator can set, the default first.
pub const CATEGORIES: [&str; 4] = [DEFAULT_CATEGORY, "admin", "security", "alert"];
/// Bounds of `days` when it is set.
pub const MIN_DAYS: i64 = 30;
pub const MAX_DAYS: i64 = 3650;
/// Rows removed per statement, so one prune never holds a long write lock.
const BATCH: i64 = 2000;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditRetentionRow {
    pub category: String,
    /// `None` keeps every entry.
    pub days: Option<i64>,
    pub legal_hold: bool,
    pub updated_by: String,
    pub updated_at: i64,
}

/// What one prune removed, per category (in [`CATEGORIES`] order without `*`,
/// then `other`).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditPruneReport {
    pub removed: Vec<(String, u64)>,
    pub total: u64,
    /// The checkpoint written, when anything was removed.
    pub checkpoint_id: Option<String>,
}

/// The last prune checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditCheckpoint {
    pub id: String,
    pub pruned_through_id: i64,
    pub pruned_through_hash: Option<String>,
    pub ts: i64,
}

/// `true` when `days` is acceptable (`None` = keep every entry).
pub fn valid_days(days: Option<i64>) -> bool {
    days.is_none_or(|d| (MIN_DAYS..=MAX_DAYS).contains(&d))
}

/// What applies to one category: its own row, else the default row.
fn effective<'a>(rows: &'a [AuditRetentionRow], category: &str) -> Option<&'a AuditRetentionRow> {
    rows.iter()
        .find(|r| r.category == category)
        .or_else(|| rows.iter().find(|r| r.category == DEFAULT_CATEGORY))
}

/// The UTC cutoff as the fixed-width `YYYY-MM-DDTHH:MM:SS` prefix every
/// `audit_log.ts` starts with (RFC 3339 in UTC). Comparing the first 19
/// characters is a plain digit comparison in both dialects and any collation.
fn cutoff(now: i64, days: i64) -> String {
    let at = now.saturating_sub(days.saturating_mul(86_400));
    chrono::DateTime::from_timestamp(at, 0)
        .unwrap_or_default()
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string()
}

impl Store {
    /// Every retention row, in [`CATEGORIES`] order.
    pub async fn audit_retention(&self) -> Result<Vec<AuditRetentionRow>, StoreError> {
        let mut rows: Vec<AuditRetentionRow> =
            q("SELECT category, days, legal_hold, updated_by, updated_at FROM audit_retention")
                .fetch_all(&self.backend)
                .await?
                .iter()
                .map(|r| AuditRetentionRow {
                    category: r.get_string("category"),
                    days: r.get_opt_i64("days"),
                    legal_hold: r.get_i64("legal_hold") != 0,
                    updated_by: r.get_string("updated_by"),
                    updated_at: r.get_i64("updated_at"),
                })
                .collect();
        rows.sort_by_key(|r| {
            CATEGORIES
                .iter()
                .position(|c| *c == r.category)
                .unwrap_or(CATEGORIES.len())
        });
        Ok(rows)
    }

    /// Set one category's retention. Returns `false`, and writes nothing, for an
    /// unknown category or `days` out of bounds.
    pub async fn put_audit_retention(&self, row: &AuditRetentionRow) -> Result<bool, StoreError> {
        if !CATEGORIES.contains(&row.category.as_str()) || !valid_days(row.days) {
            return Ok(false);
        }
        q(
            "INSERT INTO audit_retention (category, days, legal_hold, updated_by, updated_at) \
           VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT (category) DO UPDATE SET \
           days = excluded.days, legal_hold = excluded.legal_hold, \
           updated_by = excluded.updated_by, updated_at = excluded.updated_at",
        )
        .bind(&row.category)
        .bind(row.days)
        .bind(i64::from(row.legal_hold))
        .bind(&row.updated_by)
        .bind(row.updated_at)
        .execute(&self.backend)
        .await?;
        Ok(true)
    }

    /// Remove one category's row (it then follows the default). `true` when a
    /// row was removed.
    pub async fn delete_audit_retention(&self, category: &str) -> Result<bool, StoreError> {
        Ok(q("DELETE FROM audit_retention WHERE category = ?1")
            .bind(category)
            .execute(&self.backend)
            .await?
            > 0)
    }

    /// The newest prune checkpoint.
    pub async fn last_audit_prune(&self) -> Result<Option<AuditCheckpoint>, StoreError> {
        Ok(q(
            "SELECT id, pruned_through_id, pruned_through_hash, ts FROM audit_checkpoint \
              WHERE kind = 'prune' ORDER BY ts DESC, id DESC LIMIT 1",
        )
        .fetch_optional(&self.backend)
        .await?
        .map(|r| AuditCheckpoint {
            id: r.get_string("id"),
            pruned_through_id: r.get_i64("pruned_through_id"),
            pruned_through_hash: r.get_opt_string("pruned_through_hash"),
            ts: r.get_i64("ts"),
        }))
    }

    /// Delete the audit rows the retention rows no longer keep, as of `now`
    /// (unix seconds). Writes a `prune` checkpoint when anything was removed.
    pub async fn prune_audit_log(&self, now: i64) -> Result<AuditPruneReport, StoreError> {
        let rows = self.audit_retention().await?;
        let mut report = AuditPruneReport::default();
        if rows
            .iter()
            .any(|r| r.category == DEFAULT_CATEGORY && r.legal_hold)
        {
            return Ok(report);
        }
        let mut last: Option<(i64, Option<String>)> = None;
        for category in ["admin", "security", "alert", "other"] {
            let policy = effective(
                &rows,
                if category == "other" {
                    DEFAULT_CATEGORY
                } else {
                    category
                },
            );
            let Some(days) = policy.filter(|p| !p.legal_hold).and_then(|p| p.days) else {
                continue;
            };
            let before = cutoff(now, days);
            let mut removed = 0u64;
            loop {
                let (newest, n) = self.prune_batch(category, &before).await?;
                if let Some(seq) = newest
                    && last.as_ref().is_none_or(|(s, _)| seq.0 > *s)
                {
                    last = Some(seq);
                }
                removed += n;
                if n < BATCH as u64 {
                    break;
                }
            }
            if removed > 0 {
                report.removed.push((category.to_string(), removed));
                report.total += removed;
            }
        }
        if report.total > 0 {
            let id = uuid::Uuid::new_v4().to_string();
            let (through, hash) = last.unwrap_or((0, None));
            q("INSERT INTO audit_checkpoint (id, kind, pruned_through_id, pruned_through_hash, ts) \
               VALUES (?1, 'prune', ?2, ?3, ?4)")
            .bind(&id)
            .bind(through)
            .bind(hash)
            .bind(now)
            .execute(&self.backend)
            .await?;
            report.checkpoint_id = Some(id);
        }
        Ok(report)
    }

    /// One bounded delete for `category`; returns the highest removed
    /// `seq`/`hash` (when the rows carry one) and the count.
    async fn prune_batch(
        &self,
        category: &str,
        before: &str,
    ) -> Result<(Option<(i64, Option<String>)>, u64), StoreError> {
        let (newest, delete) = match category {
            "admin" => (
                q(
                    "SELECT seq, hash FROM audit_log WHERE (category IS NULL OR category = 'admin') \
                   AND substr(ts, 1, 19) < ?1 AND seq IS NOT NULL ORDER BY seq DESC LIMIT 1",
                ),
                q(
                    "DELETE FROM audit_log WHERE id IN (SELECT id FROM audit_log \
                   WHERE (category IS NULL OR category = 'admin') AND substr(ts, 1, 19) < ?1 \
                   ORDER BY ts LIMIT ?2)",
                ),
            ),
            "security" | "alert" => (
                q("SELECT seq, hash FROM audit_log WHERE category = ?2 \
                   AND substr(ts, 1, 19) < ?1 AND seq IS NOT NULL ORDER BY seq DESC LIMIT 1"),
                q(
                    "DELETE FROM audit_log WHERE id IN (SELECT id FROM audit_log \
                   WHERE category = ?3 AND substr(ts, 1, 19) < ?1 ORDER BY ts LIMIT ?2)",
                ),
            ),
            _ => (
                q(
                    "SELECT seq, hash FROM audit_log WHERE category IS NOT NULL \
                   AND category NOT IN ('admin', 'security', 'alert') \
                   AND substr(ts, 1, 19) < ?1 AND seq IS NOT NULL ORDER BY seq DESC LIMIT 1",
                ),
                q(
                    "DELETE FROM audit_log WHERE id IN (SELECT id FROM audit_log \
                   WHERE category IS NOT NULL AND category NOT IN ('admin', 'security', 'alert') \
                   AND substr(ts, 1, 19) < ?1 ORDER BY ts LIMIT ?2)",
                ),
            ),
        };
        let named = matches!(category, "security" | "alert");
        let mut newest = newest.bind(before);
        if named {
            newest = newest.bind(category);
        }
        let newest = newest
            .fetch_optional(&self.backend)
            .await?
            .and_then(|r| r.get_opt_i64("seq").map(|s| (s, r.get_opt_string("hash"))));
        let mut delete = delete.bind(before).bind(BATCH);
        if named {
            delete = delete.bind(category);
        }
        let n = delete.execute(&self.backend).await?;
        Ok((newest, n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cutoff_is_the_fixed_width_utc_prefix() {
        assert_eq!(cutoff(86_400 * 31, 30), "1970-01-02T00:00:00");
        let stamp = chrono::DateTime::from_timestamp(86_400, 5)
            .unwrap()
            .to_rfc3339();
        assert_eq!(&stamp[..19], "1970-01-02T00:00:00");
        assert!(valid_days(None) && valid_days(Some(30)) && valid_days(Some(3650)));
        assert!(!valid_days(Some(29)) && !valid_days(Some(3651)) && !valid_days(Some(0)));
    }
}
