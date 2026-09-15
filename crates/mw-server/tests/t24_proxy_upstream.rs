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

// ── structure: every relay leg is gated ──────────────────────────────────────
//
// `t22_every_fetch_takes_the_route` scans `mw-server/src` for `mw_egress`'s unrouted
// fetch entry points. It is not extended to cover this, deliberately: it enforces
// the egress *route*, which by design is never selected for a request-derived
// upstream, and the JMAP proxy never called those entry points, so pointing it at
// `mw-jmap` would find nothing and assert the wrong thing.
//
// What B2 needs is enforced here, over every crate under `crates/` and `plugins/`:
//
// 1. **Construction.** A `JmapClient` is built only inside `upstream_client` in
//    `mw-server/src/lib.rs`, which checks origin and address and pins the
//    connection. This cannot be made a compile-time property cheaply:
//    `JmapClient::with_http` is public, and making `mw-jmap` build the hardened
//    client itself would add an `mw-egress` dependency (a `Cargo.toml` and
//    `Cargo.lock` change this lane was told not to make). So it is a sweep, and the
//    sweep matches any `with_http(` call, so a `use … as` alias does not escape it.
// 2. **Session URLs.** A read of `.jmap_url`, `.api_url`, `.download_url`,
//    `.upload_url` or `.event_source_url` in production code must sit in a function
//    on the named list below, each with its reason. A new relay leg (a sixth) that
//    fetches a session-supplied URL with a plain `reqwest` client fails here and
//    has to be reviewed onto the list.
//
// Both rules run against embedded fixtures first, so "nothing found" cannot come
// from a scanner that stopped matching.

/// Where a `JmapClient` may be built.
const GATED_CONSTRUCTION: (&str, &str) = ("crates/mw-server/src/lib.rs", "upstream_client");

/// Every production function that reads a session URL field, and why it is safe.
const SESSION_URL_READERS: &[(&str, &str, &str)] = &[
    (
        "crates/mw-server/src/lib.rs",
        "login",
        "gated: upstream_session + session_urls_on_origin before the session is stored",
    ),
    (
        "crates/mw-server/src/lib.rs",
        "header_auth_login",
        "stores the URL only; every later leg gates it through upstream_client",
    ),
    (
        "crates/mw-server/src/lib.rs",
        "engine_login",
        "engine mode: passed to engine_mode (IMAP/POP3 dial, t23-e2-02), no JMAP relay",
    ),
    (
        "crates/mw-server/src/lib.rs",
        "rotate_session",
        "copies stored values into a new session row; no request",
    ),
    (
        "crates/mw-server/src/lib.rs",
        "jmap_session",
        "gated: upstream_session; writes local /jmap/* URLs over the upstream's",
    ),
    (
        "crates/mw-server/src/lib.rs",
        "jmap_api",
        "gated: upstream_client",
    ),
    (
        "crates/mw-server/src/lib.rs",
        "proxy_download",
        "gated: upstream_client",
    ),
    (
        "crates/mw-server/src/lib.rs",
        "proxy_upload",
        "gated: upstream_client",
    ),
    (
        "crates/mw-server/src/lib.rs",
        "session_urls_on_origin",
        "the same-origin check itself",
    ),
    (
        "crates/mw-server/src/rest.rs",
        "dispatch_jmap",
        "gated: upstream_client",
    ),
    (
        "crates/mw-server/src/twofa_routes.rs",
        "complete_login",
        "stores the URLs login already checked; no request",
    ),
    (
        "crates/mw-passwd/src/dovecot.rs",
        "build_request",
        "not a JMAP session: DovecotConfig.api_url is operator configuration",
    ),
];

