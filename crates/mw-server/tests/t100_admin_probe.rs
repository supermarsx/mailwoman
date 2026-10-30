//! t100: admin-probe protection over the real router and real sockets.
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

const SLUG: &str = "/ops-t100-probe-k3x9q7";
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

#[tokio::test]
async fn the_grace_allowance_is_ignored_and_the_next_misses_block() {
    let f = fixture("t100-grace", &[]).await;
    let attacker = client(2);
    // Grace 3, then tarpit, then the 5th miss blocks (grace 3 + blockAfter 2).
    for (n, path) in ["/wp-admin", "/.env", "/phpmyadmin", "/wp-login.php"]
        .iter()
        .enumerate()
    {
        assert_eq!(
            status(&attacker, format!("{}{path}", f.base)).await,
            404,
            "miss {n}"
        );
    }
    let blocked = attacker
        .get(format!("{}/administrator", f.base))
        .send()
        .await
        .unwrap();
    assert_eq!(blocked.status(), 429);
    let retry: u64 = blocked.headers()["retry-after"]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!((100..=120).contains(&retry), "{retry}");
    // Now every path is refused, the mail app included.
    assert_eq!(status(&attacker, format!("{}/", f.base)).await, 429);
    assert_eq!(
        status(&attacker, format!("{}/api/public-config", f.base)).await,
        429
    );

    // A stale bookmark and one typo from another source never trigger anything.
    let user = client(3);
    assert_eq!(status(&user, format!("{}/admin", f.base)).await, 404);
    assert_eq!(status(&user, format!("{}/inbxo", f.base)).await, 404);
    for _ in 0..30 {
        assert_ne!(
            status(&user, format!("{}/api/public-config", f.base)).await,
            429
        );
    }

    // The block survives a restart (it is read back from the store at start).
    let restarted = app(&f.db, &f.upstream).await;
    let mut seen = 0;
    for _ in 0..50 {
        seen = status(&attacker, format!("{restarted}/")).await;
        if seen == 429 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert_eq!(seen, 429, "the block is restored after a restart");
    assert_ne!(
        status(&user, format!("{restarted}/api/public-config")).await,
        429
    );
}

#[tokio::test]
async fn slug_guesses_and_failed_admin_sign_ins_count_and_a_good_sign_in_clears() {
    let f = fixture("t100-clear", &[]).await;
    let admin = client(4);
    // 2 refused sign-ins + 2 slug guesses = 4: one short of a block.
    assert_eq!(sign_in(&admin, &f.base, "wrong").await, 401);
    assert_eq!(sign_in(&admin, &f.base, "wrong-again").await, 401);
    assert_eq!(
        status(&admin, format!("{}/ops-guess-one", f.base)).await,
        404
    );
    assert_eq!(
        status(&admin, format!("{}/ops-guess-two", f.base)).await,
        404
    );
    // The right password clears this source.
    assert_eq!(sign_in(&admin, &f.base, "hunter2").await, 200);
    for path in ["/wp-admin", "/ops-guess-3", "/ops-guess-4", "/.git/config"] {
        assert_eq!(
            status(&admin, format!("{}{path}", f.base)).await,
            404,
            "{path}"
        );
    }
    assert_ne!(
        status(&admin, format!("{}{SLUG}/session", f.base)).await,
        429
    );

    // Without the sign-in, the same mix blocks.
    let other = client(5);
    assert_eq!(sign_in(&other, &f.base, "wrong").await, 401);
    assert_eq!(sign_in(&other, &f.base, "wrong-again").await, 401);
    assert_eq!(
        status(&other, format!("{}/ops-guess-one", f.base)).await,
        404
    );
    assert_eq!(
        status(&other, format!("{}/ops-guess-two", f.base)).await,
        404
    );
    assert_eq!(
        status(&other, format!("{}/ops-guess-three", f.base)).await,
        429
    );
    // Static-file names a browser asks for by itself never count.
    let browser = client(6);
    for _ in 0..10 {
        assert_eq!(
            status(&browser, format!("{}/favicon.png", f.base)).await,
            404
        );
        assert_eq!(
            status(&browser, format!("{}/sitemap.xml", f.base)).await,
            404
        );
    }
    assert_ne!(status(&browser, format!("{}/", f.base)).await, 429);
}

#[tokio::test]
async fn admin_api_calls_without_a_session_count_once_per_cookie() {
    let f = fixture("t100-unauth", &[]).await;
    let stale = client(7);
    // An expired console tab polling with the same dead cookie: one point.
    for _ in 0..20 {
        let r = stale
            .get(format!("{}{SLUG}/users", f.base))
            .header("cookie", "mw_admin_session=expired-token")
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 401);
    }
    // Calls with no cookie at all count each time.
    let scanner = client(8);
    for _ in 0..4 {
        assert_eq!(
            status(&scanner, format!("{}{SLUG}/users", f.base)).await,
            401
        );
    }
    assert_eq!(
        status(&scanner, format!("{}{SLUG}/domains", f.base)).await,
        429
    );
}

