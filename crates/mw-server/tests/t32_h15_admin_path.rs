//! t32-h15 — the admin console's URL path is set by the operator (`MW_ADMIN_PATH`).
//!
//! 1. **The console moves.** With `MW_ADMIN_PATH=/ops-…` the console page, the admin
//!    sign-in and every admin API route answer under `{MW_BASE_PATH}/ops-…`, and the
//!    page served there says it is the console (`<meta name="mw-console">`) so the
//!    web app never needs the path compiled in.
//! 2. **The old path is gone without a trace.** `/admin`, `/admin/`, `/admin/users`
//!    and `POST /admin/login` get exactly what an unknown path gets: the same status,
//!    the same headers and the same bytes.
//! 3. **The path does not leak.** No response to a request outside the admin path
//!    carries it: the mail shell, the base shim, the service worker, the bundle,
//!    public API answers, error pages, redirects and headers.
//! 4. **A bad value stops the start**, with an error naming the variable.
//! 5. **Unset, nothing changes**: the console stays at `/admin`.
//! 6. **One source of truth**: no admin route in `mw-server` is written as a
//!    `"/admin…"` literal; every one goes through `mw_admin::config::admin_route`.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use reqwest::redirect::Policy;
use serde_json::json;
use tokio::sync::{Mutex, MutexGuard};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

mod common;
use common::test_db;

const KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
/// Shaped like the Vite build: relative asset URLs in `<head>`.
const INDEX_HTML: &str = "<!doctype html><html><head><meta charset=\"UTF-8\" />\
<link rel=\"icon\" href=\"./favicon.svg\" type=\"image/svg+xml\" />\
<script type=\"module\" crossorigin src=\"./assets/index-AbCdEf12.js\"></script>\
<link rel=\"stylesheet\" crossorigin href=\"./assets/index-AbCdEf12.css\">\
</head><body><div id=\"root\">MW_SHELL</div></body></html>";
const ADMIN_USER: &str = "root";
const ADMIN_PASS: &str = "hunter2";
/// A path no file, route or default contains, so finding it anywhere is a leak.
const SECRET_SEGMENT: &str = "ops-h15q7zx";
const META: &str = r#"<meta name="mw-console" content="admin">"#;

/// What Chromium sends for a top-level navigation.
const NAVIGATION: &[(&str, &str)] = &[
    (
        "accept",
        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    ),
    ("sec-fetch-dest", "document"),
    ("sec-fetch-mode", "navigate"),
];

/// A `fetch()` from the page.
const PAGE_FETCH: &[(&str, &str)] = &[
    ("accept", "application/json"),
    ("sec-fetch-dest", "empty"),
    ("sec-fetch-mode", "cors"),
];

// ── servers ────────────────────────────────────────────────────────────────────

/// Serialises the process environment every app build in this binary reads.
fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(Default::default)
}

struct Env {
    base_path: Option<&'static str>,
    admin_path: Option<&'static str>,
}

impl Env {
    const fn admin(admin_path: &'static str) -> Self {
        Self {
            base_path: None,
            admin_path: Some(admin_path),
        }
    }
}

async fn set_env(env: &Env) -> MutexGuard<'static, ()> {
    let guard = env_lock().lock().await;
    // SAFETY: the guard serialises every mutation and every read that depends on one
    // within this binary (house pattern, see `t20_forwarded_spoof.rs`).
    unsafe {
        for (k, v) in [
            ("MW_BASE_PATH", env.base_path),
            ("MW_ADMIN_PATH", env.admin_path),
        ] {
            match v {
                Some(v) => std::env::set_var(k, v),
                None => std::env::remove_var(k),
            }
        }
    }
    guard
}

struct Server {
    url: String,
    _guard: MutexGuard<'static, ()>,
}

fn config(web: PathBuf, db: &Path) -> AppConfig {
    AppConfig {
        db_path: db.to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    }
}

