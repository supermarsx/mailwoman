// Real DEFAULT36 router cases; no policy/auth/source getter mocks.
#[test]
fn warm_primary_hint_is_fused_into_fresh_first_fence_with_exact_one_read_removed(){
    run_fixture(||async {
        let fixture=Fixture::new().await;
        let cold=fixture.send(false).await;
        assert_eq!(cold.0,StatusCode::OK);assert_eq!(cold.2.hints,1,"original unpublished cold shaping still uses its full hint");
        fixture.warm_published().await;
        let ((legacy,legacy_counts),legacy_queries)=mw_store::pool_telemetry::measure_queries(
            test_control::count_published_legacy_hint(fixture.router.clone().oneshot(fixture.request()))
        ).await;
        let legacy=legacy.unwrap();assert_eq!(legacy.status(),StatusCode::OK);
        let legacy_body:Value=serde_json::from_slice(&to_bytes(legacy.into_body(),4*1024*1024).await.unwrap()).unwrap();
        assert_eq!(legacy_counts.hints,1);assert_eq!(legacy_counts.fences,2);assert_eq!(legacy_counts.observations,0);
        let fused=fixture.send_published().await;
        assert_eq!(fused.0,StatusCode::OK);assert_eq!(fused.1,legacy_body);assert_eq!(fused.1,cold.1);
        assert_eq!(fused.2.hints,0);assert_eq!(fused.2.fences,2);assert_eq!(fused.2.observations,0);
        assert_eq!(fused.2.dispatches,legacy_counts.dispatches);
        assert_eq!(legacy_queries.read.calls+legacy_queries.other.calls,fused.3.read.calls+fused.3.other.calls+1,"one actual primary hint statement removed on warm path; no wholeHTTP0SQL claim");
        assert_eq!(legacy_queries.write.calls,fused.3.write.calls);
        eprintln!("{}",json!({"kind":"warm-native-primary-hint-fusion","legacyHintReads":legacy_queries.read.calls,"fusedReads":fused.3.read.calls,"legacyHints":legacy_counts.hints,"fusedHints":fused.2.hints,"freshFences":fused.2.fences,"workerQueriesIncluded":false,"realPeerGuardsIncluded":false}));
    });
}
#[test]
fn prepared_default36_emits_actual_fifty_rows_without_engine_or_index_replay() {
    run_fixture(||async {
        let fixture=Fixture::new().await;
        policy(&fixture.writer,"performance.preparedMail",serde_json::to_value(mw_engine::PreparedMailOptions {enabled:true,..Default::default()}).unwrap()).await;
        let original=fixture.send(true).await;
        let ready=fixture.warm_prepared_mail().await;
        assert_eq!(ready.1,original.1);
        let index=fixture.engine.search();
        let before=(index.list_projection_search_count(),index.list_projection_doc_read_count(),index.doc_fetch_count());
        let hit=fixture.send_published().await;
        assert_eq!(hit.0,StatusCode::OK);assert_eq!(hit.1,original.1);
        assert_eq!(hit.2.prepared_results,1);assert_eq!(hit.2.dispatches+hit.2.ordinary_dispatches,0);
        assert_eq!(before,(index.list_projection_search_count(),index.list_projection_doc_read_count(),index.doc_fetch_count()),"actual ready route must not repeat Index queries or document fetches");
        assert_eq!(hit.1["methodResponses"][0][1]["list"].as_array().unwrap().len(),50);
        assert!(hit.3.read.calls+hit.3.other.calls<original.3.read.calls+original.3.other.calls);
        eprintln!("{}",json!({"kind":"prepared-native-whole-router-query-counts","originalReads":original.3.read.calls,"originalOther":original.3.other.calls,"readyReads":hit.3.read.calls,"readyOther":hit.3.other.calls,"readyWrites":hit.3.write.calls,"readyAcquires":hit.3.acquire.calls,"preparedResults":hit.2.prepared_results,"engineDispatches":hit.2.dispatches+hit.2.ordinary_dispatches,"workerQueriesIncluded":false}));
        // The ordinary dispatcher preserves outer createdIds, which is not in
        // the narrow raw envelope. It must remain on that path rather than
        // silently lose a valid part of the request/response contract.
        let mut request=fixture.request();
        let created=json!({"prior-client-id":fixture.ids[0]});
        *request.body_mut()=Body::from(serde_json::to_vec(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"createdIds":created,"methodCalls":[["Email/get",{"accountId":fixture.account,"ids":fixture.ids,"properties":default_properties()},"outer-map"]]})).unwrap());
        let (reply,counts)=test_control::count_published(fixture.router.clone().oneshot(request)).await;
        let reply=reply.unwrap();assert_eq!(reply.status(),StatusCode::OK);
        let body=to_bytes(reply.into_body(),4*1024*1024).await.unwrap();
        let value:Value=serde_json::from_slice(&body).unwrap();assert_eq!(value["createdIds"],created);
        assert_eq!(counts.prepared_results,0);assert_eq!(counts.dispatches,1);
        fixture.remove_prepared_account_source_raw().await;
        let missing=fixture.send_published().await;
        assert_eq!(missing.0,StatusCode::OK);assert_eq!(missing.2.prepared_results,0);assert_eq!(missing.2.dispatches,1);
        assert_eq!(missing.1,original.1,"missing derived DATA must preserve the ordinary response while rebuilding is deferred");
        assert_eq!(fixture.engine.prepared_mail_status().entries,Some(0),"missing final DATA must eject the old candidate rather than trap every later read in fallback");
        assert_eq!(fixture.warm_prepared_mail().await.1,original.1,"a subsequent cold request may seed owned derived DATA and rebuild normally");
    });
}
#[test]
fn prepared_default36_never_emits_after_session_revocation_during_preparation() {
    run_fixture(||async {
        let fixture=Fixture::new().await;
        policy(&fixture.writer,"performance.preparedMail",serde_json::to_value(mw_engine::PreparedMailOptions {enabled:true,..Default::default()}).unwrap()).await;
        fixture.warm_prepared_mail().await;
        let pause=test_control::pause(test_control::Stage::BeforeConnector,&fixture.session);
        let pending=fixture.send_published();tokio::pin!(pending);
        tokio::select! {_=pause.entered.notified()=>{},reply=&mut pending=>panic!("request completed before final proof: {}",reply.0)}
        fixture.writer.delete_session(&fixture.session).await.unwrap();
        pause.release.notify_one();
        let reply=pending.await;
        assert_ne!(reply.0,StatusCode::OK);
        assert_eq!(reply.2.hints,0,"warm fused preparation still waits for final session revocation proof");
        assert_eq!(reply.2.prepared_results,0);assert_eq!(reply.2.dispatches+reply.2.ordinary_dispatches,0);
    });
}
impl Fixture {
    async fn remove_prepared_account_source_raw(&self) {
        let options=mw_store::encryption::connect_options(&self.db,self.db_key.as_ref(),false);
        let pool=sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
        assert_eq!(sqlx::query("DELETE FROM mail_data_accounts WHERE account_id=?1").bind(&self.account).execute(&pool).await.unwrap().rows_affected(),1);
        pool.close().await;
    }
    async fn warm_prepared_mail(&self)->(StatusCode,Value,test_control::Counts,mw_store::pool_telemetry::QuerySnapshot) {
        for _ in 0..200 {
            let reply=self.send_published().await;assert_eq!(reply.0,StatusCode::OK);
            if reply.2.prepared_results==1{return reply;}
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("actual DEFAULT36 prepared emission did not qualify: {}",serde_json::to_string(&self.engine.prepared_mail_status()).unwrap());
    }
    async fn send_published(
        &self,
    ) -> (
        StatusCode,
        Value,
        test_control::Counts,
        mw_store::pool_telemetry::QuerySnapshot,
    ) {
        let ((reply, counts), sql) = mw_store::pool_telemetry::measure_queries(
            test_control::count_published(self.router.clone().oneshot(self.request())),
        )
        .await;
        let reply = reply.unwrap();
        let status = reply.status();
        let bytes = to_bytes(reply.into_body(), 4 * 1024 * 1024).await.unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
            counts,
            sql,
        )
    }
    async fn warm_published(
        &self,
    ) -> (
        StatusCode,
        Value,
        test_control::Counts,
        mw_store::pool_telemetry::QuerySnapshot,
    ) {
        for _ in 0..200 {
            let reply = self.send_published().await;
            assert_eq!(reply.0, StatusCode::OK);
            if reply.2.fences == 2 && reply.2.observations == 0 {
                return reply;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        let worker = publication::Worker::start(self.measured.clone()).unwrap();
        panic!(
            "published DEFAULT36 candidate never used; aggregate worker counters={:?}",
            worker.statistics()
        );
    }
    async fn expire_activity_raw(&self) {
        let options = mw_store::encryption::connect_options(&self.db, self.db_key.as_ref(), false);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        sqlx::query("UPDATE sessions SET activity_at='2000-01-01T00:00:00Z' WHERE id=?1")
            .bind(&self.session)
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;
    }
}
#[test]
fn actual_default36_uses_published_inputs_two_fences_and_matches_original() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let original = fixture.send(true).await;
        let published = fixture.warm_published().await;
        assert_eq!(published.2.dispatches, 1);
        eprintln!(
            "{}",
            json!({
                "kind":"published-native-whole-router-query-counts",
                "original":{"reads":original.3.read.calls,"writes":original.3.write.calls,"other":original.3.other.calls,"acquires":original.3.acquire.calls},
                "ready":{"reads":published.3.read.calls,"writes":published.3.write.calls,"other":published.3.other.calls,"acquires":published.3.acquire.calls,"fences":published.2.fences,"observations":published.2.observations,"dispatches":published.2.dispatches},
                "workerQueriesIncluded":false
            })
        );
        assert_eq!(
            published.2.observations, 0,
            "no request-driven full composite on ready hit"
        );
        assert_eq!(
            published.2.fences, 2,
            "pre-permit and final fresh fences both remain"
        );
        assert_eq!(published.1, original.1);
        assert_eq!(
            published.1["methodResponses"][0][1]["list"]
                .as_array()
                .unwrap()
                .len(),
            50
        );
        assert!(
            published.3.read.calls < original.3.read.calls,
            "whole router reads must improve versus complete original auth, not merely handler subtotal: published={:?},original={:?}",
            published.3,
            original.3
        );
        let a = publication::Worker::start(fixture.measured.clone()).unwrap();
        let b = publication::Worker::start(fixture.measured.clone()).unwrap();
        assert!(Arc::ptr_eq(&a, &b), "one owner per shared Store");
    });
}
#[test]
fn published_inputs_final_fence_rejects_independent_flag_change_while_queued() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        fixture.warm_published().await;
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
        let request = fixture.send_published();
        tokio::pin!(request);
        tokio::select! {_=pause.entered.notified()=>{},reply=&mut request=>panic!("request did not reach preparation: {}",reply.0)}
        pause.release.notify_one();
        std::future::poll_fn(|cx| match request.as_mut().poll(cx) {
            std::task::Poll::Pending => std::task::Poll::Ready(()),
            std::task::Poll::Ready(_) => panic!("actual permit wait required"),
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
        let reply = request.await;
        assert_ne!(reply.0, StatusCode::OK);
        assert_eq!(reply.2.dispatches, 0);
        assert_eq!(reply.2.fences, 2);
        assert_ne!(reply.1["methodResponses"][0][0], "Email/get");
    });
}
#[test]
fn published_inputs_fresh_clock_expiry_is_not_hidden_by_unchanged_epoch() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        fixture.warm_published().await;
        let pause = test_control::pause(test_control::Stage::BeforeConnector, &fixture.session);
        let request = fixture.send_published();
        tokio::pin!(request);
        tokio::select! {_=pause.entered.notified()=>{},reply=&mut request=>panic!("request did not reach connector: {}",reply.0)}
        let before = fixture
            .writer
            .native_authority_fence(&fixture.session, &["native-composite.test".into()])
            .await
            .unwrap()
            .unwrap();
        fixture.expire_activity_raw().await;
        let after = fixture
            .writer
            .native_authority_fence(&fixture.session, &["native-composite.test".into()])
            .await
            .unwrap()
            .unwrap();
        assert!(
            before.same_published_source(&after).unwrap(),
            "hot clock write must not rebuild immutable rule inputs"
        );
        assert_ne!(before.clocks(), after.clocks());
        pause.release.notify_one();
        let reply = request.await;
        assert_ne!(reply.0, StatusCode::OK);
        assert_eq!(reply.2.hints,0,"fresh activity expiry is still checked on the fused warm path");
        assert_eq!(reply.2.dispatches, 0);
        assert_eq!(reply.2.fences, 2);
    });
}
#[test]
fn invalid_cookie_cannot_queue_publication_and_cold_builder_failure_falls_back() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let worker = publication::Worker::start(fixture.measured.clone()).unwrap();
        let before = worker.statistics();
        for _ in 0..4 {
            let mut request = fixture.request();
            request.headers_mut().insert(
                "cookie",
                "mw_session=not-a-valid-fixture-cookie".parse().unwrap(),
            );
            let reply =
                test_control::count_published(fixture.router.clone().oneshot(request)).await;
            assert_ne!(reply.0.unwrap().status(), StatusCode::OK);
        }
        assert_eq!(
            worker.statistics().0,
            before.0,
            "unauthenticated hints must not start build work"
        );
        // Bounded publication excludes oversized unrelated global policy data;
        // ordinary resolution of supported requested keys still works unchanged.
        policy(
            &fixture.writer,
            "branding.name",
            json!("x".repeat(3 * 1024 * 1024)),
        )
        .await;
        // Exercise the real cold builder, not a synthetic failed result. The
        // fixed policy payload bound must reject before any shared installation.
        assert!(matches!(
            worker.build_once_for_test(&fixture.measured, &fixture.session, &fixture.account).await,
            Err(mw_store::StoreError::Invalid("publication settings bound or shape"))
        ));
        assert_eq!(worker.statistics().1, 0, "oversized DATA must not install");
        assert_eq!(worker.statistics().3, 0);
        let reply = fixture.send_published().await;
        assert_eq!(reply.0, StatusCode::OK);
        assert_eq!(reply.1["methodResponses"][0][0], "Email/get");
        assert_eq!(reply.1["methodResponses"][0][1]["list"].as_array().unwrap().len(), 50);
        // observations counts only successful full composites: ordinary auth or
        // an early eligibility fallback need not complete two such observations.
        // Instead prove the actual safe outcome: no published hit, one Engine
        // dispatch through either original or legacy-native authentication.
        assert_eq!(reply.2.fences, 0);
        assert_eq!(reply.2.dispatches + reply.2.ordinary_dispatches, 1);
        assert_eq!(worker.statistics().1, 0);
        assert_eq!(worker.statistics().3, 0);
        println!("{}", json!({"kind":"published-cold-bound-fallback", "observations":reply.2.observations, "fences":reply.2.fences, "nativeDispatches":reply.2.dispatches, "ordinaryDispatches":reply.2.ordinary_dispatches, "installed":worker.statistics().1}));
    });
}

