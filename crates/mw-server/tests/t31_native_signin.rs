//! t31-e8 — native shell sign-in against a remote server, strict CSRF on.
//!
//! The desktop and mobile shells load the SPA from a fixed Tauri origin
//! (`http://tauri.localhost` on Windows and Android, `tauri://localhost` on macOS,
//! iOS and Linux, `https://tauri.localhost` with `useHttpsScheme`) and talk to the
//! user's server across origins: the webview sends that `Origin` on every request
//! and, before signing in, holds no bearer token. These tests drive that path over
//! the real router:
//!
//! * `POST /api/login` from a Tauri origin with no cookie signs in and returns a
//!   bearer token; the response carries the CORS headers the webview needs to read
//!   it;
//! * a preflight for `PUT`/`PATCH`/`DELETE` from a Tauri origin is answered, and the
//!   token then authorises a `PUT` and a `DELETE` on a mailbox route;
//! * signing out with the token ends the session on the server.
//!
//! And the negatives that keep this from being a CSRF hole:
//!
//! * a Tauri-origin request that carries a session cookie is checked exactly as a
//!   cross-origin request: refused without the CSRF token, and refused for its
//!   origin with it;
//! * a sign-in from any other origin is still refused, and gets no CORS headers;
//! * a cookie-less Tauri-origin write to a route other than the sign-in routes is
//!   still refused for its origin;
//! * a bearer-only request from another origin passes the guard (a bearer token is
//!   no ambient authority) but gets no CORS headers, so a web page cannot read it.
//!
//! Every refusal is paired with the request that succeeds, so a test cannot pass
//! because a route was unreachable. SQLite only; proxy mode against the JMAP mock.

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
/// The Windows/Android origin; the one most cases use.
const TAURI: &str = "http://tauri.localhost";
const EVIL: &str = "https://evil.example";

const CSRF_REFUSAL: &str = "missing or invalid CSRF token";
const ORIGIN_REFUSAL: &str = "cross-origin request rejected";

const APPEARANCE: &str = "/api/account/appearance";

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
    let dir = test_db::unique_dir("mw-t31-native");
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

/// What a response said, plus the CORS headers it carried.
#[derive(Debug)]
struct Answer {
    status: u16,
    body: String,
    allow_origin: Option<String>,
    allow_methods: Option<String>,
    allow_credentials: Option<String>,
    set_cookie: Vec<String>,
}

fn header(resp: &reqwest::Response, name: reqwest::header::HeaderName) -> Option<String> {
    resp.headers()
        .get(name)
        .map(|v| v.to_str().unwrap().to_string())
}

async fn answer(resp: reqwest::Response) -> Answer {
    let allow_origin = header(&resp, reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN);
    let allow_methods = header(&resp, reqwest::header::ACCESS_CONTROL_ALLOW_METHODS);
    let allow_credentials = header(&resp, reqwest::header::ACCESS_CONTROL_ALLOW_CREDENTIALS);
    let set_cookie = resp
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect();
    let status = resp.status().as_u16();
    Answer {
        status,
        body: resp.text().await.unwrap(),
        allow_origin,
        allow_methods,
        allow_credentials,
        set_cookie,
    }
}

/// One request with exactly the headers given.
#[derive(Default)]
struct Req<'a> {
    origin: Option<&'a str>,
    cookie: Option<&'a str>,
    csrf: Option<&'a str>,
    bearer: Option<&'a str>,
    body: Option<Value>,
}

async fn send(c: &reqwest::Client, method: &str, url: String, r: Req<'_>) -> Answer {
    let mut req = c.request(method.parse().unwrap(), url);
    if let Some(origin) = r.origin {
        req = req.header(reqwest::header::ORIGIN, origin);
    }
    if let Some(cookie) = r.cookie {
        req = req.header(reqwest::header::COOKIE, cookie);
    }
    if let Some(csrf) = r.csrf {
        req = req.header("x-csrf-token", csrf);
    }
    if let Some(token) = r.bearer {
        req = req.header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"));
    }
    if let Some(body) = r.body {
        req = req.json(&body);
    }
    answer(req.send().await.unwrap()).await
}

fn login_body(_mock: &str, native: bool) -> Value {
    let mut body = json!({
        "username": mw_mock_jmap::USER,
        "password": mw_mock_jmap::PASS,
    });
    if native {
        body["clientType"] = json!("native");
    }
    body
}

