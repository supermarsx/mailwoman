//! t50hp B1: per-request statement budgets for the guard chain on the actual
//! production router (no guard, store or policy getter is mocked). Published
//! request inputs and the guard snapshot are not enabled here, so every guard
//! takes its direct-read path: SQLite "off" and, with `MW_TEST_PG`, PostgreSQL.
//! A budget is an upper bound that a change must not raise; lower it when a
//! change removes statements.
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request as HttpRequest, StatusCode},
};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};
use serde_json::{Value, json};
use std::future::Future;
use tower::ServiceExt;

struct Fixture {
    router: Router,
    writer: Store,
    session: String,
    label: &'static str,
}

impl Fixture {
    async fn new(db_path: String, label: &'static str) -> Self {
        let key = ServerKey::generate();
        let writer = Store::open(&db_path, key.clone()).await.unwrap();
        let username = format!("budget-{}@guard-budget.test", uuid::Uuid::new_v4().simple());
        let credentials = Credentials {
            username: username.clone(),
            password: "fixture-only-not-a-real-password".into(),
        };
        let account = writer
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "127.0.0.1",
                    port: 9,
                    tls: "starttls",
                    username: &credentials.username,
                    sync_policy_json: "{}",
                },
                &credentials,
            )
            .await
            .unwrap();
        let session = writer
            .create_session(&account, &credentials.username, "engine", "engine", &credentials)
            .await
            .unwrap();
        writer.set_session_authentication(&session, "password").await.unwrap();
        writer
            .bind_verified_primary(
                &session,
                mw_store::identity_principal::VerifiedPrimary {
                    account_id: &account,
                    kind: "mailbox",
                    authority: "guard-budget-fixture",
                    subject: &account,
                },
                chrono::Utc::now().timestamp(),
            )
            .await
            .unwrap()
            .unwrap();
        let (router, _push) = crate::build_app_full(
            crate::AppConfig {
                db_path,
                server_key_hex: Some(key.to_hex()),
                web_dir: None,
                cookie_secure: false,
                mode: crate::ServerMode::Engine,
                hardening: Default::default(),
                security: Default::default(),
            },
            crate::V6Config { admin_enabled: false, ..Default::default() },
        )
        .await
        .unwrap();
        Self { router, writer, session, label }
    }

    fn me(&self) -> HttpRequest<Body> {
        self.me_from("127.0.0.1:48219")
    }

    fn me_from(&self, peer: &str) -> HttpRequest<Body> {
        let mut request = HttpRequest::builder()
            .method("GET")
            .uri("/api/me")
            .header("host", "localhost")
            .header("user-agent", "Mozilla/5.0")
            .header("cookie", format!("mw_session={}", self.session))
            .body(Body::empty())
            .unwrap();
        request.extensions_mut().insert(axum::extract::ConnectInfo(
            peer.parse::<std::net::SocketAddr>().unwrap(),
        ));
        request
    }

    fn echo(&self) -> HttpRequest<Body> {
        let csrf = self.writer.session_csrf_token(&self.session);
        let mut request = HttpRequest::builder()
            .method("POST")
            .uri("/jmap/api")
            .header("host", "localhost")
            .header("origin", "http://localhost")
            .header("user-agent", "Mozilla/5.0")
            .header("content-type", "application/json")
            .header("cookie", format!("mw_session={}; mw_csrf={csrf}", self.session))
            .header("x-csrf-token", csrf)
            .body(Body::from(
                serde_json::to_vec(&json!({"using":["urn:ietf:params:jmap:core"],
                    "methodCalls":[["Core/echo",{"budget":true},"echo"]]}))
                .unwrap(),
            ))
            .unwrap();
        request.extensions_mut().insert(axum::extract::ConnectInfo(
            "127.0.0.1:48219".parse::<std::net::SocketAddr>().unwrap(),
        ));
        request
    }

    async fn send(
        &self,
        request: HttpRequest<Body>,
    ) -> (StatusCode, Value, mw_store::pool_telemetry::QuerySnapshot) {
        let (response, sql) =
            mw_store::pool_telemetry::measure_queries(self.router.clone().oneshot(request)).await;
        let response = response.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null), sql)
    }

    /// Warm every revision-validated memo, then measure one request.
    async fn warm(&self, request: impl Fn() -> HttpRequest<Body>) -> mw_store::pool_telemetry::QuerySnapshot {
        for _ in 0..3 {
            let (status, body, _) = self.send(request()).await;
            assert_eq!(status, StatusCode::OK, "{}: {body}", self.label);
        }
        let (status, body, sql) = self.send(request()).await;
        assert_eq!(status, StatusCode::OK, "{}: {body}", self.label);
        sql
    }
}

