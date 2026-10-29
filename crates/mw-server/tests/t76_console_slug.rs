//! t76 — the admin console is reachable only at its configured path.
//!
//! These tests run the start-up the way `mailwoman serve` does
//! (`console_path::provision_on_first_start`), so nothing falls back to `/admin`:
//!
//! 1. **A new installation** gets a generated path (stored, logged once); `/admin`
//!    gets exactly what an unknown path gets; the console, its sign-in and its API
//!    answer at the generated path; the admin cookie is scoped to it.
//! 2. **An upgraded installation** (the database already holds data) keeps
//!    `/admin`, with a warning that recommends a path of its own.
//! 3. **A change** made in the console needs step-up, is validated, is audited,
//!    and is applied by the next start, which ends every admin session.
//! 4. **`MW_ADMIN_PATH` pins the path**: the console refuses to change it.
//! 5. **No leak**: no answer outside the console names the generated path.
//! 6. **The CLI** shows the path and stores a new one under the same rules.

use std::io::Write;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex as StdMutex, OnceLock};

use reqwest::redirect::Policy;
use serde_json::{Value, json};
use tokio::sync::{Mutex, MutexGuard};

use mw_server::console_path::{APPLIED_KEY, STORED_KEY};
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{ServerKey, Store};

mod common;
use common::test_db;

const KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const INDEX_HTML: &str = "<!doctype html><html><head><meta charset=\"UTF-8\" />\
<script type=\"module\" crossorigin src=\"./assets/index-AbCdEf12.js\"></script>\
</head><body><div id=\"root\">MW_SHELL</div></body></html>";
const ADMIN_USER: &str = "root";
const ADMIN_PASS: &str = "hunter2";
const META: &str = r#"<meta name="mw-console" content="admin">"#;
const NAVIGATION: &[(&str, &str)] = &[
    ("accept", "text/html,application/xhtml+xml"),
    ("sec-fetch-dest", "document"),
    ("sec-fetch-mode", "navigate"),
];
const PAGE_FETCH: &[(&str, &str)] = &[
    ("accept", "application/json"),
    ("sec-fetch-dest", "empty"),
    ("sec-fetch-mode", "cors"),
];

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(Default::default)
}

/// Serialise the process environment and the process-wide console path.
async fn lock_env(admin_path: Option<&str>) -> MutexGuard<'static, ()> {
    let guard = env_lock().lock().await;
    mw_server::console_path::provision_on_first_start();
    // SAFETY: every mutation and dependent read in this binary holds the guard.
    unsafe {
        std::env::remove_var("MW_BASE_PATH");
        match admin_path {
            Some(v) => std::env::set_var("MW_ADMIN_PATH", v),
            None => std::env::remove_var("MW_ADMIN_PATH"),
        }
    }
    guard
}

struct Site {
    web: PathBuf,
    db: PathBuf,
}

fn site(tag: &str) -> Site {
    let dir = test_db::unique_dir(tag);
    let web = dir.join("web");
    std::fs::create_dir_all(web.join("assets")).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    std::fs::write(web.join("assets/index-AbCdEf12.js"), "console.log(1)").unwrap();
    let sw = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/web/public/sw.js");
    std::fs::write(web.join("sw.js"), std::fs::read(sw).unwrap()).unwrap();
    Site {
        web,
        db: dir.join("mw.db"),
    }
}

fn config(s: &Site) -> AppConfig {
    AppConfig {
        db_path: s.db.to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(s.web.clone()),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
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

/// Log lines written while the app was built.
#[derive(Clone, Default)]
struct Captured(Arc<StdMutex<Vec<u8>>>);

impl Write for Captured {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Captured {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
    }
}

/// Build and serve the app on `s`; return its URL and what it logged at start.
async fn start(s: &Site) -> (String, String) {
    let logs = Captured::default();
    let writer = logs.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        .with_writer(move || writer.clone())
        .finish();
    let app = {
        let _g = tracing::subscriber::set_default(subscriber);
        build_app_full(config(s), v6())
            .await
            .expect("server boots")
            .0
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });
    (format!("http://{addr}"), logs.text())
}

async fn store_of(s: &Site) -> Store {
    Store::open(
        &s.db.to_string_lossy(),
        ServerKey::from_hex(KEY_HEX).unwrap(),
    )
    .await
    .unwrap()
}

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
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or(Value::Null)
    }
}

