//! Scheduler and background jobs (26.20 t32-e72, worker f12; mini-slot `t32-jobs`).
//!
//! A **job kind** is a named piece of work with a run function, an optional default
//! schedule and a lease length ([`JobSpec`]). Kinds are listed in [`builtin`]
//! (one `r.register(..)` line per kind; later lanes add theirs there) or added at
//! run time with [`register`]. Nothing runs unless an administrator creates an
//! enabled `scheduled_task` for the kind or presses "Run now" (DC7: no task is
//! enabled by default).
//!
//! One scheduler loop per process ([`start`], started from `build_app`) wakes every
//! [`tick`] and:
//! 1. marks `running` rows whose heartbeat is older than 3x their kind's lease as
//!    `abandoned` (that frees the kind again);
//! 2. gives enabled tasks without a next run their first one;
//! 3. for each due task, takes the task's **DB lease** ([`mw_store::Store::claim_task_lease`])
//!    and, holding it, starts a run. With several processes on one database a due
//!    task runs once; a lease nobody renews is taken over when it expires.
//!
//! A run holds its lease and heartbeat fresh every third of the lease. When the
//! heartbeat finds its row no longer `running` (another process marked it
//! abandoned) the run is stopped. At most one run per kind is `running` at a
//! time ([`mw_store::Store::start_job_run`]).
//!
//! Loops that schedule themselves (e67's usage measurer) report their runs with
//! [`observe`], so they show in the same history.
//!
//! Schedules ([`Schedule`]) are `{everySecs}`, `{dailyAt, tz}`,
//! `{weekly:{days, at, tz}}` or `{monthly:{day, at, tz}}`. Times are wall-clock times
//! in `tz` ([`local_at`]): a time that falls in a skipped DST hour runs at the next
//! valid minute, a repeated hour runs once (the first time).

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Duration;

use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use mw_store::jobs::{JobRunRow, JobStatus, ScheduledTaskRow};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::AppState;
#[path = "jobs_window.rs"]
mod window;

// ─── job kinds ──────────────────────────────────────────────────────────────────

/// What a run returns: a one-line summary for lists and a detail object. Neither
/// may carry mail content, subjects or secrets (they are shown in the console and
/// included in support bundles).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct JobOutput {
    pub summary: String,
    pub detail: Value,
}

impl JobOutput {
    pub fn new(summary: impl Into<String>, detail: Value) -> Self {
        Self {
            summary: summary.into(),
            detail,
        }
    }
}

/// The future a run function returns. `Err` = the run failed, with a short reason.
pub type JobFuture = Pin<Box<dyn Future<Output = Result<JobOutput, String>> + Send>>;

/// A run function: `|ctx| Box::pin(my_job(ctx))`.
pub type JobFn = fn(JobCtx) -> JobFuture;

/// Everything a run gets.
pub struct JobCtx {
    pub kind: String,
    /// The `job_run` id.
    pub run_id: String,
    /// The scheduled task that started the run; `None` for "Run now".
    pub task_id: Option<String>,
    /// The task's (or the "Run now" request's) parameters; `{}` when none.
    pub params: Value,
    /// The admin who pressed "Run now", or `scheduler`.
    pub triggered_by: String,
    pub(crate) state: Option<AppState>,
}

impl JobCtx {
    /// The application state. `None` only for runs started by unit tests.
    pub(crate) fn state(&self) -> Result<&AppState, String> {
        self.state
            .as_ref()
            .ok_or_else(|| "no application state".to_string())
    }
}

/// A job kind.
#[derive(Clone)]
pub struct JobSpec {
    pub run: JobFn,
    /// Offered when an administrator creates a task for the kind. Never applied by
    /// itself.
    pub default_schedule: Option<Schedule>,
    /// How long a run holds its task's lease between heartbeats; a run whose
    /// heartbeat is older than 3x this is abandoned.
    pub lease: Duration,
}

impl JobSpec {
    /// A kind with no default schedule and a 5 minute lease.
    pub fn new(run: JobFn) -> Self {
        Self {
            run,
            default_schedule: None,
            lease: Duration::from_secs(300),
        }
    }

    pub fn schedule(mut self, s: Schedule) -> Self {
        self.default_schedule = Some(s);
        self
    }

    pub fn lease(mut self, lease: Duration) -> Self {
        self.lease = lease;
        self
    }

    fn lease_secs(&self) -> i64 {
        // Whole-second DB timestamps need at least two seconds so renewal every
        // third of the lease can cross a clock boundary without expiring first.
        self.lease.as_secs().clamp(2, 86_400) as i64
    }
}

/// Something job kinds are registered with.
pub struct Registry(BTreeMap<String, JobSpec>);

impl Registry {
    pub fn register(&mut self, kind: &str, spec: JobSpec) {
        assert!(
            valid_kind(kind),
            "job kind {kind:?} must match [a-z][a-z0-9_]{{0,47}}"
        );
        assert!(
            !self.0.contains_key(kind),
            "job kind {kind} registered twice"
        );
        self.0.insert(kind.to_string(), spec);
    }
}

