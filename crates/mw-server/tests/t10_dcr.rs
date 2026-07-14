//! t10-e14 backend live-E2E — OAuth Dynamic Client Registration (RFC 7591/7592).
//!
//! Drives the REAL mounted `/oauth/register*` routes against a REAL running mw-server
//! (`build_app_full` → `axum::serve` on a live TCP socket, reqwest over the wire — the
//! same AuthServer that issues authorization codes). Proves the whole policy gate:
//!   * default-DISABLED  → `POST /oauth/register` is `403` before any policy is enabled;
//!   * enabled           → `201` + a one-time registration-access-token + client_id;
//!   * RFC 7592 lifecycle → `GET/PUT/DELETE /oauth/register/{id}` authed by that token;
//!   * redirect allowlist → a redirect host OUTSIDE the suffix allowlist is `400`;
//!   * no scope escalation → the granted scope is the policy default, never the request's.
//!
//! ## Postgres path (the V6 lesson — plan §5)
//! Runs against **SQLite by default** (so it's in the default `cargo test` gate) AND,
//! when `MW_E14_PG_DSN` (or `DATABASE_URL_PG`) is set, against a **live Postgres** — the
//! bool-bind + parameter-cast bugs that only surface on real Postgres (the V6
//! `oauth_dcr.enabled` / `require_initial_access_token` INTEGER-vs-BOOLEAN invariant) are
//! exercised there. The scenario is identical on both backends; only the DSN changes.
//!
//!   docker compose -f docker-compose.ci.yml up -d --wait postgres
//!   MW_E14_PG_DSN=postgres://mailwoman:mailwoman@localhost:5432/mailwoman \
//!     cargo test -p mw-server --test t10_dcr -- --nocapture

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{
    AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full,
};
use mw_store::{OAuthDcrPolicyRow, ServerKey, Store};

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";
// Fixed key so the seeding Store and the server share the sealed-column key.
const SERVER_KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

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
        eprintln!("[t10-e14 dcr] running against LIVE Postgres (the V6 bool-bind lesson)");
        (dsn, true)
    } else {
        eprintln!(
            "[t10-e14 dcr] MW_E14_PG_DSN unset — running on SQLite only. Set it (bring up \
             docker-compose.ci.yml postgres) to also exercise the Postgres path."
        );
        let p = std::env::temp_dir().join(format!("mw-e14-dcr-{}.db", unique()));
        (p.to_string_lossy().into_owned(), false)
    }
}

fn web_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mw-e14-dcr-web-{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), INDEX_HTML).unwrap();
    dir
}

async fn store_for(db_path: &str) -> Store {
    Store::open(db_path, ServerKey::from_hex(SERVER_KEY_HEX).unwrap())
        .await
        .expect("open store")
}

/// Enable the DCR policy with a redirect-host-suffix allowlist (no IAT required).
async fn enable_policy(store: &Store, suffixes: &[&str]) {
    store
        .put_oauth_dcr_policy(&OAuthDcrPolicyRow {
            enabled: true,
            require_initial_access_token: false,
            allowed_redirect_host_suffixes_json: serde_json::to_string(suffixes).unwrap(),
            default_scope_json: "{}".into(),
            updated_at: "2026-07-14T00:00:00Z".into(),
        })
        .await
        .expect("put policy");
}

