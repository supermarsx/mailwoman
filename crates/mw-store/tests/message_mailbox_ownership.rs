use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
    StoreError,
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
async fn exercise(store: Store) {
    let tag = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let actor = account(&store, &format!("actor-{tag}@example.org")).await;
    let victim = account(&store, &format!("victim-{tag}@example.org")).await;
    let foreign = mailbox(&store, &victim, "private").await;
    assert!(
        matches!(
            store.upsert_message(&message(&actor, &foreign)).await,
            Err(StoreError::NotFound)
        ),
        "a message cannot reference another account's mailbox"
    );
    assert_eq!(store.count_messages_in_mailbox(&foreign).await.unwrap(), 0);
    for n in 0..12 {
        let own = mailbox(&store, &actor, &format!("race-{n}")).await;
        store.upsert_message(&message(&actor, &own)).await.unwrap();
        let row = message(&actor, &own);
        let (write, delete) = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            tokio::join!(store.upsert_message(&row), store.delete_mailbox(&own))
        })
        .await
        .expect("delete/write lock ordering must not deadlock");
        assert!(
            write.is_ok() || matches!(write, Err(StoreError::NotFound)),
            "{write:?}"
        );
        delete.unwrap();
        assert!(matches!(
            store.get_mailbox(&own).await,
            Err(StoreError::NotFound)
        ));
        assert_eq!(store.count_messages_in_mailbox(&own).await.unwrap(), 0);
    }
}
#[tokio::test]
async fn mailbox_owner_and_delete_races_sqlite() {
    exercise(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn mailbox_owner_and_delete_races_postgres() {
    if let Ok(url) = std::env::var("MW_TEST_PG") {
        exercise(Store::open(&url, ServerKey::generate()).await.unwrap()).await;
    }
}
