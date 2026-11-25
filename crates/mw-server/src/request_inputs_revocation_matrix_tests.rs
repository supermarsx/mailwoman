//! Permanent regression (t50hp-e15a review, rebuilt by t50hp-e22): revocation
//! on the newly eligible routes with the touch buffer on, in the same AppState
//! and in another one on the same database, each request bounded in wall-clock
//! time and statement count.
//!
//! Routes: /api/bootstrap, /api/settings, /api/account/preferences,
//! /api/policy, /api/features. Revocations: sign-out, session revoke, account
//! disable, password change. Per case: the session is touch-due and the request
//! before the revocation runs on published inputs; the revocation is made in
//! B (same) or A (other, then the mirror bound); the next request in B is
//! refused (password change: held, `me` reports it) within the bounds; the
//! buffered touch never recreates a deleted session.
//!
//! Both AppStates register a backend for the account, as after a sign-in. The
//! t49pg fixture does not: its account points at 127.0.0.1:9, which on some
//! Windows hosts is the "Simple TCP/IP Services" discard port, so start-up
//! registration holds the account's connect lock for up to 600 s; that is what
//! made the review's bootstrap run take more than 15 minutes.
use super::*;
use crate::request_inputs::snapshot::{
    Mode,
    pg_tests::{pg_dsn, pg_schema, sqlite_db},
    test_control as snap,
};
use crate::request_inputs::test_control as inputs_control;
use std::net::SocketAddr;

const BOUND: Duration = Duration::from_millis(100);
const SHORT: Mode = Mode::Bounded(BOUND);
const DOMAIN: &str = "e22-matrix.test";
const PEER: &str = "127.0.0.1:41022";
/// Wall-clock bound for one request after a revocation (debug build, loaded
/// host; measured well below it, see the eprintln lines).
const TIME_BOUND: Duration = Duration::from_secs(10);

struct Node {
    router: Router,
    state: crate::AppState,
}
struct Matrix {
    url: String,
    writer: Store,
    session: String,
    account: String,
    a: Node,
    b: Node,
}
struct Sent {
    status: StatusCode,
    body: Value,
    inputs: inputs_control::Counts,
    snap: snap::Counts,
    state: Option<crate::AppState>,
}

fn request(session: &str, method: &str, path: &str) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "localhost")
        .header("origin", "http://localhost")
        .header(
            "user-agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/125.0.0.0 Safari/537.36",
        )
        .header("cookie", format!("mw_session={session}"));
    if method != "GET" {
        // The double-submit CSRF pair (any value equal in both places is the
        // session's token only through the store; logout checks it).
        builder = builder.header("x-csrf-token", "unused");
    }
    let mut request = builder.body(Body::empty()).unwrap();
    request.extensions_mut().insert(axum::extract::ConnectInfo(
        PEER.parse::<SocketAddr>().unwrap(),
    ));
    request
}

async fn send(router: &Router, request: Request<Body>) -> Sent {
    let measured = crate::session_watch::test_control::measure(snap::measure(snap::with_mode(
        SHORT,
        inputs_control::measure(true, router.clone().oneshot(request)),
    )));
    let ((((result, inputs), snap), _watch, state), _frames) =
        inputs_control::process_wide(measured).await;
    let response = result.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 8 * 1024 * 1024)
        .await
        .unwrap();
    Sent {
        status,
        body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        inputs,
        snap,
        state,
    }
}

