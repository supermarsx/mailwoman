//! t48-e2 — console settings that now take effect, over the real router:
//!
//! - `passwd.policy.*`: the domain's policy is what `GET /api/password/policy`
//!   shows and what `POST /api/password` and `mailwoman password` enforce; a
//!   domain minimum below the built-in 12 is honoured (never below 8); a stored
//!   password that does not meet a newly set policy still signs in.
//! - `assistant.enabled`: an administrator's per-user `false` turns Assist off for
//!   that user only.
//! - `limit.ai_calls_per_day`: the (N+1)th call of the day is refused with 429 and
//!   nothing is sent; a call the gateway refuses is not counted.
//! - `assistant.rateLimitPerMin`: the console value reaches the gateway at the
//!   next start (the environment-pin precedence is covered by the unit tests in
//!   `v7_mount.rs`, which can set the variable without a server).
//!
//! Run:
//!   cargo test -p mw-server --test t48_password_assist_settings --locked -- --test-threads=1

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};

mod common;
use common::test_db;

const KEY_HEX: &str = "48e20a1b2c3d4e5f60718293a4b5c6d748e20a1b2c3d4e5f60718293a4b5c6d7";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T48</div>";
const USER: &str = "reader@example.org";
const DOMAIN: &str = "example.org";
const PASS: &str = "Read-Passw0rd!";

// ── a mock OpenAI-compatible provider ────────────────────────────────────────

const OPENAI_SSE: &str = concat!(
    "data: {\"choices\":[{\"delta\":{\"content\":\"OK\"},\"finish_reason\":null}]}\n\n",
    "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
    "data: [DONE]\n\n",
);

#[derive(Clone, Default)]
struct Mock {
    seen: Arc<Mutex<usize>>,
}

impl Mock {
    fn count(&self) -> usize {
        *self.seen.lock().unwrap()
    }
}

async fn mock_any(State(m): State<Mock>, _uri: Uri) -> Response {
    *m.seen.lock().unwrap() += 1;
    (
        StatusCode::OK,
        [("content-type", "text/event-stream")],
        OPENAI_SSE,
    )
        .into_response()
}

async fn spawn_mock() -> (String, Mock) {
    let mock = Mock::default();
    let app = axum::Router::new()
        .fallback(mock_any)
        .with_state(mock.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("127.0.0.1:{}", addr.port()), mock)
}

// ── server harness ───────────────────────────────────────────────────────────

fn admin_v6() -> V6Config {
    V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    }
}

fn fresh_dir(tag: &str) -> PathBuf {
    let dir = test_db::unique_dir(tag);
    let web = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    dir
}

/// Boot an engine-mode server over `dir` (booting twice is a restart) whose
/// upstream is the scripted POP3 mock. Per-user administrator values apply to
/// native accounts, which is why this runs in engine mode.
async fn boot(dir: &Path, pop: &common::mock_mail::MockPop3) -> String {
    // SAFETY: the canonical gate runs this binary with --test-threads=1, and no
    // test here reads these variables otherwise. The mock speaks plain POP3.
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
        std::env::remove_var("MW_PASSWD_BACKEND");
        std::env::remove_var("MW_ASSIST_RATE_LIMIT_PER_MIN");
    }
    let config = AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(dir.join("web")),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            upstream_default: Some(pop.url()),
            ..SecurityConfig::default()
        },
    };
    let app = mw_server::build_app_full(config, admin_v6())
        .await
        .expect("build_app_full")
        .0;
    common::auth::no_signin_cooldown_at(&dir.join("mw.db").to_string_lossy(), KEY_HEX).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{addr}")
}

/// Sign in as [`USER`] with `password`: the client (with the storage-authority
/// header the engine requires) and the account id, or the login status.
async fn sign_in(base: &str, password: &str) -> Result<(reqwest::Client, String), u16> {
    let jar = Arc::new(reqwest::cookie::Jar::default());
    let login = reqwest::Client::builder()
        .cookie_provider(jar.clone())
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let r = login
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": USER, "password": password }))
        .send()
        .await
        .unwrap();
    let status = r.status().as_u16();
    let me: Value = r.json().await.unwrap_or(Value::Null);
    if me["ok"] != json!(true) {
        return Err(status);
    }
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "x-mailwoman-storage-authority",
        me["storageAuthority"].as_str().unwrap().parse().unwrap(),
    );
    let c = reqwest::Client::builder()
        .cookie_provider(jar)
        .default_headers(headers)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    Ok((c, me["accountId"].as_str().unwrap().to_string()))
}

