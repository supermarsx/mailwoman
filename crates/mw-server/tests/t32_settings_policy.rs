//! t32-e10b (worker f01) — the settings model C over the real router.
//!
//! Every test signs in through the real `POST /admin/login` and `POST /api/login`,
//! so the cookies, the session rows and every gate are the production ones. A
//! refusal is checked next to a control that shows the same request going through,
//! so no test passes because a route was unreachable.
//!
//! * Resolution order (L6): global < domain < admin per-user < user, a lock at
//!   any scope beating every narrower one, user overrides switched off.
//! * Enforcement: a held key refuses the user's write and keeps the stored value;
//!   the appearance object's held fields are replaced; notifications and read
//!   receipts refuse a held change.
//! * The admin routes' contract (R-CFG 1-4): `{code, detail, field}` refusals,
//!   `useDedicatedRoute` for `signin.*`, `pinnedByEnv`, `If-Match` → 412, the
//!   lock impact on `dryRun`, one `setting-changed` audit row per write.
//! * The deployment appearance is stored in the database, survives a restart and
//!   is what accounts with no appearance of their own are served.
//! * The 401 cases.
//!
//! SQLite only: the store methods are run on Postgres by `mw-store`'s
//! `settings_policy::tests::postgres_legs`.

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

mod common;
use common::mock_mail::MockPop3;
use common::test_db;

const KEY_HEX: &str = "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";
const ENGINE_USER: &str = "reader@example.org";
const ENGINE_PASS: &str = "Read-Passw0rd!";

// ── servers ────────────────────────────────────────────────────────────────────

fn admin_v6() -> V6Config {
    V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    }
}

fn config(db: &str, mode: ServerMode, upstream: String) -> AppConfig {
    let web: PathBuf = PathBuf::from(db).parent().unwrap().join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    AppConfig {
        db_path: db.to_string(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(match mode {
                ServerMode::Proxy => vec![upstream.clone()],
                _ => Vec::new(),
            }),
            upstream_default: Some(upstream),
            ..SecurityConfig::default()
        },
    }
}

async fn serve(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{addr}")
}

async fn spawn_mock_jmap() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

struct Server {
    base: String,
    db: String,
    upstream: String,
}

fn new_db(tag: &str) -> String {
    test_db::unique_dir(tag)
        .join("mw.db")
        .to_string_lossy()
        .into_owned()
}

/// A proxy-mode server in front of a mock JMAP upstream.
async fn proxy_server(tag: &str) -> Server {
    let db = new_db(tag);
    let upstream = spawn_mock_jmap().await;
    proxy_server_on(db, upstream).await
}

async fn proxy_server_on(db: String, upstream: String) -> Server {
    let app = build_app_full(config(&db, ServerMode::Proxy, upstream.clone()), admin_v6())
        .await
        .expect("server boots")
        .0;
    Server {
        base: serve(app).await,
        db,
        upstream,
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

async fn admin(base: &str) -> reqwest::Client {
    let c = client();
    let r = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "admin login");
    c
}

/// A signed-in mailbox user and their account id. The client sends the login's
/// storage authority on every request, as the web client does: a personal-settings
/// write without it answers `428 storageAuthorityRequired` (0a11b581).
async fn user(base: &str) -> (reqwest::Client, String) {
    let jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
    let login = reqwest::Client::builder()
        .cookie_provider(jar.clone())
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let me: Value = login
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(me["ok"], json!(true), "mailbox login: {me}");
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "x-mailwoman-storage-authority",
        me["storageAuthority"]
            .as_str()
            .expect("login names a storageAuthority")
            .parse()
            .unwrap(),
    );
    let c = reqwest::Client::builder()
        .cookie_provider(jar)
        .default_headers(headers)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    (c, me["accountId"].as_str().unwrap().to_string())
}

