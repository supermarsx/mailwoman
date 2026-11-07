//! t17-e-e2e — Identity.signatureName persistence (0020 column) over the REAL
//! prefs HTTP route.
//!
//! 26.16 accepted `signatureName` on an identity then dropped it (no column). 26.17
//! adds the 0020 `identities.signature_name` column and wires it through `v2.rs` +
//! `prefs_routes.rs`. This leg proves the WHOLE path: a real `POST
//! /api/account/identities` with `signatureName` set, behind a real session, then a
//! `GET` that returns the same value — persisted through the new column and the prefs
//! route (not just unit-tested in isolation).

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};
use sqlx::sqlite::SqliteConnectOptions;

use mw_server::{AppConfig, build_app};

mod common;
use common::test_db;

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

/// Origins of the mock upstreams this test binary has started. Proxy mode refuses a
/// loopback JMAP upstream unless the operator allowlist names it (t24 B2), so the
/// server under test lists these in `SecurityConfig::jmap_upstreams` — the
/// production control an operator uses for an internal upstream, not a test switch.
/// A test must start its mock before its server.
static MOCK_UPSTREAMS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// The mock upstreams started so far, as an allowlist.
fn mock_upstreams() -> Option<Vec<String>> {
    Some(MOCK_UPSTREAMS.lock().unwrap().clone())
}

// The mock this test started last. The server under test signs in to it as
// its configured upstream (`MW_UPSTREAM_DEFAULT`): a login can no longer name
// one (t32-h9). Per thread, so tests running in parallel keep their own.
thread_local! {
    static LAST_MOCK: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

fn last_mock() -> Option<String> {
    LAST_MOCK.with(|m| m.borrow().clone())
}

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    let origin = format!("http://{addr}");
    MOCK_UPSTREAMS.lock().unwrap().push(origin.clone());
    LAST_MOCK.with(|m| *m.borrow_mut() = Some(origin.clone()));
    origin
}

