//! Real router/data-path oracle. No authentication, policy, store or response
//! getter is mocked; the installed transport refuses external network/sending.
use super::*;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use mw_engine::{Engine, MailSubmitter, account::AccountRuntime, backend::*};
use mw_store::{
    AccountKind, Credentials, DomainConfigRow, MailboxUpsert, MessageUpsert, NewAccount, ServerKey,
    Store,
};
use serde_json::json;
use std::{
    future::Future,
    sync::{Arc, OnceLock},
    time::Duration,
};
use tower::ServiceExt;

struct ReadOnlyBackend;
#[async_trait::async_trait]
impl AccountBackend for ReadOnlyBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(Default::default())
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(vec![RawMailbox {
            mailbox_ref: RawMailboxRef {
                name: "INBOX".into(),
                uidvalidity: 1,
            },
            role: MailboxRole::Inbox,
            parent: None,
            uidnext: 51,
            highestmodseq: 0,
            total: 50,
            unread: 50,
        }])
    }
    async fn sync_mailbox(&self, _: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        Ok(MailboxDelta {
            added: vec![],
            flag_changes: vec![],
            removed: vec![],
            next_cursor: c.clone(),
        })
    }
    async fn fetch_raw(&self, _: &[MessageRef]) -> Result<Vec<RawMessage>> {
        Err(EngineError::Unsupported(
            "fixture must use owned cached source".into(),
        ))
    }
    async fn store_flags(&self, _: &[MessageRef], _: &[Flag], _: &[Flag]) -> Result<()> {
        Err(EngineError::Unsupported("read-only fixture".into()))
    }
    async fn move_messages(&self, _: &[MessageRef], _: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("read-only fixture".into()))
    }
    async fn append(&self, _: &RawMailboxRef, _: &[u8], _: &[Flag]) -> Result<MessageRef> {
        Err(EngineError::Unsupported("read-only fixture".into()))
    }
    async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported("no fixture watch".into()))
    }
}
struct NoSend;
#[async_trait::async_trait]
impl MailSubmitter for NoSend {
    async fn submit(&self, _: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> {
        Err(EngineError::Unsupported("no fixture submission".into()))
    }
}
struct Fixture {
    router: Router,
    writer: Store,
    measured: Store,
    engine: Arc<Engine>,
    session: String,
    account: String,
    ids: Vec<String>,
    db: std::path::PathBuf,
    db_key: Option<mw_store::encryption::DbKey>,
}
fn default_properties() -> Vec<Value> {
    // Parse the actual composed fixture constant rather than duplicate a small
    // convenient subset. The existing JS contract ties these arrays to the UI.
    let source = include_str!("../../../scripts/perf/fixture-properties.mjs");
    let arrays = [
        "HEADER_PROPERTIES",
        "HEADER_PRELOAD_PROPERTIES",
        "DELIVERY_HEADER_PROPERTIES",
        "OPTIONAL_HEADER_PROPERTIES",
    ];
    let mut out = Vec::new();
    for name in arrays {
        let start = source
            .find(&format!("export const {name}=Object.freeze(["))
            .unwrap();
        let body = &source[start..];
        let start = body.find('[').unwrap() + 1;
        let end = body[start..].find(']').unwrap() + start;
        // The sole single-quoted fixture array contains literal ASCII strings.
        let json = format!("[{}]", body[start..end].replace('\'', "\""));
        out.extend(serde_json::from_str::<Vec<Value>>(&json).unwrap());
    }
    assert_eq!(out.len(), 36, "must cover actual default list properties");
    out
}
async fn policy(store: &Store, key: &str, value: Value) {
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "native-admission-fixture".into(),
            updated_at: chrono::Utc::now().timestamp(),
        })
        .await
        .unwrap();
}
impl Fixture {
    async fn new() -> Self {
        let dir =
            std::env::temp_dir().join(format!("mw-native-composite-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let db = dir.join("mail.db");
        let key = ServerKey::generate();
        let db_key = mw_store::encryption::global()
            .unwrap()
            .key_for_open(&db)
            .unwrap()
            .cloned();
        let writer = Store::open(db.to_str().unwrap(), key.clone())
            .await
            .unwrap();
        let user = "person@native-composite.test";
        let credentials = Credentials {
            username: user.into(),
            password: "fixture-secret-never-log".into(),
        };
        writer.put_domain_config(&DomainConfigRow{name:"native-composite.test".into(),upstream_json:json!({"imap":{"host":"127.0.0.1","port":9,"tls":"starttls","auth":"password","allowPrivate":true}}).to_string(),allowlist_json:"[]".into(),blocklist_json:"[]".into(),enabled:true,preferred:false,settings_json:"{}".into(),updated_at:1}).await.unwrap();
        let account = writer
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "127.0.0.1",
                    port: 9,
                    tls: "starttls",
                    username: user,
                    sync_policy_json: "{}",
                },
                &credentials,
            )
            .await
            .unwrap();
        writer
            .remember_account_names(&account, &[user])
            .await
            .unwrap();
        let mailbox = writer
            .upsert_mailbox(&MailboxUpsert {
                account_id: &account,
                name: "INBOX",
                role: Some("inbox"),
                uidvalidity: 1,
                uidnext: 51,
                highestmodseq: 0,
                total: 50,
                unread: 50,
                parent_id: None,
            })
            .await
            .unwrap();
        let mut ids = Vec::new();
        for uid in 1..=50 {
            let raw = format!(
                "From: sender@native-composite.test\r\nTo: {user}\r\nDelivered-To: {user}\r\nSubject: Composite {uid}\r\nMessage-ID: <{uid}@native-composite.test>\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nOwned cached body.\r\n"
            );
            let parsed = mw_mime::parse(raw.as_bytes()).unwrap();
            let envelope = mw_engine::encode_message_envelope(&parsed.email, raw.as_bytes());
            ids.push(
                writer
                    .upsert_message_with_body(
                        &MessageUpsert {
                            account_id: &account,
                            mailbox_id: &mailbox,
                            uid,
                            uidvalidity: 1,
                            message_id: None,
                            thread_id: None,
                            internaldate: Some("2026-11-01T00:00:00Z"),
                            size: raw.len() as u64,
                            flags_json: "[]",
                            envelope: Some(&envelope),
                            blob_ref: None,
                        },
                        raw.as_bytes(),
                    )
                    .await
                    .unwrap()
                    .0,
            );
        }
        let session = writer
            .create_session(&account, user, "engine", "engine", &credentials)
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
                    subject: user,
                },
                chrono::Utc::now().timestamp(),
            )
            .await
            .unwrap()
            .unwrap();
        policy(&writer, "access.protocols", json!(["imap"])).await;
        policy(&writer, "ratelimit.jmap.perAccountConcurrent", json!(1)).await;
        let captured = Arc::new(OnceLock::new());
        let captured_engine = Arc::new(OnceLock::new());
        let install_engine = captured_engine.clone();
        let install = captured.clone();
        let registered = account.clone();
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
            crate::V6Config {
                admin_enabled: false,
                ..Default::default()
            },
            crate::sso::SsoProviderSource::Store,
            Some(Box::new(move |store| {
                assert!(install.set(store.clone()).is_ok());
                let engine = Arc::new(Engine::new(store.clone()));
                assert!(install_engine.set(engine.clone()).is_ok());
                engine.register_backend(
                    registered,
                    AccountRuntime::new(Arc::new(ReadOnlyBackend), Arc::new(NoSend), user),
                );
                engine
            })),
            crate::core_test_build_info::info().unwrap(),
        )
        .await
        .unwrap();
        let fixture = Self {
            router,
            writer,
            measured: captured.get().unwrap().clone(),
            engine: captured_engine.get().unwrap().clone(),
            session,
            account,
            ids,
            db,
            db_key,
        };
        // Exercise original router to warm only its real resolver caches.
        for _ in 0..2 {
            let reply = fixture.send(true).await;
            assert_eq!(reply.0, StatusCode::OK);
            assert_eq!(reply.1["methodResponses"][0][0], "Email/get");
        }
        fixture
    }
    async fn replace_account_kind(&self) {
        let options = mw_store::encryption::connect_options(&self.db, self.db_key.as_ref(), false);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        sqlx::query("UPDATE accounts SET kind='pop3' WHERE id=?1")
            .bind(&self.account)
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;
    }
    fn request(&self) -> Request<Body> {
        let csrf = self.writer.session_csrf_token(&self.session);
        Request::builder().method("POST").uri("/jmap/api").header("host","localhost").header("origin","http://localhost").header("content-type","application/json").header("cookie",format!("mw_session={}; mw_csrf={csrf}",self.session)).header("x-csrf-token",csrf)
            .body(Body::from(serde_json::to_vec(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/get",{"accountId":self.account,"ids":self.ids,"properties":default_properties()},"headers"]]})).unwrap())).unwrap()
    }
    async fn send(
        &self,
        original: bool,
    ) -> (
        StatusCode,
        Value,
        test_control::Counts,
        mw_store::pool_telemetry::QuerySnapshot,
    ) {
        let measured = test_control::count(original, self.router.clone().oneshot(self.request()));
        #[cfg(feature = "sql-shape-diagnostics")]
        let ((response, counts), sql) = {
            // One counter scope only: shapes owns the existing query counter.
            // These fixture-only records contain hashes and fixed token/phase
            // labels, never SQL text, parameters, identities or credentials.
            let (result, report) = mw_store::pool_telemetry::shapes::measure(measured).await;
            report.require_valid().expect("whole-router attribution must be complete");
            eprintln!("native-admission-query-phases original={original} {}",
                serde_json::to_string(&report).unwrap());
            (result, report.query_counts)
        };
        #[cfg(not(feature = "sql-shape-diagnostics"))]
        let ((response, counts), sql) = mw_store::pool_telemetry::measure_queries(measured).await;
        let response = response.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 4 * 1024 * 1024)
            .await
            .unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
            counts,
            sql,
        )
    }
}
// Full production-router debug futures exceed ordinary libtest stack budgets.
// Construct AND poll each future inside this bounded test-owned thread; the
// caller hands over only a small factory closure, never a prebuilt large future.
// Panics cross the join unchanged so assertion failures remain test failures.
fn run_fixture<F, Fut>(factory: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()>,
{
    let test = std::thread::Builder::new()
        .name("mw-native-admission-test".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("test runtime");
            runtime.block_on(factory());
        })
        .expect("bounded fixture thread");
    if let Err(panic) = test.join() {
        std::panic::resume_unwind(panic);
    }
}

