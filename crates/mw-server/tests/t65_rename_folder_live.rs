//! t65 — renaming a folder from the folder menu, against a real IMAP server,
//! through the routes the browser uses in engine mode (`/api/login`,
//! `/jmap/api` `Mailbox/set` update of `name`).
//!
//! What the server holds is read back over a fresh IMAP connection (`LIST`,
//! `LSUB`); what the browser sees is read over JMAP (`Mailbox/get`,
//! `Email/query`).
//!
//! ## Running
//! The same servers as `t28_folders_live` (any IMAP server with
//! `testuser`/`testpass` and a second login, `otheruser`/`otherpass`; Greenmail
//! with `-Dgreenmail.auth.disabled` creates both on login), plaintext:
//!
//! ```text
//! MW_T28E17_IMAP=127.0.0.1:3143 \
//!   cargo test -p mw-server --test t65_rename_folder_live -- --test-threads=1
//! ```
//!
//! Unset, the cases skip and say so.

mod common;

use std::net::SocketAddr;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use tokio::sync::{Mutex, MutexGuard};

use mw_imap::session::{Credentials, Session};
use mw_imap::transport::TlsMode;
use mw_server::{AppConfig, HardeningConfig, ServerMode, build_app};

use common::test_db;

const USER: &str = "testuser";
const PASS: &str = "testpass";
const OTHER: &str = "otheruser";
const OTHER_PASS: &str = "otherpass";

fn imap_servers() -> Vec<String> {
    std::env::var("MW_T28E17_IMAP")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

async fn serial() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK.get_or_init(Default::default).lock().await;
    // SAFETY: the guard serialises every mutation and every dependent read here.
    unsafe { std::env::set_var("MW_ENGINE_TLS", "plaintext") };
    guard
}

fn stem(case: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("T65{case}{}", nanos % 1_000_000_000)
}

async fn spawn_engine_server(upstream: &str) -> String {
    let base = test_db::unique_dir("mw-t65");
    let web_dir = base.join("web");
    std::fs::create_dir_all(&web_dir).unwrap();
    std::fs::write(
        web_dir.join("index.html"),
        "<!doctype html><title>MW</title>",
    )
    .unwrap();
    let config = AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web_dir),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
        security: mw_server::SecurityConfig {
            upstream_default: Some(upstream.to_string()),
            ..mw_server::SecurityConfig::default()
        },
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

struct Browser {
    c: reqwest::Client,
    server: String,
    account: String,
}

impl Browser {
    async fn login(server: &str, user: &str, pass: &str) -> Browser {
        let c = reqwest::Client::builder()
            .cookie_store(true)
            .no_proxy()
            .build()
            .unwrap();
        let login = c
            .post(format!("{server}/api/login"))
            .json(&json!({ "username": user, "password": pass }))
            .send()
            .await
            .unwrap();
        let status = login.status();
        let body: Value = login.json().await.unwrap_or_default();
        assert_eq!(status, 200, "engine login as {user}: {body}");
        let account = body["accountId"].as_str().unwrap().to_string();
        Browser {
            c,
            server: server.to_string(),
            account,
        }
    }

