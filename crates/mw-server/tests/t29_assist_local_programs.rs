//! t29-e14 — a `local-process` Assist adapter runs only a program the operator
//! listed in `MW_ASSIST_LOCAL_PROGRAMS`.
//!
//! Before 26.20 `PUT /admin/assist` accepted any `local-process` adapter, and the
//! gateway spawned its `program` for every Assist request: any admin session could
//! make the server run any binary on the host. Now the panel can only pick from a
//! list the operator sets where the server starts, a stored adapter the list does
//! not name is not started, and the admin status says why.
//!
//! Every leg drives the real router over HTTP. The program that runs is a real
//! process (`cmd /C type` on Windows, `cat` elsewhere) printing a canned reply, so
//! "the helper ran" is measured as its text arriving in the SSE stream.
//!
//! The second test is the admin gate these routes share: it follows a renamed
//! admin cookie (`MW_ADMIN_SESSION_COOKIE`) like every other admin gate.
//!
//! Both tests change process environment, so they live in their own binary and take
//! turns ([`turn`]): nothing else reads the variables while they change.
//!
//! Run:
//!   cargo test -p mw-server --test t29_assist_local_programs --locked -- --test-threads=1

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};

mod common;
use common::test_db;

const KEY_HEX: &str = "29e14a1b2c3d4e5f60718293a4b5c6d729e14a1b2c3d4e5f60718293a4b5c6d7";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T29</div>";
const ENV: &str = "MW_ASSIST_LOCAL_PROGRAMS";
const HELPER_REPLY: &str = "reply from the listed local program";

// ── harness ──────────────────────────────────────────────────────────────────

fn admin_v6() -> V6Config {
    V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    }
}

fn fresh_dir() -> PathBuf {
    let dir = test_db::unique_dir("mw-t29-e14-local");
    let web = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    dir
}

/// Boot a proxy-mode server over the database in `dir`. Booting twice over one
/// `dir` is a restart: the Assist gateway is built once per boot.
async fn boot(dir: &Path, upstream: &str) -> String {
    let config = AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(dir.join("web")),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(vec![upstream.to_string()]),
            ..SecurityConfig::default()
        },
    };
    let app = mw_server::build_app_full(config, admin_v6())
        .await
        .expect("build_app_full")
        .0;
    // Sign-in pacing is not what this test is about (see common::auth).
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

async fn spawn_mock_jmap() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
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

async fn mailbox(base: &str, _upstream: &str) -> reqwest::Client {
    let c = browser();
    let r = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "mailbox login");
    c
}

async fn get_json(c: &reqwest::Client, url: String) -> Value {
    let r = c.get(&url).send().await.unwrap();
    assert_eq!(r.status(), 200, "GET {url}");
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

/// `POST /api/assist/invoke` for `summarize` with a bare prompt; the status and the
/// raw SSE body.
async fn invoke(user: &reqwest::Client, base: &str) -> (u16, String) {
    let r = user
        .post(format!("{base}/api/assist/invoke"))
        .json(&json!({ "capability": "summarize", "input": { "prompt": "say hello" } }))
        .send()
        .await
        .unwrap();
    let status = r.status().as_u16();
    let body = tokio::time::timeout(Duration::from_secs(30), r.text())
        .await
        .expect("the reply terminates")
        .unwrap();
    (status, body)
}

/// A program that prints `reply` as the adapter's JSON answer, whatever it is sent:
/// the absolute path to spawn and its arguments.
fn helper(dir: &Path) -> (String, Vec<String>) {
    let reply = dir.join("helper-reply.json");
    std::fs::write(
        &reply,
        json!({ "content": HELPER_REPLY, "text": HELPER_REPLY }).to_string(),
    )
    .unwrap();
    let reply = reply.to_string_lossy().into_owned();
    if cfg!(windows) {
        let cmd =
            std::env::var("ComSpec").unwrap_or_else(|_| r"C:\Windows\System32\cmd.exe".into());
        (cmd, vec!["/C".into(), "type".into(), reply])
    } else {
        ("/bin/cat".into(), vec![reply])
    }
}

fn local_config(program: &str, args: &[String]) -> Value {
    json!({
        "enabled": true,
        "adapter": { "kind": "local-process", "program": program, "args": args },
        "capabilityGrants": ["summarize"],
        "dataCeilings": {
            "accounts": [],
            "folders": [],
            "includeE2ee": false,
            "includeAttachments": false,
        },
    })
}

/// One test at a time: each changes process environment that a boot reads.
async fn turn() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    LOCK.lock().await
}

fn set_var(key: &str, value: Option<&str>) {
    // SAFETY: only while [`turn`]'s guard is held; the tests in this binary take
    // turns, and nothing else in it reads process environment meanwhile.
    match value {
        Some(v) => unsafe { std::env::set_var(key, v) },
        None => unsafe { std::env::remove_var(key) },
    }
}

fn set_env(value: Option<&str>) {
    set_var(ENV, value);
}

