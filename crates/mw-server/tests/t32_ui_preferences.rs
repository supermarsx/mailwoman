use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
use std::net::SocketAddr;
mod common;

async fn serve(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{address}")
}

#[tokio::test]
async fn authenticated_preferences_use_cas_and_reject_foreign_authority_fields() {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("ui-preferences");
    let (router, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: None,
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(upstream.clone()),
                jmap_upstreams: Some(vec![upstream]),
                ..SecurityConfig::default()
            },
        },
        V6Config::default(),
    )
    .await
    .unwrap();
    let base = serve(router).await;
    let endpoint = format!("{base}/api/account/preferences");
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    assert_eq!(client.get(&endpoint).send().await.unwrap().status(), 401);
    let login = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "{}", login.text().await.unwrap());
    let initial = client.get(&endpoint).send().await.unwrap();
    assert_eq!(initial.status(), 200);
    assert_eq!(initial.headers()["cache-control"], "private, no-store");
    assert_eq!(initial.headers()["etag"], "\"0\"");
    let document = json!({"version":1,"preferences":{"markRead":"never","folderPinned":["inbox"],"folderIcons":{"inbox":"star"}}});
    assert_eq!(
        client
            .put(&endpoint)
            .json(&document)
            .send()
            .await
            .unwrap()
            .status(),
        428
    );
    let saved = client
        .put(&endpoint)
        .header("if-match", "\"0\"")
        .json(&document)
        .send()
        .await
        .unwrap();
    assert_eq!(saved.status(), 200);
    let revision = saved.headers()["etag"].clone();
    assert_eq!(saved.json::<Value>().await.unwrap(), document);
    assert_eq!(
        client
            .put(&endpoint)
            .header("if-match", "\"0\"")
            .json(&document)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    let invalid = json!({"version":1,"preferences":{"accountId":"victim","password":"secret"}});
    assert_eq!(
        client
            .put(&endpoint)
            .header("if-match", revision.clone())
            .json(&invalid)
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    let latest = client
        .get(&endpoint)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(latest, document);
    for preferences in [
        json!({"folderSidebarWidth":159}),
        json!({"workspaceSplitRatio":0.9}),
        json!({"folderIcons":{"inbox":"https://external.invalid/avatar"}}),
    ] {
        assert_eq!(
            client
                .put(&endpoint)
                .header("if-match", revision.clone())
                .json(&json!({"version":1,"preferences":preferences}))
                .send()
                .await
                .unwrap()
                .status(),
            400
        );
    }
    let first = client
        .put(&endpoint)
        .header("if-match", revision.clone())
        .json(&json!({"version":1,"preferences":{"direction":"rtl"}}));
    let second = client
        .put(&endpoint)
        .header("if-match", revision)
        .json(&json!({"version":1,"preferences":{"markRead":"on-open"}}));
    let (first, second) = tokio::join!(first.send(), second.send());
    let mut statuses = [
        first.unwrap().status().as_u16(),
        second.unwrap().status().as_u16(),
    ];
    statuses.sort();
    assert_eq!(
        statuses,
        [200, 412],
        "only one concurrent device may replace the same version"
    );
}
