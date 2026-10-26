//! Actual password and second-factor linking keeps the primary cookie intact.
mod common;
use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
use mw_store::{DomainConfigRow, ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};
const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

async fn policy(store: &Store, key: &str, value: &str) {
    store
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "linked-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
}
async fn value(response: reqwest::Response, expected: u16) -> Value {
    let status = response.status();
    let text = response.text().await.unwrap();
    assert_eq!(status, expected, "{text}");
    serde_json::from_str(&text).unwrap()
}
#[tokio::test]
async fn linked_password_mfa_context_personal_data_and_revocation_are_real() {
    let mail = common::mock_mail::MockPop3::start_accounts(&[
        ("parent@example.org", "parent-pass"),
        ("stranger@example.org", "stranger-pass"),
    ])
    .await;
    let child_mail = common::mock_imap::MockImap::start("child@child.test", "child-pass").await;
    let db = common::test_db::unique_db_path("linked-http");
    let uploads = db.parent().unwrap().join("uploads");
    std::fs::create_dir_all(&uploads).unwrap();
    // Single test in this binary; this fixture owns the process-local setting.
    unsafe {
        std::env::set_var("MW_UPLOAD_DIR", &uploads);
    }
    let store = Store::open(db.to_str().unwrap(), ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    store.put_domain_config(&DomainConfigRow{name:"example.org".into(),upstream_json:json!({"pop3":{"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","allowPrivate":true}}).to_string(),allowlist_json:"[]".into(),blocklist_json:"[]".into(),enabled:true,preferred:false,settings_json:"{}".into(),updated_at:1}).await.unwrap();
    store.put_domain_config(&DomainConfigRow{name:"child.test".into(),upstream_json:json!({"imap":{"host":"127.0.0.1","port":child_mail.addr.port(),"tls":"none","allowPrivate":true}}).to_string(),allowlist_json:"[]".into(),blocklist_json:"[]".into(),enabled:true,preferred:false,settings_json:"{}".into(),updated_at:1}).await.unwrap();
    policy(&store, "signin.retryCooldownSecs", "0").await;
    let app = build_app(AppConfig {
        db_path: db.to_string_lossy().into(),
        server_key_hex: Some(KEY.into()),
        web_dir: None,
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: mw_server::HardeningConfig {
            csrf_strict: true,
            ..Default::default()
        },
        security: SecurityConfig {
            upstream_default: Some(mail.url()),
            ..Default::default()
        },
    })
    .await
    .unwrap();
    let socket = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", socket.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(
            socket,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap()
    });
    let client = || {
        reqwest::Client::builder()
            .no_proxy()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap()
    };
    let parent_jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
    let mut parent = reqwest::Client::builder()
        .no_proxy()
        .cookie_provider(parent_jar.clone())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    let child = client();
    let stranger = client();
    let p = value(
        parent
            .post(format!("{base}/api/login"))
            .json(&json!({"username":"parent@example.org","password":"parent-pass"}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let c = value(
        child
            .post(format!("{base}/api/login"))
            .json(&json!({"username":"child@child.test","password":"child-pass"}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let s = value(
        stranger
            .post(format!("{base}/api/login"))
            .json(&json!({"username":"stranger@example.org","password":"stranger-pass"}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "x-csrf-token",
        p["csrfToken"].as_str().unwrap().parse().unwrap(),
    );
    parent = reqwest::Client::builder()
        .no_proxy()
        .cookie_provider(parent_jar)
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    let pa = p["storageAuthority"].as_str().unwrap();
    let ca = c["storageAuthority"].as_str().unwrap();
    let initial = value(
        parent
            .get(format!("{base}/api/accounts"))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert_eq!(initial["policy"]["enabled"], false);
    assert_eq!(
        parent
            .post(format!("{base}/api/accounts/link"))
            .header("x-mailwoman-storage-authority", pa)
            .json(&json!({"username":"child@child.test","password":"child-pass"}))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    policy(&store, "accounts.linkingEnabled", "true").await;
    let endpoint = format!("{base}/api/accounts/link");
    let no_proof = value(
        parent
            .post(&endpoint)
            .header("x-mailwoman-storage-authority", pa)
            .json(&json!({"username":"child@child.test","password":"child-pass"}))
            .send()
            .await
            .unwrap(),
        403,
    )
    .await;
    assert_eq!(no_proof["code"], "step_up_required");
    assert_eq!(
        parent
            .post(format!("{base}/api/security/step-up"))
            .json(&json!({"password":"parent-pass"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        parent
            .post(&endpoint)
            .header("x-mailwoman-storage-authority", pa)
            .json(&json!({"username":"child@child.test","password":"wrong"}))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        value(
            parent
                .get(format!("{base}/api/accounts"))
                .send()
                .await
                .unwrap(),
            200
        )
        .await["accounts"],
        json!([])
    );
    let secret = b"linked-child-totp-secret";
    store
        .put_totp_secret(c["accountId"].as_str().unwrap(), secret, true)
        .await
        .unwrap();
    let response = parent
        .post(&endpoint)
        .header("x-mailwoman-storage-authority", pa)
        .json(&json!({"username":"child@child.test","password":"child-pass","displayName":"Work"}))
        .send()
        .await
        .unwrap();
    assert!(response.headers().get("set-cookie").is_none());
    let challenge = value(response, 202).await;
    assert!(challenge.get("pendingToken").is_none());
    assert!(challenge.get("token").is_none());
    let id = challenge["challengeId"].as_str().unwrap();
    assert_eq!(
        stranger
            .post(format!("{base}/api/security/step-up"))
            .header("x-csrf-token", s["csrfToken"].as_str().unwrap())
            .json(&json!({"password":"stranger-pass"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        stranger
            .post(format!("{base}/api/accounts/challenges/{id}/confirm"))
            .header("x-csrf-token", s["csrfToken"].as_str().unwrap())
            .header(
                "x-mailwoman-storage-authority",
                s["storageAuthority"].as_str().unwrap()
            )
            .json(&json!({"method":"totp","code":"000000"}))
            .send()
            .await
            .unwrap()
            .status(),
        410
    );
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let code = mw_mfa::totp::totp_at(secret, now, &mw_mfa::totp::TotpParams::default());
    let response = parent
        .post(format!("{base}/api/accounts/challenges/{id}/confirm"))
        .header("x-mailwoman-storage-authority", pa)
        .json(&json!({"method":"totp","code":code}))
        .send()
        .await
        .unwrap();
    assert!(response.headers().get("set-cookie").is_none());
    let linked = value(response, 201).await;
    assert!(linked.get("token").is_none());
    let link = linked["account"]["id"].as_str().unwrap();
    let revision = linked["account"]["revision"].as_str().unwrap();
    assert_eq!(linked["account"]["storageAuthority"], ca);
    let primary_me = value(
        parent.get(format!("{base}/api/me")).send().await.unwrap(),
        200,
    )
    .await;
    assert_eq!(primary_me["accountId"], p["accountId"]);
    let switched = value(
        parent
            .post(format!("{base}/api/accounts/{link}/switch"))
            .header("x-mailwoman-storage-authority", pa)
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert_eq!(switched["context"]["storageAuthority"], ca);
    let me = value(
        parent
            .get(format!("{base}/api/me"))
            .header("x-mailwoman-link", link)
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert_eq!(me["accountId"], c["accountId"]);
    assert_eq!(me["storageAuthority"], ca);
    assert_eq!(
        me["csrfToken"], p["csrfToken"],
        "selected Me must preserve the primary CSRF authority"
    );
    let discovery = value(
        parent
            .get(format!("{base}/jmap/session"))
            .header("x-mailwoman-link", link)
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert!(discovery["accounts"][c["accountId"].as_str().unwrap()].is_object());
    let folders=value(parent.post(format!("{base}/jmap/api")).header("x-mailwoman-link",link).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Mailbox/get",{"accountId":c["accountId"]},"m"]]})).send().await.unwrap(),200).await;
    assert_eq!(folders["methodResponses"][0][0], "Mailbox/get");
    assert!(
        !folders["methodResponses"][0][1]["list"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        parent
            .get(format!("{base}/api/accounts"))
            .header("x-mailwoman-link", link)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        parent
            .get(format!("{base}/jmap/eventsource"))
            .header("x-mailwoman-link", link)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        stranger
            .get(format!("{base}/api/me"))
            .header("x-mailwoman-link", link)
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    let signatures = format!("{base}/api/account/signatures");
    let child_account = c["accountId"].as_str().unwrap();
    let inbox = folders["methodResponses"][0][1]["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["role"] == "inbox")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    let raw = "From: Sender <sender@example.test>\r\nTo: child@child.test\r\nSubject: Linked attachment\r\nMessage-ID: <linked-proof@example.test>\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=linked\r\n\r\n--linked\r\nContent-Type: text/plain\r\n\r\nChild body\r\n--linked\r\nContent-Type: image/png\r\nContent-Disposition: inline; filename=linked.png\r\nContent-ID: <linked-part>\r\nContent-Transfer-Encoding: base64\r\n\r\niVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGMQVDL+DwACFAFmBODefwAAAABJRU5ErkJggg==\r\n--linked--\r\n";
    let upload = value(
        parent
            .post(format!("{base}/jmap/upload/{child_account}"))
            .header("x-mailwoman-link", link)
            .header("content-type", "message/rfc822")
            .body(raw)
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let imported=value(parent.post(format!("{base}/jmap/api")).header("x-mailwoman-link",link).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/import",{"accountId":child_account,"emails":{"i":{"blobId":upload["blobId"],"mailboxIds":{inbox:true}}}},"i"]]})).send().await.unwrap(),200).await;
    let email = imported["methodResponses"][0][1]["created"]["i"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("{imported}"));
    let message=value(parent.post(format!("{base}/jmap/api")).header("x-mailwoman-link",link).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/get",{"accountId":child_account,"ids":[email],"properties":["id","mailwomanInlineParts"]},"g"]]})).send().await.unwrap(),200).await;
    let blob = message["methodResponses"][0][1]["list"][0]["mailwomanInlineParts"][0]["blobId"]
        .as_str()
        .unwrap_or_else(|| panic!("{message}"));
    let signed = value(
        parent
            .post(format!("{base}/api/rc/sign"))
            .header("x-mailwoman-link", link)
            .json(&json!({"emailId":email,"parts":[blob]}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let signed_path = signed["parts"][0]["src"]
        .as_str()
        .unwrap_or_else(|| panic!("{signed}"));
    let public = client();
    let part = public
        .get(format!("{base}{signed_path}"))
        .send()
        .await
        .unwrap();
    assert_eq!(part.status(), 200);
    assert!(part.bytes().await.unwrap().starts_with(b"\x89PNG"));
    let download = parent
        .get(format!(
            "{base}/jmap/download/{child_account}/{blob}/note.txt?mwLink={link}"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(download.status(), 200);
    assert!(download.bytes().await.unwrap().starts_with(b"\x89PNG"));
    for internal in store.sessions_by_account(child_account).await.unwrap() {
        if store.linked_session_kind(&internal.id).await.unwrap() != Some(0) {
            assert_eq!(
                public
                    .get(format!("{base}/api/me"))
                    .header("cookie", format!("mw_session={}", internal.id))
                    .send()
                    .await
                    .unwrap()
                    .status(),
                401
            );
            assert_eq!(
                public
                    .get(format!("{base}/api/me"))
                    .bearer_auth(&internal.id)
                    .send()
                    .await
                    .unwrap()
                    .status(),
                401
            );
        }
    }
    policy(&store, "accounts.linkingEnabled", "false").await;
    assert_eq!(
        parent
            .get(format!("{base}/api/me"))
            .header("x-mailwoman-link", link)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert!(
        public
            .get(format!("{base}{signed_path}"))
            .send()
            .await
            .unwrap()
            .status()
            .is_client_error()
    );
    policy(&store, "accounts.linkingEnabled", "true").await;
    let body = json!({"name":"linked","body":"Only child","text":"Only child","isDefault":true});
    assert_eq!(
        parent
            .post(&signatures)
            .header("x-mailwoman-link", link)
            .header("x-mailwoman-storage-authority", pa)
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    assert_eq!(
        parent
            .post(&signatures)
            .header("x-mailwoman-link", link)
            .header("x-mailwoman-storage-authority", ca)
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        value(parent.get(&signatures).send().await.unwrap(), 200).await["signatures"],
        json!([])
    );
    assert_eq!(
        value(child.get(&signatures).send().await.unwrap(), 200).await["signatures"][0]["body"],
        "Only child"
    );
    assert_eq!(
        parent
            .delete(format!("{base}/api/accounts/{link}"))
            .header("x-mailwoman-storage-authority", pa)
            .send()
            .await
            .unwrap()
            .status(),
        428
    );
    assert_eq!(
        parent
            .delete(format!("{base}/api/accounts/{link}"))
            .header("x-mailwoman-storage-authority", pa)
            .header("if-match", format!("\"{revision}\""))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(
        parent
            .get(format!("{base}/api/me"))
            .header("x-mailwoman-link", link)
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    assert_eq!(
        child
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert!(
        public
            .get(format!("{base}{signed_path}"))
            .send()
            .await
            .unwrap()
            .status()
            .is_client_error(),
        "unlink revokes already signed child resources"
    );
    assert_eq!(
        value(child.get(&signatures).send().await.unwrap(), 200).await["signatures"][0]["body"],
        "Only child"
    );
    let another = value(
        parent
            .post(&endpoint)
            .header("x-mailwoman-storage-authority", pa)
            .json(&json!({"username":"stranger@example.org","password":"stranger-pass"}))
            .send()
            .await
            .unwrap(),
        201,
    )
    .await;
    let another = another["account"]["id"].as_str().unwrap();
    assert_eq!(
        parent
            .get(format!("{base}/api/me"))
            .header("x-mailwoman-link", another)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        parent
            .post(format!("{base}/api/logout"))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(
        parent
            .get(format!("{base}/api/me"))
            .header("x-mailwoman-link", another)
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        stranger
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    server.abort();
}

/// Isolated real upstream fixture for the linked-account browser acceptance.
#[tokio::test]
#[ignore]
async fn linked_browser_fixture() {
    let mail = common::mock_mail::MockPop3::start_accounts(&[
        ("parent@example.org", "parent-pass"),
        ("stranger@example.org", "stranger-pass"),
    ])
    .await;
    let child = common::mock_imap::MockImap::start("child@child.test", "child-pass").await;
    let db = common::test_db::unique_db_path("linked-browser");
    let store = Store::open(db.to_str().unwrap(), ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    for (name, upstream) in [
        (
            "example.org",
            json!({"pop3":{"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","allowPrivate":true}}),
        ),
        (
            "child.test",
            json!({"imap":{"host":"127.0.0.1","port":child.addr.port(),"tls":"none","allowPrivate":true}}),
        ),
    ] {
        store
            .put_domain_config(&DomainConfigRow {
                name: name.into(),
                upstream_json: upstream.to_string(),
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
    policy(&store, "signin.retryCooldownSecs", "0").await;
    policy(&store, "accounts.linkingEnabled", "true").await;
    let app = build_app(AppConfig {
        db_path: db.to_string_lossy().into_owned(),
        server_key_hex: Some(KEY.into()),
        web_dir: Some(
            std::env::var("MW_LINKED_FIXTURE_WEB")
                .expect("explicit fixture web directory")
                .into(),
        ),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: mw_server::HardeningConfig {
            csrf_strict: true,
            ..Default::default()
        },
        security: SecurityConfig {
            upstream_default: Some(mail.url()),
            ..Default::default()
        },
    })
    .await
    .unwrap();
    let port: u16 = std::env::var("MW_LINKED_FIXTURE_PORT")
        .expect("explicit fixture port")
        .parse()
        .unwrap();
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
        .await
        .unwrap();
    eprintln!(
        "linked browser fixture ready on {port}; database {}",
        db.display()
    );
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}
