use base64::Engine as _;
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;
#[tokio::test]
async fn branding_publishes_only_committed_safe_assets_and_requires_cas() {
    let (app, _) = build_app_full(
        AppConfig {
            db_path: common::test_db::unique_db_path("branding")
                .to_string_lossy()
                .into(),
            server_key_hex: None,
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
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
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let c = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    assert_eq!(
        c.get(format!("{base}/admin/branding"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        c.post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let r = c
        .get(format!("{base}/admin/branding"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let tag = r.headers()["etag"].to_str().unwrap().to_owned();
    let mut v: Value = r.json().await.unwrap();
    for bytes in [
        b"<svg xmlns='http://www.w3.org/2000/svg'/>".as_slice(),
        b"not an image",
    ] {
        assert_eq!(c.post(format!("{base}/admin/branding/assets")).json(&json!({"kind":"logo","bytesBase64":base64::engine::general_purpose::STANDARD.encode(bytes)})).send().await.unwrap().status(),422);
    }
    let png = include_bytes!("../../../apps/web/public/apple-touch-icon.png");
    let r=c.post(format!("{base}/admin/branding/assets")).json(&json!({"kind":"logo","bytesBase64":base64::engine::general_purpose::STANDARD.encode(png)})).send().await.unwrap();
    assert_eq!(r.status(), 200, "{}", r.text().await.unwrap());
    let r=c.post(format!("{base}/admin/branding/assets")).json(&json!({"kind":"logo","bytesBase64":base64::engine::general_purpose::STANDARD.encode(png)})).send().await.unwrap();
    let uploaded: Value = r.json().await.unwrap();
    let id = uploaded["id"].as_str().unwrap();
    let url = format!("{base}/api/assets/{id}");
    assert_eq!(c.get(&url).send().await.unwrap().status(), 404);
    v["brandName"] = json!("Example Mail");
    v["loginNotice"] = json!("Welcome here");
    v["images"]["logoId"] = json!(id);
    v.as_object_mut().unwrap().remove("assets");
    assert_eq!(
        c.put(format!("{base}/admin/branding"))
            .json(&v)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    let r = c
        .put(format!("{base}/admin/branding"))
        .header("If-Match", &tag)
        .json(&v)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "{}", r.text().await.unwrap());
    assert_eq!(
        c.put(format!("{base}/admin/branding"))
            .header("If-Match", &tag)
            .json(&v)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    let public: Value = c
        .get(format!("{base}/api/public-config"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(public["brandName"], "Example Mail");
    assert_eq!(public["logoUrl"], format!("/api/assets/{id}"));
    let r = reqwest::get(&url).await.unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(r.headers()["content-type"], "image/png");
    assert_eq!(r.headers()["x-content-type-options"], "nosniff");
    let r = c
        .get(format!("{base}/admin/branding"))
        .send()
        .await
        .unwrap();
    let tag = r.headers()["etag"].to_str().unwrap().to_owned();
    v["images"]["logoId"] = Value::Null;
    assert_eq!(
        c.put(format!("{base}/admin/branding"))
            .header("If-Match", tag)
            .json(&v)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(c.get(url).send().await.unwrap().status(), 404);
    let proof = c
        .post(format!("{base}/admin/step-up"))
        .json(&json!({"password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(proof.status(), 200, "{}", proof.text().await.unwrap());
    let role = c
        .post(format!("{base}/admin/roles"))
        .json(&json!({"name":"Scoped branding","capabilities":["branding.read","branding.manage"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(role.status(), 201);
    let role: Value = role.json().await.unwrap();
    let account=c.post(format!("{base}/admin/admins")).json(&json!({"username":"brand-scoped","password":"another-secret-password","roles":[{"roleId":role["id"],"domains":["example.org"]}]})).send().await.unwrap();
    assert_eq!(account.status(), 201, "{}", account.text().await.unwrap());
    let scoped = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    assert_eq!(
        scoped
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"brand-scoped","password":"another-secret-password"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        scoped
            .get(format!("{base}/admin/branding"))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        scoped
            .put(format!("{base}/admin/branding"))
            .json(&v)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        scoped
            .post(format!("{base}/admin/branding/assets"))
            .json(&json!({"kind":"logo","bytesBase64":""}))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
}
