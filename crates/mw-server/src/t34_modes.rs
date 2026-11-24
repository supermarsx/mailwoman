//! t34 operating modes at the process boundary: one startup check, one log
//! line, and the graceful-shutdown drain.
//!
//! The three switches (`MW_GUARD_SNAPSHOT`, `MW_INDEX_PUBLICATION_ACK`,
//! `MW_REQUEST_PROTECTION_ACCOUNTING`, with their tuning variables) and
//! `MW_SQLITE_STARTUP_TIMEOUT_SECONDS` and `MW_SQLITE_WRITER` are parsed by
//! their own modules. This
//! module runs every parser once before the database opens, so an invalid
//! value stops `serve` with one error naming the variable instead of being
//! found later (or, for the guard snapshot, silently treated as `off`).
//!
//! Combinations:
//! * PostgreSQL + `MW_INDEX_PUBLICATION_ACK=journal` (engine mode): refused.
//! * PostgreSQL + `MW_GUARD_SNAPSHOT=bounded`: accepted, inert (no mirror is
//!   started; every request reads fresh). Logged as a warning.
//! * PostgreSQL + `MW_REQUEST_PROTECTION_ACCOUNTING=write-behind`: accepted,
//!   accounting stays synchronous. Logged as a warning.
//! * Proxy mode: the index publication variables are not used (no engine).
//! * A second process on the same SQLite file in journal mode is refused by
//!   `engine_mode::build_engine` (the `.index-journal.lock` sidecar), which
//!   needs the file system and runs after this check.
//!
//! With every variable unset the result is all-off and nothing else changes.
//!
//! Shutdown: `build_app` registers the app state weakly ([`register`]);
//! `serve` upgrades it ([`hold_for_shutdown`]) so the engine, protection
//! runtime and metric recorders outlive the HTTP server; after HTTP and
//! WebSocket actions have stopped, [`drain_for_shutdown`] waits for accepted
//! index publication jobs (at most `MW_INDEX_PUBLICATION_DRAIN_SECS`), commits
//! queued write-behind observations (at most [`ACCOUNTING_FLUSH_BUDGET`]) and
//! saves the request-history and client-timing buckets (at most 5 s each)
//! before the handles, and with them the store, are released.
use crate::request_inputs::snapshot;
use crate::request_protection::{Runtime, write_behind};
use mw_engine::Engine;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

/// Longest time graceful shutdown spends committing queued write-behind
/// accounting. Observations still queued after it are lost, as on a crash.
pub const ACCOUNTING_FLUSH_BUDGET: Duration = Duration::from_secs(5);

/// Effective t34 modes after validation.
#[derive(Debug)]
pub struct Modes {
    pub(crate) guard: snapshot::Mode,
    /// `false` when the guard snapshot was requested on PostgreSQL (inert).
    pub(crate) guard_live: bool,
    /// `None` in proxy mode (no engine).
    pub(crate) index: Option<mw_engine::PublicationOptions>,
    pub(crate) accounting: write_behind::AccountingConfig,
    /// The accounting mode actually used (`synchronous` on PostgreSQL).
    pub(crate) accounting_effective: write_behind::AccountingMode,
    /// Warnings for accepted but inert combinations.
    pub warnings: Vec<String>,
}

fn is_postgres(db_path: &str) -> bool {
    db_path.starts_with("postgres://") || db_path.starts_with("postgresql://")
}

fn index_variable_set(var: &impl Fn(&str) -> Option<String>) -> bool {
    [
        "MW_INDEX_PUBLICATION_ACK",
        "MW_INDEX_PUBLICATION_WINDOW_MS",
        "MW_INDEX_PUBLICATION_MAX_BATCH_JOBS",
        "MW_INDEX_PUBLICATION_DRAIN_SECS",
        "MW_INDEX_READ_FENCE_TIMEOUT_MS",
    ]
    .iter()
    .any(|name| var(name).is_some_and(|value| !value.trim().is_empty()))
}

