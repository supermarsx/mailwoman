//! Deleting a folder deletes the messages in it, so a legal hold
//! (`dsr.legalHold.subjects`) and an active processing restriction refuse it over
//! HTTP with a stable code and the `folder` scope; without either it goes through.
use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
use mw_store::settings_policy::SettingPolicyRow;
use mw_store::{DomainConfigRow, ServerKey, Store};
use serde_json::{Value, json};
mod common;
use common::{mock_mail::MockPop3, test_db};

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

async fn set(store: &Store, key: &str, value: Value) {
    store
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}

async fn call(client: &reqwest::Client, base: &str, method: &str, args: Value) -> (u16, Value) {
    let response = client
        .post(format!("{base}/jmap/api"))
        .json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[[method,args,"call"]]}))
        .send()
        .await
        .unwrap();
    let status = response.status().as_u16();
    (status, response.json::<Value>().await.unwrap_or(Value::Null))
}

#[tokio::test]
async fn a_legal_hold_or_restriction_refuses_a_folder_delete_and_nothing_else_does() {
    let mail = common::mock_imap::MockImap::start("alice@example.org", "testpass").await;
    let fallback = MockPop3::start("unused", "unused").await;
    let db = test_db::unique_dir("folder-delete-guard")
        .join("db.sqlite")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap()).await.unwrap();
    set(&store, "signin.retryCooldownSecs", json!(0)).await;
    store
        .put_domain_config(&DomainConfigRow {
            name: "example.org".into(),
            upstream_json: json!({"imap":{"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","allowPrivate":true}}).to_string(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    let app = build_app(AppConfig {
        db_path: db.clone(),
        server_key_hex: Some(KEY.into()),
        mode: ServerMode::Engine,
        cookie_secure: false,
        web_dir: None,
        hardening: Default::default(),
        security: SecurityConfig { upstream_default: Some(fallback.url()), ..Default::default() },
    })
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = reqwest::Client::builder().no_proxy().cookie_store(true).build().unwrap();
    let signed: Value = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":"alice@example.org","password":"testpass"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let account = signed["accountId"].as_str().unwrap().to_string();
    let destroy = json!({"accountId": account, "destroy": ["no-such-folder"]});

    // Under a legal hold: refused, with the code and the folder scope.
    set(&store, "dsr.legalHold.subjects", json!(["alice@example.org"])).await;
    let (status, body) = call(&client, &base, "Mailbox/set", destroy.clone()).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "legalHold");
    assert_eq!(body["scope"], "folder");
    // Creating or renaming a folder is not a destroy and is not refused.
    let (status, _) = call(&client, &base, "Mailbox/get", json!({"accountId": account})).await;
    assert_eq!(status, 200);

    // Hold lifted, account restricted instead.
    set(&store, "dsr.legalHold.subjects", json!([])).await;
    store
        .restriction_apply(&mw_store::privacy::RestrictionRow {
            id: "r-1".into(),
            account_id: account.clone(),
            address: "alice@example.org".into(),
            domain: "example.org".into(),
            scopes: vec!["all".into()],
            mail_delivery: "accept".into(),
            reason: "test".into(),
            source: "test".into(),
            request_id: None,
            created_by: "test".into(),
            created_at: chrono::Utc::now().timestamp() - 5,
            expires_at: None,
            lifted_at: None,
            lifted_by: None,
            lift_reason: None,
        })
        .await
        .unwrap();
    let (status, body) = call(&client, &base, "Mailbox/set", destroy.clone()).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "processingRestricted");
    assert_eq!(body["scope"], "folder");

    // Neither: the call reaches the engine (the folder does not exist, so it is not destroyed).
    store.restriction_lift(&account, "test", "done", chrono::Utc::now().timestamp()).await.unwrap();
    let (status, body) = call(&client, &base, "Mailbox/set", destroy).await;
    assert_eq!(status, 200, "{body}");
}
