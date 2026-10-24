//! t32-h8 — two ways the admin console failed to load, through the real router.
//!
//! 1. **A navigation fetched by a service worker.** A worker that answers a
//!    navigation with `fetch(event.request)` sends it with `Sec-Fetch-Dest: empty`
//!    (Chromium, observed on the wire), while `Sec-Fetch-Mode` stays `navigate`.
//!    `wants_spa_shell` read only the destination, so `/admin` (and any other
//!    pathname deep link) got the plain-text 404 whenever a worker was in control.
//!    `navigate` + `empty` now gets the shell; `empty` with any other mode (a
//!    `fetch()` from the page) is still a 404, as are subresources and API paths.
//! 2. **`/admin/` with a trailing slash.** The shell's asset URLs are relative
//!    (`./assets/…`), so at `/admin/` they resolve to `/admin/assets/…` and 404. It
//!    is now a 308 to `/admin`, keeping the query; the browser keeps the fragment
//!    (`/admin/#/users` → `/admin#/users`). Under `MW_BASE_PATH=/mail` the same
//!    holds for `/mail/admin/` → `/mail/admin`.

use std::path::PathBuf;
use std::sync::OnceLock;

use reqwest::redirect::Policy;
use tokio::sync::{Mutex, MutexGuard};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, build_app};

mod common;
use common::test_db;

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=root>MW_SHELL</div>";

/// What Chromium sends for a top-level navigation the browser makes itself.
const NAVIGATION: &[(&str, &str)] = &[
    (
        "accept",
        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    ),
    ("sec-fetch-dest", "document"),
    ("sec-fetch-mode", "navigate"),
];

/// The same navigation after a service worker re-fetched it with
/// `fetch(event.request)`: only the destination changes.
const WORKER_NAVIGATION: &[(&str, &str)] = &[
    (
        "accept",
        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    ),
    ("sec-fetch-dest", "empty"),
    ("sec-fetch-mode", "navigate"),
];

/// A `fetch()` from the page for an extensionless path.
const PAGE_FETCH: &[(&str, &str)] = &[
    ("accept", "*/*"),
    ("sec-fetch-dest", "empty"),
    ("sec-fetch-mode", "cors"),
];

/// Serialises the `MW_BASE_PATH` mutation every `build_app` in this binary reads.
fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(Default::default)
}

/// Start a proxy-mode server serving a stub `index.html`, under `base_path` if
/// given. The guard is held until the test ends.
async fn spawn_server(base_path: Option<&str>) -> (String, MutexGuard<'static, ()>) {
    let guard = env_lock().lock().await;
    // SAFETY: the guard serialises every mutation and every read that depends on one
    // within this binary (house pattern, see `t20_forwarded_spoof.rs`).
    unsafe {
        match base_path {
            Some(v) => std::env::set_var("MW_BASE_PATH", v),
            None => std::env::remove_var("MW_BASE_PATH"),
        }
    }
    let dir = test_db::unique_dir("mw-t32h8");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), guard)
}

struct Answer {
    status: u16,
    content_type: String,
    location: Option<String>,
    url: String,
    body: String,
}

async fn send(
    method: reqwest::Method,
    url: &str,
    headers: &[(&str, &str)],
    follow: bool,
) -> Answer {
    let client = reqwest::Client::builder()
        .redirect(if follow {
            Policy::limited(5)
        } else {
            Policy::none()
        })
        .build()
        .unwrap();
    let mut req = client.request(method, url);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    let resp = req.send().await.unwrap();
    let header = |name: reqwest::header::HeaderName| {
        resp.headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
    };
    let content_type = header(reqwest::header::CONTENT_TYPE).unwrap_or_default();
    let location = header(reqwest::header::LOCATION);
    let status = resp.status().as_u16();
    let url = resp.url().to_string();
    Answer {
        status,
        content_type,
        location,
        url,
        body: resp.text().await.unwrap(),
    }
}

async fn get(url: &str, headers: &[(&str, &str)]) -> Answer {
    send(reqwest::Method::GET, url, headers, false).await
}

fn assert_shell(a: &Answer, what: &str) {
    assert_eq!(a.status, 200, "{what}: status ({})", a.body);
    assert!(
        a.content_type.starts_with("text/html"),
        "{what}: content-type {:?}",
        a.content_type
    );
    assert!(a.body.contains("MW_SHELL"), "{what}: body {:?}", a.body);
}

fn assert_plain_404(a: &Answer, what: &str) {
    assert_eq!(a.status, 404, "{what}: status");
    assert!(
        !a.content_type.starts_with("text/html") && !a.body.contains("MW_SHELL"),
        "{what}: a 404 that is not the shell, got {:?} {:?}",
        a.content_type,
        a.body
    );
}

