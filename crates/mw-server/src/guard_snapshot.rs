//! t34 bounded-staleness guard snapshot (`MW_GUARD_SNAPSHOT=off|bounded`).
//!
//! In `bounded` mode an eligible request may reuse the native publication fence
//! (`Store::native_publication_fence`, the one-statement authority/session source
//! read) that an earlier request of the same published entry read, instead of
//! issuing it again. Reuse is allowed only while the process [`EpochMirror`]
//! proves that no authority change applied since the fence was read:
//! `EpochMirror::still_valid(fence, G)` where `G = stamp_gen()` was taken
//! strictly BEFORE that fence statement started ([`Slot::read`]). Everything else
//! (environment, configuration, revision, clocks, process session map, policy
//! evaluation) still runs per request on the reused DATA exactly as on a fresh
//! fence. Anything the mirror cannot certify takes the existing fresh SQL path.
//!
//! Scope (enforced by the callers, not here): SQLite, and PostgreSQL in bounded
//! mode with a live mirror ([`published_inputs`]); cookie sessions on the
//! ordinary published request-input routes and the declared native metadata read
//! (SQLite only, [`declared_read_inputs`]). Bearer, API-key, selector/linked, admin and
//! WebSocket-frame requests never reach these callers. The final mail-DATA fence
//! (`native_publication_fence_with_mail_data`) and cold publication builds always
//! read fresh.
//!
//! Session clocks (`created_at`, `activity_at`, `last_seen`) have no authority
//! trigger. A reused fence carries clocks that can only be OLDER than the row
//! (writers move them forward). So a reused fence is accepted only when (a) the
//! existing touch-due test is false for its `last_seen`, and (b) where the caller
//! knows the clock policy, the session deadline computed from its clocks does not
//! pass within [`CLOCK_MARGIN`]. Otherwise the caller reads fresh, so expiry and
//! touch decisions are never made from a stale clock.
//!
//! Local revocations call [`expedite`]: every later `still_valid` fails until a
//! mirror poll that started after the call completes, so the next request after
//! a revoking request returns reads fresh SQL. [`expedites`] applies this to every
//! state-changing HTTP request and WebSocket action frame (request_inputs layer);
//! revocations reachable from GET or JMAP call [`expedite`] at the write site.
use crate::AppState;
use mw_store::{
    Store, StoreError,
    authority_publication::{
        NativeAuthorityFence,
        mirror::{self, EpochMirror, MirrorGen},
    },
};
use std::{
    sync::{Arc, Mutex, OnceLock, atomic::{AtomicU64, Ordering}},
    time::Duration,
};

/// A reused fence must keep its session at least this far from its deadline, so
/// a clock decision made later in the same request from the reused (older)
/// clocks cannot expire a live session. Each reuse point re-checks it.
pub(crate) const CLOCK_MARGIN: chrono::TimeDelta = chrono::TimeDelta::seconds(120);
/// Logical retention charge of one cached fence on a published entry.
pub(crate) const SLOT_BYTES: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Off,
    Bounded(Duration),
}

/// Pure parser for the two switches. Unset mode is `off`; unset bound is the
/// default. Any other value is an error (startup must refuse it).
pub(crate) fn parse(mode: Option<&str>, bound: Option<&str>) -> Result<Mode, String> {
    let bound = match bound.map(str::trim) {
        None | Some("") => mirror::DEFAULT_STALENESS,
        Some(raw) => {
            let millis = raw.parse::<u64>().map_err(|_| {
                format!("MW_GUARD_SNAPSHOT_MAX_STALENESS_MS must be an integer from 100 to 5000, got {raw:?}")
            })?;
            let bound = Duration::from_millis(millis);
            if !mirror::valid_staleness(bound) {
                return Err(format!(
                    "MW_GUARD_SNAPSHOT_MAX_STALENESS_MS must be from 100 to 5000, got {millis}"
                ));
            }
            bound
        }
    };
    match mode.map(str::trim) {
        None | Some("") | Some("off") => Ok(Mode::Off),
        Some("bounded") => Ok(Mode::Bounded(bound)),
        Some(other) => Err(format!("MW_GUARD_SNAPSHOT must be off or bounded, got {other:?}")),
    }
}

