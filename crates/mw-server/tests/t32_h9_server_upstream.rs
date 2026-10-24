//! t32-h9 — the server decides the mail upstream; a client never names it.
//!
//! The sign-in form sends an address and a password. Which JMAP server (proxy
//! mode) or IMAP/POP3 server (engine mode) that address signs in to is server
//! configuration: `MW_UPSTREAM_DEFAULT` (`SecurityConfig::upstream_default`), or
//! in proxy mode a `MW_JMAP_UPSTREAMS` list naming exactly one origin. These tests
//! drive `POST /api/login` and `POST /api/discover` over the real router:
//!
//! * a login body that names a server (`jmapUrl` and its spellings) is refused
//!   with 400 `clientServerNotAllowed`, before any credential check, and gets no
//!   session — even when the named server is the configured one;
//! * a login with only the address and password signs in to the configured
//!   upstream, in proxy mode and in engine mode;
//! * with no upstream configured the answer is 404 `addressNotConfigured`, never a
//!   session;
//! * `/api/discover` is gone (410): it used to hand the browser the hosts the
//!   server found for a domain.
//!
//! Every refusal is paired with the request that succeeds on the same server, so
//! a refusal cannot pass because the route was unreachable.

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};
use tokio::net::TcpListener;

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

mod common;
use common::mock_mail::MockPop3;
use common::test_db;

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

async fn spawn_mock() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, mw_mock_jmap::router()).await;
    });
    format!("http://{addr}")
}

async fn spawn_server(mode: ServerMode, security: SecurityConfig) -> String {
    let dir = test_db::unique_dir("mw-t32-h9");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode,
        hardening: HardeningConfig::default(),
        security,
    };
    let v6 = V6Config {
        admin_enabled: false,
        admin_username: None,
        admin_password: None,
        redis_url: None,
    };
    let app = build_app_full(config, v6).await.expect("server boots").0;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });
    format!("http://{addr}")
}

/// `POST {base}{path}` with `body`; returns the status, whether a session cookie
/// was set, and the JSON body (`Null` when there is none).
async fn post(base: &str, path: &str, body: Value) -> (u16, bool, Value) {
    let resp = reqwest::Client::new()
        .post(format!("{base}{path}"))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let cookie = resp
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .any(|v| v.to_str().unwrap_or_default().contains("mw_session="));
    let text = resp.text().await.unwrap();
    (
        status,
        cookie,
        serde_json::from_str(&text).unwrap_or(Value::Null),
    )
}

fn credentials() -> Value {
    json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS })
}

/// The proxy server most cases use: the mock is the only `MW_JMAP_UPSTREAMS`
/// origin, which before t32-h9 is what let a login name it.
async fn proxy_listing(mock: &str) -> String {
    spawn_server(
        ServerMode::Proxy,
        SecurityConfig {
            jmap_upstreams: Some(vec![mock.to_string()]),
            ..SecurityConfig::default()
        },
    )
    .await
}

#[tokio::test]
async fn a_login_that_names_a_server_is_refused() {
    let mock = spawn_mock().await;
    let base = proxy_listing(&mock).await;
    // Every spelling a client could use to choose the upstream, each naming the
    // server the deployment itself would pick.
    for field in [
        "jmapUrl",
        "server",
        "serverUrl",
        "host",
        "port",
        "imapUrl",
        "upstream",
        "url",
    ] {
        let mut body = credentials();
        body[field] = if field == "port" {
            json!(8181)
        } else {
            json!(mock)
        };
        let (status, cookie, answer) = post(&base, "/api/login", body).await;
        assert_eq!(status, 400, "a login naming `{field}` is refused: {answer}");
        assert_eq!(
            answer["code"],
            json!("clientServerNotAllowed"),
            "{field}: {answer}"
        );
        assert!(!cookie, "a refused login sets no session ({field})");
    }
    // An empty value still names the field: refused, not read as absent.
    let mut body = credentials();
    body["jmapUrl"] = json!("");
    let (status, _, answer) = post(&base, "/api/login", body).await;
    assert_eq!(status, 400, "an empty jmapUrl is refused too: {answer}");
}

#[tokio::test]
async fn discover_is_gone() {
    let mock = spawn_mock().await;
    let base = proxy_listing(&mock).await;
    let (status, _, answer) = post(
        &base,
        "/api/discover",
        json!({ "email": "ada@example.org" }),
    )
    .await;
    assert_eq!(status, 410, "/api/discover no longer answers: {answer}");
    assert!(
        !answer.to_string().contains("imap") && !answer.to_string().contains("jmapSrv"),
        "no server data in the answer: {answer}"
    );
}

/// Proxy server whose only upstream configuration is `MW_UPSTREAM_DEFAULT`.
async fn proxy_default(default: &str) -> String {
    spawn_server(
        ServerMode::Proxy,
        SecurityConfig {
            upstream_default: Some(default.to_string()),
            ..SecurityConfig::default()
        },
    )
    .await
}

