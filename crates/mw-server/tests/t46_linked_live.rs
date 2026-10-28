//! t46: linked child mailboxes get live updates over the parent's own realtime
//! stream, and only while the link is valid.
//!
//! Every leg drives the real router over HTTP in engine mode: real sign-ins,
//! a real step-up and the real link endpoint. The synthetic legs inject changes
//! through the [`PushHandle`] the app returns (the channel the engine bridge
//! feeds), each with a unique `Email` token, so a frame the engine emits on its
//! own cannot be mistaken for one. The live leg delivers a message over SMTP to
//! a Greenmail mailbox and waits for the engine's IDLE connection to turn it
//! into a frame on the parent's stream.
//!
//! Every "nothing arrives" assertion is paired with a delivery on the same
//! stream or on a peer stream after the cause, so an absence that would also
//! happen on a dead stream proves nothing.
//!
//! Run:
//!   cargo test -p mw-server --test t46_linked_live --locked -- --test-threads=1
//!   MW_T46_GREENMAIL=127.0.0.1:<imap>,127.0.0.1:<smtp> cargo test ... (live leg)

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, MutexGuard};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use mw_engine::StateChange;
use mw_server::{AppConfig, HardeningConfig, PushHandle, SecurityConfig, ServerMode, V6Config};
use mw_store::settings_policy::SettingPolicyRow;

mod common;
use common::mock_mail::MockPop3;
use common::test_db;

const KEY_HEX: &str = "46a1b2c3d4e5f6071829304a5b6c7d8e46a1b2c3d4e5f6071829304a5b6c7d8e";
const PASS: &str = "Linked-Passw0rd!";

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

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
    _pop: Option<MockPop3>,
}

async fn policy(db: &str, key: &str, value: &str) {
    mw_store::Store::open(db, mw_store::ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap()
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "t46-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
}

/// An engine-mode server whose one upstream (`upstream`, or a scripted POP3
/// server accepting `users`) serves every account, with linking switched on.
async fn spawn(users: &[&str], upstream: Option<String>) -> Server {
    let pop = if upstream.is_none() {
        let accounts: Vec<(&str, &str)> = users.iter().map(|u| (*u, PASS)).collect();
        Some(MockPop3::start_accounts(&accounts).await)
    } else {
        None
    };
    let upstream = upstream.unwrap_or_else(|| pop.as_ref().unwrap().url());
    let dir = test_db::unique_dir("mw-t46");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), "<!doctype html><title>MW</title>").unwrap();
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    policy(&db, "signin.retryCooldownSecs", "0").await;
    policy(&db, "accounts.linkingEnabled", "true").await;
    let config = AppConfig {
        db_path: db.clone(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(Vec::new()),
            upstream_default: Some(upstream),
            ..SecurityConfig::default()
        },
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    };
    let (app, push) = mw_server::build_app_full(config, v6).await.unwrap();
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
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap()
}

/// A browser sign-in: every cookie it set, its CSRF token, account and
/// storage authority.
#[derive(Clone)]
struct User {
    cookie: String,
    csrf: String,
    account_id: String,
    authority: String,
}

