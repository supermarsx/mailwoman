//! t22-e14 — the other half of `t22_egress_admin_no_leak.rs`'s acceptance criterion:
//! **the proxy password across a FULL PROXIED FETCH**, not just the configuration path.
//!
//! e12's file drives the admin routes and captures `tracing`, response bodies and
//! panic messages while the credential is written, read back and deleted. It says in
//! its own module docs that it cannot cover the fetch, because at the time nothing
//! wired a stored route to the transport. This file is that extension, and it drives
//! the whole chain end to end:
//!
//! ```text
//!   admin API  →  sealed column  →  active_egress_proxy  →  ProxyRoute
//!              →  CONNECT + Proxy-Authorization on the wire  →  tunnelled fetch
//! ```
//!
//! # The trap this file is built around
//! **A capture taken across a fetch through an UNAUTHENTICATED proxy proves nothing.**
//! The credential would never have been transmitted, so "the password does not appear
//! in the logs" would be true of a run in which the password played no part at all —
//! a clean-looking answer from a check that could not see the thing it was asked
//! about.
//!
//! So the stand-in proxy **requires** authentication: it answers `407 Proxy
//! Authentication Required` when `Proxy-Authorization` is absent. And the assertions
//! run in this order, deliberately:
//!
//!   1. the **exact** `Proxy-Authorization: Basic <base64(user:password)>` value,
//!      computed independently in this test, is found in the head the proxy actually
//!      received — the secret was in play, on the wire, in this run;
//!   2. a route with **no** credentials against the same proxy fails — so the
//!      credential was *necessary*, not merely present;
//!   3. the capture is non-empty — it could have observed a leak;
//!   4. only then: neither the password nor its base64 encoding appears anywhere in
//!      the captured output or in any response body.
//!
//! Steps 1–3 are what make step 4 mean something. Any of them missing and a green run
//! is indistinguishable from a run that measured nothing.
//!
//! # No packet leaves the machine
//! The origin is `http://1.2.3.4/img` — ordinary public unicast, so it passes the
//! SSRF address gate, which a loopback origin cannot. The stand-in proxy answers
//! `200 Connection established` and then serves the response **itself** instead of
//! connecting to the authority it was handed, so `1.2.3.4` is only ever a string in a
//! `CONNECT` line.
//!
//! Run:
//!   cargo test -p mw-server --test t22_egress_fetch_no_leak -- --test-threads=1

use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

use mw_server::{AppConfig, build_app};
use mw_store::{ServerKey, Store};

mod common;
use common::test_db;

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";
const ADMIN_TOKEN: &str = "t22-e14-admin-token";

/// Distinctive enough that a substring match cannot collide with anything else the
/// server might legitimately emit.
const PASSWORD: &str = "vx9-Egress-Fetch-Secret-Never-Log-8c1d";
const USERNAME: &str = "svc-mail";

/// Public unicast, so the address gate admits it; nothing ever dials it.
const ORIGIN: &str = "1.2.3.4";

// ── capture ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct CapturedWriter(Arc<Mutex<Vec<u8>>>);

