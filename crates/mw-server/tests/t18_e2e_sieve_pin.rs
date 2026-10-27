//! Client-supplied ManageSieve destinations are refused before connection.
//! Private-network access is now an operator endpoint permission; guarded
//! proxy/pinning behavior is covered in mw-egress plan::sieve_tests.
//!
//! Since 7e9af0d0 the refusal is structural: the old `POST /api/account/sieve/sync`
//! upload route, the one place a request body could name a ManageSieve host, port
//! and credentials, is removed (`sieve_sync_router` is an empty seam) and ManageSieve
//! is reached only through the operator-selected endpoint. So the route answers
//! `404` for every body. The leg also points one body at a live local listener and
//! checks it is never dialled, which a `404` alone would not prove.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use serde_json::json;

use mw_server::{AppConfig, build_app};
use mw_store::{Credentials, ServerKey, Store};

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";

mod common;
use common::test_db;

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

fn temp_db() -> String {
    let dir = test_db::unique_dir("mw-t18-sievepin");
    dir.join("mw.db").to_string_lossy().into_owned()
}

/// Spawn `build_app` in ENGINE mode (the sieve route requires an engine) on `db_path`.
async fn spawn_engine_server(db_path: &str) -> SocketAddr {
    let web = PathBuf::from(db_path)
        .parent()
        .unwrap()
        .join(format!("web-{}", test_db::unique_tag()));
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: db_path.to_string(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: mw_server::ServerMode::Engine,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    };
    let app = build_app(config).await.expect("build_app engine mode");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

/// Seed a real session row (same key as the server) and return its cookie token.
async fn seed_session(db_path: &str) -> String {
    let store = Store::open(db_path, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    store
        .create_session(
            "acct-sieve-pin",
            "user@example.org",
            "http://upstream.invalid",
            "http://upstream.invalid",
            &Credentials {
                username: "user@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap()
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(4))
        .build()
        .unwrap()
}

async fn sieve_sync(
    c: &reqwest::Client,
    base: &str,
    cookie: &str,
    host: &str,
    port: u16,
) -> Result<reqwest::StatusCode, reqwest::Error> {
    c.post(format!("{base}/api/account/sieve/sync"))
        .header("Cookie", format!("mw_session={cookie}"))
        .json(&json!({
            "host": host,
            "port": port,
            "tls": "plaintext",
            "username": "u",
            "password": "p",
        }))
        .send()
        .await
        .map(|r| r.status())
}

#[tokio::test]
async fn sieve_refuses_client_selected_destinations_and_credentials() {
    let db = temp_db();
    let addr = spawn_engine_server(&db).await;
    let base = format!("http://{addr}");
    let cookie = seed_session(&db).await;
    let client = client();
    for host in [
        "169.254.169.254",
        "127.0.0.1",
        "::1",
        "10.255.255.1",
        "mail.example.org",
    ] {
        let status = sieve_sync(&client, &base, &cookie, host, 4190)
            .await
            .unwrap();
        assert_eq!(
            status.as_u16(),
            404,
            "request body cannot configure {host}: the client-selected sync route is gone \
             (7e9af0d0), so no body reaches a ManageSieve dial"
        );
    }

    // The same body aimed at a real listener: it must never be dialled.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let status = sieve_sync(&client, &base, &cookie, "127.0.0.1", port)
        .await
        .unwrap();
    assert_eq!(status.as_u16(), 404);
    let dialled = tokio::time::timeout(Duration::from_millis(300), listener.accept()).await;
    assert!(
        dialled.is_err(),
        "a client-named ManageSieve destination was contacted"
    );
}