/// An engine-mode server in front of a scripted POP3 upstream, with the mailbox
/// [`ENGINE_USER`] signed in: a native account. Since 0e50de6a an admin per-user
/// value and the user-value impact report address native accounts only; a
/// proxy-mode account id is an unverified upstream id, so its own values are
/// kept in its personal store instead.
async fn engine_user(tag: &str) -> (Server, MockPop3, reqwest::Client, String) {
    // The mock speaks plain POP3; the engine reads MW_ENGINE_TLS per login.
    // SAFETY: the canonical gate runs this binary with --test-threads=1, and no
    // test here reads this variable otherwise.
    unsafe { std::env::set_var("MW_ENGINE_TLS", "plaintext") };
    let pop = MockPop3::start(ENGINE_USER, ENGINE_PASS).await;
    let db = new_db(tag);
    let app = build_app_full(config(&db, ServerMode::Engine, pop.url()), admin_v6())
        .await
        .expect("engine server boots")
        .0;
    let base = serve(app).await;
    let jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
    let login = reqwest::Client::builder()
        .cookie_provider(jar.clone())
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let me: Value = login
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": ENGINE_USER, "password": ENGINE_PASS }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(me["ok"], json!(true), "engine login: {me}");
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "x-mailwoman-storage-authority",
        me["storageAuthority"]
            .as_str()
            .expect("login names a storageAuthority")
            .parse()
            .unwrap(),
    );
    let c = reqwest::Client::builder()
        .cookie_provider(jar)
        .default_headers(headers)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let account = me["accountId"].as_str().unwrap().to_string();
    let upstream = pop.url();
    (Server { base, db, upstream }, pop, c, account)
}

async fn send(req: reqwest::RequestBuilder) -> (u16, Value) {
    let r = req.send().await.unwrap();
    let status = r.status().as_u16();
    let text = r.text().await.unwrap();
    (
        status,
        serde_json::from_str(&text).unwrap_or(Value::String(text)),
    )
}

async fn put_global(a: &reqwest::Client, base: &str, key: &str, body: Value) -> (u16, Value) {
    send(a.put(format!("{base}/admin/settings/{key}")).json(&body)).await
}

async fn user_settings(u: &reqwest::Client, base: &str) -> Value {
    let (status, body) = send(u.get(format!("{base}/api/settings"))).await;
    assert_eq!(status, 200, "{body}");
    body["settings"].clone()
}

async fn entry(u: &reqwest::Client, base: &str, key: &str) -> Value {
    user_settings(u, base).await[key].clone()
}

// ── 401 ────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn requests_without_a_session_are_refused() {
    let s = proxy_server("mw-t32-f01-401").await;
    let anon = client();
    let (status, _) = send(anon.get(format!("{}/api/settings", s.base))).await;
    assert_eq!(status, 401, "user settings need a mailbox session");
    let (status, _) = send(anon.get(format!("{}/admin/settings", s.base))).await;
    assert_eq!(status, 401, "admin settings need an admin session");
    let (status, _) = send(
        anon.put(format!("{}/api/settings/ui.language", s.base))
            .json(&json!({ "value": "de" })),
    )
    .await;
    assert_eq!(status, 401);

    // A mailbox session is not an admin session.
    let (u, _) = user(&s.base).await;
    let (status, _) = put_global(&u, &s.base, "ui.language", json!({ "globalValue": "de" })).await;
    assert_eq!(
        status, 401,
        "a mailbox cookie does not open the admin routes"
    );
    // Controls: the same requests with the right sessions go through.
    assert_eq!(send(u.get(format!("{}/api/settings", s.base))).await.0, 200);
    let a = admin(&s.base).await;
    assert_eq!(
        send(a.get(format!("{}/admin/settings", s.base))).await.0,
        200
    );
}

// ── the overview ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn the_overview_describes_every_key_with_an_etag() {
    let s = proxy_server("mw-t32-f01-list").await;
    let a = admin(&s.base).await;
    let r = a
        .get(format!("{}/admin/settings", s.base))
        .send()
        .await
        .unwrap();
    assert!(
        r.headers().get("etag").is_some(),
        "the overview carries an ETag"
    );
    let body: Value = r.json().await.unwrap();
    let items = body["settings"].as_array().unwrap();
    let find = |k: &str| {
        items
            .iter()
            .find(|i| i["key"] == k)
            .cloned()
            .unwrap_or(Value::Null)
    };
    let lang = find("ui.language");
    assert_eq!(lang["kind"]["type"], "locale");
    assert_eq!(lang["default"], "browser");
    assert_eq!(lang["scopes"], "global+domain+user");
    assert_eq!(lang["hot"], true);
    assert!(lang["etag"].is_string());
    assert_eq!(find("appearance.theme")["default"], "dark");
    assert!(
        find("signin.newUsers")["dedicatedRoute"]
            .as_str()
            .unwrap()
            .ends_with("/signin-policy")
    );
    assert_eq!(find("limit.storage_bytes")["scopeId"], "address");
    assert_eq!(find("limit.storage_bytes")["ceiling"], true);
    assert_eq!(
        find("realtime.recheckSecs")["env"]["var"],
        "MW_REALTIME_RECHECK_SECS"
    );
    assert!(
        body["families"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["key"] == "serverText.*")
    );
    assert_eq!(body["propagationSecs"], 30);
}