/// A navigation a service worker re-fetched gets the shell, at the root and under a
/// sub-path; the browser's own navigation still does (the control).
#[tokio::test]
async fn a_worker_fetched_navigation_gets_the_shell() {
    for prefix in [None, Some("/mail")] {
        let (base, _guard) = spawn_server(prefix).await;
        let p = prefix.unwrap_or("");
        // The app's own pages. Any other path is a 404 page since t32-h10
        // (`t32_h10_errors.rs`), so `/settings/keys` is no longer in this list.
        for path in ["/admin", "/oauth/authorize", "/"] {
            let url = format!("{base}{p}{path}");
            assert_shell(
                &get(&url, NAVIGATION).await,
                &format!("{p}{path} (browser)"),
            );
            assert_shell(
                &get(&url, WORKER_NAVIGATION).await,
                &format!("{p}{path} (through a service worker)"),
            );
        }
    }
}

/// Not widened beyond navigations: a page `fetch()`, a subresource and an unmounted
/// API path still get the plain 404, whatever the worker-navigation rule says.
#[tokio::test]
async fn only_navigations_get_the_shell() {
    let (base, _guard) = spawn_server(None).await;

    assert_plain_404(
        &get(&format!("{base}/admin"), PAGE_FETCH).await,
        "a fetch() of /admin",
    );
    // `empty` with a mode that is not `navigate`, one by one.
    for mode in ["cors", "same-origin", "no-cors"] {
        let a = get(
            &format!("{base}/settings/keys"),
            &[
                ("accept", "text/html"),
                ("sec-fetch-dest", "empty"),
                ("sec-fetch-mode", mode),
            ],
        )
        .await;
        assert_plain_404(&a, &format!("dest=empty mode={mode}"));
    }
    // A subresource extension settles it before any header.
    assert_plain_404(
        &get(&format!("{base}/fonts/inter-400.woff2"), WORKER_NAVIGATION).await,
        "a missing font, navigate+empty",
    );
    assert_plain_404(
        &get(
            &format!("{base}/assets/index-deadbeef.js"),
            WORKER_NAVIGATION,
        )
        .await,
        "a missing chunk, navigate+empty",
    );
    // A write is never a navigation.
    let post = send(
        reqwest::Method::POST,
        &format!("{base}/mcp"),
        WORKER_NAVIGATION,
        false,
    )
    .await;
    assert_plain_404(&post, "POST /mcp, navigate+empty");
}

/// `/admin/` redirects to `/admin` with a 308, keeping the query; following it lands
/// on the shell.
#[tokio::test]
async fn admin_with_a_trailing_slash_redirects_to_admin() {
    let (base, _guard) = spawn_server(None).await;

    let a = get(&format!("{base}/admin/"), NAVIGATION).await;
    assert_eq!(a.status, 308, "/admin/ status ({})", a.body);
    assert_eq!(a.location.as_deref(), Some("/admin"));

    let a = get(&format!("{base}/admin/?next=users"), NAVIGATION).await;
    assert_eq!(a.status, 308);
    assert_eq!(a.location.as_deref(), Some("/admin?next=users"));

    // A worker-fetched navigation is redirected the same way.
    let a = get(&format!("{base}/admin/"), WORKER_NAVIGATION).await;
    assert_eq!(a.status, 308);
    assert_eq!(a.location.as_deref(), Some("/admin"));

    let followed = send(
        reqwest::Method::GET,
        &format!("{base}/admin/"),
        NAVIGATION,
        true,
    )
    .await;
    assert_eq!(followed.url, format!("{base}/admin"));
    assert_shell(&followed, "/admin/ followed");

    // Only the console's own path: `/admin` itself is the shell, an admin API route
    // is untouched, and `/administrator/` is not the console.
    assert_shell(&get(&format!("{base}/admin"), NAVIGATION).await, "/admin");
    let api = get(&format!("{base}/admin/session"), PAGE_FETCH).await;
    assert_ne!(api.status, 308, "/admin/session is not redirected");
    let other = get(&format!("{base}/administrator/"), NAVIGATION).await;
    assert_ne!(other.status, 308, "/administrator/ is not the console");
}

/// Under `MW_BASE_PATH=/mail`, `/mail/admin/` redirects to `/mail/admin`.
#[tokio::test]
async fn under_a_sub_path_the_redirect_keeps_the_prefix() {
    let (base, _guard) = spawn_server(Some("/mail")).await;

    let a = get(&format!("{base}/mail/admin/"), NAVIGATION).await;
    assert_eq!(a.status, 308, "/mail/admin/ status ({})", a.body);
    assert_eq!(a.location.as_deref(), Some("/mail/admin"));

    let a = get(&format!("{base}/mail/admin/?next=users"), NAVIGATION).await;
    assert_eq!(a.location.as_deref(), Some("/mail/admin?next=users"));

    let followed = send(
        reqwest::Method::GET,
        &format!("{base}/mail/admin/"),
        NAVIGATION,
        true,
    )
    .await;
    assert_eq!(followed.url, format!("{base}/mail/admin"));
    assert_shell(&followed, "/mail/admin/ followed");

    // The bare prefix still goes to the trailing-slash form (t20 B4), unchanged.
    let a = get(&format!("{base}/mail"), NAVIGATION).await;
    assert_eq!(a.status, 308);
    assert_eq!(a.location.as_deref(), Some("/mail/"));
}
