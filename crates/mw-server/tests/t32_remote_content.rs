//! t32 e31a — signed remote-content URLs (`/api/rc/*`), end to end.
//!
//! A user signs in (proxy mode, mock JMAP upstream), holds a grant for a message,
//! and asks `POST /api/rc/sign` for image and part URLs. The image URL is then
//! fetched **without a cookie**, as the sandboxed body frame does. Remote origins
//! are `http://1.2.3.4/…` through a stand-in proxy route that answers the
//! tunnelled request itself, so nothing leaves the machine; the stand-in records
//! what it was sent and how many connections it saw.
//!
//! The token format is checked from outside: this file derives the signing key
//! from the server key with its own HMAC code and re-mints the server's token
//! byte for byte, then forges expired and retargeted variants with it.
//!
//! Run:
//!   cargo test -p mw-server --test t32_remote_content -- --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, KeyInit, Mac};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use mw_server::{AppConfig, V6Config, build_app_full};
use mw_store::{ServerKey, Store};

mod common;
use common::test_db;

const KEY_HEX: &str = "2f2e3d4c5b6a79889796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f2";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

/// A 4×4 opaque PNG (larger than a tracking pixel).
fn png(w: u32, h: u32) -> Vec<u8> {
    // Built here (not a fixture) so the size is visible in the test.
    fn crc(data: &[u8]) -> u32 {
        let mut c = 0xffff_ffffu32;
        for &b in data {
            c ^= u32::from(b);
            for _ in 0..8 {
                c = if c & 1 != 0 {
                    0xedb8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
            }
        }
        !c
    }
    fn adler(data: &[u8]) -> u32 {
        let (mut a, mut b) = (1u32, 0u32);
        for &d in data {
            a = (a + u32::from(d)) % 65521;
            b = (b + a) % 65521;
        }
        (b << 16) | a
    }
    fn chunk(out: &mut Vec<u8>, kind: &[u8], data: &[u8]) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut c = kind.to_vec();
        c.extend_from_slice(data);
        out.extend_from_slice(&c);
        out.extend_from_slice(&crc(&c).to_be_bytes());
    }
    let mut raw = Vec::new();
    for _ in 0..h {
        raw.extend_from_slice(&[0u8]);
        for _ in 0..w {
            raw.extend_from_slice(&[0x20, 0x40, 0x80]);
        }
    }
    // zlib "stored" block: no compression needed for a tiny image.
    let mut z = vec![0x78, 0x01, 0x01];
    z.extend_from_slice(&(raw.len() as u16).to_le_bytes());
    z.extend_from_slice(&(!(raw.len() as u16)).to_le_bytes());
    z.extend_from_slice(&raw);
    z.extend_from_slice(&adler(&raw).to_be_bytes());
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}

fn png_size(png: &[u8]) -> (u32, u32) {
    (
        u32::from_be_bytes(png[16..20].try_into().unwrap()),
        u32::from_be_bytes(png[20..24].try_into().unwrap()),
    )
}

// ── the stand-in proxy ───────────────────────────────────────────────────────────

#[derive(Default)]
struct Seen {
    connections: usize,
    requests: Vec<String>,
}

/// Read an HTTP head through `\r\n\r\n`.
async fn read_head(s: &mut TcpStream) -> String {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while s.read(&mut byte).await.unwrap_or(0) == 1 {
        head.push(byte[0]);
        if head.ends_with(b"\r\n\r\n") || head.len() > 16384 {
            break;
        }
    }
    String::from_utf8_lossy(&head).into_owned()
}

/// A `CONNECT` proxy that answers each tunnelled request with the next reply from
/// `replies` (the last one repeats), recording the request heads.
async fn standin_proxy(replies: Vec<(String, Vec<u8>)>) -> (SocketAddr, Arc<Mutex<Seen>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let seen = Arc::new(Mutex::new(Seen::default()));
    let sink = Arc::clone(&seen);
    let replies = Arc::new(replies);
    tokio::spawn(async move {
        while let Ok((mut c, _)) = listener.accept().await {
            let sink = Arc::clone(&sink);
            let replies = Arc::clone(&replies);
            tokio::spawn(async move {
                let n = {
                    let mut s = sink.lock().unwrap();
                    s.connections += 1;
                    s.connections - 1
                };
                let _ = read_head(&mut c).await;
                let _ = c
                    .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                    .await;
                let req = read_head(&mut c).await;
                sink.lock().unwrap().requests.push(req);
                let (ctype, body) = &replies[n.min(replies.len() - 1)];
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\n\r\n",
                    body.len()
                );
                let _ = c.write_all(head.as_bytes()).await;
                let _ = c.write_all(body).await;
                let _ = c.flush().await;
            });
        }
    });
    (addr, seen)
}

// ── the server ───────────────────────────────────────────────────────────────────

static MOCK_UPSTREAMS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

