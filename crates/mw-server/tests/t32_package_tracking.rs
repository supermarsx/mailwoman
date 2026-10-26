use mw_server::{AppConfig,HardeningConfig,SecurityConfig,ServerMode,V6Config,build_app_full};
use serde_json::{Value,json};
mod common;
async fn serve(app:axum::Router)->String {
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
    tokio::spawn(async move {axum::serve(listener,app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap();});
    format!("http://{address}")
}
fn client()->reqwest::Client {reqwest::Client::builder().no_proxy().cookie_store(true).redirect(reqwest::redirect::Policy::none()).build().unwrap()}
async fn send(request:reqwest::RequestBuilder)->(u16,Value) {let response=request.send().await.unwrap();let status=response.status().as_u16();let text=response.text().await.unwrap();(status,serde_json::from_str(&text).unwrap_or(Value::String(text)))}
#[tokio::test]
async fn package_tracking_requires_opt_in_owner_revision_and_protected_provider() {
    let upstream=serve(mw_mock_jmap::router()).await;
    let root=common::test_db::unique_dir("packages-http");std::fs::create_dir_all(root.join("web")).unwrap();std::fs::write(root.join("web/index.html"),"<!doctype html><title>Mail</title>").unwrap();
    let (router,_)=build_app_full(AppConfig {db_path:root.join("db.sqlite").to_string_lossy().into(),server_key_hex:None,web_dir:Some(root.join("web")),cookie_secure:false,mode:ServerMode::Proxy,hardening:HardeningConfig::default(),security:SecurityConfig{upstream_default:Some(upstream.clone()),jmap_upstreams:Some(vec![upstream]),..Default::default()}},V6Config{admin_enabled:true,admin_username:Some("root".into()),admin_password:Some("hunter2".into()),redis_url:None}).await.unwrap();
    let base=serve(router).await;let admin=client();let user=client();
    assert_eq!(send(user.get(format!("{base}/api/account/packages"))).await.0,401);
    assert_eq!(send(admin.post(format!("{base}/admin/login")).json(&json!({"username":"root","password":"hunter2"}))).await.0,200);
    let initial=admin.get(format!("{base}/admin/package-tracking")).send().await.unwrap();let etag=initial.headers()["etag"].to_str().unwrap().to_string();assert_eq!(initial.json::<Value>().await.unwrap()["provider"],Value::Null);
    let config=json!({"enabled":true,"refreshSecs":900,"apiKey":"only-a-test-provider-key"});
    assert_eq!(send(admin.put(format!("{base}/admin/package-tracking")).header("if-match",&etag).json(&config)).await.0,403);
    assert_eq!(send(admin.post(format!("{base}/admin/step-up")).json(&json!({"password":"hunter2"}))).await.0,200);
    let (status,saved)=send(admin.put(format!("{base}/admin/package-tracking")).header("if-match",&etag).json(&config)).await;assert_eq!(status,200,"{saved}");assert!(!saved.to_string().contains("only-a-test-provider-key"));
    assert_eq!(send(admin.put(format!("{base}/admin/package-tracking")).header("if-match",&etag).json(&config)).await.0,412);
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let (status,login)=send(user.post(format!("{base}/api/login")).json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))).await;assert_eq!(status,200,"{login}");let account=login["accountId"].as_str().unwrap();
    let (_,view)=send(user.get(format!("{base}/api/account/packages"))).await;assert_eq!(view["state"]["enabled"],false);assert_eq!(view["available"],true);
    let draft=json!({"accountId":account,"revision":null,"label":"Private label","trackingNumber":"RA123456789PT","carrier":"portugal-ctt"});
    assert_eq!(send(user.post(format!("{base}/api/account/packages")).json(&draft)).await.0,403);
    assert_eq!(send(user.put(format!("{base}/api/account/packages")).json(&json!({"accountId":"other","revision":null,"enabled":true}))).await.0,409);
    let (status,enabled)=send(user.put(format!("{base}/api/account/packages")).json(&json!({"accountId":account,"revision":null,"enabled":true}))).await;assert_eq!(status,200);
    let mut draft=draft;draft["revision"]=enabled["revision"].clone();
    let (status,added)=send(user.post(format!("{base}/api/account/packages")).json(&draft)).await;assert_eq!(status,200,"{added}");
    assert_eq!(send(user.post(format!("{base}/api/account/packages")).json(&draft)).await.0,409);
    let (_,view)=send(user.get(format!("{base}/api/account/packages"))).await;assert_eq!(view["state"]["packages"].as_array().unwrap().len(),1);let id=view["state"]["packages"][0]["id"].as_str().unwrap();assert_eq!(view["state"]["packages"][0]["attemptedAt"],Value::Null,"saving never contacts provider");
    let (status,disabled)=send(user.put(format!("{base}/api/account/packages")).json(&json!({"accountId":account,"revision":view["revision"],"enabled":false}))).await;assert_eq!(status,200);
    assert_eq!(send(user.post(format!("{base}/api/account/packages/{id}/refresh")).json(&json!({"accountId":account,"revision":disabled["revision"]}))).await.0,403);
    assert_eq!(send(user.delete(format!("{base}/api/account/packages/{id}")).json(&json!({"accountId":account,"revision":disabled["revision"]}))).await.0,200,"removal remains available while opted out");
    assert_eq!(send(user.get(format!("{base}/admin/package-tracking"))).await.0,401);
}
