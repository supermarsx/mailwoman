//! Art.18 restriction, the consent ledger and the Your data overview end to end:
//! an approved restriction request restricts the person, the choke points answer
//! 403 while it lasts, an administrator lifts it, consent decisions are receipts
//! that gate the feature at runtime, and the overview reports counts and status.

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::settings_policy::SettingPolicyRow;
use serde_json::{Value, json};
use std::net::SocketAddr;

mod common;
use common::auth;

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

async fn serve(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
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
}

async fn fixture() -> Fixture {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("t50-data-requests");
    let store = mw_store::Store::open(db.to_str().unwrap(), mw_store::ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    set(&store, "signin.retryCooldownSecs", json!(0)).await;
    // The approval below is made directly; the step-up proof is covered by t50_data_requests.
    set(&store, "dsr.admin.stepUp", json!(false)).await;
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
    tokio::time::sleep(std::time::Duration::from_millis(3100)).await;
    let admin = reqwest::Client::builder().no_proxy().cookie_store(true).build().unwrap();
    let r = admin
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":"root","password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    Fixture {
        base,
        store,
        user,
        admin,
    }
}

impl Fixture {
    /// A native account bound to the person's address: the local mail cache and
    /// PIM rows hang off an account row.
    async fn native_account(&self) -> String {
        let id = self
            .store
            .create_account(
                &mw_store::NewAccount {
                    kind: mw_store::AccountKind::Imap,
                    host: "imap.example.test",
                    port: 993,
                    tls: "tls",
                    username: mw_mock_jmap::USER,
                    sync_policy_json: "{}",
                },
                &mw_store::Credentials { username: mw_mock_jmap::USER.into(), password: "x".into() },
            )
            .await
            .unwrap();
        let (local, domain) = mw_mock_jmap::USER.split_once('@').unwrap();
        self.store
            .register_successful_signin(mw_store::user_directory::SuccessfulSignin {
                account_id: &id,
                address: mw_mock_jmap::USER,
                domain,
                local_part: local,
                mode: "engine",
                now: chrono::Utc::now().timestamp(),
            })
            .await
            .unwrap();
        id
    }
    async fn file(&self, body: Value) -> (u16, Value) {
        let r = self.user.post(format!("{}/api/privacy/requests", self.base)).json(&body).send().await.unwrap();
        (r.status().as_u16(), r.json().await.unwrap_or(Value::Null))
    }
    async fn user_post(&self, path: &str, body: Value) -> (u16, Value) {
        let r = self.user.post(format!("{}/api/privacy/requests{path}", self.base)).json(&body).send().await.unwrap();
        (r.status().as_u16(), r.json().await.unwrap_or(Value::Null))
    }
    async fn admin_post(&self, path: &str, body: Value) -> (u16, Value) {
        let r = self.admin.post(format!("{}/admin/dsr{path}", self.base)).json(&body).send().await.unwrap();
        (r.status().as_u16(), r.json().await.unwrap_or(Value::Null))
    }
    async fn admin_get(&self, path: &str) -> Value {
        let r = self.admin.get(format!("{}/admin/dsr{path}", self.base)).send().await.unwrap();
        assert_eq!(r.status(), 200, "{path}");
        r.json().await.unwrap()
    }
    async fn step_up(&self) {
        auth::user_step_up(&self.user, &self.base, mw_mock_jmap::PASS).await;
    }
}


impl Fixture {
    async fn user_get(&self, path: &str) -> (u16, Value) {
        let r = self.user.get(format!("{}{path}", self.base)).send().await.unwrap();
        (r.status().as_u16(), r.json().await.unwrap_or(Value::Null))
    }
    async fn user_post_raw(&self, path: &str, body: Value) -> (u16, Value) {
        let r = self.user.post(format!("{}{path}", self.base)).json(&body).send().await.unwrap();
        (r.status().as_u16(), r.json().await.unwrap_or(Value::Null))
    }
    async fn admin_raw_post(&self, path: &str, body: Value) -> (u16, Value) {
        let r = self.admin.post(format!("{}{path}", self.base)).json(&body).send().await.unwrap();
        (r.status().as_u16(), r.json().await.unwrap_or(Value::Null))
    }
    async fn admin_raw_get(&self, path: &str) -> (u16, Value) {
        let r = self.admin.get(format!("{}{path}", self.base)).send().await.unwrap();
        (r.status().as_u16(), r.json().await.unwrap_or(Value::Null))
    }
}

#[tokio::test]
async fn an_approved_restriction_request_restricts_and_the_assistant_route_refuses() {
    let f = fixture().await;
    f.native_account().await;
    set(&f.store, "dsr.confirmation.required", json!(false)).await;
    set(&f.store, "cap.ai_assistant", json!(true)).await;

    // Before: the assistant feature gate passes (the route may still be unconfigured,
    // but it is not refused for restriction).
    let (_, before) = f.user_get("/api/assist/config").await;
    assert_ne!(before["code"], "processing_restricted", "{before}");
    assert_eq!(f.user_get("/api/privacy/restriction").await.1["active"], false);

    let (status, body) = f.file(json!({"kind":"restriction","details":"I contest the accuracy"})).await;
    assert_eq!(status, 201, "{body}");
    let id = body["request"]["id"].as_str().unwrap().to_string();
    let (status, body) = f.admin_post(&format!("/{id}/approve"), json!({"note":"ok"})).await;
    assert_eq!(status, 200, "{body}");

    // Applied straight away (restriction.autoApplyOnApproval defaults on).
    let (_, now) = f.user_get("/api/privacy/restriction").await;
    assert_eq!(now["active"], true, "{now}");
    assert_eq!(now["source"], "dsr");
    let scopes: Vec<&str> = now["scopes"].as_array().unwrap().iter().filter_map(Value::as_str).collect();
    assert!(scopes.contains(&"assistant") && scopes.contains(&"push"), "{scopes:?}");

    // The choke point answers.
    let (status, refused) = f.user_get("/api/assist/config").await;
    assert_eq!(status, 403, "{refused}");
    assert_eq!(refused["code"], "processing_restricted");
    assert_eq!(refused["scope"], "assistant");

    // The request's trail records the application.
    let detail = f.admin_get(&format!("/{id}")).await;
    assert!(detail.to_string().contains("restriction-applied"), "{detail}");

    // An administrator lifts it with a reason; the route works again.
    let (status, _) = f.admin_raw_post("/admin/restrictions/lift", json!({"address": mw_mock_jmap::USER, "reason": "settled"})).await;
    assert_eq!(status, 200);
    assert_eq!(f.user_get("/api/privacy/restriction").await.1["active"], false);
    let (_, after) = f.user_get("/api/assist/config").await;
    assert_ne!(after["code"], "processing_restricted", "{after}");
    let (_, list) = f.admin_raw_get("/admin/restrictions?all=true").await;
    assert_eq!(list["restrictions"][0]["liftReason"], "settled", "{list}");
}

#[tokio::test]
async fn auto_apply_off_leaves_the_request_for_the_administrator_to_apply() {
    let f = fixture().await;
    f.native_account().await;
    set(&f.store, "dsr.confirmation.required", json!(false)).await;
    set(&f.store, "restriction.autoApplyOnApproval", json!(false)).await;
    let (_, body) = f.file(json!({"kind":"restriction"})).await;
    let id = body["request"]["id"].as_str().unwrap().to_string();
    f.admin_post(&format!("/{id}/approve"), json!({"note":""})).await;
    assert_eq!(f.user_get("/api/privacy/restriction").await.1["active"], false);

    // Admin applies it by hand, with chosen scopes and a reason.
    let (status, body) = f.admin_raw_post("/admin/restrictions", json!({"address": mw_mock_jmap::USER, "scopes": ["indexing"], "reason": "manual"})).await;
    assert_eq!(status, 201, "{body}");
    let (_, now) = f.user_get("/api/privacy/restriction").await;
    assert_eq!(now["scopes"], json!(["indexing"]));
    // A reason is required, and unknown scopes are refused.
    assert_eq!(f.admin_raw_post("/admin/restrictions", json!({"address": mw_mock_jmap::USER, "scopes": ["indexing"], "reason": " "})).await.0, 422);
    assert_eq!(f.admin_raw_post("/admin/restrictions", json!({"address": mw_mock_jmap::USER, "scopes": ["bogus"], "reason": "x"})).await.0, 422);
}

#[tokio::test]
async fn restrictions_can_be_switched_off() {
    let f = fixture().await;
    f.native_account().await;
    set(&f.store, "restriction.enabled", json!(false)).await;
    let (status, body) = f.admin_raw_post("/admin/restrictions", json!({"address": mw_mock_jmap::USER, "reason": "x"})).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "restrictionDisabled");
}