/// Pure validation over a variable lookup. `Err` names the variable.
pub(crate) fn validate_from(
    db_path: &str,
    engine: bool,
    var: impl Fn(&str) -> Option<String>,
) -> Result<Modes, String> {
    let postgres = is_postgres(db_path);
    let mut warnings = Vec::new();

    let guard = snapshot::parse(
        var("MW_GUARD_SNAPSHOT").as_deref(),
        var("MW_GUARD_SNAPSHOT_MAX_STALENESS_MS").as_deref(),
    )?;
    let guard_live = !matches!(guard, snapshot::Mode::Off) && !postgres;
    if !matches!(guard, snapshot::Mode::Off) && postgres {
        warnings.push(
            "MW_GUARD_SNAPSHOT=bounded has no effect on PostgreSQL; every request reads authority state fresh".into(),
        );
    }

    let index = if engine {
        Some(crate::engine_mode::publication_options_from(db_path, &var)?)
    } else {
        if index_variable_set(&var) {
            warnings.push(
                "MW_INDEX_PUBLICATION_* / MW_INDEX_READ_FENCE_TIMEOUT_MS are set but not used in proxy mode".into(),
            );
        }
        None
    };

    let accounting = write_behind::AccountingConfig::parse(
        var(write_behind::MODE_VAR).as_deref(),
        var(write_behind::FLUSH_VAR).as_deref(),
        var(write_behind::MAX_PENDING_VAR).as_deref(),
    )?;
    let accounting_effective = accounting.mode.effective(!postgres);
    if accounting_effective != accounting.mode {
        warnings.push(format!(
            "{}=write-behind is SQLite-only; accounting stays synchronous on PostgreSQL",
            write_behind::MODE_VAR
        ));
    }

    crate::inbound_http::HttpLimits::validate_env(&var)?; // t57: HTTP limits, request queue, MW_HTTP_ADAPTIVE_CONCURRENCY

    // Same rule as mw-store's `sqlite_startup::budget`: when set, an integer
    // from 1 to 86400 (an empty value is not the default there either).
    if !postgres
        && let Some(raw) = var("MW_SQLITE_STARTUP_TIMEOUT_SECONDS")
        && !raw.parse::<u64>().is_ok_and(|seconds| (1..=86_400).contains(&seconds))
    {
        // The same text mw-store reports (docs/admin/database-startup.md).
        return Err("MW_SQLITE_STARTUP_TIMEOUT_SECONDS must be an integer from 1 to 86400".into());
    }
    // MW_SQLITE_WRITER (in-process write gate): mw-store's own parser, so the
    // rule and the text are the ones the store applies when it opens SQLite.
    if !postgres {
        mw_store::parse_sqlite_writer_mode(var("MW_SQLITE_WRITER").as_deref())?;
    }

    Ok(Modes { guard, guard_live, index, accounting, accounting_effective, warnings })
}

impl Modes {
    /// The one startup log line.
    pub fn summary(&self) -> String {
        let guard = match self.guard {
            snapshot::Mode::Off => "off".to_owned(),
            snapshot::Mode::Bounded(bound) if self.guard_live => {
                format!("bounded ({} ms)", bound.as_millis())
            }
            snapshot::Mode::Bounded(_) => "bounded (inert on PostgreSQL)".to_owned(),
        };
        let index = match &self.index {
            None => "not used (proxy mode)".to_owned(),
            Some(options) => format!(
                "{} (read fence {} ms, shutdown drain {} s)",
                options.ack.as_str(),
                options.read_fence_timeout.as_millis(),
                options.drain_timeout.as_secs()
            ),
        };
        let accounting = match self.accounting_effective {
            write_behind::AccountingMode::Synchronous => "synchronous".to_owned(),
            write_behind::AccountingMode::WriteBehind => format!(
                "write-behind (flush {} ms, at most {} queued)",
                self.accounting.flush_every.as_millis(),
                self.accounting.max_pending
            ),
        };
        format!(
            "t34 modes: guard snapshot {guard}; index publication {index}; request-protection accounting {accounting}"
        )
    }
}

/// Validate every t34 switch from the process environment before the database
/// opens and log the effective modes once. `Err` means: refuse to start.
pub fn validate_startup(db_path: &str, engine: bool) -> Result<Modes, String> {
    let modes = validate_from(db_path, engine, |name| std::env::var(name).ok())?;
    // The guard snapshot caches its parse for the process; it read the same
    // (frozen) environment, so this only confirms both agree.
    let cached = snapshot::validate_startup()?;
    debug_assert_eq!(cached, modes.guard);
    for warning in &modes.warnings {
        tracing::warn!("{warning}");
    }
    tracing::info!("{}", modes.summary());
    Ok(modes)
}

// ---- graceful shutdown ----------------------------------------------------

/// What graceful shutdown drains. Holding it keeps the engine (and the store
/// it owns), the protection runtime and the metric recorders alive until
/// [`Held::drain`] ends.
pub(crate) struct Held {
    store: mw_store::Store,
    engine: Option<Arc<Engine>>,
    protection: Arc<Runtime>,
    request_history: Arc<crate::request_history::Recorder>,
    client_timing: Arc<crate::client_timing::Recorder>,
}

