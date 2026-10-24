//! 26.20 t28-e17 — folder management and permanent removal against real IMAP
//! servers, through the routes the browser uses in engine mode (`/api/login`,
//! `/jmap/session`, `/jmap/api`).
//!
//! Every outcome is read back over a **fresh IMAP connection** opened by the test
//! itself (`LIST`, `SELECT` + `FETCH`), never from the engine's own answer: the
//! question is what the mail server now holds.
//!
//! ## Running
//! Greenmail and Dovecot (the dev compose services, or any equivalent) with the
//! `testuser`/`testpass` account, plaintext:
//!
//! ```text
//! MW_T28E17_IMAP=127.0.0.1:3143,127.0.0.1:2143 \
//! MW_T28E17_POP3=127.0.0.1:3110 \
//!   cargo test -p mw-server --test t28_folders_live -- --test-threads=1
//! ```
//!
//! `MW_T28E17_IMAP` is a comma-separated list; every case runs against each
//! server in it. Unset, the cases skip and say so.

mod common;

use std::net::SocketAddr;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use tokio::sync::{Mutex, MutexGuard};

use mw_engine::backend::RawMailboxRef;
use mw_imap::session::{Credentials, SelectMode, Session};
use mw_imap::transport::TlsMode;
use mw_server::{AppConfig, HardeningConfig, ServerMode, build_app};

use common::test_db;

const USER: &str = "testuser";
const PASS: &str = "testpass";

/// The IMAP servers to run against (`host:port`), from `MW_T28E17_IMAP`.
fn imap_servers() -> Vec<String> {
    std::env::var("MW_T28E17_IMAP")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Serialises the cases (they set `MW_ENGINE_TLS`, which login reads).
async fn serial() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK.get_or_init(Default::default).lock().await;
    // SAFETY: the guard serialises every mutation and every dependent read here.
    unsafe { std::env::set_var("MW_ENGINE_TLS", "plaintext") };
    guard
}

/// A unique folder-name stem per case, so reruns against a long-lived server
/// never collide with what an earlier run left.
fn stem(case: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("T28{case}{}", nanos % 1_000_000_000)
}

async fn spawn_engine_server(upstream: &str) -> String {
    let base = test_db::unique_dir("mw-t28e17");
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

/// A logged-in browser: its cookie jar, the server, and the account id. The
/// mail server is the deployment's upstream (`MW_UPSTREAM_DEFAULT`); a client
/// cannot name one at login.
struct Browser {
    c: reqwest::Client,
    server: String,
    account: String,
}

impl Browser {
    async fn login(url: &str) -> Browser {
        let server = spawn_engine_server(url).await;
        let c = reqwest::Client::builder()
            .cookie_store(true)
            .no_proxy()
            .build()
            .unwrap();
        let login = c
            .post(format!("{server}/api/login"))
            .json(&json!({ "username": USER, "password": PASS }))
            .send()
            .await
            .unwrap();
        let status = login.status();
        let body: Value = login.json().await.unwrap_or_default();
        assert_eq!(status, 200, "engine login to {url}: {body}");
        let account = body["accountId"].as_str().unwrap().to_string();
        Browser { c, server, account }
    }

    async fn session(&self) -> Value {
        self.c
            .get(format!("{}/jmap/session", self.server))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    async fn call(&self, name: &str, args: Value) -> Value {
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
        resp["methodResponses"][0][1].clone()
    }

    async fn mailbox_id(&self, pred: impl Fn(&Value) -> bool) -> Option<String> {
        let got = self.call("Mailbox/get", json!({})).await;
        got["list"]
            .as_array()?
            .iter()
            .find(|m| pred(m))
            .and_then(|m| m["id"].as_str())
            .map(str::to_string)
    }
}

/// A fresh, independent IMAP connection, as another client would open it.
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

/// The server's folder names, from a fresh `LIST`.
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

/// The message count of `mailbox`, from a fresh `LIST` + `STATUS`.
///
/// Read without selecting the folder: Greenmail 2.1.4 drops the connection of
/// a later DELETE of a folder another (already logged-out) session selected
/// (`NullPointerException` in `ImapSessionFolder.mailboxDeleted`).
async fn server_total(addr: &str, mailbox: &str) -> Option<u32> {
    let mut s = fresh(addr).await;
    let total = s
        .list_mailboxes()
        .await
        .unwrap()
        .into_iter()
        .find(|m| m.mailbox_ref.name == mailbox)
        .map(|m| m.total);
    let _ = s.logout().await;
    total
}

/// The raw messages in `mailbox`, from a fresh `SELECT` + `FETCH`.
async fn server_messages(addr: &str, mailbox: &str) -> Vec<String> {
    let mut s = fresh(addr).await;
    let sel = s.select(mailbox, SelectMode::Plain).await.unwrap();
    let uids = s.uid_search_all().await.unwrap();
    let mbref = RawMailboxRef {
        name: mailbox.to_string(),
        uidvalidity: sel.uidvalidity,
    };
    let raws = s.fetch_raw(&mbref, &uids).await.unwrap();
    let _ = s.logout().await;
    raws.into_iter()
        .map(|r| String::from_utf8_lossy(&r.raw).into_owned())
        .collect()
}

/// APPEND a message carrying `marker` in its Message-ID and subject.
async fn append(addr: &str, mailbox: &str, marker: &str) {
    let raw = format!(
        "Message-ID: <{marker}@t28e17.test>\r\n\
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
            "[t28-e17 {case}] MW_T28E17_IMAP unset: no live IMAP server driven"
        ));
        return true;
    }
    false
}

