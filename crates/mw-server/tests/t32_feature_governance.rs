//! Real admin catalogue CAS and proxy/native policy isolation through the router.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{
    ServerKey, Store,
    settings_policy::{SettingScope, SettingValueRow},
};
use serde_json::{Value, json};
mod common;
const KEY: &str = "1111111111111111111111111111111111111111111111111111111111111111";
async fn serve(app: axum::Router) -> String {
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
    base
}
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}
async fn catalogue(client: &reqwest::Client, base: &str) -> (Value, String) {
    let response = client
        .get(format!("{base}/admin/features"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let etag = response.headers()["etag"].to_str().unwrap().to_string();
    (response.json().await.unwrap(), etag)
}
#[tokio::test]
async fn feature_catalogue_cas_and_verified_proxy_policy() {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("feature-governance");
    let store = Store::open(db.to_str().unwrap(), ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    let (app, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(KEY.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                jmap_upstreams: Some(vec![upstream.clone()]),
                upstream_default: Some(upstream),
                ..SecurityConfig::default()
            },
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
    let base = serve(app).await;
    let admin = client();
    assert_eq!(
        admin
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let profile = json!({"name":"Work","values":{"cap.pim.contacts":{"value":false,"locked":false}}});
    let (_, initial) = catalogue(&admin, &base).await;
    let path = format!("{base}/admin/features/profiles/work");
    assert_eq!(
        admin
            .put(&path)
            .json(&profile)
            .send()
            .await
            .unwrap()
            .status(),
        428
    );
    assert_eq!(
        admin
            .put(&path)
            .header("If-Match", "*")
            .json(&profile)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    let (a, b) = tokio::join!(
        admin
            .put(&path)
            .header("If-Match", &initial)
            .json(&profile)
            .send(),
        admin
            .put(format!("{base}/admin/features/profiles/other"))
            .header("If-Match", &initial)
            .json(&json!({"name":"Other","values":{}}))
            .send()
    );
    let (a,b)=(a.unwrap(),b.unwrap());
    let (a_status,b_status)=(a.status().as_u16(),b.status().as_u16());
    let details=format!("{} / {}",a.text().await.unwrap(),b.text().await.unwrap());
    let (a,b)=(a_status,b_status);
    assert!(
        matches!((a, b), (200, 412) | (412, 200)),
        "atomic catalogue writers: {a}/{b}: {details}"
    );
    let (_, tag) = catalogue(&admin, &base).await;
    assert_eq!(
        admin
            .put(&path)
            .header("If-Match", tag)
            .json(&profile)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let (_, tag) = catalogue(&admin, &base).await;
    assert_eq!(
        admin
            .put(format!("{base}/admin/features/assignments"))
            .header("If-Match", &tag)
            .json(&json!({"scopeKind":"user","scopeId":"invented","profileId":"work","priority":1}))
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    assert_eq!(
        client()
            .get(format!("{base}/admin/features"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    let user = client();
    let response = user
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let me: Value = response.json().await.unwrap();
    let raw = me["accountId"].as_str().unwrap();
    let authority = me["storageAuthority"].as_str().unwrap();
    // A native account row deliberately collides with the authenticated proxy ID.
    let pool = sqlx::sqlite::SqlitePoolOptions::new().connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&db))
        .await
        .unwrap();
    sqlx::query("INSERT INTO accounts(id,kind,host,port,tls,username,sealed_creds) VALUES(?1,'imap','native.test',993,'implicit',?2,?3)").bind(raw).bind(mw_mock_jmap::USER).bind(vec![0u8]).execute(&pool).await.unwrap();
    let (_, tag) = catalogue(&admin, &base).await;
    assert_eq!(
        admin
            .put(format!("{base}/admin/features/assignments"))
            .header("If-Match", tag)
            .json(&json!({"scopeKind":"user","scopeId":raw,"profileId":"work","priority":1}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    for (scope, id) in [
        (SettingScope::AdminUser, raw),
        (SettingScope::AdminUser, mw_mock_jmap::USER),
        (SettingScope::User, raw),
    ] {
        store
            .put_setting_value(&SettingValueRow {
                scope,
                scope_id: id.into(),
                key: "mail.showFrom".into(),
                value: "true".into(),
                locked: false,
                updated_by: "legacy".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
    }
    let settings: Value = user
        .get(format!("{base}/api/settings"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        settings["settings"]["mail.showFrom"]["value"], false,
        "proxy must not inherit native rows"
    );
    let features: Value = user
        .get(format!("{base}/api/features"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        features["features"]["contacts"]["enabled"], true,
        "native profile must not leak"
    );
    let setting = format!("{base}/api/settings/mail.showFrom");
    assert_eq!(
        user.put(&setting)
            .json(&json!({"value":true}))
            .send()
            .await
            .unwrap()
            .status(),
        428
    );
    assert_eq!(
        user.put(&setting)
            .header("X-Mailwoman-Storage-Authority", "msa1_wrong")
            .json(&json!({"value":true}))
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    assert_eq!(
        user.put(&setting)
            .header("X-Mailwoman-Storage-Authority", authority)
            .json(&json!({"value":true}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let settings: Value = user
        .get(format!("{base}/api/settings"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(settings["settings"]["mail.showFrom"]["value"], true);
    assert_eq!(
        user.delete(&setting)
            .header("X-Mailwoman-Storage-Authority", authority)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let settings: Value = user
        .get(format!("{base}/api/settings"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(settings["settings"]["mail.showFrom"]["value"], false);
    assert!(
        store
            .get_setting_value(SettingScope::User, raw, "mail.showFrom")
            .await
            .unwrap()
            .is_some(),
        "legacy rows remain inert, not deleted"
    );
    // Global hard disable remains authoritative for both display and admission.
    assert_eq!(admin.put(format!("{base}/admin/settings/cap.pim.contacts")).json(&json!({"globalValue":false,"locked":true,"allowUserOverride":false})).send().await.unwrap().status(),200);
    let features:Value=user.get(format!("{base}/api/features")).send().await.unwrap().json().await.unwrap();
    assert_eq!(features["features"]["contacts"]["enabled"],false);
    let denied=user.post(format!("{base}/jmap/api")).json(&json!({"using":["urn:ietf:params:jmap:core"],"methodCalls":[["ContactCard/get",{"accountId":raw,"ids":[]},"c"]]})).send().await.unwrap();
    assert_eq!(denied.status(),403,"feature policy must refuse before forwarding");
    // A domain-scoped administrator cannot read or mutate this global catalogue.
    let mut scoped = store
        .list_admin_accounts()
        .await
        .unwrap()
        .into_iter()
        .find(|a| a.username == "root")
        .unwrap();
    scoped.id = "scoped-feature-admin".into();
    scoped.username = "scoped".into();
    scoped.source = "db".into();
    scoped.password_hash = Some(
        mw_admin::password::hash_password("Scoped-Pass123!", &mw_admin::SecurityPolicy::default())
            .unwrap(),
    );
    store
        .insert_admin_account_with_roles(
            &scoped,
            &[mw_store::AdminRoleGrant {
                role_id: "super_admin".into(),
                domains: Some(vec!["example.test".into()]),
            }],
            false,
        )
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    let restricted = client();
    assert_eq!(
        restricted
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"scoped","password":"Scoped-Pass123!"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        restricted
            .get(format!("{base}/admin/features"))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    let (_, tag) = catalogue(&admin, &base).await;
    assert_eq!(
        restricted
            .put(&path)
            .header("If-Match", tag)
            .json(&profile)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
}

#[tokio::test]
async fn native_policy_compatibility_and_user_override_are_preserved() {
    let mail=common::mock_mail::MockPop3::start("native@example.test","native-pass").await;
    let db=common::test_db::unique_db_path("feature-native-policy");
    let store=Store::open(db.to_str().unwrap(),ServerKey::from_hex(KEY).unwrap()).await.unwrap();
    store.put_domain_config(&mw_store::DomainConfigRow{name:"example.test".into(),upstream_json:json!({"pop3":{"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","allowPrivate":true}}).to_string(),allowlist_json:"[]".into(),blocklist_json:"[]".into(),enabled:true,preferred:false,settings_json:"{}".into(),updated_at:1}).await.unwrap();
    let app=mw_server::build_app(AppConfig{db_path:db.to_string_lossy().into(),server_key_hex:Some(KEY.into()),web_dir:None,cookie_secure:false,mode:ServerMode::Engine,hardening:HardeningConfig::default(),security:SecurityConfig{upstream_default:Some(mail.url()),..Default::default()}}).await.unwrap();
    let base=serve(app).await;
    let user=client();
    let response=user.post(format!("{base}/api/login")).json(&json!({"username":"native@example.test","password":"native-pass"})).send().await.unwrap();
    assert_eq!(response.status(),200);
    let me:Value=response.json().await.unwrap();
    let raw=me["accountId"].as_str().unwrap();
    let authority=me["storageAuthority"].as_str().unwrap();
    let setting=SettingValueRow{scope:SettingScope::AdminUser,scope_id:raw.into(),key:"mail.showFrom".into(),value:"true".into(),locked:false,updated_by:"test".into(),updated_at:1};
    store.put_setting_value(&setting).await.unwrap();
    let settings:Value=user.get(format!("{base}/api/settings")).send().await.unwrap().json().await.unwrap();
    assert_eq!(settings["settings"]["mail.showFrom"]["value"],true);
    assert_eq!(settings["settings"]["mail.showFrom"]["source"],"admin_user");
    assert_eq!(user.put(format!("{base}/api/settings/mail.showFrom")).header("X-Mailwoman-Storage-Authority",authority).json(&json!({"value":false})).send().await.unwrap().status(),200);
    assert_eq!(store.get_setting_value(SettingScope::User,raw,"mail.showFrom").await.unwrap().unwrap().value,"false");
    let owner=store.native_personal_subject(raw).await.unwrap().unwrap();
    assert!(store.personal_setting_values(&owner).await.unwrap().is_empty(),"native existing namespace remains compatible");
    store.put_setting_value(&SettingValueRow{scope:SettingScope::Domain,scope_id:"example.test".into(),key:"mail.showFrom".into(),value:"true".into(),locked:true,updated_by:"test".into(),updated_at:2}).await.unwrap();
    assert_eq!(user.put(format!("{base}/api/settings/mail.showFrom")).header("X-Mailwoman-Storage-Authority",authority).json(&json!({"value":false})).send().await.unwrap().status(),403);
}
