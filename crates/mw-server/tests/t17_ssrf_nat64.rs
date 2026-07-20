//! t17-e-e2e — SSRF NAT64 / 6to4 embedded-IPv4 refusal (L3) LIVE through the real
//! image proxy.
//!
//! 26.16's SSRF gate blocked literal private/loopback/metadata targets but not those
//! same addresses SMUGGLED inside an IPv6 NAT64 (`64:ff9b::/96`) or 6to4 (`2002::/16`)
//! embedding. 26.17 decodes the embedded IPv4 and re-checks it. `image_proxy.rs`
//! unit-tests the decode; THIS leg drives the REAL `/api/image-proxy` route (behind a
//! real session) and asserts a private/loopback/metadata v4 smuggled through NAT64/6to4
//! is REFUSED live (403), while a public v4 via NAT64 passes the gate (it is NOT a 403 —
//! it only fails later because this host has no NAT64 gateway).
//!
//! Run:
//!   cargo test -p mw-server --test t17_ssrf_nat64 -- --nocapture --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, build_app};

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

async fn spawn_server() -> SocketAddr {
    let base = std::env::temp_dir().join(format!("mw-t17-nat64-{}", unique()));
    let web = base.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web as PathBuf),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn login(c: &reqwest::Client, base: &str, mock: &str) {
    let body: Value = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "jmapUrl": mock, "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["ok"], json!(true), "plain login (no 2FA): {body}");
}

fn urlencoding(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

async fn proxy_get(c: &reqwest::Client, base: &str, target: &str) -> reqwest::StatusCode {
    let url = format!("{base}/api/image-proxy?url={}", urlencoding(target));
    c.get(url).send().await.unwrap().status()
}

#[tokio::test]
async fn nat64_and_6to4_smuggled_private_targets_are_refused_live() {
    let mock = spawn_mock().await;
    let addr = spawn_server().await;
    let base = format!("http://{addr}");
    let c = client();
    login(&c, &base, &mock).await;

    // Each of these embeds a forbidden IPv4 (127.0.0.1 / 169.254.169.254 / 10.0.0.1 /
    // 192.168.1.1) inside a NAT64 (64:ff9b::/96) or 6to4 (2002::/16) address. All must
    // be refused live (403) — the decode + re-check closes the smuggling path.
    for (target, what) in [
        ("http://[64:ff9b::7f00:1]/x.png", "NAT64 loopback 127.0.0.1"),
        (
            "http://[64:ff9b::a9fe:a9fe]/latest/meta-data/",
            "NAT64 metadata 169.254.169.254",
        ),
        ("http://[64:ff9b::a00:1]/x.png", "NAT64 private 10.0.0.1"),
        ("http://[2002:7f00:1::]/x.png", "6to4 loopback 127.0.0.1"),
        (
            "http://[2002:a9fe:a9fe::]/x",
            "6to4 metadata 169.254.169.254",
        ),
        (
            "http://[2002:c0a8:101::]/logo.png",
            "6to4 private 192.168.1.1",
        ),
    ] {
        let status = proxy_get(&c, &base, target).await;
        assert_eq!(
            status, 403,
            "SSRF: {what} ({target}) must be refused live (got {status})"
        );
    }

    // A PUBLIC v4 via NAT64 (8.8.8.8) passes the SSRF gate — it is NOT a 403. (It fails
    // later with a gateway error because this host has no NAT64 route, which is fine:
    // the point is that the gate permitted it rather than refusing it.)
    let public_status = proxy_get(&c, &base, "http://[64:ff9b::808:808]/x.png").await;
    assert_ne!(
        public_status, 403,
        "a public v4 via NAT64 (8.8.8.8) must NOT be refused by the SSRF gate (got {public_status})"
    );
}