// ── resolution order ───────────────────────────────────────────────────────────

#[tokio::test]
async fn resolution_follows_the_scope_order_and_locks() {
    let (s, _pop, u, account) = engine_user("mw-t32-f01-order").await;
    let a = admin(&s.base).await;
    let base = &s.base;
    let key = "ui.language";

    let e = entry(&u, base, key).await;
    assert_eq!(
        (e["value"].clone(), e["source"].clone()),
        (json!("browser"), json!("default"))
    );
    assert_eq!(e["locked"], false);

    // Global.
    assert_eq!(
        put_global(&a, base, key, json!({ "globalValue": "de" }))
            .await
            .0,
        200
    );
    let e = entry(&u, base, key).await;
    assert_eq!(
        (e["value"].clone(), e["source"].clone()),
        (json!("de"), json!("global"))
    );

    // The user's own value.
    let (status, e) = send(
        u.put(format!("{base}/api/settings/{key}"))
            .json(&json!({ "value": "fr" })),
    )
    .await;
    assert_eq!(status, 200, "{e}");
    assert_eq!(
        (e["value"].clone(), e["source"].clone()),
        (json!("fr"), json!("user"))
    );

    // Domain and admin per-user values are broader than the user's.
    let (status, body) = send(
        a.put(format!("{base}/admin/settings/{key}/domains/example.org"))
            .json(&json!({ "value": "it" })),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let (status, body) = send(
        a.put(format!("{base}/admin/settings/{key}/users/{account}"))
            .json(&json!({ "value": "ja" })),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        entry(&u, base, key).await["value"],
        "fr",
        "the user's value is the narrowest"
    );

    // Overrides off: the admin per-user value applies; the user's is reported.
    assert_eq!(
        put_global(&a, base, key, json!({ "allowUserOverride": false }))
            .await
            .0,
        200
    );
    let e = entry(&u, base, key).await;
    assert_eq!(
        (e["value"].clone(), e["source"].clone()),
        (json!("ja"), json!("admin_user"))
    );
    assert_eq!(e["locked"], true);
    assert_eq!(e["userValue"], "fr");

    // Without the per-user value, the domain value.
    let (status, _) = send(a.delete(format!("{base}/admin/settings/{key}/users/{account}"))).await;
    assert_eq!(status, 204);
    assert_eq!(entry(&u, base, key).await["source"], "domain");

    // Overrides back on, the domain locks: the domain value beats the user's.
    assert_eq!(
        put_global(&a, base, key, json!({ "allowUserOverride": true }))
            .await
            .0,
        200
    );
    assert_eq!(entry(&u, base, key).await["value"], "fr");
    let (status, _) = send(
        a.put(format!("{base}/admin/settings/{key}/domains/example.org"))
            .json(&json!({ "value": "it", "locked": true })),
    )
    .await;
    assert_eq!(status, 200);
    let e = entry(&u, base, key).await;
    assert_eq!(
        (e["value"].clone(), e["lockedBy"].clone()),
        (json!("it"), json!("domain"))
    );

    // The global lock beats the domain lock.
    assert_eq!(
        put_global(&a, base, key, json!({ "locked": true })).await.0,
        200
    );
    let e = entry(&u, base, key).await;
    assert_eq!(
        (e["value"].clone(), e["lockedBy"].clone()),
        (json!("de"), json!("global"))
    );

    // DELETE of the policy row: the global value and its lock are gone.
    let (status, item) = send(a.delete(format!("{base}/admin/settings/{key}"))).await;
    assert_eq!(status, 200, "{item}");
    assert_eq!(item["globalValue"], Value::Null);
    assert_eq!(entry(&u, base, key).await["lockedBy"], "domain");
}

// ── enforcement ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_held_key_refuses_the_users_write_and_keeps_the_stored_value() {
    let s = proxy_server("mw-t32-f01-held").await;
    let a = admin(&s.base).await;
    let (u, _) = user(&s.base).await;
    let base = &s.base;
    let key = "offline.budgetMb";

    // Control: the user can write while the key is free.
    let (status, _) = send(
        u.put(format!("{base}/api/settings/{key}"))
            .json(&json!({ "value": 400 })),
    )
    .await;
    assert_eq!(status, 200);

    assert_eq!(
        put_global(&a, base, key, json!({ "globalValue": 100, "locked": true }))
            .await
            .0,
        200
    );
    let (status, body) = send(
        u.put(format!("{base}/api/settings/{key}"))
            .json(&json!({ "value": 900 })),
    )
    .await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "setting_locked");
    let (status, body) = send(u.delete(format!("{base}/api/settings/{key}"))).await;
    assert_eq!(
        status, 200,
        "removing one's own value is always allowed: {body}"
    );

    let e = entry(&u, base, key).await;
    assert_eq!(e["value"], 100);
    assert_eq!(e["locked"], true);
    // The refused write stored nothing: the user value is gone (deleted), not 900.
    assert!(e.get("userValue").is_none(), "{e}");

    // Admin-only keys are not the user's to read or write.
    let (status, _) = send(
        u.put(format!("{base}/api/settings/limit.storage_bytes"))
            .json(&json!({ "value": 0 })),
    )
    .await;
    assert_eq!(status, 404);
    assert!(
        user_settings(&u, base)
            .await
            .get("limit.storage_bytes")
            .is_none()
    );
}