/// The built-in kinds. Landing order follows `t32-jobs` (t32-fast §4): later lanes
/// add `db_backup`, `update_check` (f13), `audit_prune` (f30), `diag_suite` (f14),
/// `metric_rollup` (f09) and `data_export` (f07) here.
fn builtin(r: &mut Registry) {
    // f12 (e72): store and index upkeep.
    r.register(
        "vacuum",
        JobSpec::new(|ctx| Box::pin(vacuum(ctx))).schedule(Schedule::Weekly {
            days: vec![Weekday::Sun],
            at: "03:30".into(),
            tz: "UTC".into(),
        }),
    );
    r.register(
        "gc_uploads",
        JobSpec::new(|ctx| Box::pin(gc_uploads(ctx))).schedule(Schedule::Daily {
            daily_at: "04:00".into(),
            tz: "UTC".into(),
        }),
    );
    r.register(
        "reindex",
        JobSpec::new(|ctx| Box::pin(reindex(ctx))).lease(Duration::from_secs(600)),
    );
    r.register(
        "rethread_all",
        JobSpec::new(|ctx| Box::pin(rethread_all(ctx))).lease(Duration::from_secs(600)),
    );
}

fn valid_kind(kind: &str) -> bool {
    let b = kind.as_bytes();
    !b.is_empty()
        && b.len() <= 48
        && b[0].is_ascii_lowercase()
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'_')
}

fn registry() -> &'static RwLock<BTreeMap<String, JobSpec>> {
    static REGISTRY: OnceLock<RwLock<BTreeMap<String, JobSpec>>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut r = Registry(BTreeMap::new());
        builtin(&mut r);
        RwLock::new(r.0)
    })
}

/// Add a kind at run time (tests, plugins). Panics on a malformed or duplicate
/// kind, like the built-in list does.
pub fn register(kind: &str, spec: JobSpec) {
    assert!(valid_kind(kind), "invalid job kind {kind:?}");
    let mut map = registry().write().unwrap_or_else(|e| e.into_inner());
    assert!(!map.contains_key(kind), "job kind {kind} registered twice");
    map.insert(kind.to_string(), spec);
}

/// The spec of `kind`, if registered.
pub fn spec(kind: &str) -> Option<JobSpec> {
    registry()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .get(kind)
        .cloned()
}

/// Every registered kind with its spec, in name order.
pub fn kinds() -> Vec<(String, JobSpec)> {
    registry()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .map(|(k, s)| (k.clone(), s.clone()))
        .collect()
}

// ─── schedules ──────────────────────────────────────────────────────────────────

/// When a task runs. Wire shapes (`scheduled_task.schedule_json`):
/// `{"everySecs":3600}`, `{"dailyAt":"03:00","tz":"Europe/Berlin"}`,
/// `{"weekly":{"days":["mon","thu"],"at":"03:00","tz":"UTC"}}`,
/// `{"monthly":{"day":1,"at":"03:00","tz":"UTC"}}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Schedule {
    Every {
        secs: u64,
    },
    Daily {
        daily_at: String,
        tz: String,
    },
    Weekly {
        days: Vec<Weekday>,
        at: String,
        tz: String,
    },
    /// `day` 1-31; in a shorter month the task runs on its last day.
    Monthly {
        day: u32,
        at: String,
        tz: String,
    },
}

/// Bounds of `everySecs`: one minute to 31 days.
pub const EVERY_MIN_SECS: u64 = 60;
pub const EVERY_MAX_SECS: u64 = 31 * 24 * 3600;

#[derive(Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
enum ScheduleWire {
    Every {
        #[serde(rename = "everySecs")]
        every_secs: u64,
    },
    Daily {
        #[serde(rename = "dailyAt")]
        daily_at: String,
        #[serde(default = "utc")]
        tz: String,
    },
    Weekly {
        weekly: WeeklyWire,
    },
    Monthly {
        monthly: MonthlyWire,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WeeklyWire {
    days: Vec<String>,
    at: String,
    #[serde(default = "utc")]
    tz: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MonthlyWire {
    day: u32,
    at: String,
    #[serde(default = "utc")]
    tz: String,
}

fn utc() -> String {
    "UTC".into()
}

const DAY_NAMES: [(&str, Weekday); 7] = [
    ("mon", Weekday::Mon),
    ("tue", Weekday::Tue),
    ("wed", Weekday::Wed),
    ("thu", Weekday::Thu),
    ("fri", Weekday::Fri),
    ("sat", Weekday::Sat),
    ("sun", Weekday::Sun),
];

/// `"mon"` … `"sun"`.
pub fn weekday_name(d: Weekday) -> &'static str {
    DAY_NAMES
        .iter()
        .find(|(_, w)| *w == d)
        .map(|(n, _)| *n)
        .unwrap_or("mon")
}

/// Parse `"mon"` … `"sun"`.
pub fn parse_weekday(s: &str) -> Option<Weekday> {
    DAY_NAMES.iter().find(|(n, _)| *n == s).map(|(_, w)| *w)
}

/// A schedule that does not validate: the JSON pointer of the bad field (relative
/// to the schedule object) and a code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleError {
    pub field: &'static str,
    pub code: &'static str,
}

impl Schedule {
    /// Parse and validate a schedule object.
    pub fn from_json(v: &Value) -> Result<Self, ScheduleError> {
        let bad = |field, code| ScheduleError { field, code };
        let wire: ScheduleWire =
            serde_json::from_value(v.clone()).map_err(|_| bad("", "invalidSchedule"))?;
        let s = match wire {
            ScheduleWire::Every { every_secs } => Schedule::Every { secs: every_secs },
            ScheduleWire::Daily { daily_at, tz } => Schedule::Daily { daily_at, tz },
            ScheduleWire::Weekly { weekly } => {
                let mut days = Vec::new();
                for d in &weekly.days {
                    let day = parse_weekday(d).ok_or(bad("/weekly/days", "invalidDay"))?;
                    if !days.contains(&day) {
                        days.push(day);
                    }
                }
                Schedule::Weekly {
                    days,
                    at: weekly.at,
                    tz: weekly.tz,
                }
            }
            ScheduleWire::Monthly { monthly } => Schedule::Monthly {
                day: monthly.day,
                at: monthly.at,
                tz: monthly.tz,
            },
        };
        s.validate()?;
        Ok(s)
    }