/// A preflight as a webview sends it before a cross-origin write.
async fn preflight(c: &reqwest::Client, url: String, origin: &str, method: &str) -> Answer {
    let resp = c
        .request(reqwest::Method::OPTIONS, url)
        .header(reqwest::header::ORIGIN, origin)
        .header("access-control-request-method", method)
        .header(
            "access-control-request-headers",
            "authorization, content-type",
        )
        .send()
        .await
        .unwrap();
    answer(resp).await
}

/// Sign in from `origin` as the shell does: no cookie, `clientType: "native"`.
/// Returns the bearer token.
async fn native_login(c: &reqwest::Client, base: &str, mock: &str, origin: &str) -> String {
    let a = send(
        c,
        "POST",
        format!("{base}/api/login"),
        Req {
            origin: Some(origin),
            body: Some(login_body(mock, true)),
            ..Req::default()
        },
    )
    .await;
    assert_eq!(a.status, 200, "native sign-in from {origin}: {a:?}");
    assert!(
        a.set_cookie.is_empty(),
        "a native sign-in sets no cookie: {a:?}"
    );
    assert_eq!(
        a.allow_origin.as_deref(),
        Some(origin),
        "the webview can read the answer only with Access-Control-Allow-Origin: {a:?}"
    );
    assert!(a.allow_credentials.is_none(), "no credentialed CORS: {a:?}");
    let body: Value = serde_json::from_str(&a.body).unwrap();
    body["token"]
        .as_str()
        .expect("token in the body")
        .to_string()
}

#[tokio::test]
async fn a_tauri_origin_signs_in_without_a_cookie_and_its_token_writes() {
    let mock = spawn_mock().await;
    let base = spawn_server(&mock).await;
    let c = client();

    for origin in TAURI_ORIGINS {
        let token = native_login(&c, &base, &mock, origin).await;

        // The preflight for each write method is answered for this origin.
        for method in ["PUT", "PATCH", "DELETE"] {
            let p = preflight(&c, format!("{base}{APPEARANCE}"), origin, method).await;
            assert!(
                p.status == 204 || p.status == 200,
                "{origin} {method} preflight: {p:?}"
            );
            assert_eq!(p.allow_origin.as_deref(), Some(origin), "{p:?}");
            let methods = p.allow_methods.clone().unwrap_or_default();
            assert!(
                methods.split(',').any(|m| m.trim() == method),
                "{origin}: {method} missing from Access-Control-Allow-Methods {methods:?}"
            );
            assert!(p.allow_credentials.is_none(), "{p:?}");
        }

        // PUT with the token: stored, and readable back with it.
        let put = send(
            &c,
            "PUT",
            format!("{base}{APPEARANCE}"),
            Req {
                origin: Some(origin),
                bearer: Some(&token),
                body: Some(json!({ "appearance": { "theme": "dark" } })),
                ..Req::default()
            },
        )
        .await;
        assert_eq!(put.status, 200, "{origin} PUT with the token: {put:?}");
        assert_eq!(put.allow_origin.as_deref(), Some(origin), "{put:?}");
        let got = send(
            &c,
            "GET",
            format!("{base}{APPEARANCE}"),
            Req {
                origin: Some(origin),
                bearer: Some(&token),
                ..Req::default()
            },
        )
        .await;
        assert_eq!(got.status, 200, "{got:?}");
        assert!(got.body.contains("dark"), "the PUT was stored: {got:?}");

        // The same PUT with no token is refused for its origin: the guard lets a
        // cookie-less Tauri request through without a token only to sign in.
        let anon = send(
            &c,
            "PUT",
            format!("{base}{APPEARANCE}"),
            Req {
                origin: Some(origin),
                body: Some(json!({ "appearance": { "theme": "dark" } })),
                ..Req::default()
            },
        )
        .await;
        assert_eq!(anon.status, 403, "{origin} PUT without a token: {anon:?}");
        assert!(anon.body.contains(ORIGIN_REFUSAL), "{anon:?}");

        // DELETE with the token.
        let del = send(
            &c,
            "DELETE",
            format!("{base}{APPEARANCE}"),
            Req {
                origin: Some(origin),
                bearer: Some(&token),
                ..Req::default()
            },
        )
        .await;
        assert_eq!(del.status, 200, "{origin} DELETE with the token: {del:?}");
    }
}

