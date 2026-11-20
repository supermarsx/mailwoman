// Included in email_get_batch_tests: real account-scoped Store and SQL recorder.
fn header_batch_args(ids: &[String]) -> Value {
    json!({"ids":ids,"properties":["id","subject"],"mailwomanBodyUse":false})
}
fn attach_header_cache(f: &Fixture) -> mw_cache::Cache {
    let cache = mw_cache::Cache::default();
    f.engine
        .attach_v6(crate::v6::V6Hooks::new().with_cache(cache.clone()));
    cache
}

#[tokio::test]
async fn cached_header_pages_batch_cold_and_partial_misses_not_full_reader_values() {
    for count in [5, 50] {
        let f = Fixture::with(count).await;
        attach_header_cache(&f);
        let args = header_batch_args(&f.ids);
        let (cold, reads) = counted(&f.threads, f.engine.email_get(&f.account, &args)).await;
        assert_eq!(cold["list"].as_array().unwrap().len(), count);
        assert_eq!(
            reads.against("message_list_envelope"),
            1,
            "one physical miss batch, not N: {reads:#?}"
        );
        assert_eq!(reads.against("bodies"), 0);
        assert!(reads.matching("envelope_json").is_empty());
        let (warm, reads) = counted(&f.threads, f.engine.email_get(&f.account, &args)).await;
        assert_eq!(warm, cold);
        assert_eq!(
            reads.against("message_list_envelope"),
            0,
            "warm hit still starts from scoped ownership rows: {reads:#?}"
        );
        f.store().set_flags(&f.ids[0], "[]").await.unwrap();
        let (changed, reads) = counted(&f.threads, f.engine.email_get(&f.account, &args)).await;
        assert_eq!(changed["list"].as_array().unwrap().len(), count);
        assert_eq!(
            reads.against("message_list_envelope"),
            1,
            "one miss among hits remains a single scoped batch: {reads:#?}"
        );
    }
}

#[tokio::test]
async fn header_batch_late_revision_mismatch_does_not_partially_populate_earlier_chunks() {
    let f = Fixture::with(65).await;
    attach_header_cache(&f);
    let rows = f
        .store()
        .get_account_messages(&f.account, &f.ids)
        .await
        .unwrap();
    f.store().set_flags(&f.ids[64], "[]").await.unwrap();
    assert!(
        f.engine
            .cached_header_envelopes_with_posture(
                &f.account,
                &rows,
                crate::v6::AccountPosture::Standard
            )
            .await
            .is_err()
    );
    let (result, reads) = counted(
        &f.threads,
        f.engine.cached_header_envelopes_with_posture(
            &f.account,
            &rows[..1],
            crate::v6::AccountPosture::Standard,
        ),
    )
    .await;
    assert!(result.unwrap()[0].is_some());
    assert_eq!(
        reads.against("message_list_envelope"),
        1,
        "valid earlier chunk must not have entered cache before later revision failure: {reads:#?}"
    );
    let current = f
        .store()
        .get_account_messages(&f.account, &f.ids)
        .await
        .unwrap();
    assert_eq!(
        f.engine
            .cached_header_envelopes_with_posture(
                &f.account,
                &current,
                crate::v6::AccountPosture::Standard
            )
            .await
            .unwrap()
            .len(),
        65
    );
}

struct HeaderBatchZero;
impl crate::v6::AccountPostureSource for HeaderBatchZero {
    fn posture(&self, _: &str) -> crate::v6::AccountPosture {
        crate::v6::AccountPosture::ZeroAccess
    }
}
#[tokio::test]
async fn header_batch_zero_access_ignores_warm_shared_entries_and_foreign_ids_cost_no_lookup() {
    let f = Fixture::with(5).await;
    let cache = attach_header_cache(&f);
    let args = header_batch_args(&f.ids);
    f.engine.email_get(&f.account, &args).await;
    f.engine.attach_v6(
        crate::v6::V6Hooks::new()
            .with_cache(cache)
            .with_posture_source(std::sync::Arc::new(HeaderBatchZero)),
    );
    for _ in 0..2 {
        let (response, reads) = counted(&f.threads, f.engine.email_get(&f.account, &args)).await;
        assert_eq!(response["list"].as_array().unwrap().len(), 5);
        assert_eq!(
            reads.against("message_list_envelope"),
            1,
            "live zero-access posture must bypass old warm cache: {reads:#?}"
        );
    }
    let (foreign, reads) = counted(&f.threads, f.engine.email_get("foreign-account", &args)).await;
    assert_eq!(foreign["list"], json!([]));
    assert_eq!(foreign["notFound"], json!(f.ids));
    assert_eq!(reads.against("message_list_envelope"), 0);
    let owned = f
        .store()
        .get_account_messages(&f.account, &f.ids[..1])
        .await
        .unwrap();
    assert!(
        f.engine
            .cached_header_envelopes_with_posture(
                "foreign-account",
                &owned,
                crate::v6::AccountPosture::Standard
            )
            .await
            .is_err()
    );
}

#[tokio::test]
async fn header_batch_preserves_duplicate_positions_and_uncacheable_legacy_fallback() {
    let f = Fixture::with(2).await;
    attach_header_cache(&f);
    let ids = vec![
        f.ids[1].clone(),
        f.ids[0].clone(),
        f.ids[1].clone(),
        "missing".into(),
    ];
    let response = f
        .engine
        .email_get(&f.account, &header_batch_args(&ids))
        .await;
    let list = response["list"].as_array().unwrap();
    assert_eq!(
        list.iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![f.ids[1].as_str(), f.ids[0].as_str(), f.ids[1].as_str()]
    );
    assert_eq!(response["notFound"], json!(["missing"]));
    let huge = "s".repeat(140 * 1024);
    seed_legacy_projection(
        &f,
        None,
        &serde_json::to_vec(&json!({"subject":huge,"hasAttachment":false})).unwrap(),
    )
    .await;
    let args = header_batch_args(&f.ids[..1]);
    for _ in 0..2 {
        let (response, reads) = counted(&f.threads, f.engine.email_get(&f.account, &args)).await;
        assert_eq!(
            response["list"][0]["subject"].as_str().unwrap().len(),
            140 * 1024
        );
        assert_eq!(reads.against("message_list_envelope"), 1);
        assert!(
            !reads.matching("envelope_json").is_empty(),
            "uncacheable fallback is loaded per request: {reads:#?}"
        );
    }
}
