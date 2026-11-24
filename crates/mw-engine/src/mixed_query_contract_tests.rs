// Included in keyword_noop_tests to reuse its real Email/set backend/runtime.
// Expectations are explicit IDs, never computed by the implementation under test.
// No hand-built IndexDoc: every content witness comes from real raw publication.

struct MixedContract {
    f: Fixture,
    b: String,
    c: String,
    sent: String,
}

impl MixedContract {
    async fn new() -> Self {
        let f = Fixture::new("[]").await;
        let sent = f
            .engine
            .store()
            .upsert_mailbox(&MailboxUpsert {
                account_id: &f.account,
                name: "Sent",
                role: Some("sent"),
                uidvalidity: 2,
                uidnext: 20,
                highestmodseq: 0,
                total: 0,
                unread: 0,
                parent_id: None,
            })
            .await
            .unwrap();
        let mut ids = Vec::new();
        for (uid, mid, subject, from, to, body, date) in [
            (
                1,
                "<keyword@fixture.invalid>",
                "Zulu",
                "owner@fixture.invalid",
                "friend@example.test",
                "amberneedle",
                "2026-11-20T00:00:00Z",
            ),
            (
                2,
                "<mixed-b@fixture.invalid>",
                "Alpha",
                "friend@example.test",
                "owner@fixture.invalid",
                "amberneedle cobaltneedle",
                "2026-11-20T00:00:00Z",
            ),
            (
                3,
                "<mixed-c@fixture.invalid>",
                "Gamma",
                "third@example.test",
                "owner@fixture.invalid",
                "cobaltneedle",
                "2026-11-19T00:00:00Z",
            ),
        ] {
            let raw = format!(
                "Message-ID: {mid}\r\nFrom: {from}\r\nTo: {to}\r\nSubject: {subject}\r\nDate: Fri, 20 Nov 2026 00:00:00 +0000\r\nContent-Type: text/plain\r\n\r\n{body}\r\n"
            );
            let blob = f
                .engine
                .store()
                .put_body(&f.account, raw.as_bytes())
                .await
                .unwrap();
            let email = mw_mime::parse(raw.as_bytes()).unwrap().email;
            let envelope = crate::encode_message_envelope(&email, raw.as_bytes());
            ids.push(
                f.engine
                    .store()
                    .upsert_message(&MessageUpsert {
                        account_id: &f.account,
                        mailbox_id: &f.inbox,
                        uid,
                        uidvalidity: 1,
                        message_id: Some(mid),
                        thread_id: None,
                        internaldate: Some(date),
                        size: raw.len() as u64,
                        flags_json: "[]",
                        envelope: Some(&envelope),
                        blob_ref: Some(&blob),
                    })
                    .await
                    .unwrap(),
            );
        }
        assert_eq!(ids[0], f.id, "fixture replaces the same source identity");
        f.engine
            .publish_current_index(
                &f.account,
                ids.clone(),
                crate::index_publication::PublicationMode::Rebuild,
            )
            .await
            .unwrap();
        for id in &ids {
            f.engine
                .record_change(&f.account, ChangeType::Email, id, ChangeOp::Created)
                .await
                .unwrap();
            assert!(
                f.engine
                    .search()
                    .fetch_doc(id)
                    .unwrap()
                    .unwrap()
                    .source_content_ref
                    .is_some()
            );
        }
        Self {
            f,
            b: ids[1].clone(),
            c: ids[2].clone(),
            sent,
        }
    }

    async fn query(&self, args: Value) -> Value {
        let out = self.f.engine.email_query(&self.f.account, &args).await;
        assert!(out.get("ids").is_some(), "{out}");
        out
    }

    async fn expect(&self, filter: Value, wanted: &[&str]) {
        let out = self.query(json!({"filter":filter,"calculateTotal":true,
            "sort":[{"property":"receivedAt","isAscending":false},{"property":"subject","isAscending":true}]})).await;
        assert_eq!(out["ids"], json!(wanted), "{out}");
        assert_eq!(out["total"], wanted.len(), "{out}");
    }