#[tokio::test]
async fn folders_created_renamed_and_deleted_over_jmap_are_what_the_server_lists() {
    if skip_without_servers("folders") {
        return;
    }
    let _env = serial().await;
    for addr in imap_servers() {
        let b = Browser::login(&format!("imap://{addr}")).await;
        let d = delimiter(&addr).await;
        let top = stem("Top");
        let renamed = stem("Ren");
        let child = format!("{top}{d}Kid");
        let renamed_child = format!("{renamed}{d}Kid");

        // The session says folders can be managed on this account.
        let session = b.session().await;
        assert_eq!(
            session["accounts"][&b.account]["accountCapabilities"]["urn:ietf:params:jmap:mail"]["mailwomanMailboxSet"],
            true,
            "{addr}: {session}"
        );

        let before = server_names(&addr).await;
        assert!(!before.contains(&top), "{addr}: precondition {before:?}");

        // Create a parent and a child (by creation reference).
        let set = b
            .call(
                "Mailbox/set",
                json!({ "create": {
                    "p": { "name": top },
                    "k": { "name": "Kid", "parentId": "#p" },
                } }),
            )
            .await;
        let parent_id = set["created"]["p"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("{addr}: {set}"))
            .to_string();
        let child_id = set["created"]["k"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("{addr}: {set}"))
            .to_string();
        let names = server_names(&addr).await;
        assert!(
            names.contains(&top) && names.contains(&child),
            "{addr}: {names:?}"
        );

        // Mailbox/get nests the child under the parent by its leaf name.
        let got = b.call("Mailbox/get", json!({ "ids": [&child_id] })).await;
        assert_eq!(
            got["list"][0]["parentId"],
            json!(parent_id),
            "{addr}: {got}"
        );
        assert_eq!(got["list"][0]["name"], "Kid", "{addr}: {got}");

        // Rename the parent: the server renames the child too.
        let set = b
            .call(
                "Mailbox/set",
                json!({ "update": { &parent_id: { "name": renamed } } }),
            )
            .await;
        assert!(
            set["updated"]
                .as_object()
                .is_some_and(|u| u.contains_key(&parent_id)),
            "{addr}: {set}"
        );
        let names = server_names(&addr).await;
        assert!(
            names.contains(&renamed) && names.contains(&renamed_child),
            "{addr}: {names:?}"
        );
        assert!(
            !names.contains(&top) && !names.contains(&child),
            "{addr}: {names:?}"
        );

        // A message another client files into the child: deleting it without
        // onDestroyRemoveEmails is refused, and so is deleting the parent.
        let marker = stem("Msg");
        append(&addr, &renamed_child, &marker).await;
        let set = b
            .call("Mailbox/set", json!({ "destroy": [&child_id] }))
            .await;
        assert_eq!(
            set["notDestroyed"][&child_id]["type"], "mailboxHasEmail",
            "{addr}: {set}"
        );
        let set = b
            .call("Mailbox/set", json!({ "destroy": [&parent_id] }))
            .await;
        assert_eq!(
            set["notDestroyed"][&parent_id]["type"], "mailboxHasChild",
            "{addr}: {set}"
        );
        let names = server_names(&addr).await;
        assert!(
            names.contains(&renamed_child),
            "{addr}: refusals deleted nothing: {names:?}"
        );
        assert_eq!(server_total(&addr, &renamed_child).await, Some(1));

        // Both, with the flag: gone from the server.
        let set = b
            .call(
                "Mailbox/set",
                json!({ "destroy": [&parent_id, &child_id], "onDestroyRemoveEmails": true }),
            )
            .await;
        assert_eq!(
            set["destroyed"].as_array().map(Vec::len),
            Some(2),
            "{addr}: {set}"
        );
        let names = server_names(&addr).await;
        assert!(
            !names.contains(&renamed) && !names.contains(&renamed_child),
            "{addr}: {names:?}"
        );

        // Special-use folders stay.
        let inbox = b.mailbox_id(|m| m["role"] == "inbox").await.unwrap();
        let set = b.call("Mailbox/set", json!({ "destroy": [&inbox] })).await;
        assert_eq!(
            set["notDestroyed"][&inbox]["type"], "forbidden",
            "{addr}: {set}"
        );
    }
}