/// Returns (bound addr, db path). The db path lets the test seed the mock account row
/// the identity FK requires (sqlx enables `foreign_keys` by default).
async fn spawn_server() -> (SocketAddr, String) {
    let base = test_db::unique_dir("mw-t17-ident");
    let web = base.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let db_path = base.join("mw.db").to_string_lossy().into_owned();
    let config = AppConfig {
        db_path: db_path.clone(),
        server_key_hex: None,
        web_dir: Some(web as PathBuf),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig {
            jmap_upstreams: mock_upstreams(),
            upstream_default: last_mock(),
            ..mw_server::SecurityConfig::default()
        },
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (addr, db_path)
}

/// Seed the `accounts` row for the mock account so the `identities` FK is satisfied
/// (the login derives `account_id == mw_mock_jmap::ACCOUNT_ID`, but proxy-mode login
/// does not create an accounts row — a real deployment always has one).
async fn seed_mock_account(db_path: &str) {
    let opts = SqliteConnectOptions::new().filename(db_path);
    let pool = sqlx::SqlitePool::connect_with(opts).await.unwrap();
    sqlx::query(
        "INSERT OR IGNORE INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json)
         VALUES (?1, 'imap', 'h', 993, 'implicit', ?2, ?3, '{}')",
    )
    .bind(mw_mock_jmap::ACCOUNT_ID)
    .bind(mw_mock_jmap::USER)
    .bind(vec![0u8; 8])
    .execute(&pool)
    .await
    .unwrap();
    pool.close().await;
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

/// Log in and return the session's `storageAuthority`, which identity writes must
/// send back (0a11b581).
async fn login(c: &reqwest::Client, base: &str, _mock: &str) -> String {
    let body: Value = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["ok"], json!(true), "plain login (no 2FA): {body}");
    common::auth::storage_authority(&body)
}

#[tokio::test]
async fn identity_signature_name_round_trips_through_the_prefs_route() {
    let mock = spawn_mock().await;
    let (addr, db_path) = spawn_server().await;
    seed_mock_account(&db_path).await;
    let base = format!("http://{addr}");
    let c = client();
    let authority = login(&c, &base, &mock).await;

    // References must name an owned template; create the profile's signature first.
    let signature = c
        .post(format!("{base}/api/account/signatures"))
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .json(&json!({"name":"work-template","body":"Work signature","isDefault":false}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        signature.status(),
        200,
        "{}",
        signature.text().await.unwrap()
    );
    let reply_signature = c.post(format!("{base}/api/account/signatures"))
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .json(&json!({"name":"reply-template","body":"Short reply","isDefault":false}))
        .send().await.unwrap();
    assert_eq!(reply_signature.status(), 200, "{}", reply_signature.text().await.unwrap());

    // POST an identity carrying signatureName (the 26.16-dropped, 26.17-persisted field).
    // This fixture uses the upstream address so it remains eligible for later
    // sender resolution; creating the profile itself no longer requires that.
    let create = c
        .post(format!("{base}/api/account/identities"))
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .json(&json!({
            "name": "Work",
            "email": mw_mock_jmap::USER,
            "signatureName": "work-template",
            "replySignatureName": "reply-template",
        }))
        .send()
        .await
        .unwrap();
    let status = create.status();
    let created: Value = create.json().await.unwrap_or(Value::Null);
    assert_eq!(status, 200, "identity create accepted: {created}");
    assert_eq!(created["ok"], json!(true), "create ok: {created}");
    let id = created["id"]
        .as_str()
        .expect("server assigned an id")
        .to_string();

    // GET the identities back → signatureName survives (0020 column + prefs route).
    let list: Value = c
        .get(format!("{base}/api/account/identities"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let identities = list["identities"].as_array().expect("identities array");
    let mine = identities
        .iter()
        .find(|i| i["id"] == json!(id))
        .unwrap_or_else(|| panic!("created identity present: {list}"));
    assert_eq!(
        mine["signatureName"],
        json!("work-template"),
        "signatureName persisted + round-tripped through the prefs route: {mine}"
    );
    assert_eq!(mine["email"], json!(mw_mock_jmap::USER));
    assert_eq!(mine["replySignatureName"], "reply-template");
    let disabled = c.post(format!("{base}/api/account/identities"))
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .json(&json!({"id":id,"name":"Work","email":mw_mock_jmap::USER,"signatureName":"work-template","replySignatureName":""}))
        .send().await.unwrap();
    assert_eq!(disabled.status(), 200, "{}", disabled.text().await.unwrap());
    let disabled:Value = c.get(format!("{base}/api/account/identities")).send().await.unwrap().json().await.unwrap();
    assert_eq!(disabled["identities"].as_array().unwrap().iter().find(|item|item["id"]==id).unwrap()["replySignatureName"], "");
    let inherit = c.post(format!("{base}/api/account/identities"))
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .json(&json!({"id":id,"name":"Work","email":mw_mock_jmap::USER,"signatureName":"work-template"}))
        .send().await.unwrap();
    assert_eq!(inherit.status(), 200, "{}", inherit.text().await.unwrap());
    let inherited:Value = c.get(format!("{base}/api/account/identities")).send().await.unwrap().json().await.unwrap();
    assert!(inherited["identities"].as_array().unwrap().iter().find(|item|item["id"]==id).unwrap().get("replySignatureName").is_none());
    let foreign = c.post(format!("{base}/api/account/identities"))
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .json(&json!({"id":id,"name":"Work","email":mw_mock_jmap::USER,"replySignatureName":"unowned-template"}))
        .send().await.unwrap();
    assert!(!foreign.status().is_success(), "unowned reply templates must be refused");
    let reply_only = c.post(format!("{base}/api/account/identities"))
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .json(&json!({"id":id,"name":"Work","email":mw_mock_jmap::USER,"signatureName":"","replySignatureName":"reply-template"}))
        .send().await.unwrap();
    assert_eq!(reply_only.status(), 200, "{}", reply_only.text().await.unwrap());
    let reply_only:Value = c.get(format!("{base}/api/account/identities")).send().await.unwrap().json().await.unwrap();
    let reply_only = reply_only["identities"].as_array().unwrap().iter().find(|item|item["id"]==id).unwrap();
    assert_eq!(reply_only["signatureName"], "");
    assert_eq!(reply_only["replySignatureName"], "reply-template");

    // An identity POSTed WITHOUT signatureName comes back without it (Option is honoured,
    // not defaulted to a stale value) — proves the column is genuinely per-row.
    let create2: Value = c
        .post(format!("{base}/api/account/identities"))
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .json(&json!({ "name": "Plain", "email": mw_mock_jmap::USER }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id2 = create2["id"].as_str().unwrap().to_string();
    let list2: Value = c
        .get(format!("{base}/api/account/identities"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let plain = list2["identities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == json!(id2))
        .unwrap();
    assert!(
        plain.get("signatureName").is_none() || plain["signatureName"].is_null(),
        "an identity with no signatureName does not fabricate one: {plain}"
    );
}

#[tokio::test]
async fn new_identity_profiles_are_allowed_by_default_without_granting_sender_authority() {
    let mock = spawn_mock().await;
    let (addr, db_path) = spawn_server().await;
    seed_mock_account(&db_path).await;
    let base = format!("http://{addr}");
    let c = client();
    let authority = login(&c, &base, &mock).await;
    let email = "new-profile@example.test";
    let created = c
        .post(format!("{base}/api/account/identities"))
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .json(&json!({"name":"New profile","email":email}))
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), 200, "{}", created.text().await.unwrap());
    let identities: Value = c
        .get(format!("{base}/api/account/identities"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        identities["identities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["email"] == email)
    );
    let sender = c
        .post(format!("{base}/api/account/sender/resolve"))
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .json(&json!({"email":email}))
        .send()
        .await
        .unwrap();
    assert_eq!(sender.status(), 403);
    assert_eq!(
        sender.json::<Value>().await.unwrap()["code"],
        "forbiddenFrom"
    );
}

#[tokio::test]
async fn explicit_administrator_identity_deny_still_refuses_creation() {
    let mock = spawn_mock().await;
    let (addr, db_path) = spawn_server().await;
    seed_mock_account(&db_path).await;
    let base = format!("http://{addr}");
    let c = client();
    let authority = login(&c, &base, &mock).await;
    let pool = sqlx::SqlitePool::connect_with(SqliteConnectOptions::new().filename(&db_path))
        .await
        .unwrap();
    sqlx::query("INSERT INTO setting_policy(key,global_value,allow_user_override,global_locked,updated_by,updated_at) VALUES('cap.identities','false',0,1,'test-admin',1)")
        .execute(&pool).await.unwrap();
    pool.close().await;
    let response = c
        .post(format!("{base}/api/account/identities"))
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .json(&json!({"name":"Blocked","email":mw_mock_jmap::USER}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 403);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["code"], "feature_disabled");
    assert_eq!(body["key"], "cap.identities");
}

#[tokio::test]
async fn large_html_signature_respects_byte_limits_and_preserves_embedded_logos() {
    const HTML_LIMIT: usize = 2 * 1024 * 1024;
    const TEXT_LIMIT: usize = 64 * 1024;
    const JSON_LIMIT: usize = 6 * HTML_LIMIT + 1024 * 1024;
    let mock = spawn_mock().await;
    let (addr, db_path) = spawn_server().await;
    seed_mock_account(&db_path).await;
    let base = format!("http://{addr}");
    let c = client();
    let authority = login(&c, &base, &mock).await;
    let url = format!("{base}/api/account/signatures");
    let prefix = "<p>Signature</p><img src=\"data:image/png;base64,";
    let suffix = "\">";
    let payload_bytes = (HTML_LIMIT - prefix.len() - suffix.len()) / 4 * 4;
    let mut html = format!("{prefix}{}{suffix}", "A".repeat(payload_bytes));
    html.push_str(&" ".repeat(HTML_LIMIT - html.len()));
    assert_eq!(html.len(), HTML_LIMIT);
    let dto = json!({
        "name":"large-logo", "format":"html", "html":html, "isDefault":false,
        "body":"b".repeat(TEXT_LIMIT), "text":"t".repeat(TEXT_LIMIT),
    });
    // A legal escaped envelope approaches the worst-case six bytes per source
    // byte. Its decoded HTML must still be measured as UTF-8, not wire JSON.
    let escaped = serde_json::to_string(&dto).unwrap().replace('A', "\\u0041");
    assert!(escaped.len() > 6 * HTML_LIMIT - 1024);
    assert!(escaped.len() < JSON_LIMIT);
    let saved = c.post(&url)
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .header("content-type", "application/json")
        .body(escaped).send().await.unwrap();
    assert_eq!(saved.status(), 200, "{}", saved.text().await.unwrap());
    let list: Value = c.get(&url).send().await.unwrap().json().await.unwrap();
    let stored = &list["signatures"][0];
    // The sanitizer tidies trailing whitespace, so the padding that fills the byte limit collapses to at most one space.
    assert_eq!(stored["html"].as_str().unwrap().trim_end(), dto["html"].as_str().unwrap().trim_end(), "the complete raster data URL survives sanitization and persistence");
    assert_eq!(stored["body"], dto["body"]);
    assert_eq!(stored["text"], dto["text"]);
    let resaved = c.post(&url)
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .json(stored).send().await.unwrap();
    assert_eq!(resaved.status(), 200, "stored signatures must save unchanged");
    let mut expansion = dto.clone();
    let expanding_html = format!("<p>{}</p>", "&".repeat(HTML_LIMIT / 2));
    assert!(expanding_html.len() < HTML_LIMIT);
    assert!(mw_sanitize::sanitize_signature_html(&expanding_html).len() > HTML_LIMIT);
    expansion["html"] = json!(expanding_html);
    let rejected = c.post(&url)
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .json(&expansion).send().await.unwrap();
    assert_eq!(rejected.status(), 400, "sanitizer expansion must not exceed the stored HTML limit");
    let unchanged: Value = c.get(&url).send().await.unwrap().json().await.unwrap();
    assert_eq!(unchanged["signatures"][0]["html"].as_str().unwrap().trim_end(), dto["html"].as_str().unwrap().trim_end());
    for field in ["html", "body", "text"] {
        let mut oversized = dto.clone();
        oversized[field] = json!(format!("{}x", oversized[field].as_str().unwrap()));
        let rejected = c.post(&url)
            .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
            .json(&oversized).send().await.unwrap();
        assert_eq!(rejected.status(), 400, "one byte over the {field} limit must be refused");
    }
    let oversized_envelope = c.post(&url)
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .header("content-type", "application/json")
        .body(" ".repeat(JSON_LIMIT + 1)).send().await.unwrap();
    assert_eq!(oversized_envelope.status(), 413);
    let anonymous = client().post(&url)
        .header("content-type", "application/json")
        .body("invalid JSON").send().await.unwrap();
    assert_eq!(anonymous.status(), 401, "authentication must run before JSON extraction");
}

#[tokio::test]
async fn identity_default_and_variants_work_through_the_prefs_routes() {
    let mock = spawn_mock().await;
    let (addr, db_path) = spawn_server().await;
    seed_mock_account(&db_path).await;
    let base = format!("http://{addr}");
    let c = client();
    let authority = login(&c, &base, &mock).await;
    let h = common::auth::STORAGE_AUTHORITY_HEADER;
    let post = |body: Value| {
        let (c, base, authority) = (c.clone(), base.clone(), authority.clone());
        async move {
            c.post(format!("{base}/api/account/identities"))
                .header(h, authority)
                .json(&body)
                .send()
                .await
                .unwrap()
        }
    };
    let list = || {
        let (c, base) = (c.clone(), base.clone());
        async move {
            c.get(format!("{base}/api/account/identities"))
                .send()
                .await
                .unwrap()
                .json::<Value>()
                .await
                .unwrap()["identities"]
                .as_array()
                .unwrap()
                .clone()
        }
    };
    let create = |body: Value| {
        let post = &post;
        async move {
            let r = post(body).await;
            assert_eq!(r.status(), 200);
            r.json::<Value>().await.unwrap()["id"].as_str().unwrap().to_string()
        }
    };
    let first = create(json!({"name":"Me","email":"me@example.test","replyTo":"help@example.test"})).await;
    let second = create(json!({"name":"Other","email":"other@example.test"})).await;
    let rows = list().await;
    let default_of = |rows: &[Value]| -> Vec<String> {
        rows.iter().filter(|r| r["isDefault"] == true).map(|r| r["id"].as_str().unwrap().to_string()).collect()
    };
    assert_eq!(default_of(&rows), [first.clone()], "the first identity is the default");

    let made = c
        .post(format!("{base}/api/account/identities/{second}/default"))
        .header(h, &authority)
        .send()
        .await
        .unwrap();
    assert_eq!(made.status(), 200);
    assert_eq!(default_of(&list().await), [second.clone()]);
    let missing = c
        .post(format!("{base}/api/account/identities/nope/default"))
        .header(h, &authority)
        .send()
        .await
        .unwrap();
    assert_eq!(missing.status(), 404);

    // Variants: label required, base must exist, no nesting.
    assert_eq!(post(json!({"name":"V","email":"v@example.test","variantOf":first})).await.status(), 400);
    assert_eq!(post(json!({"name":"V","email":"v@example.test","variantOf":"nope","label":"X"})).await.status(), 400);
    assert_eq!(post(json!({"name":"V","email":"v@example.test","label":"X"})).await.status(), 400);
    let variant = create(json!({"name":"Me (news)","email":"me@example.test","variantOf":first,"label":"News"})).await;
    assert_eq!(
        post(json!({"name":"W","email":"w@example.test","variantOf":variant,"label":"Nested"})).await.status(),
        400
    );
    let rows = list().await;
    let v = rows.iter().find(|r| r["id"] == variant.as_str()).unwrap();
    assert_eq!(v["variantOf"], first.as_str());
    assert_eq!(v["label"], "News");
    assert!(v.get("replyTo").is_none(), "REST returns raw fields; the client resolves inheritance");

    // A base with variants needs a choice.
    let refused = c
        .delete(format!("{base}/api/account/identities/{first}"))
        .header(h, &authority)
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), 409);
    assert_eq!(refused.json::<Value>().await.unwrap()["variants"], 1);
    let promoted = c
        .delete(format!("{base}/api/account/identities/{first}?variants=promote"))
        .header(h, &authority)
        .send()
        .await
        .unwrap();
    assert_eq!(promoted.status(), 200);
    let rows = list().await;
    let v = rows.iter().find(|r| r["id"] == variant.as_str()).unwrap();
    assert!(v.get("variantOf").is_none());
    assert_eq!(v["replyTo"], "help@example.test", "promotion copies inherited fields");
    assert_eq!(default_of(&rows), [second.clone()]);

    // Deleting the default promotes another.
    let del = c
        .delete(format!("{base}/api/account/identities/{second}"))
        .header(h, &authority)
        .send()
        .await
        .unwrap();
    assert_eq!(del.status(), 200);
    assert_eq!(default_of(&list().await), [variant.clone()]);
}
