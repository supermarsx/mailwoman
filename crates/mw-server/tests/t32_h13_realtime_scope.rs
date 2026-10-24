//! t32-h13 (D-27): a realtime stream carries only its own account's changes and
//! ends when its session stops being valid.
//!
//! Before this hotfix `/jmap/ws` and `/jmap/eventsource` checked the session once
//! and then drained one process-wide broadcast, so every signed-in user received
//! every account's `StateChange` (account id and state tokens), and a stream kept
//! running after the account was disabled or its session revoked.
//!
//! Every leg drives the real router over HTTP in engine mode with two mailbox
//! users on one scripted POP3 upstream. Changes are injected through the
//! [`PushHandle`] the app returns, which is the channel the engine bridge feeds.
//! Each injected change carries a unique `Email` state token, so a frame the
//! engine itself happens to emit cannot be mistaken for it.
//!
//! Every "the stream ended" assertion is preceded, on the same stream, by the
//! stream surviving at least two re-check periods and still delivering — an end
//! that would also happen without the cause proves nothing.
//!
//! Legs (each on WebSocket and EventSource):
//!   * cookie session: B's stream never carries A's change, A's never B's
//!   * native bearer session: the same scope
//!   * disabling the account ends its streams; the other user's stream stays
//!   * the panel's "revoke sessions" ends them; a cookie sign-out ends them
//!   * a native bearer sign-out ends the bearer streams
//!   * an API key opens no stream at all
//!   * a store that cannot be read ends them (fail closed)
//!   * a session past its idle timeout ends them, and the re-check itself does
//!     not count as activity
//!
//! Run:
//!   cargo test -p mw-server --test t32_h13_realtime_scope --locked -- --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, MutexGuard};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use mw_engine::StateChange;
use mw_server::{AppConfig, HardeningConfig, PushHandle, SecurityConfig, ServerMode, V6Config};

mod common;
use common::mock_mail::MockPop3;
use common::test_db;

const KEY_HEX: &str = "32a1b2c3d4e5f6071829304a5b6c7d8e32a1b2c3d4e5f6071829304a5b6c7d8e";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T32_H13</div>";

const A_USER: &str = "alice@example.org";
const A_PASS: &str = "Alice-Passw0rd!";
const B_USER: &str = "bob@example.org";
const B_PASS: &str = "Bob-Passw0rd!";

/// The re-check period this binary runs with (`MW_REALTIME_RECHECK_SECS=1`).
const RECHECK: Duration = Duration::from_secs(1);
/// How long a stream must survive to show the re-check keeps a valid one open.
const SURVIVES: Duration = Duration::from_millis(2500);
/// How long a stream may take to end once its session is no longer valid: one
/// re-check period plus generous slack for a loaded machine.
const ENDS_WITHIN: Duration = Duration::from_secs(6);

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

// ── harness ──────────────────────────────────────────────────────────────────

/// Serialises the tests in this binary and sets the environment the server reads.
async fn serial() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK.get_or_init(Default::default).lock().await;
    // SAFETY: the guard serialises every mutation and every dependent read here.
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
        std::env::set_var("MW_REALTIME_RECHECK_SECS", "1");
    }
    guard
}

struct Server {
    base: String,
    addr: SocketAddr,
    db: String,
    push: PushHandle,
    _pop: MockPop3,
}

