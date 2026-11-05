//! Retention sweeper primitives: batched, idempotent deletes over the stores that
//! hold personal or behavioural data and have no retention of their own, plus IP
//! truncation or hashing for older history rows.
//!
//! Each store is a [`Target`]: a table, its key expression, a time column and an
//! optional guard that keeps rows still needed (an unsent submission, a running
//! job). The server's sweeper decides the cut-off per store from settings; this
//! module only deletes `batch` rows at a time so a long sweep never holds a write
//! lock for long, and a repeated sweep finds nothing new to do.
//!
//! Stores that already prune themselves (audit log, authentication history,
//! request metrics, protection history, client timing, session watch, DSR, open
//! tracking, diagnostics) are not repeated here; `docs/admin/retention.md` lists
//! who purges what.

use crate::backend::q;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use crate::{Store, StoreError};
use sha2::{Digest, Sha256};

/// How a target's age is read.
#[derive(Debug, Clone, Copy)]
pub enum Age {
    /// An RFC 3339 text column; compared on its fixed-width UTC prefix.
    Text(&'static str),
    /// A unix-seconds integer column.
    Unix(&'static str),
    /// `rc.h.*` rows in `settings`: the value starts with a 12-digit expiry.
    RcHandle,
}

/// One store the sweeper may delete from.
#[derive(Debug, Clone, Copy)]
pub struct Target {
    /// Stable id used in settings, the API and the docs.
    pub id: &'static str,
    pub table: &'static str,
    /// Key expression, one column or a parenthesised tuple.
    pub pk: &'static str,
    pub age: Age,
    /// Extra condition a row must meet to be deletable (keeps needed rows).
    pub guard: Option<&'static str>,
    /// Column holding the owning account, for legal-hold exclusion.
    pub account_col: Option<&'static str>,
}

/// Every store the sweeper covers.
pub const TARGETS: &[Target] = &[
    Target { id: "passwordChangeAudit", table: "password_change_audit", pk: "id", age: Age::Text("ts"), guard: None, account_col: Some("account_id") },
    Target { id: "ssoLoginAudit", table: "sso_login_audit", pk: "id", age: Age::Text("ts"), guard: None, account_col: None },
    Target { id: "dlpAudit", table: "dlp_audit", pk: "id", age: Age::Text("at"), guard: None, account_col: Some("account_id") },
    Target { id: "assistAudit", table: "assist_audit", pk: "id", age: Age::Text("ts"), guard: None, account_col: None },
    Target { id: "alertEvents", table: "alert_event", pk: "id", age: Age::Unix("fired_at"), guard: None, account_col: None },
    Target { id: "jobRuns", table: "job_run", pk: "id", age: Age::Unix("started_at"), guard: Some("status <> 'running'"), account_col: None },
    Target {
        id: "outbox",
        table: "submissions",
        pk: "id",
        age: Age::Text("created_at"),
        // Only a delivered row (final, no retry or filing error, no backoff) or a
        // cancelled one. Pending, claimed, failed and uncertain rows stay.
        guard: Some("(undo_status IN ('canceled','cancelled') OR (undo_status = 'final' AND last_error IS NULL AND next_attempt_at IS NULL))"),
        account_col: Some("account_id"),
    },
    Target { id: "rememberedDevices", table: "remembered_device", pk: "id", age: Age::Unix("expires_at"), guard: None, account_col: Some("account_id") },
    Target { id: "sessions", table: "sessions", pk: "id", age: Age::Text("last_seen"), guard: None, account_col: Some("account_id") },
    Target { id: "nativeSessions", table: "native_sessions", pk: "token_hash", age: Age::Text("last_seen"), guard: None, account_col: Some("account_id") },
    Target { id: "signinChallenges", table: "signin_challenge", pk: "nonce", age: Age::Unix("expires_at"), guard: None, account_col: None },
    Target { id: "oidcTokens", table: "oidc_token", pk: "token_hash", age: Age::Unix("expires_at"), guard: None, account_col: None },
    Target { id: "oidcCodes", table: "oidc_code", pk: "code_hash", age: Age::Unix("expires_at"), guard: None, account_col: None },
    Target { id: "oauthTokens", table: "oauth_tokens", pk: "token_hash", age: Age::Text("expires_at"), guard: None, account_col: Some("account_id") },
    Target { id: "webhookNonces", table: "webhook_nonce", pk: "(hook_id, nonce)", age: Age::Unix("seen_at"), guard: None, account_col: None },
    Target { id: "sourceFailures", table: "auth_source_failure", pk: "(surface, source_ip)", age: Age::Unix("expires_at"), guard: None, account_col: None },
    Target { id: "imageHandles", table: "settings", pk: "key", age: Age::RcHandle, guard: None, account_col: None },
];

/// The target with `id`.
pub fn target(id: &str) -> Option<&'static Target> {
    TARGETS.iter().find(|t| t.id == id)
}

/// Fixed-width UTC prefix an RFC 3339 text column is compared against.
pub fn text_cutoff(unix: i64) -> String {
    chrono::DateTime::from_timestamp(unix, 0)
        .unwrap_or_default()
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string()
}

/// What one sweep of one target did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Swept {
    /// Rows deleted (or, for a dry run, rows that would be).
    pub rows: u64,
    /// The batch limit was reached with rows left; the next pass continues.
    pub more: bool,
}

