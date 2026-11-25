use super::*;
use crate::backend::Dialect;
use crate::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, StoredMeta, backend::q,
};
use std::collections::BTreeSet;

const SQLITE_MIGRATION: &str = include_str!("../../migrations/0116_index_publication_journal.sql");
const PG_MIGRATION: &str = include_str!("../../migrations_pg/0116_index_publication_journal.sql");
// t50hp-e32 slice 1: 0126 redefines the messages trigger without flags_json and
// drops the message_meta triggers (keywords, pin, snooze, follow-up are SQL-only).
const SQLITE_MB: &str = include_str!("../../migrations/0126_flags_leave_index_journal.sql");
const PG_MB: &str = include_str!("../../migrations_pg/0126_flags_leave_index_journal.sql");

/// The column list the installed triggers compare for `table`: messages from
/// 0126, message_meta none (0126 dropped its triggers), the lease from 0116.
fn effective_columns(postgres: bool, table: &str) -> BTreeSet<String> {
    match table {
        "messages" => journaled_columns(if postgres { PG_MB } else { SQLITE_MB }, table),
        "message_meta" => BTreeSet::new(),
        _ => journaled_columns(if postgres { PG_MIGRATION } else { SQLITE_MIGRATION }, table),
    }
}

// ── fixture ──────────────────────────────────────────────────────────────────

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
async fn mailbox(store: &Store, account: &str, name: &str) -> String {
    store
        .upsert_mailbox(&MailboxUpsert {
            account_id: account,
            name,
            role: None,
            uidvalidity: 1,
            uidnext: 100,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        })
        .await
        .unwrap()
}
async fn message(store: &Store, account: &str, mailbox: &str, uid: u32) -> String {
    let body = store
        .put_body(account, b"From: a@example.test\r\nSubject: s\r\n\r\nbody")
        .await
        .unwrap();
    store
        .upsert_message(&MessageUpsert {
            account_id: account,
            mailbox_id: mailbox,
            uid,
            uidvalidity: 1,
            message_id: Some("<fixture@example.test>"),
            thread_id: None,
            internaldate: Some("2020-01-01T00:00:00Z"),
            size: 40,
            flags_json: "[]",
            envelope: Some(b"{}"),
            blob_ref: Some(&body),
        })
        .await
        .unwrap()
}
/// A Message with no children (meta/lease/list envelope), for key rewrites.
async fn bare(store: &Store, account: &str, mailbox: &str, id: &str, uid: i64) {
    q("INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity) VALUES(?1,?2,?3,?4,1)")
        .bind(id)
        .bind(account)
        .bind(mailbox)
        .bind(uid)
        .execute(&store.backend)
        .await
        .unwrap();
}
/// Enable capture. PostgreSQL refuses the Store API by design, so its parity
/// leg writes the control row directly to exercise the same triggers.
async fn enable(store: &Store) {
    match store.backend.dialect() {
        Dialect::Sqlite => store.set_index_journal_enabled(true).await.unwrap(),
        Dialect::Postgres => {
            q("INSERT INTO index_publication_journal_control(id,enabled) VALUES(1,1) ON CONFLICT(id) DO UPDATE SET enabled=1")
                .execute(&store.backend)
                .await
                .unwrap();
        }
    }
    assert!(store.index_journal_enabled().await.unwrap());
}
async fn all(store: &Store) -> Vec<IndexJournalEntry> {
    store
        .index_journal_page(0, MAX_INDEX_JOURNAL_BATCH)
        .await
        .unwrap()
}
/// Rows appended by `op`, in seq order.
async fn appended<F: std::future::Future<Output = ()>>(
    store: &Store,
    op: F,
) -> Vec<(String, String, IndexJournalMode)> {
    let before = store.index_journal_max_seq().await.unwrap();
    op.await;
    store
        .index_journal_page(before, MAX_INDEX_JOURNAL_BATCH)
        .await
        .unwrap()
        .into_iter()
        .map(|e| (e.account_id, e.stable_id, e.mode))
        .collect()
}
fn keys(rows: &[(String, String, IndexJournalMode)]) -> BTreeSet<(String, String)> {
    rows.iter()
        .map(|(a, s, _)| (a.clone(), s.clone()))
        .collect()
}
fn key(account: &str, id: &str) -> (String, String) {
    (account.to_owned(), id.to_owned())
}

// ── contract (both dialects) ─────────────────────────────────────────────────

