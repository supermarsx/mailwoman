//! t39 L1 / decision D3: bounded mail-backend failover through the real login path.
//! A dead selected backend fails over to the next one; an authentication
//! rejection never reaches a second backend; the account keeps the selected
//! backend as its identity.
use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
use mw_store::{DomainConfigRow, ServerKey, Store};
use serde_json::{Value, json};
mod common;
use common::{mock_mail::MockPop3, test_db};

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

fn socket(port: u16) -> Value {
    json!({"host":"127.0.0.1","port":port,"tls":"none","allowPrivate":true,
        "security":{"mode":"fixed","allowed":["none"],"allowPlaintext":true}})
}

/// A loopback port with nothing listening on it.
async fn dead_port() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

async fn fixture(name: &str, primary: u16, secondary: u16) -> (Store, String, String) {
    let db = test_db::unique_dir(name)
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "failover-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    let config = json!({"version":2,"failover":true,"backends":[
        {"id":"primary","role":"mail","protocol":"pop3","priority":1,"endpoint":socket(primary)},
        {"id":"replica","role":"mail","protocol":"pop3","priority":2,"endpoint":socket(secondary)}
    ]});
    store
        .put_domain_config(&DomainConfigRow {
            name: "example.org".into(),
            upstream_json: config.to_string(),
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
        security: SecurityConfig::default(),
    })
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (store, format!("http://{addr}"), db)
}

async fn login(base: &str, password: &str) -> reqwest::Response {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap()
        .post(format!("{base}/api/login"))
        .json(&json!({"username":"alice@example.org","password":password}))
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn a_dead_selected_backend_fails_over_and_keeps_its_identity() {
    let primary = dead_port().await;
    let replica = MockPop3::start("alice@example.org", "testpass").await;
    let (store, base, _db) = fixture("t39-failover-dead", primary, replica.addr.port()).await;

    let response = login(&base, "testpass").await;
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    assert!(
        replica.logins_accepted() >= 1,
        "the replica received the login"
    );

    let account = store.list_accounts().await.unwrap().remove(0);
    assert_eq!(account.username, "alice@example.org");
    assert_eq!(
        (account.host.as_str(), account.port),
        ("127.0.0.1", primary),
        "the selected backend stays the account identity"
    );
}

#[tokio::test]
async fn an_authentication_rejection_never_fails_over() {
    let primary = MockPop3::start("alice@example.org", "testpass").await;
    let replica = MockPop3::start("alice@example.org", "wrong").await;
    let (store, base, _db) = fixture(
        "t39-failover-auth",
        primary.addr.port(),
        replica.addr.port(),
    )
    .await;

    assert_eq!(login(&base, "wrong").await.status(), 401);
    assert!(
        primary.logins_refused() > 0,
        "the selected backend refused it"
    );
    assert_eq!(
        replica.logins_accepted() + replica.logins_refused(),
        0,
        "credentials were never sent to the second backend"
    );
    assert!(store.list_accounts().await.unwrap().is_empty());
}

#[tokio::test]
async fn a_live_selected_backend_is_used_and_the_replica_is_not_contacted() {
    let primary = MockPop3::start("alice@example.org", "testpass").await;
    let replica = MockPop3::start("alice@example.org", "testpass").await;
    let (_store, base, _db) = fixture(
        "t39-failover-live",
        primary.addr.port(),
        replica.addr.port(),
    )
    .await;

    assert_eq!(login(&base, "testpass").await.status(), 200);
    assert!(primary.logins_accepted() >= 1);
    assert_eq!(replica.logins_accepted() + replica.logins_refused(), 0);
}
