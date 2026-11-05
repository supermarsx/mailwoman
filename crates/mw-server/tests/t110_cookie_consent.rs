//! Cookie consent over the real router: the public configuration resolved for a
//! request, anonymous consent recording, the storage scan report with its review
//! list, administrator classification, statistics and the CSV export.
//!
//! SQLite only. Settings are written to the store before the app boots, as the
//! settings API would store them.

use std::net::SocketAddr;

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::settings_policy::SettingPolicyRow;
use serde_json::{Value, json};
use tokio::net::TcpListener;

mod common;
use common::test_db;

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

async fn spawn(settings: &[(&str, Value)]) -> String {
    let dir = test_db::unique_dir("mw-t110-cookies");
    let web = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let store = mw_store::Store::open(&config.db_path, mw_store::ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    for (key, value) in settings {
        store
            .put_setting_policy(&SettingPolicyRow {
                key: (*key).into(),
                global_value: Some(value.to_string()),
                allow_user_override: false,
                global_locked: false,
                updated_by: "test-fixture".into(),
                updated_at: 0,
            })
            .await
            .unwrap();
    }
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    };
    let app = build_app_full(config, v6).await.expect("server boots").0;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await;
    });
    format!("http://{addr}")
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().redirect(reqwest::redirect::Policy::none()).build().unwrap()
}

