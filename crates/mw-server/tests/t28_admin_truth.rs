//! t28-e8 — the admin panel says what the server does, over the real router.
//!
//! Each test logs in through the real `POST /admin/login`, so the cookie, the
//! session row and every gate are the production ones. Every negative assertion is
//! preceded by a control showing the same request succeeding, so a test cannot
//! pass because the route was simply unreachable.
//!
//! * **Admin sessions expire (0030).** Time is moved by rewriting the row's
//!   deadlines with raw SQL — the stored deadline is the only clock input
//!   `Store::get_admin_session` has besides the wall clock, so a deadline in the
//!   past is exactly what a session left idle looks like.
//! * **Egress.** `POST /admin/egress/proxies/{id}/test` against a stand-in CONNECT
//!   proxy; the origin is a public literal address that nothing dials, because the
//!   stand-in answers the tunnelled request itself (the `t22_egress_fetch_no_leak`
//!   arrangement). Activation is then shown to change what the image proxy does.
//! * **Shapes.** Security policy, domains and integrations return what
//!   `crates/mw-server/src/admin.rs` documents and refuse what it removed.
//!
//! SQLite only: the expiry and `0030` are also driven on live Postgres by
//! `mw-store`'s `v6::tests::admin_session_expiry_on_postgres`.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{ServerKey, Store};

mod common;
use common::test_db;

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";
const ADMIN_USER: &str = "root";
const ADMIN_PASS: &str = "hunter2";

/// Public unicast, so the address policy admits it. Nothing dials it: the stand-in
/// proxy answers the tunnelled request itself.
const ORIGIN: &str = "1.2.3.4";

// ── server ─────────────────────────────────────────────────────────────────────

/// Mock JMAP upstreams started by this binary; proxy mode refuses a loopback
/// upstream the operator allowlist does not name.
static MOCK_UPSTREAMS: Mutex<Vec<String>> = Mutex::new(Vec::new());

async fn spawn_mock() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, mw_mock_jmap::router()).await;
    });
    let origin = format!("http://{addr}");
    MOCK_UPSTREAMS.lock().unwrap().push(origin.clone());
    origin
}

fn new_db(tag: &str) -> String {
    test_db::unique_dir(tag)
        .join("mw.db")
        .to_string_lossy()
        .into_owned()
}