async fn off_by_default(store: &Store) {
    assert!(!store.index_journal_enabled().await.unwrap());
    let a = account(store, "off@example.test").await;
    let inbox = mailbox(store, &a, "INBOX").await;
    let id = message(store, &a, &inbox, 1).await;
    store.set_thread(&id, "t-off").await.unwrap();
    store
        .upsert_message_meta(
            &id,
            &StoredMeta {
                pinned: true,
                snoozed_until: None,
                follow_up_at: None,
            },
        )
        .await
        .unwrap();
    store.delete_message(&id).await.unwrap();
    // An explicit disabled row behaves like an absent one.
    store.set_index_journal_enabled(false).await.unwrap();
    let id = message(store, &a, &inbox, 2).await;
    store.set_thread(&id, "t-off-2").await.unwrap();
    assert!(!store.index_journal_enabled().await.unwrap());
    assert_eq!(store.index_journal_len().await.unwrap(), 0);
    assert_eq!(store.index_journal_max_seq().await.unwrap(), 0);
}

/// Every column the trigger lists, changed alone by raw SQL, journals the new key
/// (and the old key as a delete when the key itself moves). Exempt columns do not.
async fn raw_column_updates(store: &Store) {
    let a = account(store, "raw@example.test").await;
    let b = account(store, "raw-b@example.test").await;
    let inbox = mailbox(store, &a, "INBOX").await;
    let other = mailbox(store, &a, "Archive").await;
    let id = message(store, &a, &inbox, 10).await;
    enable(store).await;
    let cases: &[(&str, &'static str)] = &[
        (
            "mailbox_id",
            "UPDATE messages SET mailbox_id=?2 WHERE stable_id=?1",
        ),
        ("uid", "UPDATE messages SET uid=uid+100 WHERE stable_id=?1"),
        (
            "uidvalidity",
            "UPDATE messages SET uidvalidity=uidvalidity+1 WHERE stable_id=?1",
        ),
        (
            "message_id",
            "UPDATE messages SET message_id='<changed@example.test>' WHERE stable_id=?1",
        ),
        (
            "thread_id",
            "UPDATE messages SET thread_id='raw-thread' WHERE stable_id=?1",
        ),
        (
            "internaldate",
            "UPDATE messages SET internaldate='2001-02-03T04:05:06Z' WHERE stable_id=?1",
        ),
        ("size", "UPDATE messages SET size=size+1 WHERE stable_id=?1"),
        (
            "envelope_json",
            "UPDATE messages SET envelope_json=NULL WHERE stable_id=?1",
        ),
        (
            "blob_ref",
            "UPDATE messages SET blob_ref=NULL WHERE stable_id=?1",
        ),
        (
            "content_revision",
            "UPDATE messages SET content_revision='raw-content' WHERE stable_id=?1",
        ),
    ];
    let mut covered = BTreeSet::new();
    for (column, sql) in cases {
        let mut statement = q(sql).bind(id.as_str());
        if sql.contains("?2") {
            statement = statement.bind(other.as_str());
        }
        let rows = appended(store, async {
            assert_eq!(statement.execute(&store.backend).await.unwrap(), 1);
        })
        .await;
        assert_eq!(
            keys(&rows),
            BTreeSet::from([key(&a, &id)]),
            "{column}: {rows:?}"
        );
        covered.insert(*column);
    }
    // Key rewrites journal the old key as a delete and the new key.
    let moved = "raw-move-account";
    bare(store, &a, &inbox, moved, 500).await;
    let rows = appended(store, async {
        q("UPDATE messages SET account_id=?2 WHERE stable_id=?1")
            .bind(moved)
            .bind(b.as_str())
            .execute(&store.backend)
            .await
            .unwrap();
    })
    .await;
    assert!(
        rows.contains(&(a.clone(), moved.into(), IndexJournalMode::Delete)),
        "{rows:?}"
    );
    assert!(keys(&rows).contains(&key(&b, moved)), "{rows:?}");
    covered.insert("account_id");
    bare(store, &a, &inbox, "raw-old-id", 501).await;
    let rows = appended(store, async {
        q("UPDATE messages SET stable_id='raw-new-id' WHERE stable_id='raw-old-id'")
            .execute(&store.backend)
            .await
            .unwrap();
    })
    .await;
    assert!(
        rows.contains(&(a.clone(), "raw-old-id".into(), IndexJournalMode::Delete)),
        "{rows:?}"
    );
    assert!(keys(&rows).contains(&key(&a, "raw-new-id")), "{rows:?}");
    covered.insert("stable_id");
    let covered: BTreeSet<String> = covered.into_iter().map(str::to_owned).collect();
    assert_eq!(covered, effective_columns(false, "messages"));

    // A flag change journals nothing (0126: flags are read from SQL only).
    let rows = appended(store, async {
        q("UPDATE messages SET flags_json='[\"Seen\"]' WHERE stable_id=?1")
            .bind(id.as_str())
            .execute(&store.backend)
            .await
            .unwrap();
    })
    .await;
    assert!(rows.is_empty(), "flags_json: {rows:?}");

    // cache_revision is advanced by 0105 on every UPDATE; alone it journals nothing.
    let rows = appended(store, async {
        q("UPDATE messages SET cache_revision='raw-cache' WHERE stable_id=?1")
            .bind(id.as_str())
            .execute(&store.backend)
            .await
            .unwrap();
    })
    .await;
    assert!(rows.is_empty(), "{rows:?}");

    // message_meta journals nothing (0126): pin, snooze and follow-up are SQL-only.
    let rows = appended(store, async {
        q("INSERT INTO message_meta(stable_id,pinned) VALUES(?1,0)")
            .bind(id.as_str())
            .execute(&store.backend)
            .await
            .unwrap();
    })
    .await;
    assert!(rows.is_empty(), "{rows:?}");
    for sql in [
        "UPDATE message_meta SET pinned=1 WHERE stable_id=?1",
        "UPDATE message_meta SET snoozed_until='2030-01-01T00:00:00Z' WHERE stable_id=?1",
        "UPDATE message_meta SET follow_up_at='2030-01-01T00:00:00Z' WHERE stable_id=?1",
        "DELETE FROM message_meta WHERE stable_id=?1",
    ] {
        let rows = appended(store, async {
            q(sql)
                .bind(id.as_str())
                .execute(&store.backend)
                .await
                .unwrap();
        })
        .await;
        assert!(rows.is_empty(), "{sql}: {rows:?}");
    }
    bare(store, &a, &inbox, "meta-old", 502).await;
    bare(store, &a, &inbox, "meta-new", 503).await;
    q("INSERT INTO message_meta(stable_id,pinned) VALUES('meta-old',1)")
        .execute(&store.backend)
        .await
        .unwrap();
    let rows = appended(store, async {
        q("UPDATE message_meta SET stable_id='meta-new' WHERE stable_id='meta-old'")
            .execute(&store.backend)
            .await
            .unwrap();
    })
    .await;
    assert!(rows.is_empty(), "{rows:?}");

    // Body content: 0110 advances content_revision on the Message.
    bare(store, &a, &inbox, "body-late", 504).await;
    q("UPDATE messages SET blob_ref='raw-blob' WHERE stable_id='body-late'")
        .execute(&store.backend)
        .await
        .unwrap();
    for (n, sql) in [
        "INSERT INTO bodies(blob_ref,account_id,sealed_bytes) VALUES('raw-blob',?1,?2)",
        "UPDATE bodies SET sealed_bytes=?2 WHERE blob_ref='raw-blob' AND account_id=?1",
        "DELETE FROM bodies WHERE blob_ref='raw-blob' AND account_id=?1 AND octet_length(?2)>0",
    ]
    .into_iter()
    .enumerate()
    {
        let rows = appended(store, async {
            q(sql)
                .bind(a.as_str())
                .bind(vec![1u8, 2, 3, n as u8])
                .execute(&store.backend)
                .await
                .unwrap();
        })
        .await;
        assert_eq!(
            keys(&rows),
            BTreeSet::from([key(&a, "body-late")]),
            "{sql}: {rows:?}"
        );
    }

    // Lease: matters only while the Message has no blob_ref; renewal never journals.
    bare(store, &a, &inbox, "leased", 505).await;
    let lease_rows = appended(store, async {
        q("INSERT INTO message_body_cache_lease(stable_id,account_id,blob_ref,uidvalidity,uid,last_used_at) VALUES('leased',?1,'lease-blob',1,505,1)")
            .bind(a.as_str())
            .execute(&store.backend)
            .await
            .unwrap();
    })
    .await;
    assert_eq!(keys(&lease_rows), BTreeSet::from([key(&a, "leased")]));
    let rows = appended(store, async {
        q("UPDATE message_body_cache_lease SET last_used_at=last_used_at+100 WHERE stable_id='leased'")
            .execute(&store.backend)
            .await
            .unwrap();
        store.renew_cached_body_use(&a, "leased").await.unwrap();
    })
    .await;
    assert!(rows.is_empty(), "lease renewal journaled: {rows:?}");
    for sql in [
        "UPDATE message_body_cache_lease SET blob_ref='lease-blob-2' WHERE stable_id='leased'",
        "UPDATE message_body_cache_lease SET uid=999 WHERE stable_id='leased'",
        "UPDATE message_body_cache_lease SET uidvalidity=2 WHERE stable_id='leased'",
        "UPDATE message_body_cache_lease SET account_id=?1 WHERE stable_id='leased'",
        "DELETE FROM message_body_cache_lease WHERE stable_id='leased'",
    ] {
        let mut statement = q(sql);
        if sql.contains("?1") {
            statement = statement.bind(b.as_str());
        }
        let rows = appended(store, async {
            statement.execute(&store.backend).await.unwrap();
        })
        .await;
        assert_eq!(keys(&rows), BTreeSet::from([key(&a, "leased")]), "{sql}");
    }
    // With a blob present the lease does not change the content identity.
    let with_blob = message(store, &a, &inbox, 20).await;
    let rows = appended(store, async {
        assert_eq!(
            store
                .register_refetchable_cached_body(&a, &with_blob)
                .await
                .unwrap(),
            1
        );
    })
    .await;
    assert!(rows.is_empty(), "{rows:?}");
}

/// Store APIs that write indexed state, each shown to journal its key.
async fn api_write_paths(store: &Store) {
    let a = account(store, "api@example.test").await;
    let inbox = mailbox(store, &a, "INBOX").await;
    let other = mailbox(store, &a, "Archive").await;
    enable(store).await;
    let mut id = String::new();
    let rows = appended(store, async {
        id = message(store, &a, &inbox, 1).await;
    })
    .await;
    eprintln!(
        "journal rows for put_body+upsert_message insert: {}",
        rows.len()
    );
    assert_eq!(keys(&rows), BTreeSet::from([key(&a, &id)]));
    // Re-ingest with changed flags goes through the ON CONFLICT DO UPDATE path.
    let rows = appended(store, async {
        store
            .upsert_message(&MessageUpsert {
                account_id: &a,
                mailbox_id: &inbox,
                uid: 1,
                uidvalidity: 1,
                message_id: Some("<fixture@example.test>"),
                thread_id: None,
                internaldate: Some("2020-01-01T00:00:00Z"),
                size: 40,
                flags_json: "[\"\\\\Seen\"]",
                envelope: None,
                blob_ref: None,
            })
            .await
            .unwrap();
    })
    .await;
    // A re-ingest that changes only the flags journals nothing since 0126.
    assert!(rows.is_empty(), "{rows:?}");
    // Hot path cost: a flag change journals nothing (0126: SQL-only metadata).
    let rows = appended(store, async {
        store
            .set_flags_batch(&[(id.clone(), "[\"\\\\Flagged\"]".into())])
            .await
            .unwrap();
    })
    .await;
    assert!(rows.is_empty(), "{rows:?}");
    let current = store.get_message(&id).await.unwrap();
    let rows = appended(store, async {
        assert!(store.set_flags_if_current(&current, "[]").await.unwrap());
    })
    .await;
    assert!(rows.is_empty(), "{rows:?}");
    let rows = appended(store, async {
        store.set_thread(&id, "api-thread").await.unwrap()
    })
    .await;
    assert_eq!(
        rows,
        vec![(a.clone(), id.clone(), IndexJournalMode::Metadata)]
    );
    let rows = appended(store, async {
        store.relocate_message(&id, &other, 7, 1).await.unwrap();
    })
    .await;
    assert_eq!(keys(&rows), BTreeSet::from([key(&a, &id)]));
    let current = store.get_message(&id).await.unwrap();
    let rows = appended(store, async {
        assert!(
            store
                .relocate_message_if_current(&current, &inbox, 8, 1)
                .await
                .unwrap()
        );
    })
    .await;
    assert_eq!(keys(&rows), BTreeSet::from([key(&a, &id)]));
    let rows = appended(store, async {
        store
            .upsert_message_meta(
                &id,
                &StoredMeta {
                    pinned: true,
                    snoozed_until: None,
                    follow_up_at: None,
                },
            )
            .await
            .unwrap();
        store
            .upsert_message_meta(
                &id,
                &StoredMeta {
                    pinned: true,
                    snoozed_until: None,
                    follow_up_at: Some("2030-01-01T00:00:00Z".into()),
                },
            )
            .await
            .unwrap();
    })
    .await;
    // Pin and follow-up journal nothing since 0126.
    assert!(rows.is_empty(), "{rows:?}");
    let current = store.get_message(&id).await.unwrap();
    let rows = appended(store, async {
        assert!(store.delete_message_if_current(&current).await.unwrap());
    })
    .await;
    assert!(
        rows.contains(&(a.clone(), id.clone(), IndexJournalMode::Delete)),
        "{rows:?}"
    );
    let second = message(store, &a, &inbox, 2).await;
    let rows = appended(store, async {
        store.delete_message(&second).await.unwrap()
    })
    .await;
    assert!(
        rows.contains(&(a.clone(), second.clone(), IndexJournalMode::Delete)),
        "{rows:?}"
    );
    // Bulk: mailbox deletion and account deletion reach every Message by cascade.
    let in_other = [
        message(store, &a, &other, 3).await,
        message(store, &a, &other, 4).await,
    ];
    let rows = appended(store, async { store.delete_mailbox(&other).await.unwrap() }).await;
    for m in &in_other {
        assert!(
            rows.contains(&(a.clone(), m.clone(), IndexJournalMode::Delete)),
            "{rows:?}"
        );
    }
    let last = message(store, &a, &inbox, 5).await;
    let rows = appended(store, async {
        q("DELETE FROM accounts WHERE id=?1")
            .bind(a.as_str())
            .execute(&store.backend)
            .await
            .unwrap();
    })
    .await;
    assert!(
        rows.contains(&(a.clone(), last.clone(), IndexJournalMode::Delete)),
        "{rows:?}"
    );
}

async fn rollback_and_acknowledgement(store: &Store) {
    let a = account(store, "ack@example.test").await;
    let b = account(store, "ack-b@example.test").await;
    let inbox = mailbox(store, &a, "INBOX").await;
    let inbox_b = mailbox(store, &b, "INBOX").await;
    let id = message(store, &a, &inbox, 1).await;
    let other = message(store, &b, &inbox_b, 1).await;
    enable(store).await;
    let before = store.index_journal_len().await.unwrap();
    let mut tx = store.backend.begin().await.unwrap();
    q("UPDATE messages SET flags_json='[\"rolled-back\"]' WHERE stable_id=?1")
        .bind(id.as_str())
        .execute_tx(&mut tx)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    assert_eq!(store.index_journal_len().await.unwrap(), before);

    store.set_thread(&id, "one").await.unwrap();
    store.set_thread(&other, "one").await.unwrap();
    // The publisher captures upto, then a newer change lands for the same key.
    let upto = store.index_journal_max_seq().await.unwrap();
    store.set_thread(&id, "two").await.unwrap();
    let newer = store.index_journal_max_seq().await.unwrap();
    assert!(newer > upto);
    let deleted = store
        .index_journal_complete(&[(a.as_str(), id.as_str(), upto)])
        .await
        .unwrap();
    assert!(deleted >= 1);
    let left = all(store).await;
    assert!(
        left.iter().any(|e| e.stable_id == id && e.seq == newer),
        "newer row kept"
    );
    assert!(left.iter().all(|e| e.stable_id != id || e.seq > upto));
    assert!(
        left.iter().any(|e| e.stable_id == other),
        "other key untouched"
    );
    // Wrong account for the key deletes nothing.
    assert_eq!(
        store
            .index_journal_complete(&[(b.as_str(), id.as_str(), newer)])
            .await
            .unwrap(),
        0
    );
    // Paging is seq ordered and resumes after the last seq.
    let first = store.index_journal_page(0, 1).await.unwrap();
    let rest = store
        .index_journal_page(first[0].seq, MAX_INDEX_JOURNAL_BATCH)
        .await
        .unwrap();
    assert_eq!(
        first.len() + rest.len(),
        store.index_journal_len().await.unwrap() as usize
    );
    assert!(rest.windows(2).all(|w| w[0].seq < w[1].seq));
    assert!(rest.iter().all(|e| e.seq > first[0].seq));
    // Purge removes one account only.
    assert!(store.index_journal_purge_account(&b).await.unwrap() >= 1);
    assert!(all(store).await.iter().all(|e| e.account_id != b));
    assert!(all(store).await.iter().any(|e| e.account_id == a));
    // complete_all trims up to the captured seq and keeps later rows.
    let cut = store.index_journal_max_seq().await.unwrap();
    store.set_thread(&id, "three").await.unwrap();
    store.index_journal_complete_all(cut).await.unwrap();
    let left = all(store).await;
    assert_eq!(left.len(), 1);
    assert!(left[0].seq > cut);
    // seq never goes backwards after the table is emptied (AUTOINCREMENT/identity).
    let high = left[0].seq;
    store.index_journal_complete_all(high).await.unwrap();
    assert_eq!(store.index_journal_len().await.unwrap(), 0);
    assert_eq!(store.index_journal_max_seq().await.unwrap(), 0);
    store.set_thread(&id, "four").await.unwrap();
    assert!(all(store).await[0].seq > high);
    // Input limits.
    assert!(store.index_journal_page(0, 0).await.is_err());
    assert!(
        store
            .index_journal_page(0, MAX_INDEX_JOURNAL_BATCH + 1)
            .await
            .is_err()
    );
    assert!(store.index_journal_complete(&[("", "x", 1)]).await.is_err());
    assert_eq!(store.index_journal_complete(&[]).await.unwrap(), 0);
    // Disabling stops capture but keeps rows for replay.
    let kept = store.index_journal_len().await.unwrap();
    q("UPDATE index_publication_journal_control SET enabled=0")
        .execute(&store.backend)
        .await
        .unwrap();
    store.set_thread(&id, "five").await.unwrap();
    assert_eq!(store.index_journal_len().await.unwrap(), kept);
}

async fn contract(store: Store) {
    off_by_default(&store).await;
    raw_column_updates(&store).await;
    api_write_paths(&store).await;
    rollback_and_acknowledgement(&store).await;
}

#[tokio::test]
async fn index_journal_sqlite_contract() {
    contract(
        Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap(),
    )
    .await;
}

#[tokio::test]
async fn index_journal_postgres_contract() {
    use sqlx::Connection;
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        mw_test_gate::skip("MW_TEST_PG absent; index journal PostgreSQL contract not executed");
        return;
    };
    let schema = format!("index_journal_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&mut admin)
        .await
        .unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    let store = Store::open(scoped.as_str(), crate::ServerKey::generate())
        .await
        .unwrap();
    let refused = store.set_index_journal_enabled(true).await;
    assert!(
        matches!(refused, Err(StoreError::Invalid(_))),
        "{refused:?}"
    );
    assert!(!store.index_journal_enabled().await.unwrap());
    contract(store).await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&mut admin)
        .await
        .unwrap();
}