async fn send(
    method: reqwest::Method,
    url: &str,
    headers: &[(&str, &str)],
    cookie: Option<&str>,
    body: Option<Value>,
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
    Answer {
        status: resp.status().as_u16(),
        headers: resp.headers().clone(),
        body: resp.text().await.unwrap(),
    }
}

async fn get(url: &str, headers: &[(&str, &str)], cookie: Option<&str>) -> Answer {
    send(reqwest::Method::GET, url, headers, cookie, None).await
}

fn set_cookies(a: &Answer) -> Vec<String> {
    a.headers
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok().map(str::to_string))
        .collect()
}

fn cookie_pair(a: &Answer, name: &str) -> Option<String> {
    set_cookies(a).into_iter().find_map(|c| {
        let first = c.split(';').next()?.to_string();
        let (k, v) = first.split_once('=')?;
        (k.trim() == name && !v.is_empty()).then_some(first)
    })
}

struct Admin {
    cookie: String,
    csrf: String,
    set_cookie: String,
}

/// Sign in at `{console}/login`.
async fn sign_in(console: &str) -> Admin {
    let a = send(
        reqwest::Method::POST,
        &format!("{console}/login"),
        &[],
        None,
        Some(json!({ "username": ADMIN_USER, "password": ADMIN_PASS })),
    )
    .await;
    assert_eq!(a.status, 200, "admin login at {console}: {}", a.body);
    let session = cookie_pair(&a, "mw_admin_session").expect("admin cookie");
    let csrf = cookie_pair(&a, "mw_csrf").expect("csrf cookie");
    let set_cookie = set_cookies(&a)
        .into_iter()
        .find(|c| c.starts_with("mw_admin_session="))
        .unwrap();
    Admin {
        cookie: format!("{session}; {csrf}"),
        csrf: csrf.split_once('=').unwrap().1.to_string(),
        set_cookie,
    }
}

async fn step_up(console: &str, admin: &Admin) {
    let a = send(
        reqwest::Method::POST,
        &format!("{console}/step-up"),
        &[("x-csrf-token", &admin.csrf)],
        Some(&admin.cookie),
        Some(json!({ "password": ADMIN_PASS })),
    )
    .await;
    assert_eq!(a.status, 200, "step-up: {}", a.body);
}

async fn read_path(console: &str, admin: &Admin) -> (Value, String) {
    let a = get(
        &format!("{console}/console-path"),
        PAGE_FETCH,
        Some(&admin.cookie),
    )
    .await;
    assert_eq!(a.status, 200, "GET console-path: {}", a.body);
    let tag = a.headers["etag"].to_str().unwrap().to_string();
    (a.json(), tag)
}

async fn put_path(console: &str, admin: &Admin, tag: &str, body: Value) -> Answer {
    send(
        reqwest::Method::PUT,
        &format!("{console}/console-path"),
        &[("x-csrf-token", &admin.csrf), ("if-match", tag)],
        Some(&admin.cookie),
        Some(body),
    )
    .await
}

/// `path` answers exactly like a path nothing serves.
async fn assert_unknown(url: &str, path: &str) {
    for headers in [NAVIGATION, PAGE_FETCH] {
        let got = get(&format!("{url}{path}"), headers, None).await;
        let unknown = get(&format!("{url}/zz-t76-nothing-here"), headers, None).await;
        assert_eq!(got.status, 404, "{path}: {}", got.body);
        assert_eq!(got.status, unknown.status, "{path}");
        assert_eq!(got.body, unknown.body, "{path}");
    }
    let login = send(
        reqwest::Method::POST,
        &format!("{url}{path}/login"),
        &[],
        None,
        Some(json!({ "username": ADMIN_USER, "password": ADMIN_PASS })),
    )
    .await;
    let unknown = send(
        reqwest::Method::POST,
        &format!("{url}/zz-t76-nothing-here/login"),
        &[],
        None,
        Some(json!({ "username": ADMIN_USER, "password": ADMIN_PASS })),
    )
    .await;
    assert_eq!(login.status, unknown.status, "POST {path}/login");
    assert_eq!(login.body, unknown.body, "POST {path}/login");
}

