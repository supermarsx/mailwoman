use super::*;
use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
};
async fn account(store: &Store, name: &str) -> String {
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "fixture.test",
                port: 993,
                tls: "implicit",
                username: name,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: name.into(),
                password: "synthetic".into(),
            },
        )
        .await
        .unwrap()
}
async fn fixture(store: &Store) -> (String, String, String) {
    let a = account(store, "a@example.test").await;
    let b = account(store, "b@example.test").await;
    let mailbox = store
        .upsert_mailbox(&MailboxUpsert {
            account_id: &a,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 1,
            uidnext: 2,
            highestmodseq: 0,
            total: 1,
            unread: 1,
            parent_id: None,
        })
        .await
        .unwrap();
    let body = store
        .put_body(
            &a,
            b"From: a@example.test\r\nSubject: synthetic\r\n\r\nbody",
        )
        .await
        .unwrap();
    let id = store
        .upsert_message(&MessageUpsert {
            account_id: &a,
            mailbox_id: &mailbox,
            uid: 1,
            uidvalidity: 1,
            message_id: Some("<fixture@example.test>"),
            thread_id: None,
            internaldate: None,
            size: 57,
            flags_json: "[]",
            envelope: Some(b"{}"),
            blob_ref: Some(&body),
        })
        .await
        .unwrap();
    (a, b, id)
}

async fn cached_fixture() -> (Engine, String, String, String) {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (a, b, id) = fixture(&store).await;
    let (source, raw) = store
        .security_verdict_source(&a, &id)
        .await
        .unwrap()
        .unwrap();
    let mut verdict = compute_verdict(&id, &raw, None, None, false).await;
    verdict.attachments.push(AttachmentRisk {
        name: "private-foreign-attachment.pdf".into(),
        declared_type: Some("application/pdf".into()),
        detected_type: Some("application/pdf".into()),
        mismatch: false,
        risk: "none".into(),
    });
    let bytes = serde_json::to_vec(&verdict).unwrap();
    assert!(
        store
            .security_verdict_cache_if_current(&source, &hex_sha256(&raw), &bytes)
            .await
            .unwrap()
    );
    (Engine::new(store), a, b, id)
}
#[tokio::test]
async fn foreign_cached_verdict_is_uniform_not_found_and_mixed_ids_stay_scoped() {
    let (engine, a, b, id) = cached_fixture().await;
    let foreign = engine
        .security_verdict_get_owned(&b, &json!({"ids":[id,"absent"]}))
        .await;
    assert_eq!(foreign["list"], json!([]));
    assert_eq!(foreign["notFound"], json!([id, "absent"]));
    assert!(!foreign.to_string().contains("private-foreign"));
    let own = engine
        .security_verdict_get_owned(&a, &json!({"ids":[id,"absent",id]}))
        .await;
    assert_eq!(own["list"].as_array().unwrap().len(), 2);
    assert_eq!(own["notFound"], json!(["absent"]));
    let rows = engine
        .store()
        .get_account_messages(&a, &[id.clone()])
        .await
        .unwrap();
    let source_rows = std::collections::HashMap::from([(
        id.clone(),
        rows[0].as_ref().unwrap().cache_revision.clone(),
    )]);
    let mut values = vec![Some(json!({"id":id}))];
    engine
        .fill_auth(&b, &mut values, &source_rows)
        .await
        .unwrap();
    assert_eq!(values[0].as_ref().unwrap()["mailwomanAuth"], Value::Null);
    engine
        .fill_auth(&a, &mut values, &source_rows)
        .await
        .unwrap();
    assert!(values[0].as_ref().unwrap()["mailwomanAuth"].is_object());
    engine.store().set_flags(&id, "[]").await.unwrap();
    let fresh = engine
        .store()
        .get_account_messages(&a, &[id.clone()])
        .await
        .unwrap();
    let fresh = std::collections::HashMap::from([(
        id.clone(),
        fresh[0].as_ref().unwrap().cache_revision.clone(),
    )]);
    engine.fill_auth(&a, &mut values, &fresh).await.unwrap();
    assert!(values[0].as_ref().unwrap()["mailwomanAuth"].is_object());
}
#[tokio::test]
async fn source_replaced_during_verification_cannot_cache_or_return_old_verdict() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (a, _, id) = fixture(&store).await;
    let engine = Engine::new(store);
    let (source, raw) = engine.raw_for_email(&a, &id).await.unwrap();
    let raw2 = b"From: changed@example.test\r\n\r\nchanged";
    let result = engine
        .verdict_from_source(&a, &source, &raw, || async {
            let blob = engine.store().put_body(&a, raw2).await.unwrap();
            let old = engine.store().get_message(&id).await.unwrap();
            engine
                .store()
                .upsert_message(&MessageUpsert {
                    account_id: &a,
                    mailbox_id: &old.mailbox_id,
                    uid: old.uid,
                    uidvalidity: old.uidvalidity,
                    message_id: old.message_id.as_deref(),
                    thread_id: old.thread_id.as_deref(),
                    internaldate: old.internaldate.as_deref(),
                    size: raw2.len() as u64,
                    flags_json: &old.flags_json,
                    envelope: Some(b"{}"),
                    blob_ref: Some(&blob),
                })
                .await
                .unwrap();
            tokio::task::yield_now().await;
            compute_verdict(&id, &raw, None, None, false).await
        })
        .await;
    assert!(result.is_none());
    assert!(
        engine
            .store()
            .get_security_verdicts(&a, &[id])
            .await
            .unwrap()
            .is_empty()
    );
}
struct Zero;
impl crate::v6::AccountPostureSource for Zero {
    fn posture(&self, _: &str) -> crate::v6::AccountPosture {
        crate::v6::AccountPosture::ZeroAccess
    }
}
#[tokio::test]
async fn zero_access_and_deleted_sources_cannot_reuse_cached_or_computed_verdict() {
    let (engine, a, _, id) = cached_fixture().await;
    let (source, raw) = engine.raw_for_email(&a, &id).await.unwrap();
    let source_rows = engine
        .store()
        .get_account_messages(&a, &[id.clone()])
        .await
        .unwrap();
    let source_rows = std::collections::HashMap::from([(
        id.clone(),
        source_rows[0].as_ref().unwrap().cache_revision.clone(),
    )]);
    engine.attach_v6(crate::v6::V6Hooks::new().with_posture_source(std::sync::Arc::new(Zero)));
    assert!(engine.raw_for_email(&a, &id).await.is_none());
    let mut list = vec![Some(json!({"id":id}))];
    engine.fill_auth(&a, &mut list, &source_rows).await.unwrap();
    assert_eq!(list[0].as_ref().unwrap()["mailwomanAuth"], Value::Null);
    assert!(
        engine
            .verdict_from_source(&a, &source, &raw, || async {
                panic!("zero access must not compute or use cached verdict")
            })
            .await
            .is_none()
    );
    engine.attach_v6(crate::v6::V6Hooks::new());
    engine.store().delete_message(&id).await.unwrap();
    assert!(
        engine
            .verdict_from_source(&a, &source, &raw, || async {
                compute_verdict(&id, &raw, None, None, false).await
            })
            .await
            .is_none()
    );
}

