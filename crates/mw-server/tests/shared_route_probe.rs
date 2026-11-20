//! Isolated structural probe of the actual installed Axum/Tower stack.
//! This module is intentionally not wired into the production router.
#[path = "../src/shared_route.rs"]
mod shared_route;
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{Request, State},
    middleware::{Next, from_fn_with_state},
    response::Response,
    routing::{get, post},
};
use shared_route::SharedRouteLayer;
use std::{
    convert::Infallible,
    future::{Future, Ready, ready},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context, Poll, Waker},
};
use tower::{Layer, Service, ServiceExt, service_fn};

struct CloneCount(Arc<AtomicUsize>);
impl Clone for CloneCount {
    fn clone(&self) -> Self {
        self.0.fetch_add(1, Ordering::SeqCst);
        Self(self.0.clone())
    }
}
async fn pass(State(_): State<CloneCount>, request: Request, next: Next) -> Response {
    next.run(request).await
}
async fn counted(layers: usize, shared: bool) -> usize {
    let count = Arc::new(AtomicUsize::new(0));
    let mut router = Router::new().route("/", get(|| async { "ok" }));
    for _ in 0..layers {
        let layer = from_fn_with_state(CloneCount(count.clone()), pass);
        router = if shared {
            router.layer(shared_route::wrap(layer))
        } else {
            router.layer(layer)
        };
    }
    count.store(0, Ordering::SeqCst);
    let response = router.oneshot(Request::new(Body::empty())).await.unwrap();
    assert_eq!(to_bytes(response.into_body(), 64).await.unwrap(), "ok");
    count.load(Ordering::SeqCst)
}
#[tokio::test]
async fn actual_axum_layers_replace_quadratic_recursive_clones_with_linear_state_clones() {
    let mut observations = Vec::new();
    for layers in [10, 20, 40] {
        let baseline = counted(layers, false).await;
        let shared = counted(layers, true).await;
        eprintln!(
            "actual Axum layer probe: layers={layers}, baseline_state_clones={baseline}, shared_state_clones={shared}"
        );
        assert!(
            shared <= layers * 8 + 8,
            "shared clone cost must remain bounded and linear"
        );
        assert!(
            baseline > shared * 2,
            "actual Axum should demonstrate a material clone reduction"
        );
        observations.push((baseline, shared));
    }
    assert!(observations[2].0 > observations[1].0 * 3);
    assert!(observations[2].1 <= observations[1].1 * 2 + 2);
}

#[derive(Clone)]
struct Trace {
    name: &'static str,
    deny: bool,
    log: Arc<Mutex<Vec<String>>>,
}
async fn trace(State(state): State<Trace>, request: Request, next: Next) -> Response {
    state
        .log
        .lock()
        .unwrap()
        .push(format!("request:{}", state.name));
    if state.deny {
        state
            .log
            .lock()
            .unwrap()
            .push(format!("deny:{}", state.name));
        return Response::builder()
            .status(403)
            .body(Body::from("denied"))
            .unwrap();
    }
    let mut response = next.run(request).await;
    state
        .log
        .lock()
        .unwrap()
        .push(format!("response:{}", state.name));
    response
        .headers_mut()
        .append("x-trace", state.name.parse().unwrap());
    response
}
async fn trace_request(shared: bool, deny: bool) -> (u16, Vec<String>, Vec<String>, Vec<u8>) {
    let log = Arc::new(Mutex::new(Vec::new()));
    let handler_log = log.clone();
    let mut router = Router::new().route(
        "/",
        post(move |request: Request| {
            let log = handler_log.clone();
            async move {
                assert_eq!(request.uri().query(), Some("kept=yes"));
                assert_eq!(request.headers()["x-request"], "preserved");
                assert_eq!(
                    to_bytes(request.into_body(), 64).await.unwrap(),
                    "preserved request body"
                );
                log.lock().unwrap().push("handler".into());
                "ok"
            }
        }),
    );
    for (name, denied) in [("inner", deny), ("outer", false)] {
        let layer = from_fn_with_state(
            Trace {
                name,
                deny: denied,
                log: log.clone(),
            },
            trace,
        );
        router = if shared {
            router.layer(shared_route::wrap(layer))
        } else {
            router.layer(layer)
        };
    }
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/?kept=yes")
                .header("x-request", "preserved")
                .body(Body::from("preserved request body"))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status().as_u16();
    let headers = response
        .headers()
        .get_all("x-trace")
        .iter()
        .map(|value| value.to_str().unwrap().to_owned())
        .collect();
    let bytes = to_bytes(response.into_body(), 64).await.unwrap().to_vec();
    let trace = log.lock().unwrap().clone();
    (status, trace, headers, bytes)
}
#[tokio::test]
async fn actual_axum_request_response_order_headers_and_early_denial_are_preserved() {
    for deny in [false, true] {
        let baseline = trace_request(false, deny).await;
        let shared = trace_request(true, deny).await;
        assert_eq!(baseline, shared);
        if deny {
            assert_eq!(shared.0, 403);
            assert_eq!(
                shared.1,
                vec![
                    "request:outer",
                    "request:inner",
                    "deny:inner",
                    "response:outer"
                ]
            );
        } else {
            assert_eq!(
                shared.1,
                vec![
                    "request:outer",
                    "request:inner",
                    "handler",
                    "response:inner",
                    "response:outer"
                ]
            );
            assert_eq!(shared.2, vec!["inner", "outer"]);
        }
    }
}