#[test]
fn published_ready_entry_cannot_survive_missing_invalidation_trigger() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        fixture.warm_published().await;
        let options =
            mw_store::encryption::connect_options(&fixture.db, fixture.db_key.as_ref(), false);
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
        let fallback = fixture.send_published().await;
        assert_eq!(fallback.0, StatusCode::OK);
        assert_eq!(fallback.1["methodResponses"][0][0], "Email/get");
        assert!(
            fallback.2.observations >= 2,
            "changed schema must use fresh original source, not the installed DATA"
        );
        assert!(
            fallback.2.fences <= 1,
            "missing required trigger cannot qualify a two-fence publication hit"
        );
        fixture
            .writer
            .set_setting(
                &format!("v6:admin:flags:{}", fixture.account),
                r#"{"disabled":true}"#,
            )
            .await
            .unwrap();
        let denied = fixture.send_published().await;
        assert_ne!(denied.0, StatusCode::OK);
        assert_eq!(denied.2.dispatches + denied.2.restart_dispatches, 0);
    });
}

#[test]
fn published_valid_sibling_name_set_preserves_original_admission_after_telemetry_reorders_it() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let credentials = Credentials {
            username: "sibling@native-composite.test".into(),
            password: "owned-fixture-sibling".into(),
        };
        let sibling = fixture
            .writer
            .create_session(
                &fixture.account,
                &credentials.username,
                "engine",
                "engine",
                &credentials,
            )
            .await
            .unwrap();
        let options =
            mw_store::encryption::connect_options(&fixture.db, fixture.db_key.as_ref(), false);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        sqlx::query("UPDATE sessions SET last_seen='2000-01-01T00:00:00Z' WHERE id=?1")
            .bind(&sibling)
            .execute(&pool)
            .await
            .unwrap();
        fixture.warm_published().await;
        let before = fixture
            .writer
            .native_authority_fence(&fixture.session, &["native-composite.test".into()])
            .await
            .unwrap()
            .unwrap();
        sqlx::query("UPDATE sessions SET last_seen='2099-01-01T00:00:00Z' WHERE id=?1")
            .bind(&sibling)
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;
        let after = fixture
            .writer
            .native_authority_fence(&fixture.session, &["native-composite.test".into()])
            .await
            .unwrap()
            .unwrap();
        assert!(
            before.same_published_source(&after).unwrap(),
            "sibling telemetry alone does not dirty authority inputs"
        );
        let published = fixture.send_published().await;
        let original = fixture.send(true).await;
        assert_eq!(published.0, StatusCode::OK);
        assert_eq!(original.0, StatusCode::OK);
        assert_eq!(published.1, original.1);
        assert_eq!(published.2.fences, 2);
        assert_eq!(published.2.observations, 0);
        assert_eq!(published.2.dispatches, 1);
    });
}

