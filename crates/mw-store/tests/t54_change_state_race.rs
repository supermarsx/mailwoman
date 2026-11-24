//! t54: the per-`(account, type)` change state must be allocated so that
//! concurrent writers never share a state and never commit a state lower than
//! one a reader has already observed.
//!
//! `record_change` / `record_changes` (and the ingest path) compute the next
//! state as `MAX(state) + 1` inside the INSERT. SQLite has one writer, so the
//! maximum is always read after the previous writer committed. PostgreSQL under
//! READ COMMITTED gives each statement its own snapshot and has no unique index
//! on `(account_id, type, state)`, so without a lock two concurrent writers can
//! compute the same next state, and a writer that computed a lower state can
//! commit after one that computed a higher state.
//!
//! Each scenario runs concurrent writers against one account while a reader
//! polls the change log the way `Email/changes` does (`state > since`). It
//! asserts:
//! * every state belongs to exactly one write batch (one `record_change` call,
//!   one `record_changes` call, or one ingested message);
//! * no row becomes visible at a state at or below the highest state the reader
//!   had already seen in an earlier poll (that row would be missed by a client
//!   that synced to the higher state).
//!
//! PostgreSQL legs run when `MW_TEST_PG` (or `DATABASE_URL_PG`) is set, each in
//! its own schema which is dropped afterwards.
use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const WRITERS: usize = 16;
const ROUNDS: usize = 40;

async fn account(store: &Store) -> String {
    let user = format!("t54-{}@example.org", rand::random::<u64>());
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: &user,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: user.clone(),
                password: "fixture".into(),
            },
        )
        .await
        .unwrap()
}

