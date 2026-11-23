//! The cookie inventory against the cookies the running server sets.
//!
//! (a) Drives the real flows that set cookies (mailbox sign-in, session refresh, remembered device,
//! admin sign-in, OIDC authorize, the client-access check), collects every `Set-Cookie` the server
//! sends, and requires each name to match an entry of the inventory the server publishes at
//! `/api/cookies/config`, with the same `HttpOnly` flag. The oracle is the server's own output, so a
//! cookie nobody listed fails the test; the inventory is not read from source.
//!
//! (b) Parses the cookie table of `docs/admin/cookies.md` and requires it to list the same cookies
//! with the same category, purpose, lifetime text and `HttpOnly` flag as the inventory.
//!
//! SQLite only. The admin cookie name is process-global, so the tests share a lock.

use std::collections::BTreeMap;
use std::net::SocketAddr;

use base64::Engine as _;
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{ServerKey, Store, oidc_provider::OidcClientRow, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

mod common;

static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
const PSK: &str = "entry-test-secret-2468";

async fn serve(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap()
    });
    format!("http://{addr}")
}

fn browser() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// One `Set-Cookie` header, split into the name, its attributes and the raw line.
#[derive(Debug, Clone)]
struct SetCookie {
    name: String,
    value: String,
    http_only: bool,
    max_age: Option<i64>,
    raw: String,
}

fn parse(raw: &str) -> SetCookie {
    let mut parts = raw.split(';').map(str::trim);
    let (name, value) = parts.next().unwrap().split_once('=').unwrap_or((raw, ""));
    let mut http_only = false;
    let mut max_age = None;
    for attr in parts {
        let lower = attr.to_ascii_lowercase();
        if lower == "httponly" {
            http_only = true;
        } else if let Some(v) = lower.strip_prefix("max-age=") {
            max_age = v.parse().ok();
        }
    }
    SetCookie { name: name.to_owned(), value: value.to_owned(), http_only, max_age, raw: raw.to_owned() }
}

/// Every cookie the server set, in order, across all the flows.
#[derive(Default)]
struct Jar(Vec<SetCookie>);

impl Jar {
    fn take(&mut self, response: &reqwest::Response) {
        for value in response.headers().get_all(reqwest::header::SET_COOKIE) {
            self.0.push(parse(value.to_str().unwrap()));
        }
    }
    fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.0.iter().map(|c| c.name.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        names
    }
    fn first(&self, name: &str) -> &SetCookie {
        self.0.iter().find(|c| c.name == name && c.max_age != Some(0)).unwrap_or_else(|| panic!("{name} was not set"))
    }
}

struct Fixture {
    base: String,
    store: Store,
    upstream: String,
    dir: std::path::PathBuf,
}

async fn boot() -> Fixture {
    let upstream = serve(mw_mock_jmap::router()).await;
    let dir = common::test_db::unique_dir("t48-cookies");
    let web = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), "<!doctype html><title>Mail shell</title><main>MAIL_SHELL</main>").unwrap();
    let key = ServerKey::generate();
    let db = dir.join("store.db");
    let store = Store::open(db.to_str().unwrap(), key.clone()).await.unwrap();
    for (setting, value) in [("signin.retryCooldownSecs", "0"), ("cookies.enabled", "true"), ("twofa.rememberDays", "7")] {
        store
            .put_setting_policy(&SettingPolicyRow {
                key: setting.into(),
                global_value: Some(value.into()),
                allow_user_override: false,
                global_locked: false,
                updated_by: "fixture".into(),
                updated_at: 0,
            })
            .await
            .unwrap();
    }
    let issuer = "https://issuer.example/oidc";
    store.put_oidc_provider_config(issuer, true, None, 1).await.unwrap();
    let (pem, jwk) = mw_sso::issuer::generate_rs256_key("k").unwrap();
    store.rotate_oidc_signing_key("k", issuer, pem.as_bytes(), &jwk, 1).await.unwrap();
    store
        .create_oidc_client(
            &OidcClientRow {
                client_id: "rp".into(),
                config: json!({"redirectUris":["https://rp.example/callback"],"postLogoutRedirectUris":[],"scopes":["openid","email"],"authMethod":"none"}),
                enabled: true,
                revision: String::new(),
                updated_at: 1,
                secret_hash: None,
            },
            "Test relying party",
            "fixture",
        )
        .await
        .unwrap();
    let app = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(key.to_hex()),
            web_dir: Some(web),
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(upstream.clone()),
                jmap_upstreams: Some(vec![upstream.clone()]),
                ..Default::default()
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
    .unwrap()
    .0;
    let base = serve(app).await;
    Fixture { base, store, upstream, dir }
}

