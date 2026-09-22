//! t24-e14 — the SPA fallback answers **navigations**, and only navigations.
//!
//! The defect: `static_handler` answered every unmatched path with `index.html` and a
//! `200`. A subresource that does not exist therefore arrived as an HTML document:
//!
//!   * `GET /fonts/inter-400.woff2` → `200 text/html`, which Chrome reports as
//!     `OTS parsing error: invalid sfntVersion: 1008821359` — `0x3C21444F`, the ASCII
//!     bytes `<!DO`. This is what broke the `e2e-engine` job's layout.
//!   * `POST /mcp` on a proxy-mode server (where `/mcp` is deliberately not mounted)
//!     → `200 text/html`, which a JSON-RPC client reports as
//!     `SyntaxError: Unexpected token '<', "<!doctype "... is not valid JSON`. That is
//!     the `e2e-v6` `mcp.spec.ts` failure; the spec was already written to skip on a
//!     404 and never got one.
//!
//! These assertions are the ones that would have caught both. The fonts themselves
//! are an operator-supplied overlay by design (`fonts/manifest.json` +
//! `mailwoman fonts pull`, which nothing in CI runs), so a missing font file is
//! correct — the `200` was the only defect.
//!
//! The server here is PROXY mode with no engine, which is exactly the `e2e-v6`
//! configuration, so `/mcp` is genuinely absent.

use std::path::PathBuf;

use mw_server::{AppConfig, build_app};

mod common;
use common::test_db;

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_SHELL</div>";

/// What a browser sends on a real top-level navigation.
const NAVIGATION: &[(&str, &str)] = &[
    (
        "accept",
        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    ),
    ("sec-fetch-dest", "document"),
    ("sec-fetch-mode", "navigate"),
];

/// What a browser sends when the stylesheet's `@font-face` pulls a webfont.
const FONT_FETCH: &[(&str, &str)] = &[
    ("accept", "*/*"),
    ("sec-fetch-dest", "font"),
    ("sec-fetch-mode", "cors"),
];

async fn spawn_server() -> String {
    let base = test_db::unique_dir("mw-t24-spa");
    let web = base.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web as PathBuf),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().build().unwrap()
}

async fn get(base: &str, path: &str, headers: &[(&str, &str)]) -> (u16, String, String) {
    let mut req = client().get(format!("{base}{path}"));
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    let resp = req.send().await.unwrap();
    let status = resp.status().as_u16();
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    (status, content_type, resp.text().await.unwrap())
}

/// A missing subresource is `404` with no HTML body — never the shell.
#[tokio::test]
async fn a_missing_subresource_is_404_and_never_the_shell() {
    let base = spawn_server().await;

    // The traced e2e-engine failure, asserted directly.
    let (status, content_type, body) = get(&base, "/fonts/inter-400.woff2", FONT_FETCH).await;
    assert_eq!(status, 404, "a missing webfont is not found, not the shell");
    assert!(
        !content_type.starts_with("text/html"),
        "a webfont request must never be answered with HTML; content-type was \
         {content_type:?}"
    );
    assert!(
        !body.contains("MW_SHELL") && !body.trim_start().starts_with("<!"),
        "the 404 body is not the SPA shell: {body:?}"
    );
    assert!(
        !body.starts_with("<!DO"),
        "`<!DO` is the literal byte sequence the browser reported as \
         `invalid sfntVersion: 1008821359`"
    );

    // The same path with no Fetch Metadata and no useful `Accept` (curl, an HTTP
    // library, an old browser) is still 404 — the extension settles it before any
    // header is consulted.
    let (status, _, _) = get(&base, "/fonts/inter-400.woff2", &[]).await;
    assert_eq!(status, 404, "a bare client gets the same answer");

    // Other subresource shapes, each with the header set its own consumer sends.
    for (path, dest) in [
        ("/assets/index-deadbeef.js", "script"),
        ("/assets/index-deadbeef.css", "style"),
        ("/img/missing.png", "image"),
        ("/missing.wasm", "empty"),
    ] {
        let (status, content_type, _) =
            get(&base, path, &[("accept", "*/*"), ("sec-fetch-dest", dest)]).await;
        assert_eq!(status, 404, "{path} is not found");
        assert!(
            !content_type.starts_with("text/html"),
            "{path} answered {content_type:?}"
        );
    }
}

/// A client-side route still reloads into the app.
#[tokio::test]
async fn a_navigation_still_gets_the_shell() {
    let base = spawn_server().await;

    for path in ["/mail/inbox", "/mail/inbox/42", "/settings/keys"] {
        let (status, content_type, body) = get(&base, path, NAVIGATION).await;
        assert_eq!(status, 200, "{path} reloads into the SPA");
        assert!(
            content_type.starts_with("text/html"),
            "{path} answered {content_type:?}"
        );
        assert!(body.contains("MW_SHELL"), "{path} served the shell: {body}");
    }

    // `/` is served as `index.html` directly (it never reaches the fallback), and
    // must keep working for a health probe that sends no headers at all.
    let (status, content_type, body) = get(&base, "/", &[]).await;
    assert_eq!(status, 200, "the root is the shell");
    assert!(content_type.starts_with("text/html"));
    assert!(body.contains("MW_SHELL"));

    // Fetch Metadata is authoritative where it is present: a `fetch()` for an
    // extensionless path is NOT a navigation, so it does not get the shell either.
    let (status, content_type, _) = get(
        &base,
        "/mail/inbox",
        &[
            ("accept", "*/*"),
            ("sec-fetch-dest", "empty"),
            ("sec-fetch-mode", "cors"),
        ],
    )
    .await;
    assert_eq!(status, 404, "a same-origin fetch() is not a navigation");
    assert!(!content_type.starts_with("text/html"));
}

/// A non-GET that reaches the fallback is a call to a route that is not mounted.
/// `/mcp` in proxy mode is exactly that, and its absence is deliberate — see the
/// mount comment in `crates/mw-server/src/lib.rs`.
#[tokio::test]
async fn an_unmounted_post_is_404_not_a_page_of_html() {
    let base = spawn_server().await;

    let resp = client()
        .post(format!("{base}/mcp"))
        .header("content-type", "application/json")
        .body(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let body = resp.text().await.unwrap();
    assert_eq!(
        status, 404,
        "/mcp is not mounted in proxy mode; the transport must say so. \
         `apps/web/e2e/mcp.spec.ts` skips on exactly this 404"
    );
    assert!(
        !content_type.starts_with("text/html"),
        "an unmounted POST answered {content_type:?} — an HTML body here is what a \
         JSON client reports as `Unexpected token '<'`"
    );
    assert!(
        !body.contains("MW_SHELL"),
        "the unmounted POST body is not the SPA shell: {body:?}"
    );
}
