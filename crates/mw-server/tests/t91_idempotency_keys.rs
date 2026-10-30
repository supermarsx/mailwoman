//! t91: `Idempotency-Key` makes a retried create safe.
//!
//! The web client retries a create after a transient failure (a 503 from a
//! proxy, a dropped connection) only with an `Idempotency-Key`, the same on every
//! attempt of one action. The server keeps the first 2xx answer per session,
//! method, path and key, and answers a repeat with it instead of running it.
//!
//! Real router over HTTP, engine mode, real sign-ins (the t82 fixture). Covered:
//!   - a JMAP create (`Note/set`) sent twice with one key creates ONE note, and
//!     the repeat answers the same body with `Idempotent-Replayed: true`; without
//!     a key the same two sends create two (the count can see a duplicate);
//!   - two sends racing with one key create one note;
//!   - another key is another request; the same key with another body is 422;
//!   - the record belongs to the session: another sign-in's same key runs;
//!   - a malformed key is refused 400;
//!   - `POST /api/account/identities` and `PUT /api/account/saved-searches`
//!     (creates without an id) twice with one key: one record each;
//!   - linking a mailbox twice with one key: one link with one id; a refused
//!     password is not kept, so the corrected retry with that key links.
//!
//! Run: cargo test -p mw-server --test t91_idempotency_keys -- --test-threads=1
mod common;

use std::net::SocketAddr;
use std::sync::Arc;

use common::mock_mail::MockPop3;
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::{DomainConfigRow, ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};

const KEY: &str = "91a1b2c3d4e5f6071829304a5b6c7d8e91a1b2c3d4e5f6071829304a5b6c7d8e";
const PARENT_PASS: &str = "Parent-Passw0rd!";
const CHILD_PASS: &str = "Child-Secret-t91-4e1b";

async fn store(db: &str) -> Store {
    Store::open(db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap()
}
async fn policy(db: &str, key: &str, value: &str) {
    store(db)
        .await
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "t91-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
}
async fn domain(db: &str, name: &str, addr: SocketAddr) {
    store(db)
        .await
        .put_domain_config(&DomainConfigRow {
            name: name.into(),
            upstream_json: json!({"pop3":{"host":"127.0.0.1","port":addr.port(),"tls":"none","allowPrivate":true}}).to_string(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}

async fn serve(db: &str) -> String {
    let config = AppConfig {
        db_path: db.into(),
        server_key_hex: Some(KEY.into()),
        web_dir: None,
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig {
            csrf_strict: true,
            ..Default::default()
        },
        security: SecurityConfig {
            jmap_upstreams: Some(Vec::new()),
            ..Default::default()
        },
    };
    let v6 = V6Config {
        admin_enabled: false,
        admin_username: None,
        admin_password: None,
        redis_url: None,
    };
    let (app, _push) = mw_server::build_app_full(config, v6).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap()
    });
    base
}

async fn json_of(response: reqwest::Response, expected: u16) -> Value {
    let status = response.status();
    let text = response.text().await.unwrap();
    assert_eq!(status, expected, "{text}");
    if text.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&text).unwrap_or(Value::String(text))
    }
}

