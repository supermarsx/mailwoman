//! t57: at the concurrent-request limit N, N + K simultaneous requests give K waits
//! that are served as permits free, not K 503s; with waiting off they are 503s as
//! before; health probes are answered while the pool is full and requests wait.
use axum::{Router, routing::get};
use mw_server::inbound_http::{HttpLimits, serve};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::sync::{oneshot, watch};

struct Fixture {
    addr: std::net::SocketAddr,
    entered: Arc<AtomicUsize>,
    release: watch::Sender<bool>,
    _stop: oneshot::Sender<()>,
}

async fn fixture(limits: HttpLimits) -> Fixture {
    let entered = Arc::new(AtomicUsize::new(0));
    let (release, gate) = watch::channel(false);
    let app = Router::new()
        .route(
            "/slow",
            get({
                let entered = entered.clone();
                move || {
                    let entered = entered.clone();
                    let mut gate = gate.clone();
                    async move {
                        entered.fetch_add(1, Ordering::SeqCst);
                        let _ = gate.wait_for(|open| *open).await;
                        "done"
                    }
                }
            }),
        )
        .route("/healthz", get(|| async { "ok" }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (stop, stopped) = oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = serve(listener, app, limits, async {
            let _ = stopped.await;
        })
        .await;
    });
    Fixture {
        addr,
        entered,
        release,
        _stop: stop,
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap()
}

async fn until(what: &str, mut done: impl FnMut() -> bool) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while !done() {
        assert!(tokio::time::Instant::now() < deadline, "timed out: {what}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

const N: usize = 2;
const K: usize = 3;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn requests_over_the_limit_wait_and_are_served_when_permits_free() {
    let server = fixture(HttpLimits {
        requests: N,
        queue_wait: Duration::from_secs(2),
        queue_max: K,
        ..HttpLimits::default()
    })
    .await;
    let client = client();
    let url = format!("http://{}/slow", server.addr);
    let calls: Vec<_> = (0..N + K)
        .map(|_| {
            let (client, url) = (client.clone(), url.clone());
            tokio::spawn(async move { client.get(url).send().await.unwrap().status() })
        })
        .collect();
    until("N handlers entered", || {
        server.entered.load(Ordering::SeqCst) == N
    })
    .await;
    // The other K are queued, not refused: none has finished.
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(server.entered.load(Ordering::SeqCst), N);
    assert!(
        calls.iter().all(|call| !call.is_finished()),
        "a request was refused instead of waiting"
    );
    // A probe is answered while the pool is full and requests wait.
    let probe = client
        .get(format!("http://{}/healthz", server.addr))
        .send()
        .await
        .unwrap();
    assert_eq!(probe.status(), 200);
    // One more than the queue holds is refused at once.
    let started = tokio::time::Instant::now();
    let refused = client.get(url.clone()).send().await.unwrap();
    assert_eq!(refused.status(), 503);
    assert_eq!(refused.headers()["retry-after"], "1");
    assert!(started.elapsed() < Duration::from_secs(1));
    server.release.send(true).unwrap();
    for call in calls {
        assert_eq!(call.await.unwrap(), 200);
    }
    assert_eq!(server.entered.load(Ordering::SeqCst), N + K);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn with_waiting_off_the_excess_is_refused_at_once() {
    let server = fixture(HttpLimits {
        requests: N,
        queue_wait: Duration::ZERO,
        ..HttpLimits::default()
    })
    .await;
    let client = client();
    let url = format!("http://{}/slow", server.addr);
    let held: Vec<_> = (0..N)
        .map(|_| {
            let (client, url) = (client.clone(), url.clone());
            tokio::spawn(async move { client.get(url).send().await.unwrap().status() })
        })
        .collect();
    until("N handlers entered", || {
        server.entered.load(Ordering::SeqCst) == N
    })
    .await;
    for _ in 0..K {
        let response = client.get(url.clone()).send().await.unwrap();
        assert_eq!(response.status(), 503);
        assert_eq!(response.headers()["retry-after"], "1");
    }
    server.release.send(true).unwrap();
    for call in held {
        assert_eq!(call.await.unwrap(), 200);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_short_deadline_refuses_with_503_after_waiting() {
    let server = fixture(HttpLimits {
        requests: 1,
        queue_wait: Duration::from_millis(150),
        ..HttpLimits::default()
    })
    .await;
    let client = client();
    let url = format!("http://{}/slow", server.addr);
    let held = {
        let (client, url) = (client.clone(), url.clone());
        tokio::spawn(async move { client.get(url).send().await.unwrap().status() })
    };
    until("handler entered", || {
        server.entered.load(Ordering::SeqCst) == 1
    })
    .await;
    let started = tokio::time::Instant::now();
    let response = client.get(url.clone()).send().await.unwrap();
    assert_eq!(response.status(), 503);
    assert!(started.elapsed() >= Duration::from_millis(140));
    server.release.send(true).unwrap();
    assert_eq!(held.await.unwrap(), 200);
}
