//! Bounded, credential-free dashboard reads. No probes, upstream access or writes.
use crate::settings_policy::normalize_policy_domain;
use crate::{Backend, Store, StoreError, q};
use std::collections::{BTreeMap, BTreeSet};

pub const ROW_LIMIT: usize = 5_000;
pub const AUDIT_LIMIT: usize = 1_000;
#[derive(Clone)]
pub struct DashboardDomain {
    pub name: String,
    pub enabled: bool,
    pub preferred: bool,
    pub configured: bool,
}
pub struct DashboardAudit {
    pub id: String,
    pub ts: String,
    pub actor: String,
    pub actor_kind: String,
    pub action: String,
    pub target: Option<String>,
    pub detail: String,
}
pub struct DashboardSnapshot {
    pub domains: Vec<DashboardDomain>,
    pub aliases: BTreeMap<String, String>,
    pub provisioned_domains: Vec<String>,
    pub signed_in_24h: Vec<String>,
    pub signed_in_7d: Vec<String>,
    pub audit: Vec<DashboardAudit>,
    pub active_admin_sessions: Option<u64>,
    pub enabled_admins: Option<u64>,
    pub limited: BTreeSet<&'static str>,
}
impl DashboardSnapshot {
    /// Same explicit-alias identity as the store policy resolver, using this
    /// snapshot's bounded map instead of one database query per account.
    pub fn canonical_domain(&self, raw: &str) -> Option<String> {
        if raw.is_empty() {
            return Some(String::new());
        }
        let mut domain = normalize_policy_domain(raw).ok()?;
        let mut seen = BTreeSet::new();
        for _ in 0..16 {
            if !seen.insert(domain.clone()) {
                return None;
            }
            match self.aliases.get(&domain) {
                Some(next) => domain = normalize_policy_domain(next).ok()?,
                None => return Some(domain),
            }
        }
        None
    }
    pub fn address_domain(&self, address: &str) -> Option<String> {
        self.canonical_domain(address.rsplit_once('@').map_or("", |(_, domain)| domain))
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreDiagnostics {
    pub backend: &'static str,
    pub pool_size: u32,
    pub pool_idle: usize,
    pub db_bytes: Option<u64>,
    pub wal_bytes: Option<u64>,
    pub migration_applied: i64,
    pub migration_expected: usize,
    pub db_ping_ms: f64,
}

impl Store {
    pub async fn dashboard_snapshot(
        &self,
        now: i64,
        global: bool,
    ) -> Result<DashboardSnapshot, StoreError> {
        let mut limited = BTreeSet::new();
        let rows=q("SELECT name,enabled,preferred,CAST(CASE WHEN upstream_json IN ('{}','null','') THEN 0 ELSE 1 END AS BIGINT) AS configured FROM domains ORDER BY name LIMIT ?1").bind((ROW_LIMIT+1) as i64).fetch_all(&self.backend).await?;
        if rows.len() > ROW_LIMIT {
            limited.insert("domains");
        }
        let domains = rows
            .into_iter()
            .take(ROW_LIMIT)
            .map(|r| DashboardDomain {
                name: r.get_string("name"),
                enabled: r.get_i64("enabled") != 0,
                preferred: r.get_i64("preferred") != 0,
                configured: r.get_i64("configured") != 0,
            })
            .collect();
        let rows = q("SELECT alias,domain FROM domain_alias ORDER BY alias LIMIT ?1")
            .bind((ROW_LIMIT + 1) as i64)
            .fetch_all(&self.backend)
            .await?;
        if rows.len() > ROW_LIMIT {
            limited.insert("aliases");
        }
        let aliases = rows
            .into_iter()
            .take(ROW_LIMIT)
            .map(|r| (r.get_string("alias"), r.get_string("domain")))
            .collect();
        let rows = q("SELECT domain,deleted_at FROM mail_user ORDER BY address LIMIT ?1")
            .bind((ROW_LIMIT + 1) as i64)
            .fetch_all(&self.backend)
            .await?;
        if rows.len() > ROW_LIMIT {
            limited.insert("provisionedUsers");
        }
        let provisioned_domains = rows
            .into_iter()
            .take(ROW_LIMIT)
            .filter(|r| r.get_opt_i64("deleted_at").is_none())
            .map(|r| r.get_string("domain"))
            .collect();
        let signins = q("SELECT address,seen_at FROM signin_identity ORDER BY account_id LIMIT ?1")
            .bind((ROW_LIMIT + 1) as i64)
            .fetch_all(&self.backend)
            .await?;
        if signins.len() > ROW_LIMIT {
            limited.insert("signins");
        }
        let signed_in_24h = signins
            .iter()
            .take(ROW_LIMIT)
            .filter(|r| r.get_i64("seen_at") >= now.saturating_sub(86400))
            .map(|r| r.get_string("address"))
            .collect();
        let signed_in_7d = signins
            .iter()
            .take(ROW_LIMIT)
            .filter(|r| r.get_i64("seen_at") >= now.saturating_sub(7 * 86400))
            .map(|r| r.get_string("address"))
            .collect();
        let rows = if global {
            q("SELECT id,ts,actor,actor_kind,action,target,detail_json FROM audit_log ORDER BY ts DESC LIMIT ?1").bind((AUDIT_LIMIT+1) as i64).fetch_all(&self.backend).await?
        } else {
            Vec::new()
        };
        if rows.len() > AUDIT_LIMIT {
            limited.insert("audit");
        }
        let audit = rows
            .into_iter()
            .take(AUDIT_LIMIT)
            .map(|r| DashboardAudit {
                id: r.get_string("id"),
                ts: r.get_string("ts"),
                actor: r.get_string("actor"),
                actor_kind: r.get_string("actor_kind"),
                action: r.get_string("action"),
                target: r.get_opt_string("target"),
                detail: r.get_string("detail_json"),
            })
            .collect();
        let admin_count=q("SELECT COUNT(*) AS total, COALESCE(SUM(CASE WHEN expires_at>?1 AND absolute_expires_at>?1 THEN 1 ELSE 0 END),0) AS live FROM (SELECT expires_at,absolute_expires_at FROM admin_sessions LIMIT ?2) dashboard_rows").bind(now).bind((ROW_LIMIT+1) as i64).fetch_one(&self.backend).await?;
        let active_admin_sessions = (admin_count.get_i64("total") <= ROW_LIMIT as i64)
            .then_some(admin_count.get_i64("live") as u64);
        let admins=q("SELECT COUNT(*) AS total, COALESCE(SUM(CASE WHEN disabled=0 THEN 1 ELSE 0 END),0) AS live FROM (SELECT disabled FROM admin_account LIMIT ?1) dashboard_rows").bind((ROW_LIMIT+1) as i64).fetch_one(&self.backend).await?;
        let enabled_admins =
            (admins.get_i64("total") <= ROW_LIMIT as i64).then_some(admins.get_i64("live") as u64);
        Ok(DashboardSnapshot {
            domains,
            aliases,
            provisioned_domains,
            signed_in_24h,
            signed_in_7d,
            audit,
            active_admin_sessions,
            enabled_admins,
            limited,
        })
    }

    pub async fn store_diagnostics(&self) -> Result<StoreDiagnostics, StoreError> {
        let start = std::time::Instant::now();
        q("SELECT 1").fetch_one(&self.backend).await?;
        let db_ping_ms = start.elapsed().as_secs_f64() * 1000.0;
        let migration_applied = q("SELECT COUNT(*) FROM _sqlx_migrations WHERE success=TRUE")
            .fetch_scalar_i64(&self.backend)
            .await?;
        let (backend, pool_size, pool_idle, db_bytes, wal_bytes, migration_expected) =
            match &self.backend {
                Backend::Sqlite(pool) => {
                    let rows = q("PRAGMA database_list").fetch_all(&self.backend).await?;
                    let path = rows
                        .iter()
                        .find(|r| r.get_string("name") == "main")
                        .map(|r| r.get_string("file"))
                        .filter(|p| !p.is_empty());
                    let db = path
                        .as_ref()
                        .and_then(|p| std::fs::metadata(p).ok())
                        .map(|m| m.len());
                    let wal =
                        path.as_ref()
                            .and_then(|p| match std::fs::metadata(format!("{p}-wal")) {
                                Ok(m) => Some(m.len()),
                                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Some(0),
                                Err(_) => None,
                            });
                    (
                        "sqlite",
                        pool.size(),
                        pool.num_idle(),
                        db,
                        wal,
                        sqlx::migrate!("./migrations").migrations.len(),
                    )
                }
                Backend::Postgres(pool) => {
                    let size = q("SELECT pg_database_size(current_database())")
                        .fetch_scalar_i64(&self.backend)
                        .await?;
                    (
                        "postgres",
                        pool.size(),
                        pool.num_idle(),
                        u64::try_from(size).ok(),
                        None,
                        sqlx::migrate!("./migrations_pg").migrations.len(),
                    )
                }
            };
        Ok(StoreDiagnostics {
            backend,
            pool_size,
            pool_idle,
            db_bytes,
            wal_bytes,
            migration_applied,
            migration_expected,
            db_ping_ms,
        })
    }
}

/// One migration as the database records it.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationEntry {
    pub version: i64,
    pub description: String,
}

/// The applied migration set against the one compiled into this build (t103).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationDetail {
    /// Rows in `_sqlx_migrations` that completed.
    pub applied: usize,
    /// Migrations compiled into this build.
    pub expected: usize,
    /// The newest completed migration.
    pub latest: Option<MigrationEntry>,
    /// The newest migration this build carries.
    pub latest_expected: Option<MigrationEntry>,
    /// In this build, not applied (or applied without success).
    pub pending: Vec<MigrationEntry>,
    /// Recorded as started but not completed.
    pub failed: Vec<MigrationEntry>,
    /// Applied by a newer build: this build does not know them.
    pub unknown: Vec<MigrationEntry>,
}