#[test]
fn actual_default_fifty_header_router_uses_composite_and_matches_original_with_fewer_reads() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let candidate = fixture.send(false).await;
        let original = fixture.send(true).await;
        assert_eq!(candidate.0, StatusCode::OK);
        assert_eq!(
            candidate.2.dispatches, 1,
            "actual default36 request must USE candidate, not silently fall back"
        );
        assert_eq!(candidate.2.hints, 1);
        assert_eq!(
            candidate.2.observations, 2,
            "one preparation plus one final snapshot"
        );
        assert_eq!(candidate.1, original.1);
        assert_eq!(
            candidate.1["methodResponses"][0][1]["list"]
                .as_array()
                .unwrap()
                .len(),
            50
        );
        assert!(
            candidate.3.read.calls < original.3.read.calls,
            "physical whole-request read count must actually improve: candidate={:?}, original={:?}",
            candidate.3,
            original.3
        );
    });
}
#[test]
fn queued_permit_revalidates_independent_writer_disabled_flag_before_dispatch() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let warm = fixture.send(false).await;
        assert_eq!(warm.2.dispatches, 1);
        let revision = fixture.measured.settings_read_revision().await.unwrap();
        let knobs = crate::rate_leeway::knobs_at_revision(
            &fixture.measured,
            "native-composite.test",
            &revision,
        )
        .await;
        let held = crate::jmap_limits::request_with_knobs(&fixture.account, knobs)
            .await
            .unwrap();
        let pause = test_control::pause(test_control::Stage::AfterPrepared, &fixture.session);
        let request = fixture.send(false);
        tokio::pin!(request);
        tokio::select! {_=pause.entered.notified()=>{},reply=&mut request=>panic!("candidate did not reach preparation: {}",reply.0)}
        pause.release.notify_one();
        std::future::poll_fn(|cx| match request.as_mut().poll(cx) {
            std::task::Poll::Pending => std::task::Poll::Ready(()),
            std::task::Poll::Ready(_) => panic!("request must really wait behind held JMAP permit"),
        })
        .await;
        fixture
            .writer
            .set_setting(
                &format!("v6:admin:flags:{}", fixture.account),
                r#"{"disabled":true}"#,
            )
            .await
            .unwrap();
        drop(held);
        let reply = tokio::time::timeout(Duration::from_secs(5), request)
            .await
            .unwrap();
        assert_eq!(reply.0, StatusCode::UNAUTHORIZED);
        assert_eq!(reply.2.dispatches, 0);
        assert!(reply.1.get("methodResponses").is_none());
    });
}
#[test]
fn connector_wait_revalidates_credential_rejection_and_domain_retarget_before_dispatch() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        assert_eq!(fixture.send(false).await.2.dispatches, 1);
        let lock = crate::engine_mode::connect_lock(&fixture.account);
        let held = lock.lock().await;
        let pause = test_control::pause(test_control::Stage::BeforeConnector, &fixture.session);
        let request = fixture.send(false);
        tokio::pin!(request);
        tokio::select! {_=pause.entered.notified()=>{},reply=&mut request=>panic!("candidate did not reach connector wait: {}",reply.0)}
        pause.release.notify_one();
        std::future::poll_fn(|cx| match request.as_mut().poll(cx) {
            std::task::Poll::Pending => std::task::Poll::Ready(()),
            std::task::Poll::Ready(_) => {
                panic!("request must really wait behind held connector mutex")
            }
        })
        .await;
        let mut row = fixture
            .writer
            .get_domain_config("native-composite.test")
            .await
            .unwrap()
            .unwrap();
        row.upstream_json=json!({"imap":{"host":"different.native-composite.test","port":993,"tls":"implicit","auth":"password"}}).to_string();
        fixture.writer.put_domain_config(&row).await.unwrap();
        drop(held);
        let reply = tokio::time::timeout(Duration::from_secs(5), request)
            .await
            .unwrap();
        assert_eq!(reply.0, StatusCode::UNAUTHORIZED);
        assert_eq!(reply.2.dispatches, 0);
        assert!(reply.1.get("methodResponses").is_none());
    });
}
#[test]
fn same_account_sibling_credential_name_flag_is_not_skipped_by_composite_connector() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let creds = Credentials {
            username: "alias@native-composite.test".into(),
            password: "sibling-fixture-secret".into(),
        };
        fixture
            .writer
            .create_session(
                &fixture.account,
                "person@native-composite.test",
                "engine",
                "engine",
                &creds,
            )
            .await
            .unwrap();
        fixture
            .writer
            .set_setting(
                "v6:admin:flags:alias@native-composite.test",
                r#"{"disabled":true}"#,
            )
            .await
            .unwrap();
        let candidate = fixture.send(false).await;
        let original = fixture.send(true).await;
        assert_eq!(
            candidate.2.dispatches, 0,
            "sibling-only credential name must remain part of connector gate"
        );
        assert_ne!(candidate.0, StatusCode::OK);
        assert_eq!(candidate.0, original.0);
        assert_eq!(candidate.1, original.1);
    });
}
#[test]
fn connector_wait_revalidates_current_credential_revision_rejection() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        assert_eq!(fixture.send(false).await.2.dispatches, 1);
        let lock = crate::engine_mode::connect_lock(&fixture.account);
        let held = lock.lock().await;
        let pause = test_control::pause(test_control::Stage::BeforeConnector, &fixture.session);
        let request = fixture.send(false);
        tokio::pin!(request);
        tokio::select! {_=pause.entered.notified()=>{},reply=&mut request=>panic!("candidate did not reach connector wait: {}",reply.0)}
        pause.release.notify_one();
        std::future::poll_fn(|cx| match request.as_mut().poll(cx) {
            std::task::Poll::Pending => std::task::Poll::Ready(()),
            std::task::Poll::Ready(_) => panic!("must await held connector lock"),
        })
        .await;
        let current = fixture
            .writer
            .account_credential_revision(&fixture.account)
            .await
            .unwrap();
        assert!(
            fixture
                .writer
                .reject_account_credential_revision(&fixture.account, &current)
                .await
                .unwrap()
        );
        drop(held);
        let reply = tokio::time::timeout(Duration::from_secs(5), request)
            .await
            .unwrap();
        // Restart reaches the complete original connector check. A durable
        // credential rejection is its exact upstream refusal, not an invented
        // successful retry or the old generic late-fence 503.
        let original = fixture.send(true).await;
        assert_eq!(reply.0, StatusCode::BAD_GATEWAY);
        assert_eq!(reply.0, original.0);
        assert_eq!(reply.1, original.1);
        assert_eq!(reply.2.restarts, 1);
        assert_eq!(reply.2.dispatches + reply.2.restart_dispatches, 0);
        assert!(crate::engine_mode::stored_credentials_rejected(
            &fixture.writer.get_account(&fixture.account).await.unwrap()
        ));
        assert!(reply.1.get("methodResponses").is_none());
    });
}