    async fn patch(&self, id: &str, patch: Value) -> Value {
        let out = self
            .f
            .engine
            .email_set(
                &self.f.account,
                &self.f.runtime,
                &json!({"update":{(id):patch}}),
                &mut HashMap::new(),
            )
            .await;
        assert!(
            out["updated"]
                .as_object()
                .is_some_and(|rows| rows.contains_key(id)),
            "{out}"
        );
        assert!(out.get("notUpdated").is_none(), "{out}");
        let health = self.f.engine.index_publication_status();
        assert!(
            !health.poisoned && health.pending_jobs == 0 && health.pending_repair_messages == 0
        );
        out
    }
}

#[tokio::test]
async fn mixed_contract_real_flag_pin_followup_and_move_preserve_boolean_fulltext() {
    let s = MixedContract::new().await;
    let a = s.f.id.as_str();
    let b = s.b.as_str();
    let c = s.c.as_str();
    s.expect(
        json!({"text":"body:amberneedle OR body:cobaltneedle"}),
        &[b, a, c],
    )
    .await;
    s.expect(
        json!({"text":"body:amberneedle AND NOT body:cobaltneedle"}),
        &[a],
    )
    .await;
    s.patch(
        a,
        json!({"keywords/$seen":true,"pinned":true,"followUpAt":"2030-01-01T00:00:00Z"}),
    )
    .await;
    s.expect(
        json!({"text":"(body:amberneedle AND is:read) OR (body:cobaltneedle AND NOT is:read)"}),
        &[b, a, c],
    )
    .await;
    s.expect(
        json!({"text":"body:amberneedle AND pinned:true AND is:followup"}),
        &[a],
    )
    .await;
    s.patch(
        a,
        json!({"keywords/mw-followup-completed":true,"keywords/$flagged":true}),
    )
    .await;
    s.expect(json!({"text":"body:amberneedle AND is:followup"}), &[])
        .await;
    s.expect(json!({"text":"body:amberneedle AND is:flagged"}), &[a])
        .await;
    s.patch(a, json!({"keywords/mw-followup-completed":null,"keywords/$seen":null,"followUpAt":null,"keywords/$flagged":null,"pinned":false})).await;
    s.expect(
        json!({"text":"body:amberneedle AND NOT pinned:true AND NOT is:read"}),
        &[b, a],
    )
    .await;
    s.expect(json!({"text":"body:amberneedle AND is:followup"}), &[])
        .await;
    s.patch(a, json!({"mailboxIds":{s.f.archive.clone():true}}))
        .await;
    s.expect(
        json!({"text":"body:amberneedle","inMailboxes":[s.f.archive]}),
        &[a],
    )
    .await;
    s.expect(
        json!({"text":"body:amberneedle","inMailboxOtherThan":[s.f.archive]}),
        &[b],
    )
    .await;
    s.patch(a, json!({"mailboxIds":{s.f.inbox.clone():true}}))
        .await;
    s.expect(
        json!({"text":"body:amberneedle","inMailbox":s.f.inbox}),
        &[b, a],
    )
    .await;
}

