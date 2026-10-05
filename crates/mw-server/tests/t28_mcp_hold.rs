//! 26.20 t28-e12 (t26 audit OH-5): an MCP `mail.send` from a key without
//! unattended send is held until the mailbox owner releases it.
//!
//! Before this change the two branches of the MCP send gate had the same body:
//! both created a draft and an ordinary submission, which the engine transmits
//! inline. The response said `{"queued": true}` and the tool description said a
//! person would confirm, while the message had already been handed to SMTP. No
//! test caught it because the existing ones asserted against `mw-mcp`'s mock
//! backend, which only counts which method was called.
//!
//! These tests run the real router in engine mode (`build_app_full`), log in
//! through `/api/login` against a scripted POP3 server, mint keys through
//! `/api/keys`, call `/mcp`, and read and release the held submission through
//! `/jmap/api` — the requests the web Outbox makes. The engine's submitter is the
//! production one (`mw-smtp`), pointed at a scripted SMTP server that records
//! every envelope and counts every message it accepts. That count is the
//! property: how many times the recipient would have received the message.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, MutexGuard};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::{ServerKey, Store};

mod common;
use common::mock_mail::MockPop3;
use common::test_db;

const KEY_HEX: &str = "28e12a1b2c3d4e5f60718293a4b5c6d7e28e12a1b2c3d4e5f60718293a4b5c6d";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T28_E12</div>";
const USER: &str = "owner@example.org";
const PASS: &str = "Owner-Passw0rd!";
const RECIPIENT: &str = "recipient@example.net";

/// Longer than several dispatcher scans (500 ms each): if the dispatcher were
/// going to send a row, it would have by then.
const SEVERAL_SCANS: Duration = Duration::from_millis(2200);

// ── scripted SMTP server ─────────────────────────────────────────────────────

/// An SMTP server that accepts everything, records each `RCPT TO` line and
/// counts the messages it accepted (a completed `DATA`).
#[derive(Clone)]
struct Smtp {
    addr: SocketAddr,
    delivered: Arc<AtomicUsize>,
    recipients: Arc<std::sync::Mutex<Vec<String>>>,
}

impl Smtp {
    async fn start() -> Smtp {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let smtp = Smtp {
            addr: listener.local_addr().unwrap(),
            delivered: Arc::default(),
            recipients: Arc::default(),
        };
        let this = smtp.clone();
        tokio::spawn(async move {
            loop {
                let Ok((sock, _)) = listener.accept().await else {
                    return;
                };
                let this = this.clone();
                tokio::spawn(async move {
                    let _ = this.serve(sock).await;
                });
            }
        });
        smtp
    }

    fn delivered(&self) -> usize {
        self.delivered.load(Ordering::SeqCst)
    }

    fn recipients(&self) -> Vec<String> {
        self.recipients.lock().unwrap().clone()
    }

    async fn serve(&self, sock: tokio::net::TcpStream) -> std::io::Result<()> {
        let (read, mut write) = sock.into_split();
        let mut lines = BufReader::new(read).lines();
        write.write_all(b"220 mock ESMTP\r\n").await?;
        // Lines of a multi-step AUTH exchange still expected from the client.
        let mut auth_lines = 0usize;
        let mut in_data = false;
        while let Some(line) = lines.next_line().await? {
            if in_data {
                if line == "." {
                    in_data = false;
                    self.delivered.fetch_add(1, Ordering::SeqCst);
                    write.write_all(b"250 queued\r\n").await?;
                }
                continue;
            }
            if auth_lines > 0 {
                auth_lines -= 1;
                let reply: &[u8] = if auth_lines == 0 {
                    b"235 authenticated\r\n"
                } else {
                    b"334 \r\n"
                };
                write.write_all(reply).await?;
                continue;
            }
            let upper = line.to_ascii_uppercase();
            let reply: &[u8] = if upper.starts_with("EHLO") {
                b"250-mock\r\n250 AUTH PLAIN LOGIN\r\n"
            } else if upper.starts_with("HELO") {
                b"250 mock\r\n"
            } else if upper.starts_with("AUTH PLAIN") {
                if upper.trim() == "AUTH PLAIN" {
                    auth_lines = 1;
                    b"334 \r\n"
                } else {
                    b"235 authenticated\r\n"
                }
            } else if upper.starts_with("AUTH LOGIN") {
                auth_lines = if upper.trim() == "AUTH LOGIN" { 2 } else { 1 };
                b"334 \r\n"
            } else if upper.starts_with("DATA") {
                in_data = true;
                b"354 go ahead\r\n"
            } else if upper.starts_with("QUIT") {
                write.write_all(b"221 bye\r\n").await?;
                return Ok(());
            } else {
                if upper.starts_with("RCPT TO") {
                    self.recipients.lock().unwrap().push(line.clone());
                }
                // MAIL FROM, RCPT TO, RSET, NOOP.
                b"250 ok\r\n"
            };
            write.write_all(reply).await?;
        }
        Ok(())
    }
}

