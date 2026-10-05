//! t28-e7 (audit §13 row 34) — the doveadm, poppassd and HMAC-webhook password
//! backends are selectable, and a forced password change can be completed.
//!
//! `mw-passwd` has carried all three backends since V7, each with its own unit
//! tests, but `build_passwd_backend` matched only `ldap3062` and fell through to
//! `Local` for everything else — under a doc comment saying the other three were
//! "constructed the same way when configured". So `MW_PASSWD_BACKEND=webhook`
//! silently gave a deployment the local-hash backend.
//!
//! Three kinds of leg:
//!   * the builder, given settings, returns the backend they select, and refuses
//!     an unusable setting with an error naming the variable;
//!   * each built backend, asked to change a password, performs its protocol
//!     against a loopback stand-in — so the settings are shown to reach the
//!     backend, not only to select it;
//!   * the real router: `force_password_change` set through the admin route holds
//!     the account, `POST /api/password` through the webhook backend changes the
//!     password at the stand-in, and the hold is released. A server booted with a
//!     misconfigured backend does not come up.
//!
//! Run:
//!   cargo test -p mw-server --test t28_passwd_backends --locked -- --test-threads=1

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex, OnceLock};

use base64::Engine as _;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{Mutex, MutexGuard};

use mw_passwd::{BackendKind, Ctx, PasswordError, Secret};
use mw_server::v7_mount::{PasswdBackendConfigError, passwd_backend_from};
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::{Credentials, ServerKey, Store};

mod common;
use common::test_db;

const KEY_HEX: &str = "28e7b1b2c3d4e5f60718293a4b5c6d7e28e7b1b2c3d4e5f60718293a4b5c6d7e";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T28</div>";
const WEBHOOK_SECRET: &str = "t28-webhook-secret-0123456789";
const NEW_PASSWORD: &str = "a-new-long-password";

/// Every variable the builder reads. The router legs clear them all first.
const PASSWD_VARS: [&str; 12] = [
    "MW_PASSWD_BACKEND",
    "MW_PASSWD_LDAP_URL",
    "MW_PASSWD_LDAP_STARTTLS",
    "MW_PASSWD_LDAP_BIND_DN",
    "MW_PASSWD_LDAP_BIND_PW",
    "MW_PASSWD_DOVECOT_URL",
    "MW_PASSWD_DOVECOT_API_KEY",
    "MW_PASSWD_DOVECOT_COMMAND",
    "MW_PASSWD_POPPASSD_HOST",
    "MW_PASSWD_POPPASSD_PORT",
    "MW_PASSWD_WEBHOOK_URL",
    "MW_PASSWD_WEBHOOK_SECRET",
];

// ── builder harness ──────────────────────────────────────────────────────────

fn settings(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

async fn memory_store() -> Store {
    Store::open_in_memory(ServerKey::generate()).await.unwrap()
}

/// Build from a settings map, the way the server builds from its environment.
fn build(
    store: &Store,
    pairs: &[(&str, &str)],
) -> Result<Arc<dyn mw_passwd::PasswordChangeBackend>, PasswdBackendConfigError> {
    let map = settings(pairs);
    passwd_backend_from(store, &|key| map.get(key).cloned())
}

fn build_err(store: &Store, pairs: &[(&str, &str)]) -> PasswdBackendConfigError {
    match build(store, pairs) {
        Ok(b) => panic!("{pairs:?} built a {:?} backend", b.kind()),
        Err(e) => e,
    }
}

// ── 1. selection ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn each_backend_name_selects_that_backend() {
    let store = memory_store().await;
    let cases: Vec<(&[(&str, &str)], BackendKind)> = vec![
        // Precondition: unset and `local` are the local backend, as before.
        (&[], BackendKind::Local),
        (&[("MW_PASSWD_BACKEND", "local")], BackendKind::Local),
        (
            &[
                ("MW_PASSWD_BACKEND", "ldap3062"),
                ("MW_PASSWD_LDAP_URL", "ldap://127.0.0.1:1"),
            ],
            BackendKind::Ldap3062,
        ),
        (
            &[
                ("MW_PASSWD_BACKEND", "dovecot"),
                ("MW_PASSWD_DOVECOT_URL", "http://127.0.0.1:1/doveadm/v1"),
                ("MW_PASSWD_DOVECOT_API_KEY", "key"),
            ],
            BackendKind::DovecotHttp,
        ),
        (
            &[
                ("MW_PASSWD_BACKEND", "poppassd"),
                ("MW_PASSWD_POPPASSD_HOST", "127.0.0.1"),
            ],
            BackendKind::Poppassd,
        ),
        (
            &[
                ("MW_PASSWD_BACKEND", "webhook"),
                ("MW_PASSWD_WEBHOOK_URL", "https://hooks.example.test/pw"),
                ("MW_PASSWD_WEBHOOK_SECRET", WEBHOOK_SECRET),
            ],
            BackendKind::WebhookHmac,
        ),
    ];
    for (pairs, kind) in cases {
        let backend = build(&store, pairs).unwrap_or_else(|e| panic!("{pairs:?}: {e}"));
        assert_eq!(backend.kind(), kind, "{pairs:?}");
    }
}