async fn assert_console(url: &str, path: &str) {
    let page = get(&format!("{url}{path}"), NAVIGATION, None).await;
    assert_eq!(page.status, 200, "console page at {path}: {}", page.body);
    assert!(page.body.contains(META), "console marker at {path}");
    let csp: Vec<_> = page
        .headers
        .get_all("content-security-policy")
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect();
    assert!(
        csp.iter().any(|c| c.contains("frame-ancestors 'none'")),
        "{csp:?}"
    );
    assert_eq!(page.headers["x-robots-tag"], "noindex, nofollow");
}

#[tokio::test]
async fn a_new_installation_gets_a_generated_path_and_admin_is_unknown() {
    let _g = lock_env(None).await;
    let s = site("mw-t76-new");
    let (url, logs) = start(&s).await;
    let stored = store_of(&s)
        .await
        .get_setting(STORED_KEY)
        .await
        .unwrap()
        .expect("the generated path is stored");
    assert_eq!(stored.len(), 21, "{stored}");
    assert!(mw_admin::config::validate_chosen_admin_path(&stored).is_ok());
    assert!(
        logs.contains("new installation") && logs.contains(&format!("{stored}/")),
        "the generated path is logged once: {logs}"
    );
    assert_unknown(&url, "/admin").await;
    assert_console(&url, &stored).await;
    let console = format!("{url}{stored}");
    let admin = sign_in(&console).await;
    assert!(
        admin.set_cookie.contains(&format!("Path={stored};")),
        "the admin cookie is scoped to the console: {}",
        admin.set_cookie
    );
    let (view, _) = read_path(&console, &admin).await;
    assert_eq!(view["path"], stored.as_str());
    assert_eq!(view["source"], "generated");
    assert_eq!(view["legacyDefault"], false);

    // A second start reads the stored path; nothing is generated or logged again.
    let (url2, logs2) = start(&s).await;
    assert!(!logs2.contains(&stored), "logged once only: {logs2}");
    assert_console(&url2, &stored).await;
    assert_unknown(&url2, "/admin").await;
    let console2 = format!("{url2}{stored}");
    let admin2 = sign_in(&console2).await;
    assert_eq!(read_path(&console2, &admin2).await.0["source"], "stored");

    // Sign-out clears the scoped cookie and the pre-t76 `Path=/` one.
    let out = send(
        reqwest::Method::POST,
        &format!("{console2}/logout"),
        &[("x-csrf-token", &admin2.csrf)],
        Some(&admin2.cookie),
        None,
    )
    .await;
    let cleared = set_cookies(&out);
    assert!(
        cleared
            .iter()
            .any(|c| c.contains(&format!("Path={stored};"))),
        "{cleared:?}"
    );
    assert!(cleared.iter().any(|c| c.contains("Path=/;")), "{cleared:?}");
}

#[tokio::test]
async fn an_upgraded_installation_keeps_admin_with_a_warning() {
    let _g = lock_env(None).await;
    let s = site("mw-t76-upgrade");
    {
        // What an earlier version leaves behind.
        let store = store_of(&s).await;
        store
            .set_setting("domain.upstream.imported", "1")
            .await
            .unwrap();
    }
    let (url, logs) = start(&s).await;
    assert!(
        logs.contains("the first path a scanner tries"),
        "the upgrade warning: {logs}"
    );
    let store = store_of(&s).await;
    assert_eq!(
        store.get_setting(STORED_KEY).await.unwrap().as_deref(),
        Some("/admin")
    );
    assert_console(&url, "/admin").await;
    let console = format!("{url}/admin");
    let admin = sign_in(&console).await;
    assert!(
        admin.set_cookie.contains("Path=/admin;"),
        "{}",
        admin.set_cookie
    );
    let (view, _) = read_path(&console, &admin).await;
    assert_eq!(view["source"], "upgrade");
    assert_eq!(view["legacyDefault"], true);

    // Every later start warns again until the path is changed.
    let (_, logs2) = start(&s).await;
    assert!(logs2.contains("the first path a scanner tries"), "{logs2}");
}

