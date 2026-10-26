//! 26.20 t29-e9 — the read-receipt policy route, and that the engine's `MDN/send`
//! reads what the route stores.
//!
//! `GET|PUT /api/account/mdn-policy` keeps one word per account (`ask`, `never`,
//! `always`). The only reader is the engine: `MDN/send { automatic: true }` is
//! refused unless the word is `always`. A route that stored the word somewhere the
//! engine does not look would pass a round-trip test and change nothing, so the
//! first case sets the policy through the route and observes the engine's answer
//! change, over HTTP, for a message imported over HTTP.
//!
//! Every refusal is preceded, in the same server and for the same session, by the
//! same request being answered.
//!
//! Legs:
//!   * engine mode: default, validation, storage key, the engine reading it, no
//!     session
//!   * proxy mode: the route is 404 and stores nothing
//!   * a disabled account, and an account held for a password change, reach
//!     neither the route nor `MDN/send`
//!
//! Run:
//!   cargo test -p mw-server --test t29_mdn_policy --locked -- --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde_json::{Value, json};
use tokio::sync::{Mutex, MutexGuard};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::{ServerKey, Store};

mod common;
use common::mock_imap::MockImap;
use common::test_db;

const KEY_HEX: &str = "29e9a1b2c3d4e5f60718293a4b5c6d7e29e9a1b2c3d4e5f60718293a4b5c6d7e";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T29</div>";

/// The engine-mode mailbox the scripted IMAP server accepts.
const ENGINE_USER: &str = "owner@example.org";
const ENGINE_PASS: &str = "Old-Passw0rd!";

const ANN: &str = "ann@example.org";

/// A received message that asks for a receipt at its own return path.
const ASKING: &str = "Return-Path: <ann@example.org>\r\n\
From: Ann <ann@example.org>\r\n\
To: owner@example.org\r\n\
Subject: lunch\r\n\
Message-ID: <lunch-1@example.org>\r\n\
Date: Mon, 05 Oct 2026 10:00:00 +0000\r\n\
Disposition-Notification-To: ann@example.org\r\n\
\r\n\
at noon?\r\n";

// ── harness ──────────────────────────────────────────────────────────────────

/// Serialises the tests in this binary: they set process environment
/// (`MW_ENGINE_TLS`, `MW_UPLOAD_DIR`) that the server reads when it is built and
/// when it dials the mail server.
async fn serial() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK.get_or_init(Default::default).lock().await;
    let uploads = test_db::unique_dir("mw-t29-e9-uploads");
    // SAFETY: the guard serialises every mutation and every dependent read here.
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
        std::env::set_var("MW_UPLOAD_DIR", &uploads);
        for k in [
            "MW_HEADER_AUTH",
            "MW_LDAP_BIND_AUTH",
            "MW_PASSWD_BACKEND",
            "MW_MCP_RESOURCE",
        ] {
            std::env::remove_var(k);
        }
    }
    guard
}

struct Server {
    base: String,
    db: String,
}

impl Server {
    /// A second handle on the server's database, to read what a route stored.
    async fn store(&self) -> Store {
        Store::open(&self.db, ServerKey::from_hex(KEY_HEX).unwrap())
            .await
            .expect("open the server's store")
    }
}

/// A server in `mode` whose configured upstream (`MW_UPSTREAM_DEFAULT`) is
/// `upstream`: the JMAP mock in proxy mode, the IMAP mock in engine mode. A login
/// no longer names one (t32-h9).
async fn spawn(mode: ServerMode, upstream: String) -> Server {
    let dir = test_db::unique_dir("mw-t29-e9");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    let fixture_store = Store::open(&db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    fixture_store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "mdn-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    let config = AppConfig {
        db_path: db.clone(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
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
    let app = mw_server::build_app_full(config, v6)
        .await
        .expect("build_app_full")
        .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
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
        db,
    }
}

fn browser() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// Log in to an engine-mode server; returns the session's account id. `_pop` is
/// the server's configured upstream; the login does not name it.
async fn engine_login(c: &reqwest::Client, base: &str, _pop: &MockImap) -> String {
    let r = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": ENGINE_USER, "password": ENGINE_PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "engine login");
    r.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .expect("the login names the account")
        .to_string()
}

/// A logged-in admin-panel client.
async fn admin(base: &str) -> reqwest::Client {
    let c = browser();
    let r = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "admin login");
    c
}