#[tokio::test]
async fn an_unusable_setting_is_an_error_naming_the_variable() {
    let store = memory_store().await;
    const DOVECOT_URL: (&str, &str) = ("MW_PASSWD_DOVECOT_URL", "http://127.0.0.1:1/doveadm/v1");
    const WEBHOOK_URL: (&str, &str) = ("MW_PASSWD_WEBHOOK_URL", "https://hooks.example.test/pw");
    let cases: Vec<(&[(&str, &str)], &str)> = vec![
        // The three names that used to fall through to `local`, misspelt.
        (&[("MW_PASSWD_BACKEND", "dovecott")], "MW_PASSWD_BACKEND"),
        (&[("MW_PASSWD_BACKEND", "Webhook")], "MW_PASSWD_BACKEND"),
        (&[("MW_PASSWD_BACKEND", "ldap3062")], "MW_PASSWD_LDAP_URL"),
        (&[("MW_PASSWD_BACKEND", "dovecot")], "MW_PASSWD_DOVECOT_URL"),
        (
            &[("MW_PASSWD_BACKEND", "dovecot"), DOVECOT_URL],
            "MW_PASSWD_DOVECOT_API_KEY",
        ),
        (
            &[
                ("MW_PASSWD_BACKEND", "dovecot"),
                ("MW_PASSWD_DOVECOT_URL", "mail.example.test:8080"),
                ("MW_PASSWD_DOVECOT_API_KEY", "key"),
            ],
            "MW_PASSWD_DOVECOT_URL",
        ),
        (
            &[("MW_PASSWD_BACKEND", "poppassd")],
            "MW_PASSWD_POPPASSD_HOST",
        ),
        (
            &[
                ("MW_PASSWD_BACKEND", "poppassd"),
                ("MW_PASSWD_POPPASSD_HOST", "127.0.0.1"),
                ("MW_PASSWD_POPPASSD_PORT", "poppassd"),
            ],
            "MW_PASSWD_POPPASSD_PORT",
        ),
        (
            &[
                ("MW_PASSWD_BACKEND", "poppassd"),
                ("MW_PASSWD_POPPASSD_HOST", "127.0.0.1"),
                ("MW_PASSWD_POPPASSD_PORT", "0"),
            ],
            "MW_PASSWD_POPPASSD_PORT",
        ),
        (&[("MW_PASSWD_BACKEND", "webhook")], "MW_PASSWD_WEBHOOK_URL"),
        (
            &[("MW_PASSWD_BACKEND", "webhook"), WEBHOOK_URL],
            "MW_PASSWD_WEBHOOK_SECRET",
        ),
        (
            &[
                ("MW_PASSWD_BACKEND", "webhook"),
                WEBHOOK_URL,
                ("MW_PASSWD_WEBHOOK_SECRET", "short"),
            ],
            "MW_PASSWD_WEBHOOK_SECRET",
        ),
        (
            &[
                ("MW_PASSWD_BACKEND", "webhook"),
                ("MW_PASSWD_WEBHOOK_URL", "ftp://hooks.example.test/pw"),
                ("MW_PASSWD_WEBHOOK_SECRET", WEBHOOK_SECRET),
            ],
            "MW_PASSWD_WEBHOOK_URL",
        ),
    ];
    for (pairs, var) in cases {
        let message = build_err(&store, pairs).to_string();
        assert!(message.contains(var), "{pairs:?}: {message:?} names {var}");
        for (_, value) in pairs.iter().filter(|(k, _)| k.ends_with("SECRET")) {
            assert!(
                !message.contains(value),
                "the message does not repeat a secret: {message:?}"
            );
        }
    }
    // An empty value is an unset one, not a present one.
    assert_eq!(
        build(&store, &[("MW_PASSWD_BACKEND", "")]).unwrap().kind(),
        BackendKind::Local
    );
}

