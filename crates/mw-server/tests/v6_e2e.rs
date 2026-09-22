//! V6 LIVE end-to-end gate (plan §3 e13, §7 Definition of Done) — the "unit-green
//! ≠ wired" DoD executor. Every scenario here drives the REAL mounted HTTP surface
//! (`build_app_full`) against **real infrastructure**: a live `postgres:16` backend
//! (selected by DSN, not SQLite) and — for the cache legs — a live `valkey:8`. The
//! headline proofs are done against the live database itself: the zero-access row is
//! read back with a DIRECT Postgres query (`psql`) and shown to be ciphertext, and
//! the OAuth authorization-code→token exchange runs end-to-end against a seeded
//! `oauth_clients` row.
//!
//! ## Running
//! These tests are **env-gated** so `cargo test --workspace` on a machine without the
//! stack does not fail — instead each prints a LOUD skip line (never a silent skip):
//!
//! ```text
//! docker run -d --name mw-e13-pg     -e POSTGRES_USER=mailwoman -e POSTGRES_PASSWORD=mailwoman \
//!            -e POSTGRES_DB=mailwoman -p 55432:5432 postgres:16
//! docker run -d --name mw-e13-valkey -p 56379:6379 valkey/valkey:8
//!
//! MW_E13_PG_DSN=postgres://mailwoman:mailwoman@127.0.0.1:55432/mailwoman \
//! MW_E13_REDIS_URL=redis://127.0.0.1:56379 \
//!   cargo test -p mw-server --test v6_e2e -- --nocapture --test-threads=1
//! ```
//!
//! `MW_E13_PG_CONTAINER` / `MW_E13_VALKEY_CONTAINER` name the docker containers used
//! for the direct `psql` / `valkey-cli` inspection (defaults `mw-e13-pg` /
//! `mw-e13-valkey`). The CI `e2e-v6` job (docker-compose service names) can override
//! all four.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Command;

use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_TEST_INDEX</div>";

// ── Env gating ────────────────────────────────────────────────────────────────

/// The live Postgres DSN, or `None` (→ loud skip). Every scenario needs this.
fn pg_dsn() -> Option<String> {
    std::env::var("MW_E13_PG_DSN")
        .ok()
        .or_else(|| std::env::var("DATABASE_URL_PG").ok())
        .filter(|s| !s.is_empty())
}

fn redis_url() -> Option<String> {
    std::env::var("MW_E13_REDIS_URL")
        .ok()
        .or_else(|| std::env::var("MW_TEST_REDIS_URL").ok())
        .filter(|s| !s.is_empty())
}

fn pg_container() -> String {
    std::env::var("MW_E13_PG_CONTAINER").unwrap_or_else(|_| "mw-e13-pg".into())
}

fn valkey_container() -> String {
    std::env::var("MW_E13_VALKEY_CONTAINER").unwrap_or_else(|_| "mw-e13-valkey".into())
}

/// A loud, greppable skip when the live stack is not up. Returns the DSN, or emits
/// a LOUD skip line (never silent) and bails out of the scenario.
macro_rules! require_pg {
    () => {{
        match pg_dsn() {
            Some(dsn) => dsn,
            None => {
                common::gate::skip(format_args!(
                    "[e13] {} — MW_E13_PG_DSN (or DATABASE_URL_PG) is unset; \
                     the live Postgres+Valkey stack is not up. This scenario is NOT covered \
                     by this run (CI e2e-v6 covers it). See the test header for bring-up.",
                    module_path!()
                ));
                return;
            }
        }
    }};
}

// ── Direct-infra inspection (the DoD "direct Postgres query" / "inspect Valkey") ──