async fn admin(base: &str) -> reqwest::Client {
    let c = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let r = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "admin login");
    c
}

async fn put_setting(c: &reqwest::Client, url: String, body: Value) {
    let r = c.put(&url).json(&body).send().await.unwrap();
    let status = r.status();
    let text = r.text().await.unwrap();
    assert!(status.is_success(), "PUT {url}: {status} {text}");
}

async fn delete_setting(c: &reqwest::Client, url: String) {
    let r = c.delete(&url).send().await.unwrap();
    assert!(r.status().is_success(), "DELETE {url}: {}", r.status());
}

async fn open_store(dir: &Path) -> mw_store::Store {
    mw_store::Store::open(
        &dir.join("mw.db").to_string_lossy(),
        mw_store::ServerKey::from_hex(KEY_HEX).unwrap(),
    )
    .await
    .unwrap()
}

// ── passwd.policy.* ──────────────────────────────────────────────────────────

async fn policy(u: &reqwest::Client, base: &str) -> Value {
    let r = u
        .get(format!("{base}/api/password/policy"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    r.json().await.unwrap()
}

async fn change(u: &reqwest::Client, base: &str, old: &str, new: &str) -> (u16, Value) {
    let r = u
        .post(format!("{base}/api/password"))
        .json(&json!({ "oldPassword": old, "newPassword": new }))
        .send()
        .await
        .unwrap();
    let status = r.status().as_u16();
    (status, r.json().await.unwrap_or(Value::Null))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_domain_password_policy_is_shown_and_enforced_at_change_time_only() {
    let dir = fresh_dir("mw-t48-e2-passwd");
    let pop = common::mock_mail::MockPop3::start(USER, PASS).await;
    let base = boot(&dir, &pop).await;
    let c = admin(&base).await;
    let (u, account) = sign_in(&base, PASS).await.expect("first sign-in");

    // The local backend changes the hash it holds; give it the current password.
    let store = open_store(&dir).await;
    let phc = mw_passwd::hash_password_with(PASS, mw_passwd::argon_cost()).unwrap();
    store
        .set_setting(&format!("passwd_local:{account}"), &phc)
        .await
        .unwrap();

    // Precondition: nothing set, the registry default (12, no classes).
    let p = policy(&u, &base).await;
    assert_eq!(
        (p["minLength"].clone(), p["requireSymbol"].clone()),
        (json!(12), json!(false)),
        "{p}"
    );

    // The domain asks for 20 characters and a symbol.
    let key = |k: &str| format!("{base}/admin/settings/passwd.policy.{k}/domains/{DOMAIN}");
    put_setting(
        &c,
        key("minLength"),
        json!({ "value": 20, "locked": false }),
    )
    .await;
    put_setting(
        &c,
        key("requireSymbol"),
        json!({ "value": true, "locked": false }),
    )
    .await;

    // The stored password (14 characters) does not meet it, and still signs in:
    // the policy applies when a password is changed, not at sign-in.
    let (u, again) = sign_in(&base, PASS)
        .await
        .expect("an existing password that misses the new policy still signs in");
    assert_eq!(again, account);

    // The policy endpoint shows the domain's values.
    let p = policy(&u, &base).await;
    assert_eq!(p["minLength"], json!(20), "{p}");
    assert_eq!(p["requireSymbol"], json!(true), "{p}");
    assert_eq!(
        p["description"],
        json!("At least 20 characters, including a symbol.")
    );

    // 20 characters without a symbol, and a symbol with 19 characters: both 400.
    let (s, body) = change(&u, &base, PASS, "abcdefghijklmnopqrst").await;
    assert_eq!(s, 400, "no symbol: {body}");
    assert!(body["error"].as_str().unwrap().contains("symbol"), "{body}");
    let (s, body) = change(&u, &base, PASS, "abcdefghijklmnopqr!").await;
    assert_eq!(s, 400, "19 characters: {body}");
    // The refused changes left the hash alone.
    assert_eq!(
        store
            .get_setting(&format!("passwd_local:{account}"))
            .await
            .unwrap()
            .as_deref(),
        Some(phc.as_str())
    );

    // `mailwoman password` applies the same policy.
    let backend = mw_server::v7_mount::passwd_backend_from(&store, &|_| None).unwrap();
    let cli = mw_server::v7_mount::cli_password_policy(&store, backend.as_ref(), &account)
        .await
        .unwrap();
    assert_eq!((cli.min_length, cli.require_symbol), (20, true));
    let refused = mw_server::v7_mount::cli_password_change(
        &store,
        backend.as_ref(),
        &account,
        PASS,
        "short!",
    )
    .await;
    assert!(
        refused.is_err(),
        "the CLI refuses a password under the policy"
    );

    // The domain lowers the minimum to 8, below the built-in 12: a 9-character
    // password is accepted (the backend keeps only the floor of 8).
    put_setting(&c, key("minLength"), json!({ "value": 8, "locked": false })).await;
    put_setting(
        &c,
        key("requireSymbol"),
        json!({ "value": false, "locked": false }),
    )
    .await;
    // The registry never stores a minimum below 8.
    let r = c
        .put(key("minLength"))
        .json(&json!({ "value": 7, "locked": false }))
        .send()
        .await
        .unwrap();
    assert!(r.status().is_client_error(), "minLength 7: {}", r.status());
    assert_eq!(policy(&u, &base).await["minLength"], json!(8));
    let (s, body) = change(&u, &base, PASS, "seven77").await;
    assert_eq!(s, 400, "7 characters: {body}");
    let (s, body) = change(&u, &base, PASS, "ninechars").await;
    assert_eq!(s, 200, "9 characters under a minimum of 8: {body}");
    assert_eq!(body["changed"], json!(true), "{body}");
    let stored = store
        .get_setting(&format!("passwd_local:{account}"))
        .await
        .unwrap()
        .unwrap();
    assert!(
        matches!(
            mw_passwd::verify_and_upgrade("ninechars", &stored, mw_passwd::argon_cost()),
            Ok(mw_passwd::VerifyOutcome::Match { .. })
        ),
        "the new hash verifies the new password"
    );
}

// ── assistant.enabled, limit.ai_calls_per_day, assistant.rateLimitPerMin ─────

/// Invoke `capability` as the user: the status, and whether the mock got a
/// request for it.
async fn invoke(
    u: &reqwest::Client,
    base: &str,
    mock: &Mock,
    account: &str,
    capability: &str,
) -> (u16, Value, bool) {
    let before = mock.count();
    let r = u
        .post(format!("{base}/api/assist/invoke"))
        .json(&json!({
            "capability": capability,
            "scope": { "accounts": [account] },
            "input": { "prompt": "Summarize this", "context": [
                { "account": account, "folder": "", "text": "hello", "kind": "plain" }
            ] }
        }))
        .send()
        .await
        .unwrap();
    let status = r.status().as_u16();
    let text = r.text().await.unwrap_or_default();
    let body = serde_json::from_str(&text).unwrap_or(Value::Null);
    (status, body, mock.count() > before)
}

async fn availability(u: &reqwest::Client, base: &str) -> Value {
    let cfg: Value = u
        .get(format!("{base}/api/assist/config"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    cfg["availability"].clone()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn assist_enabled_daily_calls_and_rate_limit_come_from_the_console() {
    let (mock_hostport, mock) = spawn_mock().await;
    let dir = fresh_dir("mw-t48-e2-assist");
    let pop = common::mock_mail::MockPop3::start(USER, PASS).await;
    let base = boot(&dir, &pop).await;
    let c = admin(&base).await;
    let (_, account) = sign_in(&base, PASS).await.expect("sign-in");

    // Assist on, Summarize granted, the account in the ceiling; a provider profile
    // on the mock serves it (the shape t39_assist_profiles uses).
    let r = c
        .put(format!("{base}/admin/assist"))
        .json(&json!({
            "enabled": true,
            "adapter": null,
            "capabilityGrants": ["summarize"],
            "dataCeilings": { "accounts": [account], "folders": [],
                              "includeE2ee": false, "includeAttachments": false }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    common::auth::admin_step_up(&c, &format!("{base}/admin"), "hunter2", &[]).await;
    let r = c
        .post(format!("{base}/admin/assist/egress-grants"))
        .json(&json!({ "destination": mock_hostport, "note": "mock" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "grant");
    let profile: Value = c
        .post(format!("{base}/admin/assist/profiles"))
        .json(&json!({
            "name": "mock", "preset": "openai-compatible",
            "endpoint": format!("http://{mock_hostport}/v1"), "model": "m"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let profile = profile["id"].as_str().expect("profile id").to_string();
    // A console rate limit of 3 a minute, read when the gateway is built.
    put_setting(
        &c,
        format!("{base}/admin/settings/assistant.rateLimitPerMin"),
        json!({ "globalValue": 3 }),
    )
    .await;

    // Restart: the gateway is built switched on, with the console rate limit.
    let base = boot(&dir, &pop).await;
    let c = admin(&base).await;
    let (u, again) = sign_in(&base, PASS).await.expect("sign-in after restart");
    assert_eq!(again, account);
    put_setting(
        &c,
        format!("{base}/admin/settings/assist.profile"),
        json!({ "globalValue": profile }),
    )
    .await;
    assert_eq!(availability(&u, &base).await, json!("enabled"));

    // ── assistant.rateLimitPerMin = 3 (console): the 4th request in the minute ──
    for n in 1..=3 {
        let (s, body, sent) = invoke(&u, &base, &mock, &account, "summarize").await;
        assert_eq!((s, sent), (200, true), "request {n} of 3: {body}");
    }
    let (s, body, sent) = invoke(&u, &base, &mock, &account, "summarize").await;
    assert_eq!(
        (s, sent),
        (429, false),
        "the console rate limit applies: {body}"
    );
    assert_eq!(body["error"], json!("assist rate limit exceeded"));

    // ── assistant.enabled: a per-user `false` turns Assist off for that user ──
    // (checked before the gateway, so the spent rate window does not matter)
    let user_key = format!("{base}/admin/settings/assistant.enabled/users/{account}");
    put_setting(&c, user_key.clone(), json!({ "value": false })).await;
    assert_eq!(availability(&u, &base).await, json!("disabled"));
    let (s, _, sent) = invoke(&u, &base, &mock, &account, "summarize").await;
    assert_eq!((s, sent), (404, false), "assistant.enabled=false refuses");
    delete_setting(&c, user_key).await;
    assert_eq!(availability(&u, &base).await, json!("enabled"));

    // Back to no rate limit (0 = none), applied at the next start.
    put_setting(
        &c,
        format!("{base}/admin/settings/assistant.rateLimitPerMin"),
        json!({ "globalValue": 0 }),
    )
    .await;
    let base = boot(&dir, &pop).await;
    let c = admin(&base).await;
    let (u, _) = sign_in(&base, PASS).await.expect("sign-in after restart");

    // ── limit.ai_calls_per_day ──
    // Today's calls so far were made with no daily limit, so none was counted.
    put_setting(
        &c,
        format!("{base}/admin/settings/limit.ai_calls_per_day/domains/{DOMAIN}"),
        json!({ "value": 2, "locked": false }),
    )
    .await;
    // A call the gateway refuses (Draft is not granted) is given back.
    let (s, body, sent) = invoke(&u, &base, &mock, &account, "draft").await;
    assert_eq!((s, sent), (403, false), "{body}");
    for n in 1..=2 {
        let (s, body, sent) = invoke(&u, &base, &mock, &account, "summarize").await;
        assert_eq!((s, sent), (200, true), "call {n} of 2: {body}");
    }
    let (s, body, sent) = invoke(&u, &base, &mock, &account, "summarize").await;
    assert_eq!((s, sent), (429, false), "the third call of the day: {body}");
    assert_eq!(body["code"], json!("limit.ai_calls_per_day"), "{body}");
    assert_eq!(body["limit"], json!(2));

    // The counter is the shared `usage_counter` row: another handle sees 2.
    let store = open_store(&dir).await;
    let day = mw_store::usage::period_of(
        mw_store::usage::ASSIST_CALLS,
        chrono::Utc::now().timestamp(),
    );
    assert_eq!(
        store
            .usage_sum(USER, mw_store::usage::ASSIST_CALLS, day, day)
            .await
            .unwrap(),
        2
    );

    // Raising the limit admits the next call; 0 removes it.
    put_setting(
        &c,
        format!("{base}/admin/settings/limit.ai_calls_per_day/domains/{DOMAIN}"),
        json!({ "value": 0, "locked": false }),
    )
    .await;
    let (s, body, sent) = invoke(&u, &base, &mock, &account, "summarize").await;
    assert_eq!((s, sent), (200, true), "0 = no limit: {body}");
}