async fn mailbox(store: &Store, account: &str) -> String {
    store
        .upsert_mailbox(&MailboxUpsert {
            account_id: account,
            name: "INBOX",
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

/// What a writer does in one round.
#[derive(Clone, Copy)]
enum Writer {
    /// Alternate `record_change` and `record_changes` (2–3 ids).
    Record,
    /// `ingest_message_with_body` of a new message (records Email + Mailbox).
    Ingest,
}

struct Outcome {
    /// States carried by rows of more than one write batch.
    duplicates: Vec<String>,
    /// Rows first seen at a state at or below one an earlier poll had seen.
    late: Vec<String>,
    rows: usize,
    polls: usize,
}

/// Run `plan` writers concurrently on `stores` (round-robin) against one
/// account, with a reader polling the Email change log until they finish.
async fn race(stores: &[Store], plan: &[Writer]) -> Outcome {
    let store = &stores[0];
    let acct = account(store).await;
    let mbox = mailbox(store, &acct).await;
    let done = Arc::new(AtomicBool::new(false));
    // stable_id -> batch label. Record batches name their own ids; ingest
    // stable ids are minted by the store and reported back.
    let labels: Arc<std::sync::Mutex<HashMap<String, String>>> = Default::default();

    let reader = {
        let store = stores[stores.len() - 1].clone();
        let acct = acct.clone();
        let done = done.clone();
        tokio::spawn(async move {
            let mut seen: HashSet<(u64, String)> = HashSet::new();
            let mut high = 0u64;
            let mut late = Vec::new();
            let mut polls = 0usize;
            loop {
                let finished = done.load(Ordering::SeqCst);
                let rows = store.changes_since(&acct, "Email", 0).await.unwrap();
                polls += 1;
                let mut poll_high = high;
                for row in rows {
                    if seen.insert((row.state, row.stable_id.clone())) {
                        if polls > 1 && row.state <= high {
                            let relation = if row.state < high { "below" } else { "at" };
                            late.push(format!(
                                "{} appeared at state {} ({relation}) after the reader had seen state {}",
                                row.stable_id, row.state, high
                            ));
                        }
                        poll_high = poll_high.max(row.state);
                    }
                }
                high = poll_high;
                if finished {
                    return (late, polls);
                }
                tokio::task::yield_now().await;
            }
        })
    };

    let mut tasks = Vec::new();
    for (w, kind) in plan.iter().copied().enumerate() {
        let store = stores[w % stores.len()].clone();
        let acct = acct.clone();
        let mbox = mbox.clone();
        let labels = labels.clone();
        tasks.push(tokio::spawn(async move {
            for i in 0..ROUNDS {
                let label = format!("w{w}-r{i}");
                match kind {
                    Writer::Record if i % 2 == 0 => {
                        let id = format!("{label}-0");
                        labels.lock().unwrap().insert(id.clone(), label);
                        store
                            .record_change(&acct, "Email", &id, "updated")
                            .await
                            .unwrap();
                    }
                    Writer::Record => {
                        let ids: Vec<String> =
                            (0..2 + i % 2).map(|k| format!("{label}-{k}")).collect();
                        for id in &ids {
                            labels.lock().unwrap().insert(id.clone(), label.clone());
                        }
                        store
                            .record_changes(&acct, "Email", &ids, "updated")
                            .await
                            .unwrap();
                    }
                    Writer::Ingest => {
                        let uid = (w * ROUNDS + i + 1) as u32;
                        let message_id = format!("<{label}@example.org>");
                        let raw = format!(
                            "Message-ID: {message_id}\r\nSubject: {label}\r\n\r\nbody {label}\r\n"
                        );
                        let outcome = store
                            .ingest_message_with_body(
                                &MessageUpsert {
                                    account_id: &acct,
                                    mailbox_id: &mbox,
                                    uid,
                                    uidvalidity: 1,
                                    message_id: Some(&message_id),
                                    thread_id: None,
                                    internaldate: None,
                                    size: raw.len() as u64,
                                    flags_json: "[]",
                                    envelope: None,
                                    blob_ref: None,
                                },
                                raw.as_bytes(),
                            )
                            .await
                            .unwrap();
                        labels.lock().unwrap().insert(outcome.stable_id, label);
                    }
                }
            }
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }
    done.store(true, Ordering::SeqCst);
    let (late, polls) = reader.await.unwrap();

    let rows = store.changes_since(&acct, "Email", 0).await.unwrap();
    let labels = labels.lock().unwrap();
    let mut by_state: BTreeMap<u64, HashSet<String>> = BTreeMap::new();
    for row in &rows {
        let label = labels
            .get(&row.stable_id)
            .cloned()
            .unwrap_or_else(|| format!("unlabelled:{}", row.stable_id));
        by_state.entry(row.state).or_default().insert(label);
    }
    let duplicates = by_state
        .iter()
        .filter(|(_, batches)| batches.len() > 1)
        .map(|(state, batches)| {
            let mut batches: Vec<_> = batches.iter().cloned().collect();
            batches.sort();
            format!("state {state} shared by {batches:?}")
        })
        .collect();
    Outcome {
        duplicates,
        late,
        rows: rows.len(),
        polls,
    }
}

fn assert_clean(name: &str, outcome: Outcome) {
    eprintln!(
        "[t54] {name}: {} Email rows, {} reader polls, {} shared states, {} late rows ({} strictly below)",
        outcome.rows,
        outcome.polls,
        outcome.duplicates.len(),
        outcome.late.len(),
        outcome
            .late
            .iter()
            .filter(|line| line.contains("(below)"))
            .count()
    );
    for line in outcome.duplicates.iter().take(5) {
        eprintln!("[t54] {name}: {line}");
    }
    for line in outcome
        .late
        .iter()
        .filter(|line| line.contains("(below)"))
        .take(3)
    {
        eprintln!("[t54] {name}: {line}");
    }
    for line in outcome.late.iter().take(5) {
        eprintln!("[t54] {name}: {line}");
    }
    assert!(
        outcome.duplicates.is_empty() && outcome.late.is_empty(),
        "{name}: {} states shared by different write batches, {} rows became visible below a state already observed",
        outcome.duplicates.len(),
        outcome.late.len()
    );
}

fn record_plan() -> Vec<Writer> {
    vec![Writer::Record; WRITERS]
}

/// Ingest writers serialize among themselves on the account lock; one
/// `record_change` writer among them makes any shared state a cross-path one.
fn mixed_plan() -> Vec<Writer> {
    let mut plan = vec![Writer::Ingest; WRITERS - 1];
    plan.push(Writer::Record);
    plan
}

fn pg_dsn() -> Option<String> {
    std::env::var("MW_TEST_PG")
        .ok()
        .or_else(|| std::env::var("DATABASE_URL_PG").ok())
        .filter(|dsn| !dsn.is_empty())
}

/// Open `count` independent stores (each with its own pool) on a fresh schema,
/// run `body`, then drop the schema.
async fn with_pg<F, Fut>(name: &str, count: usize, body: F)
where
    F: FnOnce(Vec<Store>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send,
{
    let Some(dsn) = pg_dsn() else {
        eprintln!("[t54] SKIP {name}: MW_TEST_PG and DATABASE_URL_PG unset");
        return;
    };
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    let schema = format!("t54_{}", rand::random::<u64>());
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await
        .unwrap();
    let sep = if dsn.contains('?') { '&' } else { '?' };
    let scoped = format!("{dsn}{sep}options=-c%20search_path%3D{schema}");
    let key = ServerKey::generate();
    let result = tokio::spawn(async move {
        let mut stores = Vec::with_capacity(count);
        for _ in 0..count {
            stores.push(Store::open_postgres(&scoped, key.clone()).await.unwrap());
        }
        body(stores).await;
    })
    .await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&admin)
        .await
        .unwrap();
    result.unwrap();
}

async fn sqlite_store(dir: &tempfile::TempDir) -> Store {
    let path = dir.path().join("t54.db");
    Store::open(path.to_str().unwrap(), ServerKey::generate())
        .await
        .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn postgres_concurrent_record_changes_get_distinct_ordered_states() {
    // 4 stores x 5 connections: 16 writers plus the reader each get a connection.
    with_pg("postgres record", 4, |stores| async move {
        assert_clean("postgres record", race(&stores, &record_plan()).await);
    })
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn postgres_ingest_and_record_change_share_one_state_sequence() {
    with_pg("postgres ingest+record", 4, |stores| async move {
        assert_clean("postgres ingest+record", race(&stores, &mixed_plan()).await);
    })
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn sqlite_concurrent_record_changes_get_distinct_ordered_states() {
    let dir = tempfile::tempdir().unwrap();
    let store = sqlite_store(&dir).await;
    assert_clean("sqlite record", race(&[store], &record_plan()).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn sqlite_ingest_and_record_change_share_one_state_sequence() {
    let dir = tempfile::tempdir().unwrap();
    let store = sqlite_store(&dir).await;
    assert_clean("sqlite ingest+record", race(&[store], &mixed_plan()).await);
}
