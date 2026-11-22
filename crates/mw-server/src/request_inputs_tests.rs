//! Actual original router/Store/Engine tests; no authentication reader mocks.
use super::*;
use crate::request_inputs::test_control as inputs_control;

struct Reply {
    status: StatusCode,
    body: Value,
    sql: mw_store::pool_telemetry::QuerySnapshot,
    inputs: inputs_control::Counts,
    native: test_control::Counts,
    auth_calls: u64,
    guards: u32,
}
fn peer(mut request: Request<Body>) -> Request<Body> {
    request.extensions_mut().insert(axum::extract::ConnectInfo(
        "127.0.0.1:48213".parse::<std::net::SocketAddr>().unwrap()));
    request.headers_mut().insert("user-agent", "Mozilla/5.0".parse().unwrap());
    request
}
fn me(fixture: &Fixture) -> Request<Body> {
    let mut request = fixture.request();
    *request.method_mut() = axum::http::Method::GET;
    *request.uri_mut() = "/api/me".parse().unwrap();
    *request.body_mut() = Body::empty();
    peer(request)
}
async fn send(fixture: &Fixture, request: Request<Body>, enabled: bool) -> Reply {
    // Disable the existing native metadata dispatcher in BOTH arms. Thus these
    // are ordinary handlers using actual request INPUT readers, not ready JSON
    // or a smaller native path versus the original full operation.
    let measured = crate::feature_gate::auth_boundary::count_auth(
        crate::published_guard_context::test_control::count(
            inputs_control::measure(enabled, test_control::count(true,
                fixture.router.clone().oneshot(request)))));
    let (((((result, native), inputs), guards), auth_calls), sql) =
        mw_store::pool_telemetry::measure_queries(measured).await;
    let response = result.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 4 * 1024 * 1024).await.unwrap();
    Reply { status, body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        sql, inputs, native, auth_calls, guards }
}
async fn warm(fixture: &Fixture) -> Arc<publication::Worker> {
    let reply = send(fixture, peer(fixture.request()), true).await;
    assert_eq!(reply.status, StatusCode::OK);
    let worker = publication::Worker::start(fixture.measured.clone()).unwrap();
    await_ready(fixture, &worker).await;
    worker
}
async fn await_ready(fixture: &Fixture, worker: &publication::Worker) {
    await_actor_ready(fixture, worker, &fixture.session).await;
}
async fn await_actor_ready(fixture: &Fixture, worker: &publication::Worker, cookie: &str) {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let environment = crate::request_inputs::Environment::capture().unwrap();
            if let Some(entry) = worker.request_inputs(cookie, &environment)
                && fixture.measured.native_publication_fence(cookie, &entry.domains, &entry.subjects)
                    .await.unwrap().is_some_and(|source| entry.matches(&source)) { break; }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("original authenticated route must publish bounded request inputs");
}
fn no_dispatch(reply: &Reply) {
    assert_eq!(reply.native.dispatches + reply.native.ordinary_dispatches
        + reply.native.restart_dispatches + reply.native.prepared_results, 0);
}