/// How a stored IP address is reduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpMode {
    /// IPv4 to its /24, IPv6 to its /48.
    Truncate,
    /// A keyed short hash, `h:<12 hex>`.
    Hash,
}

/// Reduce `ip`. Idempotent: an already reduced value is returned unchanged.
pub fn reduce_ip(ip: &str, mode: IpMode, salt: &str) -> String {
    let ip = ip.trim();
    if ip.is_empty() || ip.starts_with("h:") {
        return ip.to_string();
    }
    match mode {
        IpMode::Hash => {
            let d = Sha256::digest(format!("{salt}|{ip}").as_bytes());
            let hex: String = d.iter().take(6).map(|b| format!("{b:02x}")).collect();
            format!("h:{hex}")
        }
        IpMode::Truncate => {
            if let Ok(v4) = ip.parse::<std::net::Ipv4Addr>() {
                let o = v4.octets();
                return format!("{}.{}.{}.0", o[0], o[1], o[2]);
            }
            if let Ok(v6) = ip.parse::<std::net::Ipv6Addr>() {
                let s = v6.segments();
                return format!("{:x}:{:x}:{:x}::", s[0], s[1], s[2]);
            }
            // Not an address (a label such as "unknown"): leave it.
            ip.to_string()
        }
    }
}

/// The columns a retention-aware IP reduction may touch.
pub const IP_TABLES: &[(&str, &str, Age, Option<&str>)] = &[
    ("signin_event", "id", Age::Unix("ts"), None),
    // Audit rows whose hash chain is in use are left alone.
    ("audit_log", "id", Age::Text("ts"), Some("hash IS NULL")),
];

/// One reduction pass result.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reduced {
    pub rows: u64,
    pub more: bool,
}

/// Tables that prune themselves, whose size the console still shows.
pub const FOOTPRINT_TABLES: &[&str] = &[
    "audit_log",
    "signin_event",
    "request_metrics_bucket",
    "client_timing_bucket",
    "request_protection_event",
    "request_protection_subject",
    "dsr_request",
    "open_events",
    "diag_run",
    "usage_counter",
    "uploaded_blobs",
];

/// Live and total row counts of one store, for the console.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Footprint {
    pub rows: i64,
    /// Bytes on disk when the backend reports it.
    pub bytes: Option<i64>,
}

/// A statement string with `'static` lifetime, interned: the statements are built
/// from a fixed target list and a short account-id list, so the set stays small.
fn stmt(sql: String) -> &'static str {
    static MEMO: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();
    let mut m = MEMO
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(s) = m.get(&sql) {
        return s;
    }
    let leaked: &'static str = Box::leak(sql.clone().into_boxed_str());
    m.insert(sql, leaked);
    leaked
}

fn held_clause(t: &Target, held: &[String]) -> String {
    let (Some(col), false) = (t.account_col, held.is_empty()) else {
        return String::new();
    };
    // Ids are checked here, so inlining them cannot inject anything.
    let ids: Vec<String> = held
        .iter()
        .filter(|id| {
            !id.is_empty()
                && id.len() <= 128
                && id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        })
        .map(|id| format!("'{id}'"))
        .collect();
    if ids.is_empty() {
        return String::new();
    }
    format!(" AND {col} NOT IN ({})", ids.join(","))
}

fn where_clause(t: &Target, held: &[String]) -> String {
    let age = match t.age {
        Age::Text(c) => format!("substr({c}, 1, 19) < ?1"),
        Age::Unix(c) => format!("{c} < ?1"),
        Age::RcHandle => "key LIKE 'rc.h.%' AND substr(value, 1, 12) < ?1".to_string(),
    };
    let guard = t.guard.map(|g| format!(" AND {g}")).unwrap_or_default();
    format!("{age}{guard}{}", held_clause(t, held))
}