/// [`config`] with `MW_CSRF_STRICT` on: every cookie-less write without a token is
/// refused unless its route is on the exempt list.
fn strict_config(web: PathBuf, db: &Path) -> AppConfig {
    AppConfig {
        hardening: HardeningConfig {
            csrf_strict: true,
            ..HardeningConfig::default()
        },
        ..config(web, db)
    }
}

fn v6() -> V6Config {
    V6Config {
        admin_enabled: true,
        admin_username: Some(ADMIN_USER.into()),
        admin_password: Some(ADMIN_PASS.into()),
        redis_url: None,
    }
}

/// A stub web build: the shell, one asset and the real service worker.
fn stub_web(tag: &str) -> (PathBuf, PathBuf) {
    let dir = test_db::unique_dir(tag);
    let web = dir.join("web");
    std::fs::create_dir_all(web.join("assets")).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    std::fs::write(
        web.join("assets").join("index-AbCdEf12.js"),
        "console.log(1)",
    )
    .unwrap();
    std::fs::write(web.join("assets").join("index-AbCdEf12.css"), "body{}").unwrap();
    let sw = repo_root().join("apps/web/public/sw.js");
    std::fs::write(web.join("sw.js"), std::fs::read(sw).unwrap()).unwrap();
    std::fs::write(web.join("favicon.svg"), "<svg/>").unwrap();
    (web, dir.join("mw.db"))
}

async fn spawn_with(env: Env, web: PathBuf, db: PathBuf) -> Server {
    spawn_config(env, config(web, &db)).await
}

async fn spawn_config(env: Env, config: AppConfig) -> Server {
    let guard = set_env(&env).await;
    let app = build_app_full(config, v6()).await.expect("server boots").0;
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
        _guard: guard,
    }
}

async fn spawn(env: Env) -> Server {
    let (web, db) = stub_web("mw-t32h15");
    spawn_with(env, web, db).await
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
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
    body: Vec<u8>,
}

impl Answer {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
    /// Every header but `date`, which differs between any two responses.
    fn comparable_headers(&self) -> BTreeMap<String, Vec<String>> {
        let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (k, v) in &self.headers {
            if k.as_str() == "date" {
                continue;
            }
            out.entry(k.as_str().to_string())
                .or_default()
                .push(v.to_str().unwrap_or("<binary>").to_string());
        }
        out
    }
}

async fn send(
    method: reqwest::Method,
    url: &str,
    headers: &[(&str, &str)],
    cookie: Option<&str>,
    body: Option<serde_json::Value>,
) -> Answer {
    let mut req = client().request(method, url);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    if let Some(b) = body {
        req = req.json(&b);
    }
    let resp = req.send().await.unwrap();
    let status = resp.status().as_u16();
    let headers = resp.headers().clone();
    let body = resp.bytes().await.unwrap().to_vec();
    Answer {
        status,
        headers,
        body,
    }
}

async fn get(url: &str, headers: &[(&str, &str)]) -> Answer {
    send(reqwest::Method::GET, url, headers, None, None).await
}

fn cookie_value(a: &Answer, name: &str) -> Option<String> {
    a.headers
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find_map(|c| {
            let first = c.split(';').next()?;
            let (k, v) = first.split_once('=')?;
            (k.trim() == name && !v.is_empty()).then(|| v.to_string())
        })
}

/// Sign in at `{console}/login` and return the cookie header for later calls.
async fn admin_sign_in(console: &str) -> String {
    let a = send(
        reqwest::Method::POST,
        &format!("{console}/login"),
        &[],
        None,
        Some(json!({ "username": ADMIN_USER, "password": ADMIN_PASS })),
    )
    .await;
    assert_eq!(a.status, 200, "admin login at {console}: {}", a.text());
    let session = cookie_value(&a, "mw_admin_session").expect("admin cookie");
    let csrf = cookie_value(&a, "mw_csrf").expect("csrf cookie");
    format!("mw_admin_session={session}; mw_csrf={csrf}")
}

