//! **The proxy-mode upstream is checked before it is trusted, and it can only speak
//! for itself** (t24-e8, blocker B2 from the t23 production-readiness review).
//!
//! # The defect these tests pin
//! In proxy mode (the default), anonymous `POST /api/login` names the JMAP upstream
//! in its body. Before this change the server fetched that URL with a client that
//! applied no address policy and followed redirects, then stored the `apiUrl` the
//! upstream's own session document named, and later relayed `/jmap/api`,
//! `/jmap/download` and `/jmap/upload` to whatever URLs that document supplied —
//! with the login's Basic credential attached. An attacker who controls one JMAP
//! server could read back cloud metadata or an internal service through the
//! download and API legs.
//!
//! # What is asserted
//! * a login naming a loopback, RFC1918, link-local/metadata, CGNAT, ULA or
//!   IPv4-mapped private upstream is refused **before any connection is made** —
//!   proven with listeners that count accepted connections wherever a local
//!   listener can stand in for the target;
//! * an operator allowlist (`SecurityConfig::jmap_upstreams`) permits the upstream
//!   it names and refuses every other one;
//! * the session's `apiUrl`/`downloadUrl`/`uploadUrl` must share the upstream's
//!   origin, at login and on every relayed leg;
//! * redirects are re-checked per hop: a same-origin redirect on the session fetch
//!   is followed, anything else is not, and the relayed legs follow none;
//! * the download leg does not relay the upstream's `Content-Type` /
//!   `Content-Disposition` as received.
//!
//! # Why the working upstreams here are on an allowlist
//! Every local listener is on loopback, and the address floor correctly refuses
//! loopback. The tests that need a working upstream therefore name it in the
//! **operator allowlist** — the same production control an operator uses to permit
//! an internal JMAP server. There is no test-only switch.
//!
//! Run:
//!   cargo test -p mw-server --test t24_proxy_upstream -- --test-threads=1

use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::Router;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use mw_server::{AppConfig, SecurityConfig, build_app};

mod common;
use common::test_db;

const ACCOUNT: &str = "acct-b2";

/// A refusal by policy never dials, so it answers well inside this. The unfixed
/// code dials, and a dial to an unroutable address hangs far longer.
const REFUSAL_BUDGET: Duration = Duration::from_secs(5);

// ── harness ──────────────────────────────────────────────────────────────────

/// A stand-in for an internal service: counts every accepted TCP connection and
/// answers anything with a short "secret" body.
struct Internal {
    addr: SocketAddr,
    accepts: Arc<AtomicUsize>,
}

impl Internal {
    async fn spawn() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let accepts = Arc::new(AtomicUsize::new(0));
        let counter = accepts.clone();
        tokio::spawn(async move {
            while let Ok((mut sock, _)) = listener.accept().await {
                counter.fetch_add(1, Ordering::SeqCst);
                tokio::spawn(async move {
                    let mut buf = [0u8; 4096];
                    let _ = sock.read(&mut buf).await;
                    let _ = sock
                        .write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
                              Content-Length: 22\r\nConnection: close\r\n\r\n\
                              {\"secret\":\"internal\"}\n",
                        )
                        .await;
                });
            }
        });
        Self { addr, accepts }
    }

    fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn accepts(&self) -> usize {
        self.accepts.load(Ordering::SeqCst)
    }
}

/// A scriptable JMAP upstream. `session` is the JSON the session endpoint serves;
/// the string `{self}` in it is replaced by the upstream's own origin, so a test can
/// flip a URL to another origin after login.
#[derive(Clone)]
struct Upstream {
    origin: String,
    session: Arc<Mutex<Value>>,
    /// When set, the session endpoint answers with this redirect instead.
    redirect: Arc<Mutex<Option<String>>>,
    /// When set, the API endpoint answers with this redirect instead.
    api_redirect: Arc<Mutex<Option<String>>>,
    hits: Arc<AtomicUsize>,
}