fn configured() -> &'static Result<Mode, String> {
    static MODE: OnceLock<Result<Mode, String>> = OnceLock::new();
    MODE.get_or_init(|| {
        let mode = std::env::var("MW_GUARD_SNAPSHOT").ok();
        let bound = std::env::var("MW_GUARD_SNAPSHOT_MAX_STALENESS_MS").ok();
        let parsed = parse(mode.as_deref(), bound.as_deref());
        if let Err(error) = &parsed {
            tracing::error!("{error}; guard snapshot stays off");
        }
        parsed
    })
}

/// For the startup path (t34 e8): an invalid value must stop the server. Until
/// that is wired, an invalid value leaves the mode `off` (fresh path) and logs.
#[allow(dead_code)] // wired at startup by t34 e8 (lib.rs/main.rs)
pub(crate) fn validate_startup() -> Result<Mode, String> {
    configured().clone()
}

/// Process-immutable effective mode. Invalid configuration is `off`.
pub(crate) fn mode() -> Mode {
    #[cfg(test)]
    if let Some(mode) = test_control::mode() {
        return mode;
    }
    configured().clone().unwrap_or(Mode::Off)
}

/// The live mirror for this AppState, started on first use in `bounded` mode.
/// The publication worker that AppState owns keeps the `Arc` alive. Live on
/// SQLite and PostgreSQL (`EpochMirror::for_store`); `None` in mode off, without
/// a publication worker, or when the mirror is inert (no runtime, full registry).
/// Live does not mean fresh: every reuse is still decided by `still_valid` /
/// `globals_unchanged`, which fail while the mirror is not fresh.
pub(crate) fn mirror(state: &AppState) -> Option<Arc<EpochMirror>> {
    let Mode::Bounded(bound) = mode() else { return None };
    let mirror = state.native_publication.as_ref()?.guard_mirror(&state.store, bound);
    mirror.is_live().then_some(mirror)
}

/// Whether the request layers may use published request inputs
/// (`request_inputs` and the native metadata admission lookup). The single rule
/// for both, so the backend is tested in one place:
///
/// * SQLite: always, in every mode (the reviewed fresh-fence path; bounded
///   mode adds certified reuse on top).
/// * PostgreSQL: only in bounded mode with a LIVE mirror. Mode off (or an inert
///   mirror) keeps every PostgreSQL request on its original readers, exactly as
///   before t49pg. While the live mirror is not fresh, entries are still used
///   but every fence is read fresh (`Slot::read` reuses nothing then).
///
/// What the published path assumes and why each holds on PostgreSQL
/// (t49pg-e2): every use of an entry compares it with a fence that is either
/// read in one statement (one snapshot on both backends) or certified by the
/// mirror, whose PostgreSQL change source covers every process's commits; the
/// outbox claim, which only one process receives, only evicts memory and never
/// validates; `versions.generation` is `None` on PostgreSQL on both sides of
/// every comparison; the process session map, session touch and clock checks
/// run exactly as on the original path. Nothing here assumes that this process
/// saw every write.
pub(crate) fn published_inputs(state: &AppState) -> bool {
    state.store.uses_sqlite() || mirror(state).is_some()
}

/// Whether the DECLARED native metadata read (`x-mailwoman-read-shape`,
/// `published_guard_context`) may consume published guard inputs. SQLite only,
/// in every mode: once its outer guards have consumed inputs, a due session
/// touch is settled by `Store::settle_native_last_seen`, which is implemented
/// for SQLite only (it returns an error on PostgreSQL, which would refuse the
/// read with 503 until some other request touched the session). On PostgreSQL
/// the declared read keeps its original outer guards; its inner native
/// admission still follows [`published_inputs`] without consuming anything.
pub(crate) fn declared_read_inputs(state: &AppState) -> bool {
    state.store.uses_sqlite()
}

/// Local revocation committed: no cached fence validates until a poll that
/// starts after this call completes. No-op when no mirror was ever started
/// (nothing can be cached then).
pub(crate) fn expedite(state: &AppState) {
    if let Some(mirror) = state.native_publication.as_ref().and_then(|worker| worker.existing_guard_mirror()) {
        mirror.expedite();
    }
}

/// State-changing requests whose completion must expedite the mirror. Every
/// non-safe method except the JMAP API, whose methods commit no authority
/// change (auth-path writes reachable from it expedite at their own site), and
/// the mail refresh, which writes only sync state. WebSocket action frames are
/// replayed through the same router, so frames are covered too.
pub(crate) fn expedites(method: &axum::http::Method, path: &str) -> bool {
    use axum::http::Method;
    !(matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
        || (*method == Method::POST && matches!(path, "/jmap/api" | "/api/mail/refresh")))
}