#[test]
fn ordinary_request_inputs_share_parsed_route_only_for_exact_committed_source() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let worker = publication::Worker::start(fixture.measured.clone()).unwrap();
        let user = "second@native-composite.test";
        let credentials = Credentials { username: user.into(), password: "owned-second-only".into() };
        let second_account = fixture.writer.create_account(&NewAccount {
            kind: AccountKind::Imap, host: "127.0.0.1", port: 9, tls: "starttls",
            username: user, sync_policy_json: "{}",
        }, &credentials).await.unwrap();
        fixture.writer.remember_account_names(&second_account, &[user]).await.unwrap();
        let second_session = fixture.writer.create_session(&second_account, user, "engine", "engine", &credentials).await.unwrap();
        fixture.writer.set_session_authentication(&second_session, "password").await.unwrap();
        fixture.writer.bind_verified_primary(&second_session, mw_store::identity_principal::VerifiedPrimary {
            account_id: &second_account, kind: "mailbox", authority: "fixture-native", subject: user,
        }, chrono::Utc::now().timestamp()).await.unwrap().unwrap();
        let first = worker.build_request_once_for_test(&fixture.measured, &fixture.session, &fixture.account)
            .await.unwrap().expect("real first source frame");
        let first_charge = worker.route_retention_for_test();
        assert_eq!(first_charge.0, 1);
        assert!(first_charge.1 > 0);
        let second = worker.build_request_once_for_test(&fixture.measured, &second_session, &second_account)
            .await.unwrap().expect("real second source frame");
        assert_eq!(worker.route_retention_for_test(), first_charge, "same component charged exactly once");
        let first_inputs = first.request.as_ref().unwrap();
        let second_inputs = second.request.as_ref().unwrap();
        assert_ne!(first_inputs.account.id, second_inputs.account.id);
        assert_ne!(first_inputs.identity.identity.address, second_inputs.identity.identity.address);
        assert!(Arc::ptr_eq(&first_inputs.configuration, &second_inputs.configuration),
            "same exact committed domain parser input must share one parsed configuration");
        let private_before = first_inputs.retained_bytes();
        let mut changed = fixture.writer.get_domain_config("native-composite.test").await.unwrap().unwrap();
        changed.upstream_json = json!({"imap":{"host":"127.0.0.1","port":10,"tls":"starttls","auth":"password","allowPrivate":true}}).to_string();
        fixture.writer.put_domain_config(&changed).await.unwrap();
        let third = worker.build_request_once_for_test(&fixture.measured, &second_session, &second_account)
            .await.unwrap().expect("changed committed source remains supported DATA");
        let third_inputs = third.request.as_ref().unwrap();
        assert_eq!(worker.route_retention_for_test().0, 2, "held old DATA and new version are both charged");
        assert!(!Arc::ptr_eq(&first_inputs.configuration, &third_inputs.configuration));
        assert_ne!(first_inputs.upstream, third_inputs.upstream);
        assert_eq!(first_inputs.retained_bytes(), private_before, "old immutable entry remains unchanged");
        assert!(first_inputs.configuration.imap.as_ref().is_some_and(|endpoint| endpoint.port == 9));
        assert!(third_inputs.configuration.imap.as_ref().is_some_and(|endpoint| endpoint.port == 10));
    });
}

