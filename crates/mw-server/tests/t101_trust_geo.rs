//! Unknown-country admin discovery block (security.trust.adminGeo*) over the real router.
//!
//! Each test builds its own server (its own probe table) on one SQLite file and
//! talks to it from distinct loopback source addresses (`127.0.0.N`). The policy
//! is tightened so the thresholds are small: grace 3, block after 2 more, no
//! tarpit delay. The decay and backoff arithmetic is covered with a mock clock in
//! `src/admin_probe/tracker.rs`.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
mod common;

const SLUG: &str = "/ops-t101-probe-k3x9q7";
const KEY: &str = "6a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";
const PROXY: u8 = 50;

fn env() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        // Same values for every test in this binary.
        std::env::set_var("MW_ADMIN_PATH", SLUG);
        std::env::set_var("MW_FORWARDED_MODE", "xff");
        std::env::set_var("MW_TRUSTED_PROXIES", format!("127.0.0.{PROXY}"));
    });
}

async fn serve(router: axum::Router) -> String {
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
    format!("http://{address}")
}

fn client(ip: u8) -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .local_address(IpAddr::V4(Ipv4Addr::new(127, 0, 0, ip)))
        .build()
        .unwrap()
}

/// A public source (192.0.2.N, TEST-NET-1) reached through the trusted proxy
/// (127.0.0.50, `X-Forwarded-For`). A bare loopback client is a local network and
/// is never counted by the country rule.
fn via(n: u8) -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("x-forwarded-for", format!("192.0.2.{n}").parse().unwrap());
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .local_address(IpAddr::V4(Ipv4Addr::new(127, 0, 0, PROXY)))
        .default_headers(headers)
        .build()
        .unwrap()
}