#[test]
fn new_restrictive_policy_never_uses_prior_warm_inputs_or_changes_original_refusal() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        assert_eq!(fixture.send(false).await.2.dispatches, 1);
        policy(&fixture.writer, "access.protocols", json!([])).await;
        let candidate = fixture.send(false).await;
        let original = fixture.send(true).await;
        assert_eq!(candidate.0, StatusCode::FORBIDDEN);
        assert_eq!(candidate.2.dispatches, 0);
        assert_eq!(candidate.0, original.0);
        assert_eq!(candidate.1, original.1);
    });
}
#[test]
fn connector_wait_revalidates_independent_account_kind_change() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        assert_eq!(fixture.send(false).await.2.dispatches, 1);
        let lock = crate::engine_mode::connect_lock(&fixture.account);
        let held = lock.lock().await;
        let pause = test_control::pause(test_control::Stage::BeforeConnector, &fixture.session);
        let request = fixture.send(false);
        tokio::pin!(request);
        tokio::select! {_=pause.entered.notified()=>{},reply=&mut request=>panic!("candidate did not reach connector wait: {}",reply.0)}
        pause.release.notify_one();
        std::future::poll_fn(|cx| match request.as_mut().poll(cx) {
            std::task::Poll::Pending => std::task::Poll::Ready(()),
            std::task::Poll::Ready(_) => panic!("must await held connector lock"),
        })
        .await;
        fixture.replace_account_kind().await;
        drop(held);
        let reply = tokio::time::timeout(Duration::from_secs(5), request)
            .await
            .unwrap();
        assert_eq!(reply.0, StatusCode::UNAUTHORIZED);
        assert_eq!(reply.2.dispatches, 0);
        assert!(reply.1.get("methodResponses").is_none());
    });
}