#[tokio::test]
async fn a_held_appearance_field_is_replaced_in_the_object_and_reported() {
    let s = proxy_server("mw-t32-f01-appearance").await;
    let a = admin(&s.base).await;
    let (u, _) = user(&s.base).await;
    let base = &s.base;
    let prefs = json!({
        "mode": "system", "theme": "light", "lightTheme": "light", "darkTheme": "dark",
        "density": "compact", "accentId": "teal", "chosen": true,
    });

    // Control: nothing held, the object is stored as sent.
    let (status, body) = send(
        u.put(format!("{base}/api/account/appearance"))
            .json(&json!({ "appearance": prefs })),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(body["lockedFields"], json!([]));

    assert_eq!(
        put_global(
            &a,
            base,
            "appearance.theme",
            json!({ "globalValue": "hc-dark", "locked": true })
        )
        .await
        .0,
        200
    );
    let (status, body) = send(
        u.put(format!("{base}/api/account/appearance"))
            .json(&json!({ "appearance": prefs })),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        body["lockedFields"],
        json!(["mode", "theme", "lightTheme", "darkTheme"])
    );

    let (_, got) = send(u.get(format!("{base}/api/account/appearance"))).await;
    let stored = &got["appearance"];
    assert_eq!(stored["theme"], "hc-dark");
    assert_eq!(stored["mode"], "fixed");
    assert_eq!(stored["darkTheme"], "hc-dark");
    assert_eq!(stored["density"], "compact", "the free fields are kept");
    assert_eq!(stored["accentId"], "teal");
    assert_eq!(got["deploymentDefault"]["theme"], "hc-dark");
    assert_eq!(
        got["lockedFields"],
        json!(["mode", "theme", "lightTheme", "darkTheme"])
    );

    // /api/settings reads the object's free fields as the user's values, and
    // sends writes of these keys to the appearance route.
    let all = user_settings(&u, base).await;
    assert_eq!(all["appearance.theme"]["locked"], true);
    assert_eq!(all["appearance.density"]["value"], "compact");
    assert_eq!(all["appearance.density"]["source"], "user");
    let (status, body) = send(
        u.put(format!("{base}/api/settings/appearance.density"))
            .json(&json!({ "value": "cozy" })),
    )
    .await;
    assert_eq!(status, 409, "{body}");
    assert_eq!(body["code"], "useDedicatedRoute");
    assert_eq!(body["route"], "/api/account/appearance");
}

/// t40: the unread badge colour follows the accent's path: a default the
/// client adopts, a user choice in the appearance object, and a lock that
/// replaces that choice.
#[tokio::test]
async fn badge_colour_defaults_and_locks_apply_to_the_account_blob() {
    let s = proxy_server("mw-badge-policy").await;
    let a = admin(&s.base).await;
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let (u, _) = user(&s.base).await;
    let base = &s.base;
    assert_eq!(
        put_global(&a, base, "appearance.badgeColor", json!({"globalValue":"teal","locked":false}))
            .await
            .0,
        200
    );
    let (_, initial) = send(u.get(format!("{base}/api/account/appearance"))).await;
    assert_eq!(initial["deploymentDefault"]["badgeColor"], "teal");
    assert_eq!(
        send(u.put(format!("{base}/api/account/appearance")).json(&json!({"appearance":{"badgeColor":"red"}})))
            .await
            .0,
        200
    );
    let all = user_settings(&u, base).await;
    assert_eq!(all["appearance.badgeColor"]["value"], "red");
    assert_eq!(all["appearance.badgeColor"]["source"], "user");
    assert_eq!(
        put_global(&a, base, "appearance.badgeColor", json!({"globalValue":"#ff0000","locked":true}))
            .await
            .0,
        422
    );
    assert_eq!(
        put_global(&a, base, "appearance.badgeColor", json!({"globalValue":"accent","locked":true}))
            .await
            .0,
        200
    );
    let (status, saved) = send(u.put(format!("{base}/api/account/appearance")).json(&json!({"appearance":{"badgeColor":"green"}}))).await;
    assert_eq!(status, 200);
    assert_eq!(saved["lockedFields"], json!(["badgeColor"]));
    let (_, got) = send(u.get(format!("{base}/api/account/appearance"))).await;
    assert_eq!(got["appearance"]["badgeColor"], "accent");
    assert_eq!(
        send(u.put(format!("{base}/api/settings/appearance.badgeColor")).json(&json!({"value":"red"})))
            .await
            .0,
        409
    );
}

#[tokio::test]
async fn font_defaults_and_locks_apply_to_the_account_blob() {
    let s = proxy_server("mw-font-policy").await;
    let a = admin(&s.base).await;
    // Admin and mailbox logins intentionally share a client-address cooldown.
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let (u, _) = user(&s.base).await;
    let base = &s.base;
    assert_eq!(
        put_global(
            &a,
            base,
            "appearance.font",
            json!({"globalValue":"inter","locked":false})
        )
        .await
        .0,
        200
    );
    let (_, initial) = send(u.get(format!("{base}/api/account/appearance"))).await;
    assert_eq!(initial["deploymentDefault"]["font"], "inter");
    assert_eq!(
        send(
            u.put(format!("{base}/api/account/appearance"))
                .json(&json!({"appearance":{"font":"mono"}}))
        )
        .await
        .0,
        200
    );
    let all = user_settings(&u, base).await;
    assert_eq!(all["appearance.font"]["value"], "mono");
    assert_eq!(all["appearance.font"]["source"], "user");
    assert_eq!(
        put_global(
            &a,
            base,
            "appearance.font",
            json!({"globalValue":"external-font-url","locked":true})
        )
        .await
        .0,
        422
    );
    assert_eq!(
        put_global(
            &a,
            base,
            "appearance.font",
            json!({"globalValue":"roboto","locked":true})
        )
        .await
        .0,
        200
    );
    let (status, saved) = send(
        u.put(format!("{base}/api/account/appearance"))
            .json(&json!({"appearance":{"font":"jost"}})),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(saved["lockedFields"], json!(["font"]));
    let (_, got) = send(u.get(format!("{base}/api/account/appearance"))).await;
    assert_eq!(got["appearance"]["font"], "roboto");
    assert_eq!(got["lockedFields"], json!(["font"]));
    assert_eq!(
        send(
            u.put(format!("{base}/api/settings/appearance.font"))
                .json(&json!({"value":"mono"}))
        )
        .await
        .0,
        409
    );
}

#[tokio::test]
async fn a_held_notifications_switch_refuses_a_change() {
    let s = proxy_server("mw-t32-f01-notify").await;
    let a = admin(&s.base).await;
    let (u, _) = user(&s.base).await;
    let base = &s.base;
    let cfg = |enabled: bool| json!({ "enabled": enabled, "rules": [], "quietHours": { "enabled": false, "start": "22:00", "end": "07:00" } });
    assert_eq!(
        send(
            u.put(format!("{base}/api/account/notifications"))
                .json(&cfg(true))
        )
        .await
        .0,
        200
    );

    assert_eq!(
        put_global(
            &a,
            base,
            "notifications.enabled",
            json!({ "globalValue": false, "locked": true })
        )
        .await
        .0,
        200
    );
    let (status, body) = send(
        u.put(format!("{base}/api/account/notifications"))
            .json(&cfg(true)),
    )
    .await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "setting_locked");
    assert_eq!(
        send(
            u.put(format!("{base}/api/account/notifications"))
                .json(&cfg(false))
        )
        .await
        .0,
        200
    );
    let (_, got) = send(u.get(format!("{base}/api/account/notifications"))).await;
    assert_eq!(got["enabled"], false);
    assert_eq!(got["lockedFields"], json!(["enabled"]));
}

#[tokio::test]
async fn a_held_read_receipt_policy_refuses_a_change() {
    // The mock speaks plain POP3; the engine reads MW_ENGINE_TLS per login.
    // SAFETY: the canonical gate runs this binary with --test-threads=1, and no
    // other test here reads this variable.
    unsafe { std::env::set_var("MW_ENGINE_TLS", "plaintext") };
    let pop = MockPop3::start(ENGINE_USER, ENGINE_PASS).await;
    let db = new_db("mw-t32-f01-mdn");
    let app = build_app_full(config(&db, ServerMode::Engine, pop.url()), admin_v6())
        .await
        .expect("engine server boots")
        .0;
    let base = serve(app).await;
    let u = client();
    let r = u
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": ENGINE_USER, "password": ENGINE_PASS }))
        .send()
        .await
        .unwrap();
    let status = r.status();
    assert_eq!(
        status,
        200,
        "engine login: {}",
        r.text().await.unwrap_or_default()
    );
    let a = admin(&base).await;

    let (status, _) = send(
        u.put(format!("{base}/api/account/mdn-policy"))
            .json(&json!({ "policy": "always" })),
    )
    .await;
    assert_eq!(status, 200, "control: the policy is free");

    let (status, _) = send(
        a.put(format!(
            "{base}/admin/settings/mail.readReceipts/domains/example.org"
        ))
        .json(&json!({ "value": "never", "locked": true })),
    )
    .await;
    assert_eq!(status, 200);
    let (status, body) = send(
        u.put(format!("{base}/api/account/mdn-policy"))
            .json(&json!({ "policy": "always" })),
    )
    .await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "setting_locked");
    let (_, got) = send(u.get(format!("{base}/api/account/mdn-policy"))).await;
    assert_eq!(
        (got["policy"].clone(), got["locked"].clone()),
        (json!("never"), json!(true))
    );
    let e = entry(&u, &base, "mail.readReceipts").await;
    assert_eq!(e["value"], "never");
    assert_eq!(
        e["userValue"], "always",
        "the user's stored word is kept, not applied"
    );
}