    /// The wire object.
    pub fn to_json(&self) -> Value {
        match self {
            Schedule::Every { secs } => json!({ "everySecs": secs }),
            Schedule::Daily { daily_at, tz } => json!({ "dailyAt": daily_at, "tz": tz }),
            Schedule::Weekly { days, at, tz } => json!({ "weekly": {
                "days": days.iter().map(|d| weekday_name(*d)).collect::<Vec<_>>(),
                "at": at,
                "tz": tz,
            }}),
            Schedule::Monthly { day, at, tz } => {
                json!({ "monthly": { "day": day, "at": at, "tz": tz } })
            }
        }
    }

    fn validate(&self) -> Result<(), ScheduleError> {
        let bad = |field, code| Err(ScheduleError { field, code });
        let (at, tz, at_field, tz_field) = match self {
            Schedule::Every { secs } => {
                if !(EVERY_MIN_SECS..=EVERY_MAX_SECS).contains(secs) {
                    return bad("/everySecs", "outOfRange");
                }
                return Ok(());
            }
            Schedule::Daily { daily_at, tz } => (daily_at, tz, "/dailyAt", "/tz"),
            Schedule::Weekly { days, at, tz } => {
                if days.is_empty() {
                    return bad("/weekly/days", "required");
                }
                (at, tz, "/weekly/at", "/weekly/tz")
            }
            Schedule::Monthly { day, at, tz } => {
                if !(1..=31).contains(day) {
                    return bad("/monthly/day", "outOfRange");
                }
                (at, tz, "/monthly/at", "/monthly/tz")
            }
        };
        if parse_hhmm(at).is_none() {
            return bad(at_field, "invalidTime");
        }
        if parse_tz(tz).is_none() {
            return bad(tz_field, "invalidTimeZone");
        }
        Ok(())
    }

    /// The first run strictly after `now`. `None` only for a schedule that does not
    /// validate.
    pub fn next_after(&self, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            Schedule::Every { secs } => Some(now + chrono::Duration::seconds(*secs as i64)),
            Schedule::Daily { daily_at, tz } => {
                let (tz, at) = (parse_tz(tz)?, parse_hhmm(daily_at)?);
                let today = now.with_timezone(&tz).date_naive();
                (0..3)
                    .filter_map(|i| today.checked_add_days(chrono::Days::new(i)))
                    .filter_map(|d| local_at(tz, d, at))
                    .find(|t| *t > now)
            }
            Schedule::Weekly { days, at, tz } => {
                let (tz, at) = (parse_tz(tz)?, parse_hhmm(at)?);
                let today = now.with_timezone(&tz).date_naive();
                (0..9)
                    .filter_map(|i| today.checked_add_days(chrono::Days::new(i)))
                    .filter(|d| days.contains(&d.weekday()))
                    .filter_map(|d| local_at(tz, d, at))
                    .find(|t| *t > now)
            }
            Schedule::Monthly { day, at, tz } => {
                let (tz, at) = (parse_tz(tz)?, parse_hhmm(at)?);
                let local = now.with_timezone(&tz).date_naive();
                let (mut y, mut m) = (local.year(), local.month());
                for _ in 0..14 {
                    let d = month_day_clamped(y, m, *day)?;
                    let t = local_at(tz, d, at)?;
                    if t > now {
                        return Some(t);
                    }
                    (y, m) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
                }
                None
            }
        }
    }
}

/// `day` of month `m`, or the month's last day when it is shorter.
pub fn month_day_clamped(y: i32, m: u32, day: u32) -> Option<NaiveDate> {
    (1..=day.min(31))
        .rev()
        .find_map(|d| NaiveDate::from_ymd_opt(y, m, d))
}

