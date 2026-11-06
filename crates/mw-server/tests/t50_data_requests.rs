//! Data subject requests end to end: filing, step-up, confirmation, approval, the
//! export download (signed, single use), erasure with a grace period, a cancel,
//! a legal hold, an extension, the admin queue and its CSV, and the rate limit.
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
    // These tests drive approvals directly; the step-up proof has its own test below.
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

#[tokio::test]
async fn export_download_is_signed_single_use_and_holds_the_persons_data() {
    let f = fixture().await;
    set(&f.store, "dsr.confirmation.required", json!(false)).await;
    // Something to export: one address book with one contact.
    let native = f.native_account().await;
    f.store
        .upsert_address_book(&mw_store::AddressBookRow {
            id: "book-1".into(),
            account_id: native.clone(),
            name: "Personal".into(),
            is_default: true,
            carddav_url: None,
            sync_token: None,
            ctag: None,
        })
        .await
        .unwrap();
    f.store
        .upsert_contact(&mw_store::ContactRow {
            id: "c-1".into(),
            address_book_id: "book-1".into(),
            uid: "uid-1".into(),
            etag: None,
            vcard_raw: "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Zebulon Quux\r\nEND:VCARD\r\n".into(),
            json: None,
            full_name: "Zebulon Quux".into(),
            is_favorite: false,
            photo_blob_id: None,
            pgp_key: None,
            smime_cert: None,
        })
        .await
        .unwrap();

    let overview: Value = f.user.get(format!("{}/api/privacy/requests/overview", f.base)).send().await.unwrap().json().await.unwrap();
    assert_eq!(overview["policy"]["enabled"], true);
    assert_eq!(overview["policy"]["graceDays"], 14);

    // Filing an export needs a fresh proof of the password.
    let (status, body) = f.file(json!({"kind":"access"})).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "step_up_required");
    f.step_up().await;
    let (status, body) = f.file(json!({"kind":"access"})).await;
    assert_eq!(status, 201, "{body}");
    let id = body["request"]["id"].as_str().unwrap().to_string();
    assert_eq!(body["request"]["status"], "queued", "approval is automatic for exports");
    // A second open request of the same type is refused.
    assert_eq!(f.file(json!({"kind":"access"})).await.0, 409);

    // Run the worker's step now; the export becomes ready.
    let (status, body) = f.admin_post(&format!("/{id}/run"), json!({})).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["request"]["status"], "ready", "{body}");

    // No link, no download.
    let r = f.user.get(format!("{}/api/privacy/requests/{id}/download?token=nope", f.base)).send().await.unwrap();
    assert_eq!(r.status(), 403);
    let (status, link) = f.user_post(&format!("/{id}/download-link"), json!({})).await;
    assert_eq!(status, 200, "{link}");
    let url = link["url"].as_str().unwrap().to_string();
    let r = f.user.get(format!("{}{url}", f.base)).send().await.unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(r.headers()["content-type"], "application/zip");
    assert_eq!(r.headers()["cache-control"], "no-store");
    let zip = r.bytes().await.unwrap();
    assert_eq!(&zip[..2], b"PK");
    let text = String::from_utf8_lossy(&zip);
    for name in ["manifest.json", "profile.json", "contacts.vcf", "data-requests.json"] {
        assert!(text.contains(name), "{name} is in the archive");
    }
    assert!(text.contains("Zebulon Quux"), "the contact is exported");
    // The link works once.
    let r = f.user.get(format!("{}{url}", f.base)).send().await.unwrap();
    assert_eq!(r.status(), 410);
    // Another person's session cannot use it: no cookie, no access.
    let anon = reqwest::Client::new();
    assert_eq!(anon.get(format!("{}{url}", f.base)).send().await.unwrap().status().as_u16(), 401);

    let (_, one) = (0, f.user.get(format!("{}/api/privacy/requests/{id}", f.base)).send().await.unwrap().json::<Value>().await.unwrap());
    assert_eq!(one["request"]["status"], "downloaded");
    let kinds: Vec<&str> = one["trail"].as_array().unwrap().iter().map(|e| e["kind"].as_str().unwrap()).collect();
    for k in ["submitted", "approved", "export-ready", "download-link-created", "downloaded"] {
        assert!(kinds.contains(&k), "{k} in {kinds:?}");
    }
}

