use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
};
async fn account(store: &Store, user: &str) -> String {
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: user,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: user.into(),
                password: "fixture".into(),
            },
        )
        .await
        .unwrap()
}
async fn mailbox(store: &Store, account: &str, name: &str) -> String {
    store
        .upsert_mailbox(&MailboxUpsert {
            account_id: account,
            name,
            role: None,
            uidvalidity: 1,
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        })
        .await
        .unwrap()
}
fn message<'a>(account: &'a str, mailbox: &'a str) -> MessageUpsert<'a> {
    MessageUpsert {
        account_id: account,
        mailbox_id: mailbox,
        uid: 1,
        uidvalidity: 1,
        message_id: Some("<test@example.org>"),
        thread_id: None,
        internaldate: None,
        size: 4,
        flags_json: "[]",
        envelope: None,
        blob_ref: None,
    }
}
async fn exercise_dates(store: Store) {
    let tag = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let actor = account(&store, &format!("dates-owner-{tag}@example.org")).await;
    let other = account(&store, &format!("dates-other-{tag}@example.org")).await;
    let inbox = mailbox(&store, &actor, "INBOX").await;
    let foreign = mailbox(&store, &other, "INBOX").await;
    let mut old = message(&actor, &inbox);
    old.internaldate = Some("26-Oct-2026 02:20:35 +0000");
    let old_id = store.upsert_message(&old).await.unwrap();
    let mut fresh = message(&actor, &inbox);
    fresh.uid = 2;
    fresh.internaldate = Some("2026-10-26T03:28:33Z");
    let fresh_id = store.upsert_message(&fresh).await.unwrap();
    let mut away = message(&other, &foreign);
    away.internaldate = old.internaldate;
    let away_id = store.upsert_message(&away).await.unwrap();
    assert_eq!(
        store.list_message_ids(&inbox, 10, 0).await.unwrap(),
        vec![old_id.clone(), fresh_id.clone()],
        "legacy raw timestamps incorrectly hide new mail"
    );
    assert_eq!(store.normalize_cached_imap_dates(&actor).await.unwrap(), 1);
    assert_eq!(
        store.list_message_ids(&inbox, 10, 0).await.unwrap(),
        vec![fresh_id, old_id.clone()]
    );
    assert_eq!(
        store
            .get_message(&old_id)
            .await
            .unwrap()
            .internaldate
            .as_deref(),
        Some("2026-10-26T02:20:35Z")
    );
    assert_eq!(
        store
            .get_message(&away_id)
            .await
            .unwrap()
            .internaldate
            .as_deref(),
        old.internaldate
    );
    assert_eq!(store.normalize_cached_imap_dates(&actor).await.unwrap(), 0);
}
#[tokio::test]
async fn cached_imap_dates_sqlite() {
    exercise_dates(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn cached_imap_dates_postgres() {
    if let Ok(url) = std::env::var("MW_TEST_PG") {
        exercise_dates(Store::open(&url, ServerKey::generate()).await.unwrap()).await;
    }
}
