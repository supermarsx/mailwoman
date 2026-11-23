//! `/readyz` answers 200, and 503 when a data filesystem is below
//! `health.diskFreeMinPercent` (t48). Driven through a real listener.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
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

/// A server with the given settings; returns `host:port`.
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

async fn get_ready(addr: SocketAddr) -> (u16, String) {
    let response = client(1)
        .get(format!("http://{addr}/readyz"))
        .send()
        .await
        .unwrap();
    (response.status().as_u16(), response.text().await.unwrap())
}

#[tokio::test]
async fn readyz_is_ok_with_the_default_floor() {
    let addr = server("readyz-default", &[]).await;
    // A test host with under 5 percent free would make this 503 by design.
    let (status, body) = get_ready(addr).await;
    if status == 503 {
        assert!(body.contains("below the floor"), "{body}");
    } else {
        assert_eq!((status, body.as_str()), (200, "ok"));
    }
}

#[tokio::test]
async fn readyz_is_503_when_the_floor_is_above_the_free_space() {
    // Stored directly: the console limits the value to 50, a stored 100 is the
    // only floor a test host is certain to be below.
    let addr = server("readyz-floor", &[("health.diskFreeMinPercent", json!(100))]).await;
    let (status, body) = get_ready(addr).await;
    assert_eq!(status, 503, "{body}");
    assert!(body.contains("data"), "{body}");
}

#[tokio::test]
async fn readyz_follows_healthz_access_rules() {
    let addr = server("readyz-access", &[]).await;
    let want = {
        let r = client(PROXY)
            .get(format!("http://{addr}/no-such-path-q7"))
            .header("x-forwarded-for", "8.8.8.8")
            .send()
            .await
            .unwrap();
        (r.status().as_u16(), r.text().await.unwrap())
    };
    let r = client(PROXY)
        .get(format!("http://{addr}/readyz"))
        .header("x-forwarded-for", "8.8.8.8")
        .send()
        .await
        .unwrap();
    assert_eq!((r.status().as_u16(), r.text().await.unwrap()), want);
}