/// Statements per warm request: (reads, writes, other).
struct Budget {
    route: &'static str,
    reads: u64,
    writes: u64,
    other: u64,
}

fn check(label: &str, budget: &Budget, sql: &mw_store::pool_telemetry::QuerySnapshot) {
    eprintln!(
        "{}",
        json!({"kind":"t50hp-guard-statement-budget","backend":label,"route":budget.route,
            "reads":sql.read.calls,"writes":sql.write.calls,"other":sql.other.calls,
            "acquires":sql.acquire.calls,
            "budget":{"reads":budget.reads,"writes":budget.writes,"other":budget.other}})
    );
    assert!(
        sql.read.calls <= budget.reads && sql.write.calls <= budget.writes && sql.other.calls <= budget.other,
        "{label} {} exceeded its statement budget: {sql:?} > {}/{}/{}",
        budget.route, budget.reads, budget.writes, budget.other
    );
}

async fn run_budgets(fixture: &Fixture, me: &Budget, echo: &Budget) {
    let sql = fixture.warm(|| fixture.me()).await;
    check(fixture.label, me, &sql);
    let sql = fixture.warm(|| fixture.echo()).await;
    check(fixture.label, echo, &sql);
}

// Full production-router debug futures exceed ordinary libtest stack budgets.
fn run_fixture<F, Fut>(factory: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()>,
{
    let test = std::thread::Builder::new()
        .name("mw-guard-budget-test".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("test runtime")
                .block_on(factory());
        })
        .expect("bounded fixture thread");
    if let Err(panic) = test.join() {
        std::panic::resume_unwind(panic);
    }
}

#[test]
fn sqlite_guard_chain_statement_budget() {
    run_fixture(|| async {
        let fixture = Fixture::new(sqlite_db(), "sqlite-off").await;
        run_budgets(
            &fixture,
            &Budget { route: "GET /api/me", reads: 17, writes: 0, other: 0 },
            &Budget { route: "POST /jmap/api Core/echo", reads: 19, writes: 0, other: 0 },
        )
        .await;
    });
}

#[test]
fn postgres_guard_chain_statement_budget() {
    let Some(url) = std::env::var("MW_TEST_PG").ok().filter(|url| !url.is_empty()) else {
        eprintln!("[guard_read_budget] MW_TEST_PG unset: Postgres leg not run");
        return;
    };
    run_fixture(move || async move {
        let fixture = Fixture::new(url, "postgres").await;
        run_budgets(
            &fixture,
            &Budget { route: "GET /api/me", reads: 17, writes: 0, other: 0 },
            &Budget { route: "POST /jmap/api Core/echo", reads: 19, writes: 0, other: 0 },
        )
        .await;
    });
}

// ---------------------------------------------------------------------------
// Precondition proofs (t50hp B1, tier R). A guard may resolve a key at the
// revision its request's middleware share only if every write of that key
// bumps `settings_revision` in the same transaction, so the very next request
// (in this process or through another Store handle on the same database) sees
// the change. Keys in the key-value `settings` table do not bump it; their
// guards keep their own direct read, and these tests pin that too.
// ---------------------------------------------------------------------------

async fn put_policy(store: &Store, key: &str, value: Value) {
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "guard-budget-fixture".into(),
            updated_at: chrono::Utc::now().timestamp(),
        })
        .await
        .unwrap();
}

async fn remove_policy(store: &Store, key: &str) {
    store.delete_setting_policy(key).await.unwrap();
}

const PUBLIC_PEER: &str = "203.0.113.9:40100";

