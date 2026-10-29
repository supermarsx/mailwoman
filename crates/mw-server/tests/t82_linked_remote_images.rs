//! t82: "Could not update remote image permissions. Please try again."
//!
//! Root cause: with a linked mailbox open, every request carries
//! `x-mailwoman-link`, and the remote-image grant routes were not in the linked
//! scope, so they were refused (`403 linkedContextUnsupported`; the web client
//! refused them before sending). The grant belongs to the mailbox being read.
//!
//! Real router over HTTP, engine mode, real sign-ins, step-up and link (the
//! t70 fixture). Covered:
//!   - in the child's context, grant/list/revoke answer 200 and act on the
//!     CHILD's account; the parent's grants are untouched;
//!   - grant and revoke are idempotent (the client retries them on a transient
//!     failure): sending each twice leaves one active grant, then none;
//!   - the primary context still works, and an unknown scope is still refused.
//!
//! Run: cargo test -p mw-server --test t82_linked_remote_images -- --test-threads=1
mod common;

use std::net::SocketAddr;
use std::sync::Arc;

use common::mock_mail::MockPop3;
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::{DomainConfigRow, ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};

const KEY: &str = "82a1b2c3d4e5f6071829304a5b6c7d8e82a1b2c3d4e5f6071829304a5b6c7d8e";
const PARENT_PASS: &str = "Parent-Passw0rd!";
const CHILD_PASS: &str = "Child-Secret-t82-7c2a";

async fn store(db: &str) -> Store {
    Store::open(db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap()
}
async fn policy(db: &str, key: &str, value: &str) {
    store(db)
        .await
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "t82-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
}
async fn domain(db: &str, name: &str, addr: SocketAddr) {
    store(db)
        .await
        .put_domain_config(&DomainConfigRow {
            name: name.into(),
            upstream_json: json!({"pop3":{"host":"127.0.0.1","port":addr.port(),"tls":"none","allowPrivate":true}}).to_string(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}

async fn serve(db: &str) -> String {
    let config = AppConfig {
        db_path: db.into(),
        server_key_hex: Some(KEY.into()),
        web_dir: None,
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig {
            csrf_strict: true,
            ..Default::default()
        },
        security: SecurityConfig {
            jmap_upstreams: Some(Vec::new()),
            ..Default::default()
        },
    };
    let v6 = V6Config {
        admin_enabled: false,
        admin_username: None,
        admin_password: None,
        redis_url: None,
    };
    let (app, _push) = mw_server::build_app_full(config, v6).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap()
    });
    base
}

async fn json_of(response: reqwest::Response, expected: u16) -> Value {
    let status = response.status();
    let text = response.text().await.unwrap();
    assert_eq!(status, expected, "{text}");
    if text.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&text).unwrap_or(Value::String(text))
    }
}

async fn sign_in(base: &str, user: &str, pass: &str) -> reqwest::Client {
    let jar = Arc::new(reqwest::cookie::Jar::default());
    let bare = reqwest::Client::builder()
        .no_proxy()
        .cookie_provider(jar.clone())
        .build()
        .unwrap();
    let login = json_of(
        bare.post(format!("{base}/api/login"))
            .json(&json!({"username":user,"password":pass}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "x-csrf-token",
        login["csrfToken"].as_str().unwrap().parse().unwrap(),
    );
    headers.insert(
        "x-mailwoman-storage-authority",
        login["storageAuthority"].as_str().unwrap().parse().unwrap(),
    );
    reqwest::Client::builder()
        .no_proxy()
        .cookie_provider(jar)
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap()
}

async fn grants(http: &reqwest::Client, base: &str, link: Option<&str>) -> Value {
    let mut request = http.get(format!("{base}/api/remote-images/grants"));
    if let Some(link) = link {
        request = request.header("x-mailwoman-link", link);
    }
    json_of(request.send().await.unwrap(), 200).await
}
async fn change(
    http: &reqwest::Client,
    base: &str,
    link: Option<&str>,
    action: &str,
    body: Value,
    expected: u16,
) -> Value {
    let mut request = http
        .post(format!("{base}/api/remote-images/{action}"))
        .json(&body);
    if let Some(link) = link {
        request = request.header("x-mailwoman-link", link);
    }
    json_of(request.send().await.unwrap(), expected).await
}
fn scopes(listing: &Value) -> Vec<(String, String)> {
    listing["list"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| {
            (
                g["scopeKind"].as_str().unwrap().to_string(),
                g["scopeValue"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[tokio::test]
async fn remote_image_grants_work_in_a_linked_mailbox_and_are_idempotent() {
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
    }
    let parents = MockPop3::start_accounts(&[("parent@example.org", PARENT_PASS)]).await;
    let child = MockPop3::start_accounts(&[("child@child.test", CHILD_PASS)]).await;
    let dir = common::test_db::unique_dir("mw-t82");
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    policy(&db, "signin.retryCooldownSecs", "0").await;
    policy(&db, "accounts.linkingEnabled", "true").await;
    domain(&db, "example.org", parents.addr).await;
    domain(&db, "child.test", child.addr).await;
    let base = serve(&db).await;

    let parent = sign_in(&base, "parent@example.org", PARENT_PASS).await;
    json_of(
        parent
            .post(format!("{base}/api/security/step-up"))
            .json(&json!({"password":PARENT_PASS}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let linked = json_of(
        parent
            .post(format!("{base}/api/accounts/link"))
            .json(
                &json!({"username":"child@child.test","password":CHILD_PASS,"displayName":"Child"}),
            )
            .send()
            .await
            .unwrap(),
        201,
    )
    .await;
    let link = linked["account"]["id"].as_str().unwrap().to_string();
    let link = Some(link.as_str());

    // The child's context: the routes answer, and act on the child's account.
    let child_before = grants(&parent, &base, link).await;
    let parent_before = grants(&parent, &base, None).await;
    assert_ne!(
        child_before["accountId"], parent_before["accountId"],
        "the linked context lists the child's grants"
    );
    let sender = json!({"scopeKind":"per-sender","scopeValue":"news@linked-news.test"});
    // Sent twice, as a retry after a lost answer would be: one grant.
    for _ in 0..2 {
        let ok = change(&parent, &base, link, "grant", sender.clone(), 200).await;
        assert_eq!(ok["ok"], true);
    }
    assert_eq!(
        scopes(&grants(&parent, &base, link).await),
        vec![(
            "per-sender".to_string(),
            "news@linked-news.test".to_string()
        )]
    );
    assert!(
        scopes(&grants(&parent, &base, None).await).is_empty(),
        "the parent's grants are untouched"
    );
    // Revoked twice: none.
    for _ in 0..2 {
        change(&parent, &base, link, "revoke", sender.clone(), 200).await;
    }
    assert!(scopes(&grants(&parent, &base, link).await).is_empty());

    // The primary context still works; an unknown scope is still refused.
    change(
        &parent,
        &base,
        None,
        "grant",
        json!({"scopeKind":"single","scopeValue":"m1"}),
        200,
    )
    .await;
    assert_eq!(
        scopes(&grants(&parent, &base, None).await),
        vec![("single".to_string(), "m1".to_string())]
    );
    assert!(scopes(&grants(&parent, &base, link).await).is_empty());
    change(
        &parent,
        &base,
        link,
        "grant",
        json!({"scopeKind":"everything","scopeValue":""}),
        400,
    )
    .await;
}
