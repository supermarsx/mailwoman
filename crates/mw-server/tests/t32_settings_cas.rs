use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;
async fn read(c: &reqwest::Client, base: &str, path: &str) -> (Value, String) {
    let r = c.get(format!("{base}{path}")).send().await.unwrap();
    assert_eq!(r.status(), 200);
    let tag = r.headers()["etag"].to_str().unwrap().to_owned();
    (r.json().await.unwrap(), tag)
}
#[tokio::test]
async fn registry_and_branding_http_share_conditional_writes_and_public_snapshot() {
    let (app, _) = build_app_full(
        AppConfig {
            db_path: common::test_db::unique_db_path("settings-cas")
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
        c.post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    for n in 0..8 {
        let (brand, brand_tag) = read(&c, &base, "/admin/branding").await;
        let (_, policy_tag) = read(&c, &base, "/admin/settings/appearance.brandName").await;
        let brand_write=c.put(format!("{base}/admin/branding")).header("If-Match",brand_tag).json(&json!({"brandName":format!("Editor {n}"),"loginNotice":format!("Notice {n}"),"images":brand["images"]}));
        let policy_write = c
            .put(format!("{base}/admin/settings/appearance.brandName"))
            .header("If-Match", &policy_tag)
            .json(&json!({"globalValue":format!("Registry {n}")}));
        let (a, b) = tokio::join!(brand_write.send(), policy_write.send());
        let a = a.unwrap().status().as_u16();
        let b = b.unwrap().status().as_u16();
        assert!(
            matches!((a, b), (200, 412) | (412, 200)),
            "one writer wins, one is stale: {a}/{b}"
        );
        assert_eq!(
            c.delete(format!("{base}/admin/settings/appearance.brandName"))
                .header("If-Match", policy_tag)
                .send()
                .await
                .unwrap()
                .status(),
            412
        );
    }
    // Every public branding response must represent one complete multi-field update.
    let (_, tag) = read(&c, &base, "/admin/branding").await;
    assert_eq!(c.put(format!("{base}/admin/branding")).header("If-Match",tag).json(&json!({"brandName":"0","loginNotice":"0","images":{"logoId":null,"iconMasterId":null,"loginBackgroundId":null}})).send().await.unwrap().status(),200);
    let writer = async {
        for n in 1..12 {
            let (value, tag) = read(&c, &base, "/admin/branding").await;
            assert_eq!(c.put(format!("{base}/admin/branding")).header("If-Match",tag).json(&json!({"brandName":n.to_string(),"loginNotice":n.to_string(),"images":value["images"]})).send().await.unwrap().status(),200);
        }
    };
    let reader = async {
        for _ in 0..32 {
            let public: Value = c
                .get(format!("{base}/api/public-config"))
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            assert_eq!(
                public["brandName"], public["loginNotice"],
                "public response mixed revisions"
            );
        }
    };
    tokio::join!(writer, reader);
}
