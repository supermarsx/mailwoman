//! Domain-selected upstreams and pre-authentication domain refusal over HTTP.
use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
use mw_store::{DomainConfigRow, ServerKey, Store};
use serde_json::{Value, json};
mod common;
use common::{mock_mail::MockPop3, test_db};

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

#[tokio::test]
async fn login_and_me_publish_verified_storage_authority_for_browser_and_native() {
    let mail = MockPop3::start("alice@example.org", "testpass").await;
    let db = test_db::unique_dir("storage-authority-http").join("mw.db").to_string_lossy().into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap()).await.unwrap();
    store.put_domain_config(&row("example.org", &mail)).await.unwrap();
    let base = server(&db, &mail).await;
    let browser = reqwest::Client::builder().no_proxy().cookie_store(true).build().unwrap();
    let response = login(&browser, &base, "alice@example.org").await;
    assert_eq!(response.status(), 200);
    let login_body: Value = response.json().await.unwrap();
    let authority = login_body["storageAuthority"].as_str().unwrap();
    assert!(authority.starts_with("msa1_") && authority.len() == 69);
    let me: Value = browser.get(format!("{base}/api/me")).send().await.unwrap().json().await.unwrap();
    assert_eq!(me["storageAuthority"], authority);
    assert_eq!(me["accountId"], login_body["accountId"]);
    let native = reqwest::Client::builder().no_proxy().build().unwrap();
    let response = native.post(format!("{base}/api/login"))
        .json(&json!({"username":"alice@example.org","password":"testpass","clientType":"native"}))
        .send().await.unwrap();
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["storageAuthority"], authority);
    assert_eq!(body["accountId"], login_body["accountId"]);
    let token = body["token"].as_str().unwrap();
    let me: Value = native.get(format!("{base}/api/me")).bearer_auth(token).send().await.unwrap().json().await.unwrap();
    assert_eq!(me["storageAuthority"], authority);
}

#[tokio::test]
async fn authenticated_login_returns_before_slow_message_enumeration() {
    use std::{sync::Arc, time::Duration};
    let gate = Arc::new(tokio::sync::Semaphore::new(0));
    let mail = MockPop3::start_accounts_with_index_gate(&[("alice@example.org", "testpass")], Some(gate.clone())).await;
    let db = test_db::unique_dir("slow-index-login").join("mw.db").to_string_lossy().into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap()).await.unwrap();
    store.put_domain_config(&row("example.org", &mail)).await.unwrap();
    let base = server(&db, &mail).await;
    let client = reqwest::Client::builder().no_proxy().cookie_store(true).build().unwrap();
    let response = tokio::time::timeout(Duration::from_secs(5), login(&client, &base, "alice@example.org")).await.expect("login must not wait for UIDL");
    assert_eq!(response.status(), 200);
    assert_eq!(client.get(format!("{base}/api/me")).send().await.unwrap().status(), 200);
    let account = store.list_accounts().await.unwrap().remove(0);
    assert!(!store.list_mailboxes(&account.id).await.unwrap().is_empty(), "authenticated folder metadata must already exist");
    gate.add_permits(1);
}