async fn spawn_mock() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let app = mw_mock_jmap::router().layer(axum::middleware::map_response(move |response: axum::response::Response| async move {
            let (parts, body) = response.into_parts();
            let bytes = axum::body::to_bytes(body, 1024 * 1024).await.unwrap();
            let mut value = serde_json::from_slice::<Value>(&bytes).ok();
            if let Some(calls) = value.as_mut().and_then(|v| v["methodResponses"].as_array_mut()) {
                for call in calls {
                    if call[0] == "Email/get" && let Some(list) = call[1]["list"].as_array_mut() {
                        for email in list {
                            if email["id"] == "e1" {
                                email["bodyStructure"] = json!({"type":"multipart/related","subParts":[{"blobId":"Mblob1","cid":"logo@example.test","type":"image/png"}]});
                                email["htmlBody"] = json!([{"partId":"html","type":"text/html"}]);
                                email["bodyValues"] = json!({"html":{"value":"<a href='http://1.2.3.4/dl/report.html'>Report</a><img src='http://1.2.3.4/download-image.png'><img src='http://1.2.3.4/download-svg.svg'><img src='http://1.2.3.4/declared-pixel.png' width='1' height='1'>","isTruncated":false,"isEncodingProblem":false}});
                            }
                        }
                    }
                }
            }
            // Each isolated test server owns a distinct account; exhausting its
            // rate budget must not consume another fixture's process-wide bucket.
            let body = value.map_or_else(|| bytes.to_vec(), |v| {
                serde_json::to_string(&v).unwrap().replace(
                    &format!("\"{}\"", mw_mock_jmap::ACCOUNT_ID),
                    &format!("\"rc-account-{}\"", addr.port()),
                ).into_bytes()
            });
            let mut result = axum::response::Response::from_parts(parts, axum::body::Body::from(body));
            result.headers_mut().remove("content-length");
            result
        }));
        let _ = axum::serve(listener, app).await;
    });
    let origin = format!("http://{addr}");
    MOCK_UPSTREAMS.lock().unwrap().push(origin.clone());
    origin
}

async fn spawn_server(db_path: &str, mock: &str) -> SocketAddr {
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
        security: mw_server::SecurityConfig {
            jmap_upstreams: Some(MOCK_UPSTREAMS.lock().unwrap().clone()),
            upstream_default: Some(mock.to_string()),
            ..mw_server::SecurityConfig::default()
        },
    };
    let (app, _) = build_app_full(
        config,
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("rc-fixture-password".into()),
            redis_url: None,
        },
    )
    .await
    .expect("build_app");
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

// ── the token, from outside ──────────────────────────────────────────────────────

fn hmac(key: &[u8], parts: &[&[u8]]) -> Vec<u8> {
    let mut m = Hmac::<Sha256>::new_from_slice(key).unwrap();
    for p in parts {
        m.update(p);
    }
    m.finalize().into_bytes().to_vec()
}

/// The signing key, derived as the server derives it.
fn signing_key() -> Vec<u8> {
    let server_key: Vec<u8> = (0..32)
        .map(|i| u8::from_str_radix(&KEY_HEX[i * 2..i * 2 + 2], 16).unwrap())
        .collect();
    hmac(
        &server_key,
        &[b"mailwoman-subkey:", b"mw-remote-content-v1"],
    )
}

fn mint(claims: &Value) -> String {
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims).unwrap());
    let tag = hmac(&signing_key(), &[b"mw-rc-token\0", payload.as_bytes()]);
    format!("{payload}.{}", URL_SAFE_NO_PAD.encode(tag))
}

fn claims_of(token: &str) -> Value {
    let payload = token.split_once('.').unwrap().0;
    serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).unwrap()).unwrap()
}

// ── fixture ──────────────────────────────────────────────────────────────────────

struct Fixture {
    base: String,
    user: reqwest::Client,
    admin: reqwest::Client,
    anon: reqwest::Client,
    email_id: String,
    account: String,
    db: String,
}

async fn fixture(proxy: SocketAddr, tag: &str) -> Fixture {
    let db = test_db::unique_dir(tag)
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let mock = spawn_mock().await;
    let addr = spawn_server(&db, &mock).await;
    let base = format!("http://{addr}");
    let user = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();
    let login: Value = user
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(login["ok"], json!(true), "login failed: {login}");
    let account = login["accountId"].as_str().unwrap().to_string();
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
        .unwrap();
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
            .unwrap();
        store
            .upsert_message(&mw_store::MessageUpsert {
                account_id: &account,
                mailbox_id: &mailbox,
                uid: 1,
                uidvalidity: 1,
                message_id: Some("<rc@test>"),
                thread_id: None,
                internaldate: Some("2026-10-25T12:00:00Z"),
                size: 42,
                flags_json: "[]",
                envelope: None,
                blob_ref: None,
            })
            .await
            .unwrap()
    };
    // Console and mailbox sign-ins share the trusted-client cooldown.
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    // The route that carries remote content: the stand-in proxy.
    let admin = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let login = admin
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":"root","password":"rc-fixture-password"}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "{}", login.text().await.unwrap());
    let proof = admin
        .post(format!("{base}/admin/step-up"))
        .json(&json!({"password":"rc-fixture-password"}))
        .send()
        .await
        .unwrap();
    assert_eq!(proof.status(), 200, "{}", proof.text().await.unwrap());
    for (path, body) in [
        (
            "/admin/egress/proxies".to_string(),
            Some(json!({
                "id": "rc",
                "scheme": "http",
                "host": proxy.ip().to_string(),
                "port": proxy.port(),
                "allowPlaintext": true,
            })),
        ),
        ("/admin/egress/proxies/rc/activate".to_string(), None),
    ] {
        let mut req = admin.post(format!("{base}{path}"));
        if let Some(b) = body {
            req = req.json(&b);
        }
        let r = req.send().await.unwrap();
        assert_eq!(r.status(), 200, "{path}: {}", r.text().await.unwrap());
    }
    Fixture {
        base,
        user,
        admin,
        anon: reqwest::Client::builder().no_proxy().build().unwrap(),
        email_id,
        account,
        db,
    }
}

impl Fixture {
    async fn sign(&self, urls: &[&str], parts: &[&str]) -> (u16, Value) {
        let r = self
            .user
            .post(format!("{}/api/rc/sign", self.base))
            .json(&json!({ "emailId": self.email_id, "urls": urls, "parts": parts }))
            .send()
            .await
            .unwrap();
        let status = r.status().as_u16();
        (status, r.json().await.unwrap_or(Value::Null))
    }