fn same_origin_session() -> Value {
    json!({
        "capabilities": { "urn:ietf:params:jmap:core": {}, "urn:ietf:params:jmap:mail": {} },
        "accounts": { ACCOUNT: { "name": "b2", "isPersonal": true, "isReadOnly": false } },
        "primaryAccounts": { "urn:ietf:params:jmap:mail": ACCOUNT },
        "username": "b2@example.org",
        "apiUrl": "{self}/jmap",
        "downloadUrl": "{self}/download/{accountId}/{blobId}/{name}",
        "uploadUrl": "{self}/upload/{accountId}",
        "eventSourceUrl": "{self}/events",
        "state": "s0"
    })
}

impl Upstream {
    async fn spawn() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let up = Upstream {
            origin: format!("http://{addr}"),
            session: Arc::new(Mutex::new(same_origin_session())),
            redirect: Arc::new(Mutex::new(None)),
            api_redirect: Arc::new(Mutex::new(None)),
            hits: Arc::new(AtomicUsize::new(0)),
        };
        let s = up.clone();
        let router = Router::new()
            .route(
                "/.well-known/jmap",
                get({
                    let s = s.clone();
                    move || async move { s.serve_session() }
                }),
            )
            .route(
                "/jmap/session",
                get({
                    let s = s.clone();
                    move || async move {
                        *s.redirect.lock().unwrap() = None;
                        s.serve_session()
                    }
                }),
            )
            .route(
                "/jmap",
                post({
                    let s = s.clone();
                    move || async move {
                        s.hits.fetch_add(1, Ordering::SeqCst);
                        if let Some(loc) = s.api_redirect.lock().unwrap().clone() {
                            return (StatusCode::TEMPORARY_REDIRECT, [(header::LOCATION, loc)])
                                .into_response();
                        }
                        axum::Json(json!({ "methodResponses": [], "sessionState": "s0" }))
                            .into_response()
                    }
                }),
            )
            .route(
                "/download/{a}/{b}/{c}",
                get({
                    let s = s.clone();
                    move || async move {
                        s.hits.fetch_add(1, Ordering::SeqCst);
                        (
                            [
                                (header::CONTENT_TYPE, "text/html"),
                                (header::CONTENT_DISPOSITION, "inline"),
                            ],
                            "<script>alert(1)</script>",
                        )
                            .into_response()
                    }
                }),
            )
            .route(
                "/upload/{a}",
                post({
                    let s = s.clone();
                    move || async move {
                        s.hits.fetch_add(1, Ordering::SeqCst);
                        axum::Json(json!({ "accountId": ACCOUNT, "blobId": "B1", "type": "text/plain", "size": 3 }))
                    }
                }),
            );
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        up
    }

    fn serve_session(&self) -> axum::response::Response {
        self.hits.fetch_add(1, Ordering::SeqCst);
        if let Some(loc) = self.redirect.lock().unwrap().clone() {
            return (StatusCode::MOVED_PERMANENTLY, [(header::LOCATION, loc)]).into_response();
        }
        let body = self
            .session
            .lock()
            .unwrap()
            .to_string()
            .replace("{self}", &self.origin);
        ([(header::CONTENT_TYPE, "application/json")], body).into_response()
    }

    /// Point one session URL field somewhere else.
    fn set_field(&self, key: &str, value: &str) {
        self.session.lock().unwrap()[key] = json!(value);
    }
}

/// Proxy-mode security config whose allowlist names `origins`.
fn allowing(origins: &[&str]) -> SecurityConfig {
    SecurityConfig {
        jmap_upstreams: Some(origins.iter().map(|o| o.to_string()).collect()),
        ..SecurityConfig::default()
    }
}

async fn spawn_server(security: SecurityConfig) -> String {
    let base = test_db::unique_dir("mw-t24-b2");
    let web_dir = base.join("web");
    std::fs::create_dir_all(&web_dir).unwrap();
    std::fs::write(
        web_dir.join("index.html"),
        "<!doctype html><title>t</title>",
    )
    .unwrap();
    let config = AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web_dir),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security,
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
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

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap()
}

/// `POST /api/login` naming `jmap_url`; returns the status, bounded by
/// [`REFUSAL_BUDGET`] (a timeout is reported as `None`).
async fn login(c: &reqwest::Client, server: &str, jmap_url: &str) -> Option<u16> {
    let fut = c
        .post(format!("{server}/api/login"))
        .json(&json!({ "jmapUrl": jmap_url, "username": "b2", "password": "pw" }))
        .send();
    match tokio::time::timeout(REFUSAL_BUDGET, fut).await {
        Ok(Ok(resp)) => Some(resp.status().as_u16()),
        Ok(Err(e)) => panic!("login request failed at the transport: {e}"),
        Err(_) => None,
    }
}

