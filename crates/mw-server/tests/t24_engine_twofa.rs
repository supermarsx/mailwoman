//! t24-e6 (B1) — an engine-mode login finds the account it created last time, so
//! the second factor enrolled on that account is actually asked for.
//!
//! Before 26.20 every engine-mode login inserted a new `accounts` row with a random
//! id. The 2FA gate reads factors by account id, so the second login found none and
//! completed on the password alone — and under a require-2FA policy every login
//! was sent to fresh enrolment. `t16_twofa.rs` drives only proxy mode, where the
//! account id comes from the upstream and is stable, which is why nothing caught it.
//!
//! Legs:
//!   * Two engine-mode logins against a scripted POP3 server (`common::mock_mail`):
//!     a TOTP factor held by the account from login 1 is demanded on login 2, and no
//!     session cookie is issued without it.
//!   * A refused login persists no account row, and does not overwrite the stored
//!     credentials of an account that already exists.
//!   * Under an admin require-2FA policy, login 1 enrols through forced enrolment and
//!     login 2 is challenged for that factor instead of being sent to enrol again.
//!   * Migration 0028 over a POPULATED pre-0028 database, on SQLite and on live
//!     Postgres: duplicate accounts merge onto one survivor that keeps the factor and
//!     the union of both accounts' distinct cached messages, a message cached twice
//!     appears once, nothing is left pointing at the removed account, and the unique
//!     index exists and folds case identically on both backends. Where several
//!     duplicates enrolled second factors, only the earliest-enrolled account's
//!     remain, and every dropped factor has a content-free `audit_log` row.
//!
//! Run:
//!   cargo test -p mw-server --test t24_engine_twofa -- --test-threads=1
//!   DATABASE_URL_PG=postgres://… cargo test -p mw-server --test t24_engine_twofa -- --test-threads=1

use std::borrow::Cow;
use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_mfa::totp::{self, TotpParams};
use mw_server::{AppConfig, build_app};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

mod common;
use common::mock_mail::MockPop3;
use common::test_db;

const KEY_HEX: &str = "7a6b5c4d3e2f10017a6b5c4d3e2f10017a6b5c4d3e2f10017a6b5c4d3e2f1001";
const USER: &str = "owner@example.org";
const PASS: &str = "correct horse";