/// Where the database is, without secrets (t103).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInfo {
    pub backend: &'static str,
    pub server_version: Option<String>,
    /// SQLite: the database file name (no directory). Postgres: the database name.
    pub name: Option<String>,
    /// Postgres only: the server address as the server reports it.
    pub host: Option<String>,
    pub port: Option<i64>,
    /// SQLite only: the journal mode.
    pub journal_mode: Option<String>,
}

/// Submission rows by state (t103). Bounded: at most [`OUTBOX_ROW_LIMIT`]
/// pending rows are classified; `limited` says the counts stopped there.
#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboxCounts {
    /// Due now and not yet tried.
    pub queued: u64,
    /// Waiting for their send time or undo window.
    pub scheduled: u64,
    /// Held until a person releases them.
    pub held: u64,
    /// Tried, not accepted, waiting for the next attempt.
    pub deferred: u64,
    /// Claimed by a sender right now.
    pub sending: u64,
    /// Given up; still listed in the owner's Outbox.
    pub failed: u64,
    /// The connection dropped after the data was sent; delivery is unknown.
    pub uncertain: u64,
    /// Age in seconds of the oldest row that is due and not sent (queued or deferred).
    pub oldest_due_age_secs: Option<i64>,
    pub limited: bool,
}

pub const OUTBOX_ROW_LIMIT: usize = 2_000;