// ── the admin routes' contract ─────────────────────────────────────────────────

#[tokio::test]
async fn refused_values_name_the_field() {
    let s = proxy_server("mw-t32-f01-invalid").await;
    let a = admin(&s.base).await;
    let (u, _) = user(&s.base).await;
    let base = &s.base;

    let (status, body) = put_global(
        &a,
        base,
        "offline.budgetMb",
        json!({ "globalValue": 99_999 }),
    )
    .await;
    assert_eq!(status, 422, "{body}");
    assert_eq!(
        (body["code"].clone(), body["field"].clone()),
        (json!("invalidValue"), json!("/globalValue"))
    );
    assert!(body["detail"].as_str().unwrap().contains("10240"));

    let (status, body) = put_global(&a, base, "no.such.key", json!({ "globalValue": 1 })).await;
    assert_eq!((status, body["code"].clone()), (404, json!("notFound")));

    let (status, body) = send(
        a.put(format!(
            "{base}/admin/settings/appearance.brandName/domains/example.org"
        ))
        .json(&json!({ "value": "Acme" })),
    )
    .await;
    assert_eq!(
        (status, body["code"].clone()),
        (422, json!("scopeNotAllowed"))
    );

    let (status, body) = put_global(
        &a,
        base,
        "ui.language",
        json!({ "globalValue": "de", "extra": 1 }),
    )
    .await;
    assert_eq!((status, body["code"].clone()), (422, json!("invalidBody")));

    let (status, body) = put_global(
        &a,
        base,
        "limit.send_per_day",
        json!({ "allowUserOverride": true }),
    )
    .await;
    assert_eq!(
        (status, body["field"].clone()),
        (422, json!("/allowUserOverride"))
    );

    let (status, body) = send(
        u.put(format!("{base}/api/settings/ui.language"))
            .json(&json!({ "value": "xx" })),
    )
    .await;
    assert_eq!(status, 422, "{body}");
    assert_eq!(body["field"], "/value");
    assert!(body.get("detail").is_none() || body["detail"].is_string());

    // Control: a valid value is stored.
    assert_eq!(
        put_global(&a, base, "offline.budgetMb", json!({ "globalValue": 1024 }))
            .await
            .0,
        200
    );
}