#[derive(Debug, Clone, PartialEq, Eq)]
enum SiteKind {
    ClientConstruction,
    SessionUrlRead,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Site {
    path: String,
    line: usize,
    function: String,
    kind: SiteKind,
    text: String,
}

const SESSION_URL_FIELDS: [&str; 5] = [
    ".jmap_url",
    ".api_url",
    ".download_url",
    ".upload_url",
    ".event_source_url",
];

/// The identifier after the last `fn ` on a code line, if any.
fn fn_name(code: &str) -> Option<String> {
    let at = code.rfind("fn ")?;
    // `fn` must be a whole word (not the tail of e.g. `dyn_fn `).
    if at > 0
        && code[..at]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
    {
        return None;
    }
    let name: String = code[at + 3..]
        .trim_start()
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}

/// Classify one source file. Pure, so the fixtures below exercise exactly the code
/// the tree sweep runs.
fn scan_source(path: &str, source: &str) -> Vec<Site> {
    let mut sites = Vec::new();
    let mut function = String::new();
    let mut depth = 0i32;
    let mut in_tests = false;
    let mut region_depth = 0i32;
    let mut pending = false;
    for (i, raw) in source.lines().enumerate() {
        let code = raw.split("//").next().unwrap_or_default();
        let trimmed = code.trim();

        // Inline `#[cfg(test)] mod … { … }` only; an out-of-line `mod tests;` opens
        // no region (the scanner bug t22-e-sec caught in its own sweep).
        if trimmed == "#[cfg(test)]" {
            pending = true;
        } else if pending && !trimmed.is_empty() {
            if code.contains('{') {
                in_tests = true;
                region_depth = depth;
            }
            pending = false;
        }
        if let Some(name) = fn_name(code) {
            function = name;
        }

        if !in_tests {
            let definition = trimmed.contains("fn with_http(")
                || trimmed.contains("struct JmapClient")
                || trimmed.starts_with("impl ");
            // `-> JmapClient {` on a signature is a return type, not a literal.
            let literal = code.contains("JmapClient {") && fn_name(code).is_none();
            if !definition
                && (code.contains("with_http(") || code.contains("JmapClient::new(") || literal)
            {
                sites.push(Site {
                    path: path.to_string(),
                    line: i + 1,
                    function: function.clone(),
                    kind: SiteKind::ClientConstruction,
                    text: trimmed.to_string(),
                });
            }
            if SESSION_URL_FIELDS.iter().any(|f| {
                code.match_indices(f).any(|(at, _)| {
                    // `.api_url` but not `.api_url_template`.
                    !code[at + f.len()..]
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_alphanumeric() || c == '_')
                })
            }) {
                sites.push(Site {
                    path: path.to_string(),
                    line: i + 1,
                    function: function.clone(),
                    kind: SiteKind::SessionUrlRead,
                    text: trimmed.to_string(),
                });
            }
        }

        depth += code.matches('{').count() as i32;
        depth -= code.matches('}').count() as i32;
        if in_tests && depth <= region_depth {
            in_tests = false;
        }
    }
    sites
}

/// Sites that break a rule: a construction outside the gate, or a session-URL read
/// in a function not on the list.
fn violations(sites: &[Site]) -> Vec<Site> {
    sites
        .iter()
        .filter(|s| match s.kind {
            SiteKind::ClientConstruction => {
                (s.path.as_str(), s.function.as_str()) != GATED_CONSTRUCTION
            }
            SiteKind::SessionUrlRead => !SESSION_URL_READERS
                .iter()
                .any(|(p, f, _)| *p == s.path && *f == s.function),
        })
        .cloned()
        .collect()
}

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
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Run [`scan_source`] over every `crates/*/src` and `plugins/*/src` file.
fn scan_tree() -> Vec<Site> {
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
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        sites.extend(scan_source(&rel, &body));
    }
    sites
}