#[tokio::test]
async fn confirmation_code_gates_the_request_and_an_admin_can_verify_instead() {
    let f = fixture().await;
    let (status, body) = f.file(json!({"kind":"rectification","details":"my name is misspelt"})).await;
    assert_eq!(status, 201, "{body}");
    let id = body["request"]["id"].as_str().unwrap().to_string();
    assert_eq!(body["request"]["status"], "awaiting_confirmation");
    assert_eq!(body["request"]["confirmationSent"], true);
    let (status, body) = f.user_post(&format!("/{id}/confirm"), json!({"code":"WRONG123"})).await;
    assert_eq!((status, body["code"].as_str()), (403, Some("wrongCode")));
    // The administrator verifies the person another way; a note is required.
    assert_eq!(f.admin_post(&format!("/{id}/verify"), json!({"note":""})).await.0, 422);
    let (status, body) = f.admin_post(&format!("/{id}/verify"), json!({"note":"checked by phone"})).await;
    assert_eq!(status, 200, "{body}");
    // Rectification needs an administrator's approval, then is handled by hand.
    assert_eq!(body["request"]["status"], "awaiting_approval");
    let (_, body) = f.admin_post(&format!("/{id}/approve"), json!({})).await;
    assert_eq!(body["request"]["status"], "in_progress");
    assert_eq!(f.admin_post(&format!("/{id}/complete"), json!({"note":""})).await.0, 422);
    let (_, body) = f.admin_post(&format!("/{id}/complete"), json!({"note":"name corrected"})).await;
    assert_eq!(body["request"]["status"], "completed");
    assert_eq!(body["request"]["completion"]["outcome"], "name corrected");
}

#[tokio::test]
async fn erasure_waits_out_the_grace_period_can_be_cancelled_is_gated_by_a_hold_and_then_runs() {
    let f = fixture().await;
    set(&f.store, "dsr.confirmation.required", json!(false)).await;
    f.step_up().await;
    let (status, body) = f.file(json!({"kind":"erasure"})).await;
    assert_eq!(status, 201, "{body}");
    let id = body["request"]["id"].as_str().unwrap().to_string();
    assert_eq!(body["request"]["status"], "awaiting_approval", "erasure needs an administrator by default");
    let (_, body) = f.admin_post(&format!("/{id}/approve"), json!({})).await;
    assert_eq!(body["request"]["status"], "scheduled");
    assert!(body["request"]["graceUntil"].as_i64().unwrap() > chrono::Utc::now().timestamp() + 13 * 86_400);
    // Within the grace period the person cancels.
    let (status, body) = f.user_post(&format!("/{id}/cancel"), json!({})).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["request"]["status"], "cancelled");

    // A second erasure, now with the address under a legal hold.
    let address = mw_mock_jmap::USER;
    set(&f.store, "dsr.legalHold.subjects", json!([address])).await;
    f.step_up().await;
    let (_, body) = f.file(json!({"kind":"erasure"})).await;
    let id = body["request"]["id"].as_str().unwrap().to_string();
    let (_, body) = f.admin_post(&format!("/{id}/approve"), json!({})).await;
    assert_eq!(body["request"]["status"], "blocked", "{body}");
    assert_eq!(body["request"]["legalHold"]["active"], true);
    assert_eq!(body["request"]["legalHold"]["source"], "policy");
    // Running it does not erase anything while the hold stands.
    let (_, body) = f.admin_post(&format!("/{id}/run"), json!({})).await;
    assert_eq!(body["request"]["status"], "blocked");
    // Approving it is refused while the hold applies.
    assert_eq!(f.admin_post(&format!("/{id}/approve"), json!({})).await.0, 409);

    // Remove the hold: the next pass schedules it; run now skips the wait.
    set(&f.store, "dsr.legalHold.subjects", json!([])).await;
    let native = f.native_account().await;
    f.store
        .upsert_address_book(&mw_store::AddressBookRow {
            id: "book-9".into(),
            account_id: native.clone(),
            name: "Erase me".into(),
            is_default: false,
            carddav_url: None,
            sync_token: None,
            ctag: None,
        })
        .await
        .unwrap();
    let (_, body) = f.admin_post(&format!("/{id}/run"), json!({})).await;
    assert_eq!(body["request"]["status"], "scheduled", "{body}");
    let (_, body) = f.admin_post(&format!("/{id}/run"), json!({})).await;
    assert_eq!(body["request"]["status"], "completed", "{body}");
    let completion = &body["request"]["completion"];
    assert_eq!(completion["verified"], true);
    assert_eq!(completion["remainingAfterVerification"], 0);
    assert!(completion["deleted"]["address_books"].as_u64().unwrap() >= 1, "{completion}");
    assert!(completion["digest"].as_str().unwrap().len() >= 32);
    assert!(completion["retained"].as_array().unwrap().len() >= 3);
    // The data and the session are gone.
    assert!(f.store.list_address_books(&native).await.unwrap().is_empty());
    assert!(completion["deleted"]["accounts"].as_u64().unwrap() >= 1, "{completion}");
    let r = f.user.get(format!("{}/api/privacy/requests", f.base)).send().await.unwrap();
    assert_eq!(r.status(), 401, "the session was revoked");
    let detail = f.admin_get(&format!("/{id}")).await;
    let kinds: Vec<&str> = detail["trail"].as_array().unwrap().iter().map(|e| e["kind"].as_str().unwrap()).collect();
    for k in ["blocked-by-hold", "hold-cleared", "grace-skipped", "erasure-completed"] {
        assert!(kinds.contains(&k), "{k} in {kinds:?}");
    }
}

