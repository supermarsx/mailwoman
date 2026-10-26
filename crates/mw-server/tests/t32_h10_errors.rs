//! t32-h10 — what a request nothing else answers gets, through the real router.
//!
//! 1. **A person's browser opening an unknown page** gets a small HTML page in the
//!    app's style with the real `404`, a link back to the mail app (base-path
//!    aware) and, under `/admin`, to the admin console. It used to get the SPA
//!    shell with a `200` (a soft 404: the mailbox rendered at a path that does not
//!    exist, and at any depth below one segment the shell's relative asset URLs
//!    404ed and left a blank page). The page has no script, carries the shell CSP
//!    and the other security headers, and is `no-store`.
//! 2. **Every other request** (a `fetch()`, an API client, a write, a missing
//!    subresource) gets `{"error", "code"}` as JSON with the real status, never
//!    HTML. A `405` on a mounted route has the same shape.
//! 3. **Odd paths** (`..`, encoded dots and slashes, backslashes, NUL, overlong
//!    UTF-8) get a clean `400`/`404` and never a file outside the web directory.
//! 4. **Return targets taken from a request** (`relayState` on the SSO login) go
//!    through one validator: only a same-origin path under the deploy prefix.
//! 5. **No internal detail** in an error body: a store failure says the operation
//!    failed, not which table was missing.
//! 6. **OAuth "deny"** redirects only to a redirect URI registered for the client.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use reqwest::redirect::Policy;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, MutexGuard};

use mw_server::sso::{SsoEntry, SsoMeta};
use mw_server::{
    AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full,
    build_app_with_sso_mock,
};
use mw_sso::{
    BeginRedirect, ClaimMap, FirstLoginPolicy, Metadata, PendingState, Redirect, SsoCallback,
    SsoError, SsoIdentity, SsoKind, SsoLogin, SsoScope,
};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

mod common;
use common::test_db;

const KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=root>MW_SHELL</div>";
const SECRET: &str = "MW_H10_SECRET_OUTSIDE_THE_WEB_DIR";
const ADMIN_USER: &str = "root";
const ADMIN_PASS: &str = "hunter2";

/// What Chromium sends for a top-level navigation.
const NAVIGATION: &[(&str, &str)] = &[
    (
        "accept",
        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    ),
    ("sec-fetch-dest", "document"),
    ("sec-fetch-mode", "navigate"),
];

/// The same navigation re-fetched by a service worker (t32-h8).
const WORKER_NAVIGATION: &[(&str, &str)] = &[
    ("accept", "text/html"),
    ("sec-fetch-dest", "empty"),
    ("sec-fetch-mode", "navigate"),
];

/// A `fetch()` from the page.
const PAGE_FETCH: &[(&str, &str)] = &[
    ("accept", "application/json"),
    ("sec-fetch-dest", "empty"),
    ("sec-fetch-mode", "cors"),
];

// ── servers ────────────────────────────────────────────────────────────────────

/// Serialises the `MW_BASE_PATH` mutation every app build in this binary reads.
fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(Default::default)
}

async fn set_base_path(base_path: Option<&str>) -> MutexGuard<'static, ()> {
    let guard = env_lock().lock().await;
    // SAFETY: the guard serialises every mutation and every read that depends on one
    // within this binary (house pattern, see `t20_forwarded_spoof.rs`).
    unsafe {
        match base_path {
            Some(v) => std::env::set_var("MW_BASE_PATH", v),
            None => std::env::remove_var("MW_BASE_PATH"),
        }
    }
    guard
}

struct Server {
    url: String,
    addr: SocketAddr,
    db: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

/// A proxy-mode server with the admin panel on, serving a stub `index.html` from a
/// web directory that has a secret file next to it (outside it).
async fn spawn(base_path: Option<&str>, upstream: Option<&str>) -> Server {
    let guard = set_base_path(base_path).await;
    let dir = test_db::unique_dir("mw-t32h10");
    let web = dir.join("web");
    std::fs::create_dir_all(web.join("assets")).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    std::fs::write(web.join("assets").join("app.js"), "console.log(1)").unwrap();
    std::fs::write(dir.join("secret.txt"), SECRET).unwrap();
    let db = dir.join("mw.db");
    let config = AppConfig {
        db_path: db.to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: upstream.map(|u| vec![u.to_string()]),
            ..SecurityConfig::default()
        },
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some(ADMIN_USER.into()),
        admin_password: Some(ADMIN_PASS.into()),
        redis_url: None,
    };
    let app = build_app_full(config, v6).await.expect("server boots").0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });
    Server {
        url: format!("http://{addr}"),
        addr,
        db,
        _guard: guard,
    }
}

