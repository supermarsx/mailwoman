//! Production middleware and the authenticated process summary share counters.
use mw_server::{AppConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;

#[tokio::test]
async fn summary_requires_admin_and_counts_real_requests_without_paths() {
    let db = common::test_db::unique_db_path("http-metrics")
        .to_string_lossy()
        .into_owned();
    let key = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
    let (app, _) = build_app_full(
        AppConfig {
            db_path: db.clone(),
            server_key_hex: Some(key.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: Default::default(),
            security: SecurityConfig::default(),
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("synthetic-test-password".into()),
            redis_url: None,
        },
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let path = format!("{base}/admin/metrics/summary");
    assert_eq!(client.get(&path).send().await.unwrap().status(), 401);
    let history = format!("{base}/admin/metrics/history");
    assert_eq!(client.get(&history).send().await.unwrap().status(), 401);
    assert_eq!(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"synthetic-test-password"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    for _ in 0..3 {
        assert_eq!(
            client
                .get(format!("{base}/healthz?secret=should-never-be-a-label"))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
    }
    let response = client.get(path).send().await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let data: Value = response.json().await.unwrap();
    assert_eq!(data["perProcess"], true);
    assert_eq!(data["series"].as_array().unwrap().len(), 60);
    let api = data["totals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["class"] == "api")
        .unwrap();
    assert!(api["requests"].as_u64().unwrap() >= 3);
    let admin = data["totals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["class"] == "admin")
        .unwrap();
    assert!(admin["errors4xx"].as_u64().unwrap() >= 1);
    assert!(!data.to_string().contains("should-never-be-a-label"));
    let endpoint = mw_server::admin_router::registry()
        .into_iter()
        .find(|endpoint| endpoint.suffix == "/metrics/summary")
        .unwrap();
    assert!(
        matches!(endpoint.access, mw_server::admin_router::Access::Cap(cap)
        if cap == mw_server::capabilities::Capability::ObservabilityRead && cap.unscoped_only())
    );
    let store = mw_store::Store::open(&db, mw_store::ServerKey::from_hex(key).unwrap())
        .await
        .unwrap();
    let current = chrono::Utc::now().timestamp();
    let start = current / 60 * 60 - 600;
    let mut counts = mw_store::request_metrics::Counts::default();
    counts.record(200, 37);
    counts.record(500, 83);
    let bucket = mw_store::request_metrics::Bucket {
        seconds: 60,
        start,
        observed_from: start,
        observed_to: start + 60,
        counts,
    };
    store
        .save_request_metrics("http-fixture", std::slice::from_ref(&bucket), current, 30)
        .await
        .unwrap();
    store
        .save_request_metrics("http-fixture", &[bucket], current, 30)
        .await
        .unwrap();
    let response = client
        .get(format!(
            "{history}?from={start}&to={}&maxPoints=2",
            start + 120
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["series"][0]["requests"], 2);
    assert_eq!(body["series"][0]["errors5xx"], 1);
    assert_eq!(body["series"][0]["p50Ms"], 37);
    assert_eq!(body["series"][0]["p95Ms"], 83);
    assert_eq!(body["series"][0]["p95ResolutionMs"], 1);
    assert_eq!(body["series"][0]["legacyLatencyEstimate"], false);
    assert_eq!(body["series"][0]["coverage"], "complete");
    assert!(body["series"][1]["requests"].is_null());
    assert_eq!(body["series"][1]["coverage"], "unavailable");
    assert_eq!(body["coverage"]["observedSeconds"], 60);
    let legacy=mw_store::request_metrics::Bucket{seconds:60,start:start-60,observed_from:start-60,observed_to:start,counts:serde_json::from_value(json!({"requests":2,"errors5xx":0,"histogram":[0,0,0,0,0,1,1,0,0,0,0,0,0,0],"max_ms":83})).unwrap()};
    store
        .save_request_metrics("legacy-fixture", &[legacy], current, 30)
        .await
        .unwrap();
    let mixed: Value = client
        .get(format!(
            "{history}?from={}&to={}&maxPoints=1",
            start - 60,
            start + 60
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(mixed["series"][0]["p95Ms"], 100);
    assert_eq!(mixed["series"][0]["legacyLatencyEstimate"], true);
    assert!(mixed["series"][0]["p95ResolutionMs"].is_null());
    assert_eq!(
        client
            .get(format!("{history}?from={start}"))
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    assert_eq!(
        client
            .get(format!("{history}?maxPoints=1001"))
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    assert_eq!(client.get(&history).send().await.unwrap().status(), 200);
    assert_eq!(
        client
            .post(format!("{base}/admin/step-up"))
            .json(&json!({"password":"synthetic-test-password"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    for (name, cap, domains, expected) in [
        ("history-observer", "observability.read", json!(null), 200),
        ("history-dashboard", "dashboard.read", json!(null), 200),
        (
            "history-scoped",
            "dashboard.read",
            json!(["example.org"]),
            403,
        ),
        ("history-denied", "users.read", json!(null), 403),
    ] {
        let response = client
            .post(format!("{base}/admin/roles"))
            .json(&json!({"name":name,"capabilities":[cap]}))
            .send()
            .await
            .unwrap();
        assert!(
            response.status().is_success(),
            "{}",
            response.text().await.unwrap()
        );
        let role: Value = response.json().await.unwrap();
        let response=client.post(format!("{base}/admin/admins")).json(&json!({"username":name,"password":"long test password","roles":[{"roleId":role["id"],"domains":domains}]})).send().await.unwrap();
        assert!(
            response.status().is_success(),
            "{}",
            response.text().await.unwrap()
        );
        let reader = reqwest::Client::builder()
            .no_proxy()
            .cookie_store(true)
            .build()
            .unwrap();
        assert_eq!(
            reader
                .post(format!("{base}/admin/login"))
                .json(&json!({"username":name,"password":"long test password"}))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        assert_eq!(
            reader.get(&history).send().await.unwrap().status(),
            expected,
            "{name}"
        );
    }
    // The periodic writer captures actual middleware requests, not just fixtures.
    tokio::time::sleep(std::time::Duration::from_secs(16)).await;
    let rows = store
        .request_metrics_history(current / 60 * 60, chrono::Utc::now().timestamp() + 60, 60)
        .await
        .unwrap();
    assert!(rows.iter().map(|row| row.counts.requests).sum::<u64>() >= 3);
}
