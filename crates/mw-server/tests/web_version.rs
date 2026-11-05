//! `GET /api/version`: session required; reports the binary version and a hash of
//! the served web bundle that changes when the bundle does (new-version toast).

use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, build_app_with_push};

mod common;
use common::test_db;

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

async fn spawn() -> (String, PathBuf) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    let upstream = format!("http://{addr}");
    let base = test_db::unique_dir("mw-web-version");
    let web_dir = base.join("web");
    std::fs::create_dir_all(&web_dir).unwrap();
    std::fs::write(web_dir.join("index.html"), INDEX_HTML).unwrap();
    std::fs::create_dir_all(web_dir.join("assets")).unwrap();
    std::fs::write(web_dir.join("assets/index-aaaa.js"), "a").unwrap();
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
    (format!("http://{addr}"), web_dir)
}

async fn sign_in(server: &str) -> reqwest::Client {
    let c = reqwest::Client::builder().cookie_store(true).build().unwrap();
    let resp = c
        .post(format!("{server}/api/login"))
        .json(&json!({"username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    c
}

#[tokio::test]
async fn version_needs_a_session_and_tracks_the_served_bundle() {
    let (server, web_dir) = spawn().await;
    let anon = reqwest::get(format!("{server}/api/version")).await.unwrap();
    assert_eq!(anon.status(), 401);

    let c = sign_in(&server).await;
    let get = || async {
        let r = c.get(format!("{server}/api/version")).send().await.unwrap();
        assert_eq!(r.status(), 200);
        assert_eq!(r.headers()["cache-control"], "no-store");
        r.json::<Value>().await.unwrap()
    };
    let first = get().await;
    assert_eq!(first["version"], env!("CARGO_PKG_VERSION"));
    let build = first["webBuild"].as_str().unwrap().to_owned();
    assert_eq!(build.len(), 16);
    assert_eq!(get().await["webBuild"], build, "stable while the bundle is unchanged");

    std::fs::write(web_dir.join("assets/index-bbbb.js"), "b").unwrap();
    assert_ne!(get().await["webBuild"], build, "a redeployed bundle changes the id");
}