#[tokio::test]
async fn domain_password_policy_uses_capabilities_and_changes_revoke_existing_authority() {
    let mail = MockPop3::start("alice@example.org", "testpass").await;
    let fallback = MockPop3::start("unused", "unused").await;
    let db = test_db::unique_dir("domain-password-policy")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    let mut domain = row("example.org", &mail);
    let mut upstream: Value = serde_json::from_str(&domain.upstream_json).unwrap();
    upstream["pop3"]["authentication"] =
        json!({"mode":"auto","mechanisms":["plain","pop3-user-pass"]});
    domain.upstream_json = upstream.to_string();
    store.put_domain_config(&domain).await.unwrap();
    let base = server(&db, &fallback).await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    assert_eq!(
        login(&client, &base, "alice@example.org").await.status(),
        200
    );
    let original = store.list_accounts().await.unwrap().remove(0);
    assert!(
        mw_engine::AccountPolicy::from_json(&original.sync_policy_json)
            .domain_binding
            .is_some()
    );

    upstream["pop3"]["authentication"] = json!({"mode":"fixed","mechanisms":["plain"]});
    domain.upstream_json = upstream.to_string();
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
    let accepted = mail.logins_accepted();
    let refused = mail.logins_refused();
    assert_eq!(
        login(&client, &base, "alice@example.org").await.status(),
        401
    );
    assert_eq!(mail.logins_accepted(), accepted);
    assert_eq!(
        mail.logins_refused(),
        refused,
        "unsupported SASL sends no password and never falls back to USER/PASS"
    );
    assert_eq!(fallback.logins_accepted() + fallback.logins_refused(), 0);

    upstream["pop3"]["authentication"] = json!({"mode":"fixed","mechanisms":["pop3-user-pass"]});
    domain.upstream_json = upstream.to_string();
    store.put_domain_config(&domain).await.unwrap();
    assert_eq!(
        login(&client, &base, "alice@example.org").await.status(),
        200
    );
    let current = store.list_accounts().await.unwrap().remove(0);
    assert_eq!(current.id, original.id);
    assert_eq!(current.username, "alice@example.org");
}

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
    assert!(public["usernameDomainHint"].is_null());
    assert_eq!(public["showDomainHint"], false);
    assert_eq!(public["rememberSignins"], true);
    assert_eq!(public["showRememberedHint"], false);
    let allowed = [
        "brandName",
        "theme",
        "accent",
        "loginNotice",
        "usernameDomainHint",
        "showDomainHint",
        "rememberSignins",
        "showRememberedHint",
        "retryCooldownSecs",
        "logoUrl",
        "faviconUrl",
        "loginBackgroundUrl",
        "antiBot",
        "maintenance",
    ];
    assert_eq!(public.as_object().unwrap().len(), allowed.len());
    assert!(
        public
            .as_object()
            .unwrap()
            .keys()
            .all(|key| allowed.contains(&key.as_str())),
        "public config must not disclose deployment inventory or secrets"
    );
    assert_eq!(public["retryCooldownSecs"], 0);
    assert!(public["brandName"].is_string());
    for key in ["logoUrl", "faviconUrl", "loginBackgroundUrl"] {
        assert!(public[key].is_null());
    }
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "login.showDomainHint".into(),
            global_value: Some("true".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    for (key, value) in [("login.rememberSignins", "false"), ("login.showRememberedHint", "true")] {
        store.put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: key.into(), global_value: Some(value.into()), allow_user_override: false,
            global_locked: false, updated_by: "test".into(), updated_at: 2,
        }).await.unwrap();
    }
    let opted_in: Value = client
        .get(format!("{base}/api/public-config"))
        .header("host", "mail.test")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(opted_in["usernameDomainHint"], "mapped.test");
    assert_eq!(opted_in["showDomainHint"], true);
    assert_eq!(opted_in["rememberSignins"], false);
    assert_eq!(opted_in["showRememberedHint"], true);
    assert_eq!(opted_in.as_object().unwrap().len(), allowed.len());
    assert!(opted_in.as_object().unwrap().keys().all(|key| allowed.contains(&key.as_str())));
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

