//! Domain-selected upstreams and pre-authentication domain refusal over HTTP.
use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
use mw_store::{DomainConfigRow, ServerKey, Store};
use serde_json::{Value, json};
mod common;
use common::{mock_mail::MockPop3, test_db};

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
async fn domain_lists_apply_after_credentials_and_block_overrides_allow() {
    let mail = MockPop3::start_accounts(&[
        ("alice@example.org", "testpass"),
        ("alice@alias.test", "testpass"),
    ])
    .await;
    let fallback = MockPop3::start("unused", "unused").await;
    let db = test_db::unique_dir("domain-lists")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    let mut domain = row("example.org", &mail);
    domain.allowlist_json = json!(["*@example.org"]).to_string();
    domain.blocklist_json = json!(["ALICE@example.org"]).to_string();
    store.put_domain_config(&domain).await.unwrap();
    store
        .put_domain_alias("alias.test", "example.org")
        .await
        .unwrap();
    let base = server(&db, &fallback).await;
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let wrong = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":"alice@example.org","password":"wrong"}))
        .send()
        .await
        .unwrap();
    assert_eq!(wrong.status(), 401);
    let wrong_body = wrong.text().await.unwrap();
    for username in ["alice@example.org", "alice@alias.test"] {
        let denied = login(&client, &base, username).await;
        assert_eq!(denied.status(), 401);
        assert_eq!(denied.text().await.unwrap(), wrong_body);
    }
    assert!(
        mail.logins_accepted() >= 2,
        "lists are checked after credentials"
    );
    domain.blocklist_json = "[]".into();
    store.put_domain_config(&domain).await.unwrap();
    assert_eq!(
        login(&client, &base, "alice@alias.test").await.status(),
        200
    );
    domain.allowlist_json = json!(["bob@example.org"]).to_string();
    store.put_domain_config(&domain).await.unwrap();
    assert_eq!(
        login(&client, &base, "alice@example.org").await.status(),
        401
    );
    assert_eq!(fallback.logins_accepted() + fallback.logins_refused(), 0);
}

#[tokio::test]
async fn configured_domain_overrides_default_and_disabled_alias_never_dials() {
    let mail = MockPop3::start_accounts(&[
        ("alice@example.org", "testpass"),
        ("alice@alias.test", "testpass"),
    ])
    .await;
    let fallback = MockPop3::start("unused", "unused").await;
    let db = test_db::unique_dir("domain-login")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    let mut domain = row("example.org", &mail);
    store.put_domain_config(&domain).await.unwrap();
    store
        .set_preferred_domain(Some("example.org"))
        .await
        .unwrap();
    store
        .put_domain_alias("alias.test", "example.org")
        .await
        .unwrap();
    let base = server(&db, &fallback).await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let response = login(&client, &base, "alice").await;
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    assert!(mail.logins_accepted() > 0);
    assert_eq!(fallback.logins_accepted() + fallback.logins_refused(), 0);
    assert_eq!(
        login(&client, &base, "alice@alias.test").await.status(),
        200
    );
    domain.enabled = false;
    store.put_domain_config(&domain).await.unwrap();
    assert_eq!(
        client
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let before = mail.logins_accepted() + mail.logins_refused();
    for name in ["alice@example.org", "alice@alias.test"] {
        let response = login(&client, &base, name).await;
        assert_eq!(response.status(), 404);
        let body: Value = response.json().await.unwrap();
        assert_eq!(body["code"], "addressNotConfigured");
    }
    assert_eq!(mail.logins_accepted() + mail.logins_refused(), before);
    assert_eq!(fallback.logins_accepted() + fallback.logins_refused(), 0);
}

#[tokio::test]
async fn host_mapping_expands_bare_name_and_untrusted_forwarded_host_is_ignored() {
    let mail = MockPop3::start("alice@mapped.test", "testpass").await;
    let fallback = MockPop3::start("unused", "unused").await;
    let db = test_db::unique_dir("domain-host")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    store
        .put_domain_config(&row("mapped.test", &mail))
        .await
        .unwrap();
    store
        .put_domain_host("mail.test", "mapped.test")
        .await
        .unwrap();
    let base = server(&db, &fallback).await;
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let public = client
        .get(format!("{base}/api/public-config"))
        .header("host", "mail.test")
        .send()
        .await
        .unwrap();
    assert_eq!(public.status(), 200);
    assert_eq!(public.headers()["cache-control"], "no-store");
    let public: Value = public.json().await.unwrap();
    assert_eq!(public["usernameDomainHint"], "mapped.test");
    assert_eq!(public.as_object().unwrap().len(), 5);
    let response = client
        .post(format!("{base}/api/login"))
        .header("host", "mail.test")
        .header("x-forwarded-host", "attacker.test")
        .json(&json!({"username":"alice","password":"testpass"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    assert!(mail.logins_accepted() > 0);
    assert_eq!(fallback.logins_accepted() + fallback.logins_refused(), 0);
}

#[tokio::test]
async fn replaced_upstream_refuses_old_session_and_next_login_uses_new_target() {
    let first = MockPop3::start("alice@example.org", "testpass").await;
    let second = MockPop3::start("alice@example.org", "testpass").await;
    let db = common::test_db::unique_dir("domain-repoint")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    store
        .put_domain_config(&row("example.org", &first))
        .await
        .unwrap();
    let base = server(&db, &first).await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let response = login(&client, &base, "alice@example.org").await;
    assert_eq!(response.status(), 200);
    let previous: Value = response.json().await.unwrap();
    store
        .put_domain_config(&row("example.org", &second))
        .await
        .unwrap();
    assert_eq!(
        client
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let before = first.logins_accepted() + first.logins_refused();
    let response = login(&client, &base, "alice@example.org").await;
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    let current: Value = response.json().await.unwrap();
    assert_ne!(previous["accountId"], current["accountId"]);
    assert!(second.logins_accepted() > 0);
    assert_eq!(first.logins_accepted() + first.logins_refused(), before);
}

#[tokio::test]
async fn proxy_session_follows_reconfigured_domain_and_persists_new_authority() {
    async fn mock() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move {
            axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
        });
        base
    }
    let first = mock().await;
    let second = mock().await;
    let db = common::test_db::unique_dir("domain-proxy-repoint")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    let mut domain = DomainConfigRow {
        name: "example.org".into(),
        upstream_json: json!({"jmap":{"sessionUrl":first}}).to_string(),
        allowlist_json: "[]".into(),
        blocklist_json: "[]".into(),
        enabled: true,
        preferred: false,
        settings_json: "{}".into(),
        updated_at: 1,
    };
    store.put_domain_config(&domain).await.unwrap();
    let app = build_app(AppConfig {
        db_path: db,
        server_key_hex: Some(KEY.into()),
        mode: ServerMode::Proxy,
        cookie_secure: false,
        web_dir: None,
        hardening: Default::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(vec![first.clone(), second.clone()]),
            ..Default::default()
        },
    })
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let response = login(&client, &base, mw_mock_jmap::USER).await;
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    domain.upstream_json = json!({"jmap":{"sessionUrl":second}}).to_string();
    store.put_domain_config(&domain).await.unwrap();
    assert_eq!(
        client
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let sessions = store
        .sessions_by_account(mw_mock_jmap::ACCOUNT_ID)
        .await
        .unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].jmap_url, second);
    assert!(sessions[0].api_url.starts_with(&second));
}
