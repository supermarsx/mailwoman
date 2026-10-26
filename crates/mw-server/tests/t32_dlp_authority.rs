//! DLP authority is checked before any proxy batch operation runs.
use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
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
async fn governed_proxy_batch_fails_closed_before_any_upstream_mutation() {
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed = calls.clone();
    let upstream = serve(mw_mock_jmap::router().layer(axum::middleware::from_fn(
        move |request: axum::extract::Request, next: axum::middleware::Next| {
            let calls = observed.clone();
            async move {
                calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                next.run(request).await
            }
        },
    )))
    .await;
    let root = common::test_db::unique_dir("dlp-authority");
    let key = "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";
    let db = root.join("db.sqlite").to_string_lossy().into_owned();
    let store = mw_store::Store::open(&db, mw_store::ServerKey::from_hex(key).unwrap())
        .await
        .unwrap();
    let base = serve(
        build_app(AppConfig {
            db_path: db,
            server_key_hex: Some(key.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: Default::default(),
            security: SecurityConfig {
                upstream_default: Some(upstream.clone()),
                jmap_upstreams: Some(vec![upstream]),
                ..Default::default()
            },
        })
        .await
        .unwrap(),
    )
    .await;
    let user = client();
    let (status, login) = send(
        user.post(format!("{base}/api/login"))
            .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS})),
    )
    .await;
    assert_eq!(status, 200, "{login}");
    let before = calls.load(std::sync::atomic::Ordering::SeqCst);
    let batch = json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail","urn:ietf:params:jmap:submission"],"methodCalls":[["Email/set",{"create":{"draft":{"subject":"not forwarded"}}},"a"],["EmailSubmission/set",{"create":{"send":{"emailId":"#draft"}}},"b"]]});
    store
        .set_setting(
            "v6:admin:security_policy",
            r#"{"dlp_rules_json":"malformed"}"#,
        )
        .await
        .unwrap();
    let (status, result) = send(user.post(format!("{base}/jmap/api")).json(&batch)).await;
    assert_eq!(status, 503, "{result}");
    assert_eq!(result["code"], "dlpPolicyUnavailable");
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), before);
    let rules = json!([{"id":"block","name":"Sensitive","enabled":true,"priority":1,"conditions":{"detectors":["pan"],"customRegex":null,"dictionaries":[],"attachmentTypes":[],"maxAttachmentSize":null,"recipientDomains":[],"recipientDomainMode":null,"classification":null},"action":"block","message":"Restricted"}]);
    store
        .set_setting(
            "v6:admin:security_policy",
            &json!({"dlp_rules_json":rules.to_string()}).to_string(),
        )
        .await
        .unwrap();
    let (status, result) = send(user.post(format!("{base}/jmap/api")).json(&batch)).await;
    assert_eq!(status, 403, "{result}");
    assert_eq!(result["code"], "dlpProxyUnsupported");
    assert_eq!(
        calls.load(std::sync::atomic::Ordering::SeqCst),
        before,
        "not even first mutation forwarded"
    );
    let cancel = json!({"methodCalls":[["EmailSubmission/set",{"update":{"queued":{"undoStatus":"canceled"}}},"cancel"]]});
    let (status, result) = send(user.post(format!("{base}/jmap/api")).json(&cancel)).await;
    assert_eq!(status, 200, "{result}");
    assert!(calls.load(std::sync::atomic::Ordering::SeqCst) > before);
    let before = calls.load(std::sync::atomic::Ordering::SeqCst);
    store
        .set_setting("v6:admin:security_policy", r#"{"dlp_rules_json":"[]"}"#)
        .await
        .unwrap();
    let (status, result) = send(user.post(format!("{base}/jmap/api")).json(&batch)).await;
    assert_eq!(status, 200, "{result}");
    assert!(calls.load(std::sync::atomic::Ordering::SeqCst) > before);
}
