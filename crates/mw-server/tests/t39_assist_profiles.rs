//! t39-e11 (N8) — AI provider profiles, end to end over the real router.
//!
//! A local mock provider speaks the OpenAI-compatible chat-completions SSE format
//! and the Anthropic Messages SSE format, and records every request it gets. The
//! legs:
//!
//! - the egress gate: a profile on 127.0.0.1 is refused before anything is sent,
//!   and works once the exact `host:port` is granted (step-up required);
//! - "Test profile": latency and reply on success; `timeout`, `unauthorized`
//!   (with the provider's code, not its message), `not-found`, `rate-limited`,
//!   `upstream` on failure; the Anthropic dialect's headers and body;
//! - the sealed key never appears in an admin response, the admin audit log or
//!   the server's log output — even when the provider echoes it back;
//! - assignment precedence through `assist.profile` (user over domain over
//!   deployment, with domain and deployment locks), a missing profile turning
//!   Assist off rather than falling back, and the kill switch;
//! - a zero-access account's content is never forwarded.
//!
//! Run:
//!   cargo test -p mw-server --test t39_assist_profiles -- --test-threads=1

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};

mod common;
use common::test_db;

const KEY_HEX: &str = "39e11a1b2c3d4e5f60718293a4b5c6d739e11a1b2c3d4e5f60718293a4b5c6d7";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T39</div>";
const SECRET: &str = "sk-t39e11-SEALED-KEY-never-shown-7c1f";

// ── log capture ──────────────────────────────────────────────────────────────

/// Everything the process logs at debug level and above, for the "the key is
/// never logged" assertions.
fn log_buffer() -> Arc<Mutex<Vec<u8>>> {
    static BUF: OnceLock<Arc<Mutex<Vec<u8>>>> = OnceLock::new();
    BUF.get_or_init(|| {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let writer = buf.clone();
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::new("debug"))
            .with_ansi(false)
            .with_writer(move || LogWriter(writer.clone()))
            .try_init();
        buf
    })
    .clone()
}

struct LogWriter(Arc<Mutex<Vec<u8>>>);
impl std::io::Write for LogWriter {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// ── the mock provider ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Seen {
    path: String,
    headers: Vec<(String, String)>,
    body: String,
}

#[derive(Clone, Default)]
struct Mock {
    seen: Arc<Mutex<Vec<Seen>>>,
}

impl Mock {
    fn requests(&self) -> Vec<Seen> {
        self.seen.lock().unwrap().clone()
    }
    fn count(&self) -> usize {
        self.seen.lock().unwrap().len()
    }
}

const OPENAI_SSE: &str = concat!(
    "data: {\"choices\":[{\"delta\":{\"content\":\"OK\"},\"finish_reason\":null}]}\n\n",
    "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
    "data: [DONE]\n\n",
);

const ANTHROPIC_SSE: &str = concat!(
    "event: message_start\n",
    "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[]}}\n\n",
    "event: content_block_start\n",
    "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
    "event: content_block_delta\n",
    "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"OK\"}}\n\n",
    "event: content_block_stop\n",
    "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
    "event: message_delta\n",
    "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"}}\n\n",
    "event: message_stop\n",
    "data: {\"type\":\"message_stop\"}\n\n",
);

fn sse(body: &'static str) -> Response {
    (
        StatusCode::OK,
        [("content-type", "text/event-stream")],
        body,
    )
        .into_response()
}

