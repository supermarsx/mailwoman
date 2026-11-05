//! `/healthz` is internal-only by default (`health_access.rs`), driven through a
//! real listener with `ConnectInfo`, the way `main.rs` serves.
//!
//! Client addresses: the server trusts `127.0.0.50` as a reverse proxy
//! (`MW_FORWARDED_MODE=xff`), so a request from that address carrying
//! `X-Forwarded-For: 8.8.8.8` is a public client behind a trusted proxy.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
mod common;

const KEY: &str = "6a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";
const TOKEN: &str = "monitor-token-k3x9q7";
const PROXY: u8 = 50;

fn env() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        std::env::set_var("MW_FORWARDED_MODE", "xff");
        std::env::set_var("MW_TRUSTED_PROXIES", format!("127.0.0.{PROXY}"));
        std::env::set_var("MW_HEALTH_TOKEN", TOKEN);
    });
}

fn client(ip: u8) -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .local_address(IpAddr::V4(Ipv4Addr::new(127, 0, 0, ip)))
        .build()
        .unwrap()
}

async fn put(store: &Store, key: &str, value: Value) {
    store
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "fixture".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}

/// A server with the given `security.health.*` settings; returns `host:port`.
async fn server(name: &str, settings: &[(&str, Value)]) -> SocketAddr {
    env();
    let db = common::test_db::unique_db_path(name)
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    for (key, value) in settings {
        put(&store, key, value.clone()).await;
    }
    drop(store);
    let (router, _) = build_app_full(
        AppConfig {
            db_path: db,
            server_key_hex: Some(KEY.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
        },
        V6Config {
            admin_enabled: false,
            admin_username: None,
            admin_password: None,
            redis_url: None,
        },
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    address
}

/// GET `path` from the trusted proxy address, naming `client_ip` as the client.
async fn via_proxy(addr: SocketAddr, path: &str, client_ip: &str) -> (u16, String) {
    let response = client(PROXY)
        .get(format!("http://{addr}{path}"))
        .header("x-forwarded-for", client_ip)
        .send()
        .await
        .unwrap();
    (response.status().as_u16(), response.text().await.unwrap())
}

async fn unknown(addr: SocketAddr, client_ip: &str) -> (u16, String) {
    via_proxy(addr, "/no-such-path-q7", client_ip).await
}

#[tokio::test]
async fn loopback_is_allowed() {
    let addr = server("health-loopback", &[]).await;
    let response = client(1)
        .get(format!("http://{addr}/healthz"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "ok");
}

#[tokio::test]
async fn private_client_behind_the_trusted_proxy_is_allowed() {
    let addr = server("health-private", &[]).await;
    for ip in ["10.1.2.3", "192.168.0.9", "fd00::7", "fe80::1"] {
        assert_eq!(via_proxy(addr, "/healthz", ip).await.0, 200, "{ip}");
    }
}

#[tokio::test]
async fn public_client_behind_the_trusted_proxy_gets_the_unknown_path_404() {
    let addr = server("health-public", &[]).await;
    let want = unknown(addr, "8.8.8.8").await;
    assert_eq!(want.0, 404);
    for path in ["/healthz", "/readyz", "/healthz?x=1"] {
        assert_eq!(via_proxy(addr, path, "8.8.8.8").await, want, "{path}");
    }
}

#[tokio::test]
async fn token_allows_a_public_client() {
    let addr = server("health-token", &[]).await;
    let c = client(PROXY);
    let get = |path: String, header: Option<&'static str>, value: String| {
        let c = c.clone();
        async move {
            let mut r = c
                .get(format!("http://{addr}{path}"))
                .header("x-forwarded-for", "8.8.8.8");
            if let Some(h) = header {
                r = r.header(h, value);
            }
            r.send().await.unwrap().status().as_u16()
        }
    };
    assert_eq!(get("/healthz".into(), Some("x-health-token"), TOKEN.into()).await, 200);
    assert_eq!(get(format!("/healthz?token={TOKEN}"), None, String::new()).await, 200);
    assert_eq!(get("/healthz".into(), Some("x-health-token"), "wrong".into()).await, 404);
    assert_eq!(get("/healthz?token=wrong".into(), None, String::new()).await, 404);
}

#[tokio::test]
async fn allowlist_admits_only_listed_public_networks() {
    let addr = server(
        "health-allowlist",
        &[
            ("security.health.mode", json!("allowlist")),
            ("security.health.networks", json!(["203.0.113.0/24"])),
        ],
    )
    .await;
    assert_eq!(via_proxy(addr, "/healthz", "203.0.113.9").await.0, 200);
    assert_eq!(via_proxy(addr, "/healthz", "198.51.100.1").await.0, 404);
    assert_eq!(via_proxy(addr, "/healthz", "10.0.0.1").await.0, 200);
}

#[tokio::test]
async fn networks_are_ignored_in_internal_only_mode_and_open_admits_everyone() {
    let listed = [("security.health.networks", json!(["203.0.113.0/24"]))];
    let addr = server("health-netsonly", &listed).await;
    assert_eq!(via_proxy(addr, "/healthz", "203.0.113.9").await.0, 404);
    let addr = server("health-open", &[("security.health.mode", json!("open"))]).await;
    assert_eq!(via_proxy(addr, "/healthz", "8.8.8.8").await.0, 200);
}

/// Send a raw request whose Host header the protocol guard enforces against.
/// An admitted probe is exempt from that enforcement; a refused one must not be.
async fn raw_bad_host(addr: SocketAddr, from: u8, client_ip: &str) -> String {
    let socket = tokio::net::TcpSocket::new_v4().unwrap();
    socket
        .bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, from)), 0))
        .unwrap();
    let mut stream = socket.connect(addr).await.unwrap();
    let request = format!(
        "GET /healthz HTTP/1.1\r\nHost: bad host\r\nX-Forwarded-For: {client_ip}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut out = Vec::new();
    stream.read_to_end(&mut out).await.unwrap();
    String::from_utf8_lossy(&out).into_owned()
}

#[tokio::test]
async fn a_refused_probe_gets_no_protection_exemption() {
    let addr = server("health-bypass", &[]).await;
    // Control: an admitted probe (internal client) keeps the exemption.
    let admitted = raw_bad_host(addr, PROXY, "10.0.0.5").await;
    assert!(admitted.starts_with("HTTP/1.1 200"), "{admitted}");
    // A public client naming /healthz is held to the guard like any other path.
    let refused = raw_bad_host(addr, PROXY, "8.8.8.8").await;
    assert!(refused.starts_with("HTTP/1.1 400"), "{refused}");
}
