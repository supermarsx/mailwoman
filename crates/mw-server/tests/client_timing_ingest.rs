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
    let c = reqwest::Client::builder().cookie_store(true).build().unwrap();
    let resp = c
        .post(format!("{server}/api/login"))
        .json(&json!({"username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    (c, body["csrfToken"].as_str().unwrap().to_owned())
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
    assert_eq!(r.status(), 401);

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
