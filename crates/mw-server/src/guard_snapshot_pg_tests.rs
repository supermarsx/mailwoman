//! t49pg-e2: published request inputs and the bounded guard snapshot on
//! PostgreSQL, over the real router. Every PostgreSQL case runs only with
//! `MW_TEST_PG` and in its own schema. The cross-process cases also run on
//! SQLite (two Stores on one file) as the control.
//!
//! "Two processes" here are two AppStates, each with its own Store, publication
//! worker and EpochMirror, on one database. Nothing is shared between them in
//! memory: a commit through A reaches B only through the database.
use super::{Mode, test_control as snap};
use crate::request_inputs::test_control as inputs;
use crate::AppState;
use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::StatusCode,
};
use mw_store::{AccountKind, Credentials, DomainConfigRow, NewAccount, ServerKey, Store};
use serde_json::{Value, json};
use std::{future::Future, net::SocketAddr, sync::Arc, time::Duration};
use tower::ServiceExt;

pub(crate) const OFF: Mode = Mode::Off;
pub(crate) const BOUND: Duration = Duration::from_millis(100);
pub(crate) const SHORT: Mode = Mode::Bounded(BOUND);
pub(crate) const CHROME: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36";
pub(crate) const HOME: &str = "127.0.0.1:41000";
const DOMAIN: &str = "t49pg-snapshot.test";

pub(crate) fn pg_dsn() -> Option<String> {
    let dsn = std::env::var("MW_TEST_PG").ok().filter(|dsn| !dsn.is_empty());
    if dsn.is_none() {
        eprintln!("[guard_snapshot_pg] MW_TEST_PG unset: PostgreSQL leg not run");
    }
    dsn
}