    async fn grant(&self) {
        store_at(&self.db)
            .await
            .grant_remote_image(&self.account, "all", "")
            .await
            .unwrap();
    }

    /// GET a signed path the way the sandboxed frame does: no cookie.
    async fn load(&self, src: &str) -> reqwest::Response {
        self.anon
            .get(format!("{}{src}", self.base))
            .send()
            .await
            .unwrap()
    }
}

fn image_src(signed: &Value) -> String {
    signed["images"][0]["src"].as_str().unwrap().to_string()
}

fn token_of(src: &str) -> String {
    src.rsplit('/').next().unwrap().to_string()
}

// ── the tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_signed_image_loads_without_a_cookie_and_forwards_nothing_of_the_reader() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture(proxy, "mw-t32-rc-load").await;

    // Without a grant nothing is signed.
    let (status, _) = f.sign(&["http://1.2.3.4/load.png"], &[]).await;
    assert_eq!(status, 403);
    f.grant().await;
    let (status, signed) = f.sign(&["http://1.2.3.4/load.png"], &[]).await;
    assert_eq!(status, 200, "{signed}");
    let src = image_src(&signed);
    assert!(src.starts_with("/api/rc/i/"), "{src}");

    let r = f
        .anon
        .get(format!("{}{src}", f.base))
        .header("cookie", "mw_session=not-forwarded-xyz")
        .header("referer", "https://reader.example/secret-path")
        .header("user-agent", "Mozilla/5.0 (Reader Browser)")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(r.headers()["content-type"], "image/png");
    assert_eq!(r.headers()["cross-origin-resource-policy"], "cross-origin");
    assert_eq!(r.headers()["cache-control"], "private, no-cache");
    assert!(r.headers().get("x-mw-remote-content").is_none());
    let body = r.bytes().await.unwrap();
    assert_eq!(png_size(&body), (4, 4), "re-encoded, same size");

    let seen = seen.lock().unwrap();
    assert_eq!(seen.requests.len(), 1);
    let head = seen.requests[0].to_ascii_lowercase();
    assert!(head.starts_with("get /load.png http/1.1"), "{head}");
    for leaked in [
        "cookie:",
        "referer:",
        "not-forwarded",
        "secret-path",
        "mozilla",
        "reader browser",
    ] {
        assert!(
            !head.contains(leaked),
            "{leaked} reached the origin: {head}"
        );
    }
    assert!(head.contains("user-agent: mailwoman-image-proxy"), "{head}");
}

#[tokio::test]
async fn a_tampered_retargeted_or_expired_token_is_refused_before_any_fetch() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture(proxy, "mw-t32-rc-forge").await;
    f.grant().await;
    let (_, signed) = f.sign(&["http://1.2.3.4/forge.png"], &[]).await;
    let token = token_of(&image_src(&signed));
    let claims = claims_of(&token);

    // This file's own key derivation reproduces the server's tag for the server's
    // payload — so the forgeries below differ from a valid token only in what they
    // change. (The payload is compared as bytes: a re-serialized JSON object may
    // order its keys differently, which the server accepts but which is not the
    // same string.)
    let (payload, tag) = token.split_once('.').unwrap();
    assert_eq!(
        URL_SAFE_NO_PAD.encode(hmac(
            &signing_key(),
            &[b"mw-rc-token\0", payload.as_bytes()]
        )),
        tag,
        "key derivation and token format"
    );
    // And a token this file mints for the same claims is accepted (positive
    // control for the forged ones below).
    let remint = mint(&claims);

    let mut retarget = claims.clone();
    retarget["u"] = json!("http://169.254.169.254/latest/meta-data/");
    let retargeted = format!(
        "{}.{tag}",
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&retarget).unwrap())
    );
    let mut expired = claims.clone();
    expired["x"] = json!(chrono::Utc::now().timestamp() - 10);
    let mut wrong_session = claims.clone();
    wrong_session["s"] = json!("00".repeat(16));
    let mut as_part = claims.clone();
    as_part["k"] = json!("p");

    let mut flipped = token.clone().into_bytes();
    let last = flipped.len() - 1;
    flipped[last] = if flipped[last] == b'A' { b'B' } else { b'A' };
    for (what, t) in [
        ("tampered tag", String::from_utf8(flipped).unwrap()),
        ("retargeted", retargeted),
        ("expired", mint(&expired)),
        ("another session", mint(&wrong_session)),
        ("junk", "abc.def".to_string()),
    ] {
        let r = f.load(&format!("/api/rc/i/{t}")).await;
        assert_eq!(r.status(), 403, "{what}");
    }
    // A valid image token is not a part token.
    let r = f.load(&format!("/api/rc/p/{token}")).await;
    assert_eq!(r.status(), 403, "kind is bound");
    let r = f.load(&format!("/api/rc/i/{}", mint(&as_part))).await;
    assert_eq!(r.status(), 403, "kind is bound");

    assert_eq!(
        seen.lock().unwrap().connections,
        0,
        "a refused token must not reach the network"
    );
    // Positive control: the untouched token does fetch, and so does this file's
    // re-mint of the same claims (served from the cache the first load filled).
    assert_eq!(f.load(&image_src(&signed)).await.status(), 200);
    assert_eq!(seen.lock().unwrap().connections, 1);
    assert_eq!(f.load(&format!("/api/rc/i/{remint}")).await.status(), 200);
}