impl io::Write for CapturedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("capture lock").extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CapturedWriter {
    type Writer = CapturedWriter;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

// ── the stand-in proxy ─────────────────────────────────────────────────────────

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

/// A `CONNECT` proxy that **demands** `Proxy-Authorization`, records every head it
/// receives, and on success serves `body` itself rather than connecting upstream.
///
/// The `407` arm is the whole point: without it, a route carrying no credentials
/// would succeed, the password would never reach the wire, and every "it did not
/// leak" assertion below would be vacuously true.
async fn authenticating_proxy(body: &'static [u8]) -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&log);
    tokio::spawn(async move {
        while let Ok((mut client, _)) = listener.accept().await {
            let sink = Arc::clone(&sink);
            tokio::spawn(async move {
                let head = read_head(&mut client).await;
                sink.lock().unwrap().push(head.clone());
                if !head.contains("Proxy-Authorization:") {
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
                let _ = read_head(&mut client).await; // the tunnelled request head
                let mut resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\n\r\n",
                    body.len()
                )
                .into_bytes();
                resp.extend_from_slice(body);
                let _ = client.write_all(&resp).await;
                let _ = client.flush().await;
            });
        }
    });
    (addr, log)
}

/// RFC 7617 Basic credentials, computed here **independently of the implementation**.
///
/// Deliberately not `mw_egress`'s encoder: asserting the wire against the same
/// function that produced it would only prove the transport is self-consistent. This
/// value is what the secret looks like when it is genuinely in flight, derived from
/// the constants at the top of this file.
fn basic(user: &str, pass: &str) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let raw = format!("{user}:{pass}");
    let bytes = raw.as_bytes();
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[(n >> 18) as usize & 0x3f] as char);
        out.push(ALPHABET[(n >> 12) as usize & 0x3f] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 0x3f] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 0x3f] as char
        } else {
            '='
        });
    }
    out
}

// ── server + fixtures ──────────────────────────────────────────────────────────

/// A 1×1 opaque PNG, so the wasm media jail has something real to re-encode.
const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
    0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB0, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
    0x44, 0xAE, 0x42, 0x60, 0x82,
];

async fn spawn_mock() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, mw_mock_jmap::router()).await;
    });
    format!("http://{addr}")
}

async fn spawn_server(db_path: &str) -> SocketAddr {
    let web = PathBuf::from(db_path)
        .parent()
        .unwrap()
        .join(format!("web-{}", test_db::unique_tag()));
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: db_path.to_string(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    };
    let app = build_app(config).await.expect("build_app");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });
    addr
}

