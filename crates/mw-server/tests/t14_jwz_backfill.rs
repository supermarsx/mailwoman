//! t14-E-e2e — LEG 3: JWZ historical backfill correctness through the admin
//! endpoint, on a REAL store (SQLite always; live Postgres via `MW_E14_PG_DSN`).
//!
//! Seeds an OUT-OF-ORDER corpus (replies — one with a TRUNCATED References chain —
//! ingested before their original, plus an unrelated standalone), then drives the
//! admin-gated `POST /admin/maintenance/rethread { accountId }` (E-mount) end-to-end
//! and asserts:
//!   * the summary reports the full-set JWZ grouping — `messages: 5`, `threads: 2`
//!     (the 4-message reply chain converges into ONE thread, the standalone is its
//!     own), `reassigned: 5` (every message moved from unthreaded),
//!   * re-keying is idempotent — a SECOND POST returns `reassigned: 0`, and
//!   * reading the store back, the whole reply chain (incl. the truncated-References
//!     follow-up) shares one `thread_id` and the standalone is distinct.
//!
//! This is the "wired" proof of WS3 against the admin HTTP surface + the real store
//! dialect (the engine unit test pins the algorithm; this pins the endpoint + store
//! round-trip on both SQLite and live Postgres).
//!
//! ## Running
//!   cargo test -p mw-server --test t14_jwz_backfill                     # SQLite always
//!   docker compose -f docker-compose.ci.yml up -d --wait postgres
//!   MW_E14_PG_DSN=postgres://mailwoman:mailwoman@localhost:5432/mailwoman \
//!     cargo test -p mw-server --test t14_jwz_backfill -- --nocapture --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
};

const ADMIN_USER: &str = "root";
const ADMIN_PASS: &str = "hunter2";
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

fn web_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mw-t14-jwz-web-{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("index.html"),
        "<!doctype html><title>MW</title><div id=app>MW</div>",
    )
    .unwrap();
    dir
}

/// A minimal RFC822 message carrying the threading headers under test.
fn raw_msg(mid: &str, refs: &[&str], irt: Option<&str>, subject: &str) -> Vec<u8> {
    let mut h = format!(
        "Message-ID: <{mid}>\r\nFrom: s@x\r\nTo: me@x\r\nSubject: {subject}\r\n\
         Date: Wed, 01 Jul 2026 09:00:00 +0000\r\n"
    );
    if !refs.is_empty() {
        let joined = refs
            .iter()
            .map(|r| format!("<{r}>"))
            .collect::<Vec<_>>()
            .join(" ");
        h.push_str(&format!("References: {joined}\r\n"));
    }
    if let Some(i) = irt {
        h.push_str(&format!("In-Reply-To: <{i}>\r\n"));
    }
    h.push_str("\r\nbody text\r\n");
    h.into_bytes()
}

async fn seed(
    store: &Store,
    account: &str,
    mailbox: &str,
    uid: u32,
    mid: &str,
    raw: &[u8],
) -> String {
    let blob = store.put_body(account, raw).await.unwrap();
    store
        .upsert_message(&MessageUpsert {
            account_id: account,
            mailbox_id: mailbox,
            uid,
            uidvalidity: 1,
            message_id: Some(mid),
            thread_id: None,
            internaldate: Some("2026-07-01T09:00:00Z"),
            size: raw.len() as u64,
            flags_json: "[]",
            envelope: None,
            blob_ref: Some(&blob),
        })
        .await
        .unwrap()
}