#[tokio::test]
async fn signing_out_with_the_token_ends_the_native_session() {
    let mock = spawn_mock().await;
    let base = spawn_server(&mock).await;
    let c = client();
    let token = native_login(&c, &base, &mock, TAURI).await;
    let me = |c: &reqwest::Client, token: &str| {
        let url = format!("{base}/api/me");
        let token = token.to_string();
        let c = c.clone();
        async move {
            send(
                &c,
                "GET",
                url,
                Req {
                    origin: Some(TAURI),
                    bearer: Some(&token),
                    ..Req::default()
                },
            )
            .await
        }
    };
    assert_eq!(me(&c, &token).await.status, 200, "signed in");

    let out = send(
        &c,
        "POST",
        format!("{base}/api/logout"),
        Req {
            origin: Some(TAURI),
            bearer: Some(&token),
            ..Req::default()
        },
    )
    .await;
    assert!(out.status == 204 || out.status == 200, "{out:?}");
    assert_eq!(
        me(&c, &token).await.status,
        401,
        "the token no longer works"
    );
}

#[tokio::test]
async fn a_tauri_origin_with_a_session_cookie_is_checked_as_before() {
    let mock = spawn_mock().await;
    let base = spawn_server(&mock).await;
    let c = client();

    // A browser sign-in (no Origin: same-site) to get a session cookie and token.
    let a = send(
        &c,
        "POST",
        format!("{base}/api/login"),
        Req {
            body: Some(login_body(&mock, false)),
            ..Req::default()
        },
    )
    .await;
    assert_eq!(a.status, 200, "{a:?}");
    let pick = |name: &str| {
        a.set_cookie
            .iter()
            .find(|c| c.starts_with(&format!("{name}=")))
            .map(|c| c.split(';').next().unwrap().to_string())
            .unwrap_or_else(|| panic!("login sets {name}"))
    };
    let session = pick("mw_session");
    let csrf_pair = pick("mw_csrf");
    let csrf = csrf_pair.split_once('=').unwrap().1.to_string();
    let cookies = format!("{session}; {csrf_pair}");
    let put = |origin: Option<&'static str>, with_token: bool| {
        let c = c.clone();
        let url = format!("{base}{APPEARANCE}");
        let cookies = cookies.clone();
        let csrf = csrf.clone();
        async move {
            send(
                &c,
                "PUT",
                url,
                Req {
                    origin,
                    cookie: Some(&cookies),
                    csrf: with_token.then_some(csrf.as_str()),
                    body: Some(json!({ "appearance": { "theme": "light" } })),
                    ..Req::default()
                },
            )
            .await
        }
    };

    // Control: same-site with the token is accepted.
    let ok = put(None, true).await;
    assert_eq!(
        ok.status, 200,
        "same-site PUT with cookie and token: {ok:?}"
    );

    // Same-site without the token: strict mode refuses it.
    let no_tok = put(None, false).await;
    assert_eq!(no_tok.status, 403, "{no_tok:?}");
    assert!(no_tok.body.contains(CSRF_REFUSAL), "{no_tok:?}");

    // Tauri origin with the cookie: refused for its origin, with or without the
    // token — a cookie request from another origin is not a native request.
    for origin in TAURI_ORIGINS {
        for with_token in [false, true] {
            let r = put(Some(origin), with_token).await;
            assert_eq!(r.status, 403, "{origin} token={with_token}: {r:?}");
            assert!(r.body.contains(ORIGIN_REFUSAL), "{r:?}");
        }
    }

    // A Tauri-origin sign-in that carries a session cookie is checked as before
    // too: refused for its origin.
    let r = send(
        &c,
        "POST",
        format!("{base}/api/login"),
        Req {
            origin: Some(TAURI),
            cookie: Some(&cookies),
            body: Some(login_body(&mock, true)),
            ..Req::default()
        },
    )
    .await;
    assert_eq!(r.status, 403, "{r:?}");
    assert!(r.body.contains(ORIGIN_REFUSAL), "{r:?}");
}