/// Outcome of the shutdown drain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrainReport {
    pub index_drained: bool,
    pub accounting_flushed: bool,
    pub request_history_saved: bool,
    pub client_timing_saved: bool,
}
impl DrainReport {
    const NOTHING_HELD: Self = Self {
        index_drained: true,
        accounting_flushed: true,
        request_history_saved: true,
        client_timing_saved: true,
    };
}

static REGISTERED: Mutex<Option<Weak<crate::AppStateInner>>> = Mutex::new(None);
static HELD: Mutex<Option<Held>> = Mutex::new(None);

/// Called by `build_app` once the state exists: the latest app, weakly (tests
/// that build many apps keep nothing alive through this).
pub(crate) fn register(state: &crate::AppState) {
    if let Ok(mut slot) = REGISTERED.lock() {
        *slot = Some(Arc::downgrade(&state.0));
    }
}

/// Called by `serve` right after `build_app`: keep the registered app's
/// handles alive until [`drain_for_shutdown`]. Returns whether anything was held.
pub fn hold_for_shutdown() -> bool {
    let held = REGISTERED
        .lock()
        .ok()
        .and_then(|mut slot| slot.take())
        .and_then(|weak| weak.upgrade())
        .map(|state| Held {
            store: state.store.clone(),
            engine: state.engine.clone(),
            protection: state.request_protection.clone(),
            request_history: state.request_history.clone(),
            client_timing: state.client_timing.clone(),
        });
    let some = held.is_some();
    if let Ok(mut slot) = HELD.lock() {
        *slot = held;
    }
    some
}

/// The store held for shutdown, for start-up readers such as the HTTP limits.
pub(crate) fn held_store() -> Option<mw_store::Store> {
    HELD.lock().ok()?.as_ref().map(|held| held.store.clone())
}

/// Budget for saving each metric recorder at shutdown.
const RECORDER_SAVE_BUDGET: Duration = Duration::from_secs(5);

impl Held {
    #[cfg(test)]
    pub(crate) fn new(
        store: mw_store::Store,
        engine: Option<Arc<Engine>>,
        protection: Arc<Runtime>,
        request_history: Arc<crate::request_history::Recorder>,
        client_timing: Arc<crate::client_timing::Recorder>,
    ) -> Self {
        Self { store, engine, protection, request_history, client_timing }
    }

    /// Index publication first (its jobs were accepted by requests that have
    /// already answered), then write-behind accounting, then the request
    /// history and client timing buckets. Each step has its own budget; the
    /// handles drop when this returns.
    pub(crate) async fn drain(self, accounting_budget: Duration) -> DrainReport {
        let index_drained = match &self.engine {
            Some(engine) => crate::engine_mode::drain_index_publication(engine).await,
            None => true,
        };
        let deadline = tokio::time::Instant::now() + accounting_budget;
        let accounting_flushed = match self.protection.flush_all(deadline).await {
            Ok(()) => true,
            Err(incomplete) => {
                tracing::warn!(
                    "request-protection accounting: shutdown flush incomplete ({incomplete:?}); the queued observations are lost"
                );
                false
            }
        };
        let saved = |what: &str, result: Result<Result<(), mw_store::StoreError>, tokio::time::error::Elapsed>| match result {
            Ok(Ok(())) => true,
            _ => {
                tracing::warn!("{what}: shutdown save did not complete; the last interval is not stored");
                false
            }
        };
        let _ = tokio::time::timeout(RECORDER_SAVE_BUDGET, self.store.flush_session_touches()).await; // t50hp-e22: buffered last_seen
        let request_history_saved = saved(
            "request history",
            tokio::time::timeout(RECORDER_SAVE_BUDGET, self.request_history.flush_now(&self.store)).await,
        );
        let client_timing_saved = saved(
            "client timing",
            tokio::time::timeout(RECORDER_SAVE_BUDGET, self.client_timing.flush_now(&self.store)).await,
        );
        DrainReport { index_drained, accounting_flushed, request_history_saved, client_timing_saved }
    }
}

/// Graceful shutdown step, after HTTP acceptance and WebSocket actions have
/// stopped and before telemetry flushing. A no-op unless
/// [`hold_for_shutdown`] held something.
pub async fn drain_for_shutdown() -> DrainReport {
    let held = HELD.lock().ok().and_then(|mut slot| slot.take());
    match held {
        Some(held) => held.drain(ACCOUNTING_FLUSH_BUDGET).await,
        None => DrainReport::NOTHING_HELD,
    }
}

#[cfg(test)]
#[path = "t34_modes_tests.rs"]
mod tests;