/// Seed the out-of-order corpus into `db_path`; return `(account_id, [o,a,b,c,other])`
/// stable ids for the post-run store check.
async fn seed_corpus(db_path: &str) -> (String, [String; 5]) {
    let key = ServerKey::from_hex(SERVER_KEY_HEX).unwrap();
    let store = Store::open(db_path, key).await.expect("open seed store");
    // Unique username so a persistent Postgres never collides across runs.
    let uname = format!("me-{}@x", unique());
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: &uname,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: uname.clone(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    let mailbox = store
        .upsert_mailbox(&MailboxUpsert {
            account_id: &account,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 1,
            uidnext: 100,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        })
        .await
        .unwrap();

    // OUT OF ORDER: replies (one truncated to only reach a@x) before the original.
    let b = seed(
        &store,
        &account,
        &mailbox,
        1,
        "b@x",
        &raw_msg("b@x", &["o@x", "a@x"], Some("a@x"), "Re: Plan"),
    )
    .await;
    let c = seed(
        &store,
        &account,
        &mailbox,
        2,
        "c@x",
        &raw_msg("c@x", &["a@x"], Some("a@x"), "Re: Plan"),
    )
    .await;
    let a = seed(
        &store,
        &account,
        &mailbox,
        3,
        "a@x",
        &raw_msg("a@x", &["o@x"], Some("o@x"), "Re: Plan"),
    )
    .await;
    let o = seed(
        &store,
        &account,
        &mailbox,
        4,
        "o@x",
        &raw_msg("o@x", &[], None, "Plan"),
    )
    .await;
    let other = seed(
        &store,
        &account,
        &mailbox,
        5,
        "other@x",
        &raw_msg("other@x", &[], None, "Something else"),
    )
    .await;
    (account, [o, a, b, c, other])
}

async fn spawn(db_path: String) -> String {
    let config = AppConfig {
        db_path,
        server_key_hex: Some(SERVER_KEY_HEX.into()),
        web_dir: Some(web_dir()),
        cookie_secure: false,
        mode: ServerMode::Engine,
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

async fn admin_login(c: &reqwest::Client, base: &str) -> String {
    let resp = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": ADMIN_USER, "password": ADMIN_PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "admin login succeeds");
    resp.headers()
        .get(reqwest::header::SET_COOKIE)
        .expect("login sets a cookie")
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string()
}

async fn rethread(c: &reqwest::Client, base: &str, cookie: &str, account: &str) -> Value {
    let resp = c
        .post(format!("{base}/admin/maintenance/rethread"))
        .header(reqwest::header::COOKIE, cookie)
        .json(&json!({ "accountId": account }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "admin drives the backfill");
    resp.json().await.unwrap()
}

/// Drive the full backfill leg against a store opened at `db_path`, then verify the
/// stored thread_ids converged. `dialect` labels assertions.
async fn drive(db_path: &str, dialect: &str) {
    let (account, [o, a, b, c, other]) = seed_corpus(db_path).await;
    let base = spawn(db_path.to_string()).await;
    let cl = reqwest::Client::new();
    let cookie = admin_login(&cl, &base).await;

    // ── First run: full-set grouping ──
    let s1 = rethread(&cl, &base, &cookie, &account).await;
    assert_eq!(s1["accounts"], 1, "[{dialect}] one account: {s1}");
    assert_eq!(s1["messages"], 5, "[{dialect}] five stored messages: {s1}");
    assert_eq!(
        s1["threads"], 2,
        "[{dialect}] full-set JWZ grouping = 2 threads (chain + standalone): {s1}"
    );
    assert_eq!(
        s1["reassigned"], 5,
        "[{dialect}] every message moved from unthreaded: {s1}"
    );

    // ── Second run: idempotent no-op ──
    let s2 = rethread(&cl, &base, &cookie, &account).await;
    assert_eq!(s2["messages"], 5, "[{dialect}] re-run messages: {s2}");
    assert_eq!(s2["threads"], 2, "[{dialect}] re-run threads: {s2}");
    assert_eq!(
        s2["reassigned"], 0,
        "[{dialect}] re-run reassigns nothing (idempotent): {s2}"
    );

    // ── Read the store back: the chain converged, the standalone is distinct. ──
    let key = ServerKey::from_hex(SERVER_KEY_HEX).unwrap();
    let store = Store::open(db_path, key).await.expect("reopen store");
    let tid = |id: &str| {
        let store = &store;
        let id = id.to_string();
        async move { store.get_message(&id).await.unwrap().thread_id }
    };
    let to = tid(&o).await;
    assert!(to.is_some(), "[{dialect}] original is threaded");
    assert_eq!(tid(&a).await, to, "[{dialect}] a converges with original");
    assert_eq!(tid(&b).await, to, "[{dialect}] b converges with original");
    assert_eq!(
        tid(&c).await,
        to,
        "[{dialect}] truncated-References c still converges"
    );
    let tother = tid(&other).await;
    assert!(tother.is_some(), "[{dialect}] standalone is threaded");
    assert_ne!(
        tother, to,
        "[{dialect}] the standalone is a distinct thread"
    );
}

#[tokio::test]
async fn jwz_backfill_endpoint_converges_and_idempotent_sqlite() {
    let db = std::env::temp_dir().join(format!("mw-t14-jwz-{}.db", unique()));
    drive(&db.to_string_lossy(), "sqlite").await;
}

#[tokio::test]
async fn jwz_backfill_endpoint_converges_and_idempotent_postgres() {
    let Ok(dsn) = std::env::var("MW_E14_PG_DSN") else {
        eprintln!("\n[t14 JWZ SKIP] MW_E14_PG_DSN unset — live Postgres backfill not driven.\n");
        return;
    };
    drive(&dsn, "postgres").await;
}
