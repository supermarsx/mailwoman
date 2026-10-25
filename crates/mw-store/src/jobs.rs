//! 0033 scheduler and job history (26.20 t32-e72, worker f12): dual-backend
//! `Store` methods over `scheduled_task` and `job_run`.
//!
//! # Tables
//! * `scheduled_task` — one row per configured task: the job kind, its schedule
//!   and parameters (JSON text the server validates), `enabled`, `only_in_window`,
//!   the last and next run, and the **DB lease** (`lease_owner`, `lease_until`).
//!   A process runs a due task only after [`Store::claim_task_lease`] moved the
//!   lease to it, so with several processes on one database a due task runs once,
//!   and a lease that was not renewed is taken over once it expired.
//! * `job_run` — one row per run: who ran it (`instance` = host name + process
//!   start time), when, its heartbeat, and how it ended. At most one `running`
//!   row per kind ([`Store::start_job_run`] refuses a second one). A `running` row
//!   whose heartbeat stopped is marked `abandoned` by
//!   [`Store::abandon_stale_job_runs`], which releases the kind again.
//!
//! The store does not know what a kind does or how a schedule is read; the
//! server's `jobs` module does. Timestamps are unix seconds and booleans 0/1
//! (0033 wire contract).

use crate::backend::{Backend, Row, q};
use crate::{Store, StoreError};

/// How a run ended (`job_run.status`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JobStatus {
    Running,
    Ok,
    Failed,
    Cancelled,
    Abandoned,
}

impl JobStatus {
    pub const ALL: [JobStatus; 5] = [
        JobStatus::Running,
        JobStatus::Ok,
        JobStatus::Failed,
        JobStatus::Cancelled,
        JobStatus::Abandoned,
    ];

    /// The column value.
    pub fn as_str(self) -> &'static str {
        match self {
            JobStatus::Running => "running",
            JobStatus::Ok => "ok",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
            JobStatus::Abandoned => "abandoned",
        }
    }

    /// Parse a column value.
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == s)
    }
}

/// One `scheduled_task` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledTaskRow {
    pub id: String,
    pub kind: String,
    pub schedule_json: String,
    pub params_json: String,
    pub enabled: bool,
    pub only_in_window: bool,
    pub created_by: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_run_at: Option<i64>,
    pub last_status: Option<String>,
    pub next_run_at: Option<i64>,
    pub lease_owner: Option<String>,
    pub lease_until: Option<i64>,
}

/// One `job_run` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobRunRow {
    pub id: String,
    pub task_id: Option<String>,
    pub kind: String,
    pub instance: String,
    pub started_at: i64,
    pub heartbeat_at: i64,
    pub finished_at: Option<i64>,
    pub status: JobStatus,
    pub summary: Option<String>,
    pub detail_json: Option<String>,
    pub triggered_by: String,
}

/// A page request over `job_run`, newest first.
#[derive(Debug, Clone, Default)]
pub struct JobRunQuery {
    pub kind: Option<String>,
    pub status: Option<JobStatus>,
    pub task_id: Option<String>,
    /// Continue after this `(started_at, id)` (the last row of the previous page).
    pub after: Option<(i64, String)>,
    pub limit: i64,
}

/// Scheduler projection of an uncancelled global maintenance window.
#[derive(Debug, Clone)]
pub struct SchedulerWindow {
    pub starts_at: i64,
    pub ends_at: Option<i64>,
    pub tz: String,
    pub recurrence_json: Option<String>,
    pub occurrence_ended_at: Option<i64>,
}

fn task_from_row(r: &Row) -> ScheduledTaskRow {
    ScheduledTaskRow {
        id: r.get_string("id"),
        kind: r.get_string("kind"),
        schedule_json: r.get_string("schedule_json"),
        params_json: r.get_string("params_json"),
        enabled: r.get_i64("enabled") != 0,
        only_in_window: r.get_i64("only_in_window") != 0,
        created_by: r.get_string("created_by"),
        created_at: r.get_i64("created_at"),
        updated_at: r.get_i64("updated_at"),
        last_run_at: r.get_opt_i64("last_run_at"),
        last_status: r.get_opt_string("last_status"),
        next_run_at: r.get_opt_i64("next_run_at"),
        lease_owner: r.get_opt_string("lease_owner"),
        lease_until: r.get_opt_i64("lease_until"),
    }
}