// ── 2. each built backend performs its protocol ──────────────────────────────

/// One request: its headers and its body.
type Request = (axum::http::HeaderMap, Vec<u8>);

/// What a loopback HTTP stand-in received.
#[derive(Clone, Default)]
struct Received(Arc<StdMutex<Vec<Request>>>);

impl Received {
    fn all(&self) -> Vec<Request> {
        self.0.lock().unwrap().clone()
    }
}

/// An HTTP endpoint at `path` that records each POST and answers `reply`.
async fn spawn_http(path: &'static str, reply: Value) -> (String, Received) {
    let received = Received::default();
    let sink = received.clone();
    let app = axum::Router::new().route(
        path,
        axum::routing::post(
            move |headers: axum::http::HeaderMap, body: axum::body::Bytes| {
                let sink = sink.clone();
                let reply = reply.clone();
                async move {
                    sink.0.lock().unwrap().push((headers, body.to_vec()));
                    axum::Json(reply)
                }
            },
        ),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}{path}"), received)
}

/// A session row whose sealed credentials carry `password`, as a login leaves it.
async fn seed_session(store: &Store, account: &str, user: &str, password: &str) {
    store
        .create_session(
            account,
            user,
            "http://mock",
            "http://mock",
            &Credentials {
                username: user.into(),
                password: password.into(),
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn the_dovecot_backend_posts_the_doveadm_command_with_the_configured_key() {
    let store = memory_store().await;
    let (url, received) = spawn_http("/doveadm/v1", json!([["doveadmResponse", [], "tag"]])).await;
    let backend = build(
        &store,
        &[
            ("MW_PASSWD_BACKEND", "dovecot"),
            ("MW_PASSWD_DOVECOT_URL", &url),
            ("MW_PASSWD_DOVECOT_API_KEY", "doveadm-key-t28"),
            ("MW_PASSWD_DOVECOT_COMMAND", "mwSetPassword"),
        ],
    )
    .unwrap();
    seed_session(&store, "acct-d", "dana@example.org", "current-password").await;
    let ctx = Ctx::new("acct-d", "dana@example.org");

    // doveadm does not check the old password, so the wiring does: a wrong one is
    // refused before any request leaves.
    let wrong = backend
        .change(&ctx, Secret::new("not-it"), Secret::new(NEW_PASSWORD))
        .await;
    assert!(
        matches!(wrong, Err(PasswordError::WrongCurrent)),
        "{wrong:?}"
    );
    assert!(
        received.all().is_empty(),
        "nothing was sent for a wrong current password"
    );

    let outcome = backend
        .change(
            &ctx,
            Secret::new("current-password"),
            Secret::new(NEW_PASSWORD),
        )
        .await
        .expect("the change goes through");
    assert!(outcome.changed);
    let got = received.all();
    assert_eq!(got.len(), 1, "one doveadm request");
    let (headers, body) = &got[0];
    let key = base64::engine::general_purpose::STANDARD.encode("doveadm-key-t28");
    assert_eq!(
        headers.get("authorization").unwrap().to_str().unwrap(),
        format!("X-Dovecot-API {key}")
    );
    let body: Value = serde_json::from_slice(body).unwrap();
    assert_eq!(
        body,
        json!([[
            "mwSetPassword",
            { "user": "dana@example.org", "password": NEW_PASSWORD },
            "mw-passwd-acct-d"
        ]]),
        "the configured command, the session's username, the new password"
    );
}

/// A poppassd stand-in that accepts `current` for `pass` and records every line.
async fn spawn_poppassd(current: &'static str) -> (u16, Arc<StdMutex<Vec<String>>>) {
    let lines = Arc::new(StdMutex::new(Vec::new()));
    let sink = Arc::clone(&lines);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                return;
            };
            let sink = Arc::clone(&sink);
            tokio::spawn(async move {
                let mut io = BufReader::new(stream);
                let _ = io.get_mut().write_all(b"200 poppassd ready\r\n").await;
                let mut line = String::new();
                loop {
                    line.clear();
                    if io.read_line(&mut line).await.unwrap_or(0) == 0 {
                        return;
                    }
                    let text = line.trim_end().to_string();
                    sink.lock().unwrap().push(text.clone());
                    let reply: &[u8] = match text.split_once(' ') {
                        Some(("pass", given)) if given != current => b"500 wrong password\r\n",
                        _ => b"200 ok\r\n",
                    };
                    let _ = io.get_mut().write_all(reply).await;
                    if text == "quit" {
                        return;
                    }
                }
            });
        }
    });
    (port, lines)
}