/// Set an account's flags the way the panel does.
async fn set_flags(admin: &reqwest::Client, base: &str, id: &str, disabled: bool, force: bool) {
    let r = admin
        .put(format!("{base}/admin/users/{id}/flags"))
        .json(&json!({
            "zeroAccess": false,
            "forcePasswordChange": force,
            "remoteCacheWipe": false,
            "disabled": disabled,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204, "the panel's flag write is accepted");
}

async fn get_policy(c: &reqwest::Client, base: &str) -> reqwest::Response {
    c.get(format!("{base}/api/account/mdn-policy"))
        .send()
        .await
        .unwrap()
}

async fn put_policy(c: &reqwest::Client, base: &str, body: Value) -> reqwest::Response {
    c.put(format!("{base}/api/account/mdn-policy"))
        .json(&body)
        .send()
        .await
        .unwrap()
}

/// The policy as the route reports it.
async fn policy(c: &reqwest::Client, base: &str) -> Value {
    let r = get_policy(c, base).await;
    assert_eq!(r.status(), 200, "GET the policy");
    r.json().await.unwrap()
}

/// One JMAP method call; the raw HTTP response.
async fn jmap_call(
    c: &reqwest::Client,
    base: &str,
    method: &str,
    args: Value,
) -> reqwest::Response {
    c.post(format!("{base}/jmap/api"))
        .json(&json!({
            "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            "methodCalls": [[method, args, "c"]]
        }))
        .send()
        .await
        .unwrap()
}

/// One JMAP method call that must be answered; the method's result.
async fn jmap(c: &reqwest::Client, base: &str, method: &str, args: Value) -> Value {
    let r = jmap_call(c, base, method, args).await;
    assert_eq!(r.status(), 200, "{method} reaches the engine");
    let body: Value = r.json().await.unwrap();
    body["methodResponses"][0][1].clone()
}

/// Upload [`ASKING`] and import it into the account's inbox, over HTTP. Returns
/// the Email id.
async fn import_asking(c: &reqwest::Client, base: &str, account: &str) -> String {
    let mailboxes = jmap(c, base, "Mailbox/get", json!({ "accountId": account })).await;
    let inbox = mailboxes["list"]
        .as_array()
        .and_then(|list| {
            list.iter().find(|m| {
                m["role"]
                    .as_str()
                    .is_some_and(|r| r.eq_ignore_ascii_case("inbox"))
            })
        })
        .unwrap_or_else(|| panic!("an inbox: {mailboxes}"))["id"]
        .as_str()
        .unwrap()
        .to_string();
    let r = c
        .post(format!("{base}/jmap/upload/{account}"))
        .header(reqwest::header::CONTENT_TYPE, "message/rfc822")
        .body(ASKING)
        .send()
        .await
        .unwrap();
    let status = r.status();
    let uploaded: Value = r.json().await.unwrap();
    assert_eq!(status, 200, "upload: {uploaded}");
    let imported = jmap(
        c,
        base,
        "Email/import",
        json!({ "accountId": account, "emails": { "i": {
            "blobId": uploaded["blobId"], "mailboxIds": { inbox: true }
        } } }),
    )
    .await;
    imported["created"]["i"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("the message was imported: {imported}"))
        .to_string()
}

async fn receipt_state(c: &reqwest::Client, base: &str, id: &str) -> Value {
    let got = jmap(
        c,
        base,
        "Email/get",
        json!({ "ids": [id], "properties": ["mailwomanMdn"] }),
    )
    .await;
    got["list"][0]["mailwomanMdn"].clone()
}

// ── tests ────────────────────────────────────────────────────────────────────

/// The route stores the policy where the engine reads it: an automatic receipt
/// is refused under the default, and gets past that refusal only once the route
/// has stored `always`.
#[tokio::test]
async fn the_engine_reads_the_policy_the_route_stores() {
    let _g = serial().await;
    let pop = MockImap::start(ENGINE_USER, ENGINE_PASS).await;
    let srv = spawn(ServerMode::Engine, pop.url()).await;
    let base = &srv.base;
    let store = srv.store().await;
    let c = browser();
    let account = engine_login(&c, base, &pop).await;
    let key = mw_engine::Engine::mdn_policy_key(&account);

    // Default: nothing stored reads as `ask`.
    assert_eq!(
        policy(&c, base).await,
        json!({ "policy": "ask", "locked": false, "allowedModes": mw_engine::Engine::MDN_POLICIES })
    );
    assert_eq!(store.get_setting(&key).await.unwrap(), None);

    let id = import_asking(&c, base, &account).await;
    assert_eq!(
        receipt_state(&c, base, &id).await,
        json!({ "requestedBy": ANN, "sameAsSender": true, "fromList": false, "sent": false })
    );
    let automatic = json!({ "emailId": id, "automatic": true });
    let refused = jmap(&c, base, "MDN/send", automatic.clone()).await;
    assert_eq!(
        refused["type"], "mdnAutomaticRefused",
        "under ask: {refused}"
    );

    // Only the three words are stored.
    for bad in [json!({ "policy": "sometimes" }), json!({ "policy": "" })] {
        let r = put_policy(&c, base, bad.clone()).await;
        assert_eq!(r.status(), 400, "{bad}");
    }
    for malformed in [json!({ "policy": 1 }), json!({}), json!("always")] {
        let r = put_policy(&c, base, malformed.clone()).await;
        assert!(r.status().is_client_error(), "{malformed}: {}", r.status());
    }
    assert_eq!(
        store.get_setting(&key).await.unwrap(),
        None,
        "nothing stored"
    );

    // `never` is stored, and is not `always`.
    let r = put_policy(&c, base, json!({ "policy": "never" })).await;
    assert_eq!(r.status(), 200);
    assert_eq!(
        policy(&c, base).await,
        json!({ "policy": "never", "locked": false, "allowedModes": mw_engine::Engine::MDN_POLICIES })
    );
    let refused = jmap(&c, base, "MDN/send", automatic.clone()).await;
    assert_eq!(
        refused["type"], "mdnAutomaticRefused",
        "under never: {refused}"
    );

    let r = put_policy(&c, base, json!({ "policy": "always" })).await;
    assert_eq!(r.status(), 200);
    assert_eq!(
        r.json::<Value>().await.unwrap(),
        json!({ "ok": true, "policy": "always" })
    );
    assert_eq!(
        policy(&c, base).await,
        json!({ "policy": "always", "locked": false, "allowedModes": mw_engine::Engine::MDN_POLICIES })
    );
    assert_eq!(
        store.get_setting(&key).await.unwrap().as_deref(),
        Some("always"),
        "stored under the key the engine reads"
    );

    // The engine now lets the automatic receipt through to the submitter. This
    // account has no reachable SMTP server, so the submission itself fails —
    // which is past the policy check. Transport errors conservatively retain
    // the once-only claim because this interface cannot prove non-delivery.
    let attempted = jmap(&c, base, "MDN/send", automatic.clone()).await;
    assert_eq!(attempted["type"], "serverFail", "under always: {attempted}");
    assert_eq!(receipt_state(&c, base, &id).await["sent"], true);
    assert_eq!(jmap(&c, base, "MDN/send", automatic).await["type"], "mdnAlreadySent");

    // Without a session neither verb is answered, and nothing changes.
    let anonymous = browser();
    assert_eq!(get_policy(&anonymous, base).await.status(), 401);
    let r = put_policy(&anonymous, base, json!({ "policy": "ask" })).await;
    assert_eq!(r.status(), 401);
    assert_eq!(
        store.get_setting(&key).await.unwrap().as_deref(),
        Some("always")
    );
}

/// A proxy-mode server has no engine to read the policy: the route says so and
/// stores nothing.
#[tokio::test]
async fn the_policy_route_is_not_offered_in_proxy_mode() {
    let _g = serial().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    let srv = spawn(ServerMode::Proxy, mock.clone()).await;
    let base = &srv.base;
    let c = browser();
    let r = c
        .post(format!("{base}/api/login"))
        .json(&json!({
            "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "proxy login");
    let account = r.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .unwrap()
        .to_string();
    // Control: the session reaches another route of the same router.
    let r = c
        .get(format!("{base}/api/account/signatures"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "control: a preferences route answers");

    assert_eq!(get_policy(&c, base).await.status(), 404);
    let r = put_policy(&c, base, json!({ "policy": "always" })).await;
    assert_eq!(r.status(), 404);
    let stored = srv
        .store()
        .await
        .get_setting(&mw_engine::Engine::mdn_policy_key(&account))
        .await
        .unwrap();
    assert_eq!(stored, None, "nothing stored");
}

/// A disabled account's session reaches neither the policy nor `MDN/send`.
#[tokio::test]
async fn a_disabled_account_cannot_set_the_policy_or_send_a_receipt() {
    let _g = serial().await;
    let pop = MockImap::start(ENGINE_USER, ENGINE_PASS).await;
    let srv = spawn(ServerMode::Engine, pop.url()).await;
    let base = &srv.base;
    let admin = admin(base).await;
    let c = browser();
    let account = engine_login(&c, base, &pop).await;
    let id = import_asking(&c, base, &account).await;
    let send = json!({ "emailId": id, "automatic": true });

    // Controls: both are answered for this session.
    let r = put_policy(&c, base, json!({ "policy": "never" })).await;
    assert_eq!(r.status(), 200, "control: PUT");
    assert_eq!(
        policy(&c, base).await,
        json!({ "policy": "never", "locked": false, "allowedModes": mw_engine::Engine::MDN_POLICIES })
    );
    let refused = jmap(&c, base, "MDN/send", send.clone()).await;
    assert_eq!(refused["type"], "mdnAutomaticRefused", "control: {refused}");

    set_flags(&admin, base, ENGINE_USER, true, false).await;

    assert_eq!(get_policy(&c, base).await.status(), 401);
    let r = put_policy(&c, base, json!({ "policy": "always" })).await;
    assert_eq!(r.status(), 401);
    assert_eq!(jmap_call(&c, base, "MDN/send", send).await.status(), 401);
    let stored = srv
        .store()
        .await
        .get_setting(&mw_engine::Engine::mdn_policy_key(&account))
        .await
        .unwrap();
    assert_eq!(
        stored.as_deref(),
        Some("never"),
        "the refused PUT stored nothing"
    );
}

/// An account held for a password change reaches neither, and is told why.
#[tokio::test]
async fn a_held_account_cannot_set_the_policy_or_send_a_receipt() {
    let _g = serial().await;
    let pop = MockImap::start(ENGINE_USER, ENGINE_PASS).await;
    let srv = spawn(ServerMode::Engine, pop.url()).await;
    let base = &srv.base;
    let admin = admin(base).await;
    let c = browser();
    let account = engine_login(&c, base, &pop).await;
    let id = import_asking(&c, base, &account).await;
    let send = json!({ "emailId": id, "automatic": true });

    let r = put_policy(&c, base, json!({ "policy": "never" })).await;
    assert_eq!(r.status(), 200, "control: PUT");
    let refused = jmap(&c, base, "MDN/send", send.clone()).await;
    assert_eq!(refused["type"], "mdnAutomaticRefused", "control: {refused}");

    set_flags(&admin, base, ENGINE_USER, false, true).await;

    let held = json!({ "error": "password change required", "passwordChangeRequired": true });
    for (what, resp) in [
        ("GET the policy", get_policy(&c, base).await),
        (
            "PUT the policy",
            put_policy(&c, base, json!({ "policy": "always" })).await,
        ),
        ("MDN/send", jmap_call(&c, base, "MDN/send", send).await),
    ] {
        assert_eq!(resp.status(), 403, "{what} is held");
        assert_eq!(resp.json::<Value>().await.unwrap(), held, "{what}");
    }
    let stored = srv
        .store()
        .await
        .get_setting(&mw_engine::Engine::mdn_policy_key(&account))
        .await
        .unwrap();
    assert_eq!(
        stored.as_deref(),
        Some("never"),
        "the held PUT stored nothing"
    );
}

#[tokio::test]
async fn sender_rules_require_authority_current_revision_and_real_account() {
    let _g = serial().await;
    let imap = MockImap::start(ENGINE_USER, ENGINE_PASS).await;
    let srv = spawn(ServerMode::Engine, imap.url()).await;
    let client = browser();
    let account = engine_login(&client, &srv.base, &imap).await;
    let url = format!("{}/api/account/mdn-senders", srv.base);
    assert_eq!(browser().get(&url).send().await.unwrap().status(),401);
    let first=client.get(&url).send().await.unwrap();assert_eq!(first.status(),200);
    let etag=first.headers()["etag"].to_str().unwrap().to_owned();
    assert_eq!(first.json::<Value>().await.unwrap()["rules"],json!([]));
    let body=json!({"address":ANN,"decision":"always"});
    assert_eq!(client.put(&url).json(&body).send().await.unwrap().status(),428);
    let response=client.put(&url).header("if-match",&etag).json(&body).send().await.unwrap();assert_eq!(response.status(),200);
    assert_eq!(client.put(&url).header("if-match",&etag).json(&json!({"address":ANN,"decision":null})).send().await.unwrap().status(),412);
    let current=client.get(&url).send().await.unwrap();let etag=current.headers()["etag"].to_str().unwrap().to_owned();
    assert_eq!(current.json::<Value>().await.unwrap()["rules"][0]["address"],ANN);
    let store=srv.store().await;
    assert_eq!(store.receipt_sender_decision(&account,ANN).await.unwrap(),Some(mw_store::receipts::SenderDecision::Always));
    store.put_setting_policy(&mw_store::settings_policy::SettingPolicyRow{key:"mail.readReceipts".into(),global_value:Some("\"ask\"".into()),allow_user_override:false,global_locked:true,updated_by:"fixture".into(),updated_at:0}).await.unwrap();
    assert_eq!(client.put(&url).header("if-match",&etag).json(&body).send().await.unwrap().status(),403);
    assert_eq!(client.get(format!("{}/api/account/mdn-decision/unknown",srv.base)).send().await.unwrap().status(),404);
}

#[tokio::test]
async fn rfc9007_authenticated_http_sends_once_with_null_envelope_and_parses_report() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let _guard = serial().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let smtp = tokio::spawn(async move {
        tokio::time::timeout(std::time::Duration::from_secs(30), async move {
            loop {
            let (stream, _) = listener.accept().await.unwrap();
            let (read, mut write) = stream.into_split();
            let mut read = BufReader::new(read);
            write.write_all(b"220 fixture ready\r\n").await.unwrap();
            let mut envelope = false;
            let mut recipient = false;
            let mut data = Vec::new();
            loop {
                let mut line = String::new();
                assert!(read.read_line(&mut line).await.unwrap() > 0);
                assert!(line.len() < 8192);
                let command = line.trim_end();
                if command.starts_with("EHLO") {
                    write
                        .write_all(b"250-fixture\r\n250 AUTH PLAIN\r\n")
                        .await
                        .unwrap();
                } else if command.starts_with("AUTH PLAIN") {
                    write.write_all(b"235 authenticated\r\n").await.unwrap();
                } else if command.starts_with("MAIL FROM:") {
                    assert_eq!(command, "MAIL FROM:<>");
                    envelope = true;
                    write.write_all(b"250 sender accepted\r\n").await.unwrap();
                } else if command.starts_with("RCPT TO:") {
                    assert!(envelope);
                    assert_eq!(command, "RCPT TO:<ann@example.org>");
                    recipient = true;
                    write
                        .write_all(b"250 recipient accepted\r\n")
                        .await
                        .unwrap();
                } else if command == "DATA" {
                    assert!(recipient);
                    write.write_all(b"354 send data\r\n").await.unwrap();
                    loop {
                        let mut line = Vec::new();
                        assert!(read.read_until(b'\n', &mut line).await.unwrap() > 0);
                        if line == b".\r\n" {
                            break;
                        }
                        if line.starts_with(b"..") {
                            line.remove(0);
                        }
                        data.extend(line);
                        assert!(data.len() <= 9 * 1024 * 1024);
                    }
                    write.write_all(b"250 queued\r\n").await.unwrap();
                } else if command == "QUIT" {
                    write.write_all(b"221 bye\r\n").await.unwrap();
                    break;
                } else {
                    panic!("unexpected fixture command");
                }
            }
            if !data.is_empty() { break data; }
            }
        })
        .await
        .unwrap()
    });
    // SAFETY: all tests in this binary hold serial() across environment use.
    unsafe {
        std::env::set_var("MW_SMTP_HOST", "127.0.0.1");
        std::env::set_var("MW_SMTP_PORT", port.to_string());
        std::env::set_var("MW_SMTP_SECURITY", "plaintext");
    }
    let imap = MockImap::start(ENGINE_USER, ENGINE_PASS).await;
    let server = spawn(ServerMode::Engine, imap.url()).await;
    let client = browser();
    let account = engine_login(&client, &server.base, &imap).await;
    let session: Value = client.get(format!("{}/jmap/session",server.base))
        .send().await.unwrap().json().await.unwrap();
    assert_eq!(session["capabilities"][mw_engine::Engine::STANDARD_MDN_CAPABILITY],json!({}));
    assert_eq!(session["accounts"][&account]["accountCapabilities"][mw_engine::Engine::STANDARD_MDN_CAPABILITY],json!({}));
    let id = import_asking(&client, &server.base, &account).await;
    let identities = jmap(
        &client,
        &server.base,
        "Identity/get",
        json!({"accountId":account}),
    )
    .await;
    let identity = identities["list"][0]["id"].as_str().unwrap();
    let request = json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail",mw_engine::Engine::STANDARD_MDN_CAPABILITY],"methodCalls":[["MDN/send",{"accountId":account,"identityId":identity,"send":{"receipt":{"forEmailId":id,"subject":"HTTP standard receipt","textBody":"Displayed by client","includeOriginalMessage":true,"disposition":{"actionMode":"manual-action","sendingMode":"mdn-sent-manually","type":"displayed"}}},"onSuccessUpdateEmail":{"#receipt":{"keywords/$mdnsent":true}}},"s"]]});
    let anonymous = browser()
        .post(format!("{}/jmap/api", server.base))
        .json(&request)
        .send()
        .await
        .unwrap();
    assert_eq!(anonymous.status(), 401);
    let response = client
        .post(format!("{}/jmap/api", server.base))
        .json(&request)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let response: Value = response.json().await.unwrap();
    assert_eq!(response["methodResponses"][0][0], "MDN/send");
    assert!(
        response["methodResponses"][0][1]["sent"]["receipt"].is_object(),
        "{response}"
    );
    assert_eq!(response["methodResponses"][1][0], "Email/set");
    let raw = smtp.await.unwrap();
    let wire = String::from_utf8_lossy(&raw);
    assert!(wire.contains("HTTP standard receipt"));
    assert!(wire.contains("message/rfc822"));
    let duplicate: Value = client
        .post(format!("{}/jmap/api", server.base))
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        duplicate["methodResponses"][0][1]["notSent"]["receipt"]["type"],
        "mdnAlreadySent"
    );
    let upload: Value = client
        .post(format!("{}/jmap/upload/{account}", server.base))
        .header("content-type", "message/rfc822")
        .body(raw)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let blob = upload["blobId"].as_str().unwrap();
    let parsed:Value=client.post(format!("{}/jmap/api",server.base)).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail",mw_engine::Engine::STANDARD_MDN_CAPABILITY],"methodCalls":[["MDN/parse",{"accountId":account,"blobIds":[blob]},"p"]]})).send().await.unwrap().json().await.unwrap();
    assert_eq!(parsed["methodResponses"][0][0], "MDN/parse");
    assert_eq!(
        parsed["methodResponses"][0][1]["parsed"][blob]["subject"],
        "HTTP standard receipt"
    );
    unsafe {
        std::env::remove_var("MW_SMTP_HOST");
        std::env::remove_var("MW_SMTP_PORT");
        std::env::remove_var("MW_SMTP_SECURITY");
    }
}

#[tokio::test]
async fn dsn_http_capability_probe_is_truthful_and_changed_support_never_sends_plain() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let _guard=serial().await;
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port=listener.local_addr().unwrap().port();
    let smtp=tokio::spawn(async move {
        tokio::time::timeout(std::time::Duration::from_secs(30),async move {
            let mut commands=Vec::new();
            for attempt in 0..2 {
                let (stream,_)=listener.accept().await.unwrap();
                let (read,mut write)=stream.into_split(); let mut read=BufReader::new(read);
                write.write_all(b"220 fixture ready\r\n").await.unwrap();
                loop {
                    let mut line=String::new();
                    if read.read_line(&mut line).await.unwrap()==0 {break;}
                    assert!(line.len()<8192); let command=line.trim_end();
                    commands.push(command.to_owned());
                    if command.starts_with("EHLO") {
                        write.write_all(if attempt==0 {b"250-fixture\r\n250-DSN\r\n250 AUTH PLAIN\r\n"} else {b"250-fixture\r\n250 AUTH PLAIN\r\n"}).await.unwrap();
                    } else if command.starts_with("AUTH PLAIN") {write.write_all(b"235 authenticated\r\n").await.unwrap();}
                    else if command=="QUIT" {write.write_all(b"221 bye\r\n").await.unwrap();break;}
                    else {panic!("unsupported DSN must stop before any message command");}
                }
            }
            commands
        }).await.unwrap()
    });
    // SAFETY: serialized environment fixture, no parallel mutation.
    unsafe { std::env::set_var("MW_SMTP_HOST","127.0.0.1"); std::env::set_var("MW_SMTP_PORT",port.to_string()); std::env::set_var("MW_SMTP_SECURITY","plaintext"); }
    let imap=MockImap::start(ENGINE_USER,ENGINE_PASS).await;
    let server=spawn(ServerMode::Engine,imap.url()).await; let client=browser();
    let account=engine_login(&client,&server.base,&imap).await;
    let session:Value=client.get(format!("{}/jmap/session",server.base)).send().await.unwrap().json().await.unwrap();
    assert_eq!(session["accounts"][&account]["accountCapabilities"]["urn:ietf:params:jmap:submission"]["submissionExtensions"],json!({"DSN":[]}));
    let draft=jmap(&client,&server.base,"Email/set",json!({"accountId":account,"create":{"draft":{"from":[{"email":ENGINE_USER}],"to":[{"email":"friend@example.org"}],"subject":"DSN required","bodyValues":{"1":{"value":"hello"}},"textBody":[{"partId":"1","type":"text/plain"}]}}})).await;
    let id=draft["created"]["draft"]["id"].as_str().expect("draft created");
    let response:Value=client.post(format!("{}/jmap/api",server.base)).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail","urn:ietf:params:jmap:submission"],"methodCalls":[["EmailSubmission/set",{"accountId":account,"create":{"send":{"emailId":id,"envelope":{"mailFrom":{"email":ENGINE_USER},"rcptTo":[{"email":"friend@example.org","parameters":{"NOTIFY":"SUCCESS,FAILURE"}}]}}}},"send"]]})).send().await.unwrap().json().await.unwrap();
    assert!(response["methodResponses"][0][1]["notCreated"]["send"].is_object(),"{response}");
    let commands=smtp.await.unwrap();
    assert_eq!(commands.iter().filter(|line|line.starts_with("AUTH PLAIN")).count(),2);
    assert!(!commands.iter().any(|line|line.starts_with("MAIL") || line.starts_with("RCPT") || line=="DATA"));
    let session:Value=client.get(format!("{}/jmap/session",server.base)).send().await.unwrap().json().await.unwrap();
    assert_eq!(session["accounts"][&account]["accountCapabilities"]["urn:ietf:params:jmap:submission"]["submissionExtensions"],json!({}));
    unsafe {std::env::remove_var("MW_SMTP_HOST");std::env::remove_var("MW_SMTP_PORT");std::env::remove_var("MW_SMTP_SECURITY");}
}

#[tokio::test]
async fn dsn_default_is_account_preference_with_admin_lock_and_readonly_permission() {
    let _guard=serial().await;
    let imap=MockImap::start(ENGINE_USER,ENGINE_PASS).await;
    let server=spawn(ServerMode::Engine,imap.url()).await;
    let client=browser();let _account=engine_login(&client,&server.base,&imap).await;
    let url=format!("{}/api/settings",server.base);
    let initial:Value=client.get(&url).send().await.unwrap().json().await.unwrap();
    assert_eq!(initial["settings"]["mail.requestDeliveryReceipt"]["value"],false);
    assert_eq!(initial["settings"]["receipts.dsnAllowed"]["value"],true);
    let own=format!("{url}/mail.requestDeliveryReceipt");
    assert_eq!(client.put(&own).json(&json!({"value":true})).send().await.unwrap().status(),200);
    let saved:Value=client.get(&url).send().await.unwrap().json().await.unwrap();
    assert_eq!(saved["settings"]["mail.requestDeliveryReceipt"]["value"],true);
    let forbidden=client.put(format!("{url}/receipts.dsnAllowed")).json(&json!({"value":true})).send().await.unwrap();
    assert_eq!(forbidden.status(),403);
    assert_eq!(forbidden.json::<Value>().await.unwrap()["code"],"setting_locked");
    server.store().await.put_setting_policy(&mw_store::settings_policy::SettingPolicyRow{
        key:"mail.requestDeliveryReceipt".into(),global_value:Some("false".into()),
        allow_user_override:false,global_locked:true,updated_by:"fixture".into(),updated_at:0,
    }).await.unwrap();
    let locked:Value=client.get(&url).send().await.unwrap().json().await.unwrap();
    assert_eq!(locked["settings"]["mail.requestDeliveryReceipt"]["value"],false);
    assert_eq!(locked["settings"]["mail.requestDeliveryReceipt"]["locked"],true);
    assert_eq!(client.put(&own).json(&json!({"value":true})).send().await.unwrap().status(),403);
}

#[tokio::test]
async fn sending_identity_creation_requires_current_address_authority() {
    let _guard=serial().await;
    let imap=MockImap::start(ENGINE_USER,ENGINE_PASS).await;
    let server=spawn(ServerMode::Engine,imap.url()).await;
    let client=browser();let account=engine_login(&client,&server.base,&imap).await;
    let url=format!("{}/api/account/identities",server.base);
    let denied=client.post(&url).json(&json!({"name":"Unverified","email":"unverified@example.net","source":"server"})).send().await.unwrap();
    assert_eq!(denied.status(),403);
    assert_eq!(denied.json::<Value>().await.unwrap()["code"],"forbiddenFrom");
    let saved=client.post(&url).json(&json!({"name":"My identity","email":ENGINE_USER})).send().await.unwrap();
    assert_eq!(saved.status(),200);
    let id=saved.json::<Value>().await.unwrap()["id"].as_str().unwrap().to_string();
    let denied=client.post(&url).json(&json!({"id":id,"name":"Changed","email":"unverified@example.net"})).send().await.unwrap();
    assert_eq!(denied.status(),403);
    let row=server.store().await.get_identity(&id).await.unwrap().unwrap();
    assert_eq!(row.account_id,account);assert_eq!(row.email,ENGINE_USER);
}
