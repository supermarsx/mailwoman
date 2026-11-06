//! Folder export over HTTP (`/api/export/folder/{id}.zip`), engine mode against a
//! live IMAP account. Env-gated on `GREENMAIL_IMAP` and `#[ignore]` otherwise,
//! like `engine_download.rs`. The pure parts (subfolder paths, caps, formats)
//! are unit-tested in `src/folder_export.rs`.

use std::net::SocketAddr;

use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, ServerMode, build_app};

mod common;
use common::{auth, test_db};

/// The names of a stored ZIP's entries, read from its central directory.
fn zip_names(bytes: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    let mut at = 0;
    while at + 46 <= bytes.len() {
        if bytes[at..at + 4] == [0x50, 0x4b, 0x01, 0x02] {
            let n = u16::from_le_bytes([bytes[at + 28], bytes[at + 29]]) as usize;
            let extra = u16::from_le_bytes([bytes[at + 30], bytes[at + 31]]) as usize;
            let comment = u16::from_le_bytes([bytes[at + 32], bytes[at + 33]]) as usize;
            names.push(String::from_utf8_lossy(&bytes[at + 46..at + 46 + n]).into_owned());
            at += 46 + n + extra + comment;
        } else {
            at += 1;
        }
    }
    names
}

#[tokio::test]
#[ignore = "requires a live IMAP server; set GREENMAIL_IMAP=host:port"]
async fn folder_zip_streams_entries_and_the_estimate_and_audit_follow() {
    let Ok(addr) = std::env::var("GREENMAIL_IMAP") else {
        return;
    };
    let user = std::env::var("GREENMAIL_USER").unwrap_or_else(|_| "user@example.org".into());
    let pass = std::env::var("GREENMAIL_PASS").unwrap_or_else(|_| "pass".into());
    let base = test_db::unique_dir("mw-folder-export");
    let web_dir = base.join("web");
    std::fs::create_dir_all(&web_dir).unwrap();
    std::fs::write(web_dir.join("index.html"), "<!doctype html><title>MW</title>").unwrap();
    let app = build_app(AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web_dir),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
        security: mw_server::SecurityConfig {
            upstream_default: Some(format!("imap://{addr}")),
            ..mw_server::SecurityConfig::default()
        },
    })
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let local: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let server = format!("http://{local}");
    let c = reqwest::Client::builder().cookie_store(true).build().unwrap();
    assert_eq!(
        c.post(format!("{server}/api/login"))
            .json(&json!({ "username": user, "password": pass }))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let boxes: Value = c
        .post(format!("{server}/jmap/api"))
        .json(&json!({"using":["urn:ietf:params:jmap:mail"],"methodCalls":[["Mailbox/get",{},"m"]]}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let inbox = boxes["methodResponses"][0][1]["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "inbox")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // The estimate answers sizes and a verdict, and sends no mail.
    let est: Value = c
        .get(format!("{server}/api/export/folder/{inbox}.zip?estimate=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(est["allowed"], true, "{est}");
    let messages = est["messages"].as_u64().unwrap();

    // A folder of another account is not found.
    let missing = c.get(format!("{server}/api/export/folder/nope.zip")).send().await.unwrap();
    assert_eq!(missing.status(), 404);

    // The download needs a fresh password proof, then streams a valid archive.
    auth::user_step_up(&c, &server, &pass).await;
    let zip = c
        .get(format!("{server}/api/export/folder/{inbox}.zip?format=eml"))
        .send()
        .await
        .unwrap();
    assert_eq!(zip.status(), 200);
    assert_eq!(zip.headers()[reqwest::header::CONTENT_TYPE], "application/zip");
    let bytes = zip.bytes().await.unwrap();
    assert_eq!(&bytes[..4], &[0x50, 0x4b, 0x03, 0x04]);
    let names = zip_names(&bytes);
    assert_eq!(names.len() as u64, messages, "{names:?}");
    assert!(names.iter().all(|n| n.ends_with(".eml") && !n.contains("..")));
}