#[tokio::test]
async fn mixed_contract_effective_keyword_collisions_keep_exact_negation_and_importance() {
    let s = MixedContract::new().await;
    // These are ordinary custom flags accepted by today's mapping. An overlay
    // must not discard them just because content-derived flags share a prefix.
    s.patch(
        &s.f.id,
        json!({
        (format!("keywords/{}",mw_search::KW_FOLLOWUP_ACTIVE)):true,
        (format!("keywords/{}",mw_search::KW_PRIORITY_HIGH)):true,
        "keywords/Work":true}),
    )
    .await;
    let current = s.f.engine.search().fetch_doc(&s.f.id).unwrap().unwrap();
    assert!(
        current
            .keywords
            .iter()
            .any(|key| key == "$mw-followup-active")
    );
    s.expect(
        json!({"text":"body:amberneedle AND is:followup"}),
        &[&s.f.id],
    )
    .await;
    s.expect(json!({"text":"body:amberneedle AND tag:Work"}), &[&s.f.id])
        .await;
    s.expect(
        json!({"text":"body:amberneedle AND NOT tag:work"}),
        &[&s.b, &s.f.id],
    )
    .await;
    s.patch(&s.f.id, json!({"keywords/mw-followup-completed":true}))
        .await;
    s.expect(
        json!({"text":"(body:amberneedle OR body:cobaltneedle) AND NOT is:followup"}),
        &[&s.b, &s.f.id, &s.c],
    )
    .await;
    let out = s.query(json!({"filter":{"text":"body:amberneedle"},"calculateTotal":true,
        "sort":[{"property":"importance","isAscending":false},{"property":"subject","isAscending":true}]})).await;
    assert_eq!(
        out["ids"],
        json!([s.f.id, s.b]),
        "effective custom priority participates in the existing comparator"
    );
    assert_eq!(out["total"], 2);
    assert_eq!(out["ids"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn mixed_contract_compound_sort_anchor_total_and_query_changes_reconstruct_query() {
    let s = MixedContract::new().await;
    let filter = json!({"text":"body:amberneedle OR body:cobaltneedle","notKeyword":"$seen"});
    let sort = json!([{"property":"receivedAt","isAscending":false},{"property":"subject","isAscending":true}]);
    let before = s
        .query(json!({"filter":filter,"sort":sort,"calculateTotal":true}))
        .await;
    assert_eq!(before["ids"], json!([s.b, s.f.id, s.c]));
    let page = s.query(json!({"filter":filter,"sort":sort,"anchor":s.f.id,"anchorOffset":-1,"limit":2,"calculateTotal":true})).await;
    assert_eq!(page["ids"], json!([s.b, s.f.id]));
    assert_eq!(page["position"], 0);
    assert_eq!(page["total"], 3);
    s.patch(&s.b, json!({"keywords/$seen":true})).await;
    let delta = s.f.engine.email_query_changes(&s.f.account, &json!({"filter":filter,"sort":sort,"sinceQueryState":before["queryState"],"calculateTotal":true})).await;
    assert!(delta.get("removed").is_some(), "{delta}");
    assert!(
        delta["removed"].as_array().unwrap().contains(&json!(s.b)),
        "{delta}"
    );
    let mut reconstructed = before["ids"].as_array().unwrap().clone();
    reconstructed.retain(|id| !delta["removed"].as_array().unwrap().contains(id));
    for added in delta["added"].as_array().unwrap() {
        reconstructed.insert(
            added["index"].as_u64().unwrap() as usize,
            added["id"].clone(),
        );
    }
    let after = s
        .query(json!({"filter":filter,"sort":sort,"calculateTotal":true}))
        .await;
    assert_eq!(after["ids"], json!([s.f.id, s.c]));
    assert_eq!(json!(reconstructed), after["ids"]);
    assert_eq!(delta["total"], after["total"]);
    let absent =
        s.f.engine
            .email_query(
                &s.f.account,
                &json!({"filter":filter,"sort":sort,"anchor":s.b}),
            )
            .await;
    assert_eq!(absent["type"], "anchorNotFound");
    for unsupported in [
        json!({"operator":"OR","conditions":[{"hasKeyword":"$seen"}]}),
        json!({"unknownPredicate":true}),
    ] {
        let out =
            s.f.engine
                .email_query(&s.f.account, &json!({"filter":unsupported}))
                .await;
        assert_eq!(out["type"], "invalidArguments", "{out}");
    }
}

#[tokio::test]
async fn mixed_contract_focus_sent_scope_and_real_content_replacement_are_not_metadata_only() {
    let s = MixedContract::new().await;
    let focus = || json!({"ids":[s.b],"properties":["id","from","mailwomanFocus"]});
    let initial = s.f.engine.email_get(&s.f.account, &focus()).await;
    assert_eq!(
        initial["list"][0]["mailwomanFocus"]["reason"], "default",
        "{initial}"
    );
    s.patch(&s.f.id, json!({"mailboxIds":{s.sent.clone():true}}))
        .await;
    let moved = s.f.engine.email_get(&s.f.account, &focus()).await;
    assert_eq!(
        moved["list"][0]["mailwomanFocus"]["reason"], "correspondent",
        "{moved}"
    );
    s.patch(&s.f.id, json!({"mailboxIds":{s.f.inbox.clone():true}}))
        .await;
    let restored = s.f.engine.email_get(&s.f.account, &focus()).await;
    assert_eq!(
        restored["list"][0]["mailwomanFocus"]["reason"], "default",
        "{restored}"
    );
    let old = s.f.engine.search().fetch_doc(&s.f.id).unwrap().unwrap();
    let source = s.f.engine.store().get_message(&s.f.id).await.unwrap();
    let raw=b"From: owner@fixture.invalid\r\nTo: friend@example.test\r\nSubject: Replacement\r\nContent-Type: text/plain\r\n\r\nreplacementneedle\r\n";
    let blob =
        s.f.engine
            .store()
            .put_body(&s.f.account, raw)
            .await
            .unwrap();
    let email = mw_mime::parse(raw).unwrap().email;
    let envelope = crate::encode_message_envelope(&email, raw);
    let same =
        s.f.engine
            .store()
            .upsert_message(&MessageUpsert {
                account_id: &s.f.account,
                mailbox_id: &source.mailbox_id,
                uid: source.uid,
                uidvalidity: source.uidvalidity,
                message_id: source.message_id.as_deref(),
                thread_id: source.thread_id.as_deref(),
                internaldate: source.internaldate.as_deref(),
                size: raw.len() as u64,
                flags_json: &source.flags_json,
                envelope: Some(&envelope),
                blob_ref: Some(&blob),
            })
            .await
            .unwrap();
    assert_eq!(same, s.f.id);
    s.f.engine
        .publish_current_index(
            &s.f.account,
            vec![same],
            crate::index_publication::PublicationMode::Rebuild,
        )
        .await
        .unwrap();
    s.expect(json!({"text":"body:replacementneedle"}), &[&s.f.id])
        .await;
    s.expect(json!({"text":"body:amberneedle"}), &[&s.b]).await;
    let new = s.f.engine.search().fetch_doc(&s.f.id).unwrap().unwrap();
    assert_ne!(new.source_content_ref, old.source_content_ref);
    assert!(new.body.contains("replacementneedle"));
    assert!(!new.body.contains("amberneedle"));
}

#[tokio::test]
async fn mixed_contract_foreign_matching_content_never_enters_ids_totals_or_mutations() {
    let s = MixedContract::new().await;
    let other =
        s.f.engine
            .store()
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "other.invalid",
                    port: 993,
                    tls: "implicit",
                    username: "other@example.test",
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: "other@example.test".into(),
                    password: "fixture-only".into(),
                },
            )
            .await
            .unwrap();
    let folder =
        s.f.engine
            .store()
            .upsert_mailbox(&MailboxUpsert {
                account_id: &other,
                name: "INBOX",
                role: Some("inbox"),
                uidvalidity: 1,
                uidnext: 2,
                highestmodseq: 0,
                total: 0,
                unread: 0,
                parent_id: None,
            })
            .await
            .unwrap();
    let raw=b"From: foreign@example.test\r\nTo: other@example.test\r\nSubject: Foreign\r\nContent-Type: text/plain\r\n\r\namberneedle cobaltneedle\r\n";
    let blob = s.f.engine.store().put_body(&other, raw).await.unwrap();
    let envelope = crate::encode_message_envelope(&mw_mime::parse(raw).unwrap().email, raw);
    let id =
        s.f.engine
            .store()
            .upsert_message(&MessageUpsert {
                account_id: &other,
                mailbox_id: &folder,
                uid: 1,
                uidvalidity: 1,
                message_id: Some("<foreign@x>"),
                thread_id: None,
                internaldate: Some("2026-11-21T00:00:00Z"),
                size: raw.len() as u64,
                flags_json: "[]",
                envelope: Some(&envelope),
                blob_ref: Some(&blob),
            })
            .await
            .unwrap();
    s.f.engine
        .publish_current_index(
            &other,
            vec![id.clone()],
            crate::index_publication::PublicationMode::Rebuild,
        )
        .await
        .unwrap();
    s.expect(
        json!({"text":"body:amberneedle OR body:cobaltneedle"}),
        &[&s.b, &s.f.id, &s.c],
    )
    .await;
    s.expect(
        json!({"text":"body:amberneedle","inMailboxes":[folder]}),
        &[],
    )
    .await;
    let before = s.f.engine.store().get_message(&id).await.unwrap();
    let before_meta = s.f.engine.store().get_message_meta(&id).await.unwrap();
    let owner_state =
        s.f.engine
            .type_state(&s.f.account, ChangeType::Email)
            .await
            .unwrap();
    let foreign_state =
        s.f.engine
            .type_state(&other, ChangeType::Email)
            .await
            .unwrap();
    let submitted = s.f.engine.index_publication_status().submitted_jobs;
    let upstream_flags = s.f.remote.flags.load(Ordering::SeqCst);
    let upstream_moves = s.f.remote.moves.load(Ordering::SeqCst);
    let missing = "mixed-definitely-missing";
    let out =
        s.f.engine
            .email_set(
                &s.f.account,
                &s.f.runtime,
                &json!({"update":{
                    id.clone():{"keywords/$seen":true,"pinned":true},
                    (missing):{"keywords/$seen":true,"pinned":true}
                }}),
                &mut HashMap::new(),
            )
            .await;
    // Missing and foreign ids get the same RFC 8620 `notFound` SetError with
    // the same fixed text (t55): the answer never reveals whether another
    // owner has this ID.
    assert_eq!(out["notUpdated"][&id]["type"], "notFound", "{out}");
    assert_eq!(out["notUpdated"][&id], out["notUpdated"][missing], "{out}");
    assert_eq!(out["notUpdated"].as_object().unwrap().len(), 2);
    assert!(
        out.get("updated")
            .and_then(Value::as_object)
            .is_none_or(|rows| rows.is_empty()),
        "{out}"
    );
    let after = s.f.engine.store().get_message(&id).await.unwrap();
    assert_eq!(before.cache_revision, after.cache_revision);
    assert_eq!(before.flags_json, after.flags_json);
    assert_eq!(
        before_meta,
        s.f.engine.store().get_message_meta(&id).await.unwrap()
    );
    assert_eq!(
        owner_state,
        s.f.engine
            .type_state(&s.f.account, ChangeType::Email)
            .await
            .unwrap()
    );
    assert_eq!(
        foreign_state,
        s.f.engine
            .type_state(&other, ChangeType::Email)
            .await
            .unwrap()
    );
    assert_eq!(
        submitted,
        s.f.engine.index_publication_status().submitted_jobs
    );
    assert_eq!(upstream_flags, s.f.remote.flags.load(Ordering::SeqCst));
    assert_eq!(upstream_moves, s.f.remote.moves.load(Ordering::SeqCst));
    let foreign =
        s.f.engine
            .email_query(
                &other,
                &json!({"filter":{"text":"body:amberneedle"},"calculateTotal":true}),
            )
            .await;
    assert_eq!(foreign["ids"], json!([id]));
    assert_eq!(foreign["total"], 1);
}
