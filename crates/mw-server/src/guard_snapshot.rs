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
//! Scope (enforced by the callers, not here): SQLite only (the mirror is inert on
//! PostgreSQL), cookie sessions on the ordinary published request-input routes and
//! the declared native metadata read. Bearer, API-key, selector/linked, admin and
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
    sync::{Arc, Mutex, OnceLock},
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
/// The publication worker that AppState owns keeps the `Arc` alive.
pub(crate) fn mirror(state: &AppState) -> Option<Arc<EpochMirror>> {
    let Mode::Bounded(bound) = mode() else { return None };
    if !state.store.uses_sqlite() {
        return None;
    }
    let mirror = state.native_publication.as_ref()?.guard_mirror(&state.store, bound);
    mirror.is_live().then_some(mirror)
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
                    #[cfg(test)]
                    test_control::record(|counts| counts.reused += 1);
                    return Ok(Some(Observed { fence, reused: true, built: Some(built) }));
                }
            }
        }
        // Strictly BEFORE the statement: see the EpochMirror correctness note.
        let built = mirror.stamp_gen();
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
    /// PostgreSQL: the mirror is inert (no commit-ordered change log), so no
    /// fence can ever be certified and every caller reads fresh. Runs only
    /// with `MW_TEST_PG` (the request layers are SQLite-gated as well).
    #[tokio::test]
    async fn postgres_store_never_gets_a_live_mirror() {
        let Some(url) = std::env::var("MW_TEST_PG").ok().filter(|url| !url.is_empty()) else {
            eprintln!("[guard_snapshot] MW_TEST_PG unset: Postgres leg not run");
            return;
        };
        let store = Store::open(&url, mw_store::ServerKey::generate()).await.unwrap();
        assert!(!store.uses_sqlite());
        let mirror = EpochMirror::for_store(&store, Duration::from_millis(100));
        assert!(!mirror.is_live());
        assert!(mirror.poll_now().await.is_ok_and(|flushed| !flushed));
        assert!(!mirror.is_fresh());
        assert!(mirror.stamp_gen().is_none());
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