#[tokio::test]
async fn allowlisted_networks_are_never_blocked() {
    let f = fixture(
        "t100-allow",
        &[("security.adminProbe.allowlist", json!(["127.0.0.9/32"]))],
    )
    .await;
    let office = client(9);
    for n in 0..40 {
        assert_eq!(
            status(&office, format!("{}/wp-admin/{n}", f.base)).await,
            404
        );
    }
    let outsider = client(10);
    for _ in 0..4 {
        status(&outsider, format!("{}/wp-admin", f.base)).await;
    }
    assert_eq!(status(&outsider, format!("{}/wp-admin", f.base)).await, 429);
}

#[tokio::test]
async fn forwarded_addresses_are_read_only_from_a_trusted_proxy() {
    let f = fixture("t100-xff", &[]).await;
    // An untrusted peer rotating X-Forwarded-For is still one source.
    let spoofer = client(11);
    let mut last = 0;
    for n in 0..5 {
        last = spoofer
            .get(format!("{}/wp-admin", f.base))
            .header("x-forwarded-for", format!("198.51.100.{n}"))
            .send()
            .await
            .unwrap()
            .status()
            .as_u16();
    }
    assert_eq!(last, 429);

    // Behind the trusted proxy, the forwarded client is the source.
    let proxy = client(PROXY);
    let via = |ip: &'static str| {
        let proxy = proxy.clone();
        let url = format!("{}/wp-admin", f.base);
        async move {
            proxy
                .get(url)
                .header("x-forwarded-for", ip)
                .send()
                .await
                .unwrap()
                .status()
                .as_u16()
        }
    };
    for _ in 0..4 {
        assert_eq!(via("203.0.113.1").await, 404);
    }
    assert_eq!(via("203.0.113.1").await, 429);
    assert_eq!(
        via("203.0.113.2").await,
        404,
        "another client behind the proxy"
    );
    // The proxy's own address is never counted (that would block everyone).
    for _ in 0..20 {
        assert_eq!(status(&proxy, format!("{}/wp-admin", f.base)).await, 404);
    }
}

fn comparable(r: &reqwest::Response) -> Vec<(String, String)> {
    let mut headers: Vec<(String, String)> = r
        .headers()
        .iter()
        .filter(|(k, _)| !matches!(k.as_str(), "date" | "retry-after" | "x-request-id"))
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect();
    headers.sort();
    headers
}