#[tokio::test]
async fn queue_extension_csv_rate_limit_and_switches() {
    let f = fixture().await;
    set(&f.store, "dsr.confirmation.required", json!(false)).await;
    set(&f.store, "dsr.reauth.required", json!(false)).await;
    set(&f.store, "dsr.rateLimit.perDay", json!(2)).await;
    let (_, a) = f.file(json!({"kind":"objection","details":"stop profiling"})).await;
    let id = a["request"]["id"].as_str().unwrap().to_string();
    let (_, b) = f.file(json!({"kind":"restriction"})).await;
    assert_eq!(b["request"]["status"], "awaiting_approval");
    assert_eq!(f.file(json!({"kind":"other"})).await.0, 429, "the daily limit applies");

    // Extension: bounded by dsr.sla.maxExtensionDays (60), reason required.
    let due = a["request"]["dueAt"].as_i64().unwrap();
    assert_eq!(f.admin_post(&format!("/{id}/extend"), json!({"days":61,"reason":"complex"})).await.0, 422);
    assert_eq!(f.admin_post(&format!("/{id}/extend"), json!({"days":30,"reason":""})).await.0, 422);
    let (status, body) = f.admin_post(&format!("/{id}/extend"), json!({"days":30,"reason":"complex request"})).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["request"]["dueAt"].as_i64().unwrap(), due + 30 * 86_400);
    assert_eq!(f.admin_post(&format!("/{id}/extend"), json!({"days":31,"reason":"more"})).await.0, 422, "30 + 31 exceeds 60");

    // Rejecting needs a reason and tells the person.
    assert_eq!(f.admin_post(&format!("/{id}/reject"), json!({"reason":" "})).await.0, 422);
    let (_, body) = f.admin_post(&format!("/{id}/reject"), json!({"reason":"not applicable"})).await;
    assert_eq!(body["request"]["status"], "rejected");
    let mine: Value = f.user.get(format!("{}/api/privacy/requests", f.base)).send().await.unwrap().json().await.unwrap();
    let rejected = mine["requests"].as_array().unwrap().iter().find(|r| r["id"] == id.as_str()).unwrap();
    assert_eq!(rejected["decisionReason"], "not applicable");

    // The queue: filters, stats, assignment and the CSV log.
    let list = f.admin_get("?status=open").await;
    assert_eq!(list["matched"], 1);
    assert_eq!(list["stats"]["total"], 2);
    assert_eq!(list["stats"]["open"], 1);
    let rid = list["requests"][0]["id"].as_str().unwrap().to_string();
    let (_, body) = f.admin_post(&format!("/{rid}/assign"), json!({"assignee":"dpo"})).await;
    assert_eq!(body["request"]["assignee"], "dpo");
    assert_eq!(f.admin_get("?assignee=none").await["matched"], 1, "the rejected one has no assignee");
    assert_eq!(f.admin_get("?assignee=dpo").await["matched"], 1);
    let csv = f.admin.get(format!("{}/admin/dsr/export?status=", f.base)).send().await.unwrap();
    assert_eq!(csv.status(), 200);
    assert!(csv.headers()["content-type"].to_str().unwrap().starts_with("text/csv"));
    let csv = csv.text().await.unwrap();
    assert!(csv.starts_with("id,address,domain,kind,status"));
    assert!(csv.contains("objection") && csv.contains("restriction"));

    // On behalf: allowed by default; a switched-off type is refused for everyone.
    set(&f.store, "dsr.type.other", json!(false)).await;
    assert_eq!(f.file(json!({"kind":"other"})).await.0, 403);
    set(&f.store, "dsr.onBehalf.allowed", json!(false)).await;
    let (status, body) = f.admin_post("", json!({"address":mw_mock_jmap::USER,"kind":"access"})).await;
    assert_eq!((status, body["code"].as_str()), (403, Some("onBehalfDisabled")));
    set(&f.store, "dsr.onBehalf.allowed", json!(true)).await;
    set(&f.store, "dsr.enabled", json!(false)).await;
    assert_eq!(f.file(json!({"kind":"objection"})).await.0, 403, "the whole feature can be switched off");
}

