//! Real HTTP import through a writable IMAP socket, including durable APPENDUID.
mod common;
use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
use serde_json::{Value, json};
const KEY: &str = "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";
async fn jmap(client: &reqwest::Client, base: &str, method: &str, args: Value) -> Value {
    let response=client.post(format!("{base}/jmap/api")).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[[method,args,"a"]]})).send().await.unwrap();
    let status = response.status();
    let value: Value = response.json().await.unwrap();
    assert_eq!(status, 200, "{value}");
    value["methodResponses"][0][1].clone()
}
#[tokio::test]
async fn import_writes_upstream_and_exports_under_existing_step_up_policy() {
    let mail = common::mock_imap::MockImap::start("owner@example.org", "test-password").await;
    let root = common::test_db::unique_dir("durable-import-http");
    let uploads = root.join("uploads");
    std::fs::create_dir_all(&uploads).unwrap();
    // One test in this binary owns these startup-only fixture variables.
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
        std::env::set_var("MW_UPLOAD_DIR", &uploads);
    }
    let app = build_app(AppConfig {
        db_path: root.join("db.sqlite").to_string_lossy().into_owned(),
        server_key_hex: Some(KEY.into()),
        web_dir: None,
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: Default::default(),
        security: SecurityConfig {
            upstream_default: Some(mail.url()),
            ..Default::default()
        },
    })
    .await
    .unwrap();
    let socket = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", socket.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            socket,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .unwrap();
    let response = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":"owner@example.org","password":"test-password"}))
        .send()
        .await
        .unwrap();
    let status = response.status();
    let login: Value = response.json().await.unwrap();
    assert_eq!(status, 200, "{login}");
    let account = login["accountId"].as_str().unwrap();
    let formats: Value = client
        .get(format!("{base}/api/import/formats"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(formats["available"], true, "{formats}");
    let boxes = jmap(&client, &base, "Mailbox/get", json!({"accountId":account})).await;
    let inbox = boxes["list"][0]["id"].as_str().unwrap();
    let raw=b"Message-ID: <durable-import@example.org>\r\nFrom: sender@example.org\r\nTo: owner@example.org\r\nSubject: Durable import\r\n\r\nHistorical body\r\n";
    let refused = client
        .post(format!("{base}/api/import/eml?mailboxId=not-owned"))
        .body(raw.as_slice())
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), 400);
    assert!(mail.messages.lock().await.is_empty());
    let response = client
        .post(format!("{base}/api/import/eml?mailboxId={inbox}"))
        .header("content-type", "message/rfc822")
        .body(raw.as_slice())
        .send()
        .await
        .unwrap();
    let status = response.status();
    let imported: Value = response.json().await.unwrap();
    assert_eq!(status, 200, "{imported}");
    assert_eq!(imported["imported"], 1, "{imported}");
    assert_eq!(mail.messages.lock().await.as_slice(), &[raw.to_vec()]);
    let queried = jmap(
        &client,
        &base,
        "Email/query",
        json!({"filter":{"inMailbox":inbox}}),
    )
    .await;
    let id = queried["ids"][0].as_str().unwrap();
    let refused = client
        .get(format!("{base}/api/export/{id}?format=eml"))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), 403);
    let proof = client
        .post(format!("{base}/api/security/step-up"))
        .json(&json!({"password":"test-password"}))
        .send()
        .await
        .unwrap();
    assert_eq!(proof.status(), 200, "{}", proof.text().await.unwrap());
    let exported = client
        .get(format!("{base}/api/export/{id}?format=eml"))
        .send()
        .await
        .unwrap();
    assert_eq!(exported.status(), 200);
    assert_eq!(exported.bytes().await.unwrap().as_ref(), raw);
    let before = mail.messages.lock().await.len();
    let wrong_state = jmap(
        &client,
        &base,
        "Email/import",
        json!({"ifInState":"stale","emails":{"x":{"blobId":id,"mailboxIds":{inbox:true}}}}),
    )
    .await;
    assert_eq!(wrong_state["type"], "stateMismatch");
    assert_eq!(mail.messages.lock().await.len(), before);
}