#[test]
fn ordinary_request_inputs_enabled_guards_reuse_stored_session_and_keep_live_effects() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let worker = warm(&fixture).await;
        let mut bot = me(&fixture);
        bot.headers_mut().insert("user-agent", "GPTBot/1.0".parse().unwrap());
        let original = send(&fixture, bot, false).await;
        assert_eq!(original.status, StatusCode::OK, "actual signed-in evidence retains the original UA exception");
        assert!(original.inputs.legacy_guard_session_reads > 0);
        let mut bot = me(&fixture);
        bot.headers_mut().insert("user-agent", "GPTBot/1.0".parse().unwrap());
        let candidate = send(&fixture, bot, true).await;
        assert_eq!(candidate.status, original.status);
        assert_eq!(candidate.body, original.body);
        assert!(candidate.inputs.stored_reader_hits > 0);
        assert_eq!(candidate.inputs.legacy_guard_session_reads, 0);
        assert_eq!(candidate.inputs.session_decodes, 1);
        assert_eq!(candidate.auth_calls, original.auth_calls);
        no_dispatch(&candidate);

        policy(&fixture.writer, "security.sessionBinding.enabled", json!(true)).await;
        // A real newly issued/bound cookie has no Watch record yet. Publishing
        // its sealed source must not create that mutable record itself.
        let source = fixture.writer.get_primary_session(&fixture.session).await.unwrap();
        let cookie = fixture.writer.create_session(&fixture.account, &source.username, "engine", "engine", &source.credentials).await.unwrap();
        fixture.writer.set_session_authentication(&cookie, "password").await.unwrap();
        fixture.writer.bind_verified_primary(&cookie, mw_store::identity_principal::VerifiedPrimary {
            account_id: &fixture.account, kind: "mailbox", authority: "fixture-native", subject: &source.username,
        }, chrono::Utc::now().timestamp()).await.unwrap().unwrap();
        worker.hint_request_inputs(&cookie, &fixture.account);
        await_actor_ready(&fixture, &worker, &cookie).await;
        let hash = crate::push_relay::hash_token(&cookie);
        let key = format!("sess-sec:v1:{hash}");
        assert!(fixture.writer.get_setting(&key).await.unwrap().is_none());
        let request = || {
            let mut request = me(&fixture);
            let csrf = fixture.writer.session_csrf_token(&cookie);
            request.headers_mut().insert("cookie", format!("mw_session={cookie}; mw_csrf={csrf}").parse().unwrap());
            request.headers_mut().insert("x-csrf-token", csrf.parse().unwrap());
            request
        };
        let candidate = send(&fixture, request(), true).await;
        assert_eq!(candidate.status, StatusCode::OK);
        assert!(candidate.inputs.stored_reader_hits > 0, "enabled Watch creates its real dynamic record from typed stored DATA");
        assert_eq!(candidate.inputs.legacy_guard_session_reads, 0);
        assert_eq!(candidate.inputs.session_decodes, 1);
        assert!(fixture.writer.get_setting(&key).await.unwrap().is_some(), "original SessionSec write/effect remains live");
        let reference = send(&fixture, request(), false).await;
        assert_eq!(candidate.status, reference.status);
        assert_eq!(candidate.body, reference.body);
        no_dispatch(&candidate);
    });
}

#[test]
fn ordinary_request_inputs_guard_session_revocation_refuses_without_anonymous_fallback() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let _worker = warm(&fixture).await;
        let mut request = me(&fixture);
        request.headers_mut().insert("user-agent", "GPTBot/1.0".parse().unwrap());
        let pause = inputs_control::EffectPause::new(crate::published_guard_context::Guard::Protection);
        let pending = inputs_control::stored_probe(pause.clone(), send(&fixture, request, true));
        tokio::pin!(pending);
        tokio::time::timeout(Duration::from_secs(10), async {
            tokio::select! { _=pause.entered.notified()=>{}, _=&mut pending=>panic!("request did not reach its actual stored-session guard reader") }
        }).await.expect("bounded real guard-reader wait");
        fixture.writer.delete_session(&fixture.session).await.unwrap();
        pause.release.notify_one();
        let reply = tokio::time::timeout(Duration::from_secs(10), pending).await.unwrap();
        assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(reply.body["code"], "requestInputSourceChanged");
        assert_eq!(reply.inputs.stored_reader_hits, 0);
        assert_eq!(reply.inputs.legacy_guard_session_reads, 0, "changed consumed DATA cannot be treated as anonymous/legacy evidence");
        assert_eq!(reply.auth_calls, 0);
        no_dispatch(&reply);
    });
}

