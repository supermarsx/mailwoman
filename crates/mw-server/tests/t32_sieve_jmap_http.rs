//! Public HTTP Sieve API and explicit script-key permission regressions.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::{ServerKey, Store};
use serde_json::{Value, json};
mod common;
use common::{mock_mail::MockPop3, test_db};
const KEY: &str = "28e12c02c3d4e5f60718293a4b5c6d7e28e12c02c3d4e5f60718293a4b5c6d7e";
#[tokio::test]
async fn http_sieve_scopes_cookie_native_and_headless_are_account_bound() {
    // This test binary contains exactly one test; environment is set before server tasks.
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
    }
    let pop = MockPop3::start("alice@example.org", "test-password").await;
    let dir = test_db::unique_dir("mw-codex-sieve-http");
    unsafe {
        std::env::set_var("MW_UPLOAD_DIR", dir.join("uploads"));
    }
    let db = dir.join("db.sqlite").to_string_lossy().to_string();
    std::fs::write(dir.join("index.html"), "<!doctype html><title>test</title>").unwrap();
    let app = mw_server::build_app_full(
        AppConfig {
            db_path: db.clone(),
            server_key_hex: Some(KEY.into()),
            web_dir: Some(dir),
            cookie_secure: false,
            mode: ServerMode::Engine,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(pop.url()),
                ..Default::default()
            },
        },
        V6Config {
            admin_enabled: false,
            admin_username: None,
            admin_password: None,
            redis_url: None,
        },
    )
    .await
    .unwrap()
    .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap()
    });
    let browser = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let login: Value = browser
        .post(format!("{base}/api/login"))
        .json(&json!({"username":"alice@example.org","password":"test-password"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let account = login["accountId"]
        .as_str()
        .unwrap_or_else(|| panic!("{}", login));
    let proof = browser
        .post(format!("{base}/api/security/step-up"))
        .json(&json!({"password":"test-password"}))
        .send()
        .await
        .unwrap();
    assert_eq!(proof.status(), 200, "{}", proof.text().await.unwrap());
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    store
        .put_setting_value(&mw_store::settings_policy::SettingValueRow {
            scope: mw_store::settings_policy::SettingScope::AdminUser,
            scope_id: account.into(),
            key: "cap.sieve.raw_edit".into(),
            value: "true".into(),
            locked: true,
            updated_by: "test".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    async fn mint(c: &reqwest::Client, base: &str, account: &str, scripts: bool) -> String {
        let mut scope = serde_json::to_value(mw_oauth::Scope::read_only(account)).unwrap();
        scope["scripts"] = scripts.into();
        let value: Value = c
            .post(format!("{base}/api/keys"))
            .json(&json!({"label":"sieve","accountId":account,"scope":scope}))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        value["displayToken"]
            .as_str()
            .unwrap_or_else(|| panic!("{}", value))
            .into()
    }
    let granted = mint(&browser, &base, account, true).await;
    let denied = mint(&browser, &base, account, false).await;
    let query = json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:sieve"],"methodCalls":[["SieveScript/get",{"accountId":account},"g"]]});
    for key in [&granted, &denied] {
        let response = browser
            .post(format!("{base}/jmap/api"))
            .bearer_auth(key)
            .json(&query)
            .send()
            .await
            .unwrap();
        assert_eq!(
            response.status().as_u16(),
            if key == &granted { 200 } else { 403 },
            "{}",
            response.text().await.unwrap()
        );
    }
    let headless = reqwest::Client::new();
    let response = headless
        .post(format!("{base}/jmap/api"))
        .bearer_auth(&granted)
        .json(&query)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    let mut other = query.clone();
    other["methodCalls"][0][1]["accountId"] = "another-account".into();
    assert_eq!(
        headless
            .post(format!("{base}/jmap/api"))
            .bearer_auth(&granted)
            .json(&other)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    let upload: Value = browser
        .post(format!("{base}/jmap/upload/{account}"))
        .header("content-type", "application/sieve")
        .body("keep;")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let blob = upload["blobId"]
        .as_str()
        .unwrap_or_else(|| panic!("{}", upload));
    let created:Value=browser.post(format!("{base}/jmap/api")).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:sieve"],"methodCalls":[["SieveScript/set",{"accountId":account,"create":{"one":{"name":"main","blobId":blob}},"onSuccessActivateScript":"#one"},"s"]]})).send().await.unwrap().json().await.unwrap();
    assert_eq!(
        created["methodResponses"][0][1]["created"]["one"]["isActive"], true,
        "{created}"
    );
    let got: Value = browser
        .post(format!("{base}/jmap/api"))
        .header("x-api-key", &granted)
        .json(&query)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        got["methodResponses"][0][1]["list"][0]["name"], "main",
        "{got}"
    );
    // A native token is not a scoped key and remains authorized by its native session.
    let native:Value=headless.post(format!("{base}/api/login")).json(&json!({"username":"alice@example.org","password":"test-password","clientType":"native"})).send().await.unwrap().json().await.unwrap();
    let token = native["token"]
        .as_str()
        .unwrap_or_else(|| panic!("{}", native));
    assert_eq!(
        headless
            .post(format!("{base}/jmap/api"))
            .bearer_auth(token)
            .json(&query)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let capability = "urn:ietf:params:jmap:sieve";
    let session: Value = headless
        .get(format!("{base}/jmap/session"))
        .bearer_auth(&granted)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        session["capabilities"].get(capability).is_some(),
        "{session}"
    );
    assert!(
        session["capabilities"]
            .get("urn:ietf:params:jmap:mail")
            .is_none()
    );
    let uploaded: Value = headless
        .post(format!("{base}/jmap/upload/{account}"))
        .bearer_auth(&granted)
        .header("content-type", "application/sieve")
        .body("keep;")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let script_blob = uploaded["blobId"]
        .as_str()
        .unwrap_or_else(|| panic!("{}", uploaded));
    let saved:Value=headless.post(format!("{base}/jmap/api")).bearer_auth(&granted)
        .json(&json!({"using":["urn:ietf:params:jmap:core",capability],"methodCalls":[["SieveScript/set",{"accountId":account,"create":{"key":{"name":"key-created","blobId":script_blob}}},"s"]]})).send().await.unwrap().json().await.unwrap();
    assert!(
        saved["methodResponses"][0][1]["created"]["key"]["id"].is_string(),
        "{saved}"
    );
    let download = headless
        .get(format!(
            "{base}/jmap/download/{account}/{script_blob}/script.sieve"
        ))
        .bearer_auth(&granted)
        .send()
        .await
        .unwrap();
    let status = download.status();
    let body = download.text().await.unwrap();
    assert_eq!(status, 200, "the key downloads its Sieve script: {body}");
    assert_eq!(body, "keep;");
    let unrelated: Value = browser
        .post(format!("{base}/jmap/upload/{account}"))
        .header("content-type", "text/plain")
        .body("private mail attachment")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let unrelated = unrelated["blobId"].as_str().unwrap();
    assert_eq!(
        headless
            .get(format!(
                "{base}/jmap/download/{account}/{unrelated}/mail.txt"
            ))
            .bearer_auth(&granted)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        headless
            .post(format!("{base}/jmap/upload/{account}"))
            .bearer_auth(&granted)
            .header("content-type", "text/plain")
            .body("keep;")
            .send()
            .await
            .unwrap()
            .status(),
        415
    );
    assert_eq!(
        headless
            .post(format!("{base}/jmap/upload/{account}"))
            .bearer_auth(&denied)
            .header("content-type", "application/sieve")
            .body("keep;")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    store
        .put_setting_value(&mw_store::settings_policy::SettingValueRow {
            scope: mw_store::settings_policy::SettingScope::AdminUser,
            scope_id: account.into(),
            key: "cap.sieve.forward_external".into(),
            value: "true".into(),
            locked: true,
            updated_by: "test".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    let save = json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:sieve"],
        "methodCalls":[["SieveScript/set",{"accountId":account},"s"]]});
    // The browser has a fresh proof; a headless key must not borrow it from a
    // server-side session lookup merely because it owns the same account.
    assert_eq!(
        browser
            .post(format!("{base}/jmap/api"))
            .header("x-api-key", &granted)
            .json(&save)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let refused = headless
        .post(format!("{base}/jmap/api"))
        .bearer_auth(&granted)
        .json(&save)
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), 403);
    assert_eq!(
        refused.json::<Value>().await.unwrap()["code"],
        "step_up_required"
    );
    if let Ok(target) = std::env::var("MW_SIEVE_HTTP_LIVE") {
        let (host, port) = target.rsplit_once(':').unwrap();
        let mail = reqwest::Url::parse(&pop.url()).unwrap();
        store.put_domain_config(&mw_store::DomainConfigRow {
            name:"example.org".into(),
            upstream_json:json!({"pop3":{"host":mail.host_str().unwrap(),"port":mail.port().unwrap(),"tls":"none","allowPrivate":true},
                "sieve":{"host":host,"port":port.parse::<u16>().unwrap(),"tls":"none","allowPrivate":true}}).to_string(),
            allowlist_json:"[]".into(),blocklist_json:"[]".into(),enabled:true,preferred:false,settings_json:"{}".into(),updated_at:1,
        }).await.unwrap();
        let session: Value = browser
            .get(format!("{base}/jmap/session"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(
            session["accounts"][account]["accountCapabilities"][capability]["urn:mailwoman:policy"]
                ["runsAt"],
            "server-sieve",
            "{session}"
        );
        let made:Value=browser.post(format!("{base}/jmap/api")).json(&json!({"using":["urn:ietf:params:jmap:core",capability],
            "methodCalls":[["SieveScript/set",{"accountId":account,"create":{"remote":{"name":"http-live","blobId":script_blob}},"onSuccessActivateScript":"#remote"},"s"]]})).send().await.unwrap().json().await.unwrap();
        let id = made["methodResponses"][0][1]["created"]["remote"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("{made}"));
        let mut client = mw_sieve::ManageSieveClient::connect(
            host,
            port.parse().unwrap(),
            mw_sieve::TlsMode::Plaintext,
            mw_sieve::Credentials::Plain {
                username: "alice@example.org".into(),
                password: "test-password".into(),
            },
        )
        .await
        .unwrap();
        assert!(
            client
                .list_scripts()
                .await
                .unwrap()
                .iter()
                .any(|s| s.name == "http-live" && s.active)
        );
        assert_eq!(client.get_script("http-live").await.unwrap(), "keep;");
        let cleanup: Value = browser
            .post(format!("{base}/jmap/api"))
            .json(
                &json!({"using":["urn:ietf:params:jmap:core",capability],"methodCalls":[
            ["SieveScript/set",{"accountId":account,"onSuccessDeactivateScript":true},"off"],
            ["SieveScript/set",{"accountId":account,"destroy":[id]},"del"]]}),
            )
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(
            cleanup["methodResponses"][1][1]["destroyed"],
            json!([id]),
            "{cleanup}"
        );
        assert!(client.list_scripts().await.unwrap().is_empty());
    }
    task.abort();
}
