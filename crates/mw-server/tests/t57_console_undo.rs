//! t57: a console save of the smallest request values cannot lock the admin out.
//! With the request limit at 1, no wait, and the one slot held, an ordinary request
//! is refused at once, but an admin-console request is still admitted (from the
//! admin reserve) and can put the values back; ordinary requests are then served.
//! The handler under the admin path stands in for the settings write: it applies
//! console values to the running listener the way a settings change does.
use axum::{Router, extract::Query, routing::get};
use mw_server::inbound_http::{
    HttpLimits,
    admission::current_admission,
    live::Console,
    serve,
};
use std::{collections::HashMap, time::Duration};
use tokio::sync::{oneshot, watch};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_minimum_console_save_can_be_undone_from_the_console() {
    let (release, gate) = watch::channel(false);
    let app = Router::new()
        .route(
            "/slow",
            get(move || {
                let mut gate = gate.clone();
                async move {
                    let _ = gate.wait_for(|open| *open).await;
                    "done"
                }
            }),
        )
        .route("/plain", get(|| async { "plain" }))
        .route(
            "/admin/limits",
            get(|Query(q): Query<HashMap<String, u64>>| async move {
                let console = Console {
                    requests: q.get("requests").copied(),
                    queue_wait_ms: q.get("wait").copied(),
                    ..Console::default()
                };
                current_admission().expect("serving").apply(console);
                "applied"
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (_stop, stopped) = oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = serve(listener, app, HttpLimits::default(), async {
            let _ = stopped.await;
        })
        .await;
    });
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let get = |path: &str| client.get(format!("http://{addr}{path}")).send();

    // The smallest values: one request at a time, no wait.
    assert_eq!(get("/admin/limits?requests=1&wait=0").await.unwrap().status(), 200);
    let held = {
        let (client, url) = (client.clone(), format!("http://{addr}/slow"));
        tokio::spawn(async move { client.get(url).send().await })
    };
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let status = get("/plain").await.unwrap().status();
        if status == 503 {
            break;
        }
        assert!(tokio::time::Instant::now() < deadline, "the slot was never taken");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    // The console still gets in and puts the values back.
    assert_eq!(get("/admin/limits").await.unwrap().status(), 200);
    assert_eq!(get("/plain").await.unwrap().status(), 200);
    let snapshot = current_admission().unwrap().snapshot();
    assert_eq!(snapshot["effective"]["requests"]["value"], 256);
    assert!(snapshot["admittedFromAdminReserve"].as_u64().unwrap() >= 1);
    release.send(true).unwrap();
    assert_eq!(held.await.unwrap().unwrap().status(), 200);
}