#[tokio::test]
async fn the_blocked_answer_is_the_same_everywhere_and_never_names_the_console() {
    let f = fixture("t100-same", &[]).await;
    let attacker = client(12);
    let mut seen = Vec::new();
    for path in ["/wp-admin", "/admin", "/ops-a", "/ops-b", "/ops-c"] {
        let r = attacker
            .get(format!("{}{path}", f.base))
            .send()
            .await
            .unwrap();
        let headers = format!("{:?}", r.headers());
        seen.push(format!("{headers}{}", r.text().await.unwrap()));
    }
    let mut answers = Vec::new();
    for (method, path) in [
        ("GET", "/".to_string()),
        ("GET", "/wp-admin".to_string()),
        ("GET", "/no-such-page".to_string()),
        ("GET", SLUG.to_string()),
        ("GET", format!("{SLUG}/users")),
        ("POST", format!("{SLUG}/login")),
        ("GET", "/api/me".to_string()),
        ("DELETE", "/api/anything".to_string()),
    ] {
        let mut req = attacker.request(method.parse().unwrap(), format!("{}{path}", f.base));
        if method == "POST" {
            req = req.json(&json!({"username":"root","password":"hunter2"}));
        }
        let r = req.send().await.unwrap();
        let status = r.status().as_u16();
        let headers = comparable(&r);
        let retry: u64 = r.headers()["retry-after"]
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        let raw = format!("{:?}", r.headers());
        let body = r.text().await.unwrap();
        seen.push(format!("{raw}{body}"));
        answers.push((path, status, headers, body, retry));
    }
    let (_, status0, headers0, body0, retry0) = answers[0].clone();
    assert_eq!(status0, 429);
    for (path, status, headers, body, retry) in &answers {
        assert_eq!(*status, status0, "{path}");
        assert_eq!(*headers, headers0, "{path}");
        assert_eq!(*body, body0, "{path}");
        assert!(retry.abs_diff(retry0) <= 1, "{path}");
    }
    // Not even the right password at the real console gets through, and nothing
    // the attacker received names the console path.
    let needle = SLUG.trim_start_matches('/');
    for text in &seen {
        assert!(!text.contains(needle), "{text}");
    }
    assert!(!body0.contains("admin"), "{body0}");
}