async fn login(srv: &Server, user: &str) -> User {
    let r = client()
        .post(format!("{}/api/login", srv.base))
        .json(&json!({ "username": user, "password": PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "login for {user}");
    let cookie = r
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .filter_map(|c| c.split(';').next())
        .filter(|c| !c.ends_with('='))
        .collect::<Vec<_>>()
        .join("; ");
    assert!(cookie.contains("mw_session="), "login sets mw_session");
    let body: Value = r.json().await.unwrap();
    User {
        cookie,
        csrf: body["csrfToken"].as_str().unwrap_or_default().to_string(),
        account_id: body["accountId"].as_str().unwrap().to_string(),
        authority: body["storageAuthority"].as_str().unwrap().to_string(),
    }
}

impl User {
    fn req(&self, m: reqwest::Method, url: String) -> reqwest::RequestBuilder {
        client()
            .request(m, url)
            .header("cookie", &self.cookie)
            .header("x-csrf-token", &self.csrf)
            .header("x-mailwoman-storage-authority", &self.authority)
    }
}

/// A verified link from `parent` to `child` (password login; no second factor).
/// Returns the link id, its revision and the child's storage authority.
async fn link(srv: &Server, parent: &User, child: &str) -> (String, String, String) {
    let step = parent
        .req(
            reqwest::Method::POST,
            format!("{}/api/security/step-up", srv.base),
        )
        .json(&json!({ "password": PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(step.status(), 200, "step-up");
    let r = parent
        .req(
            reqwest::Method::POST,
            format!("{}/api/accounts/link", srv.base),
        )
        .json(&json!({ "username": child, "password": PASS }))
        .send()
        .await
        .unwrap();
    let status = r.status();
    let body: Value = r.json().await.unwrap();
    assert_eq!(status, 201, "link {child}: {body}");
    (
        body["account"]["id"].as_str().unwrap().to_string(),
        body["account"]["revision"].as_str().unwrap().to_string(),
        body["account"]["storageAuthority"]
            .as_str()
            .unwrap()
            .to_string(),
    )
}

async fn unlink(srv: &Server, parent: &User, link: &str, revision: &str) {
    let r = parent
        .req(
            reqwest::Method::DELETE,
            format!("{}/api/accounts/{link}", srv.base),
        )
        .header("if-match", json!(revision).to_string())
        .send()
        .await
        .unwrap();
    assert!(r.status().is_success(), "unlink: {}", r.status());
}

/// The child's raw account id, as only the server knows it.
async fn child_account(srv: &Server, parent: &User, link: &str) -> String {
    let me: Value = parent
        .req(reqwest::Method::GET, format!("{}/api/me", srv.base))
        .header("x-mailwoman-link", link)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    me["accountId"].as_str().unwrap().to_string()
}

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

fn change(account_id: &str, token: &str) -> StateChange {
    StateChange {
        account_id: account_id.to_string(),
        email: token.into(),
        mailbox: token.into(),
        submission: "1".into(),
        thread: token.into(),
        crypto_key: "0".into(),
        mail_rule: "0".into(),
    }
}

// ── a stream, either transport ───────────────────────────────────────────────

enum Stream {
    Ws(Box<Ws>),
    Sse {
        body: futures_util::stream::BoxStream<'static, reqwest::Result<Vec<u8>>>,
        buf: String,
    },
}

async fn open(srv: &Server, user: &User, ws: bool) -> Stream {
    if ws {
        let mut req = format!("ws://{}/jmap/ws", srv.addr)
            .into_client_request()
            .unwrap();
        req.headers_mut()
            .insert("cookie", user.cookie.parse().unwrap());
        let (ws, _) = tokio_tungstenite::connect_async(req).await.expect("ws");
        Stream::Ws(Box::new(ws))
    } else {
        let resp = reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .get(format!("{}/jmap/eventsource", srv.base))
            .header("cookie", &user.cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "eventsource");
        Stream::Sse {
            body: resp.bytes_stream().map(|r| r.map(|b| b.to_vec())).boxed(),
            buf: String::new(),
        }
    }
}

impl Stream {
    /// The next non-ping frame, or `None` when `within` passes or it ends.
    async fn next(&mut self, within: Duration) -> Option<Value> {
        let deadline = tokio::time::Instant::now() + within;
        match self {
            Stream::Ws(ws) => loop {
                match tokio::time::timeout_at(deadline, ws.next()).await {
                    Ok(Some(Ok(Message::Text(t)))) => {
                        let v: Value = serde_json::from_str(t.as_str()).unwrap();
                        if v["@type"] != "ping" {
                            return Some(v);
                        }
                    }
                    Ok(Some(Ok(Message::Close(_)))) | Ok(None) | Ok(Some(Err(_))) | Err(_) => {
                        return None;
                    }
                    Ok(Some(Ok(_))) => {}
                }
            },
            Stream::Sse { body, buf } => loop {
                while let Some(at) = buf.find("\n\n") {
                    let event: String = buf.drain(..at + 2).collect();
                    for line in event.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            let v: Value = serde_json::from_str(data).unwrap();
                            if v["@type"] != "ping" {
                                return Some(v);
                            }
                        }
                    }
                }
                match tokio::time::timeout_at(deadline, body.next()).await {
                    Ok(Some(Ok(chunk))) => buf.push_str(&String::from_utf8_lossy(&chunk)),
                    _ => return None,
                }
            },
        }
    }

    /// Every frame up to and including the first that `done` accepts. Panics
    /// when it does not arrive within `within`.
    async fn until(&mut self, within: Duration, done: impl Fn(&Value) -> bool) -> Vec<Value> {
        let deadline = tokio::time::Instant::now() + within;
        let mut seen = Vec::new();
        loop {
            let left = deadline.saturating_duration_since(tokio::time::Instant::now());
            match self.next(left).await {
                Some(v) => {
                    let hit = done(&v);
                    seen.push(v);
                    if hit {
                        return seen;
                    }
                }
                None => panic!("the expected frame never arrived; saw {seen:#?}"),
            }
        }
    }

    /// Every frame for `within`.
    async fn drain(&mut self, within: Duration) -> Vec<Value> {
        let deadline = tokio::time::Instant::now() + within;
        let mut seen = Vec::new();
        loop {
            let left = deadline.saturating_duration_since(tokio::time::Instant::now());
            if left.is_zero() {
                return seen;
            }
            match self.next(left).await {
                Some(v) => seen.push(v),
                None => return seen,
            }
        }
    }
}

fn linked_token(frame: &Value, link: &str, token: &str) -> bool {
    frame["@type"] == "MailboxChange" && frame["link"] == link && frame["changed"]["Email"] == token
}

fn carries_token(frames: &[Value], token: &str) -> bool {
    frames.iter().any(|f| {
        f["changed"]["Email"] == token
            || f["changed"]
                .as_object()
                .is_some_and(|m| m.values().any(|v| v["Email"] == token))
    })
}

// ── legs ─────────────────────────────────────────────────────────────────────

/// Two parents link the same child (the same raw account id). Each stream gets
/// the child's change under its own link id and the child's authority; neither
/// sees the other's link id, a stranger sees nothing, and no frame carries the
/// child's raw account id. Unlinking stops one parent's delivery and not the
/// other's; revoking the child's sessions stops the rest.
#[tokio::test]
async fn same_child_two_owners_unlink_and_revocation() {
    let _guard = serial().await;
    for ws in [true, false] {
        let users = [
            "p1@example.org",
            "p2@example.org",
            "s@example.org",
            "c@example.org",
        ];
        let srv = spawn(&users, None).await;
        let p1 = login(&srv, users[0]).await;
        let p2 = login(&srv, users[1]).await;
        let stranger = login(&srv, users[2]).await;
        let (l1, r1, ca1) = link(&srv, &p1, users[3]).await;
        let (l2, _r2, ca2) = link(&srv, &p2, users[3]).await;
        assert_ne!(l1, l2, "each owner holds its own link id");
        assert_eq!(ca1, ca2, "the same mailbox has one storage authority");
        let child = child_account(&srv, &p1, &l1).await;
        assert_eq!(child, child_account(&srv, &p2, &l2).await);

        let mut s1 = open(&srv, &p1, ws).await;
        let mut s2 = open(&srv, &p2, ws).await;
        let mut s3 = open(&srv, &stranger, ws).await;
        // A stream opens with the current count of each of its links.
        let opening = s1
            .until(Duration::from_secs(5), |f| {
                f["@type"] == "MailboxChange" && f["link"] == l1.as_str()
            })
            .await;
        let first = opening.last().unwrap();
        assert_eq!(first["storageAuthority"], ca1.as_str());
        assert_eq!(first["changed"], json!({}));

        srv.push.send(change(&child, "t46-a"));
        let got1 = s1
            .until(Duration::from_secs(5), |f| linked_token(f, &l1, "t46-a"))
            .await;
        let got2 = s2
            .until(Duration::from_secs(5), |f| linked_token(f, &l2, "t46-a"))
            .await;
        assert_eq!(got1.last().unwrap()["storageAuthority"], ca1.as_str());
        for (frames, other) in [(&got1, &l2), (&got2, &l1)] {
            let text = serde_json::to_string(frames).unwrap();
            assert!(
                !text.contains(other.as_str()),
                "another owner's link id leaked: {text}"
            );
            assert!(
                !text.contains(&format!("\"{child}\"")),
                "raw child account id on the wire: {text}"
            );
        }
        // A parent's own change still arrives; the other owner never sees it.
        srv.push.send(change(&p1.account_id, "t46-own"));
        s1.until(Duration::from_secs(5), |f| {
            f["changed"][&p1.account_id]["Email"] == "t46-own"
        })
        .await;
        let leaked = s2.drain(Duration::from_millis(1500)).await;
        assert!(
            !carries_token(&leaked, "t46-own"),
            "p2 saw p1's own change: {leaked:?}"
        );
        let quiet = s3.drain(Duration::from_millis(500)).await;
        assert!(
            !carries_token(&quiet, "t46-a") && !carries_token(&quiet, "t46-own"),
            "a stranger saw linked traffic: {quiet:?}"
        );

        // Unlink: p1 stops at once, p2 keeps delivering the same change.
        unlink(&srv, &p1, &l1, &r1).await;
        srv.push.send(change(&child, "t46-b"));
        s2.until(Duration::from_secs(5), |f| linked_token(f, &l2, "t46-b"))
            .await;
        let after = s1.drain(Duration::from_millis(2500)).await;
        assert!(
            !carries_token(&after, "t46-b"),
            "unlinked owner still received: {after:?}"
        );
        srv.push.send(change(&p1.account_id, "t46-own-2"));
        s1.until(Duration::from_secs(5), |f| {
            f["changed"][&p1.account_id]["Email"] == "t46-own-2"
        })
        .await;

        // Revoked: the admin revokes every session of the child account, which
        // includes the link's private source session.
        let r = admin(&srv)
            .await
            .post(format!("{}/admin/users/{child}/revoke-sessions", srv.base))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 200, "revoke-sessions");
        srv.push.send(change(&child, "t46-c"));
        srv.push.send(change(&p2.account_id, "t46-p2-own"));
        let revoked = s2
            .until(Duration::from_secs(5), |f| {
                f["changed"][&p2.account_id]["Email"] == "t46-p2-own"
            })
            .await;
        assert!(
            !carries_token(&revoked, "t46-c"),
            "revoked link still delivered: {revoked:?}"
        );
        let later = s2.drain(Duration::from_millis(2500)).await;
        assert!(
            !carries_token(&later, "t46-c"),
            "revoked link delivered late: {later:?}"
        );
        // A stream opened now carries no link at all.
        let mut fresh = open(&srv, &p2, ws).await;
        srv.push.send(change(&child, "t46-d"));
        srv.push.send(change(&p2.account_id, "t46-p2-own-2"));
        let frames = fresh
            .until(Duration::from_secs(5), |f| {
                f["changed"][&p2.account_id]["Email"] == "t46-p2-own-2"
            })
            .await;
        assert!(
            frames
                .iter()
                .all(|f| f["@type"] != "MailboxChange" || f["link"].is_null()),
            "a revoked link reached a new stream: {frames:?}"
        );
    }
}

/// A link made while the parent's stream is open joins it without a reconnect,
/// and the parent's stream carries at most `accounts.maxLinked` links: lowering
/// the limit drops the links past it from open and new streams alike.
#[tokio::test]
async fn new_links_join_and_the_limit_is_enforced() {
    let _guard = serial().await;
    for ws in [true, false] {
        let users = ["p@example.org", "c1@example.org", "c2@example.org"];
        let srv = spawn(&users, None).await;
        let p = login(&srv, users[0]).await;
        let mut s = open(&srv, &p, ws).await;
        let (l1, _, _) = link(&srv, &p, users[1]).await;
        let (l2, _, _) = link(&srv, &p, users[2]).await;
        let c1 = child_account(&srv, &p, &l1).await;
        let c2 = child_account(&srv, &p, &l2).await;
        // Both links join the open stream (at the next re-check at the latest).
        s.until(Duration::from_secs(6), |f| f["link"] == l2.as_str())
            .await;
        srv.push.send(change(&c1, "t46-l1"));
        srv.push.send(change(&c2, "t46-l2"));
        s.until(Duration::from_secs(5), |f| linked_token(f, &l2, "t46-l2"))
            .await;

        policy(&srv.db, "accounts.maxLinked", "1").await;
        // One re-check period (1 s here) later the stream is down to one link.
        tokio::time::sleep(Duration::from_millis(2500)).await;
        let _ = s.drain(Duration::from_millis(200)).await;
        // Links created in the same second have no defined order, so which one
        // stays is not asserted; that exactly one does is.
        srv.push.send(change(&c1, "t46-one"));
        srv.push.send(change(&c2, "t46-two"));
        srv.push.send(change(&p.account_id, "t46-marker"));
        let mut frames = s
            .until(Duration::from_secs(5), |f| {
                f["changed"][&p.account_id]["Email"] == "t46-marker"
            })
            .await;
        frames.extend(s.drain(Duration::from_millis(1500)).await);
        let delivered = [
            carries_token(&frames, "t46-one"),
            carries_token(&frames, "t46-two"),
        ];
        assert_eq!(
            delivered.iter().filter(|d| **d).count(),
            1,
            "exactly one link stays live under accounts.maxLinked=1: {frames:?}"
        );
        let kept = if delivered[0] { &l1 } else { &l2 };

        let mut fresh = open(&srv, &p, ws).await;
        let opening = fresh.drain(Duration::from_millis(1500)).await;
        let links: Vec<&Value> = opening
            .iter()
            .filter(|f| f["@type"] == "MailboxChange" && !f["link"].is_null())
            .map(|f| &f["link"])
            .collect();
        assert_eq!(
            links,
            vec![&json!(kept)],
            "a new stream opens with one link: {opening:?}"
        );
    }
}

// ── live: Greenmail IDLE ─────────────────────────────────────────────────────

async fn smtp_deliver(addr: &str, to: &str, subject: &str) {
    let stream = TcpStream::connect(addr).await.expect("smtp connect");
    let (r, mut w) = stream.into_split();
    let mut r = BufReader::new(r);
    let mut line = String::new();
    async fn reply(r: &mut BufReader<tokio::net::tcp::OwnedReadHalf>, line: &mut String) {
        loop {
            line.clear();
            r.read_line(line).await.unwrap();
            assert!(!line.is_empty(), "smtp closed");
            if line.as_bytes().get(3) != Some(&b'-') {
                break;
            }
        }
        assert!(
            line.starts_with('2') || line.starts_with('3'),
            "smtp: {line}"
        );
    }
    reply(&mut r, &mut line).await;
    for cmd in [
        "EHLO t46.test".to_string(),
        "MAIL FROM:<sender@t46.test>".into(),
        format!("RCPT TO:<{to}>"),
        "DATA".into(),
    ] {
        w.write_all(format!("{cmd}\r\n").as_bytes()).await.unwrap();
        reply(&mut r, &mut line).await;
    }
    let body = format!(
        "From: sender@t46.test\r\nTo: {to}\r\nSubject: {subject}\r\nMessage-ID: <{subject}@t46.test>\r\n\r\nLive.\r\n.\r\n"
    );
    w.write_all(body.as_bytes()).await.unwrap();
    reply(&mut r, &mut line).await;
    w.write_all(b"QUIT\r\n").await.unwrap();
}

/// A message delivered to the child's Greenmail INBOX reaches the parent's
/// stream through the engine's IDLE connection, with the new unread count, and
/// the child's mailbox serves it when the parent switches to it.
#[tokio::test]
async fn greenmail_idle_reaches_the_parent_stream() {
    let Ok(spec) = std::env::var("MW_T46_GREENMAIL") else {
        eprintln!(
            "SKIP greenmail_idle_reaches_the_parent_stream: MW_T46_GREENMAIL=imap,smtp unset"
        );
        return;
    };
    let (imap, smtp) = spec.split_once(',').expect("MW_T46_GREENMAIL=imap,smtp");
    let _guard = serial().await;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let parent = format!("parent{nonce}@example.org");
    let child = format!("child{nonce}@example.org");
    for user in [&parent, &child] {
        // Greenmail creates the user on first delivery.
        smtp_deliver(smtp, user, &format!("seed-{nonce}")).await;
    }
    let srv = spawn(&[], Some(format!("imap://{imap}"))).await;
    let p = login(&srv, &parent).await;
    let (l, _, ca) = link(&srv, &p, &child).await;
    for ws in [true, false] {
        let mut s = open(&srv, &p, ws).await;
        let opening = s
            .until(Duration::from_secs(10), |f| f["link"] == l.as_str())
            .await;
        let before = opening.last().unwrap()["inboxUnread"].as_u64().unwrap_or(0);
        // Let the IDLE connection settle after the initial sync.
        tokio::time::sleep(Duration::from_secs(2)).await;
        let subject = format!("live-{nonce}-{ws}");
        let sent = std::time::Instant::now();
        smtp_deliver(smtp, &child, &subject).await;
        let frames = s
            .until(Duration::from_secs(30), |f| {
                f["@type"] == "MailboxChange"
                    && f["link"] == l.as_str()
                    && f["inboxUnread"].as_u64().unwrap_or(0) > before
            })
            .await;
        let elapsed = sent.elapsed();
        let last = frames.last().unwrap();
        assert_eq!(last["storageAuthority"], ca.as_str());
        eprintln!(
            "t46 live ({}): new mail reached the parent stream in {elapsed:?}, inboxUnread {before} -> {}",
            if ws { "ws" } else { "sse" },
            last["inboxUnread"]
        );
        // Switching to the child serves the new message straight away.
        let child_id = child_account(&srv, &p, &l).await;
        let q: Value = p
            .req(reqwest::Method::POST, format!("{}/jmap/api", srv.base))
            .header("x-mailwoman-link", &l)
            .header("x-mailwoman-storage-authority", &ca)
            .json(&json!({
                "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
                "methodCalls": [
                    ["Email/query", {"accountId": child_id, "sort": [{"property": "receivedAt", "isAscending": false}], "limit": 5}, "q"],
                    ["Email/get", {"accountId": child_id, "#ids": {"resultOf": "q", "name": "Email/query", "path": "/ids"}, "properties": ["subject"]}, "g"]
                ]
            }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let subjects = q["methodResponses"][1][1]["list"].to_string();
        assert!(
            subjects.contains(&subject),
            "the child's list is current: {q}"
        );
    }
}