#[tokio::test]
async fn a_change_needs_step_up_is_checked_and_applies_at_the_next_start() {
    let _g = lock_env(None).await;
    let s = site("mw-t76-change");
    let (url, _) = start(&s).await;
    let generated = store_of(&s)
        .await
        .get_setting(STORED_KEY)
        .await
        .unwrap()
        .unwrap();
    let console = format!("{url}{generated}");
    let admin = sign_in(&console).await;
    let (_, tag) = read_path(&console, &admin).await;

    // Without a fresh confirmation: 403 step_up_required, nothing stored.
    let refused = put_path(&console, &admin, &tag, json!({"path": "/ops-console-t76x"})).await;
    assert_eq!(refused.status, 403, "{}", refused.body);
    assert_eq!(refused.json()["code"], "step_up_required");
    assert_eq!(refused.json()["action"], "consolePath");

    step_up(&console, &admin).await;
    for bad in [
        json!({"path": "/admin"}),
        json!({"path": "/short"}),
        json!({"path": "/api/abcdefghijklmn"}),
        json!({"path": "/assets-x/../abcdefghijkl"}),
        json!({"path": "/ops-console-t76x", "generate": true}),
        json!({}),
    ] {
        let a = put_path(&console, &admin, &tag, bad.clone()).await;
        assert_eq!(a.status, 422, "{bad}: {}", a.body);
    }
    let stale = put_path(
        &console,
        &admin,
        "\"0\"",
        json!({"path": "/ops-console-t76x"}),
    )
    .await;
    assert_eq!(stale.status, 412, "{}", stale.body);

    let ok = put_path(&console, &admin, &tag, json!({"path": "/ops-console-t76x"})).await;
    assert_eq!(ok.status, 200, "{}", ok.body);
    assert_eq!(ok.json()["restartRequired"], true);
    assert_eq!(ok.json()["pending"], "/ops-console-t76x");
    assert_eq!(ok.json()["path"], generated.as_str());

    // Audited without the path itself.
    let store = store_of(&s).await;
    let audit = store.list_audit(50).await.unwrap();
    let row = audit
        .iter()
        .find(|r| r.action == "admin-console-path-changed")
        .expect("audit entry");
    assert!(!format!("{row:?}").contains("ops-console-t76x"), "{row:?}");

    // Until the restart the console stays where it is.
    assert_console(&url, &generated).await;
    assert_unknown(&url, "/ops-console-t76x").await;

    // The next start applies it and ends every admin session.
    let (url2, logs2) = start(&s).await;
    assert!(logs2.contains("ended 1 admin session"), "{logs2}");
    assert_console(&url2, "/ops-console-t76x").await;
    assert_unknown(&url2, &generated).await;
    assert_unknown(&url2, "/admin").await;
    let old = get(
        &format!("{url2}/ops-console-t76x/session"),
        PAGE_FETCH,
        Some(&admin.cookie),
    )
    .await;
    assert_eq!(old.status, 401, "the old session ended: {}", old.body);
    assert_eq!(
        store.get_setting(APPLIED_KEY).await.unwrap().as_deref(),
        Some("/ops-console-t76x")
    );
    let admin2 = sign_in(&format!("{url2}/ops-console-t76x")).await;
    let (view, tag2) = read_path(&format!("{url2}/ops-console-t76x"), &admin2).await;
    assert_eq!(view["pending"], Value::Null);

    // A generated one works the same way.
    step_up(&format!("{url2}/ops-console-t76x"), &admin2).await;
    let gen_ = put_path(
        &format!("{url2}/ops-console-t76x"),
        &admin2,
        &tag2,
        json!({"generate": true}),
    )
    .await;
    assert_eq!(gen_.status, 200, "{}", gen_.body);
    let pending = gen_.json()["pending"].as_str().unwrap().to_string();
    assert_eq!(pending.len(), 21);
}

