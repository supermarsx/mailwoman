// Included inside email_get_batch_tests to reuse its real Store/SQL recorder.
#[test]
fn light_routing_is_positive_and_preserves_explicit_body_semantics() {
    let default = json!({"properties":["id","blobId","from","to","subject","receivedAt","preview","keywords","mailboxIds","threadId","pinned","snoozedUntil","followUpAt","hasAttachment","size","mailwomanFocus","mailwomanAuth","mailwomanPriority","mailwomanMentions","cc","bcc","replyTo","sentAt","messageId","inReplyTo","references","header:Delivered-To:asAddresses:all","header:X-Original-To:asAddresses:all","header:Envelope-To:asAddresses:all","header:X-Forwarded-To:asAddresses:all","header:Resent-To:asAddresses:all","header:Expires:asText","header:Expiry-Date:asText","header:Reply-By:asText","header:Sensitivity:asText","header:Supersedes:asText"],"fetchHTMLBodyValues":false,"fetchTextBodyValues":false,"mailwomanBodyUse":false});
    assert!(metadata_envelope_only(&default));
    assert!(metadata_envelope_only(&json!({"properties":[]})));
    for args in [
        json!({}),
        json!({"properties":null}),
        json!({"properties":"id"}),
        json!({"properties":[1]}),
        json!({"properties":["futureMetadata"]}),
        json!({"properties":["header:Subject:asText"]}),
        json!({"properties":["bodyValues"],"fetchHTMLBodyValues":false,"fetchTextBodyValues":false,"mailwomanBodyUse":false}),
        json!({"properties":["htmlBody"]}),
        json!({"properties":["textBody"]}),
        json!({"properties":["attachments"]}),
        json!({"properties":["bodyStructure"]}),
        json!({"properties":["mailwomanMdn"]}),
        json!({"properties":["id"],"mailwomanBodyUse":true}),
        json!({"properties":["id"],"fetchAllBodyValues":true}),
        json!({"properties":["id"],"fetchTextBodyValues":true}),
        json!({"properties":["id"],"fetchHTMLBodyValues":true}),
    ] {
        assert!(
            !metadata_envelope_only(&args),
            "must retain full semantics: {args}"
        );
    }
}

async fn light_seed(f: &Fixture, subject: &str, body_bytes: usize, extra: bool) {
    let raw = format!(
        "From: sender@example.org\r\nSubject: {subject}\r\nDelivered-To: owner@example.org\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{}",
        "x".repeat(body_bytes)
    );
    let parsed = mw_mime::parse(raw.as_bytes()).unwrap();
    let mut envelope: Value = serde_json::from_slice(&crate::encode_message_envelope(
        &parsed.email,
        raw.as_bytes(),
    ))
    .unwrap();
    if extra {
        envelope["futureMetadata"] = json!({"keep":true});
    }
    seed_legacy_projection(
        f,
        Some(raw.as_bytes()),
        &serde_json::to_vec(&envelope).unwrap(),
    )
    .await;
}
fn light_args(id: &str) -> Value {
    json!({"ids":[id],"properties":["id","subject","preview","header:Delivered-To:asAddresses:all"],"fetchHTMLBodyValues":false,"fetchTextBodyValues":false,"mailwomanBodyUse":false})
}

#[tokio::test]
async fn light_current_pages_avoid_full_envelope_reads_and_reader_cache_stays_complete() {
    for cached in [false, true] {
        let f = Fixture::with(1).await;
        light_seed(&f, "Small header, large body", 512 * 1024, true).await;
        // Legacy storage may need a one-time companion backfill. Timed/read
        // assertions below cover the resulting current lightweight projection.
        let projected = f
            .store()
            .get_account_header_envelope(&f.account, &f.ids[0])
            .await
            .unwrap()
            .unwrap();
        assert!(projected.projected);
        assert!(projected.envelope.len() < 16 * 1024);
        assert!(
            !serde_json::from_slice::<Value>(&projected.envelope)
                .unwrap()
                .as_object()
                .unwrap()
                .contains_key("bodyValues")
        );
        if cached {
            f.engine.attach_v6(
                crate::v6::V6Hooks::new().with_cache(mw_cache::Cache::in_memory(
                    mw_cache::ScopeMatrix::spec_defaults(),
                )),
            );
        }
        for _ in 0..2 {
            let (response, reads) = counted(
                &f.threads,
                f.engine.email_get(&f.account, &light_args(&f.ids[0])),
            )
            .await;
            assert_eq!(response["list"][0]["subject"], "Small header, large body");
            assert_eq!(reads.against("bodies"), 0, "{reads:#?}");
            assert!(
                reads.matching("envelope_json").is_empty(),
                "full envelope must not be selected: {reads:#?}"
            );
        }
        // Explicit stored bodyValues remains a full read even with all fetch
        // flags false; a light cache hit must never satisfy this request.
        let args = json!({"ids":[f.ids[0]],"properties":["id","bodyValues","futureMetadata"],"mailwomanBodyUse":false,"fetchTextBodyValues":false});
        let (reader, reads) = counted(&f.threads, f.engine.email_get(&f.account, &args)).await;
        assert!(reads.against("envelope_json") > 0, "{reads:#?}");
        assert_eq!(reader["list"][0]["futureMetadata"], json!({"keep":true}));
        assert!(
            reader["list"][0]["bodyValues"]
                .as_object()
                .unwrap()
                .values()
                .any(|value| value["value"]
                    .as_str()
                    .is_some_and(|text| text.len() >= 512 * 1024))
        );
        let reader2 = f.engine.email_get(&f.account, &args).await;
        assert_eq!(reader2, reader);
        // A new authoritative envelope invalidates both revision namespaces.
        light_seed(&f, "Changed header", 64, false).await;
        let changed = f.engine.email_get(&f.account, &light_args(&f.ids[0])).await;
        assert_eq!(changed["list"][0]["subject"], "Changed header");
        let (foreign, reads) = counted(
            &f.threads,
            f.engine
                .email_get("foreign-account", &light_args(&f.ids[0])),
        )
        .await;
        assert_eq!(foreign["list"], json!([]));
        assert_eq!(foreign["notFound"], json!([f.ids[0]]));
        assert!(
            reads
                .matching("message_list_envelope")
                .iter()
                .all(|statement| statement.rows == 0)
        );
    }
}