/// `"HH:MM"` (24 h).
pub fn parse_hhmm(s: &str) -> Option<NaiveTime> {
    let (h, m) = s.split_once(':')?;
    if h.len() != 2 || m.len() != 2 {
        return None;
    }
    NaiveTime::from_hms_opt(h.parse().ok()?, m.parse().ok()?, 0)
}

/// An IANA time zone name (`UTC`, `Europe/Berlin`).
pub fn parse_tz(s: &str) -> Option<chrono_tz::Tz> {
    s.parse::<chrono_tz::Tz>().ok()
}

/// The instant the wall clock in `tz` shows `time` on `date`. A time inside a
/// skipped DST hour moves to the next valid minute; a time inside a repeated hour
/// is its first occurrence.
pub fn local_at(tz: chrono_tz::Tz, date: NaiveDate, time: NaiveTime) -> Option<DateTime<Utc>> {
    let mut naive = date.and_time(time);
    // Some zones skipped an entire date (Pacific/Apia, 2011-12-30).
    // Bound the search and fail closed rather than inventing a UTC occurrence.
    for _ in 0..=(48 * 60) {
        match tz.from_local_datetime(&naive) {
            chrono::LocalResult::Single(t) => return Some(t.with_timezone(&Utc)),
            chrono::LocalResult::Ambiguous(first, _) => return Some(first.with_timezone(&Utc)),
            chrono::LocalResult::None => {
                naive = naive.checked_add_signed(chrono::Duration::minutes(1))?
            }
        }
    }
    None
}

// ─── the scheduler ──────────────────────────────────────────────────────────────

/// How often the scheduler loop wakes.
pub const DEFAULT_TICK: Duration = Duration::from_secs(15);

static TICK_MS: AtomicU64 = AtomicU64::new(15_000);

/// The scheduler loop interval.
pub fn tick() -> Duration {
    Duration::from_millis(TICK_MS.load(Ordering::Relaxed).max(10))
}

/// Test seam: change the loop interval of every scheduler in this process.
#[doc(hidden)]
pub fn set_tick(d: Duration) {
    TICK_MS.store(
        d.as_millis().clamp(10, u64::MAX as u128) as u64,
        Ordering::Relaxed,
    );
}

/// The running scheduler of one app.
pub struct Scheduler {
    /// `host@process-start` (plus `/n` for the n-th app in one process).
    pub(crate) instance: String,
    /// Unix seconds of the last completed loop pass (0 = none yet).
    last_pass: AtomicI64,
}

impl Scheduler {
    pub fn instance(&self) -> &str {
        &self.instance
    }

    /// Whether the loop completed a pass recently (within 3 ticks, at least 60 s).
    /// `/readyz` reports a stalled scheduler from this.
    pub fn alive(&self, now: i64) -> bool {
        let last = self.last_pass.load(Ordering::Relaxed);
        let window = (tick().as_secs() as i64 * 3).max(60);
        last > 0 && now - last <= window
    }

    pub fn last_pass(&self) -> i64 {
        self.last_pass.load(Ordering::Relaxed)
    }
}

fn process_start() -> i64 {
    static START: OnceLock<i64> = OnceLock::new();
    *START.get_or_init(|| Utc::now().timestamp())
}

fn host_name() -> String {
    let from_env = ["HOSTNAME", "COMPUTERNAME"]
        .iter()
        .find_map(|k| std::env::var(k).ok().filter(|s| !s.trim().is_empty()));
    let name = from_env
        .or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "localhost".into());
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_'))
        .take(64)
        .collect()
}

fn next_instance() -> String {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let base = format!(
        "{}@{}:{}:{}",
        host_name(),
        process_start(),
        std::process::id(),
        uuid::Uuid::new_v4()
    );
    if n == 0 { base } else { format!("{base}/{n}") }
}

/// A new scheduler handle (kept in `AppState` for `/readyz` and the jobs API).
/// Its loop starts with [`start`] once the state exists.
pub(crate) fn new_scheduler() -> Arc<Scheduler> {
    Arc::new(Scheduler {
        instance: next_instance(),
        last_pass: AtomicI64::new(0),
    })
}

/// Start the scheduler loop of `state`'s app (from `build_app`).
pub(crate) fn start(state: AppState) {
    tokio::spawn(async move {
        loop {
            if !paused() {
                let s = &state.scheduler;
                match pass(&state, &s.instance).await {
                    Ok(()) => s.last_pass.store(Utc::now().timestamp(), Ordering::Relaxed),
                    Err(e) => tracing::warn!("scheduler pass failed: {e}"),
                }
            }
            tokio::time::sleep(tick()).await;
        }
    });
}

static PAUSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn paused() -> bool {
    PAUSED.load(Ordering::Relaxed)
}

/// Test seam: stop (or resume) every scheduler loop in this process, so a test can
/// show `/readyz` reporting a stalled scheduler.
#[doc(hidden)]
pub fn pause_all(paused: bool) {
    PAUSED.store(paused, Ordering::Relaxed);
}