impl Store {
    /// Delete rows of `t` older than `cutoff` (unix seconds; for the expiry-based
    /// stores, the moment before which a credential counts as long expired), at
    /// most `batch` per statement and `max_batches` statements. `dry_run` only
    /// counts. `held` are account ids whose rows are never deleted.
    pub async fn retention_sweep(
        &self,
        t: &Target,
        cutoff: i64,
        batch: i64,
        max_batches: u32,
        dry_run: bool,
        held: &[String],
    ) -> Result<Swept, StoreError> {
        let cond = where_clause(t, held);
        let text = matches!(t.age, Age::Text(_));
        let rc = matches!(t.age, Age::RcHandle);
        let bind_cut = |s: crate::backend::Sql| {
            if text {
                s.bind(text_cutoff(cutoff))
            } else if rc {
                s.bind(format!("{:012}", cutoff.max(0)))
            } else {
                s.bind(cutoff)
            }
        };
        if dry_run {
            let sql = format!("SELECT COUNT(*) AS n FROM {} WHERE {cond}", t.table);
            let row = bind_cut(q(stmt(sql.clone()))).fetch_one(&self.backend).await?;
            return Ok(Swept { rows: row.get_i64("n").max(0) as u64, more: false });
        }
        let order = match t.age {
            Age::Text(c) | Age::Unix(c) => c,
            Age::RcHandle => "value",
        };
        let sql = format!(
            "DELETE FROM {tbl} WHERE {pk} IN (SELECT {cols} FROM {tbl} WHERE {cond} ORDER BY {order} LIMIT ?2)",
            tbl = t.table,
            pk = t.pk,
            // A composite key is `(a, b)` outside and `a, b` in the select list.
            cols = t.pk.trim_matches(|c| c == '(' || c == ')')
        );
        let mut total = 0u64;
        let mut more = false;
        for i in 0..max_batches.max(1) {
            let n = bind_cut(q(stmt(sql.clone()))).bind(batch).execute(&self.backend).await?;
            total += n;
            if n < batch as u64 {
                break;
            }
            if i + 1 == max_batches.max(1) {
                more = true;
            }
        }
        Ok(Swept { rows: total, more })
    }

    /// Account ids whose address (`user@host`) or domain (`@host`) is in
    /// `subjects`; their rows are exempt from deletion (legal hold).
    pub async fn retention_held_accounts(
        &self,
        subjects: &[String],
    ) -> Result<Vec<String>, StoreError> {
        let mut ids = Vec::new();
        for s in subjects.iter().take(500) {
            let s = s.trim().to_ascii_lowercase();
            if s.is_empty() || s.contains(['%', '_']) {
                continue;
            }
            let rows = if let Some(domain) = s.strip_prefix('@') {
                q("SELECT id FROM accounts WHERE lower(username) LIKE ?1")
                    .bind(format!("%@{domain}"))
                    .fetch_all(&self.backend)
                    .await?
            } else {
                q("SELECT id FROM accounts WHERE lower(username) = ?1")
                    .bind(s)
                    .fetch_all(&self.backend)
                    .await?
            };
            ids.extend(rows.iter().map(|r| r.get_string("id")));
        }
        ids.sort();
        ids.dedup();
        Ok(ids)
    }

    /// Row count and size of one of the tables other retention code owns, for the
    /// console. Only names in [`FOOTPRINT_TABLES`] are accepted.
    pub async fn retention_table_footprint(&self, table: &str) -> Result<Option<Footprint>, StoreError> {
        let Some(table) = FOOTPRINT_TABLES.iter().find(|t| **t == table) else {
            return Ok(None);
        };
        let rows = q(stmt(format!("SELECT COUNT(*) AS n FROM {table}")))
            .fetch_one(&self.backend)
            .await?
            .get_i64("n");
        Ok(Some(Footprint { rows, bytes: self.table_bytes(table).await }))
    }

    /// Row count and, where the backend reports it, size of `t`'s table.
    pub async fn retention_footprint(&self, t: &Target) -> Result<Footprint, StoreError> {
        let filter = if matches!(t.age, Age::RcHandle) {
            " WHERE key LIKE 'rc.h.%'"
        } else {
            ""
        };
        let rows = q(stmt(format!("SELECT COUNT(*) AS n FROM {}{filter}", t.table)))
            .fetch_one(&self.backend)
            .await?
            .get_i64("n");
        // A shared table's size is not the store's size.
        let bytes = if matches!(t.age, Age::RcHandle) {
            None
        } else {
            self.table_bytes(t.table).await
        };
        Ok(Footprint { rows, bytes })
    }