#[tokio::test]
async fn signin_keys_are_written_through_their_own_route() {
    let s = proxy_server("mw-t32-f01-dedicated").await;
    let a = admin(&s.base).await;
    for (url, body) in [
        (
            format!("{}/admin/settings/signin.newUsers", s.base),
            json!({ "globalValue": "closed" }),
        ),
        (
            format!(
                "{}/admin/settings/signin.newUsers/domains/example.org",
                s.base
            ),
            json!({ "value": "closed" }),
        ),
        (
            format!("{}/admin/settings/signin.refusalDetail", s.base),
            json!({ "globalValue": "uniform" }),
        ),
    ] {
        let (status, body) = send(a.put(url.clone()).json(&body)).await;
        assert_eq!(status, 403, "{url}: {body}");
        assert_eq!(body["code"], "useDedicatedRoute");
        assert!(
            body["route"].as_str().unwrap().ends_with("/signin-policy"),
            "{body}"
        );
    }
    let (status, _) = send(a.delete(format!("{}/admin/settings/signin.domainMode", s.base))).await;
    assert_eq!(status, 403);
    // Control: a key without a dedicated route takes the same request.
    assert_eq!(
        put_global(
            &a,
            &s.base,
            "usage.meterApi",
            json!({ "globalValue": false })
        )
        .await
        .0,
        200
    );
}