/// One loop pass.
pub(crate) async fn pass(state: &AppState, instance: &str) -> Result<(), mw_store::StoreError> {
    let store = &state.store;
    let now = Utc::now();
    let ts = now.timestamp();

    // 1. Abandon runs whose heartbeat stopped (3x the kind's lease).
    let running = store.running_job_runs().await?;
    let mut checked = std::collections::BTreeSet::new();
    for r in running {
        if !checked.insert(r.kind.clone()) {
            continue;
        }
        let lease = spec(&r.kind).map_or(300, |s| s.lease_secs());
        let n = store
            .abandon_stale_job_runs(&r.kind, ts - 3 * lease, ts)
            .await?;
        if n > 0 {
            tracing::warn!(kind = %r.kind, "marked {n} job run(s) abandoned (heartbeat stopped)");
        }
    }

    // 2. First run of newly enabled tasks.
    for t in store.unscheduled_tasks().await? {
        if let Some(next) = task_schedule(&t).and_then(|s| s.next_after(now)) {
            store.set_task_first_run(&t.id, next.timestamp()).await?;
        }
    }

    // 3. Due tasks.
    for t in store.due_scheduled_tasks(ts).await? {
        let now = Utc::now();
        let ts = now.timestamp();
        let Some(job) = spec(&t.kind) else {
            // A kind this build does not know (a downgrade, a removed plugin): move
            // the task on so it does not come up on every pass.
            if store.claim_task_lease(&t.id, instance, ts, ts + 60).await? {
                let next = task_schedule(&t).and_then(|s| s.next_after(now));
                store
                    .release_task_lease(
                        &t.id,
                        instance,
                        None,
                        Some("unknownKind"),
                        next.map(|n| n.timestamp()),
                    )
                    .await?;
            }
            continue;
        };
        let lease = job.lease_secs();
        if !store
            .claim_task_lease(&t.id, instance, ts, ts + lease)
            .await?
        {
            continue;
        }
        // The editable snapshot may have changed between the due query and
        // lease acquisition. Start/heartbeat also fence concurrent later edits.
        let Some(t) = store.get_scheduled_task(&t.id).await? else {
            continue;
        };
        let schedule = task_schedule(&t);
        let params = serde_json::from_str::<Value>(&t.params_json).ok();
        if schedule.is_none() || !params.as_ref().is_some_and(Value::is_object) {
            store
                .release_task_lease(&t.id, instance, None, Some("invalidConfiguration"), None)
                .await?;
            continue;
        }
        let next = schedule
            .as_ref()
            .and_then(|s| s.next_after(now))
            .map(|n| n.timestamp());
        if t.only_in_window && !in_maintenance_window(state, ts).await {
            store
                .release_task_lease(&t.id, instance, None, Some("skippedOutsideWindow"), next)
                .await?;
            continue;
        }
        let params = params.expect("validated object above");
        let run = new_run(&t.kind, Some(t.id.clone()), instance, ts, "scheduler");
        if !store.start_job_run(&run).await? {
            // The kind is running already (a "Run now", or another task of the
            // kind): try again on a later pass.
            store
                .release_task_lease(&t.id, instance, None, None, t.next_run_at)
                .await?;
            continue;
        }
        let state = state.clone();
        let instance = instance.to_string();
        let task = t.clone();
        tokio::spawn(async move {
            let status = execute(
                &state,
                job,
                run,
                params,
                Some(&task.id),
                task.only_in_window,
                &instance,
            )
            .await;
            let done = Utc::now();
            let next = schedule
                .as_ref()
                .and_then(|s| s.next_after(done))
                .map(|n| n.timestamp());
            if let Err(e) = state
                .store
                .release_task_lease(
                    &task.id,
                    &instance,
                    Some(done.timestamp()),
                    Some(status.as_str()),
                    next,
                )
                .await
            {
                tracing::warn!(task = %task.id, "releasing the task lease failed: {e}");
            }
        });
    }
    Ok(())
}

fn task_schedule(t: &ScheduledTaskRow) -> Option<Schedule> {
    let v: Value = serde_json::from_str(&t.schedule_json).ok()?;
    Schedule::from_json(&v).ok()
}

fn new_run(
    kind: &str,
    task_id: Option<String>,
    instance: &str,
    now: i64,
    triggered_by: &str,
) -> JobRunRow {
    JobRunRow {
        id: uuid::Uuid::new_v4().to_string(),
        task_id,
        kind: kind.to_string(),
        instance: instance.to_string(),
        started_at: now,
        heartbeat_at: now,
        finished_at: None,
        status: JobStatus::Running,
        summary: None,
        detail_json: None,
        triggered_by: triggered_by.to_string(),
    }
}

/// Whether a global maintenance window is active at `now` (for `only_in_window`).
/// The maintenance gate may install a shared hook. Until then, evaluate the
/// persisted global windows directly; malformed/unavailable configuration denies.
async fn in_maintenance_window(state: &AppState, now: i64) -> bool {
    let hook = WINDOW_HOOK.get().copied();
    match hook {
        Some(f) => f(state.clone(), now).await,
        None => state
            .store
            .scheduler_windows()
            .await
            .is_ok_and(|rows| rows.iter().any(|row| window::active(row, now))),
    }
}