#[tokio::test]
async fn reader_bridge_has_one_independent_policy_while_shell_stays_restricted() {
    let (proxy, _) = standin_proxy(vec![]).await;
    let f = fixture(proxy, "mw-t32-rc-frame-policy").await;
    let compatible = f
        .anon
        .get(format!("{}/api/rc/frame?coep=require-corp", f.base))
        .send()
        .await
        .unwrap();
    assert_eq!(compatible.status(), 200);
    assert_eq!(
        compatible.headers()["cross-origin-embedder-policy"],
        "require-corp"
    );
    assert_eq!(
        compatible.headers()["cross-origin-resource-policy"],
        "cross-origin"
    );
    assert_eq!(compatible.headers()["x-frame-options"], "SAMEORIGIN");
    let compatible_csp = compatible.headers()["content-security-policy"]
        .to_str()
        .unwrap();
    assert!(compatible_csp.contains("img-src 'self' data: blob:;"));
    assert!(!compatible_csp.contains("https:"));
    assert!(compatible_csp.contains("frame-ancestors 'self'"));
    let response = f
        .anon
        .get(format!("{}/api/rc/frame", f.base))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        response
            .headers()
            .get_all("content-security-policy")
            .iter()
            .count(),
        1
    );
    let csp = response.headers()["content-security-policy"]
        .to_str()
        .unwrap();
    assert!(csp.contains("script-src 'sha256-"));
    assert!(csp.contains("frame-ancestors 'self'"));
    assert!(!csp.contains("connect-src"));
    let csp = csp.to_owned();
    assert_eq!(response.headers()["x-frame-options"], "SAMEORIGIN");
    assert_eq!(
        response.headers()["cross-origin-embedder-policy"],
        "credentialless"
    );
    assert_eq!(
        response.headers()["cross-origin-resource-policy"],
        "cross-origin"
    );
    let html = response.text().await.unwrap();
    assert!(html.contains("mw-reader-ready"));
    let script = html
        .split_once("<script>")
        .unwrap()
        .1
        .split_once("</script>")
        .unwrap()
        .0;
    let normalized = script.replace("\r\n", "\n").replace('\r', "\n");
    let digest =
        base64::engine::general_purpose::STANDARD.encode(Sha256::digest(normalized.as_bytes()));
    assert!(
        csp.contains(&format!("'sha256-{digest}'")),
        "CSP hashes the HTML parser's normalized script bytes"
    );
    let shell = f.anon.get(format!("{}/", f.base)).send().await.unwrap();
    assert_eq!(shell.headers()["x-frame-options"], "DENY");
    assert_eq!(
        shell.headers()["cross-origin-embedder-policy"],
        "require-corp"
    );
    assert_eq!(
        shell.headers()["cross-origin-resource-policy"],
        "same-origin"
    );
    assert!(
        shell.headers()["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("img-src 'self' blob: data:")
    );
}

#[tokio::test]
async fn proxy_message_without_an_engine_cache_can_load_signed_images() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let mut f = fixture(proxy, "mw-t32-rc-cacheless-proxy").await;
    f.email_id = "e1".into();
    assert!(store_at(&f.db).await.get_message("e1").await.is_err());
    f.grant().await;
    let (status, signed) = f.sign(&["http://1.2.3.4/proxy.png"], &[]).await;
    assert_eq!(status, 200, "{signed}");
    assert_eq!(f.load(&image_src(&signed)).await.status(), 200);
    assert_eq!(seen.lock().unwrap().connections, 1);
}

#[tokio::test]
async fn privacy_modes_and_capabilities_are_hot_and_do_not_bypass_message_ownership() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture(proxy, "mw-t32-rc-policy").await;
    let store = store_at(&f.db).await;
    async fn setting(store: &Store, key: &str, value: Value) {
        store
            .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                key: key.into(),
                global_value: Some(value.to_string()),
                allow_user_override: true,
                global_locked: true,
                updated_by: "test".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
    }
    let url = "http://1.2.3.4/policy.png";
    assert_eq!(f.sign(&[url], &[]).await.0, 403);
    setting(&store, "privacy.remoteContent", json!("proxy")).await;
    let (status, signed) = f.sign(&[url], &[]).await;
    assert_eq!(status, 200, "{signed}");
    let src = image_src(&signed);
    let absent = f
        .user
        .post(format!("{}/api/rc/sign", f.base))
        .json(&json!({"emailId":"not-owned", "urls":[url]}))
        .send()
        .await
        .unwrap();
    assert_eq!(absent.status(), 403);
    setting(&store, "privacy.remoteContent", json!("block")).await;
    assert_eq!(f.load(&src).await.status(), 403);
    assert_eq!(seen.lock().unwrap().connections, 0);
    setting(&store, "privacy.remoteContent", json!("direct")).await;
    setting(&store, "privacy.allowDirect", json!(true)).await;
    let policy = f
        .user
        .get(format!("{}/api/rc/policy", f.base))
        .send()
        .await
        .unwrap();
    assert_eq!(policy.headers()["cache-control"], "no-store");
    let policy: Value = policy.json().await.unwrap();
    assert_eq!(policy["mode"], "ask");
    assert_eq!(policy["allowDirect"], false);
    assert_eq!(policy["locked"]["mode"], true);
    setting(&store, "cap.remote_content.direct", json!(true)).await;
    let policy: Value = f
        .user
        .get(format!("{}/api/rc/policy", f.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(policy["mode"], "direct");
    assert_eq!(policy["allowDirect"], true);
    assert_eq!(f.load(&src).await.status(), 200);
    setting(&store, "privacy.trackers", json!(false)).await;
    let (status, tracker) = f
        .sign(&["https://x.list-manage.com/track/open.php?u=1"], &[])
        .await;
    assert_eq!(status, 200);
    setting(&store, "privacy.trackers", json!(true)).await;
    assert_eq!(f.load(&image_src(&tracker)).await.status(), 403);
    assert_eq!(
        seen.lock().unwrap().connections,
        1,
        "new tracker policy must block before fetch"
    );
}

#[tokio::test]
async fn account_restrictions_revoke_already_signed_content_before_fetch() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture(proxy, "mw-t32-rc-account-policy").await;
    f.grant().await;
    let (status, signed) = f.sign(&["http://1.2.3.4/restricted.png"], &[]).await;
    assert_eq!(status, 200);
    let src = image_src(&signed);
    let store = store_at(&f.db).await;
    let key = format!("v6:admin:flags:{}", f.account.to_lowercase());
    for flags in [
        json!({"disabled":true}),
        json!({"force_password_change":true}),
    ] {
        store.set_setting(&key, &flags.to_string()).await.unwrap();
        assert_eq!(f.load(&src).await.status(), 403);
    }
    assert_eq!(seen.lock().unwrap().connections, 0);
    store.set_setting(&key, "{}").await.unwrap();
    assert_eq!(f.load(&src).await.status(), 200);
}

