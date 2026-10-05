//! t28-e7 (audit OH-4) — the admin Assist routes speak one shape, and the kill
//! switch stops the gateway that is running.
//!
//! Before 26.20 the admin screen sent `endpoint_allowlist` / `capability_locks` /
//! `data_ceilings`; `PUT /admin/assist` knew none of those names, defaulted all
//! three columns, overwrote the stored row with `{}` / `[]` / `{}` and answered
//! `204`. `GET` then returned a shape the screen could not read. The kill switch
//! ignored its body and only edited the stored row, so a running deployment kept
//! sending until it was restarted.
//!
//! Every leg drives the real router over HTTP. The kill-switch leg counts requests
//! at a loopback endpoint the gateway is configured to call, so "stopped" is
//! measured as nothing arriving there, not as a flag reading back.
//!
//! Run:
//!   cargo test -p mw-server --test t28_assist_admin --locked -- --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};

mod common;
use common::test_db;

const KEY_HEX: &str = "28e7a1b2c3d4e5f60718293a4b5c6d7e28e7a1b2c3d4e5f60718293a4b5c6d7e";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T28</div>";

// ── harness ──────────────────────────────────────────────────────────────────

fn admin_v6() -> V6Config {
    V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    }
}

/// A fresh database directory with the web root the server needs.
fn fresh_dir() -> PathBuf {
    let dir = test_db::unique_dir("mw-t28-e7-assist");
    let web = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    dir
}

/// Boot a proxy-mode server over the database in `dir`. Calling it twice with the
/// same `dir` is a restart as far as the Assist gateway is concerned: the gateway
/// is built once per boot, from the stored row.
async fn boot(dir: &std::path::Path, upstreams: Vec<String>) -> String {
    let config = AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(dir.join("web")),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(upstreams),
            ..SecurityConfig::default()
        },
    };
    let app = mw_server::build_app_full(config, admin_v6())
        .await
        .expect("build_app_full")
        .0;
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