#[test]
fn ordinary_request_inputs_api_me_and_unmarked_jmap_remove_reconstruction_reads() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let original_me = send(&fixture, me(&fixture), false).await;
        let original_mail = send(&fixture, peer(fixture.request()), false).await;
        let worker = warm(&fixture).await;
        let environment = crate::request_inputs::Environment::capture().unwrap();
        let (private, resident, budget) = worker.request_retention(&fixture.session, &environment).unwrap();
        let projected = resident.saturating_add(private.saturating_mul(9999));
        eprintln!("{}", json!({"kind":"ordinary-request-input-logical-budget","measuredAccounts":1,
            "privateBytesPerEntry":private,"residentLogicalBytes":resident,"budgetBytes":budget,
            "projectedAccounts":10000,"projectedLogicalBytes":projected,"projectedWithinBudget":projected<=budget,
            "projectionAssumesSameShapeAndSharedGlobalDomainData":true,"workingSetFitMeasured":false,"rssBytes":null}));
        for (request, original, expected_dispatches) in [
            (me(&fixture), original_me, 0), (peer(fixture.request()), original_mail, 1),
        ] {
            let reply = send(&fixture, request, true).await;
            assert_eq!(reply.status, StatusCode::OK);
            assert_eq!(reply.body, original.body);
            assert_eq!(reply.inputs.first_fences, 1);
            assert_eq!(reply.inputs.session_decodes, 1);
            assert_eq!(reply.inputs.legacy_session_reads, 0);
            assert_eq!(reply.inputs.failure_classifications, 0);
            assert_eq!(reply.native.ordinary_dispatches, expected_dispatches);
            assert_eq!(reply.native.dispatches + reply.native.prepared_results + reply.native.restarts, 0);
            assert_eq!(reply.auth_calls, original.auth_calls, "original evaluators still run at their original boundaries");
            assert_eq!(reply.guards & 511, 511, "actual peer guards consume typed inputs");
            assert_eq!(reply.sql.write.calls, 0);
            assert!(reply.sql.read.calls + reply.sql.other.calls < original.sql.read.calls + original.sql.other.calls);
            eprintln!("{}", json!({"kind":"ordinary-request-input-reads","jmap":expected_dispatches==1,
                "originalReads":original.sql.read.calls,"originalOther":original.sql.other.calls,
                "inputReads":reply.sql.read.calls,"inputOther":reply.sql.other.calls,
                "sourceFences":reply.inputs.first_fences+reply.inputs.refresh_fences,
                "primaryDecodes":reply.inputs.session_decodes,"legacySessionReads":reply.inputs.legacy_session_reads,
                "workerQueriesIncluded":false,"tcpMeasured":false}));
        }
    });
}

fn held_body(fixture: &Fixture) -> (Request<Body>, Arc<tokio::sync::Notify>, tokio::sync::oneshot::Sender<()>) {
    let mut request = peer(fixture.request());
    let bytes = serde_json::to_vec(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],
        "methodCalls":[["Email/get",{"accountId":fixture.account,"ids":fixture.ids,"properties":default_properties()},"headers"]]})).unwrap();
    let entered = Arc::new(tokio::sync::Notify::new());
    let body_entered = entered.clone();
    let (release, receiver) = tokio::sync::oneshot::channel();
    *request.body_mut() = Body::from_stream(futures_util::stream::once(async move {
        body_entered.notify_one();
        receiver.await.map_err(|_| std::io::Error::other("owned body release dropped"))?;
        Ok::<_, std::io::Error>(bytes::Bytes::from(bytes))
    }));
    (request, entered, release)
}

#[test]
fn ordinary_request_inputs_revocation_while_body_waits_never_dispatches() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let _worker = warm(&fixture).await;
        let (request, entered, release) = held_body(&fixture);
        let pending = send(&fixture, request, true);
        tokio::pin!(pending);
        tokio::time::timeout(Duration::from_secs(10), async {
            tokio::select! { _=entered.notified()=>{}, _=&mut pending=>panic!("request did not wait for its actual body") }
        }).await.expect("bounded actual body wait");
        fixture.writer.delete_session(&fixture.session).await.unwrap();
        release.send(()).unwrap();
        let reply = pending.await;
        let original = send(&fixture, peer(fixture.request()), false).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
        assert_eq!(reply.status, original.status); assert_eq!(reply.body, original.body);
        assert_eq!(reply.inputs.failure_classifications, 1);
        no_dispatch(&reply);
    });
}