fn run_from_row(r: &Row) -> Result<JobRunRow, StoreError> {
    let status = r.get_string("status");
    Ok(JobRunRow {
        id: r.get_string("id"),
        task_id: r.get_opt_string("task_id"),
        kind: r.get_string("kind"),
        instance: r.get_string("instance"),
        started_at: r.get_i64("started_at"),
        heartbeat_at: r.get_i64("heartbeat_at"),
        finished_at: r.get_opt_i64("finished_at"),
        status: JobStatus::parse(&status)
            .ok_or_else(|| StoreError::Corrupt(format!("job_run.status {status}")))?,
        summary: r.get_opt_string("summary"),
        detail_json: r.get_opt_string("detail_json"),
        triggered_by: r.get_string("triggered_by"),
    })
}

impl Store {
    pub async fn scheduler_windows(&self) -> Result<Vec<SchedulerWindow>, StoreError> {
        let rows = q("SELECT starts_at, ends_at, tz, recurrence_json, occurrence_ended_at
            FROM maintenance_window WHERE scope_kind = 'global' AND scope_id = '' AND cancelled_at IS NULL")
            .fetch_all(&self.backend).await?;
        Ok(rows.into_iter().map(|r| SchedulerWindow {
            starts_at: r.get_i64("starts_at"), ends_at: r.get_opt_i64("ends_at"),
            tz: r.get_string("tz"), recurrence_json: r.get_opt_string("recurrence_json"),
            occurrence_ended_at: r.get_opt_i64("occurrence_ended_at"),
        }).collect())
    }
    // ── scheduled_task ──────────────────────────────────────────────────────

    /// Insert a new task row.
    pub async fn insert_scheduled_task(&self, t: &ScheduledTaskRow) -> Result<(), StoreError> {
        q("INSERT INTO scheduled_task
               (id, kind, schedule_json, params_json, enabled, only_in_window, created_by,
                created_at, updated_at, last_run_at, last_status, next_run_at, lease_owner,
                lease_until)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)")
        .bind(&t.id)
        .bind(&t.kind)
        .bind(&t.schedule_json)
        .bind(&t.params_json)
        .bind(i64::from(t.enabled))
        .bind(i64::from(t.only_in_window))
        .bind(&t.created_by)
        .bind(t.created_at)
        .bind(t.updated_at)
        .bind(t.last_run_at)
        .bind(t.last_status.clone())
        .bind(t.next_run_at)
        .bind(t.lease_owner.clone())
        .bind(t.lease_until)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// One task row.
    pub async fn get_scheduled_task(
        &self,
        id: &str,
    ) -> Result<Option<ScheduledTaskRow>, StoreError> {
        let row = q("SELECT id, kind, schedule_json, params_json, enabled, only_in_window, \
             created_by, created_at, updated_at, last_run_at, last_status, next_run_at, \
             lease_owner, lease_until FROM scheduled_task WHERE id = ?1")
        .bind(id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(task_from_row))
    }

    /// Every task row, oldest first.
    pub async fn list_scheduled_tasks(&self) -> Result<Vec<ScheduledTaskRow>, StoreError> {
        let rows = q("SELECT id, kind, schedule_json, params_json, enabled, only_in_window, \
             created_by, created_at, updated_at, last_run_at, last_status, next_run_at, \
             lease_owner, lease_until FROM scheduled_task ORDER BY created_at, id")
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(task_from_row).collect())
    }

    /// Replace the editable columns of a task (schedule, parameters, switches, next
    /// run). Run history is retained. Editing revokes the old lease so a worker
    /// cannot overwrite the new schedule or continue with stale parameters.
    pub async fn update_scheduled_task(&self, t: &ScheduledTaskRow) -> Result<bool, StoreError> {
        let n = q("UPDATE scheduled_task SET schedule_json = ?2, params_json = ?3, enabled = ?4,
               only_in_window = ?5, updated_at = ?6, next_run_at = ?7,
               lease_owner = NULL, lease_until = NULL
           WHERE id = ?1")
        .bind(&t.id)
        .bind(&t.schedule_json)
        .bind(&t.params_json)
        .bind(i64::from(t.enabled))
        .bind(i64::from(t.only_in_window))
        .bind(t.updated_at)
        .bind(t.next_run_at)
        .execute(&self.backend)
        .await?;
        Ok(n > 0)
    }

    /// Delete a task. Its run history stays. `true` when a row was removed.
    pub async fn delete_scheduled_task(&self, id: &str) -> Result<bool, StoreError> {
        let n = q("DELETE FROM scheduled_task WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(n > 0)
    }

    /// Enabled tasks whose next run is at or before `now`, earliest first.
    /// Includes tasks another process holds a live lease on; the claim decides.
    pub async fn due_scheduled_tasks(&self, now: i64) -> Result<Vec<ScheduledTaskRow>, StoreError> {
        let rows = q("SELECT id, kind, schedule_json, params_json, enabled, only_in_window, \
             created_by, created_at, updated_at, last_run_at, last_status, next_run_at, \
             lease_owner, lease_until FROM scheduled_task \
             WHERE enabled = 1 AND next_run_at IS NOT NULL AND next_run_at <= ?1 \
             ORDER BY next_run_at, id")
        .bind(now)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(task_from_row).collect())
    }

    /// Enabled tasks with no next run yet (just enabled, or written without one).
    pub async fn unscheduled_tasks(&self) -> Result<Vec<ScheduledTaskRow>, StoreError> {
        let rows = q("SELECT id, kind, schedule_json, params_json, enabled, only_in_window, \
             created_by, created_at, updated_at, last_run_at, last_status, next_run_at, \
             lease_owner, lease_until FROM scheduled_task \
             WHERE enabled = 1 AND next_run_at IS NULL ORDER BY created_at, id")
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(task_from_row).collect())
    }

    /// Set the next run of a task that has none (`next_run_at IS NULL`). `true`
    /// when this call set it.
    pub async fn set_task_first_run(&self, id: &str, next_run_at: i64) -> Result<bool, StoreError> {
        let n = q("UPDATE scheduled_task SET next_run_at = ?2
                   WHERE id = ?1 AND next_run_at IS NULL")
        .bind(id)
        .bind(next_run_at)
        .execute(&self.backend)
        .await?;
        Ok(n > 0)
    }

    /// Take the lease of a due task for `owner` until `until`. Succeeds only while
    /// the task is enabled and due at `now` and nobody else holds a live lease
    /// (`lease_until > now`). One statement, so two processes racing for the same
    /// task cannot both win. `true` when `owner` now holds the lease.
    pub async fn claim_task_lease(
        &self,
        id: &str,
        owner: &str,
        now: i64,
        until: i64,
    ) -> Result<bool, StoreError> {
        if until <= now || owner.is_empty() {
            return Ok(false);
        }
        let n = q("UPDATE scheduled_task SET lease_owner = ?2, lease_until = ?4
           WHERE id = ?1 AND enabled = 1 AND next_run_at IS NOT NULL AND next_run_at <= ?3
             AND (lease_owner IS NULL OR lease_until IS NULL OR lease_until <= ?3)")
        .bind(id)
        .bind(owner)
        .bind(now)
        .bind(until)
        .execute(&self.backend)
        .await?;
        Ok(n > 0)
    }

    /// Extend a lease `owner` still holds. `false` when the lease was lost (taken
    /// over after it expired, or the task was deleted).
    pub async fn renew_task_lease(
        &self,
        id: &str,
        owner: &str,
        until: i64,
    ) -> Result<bool, StoreError> {
        let n = q("UPDATE scheduled_task SET lease_until = ?3
                   WHERE id = ?1 AND lease_owner = ?2")
        .bind(id)
        .bind(owner)
        .bind(until)
        .execute(&self.backend)
        .await?;
        Ok(n > 0)
    }

    /// Give a lease back and record the run: last run, its status, and the next run.
    /// `last_run_at`/`last_status` `None` keep the stored values (a claim that was
    /// given back without running). Only the holder can do this; `false` otherwise.
    pub async fn release_task_lease(
        &self,
        id: &str,
        owner: &str,
        last_run_at: Option<i64>,
        last_status: Option<&str>,
        next_run_at: Option<i64>,
    ) -> Result<bool, StoreError> {
        let n = q("UPDATE scheduled_task SET lease_owner = NULL, lease_until = NULL,
               last_run_at = COALESCE(?3, last_run_at),
               last_status = COALESCE(?4, last_status),
               next_run_at = ?5
           WHERE id = ?1 AND lease_owner = ?2")
        .bind(id)
        .bind(owner)
        .bind(last_run_at)
        .bind(last_status)
        .bind(next_run_at)
        .execute(&self.backend)
        .await?;
        Ok(n > 0)
    }

    /// The `vacuum` job: SQLite `VACUUM` (whole file) or a plain Postgres `VACUUM`
    /// of the database (never `FULL`, so reads and writes continue). Runs on the
    /// pool in autocommit; `VACUUM` cannot run inside a transaction.
    pub async fn vacuum_database(&self) -> Result<(), StoreError> {
        use sqlx::Executor as _;
        match &self.backend {
            Backend::Sqlite(pool) => {
                pool.execute("VACUUM").await?;
            }
            Backend::Postgres(pool) => {
                pool.execute("VACUUM").await?;
            }
        }
        Ok(())
    }

    // ── job_run ─────────────────────────────────────────────────────────────

    /// Insert `run` as `running` unless a `running` row of the same kind exists.
    /// `false` (nothing written) when one does. Atomic across processes: SQLite runs
    /// the guarded insert as one statement under its write lock; Postgres holds a
    /// transaction-scoped advisory lock on the kind around the check and the insert.
    pub async fn start_job_run(&self, run: &JobRunRow) -> Result<bool, StoreError> {
        const INSERT: &str = "INSERT INTO job_run
               (id, task_id, kind, instance, started_at, heartbeat_at, finished_at, status,
                summary, detail_json, triggered_by)
           SELECT ?1, ?2, ?3, ?4, ?5, ?6, NULL, 'running', NULL, NULL, ?7
           WHERE NOT EXISTS (SELECT 1 FROM job_run WHERE kind = ?3 AND status = 'running')
             AND (?2 IS NULL OR EXISTS (SELECT 1 FROM scheduled_task
               WHERE id = ?2 AND kind = ?3 AND enabled = 1
                 AND lease_owner = ?4 AND lease_until > ?5))";
        let bind = |sql: crate::backend::Sql| {
            sql.bind(&run.id)
                .bind(run.task_id.clone())
                .bind(&run.kind)
                .bind(&run.instance)
                .bind(run.started_at)
                .bind(run.heartbeat_at)
                .bind(&run.triggered_by)
        };
        let n = match &self.backend {
            Backend::Sqlite(_) => bind(q(INSERT)).execute(&self.backend).await?,
            Backend::Postgres(_) => {
                let mut tx = self.backend.begin().await?;
                q("SELECT pg_advisory_xact_lock(hashtext('mw_job_run:' || ?1))")
                    .bind(&run.kind)
                    .execute_tx(&mut tx)
                    .await?;
                let n = bind(q(INSERT)).execute_tx(&mut tx).await?;
                tx.commit().await?;
                n
            }
        };
        Ok(n > 0)
    }

    /// Insert a finished run as it is (a loop that schedules itself reporting a
    /// run, `jobs::observe`). No one-per-kind check: the row is never `running`.
    pub async fn insert_job_run(&self, run: &JobRunRow) -> Result<(), StoreError> {
        if run.status == JobStatus::Running || run.finished_at.is_none() {
            return Err(StoreError::Corrupt("observed job must be finished".into()));
        }
        q("INSERT INTO job_run
               (id, task_id, kind, instance, started_at, heartbeat_at, finished_at, status,
                summary, detail_json, triggered_by)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)")
        .bind(&run.id)
        .bind(run.task_id.clone())
        .bind(&run.kind)
        .bind(&run.instance)
        .bind(run.started_at)
        .bind(run.heartbeat_at)
        .bind(run.finished_at)
        .bind(run.status.as_str())
        .bind(run.summary.clone())
        .bind(run.detail_json.clone())
        .bind(&run.triggered_by)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Record that a running run is alive. `false` when the row is no longer
    /// `running` (it was marked abandoned).
    pub async fn heartbeat_job_run(&self, id: &str, now: i64) -> Result<bool, StoreError> {
        let n = q("UPDATE job_run SET heartbeat_at = ?2 WHERE id = ?1 AND status = 'running'")
            .bind(id)
            .bind(now)
            .execute(&self.backend)
            .await?;
        Ok(n > 0)
    }

    /// Renew a run and its task lease in one transaction. A stale owner cannot
    /// resurrect an expired lease or refresh a cancelled/replaced run.
    pub async fn renew_job_ownership(
        &self, run_id: &str, owner: &str, task_id: Option<&str>, now: i64, until: i64,
    ) -> Result<bool, StoreError> {
        if until <= now { return Ok(false); }
        let mut tx = self.backend.begin().await?;
        let n = q("UPDATE job_run SET heartbeat_at = ?3
            WHERE id = ?1 AND instance = ?2 AND status = 'running'
              AND ((task_id IS NULL AND ?4 IS NULL) OR task_id = ?4)")
            .bind(run_id).bind(owner).bind(now).bind(task_id)
            .execute_tx(&mut tx).await?;
        if n != 1 { return Ok(false); }
        if let Some(id) = task_id {
            let n = q("UPDATE scheduled_task SET lease_until = ?4
                WHERE id = ?1 AND lease_owner = ?2 AND enabled = 1 AND lease_until > ?3")
                .bind(id).bind(owner).bind(now).bind(until).execute_tx(&mut tx).await?;
            if n != 1 { return Ok(false); }
        }
        tx.commit().await?;
        Ok(true)
    }

    /// Finish a running run. `false` when it is no longer `running` (an abandoned
    /// row keeps its status: another process may already have run the kind again).
    pub async fn finish_job_run(
        &self,
        id: &str,
        status: JobStatus,
        finished_at: i64,
        summary: Option<&str>,
        detail_json: Option<&str>,
    ) -> Result<bool, StoreError> {
        if status == JobStatus::Running {
            return Err(StoreError::Corrupt("finished job cannot remain running".into()));
        }
        let n = q("UPDATE job_run SET status = ?2, finished_at = ?3, heartbeat_at = ?3,
               summary = ?4, detail_json = ?5
           WHERE id = ?1 AND status = 'running'")
        .bind(id)
        .bind(status.as_str())
        .bind(finished_at)
        .bind(summary)
        .bind(detail_json)
        .execute(&self.backend)
        .await?;
        Ok(n > 0)
    }

    /// Mark every `running` row of `kind` whose heartbeat is older than `cutoff` as
    /// `abandoned` (finished at `now`). Returns how many rows changed.
    pub async fn abandon_stale_job_runs(
        &self,
        kind: &str,
        cutoff: i64,
        now: i64,
    ) -> Result<u64, StoreError> {
        Ok(q("UPDATE job_run SET status = 'abandoned', finished_at = ?3,
                  summary = COALESCE(summary, 'heartbeat stopped')
              WHERE kind = ?1 AND status = 'running' AND heartbeat_at < ?2")
        .bind(kind)
        .bind(cutoff)
        .bind(now)
        .execute(&self.backend)
        .await?)
    }

    /// One run.
    pub async fn get_job_run(&self, id: &str) -> Result<Option<JobRunRow>, StoreError> {
        let row = q("SELECT id, task_id, kind, instance, started_at, heartbeat_at, finished_at, \
             status, summary, detail_json, triggered_by FROM job_run WHERE id = ?1")
        .bind(id)
        .fetch_optional(&self.backend)
        .await?;
        row.as_ref().map(run_from_row).transpose()
    }

    /// Every `running` row.
    pub async fn running_job_runs(&self) -> Result<Vec<JobRunRow>, StoreError> {
        let rows = q("SELECT id, task_id, kind, instance, started_at, heartbeat_at, finished_at, \
             status, summary, detail_json, triggered_by FROM job_run \
             WHERE status = 'running' ORDER BY started_at, id")
        .fetch_all(&self.backend)
        .await?;
        rows.iter().map(run_from_row).collect()
    }

    /// The newest run of `kind` (any status).
    pub async fn last_job_run(&self, kind: &str) -> Result<Option<JobRunRow>, StoreError> {
        let row = q("SELECT id, task_id, kind, instance, started_at, heartbeat_at, finished_at, \
             status, summary, detail_json, triggered_by FROM job_run WHERE kind = ?1 \
             ORDER BY started_at DESC, id DESC LIMIT 1")
        .bind(kind)
        .fetch_optional(&self.backend)
        .await?;
        row.as_ref().map(run_from_row).transpose()
    }

    /// A page of runs, newest first, filtered by kind, status and task.
    pub async fn list_job_runs(&self, query: &JobRunQuery) -> Result<Vec<JobRunRow>, StoreError> {
        let (after_ts, after_id) = match &query.after {
            Some((ts, id)) => (Some(*ts), Some(id.clone())),
            None => (None, None),
        };
        let rows = q("SELECT id, task_id, kind, instance, started_at, heartbeat_at, finished_at, \
             status, summary, detail_json, triggered_by FROM job_run \
             WHERE (?1 IS NULL OR kind = ?1) AND (?2 IS NULL OR status = ?2) \
               AND (?3 IS NULL OR task_id = ?3) \
               AND (?4 IS NULL OR started_at < ?4 OR (started_at = ?4 AND id < ?5)) \
             ORDER BY started_at DESC, id DESC LIMIT ?6")
        .bind(query.kind.clone())
        .bind(query.status.map(|s| s.as_str().to_string()))
        .bind(query.task_id.clone())
        .bind(after_ts)
        .bind(after_id)
        .bind(query.limit.clamp(1, 500))
        .fetch_all(&self.backend)
        .await?;
        rows.iter().map(run_from_row).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    fn task(id: &str, next: Option<i64>) -> ScheduledTaskRow {
        ScheduledTaskRow {
            id: id.into(),
            kind: "vacuum".into(),
            schedule_json: r#"{"everySecs":3600}"#.into(),
            params_json: "{}".into(),
            enabled: true,
            only_in_window: false,
            created_by: "root".into(),
            created_at: 10,
            updated_at: 10,
            last_run_at: None,
            last_status: None,
            next_run_at: next,
            lease_owner: None,
            lease_until: None,
        }
    }

    fn run(id: &str, kind: &str, at: i64) -> JobRunRow {
        JobRunRow {
            id: id.into(),
            task_id: None,
            kind: kind.into(),
            instance: "host-a@1".into(),
            started_at: at,
            heartbeat_at: at,
            finished_at: None,
            status: JobStatus::Running,
            summary: None,
            detail_json: None,
            triggered_by: "root".into(),
        }
    }

    #[tokio::test]
    async fn tasks_round_trip_and_due() {
        tasks(&store().await).await;
    }

    async fn tasks(s: &Store) {
        let mut t = task("t1", Some(100));
        s.insert_scheduled_task(&t).await.unwrap();
        s.insert_scheduled_task(&task("t2", None)).await.unwrap();
        let mut off = task("t3", Some(50));
        off.enabled = false;
        s.insert_scheduled_task(&off).await.unwrap();
        assert_eq!(s.get_scheduled_task("t1").await.unwrap().as_ref(), Some(&t));
        assert_eq!(s.list_scheduled_tasks().await.unwrap().len(), 3);
        // Disabled and unscheduled tasks are never due.
        assert!(s.due_scheduled_tasks(99).await.unwrap().is_empty());
        let due = s.due_scheduled_tasks(100).await.unwrap();
        assert_eq!(due.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(), ["t1"]);
        let un = s.unscheduled_tasks().await.unwrap();
        assert_eq!(un.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(), ["t2"]);
        assert!(s.set_task_first_run("t2", 500).await.unwrap());
        assert!(!s.set_task_first_run("t2", 600).await.unwrap());

        t.params_json = r#"{"x":1}"#.into();
        t.next_run_at = Some(200);
        t.updated_at = 11;
        assert!(s.update_scheduled_task(&t).await.unwrap());
        assert_eq!(s.get_scheduled_task("t1").await.unwrap().as_ref(), Some(&t));
        assert!(s.delete_scheduled_task("t3").await.unwrap());
        assert!(!s.delete_scheduled_task("t3").await.unwrap());
        for id in ["t1", "t2"] {
            s.delete_scheduled_task(id).await.unwrap();
        }
    }

    #[tokio::test]
    async fn lease_is_claimed_once_and_taken_over_after_expiry() {
        lease(&store().await).await;
    }

    async fn lease(s: &Store) {
        s.insert_scheduled_task(&task("L", Some(100))).await.unwrap();
        // Not due yet: nobody may claim it.
        assert!(!s.claim_task_lease("L", "a", 99, 160).await.unwrap());
        assert!(s.claim_task_lease("L", "a", 100, 160).await.unwrap());
        // A second process while the lease is live.
        assert!(!s.claim_task_lease("L", "b", 120, 180).await.unwrap());
        assert!(s.renew_task_lease("L", "a", 200).await.unwrap());
        assert!(!s.renew_task_lease("L", "b", 300).await.unwrap());
        // The holder stops renewing; after expiry another process takes it over.
        assert!(!s.claim_task_lease("L", "b", 199, 260).await.unwrap());
        assert!(s.claim_task_lease("L", "b", 200, 260).await.unwrap());
        assert!(!s.renew_task_lease("L", "a", 300).await.unwrap());
        assert!(!s.release_task_lease("L", "a", Some(1), Some("ok"), Some(9)).await.unwrap());
        assert!(s.release_task_lease("L", "b", Some(210), Some("ok"), Some(3800)).await.unwrap());
        let t = s.get_scheduled_task("L").await.unwrap().unwrap();
        assert_eq!(
            (t.lease_owner, t.lease_until, t.last_run_at, t.last_status.as_deref(), t.next_run_at),
            (None, None, Some(210), Some("ok"), Some(3800))
        );
        // A claim given back without a run keeps the last run.
        s.release_task_lease("L", "nobody", None, None, Some(1)).await.unwrap();
        assert!(s.claim_task_lease("L", "c", 3800, 3860).await.unwrap());
        assert!(s.release_task_lease("L", "c", None, None, Some(7200)).await.unwrap());
        let t = s.get_scheduled_task("L").await.unwrap().unwrap();
        assert_eq!((t.last_run_at, t.next_run_at), (Some(210), Some(7200)));
        s.delete_scheduled_task("L").await.unwrap();
    }

    #[tokio::test]
    async fn one_running_row_per_kind_until_abandoned() {
        runs(&store().await).await;
    }

    async fn runs(s: &Store) {
        assert!(s.start_job_run(&run("r1", "reindex", 100)).await.unwrap());
        assert!(!s.start_job_run(&run("r2", "reindex", 101)).await.unwrap());
        assert!(s.get_job_run("r2").await.unwrap().is_none());
        // Another kind is independent.
        assert!(s.start_job_run(&run("r3", "vacuum", 101)).await.unwrap());
        assert!(s.heartbeat_job_run("r1", 130).await.unwrap());
        // Heartbeat newer than the cutoff: kept.
        assert_eq!(s.abandon_stale_job_runs("reindex", 130, 200).await.unwrap(), 0);
        assert_eq!(s.abandon_stale_job_runs("reindex", 131, 200).await.unwrap(), 1);
        let r1 = s.get_job_run("r1").await.unwrap().unwrap();
        assert_eq!((r1.status, r1.finished_at), (JobStatus::Abandoned, Some(200)));
        // The kind is free again; the late finish of the abandoned run is refused.
        assert!(s.start_job_run(&run("r4", "reindex", 201)).await.unwrap());
        assert!(!s.finish_job_run("r1", JobStatus::Ok, 202, None, None).await.unwrap());
        assert!(!s.heartbeat_job_run("r1", 202).await.unwrap());
        assert!(
            s.finish_job_run("r4", JobStatus::Failed, 205, Some("boom"), Some("{}"))
                .await
                .unwrap()
        );
        assert_eq!(s.running_job_runs().await.unwrap().len(), 1);
        let last = s.last_job_run("reindex").await.unwrap().unwrap();
        assert_eq!(
            (last.id.as_str(), last.status, last.summary.as_deref()),
            ("r4", JobStatus::Failed, Some("boom"))
        );
        s.finish_job_run("r3", JobStatus::Ok, 206, None, None).await.unwrap();
        let mut observed = run("r5", "measure", 300);
        observed.status = JobStatus::Ok;
        observed.finished_at = Some(301);
        s.insert_job_run(&observed).await.unwrap();

        // Paging newest first, with filters.
        let page = |kind: Option<&str>, status: Option<JobStatus>, after: Option<(i64, String)>| {
            JobRunQuery {
                kind: kind.map(str::to_string),
                status,
                task_id: None,
                after,
                limit: 2,
            }
        };
        let p1 = s.list_job_runs(&page(None, None, None)).await.unwrap();
        assert_eq!(p1.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["r5", "r4"]);
        let p2 = s
            .list_job_runs(&page(None, None, Some((201, "r4".into()))))
            .await
            .unwrap();
        assert_eq!(p2.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["r3", "r1"]);
        let fails = s
            .list_job_runs(&page(Some("reindex"), Some(JobStatus::Failed), None))
            .await
            .unwrap();
        assert_eq!(fails.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["r4"]);
    }

    fn pg_dsn() -> Option<String> {
        std::env::var("DATABASE_URL_PG")
            .ok()
            .or_else(|| std::env::var("MW_TEST_PG").ok())
            .filter(|s| !s.trim().is_empty())
    }

    async fn ownership(s: &Store) {
        let mut task = task("owned", Some(100));
        task.kind = "owned_kind".into();
        s.insert_scheduled_task(&task).await.unwrap();
        assert!(!s.claim_task_lease("owned", "a", 100, 100).await.unwrap());
        let mut r = run("owned_run", "owned_kind", 100);
        r.task_id = Some("owned".into()); r.instance = "a".into();
        assert!(!s.start_job_run(&r).await.unwrap(), "no run without lease");
        assert!(s.claim_task_lease("owned", "a", 100, 150).await.unwrap());
        assert!(s.start_job_run(&r).await.unwrap());
        assert!(!s.renew_job_ownership(&r.id,"b",Some("owned"),110,160).await.unwrap());
        assert!(s.renew_job_ownership(&r.id,"a",Some("owned"),110,160).await.unwrap());
        assert!(!s.renew_job_ownership(&r.id,"a",Some("owned"),160,210).await.unwrap(), "expired lease cannot resurrect");
        assert_eq!(s.get_job_run(&r.id).await.unwrap().unwrap().heartbeat_at,110, "failed lease renewal rolled back heartbeat");
        assert!(s.claim_task_lease("owned","b",160,210).await.unwrap());
        assert!(!s.renew_job_ownership(&r.id,"a",Some("owned"),161,211).await.unwrap());
        assert!(!s.release_task_lease("owned","a",Some(161),Some("ok"),Some(999)).await.unwrap());
        let mut edited=s.get_scheduled_task("owned").await.unwrap().unwrap();
        edited.next_run_at=Some(900);edited.params_json="{\"changed\":true}".into();
        s.update_scheduled_task(&edited).await.unwrap();
        assert!(!s.release_task_lease("owned","b",Some(162),Some("ok"),Some(800)).await.unwrap(),"an edited task revokes the old run's schedule write");
        assert_eq!(s.get_scheduled_task("owned").await.unwrap().unwrap().next_run_at,Some(900));
        assert!(s.finish_job_run(&r.id,JobStatus::Cancelled,162,None,None).await.unwrap());
        assert!(!s.heartbeat_job_run(&r.id,163).await.unwrap());
        assert!(s.insert_job_run(&run("bad_observed","other",1)).await.is_err());
        assert!(s.finish_job_run(&r.id,JobStatus::Running,164,None,None).await.is_err());
    }

    #[tokio::test]
    async fn ownership_renewal_is_atomic_and_fenced() { ownership(&store().await).await; }

    #[tokio::test]
    async fn lease_process_child() {
        let Ok(dsn) = std::env::var("MW_JOB_CHILD_DSN") else { return };
        let id=std::env::var("MW_JOB_CHILD_ID").unwrap();
        let s=Store::open(&dsn,ServerKey::generate()).await.unwrap();
        let owner=format!("child-{}", std::process::id());
        if s.claim_task_lease(&id,&owner,100,1000).await.unwrap() { println!("LEASE_WON"); }
    }

    async fn processes(s: &Store, dsn: &str) {
        let id=format!("proc_{}",std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        s.insert_scheduled_task(&task(&id,Some(100))).await.unwrap();
        let dsn=dsn.to_owned();
        let outputs=tokio::task::spawn_blocking(move || {
            let mut children=Vec::new();
            for _ in 0..4 {
                children.push(std::process::Command::new(std::env::current_exe().unwrap())
                    .args(["--exact","jobs::tests::lease_process_child","--nocapture"])
                    .env("MW_JOB_CHILD_DSN",&dsn).env("MW_JOB_CHILD_ID",&id)
                    .stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped())
                    .spawn().unwrap());
            }
            children.into_iter().map(|child|child.wait_with_output().unwrap()).collect::<Vec<_>>()
        }).await.unwrap();
        let mut won=0;
        for output in outputs {
            assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
            won+=usize::from(String::from_utf8_lossy(&output.stdout).contains("LEASE_WON"));
        }
        assert_eq!(won,1,"exactly one OS process claims lease");
    }

    #[tokio::test]
    async fn sqlite_processes_claim_once() {
        let dir=std::env::temp_dir().join(format!("mw-jobs-{}",std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&dir).unwrap();
        let dsn=dir.join("jobs.sqlite").to_string_lossy().into_owned();
        let s=Store::open(&dsn,ServerKey::generate()).await.unwrap();
        processes(&s,&dsn).await;
    }

    /// The Postgres legs, env-gated like the rest of the crate (`MW_REQUIRE_LIVE`
    /// turns a skip into a failure).
    #[tokio::test]
    async fn postgres_legs() {
        let Some(dsn) = pg_dsn() else {
            mw_test_gate::skip(
                "[mw-store] jobs Postgres legs (set DATABASE_URL_PG or MW_TEST_PG to a live \
                 postgres:16 to run them). The SQLite path still asserted.",
            );
            return;
        };
        let s = Store::open_postgres(&dsn, ServerKey::generate())
            .await
            .expect("DATABASE_URL_PG is set but Postgres is not reachable");
        for t in ["job_run", "scheduled_task"] {
            let sql: &'static str = if t == "job_run" {
                "DELETE FROM job_run"
            } else {
                "DELETE FROM scheduled_task"
            };
            q(sql).execute(&s.backend).await.unwrap();
        }
        tasks(&s).await;
        lease(&s).await;
        runs(&s).await;
        ownership(&s).await;
        processes(&s,&dsn).await;
        // Concurrent starts of one kind on Postgres: exactly one wins.
        q("DELETE FROM job_run").execute(&s.backend).await.unwrap();
        let mut handles = Vec::new();
        for i in 0..8 {
            let s = s.clone();
            handles.push(tokio::spawn(async move {
                s.start_job_run(&run(&format!("c{i}"), "race", 1)).await.unwrap()
            }));
        }
        let mut won = 0;
        for h in handles {
            won += usize::from(h.await.unwrap());
        }
        assert_eq!(won, 1);
    }
}