    /// One method call on `account` (this browser's own unless named).
    async fn call_on(&self, account: &str, name: &str, mut args: Value) -> Value {
        args["accountId"] = json!(account);
        let resp: Value = self
            .c
            .post(format!("{}/jmap/api", self.server))
            .json(&json!({
                "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
                "methodCalls": [[name, args, "c"]],
            }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        resp["methodResponses"][0].clone()
    }

    async fn call(&self, name: &str, args: Value) -> Value {
        self.call_on(&self.account.clone(), name, args).await[1].clone()
    }

    async fn mailboxes(&self) -> Vec<Value> {
        self.call("Mailbox/get", json!({}))
            .await
            .get("list")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    }

    async fn mailbox(&self, id: &str) -> Value {
        self.call("Mailbox/get", json!({ "ids": [id] })).await["list"][0].clone()
    }

    async fn rename(&self, id: &str, name: &str) -> Value {
        self.call("Mailbox/set", json!({ "update": { id: { "name": name } } }))
            .await
    }

    /// The ids `Email/query` finds in `mailbox`, once the engine has
    /// indexed it (the first sync runs in the background after login).
    async fn emails_in(&self, mailbox: &str, expect: usize) -> Vec<String> {
        for _ in 0..120 {
            let got = self
                .call("Email/query", json!({ "filter": { "inMailbox": mailbox } }))
                .await;
            let ids: Vec<String> = got["ids"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            if ids.len() >= expect {
                return ids;
            }
            let _ = self.call("Mailbox/get", json!({ "ids": [mailbox] })).await;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        panic!("mailbox {mailbox} never showed {expect} message(s)");
    }
}

async fn fresh(addr: &str) -> Session {
    let (host, port) = addr.rsplit_once(':').expect("host:port");
    let mut s = Session::connect(host, port.parse().unwrap(), TlsMode::Plaintext)
        .await
        .unwrap_or_else(|e| panic!("connect {addr}: {e}"));
    s.login(&Credentials::Password {
        username: USER.into(),
        password: PASS.into(),
    })
    .await
    .unwrap();
    s
}

/// The server's folder names (wire spelling), from a fresh `LIST`.
async fn server_names(addr: &str) -> Vec<String> {
    let mut s = fresh(addr).await;
    let names = s
        .list_mailboxes()
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.mailbox_ref.name)
        .collect();
    let _ = s.logout().await;
    names
}

async fn server_subscribed(addr: &str) -> Vec<String> {
    let mut s = fresh(addr).await;
    let names = s.subscribed_mailboxes().await.unwrap();
    let _ = s.logout().await;
    names
}

async fn append(addr: &str, mailbox: &str, marker: &str) {
    let raw = format!(
        "Message-ID: <{marker}@t65.test>\r\n\
         From: alice@example.org\r\n\
         To: {USER}@example.org\r\n\
         Subject: {marker}\r\n\
         Date: Mon, 06 Jul 2026 09:00:00 +0000\r\n\
         \r\n\
         body {marker}\r\n"
    );
    let mut s = fresh(addr).await;
    s.append(mailbox, raw.as_bytes(), &[]).await.unwrap();
    let _ = s.logout().await;
}

async fn delimiter(addr: &str) -> char {
    let mut s = fresh(addr).await;
    let d = s
        .hierarchy_delimiter()
        .await
        .unwrap()
        .expect("a hierarchical server");
    let _ = s.logout().await;
    d
}

fn skip_without_servers(case: &str) -> bool {
    if imap_servers().is_empty() {
        common::gate::skip(format_args!(
            "[t65 {case}] MW_T28E17_IMAP unset: no live IMAP server driven"
        ));
        return true;
    }
    false
}

/// A parent with a child and a message, renamed to a name outside ASCII: the
/// server holds the modified UTF-7 spelling for both, the browser reads the
/// typed name back, and the folder ids, the message id, the counts, the
/// nesting and the subscription all follow the folder.
#[tokio::test]
async fn renaming_a_parent_to_a_non_ascii_name_keeps_its_child_messages_and_subscription() {
    if skip_without_servers("nested-utf7") {
        return;
    }
    let _env = serial().await;
    for addr in imap_servers() {
        let d = delimiter(&addr).await;
        let top = stem("Top");
        let tag = stem("");
        let typed = format!("Relatórios {tag}");
        let wire = format!("Relat&APM-rios {tag}");
        let child_wire = format!("{top}{d}Kid");
        // Filed before login, as another client would, so the login's first
        // sync caches the folders and the message.
        let mut s = fresh(&addr).await;
        s.create_mailbox(&top).await.unwrap();
        s.create_mailbox(&child_wire).await.unwrap();
        s.subscribe(&top, true).await.unwrap();
        s.subscribe(&child_wire, true).await.unwrap();
        let _ = s.logout().await;
        let marker = stem("Msg");
        append(&addr, &child_wire, &marker).await;

        let server = spawn_engine_server(&format!("imap://{addr}")).await;
        let b = Browser::login(&server, USER, PASS).await;
        let by_upstream = |name: String| {
            let b = &b;
            async move {
                for _ in 0..60 {
                    if let Some(id) = b
                        .mailboxes()
                        .await
                        .iter()
                        .find(|m| m["mailwomanUpstreamName"] == json!(name))
                        .and_then(|m| m["id"].as_str().map(str::to_string))
                    {
                        return id;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
                panic!("{name} never listed");
            }
        };
        let parent = by_upstream(top.clone()).await;
        let child = by_upstream(child_wire.clone()).await;
        let before = b.emails_in(&child, 1).await;
        let counted = b.mailbox(&child).await;
        assert_eq!(counted["totalEmails"], 1, "{addr}: {counted}");
        assert!(
            server_subscribed(&addr).await.contains(&top),
            "{addr}: precondition: a new folder is subscribed"
        );

        let set = b.rename(&parent, &typed).await;
        assert!(
            set["updated"]
                .as_object()
                .is_some_and(|u| u.contains_key(&parent)),
            "{addr}: {set}"
        );

        // On the server: the modified UTF-7 spelling, the child moved with it.
        let names = server_names(&addr).await;
        let wire_child = format!("{wire}{d}Kid");
        assert!(
            names.contains(&wire) && names.contains(&wire_child),
            "{addr}: {names:?}"
        );
        assert!(
            !names.contains(&top) && !names.contains(&child_wire),
            "{addr}: {names:?}"
        );
        let subscribed = server_subscribed(&addr).await;
        assert!(
            subscribed.contains(&wire),
            "{addr}: subscription follows: {subscribed:?}"
        );
        assert!(
            !subscribed.contains(&top),
            "{addr}: old name unsubscribed: {subscribed:?}"
        );

        // In the browser: the typed name, the same ids, the nesting, the counts.
        let got = b.mailbox(&parent).await;
        assert_eq!(got["name"], json!(typed), "{addr}: {got}");
        assert_eq!(got["mailwomanUpstreamName"], json!(wire), "{addr}: {got}");
        let got = b.mailbox(&child).await;
        assert_eq!(got["name"], "Kid", "{addr}: {got}");
        assert_eq!(got["parentId"], json!(parent), "{addr}: {got}");
        assert_eq!(got["totalEmails"], 1, "{addr}: {got}");
        assert_eq!(got["myRights"]["mayRename"], true, "{addr}: {got}");
        let after = b.emails_in(&child, 1).await;
        assert_eq!(after, before, "{addr}: the message keeps its id");

        // A fresh engine (a reload after a server restart) reads the same name.
        let server2 = spawn_engine_server(&format!("imap://{addr}")).await;
        let b2 = Browser::login(&server2, USER, PASS).await;
        let listed = b2.mailboxes().await;
        assert!(
            listed
                .iter()
                .any(|m| m["name"] == json!(typed) && m["parentId"].is_null()),
            "{addr}: {listed:?}"
        );

        // Renaming back to ASCII works the same way.
        let set = b.rename(&parent, &top).await;
        assert!(
            set["updated"]
                .as_object()
                .is_some_and(|u| u.contains_key(&parent)),
            "{addr}: {set}"
        );
        assert!(server_names(&addr).await.contains(&child_wire), "{addr}");

        let _ = b
            .call(
                "Mailbox/set",
                json!({ "destroy": [&parent, &child], "onDestroyRemoveEmails": true }),
            )
            .await;
    }
}

/// Role folders, a sibling's name, a delimiter in the name and an empty name
/// are refused, and the server is left as it was.
#[tokio::test]
async fn rename_refuses_role_folders_duplicates_and_bad_names() {
    if skip_without_servers("refusals") {
        return;
    }
    let _env = serial().await;
    for addr in imap_servers() {
        let server = spawn_engine_server(&format!("imap://{addr}")).await;
        let b = Browser::login(&server, USER, PASS).await;
        let d = delimiter(&addr).await;
        let first = stem("A");
        let second = stem("B");
        let set = b
            .call(
                "Mailbox/set",
                json!({ "create": { "a": { "name": first }, "b": { "name": second } } }),
            )
            .await;
        let a = set["created"]["a"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("{addr}: {set}"))
            .to_string();
        let bb = set["created"]["b"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("{addr}: {set}"))
            .to_string();

        let set = b.rename(&bb, &first).await;
        assert_eq!(
            set["notUpdated"][&bb]["type"], "alreadyExists",
            "{addr}: {set}"
        );
        let set = b.rename(&bb, &format!("x{d}y")).await;
        assert_eq!(
            set["notUpdated"][&bb]["type"], "invalidProperties",
            "{addr}: {set}"
        );
        let set = b.rename(&bb, "   ").await;
        assert_eq!(
            set["notUpdated"][&bb]["type"], "invalidProperties",
            "{addr}: {set}"
        );
        let set = b.rename(&bb, &"x".repeat(256)).await;
        assert_eq!(
            set["notUpdated"][&bb]["type"], "invalidProperties",
            "{addr}: {set}"
        );

        for m in b.mailboxes().await {
            if m["role"].is_null() || m.get("mailwomanSearchQuery").is_some() {
                continue;
            }
            let id = m["id"].as_str().unwrap();
            assert_eq!(m["myRights"]["mayRename"], false, "{addr}: {m}");
            let set = b.rename(id, &stem("Role")).await;
            assert_eq!(
                set["notUpdated"][id]["type"], "forbidden",
                "{addr}: {m} -> {set}"
            );
        }
        let names = server_names(&addr).await;
        assert!(
            names.contains(&first) && names.contains(&second),
            "{addr}: {names:?}"
        );
        assert!(
            names.iter().any(|n| n.eq_ignore_ascii_case("inbox")),
            "{addr}: {names:?}"
        );

        let _ = b.call("Mailbox/set", json!({ "destroy": [&a, &bb] })).await;
    }
}

/// A folder of one account cannot be renamed from another: naming the other
/// account is refused before any method runs, and the other account's folder
/// id is unknown in one's own.
#[tokio::test]
async fn rename_is_limited_to_the_signed_in_accounts_folders() {
    if skip_without_servers("ownership") {
        return;
    }
    let _env = serial().await;
    for addr in imap_servers() {
        let server = spawn_engine_server(&format!("imap://{addr}")).await;
        let owner = Browser::login(&server, USER, PASS).await;
        let other = Browser::login(&server, OTHER, OTHER_PASS).await;
        assert_ne!(owner.account, other.account, "{addr}");
        let name = stem("Own");
        let set = owner
            .call(
                "Mailbox/set",
                json!({ "create": { "o": { "name": name } } }),
            )
            .await;
        let id = set["created"]["o"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("{addr}: {set}"))
            .to_string();

        // The other user names the owner's account: refused, or answered
        // in the caller's own account, where the folder does not exist.
        let reply = other
            .call_on(
                &owner.account,
                "Mailbox/set",
                json!({ "update": { &id: { "name": stem("Stolen") } } }),
            )
            .await;
        assert!(
            reply[0] == "error"
                || (reply[1]["accountId"] == json!(other.account)
                    && reply[1]["notUpdated"][&id]["type"] == "notFound"),
            "{addr}: {reply}"
        );
        // The other user names the owner's folder id in their own account.
        let set = other.rename(&id, &stem("Stolen")).await;
        assert_eq!(set["notUpdated"][&id]["type"], "notFound", "{addr}: {set}");

        assert!(
            server_names(&addr).await.contains(&name),
            "{addr}: untouched"
        );
        let _ = owner.call("Mailbox/set", json!({ "destroy": [&id] })).await;
    }
}