#[tokio::test]
async fn a_consent_based_assistant_waits_for_a_yes_and_stops_on_withdrawal() {
    let f = fixture().await;
    f.native_account().await;
    set(&f.store, "cap.ai_assistant", json!(true)).await;
    set(&f.store, "consent.assistant.basis", json!("consent")).await;

    // Default off: the purpose is listed undecided, with its basis.
    let (status, view) = f.user_get("/api/privacy/consents").await;
    assert_eq!(status, 200, "{view}");
    let assistant = view["purposes"].as_array().unwrap().iter().find(|p| p["id"] == "assistant").unwrap();
    assert_eq!(assistant["state"], "unset");
    assert_eq!(assistant["basis"], "consent");
    assert_eq!(assistant["defaultOn"], false);

    // The runtime check is the ledger.
    let account = f.session_account().await;
    assert!(!mw_server_allowed(&f, &account, "assistant").await);
    let (status, view) = f.user_post_raw("/api/privacy/consents/assistant", json!({"decision":"granted"})).await;
    assert_eq!(status, 200, "{view}");
    assert!(mw_server_allowed(&f, &account, "assistant").await);

    // Withdrawal is one call and takes effect at once.
    let (status, _) = f.user_post_raw("/api/privacy/consents/assistant", json!({"decision":"withdrawn"})).await;
    assert_eq!(status, 200);
    assert!(!mw_server_allowed(&f, &account, "assistant").await);

    // Receipts: three rows, in order, with purpose, version and source.
    let (_, view) = f.user_get("/api/privacy/consents").await;
    let receipts = view["receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 2, "{view}");
    assert!(receipts.iter().all(|r| r["purpose"] == "assistant" && r["version"] == 1 && r["source"] == "settings"));
}

#[tokio::test]
async fn raising_the_text_version_asks_again() {
    let f = fixture().await;
    f.native_account().await;
    set(&f.store, "consent.push.basis", json!("consent")).await;
    let account = f.session_account().await;
    f.user_post_raw("/api/privacy/consents/push", json!({"decision":"granted"})).await;
    assert!(mw_server_allowed(&f, &account, "push").await);
    set(&f.store, "consent.push.version", json!(2)).await;
    assert!(!mw_server_allowed(&f, &account, "push").await, "version 1 consent no longer counts");
    let (_, view) = f.user_get("/api/privacy/consents").await;
    let push = view["purposes"].as_array().unwrap().iter().find(|p| p["id"] == "push").unwrap();
    assert_eq!(push["state"], "stale");
}

#[tokio::test]
async fn a_feature_opt_out_blocks_that_feature_even_when_the_basis_is_not_consent() {
    let f = fixture().await;
    f.native_account().await;
    let account = f.session_account().await;
    // Default basis for the assistant is not consent, so it is allowed...
    assert!(mw_server_allowed(&f, &account, "assistant.summarize").await);
    // ...until the person switches this one feature off.
    let (status, _) = f.user_post_raw("/api/privacy/consents/assistant.summarize", json!({"decision":"withdrawn"})).await;
    assert_eq!(status, 200);
    assert!(!mw_server_allowed(&f, &account, "assistant.summarize").await);
    assert!(mw_server_allowed(&f, &account, "assistant.draft").await, "other features are untouched");
    // Unknown purposes and decisions are refused; preference purposes are managed elsewhere.
    assert_eq!(f.user_post_raw("/api/privacy/consents/assistant.nope", json!({"decision":"granted"})).await.0, 404);
    assert_eq!(f.user_post_raw("/api/privacy/consents/assistant", json!({"decision":"maybe"})).await.0, 400);
    assert_eq!(f.user_post_raw("/api/privacy/consents/analytics", json!({"decision":"granted"})).await.0, 409);
}

#[tokio::test]
async fn the_overview_reports_categories_consents_and_restriction() {
    let f = fixture().await;
    f.native_account().await;
    let (status, o) = f.user_get("/api/privacy/overview").await;
    assert_eq!(status, 200, "{o}");
    let ids: Vec<&str> = o["categories"].as_array().unwrap().iter().filter_map(|c| c["id"].as_str()).collect();
    for want in ["mail", "contacts", "calendars", "notes", "devices", "signInHistory", "audit", "assistant", "push", "tracking", "consents", "requests"] {
        assert!(ids.contains(&want), "{want} in {ids:?}");
    }
    assert_eq!(o["restriction"]["active"], false);
    assert_eq!(o["consents"]["enabled"], true);
    assert_eq!(o["links"]["policy"], "/privacy");
    // Retention comes from the administrator's setting.
    set(&f.store, "retention.ipAddresses.days", json!(45)).await;
    let (_, o) = f.user_get("/api/privacy/overview").await;
    let sign_in = o["categories"].as_array().unwrap().iter().find(|c| c["id"] == "signInHistory").unwrap();
    assert_eq!(sign_in["retention"]["days"], 45);
    assert_eq!(sign_in["retention"]["source"], "setting");
}

#[tokio::test]
async fn the_admin_consent_overview_counts_current_decisions_and_exports_csv() {
    let f = fixture().await;
    f.native_account().await;
    f.user_post_raw("/api/privacy/consents/push", json!({"decision":"granted"})).await;
    f.user_post_raw("/api/privacy/consents/assistant", json!({"decision":"granted"})).await;
    f.user_post_raw("/api/privacy/consents/assistant", json!({"decision":"withdrawn"})).await;
    let (status, stats) = f.admin_raw_get("/admin/consents/stats").await;
    assert_eq!(status, 200, "{stats}");
    let row = |p: &str| stats["purposes"].as_array().unwrap().iter().find(|r| r["purpose"] == p).cloned().unwrap();
    assert_eq!(row("push")["granted"], 1);
    assert_eq!(row("assistant")["withdrawn"], 1);
    assert_eq!(row("assistant")["withdrawalRate"], 1.0);
    assert!(stats["series"].as_array().unwrap().iter().any(|d| d["granted"].as_i64().unwrap_or(0) > 0));
    let csv = f.admin.get(format!("{}/admin/consents/ledger.csv", f.base)).send().await.unwrap();
    assert_eq!(csv.status(), 200);
    let text = csv.text().await.unwrap();
    assert!(text.starts_with("time,domain,purpose,decision,version,basis,source,accountId"));
    assert_eq!(text.lines().count(), 4, "{text}");
}

impl Fixture {
    /// The account id the signed-in session acts as.
    async fn session_account(&self) -> String {
        let (_, o) = self.user_get("/api/account/sessions").await;
        let _ = o;
        let accounts = self.store.lifecycle_account_bindings(mw_mock_jmap::USER).await.unwrap();
        accounts.first().cloned().expect("a bound account")
    }
}

/// The runtime question the choke points ask, through the same function.
async fn mw_server_allowed(f: &Fixture, account: &str, purpose: &str) -> bool {
    mw_server::consent_allowed(&f.store, account, purpose).await
}
