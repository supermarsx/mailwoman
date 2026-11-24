//! A probe-path hit blocks its source address, but (t45 B1) only when it is
//! evidence about that source: a browser request made for another site is not
//! recorded, and a verified mailbox or administrator session is neither recorded
//! nor refused by an active probe block. Anonymous requests stay blocked.
//! Runs once with synchronous accounting and once (in a child process) with
//! `MW_REQUEST_PROTECTION_ACCOUNTING=write-behind`.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{ServerKey, Store, request_protection_model::Signal, settings_policy::SettingPolicyRow};
use serde_json::json;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
mod common;

async fn serve(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    });
    format!("http://{address}")
}
fn builder(ip: u8) -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .local_address(IpAddr::V4(Ipv4Addr::new(127, 0, 0, ip)))
}
fn client(ip: u8) -> reqwest::Client {
    builder(ip).cookie_store(true).build().unwrap()
}
async fn fixture() -> (String, Store) {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("probe-block");
    let key = "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";
    let store = Store::open(db.to_str().unwrap(), ServerKey::from_hex(key).unwrap())
        .await
        .unwrap();
    for (key, value) in [
        ("signin.retryCooldownSecs", json!(0)),
        ("security.adminProbe.enabled", json!(false)),
        ("security.protection.signals", json!([])),
        ("security.protection.blockSecs", json!(120)),
    ] {
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
    let (router, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(key.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(upstream.clone()),
                jmap_upstreams: Some(vec![upstream]),
                ..Default::default()
            },
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("hunter2".into()),
            redis_url: None,
        },
    )
    .await
    .unwrap();
    (serve(router).await, store)
}
/// Signs in without a cookie jar and returns the `Cookie` header value.
async fn mailbox_cookie(base: &str, ip: u8) -> String {
    let response = builder(ip)
        .build()
        .unwrap()
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let cookies: Vec<String> = response
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok()?.split(';').next().map(str::to_owned))
        .filter(|pair| pair.split_once('=').is_some_and(|(_, value)| !value.is_empty()))
        .collect();
    assert!(!cookies.is_empty());
    cookies.join("; ")
}
async fn probe_rows(store: &Store) -> usize {
    store
        .request_protection_rows(chrono::Utc::now().timestamp(), None, 100)
        .await
        .unwrap()
        .iter()
        .filter(|row| row.snapshot.last_signal == Some(Signal::ProbePath))
        .count()
}
async fn status(request: reqwest::RequestBuilder) -> u16 {
    request.send().await.unwrap().status().as_u16()
}

async fn probe_block_rules() {
    let (base, store) = fixture().await;
    // 1. A browser request for another site's page is not recorded.
    let browser = client(60);
    let unblocked = status(browser.get(format!("{base}/.environment"))).await;
    assert_ne!(unblocked, 429);
    for site in ["cross-site", "same-site", "Cross-Site"] {
        let hit = status(browser.get(format!("{base}/.env")).header("sec-fetch-site", site)).await;
        assert_eq!(hit, unblocked, "{site}: response unchanged, no block");
    }
    assert_eq!(probe_rows(&store).await, 0, "no probe-path subject created");
    assert_eq!(status(browser.get(format!("{base}/api/me"))).await, 401);
    // A typed URL and a request without the header (scanners) still count.
    assert_eq!(status(client(61).get(format!("{base}/.env")).header("sec-fetch-site", "none")).await, 429);
    assert_eq!(probe_rows(&store).await, 1);
    let scanner = client(62);
    assert_eq!(status(scanner.get(format!("{base}/.env"))).await, 429);
    assert_eq!(status(scanner.get(format!("{base}/api/me"))).await, 429, "source blocked");
    assert_eq!(probe_rows(&store).await, 2);
    // 2. A verified mailbox session passes an active probe block; anonymous
    // and forged-cookie requests from the same address stay blocked.
    let cookie = mailbox_cookie(&base, 63).await;
    let anonymous = client(63);
    assert_eq!(status(anonymous.get(format!("{base}/cmd_sco"))).await, 429);
    let user = builder(63).build().unwrap();
    assert_eq!(status(user.get(format!("{base}/api/me")).header("cookie", &cookie)).await, 200);
    assert_eq!(status(anonymous.get(format!("{base}/api/me"))).await, 429);
    let forged: String = cookie
        .split("; ")
        .map(|pair| {
            let (name, value) = pair.split_once('=').unwrap();
            let last = if value.ends_with('x') { 'y' } else { 'x' };
            format!("{name}={}{last}", &value[..value.len() - 1])
        })
        .collect::<Vec<_>>()
        .join("; ");
    assert_eq!(status(user.get(format!("{base}/api/me")).header("cookie", forged)).await, 429);
    // 3. A signed-in user's own probe-path request is not recorded.
    let rows = probe_rows(&store).await;
    let cookie = mailbox_cookie(&base, 64).await;
    let user = builder(64).build().unwrap();
    assert_ne!(status(user.get(format!("{base}/.git/config")).header("cookie", &cookie)).await, 429);
    assert_eq!(probe_rows(&store).await, rows);
    assert_eq!(status(client(64).get(format!("{base}/api/me"))).await, 401, "no block recorded");
    // 4. An administrator mistyping a console URL ending in a catalogued
    // segment does not block their own address.
    let admin = client(65);
    let login = admin
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":"root","password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200);
    assert_ne!(status(admin.get(format!("{base}/admin/security/console"))).await, 429);
    assert_eq!(probe_rows(&store).await, rows);
    assert_eq!(status(client(65).get(format!("{base}/api/me"))).await, 401);
    // The same console path without a session is still a probe.
    assert_eq!(status(client(66).get(format!("{base}/admin/security/console"))).await, 429);
}

#[tokio::test]
async fn probe_blocks_skip_cross_site_browser_requests_and_verified_sessions() {
    probe_block_rules().await;
}

#[tokio::test]
async fn probe_block_rules_hold_with_write_behind_accounting() {
    if std::env::var_os("MW_T45_PROBE_CHILD").is_some() {
        return;
    }
    // Accounting mode is read once at startup from the process environment, so
    // the write-behind run is a separate process.
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "probe_blocks_skip_cross_site_browser_requests_and_verified_sessions",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("MW_T45_PROBE_CHILD", "1")
        .env("MW_REQUEST_PROTECTION_ACCOUNTING", "write-behind")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{text}\n{}", String::from_utf8_lossy(&output.stderr));
    assert!(text.contains("1 passed"), "{text}");
}
