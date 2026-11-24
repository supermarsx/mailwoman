//! t54: the per-`(account, type)` change state must be allocated so that
//! concurrent writers never share a state and never commit a state lower than
//! one a reader has already observed.
//!
//! Every change log (`changes`, `pim_changes`, `crypto_changes`) computes the
//! next state as `MAX(state) + 1` inside the INSERT. SQLite has one writer, so
//! the maximum is always read after the previous writer committed. PostgreSQL
//! under READ COMMITTED gives each statement its own snapshot and has no unique
//! index on `(account_id, type, state)`, so without a lock two concurrent
//! writers can compute the same next state, and a writer that computed a lower
//! state can commit after one that computed a higher state.
//!
//! Each scenario runs concurrent writers against one account while a reader
//! polls one change log the way `*/changes` does (`state > since`). It asserts:
//! * every state belongs to exactly one write batch (one `record_*` call, one
//!   ingested message, or one synced deletion);
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

/// Ingest one new message (records Email `created` + Mailbox `updated`).
async fn ingest(store: &Store, account: &str, mailbox: &str, uid: u32, label: &str) -> String {
    let message_id = format!("<{label}@example.org>");
    let raw = format!("Message-ID: {message_id}\r\nSubject: {label}\r\n\r\nbody {label}\r\n");
    store
        .ingest_message_with_body(
            &MessageUpsert {
                account_id: account,
                mailbox_id: mailbox,
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
        .unwrap()
        .stable_id
}

/// What a writer does in one round.
#[derive(Clone, Copy, PartialEq)]
enum Writer {
    /// Alternate `record_change` and `record_changes` (2–3 ids).
    Record,
    /// `ingest_message_with_body` of a new message.
    Ingest,
    /// `delete_synced_message_if_source` of a message seeded before the race.
    DeleteSynced,
    /// `record_pim_change` (CalendarEvent).
    Pim,
    /// `record_crypto_change` (MailRule).
    Crypto,
}

/// The change log the reader polls.
#[derive(Clone, Copy)]
enum Log {
    Email,
    Pim,
    Crypto,
}

const PIM_TYPE: &str = "CalendarEvent";
const CRYPTO_TYPE: &str = "MailRule";

/// `(state, object id, op)` rows of `log`, oldest first.
async fn read_log(store: &Store, account: &str, log: Log) -> Vec<(u64, String, String)> {
    match log {
        Log::Email => store
            .changes_since(account, "Email", 0)
            .await
            .unwrap()
            .into_iter()
            .map(|r| (r.state, r.stable_id, r.op))
            .collect(),
        Log::Pim => store
            .pim_changes_since(account, PIM_TYPE, 0)
            .await
            .unwrap()
            .into_iter()
            .map(|r| (r.state, r.object_id, r.op))
            .collect(),
        Log::Crypto => store
            .crypto_changes_since(account, CRYPTO_TYPE, 0)
            .await
            .unwrap()
            .into_iter()
            .map(|r| (r.state, r.object_id, r.op))
            .collect(),
    }
}

struct Outcome {
    /// States carried by rows of more than one write batch.
    duplicates: Vec<String>,
    /// Rows first seen at a state at or below one an earlier poll had seen.
    late: Vec<String>,
    rows: usize,
    polls: usize,
}

type Labels = Arc<std::sync::Mutex<HashMap<(String, String), String>>>;

fn label(labels: &Labels, id: &str, op: &str, batch: &str) {
    labels
        .lock()
        .unwrap()
        .insert((id.to_string(), op.to_string()), batch.to_string());
}

/// Run `plan` writers concurrently on `stores` (round-robin) against one
/// account, with a reader polling `log` until they finish.
async fn race(stores: &[Store], plan: &[Writer], log: Log) -> Outcome {
    let store = &stores[0];
    let acct = account(store).await;
    let mbox = mailbox(store, &acct).await;
    let done = Arc::new(AtomicBool::new(false));
    // (object id, op) -> batch label.
    let labels: Labels = Default::default();

    // Messages for the synced-deletion writers, ingested serially up front.
    let mut seeded: HashMap<usize, Vec<String>> = HashMap::new();
    for (w, kind) in plan.iter().enumerate() {
        if *kind == Writer::DeleteSynced {
            for i in 0..ROUNDS {
                let seed = format!("seed-w{w}-r{i}");
                let uid = (100_000 + w * ROUNDS + i) as u32;
                let id = ingest(store, &acct, &mbox, uid, &seed).await;
                label(&labels, &id, "created", &seed);
                seeded.entry(w).or_default().push(id);
            }
        }
    }

    let reader = {
        let store = stores[stores.len() - 1].clone();
        let acct = acct.clone();
        let done = done.clone();
        tokio::spawn(async move {
            let mut seen: HashSet<(u64, String, String)> = HashSet::new();
            let mut high = 0u64;
            let mut late = Vec::new();
            let mut polls = 0usize;
            loop {
                let finished = done.load(Ordering::SeqCst);
                let rows = read_log(&store, &acct, log).await;
                polls += 1;
                let mut poll_high = high;
                for row in rows {
                    if seen.insert(row.clone()) {
                        if polls > 1 && row.0 <= high {
                            let relation = if row.0 < high { "below" } else { "at" };
                            late.push(format!(
                                "{} {} appeared at state {} ({relation}) after the reader had seen state {}",
                                row.1, row.2, row.0, high
                            ));
                        }
                        poll_high = poll_high.max(row.0);
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
        let mine = seeded.remove(&w).unwrap_or_default();
        tasks.push(tokio::spawn(async move {
            for i in 0..ROUNDS {
                let batch = format!("w{w}-r{i}");
                match kind {
                    Writer::Record if i % 2 == 0 => {
                        let id = format!("{batch}-0");
                        label(&labels, &id, "updated", &batch);
                        store
                            .record_change(&acct, "Email", &id, "updated")
                            .await
                            .unwrap();
                    }
                    Writer::Record => {
                        let ids: Vec<String> =
                            (0..2 + i % 2).map(|k| format!("{batch}-{k}")).collect();
                        for id in &ids {
                            label(&labels, id, "updated", &batch);
                        }
                        store
                            .record_changes(&acct, "Email", &ids, "updated")
                            .await
                            .unwrap();
                    }
                    Writer::Ingest => {
                        let uid = (w * ROUNDS + i + 1) as u32;
                        let id = ingest(&store, &acct, &mbox, uid, &batch).await;
                        label(&labels, &id, "created", &batch);
                    }
                    Writer::DeleteSynced => {
                        let id = mine.get(i).expect("seeded message");
                        label(&labels, id, "destroyed", &batch);
                        let current = store.get_message(id).await.unwrap();
                        assert!(
                            store
                                .delete_synced_message_if_source(&current)
                                .await
                                .unwrap(),
                            "seeded message {id} was deleted"
                        );
                    }
                    Writer::Pim => {
                        let id = format!("{batch}-0");
                        label(&labels, &id, "updated", &batch);
                        store
                            .record_pim_change(&acct, PIM_TYPE, &id, "updated")
                            .await
                            .unwrap();
                    }
                    Writer::Crypto => {
                        let id = format!("{batch}-0");
                        label(&labels, &id, "updated", &batch);
                        store
                            .record_crypto_change(&acct, CRYPTO_TYPE, &id, "updated")
                            .await
                            .unwrap();
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

    let rows = read_log(store, &acct, log).await;
    let labels = labels.lock().unwrap();
    let mut by_state: BTreeMap<u64, HashSet<String>> = BTreeMap::new();
    for (state, id, op) in &rows {
        let batch = labels
            .get(&(id.clone(), op.clone()))
            .cloned()
            .unwrap_or_else(|| format!("unlabelled:{id}:{op}"));
        by_state.entry(*state).or_default().insert(batch);
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
        "[t54] {name}: {} rows, {} reader polls, {} shared states, {} late rows ({} strictly below)",
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

/// Synced deletions of one mailbox serialize among themselves on the mailbox
/// row lock; a `record_change` writer and an ingest writer among them make any
/// shared state a cross-path one.
fn delete_plan() -> Vec<Writer> {
    let mut plan = vec![Writer::DeleteSynced; WRITERS - 2];
    plan.push(Writer::Record);
    plan.push(Writer::Ingest);
    plan
}

fn pim_plan() -> Vec<Writer> {
    vec![Writer::Pim; WRITERS]
}

fn crypto_plan() -> Vec<Writer> {
    vec![Writer::Crypto; WRITERS]
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

/// One scenario on PostgreSQL (4 stores x 5 connections: 16 writers plus the
/// reader each get a connection) and on SQLite.
macro_rules! scenario {
    ($pg:ident, $sqlite:ident, $name:literal, $plan:expr, $log:expr) => {
        #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
        async fn $pg() {
            with_pg(concat!("postgres ", $name), 4, |stores| async move {
                assert_clean(
                    concat!("postgres ", $name),
                    race(&stores, &$plan, $log).await,
                );
            })
            .await;
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
        async fn $sqlite() {
            let dir = tempfile::tempdir().unwrap();
            let store = sqlite_store(&dir).await;
            assert_clean(
                concat!("sqlite ", $name),
                race(&[store], &$plan, $log).await,
            );
        }
    };
}

scenario!(
    postgres_concurrent_record_changes_get_distinct_ordered_states,
    sqlite_concurrent_record_changes_get_distinct_ordered_states,
    "record",
    record_plan(),
    Log::Email
);
scenario!(
    postgres_ingest_and_record_change_share_one_state_sequence,
    sqlite_ingest_and_record_change_share_one_state_sequence,
    "ingest+record",
    mixed_plan(),
    Log::Email
);
scenario!(
    postgres_synced_delete_shares_the_mail_state_sequence,
    sqlite_synced_delete_shares_the_mail_state_sequence,
    "delete-synced+record+ingest",
    delete_plan(),
    Log::Email
);
scenario!(
    postgres_concurrent_pim_changes_get_distinct_ordered_states,
    sqlite_concurrent_pim_changes_get_distinct_ordered_states,
    "pim",
    pim_plan(),
    Log::Pim
);
scenario!(
    postgres_concurrent_crypto_changes_get_distinct_ordered_states,
    sqlite_concurrent_crypto_changes_get_distinct_ordered_states,
    "crypto",
    crypto_plan(),
    Log::Crypto
);