/// Geography keys (setting_policy): a write through the second handle is seen
/// by the very next request; the guard resolves at the shared request revision.
async fn geography_write_is_seen_by_the_next_request(fixture: &Fixture) {
    for _ in 0..3 {
        let (status, body, _) = fixture.send(fixture.me_from(PUBLIC_PEER)).await;
        assert_eq!(status, StatusCode::OK, "{}: {body}", fixture.label);
    }
    let before = fixture.writer.settings_revision().await.unwrap();
    put_policy(&fixture.writer, "security.signin.geoScope", json!("all")).await;
    put_policy(&fixture.writer, "security.signin.geoEnforceSessions", json!(true)).await;
    let after = fixture.writer.settings_revision().await.unwrap();
    assert!(after > before, "{}: setting_policy write must bump the revision", fixture.label);
    // No GeoIP database in tests: an enforced policy refuses (fail closed).
    let (status, body, _) = fixture.send(fixture.me_from(PUBLIC_PEER)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{}: {body}", fixture.label);
    assert_eq!(body["code"], "sessionGeographyUnavailable", "{}", fixture.label);
    remove_policy(&fixture.writer, "security.signin.geoEnforceSessions").await;
    remove_policy(&fixture.writer, "security.signin.geoScope").await;
    let (status, body, _) = fixture.send(fixture.me_from(PUBLIC_PEER)).await;
    assert_eq!(status, StatusCode::OK, "{}: {body}", fixture.label);
}

/// The four key-value guard inputs do not bump `settings_revision`, so they
/// are NOT memoised by it: each request still reads them, and a write through
/// the second handle is seen by the very next request.
async fn key_value_guard_inputs_do_not_bump_the_revision(fixture: &Fixture) {
    let (status, body, _) = fixture.send(fixture.me()).await;
    assert_eq!(status, StatusCode::OK, "{}: {body}", fixture.label);
    let before = fixture.writer.settings_revision().await.unwrap();
    // Rewrites of the current (or empty) values: no behaviour change.
    for (key, empty) in [
        ("security.inbound-allowlist.v1", None),
        ("security.clientAccess.v1", None),
        ("v6:admin:bans", Some("[]")),
    ] {
        let current = fixture.writer.get_setting(key).await.unwrap();
        let Some(value) = current.as_deref().or(empty) else { continue };
        fixture.writer.set_setting(key, value).await.unwrap();
        assert_eq!(
            fixture.writer.settings_revision().await.unwrap(),
            before,
            "{}: a {key} write now bumps settings_revision; it could then be memoised",
            fixture.label
        );
    }
    fixture
        .writer
        .set_setting("maintenance.mode.v1", r#"{"enabled":true,"message":"","retryAfterSecs":60}"#)
        .await
        .unwrap();
    assert_eq!(
        fixture.writer.settings_revision().await.unwrap(),
        before,
        "{}: a maintenance.mode.v1 write now bumps settings_revision",
        fixture.label
    );
    let (status, body, _) = fixture.send(fixture.me()).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{}: {body}", fixture.label);
    assert_eq!(body["code"], "maintenance", "{}", fixture.label);
    fixture
        .writer
        .set_setting("maintenance.mode.v1", r#"{"enabled":false,"message":"","retryAfterSecs":60}"#)
        .await
        .unwrap();
    let (status, body, _) = fixture.send(fixture.me()).await;
    assert_eq!(status, StatusCode::OK, "{}: {body}", fixture.label);
}

/// Trust-tier keys (setting_policy), resolved by `admin_probe::guard` at the
/// request's shared revision: a write through another handle is seen by the
/// next request; inside one request the shared proof is reused.
async fn trust_policy_follows_the_request_revision(reader: &Store, writer: &Store, label: &str) {
    let fresh = || HttpRequest::new(Body::empty());
    let mut first = fresh();
    let initial = crate::trust_tier::policy_for_request(reader, &mut first).await;
    let target = if initial.known_percent == 300 { 400 } else { 300 };
    let before = writer.settings_revision().await.unwrap();
    put_policy(writer, "security.trust.knownPercent", json!(target)).await;
    assert!(
        writer.settings_revision().await.unwrap() > before,
        "{label}: a trust write must bump the revision"
    );
    // The same request keeps the proof its middleware already share.
    let same = crate::trust_tier::policy_for_request(reader, &mut first).await;
    assert_eq!(same.known_percent, initial.known_percent, "{label}");
    // The next request takes a new proof and sees the write.
    let next = crate::trust_tier::policy_for_request(reader, &mut fresh()).await;
    assert_eq!(next.known_percent, target, "{label}");
    remove_policy(writer, "security.trust.knownPercent").await;
    let restored = crate::trust_tier::policy_for_request(reader, &mut fresh()).await;
    assert_eq!(restored.known_percent, initial.known_percent, "{label}");
}

fn sqlite_db() -> String {
    let directory =
        std::env::temp_dir().join(format!("mw-guard-budget-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    directory.join("mail.db").to_string_lossy().into_owned()
}

#[test]
fn sqlite_revision_preconditions_for_shared_guard_reads() {
    run_fixture(|| async {
        let db = sqlite_db();
        let fixture = Fixture::new(db.clone(), "sqlite-off").await;
        geography_write_is_seen_by_the_next_request(&fixture).await;
        key_value_guard_inputs_do_not_bump_the_revision(&fixture).await;
        let reader = Store::open(&db, ServerKey::generate()).await.unwrap();
        trust_policy_follows_the_request_revision(&reader, &fixture.writer, "sqlite").await;
    });
}

#[test]
fn postgres_revision_preconditions_for_shared_guard_reads() {
    let Some(url) = std::env::var("MW_TEST_PG").ok().filter(|url| !url.is_empty()) else {
        eprintln!("[guard_read_budget] MW_TEST_PG unset: Postgres leg not run");
        return;
    };
    run_fixture(move || async move {
        let fixture = Fixture::new(url.clone(), "postgres").await;
        geography_write_is_seen_by_the_next_request(&fixture).await;
        key_value_guard_inputs_do_not_bump_the_revision(&fixture).await;
        let reader = Store::open(&url, ServerKey::generate()).await.unwrap();
        trust_policy_follows_the_request_revision(&reader, &fixture.writer, "postgres").await;
    });
}

// ---------------------------------------------------------------------------
// t50hp-e8 acceptance 7: the enforced protection profile on the production
// router, synchronous accounting against write-behind accounting with the
// ownership rule. The warm request of the write-behind run carries no
// protection statement: no write, no BEGIN, and exactly the synchronous run's
// reads minus its protection reads. The enforced profile's other statements
// are unchanged.
// ---------------------------------------------------------------------------

const ENFORCED: [(&str, &str); 4] = [
    ("security.protection.rateMode", "enforce"),
    ("security.protection.adaptiveMode", "enforce"),
    ("security.protection.probePathMode", "enforce"),
    ("security.protection.userAgentMode", "enforce"),
];

type Warm = (mw_store::pool_telemetry::QuerySnapshot, mw_store::pool_telemetry::QuerySnapshot);

fn measure_enforced(
    accounting: Option<crate::request_protection::write_behind::AccountingConfig>,
    enforced: bool,
) -> Warm {
    let (sender, receiver) = std::sync::mpsc::channel();
    run_fixture(move || async move {
        crate::request_protection::write_behind::TEST_CONFIG.with(|cell| cell.set(accounting));
        let fixture = Fixture::new(sqlite_db(), "sqlite-enforced").await;
        if enforced {
            for (key, value) in ENFORCED {
                put_policy(&fixture.writer, key, json!(value)).await;
            }
        }
        // The accounting's first ownership check runs on its flusher task.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let me = fixture.warm(|| fixture.me()).await;
        let echo = fixture.warm(|| fixture.echo()).await;
        sender.send((me, echo)).unwrap();
    });
    receiver.recv().unwrap()
}

#[test]
fn sqlite_write_behind_warm_request_has_no_protection_statements() {
    use crate::request_protection::write_behind::{AccountingConfig, AccountingMode};
    let write_behind = AccountingConfig {
        mode: AccountingMode::WriteBehind,
        flush_every: std::time::Duration::from_millis(250),
        max_pending: 4096,
    };
    let off = measure_enforced(None, false);
    let synchronous = measure_enforced(None, true);
    let deferred = measure_enforced(Some(write_behind), true);
    for (route, off, synchronous, deferred) in [
        ("GET /api/me", off.0, synchronous.0, deferred.0),
        ("POST /jmap/api Core/echo", off.1, synchronous.1, deferred.1),
    ] {
        eprintln!(
            "{}",
            json!({"kind":"t50hp-e8-protection-statements","route":route,
                "off":{"reads":off.read.calls,"writes":off.write.calls,"other":off.other.calls},
                "enforcedSynchronous":{"reads":synchronous.read.calls,"writes":synchronous.write.calls,"other":synchronous.other.calls},
                "enforcedWriteBehind":{"reads":deferred.read.calls,"writes":deferred.write.calls,"other":deferred.other.calls}})
        );
        assert_eq!((deferred.write.calls, deferred.other.calls), (0, 0), "{route}: {deferred:?}");
        assert!(synchronous.write.calls >= 4 && synchronous.other.calls >= 2, "{route}: {synchronous:?}");
        // Protection reads of the synchronous run: one probe-decision read and one
        // state read inside each of the two accounting transactions.
        assert_eq!(synchronous.read.calls, deferred.read.calls + 3, "{route}: {synchronous:?} / {deferred:?}");
        // Without protection statements the enforced profile reads no more than
        // the default profile (measured: one read fewer, outside protection).
        assert!(deferred.read.calls <= off.read.calls, "{route}: {deferred:?} / {off:?}");
    }
}