async fn spawn_engine_server(db_path: &str) -> SocketAddr {
    // The scripted server is plaintext; engine mode reads this per login.
    unsafe { std::env::set_var("MW_ENGINE_TLS", "plaintext") };
    let base = PathBuf::from(db_path)
        .parent()
        .unwrap()
        .join(format!("web-{}", test_db::unique_tag()));
    std::fs::create_dir_all(&base).unwrap();
    std::fs::write(base.join("index.html"), "<!doctype html><title>MW</title>").unwrap();
    let config = AppConfig {
        db_path: db_path.to_string(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(base),
        cookie_secure: false,
        mode: mw_server::ServerMode::Engine,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    };
    let app = build_app(config).await.expect("build_app");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

fn temp_db(tag: &str) -> String {
    let dir = test_db::unique_dir(&format!("mw-t24-e6-{tag}"));
    dir.join("mw.db").to_string_lossy().into_owned()
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn post(c: &reqwest::Client, url: String, body: Value) -> reqwest::Response {
    c.post(url).json(&body).send().await.unwrap()
}

fn login_body(server: &MockPop3, password: &str) -> Value {
    json!({ "jmapUrl": server.url(), "username": USER, "password": password })
}

fn set_session_cookie(resp: &reqwest::Response) -> bool {
    resp.headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .any(|c| c.starts_with("mw_session="))
}

async fn seed_store(db_path: &str) -> Store {
    Store::open(db_path, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .expect("open seed store")
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// ── two logins, one account, the factor is demanded ──────────────────────────

#[tokio::test]
async fn second_engine_login_is_challenged_for_the_factor_the_first_login_left() {
    let db = temp_db("two-logins");
    let pop = MockPop3::start(USER, PASS).await;
    let base = format!("http://{}", spawn_engine_server(&db).await);

    // Login 1: nothing enrolled, no policy → an ordinary session.
    let c1 = client();
    let r1 = post(&c1, format!("{base}/api/login"), login_body(&pop, PASS)).await;
    assert_eq!(r1.status(), 200, "login 1 succeeds");
    assert!(set_session_cookie(&r1), "login 1 issues a session");
    let b1: Value = r1.json().await.unwrap();
    let account_id = b1["accountId"].as_str().expect("accountId").to_string();

    // The owner enrols TOTP on that account.
    let store = seed_store(&db).await;
    let secret = totp::generate_secret();
    store
        .put_totp_secret(&account_id, &secret, true)
        .await
        .unwrap();

    // Login 2, a different browser, password only: challenged, no session.
    let c2 = client();
    let r2 = post(&c2, format!("{base}/api/login"), login_body(&pop, PASS)).await;
    assert_eq!(r2.status(), 200);
    assert!(
        !set_session_cookie(&r2),
        "an account with an enrolled factor must not get a session from the password alone"
    );
    let b2: Value = r2.json().await.unwrap();
    assert_eq!(
        b2["twofaRequired"],
        json!(true),
        "login 2 is challenged: {b2}"
    );
    assert!(
        b2["factors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f == "totp"),
        "the factor from login 1's account is offered: {b2}"
    );
    let pre = c2
        .get(format!("{base}/api/account/2fa"))
        .send()
        .await
        .unwrap();
    assert_eq!(pre.status(), 401, "no session before the factor clears");

    // Clearing the factor lands on the SAME account.
    let code = totp::totp_at(&secret, now_unix(), &TotpParams::default());
    let step2 = post(
        &c2,
        format!("{base}/api/login/2fa"),
        json!({ "pendingToken": b2["pendingToken"], "method": "totp", "code": code }),
    )
    .await;
    assert_eq!(step2.status(), 200, "the right code completes login 2");
    assert!(set_session_cookie(&step2));
    let s2: Value = step2.json().await.unwrap();
    assert_eq!(
        s2["accountId"],
        json!(account_id),
        "same account both times"
    );

    let accounts = store.list_accounts().await.unwrap();
    assert_eq!(
        accounts.len(),
        1,
        "two logins, one account row: {accounts:?}"
    );
    assert!(
        pop.logins_accepted() >= 2,
        "both logins reached the mail server"
    );
}

// ── a refused login writes nothing ───────────────────────────────────────────

#[tokio::test]
async fn refused_engine_login_persists_nothing_and_keeps_stored_credentials() {
    let db = temp_db("refused");
    let pop = MockPop3::start(USER, PASS).await;
    let base = format!("http://{}", spawn_engine_server(&db).await);
    let store = seed_store(&db).await;

    let r = post(
        &client(),
        format!("{base}/api/login"),
        login_body(&pop, "wrong"),
    )
    .await;
    assert_eq!(r.status(), 401, "a wrong password is refused");
    assert!(pop.logins_refused() >= 1, "the server really refused it");
    assert!(
        store.list_accounts().await.unwrap().is_empty(),
        "a refused first login must leave no account row (with sealed credentials) behind"
    );

    // With an account in place, a refused login must not re-seal its credentials.
    let ok = post(
        &client(),
        format!("{base}/api/login"),
        login_body(&pop, PASS),
    )
    .await;
    assert_eq!(ok.status(), 200);
    let account_id = ok.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .unwrap()
        .to_string();
    let bad = post(
        &client(),
        format!("{base}/api/login"),
        login_body(&pop, "wrong"),
    )
    .await;
    assert_eq!(bad.status(), 401);
    assert_eq!(
        store
            .account_credentials(&account_id)
            .await
            .unwrap()
            .password,
        PASS,
        "the stored credentials are still the ones the server accepted"
    );
    assert_eq!(store.list_accounts().await.unwrap().len(), 1);
}

// ── require-2FA policy: login 2 is challenged, not sent to enrol again ───────

#[tokio::test]
async fn policy_required_second_login_is_challenged_not_re_enrolled() {
    let db = temp_db("policy");
    let pop = MockPop3::start(USER, PASS).await;
    let base = format!("http://{}", spawn_engine_server(&db).await);
    let store = seed_store(&db).await;
    store
        .set_twofa_policy(&mw_store::TwofaPolicyRow {
            scope_kind: "global".into(),
            scope_value: String::new(),
            require_2fa: true,
            updated_by: "admin@example.org".into(),
            updated_at: String::new(),
        })
        .await
        .unwrap();

    // Login 1: nothing enrolled → forced enrolment, completed through the real routes.
    let c1 = client();
    let b1: Value = post(&c1, format!("{base}/api/login"), login_body(&pop, PASS))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(
        b1["enrollmentRequired"],
        json!(true),
        "login 1 enrols: {b1}"
    );
    let pending = b1["pendingToken"].as_str().unwrap().to_string();
    let begin: Value = post(
        &c1,
        format!("{base}/api/login/2fa/enroll/totp/begin"),
        json!({ "pendingToken": pending }),
    )
    .await
    .json()
    .await
    .unwrap();
    let secret = totp::base32_decode(begin["secret"].as_str().unwrap()).unwrap();
    let confirm = post(
        &c1,
        format!("{base}/api/login/2fa/enroll/totp/confirm"),
        json!({
            "pendingToken": pending,
            "code": totp::totp_at(&secret, now_unix(), &TotpParams::default()),
        }),
    )
    .await;
    assert_eq!(confirm.status(), 200, "forced enrolment completes");
    assert!(set_session_cookie(&confirm));

    // Login 2: the enrolled factor is demanded; no second enrolment is offered.
    let c2 = client();
    let r2 = post(&c2, format!("{base}/api/login"), login_body(&pop, PASS)).await;
    assert!(
        !set_session_cookie(&r2),
        "no session from the password alone"
    );
    let b2: Value = r2.json().await.unwrap();
    assert_eq!(b2["twofaRequired"], json!(true), "{b2}");
    assert_ne!(
        b2["enrollmentRequired"],
        json!(true),
        "login 2 must not be sent to enrol a new authenticator: {b2}"
    );
    assert!(
        b2["factors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f == "totp"),
        "the TOTP enrolled on login 1 is demanded: {b2}"
    );
    assert_eq!(store.list_accounts().await.unwrap().len(), 1);
}

// ── store: same identity, same id ────────────────────────────────────────────

#[tokio::test]
async fn store_upsert_returns_one_id_per_identity() {
    let store = Store::open(&temp_db("upsert"), ServerKey::generate())
        .await
        .unwrap();
    let acct = |host: &'static str, user: &'static str| NewAccount {
        kind: AccountKind::Imap,
        host,
        port: 993,
        tls: "implicit",
        username: user,
        sync_policy_json: "{}",
    };
    let creds = |p: &str| Credentials {
        username: USER.into(),
        password: p.into(),
    };
    let a = store
        .upsert_account_by_identity(
            &acct("mail.example.org", "Owner@Example.org"),
            &creds("one"),
        )
        .await
        .unwrap();
    let b = store
        .upsert_account_by_identity(
            &acct("MAIL.example.org.", "owner@example.ORG"),
            &creds("two"),
        )
        .await
        .unwrap();
    assert_eq!(a, b, "case and a trailing dot do not make a new identity");
    assert_eq!(store.account_credentials(&a).await.unwrap().password, "two");
    // A plain insert of the same identity is refused by the 0028 index.
    assert!(
        store
            .create_account(&acct("Mail.Example.Org", "OWNER@example.org"), &creds("x"))
            .await
            .is_err(),
        "the database refuses a second row for one identity"
    );
    let other = store
        .upsert_account_by_identity(
            &acct("mail.example.org", "someone@example.org"),
            &creds("z"),
        )
        .await
        .unwrap();
    assert_ne!(other, a);
    assert_eq!(store.list_accounts().await.unwrap().len(), 2);
}

// ── migration 0028 over a populated pre-0028 database ────────────────────────

/// The migrator for `dir`, stopped before 0028.
fn pre_0028(mut m: sqlx::migrate::Migrator) -> sqlx::migrate::Migrator {
    let kept: Vec<_> = m
        .migrations
        .iter()
        .filter(|x| x.version < 28)
        .cloned()
        .collect();
    assert!(
        m.migrations.iter().any(|x| x.version == 28),
        "0028 must exist in this migration set"
    );
    m.migrations = Cow::Owned(kept);
    m
}

/// Two accounts for ONE identity (case and a trailing dot differ), `a-dup` without
/// a factor and `b-dup` with a TOTP, so the survivor is not simply the lowest id.
/// Each holds distinct cached messages, and both hold the same upstream message
/// (INBOX, uidvalidity 7, uid 1). `n1`/`n2` differ only in non-ASCII case and must
/// stay two accounts on BOTH backends. `{B}` is the dialect's one-byte blob literal.
const SEED: &[&str] = &[
    "INSERT INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json) VALUES
        ('a-dup', 'imap', 'Mail.Example.org', 993, 'implicit', 'Alice@Example.org', {B}, '{}'),
        ('b-dup', 'imap', 'mail.example.org.', 993, 'implicit', 'alice@example.org', {B}, '{}'),
        ('n1', 'imap', 'mail.example.org', 993, 'implicit', 'ÄBC@example.org', {B}, '{}'),
        ('n2', 'imap', 'mail.example.org', 993, 'implicit', 'äbc@example.org', {B}, '{}')",
    "INSERT INTO totp_secrets (account_id, sealed_secret, confirmed, created_at)
        VALUES ('b-dup', {B}, 1, '2026-01-02T00:00:00+00:00')",
    "INSERT INTO sessions (id, account_id, username, jmap_url, api_url, sealed_creds, created_at, last_seen)
        VALUES ('sess-a', 'a-dup', 'Alice@Example.org', 'engine', 'engine', {B}, 't', 't')",
    "INSERT INTO mailboxes (id, account_id, name, uidvalidity) VALUES
        ('mbA-inbox', 'a-dup', 'INBOX', 7),
        ('mbA-arch', 'a-dup', 'Archive', 1),
        ('mbB-inbox', 'b-dup', 'INBOX', 7)",
    "INSERT INTO threads (thread_id, account_id, root_message_id) VALUES
        ('t-a', 'a-dup', '<root-1@x>'),
        ('t-b', 'b-dup', '<root-1@x>')",
    "INSERT INTO bodies (blob_ref, account_id, sealed_bytes) VALUES
        ('blob-a1', 'a-dup', {B}),
        ('blob-a2', 'a-dup', {B})",
    "INSERT INTO messages (stable_id, account_id, mailbox_id, uid, uidvalidity, thread_id, blob_ref) VALUES
        ('m-a1', 'a-dup', 'mbA-inbox', 1, 7, 't-a', 'blob-a1'),
        ('m-a2', 'a-dup', 'mbA-inbox', 2, 7, 't-a', 'blob-a2'),
        ('m-a3', 'a-dup', 'mbA-arch', 1, 1, NULL, NULL),
        ('m-b1', 'b-dup', 'mbB-inbox', 1, 7, 't-b', NULL),
        ('m-b3', 'b-dup', 'mbB-inbox', 3, 7, NULL, NULL)",
    "INSERT INTO message_meta (stable_id, pinned) VALUES ('m-a1', 1), ('m-a2', 1)",
    "INSERT INTO security_verdicts (email_id, account_id, raw_hash, verdict_json, computed_at)
        VALUES ('m-a1', 'a-dup', 'h', {B}, 't')",
    "INSERT INTO pop3_uidl (account_id, uidl, stable_id) VALUES
        ('a-dup', 'U1', 'm-a1'), ('b-dup', 'U1', 'm-b1'), ('a-dup', 'U2', 'm-a2')",
    "INSERT INTO sync_state (account_id, mailbox_id, cursor_json) VALUES
        ('a-dup', 'mbA-inbox', 'cursor-a'),
        ('a-dup', 'mbA-arch', 'cursor-a-arch'),
        ('b-dup', 'mbB-inbox', 'cursor-b')",
    "INSERT INTO identities (id, account_id, email, sent_mailbox_id)
        VALUES ('id-a', 'a-dup', 'alice@example.org', 'mbA-inbox')",
    "INSERT INTO signatures (account_id, name, body, updated_at) VALUES
        ('a-dup', 'work', 'from a', 't'), ('a-dup', 'home', 'home a', 't'),
        ('b-dup', 'work', 'from b', 't')",
    "INSERT INTO changes (account_id, type, state, stable_id, op, at) VALUES
        ('a-dup', 'Email', 1, 'm-a1', 'created', 't'),
        ('a-dup', 'Mailbox', 1, 'mbA-inbox', 'created', 't')",
    "INSERT INTO quotas (account_id, bytes_limit, msg_limit) VALUES ('a-dup', 10, 20)",
    "INSERT INTO remote_image_grants (account_id, scope_kind, scope_value, granted_at)
        VALUES ('a-dup', 'single', 'm-a1', 't')",
    "INSERT INTO assist_config (scope, enabled) VALUES ('user:a-dup', 1), ('user:b-dup', 0)",
];

/// Every table with an account key, and that key's column.
const ACCOUNT_KEYED: &[(&str, &str)] = &[
    ("sessions", "account_id"),
    ("mailboxes", "account_id"),
    ("messages", "account_id"),
    ("bodies", "account_id"),
    ("threads", "account_id"),
    ("pop3_uidl", "account_id"),
    ("sync_state", "account_id"),
    ("submissions", "account_id"),
    ("identities", "account_id"),
    ("changes", "account_id"),
    ("calendars", "account_id"),
    ("notebooks", "account_id"),
    ("notes", "account_id"),
    ("address_books", "account_id"),
    ("pim_changes", "account_id"),
    ("crypto_keys", "account_id"),
    ("key_associations", "account_id"),
    ("security_verdicts", "account_id"),
    ("dlp_audit", "account_id"),
    ("sender_controls", "account_id"),
    ("crypto_changes", "account_id"),
    ("push_subscriptions", "account_id"),
    ("native_sessions", "account_id"),
    ("api_keys", "account_id"),
    ("oauth_tokens", "account_id"),
    ("webhooks", "account_id"),
    ("quotas", "account_id"),
    ("zeroaccess_accounts", "account_id"),
    ("plugin_grants", "account_id"),
    ("password_change_audit", "account_id"),
    ("passwd_config", "account_id"),
    ("bridge_accounts", "account_id"),
    ("masked_email", "account_id"),
    ("ews_account_cred", "account_id"),
    ("uploaded_blobs", "account_id"),
    ("plugin_kv", "account_id"),
    ("totp_secrets", "account_id"),
    ("webauthn_credentials", "account_id"),
    ("recovery_codes", "account_id"),
    ("remote_image_grants", "account_id"),
    ("signatures", "account_id"),
    ("notification_rules", "account_id"),
    ("message_embeddings", "account_id"),
    ("bridge_oauth_tokens", "bridge_account_id"),
    ("tags", "\"user\""),
    ("saved_searches", "\"user\""),
];

/// Seed, open (which applies 0028), then assert. A macro so one body drives both
/// pool types.
macro_rules! d5_merge_scenario {
    ($pool:expr, $blob:expr, $open:expr, $index_count_sql:expr) => {{
        let pool = $pool;
        for stmt in SEED {
            sqlx::query(&stmt.replace("{B}", $blob))
                .execute(&pool)
                .await
                .unwrap_or_else(|e| panic!("seed failed: {e}\n{stmt}"));
        }

        let store: Store = $open.await;

        let one = |sql: &'static str| {
            let pool = pool.clone();
            async move {
                sqlx::query_scalar::<_, i64>(sql)
                    .fetch_one(&pool)
                    .await
                    .unwrap_or_else(|e| panic!("{e}\n{sql}"))
            }
        };
        let strings = |sql: &'static str| {
            let pool = pool.clone();
            async move {
                let mut v = sqlx::query_scalar::<_, String>(sql)
                    .fetch_all(&pool)
                    .await
                    .unwrap_or_else(|e| panic!("{e}\n{sql}"));
                v.sort();
                v
            }
        };

        // One survivor per identity; the non-ASCII pair stays two accounts.
        assert_eq!(
            strings("SELECT id FROM accounts").await,
            vec!["b-dup", "n1", "n2"],
            "the duplicate merged onto the factor holder; ÄBC and äbc are distinct"
        );
        // The factor is preserved on the survivor, and no factor was dropped.
        assert_eq!(one("SELECT COUNT(*) FROM audit_log").await, 0);
        assert_eq!(
            one("SELECT COUNT(*) FROM totp_secrets WHERE account_id = 'b-dup' AND confirmed = 1").await,
            1
        );
        // Union of distinct messages; the identical one exactly once.
        assert_eq!(
            strings("SELECT stable_id FROM messages WHERE account_id = 'b-dup'").await,
            vec!["m-a2", "m-a3", "m-b1", "m-b3"],
            "every distinct message is kept, the copy cached twice is kept once"
        );
        assert_eq!(
            one("SELECT COUNT(*) FROM messages m, mailboxes b
                  WHERE m.mailbox_id = b.id AND b.name = 'INBOX' AND m.uidvalidity = 7 AND m.uid = 1").await,
            1
        );
        assert_eq!(
            strings("SELECT id FROM mailboxes WHERE account_id = 'b-dup'").await,
            vec!["mbA-arch", "mbB-inbox"]
        );
        assert_eq!(
            strings("SELECT mailbox_id FROM messages WHERE stable_id = 'm-a2'").await,
            vec!["mbB-inbox"]
        );
        // The kept copy inherits the dropped copy's body and metadata.
        assert_eq!(
            strings("SELECT blob_ref FROM messages WHERE stable_id = 'm-b1'").await,
            vec!["blob-a1"]
        );
        assert_eq!(
            strings("SELECT blob_ref FROM bodies WHERE account_id = 'b-dup'").await,
            vec!["blob-a1", "blob-a2"]
        );
        assert_eq!(one("SELECT pinned FROM message_meta WHERE stable_id = 'm-b1'").await, 1);
        assert_eq!(
            strings("SELECT email_id FROM security_verdicts WHERE account_id = 'b-dup'").await,
            vec!["m-b1"]
        );
        assert_eq!(
            strings("SELECT scope_value FROM remote_image_grants WHERE account_id = 'b-dup'").await,
            vec!["m-b1"]
        );
        // Threads, UIDLs, cursors, identities, signatures, settings.
        assert_eq!(strings("SELECT thread_id FROM threads").await, vec!["t-b"]);
        assert_eq!(
            strings("SELECT thread_id FROM messages WHERE stable_id IN ('m-a2', 'm-b1')").await,
            vec!["t-b", "t-b"]
        );
        assert_eq!(
            strings("SELECT stable_id FROM pop3_uidl WHERE account_id = 'b-dup'").await,
            vec!["m-a2", "m-b1"]
        );
        assert_eq!(
            strings("SELECT cursor_json FROM sync_state WHERE account_id = 'b-dup'").await,
            vec!["cursor-a-arch", "cursor-b"]
        );
        assert_eq!(
            strings("SELECT sent_mailbox_id FROM identities WHERE account_id = 'b-dup'").await,
            vec!["mbB-inbox"]
        );
        assert_eq!(
            strings("SELECT body FROM signatures WHERE account_id = 'b-dup'").await,
            vec!["from b", "home a"]
        );
        assert_eq!(
            strings("SELECT stable_id FROM changes WHERE account_id = 'b-dup'").await,
            vec!["m-b1", "mbB-inbox"]
        );
        assert_eq!(one("SELECT COUNT(*) FROM quotas WHERE account_id = 'b-dup'").await, 1);
        assert_eq!(strings("SELECT scope FROM assist_config").await, vec!["user:b-dup"]);
        assert_eq!(
            strings("SELECT account_id FROM sessions").await,
            vec!["b-dup"],
            "an existing session now names the survivor"
        );

        // Nothing is left pointing at the removed account, or at removed rows.
        for (table, col) in ACCOUNT_KEYED {
            let sql = format!("SELECT COUNT(*) FROM {table} WHERE {col} = 'a-dup'");
            let n: i64 = sqlx::query_scalar(&sql).fetch_one(&pool).await.unwrap();
            assert_eq!(n, 0, "{table}.{col} still references the removed account");
        }
        for (label, sql) in [
            ("messages→accounts", "SELECT COUNT(*) FROM messages WHERE account_id NOT IN (SELECT id FROM accounts)"),
            ("messages→mailboxes", "SELECT COUNT(*) FROM messages WHERE mailbox_id NOT IN (SELECT id FROM mailboxes)"),
            ("sync_state→mailboxes", "SELECT COUNT(*) FROM sync_state WHERE mailbox_id NOT IN (SELECT id FROM mailboxes)"),
            ("message_meta→messages", "SELECT COUNT(*) FROM message_meta WHERE stable_id NOT IN (SELECT stable_id FROM messages)"),
            ("pop3_uidl→messages", "SELECT COUNT(*) FROM pop3_uidl WHERE stable_id NOT IN (SELECT stable_id FROM messages)"),
            ("verdicts→messages", "SELECT COUNT(*) FROM security_verdicts WHERE email_id NOT IN (SELECT stable_id FROM messages)"),
            ("messages→threads", "SELECT COUNT(*) FROM messages WHERE thread_id IS NOT NULL AND thread_id NOT IN (SELECT thread_id FROM threads)"),
            ("messages→bodies", "SELECT COUNT(*) FROM messages WHERE blob_ref IS NOT NULL AND blob_ref NOT IN (SELECT blob_ref FROM bodies)"),
            ("bodies unreferenced", "SELECT COUNT(*) FROM bodies WHERE blob_ref NOT IN (SELECT blob_ref FROM messages WHERE blob_ref IS NOT NULL)"),
        ] {
            assert_eq!(one(sql).await, 0, "orphaned rows: {label}");
        }

        // The unique index exists and folds case the same way the lookup does.
        assert_eq!(one($index_count_sql).await, 1, "idx_accounts_identity exists");
        let dup = sqlx::query(&format!(
            "INSERT INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json)
             VALUES ('c-dup', 'imap', 'MAIL.EXAMPLE.ORG', 993, 'implicit', 'ALICE@EXAMPLE.ORG', {}, '{{}}')",
            $blob
        ))
        .execute(&pool)
        .await;
        assert!(dup.is_err(), "the index refuses an ASCII case variant");
        assert_eq!(
            store
                .account_id_by_identity(AccountKind::Imap, "MAIL.example.ORG", 993, "ALICE@example.org")
                .await
                .unwrap()
                .as_deref(),
            Some("b-dup")
        );
        assert_eq!(
            store
                .account_id_by_identity(AccountKind::Imap, "mail.example.org", 993, "äbc@example.org")
                .await
                .unwrap()
                .as_deref(),
            Some("n2"),
            "non-ASCII case is not folded by the lookup on this backend either"
        );
        store
    }};
}

/// Three accounts for one POP3 identity. `x3` holds the EARLIEST factor (a TOTP
/// created 02-01, plus a recovery code) and survives although it has the highest
/// id. `x2` enrolled later (03-01): a confirmed TOTP, a passkey and a recovery code.
/// `x1` holds only a pending, unconfirmed TOTP. Under the t24 rule only `x3`'s
/// factors remain, and each of the others is recorded in `audit_log` without its
/// secret material (`{S}` / `{K}` are recognisable bytes the audit must not carry).
///
/// The mail cache still merges fully: the survivor has no INBOX, so the two
/// duplicates' INBOX copies collide with EACH OTHER and the rank-2 account's copy
/// (`x2`) is kept; the same holds for the message both of them cached (uid 42).
const THREE_WAY: &[&str] = &[
    "INSERT INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json) VALUES
        ('x1', 'pop3', 'pop.example.org', 995, 'implicit', 'bob', {B}, '{}'),
        ('x2', 'pop3', 'POP.example.org', 995, 'implicit', 'Bob', {B}, '{}'),
        ('x3', 'pop3', 'pop.example.org', 995, 'implicit', 'BOB', {B}, '{}')",
    "INSERT INTO totp_secrets (account_id, sealed_secret, confirmed, created_at) VALUES
        ('x1', {B}, 0, '2026-01-01T00:00:00+00:00'),
        ('x2', {S}, 1, '2026-03-01T00:00:00+00:00'),
        ('x3', {B}, 1, '2026-02-01T00:00:00+00:00')",
    "INSERT INTO webauthn_credentials (credential_id, account_id, cose_public_key, created_at)
        VALUES ('pk2', 'x2', {K}, '2026-03-02T00:00:00+00:00')",
    "INSERT INTO recovery_codes (account_id, code_hash, created_at) VALUES
        ('x3', 'rc3', '2026-02-01T00:00:00+00:00'),
        ('x2', 'RC2-ARGON2-HASH', '2026-03-01T00:00:00+00:00')",
    "INSERT INTO mailboxes (id, account_id, name, uidvalidity) VALUES
        ('mb1', 'x1', 'INBOX', 0), ('mb2', 'x2', 'INBOX', 0)",
    "INSERT INTO messages (stable_id, account_id, mailbox_id, uid, uidvalidity) VALUES
        ('p1', 'x1', 'mb1', 42, 0), ('p1b', 'x1', 'mb1', 43, 0), ('p2', 'x2', 'mb2', 42, 0)",
    "INSERT INTO message_meta (stable_id, pinned) VALUES ('p1', 1)",
    "INSERT INTO message_embeddings (stable_id, account_id, model, dim, vector_sealed, updated_at) VALUES
        ('p1', 'x1', 'm1', 1, {B}, 't'), ('p2', 'x2', 'm2', 1, {B}, 't')",
    "INSERT INTO pop3_uidl (account_id, uidl, stable_id) VALUES
        ('x1', 'UIDL-42', 'p1'), ('x2', 'UIDL-42', 'p2'), ('x1', 'UIDL-43', 'p1b')",
    "INSERT INTO sync_state (account_id, mailbox_id, cursor_json) VALUES
        ('x1', 'mb1', 'c1'), ('x2', 'mb2', 'c2')",
];

/// Bytes of the later duplicate's TOTP secret and passkey key, as seeded.
const X2_TOTP_SECRET: &[u8] = b"TOTP-SECRET-OF-X2";
const X2_PASSKEY_KEY: &[u8] = b"COSE-KEY-OF-X2";

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

macro_rules! three_way_scenario {
    ($pool:expr, $blob:expr, $bytes_literal:expr, $open:expr) => {{
        let pool = $pool;
        let bytes_literal: fn(&[u8]) -> String = $bytes_literal;
        for stmt in THREE_WAY {
            let sql = stmt
                .replace("{B}", $blob)
                .replace("{S}", &bytes_literal(X2_TOTP_SECRET))
                .replace("{K}", &bytes_literal(X2_PASSKEY_KEY));
            sqlx::query(&sql)
                .execute(&pool)
                .await
                .unwrap_or_else(|e| panic!("seed failed: {e}\n{sql}"));
        }
        let _store: Store = $open.await;
        let strings = |sql: &'static str| {
            let pool = pool.clone();
            async move {
                let mut v = sqlx::query_scalar::<_, String>(sql)
                    .fetch_all(&pool)
                    .await
                    .unwrap_or_else(|e| panic!("{e}\n{sql}"));
                v.sort();
                v
            }
        };
        assert_eq!(
            strings("SELECT id FROM accounts").await,
            vec!["x3"],
            "the earliest-enrolled account survives, not the lowest id"
        );

        // Only the survivor's own factors remain.
        assert_eq!(
            strings("SELECT account_id || ' ' || created_at FROM totp_secrets").await,
            vec!["x3 2026-02-01T00:00:00+00:00"],
            "the later duplicate's TOTP (and x1's pending one) are gone"
        );
        assert!(
            strings("SELECT credential_id FROM webauthn_credentials")
                .await
                .is_empty(),
            "the later duplicate's passkey is gone, not moved"
        );
        assert_eq!(
            strings("SELECT account_id || ' ' || code_hash FROM recovery_codes").await,
            vec!["x3 rc3"],
            "only the survivor's recovery codes remain"
        );

        // One content-free audit row per dropped factor.
        let rows: Vec<(String, String, String, String, String)> =
            sqlx::query_as("SELECT actor, actor_kind, action, target, detail_json FROM audit_log")
                .fetch_all(&pool)
                .await
                .unwrap();
        let mut details: Vec<serde_json::Value> = Vec::new();
        for (actor, actor_kind, action, target, detail) in &rows {
            assert_eq!(
                (
                    actor.as_str(),
                    actor_kind.as_str(),
                    action.as_str(),
                    target.as_str()
                ),
                (
                    "migration-0028",
                    "system",
                    "twofa-factor-dropped-on-merge",
                    "x3"
                )
            );
            let v: serde_json::Value = serde_json::from_str(detail).expect("detail_json is JSON");
            assert_eq!(v["survivorAccountId"], json!("x3"), "{v}");
            details.push(v);
        }
        let find = |factor: &str, from: &str| {
            details
                .iter()
                .filter(|v| v["factor"] == json!(factor) && v["duplicateAccountId"] == json!(from))
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(
            rows.len(),
            4,
            "one audit row per dropped factor: {details:?}"
        );
        let x2_totp = find("totp", "x2");
        assert_eq!(x2_totp.len(), 1, "{details:?}");
        assert_eq!(x2_totp[0]["confirmed"], json!(true));
        assert_eq!(x2_totp[0]["createdAt"], json!("2026-03-01T00:00:00+00:00"));
        let x2_passkey = find("passkey", "x2");
        assert_eq!(x2_passkey.len(), 1, "{details:?}");
        assert_eq!(x2_passkey[0]["credentialId"], json!("pk2"));
        assert_eq!(
            x2_passkey[0]["createdAt"],
            json!("2026-03-02T00:00:00+00:00")
        );
        let x2_codes = find("recovery-codes", "x2");
        assert_eq!(x2_codes.len(), 1, "{details:?}");
        assert_eq!(
            (
                x2_codes[0]["count"].as_i64(),
                x2_codes[0]["unused"].as_i64()
            ),
            (Some(1), Some(1))
        );
        let x1_pending = find("totp", "x1");
        assert_eq!(x1_pending.len(), 1, "{details:?}");
        assert_eq!(x1_pending[0]["confirmed"], json!(false));

        // No secret material in any audit column, in any encoding.
        let all: Vec<(
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, ts, actor, actor_kind, action, target, detail_json, ip FROM audit_log",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        let text = format!("{all:?}").to_ascii_lowercase();
        for needle in [
            String::from_utf8_lossy(X2_TOTP_SECRET).to_ascii_lowercase(),
            hex(X2_TOTP_SECRET),
            String::from_utf8_lossy(X2_PASSKEY_KEY).to_ascii_lowercase(),
            hex(X2_PASSKEY_KEY),
            "rc2-argon2-hash".to_string(),
        ] {
            assert!(
                !text.contains(&needle),
                "audit_log carries secret material ({needle}): {text}"
            );
        }

        // The mail cache still merges fully.
        assert_eq!(strings("SELECT id FROM mailboxes").await, vec!["mb2"]);
        assert_eq!(
            strings("SELECT stable_id FROM messages WHERE account_id = 'x3'").await,
            vec!["p1b", "p2"],
            "the duplicates' shared message is kept once, x1's own message is moved"
        );
        assert_eq!(
            strings("SELECT mailbox_id FROM messages").await,
            vec!["mb2", "mb2"]
        );
        assert_eq!(
            strings("SELECT stable_id FROM message_meta").await,
            vec!["p2"],
            "the dropped copy's metadata moved to the kept copy, which had none"
        );
        assert_eq!(
            strings("SELECT model FROM message_embeddings").await,
            vec!["m2"],
            "the kept copy keeps its own embedding"
        );
        assert_eq!(
            strings("SELECT uidl || '=' || stable_id FROM pop3_uidl WHERE account_id = 'x3'").await,
            vec!["UIDL-42=p2", "UIDL-43=p1b"]
        );
        assert_eq!(
            strings("SELECT cursor_json FROM sync_state").await,
            vec!["c2"]
        );
        for (table, col) in ACCOUNT_KEYED {
            let sql = format!("SELECT COUNT(*) FROM {table} WHERE {col} IN ('x1', 'x2')");
            let n: i64 = sqlx::query_scalar(&sql).fetch_one(&pool).await.unwrap();
            assert_eq!(n, 0, "{table}.{col} still references a removed account");
        }
    }};
}

/// A fresh SQLite database at the pre-0028 schema.
async fn sqlite_at_0027(tag: &str) -> (String, sqlx::SqlitePool) {
    let path = temp_db(tag);
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{path}?mode=rwc"))
        .await
        .unwrap();
    pre_0028(sqlx::migrate!("../mw-store/migrations"))
        .run(&pool)
        .await
        .expect("apply 0001..0027");
    (path, pool)
}

/// A private Postgres schema at the pre-0028 schema: the shared test database may
/// already be at 0028. Returns the schema name, a DSN scoped to it, and a pool.
async fn postgres_at_0027(dsn: &str) -> (String, String, sqlx::PgPool) {
    let schema = format!("mw_t24_mig_{}", test_db::unique_tag().replace('-', "_"));
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(dsn)
        .await
        .expect("connect to Postgres");
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await
        .unwrap();
    admin.close().await;
    let sep = if dsn.contains('?') { '&' } else { '?' };
    let scoped = format!("{dsn}{sep}options[search_path]={schema}");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&scoped)
        .await
        .unwrap();
    pre_0028(sqlx::migrate!("../mw-store/migrations_pg"))
        .run(&pool)
        .await
        .expect("apply 0001..0027 on Postgres");
    (schema, scoped, pool)
}

async fn drop_schema(dsn: &str, schema: &str) {
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(dsn)
        .await
        .unwrap();
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&admin)
        .await
        .unwrap();
}

#[tokio::test]
async fn migration_0028_merges_a_populated_sqlite_database_keeping_all_data() {
    let (path, pool) = sqlite_at_0027("mig-sqlite").await;
    let key = ServerKey::generate();
    let store = d5_merge_scenario!(
        pool.clone(),
        "X'00'",
        async {
            Store::open(&path, key.clone())
                .await
                .expect("open applies 0028")
        },
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_accounts_identity'"
    );
    drop(store);
    // Opening again is a no-op: nothing to apply, nothing changes.
    Store::open(&path, key).await.expect("reopen");
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 4);

    let (path, pool) = sqlite_at_0027("mig-sqlite-3way").await;
    three_way_scenario!(pool, "X'00'", |b| format!("X'{}'", hex(b)), async {
        Store::open(&path, ServerKey::generate())
            .await
            .expect("open applies 0028")
    });
}

#[tokio::test]
async fn migration_0028_merges_a_populated_postgres_database_keeping_all_data() {
    let Some(dsn) = common::gate::pg_dsn() else {
        common::gate::skip(
            "[t24 0028] MW_E14_PG_DSN and DATABASE_URL_PG unset — the merge migration is not proven on Postgres.",
        );
        return;
    };
    let (schema, scoped, pool) = postgres_at_0027(&dsn).await;
    let key = ServerKey::generate();
    let index_sql: &'static str = Box::leak(
        format!(
            "SELECT COUNT(*) FROM pg_indexes WHERE schemaname = '{schema}' AND indexname = 'idx_accounts_identity'"
        )
        .into_boxed_str(),
    );
    let store = d5_merge_scenario!(
        pool.clone(),
        "'\\x00'::bytea",
        async {
            Store::open(&scoped, key.clone())
                .await
                .expect("open applies 0028 on Postgres")
        },
        index_sql
    );
    drop(store);
    Store::open(&scoped, key).await.expect("reopen on Postgres");
    pool.close().await;
    drop_schema(&dsn, &schema).await;

    let (schema3, scoped3, pool3) = postgres_at_0027(&dsn).await;
    three_way_scenario!(
        pool3.clone(),
        "'\\x00'::bytea",
        |b| format!("'\\x{}'::bytea", hex(b)),
        async {
            Store::open(&scoped3, ServerKey::generate())
                .await
                .expect("open applies 0028 on Postgres")
        }
    );
    pool3.close().await;
    drop_schema(&dsn, &schema3).await;
    eprintln!("[t24 0028] Postgres leg RAN in schemas {schema} and {schema3}");
}