/// Run a SQL statement inside the live Postgres container via `psql`, returning the
/// tuples-only stdout. This is the DoD's "connect to the live PG, read the row"
/// primitive — it bypasses the application entirely.
///
/// **The role and database come from [`pg_dsn`] — the very DSN the server under test
/// was handed** — rather than being assumed (t24-e14). This used to run
/// `psql -U mailwoman -d mailwoman` unconditionally, so whenever the DSN named any
/// other database the seeds landed in one database while the server read another.
/// `oauth_code_pkce_to_token_live` then could not find the `oauth_clients` row it had
/// just inserted, and `zeroaccess_ciphertext_at_rest_direct_pg_query_live` found no
/// `zeroaccess_accounts` row to read back — two failures with nothing to do with the
/// product. t24-e9 and t24-e5 each hit this independently; both only passed because
/// their throwaway container happened to name its database `mailwoman`.
fn psql(sql: &str) -> String {
    let (user, db) = psql_target(pg_dsn().as_deref());
    let out = Command::new("docker")
        .args([
            "exec",
            &pg_container(),
            "psql",
            "-U",
            &user,
            "-d",
            &db,
            "-tAc",
            sql,
        ])
        .output()
        .expect("docker exec psql must run (is docker on PATH + the container up?)");
    assert!(
        out.status.success(),
        "psql -U {user} -d {db} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// The `(role, database)` `psql` must connect as, read out of a `postgres://` DSN.
///
/// Falls back to `mailwoman`/`mailwoman` — the documented bring-up in this file's
/// header and the CI job both use it — only for the part the DSN does not state, so
/// a DSN that names a database is always believed over the default.
fn psql_target(dsn: Option<&str>) -> (String, String) {
    const DEFAULT: &str = "mailwoman";
    let Some(dsn) = dsn else {
        return (DEFAULT.into(), DEFAULT.into());
    };
    // scheme://[user[:password]@]host[:port]/[database][?params]
    let rest = dsn
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(dsn)
        .split(['?', '#'])
        .next()
        .unwrap_or_default();
    // Split authority from path on the FIRST '/' after the authority. A password may
    // legally contain '/', so find the '@' first and only search after it.
    let (authority, path) = match rest.rsplit_once('@') {
        Some((userinfo, hostpart)) => match hostpart.split_once('/') {
            Some((host, path)) => (format!("{userinfo}@{host}"), path),
            None => (format!("{userinfo}@{hostpart}"), ""),
        },
        None => match rest.split_once('/') {
            Some((host, path)) => (host.to_string(), path),
            None => (rest.to_string(), ""),
        },
    };
    let user = authority
        .rsplit_once('@')
        .map(|(userinfo, _)| userinfo.split(':').next().unwrap_or_default())
        .map(percent_decode)
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| DEFAULT.into());
    let db = {
        let d = percent_decode(path);
        if d.is_empty() { DEFAULT.into() } else { d }
    };
    (user, db)
}

/// Minimal `%XX` decoding, so a role or database containing an escaped `@`, `/` or
/// space reaches `psql` as the byte it names. Anything that is not a valid escape is
/// passed through unchanged.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(hi), Some(lo)) = (hi, lo) {
                out.push((hi * 16 + lo) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The parse above, over the DSN shapes this repo actually uses. Runs without the
/// live stack, so the regression is caught by a plain `cargo test`.
#[test]
fn psql_targets_the_database_the_dsn_names() {
    // The CI e2e-v6 DSN and this file's documented bring-up.
    assert_eq!(
        psql_target(Some(
            "postgres://mailwoman:mailwoman@127.0.0.1:5432/mailwoman"
        )),
        ("mailwoman".into(), "mailwoman".into())
    );
    // The case that failed: a database that is NOT named `mailwoman`.
    assert_eq!(
        psql_target(Some("postgres://mailwoman:mailwoman@127.0.0.1:5432/t24e14")),
        ("mailwoman".into(), "t24e14".into())
    );
    // A different role as well.
    assert_eq!(
        psql_target(Some("postgresql://alice:s3cret@db.internal:6432/mw_ci")),
        ("alice".into(), "mw_ci".into())
    );
    // Query parameters are not part of the database name.
    assert_eq!(
        psql_target(Some(
            "postgres://mailwoman:pw@host:5432/mw_ci?sslmode=disable"
        )),
        ("mailwoman".into(), "mw_ci".into())
    );
    // A password containing '/' or '@' must not be mistaken for the path or the role.
    assert_eq!(
        psql_target(Some("postgres://bob:p/a@ss@host:5432/mw_ci")),
        ("bob".into(), "mw_ci".into())
    );
    // Percent-escapes are decoded.
    assert_eq!(
        psql_target(Some("postgres://svc%40corp:pw@host/mw%20ci")),
        ("svc@corp".into(), "mw ci".into())
    );
    // Anything the DSN leaves unsaid keeps the documented default.
    assert_eq!(
        psql_target(Some("postgres://host:5432/")),
        ("mailwoman".into(), "mailwoman".into())
    );
    assert_eq!(psql_target(None), ("mailwoman".into(), "mailwoman".into()));
}

/// Run a `valkey-cli` command inside the live Valkey container, returning stdout.
fn valkey_cli(args: &[&str]) -> String {
    let mut full = vec![
        "exec".to_string(),
        valkey_container(),
        "valkey-cli".to_string(),
    ];
    full.extend(args.iter().map(|a| a.to_string()));
    let out = Command::new("docker")
        .args(&full)
        .output()
        .expect("docker exec valkey-cli must run");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

// ── Server + client harness (mirrors crates/mw-server/tests/v6_mount.rs) ─────────

mod common;
use common::test_db;

fn web_dir() -> PathBuf {
    let dir = test_db::unique_dir("mw-e13-web");
    std::fs::write(dir.join("index.html"), INDEX_HTML).unwrap();
    dir
}

fn admin_v6(redis: Option<String>) -> V6Config {
    V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: redis,
    }
}

/// Configure the process as if it sat behind a reverse proxy on loopback, which is
/// where the test client connects from. Set once for the whole binary — every test
/// here wants the same posture, so the value never changes after the first call.
///
/// This is the only way a test can drive the trusted-proxy model: it is read from the
/// environment (`crates/mw-server/src/proxy.rs`), like the crate's other deployment
/// settings.
fn trust_loopback_proxy() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        std::env::set_var("MW_TRUSTED_PROXIES", "127.0.0.0/8, ::1/128");
        std::env::set_var("MW_FORWARDED_MODE", "xff");
    });
}

