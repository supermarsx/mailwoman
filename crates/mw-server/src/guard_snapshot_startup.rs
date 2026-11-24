//! t49pg-e3: start-up validation and run-time health of the bounded guard
//! snapshot on PostgreSQL.
//!
//! `MW_GUARD_SNAPSHOT=bounded` on PostgreSQL is an explicit opt-in that needs
//! more of the database than SQLite does. At start-up (after the store is
//! open, before the listener binds) [`validate`] checks, in this order, and
//! refuses to start with a message naming the first one that fails:
//!
//! 1. `server_version_num >= 130000` (the change log's transaction id is `xid8`);
//! 2. `pg_is_in_recovery()` is false (a standby or read replica has no commit
//!    order the poll can rely on and cannot be pruned);
//! 3. `authority_change_log.xid` exists with type `xid8` (migration 0120);
//! 4. the required trigger and function definitions are present, enabled and
//!    unaltered: `Store::validate_current_native_publication_schema`, the
//!    validator the request path runs cold, with the list defined only there;
//! 5. the mirror's dedicated poll connection opens and one complete poll
//!    returns the schema stamp.
//!
//! Mode off, and SQLite in either mode, run none of this and open no extra
//! connection.
//!
//! After start-up a failure does not exit. The mirror already stops reuse when
//! it is not fresh (every request then reads fresh). [`Monitor`] adds the
//! operator-facing side: when the mirror has been not fresh for longer than a
//! grace period it logs ONE warning, sets the `mw_guard_snapshot_degraded`
//! gauge and the witness flag; when polls succeed again it logs ONE recovery
//! and clears them. No per-request log lines.
use crate::request_inputs::snapshot::Mode;
use mw_store::Store;
use mw_store::authority_publication::{PostgresSnapshotFacts, mirror::EpochMirror};
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::{Duration, Instant};

/// First PostgreSQL release with `xid8` and `pg_snapshot`.
pub(crate) const MIN_SERVER_VERSION_NUM: i64 = 130_000;
const HOW_TO_TURN_OFF: &str = "set MW_GUARD_SNAPSHOT=off to start without the guard snapshot";
/// Attempts at the first complete poll: freshness is measured against the
/// bound, so one slow poll can finish already stale.
const FIRST_POLL_ATTEMPTS: usize = 3;

/// The mirror started for the check; kept so the mirror the first request
/// asks for is this warm one. Released at shutdown.
static STARTED: Mutex<Option<Arc<EpochMirror>>> = Mutex::new(None);
static CURRENT: Mutex<Option<Arc<Status>>> = Mutex::new(None);

/// Pure part of checks 1 to 3 (the seam for a simulated server).
pub(crate) fn check_facts(facts: &PostgresSnapshotFacts) -> Result<(), String> {
    if facts.server_version_num < MIN_SERVER_VERSION_NUM {
        return Err(format!(
            "MW_GUARD_SNAPSHOT=bounded on PostgreSQL needs PostgreSQL 13 or later (the change log uses xid8); this server reports server_version_num {}; upgrade PostgreSQL or {HOW_TO_TURN_OFF}",
            facts.server_version_num
        ));
    }
    if facts.in_recovery {
        return Err(format!(
            "MW_GUARD_SNAPSHOT=bounded on PostgreSQL needs the primary: pg_is_in_recovery() is true, so the database URL points at a standby or read replica; point it at the primary or {HOW_TO_TURN_OFF}"
        ));
    }
    if !facts.change_log_xid_column {
        return Err(format!(
            "MW_GUARD_SNAPSHOT=bounded on PostgreSQL needs the column authority_change_log.xid (type xid8, migration 0120), which is missing or has another type; apply the database migrations or {HOW_TO_TURN_OFF}"
        ));
    }
    Ok(())
}

/// Validate for `mode` against `store`. Mode off and SQLite: `Ok` without
/// touching the database.
pub(crate) async fn validate(store: &Store, mode: Mode) -> Result<(), String> {
    let Mode::Bounded(bound) = mode else { return Ok(()) };
    let facts = match store.postgres_snapshot_facts().await {
        Ok(Some(facts)) => facts,
        Ok(None) => return Ok(()),
        Err(error) => {
            return Err(format!(
                "MW_GUARD_SNAPSHOT=bounded on PostgreSQL: the server version, recovery state and change-log column could not be read ({error}); {HOW_TO_TURN_OFF}"
            ));
        }
    };
    validate_with(store, bound, facts).await
}

/// [`validate`] with the facts supplied (tests simulate an old server or a
/// standby here; checks 4 and 5 always run against the real database).
pub(crate) async fn validate_with(store: &Store, bound: Duration, facts: PostgresSnapshotFacts) -> Result<(), String> {
    check_facts(&facts)?;
    match store.validate_current_native_publication_schema().await {
        Ok(true) => {}
        Ok(false) => {
            return Err(format!(
                "MW_GUARD_SNAPSHOT=bounded on PostgreSQL: the required authority triggers or functions (migrations 0078, 0111, 0112, 0114 and 0120) are missing, disabled or changed; apply the database migrations and re-enable them, or {HOW_TO_TURN_OFF}"
            ));
        }
        Err(error) => {
            return Err(format!(
                "MW_GUARD_SNAPSHOT=bounded on PostgreSQL: the required trigger and function definitions could not be checked ({error}); {HOW_TO_TURN_OFF}"
            ));
        }
    }
    let mirror = EpochMirror::for_store(store, bound);
    if !mirror.is_live() {
        return Err(format!(
            "MW_GUARD_SNAPSHOT=bounded on PostgreSQL: the guard snapshot mirror could not start; {HOW_TO_TURN_OFF}"
        ));
    }
    let mut last = String::new();
    for _ in 0..FIRST_POLL_ATTEMPTS {
        match mirror.poll_now().await {
            Ok(_) if mirror.is_fresh() => {
                *STARTED.lock().map_err(|_| "guard snapshot start-up state poisoned".to_owned())? = Some(mirror.clone());
                Monitor::spawn(&mirror, Monitor::TICK, Monitor::grace(bound));
                return Ok(());
            }
            Ok(_) => last = "the poll finished without a current schema stamp".to_owned(),
            Err(error) => last = error.to_string(),
        }
    }
    Err(format!(
        "MW_GUARD_SNAPSHOT=bounded on PostgreSQL: the dedicated poll connection could not be established or one complete poll did not succeed ({last}); the poll needs one extra connection to the primary that allows REPEATABLE READ; check the connection limit and the database URL or {HOW_TO_TURN_OFF}"
    ))
}