/// A new schema on the `MW_TEST_PG` database; returns its scoped URL.
pub(crate) async fn pg_schema(dsn: &str) -> String {
    use sqlx::Connection;
    let schema = format!("t49pg_e2_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&mut admin).await.unwrap();
    admin.close().await.unwrap();
    let mut scoped = url::Url::parse(dsn).unwrap();
    scoped.query_pairs_mut().append_pair("options", &format!("-csearch_path={schema}"));
    scoped.to_string()
}

pub(crate) fn sqlite_db() -> String {
    let dir = std::env::temp_dir().join(format!("mw-t49pg-e2-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    dir.join("mail.db").to_string_lossy().into_owned()
}

/// Production-router futures exceed the default test stack.
pub(crate) fn run<F, Fut>(factory: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()>,
{
    let test = std::thread::Builder::new()
        .name("mw-t49pg-e2-test".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(factory());
        })
        .unwrap();
    if let Err(panic) = test.join() {
        std::panic::resume_unwind(panic);
    }
}

pub(crate) async fn policy(store: &Store, key: &str, value: Value) {
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "t49pg-e2-fixture".into(),
            updated_at: chrono::Utc::now().timestamp(),
        })
        .await
        .unwrap();
}

pub(crate) struct Node {
    pub router: axum::Router,
    pub state: AppState,
}
pub(crate) struct Sent {
    pub status: StatusCode,
    pub body: Value,
    pub inputs: inputs::Counts,
    pub snap: snap::Counts,
    pub watch: crate::session_watch::test_control::Counts,
}
pub(crate) struct Fixture {
    pub url: String,
    pub writer: Store,
    pub session: String,
    pub account: String,
    pub a: Node,
    pub b: Node,
}

pub(crate) async fn send(node: &Node, request: Request, mode: Mode) -> Sent {
    send_capturing(&node.router, request, mode).await.0
}

async fn send_capturing(router: &axum::Router, request: Request, mode: Mode) -> (Sent, Option<AppState>) {
    let measured = crate::session_watch::test_control::measure(snap::measure(snap::with_mode(mode,
        inputs::measure(true, router.clone().oneshot(request)))));
    let (((result, inputs), snap), watch, state) = measured.await;
    let response = result.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (Sent { status, body: serde_json::from_slice(&bytes).unwrap_or(Value::Null), inputs, snap, watch }, state)
}

impl Fixture {
    /// One account and one engine session, and two AppStates (A and B) on
    /// `url`, each with its own Store.
    pub(crate) async fn new(url: String) -> Self {
        let key = ServerKey::generate();
        let writer = Store::open(&url, key.clone()).await.unwrap();
        let user = format!("p{}@{DOMAIN}", &uuid::Uuid::new_v4().simple().to_string()[..12]);
        let credentials = Credentials { username: user.clone(), password: "fixture-secret-never-log".into() };
        writer.put_domain_config(&DomainConfigRow { name: DOMAIN.into(), upstream_json: json!({"imap":{"host":"127.0.0.1","port":9,"tls":"starttls","auth":"password","allowPrivate":true}}).to_string(), allowlist_json: "[]".into(), blocklist_json: "[]".into(), enabled: true, preferred: false, settings_json: "{}".into(), updated_at: 1 }).await.unwrap();
        let account = writer
            .create_account(&NewAccount { kind: AccountKind::Imap, host: "127.0.0.1", port: 9, tls: "starttls", username: &user, sync_policy_json: "{}" }, &credentials)
            .await
            .unwrap();
        writer.remember_account_names(&account, &[user.as_str()]).await.unwrap();
        let session = writer.create_session(&account, &user, "engine", "engine", &credentials).await.unwrap();
        writer.set_session_authentication(&session, "password").await.unwrap();
        writer
            .bind_verified_primary(
                &session,
                mw_store::identity_principal::VerifiedPrimary { account_id: &account, kind: "mailbox", authority: "fixture-native", subject: &user },
                chrono::Utc::now().timestamp(),
            )
            .await
            .unwrap()
            .unwrap();
        writer.touch_session(&session).await.unwrap();
        policy(&writer, "access.protocols", json!(["imap"])).await;
        let a = Self::node(&url, &key, &session).await;
        let b = Self::node(&url, &key, &session).await;
        assert_ne!(a.state.store.cache_scope(), b.state.store.cache_scope(), "two Stores");
        Self { url, writer, session, account, a, b }
    }
    async fn node(url: &str, key: &ServerKey, session: &str) -> Node {
        let (router, _) = crate::build_app_inner(
            crate::AppConfig {
                db_path: url.to_owned(),
                server_key_hex: Some(key.to_hex()),
                web_dir: None,
                cookie_secure: false,
                mode: crate::ServerMode::Engine,
                hardening: Default::default(),
                security: Default::default(),
            },
            crate::V6Config { admin_enabled: false, ..Default::default() },
            crate::sso::SsoProviderSource::Store,
            None,
            crate::core_test_build_info::info().unwrap(),
        )
        .await
        .unwrap();
        // The AppState, captured by the session watch of one off-mode request.
        let (reply, state) = send_capturing(&router, me_request(session, HOME, CHROME), OFF).await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
        Node { router, state: state.expect("the watch ran") }
    }
    pub(crate) fn me(&self) -> Request {
        me_request(&self.session, HOME, CHROME)
    }
    pub(crate) fn me_from(&self, peer: &str, agent: &str) -> Request {
        me_request(&self.session, peer, agent)
    }
    pub(crate) fn logout(&self) -> Request {
        let csrf = self.writer.session_csrf_token(&self.session);
        let mut request = axum::http::Request::builder()
            .method("POST")
            .uri("/api/logout")
            .header("host", "localhost")
            .header("origin", "http://localhost")
            .header("user-agent", CHROME)
            .header("cookie", format!("mw_session={}; mw_csrf={csrf}", self.session))
            .header("x-csrf-token", csrf)
            .body(Body::empty())
            .unwrap();
        request.extensions_mut().insert(axum::extract::ConnectInfo(HOME.parse::<SocketAddr>().unwrap()));
        request
    }
    /// Requests in `mode` until one ran on this session's published inputs.
    pub(crate) async fn ready(&self, node: &Node, mode: Mode) {
        for _ in 0..400 {
            let reply = send(node, self.me(), mode).await;
            assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
            if reply.inputs.first_fences == 1 || reply.snap.reused > 0 {
                return;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("request inputs never published");
    }
    /// The node's mirror (started by an earlier bounded request), once fresh.
    pub(crate) async fn mirror(&self, node: &Node) -> Arc<mw_store::authority_publication::mirror::EpochMirror> {
        let mirror = node.state.native_publication.as_ref().unwrap().existing_guard_mirror()
            .expect("a bounded request started the mirror");
        assert!(mirror.is_live());
        tokio::time::timeout(Duration::from_secs(10), async {
            while !mirror.is_fresh() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("mirror completes a poll");
        mirror
    }
    /// Bounded requests until one reused a certified fence and read none
    /// (and, with `globals`, also reused the parsed ban list).
    pub(crate) async fn reusing(&self, node: &Node, mode: Mode, globals: bool) -> Sent {
        for _ in 0..400 {
            let reply = send(node, self.me(), mode).await;
            assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
            if reply.snap.reused > 0 && reply.snap.fresh == 0 && (!globals || reply.snap.globals_reused > 0) {
                return reply;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        panic!("bounded mode never reused a certified fence");
    }
    /// This node's published request-input entry for the session, if any.
    pub(crate) fn entry(&self, node: &Node) -> bool {
        let environment = crate::request_inputs::Environment::capture().unwrap();
        node.state.native_publication.as_ref().unwrap().request_inputs(&self.session, &environment).is_some()
    }
    /// One statement on a connection of its own (PostgreSQL fixtures only).
    pub(crate) async fn pg(&self, sql: &str) -> u64 {
        use sqlx::Connection;
        let mut conn = sqlx::PgConnection::connect(&self.url).await.unwrap();
        let changed = sqlx::query(sql).execute(&mut conn).await.unwrap().rows_affected();
        conn.close().await.unwrap();
        changed
    }
}

pub(crate) fn me_request(session: &str, peer: &str, agent: &str) -> Request {
    let mut request = axum::http::Request::builder()
        .method("GET")
        .uri("/api/me")
        .header("host", "localhost")
        .header("origin", "http://localhost")
        .header("user-agent", agent)
        .header("cookie", format!("mw_session={session}"))
        .body(Body::empty())
        .unwrap();
    request.extensions_mut().insert(axum::extract::ConnectInfo(peer.parse::<SocketAddr>().unwrap()));
    request
}

// ---------------------------------------------------------------------------
// One AppState: published inputs and reuse only in bounded mode with a live,
// fresh mirror; mode off is today's PostgreSQL path.
// ---------------------------------------------------------------------------

#[test]
fn postgres_off_mode_publishes_nothing_and_bounded_mode_publishes_and_reuses() {
    let Some(dsn) = pg_dsn() else { return };
    run(move || async move {
        let f = Fixture::new(pg_schema(&dsn).await).await;
        assert!(!f.b.state.store.uses_sqlite());
        // Mode off: no published layer, no mirror, no entry, ever.
        let mut off_body = Value::Null;
        for _ in 0..6 {
            let reply = send(&f.b, f.me(), OFF).await;
            assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
            assert_eq!((reply.inputs.first_fences, reply.inputs.refresh_fences, reply.inputs.stored_reader_hits), (0, 0, 0));
            assert_eq!((reply.snap.reused, reply.snap.fresh, reply.snap.globals_reused), (0, 0, 0));
            assert!(reply.inputs.legacy_session_reads > 0, "original session reader");
            off_body = reply.body;
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        assert!(!snap::with_mode(OFF, async { super::published_inputs(&f.b.state) }).await);
        assert!(f.b.state.native_publication.as_ref().unwrap().existing_guard_mirror().is_none(), "off never starts a mirror");
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert!(!f.entry(&f.b), "off mode: no request-input entry published");
        // Bounded: the first request starts the mirror and queues the build.
        f.ready(&f.b, SHORT).await;
        assert!(f.entry(&f.b), "bounded mode publishes the entry");
        let mirror = f.mirror(&f.b).await;
        assert!(snap::with_mode(SHORT, async { super::published_inputs(&f.b.state) }).await);
        assert!(!super::declared_read_inputs(&f.b.state), "declared reads stay off on PostgreSQL");
        let hit = f.reusing(&f.b, SHORT, true).await;
        assert_eq!(hit.body, off_body, "same reply from reused DATA");
        assert_eq!((hit.inputs.first_fences, hit.inputs.refresh_fences, hit.inputs.legacy_session_reads), (0, 0, 0));
        assert!(mirror.counters().hits > 0 && mirror.counters().connects >= 1);
        let witness = snap::with_mode(SHORT, async { super::witness() }).await;
        assert!(witness["liveByBackend"]["postgres"].as_u64().unwrap() >= 1, "{witness}");
        assert_eq!(witness["mode"], "bounded");
        // Off again in the same process: the original readers, no reuse.
        let again = send(&f.b, f.me(), OFF).await;
        assert_eq!(again.status, StatusCode::OK);
        assert_eq!((again.inputs.first_fences, again.snap.reused, again.snap.fresh), (0, 0, 0));
        assert_eq!(again.body, off_body);
    });
}

/// A not-fresh mirror reuses nothing; every published request reads its fence
/// fresh. Two ways: the poll blocked (it waits on a lock on the change log,
/// which no fence statement reads) and the poll connection terminated with
/// `pg_terminate_backend` while blocked. Releasing the lock recovers.
#[test]
fn postgres_mirror_not_fresh_means_no_reuse_and_fresh_reads() {
    let Some(dsn) = pg_dsn() else { return };
    run(move || async move {
        use sqlx::Connection;
        let f = Fixture::new(pg_schema(&dsn).await).await;
        f.ready(&f.b, SHORT).await;
        let mirror = f.mirror(&f.b).await;
        f.reusing(&f.b, SHORT, false).await;
        let (connects, failures) = (mirror.counters().connects, mirror.counters().failures);
        let mut lock = sqlx::PgConnection::connect(&f.url).await.unwrap();
        sqlx::query("BEGIN").execute(&mut lock).await.unwrap();
        sqlx::query("LOCK TABLE authority_change_log IN ACCESS EXCLUSIVE MODE").execute(&mut lock).await.unwrap();
        tokio::time::timeout(Duration::from_secs(5), async {
            while mirror.is_fresh() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("a blocked poll lets the mirror go stale within the bound");
        let fr = &f;
        let check = |label: &'static str| async move {
            for _ in 0..5 {
                let reply = send(&fr.b, fr.me(), SHORT).await;
                assert_eq!(reply.status, StatusCode::OK, "{label}: {}", reply.body);
                assert_eq!(reply.snap.reused, 0, "{label}: no reuse while not fresh");
                assert_eq!(reply.snap.globals_reused, 0, "{label}: no global reuse while not fresh");
                assert!(reply.snap.fresh >= 1 && reply.inputs.first_fences == 1, "{label}: fresh fence on published inputs");
            }
        };
        check("blocked poll").await;
        // Terminate every other backend of this database, the dedicated poll
        // connection included (its application_name is not visible in
        // pg_stat_activity, so it is not singled out). The Store pools replace
        // their connections on the next acquire.
        let terminated: Vec<bool> = sqlx::query_scalar(
            "SELECT pg_catalog.pg_terminate_backend(pid) FROM pg_catalog.pg_stat_activity WHERE datname=current_database() AND backend_type='client backend' AND pid<>pg_catalog.pg_backend_pid()")
            .fetch_all(&mut lock).await.unwrap();
        assert!(terminated.iter().any(|done| *done), "backends terminated");
        let connects_before = mirror.counters().connects;
        tokio::time::timeout(Duration::from_secs(10), async {
            while mirror.counters().connects == connects_before {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("the mirror opens a new poll connection after the termination");
        assert!(!mirror.is_fresh());
        check("terminated poll connection").await;
        sqlx::query("COMMIT").execute(&mut lock).await.unwrap();
        lock.close().await.unwrap();
        // Recovery: a new poll connection, a complete poll, reuse again.
        let recovered = f.reusing(&f.b, SHORT, false).await;
        assert!(recovered.snap.reused > 0);
        assert!(mirror.counters().connects > connects, "the poll connection was reopened");
        assert!(mirror.counters().failures > failures, "polls failed while blocked");
    });
}

/// A required trigger disabled or dropped: the schema stamp changes, so no
/// fence is reused and no request-input entry is published (asserted on the
/// worker's memory directly). Re-enabling publishes again.
#[test]
fn postgres_disabled_or_dropped_required_trigger_stops_reuse_and_publication() {
    let Some(dsn) = pg_dsn() else { return };
    run(move || async move {
        use sqlx::Connection;
        let f = Fixture::new(pg_schema(&dsn).await).await;
        f.ready(&f.b, SHORT).await;
        let mirror = f.mirror(&f.b).await;
        f.reusing(&f.b, SHORT, false).await;
        let mut conn = sqlx::PgConnection::connect(&f.url).await.unwrap();
        let triggers = |table: &'static str| {
            format!("SELECT t.tgname::text FROM pg_catalog.pg_trigger t JOIN pg_catalog.pg_class c ON c.oid=t.tgrelid JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=current_schema() AND c.relname='{table}' AND NOT t.tgisinternal ORDER BY 1")
        };
        let epoch: Vec<String> = sqlx::query_scalar(&triggers("authority_epoch")).fetch_all(&mut conn).await.unwrap();
        let log = epoch.iter().find(|name| name.contains("change_log")).expect("0120 change-log trigger").clone();
        let sessions: Vec<String> = sqlx::query_scalar(&triggers("sessions")).fetch_all(&mut conn).await.unwrap();
        let session_trigger = sessions.first().expect("0111 sessions trigger").clone();
        conn.close().await.unwrap();
        let (fr, mr) = (&f, &mirror);
        let stopped = |label: String| async move {
            let flushes = mr.counters().schema_flushes;
            // Until the stale entry was discarded and no rebuild was installed.
            let mut quiet = 0;
            for _ in 0..200 {
                let reply = send(&fr.b, fr.me(), SHORT).await;
                assert_eq!(reply.status, StatusCode::OK, "{label}: {}", reply.body);
                if reply.snap.reused == 0 && reply.inputs.first_fences == 0 && !fr.entry(&fr.b) {
                    quiet += 1;
                    if quiet == 20 { break; }
                } else {
                    quiet = 0;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            assert_eq!(quiet, 20, "{label}: 20 requests over 1 s without reuse, published inputs or an entry");
            assert!(mr.counters().schema_flushes > flushes, "{label}: the stamp change flushed the mirror");
        };
        f.pg(&format!("ALTER TABLE authority_epoch DISABLE TRIGGER {log}")).await;
        stopped(format!("disabled {log}")).await;
        f.pg(&format!("ALTER TABLE authority_epoch ENABLE TRIGGER {log}")).await;
        f.ready(&f.b, SHORT).await;
        assert!(f.entry(&f.b), "re-enabled: published again");
        f.reusing(&f.b, SHORT, false).await;
        f.pg(&format!("DROP TRIGGER {session_trigger} ON sessions")).await;
        stopped(format!("dropped {session_trigger}")).await;
    });
}

// ---------------------------------------------------------------------------
// Two AppStates on one database: a commit through A stops reuse in B no later
// than B's bound, with no expedite reaching B, after B demonstrably reused.
// ---------------------------------------------------------------------------

async fn cross_process(url: String, case: &'static str) {
    let f = Fixture::new(url).await;
    f.ready(&f.a, SHORT).await;
    f.ready(&f.b, SHORT).await;
    let mirror = f.mirror(&f.b).await;
    let ban = case == "ban";
    let before = f.reusing(&f.b, SHORT, ban).await;
    assert!(before.snap.reused > 0, "{case}: B reused before the change");
    let expedites = mirror.counters().expedites;
    match case {
        "sign-out" => {
            let out = send(&f.a, f.logout(), SHORT).await;
            assert!(out.status.is_success(), "{case}: {} {}", out.status, out.body);
        }
        "session-revoke" => {
            f.a.state.store.delete_session(&f.session).await.unwrap();
            super::expedite(&f.a.state);
        }
        "account-disable" => {
            crate::account_gate::update_flags(&f.a.state.store, &f.account, |flags| flags.disabled = true).await.unwrap();
        }
        "password-change" => {
            crate::account_gate::update_flags(&f.a.state.store, &f.account, |flags| flags.force_password_change = true).await.unwrap();
        }
        "policy" => policy(&f.a.state.store, "session.absoluteSecs", json!(86_400 * 29)).await,
        "ban" => f.a.state.v6.admin.ban_ip("t49pg-e2", "127.0.0.1", "fixture", None).await.unwrap(),
        _ => unreachable!(),
    }
    // The mirror's own clock: monotonic, this process.
    let committed = tokio::time::Instant::now();
    tokio::time::sleep_until(committed + BOUND).await;
    let after = send(&f.b, f.me(), SHORT).await;
    assert_eq!(mirror.counters().expedites, expedites, "{case}: B was never expedited");
    assert_eq!(after.snap.reused, 0, "{case}: no fence reuse in B at commit + bound");
    assert_eq!(after.snap.globals_reused, 0, "{case}: no global reuse in B at commit + bound");
    match case {
        "sign-out" | "session-revoke" | "account-disable" => {
            assert_eq!(after.status, StatusCode::UNAUTHORIZED, "{case}: {}", after.body);
        }
        "password-change" => {
            assert_eq!(after.status, StatusCode::OK, "{case}: {}", after.body);
            assert_eq!(after.body["passwordChangeRequired"], true, "{case}: {}", after.body);
        }
        "policy" => {
            assert_eq!(after.status, StatusCode::OK, "{case}: {}", after.body);
            // Fresh either way: a fresh fence on the published path, or the
            // original readers when the outbox hint already evicted the entry.
            assert!(after.snap.fresh >= 1 || after.inputs.legacy_session_reads > 0,
                "{case}: read fresh ({:?}, {:?})", after.snap, after.inputs);
        }
        "ban" => assert_eq!(after.status, StatusCode::FORBIDDEN, "{case}: {}", after.body),
        _ => unreachable!(),
    }
}

const CASES: [&str; 6] = ["sign-out", "session-revoke", "account-disable", "password-change", "policy", "ban"];

// One runtime per case: dropping it ends every task that holds a Store, so
// each case's connections are closed before the next one opens its own.
#[test]
fn postgres_commit_in_one_appstate_stops_reuse_in_another_within_the_bound() {
    let Some(dsn) = pg_dsn() else { return };
    for case in CASES {
        let dsn = dsn.clone();
        run(move || async move { cross_process(pg_schema(&dsn).await, case).await });
    }
}

#[test]
fn sqlite_commit_in_one_appstate_stops_reuse_in_another_within_the_bound() {
    for case in CASES {
        run(move || async move { cross_process(sqlite_db(), case).await });
    }
}