/// Boot the real app on `db` with admin credentials and return its base URL.
async fn spawn_server(db: &str) -> String {
    let web = PathBuf::from(db)
        .parent()
        .unwrap()
        .join(format!("web-{}", test_db::unique_tag()));
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: db.to_string(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(MOCK_UPSTREAMS.lock().unwrap().clone()),
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

fn client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

/// What `POST /admin/login` set: the `name=value` pair to send back, and the whole
/// `Set-Cookie` value.
struct Login {
    cookie: String,
    set_cookie: String,
}

impl Login {
    /// The raw token, as the browser would hold it.
    fn token(&self) -> &str {
        self.cookie.split_once('=').unwrap().1
    }
}

async fn admin_login(c: &reqwest::Client, base: &str) -> Login {
    let resp = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": ADMIN_USER, "password": ADMIN_PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "admin login");
    let set_cookie = resp
        .headers()
        .get(reqwest::header::SET_COOKIE)
        .expect("login sets a cookie")
        .to_str()
        .unwrap()
        .to_string();
    Login {
        cookie: set_cookie.split(';').next().unwrap().to_string(),
        set_cookie,
    }
}

async fn get(c: &reqwest::Client, url: String, cookie: &str) -> reqwest::Response {
    c.get(url)
        .header(reqwest::header::COOKIE, cookie)
        .send()
        .await
        .unwrap()
}

async fn post(
    c: &reqwest::Client,
    url: String,
    cookie: &str,
    body: Option<Value>,
) -> reqwest::Response {
    let mut req = c.post(url).header(reqwest::header::COOKIE, cookie);
    if let Some(b) = body {
        req = req.json(&b);
    }
    req.send().await.unwrap()
}

/// The `admin_sessions` key for a token: lowercase-hex SHA-256, what
/// `push_relay::hash_token` (crate-private) produces. If this diverged from the
/// server's, the raw updates below would match no row and the "still works"
/// controls after them would fail.
fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(token.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

async fn raw(db: &str) -> sqlx::SqlitePool {
    sqlx::SqlitePool::connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(db))
        .await
        .expect("open the server's database")
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// `(expires_at, absolute_expires_at)` of one session row.
async fn deadlines(pool: &sqlx::SqlitePool, hash: &str) -> Option<(i64, i64)> {
    sqlx::query_as::<_, (i64, i64)>(
        "SELECT expires_at, absolute_expires_at FROM admin_sessions WHERE token_hash = ?1",
    )
    .bind(hash)
    .fetch_optional(pool)
    .await
    .unwrap()
}

/// Rewrite one column of one session row; asserts exactly one row changed, so a
/// wrong hash cannot make a later assertion vacuous.
async fn set_deadline(pool: &sqlx::SqlitePool, hash: &str, column: &str, value: i64) {
    let n = sqlx::query(&format!(
        "UPDATE admin_sessions SET {column} = ?1 WHERE token_hash = ?2"
    ))
    .bind(value)
    .bind(hash)
    .execute(pool)
    .await
    .unwrap()
    .rows_affected();
    assert_eq!(n, 1, "the session row for this token exists");
}

// ── admin sessions expire ──────────────────────────────────────────────────────

#[tokio::test]
async fn the_admin_cookie_carries_the_absolute_lifetime() {
    let db = new_db("mw-t28e8-cookie");
    let base = spawn_server(&db).await;
    let login = admin_login(&client(), &base).await;
    assert!(
        login.set_cookie.contains("Max-Age=43200"),
        "12 hours, the session's absolute cap: {}",
        login.set_cookie
    );
    assert_eq!(Store::ADMIN_SESSION_MAX_SECS, 43_200);
    for attr in ["HttpOnly", "SameSite=Strict", "Path=/"] {
        assert!(
            login.set_cookie.contains(attr),
            "{attr}: {}",
            login.set_cookie
        );
    }
}

#[tokio::test]
async fn an_idle_admin_session_is_refused_on_every_admin_gate() {
    let db = new_db("mw-t28e8-idle");
    let base = spawn_server(&db).await;
    let c = client();
    let login = admin_login(&c, &base).await;
    let hash = hash_token(login.token());
    let pool = raw(&db).await;

    // Three gates implemented in three different files: `admin.rs`, the egress
    // admin (`v7_mount::require_admin`) and the plugin registry (`plugins.rs`).
    let gates = ["/admin/session", "/admin/egress/proxies", "/admin/plugins"];

    // Precondition: this very token is accepted by each of them.
    for path in gates {
        let resp = get(&c, format!("{base}{path}"), &login.cookie).await;
        assert_eq!(resp.status(), 200, "{path} accepts a fresh session");
    }
    let (idle, cap) = deadlines(&pool, &hash).await.expect("session row");
    let now = unix_now();
    assert!(
        (idle - now - Store::ADMIN_SESSION_IDLE_SECS).abs() <= 10,
        "{idle}"
    );
    assert!(
        (cap - now - Store::ADMIN_SESSION_MAX_SECS).abs() <= 10,
        "{cap}"
    );

    // The session sits unused past its idle deadline.
    set_deadline(&pool, &hash, "expires_at", now - 1).await;

    let resp = get(&c, format!("{base}/admin/session"), &login.cookie).await;
    assert_eq!(resp.status(), 401, "an idle session is refused");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "admin authentication required");
    assert!(
        deadlines(&pool, &hash).await.is_none(),
        "the expired row is deleted"
    );
    for path in gates {
        let resp = get(&c, format!("{base}{path}"), &login.cookie).await;
        assert_eq!(resp.status(), 401, "{path} refuses the expired session");
    }

    // The refusal is about that session, not the route: a new login works.
    let again = admin_login(&c, &base).await;
    assert_ne!(again.token(), login.token());
    let resp = get(&c, format!("{base}/admin/session"), &again.cookie).await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn using_a_session_moves_its_idle_deadline_but_never_past_the_cap() {
    let db = new_db("mw-t28e8-refresh");
    let base = spawn_server(&db).await;
    let c = client();
    let login = admin_login(&c, &base).await;
    let hash = hash_token(login.token());
    let pool = raw(&db).await;
    let (_, cap) = deadlines(&pool, &hash).await.unwrap();

    // Two minutes of idle time left.
    let now = unix_now();
    set_deadline(&pool, &hash, "expires_at", now + 120).await;
    let resp = get(&c, format!("{base}/admin/session"), &login.cookie).await;
    assert_eq!(resp.status(), 200, "still inside the idle window");
    let (idle, cap_after) = deadlines(&pool, &hash).await.unwrap();
    assert!(
        (idle - unix_now() - Store::ADMIN_SESSION_IDLE_SECS).abs() <= 10,
        "the request moved the idle deadline to now + 30 min, got {idle}"
    );
    assert_eq!(cap_after, cap, "the absolute cap does not move");

    // Near the cap, the idle deadline stops at the cap.
    let now = unix_now();
    set_deadline(&pool, &hash, "absolute_expires_at", now + 300).await;
    set_deadline(&pool, &hash, "expires_at", now + 60).await;
    let resp = get(&c, format!("{base}/admin/session"), &login.cookie).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(deadlines(&pool, &hash).await.unwrap().0, now + 300);

    // Past the cap the session ends although its idle deadline is in the future.
    set_deadline(&pool, &hash, "absolute_expires_at", unix_now() - 1).await;
    let resp = get(&c, format!("{base}/admin/session"), &login.cookie).await;
    assert_eq!(resp.status(), 401, "a session in use still ends at the cap");
}

#[tokio::test]
async fn a_session_row_from_before_0030_is_refused() {
    let db = new_db("mw-t28e8-legacy");
    let base = spawn_server(&db).await;
    let c = client();
    let login = admin_login(&c, &base).await;
    let hash = hash_token(login.token());
    let pool = raw(&db).await;
    let resp = get(&c, format!("{base}/admin/session"), &login.cookie).await;
    assert_eq!(resp.status(), 200, "control");

    // What 0030 leaves in a row that existed before it: both deadlines 0.
    set_deadline(&pool, &hash, "expires_at", 0).await;
    set_deadline(&pool, &hash, "absolute_expires_at", 0).await;
    let resp = get(&c, format!("{base}/admin/session"), &login.cookie).await;
    assert_eq!(resp.status(), 401);
}

// ── egress: test route, activation ─────────────────────────────────────────────

async fn read_head(stream: &mut TcpStream) -> String {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while stream.read(&mut byte).await.unwrap_or(0) == 1 {
        head.push(byte[0]);
        if head.ends_with(b"\r\n\r\n") || head.len() > 8192 {
            break;
        }
    }
    String::from_utf8_lossy(&head).into_owned()
}

/// A 1×1 opaque PNG, so the image proxy has something real to re-encode.
const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
    0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB0, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
    0x44, 0xAE, 0x42, 0x60, 0x82,
];

/// A stand-in `CONNECT` proxy. It records every request head it receives. When
/// `require_auth` is set it answers `407` to a `CONNECT` without
/// `Proxy-Authorization`; otherwise it accepts the tunnel and serves [`TINY_PNG`]
/// itself in place of the origin.
async fn stand_in_proxy(require_auth: bool) -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&log);
    tokio::spawn(async move {
        while let Ok((mut client, _)) = listener.accept().await {
            let sink = Arc::clone(&sink);
            tokio::spawn(async move {
                let head = read_head(&mut client).await;
                if head.is_empty() {
                    return; // the route test's bare TCP connect
                }
                sink.lock().unwrap().push(head.clone());
                if require_auth && !head.contains("Proxy-Authorization:") {
                    let _ = client
                        .write_all(
                            b"HTTP/1.1 407 Proxy Authentication Required\r\n\
                              Proxy-Authenticate: Basic realm=\"mailwoman-test\"\r\n\
                              Content-Length: 0\r\n\r\n",
                        )
                        .await;
                    return;
                }
                let _ = client
                    .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                    .await;
                let _ = read_head(&mut client).await;
                let mut resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\n\r\n",
                    TINY_PNG.len()
                )
                .into_bytes();
                resp.extend_from_slice(TINY_PNG);
                let _ = client.write_all(&resp).await;
                let _ = client.flush().await;
            });
        }
    });
    (addr, log)
}

