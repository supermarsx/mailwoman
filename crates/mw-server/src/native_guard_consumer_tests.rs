// Real original router/default36; these tests do not replace auth/data getters.
impl Fixture {
    fn declared_request(&self) -> Request<Body> {
        let mut request = self.request();
        request.headers_mut().insert(
            "x-mailwoman-read-shape",
            "native-metadata-v1".parse().unwrap(),
        );
        request.extensions_mut().insert(axum::extract::ConnectInfo(
            "127.0.0.1:48111".parse::<std::net::SocketAddr>().unwrap(),
        ));
        request
    }
    async fn send_declared_request(
        &self,
        request: Request<Body>,
    ) -> (
        StatusCode,
        Value,
        test_control::Counts,
        mw_store::pool_telemetry::QuerySnapshot,
        u32,
    ) {
        let ((result, used), sql) = mw_store::pool_telemetry::measure_queries(
            crate::published_guard_context::test_control::count(test_control::count_published(
                self.router.clone().oneshot(request),
            )),
        )
        .await;
        let (response, counts) = result;
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
            used,
        )
    }
    async fn send_declared(
        &self,
    ) -> (
        StatusCode,
        Value,
        test_control::Counts,
        mw_store::pool_telemetry::QuerySnapshot,
        u32,
    ) {
        self.send_declared_request(self.declared_request()).await
    }
}
#[test]
fn declared_default36_uses_guard_inputs_and_preserves_real_fifty_row_reply() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        fixture.warm_published().await;
        fixture
            .writer
            .touch_session(&fixture.session)
            .await
            .unwrap();
        // Equal peer/Host/CSRF on both arms; only the optional declaration differs.
        let mut original = fixture.declared_request();
        original.headers_mut().remove("x-mailwoman-read-shape");
        let original = fixture.send_declared_request(original).await;
        let ready = fixture.send_declared().await;
        assert_eq!(ready.0, StatusCode::OK);
        assert_eq!(ready.1, original.1);
        use crate::published_guard_context::Guard;
        let expected = (Guard::Protocol as u32)
            | (Guard::Probe as u32)
            | (Guard::Protection as u32)
            | (Guard::Inbound as u32)
            | (Guard::Client as u32)
            | (Guard::Maintenance as u32)
            | (Guard::Watch as u32)
            | (Guard::Geography as u32)
            | (Guard::Trust as u32);
        assert_eq!(
            ready.4 & expected,
            expected,
            "all reached configuration adapters must actually consume published DATA"
        );
        assert_eq!(ready.2.observations, 0);
        assert_eq!(ready.2.restarts, 0);
        assert_eq!(ready.2.dispatches + ready.2.prepared_results, 1);
        assert_eq!(
            ready.1["methodResponses"][0][1]["list"]
                .as_array()
                .unwrap()
                .len(),
            50
        );
        assert!(
            ready.3.read.calls < original.3.read.calls,
            "whole router counts: ready={:?},original={:?}",
            ready.3,
            original.3
        );
        eprintln!(
            "{}",
            json!({"kind":"declared-guard-whole-router-counts","originalReads":original.3.read.calls,"readyReads":ready.3.read.calls,"readyWrites":ready.3.write.calls,"readyOther":ready.3.other.calls,"usedMask":ready.4,"workerQueriesIncluded":false})
        );
    });
}
#[test]
fn declared_mutation_mixed_and_created_ids_never_reach_application_dispatch() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        fixture.warm_published().await;
        let sample = crate::published_guard_context::sample_request();
        let mut mutation = sample.clone();
        mutation["methodCalls"][0][0] = json!("Email/set");
        let mut mixed = sample.clone();
        mixed["methodCalls"]
            .as_array_mut()
            .unwrap()
            .push(json!(["Mailbox/get", {}, "other"]));
        let mut created = sample;
        created["createdIds"] = json!({});
        for body in [mutation, mixed, created] {
            let mut request = fixture.declared_request();
            *request.body_mut() = Body::from(serde_json::to_vec(&body).unwrap());
            let reply = fixture.send_declared_request(request).await;
            assert_eq!(reply.0, StatusCode::BAD_REQUEST);
            assert_eq!(reply.1["code"], "invalidDeclaredReadShape");
            assert_eq!(
                reply.2.dispatches
                    + reply.2.ordinary_dispatches
                    + reply.2.restart_dispatches
                    + reply.2.prepared_results,
                0
            );
        }
    });
}
async fn queued_declared(
    f: &Fixture,
    expire: bool,
    concurrent_touch: bool,
    revoke: bool,
) -> (
    StatusCode,
    Value,
    test_control::Counts,
    mw_store::pool_telemetry::QuerySnapshot,
    u32,
) {
    f.writer.touch_session(&f.session).await.unwrap();
    let warm = f.send_declared().await;
    assert_eq!(warm.0, StatusCode::OK);
    assert_ne!(warm.4, 0);
    let revision = f.measured.settings_read_revision().await.unwrap();
    let knobs =
        crate::rate_leeway::knobs_at_revision(&f.measured, "native-composite.test", &revision)
            .await;
    let held = crate::jmap_limits::request_with_knobs(&f.account, knobs)
        .await
        .unwrap();
    let pause = test_control::pause(test_control::Stage::AfterPrepared, &f.session);
    let pending = f.send_declared();
    tokio::pin!(pending);
    tokio::select! {_=pause.entered.notified()=>{},reply=&mut pending=>panic!("declared request did not prepare: {} used={}",reply.0,reply.4)}
    pause.release.notify_one();
    std::future::poll_fn(|cx| match pending.as_mut().poll(cx) {
        std::task::Poll::Pending => std::task::Poll::Ready(()),
        std::task::Poll::Ready(_) => panic!("must actually wait for held permit"),
    })
    .await;
    f.age_last_seen_for_restart(expire).await;
    if concurrent_touch {
        f.writer.touch_session(&f.session).await.unwrap();
    }
    if revoke {
        policy(&f.writer, "access.protocols", json!(["pop3"])).await;
    }
    drop(held);
    tokio::time::timeout(Duration::from_secs(5), pending)
        .await
        .expect("bounded declared read")
}
#[test]
fn declared_queued_due_touch_and_concurrent_touch_succeed_without_inner_restart() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        f.warm_published().await;
        for concurrent in [false, true] {
            let reply = queued_declared(&f, false, concurrent, false).await;
            assert_eq!(reply.0, StatusCode::OK);
            assert_ne!(reply.4, 0);
            assert_eq!(reply.2.restarts, 0);
            assert_eq!(reply.2.dispatches + reply.2.prepared_results, 1);
        }
    });
}
#[test]
fn declared_queued_expiry_and_independent_policy_revocation_cannot_fall_through() {
    run_fixture(|| async {
        for expire in [false, true] {
            let f = Fixture::new().await;
            f.warm_published().await;
            let reply = queued_declared(&f, expire, true, !expire).await;
            assert_ne!(reply.0, StatusCode::OK);
            assert_ne!(reply.4, 0);
            assert_eq!(reply.2.restarts, 0);
            assert_eq!(
                reply.2.dispatches
                    + reply.2.ordinary_dispatches
                    + reply.2.restart_dispatches
                    + reply.2.prepared_results,
                0
            );
            if expire {
                assert!(
                    matches!(
                        f.writer.get_session(&f.session).await,
                        Err(mw_store::StoreError::NotFound)
                    ),
                    "expiry cleanup remains terminal despite telemetry touch"
                );
            }
        }
    });
}