#[tokio::test]
async fn a_pinned_key_is_refused_with_its_variable() {
    let s = proxy_server("mw-t32-f01-pin").await;
    let a = admin(&s.base).await;
    let key = "realtime.recheckSecs";
    // Control first: unpinned, the write goes through.
    assert_eq!(
        put_global(&a, &s.base, key, json!({ "globalValue": 60 }))
            .await
            .0,
        200
    );

    // SAFETY: the canonical gate runs this binary with --test-threads=1, and no
    // other test here reads this variable.
    unsafe { std::env::set_var("MW_REALTIME_RECHECK_SECS", "45") };
    let (status, body) = put_global(&a, &s.base, key, json!({ "globalValue": 90 })).await;
    let (_, item) = send(a.get(format!("{}/admin/settings/{key}", s.base))).await;
    unsafe { std::env::remove_var("MW_REALTIME_RECHECK_SECS") };

    assert_eq!(status, 409, "{body}");
    assert_eq!(
        (body["code"].clone(), body["var"].clone()),
        (json!("pinnedByEnv"), json!("MW_REALTIME_RECHECK_SECS"))
    );
    assert_eq!(item["envState"]["set"], true);
    assert_eq!(item["effective"]["source"], "envPin");
    assert_eq!(item["effective"]["value"], 45);
    assert_eq!(
        item["globalValue"], 60,
        "the stored value is kept for when the variable is unset"
    );
}