#[test]
fn the_relay_sweep_goes_red_on_an_ungated_leg() {
    // Each fixture is a way a sixth relay leg could be added. Every one must be
    // reported, or the tree assertion below could pass because the scanner is blind.
    let lib = "crates/mw-server/src/lib.rs";
    let ungated_client = r#"
async fn proxy_thumbnail(session: &Session) -> Response {
    let http = reqwest::Client::new();
    let client = JmapClient::with_http(http, &session.credentials.username, "p");
    todo!()
}
"#;
    let aliased = r#"
use mw_jmap::JmapClient as Upstream;
async fn proxy_preview(session: &Session) {
    let client = Upstream::with_http(reqwest::Client::new(), "u", "p");
}
"#;
    let raw_reqwest_on_session_url = r#"
async fn proxy_preview(session: &mw_store::Session) -> Response {
    let body = reqwest::get(&session.download_url).await;
    todo!()
}
"#;
    for (name, source, kind) in [
        (
            "ungated client",
            ungated_client,
            SiteKind::ClientConstruction,
        ),
        ("aliased client", aliased, SiteKind::ClientConstruction),
        (
            "plain reqwest on a session URL",
            raw_reqwest_on_session_url,
            SiteKind::SessionUrlRead,
        ),
    ] {
        let found = violations(&scan_source(lib, source));
        assert!(
            found.iter().any(|s| s.kind == kind),
            "the sweep missed the {name} fixture: {found:#?}"
        );
    }

    // And it does not cry wolf: the gate itself, definitions, a listed reader, and a
    // test module are all quiet.
    let quiet = r#"
pub struct JmapClient {
    http: reqwest::Client,
}
impl JmapClient {
    pub fn with_http(http: reqwest::Client, username: &str, password: &str) -> Self {
        Self { http }
    }
}
pub(crate) async fn upstream_client(
    security: &SecurityConfig,
) -> Result<JmapClient, UpstreamRefusal> {
    Ok(JmapClient::with_http(
        http,
        &credentials.username,
        &credentials.password,
    ))
}
async fn jmap_api(State(state): State<AppState>) -> Response {
    let client = upstream_client(&state.security, &session.jmap_url, &session.api_url).await;
}
async fn upstream_client_for(security: &SecurityConfig) -> JmapClient {
    todo!()
}
#[cfg(test)]
mod tests;
#[cfg(test)]
mod more_tests {
    fn t() {
        let c = JmapClient::with_http(reqwest::Client::new(), "u", "p");
        assert_eq!(s.api_url, "x");
    }
}
"#;
    let found = violations(&scan_source(lib, quiet));
    assert!(found.is_empty(), "false positives: {found:#?}");
    // `mod tests;` must not swallow what follows it: a leg after it is still seen.
    let after_out_of_line = format!("#[cfg(test)]\nmod tests;\n{ungated_client}");
    assert!(
        !violations(&scan_source(lib, &after_out_of_line)).is_empty(),
        "an out-of-line `mod tests;` hid the code after it"
    );
}

#[test]
fn every_relay_leg_in_the_tree_is_gated() {
    let sites = scan_tree();

    // Calibration against the real tree: the gate and at least the six gated legs
    // must be seen, or an empty violation list means nothing.
    assert!(
        sites.iter().any(|s| s.kind == SiteKind::ClientConstruction
            && (s.path.as_str(), s.function.as_str()) == GATED_CONSTRUCTION),
        "the sweep did not see the construction in `upstream_client`: {sites:#?}"
    );
    for leg in [
        "login",
        "jmap_session",
        "jmap_api",
        "proxy_download",
        "proxy_upload",
        "dispatch_jmap",
    ] {
        assert!(
            sites
                .iter()
                .any(|s| s.kind == SiteKind::SessionUrlRead && s.function == leg),
            "the sweep did not see the `{leg}` leg reading a session URL"
        );
    }

    let bad = violations(&sites);
    assert!(
        bad.is_empty(),
        "A JMAP relay path that does not go through `mw_server::upstream_client` talks to \
         an upstream URL that no origin check, address policy or DNS pin has seen (t24 B2).\n\n\
         * A `JmapClient` built outside `upstream_client`: build it there instead.\n\
         * A new function reading a session URL field: if it sends a request, route it \
         through `upstream_client`/`upstream_session`; then add it to SESSION_URL_READERS \
         with the reason it is safe.\n\nfound: {bad:#?}"
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
    let code: Vec<&str> = body
        .lines()
        .map(|l| l.split("//").next().unwrap_or_default())
        .collect();
    let builders: Vec<&&str> = code
        .iter()
        .filter(|c| c.contains("Client::builder(") || c.contains("Client::new("))
        .collect();
    assert!(
        builders.is_empty(),
        "mw-jmap must take its HTTP client from the caller, who owns the address policy \
         and the pin: {builders:#?}"
    );
    // `with_http` is the crate's only constructor; a second `Self { .. }` would be a
    // second way in.
    let self_literals = code
        .iter()
        .filter(|c| c.trim_start().starts_with("Self {"))
        .count();
    assert_eq!(
        self_literals, 1,
        "JmapClient must have exactly one constructor (`with_http`)"
    );
}
