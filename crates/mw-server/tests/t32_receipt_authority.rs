//! Receipt authority is checked before any proxy batch operation runs.
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
async fn held_never_blocks_entire_proxy_batch_before_upstream() {
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
    let root = common::test_db::unique_dir("receipt-authority");
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
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "mail.readReceipts".into(),
            global_value: Some("\"never\"".into()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "admin".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    let before = calls.load(std::sync::atomic::Ordering::SeqCst);
    let (status,result)=send(user.post(format!("{base}/jmap/api")).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/set",{"create":{"first":{}}},"a"],["MDN/send",{"emailId":"message","automatic":false},"b"]]}))).await;
    assert_eq!(status, 403, "{result}");
    assert_eq!(
        calls.load(std::sync::atomic::Ordering::SeqCst),
        before,
        "no first mutation or receipt reaches upstream"
    );
    let rewrite = json!({"methodCalls":[["Email/set",{"accountId":mw_mock_jmap::ACCOUNT_ID,"update":{"newly-created-draft":{"bodyStructure/type":"message/disposition-notification"}}},"edit"]]});
    let (status, result) = send(user.post(format!("{base}/jmap/api")).json(&rewrite)).await;
    assert_eq!(
        status, 403,
        "parallel MIME rewrite must be denied: {result}"
    );
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), before);
    for draft in [
        json!({"bodyStructure":{"type":"multipart/report","subParts":[{"type":"message/disposition-notification"}]}}),
        json!({"header:Content-Type:asRaw":"message/disposition-notification"}),
        json!({"blobId":"opaque-raw"}),
    ] {
        let request = json!({"methodCalls":[["Email/set",{"accountId":mw_mock_jmap::ACCOUNT_ID,"create":{"draft":draft}},"e"],["EmailSubmission/set",{"accountId":mw_mock_jmap::ACCOUNT_ID,"create":{"send":{"emailId":"#draft"}}},"s"]]});
        let (status, result) = send(user.post(format!("{base}/jmap/api")).json(&request)).await;
        assert_eq!(status, 403, "{result}");
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::SeqCst),
            before,
            "raw report is refused before the draft mutation"
        );
    }
    let ordinary = json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail","urn:ietf:params:jmap:submission"],"methodCalls":[["Email/set",{"accountId":mw_mock_jmap::ACCOUNT_ID,"create":{"draft":{"from":[{"email":mw_mock_jmap::USER}],"to":[{"email":"friend@example.org"}],"subject":"ordinary composition","textBody":[{"partId":"p","type":"text/plain"}],"bodyValues":{"p":{"value":"hello"}}}}},"e"],["EmailSubmission/set",{"accountId":mw_mock_jmap::ACCOUNT_ID,"create":{"send":{"emailId":"#draft"}}},"s"]]});
    let mut success_rewrite = ordinary.clone();
    success_rewrite["methodCalls"][1][1]["onSuccessUpdateEmail"] =
        json!({"#send":{"bodyStructure/type":"message/disposition-notification"}});
    let (status, result) = send(user.post(format!("{base}/jmap/api")).json(&success_rewrite)).await;
    assert_eq!(status, 403, "success MIME rewrite must be denied: {result}");
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), before);
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "receipts.dsnAllowed".into(),
            global_value: Some("false".into()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "admin".into(),
            updated_at: 2,
        })
        .await
        .unwrap();
    let mut dsn_request = ordinary.clone();
    dsn_request["methodCalls"][1][1]["create"]["send"]["envelope"] = json!({"mailFrom":{"email":mw_mock_jmap::USER,"parameters":{"RET":"HDRS"}},"rcptTo":[{"email":"friend@example.org","parameters":{"NOTIFY":"FAILURE"}}]});
    let (status, result) = send(user.post(format!("{base}/jmap/api")).json(&dsn_request)).await;
    assert_eq!(
        status, 403,
        "DSN envelope policy must apply in proxy mode: {result}"
    );
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), before);
    let (status, result) = send(user.post(format!("{base}/jmap/api")).json(&ordinary)).await;
    assert_eq!(
        status, 200,
        "ordinary composition remains admitted: {result}"
    );
    assert!(calls.load(std::sync::atomic::Ordering::SeqCst) > before);
}