/// Whether the request carries a credential that could authorize a
/// state-changing route: a non-empty `mw_session` or admin session cookie, an
/// `Authorization` header or an API key. Header inspection only.
pub(crate) fn presents_credential(headers: &axum::http::HeaderMap) -> bool {
    use axum::http::header;
    if headers.contains_key(header::AUTHORIZATION) || headers.contains_key("x-api-key") {
        return true;
    }
    let admin = mw_admin::config::session_cookie_name();
    headers.get_all(header::COOKIE).iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .any(|(name, value)| !value.is_empty() && (name == "mw_session" || name == admin))
}

/// A 4xx response means the request was refused (pre-authentication guard,
/// authentication, validation, not found, rate limit): it committed nothing
/// through the layer rule. Revocations that commit and then answer 4xx (the
/// session-watch theft revoke, disabled/expired-session cleanup) expedite
/// explicitly or are clock-covered. 2xx/3xx/5xx keep expediting.
pub(crate) fn may_have_committed(status: axum::http::StatusCode) -> bool {
    !status.is_client_error()
}

// ---- process-global activity witness (counts only; no ids or fence data) ----
static WITNESS_REUSED: AtomicU64 = AtomicU64::new(0);
static WITNESS_FRESH: AtomicU64 = AtomicU64::new(0);
static WITNESS_GLOBALS_REUSED: AtomicU64 = AtomicU64::new(0);
/// Started mirrors and whether each polls a SQLite Store (else PostgreSQL).
static WITNESS_MIRRORS: Mutex<Vec<(std::sync::Weak<EpochMirror>, bool)>> = Mutex::new(Vec::new());

/// Called once per started mirror (the publication worker's `guard_mirror`).
pub(crate) fn register_for_witness(mirror: &Arc<EpochMirror>, store: &Store) {
    if let Ok(mut mirrors) = WITNESS_MIRRORS.lock() {
        mirrors.retain(|(weak, _)| weak.strong_count() > 0);
        if mirrors.len() < 64 {
            mirrors.push((Arc::downgrade(mirror), store.uses_sqlite()));
        }
    }
}

/// Load-fixture evidence that the bounded snapshot really ran: the effective
/// mode, how many live mirrors poll each backend (`liveByBackend`), the summed
/// `EpochMirror` counters of every started mirror in this process,
/// and how many authority fences / global ban lists were reused or read fresh in
/// bounded mode. Counts and booleans only. Off mode never starts a mirror and
/// never counts, so every number stays 0.
#[cfg_attr(not(any(test, feature = "load-test-fixtures")), allow(dead_code))]
pub(crate) fn witness() -> serde_json::Value {
    let (mode, bound) = match mode() {
        Mode::Off => ("off", None),
        Mode::Bounded(bound) => ("bounded", Some(bound.as_millis() as u64)),
    };
    let started = WITNESS_MIRRORS.lock().map(|mirrors| mirrors.iter()
        .filter_map(|(weak, sqlite)| weak.upgrade().map(|mirror| (mirror, *sqlite))).collect::<Vec<_>>())
        .unwrap_or_default();
    let live = |sqlite: bool| started.iter().filter(|(mirror, is)| *is == sqlite && mirror.is_live()).count();
    let backends = serde_json::json!({"sqlite": live(true), "postgres": live(false)});
    let mirrors = started.into_iter().map(|(mirror, _)| mirror).collect::<Vec<_>>();
    let mut sum = mirror::MirrorCounters::default();
    for counters in mirrors.iter().map(|mirror| mirror.counters()) {
        sum.polls += counters.polls;
        sum.failures += counters.failures;
        sum.rows_applied += counters.rows_applied;
        sum.overflow_flushes += counters.overflow_flushes;
        sum.horizon_flushes += counters.horizon_flushes;
        sum.schema_flushes += counters.schema_flushes;
        sum.generation_flushes += counters.generation_flushes;
        sum.capacity_flushes += counters.capacity_flushes;
        sum.expedites += counters.expedites;
        sum.hits += counters.hits;
        sum.misses += counters.misses;
        sum.gap_flushes += counters.gap_flushes;
        sum.connects += counters.connects;
        sum.pruned_rows += counters.pruned_rows;
        sum.prune_failures += counters.prune_failures;
    }
    serde_json::json!({
        "supported": true, "mode": mode, "maxStalenessMs": bound,
        "mirrors": mirrors.len(), "liveByBackend": backends,
        "live": mirrors.iter().any(|mirror| mirror.is_live()),
        "fresh": mirrors.iter().any(|mirror| mirror.is_fresh()),
        "polls": sum.polls, "failures": sum.failures, "rowsApplied": sum.rows_applied,
        "hits": sum.hits, "misses": sum.misses, "expedites": sum.expedites,
        "flushes": {"overflow": sum.overflow_flushes, "horizon": sum.horizon_flushes,
            "schema": sum.schema_flushes, "generation": sum.generation_flushes,
            "capacity": sum.capacity_flushes, "gap": sum.gap_flushes},
        "pollConnects": sum.connects, "prunedRows": sum.pruned_rows, "pruneFailures": sum.prune_failures,
        "fenceReuses": WITNESS_REUSED.load(Ordering::Relaxed),
        "fenceFreshReads": WITNESS_FRESH.load(Ordering::Relaxed),
        "globalReuses": WITNESS_GLOBALS_REUSED.load(Ordering::Relaxed),
    })
}