/// Drop the start-up reference (graceful shutdown); the monitor and the poll
/// task end with the last reference.
pub(crate) fn release() {
    if let Ok(mut slot) = STARTED.lock() {
        *slot = None;
    }
}

// ---- run-time health --------------------------------------------------------

/// What the monitor reports. Atomics only: read by the witness and the gauge.
#[derive(Default)]
pub(crate) struct Status {
    degraded: AtomicBool,
    degradations: AtomicU64,
    recoveries: AtomicU64,
}
impl Status {
    pub(crate) fn degraded(&self) -> bool {
        self.degraded.load(Ordering::Relaxed)
    }
    pub(crate) fn degradations(&self) -> u64 {
        self.degradations.load(Ordering::Relaxed)
    }
    pub(crate) fn recoveries(&self) -> u64 {
        self.recoveries.load(Ordering::Relaxed)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Transition {
    Degraded,
    Recovered,
}

/// The state machine: degraded after `grace` of continuous not-fresh
/// observations, recovered on the first fresh one. Pure: time is an input.
#[derive(Default)]
pub(crate) struct Health {
    degraded: bool,
    not_fresh_since: Option<Instant>,
}
impl Health {
    pub(crate) fn observe(&mut self, fresh: bool, now: Instant, grace: Duration) -> Option<Transition> {
        if fresh {
            self.not_fresh_since = None;
            return std::mem::take(&mut self.degraded).then_some(Transition::Recovered);
        }
        let since = *self.not_fresh_since.get_or_insert(now);
        if !self.degraded && now.saturating_duration_since(since) >= grace {
            self.degraded = true;
            return Some(Transition::Degraded);
        }
        None
    }
}

pub(crate) struct Monitor;
impl Monitor {
    pub(crate) const TICK: Duration = Duration::from_millis(250);
    /// Long enough that one slow poll or a brief connection reset (the next
    /// poll reconnects) is not a state change.
    pub(crate) fn grace(bound: Duration) -> Duration {
        (bound * 2).max(Duration::from_secs(2))
    }

    /// Watches `mirror` until its last strong reference is gone. Returns the
    /// status it updates (also published for the witness).
    pub(crate) fn spawn(mirror: &Arc<EpochMirror>, tick: Duration, grace: Duration) -> Arc<Status> {
        let status = Arc::new(Status::default());
        if let Ok(mut current) = CURRENT.lock() {
            *current = Some(status.clone());
        }
        metrics::gauge!("mw_guard_snapshot_degraded", "backend" => "postgresql").set(0.0);
        let weak: Weak<EpochMirror> = Arc::downgrade(mirror);
        let shared = status.clone();
        tokio::spawn(async move {
            let mut health = Health::default();
            loop {
                tokio::time::sleep(tick).await;
                let Some(mirror) = weak.upgrade() else { break };
                let fresh = mirror.is_fresh();
                drop(mirror);
                match health.observe(fresh, Instant::now(), grace) {
                    Some(Transition::Degraded) => {
                        shared.degraded.store(true, Ordering::Relaxed);
                        shared.degradations.fetch_add(1, Ordering::Relaxed);
                        metrics::gauge!("mw_guard_snapshot_degraded", "backend" => "postgresql").set(1.0);
                        tracing::warn!(
                            "guard snapshot degraded on PostgreSQL: the change-log poll has not succeeded for {} ms; reuse is stopped and every request reads authority state fresh until polls succeed again",
                            grace.as_millis()
                        );
                    }
                    Some(Transition::Recovered) => {
                        shared.degraded.store(false, Ordering::Relaxed);
                        shared.recoveries.fetch_add(1, Ordering::Relaxed);
                        metrics::gauge!("mw_guard_snapshot_degraded", "backend" => "postgresql").set(0.0);
                        tracing::info!("guard snapshot recovered on PostgreSQL: polls succeed again; reuse resumed");
                    }
                    None => {}
                }
            }
        });
        status
    }
}

/// Load-fixture witness: whether the monitor runs and what it saw. Counts and
/// booleans only.
#[cfg_attr(not(any(test, feature = "load-test-fixtures")), allow(dead_code))]
pub(crate) fn witness() -> serde_json::Value {
    match CURRENT.lock().ok().and_then(|current| current.clone()) {
        Some(status) => serde_json::json!({
            "validated": true, "degraded": status.degraded(),
            "degradations": status.degradations(), "recoveries": status.recoveries(),
        }),
        None => serde_json::json!({"validated": false, "degraded": false, "degradations": 0, "recoveries": 0}),
    }
}

#[cfg(test)]
#[path = "guard_snapshot_startup_tests.rs"]
mod tests;
