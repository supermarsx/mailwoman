//! H14 diagnostic: the effective default page cache, the size of a fixture-like
//! database, and where page-cache misses on a hot read mix come from (cache size
//! versus commits by other connections). Counters are SQLite pager events, not
//! disk reads; nothing here is a throughput claim.
use crate::{
    Credentials, ServerKey, Store,
    backend::Backend,
    cache::{AccountKind, MailboxUpsert, MessageUpsert, NewAccount},
    encryption::{DbKey, EncryptionConfig, EncryptionMode},
};
use sqlx::{SqliteConnection, sqlite::SqliteConnectOptions};

const SESSION_READ: &str = "SELECT id,account_id,username,jmap_url,api_url,sealed_creds,last_seen FROM sessions WHERE id=?1 AND linked_internal=0";
const MAILBOX_READ: &str = "SELECT * FROM mailboxes WHERE account_id=?1";
const LIST_READ: &str = "SELECT stable_id,envelope_json FROM messages WHERE account_id=?1 ORDER BY internaldate DESC LIMIT 25";
const TOUCH: &str = "UPDATE sessions SET last_seen=?2 WHERE id=?1";

async fn encrypted_store(dir: &std::path::Path) -> Store {
    let config = EncryptionConfig::with_key(EncryptionMode::Optional, DbKey::generate());
    Store::open_with_encryption(
        dir.join("working-set.db").to_str().unwrap(),
        ServerKey::generate(),
        &config,
    )
    .await
    .unwrap()
}

async fn main_cache_size(connection: &mut SqliteConnection) -> i64 {
    crate::database_observability::sqlite_pragma_i64(connection, "PRAGMA main.cache_size")
        .await
        .unwrap()
}

/// Default configuration: no `cache_size` is set, so every foreground connection
/// runs with the bundled SQLCipher default of 2000 KiB.
#[tokio::test]
async fn default_foreground_cache_is_the_driver_default_2000_kib() {
    let dir = tempfile::tempdir().unwrap();
    let store = encrypted_store(dir.path()).await;
    let Backend::Sqlite(live) = &store.backend else {
        unreachable!()
    };
    let mut connection = live.acquire().await.unwrap();
    assert_eq!(main_cache_size(&mut connection).await, -2000);
    let page_size =
        crate::database_observability::sqlite_pragma_i64(&mut connection, "PRAGMA page_size")
            .await
            .unwrap();
    assert_eq!(page_size, 4096, "SQLCipher 4 default page size");
    drop(connection);
    store.close().await;
}

struct Seeded {
    sessions: Vec<(String, String)>,
}
async fn seed(store: &Store, users: usize, messages: usize) -> Seeded {
    let envelope = serde_json::json!({
        "subject": "A subject line of ordinary length for a list view",
        "from": [{"name": "Sender Name", "email": "sender@example.test"}],
        "to": [{"name": "Recipient", "email": "user@example.test"}],
        "preview": "x".repeat(256),
    })
    .to_string();
    let mut sessions = Vec::new();
    for user in 0..users {
        let username = format!("user{user}@example.test");
        let account = store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "mail.example.test",
                    port: 993,
                    tls: "implicit",
                    username: &username,
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: username.clone(),
                    password: "secret".into(),
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
                uidnext: messages as u32 + 1,
                highestmodseq: 0,
                total: messages as u32,
                unread: 0,
                parent_id: None,
            })
            .await
            .unwrap();
        for uid in 1..=messages as u32 {
            let date = format!("2026-11-{:02}T10:{:02}:00+00:00", 1 + uid % 28, uid % 60);
            let message_id = format!("<{user}.{uid}@example.test>");
            store
                .upsert_message(&MessageUpsert {
                    account_id: &account,
                    mailbox_id: &mailbox,
                    uid,
                    uidvalidity: 1,
                    message_id: Some(&message_id),
                    thread_id: None,
                    internaldate: Some(&date),
                    size: 4096,
                    flags_json: "[]",
                    envelope: Some(envelope.as_bytes()),
                    blob_ref: None,
                })
                .await
                .unwrap();
        }
        let session = store
            .create_session(
                &account,
                &username,
                "https://mail.example.test/jmap",
                "https://mail.example.test/api",
                &Credentials {
                    username: username.clone(),
                    password: "secret".into(),
                },
            )
            .await
            .unwrap();
        sessions.push((session, account));
    }
    Seeded { sessions }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Leg {
    cache_kib: i64,
    commits_between_requests: bool,
    requests: usize,
    misses_per_request: f64,
    hits_per_request: f64,
}