#[tokio::test]
async fn light_overcap_concurrent_fallback_is_returned_per_request_and_never_cached() {
    let f = Fixture::with(1).await;
    let subject = "s".repeat(140 * 1024);
    light_seed(&f, &subject, 256 * 1024, false).await;
    f.engine.attach_v6(
        crate::v6::V6Hooks::new().with_cache(mw_cache::Cache::in_memory(
            mw_cache::ScopeMatrix::spec_defaults(),
        )),
    );
    let args = light_args(&f.ids[0]);
    let (a, b, c) = tokio::join!(
        f.engine.email_get(&f.account, &args),
        f.engine.email_get(&f.account, &args),
        f.engine.email_get(&f.account, &args)
    );
    for response in [a, b, c] {
        assert_eq!(response["list"][0]["subject"], subject);
    }
    for _ in 0..2 {
        let (response, reads) = counted(&f.threads, f.engine.email_get(&f.account, &args)).await;
        assert_eq!(response["list"][0]["subject"], subject);
        assert!(
            reads.against("envelope_json") > 0,
            "fallback must not enter shared light cache: {reads:#?}"
        );
    }
}

#[tokio::test]
async fn light_legacy_projection_repairs_without_losing_fields_then_avoids_raw() {
    let f = Fixture::with(1).await;
    let legacy = br#"{"subject":"Preserved legacy subject","hasAttachment":false}"#;
    seed_legacy_projection(&f, Some(PROJECTED_HEADER_RAW), legacy).await;
    f.engine.attach_v6(
        crate::v6::V6Hooks::new().with_cache(mw_cache::Cache::in_memory(
            mw_cache::ScopeMatrix::spec_defaults(),
        )),
    );
    let args = light_args(&f.ids[0]);
    let first = f.engine.email_get(&f.account, &args).await;
    assert_eq!(first["list"][0]["subject"], "Preserved legacy subject");
    assert_eq!(
        first["list"][0]["header:Delivered-To:asAddresses:all"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let (second, reads) = counted(&f.threads, f.engine.email_get(&f.account, &args)).await;
    assert_eq!(second, first);
    assert_eq!(
        reads.against("bodies"),
        0,
        "legacy raw repair must settle: {reads:#?}"
    );
    assert!(
        reads.matching("envelope_json").is_empty(),
        "legacy companion repair must settle: {reads:#?}"
    );
}

#[tokio::test]
async fn light_cold_loader_rejects_a_changed_source_witness_before_caching() {
    for cached in [false, true] {
        let f = Fixture::with(1).await;
        light_seed(&f, "Earlier source", 256, false).await;
        let earlier = f
            .store()
            .get_account_messages(&f.account, &[f.ids[0].clone()])
            .await
            .unwrap()
            .remove(0)
            .unwrap();
        if cached {
            f.engine.attach_v6(
                crate::v6::V6Hooks::new().with_cache(mw_cache::Cache::in_memory(
                    mw_cache::ScopeMatrix::spec_defaults(),
                )),
            );
        }
        light_seed(&f, "Replacement source", 256, false).await;
        let result = f
            .engine
            .cached_header_envelope_with_posture(&earlier, crate::v6::AccountPosture::Standard)
            .await;
        assert!(
            result.is_err(),
            "new envelope must not be associated with old revision"
        );
        let current = f.engine.email_get(&f.account, &light_args(&f.ids[0])).await;
        assert_eq!(current["list"][0]["subject"], "Replacement source");
    }
}