#[tokio::test]
async fn logging_out_or_revoking_the_grant_ends_the_tokens() {
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture(proxy, "mw-t32-rc-logout").await;
    f.grant().await;
    let (_, signed) = f
        .sign(
            &["http://1.2.3.4/out-a.png", "http://1.2.3.4/out-b.png"],
            &[],
        )
        .await;
    let a = signed["images"][0]["src"].as_str().unwrap().to_string();
    let b = signed["images"][1]["src"].as_str().unwrap().to_string();
    assert_eq!(f.load(&a).await.status(), 200);

    store_at(&f.db)
        .await
        .revoke_remote_image(&f.account, "all", "")
        .await
        .unwrap();
    assert_eq!(f.load(&b).await.status(), 403, "the grant was revoked");
    f.grant().await;
    assert_eq!(f.load(&b).await.status(), 200);

    let out = f
        .user
        .post(format!("{}/api/logout", f.base))
        .send()
        .await
        .unwrap();
    assert!(out.status().is_success(), "{}", out.status());
    // Served from the cache before logout; after logout even a cached URL is refused.
    assert_eq!(f.load(&a).await.status(), 403, "a token after logout");
    assert_eq!(seen.lock().unwrap().connections, 2);
}

#[tokio::test]
async fn only_allowed_image_types_are_served_and_a_tracking_pixel_is_replaced() {
    let (proxy, _) = standin_proxy(vec![
        (
            "image/svg+xml".into(),
            b"<svg xmlns='http://www.w3.org/2000/svg'/>".to_vec(),
        ),
        ("text/html".into(), png(4, 4)),
        ("image/gif".into(), png(4, 4)),
        ("image/png".into(), png(1, 1)),
    ])
    .await;
    let f = fixture(proxy, "mw-t32-rc-types").await;
    f.grant().await;
    let urls = [
        "http://1.2.3.4/types-a.svg",
        "http://1.2.3.4/types-b.png",
        "http://1.2.3.4/types-c.gif",
        "http://1.2.3.4/types-d.png",
    ];
    let (_, signed) = f.sign(&urls, &[]).await;
    let src = |i: usize| signed["images"][i]["src"].as_str().unwrap().to_string();

    let r = f.load(&src(0)).await;
    assert_eq!(r.status(), 415);
    assert_eq!(r.text().await.unwrap(), "SVG images are not loaded");
    let r = f.load(&src(1)).await;
    assert_eq!(r.status(), 415, "declared type not allowed");
    let r = f.load(&src(2)).await;
    assert_eq!(r.status(), 415);
    assert_eq!(
        r.text().await.unwrap(),
        "the image does not match its declared type"
    );

    let r = f.load(&src(3)).await;
    assert_eq!(r.status(), 200);
    assert_eq!(r.headers()["x-mw-remote-content"], "tracker");
    let body = r.bytes().await.unwrap();
    assert_eq!(png_size(&body), (1, 1));
    assert_eq!(
        body[25], 6,
        "the replacement is RGBA (transparent), not the origin's RGB pixel"
    );
    // Reloading the same decoded pixel from cache must not count it twice.
    assert_eq!(f.load(&src(3)).await.status(), 200);
    let stats: Value = f
        .user
        .get(format!("{}/api/rc/stats", f.base))
        .query(&[("emailId", &f.email_id)])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(stats["decodedTrackersReplaced"], 1);
    assert_eq!(stats["limited"], false);
    assert_eq!(stats["windowSecs"], 3600);
}