/// Cached `(fence, G)` on one published entry. Pairs are only ever written by
/// [`Slot::read`], with `G` stamped before that fence's statement began.
#[derive(Default)]
pub(crate) struct Slot(Mutex<Option<(Arc<NativeAuthorityFence>, MirrorGen)>>);

/// One authority-only publication fence: reused when certified, else fresh.
pub(crate) struct Observed {
    pub(crate) fence: Arc<NativeAuthorityFence>,
    /// True when no SQL ran: an earlier certified fence was reused.
    pub(crate) reused: bool,
    /// The generation that certifies `fence` (stamped before it was read), or
    /// None when the mirror was not fresh then: such a fence is never reused.
    pub(crate) built: Option<MirrorGen>,
}

impl Slot {
    fn cached(&self) -> Option<(Arc<NativeAuthorityFence>, MirrorGen)> {
        self.0.lock().ok()?.clone()
    }
    pub(crate) fn clear(&self) {
        if let Ok(mut slot) = self.0.lock() {
            *slot = None;
        }
    }
    /// `reusable` runs only on a cached candidate (clock/touch conditions); a
    /// candidate it rejects is replaced by a fresh read. `allow_reuse=false`
    /// forces the fresh read (it still refreshes the slot).
    pub(crate) async fn read(
        &self,
        state: &AppState,
        cookie: &str,
        domains: &[String],
        subjects: &[String],
        allow_reuse: bool,
        reusable: impl Fn(&NativeAuthorityFence) -> bool,
    ) -> Result<Option<Observed>, StoreError> {
        self.read_preferring(state, cookie, domains, subjects, allow_reuse, (None::<Arc<NativeAuthorityFence>>, None), reusable).await
    }
    /// As [`Slot::read`], but first tries the caller's own last observation
    /// (`own`, with the generation it was certified at), then the slot.
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn read_preferring(
        &self,
        state: &AppState,
        cookie: &str,
        domains: &[String],
        subjects: &[String],
        allow_reuse: bool,
        own: (impl Into<Option<Arc<NativeAuthorityFence>>>, Option<MirrorGen>),
        reusable: impl Fn(&NativeAuthorityFence) -> bool,
    ) -> Result<Option<Observed>, StoreError> {
        let Some(mirror) = mirror(state) else {
            return fresh(&state.store, cookie, domains, subjects).await.map(|fence| {
                fence.map(|fence| Observed { fence: Arc::new(fence), reused: false, built: None })
            });
        };
        let own = match own {
            (fence, Some(built)) => fence.into().map(|fence| (fence, built)),
            _ => None,
        };
        if allow_reuse {
            for (fence, built) in own.into_iter().chain(self.cached()) {
                if fence.session_id() == cookie && mirror.still_valid(&fence, built) && reusable(&fence) {
                    WITNESS_REUSED.fetch_add(1, Ordering::Relaxed);
                    #[cfg(test)]
                    test_control::record(|counts| counts.reused += 1);
                    return Ok(Some(Observed { fence, reused: true, built: Some(built) }));
                }
            }
        }
        // Strictly BEFORE the statement: see the EpochMirror correctness note.
        let built = mirror.stamp_gen();
        WITNESS_FRESH.fetch_add(1, Ordering::Relaxed);
        let Some(fence) = fresh(&state.store, cookie, domains, subjects).await? else {
            self.clear();
            return Ok(None);
        };
        let fence = Arc::new(fence);
        if let (Some(built), Ok(mut slot)) = (built, self.0.lock()) {
            *slot = Some((fence.clone(), built));
        }
        Ok(Some(Observed { fence, reused: false, built }))
    }
}