#[tokio::test]
async fn index_journal_seq_is_commit_ordered_on_a_file_database() {
    // Two pooled connections on a real WAL file: the publisher's capture must not
    // cover a write that commits after it, whichever connection performs it.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("journal.db");
    let store = Store::open(&path.to_string_lossy(), crate::ServerKey::generate())
        .await
        .unwrap();
    let a = account(&store, "file@example.test").await;
    let inbox = mailbox(&store, &a, "INBOX").await;
    let id = message(&store, &a, &inbox, 1).await;
    store.set_index_journal_enabled(true).await.unwrap();
    let mut tx = store.backend.begin().await.unwrap();
    q("UPDATE messages SET thread_id='in-flight' WHERE stable_id=?1")
        .bind(id.as_str())
        .execute_tx(&mut tx)
        .await
        .unwrap();
    let upto = store.index_journal_max_seq().await.unwrap();
    tx.commit().await.unwrap();
    store
        .index_journal_complete(&[(a.as_str(), id.as_str(), upto)])
        .await
        .unwrap();
    let left = all(&store).await;
    assert!(
        left.iter().any(|e| e.stable_id == id && e.seq > upto),
        "{left:?}"
    );
}

// ── column-coverage parity ───────────────────────────────────────────────────

/// `alias.column` references in an SQL text.
fn references(sql: &str, alias: &str) -> BTreeSet<String> {
    let needle = format!("{alias}.");
    let bytes = sql.as_bytes();
    let mut out = BTreeSet::new();
    let mut from = 0;
    while let Some(at) = sql[from..].find(&needle) {
        let start = from + at;
        from = start + needle.len();
        if start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
            continue;
        }
        let column: String = sql[from..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !column.is_empty() {
            out.insert(column);
        }
    }
    out
}
/// The column list a migration's UPDATE condition compares for `table`, read from
/// the trigger text (SQLite `IS NOT`, PostgreSQL `IS DISTINCT FROM`).
fn journaled_columns(migration: &str, table: &str) -> BTreeSet<String> {
    // A Windows checkout with core.autocrlf=true embeds CRLF via include_str!.
    let migration = &migration.replace("\r\n", "\n");
    let sqlite_marker = format!("AFTER UPDATE ON {table}\n");
    let condition = if let Some(at) = migration.find(&sqlite_marker) {
        let rest = &migration[at + sqlite_marker.len()..];
        rest[..rest.find("BEGIN").unwrap()].to_owned()
    } else {
        let function = format!("ON {table} FOR EACH ROW EXECUTE FUNCTION ");
        let call = migration
            .find(&function)
            .unwrap_or_else(|| panic!("no trigger for {table}"));
        let name: String = migration[call + function.len()..]
            .chars()
            .take_while(|c| *c != '(')
            .collect();
        let body = &migration[migration
            .find(&format!("CREATE FUNCTION {name}()"))
            .or_else(|| migration.find(&format!("CREATE OR REPLACE FUNCTION {name}()")))
            .unwrap()..];
        let body = &body[body.find("IF NOT (").unwrap()..];
        body[..body.find(") THEN").unwrap()].to_owned()
    };
    let new = references(&condition, "NEW");
    assert_eq!(
        new,
        references(&condition, "OLD"),
        "{table}: NEW/OLD lists differ"
    );
    new
}
fn live_columns(table: &str, rows: &[(String, String)]) -> BTreeSet<String> {
    rows.iter()
        .filter(|(t, _)| t == table)
        .map(|(_, c)| c.clone())
        .collect()
}
fn set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|v| (*v).to_owned()).collect()
}