#[tokio::test]
async fn signing_refuses_trackers_and_non_http_urls_and_parts_load_without_a_cookie() {
    let (proxy, _) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let mut f = fixture(proxy, "mw-t32-rc-sign").await;
    f.email_id = "e1".into();
    f.grant().await;
    let (status, signed) = f
        .sign(
            &[
                "https://x.list-manage.com/track/open.php?u=1",
                "javascript:alert(1)",
                "http://1.2.3.4/sign-ok.png",
            ],
            &["Mblob1", "../etc", "other-message-blob"],
        )
        .await;
    assert_eq!(status, 200, "{signed}");
    let reasons: Vec<&str> = signed["refused"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["reason"].as_str().unwrap())
        .collect();
    assert_eq!(
        reasons,
        ["tracker", "scheme", "blobId", "notMessagePart"],
        "{signed}"
    );
    assert_eq!(signed["images"].as_array().unwrap().len(), 1);

    // The mock upstream serves a part as application/octet-stream text: the signed
    // part URL reaches it without a cookie (proxy mode, the session's credentials)
    // and is refused as not an image.
    let part = signed["parts"][0]["src"].as_str().unwrap().to_string();
    assert!(part.starts_with("/api/rc/p/"), "{part}");
    let r = f.load(&part).await;
    assert_eq!(r.status(), 415, "{}", r.text().await.unwrap());
    let claims = claims_of(&token_of(&part));
    let ttl = claims["x"].as_i64().unwrap() - chrono::Utc::now().timestamp();
    assert!(
        ttl <= 300 && ttl > 290,
        "part tokens live five minutes: {ttl}"
    );
    let img = claims_of(&token_of(signed["images"][0]["src"].as_str().unwrap()));
    let ttl = img["x"].as_i64().unwrap() - chrono::Utc::now().timestamp();
    assert!(
        ttl <= 3600 && ttl > 3590,
        "image tokens live an hour: {ttl}"
    );
}