/// The console works end to end at `console` (a full URL): the page, sign-in, the
/// session probe, one admin API route, and sign-out.
async fn assert_console_at(console: &str) {
    let page = get(console, NAVIGATION).await;
    assert_eq!(
        page.status,
        200,
        "console page at {console}: {}",
        page.text()
    );
    let html = page.text();
    assert!(html.contains("MW_SHELL"), "the app shell at {console}");
    assert!(
        html.contains(META),
        "the console marker at {console}: {html}"
    );

    let cookie = admin_sign_in(console).await;
    let session = send(
        reqwest::Method::GET,
        &format!("{console}/session"),
        PAGE_FETCH,
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(
        session.status,
        200,
        "session at {console}: {}",
        session.text()
    );
    let users = send(
        reqwest::Method::GET,
        &format!("{console}/users"),
        PAGE_FETCH,
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(users.status, 200, "users at {console}: {}", users.text());

    let csrf = cookie
        .split("mw_csrf=")
        .nth(1)
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    let out = send(
        reqwest::Method::POST,
        &format!("{console}/logout"),
        &[("x-csrf-token", csrf.as_str())],
        Some(&cookie),
        None,
    )
    .await;
    assert!(
        out.status == 200 || out.status == 204,
        "logout at {console}: {} {}",
        out.status,
        out.text()
    );
}

// ── 1 + 5: the console answers where it is configured ──────────────────────────

#[tokio::test]
async fn the_console_answers_at_the_configured_path() {
    let s = spawn(Env::admin("/ops-h15q7zx")).await;
    assert_console_at(&format!("{}/ops-h15q7zx", s.url)).await;

    // The trailing-slash form redirects to the page, as `/admin/` did.
    let slash = get(&format!("{}/ops-h15q7zx/?x=1", s.url), NAVIGATION).await;
    assert_eq!(slash.status, 308, "{}", slash.text());
    assert_eq!(slash.headers["location"], "/ops-h15q7zx?x=1");

    // The mail client's shell does not say it is the console.
    let mail = get(&format!("{}/", s.url), NAVIGATION).await;
    assert_eq!(mail.status, 200);
    assert!(mail.text().contains("MW_SHELL"));
    assert!(!mail.text().contains("mw-console"), "{}", mail.text());
}

#[tokio::test]
async fn the_console_answers_under_a_base_path_and_at_a_deeper_path() {
    let s = spawn(Env {
        base_path: Some("/mail"),
        admin_path: Some("/ops/h15-console"),
    })
    .await;
    let console = format!("{}/mail/ops/h15-console", s.url);
    assert_console_at(&console).await;
    // At two segments deep the shell's `./assets/…` would resolve under `/mail/ops/`;
    // the console page names its files from the deploy prefix instead.
    let html = get(&console, NAVIGATION).await.text();
    assert!(
        html.contains(r#"src="/mail/assets/index-AbCdEf12.js""#),
        "{html}"
    );
    assert!(
        html.contains(r#"href="/mail/assets/index-AbCdEf12.css""#),
        "{html}"
    );
    assert!(!html.contains("\"./"), "{html}");
    let shim = get(&format!("{}/mail/assets/index-AbCdEf12.js", s.url), &[]).await;
    assert_eq!(shim.status, 200);
}

#[tokio::test]
async fn unset_the_console_stays_at_admin() {
    let s = spawn(Env {
        base_path: None,
        admin_path: None,
    })
    .await;
    assert_console_at(&format!("{}/admin", s.url)).await;
    let slash = get(&format!("{}/admin/", s.url), NAVIGATION).await;
    assert_eq!(slash.status, 308);
    assert_eq!(slash.headers["location"], "/admin");
    let other = get(&format!("{}/{SECRET_SEGMENT}", s.url), NAVIGATION).await;
    assert_eq!(other.status, 404);
    assert!(!other.text().contains("MW_SHELL"));
}

#[tokio::test]
async fn setting_the_default_explicitly_changes_nothing() {
    let s = spawn(Env::admin("/admin")).await;
    assert_console_at(&format!("{}/admin", s.url)).await;
}

/// Under strict CSRF the admin sign-in is exempt from the token check at the
/// configured path (no admin session exists yet), and the old path's sign-in is
/// refused exactly like any unknown path's.
#[tokio::test]
async fn under_strict_csrf_the_sign_in_follows_the_console() {
    let (web, db) = stub_web("mw-t32h15-strict");
    let s = spawn_config(Env::admin("/ops-h15q7zx"), strict_config(web, &db)).await;
    assert_console_at(&format!("{}/ops-h15q7zx", s.url)).await;
    let login = json!({ "username": ADMIN_USER, "password": ADMIN_PASS });
    let old = send(
        reqwest::Method::POST,
        &format!("{}/admin/login", s.url),
        PAGE_FETCH,
        None,
        Some(login.clone()),
    )
    .await;
    let unknown = send(
        reqwest::Method::POST,
        &format!("{}/zz-h15-none/login", s.url),
        PAGE_FETCH,
        None,
        Some(login),
    )
    .await;
    assert_eq!(old.status, unknown.status, "{}", old.text());
    assert_eq!(old.body, unknown.body);
    assert_eq!(old.comparable_headers(), unknown.comparable_headers());
}

// ── 2: the old path is an unknown path ──────────────────────────────────────────

/// One comparison: method, old admin path, unknown path, headers, JSON body.
type Case = (
    reqwest::Method,
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
    Option<serde_json::Value>,
);

#[tokio::test]
async fn the_old_admin_path_gets_exactly_the_unknown_path_answer() {
    let s = spawn(Env::admin("/ops-h15q7zx")).await;
    let login = json!({ "username": ADMIN_USER, "password": ADMIN_PASS });
    let cases: Vec<Case> = vec![
        (
            reqwest::Method::GET,
            "/admin",
            "/zz-h15-none",
            NAVIGATION,
            None,
        ),
        (
            reqwest::Method::GET,
            "/admin/",
            "/zz-h15-none/",
            NAVIGATION,
            None,
        ),
        (
            reqwest::Method::GET,
            "/admin",
            "/zz-h15-none",
            PAGE_FETCH,
            None,
        ),
        (
            reqwest::Method::GET,
            "/admin/users",
            "/zz-h15-none/users",
            PAGE_FETCH,
            None,
        ),
        (
            reqwest::Method::GET,
            "/admin/users",
            "/zz-h15-none/users",
            NAVIGATION,
            None,
        ),
        (
            reqwest::Method::GET,
            "/admin/session",
            "/zz-h15-none/session",
            PAGE_FETCH,
            None,
        ),
        (
            reqwest::Method::GET,
            "/admin/session",
            "/zz-h15-none/session",
            &[],
            None,
        ),
        (
            reqwest::Method::HEAD,
            "/admin",
            "/zz-h15-none",
            NAVIGATION,
            None,
        ),
        (
            reqwest::Method::POST,
            "/admin/login",
            "/zz-h15-none/login",
            PAGE_FETCH,
            Some(login.clone()),
        ),
        (
            reqwest::Method::POST,
            "/admin/login",
            "/zz-h15-none/login",
            &[],
            Some(login.clone()),
        ),
        (
            reqwest::Method::DELETE,
            "/admin/bans/1.2.3.4",
            "/zz-h15-none/bans/1.2.3.4",
            PAGE_FETCH,
            None,
        ),
    ];
    for (method, old, unknown, headers, body) in cases {
        let a = send(
            method.clone(),
            &format!("{}{old}", s.url),
            headers,
            None,
            body.clone(),
        )
        .await;
        let b = send(
            method.clone(),
            &format!("{}{unknown}", s.url),
            headers,
            None,
            body,
        )
        .await;
        let what = format!("{method} {old} vs {unknown}");
        assert_eq!(a.status, b.status, "{what}: status ({})", a.text());
        assert!(
            a.status == 404 || a.status == 403,
            "{what}: {} {}",
            a.status,
            a.text()
        );
        assert_eq!(
            a.comparable_headers(),
            b.comparable_headers(),
            "{what}: headers"
        );
        assert_eq!(a.body, b.body, "{what}: body");
        assert!(!a.text().contains("MW_SHELL"), "{what}: not the shell");
        assert!(
            !a.text().contains("admin console"),
            "{what}: no console link"
        );
    }
}

// ── 3: no leak ──────────────────────────────────────────────────────────────────

fn assert_no_leak(what: &str, a: &Answer) {
    assert!(
        !a.text().contains(SECRET_SEGMENT),
        "{what}: the admin path is in the body: {}",
        a.text()
    );
    for (k, v) in &a.headers {
        let v = v.to_str().unwrap_or_default();
        assert!(
            !v.contains(SECRET_SEGMENT),
            "{what}: the admin path is in the {k} header: {v}"
        );
    }
}

/// Requests outside the admin path: none of their answers may name it.
async fn scan_public_routes(url: &str, prefix: &str) -> usize {
    let mut n = 0;
    let gets = [
        "/",
        "/index.html",
        "/__mw_base.js",
        "/__mw_error.css",
        "/sw.js",
        "/favicon.svg",
        "/assets/index-AbCdEf12.js",
        "/assets/index-AbCdEf12.css",
        "/assets/missing-h15.js",
        "/robots.txt",
        "/admin",
        "/admin/",
        "/admin/session",
        "/admin/login",
        "/oauth/authorize",
        "/api/policy",
        "/api/me",
        "/api/sso/providers",
        "/api/push/vapid",
        "/api/holidays",
        "/api/discover",
        "/jmap/session",
        "/.well-known/jmap",
        "/.well-known/openpgpkey/policy",
        "/.well-known/oauth-authorization-server",
        "/healthz",
        "/metrics",
        "/mcp",
        "/zz-h15-none",
        "/zz-h15-none/deeper/still",
        "/%2e%2e/secret",
    ];
    for path in gets {
        for headers in [NAVIGATION, PAGE_FETCH, &[][..]] {
            let a = get(&format!("{url}{prefix}{path}"), headers).await;
            assert_no_leak(&format!("GET {prefix}{path}"), &a);
            n += 1;
        }
    }
    let writes = [
        (
            "/api/login",
            json!({ "email": "a@b.example", "password": "x" }),
        ),
        ("/api/logout", json!({})),
        (
            "/admin/login",
            json!({ "username": ADMIN_USER, "password": ADMIN_PASS }),
        ),
        ("/errors", json!({ "message": "x" })),
        ("/jmap/api", json!({})),
    ];
    for (path, body) in writes {
        let a = send(
            reqwest::Method::POST,
            &format!("{url}{prefix}{path}"),
            PAGE_FETCH,
            None,
            Some(body),
        )
        .await;
        assert_no_leak(&format!("POST {prefix}{path}"), &a);
        n += 1;
    }
    let preflight = send(
        reqwest::Method::OPTIONS,
        &format!("{url}{prefix}/api/login"),
        &[
            ("origin", "tauri://localhost"),
            ("access-control-request-method", "POST"),
        ],
        None,
        None,
    )
    .await;
    assert_no_leak("OPTIONS /api/login", &preflight);
    n + 1
}

#[tokio::test]
async fn the_admin_path_is_in_no_answer_outside_it() {
    let s = spawn(Env::admin("/ops-h15q7zx")).await;
    let n = scan_public_routes(&s.url, "").await;
    assert!(n > 90, "scanned {n}");
    // The console page itself does carry it nowhere either: it is located by the
    // browser's own address, not by a value the server writes.
    let page = get(&format!("{}/ops-h15q7zx", s.url), NAVIGATION).await;
    assert_eq!(page.status, 200);
}

#[tokio::test]
async fn the_admin_path_is_in_no_answer_outside_it_under_a_base_path() {
    let s = spawn(Env {
        base_path: Some("/mail"),
        admin_path: Some("/ops-h15q7zx"),
    })
    .await;
    scan_public_routes(&s.url, "/mail").await;
    scan_public_routes(&s.url, "").await;
    let bare = get(&format!("{}/mail", s.url), NAVIGATION).await;
    assert_eq!(bare.status, 308);
    assert_no_leak("GET /mail", &bare);
}

/// The real web build, when one is present: every file it serves, fetched through
/// the server, is free of the configured path (nothing in the bundle can name it,
/// and nothing the server rewrites on the way out adds it).
#[tokio::test]
async fn the_admin_path_is_in_no_file_of_the_real_build() {
    let dist = repo_root().join("apps/web/dist");
    if !dist.join("index.html").is_file() {
        eprintln!("SKIP: apps/web/dist not built");
        return;
    }
    let dir = test_db::unique_dir("mw-t32h15-dist");
    let s = spawn_with(
        Env {
            base_path: Some("/mail"),
            admin_path: Some("/ops-h15q7zx"),
        },
        dist.clone(),
        dir.join("mw.db"),
    )
    .await;
    let mut files = Vec::new();
    let mut stack = vec![dist.clone()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                files.push(p);
            }
        }
    }
    assert!(
        files.len() > 20,
        "a real build has more than {} files",
        files.len()
    );
    for f in &files {
        let rel = f
            .strip_prefix(&dist)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let a = get(&format!("{}/mail/{rel}", s.url), &[]).await;
        assert_eq!(a.status, 200, "GET /mail/{rel}");
        assert_no_leak(&format!("GET /mail/{rel}"), &a);
    }
    let shell = get(&format!("{}/mail/", s.url), NAVIGATION).await;
    assert_no_leak("GET /mail/", &shell);
    assert!(!shell.text().contains("mw-console"));
}

// ── 4: a bad value stops the start ──────────────────────────────────────────────

#[tokio::test]
async fn an_invalid_admin_path_stops_the_start() {
    let bad: &[&'static str] = &[
        "",
        "/",
        "admin2",
        "/ops/",
        "//ops",
        "/ops//x",
        "/ops/../admin",
        "/./ops",
        "/a/b/c/d/e",
        "/ops?x=1",
        "/ops#x",
        "/ops x",
        "/ops%2fx",
        "/api",
        "/API",
        "/api/console",
        "/jmap",
        "/oauth",
        "/.well-known",
        "/assets",
        "/dav",
        "/caldav",
        "/carddav",
        "/mcp",
        "/__mw_error.css",
        "/__mw_base.js",
        "/sw.js",
        "/index.html",
        "/favicon.ico",
        "/manifest.webmanifest",
        "/fonts",
        "/themes",
        "/icons",
        "/healthz",
        "/metrics",
        "/errors",
        "/t",
    ];
    for value in bad {
        let _guard = set_env(&Env::admin(value)).await;
        let (web, db) = stub_web("mw-t32h15-bad");
        let err = match build_app_full(config(web, &db), v6()).await {
            Ok(_) => panic!("MW_ADMIN_PATH={value:?} started the server"),
            Err(e) => e.to_string(),
        };
        assert!(err.contains("MW_ADMIN_PATH"), "{value:?}: {err}");
    }
    // A segment shared with the deploy prefix would be shadowed by its redirect.
    let _guard = set_env(&Env {
        base_path: Some("/mail"),
        admin_path: Some("/mail"),
    })
    .await;
    let (web, db) = stub_web("mw-t32h15-bad");
    let err = build_app_full(config(web, &db), v6())
        .await
        .err()
        .expect("MW_ADMIN_PATH equal to MW_BASE_PATH refused")
        .to_string();
    assert!(err.contains("MW_ADMIN_PATH"), "{err}");
}

#[tokio::test]
async fn valid_admin_paths_start() {
    for value in ["/ops-7f3k", "/a.b_c~d-e", "/x/y/z/w", "/Console9"] {
        let _guard = set_env(&Env::admin(value)).await;
        let (web, db) = stub_web("mw-t32h15-ok");
        if let Err(e) = build_app_full(config(web, &db), v6()).await {
            panic!("MW_ADMIN_PATH={value:?} refused: {e}");
        }
    }
}

// ── 6: one source of truth ──────────────────────────────────────────────────────

/// Every `.rs` file under `dir`.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// The shipped (non-test, non-comment) lines of every `.rs` file under
/// `crates/mw-server/src`, with their file and line number. A `#[cfg(test)] mod … {`
/// block is skipped to its closing brace.
fn shipped_lines() -> Vec<(String, usize, String)> {
    let src = repo_root().join("crates/mw-server/src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    let mut out = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap();
        let mut depth: Option<i32> = None;
        let mut armed = false;
        for (n, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if let Some(d) = depth.as_mut() {
                *d += code.matches('{').count() as i32 - code.matches('}').count() as i32;
                if *d <= 0 {
                    depth = None;
                }
                continue;
            }
            if code.starts_with("#[cfg(test)]") {
                armed = true;
                continue;
            }
            if armed && code.starts_with("mod ") && code.ends_with('{') {
                depth = Some(1);
                armed = false;
                continue;
            }
            armed = false;
            if code.starts_with("//") {
                continue;
            }
            let code = code.split(" //").next().unwrap_or(code);
            out.push((f.display().to_string(), n + 1, code.to_string()));
        }
    }
    assert!(
        out.len() > 20_000,
        "only {} lines of shipped code",
        out.len()
    );
    out
}

/// No `"/admin…"` string literal in `mw-server`'s shipped code: a route, a page
/// check, a redirect or a link written that way would stay at `/admin` when the
/// operator moves the console. Comments and `#[cfg(test)]` modules are skipped
/// (tests run with the default path).
#[test]
fn no_admin_literal_outside_the_helper() {
    let mut found = Vec::new();
    for (file, n, code) in shipped_lines() {
        for needle in [
            "\"/admin",
            "\"admin/",
            "/admin\"",
            "| \"admin\"",
            "\"admin\" |",
        ] {
            if code.contains(needle) {
                found.push(format!("{file}:{n}: {code}"));
            }
        }
    }
    assert!(
        found.is_empty(),
        "admin path literals outside mw_admin::config::admin_path/admin_route:\n{}",
        found.join("\n")
    );
}

/// Every first segment a route is mounted under, and every file at the root of the
/// web build, is one the admin path may not take
/// (`mw_admin::config::RESERVED_ADMIN_SEGMENTS`), so a console path can never
/// shadow a route or a file, or be shadowed by one.
#[test]
fn the_reserved_segments_cover_every_mounted_prefix_and_root_file() {
    let reserved = |seg: &str| {
        mw_admin::config::RESERVED_ADMIN_SEGMENTS
            .iter()
            .any(|r| r.eq_ignore_ascii_case(seg))
    };
    let mut missing = Vec::new();
    let mut seen = 0;
    let mut after_open = false;
    for (file, n, code) in shipped_lines() {
        let literal = if after_open {
            code.strip_prefix('"')
        } else {
            [".route(\"", "nest(\"", "nest_service(\""]
                .iter()
                .find_map(|open| code.find(open).map(|i| &code[i + open.len() - 1..]))
                .and_then(|rest| rest.strip_prefix('"'))
        };
        after_open = code.ends_with(".route(") || code.ends_with("nest(");
        let Some(literal) = literal else { continue };
        let Some(path) = literal.strip_prefix('/') else {
            continue;
        };
        let first = path.split(['/', '"']).next().unwrap_or_default();
        seen += 1;
        if !reserved(first) {
            missing.push(format!("{file}:{n}: /{first}"));
        }
    }
    assert!(seen > 100, "only {seen} route literals found");
    for dir in ["apps/web/public", "apps/web/dist"] {
        let Ok(entries) = std::fs::read_dir(repo_root().join(dir)) else {
            continue;
        };
        for e in entries {
            let name = e.unwrap().file_name().to_string_lossy().into_owned();
            if !name.starts_with('.') && !reserved(&name) {
                missing.push(format!("{dir}/{name}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "not in RESERVED_ADMIN_SEGMENTS:\n{}",
        missing.join("\n")
    );
}
