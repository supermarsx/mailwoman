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
//! request short-circuits to `429` before the fetch. Both accounts present a
//! remote-image grant on a seeded message ([`seed_grant`]), so their requests take the
//! real fetch path (grant gate passed, rate check, egress gate) rather than the
//! ungranted refusal, and B's admitted request is identified by the EGRESS body. (A blocked target is never cached,
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

mod common;
use common::test_db;

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

fn temp_db() -> String {
    let dir = test_db::unique_dir("mw-t18-ratelimit");
    dir.join("mw.db").to_string_lossy().into_owned()
}

/// Origins of the mock upstreams this test binary has started. Proxy mode refuses a
/// loopback JMAP upstream unless the operator allowlist names it (t24 B2), so the
/// server under test lists these in `SecurityConfig::jmap_upstreams` — the
/// production control an operator uses for an internal upstream, not a test switch.
/// A test must start its mock before its server.
static MOCK_UPSTREAMS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// The mock upstreams started so far, as an allowlist.
fn mock_upstreams() -> Option<Vec<String>> {
    Some(MOCK_UPSTREAMS.lock().unwrap().clone())
}

// The mock this test started last. The server under test signs in to it as
// its configured upstream (`MW_UPSTREAM_DEFAULT`): a login can no longer name
// one (t32-h9). Per thread, so tests running in parallel keep their own.
thread_local! {
    static LAST_MOCK: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

fn last_mock() -> Option<String> {
    LAST_MOCK.with(|m| m.borrow().clone())
}

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    let origin = format!("http://{addr}");
    MOCK_UPSTREAMS.lock().unwrap().push(origin.clone());
    LAST_MOCK.with(|m| *m.borrow_mut() = Some(origin.clone()));
    origin
}

async fn spawn_server(db_path: &str) -> SocketAddr {
    let web = PathBuf::from(db_path)
        .parent()
        .unwrap()
        .join(format!("web-{}", test_db::unique_tag()));
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
        security: mw_server::SecurityConfig {
            jmap_upstreams: mock_upstreams(),
            upstream_default: last_mock(),
            ..mw_server::SecurityConfig::default()
        },
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

async fn login_cookie(c: &reqwest::Client, base: &str, _mock: &str) -> String {
    let body: Value = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["ok"], json!(true), "plain login (no 2FA): {body}");
    body["accountId"]
        .as_str()
        .expect("login names the session's accountId")
        .to_string()
}