async fn get_config(admin: &reqwest::Client, base: &str) -> Value {
    let r = admin
        .get(format!("{base}/admin/assist"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "GET /admin/assist");
    r.json().await.unwrap()
}

async fn put_config(admin: &reqwest::Client, base: &str, body: &Value) -> reqwest::Response {
    admin
        .put(format!("{base}/admin/assist"))
        .json(body)
        .send()
        .await
        .unwrap()
}

async fn status(admin: &reqwest::Client, base: &str) -> Value {
    let r = admin
        .get(format!("{base}/admin/assist/status"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "GET /admin/assist/status");
    r.json().await.unwrap()
}

/// The configuration with no row stored: every key present, everything off.
fn default_config() -> Value {
    json!({
        "enabled": false,
        "adapter": null,
        "capabilityGrants": [],
        "dataCeilings": {
            "accounts": [],
            "folders": [],
            "includeE2ee": false,
            "includeAttachments": false,
        },
    })
}

/// A configuration that sets every key to something other than its default.
fn full_config(base_url: &str) -> Value {
    json!({
        "enabled": true,
        "adapter": {
            "kind": "open-ai-compatible",
            "baseUrl": base_url,
            "apiKey": "sk-t28",
            "chatModel": "chat-t28",
            "embedModel": "embed-t28",
            "sttModel": "stt-t28",
        },
        "capabilityGrants": ["summarize", "dictation", "search-semantic"],
        "dataCeilings": {
            "accounts": ["acct-1", "acct-2"],
            "folders": ["inbox"],
            "includeE2ee": false,
            "includeAttachments": true,
        },
    })
}

// ── 1. one shape, round-tripped; anything else refused and nothing erased ─────

#[tokio::test]
async fn put_then_get_round_trips_and_other_shapes_are_refused_without_erasing() {
    let dir = fresh_dir();
    let base = boot(&dir, Vec::new()).await;

    let anon = browser();
    for (method, path) in [
        ("GET", "/admin/assist"),
        ("GET", "/admin/assist/status"),
        ("PUT", "/admin/assist"),
        ("POST", "/admin/assist/kill"),
    ] {
        let req = match method {
            "GET" => anon.get(format!("{base}{path}")),
            "PUT" => anon
                .put(format!("{base}{path}"))
                .json(&full_config("https://x.example/v1")),
            _ => anon
                .post(format!("{base}{path}"))
                .json(&json!({ "on": true })),
        };
        assert_eq!(
            req.send().await.unwrap().status(),
            401,
            "{method} {path} needs an admin session"
        );
    }

    let admin = admin(&base).await;

    // Precondition: nothing is stored, and GET still returns every key.
    assert_eq!(get_config(&admin, &base).await, default_config());

    // The round trip: what was PUT is what GET returns, key for key.
    let full = full_config("https://llm.example.test/v1");
    let put = put_config(&admin, &base, &full).await;
    assert_eq!(
        put.status(),
        200,
        "a whole config in the wire shape is saved"
    );
    let raw = admin
        .get(format!("{base}/admin/assist"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let got: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(got, full, "GET returns exactly what PUT stored");
    assert_eq!(
        raw,
        serde_json::to_string(&full).unwrap(),
        "byte for byte, as serde_json writes the body that was PUT"
    );
    assert!(
        !raw.contains('_'),
        "no snake_case key anywhere on the wire: {raw}"
    );

    // Every refusal below must leave that stored config exactly as it is.
    let with = |edit: &dyn Fn(&mut Value)| {
        let mut v = full.clone();
        edit(&mut v);
        v
    };
    let refused: Vec<(&str, Value)> = vec![
        (
            "the pre-26.20 client's body",
            json!({
                "enabled": true,
                "endpoint_allowlist": ["api.openai.com"],
                "capability_locks": { "assistant": "locked" },
                "data_ceilings": { "include_e2ee": false, "include_attachments": false },
            }),
        ),
        (
            "a snake_case top-level key",
            with(&|v| {
                let grants = v.as_object_mut().unwrap().remove("capabilityGrants");
                v["capability_grants"] = grants.unwrap();
            }),
        ),
        (
            "an extra top-level key",
            with(&|v| v["endpointAllowlist"] = json!([])),
        ),
        (
            "a missing top-level key",
            with(&|v| {
                v.as_object_mut().unwrap().remove("adapter");
            }),
        ),
        (
            "a snake_case key inside dataCeilings",
            with(&|v| {
                let c = v["dataCeilings"].as_object_mut().unwrap();
                let flag = c.remove("includeE2ee").unwrap();
                c.insert("include_e2ee".into(), flag);
            }),
        ),
        (
            "a missing key inside dataCeilings",
            with(&|v| {
                v["dataCeilings"].as_object_mut().unwrap().remove("folders");
            }),
        ),
        (
            "a snake_case key inside adapter",
            with(&|v| {
                let a = v["adapter"].as_object_mut().unwrap();
                let url = a.remove("baseUrl").unwrap();
                a.insert("base_url".into(), url);
            }),
        ),
        (
            "an unknown key inside adapter",
            with(&|v| v["adapter"]["temperature"] = json!(1)),
        ),
        (
            "an unknown adapter kind",
            with(&|v| v["adapter"]["kind"] = json!("OpenAiCompatible")),
        ),
        (
            "an adapter URL that is not http(s)",
            with(&|v| v["adapter"]["baseUrl"] = json!("llm.example.test")),
        ),
        (
            "a capability that does not exist",
            with(&|v| v["capabilityGrants"] = json!(["summarize", "send"])),
        ),
        (
            "a capability listed twice",
            with(&|v| v["capabilityGrants"] = json!(["summarize", "summarize"])),
        ),
        ("an empty object", json!({})),
    ];
    for (what, body) in &refused {
        let r = put_config(&admin, &base, body).await;
        assert_eq!(r.status(), 400, "{what} is refused");
        let err: Value = r.json().await.unwrap();
        assert!(
            err["error"].as_str().is_some_and(|m| !m.is_empty()),
            "{what}: the refusal says why: {err}"
        );
        assert_eq!(
            get_config(&admin, &base).await,
            full,
            "{what}: the stored config is untouched"
        );
    }

    // An adapter may leave out the fields that have defaults; GET fills them in.
    let short = with(&|v| {
        v["adapter"] = json!({ "kind": "anthropic", "apiKey": "k" });
    });
    assert_eq!(put_config(&admin, &base, &short).await.status(), 200);
    let adapter = &get_config(&admin, &base).await["adapter"];
    for key in [
        "kind",
        "baseUrl",
        "apiKey",
        "model",
        "anthropicVersion",
        "maxTokens",
    ] {
        assert!(
            adapter.get(key).is_some(),
            "GET returns adapter.{key}: {adapter}"
        );
    }

    // `null` clears the adapter and is itself round-tripped.
    let cleared = with(&|v| v["adapter"] = Value::Null);
    assert_eq!(put_config(&admin, &base, &cleared).await.status(), 200);
    assert_eq!(get_config(&admin, &base).await, cleared);
}

// ── 2. the kill switch stops the running gateway ─────────────────────────────

/// A loopback stand-in for an OpenAI-compatible endpoint that counts the
/// transcription requests reaching it.
async fn spawn_endpoint() -> (String, Arc<AtomicUsize>) {
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&hits);
    let app = axum::Router::new().route(
        "/v1/audio/transcriptions",
        axum::routing::post(move || {
            let counter = Arc::clone(&counter);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                axum::Json(json!({ "text": "transcribed by the endpoint" }))
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}/v1"), hits)
}

async fn spawn_mock_jmap() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

async fn transcribe(c: &reqwest::Client, base: &str) -> reqwest::Response {
    c.post(format!("{base}/api/assist/transcribe"))
        .json(&json!({ "audioBase64": "AAAA", "mime": "audio/webm" }))
        .send()
        .await
        .unwrap()
}

async fn kill(admin: &reqwest::Client, base: &str, body: &Value) -> reqwest::Response {
    admin
        .post(format!("{base}/admin/assist/kill"))
        .json(body)
        .send()
        .await
        .unwrap()
}

/// Rows in `assist_audit`, read straight from the server's database file.
async fn audit_rows(dir: &std::path::Path) -> i64 {
    let options = sqlx::sqlite::SqliteConnectOptions::new().filename(dir.join("mw.db"));
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let n: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM assist_audit")
        .fetch_one(&pool)
        .await
        .unwrap();
    pool.close().await;
    n.0
}

/// The audit sink writes from a spawned task; give it time to land before counting.
async fn settled_audit_rows(dir: &std::path::Path, at_least: i64) -> i64 {
    for _ in 0..50 {
        if audit_rows(dir).await >= at_least {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
    }
    audit_rows(dir).await
}

#[tokio::test]
async fn the_kill_switch_stops_the_running_gateway_and_releasing_it_resumes() {
    let (endpoint, hits) = spawn_endpoint().await;
    let mock = spawn_mock_jmap().await;
    let dir = fresh_dir();
    let mut config = full_config(&endpoint);
    config["dataCeilings"]["accounts"] = json!([]);

    // First boot: no row. Saving a config does not change the gateway that is
    // already running, and the status says so.
    let first = boot(&dir, vec![mock.clone()]).await;
    let first_admin = admin(&first).await;
    assert_eq!(
        status(&first_admin, &first).await,
        json!({ "enabled": false, "running": false, "endpointHost": null, "restartPending": false }),
        "precondition: a fresh deployment has Assist off and nothing pending"
    );
    let saved = put_config(&first_admin, &first, &config).await;
    assert_eq!(saved.status(), 200);
    assert_eq!(
        saved.json::<Value>().await.unwrap(),
        json!({ "enabled": true, "running": false, "endpointHost": null, "restartPending": true }),
        "a saved config waits for the restart, and PUT says so"
    );
    let user = browser();
    let login = user
        .post(format!("{first}/api/login"))
        .json(&json!({ "jmapUrl": mock, "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "mailbox login");
    assert_eq!(
        transcribe(&user, &first).await.status(),
        404,
        "the gateway built before the save is still off"
    );
    assert_eq!(hits.load(Ordering::SeqCst), 0);

    // Restart: the gateway is built from the saved row.
    let base = boot(&dir, vec![mock.clone()]).await;
    let admin = admin(&base).await;
    let user = browser();
    let login = user
        .post(format!("{base}/api/login"))
        .json(&json!({ "jmapUrl": mock, "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "mailbox login after the restart");
    let endpoint_host = endpoint
        .trim_start_matches("http://")
        .trim_end_matches("/v1");
    assert_eq!(
        status(&admin, &base).await,
        json!({ "enabled": true, "running": true, "endpointHost": endpoint_host, "restartPending": false }),
    );

    // Precondition for the kill: a request goes out, and is audited.
    let ok = transcribe(&user, &base).await;
    assert_eq!(ok.status(), 200, "Assist works before the kill switch");
    assert_eq!(
        ok.json::<Value>().await.unwrap()["text"],
        json!("transcribed by the endpoint")
    );
    assert_eq!(hits.load(Ordering::SeqCst), 1, "it reached the endpoint");
    assert_eq!(settled_audit_rows(&dir, 1).await, 1, "and was audited");

    // A kill request that does not say which way is refused, and changes nothing.
    for (what, r) in [
        (
            "no body",
            admin
                .post(format!("{base}/admin/assist/kill"))
                .send()
                .await
                .unwrap(),
        ),
        ("an empty object", kill(&admin, &base, &json!({})).await),
        (
            "an extra key",
            kill(&admin, &base, &json!({ "on": true, "why": "x" })).await,
        ),
    ] {
        assert_eq!(r.status(), 400, "{what} is refused");
    }
    assert_eq!(status(&admin, &base).await["running"], json!(true));

    // Kill: same process, no restart.
    let killed = kill(&admin, &base, &json!({ "on": true })).await;
    assert_eq!(killed.status(), 200);
    assert_eq!(
        killed.json::<Value>().await.unwrap(),
        json!({ "enabled": false, "running": false, "endpointHost": null, "restartPending": false }),
        "off is in effect at once, so nothing is pending"
    );
    for _ in 0..3 {
        let refused = transcribe(&user, &base).await;
        assert_eq!(refused.status(), 404, "the running gateway refuses");
        assert_eq!(
            refused.json::<Value>().await.unwrap(),
            json!({ "error": "assist disabled" })
        );
    }
    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "nothing reached the endpoint after the kill"
    );
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert_eq!(
        audit_rows(&dir).await,
        1,
        "no audit row claims a refused request left"
    );
    // The kill kept the rest of the configuration.
    let mut expected = config.clone();
    expected["enabled"] = json!(false);
    assert_eq!(get_config(&admin, &base).await, expected);

    // Release: the same gateway answers again.
    let released = kill(&admin, &base, &json!({ "on": false })).await;
    assert_eq!(released.status(), 200);
    assert_eq!(
        released.json::<Value>().await.unwrap(),
        json!({ "enabled": true, "running": true, "endpointHost": endpoint_host, "restartPending": false }),
    );
    assert_eq!(transcribe(&user, &base).await.status(), 200);
    assert_eq!(hits.load(Ordering::SeqCst), 2);

    // Saving `enabled: false` is the same stop, through the other route.
    let mut off = config.clone();
    off["enabled"] = json!(false);
    assert_eq!(put_config(&admin, &base, &off).await.status(), 200);
    assert_eq!(transcribe(&user, &base).await.status(), 404);
    assert_eq!(hits.load(Ordering::SeqCst), 2);

    // Releasing on the first server — whose gateway was built with Assist off —
    // cannot start it, and the answer says a restart is needed instead of
    // claiming it is running.
    let late = kill(&first_admin, &first, &json!({ "on": false })).await;
    assert_eq!(
        late.json::<Value>().await.unwrap(),
        json!({ "enabled": true, "running": false, "endpointHost": null, "restartPending": true }),
    );
}
