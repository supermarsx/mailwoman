//! t11-e3 live-E2E — DCR admin enable route (`GET/PUT /admin/oauth-dcr`) end-to-end.
//!
//! Proves the 26.11 slice `e7503df` (admin GET/PUT `/admin/oauth-dcr`) is WIRED and
//! COMPOSES with the DCR registration surface: against a REAL running `build_app_full`
//! server, an admin flips the `oauth_dcr` policy through the mounted admin route and the
//! public `POST /oauth/register` transitions accordingly:
//!   * unauth / bogus-cookie GET+PUT → 401 (fail-closed); a rejected PUT never enables DCR;
//!   * admin logs in (`POST /admin/login`, root/hunter2) → `mw_admin_session` cookie;
//!   * default policy → `/oauth/register` is 403;
//!   * admin PUT `enabled:true` (+ allowlist + defaultScope) → 200; admin GET reflects it;
//!   * `/oauth/register` transitions 403 → 201 (allowlisted redirect);
//!   * admin PUT `enabled:false` → 200; `/oauth/register` returns to 403.
//!
//! ## Postgres path (the V6 lesson — plan §5)
//! Runs against **SQLite by default** AND, when `MW_E14_PG_DSN` (or `DATABASE_URL_PG`) is
//! set, against a **live Postgres** — the INTEGER-vs-BOOLEAN `oauth_dcr.enabled` bind that
//! only misbehaves on real Postgres is exercised there. The scenario is identical on both
//! backends; only the DSN changes.
//!
//!   docker compose -f docker-compose.ci.yml up -d --wait postgres
//!   MW_E14_PG_DSN=postgres://mailwoman:mailwoman@localhost:5432/mailwoman \
//!     cargo test -p mw-server --test t11_dcr_admin -- --nocapture

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";
const SERVER_KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const ADMIN_USER: &str = "root";
const ADMIN_PASS: &str = "hunter2";
const ADMIN_COOKIE: &str = "mw_admin_session";

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}_{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

/// The db_path under test: the live Postgres DSN when set (the V6 lesson), else a fresh
/// temp SQLite file (default gate). Returns `(db_path, on_postgres)`.
fn db_path() -> (String, bool) {
    if let Some(dsn) = std::env::var("MW_E14_PG_DSN")
        .ok()
        .or_else(|| std::env::var("DATABASE_URL_PG").ok())
        .filter(|s| !s.is_empty())
    {
        eprintln!("[t11-e3 dcr] running against LIVE Postgres (the V6 bool-bind lesson)");
        (dsn, true)
    } else {
        eprintln!(
            "[t11-e3 dcr] MW_E14_PG_DSN unset — running on SQLite only. Set it (bring up \
             docker-compose.ci.yml postgres) to also exercise the Postgres path."
        );
        let p = std::env::temp_dir().join(format!("mw-t11e3-dcr-{}.db", unique()));
        (p.to_string_lossy().into_owned(), false)
    }
}

fn web_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mw-t11e3-dcr-web-{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), INDEX_HTML).unwrap();
    dir
}

async fn spawn(db_path: String) -> String {
    let config = AppConfig {
        db_path,
        server_key_hex: Some(SERVER_KEY_HEX.into()),
        web_dir: Some(web_dir()),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some(ADMIN_USER.into()),
        admin_password: Some(ADMIN_PASS.into()),
        redis_url: None,
    };
    let app = build_app_full(config, v6).await.expect("server boots").0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().build().unwrap()
}

/// Log in through the REAL admin login route and return the `mw_admin_session` cookie
/// value (extracted from `Set-Cookie`) as a ready-to-send `Cookie` header.
async fn admin_login(c: &reqwest::Client, base: &str) -> String {
    let resp = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": ADMIN_USER, "password": ADMIN_PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "admin login succeeds with valid creds");
    let set_cookie = resp
        .headers()
        .get(reqwest::header::SET_COOKIE)
        .expect("login sets a session cookie")
        .to_str()
        .unwrap();
    let pair = set_cookie
        .split(';')
        .next()
        .expect("cookie name=value")
        .to_string();
    assert!(
        pair.starts_with(&format!("{ADMIN_COOKIE}=")),
        "the session cookie is {ADMIN_COOKIE}: {pair}"
    );
    pair
}

