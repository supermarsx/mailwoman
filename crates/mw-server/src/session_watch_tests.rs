//! t34-e13: session watch under the bounded guard snapshot, over the real
//! router (SQLite, engine mode, published request inputs). Statement counts are
//! taken around the session-watch middleware only (`test_control::observe`).
use super::*;
use crate::request_inputs::snapshot::{Mode, test_control as snap};
use crate::request_inputs::test_control as inputs;
use axum::body::{Body, to_bytes};
use mw_store::{AccountKind, Credentials, DomainConfigRow, NewAccount, ServerKey, Store};
use serde_json::Value;
use std::{future::Future, net::SocketAddr, sync::Arc, time::Duration};
use tower::ServiceExt;

const OFF: Mode = Mode::Off;
const SHORT: Mode = Mode::Bounded(Duration::from_millis(100));
const LONG: Mode = Mode::Bounded(Duration::from_millis(5000));
const CHROME: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36";
const FIREFOX: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:126.0) Gecko/20100101 Firefox/126.0";
/// First network the session is seen from.
const HOME: &str = "127.0.0.1:41000";
/// A public address: no GeoIP database in tests, so the country is unknown and
/// the `security.geo.sessionLookupFailure` switch is read.
const PUBLIC: &str = "8.8.8.8:41000";
const USER: &str = "person@watch-snapshot.test";

struct Fixture {
    router: axum::Router,
    writer: Store,
    session: String,
    account: String,
    db: std::path::PathBuf,
    db_key: Option<mw_store::encryption::DbKey>,
}
struct Sent {
    status: StatusCode,
    body: Value,
    watch: test_control::Counts,
    inputs: inputs::Counts,
    state: Option<AppState>,
}

fn run<F, Fut>(factory: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()>,
{
    // Production-router futures exceed the default test stack.
    let test = std::thread::Builder::new()
        .name("mw-session-watch-snapshot-test".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(factory());
        })
        .unwrap();
    if let Err(panic) = test.join() {
        std::panic::resume_unwind(panic);
    }
}

async fn policy(store: &Store, key: &str, value: Value) {
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "session-watch-snapshot-fixture".into(),
            updated_at: chrono::Utc::now().timestamp(),
        })
        .await
        .unwrap();
}