#[test]
fn independent_writer_during_worker_build_prevents_installing_old_inputs() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let worker = publication::Worker::start(fixture.measured.clone()).unwrap();
        let pause = test_control::pause(
            test_control::Stage::PublicationBeforeFinal,
            &fixture.session,
        );
        let build =
            worker.build_once_for_test(&fixture.measured, &fixture.session, &fixture.account);
        tokio::pin!(build);
        tokio::select! {
            _ = pause.entered.notified() => {},
            _ = &mut build => panic!("worker did not reach actual after-inputs fence"),
        }
        fixture
            .writer
            .set_setting(
                &format!("v6:admin:flags:{}", fixture.account),
                r#"{"disabled":true}"#,
            )
            .await
            .unwrap();
        pause.release.notify_one();
        assert!(
            build.await.unwrap().is_none(),
            "changed source must reject this build before any shared component installation"
        );
        assert_eq!(worker.statistics().3, 0);
        let denied = fixture.send(true).await;
        assert_ne!(denied.0, StatusCode::OK);
        assert_eq!(denied.2.dispatches + denied.2.restart_dispatches, 0);
    });
}

// ---------------------------------------------------------------------------
// t34-e6: bounded-staleness guard snapshot (`MW_GUARD_SNAPSHOT=bounded`).
// Actual router, real Store/EpochMirror; only the process mode is scoped to
// each request through the snapshot test control.
// ---------------------------------------------------------------------------
mod guard_snapshot_cases {
    use super::*;
    use crate::request_inputs::snapshot::{Mode, test_control as snap};
    use crate::request_inputs::test_control as inputs;

    pub(super) struct Sent {
        pub status: StatusCode,
        pub body: Value,
        pub inputs: inputs::Counts,
        pub snap: snap::Counts,
        pub sql: mw_store::pool_telemetry::QuerySnapshot,
    }
    pub(super) const OFF: Mode = Mode::Off;
    pub(super) const SHORT: Mode = Mode::Bounded(Duration::from_millis(100));
    pub(super) const LONG: Mode = Mode::Bounded(Duration::from_millis(5000));