type WindowHook = fn(AppState, i64) -> Pin<Box<dyn Future<Output = bool> + Send>>;

static WINDOW_HOOK: OnceLock<WindowHook> = OnceLock::new();

/// Installed once by the maintenance module.
#[allow(dead_code)]
pub(crate) fn set_window_hook(f: WindowHook) {
    let _ = WINDOW_HOOK.set(f);
}

/// Run `job` for the already-inserted `running` row `run`: keep its heartbeat (and
/// the task's lease) fresh, stop it when the row was abandoned, and record how it
/// ended. Returns the final status.
async fn execute(
    state: &AppState,
    job: JobSpec,
    run: JobRunRow,
    params: Value,
    task_id: Option<&str>,
    only_in_window: bool,
    instance: &str,
) -> JobStatus {
    let lease = job.lease_secs();
    let beat = Duration::from_millis(((lease * 1000) / 3).clamp(200, 30_000) as u64);
    let now = Utc::now().timestamp();
    if !matches!(
        tokio::time::timeout(
            beat,
            state
                .store
                .renew_job_ownership(&run.id, instance, task_id, now, now + lease,)
        )
        .await,
        Ok(Ok(true))
    ) {
        let _ = state
            .store
            .finish_job_run(
                &run.id,
                JobStatus::Cancelled,
                now,
                Some("ownership lost before execution"),
                None,
            )
            .await;
        return JobStatus::Cancelled;
    }
    let ctx = JobCtx {
        kind: run.kind.clone(),
        run_id: run.id.clone(),
        task_id: task_id.map(str::to_string),
        params,
        triggered_by: run.triggered_by.clone(),
        state: Some(state.clone()),
    };
    let mut work = tokio::spawn(async move { (job.run)(ctx).await });
    let outcome = loop {
        tokio::select! {
            r = &mut work => break Some(r),
            _ = tokio::time::sleep(beat) => {
                let now = Utc::now().timestamp();
                let owned = tokio::time::timeout(beat, state.store.renew_job_ownership(
                    &run.id, instance, task_id, now, now + lease,
                )).await;
                let window_ok = !only_in_window || tokio::time::timeout(
                    beat, in_maintenance_window(state, now),
                ).await.unwrap_or(false);
                if !matches!(owned, Ok(Ok(true))) || !window_ok {
                    work.abort();
                    let _ = work.await;
                    break None;
                }
            }
        }
    };
    let (status, summary, detail) = match outcome {
        None => {
            tracing::warn!(kind = %run.kind, run = %run.id, "job stopped: ownership or maintenance window lost");
            let _ = state
                .store
                .finish_job_run(
                    &run.id,
                    JobStatus::Cancelled,
                    Utc::now().timestamp(),
                    Some("ownership or maintenance window lost"),
                    None,
                )
                .await;
            return state
                .store
                .get_job_run(&run.id)
                .await
                .ok()
                .flatten()
                .map_or(JobStatus::Cancelled, |row| row.status);
        }
        Some(Ok(Ok(out))) => (JobStatus::Ok, out.summary, out.detail),
        Some(Ok(Err(reason))) => (JobStatus::Failed, reason, Value::Null),
        Some(Err(join)) => (
            JobStatus::Failed,
            if join.is_panic() {
                "the job panicked".to_string()
            } else {
                "the job was cancelled".to_string()
            },
            Value::Null,
        ),
    };
    let finished = Utc::now().timestamp();
    // Fence even a fast completion: loss between the last heartbeat and the
    // result must never be recorded as success.
    if !matches!(
        tokio::time::timeout(
            beat,
            state
                .store
                .renew_job_ownership(&run.id, instance, task_id, finished, finished + lease,)
        )
        .await,
        Ok(Ok(true))
    ) {
        let _ = state
            .store
            .finish_job_run(
                &run.id,
                JobStatus::Cancelled,
                finished,
                Some("ownership lost before completion"),
                None,
            )
            .await;
        return JobStatus::Cancelled;
    }
    let detail = (!detail.is_null()).then(|| detail.to_string());
    match state
        .store
        .finish_job_run(&run.id, status, finished, Some(&summary), detail.as_deref())
        .await
    {
        Ok(true) => status,
        Ok(false) => JobStatus::Abandoned,
        Err(e) => {
            tracing::warn!(run = %run.id, "recording the job result failed: {e}");
            status
        }
    }
}

/// Why "Run now" did not start a run.
#[derive(Debug, Clone, PartialEq, Eq)]
#[expect(dead_code, reason = "jobs admin API is a subsequent f12 slice")]
pub(crate) enum RunNowError {
    UnknownKind,
    AlreadyRunning,
    Store(String),
}

