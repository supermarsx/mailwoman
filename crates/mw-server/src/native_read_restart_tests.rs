// Real-router regression: no wall-clock deadline sleep, no auth/store mocks.
impl Fixture {
    async fn send_restart_mode(&self, legacy: bool) -> (StatusCode, Value, test_control::Counts) {
        let future = self.router.clone().oneshot(self.request());
        let (reply, counts) = if legacy {
            test_control::count_legacy_late(future).await
        } else {
            test_control::count(false, future).await
        };
        let reply = reply.unwrap();
        let status = reply.status();
        let bytes = to_bytes(reply.into_body(), 4 * 1024 * 1024).await.unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
            counts,
        )
    }
    async fn age_last_seen_for_restart(&self, expire_activity: bool) {
        let options = mw_store::encryption::connect_options(&self.db, self.db_key.as_ref(), false);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        let sql = if expire_activity {
            "UPDATE sessions SET last_seen='2000-01-01T00:00:00Z',activity_at='2000-01-01T00:00:00Z' WHERE id=?1"
        } else {
            "UPDATE sessions SET last_seen='2000-01-01T00:00:00Z' WHERE id=?1"
        };
        sqlx::query(sql)
            .bind(&self.session)
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;
    }
}
async fn queued_restart(
    f: &Fixture,
    legacy: bool,
    expire: bool,
    deny_protocol: bool,
) -> (StatusCode, Value, test_control::Counts) {
    // Set up a fresh telemetry timestamp explicitly. Fixture construction and
    // earlier oracle requests are deliberately not a wall-clock guarantee that
    // the 15-second touch threshold is still clear when this race starts.
    f.writer.touch_session(&f.session).await.unwrap();
    // A 200 alone could be the original path. Prove actual native eligibility
    // before deliberately holding the permit that original fallback also needs.
    for attempt in 0..2 {
        let warm = f.send(false).await;
        assert_eq!(warm.0, StatusCode::OK);
        if attempt == 1 {
            assert_eq!(warm.2.dispatches, 1, "warm request must actually use native admission: {:?}", warm.2);
        }
    }
    f.writer.touch_session(&f.session).await.unwrap();
    let revision = f.measured.settings_read_revision().await.unwrap();
    let knobs =
        crate::rate_leeway::knobs_at_revision(&f.measured, "native-composite.test", &revision)
            .await;
    let held = crate::jmap_limits::request_with_knobs(&f.account, knobs)
        .await
        .unwrap();
    let pause = test_control::pause(test_control::Stage::AfterPrepared, &f.session);
    let pending = f.send_restart_mode(legacy);
    tokio::pin!(pending);
    tokio::select! {_=pause.entered.notified()=>{},reply=&mut pending=>panic!("native read did not reach initial preparation: status={}, counts={:?}, problem_type={:?}, problem_code={:?}",reply.0,reply.2,reply.1.get("type"),reply.1.get("code"))}
    pause.release.notify_one();
    std::future::poll_fn(|cx| match pending.as_mut().poll(cx) {
        std::task::Poll::Pending => std::task::Poll::Ready(()),
        std::task::Poll::Ready(_) => panic!("must actually await the occupied JMAP permit"),
    })
    .await;
    f.age_last_seen_for_restart(expire).await;
    if deny_protocol {
        policy(&f.writer, "access.protocols", json!(["pop3"])).await;
    }
    drop(held);
    tokio::time::timeout(Duration::from_secs(5), pending)
        .await
        .expect("bounded read restart")
}
#[test]
fn due_touch_after_queued_preparation_reproduces_legacy503_and_fresh_restart200() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        let old = queued_restart(&f, true, false, false).await;
        assert_eq!(old.0, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(old.1["code"], "authorizationNotConfirmedBeforeDispatch");
        assert_eq!(old.1["notDispatched"], true);
        assert_eq!(old.2.dispatches + old.2.restart_dispatches, 0);
        let new = queued_restart(&f, false, false, false).await;
        assert_eq!(new.0, StatusCode::OK);
        assert_eq!(new.2.restarts, 1);
        assert_eq!(new.2.dispatches, 0);
        assert_eq!(
            new.2.restart_dispatches, 1,
            "exactly one Engine dispatch, through the complete original chain"
        );
        assert_eq!(new.1["methodResponses"][0][0], "Email/get");
        assert_eq!(
            new.1["methodResponses"][0][1]["list"]
                .as_array()
                .unwrap()
                .len(),
            50
        );
    });
}
#[test]
fn read_restart_recomputes_previously_absent_protocol_policy() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        f.writer
            .delete_setting_policy("access.protocols")
            .await
            .unwrap();
        // The first guard may prune protocol while it is genuinely absent.
        let denied = queued_restart(&f, false, false, true).await;
        assert_eq!(denied.0, StatusCode::FORBIDDEN);
        assert_eq!(denied.2.restarts, 1);
        assert_eq!(denied.2.dispatches + denied.2.restart_dispatches, 0);
        assert!(denied.1.get("methodResponses").is_none());
    });
}
#[test]
fn read_restart_does_not_resurrect_expired_persisted_activity() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        let denied = queued_restart(&f, false, true, false).await;
        assert_eq!(denied.0, StatusCode::UNAUTHORIZED);
        assert_eq!(denied.2.dispatches + denied.2.restart_dispatches, 0);
        assert!(matches!(
            f.writer.get_session(&f.session).await,
            Err(mw_store::StoreError::NotFound)
        ));
    });
}
#[test]
fn second_change_during_restarted_chain_is_refused_without_another_restart() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        let last = test_control::pause(test_control::Stage::RestartBeforeFinal, &f.session);
        let pending = queued_restart(&f, false, false, false);
        tokio::pin!(pending);
        tokio::select! {_=last.entered.notified()=>{},reply=&mut pending=>panic!("did not reach restarted final observation: {}",reply.0)}
        // This legacy flag is deliberately outside the narrow primary tuple;
        // only the LAST full composite can preserve its late revocation check.
        f.writer
            .set_setting(
                &format!("v6:admin:flags:{}", f.account),
                r#"{"disabled":true}"#,
            )
            .await
            .unwrap();
        last.release.notify_one();
        let denied = pending.await;
        assert_eq!(denied.0, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(denied.1["notDispatched"], true);
        assert_eq!(denied.2.restarts, 1);
        assert_eq!(denied.2.dispatches + denied.2.restart_dispatches, 0);
    });
}