/// The mode is the first path segment: `/ok/…`, `/slow/…`, `/e401/…`, `/e404/…`,
/// `/e429/…`, `/e500/…`, `/anth/…`, or any other name for an `ok` reply (the
/// precedence legs use `/a`, `/b`, `/c`).
async fn mock_any(State(m): State<Mock>, uri: Uri, headers: HeaderMap, body: String) -> Response {
    let path = uri.path().to_string();
    m.seen.lock().unwrap().push(Seen {
        path: path.clone(),
        headers: headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect(),
        body,
    });
    let mode = path.trim_start_matches('/').split('/').next().unwrap_or("");
    match mode {
        "slow" => {
            tokio::time::sleep(Duration::from_secs(5)).await;
            sse(OPENAI_SSE)
        }
        "e401" => (
            StatusCode::UNAUTHORIZED,
            [("content-type", "application/json")],
            // A provider that echoes the key it refused, as OpenAI partly does.
            format!(
                r#"{{"error":{{"message":"Incorrect API key provided: {SECRET}","type":"invalid_request_error","code":"invalid_api_key"}}}}"#
            ),
        )
            .into_response(),
        "e404" => (
            StatusCode::NOT_FOUND,
            r#"{"error":{"code":"model_not_found"}}"#,
        )
            .into_response(),
        "e429" => (StatusCode::TOO_MANY_REQUESTS, "{}").into_response(),
        "e500" => (StatusCode::INTERNAL_SERVER_ERROR, "boom").into_response(),
        "anth" => sse(ANTHROPIC_SSE),
        _ => sse(OPENAI_SSE),
    }
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

fn fresh_dir() -> PathBuf {
    let dir = test_db::unique_dir("mw-t39-e11-profiles");
    let web = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    dir
}

async fn spawn_jmap() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

/// Boot a proxy-mode server over `dir`. Booting twice over one `dir` is a
/// restart: the Assist gateway is built once per boot.
async fn boot(dir: &Path, jmap: &str) -> String {
    let config = AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(dir.join("web")),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(vec![jmap.to_string()]),
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

fn browser() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

async fn admin(base: &str) -> reqwest::Client {
    let c = browser();
    let r = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "admin login");
    c
}

async fn user(base: &str) -> (reqwest::Client, String) {
    let c = browser();
    let r = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap();
    let status = r.status();
    let body: Value = r.json().await.unwrap();
    assert!(status.is_success(), "user login: {status} {body}");
    (c, body["accountId"].as_str().unwrap().to_string())
}

/// Every admin response body is collected here, for the key-never-shown check.
#[derive(Default)]
struct Bodies(Vec<String>);

impl Bodies {
    async fn send(&mut self, req: reqwest::RequestBuilder) -> (StatusCode, Value) {
        let r = req.send().await.unwrap();
        let status = r.status();
        let text = r.text().await.unwrap();
        self.0.push(text.clone());
        (status, serde_json::from_str(&text).unwrap_or(Value::Null))
    }
}

fn profile(name: &str, preset: &str, endpoint: &str, model: &str) -> Value {
    json!({ "name": name, "preset": preset, "endpoint": endpoint, "model": model })
}

async fn create(b: &mut Bodies, c: &reqwest::Client, base: &str, body: Value) -> String {
    let (s, v) = b
        .send(c.post(format!("{base}/admin/assist/profiles")).json(&body))
        .await;
    assert_eq!(s, 200, "create profile: {v}");
    v["id"].as_str().unwrap().to_string()
}

async fn test_profile(b: &mut Bodies, c: &reqwest::Client, base: &str, id: &str) -> Value {
    let (s, v) = b
        .send(c.post(format!("{base}/admin/assist/profiles/{id}/test")))
        .await;
    assert_eq!(s, 200, "test profile: {v}");
    v
}

async fn grant(b: &mut Bodies, c: &reqwest::Client, base: &str, destination: &str) {
    let (s, v) = b
        .send(
            c.post(format!("{base}/admin/assist/egress-grants"))
                .json(&json!({ "destination": destination, "note": "mock" })),
        )
        .await;
    assert_eq!(s, 200, "grant: {v}");
}

async fn step_up(c: &reqwest::Client, base: &str) {
    common::auth::admin_step_up(c, &format!("{base}/admin"), "hunter2", &[]).await;
}

/// The admin audit log's text, read straight from the database file.
fn audit_text(dir: &Path) -> String {
    let rt = tokio::runtime::Handle::current();
    let path = dir.join("mw.db").to_string_lossy().into_owned();
    std::thread::spawn(move || {
        rt.block_on(async move {
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(1)
                .connect(&format!("sqlite://{path}?mode=ro"))
                .await
                .unwrap();
            let rows: Vec<(String, String, String)> = sqlx::query_as(
                "SELECT action, COALESCE(target,''), COALESCE(detail_json,'') FROM audit_log",
            )
            .fetch_all(&pool)
            .await
            .unwrap();
            rows.into_iter()
                .map(|(a, t, d)| {
                    format!(
                        "{a} {t} {d}
"
                    )
                })
                .collect::<String>()
        })
    })
    .join()
    .unwrap()
}

// ── legs ─────────────────────────────────────────────────────────────────────

/// The egress gate, the test call, error mapping, the Anthropic dialect, and the
/// key never leaving in a response, the audit log or a log line.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn profiles_are_egress_gated_testable_and_never_show_their_key() {
    let logs = log_buffer();
    let (mock_hostport, mock) = spawn_mock().await;
    let dir = fresh_dir();
    let base = boot(&dir, &spawn_jmap().await).await;
    let c = admin(&base).await;
    let mut b = Bodies::default();

    // Presets: about fifteen, only the two dialect mocks claimed.
    let (s, presets) = b.send(c.get(format!("{base}/admin/assist/presets"))).await;
    assert_eq!(s, 200);
    let list = presets["presets"].as_array().unwrap();
    assert!(list.len() >= 15, "{}", list.len());
    let checked: Vec<&str> = list
        .iter()
        .filter(|p| p["verification"] != "unverified")
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert_eq!(checked, ["openai-compatible", "anthropic"]);

    // ── SSRF: a loopback endpoint without a grant is refused before sending ──
    let mut ok = profile(
        "Mock OK",
        "openai-compatible",
        &format!("http://{mock_hostport}/ok/v1"),
        "mock-model",
    );
    ok["apiKey"] = json!(SECRET);
    ok["params"] = json!({ "timeoutMs": 3000, "temperature": 0.2, "maxTokens": 32 });
    let ok_id = create(&mut b, &c, &base, ok).await;
    let refused = test_profile(&mut b, &c, &base, &ok_id).await;
    assert_eq!(refused["ok"], false);
    assert_eq!(refused["error"]["kind"], "egress-refused", "{refused}");
    assert_eq!(mock.count(), 0, "nothing reached the loopback endpoint");

    // A grant needs a step-up, and names the exact host:port.
    let (s, v) = b
        .send(
            c.post(format!("{base}/admin/assist/egress-grants"))
                .json(&json!({ "destination": mock_hostport })),
        )
        .await;
    assert_eq!(s, 403, "grant without step-up: {v}");
    step_up(&c, &base).await;
    let (s, _) = b
        .send(
            c.post(format!("{base}/admin/assist/egress-grants"))
                .json(&json!({ "destination": "169.254.169.254:80" })),
        )
        .await;
    assert_eq!(s, 422, "metadata can never be granted");
    grant(&mut b, &c, &base, &mock_hostport).await;

    // ── the test call: reply, latency, the request as sent ──
    let passed = test_profile(&mut b, &c, &base, &ok_id).await;
    assert_eq!(passed["ok"], true, "{passed}");
    assert_eq!(passed["reply"], "OK");
    assert!(passed["latencyMs"].as_u64().is_some());
    let sent = mock.requests();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].path, "/ok/v1/chat/completions");
    let auth = sent[0]
        .headers
        .iter()
        .find(|(k, _)| k == "authorization")
        .map(|(_, v)| v.clone());
    assert_eq!(auth.as_deref(), Some(format!("Bearer {SECRET}").as_str()));
    let body: Value = serde_json::from_str(&sent[0].body).unwrap();
    assert_eq!(body["model"], "mock-model");
    assert_eq!(body["max_tokens"], 32);
    assert_eq!(body["temperature"], 0.2);
    assert_eq!(body["stream"], true);

    // ── timeout ──
    let mut slow = profile(
        "Slow",
        "openai-compatible",
        &format!("http://{mock_hostport}/slow/v1"),
        "m",
    );
    slow["params"] = json!({ "timeoutMs": 1000 });
    let slow_id = create(&mut b, &c, &base, slow).await;
    let timed_out = test_profile(&mut b, &c, &base, &slow_id).await;
    assert_eq!(timed_out["error"]["kind"], "timeout", "{timed_out}");
    assert!(timed_out["latencyMs"].as_u64().unwrap() < 4000);

    // ── error mapping ──
    for (mode, kind, status, code) in [
        ("e401", "unauthorized", 401, Some("invalid_api_key")),
        ("e404", "not-found", 404, Some("model_not_found")),
        ("e429", "rate-limited", 429, None),
        ("e500", "upstream", 500, None),
    ] {
        let mut p = profile(
            mode,
            "openai-compatible",
            &format!("http://{mock_hostport}/{mode}/v1"),
            "m",
        );
        p["apiKey"] = json!(SECRET);
        let id = create(&mut b, &c, &base, p).await;
        let out = test_profile(&mut b, &c, &base, &id).await;
        assert_eq!(out["ok"], false, "{mode}");
        assert_eq!(out["error"]["kind"], kind, "{mode}: {out}");
        assert_eq!(out["error"]["status"], status, "{mode}");
        assert_eq!(out["error"]["upstreamCode"].as_str(), code, "{mode}: {out}");
    }

    // ── the Anthropic dialect ──
    let mut anth = profile(
        "Anthropic mock",
        "anthropic",
        &format!("http://{mock_hostport}/anth"),
        "claude-haiku-5-5",
    );
    anth["apiKey"] = json!(SECRET);
    let anth_id = create(&mut b, &c, &base, anth).await;
    let a = test_profile(&mut b, &c, &base, &anth_id).await;
    assert_eq!(a["ok"], true, "{a}");
    assert_eq!(a["reply"], "OK");
    let last = mock.requests().pop().unwrap();
    assert_eq!(last.path, "/anth/v1/messages");
    let h = |name: &str| {
        last.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
    };
    assert_eq!(h("x-api-key").as_deref(), Some(SECRET));
    assert_eq!(h("anthropic-version").as_deref(), Some("2023-06-01"));
    assert!(h("authorization").is_none());
    let body: Value = serde_json::from_str(&last.body).unwrap();
    assert_eq!(body["model"], "claude-haiku-5-5");
    assert_eq!(body["max_tokens"], 16000);
    assert!(body["system"].is_string());

    // ── update keeps the key; moving the endpoint without one is refused ──
    let (s, v) = b
        .send(
            c.put(format!("{base}/admin/assist/profiles/{anth_id}"))
                .json(&profile(
                    "Renamed",
                    "anthropic",
                    &format!("http://{mock_hostport}/anth"),
                    "claude-sonnet-5-5",
                )),
        )
        .await;
    assert_eq!(s, 200, "{v}");
    assert_eq!(v["hasApiKey"], true);
    let (s, v) = b
        .send(
            c.put(format!("{base}/admin/assist/profiles/{anth_id}"))
                .json(&profile(
                    "Moved",
                    "anthropic",
                    &format!("http://{mock_hostport}/elsewhere"),
                    "claude-sonnet-5-5",
                )),
        )
        .await;
    assert_eq!(s, 422, "endpoint moved without a key: {v}");

    // ── the key is in no response, no audit row and no log line ──
    let (s, listed) = b.send(c.get(format!("{base}/admin/assist/profiles"))).await;
    assert_eq!(s, 200);
    assert!(
        listed["profiles"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p.get("apiKey").is_none())
    );
    for body in &b.0 {
        assert!(!body.contains(SECRET), "a response carried the key: {body}");
    }
    let audit = audit_text(&dir);
    assert!(audit.contains("assist-profile-tested"), "{audit}");
    assert!(!audit.contains(SECRET), "the audit log carried the key");
    let logged = String::from_utf8_lossy(&logs.lock().unwrap()).into_owned();
    assert!(!logged.is_empty(), "the log capture is live");
    assert!(!logged.contains(SECRET), "a log line carried the key");
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