#[tokio::test]
async fn a_linked_file_downloads_through_the_server_and_is_never_rendered() {
    let (proxy, seen) = standin_proxy(vec![(
        "text/html".into(),
        b"<script>window.__ran=1</script>".to_vec(),
    )])
    .await;
    let f = fixture(proxy, "mw-t32-rc-file").await;
    // No grant: a file download is a deliberate click on the reader's button.
    let (status, signed) = f
        .user
        .post(format!("{}/api/rc/sign", f.base))
        .json(&json!({ "emailId": "e1", "files": ["http://1.2.3.4/dl/report.html", "file:///etc/passwd"] }))
        .send()
        .await
        .map(|r| (r.status().as_u16(), r))
        .unwrap();
    assert_eq!(status, 200);
    let signed: Value = signed.json().await.unwrap();
    assert_eq!(signed["refused"][0]["reason"], "scheme", "{signed}");
    let src = signed["files"][0]["src"].as_str().unwrap().to_string();
    assert!(src.starts_with("/api/rc/f/"), "{src}");

    let r = f.load(&src).await;
    assert_eq!(r.status(), 200);
    let h = r.headers();
    assert_eq!(h["content-type"], "application/octet-stream");
    assert_eq!(
        h["content-disposition"],
        "attachment; filename=\"report.html\""
    );
    let csp: Vec<_> = h
        .get_all("content-security-policy")
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect();
    assert!(
        csp.iter().any(|v| v == "sandbox; default-src 'none'"),
        "{csp:?}"
    );
    assert_eq!(
        r.bytes().await.unwrap(),
        &b"<script>window.__ran=1</script>"[..]
    );
    assert_eq!(seen.lock().unwrap().connections, 1);

    // Another account's message cannot be used to sign a file.
    let r = f
        .user
        .post(format!("{}/api/rc/sign", f.base))
        .json(&json!({ "emailId": "no-such-message", "files": ["http://1.2.3.4/x.zip"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 403);
}

async fn locked_policy(f: &Fixture, key: &str, value: Value) {
    store_at(&f.db)
        .await
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: true,
            global_locked: true,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn legacy_image_decoder_and_cache_follow_tracker_and_type_policy() {
    let (proxy, seen) = standin_proxy(vec![
        ("image/png".into(), png(2, 2)),
        ("image/png".into(), png(2, 2)),
        ("text/html".into(), png(4, 4)),
    ])
    .await;
    let f = fixture(proxy, "mw-t32-rc-legacy-policy").await;
    f.grant().await;
    locked_policy(&f, "privacy.trackers", json!(false)).await;
    let url = format!("{}/api/image-proxy", f.base);
    let query = [
        ("url", "http://1.2.3.4/legacy-policy-pixel.png"),
        ("emailId", f.email_id.as_str()),
    ];
    let response = f.user.get(&url).query(&query).send().await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(png_size(&response.bytes().await.unwrap()), (2, 2));
    locked_policy(&f, "privacy.trackers", json!(true)).await;
    let response = f.user.get(&url).query(&query).send().await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["x-mw-remote-content"], "tracker");
    assert_eq!(png_size(&response.bytes().await.unwrap()), (1, 1));
    assert_eq!(seen.lock().unwrap().connections, 2);
    let response = f
        .user
        .get(&url)
        .query(&[
            ("url", "http://1.2.3.4/legacy-policy-type.png"),
            ("emailId", f.email_id.as_str()),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 415);
}

#[tokio::test]
async fn part_redemption_rechecks_membership_and_charges_rate_budget() {
    let (proxy, _) = standin_proxy(vec![]).await;
    let mut f = fixture(proxy, "mw-t32-rc-part-limit").await;
    f.email_id = "e1".into();
    locked_policy(&f, "privacy.ratePerMin", json!(10)).await;
    let (status, signed) = f.sign(&[], &["Mblob1"]).await;
    assert_eq!(status, 200, "{signed}");
    let src = signed["parts"][0]["src"].as_str().unwrap();
    let mut claims = claims_of(&token_of(src));
    claims["u"] = json!("another-message-blob");
    assert_eq!(
        f.load(&format!("/api/rc/p/{}", mint(&claims)))
            .await
            .status(),
        403
    );
    claims["u"] = json!("Mblob1");
    claims["e"] = json!("deleted-message");
    assert_eq!(
        f.load(&format!("/api/rc/p/{}", mint(&claims)))
            .await
            .status(),
        403
    );
    let mut refused = false;
    for _ in 0..12 {
        let status = f.load(src).await.status();
        if status == 429 {
            refused = true;
            break;
        }
        assert_eq!(status, 415, "bound part is served by mock as non-image");
    }
    assert!(
        refused,
        "part decoding must consume the account rate budget"
    );
}

#[tokio::test]
async fn domain_alias_inherits_hot_locks_direct_capability_and_scoped_egress_without_changing_login()
 {
    use mw_store::settings_policy::{SettingScope, SettingValueRow};
    let (proxy, seen) = standin_proxy(vec![("image/png".into(), png(4, 4))]).await;
    let f = fixture(proxy, "mw-t32-rc-alias").await;
    let store = store_at(&f.db).await;
    let before = store
        .sessions_by_account(&f.account)
        .await
        .unwrap()
        .remove(0);
    assert_eq!(before.username, mw_mock_jmap::USER);
    store
        .delete_domain_config("example.org", true)
        .await
        .unwrap();
    store
        .put_domain_config(&mw_store::DomainConfigRow {
            name: "canonical.test".into(),
            upstream_json: json!({"jmap":{"sessionUrl":before.jmap_url}}).to_string(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    store
        .put_domain_alias("example.org", "canonical.test")
        .await
        .unwrap();
    let value = |key: &str, value: Value| SettingValueRow {
        scope: SettingScope::Domain,
        scope_id: "canonical.test".into(),
        key: key.into(),
        value: value.to_string(),
        locked: true,
        updated_by: "root".into(),
        updated_at: 1,
    };
    store
        .put_setting_value(&value("privacy.remoteContent", json!("block")))
        .await
        .unwrap();
    let response = f
        .user
        .get(format!("{}/api/rc/policy", f.base))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let policy: Value = response.json().await.unwrap();
    assert_eq!(policy["mode"], "block");
    assert_eq!(policy["locked"]["mode"], true);
    // A personal settings write must name the session's storage authority (0a11b581,
    // 428 `storageAuthorityRequired` otherwise), so the refusal below is the lock's.
    let me: Value = f
        .user
        .get(format!("{}/api/me", f.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let write = f
        .user
        .put(format!("{}/api/settings/privacy.remoteContent", f.base))
        .header(
            "x-mailwoman-storage-authority",
            me["storageAuthority"]
                .as_str()
                .expect("/api/me names a storageAuthority"),
        )
        .json(&json!({"value":"direct"}))
        .send()
        .await
        .unwrap();
    assert_eq!(write.status(), 403, "alias cannot override canonical lock");
    let refusal: Value = write.json().await.unwrap();
    assert_eq!(refusal["code"], "setting_locked", "{refusal}");
    locked_policy(&f, "privacy.allowDirect", json!(true)).await;
    store
        .put_setting_value(&value("privacy.remoteContent", json!("direct")))
        .await
        .unwrap();
    store
        .put_setting_value(&value("cap.remote_content.direct", json!(false)))
        .await
        .unwrap();
    let policy: Value = f
        .user
        .get(format!("{}/api/rc/policy", f.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(policy["mode"], "ask");
    assert_eq!(policy["allowDirect"], false);
    store
        .put_setting_value(&value("privacy.remoteContent", json!("proxy")))
        .await
        .unwrap();
    let mut route = store.get_egress_proxy("rc").await.unwrap().unwrap();
    route.scope = "domain:canonical.test".into();
    store.put_egress_proxy(&route).await.unwrap();
    let (status, signed) = f.sign(&["http://1.2.3.4/alias-route.png"], &[]).await;
    assert_eq!(status, 200, "{signed}");
    assert_eq!(f.load(&image_src(&signed)).await.status(), 200);
    assert_eq!(
        seen.lock().unwrap().connections,
        1,
        "canonical scoped proxy must carry alias traffic"
    );
    // Configuration rows using the alias are resolved to the same scope as well.
    route.scope = "domain:example.org".into();
    store.put_egress_proxy(&route).await.unwrap();
    let (_, signed) = f
        .sign(&["http://1.2.3.4/alias-route-config.png"], &[])
        .await;
    assert_eq!(f.load(&image_src(&signed)).await.status(), 200);
    assert_eq!(seen.lock().unwrap().connections, 2);
    let after = store
        .sessions_by_account(&f.account)
        .await
        .unwrap()
        .remove(0);
    assert_eq!(after.username, before.username);
    assert_eq!(after.account_id, before.account_id);
    assert_eq!(after.credentials.username, before.credentials.username);
}

#[tokio::test]
async fn message_tracker_stats_require_session_and_owned_message_and_start_at_zero() {
    let (proxy, _) = standin_proxy(vec![]).await;
    let f = fixture(proxy, "mw-t32-rc-stats-auth").await;
    let store = store_at(&f.db).await;
    let other = store
        .create_account(
            &mw_store::NewAccount {
                kind: mw_store::AccountKind::Imap,
                host: "other.test",
                port: 993,
                tls: "implicit",
                username: "other@other.test",
                sync_policy_json: "{}",
            },
            &mw_store::Credentials {
                username: "other@other.test".into(),
                password: "test".into(),
            },
        )
        .await
        .unwrap();
    let mailbox = store
        .upsert_mailbox(&mw_store::MailboxUpsert {
            account_id: &other,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 1,
            uidnext: 2,
            highestmodseq: 0,
            total: 1,
            unread: 1,
            parent_id: None,
        })
        .await
        .unwrap();
    let foreign = store
        .upsert_message(&mw_store::MessageUpsert {
            account_id: &other,
            mailbox_id: &mailbox,
            uid: 1,
            uidvalidity: 1,
            message_id: None,
            thread_id: None,
            internaldate: None,
            size: 1,
            flags_json: "[]",
            envelope: None,
            blob_ref: None,
        })
        .await
        .unwrap();
    let url = format!("{}/api/rc/stats", f.base);
    assert_eq!(
        f.anon
            .get(&url)
            .query(&[("emailId", &f.email_id)])
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        f.user
            .get(&url)
            .query(&[("emailId", foreign.as_str())])
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    let response = f
        .user
        .get(&url)
        .query(&[("emailId", &f.email_id)])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert!(
        response.headers()["cache-control"]
            .to_str()
            .unwrap()
            .contains("no-store")
    );
    let stats: Value = response.json().await.unwrap();
    assert_eq!(
        stats,
        json!({"decodedTrackersReplaced":0,"limited":false,"windowSecs":3600})
    );
}

#[tokio::test]
async fn manual_image_downloads_require_policy_and_real_message_membership_at_every_use() {
    let (proxy, seen) = standin_proxy(vec![
        ("image/png".into(), png(4, 4)),
        ("image/svg+xml".into(), b"<svg/>".to_vec()),
    ])
    .await;
    let f = fixture(proxy, "mw-rc-image-download").await;
    let endpoint = format!("{}/api/rc/sign", f.base);
    let body = json!({"emailId":"e1","imageDownloads":["http://1.2.3.4/download-image.png","http://1.2.3.4/not-in-message.png","http://1.2.3.4/declared-pixel.png","http://1.2.3.4/download-svg.svg"]});
    assert_eq!(
        f.user
            .post(&endpoint)
            .header("x-mailwoman-account", &f.account)
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(seen.lock().unwrap().connections, 0);
    async fn enable(f: &Fixture, value: bool) {
        let url = format!("{}/admin/settings/privacy.remoteImageDownloads", f.base);
        let current = f.admin.get(&url).send().await.unwrap();
        let tag = current.headers()["etag"].to_str().unwrap().to_owned();
        let response = f
            .admin
            .put(url)
            .header("if-match", tag)
            .json(&json!({"globalValue":value}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    }
    enable(&f, true).await;
    assert_eq!(
        f.user
            .post(&endpoint)
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    assert_eq!(
        f.user
            .post(&endpoint)
            .header("x-mailwoman-account", "foreign")
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    let response = f
        .user
        .post(&endpoint)
        .header("x-mailwoman-account", &f.account)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let signed: Value = response.json().await.unwrap();
    assert_eq!(
        signed["imageDownloads"].as_array().unwrap().len(),
        2,
        "{signed}"
    );
    assert_eq!(signed["refused"][0]["reason"], "message");
    assert_eq!(signed["refused"][1]["reason"], "tracker");
    let image = signed["imageDownloads"][0]["src"].as_str().unwrap();
    assert_eq!(
        f.load(&image.replace("/api/rc/d/", "/api/rc/i/"))
            .await
            .status(),
        403
    );
    assert_eq!(
        f.load(image).await.status(),
        401,
        "manual download requires the current authenticated account"
    );
    assert_eq!(
        f.user
            .get(format!("{}{image}", f.base))
            .header("x-mailwoman-account", "foreign")
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    let loaded = f
        .user
        .get(format!("{}{image}", f.base))
        .send()
        .await
        .unwrap();
    assert_eq!(loaded.status(), 200);
    assert_eq!(
        loaded.headers()["content-disposition"],
        "attachment; filename=\"message-image.png\""
    );
    assert_eq!(loaded.headers()["content-type"], "image/png");
    assert!(
        loaded
            .bytes()
            .await
            .unwrap()
            .starts_with(b"\x89PNG\r\n\x1a\n")
    );
    assert_eq!(seen.lock().unwrap().connections, 1);
    // Even another valid session with the same provider-local account ID cannot
    // redeem a token minted for this browser's authority.
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let other = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();
    let login = other
        .post(format!("{}/api/login", f.base))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "{}", login.text().await.unwrap());
    assert_eq!(
        other
            .get(format!("{}{image}", f.base))
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    assert_eq!(seen.lock().unwrap().connections, 1);
    let file = f
        .user
        .post(&endpoint)
        .json(&json!({"emailId":"e1","files":["http://1.2.3.4/not-in-message.zip"]}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(file["files"].as_array().unwrap().len(), 0);
    assert_eq!(file["refused"][0]["reason"], "message");
    assert_eq!(seen.lock().unwrap().connections, 1);
    let svg = signed["imageDownloads"][1]["src"].as_str().unwrap();
    assert_eq!(
        f.user
            .get(format!("{}{svg}", f.base))
            .send()
            .await
            .unwrap()
            .status(),
        415
    );
    assert_eq!(seen.lock().unwrap().connections, 2);
    enable(&f, false).await;
    assert_eq!(
        f.user
            .get(format!("{}{image}", f.base))
            .send()
            .await
            .unwrap()
            .status(),
        403,
        "cached image cannot bypass the current download policy"
    );
    assert_eq!(seen.lock().unwrap().connections, 2);
    enable(&f, true).await;
    // The signing secret is available only to this test: forge a valid token for
    // an absent URL, proving redemption rechecks membership rather than trusting it.
    let mut claims = claims_of(&token_of(image));
    claims["u"] = json!("http://1.2.3.4/retargeted.png");
    let forged = mint(&claims);
    assert_eq!(
        f.user
            .get(format!("{}/api/rc/d/{forged}", f.base))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(seen.lock().unwrap().connections, 2);
    claims["k"] = json!("f");
    let forged_file = mint(&claims);
    assert_eq!(
        f.load(&format!("/api/rc/f/{forged_file}")).await.status(),
        403,
        "linked files also recheck exact message membership at redemption"
    );
    assert_eq!(seen.lock().unwrap().connections, 2);
}