fn connects(log: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
    log.lock()
        .unwrap()
        .iter()
        .filter(|h| h.starts_with("CONNECT "))
        .cloned()
        .collect()
}

/// Point the route test at a plaintext URL on the literal origin, which the
/// stand-in proxy can answer. Every test in this binary sets the same value, and
/// the binary runs single-threaded, so the write does not race a read.
fn use_test_probe_url() -> String {
    let url = format!("http://{ORIGIN}/probe");
    // SAFETY: see above — one value, set before any server in this test is asked
    // to read it.
    unsafe { std::env::set_var("MW_EGRESS_PROBE_URL", &url) };
    url
}

async fn put_route(
    c: &reqwest::Client,
    base: &str,
    cookie: &str,
    id: &str,
    host: &str,
    port: u16,
    allow_plaintext: bool,
) {
    let resp = post(
        c,
        format!("{base}/admin/egress/proxies"),
        cookie,
        Some(json!({
            "id": id,
            "scheme": "http",
            "host": host,
            "port": port,
            "allowPlaintext": allow_plaintext,
        })),
    )
    .await;
    assert_eq!(resp.status(), 200, "save route {id}");
}

async fn activate(c: &reqwest::Client, base: &str, cookie: &str, id: &str) {
    let resp = post(
        c,
        format!("{base}/admin/egress/proxies/{id}/activate"),
        cookie,
        None,
    )
    .await;
    assert_eq!(resp.status(), 200, "activate {id}");
}

