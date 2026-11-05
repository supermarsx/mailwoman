//! `POST /api/client-timing`: session required, strict validation, per-session
//! rate limit, CSRF, 204 with no body. Over loopback against the mock upstream.

use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, build_app_with_push};

mod common;
use common::test_db;

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

async fn spawn() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    let upstream = format!("http://{addr}");
    let base = test_db::unique_dir("mw-client-timing");
    let web_dir = base.join("web");
    std::fs::create_dir_all(&web_dir).unwrap();
    std::fs::write(web_dir.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(PathBuf::from(&web_dir)),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: HardeningConfig {
            csrf_strict: true,
            ..HardeningConfig::default()
        },
        security: mw_server::SecurityConfig {
            jmap_upstreams: Some(vec![upstream.clone()]),
            upstream_default: Some(upstream),
            ..mw_server::SecurityConfig::default()
        },
    };
    let (app, _push) = build_app_with_push(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

async fn sign_in(server: &str) -> (reqwest::Client, String) {
    let (c, token, _) = sign_in_full(server).await;
    (c, token)
}

async fn sign_in_full(server: &str) -> (reqwest::Client, String, String) {
    let c = reqwest::Client::builder().cookie_store(true).build().unwrap();
    let resp = c
        .post(format!("{server}/api/login"))
        .json(&json!({"username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    (
        c,
        body["csrfToken"].as_str().unwrap().to_owned(),
        body["storageAuthority"].as_str().unwrap_or_default().to_owned(),
    )
}

fn beacon(c: &reqwest::Client, server: &str, token: Option<&str>, body: String) -> reqwest::RequestBuilder {
    let req = c
        .post(format!("{server}/api/client-timing"))
        .header("content-type", "application/json")
        .body(body);
    match token {
        Some(t) => req.header("x-csrf-token", t),
        None => req,
    }
}

const OK: &str = r#"{"device":"desktop","metrics":[{"name":"timeToUsable","ms":1200},{"name":"fcp","ms":400}]}"#;

#[tokio::test]
async fn anonymous_is_rejected_and_a_session_gets_204_with_no_body() {
    let server = spawn().await;
    let anon = reqwest::Client::new();
    let r = beacon(&anon, &server, None, OK.into()).send().await.unwrap();
    // No session: refused by the strict CSRF guard (403) before the handler's own 401.
    assert!(matches!(r.status().as_u16(), 401 | 403), "{}", r.status());

    let (c, token) = sign_in(&server).await;
    // Same-session writes without the CSRF token are refused first.
    let r = beacon(&c, &server, None, OK.into()).send().await.unwrap();
    assert_eq!(r.status(), 403);
    let r = beacon(&c, &server, Some(&token), OK.into()).send().await.unwrap();
    assert_eq!(r.status(), 204);
    assert!(r.text().await.unwrap().is_empty());

    let policy: Value = c
        .get(format!("{server}/api/policy"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(policy["clientTiming"]["enabled"], true);
    assert_eq!(policy["clientTiming"]["sampleRatePercent"], 100);
}

#[tokio::test]
async fn invalid_beacons_are_refused() {
    let server = spawn().await;
    let (c, token) = sign_in(&server).await;
    for body in [
        r#"{"metrics":[{"name":"path","ms":5}]}"#,
        r#"{"metrics":[{"name":"fcp","ms":-4}]}"#,
        r#"{"metrics":[{"name":"fcp","ms":1},{"name":"fcp","ms":2}]}"#,
        r#"{"device":"tv","metrics":[{"name":"fcp","ms":1}]}"#,
        r#"{"url":"/inbox","metrics":[{"name":"fcp","ms":1}]}"#,
        "not json",
    ] {
        let r = beacon(&c, &server, Some(&token), body.into()).send().await.unwrap();
        assert_eq!(r.status(), 400, "{body}");
    }
    // The invalid beacons above spent this session's burst (6); a new session has its own.
    let (c, token) = sign_in(&server).await;
    // An out-of-range value is clamped, not refused.
    let r = beacon(
        &c,
        &server,
        Some(&token),
        r#"{"metrics":[{"name":"load","ms":99999999}]}"#.into(),
    )
    .send()
    .await
    .unwrap();
    assert_eq!(r.status(), 204);
    let big = format!(r#"{{"metrics":[{{"name":"fcp","ms":1}}],"pad":"{}"}}"#, "x".repeat(4000));
    let r = beacon(&c, &server, Some(&token), big).send().await.unwrap();
    assert_eq!(r.status(), 413);
}

#[tokio::test]
async fn one_session_cannot_flood_the_aggregate() {
    let server = spawn().await;
    let (c, token) = sign_in(&server).await;
    let mut statuses = Vec::new();
    for _ in 0..12 {
        statuses.push(
            beacon(&c, &server, Some(&token), OK.into())
                .send()
                .await
                .unwrap()
                .status()
                .as_u16(),
        );
    }
    assert_eq!(statuses.iter().filter(|s| **s == 204).count(), 6, "{statuses:?}");
    assert!(statuses.iter().all(|s| *s == 204 || *s == 429), "{statuses:?}");
}

const INTERACTIONS: &str = r#"{"device":"desktop","metrics":[{"name":"folderLoadCold","ms":420},{"name":"folderLoadCached","ms":18},{"name":"messageLoadCold","ms":900},{"name":"messageLoadCached","ms":25},{"name":"messageLoadCold","ms":1500}]}"#;

#[tokio::test]
async fn folder_and_message_loads_are_on_by_default_and_a_person_can_opt_out() {
    let server = spawn().await;
    let (c, token, authority) = sign_in_full(&server).await;
    let policy = |c: reqwest::Client, server: String| async move {
        let v: Value = c.get(format!("{server}/api/policy")).send().await.unwrap().json().await.unwrap();
        v["clientTiming"].clone()
    };
    let before = policy(c.clone(), server.clone()).await;
    assert_eq!(before["folderLoads"], true);
    assert_eq!(before["messageLoads"], true);

    // A batched interaction beacon is accepted; its own bucket allows more than the page-load one.
    for _ in 0..10 {
        let r = beacon(&c, &server, Some(&token), INTERACTIONS.into()).send().await.unwrap();
        assert_eq!(r.status(), 204);
    }

    // The person turns message loads off: the policy says so, and a stale client's beacon is
    // still answered 204 (and dropped), never an error.
    let off = c
        .put(format!("{server}/api/settings/observability.clientTiming.messageLoads"))
        .header("x-csrf-token", &token)
        .header("x-mailwoman-storage-authority", &authority)
        .json(&json!({"value": false}))
        .send()
        .await
        .unwrap();
    assert_eq!(off.status(), 200, "{}", off.text().await.unwrap());
    let after = policy(c.clone(), server.clone()).await;
    assert_eq!(after["messageLoads"], false);
    assert_eq!(after["folderLoads"], true);
    let r = beacon(&c, &server, Some(&token), INTERACTIONS.into()).send().await.unwrap();
    assert_eq!(r.status(), 204);
}

#[tokio::test]
async fn interaction_beacons_are_validated_like_page_loads() {
    let server = spawn().await;
    let (c, token) = sign_in(&server).await;
    for body in [
        r#"{"metrics":[{"name":"messageLoad:inbox","ms":5}]}"#,
        r#"{"metrics":[{"name":"folderLoadCold","ms":5,"folder":"Inbox"}]}"#,
        r#"{"metrics":[{"name":"folderLoadCold","ms":-5}]}"#,
    ] {
        let r = beacon(&c, &server, Some(&token), body.into()).send().await.unwrap();
        assert_eq!(r.status(), 400, "{body}");
    }
    let r = beacon(&c, &server, Some(&token), r#"{"metrics":[{"name":"messageLoadCold","ms":99999999}]}"#.into())
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204);
}