/// The CSRF token the login cookie jar now carries, for state-changing legs.
async fn csrf(c: &reqwest::Client, server: &str) -> String {
    let me: Value = c
        .get(format!("{server}/api/me"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap_or(Value::Null);
    me["csrfToken"].as_str().unwrap_or_default().to_string()
}

// ── the address floor (no allowlist configured) ──────────────────────────────

#[tokio::test]
async fn login_refuses_a_loopback_upstream_without_connecting() {
    let internal = Internal::spawn().await;
    let server = spawn_server(SecurityConfig::default()).await;
    let status = login(&client(), &server, &internal.url()).await;
    assert_eq!(status, Some(401), "a loopback upstream must be refused");
    assert_eq!(
        internal.accepts(),
        0,
        "the refusal must happen before any connection to the named upstream"
    );
}

#[tokio::test]
async fn login_refuses_a_hostname_that_resolves_to_loopback_without_connecting() {
    let internal = Internal::spawn().await;
    let server = spawn_server(SecurityConfig::default()).await;
    let url = format!("http://localhost:{}", internal.addr.port());
    let status = login(&client(), &server, &url).await;
    assert_eq!(
        status,
        Some(401),
        "`localhost` resolves to loopback and must be refused"
    );
    assert_eq!(
        internal.accepts(),
        0,
        "no connection may be made to the resolved address"
    );
}

#[tokio::test]
async fn login_refuses_an_ipv4_mapped_loopback_upstream_without_connecting() {
    let internal = Internal::spawn().await;
    let server = spawn_server(SecurityConfig::default()).await;
    let url = format!("http://[::ffff:127.0.0.1]:{}", internal.addr.port());
    let status = login(&client(), &server, &url).await;
    assert_eq!(
        status,
        Some(401),
        "an IPv4-mapped loopback upstream must be refused"
    );
    assert_eq!(internal.accepts(), 0, "no connection may be made");
}

#[tokio::test]
async fn login_refuses_metadata_private_cgnat_and_ula_upstreams_promptly() {
    // No local listener can stand in for these addresses, so the evidence that
    // nothing was dialled is the answer arriving immediately: the unfixed code
    // attempts the TCP connect, which for an unroutable address outlasts the
    // budget. The listener-backed tests above carry the direct proof.
    let server = spawn_server(SecurityConfig::default()).await;
    let c = client();
    for url in [
        "http://169.254.169.254",
        "http://169.254.169.254/latest/meta-data/",
        "http://10.255.255.1:8080",
        "http://172.16.0.1",
        "http://192.168.255.254:81",
        "http://100.64.0.1",
        "http://[::ffff:10.255.255.1]:8080",
        "http://[::ffff:169.254.169.254]",
        "http://[fd00::1]",
        "http://[fe80::1]",
    ] {
        let started = Instant::now();
        let status = login(&c, &server, url).await;
        assert_eq!(
            status,
            Some(401),
            "{url} must be refused (None = the server was still dialling after {REFUSAL_BUDGET:?})"
        );
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "{url} took {:?}: a refusal by policy does not wait on a connect",
            started.elapsed()
        );
    }
}

// ── the operator allowlist ───────────────────────────────────────────────────

#[tokio::test]
async fn an_allowlisted_upstream_works_end_to_end() {
    // The allowlist is how an operator permits an internal upstream, and it is the
    // only way a loopback test upstream is reachable at all.
    let up = Upstream::spawn().await;
    let server = spawn_server(allowing(&[&up.origin])).await;
    let c = client();
    assert_eq!(login(&c, &server, &up.origin).await, Some(200));

    let token = csrf(&c, &server).await;
    let api = c
        .post(format!("{server}/jmap/api"))
        .header("x-csrf-token", &token)
        .json(&json!({ "using": [], "methodCalls": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        api.status(),
        200,
        "the API leg relays to a same-origin apiUrl"
    );

    let session: Value = c
        .get(format!("{server}/jmap/session"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(session["apiUrl"], json!("/jmap/api"));

    let dl = c
        .get(format!(
            "{server}/jmap/download/{ACCOUNT}/blob1/report.html"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(
        dl.status(),
        200,
        "the download leg relays a same-origin downloadUrl"
    );
}

#[tokio::test]
async fn an_allowlist_refuses_an_upstream_it_does_not_name() {
    let internal = Internal::spawn().await;
    let server = spawn_server(allowing(&["https://jmap.example.org"])).await;
    assert_eq!(login(&client(), &server, &internal.url()).await, Some(401));
    assert_eq!(
        internal.accepts(),
        0,
        "an off-list upstream is never contacted"
    );
}

#[tokio::test]
async fn an_allowlist_entry_is_an_origin_not_a_host() {
    // Same host, different port: a different origin, so not on the list.
    let internal = Internal::spawn().await;
    let other_port = format!("http://127.0.0.1:{}", internal.addr.port().wrapping_add(1));
    let server = spawn_server(allowing(&[&other_port])).await;
    assert_eq!(login(&client(), &server, &internal.url()).await, Some(401));
    assert_eq!(internal.accepts(), 0);
}

// ── same-origin binding of the session's URLs ────────────────────────────────

#[tokio::test]
async fn login_refuses_a_session_whose_api_url_names_another_origin() {
    let up = Upstream::spawn().await;
    let internal = Internal::spawn().await;
    up.set_field("apiUrl", &format!("{}/v1/secrets", internal.url()));
    let server = spawn_server(allowing(&[&up.origin])).await;
    let c = client();
    let status = login(&c, &server, &up.origin).await;

    // Whatever the login said, drive the API leg: on the unfixed code this is the
    // read-back.
    let token = csrf(&c, &server).await;
    let _ = c
        .post(format!("{server}/jmap/api"))
        .header("x-csrf-token", &token)
        .json(&json!({ "using": [], "methodCalls": [] }))
        .send()
        .await
        .unwrap();

    assert_eq!(
        status,
        Some(401),
        "a cross-origin apiUrl must refuse the login"
    );
    assert_eq!(
        internal.accepts(),
        0,
        "the foreign apiUrl must never be contacted"
    );
}

#[tokio::test]
async fn login_refuses_a_session_whose_download_or_upload_url_names_another_origin() {
    for key in ["downloadUrl", "uploadUrl"] {
        let up = Upstream::spawn().await;
        let internal = Internal::spawn().await;
        up.set_field(key, &format!("{}/{{accountId}}", internal.url()));
        let server = spawn_server(allowing(&[&up.origin])).await;
        assert_eq!(
            login(&client(), &server, &up.origin).await,
            Some(401),
            "a cross-origin {key} must refuse the login"
        );
        assert_eq!(internal.accepts(), 0);
    }
}

#[tokio::test]
async fn the_download_leg_refuses_a_download_url_that_changed_origin_after_login() {
    // The relayed legs re-read the upstream session, so a check at login alone is
    // bypassed by an upstream that changes its answer afterwards.
    let up = Upstream::spawn().await;
    let internal = Internal::spawn().await;
    let server = spawn_server(allowing(&[&up.origin])).await;
    let c = client();
    assert_eq!(login(&c, &server, &up.origin).await, Some(200));

    up.set_field(
        "downloadUrl",
        &format!("{}/latest/meta-data/{{blobId}}", internal.url()),
    );
    let dl = c
        .get(format!("{server}/jmap/download/{ACCOUNT}/iam/x"))
        .send()
        .await
        .unwrap();
    let status = dl.status().as_u16();
    let body = dl.text().await.unwrap_or_default();
    assert_eq!(
        internal.accepts(),
        0,
        "the foreign downloadUrl must never be contacted"
    );
    assert_eq!(status, 502, "the download leg refuses rather than relays");
    assert!(
        !body.contains("internal"),
        "no internal bytes reach the client: {body}"
    );
}

#[tokio::test]
async fn the_upload_leg_refuses_an_upload_url_that_changed_origin_after_login() {
    let up = Upstream::spawn().await;
    let internal = Internal::spawn().await;
    let server = spawn_server(allowing(&[&up.origin])).await;
    let c = client();
    assert_eq!(login(&c, &server, &up.origin).await, Some(200));
    let token = csrf(&c, &server).await;

    up.set_field("uploadUrl", &format!("{}/{{accountId}}", internal.url()));
    let resp = c
        .post(format!("{server}/jmap/upload/{ACCOUNT}"))
        .header("x-csrf-token", &token)
        .header(header::CONTENT_TYPE.as_str(), "text/plain")
        .body("abc")
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    assert_eq!(
        internal.accepts(),
        0,
        "the foreign uploadUrl must never be contacted"
    );
    assert_eq!(status, 502, "the upload leg refuses rather than relays");
}

// ── redirects ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_session_redirect_to_another_address_is_not_followed() {
    let up = Upstream::spawn().await;
    let internal = Internal::spawn().await;
    *up.redirect.lock().unwrap() = Some(format!("{}/.well-known/jmap", internal.url()));
    let server = spawn_server(allowing(&[&up.origin])).await;
    assert_eq!(login(&client(), &server, &up.origin).await, Some(401));
    assert_eq!(
        internal.accepts(),
        0,
        "the redirect target must never be contacted"
    );

    // And the metadata address, which no listener can stand in for.
    *up.redirect.lock().unwrap() = Some("http://169.254.169.254/latest/meta-data/".into());
    let started = Instant::now();
    assert_eq!(login(&client(), &server, &up.origin).await, Some(401));
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[tokio::test]
async fn a_same_origin_session_redirect_is_followed() {
    // RFC 8620 §2.2 lets `/.well-known/jmap` redirect, and real providers do
    // (to `/jmap/session` on the same origin). That must keep working.
    let up = Upstream::spawn().await;
    *up.redirect.lock().unwrap() = Some(format!("{}/jmap/session", up.origin));
    let server = spawn_server(allowing(&[&up.origin])).await;
    // The `/jmap/session` route clears the redirect, so it serves the document.
    assert_eq!(login(&client(), &server, &up.origin).await, Some(200));
}

#[tokio::test]
async fn the_api_leg_does_not_follow_a_redirect() {
    let up = Upstream::spawn().await;
    let internal = Internal::spawn().await;
    let server = spawn_server(allowing(&[&up.origin])).await;
    let c = client();
    assert_eq!(login(&c, &server, &up.origin).await, Some(200));
    let token = csrf(&c, &server).await;

    *up.api_redirect.lock().unwrap() = Some(format!("{}/v1/secrets", internal.url()));
    let resp = c
        .post(format!("{server}/jmap/api"))
        .header("x-csrf-token", &token)
        .json(&json!({ "using": [], "methodCalls": [] }))
        .send()
        .await
        .unwrap();
    let body = resp.text().await.unwrap_or_default();
    assert_eq!(
        internal.accepts(),
        0,
        "a redirect on the relayed API leg is not followed"
    );
    assert!(
        !body.contains("internal"),
        "no internal bytes reach the client: {body}"
    );
}

// ── response headers ─────────────────────────────────────────────────────────

#[tokio::test]
async fn the_download_leg_does_not_relay_upstream_content_headers_as_received() {
    let up = Upstream::spawn().await;
    let server = spawn_server(allowing(&[&up.origin])).await;
    let c = client();
    assert_eq!(login(&c, &server, &up.origin).await, Some(200));
    let dl = c
        .get(format!(
            "{server}/jmap/download/{ACCOUNT}/blob1/report.html"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(dl.status(), 200);
    let disposition = dl
        .headers()
        .get(header::CONTENT_DISPOSITION.as_str())
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        disposition.starts_with("attachment"),
        "the upstream said `inline`; the proxy must force an attachment, got {disposition:?}"
    );
    assert_eq!(
        dl.headers()
            .get("x-content-type-options")
            .and_then(|v| v.to_str().ok()),
        Some("nosniff")
    );
    let content_type = dl
        .headers()
        .get(header::CONTENT_TYPE.as_str())
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        !content_type.starts_with("text/html"),
        "an upstream `text/html` must not be relayed as-is, got {content_type:?}"
    );
}

// ── structure: no JMAP client is built outside the gate ──────────────────────
//
// `t22_every_fetch_takes_the_route` scans `mw-server/src` for `mw_egress`'s unrouted
// fetch entry points. The JMAP proxy never called those, so that scan could not
// see it, and it should not be extended to: that test enforces the egress
// *route*, which by design is never selected for a request-derived upstream.
// What has to hold for B2 is different — every JMAP client is built by
// `upstream_client`, which checks origin and address and pins the connection —
// so that is what this sweep asserts, over every crate that could build one.

fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/mw-server sits two levels below the workspace root")
        .to_path_buf()
}

fn rust_sources(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|n| n == "target" || n == "node_modules")
            {
                continue;
            }
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Every non-comment line under `crates/*/src` and `plugins/*/src` that builds a
/// `JmapClient`, as `(path, line index, enclosing fn line)`.
fn jmap_client_constructions() -> Vec<(String, usize, String)> {
    let root = workspace_root();
    let mut files = Vec::new();
    for top in ["crates", "plugins"] {
        let Ok(entries) = std::fs::read_dir(root.join(top)) else {
            continue;
        };
        for krate in entries.flatten() {
            rust_sources(&krate.path().join("src"), &mut files);
        }
    }
    let mut sites = Vec::new();
    for path in files {
        let Ok(body) = std::fs::read_to_string(&path) else {
            continue;
        };
        let lines: Vec<&str> = body.lines().collect();
        for (i, raw) in lines.iter().enumerate() {
            let code = raw.split("//").next().unwrap_or_default();
            // `struct JmapClient {` and `impl JmapClient {` are definitions, not
            // constructions.
            let definition = code.contains("struct JmapClient") || code.contains("impl ");
            if !definition
                && (code.contains("JmapClient::with_http(")
                    || code.contains("JmapClient::new(")
                    || code.contains("JmapClient {"))
            {
                let enclosing = lines[..=i]
                    .iter()
                    .rev()
                    .find(|l| l.contains("fn "))
                    .map(|l| l.trim().to_string())
                    .unwrap_or_default();
                let rel = path
                    .strip_prefix(&root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                sites.push((rel, i + 1, enclosing));
            }
        }
    }
    sites
}

#[test]
fn every_jmap_client_is_built_by_the_upstream_gate() {
    let sites = jmap_client_constructions();
    // Calibration: the scanner must see the one legitimate site. A scanner that read
    // nothing would report an empty "outside" list, which must not pass.
    // (`with_http`'s own `Self { .. }` is not matched: it does not name the type, and
    // `mw_jmap_does_not_build_its_own_http_client` covers that crate.)
    assert!(
        sites
            .iter()
            .any(|(path, _, enclosing)| path == "crates/mw-server/src/lib.rs"
                && enclosing.contains("fn upstream_client(")),
        "the scanner did not find the construction in `upstream_client` — it is not \
         reading the tree, so an empty result below would mean nothing: {sites:#?}"
    );
    let outside: Vec<_> = sites
        .iter()
        .filter(|(path, _, enclosing)| {
            !(path == "crates/mw-server/src/lib.rs" && enclosing.contains("fn upstream_client("))
        })
        .collect();
    assert!(
        outside.is_empty(),
        "A JmapClient built anywhere but `mw_server::upstream_client` talks to an upstream \
         URL that no origin check, address policy or DNS pin has seen (t24 B2). Use \
         `upstream_client` / `upstream_session`.\n\nfound: {outside:#?}"
    );
}

#[test]
fn mw_jmap_does_not_build_its_own_http_client() {
    let path = workspace_root().join("crates/mw-jmap/src/client.rs");
    let body = std::fs::read_to_string(&path).expect("read mw-jmap client.rs");
    assert!(
        body.contains("pub fn with_http("),
        "calibration: the constructor this test is about must exist"
    );
    let offending: Vec<&str> = body
        .lines()
        .map(|l| l.split("//").next().unwrap_or_default())
        .filter(|code| code.contains("Client::builder(") || code.contains("Client::new("))
        .collect();
    assert!(
        offending.is_empty(),
        "mw-jmap must take its HTTP client from the caller, who owns the address policy \
         and the pin: {offending:#?}"
    );
}