    pub(super) fn me(f: &Fixture) -> Request<Body> {
        let mut request = f.request();
        *request.method_mut() = axum::http::Method::GET;
        *request.uri_mut() = "/api/me".parse().unwrap();
        *request.body_mut() = Body::empty();
        peer(request)
    }
    pub(super) fn mail(f: &Fixture) -> Request<Body> {
        peer(f.request())
    }
    pub(super) fn logout(f: &Fixture) -> Request<Body> {
        let mut request = f.request();
        *request.uri_mut() = "/api/logout".parse().unwrap();
        *request.body_mut() = Body::empty();
        peer(request)
    }
    fn peer(mut request: Request<Body>) -> Request<Body> {
        request.extensions_mut().insert(axum::extract::ConnectInfo(
            "127.0.0.1:48219".parse::<std::net::SocketAddr>().unwrap()));
        request.headers_mut().insert("user-agent", "Mozilla/5.0".parse().unwrap());
        request
    }
    /// Ordinary published request inputs ON, native metadata dispatcher OFF
    /// (ordinary handlers), as the existing request-input oracle does.
    pub(super) async fn send(f: &Fixture, request: Request<Body>, mode: Mode) -> Sent {
        let measured = snap::measure(snap::with_mode(mode,
            inputs::measure(true, test_control::count(true, f.router.clone().oneshot(request)))));
        let ((((result, _native), inputs), snap), sql) =
            mw_store::pool_telemetry::measure_queries(measured).await;
        let response = result.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 4 * 1024 * 1024).await.unwrap();
        Sent { status, body: serde_json::from_slice(&bytes).unwrap_or(Value::Null), inputs, snap, sql }
    }
    /// Until the worker published this session's request inputs.
    pub(super) async fn published(f: &Fixture) -> Sent {
        for _ in 0..400 {
            let reply = send(f, mail(f), OFF).await;
            assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
            if reply.inputs.first_fences == 1 { return reply; }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("ordinary request inputs never published");
    }
    pub(super) fn mirror(f: &Fixture) -> Option<Arc<mw_store::authority_publication::mirror::EpochMirror>> {
        publication::Worker::start(f.measured.clone()).unwrap().existing_guard_mirror()
    }
    pub(super) async fn fresh(mirror: &mw_store::authority_publication::mirror::EpochMirror) {
        tokio::time::timeout(Duration::from_secs(10), async {
            while !mirror.is_fresh() { tokio::time::sleep(Duration::from_millis(5)).await; }
        }).await.expect("mirror completes a poll");
    }
    /// Starts the mirror (first bounded request) and waits for a fresh poll.
    pub(super) async fn started(f: &Fixture, mode: Mode) -> Arc<mw_store::authority_publication::mirror::EpochMirror> {
        let reply = send(f, mail(f), mode).await;
        assert_eq!(reply.status, StatusCode::OK);
        let mirror = mirror(f).expect("first bounded request starts the mirror");
        fresh(&mirror).await;
        mirror
    }
    /// Bounded requests until one reused a certified fence and read none.
    pub(super) async fn reusing(f: &Fixture, mode: Mode, request: impl Fn() -> Request<Body>) -> Sent {
        for _ in 0..40 {
            let reply = send(f, request(), mode).await;
            assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
            // Also wait for the ban list to be stamped: a request that ran before the
            // mirror's first complete poll loaded it without caching it.
            if reply.snap.reused > 0 && reply.snap.fresh == 0 && reply.snap.globals_reused > 0 { return reply; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        panic!("bounded mode never reused a certified fence");
    }
    pub(super) async fn raw(f: &Fixture, sql: &str, bind: &str) -> u64 {
        let options = mw_store::encryption::connect_options(&f.db, f.db_key.as_ref(), false);
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1)
            .connect_with(options).await.unwrap();
        let changed = sqlx::query(sql).bind(bind).execute(&pool).await.unwrap().rows_affected();
        pool.close().await;
        changed
    }
    pub(super) fn ago(seconds: i64) -> String {
        (chrono::Utc::now() - chrono::TimeDelta::seconds(seconds)).to_rfc3339()
    }
}
use guard_snapshot_cases as snapcase;

#[test]
fn bounded_snapshot_reuses_certified_fence_and_off_mode_counts_are_unchanged() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        f.writer.touch_session(&f.session).await.unwrap();
        snapcase::published(&f).await;
        let off_mail = snapcase::send(&f, snapcase::mail(&f), snapcase::OFF).await;
        let off_me = snapcase::send(&f, snapcase::me(&f), snapcase::OFF).await;
        assert_eq!(off_mail.inputs.first_fences, 1);
        assert_eq!(off_me.inputs.first_fences, 1);
        assert_eq!(off_mail.snap.reused + off_me.snap.reused, 0);
        assert!(snapcase::mirror(&f).is_none(), "off mode never starts a mirror");
        let mirror = snapcase::started(&f, snapcase::SHORT).await;
        let hit_mail = snapcase::reusing(&f, snapcase::SHORT, || snapcase::mail(&f)).await;
        let hit_me = snapcase::reusing(&f, snapcase::SHORT, || snapcase::me(&f)).await;
        for (hit, off) in [(&hit_mail, &off_mail), (&hit_me, &off_me)] {
            assert_eq!(hit.body, off.body, "same reply from reused DATA");
            assert_eq!((hit.inputs.first_fences, hit.inputs.refresh_fences), (0, 0));
            assert_eq!(hit.inputs.legacy_session_reads, 0);
            // Every fence the off request issued is exactly one statement, and
            // a reused parsed ban list (step 2) is the one `v6:admin:bans` read.
            assert_eq!(hit.snap.globals_reused, 1, "source bans reused");
            assert_eq!(off.sql.read.calls - hit.sql.read.calls,
                off.inputs.first_fences + off.inputs.refresh_fences + hit.snap.globals_reused,
                "bounded={:?} off={:?}", hit.sql, off.sql);
            assert_eq!(hit.sql.write.calls, off.sql.write.calls);
        }
        assert!(mirror.counters().hits > 0);
        // Off mode after bounded mode in the same process: today's path again.
        let again = snapcase::send(&f, snapcase::mail(&f), snapcase::OFF).await;
        assert_eq!((again.inputs.first_fences, again.inputs.refresh_fences),
            (off_mail.inputs.first_fences, off_mail.inputs.refresh_fences));
        assert_eq!(again.sql.read.calls, off_mail.sql.read.calls);
        assert_eq!(again.snap.reused, 0);
        eprintln!("{}", json!({"kind":"t34-guard-snapshot-whole-router-counts",
            "mail":{"offReads":off_mail.sql.read.calls,"boundedReads":hit_mail.sql.read.calls,
                "offFirst":off_mail.inputs.first_fences,"offRefresh":off_mail.inputs.refresh_fences},
            "identity":{"offReads":off_me.sql.read.calls,"boundedReads":hit_me.sql.read.calls,
                "offFirst":off_me.inputs.first_fences,"offRefresh":off_me.inputs.refresh_fences},
            "realPeerGuardsIncluded":true,"workerQueriesIncluded":false}));
    });
}

#[test]
fn bounded_snapshot_local_logout_is_refused_by_the_very_next_request() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        f.writer.touch_session(&f.session).await.unwrap();
        snapcase::published(&f).await;
        // A 5 s bound polls every 1.25 s: the expedite is what refuses at once.
        let mirror = snapcase::started(&f, snapcase::LONG).await;
        snapcase::reusing(&f, snapcase::LONG, || snapcase::me(&f)).await;
        let before = mirror.counters().expedites;
        let out = snapcase::send(&f, snapcase::logout(&f), snapcase::LONG).await;
        assert!(out.status.is_success(), "logout {}", out.status);
        assert_eq!(mirror.counters().expedites, before + 1, "state-changing request expedites");
        for request in [snapcase::me(&f), snapcase::mail(&f)] {
            let denied = snapcase::send(&f, request, snapcase::LONG).await;
            assert_eq!(denied.status, StatusCode::UNAUTHORIZED, "{}", denied.body);
            assert_eq!(denied.snap.reused, 0, "no reuse after a local revocation");
        }
    });
}

#[test]
fn bounded_snapshot_cross_connection_revocations_take_effect_within_one_poll() {
    run_fixture(|| async {
        // Writers outside this process's request path: a second Store on the
        // same file (policy, domain, flags) and raw SQL (session row). Nothing
        // here expedites; only the mirror poll can notice.
        for case in ["raw-session-delete", "protocol-policy", "domain-disable", "account-disable"] {
            let f = Fixture::new().await;
            f.writer.touch_session(&f.session).await.unwrap();
            snapcase::published(&f).await;
            let mirror = snapcase::started(&f, snapcase::SHORT).await;
            snapcase::reusing(&f, snapcase::SHORT, || snapcase::mail(&f)).await;
            match case {
                "raw-session-delete" => assert_eq!(snapcase::raw(&f, "DELETE FROM sessions WHERE id=?1", &f.session).await, 1),
                "protocol-policy" => policy(&f.writer, "access.protocols", json!(["pop3"])).await,
                "domain-disable" => f.writer.put_domain_config(&DomainConfigRow{name:"native-composite.test".into(),upstream_json:json!({"imap":{"host":"127.0.0.1","port":9,"tls":"starttls","auth":"password","allowPrivate":true}}).to_string(),allowlist_json:"[]".into(),blocklist_json:"[]".into(),enabled:false,preferred:false,settings_json:"{}".into(),updated_at:2}).await.unwrap(),
                _ => { crate::account_gate::update_flags(&f.writer, &f.account, |flags| flags.disabled = true).await.unwrap(); }
            }
            // One complete poll that started after the commit.
            mirror.poll_now().await.unwrap();
            // `access.protocols` gates mail protocols, not `GET /api/me`.
            let requests = if case == "protocol-policy" { vec![snapcase::mail(&f)] }
                else { vec![snapcase::mail(&f), snapcase::me(&f)] };
            for request in requests {
                let denied = snapcase::send(&f, request, snapcase::SHORT).await;
                assert_ne!(denied.status, StatusCode::OK, "{case}: {}", denied.body);
                assert_eq!(denied.snap.reused, 0, "{case}");
            }
        }
    });
}

