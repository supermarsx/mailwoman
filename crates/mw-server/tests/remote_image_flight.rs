#[path = "../src/remote_image_flight.rs"]
mod flight;

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

#[tokio::test]
async fn concurrent_misses_fetch_and_decode_once_then_read_completed_cache() {
    let cache = Arc::new(tokio::sync::Mutex::new(None::<Vec<u8>>));
    let fetches = Arc::new(AtomicUsize::new(0));
    let decodes = Arc::new(AtomicUsize::new(0));
    let counted = fetches.clone();
    let router = axum::Router::new().route(
        "/image",
        axum::routing::get(move || {
            let counted = counted.clone();
            async move {
                counted.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(30)).await;
                include_bytes!("../../../apps/web/public/icons/icon-192.png").as_slice()
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/image", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let mut jobs = Vec::new();
    for _ in 0..12 {
        let (cache, decodes, client, url) =
            (cache.clone(), decodes.clone(), client.clone(), url.clone());
        jobs.push(tokio::spawn(async move {
            let _guard = flight::acquire(
                "dedup",
                "route:policy",
                "image",
                Instant::now() + Duration::from_secs(20),
            )
            .await
            .unwrap();
            if let Some(value) = cache.lock().await.clone() {
                return value;
            }
            let raw = client.get(url).send().await.unwrap().bytes().await.unwrap();
            decodes.fetch_add(1, Ordering::SeqCst);
            let png = tokio::task::spawn_blocking(move || {
                mw_render::media_jail::reencode_image(&raw).unwrap()
            })
            .await
            .unwrap();
            *cache.lock().await = Some(png.clone());
            png
        }));
    }
    for job in jobs {
        assert!(job.await.unwrap().starts_with(b"\x89PNG"));
    }
    assert_eq!(fetches.load(Ordering::SeqCst), 1);
    assert_eq!(decodes.load(Ordering::SeqCst), 1);
    server.abort();
}

#[tokio::test]
async fn canceled_leader_releases_waiter_and_wait_budget_is_not_reset() {
    let leader = flight::acquire(
        "cancel",
        "route",
        "url",
        Instant::now() + Duration::from_secs(1),
    )
    .await
    .unwrap();
    assert!(
        flight::acquire(
            "cancel",
            "route",
            "url",
            Instant::now() + Duration::from_millis(15)
        )
        .await
        .is_none()
    );
    let (tx, rx) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(async move {
        let _held = leader;
        let _ = tx.send(());
        std::future::pending::<()>().await;
    });
    rx.await.unwrap();
    task.abort();
    let _ = task.await;
    assert!(
        flight::acquire(
            "cancel",
            "route",
            "url",
            Instant::now() + Duration::from_millis(50)
        )
        .await
        .is_some()
    );
}

#[tokio::test]
async fn account_route_policy_and_url_partition_independent_work() {
    let _held = flight::acquire(
        "partition",
        "route:policy",
        "url",
        Instant::now() + Duration::from_secs(1),
    )
    .await
    .unwrap();
    for (account, route, url) in [
        ("other", "route:policy", "url"),
        ("partition", "other:policy", "url"),
        ("partition", "route:other", "url"),
        ("partition", "route:policy", "other"),
    ] {
        assert!(
            flight::acquire(
                account,
                route,
                url,
                Instant::now() + Duration::from_millis(30)
            )
            .await
            .is_some()
        );
    }
}
