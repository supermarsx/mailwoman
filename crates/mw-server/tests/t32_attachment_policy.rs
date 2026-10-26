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
async fn restrictive_proxy_policy_uses_owned_metadata_not_url_names_and_is_hot() {
    let upstream=serve(mw_mock_jmap::router()).await;
    let db=common::test_db::unique_db_path("attachment-policy");
    let (router,_)=build_app_full(AppConfig {db_path:db.to_string_lossy().into(),server_key_hex:None,web_dir:None,cookie_secure:false,mode:ServerMode::Proxy,hardening:HardeningConfig::default(),security:SecurityConfig {upstream_default:Some(upstream.clone()),jmap_upstreams:Some(vec![upstream]),..SecurityConfig::default()}},V6Config {admin_enabled:false,admin_username:None,admin_password:None,redis_url:None}).await.unwrap();
    let base=serve(router).await;
    let pool=sqlx::SqlitePool::connect(&format!("sqlite:{}",db.display())).await.unwrap();
    let client=reqwest::Client::builder().cookie_store(true).build().unwrap();
    assert_eq!(client.get(format!("{base}/api/attachments/policy")).send().await.unwrap().status(),401);
    let login=client.post(format!("{base}/api/login")).json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS})).send().await.unwrap();
    assert_eq!(login.status(),200);
    let account=login.json::<Value>().await.unwrap()["accountId"].as_str().unwrap().to_owned();
    let created=client.post(format!("{base}/jmap/api")).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/set",{"accountId":account,"create":{"x":{"subject":"policy","attachments":[{"blobId":"danger","cid":"danger","name":"malware.exe","type":"application/octet-stream","size":10},{"blobId":"safe","name":"note.txt","type":"application/octet-stream","size":10}]} }},"c"]]})).send().await.unwrap();
    assert_eq!(created.status(),200);
    let body=created.json::<Value>().await.unwrap();
    let email=body["methodResponses"][0][1]["created"]["x"]["id"].as_str().unwrap();
    let unsafe_url=format!("{base}/jmap/download/{account}/danger/innocent.pdf?emailId={email}");
    assert_eq!(client.get(&unsafe_url).send().await.unwrap().status(),200);
    let signed:Value=client.post(format!("{base}/api/rc/sign")).json(&json!({"emailId":email,"parts":["danger"]})).send().await.unwrap().json().await.unwrap();
    let cid=signed["parts"][0]["src"].as_str().expect("owned CID signature").to_owned();
    set(&pool,"attachments.blockedExtensions",json!(["exe"])).await;
    let blocked=client.get(&unsafe_url).send().await.unwrap();
    assert_eq!(blocked.status(),403);assert_eq!(blocked.json::<Value>().await.unwrap()["code"],"attachmentBlocked");
    let cid_response=client.get(format!("{base}{cid}")).send().await.unwrap();assert_eq!(cid_response.status(),403);assert_eq!(cid_response.json::<Value>().await.unwrap()["code"],"attachmentBlocked");
    assert_eq!(client.get(format!("{base}/jmap/download/{account}/blob-{email}/innocent.txt?emailId={email}")).send().await.unwrap().json::<Value>().await.unwrap()["code"],"attachmentContainerRestricted");
    assert_eq!(client.get(format!("{base}/jmap/download/{account}/safe/note.txt")).send().await.unwrap().status(),403);
    assert_eq!(client.get(format!("{base}/jmap/download/{account}/safe/note.txt?emailId=foreign")).send().await.unwrap().status(),403);
    let safe=format!("{base}/jmap/download/{account}/safe/note.txt?emailId={email}");
    assert_eq!(client.get(&safe).send().await.unwrap().status(),200);
    set(&pool,"attachments.warnExtensions",json!(["txt"])).await;
    assert_eq!(client.get(&safe).send().await.unwrap().json::<Value>().await.unwrap()["code"],"attachmentWarningRequired");
    assert_eq!(client.get(format!("{safe}&acknowledgeWarning=true")).send().await.unwrap().status(),200);
    set(&pool,"attachments.previewAllowed",json!(false)).await;
    assert_eq!(client.get(format!("{safe}&acknowledgeWarning=true&preview=true")).send().await.unwrap().status(),403);
    let download=client.get(format!("{safe}&acknowledgeWarning=true")).send().await.unwrap();assert_eq!(download.status(),200);assert_eq!(download.headers()["x-mailwoman-preview-allowed"],"false");
    set(&pool,"attachments.downloadAllowed",json!(false)).await;
    assert_eq!(client.get(format!("{safe}&acknowledgeWarning=true")).send().await.unwrap().status(),403);
}
async fn set(pool:&sqlx::SqlitePool,key:&str,value:Value) {
    sqlx::query("INSERT INTO setting_policy(key,global_value,updated_by,updated_at) VALUES(?1,?2,'test',0) ON CONFLICT(key) DO UPDATE SET global_value=excluded.global_value").bind(key).bind(value.to_string()).execute(pool).await.unwrap();
}