#[test]
fn bounded_snapshot_raw_revocation_is_refused_after_the_bound_without_a_manual_poll() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        f.writer.touch_session(&f.session).await.unwrap();
        snapcase::published(&f).await;
        snapcase::started(&f, snapcase::SHORT).await;
        snapcase::reusing(&f, snapcase::SHORT, || snapcase::mail(&f)).await;
        assert_eq!(snapcase::raw(&f, "DELETE FROM sessions WHERE id=?1", &f.session).await, 1);
        tokio::time::sleep(Duration::from_millis(150)).await;
        let denied = snapcase::send(&f, snapcase::mail(&f), snapcase::SHORT).await;
        assert_eq!(denied.status, StatusCode::UNAUTHORIZED, "{}", denied.body);
        assert_eq!(denied.snap.reused, 0);
    });
}

#[test]
fn bounded_snapshot_overflow_and_failed_polls_take_the_fresh_path() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        f.writer.touch_session(&f.session).await.unwrap();
        snapcase::published(&f).await;
        let mirror = snapcase::started(&f, snapcase::SHORT).await;
        snapcase::reusing(&f, snapcase::SHORT, || snapcase::mail(&f)).await;
        // Overflow: 4100 changes to scopes this session does not observe. They
        // alone would not invalidate it; a full page flushes everything.
        snapcase::raw(&f, "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i+1 FROM n WHERE i<4100) INSERT INTO authority_epoch(scope_kind,scope_id) SELECT 'subject','t34-overflow-'||i||?1 FROM n", "").await;
        mirror.poll_now().await.unwrap();
        assert!(mirror.counters().overflow_flushes >= 1);
        let after = snapcase::send(&f, snapcase::mail(&f), snapcase::SHORT).await;
        assert_eq!(after.status, StatusCode::OK);
        // A flushed mirror certifies nothing stamped before it: the first fence
        // is read fresh (later boundaries may reuse it once a newer poll ran).
        assert_eq!(after.inputs.first_fences, 1);
        snapcase::fresh(&mirror).await;
        // Unrelated changes after a re-stamp do not invalidate this session.
        snapcase::raw(&f, "INSERT INTO authority_epoch(scope_kind,scope_id) VALUES('subject','t34-unrelated'||?1)", "").await;
        mirror.poll_now().await.unwrap();
        snapcase::reusing(&f, snapcase::SHORT, || snapcase::mail(&f)).await;
        // Failing polls: the change log disappears, so no poll completes and
        // after the bound nothing is fresh. Requests take the fresh path.
        snapcase::raw(&f, "DROP TABLE authority_change_log", "").await;
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(!mirror.is_fresh());
        assert!(mirror.stamp_gen().is_none());
        assert!(mirror.counters().failures >= 1);
        let stale = snapcase::send(&f, snapcase::mail(&f), snapcase::SHORT).await;
        assert_eq!(stale.snap.reused, 0, "stale mirror never certifies a reuse");
        assert_eq!(stale.status, StatusCode::OK, "{}", stale.body);
    });
}

#[test]
fn bounded_snapshot_clock_decisions_never_use_a_reused_fence() {
    run_fixture(|| async {
        // Idle 125 s: a fence whose activity is 10 s old is 115 s from its
        // deadline, inside the 120 s margin, so it is never reused.
        let f = Fixture::new().await;
        policy(&f.writer, "session.idleSecs", json!(125)).await;
        f.writer.touch_session(&f.session).await.unwrap();
        snapcase::published(&f).await;
        snapcase::raw(&f, "UPDATE sessions SET activity_at=?1", &snapcase::ago(10)).await;
        snapcase::started(&f, snapcase::SHORT).await;
        for _ in 0..3 {
            let near = snapcase::send(&f, snapcase::mail(&f), snapcase::SHORT).await;
            assert_eq!(near.status, StatusCode::OK, "{}", near.body);
            assert_eq!(near.snap.reused, 0, "inside the clock margin every check reads fresh");
        }
        // Natural expiry: the cached candidate's own clocks are past the
        // deadline as well, so the fresh path decides and cleans up.
        snapcase::raw(&f, "UPDATE sessions SET activity_at=?1", &snapcase::ago(130)).await;
        let expired = snapcase::send(&f, snapcase::mail(&f), snapcase::SHORT).await;
        assert_eq!(expired.status, StatusCode::UNAUTHORIZED, "{}", expired.body);
        assert_eq!(expired.snap.reused, 0);
        assert!(matches!(f.writer.get_session(&f.session).await, Err(mw_store::StoreError::NotFound)));
    });
}

#[test]
fn bounded_snapshot_due_touch_is_never_skipped_by_a_reused_fence() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        // The original rule, without the store's touch buffer (t50hp-e22; the
        // buffered case is the next test).
        f.measured.disable_session_touch_buffer().await.unwrap();
        f.writer.touch_session(&f.session).await.unwrap();
        snapcase::published(&f).await;
        // last_seen 14 s ago: not due (15 s) when the slot is stamped.
        snapcase::raw(&f, "UPDATE sessions SET last_seen=?1", &snapcase::ago(14)).await;
        snapcase::started(&f, snapcase::SHORT).await;
        snapcase::reusing(&f, snapcase::SHORT, || snapcase::mail(&f)).await;
        tokio::time::sleep(Duration::from_millis(1500)).await;
        // Now the stamped last_seen is due: the reuse is refused, the fresh
        // fence is due as well, and the original chain performs the touch.
        let due = snapcase::send(&f, snapcase::mail(&f), snapcase::SHORT).await;
        assert_eq!(due.status, StatusCode::OK);
        assert_eq!(due.snap.reused, 0);
        assert_eq!(due.inputs.legacy_session_reads, 1, "original auth chain ran");
        assert!(due.sql.write.calls >= 1, "due touch written");
        let next = snapcase::reusing(&f, snapcase::SHORT, || snapcase::mail(&f)).await;
        assert_eq!(next.sql.write.calls, 0);
    });
}

/// t50hp-e22: with the touch buffer the same due touch keeps the reuse; the
/// touch is recorded in memory, nothing is written by the request, and the
/// buffer's flush writes it.
#[test]
fn bounded_snapshot_due_touch_with_the_buffer_is_recorded_without_leaving_the_reuse() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        assert!(f.measured.session_touch_buffered());
        f.writer.touch_session(&f.session).await.unwrap();
        snapcase::published(&f).await;
        snapcase::raw(&f, "UPDATE sessions SET last_seen=?1", &snapcase::ago(14)).await;
        snapcase::started(&f, snapcase::SHORT).await;
        snapcase::reusing(&f, snapcase::SHORT, || snapcase::mail(&f)).await;
        // Start from no held touch younger than the throttle.
        f.measured.disable_session_touch_buffer().await.unwrap();
        assert!(crate::session_persistence::enable_touch_buffer(&f.measured));
        tokio::time::sleep(Duration::from_millis(1500)).await;
        let due = snapcase::send(&f, snapcase::mail(&f), snapcase::SHORT).await;
        assert_eq!(due.status, StatusCode::OK);
        assert!(due.snap.reused > 0, "the due touch does not refuse the reuse: {:?}", due.snap);
        assert_eq!(due.inputs.legacy_session_reads, 0, "published inputs kept");
        assert_eq!(due.sql.write.calls, 0, "the request writes nothing");
        assert!(f.measured.buffered_last_seen(&f.session).is_some(), "touch recorded in memory");
        assert_eq!(f.measured.flush_session_touches().await.unwrap(), 1, "the flush writes it");
    });
}

