//! Explicit fixture-only phase timings. No request paths, identities, headers,
//! credentials, SQL or bind values are recorded. No HTTP endpoint is installed.
//! Enable before server startup with MW_FIXTURE_REQUEST_PROFILE=1. Measurements
//! include profiling overhead; reset only after warmup requests have drained.
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use futures_util::future::{BoxFuture, Either};
use serde_json::{Value, json};
use std::{
    future::Future,
    sync::{
        Arc, LazyLock, Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

const BOUNDS_US: [u64; 15] = [
    10, 50, 100, 250, 500, 1000, 2500, 5000, 10000, 25000, 50000, 100000, 250000, 500000, 1000000,
];
const CHILD_LIMIT: usize = 128;
macro_rules! phases {
    ($($variant:ident => $label:literal),+ $(,)?) => {
        #[derive(Clone, Copy)]
        pub(crate) enum Phase { $($variant),+ }
        const LABELS: &[&str] = &[$($label),+];
        #[cfg(feature = "sql-shape-diagnostics")]
        impl Phase {
            fn query_phase(self) -> mw_store::pool_telemetry::shapes::Phase {
                match self { $(Self::$variant=>mw_store::pool_telemetry::shapes::Phase::$variant),+ }
            }
        }
    };
}
phases! {
 Request=>"request", HealthAccess=>"guard.health-access", Protocol=>"guard.protocol", Probe=>"guard.probe", Protection=>"guard.protection",
 SourceBans=>"guard.source-bans", InboundAllowlist=>"guard.inbound-allowlist", SessionWatch=>"guard.session-watch",
 SessionGeo=>"guard.session-geography", ClientAccess=>"guard.client-access", LinkedScope=>"guard.linked-scope",
 Maintenance=>"guard.maintenance", Features=>"guard.features", Auth=>"auth.total",
 SessionRead=>"auth.session-read", NativeRead=>"auth.native-read", SessionClock=>"auth.session-clock",
 SessionTouch=>"auth.session-touch", SessionTouchSkipped=>"auth.session-touch-skipped", LinkedLogin=>"auth.linked-login", DomainUpgrade=>"auth.domain-upgrade",
 AccountGate=>"auth.account-gate", LinkedContext=>"auth.linked-context", FeaturePresence=>"feature.key-presence",
 FeatureResolution=>"feature.resolution", SessionWatchConfig=>"session-watch.config",
 AfterBody=>"jmap.after-body", Handler=>"jmap.handler", Engine=>"jmap.engine"
}
#[derive(Default)]
struct Timing {
    count: AtomicU64,
    micros: AtomicU64,
    max: AtomicU64,
    buckets: [AtomicU64; 16],
}
impl Timing {
    fn add(&self, duration: Duration) {
        let micros = duration.as_micros().min(u64::MAX as u128) as u64;
        self.count.fetch_add(1, Ordering::Relaxed);
        self.micros.fetch_add(micros, Ordering::Relaxed);
        self.max.fetch_max(micros, Ordering::Relaxed);
        let bucket = BOUNDS_US
            .iter()
            .position(|bound| micros <= *bound)
            .unwrap_or(15);
        self.buckets[bucket].fetch_add(1, Ordering::Relaxed);
    }
    fn json(&self) -> Value {
        json!({"count":self.count.load(Ordering::Relaxed),"totalMicros":self.micros.load(Ordering::Relaxed),"maxMicros":self.max.load(Ordering::Relaxed),"buckets":self.buckets.iter().enumerate().map(|(index,count)|json!({"upperMicros":BOUNDS_US.get(index),"count":count.load(Ordering::Relaxed)})).collect::<Vec<_>>()})
    }
    fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
        self.micros.store(0, Ordering::Relaxed);
        self.max.store(0, Ordering::Relaxed);
        for bucket in &self.buckets {
            bucket.store(0, Ordering::Relaxed);
        }
    }
}
#[derive(Default)]
struct Counts {
    inclusive: Timing,
    self_time: Timing,
    self_unavailable: AtomicU64,
}
static QUERIES: LazyLock<Mutex<mw_store::pool_telemetry::QuerySnapshot>> =
    LazyLock::new(|| Mutex::new(Default::default()));
static COUNTERS: LazyLock<Vec<Counts>> =
    LazyLock::new(|| LABELS.iter().map(|_| Counts::default()).collect());
struct Frame {
    children: Mutex<Vec<(Instant, Instant)>>,
    overflow: AtomicU64,
}
impl Frame {
    fn new() -> Self {
        Self {
            children: Mutex::new(Vec::new()),
            overflow: AtomicU64::new(0),
        }
    }
    fn child(&self, start: Instant, end: Instant) {
        let mut children = self.children.lock().unwrap_or_else(|p| p.into_inner());
        if children.len() < CHILD_LIMIT {
            children.push((start, end));
        } else {
            self.overflow.fetch_add(1, Ordering::Relaxed);
        }
    }
    fn self_duration(&self, start: Instant, end: Instant) -> Option<Duration> {
        if self.overflow.load(Ordering::Relaxed) != 0 {
            return None;
        }
        let mut children = self.children.lock().unwrap_or_else(|p| p.into_inner());
        Some(
            end.saturating_duration_since(start)
                .saturating_sub(union_duration(&mut children, start, end)),
        )
    }
}
fn union_duration(intervals: &mut [(Instant, Instant)], start: Instant, end: Instant) -> Duration {
    intervals.sort_unstable_by_key(|&(from, _)| from);
    let mut covered = Duration::ZERO;
    let mut cursor = start;
    for &(from, to) in intervals.iter() {
        let from = from.max(start);
        let to = to.min(end);
        if to > cursor && to > from {
            covered += to.saturating_duration_since(from.max(cursor));
            cursor = to;
        }
    }
    covered
}
tokio::task_local! {static CURRENT:Arc<Frame>;}
/// CPU/OS profiling must be enabled before startup; the environment is read once.
pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED
        .get_or_init(|| std::env::var("MW_FIXTURE_REQUEST_PROFILE").is_ok_and(|value| value == "1"))
}
// Default builds retain the original function directly. Phase task-local work
// exists only in the explicit query-shape feature and an active bounded scope.
#[cfg(not(feature = "sql-shape-diagnostics"))]
pub(crate) use measure_timings as measure;
#[cfg(feature = "sql-shape-diagnostics")]
pub(crate) fn measure<'a, T: Send + 'a>(
    phase: Phase,
    operation: impl Future<Output = T> + Send + 'a,
) -> impl Future<Output = T> + Send + 'a {
    if mw_store::pool_telemetry::shapes::active() {
        Either::Left(mw_store::pool_telemetry::shapes::in_phase(phase.query_phase(),measure_timings(phase,operation)))
    } else { Either::Right(measure_timings(phase,operation)) }
}
pub(crate) fn measure_timings<'a, T: Send + 'a>(
    phase: Phase,
    operation: impl Future<Output = T> + Send + 'a,
) -> impl Future<Output = T> + Send + 'a {
    if !enabled() {
        // Return the child directly instead of storing it again in an enclosing
        // async state machine. Normal requests allocate no profiling boxes.
        Either::Left(operation)
    } else {
        // Only explicit profiling pays this allocation. Keeping the task-local
        // scope's child behind a pointer bounds the alternative future frame.
        let operation: BoxFuture<'a, T> = Box::pin(operation);
        Either::Right(measure_enabled(phase, operation))
    }
}
async fn measure_enabled<T: Send>(phase: Phase, operation: BoxFuture<'_, T>) -> T {
    let parent = CURRENT.try_with(Arc::clone).ok();
    let frame = Arc::new(Frame::new());
    let start = Instant::now();
    let output = CURRENT.scope(frame.clone(), operation).await;
    let end = Instant::now();
    if let Some(parent) = parent {
        parent.child(start, end);
    }
    let counters = &COUNTERS[phase as usize];
    counters.inclusive.add(end.saturating_duration_since(start));
    if let Some(duration) = frame.self_duration(start, end) {
        counters.self_time.add(duration);
    } else {
        counters.self_unavailable.fetch_add(1, Ordering::Relaxed);
    }
    output
}
/// Concurrent snapshots are approximate. Drain fixture requests before taking a
/// final snapshot; self time subtracts the UNION of measured child intervals.
pub fn snapshot() -> Value {
    json!({"enabled":enabled(),"units":"microseconds","selfTime":"inclusive minus union of measured child intervals; uninstrumented work remains in self time","responseBoundary":"response construction, excluding streaming body drain","queries":*QUERIES.lock().unwrap_or_else(|p|p.into_inner()),"queryBoundary":"q() statements/BEGIN and central acquisitions; query elapsed includes acquisition; direct SQLx statements and detached tasks excluded","phases":LABELS.iter().zip(COUNTERS.iter()).map(|(label,count)|json!({"phase":label,"inclusive":count.inclusive.json(),"self":count.self_time.json(),"selfUnavailable":count.self_unavailable.load(Ordering::Relaxed)})).collect::<Vec<_>>()})
}
/// Reset only when no fixture request is in flight. This is not a production
/// telemetry interface and is intentionally not exposed over HTTP.
pub fn reset() {
    *QUERIES.lock().unwrap_or_else(|p| p.into_inner()) = Default::default();
    for count in COUNTERS.iter() {
        count.inclusive.reset();
        count.self_time.reset();
        count.self_unavailable.store(0, Ordering::Relaxed);
    }
}
pub(crate) async fn request(request: Request, next: Next) -> Response {
    if !enabled() {
        return next.run(request).await;
    }
    let (response, queries) =
        mw_store::pool_telemetry::measure_queries(measure(Phase::Request, next.run(request))).await;
    QUERIES
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .merge(&queries);
    response
}
macro_rules! guard {
    ($name:ident,$phase:ident,$handler:path) => {
        pub(crate) async fn $name(
            state: State<crate::AppState>,
            request: Request,
            next: Next,
        ) -> Response {
            measure(Phase::$phase, ($handler)(state, request, next)).await
        }
    };
}
#[cfg(feature = "sql-shape-diagnostics")]
guard!(health_access, HealthAccess, crate::health_access::layer);
#[cfg(not(feature = "sql-shape-diagnostics"))]
pub(crate) use crate::health_access::layer as health_access;