/// The inventory entries the server publishes (only the cookie ones when `cookies_only`).
async fn published(base: &str) -> Vec<Value> {
    let body: Value = reqwest::Client::new()
        .get(format!("{base}/api/cookies/config"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    body["inventory"].as_array().unwrap_or_else(|| panic!("no inventory in {body}")).clone()
}

fn glob(pattern: &str, name: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == name,
        Some((head, tail)) => name.len() >= head.len() + tail.len() && name.starts_with(head) && name.ends_with(tail),
    }
}

fn entry_for<'a>(inventory: &'a [Value], name: &str) -> Option<&'a Value> {
    let cookies = || inventory.iter().filter(|e| e["kind"] == "cookie");
    cookies()
        .find(|e| e["pattern"] == name)
        .or_else(|| cookies().find(|e| glob(e["pattern"].as_str().unwrap(), name)))
}

/// Sign the admin in through the real route and return its CSRF token.
async fn admin_sign_in(base: &str, client: &reqwest::Client, jar: &mut Jar) -> String {
    let response = client
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":"root","password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    jar.take(&response);
    response.json::<Value>().await.unwrap()["csrfToken"].as_str().unwrap().to_owned()
}

async fn mailbox_flows(f: &Fixture, jar: &mut Jar) -> reqwest::Client {
    let client = browser();
    let login = client
        .post(format!("{}/api/login", f.base))
        .json(&json!({"username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200);
    jar.take(&login);
    let csrf = login.json::<Value>().await.unwrap()["csrfToken"].as_str().unwrap().to_owned();
    // A plain authenticated read sets the CSRF cookie again without an expiry when it is missing; the
    // refresh below sets it with one.
    let me = client.get(format!("{}/api/me", f.base)).send().await.unwrap();
    jar.take(&me);
    common::auth::user_step_up_with(&client, &f.base, mw_mock_jmap::PASS, &[("x-csrf-token", &csrf)]).await;
    let remembered = client
        .post(format!("{}/api/account/security/remembered-devices", f.base))
        .header("x-csrf-token", &csrf)
        .json(&json!({"label":"t48 browser"}))
        .send()
        .await
        .unwrap();
    assert_eq!(remembered.status(), 200, "remember a device");
    jar.take(&remembered);
    let refreshed = client
        .post(format!("{}/api/account/session/activity", f.base))
        .header("x-csrf-token", &csrf)
        .send()
        .await
        .unwrap();
    assert_eq!(refreshed.status(), 200);
    jar.take(&refreshed);
    client
}

async fn oidc_authorize(f: &Fixture, client: &reqwest::Client, jar: &mut Jar) {
    let verifier = "a".repeat(43);
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let response = client
        .get(format!("{}/oidc/authorize", f.base))
        .query(&[
            ("client_id", "rp"),
            ("redirect_uri", "https://rp.example/callback"),
            ("response_type", "code"),
            ("scope", "openid email"),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("state", "s"),
            ("nonce", "n"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 303);
    jar.take(&response);
}

/// Require the entire gate: set a pre-shared key as the admin, then pass the check as a guest.
async fn client_access_flows(f: &Fixture, admin: &reqwest::Client, csrf: &str, jar: &mut Jar) {
    let base = &f.base;
    // A granted step-up replaces the admin session, so the CSRF token changes.
    let stepped = admin
        .post(format!("{base}/admin/step-up"))
        .header("x-csrf-token", csrf)
        .json(&json!({"password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(stepped.status(), 200);
    let csrf = stepped
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find_map(|v| v.strip_prefix("mw_csrf="))
        .map(|v| v.split(';').next().unwrap().to_owned())
        .expect("step-up reissues mw_csrf");
    jar.take(&stepped);
    let current = admin.get(format!("{base}/admin/client-access")).send().await.unwrap();
    let etag = current.headers()["etag"].clone();
    let saved = admin
        .put(format!("{base}/admin/client-access"))
        .header("if-match", etag)
        .header("x-csrf-token", csrf)
        .json(&json!({"pskEnabled":true,"slugEnabled":false,"slug":null,"grantHours":24,"psk":PSK}))
        .send()
        .await
        .unwrap();
    assert_eq!(saved.status(), 200);
    let guest = browser();
    let page = guest.get(format!("{base}/anywhere")).header("accept", "text/html").send().await.unwrap();
    jar.take(&page);
    let html = page.text().await.unwrap();
    let nonce = html.split("name=\"nonce\" value=\"").nth(1).unwrap().split('"').next().unwrap().to_owned();
    let unlocked = guest
        .post(format!("{base}/api/client-access/unlock"))
        .form(&[("psk", PSK), ("nonce", &nonce)])
        .send()
        .await
        .unwrap();
    assert_eq!(unlocked.status(), 303);
    jar.take(&unlocked);
}

/// The flows' fixed-lifetime cookies carry the `Max-Age` their inventory text promises.
fn check_lifetimes(jar: &Jar, inventory: &[Value]) {
    for (name, max_age) in [
        ("mw_browser_device", 365 * 86_400),
        ("mw_admin_browser", 400 * 86_400),
        ("mw_oidc_flow", 600),
        ("mw_remembered_device", 7 * 86_400),
    ] {
        assert_eq!(jar.first(name).max_age, Some(max_age), "{name} Max-Age");
    }
    let admin = inventory.iter().find(|e| e["id"] == "admin-session").unwrap();
    let admin_name = admin["pattern"].as_str().unwrap();
    assert_eq!(jar.first(admin_name).max_age, Some(43_200), "the admin cookie carries admin.session.absoluteSecs");
    for name in ["mw_session", "mw_client_access", "mw_client_access_pending"] {
        assert!(jar.first(name).max_age.is_some_and(|v| v > 0), "{name} is set with a Max-Age: {}", jar.first(name).raw);
    }
    let texts: BTreeMap<&str, &str> = ["mw_browser_device", "mw_admin_browser", "mw_oidc_flow"]
        .iter()
        .map(|n| (*n, entry_for(inventory, n).unwrap()["duration"].as_str().unwrap()))
        .collect();
    assert_eq!(texts["mw_browser_device"], "365 days");
    assert_eq!(texts["mw_admin_browser"], "400 days");
    assert_eq!(texts["mw_oidc_flow"], "10 minutes");
}

fn check_against_inventory(jar: &Jar, inventory: &[Value]) {
    let mut unlisted = Vec::new();
    let mut wrong_flag = Vec::new();
    for cookie in &jar.0 {
        match entry_for(inventory, &cookie.name) {
            None => unlisted.push(cookie.name.clone()),
            Some(entry) if entry["httpOnly"] != json!(cookie.http_only) => {
                wrong_flag.push(format!("{} (set: {}, inventory: {})", cookie.name, cookie.http_only, entry["httpOnly"]));
            }
            Some(_) => {}
        }
    }
    unlisted.sort();
    unlisted.dedup();
    assert!(unlisted.is_empty(), "cookies the server sets that the inventory does not list: {unlisted:?}");
    assert!(wrong_flag.is_empty(), "HttpOnly differs from the inventory: {wrong_flag:?}");
}

const EXPECTED: [&str; 9] = [
    "mw_admin_browser",
    "mw_admin_session",
    "mw_browser_device",
    "mw_client_access",
    "mw_client_access_pending",
    "mw_csrf",
    "mw_oidc_flow",
    "mw_remembered_device",
    "mw_session",
];

fn assert_flows_ran(jar: &Jar, admin_name: &str) {
    let seen = jar.names();
    for name in EXPECTED.iter().map(|n| if *n == "mw_admin_session" { admin_name } else { n }) {
        assert!(seen.contains(&name), "the flows never set {name}; saw {seen:?}");
    }
    assert!(
        !jar.first("mw_csrf").http_only && jar.0.iter().filter(|c| c.name == "mw_csrf").any(|c| c.max_age.is_some()),
        "mw_csrf is readable and is set with a Max-Age at sign-in"
    );
    assert!(
        jar.0.iter().filter(|c| c.name == "mw_csrf").any(|c| c.max_age.is_none()),
        "mw_csrf is also set without an expiry (GET /api/me, admin step-up)"
    );
    assert!(!jar.0.iter().any(|c| c.value.is_empty() && c.max_age != Some(0)), "an empty cookie without Max-Age=0");
}

#[tokio::test]
async fn every_cookie_the_server_sets_is_in_the_inventory() {
    let _serial = LOCK.lock().await;
    let f = boot().await;
    let inventory = published(&f.base).await;
    let mut jar = Jar::default();

    let user = mailbox_flows(&f, &mut jar).await;
    oidc_authorize(&f, &user, &mut jar).await;
    let admin = browser();
    let csrf = admin_sign_in(&f.base, &admin, &mut jar).await;
    client_access_flows(&f, &admin, &csrf, &mut jar).await;

    assert_flows_ran(&jar, "mw_admin_session");
    check_against_inventory(&jar, &inventory);
    check_lifetimes(&jar, &inventory);
    let _ = (&f.store, &f.upstream, &f.dir);
}

#[tokio::test]
async fn a_renamed_admin_cookie_still_matches_its_entry() {
    let _serial = LOCK.lock().await;
    let f = boot().await;
    // The name is applied at start from MW_ADMIN_SESSION_COOKIE; set it as that start does.
    mw_admin::config::set_session_cookie_name(Some("corp_admin"));
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            mw_admin::config::set_session_cookie_name(None);
        }
    }
    let _reset = Reset;
    let inventory = published(&f.base).await;
    let mut jar = Jar::default();
    admin_sign_in(&f.base, &browser(), &mut jar).await;
    assert!(jar.names().contains(&"corp_admin"), "the console set {:?}", jar.names());
    assert!(!jar.names().contains(&"mw_admin_session"));
    check_against_inventory(&jar, &inventory);
    assert_eq!(entry_for(&inventory, "corp_admin").unwrap()["id"], "admin-session");
}

/// The rows of the first table in `docs/admin/cookies.md` whose header starts with `Cookie`.
fn docs_rows() -> Vec<Vec<String>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/admin/cookies.md");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut rows = Vec::new();
    let mut in_table = false;
    for line in text.lines() {
        let cells: Vec<String> = line
            .trim()
            .trim_matches('|')
            .split(" | ")
            .map(|c| c.trim().trim_matches('`').to_owned())
            .collect();
        if !in_table {
            in_table = line.trim_start().starts_with("| Cookie |");
            continue;
        }
        if !line.trim_start().starts_with('|') {
            break;
        }
        if cells[0].starts_with("---") {
            continue;
        }
        rows.push(cells);
    }
    assert!(!rows.is_empty(), "docs/admin/cookies.md has no table headed `| Cookie |`");
    rows
}

#[tokio::test]
async fn the_docs_cookie_table_matches_the_inventory() {
    let _serial = LOCK.lock().await;
    let f = boot().await;
    let inventory = published(&f.base).await;
    let served: BTreeMap<String, Vec<String>> = inventory
        .iter()
        .filter(|e| e["kind"] == "cookie")
        .map(|e| {
            (
                e["pattern"].as_str().unwrap().to_owned(),
                vec![
                    e["category"].as_str().unwrap().to_owned(),
                    e["purpose"].as_str().unwrap().to_owned(),
                    e["duration"].as_str().unwrap().to_owned(),
                    if e["httpOnly"] == json!(true) { "Yes" } else { "No" }.to_owned(),
                ],
            )
        })
        .collect();
    let documented: BTreeMap<String, Vec<String>> = docs_rows()
        .into_iter()
        .map(|mut row| {
            assert_eq!(row.len(), 5, "a cookie row has name, category, purpose, lifetime and HttpOnly: {row:?}");
            let name = row.remove(0);
            // The docs write the category capitalised.
            row[0] = row[0].to_lowercase();
            (name, row)
        })
        .collect();
    assert_eq!(
        documented.keys().collect::<Vec<_>>(),
        served.keys().collect::<Vec<_>>(),
        "the documented cookie names differ from the inventory"
    );
    for (name, row) in &served {
        assert_eq!(&documented[name], row, "docs row of {name} differs from the inventory (category, purpose, lifetime, HttpOnly)");
    }
}