#[test]
fn actual_default_properties_are_metadata_eligible_but_reader_and_result_references_are_not() {
    let mut request = json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/get",{"accountId":"a","ids":["id"],"properties":default_properties()},"x"]]});
    let bytes = serde_json::to_vec(&request).unwrap();
    assert!(eligible_request(&bytes).is_some());
    assert!(eligible_parsed_request(&request, bytes.len()).is_some());
    assert!(eligible_parsed_request(&request, mw_engine::jmap::core::MAX_SIZE_REQUEST + 1).is_none());
    let mut mutation = request.clone();
    mutation["methodCalls"][0][0] = json!("Email/set");
    assert!(eligible_parsed_request(&mutation, bytes.len()).is_none());
    let mut mixed = request.clone();
    mixed["methodCalls"].as_array_mut().unwrap().push(json!(["Mailbox/get",{},"m"]));
    assert!(eligible_parsed_request(&mixed, bytes.len()).is_none());
    request["methodCalls"][0][1]["fetchAllBodyValues"] = json!(true);
    assert!(eligible_request(&serde_json::to_vec(&request).unwrap()).is_none());
    request["methodCalls"][0][1]["fetchAllBodyValues"] = json!(false);
    request["methodCalls"][0][1]["#ids"] =
        json!({"resultOf":"q","name":"Email/query","path":"/ids"});
    assert!(eligible_request(&serde_json::to_vec(&request).unwrap()).is_none());
}