/// Start a run of `kind` now on this process (`POST /admin/jobs/{kind}/run`).
/// Returns the run id; the run continues in the background.
#[expect(dead_code, reason = "jobs admin API is a subsequent f12 slice")]
pub(crate) async fn run_now(
    state: &AppState,
    kind: &str,
    params: Value,
    triggered_by: &str,
) -> Result<String, RunNowError> {
    let job = spec(kind).ok_or(RunNowError::UnknownKind)?;
    let instance = state.scheduler.instance().to_string();
    let run = new_run(kind, None, &instance, Utc::now().timestamp(), triggered_by);
    match state.store.start_job_run(&run).await {
        Ok(true) => {}
        Ok(false) => return Err(RunNowError::AlreadyRunning),
        Err(e) => return Err(RunNowError::Store(e.to_string())),
    }
    let id = run.id.clone();
    let state = state.clone();
    tokio::spawn(async move {
        execute(&state, job, run, params, None, false, &instance).await;
    });
    Ok(id)
}

/// Record a run of a loop that schedules itself (it is not started by the
/// scheduler): `started_at` to now, with its status and summary.
#[expect(
    dead_code,
    reason = "registered self-scheduling observers land in subsequent lanes"
)]
pub(crate) async fn observe(
    state: &AppState,
    kind: &str,
    started_at: i64,
    status: JobStatus,
    summary: Option<&str>,
) {
    let now = Utc::now().timestamp();
    let mut run = new_run(kind, None, state.scheduler.instance(), started_at, "loop");
    run.heartbeat_at = now;
    run.finished_at = Some(now);
    run.status = status;
    run.summary = summary.map(str::to_string);
    if let Err(e) = state.store.insert_job_run(&run).await {
        tracing::warn!(kind, "recording an observed run failed: {e}");
    }
}

// ─── built-in kinds ─────────────────────────────────────────────────────────────

/// Reclaim free pages (SQLite) / dead tuples (Postgres, plain `VACUUM`, never
/// `FULL`).
async fn vacuum(ctx: JobCtx) -> Result<JobOutput, String> {
    let state = ctx.state()?;
    state
        .store
        .vacuum_database()
        .await
        .map_err(|e| format!("vacuum failed: {e}"))?;
    Ok(JobOutput::new("vacuum finished", json!({})))
}

/// Delete uploaded blobs older than `olderThanHours` (default 24, 1-720).
async fn gc_uploads(ctx: JobCtx) -> Result<JobOutput, String> {
    let hours = ctx
        .params
        .get("olderThanHours")
        .and_then(Value::as_i64)
        .unwrap_or(24)
        .clamp(1, 720);
    let state = ctx.state()?;
    let n = state
        .store
        .sweep_uploads(chrono::Duration::hours(hours))
        .await
        .map_err(|e| format!("upload sweep failed: {e}"))?;
    Ok(JobOutput::new(
        format!("{n} upload(s) removed"),
        json!({ "removed": n, "olderThanHours": hours }),
    ))
}

/// Rebuild the search index of every account (engine mode).
async fn reindex(ctx: JobCtx) -> Result<JobOutput, String> {
    let state = ctx.state()?;
    let engine = state
        .engine
        .as_ref()
        .ok_or_else(|| "reindex requires engine mode".to_string())?;
    let s = engine
        .rebuild_search_index_all()
        .await
        .map_err(|e| format!("reindex failed: {e}"))?;
    if s.failed > 0 {
        return Err(format!("reindex failed for {} message(s)", s.failed));
    }
    Ok(JobOutput::new(
        format!(
            "{} message(s) indexed in {} account(s)",
            s.indexed, s.accounts
        ),
        json!({
            "accounts": s.accounts,
            "messages": s.messages,
            "indexed": s.indexed,
            "removed": s.removed,
            "zeroAccess": s.zero_access,
            "failed": s.failed,
        }),
    ))
}