#[test]
fn declared_queued_raw_guard_updates_and_schema_loss_cannot_restart_past_outer_guards() {
    run_fixture(|| async {
        for raw_key in [
            Some("security.inbound-allowlist.v1"),
            Some("security.clientAccess.v1"),
            Some("maintenance.mode.v1"),
            None,
        ] {
            let f = Fixture::new().await;
            f.warm_published().await;
            f.writer.touch_session(&f.session).await.unwrap();
            let warm = f.send_declared().await;
            assert_eq!(warm.0, StatusCode::OK);
            assert_ne!(warm.4, 0);
            let pause = test_control::pause(test_control::Stage::BeforeConnector, &f.session);
            let pending = f.send_declared();
            tokio::pin!(pending);
            tokio::select! {_=pause.entered.notified()=>{},reply=&mut pending=>panic!("must reach late boundary: {}",reply.0)}
            if let Some(key) = raw_key {
                f.writer.set_setting(key, "{").await.unwrap();
            } else {
                let options =
                    mw_store::encryption::connect_options(&f.db, f.db_key.as_ref(), false);
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(1)
                    .connect_with(options)
                    .await
                    .unwrap();
                sqlx::query("DROP TRIGGER authority_settings_update")
                    .execute(&pool)
                    .await
                    .unwrap();
                pool.close().await;
            }
            pause.release.notify_one();
            let reply = pending.await;
            assert_ne!(reply.0, StatusCode::OK);
            assert_ne!(reply.4, 0);
            assert_eq!(reply.2.restarts, 0);
            assert_eq!(
                reply.2.dispatches
                    + reply.2.ordinary_dispatches
                    + reply.2.restart_dispatches
                    + reply.2.prepared_results,
                0
            );
        }
    });
}
#[test]
fn declared_denied_source_does_not_poll_body_and_cold_unmarked_route_keeps_original_readers() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        let cold = f.send_declared().await;
        assert_eq!(cold.0, StatusCode::OK);
        assert_eq!(cold.4, 0, "cold context must use original readers");
        f.writer
            .set_setting(
                "security.inbound-allowlist.v1",
                r#"{"webmail":{"enabled":true,"cidrs":["192.0.2.1/32"]}}"#,
            )
            .await
            .unwrap();
        let polled = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let seen = polled.clone();
        let body = futures_util::stream::once(async move {
            seen.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok::<_, std::io::Error>(axum::body::Bytes::from_static(b"must not read"))
        });
        let mut request = f.declared_request();
        *request.body_mut() = Body::from_stream(body);
        let denied = f.send_declared_request(request).await;
        assert_eq!(denied.0, StatusCode::FORBIDDEN);
        assert_eq!(polled.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert_eq!(
            denied.2.dispatches
                + denied.2.ordinary_dispatches
                + denied.2.restart_dispatches
                + denied.2.prepared_results,
            0
        );
    });
}

#[test]
fn declared_initial_expiry_keeps_original_cleanup_without_application_fallback() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        f.warm_published().await;
        f.age_last_seen_for_restart(true).await;
        let denied = f.send_declared().await;
        assert_eq!(denied.0, StatusCode::UNAUTHORIZED);
        assert_ne!(denied.4, 0);
        assert_eq!(
            denied.2.dispatches
                + denied.2.ordinary_dispatches
                + denied.2.restart_dispatches
                + denied.2.prepared_results,
            0
        );
        assert!(matches!(
            f.writer.get_session(&f.session).await,
            Err(mw_store::StoreError::NotFound)
        ));
    });
}
