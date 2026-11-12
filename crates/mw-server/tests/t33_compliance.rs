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

async fn spawn(settings: &[(&str,Value)])->String{spawn_state(settings).await.0}

async fn spawn_state(settings: &[(&str, Value)]) -> (String,mw_store::Store) {
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
    (format!("http://{addr}"),store)
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

#[tokio::test]
async fn governance_links_contracts_and_validates_international_transfers_and_verification(){
    let base=spawn(&[]).await;let c=client();let cookie=admin(&c,&base).await;
    assert_eq!(c.get(format!("{base}/admin/compliance/governance")).send().await.unwrap().status(),401);
    let view=get_json(&c,format!("{base}/admin/compliance/governance"),&cookie).await;
    assert_eq!(view["catalog"].as_array().unwrap().len(),15);
    assert!(view["catalog"].as_array().unwrap().iter().all(|row|row["status"]=="unassessed"));
    let post=|body:Value|c.post(format!("{base}/admin/compliance/records")).header("Cookie",&cookie).json(&body);
    let contract:Value=post(json!({"kind":"contract","name":"Hosting DPA","fields":{"status":"active","contractType":"dpa","owner":"Privacy","counterparty":"Provider","signedAt":"2026-11-01","documentUrl":"https://example.org/dpa"}})).send().await.unwrap().json().await.unwrap();
    assert!(contract["id"].is_string(),"{contract}");
    let bad=post(json!({"kind":"control","name":"Encryption","fields":{"status":"verified"}})).send().await.unwrap();assert_eq!(bad.status(),422);
    let bad=post(json!({"kind":"transfer","name":"Transfer","fields":{"status":"active","direction":"international","mechanism":"internal"}})).send().await.unwrap();assert_eq!(bad.status(),422);
    let transfer=post(json!({"kind":"transfer","name":"Offshore support","fields":{"status":"active","direction":"international","mechanism":"scc","owner":"Privacy","exporter":"Controller","importer":"Support","sourceCountry":"PT","destinationCountry":"US","purpose":"Support","categories":"Account metadata","legalBasis":"Contract","safeguards":"Restricted access","assessmentUrl":"https://example.org/tia","reviewAt":"2027-11-01","contractId":contract["id"]}})).send().await.unwrap();assert_eq!(transfer.status(),201,"{}",transfer.text().await.unwrap());
    let exported=c.get(format!("{base}/admin/compliance/records/export?set=contract&format=csv")).header("Cookie",&cookie).send().await.unwrap().text().await.unwrap();assert!(exported.contains("Hosting DPA")&&exported.contains("documentUrl"));
    let test=c.post(format!("{base}/admin/compliance/mail/test")).header("Cookie",&cookie).json(&json!({})).send().await.unwrap();assert_eq!(test.status(),422);
}

#[test]
fn encrypted_file_migration_and_uncertain_delivery_resolution_require_admin_control(){
    // Logging policy is process-wide; isolate this fixture from parallel servers.
    let result=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","isolated_file_migration_and_delivery_resolution","--ignored","--nocapture"]).output().unwrap();
    assert!(result.status.success()&&String::from_utf8_lossy(&result.stdout).contains("1 passed"),"{}\n{}",String::from_utf8_lossy(&result.stdout),String::from_utf8_lossy(&result.stderr));
}

#[tokio::test]
#[ignore = "exercised by the parent test in an isolated process"]
async fn isolated_file_migration_and_delivery_resolution(){
    use base64::Engine as _;
    let dir=test_db::unique_dir("mw-compliance-encrypted-logs");std::fs::create_dir_all(&dir).unwrap();
    let log=dir.join("mailwoman.log");std::fs::write(&log,b"privateperson@example.org event\n").unwrap();
    let (base,store)=spawn_state(&[("logging.file.directory",json!(dir)),("logging.file.enabled",json!(false))]).await;
    let c=client();let cookie=admin(&c,&base).await;
    let migration=c.post(format!("{base}/admin/encryption/encrypt-files")).header("Cookie",&cookie).json(&json!({})).send().await.unwrap();assert_eq!(migration.status(),200,"{}",migration.text().await.unwrap());
    let bytes=std::fs::read(&log).unwrap();assert!(bytes.starts_with(b"MWLOG1:"));assert!(!String::from_utf8_lossy(&bytes).contains("privateperson@example.org"));
    let notice=json!({"recipient":"external@example.org","subject":"notice","text":"private body","attempts":1,"error":"uncertain"});
    let payload=store.seal_compliance_delivery("uncertain-notice",notice.to_string().as_bytes()).unwrap();
    store.put_compliance_record(&mw_store::compliance::ComplianceRecord{id:"uncertain-notice".into(),kind:"compliance-mail:uncertain".into(),name:"test-event".into(),body_json:base64::engine::general_purpose::STANDARD.encode(payload),updated_by:"system".into(),updated_at:1}).await.unwrap();
    let url=format!("{base}/admin/compliance/mail/uncertain-notice/resolve");
    assert_eq!(c.post(&url).header("Cookie",&cookie).json(&json!({"action":"retry"})).send().await.unwrap().status(),422);
    assert_eq!(c.post(&url).header("Cookie",&cookie).json(&json!({"action":"retry","acceptDuplicateRisk":true})).send().await.unwrap().status(),200);
    assert_eq!(store.get_compliance_record("uncertain-notice").await.unwrap().unwrap().kind,"compliance-mail:pending");
}


#[tokio::test]
async fn operational_incidents_debounce_health_verify_recovery_and_separate_nis2_reporting(){
 let(base,store)=spawn_state(&[("incidents.auto.failureSamples",json!(2))]).await;let c=client();let cookie=admin(&c,&base).await;
 assert_eq!(c.get(format!("{base}/admin/incidents")).send().await.unwrap().status(),401);
 let post=|suffix:&str,body:Value|c.post(format!("{base}/admin/incidents{suffix}")).header("Cookie",&cookie).json(&body);
 let at=chrono::Utc::now().timestamp();let monitor_id="operational-monitor:external-mail";
 store.put_compliance_record(&mw_store::compliance::ComplianceRecord{id:monitor_id.into(),kind:"operational-monitor".into(),name:"external-mail".into(),body_json:json!({"lastAt":at-61,"failures":1,"healthy":0,"since":at-120}).to_string(),updated_by:"test".into(),updated_at:at-61}).await.unwrap();
 assert_eq!(post("/observations",json!({"component":"mail","status":"fail"})).send().await.unwrap().status(),200);
 assert_eq!(post("/observations",json!({"component":"mail","status":"fail"})).send().await.unwrap().status(),200);
 let list=get_json(&c,format!("{base}/admin/incidents"),&cookie).await;
 let rows=list["items"].as_array().unwrap();let rows:Vec<_>=rows.iter().filter(|r|r["incident"]["component"]=="external-mail").collect();assert_eq!(rows.len(),1,"{list}");let id=rows[0]["id"].as_str().unwrap();assert_eq!(c.delete(format!("{base}/admin/compliance/records/{id}")).header("Cookie",&cookie).send().await.unwrap().status(),404,"Internal incidents must not be deleted through generic settings-write routes");assert_eq!(rows[0]["incident"]["significance"],"unassessed");assert_eq!(rows[0]["deadlines"],json!({}));
 let mut monitor=store.get_compliance_record(monitor_id).await.unwrap().unwrap();let mut payload:Value=serde_json::from_str(&monitor.body_json).unwrap();payload["lastAt"]=json!(at-61);payload["healthy"]=json!(1);payload["healthySince"]=json!(at-61);monitor.body_json=payload.to_string();store.put_compliance_record(&monitor).await.unwrap();
 assert_eq!(post("/observations",json!({"component":"mail","status":"ok"})).send().await.unwrap().status(),200);
 let list=get_json(&c,format!("{base}/admin/incidents"),&cookie).await;let row=list["items"].as_array().unwrap().iter().find(|r|r["id"]==id).unwrap();assert_eq!(row["incident"]["status"],"recovering");assert_eq!(row["recoveryVerified"],false);
 let mut body=json!({"title":"Mail outage","category":"outage","status":"closed","significance":"significant","awareAt":at-120,"updatedAt":row["updatedAt"],"version":row["version"]});
 let put=|body:Value|c.put(format!("{base}/admin/incidents/{id}")).header("Cookie",&cookie).json(&body);
 assert_eq!(put(body.clone()).send().await.unwrap().status(),422);
 body["rootCause"]=json!("Dependency outage");body["recoveryEvidence"]=json!("Delivery and login verified by operator");let stale=body.clone();assert_eq!(put(body).send().await.unwrap().status(),200);assert_eq!(put(stale).send().await.unwrap().status(),409);
 let report=post(&format!("/{id}/reports"),json!({"phase":"notification","submittedAt":at,"reference":"Authority case 123"})).send().await.unwrap();let code=report.status();let detail=report.text().await.unwrap();assert_eq!(code,200,"{detail}");
 let exported=get_json(&c,format!("{base}/admin/incidents/{id}/export"),&cookie).await;assert_eq!(exported["preparationOnly"],true);assert_eq!(exported["deadlines"]["final"],json!(chrono::DateTime::from_timestamp(at,0).unwrap().checked_add_months(chrono::Months::new(1)).unwrap().timestamp()));assert_eq!(exported["recoveryVerified"],true);
}

#[tokio::test]
async fn security_signal_cursor_survives_equal_timestamp_pages_and_monitoring_gaps_are_explicit(){
 let(base,store)=spawn_state(&[]).await;let c=client();let cookie=admin(&c,&base).await;let at=chrono::Utc::now();
 for(id,name,payload,updated)in [
  ("operational:audit-cursor","audit-cursor",json!({"ts":(at-chrono::Duration::seconds(60)).to_rfc3339_opts(chrono::SecondsFormat::Nanos,false),"id":""}),at.timestamp()),
  ("operational:heartbeat","heartbeat",json!({"at":at.timestamp()-600}),at.timestamp()-600)
 ]{store.put_compliance_record(&mw_store::compliance::ComplianceRecord{id:id.into(),kind:"operational-monitor".into(),name:name.into(),body_json:payload.to_string(),updated_by:"test".into(),updated_at:updated}).await.unwrap();}
 let ts=(at-chrono::Duration::seconds(30)).to_rfc3339_opts(chrono::SecondsFormat::Nanos,false);
 for n in 0..201{store.append_audit(&mw_store::AuditRow{id:format!("signal-{n:03}"),ts:ts.clone(),actor:"private-account@example.org".into(),actor_kind:"user".into(),action:if n==200{"session-security-alert"}else{"irrelevant-event"}.into(),target:Some("private-account@example.org".into()),detail_json:json!({"kind":"rotationReuse","reason":"private evidence"}).to_string(),ip:Some("192.0.2.1".into())}).await.unwrap();}
 for _ in 0..3{assert_eq!(c.post(format!("{base}/admin/incidents/scan")).header("Cookie",&cookie).json(&json!({})).send().await.unwrap().status(),200);}
 let list=get_json(&c,format!("{base}/admin/incidents"),&cookie).await;let items=list["items"].as_array().unwrap();let security:Vec<_>=items.iter().filter(|r|r["incident"]["source"]=="security-audit").collect();assert_eq!(security.len(),1,"{list}");assert_eq!(security[0]["incident"]["auditReference"],"signal-200");assert_eq!(security[0]["incident"]["significance"],"unassessed");assert!(!list.to_string().contains("private-account@example.org"));assert!(!list.to_string().contains("192.0.2.1"));let gaps:Vec<_>=items.iter().filter(|r|r["incident"]["source"]=="monitoring-gap").collect();assert_eq!(gaps.len(),1);assert!(gaps[0]["incident"]["impact"].as_str().unwrap().contains("unknown"));
}


#[tokio::test]
async fn nis2_final_deadline_clamps_calendar_month_without_extending_prompt_recovery(){
 let base=spawn(&[]).await;let c=client();let cookie=admin(&c,&base).await;
 let created=c.post(format!("{base}/admin/incidents")).header("Cookie",&cookie).json(&json!({"title":"Historical recovery exercise","category":"outage","status":"resolved","significance":"significant","awareAt":1769731200_i64,"handledAt":1769904000_i64,"rootCause":"Dependency interruption","recoveryEvidence":"Operator recovery checks"})).send().await.unwrap();assert_eq!(created.status(),201);let created:Value=created.json().await.unwrap();let id=created["id"].as_str().unwrap();
 let report=c.post(format!("{base}/admin/incidents/{id}/reports")).header("Cookie",&cookie).json(&json!({"phase":"notification","submittedAt":1769817600_i64,"reference":"Historical authority reference"})).send().await.unwrap();let code=report.status();let detail=report.text().await.unwrap();assert_eq!(code,200,"{detail}");
 let export=get_json(&c,format!("{base}/admin/incidents/{id}/export"),&cookie).await;assert_eq!(export["deadlines"]["final"],1772236800_i64,"January 31 notification must be due February 28, even though recovery was February 1");assert!(export["deadlines"].get("progress").is_none());
}
