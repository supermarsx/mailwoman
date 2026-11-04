//! Real HTTP saves use IMAP APPEND; replay and refusals cannot write again.
mod common;
use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
use serde_json::{Value, json};
#[tokio::test]
async fn drafts_confirm_upstream_and_fence_replay_authority_and_partial_content() {
    let mail =
        common::mock_imap::MockImap::start_drafts("owner@example.org", "test-password").await;
    let root = common::test_db::unique_dir("account-drafts-http");
    let uploads = root.join("uploads");
    std::fs::create_dir_all(&uploads).unwrap();
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
        std::env::set_var("MW_UPLOAD_DIR", &uploads);
    }
    let app = build_app(AppConfig {
        db_path: root.join("db.sqlite").to_string_lossy().into(),
        server_key_hex: Some(
            "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b".into(),
        ),
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
        .timeout(std::time::Duration::from_secs(30))
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
    let authority = login["storageAuthority"].as_str().unwrap();
    let upload: Value = client
        .post(format!(
            "{base}/jmap/upload/{}",
            login["accountId"].as_str().unwrap()
        ))
        .header("content-type", "text/plain")
        .body("durable attachment")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let blob = upload["blobId"]
        .as_str()
        .unwrap_or_else(|| panic!("{upload}"));
    let endpoint = format!("{base}/api/account/drafts/draft-test");
    let input = json!({"operationId":"operation-one","expectedRevision":null,"state":{"version":1,"bodyFormat":"plain","subject":"Saved across devices","bodyText":"Never sent","to":"incomplete, valid@example.test","bcc":"private@example.test","attachments":[{"blobId":blob,"name":"note.txt","size":1,"contentType":"text/plain"}]}});
    assert_eq!(
        client
            .put(&endpoint)
            .json(&input)
            .send()
            .await
            .unwrap()
            .status(),
        428
    );
    assert_eq!(
        client
            .put(&endpoint)
            .header("x-mailwoman-storage-authority", "other-realm")
            .json(&input)
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    let response = client
        .put(&endpoint)
        .header("x-mailwoman-storage-authority", authority)
        .json(&input)
        .send()
        .await
        .unwrap();
    let status = response.status();
    let saved: Value = response.json().await.unwrap();
    assert_eq!(status, 200, "{saved}");
    assert_eq!(saved["status"], "saved");
    assert!(saved["revision"].is_string());
    assert!(saved["emailId"].is_string());
    assert_eq!(saved["state"]["to"], "incomplete, valid@example.test");
    assert!(saved["state"].get("editorMode").is_none());
    assert!(saved["state"].get("source").is_none());
    assert!(saved["summary"].get("source").is_none());
    assert!(
        saved["state"]["attachments"][0]["blobId"]
            .as_str()
            .unwrap()
            .starts_with(&format!("{}.", saved["emailId"].as_str().unwrap()))
    );
    assert_eq!(saved["state"]["attachments"][0]["size"], 18);

    {
        let raw = mail.messages.lock().await;
        assert_eq!(raw.len(), 1);
        let text = String::from_utf8_lossy(&raw[0]);
        assert!(text.contains("Never sent"));
        assert!(text.contains("valid@example.test"));
        assert!(!text.contains("incomplete"));
    }
    let response = client
        .put(&endpoint)
        .header("x-mailwoman-storage-authority", authority)
        .json(&input)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        mail.messages.lock().await.len(),
        1,
        "same operation must not append twice"
    );
    let loaded = client.get(&endpoint).send().await.unwrap();
    assert_eq!(loaded.headers()["cache-control"], "no-store");
    let loaded: Value = loaded.json().await.unwrap();
    assert_eq!(loaded["state"]["bcc"], "private@example.test");
    assert_eq!(loaded["revision"], saved["revision"]);
    assert!(loaded["state"].get("editorMode").is_none());
    let mut conflict = input.clone();
    conflict["operationId"] = json!("other-device");
    conflict["state"]["bodyText"] = json!("stale replacement");
    assert_eq!(
        client
            .put(&endpoint)
            .header("x-mailwoman-storage-authority", authority)
            .json(&conflict)
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    assert_eq!(mail.messages.lock().await.len(), 1);
    let mut protected = input.clone();
    protected["state"]["protectedContent"] = json!(true);
    assert_eq!(
        client
            .put(format!("{base}/api/account/drafts/protected"))
            .header("x-mailwoman-storage-authority", authority)
            .json(&protected)
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    assert_eq!(mail.messages.lock().await.len(), 1);
    let oversized = vec![b' '; 1024 * 1024 + 8193];
    assert_eq!(
        client
            .put(format!("{base}/api/account/drafts/oversized"))
            .header("x-mailwoman-storage-authority", authority)
            .body(oversized)
            .send()
            .await
            .unwrap()
            .status(),
        413
    );
    assert_eq!(mail.messages.lock().await.len(), 1);
    let resolved: Value = client
        .get(format!("{base}/api/account/drafts/resolve"))
        .query(&[("emailId", saved["emailId"].as_str().unwrap())])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resolved["draft"]["id"], "draft-test");
    let refused = client
        .delete(&endpoint)
        .header("x-mailwoman-storage-authority", authority)
        .json(&json!({"expectedRevision":"stale"}))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), 409);
    assert_eq!(mail.messages.lock().await.len(), 1);
    let deleted = client
        .delete(&endpoint)
        .header("x-mailwoman-storage-authority", authority)
        .json(&json!({"expectedRevision":saved["revision"]}))
        .send()
        .await
        .unwrap();
    let status = deleted.status();
    let deleted: Value = deleted.json().await.unwrap();
    assert_eq!(status, 200, "{deleted}");
    assert_eq!(deleted["deleted"], true);
    assert!(mail.messages.lock().await.is_empty());
    assert_eq!(client.get(&endpoint).send().await.unwrap().status(), 404);
    let anonymous = reqwest::Client::new().get(&endpoint).send().await.unwrap();
    assert_eq!(anonymous.status(), 401);

    // The HTML editing surface survives remote resume without changing the MIME
    // body or granting any sender authority. Legacy drafts above omit the field.
    let html = "<table><tbody><tr><td>Preserved HTML layout</td></tr></tbody></table>";
    for (mode, keyword) in [("rich", "$answered"), ("html", "$forwarded")] {
        let endpoint = format!("{base}/api/account/drafts/editor-{mode}");
        let original = if mode == "html" {
            "x".repeat(512)
        } else {
            format!("original-source-{mode}")
        };
        let source = json!({"emailId":original,"keyword":keyword});
        let input = json!({"operationId":format!("editor-{mode}"),"expectedRevision":null,"state":{"version":1,"bodyFormat":"html","editorMode":mode,"bodyHtml":html,"bodyText":"Preserved HTML layout","source":source}});
        let response = client
            .put(&endpoint)
            .header("x-mailwoman-storage-authority", authority)
            .json(&input)
            .send()
            .await
            .unwrap();
        let status = response.status();
        let saved: Value = response.json().await.unwrap();
        assert_eq!(status, 200, "{saved}");
        assert_eq!(saved["state"]["editorMode"], mode);
        assert_eq!(saved["state"]["source"], source);
        assert_eq!(saved["summary"]["source"], source);
        let loaded: Value = client
            .get(&endpoint)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(loaded["state"]["editorMode"], mode);
        assert_eq!(loaded["state"]["bodyHtml"], html);
        assert_eq!(loaded["state"]["source"], source);
        let listed: Value = client
            .get(format!("{base}/api/account/drafts"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let item = listed["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["id"] == format!("editor-{mode}"))
            .unwrap();
        assert_eq!(item["summary"]["source"], source);
        assert!(
            item.get("state").is_none(),
            "listing must not expose editor bodies"
        );
        {
            let raw = mail.messages.lock().await;
            assert_eq!(raw.len(), 1);
            assert!(!String::from_utf8_lossy(&raw[0]).contains("editorMode"));
            assert!(
                !String::from_utf8_lossy(&raw[0]).contains(original.as_str()),
                "draft source metadata must not enter unsent MIME"
            );
            assert!(
                !String::from_utf8_lossy(&raw[0]).contains(keyword),
                "saving a draft must not apply source keywords"
            );
        }
        let response = client
            .delete(&endpoint)
            .header("x-mailwoman-storage-authority", authority)
            .json(&json!({"expectedRevision":saved["revision"]}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        assert!(mail.messages.lock().await.is_empty());
    }
    for invalid in ["plain", "source", "HTML", ""] {
        let response = client
            .put(format!("{base}/api/account/drafts/invalid-editor"))
            .header("x-mailwoman-storage-authority", authority)
            .json(&json!({"operationId":"invalid-editor","expectedRevision":null,"state":{"version":1,"bodyFormat":"html","editorMode":invalid,"bodyHtml":html}}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 400, "{invalid}");
        assert_eq!(
            response.json::<Value>().await.unwrap()["error"],
            "invalidDraftState"
        );
        assert!(mail.messages.lock().await.is_empty());
    }
    for (source, expected) in [
        (json!({"emailId":"","keyword":"$answered"}), 400),
        (
            json!({"emailId":"x".repeat(513),"keyword":"$answered"}),
            400,
        ),
        (
            json!({"emailId":"bad\r\nsource","keyword":"$forwarded"}),
            400,
        ),
        (
            json!({"emailId":"bad\u{0000}source","keyword":"$answered"}),
            400,
        ),
        (json!({"emailId":"source","keyword":"$seen"}), 400),
        (
            json!({"emailId":"source","keyword":"$answered","unexpected":true}),
            400,
        ),
        (json!({"emailId":"source"}), 400),
        (json!({"emailId":7,"keyword":"$answered"}), 400),
        (json!("source"), 400),
    ] {
        let response = client
            .put(format!("{base}/api/account/drafts/invalid-source"))
            .header("x-mailwoman-storage-authority", authority)
            .json(&json!({"operationId":"invalid-source","expectedRevision":null,"state":{"version":1,"bodyFormat":"plain","bodyText":"Never sent","source":source}}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), expected, "{source}");
        assert!(
            mail.messages.lock().await.is_empty(),
            "refused metadata must never append a draft"
        );
    }
}