#[tokio::test]
async fn the_address_and_password_sign_in_to_the_configured_default() {
    let mock = spawn_mock().await;
    // The mock listens on loopback. A configured default is operator-named like a
    // listed origin, so the address floor does not refuse it.
    let base = proxy_default(&mock).await;
    let (status, cookie, answer) = post(&base, "/api/login", credentials()).await;
    assert_eq!(status, 200, "signs in to MW_UPSTREAM_DEFAULT: {answer}");
    assert!(cookie, "a session cookie is set");
    assert_eq!(answer["accountId"], json!(mw_mock_jmap::ACCOUNT_ID));
    // Wrong password against the same upstream: the uniform 401, not a 404.
    let (status, cookie, _) = post(
        &base,
        "/api/login",
        json!({ "username": mw_mock_jmap::USER, "password": "wrong" }),
    )
    .await;
    assert_eq!((status, cookie), (401, false));
    // Naming the very server the default names is still refused.
    let mut body = credentials();
    body["jmapUrl"] = json!(mock);
    let (status, cookie, answer) = post(&base, "/api/login", body).await;
    assert_eq!((status, cookie), (400, false), "{answer}");
}

#[tokio::test]
async fn a_single_listed_origin_still_signs_in_without_the_client_naming_it() {
    let mock = spawn_mock().await;
    let base = proxy_listing(&mock).await;
    let (status, cookie, answer) = post(&base, "/api/login", credentials()).await;
    assert_eq!(
        status, 200,
        "MW_JMAP_UPSTREAMS with one origin places every address: {answer}"
    );
    assert!(cookie);
}

#[tokio::test]
async fn with_no_upstream_configured_the_address_is_not_placed() {
    let mock = spawn_mock().await;
    // Control on a configured server first, so the 404 below is about the
    // configuration and not an unreachable route.
    let configured = proxy_default(&mock).await;
    assert_eq!(post(&configured, "/api/login", credentials()).await.0, 200);

    let bare = spawn_server(ServerMode::Proxy, SecurityConfig::default()).await;
    let (status, cookie, answer) = post(&bare, "/api/login", credentials()).await;
    assert_eq!(status, 404, "{answer}");
    assert_eq!(answer["code"], json!("addressNotConfigured"));
    assert!(!cookie);

    // Two listed origins: the server cannot pick one, so neither can the login.
    let other = spawn_mock().await;
    let two = spawn_server(
        ServerMode::Proxy,
        SecurityConfig {
            jmap_upstreams: Some(vec![mock.clone(), other]),
            ..SecurityConfig::default()
        },
    )
    .await;
    let (status, _, answer) = post(&two, "/api/login", credentials()).await;
    assert_eq!(status, 404, "{answer}");

    // An engine URL is no JMAP server: a proxy default of that form places nothing.
    let wrong_kind = proxy_default("imap://127.0.0.1:1").await;
    let (status, _, answer) = post(&wrong_kind, "/api/login", credentials()).await;
    assert_eq!(status, 404, "{answer}");
}

const ENGINE_USER: &str = "h9";
const ENGINE_PASS: &str = "h9-pass";

#[tokio::test]
async fn engine_mode_dials_the_configured_server_and_never_the_named_one() {
    // SAFETY: the only test in this binary that reads or writes the variable.
    unsafe { std::env::set_var("MW_ENGINE_TLS", "plaintext") };
    let configured = MockPop3::start(ENGINE_USER, ENGINE_PASS).await;
    let named = MockPop3::start(ENGINE_USER, ENGINE_PASS).await;
    let base = spawn_server(
        ServerMode::Engine,
        SecurityConfig {
            upstream_default: Some(configured.url()),
            ..SecurityConfig::default()
        },
    )
    .await;
    let engine_creds = json!({ "username": ENGINE_USER, "password": ENGINE_PASS });

    // A login naming another server is refused before anything is dialled.
    let mut body = engine_creds.clone();
    body["jmapUrl"] = json!(named.url());
    let (status, cookie, answer) = post(&base, "/api/login", body).await;
    assert_eq!((status, cookie), (400, false), "{answer}");
    assert_eq!(named.logins_accepted() + named.logins_refused(), 0);
    assert_eq!(
        configured.logins_accepted() + configured.logins_refused(),
        0
    );

    // The address and the password sign in to the configured server.
    let (status, cookie, answer) = post(&base, "/api/login", engine_creds).await;
    assert_eq!(status, 200, "{answer}");
    assert!(cookie);
    assert!(
        configured.logins_accepted() >= 1,
        "the configured server took the login"
    );
    assert_eq!(
        named.logins_accepted() + named.logins_refused(),
        0,
        "the named one was never used"
    );

    // Engine mode with nothing configured places nothing.
    let bare = spawn_server(ServerMode::Engine, SecurityConfig::default()).await;
    let (status, _, answer) = post(
        &bare,
        "/api/login",
        json!({ "username": ENGINE_USER, "password": ENGINE_PASS }),
    )
    .await;
    assert_eq!(status, 404, "{answer}");
    assert_eq!(answer["code"], json!("addressNotConfigured"));
}
