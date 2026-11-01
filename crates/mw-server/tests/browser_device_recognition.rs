use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::Request,
    middleware::Next,
    response::IntoResponse,
};
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, build_app};
use mw_store::{ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
mod common;

async fn serve(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    url
}
async fn captured_upstream(captured: Arc<Mutex<Vec<(Value, String)>>>) -> String {
    let uploaded = Arc::new(Mutex::new(String::new()));
    let router=mw_mock_jmap::router().layer(axum::middleware::from_fn(move |request:Request,next:Next| {
        let captured=captured.clone();let uploaded=uploaded.clone();
        async move {
            let path=request.uri().path().to_string();
            if request.method()==axum::http::Method::POST && (path=="/jmap" || path.starts_with("/jmap/upload/")) {
                let (parts,body)=request.into_parts();let bytes=to_bytes(body,1024*1024).await.unwrap();
                if path.starts_with("/jmap/upload/") {*uploaded.lock().unwrap()=String::from_utf8(bytes.to_vec()).unwrap();}
                else if let Ok(body)=serde_json::from_slice::<Value>(&bytes) {
                    if body["methodCalls"][0][0]=="Email/import" {
                        captured.lock().unwrap().push((body.clone(),uploaded.lock().unwrap().clone()));
                        return Json(json!({"methodResponses":[["Email/import",{"accountId":mw_mock_jmap::ACCOUNT_ID,"created":{"alert":{"id":"delivered-notice"}}},"alert"]],"sessionState":"s"})).into_response();
                    }
                }
                return next.run(Request::from_parts(parts,Body::from(bytes))).await;
            }
            next.run(request).await
        }
    }));
    serve(router).await
}
async fn fixture(secure: bool) -> (String, Store, Arc<Mutex<Vec<(Value, String)>>>) {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let upstream = captured_upstream(captured.clone()).await;
    let path = common::test_db::unique_db_path("browser-device-recognition");
    let key = "6b69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";
    let store = Store::open(path.to_str().unwrap(), ServerKey::from_hex(key).unwrap())
        .await
        .unwrap();
    store
        .put_setting_policy(&SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    let router = build_app(AppConfig {
        db_path: path.to_string_lossy().into(),
        server_key_hex: Some(key.into()),
        web_dir: None,
        cookie_secure: secure,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            upstream_default: Some(upstream.clone()),
            jmap_upstreams: Some(vec![upstream]),
            ..Default::default()
        },
    })
    .await
    .unwrap();
    (serve(router).await, store, captured)
}
fn browser(jar: Arc<reqwest::cookie::Jar>, agent: &str, ip: u8) -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_provider(jar)
        .user_agent(agent)
        .local_address(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, ip)))
        .build()
        .unwrap()
}
async fn login(client: &reqwest::Client, base: &str, invalid_cookie: bool) -> reqwest::Response {
    let mut request = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}));
    if invalid_cookie {
        request = request.header("cookie", "mw_browser_device=invalid");
    }
    let response = request.send().await.unwrap();
    assert_eq!(response.status(), 200);
    response
}
async fn wait_notices(captured: &Arc<Mutex<Vec<(Value, String)>>>, expected: usize) {
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            if captured.lock().unwrap().len() >= expected {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
}
#[tokio::test]
async fn login_recognizes_browser_marker_across_logout_agent_and_network_changes() {
    let (base, store, captured) = fixture(false).await;
    let jar = Arc::new(reqwest::cookie::Jar::default());
    let first = browser(jar.clone(), "Browser/1.0", 2);
    let response = login(&first, &base, false).await;
    let cookie = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find(|value| value.starts_with("mw_browser_device="))
        .unwrap()
        .to_string();
    assert!(
        cookie.contains("HttpOnly")
            && cookie.contains("SameSite=Lax")
            && cookie.contains("Max-Age=31536000")
    );
    assert!(!cookie.contains("Domain="));
    let token = cookie.split(';').next().unwrap().split_once('=').unwrap().1;
    assert_eq!(token.len(), 64);
    let body: Value = response.json().await.unwrap();
    assert!(!body.to_string().contains(token));
    wait_notices(&captured, 1).await;
    assert_eq!(
        first
            .post(format!("{base}/api/logout"))
            .header("x-csrf-token", body["csrfToken"].as_str().unwrap())
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    let returning = browser(jar, "Browser/2.0", 3);
    let response = login(&returning, &base, false).await;
    assert!(
        !response
            .headers()
            .get_all("set-cookie")
            .iter()
            .any(|value| value.to_str().unwrap().starts_with("mw_browser_device="))
    );
    let second = browser(Arc::new(reqwest::cookie::Jar::default()), "Browser/2.0", 3);
    login(&second, &base, false).await;
    wait_notices(&captured, 2).await;
    let invalid = browser(Arc::new(reqwest::cookie::Jar::default()), "Browser/2.0", 3);
    let regenerated = login(&invalid, &base, true).await;
    assert!(
        regenerated
            .headers()
            .get_all("set-cookie")
            .iter()
            .any(|value| value.to_str().unwrap().starts_with("mw_browser_device="))
    );
    wait_notices(&captured, 3).await;
    login(&invalid, &base, false).await;
    let history = store
        .signin_history(mw_mock_jmap::ACCOUNT_ID, i64::MAX, Some("success"), 100)
        .await
        .unwrap();
    assert!(
        history
            .iter()
            .any(|event| event.user_agent == "Browser/1.0" && event.ip == "127.0.0.2")
    );
    assert!(
        history
            .iter()
            .any(|event| event.user_agent == "Browser/2.0" && event.ip == "127.0.0.3")
    );
    let events = store.list_audit(100).await.unwrap();
    assert_eq!(events.iter().filter(|event|serde_json::from_str::<Value>(&event.detail_json).unwrap_or_default()["alert"]=="newDevice").count(),3);
    assert!(
        !serde_json::to_string(
            &events
                .iter()
                .map(|event| &event.detail_json)
                .collect::<Vec<_>>()
        )
        .unwrap()
        .contains(token)
    );
    let sessions = returning
        .get(format!("{base}/api/account/sessions"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(!sessions.contains(token) && !sessions.contains("recognitionHash"));
    let notices = captured.lock().unwrap();
    assert_eq!(notices.len(), 3);
    for (import, raw) in notices.iter() {
        assert_eq!(
            import["methodCalls"][0][1]["accountId"],
            mw_mock_jmap::ACCOUNT_ID
        );
        assert_eq!(
            import["methodCalls"][0][1]["emails"]["alert"]["mailboxIds"]
                .as_object()
                .unwrap()
                .len(),
            1
        );
        assert!(raw.contains("new browser or device"));
        assert!(!raw.contains(token));
    }
}

#[tokio::test]
async fn recognition_cookie_is_secure_when_deployment_requires_tls() {
    let (base, _store, _captured) = fixture(true).await;
    let client = browser(Arc::new(reqwest::cookie::Jar::default()), "Browser/1.0", 2);
    let response = login(&client, &base, false).await;
    let marker = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find(|value| value.starts_with("mw_browser_device="))
        .unwrap();
    assert!(
        marker.contains("; Secure")
            && marker.contains("HttpOnly")
            && marker.contains("SameSite=Lax")
    );
}
