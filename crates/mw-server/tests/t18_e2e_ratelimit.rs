//! t18-e-e2e — image-proxy per-account rate limit (R6) LIVE through `/api/image-proxy`.
//!
//! 26.18 (R6) adds a coarse in-memory per-account token-bucket to the image-proxy
//! fetch path (burst 120, refill ~1/s): one account cannot drive unbounded distinct
//! upstream fetches. It is charged on a cache MISS AFTER the session check, so a cache
//! hit is free and a normal reader under the threshold is never limited. `image_proxy.rs`
//! unit-tests the bucket (`rate_limiter_allows_burst_then_429s_and_is_per_account`);
//! THIS leg proves the whole thing WIRED over real HTTP:
//!   * one account hammering the proxy is admitted for a real burst, then gets `429`;
//!   * a SECOND account (independent bucket) is NOT limited by the first's spend — so a
//!     normal reader under the threshold is fine and the limit is genuinely per-account.
//!
//! We drive the fetch with a target the SSRF gate refuses (`127.0.0.1`) so no real
//! upstream is needed: the rate check runs BEFORE the fetch, so each request still
//! charges a token and returns `403` until the bucket is drained, at which point the
//! request short-circuits to `429` before the fetch. (A blocked target is never cached,
//! so every request is a genuine cache MISS that charges — exactly the fetch budget R6
//! caps.)
//!
//! Cache-hit-is-free is a SOURCE-ORDERING property (in `proxy_image` the cache `get`
//! returns before `rate_limiter().check()`), unit-covered; it cannot be driven live
//! here because the only reachable local origin (loopback) is refused by the SSRF gate,
//! so no cacheable success is reachable without an external public image host. Noted,
//! not silently skipped.
//!
//! Run:
//!   cargo test -p mw-server --test t18_e2e_ratelimit -- --nocapture --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, build_app};
use mw_store::{Credentials, ServerKey, Store};

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

fn temp_db() -> String {
    let dir = std::env::temp_dir().join(format!("mw-t18-ratelimit-{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("mw.db").to_string_lossy().into_owned()
}

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

async fn spawn_server(db_path: &str) -> SocketAddr {
    let web = PathBuf::from(db_path)
        .parent()
        .unwrap()
        .join(format!("web-{}", unique()));
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: db_path.to_string(),
        // FIXED key so a separately-opened store seals session B under the same key.
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    };
    let app = build_app(config).await.expect("build_app");
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

async fn login_cookie(c: &reqwest::Client, base: &str, mock: &str) {
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

/// Seed a real session for a DISTINCT account (its own rate-limit bucket).
async fn seed_session(db_path: &str, account_id: &str) -> String {
    let store = Store::open(db_path, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    store
        .create_session(
            account_id,
            "b@example.org",
            "http://upstream.invalid",
            "http://upstream.invalid",
            &Credentials {
                username: "b@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap()
}

/// One `/api/image-proxy` GET for a blocked (SSRF-refused) target, with an explicit
/// session cookie (so we can drive two distinct accounts).
async fn proxy_get(c: &reqwest::Client, base: &str, cookie: &str) -> u16 {
    c.get(format!(
        "{base}/api/image-proxy?url=http%3A%2F%2F127.0.0.1%2Fx.png"
    ))
    .header("Cookie", format!("mw_session={cookie}"))
    .send()
    .await
    .unwrap()
    .status()
    .as_u16()
}

#[tokio::test]
async fn image_proxy_rate_limits_per_account_with_429() {
    let db = temp_db();
    let mock = spawn_mock().await;
    let addr = spawn_server(&db).await;
    let base = format!("http://{addr}");

    // Account A: log in through the mock (cookie stored in its jar). Recover the raw
    // cookie value so we can send it explicitly alongside account B.
    let ca = client();
    login_cookie(&ca, &base, &mock).await;
    let cookie_a = ca
        .get(format!(
            "{base}/api/image-proxy?url=http%3A%2F%2F127.0.0.1%2Fx.png"
        ))
        .send()
        .await
        .unwrap();
    // First request for account A must NOT already be 429 (a fresh bucket admits it;
    // the target is SSRF-refused so it is a 403).
    assert_ne!(
        cookie_a.status().as_u16(),
        429,
        "account A's very first proxy request must be under the threshold, not 429"
    );
    // Pull A's session cookie out of the jar to reuse it explicitly.
    let a_cookie = login_raw_cookie(&base, &mock).await;

    // Account B: a DISTINCT account with its own bucket.
    let b_cookie = seed_session(&db, "acct-ratelimit-b").await;

    // Hammer the proxy as account A until it returns 429 (or a generous cap). Count the
    // non-429 responses that preceded the first 429 — that is the admitted burst.
    let c = client();
    let mut preceding_non_429 = 1usize; // the probe above already spent one token
    let mut got_429_at: Option<usize> = None;
    for i in 2..=200 {
        let status = proxy_get(&c, &base, &a_cookie).await;
        if status == 429 {
            got_429_at = Some(i);
            break;
        }
        assert_eq!(
            status, 403,
            "under the threshold a blocked target is 403 (charged), not {status} (req {i})"
        );
        preceding_non_429 += 1;
    }

    let at = got_429_at.expect("account A must hit 429 after exhausting its bucket");
    assert!(
        preceding_non_429 >= 100,
        "the limiter must admit a real burst before 429 (admitted {preceding_non_429}, 429 at req {at}) \
         — a tiny cap would indicate a mis-wired budget"
    );
    eprintln!(
        "[t18 ratelimit] account A: admitted {preceding_non_429} requests, then 429 at request {at}"
    );

    // Account B (independent bucket) is NOT limited by A's spend: a normal reader under
    // the threshold is fine. Its target is still SSRF-refused → 403, crucially NOT 429.
    let status_b = proxy_get(&c, &base, &b_cookie).await;
    assert_ne!(
        status_b, 429,
        "a second account must have its own budget (got {status_b}) — the limit is per-account"
    );
    assert_eq!(
        status_b, 403,
        "account B's request is admitted (403 from the SSRF gate), proving it was not rate-limited"
    );
    eprintln!("[t18 ratelimit] account B (independent bucket) → {status_b} (not 429)");
}

/// Perform a fresh login on a throwaway client and return account A's raw session
/// cookie value (so it can be sent explicitly).
async fn login_raw_cookie(base: &str, mock: &str) -> String {
    let c = reqwest::Client::builder().build().unwrap();
    let resp = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "jmapUrl": mock, "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap();
    let cookie = resp
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find_map(|c| c.strip_prefix("mw_session="))
        .and_then(|c| c.split(';').next())
        .expect("login set an mw_session cookie")
        .to_string();
    let body: Value = resp.json().await.unwrap_or(json!({"ok": true}));
    let _ = body;
    cookie
}
