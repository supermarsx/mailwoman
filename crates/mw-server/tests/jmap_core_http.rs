//! Domain-selected upstreams and pre-authentication domain refusal over HTTP.
use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
use mw_store::{DomainConfigRow, ServerKey, Store};
use serde_json::{Value, json};
mod common;
use common::{mock_mail::MockPop3, test_db};

static TEST_ENV: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
struct UploadEnv(Option<std::ffi::OsString>);
impl Drop for UploadEnv {
    fn drop(&mut self) {
        unsafe {
            match self.0.take() {
                Some(value) => std::env::set_var("MW_UPLOAD_DIR", value),
                None => std::env::remove_var("MW_UPLOAD_DIR"),
            }
        }
    }
}

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

fn row(name: &str, mail: &MockPop3) -> DomainConfigRow {
    DomainConfigRow {
        name: name.into(),
        upstream_json: json!({"pop3":{"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","allowPrivate":true}}).to_string(),
        allowlist_json: "[]".into(), blocklist_json: "[]".into(),
        enabled: true, preferred: false, settings_json: "{}".into(), updated_at: 1,
    }
}

async fn server(store_path: &str, fallback: &MockPop3) -> String {
    // These cases exercise domain routing and authentication, not retry timing.
    let store = Store::open(store_path, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "domain-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    let config = AppConfig {
        db_path: store_path.into(),
        server_key_hex: Some(KEY.into()),
        mode: ServerMode::Engine,
        cookie_secure: false,
        web_dir: None,
        hardening: Default::default(),
        security: SecurityConfig {
            upstream_default: Some(fallback.url()),
            ..Default::default()
        },
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

async fn login(client: &reqwest::Client, base: &str, username: &str) -> reqwest::Response {
    client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":username,"password":"testpass"}))
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn native_core_enforces_envelopes_and_reports_method_errors_without_stopping_batch() {
    let _serial = TEST_ENV.lock().await;
    let mail = MockPop3::start("alice@example.org", "testpass").await;
    let db = test_db::unique_dir("jmap-core-http")
        .join("db.sqlite")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    store
        .put_domain_config(&row("example.org", &mail))
        .await
        .unwrap();
    let base = server(&db, &mail).await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    assert_eq!(
        login(&client, &base, "alice@example.org").await.status(),
        200
    );
    let api = format!("{base}/jmap/api");
    let session: Value = client
        .get(format!("{base}/jmap/session"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let limits = &session["capabilities"]["urn:ietf:params:jmap:core"];
    assert_eq!(
        limits["maxSizeRequest"],
        mw_engine::jmap::core::MAX_SIZE_REQUEST
    );
    assert_eq!(
        limits["maxCallsInRequest"],
        mw_engine::jmap::core::MAX_CALLS
    );
    assert_eq!(limits["maxConcurrentUpload"], 4);
    let excess=client.post(&api).json(&json!({"using":["urn:ietf:params:jmap:core"],"methodCalls":vec![json!(["Core/echo",{},"x"]);mw_engine::jmap::core::MAX_CALLS+1]})).send().await.unwrap();
    assert_eq!(excess.status(), 400);
    assert_eq!(
        excess.json::<Value>().await.unwrap()["limit"],
        "maxCallsInRequest"
    );

    for (request, kind) in [
        (json!({"methodCalls":[]}), "notRequest"),
        (
            json!({"using":["unsupported"],"methodCalls":[]}),
            "unknownCapability",
        ),
        (
            json!({"using":[],"methodCalls":[["Core/echo",{},"ok"],["Core/echo",[],"invalid"]]}),
            "notRequest",
        ),
    ] {
        let response = client.post(&api).json(&request).send().await.unwrap();
        assert_eq!(response.status(), 400);
        assert!(
            response.headers()["content-type"]
                .to_str()
                .unwrap()
                .starts_with("application/problem+json")
        );
        let body: Value = response.json().await.unwrap();
        assert_eq!(body["type"], format!("urn:ietf:params:jmap:error:{kind}"));
    }
    let response = client
        .post(&api)
        .body("{bad json")
        .header("content-type", "application/json")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 400);
    assert_eq!(
        response.json::<Value>().await.unwrap()["type"],
        "urn:ietf:params:jmap:error:notJSON"
    );
    let response=client.post(&api).json(&json!({"using":["urn:ietf:params:jmap:core"],"createdIds":{"prior":"stable"},"methodCalls":[["Core/echo",{"type":"client-value"},"a"],["NoSuch/method",{},"b"],["Core/echo",{"after":true},"c"]]})).send().await.unwrap();
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(
        body["methodResponses"][0],
        json!(["Core/echo",{"type":"client-value"},"a"])
    );
    assert_eq!(body["methodResponses"][1][0], "error");
    assert_eq!(body["methodResponses"][1][1]["type"], "unknownMethod");
    assert_eq!(body["methodResponses"][2][1], json!({"after":true}));
    assert_eq!(body["createdIds"], json!({"prior":"stable"}));
    let response = client
        .post(&api)
        .body(" ".repeat(mw_engine::jmap::core::MAX_SIZE_REQUEST + 1))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 400);
    assert_eq!(
        response.json::<Value>().await.unwrap()["limit"],
        "maxSizeRequest"
    );

    for (state, expected) in [
        (json!("definitely-stale"), "stateMismatch"),
        (json!(42), "invalidArguments"),
    ] {
        let response: Value = client.post(&api).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/set",{"ifInState":state,"destroy":["must-not-be-dispatched"]},"guard"]]})).send().await.unwrap().json().await.unwrap();
        assert_eq!(response["methodResponses"][0][0], "error");
        assert_eq!(response["methodResponses"][0][1]["type"], expected);
    }
}

#[tokio::test]
async fn fresh_trash_state_destroys_the_owned_message_and_stale_state_cannot() {
    let _serial = TEST_ENV.lock().await;
    let _restore = UploadEnv(std::env::var_os("MW_UPLOAD_DIR"));
    let uploads = test_db::unique_dir("core-trash-uploads");
    unsafe {
        std::env::set_var("MW_UPLOAD_DIR", &uploads);
    }

    let mail =
        common::mock_imap::MockImap::start_single_trash("alice@example.org", "testpass").await;
    let fallback = MockPop3::start("unused", "unused").await;
    let db = test_db::unique_dir("jmap-trash-http")
        .join("db.sqlite")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    let mut domain = row("example.org", &fallback);
    domain.upstream_json=json!({"imap":{"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","allowPrivate":true}}).to_string();
    store.put_domain_config(&domain).await.unwrap();
    let base = server(&db, &fallback).await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let signed: Value = login(&client, &base, "alice@example.org")
        .await
        .json()
        .await
        .unwrap();
    let account = signed["accountId"].as_str().unwrap();
    async fn call(client: &reqwest::Client, base: &str, method: &str, args: Value) -> Value {
        let response=client.post(format!("{base}/jmap/api")).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[[method,args,"call"]]})).send().await.unwrap();
        assert_eq!(response.status(), 200);
        response.json::<Value>().await.unwrap()["methodResponses"][0][1].clone()
    }
    let boxes = call(&client, &base, "Mailbox/get", json!({"accountId":account})).await;
    let trash = boxes["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["role"] == "trash")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    let upload:Value=client.post(format!("{base}/jmap/upload/{account}")).header("content-type","message/rfc822").body("From: sender@example.net\r\nTo: alice@example.org\r\nSubject: Isolated trash fixture\r\nMessage-ID: <isolated-trash@example.net>\r\n\r\nDelete only this controlled test message.\r\n").send().await.unwrap().json().await.unwrap();
    let imported=call(&client,&base,"Email/import",json!({"accountId":account,"emails":{"owned":{"blobId":upload["blobId"],"mailboxIds":{trash:true}}}})).await;
    let id = imported["created"]["owned"]["id"]
        .as_str()
        .expect("real IMAP import");
    assert_eq!(mail.messages.lock().await.len(), 1);
    let refusal = call(
        &client,
        &base,
        "Email/set",
        json!({"accountId":account,"ifInState":"stale","destroy":[id]}),
    )
    .await;
    assert_eq!(refusal["type"], "stateMismatch");
    assert_eq!(
        mail.messages.lock().await.len(),
        1,
        "stale state cannot issue expunge"
    );
    let preview = call(
        &client,
        &base,
        "Email/get",
        json!({"accountId":account,"ids":[id],"properties":["id","mailboxIds"]}),
    )
    .await;
    assert_eq!(preview["list"][0]["mailboxIds"][trash], true);
    let destroyed = call(
        &client,
        &base,
        "Email/set",
        json!({"accountId":account,"ifInState":preview["state"],"destroy":[id]}),
    )
    .await;
    assert_eq!(destroyed["destroyed"], json!([id]), "{destroyed}");
    assert!(
        mail.messages.lock().await.is_empty(),
        "real upstream UID EXPUNGE completed"
    );
    let missing = call(
        &client,
        &base,
        "Email/get",
        json!({"accountId":account,"ids":[id]}),
    )
    .await;
    assert_eq!(missing["notFound"], json!([id]));
}