// ── the legs ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_local_program_runs_only_when_the_operator_lists_it() {
    let _turn = turn().await;
    let mock = spawn_mock_jmap().await;
    let dir = fresh_dir();
    let (program, args) = helper(&dir);
    let other = if cfg!(windows) {
        r"C:\Windows\System32\whoami.exe"
    } else {
        "/usr/bin/id"
    };
    let config = local_config(&program, &args);

    // 1. Variable unset: a local-process adapter cannot be configured at all.
    set_env(None);
    let base = boot(&dir, &mock).await;
    let a = admin(&base).await;
    let stored_before = get_json(&a, format!("{base}/admin/assist")).await;
    let refused = put_config(&a, &base, &config).await;
    assert_eq!(refused.status(), 422, "unset ⇒ refused");
    let why = refused.json::<Value>().await.unwrap()["error"].to_string();
    assert!(why.contains(ENV), "the refusal names the variable: {why}");
    assert_eq!(
        get_json(&a, format!("{base}/admin/assist")).await,
        stored_before,
        "nothing was written"
    );
    let st = get_json(&a, format!("{base}/admin/assist/status")).await;
    assert_eq!(st["localPrograms"], json!([]));
    assert_eq!(st["stoppedReason"], json!(null));

    // 2. Variable set: the listed program is accepted, any other is refused.
    set_env(Some(&format!("{program}, {other}")));
    let st = get_json(&a, format!("{base}/admin/assist/status")).await;
    assert_eq!(st["localPrograms"], json!([program, other]));
    let unlisted = local_config(
        if cfg!(windows) {
            r"C:\Windows\System32\notepad.exe"
        } else {
            "/bin/sh"
        },
        &[],
    );
    let refused = put_config(&a, &base, &unlisted).await;
    assert_eq!(
        refused.status(),
        422,
        "a program not on the list is refused"
    );
    let why = refused.json::<Value>().await.unwrap()["error"].to_string();
    assert!(why.contains(ENV), "{why}");
    assert_eq!(
        get_json(&a, format!("{base}/admin/assist")).await,
        stored_before,
        "nothing was written"
    );
    let saved = put_config(&a, &base, &config).await;
    assert_eq!(saved.status(), 200, "the listed program is accepted");
    assert_eq!(
        get_json(&a, format!("{base}/admin/assist")).await,
        config,
        "and stored as sent"
    );

    // 3. Restart with the variable still set: the helper answers a chat request.
    let base = boot(&dir, &mock).await;
    let a = admin(&base).await;
    let st = get_json(&a, format!("{base}/admin/assist/status")).await;
    assert_eq!(st["running"], json!(true), "{st}");
    assert_eq!(st["endpointHost"], json!("local-process"));
    assert_eq!(st["stoppedReason"], json!(null));
    let user = mailbox(&base, &mock).await;
    let shown = get_json(&user, format!("{base}/api/assist/config")).await;
    assert_eq!(shown["availability"], json!("enabled"), "{shown}");
    let (code, body) = invoke(&user, &base).await;
    assert_eq!(code, 200, "{body}");
    assert!(
        body.contains(HELPER_REPLY),
        "the listed program ran and its output came back: {body}"
    );

    // 4. Restart with the variable unset: the stored adapter does not run.
    set_env(None);
    let base = boot(&dir, &mock).await;
    let a = admin(&base).await;
    assert_eq!(
        get_json(&a, format!("{base}/admin/assist")).await,
        config,
        "the stored row is kept"
    );
    let st = get_json(&a, format!("{base}/admin/assist/status")).await;
    assert_eq!(
        st,
        json!({
            "enabled": true,
            "running": false,
            "endpointHost": null,
            "restartPending": false,
            "stoppedReason": "local-program-not-allowed",
            "localPrograms": [],
        }),
        "stopped, with the reason"
    );
    let user = mailbox(&base, &mock).await;
    let shown = get_json(&user, format!("{base}/api/assist/config")).await;
    assert_eq!(shown["availability"], json!("disabled"), "{shown}");
    assert_eq!(shown["capabilities"], json!([]));
    let (code, body) = invoke(&user, &base).await;
    assert_eq!(code, 404, "{body}");
    assert!(!body.contains(HELPER_REPLY), "{body}");

    // A list that names a different program does not start it either.
    set_env(Some(other));
    let base = boot(&dir, &mock).await;
    let a = admin(&base).await;
    let st = get_json(&a, format!("{base}/admin/assist/status")).await;
    assert_eq!(st["running"], json!(false), "{st}");
    assert_eq!(st["stoppedReason"], json!("local-program-not-allowed"));
    set_env(None);
}

// ── the admin gate follows MW_ADMIN_SESSION_COOKIE ───────────────────────────

#[tokio::test]
async fn the_assist_admin_routes_accept_the_renamed_admin_cookie_only() {
    let _turn = turn().await;
    set_var("MW_ADMIN_SESSION_COOKIE", Some("corp_admin"));
    let mock = spawn_mock_jmap().await;
    let dir = fresh_dir();
    let base = boot(&dir, &mock).await;
    let login = reqwest::Client::new()
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "admin login");
    let set_cookie = login
        .headers()
        .get(reqwest::header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        set_cookie.starts_with("corp_admin="),
        "precondition: login sets the renamed cookie: {set_cookie}"
    );
    let token = set_cookie
        .split(';')
        .next()
        .and_then(|kv| kv.split_once('='))
        .map(|(_, v)| v.to_string())
        .unwrap();
    let status = |path: &str, cookie: String| {
        let req = reqwest::Client::new()
            .get(format!("{base}{path}"))
            .header(reqwest::header::COOKIE, cookie);
        async move { req.send().await.unwrap().status().as_u16() }
    };
    // The routes `v7_mount::require_admin` gates: the Assist routes, and the
    // allowlist and egress routers merged beside them.
    for path in [
        "/admin/assist",
        "/admin/assist/status",
        "/admin/egress/proxies",
    ] {
        assert_eq!(
            status(path, format!("corp_admin={token}")).await,
            200,
            "{path} accepts the renamed cookie"
        );
        assert_eq!(
            status(path, format!("mw_admin_session={token}")).await,
            401,
            "{path} ignores the default name"
        );
    }
    set_var("MW_ADMIN_SESSION_COOKIE", None);
}
