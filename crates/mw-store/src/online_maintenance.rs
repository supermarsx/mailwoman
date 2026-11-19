//! Online logical conversion with an explicitly bounded, queued generation cutover.
//! A published generation is never rolled back automatically after it accepts writes.
use crate::{
    Store,
    backend::Backend,
    generation_files::{self},
    online_copy::{CopyOptions, CopyProgress, CopySession},
    sqlite_live::{ExclusiveSqlite, LiveSqlite},
};
use serde::{Deserialize, Serialize};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        Arc, LazyLock, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversionOptions {
    pub batch_rows: u32,
    pub batch_mib: u32,
    pub journal_limit: u64,
    pub pause_ms: u64,
    pub max_wal_mib: u64,
    pub max_cutover_ms: u64,
}
impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            batch_rows: 16,
            batch_mib: 256,
            journal_limit: 1_000_000,
            pause_ms: 10,
            max_wal_mib: 512,
            max_cutover_ms: 2000,
        }
    }
}
impl ConversionOptions {
    fn validate(&self) -> Result<(), String> {
        if !(1..=128).contains(&self.batch_rows)
            || !(1..=256).contains(&self.batch_mib)
            || !(1024..=2_000_000).contains(&self.journal_limit)
            || self.pause_ms > 1000
            || !(32..=4096).contains(&self.max_wal_mib)
            || !(100..=10_000).contains(&self.max_cutover_ms)
        {
            return Err("Online conversion options exceed the supported bounds".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionReport {
    pub id: String,
    pub phase: String,
    pub started_at: i64,
    pub updated_at: i64,
    pub progress: Option<CopyProgress>,
    pub error: Option<String>,
    pub retained_previous: bool,
    #[serde(default)]
    pub source_generation: Option<String>,
    #[serde(default)]
    pub target_generation: Option<String>,
}
struct Job {
    report: Mutex<ConversionReport>,
    cancel: Arc<AtomicBool>,
    running: AtomicBool,
    last_write: Mutex<Instant>,
}
static JOBS: LazyLock<Mutex<HashMap<PathBuf, Arc<Job>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
fn terminal(phase: &str) -> bool {
    matches!(phase, "completed" | "cancelled" | "failed")
}
fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_owned();
    value.push(suffix);
    value.into()
}
fn status_path(path: &Path) -> PathBuf {
    sidecar(path, ".mw-online-status.json")
}
fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
fn snapshot(job: &Job) -> ConversionReport {
    job.report.lock().unwrap_or_else(|p| p.into_inner()).clone()
}
fn publish_status(
    path: &Path,
    job: &Job,
    phase: &str,
    progress: Option<CopyProgress>,
    error: Option<String>,
) -> Result<(), String> {
    let report = {
        let mut r = job.report.lock().unwrap_or_else(|p| p.into_inner());
        r.phase = phase.into();
        r.updated_at = now();
        if progress.is_some() {
            r.progress = progress;
        }
        r.error = error;
        r.clone()
    };
    let mut at = job.last_write.lock().unwrap_or_else(|p| p.into_inner());
    if at.elapsed() < Duration::from_secs(1) && matches!(phase, "copying" | "catching-up") {
        return Ok(());
    }
    let bytes = serde_json::to_vec(&report).map_err(|_| "Could not encode conversion status")?;
    generation_files::private_atomic_write(&status_path(path), &bytes)
        .map_err(|e| e.to_string())?;
    *at = Instant::now();
    Ok(())
}
impl Store {
    pub fn online_conversion_status(&self) -> Option<ConversionReport> {
        let Backend::Sqlite(live) = &self.backend else {
            return None;
        };
        if let Some(job) = JOBS
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(live.logical_path())
        {
            return Some(snapshot(job));
        }
        generation_files::bounded_read(&status_path(live.logical_path()), 65536)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
    }
    pub fn cancel_online_conversion(&self) -> bool {
        let Backend::Sqlite(live) = &self.backend else {
            return false;
        };
        let jobs = JOBS.lock().unwrap_or_else(|p| p.into_inner());
        let Some(job) = jobs.get(live.logical_path()) else {
            return false;
        };
        let r = job.report.lock().unwrap_or_else(|p| p.into_inner());
        if terminal(&r.phase)
            || matches!(
                r.phase.as_str(),
                "switching" | "publishing" | "recovery-required"
            )
        {
            return false;
        }
        job.cancel.store(true, Ordering::Release);
        true
    }
    pub async fn start_online_conversion(
        &self,
        options: ConversionOptions,
    ) -> Result<ConversionReport, String> {
        self.start_online_conversion_with_id(options, uuid::Uuid::new_v4().to_string())
            .await
    }
    pub(crate) async fn start_online_conversion_with_id(
        &self,
        options: ConversionOptions,
        id: String,
    ) -> Result<ConversionReport, String> {
        options.validate()?;
        let Backend::Sqlite(live) = &self.backend else {
            return Err("Online conversion is available only for SQLite installations".into());
        };
        if !live.is_managed() {
            return Err("Online conversion requires an exclusively owned file database".into());
        }
        // Preserve the recovery record of an unconfirmed shadow. A retry must
        // not overwrite its generation IDs before the retained fence is reset.
        if live.files().is_some_and(|files|files.conversion_active()) {
            return Err("Conversion still owns this installation; wait for completion or restart the server if worker shutdown was unconfirmed".into());
        }
        let path = live.logical_path().to_path_buf();
        let report = ConversionReport {
            id,
            phase: "preparing".into(),
            started_at: now(),
            updated_at: now(),
            progress: None,
            error: None,
            retained_previous: false,
            source_generation: None,
            target_generation: None,
        };
        let job = Arc::new(Job {
            report: Mutex::new(report.clone()),
            cancel: Arc::new(AtomicBool::new(false)),
            running: AtomicBool::new(true),
            last_write: Mutex::new(Instant::now() - Duration::from_secs(2)),
        });
        {
            let mut jobs = JOBS.lock().unwrap_or_else(|p| p.into_inner());
            if jobs
                .get(&path)
                .is_some_and(|j| j.running.load(Ordering::Acquire))
            {
                return Err("An online conversion is already running".into());
            }
            if jobs.len() >= 32 && !jobs.contains_key(&path) {
                jobs.retain(|_, j| j.running.load(Ordering::Acquire));
                if jobs.len() >= 32 {
                    return Err("Too many database conversions are active".into());
                }
            }
            jobs.insert(path.clone(), job.clone());
        }
        if let Err(error) = publish_status(&path, &job, "preparing", None, None) {
            // Persistence can fail before a worker exists. Keep the process cache
            // terminal even when the diagnostic sidecar cannot be updated.
            {
                let mut report = job.report.lock().unwrap_or_else(|p| p.into_inner());
                report.phase = "failed".into();
                report.updated_at = now();
                report.error = Some(error.chars().take(512).collect());
            }
            job.running.store(false, Ordering::Release);
            return Err(error);
        }
        let live = live.clone();
        tokio::spawn(async move {
            let result = convert(live, &path, &options, &job).await;
            if let Err(error) = result {
                let phase = if job.cancel.load(Ordering::Acquire) {
                    "cancelled"
                } else {
                    "failed"
                };
                let _ = publish_status(
                    &path,
                    &job,
                    phase,
                    None,
                    Some(error.chars().take(512).collect()),
                );
            }
            job.running.store(false, Ordering::Release);
        });
        Ok(report)
    }
    /// Recover abandoned capture on the currently selected generation, never select an old one.
    pub async fn recover_online_conversion(&self) -> Result<(), String> {
        let Backend::Sqlite(live) = &self.backend else {
            return Ok(());
        };
        if !live.is_managed() {
            return Ok(());
        }
        if JOBS
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(live.logical_path())
            .is_some_and(|j| j.running.load(Ordering::Acquire))
        {
            return Ok(());
        }
        let pin = live.pin().await;
        if live.files().is_some_and(|files| files.conversion_active()) {
            return Err("Private conversion workers may still exist; restart the server process before recovery".into());
        }
        let options = (*pin.active_options()).clone();
        crate::online_copy::cleanup_abandoned(options)
            .await
            .map_err(|e| e.to_string())?;
        if let Some(mut report) = self.online_conversion_status() {
            let files = live.files().ok_or("Missing generation owner")?;
            let active = files.resolve_active().map_err(|e| e.to_string())?;
            // Startup owns the lifecycle fence, so no worker from the stopped
            // process can still hold this unpublished target. Never clean a
            // completed copy or any generation selected by CURRENT.
            if report.phase != "completed"
                && report.source_generation.as_deref() == Some(active.id.as_str())
            {
                if let Some(id) = report
                    .target_generation
                    .as_deref()
                    .filter(|id| *id != active.id)
                {
                    let target = files.reserved_generation(id).map_err(|e| e.to_string())?;
                    files
                        .discard_closed_generation(&active, &target)
                        .map_err(|e| e.to_string())?;
                }
            }
            if !terminal(&report.phase) {
                if report.target_generation.as_deref() == Some(active.id.as_str()) {
                    report.phase = "completed".into();
                    report.retained_previous = true;
                    report.error = None;
                } else {
                    report.phase = "failed".into();
                    report.error=Some("The server restarted before publication; the selected database remains authoritative. Start a new conversion.".into());
                }
                report.updated_at = now();
                // Same-process recovery must update the same cache consulted by
                // status(), even if the following sidecar write fails. Never
                // replace a newer job that started while recovery was working.
                if let Some(job) = JOBS
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .get(live.logical_path())
                {
                    if !job.running.load(Ordering::Acquire) {
                        let mut cached = job.report.lock().unwrap_or_else(|p| p.into_inner());
                        if cached.id == report.id {
                            *cached = report.clone();
                        }
                    }
                }
                generation_files::private_atomic_write(
                    &status_path(live.logical_path()),
                    &serde_json::to_vec(&report).map_err(|_| "Could not encode recovery status")?,
                )
                .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}
async fn pool(options: SqliteConnectOptions) -> Result<SqlitePool, String> {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            options
                .foreign_keys(true)
                .pragma("recursive_triggers", "ON"),
        )
        .await
        .map_err(|e| e.to_string())
}
fn check_resources(
    source: &Path,
    target: &Path,
    options: &ConversionOptions,
    job: &Job,
) -> Result<(), String> {
    if job.cancel.load(Ordering::Acquire) {
        return Err("Online conversion was cancelled; the source remains authoritative".into());
    }
    let wal = sidecar(source, "-wal");
    if std::fs::metadata(wal).map(|m| m.len()).unwrap_or(0) > options.max_wal_mib * 1024 * 1024 {
        return Err(
            "Source WAL pressure exceeded the conversion budget; retry after long readers finish"
                .into(),
        );
    }
    let parent = target.parent().ok_or("No data directory")?;
    let free = mw_platform_fs::available_space(parent)
        .map_err(|e| format!("Could not inspect free disk space: {e}"))?;
    if free < 512 * 1024 * 1024 {
        return Err("Conversion stopped before exhausting local disk space".into());
    }
    Ok(())
}
async fn convert(
    live: LiveSqlite,
    path: &Path,
    options: &ConversionOptions,
    job: &Job,
) -> Result<(), String> {
    let files = live
        .files()
        .ok_or("Missing exclusive SQLite generation owner")?;
    // Other ordinary SQLite processes may coexist, but live conversion needs
    // sole cooperative ownership from before capture until private workers close.
    let conversion_lease = files.acquire_conversion().map_err(|e| e.to_string())?;
    let _rebuild = live.try_rebuild_guard().map_err(|e| e.to_string())?;
    let source = files.resolve_active().map_err(|e| e.to_string())?;
    let source_options = {
        let pin = live.pin().await;
        (*pin.active_options()).clone()
    };
    let mut initial = live.acquire().await.map_err(|e| e.to_string())?;
    let mode = crate::database_observability::sqlite_pragma_i64(&mut initial, "PRAGMA auto_vacuum")
        .await
        .map_err(|e| e.to_string())?;
    if mode == 2 {
        let _ = publish_status(path, job, "completed", None, None);
        return Ok(());
    }
    if mode != 0 {
        return Err(
            "This conversion supports NONE-mode SQLite databases; the current mode is unchanged"
                .into(),
        );
    }
    let pages = crate::database_observability::sqlite_pragma_i64(&mut initial, "PRAGMA page_count")
        .await
        .map_err(|e| e.to_string())?;
    let free_pages =
        crate::database_observability::sqlite_pragma_i64(&mut initial, "PRAGMA freelist_count")
            .await
            .map_err(|e| e.to_string())?;
    let page_size =
        crate::database_observability::sqlite_pragma_i64(&mut initial, "PRAGMA page_size")
            .await
            .map_err(|e| e.to_string())?;
    drop(initial);
    let required = pages
        .saturating_sub(free_pages)
        .saturating_mul(page_size)
        .max(0) as u64;
    let available = mw_platform_fs::available_space(path.parent().ok_or("No database directory")?)
        .map_err(|e| e.to_string())?;
    if available < required.saturating_add(1024 * 1024 * 1024) {
        return Err(
            "Insufficient free space for a complete replacement plus the safety reserve".into(),
        );
    }
    crate::online_copy::cleanup_abandoned(source_options.clone())
        .await
        .map_err(|e| e.to_string())?;
    let target = files.reserve_generation().map_err(|e| e.to_string())?;
    {
        let mut r = job.report.lock().unwrap_or_else(|p| p.into_inner());
        r.source_generation = Some(source.id.clone());
        r.target_generation = Some(target.id.clone());
    }
    if let Err(error) = publish_status(path, job, "preparing", None, None) {
        let _ = files.discard_reserved(target);
        return Err(error);
    }
    let target_options = source_options
        .clone()
        .filename(&target.path)
        .create_if_missing(false)
        .foreign_keys(false)
        .synchronous(SqliteSynchronous::Full)
        .journal_mode(SqliteJournalMode::Wal);
    let mut copy_fence = crate::lifecycle_copy_guard::CopyFence::new(conversion_lease);
    let mut session = match CopySession::prepare(
        source_options.clone(),
        target_options.clone(),
        CopyOptions {
            batch_rows: options.batch_rows,
            batch_bytes: u64::from(options.batch_mib) * 1024 * 1024,
            journal_limit: options.journal_limit,
            cancel: job.cancel.clone(),
        },
    )
    .await
    {
        Ok(session) => session,
        Err(error) => {
            let _ = crate::online_copy::cleanup_abandoned(source_options.clone()).await;
            let _ = files.discard_reserved(target);
            // prepare returns Cancelled only in its initial check, before
            // opening either private connection.
            if matches!(error, crate::online_copy::CopyError::Cancelled) {
                copy_fence.mark_complete();
                return Err(error.to_string());
            }
            return Err(format!(
                "{error}; private worker shutdown was not confirmed, restart the server before retrying conversion"
            ));
        }
    };
    let mut target_cleanup_safe = true;
    let mut workers_closed = false;
    let mut worker_close_attempted = false;
    let result = async {
        loop {
            check_resources(&source.path, &target.path, options, job)?;
            let progress = session.seed_batch().await.map_err(|e| e.to_string())?;
            let done = progress.seed_complete;
            publish_status(path, job, "copying", Some(progress), None)?;
            if done { break; }
            tokio::time::sleep(Duration::from_millis(options.pause_ms)).await;
        }
        publish_status(path, job, "verifying", None, None)?;
        session.verify_bulk().await.map_err(|e| e.to_string())?;
        let catch_started = Instant::now();
        loop {
            check_resources(&source.path, &target.path, options, job)?;
            let progress = session.catch_up_batch().await.map_err(|e| e.to_string())?;
            let done = progress.pending == 0;
            publish_status(path, job, "catching-up", Some(progress), None)?;
            if done { break; }
            if catch_started.elapsed() > Duration::from_secs(300) {
                return Err("Foreground writes outpaced bounded catch-up; the source remains authoritative".into());
            }
            tokio::task::yield_now().await;
        }
        publish_status(path, job, "waiting-for-cutover", None, None)?;
        let mut exclusive = tokio::time::timeout(Duration::from_millis(options.max_cutover_ms), live.exclusive())
            .await.map_err(|_| "Busy operations prevented a short cutover; retry when the database is quieter")?;
        check_resources(&source.path, &target.path, options, job)?;
        // Acceptance of cancellation and admission to switching share one lock.
        {
            let mut report = job.report.lock().unwrap_or_else(|p| p.into_inner());
            if job.cancel.load(Ordering::Acquire) { return Err("Online conversion was cancelled before cutover".into()); }
            report.phase = "switching".into(); report.updated_at = now();
        }
        publish_status(path, job, "switching", None, None)?;
        session.set_deadline(Instant::now() + Duration::from_millis(options.max_cutover_ms)).await.map_err(|e| e.to_string())?;
        // Never cancel a close/open future: taking a connection out of an Option
        // does not prove its worker stopped. Keep the exclusive gate until all
        // private workers have actually closed and a usable selected pool exists.
        exclusive.close_old().await;
        let prepared = tokio::time::timeout(Duration::from_millis(options.max_cutover_ms), async {
            loop { let p = session.catch_up_batch().await.map_err(|e| e.to_string())?; if p.pending == 0 { break; } }
            session.finalize().await.map_err(|e| e.to_string())?;
            session.cleanup_source().await.map_err(|e| e.to_string())?;
            Ok::<_, String>(())
        }).await;
        let _ = session.clear_deadline().await;
        worker_close_attempted=true;
        let target_close = session.close_target().await;
        let source_close = session.close_source().await;
        workers_closed=target_close.is_ok() && source_close.is_ok();
        target_cleanup_safe = target_close.is_ok();
        let preparation_error = match prepared {
            Ok(Ok(())) => target_close.err().or_else(||source_close.err()).map(|e|e.to_string()),
            Ok(Err(error)) => Some(error),
            Err(_) => Some("Cutover exceeded its SQL queue budget; the original database was reopened".into()),
        };
        if let Some(error) = preparation_error {
            restore_source(&mut exclusive, source_options.clone(), path, job).await;
            return Err(error);
        }
        // If connecting fails, SQLx owns any asynchronous error-path teardown.
        // Preserve this unselected shadow rather than unlink unconfirmed handles.
        target_cleanup_safe = false;
        let next = match pool(target_options.clone().foreign_keys(true).synchronous(SqliteSynchronous::Normal)).await {
            Ok(next) => next,
            Err(error) => { restore_source(&mut exclusive, source_options.clone(), path, job).await; return Err(error); }
        };
        let mode = match next.acquire().await {
            Ok(mut conn) => crate::database_observability::sqlite_pragma_i64(&mut conn,"PRAGMA auto_vacuum").await,
            Err(error) => Err(error),
        };
        if !matches!(mode, Ok(2)) {
            next.close().await; target_cleanup_safe = true;
            restore_source(&mut exclusive, source_options.clone(), path, job).await;
            return Err("Replacement database could not be verified in incremental mode".into());
        }
        if let Err(error) = publish_status(path, job, "publishing", None, None) {
            next.close().await; target_cleanup_safe = true;
            restore_source(&mut exclusive, source_options.clone(), path, job).await;
            return Err(error);
        }
        let selected = match files.publish(&source, &target) {
            Ok(active) => active,
            Err(_) => {
                let _ = publish_status(path, job, "recovery-required", None, Some("Verifying durable generation publication; database access remains fenced".into()));
                loop { match files.recover_publication() {
                    Ok(active) => break active,
                    Err(_) => tokio::time::sleep(Duration::from_secs(2)).await,
                }}
            }
        };
        if selected.id == target.id {
            exclusive.replace(next);
            job.report.lock().unwrap_or_else(|p|p.into_inner()).retained_previous = true;
        } else if selected.id == source.id {
            next.close().await; target_cleanup_safe = true;
            restore_source(&mut exclusive, source_options.clone(), path, job).await;
            return Err("Publication did not select the replacement; the original database was reopened".into());
        } else {
            next.close().await;
            let _ = publish_status(path, job, "recovery-required", None, Some("Unexpected selected generation; database access remains fenced for operator recovery".into()));
            std::future::pending::<()>().await; unreachable!();
        }
        drop(exclusive);
        let _ = publish_status(path, job, "completed", None, None);
        Ok(())
    }.await;
    if result.is_err() {
        let _ = session.clear_deadline().await;
        let target_closed = if worker_close_attempted {
            workers_closed
        } else {
            session.close_target().await.is_ok()
        };
        let source_closed = if worker_close_attempted {
            workers_closed
        } else {
            session.close_source().await.is_ok()
        };
        workers_closed = target_closed && source_closed;
        if files
            .resolve_active()
            .is_ok_and(|active| active.id == source.id)
        {
            let _ = crate::online_copy::cleanup_abandoned(source_options.clone()).await;
            if target_closed && target_cleanup_safe {
                let _ = files.discard_closed_generation(&source, &target);
            }
        }
    }
    if workers_closed && (target_cleanup_safe || result.is_ok()) {
        copy_fence.mark_complete();
    }
    if result.is_err() && (!target_cleanup_safe || !workers_closed) {
        return result.map_err(|error|format!("{error}; temporary worker shutdown was not confirmed, restart the server before retrying conversion"));
    }
    result
}

// Before publication, reopening the known source is safe. If disk/OS failures
// prevent it, retain admission fencing and retry rather than admitting work on
// a closed pool or silently choosing a different generation.
async fn restore_source(
    exclusive: &mut ExclusiveSqlite,
    options: SqliteConnectOptions,
    path: &Path,
    job: &Job,
) {
    loop {
        match pool(options.clone()).await {
            Ok(pool) => {
                exclusive.replace(pool);
                return;
            }
            Err(_) => {
                let _ = publish_status(
                    path,
                    job,
                    "recovery-required",
                    None,
                    Some("Reopening the selected source; database access remains fenced".into()),
                );
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

#[cfg(test)]
#[path = "online_maintenance_tests.rs"]
mod tests;