async fn spawn_mock_jmap() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, mw_mock_jmap::router()).await;
    });
    format!("http://{addr}")
}

// ── requests ───────────────────────────────────────────────────────────────────

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(Policy::none())
        .build()
        .unwrap()
}

struct Answer {
    status: u16,
    headers: reqwest::header::HeaderMap,
    body: String,
}

impl Answer {
    fn header(&self, name: &str) -> String {
        self.headers
            .get_all(name)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect::<Vec<_>>()
            .join(", ")
    }
    fn content_type(&self) -> String {
        self.header("content-type")
    }
}

async fn send(
    method: reqwest::Method,
    url: &str,
    headers: &[(&str, &str)],
    cookie: Option<&str>,
) -> Answer {
    let mut req = client().request(method, url);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    let resp = req.send().await.unwrap();
    let status = resp.status().as_u16();
    let headers = resp.headers().clone();
    let body = resp.text().await.unwrap();
    Answer {
        status,
        headers,
        body,
    }
}

async fn get(url: &str, headers: &[(&str, &str)]) -> Answer {
    send(reqwest::Method::GET, url, headers, None).await
}

/// A request written byte for byte, so the path reaches the server exactly as given
/// (an HTTP client would normalise `..` and reject a backslash before sending).
async fn raw_get(addr: SocketAddr, target: &str, extra: &str) -> (u16, String) {
    let mut s = tokio::net::TcpStream::connect(addr).await.unwrap();
    let req =
        format!("GET {target} HTTP/1.1\r\nHost: localhost\r\n{extra}Connection: close\r\n\r\n");
    s.write_all(req.as_bytes()).await.unwrap();
    let mut buf = Vec::new();
    let _ = s.read_to_end(&mut buf).await;
    let text = String::from_utf8_lossy(&buf).into_owned();
    let status = text
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (status, text)
}

// ── assertions ─────────────────────────────────────────────────────────────────

/// The designed page, with the security headers every page carries.
fn assert_error_page(a: &Answer, status: u16, what: &str) {
    assert_eq!(a.status, status, "{what}: status; body {:?}", a.body);
    assert!(
        a.content_type().starts_with("text/html"),
        "{what}: content-type {:?}, body {:?}",
        a.content_type(),
        a.body
    );
    assert!(
        !a.body.contains("MW_SHELL"),
        "{what}: the SPA shell, not the error page"
    );
    assert!(a.body.contains("Mailwoman"), "{what}: the app name");
    assert!(!a.body.contains("<script"), "{what}: no script");
    assert!(
        !a.body.contains("<style") && !a.body.contains("style="),
        "{what}: no inline style"
    );
    assert!(
        !a.body.contains("http://") && !a.body.contains("https://"),
        "{what}: loads nothing from elsewhere"
    );
    assert_security_headers(a, what);
    assert_eq!(
        a.header("cache-control"),
        "no-store",
        "{what}: cache-control"
    );
}

fn assert_security_headers(a: &Answer, what: &str) {
    let csp = a.header("content-security-policy");
    assert!(
        csp.contains("default-src 'none'") && csp.contains("script-src 'self'"),
        "{what}: the shell CSP, got {csp:?}"
    );
    assert!(
        !csp.contains("unsafe-inline"),
        "{what}: CSP unchanged, got {csp:?}"
    );
    assert_eq!(a.header("x-content-type-options"), "nosniff", "{what}");
    assert_eq!(a.header("x-frame-options"), "DENY", "{what}");
    assert_eq!(a.header("referrer-policy"), "no-referrer", "{what}");
    assert!(
        a.headers.get("server").is_none(),
        "{what}: no Server header"
    );
    assert!(
        a.headers.get("x-powered-by").is_none(),
        "{what}: no X-Powered-By"
    );
}

fn assert_json_error(a: &Answer, status: u16, code: &str, what: &str) {
    assert_eq!(a.status, status, "{what}: status; body {:?}", a.body);
    assert!(
        a.content_type().starts_with("application/json"),
        "{what}: content-type {:?}, body {:?}",
        a.content_type(),
        a.body
    );
    let v: Value = serde_json::from_str(&a.body)
        .unwrap_or_else(|e| panic!("{what}: body is not JSON ({e}): {:?}", a.body));
    assert_eq!(v["code"], code, "{what}: {v}");
    assert!(v["error"].is_string(), "{what}: {v}");
    assert_security_headers(a, what);
}

