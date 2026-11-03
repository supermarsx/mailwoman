//! t29-e8 — open tracking through the real router.
//!
//! Drives the routes `build_app` mounts, over a loopback socket, with sessions
//! seeded in the server's own store:
//!
//!   * mint → bind → the count is 0 (precondition) → the image is fetched with no
//!     cookie → 200 `image/gif`, 43 bytes, `no-store, private`, CORP `cross-origin`
//!     → the count is 1 with a timestamp;
//!   * an unknown token gets the identical response and writes no row;
//!   * another account's session gets 404 on the count and on the bind;
//!   * loads past the per-token burst are served but not counted;
//!   * the image URL is absolute, from `MW_PUBLIC_URL` (and `MW_BASE_PATH`), never
//!     from the request's `Host`;
//!   * without `MW_PUBLIC_URL` the mint answers 409 `publicUrlRequired` and
//!     `/api/policy` says `openTracking: false`.
//!
//! The Postgres leg runs the main scenario against `DATABASE_URL_PG` when set.
//!
//!   cargo test --locked -p mw-server --test t29_open_tracking -- --test-threads=1

use std::path::PathBuf;
use std::sync::OnceLock;

use serde_json::{Value, json};
use tokio::sync::{Mutex, MutexGuard};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, build_app};
use mw_store::{Credentials, ServerKey, Store};

mod common;
use common::test_db;

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=root></div>";
const KEY_HEX: &str = "2929292929292929292929292929292929292929292929292929292929292929";
const PUBLIC_URL: &str = "https://mail.example.org";

/// Serialises the env mutations (`MW_PUBLIC_URL`, `MW_BASE_PATH`) every
/// `build_app` in this binary reads.
fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(Default::default)
}

async fn env(public_url: Option<&str>, base_path: Option<&str>) -> MutexGuard<'static, ()> {
    let guard = env_lock().lock().await;
    // SAFETY: the guard serialises every mutation and every read that depends on one
    // within this binary (house pattern, see `t20_forwarded_spoof.rs`).
    unsafe {
        for (k, v) in [("MW_PUBLIC_URL", public_url), ("MW_BASE_PATH", base_path)] {
            match v {
                Some(v) => std::env::set_var(k, v),
                None => std::env::remove_var(k),
            }
        }
        std::env::remove_var("MW_COOKIE_SECURE");
    }
    guard
}

struct Server {
    base: String,
    db: String,
    /// The configured JMAP upstream (an in-process mock). A session row has to name
    /// it: since 7e9af0d0 every request re-checks a session against the configured
    /// upstream, and a row naming another one is refused.
    upstream: String,
}

async fn spawn_mock_jmap() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

fn sqlite_db() -> String {
    test_db::unique_db_path("mw-t29e8")
        .to_string_lossy()
        .into_owned()
}

async fn serve(db: String) -> Server {
    let upstream = spawn_mock_jmap().await;
    let dir = test_db::unique_dir("mw-t29e8-web");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: db.clone(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(vec![upstream.clone()]),
            upstream_default: Some(upstream.clone()),
            ..SecurityConfig::default()
        },
    };
    let app = build_app(config).await.expect("build_app");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    Server {
        base: format!("http://{addr}"),
        db,
        upstream,
    }
}

/// A real session row for `account_id` in the server's store; returns its cookie.
async fn seed_session(srv: &Server, account_id: &str) -> String {
    seed_named_session(srv, account_id, &format!("{account_id}@example.org")).await
}