async fn sign_in(base: &str, user: &str, pass: &str) -> reqwest::Client {
    let jar = Arc::new(reqwest::cookie::Jar::default());
    let bare = reqwest::Client::builder()
        .no_proxy()
        .cookie_provider(jar.clone())
        .build()
        .unwrap();
    let login = json_of(
        bare.post(format!("{base}/api/login"))
            .json(&json!({"username":user,"password":pass}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "x-csrf-token",
        login["csrfToken"].as_str().unwrap().parse().unwrap(),
    );
    headers.insert(
        "x-mailwoman-storage-authority",
        login["storageAuthority"].as_str().unwrap().parse().unwrap(),
    );
    reqwest::Client::builder()
        .no_proxy()
        .cookie_provider(jar)
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap()
}

const NOTES: [&str; 2] = ["urn:ietf:params:jmap:core", "urn:mailwoman:notes"];

async fn post_jmap(
    http: &reqwest::Client,
    base: &str,
    key: Option<&str>,
    body: &Value,
) -> reqwest::Response {
    let mut request = http.post(format!("{base}/jmap/api")).json(body);
    if let Some(key) = key {
        request = request.header("idempotency-key", key);
    }
    request.send().await.unwrap()
}

fn create_note(account: &str, title: &str) -> Value {
    json!({"using":NOTES,"methodCalls":[["Note/set",{"accountId":account,"create":{"new":{"title":title,"bodyHtml":"","tags":[],"pinned":false,"links":[]}}},"s"]]})
}

/// The account id the session's JMAP calls name.
async fn account_id(http: &reqwest::Client, base: &str) -> String {
    let session = json_of(
        http.get(format!("{base}/jmap/session"))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let primary = &session["primaryAccounts"];
    primary["urn:mailwoman:notes"]
        .as_str()
        .or_else(|| primary["urn:ietf:params:jmap:mail"].as_str())
        .unwrap_or_else(|| panic!("no account in {session}"))
        .to_string()
}

/// How many notes carry `title`.
async fn notes_titled(http: &reqwest::Client, base: &str, account: &str, title: &str) -> usize {
    let listing = json_of(
        post_jmap(
            http,
            base,
            None,
            &json!({"using":NOTES,"methodCalls":[["Note/get",{"accountId":account,"ids":null},"g"]]}),
        )
        .await,
        200,
    )
    .await;
    listing["methodResponses"][0][1]["list"]
        .as_array()
        .unwrap_or_else(|| panic!("Note/get answered {listing}"))
        .iter()
        .filter(|note| note["title"] == title)
        .count()
}

fn created_id(response: &Value) -> String {
    response["methodResponses"][0][1]["created"]["new"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("no note was created: {response}"))
        .to_string()
}

fn replayed(response: &reqwest::Response) -> bool {
    response
        .headers()
        .get("idempotent-replayed")
        .is_some_and(|v| v == "true")
}

async fn setup(tag: &str) -> (String, reqwest::Client, MockPop3, MockPop3) {
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
    }
    let parents = MockPop3::start_accounts(&[("parent@example.org", PARENT_PASS)]).await;
    let child = MockPop3::start_accounts(&[("child@child.test", CHILD_PASS)]).await;
    let dir = common::test_db::unique_dir(tag);
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    policy(&db, "signin.retryCooldownSecs", "0").await;
    policy(&db, "accounts.linkingEnabled", "true").await;
    domain(&db, "example.org", parents.addr).await;
    domain(&db, "child.test", child.addr).await;
    let base = serve(&db).await;
    let parent = sign_in(&base, "parent@example.org", PARENT_PASS).await;
    (base, parent, parents, child)
}

#[tokio::test]
async fn a_jmap_create_sent_twice_with_one_key_creates_one_record() {
    let (base, parent, _p, _c) = setup("mw-t91-jmap").await;
    let account = account_id(&parent, &base).await;

    // Baseline: without a key, two sends are two notes (the count sees a duplicate).
    let plain = create_note(&account, "t91 plain");
    for _ in 0..2 {
        json_of(post_jmap(&parent, &base, None, &plain).await, 200).await;
    }
    assert_eq!(notes_titled(&parent, &base, &account, "t91 plain").await, 2);

    // One key, sent twice as a retry after a lost answer is: one note.
    let keyed = create_note(&account, "t91 keyed");
    let first = post_jmap(&parent, &base, Some("t91-key-one"), &keyed).await;
    assert!(!replayed(&first));
    let first = json_of(first, 200).await;
    let second = post_jmap(&parent, &base, Some("t91-key-one"), &keyed).await;
    assert!(replayed(&second), "the repeat is answered from the record");
    let second = json_of(second, 200).await;
    assert_eq!(first, second, "the repeat gets the first answer");
    assert_eq!(created_id(&first), created_id(&second));
    assert_eq!(notes_titled(&parent, &base, &account, "t91 keyed").await, 1);

    // Two sends racing with one key: one note.
    let raced = create_note(&account, "t91 raced");
    let (a, b) = tokio::join!(
        post_jmap(&parent, &base, Some("t91-key-race"), &raced),
        post_jmap(&parent, &base, Some("t91-key-race"), &raced),
    );
    let (a, b) = (json_of(a, 200).await, json_of(b, 200).await);
    assert_eq!(created_id(&a), created_id(&b));
    assert_eq!(notes_titled(&parent, &base, &account, "t91 raced").await, 1);

    // Another key is another request.
    json_of(
        post_jmap(&parent, &base, Some("t91-key-two"), &keyed).await,
        200,
    )
    .await;
    assert_eq!(notes_titled(&parent, &base, &account, "t91 keyed").await, 2);

    // The same key with another body is refused, and creates nothing.
    let other = create_note(&account, "t91 other");
    let refused = json_of(
        post_jmap(&parent, &base, Some("t91-key-one"), &other).await,
        422,
    )
    .await;
    assert_eq!(refused["code"], "idempotencyKeyReused");
    assert_eq!(notes_titled(&parent, &base, &account, "t91 other").await, 0);

    // The record belongs to the session: another sign-in's same key runs.
    let again = sign_in(&base, "parent@example.org", PARENT_PASS).await;
    let fresh = post_jmap(&again, &base, Some("t91-key-one"), &keyed).await;
    assert!(!replayed(&fresh));
    json_of(fresh, 200).await;
    assert_eq!(notes_titled(&parent, &base, &account, "t91 keyed").await, 3);

    // A malformed key is refused before anything runs.
    let bad = json_of(
        post_jmap(&parent, &base, Some("has space"), &other).await,
        400,
    )
    .await;
    assert_eq!(bad["code"], "invalidIdempotencyKey");
    assert_eq!(notes_titled(&parent, &base, &account, "t91 other").await, 0);
}

#[tokio::test]
async fn identity_and_saved_search_creates_with_one_key_make_one_record() {
    let (base, parent, _p, _c) = setup("mw-t91-prefs").await;
    let identity = json!({"name":"t91 Desk","email":"parent@example.org"});
    let mut ids = Vec::new();
    for n in 0..2 {
        let response = parent
            .post(format!("{base}/api/account/identities"))
            .header("idempotency-key", "t91-identity")
            .json(&identity)
            .send()
            .await
            .unwrap();
        assert_eq!(replayed(&response), n == 1);
        let saved = json_of(response, 200).await;
        ids.push(saved["id"].as_str().unwrap().to_string());
    }
    assert_eq!(ids[0], ids[1]);
    let listing = json_of(
        parent
            .get(format!("{base}/api/account/identities"))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let named = listing["identities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["name"] == "t91 Desk")
        .count();
    assert_eq!(named, 1, "{listing}");

    let search = json!({"name":"t91 Search","queryJson":"{}","asFolder":true});
    for n in 0..2 {
        let response = parent
            .put(format!("{base}/api/account/saved-searches"))
            .header("idempotency-key", "t91-search")
            .json(&search)
            .send()
            .await
            .unwrap();
        assert_eq!(replayed(&response), n == 1);
        json_of(response, 200).await;
    }
    let listing = json_of(
        parent
            .get(format!("{base}/api/account/saved-searches"))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let named = listing["savedSearches"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["name"] == "t91 Search")
        .count();
    assert_eq!(named, 1, "{listing}");
}

#[tokio::test]
async fn linking_twice_with_one_key_links_once_and_a_refused_password_is_not_kept() {
    let (base, parent, _p, _c) = setup("mw-t91-link").await;
    json_of(
        parent
            .post(format!("{base}/api/security/step-up"))
            .json(&json!({"password":PARENT_PASS}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let link = |password: &'static str| {
        parent
            .post(format!("{base}/api/accounts/link"))
            .header("idempotency-key", "t91-link")
            .json(&json!({"username":"child@child.test","password":password,"displayName":"Child"}))
            .send()
    };
    // A refused password is answered and not kept.
    let refused = link("wrong-password").await.unwrap();
    assert!(!refused.status().is_success(), "{}", refused.status());
    assert!(!replayed(&refused));
    // The same key may then carry the corrected password; its repeat is the same link.
    let first = json_of(link(CHILD_PASS).await.unwrap(), 201).await;
    let repeat = link(CHILD_PASS).await.unwrap();
    assert!(replayed(&repeat));
    let repeat = json_of(repeat, 201).await;
    assert_eq!(first["account"]["id"], repeat["account"]["id"]);
    let listing = json_of(
        parent
            .get(format!("{base}/api/accounts"))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert_eq!(
        listing["accounts"].as_array().unwrap().len(),
        1,
        "{listing}"
    );
}
