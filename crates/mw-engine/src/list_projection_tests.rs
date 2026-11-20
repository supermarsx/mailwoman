//! Actual Email/get list projections: batch search counts and tenant/posture fences.
use super::*;
use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

struct LivePosture {
    zero: AtomicBool,
    reads: AtomicUsize,
}
impl crate::v6::AccountPostureSource for LivePosture {
    fn posture(&self, _: &str) -> crate::v6::AccountPosture {
        crate::v6::AccountPosture::Standard
    }
    fn posture_live<'a>(
        &'a self,
        _: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::v6::AccountPosture> + Send + 'a>>
    {
        Box::pin(async move {
            self.reads.fetch_add(1, Ordering::SeqCst);
            if self.zero.load(Ordering::SeqCst) {
                crate::v6::AccountPosture::ZeroAccess
            } else {
                crate::v6::AccountPosture::Standard
            }
        })
    }
}
async fn account(store: &Store, name: &str) -> (String, String) {
    let account = store
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
                password: "fixture".into(),
            },
        )
        .await
        .unwrap();
    let mailbox = store
        .upsert_mailbox(&MailboxUpsert {
            account_id: &account,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 1,
            uidnext: 31,
            highestmodseq: 0,
            total: 30,
            unread: 30,
            parent_id: None,
        })
        .await
        .unwrap();
    (account, mailbox)
}
async fn message(store: &Store, account: &str, mailbox: &str, uid: u32) -> String {
    let envelope=serde_json::to_vec(&json!({"subject":"projection","from":[{"email":"human@example.test"}],"attachments":[],"textBody":[],"htmlBody":[],"bodyValues":{},"_mailwomanAttachmentSchema":1})).unwrap();
    store
        .upsert_message(&MessageUpsert {
            account_id: account,
            mailbox_id: mailbox,
            uid,
            uidvalidity: 1,
            message_id: None,
            thread_id: None,
            internaldate: None,
            size: 10,
            flags_json: "[]",
            envelope: Some(&envelope),
            blob_ref: None,
        })
        .await
        .unwrap()
}

#[tokio::test]
async fn one_owned_projection_search_replaces_sixty_and_zero_access_bypasses_stale_index() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (owned, mailbox) = account(&store, "own@example.test").await;
    let (other, foreign_mailbox) = account(&store, "other@example.test").await;
    let mut ids = Vec::new();
    for uid in 1..=30 {
        ids.push(message(&store, &owned, &mailbox, uid).await);
    }
    let foreign = message(&store, &other, &foreign_mailbox, 1).await;
    let engine = crate::Engine::new(store);
    let mut docs = ids
        .iter()
        .enumerate()
        .map(|(n, id)| mw_search::IndexDoc {
            stable_id: id.clone(),
            account_id: owned.clone(),
            mailbox_id: mailbox.clone(),
            keywords: vec!["$mw-mention:me@example.test".into()],
            list_signal: (n % 2 == 0).then(|| "list-id".into()),
            calendar: (n == 0).then(|| mw_search::CalendarHint {
                title: "Meeting".into(),
                ..Default::default()
            }),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    docs.push(mw_search::IndexDoc {
        stable_id: foreign.clone(),
        account_id: other,
        mailbox_id: foreign_mailbox,
        keywords: vec!["$mw-mention:foreign-secret@example.test".into()],
        ..Default::default()
    });
    engine.search().upsert_batch(&docs).unwrap();
    let before = engine.search().doc_fetch_count();
    for id in ids.iter().chain(ids.iter()) {
        assert!(engine.search().fetch_doc(id).unwrap().is_some());
    }
    assert_eq!(engine.search().doc_fetch_count() - before, 60);
    let before_old = engine.search().doc_fetch_count();
    let before_batch = engine.search().list_projection_search_count();
    let mut requested = ids.clone();
    requested.extend([ids[0].clone(), foreign.clone(), "missing".into()]);
    let args = json!({"ids":requested,"properties":["id","mailwomanMentions","mailwomanCalendar","mailwomanFocus"],"mailwomanBodyUse":false});
    let response = engine.email_get(&owned, &args).await;
    let rows = response["list"].as_array().unwrap();
    assert_eq!(rows.len(), 31);
    assert_eq!(response["notFound"], json!([foreign, "missing"]));
    assert_eq!(
        engine.search().list_projection_search_count() - before_batch,
        1
    );
    assert_eq!(engine.search().list_projection_doc_read_count(), 30);
    assert_eq!(engine.search().doc_fetch_count(), before_old);
    for (n, row) in rows[..30].iter().enumerate() {
        assert_eq!(row["mailwomanMentions"], json!(["me@example.test"]));
        assert_eq!(
            row["mailwomanFocus"]["reason"],
            if n % 2 == 0 {
                json!("listHeader")
            } else {
                json!("default")
            }
        );
    }
    assert_eq!(rows[0]["mailwomanCalendar"]["title"], "Meeting");
    assert_eq!(rows[0], rows[30]);
    assert!(!response.to_string().contains("foreign-secret"));

    let posture = Arc::new(LivePosture {
        zero: AtomicBool::new(true),
        reads: AtomicUsize::new(0),
    });
    engine.attach_v6(crate::v6::V6Hooks::new().with_posture_source(posture.clone()));
    let before = engine.search().list_projection_search_count();
    let opaque = engine.email_get(&owned, &args).await;
    assert_eq!(posture.reads.load(Ordering::SeqCst), 1);
    assert_eq!(engine.search().list_projection_search_count(), before);
    assert_eq!(engine.search().doc_fetch_count(), before_old);
    for row in opaque["list"].as_array().unwrap() {
        assert_eq!(row["mailwomanMentions"], json!([]));
        assert_eq!(row["mailwomanCalendar"], Value::Null);
        assert_eq!(row["mailwomanFocus"]["reason"], "default");
    }
    posture.zero.store(false, Ordering::SeqCst);
    engine.email_get(&owned, &args).await;
    assert_eq!(posture.reads.load(Ordering::SeqCst), 2);
    assert_eq!(engine.search().list_projection_search_count() - before, 1);

    engine.store().delete_message(&ids[0]).await.unwrap();
    let before = engine.search().list_projection_search_count();
    let absent=engine.email_get(&owned,&json!({"ids":[ids[0],foreign,"missing"],"properties":["id","mailwomanMentions","mailwomanFocus"]})).await;
    assert_eq!(absent["list"], json!([]));
    assert_eq!(
        engine.search().list_projection_search_count(),
        before,
        "expunged and foreign IDs never reach the index"
    );
    engine.store().close().await;
}