// ── harness ──────────────────────────────────────────────────────────────────

/// Serialises the tests in this binary and points the engine's submitter at
/// `smtp`: the server reads `MW_SMTP_*` and `MW_ENGINE_TLS` from the process
/// environment when an account logs in.
async fn serial(smtp: &Smtp) -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK.get_or_init(Default::default).lock().await;
    // SAFETY: the guard serialises every mutation and every dependent read here.
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
        std::env::set_var("MW_SMTP_HOST", "127.0.0.1");
        std::env::set_var("MW_SMTP_PORT", smtp.addr.port().to_string());
        std::env::set_var("MW_SMTP_SECURITY", "plaintext");
        for k in ["MW_MCP_RESOURCE", "MW_WEBAUTHN_ORIGIN", "MW_HEADER_AUTH"] {
            std::env::remove_var(k);
        }
    }
    guard
}

/// One engine-mode server with a logged-in mailbox owner.
struct Fixture {
    base: String,
    db: String,
    /// The owner's browser session.
    owner: reqwest::Client,
    account_id: String,
    smtp: Smtp,
    _pop: MockPop3,
}

impl Fixture {
    async fn start(smtp: &Smtp) -> Fixture {
        let pop = MockPop3::start(USER, PASS).await;
        let dir = test_db::unique_dir("mw-t28-e12");
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
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
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
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
            .unwrap();
        });
        let base = format!("http://{addr}");

        let owner = reqwest::Client::builder()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let login = owner
            .post(format!("{base}/api/login"))
            .json(&json!({ "jmapUrl": pop.url(), "username": USER, "password": PASS }))
            .send()
            .await
            .unwrap();
        assert_eq!(login.status(), 200, "engine login");
        let account_id = login.json::<Value>().await.unwrap()["accountId"]
            .as_str()
            .expect("the login names the account")
            .to_string();
        Fixture {
            base,
            db,
            owner,
            account_id,
            smtp: smtp.clone(),
            _pop: pop,
        }
    }

    /// A second handle on the server's database, for reading rows and for the
    /// one write no route allows (a countersign on a key that did not ask).
    async fn store(&self) -> Store {
        Store::open(&self.db, ServerKey::from_hex(KEY_HEX).unwrap())
            .await
            .expect("open the server's store")
    }

    /// Mint an API key with the `mail.send` tool through `/api/keys`, as the
    /// owner. Returns `(token, prefix)`.
    async fn mint_send_key(&self, unattended: bool) -> (String, String) {
        let scope = json!({
            "read": false, "send": true, "delete": false,
            "accounts": { "subset": [self.account_id] }, "folders": "all",
            "mail": true, "pim": false, "ip_allowlist": [], "expires_at": null,
            "rate_limit": null, "mcp_tools": ["mail.send"], "unattended_send": unattended,
        });
        let mint: Value = self
            .owner
            .post(format!("{}/api/keys", self.base))
            .json(&json!({ "label": "t28-e12", "accountId": self.account_id, "scope": scope }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let token = mint["displayToken"]
            .as_str()
            .unwrap_or_else(|| panic!("/api/keys returned no displayToken: {mint}"))
            .to_string();
        let prefix = token
            .strip_prefix("mwk_")
            .and_then(|rest| rest.split('.').next())
            .expect("mwk_<prefix>.<secret>")
            .to_string();
        (token, prefix)
    }

    /// Set or withdraw a key's countersign the way an administrator does:
    /// `PUT /admin/api-keys/{id}/unattended-send` on an admin session. Returns
    /// the HTTP status. The server keeps running; nothing else is told.
    async fn countersign(&self, prefix: &str, approved: bool) -> u16 {
        let admin = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();
        let login = admin
            .post(format!("{}/admin/login", self.base))
            .json(&json!({ "username": "root", "password": "hunter2" }))
            .send()
            .await
            .unwrap();
        assert_eq!(login.status(), 200, "admin login");
        // A key's row id is its prefix (`stores_v6::api_key_to_row`).
        admin
            .put(format!(
                "{}/admin/api-keys/{prefix}/unattended-send",
                self.base
            ))
            .json(&json!({ "approved": approved }))
            .send()
            .await
            .unwrap()
            .status()
            .as_u16()
    }

    async fn countersign_flag(&self, prefix: &str) -> bool {
        self.store()
            .await
            .get_api_key(prefix)
            .await
            .unwrap()
            .expect("the minted key is stored")
            .unattended_send
    }

    /// One JSON-RPC request to `/mcp` with `token` as the bearer.
    async fn mcp(&self, token: &str, method: &str, params: Value) -> Value {
        let resp = reqwest::Client::new()
            .post(format!("{}/mcp", self.base))
            .bearer_auth(token)
            .json(&json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }))
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let body: Value = resp.json().await.unwrap_or(Value::Null);
        assert!(
            body.is_object(),
            "/mcp answered {status} without a JSON-RPC body"
        );
        body
    }

    async fn mcp_send(&self, token: &str, subject: &str) -> Value {
        self.mcp(
            token,
            "tools/call",
            json!({ "name": "mail.send", "arguments": {
                "account": self.account_id,
                "to": [RECIPIENT],
                "subject": subject,
                "body_text": "written by an agent",
            } }),
        )
        .await
    }

    /// One JMAP method call on the owner's session; returns its arguments.
    async fn jmap(&self, method: &str, mut args: Value) -> Value {
        args["accountId"] = json!(self.account_id);
        let resp = self
            .owner
            .post(format!("{}/jmap/api", self.base))
            .json(&json!({
                "using": [
                    "urn:ietf:params:jmap:core",
                    "urn:ietf:params:jmap:mail",
                    "urn:ietf:params:jmap:submission",
                ],
                "methodCalls": [[method, args, "c"]],
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "{method} over the owner's session");
        let body: Value = resp.json().await.unwrap();
        assert_eq!(body["methodResponses"][0][0], method, "{body}");
        body["methodResponses"][0][1].clone()
    }

    /// The Outbox as the web client loads it: every submission, hydrated.
    async fn outbox(&self) -> Vec<Value> {
        let ids = self.jmap("EmailSubmission/query", json!({})).await["ids"].clone();
        self.jmap("EmailSubmission/get", json!({ "ids": ids }))
            .await["list"]
            .as_array()
            .expect("EmailSubmission/get list")
            .clone()
    }

    async fn submission(&self, id: &str) -> Value {
        self.jmap("EmailSubmission/get", json!({ "ids": [id] }))
            .await["list"][0]
            .clone()
    }

    /// What the Outbox "Release" / "Send now" button sends.
    async fn release(&self, id: &str) -> Value {
        self.jmap(
            "EmailSubmission/set",
            json!({ "update": { id: { "sendAt": null, "mailwomanHoldSeconds": 0 } } }),
        )
        .await
    }
}

fn structured(resp: &Value) -> &Value {
    &resp["result"]["structuredContent"]
}

// ── tests ────────────────────────────────────────────────────────────────────

/// The whole of OH-5 in one run: the send is held, the dispatcher leaves it
/// alone, the owner sees who created it, and a release is what sends it — once.
#[tokio::test]
async fn an_mcp_send_from_an_ordinary_key_is_held_until_the_owner_releases_it() {
    let smtp = Smtp::start().await;
    let _g = serial(&smtp).await;
    let f = Fixture::start(&smtp).await;
    let (key, prefix) = f.mint_send_key(false).await;
    assert_eq!(f.smtp.delivered(), 0, "precondition: nothing sent yet");
    assert!(f.outbox().await.is_empty(), "precondition: an empty Outbox");

    let resp = f.mcp_send(&key, "held until released").await;
    let outcome = structured(&resp);
    assert_eq!(outcome["queued"], true, "{resp}");
    assert_eq!(outcome["sent"], false, "{resp}");
    assert_eq!(outcome["note"], mw_mcp::HELD_NOTE, "{resp}");
    let sub_id = outcome["outboxId"]
        .as_str()
        .unwrap_or_else(|| panic!("an outboxId: {resp}"))
        .to_string();
    assert_eq!(f.smtp.delivered(), 0, "the call handed nothing to SMTP");

    // The real dispatcher is running (started at boot); give it several scans.
    tokio::time::sleep(SEVERAL_SCANS).await;
    assert_eq!(
        f.smtp.delivered(),
        0,
        "the dispatcher did not send it either"
    );

    // What the owner's Outbox shows: one held row, naming the key.
    let rows = f.outbox().await;
    assert_eq!(rows.len(), 1, "{rows:?}");
    let row = &rows[0];
    assert_eq!(row["id"], sub_id.as_str());
    assert_eq!(row["undoStatus"], "pending");
    assert_eq!(row["mailwomanHold"], "manual");
    assert_eq!(
        row["mailwomanOrigin"],
        json!({ "kind": "apiKey", "name": prefix })
    );
    // And the store agrees with the wire.
    let held = f.store().await.get_submission_hold(&sub_id).await.unwrap();
    assert_eq!(held.unwrap().hold.as_deref(), Some("manual"));

    // The owner releases it: now, and only now, it reaches SMTP.
    let set = f.release(&sub_id).await;
    assert_eq!(set["updated"][&sub_id]["undoStatus"], "final", "{set}");
    assert_eq!(f.smtp.delivered(), 1, "the release sent it");
    assert_eq!(f.smtp.recipients().len(), 1, "{:?}", f.smtp.recipients());
    assert!(
        f.smtp.recipients()[0].contains(RECIPIENT),
        "to the recipient the agent named: {:?}",
        f.smtp.recipients()
    );
    let sent = f.submission(&sub_id).await;
    assert_eq!(sent["undoStatus"], "final");
    assert_eq!(sent["mailwomanHold"], Value::Null);
    assert_eq!(sent["mailwomanOrigin"]["name"], prefix.as_str());

    tokio::time::sleep(SEVERAL_SCANS).await;
    assert_eq!(f.smtp.delivered(), 1, "sent once");
    // A second release is refused and sends nothing.
    let again = f.release(&sub_id).await;
    assert!(again["notUpdated"].get(&sub_id).is_some(), "{again}");
    assert_eq!(f.smtp.delivered(), 1);
}

/// The owner can discard a held send instead; it is then never transmitted and
/// cannot be released.
#[tokio::test]
async fn a_held_mcp_send_can_be_discarded_and_is_never_sent() {
    let smtp = Smtp::start().await;
    let _g = serial(&smtp).await;
    let f = Fixture::start(&smtp).await;
    let (key, _prefix) = f.mint_send_key(false).await;

    let resp = f.mcp_send(&key, "to be discarded").await;
    let sub_id = structured(&resp)["outboxId"]
        .as_str()
        .unwrap_or_else(|| panic!("an outboxId: {resp}"))
        .to_string();

    let set = f
        .jmap(
            "EmailSubmission/set",
            json!({ "update": { &sub_id: { "undoStatus": "canceled" } } }),
        )
        .await;
    assert!(set["updated"].get(&sub_id).is_some(), "{set}");
    assert_eq!(f.submission(&sub_id).await["undoStatus"], "canceled");

    let released = f.release(&sub_id).await;
    assert!(released["notUpdated"].get(&sub_id).is_some(), "{released}");
    tokio::time::sleep(SEVERAL_SCANS).await;
    assert_eq!(
        f.smtp.delivered(),
        0,
        "a discarded send is never transmitted"
    );
}

/// A key with unattended send transmits during the call when its countersign
/// flag is set, and the flag is read from the store for each call: set after
/// boot it is honoured without a restart, and cleared it stops being honoured.
#[tokio::test]
async fn the_countersign_flag_is_read_live_and_a_countersigned_key_sends_at_once() {
    let smtp = Smtp::start().await;
    let _g = serial(&smtp).await;
    let f = Fixture::start(&smtp).await;
    // Minted after boot: the server had no snapshot that could contain it.
    let (key, prefix) = f.mint_send_key(true).await;

    // As minted the key asks for unattended send and has no countersign:
    // refused outright (-32002). Not sent, and not queued either.
    assert!(
        !f.countersign_flag(&prefix).await,
        "precondition: asking for unattended send does not countersign the key"
    );
    let resp = f.mcp_send(&key, "not countersigned").await;
    assert_eq!(resp["error"]["code"], -32002, "{resp}");
    assert_eq!(f.smtp.delivered(), 0);
    assert!(f.outbox().await.is_empty(), "a refused send leaves no row");

    // An administrator countersigns it, on the same running server: the next
    // call transmits.
    assert_eq!(f.countersign(&prefix, true).await, 200);
    assert!(f.countersign_flag(&prefix).await);
    let resp = f.mcp_send(&key, "countersigned").await;
    let outcome = structured(&resp);
    assert_eq!(outcome["sent"], true, "{resp}");
    assert_eq!(outcome["queued"], false, "{resp}");
    assert_eq!(f.smtp.delivered(), 1, "sent at once, by the call");
    let sub_id = outcome["submissionId"].as_str().expect("a submissionId");
    let row = f.submission(sub_id).await;
    assert_eq!(row["undoStatus"], "final");
    assert_eq!(row["mailwomanHold"], Value::Null);

    // Withdrawn: refused again from the next call on.
    assert_eq!(f.countersign(&prefix, false).await, 200);
    let resp = f.mcp_send(&key, "withdrawn").await;
    assert_eq!(resp["error"]["code"], -32002, "{resp}");
    assert_eq!(f.smtp.delivered(), 1);

    // The flag alone does not make a key unattended: a key whose scope lacks
    // unattended send is held even with the flag set.
    let (plain, plain_prefix) = f.mint_send_key(false).await;
    assert_eq!(
        f.countersign(&plain_prefix, true).await,
        409,
        "the admin route does not countersign a key that did not ask"
    );
    // Written past the route, to show the gate does not rest on that refusal.
    assert!(
        f.store()
            .await
            .set_api_key_unattended_send(&plain_prefix, true)
            .await
            .unwrap()
    );
    let resp = f.mcp_send(&plain, "flag without scope").await;
    assert_eq!(structured(&resp)["queued"], true, "{resp}");
    assert_eq!(structured(&resp)["sent"], false, "{resp}");
    tokio::time::sleep(SEVERAL_SCANS).await;
    assert_eq!(f.smtp.delivered(), 1, "held, not sent");
}

/// What `/mcp` tells every client about `mail.send` matches what the server
/// does with it.
#[tokio::test]
async fn the_served_tool_description_matches_the_behaviour() {
    let smtp = Smtp::start().await;
    let _g = serial(&smtp).await;
    let f = Fixture::start(&smtp).await;
    let (key, _prefix) = f.mint_send_key(false).await;

    let list = f.mcp(&key, "tools/list", json!({})).await;
    let desc = list["result"]["tools"]
        .as_array()
        .unwrap_or_else(|| panic!("tools/list: {list}"))
        .iter()
        .find(|t| t["name"] == "mail.send")
        .and_then(|t| t["description"].as_str())
        .expect("mail.send is listed")
        .to_string();
    assert!(desc.contains("does NOT send"), "{desc}");
    assert!(desc.contains("sent: false"), "{desc}");

    // And the behaviour it describes, on this same server.
    let resp = f.mcp_send(&key, "as described").await;
    assert_eq!(structured(&resp)["sent"], false, "{resp}");
    assert_eq!(f.smtp.delivered(), 0);
}