async fn leg(
    reader: &mut SqliteConnection,
    writer: &mut SqliteConnection,
    seeded: &Seeded,
    cache_kib: i64,
    commits: bool,
) -> Leg {
    sqlx::query(&format!("PRAGMA cache_size=-{cache_kib}"))
        .execute(&mut *reader)
        .await
        .unwrap();
    // Warm pass over every user, then the measured pass.
    for pass in 0..2 {
        let before = mw_sqlite_status::cache_status(reader).await.unwrap();
        for (session, account) in &seeded.sessions {
            if commits {
                sqlx::query(TOUCH)
                    .bind(session)
                    .bind(chrono::Utc::now().to_rfc3339())
                    .execute(&mut *writer)
                    .await
                    .unwrap();
            }
            sqlx::query(SESSION_READ).bind(session).fetch_optional(&mut *reader).await.unwrap();
            sqlx::query(MAILBOX_READ).bind(account).fetch_all(&mut *reader).await.unwrap();
            sqlx::query(LIST_READ).bind(account).fetch_all(&mut *reader).await.unwrap();
        }
        let after = mw_sqlite_status::cache_status(reader).await.unwrap();
        if pass == 1 {
            let n = seeded.sessions.len() as f64;
            return Leg {
                cache_kib,
                commits_between_requests: commits,
                requests: seeded.sessions.len(),
                misses_per_request: (after.misses.unwrap() - before.misses.unwrap()) as f64 / n,
                hits_per_request: (after.hits.unwrap() - before.hits.unwrap()) as f64 / n,
            };
        }
    }
    unreachable!()
}

/// Report-only. Run with
/// `cargo test -p mw-store --lib page_cache_working_set -- --ignored --nocapture`.
#[tokio::test]
#[ignore = "diagnostic: seeds 100 users x 50 messages and prints pager counters"]
async fn page_cache_working_set_and_commit_resets() {
    let dir = tempfile::tempdir().unwrap();
    let store = encrypted_store(dir.path()).await;
    let seeded = seed(&store, 100, 50).await;
    let Backend::Sqlite(live) = &store.backend else {
        unreachable!()
    };
    let options: SqliteConnectOptions = (*live.pin().await.active_options()).clone();
    let mut reader = <SqliteConnection as sqlx::Connection>::connect_with(&options).await.unwrap();
    let mut writer = <SqliteConnection as sqlx::Connection>::connect_with(&options).await.unwrap();
    let pages = crate::database_observability::sqlite_pragma_i64(&mut reader, "PRAGMA page_count")
        .await
        .unwrap();
    let mut legs = Vec::new();
    for cache_kib in [2000, 16384, 65536] {
        for commits in [false, true] {
            legs.push(leg(&mut reader, &mut writer, &seeded, cache_kib, commits).await);
        }
    }
    sqlx::Connection::close(reader).await.unwrap();
    sqlx::Connection::close(writer).await.unwrap();
    eprintln!(
        "page-cache-working-set {}",
        serde_json::json!({
            "users": 100, "messagesPerUser": 50, "pageSize": 4096,
            "databasePages": pages, "databaseBytes": pages * 4096,
            "defaultCacheKiB": 2000, "legs": legs,
        })
    );
    store.close().await;
}