async fn fresh(
    store: &Store,
    cookie: &str,
    domains: &[String],
    subjects: &[String],
) -> Result<Option<NativeAuthorityFence>, StoreError> {
    #[cfg(test)]
    test_control::record(|counts| counts.fresh += 1);
    store.native_publication_fence(cookie, domains, subjects).await
}

/// Process DATA keyed only by the global authority epochs and the settings
/// revision (step 2: parsed source bans). Reused while
/// `EpochMirror::globals_unchanged(G)` holds, `G` stamped before the read.
pub(crate) struct GlobalSlot<T>(Mutex<Option<(Arc<T>, MirrorGen)>>);
impl<T> Default for GlobalSlot<T> {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}
impl<T> GlobalSlot<T> {
    /// `load` must be ONE read statement of data that only global-epoch
    /// writers change. Without a live mirror it always runs and caches nothing.
    pub(crate) async fn get_or_load<E, F>(&self, state: &AppState, load: F) -> Result<Arc<T>, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let Some(mirror) = mirror(state) else {
            return load.await.map(Arc::new);
        };
        if let Some((value, built)) = self.0.lock().ok().and_then(|slot| slot.clone())
            && mirror.globals_unchanged(built)
        {
            WITNESS_GLOBALS_REUSED.fetch_add(1, Ordering::Relaxed);
            #[cfg(test)]
            test_control::record(|counts| counts.globals_reused += 1);
            return Ok(value);
        }
        let built = mirror.stamp_gen();
        let value = Arc::new(load.await?);
        if let (Some(built), Ok(mut slot)) = (built, self.0.lock()) {
            *slot = Some((value.clone(), built));
        }
        Ok(value)
    }
}

/// The touch-due half of the clock condition, for any reused fence.
pub(crate) fn touch_not_due(store: &Store, fence: &NativeAuthorityFence) -> bool {
    !store.session_touch_may_be_due_at(
        fence.last_seen(),
        crate::session_persistence::flush_seconds(),
        chrono::Utc::now(),
    )
}

/// The deadline half: the session deadline from this fence's clocks does not
/// pass within [`CLOCK_MARGIN`] under the given effective clock policy.
pub(crate) fn clocks_clear(fence: &NativeAuthorityFence, idle_secs: u64, absolute_secs: u64) -> bool {
    let (created, activity) = fence.clocks();
    chrono::Utc::now()
        .checked_add_signed(CLOCK_MARGIN)
        .is_some_and(|later| !crate::security_center::session_deadline_passed(
            created, activity, later, idle_secs, absolute_secs))
}

#[cfg(test)]
pub(crate) mod test_control {
    use super::Mode;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
    pub(crate) struct Counts {
        /// Authority fences reused / read fresh by `Slot`.
        pub reused: u64,
        pub fresh: u64,
        /// Global-only DATA (source bans) reused by `GlobalSlot`.
        pub globals_reused: u64,
    }
    tokio::task_local! {
        static MODE: Mode;
        static COUNTS: std::cell::RefCell<Counts>;
    }
    pub(super) fn mode() -> Option<Mode> {
        MODE.try_with(|mode| *mode).ok()
    }
    pub(super) fn record(update: impl FnOnce(&mut Counts)) {
        let _ = COUNTS.try_with(|counts| update(&mut counts.borrow_mut()));
    }
    /// Run `future` with this mode instead of the process configuration.
    pub(crate) async fn with_mode<F: std::future::Future>(mode: Mode, future: F) -> F::Output {
        MODE.scope(mode, future).await
    }
    pub(crate) async fn measure<F: std::future::Future>(future: F) -> (F::Output, Counts) {
        COUNTS.scope(std::cell::RefCell::new(Counts::default()), async {
            let output = future.await;
            (output, COUNTS.with(|counts| *counts.borrow()))
        }).await
    }
}