#[tokio::test]
async fn approving_needs_a_fresh_admin_proof_when_dsr_admin_step_up_is_on() {
    let f = fixture().await;
    set(&f.store, "dsr.admin.stepUp", json!(true)).await;
    set(&f.store, "dsr.confirmation.required", json!(false)).await;
    set(&f.store, "dsr.reauth.required", json!(false)).await;
    let (_, filed) = f.file(json!({"kind":"restriction"})).await;
    let id = filed["request"]["id"].as_str().unwrap().to_string();
    assert_eq!(filed["request"]["status"], "awaiting_approval");

    let (status, body) = f.admin_post(&format!("/{id}/approve"), json!({"note":"ok"})).await;
    assert_eq!((status, body["code"].as_str(), body["action"].as_str()), (403, Some("step_up_required"), Some("dsrDecision")));
    // Rejecting is not a decision about the data and needs no proof.
    // Bulk approve is refused whole, before anything changes.
    let (status, body) = f.admin_post("/bulk", json!({"action":"approve","ids":[id]})).await;
    assert_eq!((status, body["code"].as_str()), (403, Some("step_up_required")));
    let still = f.admin_get(&format!("/{id}")).await;
    assert_eq!(still["request"]["status"], "awaiting_approval");

    auth::admin_step_up(&f.admin, &format!("{}/admin", f.base), "hunter2", &[]).await;
    let (status, body) = f.admin_post(&format!("/{id}/approve"), json!({"note":"ok"})).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["request"]["status"], "in_progress");
}