#[tokio::test]
async fn the_poppassd_backend_runs_the_dialogue_on_the_configured_port() {
    let store = memory_store().await;
    let (port, lines) = spawn_poppassd("current-password").await;
    let backend = build(
        &store,
        &[
            ("MW_PASSWD_BACKEND", "poppassd"),
            ("MW_PASSWD_POPPASSD_HOST", "127.0.0.1"),
            ("MW_PASSWD_POPPASSD_PORT", &port.to_string()),
        ],
    )
    .unwrap();
    let ctx = Ctx::new("acct-p", "pat@example.org");

    let wrong = backend
        .change(&ctx, Secret::new("not-it"), Secret::new(NEW_PASSWORD))
        .await;
    assert!(
        matches!(wrong, Err(PasswordError::WrongCurrent)),
        "{wrong:?}"
    );
    assert!(
        !lines
            .lock()
            .unwrap()
            .iter()
            .any(|l| l.starts_with("newpass")),
        "a rejected current password never reaches newpass"
    );
    lines.lock().unwrap().clear();

    backend
        .change(
            &ctx,
            Secret::new("current-password"),
            Secret::new(NEW_PASSWORD),
        )
        .await
        .expect("the change goes through");
    assert_eq!(
        *lines.lock().unwrap(),
        vec![
            "user pat@example.org".to_string(),
            "pass current-password".to_string(),
            format!("newpass {NEW_PASSWORD}"),
            "quit".to_string(),
        ]
    );
}

#[tokio::test]
async fn the_webhook_backend_signs_the_body_with_the_configured_secret() {
    let store = memory_store().await;
    let (url, received) = spawn_http("/hook", json!({ "ok": true })).await;
    let backend = build(
        &store,
        &[
            ("MW_PASSWD_BACKEND", "webhook"),
            ("MW_PASSWD_WEBHOOK_URL", &url),
            ("MW_PASSWD_WEBHOOK_SECRET", WEBHOOK_SECRET),
        ],
    )
    .unwrap();
    // No stored session: the operator's `mailwoman password` case, let through.
    backend
        .change(
            &Ctx::new("acct-w", "wim@example.org"),
            Secret::new("unused"),
            Secret::new(NEW_PASSWORD),
        )
        .await
        .expect("the change goes through");
    let got = received.all();
    assert_eq!(got.len(), 1);
    let (headers, body) = &got[0];
    let signature = headers.get("x-signature").unwrap().to_str().unwrap();
    assert!(
        mw_passwd::verify_signature(WEBHOOK_SECRET.as_bytes(), body, signature),
        "the signature verifies under the configured secret"
    );
    assert!(
        !mw_passwd::verify_signature(b"another-secret-0123456789", body, signature),
        "and under no other"
    );
}

// ── 3. the real router ───────────────────────────────────────────────────────

