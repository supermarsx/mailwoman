//! Real admission, exact CAS and admin recovery while mailbox sessions stay intact.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};
mod common;
const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
#[tokio::test]
async fn maintenance_is_reversible_and_admin_recovery_survives_custom_paths() {
    let previous_base = std::env::var_os("MW_BASE_PATH");
    let previous_admin = std::env::var_os("MW_ADMIN_PATH");
    for prefix in ["", "/mail"] {
        unsafe {
            std::env::set_var("MW_BASE_PATH", prefix);
            std::env::set_var("MW_ADMIN_PATH", "/ops-test");
        }
        let mail = common::mock_mail::MockPop3::start("user@example.org", "mail-pass").await;
        let db = common::test_db::unique_db_path("maintenance-mode");
        let store = Store::open(db.to_str().unwrap(), ServerKey::from_hex(KEY).unwrap())
            .await
            .unwrap();
        store.put_domain_config(&mw_store::DomainConfigRow {
            name:"example.org".into(),
            upstream_json:json!({"pop3":{"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","allowPrivate":true}}).to_string(),
            allowlist_json:"[]".into(),blocklist_json:"[]".into(),enabled:true,preferred:false,settings_json:"{}".into(),updated_at:1,
        }).await.unwrap();
        store
            .put_setting_policy(&SettingPolicyRow {
                key: "signin.retryCooldownSecs".into(),
                global_value: Some("0".into()),
                allow_user_override: false,
                global_locked: false,
                updated_by: "fixture".into(),
                updated_at: 0,
            })
            .await
            .unwrap();
        let (app, _) = build_app_full(
            AppConfig {
                db_path: db.to_string_lossy().into_owned(),
                server_key_hex: Some(KEY.into()),
                web_dir: None,
                cookie_secure: false,
                mode: ServerMode::Engine,
                hardening: HardeningConfig::default(),
                security: SecurityConfig {
                    upstream_default: Some(mail.url()),
                    ..Default::default()
                },
            },
            V6Config {
                admin_enabled: true,
                admin_username: Some("root".into()),
                admin_password: Some("hunter2".into()),
                redis_url: None,
            },
        )
        .await
        .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let base = format!("{origin}{prefix}");
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await
            .unwrap()
        });
        let client = || {
            reqwest::Client::builder()
                .no_proxy()
                .cookie_store(true)
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap()
        };
        let admin = client();
        let user = client();
        let anonymous = client();
        let endpoint = format!("{base}/ops-test/maintenance/mode");
        assert_eq!(anonymous.get(&endpoint).send().await.unwrap().status(), 401);
        assert_eq!(
            admin
                .post(format!("{base}/ops-test/login"))
                .json(&json!({"username":"root","password":"hunter2"}))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        let login = user
            .post(format!("{base}/api/login"))
            .json(&json!({"username":"user@example.org","password":"mail-pass"}))
            .send()
            .await
            .unwrap();
        assert_eq!(login.status(), 200);
        let user_id = login.json::<Value>().await.unwrap()["accountId"].clone();
        let initial = admin.get(&endpoint).send().await.unwrap();
        assert_eq!(initial.status(), 200);
        let etag = initial.headers()["etag"].to_str().unwrap().to_string();
        assert_eq!(
            initial.json::<Value>().await.unwrap(),
            json!({"enabled":false,"message":"","retryAfterSecs":300,"configurationInvalid":false})
        );
        let enabled = json!({"enabled":true,"message":"Planned <work>","retryAfterSecs":73});
        assert_eq!(
            admin
                .put(&endpoint)
                .header("if-match", &etag)
                .json(&enabled)
                .send()
                .await
                .unwrap()
                .status(),
            403
        );
        assert_eq!(
            admin
                .post(format!("{base}/ops-test/step-up"))
                .json(&json!({"password":"hunter2"}))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        assert_eq!(
            admin
                .put(&endpoint)
                .json(&enabled)
                .send()
                .await
                .unwrap()
                .status(),
            428
        );
        assert_eq!(
            admin
                .put(&endpoint)
                .header("if-match", "*")
                .json(&enabled)
                .send()
                .await
                .unwrap()
                .status(),
            412
        );
        for invalid in [
            json!({"enabled":true,"message":"bad\r\nheader","retryAfterSecs":73}),
            json!({"enabled":true,"message":"","retryAfterSecs":0}),
            json!({"enabled":true,"message":"","retryAfterSecs":86401}),
        ] {
            assert_eq!(
                admin
                    .put(&endpoint)
                    .header("if-match", &etag)
                    .json(&invalid)
                    .send()
                    .await
                    .unwrap()
                    .status(),
                422
            );
        }
        assert!(
            store
                .get_setting("maintenance.mode.v1")
                .await
                .unwrap()
                .is_none(),
            "off/default reads and rejected writes must not create settings"
        );
        let saved = admin
            .put(&endpoint)
            .header("if-match", &etag)
            .json(&enabled)
            .send()
            .await
            .unwrap();
        assert_eq!(saved.status(), 200);
        let enabled_etag = saved.headers()["etag"].to_str().unwrap().to_string();
        assert_eq!(
            admin
                .put(&endpoint)
                .header("if-match", &etag)
                .json(&enabled)
                .send()
                .await
                .unwrap()
                .status(),
            412
        );
        for path in [
            "/api/me",
            "/jmap/session",
            "/api/accounts",
            "/api/rc/p/not-a-token",
            "/dav/calendar",
            "/mcp",
        ] {
            let response = user.get(format!("{base}{path}")).send().await.unwrap();
            assert_eq!(response.status(), 503, "{path}");
            assert_eq!(response.headers()["retry-after"], "73");
            let body = response.json::<Value>().await.unwrap();
            assert_eq!(body["code"], "maintenance");
            assert_eq!(body["message"], "Planned <work>");
        }
        assert_eq!(
            anonymous
                .post(format!("{base}/api/login"))
                .json(&json!({"username":"user@example.org","password":"mail-pass"}))
                .send()
                .await
                .unwrap()
                .status(),
            503
        );
        assert_eq!(
            anonymous
                .get(format!("{base}/healthz"))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        assert_eq!(
            anonymous
                .get(format!("{base}/workspace-popup.html"))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        let public = anonymous
            .get(format!("{base}/api/public-config"))
            .send()
            .await
            .unwrap();
        assert_eq!(public.status(), 200);
        assert_eq!(
            public.json::<Value>().await.unwrap()["maintenance"],
            enabled
        );
        assert_eq!(admin.get(&endpoint).send().await.unwrap().status(), 200);
        let persisted = Store::open(db.to_str().unwrap(), ServerKey::from_hex(KEY).unwrap())
            .await
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(
                &persisted
                    .get_setting("maintenance.mode.v1")
                    .await
                    .unwrap()
                    .unwrap()
            )
            .unwrap(),
            enabled
        );
        let off = json!({"enabled":false,"message":"Planned <work>","retryAfterSecs":73});
        assert_eq!(
            admin
                .put(&endpoint)
                .header("if-match", enabled_etag)
                .json(&off)
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        let restored = user.get(format!("{base}/api/me")).send().await.unwrap();
        assert_eq!(restored.status(), 200);
        assert_eq!(
            restored.json::<Value>().await.unwrap()["accountId"],
            user_id
        );
        store
            .set_setting("maintenance.mode.v1", "invalid private configuration")
            .await
            .unwrap();
        assert_eq!(
            user.get(format!("{base}/api/me"))
                .send()
                .await
                .unwrap()
                .status(),
            503
        );
        assert_eq!(
            anonymous
                .get(format!("{base}/api/public-config"))
                .send()
                .await
                .unwrap()
                .status(),
            503
        );
        let repair = admin.get(&endpoint).send().await.unwrap();
        assert_eq!(repair.status(), 200);
        let repair_etag = repair.headers()["etag"].to_str().unwrap().to_owned();
        let repair_body = repair.json::<Value>().await.unwrap();
        assert_eq!(repair_body["configurationInvalid"], true);
        assert_eq!(repair_body["enabled"], true);
        assert_eq!(repair_body["message"], "");
        store
            .set_setting("maintenance.mode.v1", "different invalid configuration")
            .await
            .unwrap();
        assert_eq!(
            admin
                .put(&endpoint)
                .header("if-match", repair_etag)
                .json(&off)
                .send()
                .await
                .unwrap()
                .status(),
            412
        );
        let repair = admin.get(&endpoint).send().await.unwrap();
        let repair_etag = repair.headers()["etag"].to_str().unwrap().to_owned();
        assert_eq!(
            admin
                .put(&endpoint)
                .header("if-match", repair_etag)
                .json(&off)
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        assert_eq!(
            user.get(format!("{base}/api/me"))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        server.abort();
    }
    unsafe {
        match previous_base {
            Some(v) => std::env::set_var("MW_BASE_PATH", v),
            None => std::env::remove_var("MW_BASE_PATH"),
        };
        match previous_admin {
            Some(v) => std::env::set_var("MW_ADMIN_PATH", v),
            None => std::env::remove_var("MW_ADMIN_PATH"),
        };
    }
}