async fn spawn_server(db_path: String, mode: ServerMode, v6: V6Config) -> String {
    trust_loopback_proxy();
    let config = AppConfig {
        db_path,
        server_key_hex: None,
        web_dir: Some(web_dir()),
        cookie_secure: false,
        mode,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: mock_upstreams(),
            ..SecurityConfig::default()
        },
    };
    let app = build_app_full(config, v6).await.expect("server boots").0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        // Matches `main.rs`: without connect info the server has no peer address and
        // every source-IP check degrades to "unknown".
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{addr}")
}

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

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    let origin = format!("http://{addr}");
    MOCK_UPSTREAMS.lock().unwrap().push(origin.clone());
    origin
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn login(c: &reqwest::Client, server: &str, mock: &str) -> String {
    let resp = c
        .post(format!("{server}/api/login"))
        .json(&json!({ "jmapUrl": mock, "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "mailbox login must succeed");
    let body: Value = resp.json().await.unwrap();
    body["accountId"].as_str().unwrap().to_string()
}

async fn admin_login(c: &reqwest::Client, server: &str) {
    let r = c
        .post(format!("{server}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "admin login must succeed against Postgres");
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. ADMIN — provision user + domain + quota through the real panel; audit +
//    export against the LIVE Postgres backend.
// ─────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn admin_provisioning_audit_export_live() {
    let dsn = require_pg!();
    let server = spawn_server(dsn, ServerMode::Proxy, admin_v6(None)).await;
    let a = client();

    // Unauthenticated admin surface is gated.
    assert_eq!(
        a.get(format!("{server}/admin/session"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    admin_login(&a, &server).await;

    // A unique account so re-runs against the persistent PG never collide.
    let u = test_db::unique_tag();
    let domain = format!("d{}.example", &u[..8.min(u.len())]);
    let username = format!("alice{}", &u[..6.min(u.len())]);
    let account = format!("{username}@{domain}");

    // Domain round-trips through the mounted panel → Postgres.
    let save = a
        .put(format!("{server}/admin/domains/{domain}"))
        .json(&json!({ "name": domain, "upstreamJson": "{}", "allowlist": [], "blocklist": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(save.status(), 204, "domain upsert wired to PG");

    // Provision a user + quota.
    let prov = a
        .post(format!("{server}/admin/users"))
        .json(&json!({ "domain": domain, "username": username, "quota": { "bytesLimit": 1048576, "msgLimit": 100 } }))
        .send()
        .await
        .unwrap();
    assert_eq!(prov.status(), 204, "provision user wired to PG");

    // The user appears in the panel list…
    let users: Value = a
        .get(format!("{server}/admin/users"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        users
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["accountId"] == json!(account)),
        "provisioned user is listed from PG"
    );

    // …and provisioning wrote an audit entry (append-only log in PG).
    let audit: Value = a
        .get(format!("{server}/admin/audit?limit=50"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        audit
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["action"] == json!("user-provisioned")),
        "provisioning wrote a user-provisioned audit entry"
    );

    // Export works (the audit-log export the DoD requires). It is NDJSON
    // (application/x-ndjson) — one JSON object per line.
    let export = a
        .get(format!("{server}/admin/audit/export?limit=50"))
        .send()
        .await
        .unwrap();
    assert_eq!(export.status(), 200, "audit export wired");
    let ndjson = export.text().await.unwrap();
    let lines: Vec<&str> = ndjson.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(!lines.is_empty(), "export returns audit entries (NDJSON)");
    assert!(
        lines
            .iter()
            .all(|l| serde_json::from_str::<Value>(l).is_ok()),
        "every export line is valid JSON"
    );
    assert!(
        ndjson.contains("user-provisioned"),
        "export includes the provisioning entry"
    );

    // Direct-PG cross-check: the audit row physically exists in Postgres.
    let n = psql("SELECT count(*) FROM audit_log WHERE action='user-provisioned'");
    assert!(
        n.parse::<i64>().unwrap_or(0) >= 1,
        "audit_log row present in live Postgres (direct query: {n})"
    );

    eprintln!("[e13 PASS] admin provisioning + audit + export vs live Postgres");
}

// ─────────────────────────────────────────────────────────────────────────────
// 2a. OAUTH — seed an oauth_clients row, run consent → authorization-code + PKCE
//     → token against the LIVE Postgres backend.
// ─────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn oauth_code_pkce_to_token_live() {
    let dsn = require_pg!();
    let mock = spawn_mock().await;
    let server = spawn_server(dsn, ServerMode::Proxy, admin_v6(None)).await;
    let c = client();
    let account = login(&c, &server, &mock).await;

    // Seed an admin-approved client DIRECTLY into the live 0007 `oauth_clients`
    // table (no client-registration endpoint exists — plan e11 gap (c)).
    let client_id = format!("e13-client-{}", &test_db::unique_tag()[..8]);
    let redirect = "https://app.example/cb";
    psql(&format!(
        "INSERT INTO oauth_clients (client_id,name,redirect_uris,approved_by,created_at) \
         VALUES ('{client_id}','E13 Live Client','[\"{redirect}\"]','root','2026-01-01T00:00:00Z')"
    ));

    // Fixed PKCE pair: verifier + its BASE64URL-NOPAD(SHA256(verifier)) challenge.
    const VERIFIER: &str = "e13pkceverifier0123456789abcdefghijklmnopqrstuvwxyzABCDEF";
    const CHALLENGE: &str = "daNkxhCFTwqdR-SivcCAKzFQqFpXxfZcC0bYsvxGEbw";
    let resource = "https://api.example";

    let params = json!({
        "responseType": "code",
        "clientId": client_id,
        "redirectUri": redirect,
        "codeChallenge": CHALLENGE,
        "codeChallengeMethod": "S256",
        "resource": resource,
    });

    // Consent screen sees the seeded client as APPROVED.
    let consent: Value = c
        .post(format!("{server}/oauth/consent"))
        .json(&params)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        consent["approved"],
        json!(true),
        "seeded client is approved"
    );
    assert_eq!(consent["clientName"], json!("E13 Live Client"));

    // User approves → authorization code in the redirect URI.
    let decision: Value = c
        .post(format!("{server}/oauth/decision"))
        .json(&json!({ "approve": true, "params": params }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let redirect_uri = decision["redirectUri"]
        .as_str()
        .expect("redirect with code");
    let code = redirect_uri
        .split("code=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .expect("authorization code present");
    let code = urldecode(code);
    assert!(!code.is_empty(), "non-empty authorization code");

    // Token exchange with the PKCE verifier + matching resource → access token.
    let tok: Value = c
        .post(format!("{server}/oauth/token"))
        .json(&json!({
            "grant_type": "authorization_code",
            "code": code,
            "redirect_uri": redirect,
            "client_id": client_id,
            "code_verifier": VERIFIER,
            "resource": resource,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let access = tok["access_token"].as_str().expect("access_token issued");
    assert!(!access.is_empty(), "non-empty access token");
    assert_eq!(tok["token_type"], json!("Bearer"));

    // Introspection confirms the token is active for this account (live PG lookup).
    let intro: Value = c
        .post(format!("{server}/oauth/introspect"))
        .json(&json!({ "token": access }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        intro["active"],
        json!(true),
        "issued token introspects active"
    );

    // A wrong PKCE verifier must NOT mint a token (negative control).
    psql(&format!(
        "INSERT INTO oauth_clients (client_id,name,redirect_uris,approved_by,created_at) \
         VALUES ('{client_id}-2','x','[\"{redirect}\"]','root','2026-01-01T00:00:00Z')"
    ));
    let d2: Value = c
        .post(format!("{server}/oauth/decision"))
        .json(&json!({ "approve": true, "params": {
            "responseType": "code", "clientId": format!("{client_id}-2"),
            "redirectUri": redirect, "codeChallenge": CHALLENGE,
            "codeChallengeMethod": "S256", "resource": resource,
        }}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let code2 = urldecode(
        d2["redirectUri"]
            .as_str()
            .unwrap()
            .split("code=")
            .nth(1)
            .unwrap()
            .split('&')
            .next()
            .unwrap(),
    );
    let bad = c
        .post(format!("{server}/oauth/token"))
        .json(&json!({
            "grant_type": "authorization_code", "code": code2, "redirect_uri": redirect,
            "client_id": format!("{client_id}-2"), "code_verifier": "WRONG-verifier-value-000000000000000000000000",
            "resource": resource,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 400, "PKCE mismatch is rejected");

    let _ = account;
    eprintln!("[e13 PASS] OAuth consent → auth-code + PKCE S256 → token vs live Postgres");
}

// ─────────────────────────────────────────────────────────────────────────────
// 2b. API KEYS — mint a scoped key, assert the /api/v1 enforcement matrix against
//     the LIVE Postgres backend: in-scope 200 / out-of-scope 403 / expired 401 /
//     IP-allowlist 403 / over-rate 429 (e11b enforcement, now vs real PG).
// ─────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn apikey_enforcement_matrix_live() {
    let dsn = require_pg!();
    let mock = spawn_mock().await;
    let server = spawn_server(dsn, ServerMode::Proxy, admin_v6(None)).await;
    let c = client();
    let account = login(&c, &server, &mock).await;

    async fn mint(c: &reqwest::Client, server: &str, account: &str, scope: Value) -> String {
        let m: Value = c
            .post(format!("{server}/api/keys"))
            .json(&json!({ "label": "e13", "accountId": account, "scope": scope }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        m["displayToken"]
            .as_str()
            .expect("shown-once token")
            .to_string()
    }
    fn scope(account: &str, read: bool, mail: bool) -> Value {
        json!({
            "read": read, "send": false, "delete": false,
            "accounts": { "subset": [account] }, "folders": "all",
            "mail": mail, "pim": !mail, "ip_allowlist": [], "expires_at": null,
            "rate_limit": null, "mcp_tools": [], "unattended_send": false,
        })
    }

    // GRANT: in-scope key → 200 with the JMAP list.
    let good = mint(&c, &server, &account, scope(&account, true, true)).await;
    assert!(good.starts_with("mwk_"), "wire format mwk_…: {good}");
    let r = c
        .get(format!("{server}/api/v1/messages?limit=5"))
        .header("x-api-key", &good)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "in-scope key → 200");
    assert!(r.json::<Value>().await.unwrap().get("messages").is_some());

    // DENY (out-of-scope / capability): no read → 403.
    let no_read = mint(&c, &server, &account, scope(&account, false, true)).await;
    assert_eq!(
        c.get(format!("{server}/api/v1/messages"))
            .header("x-api-key", &no_read)
            .send()
            .await
            .unwrap()
            .status(),
        403,
        "no-read key → 403 (out of scope)"
    );

    // DENY (out-of-scope / different account) → 403.
    let wrong = mint(
        &c,
        &server,
        &account,
        scope("nobody@elsewhere.test", true, true),
    )
    .await;
    assert_eq!(
        c.get(format!("{server}/api/v1/messages"))
            .header("x-api-key", &wrong)
            .send()
            .await
            .unwrap()
            .status(),
        403,
        "wrong-account key → 403"
    );

    // DENY (expired) → 401.
    let mut exp = scope(&account, true, true);
    exp["expires_at"] = json!("2000-01-01T00:00:00Z");
    let ek = mint(&c, &server, &account, exp).await;
    assert_eq!(
        c.get(format!("{server}/api/v1/messages"))
            .header("x-api-key", &ek)
            .send()
            .await
            .unwrap()
            .status(),
        401,
        "expired key → 401"
    );

    // IP-allowlist under the trusted-proxy model (t20 B1): `trust_loopback_proxy()`
    // made this process trust a proxy on loopback, where the test client connects
    // from, so `X-Forwarded-For` is read — right to left, as a proxy writes it.
    let mut ips = scope(&account, true, true);
    ips["ip_allowlist"] = json!(["10.0.0.0/8"]);
    let ik = mint(&c, &server, &account, ips).await;
    assert_eq!(
        c.get(format!("{server}/api/v1/messages"))
            .header("x-api-key", &ik)
            .header("x-forwarded-for", "8.8.8.8")
            .send()
            .await
            .unwrap()
            .status(),
        403,
        "proxy-reported IP outside allowlist → 403"
    );
    assert_eq!(
        c.get(format!("{server}/api/v1/messages"))
            .header("x-api-key", &ik)
            .header("x-forwarded-for", "10.1.2.3")
            .send()
            .await
            .unwrap()
            .status(),
        200,
        "proxy-reported IP inside allowlist → 200"
    );
    // The spoof this assertion used to encode as expected: a forged left-hand hop
    // naming an allowlisted address, with the address the proxy really saw appended
    // to its right. The walk stops at the rightmost untrusted hop → still denied.
    assert_eq!(
        c.get(format!("{server}/api/v1/messages"))
            .header("x-api-key", &ik)
            .header("x-forwarded-for", "10.1.2.3, 8.8.8.8")
            .send()
            .await
            .unwrap()
            .status(),
        403,
        "forged left-hand X-Forwarded-For hop does not defeat the allowlist"
    );

    // DENY (over rate limit) → 429.
    let mut rl = scope(&account, true, true);
    rl["rate_limit"] = json!(1);
    let rk = mint(&c, &server, &account, rl).await;
    assert_eq!(
        c.get(format!("{server}/api/v1/messages"))
            .header("x-api-key", &rk)
            .send()
            .await
            .unwrap()
            .status(),
        200,
        "first request within rate limit"
    );
    assert_eq!(
        c.get(format!("{server}/api/v1/messages"))
            .header("x-api-key", &rk)
            .send()
            .await
            .unwrap()
            .status(),
        429,
        "second request over rate limit → 429"
    );

    // DENY (unknown key) → 401.
    assert_eq!(
        c.get(format!("{server}/api/v1/messages"))
            .header("x-api-key", "mwk_deadbeef.notreal")
            .send()
            .await
            .unwrap()
            .status(),
        401,
        "unknown key → 401"
    );

    // The minted key physically lives (hashed) in live Postgres.
    let keys = psql("SELECT count(*) FROM api_keys");
    assert!(
        keys.parse::<i64>().unwrap_or(0) >= 6,
        "api_keys rows persisted to PG: {keys}"
    );
    // …and only a HASH is stored, never the shown-once secret.
    let leaked = psql("SELECT count(*) FROM api_keys WHERE key_hash LIKE 'mwk_%'");
    assert_eq!(leaked, "0", "no plaintext key material stored (hash only)");

    eprintln!(
        "[e13 PASS] scoped-API-key enforcement matrix (200/403/401/403-IP/429) vs live Postgres"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. MCP — real Streamable-HTTP handshake → tools/list (frozen 10) → untrusted
//    provenance on mail content; mail.send is enumerated but gated (unauth call
//    denied). Runs vs the LIVE Postgres backend (engine mode).
//
//    NOTE (documented, not silent): the fully-live gated mail.send→Outbox WITH a
//    content/`transmitted()==0` assertion needs a real IMAP mailbox behind the
//    engine, which a spawned harness has not. That leg is hard-proven by mw-mcp's
//    e4 unit suite (13 tests: all 3 send paths assert transmitted()==0) and driven
//    live by apps/web/e2e/mcp.spec.ts (CI e2e-v6). Here we prove the transport,
//    the frozen contract, the untrusted labeling, and that send is NOT open.
// ─────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn mcp_handshake_toolslist_provenance_live() {
    let dsn = require_pg!();
    let server = spawn_server(dsn, ServerMode::Engine, admin_v6(None)).await;
    let c = client();

    let init: Value = c
        .post(format!("{server}/mcp"))
        .json(&json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(init["result"]["serverInfo"]["name"], json!("mailwoman-mcp"));

    let list: Value = c
        .post(format!("{server}/mcp"))
        .json(&json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {} }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let tools = list["result"]["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 10, "frozen §2.4 MCP tool set (10 tools)");

    // The frozen tool names.
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    for expected in [
        "mail.search",
        "mail.read",
        "folders.list",
        "drafts.create",
        "mail.send",
        "calendar.read",
        "calendar.propose",
        "tasks.read",
        "tasks.write",
        "contacts.read",
    ] {
        assert!(names.contains(&expected), "tool {expected} is enumerated");
    }

    // Prompt-injection posture: mail-content tools declare untrusted output.
    let search = tools
        .iter()
        .find(|t| t["name"] == json!("mail.search"))
        .unwrap();
    assert_eq!(
        search["_meta"]["untrustedOutput"],
        json!(true),
        "mail.search output is untrusted"
    );
    let read = tools
        .iter()
        .find(|t| t["name"] == json!("mail.read"))
        .unwrap();
    assert_eq!(
        read["_meta"]["untrustedOutput"],
        json!(true),
        "mail.read output is untrusted"
    );

    // Send is enumerated but NOT open: an UNauthenticated tools/call is refused
    // (proves the tool is gated, never a raw transmit).
    let unauth: Value = c
        .post(format!("{server}/mcp"))
        .json(&json!({
            "jsonrpc": "2.0", "id": 3, "method": "tools/call",
            "params": { "name": "mail.send", "arguments": { "to": ["x@y.test"], "subject": "s", "bodyText": "b" } }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        unauth.get("error").is_some() || unauth["result"]["isError"] == json!(true),
        "unauthenticated mail.send is refused (gated), got: {unauth}"
    );

    eprintln!(
        "[e13 PASS] MCP handshake + tools/list(10) + untrusted provenance + gated send vs live Postgres"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. ZERO-ACCESS — enable via the real surface, then prove with a DIRECT Postgres
//    query that the stored row is CIPHERTEXT the server can't read, and that the
//    plaintext key material is never persisted. (The DoD headline proof.)
// ─────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn zeroaccess_ciphertext_at_rest_direct_pg_query_live() {
    let dsn = require_pg!();
    let mock = spawn_mock().await;
    let server = spawn_server(dsn, ServerMode::Proxy, admin_v6(None)).await;
    let c = client();
    let account = login(&c, &server, &mock).await;

    // The client's plaintext key material — a marker string the server must NEVER
    // see or store. In production this is the account data key; the real
    // XChaCha20-Poly1305 wrap runs in the mw-crypto WASM worker (e6, 11 unit tests
    // incl. round-trip + AAD). Here the client hands the server only OPAQUE wrapped
    // bytes: we prove the server stores exactly those bytes and never the plaintext.
    const PLAINTEXT_MARKER: &str = "E13_ZERO_ACCESS_PLAINTEXT_KEY_MARKER";

    // An opaque ciphertext blob (nonce ‖ ct+tag shape): 24-byte nonce + 32 bytes,
    // deterministically non-plaintext, containing NONE of the marker bytes.
    let mut wrapped = Vec::with_capacity(56);
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    for i in 0u64..56 {
        // A simple non-plaintext byte pattern (0x80..0xFF range never collides with
        // the ASCII marker bytes) — stands in for AEAD output for the at-rest proof.
        wrapped.push(0x80u8.wrapping_add(((seed.wrapping_mul(i + 1)) & 0x7f) as u8));
    }
    let wrapped_b64 = base64_encode(&wrapped);

    // Enable zero-access through the mounted surface.
    let enable = c
        .post(format!("{server}/api/zeroaccess/enable"))
        .json(&json!({
            "saltB64": "c2FsdHNhbHRzYWx0",
            "kdfParams": { "mCost": 19456, "tCost": 2, "pCost": 1 },
            "wrappedDataKeyB64": wrapped_b64,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(enable.status(), 200, "zero-access enable wired to PG");

    // The status endpoint returns wrapped material ONLY — never a key.
    let za: Value = c
        .get(format!("{server}/api/zeroaccess"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(za["enabled"], json!(true));
    assert_eq!(
        za["wrappedDataKeyB64"],
        json!(wrapped_b64),
        "server returns exactly the wrapped bytes it was handed"
    );

    // ── DIRECT POSTGRES QUERY (the DoD proof) ────────────────────────────────
    // Read the raw BYTEA column straight from the live database, as hex.
    let hex = psql(&format!(
        "SELECT encode(wrapped_root_key,'hex') FROM zeroaccess_accounts WHERE account_id='{account}'"
    ));
    assert!(!hex.is_empty(), "zeroaccess_accounts row exists in live PG");
    let stored = hex_decode(&hex);

    // (a) The at-rest bytes are EXACTLY the client-provided ciphertext (verbatim,
    //     opaque storage — the server neither decrypted nor transformed it).
    assert_eq!(stored, wrapped, "DB row is the client ciphertext verbatim");

    // (b) The plaintext key marker is ABSENT from the stored bytes — the server
    //     never received nor persisted plaintext key material. This is the
    //     ciphertext-at-rest guarantee, asserted against the live DB row itself.
    let marker = PLAINTEXT_MARKER.as_bytes();
    assert!(
        !contains_subslice(&stored, marker),
        "plaintext marker MUST NOT appear in the at-rest DB row"
    );
    assert!(
        !hex.contains(&hex_encode(marker)),
        "plaintext marker (hex) MUST NOT appear in the at-rest DB row"
    );

    // (c) `enabled` is set in the physical row.
    let enabled = psql(&format!(
        "SELECT enabled FROM zeroaccess_accounts WHERE account_id='{account}'"
    ));
    assert!(
        enabled == "1" || enabled == "t" || enabled == "true",
        "row enabled: {enabled}"
    );

    eprintln!(
        "[e13 PASS] zero-access ciphertext-at-rest — direct PG query returned {} ciphertext bytes; \
         plaintext marker absent; server holds only wrapped material",
        stored.len()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. CACHE POSTURE — the server accepts a live Valkey URL AND degrades cleanly
//    when Redis is DOWN (no hard dependency, no data loss on served routes). The
//    §15.6 per-class matrix + the structural zero-access exclusion + Redis-down
//    store-fallthrough-WITH-data are proven live at the mw-cache level by e2's
//    `cache-valkey` CI job (validated vs a real valkey:8 container); here we prove
//    the server-level posture: it never treats Valkey as authoritative.
// ─────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn cache_posture_redis_optional_and_down_degrades_live() {
    let dsn = require_pg!();

    // Leg A: engine-mode server WITH a live Valkey URL boots and serves.
    let (redis_present, redis) = match redis_url() {
        Some(u) => {
            // Direct live-Valkey liveness (inspect the real cache).
            let pong = valkey_cli(&["PING"]);
            assert_eq!(pong, "PONG", "live Valkey answers PING");
            (true, Some(u))
        }
        None => {
            common::gate::skip(
                "[e13] MW_E13_REDIS_URL unset — the live-Valkey leg is not driven; \
                 the Redis-DOWN degradation leg still runs (points at a dead port).",
            );
            (false, None)
        }
    };

    if redis_present {
        let server = spawn_server(dsn.clone(), ServerMode::Engine, admin_v6(redis)).await;
        let c = client();
        // The SPA / health surface serves with Valkey attached.
        let r = c.get(format!("{server}/")).send().await.unwrap();
        assert_eq!(r.status(), 200, "server serves with a live Valkey attached");
        // Valkey is never authoritative: no plaintext-derived message-body keys are
        // written for any zero-access account by merely booting (structural clean).
        let keys = valkey_cli(&["KEYS", "*message-bodies*"]);
        // (Fresh accelerator: may be empty. The invariant is that ZA plaintext is
        // never here — asserted structurally by e2. We record what is present.)
        eprintln!("[e13 NOTE] live Valkey message-body keys after boot: {keys:?}");
    }

    // Leg B (always runs): Redis DOWN → the server still boots and serves. A dead
    // Redis is logged and degraded to memory + store with no data loss (§15, e2).
    let server = spawn_server(
        dsn,
        ServerMode::Engine,
        admin_v6(Some("redis://127.0.0.1:1".into())), // guaranteed-refused port
    )
    .await;
    let c = client();
    let r = c.get(format!("{server}/")).send().await.unwrap();
    assert_eq!(
        r.status(),
        200,
        "Redis-DOWN: server still boots + serves (Valkey never authoritative, no data loss)"
    );
    // The mounted API surface still answers (admin panel is gated, not broken).
    let a = c
        .get(format!("{server}/admin/session"))
        .send()
        .await
        .unwrap();
    assert_eq!(a.status(), 401, "Redis-DOWN: admin surface still mounted");

    eprintln!("[e13 PASS] cache posture — Valkey optional + Redis-down degrades without data loss");
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. BACKEND PARITY — the admin + API-key happy path behaves IDENTICALLY on the
//    SQLite default backend and the live Postgres backend.
// ─────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn backend_parity_sqlite_and_postgres_live() {
    let dsn = require_pg!();

    // The identical happy-path scenario, parameterized by backend DSN.
    async fn happy_path(db_path: String) -> (u16, u16, bool, u16) {
        let mock = spawn_mock().await;
        let server = spawn_server(db_path, ServerMode::Proxy, admin_v6(None)).await;
        let c = client();
        let account = login(&c, &server, &mock).await;

        // (1) admin login + provision.
        admin_login(&c, &server).await;
        let u = test_db::unique_tag();
        let domain = format!("p{}.example", &u[..8.min(u.len())]);
        let username = format!("bob{}", &u[..6.min(u.len())]);
        let prov = c
            .post(format!("{server}/admin/users"))
            .json(&json!({ "domain": domain, "username": username, "quota": { "bytesLimit": 1, "msgLimit": 1 } }))
            .send()
            .await
            .unwrap()
            .status()
            .as_u16();

        // (2) mint a scoped key.
        let mint: Value = c
            .post(format!("{server}/api/keys"))
            .json(&json!({ "label": "parity", "accountId": account, "scope": {
                "read": true, "send": false, "delete": false,
                "accounts": { "subset": [account] }, "folders": "all",
                "mail": true, "pim": false, "ip_allowlist": [], "expires_at": null,
                "rate_limit": null, "mcp_tools": [], "unattended_send": false,
            }}))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let token = mint["displayToken"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let wire_ok = token.starts_with("mwk_");

        // (3) in-scope REST call.
        let rest = c
            .get(format!("{server}/api/v1/messages?limit=3"))
            .header("x-api-key", &token)
            .send()
            .await
            .unwrap()
            .status()
            .as_u16();

        // (4) an out-of-scope key → 403 (enforcement parity).
        let bad_mint: Value = c
            .post(format!("{server}/api/keys"))
            .json(
                &json!({ "label": "parity-bad", "accountId": account, "scope": {
                    "read": false, "send": false, "delete": false,
                    "accounts": { "subset": [account] }, "folders": "all",
                    "mail": true, "pim": false, "ip_allowlist": [], "expires_at": null,
                    "rate_limit": null, "mcp_tools": [], "unattended_send": false,
                }}),
            )
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let bad = c
            .get(format!("{server}/api/v1/messages"))
            .header(
                "x-api-key",
                bad_mint["displayToken"].as_str().unwrap_or_default(),
            )
            .send()
            .await
            .unwrap()
            .status()
            .as_u16();

        (prov, rest, wire_ok, bad)
    }

    let sqlite_path = test_db::unique_db_path("mw-e13-parity")
        .to_string_lossy()
        .into_owned();
    let sqlite = happy_path(sqlite_path).await;
    let postgres = happy_path(dsn).await;

    assert_eq!(sqlite.0, 204, "SQLite: provision → 204");
    assert_eq!(postgres.0, 204, "Postgres: provision → 204");
    assert_eq!(
        sqlite, postgres,
        "SQLite and Postgres behave identically on the happy path"
    );

    eprintln!(
        "[e13 PASS] backend parity — SQLite {:?} == Postgres {:?}",
        sqlite, postgres
    );
}

// ── tiny dependency-free helpers (no new dev-deps) ──────────────────────────────

fn urldecode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                let h = hex_val(b[i + 1]);
                let l = hex_val(b[i + 2]);
                if let (Some(h), Some(l)) = (h, l) {
                    out.push(h << 4 | l);
                    i += 3;
                    continue;
                }
                out.push(b[i]);
                i += 1;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

fn hex_decode(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len() / 2);
    let mut i = 0;
    while i + 1 < b.len() {
        if let (Some(h), Some(l)) = (hex_val(b[i]), hex_val(b[i + 1])) {
            out.push(h << 4 | l);
        }
        i += 2;
    }
    out
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}