struct Fixture {
    base: String,
    db: String,
    upstream: String,
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

async fn app(db: &str, upstream: &str) -> String {
    let (router, _) = build_app_full(
        AppConfig {
            db_path: db.into(),
            server_key_hex: Some(KEY.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(upstream.into()),
                jmap_upstreams: Some(vec![upstream.into()]),
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
    serve(router).await
}

async fn fixture(name: &str, extra: &[(&str, Value)]) -> Fixture {
    env();
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path(name)
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    for (key, value) in [
        ("signin.retryCooldownSecs", json!(0)),
        ("security.adminProbe.graceAllowance", json!(3)),
        ("security.adminProbe.blockAfter", json!(2)),
        ("security.adminProbe.tarpitDelayMs", json!(0)),
        ("security.adminProbe.blockSecs", json!(120)),
    ]
    .into_iter()
    .chain(extra.iter().cloned())
    {
        put(&store, key, value).await;
    }
    let base = app(&db, &upstream).await;
    Fixture { base, db, upstream }
}

async fn status(c: &reqwest::Client, url: String) -> u16 {
    c.get(url).send().await.unwrap().status().as_u16()
}

async fn sign_in(c: &reqwest::Client, base: &str, password: &str) -> u16 {
    c.post(format!("{base}{SLUG}/login"))
        .json(&json!({"username":"root","password":password}))
        .send()
        .await
        .unwrap()
        .status()
        .as_u16()
}


fn geo() -> Vec<(&'static str, Value)> {
    vec![
        ("security.trust.adminGeoLookupFailure", json!("unknown")),
        ("security.adminProbe.graceAllowance", json!(50)),
        ("security.adminProbe.blockAfter", json!(50)),
        // Isolate the geo rule from the adaptive per-address probe handling.
        ("security.protection.rateMode", json!("off")),
        ("security.protection.adaptiveMode", json!("off")),
    ]
}

#[tokio::test]
async fn probes_from_an_unknown_country_block_at_the_threshold() {
    // No country database in the test environment: "unknown" makes a source
    // without a country an unknown country, so only the geo rule can block here
    // (the score thresholds are 50).
    let f = fixture("t101-geo-block", &geo()).await;
    let c = via(21);
    assert_eq!(status(&c, format!("{}/wp-admin", f.base)).await, 404);
    assert_eq!(status(&c, format!("{}/.env", f.base)).await, 404);
    assert_eq!(status(&c, format!("{}/.git/config", f.base)).await, 429, "third probe blocks");
    assert_eq!(status(&c, format!("{}/inbox", f.base)).await, 429, "every path is answered the same");
    assert_ne!(status(&c, format!("{}/healthz", f.base)).await, 429, "liveness stays reachable");
    assert_ne!(status(&c, format!("{}/readyz", f.base)).await, 429, "readiness stays reachable");
    // Another source is unaffected.
    assert_eq!(status(&via(22), format!("{}/wp-admin", f.base)).await, 404);
}

#[tokio::test]
async fn the_default_lookup_failure_policy_never_blocks_without_a_country() {
    let f = fixture(
        "t101-geo-allow",
        &[
            ("security.adminProbe.graceAllowance", json!(50)),
            ("security.adminProbe.blockAfter", json!(50)),
            ("security.protection.rateMode", json!("off")),
            ("security.protection.adaptiveMode", json!("off")),
        ],
    )
    .await;
    let c = via(23);
    // Exactly the geo threshold: the same three probes block in the test above.
    for _ in 0..3 {
        assert_eq!(status(&c, format!("{}/wp-admin", f.base)).await, 404);
    }
}

#[tokio::test]
async fn the_geo_rule_can_be_switched_off_and_block_on_first_is_opt_in() {
    let mut off = geo();
    off.push(("security.trust.adminGeoEnabled", json!(false)));
    let f = fixture("t101-geo-off", &off).await;
    let c = via(24);
    for _ in 0..3 {
        assert_eq!(status(&c, format!("{}/wp-admin", f.base)).await, 404);
    }
    let mut first = geo();
    first.push(("security.trust.adminGeoBlockOnFirst", json!(true)));
    let f = fixture("t101-geo-first", &first).await;
    let c = via(25);
    assert_eq!(status(&c, format!("{}/wp-admin", f.base)).await, 429);
}

#[tokio::test]
async fn a_forwarding_header_from_an_untrusted_peer_cannot_move_the_block() {
    let f = fixture("t101-geo-spoof", &geo()).await;
    // An untrusted peer claims 192.0.2.77 in X-Forwarded-For. The header is ignored.
    let c = client(26);
    for _ in 0..3 {
        c.get(format!("{}/wp-admin", f.base))
            .header("x-forwarded-for", "192.0.2.77")
            .send()
            .await
            .unwrap();
    }
    // Nothing landed on the claimed address (the trusted proxy now speaks for it)...
    assert_ne!(status(&via(77), format!("{}/inbox", f.base)).await, 429);
    // ...and the real peer is a local network, which the country rule never counts.
    assert_ne!(status(&c, format!("{}/inbox", f.base)).await, 429);
}

#[tokio::test]
async fn a_local_network_source_is_never_counted_as_an_unknown_country() {
    let f = fixture("t101-geo-local", &geo()).await;
    let c = client(28);
    for _ in 0..8 {
        assert_eq!(status(&c, format!("{}/wp-admin", f.base)).await, 404);
    }
    assert_ne!(status(&c, format!("{}/inbox", f.base)).await, 429);
}

async fn with_agent(base: &str, ip: u8, path: &str, agent: &str) -> u16 {
    client(ip)
        .get(format!("{base}{path}"))
        .header("user-agent", agent)
        .send()
        .await
        .unwrap()
        .status()
        .as_u16()
}

#[tokio::test]
async fn declared_ai_crawlers_are_blocked_when_anonymous_and_the_action_is_admin_selectable() {
    let f = fixture("t101-ua-class", &[("security.protection.userAgentMode", json!("enforce"))]).await;
    for (ip, agent) in [
        (31, "GPTBot/1.2"),
        (32, "Mozilla/5.0 (compatible; ClaudeBot/1.0)"),
        (33, "PerplexityBot/1.0"),
        (34, "CCBot/2.0"),
        (35, "Bytespider"),
        (36, "Amazonbot/0.1"),
        (37, "Google-Extended"),
        (38, "Applebot-Extended"),
        (39, "ChatGPT-User/1.0"),
    ] {
        assert_eq!(with_agent(&f.base, ip, "/api/me", agent).await, 403, "{agent}");
        assert_eq!(with_agent(&f.base, ip, "/", agent).await, 403, "{agent} document");
    }
    // An ordinary browser, an empty header and a script are not in a blocked class.
    let browser = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/140.0 Safari/537.36";
    assert_ne!(with_agent(&f.base, 40, "/api/me", browser).await, 403);
    assert_ne!(with_agent(&f.base, 41, "/api/me", "curl/8.0").await, 403);
    // Probes of liveness are never subject to the class action.
    assert_ne!(with_agent(&f.base, 42, "/healthz", "GPTBot/1.2").await, 403);
}

#[tokio::test]
async fn setting_a_class_to_allow_or_limit_lifts_the_block() {
    let f = fixture(
        "t101-ua-allow",
        &[
            ("security.trust.uaAiTraining", json!("allow")),
            ("security.trust.uaAiSearch", json!("limit")),
        ],
    )
    .await;
    assert_ne!(with_agent(&f.base, 43, "/api/me", "GPTBot/1.2").await, 403);
    assert_ne!(with_agent(&f.base, 44, "/api/me", "PerplexityBot/1.0").await, 403);
    // The user-triggered class is still on its default and still blocked.
    assert_eq!(with_agent(&f.base, 45, "/api/me", "Claude-User/1.0").await, 403);
}