/// Run the route test and return `(status, body)`.
async fn run_test(c: &reqwest::Client, base: &str, cookie: &str, id: &str) -> (u16, Value) {
    let resp = post(
        c,
        format!("{base}/admin/egress/proxies/{id}/test"),
        cookie,
        None,
    )
    .await;
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

#[tokio::test]
async fn the_route_test_is_admin_gated_and_answers_404_and_409_before_probing() {
    let probe = use_test_probe_url();
    let db = new_db("mw-t28e8-test-gate");
    let base = spawn_server(&db).await;
    let c = client();
    let login = admin_login(&c, &base).await;
    let (proxy, log) = stand_in_proxy(false).await;
    put_route(
        &c,
        &base,
        &login.cookie,
        "staged",
        "127.0.0.1",
        proxy.port(),
        true,
    )
    .await;

    // Not an admin.
    let (status, _) = run_test(&c, &base, "mw_admin_session=not-a-session", "staged").await;
    assert_eq!(status, 401);

    // No such route.
    let (status, body) = run_test(&c, &base, &login.cookie, "no-such-route").await;
    assert_eq!(status, 404, "{body}");
    assert_eq!(body["error"], "no such egress route");

    // A route that exists but is not the active one is not probed.
    let (status, body) = run_test(&c, &base, &login.cookie, "staged").await;
    assert_eq!(status, 409, "{body}");
    assert_eq!(body["error"], "only the active egress route can be tested");
    assert!(
        log.lock().unwrap().is_empty(),
        "none of the refusals sent anything to the proxy: {:?}",
        log.lock().unwrap()
    );

    // Control: the same route, once active, is probed and answers 200.
    activate(&c, &base, &login.cookie, "staged").await;
    let (status, body) = run_test(&c, &base, &login.cookie, "staged").await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["probeUrl"], probe);
}

