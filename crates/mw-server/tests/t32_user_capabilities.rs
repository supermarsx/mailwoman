//! Profile edits change live feature admission through the production router.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;

async fn serve(app: axum::Router) -> String {
    let socket = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = socket.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            socket,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{addr}")
}
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap()
}
async fn send(req: reqwest::RequestBuilder) -> (u16, Value) {
    let r = req.send().await.unwrap();
    let status = r.status().as_u16();
    let text = r.text().await.unwrap();
    (
        status,
        serde_json::from_str(&text).unwrap_or(Value::String(text)),
    )
}
#[tokio::test]
async fn profile_group_and_global_lock_drive_live_feature_admission() {
    let upstream_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed_calls = upstream_calls.clone();
    let upstream = serve(mw_mock_jmap::router().layer(axum::middleware::from_fn(
        move |request: axum::extract::Request, next: axum::middleware::Next| {
            let calls = observed_calls.clone();
            async move {
                calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                next.run(request).await
            }
        },
    )))
    .await;
    let root = common::test_db::unique_dir("mw-feature-http");
    let config = AppConfig {
        db_path: root.join("db.sqlite").to_string_lossy().into_owned(),
        server_key_hex: Some(
            "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b".into(),
        ),
        web_dir: None,
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            upstream_default: Some(upstream.clone()),
            jmap_upstreams: Some(vec![upstream]),
            ..Default::default()
        },
    };
    // This fixture exercises feature admission, not the shared sign-in delay.
    let store = mw_store::Store::open(
        &config.db_path,
        mw_store::ServerKey::from_hex(config.server_key_hex.as_ref().unwrap()).unwrap(),
    )
    .await
    .unwrap();
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "feature-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    let base = serve(
        build_app_full(
            config,
            V6Config {
                admin_enabled: true,
                admin_username: Some("root".into()),
                admin_password: Some("hunter2".into()),
                redis_url: None,
            },
        )
        .await
        .unwrap()
        .0,
    )
    .await;
    let user = client();
    let admin = client();
    assert_eq!(
        send(client().get(format!("{base}/api/features"))).await.0,
        401
    );
    let (status, me) = send(
        user.post(format!("{base}/api/login"))
            .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS})),
    )
    .await;
    assert_eq!(status, 200, "{me}");
    let account = me["accountId"].as_str().unwrap();
    assert_eq!(
        send(
            admin
                .post(format!("{base}/admin/login"))
                .json(&json!({"username":"root","password":"hunter2"}))
        )
        .await
        .0,
        200
    );
    assert_eq!(
        send(user.get(format!("{base}/admin/features"))).await.0,
        401
    );
    assert_eq!(
        send(user.get(format!("{base}/api/import/formats"))).await.0,
        200
    );
    let profile = json!({"name":"Restricted","values":{"cap.import":{"value":false,"locked":true},"cap.ai_assistant":{"value":false}}});
    let (status, body) = send(
        admin
            .put(format!("{base}/admin/features/profiles/restricted"))
            .json(&profile),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        send(
            admin
                .put(format!("{base}/admin/features/groups/team"))
                .json(&json!({"name":"Team"}))
        )
        .await
        .0,
        200
    );
    assert_eq!(
        send(
            admin
                .put(format!("{base}/admin/features/groups/team/members"))
                .json(&json!({"members":[account]}))
        )
        .await
        .0,
        200
    );
    assert_eq!(send(admin.put(format!("{base}/admin/features/assignments")).json(&json!({"scopeKind":"group","scopeId":"team","profileId":"restricted","priority":10}))).await.0,200);
    let (status, body) = send(user.get(format!("{base}/api/import/formats"))).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "feature_disabled");

    // No call in a mixed batch may reach the upstream when one feature is off.
    for (key, method, feature) in [
        ("cap.import", "Email/import", "import"),
        ("cap.pim.calendar", "CalendarEvent/set", "calendar"),
        ("cap.pim.contacts", "ContactGroup/get", "contacts"),
        ("cap.pim.tasks", "Task/set", "tasks"),
        ("cap.pim.notes", "Note/get", "notes"),
        ("cap.rules", "MailRule/set", "rules"),
    ] {
        assert_eq!(
            send(
                admin
                    .put(format!("{base}/admin/settings/{key}"))
                    .json(&json!({"globalValue":false}))
            )
            .await
            .0,
            200
        );
        let before = upstream_calls.load(std::sync::atomic::Ordering::SeqCst);
        let (status, body) = send(user.post(format!("{base}/jmap/api")).json(&json!({
            "using": ["urn:ietf:params:jmap:core"],
            "methodCalls": [["Email/set", {"accountId":account,"destroy":["message"]}, "first"], [method,{"accountId":account},"second"]]
        }))).await;
        assert_eq!(status, 403, "{body}");
        assert_eq!(body["feature"], feature);
        assert_eq!(
            upstream_calls.load(std::sync::atomic::Ordering::SeqCst),
            before,
            "disabled batch must not reach upstream"
        );
    }
    assert_eq!(
        send(
            admin
                .put(format!("{base}/admin/settings/cap.export"))
                .json(&json!({"globalValue":false}))
        )
        .await
        .0,
        200
    );
    let (status, body) = send(user.get(format!("{base}/api/export/missing"))).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["feature"], "export");
    let (_, body) = send(user.get(format!("{base}/api/features"))).await;
    assert_eq!(body["features"]["import"]["enabled"], false);
    assert_eq!(
        send(
            user.put(format!("{base}/api/settings/cap.import"))
                .json(&json!({"value":true}))
        )
        .await
        .0,
        404 // Admin-only capabilities are not writable through user settings.
    );
    // A broader lock beats the profile immediately, without session renewal.
    assert_eq!(
        send(
            admin
                .put(format!("{base}/admin/settings/cap.import"))
                .json(&json!({"globalValue":true,"locked":true}))
        )
        .await
        .0,
        200
    );
    assert_eq!(
        send(user.get(format!("{base}/api/import/formats"))).await.0,
        200
    );
    // Bad writes never replace the existing profile.
    assert_eq!(
        send(
            admin
                .put(format!("{base}/admin/features/profiles/restricted"))
                .json(&json!({"name":"Restricted","values":{"cap.import":{"value":"yes"}}}))
        )
        .await
        .0,
        422
    );
    let (_, body) = send(admin.get(format!("{base}/admin/features"))).await;
    assert_eq!(body["profiles"][0]["values"]["cap.import"]["value"], false);
    assert_eq!(
        send(admin.delete(format!("{base}/admin/features/groups/team")))
            .await
            .0,
        200
    );
    let (_, body) = send(admin.get(format!("{base}/admin/features"))).await;
    assert!(body["assignments"].as_array().unwrap().is_empty());
    assert_eq!(
        send(user.get(format!("{base}/api/nextcloud/list"))).await.0,
        409
    );
    assert_eq!(
        send(
            admin
                .put(format!("{base}/admin/settings/cap.integration.nextcloud"))
                .json(&json!({"globalValue":false}))
        )
        .await
        .0,
        200
    );
    let (status, body) = send(user.get(format!("{base}/api/nextcloud/list"))).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["feature"], "integrations.nextcloud");
    assert_eq!(
        send(
            admin
                .put(format!("{base}/admin/settings/cap.ui_plugin.50d858e0985ecc7f60418aaf0cc5ab587f42c2570a884095a9e8ccacd0f6545c"))
                .json(&json!({"globalValue":false}))
        )
        .await
        .0,
        200
    );
    let (status, body) = send(
        user.post(format!("{base}/api/ui-plugins/example/rpc"))
            .json(&json!({"v":1,"id":"request1","cap":"store:kv-scoped","method":"get","args":[]})),
    )
    .await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "feature_disabled");
}
