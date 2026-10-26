use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{Credentials, ServerKey, Store, identity_principal::VerifiedPrimary};
use serde_json::{Value,json};
mod common;
const KEY:&str="00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
#[tokio::test]
async fn personal_documents_are_realm_bound_and_legacy_values_remain_inert() {
    let db=common::test_db::unique_db_path("personal-storage-realms");
    let store=Store::open(db.to_str().unwrap(),ServerKey::from_hex(KEY).unwrap()).await.unwrap();
    let legacy=json!({"revision":"legacy","document":{"version":1,"preferences":{"folderPinned":["private-old-folder"],"analyticsConsent":true}}}).to_string();
    store.set_setting("ui-preferences:1",&legacy).await.unwrap();
    let old_appearance=json!({"v":1,"updated_at":1,"prefs":{"theme":"private-old-theme"}}).to_string();
    let old_customization=json!({"version":1,"css":"body{color:red}","script":"private-old-code"}).to_string();
    store.set_setting("appearance:1",&old_appearance).await.unwrap();
    store.set_setting("appearance-customization:mailbox:1",&old_customization).await.unwrap();
    let (router,_)=build_app_full(AppConfig {db_path:db.to_string_lossy().into(),server_key_hex:Some(KEY.into()),web_dir:None,cookie_secure:false,mode:ServerMode::Proxy,hardening:HardeningConfig::default(),security:SecurityConfig::default()},V6Config::default()).await.unwrap();
    let socket=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base=format!("http://{}",socket.local_addr().unwrap());
    tokio::spawn(async move {axum::serve(socket,router.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap();});
    let mut clients=Vec::new(); let mut authorities=Vec::new(); let mut sessions=Vec::new();
    for path in ["first","second"] {
        let service=format!("https://mail.example.test/{path}/session");
        let username=format!("alice@{path}.example.test");
        store.put_domain_config(&mw_store::DomainConfigRow{name:format!("{path}.example.test"),upstream_json:json!({"jmap":{"sessionUrl":service}}).to_string(),allowlist_json:"[]".into(),blocklist_json:"[]".into(),enabled:true,preferred:false,settings_json:"{}".into(),updated_at:0}).await.unwrap();
        let id=store.create_session("1",&username,&service,&format!("https://mail.example.test/{path}/api"),&Credentials{username:username.clone(),password:"synthetic".into()}).await.unwrap();
        store.set_session_authentication(&id,"header-auth").await.unwrap();
        store.bind_verified_primary(&id,VerifiedPrimary{account_id:"1",kind:"header",authority:"deployment:trusted-header",subject:"alice"},100).await.unwrap().unwrap();
        let authority=store.session_storage_authority(&id).await.unwrap().unwrap();
        let mut headers=reqwest::header::HeaderMap::new();
        headers.insert("cookie",format!("mw_session={id}").parse().unwrap());
        headers.insert("x-mailwoman-storage-authority",authority.parse().unwrap());
        clients.push(reqwest::Client::builder().no_proxy().default_headers(headers).build().unwrap()); authorities.push(authority); sessions.push(id);
    }
    let legacy_signature=mw_store::SignatureRow{account_id:"1".into(),name:"legacy".into(),body:"private legacy signature".into(),html:None,text:None,text_edited:false,is_default:false,rule_json:String::new(),updated_at:String::new()};
    store.upsert_signature(&legacy_signature).await.unwrap();
    store.upsert_saved_search(&mw_store::SavedSearchRow{id:"legacy-search".into(),user:"1".into(),name:"Private legacy".into(),query_json:"{}".into(),as_folder:true}).await.unwrap();
    for client in &clients {
        for (route,key) in [("signatures","signatures"),("saved-searches","savedSearches")] {
            let response=client.get(format!("{base}/api/account/{route}")).send().await.unwrap();assert_eq!(response.status(),200);
            let body:Value=response.json().await.unwrap();assert_eq!(body[key],json!([]));
        }
    }
    let signature_url=format!("{base}/api/account/signatures");
    let signature=json!({"name":"work","body":"plain","html":"<b>rich</b>","text":"plain","textEdited":true,"isDefault":true});
    assert_eq!(clients[0].post(&signature_url).json(&signature).send().await.unwrap().status(),200);
    let other:Value=clients[1].get(&signature_url).send().await.unwrap().json().await.unwrap();assert_eq!(other["signatures"],json!([]));
    assert_eq!(clients[1].post(&signature_url).header("x-mailwoman-storage-authority",&authorities[0]).json(&signature).send().await.unwrap().status(),409);
    let search_url=format!("{base}/api/account/saved-searches");
    let response=clients[0].put(&search_url).json(&json!({"name":"Private","queryJson":"{}","asFolder":true})).send().await.unwrap();
    assert_eq!(response.status(),200);let created:Value=response.json().await.unwrap();let search_id=created["id"].as_str().unwrap();
    assert_eq!(clients[1].delete(format!("{search_url}/{search_id}")).send().await.unwrap().status(),404);
    assert_eq!(clients[1].put(&search_url).json(&json!({"id":search_id,"name":"Hijacked","queryJson":"{}","asFolder":true})).send().await.unwrap().status(),404);
    let own:Value=clients[0].get(&search_url).send().await.unwrap().json().await.unwrap();assert_eq!(own["savedSearches"][0]["name"],"Private");
    assert_eq!(store.list_signatures("1").await.unwrap().len(),1);
    assert!(store.get_saved_search("legacy-search").await.unwrap().is_some());
    let owner_a=store.session_personal_subject(&sessions[0]).await.unwrap().unwrap();
    let owner_b=store.session_personal_subject(&sessions[1]).await.unwrap().unwrap();
    for (owner,name) in [(&owner_a,"Realm A"),(&owner_b,"Realm B")] {
        store.upsert_personal_identity(&mw_store::IdentityRow{id:"same-profile-id".into(),account_id:owner.clone(),name:name.into(),email:"allowed@example.test".into(),reply_to:None,signature_html:None,signature_text:None,signature_name:None,sent_mailbox_id:None,source:"configured".into()}).await.unwrap();
    }
    let identities_url=format!("{base}/api/account/identities");
    for (client,name) in [(&clients[0],"Realm A"),(&clients[1],"Realm B")] {
        let identities:Value=client.get(&identities_url).send().await.unwrap().json().await.unwrap();assert_eq!(identities["identities"][0]["name"],name);
    }
    assert_eq!(clients[1].delete(format!("{identities_url}/same-profile-id")).send().await.unwrap().status(),200);
    assert!(store.get_personal_identity(&owner_a,"same-profile-id").await.unwrap().is_some());
    let missing_identity=reqwest::Client::new().post(&identities_url).header("cookie",format!("mw_session={}",sessions[0])).json(&json!({"id":"","name":"No proof","email":"allowed@example.test"})).send().await.unwrap();
    assert_eq!(missing_identity.status(),428);
    let notification_url=format!("{base}/api/account/notifications");
    let config=json!({"enabled":true,"rules":[],"quietHours":{"enabled":false,"start":"22:00","end":"07:00"}});
    store.put_notification_rules(&mw_store::NotificationRulesRow{account_id:"1".into(),enabled:true,rule_json:"[]".into(),quiet_hours_json:config["quietHours"].to_string(),updated_at:String::new()}).await.unwrap();
    for client in &clients {let body:Value=client.get(&notification_url).send().await.unwrap().json().await.unwrap();assert_eq!(body["enabled"],false);}
    for client in &clients {
        let settings:Value=client.get(format!("{base}/api/settings")).send().await.unwrap().json().await.unwrap();
        assert_ne!(settings["settings"]["notifications.enabled"]["source"], "user", "legacy notification rows must not reappear through effective settings");
    }
    assert_eq!(clients[0].put(&notification_url).json(&config).send().await.unwrap().status(),200);
    let settings:Value=clients[0].get(format!("{base}/api/settings")).send().await.unwrap().json().await.unwrap();
    assert_eq!(settings["settings"]["notifications.enabled"]["value"],true);
    assert_eq!(settings["settings"]["notifications.enabled"]["source"],"user");
    let other:Value=clients[1].get(&notification_url).send().await.unwrap().json().await.unwrap();assert_eq!(other["enabled"],false);
    let subscription=json!({"transport":"unifiedpush","endpoint":"https://push.example.test/owned-realms"});
    let push_url=format!("{base}/api/push/subscribe");
    assert_eq!(clients[1].post(&push_url).header("x-mailwoman-storage-authority",&authorities[0]).json(&subscription).send().await.unwrap().status(),409);
    let registered=clients[0].post(&push_url).json(&subscription).send().await.unwrap();assert_eq!(registered.status(),200,"{}",registered.text().await.unwrap());
    assert_eq!(clients[1].put(&notification_url).json(&config).send().await.unwrap().status(),200);
    assert_eq!(clients[1].post(&push_url).json(&subscription).send().await.unwrap().status(),409);
    assert_eq!(clients[1].post(format!("{base}/api/push/unsubscribe")).json(&json!({"endpoint":subscription["endpoint"]})).send().await.unwrap().status(),204);
    assert_eq!(store.list_personal_push_subscriptions(&owner_a).await.unwrap().len(),1);
    assert!(store.list_personal_push_subscriptions(&owner_b).await.unwrap().is_empty());
    let endpoint=format!("{base}/api/account/preferences");
    for client in &clients {
        let response=client.get(&endpoint).send().await.unwrap(); assert_eq!(response.status(),200);
        let body:Value=response.json().await.unwrap(); assert_eq!(body["preferences"],json!({}),"ambiguous legacy data must not be attributed to either realm");
    }
    let missing=reqwest::Client::new().put(&endpoint).header("cookie",format!("mw_session={}",sessions[0])).header("if-match","\"0\"").json(&json!({"version":1,"preferences":{}})).send().await.unwrap();
    assert_eq!(missing.status(),428);
    let saved=clients[0].put(&endpoint).header("if-match","\"0\"").json(&json!({"version":1,"preferences":{"folderPinned":["realm-one"],"analyticsConsent":true}})).send().await.unwrap();
    assert_eq!(saved.status(),200,"{}",saved.text().await.unwrap());
    let other:Value=clients[1].get(&endpoint).send().await.unwrap().json().await.unwrap(); assert_eq!(other["preferences"],json!({}));
    let denied=clients[1].put(&endpoint).header("x-mailwoman-storage-authority",&authorities[0]).header("if-match","\"0\"").json(&json!({"version":1,"preferences":{}})).send().await.unwrap(); assert_eq!(denied.status(),409);
    for (index, client) in clients.iter().enumerate() {
        let status:Value=client.get(format!("{base}/api/analytics/status")).send().await.unwrap().json().await.unwrap();
        assert_eq!(status["consent"],index==0);
        let appearance:Value=client.get(format!("{base}/api/account/appearance")).send().await.unwrap().json().await.unwrap();
        assert_eq!(appearance["appearance"],Value::Null);
        let custom:Value=client.get(format!("{base}/api/account/appearance/customization?accountId=1")).send().await.unwrap().json().await.unwrap();
        assert_eq!(custom["document"]["css"],""); assert_eq!(custom["document"]["script"],"");
        assert_eq!(custom["storageAuthority"],authorities[index]);
    }
    let appearance_url=format!("{base}/api/account/appearance");
    assert_eq!(clients[0].put(&appearance_url).json(&json!({"appearance":{"mode":"fixed","theme":"light"}})).send().await.unwrap().status(),200);
    let own:Value=clients[0].get(&appearance_url).send().await.unwrap().json().await.unwrap();assert!(own["appearance"].is_object());
    let other:Value=clients[1].get(&appearance_url).send().await.unwrap().json().await.unwrap();assert_eq!(other["appearance"],Value::Null);
    let custom_url=format!("{base}/api/account/appearance/customization?accountId=1");
    let current=clients[0].get(&custom_url).send().await.unwrap();let tag=current.headers()["etag"].clone();
    assert_eq!(clients[0].put(&custom_url).header("if-match",tag).json(&json!({"ownerId":"1","document":{"version":1,"css":"body{color:red}","script":""}})).send().await.unwrap().status(),200);
    let other:Value=clients[1].get(&custom_url).send().await.unwrap().json().await.unwrap();assert_eq!(other["document"]["css"],"");
    assert_eq!(store.get_setting("appearance:1").await.unwrap(),Some(old_appearance));
    assert_eq!(store.get_setting("appearance-customization:mailbox:1").await.unwrap(),Some(old_customization));
    assert_eq!(store.get_setting("ui-preferences:1").await.unwrap(),Some(legacy));
}
