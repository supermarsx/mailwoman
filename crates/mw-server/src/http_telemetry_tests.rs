use super::*;
use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use tower::ServiceExt;

#[derive(Clone)]
struct Context {
    worker: Arc<Worker>,
    history: Arc<request_history::Recorder>,
}
async fn observe(State(context): State<Context>, request: Request, next: Next) -> Response {
    crate::http_metrics::with_recorders(&context.worker, &context.history, request, next).await
}
fn router(worker: Arc<Worker>, history: Arc<request_history::Recorder>) -> Router {
    Router::new()
        .route("/ok", get(|| async { StatusCode::NO_CONTENT }))
        .fallback(|| async { StatusCode::NOT_FOUND })
        .layer(middleware::from_fn_with_state(
            Context { worker, history },
            observe,
        ))
}
fn paused(capacity: usize) -> (Arc<Worker>, std::sync::mpsc::Sender<()>) {
    let (release, wait) = std::sync::mpsc::channel();
    (
        Worker::new(capacity, move || {
            let _ = wait.recv();
        }),
        release,
    )
}
async fn answer(app: Router, path: &str) -> StatusCode {
    tokio::time::timeout(
        Duration::from_millis(250),
        app.oneshot(Request::builder().uri(path).body(Body::empty()).unwrap()),
    )
    .await
    .expect("HTTP response waited for telemetry worker")
    .unwrap()
    .status()
}

#[tokio::test]
async fn actual_middleware_finishes_while_consumer_is_blocked_and_overflow_is_visible() {
    let (worker, release) = paused(1);
    let history = request_history::Recorder::new();
    let app = router(worker.clone(), history.clone());
    assert_eq!(answer(app.clone(), "/ok").await, StatusCode::NO_CONTENT);
    assert_eq!(answer(app, "/ok").await, StatusCode::NO_CONTENT);
    let before = worker.snapshot();
    assert_eq!(
        (
            before.submitted,
            before.applied,
            before.dropped,
            before.depth,
            before.queue_depth
        ),
        (2, 0, 1, 1, 1)
    );
    assert_eq!(
        history
            .snapshot(chrono::Utc::now().timestamp())
            .iter()
            .map(|bucket| bucket.counts.requests)
            .sum::<u64>(),
        0
    );
    release.send(()).unwrap();
    assert!(worker.barrier(false, Duration::from_secs(2)).await);
    let after = worker.snapshot();
    assert_eq!(
        (after.submitted, after.applied, after.dropped, after.depth),
        (2, 1, 1, 0)
    );
    assert!(worker.barrier(true, Duration::from_secs(2)).await);
}

#[tokio::test]
async fn original_finish_timestamp_and_redacted_404_survive_queued_aggregation() {
    let (worker, release) = paused(4);
    let history = request_history::Recorder::new();
    let earlier = chrono::Utc::now().timestamp() - 70;
    worker.submit_at(&history, RouteClass::Api, 200, 123, earlier, None);
    let app = router(worker.clone(), history.clone());
    assert_eq!(
        answer(
            app,
            "/missing/alice@example.test?token=do-not-retain-this-query"
        )
        .await,
        StatusCode::NOT_FOUND
    );
    release.send(()).unwrap();
    assert!(worker.barrier(false, Duration::from_secs(2)).await);
    let buckets = history.snapshot(chrono::Utc::now().timestamp());
    assert_eq!(
        buckets
            .iter()
            .find(|bucket| bucket.seconds == 60 && bucket.start == earlier / 60 * 60)
            .unwrap()
            .counts
            .requests,
        1
    );
    let paths = buckets
        .iter()
        .filter(|bucket| bucket.seconds == 3600)
        .flat_map(|bucket| {
            bucket
                .counts
                .not_found
                .as_ref()
                .into_iter()
                .flat_map(|missing| &missing.paths)
        })
        .collect::<Vec<_>>();
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].method, "GET");
    assert!(!paths[0].path.contains("alice"));
    assert!(!paths[0].path.contains("example.test"));
    assert!(!paths[0].path.contains("token"));
    assert!(!paths[0].path.contains("retain"));
    assert!(paths[0].path.len() <= mw_store::request_metrics::NOT_FOUND_PATH_MAX_BYTES);
    assert!(worker.barrier(true, Duration::from_secs(2)).await);
}

#[tokio::test]
async fn timed_out_drain_is_bounded_retryable_and_never_blocks_later_http() {
    let (worker, release) = paused(1);
    let history = request_history::Recorder::new();
    let app = router(worker.clone(), history.clone());
    assert_eq!(answer(app.clone(), "/ok").await, StatusCode::NO_CONTENT);
    assert!(!worker.barrier(true, Duration::from_millis(20)).await);
    assert_eq!(answer(app, "/ok").await, StatusCode::NO_CONTENT);
    assert_eq!(worker.snapshot().dropped, 1);
    release.send(()).unwrap();
    assert!(worker.barrier(true, Duration::from_secs(2)).await);
    assert!(
        worker.barrier(true, Duration::from_secs(2)).await,
        "shutdown is idempotent"
    );
    let done = worker.snapshot();
    assert_eq!(
        (done.submitted, done.applied, done.dropped, done.depth),
        (2, 1, 1, 0)
    );
    assert!(!done.running);
}

#[tokio::test]
async fn disconnected_worker_accounts_for_every_sample_without_fallback_io() {
    let (sender, receiver) = mpsc::channel(1);
    drop(receiver);
    let worker = Arc::new(Worker {
        sender,
        stats: Arc::new(Stats::default()),
        closing: AtomicBool::new(false),
    });
    let history = request_history::Recorder::new();
    assert_eq!(
        answer(router(worker.clone(), history), "/ok").await,
        StatusCode::NO_CONTENT
    );
    let snapshot = worker.snapshot();
    assert_eq!(
        (
            snapshot.submitted,
            snapshot.applied,
            snapshot.dropped,
            snapshot.depth
        ),
        (1, 0, 1, 0)
    );
    assert!(!snapshot.running);
}

#[test]
fn repeated_second_and_late_samples_preserve_initialization_boundaries_and_counts() {
    let history = request_history::Recorder::new();
    let now = chrono::Utc::now().timestamp();
    history.record_at(now, 200, 123, None);
    history.record_at(now, 503, 456, None);
    let current = history.snapshot(now);
    let minute = now / 60 * 60;
    let at = current
        .iter()
        .find(|bucket| bucket.seconds == 60 && bucket.start == minute)
        .unwrap();
    assert_eq!(at.counts.requests, 2);
    assert_eq!(at.counts.errors5xx, 1);
    history.record_at(now + 60, 200, 1000, None);
    history.record_at(now - 60, 503, 789, None);
    let buckets = history.snapshot(now + 61);
    assert_eq!(
        buckets
            .iter()
            .find(|bucket| bucket.seconds == 60 && bucket.start == minute - 60)
            .unwrap()
            .counts
            .requests,
        1
    );
    assert_eq!(
        buckets
            .iter()
            .find(|bucket| bucket.seconds == 60 && bucket.start == minute)
            .unwrap()
            .counts
            .requests,
        2
    );
    assert_eq!(
        buckets
            .iter()
            .find(|bucket| bucket.seconds == 60 && bucket.start == minute + 60)
            .unwrap()
            .counts
            .requests,
        1
    );
    let pruned = history.snapshot(now + 3 * 3600);
    assert!(
        pruned
            .iter()
            .all(|bucket| bucket.start + bucket.seconds > now + 3600),
        "advance still applies the original two-hour pruning bound"
    );
}
