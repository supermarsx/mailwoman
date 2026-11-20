use super::*;
use crate::{AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount};
async fn account(store: &Store, name: &str) -> String {
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "fixture.invalid",
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
async fn message(store: &Store, account: &str, mailbox: &str, uid: u32) -> Message {
    let id = store
        .upsert_message(&MessageUpsert {
            account_id: account,
            mailbox_id: mailbox,
            uid,
            uidvalidity: 1,
            message_id: None,
            thread_id: None,
            internaldate: None,
            size: 1,
            flags_json: "[]",
            envelope: None,
            blob_ref: None,
        })
        .await
        .unwrap();
    store.get_message(&id).await.unwrap()
}
async fn contract(store: Store) {
    let a = account(&store, "a@fixture.invalid").await;
    let b = account(&store, "b@fixture.invalid").await;
    let inbox = mailbox(&store, &a, "INBOX").await;
    let archive = mailbox(&store, &a, "Archive").await;
    let foreign = mailbox(&store, &b, "INBOX").await;
    let initial = message(&store, &a, &inbox, 1).await;
    assert!(!store.delete_mailbox_if_empty(&a, &inbox).await.unwrap());
    assert!(
        !store
            .relocate_message_if_current(&initial, &foreign, 1, 1)
            .await
            .unwrap()
    );
    assert!(
        store
            .set_flags_if_current(&initial, "[\"Seen\"]")
            .await
            .unwrap()
    );
    assert!(!store.delete_message_if_current(&initial).await.unwrap());
    assert!(
        !store
            .relocate_message_if_current(&initial, &archive, 2, 1)
            .await
            .unwrap()
    );
    assert!(!store.set_flags_if_current(&initial, "[]").await.unwrap());
    let current = store.get_message(&initial.stable_id).await.unwrap();
    let mut fake = current.clone();
    fake.blob_ref = Some("wrong-source".into());
    assert!(!store.delete_message_if_current(&fake).await.unwrap());
    assert!(
        !store
            .relocate_message_if_current(&fake, &archive, 2, 1)
            .await
            .unwrap()
    );
    assert!(!store.set_flags_if_current(&fake, "[]").await.unwrap());
    let history_before = q("SELECT COUNT(*) AS n FROM changes WHERE account_id=?1")
        .bind(&a)
        .fetch_one(&store.backend)
        .await
        .unwrap()
        .get_i64("n");
    assert!(
        store
            .relocate_message_if_current(&current, &archive, 2, 1)
            .await
            .unwrap()
    );
    let moved = store.get_message(&initial.stable_id).await.unwrap();
    assert_eq!(moved.mailbox_id, archive);
    assert!(!store.delete_message_if_current(&current).await.unwrap());
    assert_eq!(store.get_mailbox(&inbox).await.unwrap().total, 0);
    assert_eq!(store.get_mailbox(&archive).await.unwrap().total, 1);
    assert!(store.delete_mailbox_if_empty(&a, &inbox).await.unwrap());
    assert!(store.get_mailbox(&inbox).await.is_err());
    assert!(!store.delete_mailbox_if_empty(&a, &archive).await.unwrap());
    // Same stable ID reassigned by another authoritative writer stays untouched.
    q("UPDATE messages SET account_id=?1,mailbox_id=?2 WHERE stable_id=?3")
        .bind(&b)
        .bind(&foreign)
        .bind(&moved.stable_id)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(!store.delete_message_if_current(&moved).await.unwrap());
    assert!(
        !store
            .relocate_message_if_current(&moved, &archive, 3, 1)
            .await
            .unwrap()
    );
    assert!(!store.set_flags_if_current(&moved, "[]").await.unwrap());
    let foreign_source = store.get_message(&moved.stable_id).await.unwrap();
    assert_eq!(foreign_source.account_id, b);
    assert!(
        store
            .delete_message_if_current(&foreign_source)
            .await
            .unwrap()
    );
    let history_after = q("SELECT COUNT(*) AS n FROM changes WHERE account_id=?1")
        .bind(&a)
        .fetch_one(&store.backend)
        .await
        .unwrap()
        .get_i64("n");
    assert_eq!(
        history_before, history_after,
        "source CAS helpers leave change history to their caller"
    );
    assert!(store.delete_mailbox_if_empty(&b, &foreign).await.unwrap());
    store.close().await;
}
#[tokio::test]
async fn source_cas_mutations_sqlite_preserve_changed_and_foreign_rows() {
    contract(
        Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap(),
    )
    .await;
}
#[tokio::test]
async fn source_cas_mutations_postgres_preserve_changed_and_foreign_rows() {
    use sqlx::Connection;
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        eprintln!("SKIP: MW_TEST_PG absent");
        return;
    };
    let schema = format!("publication_mutations_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&mut admin)
        .await
        .unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    contract(
        Store::open(scoped.as_str(), crate::ServerKey::generate())
            .await
            .unwrap(),
    )
    .await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&mut admin)
        .await
        .unwrap();
}