#[tokio::test]
async fn the_route_test_reports_a_working_route_as_connected_through_the_proxy() {
    let probe = use_test_probe_url();
    let db = new_db("mw-t28e8-test-ok");
    let base = spawn_server(&db).await;
    let c = client();
    let login = admin_login(&c, &base).await;
    let (proxy, log) = stand_in_proxy(false).await;
    put_route(
        &c,
        &base,
        &login.cookie,
        "corp",
        "127.0.0.1",
        proxy.port(),
        true,
    )
    .await;
    activate(&c, &base, &login.cookie, "corp").await;
    assert!(
        connects(&log).is_empty(),
        "nothing tunnelled before the test"
    );

    let (status, body) = run_test(&c, &base, &login.cookie, "corp").await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["outcome"], "connected", "{body}");
    assert_eq!(body["stage"], "origin", "{body}");
    assert_eq!(body["traversedProxy"], true, "{body}");
    assert_eq!(
        body["endpoint"],
        format!("http://127.0.0.1:{}", proxy.port())
    );
    assert_eq!(body["probeUrl"], probe);
    assert!(body["detail"].as_str().is_some_and(|d| !d.is_empty()));

    // The verdict is backed by a real tunnel to the probe target's address.
    let seen = connects(&log);
    assert_eq!(seen.len(), 1, "{seen:?}");
    assert!(
        seen[0].starts_with(&format!("CONNECT {ORIGIN}:80 ")),
        "{seen:?}"
    );

    // And the test is on the audit record, with its outcome.
    let rows = Store::open(&db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap()
        .list_audit(50)
        .await
        .unwrap();
    assert!(
        rows.iter().any(|r| r.target.as_deref() == Some("corp")
            && r.detail_json.contains("\"tested\":true")
            && r.detail_json.contains("\"outcome\":\"connected\"")),
        "{rows:#?}"
    );
}