/// Invoke Summarize as the user and return the status plus the path of the
/// request the mock got for it (`None` when nothing was sent).
async fn summarize(
    u: &reqwest::Client,
    base: &str,
    mock: &Mock,
    account: &str,
    text: &str,
) -> (u16, Option<Seen>) {
    let before = mock.count();
    let r = u
        .post(format!("{base}/api/assist/invoke"))
        .json(&json!({
            "capability": "summarize",
            "scope": { "accounts": [account] },
            "input": { "prompt": "Summarize this", "context": [
                { "account": account, "folder": "", "text": text, "kind": "plain" }
            ] }
        }))
        .send()
        .await
        .unwrap();
    let status = r.status().as_u16();
    let _ = r.text().await;
    let seen = mock.requests();
    (
        status,
        (seen.len() > before).then(|| seen[seen.len() - 1].clone()),
    )
}

/// Assignment precedence, a missing profile, the kill switch, zero-access.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_resolved_profile_serves_assist_by_precedence_and_privacy_holds() {
    let (mock_hostport, mock) = spawn_mock().await;
    let dir = fresh_dir();
    let jmap = spawn_jmap().await;
    let base = boot(&dir, &jmap).await;
    let c = admin(&base).await;
    let mut b = Bodies::default();
    // The user signs in once so the account exists; the ceiling names it.
    let (_, account) = user(&base).await;

    // Assist on, Summarize granted, the account in the ceiling, no deployment
    // endpoint: only a profile can serve.
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
    step_up(&c, &base).await;
    grant(&mut b, &c, &base, &mock_hostport).await;
    let mut ids = Vec::new();
    for name in ["a", "b", "c"] {
        ids.push(
            create(
                &mut b,
                &c,
                &base,
                profile(
                    name,
                    "openai-compatible",
                    &format!("http://{mock_hostport}/{name}/v1"),
                    "m",
                ),
            )
            .await,
        );
    }
    let (pa, pb, pc) = (ids[0].clone(), ids[1].clone(), ids[2].clone());

    // Restart: the gateway is built switched on.
    let base = boot(&dir, &jmap).await;
    let c = admin(&base).await;
    let (u, _) = user(&base).await;
    let key = format!("{base}/admin/settings/assist.profile");
    let domain = mw_mock_jmap::USER.split('@').nth(1).unwrap().to_string();

    // No assignment and no deployment endpoint: Assist is off for the user.
    let cfg: Value = u
        .get(format!("{base}/api/assist/config"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(cfg["availability"], "disabled", "{cfg}");
    let (s, sent) = summarize(&u, &base, &mock, &account, "hello").await;
    assert_eq!((s, sent.is_none()), (404, true));

    // Deployment default.
    put_setting(&c, key.clone(), json!({ "globalValue": pa })).await;
    let cfg: Value = u
        .get(format!("{base}/api/assist/config"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(cfg["availability"], "enabled", "{cfg}");
    assert_eq!(cfg["endpoint_host"], mock_hostport.as_str());
    assert_eq!(cfg["capabilities"], json!(["summarize"]));
    let path = |s: Option<Seen>| s.map(|s| s.path).unwrap_or_default();
    let (s, sent) = summarize(&u, &base, &mock, &account, "hello").await;
    assert_eq!((s, path(sent)), (200, "/a/v1/chat/completions".to_string()));

    // A domain value overrides it.
    put_setting(
        &c,
        format!("{key}/domains/{domain}"),
        json!({ "value": pb, "locked": false }),
    )
    .await;
    let (_, sent) = summarize(&u, &base, &mock, &account, "hello").await;
    assert_eq!(path(sent), "/b/v1/chat/completions");

    // A per-user value overrides the domain.
    put_setting(&c, format!("{key}/users/{account}"), json!({ "value": pc })).await;
    let (_, sent) = summarize(&u, &base, &mock, &account, "hello").await;
    assert_eq!(path(sent), "/c/v1/chat/completions");

    // A domain lock holds over the user's value.
    put_setting(
        &c,
        format!("{key}/domains/{domain}"),
        json!({ "value": pb, "locked": true }),
    )
    .await;
    let (_, sent) = summarize(&u, &base, &mock, &account, "hello").await;
    assert_eq!(path(sent), "/b/v1/chat/completions");

    // A deployment lock holds over both.
    put_setting(
        &c,
        key.clone(),
        json!({ "globalValue": pa, "locked": true }),
    )
    .await;
    let (_, sent) = summarize(&u, &base, &mock, &account, "hello").await;
    assert_eq!(path(sent), "/a/v1/chat/completions");

    // Locks off; the user's profile is deleted: Assist is off for them, and
    // nothing is sent anywhere else.
    put_setting(
        &c,
        key.clone(),
        json!({ "globalValue": pa, "locked": false }),
    )
    .await;
    delete_setting(&c, format!("{key}/domains/{domain}")).await;
    let r = c
        .delete(format!("{base}/admin/assist/profiles/{pc}"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let (s, sent) = summarize(&u, &base, &mock, &account, "hello").await;
    assert_eq!(
        (s, sent.is_none()),
        (404, true),
        "a missing profile never falls back"
    );
    delete_setting(&c, format!("{key}/users/{account}")).await;

    // Mail content reaches the profile for an ordinary account...
    const PLAIN: &str = "T39-ORDINARY-CONTENT-41";
    let (s, sent) = summarize(&u, &base, &mock, &account, PLAIN).await;
    assert_eq!(s, 200);
    assert!(
        sent.unwrap().body.contains(PLAIN),
        "ordinary content is forwarded"
    );

    // ...and never for a zero-access account, whatever the ceiling allows.
    let r = c
        .post(format!("{base}/admin/users/{account}/zero-access"))
        .json(&json!({ "on": true }))
        .send()
        .await
        .unwrap();
    assert!(r.status().is_success(), "zero-access on: {}", r.status());
    const ZA: &str = "T39-ZERO-ACCESS-PLAINTEXT-42";
    let (s, sent) = summarize(&u, &base, &mock, &account, ZA).await;
    assert_eq!(s, 200, "the prompt itself still goes");
    let body = sent.unwrap().body;
    assert!(
        !body.contains(ZA),
        "zero-access content was forwarded: {body}"
    );
    assert!(body.contains("Summarize this"));

    // The kill switch stops profile requests before anything is sent.
    let r = c
        .post(format!("{base}/admin/assist/kill"))
        .json(&json!({ "on": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let (s, sent) = summarize(&u, &base, &mock, &account, "hello").await;
    assert_eq!(
        (s, sent.is_none()),
        (404, true),
        "kill switch holds for profiles"
    );
}