fn assert_shell(a: &Answer, what: &str) {
    assert_eq!(a.status, 200, "{what}: status; body {:?}", a.body);
    assert!(
        a.body.contains("MW_SHELL"),
        "{what}: the shell, got {:?}",
        a.body
    );
}

// ── 1. the page ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_navigation_to_an_unknown_page_gets_the_designed_404() {
    for prefix in [None, Some("/mail")] {
        let s = spawn(prefix, None).await;
        let p = prefix.unwrap_or("");
        for path in [
            "/does-not-exist",
            "/settings/keys",
            "/mail/inbox/42",
            "/x.html",
        ] {
            for (how, headers) in [("browser", NAVIGATION), ("worker", WORKER_NAVIGATION)] {
                let what = format!("{p}{path} ({how})");
                let a = get(&format!("{}{p}{path}", s.url), headers).await;
                assert_error_page(&a, 404, &what);
                assert!(a.body.contains("This page does not exist."), "{what}");
                assert!(
                    a.body.contains(&format!(r#"href="{p}/""#)),
                    "{what}: link to the mail app under the prefix: {}",
                    a.body
                );
                assert!(
                    !a.body.contains(&format!(r#"href="{p}/admin""#)),
                    "{what}: no admin link outside /admin"
                );
                assert!(
                    a.body.contains(&format!(r#"href="{p}/__mw_error.css""#)),
                    "{what}: stylesheet under the prefix"
                );
            }
        }
        // Under /admin the page also links the console.
        let a = get(&format!("{}{p}/admin/bogus", s.url), NAVIGATION).await;
        assert_error_page(&a, 404, "/admin/bogus");
        assert!(
            a.body.contains(&format!(r#"href="{p}/admin""#)),
            "{}",
            a.body
        );

        // The stylesheet it links is served, as CSS, under the same headers.
        let css = get(&format!("{}{p}/__mw_error.css", s.url), &[]).await;
        assert_eq!(css.status, 200, "stylesheet");
        assert!(
            css.content_type().starts_with("text/css"),
            "{}",
            css.content_type()
        );
        assert!(css.body.contains("prefers-color-scheme: light"));
        assert_security_headers(&css, "stylesheet");
    }
}

#[tokio::test]
async fn the_pages_the_app_has_still_get_the_shell() {
    for prefix in [None, Some("/mail")] {
        let s = spawn(prefix, None).await;
        let p = prefix.unwrap_or("");
        for path in ["/", "/index.html", "/admin", "/oauth/authorize"] {
            let a = get(&format!("{}{p}{path}", s.url), NAVIGATION).await;
            assert_shell(&a, &format!("{p}{path}"));
            let a = get(&format!("{}{p}{path}", s.url), WORKER_NAVIGATION).await;
            assert_shell(&a, &format!("{p}{path} (worker)"));
        }
        // A query on a page path is still the page.
        let a = get(
            &format!("{}{p}/oauth/authorize?client_id=x&redirect_uri=y", s.url),
            NAVIGATION,
        )
        .await;
        assert_shell(&a, "oauth authorize with a query");
    }
}

#[tokio::test]
async fn a_trailing_slash_on_the_consent_page_redirects() {
    for prefix in [None, Some("/mail")] {
        let s = spawn(prefix, None).await;
        let p = prefix.unwrap_or("");
        let a = get(
            &format!("{}{p}/oauth/authorize/?client_id=x", s.url),
            NAVIGATION,
        )
        .await;
        assert_eq!(a.status, 308, "{p}/oauth/authorize/: {}", a.body);
        assert_eq!(
            a.header("location"),
            format!("{p}/oauth/authorize?client_id=x")
        );
        assert_security_headers(&a, "redirect");
    }
}

// ── 2. machine-readable errors ─────────────────────────────────────────────────

#[tokio::test]
async fn everything_that_is_not_a_navigation_gets_a_json_error() {
    let s = spawn(None, None).await;

    for (path, headers) in [
        ("/api/does-not-exist", PAGE_FETCH),
        ("/jmap/does-not-exist", PAGE_FETCH),
        ("/admin/does-not-exist", PAGE_FETCH),
        ("/does-not-exist", PAGE_FETCH),
        ("/does-not-exist", &[("accept", "application/json")][..]),
        ("/does-not-exist", &[][..]),
        ("/assets/missing.js", &[("sec-fetch-dest", "script")][..]),
        ("/fonts/missing.woff2", &[("sec-fetch-dest", "font")][..]),
    ] {
        let a = get(&format!("{}{path}", s.url), headers).await;
        assert_json_error(&a, 404, "not_found", &format!("GET {path} {headers:?}"));
        assert!(!a.body.contains('<'), "no HTML: {}", a.body);
    }

    // A write to an unmounted path, even with navigation-looking headers.
    let a = send(
        reqwest::Method::POST,
        &format!("{}/mcp", s.url),
        NAVIGATION,
        None,
    )
    .await;
    assert_json_error(&a, 404, "not_found", "POST /mcp");

    // A mounted route with the wrong method: 405, JSON, and the Allow header.
    let a = get(&format!("{}/api/login", s.url), PAGE_FETCH).await;
    assert_json_error(&a, 405, "method_not_allowed", "GET /api/login");
    assert!(
        a.header("allow").contains("POST"),
        "allow: {:?}",
        a.header("allow")
    );
}

#[tokio::test]
async fn head_and_options_on_an_unknown_path() {
    let s = spawn(None, None).await;
    let a = send(
        reqwest::Method::HEAD,
        &format!("{}/does-not-exist", s.url),
        NAVIGATION,
        None,
    )
    .await;
    assert_eq!(a.status, 404, "HEAD of an unknown page");
    assert!(a.body.is_empty(), "HEAD has no body");
    assert!(a.content_type().starts_with("text/html"));
    assert_security_headers(&a, "HEAD");

    let a = send(
        reqwest::Method::OPTIONS,
        &format!("{}/does-not-exist", s.url),
        &[],
        None,
    )
    .await;
    assert_json_error(&a, 404, "not_found", "OPTIONS");
}

// ── 3. odd paths ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn odd_paths_never_reach_outside_the_web_dir() {
    let s = spawn(None, None).await;
    let nav = "Accept: text/html\r\nSec-Fetch-Dest: document\r\nSec-Fetch-Mode: navigate\r\n";
    for target in [
        "/../secret.txt",
        "/assets/../../secret.txt",
        "/..%2fsecret.txt",
        "/%2e%2e/secret.txt",
        "/%2e%2e%2fsecret.txt",
        "/assets/%2e%2e/%2e%2e/secret.txt",
        "/..\\secret.txt",
        "/assets\\..\\..\\secret.txt",
        "/..%5csecret.txt",
        "/%c0%ae%c0%ae/secret.txt",
        "/%252e%252e/secret.txt",
        "/index.html%00.png",
        "/%00",
        "/C:/Windows/win.ini",
        "/index.html::$DATA",
        "/./index.html",
    ] {
        for extra in ["", nav] {
            let (status, text) = raw_get(s.addr, target, extra).await;
            assert!(
                matches!(status, 400 | 404),
                "{target} ({}): status {status}\n{text}",
                if extra.is_empty() {
                    "fetch"
                } else {
                    "navigation"
                }
            );
            assert!(
                !text.contains(SECRET),
                "{target}: leaked the file outside the web dir"
            );
            assert!(
                !text.contains("MW_SHELL"),
                "{target}: answered with the shell"
            );
            assert!(
                !text.to_ascii_lowercase().contains("mw-t32h10"),
                "{target}: leaked a filesystem path\n{text}"
            );
        }
    }
    // The control: the real file and the real asset are served.
    let (status, text) = raw_get(s.addr, "/assets/app.js", "").await;
    assert_eq!(status, 200, "{text}");
}

// ── 4. return targets ──────────────────────────────────────────────────────────

const STATE_TOKEN: &str = "fixedstatetoken0123456789abcdef";

/// An OIDC provider that completes with a fixed identity, so a test can drive
/// begin → callback without an IdP (the pattern of `sso_e2e.rs`).
struct MockProvider;

#[async_trait]
impl SsoLogin for MockProvider {
    async fn begin(&self, relay_state: Option<String>) -> Result<BeginRedirect, SsoError> {
        Ok(BeginRedirect {
            url: format!("https://idp.example/authorize?state={STATE_TOKEN}"),
            state_token: STATE_TOKEN.to_string(),
            pending: PendingState::Oidc {
                pkce_verifier: "verifier".into(),
                nonce: "nonce".into(),
                relay_state,
            },
        })
    }
    async fn complete(&self, _callback: SsoCallback) -> Result<SsoIdentity, SsoError> {
        Ok(SsoIdentity {
            subject: "subject-alice".into(),
            email: Some("alice@acme.test".into()),
            display_name: None,
            groups: vec![],
            claims: BTreeMap::new(),
        })
    }
    fn metadata(&self) -> Option<Metadata> {
        None
    }
    fn logout(&self, _subject: &str) -> Option<Redirect> {
        None
    }
}

async fn spawn_sso(base_path: Option<&str>) -> (String, MutexGuard<'static, ()>) {
    let guard = set_base_path(base_path).await;
    let dir = test_db::unique_dir("mw-t32h10-sso");
    let web = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example",
                port: 993,
                tls: "implicit",
                username: "alice@acme.test",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "alice@acme.test".into(),
                password: "unused".into(),
            },
        )
        .await
        .unwrap();
    drop(store);
    let config = AppConfig {
        db_path: db,
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let v6 = V6Config {
        admin_enabled: false,
        admin_username: None,
        admin_password: None,
        redis_url: None,
    };
    let entry = SsoEntry {
        provider: Arc::new(MockProvider),
        meta: SsoMeta {
            identity_authority: "fixture:verified-idp".into(),
            display_name: "Acme".into(),
            kind: SsoKind::Oidc,
            scope: SsoScope::Deployment,
            enabled: true,
            first_login_policy: FirstLoginPolicy::Allowlist,
            claim_map: ClaimMap::default(),
        },
    };
    let app = build_app_with_sso_mock(config, v6, vec![("corp".to_string(), entry)])
        .await
        .expect("server boots")
        .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (format!("http://{addr}"), guard)
}

/// Run begin (with `relay`, already query-encoded) then the callback, and return
/// the callback's status and `Location`. A dropped connection is status 0.
async fn sso_round_trip(base: &str, prefix: &str, relay: &str) -> (u16, String) {
    let c = client();
    let begin = c
        .get(format!(
            "{base}{prefix}/api/sso/corp/begin?relayState={relay}"
        ))
        .send()
        .await
        .unwrap();
    assert!(begin.status().is_redirection(), "begin: {}", begin.status());
    match c
        .get(format!(
            "{base}{prefix}/api/sso/corp/callback?code=abc&state={STATE_TOKEN}"
        ))
        .send()
        .await
    {
        Ok(resp) => (
            resp.status().as_u16(),
            resp.headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_string(),
        ),
        Err(_) => (0, String::new()),
    }
}

#[tokio::test]
async fn the_sso_return_target_is_a_path_under_the_prefix() {
    for prefix in [None, Some("/mail")] {
        let (base, _guard) = spawn_sso(prefix).await;
        let p = prefix.unwrap_or("");
        let home = format!("{p}/");
        // (relayState as sent in the query, where the browser must land)
        let table: &[(&str, String)] = &[
            ("%2F", format!("{p}/")),
            ("%2Fadmin", format!("{p}/admin")),
            ("%2F%23%2Fcalendar", format!("{p}/#/calendar")),
            // Refused: each lands on the mail app instead.
            ("%2F%2Fevil.example", home.clone()),
            ("%2F%5Cevil.example", home.clone()),
            ("%2F%09%2Fevil.example", home.clone()),
            ("%2F%252f%2Fevil.example", home.clone()),
            ("%2F%255cevil.example", home.clone()),
            ("https%3A%2F%2Fevil.example", home.clone()),
            ("javascript%3Aalert(1)", home.clone()),
            ("%2Fx%0D%0ASet-Cookie%3A%20a%3Db", home.clone()),
            ("%2F..%2F..%2Fother", home.clone()),
            ("%2Fcaf%C3%A9", home.clone()),
        ];
        for (relay, want) in table {
            let (status, location) = sso_round_trip(&base, p, relay).await;
            assert_eq!(status, 303, "relayState={relay}: status");
            assert_eq!(&location, want, "relayState={relay} under {p:?}");
        }
    }
}

// ── 5. no internal detail ──────────────────────────────────────────────────────

fn cookie_value(resp: &reqwest::Response, name: &str) -> Option<String> {
    resp.headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find_map(|c| {
            let pair = c.split(';').next()?;
            let (k, v) = pair.split_once('=')?;
            (k == name).then(|| v.to_string())
        })
}

async fn admin_cookie(base: &str) -> String {
    let resp = client()
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": ADMIN_USER, "password": ADMIN_PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "admin login");
    let session = cookie_value(&resp, "mw_admin_session").expect("admin cookie");
    let csrf = cookie_value(&resp, "mw_csrf").expect("csrf cookie");
    format!("mw_admin_session={session}; mw_csrf={csrf}")
}

/// Words that would mean a store or library error reached the client.
const INTERNALS: &[&str] = &[
    "no such table",
    "sqlite",
    "sqlx",
    "ui_plugins",
    "SELECT",
    "database",
    "mw-t32h10",
    "panicked",
    "src/",
    ".rs:",
];

fn assert_no_internals(a: &Answer, what: &str) {
    let all = format!("{:?}\n{}", a.headers, a.body);
    for word in INTERNALS {
        assert!(
            !all.to_ascii_lowercase()
                .contains(&word.to_ascii_lowercase()),
            "{what}: {word:?} in the response\n{all}"
        );
    }
}

#[tokio::test]
async fn a_store_failure_does_not_reach_the_client() {
    let s = spawn(None, None).await;
    let cookie = admin_cookie(&s.url).await;

    // Control: the route works.
    let ok = send(
        reqwest::Method::GET,
        &format!("{}/admin/ui-plugins", s.url),
        PAGE_FETCH,
        Some(&cookie),
    )
    .await;
    assert_eq!(ok.status, 200, "{}", ok.body);

    // Break the table under it.
    let opts = sqlx::sqlite::SqliteConnectOptions::new().filename(&s.db);
    let pool = sqlx::SqlitePool::connect_with(opts).await.unwrap();
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DROP TABLE ui_plugin_grants")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DROP TABLE ui_plugins")
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;

    let a = send(
        reqwest::Method::GET,
        &format!("{}/admin/ui-plugins", s.url),
        PAGE_FETCH,
        Some(&cookie),
    )
    .await;
    assert_eq!(a.status, 500, "{}", a.body);
    assert!(a.content_type().starts_with("application/json"));
    assert_no_internals(&a, "GET /admin/ui-plugins with its table gone");
}

#[tokio::test]
async fn error_answers_carry_no_internal_detail() {
    let s = spawn(None, None).await;
    for (method, path, headers) in [
        (reqwest::Method::GET, "/does-not-exist", NAVIGATION),
        (reqwest::Method::GET, "/does-not-exist", PAGE_FETCH),
        (reqwest::Method::GET, "/api/login", PAGE_FETCH),
        (reqwest::Method::POST, "/nope", PAGE_FETCH),
    ] {
        let a = send(method.clone(), &format!("{}{path}", s.url), headers, None).await;
        assert!(a.status >= 400, "{method} {path}");
        assert_no_internals(&a, &format!("{method} {path}"));
        assert!(
            !a.body.contains("/does-not-exist"),
            "{method} {path}: echoes the path"
        );
    }
}

// ── 6. OAuth deny ──────────────────────────────────────────────────────────────

/// Sign in to the mailbox and return the cookie header. The server dials its one
/// configured upstream (the JMAP mock); a login body may not name a server (t32-h9).
async fn mailbox_cookie(base: &str) -> (String, String) {
    let resp = client()
        .post(format!("{base}/api/login"))
        .json(&json!({
            "username": mw_mock_jmap::USER,
            "password": mw_mock_jmap::PASS,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "mailbox login");
    let session = cookie_value(&resp, "mw_session").expect("session cookie");
    let csrf = cookie_value(&resp, "mw_csrf").expect("csrf cookie");
    (format!("mw_session={session}; mw_csrf={csrf}"), csrf)
}

#[tokio::test]
async fn oauth_deny_redirects_only_to_a_registered_uri() {
    let mock = spawn_mock_jmap().await;
    let s = spawn(None, Some(&mock)).await;
    let (cookie, csrf) = mailbox_cookie(&s.url).await;
    for (client_id, redirect) in [
        ("no-such-client", "https://evil.example/cb"),
        ("no-such-client", "javascript:alert(1)//"),
    ] {
        let resp = client()
            .post(format!("{}/oauth/decision", s.url))
            .header("cookie", &cookie)
            .header("x-csrf-token", &csrf)
            .header("origin", &s.url)
            .json(&json!({
                "approve": false,
                "params": { "clientId": client_id, "redirectUri": redirect },
            }))
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap();
        assert_eq!(status, 400, "deny for {client_id} -> {redirect}: {body}");
        assert!(
            !body.contains("evil.example") && !body.contains("javascript:"),
            "{body}"
        );
    }
}