#[tokio::test]
async fn ranked_backends_format_wire_usernames_without_changing_identity_or_retrying_auth() {
    let primary = MockPop3::start_accounts(&[
        ("CORP\\alice", "testpass"),
        ("alice@AUTH.REALM", "testpass"),
    ])
    .await;
    let secondary = MockPop3::start("alice@alias.test", "testpass").await;
    let fallback = MockPop3::start("alice@alias.test", "testpass").await;
    let db = test_db::unique_dir("domain-ranked-auth")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    let socket = |mail: &MockPop3| json!({"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","allowPrivate":true,"security":{"mode":"fixed","allowed":["none"],"allowPlaintext":true}});
    let mut config = json!({"version":2,"backends":[
        {"id":"preferred","role":"mail","protocol":"pop3","priority":500,"preferred":true,"endpoint":socket(&primary),"username":{"format":"domainUser","domain":"CORP"}},
        {"id":"secondary","role":"mail","protocol":"pop3","priority":0,"endpoint":socket(&secondary)}
    ]});
    let mut domain = row("example.org", &primary);
    domain.upstream_json = config.to_string();
    store.put_domain_config(&domain).await.unwrap();
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
    let refused = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":"alice@alias.test","password":"wrong"}))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), 401);
    assert!(primary.logins_refused() > 0);
    assert_eq!(
        secondary.logins_accepted() + secondary.logins_refused(),
        0,
        "an auth failure never tries the next backend"
    );
    let response = login(&client, &base, "alice@alias.test").await;
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    let account = store
        .list_accounts()
        .await
        .unwrap()
        .into_iter()
        .find(|account| account.username == "alice@alias.test")
        .unwrap();
    assert_eq!(
        store
            .account_credentials(&account.id)
            .await
            .unwrap()
            .username,
        "alice@alias.test"
    );
    let initial_binding =
        mw_engine::AccountPolicy::from_json(&account.sync_policy_json).domain_binding;
    assert!(initial_binding.is_some());
    config["backends"][0]["username"] = json!({"format":"userRealm","realm":"AUTH.REALM"});
    domain.upstream_json = config.to_string();
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
    let response = login(&client, &base, "alice@alias.test").await;
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    let fresh = store.get_account(&account.id).await.unwrap();
    assert_eq!(fresh.username, "alice@alias.test");
    assert_ne!(
        initial_binding,
        mw_engine::AccountPolicy::from_json(&fresh.sync_policy_json).domain_binding
    );
    for backend in config["backends"].as_array_mut().unwrap() {
        backend["enabled"] = json!(false);
        backend["preferred"] = json!(false);
    }
    domain.upstream_json = config.to_string();
    store.put_domain_config(&domain).await.unwrap();
    assert_eq!(
        login(&client, &base, "alice@alias.test").await.status(),
        404
    );
    assert_eq!(fallback.logins_accepted() + fallback.logins_refused(), 0);
    assert_eq!(secondary.logins_accepted() + secondary.logins_refused(), 0);
}

#[tokio::test]
async fn formatted_login_uses_endpoint_ca_and_server_name_in_the_real_engine_path() {
    use rustls_pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
    use std::sync::Arc;
    let plain = MockPop3::start("alice", "testpass").await;
    let cert_pem = include_str!("fixtures/tls/localhost-cert.pem");
    let cert = CertificateDer::from_pem_slice(cert_pem.as_bytes()).unwrap();
    let key =
        PrivateKeyDer::from_pem_slice(include_bytes!("fixtures/tls/localhost-key.pem")).unwrap();
    let tls = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(vec![cert], key)
    .unwrap();
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let plain_addr = plain.addr;
    let relay = tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            let acceptor = acceptor.clone();
            tokio::spawn(async move {
                let Ok(mut encrypted) = acceptor.accept(socket).await else {
                    return;
                };
                let Ok(mut upstream) = tokio::net::TcpStream::connect(plain_addr).await else {
                    return;
                };
                let _ = tokio::io::copy_bidirectional(&mut encrypted, &mut upstream).await;
            });
        }
    });
    let fallback = MockPop3::start("unused", "unused").await;
    let db = test_db::unique_dir("domain-tls-auth")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    let mut config = json!({"version":2,"backends":[{"id":"tls-mail","role":"mail","protocol":"pop3","endpoint":{"host":"127.0.0.1","port":port,"tls":"implicit","allowPrivate":true,"security":{"mode":"fixed","allowed":["implicit"],"serverName":"wrong.example","caPem":cert_pem}},"username":{"format":"localUser"}}]});
    let mut domain = row("example.org", &fallback);
    domain.upstream_json = config.to_string();
    store.put_domain_config(&domain).await.unwrap();
    let base = server(&db, &fallback).await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    assert_eq!(
        login(&client, &base, "alice@example.org").await.status(),
        401,
        "wrong TLS name must refuse before authentication"
    );
    assert_eq!(plain.logins_accepted() + plain.logins_refused(), 0);
    config["backends"][0]["endpoint"]["security"]["serverName"] = json!("localhost");
    config["backends"][0]["endpoint"]["security"]["caPem"] = Value::Null;
    domain.upstream_json = config.to_string();
    store.put_domain_config(&domain).await.unwrap();
    assert_eq!(
        login(&client, &base, "alice@example.org").await.status(),
        401,
        "an untrusted private CA must refuse before authentication"
    );
    assert_eq!(plain.logins_accepted() + plain.logins_refused(), 0);
    config["backends"][0]["endpoint"]["security"]["caPem"] = json!(cert_pem);
    domain.upstream_json = config.to_string();
    store.put_domain_config(&domain).await.unwrap();
    let response = login(&client, &base, "alice@example.org").await;
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    assert!(
        plain.logins_accepted() > 0,
        "local username authenticated behind TLS"
    );
    let account = store
        .list_accounts()
        .await
        .unwrap()
        .into_iter()
        .find(|account| account.username == "alice@example.org")
        .unwrap();
    assert_eq!(account.tls, "implicit");
    assert_eq!(
        store
            .account_credentials(&account.id)
            .await
            .unwrap()
            .username,
        "alice@example.org"
    );
    assert_eq!(fallback.logins_accepted() + fallback.logins_refused(), 0);
    relay.abort();
}

