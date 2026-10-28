//! t45 — `Mailbox/empty` against a real IMAP server, through the routes the
//! browser uses in engine mode (`/api/login`, `/jmap/session`, `/jmap/api`).
//!
//! Every outcome is read back over a **fresh IMAP connection** opened by the
//! test itself (`LIST` + `STATUS`, `SELECT` + `FETCH`): the question is what the
//! mail server now holds, not what the engine says.
//!
//! ## Running
//! Greenmail (or any equivalent) with the `testuser`/`testpass` account,
//! plaintext:
//!
//! ```text
//! MW_T45_IMAP=127.0.0.1:3143 \
//!   cargo test -p mw-server --test t45_empty_folder_live -- --test-threads=1
//! ```
//!
//! `MW_T45_IMAP` is a comma-separated list; the case runs against each server
//! in it. Unset, it skips and says so.

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

fn imap_servers() -> Vec<String> {
    std::env::var("MW_T45_IMAP")
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
    format!("T45{case}{}", nanos % 1_000_000_000)
}

async fn spawn_engine_server(upstream: &str) -> String {
    let base = test_db::unique_dir("mw-t45");
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

    /// One method call; an `error` response comes back as `{"error": ...}`.
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
        let call = &resp["methodResponses"][0];
        if call[0] == "error" {
            json!({ "error": call[1] })
        } else {
            call[1].clone()
        }
    }

    async fn mailbox_named(&self, upstream: &str) -> String {
        let got = self.call("Mailbox/get", json!({})).await;
        got["list"]
            .as_array()
            .unwrap_or_else(|| panic!("{got}"))
            .iter()
            .find(|m| m["mailwomanUpstreamName"] == upstream)
            .and_then(|m| m["id"].as_str())
            .map(str::to_string)
            .unwrap_or_else(|| panic!("no mailbox {upstream}: {got}"))
    }

    /// Wait until the engine's first sync has indexed every mailbox in `ids`
    /// (`mailwomanSynced`): `Mailbox/empty` counts the cache and refuses a
    /// folder it has not synchronised yet.
    async fn wait_synced(&self, ids: &[&str]) {
        for _ in 0..120 {
            let got = self
                .call(
                    "Mailbox/get",
                    json!({ "ids": ids, "properties": ["id", "mailwomanSynced"] }),
                )
                .await;
            let list = got["list"].as_array().cloned().unwrap_or_default();
            if list.len() == ids.len() && list.iter().all(|m| m["mailwomanSynced"] == true) {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        panic!("mailboxes {ids:?} never finished their first sync");
    }

    /// The web client's loop: preview, then batches until empty or no progress.
    async fn empty(&self, mailbox: &str, outcome: &str, limit: u64) -> Vec<Value> {
        let preview = self
            .call(
                "Mailbox/empty",
                json!({ "mailboxId": mailbox, "outcome": outcome, "expectedTotal": 0, "limit": 0 }),
            )
            .await;
        let mut expected = preview["total"]
            .as_u64()
            .or_else(|| preview["error"]["mailwomanTotal"].as_u64())
            .unwrap_or_else(|| panic!("preview: {preview}"));
        let mut out = Vec::new();
        loop {
            let r = self
                .call(
                    "Mailbox/empty",
                    json!({ "mailboxId": mailbox, "outcome": outcome, "expectedTotal": expected, "limit": limit }),
                )
                .await;
            let processed = r["processed"]
                .as_u64()
                .unwrap_or_else(|| panic!("batch: {r}"));
            expected = r["remaining"].as_u64().unwrap();
            out.push(r);
            if expected == 0 || processed == 0 || out.len() > 20 {
                return out;
            }
        }
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

/// The message count of `mailbox`, from a fresh `LIST` + `STATUS`.
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

async fn setup_server(addr: &str, folder: &str, child: &str, marker: &str) {
    let mut s = fresh(addr).await;
    let names: Vec<String> = s
        .list_mailboxes()
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.mailbox_ref.name)
        .collect();
    if !names.iter().any(|n| n == "Trash") {
        s.create_mailbox("Trash").await.unwrap();
    }
    s.create_mailbox(folder).await.unwrap();
    s.create_mailbox(child).await.unwrap();
    let raw = |box_: &str, i: usize| {
        format!(
            "Message-ID: <{marker}-{box_}-{i}@t45.test>\r\n\
             From: alice@example.org\r\n\
             To: {USER}@example.org\r\n\
             Subject: {marker} {box_} {i}\r\n\
             Date: Mon, 06 Jul 2026 09:00:00 +0000\r\n\
             \r\n\
             body {i}\r\n"
        )
    };
    for i in 0..7 {
        s.append(folder, raw("parent", i).as_bytes(), &[])
            .await
            .unwrap();
    }
    for i in 0..2 {
        s.append(child, raw("child", i).as_bytes(), &[])
            .await
            .unwrap();
    }
    s.append("INBOX", raw("inbox", 0).as_bytes(), &[])
        .await
        .unwrap();
    let _ = s.logout().await;
}

#[tokio::test]
async fn a_folder_empties_into_trash_and_trash_empties_for_good_on_the_server() {
    if imap_servers().is_empty() {
        common::gate::skip(format_args!(
            "[t45 empty] MW_T45_IMAP unset: no live IMAP server driven"
        ));
        return;
    }
    let _env = serial().await;
    for addr in imap_servers() {
        let marker = stem("M");
        let folder = stem("Box");
        let mut s = fresh(&addr).await;
        let d = s.hierarchy_delimiter().await.unwrap().expect("hierarchy");
        let _ = s.logout().await;
        let child = format!("{folder}{d}Kid");
        // Filed before login, so the login's sync caches it.
        setup_server(&addr, &folder, &child, &marker).await;
        let trash_before = server_total(&addr, "Trash").await.unwrap();
        let inbox_before = server_total(&addr, "INBOX").await.unwrap();
        assert_eq!(server_total(&addr, &folder).await, Some(7), "precondition");

        let b = Browser::login(&format!("imap://{addr}")).await;
        let session = b.session().await;
        let cap = &session["accounts"][&b.account]["accountCapabilities"]["urn:ietf:params:jmap:mail"]
            ["mailwomanEmptyMailbox"];
        assert_eq!(cap["moveToTrash"], true, "{addr}: {session}");
        assert_eq!(cap["destroy"], true, "{addr}: {session}");
        let parent_id = b.mailbox_named(&folder).await;
        let trash_id = b.mailbox_named("Trash").await;
        b.wait_synced(&[&parent_id, &trash_id]).await;

        // A stale count is refused and nothing moves.
        let stale = b
            .call(
                "Mailbox/empty",
                json!({ "mailboxId": parent_id, "outcome": "trash", "expectedTotal": 70, "limit": 3 }),
            )
            .await;
        assert_eq!(stale["error"]["type"], "stateMismatch", "{addr}: {stale}");
        assert_eq!(server_total(&addr, &folder).await, Some(7));

        // The ordinary folder: three bounded batches into Trash.
        let batches = b.empty(&parent_id, "trash", 3).await;
        let processed: Vec<u64> = batches
            .iter()
            .map(|r| r["processed"].as_u64().unwrap())
            .collect();
        assert_eq!(processed, [3, 3, 1], "{addr}: {batches:?}");
        assert_eq!(batches[0]["trashId"], json!(trash_id));
        assert_eq!(server_total(&addr, &folder).await, Some(0), "{addr}");
        assert_eq!(
            server_total(&addr, &child).await,
            Some(2),
            "{addr}: the child folder is not touched"
        );
        let in_trash = server_messages(&addr, "Trash").await;
        assert_eq!(
            in_trash
                .iter()
                .filter(|m| m.contains(&format!("{marker} parent")))
                .count(),
            7,
            "{addr}: every message reached Trash"
        );
        assert_eq!(server_total(&addr, "Trash").await, Some(trash_before + 7));

        // Trash: permanent, in batches of 4.
        let batches = b.empty(&trash_id, "destroy", 4).await;
        assert!(
            batches.iter().all(|r| r["outcome"] == "destroy"),
            "{addr}: {batches:?}"
        );
        assert_eq!(
            batches.last().unwrap()["remaining"],
            0,
            "{addr}: {batches:?}"
        );
        assert_eq!(server_total(&addr, "Trash").await, Some(0), "{addr}");
        assert_eq!(server_total(&addr, &child).await, Some(2));
        assert_eq!(server_total(&addr, "INBOX").await, Some(inbox_before));
    }
}