fn rfc3339_secs(text: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(text)
        .map(|t| t.timestamp())
        .ok()
}

impl Store {
    /// The applied migration set against the compiled one, by version.
    pub async fn migration_detail(&self) -> Result<MigrationDetail, StoreError> {
        let compiled = match &self.backend {
            Backend::Sqlite(_) => sqlx::migrate!("./migrations"),
            Backend::Postgres(_) => sqlx::migrate!("./migrations_pg"),
        };
        let expected: BTreeMap<i64, String> = compiled
            .migrations
            .iter()
            .filter(|m| !m.migration_type.is_down_migration())
            .map(|m| (m.version, m.description.to_string()))
            .collect();
        let rows = q("SELECT version, description, CAST(CASE WHEN success THEN 1 ELSE 0 END AS BIGINT) AS ok FROM _sqlx_migrations ORDER BY version")
            .fetch_all(&self.backend)
            .await?;
        let entry = |v: i64, d: String| MigrationEntry {
            version: v,
            description: d,
        };
        let mut ok = BTreeMap::new();
        let mut failed = Vec::new();
        for r in &rows {
            let (v, d) = (r.get_i64("version"), r.get_string("description"));
            if r.get_i64("ok") == 1 {
                ok.insert(v, d);
            } else {
                failed.push(entry(v, d));
            }
        }
        Ok(MigrationDetail {
            applied: ok.len(),
            expected: expected.len(),
            latest: ok.iter().next_back().map(|(v, d)| entry(*v, d.clone())),
            latest_expected: expected
                .iter()
                .next_back()
                .map(|(v, d)| entry(*v, d.clone())),
            pending: expected
                .iter()
                .filter(|(v, _)| !ok.contains_key(v))
                .map(|(v, d)| entry(*v, d.clone()))
                .collect(),
            failed,
            unknown: ok
                .iter()
                .filter(|(v, _)| !expected.contains_key(v))
                .map(|(v, d)| entry(*v, d.clone()))
                .collect(),
        })
    }