impl Matrix {
    async fn new(url: String) -> Self {
        let key = ServerKey::generate();
        let writer = Store::open(&url, key.clone()).await.unwrap();
        let user = format!(
            "m{}@{DOMAIN}",
            &uuid::Uuid::new_v4().simple().to_string()[..12]
        );
        let credentials = Credentials {
            username: user.clone(),
            password: "fixture-secret-never-log".into(),
        };
        writer.put_domain_config(&DomainConfigRow { name: DOMAIN.into(), upstream_json: json!({"imap":{"host":"127.0.0.1","port":9,"tls":"starttls","auth":"password","allowPrivate":true}}).to_string(), allowlist_json: "[]".into(), blocklist_json: "[]".into(), enabled: true, preferred: false, settings_json: "{}".into(), updated_at: 1 }).await.unwrap();
        let account = writer
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "127.0.0.1",
                    port: 9,
                    tls: "starttls",
                    username: &user,
                    sync_policy_json: "{}",
                },
                &credentials,
            )
            .await
            .unwrap();
        writer
            .remember_account_names(&account, &[user.as_str()])
            .await
            .unwrap();
        let session = writer
            .create_session(&account, &user, "engine", "engine", &credentials)
            .await
            .unwrap();
        writer
            .set_session_authentication(&session, "password")
            .await
            .unwrap();
        writer
            .bind_verified_primary(
                &session,
                mw_store::identity_principal::VerifiedPrimary {
                    account_id: &account,
                    kind: "mailbox",
                    authority: "fixture-native",
                    subject: &user,
                },
                chrono::Utc::now().timestamp(),
            )
            .await
            .unwrap()
            .unwrap();
        policy(&writer, "access.protocols", json!(["imap"])).await;
        let a = Self::node(&url, &key, &session, &account, &user).await;
        let b = Self::node(&url, &key, &session, &account, &user).await;
        assert_ne!(
            a.state.store.cache_scope(),
            b.state.store.cache_scope(),
            "two Stores"
        );
        Self {
            url,
            writer,
            session,
            account,
            a,
            b,
        }
    }

    async fn node(url: &str, key: &ServerKey, session: &str, account: &str, user: &str) -> Node {
        let registered = account.to_owned();
        let user = user.to_owned();
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
            crate::V6Config {
                admin_enabled: false,
                ..Default::default()
            },
            crate::sso::SsoProviderSource::Store,
            Some(Box::new(move |store| {
                let engine = Arc::new(Engine::new(store.clone()));
                engine.register_backend(
                    registered,
                    AccountRuntime::new(Arc::new(ReadOnlyBackend), Arc::new(NoSend), &user),
                );
                engine
            })),
            crate::core_test_build_info::info().unwrap(),
        )
        .await
        .unwrap();
        let sent = send(&router, request(session, "GET", "/api/me")).await;
        assert_eq!(sent.status, StatusCode::OK, "{}", sent.body);
        Node {
            router,
            state: sent.state.expect("the watch ran"),
        }
    }

    /// Requests until one ran on this session's published inputs.
    async fn ready(&self, node: &Node) {
        for _ in 0..400 {
            let sent = send(&node.router, request(&self.session, "GET", "/api/me")).await;
            assert_eq!(sent.status, StatusCode::OK, "{}", sent.body);
            if sent.inputs.frames_ready == 1 || sent.snap.reused > 0 {
                return;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("request inputs never published");
    }

    /// Touch due: no recent touch held in either process, the row 60 s old,
    /// no cached fence newer than the row.
    async fn touch_due(&self) {
        for node in [&self.a, &self.b] {
            node.state
                .store
                .disable_session_touch_buffer()
                .await
                .unwrap();
            assert!(crate::session_persistence::enable_touch_buffer(
                &node.state.store
            ));
            let environment = crate::request_inputs::Environment::capture().unwrap();
            if let Some(entry) = node
                .state
                .native_publication
                .as_ref()
                .unwrap()
                .request_inputs(&self.session, &environment)
            {
                entry.guard_fence.clear();
            }
        }
        let at = (chrono::Utc::now() - chrono::Duration::seconds(60)).to_rfc3339();
        let sql = format!(
            "UPDATE sessions SET last_seen='{at}' WHERE id='{}'",
            self.session
        );
        let changed = if self.url.starts_with("postgres") {
            use sqlx::Connection;
            let mut conn = sqlx::PgConnection::connect(&self.url).await.unwrap();
            let changed = sqlx::query(&sql)
                .execute(&mut conn)
                .await
                .unwrap()
                .rows_affected();
            conn.close().await.unwrap();
            changed
        } else {
            let path = std::path::PathBuf::from(&self.url);
            let key = mw_store::encryption::global()
                .unwrap()
                .key_for_open(&path)
                .unwrap()
                .cloned();
            let options = mw_store::encryption::connect_options(&path, key.as_ref(), false);
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .unwrap();
            let changed = sqlx::query(&sql)
                .execute(&pool)
                .await
                .unwrap()
                .rows_affected();
            pool.close().await;
            changed
        };
        assert_eq!(changed, 1);
    }

    fn logout(&self) -> Request<Body> {
        let csrf = self.writer.session_csrf_token(&self.session);
        let mut request = request(&self.session, "POST", "/api/logout");
        request.headers_mut().insert(
            "cookie",
            format!("mw_session={}; mw_csrf={csrf}", self.session)
                .parse()
                .unwrap(),
        );
        request
            .headers_mut()
            .insert("x-csrf-token", csrf.parse().unwrap());
        request
    }
}

/// Statements of every Store in the process while `future` runs (bootstrap
/// sections run in tasks of their own, which a task-local counter misses).
async fn counted<F: std::future::Future>(future: F) -> (F::Output, u64, Duration) {
    let start = mw_store::pool_telemetry::process_snapshot();
    let started = std::time::Instant::now();
    let output = tokio::time::timeout(TIME_BOUND * 3, future)
        .await
        .expect("request did not answer");
    let elapsed = started.elapsed();
    let end = mw_store::pool_telemetry::process_snapshot();
    let statements = (end.read.calls - start.read.calls)
        + (end.write.calls - start.write.calls)
        + (end.other.calls - start.other.calls);
    (output, statements, elapsed)
}

/// Statement bound per route for one request after a revocation (process
/// wide, includes the other AppState's background polls during the request).
fn statement_bound(path: &str) -> u64 {
    if path == "/api/bootstrap" { 400 } else { 60 }
}

fn refused(sent: &Sent, path: &str, case: &str, same: bool) {
    let label = format!("{path} {case} same={same}");
    // /api/me answers a held account (it is the one route that must).
    if path == "/api/me" && case == "password-change" {
        assert_eq!(sent.status, StatusCode::OK, "{label}: {}", sent.body);
        assert_eq!(sent.body["passwordChangeRequired"], true, "{label}: {}", sent.body);
        return;
    }
    if path != "/api/bootstrap" {
        assert!(
            sent.status == StatusCode::UNAUTHORIZED || sent.status == StatusCode::FORBIDDEN,
            "{label}: answered {} {}",
            sent.status,
            sent.body
        );
        return;
    }
    assert_eq!(sent.status, StatusCode::OK, "{label}: the document itself");
    for (name, section) in sent.body["sections"].as_object().unwrap() {
        if matches!(
            name.as_str(),
            "publicConfig"
                | "cookiesConfig"
                | "version"
                | "uiPlugins"
                | "assistConfig"
                | "restriction"
        ) {
            continue;
        }
        let status = section["status"].as_u64().unwrap();
        if case == "password-change" && name == "me" {
            assert_eq!(status, 200, "{label}: me");
            assert_eq!(
                section["body"]["passwordChangeRequired"], true,
                "{label}: {section}"
            );
            continue;
        }
        assert!(
            status == 401 || status == 403 || (name == "appearanceCustomization" && status == 409),
            "{label}: section {name} answered {status}: {section}"
        );
        assert!(
            section["body"].get("accountId").is_none(),
            "{label}: {name} carried data"
        );
    }
}

async fn case(url: String, route: &'static str, case: &'static str, same: bool) {
    let m = Matrix::new(url).await;
    // t50hp-e9: the customization read carries the account check.
    let uri = if route == "customization" {
        format!("/api/account/appearance/customization?accountId={}", m.account)
    } else {
        route.to_owned()
    };
    let path = uri.as_str();
    m.ready(&m.a).await;
    m.ready(&m.b).await;
    let mirror =
        m.b.state
            .native_publication
            .as_ref()
            .unwrap()
            .existing_guard_mirror()
            .expect("bounded mirror");
    // Touch-due request before the revocation: published inputs.
    m.touch_due().await;
    let before = send(&m.b.router, request(&m.session, "GET", path)).await;
    assert_eq!(
        before.status,
        StatusCode::OK,
        "{path} {case}: before {}",
        before.body
    );
    if path != "/api/bootstrap" {
        assert!(
            before.inputs.frames_ready == 1 || before.snap.reused > 0,
            "{path} {case}: before used published inputs: {:?} {:?}",
            before.inputs,
            before.snap
        );
        assert_eq!(before.inputs.legacy_session_reads, 0, "{path} {case}");
    } else {
        assert_eq!(
            before.body["sections"]["me"]["status"], 200,
            "{}",
            before.body
        );
    }
    // The next request is touch-due again.
    m.touch_due().await;
    let actor = if same { &m.b } else { &m.a };
    let expedites = mirror.counters().expedites;
    match case {
        "sign-out" => {
            let out = send(&actor.router, m.logout()).await;
            assert!(
                out.status.is_success(),
                "{case}: {} {}",
                out.status,
                out.body
            );
        }
        "session-revoke" => {
            actor.state.store.delete_session(&m.session).await.unwrap();
            crate::request_inputs::snapshot::expedite(&actor.state);
        }
        // The console's write request expedites through the request layer.
        "account-disable" => {
            crate::account_gate::update_flags(&actor.state.store, &m.account, |flags| {
                flags.disabled = true
            })
            .await
            .unwrap();
            crate::request_inputs::snapshot::expedite(&actor.state);
        }
        "password-change" => {
            crate::account_gate::update_flags(&actor.state.store, &m.account, |flags| {
                flags.force_password_change = true
            })
            .await
            .unwrap();
            crate::request_inputs::snapshot::expedite(&actor.state);
        }
        _ => unreachable!(),
    }
    if !same {
        tokio::time::sleep(BOUND).await;
        assert_eq!(
            mirror.counters().expedites,
            expedites,
            "{case}: B was never expedited"
        );
    }
    let (after, statements, elapsed) =
        counted(send(&m.b.router, request(&m.session, "GET", path))).await;
    eprintln!(
        "{}",
        json!({"kind":"t50hp-e22-revocation-matrix","backend":if m.url.starts_with("postgres") {"postgres"} else {"sqlite"},
        "route":path,"revocation":case,"same":same,"status":after.status.as_u16(),
        "ms":elapsed.as_millis() as u64,"statements":statements})
    );
    assert!(
        elapsed < TIME_BOUND,
        "{path} {case} same={same}: {elapsed:?}"
    );
    assert!(
        statements <= statement_bound(path),
        "{path} {case} same={same}: {statements} statements"
    );
    assert_eq!(
        after.snap.reused, 0,
        "{path} {case} same={same}: no reuse after the revocation"
    );
    refused(&after, path, case, same);
    // A buffered touch never recreates a deleted session.
    m.b.state.store.flush_session_touches().await.unwrap();
    if case == "sign-out" || case == "session-revoke" {
        assert!(
            m.writer
                .get_primary_session_with_activity(&m.session)
                .await
                .is_err(),
            "{path} {case}: recreated"
        );
    }
    // 40 cases in one process: release every pool now (PostgreSQL counts
    // connections against max_connections until they close).
    for store in [&m.a.state.store, &m.b.state.store, &m.writer] {
        store.close().await;
    }
}

const ROUTES: [&str; 7] = [
    "/api/me",
    "customization",
    "/api/bootstrap",
    "/api/settings",
    "/api/account/preferences",
    "/api/policy",
    "/api/features",
];
const CASES: [&str; 4] = [
    "sign-out",
    "session-revoke",
    "account-disable",
    "password-change",
];

/// One runtime per case: dropping it ends every task that holds a Store.
fn matrix(
    url: impl Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = String> + Send>>
    + Send
    + Sync
    + Copy
    + 'static,
) {
    for path in ROUTES {
        for kind in CASES {
            for same in [true, false] {
                run_fixture(move || async move { case(url().await, path, kind, same).await });
            }
        }
    }
}

#[test]
fn sqlite_revocation_matrix_on_newly_eligible_routes() {
    matrix(|| Box::pin(async { sqlite_db() }));
}

#[test]
fn postgres_revocation_matrix_on_newly_eligible_routes() {
    let Some(_) = pg_dsn() else { return };
    matrix(|| Box::pin(async { pg_schema(&pg_dsn().unwrap()).await }));
}