#[test]
fn bounded_snapshot_declared_read_reuses_authority_fences_but_keeps_final_mail_data_fresh() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        policy(&f.writer, "performance.preparedMail", serde_json::to_value(
            mw_engine::PreparedMailOptions { enabled: true, ..Default::default() }).unwrap()).await;
        f.warm_prepared_mail().await;
        f.writer.touch_session(&f.session).await.unwrap();
        use crate::request_inputs::snapshot::test_control as snap;
        let (off, off_snap) = snap::measure(snap::with_mode(snapcase::OFF, f.send_declared())).await;
        assert_eq!(off.0, StatusCode::OK);
        assert_eq!(off.2.prepared_results, 1);
        assert_eq!(off.2.fences, 2);
        assert_eq!(off_snap.fresh, 2, "guard source + first observation");
        let _ = snap::with_mode(snapcase::SHORT, f.send_declared()).await;
        let mirror = snapcase::mirror(&f).unwrap();
        snapcase::fresh(&mirror).await;
        let mut hit = None;
        for _ in 0..40 {
            let (reply, counts) = snap::measure(snap::with_mode(snapcase::SHORT, f.send_declared())).await;
            assert_eq!(reply.0, StatusCode::OK, "{}", reply.1);
            if counts.reused == 2 && counts.fresh == 0 && counts.globals_reused == 1 { hit = Some((reply, counts.globals_reused)); break; }
        }
        let (hit, hit_globals) = hit.expect("declared read reuses both authority-only fences");
        assert_eq!(hit_globals, 1, "source bans reused");
        assert_eq!(hit.1, off.1);
        assert_eq!(hit.2.prepared_results, 1);
        assert_eq!(hit.2.fences, 1, "final mail-DATA fence stays fresh");
        // At least the two authority fences and the ban read. Other bounded caches
        // (t34-e13's session-watch preferences, read here because this request has
        // no User-Agent) may remove more; the snapshot counters above are exact.
        assert!(off.3.read.calls - hit.3.read.calls >= 2 + hit_globals, "bounded={:?} off={:?}", hit.3, off.3);
        // In-process local revocation is refused by the next declared read.
        let out = snap::with_mode(snapcase::SHORT, f.router.clone().oneshot(snapcase::logout(&f))).await.unwrap();
        assert!(out.status().is_success());
        let (denied, counts) = snap::measure(snap::with_mode(snapcase::SHORT, f.send_declared())).await;
        assert_ne!(denied.0, StatusCode::OK);
        assert_eq!(counts.reused, 0);
    });
}

// Cross-process ban writers (CLI `mailwoman admin observability ban/unban`, another
// server process, raw SQL) write `v6:admin:bans` without this process's API or
// expedite. Each settings write fires `authority_settings_*` (0111), which bumps
// the `protection` global epoch; 0115 logs that epoch row and the mirror marks the
// global changed, so the cached parsed ban list is not reused after one poll.
#[test]
fn bounded_snapshot_cached_bans_follow_cross_connection_ban_and_unban() {
    run_fixture(|| async {
        for writer in ["second-store", "raw-sql"] {
            let f = Fixture::new().await;
            f.writer.touch_session(&f.session).await.unwrap();
            snapcase::published(&f).await;
            let mirror = snapcase::started(&f, snapcase::SHORT).await;
            let warm = snapcase::reusing(&f, snapcase::SHORT, || snapcase::mail(&f)).await;
            assert_eq!(warm.snap.globals_reused, 1, "{writer}: ban list cached before the write");
            let banned = json!([{"ip":"127.0.0.1","reason":"t34 cross-process","banned_at":chrono::Utc::now().to_rfc3339(),"expires_at":null}]).to_string();
            let fr = &f;
            let write = |value: String| async move {
                let f = fr;
                match writer {
                    "second-store" => f.writer.set_setting("v6:admin:bans", &value).await.unwrap(),
                    _ => {
                        let options = mw_store::encryption::connect_options(&f.db, f.db_key.as_ref(), false);
                        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
                        sqlx::query("INSERT INTO settings(key,value) VALUES('v6:admin:bans',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value")
                            .bind(&value).execute(&pool).await.unwrap();
                        pool.close().await;
                    }
                }
            };
            write(banned).await;
            // Within the bound and without a manual poll (100 ms bound, 25 ms polls).
            tokio::time::sleep(Duration::from_millis(150)).await;
            let denied = snapcase::send(&f, snapcase::mail(&f), snapcase::SHORT).await;
            assert_eq!(denied.status, StatusCode::FORBIDDEN, "{writer}: {}", denied.body);
            assert_eq!(denied.body["code"], "sourceUnavailable");
            write("[]".into()).await;
            mirror.poll_now().await.unwrap();
            let allowed = snapcase::send(&f, snapcase::mail(&f), snapcase::SHORT).await;
            assert_eq!(allowed.status, StatusCode::OK, "{writer}: unban honoured: {}", allowed.body);
            assert_eq!(allowed.snap.globals_reused, 0, "{writer}: fresh ban read after the unban");
        }
    });
}

// t34: a frozen process environment (request_inputs::freeze_environment) is the
// same input as the per-boundary scan: identical reply, statements and frames.
#[test]
fn frozen_environment_keeps_published_request_counts_and_reply() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        f.writer.touch_session(&f.session).await.unwrap();
        snapcase::published(&f).await;
        let frozen = crate::request_inputs::test_control::capture_process_environment().unwrap();
        for request in [snapcase::me as fn(&Fixture) -> Request<Body>, snapcase::mail] {
            let scanned = snapcase::send(&f, request(&f), snapcase::OFF).await;
            let reused = crate::request_inputs::test_control::with_frozen_environment(Some(frozen.clone()),
                snapcase::send(&f, request(&f), snapcase::OFF)).await;
            assert_eq!(scanned.status, StatusCode::OK);
            assert_eq!(reused.body, scanned.body);
            assert_eq!(reused.sql.read.calls, scanned.sql.read.calls);
            assert_eq!(reused.sql.write.calls, scanned.sql.write.calls);
            assert_eq!((reused.inputs.first_fences, reused.inputs.refresh_fences, reused.inputs.session_decodes),
                (scanned.inputs.first_fences, scanned.inputs.refresh_fences, scanned.inputs.session_decodes));
            assert_eq!(reused.inputs.first_fences, 1, "published inputs still used");
        }
    });
}

// t34-e9 S3: anonymous or refused state-changing requests must not expedite
// (they could otherwise keep reuse off and wake the poller back to back).
#[test]
fn bounded_snapshot_anonymous_and_refused_posts_do_not_expedite() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        f.writer.touch_session(&f.session).await.unwrap();
        snapcase::published(&f).await;
        let mirror = snapcase::started(&f, snapcase::LONG).await;
        let before = mirror.counters().expedites;
        let post = |uri: &str, cookie: Option<&str>| {
            let mut request = Request::builder().method("POST").uri(uri).header("host", "localhost")
                .header("origin", "http://localhost").header("content-type", "application/json");
            if let Some(cookie) = cookie { request = request.header("cookie", cookie); }
            request.body(Body::from("{}")).unwrap()
        };
        for (uri, cookie) in [("/nonexistent", None), ("/api/logout", None),
            ("/api/account/sessions/revoke", None), ("/nonexistent", Some("mw_session=not-a-session"))] {
            let reply = snapcase::send(&f, post(uri, cookie), snapcase::LONG).await;
            assert_eq!(mirror.counters().expedites, before, "{uri} {cookie:?} -> {}", reply.status);
        }
        // A credentialed request that is refused (unknown route) commits nothing.
        let refused = snapcase::send(&f, post("/nonexistent",
            Some(&format!("mw_session={}", f.session))), snapcase::LONG).await;
        assert!(refused.status.is_client_error(), "{}", refused.status);
        assert_eq!(mirror.counters().expedites, before);
        // The same session's real logout still expedites and is refused next.
        let out = snapcase::send(&f, snapcase::logout(&f), snapcase::LONG).await;
        assert!(out.status.is_success());
        assert_eq!(mirror.counters().expedites, before + 1);
        assert_eq!(snapcase::send(&f, snapcase::me(&f), snapcase::LONG).await.status, StatusCode::UNAUTHORIZED);
    });
}