/// An engine-mode server whose one upstream accepts both users.
async fn spawn(hardening: HardeningConfig) -> Server {
    let pop = MockPop3::start_accounts(&[(A_USER, A_PASS), (B_USER, B_PASS)]).await;
    let dir = test_db::unique_dir("mw-t32-h13");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    let config = AppConfig {
        db_path: db.clone(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening,
        security: SecurityConfig {
            jmap_upstreams: Some(Vec::new()),
            upstream_default: Some(pop.url()),
            ..SecurityConfig::default()
        },
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    };
    let (app, push) = mw_server::build_app_full(config, v6)
        .await
        .expect("build_app_full");
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
    Server {
        base: format!("http://{addr}"),
        addr,
        db,
        push,
        _pop: pop,
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// How a stream authenticates.
#[derive(Clone)]
enum Auth {
    /// The raw `mw_session=…` cookie pair.
    Cookie(String),
    /// A native bearer token.
    Bearer(String),
}

impl Auth {
    fn header(&self) -> (&'static str, String) {
        match self {
            Auth::Cookie(pair) => ("cookie", pair.clone()),
            Auth::Bearer(token) => ("authorization", format!("Bearer {token}")),
        }
    }
}

/// A signed-in user: how to authenticate and which account the session is for.
#[derive(Clone)]
struct User {
    auth: Auth,
    account_id: String,
}

/// A browser sign-in: the session cookie and the account id.
async fn cookie_login(srv: &Server, user: &str, pass: &str) -> User {
    let r = client()
        .post(format!("{}/api/login", srv.base))
        .json(&json!({ "username": user, "password": pass }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "browser login for {user}");
    let pair = r
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .filter_map(|c| c.split(';').next())
        .find(|c| c.starts_with("mw_session=") && c.len() > "mw_session=".len())
        .expect("login sets mw_session")
        .to_string();
    let body: Value = r.json().await.unwrap();
    User {
        auth: Auth::Cookie(pair),
        account_id: body["accountId"].as_str().unwrap().to_string(),
    }
}

/// A native shell sign-in: the bearer token and the account id.
async fn native_login(srv: &Server, user: &str, pass: &str) -> User {
    let r = client()
        .post(format!("{}/api/login", srv.base))
        .json(&json!({ "username": user, "password": pass, "clientType": "native" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "native login for {user}");
    let body: Value = r.json().await.unwrap();
    User {
        auth: Auth::Bearer(body["token"].as_str().expect("a bearer token").to_string()),
        account_id: body["accountId"].as_str().unwrap().to_string(),
    }
}

/// A logged-in admin-panel client.
async fn admin(srv: &Server) -> reqwest::Client {
    let c = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let r = c
        .post(format!("{}/admin/login", srv.base))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "admin login");
    c
}

/// Set `disabled` on an account the way the panel does.
async fn disable(srv: &Server, id: &str) {
    let r = admin(srv)
        .await
        .put(format!("{}/admin/users/{id}/flags", srv.base))
        .json(&json!({
            "zeroAccess": false,
            "forcePasswordChange": false,
            "remoteCacheWipe": false,
            "disabled": true,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204, "the panel's flag write is accepted");
}

/// The panel's "revoke sessions" button.
async fn revoke_sessions(srv: &Server, id: &str) {
    let r = admin(srv)
        .await
        .post(format!("{}/admin/users/{id}/revoke-sessions", srv.base))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "revoke-sessions is accepted");
}

/// Sign out with the user's own credential.
async fn logout(srv: &Server, user: &User) {
    let (name, value) = user.auth.header();
    let r = client()
        .post(format!("{}/api/logout", srv.base))
        .header(name, value)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204, "logout");
}

/// A change for `account_id` whose `Email` state is the unique `token`.
fn change(account_id: &str, token: &str) -> StateChange {
    StateChange {
        account_id: account_id.to_string(),
        email: token.into(),
        mailbox: "1".into(),
        submission: "1".into(),
        thread: "1".into(),
        crypto_key: "0".into(),
        mail_rule: "0".into(),
    }
}

/// The account ids a `StateChange` wire object names, and its `Email` tokens.
fn describe(frame: &Value) -> (Vec<String>, Vec<String>) {
    let changed = frame["changed"].as_object().cloned().unwrap_or_default();
    let accounts = changed.keys().cloned().collect();
    let tokens = changed
        .values()
        .filter_map(|v| v["Email"].as_str().map(str::to_string))
        .collect();
    (accounts, tokens)
}

// ── a stream, either transport ───────────────────────────────────────────────

enum Stream {
    Ws(Box<Ws>),
    Sse {
        body: futures_util::stream::BoxStream<'static, reqwest::Result<Vec<u8>>>,
        buf: String,
    },
}

/// What reading a stream produced.
#[derive(Debug)]
enum Next {
    Change(Value),
    /// The server ended the stream; for WS the close code it sent, if any.
    Ended(Option<u16>),
    TimedOut,
}

async fn open_ws(srv: &Server, auth: &Auth) -> Result<Stream, String> {
    let mut req = format!("ws://{}/jmap/ws", srv.addr)
        .into_client_request()
        .unwrap();
    let (name, value) = auth.header();
    req.headers_mut().insert(name, value.parse().unwrap());
    match tokio_tungstenite::connect_async(req).await {
        Ok((ws, _)) => Ok(Stream::Ws(Box::new(ws))),
        Err(e) => Err(e.to_string()),
    }
}

async fn open_sse(srv: &Server, auth: &Auth) -> Result<Stream, u16> {
    let (name, value) = auth.header();
    let resp = client()
        .get(format!("{}/jmap/eventsource", srv.base))
        .header(name, value)
        .send()
        .await
        .unwrap();
    if resp.status() != 200 {
        return Err(resp.status().as_u16());
    }
    Ok(Stream::Sse {
        body: resp.bytes_stream().map(|r| r.map(|b| b.to_vec())).boxed(),
        buf: String::new(),
    })
}

/// Open both transports for `user`.
async fn open_both(srv: &Server, user: &User) -> [Stream; 2] {
    let ws = open_ws(srv, &user.auth).await.expect("ws upgrade");
    let sse = open_sse(srv, &user.auth).await.expect("eventsource 200");
    [ws, sse]
}

impl Stream {
    fn kind(&self) -> &'static str {
        match self {
            Stream::Ws(_) => "ws",
            Stream::Sse { .. } => "sse",
        }
    }

    /// The next `StateChange`, skipping keepalives, or the end of the stream.
    async fn next(&mut self, within: Duration) -> Next {
        let deadline = tokio::time::Instant::now() + within;
        match self {
            Stream::Ws(ws) => loop {
                let msg = match tokio::time::timeout_at(deadline, ws.next()).await {
                    Err(_) => return Next::TimedOut,
                    Ok(None) | Ok(Some(Err(_))) => return Next::Ended(None),
                    Ok(Some(Ok(m))) => m,
                };
                match msg {
                    Message::Text(t) => {
                        let v: Value = serde_json::from_str(t.as_str()).unwrap();
                        if v["@type"] == "StateChange" {
                            return Next::Change(v);
                        }
                    }
                    Message::Close(frame) => {
                        return Next::Ended(frame.map(|f| u16::from(f.code)));
                    }
                    _ => {}
                }
            },
            Stream::Sse { body, buf } => loop {
                while let Some(at) = buf.find("\n\n") {
                    let event: String = buf.drain(..at + 2).collect();
                    for line in event.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            let v: Value = serde_json::from_str(data).unwrap();
                            if v["@type"] == "StateChange" {
                                return Next::Change(v);
                            }
                        }
                    }
                }
                match tokio::time::timeout_at(deadline, body.next()).await {
                    Err(_) => return Next::TimedOut,
                    Ok(None) | Ok(Some(Err(_))) => return Next::Ended(None),
                    Ok(Some(Ok(chunk))) => buf.push_str(&String::from_utf8_lossy(&chunk)),
                }
            },
        }
    }

    /// Every change up to and including the one carrying `token`. Panics if the
    /// stream ends or `token` does not arrive.
    async fn until(&mut self, token: &str) -> Vec<Value> {
        let mut seen = Vec::new();
        loop {
            match self.next(Duration::from_secs(5)).await {
                Next::Change(v) => {
                    let done = describe(&v).1.iter().any(|t| t == token);
                    seen.push(v);
                    if done {
                        return seen;
                    }
                }
                other => panic!(
                    "{} stream: {token:?} never arrived ({other:?}); saw {seen:?}",
                    self.kind()
                ),
            }
        }
    }

    /// Assert the stream ends within [`ENDS_WITHIN`], ignoring any change it
    /// still carries before that. For WS, the close code must be 1008.
    async fn assert_ends(&mut self, why: &str) {
        let deadline = tokio::time::Instant::now() + ENDS_WITHIN;
        loop {
            let left = deadline.saturating_duration_since(tokio::time::Instant::now());
            match self.next(left).await {
                Next::Change(_) => continue,
                Next::Ended(code) => {
                    if let Stream::Ws(_) = self {
                        assert_eq!(
                            code,
                            Some(1008),
                            "ws stream ended after {why} without a policy close"
                        );
                    }
                    return;
                }
                Next::TimedOut => panic!(
                    "{} stream still open {ENDS_WITHIN:?} after {why}",
                    self.kind()
                ),
            }
        }
    }
}

/// The control for every "it ended" leg: the stream survives at least two
/// re-checks and still delivers a fresh change for its own account.
async fn assert_alive(srv: &Server, streams: &mut [Stream], user: &User, tag: &str) {
    tokio::time::sleep(SURVIVES).await;
    let token = format!("h13-alive-{tag}");
    srv.push.send(change(&user.account_id, &token));
    for s in streams.iter_mut() {
        s.until(&token).await;
    }
}

// ── 1. scope ─────────────────────────────────────────────────────────────────

/// A's and B's streams each carry only their own account's changes.
async fn assert_scoped(srv: &Server, a: &User, b: &User) {
    assert_ne!(a.account_id, b.account_id, "the premise: two accounts");
    let mut a_streams = open_both(srv, a).await;
    let mut b_streams = open_both(srv, b).await;

    // A's change first, then B's: B's streams must reach B's change without
    // having carried A's on the way, and the other way round.
    srv.push.send(change(&a.account_id, "h13-a-1"));
    srv.push.send(change(&b.account_id, "h13-b-1"));
    srv.push.send(change(&a.account_id, "h13-a-2"));

    for s in b_streams.iter_mut() {
        let seen = s.until("h13-b-1").await;
        for v in &seen {
            let (accounts, tokens) = describe(v);
            assert!(
                !accounts.contains(&a.account_id) && !tokens.iter().any(|t| t.starts_with("h13-a")),
                "{} stream of B carried A's change: {v}",
                s.kind()
            );
        }
    }
    for s in a_streams.iter_mut() {
        let seen = s.until("h13-a-2").await;
        assert!(
            seen.iter()
                .any(|v| describe(v).1.iter().any(|t| t == "h13-a-1")),
            "{} stream of A missed its own first change: {seen:?}",
            s.kind()
        );
        for v in &seen {
            let (accounts, tokens) = describe(v);
            assert!(
                !accounts.contains(&b.account_id) && !tokens.iter().any(|t| t.starts_with("h13-b")),
                "{} stream of A carried B's change: {v}",
                s.kind()
            );
        }
    }
}

#[tokio::test]
async fn a_cookie_stream_carries_only_its_own_accounts_changes() {
    let _g = serial().await;
    let srv = spawn(HardeningConfig::default()).await;
    let a = cookie_login(&srv, A_USER, A_PASS).await;
    let b = cookie_login(&srv, B_USER, B_PASS).await;
    assert_scoped(&srv, &a, &b).await;
}

#[tokio::test]
async fn a_bearer_stream_carries_only_its_own_accounts_changes() {
    let _g = serial().await;
    let srv = spawn(HardeningConfig::default()).await;
    let a = native_login(&srv, A_USER, A_PASS).await;
    let b = native_login(&srv, B_USER, B_PASS).await;
    assert_scoped(&srv, &a, &b).await;
}

// ── 2. disable ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn disabling_the_account_ends_its_cookie_and_bearer_streams_only() {
    let _g = serial().await;
    let srv = spawn(HardeningConfig::default()).await;
    let a_cookie = cookie_login(&srv, A_USER, A_PASS).await;
    let a_bearer = native_login(&srv, A_USER, A_PASS).await;
    let b = cookie_login(&srv, B_USER, B_PASS).await;
    let mut a_streams = open_both(&srv, &a_cookie).await;
    let mut a_bearer_streams = open_both(&srv, &a_bearer).await;
    let mut b_streams = open_both(&srv, &b).await;
    assert_alive(&srv, &mut a_streams, &a_cookie, "a").await;
    assert_alive(&srv, &mut a_bearer_streams, &a_bearer, "a-bearer").await;

    disable(&srv, A_USER).await;

    for s in a_streams.iter_mut().chain(a_bearer_streams.iter_mut()) {
        s.assert_ends("the account was disabled").await;
    }
    // B is untouched: its streams survive the same re-checks and still deliver.
    assert_alive(&srv, &mut b_streams, &b, "b").await;
}

// ── 3. revoke and sign-out ───────────────────────────────────────────────────

#[tokio::test]
async fn revoke_sessions_ends_the_accounts_streams() {
    let _g = serial().await;
    let srv = spawn(HardeningConfig::default()).await;
    let a = cookie_login(&srv, A_USER, A_PASS).await;
    let b = cookie_login(&srv, B_USER, B_PASS).await;
    let mut a_streams = open_both(&srv, &a).await;
    let mut b_streams = open_both(&srv, &b).await;
    assert_alive(&srv, &mut a_streams, &a, "a").await;

    revoke_sessions(&srv, A_USER).await;

    for s in a_streams.iter_mut() {
        s.assert_ends("revoke-sessions").await;
    }
    assert_alive(&srv, &mut b_streams, &b, "b").await;
}

#[tokio::test]
async fn signing_out_ends_the_sessions_streams() {
    let _g = serial().await;
    let srv = spawn(HardeningConfig::default()).await;
    let cookie = cookie_login(&srv, A_USER, A_PASS).await;
    let bearer = native_login(&srv, A_USER, A_PASS).await;
    let mut cookie_streams = open_both(&srv, &cookie).await;
    let mut bearer_streams = open_both(&srv, &bearer).await;
    assert_alive(&srv, &mut cookie_streams, &cookie, "cookie").await;
    assert_alive(&srv, &mut bearer_streams, &bearer, "bearer").await;

    // Signing out the bearer token ends only the bearer streams.
    logout(&srv, &bearer).await;
    for s in bearer_streams.iter_mut() {
        s.assert_ends("the bearer token was signed out").await;
    }
    assert_alive(&srv, &mut cookie_streams, &cookie, "cookie-2").await;

    logout(&srv, &cookie).await;
    for s in cookie_streams.iter_mut() {
        s.assert_ends("the cookie session was signed out").await;
    }
}

// ── 4. API key ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn an_api_key_opens_no_stream() {
    let _g = serial().await;
    let srv = spawn(HardeningConfig::default()).await;
    let a = cookie_login(&srv, A_USER, A_PASS).await;
    let (name, value) = a.auth.header();
    let mint: Value = client()
        .post(format!("{}/api/keys", srv.base))
        .header(name, value)
        .json(&json!({
            "label": "t32-h13",
            "accountId": a.account_id,
            "scope": {
                "read": true, "send": false, "delete": false,
                "accounts": { "subset": [a.account_id] }, "folders": "all",
                "mail": true, "pim": false, "ip_allowlist": [], "expires_at": null,
                "rate_limit": null, "mcp_tools": ["mail.search"], "unattended_send": false,
            },
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let key = mint["displayToken"]
        .as_str()
        .unwrap_or_else(|| panic!("/api/keys returned no displayToken: {mint}"))
        .to_string();
    // Control: the key itself works on the REST surface it was minted for.
    let rest = client()
        .get(format!("{}/api/v1/mailboxes", srv.base))
        .bearer_auth(&key)
        .send()
        .await
        .unwrap();
    assert_eq!(rest.status(), 200, "control: the API key reads /api/v1");

    let as_key = Auth::Bearer(key);
    let ws = open_ws(&srv, &as_key).await.err().expect("ws refused");
    assert!(ws.contains("401"), "ws upgrade with an API key: {ws}");
    assert_eq!(open_sse(&srv, &as_key).await.err(), Some(401));
}

// ── 5. fail closed ───────────────────────────────────────────────────────────

#[tokio::test]
async fn a_store_that_cannot_be_read_ends_the_streams() {
    let _g = serial().await;
    let srv = spawn(HardeningConfig::default()).await;
    let a = cookie_login(&srv, A_USER, A_PASS).await;
    let bearer = native_login(&srv, A_USER, A_PASS).await;
    let mut streams = open_both(&srv, &a).await;
    let mut bearer_streams = open_both(&srv, &bearer).await;
    assert_alive(&srv, &mut streams, &a, "a").await;
    assert_alive(&srv, &mut bearer_streams, &bearer, "a-bearer").await;

    // The flags live in `settings`; the session rows stay readable, so the end
    // can only come from the re-check failing to read the account's flags.
    let raw =
        sqlx::SqlitePool::connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&srv.db))
            .await
            .unwrap();
    sqlx::query("ALTER TABLE settings RENAME TO settings_away")
        .execute(&raw)
        .await
        .unwrap();

    for s in streams.iter_mut().chain(bearer_streams.iter_mut()) {
        s.assert_ends("the flags became unreadable").await;
    }

    sqlx::query("ALTER TABLE settings_away RENAME TO settings")
        .execute(&raw)
        .await
        .unwrap();
}

// ── 6. expiry ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_session_past_its_idle_timeout_ends_its_streams() {
    let _g = serial().await;
    let idle = Duration::from_secs(4);
    let srv = spawn(HardeningConfig {
        idle_timeout: idle,
        ..HardeningConfig::default()
    })
    .await;
    let a = cookie_login(&srv, A_USER, A_PASS).await;
    let mut streams = open_both(&srv, &a).await;

    // Inside the idle window the streams are open and deliver (two re-checks
    // have run by now: had they counted as activity the session would never
    // idle out and the assertion below would fail).
    assert_alive(&srv, &mut streams, &a, "a").await;
    assert!(SURVIVES + RECHECK < idle);

    for s in streams.iter_mut() {
        s.assert_ends("the idle timeout passed").await;
    }
}
