//! 404s are a durable series of their own, with redacted top paths (t61).
use mw_server::{AppConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;

#[tokio::test]
async fn not_found_series_and_redacted_top_paths_across_windows() {
    let db = common::test_db::unique_db_path("not-found-metrics")
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
    let paths = format!("{base}/admin/metrics/not-found");
    let history = format!("{base}/admin/metrics/history");
    assert_eq!(client.get(&paths).send().await.unwrap().status(), 401);
    // Not-found probes, several carrying secrets that must never be stored.
    let probes = [
        "/wp-login.php?user=admin&password=hunter2",
        "/wp-login.php",
        "/reset/Zm9vYmFyYmF6cXV4MTIzNDU2Nzg5MA",
        "/u/alice@example.com/inbox",
        "/api/rc/zz/c2lnbmVkLXJjLXRva2VuLXNlY3JldA.c2ln",
        "/api/definitely-not-a-route/550e8400-e29b-41d4-a716-446655440000",
    ];
    for probe in probes {
        let status = client
            .get(format!("{base}{probe}"))
            .send()
            .await
            .unwrap()
            .status();
        assert_eq!(status, 404, "{probe}");
    }
    // The app's own designed miss: not counted.
    for _ in 0..3 {
        client
            .get(format!("{base}/favicon.ico"))
            .send()
            .await
            .unwrap();
    }
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
    // An older, pre-tracking snapshot: reported as "not tracked", never as zero.
    let store = mw_store::Store::open(&db, mw_store::ServerKey::from_hex(key).unwrap())
        .await
        .unwrap();
    let current = chrono::Utc::now().timestamp();
    let legacy_start = current / 3600 * 3600 - 5 * 3600;
    let legacy: mw_store::request_metrics::Counts = serde_json::from_value(
        json!({"requests":4,"errors5xx":0,"histogram":[4,0,0,0,0,0,0,0,0,0,0,0,0,0],"max_ms":0}),
    )
    .unwrap();
    store
        .save_request_metrics(
            "pre-tracking",
            &[mw_store::request_metrics::Bucket {
                seconds: 3600,
                start: legacy_start,
                observed_from: legacy_start,
                observed_to: legacy_start + 3600,
                counts: legacy,
            }],
            current,
            30,
        )
        .await
        .unwrap();
    // Let the periodic writer flush the middleware's real buckets.
    tokio::time::sleep(std::time::Duration::from_secs(17)).await;
    let now = chrono::Utc::now().timestamp();
    let minute: Value = client
        .get(format!(
            "{history}?from={}&to={now}&maxPoints=60",
            now - 3600
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let counted: u64 = minute["series"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["notFound"].as_u64())
        .sum();
    assert_eq!(counted, probes.len() as u64, "{minute}");
    assert!(minute["coverage"]["notFound"]["trackedSince"].is_i64());
    for window in [6 * 3600, 86400, 7 * 86400, 29 * 86400] {
        let body: Value = client
            .get(format!(
                "{history}?from={}&to={now}&maxPoints=240",
                now - window
            ))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let series = body["series"].as_array().unwrap();
        let counted: u64 = series.iter().filter_map(|p| p["notFound"].as_u64()).sum();
        assert_eq!(counted, probes.len() as u64, "{window}: {body}");
        if window > 48 * 3600 {
            // Hourly tier: the pre-tracking hour is marked, not zero-filled.
            assert_eq!(body["bucketSeconds"].as_i64().unwrap() % 3600, 0);
            assert!(
                series
                    .iter()
                    .any(|p| p["notFoundCoverage"] == "notTracked" && p["notFound"].is_null()),
                "{window}"
            );
            assert_eq!(
                body["coverage"]["notFound"]["notTracked"][0]["from"],
                legacy_start
            );
        }
        let top: Value = client
            .get(format!("{paths}?from={}&to={now}", now - window))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(top["total"], probes.len() as u64, "{top}");
    }
    let response = client
        .get(format!("{paths}?from={}&to={now}", now - 3600))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let top: Value = response.json().await.unwrap();
    let text = top.to_string();
    for secret in [
        "hunter2",
        "password",
        "alice",
        "example.com",
        "Zm9vYmFy",
        "c2lnbmVk",
        "550e8400",
        "favicon",
        "127.0.0.1",
    ] {
        assert!(!text.contains(secret), "{secret} leaked: {text}");
    }
    let rows: Vec<(String, String, u64)> = top["paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p["method"].as_str().unwrap().to_owned(),
                p["path"].as_str().unwrap().to_owned(),
                p["count"].as_u64().unwrap(),
            )
        })
        .collect();
    assert!(
        rows.contains(&("GET".into(), "/wp-login.php".into(), 2)),
        "{rows:?}"
    );
    assert!(
        rows.contains(&("GET".into(), "/reset/{token}".into(), 1)),
        "{rows:?}"
    );
    assert!(
        rows.contains(&("GET".into(), "/u/{email}/inbox".into(), 1)),
        "{rows:?}"
    );
    assert!(
        rows.contains(&("GET".into(), "/api/rc/zz/{rc-token}".into(), 1)),
        "{rows:?}"
    );
    assert!(
        rows.contains(&("GET".into(), "/api/definitely-not-a-route/{uuid}".into(), 1)),
        "{rows:?}"
    );
    assert_eq!(top["limit"], 50);
    assert_eq!(
        client
            .get(format!("{paths}?from={}", now - 60))
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    // Top paths need unscoped Observability read; Dashboard read sees only counts.
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
    for (name, cap, domains, history_status, paths_status) in [
        ("nf-observer", "observability.read", json!(null), 200, 200),
        ("nf-dashboard", "dashboard.read", json!(null), 200, 403),
        (
            "nf-scoped",
            "observability.read",
            json!(["example.org"]),
            403,
            403,
        ),
    ] {
        let role: Value = client
            .post(format!("{base}/admin/roles"))
            .json(&json!({"name":name,"capabilities":[cap]}))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
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
            history_status,
            "{name}"
        );
        assert_eq!(
            reader.get(&paths).send().await.unwrap().status(),
            paths_status,
            "{name}"
        );
    }
}