#[tokio::test]
async fn bulk_actions_report_each_row_and_the_queue_pages_by_cursor() {
    let f = fixture().await;
    set(&f.store, "dsr.confirmation.required", json!(false)).await;
    set(&f.store, "dsr.reauth.required", json!(false)).await;
    set(&f.store, "dsr.rateLimit.perDay", json!(50)).await;
    // Admin-filed requests for three kinds of one person (a person has one open request per kind).
    let mut ids = Vec::new();
    for kind in ["objection", "restriction", "rectification"] {
        let (status, body) = f.admin_post("", json!({"address":mw_mock_jmap::USER,"kind":kind})).await;
        assert_eq!(status, 201, "{body}");
        ids.push(body["request"]["id"].as_str().unwrap().to_string());
    }

    // Assign two at once; an unknown id is reported on its own row and does not stop the rest.
    let (status, body) = f.admin_post("/bulk", json!({"action":"assign","ids":[ids[0], ids[1], "no-such-request"],"assignee":"dpo"})).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!((body["done"].as_i64(), body["failed"].as_i64()), (Some(2), Some(1)));
    let results = body["results"].as_array().unwrap();
    assert_eq!(results[2]["ok"], false);
    assert_eq!(f.admin_get("?assignee=dpo").await["matched"], 2);

    // A bulk extend needs a reason like a single one, and a rejection of one row leaves the others open.
    let (_, body) = f.admin_post("/bulk", json!({"action":"extend","ids":[ids[0], ids[2]],"days":10,"reason":""})).await;
    assert_eq!(body["failed"], 2, "{body}");
    let (_, body) = f.admin_post("/bulk", json!({"action":"extend","ids":[ids[0], ids[2]],"days":10,"reason":"complex"})).await;
    assert_eq!(body["done"], 2, "{body}");
    let (_, body) = f.admin_post("/bulk", json!({"action":"hold","ids":[ids[1]],"reason":"litigation"})).await;
    assert_eq!(body["done"], 1, "{body}");
    assert_eq!(f.admin_get(&format!("/{}", ids[1])).await["request"]["legalHold"]["active"], true);
    let (_, body) = f.admin_post("/bulk", json!({"action":"reject","ids":[ids[2]],"reason":"duplicate"})).await;
    assert_eq!((body["done"].as_i64(), body["results"][0]["status"].as_str()), (Some(1), Some("rejected")));
    let (status, _) = f.admin_post("/bulk", json!({"action":"explode","ids":[ids[0]]})).await;
    assert_eq!(status, 422);
    let (status, _) = f.admin_post("/bulk", json!({"action":"approve","ids":[]})).await;
    assert_eq!(status, 422);

    // Cursor paging: one row per page, newest first, every row once, no ceiling on the queue.
    let mut seen = Vec::new();
    let mut cursor = String::new();
    for _ in 0..5 {
        let page = f.admin_get(&format!("?limit=1&cursor={cursor}")).await;
        assert_eq!(page["matched"], 3);
        for r in page["requests"].as_array().unwrap() {
            seen.push(r["id"].as_str().unwrap().to_string());
        }
        match page["nextCursor"].as_str() {
            Some(next) => cursor = next.to_string(),
            None => break,
        }
    }
    seen.sort();
    let mut expected = ids.clone();
    expected.sort();
    assert_eq!(seen, expected);

    // The CSV of a selection holds only the selected rows.
    let csv = f.admin.get(format!("{}/admin/dsr/export?ids={}", f.base, ids[0])).send().await.unwrap().text().await.unwrap();
    assert!(csv.contains(&ids[0]) && !csv.contains(&ids[1]), "{csv}");
}

#[tokio::test]
async fn an_approved_objection_withdraws_consent_for_the_purposes_it_names() {
    let f = fixture().await;
    set(&f.store, "dsr.confirmation.required", json!(false)).await;
    set(&f.store, "dsr.reauth.required", json!(false)).await;
    let (status, filed) = f.file(json!({"kind":"objection","details":"no tracking","purposes":["openTracking","assistant","bogus"]})).await;
    assert_eq!(status, 201, "{filed}");
    assert_eq!(filed["request"]["purposes"], json!(["openTracking", "assistant"]), "unknown purposes are dropped");
    let id = filed["request"]["id"].as_str().unwrap().to_string();
    let (status, body) = f.admin_post(&format!("/{id}/approve"), json!({"note":"ok"})).await;
    assert_eq!(status, 200, "{body}");
    let trail = f.admin_get(&format!("/{id}")).await;
    let applied = trail["trail"].as_array().unwrap().iter().find(|e| e["kind"] == "objection-applied").expect("the trail records the change");
    assert_eq!(applied["detail"]["withdrawn"], json!(["openTracking", "assistant"]));
    let consents: Value = f.user.get(format!("{}/api/privacy/consents", f.base)).send().await.unwrap().json().await.unwrap();
    let state_of = |p: &str| consents["purposes"].as_array().unwrap().iter().find(|x| x["id"] == p).map(|x| x["state"].clone());
    assert_eq!(state_of("openTracking"), Some(json!("withdrawn")));
    assert_eq!(state_of("assistant"), Some(json!("withdrawn")));
    let receipts = consents["receipts"].as_array().unwrap();
    assert!(receipts.iter().any(|r| r["purpose"] == "openTracking" && r["source"] == "dsr"), "{receipts:?}");
}