#[test]
fn ordinary_request_inputs_policy_body_wait_refuses_without_successful_auth_effects() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let _worker = warm(&fixture).await;
        let before = fixture.writer.get_primary_session_with_activity(&fixture.session).await.unwrap().1;
        let (request, entered, release) = held_body(&fixture);
        let pending = send(&fixture, request, true);
        tokio::pin!(pending);
        tokio::time::timeout(Duration::from_secs(10), async {
            tokio::select! { _=entered.notified()=>{}, _=&mut pending=>panic!("request did not wait for its actual body") }
        }).await.expect("bounded actual body wait");
        policy(&fixture.writer, "access.protocols", json!([])).await;
        release.send(()).unwrap();
        let reply = pending.await;
        assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(reply.body["code"], "requestInputSourceChanged");
        assert_eq!(reply.inputs.failure_classifications, 1);
        assert_eq!(reply.auth_calls, 0, "failure classification never calls successful authed");
        assert_eq!(reply.sql.write.calls, 0, "healthy classification does not touch or reseed Session activity");
        assert_eq!(before, fixture.writer.get_primary_session_with_activity(&fixture.session).await.unwrap().1);
        no_dispatch(&reply);
        let original = send(&fixture, peer(fixture.request()), false).await;
        let next = send(&fixture, peer(fixture.request()), true).await;
        assert_eq!(original.status, StatusCode::FORBIDDEN);
        assert_eq!(next.status, original.status); assert_eq!(next.body, original.body);
        no_dispatch(&next);
    });
}

#[test]
fn ordinary_request_inputs_revocation_after_permit_before_connector_is_final() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let _worker = warm(&fixture).await;
        let lock = crate::engine_mode::connect_lock(&fixture.account);
        let held = lock.lock().await;
        let entered = Arc::new(tokio::sync::Notify::new());
        let pending = inputs_control::connector_probe(entered.clone(), send(&fixture, peer(fixture.request()), true));
        tokio::pin!(pending);
        tokio::time::timeout(Duration::from_secs(10), async {
            tokio::select! { _=entered.notified()=>{}, _=&mut pending=>panic!("request did not reach the actual held connector lock") }
        }).await.expect("bounded actual connector wait");
        fixture.writer.delete_session(&fixture.session).await.unwrap();
        drop(held);
        let reply = pending.await;
        let original = send(&fixture, peer(fixture.request()), false).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
        assert_eq!(reply.status, original.status); assert_eq!(reply.body, original.body);
        assert_eq!(reply.inputs.session_decodes, 1);
        assert_eq!(reply.inputs.failure_classifications, 1);
        no_dispatch(&reply);
    });
}

#[test]
fn ordinary_request_inputs_initial_changed_policy_uses_original_error_and_rebuilds() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let _worker = warm(&fixture).await;
        policy(&fixture.writer, "access.protocols", json!([])).await;
        let original = send(&fixture, peer(fixture.request()), false).await;
        let refused = send(&fixture, peer(fixture.request()), true).await;
        assert_eq!(original.status, StatusCode::FORBIDDEN);
        assert_eq!(refused.status, original.status); assert_eq!(refused.body, original.body);
        assert_eq!(refused.inputs.session_decodes, 0);
        assert!(refused.inputs.legacy_session_reads > 0);
        no_dispatch(&refused);
        policy(&fixture.writer, "access.protocols", json!(["imap"])).await;
        warm(&fixture).await;
        let reply = send(&fixture, peer(fixture.request()), true).await;
        assert_eq!(reply.status, StatusCode::OK);
        assert_eq!(reply.inputs.session_decodes, 1); assert_eq!(reply.inputs.legacy_session_reads, 0);
    });
}

#[test]
fn ordinary_request_inputs_unsupported_credentials_and_selectors_stay_original() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let _worker = warm(&fixture).await;
        for (header, value) in [("authorization", "Bearer invalid-owned-fixture"),
            (crate::linked_accounts::SELECTOR, "not-an-owned-link")] {
            let make = || { let mut request = peer(fixture.request());
                request.headers_mut().insert(header, value.parse().unwrap()); request };
            let original = send(&fixture, make(), false).await;
            let candidate = send(&fixture, make(), true).await;
            assert_eq!(candidate.status, original.status); assert_eq!(candidate.body, original.body);
            assert_eq!(candidate.inputs.first_fences, 0); assert_eq!(candidate.inputs.session_decodes, 0);
            no_dispatch(&candidate);
        }
    });
}