/// Serialises the legs that set process environment, and starts each from a
/// clean slate.
async fn serial() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK.get_or_init(Default::default).lock().await;
    // SAFETY: the guard serialises every mutation and every dependent read here.
    unsafe {
        for k in PASSWD_VARS {
            std::env::remove_var(k);
        }
    }
    guard
}

fn set_env(pairs: &[(&str, &str)]) {
    // SAFETY: only called while `serial()`'s guard is held.
    unsafe {
        for (k, v) in pairs {
            std::env::set_var(k, v);
        }
    }
}

fn admin_v6() -> V6Config {
    V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    }
}

fn app_config(upstreams: Vec<String>) -> AppConfig {
    let dir = test_db::unique_dir("mw-t28-e7-passwd");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(upstreams),
            ..SecurityConfig::default()
        },
    }
}

async fn serve(app: axum::Router) -> String {
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

async fn me(c: &reqwest::Client, base: &str) -> Value {
    let r = c.get(format!("{base}/api/me")).send().await.unwrap();
    assert_eq!(r.status(), 200, "GET /api/me");
    r.json().await.unwrap()
}

/// An empty JMAP request: reaches the handler behind `authed` and needs no data.
async fn jmap(c: &reqwest::Client, base: &str) -> reqwest::Response {
    c.post(format!("{base}/jmap/api"))
        .json(&json!({ "using": ["urn:ietf:params:jmap:core"], "methodCalls": [] }))
        .send()
        .await
        .unwrap()
}

/// The `forcePasswordChange` flag as the admin panel's user list reports it.
async fn panel_force_flag(admin: &reqwest::Client, base: &str, id: &str) -> Value {
    let users: Value = admin
        .get(format!("{base}/admin/users"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    users
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["accountId"] == id)
        .unwrap_or_else(|| panic!("{id} is not in the panel user list: {users}"))["flags"]
        ["forcePasswordChange"]
        .clone()
}

#[tokio::test]
async fn a_forced_password_change_completes_through_the_webhook_backend() {
    let _g = serial().await;
    let (hook, received) = spawn_http("/hook", json!({ "ok": true })).await;
    set_env(&[
        ("MW_PASSWD_BACKEND", "webhook"),
        ("MW_PASSWD_WEBHOOK_URL", &hook),
        ("MW_PASSWD_WEBHOOK_SECRET", WEBHOOK_SECRET),
    ]);
    let mock = spawn_mock_jmap().await;
    let app = mw_server::build_app_full(app_config(vec![mock.clone()]), admin_v6())
        .await
        .expect("build_app_full")
        .0;
    let base = serve(app).await;
    let (user, pass) = (mw_mock_jmap::USER, mw_mock_jmap::PASS);

    let c = browser();
    let login = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "jmapUrl": mock, "username": user, "password": pass }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "mailbox login");
    let account_id = login.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .unwrap()
        .to_string();

    // Precondition: not held.
    assert!(me(&c, &base).await.get("passwordChangeRequired").is_none());
    assert_eq!(jmap(&c, &base).await.status(), 200);

    // The admin sets the flag, the way the panel does.
    let admin = browser();
    let r = admin
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "admin login");
    // Provisioned, so the panel's user list shows the flag.
    let (local, domain) = user.split_once('@').unwrap();
    let r = admin
        .post(format!("{base}/admin/users"))
        .json(&json!({
            "domain": domain, "username": local,
            "quota": { "bytesLimit": 0, "msgLimit": 0 },
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204, "provision the user");
    assert_eq!(panel_force_flag(&admin, &base, user).await, json!(false));
    let r = admin
        .put(format!("{base}/admin/users/{user}/flags"))
        .json(&json!({
            "zeroAccess": false,
            "forcePasswordChange": true,
            "remoteCacheWipe": false,
            "disabled": false,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204, "the panel's flag write is accepted");

    assert_eq!(panel_force_flag(&admin, &base, user).await, json!(true));

    // Held: the mailbox is closed and the session says why.
    assert_eq!(me(&c, &base).await["passwordChangeRequired"], json!(true));
    assert_eq!(jmap(&c, &base).await.status(), 403, "held");
    let policy: Value = c
        .get(format!("{base}/api/password/policy"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(policy["forceChange"], json!(true), "{policy}");

    let change = |old: &'static str, new: &'static str| {
        let req = c
            .post(format!("{base}/api/password"))
            .json(&json!({ "oldPassword": old, "newPassword": new }));
        async move { req.send().await.unwrap() }
    };

    // A wrong current password does not reach the webhook and does not release.
    let wrong = change("not-the-password", NEW_PASSWORD).await;
    assert_eq!(wrong.status(), 403, "wrong current password");
    assert!(received.all().is_empty(), "the webhook was not called");
    assert_eq!(me(&c, &base).await["passwordChangeRequired"], json!(true));
    // Nor does a new password the policy refuses.
    let weak = change(pass, "short").await;
    assert_eq!(weak.status(), 400, "policy violation");
    assert!(received.all().is_empty());

    // The change.
    let ok = change(pass, NEW_PASSWORD).await;
    assert_eq!(ok.status(), 200, "the change is accepted");
    let body: Value = ok.json().await.unwrap();
    assert_eq!(body["changed"], json!(true), "{body}");
    assert_eq!(
        body["credentialsResealed"],
        json!(1),
        "the session's stored upstream password follows: {body}"
    );

    let got = received.all();
    assert_eq!(got.len(), 1, "exactly one webhook call");
    let (headers, raw) = &got[0];
    assert!(
        mw_passwd::verify_signature(
            WEBHOOK_SECRET.as_bytes(),
            raw,
            headers.get("x-signature").unwrap().to_str().unwrap()
        ),
        "signed with MW_PASSWD_WEBHOOK_SECRET"
    );
    let payload: Value = serde_json::from_slice(raw).unwrap();
    assert_eq!(
        payload,
        json!({
            "event": "password_change",
            "account_id": account_id,
            "username": user,
            "new_password": NEW_PASSWORD,
        })
    );

    // Released: the flag is clear for the session and in the panel.
    assert!(
        me(&c, &base).await.get("passwordChangeRequired").is_none(),
        "the hold is released"
    );
    assert_ne!(
        jmap(&c, &base).await.status(),
        403,
        "the mailbox is no longer held"
    );
    assert_eq!(
        panel_force_flag(&admin, &base, user).await,
        json!(false),
        "the panel no longer shows the flag"
    );
}

#[tokio::test]
async fn a_server_with_a_misconfigured_backend_does_not_boot() {
    let _g = serial().await;

    // Precondition: with nothing set, the same config boots.
    assert!(
        mw_server::build_app_full(app_config(Vec::new()), admin_v6())
            .await
            .is_ok()
    );

    for (pairs, var) in [
        (
            vec![("MW_PASSWD_BACKEND", "webhook")],
            "MW_PASSWD_WEBHOOK_URL",
        ),
        (vec![("MW_PASSWD_BACKEND", "dovecott")], "MW_PASSWD_BACKEND"),
        (
            vec![
                ("MW_PASSWD_BACKEND", "poppassd"),
                ("MW_PASSWD_POPPASSD_HOST", "127.0.0.1"),
                ("MW_PASSWD_POPPASSD_PORT", "99999"),
            ],
            "MW_PASSWD_POPPASSD_PORT",
        ),
    ] {
        set_env(&pairs);
        let boot = tokio::spawn(async {
            mw_server::build_app_full(app_config(Vec::new()), admin_v6())
                .await
                .map(|_| ())
        })
        .await;
        // SAFETY: `serial()`'s guard is held.
        unsafe {
            for k in PASSWD_VARS {
                std::env::remove_var(k);
            }
        }
        let message = match boot {
            Ok(Ok(())) => panic!("{pairs:?}: the server booted"),
            Ok(Err(e)) => e.to_string(),
            Err(join) => {
                let payload = join.into_panic();
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
                    .unwrap_or_default()
            }
        };
        assert!(
            message.contains(var),
            "{pairs:?}: the failure names {var}: {message:?}"
        );
    }
}