include!("native_read_restart_tests.rs");

include!("native_published_input_tests.rs");

include!("native_guard_consumer_tests.rs");

// Run alone in a dedicated child process: the profiler reads its environment
// once and owns process-global counters. This is an actual HTTP router oracle,
// not a call to the timing helpers with a fabricated future.
#[cfg(all(feature = "load-test-fixtures", not(feature = "sql-shape-diagnostics")))]
#[test]
#[ignore = "run exact/single-thread in a child with MW_FIXTURE_REQUEST_PROFILE=1"]
fn native_jmap_runtime_profile_records_real_dispatch_and_prepared_return() {
    assert_eq!(std::env::var("MW_FIXTURE_REQUEST_PROFILE").as_deref(), Ok("1"),
        "this explicit profiler oracle requires the flag before process startup");
    assert!(crate::fixture_profile::enabled(), "profiler was initialized without its required flag");
    fn calls(snapshot: &Value, phase: &str) -> u64 {
        let rows = snapshot["phases"].as_array().unwrap();
        assert_eq!(rows.len(), 29, "the fixed phase schema must remain unchanged");
        let matching: Vec<_> = rows.iter().filter(|row| row["phase"] == phase).collect();
        assert_eq!(matching.len(), 1);
        matching[0]["inclusive"]["count"].as_u64().unwrap()
    }
    async fn send(fixture: &Fixture, request: Request<Body>, published: bool)
        -> (StatusCode, Value, test_control::Counts, Value)
    {
        let pending = fixture.router.clone().oneshot(request);
        let (response, counts) = if published {
            test_control::count_published(pending).await
        } else {
            test_control::count(false, pending).await
        };
        let response = response.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), 4 * 1024 * 1024).await.unwrap();
        let value = serde_json::from_slice(&body).unwrap_or(Value::Null);
        (status, value, counts, crate::fixture_profile::snapshot())
    }
    fn owned_fifty(fixture: &Fixture, value: &Value) {
        assert_eq!(value["methodResponses"][0][0], "Email/get");
        let list = value["methodResponses"][0][1]["list"].as_array().unwrap();
        assert_eq!(list.len(), 50);
        for (row, id) in list.iter().zip(&fixture.ids) {
            assert_eq!(row["id"].as_str(), Some(id.as_str()));
        }
    }
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        // A legitimate real-peer read with bodyValues is ineligible for the
        // metadata optimization and must execute the ordinary JMAP handler.
        let mut ordinary_request = fixture.declared_request();
        ordinary_request.headers_mut().remove("x-mailwoman-read-shape");
        let mut properties = default_properties();
        properties.push(json!("bodyValues"));
        *ordinary_request.body_mut() = Body::from(serde_json::to_vec(&json!({
            "using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],
            "methodCalls":[["Email/get",{"accountId":fixture.account,"ids":fixture.ids,
                "properties":properties,"fetchAllBodyValues":true},"headers"]]
        })).unwrap());
        fixture.writer.touch_session(&fixture.session).await.unwrap();
        crate::fixture_profile::reset();
        let ordinary = send(&fixture, ordinary_request, false).await;
        assert_eq!(ordinary.0, StatusCode::OK);
        owned_fifty(&fixture, &ordinary.1);
        assert!(ordinary.1["methodResponses"][0][1]["list"][0]["bodyValues"]
            .as_object().is_some_and(|values| !values.is_empty()));
        assert_eq!(ordinary.2.ordinary_dispatches, 1);
        assert_eq!(ordinary.2.dispatches + ordinary.2.prepared_results, 0);
        for phase in ["jmap.handler", "jmap.after-body", "jmap.engine"] {
            assert_eq!(calls(&ordinary.3, phase), 1, "ordinary real route did not reach {phase}");
        }
        // The actual metadata-native Engine await is a distinct call site.
        fixture.writer.touch_session(&fixture.session).await.unwrap();
        crate::fixture_profile::reset();
        let native = send(&fixture, fixture.declared_request(), false).await;
        assert_eq!(native.0, StatusCode::OK);
        owned_fifty(&fixture, &native.1);
        assert_eq!(native.2.dispatches, 1);
        assert_eq!(native.2.ordinary_dispatches + native.2.prepared_results, 0);
        for phase in ["jmap.handler", "jmap.after-body", "jmap.engine"] {
            assert_eq!(calls(&native.3, phase), 1, "native real route did not reach {phase}");
        }
        policy(&fixture.writer, "performance.preparedMail", serde_json::to_value(
            mw_engine::PreparedMailOptions {enabled:true,..Default::default()}
        ).unwrap()).await;
        // Every warm-up is the same real-peer/default36 production request.
        let warm = tokio::time::timeout(Duration::from_secs(10), async {
            for _ in 0..200 {
                fixture.writer.touch_session(&fixture.session).await.unwrap();
                let reply = send(&fixture, fixture.declared_request(), true).await;
                assert_eq!(reply.0, StatusCode::OK);
                if reply.2.prepared_results == 1 {return reply;}
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            panic!("actual DEFAULT36 prepared result never became ready");
        }).await.expect("bounded real-router prepared warmup");
        fixture.writer.touch_session(&fixture.session).await.unwrap();
        let index = fixture.engine.search();
        let index_before = (index.list_projection_search_count(), index.list_projection_doc_read_count(), index.doc_fetch_count());
        // Prior response bodies are fully drained; no HTTP task is in flight.
        crate::fixture_profile::reset();
        let ready = send(&fixture, fixture.declared_request(), true).await;
        assert_eq!(ready.0, StatusCode::OK);
        owned_fifty(&fixture, &ready.1);
        assert_eq!(ready.1, warm.1, "timing hooks must not alter the prepared reply");
        assert_eq!(ready.2.prepared_results, 1);
        assert_eq!(ready.2.dispatches + ready.2.ordinary_dispatches + ready.2.restart_dispatches, 0);
        assert_eq!(calls(&ready.3, "jmap.handler"), 1);
        assert_eq!(calls(&ready.3, "jmap.after-body"), 1);
        assert_eq!(calls(&ready.3, "jmap.engine"), 0,
            "synchronous prepared emission must not invent an Engine dispatch await");
        assert_eq!(index_before, (index.list_projection_search_count(), index.list_projection_doc_read_count(), index.doc_fetch_count()));
        // Preserve the final authority boundary: real independent-writer session
        // revocation while the prepared request is actually waiting may not emit
        // bytes or enter Engine, even with the new surrounding timing frames.
        fixture.writer.touch_session(&fixture.session).await.unwrap();
        let pause = test_control::pause(test_control::Stage::BeforeConnector, &fixture.session);
        crate::fixture_profile::reset();
        let pending = send(&fixture, fixture.declared_request(), true);
        tokio::pin!(pending);
        tokio::select! {
            _ = pause.entered.notified() => {},
            response = &mut pending => panic!("prepared request did not wait at final authority boundary: {}", response.0),
            _ = tokio::time::sleep(Duration::from_secs(10)) => panic!("prepared authority pause was not reached"),
        }
        fixture.writer.delete_session(&fixture.session).await.unwrap();
        pause.release.notify_one();
        let revoked = pending.await;
        assert!(!revoked.0.is_success());
        assert_eq!(revoked.2.prepared_results + revoked.2.dispatches + revoked.2.ordinary_dispatches + revoked.2.restart_dispatches, 0);
        assert_eq!(calls(&revoked.3, "jmap.engine"), 0);
        eprintln!("{}", json!({"kind":"actual-load-only-jmap-profile-hooks",
            "sqlShapeFeature":false,"ordinaryEngineCalls":calls(&ordinary.3,"jmap.engine"),
            "nativeEngineCalls":calls(&native.3,"jmap.engine"),
            "readyHandlerCalls":calls(&ready.3,"jmap.handler"),
            "readyAfterBodyCalls":calls(&ready.3,"jmap.after-body"),
            "readyEngineCalls":calls(&ready.3,"jmap.engine"),
            "readyRows":50,"revokedDispatches":0,"realPeerGuardsIncluded":true}));
    });
}