async fn register_status(c: &reqwest::Client, base: &str, redirect: &str) -> u16 {
    c.post(format!("{base}/oauth/register"))
        .json(&json!({ "redirect_uris": [redirect] }))
        .send()
        .await
        .unwrap()
        .status()
        .as_u16()
}

/// The whole admin-toggle lifecycle, end-to-end against the running server + the db under
/// test (SQLite default; live Postgres when `MW_E14_PG_DSN` is set).
#[tokio::test]
async fn dcr_admin_toggle_drives_registration_end_to_end() {
    let (db, on_pg) = db_path();
    let base = spawn(db).await;
    let c = client();
    let redirect = "https://apps.vogue-homes.com/cb";

    // ── 1. Fail-closed: unauth + bogus-cookie GET/PUT → 401 (never enables DCR). ──
    let no_cookie_get = c
        .get(format!("{base}/admin/oauth-dcr"))
        .send()
        .await
        .unwrap();
    assert_eq!(no_cookie_get.status(), 401, "GET requires an admin session");

    let no_cookie_put = c
        .put(format!("{base}/admin/oauth-dcr"))
        .json(&json!({ "enabled": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        no_cookie_put.status(),
        401,
        "unauth PUT is rejected (must not enable DCR)"
    );

    let bogus = c
        .get(format!("{base}/admin/oauth-dcr"))
        .header(
            reqwest::header::COOKIE,
            format!("{ADMIN_COOKIE}=not-a-session"),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(bogus.status(), 401, "an unknown admin token is rejected");

    // The rejected PUT never turned DCR on: register is still 403.
    assert_eq!(
        register_status(&c, &base, redirect).await,
        403,
        "DCR stays default-disabled after a rejected unauth PUT (on_pg={on_pg})"
    );

    // ── 2. Admin logs in → session cookie. ────────────────────────────────────────
    let cookie = admin_login(&c, &base).await;

    // Default (pre-enable) admin GET renders the DISABLED shape.
    let pre: Value = c
        .get(format!("{base}/admin/oauth-dcr"))
        .header(reqwest::header::COOKIE, &cookie)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(pre["enabled"], false, "policy is default-disabled");

    // ── 3. Admin PUT enables + sets the allowlist + a default scope → 200. ─────────
    let put = c
        .put(format!("{base}/admin/oauth-dcr"))
        .header(reqwest::header::COOKIE, &cookie)
        .json(&json!({
            "enabled": true,
            "allowedRedirectHostSuffixes": ["vogue-homes.com"],
            "defaultScope": { "read": true },
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(put.status(), 200, "admin enable succeeds");
    let put_body: Value = put.json().await.unwrap();
    assert_eq!(put_body["enabled"], true);

    // ── 4. Admin GET reflects the persisted policy. ───────────────────────────────
    let got: Value = c
        .get(format!("{base}/admin/oauth-dcr"))
        .header(reqwest::header::COOKIE, &cookie)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        got["enabled"], true,
        "GET reflects the enable (on_pg={on_pg})"
    );
    assert_eq!(got["allowedRedirectHostSuffixes"][0], "vogue-homes.com");
    assert_eq!(got["defaultScope"]["read"], true);

    // ── 5. Register transitions 403 → 201 (an allowlisted redirect). ──────────────
    let created = c
        .post(format!("{base}/oauth/register"))
        .json(&json!({ "redirect_uris": [redirect], "client_name": "t11-e3 client" }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        created.status(),
        201,
        "enabling DCR admits registration (on_pg={on_pg})"
    );
    let reg: Value = created.json().await.unwrap();
    assert!(
        reg["client_id"].as_str().is_some(),
        "a client_id was minted: {reg}"
    );

    // ── 6. Admin PUT disable → register returns to 403. ───────────────────────────
    let off = c
        .put(format!("{base}/admin/oauth-dcr"))
        .header(reqwest::header::COOKIE, &cookie)
        .json(&json!({ "enabled": false }))
        .send()
        .await
        .unwrap();
    assert_eq!(off.status(), 200, "admin disable succeeds");

    assert_eq!(
        register_status(&c, &base, redirect).await,
        403,
        "disabling DCR returns register to 403 (on_pg={on_pg})"
    );
}