#[tokio::test]
async fn zero_access_transition_during_dns_never_persists_a_new_verdict() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (a, _, id) = fixture(&store).await;
    let engine = Engine::new(store);
    let (source, raw) = engine.raw_for_email(&a, &id).await.unwrap();
    let result = engine
        .verdict_from_source(&a, &source, &raw, || async {
            tokio::task::yield_now().await;
            engine.attach_v6(
                crate::v6::V6Hooks::new().with_posture_source(std::sync::Arc::new(Zero)),
            );
            compute_verdict(&id, &raw, None, None, false).await
        })
        .await;
    assert!(result.is_none());
    engine.attach_v6(crate::v6::V6Hooks::new());
    assert!(
        engine
            .store()
            .get_security_verdicts(&a, &[id])
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn flag_change_during_dns_preserves_content_proof_and_returns_verdict() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (a, _, id) = fixture(&store).await;
    let engine = Engine::new(store);
    let (source, raw) = engine.raw_for_email(&a, &id).await.unwrap();
    let result = engine
        .verdict_from_source(&a, &source, &raw, || async {
            engine.store().set_flags(&id, "[]").await.unwrap();
            tokio::task::yield_now().await;
            compute_verdict(&id, &raw, None, None, false).await
        })
        .await;
    assert_eq!(result.unwrap().email_id, id);
    assert!(
        engine
            .store()
            .get_security_verdicts(&a, &[id])
            .await
            .unwrap()
            .len()
            == 1
    );
}