#[tokio::test]
async fn a_stale_if_match_is_refused_with_who_and_when() {
    let s = proxy_server("mw-t32-f01-etag").await;
    let a = admin(&s.base).await;
    let url = format!("{}/admin/settings/ui.language", s.base);
    let r = a.get(&url).send().await.unwrap();
    let etag0 = r.headers()["etag"].to_str().unwrap().to_string();

    let r = a
        .put(&url)
        .header("if-match", &etag0)
        .json(&json!({ "globalValue": "de" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "a current If-Match is accepted");
    let etag1 = r.headers()["etag"].to_str().unwrap().to_string();
    assert_ne!(etag0, etag1);

    let (status, body) = send(
        a.put(&url)
            .header("if-match", &etag0)
            .json(&json!({ "globalValue": "fr" })),
    )
    .await;
    assert_eq!(status, 412, "{body}");
    assert_eq!(body["code"], "stale");
    assert_eq!(body["updatedBy"], "root");
    assert!(body["updatedAt"].is_string());
    let (_, item) = send(a.get(&url)).await;
    assert_eq!(item["globalValue"], "de", "the stale write changed nothing");

    // Domain rows carry their own tag.
    let durl = format!("{}/admin/settings/ui.language/domains/example.org", s.base);
    let r = a
        .put(&durl)
        .json(&json!({ "value": "it" }))
        .send()
        .await
        .unwrap();
    let detag = r.headers()["etag"].to_str().unwrap().to_string();
    assert_eq!(
        a.put(&durl)
            .header("if-match", &detag)
            .json(&json!({ "value": "es" }))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        a.put(&durl)
            .header("if-match", &detag)
            .json(&json!({ "value": "pl" }))
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
}

#[tokio::test]
async fn a_lock_over_existing_values_previews_its_impact_without_writing() {
    let (s, _pop, u, account) = engine_user("mw-t32-f01-impact").await;
    let a = admin(&s.base).await;
    let base = &s.base;
    let key = "offline.retentionDays";
    assert_eq!(
        send(
            u.put(format!("{base}/api/settings/{key}"))
                .json(&json!({ "value": 7 }))
        )
        .await
        .0,
        200
    );

    let (status, body) = put_global(&a, base, key, json!({ "locked": true, "dryRun": true })).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["dryRun"], true);
    assert_eq!(body["impact"]["userValues"]["count"], 1);
    assert_eq!(body["impact"]["userValues"]["names"][0], json!(account));
    let (_, item) = send(a.get(format!("{base}/admin/settings/{key}"))).await;
    assert_eq!(item["globalLocked"], false, "a dry run writes nothing");
    assert_eq!(entry(&u, base, key).await["value"], 7);

    // Without a narrower value there is nothing to report.
    let (_, body) = put_global(
        &a,
        base,
        "offline.budgetMb",
        json!({ "locked": true, "dryRun": true }),
    )
    .await;
    assert_eq!(body["impact"], json!({}));
}

#[tokio::test]
async fn every_write_is_audited_with_the_setting_as_target() {
    let s = proxy_server("mw-t32-f01-audit").await;
    let a = admin(&s.base).await;
    let base = &s.base;
    assert_eq!(
        put_global(&a, base, "ui.language", json!({ "globalValue": "de" }))
            .await
            .0,
        200
    );
    assert_eq!(
        send(
            a.put(format!(
                "{base}/admin/settings/ui.language/domains/example.org"
            ))
            .json(&json!({ "value": "it" }))
        )
        .await
        .0,
        200
    );
    assert_eq!(
        send(
            a.put(format!(
                "{base}/admin/settings/limit.storage_bytes/users/TestUser@Example.org"
            ))
            .json(&json!({ "value": 1000 }))
        )
        .await
        .0,
        200
    );
    // A refused write is not audited.
    assert_eq!(
        put_global(&a, base, "ui.language", json!({ "globalValue": "xx" }))
            .await
            .0,
        422
    );

    let (_, audit) = send(a.get(format!("{base}/admin/audit?limit=50"))).await;
    let rows: Vec<&Value> = audit
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["action"] == "setting-changed")
        .collect();
    assert_eq!(rows.len(), 3, "{audit}");
    assert!(rows.iter().all(|r| r["actor"] == "root"));
    let targets: Vec<&str> = rows.iter().map(|r| r["target"].as_str().unwrap()).collect();
    assert_eq!(
        targets
            .iter()
            .filter(|t| **t == "setting:ui.language")
            .count(),
        2
    );
    assert!(targets.contains(&"setting:limit.storage_bytes"));
    let global: Value = serde_json::from_str(
        rows.iter()
            .find(|r| r["detailJson"].as_str().unwrap().contains("\"global\""))
            .unwrap()["detailJson"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(global["before"], Value::Null);
    assert_eq!(global["after"]["globalValue"], "de");

    // Per-user limit rows are keyed by the canonical address (t32-q A15).
    let (_, values) = send(a.get(format!(
        "{base}/admin/settings/limit.storage_bytes/values?scope=admin_user"
    )))
    .await;
    assert_eq!(values["values"][0]["scopeId"], "testuser@example.org");
    assert_eq!(values["values"][0]["value"], 1000);
}

// ── deployment appearance ──────────────────────────────────────────────────────

#[tokio::test]
async fn the_deployment_appearance_is_stored_and_served() {
    let s = proxy_server("mw-t32-f01-deploy").await;
    let a = admin(&s.base).await;
    let base = &s.base;

    let (_, got) = send(a.get(format!("{base}/admin/appearance"))).await;
    assert_eq!(
        got["theme"], "dark",
        "the built-in default is the dark pack"
    );

    let (status, body) = send(
        a.put(format!("{base}/admin/appearance"))
            .json(&json!({ "theme": "grove-dark", "brandName": "Acme Mail", "accent": "teal" })),
    )
    .await;
    assert_eq!(status, 204, "{body}");
    let (status, body) = send(
        a.put(format!("{base}/admin/appearance"))
            .json(&json!({ "theme": "Not A Theme", "brandName": "Acme Mail", "accent": null })),
    )
    .await;
    assert_eq!(status, 422, "{body}");

    let (_, item) = send(a.get(format!("{base}/admin/settings/appearance.theme"))).await;
    assert_eq!(
        item["globalValue"], "grove-dark",
        "the compatibility view writes the settings key"
    );

    // A user with no appearance of their own is served it.
    let (u, _) = user(base).await;
    let (_, mine) = send(u.get(format!("{base}/api/account/appearance"))).await;
    assert_eq!(mine["appearance"], Value::Null);
    assert_eq!(
        mine["deploymentDefault"],
        // The font is served beside the rest since 3b14e7ea, the badge colour
        // since t40; neither was set.
        json!({ "theme": "grove-dark", "font": "default", "badgeColor": "default", "brandName": "Acme Mail", "accent": "teal" })
    );

    // A second server process on the same database (a restart) serves it too.
    let again = proxy_server_on(s.db.clone(), s.upstream.clone()).await;
    let a2 = admin(&again.base).await;
    let (_, got) = send(a2.get(format!("{}/admin/appearance", again.base))).await;
    assert_eq!(got["theme"], "grove-dark");
    assert_eq!(got["brandName"], "Acme Mail");
}