/// Re-thread the stored mail of every account (engine mode).
async fn rethread_all(ctx: JobCtx) -> Result<JobOutput, String> {
    let state = ctx.state()?;
    let engine = state
        .engine
        .as_ref()
        .ok_or_else(|| "rethread requires engine mode".to_string())?;
    let accounts = state
        .store
        .list_accounts()
        .await
        .map_err(|e| format!("listing accounts failed: {e}"))?;
    let (mut messages, mut threads, mut reassigned, mut failed) = (0u64, 0u64, 0u64, 0u64);
    for a in &accounts {
        match engine.rethread_account(&a.id).await {
            Ok(s) => {
                messages += s.messages as u64;
                threads += s.threads as u64;
                reassigned += s.reassigned as u64;
            }
            Err(e) => {
                failed += 1;
                tracing::warn!(account = %a.id, "rethread failed: {e}");
            }
        }
    }
    let out = json!({
        "accounts": accounts.len(),
        "messages": messages,
        "threads": threads,
        "reassigned": reassigned,
        "failed": failed,
    });
    if failed > 0 {
        return Err(format!("rethread failed for {failed} account(s)"));
    }
    Ok(JobOutput::new(
        format!(
            "{} account(s) re-threaded, {reassigned} message(s) moved",
            accounts.len()
        ),
        out,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn schedules_parse_validate_and_round_trip() {
        for v in [
            json!({"everySecs": 3600}),
            json!({"dailyAt": "03:00", "tz": "Europe/Berlin"}),
            json!({"weekly": {"days": ["mon", "thu"], "at": "23:59", "tz": "UTC"}}),
            json!({"monthly": {"day": 31, "at": "00:00", "tz": "America/New_York"}}),
        ] {
            let s = Schedule::from_json(&v).unwrap();
            assert_eq!(s.to_json(), v);
        }
        let err = |v: Value| Schedule::from_json(&v).unwrap_err();
        assert_eq!(err(json!({"everySecs": 59})).field, "/everySecs");
        assert_eq!(err(json!({"dailyAt": "3:00"})).code, "invalidTime");
        assert_eq!(err(json!({"dailyAt": "24:00"})).code, "invalidTime");
        assert_eq!(
            err(json!({"dailyAt": "03:00", "tz": "Mars/Base"})).field,
            "/tz"
        );
        assert_eq!(
            err(json!({"weekly": {"days": [], "at": "03:00"}})).code,
            "required"
        );
        assert_eq!(
            err(json!({"weekly": {"days": ["xyz"], "at": "03:00"}})).field,
            "/weekly/days"
        );
        assert_eq!(
            err(json!({"monthly": {"day": 0, "at": "03:00"}})).field,
            "/monthly/day"
        );
        assert_eq!(err(json!({"nope": 1})).code, "invalidSchedule");
        // A daily schedule without tz is UTC.
        assert_eq!(
            Schedule::from_json(&json!({"dailyAt": "03:00"})).unwrap(),
            Schedule::Daily {
                daily_at: "03:00".into(),
                tz: "UTC".into()
            }
        );
    }

    #[test]
    fn next_run_is_wall_clock_time_across_dst() {
        let berlin = Schedule::Daily {
            daily_at: "02:30".into(),
            tz: "Europe/Berlin".into(),
        };
        // 2026-03-29: Berlin skips 02:00-03:00; the run moves to 03:00 local (01:00Z).
        assert_eq!(
            berlin.next_after(utc("2026-03-28T12:00:00Z")),
            Some(utc("2026-03-29T01:00:00Z"))
        );
        // The day after, 02:30 CEST = 00:30Z.
        assert_eq!(
            berlin.next_after(utc("2026-03-29T01:00:00Z")),
            Some(utc("2026-03-30T00:30:00Z"))
        );
        // 2026-10-25: 02:30 happens twice; the first (CEST, 00:30Z) is used, once.
        assert_eq!(
            berlin.next_after(utc("2026-10-24T12:00:00Z")),
            Some(utc("2026-10-25T00:30:00Z"))
        );
        assert_eq!(
            berlin.next_after(utc("2026-10-25T00:30:00Z")),
            Some(utc("2026-10-26T01:30:00Z"))
        );
        let every = Schedule::Every { secs: 600 };
        assert_eq!(
            every.next_after(utc("2026-01-01T00:00:00Z")),
            Some(utc("2026-01-01T00:10:00Z"))
        );
    }

    #[test]
    fn weekly_and_monthly_next_runs() {
        let weekly = Schedule::Weekly {
            days: vec![Weekday::Mon, Weekday::Thu],
            at: "09:00".into(),
            tz: "America/New_York".into(),
        };
        // Sunday 2026-10-25 → Monday 09:00 EDT = 13:00Z.
        assert_eq!(
            weekly.next_after(utc("2026-10-25T12:00:00Z")),
            Some(utc("2026-10-26T13:00:00Z"))
        );
        // After Thursday's run → next Monday, after the US change (EST, 14:00Z).
        assert_eq!(
            weekly.next_after(utc("2026-10-29T13:00:00Z")),
            Some(utc("2026-11-02T14:00:00Z"))
        );
        let monthly = Schedule::Monthly {
            day: 31,
            at: "00:00".into(),
            tz: "UTC".into(),
        };
        // February has no 31st: its last day.
        assert_eq!(
            monthly.next_after(utc("2026-02-01T00:00:00Z")),
            Some(utc("2026-02-28T00:00:00Z"))
        );
        assert_eq!(
            monthly.next_after(utc("2026-02-28T00:00:00Z")),
            Some(utc("2026-03-31T00:00:00Z"))
        );
    }

    #[test]
    fn builtin_kinds_are_registered_without_enabling_anything() {
        let names: Vec<String> = kinds().into_iter().map(|(k, _)| k).collect();
        for k in ["vacuum", "gc_uploads", "reindex", "rethread_all"] {
            assert!(names.iter().any(|n| n == k), "{k} missing");
        }
        assert!(valid_kind("db_backup"));
        assert!(!valid_kind("Db"));
        assert!(!valid_kind("9x"));
        assert!(!valid_kind(""));
    }

    #[test]
    fn duplicate_kind_panics_without_erasing_existing_jobs() {
        let before = spec("vacuum").unwrap().default_schedule;
        let failed = std::panic::catch_unwind(|| {
            register(
                "vacuum",
                JobSpec::new(|_| Box::pin(async { Ok(JobOutput::default()) })),
            );
        });
        assert!(failed.is_err());
        assert_eq!(spec("vacuum").unwrap().default_schedule, before);
        for kind in ["gc_uploads", "reindex", "rethread_all"] {
            assert!(spec(kind).is_some());
        }
    }
}
