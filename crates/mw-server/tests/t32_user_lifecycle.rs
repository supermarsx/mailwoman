use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
use std::net::SocketAddr;
mod common;
const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
async fn serve(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
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
async fn signin(client: &reqwest::Client, base: &str) -> reqwest::Response {
    tokio::time::sleep(std::time::Duration::from_millis(3100)).await;
    client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap()
}
#[tokio::test]
async fn local_user_access_requires_proof_scope_and_revision_and_preserves_upstream() {
    use base64::Engine as _;
    let alias_header = format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD
            .encode(format!("testuser@alias.test:{}", mw_mock_jmap::PASS))
    );
    let canonical_header = format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(format!(
            "{}:{}",
            mw_mock_jmap::USER,
            mw_mock_jmap::PASS
        ))
    );
    let upstream_router = mw_mock_jmap::router().layer(axum::middleware::from_fn(
        move |mut request: axum::extract::Request, next: axum::middleware::Next| {
            let alias = alias_header.clone();
            let canonical = canonical_header.clone();
            async move {
                if request
                    .headers()
                    .get("authorization")
                    .and_then(|h| h.to_str().ok())
                    == Some(alias.as_str())
                {
                    request
                        .headers_mut()
                        .insert("authorization", canonical.parse().unwrap());
                }
                next.run(request).await
            }
        },
    ));
    let upstream = serve(upstream_router).await;
    let db = common::test_db::unique_db_path("user-lifecycle");
    let store = mw_store::Store::open(
        db.to_str().unwrap(),
        mw_store::ServerKey::from_hex(KEY).unwrap(),
    )
    .await
    .unwrap();
    let (router, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(KEY.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(upstream.clone()),
                jmap_upstreams: Some(vec![upstream.clone()]),
                ..SecurityConfig::default()
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
    let base = serve(router).await;
    let user = client();
    assert_eq!(signin(&user, &base).await.status(), 200);
    let endpoint = format!("{base}/admin/users/{}/lifecycle", mw_mock_jmap::ACCOUNT_ID);
    assert_eq!(client().get(&endpoint).send().await.unwrap().status(), 401);
    tokio::time::sleep(std::time::Duration::from_millis(3100)).await;
    let admin = client();
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
    let read = admin.get(&endpoint).send().await.unwrap();
    assert_eq!(read.status(), 200);
    let initial = read.headers()["etag"].clone();
    assert_eq!(read.json::<Value>().await.unwrap()["status"], "active");
    assert_eq!(
        admin
            .delete(&endpoint)
            .header("if-match", initial.clone())
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        admin
            .post(format!("{base}/admin/step-up"))
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
            .json(&json!({"disabled":true}))
            .send()
            .await
            .unwrap()
            .status(),
        428
    );
    // Clearing a legacy flag has its own revision and preserves unrelated flags.
    let legacy_key = format!("v6:admin:flags:{}", mw_mock_jmap::ACCOUNT_ID);
    let legacy_url = format!(
        "{base}/admin/users/{}/legacy-block",
        mw_mock_jmap::ACCOUNT_ID
    );
    let flags = mw_admin::UserFeatureFlags {
        disabled: true,
        force_password_change: true,
        ..Default::default()
    };
    store
        .set_setting(&legacy_key, &serde_json::to_string(&flags).unwrap())
        .await
        .unwrap();
    let legacy = admin.get(&legacy_url).send().await.unwrap();
    assert_eq!(legacy.status(), 200);
    let legacy_tag = legacy.headers()["etag"].clone();
    assert_eq!(
        admin.delete(&legacy_url).send().await.unwrap().status(),
        428
    );
    assert_eq!(
        admin
            .delete(&legacy_url)
            .header("if-match", "\"stale\"")
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    assert_eq!(
        admin
            .delete(&legacy_url)
            .header("if-match", legacy_tag)
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    let preserved: mw_admin::UserFeatureFlags =
        serde_json::from_str(&store.get_setting(&legacy_key).await.unwrap().unwrap()).unwrap();
    assert!(!preserved.disabled);
    assert!(preserved.force_password_change);
    store
        .set_setting(
            &legacy_key,
            &serde_json::to_string(&mw_admin::UserFeatureFlags::default()).unwrap(),
        )
        .await
        .unwrap();
    store
        .put_domain_config(&mw_store::DomainConfigRow {
            name: "example.org".into(),
            upstream_json: "{}".into(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    store
        .put_domain_alias("alias.test", "example.org")
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(3100)).await;
    let alias_user = client();
    assert_eq!(
        alias_user
            .post(format!("{base}/api/login"))
            .json(&json!({"username":"testuser@alias.test","password":mw_mock_jmap::PASS}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let blocked = admin
        .put(&endpoint)
        .header("if-match", initial.clone())
        .json(&json!({"disabled":true}))
        .send()
        .await
        .unwrap();
    assert_eq!(blocked.status(), 200);
    let blocked_tag = blocked.headers()["etag"].clone();
    assert_eq!(
        user.get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(signin(&client(), &base).await.status(), 401);
    assert_eq!(
        alias_user
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    tokio::time::sleep(std::time::Duration::from_millis(3100)).await;
    assert_eq!(
        client()
            .post(format!("{base}/api/login"))
            .json(&json!({"username":"testuser@alias.test","password":mw_mock_jmap::PASS}))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );

    assert_eq!(
        admin
            .delete(&endpoint)
            .header("if-match", initial)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    let active = admin
        .put(&endpoint)
        .header("if-match", blocked_tag)
        .json(&json!({"disabled":false}))
        .send()
        .await
        .unwrap();
    assert_eq!(active.status(), 200);
    let active_tag = active.headers()["etag"].clone();
    let replacement = client();
    assert_eq!(signin(&replacement, &base).await.status(), 200);
    let removed = admin
        .delete(&endpoint)
        .header("if-match", active_tag)
        .send()
        .await
        .unwrap();
    assert_eq!(removed.status(), 200);
    let deleted_tag = removed.headers()["etag"].clone();
    let deleted: Value = removed.json().await.unwrap();
    assert_eq!(deleted["status"], "deleted");
    assert_eq!(deleted["canRestore"], false);
    assert_eq!(
        replacement
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(signin(&client(), &base).await.status(), 401);
    assert_eq!(
        admin
            .put(&endpoint)
            .header("if-match", deleted_tag)
            .json(&json!({"disabled":false}))
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    assert_eq!(
        store
            .lifecycle_account_bindings(mw_mock_jmap::USER)
            .await
            .unwrap(),
        vec![mw_mock_jmap::ACCOUNT_ID]
    );
    let rows: Value = admin
        .get(format!("{base}/admin/users"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(rows[0]["status"], "deleted");
    // Mailwoman never sent a deletion to the upstream. Its independently authenticated session still works.
    assert_eq!(
        client()
            .get(format!("{upstream}/jmap/session"))
            .basic_auth(mw_mock_jmap::USER, Some(mw_mock_jmap::PASS))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    // Provisioned users can be tombstoned before their first successful sign-in.
    assert_eq!(admin.post(format!("{base}/admin/users")).json(&json!({"domain":"example.org","username":"Never-Signed-In","quota":{"bytesLimit":0,"msgLimit":0}})).send().await.unwrap().status(),204);
    let provisioned_url = format!("{base}/admin/users/Never-Signed-In%40example.org/lifecycle");
    let provisioned = admin.get(&provisioned_url).send().await.unwrap();
    assert_eq!(provisioned.status(), 200);
    let provisioned_tag = provisioned.headers()["etag"].clone();
    assert_eq!(
        admin
            .delete(&provisioned_url)
            .header("if-match", provisioned_tag)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert!(
        store
            .directory_access_held("never-signed-in@example.org")
            .await
            .unwrap()
    );
    assert!(
        store
            .get_admin_user("Never-Signed-In@example.org")
            .await
            .unwrap()
            .is_some()
    );
    // The separate administrator principal and its privileges remain usable.
    assert_eq!(
        admin
            .get(format!("{base}/admin/users"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let role:Value=admin.post(format!("{base}/admin/roles")).json(&json!({"name":"wrong domain","capabilities":["users.read","users.delete","users.flags"]})).send().await.unwrap().json().await.unwrap();
    let created=admin.post(format!("{base}/admin/admins")).json(&json!({"username":"scoped","password":"long scoped password","roles":[{"roleId":role["id"],"domains":["elsewhere.test"]}]})).send().await.unwrap();
    assert!(created.status().is_success());
    tokio::time::sleep(std::time::Duration::from_millis(3100)).await;
    let scoped = client();
    assert_eq!(
        scoped
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"scoped","password":"long scoped password"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(scoped.get(&endpoint).send().await.unwrap().status(), 403);
    assert_eq!(scoped.delete(&endpoint).send().await.unwrap().status(), 403);
}