    /// The database's kind, version and location, with no credentials.
    pub async fn database_info(&self) -> Result<DatabaseInfo, StoreError> {
        match &self.backend {
            Backend::Sqlite(_) => {
                let version = q("SELECT sqlite_version() AS v")
                    .fetch_one(&self.backend)
                    .await?
                    .get_string("v");
                let file = q("PRAGMA database_list")
                    .fetch_all(&self.backend)
                    .await?
                    .iter()
                    .find(|r| r.get_string("name") == "main")
                    .map(|r| r.get_string("file"))
                    .filter(|p| !p.is_empty())
                    .map(|p| {
                        std::path::Path::new(&p)
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or(p)
                    });
                let journal = q("PRAGMA journal_mode")
                    .fetch_one(&self.backend)
                    .await
                    .ok()
                    .map(|r| r.get_string_idx(0));
                Ok(DatabaseInfo {
                    backend: "sqlite",
                    server_version: Some(version),
                    name: Some(file.unwrap_or_else(|| "in memory".into())),
                    host: None,
                    port: None,
                    journal_mode: journal,
                })
            }
            Backend::Postgres(_) => {
                let r = q("SELECT current_database() AS db, COALESCE(host(inet_server_addr()), 'local socket') AS host, CAST(COALESCE(inet_server_port(), 0) AS BIGINT) AS port, current_setting('server_version') AS v")
                    .fetch_one(&self.backend)
                    .await?;
                Ok(DatabaseInfo {
                    backend: "postgres",
                    server_version: Some(r.get_string("v")),
                    name: Some(r.get_string("db")),
                    host: Some(r.get_string("host")),
                    port: Some(r.get_i64("port")).filter(|p| *p > 0),
                    journal_mode: None,
                })
            }
        }
    }