#[tokio::test]
async fn the_route_test_tells_failures_apart_and_says_where_each_stopped() {
    use_test_probe_url();
    let db = new_db("mw-t28e8-test-fail");
    let base = spawn_server(&db).await;
    let c = client();
    let login = admin_login(&c, &base).await;

    // 1. The proxy demands credentials and the route has none.
    let (auth_proxy, auth_log) = stand_in_proxy(true).await;
    put_route(
        &c,
        &base,
        &login.cookie,
        "needs-auth",
        "127.0.0.1",
        auth_proxy.port(),
        true,
    )
    .await;
    activate(&c, &base, &login.cookie, "needs-auth").await;
    let (status, body) = run_test(&c, &base, &login.cookie, "needs-auth").await;
    assert_eq!(status, 200, "a negative verdict is still a 200: {body}");
    assert_eq!(body["outcome"], "authRejected", "{body}");
    assert_eq!(body["stage"], "tunnel", "{body}");
    assert_eq!(body["traversedProxy"], false, "{body}");
    assert_eq!(connects(&auth_log).len(), 1, "the CONNECT was really sent");

    // 2. Nothing listens on the proxy's port.
    let closed = {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        l.local_addr().unwrap().port()
    };
    put_route(&c, &base, &login.cookie, "down", "127.0.0.1", closed, true).await;
    activate(&c, &base, &login.cookie, "down").await;
    let (status, body) = run_test(&c, &base, &login.cookie, "down").await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["outcome"], "unreachable", "{body}");
    assert_eq!(body["stage"], "connect", "{body}");
    assert_eq!(body["traversedProxy"], false, "{body}");

    // 3. The proxy's name does not resolve (`.invalid` is reserved, RFC 2606).
    put_route(
        &c,
        &base,
        &login.cookie,
        "typo",
        "proxy.mailwoman-test.invalid",
        3128,
        true,
    )
    .await;
    activate(&c, &base, &login.cookie, "typo").await;
    let (status, body) = run_test(&c, &base, &login.cookie, "typo").await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["outcome"], "dnsFailed", "{body}");
    assert_eq!(body["stage"], "dns", "{body}");

    // 4. A working proxy, but the route refuses plaintext origins and the probe
    //    URL is http: refused by policy, and no tunnel is asked for.
    let (ok_proxy, ok_log) = stand_in_proxy(false).await;
    put_route(
        &c,
        &base,
        &login.cookie,
        "tls-only",
        "127.0.0.1",
        ok_proxy.port(),
        false,
    )
    .await;
    activate(&c, &base, &login.cookie, "tls-only").await;
    let (status, body) = run_test(&c, &base, &login.cookie, "tls-only").await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["outcome"], "refusedByPolicy", "{body}");
    assert_eq!(body["stage"], "tunnel", "{body}");
    assert_eq!(body["traversedProxy"], false, "{body}");
    assert!(connects(&ok_log).is_empty(), "{:?}", connects(&ok_log));

    // Control for 4: the same proxy with plaintext allowed connects, so the refusal
    // above was the route's setting and not the proxy.
    put_route(
        &c,
        &base,
        &login.cookie,
        "tls-only",
        "127.0.0.1",
        ok_proxy.port(),
        true,
    )
    .await;
    let (_, body) = run_test(&c, &base, &login.cookie, "tls-only").await;
    assert_eq!(body["outcome"], "connected", "{body}");
}