#[tokio::test]
async fn only_successful_domain_login_enrolls_the_real_account_in_admin_users() {
    let mail = MockPop3::start("alice@alias.test", "testpass").await;
    let db = test_db::unique_dir("domain-user-directory")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    store
        .put_domain_config(&row("example.org", &mail))
        .await
        .unwrap();
    store
        .put_domain_alias("alias.test", "example.org")
        .await
        .unwrap();
    let config = AppConfig {
        db_path: db.clone(),
        server_key_hex: Some(KEY.into()),
        mode: ServerMode::Engine,
        cookie_secure: false,
        web_dir: None,
        hardening: Default::default(),
        security: SecurityConfig {
            upstream_default: Some(mail.url()),
            ..Default::default()
        },
    };
    let (app, _) = mw_server::build_app_full(
        config,
        mw_server::V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("hunter2".into()),
            redis_url: None,
        },
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let admin = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    assert_eq!(
        admin
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    assert_eq!(
        login(&client, &base, "failed@alias.test").await.status(),
        401
    );
    assert!(store.directory_accounts().await.unwrap().is_empty());
    assert_eq!(
        login(&client, &base, "alice@alias.test").await.status(),
        200
    );
    let users: Value = admin
        .get(format!("{base}/admin/users"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(users.as_array().unwrap().len(), 1);
    let account = store.list_accounts().await.unwrap().remove(0);
    assert_eq!(users[0]["accountId"], account.id);
    assert_eq!(users[0]["username"], "alice");
    assert_eq!(users[0]["domain"], "alias.test");
    assert!(users[0]["lastSigninAt"].as_i64().unwrap() > 0);
    let identity: Value = admin
        .get(format!("{base}/admin/users/{}/identity", account.id))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(identity["address"], "alice@alias.test");
    assert_eq!(
        login(&client, &base, "alice@alias.test").await.status(),
        200
    );
    assert_eq!(store.directory_accounts().await.unwrap().len(), 1);
    // Simulate the historical alias-to-parent registration, without changing the
    // authenticated native account or its verified authority.
    assert!(
        store
            .register_successful_signin(mw_store::user_directory::SuccessfulSignin {
                account_id: "legacy-fixture",
                address: "alice@example.org",
                domain: "example.org",
                local_part: "alice",
                mode: "engine",
                now: chrono::Utc::now().timestamp(),
            })
            .await
            .unwrap()
    );
    let raw = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&db))
        .await
        .unwrap();
    sqlx::query("DELETE FROM mail_user_account WHERE account_id='legacy-fixture'")
        .execute(&raw)
        .await
        .unwrap();
    sqlx::query("UPDATE mail_user_account SET address='alice@example.org',mode='engine' WHERE account_id=?1").bind(&account.id).execute(&raw).await.unwrap();
    let users: Value = admin
        .get(format!("{base}/admin/users"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(users[0]["accountId"], account.id);
    assert_eq!(users[0]["domain"], "alias.test");
    assert_eq!(users[0]["loginAddress"], "alice@alias.test");
    assert_eq!(users[0]["directoryAddress"], "alice@example.org");
    assert_eq!(users[0]["legacyBinding"], true);
    let detail: Value = admin
        .get(format!("{base}/admin/users/{}/identity", account.id))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(detail["address"], "alice@example.org");
    assert_eq!(detail["loginAddress"], "alice@alias.test");
    assert_eq!(detail["legacyBinding"], true);
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT address FROM mail_user_account WHERE account_id=?1"
        )
        .bind(&account.id)
        .fetch_optional(&raw)
        .await
        .unwrap()
        .as_deref(),
        Some("alice@example.org")
    );
    assert_eq!(
        login(&client, &base, "alice@alias.test").await.status(),
        200
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT address FROM mail_user_account WHERE account_id=?1"
        )
        .bind(&account.id)
        .fetch_optional(&raw)
        .await
        .unwrap()
        .as_deref(),
        Some("alice@example.org")
    );
    // A colliding proxy ID is never evidence for native display recovery.
    sqlx::query("UPDATE mail_user_account SET mode='proxy' WHERE account_id=?1")
        .bind(&account.id)
        .execute(&raw)
        .await
        .unwrap();
    let users: Value = admin
        .get(format!("{base}/admin/users"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(users[0]["domain"], "example.org");
    assert_eq!(users[0]["legacyBinding"], false);
}

#[tokio::test]
async fn rejected_password_is_attempted_once_and_never_registered() {
    let mail = MockPop3::start("alice@example.org", "different-secret").await;
    let db = test_db::unique_dir("login-no-retry")
        .join("mw.db")
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
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    assert_eq!(
        login(&client, &base, "alice@example.org").await.status(),
        401
    );
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    assert_eq!(mail.logins_refused(), 1);
    assert_eq!(mail.logins_accepted(), 0);
    assert!(store.list_accounts().await.unwrap().is_empty());
}

#[tokio::test]
async fn hanging_upstream_login_times_out_and_drops_connection() {
    use tokio::io::AsyncReadExt;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let peer = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut bytes = [0u8; 16];
        tokio::time::timeout(std::time::Duration::from_secs(30), stream.read(&mut bytes))
            .await
            .unwrap()
            .unwrap()
    });
    let fallback = MockPop3::start("unused", "unused").await;
    let db = test_db::unique_dir("login-timeout")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    let mut domain = row("example.org", &fallback);
    domain.upstream_json =
        json!({"imap":{"host":"127.0.0.1","port":addr.port(),"tls":"none","allowPrivate":true}})
            .to_string();
    store.put_domain_config(&domain).await.unwrap();
    store
        .put_domain_alias("alias.test", "example.org")
        .await
        .unwrap();
    let base = server(&db, &fallback).await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(29))
        .build()
        .unwrap();
    let start = std::time::Instant::now();
    assert_eq!(
        login(&client, &base, "alice@alias.test").await.status(),
        504
    );
    assert!(start.elapsed() < std::time::Duration::from_secs(28));
    assert_eq!(
        peer.await.unwrap(),
        0,
        "timed-out login must close its upstream socket"
    );
    assert!(store.list_accounts().await.unwrap().is_empty());
    assert_eq!(fallback.logins_accepted(), 0);
    assert_eq!(fallback.logins_refused(), 0);
}