async fn store_at(db_path: &str) -> Store {
    Store::open(db_path, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .expect("open store")
}

/// The `admin_sessions` lookup key for a bearer token: lowercase-hex SHA-256, the
/// same shape `push_relay::hash_token` produces.
///
/// Recomputed here because that function is `pub(crate)` and an integration test
/// lives outside the crate. It needs no separate assertion: if this ever diverges
/// from the server's, every admin call becomes a `401` and `put_route` fails with
/// the server's own `admin authentication required` body in the message.
///
/// NOTE for whoever owns `t22_egress_admin_no_leak.rs`: that file calls
/// `mw_server::push_relay::hash_token` directly, which **does not compile** from a
/// test target for exactly this reason.
fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let digest = hasher.finalize();
    let mut s = String::with_capacity(digest.len() * 2);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Configure a route through the REAL admin API — so the password takes the real
/// path (JSON body → `PutProxyReq` → seal → column) rather than being written
/// straight into the store, which would skip every layer that could leak it.
async fn put_route(
    c: &reqwest::Client,
    base: &str,
    id: &str,
    proxy: SocketAddr,
    with_credentials: bool,
    bodies: &mut Vec<String>,
) {
    let mut payload = json!({
        "id": id,
        "scheme": "http",
        "host": proxy.ip().to_string(),
        "port": proxy.port(),
        "allowPlaintext": true,
    });
    if with_credentials {
        payload["username"] = json!(USERNAME);
        payload["password"] = json!(PASSWORD);
    }
    let resp = c
        .post(format!("{base}/admin/egress/proxies"))
        .header("cookie", format!("mw_admin_session={ADMIN_TOKEN}"))
        .json(&payload)
        .send()
        .await
        .expect("put route");
    let status = resp.status();
    let body = resp.text().await.unwrap();
    assert_eq!(status, 200, "put route {id} failed: {body}");
    bodies.push(body);
}

async fn activate(c: &reqwest::Client, base: &str, id: &str, bodies: &mut Vec<String>) {
    let resp = c
        .post(format!("{base}/admin/egress/proxies/{id}/activate"))
        .header("cookie", format!("mw_admin_session={ADMIN_TOKEN}"))
        .send()
        .await
        .expect("activate");
    let status = resp.status();
    let body = resp.text().await.unwrap();
    assert_eq!(status, 200, "activate {id} failed: {body}");
    bodies.push(body);
}

// ── the test ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn the_proxy_password_survives_a_full_proxied_fetch_without_leaking() {
    let captured = Arc::new(Mutex::new(Vec::<u8>::new()));
    let _guard = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(CapturedWriter(Arc::clone(&captured)))
                .with_ansi(false),
        )
        .with(tracing_subscriber::filter::LevelFilter::TRACE)
        .set_default();

    let (proxy, proxy_log) = authenticating_proxy(TINY_PNG).await;
    let db = test_db::unique_dir("mw-t22-egress-fetch-leak")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();

    // Seed the admin session before the app boots, exactly as e12's file does.
    {
        let store = store_at(&db).await;
        store
            .put_admin_session(
                &hash_token(ADMIN_TOKEN),
                "root",
                &chrono::Utc::now().to_rfc3339(),
            )
            .await
            .expect("seed admin session");
    }

    let mock = spawn_mock().await;
    let addr = spawn_server(&db).await;
    let base = format!("http://{addr}");
    let mut bodies = Vec::new();

    // A user session, and the account it belongs to.
    let user = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();
    let login: Value = user
        .post(format!("{base}/api/login"))
        .json(&json!({
            "jmapUrl": mock,
            "username": mw_mock_jmap::USER,
            "password": mw_mock_jmap::PASS,
        }))
        .send()
        .await
        .expect("login")
        .json()
        .await
        .expect("login body");
    assert_eq!(login["ok"], json!(true), "login failed: {login}");
    let account = login["accountId"].as_str().expect("accountId").to_string();

    // A message to hang the remote-image grant on. The grant gate is per-account and
    // per-message (t22-e7): without a message the request is answered by
    // `ungranted_response`, which fetches nothing at all — so this test would be
    // measuring a `403` and no credential would ever reach a wire.
    //
    // The `accounts` row is raw-inserted with the SESSION's account id. `messages`
    // has a foreign key to `accounts(id)`, and the public `create_account` mints a
    // random id, so there is no API way to make an account whose id is the one the
    // proxy-mode session carries (`lib.rs` sets it from the login username). Raw
    // insertion follows the precedent in `t17_note_seal.rs`.
    let email_id = {
        let opts = sqlx::sqlite::SqliteConnectOptions::new().filename(&db);
        let raw = sqlx::SqlitePool::connect_with(opts).await.unwrap();
        sqlx::query(
            "INSERT INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json)
             VALUES (?1, 'imap', 'mail.example', 993, 1, ?1, X'00', '{}')",
        )
        .bind(&account)
        .execute(&raw)
        .await
        .expect("seed account row");
        raw.close().await;

        let store = store_at(&db).await;
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
            .expect("seed mailbox");
        let id = store
            .upsert_message(&mw_store::MessageUpsert {
                account_id: &account,
                mailbox_id: &mailbox,
                uid: 1,
                uidvalidity: 1,
                message_id: Some("<egress-leak@test>"),
                thread_id: None,
                internaldate: Some("2026-08-16T12:00:00Z"),
                size: 42,
                flags_json: "[]",
                envelope: None,
                blob_ref: None,
            })
            .await
            .expect("seed message");
        store
            .grant_remote_image(&account, "all", "")
            .await
            .expect("grant remote images");
        id
    };

    let admin = reqwest::Client::builder().no_proxy().build().unwrap();

    // ── negative control FIRST: the proxy really does demand authentication ────
    // Without this, "the password reached the wire" would not tell us the password
    // was NEEDED — and a proxy that accepted anyone would make the whole capture a
    // measurement of nothing.
    put_route(&admin, &base, "no-creds", proxy, false, &mut bodies).await;
    activate(&admin, &base, "no-creds", &mut bodies).await;
    let url = format!(
        "{base}/api/image-proxy?url={}&emailId={}",
        urlencode(&format!("http://{ORIGIN}/img")),
        urlencode(&email_id)
    );
    let resp = user.get(&url).send().await.expect("uncredentialed fetch");
    let status = resp.status();
    bodies.push(resp.text().await.unwrap());
    assert_eq!(
        status, 502,
        "a route with no credentials must FAIL against a proxy that demands them — \
         if this succeeds, the proxy is not actually requiring authentication and \
         every leak assertion below is vacuous"
    );

    // ── the real thing: a credentialed route, a completed proxied fetch ───────
    put_route(&admin, &base, "corp", proxy, true, &mut bodies).await;
    activate(&admin, &base, "corp", &mut bodies).await;
    let resp = user.get(&url).send().await.expect("credentialed fetch");
    let status = resp.status();
    let image = resp.bytes().await.unwrap();
    assert_eq!(
        status,
        200,
        "the credentialed route must complete the fetch THROUGH the tunnel; body was \
         {} bytes",
        image.len()
    );
    assert!(
        image.starts_with(b"\x89PNG"),
        "the bytes served through the tunnel came back re-encoded"
    );

    // ── 1. the secret was in play, on the wire, in THIS run ───────────────────
    let heads = proxy_log.lock().unwrap().clone();
    let expected = format!("Proxy-Authorization: Basic {}", basic(USERNAME, PASSWORD));
    assert!(
        heads.iter().any(|h| h.contains(&expected)),
        "the proxy never received the credential, so nothing below is being tested. \
         Heads seen: {heads:#?}"
    );
    assert!(
        heads
            .iter()
            .any(|h| h.starts_with(&format!("CONNECT {ORIGIN}:80 "))),
        "and it was a real tunnel to the literal origin address: {heads:#?}"
    );

    // ── 2. the audit row tells the truth about traversal ──────────────────────
    // Requirement: `traversedProxy` comes from the transport's own progress, never
    // from "a route was configured". Both rows below had a route configured; only
    // one of them actually traversed it.
    let rows = store_at(&db)
        .await
        .list_audit(50)
        .await
        .expect("read audit log");
    let egress: Vec<&mw_store::AuditRow> = rows.iter().filter(|r| r.actor == "egress").collect();
    assert!(
        egress
            .iter()
            .any(|r| r.detail_json.contains("\"traversedProxy\":true")
                && r.detail_json.contains("\"configuredRoute\":\"corp\"")),
        "the completed proxied fetch must be audited as having traversed the proxy: \
         {egress:#?}"
    );
    assert!(
        egress
            .iter()
            .any(|r| r.detail_json.contains("\"traversedProxy\":false")
                && r.detail_json.contains("\"configuredRoute\":\"no-creds\"")),
        "and the fetch whose tunnel was REFUSED must be audited as not having \
         traversed it, even though a route was configured — a row that reported \
         intent as fact would be wrong exactly here: {egress:#?}"
    );

    // ── 3. the capture could have seen a leak ─────────────────────────────────
    tokio::task::yield_now().await;
    let logs = String::from_utf8_lossy(&captured.lock().unwrap().clone()).into_owned();
    assert!(
        !logs.is_empty(),
        "no tracing output was captured at all — this test cannot detect a log leak \
         and its clean result would mean nothing"
    );

    // ── 4. and it did not ─────────────────────────────────────────────────────
    let encoded = basic(USERNAME, PASSWORD);
    assert!(
        !logs.contains(PASSWORD),
        "the proxy password appears in captured tracing output across a proxied \
         fetch:\n{logs}"
    );
    assert!(
        !logs.contains(&encoded),
        "the BASE64 form of the credential appears in captured tracing output — a \
         leak of the encoded value is a leak:\n{logs}"
    );
    for (i, body) in bodies.iter().enumerate() {
        assert!(
            !body.contains(PASSWORD) && !body.contains(&encoded),
            "the proxy password appears in response body {i}: {body}"
        );
    }
    assert!(
        !String::from_utf8_lossy(&image).contains(PASSWORD),
        "the credential reached the proxied image bytes"
    );
}
