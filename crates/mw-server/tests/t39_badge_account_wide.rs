//! t39: the unread badge colour is one value across a user's linked mailboxes.
//!
//! The real router over HTTP in engine mode: real sign-ins, a real step-up and
//! the real link endpoint. The badge colour saved in one mailbox is what every
//! other mailbox of the same sign-in reads back; the rest of the appearance
//! object stays per mailbox. A value saved per mailbox before this change is
//! read through once, and the parent's wins over a child's.
//!
//! Run:
//!   cargo test -p mw-server --test t39_badge_account_wide --locked -- --test-threads=1

use std::path::PathBuf;
use std::time::Duration;

use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::settings_policy::SettingPolicyRow;

mod common;
use common::mock_mail::MockPop3;
use common::test_db;

const KEY_HEX: &str = "39b1c2d3e4f5061728394a5b6c7d8e9f39b1c2d3e4f5061728394a5b6c7d8e9f";
const PASS: &str = "Badge-Passw0rd!";

struct Server {
    base: String,
    db: String,
    _pop: MockPop3,
}

async fn store(db: &str) -> mw_store::Store {
    mw_store::Store::open(db, mw_store::ServerKey::from_hex(KEY_HEX).unwrap())
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
            updated_by: "t39-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
}

async fn spawn(users: &[&str]) -> Server {
    // SAFETY: run with --test-threads=1; every test sets the same value.
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
    }
    let accounts: Vec<(&str, &str)> = users.iter().map(|u| (*u, PASS)).collect();
    let pop = MockPop3::start_accounts(&accounts).await;
    let dir = test_db::unique_dir("mw-t39-badge");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), "<!doctype html><title>MW</title>").unwrap();
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    policy(&db, "signin.retryCooldownSecs", "0").await;
    policy(&db, "accounts.linkingEnabled", "true").await;
    let config = AppConfig {
        db_path: db.clone(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(Vec::new()),
            upstream_default: Some(pop.url()),
            ..SecurityConfig::default()
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
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    Server {
        base: format!("http://{addr}"),
        db,
        _pop: pop,
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap()
}

#[derive(Clone)]
struct User {
    cookie: String,
    csrf: String,
    authority: String,
}

async fn login(srv: &Server, user: &str) -> User {
    let r = client()
        .post(format!("{}/api/login", srv.base))
        .json(&json!({ "username": user, "password": PASS }))
        .send()
        .await
        .unwrap();
    let status = r.status();
    let headers = r.headers().clone();
    let body: Value = r.json().await.unwrap_or(Value::Null);
    assert_eq!(status, 200, "login for {user}: {body}");
    let cookie = headers
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .filter_map(|c| c.split(';').next())
        .filter(|c| !c.ends_with('='))
        .collect::<Vec<_>>()
        .join("; ");
    User {
        cookie,
        csrf: body["csrfToken"].as_str().unwrap_or_default().to_string(),
        authority: body["storageAuthority"].as_str().unwrap().to_string(),
    }
}

/// A request in one mailbox of `user`'s sign-in: the sign-in mailbox when
/// `link` is `None`, else the linked one (with that mailbox's authority).
fn req(
    user: &User,
    m: reqwest::Method,
    url: String,
    link: Option<(&str, &str)>,
) -> reqwest::RequestBuilder {
    let r = client()
        .request(m, url)
        .header("cookie", &user.cookie)
        .header("x-csrf-token", &user.csrf);
    match link {
        None => r.header("x-mailwoman-storage-authority", &user.authority),
        Some((id, authority)) => r
            .header("x-mailwoman-link", id)
            .header("x-mailwoman-storage-authority", authority),
    }
}

async fn link(srv: &Server, parent: &User, child: &str) -> (String, String) {
    let step = req(
        parent,
        reqwest::Method::POST,
        format!("{}/api/security/step-up", srv.base),
        None,
    )
    .json(&json!({ "password": PASS }))
    .send()
    .await
    .unwrap();
    assert_eq!(step.status(), 200, "step-up");
    let r = req(
        parent,
        reqwest::Method::POST,
        format!("{}/api/accounts/link", srv.base),
        None,
    )
    .json(&json!({ "username": child, "password": PASS }))
    .send()
    .await
    .unwrap();
    let status = r.status();
    let body: Value = r.json().await.unwrap();
    assert_eq!(status, 201, "link {child}: {body}");
    (
        body["account"]["id"].as_str().unwrap().to_string(),
        body["account"]["storageAuthority"]
            .as_str()
            .unwrap()
            .to_string(),
    )
}

async fn get(srv: &Server, user: &User, link: Option<(&str, &str)>) -> Value {
    let r = req(
        user,
        reqwest::Method::GET,
        format!("{}/api/account/appearance", srv.base),
        link,
    )
    .send()
    .await
    .unwrap();
    assert_eq!(r.status(), 200, "get appearance");
    r.json().await.unwrap()
}

async fn put(srv: &Server, user: &User, link: Option<(&str, &str)>, appearance: Value) {
    let r = req(
        user,
        reqwest::Method::PUT,
        format!("{}/api/account/appearance", srv.base),
        link,
    )
    .json(&json!({ "appearance": appearance }))
    .send()
    .await
    .unwrap();
    let status = r.status();
    assert_eq!(status, 200, "put appearance: {}", r.text().await.unwrap());
}

#[tokio::test]
async fn one_badge_colour_for_the_sign_in_and_every_linked_mailbox() {
    let srv = spawn(&[
        "parent@example.org",
        "child@example.org",
        "other@example.org",
    ])
    .await;
    let parent = login(&srv, "parent@example.org").await;
    let (link_id, child_authority) = link(&srv, &parent, "child@example.org").await;
    let child = Some((link_id.as_str(), child_authority.as_str()));

    // Saved in the sign-in mailbox: the child reads the same colour, though
    // the child has no appearance of its own.
    put(
        &srv,
        &parent,
        None,
        json!({ "theme": "ocean-dark", "badgeColor": "plum" }),
    )
    .await;
    let seen = get(&srv, &parent, child).await;
    assert_eq!(seen["appearance"]["badgeColor"], "plum", "{seen}");
    assert!(
        seen["appearance"].get("theme").is_none(),
        "the theme stays per mailbox: {seen}"
    );

    // Saved in the child: the sign-in mailbox follows, and keeps its theme.
    put(
        &srv,
        &parent,
        child,
        json!({ "theme": "grove-light", "badgeColor": "ocean" }),
    )
    .await;
    let seen = get(&srv, &parent, None).await;
    assert_eq!(seen["appearance"]["badgeColor"], "ocean", "{seen}");
    assert_eq!(seen["appearance"]["theme"], "ocean-dark", "{seen}");
    let seen = get(&srv, &parent, child).await;
    assert_eq!(seen["appearance"]["theme"], "grove-light", "{seen}");

    // Another sign-in is not affected.
    let other = login(&srv, "other@example.org").await;
    let seen = get(&srv, &other, None).await;
    assert!(seen["appearance"].is_null(), "{seen}");
}

#[tokio::test]
async fn values_saved_per_mailbox_earlier_resolve_to_the_parents() {
    let srv = spawn(&["parent@example.org", "child@example.org"]).await;
    let parent = login(&srv, "parent@example.org").await;
    let (link_id, child_authority) = link(&srv, &parent, "child@example.org").await;
    let child = Some((link_id.as_str(), child_authority.as_str()));

    // Two appearance objects as written before t39: one badge colour each.
    let db = store(&srv.db).await;
    for (authority, badge) in [(&parent.authority, "grove"), (&child_authority, "plum")] {
        let envelope = json!({ "v": 1, "updated_at": 1, "prefs": { "badgeColor": badge } });
        db.set_setting(
            &format!("personal-v2:appearance:{authority}"),
            &envelope.to_string(),
        )
        .await
        .unwrap();
    }

    let seen = get(&srv, &parent, child).await;
    assert_eq!(
        seen["appearance"]["badgeColor"], "grove",
        "the parent's value wins: {seen}"
    );
    let seen = get(&srv, &parent, None).await;
    assert_eq!(seen["appearance"]["badgeColor"], "grove", "{seen}");
}

async fn setting(srv: &Server, user: &User, link: Option<(&str, &str)>, key: &str) -> Value {
    let r = req(
        user,
        reqwest::Method::GET,
        format!("{}/api/settings", srv.base),
        link,
    )
    .send()
    .await
    .unwrap();
    assert_eq!(r.status(), 200, "get settings");
    let body: Value = r.json().await.unwrap();
    body["settings"][key]["value"].clone()
}

async fn put_setting(
    srv: &Server,
    user: &User,
    link: Option<(&str, &str)>,
    key: &str,
    value: Value,
) {
    let r = req(
        user,
        reqwest::Method::PUT,
        format!("{}/api/settings/{key}", srv.base),
        link,
    )
    .json(&json!({ "value": value }))
    .send()
    .await
    .unwrap();
    let status = r.status();
    assert_eq!(status, 200, "put {key}: {}", r.text().await.unwrap());
}

/// The counter-bubble settings are one choice for the sign-in and its links;
/// an unrelated setting stays per mailbox.
#[tokio::test]
async fn counter_bubble_settings_are_shared_by_linked_mailboxes() {
    let srv = spawn(&["parent@example.org", "child@example.org"]).await;
    let parent = login(&srv, "parent@example.org").await;
    let (link_id, child_authority) = link(&srv, &parent, "child@example.org").await;
    let child = Some((link_id.as_str(), child_authority.as_str()));

    // Defaults: all folders, and no bubble beside the mailbox name.
    assert_eq!(
        setting(&srv, &parent, child, "mail.folderCountScope").await,
        "all"
    );
    assert_eq!(
        setting(&srv, &parent, child, "mail.accountUnreadBadge").await,
        false
    );

    put_setting(&srv, &parent, None, "mail.folderCountScope", json!("main")).await;
    put_setting(
        &srv,
        &parent,
        None,
        "mail.folderCounts",
        json!("unreadIfAny"),
    )
    .await;
    assert_eq!(
        setting(&srv, &parent, child, "mail.folderCountScope").await,
        "main"
    );
    assert_eq!(
        setting(&srv, &parent, child, "mail.folderCounts").await,
        "unreadIfAny"
    );

    put_setting(&srv, &parent, child, "mail.accountUnreadBadge", json!(true)).await;
    assert_eq!(
        setting(&srv, &parent, None, "mail.accountUnreadBadge").await,
        true
    );

    // Not account wide: the child's own choice does not reach the parent.
    put_setting(&srv, &parent, child, "mail.dragToFolder", json!(false)).await;
    assert_eq!(
        setting(&srv, &parent, None, "mail.dragToFolder").await,
        true
    );
}