/// Fails when the index source reader reads a column, or a source table gains a
/// column, that the journal triggers neither compare nor exempt with a reason.
#[tokio::test]
async fn index_journal_trigger_columns_cover_the_index_source_reader() {
    // Exempt columns and why they are safe to leave out of the UPDATE condition.
    // 0105 bumps cache_revision after every UPDATE; flags_json is read from SQL by
    // every metadata query (0125 projection) and is only a display value in the
    // index since 0126.
    let exempt_messages = set(&["cache_revision", "flags_json"]);
    // message_meta (pin, snooze, follow-up) is SQL-only since 0126: every column exempt.
    let exempt_meta = set(&["stable_id", "pinned", "snoozed_until", "follow_up_at"]);
    let exempt_lease = set(&["last_used_at"]); // renewal timestamp, never read by the reader
    // Account-scope relations (accounts a, zeroaccess_accounts z, account_restriction ar)
    // are not per-message state; they are reported as a gap, not journaled here.
    let account_scope = set(&["accounts", "zeroaccess_accounts", "account_restriction"]);

    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let tables: BTreeSet<String> = q("SELECT name FROM sqlite_master WHERE type='table'")
        .fetch_all(&store.backend)
        .await
        .unwrap()
        .iter()
        .map(|r| r.get_string("name"))
        .collect();
    let mut columns = Vec::new();
    for table in [
        "messages",
        "message_meta",
        "message_body_cache_lease",
        "bodies",
    ] {
        for row in q("SELECT name FROM pragma_table_info(?1)")
            .bind(table)
            .fetch_all(&store.backend)
            .await
            .unwrap()
        {
            columns.push((table.to_owned(), row.get_string("name")));
        }
    }
    let journaled_m = effective_columns(false, "messages");
    let journaled_mm = effective_columns(false, "message_meta");
    let journaled_l = effective_columns(false, "message_body_cache_lease");

    let mut reader_tables = BTreeSet::new();
    for dialect in [Dialect::Sqlite, Dialect::Postgres] {
        for body in [true, false] {
            let sql = super::super::publication_sql_variant(dialect, body);
            for (alias, table) in [
                ("m", "messages"),
                ("mm", "message_meta"),
                ("l", "message_body_cache_lease"),
                ("b", "bodies"),
            ] {
                if alias != "b" || body {
                    assert!(
                        sql.contains(&format!("{table} {alias} ")),
                        "alias {alias} moved"
                    );
                }
            }
            for word in ["FROM ", "JOIN "] {
                for (at, _) in sql.match_indices(word) {
                    let name: String = sql[at + word.len()..]
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    if tables.contains(&name) {
                        reader_tables.insert(name);
                    }
                }
            }
            let read_m = references(sql, "m");
            let read_mm = references(sql, "mm");
            let read_l = references(sql, "l");
            let read_b = references(sql, "b");
            let missing: Vec<_> = read_m
                .difference(&journaled_m)
                .filter(|c| !exempt_messages.contains(*c))
                .collect();
            assert!(
                missing.is_empty(),
                "reader reads messages columns not journaled: {missing:?}"
            );
            assert!(
                read_mm.difference(&journaled_mm).all(|c| exempt_meta.contains(c)),
                "meta: {read_mm:?} vs {journaled_mm:?}"
            );
            let missing: Vec<_> = read_l
                .difference(&journaled_l)
                .filter(|c| !exempt_lease.contains(*c))
                .collect();
            assert!(
                missing.is_empty(),
                "reader reads lease columns not journaled: {missing:?}"
            );
            // bodies reach the journal through 0110's content_revision bump.
            if body {
                assert_eq!(read_b, set(&["account_id", "blob_ref", "sealed_bytes"]));
            } else {
                assert!(read_b.is_empty());
            }
        }
    }
    let mut expected_tables = set(&[
        "messages",
        "message_meta",
        "message_body_cache_lease",
        "bodies",
    ]);
    expected_tables.extend(account_scope.iter().cloned());
    assert_eq!(
        reader_tables, expected_tables,
        "the reader joins a table the journal does not classify"
    );

    // Every live column of a journaled table is compared or exempt (catches a new column).
    let mut all_m = journaled_m.clone();
    all_m.extend(exempt_messages.iter().cloned());
    assert_eq!(live_columns("messages", &columns), all_m);
    let mut all_mm = journaled_mm.clone();
    all_mm.extend(exempt_meta.iter().cloned());
    assert_eq!(live_columns("message_meta", &columns), all_mm);
    let mut all_l = journaled_l.clone();
    all_l.extend(exempt_lease.iter().cloned());
    assert_eq!(live_columns("message_body_cache_lease", &columns), all_l);
    assert!(
        journaled_m.contains("content_revision"),
        "body changes need content_revision"
    );
    // The body hooks that advance content_revision must cover every bodies column read.
    let body_hooks =
        q("SELECT sql FROM sqlite_master WHERE type='trigger' AND name='verdict_body_update'")
            .fetch_all(&store.backend)
            .await
            .unwrap();
    let hook = body_hooks[0].get_string("sql");
    assert!(references(&hook, "NEW").is_superset(&set(&[
        "account_id",
        "blob_ref",
        "sealed_bytes"
    ])));
    for name in [
        "verdict_body_insert",
        "verdict_body_delete",
        "message_cache_revision_update",
    ] {
        let n = q("SELECT CAST(COUNT(*) AS BIGINT) AS n FROM sqlite_master WHERE type='trigger' AND name=?1")
            .bind(name)
            .fetch_scalar_i64(&store.backend)
            .await
            .unwrap();
        assert_eq!(n, 1, "{name} missing");
    }
    // Installed triggers are the migration's (a rebuild migration that drops them fails here).
    let installed: BTreeSet<String> =
        q("SELECT name FROM sqlite_master WHERE type='trigger' AND name LIKE 'index_journal_%'")
            .fetch_all(&store.backend)
            .await
            .unwrap()
            .iter()
            .map(|r| r.get_string("name"))
            .collect();
    assert_eq!(installed.len(), 6, "{installed:?}");
}

#[test]
fn index_journal_postgres_triggers_compare_the_same_columns() {
    for table in ["messages", "message_meta", "message_body_cache_lease"] {
        assert_eq!(
            effective_columns(false, table),
            effective_columns(true, table),
            "{table}"
        );
    }
}

#[test]
fn index_journal_entry_debug_hides_identity() {
    let entry = IndexJournalEntry {
        seq: 3,
        account_id: "secret-account".into(),
        stable_id: "secret-message".into(),
        mode: IndexJournalMode::Reconcile,
    };
    let text = format!("{entry:?}");
    assert!(!text.contains("secret"), "{text}");
}