    async fn table_bytes(&self, table: &str) -> Option<i64> {
        match &self.backend {
            crate::backend::Backend::Postgres(_) => {
                let r = q("SELECT pg_total_relation_size(?1::regclass) AS n")
                    .bind(table)
                    .fetch_one(&self.backend)
                    .await
                    .ok()?;
                Some(r.get_i64("n"))
            }
            crate::backend::Backend::Sqlite(_) => {
                let r = q("SELECT COALESCE(SUM(pgsize), 0) AS n FROM dbstat WHERE name = ?1")
                    .bind(table)
                    .fetch_one(&self.backend)
                    .await
                    .ok()?;
                Some(r.get_i64("n"))
            }
        }
    }

    /// Reduce the IP of `table` rows older than `cutoff`, at most `batch` rows per
    /// pass (keyset order, so a pass never revisits what it has seen). `dry_run`
    /// counts rows that still carry a full address.
    #[allow(clippy::too_many_arguments)]
    pub async fn retention_reduce_ips(
        &self,
        table: &str,
        cutoff: i64,
        mode: IpMode,
        salt: &str,
        batch: i64,
        max_batches: u32,
        dry_run: bool,
    ) -> Result<Reduced, StoreError> {
        let Some((table, pk, age, guard)) = IP_TABLES.iter().find(|t| t.0 == table).copied()
        else {
            return Ok(Reduced::default());
        };
        let (age_sql, text) = match age {
            Age::Text(c) => (format!("substr({c}, 1, 19) < ?1"), true),
            Age::Unix(c) => (format!("{c} < ?1"), false),
            Age::RcHandle => return Ok(Reduced::default()),
        };
        let guard = guard.map(|g| format!(" AND {g}")).unwrap_or_default();
        let base = format!("{age_sql}{guard} AND ip IS NOT NULL AND ip <> '' AND ip NOT LIKE 'h:%'");
        let cut = |s: crate::backend::Sql| {
            if text { s.bind(text_cutoff(cutoff)) } else { s.bind(cutoff) }
        };
        let select = format!(
            "SELECT {pk} AS k, ip FROM {table} WHERE {base} AND {pk} > ?2 ORDER BY {pk} LIMIT ?3"
        );
        let update = format!("UPDATE {table} SET ip = ?1 WHERE {pk} = ?2 AND ip = ?3");
        let mut last = String::new();
        let mut changed = 0u64;
        let mut more = false;
        for i in 0..max_batches.max(1) {
            let rows = cut(q(stmt(select.clone())))
                .bind(&last)
                .bind(batch)
                .fetch_all(&self.backend)
                .await?;
            for r in &rows {
                let key = r.get_string("k");
                let old = r.get_string("ip");
                let new = reduce_ip(&old, mode, salt);
                last = key.clone();
                if new == old {
                    continue;
                }
                if dry_run {
                    changed += 1;
                } else {
                    changed += q(stmt(update.clone()))
                        .bind(new)
                        .bind(key)
                        .bind(old)
                        .execute(&self.backend)
                        .await?;
                }
            }
            if (rows.len() as i64) < batch {
                break;
            }
            if i + 1 == max_batches.max(1) {
                more = true;
            }
        }
        Ok(Reduced { rows: changed, more })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ip_reduction_is_idempotent() {
        assert_eq!(reduce_ip("203.0.113.77", IpMode::Truncate, ""), "203.0.113.0");
        assert_eq!(
            reduce_ip("2001:db8:abcd:1234::9", IpMode::Truncate, ""),
            "2001:db8:abcd::"
        );
        assert_eq!(reduce_ip("203.0.113.0", IpMode::Truncate, ""), "203.0.113.0");
        assert_eq!(reduce_ip("unknown", IpMode::Truncate, ""), "unknown");
        let h = reduce_ip("203.0.113.77", IpMode::Hash, "s");
        assert!(h.starts_with("h:") && h.len() == 14);
        assert_eq!(reduce_ip(&h, IpMode::Hash, "s"), h);
        assert_ne!(h, reduce_ip("203.0.113.78", IpMode::Hash, "s"));
    }

    #[test]
    fn text_cutoff_is_fixed_width() {
        assert_eq!(text_cutoff(86_400), "1970-01-02T00:00:00");
    }

    #[test]
    fn target_ids_are_unique_and_held_ids_are_checked() {
        let mut ids: Vec<_> = TARGETS.iter().map(|t| t.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), TARGETS.len());
        let t = target("outbox").unwrap();
        assert!(held_clause(t, &["a'; DROP".into()]).is_empty());
        assert!(held_clause(t, &["acct-1".into()]).contains("NOT IN ('acct-1')"));
    }
}