async fn seed_named_session(srv: &Server, account_id: &str, username: &str) -> String {
    let store = Store::open(&srv.db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    let id = store
        .create_session(
            account_id,
            username,
            &srv.upstream,
            &srv.upstream,
            &Credentials {
                username: username.into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    store
        .set_session_authentication(&id, "password")
        .await
        .unwrap();
    store
        .bind_verified_primary(
            &id,
            mw_store::identity_principal::VerifiedPrimary {
                account_id,
                kind: "mailbox",
                authority: &srv.upstream,
                subject: username,
            },
            chrono::Utc::now().timestamp(),
        )
        .await
        .unwrap()
        .unwrap();
    format!("mw_session={id}")
}

/// Raw row counts, straight from the database rather than through the routes.
async fn rows(db: &str, table: &str) -> i64 {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    if db.starts_with("postgres") {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(db)
            .await
            .unwrap();
        sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&pool)
            .await
            .unwrap()
    } else {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite://{db}?mode=ro"))
            .await
            .unwrap();
        sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&pool)
            .await
            .unwrap()
    }
}

fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

async fn mint(c: &reqwest::Client, base: &str, cookie: &str) -> reqwest::Response {
    c.post(format!("{base}/api/track/open"))
        .header("Cookie", cookie)
        .json(&json!({ "subject": "Quarterly report", "to": ["reader@example.net"] }))
        .send()
        .await
        .unwrap()
}

async fn summary(
    c: &reqwest::Client,
    base: &str,
    cookie: Option<&str>,
    email: &str,
) -> (u16, Value) {
    let mut req = c.get(format!("{base}/api/track/open?emailId={email}"));
    if let Some(ck) = cookie {
        req = req.header("Cookie", ck);
    }
    let resp = req.send().await.unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

/// Fetch an image the way a mail program does: no cookie, no session.
async fn fetch_image(
    base: &str,
    path: &str,
) -> (u16, Option<String>, Option<String>, Option<String>, Vec<u8>) {
    let resp = http().get(format!("{base}{path}")).send().await.unwrap();
    let h = |n: &str| {
        resp.headers()
            .get(n)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
    };
    let (ct, cc, corp) = (
        h("content-type"),
        h("cache-control"),
        h("cross-origin-resource-policy"),
    );
    let status = resp.status().as_u16();
    (status, ct, cc, corp, resp.bytes().await.unwrap().to_vec())
}

/// The main scenario, on whichever database `db` names.
async fn scenario(db: String, email: &str) {
    let srv = serve(db).await;
    let c = http();
    let owner = seed_session(&srv, &format!("acct-owner-{email}")).await;
    let other = seed_session(&srv, &format!("acct-other-{email}")).await;

    let resp = mint(&c, &srv.base, &owner).await;
    assert_eq!(resp.status().as_u16(), 200, "mint");
    let body: Value = resp.json().await.unwrap();
    let token = body["token"].as_str().expect("token").to_string();
    let url = body["url"].as_str().expect("url").to_string();
    assert_eq!(token.len(), 22, "128-bit base64url token: {token}");
    // Absolute, from MW_PUBLIC_URL — not from the Host the test connected to.
    assert_eq!(url, format!("{PUBLIC_URL}/t/{token}.gif"));
    let path = url.strip_prefix(PUBLIC_URL).unwrap().to_string();

    let bind = c
        .put(format!("{}/api/track/open/{token}", srv.base))
        .header("Cookie", &owner)
        .json(&json!({ "emailId": email }))
        .send()
        .await
        .unwrap();
    assert_eq!(bind.status().as_u16(), 204, "bind");

    // Precondition: nothing recorded yet.
    let (st, zero) = summary(&c, &srv.base, Some(&owner), email).await;
    assert_eq!(st, 200);
    assert_eq!(zero, json!({ "count": 0, "first": null, "last": null }));
    let events_before = rows(&srv.db, "open_events").await;

    // The load: no cookie at all.
    let (st, ct, cc, corp, bytes) = fetch_image(&srv.base, &path).await;
    assert_eq!(st, 200);
    assert_eq!(ct.as_deref(), Some("image/gif"));
    assert_eq!(cc.as_deref(), Some("no-store, private"));
    assert_eq!(
        corp.as_deref(),
        Some("cross-origin"),
        "a cross-origin <img> must be allowed"
    );
    assert_eq!(bytes.len(), 43);
    assert_eq!(&bytes[..6], b"GIF89a");

    let (st, one) = summary(&c, &srv.base, Some(&owner), email).await;
    assert_eq!(st, 200);
    assert_eq!(one["count"], json!(1), "{one}");
    let first = one["first"].as_str().expect("first is a timestamp");
    assert!(
        chrono::DateTime::parse_from_rfc3339(first).is_ok(),
        "first is RFC 3339: {first}"
    );
    assert_eq!(one["first"], one["last"]);
    assert_eq!(rows(&srv.db, "open_events").await, events_before + 1);

    // An unknown (well-formed) token: the identical response, and nothing written.
    let tracking_before = rows(&srv.db, "open_tracking").await;
    let events_before = rows(&srv.db, "open_events").await;
    let (st2, ct2, cc2, corp2, bytes2) =
        fetch_image(&srv.base, "/t/AAAAAAAAAAAAAAAAAAAAAA.gif").await;
    assert_eq!(
        (st2, ct2, cc2, corp2, bytes2),
        (st, ct, cc, corp, bytes),
        "an unknown token is indistinguishable from a known one"
    );
    assert_eq!(rows(&srv.db, "open_tracking").await, tracking_before);
    assert_eq!(rows(&srv.db, "open_events").await, events_before);

    // Another account: the count and the bind are 404, as if nothing existed.
    let (st, _) = summary(&c, &srv.base, Some(&other), email).await;
    assert_eq!(st, 404, "another account's session must not see the count");
    let steal = c
        .put(format!("{}/api/track/open/{token}", srv.base))
        .header("Cookie", &other)
        .json(&json!({ "emailId": "hijack" }))
        .send()
        .await
        .unwrap();
    assert_eq!(steal.status().as_u16(), 404);
    // And no session at all.
    let (st, _) = summary(&c, &srv.base, None, email).await;
    assert_eq!(st, 401);

    // Per-token rate limit: the burst is 10 and one is spent. 15 more loads are all
    // served, but only the remaining 9 (10 if a refill lands meanwhile) are counted.
    for _ in 0..15 {
        let (st, ..) = fetch_image(&srv.base, &path).await;
        assert_eq!(st, 200);
    }
    let (_, capped) = summary(&c, &srv.base, Some(&owner), email).await;
    let n = capped["count"].as_i64().unwrap();
    assert!((10..=11).contains(&n), "rate-limited count: {capped}");
}

#[tokio::test]
async fn open_is_recorded_without_a_session_and_only_the_owner_reads_it() {
    let _g = env(Some(PUBLIC_URL), None).await;
    scenario(sqlite_db(), "E-sqlite").await;
}

#[tokio::test]
async fn open_tracking_on_postgres() {
    let Some(dsn) = common::gate::pg_dsn() else {
        common::gate::skip(
            "[t29-e8] DATABASE_URL_PG unset — the Postgres leg of open tracking is not driven.",
        );
        return;
    };
    let _g = env(Some(PUBLIC_URL), None).await;
    let email = format!("E-pg-{}", test_db::unique_tag());
    scenario(dsn, &email).await;
}

#[tokio::test]
async fn image_url_carries_the_base_path() {
    let _g = env(Some(PUBLIC_URL), Some("/mail")).await;
    let srv = serve(sqlite_db()).await;
    let c = http();
    let owner = seed_session(&srv, "acct-subpath").await;
    let body: Value = mint(&c, &srv.base, &owner).await.json().await.unwrap();
    let token = body["token"].as_str().unwrap();
    assert_eq!(
        body["url"].as_str().unwrap(),
        format!("{PUBLIC_URL}/mail/t/{token}.gif")
    );
    let bind = c
        .put(format!("{}/mail/api/track/open/{token}", srv.base))
        .header("Cookie", &owner)
        .json(&json!({ "emailId": "E-subpath" }))
        .send()
        .await
        .unwrap();
    assert_eq!(bind.status().as_u16(), 204);
    let (st, ..) = fetch_image(&srv.base, &format!("/mail/t/{token}.gif")).await;
    assert_eq!(st, 200);
    let (_, one) = summary(&c, &format!("{}/mail", srv.base), Some(&owner), "E-subpath").await;
    assert_eq!(one["count"], json!(1), "{one}");
}

#[tokio::test]
async fn without_public_url_mint_is_refused_and_policy_says_off() {
    let _g = env(None, None).await;
    let srv = serve(sqlite_db()).await;
    let c = http();
    let owner = seed_session(&srv, "acct-nopublic").await;

    let resp = mint(&c, &srv.base, &owner).await;
    assert_eq!(resp.status().as_u16(), 409);
    assert_eq!(
        resp.json::<Value>().await.unwrap(),
        json!({ "error": "publicUrlRequired" })
    );
    assert_eq!(rows(&srv.db, "open_tracking").await, 0);

    let policy: Value = c
        .get(format!("{}/api/policy", srv.base))
        .header("Cookie", &owner)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(policy["openTracking"], json!(false), "{policy}");
}

#[tokio::test]
async fn with_public_url_policy_says_on() {
    let _g = env(Some(PUBLIC_URL), None).await;
    let srv = serve(sqlite_db()).await;
    let owner = seed_session(&srv, "acct-policy-on").await;
    let policy: Value = http()
        .get(format!("{}/api/policy", srv.base))
        .header("Cookie", &owner)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(policy["openTracking"], json!(true), "{policy}");
}

async fn tracking_store(srv: &Server) -> Store {
    Store::open(&srv.db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap()
}

async fn tracking_policy(srv: &Server, cookie: &str) -> Value {
    let response = http()
        .get(format!("{}/api/policy", srv.base))
        .header("Cookie", cookie)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    response.json().await.unwrap()
}

async fn global_setting(store: &Store, key: &str, value: Value, locked: bool) {
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: true,
            global_locked: locked,
            updated_by: "tracking-test".into(),
            updated_at: chrono::Utc::now().timestamp(),
        })
        .await
        .unwrap();
}

async fn bound_token(srv: &Server, cookie: &str, email: &str) -> String {
    let response = mint(&http(), &srv.base, cookie).await;
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    let token = body["token"].as_str().unwrap().to_owned();
    let response = http()
        .put(format!("{}/api/track/open/{token}", srv.base))
        .header("Cookie", cookie)
        .json(&json!({"emailId":email}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 204);
    token
}

#[tokio::test]
async fn default_is_off_users_can_save_it_and_admin_capabilities_are_not_user_editable() {
    let _guard = env(Some(PUBLIC_URL), None).await;
    let srv = serve(sqlite_db()).await;
    let cookie = seed_session(&srv, "tracking-settings").await;
    let client = http();
    let authority = tracking_store(&srv)
        .await
        .session_storage_authority(cookie.strip_prefix("mw_session=").unwrap())
        .await
        .unwrap()
        .unwrap();
    let settings: Value = client
        .get(format!("{}/api/settings", srv.base))
        .header("Cookie", &cookie)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(settings["settings"]["mail.openTracking"]["value"], false);
    assert_eq!(
        tracking_policy(&srv, &cookie).await["openTracking"],
        true,
        "off default still allows explicit per-message opt-in"
    );
    let saved = client
        .put(format!("{}/api/settings/mail.openTracking", srv.base))
        .header("Cookie", &cookie)
        .header("x-mailwoman-storage-authority", &authority)
        .json(&json!({"value":true}))
        .send()
        .await
        .unwrap();
    assert_eq!(saved.status(), 200);
    assert_eq!(saved.json::<Value>().await.unwrap()["value"], true);
    let refused = client
        .put(format!(
            "{}/api/settings/cap.compose.open_tracking",
            srv.base
        ))
        .header("Cookie", &cookie)
        .header("x-mailwoman-storage-authority", &authority)
        .json(&json!({"value":false}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        refused.status(),
        404,
        "administrator-only capability keys are not exposed by the user settings API"
    );
    let features: Value = client
        .get(format!("{}/api/features", srv.base))
        .header("Cookie", &cookie)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        features["features"]["compose.open_tracking"]["enabled"],
        true
    );
}

#[tokio::test]
async fn hot_global_and_locked_preference_denials_stop_new_and_existing_tracking_without_hiding_history()
 {
    let _guard = env(Some(PUBLIC_URL), None).await;
    let srv = serve(sqlite_db()).await;
    let cookie = seed_session(&srv, "tracking-hot").await;
    let store = tracking_store(&srv).await;
    let token = bound_token(&srv, &cookie, "hot").await;
    let path = format!("/t/{token}.gif");
    assert_eq!(fetch_image(&srv.base, &path).await.0, 200);
    assert_eq!(
        summary(&http(), &srv.base, Some(&cookie), "hot").await.1["count"],
        1
    );
    for key in ["cap.compose.open_tracking", "mail.openTracking"] {
        global_setting(&store, key, json!(false), true).await;
        assert_eq!(tracking_policy(&srv, &cookie).await["openTracking"], false);
        assert_eq!(mint(&http(), &srv.base, &cookie).await.status(), 403);
        let bind = http()
            .put(format!("{}/api/track/open/{token}", srv.base))
            .header("Cookie", &cookie)
            .json(&json!({"emailId":"hot"}))
            .send()
            .await
            .unwrap();
        assert_eq!(bind.status(), 403);
        assert_eq!(
            fetch_image(&srv.base, &path).await.0,
            200,
            "pixel remains indistinguishable from an unknown token"
        );
        assert_eq!(
            summary(&http(), &srv.base, Some(&cookie), "hot").await.1["count"],
            1
        );
        store.delete_setting_policy(key).await.unwrap();
    }
    assert_eq!(tracking_policy(&srv, &cookie).await["openTracking"], true);
    fetch_image(&srv.base, &path).await;
    assert_eq!(
        summary(&http(), &srv.base, Some(&cookie), "hot").await.1["count"],
        2
    );
}

#[tokio::test]
async fn domain_and_assigned_user_profile_controls_apply_to_existing_pixels_and_preserve_other_senders()
 {
    use mw_store::{
        feature_profiles::{FeatureProfile, ProfileAssignment, ProfileValue},
        settings_policy::{SettingScope, SettingValueRow},
    };
    let _guard = env(Some(PUBLIC_URL), None).await;
    let srv = serve(sqlite_db()).await;
    let owner = seed_session(&srv, "tracking-scoped").await;
    let other = seed_named_session(&srv, "tracking-other", "other@example.net").await;
    let store = tracking_store(&srv).await;
    let token = bound_token(&srv, &owner, "scoped").await;
    store
        .put_setting_value(&SettingValueRow {
            scope: SettingScope::Domain,
            scope_id: "example.org".into(),
            key: "cap.compose.open_tracking".into(),
            value: "false".into(),
            locked: true,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    assert_eq!(tracking_policy(&srv, &owner).await["openTracking"], false);
    assert_eq!(tracking_policy(&srv, &other).await["openTracking"], true);
    assert_eq!(mint(&http(), &srv.base, &owner).await.status(), 403);
    fetch_image(&srv.base, &format!("/t/{token}.gif")).await;
    assert_eq!(
        summary(&http(), &srv.base, Some(&owner), "scoped").await.1["count"],
        0
    );
    store
        .delete_setting_value(
            SettingScope::Domain,
            "example.org",
            "cap.compose.open_tracking",
        )
        .await
        .unwrap();
    let personal = store
        .session_policy_subject(owner.strip_prefix("mw_session=").unwrap())
        .await
        .unwrap()
        .unwrap()
        .personal_subject;
    store
        .put_feature_profile(
            &FeatureProfile {
                id: "no-tracking".into(),
                name: "No tracking".into(),
                values: [(
                    "cap.compose.open_tracking".into(),
                    ProfileValue {
                        value: json!(false),
                        locked: true,
                    },
                )]
                .into(),
                builtin: false,
                updated_at: 1,
            },
            "test",
        )
        .await
        .unwrap();
    let assignment = ProfileAssignment {
        scope_kind: "user".into(),
        scope_id: personal,
        profile_id: "no-tracking".into(),
        priority: 10,
    };
    store
        .put_feature_profile_assignment(&assignment)
        .await
        .unwrap();
    assert_eq!(tracking_policy(&srv, &owner).await["openTracking"], false);
    assert_eq!(tracking_policy(&srv, &other).await["openTracking"], true);
    assert_eq!(mint(&http(), &srv.base, &owner).await.status(), 403);
    fetch_image(&srv.base, &format!("/t/{token}.gif")).await;
    assert_eq!(
        summary(&http(), &srv.base, Some(&owner), "scoped").await.1["count"],
        0
    );
    store
        .delete_feature_profile_assignment(&assignment)
        .await
        .unwrap();
    fetch_image(&srv.base, &format!("/t/{token}.gif")).await;
    assert_eq!(
        summary(&http(), &srv.base, Some(&owner), "scoped").await.1["count"],
        1
    );
}

#[tokio::test]
async fn configured_retention_applies_on_read_record_and_mint_and_legacy_pixels_do_not_guess_an_owner()
 {
    let _guard = env(Some(PUBLIC_URL), None).await;
    let srv = serve(sqlite_db()).await;
    let cookie = seed_session(&srv, "tracking-expiry").await;
    let store = tracking_store(&srv).await;
    let template = bound_token(&srv, &cookie, "template").await;
    let subject = store
        .open_tracking_policy_subject(&mw_store::open_token_hash(&template))
        .await
        .unwrap()
        .unwrap();
    let retention = mw_store::OpenTrackingRetention::default();
    global_setting(
        &store,
        "privacy.openTracking.retentionDays",
        json!(1),
        false,
    )
    .await;
    global_setting(
        &store,
        "privacy.openTracking.unboundTtlHours",
        json!(1),
        false,
    )
    .await;
    let now = chrono::Utc::now();
    let old = "BBBBBBBBBBBBBBBBBBBBBB";
    let hash = mw_store::open_token_hash(old);
    store
        .create_open_token_with_policy(
            &hash,
            &subject.account_id,
            "old",
            "[]",
            now - chrono::Duration::days(2),
            retention,
            Some(&subject),
        )
        .await
        .unwrap();
    store
        .bind_open_token(&subject.account_id, &hash, "old")
        .await
        .unwrap();
    assert_eq!(
        fetch_image(&srv.base, &format!("/t/{old}.gif")).await.0,
        200
    );
    assert!(
        store
            .get_open_token(&subject.account_id, &hash)
            .await
            .unwrap()
            .is_none(),
        "expired token is removed on an image load without minting a new one"
    );
    let unbound = "CCCCCCCCCCCCCCCCCCCCCC";
    let hash = mw_store::open_token_hash(unbound);
    store
        .create_open_token_with_policy(
            &hash,
            &subject.account_id,
            "unbound",
            "[]",
            now - chrono::Duration::hours(2),
            retention,
            Some(&subject),
        )
        .await
        .unwrap();
    assert_eq!(
        summary(&http(), &srv.base, Some(&cookie), "template")
            .await
            .0,
        200
    );
    assert!(
        store
            .get_open_token(&subject.account_id, &hash)
            .await
            .unwrap()
            .is_none(),
        "reading history applies the configured unbound TTL"
    );
    let legacy = "DDDDDDDDDDDDDDDDDDDDDD";
    let hash = mw_store::open_token_hash(legacy);
    store
        .create_open_token(&hash, &subject.account_id, "legacy", "[]", now)
        .await
        .unwrap();
    store
        .bind_open_token(&subject.account_id, &hash, "legacy")
        .await
        .unwrap();
    fetch_image(&srv.base, &format!("/t/{legacy}.gif")).await;
    assert_eq!(
        summary(&http(), &srv.base, Some(&cookie), "legacy").await.1["count"],
        0
    );
}