/// Seed a real session for a DISTINCT account (its own rate-limit bucket). It names
/// the server's configured upstream (`upstream`) and the mock's credentials, as a
/// login would store them: a session whose stored upstream differs from the
/// configured one is re-validated against it on every request (`refresh_session`),
/// and made-up credentials are then refused with `401`.
async fn seed_session(db_path: &str, account_id: &str, upstream: &str) -> String {
    let store = Store::open(db_path, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    let cookie = store
        .create_session(
            account_id,
            mw_mock_jmap::USER,
            upstream,
            upstream,
            &Credentials {
                username: mw_mock_jmap::USER.into(),
                password: mw_mock_jmap::PASS.into(),
            },
        )
        .await
        .unwrap();
    // A session carries a verified primary identity since the 0033 identity model;
    // without one every policy read for it is refused ("privacy policy unavailable").
    // Bound exactly as a password login binds it (precedent: t19_e2e.rs).
    store
        .set_session_authentication(&cookie, "password")
        .await
        .expect("session authentication method");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    store
        .bind_verified_primary(
            &cookie,
            mw_store::identity_principal::VerifiedPrimary {
                account_id,
                kind: "mailbox",
                authority: upstream,
                subject: account_id,
            },
            now,
        )
        .await
        .expect("bind the verified identity")
        .expect("the session takes the verified identity");
    cookie
}

/// A message on `account`, covered by an `all` remote-image grant. The `accounts`
/// row is raw-inserted with the session's account id (`messages` has a foreign key to
/// it and `create_account` mints a random id; precedent `t22_egress_fetch_no_leak.rs`).
async fn seed_grant(db: &str, account: &str) -> String {
    let opts = sqlx::sqlite::SqliteConnectOptions::new().filename(db);
    let raw = sqlx::SqlitePool::connect_with(opts).await.unwrap();
    sqlx::query(
        "INSERT INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json)
         VALUES (?1, 'imap', 'mail.example', 993, 1, ?1, X'00', '{}')",
    )
    .bind(account)
    .execute(&raw)
    .await
    .expect("seed account row");
    raw.close().await;
    let store = Store::open(db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    let mailbox = store
        .upsert_mailbox(&mw_store::MailboxUpsert {
            account_id: account,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 1,
            uidnext: 2,
            highestmodseq: 1,
            total: 1,
            unread: 0,
            parent_id: None,
        })
        .await
        .expect("seed mailbox");
    let id = store
        .upsert_message(&mw_store::MessageUpsert {
            account_id: account,
            mailbox_id: &mailbox,
            uid: 1,
            uidvalidity: 1,
            message_id: Some("<ratelimit-grant@test>"),
            thread_id: None,
            internaldate: Some("2026-08-16T12:00:00Z"),
            size: 42,
            flags_json: "[]",
            envelope: None,
            blob_ref: None,
        })
        .await
        .expect("seed message");
    store
        .grant_remote_image(account, "all", "")
        .await
        .expect("grant remote images");
    id
}

/// The egress gate's refusal body (`image_proxy.rs::refusal_response`, `Blocked`).
const EGRESS_REFUSAL: &str = "target address is not permitted";

/// One `/api/image-proxy` GET for a blocked (SSRF-refused) target, with an explicit
/// session cookie (so we can drive two distinct accounts) and that account's granted
/// message.
async fn proxy_get(c: &reqwest::Client, base: &str, cookie: &str, email_id: &str) -> (u16, String) {
    let resp = c
        .get(format!(
            "{base}/api/image-proxy?url=http%3A%2F%2F127.0.0.1%2Fx.png&emailId={email_id}"
        ))
        .header("Cookie", format!("mw_session={cookie}"))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.text().await.unwrap())
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
    let account_a = login_cookie(&ca, &base, &mock).await;
    let email_a = seed_grant(&db, &account_a).await;
    let cookie_a = ca
        .get(format!(
            "{base}/api/image-proxy?url=http%3A%2F%2F127.0.0.1%2Fx.png&emailId={email_a}"
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
    let b_cookie = seed_session(&db, "acct-ratelimit-b", &mock).await;
    let email_b = seed_grant(&db, "acct-ratelimit-b").await;

    // Hammer the proxy as account A until it returns 429 (or a generous cap). Count the
    // non-429 responses that preceded the first 429 — that is the admitted burst.
    let c = client();
    let mut preceding_non_429 = 1usize; // the probe above already spent one token
    let mut got_429_at: Option<usize> = None;
    for i in 2..=200 {
        let (status, _) = proxy_get(&c, &base, &a_cookie, &email_a).await;
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
    let (status_b, body_b) = proxy_get(&c, &base, &b_cookie, &email_b).await;
    assert_ne!(
        status_b, 429,
        "a second account must have its own budget (got {status_b}) — the limit is per-account"
    );
    assert_eq!(
        (status_b, body_b.as_str()),
        (403, EGRESS_REFUSAL),
        "account B's request is admitted (403 from the SSRF gate), proving it was not rate-limited"
    );
    eprintln!("[t18 ratelimit] account B (independent bucket) → {status_b} (not 429)");
}

/// Perform a fresh login on a throwaway client and return account A's raw session
/// cookie value (so it can be sent explicitly).
async fn login_raw_cookie(base: &str, _mock: &str) -> String {
    let c = reqwest::Client::builder().build().unwrap();
    let resp = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
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