#[test]
fn ordinary_request_inputs_malformed_body_keeps_original_error_and_no_dispatch() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let _worker = warm(&fixture).await;
        let make = || { let mut request = peer(fixture.request()); *request.body_mut() = Body::from("{"); request };
        let original = send(&fixture, make(), false).await;
        let candidate = send(&fixture, make(), true).await;
        assert_eq!(original.status, StatusCode::BAD_REQUEST);
        assert_eq!(candidate.status, original.status); assert_eq!(candidate.body, original.body);
        no_dispatch(&candidate);
    });
}

#[test]
fn ordinary_request_inputs_publication_is_bound_to_committed_policy_revision() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let pause = test_control::pause(test_control::Stage::PublicationBeforeFinal, &fixture.session);
        let first = send(&fixture, peer(fixture.request()), true).await;
        assert_eq!(first.status, StatusCode::OK);
        tokio::time::timeout(Duration::from_secs(10), pause.entered.notified()).await.unwrap();
        policy(&fixture.writer, "access.protocols", json!(["imap", "pop3"])).await;
        pause.release.notify_one();
        let worker = publication::Worker::start(fixture.measured.clone()).unwrap();
        let entry = tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let environment = crate::request_inputs::Environment::capture().unwrap();
                if let Some(entry) = worker.request_inputs(&fixture.session, &environment) { break entry; }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }).await.unwrap();
        let source = fixture.measured.native_publication_fence(&fixture.session, &entry.domains, &entry.subjects).await.unwrap().unwrap();
        assert!(entry.matches(&source));
        assert_eq!(entry.snapshot.revision(&fixture.measured).unwrap().number(),
            source.primary_policy_revision(&fixture.measured).unwrap().unwrap().number());
        assert_eq!(entry.request.as_ref().unwrap().value("access.protocols"), Some(&json!(["imap", "pop3"])));
        let reply = send(&fixture, peer(fixture.request()), true).await;
        assert_eq!(reply.status, StatusCode::OK); assert_eq!(reply.inputs.legacy_session_reads, 0);
    });
}

#[test]
fn ordinary_request_inputs_new_mutable_policy_refuses_at_original_guard_load() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        policy(&fixture.writer, "security.sessionBinding.enabled", json!(false)).await;
        let worker = warm(&fixture).await;
        let environment = crate::request_inputs::Environment::capture().unwrap();
        let entry = worker.request_inputs(&fixture.session, &environment).unwrap();
        assert_eq!(entry.guard.as_ref().unwrap().global("security.sessionBinding.enabled").unwrap().value, json!(false));
        let pause = inputs_control::EffectPause::new(crate::published_guard_context::Guard::Watch);
        let pending = inputs_control::effect_probe(pause.clone(), send(&fixture, peer(fixture.request()), true));
        tokio::pin!(pending);
        tokio::time::timeout(Duration::from_secs(10), async {
            tokio::select! { _=pause.entered.notified()=>{}, _=&mut pending=>panic!("original Watch load point was not reached") }
        }).await.expect("bounded original mutable load wait");
        policy(&fixture.writer, "security.sessionBinding.enabled", json!(true)).await;
        let actual = crate::settings_registry::effective_for(&fixture.writer,
            "security.sessionBinding.enabled", &Default::default()).await.unwrap().unwrap();
        assert_eq!(actual.value, json!(true));
        pause.release.notify_one();
        let reply = pending.await;
        assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(reply.body["code"], "requestInputSourceChanged");
        assert_ne!(reply.guards & 1, 0, "the earlier actual Protocol guard consumed the old frame");
        assert_eq!(reply.guards & 64, 0, "refuse BEFORE the old disabled Watch configuration/effects are consumed");
        assert_eq!(reply.auth_calls, 0); assert_eq!(reply.inputs.failure_classifications, 0);
        assert_eq!(reply.inputs.legacy_session_reads, 0);
        no_dispatch(&reply);
    });
}