#[cfg(test)]
#[path = "guard_snapshot_pg_tests.rs"]
pub(crate) mod pg_tests;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn switch_values_parse_or_fail_clearly() {
        assert_eq!(parse(None, None), Ok(Mode::Off));
        assert_eq!(parse(Some("off"), Some("100")), Ok(Mode::Off));
        assert_eq!(parse(Some("bounded"), None), Ok(Mode::Bounded(Duration::from_millis(1000))));
        assert_eq!(parse(Some("bounded"), Some("250")), Ok(Mode::Bounded(Duration::from_millis(250))));
        assert!(parse(Some("on"), None).unwrap_err().contains("MW_GUARD_SNAPSHOT"));
        assert!(parse(Some("bounded"), Some("99")).unwrap_err().contains("100 to 5000"));
        assert!(parse(Some("bounded"), Some("5001")).is_err());
        assert!(parse(Some("off"), Some("1s")).is_err(), "an invalid bound fails even when off");
    }
    /// PostgreSQL (t49pg): `for_store` is live; a fresh mirror stamps a
    /// generation. Outside a runtime or with an invalid bound it stays inert.
    /// The request-level legs are in `guard_snapshot_pg_tests.rs`. Runs only
    /// with `MW_TEST_PG`.
    #[tokio::test]
    async fn postgres_store_gets_a_live_mirror() {
        let Some(dsn) = pg_tests::pg_dsn() else { return };
        let store = Store::open(&pg_tests::pg_schema(&dsn).await, mw_store::ServerKey::generate()).await.unwrap();
        assert!(!store.uses_sqlite());
        let inert = EpochMirror::for_store(&store, Duration::from_millis(99));
        assert!(!inert.is_live() && inert.stamp_gen().is_none(), "invalid bound: inert");
        let mirror = EpochMirror::for_store(&store, Duration::from_millis(100));
        assert!(mirror.is_live());
        // The first poll flushes (nothing cached before it validates); the
        // next one stamps.
        tokio::time::timeout(Duration::from_secs(10), async {
            while mirror.stamp_gen().is_none() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("a live PostgreSQL mirror polls and stamps");
        assert!(mirror.is_fresh() && mirror.counters().connects >= 1);
        drop(mirror);
        store.close().await;
    }
    // ---- t34 frozen process environment (request_inputs::freeze_environment) ----
    use super::super::{Environment, test_control as inputs};

    fn registry_env_name() -> &'static str {
        crate::settings_registry::REGISTRY.iter().find_map(|d| d.env.map(|v| v.0))
            .expect("registry declares environment-backed settings")
    }

    #[tokio::test]
    async fn frozen_snapshot_equals_the_full_scan() {
        let frozen = inputs::capture_process_environment().expect("capturable test environment");
        let scanned = Environment::capture().unwrap();
        assert!(*frozen.environment == scanned);
        assert_eq!(frozen.fingerprint, scanned.fingerprint());
        assert!(scanned.matches());
        // Unfrozen: `unchanged` IS the full scan.
        assert_eq!(scanned.unchanged(), scanned.matches());
        // Frozen: the same answer without a scan.
        let frozen_answer = inputs::with_frozen_environment(Some(frozen.clone()), async {
            (scanned.unchanged(), super::super::process_environment().map(|p| Arc::ptr_eq(&p, &frozen)))
        }).await;
        assert_eq!(frozen_answer, (true, Some(true)), "requests reuse the one snapshot");
        assert!(!inputs::with_frozen_environment(None, async { scanned.unchanged() }).await,
            "an uncapturable frozen environment never certifies published inputs");
    }

    #[tokio::test]
    async fn unfrozen_boundary_still_sees_a_runtime_environment_change() {
        let name = registry_env_name();
        let before = Environment::capture().unwrap();
        let previous = std::env::var_os(name);
        // SAFETY: test-only, single-threaded test binary run (--test-threads=1);
        // restored below before any other assertion can observe it.
        unsafe { std::env::set_var(name, "t34-e6-runtime-change") };
        let changed = (before.matches(), before.unchanged(), Environment::capture().map(|e| e == before));
        match previous {
            Some(value) => unsafe { std::env::set_var(name, value) },
            None => unsafe { std::env::remove_var(name) },
        }
        assert_eq!(changed, (false, false, Some(false)),
            "without freeze_environment every boundary still rescans and refuses");
        assert!(before.unchanged());
    }

    /// Cost of the 13 environment checks a published request makes (first
    /// fence + 12 boundaries). Timing only; run explicitly:
    /// `cargo test -p mw-server-core --lib env_check_cost -- --ignored --nocapture`
    #[tokio::test]
    #[ignore = "timing report, not an assertion"]
    async fn env_check_cost_per_published_request() {
        let entry = Environment::capture().unwrap();
        let frozen = inputs::capture_process_environment().unwrap();
        const REQUESTS: u32 = 2000;
        let started = std::time::Instant::now();
        for _ in 0..REQUESTS {
            let process = Environment::capture().unwrap();
            let _ = std::hint::black_box(process.fingerprint());
            for _ in 0..13 { assert!(std::hint::black_box(&entry).matches()); }
        }
        let scan = started.elapsed() / REQUESTS;
        let frozen_cost = inputs::with_frozen_environment(Some(frozen), async {
            let started = std::time::Instant::now();
            for _ in 0..REQUESTS {
                let process = super::super::process_environment().unwrap();
                let _ = std::hint::black_box(&process.fingerprint);
                for _ in 0..13 { assert!(std::hint::black_box(&entry).unchanged()); }
            }
            started.elapsed() / REQUESTS
        }).await;
        eprintln!("{}", serde_json::json!({"kind":"t34-env-check-cost-per-request",
            "processEnvVars": std::env::vars_os().count(),
            "scanMicros": scan.as_secs_f64() * 1e6, "frozenMicros": frozen_cost.as_secs_f64() * 1e6,
            "debugBuild": cfg!(debug_assertions)}));
    }
    #[test]
    fn only_credentialed_committing_requests_expedite() {
        use axum::http::{HeaderMap, HeaderValue, StatusCode};
        let with = |name: &'static str, value: &str| {
            let mut headers = HeaderMap::new();
            headers.insert(name, HeaderValue::from_str(value).unwrap());
            headers
        };
        assert!(!presents_credential(&HeaderMap::new()));
        assert!(!presents_credential(&with("cookie", "mw_csrf=x; mw_session=")));
        assert!(!presents_credential(&with("cookie", "other=mw_session=x")));
        assert!(presents_credential(&with("cookie", "mw_csrf=x; mw_session=abc")));
        let admin = format!("{}=abc", mw_admin::config::session_cookie_name());
        assert!(presents_credential(&with("cookie", &admin)));
        assert!(presents_credential(&with("authorization", "Bearer t")));
        assert!(presents_credential(&with("x-api-key", "k")));
        assert!(may_have_committed(StatusCode::NO_CONTENT));
        assert!(may_have_committed(StatusCode::SEE_OTHER));
        assert!(may_have_committed(StatusCode::INTERNAL_SERVER_ERROR));
        for status in [StatusCode::UNAUTHORIZED, StatusCode::FORBIDDEN, StatusCode::NOT_FOUND, StatusCode::TOO_MANY_REQUESTS] {
            assert!(!may_have_committed(status));
        }
    }
    /// The production entry point freezes the environment before it builds
    /// the app, so no request can run unfrozen in `mailwoman serve`.
    #[test]
    fn serve_freezes_the_environment_before_building_the_app() {
        let main = include_str!("main.rs");
        let serve = &main[main.find("async fn serve(").expect("serve entry point")..];
        let freeze = serve.find("mw_server::freeze_environment();").expect("serve freezes the environment");
        let build = serve.find("build_app(config)").expect("serve builds the app");
        assert!(freeze < build);
    }
    #[test]
    fn only_state_changing_non_jmap_requests_expedite() {
        use axum::http::Method;
        assert!(!expedites(&Method::GET, "/api/me"));
        assert!(!expedites(&Method::POST, "/jmap/api"));
        assert!(!expedites(&Method::POST, "/api/mail/refresh"));
        assert!(expedites(&Method::POST, "/api/logout"));
        assert!(expedites(&Method::POST, "/api/account/sessions/revoke"));
        assert!(expedites(&Method::PUT, "/admin/users/a/flags"));
        assert!(expedites(&Method::DELETE, "/api/accounts/x"));
        assert!(expedites(&Method::PUT, "/jmap/api"));
    }
}