#[derive(Default)]
struct Readiness {
    open: AtomicBool,
    error: AtomicBool,
    polls: AtomicUsize,
    calls: AtomicUsize,
    drops: AtomicUsize,
}
struct ReadinessService {
    state: Arc<Readiness>,
    ready: bool,
}
impl Clone for ReadinessService {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            ready: false,
        }
    }
}
impl Drop for ReadinessService {
    fn drop(&mut self) {
        self.state.drops.fetch_add(1, Ordering::SeqCst);
    }
}
impl Service<()> for ReadinessService {
    type Response = &'static str;
    type Error = &'static str;
    type Future = Ready<Result<Self::Response, Self::Error>>;
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.state.polls.fetch_add(1, Ordering::SeqCst);
        if self.state.error.load(Ordering::SeqCst) {
            return Poll::Ready(Err("readiness failure"));
        }
        if !self.state.open.load(Ordering::SeqCst) {
            return Poll::Pending;
        }
        self.ready = true;
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, _: ()) -> Self::Future {
        assert!(self.ready, "call must use the same ready clone");
        self.ready = false;
        self.state.calls.fetch_add(1, Ordering::SeqCst);
        ready(Ok("ready"))
    }
}
#[tokio::test]
async fn oneshot_honors_underlying_readiness_and_propagates_ready_and_call_errors() {
    let state = Arc::new(Readiness::default());
    let mut service = SharedRouteLayer.layer(ReadinessService {
        state: state.clone(),
        ready: false,
    });
    let mut context = Context::from_waker(Waker::noop());
    assert!(matches!(
        service.poll_ready(&mut context),
        Poll::Ready(Ok(()))
    ));
    assert_eq!(state.polls.load(Ordering::SeqCst), 0);
    let mut future = Box::pin(service.call(()));
    assert!(future.as_mut().poll(&mut context).is_pending());
    assert_eq!(state.calls.load(Ordering::SeqCst), 0);
    state.open.store(true, Ordering::SeqCst);
    assert_eq!(future.await, Ok("ready"));
    assert_eq!(state.calls.load(Ordering::SeqCst), 1);
    state.error.store(true, Ordering::SeqCst);
    assert_eq!(service.call(()).await, Err("readiness failure"));
    assert_eq!(state.calls.load(Ordering::SeqCst), 1);
    let failed = SharedRouteLayer.layer(service_fn(|()| async { Err::<(), _>("call failure") }));
    assert_eq!(failed.oneshot(()).await, Err("call failure"));
}
#[tokio::test]
async fn dropping_readiness_or_response_future_drops_owned_service_work() {
    let state = Arc::new(Readiness::default());
    let mut wrapped = SharedRouteLayer.layer(ReadinessService {
        state: state.clone(),
        ready: false,
    });
    let mut context = Context::from_waker(Waker::noop());
    let mut future = Box::pin(wrapped.call(()));
    assert!(future.as_mut().poll(&mut context).is_pending());
    let before = state.drops.load(Ordering::SeqCst);
    drop(future);
    assert_eq!(state.drops.load(Ordering::SeqCst), before + 1);
    assert_eq!(state.calls.load(Ordering::SeqCst), 0);
    struct Work(Arc<AtomicUsize>);
    impl Drop for Work {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::SeqCst);
        }
    }
    let active = Arc::new(AtomicUsize::new(0));
    let tracked = active.clone();
    let service = SharedRouteLayer.layer(service_fn(move |()| {
        tracked.fetch_add(1, Ordering::SeqCst);
        let work = Work(tracked.clone());
        async move {
            let _work = work;
            std::future::pending::<Result<(), Infallible>>().await
        }
    }));
    let mut future = Box::pin(service.oneshot(()));
    assert!(future.as_mut().poll(&mut context).is_pending());
    assert_eq!(active.load(Ordering::SeqCst), 1);
    drop(future);
    assert_eq!(active.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn concurrent_requests_are_not_serialized_or_detached_into_background_work() {
    let barrier = Arc::new(tokio::sync::Barrier::new(8));
    let active = Arc::new(AtomicUsize::new(0));
    let maximum = Arc::new(AtomicUsize::new(0));
    let (wait, running, peak) = (barrier.clone(), active.clone(), maximum.clone());
    let service = SharedRouteLayer.layer(service_fn(move |value: usize| {
        let (wait, running, peak) = (wait.clone(), running.clone(), peak.clone());
        async move {
            let current = running.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(current, Ordering::SeqCst);
            wait.wait().await;
            running.fetch_sub(1, Ordering::SeqCst);
            Ok::<_, Infallible>(value)
        }
    }));
    let tasks = (0..8)
        .map(|value| tokio::spawn(service.clone().oneshot(value)))
        .collect::<Vec<_>>();
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        for (expected, task) in tasks.into_iter().enumerate() {
            assert_eq!(task.await.unwrap().unwrap(), expected);
        }
    })
    .await
    .expect("all eight requests must reach the barrier concurrently");
    assert_eq!(maximum.load(Ordering::SeqCst), 8);
    assert_eq!(active.load(Ordering::SeqCst), 0);
}
