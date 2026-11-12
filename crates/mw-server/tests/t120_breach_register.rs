//! The breach register end to end: opening an incident and the notice clock,
//! editing the awareness time, the risk questionnaire, status guards, the
//! authority notice (late notices need reasons for delay), reminders, the data
//! subject message with a dry run and a delivery log, automation rules and the
//! containment playbook.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::settings_policy::SettingPolicyRow;
use serde_json::{Value, json};
use std::net::SocketAddr;

mod common;
use common::auth;

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const HOUR: i64 = 3600;

async fn serve(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
    });
    format!("http://{addr}")
}

async fn set(store: &mw_store::Store, key: &str, value: Value) {
    store
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}

struct Fixture {
    base: String,
    store: mw_store::Store,
    user: reqwest::Client,
    admin: reqwest::Client,
    account_id: String,
}

async fn fixture() -> Fixture {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("t120-breach");
    let store = mw_store::Store::open(db.to_str().unwrap(), mw_store::ServerKey::from_hex(KEY).unwrap()).await.unwrap();
    set(&store, "signin.retryCooldownSecs", json!(0)).await;
    let (router, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(KEY.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(upstream.clone()),
                jmap_upstreams: Some(vec![upstream]),
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
    let base = serve(router).await;
    let signed = auth::sign_in(&base, mw_mock_jmap::USER, mw_mock_jmap::PASS).await;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::COOKIE, signed.cookie().parse().unwrap());
    if let Some(a) = &signed.storage_authority {
        headers.insert(auth::STORAGE_AUTHORITY_HEADER, a.parse().unwrap());
    }
    let user = reqwest::Client::builder().no_proxy().default_headers(headers).build().unwrap();
    // The directory binding that ties the address to the signed-in account.
    let (local, domain) = mw_mock_jmap::USER.split_once('@').unwrap();
    store
        .register_successful_signin(mw_store::user_directory::SuccessfulSignin {
            account_id: &signed.account_id,
            address: mw_mock_jmap::USER,
            domain,
            local_part: local,
            mode: "engine",
            now: chrono::Utc::now().timestamp(),
        })
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(3100)).await;
    let admin = reqwest::Client::builder().no_proxy().cookie_store(true).build().unwrap();
    let r = admin
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":"root","password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    Fixture { base, store, user, admin, account_id: signed.account_id.clone() }
}