#[tokio::test]
async fn mw_admin_path_pins_the_path() {
    let _g = lock_env(Some("/ops-pinned-t76")).await;
    let s = site("mw-t76-pinned");
    let (url, _) = start(&s).await;
    assert_unknown(&url, "/admin").await;
    let console = format!("{url}/ops-pinned-t76");
    let admin = sign_in(&console).await;
    let (view, tag) = read_path(&console, &admin).await;
    assert_eq!(view["source"], "env");
    assert_eq!(view["pinnedBy"], "MW_ADMIN_PATH");
    step_up(&console, &admin).await;
    let a = put_path(&console, &admin, &tag, json!({"generate": true})).await;
    assert_eq!(a.status, 409, "{}", a.body);
    assert_eq!(a.json()["code"], "pinnedByEnvironment");
    drop(_g);

    // MW_ADMIN_PATH=/admin is honoured (an operator's explicit choice), with the warning.
    let _g = lock_env(Some("/admin")).await;
    let s = site("mw-t76-pinned-admin");
    let (url, logs) = start(&s).await;
    assert!(logs.contains("the first path a scanner tries"), "{logs}");
    assert_console(&url, "/admin").await;
}

#[tokio::test]
async fn no_answer_outside_the_console_names_the_generated_path() {
    let _g = lock_env(None).await;
    let s = site("mw-t76-leak");
    let (url, _) = start(&s).await;
    let path = store_of(&s)
        .await
        .get_setting(STORED_KEY)
        .await
        .unwrap()
        .unwrap();
    let secret = path.trim_start_matches('/').to_string();
    for (p, headers) in [
        ("/", NAVIGATION),
        ("/index.html", NAVIGATION),
        ("/sw.js", PAGE_FETCH),
        ("/__mw_base.js", PAGE_FETCH),
        ("/assets/index-AbCdEf12.js", PAGE_FETCH),
        ("/api/public-config", PAGE_FETCH),
        ("/api/me", PAGE_FETCH),
        ("/api/session", PAGE_FETCH),
        ("/robots.txt", PAGE_FETCH),
        ("/manifest.webmanifest", PAGE_FETCH),
        ("/.well-known/security.txt", PAGE_FETCH),
        ("/admin", NAVIGATION),
        ("/admin/login", PAGE_FETCH),
        ("/zz-unknown", NAVIGATION),
        ("/zz-unknown", PAGE_FETCH),
        ("/oauth/authorize", NAVIGATION),
    ] {
        let a = get(&format!("{url}{p}"), headers, None).await;
        assert!(
            !a.body.contains(&secret),
            "{p} names the console: {}",
            a.body
        );
        for (k, v) in &a.headers {
            assert!(
                !v.to_str().unwrap_or_default().contains(&secret),
                "{p} header {k} names the console"
            );
        }
    }
    // The admin cookie is never sent outside the console: its Path says so.
    let admin = sign_in(&format!("{url}{path}")).await;
    assert!(admin.set_cookie.contains(&format!("Path={path};")));
    assert!(!admin.set_cookie.contains("Path=/;"));
}

#[tokio::test]
async fn the_cli_shows_and_stores_the_path_under_the_same_rules() {
    use mw_admin::cli::ConsolePathVerb;
    let _g = lock_env(None).await;
    let s = site("mw-t76-cli");
    let store = store_of(&s).await;
    let shown = mw_server::console_path::cli(&store, ConsolePathVerb::Show)
        .await
        .unwrap();
    assert!(shown.contains("not configured yet"), "{shown}");
    assert!(
        mw_server::console_path::cli(
            &store,
            ConsolePathVerb::Set {
                path: "/admin".into()
            }
        )
        .await
        .is_err()
    );
    assert!(
        mw_server::console_path::cli(
            &store,
            ConsolePathVerb::Set {
                path: "/short".into()
            }
        )
        .await
        .is_err()
    );
    mw_server::console_path::cli(
        &store,
        ConsolePathVerb::Set {
            path: "/ops-from-cli-t76".into(),
        },
    )
    .await
    .unwrap();
    let shown = mw_server::console_path::cli(&store, ConsolePathVerb::Show)
        .await
        .unwrap();
    assert!(shown.contains("/ops-from-cli-t76/"), "{shown}");
    let (url, _) = start(&s).await;
    assert_console(&url, "/ops-from-cli-t76").await;
    assert_unknown(&url, "/admin").await;
    drop(_g);

    let _g = lock_env(Some("/ops-pinned-t76")).await;
    let err = mw_server::console_path::cli(&store, ConsolePathVerb::Generate)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("MW_ADMIN_PATH"), "{err}");
}