    /// Submission rows by state at `now` (Unix seconds). Reads at most
    /// [`OUTBOX_ROW_LIMIT`] pending rows; never a message body.
    pub async fn outbox_counts(&self, now: i64) -> Result<OutboxCounts, StoreError> {
        let mut out = OutboxCounts::default();
        let rows = q("SELECT undo_status, send_at, created_at, hold_seconds, attempts, hold, next_attempt_at FROM submissions WHERE undo_status='pending' ORDER BY created_at LIMIT ?1")
            .bind((OUTBOX_ROW_LIMIT + 1) as i64)
            .fetch_all(&self.backend)
            .await?;
        out.limited = rows.len() > OUTBOX_ROW_LIMIT;
        for r in rows.iter().take(OUTBOX_ROW_LIMIT) {
            let created = rfc3339_secs(&r.get_string("created_at"));
            if r.get_opt_string("hold").is_some() {
                out.held += 1;
                continue;
            }
            let due_at = r
                .get_opt_string("send_at")
                .and_then(|s| rfc3339_secs(&s))
                .or_else(|| created.map(|c| c + r.get_i64("hold_seconds")));
            if due_at.is_some_and(|d| d > now) {
                out.scheduled += 1;
                continue;
            }
            if r.get_i64("attempts") > 0 {
                out.deferred += 1;
            } else {
                out.queued += 1;
            }
            if let Some(age) = due_at.or(created).map(|t| (now - t).max(0)) {
                out.oldest_due_age_secs = Some(out.oldest_due_age_secs.map_or(age, |o| o.max(age)));
            }
        }
        let rows = q("SELECT CASE WHEN undo_status LIKE 'sending:%' THEN 'sending' ELSE undo_status END AS s, COUNT(*) AS n FROM submissions WHERE undo_status IN ('failed','uncertain') OR undo_status LIKE 'sending:%' GROUP BY 1")
            .fetch_all(&self.backend)
            .await?;
        for r in rows {
            let n = u64::try_from(r.get_i64("n")).unwrap_or(0);
            match r.get_string("s").as_str() {
                "sending" => out.sending += n,
                "failed" => out.failed += n,
                "uncertain" => out.uncertain += n,
                _ => {}
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn bounded_snapshot_and_diagnostics_use_live_store_without_credentials() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        let snapshot = store.dashboard_snapshot(1_800_000_000, true).await.unwrap();
        assert!(snapshot.domains.is_empty());
        assert!(snapshot.audit.is_empty());
        assert_eq!(snapshot.active_admin_sessions, Some(0));
        let diag = store.store_diagnostics().await.unwrap();
        assert_eq!(diag.migration_applied as usize, diag.migration_expected);
        assert_eq!(diag.backend, "sqlite");
        assert!(diag.db_ping_ms >= 0.0);
        let migrations = store.migration_detail().await.unwrap();
        assert_eq!(migrations.applied, migrations.expected);
        assert!(migrations.pending.is_empty() && migrations.unknown.is_empty());
        assert_eq!(
            migrations.latest.as_ref().map(|m| m.version),
            migrations.latest_expected.as_ref().map(|m| m.version)
        );
        assert_eq!(migrations.applied as i64, diag.migration_applied);
        let info = store.database_info().await.unwrap();
        assert_eq!(info.backend, "sqlite");
        assert!(info.server_version.is_some());
        let outbox = store.outbox_counts(1_800_000_000).await.unwrap();
        assert_eq!(outbox.queued + outbox.failed + outbox.sending, 0);
        assert!(outbox.oldest_due_age_secs.is_none());
        q("INSERT INTO domains(name,upstream_json) VALUES('a.test','{}')")
            .execute(&store.backend)
            .await
            .unwrap();
        store
            .put_domain_alias("alias.test", "a.test")
            .await
            .unwrap();
        let snapshot = store
            .dashboard_snapshot(1_800_000_000, false)
            .await
            .unwrap();
        assert_eq!(snapshot.domains.len(), 1);
        assert_eq!(
            snapshot.canonical_domain("alias.test").as_deref(),
            Some("a.test")
        );
        assert!(snapshot.audit.is_empty());
        for (id, age) in [("recent", 60), ("week", 2 * 86400), ("old", 8 * 86400)] {
            q("INSERT INTO signin_identity(account_id,address,seen_at,event_id) VALUES(?1,?2,?3,?4)").bind(id).bind(format!("{id}@alias.test")).bind(1_800_000_000_i64-age).bind(id).execute(&store.backend).await.unwrap();
        }
        let snapshot = store
            .dashboard_snapshot(1_800_000_000, false)
            .await
            .unwrap();
        assert_eq!(snapshot.signed_in_24h, vec!["recent@alias.test"]);
        assert_eq!(snapshot.signed_in_7d.len(), 2);
    }
}