#[tokio::test]
async fn activating_a_route_is_what_makes_the_image_proxy_use_it() {
    let db = new_db("mw-t28e8-activate");
    let mock = spawn_mock().await;
    let base = spawn_server(&db).await;
    let admin = client();
    let login = admin_login(&admin, &base).await;
    let (proxy, log) = stand_in_proxy(false).await;

    // A mailbox session, a message and a remote-image grant — without them the
    // image proxy answers before it fetches anything (the t22 arrangement).
    let user = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();
    let me: Value = user
        .post(format!("{base}/api/login"))
        .json(&json!({
            "jmapUrl": mock,
            "username": mw_mock_jmap::USER,
            "password": mw_mock_jmap::PASS,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(me["ok"], json!(true), "mailbox login: {me}");
    let account = me["accountId"].as_str().unwrap().to_string();
    let email_id = {
        let pool = raw(&db).await;
        sqlx::query(
            "INSERT INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json)
             VALUES (?1, 'imap', 'mail.example', 993, 1, ?1, X'00', '{}')",
        )
        .bind(&account)
        .execute(&pool)
        .await
        .expect("seed account row");
        pool.close().await;
        let store = Store::open(&db, ServerKey::from_hex(KEY_HEX).unwrap())
            .await
            .unwrap();
        let mailbox = store
            .upsert_mailbox(&mw_store::MailboxUpsert {
                account_id: &account,
                name: "INBOX",
                role: Some("inbox"),
                uidvalidity: 1,
                uidnext: 2,
                highestmodseq: 1,
                total: 1,
                unread: 0,
                parent_id: None,
            })
            .await
            .unwrap();
        let id = store
            .upsert_message(&mw_store::MessageUpsert {
                account_id: &account,
                mailbox_id: &mailbox,
                uid: 1,
                uidvalidity: 1,
                message_id: Some("<t28-e8@test>"),
                thread_id: None,
                internaldate: Some("2026-10-05T12:00:00Z"),
                size: 42,
                flags_json: "[]",
                envelope: None,
                blob_ref: None,
            })
            .await
            .unwrap();
        store.grant_remote_image(&account, "all", "").await.unwrap();
        id
    };

    // Saved, not activated: the list says so, and nothing has reached the proxy.
    put_route(
        &admin,
        &base,
        &login.cookie,
        "corp",
        "127.0.0.1",
        proxy.port(),
        true,
    )
    .await;
    let list: Value = get(
        &admin,
        format!("{base}/admin/egress/proxies"),
        &login.cookie,
    )
    .await
    .json()
    .await
    .unwrap();
    assert_eq!(list["proxies"][0]["id"], "corp");
    assert_eq!(
        list["proxies"][0]["active"], false,
        "saving a route does not put it in use: {list}"
    );
    assert!(log.lock().unwrap().is_empty());

    // Activate, then fetch an image.
    activate(&admin, &base, &login.cookie, "corp").await;
    let list: Value = get(
        &admin,
        format!("{base}/admin/egress/proxies"),
        &login.cookie,
    )
    .await
    .json()
    .await
    .unwrap();
    assert_eq!(list["proxies"][0]["active"], true, "{list}");

    let resp = user
        .get(format!("{base}/api/image-proxy"))
        .query(&[
            ("url", format!("http://{ORIGIN}/img")),
            ("emailId", email_id.clone()),
        ])
        .send()
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp.bytes().await.unwrap();
    assert_eq!(status, 200, "the image came back ({} bytes)", bytes.len());
    assert!(bytes.starts_with(b"\x89PNG"));
    let seen = connects(&log);
    assert_eq!(
        seen.len(),
        1,
        "the fetch went through the activated route: {seen:?}"
    );
    assert!(
        seen[0].starts_with(&format!("CONNECT {ORIGIN}:80 ")),
        "{seen:?}"
    );

    // Deactivate: the list reports no active route.
    let resp = post(
        &admin,
        format!("{base}/admin/egress/proxies/deactivate"),
        &login.cookie,
        None,
    )
    .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["deactivated"], true, "{body}");
    let list: Value = get(
        &admin,
        format!("{base}/admin/egress/proxies"),
        &login.cookie,
    )
    .await
    .json()
    .await
    .unwrap();
    assert_eq!(list["proxies"][0]["active"], false, "{list}");
}

// ── shapes: what the admin API returns and refuses ─────────────────────────────

#[tokio::test]
async fn integrations_report_what_the_deployment_has_configured() {
    let db = new_db("mw-t28e8-integrations");
    let base = spawn_server(&db).await;
    let c = client();
    let login = admin_login(&c, &base).await;
    let nextcloud_env = [
        "MW_NEXTCLOUD_URL",
        "MW_NEXTCLOUD_USER",
        "MW_NEXTCLOUD_APP_PASSWORD",
    ]
    .iter()
    .all(|k| std::env::var(k).is_ok_and(|v| !v.is_empty()));

    let before: Value = get(&c, format!("{base}/admin/integrations"), &login.cookie)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(before["ldap"], "not-configured", "{before}");
    assert_eq!(
        before["nextcloud"],
        if nextcloud_env {
            "configured"
        } else {
            "not-configured"
        },
        "{before}"
    );
    assert_eq!(before["webhooks"], "active");
    assert_eq!(before["apiKeyOversight"], "active");
    assert!(
        !before.to_string().contains("deferred"),
        "no integration is reported as deferred: {before}"
    );

    // A disabled directory entry is not a configured directory.
    let store = Store::open(&db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    let mut row = mw_store::DirectoryConfigRow {
        id: "corp-ldap".into(),
        url: "ldaps://ldap.example.org".into(),
        base_dn: "dc=example,dc=org".into(),
        bind_dn: None,
        tls: "ldaps".into(),
        priority: 0,
        attr_map_json: "{}".into(),
        enabled: false,
    };
    store.put_directory_config(&row).await.unwrap();
    let disabled: Value = get(&c, format!("{base}/admin/integrations"), &login.cookie)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(disabled["ldap"], "not-configured", "{disabled}");

    // An enabled one is.
    row.enabled = true;
    store.put_directory_config(&row).await.unwrap();
    let after: Value = get(&c, format!("{base}/admin/integrations"), &login.cookie)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(after["ldap"], "configured", "{after}");
}

#[tokio::test]
async fn the_security_policy_route_has_two_fields_and_refuses_the_removed_ones() {
    let db = new_db("mw-t28e8-policy");
    let base = spawn_server(&db).await;
    let c = client();
    let login = admin_login(&c, &base).await;
    let url = format!("{base}/admin/security-policy");

    let got: Value = get(&c, url.clone(), &login.cookie)
        .await
        .json()
        .await
        .unwrap();
    let mut keys: Vec<&str> = got
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, ["dlpRulesJson", "maxSecurityFloor"], "{got}");

    // Control: a body with exactly those fields is stored.
    let put = |body: Value| {
        c.put(url.clone())
            .header(reqwest::header::COOKIE, &login.cookie)
            .json(&body)
            .send()
    };
    let resp = put(json!({ "dlpRulesJson": "[{\"id\":\"r1\"}]", "maxSecurityFloor": true }))
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
    let got: Value = get(&c, url.clone(), &login.cookie)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(got["dlpRulesJson"], "[{\"id\":\"r1\"}]");
    assert_eq!(got["maxSecurityFloor"], true);

    // Each removed field is refused, and the refusal changes nothing.
    for (field, value) in [
        ("minTls", json!("1.3")),
        ("require2fa", json!(true)),
        ("capturePolicy", json!("metadata")),
        ("argon2MCost", json!(65536)),
        ("argon2TCost", json!(3)),
        ("argon2PCost", json!(2)),
    ] {
        let mut body = json!({ "dlpRulesJson": "[]", "maxSecurityFloor": false });
        body[field] = value;
        let resp = put(body).await.unwrap();
        assert_eq!(resp.status(), 422, "{field} is not accepted");
    }
    let got: Value = get(&c, url, &login.cookie).await.json().await.unwrap();
    assert_eq!(
        got["maxSecurityFloor"], true,
        "a refused PUT stored nothing"
    );
}

#[tokio::test]
async fn a_domain_is_its_name_and_registering_it_again_keeps_what_was_stored() {
    let db = new_db("mw-t28e8-domains");
    let base = spawn_server(&db).await;
    let c = client();
    let login = admin_login(&c, &base).await;

    // A row as a pre-26.20 panel stored it.
    let store = Store::open(&db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    store
        .upsert_domain(&mw_store::DomainRow {
            name: "old.example".into(),
            upstream_json: "{\"imap\":\"mail.old.example\"}".into(),
            allowlist_json: "[\"a@old.example\"]".into(),
            blocklist_json: "[]".into(),
        })
        .await
        .unwrap();

    // Register a new name (no body), and re-register the old one with the body an
    // older client would send.
    let resp = c
        .put(format!("{base}/admin/domains/new.example"))
        .header(reqwest::header::COOKIE, &login.cookie)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
    let resp = c
        .put(format!("{base}/admin/domains/old.example"))
        .header(reqwest::header::COOKIE, &login.cookie)
        .json(&json!({ "name": "old.example", "upstreamJson": "{}", "allowlist": [], "blocklist": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    let list: Value = get(&c, format!("{base}/admin/domains"), &login.cookie)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(
        list,
        json!([{ "name": "new.example" }, { "name": "old.example" }]),
        "a domain is sent as its name and nothing else"
    );

    // The stored columns of the old row were not overwritten by the re-register.
    let kept = store.get_domain("old.example").await.unwrap().unwrap();
    assert_eq!(kept.upstream_json, "{\"imap\":\"mail.old.example\"}");
    assert_eq!(kept.allowlist_json, "[\"a@old.example\"]");
}