#[tokio::test]
async fn a_destroyed_message_is_gone_from_the_server() {
    if skip_without_servers("destroy") {
        return;
    }
    let _env = serial().await;
    for addr in imap_servers() {
        // Filed before login, so the login's sync caches it.
        let keep = stem("Keep");
        let doomed = stem("Gone");
        append(&addr, "INBOX", &keep).await;
        append(&addr, "INBOX", &doomed).await;
        let on_server = server_messages(&addr, "INBOX").await;
        assert!(
            on_server.iter().any(|m| m.contains(&doomed)),
            "{addr}: precondition"
        );

        let b = Browser::login(&format!("imap://{addr}")).await;
        let inbox = b.mailbox_id(|m| m["role"] == "inbox").await.unwrap();
        let q = b
            .call(
                "Email/query",
                json!({ "filter": { "inMailbox": inbox }, "limit": 500 }),
            )
            .await;
        let ids: Vec<String> = q["ids"]
            .as_array()
            .unwrap_or_else(|| panic!("{addr}: {q}"))
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let got = b
            .call(
                "Email/get",
                json!({ "ids": ids, "properties": ["id", "subject"] }),
            )
            .await;
        let id_of = |subject: &str| {
            got["list"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["subject"] == subject)
                .and_then(|e| e["id"].as_str())
                .map(str::to_string)
                .unwrap_or_else(|| panic!("{addr}: {subject} not cached: {got}"))
        };
        let doomed_id = id_of(&doomed);

        let set = b
            .call("Email/set", json!({ "destroy": [&doomed_id] }))
            .await;
        assert_eq!(set["destroyed"], json!([doomed_id]), "{addr}: {set}");

        let on_server = server_messages(&addr, "INBOX").await;
        assert!(
            !on_server.iter().any(|m| m.contains(&doomed)),
            "{addr}: still on the server"
        );
        assert!(
            on_server.iter().any(|m| m.contains(&keep)),
            "{addr}: the other message stays"
        );
        let trash = server_names(&addr)
            .await
            .into_iter()
            .find(|n| n.eq_ignore_ascii_case("trash"));
        if let Some(trash) = trash {
            let in_trash = server_messages(&addr, &trash).await;
            assert!(
                !in_trash.iter().any(|m| m.contains(&doomed)),
                "{addr}: destroy is not a move"
            );
        }
        let got = b.call("Email/get", json!({ "ids": [&doomed_id] })).await;
        assert_eq!(got["notFound"], json!([doomed_id]), "{addr}: {got}");
    }
}

#[tokio::test]
async fn a_pop3_account_cannot_manage_folders_and_its_session_says_so() {
    let Ok(addr) = std::env::var("MW_T28E17_POP3") else {
        common::gate::skip("[t28-e17 pop3] MW_T28E17_POP3 unset: no live POP3 server driven");
        return;
    };
    let _env = serial().await;
    let b = Browser::login(&format!("pop3://{addr}")).await;
    let session = b.session().await;
    let caps = &session["accounts"][&b.account]["accountCapabilities"]["urn:ietf:params:jmap:mail"];
    assert_eq!(caps["mailwomanMailboxSet"], false, "{session}");
    assert_eq!(caps["mayCreateTopLevelMailbox"], false, "{session}");

    let set = b
        .call(
            "Mailbox/set",
            json!({ "create": { "x": { "name": "Nope" } } }),
        )
        .await;
    assert_eq!(set["type"], "accountNotSupportedByMethod", "{set}");
    let got = b.call("Mailbox/get", json!({})).await;
    assert!(
        got["list"]
            .as_array()
            .unwrap()
            .iter()
            .all(|m| m["myRights"]["mayCreateChild"] == false && m["name"] != "Nope"),
        "{got}"
    );
}