// t34: the load fixture's process-global guard-snapshot witness proves an arm
// really ran bounded mode. Off mode adds no activity; a warm bounded request
// adds reuse. Counts only: no session, cookie, account or fence data.
#[test]
fn guard_snapshot_witness_counts_activity_only_in_bounded_mode() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        f.writer.touch_session(&f.session).await.unwrap();
        snapcase::published(&f).await;
        let read = || crate::request_inputs::snapshot::witness();
        let number = |value: &Value, key: &str| value[key].as_u64().unwrap();
        let before = read();
        assert_eq!(snapcase::send(&f, snapcase::mail(&f), snapcase::OFF).await.status, StatusCode::OK);
        let off = read();
        for key in ["polls", "hits", "misses", "expedites", "fenceReuses", "fenceFreshReads", "globalReuses"] {
            assert_eq!(number(&off, key), number(&before, key), "off mode adds no {key}");
        }
        assert_eq!(off["supported"], true);
        snapcase::started(&f, snapcase::SHORT).await;
        snapcase::reusing(&f, snapcase::SHORT, || snapcase::mail(&f)).await;
        let on = read();
        assert!(number(&on, "fenceReuses") > number(&off, "fenceReuses"));
        assert!(number(&on, "hits") > number(&off, "hits"));
        assert!(number(&on, "polls") > number(&off, "polls"));
        assert!(number(&on, "globalReuses") > number(&off, "globalReuses"));
        assert_eq!(on["live"], true);
        let text = on.to_string();
        for secret in [f.session.as_str(), f.account.as_str()] { assert!(!text.contains(secret)); }
        fn scalars_only(value: &Value) -> bool {
            match value { Value::Object(map) => map.values().all(scalars_only), Value::Array(_) => false, _ => true }
        }
        assert!(scalars_only(&on));
    });
}

