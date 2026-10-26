use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{Credentials, ServerKey, Store, identity_principal::VerifiedPrimary};
use serde_json::{Value,json};
use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
mod common;
const KEY:&str="00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

#[tokio::test]
async fn one_shot_sender_resolves_only_current_upstream_authority_without_saving_identity() {
    let enabled=Arc::new(AtomicBool::new(true));
    let live=enabled.clone();
    let upstream=axum::Router::new().route("/api",axum::routing::post(move |axum::Json(request):axum::Json<Value>| {
        let live=live.clone(); async move {
            assert_eq!(request["methodCalls"][0][0],"Identity/get");
            assert_eq!(request["methodCalls"][0][1]["accountId"],"1");
            let list=if live.load(Ordering::SeqCst) {json!([{"id":"provider-alias","email":"alias@example.test"}])}else{json!([])};
            axum::Json(json!({"methodResponses":[["Identity/get",{"accountId":"1","list":list},"identities"]]}))
        }
    }));
    let socket=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin=format!("http://{}",socket.local_addr().unwrap());
    tokio::spawn(async move {axum::serve(socket,upstream).await.unwrap();});
    let db=common::test_db::unique_db_path("sender-resolve");
    let store=Store::open(db.to_str().unwrap(),ServerKey::from_hex(KEY).unwrap()).await.unwrap();
    let discovery=format!("{origin}/session"); let api=format!("{origin}/api");
    store.put_domain_config(&mw_store::DomainConfigRow{name:"example.test".into(),upstream_json:json!({"jmap":{"sessionUrl":discovery}}).to_string(),allowlist_json:"[]".into(),blocklist_json:"[]".into(),enabled:true,preferred:false,settings_json:"{}".into(),updated_at:0}).await.unwrap();
    let sid=store.create_session("1","alice@example.test",&discovery,&api,&Credentials{username:"alice@example.test".into(),password:"fixture".into()}).await.unwrap();
    store.set_session_authentication(&sid,"header-auth").await.unwrap();
    store.bind_verified_primary(&sid,VerifiedPrimary{account_id:"1",kind:"header",authority:"deployment:trusted-header",subject:"alice"},100).await.unwrap().unwrap();
    let authority=store.session_storage_authority(&sid).await.unwrap().unwrap();
    let owner=store.session_personal_subject(&sid).await.unwrap().unwrap();
    let (app,_)=build_app_full(AppConfig{db_path:db.to_string_lossy().into(),server_key_hex:Some(KEY.into()),web_dir:None,cookie_secure:false,mode:ServerMode::Proxy,hardening:HardeningConfig::default(),security:SecurityConfig{jmap_upstreams:Some(vec![origin]),..SecurityConfig::default()}},V6Config::default()).await.unwrap();
    let socket=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url=format!("http://{}/api/account/sender/resolve",socket.local_addr().unwrap());
    tokio::spawn(async move {axum::serve(socket,app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap();});
    let client=reqwest::Client::builder().no_proxy().build().unwrap();
    let request=||client.post(&url).header("cookie",format!("mw_session={sid}"));
    assert_eq!(request().json(&json!({"email":"alias@example.test"})).send().await.unwrap().status(),428);
    assert_eq!(request().header("x-mailwoman-storage-authority","msa1_wrong").json(&json!({"email":"alias@example.test"})).send().await.unwrap().status(),409);
    for email in ["stranger@example.test","alice@other.test"] {
        let response=request().header("x-mailwoman-storage-authority",&authority).json(&json!({"email":email})).send().await.unwrap();
        assert_eq!(response.status(),403); assert_eq!(response.json::<Value>().await.unwrap()["code"],"forbiddenFrom");
    }
    for email in ["a@example.test\r\nBcc:other@example.test","A <alias@example.test>"] {
        assert_eq!(request().header("x-mailwoman-storage-authority",&authority).json(&json!({"email":email})).send().await.unwrap().status(),400);
    }
    let response=request().header("x-mailwoman-storage-authority",&authority).json(&json!({"email":"alias@example.test"})).send().await.unwrap();
    assert_eq!(response.status(),200,"{}",response.text().await.unwrap());
    let response=request().header("x-mailwoman-storage-authority",&authority).json(&json!({"email":"alias@example.test"})).send().await.unwrap();
    assert_eq!(response.headers()["cache-control"],"private, no-store");
    assert_eq!(response.json::<Value>().await.unwrap(),json!({"email":"alias@example.test","identityId":"provider-alias"}));
    assert!(store.list_personal_identities(&owner).await.unwrap().is_empty());
    enabled.store(false,Ordering::SeqCst);
    assert_eq!(request().header("x-mailwoman-storage-authority",&authority).json(&json!({"email":"alias@example.test"})).send().await.unwrap().status(),403);
}