async fn disable_policy(store: &Store) {
    store
        .put_oauth_dcr_policy(&OAuthDcrPolicyRow {
            enabled: false,
            require_initial_access_token: false,
            allowed_redirect_host_suffixes_json: "[]".into(),
            default_scope_json: "{}".into(),
            updated_at: "2026-07-14T00:00:00Z".into(),
        })
        .await
        .expect("put policy");
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
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
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

/// The whole DCR lifecycle, end-to-end against the running AuthServer + the db under test.
#[tokio::test]
async fn dcr_full_lifecycle_against_real_authserver() {
    let (db, on_pg) = db_path();
    // Build the server FIRST so the schema (incl. 0010 oauth_dcr) is migrated, then seed
    // the policy through a second handle sharing the same key + db.
    let server = spawn(db.clone()).await;
    let store = store_for(&db).await;
    let c = client();

    // ── 1. Default DISABLED → 403 before any enablement (deny-by-default). ────────
    disable_policy(&store).await;
    let redirect = "https://apps.vogue-homes.com/cb";
    let r = c
        .post(format!("{server}/oauth/register"))
        .json(&json!({ "redirect_uris": [redirect] }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        r.status(),
        403,
        "DCR is default-disabled — register must be 403 (on_pg={on_pg})"
    );

    // ── 2. Enable + a redirect OUTSIDE the allowlist → 400 invalid_redirect_uri. ──
    enable_policy(&store, &["apps.vogue-homes.com"]).await;
    let bad = c
        .post(format!("{server}/oauth/register"))
        .json(&json!({ "redirect_uris": ["https://evil.example/cb"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        bad.status(),
        400,
        "a redirect host outside the suffix allowlist must be rejected"
    );
    let body: Value = bad.json().await.unwrap();
    assert_eq!(body["error"], "invalid_redirect_uri", "RFC 7591 error code");

    // ── 3. Register a valid client → 201 + one-time registration-access-token. The
    //       request asks for an escalated `scope` that must NOT be granted. ─────────
    let created = c
        .post(format!("{server}/oauth/register"))
        .json(&json!({
            "redirect_uris": [redirect],
            "client_name": "e14 DCR client",
            "scope": "admin mail:write",
            "software_id": "com.vogue.e14",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), 201, "valid registration mints a client");
    let reg: Value = created.json().await.unwrap();
    let client_id = reg["client_id"].as_str().expect("client_id").to_string();
    let rat = reg["registration_access_token"]
        .as_str()
        .expect("one-time registration-access-token")
        .to_string();
    assert_eq!(reg["redirect_uris"][0], redirect);
    // No scope escalation: the granted scope is the policy default (empty), never
    // the requested "admin".
    assert!(
        !reg["scope"].as_str().unwrap_or_default().contains("admin"),
        "granted scope must never escalate to the requested privilege: {}",
        reg["scope"]
    );
    assert!(
        reg["registration_client_uri"]
            .as_str()
            .unwrap()
            .ends_with(&format!("/oauth/register/{client_id}")),
        "RFC 7592 registration_client_uri points at the config endpoint"
    );

    // ── 4. RFC 7592 GET with the registration-access-token → the client config. ──
    let read = c
        .get(format!("{server}/oauth/register/{client_id}"))
        .bearer_auth(&rat)
        .send()
        .await
        .unwrap();
    assert_eq!(read.status(), 200, "authed read returns the client config");
    let read_body: Value = read.json().await.unwrap();
    assert_eq!(read_body["client_id"], client_id);
    // The token is returned only ONCE at registration — never on read.
    assert!(read_body.get("registration_access_token").is_none());

    // A read WITHOUT the token (or a wrong token) → 401.
    let unauthed = c
        .get(format!("{server}/oauth/register/{client_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(unauthed.status(), 401, "read requires the RAT");
    let wrong = c
        .get(format!("{server}/oauth/register/{client_id}"))
        .bearer_auth("not-the-token")
        .send()
        .await
        .unwrap();
    assert_eq!(wrong.status(), 401, "a wrong RAT is rejected");

    // ── 5. RFC 7592 PUT updates the client (still under the allowlist). ──────────
    let updated = c
        .put(format!("{server}/oauth/register/{client_id}"))
        .bearer_auth(&rat)
        .json(&json!({
            "redirect_uris": [redirect, "https://apps.vogue-homes.com/cb2"],
            "client_name": "e14 DCR client (renamed)",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(updated.status(), 200, "authed update succeeds");
    let updated_body: Value = updated.json().await.unwrap();
    assert_eq!(
        updated_body["redirect_uris"].as_array().unwrap().len(),
        2,
        "the second (allowlisted) redirect is accepted"
    );
    // An update that introduces an OFF-allowlist redirect is rejected.
    let bad_update = c
        .put(format!("{server}/oauth/register/{client_id}"))
        .bearer_auth(&rat)
        .json(&json!({ "redirect_uris": ["https://evil.example/cb"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(bad_update.status(), 400, "off-allowlist update rejected");

    // ── 6. RFC 7592 DELETE deprovisions the client; a subsequent read → 401. ─────
    let deleted = c
        .delete(format!("{server}/oauth/register/{client_id}"))
        .bearer_auth(&rat)
        .send()
        .await
        .unwrap();
    assert_eq!(deleted.status(), 204, "authed delete deprovisions the client");
    let after = c
        .get(format!("{server}/oauth/register/{client_id}"))
        .bearer_auth(&rat)
        .send()
        .await
        .unwrap();
    assert_eq!(after.status(), 401, "the deleted client's RAT no longer authenticates");
}