// t49pg-e2: PostgreSQL. Mode off keeps today's path (the published lookup is
// never consulted). Bounded mode with a live mirror serves the same reply
// from the published entry, with a certified reuse once the mirror is fresh.
#[test]
fn postgres_bounded_native_admission_uses_published_entries_and_off_does_not() {
    use crate::request_inputs::snapshot::{pg_tests, test_control as snap};
    let Some(dsn) = pg_tests::pg_dsn() else { return };
    run_fixture(move || async move {
        let f = Fixture::new_on(Some(pg_tests::pg_schema(&dsn).await)).await;
        assert!(!f.measured.uses_sqlite());
        let original = f.send(true).await;
        assert_eq!(original.0, StatusCode::OK);
        for _ in 0..4 {
            let off = snap::with_mode(snapcase::OFF, f.send_published()).await;
            assert_eq!(off.0, StatusCode::OK, "{}", off.1);
            assert_eq!(off.1, original.1);
            assert_eq!(off.2.fences, 0, "off: no published fence on PostgreSQL");
            assert!(off.2.observations > 0, "off: the original cold observation");
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        let mut published = false;
        let mut reused = false;
        for _ in 0..400 {
            let (reply, counts) = snap::measure(snap::with_mode(snapcase::SHORT, f.send_published())).await;
            assert_eq!(reply.0, StatusCode::OK, "{}", reply.1);
            assert_eq!(reply.1, original.1, "same reply as the original path");
            if reply.2.observations == 0 && reply.2.fences + counts.reused >= 2 {
                published = true;
                reused |= counts.reused > 0;
                if reused { break; }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(published, "bounded mode used the published entry on PostgreSQL");
        assert!(reused, "a certified fence was reused on PostgreSQL");
    });
}


// ---------------------------------------------------------------------------
// t50hp-e9 (t62 N1/N4): warm statement budgets per read shape, both backends.
// Each statement is one database round trip. Budgets are upper bounds and
// only go down. "bounded" = published inputs with a live, fresh mirror
// (certified fences, no fence statement); "off" = SQLite published inputs
// without a mirror (fresh fences). PostgreSQL without the mirror takes the
// original readers; its numbers are printed, not budgeted here (see
// guard_read_budget_tests).
// ---------------------------------------------------------------------------
mod n1_read_budgets {
    use super::*;
    use crate::request_inputs::snapshot::pg_tests;

    fn get(f: &Fixture, uri: &str) -> Request<Body> {
        let mut r = snapcase::me(f);
        *r.uri_mut() = uri.parse().unwrap();
        r
    }
    fn jmap(f: &Fixture, calls: Value) -> Request<Body> {
        let mut r = snapcase::mail(f);
        *r.body_mut() = Body::from(serde_json::to_vec(&json!({
            "using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":calls})).unwrap());
        r.headers_mut().remove(axum::http::header::CONTENT_LENGTH);
        r
    }
    pub(super) fn customization(f: &Fixture) -> String {
        format!("/api/account/appearance/customization?accountId={}", f.account)
    }
    /// (route, bounded budget, off budget on SQLite).
    const BUDGETS: [(&str, u64, u64); 8] = [
        ("GET /api/me", 1, 4),
        ("GET /jmap/session", 2, 50),
        ("GET /api/settings", 2, 5),
        ("GET /api/account/preferences", 2, 5),
        ("GET customization", 3, 6),
        ("Email/changes", 2, 7),
        ("list batch", 4, 9),
        // Engine: 3 reads + 3 WITH statements (focus, verdicts); t50hp-e31/e13.
        ("Email/get ids", 6, 11),
    ];
    fn routes(f: &Fixture, state: String) -> Vec<(&'static str, Box<dyn Fn(&Fixture) -> Request<Body>>)> {
        let _ = f;
        vec![
            ("GET /api/me", Box::new(|f: &Fixture| get(f, "/api/me"))),
            ("GET /jmap/session", Box::new(|f: &Fixture| get(f, "/jmap/session"))),
            ("GET /api/settings", Box::new(|f: &Fixture| get(f, "/api/settings"))),
            ("GET /api/account/preferences", Box::new(|f: &Fixture| get(f, "/api/account/preferences"))),
            ("GET customization", Box::new(|f: &Fixture| get(f, &customization(f)))),
            ("Email/changes", Box::new(move |f: &Fixture| jmap(f, json!([["Email/changes",{"accountId":f.account,"sinceState":state,"maxChanges":100},"c"]])))),
            ("list batch", Box::new(|f: &Fixture| jmap(f, json!([
                ["Email/query",{"accountId":f.account,"limit":50},"q"],
                ["Email/get",{"accountId":f.account,"#ids":{"resultOf":"q","name":"Email/query","path":"/ids"},"properties":["id","subject","receivedAt","keywords","mailboxIds"]},"g"]])))),
            ("Email/get ids", Box::new(|f: &Fixture| snapcase::mail(f))),
        ]
    }
    fn trips(sent: &snapcase::Sent) -> u64 {
        sent.sql.read.calls + sent.sql.write.calls + sent.sql.other.calls
    }
    async fn measure(label: &str, url: Option<String>) {
        let pg = url.is_some();
        let f = Fixture::new_on(url).await;
        if !pg { snapcase::published(&f).await; }
        let state = snapcase::send(&f, jmap(&f, json!([["Email/get",{"accountId":f.account,"ids":[]},"g"]])), snapcase::OFF).await
            .body["methodResponses"][0][1]["state"].as_str().unwrap_or("0").to_owned();
        let routes = routes(&f, state);
        for (mode_label, mode) in [("off", snapcase::OFF), ("bounded", snapcase::LONG)] {
            if mode_label == "bounded" {
                snapcase::started(&f, mode).await;
            }
            for (route, make) in &routes {
                let (_, bounded, off) = BUDGETS.iter().find(|(name, ..)| name == route).copied().unwrap();
                // Warm, then the best of a few certified requests (a request
                // that raced the mirror's poll reads a fresh fence).
                let mut best: Option<snapcase::Sent> = None;
                for _ in 0..12 {
                    let sent = snapcase::send(&f, make(&f), mode).await;
                    assert!(sent.status.is_success(), "{label} {mode_label} {route}: {} {}", sent.status, sent.body);
                    let better = best.as_ref().is_none_or(|b| trips(&sent) < trips(b));
                    if better { best = Some(sent); }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                let best = best.unwrap();
                eprintln!("{}", json!({"kind":"t50hp-e9-read-budget","backend":label,"mode":mode_label,"route":route,
                    "reads":best.sql.read.calls,"writes":best.sql.write.calls,"other":best.sql.other.calls,
                    "roundTrips":trips(&best),"framed":best.inputs.frames_ready,"fresh":best.snap.fresh}));
                match (mode_label, pg) {
                    ("bounded", _) => {
                        assert_eq!(best.inputs.frames_ready, 1, "{label} {route}: published inputs");
                        assert!(trips(&best) <= bounded, "{label} bounded {route}: {} round trips > {bounded}", trips(&best));
                    }
                    ("off", false) => assert!(trips(&best) <= off, "{label} off {route}: {} round trips > {off}", trips(&best)),
                    _ => {}
                }
            }
        }
    }
    /// Every source a removed statement read is still honoured on the next
    /// request (off: at once; bounded: within the bound), through a SECOND
    /// Store handle on the same database (no in-process expedite):
    /// administrator policy (the request revision, the selected resolution),
    /// the personal appearance document (the one personal-layer statement), a
    /// lock (customization), and a domain alias (the family memo).
    async fn second_handle_changes(label: &str, url: Option<String>) {
        let pg = url.is_some();
        let f = Fixture::new_on(url).await;
        if !pg { snapcase::published(&f).await; }
        let modes: Vec<crate::request_inputs::snapshot::Mode> = if pg { vec![snapcase::SHORT] } else { vec![snapcase::OFF, snapcase::SHORT] };
        for (round, mode) in modes.into_iter().enumerate() {
            if mode != snapcase::OFF { snapcase::started(&f, mode).await; }
            let after = |mode| async move {
                if mode != snapcase::OFF { tokio::time::sleep(Duration::from_millis(250)).await; }
            };
            let framed = |sent: &snapcase::Sent, what: &str| {
                assert!(sent.status.is_success(), "{label} {what}: {} {}", sent.status, sent.body);
            };
            // 1. administrator policy -> /api/settings
            let before = snapcase::send(&f, get(&f, "/api/settings"), mode).await;
            framed(&before, "settings before");
            policy(&f.writer, "appearance.density", json!(if round == 0 { "compact" } else { "comfortable" })).await;
            after(mode).await;
            let changed = snapcase::send(&f, get(&f, "/api/settings"), mode).await;
            framed(&changed, "settings after policy");
            assert_ne!(changed.body, before.body, "{label} {mode:?}: policy change seen by the next request");
            // 2. personal appearance document -> /api/settings
            let owner = f.writer.session_storage_authority(&f.session).await.unwrap().unwrap();
            let key = crate::prefs_routes::personal_storage::key("appearance", &owner);
            let doc = json!({"prefs":{"chosen":true,"theme":format!("e9-theme-{round}"),"accentId":"","badgeColor":"default","density":"compact","font":"default"}});
            f.writer.set_setting(&key, &doc.to_string()).await.unwrap();
            after(mode).await;
            let personal = snapcase::send(&f, get(&f, "/api/settings"), mode).await;
            framed(&personal, "settings after personal");
            assert!(personal.body.to_string().contains(&format!("e9-theme-{round}")) || personal.body != changed.body,
                "{label} {mode:?}: personal document change seen: {}", personal.body);
            // 3. a lock -> customization
            let custom_before = snapcase::send(&f, get(&f, &customization(&f)), mode).await;
            framed(&custom_before, "customization before");
            // A new lock per round (lockedFields lists the held keys).
            if round == 0 {
                policy(&f.writer, "appearance.accent", json!("e9-accent")).await;
            } else {
                policy(&f.writer, "appearance.font", json!("e9-font")).await;
            }
            after(mode).await;
            let custom_after = snapcase::send(&f, get(&f, &customization(&f)), mode).await;
            framed(&custom_after, "customization after lock");
            assert_ne!(custom_after.body, custom_before.body, "{label} {mode:?}: lock seen by the next request");
            // 4. a domain alias -> the family memo
            let domain = f.writer.get_account(&f.account).await.unwrap().username.rsplit_once('@').unwrap().1.to_owned();
            let family_before = f.measured.policy_domain_family(&domain).await.unwrap();
            let alias = format!("e9-alias-{round}.example");
            f.writer.put_domain_alias(&alias, &domain).await.unwrap();
            let family_after = f.measured.policy_domain_family(&domain).await.unwrap();
            assert!(!family_before.contains(&alias) && family_after.contains(&alias),
                "{label}: alias seen through the other handle: {family_after:?}");
        }
    }
    #[test]
    fn sqlite_second_handle_changes_reach_the_next_request() {
        run_fixture(|| async { second_handle_changes("sqlite", None).await });
    }
    #[test]
    fn postgres_second_handle_changes_reach_the_next_request() {
        let Some(dsn) = pg_tests::pg_dsn() else { return };
        run_fixture(move || async move { second_handle_changes("postgres", Some(pg_tests::pg_schema(&dsn).await)).await });
    }
    /// The frame's shortcuts (storage authority, no linked context) rest on
    /// the fence: a session row turned into an internal linked one through a
    /// second handle is no longer a published primary session (SQLite raw SQL).
    #[test]
    fn sqlite_linked_internal_session_is_never_served_from_the_frame() {
        run_fixture(|| async {
            let f = Fixture::new().await;
            snapcase::published(&f).await;
            let before = snapcase::send(&f, get(&f, "/api/settings"), snapcase::OFF).await;
            assert_eq!(before.status, StatusCode::OK);
            assert_eq!(before.inputs.frames_ready, 1);
            snapcase::raw(&f, "UPDATE sessions SET linked_internal=2 WHERE id=?1", &f.session).await;
            let after = snapcase::send(&f, get(&f, "/api/settings"), snapcase::OFF).await;
            assert_eq!(after.inputs.frames_ready, 0, "no frame for a non-primary row");
            assert_ne!(after.status, StatusCode::OK, "{}", after.body);
        });
    }

    #[test]
    fn sqlite_warm_read_budgets() {
        run_fixture(|| async { measure("sqlite", None).await });
    }
    #[test]
    fn postgres_warm_read_budgets() {
        let Some(dsn) = pg_tests::pg_dsn() else { return };
        run_fixture(move || async move { measure("postgres", Some(pg_tests::pg_schema(&dsn).await)).await });
    }
}
