//! t28-e18 (f34) — an API key limited to some folders sees only those folders on
//! `/api/v1`.
//!
//! Before 26.20 the REST guard asked the key for no folder at all, and
//! `Subset([])` was covered by every folder grant, so a key limited to one folder
//! read the whole mailbox: `?mailbox=` outside the grant, the list without a
//! mailbox, a message by id, the mailbox list.
//!
//! The real router in proxy mode over the mock JMAP server. One message is moved to
//! the Sent folder first, so there is something on each side of the grant. Every
//! refusal is preceded by the same request succeeding for an all-folders key.
//!
//! Run:
//!   cargo test -p mw-server --test t28_rest_folder_scope --locked -- --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

mod common;
use common::test_db;

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T28_E18</div>";

fn web_dir() -> PathBuf {
    let dir = test_db::unique_dir("mw-t28-e18-web");
    std::fs::write(dir.join("index.html"), INDEX_HTML).unwrap();
    dir
}

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

async fn spawn_server(mock: &str) -> String {
    let config = AppConfig {
        db_path: test_db::unique_db_path("mw-t28-e18-rest")
            .to_string_lossy()
            .to_string(),
        server_key_hex: None,
        web_dir: Some(web_dir()),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(vec![mock.to_string()]),
            upstream_default: Some(mock.to_string()),
            ..SecurityConfig::default()
        },
    };
    let v6 = V6Config {
        admin_enabled: false,
        admin_username: None,
        admin_password: None,
        redis_url: None,
    };
    let app = build_app_full(config, v6).await.expect("server boots").0;
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
    format!("http://{addr}")
}

fn scope(account: &str, folders: Value) -> Value {
    json!({
        "read": true, "send": false, "delete": false,
        "accounts": { "subset": [account] }, "folders": folders,
        "mail": true, "pim": false, "ip_allowlist": [], "expires_at": null,
        "rate_limit": null, "mcp_tools": [], "unattended_send": false,
    })
}

async fn mint(c: &reqwest::Client, server: &str, account: &str, folders: Value) -> String {
    let m: Value = c
        .post(format!("{server}/api/keys"))
        .json(&json!({ "label": "t28-e18", "accountId": account, "scope": scope(account, folders) }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    m["displayToken"]
        .as_str()
        .unwrap_or_else(|| panic!("no shown-once token: {m}"))
        .to_string()
}

/// A key-only GET (no cookie): the key is the whole authority.
async fn get(server: &str, key: &str, path: &str) -> (u16, Value) {
    let r = reqwest::Client::new()
        .get(format!("{server}{path}"))
        .bearer_auth(key)
        .send()
        .await
        .unwrap();
    let status = r.status().as_u16();
    (status, r.json().await.unwrap_or(Value::Null))
}

fn ids(body: &Value, field: &str) -> Vec<String> {
    let mut v: Vec<String> = body[field]
        .as_array()
        .map(|l| {
            l.iter()
                .filter_map(|m| m["id"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

#[tokio::test]
async fn a_folder_scoped_key_sees_only_its_folders() {
    let mock = spawn_mock().await;
    let server = spawn_server(&mock).await;
    let c = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let login: Value = c
        .post(format!("{server}/api/login"))
        .json(&json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let account = login["accountId"].as_str().expect("login").to_string();

    // e3 moves to Sent: Inbox holds e1, e2; Sent holds e3.
    let moved: Value = c
        .post(format!("{server}/jmap/api"))
        .json(&json!({
            "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            "methodCalls": [["Email/set", {
                "accountId": mw_mock_jmap::ACCOUNT_ID,
                "update": { "e3": { "mailboxIds": { "mb-sent": true } } }
            }, "m"]]
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        moved["methodResponses"][0][1]["updated"]
            .as_object()
            .is_some_and(|u| u.contains_key("e3")),
        "setup: e3 moved to Sent: {moved}"
    );

    let all = mint(&c, &server, &account, json!("all")).await;
    let sent_only = mint(&c, &server, &account, json!({ "subset": ["mb-sent"] })).await;

    // 1. Naming a folder outside the grant. Control: the all-folders key reads it.
    let (s, body) = get(&server, &all, "/api/v1/messages?mailbox=mb-inbox").await;
    assert_eq!(s, 200, "control: an unscoped key reads Inbox");
    assert_eq!(ids(&body, "messages"), ["e1", "e2"]);
    let (s, _) = get(&server, &sent_only, "/api/v1/messages?mailbox=mb-inbox").await;
    assert_eq!(s, 403, "a key for Sent may not read Inbox");
    let (s, body) = get(&server, &sent_only, "/api/v1/messages?mailbox=mb-sent").await;
    assert_eq!(s, 200, "a key for Sent reads Sent");
    assert_eq!(ids(&body, "messages"), ["e3"]);

    // 2. The list without `?mailbox=`: only the granted folders. (The mock answers
    //    an unfiltered query with Inbox, which is what leaked before.)
    let (s, body) = get(&server, &all, "/api/v1/messages").await;
    assert_eq!(s, 200);
    assert!(
        !ids(&body, "messages").is_empty(),
        "control: an unscoped list returns mail"
    );
    let (s, body) = get(&server, &sent_only, "/api/v1/messages").await;
    assert_eq!(s, 200);
    assert_eq!(
        ids(&body, "messages"),
        ["e3"],
        "the list holds Sent's mail only: {body}"
    );

    // 3. A message by id outside the grant reads as missing.
    let (s, _) = get(&server, &all, "/api/v1/messages/e1").await;
    assert_eq!(s, 200, "control: an unscoped key reads e1");
    let (s, _) = get(&server, &sent_only, "/api/v1/messages/e1").await;
    assert_eq!(s, 404, "e1 is in Inbox, outside the key's folders");
    let (s, body) = get(&server, &sent_only, "/api/v1/messages/e3").await;
    assert_eq!(s, 200, "e3 is in Sent: {body}");

    // 4. The mailbox list holds the granted folders only.
    let (s, body) = get(&server, &all, "/api/v1/mailboxes").await;
    assert_eq!(s, 200);
    assert_eq!(ids(&body, "mailboxes"), ["mb-inbox", "mb-sent"]);
    let (s, body) = get(&server, &sent_only, "/api/v1/mailboxes").await;
    assert_eq!(s, 200);
    assert_eq!(ids(&body, "mailboxes"), ["mb-sent"]);

    // 5. A key granted no folder at all reads nothing.
    let none = mint(&c, &server, &account, json!({ "subset": [] })).await;
    let (s, body) = get(&server, &none, "/api/v1/messages").await;
    assert_eq!(s, 200);
    assert!(ids(&body, "messages").is_empty(), "{body}");
}
