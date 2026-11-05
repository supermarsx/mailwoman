//! Compliance over the real router: the generated public privacy and cookie
//! policy pages, overrides and versions, records of processing with exports, the
//! per-user processing endpoint, and the DPIA register workflow.
//!
//! SQLite only. Settings are written to the store before the app boots.

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
    let dir = test_db::unique_dir("mw-t33-compliance");
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

async fn admin(c: &reqwest::Client, base: &str) -> String {
    let resp = c
        .post(format!("{base}/admin/login"))
        .json(&json!({"username": "root", "password": "hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    resp.headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .map(|v| v.to_str().unwrap().split(';').next().unwrap().to_owned())
        .collect::<Vec<_>>()
        .join("; ")
}

fn configured() -> Vec<(&'static str, Value)> {
    vec![
        ("policy.controller.name", json!("Example Ltd")),
        ("policy.controller.address", json!("1 Example Road")),
        ("policy.legalBases", json!("Contract and legitimate interests")),
        ("policy.hosting.provider", json!("Example Hosting, Portugal")),
        ("policy.supervisoryAuthority", json!("CNPD")),
        ("cookies.enabled", json!(true)),
        ("cookies.analytics.enabled", json!(true)),
    ]
}

async fn get_json(c: &reqwest::Client, url: String, cookie: &str) -> Value {
    c.get(url).header("Cookie", cookie).send().await.unwrap().json().await.unwrap()
}

#[tokio::test]
async fn public_pages_render_the_configuration_in_both_languages_without_sign_in() {
    let base = spawn(&configured()).await;
    let c = client();
    let page = c.get(format!("{base}/privacy")).send().await.unwrap();
    assert_eq!(page.status(), 200);
    assert!(page.headers()["content-type"].to_str().unwrap().starts_with("text/html"));
    assert!(page.headers().contains_key("content-security-policy"));
    let html = page.text().await.unwrap();
    assert!(html.contains("Example Ltd") && html.contains("CNPD") && html.contains("Version 1"));
    assert!(!html.contains("not yet filled in"), "all operator fields are set");

    let pt = c.get(format!("{base}/privacy?lang=pt-PT")).send().await.unwrap().text().await.unwrap();
    assert!(pt.contains("Política de privacidade") && pt.contains("Quem é o responsável"));

    let md = c.get(format!("{base}/privacy?format=md")).send().await.unwrap().text().await.unwrap();
    assert!(md.starts_with("# Privacy policy") && md.contains("## Your rights"));

    let cookies = c.get(format!("{base}/cookies-policy")).send().await.unwrap().text().await.unwrap();
    assert!(cookies.contains("mw_session") && cookies.contains("Cookie policy"));

    let cfg: Value = c.get(format!("{base}/api/cookies/config")).send().await.unwrap().json().await.unwrap();
    assert_eq!(cfg["policyPages"], true);
}

#[tokio::test]
async fn the_pages_are_off_with_the_setting_and_blank_operator_fields_show_a_placeholder() {
    let base = spawn(&[("policy.publicPages", json!(false))]).await;
    let c = client();
    assert_eq!(c.get(format!("{base}/privacy")).send().await.unwrap().status(), 404);
    assert_eq!(c.get(format!("{base}/cookies-policy")).send().await.unwrap().status(), 404);
    let cfg: Value = c.get(format!("{base}/api/cookies/config")).send().await.unwrap().json().await.unwrap();
    assert_eq!(cfg["policyPages"], false);

    let base = spawn(&[]).await;
    let html = client().get(format!("{base}/privacy")).send().await.unwrap().text().await.unwrap();
    assert!(html.contains("not yet filled in by the operator"));
}

#[tokio::test]
async fn overrides_and_publishing_make_versions_with_a_change_list() {
    let base = spawn(&[("policy.autoVersion", json!(false))].into_iter().chain(configured()).collect::<Vec<_>>()).await;
    let c = client();
    let cookie = admin(&c, &base).await;
    let view = get_json(&c, format!("{base}/admin/compliance/policy"), &cookie).await;
    assert_eq!(view["version"], 1);
    assert_eq!(view["warnings"], json!([]));
    assert_eq!(view["pending"], json!([]));

    let put = c
        .put(format!("{base}/admin/compliance/policy/overrides"))
        .header("Cookie", &cookie)
        .json(&json!({"lang": "en", "section": "contact", "body": "Write to privacy@example.test."}))
        .send()
        .await
        .unwrap();
    assert_eq!(put.status(), 204);
    let bad = c
        .put(format!("{base}/admin/compliance/policy/overrides"))
        .header("Cookie", &cookie)
        .json(&json!({"lang": "fr", "section": "contact", "body": "x"}))
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 422);

    // Auto-versioning is off: the public page still shows version 1 without the override.
    let before = c.get(format!("{base}/privacy")).send().await.unwrap().text().await.unwrap();
    assert!(before.contains("Version 1") && !before.contains("privacy@example.test"));
    let view = get_json(&c, format!("{base}/admin/compliance/policy"), &cookie).await;
    assert_eq!(view["pending"][0]["key"], "override:en:contact");

    let published: Value = c
        .post(format!("{base}/admin/compliance/policy/publish"))
        .header("Cookie", &cookie)
        .json(&json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(published, json!({"version": 2, "changed": true}));
    let again: Value = c
        .post(format!("{base}/admin/compliance/policy/publish"))
        .header("Cookie", &cookie)
        .json(&json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(again["changed"], false);

    let after = c.get(format!("{base}/privacy")).send().await.unwrap().text().await.unwrap();
    assert!(after.contains("Version 2") && after.contains("privacy@example.test"));
    let view = get_json(&c, format!("{base}/admin/compliance/policy"), &cookie).await;
    assert_eq!(view["versions"][0]["version"], 2);
    assert_eq!(view["versions"][0]["changes"][0]["key"], "override:en:contact");
    assert_eq!(view["sections"].as_array().unwrap().iter().find(|s| s["id"] == "contact").unwrap()["overridden"], true);

    // Removing the override is a pending change again.
    c.put(format!("{base}/admin/compliance/policy/overrides"))
        .header("Cookie", &cookie)
        .json(&json!({"lang": "en", "section": "contact", "body": null}))
        .send()
        .await
        .unwrap();
    let view = get_json(&c, format!("{base}/admin/compliance/policy"), &cookie).await;
    assert_eq!(view["pending"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn blank_required_fields_are_listed_as_warnings() {
    let base = spawn(&[]).await;
    let c = client();
    let cookie = admin(&c, &base).await;
    let view = get_json(&c, format!("{base}/admin/compliance/policy"), &cookie).await;
    let warnings = view["warnings"].as_array().unwrap();
    assert!(warnings.contains(&json!("policy.controller.name")) && warnings.contains(&json!("policy.supervisoryAuthority")));
    let summary = get_json(&c, format!("{base}/admin/compliance/summary"), &cookie).await;
    assert_eq!(summary["policy"]["missing"].as_array().unwrap().len(), warnings.len());
}

#[tokio::test]
async fn records_are_generated_extended_and_exported() {
    let base = spawn(&configured()).await;
    let c = client();
    let cookie = admin(&c, &base).await;
    let recs = get_json(&c, format!("{base}/admin/compliance/records"), &cookie).await;
    let acts = recs["activities"].as_array().unwrap();
    let mail = acts.iter().find(|a| a["id"] == "mail-service").unwrap();
    assert_eq!(mail["builtin"], true);
    assert_eq!(mail["legalBasis"], "Contract and legitimate interests");
    assert_eq!(mail["recipients"], "Example Hosting, Portugal");
    assert!(acts.iter().any(|a| a["id"] == "cookie-consent" && a["enabled"] == true));
    assert!(acts.iter().any(|a| a["id"] == "open-tracking" && a["enabled"] == false));
    assert!(recs["processors"].as_array().unwrap().iter().any(|p| p["id"] == "push"));

    let created = c
        .post(format!("{base}/admin/compliance/records"))
        .header("Cookie", &cookie)
        .json(&json!({"kind": "activity", "name": "=Payroll", "fields": {"purpose": "Pay staff", "retention": "7 years"}}))
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), 201);
    let created: Value = created.json().await.unwrap();
    let bad = c
        .post(format!("{base}/admin/compliance/records"))
        .header("Cookie", &cookie)
        .json(&json!({"kind": "activity", "name": "x", "fields": {"nope": "y"}}))
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 422);

    let csv = c.get(format!("{base}/admin/compliance/records/export?format=csv")).header("Cookie", &cookie).send().await.unwrap();
    assert!(csv.headers()["content-type"].to_str().unwrap().starts_with("text/csv"));
    let csv = csv.text().await.unwrap();
    assert!(csv.starts_with("name,enabled,purpose,"));
    assert!(csv.contains("Pay staff") && csv.contains("7 years"));
    assert!(!csv.contains(",=Payroll") && !csv.starts_with("=Payroll") && !csv.contains("\r\n=Payroll"), "formula characters are defused");
    let html = c.get(format!("{base}/admin/compliance/records/export?format=html&set=processors")).header("Cookie", &cookie).send().await.unwrap().text().await.unwrap();
    assert!(html.contains("<table>") && html.contains("Push notification services"));
    let json_export: Value = c.get(format!("{base}/admin/compliance/records/export?format=json")).header("Cookie", &cookie).send().await.unwrap().json().await.unwrap();
    assert!(json_export.as_array().unwrap().len() > 8);

    let id = created["id"].as_str().unwrap();
    let del = c.delete(format!("{base}/admin/compliance/records/{id}")).header("Cookie", &cookie).send().await.unwrap();
    assert_eq!(del.status(), 204);
    assert_eq!(c.delete(format!("{base}/admin/compliance/records/{id}")).header("Cookie", &cookie).send().await.unwrap().status(), 404);
}

#[tokio::test]
async fn compliance_routes_need_an_admin_and_the_user_view_needs_a_session() {
    let base = spawn(&configured()).await;
    let c = client();
    for path in ["/admin/compliance/policy", "/admin/compliance/records", "/admin/compliance/dpia", "/admin/compliance/summary"] {
        let status = c.get(format!("{base}{path}")).send().await.unwrap().status();
        assert!(status == 401 || status == 403, "{path} -> {status}");
    }
    let status = c.get(format!("{base}/api/privacy/processing")).send().await.unwrap().status();
    assert!(status == 401 || status == 403, "processing -> {status}");
}

#[tokio::test]
async fn dpia_follows_the_workflow_with_versions_and_conflicts() {
    let base = spawn(&configured()).await;
    let c = client();
    let cookie = admin(&c, &base).await;

    let templates = get_json(&c, format!("{base}/admin/compliance/dpia/templates?lang=pt-PT"), &cookie).await;
    assert_eq!(templates["templates"].as_array().unwrap().len(), 5);

    let created: Value = c
        .post(format!("{base}/admin/compliance/dpia"))
        .header("Cookie", &cookie)
        .json(&json!({"title": "Assistant", "template": "assistant"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = created["id"].as_str().unwrap().to_owned();
    assert_eq!(created["status"], "draft");
    assert_eq!(created["body"]["risks"][0]["score"], 9);
    assert_eq!(created["body"]["risks"][0]["level"], "high");
    let url = format!("{base}/admin/compliance/dpia/{id}");
    let put = |body: Value| c.put(&url).header("Cookie", &cookie).json(&body).send();

    // draft -> approved is not a move
    let r = put(json!({"version": 1, "status": "approved"})).await.unwrap();
    assert_eq!(r.status(), 422);
    // a stale version is refused
    let r = put(json!({"version": 7, "title": "x"})).await.unwrap();
    assert_eq!(r.status(), 409);
    // draft -> review
    let r: Value = put(json!({"version": 1, "status": "review"})).await.unwrap().json().await.unwrap();
    assert_eq!(r["version"], 2);
    // approval needs residual scores on every risk
    let r = put(json!({"version": 2, "status": "approved"})).await.unwrap();
    assert_eq!(r.status(), 422);
    assert_eq!(r.json::<Value>().await.unwrap()["code"], "residualMissing");

    let mut body = created["body"].clone();
    for risk in body["risks"].as_array_mut().unwrap() {
        risk["residualLikelihood"] = json!(4);
        risk["residualSeverity"] = json!(4);
    }
    let r = put(json!({"version": 2, "body": body.clone(), "status": "approved"})).await.unwrap();
    assert_eq!(r.status(), 422);
    assert_eq!(r.json::<Value>().await.unwrap()["code"], "residualVeryHigh");

    for risk in body["risks"].as_array_mut().unwrap() {
        risk["residualLikelihood"] = json!(1);
        risk["residualSeverity"] = json!(2);
    }
    let r: Value = put(json!({"version": 2, "body": body, "status": "approved"})).await.unwrap().json().await.unwrap();
    assert_eq!(r["status"], "approved");
    assert_eq!(r["version"], 3);
    assert!(r["reviewAt"].as_i64().unwrap() > chrono_now() + 300 * 86_400, "review date set from dpia.reviewMonths");
    assert_eq!(r["body"]["risks"][0]["residualLevel"], "low");

    let one = get_json(&c, url.clone(), &cookie).await;
    assert_eq!(one["history"].as_array().unwrap().len(), 3);

    let html = c.get(format!("{url}/export")).header("Cookie", &cookie).send().await.unwrap().text().await.unwrap();
    assert!(html.contains("Assistant") && html.contains("Risks"));

    let list = get_json(&c, format!("{base}/admin/compliance/dpia"), &cookie).await;
    assert_eq!(list["items"].as_array().unwrap().len(), 1);
    // The assistant is covered by this assessment, so only the other risky features are suggested.
    let sug = get_json(&c, format!("{base}/admin/compliance/dpia/suggestions"), &cookie).await;
    assert!(!sug["items"].as_array().unwrap().iter().any(|s| s["feature"] == "assistant"));

    let bad = c.post(format!("{base}/admin/compliance/dpia")).header("Cookie", &cookie).json(&json!({"title": "x", "template": "nope"})).send().await.unwrap();
    assert_eq!(bad.status(), 422);
    assert_eq!(c.delete(&url).header("Cookie", &cookie).send().await.unwrap().status(), 204);
    assert_eq!(c.get(&url).header("Cookie", &cookie).send().await.unwrap().status(), 404);
}

fn chrono_now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64
}

#[tokio::test]
async fn an_enabled_risky_feature_without_an_assessment_is_suggested_until_covered() {
    let base = spawn(&[("mail.openTracking", json!(true))]).await;
    let c = client();
    let cookie = admin(&c, &base).await;
    let sug = get_json(&c, format!("{base}/admin/compliance/dpia/suggestions"), &cookie).await;
    assert!(sug["items"].as_array().unwrap().iter().any(|s| s["feature"] == "openTracking" && s["template"] == "email-tracking"));
    c.post(format!("{base}/admin/compliance/dpia")).header("Cookie", &cookie).json(&json!({"title": "Tracking", "template": "email-tracking"})).send().await.unwrap();
    let sug = get_json(&c, format!("{base}/admin/compliance/dpia/suggestions"), &cookie).await;
    assert!(!sug["items"].as_array().unwrap().iter().any(|s| s["feature"] == "openTracking"));
    let off = spawn(&[("mail.openTracking", json!(true)), ("dpia.suggest", json!(false))]).await;
    let cookie = admin(&c, &off).await;
    let sug = get_json(&c, format!("{off}/admin/compliance/dpia/suggestions"), &cookie).await;
    assert_eq!(sug["items"], json!([]));
}

#[tokio::test]
async fn reconsent_raises_the_cookie_policy_version_when_a_new_version_is_recorded() {
    let base = spawn(&[
        ("policy.reconsentOnChange", json!(true)),
        ("cookies.enabled", json!(true)),
        ("cookies.policyVersion", json!(4)),
    ])
    .await;
    let c = client();
    let cookie = admin(&c, &base).await;
    // First version: recorded without touching the cookie version.
    c.get(format!("{base}/privacy")).send().await.unwrap();
    let cfg: Value = c.get(format!("{base}/api/cookies/config")).send().await.unwrap().json().await.unwrap();
    assert_eq!(cfg["version"], 4);
    c.put(format!("{base}/admin/compliance/policy/overrides"))
        .header("Cookie", &cookie)
        .json(&json!({"lang": "en", "section": "contact", "body": "New contact text."}))
        .send()
        .await
        .unwrap();
    // The public request records version 2 automatically and raises the cookie version.
    let page = c.get(format!("{base}/privacy")).send().await.unwrap().text().await.unwrap();
    assert!(page.contains("Version 2"));
    let cfg: Value = c.get(format!("{base}/api/cookies/config")).send().await.unwrap().json().await.unwrap();
    assert_eq!(cfg["version"], 5);
}