#[tokio::test]
async fn other_origins_are_refused_and_get_no_cors() {
    let mock = spawn_mock().await;
    let base = spawn_server(&mock).await;
    let c = client();

    // A native sign-in from a web page's origin is refused, without CORS headers.
    for origin in [EVIL, "http://tauri.localhost.evil.example", "tauri://evil"] {
        let r = send(
            &c,
            "POST",
            format!("{base}/api/login"),
            Req {
                origin: Some(origin),
                body: Some(login_body(&mock, true)),
                ..Req::default()
            },
        )
        .await;
        assert_eq!(r.status, 403, "{origin}: {r:?}");
        assert!(r.body.contains(ORIGIN_REFUSAL), "{r:?}");
        assert!(r.allow_origin.is_none(), "{origin} must get no CORS: {r:?}");

        let p = preflight(&c, format!("{base}{APPEARANCE}"), origin, "PUT").await;
        assert!(p.allow_origin.is_none(), "{origin} preflight: {p:?}");
        assert!(p.allow_methods.is_none(), "{origin} preflight: {p:?}");
    }

    // A cookie-less Tauri-origin write to a route other than sign-in, with no
    // token, is refused for its origin: only the sign-in routes are excused.
    let r = send(
        &c,
        "POST",
        format!("{base}/api/account/signatures"),
        Req {
            origin: Some(TAURI),
            body: Some(json!({ "name": "x", "html": "x" })),
            ..Req::default()
        },
    )
    .await;
    assert_eq!(r.status, 403, "{r:?}");
    assert!(r.body.contains(ORIGIN_REFUSAL), "{r:?}");

    // The second-factor step is a sign-in route: a cookie-less Tauri request
    // reaches its handler (which refuses the unknown pending token), not the guard.
    let r = send(
        &c,
        "POST",
        format!("{base}/api/login/2fa"),
        Req {
            origin: Some(TAURI),
            body: Some(json!({ "pendingToken": "nope", "method": "totp", "code": "000000" })),
            ..Req::default()
        },
    )
    .await;
    assert_ne!(r.status, 403, "the guard must not refuse it: {r:?}");
    assert!(!r.body.contains(ORIGIN_REFUSAL), "{r:?}");

    // Bearer-only from a web origin passes the guard — a bearer token is no
    // ambient authority a forged request could borrow — but gets no CORS headers,
    // so a page could not read the answer even if it held a token.
    let token = native_login(&c, &base, &mock, TAURI).await;
    let r = send(
        &c,
        "PUT",
        format!("{base}{APPEARANCE}"),
        Req {
            origin: Some(EVIL),
            bearer: Some(&token),
            body: Some(json!({ "appearance": { "theme": "dark" } })),
            ..Req::default()
        },
    )
    .await;
    assert_eq!(r.status, 200, "{r:?}");
    assert!(r.allow_origin.is_none(), "{r:?}");
}

#[tokio::test]
async fn a_plain_options_request_is_not_answered_by_the_cors_layer() {
    // CalDAV/CardDAV clients send OPTIONS without CORS headers; the native CORS
    // layer must leave those to the router.
    let mock = spawn_mock().await;
    let base = spawn_server(&mock).await;
    let c = client();
    let r = answer(
        c.request(reqwest::Method::OPTIONS, format!("{base}/healthz"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert!(r.allow_origin.is_none(), "{r:?}");
    assert!(r.allow_methods.is_none(), "{r:?}");
}

#[tokio::test]
async fn native_preferences_expose_etags_and_allow_cas_without_widening_origins() {
    let mock = spawn_mock().await;
    let base = spawn_server(&mock).await;
    let c = client();
    let token = native_login(&c, &base, &mock, TAURI).await;
    let endpoint = format!("{base}/api/account/preferences");
    let preflight = c
        .request(reqwest::Method::OPTIONS, &endpoint)
        .header("origin", TAURI)
        .header("access-control-request-method", "PUT")
        .header(
            "access-control-request-headers",
            "authorization, content-type, if-match",
        )
        .send()
        .await
        .unwrap();
    assert_eq!(preflight.headers()["access-control-allow-origin"], TAURI);
    assert!(
        preflight.headers()["access-control-allow-headers"]
            .to_str()
            .unwrap()
            .split(',')
            .any(|v| v.trim() == "if-match")
    );
    let loaded = c
        .get(&endpoint)
        .header("origin", TAURI)
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(loaded.status(), 200);
    assert_eq!(loaded.headers()["access-control-expose-headers"], "etag");
    let etag = loaded.headers()["etag"].clone();
    let saved = c
        .put(&endpoint)
        .header("origin", TAURI)
        .bearer_auth(&token)
        .header("if-match", etag)
        .json(&json!({"version":1,"preferences":{"workspaceLayout":"stack"}}))
        .send()
        .await
        .unwrap();
    assert_eq!(saved.status(), 200);
    for origin in [Some(EVIL), None] {
        let request = c.get(&endpoint).bearer_auth(&token);
        let request = if let Some(origin) = origin {
            request.header("origin", origin)
        } else {
            request
        };
        let response = request.send().await.unwrap();
        assert!(
            response
                .headers()
                .get("access-control-expose-headers")
                .is_none()
        );
        assert!(
            response
                .headers()
                .get("access-control-allow-origin")
                .is_none()
        );
    }
}