/// The admin session cookie header.
async fn admin(c: &reqwest::Client, base: &str) -> String {
    let resp = c
        .post(format!("{base}/admin/login"))
        .json(&json!({"username": "root", "password": "hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let cookies: Vec<String> = resp
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .map(|v| v.to_str().unwrap().split(';').next().unwrap().to_owned())
        .collect();
    cookies.join("; ")
}

fn on() -> Vec<(&'static str, Value)> {
    vec![
        ("cookies.enabled", json!(true)),
        ("cookies.analytics.enabled", json!(true)),
        ("cookies.policyVersion", json!(3)),
        ("cookies.policyUrl", json!("https://example.org/cookies")),
    ]
}

#[tokio::test]
async fn config_is_minimal_while_off() {
    let base = spawn(&[]).await;
    let body: Value = client().get(format!("{base}/api/cookies/config")).send().await.unwrap().json().await.unwrap();
    assert_eq!(body, json!({"enabled": false, "policyPages": true}));
}

#[tokio::test]
async fn config_resolves_region_and_model_for_the_request() {
    let mut settings = on();
    settings.push(("cookies.mode", json!("optOut")));
    let base = spawn(&settings).await;
    let c = client();
    let get = |country: Option<&'static str>| {
        let mut r = c.get(format!("{base}/api/cookies/config"));
        if let Some(country) = country {
            r = r.header("X-Country-Code", country);
        }
        async move { r.send().await.unwrap().json::<Value>().await.unwrap() }
    };
    let pt = get(Some("PT")).await;
    assert_eq!(pt["enabled"], true);
    assert_eq!(pt["strictRegion"], true);
    assert_eq!(pt["mode"], "optIn", "the strict-region model applies in Portugal");
    assert_eq!(pt["version"], 3);
    assert_eq!(pt["policyUrl"], "https://example.org/cookies");
    assert_eq!(pt["categories"][2]["id"], "analytics");
    assert_eq!(pt["categories"][2]["enabled"], true);
    assert!(pt["inventory"].as_array().unwrap().iter().any(|e| e["pattern"] == "mw_session"));
    let us = get(Some("US")).await;
    assert_eq!(us["strictRegion"], false);
    assert_eq!(us["mode"], "optOut");
    let unknown = get(None).await;
    assert_eq!(unknown["strictRegion"], true, "an unknown country is strict by default");
}

#[tokio::test]
async fn consent_is_recorded_counted_and_exported_without_personal_data() {
    let base = spawn(&on()).await;
    let c = client();
    let post = |body: Value| c.post(format!("{base}/api/cookies/consent")).header("X-Country-Code", "PT").json(&body).send();
    assert_eq!(post(json!({"version": 3, "decision": "acceptAll"})).await.unwrap().status(), 204);
    assert_eq!(post(json!({"version": 3, "decision": "rejectAll", "signal": "gpc"})).await.unwrap().status(), 204);
    assert_eq!(
        post(json!({"version": 3, "decision": "custom", "categories": {"functional": true, "marketing": true}})).await.unwrap().status(),
        204
    );
    // Malformed requests are refused, not stored.
    assert_eq!(post(json!({"version": 3, "decision": "maybe"})).await.unwrap().status(), 400);
    assert_eq!(post(json!({"version": 0, "decision": "acceptAll"})).await.unwrap().status(), 400);
    assert_eq!(post(json!({"version": 3, "decision": "custom", "categories": {"necessary": false}})).await.unwrap().status(), 400);
    assert_eq!(post(json!({"version": 3, "decision": "acceptAll", "extra": 1})).await.unwrap().status(), 400);

    let cookie = admin(&c, &base).await;
    let stats: Value = c.get(format!("{base}/admin/cookies/stats")).header("Cookie", &cookie).send().await.unwrap().json().await.unwrap();
    assert_eq!(stats["total"], 3);
    assert_eq!(stats["decisions"]["acceptAll"], 1);
    assert_eq!(stats["decisions"]["rejectAll"], 1);
    assert_eq!(stats["decisions"]["custom"], 1);
    assert_eq!(stats["categories"]["analytics"]["accepted"], 1);
    assert_eq!(stats["categories"]["marketing"]["accepted"], 0, "marketing is not offered, so it is stored off");
    assert_eq!(stats["categories"]["functional"]["accepted"], 2);
    assert_eq!(stats["signals"]["gpc"], 1);
    assert_eq!(stats["regions"][0]["region"], "PT");

    let resp = c.get(format!("{base}/admin/cookies/consents.csv")).header("Cookie", &cookie).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.headers()["content-type"].to_str().unwrap().starts_with("text/csv"));
    assert!(resp.headers()["content-disposition"].to_str().unwrap().contains("cookie-consents.csv"));
    let csv = resp.text().await.unwrap();
    assert_eq!(csv.lines().count(), 4, "{csv}");
    assert!(csv.lines().next().unwrap().ends_with("region,signal,accountId"));
    assert!(csv.contains("\"acceptAll\",true,true,true,false,\"PT\",\"none\",\"\""), "{csv}");
}

#[tokio::test]
async fn nothing_is_recorded_while_the_log_is_off() {
    let mut settings = on();
    settings.push(("cookies.logConsent", json!(false)));
    let base = spawn(&settings).await;
    let c = client();
    let r = c.post(format!("{base}/api/cookies/consent")).json(&json!({"version": 3, "decision": "acceptAll"})).send().await.unwrap();
    assert_eq!(r.status(), 204);
    let cookie = admin(&c, &base).await;
    let stats: Value = c.get(format!("{base}/admin/cookies/stats")).header("Cookie", &cookie).send().await.unwrap().json().await.unwrap();
    assert_eq!(stats["total"], 0);
}

#[tokio::test]
async fn unclassified_storage_is_reviewed_and_classifying_adds_a_rule() {
    let base = spawn(&on()).await;
    let c = client();
    let report = |items: Value| c.post(format!("{base}/api/cookies/detected")).json(&json!({"items": items})).send();
    let r = report(json!([
        {"kind": "localStorage", "name": "vendor.key"},
        {"kind": "localStorage", "name": "mw.theme.prefs"},
        {"kind": "cookie", "name": "mw_session"},
        {"kind": "localStorage", "name": "vendor.key"},
        {"kind": "bogus", "name": "x"},
        {"kind": "cookie", "name": "bad\nname"},
    ]))
    .await
    .unwrap();
    assert_eq!(r.status(), 204);
    // A value is not part of the contract: a body that carries one is refused.
    assert_eq!(report(json!([{"kind": "cookie", "name": "a", "value": "secret"}])).await.unwrap().status(), 400);

    let cookie = admin(&c, &base).await;
    let list: Value = c.get(format!("{base}/admin/cookies/detected")).header("Cookie", &cookie).send().await.unwrap().json().await.unwrap();
    let items = list["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "known names and invalid reports are dropped: {list}");
    assert_eq!(items[0]["name"], "vendor.key");
    assert_eq!(items[0]["seenCount"], 2);
    let id = items[0]["id"].as_str().unwrap();

    let r = c
        .post(format!("{base}/admin/cookies/detected/{id}/classify"))
        .header("Cookie", &cookie)
        .json(&json!({"category": "analytics"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204);
    let list: Value = c.get(format!("{base}/admin/cookies/detected")).header("Cookie", &cookie).send().await.unwrap().json().await.unwrap();
    assert!(list["items"].as_array().unwrap().is_empty());

    // The rule now classifies the name, in the admin inventory and the public config.
    let inv: Value = c.get(format!("{base}/admin/cookies/inventory")).header("Cookie", &cookie).send().await.unwrap().json().await.unwrap();
    assert_eq!(inv["rules"][0]["pattern"], "vendor.key");
    assert_eq!(inv["rules"][0]["category"], "analytics");
    let config: Value = c.get(format!("{base}/api/cookies/config")).send().await.unwrap().json().await.unwrap();
    assert_eq!(config["rules"][0]["pattern"], "vendor.key");
    // Reporting the same name again records nothing new.
    assert_eq!(report(json!([{"kind": "localStorage", "name": "vendor.key"}])).await.unwrap().status(), 204);
    let list: Value = c.get(format!("{base}/admin/cookies/detected")).header("Cookie", &cookie).send().await.unwrap().json().await.unwrap();
    assert!(list["items"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn rules_are_validated_and_removable_and_routes_need_an_admin() {
    let base = spawn(&on()).await;
    let c = client();
    for path in ["/admin/cookies/inventory", "/admin/cookies/detected", "/admin/cookies/stats", "/admin/cookies/consents.csv"] {
        assert_eq!(c.get(format!("{base}{path}")).send().await.unwrap().status(), 401, "{path}");
    }
    assert_eq!(c.post(format!("{base}/admin/cookies/rules")).json(&json!({"kind": "cookie", "pattern": "x", "category": "analytics"})).send().await.unwrap().status(), 401);

    let cookie = admin(&c, &base).await;
    let add = |body: Value| c.post(format!("{base}/admin/cookies/rules")).header("Cookie", &cookie).json(&body).send();
    assert_eq!(add(json!({"kind": "cookie", "pattern": "_ga*", "category": "analytics", "note": "vendor"})).await.unwrap().status(), 204);
    assert_eq!(add(json!({"kind": "cookie", "pattern": "*", "category": "analytics"})).await.unwrap().status(), 422, "a catch-all would classify everything");
    assert_eq!(add(json!({"kind": "cookie", "pattern": "x", "category": "weird"})).await.unwrap().status(), 422);
    assert_eq!(add(json!({"kind": "cookie", "pattern": "x", "category": "analytics", "note": "n".repeat(201)})).await.unwrap().status(), 422);
    // Same kind and pattern again changes the category rather than adding a second rule.
    assert_eq!(add(json!({"kind": "cookie", "pattern": "_ga*", "category": "marketing"})).await.unwrap().status(), 204);
    let inv: Value = c.get(format!("{base}/admin/cookies/inventory")).header("Cookie", &cookie).send().await.unwrap().json().await.unwrap();
    assert_eq!(inv["rules"].as_array().unwrap().len(), 1);
    assert_eq!(inv["rules"][0]["category"], "marketing");
    let id = inv["rules"][0]["id"].as_str().unwrap().to_owned();
    let r = c.delete(format!("{base}/admin/cookies/rules/{id}")).header("Cookie", &cookie).send().await.unwrap();
    assert_eq!(r.status(), 204);
    assert_eq!(c.delete(format!("{base}/admin/cookies/rules/{id}")).header("Cookie", &cookie).send().await.unwrap().status(), 404);
}
