// Real DEFAULT36 router cases; no policy/auth/source getter mocks.
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
        assert_eq!(reply.2.prepared_results,0);assert_eq!(reply.2.dispatches+reply.2.ordinary_dispatches,0);
    });
}
impl Fixture {
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