impl Fixture {
    async fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("mw-watch-snapshot-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let db = dir.join("mail.db");
        let key = ServerKey::generate();
        let db_key = mw_store::encryption::global().unwrap().key_for_open(&db).unwrap().cloned();
        let writer = Store::open(db.to_str().unwrap(), key.clone()).await.unwrap();
        let credentials = Credentials { username: USER.into(), password: "fixture-secret-never-log".into() };
        writer.put_domain_config(&DomainConfigRow{name:"watch-snapshot.test".into(),upstream_json:serde_json::json!({"imap":{"host":"127.0.0.1","port":9,"tls":"starttls","auth":"password","allowPrivate":true}}).to_string(),allowlist_json:"[]".into(),blocklist_json:"[]".into(),enabled:true,preferred:false,settings_json:"{}".into(),updated_at:1}).await.unwrap();
        let account = writer
            .create_account(
                &NewAccount { kind: AccountKind::Imap, host: "127.0.0.1", port: 9, tls: "starttls", username: USER, sync_policy_json: "{}" },
                &credentials,
            )
            .await
            .unwrap();
        writer.remember_account_names(&account, &[USER]).await.unwrap();
        let session = writer.create_session(&account, USER, "engine", "engine", &credentials).await.unwrap();
        writer.set_session_authentication(&session, "password").await.unwrap();
        writer
            .bind_verified_primary(
                &session,
                mw_store::identity_principal::VerifiedPrimary { account_id: &account, kind: "mailbox", authority: "fixture-native", subject: USER },
                chrono::Utc::now().timestamp(),
            )
            .await
            .unwrap()
            .unwrap();
        writer.touch_session(&session).await.unwrap();
        policy(&writer, "access.protocols", serde_json::json!(["imap"])).await;
        let (router, _) = crate::build_app_inner(
            crate::AppConfig {
                db_path: db.to_string_lossy().into_owned(),
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
        let fixture = Self { router, writer, session, account, db, db_key };
        // The sign-in address the alert preferences resolve for.
        fixture
            .raw("INSERT INTO signin_identity(account_id,address,seen_at,event_id) VALUES(?1,?2,1,'e1')", &[&fixture.account, USER])
            .await;
        fixture
    }
    async fn raw(&self, sql: &str, binds: &[&str]) -> u64 {
        let options = mw_store::encryption::connect_options(&self.db, self.db_key.as_ref(), false);
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(options.busy_timeout(Duration::from_secs(10))).await.unwrap();
        let mut query = sqlx::query(sql);
        for bind in binds {
            query = query.bind(*bind);
        }
        let changed = query.execute(&pool).await.unwrap().rows_affected();
        pool.close().await;
        changed
    }
    async fn scalar(&self, sql: &str) -> i64 {
        let options = mw_store::encryption::connect_options(&self.db, self.db_key.as_ref(), false);
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(options.busy_timeout(Duration::from_secs(10))).await.unwrap();
        let value: i64 = sqlx::query_scalar(sql).fetch_one(&pool).await.unwrap();
        pool.close().await;
        value
    }
    /// Runs one write and returns how much it moved the settings revision, read
    /// before and after it inside the same transaction (other writers in the
    /// fixture cannot interleave).
    async fn bumps(&self, sql: &str, binds: &[&str]) -> i64 {
        let options = mw_store::encryption::connect_options(&self.db, self.db_key.as_ref(), false);
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(options.busy_timeout(Duration::from_secs(10))).await.unwrap();
        // IMMEDIATE: take the write lock first (a deferred read-then-write can
        // fail with SQLITE_BUSY without waiting).
        let mut conn = pool.acquire().await.unwrap();
        sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await.unwrap();
        let revision = "SELECT n FROM settings_revision WHERE id=1";
        let before: i64 = sqlx::query_scalar(revision).fetch_one(&mut *conn).await.unwrap();
        let mut query = sqlx::query(sql);
        for bind in binds {
            query = query.bind(*bind);
        }
        assert_eq!(query.execute(&mut *conn).await.unwrap().rows_affected(), 1, "{sql}");
        let after: i64 = sqlx::query_scalar(revision).fetch_one(&mut *conn).await.unwrap();
        sqlx::query("COMMIT").execute(&mut *conn).await.unwrap();
        drop(conn);
        pool.close().await;
        after - before
    }
    fn me(&self, peer: &str, agent: &str) -> Request {
        let mut request = axum::http::Request::builder()
            .method("GET")
            .uri("/api/me")
            .header("host", "localhost")
            .header("origin", "http://localhost")
            .header("user-agent", agent)
            .header("cookie", format!("mw_session={}", self.session))
            .body(Body::empty())
            .unwrap();
        request.extensions_mut().insert(axum::extract::ConnectInfo(peer.parse::<SocketAddr>().unwrap()));
        request
    }
    async fn send(&self, request: Request, mode: Mode) -> Sent {
        let measured = test_control::measure(snap::with_mode(mode, inputs::measure(true, self.router.clone().oneshot(request))));
        let ((result, inputs), watch, state) = measured.await;
        let response = result.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        Sent { status, body: serde_json::from_slice(&bytes).unwrap_or(Value::Null), watch, inputs, state }
    }
    async fn get(&self, peer: &str, agent: &str, mode: Mode) -> Sent {
        self.send(self.me(peer, agent), mode).await
    }
    /// Off mode until the publication worker published this session's request
    /// inputs. The watch record is bound to CHROME from HOME.
    async fn ready(&self) -> AppState {
        for _ in 0..400 {
            let reply = self.get(HOME, CHROME, OFF).await;
            assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
            if reply.inputs.first_fences == 1 {
                return reply.state.expect("the watch ran");
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("request inputs never published");
    }
    async fn mirror(&self, state: &AppState, mode: Mode) -> Arc<mw_store::authority_publication::mirror::EpochMirror> {
        let reply = self.get(HOME, CHROME, mode).await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
        let mirror = state.native_publication.as_ref().unwrap().existing_guard_mirror().expect("bounded request starts the mirror");
        tokio::time::timeout(Duration::from_secs(10), async {
            while !mirror.is_fresh() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("mirror completes a poll");
        mirror
    }
    /// Bounded requests until the watch issued no statement other than the
    /// sign-in address read and (with `settings`) reused its settings reads.
    async fn reusing(&self, peer: &str, agent: &str, mode: Mode, settings: bool) -> Sent {
        for _ in 0..400 {
            let reply = self.get(peer, agent, mode).await;
            assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
            let reused = reply.watch.prefs_reused + reply.watch.geo_reused > 0;
            // A preference reuse still reads the sign-in address (1 PK read).
            if reply.watch.reads == reply.watch.prefs_reused && (reused || !settings) {
                return reply;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("bounded session watch never reached its warm statement count");
    }
}

/// Binding action `detect`: a FIREFOX request on a CHROME-bound session is a
/// mismatch on every request (recorded, never acted on), so the watch reads the
/// account's alert preferences on each one (the e0 harness case).
async fn detecting() -> Fixture {
    let f = Fixture::new().await;
    policy(&f.writer, "security.sessionBinding.webmailAction", serde_json::json!("detect")).await;
    policy(&f.writer, "security.alerts.countryChangeEnabled", serde_json::json!(true)).await;
    f
}

#[test]
fn bounded_warm_watch_reads_only_the_sign_in_address_and_off_mode_is_unchanged() {
    run(|| async {
        let f = detecting().await;
        let state = f.ready().await;
        // The first mismatch records its debounce (one write) and the second
        // fills the settings memos for this subject; measure the third.
        for _ in 0..2 {
            assert_eq!(f.get(HOME, FIREFOX, OFF).await.status, StatusCode::OK);
        }
        let off = f.get(HOME, FIREFOX, OFF).await;
        assert_eq!(off.status, StatusCode::OK, "{}", off.body);
        assert_eq!((off.watch.inspections, off.watch.prefs_fresh, off.watch.prefs_reused), (1, 1, 0));
        // Today's statements: user_prefs: signin_identity (1) + three
        // settings_revision reads. t50hp-e9: the Watch load point is a deferred
        // decision boundary; this mismatch only reads (debounced), so no fence
        // is read here (was 5 with the boundary fence).
        assert_eq!((off.watch.reads, off.watch.writes, off.watch.other), (4, 0, 0), "{:?}", off.watch);
        f.mirror(&state, SHORT).await;
        let hit = f.reusing(HOME, FIREFOX, SHORT, true).await;
        // The one statement left: the signin_identity primary-key read.
        assert_eq!((hit.watch.reads, hit.watch.writes, hit.watch.other), (1, 0, 0), "{:?}", hit.watch);
        assert_eq!(hit.watch.prefs_reused, 1);
        assert_eq!(hit.body, off.body, "same reply");
        // Off mode after bounded mode in the same process: today's path again.
        let again = f.get(HOME, FIREFOX, OFF).await;
        assert_eq!(again.watch, off.watch);
        // A country that cannot be told (public address, no GeoIP database):
        // the geo-failure switch is read per request, and reused when bounded.
        for _ in 0..2 {
            assert_eq!(f.get(PUBLIC, CHROME, OFF).await.status, StatusCode::OK);
        }
        let off_public = f.get(PUBLIC, CHROME, OFF).await;
        assert_eq!((off_public.watch.prefs_fresh, off_public.watch.geo_reused), (0, 0), "{:?}", off_public.watch);
        // t50hp-e9: the geo switch only; the Watch fence is deferred (no effect).
        assert_eq!((off_public.watch.reads, off_public.watch.writes), (1, 0), "geo switch: {:?}", off_public.watch);
        let hit_public = f.reusing(PUBLIC, CHROME, SHORT, true).await;
        assert_eq!((hit_public.watch.reads, hit_public.watch.writes, hit_public.watch.geo_reused), (0, 0, 1), "{:?}", hit_public.watch);
        eprintln!("{}", serde_json::json!({"kind":"t34-e13-session-watch-phase","offReads":off.watch.reads,
            "boundedReads":hit.watch.reads,"offPublicReads":off_public.watch.reads,"boundedPublicReads":hit_public.watch.reads}));
    });
}

#[test]
fn bounded_cached_preferences_follow_each_input_in_process_and_across_connections() {
    run(|| async {
        let f = detecting().await;
        let state = f.ready().await;
        let mirror = f.mirror(&state, SHORT).await;
        f.reusing(HOME, FIREFOX, SHORT, true).await;
        let sensitivity = || {
            let (state, account) = (state.clone(), f.account.clone());
            async move { snap::with_mode(SHORT, async { alert_prefs(&state, &account).await.sensitivity }).await }
        };

        // 1. Sign-ins touch nothing global (0119 retired the 0117 triggers): no
        //    trigger of any kind on signin_identity, so no settings revision and
        //    no authority epoch moves, whatever the address looks like. Other
        //    accounts' cached preferences and every published entry depend only
        //    on those, so they stay valid.
        let triggers = "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger' AND tbl_name='signin_identity'";
        assert_eq!(f.scalar(triggers).await, 0, "no trigger on signin_identity");
        let id = f.account.clone();
        for (sql, binds) in [
            ("UPDATE signin_identity SET seen_at=seen_at+1,event_id='e2' WHERE account_id=?1", vec![id.as_str()]),
            ("UPDATE signin_identity SET address=upper(address) WHERE account_id=?1", vec![id.as_str()]),
            ("UPDATE signin_identity SET address='alias@watch-snapshot.test' WHERE account_id=?1", vec![id.as_str()]),
            ("DELETE FROM signin_identity WHERE account_id=?1", vec![id.as_str()]),
            ("INSERT INTO signin_identity(account_id,address,seen_at,event_id) VALUES(?1,?2,9,'e9')", vec![id.as_str(), USER]),
        ] {
            assert_eq!(f.bumps(sql, &binds).await, 0, "{sql}");
        }
        // 2. The account's own address change takes effect on its very next
        //    request (fresh address read), cross connection, no poll, no wait.
        f.reusing(HOME, FIREFOX, SHORT, true).await;
        assert_eq!(f.bumps("UPDATE signin_identity SET address=upper(address) WHERE account_id=?1", &[&id]).await, 0);
        let moved = f.get(HOME, FIREFOX, SHORT).await;
        assert_eq!((moved.watch.prefs_fresh, moved.watch.prefs_reused), (1, 0), "{:?}", moved.watch);
        let _ = &mirror;

        // 3. Cross-connection policy change, no manual poll: within the bound.
        f.reusing(HOME, FIREFOX, SHORT, true).await;
        policy(&f.writer, "security.alerts.sensitivity", serde_json::json!("low")).await;
        tokio::time::sleep(Duration::from_millis(150)).await;
        let low = f.get(HOME, FIREFOX, SHORT).await;
        assert_eq!((low.watch.prefs_fresh, low.watch.prefs_reused), (1, 0), "{:?}", low.watch);
        assert_eq!(sensitivity().await, theft::Sensitivity::Low);

        // 4. In-process write: the request layer expedites after a write
        //    request; here the store write plus that same expedite call. The
        //    very next request reads fresh, with no poll and no wait.
        f.reusing(HOME, FIREFOX, SHORT, true).await;
        policy(&state.store, "security.alerts.sensitivity", serde_json::json!("high")).await;
        crate::request_inputs::snapshot::expedite(&state);
        let high = f.get(HOME, FIREFOX, SHORT).await;
        assert_eq!((high.watch.prefs_fresh, high.watch.prefs_reused), (1, 0), "{:?}", high.watch);
        assert_eq!(sensitivity().await, theft::Sensitivity::High);
    });
}

#[test]
fn binding_mismatch_steps_up_on_the_very_next_bounded_request() {
    run(|| async {
        let f = Fixture::new().await;
        let state = f.ready().await;
        let mirror = f.mirror(&state, LONG).await;
        f.reusing(HOME, CHROME, LONG, false).await;
        let expedites = mirror.counters().expedites;
        let theft = f.get(HOME, FIREFOX, LONG).await;
        assert_eq!(theft.status, StatusCode::FORBIDDEN, "{}", theft.body);
        assert_eq!(theft.body["code"], "step_up_required");
        assert_eq!(mirror.counters().expedites, expedites + 1, "the step-up transition expedites");
        // The flag holds for the original browser as well.
        let after = f.get(HOME, CHROME, LONG).await;
        assert_eq!(after.status, StatusCode::FORBIDDEN, "{}", after.body);
    });
}

#[test]
fn theft_watch_revocation_expedites_and_the_next_request_is_refused() {
    run(|| async {
        let f = Fixture::new().await;
        policy(&f.writer, "security.sessionBinding.webmailAction", serde_json::json!("revoke")).await;
        let state = f.ready().await;
        // A 5 s bound: only the expedite can refuse at once.
        let mirror = f.mirror(&state, LONG).await;
        f.reusing(HOME, CHROME, LONG, false).await;
        let expedites = mirror.counters().expedites;
        let revoked = f.get(HOME, FIREFOX, LONG).await;
        assert_eq!(revoked.status, StatusCode::UNAUTHORIZED, "{}", revoked.body);
        assert!(mirror.counters().expedites > expedites, "revocation expedites");
        let next = f.get(HOME, CHROME, LONG).await;
        assert_eq!(next.status, StatusCode::UNAUTHORIZED, "{}", next.body);
        assert_eq!(next.inputs.first_fences, 0, "no published inputs for a deleted session");
        // The deleted id is never bound afresh: the watch found no live session
        // behind it (without the expedite a reused fence handed it the old one).
        let hash = crate::push_relay::hash_token(&f.session);
        assert!(cache_lock().get(&hash).is_none(), "no in-memory watch record");
        assert_eq!(state.store.get_setting(&state_key(&hash)).await.unwrap(), None, "no stored watch record");
        let again = f.get(HOME, CHROME, LONG).await;
        assert_eq!(again.status, StatusCode::UNAUTHORIZED, "{}", again.body);
        assert_eq!(state.store.get_setting(&state_key(&hash)).await.unwrap(), None);
    });
}

/// Review N1: the settings-revision triggers are required definitions of
/// published inputs on SQLite, so with one dropped the watch never reuses
/// settings rows (they would otherwise go stale until restart).
#[test]
fn a_dropped_settings_revision_trigger_disables_preference_reuse() {
    run(|| async {
        let f = detecting().await;
        let state = f.ready().await;
        f.mirror(&state, SHORT).await;
        f.reusing(HOME, FIREFOX, SHORT, true).await;
        f.raw("DROP TRIGGER settings_revision_setting_policy_update", &[]).await;
        // Long enough for the worker to rebuild the entry after the schema change
        // (which, without the required definition, it would publish again).
        let mut reused = 0;
        for _ in 0..80 {
            let reply = f.get(HOME, FIREFOX, SHORT).await;
            reused += reply.watch.prefs_reused;
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        assert_eq!(reused, 0, "no preference reuse without the settings-revision trigger");
    });
}

#[test]
fn cached_preferences_fold_exactly_like_user_prefs() {
    run(|| async {
        let f = Fixture::new().await;
        let state = f.ready().await;
        let globals = sec_cfg::values(&state.store, "security.alerts.").await;
        for (key, value) in [
            ("", serde_json::Value::Null),
            ("security.alerts.sensitivity", serde_json::json!("low")),
            ("security.alerts.sensitivity", serde_json::json!("high")),
            ("security.alerts.channels", serde_json::json!("app")),
            ("security.alerts.channels", serde_json::json!("email")),
            ("security.alerts.countryChange", serde_json::json!(false)),
        ] {
            if !key.is_empty() {
                policy(&state.store, key, value.clone()).await;
            }
            let original = alerts::user_prefs(&state, &f.account).await;
            let (inputs, complete) = pref_inputs(&state, &f.account).await;
            assert!(complete);
            let folded = fold_prefs(&globals, &inputs);
            assert_eq!(
                (original.country_alert, original.channels, original.sensitivity, original.email_allowed),
                (folded.country_alert, folded.channels, folded.sensitivity, folded.email_allowed),
                "{key}={value}"
            );
        }
    });
}

/// `realtime_certify_with` with a recheck that only succeeds (the real one is
/// push.rs `recheck`; these tests need its ordering, not its reads).
async fn certified(state: &AppState, session: &mw_store::Session, bearer: bool, mode: Mode) -> Option<RealtimeCertificate> {
    let state = state.clone();
    let session = session.clone();
    snap::with_mode(mode, async move {
        let (verdict, cert) = realtime_certify_with(&state, &session, bearer, || async { Ok::<_, ()>(7) }).await.unwrap();
        assert_eq!(verdict, 7, "the recheck's own verdict is returned");
        cert
    })
    .await
}

#[test]
fn realtime_certificate_costs_no_statements_and_detects_revocation_within_the_bound() {
    run(|| async {
        for case in ["raw-session-delete", "account-disable", "local-revoke", "policy", "clock"] {
            let f = Fixture::new().await;
            if case == "clock" {
                policy(&f.writer, "session.idleSecs", serde_json::json!(3600)).await;
            }
            let state = f.ready().await;
            let mirror = f.mirror(&state, SHORT).await;
            let session = state.store.get_session(&f.session).await.unwrap();
            assert!(certified(&state, &session, false, OFF).await.is_none(), "mode off: no cheap path");
            assert!(certified(&state, &session, true, SHORT).await.is_none(), "bearer: never");
            // A failed recheck: its error is returned, no certificate, no fence read.
            let (failed, sql) = mw_store::pool_telemetry::measure_queries(snap::with_mode(SHORT,
                realtime_certify_with(&state, &session, false, || async { Err::<(), _>("revoked") }))).await;
            assert!(matches!(failed, Err("revoked")));
            assert_eq!(sql.read.calls, 0, "no fence after a failed recheck");
            let cert = certified(&state, &session, false, SHORT).await.expect("published engine session certifies");
            let period = Duration::from_secs(30);
            let (check, sql) = mw_store::pool_telemetry::measure_queries(snap::with_mode(SHORT, async {
                realtime_check(&state, &cert, period)
            }))
            .await;
            assert_eq!(check, RealtimeCheck::Current, "{case}");
            assert_eq!((sql.read.calls, sql.write.calls, sql.other.calls), (0, 0, 0), "{case}: no statement");
            match case {
                "raw-session-delete" => assert_eq!(f.raw("DELETE FROM sessions WHERE id=?1", &[&f.session]).await, 1),
                "account-disable" => {
                    crate::account_gate::update_flags(&f.writer, &f.account, |flags| flags.disabled = true).await.unwrap();
                }
                "local-revoke" => {
                    state.store.delete_session(&f.session).await.unwrap();
                    crate::request_inputs::snapshot::expedite(&state);
                    // At once: no poll, no wait.
                    let now = snap::with_mode(SHORT, async { realtime_check(&state, &cert, period) }).await;
                    assert_eq!(now, RealtimeCheck::Revalidate, "local revocation is seen by the next check");
                    continue;
                }
                "policy" => policy(&f.writer, "session.absoluteSecs", serde_json::json!(60)).await,
                _ => {
                    // No authority change at all: only the clock. activity_at
                    // 3500 s ago with a 3600 s idle limit: inside period + margin.
                    let ago = (chrono::Utc::now() - chrono::TimeDelta::seconds(3500)).to_rfc3339();
                    f.raw("UPDATE sessions SET activity_at=?1 WHERE id=?2", &[&ago, &f.session]).await;
                    let session = state.store.get_session(&f.session).await.unwrap();
                    let near = certified(&state, &session, false, SHORT).await.expect("certifies");
                    let check = snap::with_mode(SHORT, async { realtime_check(&state, &near, period) }).await;
                    assert_eq!(check, RealtimeCheck::Revalidate, "deadline within period + margin is never certified current");
                    continue;
                }
            }
            // Cross connection, no expedite: within the bound (100 ms), no manual poll.
            tokio::time::sleep(Duration::from_millis(150)).await;
            let later = snap::with_mode(SHORT, async { realtime_check(&state, &cert, period) }).await;
            assert_eq!(later, RealtimeCheck::Revalidate, "{case}");
            let _ = mirror.counters();
        }
    });
}

/// The ordering hazard the API removes: an authority change committed while the
/// full recheck runs (after the stamp, before the fence read). The publication
/// worker republishes the entry at the new versions before the recheck returns,
/// so the fence read afterwards matches the entry and a certificate IS issued;
/// it must turn `Revalidate` once the mirror applied the change, because the
/// stamp predates it.
/// With a stamp taken after the recheck (not expressible: no public stamp) the
/// same sequence would certify `Current`.
#[test]
fn realtime_change_during_the_recheck_is_never_certified_current() {
    run(|| async {
        let f = Fixture::new().await;
        let state = f.ready().await;
        let mirror = f.mirror(&state, SHORT).await;
        let session = state.store.get_session(&f.session).await.unwrap();
        let period = Duration::from_secs(30);
        // A publication lookup that loses a race (try_lock) or finds the entry
        // mid-rebuild is a safe `None`; retry until one attempt certifies.
        let mut cert = None;
        for attempt in 0..20i64 {
            cert = snap::with_mode(SHORT, realtime_certify_with(&state, &session, false, || async {
                // The change, through a second Store (no expedite), mid-recheck.
                policy(&f.writer, "session.absoluteSecs", serde_json::json!(86_400 * 30 + attempt)).await;
                // Until the worker published this session's inputs at the new versions.
                for _ in 0..400 {
                    let reply = f.get(HOME, CHROME, OFF).await;
                    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
                    if reply.inputs.first_fences == 1 {
                        return Ok::<_, ()>(());
                    }
                    tokio::time::sleep(Duration::from_millis(25)).await;
                }
                panic!("request inputs never republished");
            }))
            .await
            .unwrap()
            .1;
            if cert.is_some() {
                break;
            }
        }
        let cert = cert.expect("an attempt certifies after a change made during its recheck");
        // After one complete poll that started after the change the certificate
        // is refused (before it, within the bound, it may still read Current).
        mirror.poll_now().await.unwrap();
        let after = snap::with_mode(SHORT, async { realtime_check(&state, &cert, period) }).await;
        assert_eq!(after, RealtimeCheck::Revalidate);
        // A certificate stamped after the change (a fresh full path) is Current.
        let fresh = certified(&state, &session, false, SHORT).await.expect("certifies");
        assert_eq!(snap::with_mode(SHORT, async { realtime_check(&state, &fresh, period) }).await, RealtimeCheck::Current);
    });
}

/// PostgreSQL (t49pg): mode off is today's path (no mirror, no reuse, no
/// certificate). Bounded mode with a live mirror reuses the alert-preference
/// rows and issues realtime certificates; a session deleted through another
/// AppState turns the certificate to `Revalidate` within the bound, with no
/// expedite. Sign-in writes still never move the settings revision (0119).
/// Runs only with `MW_TEST_PG`.
#[test]
fn postgres_watch_reuses_and_certifies_only_with_a_live_mirror() {
    use crate::request_inputs::snapshot::pg_tests as pg;
    let Some(dsn) = pg::pg_dsn() else { return };
    run(move || async move {
        let f = pg::Fixture::new(pg::pg_schema(&dsn).await).await;
        let state = f.b.state.clone();
        assert!(!state.store.uses_sqlite());
        let session = state.store.get_session(&f.session).await.unwrap();
        // Off: original reads, no mirror, no certificate.
        for _ in 0..3 {
            let reply = pg::send(&f.b, f.me_from(PUBLIC, CHROME), OFF).await;
            assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
            assert_eq!(reply.watch.prefs_reused + reply.watch.geo_reused, 0);
            assert_eq!(reply.inputs.first_fences, 0);
        }
        assert!(snap::with_mode(OFF, async { crate::request_inputs::snapshot::mirror(&state).is_none() }).await);
        assert!(certified(&state, &session, false, OFF).await.is_none(), "mode off: no certificate");
        // Bounded, live mirror: preference rows reused, certificate issued.
        f.ready(&f.b, pg::SHORT).await;
        let mirror = f.mirror(&f.b).await;
        let mut reused = false;
        for _ in 0..400 {
            let reply = pg::send(&f.b, f.me_from(PUBLIC, CHROME), pg::SHORT).await;
            assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
            if reply.watch.prefs_reused + reply.watch.geo_reused > 0 {
                reused = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(reused, "bounded watch on PostgreSQL reuses settings rows");
        assert!(certified(&state, &session, true, pg::SHORT).await.is_none(), "bearer: never");
        let mut cert = None;
        for _ in 0..40 {
            cert = certified(&state, &session, false, pg::SHORT).await;
            if cert.is_some() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let cert = cert.expect("a published engine session certifies on PostgreSQL");
        let period = Duration::from_secs(30);
        let (check, sql) = mw_store::pool_telemetry::measure_queries(snap::with_mode(pg::SHORT, async {
            realtime_check(&state, &cert, period)
        }))
        .await;
        assert_eq!(check, RealtimeCheck::Current);
        assert_eq!((sql.read.calls, sql.write.calls, sql.other.calls), (0, 0, 0));
        // Revoked through A (its own Store): B's check, no expedite in B.
        let expedites = mirror.counters().expedites;
        f.a.state.store.delete_session(&f.session).await.unwrap();
        let committed = tokio::time::Instant::now();
        tokio::time::sleep_until(committed + pg::BOUND).await;
        let later = snap::with_mode(pg::SHORT, async { realtime_check(&state, &cert, period) }).await;
        assert_eq!(later, RealtimeCheck::Revalidate, "within the bound, across AppStates");
        assert_eq!(mirror.counters().expedites, expedites);
        // 0119: no signin_identity trigger; sign-in writes never move the revision.
        let pool = sqlx::postgres::PgPoolOptions::new().max_connections(1).connect(&f.url).await.unwrap();
        let triggers: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pg_catalog.pg_trigger t JOIN pg_catalog.pg_class c ON c.oid=t.tgrelid JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE c.relname='signin_identity' AND n.nspname=current_schema() AND NOT t.tgisinternal")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(triggers, 0, "0119 dropped every signin_identity trigger");
        let mut tx = pool.begin().await.unwrap();
        let revision = "SELECT n FROM settings_revision WHERE id=1";
        let before: i64 = sqlx::query_scalar(revision).fetch_one(&mut *tx).await.unwrap();
        for address in ["pg@watch.test", "PG@watch.test", "alias@watch.test"] {
            sqlx::query("INSERT INTO signin_identity(account_id,address,seen_at,event_id) VALUES('e13-pg',$1,1,'e') ON CONFLICT(account_id) DO UPDATE SET address=excluded.address")
                .bind(address).execute(&mut *tx).await.unwrap();
        }
        sqlx::query("DELETE FROM signin_identity WHERE account_id='e13-pg'").execute(&mut *tx).await.unwrap();
        let after: i64 = sqlx::query_scalar(revision).fetch_one(&mut *tx).await.unwrap();
        tx.rollback().await.unwrap();
        assert_eq!(after, before, "sign-in writes never move the settings revision");
        pool.close().await;
    });
}
