//! t32-f13 (e82) — the client version header across origins.
//!
//! The shells send `X-Mailwoman-Client: desktop|mobile/<version>; updater=on|off`
//! on sign-in and on every request they make to the server, and the web client
//! sends `browser/<version>` on sign-in (t32-cfg §4.18). A shell talks to its
//! server from a Tauri origin, so the header is a non-safelisted request header:
//! the webview sends a preflight listing it, and the request is sent only when
//! `Access-Control-Allow-Headers` names it. These tests drive that over the real
//! router:
//!
//! * a preflight from each Tauri origin that lists the header (with
//!   `authorization` and `content-type`, as the shell sends them) is answered with
//!   every listed header allowed;
//! * a native sign-in carrying the header succeeds and its answer is readable
//!   from the Tauri origin;
//! * a browser sign-in from the server's own origin carrying the header succeeds;
//! * another origin still gets no CORS headers for the same preflight.
//!
//! SQLite only; proxy mode against the JMAP mock, strict CSRF on.

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};
use tokio::net::TcpListener;

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

mod common;
use common::test_db;

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

/// The three origins a Tauri 2 webview loads the SPA from.
const TAURI_ORIGINS: [&str; 3] = [
    "http://tauri.localhost",
    "https://tauri.localhost",
    "tauri://localhost",
];
const EVIL: &str = "https://evil.example";

/// The header and the values the clients send (`apps/web/src/api/transport.ts`).
const CLIENT_HEADER: &str = "x-mailwoman-client";
const DESKTOP: &str = "desktop/26.19.0; updater=off";
const BROWSER: &str = "browser/26.19.0";

/// What the shell's webview lists in `Access-Control-Request-Headers` before a
/// signed-in write: the bearer token, the JSON body and the client header.
const REQUESTED: &str = "authorization, content-type, x-mailwoman-client";

async fn spawn_mock() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, mw_mock_jmap::router()).await;
    });
    format!("http://{addr}")
}

/// Boot the real app in proxy mode with strict CSRF on and `MW_NATIVE_ORIGINS`
/// unset, as `deploy.sh` leaves it.
async fn spawn_server(upstream: &str) -> String {
    let dir = test_db::unique_dir("mw-t32-f13-client");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig {
            csrf_strict: true,
            ..HardeningConfig::default()
        },
        security: SecurityConfig {
            jmap_upstreams: Some(vec![upstream.to_string()]),
            ..SecurityConfig::default()
        },
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

/// No cookie jar and no redirects: a request sends what the test gives it.
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

fn header(resp: &reqwest::Response, name: reqwest::header::HeaderName) -> Option<String> {
    resp.headers()
        .get(name)
        .map(|v| v.to_str().unwrap().to_string())
}

/// The lower-cased, trimmed entries of a comma-separated header value.
fn listed(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(',')
        .map(|h| h.trim().to_ascii_lowercase())
        .filter(|h| !h.is_empty())
        .collect()
}

fn login_body(native: bool) -> Value {
    let mut body = json!({
        "username": mw_mock_jmap::USER,
        "password": mw_mock_jmap::PASS,
    });
    if native {
        body["clientType"] = json!("native");
    }
    body
}

#[tokio::test]
async fn a_tauri_preflight_listing_the_client_header_is_allowed() {
    let mock = spawn_mock().await;
    let base = spawn_server(&mock).await;
    let c = client();

    for origin in TAURI_ORIGINS {
        for (method, path) in [
            ("POST", "/api/login"),
            ("PUT", "/api/account/appearance"),
            ("GET", "/api/me"),
        ] {
            let resp = c
                .request(reqwest::Method::OPTIONS, format!("{base}{path}"))
                .header(reqwest::header::ORIGIN, origin)
                .header("access-control-request-method", method)
                .header("access-control-request-headers", REQUESTED)
                .send()
                .await
                .unwrap();
            let status = resp.status().as_u16();
            let allow_origin = header(&resp, reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN);
            let allow_headers = header(&resp, reqwest::header::ACCESS_CONTROL_ALLOW_HEADERS);
            assert!(
                status == 204 || status == 200,
                "{origin} {method} {path} preflight: {status}"
            );
            assert_eq!(allow_origin.as_deref(), Some(origin));
            let allowed = listed(allow_headers.as_deref());
            for wanted in listed(Some(REQUESTED)) {
                assert!(
                    allowed.contains(&wanted),
                    "{origin} {method} {path}: {wanted} missing from Access-Control-Allow-Headers {allow_headers:?}; \
                     the webview would refuse to send the request"
                );
            }
        }
    }
}

#[tokio::test]
async fn a_native_sign_in_with_the_client_header_succeeds_and_is_readable() {
    let mock = spawn_mock().await;
    let base = spawn_server(&mock).await;
    let c = client();

    for origin in TAURI_ORIGINS {
        let resp = c
            .post(format!("{base}/api/login"))
            .header(reqwest::header::ORIGIN, origin)
            .header(CLIENT_HEADER, DESKTOP)
            .json(&login_body(true))
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        let allow_origin = header(&resp, reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN);
        let body: Value = resp.json().await.unwrap();
        assert_eq!(status, 200, "native sign-in from {origin}: {body}");
        assert_eq!(allow_origin.as_deref(), Some(origin));
        assert!(
            body["token"].as_str().is_some_and(|t| !t.is_empty()),
            "{body}"
        );
    }
}

#[tokio::test]
async fn a_browser_sign_in_with_the_client_header_succeeds() {
    let mock = spawn_mock().await;
    let base = spawn_server(&mock).await;
    let c = client();

    let resp = c
        .post(format!("{base}/api/login"))
        .header(reqwest::header::ORIGIN, base.as_str())
        .header(CLIENT_HEADER, BROWSER)
        .json(&login_body(false))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let allow_origin = header(&resp, reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN);
    let cookies = resp
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .count();
    assert_eq!(status, 200, "browser sign-in with the header");
    assert!(allow_origin.is_none(), "same-origin answer carries no CORS");
    assert!(cookies > 0, "a browser sign-in sets the session cookie");
}

#[tokio::test]
async fn another_origin_gets_no_cors_for_the_client_header() {
    let mock = spawn_mock().await;
    let base = spawn_server(&mock).await;
    let c = client();

    let resp = c
        .request(reqwest::Method::OPTIONS, format!("{base}/api/login"))
        .header(reqwest::header::ORIGIN, EVIL)
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", REQUESTED)
        .send()
        .await
        .unwrap();
    assert!(
        header(&resp, reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN).is_none(),
        "{EVIL} preflight is not answered with CORS"
    );
    assert!(header(&resp, reqwest::header::ACCESS_CONTROL_ALLOW_HEADERS).is_none());
}