impl Fixture {
    async fn post(&self, path: &str, body: Value) -> (u16, Value) {
        let r = self.admin.post(format!("{}/admin{path}", self.base)).json(&body).send().await.unwrap();
        (r.status().as_u16(), r.json().await.unwrap_or(Value::Null))
    }
    async fn get(&self, path: &str) -> (u16, Value) {
        let r = self.admin.get(format!("{}/admin{path}", self.base)).send().await.unwrap();
        (r.status().as_u16(), r.json().await.unwrap_or(Value::Null))
    }
    async fn text(&self, path: &str) -> (u16, String) {
        let r = self.admin.get(format!("{}/admin{path}", self.base)).send().await.unwrap();
        (r.status().as_u16(), r.text().await.unwrap())
    }
    async fn open(&self, body: Value) -> Value {
        let (code, v) = self.post("/breaches", body).await;
        assert_eq!(code, 201, "{v}");
        v["incident"].clone()
    }
    async fn trail(&self, id: &str) -> Vec<String> {
        let (_, v) = self.get(&format!("/breaches/{id}")).await;
        v["trail"].as_array().unwrap().iter().map(|e| e["kind"].as_str().unwrap().to_string()).collect()
    }
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

const ALL_ANSWERS: [(&str, &str); 8] = [
    ("confidentiality", "limited"),
    ("special", "no"),
    ("identifiable", "easy"),
    ("volume", "some"),
    ("vulnerable", "no"),
    ("encrypted", "no"),
    ("contained", "partial"),
    ("harm", "none"),
];

fn answers() -> Value {
    Value::Object(ALL_ANSWERS.iter().map(|(k, v)| (k.to_string(), json!(v))).collect())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_notice_clock_runs_from_awareness_and_edits_are_audited() {
    let f = fixture().await;
    let aware = now() - 5 * HOUR;
    let i = f.open(json!({"title": "Mailbox exposed", "description": "A shared mailbox was readable.", "incidentType": "mailbox-access", "awareAt": aware})).await;
    let id = i["id"].as_str().unwrap().to_string();
    assert_eq!(i["awareAt"], aware);
    assert_eq!(i["deadlineAt"], aware + 72 * HOUR, "default window is 72 hours");
    assert_eq!(i["clock"]["phase"], "running");
    assert_eq!(i["status"], "detected");

    // The window is a setting.
    set(&f.store, "breach.noticeHours", json!(48)).await;
    let j = f.open(json!({"title": "Second", "awareAt": aware})).await;
    assert_eq!(j["deadlineAt"], aware + 48 * HOUR);

    // Changing the awareness time needs a reason and moves the deadline.
    let (code, v) = f.post(&format!("/breaches/{id}/aware-at"), json!({"awareAt": aware + HOUR, "reason": ""})).await;
    assert_eq!(code, 422, "{v}");
    assert_eq!(v["code"], "reasonRequired");
    let (code, v) = f.post(&format!("/breaches/{id}/aware-at"), json!({"awareAt": aware + 2 * HOUR, "reason": "Log review showed later access"})).await;
    assert_eq!(code, 200, "{v}");
    assert_eq!(v["incident"]["deadlineAt"], aware + 2 * HOUR + 48 * HOUR);
    assert!(f.trail(&id).await.contains(&"aware-at-changed".to_string()));
    let (_, audit) = f.get("/audit?limit=50").await;
    assert!(audit.to_string().contains("breach-changed"), "the change is in the main audit log too");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_draft_has_no_clock_until_it_is_confirmed() {
    let f = fixture().await;
    let i = f.open(json!({"title": "Detected by rule", "draft": true})).await;
    let id = i["id"].as_str().unwrap();
    assert_eq!(i["clock"]["phase"], "not-started");
    assert_eq!(i["draft"], true);
    let (code, v) = f.post(&format!("/breaches/{id}/authority-notice"), json!({})).await;
    assert_eq!(code, 409, "{v}");
    assert_eq!(v["code"], "clockNotStarted");
    let (code, v) = f.post(&format!("/breaches/{id}/confirm"), json!({})).await;
    assert_eq!(code, 200, "{v}");
    assert_eq!(v["incident"]["clock"]["phase"], "running");
    assert!(v["incident"]["deadlineAt"].as_i64().unwrap() > now() + 71 * HOUR);
    let (code, _) = f.post(&format!("/breaches/{id}/confirm"), json!({})).await;
    assert_eq!(code, 409, "the clock cannot be started twice");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn assessment_status_guards_and_a_late_authority_notice() {
    let f = fixture().await;
    let aware = now() - 80 * HOUR;
    let i = f.open(json!({"title": "Late one", "awareAt": aware, "dataCategories": ["email addresses"], "approxSubjects": 40, "affected": [mw_mock_jmap::USER]})).await;
    let id = i["id"].as_str().unwrap().to_string();
    assert_eq!(i["clock"]["phase"], "overdue");
    assert_eq!(i["delayReasonsRequired"], true);

    // Questionnaire: a level different from the suggestion needs reasoning.
    let (code, v) = f.post(&format!("/breaches/{id}/assess"), json!({"answers": answers()})).await;
    assert_eq!(code, 200, "{v}");
    assert_eq!(v["incident"]["severity"], "risk");
    assert_eq!(v["incident"]["status"], "assessing");
    let (code, v) = f.post(&format!("/breaches/{id}/assess"), json!({"answers": answers(), "severity": "high"})).await;
    assert_eq!(code, 422, "{v}");
    assert_eq!(v["code"], "rationaleRequired");
    let (code, v) = f.post(&format!("/breaches/{id}/assess"), json!({"answers": {"confidentiality": "bogus"}})).await;
    assert_eq!(code, 422, "{v}");

    // Guards.
    let (code, v) = f.post(&format!("/breaches/{id}/status"), json!({"status": "not-notifiable", "reason": "we think it is fine"})).await;
    assert_eq!(code, 409, "{v}");
    assert_eq!(v["code"], "needsNoRiskAssessment");
    let (code, v) = f.post(&format!("/breaches/{id}/status"), json!({"status": "notified-authority"})).await;
    assert_eq!(v["code"], "needsAuthorityRecord", "{v}");
    assert_eq!(code, 409);
    let (code, v) = f.post(&format!("/breaches/{id}/status"), json!({"status": "closed", "reason": ""})).await;
    assert_eq!(v["code"], "needsCloseReasoning", "{v}");
    assert_eq!(code, 409);

    // The notice document carries the Art. 33(3) items.
    set(&f.store, "breach.authority.name", json!("Supervisory Authority")).await;
    set(&f.store, "breach.dpo.email", json!("dpo@example.test")).await;
    let (code, v) = f.get(&format!("/breaches/{id}/authority-notice")).await;
    assert_eq!(code, 200);
    let n = &v["notice"];
    assert_eq!(n["nature"]["approximateDataSubjects"], 40);
    assert_eq!(n["nature"]["categoriesOfData"][0], "email addresses");
    assert_eq!(n["dataProtectionOfficer"]["email"], "dpo@example.test");
    assert_eq!(n["authority"]["name"], "Supervisory Authority");
    assert_eq!(n["delay"]["afterDeadline"], true);
    let (code, html) = f.text(&format!("/breaches/{id}/authority-notice?format=html&locale=pt-PT")).await;
    assert_eq!(code, 200);
    assert!(html.contains("Natureza da violação") && html.contains("dpo@example.test"));

    // A late initial notice needs the reasons for delay (Art. 33(1)).
    let (code, v) = f.post(&format!("/breaches/{id}/authority-notice"), json!({"reference": "AUTH-1"})).await;
    assert_eq!(code, 422, "{v}");
    assert_eq!(v["code"], "delayReasonsRequired");
    let (code, v) = f.post(&format!("/breaches/{id}/authority-notice"), json!({"reference": "AUTH-1", "delayReasons": "Scope was unclear until the log review finished."})).await;
    assert_eq!(code, 200, "{v}");
    assert_eq!(v["incident"]["status"], "notified-authority");
    assert_eq!(v["incident"]["clock"]["phase"], "stopped-late");
    assert_eq!(v["incident"]["delayReasonsRequired"], false);
    // A follow-up is a phased update.
    let (code, v) = f.post(&format!("/breaches/{id}/authority-notice"), json!({"phase": "update"})).await;
    assert_eq!(code, 200, "{v}");
    assert_eq!(v["incident"]["body"]["authorityNotices"].as_array().unwrap().len(), 2);

    // Now it can be closed, and reopening needs a reason.
    let (code, v) = f.post(&format!("/breaches/{id}/status"), json!({"status": "closed", "reason": "Cause removed and verified."})).await;
    assert_eq!(code, 200, "{v}");
    let (code, v) = f.post(&format!("/breaches/{id}/status"), json!({"status": "assessing", "reason": ""})).await;
    assert_eq!(v["code"], "needsReopenReason", "{v}");
    assert_eq!(code, 409);

    // Register exports.
    let year = chrono::Utc::now().format("%Y");
    let (code, csv) = f.text(&format!("/breaches/export?year={year}&format=csv")).await;
    assert_eq!(code, 200);
    assert!(csv.starts_with("reference,id,title") && csv.contains("Late one"));
    assert!(csv.contains("true"), "the late notice is flagged");
    let (_, json_export) = f.get(&format!("/breaches/export?year={year}&format=json")).await;
    assert!(json_export["count"].as_i64().unwrap() >= 1);
    let (code, report) = f.text(&format!("/breaches/{id}/report?format=html")).await;
    assert_eq!(code, 200);
    assert!(report.contains("Late one"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn not_notifiable_needs_a_no_risk_assessment_and_reasoning() {
    let f = fixture().await;
    let i = f.open(json!({"title": "Encrypted laptop"})).await;
    let id = i["id"].as_str().unwrap();
    let low = json!({"confidentiality": "none", "special": "no", "identifiable": "no", "volume": "few", "vulnerable": "no", "encrypted": "yes", "contained": "yes", "harm": "none"});
    let (_, v) = f.post(&format!("/breaches/{id}/assess"), json!({"answers": low})).await;
    assert_eq!(v["incident"]["severity"], "none");
    let (code, v) = f.post(&format!("/breaches/{id}/status"), json!({"status": "not-notifiable", "reason": "too short"})).await;
    assert_eq!(v["code"], "needsReasoning", "{v}");
    assert_eq!(code, 409);
    let (code, v) = f.post(&format!("/breaches/{id}/status"), json!({"status": "not-notifiable", "reason": "Disk was encrypted and the key was not exposed."})).await;
    assert_eq!(code, 200, "{v}");
    assert_eq!(v["incident"]["clock"]["phase"], "not-required");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reminders_fire_at_the_thresholds_and_when_overdue() {
    let f = fixture().await;
    // 49 hours after awareness: 23 hours left, past the 24 hour threshold.
    let i = f.open(json!({"title": "Reminder", "awareAt": now() - 49 * HOUR})).await;
    let id = i["id"].as_str().unwrap().to_string();
    let (code, _) = f.post("/breaches/run", json!({})).await;
    assert_eq!(code, 200);
    let trail = f.trail(&id).await;
    assert_eq!(trail.iter().filter(|k| *k == "reminder").count(), 1, "{trail:?}");
    f.post("/breaches/run", json!({})).await;
    assert_eq!(f.trail(&id).await.iter().filter(|k| *k == "reminder").count(), 1, "one reminder per threshold");

    // Past the deadline.
    let o = f.open(json!({"title": "Overdue", "awareAt": now() - 80 * HOUR})).await;
    let oid = o["id"].as_str().unwrap().to_string();
    f.post("/breaches/run", json!({})).await;
    assert_eq!(f.trail(&oid).await.iter().filter(|k| *k == "overdue").count(), 1);
    // Reminders show in the main audit log as security events.
    let (_, audit) = f.get("/audit?limit=200").await;
    assert!(audit.to_string().contains("breach-notice"));
    // A recorded authority notice ends the reminders.
    let (code, _) = f.post(&format!("/breaches/{oid}/authority-notice"), json!({"delayReasons": "Scope unclear until review ended."})).await;
    assert_eq!(code, 200);
    f.post("/breaches/run", json!({})).await;
    assert_eq!(f.trail(&oid).await.iter().filter(|k| *k == "overdue").count(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_message_to_people_has_a_dry_run_templates_and_a_delivery_log() {
    let f = fixture().await;
    set(&f.store, "breach.dpo.email", json!("dpo@example.test")).await;
    set(&f.store, "breach.template.subject", json!({"pt-PT": "Aviso: {{incident}}"})).await;
    let i = f.open(json!({"title": "Contacts exposed", "description": "A list was shared.", "dataCategories": ["names"], "affected": [mw_mock_jmap::USER, "nobody@example.test"]})).await;
    let id = i["id"].as_str().unwrap().to_string();
    assert_eq!(i["body"]["affected"].as_array().unwrap().len(), 2);

    // Dry run: nothing is written and the plan names each channel.
    let (code, v) = f.post(&format!("/breaches/{id}/subject-notice"), json!({"audience": {"kind": "all"}, "locale": "pt-PT", "dryRun": true, "advice": "Change your password."})).await;
    assert_eq!(code, 200, "{v}");
    assert_eq!(v["dryRun"], true);
    assert_eq!(v["recipients"].as_array().unwrap().len(), 2);
    assert!(v["preview"]["subject"].as_str().unwrap().starts_with("Aviso: Contacts exposed (BR-"), "{v}");
    assert!(v["preview"]["body"].as_str().unwrap().contains("dpo@example.test"));
    assert!(v["preview"]["body"].as_str().unwrap().contains("Change your password."));
    assert_eq!(v["preview"]["unknownVariables"]["body"].as_array().unwrap().len(), 0);
    let (_, d) = f.get(&format!("/breaches/{id}")).await;
    assert_eq!(d["deliveries"].as_array().unwrap().len(), 0, "a dry run delivers nothing");

    // Audiences.
    let (code, v) = f.post(&format!("/breaches/{id}/subject-notice"), json!({"audience": {"kind": "selected", "addresses": ["stranger@example.test"]}, "dryRun": true})).await;
    assert_eq!(code, 422);
    assert_eq!(v["code"], "notAffected");
    let (_, v) = f.post(&format!("/breaches/{id}/subject-notice"), json!({"audience": {"kind": "selected", "addresses": [mw_mock_jmap::USER]}, "dryRun": true})).await;
    assert_eq!(v["recipients"].as_array().unwrap().len(), 1);

    // Send: the in-app notice reaches the person; every recipient/channel is logged.
    let (code, v) = f.post(&format!("/breaches/{id}/subject-notice"), json!({"audience": {"kind": "all"}, "locale": "en", "dryRun": false})).await;
    assert_eq!(code, 202, "{v}");
    let mut finished = false;
    for _ in 0..100 {
        if f.trail(&id).await.contains(&"subject-notice-finished".to_string()) {
            finished = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert!(finished, "delivery finished");
    let (_, d) = f.get(&format!("/breaches/{id}")).await;
    let log = d["deliveries"].as_array().unwrap();
    let in_app: Vec<&Value> = log.iter().filter(|r| r["channel"] == "in-app").collect();
    assert_eq!(in_app.len(), 2);
    assert!(in_app.iter().any(|r| r["recipient"] == mw_mock_jmap::USER && r["status"] == "sent"));
    assert!(in_app.iter().any(|r| r["recipient"] == "nobody@example.test" && r["status"] == "skipped"));
    assert!(log.iter().any(|r| r["channel"] == "inbox"), "the inbox channel is attempted and logged");
    assert_eq!(d["incident"]["body"]["subjectNotices"].as_array().unwrap().len(), 1);

    let r = f.user.get(format!("{}/api/privacy/breach-notices", f.base)).send().await.unwrap();
    let notices: Value = r.json().await.unwrap();
    assert_eq!(notices["unread"], 1, "{notices}");
    assert!(notices["notices"][0]["subject"].as_str().unwrap().contains("Contacts exposed"));
    let r = f.user.post(format!("{}/api/privacy/breach-notices/ack", f.base)).json(&json!({"all": true})).send().await.unwrap();
    assert_eq!(r.json::<Value>().await.unwrap()["updated"], 1);
    let notices: Value = f.user.get(format!("{}/api/privacy/breach-notices", f.base)).send().await.unwrap().json().await.unwrap();
    assert_eq!(notices["unread"], 0);

    // The external address was skipped: a job record alone proves nothing.
    let (code, v) = f.post(&format!("/breaches/{id}/status"), json!({"status": "notified-subjects"})).await;
    assert_eq!(code, 409, "{v}");
    let (_, v) = f.get(&format!("/breaches/{id}/authority-notice")).await;
    assert_eq!(v["notice"]["dataSubjectsInformed"], false);
    let notice = d["incident"]["body"]["subjectNotices"][0]["id"].as_str().unwrap();
    for status in ["queued", "uncertain", "accepted"] {
        f.store.breach_delivery_add(&mw_store::breach::BreachDelivery {
            id: uuid::Uuid::new_v4().to_string(), incident_id: id.clone(), notice_id: notice.into(),
            recipient: "nobody@example.test".into(), channel: "email".into(), status: status.into(), detail: String::new(), ts: now(),
        }).await.unwrap();
        let (_, v) = f.get(&format!("/breaches/{id}/authority-notice")).await;
        assert_eq!(v["notice"]["dataSubjectsInformed"], status == "accepted", "{v}");
    }
    let (code, v) = f.post(&format!("/breaches/{id}/status"), json!({"status": "notified-subjects"})).await;
    assert_eq!(code, 200, "{v}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn automation_rules_open_one_draft_per_burst_and_start_no_clock() {
    let f = fixture().await;
    set(&f.store, "breach.auto.enabled", json!(true)).await;
    set(&f.store, "breach.auto.credentialStuffing.threshold", json!(2)).await;
    set(&f.store, "breach.auto.assignee", json!("dpo")).await;
    // The first pass only sets the cursor.
    f.post("/breaches/run", json!({})).await;
    let at = chrono::Utc::now() + chrono::Duration::seconds(5);
    for (n, action) in ["signin-spray-source", "signin-spray-source", "login-succeeded"].iter().enumerate() {
        f.store
            .append_audit(&mw_store::AuditRow {
                id: format!("auto-{n}"),
                ts: (at + chrono::Duration::seconds(n as i64)).to_rfc3339(),
                actor: "system".into(),
                actor_kind: "system".into(),
                action: (*action).into(),
                target: None,
                detail_json: "{}".into(),
                ip: None,
            })
            .await
            .unwrap();
    }
    f.post("/breaches/run", json!({})).await;
    let (_, list) = f.get("/breaches?status=draft").await;
    let drafts = list["incidents"].as_array().unwrap();
    assert_eq!(drafts.len(), 1, "{list}");
    assert_eq!(drafts[0]["source"], "auto");
    assert_eq!(drafts[0]["assignee"], "dpo");
    assert_eq!(drafts[0]["clock"]["phase"], "not-started");
    let id = drafts[0]["id"].as_str().unwrap();
    assert!(f.trail(id).await.contains(&"created".to_string()));
    // A rule at threshold 0 stays off.
    set(&f.store, "breach.auto.theft.threshold", json!(0)).await;
    f.store
        .append_audit(&mw_store::AuditRow {
            id: "auto-theft".into(),
            ts: (at + chrono::Duration::seconds(30)).to_rfc3339(),
            actor: "u".into(),
            actor_kind: "user".into(),
            action: "session-security-alert".into(),
            target: None,
            detail_json: json!({"kind": "impossibleTravel"}).to_string(),
            ip: None,
        })
        .await
        .unwrap();
    f.post("/breaches/run", json!({})).await;
    let (_, list) = f.get("/breaches?status=draft").await;
    assert_eq!(list["incidents"].as_array().unwrap().len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_lockdown_playbook_previews_before_it_ends_sessions() {
    let f = fixture().await;
    let i = f.open(json!({"title": "Account takeover", "incidentType": "credential-compromise", "affected": [mw_mock_jmap::USER]})).await;
    let id = i["id"].as_str().unwrap().to_string();
    assert!(i["playbook"].as_array().unwrap().iter().any(|p| p["id"] == "revoke-sessions"));
    assert_eq!(i["body"]["affected"][0]["accountId"], f.account_id);

    let (code, v) = f.post(&format!("/breaches/{id}/lockdown"), json!({"action": "revoke-sessions", "dryRun": true})).await;
    assert_eq!(code, 200, "{v}");
    assert_eq!(v["results"][0]["status"], "would-apply");
    assert!(v["results"][0]["sessions"].as_i64().unwrap() >= 1);
    assert!(f.user.get(format!("{}/api/me", f.base)).send().await.unwrap().status().is_success(), "a dry run changes nothing");

    let (code, v) = f.post(&format!("/breaches/{id}/lockdown"), json!({"action": "revoke-sessions", "dryRun": false})).await;
    assert_eq!(code, 422, "{v}");
    assert_eq!(v["code"], "confirmRequired");
    let (code, v) = f.post(&format!("/breaches/{id}/lockdown"), json!({"action": "revoke-sessions", "dryRun": false, "confirm": true})).await;
    assert_eq!(code, 200, "{v}");
    assert_eq!(v["results"][0]["status"], "applied");
    assert_eq!(f.user.get(format!("{}/api/me", f.base)).send().await.unwrap().status().as_u16(), 401);
    let done = v["incident"]["playbook"].as_array().unwrap().iter().find(|p| p["id"] == "revoke-sessions").unwrap().clone();
    assert_eq!(done["done"], true);
    assert!(f.trail(&id).await.contains(&"lockdown".to_string()));

    let (code, v) = f.post(&format!("/breaches/{id}/lockdown"), json!({"action": "force-password-reset", "dryRun": false, "confirm": true})).await;
    assert_eq!(code, 200, "{v}");
    assert_eq!(v["results"][0]["status"], "applied");
    let (code, _) = f.post(&format!("/breaches/{id}/playbook"), json!({"item": "not-a-step", "done": true})).await;
    assert_eq!(code, 422);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn timeline_and_evidence_validate_their_input() {
    let f = fixture().await;
    let i = f.open(json!({"title": "Evidence"})).await;
    let id = i["id"].as_str().unwrap();
    let (code, v) = f.post(&format!("/breaches/{id}/evidence"), json!({"kind": "link", "label": "ticket", "value": "javascript:alert(1)"})).await;
    assert_eq!(code, 422, "{v}");
    let (code, _) = f.post(&format!("/breaches/{id}/evidence"), json!({"kind": "link", "label": "ticket", "value": "https://tracker.example.test/1"})).await;
    assert_eq!(code, 200);
    let (code, v) = f.post(&format!("/breaches/{id}/timeline"), json!({"kind": "containment", "text": "Sessions revoked at 10:15."})).await;
    assert_eq!(code, 200, "{v}");
    assert_eq!(v["incident"]["body"]["timeline"].as_array().unwrap().len(), 2);
    let (code, _) = f.post(&format!("/breaches/{id}/timeline"), json!({"kind": "gossip", "text": "x"})).await;
    assert_eq!(code, 422);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn switching_the_register_off_refuses_new_incidents() {
    let f = fixture().await;
    set(&f.store, "breach.enabled", json!(false)).await;
    let (code, v) = f.post("/breaches", json!({"title": "No"})).await;
    assert_eq!(code, 409, "{v}");
    assert_eq!(v["code"], "disabled");
}