guard!(protocol, Protocol, crate::protocol_guard::guard);
guard!(probe, Probe, crate::admin_probe::guard);
guard!(protection, Protection, crate::request_protection::guard);
guard!(source_bans, SourceBans, crate::source_bans::guard);
guard!(
    inbound_allowlist,
    InboundAllowlist,
    crate::inbound_allowlist::guard
);
guard!(session_watch, SessionWatch, crate::session_watch::guard);
guard!(
    session_geo,
    SessionGeo,
    crate::signin_geography::session_guard
);
guard!(client_access, ClientAccess, crate::client_access::guard);
guard!(linked_scope, LinkedScope, crate::linked_accounts::guard);
guard!(maintenance, Maintenance, crate::maintenance_mode::guard);
guard!(features, Features, crate::feature_gate::guard);

#[cfg(test)]
mod tests {
    use super::*;
    struct LargeFuture([u8; 32 * 1024]);
    impl Future for LargeFuture {
        type Output = u8;
        fn poll(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<u8> {
            std::task::Poll::Ready(self.0[0])
        }
    }
    // The previous implementation's two inline child-owning branches. The
    // comparison measures compiler-generated frame layout, without polling or
    // needing to mutate the process-wide profiling switch.
    async fn previous_inline_measure<T>(phase: Phase, operation: impl Future<Output = T>) -> T {
        if !enabled() {
            return operation.await;
        }
        let parent = CURRENT.try_with(Arc::clone).ok();
        let frame = Arc::new(Frame::new());
        let start = Instant::now();
        let output = CURRENT.scope(frame.clone(), operation).await;
        let end = Instant::now();
        if let Some(parent) = parent {
            parent.child(start, end);
        }
        let counters = &COUNTERS[phase as usize];
        counters.inclusive.add(end.saturating_duration_since(start));
        if let Some(duration) = frame.self_duration(start, end) {
            counters.self_time.add(duration);
        } else {
            counters.self_unavailable.fetch_add(1, Ordering::Relaxed);
        }
        output
    }
    #[test]
    fn profiling_wrapper_does_not_duplicate_large_child_future_frames() {
        fn assert_send<T: Send>(_: &T) {}
        let before = previous_inline_measure(Phase::Auth, LargeFuture([7; 32 * 1024]));
        let after = measure(Phase::Auth, LargeFuture([7; 32 * 1024]));
        assert_send(&after);
        let before_bytes = std::mem::size_of_val(&before);
        let after_bytes = std::mem::size_of_val(&after);
        eprintln!(
            "profiling wrapper frame: before={before_bytes} bytes; after={after_bytes} bytes"
        );
        assert!(
            after_bytes < before_bytes,
            "the wrapper must retain less inline child state"
        );
        assert!(
            after_bytes <= std::mem::size_of::<LargeFuture>() + 512,
            "the unprofiled path must retain only one child plus bounded wrapper metadata"
        );
    }
    #[tokio::test]
    async fn boxed_profile_scope_keeps_parent_coverage_and_child_output() {
        let parent = Arc::new(Frame::new());
        let expected_parent = parent.clone();
        let child = async move {
            assert!(
                !Arc::ptr_eq(&CURRENT.with(Arc::clone), &expected_parent),
                "a profiled child needs its own task-local frame"
            );
            tokio::task::yield_now().await;
            37
        };
        let result = CURRENT
            .scope(
                parent.clone(),
                measure_enabled(Phase::Auth, Box::pin(child)),
            )
            .await;
        assert_eq!(result, 37);
        assert_eq!(
            parent.children.lock().unwrap().len(),
            1,
            "completed child coverage must still be reported to its polling parent"
        );
        assert!(
            CURRENT.try_with(Arc::clone).is_err(),
            "task-local scope must be restored"
        );
    }
    #[test]
    fn overlapping_children_are_subtracted_once_and_clipped() {
        let start = Instant::now();
        let end = start + Duration::from_millis(100);
        let mut children = [
            (
                start + Duration::from_millis(10),
                start + Duration::from_millis(50),
            ),
            (
                start + Duration::from_millis(30),
                start + Duration::from_millis(70),
            ),
            (
                start + Duration::from_millis(90),
                start + Duration::from_millis(110),
            ),
        ];
        assert_eq!(
            union_duration(&mut children, start, end),
            Duration::from_millis(70)
        );
    }
    #[test]
    fn overflowing_child_budget_marks_self_unknown_instead_of_guessing() {
        let frame = Frame::new();
        let start = Instant::now();
        for _ in 0..=CHILD_LIMIT {
            frame.child(start, start + Duration::from_millis(1));
        }
        assert!(
            frame
                .self_duration(start, start + Duration::from_millis(10))
                .is_none()
        );
        assert_eq!(frame.children.lock().unwrap().len(), CHILD_LIMIT);
    }
    #[test]
    fn aggregate_histogram_preserves_counts_and_bounded_buckets() {
        let timing = Timing::default();
        for micros in [0, 11, 1000, 1000001] {
            timing.add(Duration::from_micros(micros));
        }
        let result = timing.json();
        assert_eq!(result["count"], 4);
        assert_eq!(result["maxMicros"], 1000001);
        assert_eq!(result["buckets"].as_array().unwrap().len(), 16);
        timing.reset();
        assert_eq!(timing.json()["count"], 0);
    }
}

/// Fixture-only, off-request-path snapshot parsed by the production policy parser.
/// No policy, exemptions or counters are changed; callers must retain unknown errors.
#[cfg(feature = "load-test-fixtures")]
pub async fn protection_policy_witness(store: &mw_store::Store) -> Result<Value, &'static str> {
    let before = store.settings_read_revision().await.map_err(|_| "policy revision unavailable")?;
    if before.number().is_none() { return Err("policy revision missing"); }
    let rows = crate::settings_registry::effective_all_at_revision(store, &Default::default(),
        |definition| definition.key.starts_with("security.protection."), &before)
        .await.map_err(|_| "policy source unavailable")?;
    let no_exemptions = ["exemptClients", "exemptAccounts", "exemptAdmins", "userAgentExceptions"]
        .iter().all(|suffix| rows.iter().find(|row| row.key == format!("security.protection.{suffix}"))
            .is_some_and(|row| row.value.as_array().is_some_and(Vec::is_empty)));
    // Bind every resolved protection input, including patterns not present in
    // Config::summary, to the registered production default except the four modes.
    let defaults_except_modes = rows.iter().all(|row| {
        let expected = match row.key.as_str() {
            "security.protection.rateMode" | "security.protection.adaptiveMode"
            | "security.protection.probePathMode" | "security.protection.userAgentMode" => json!("enforce"),
            _ => match crate::settings_registry::lookup(&row.key) {
                Some(definition) => definition.default_value(),
                None => return false,
            },
        };
        row.value == expected
    });
    let parsed = crate::request_protection::config::Config::parse(rows)
        .map_err(|_| "policy parse failed")?;
    let after = store.settings_read_revision().await.map_err(|_| "policy revision unavailable")?;
    if before.number() != after.number() || before.scope() != after.scope() {
        return Err("policy source changed");
    }
    Ok(json!({"kind":"native-protection-policy-v1","parsed":parsed.summary(),"noExemptions":no_exemptions,"defaultsExceptModes":defaults_except_modes}))
}
