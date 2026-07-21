//! t18-e-e2e — SSRF Teredo / ISATAP embedded-IPv4 refusal (R4) LIVE through the real
//! image proxy.
//!
//! 26.17 (L3) decoded NAT64 (`64:ff9b::/96`) + 6to4 (`2002::/16`) embeddings. 26.18
//! (R4) extends the decode to two more transitional IPv6 forms that also carry a
//! routable IPv4 an attacker can point at an internal target:
//!   * **Teredo** `2001:0000::/32` — a plain *server* v4 (bits 32..64) AND an
//!     XOR-obfuscated mapped *client* v4 (bits 96..128); a private/metadata v4 in
//!     EITHER position is refused;
//!   * **ISATAP** `…:{0000,0200}:5efe:a.b.c.d` — v4 in the last 32 bits.
//!
//! `image_proxy.rs` unit-tests the decode (`blocks_teredo_and_isatap_embedded_private_ipv4`);
//! THIS leg drives the REAL `/api/image-proxy` route behind a real session and proves
//! the whole thing WIRED: a loopback/metadata/private v4 smuggled through Teredo or
//! ISATAP is refused live (403), while a Teredo/ISATAP address whose embedded v4(s) are
//! all public unicast passes the gate (NOT a 403).
//!
//! Run:
//!   cargo test -p mw-server --test t18_e2e_ssrf_teredo_isatap -- --nocapture --test-threads=1

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
    let base = std::env::temp_dir().join(format!("mw-t18-teredo-{}", unique()));
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
async fn teredo_and_isatap_smuggled_private_targets_are_refused_live() {
    let mock = spawn_mock().await;
    let addr = spawn_server().await;
    let base = format!("http://{addr}");
    let c = client();
    login(&c, &base, &mock).await;

    // Each of these embeds a forbidden IPv4 (127.0.0.1 / 169.254.169.254 / 10.0.0.1 /
    // 192.168.1.1) inside a Teredo (2001:0000::/32) or ISATAP (…:{0,200}:5efe:v4)
    // IPv6 address. All must be refused live (403) — the R4 decode + re-check closes
    // the smuggling path in BOTH embedded positions Teredo carries.
    for (target, what) in [
        (
            "http://[2001:0:4136:e378:8000:ffff:80ff:fffe]/x.png",
            "Teredo client v4 = 127.0.0.1 (obfuscated), public server",
        ),
        (
            "http://[2001:0:4136:e378:8000:ffff:5601:5601]/latest/meta-data/",
            "Teredo client v4 = 169.254.169.254 metadata",
        ),
        (
            "http://[2001:0:a00:1:8000:ffff:f7f7:f7f7]/x.png",
            "Teredo PRIVATE server v4 = 10.0.0.1 (public client)",
        ),
        (
            "http://[2001:470::5efe:7f00:1]/x.png",
            "ISATAP global-prefix IID wrapping 127.0.0.1",
        ),
        (
            "http://[2001:470:0:0:200:5efe:c0a8:101]/logo.png",
            "ISATAP (0x0200 flag) wrapping 192.168.1.1",
        ),
        (
            "http://[2001:470::5efe:a9fe:a9fe]/x",
            "ISATAP wrapping 169.254.169.254 metadata",
        ),
    ] {
        let status = proxy_get(&c, &base, target).await;
        assert_eq!(
            status, 403,
            "SSRF R4: {what} ({target}) must be refused live (got {status})"
        );
    }

    // A Teredo address whose BOTH embedded v4s are public (server 65.54.227.120 +
    // client 8.8.8.8), and a public-wrapping ISATAP (8.8.8.8), pass the SSRF gate — the
    // decode re-checks each v4, it does not blanket-refuse the transitional prefix. Not
    // a 403 (the fetch then fails later on transport grounds, which is fine).
    for (target, what) in [
        (
            "http://[2001:0:4136:e378:8000:ffff:f7f7:f7f7]/x.png",
            "Teredo, both embedded v4s public (server + 8.8.8.8)",
        ),
        (
            "http://[2001:470::5efe:808:808]/x.png",
            "ISATAP wrapping public 8.8.8.8",
        ),
    ] {
        let status = proxy_get(&c, &base, target).await;
        assert_ne!(
            status, 403,
            "a public-wrapping {what} ({target}) must NOT be refused by the SSRF gate (got {status})"
        );
    }
}