#[tokio::test]
async fn unblocking_needs_step_up_and_is_audited() {
    let f = fixture("t100-unblock", &[]).await;
    let attacker = client(13);
    for _ in 0..5 {
        status(&attacker, format!("{}/wp-admin", f.base)).await;
    }
    assert_eq!(status(&attacker, format!("{}/", f.base)).await, 429);

    let admin = client(14);
    assert_eq!(sign_in(&admin, &f.base, "hunter2").await, 200);
    let overview: Value = admin
        .get(format!("{}{SLUG}/security/admin-probe", f.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(overview["stats"]["activeBlocks"], 1, "{overview}");
    assert_eq!(overview["stats"]["blocksTotal"], 1, "{overview}");
    assert_eq!(overview["policy"]["graceAllowance"], 3);
    let offender = &overview["topOffenders"][0];
    assert_eq!(offender["source"], "127.0.0.0/24", "masked by default");
    assert_eq!(offender["blocked"], true);
    let id = offender["id"].as_str().unwrap().to_string();
    let blocks: Value = admin
        .get(format!("{}{SLUG}/security/admin-probe/blocks", f.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(blocks["blocks"][0]["id"], id.as_str());

    // No step-up yet: refused.
    let unblock = format!("{}{SLUG}/security/admin-probe/blocks/{id}", f.base);
    let refused = admin.delete(&unblock).send().await.unwrap();
    assert_eq!(refused.status(), 403);
    let body: Value = refused.json().await.unwrap();
    assert_eq!(body["code"], "step_up_required", "{body}");
    assert_eq!(body["action"], "banManage");
    assert_eq!(status(&attacker, format!("{}/", f.base)).await, 429);

    assert_eq!(
        admin
            .post(format!("{}{SLUG}/step-up", f.base))
            .json(&json!({"password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(admin.delete(&unblock).send().await.unwrap().status(), 204);
    assert_eq!(admin.delete(&unblock).send().await.unwrap().status(), 404);
    assert_ne!(status(&attacker, format!("{}/", f.base)).await, 429);

    // The policy route: step-up held, If-Match honoured, audited.
    let read = admin
        .get(format!("{}{SLUG}/security/admin-probe", f.base))
        .send()
        .await
        .unwrap();
    let etag = read.headers()["etag"].to_str().unwrap().to_string();
    let saved = admin
        .put(format!("{}{SLUG}/security/admin-probe/policy", f.base))
        .header("if-match", "\"stale\"")
        .json(&json!({"values":{"graceAllowance":20}}))
        .send()
        .await
        .unwrap();
    assert_eq!(saved.status(), 412);
    let saved = admin
        .put(format!("{}{SLUG}/security/admin-probe/policy", f.base))
        .header("if-match", &etag)
        .json(&json!({"values":{"graceAllowance":20,"allowlist":["192.0.2.0/24"]}}))
        .send()
        .await
        .unwrap();
    assert_eq!(saved.status(), 200);
    let saved: Value = saved.json().await.unwrap();
    assert_eq!(saved["policy"]["graceAllowance"], 20);
    assert_eq!(saved["policy"]["allowlist"], json!(["192.0.2.0/24"]));
    let invalid = admin
        .put(format!("{}{SLUG}/security/admin-probe/policy", f.base))
        .json(&json!({"values":{"probePaths":["no-slash"]}}))
        .send()
        .await
        .unwrap();
    assert_eq!(invalid.status(), 422);
    // The generic settings route sends writers to the dedicated one.
    let generic = admin
        .delete(format!(
            "{}{SLUG}/settings/security.adminProbe.graceAllowance",
            f.base
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(generic.status(), 403);
    let generic: Value = generic.json().await.unwrap();
    assert_eq!(generic["code"], "useDedicatedRoute", "{generic}");

    let rows = Store::open(&f.db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap()
        .list_audit(100)
        .await
        .unwrap();
    let blocked = rows
        .iter()
        .find(|r| r.action == "admin-probe-blocked")
        .expect("block audited");
    assert!(blocked.detail_json.contains("127.0.0.0/24"), "{blocked:?}");
    assert!(!blocked.detail_json.contains("127.0.0.13"), "{blocked:?}");
    assert_eq!(blocked.ip, None, "masked: no full address on the row");
    assert!(
        rows.iter()
            .any(|r| r.action == "admin-probe-unblocked" && r.actor == "root"),
        "{rows:#?}"
    );
    assert!(
        rows.iter()
            .any(|r| r.action == "admin-probe-policy-changed" && r.actor == "root"),
        "{rows:#?}"
    );
}

/// The live scenario: one client hammers `/wp-admin` and is blocked; a second
/// client signs in to the console and works normally the whole time.
#[tokio::test]
async fn a_scanner_is_blocked_while_the_admin_keeps_working() {
    let f = fixture(
        "t100-live",
        &[
            ("security.adminProbe.graceAllowance", json!(10)),
            ("security.adminProbe.blockAfter", json!(10)),
        ],
    )
    .await;
    let scanner = client(15);
    let admin = client(16);
    assert_eq!(sign_in(&admin, &f.base, "hunter2").await, 200);
    let mut first_block = None;
    for n in 0..60 {
        let code = status(&scanner, format!("{}/wp-admin", f.base)).await;
        if code == 429 && first_block.is_none() {
            first_block = Some(n);
        }
        if n % 10 == 0 {
            assert_eq!(
                status(&admin, format!("{}{SLUG}/session", f.base)).await,
                200,
                "the admin is unaffected at request {n}"
            );
        }
    }
    assert_eq!(first_block, Some(19), "grace 10 + 10 more");
    assert_eq!(status(&scanner, format!("{}{SLUG}", f.base)).await, 429);
    let fresh = client(17);
    assert_eq!(sign_in(&fresh, &f.base, "hunter2").await, 200);
    assert_eq!(
        status(&fresh, format!("{}{SLUG}/session", f.base)).await,
        200
    );
}
