//! t50hp-e19: single-message flag writes, `MW_SQLITE_WRITER=gate` against
//! `group`. Explicit, owned encrypted tempfile store; never part of the normal
//! suite. Run once per mode (the mode is read once per process):
//!
//! ```powershell
//! $env:MW_SQLITE_WRITER_BENCH='1'; $env:MW_SQLITE_WRITER='gate'   # then 'group'
//! cargo test --locked --release -p mw-store --lib sqlite_writer_bench -- --ignored --exact `
//!   sqlite_live::sqlite_writer::benchmark::flag_write_concurrency_ladder --test-threads=1 --nocapture
//! ```
//!
//! Each level runs `C` tasks for a fixed time; every task repeatedly writes the
//! flags of one message of its own (`Store::set_flags`, the single-message
//! Email/set path: one write job = old-state read, flag UPDATE with the 0125
//! projection triggers, unread counter UPDATE). Reported per level: writes/s,
//! commits/s (gate: one commit per write; group: the writer's commit counter),
//! p50/p99 of one write as its caller sees it.
use crate::encryption::{DbKey, EncryptionConfig, EncryptionMode};
use crate::{AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store};
use std::sync::Arc;
use std::time::{Duration, Instant};

const MESSAGES: usize = 512;
const WARMUP: Duration = Duration::from_millis(500);
const MEASURE: Duration = Duration::from_secs(4);

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "explicit owned SQLCipher diagnostic; set MW_SQLITE_WRITER_BENCH=1"]
async fn flag_write_concurrency_ladder() {
    assert_eq!(std::env::var("MW_SQLITE_WRITER_BENCH").as_deref(), Ok("1"));
    let mode = crate::sqlite_live::write_gate::configured_mode().unwrap();
    // Optional batch window for group mode (the production constant otherwise).
    let window = std::env::var("MW_SQLITE_WRITER_BENCH_BATCH_MICROS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    super::TEST_MAX_BATCH_MICROS.store(window, std::sync::atomic::Ordering::Relaxed);
    let window = if window == 0 {
        super::MAX_BATCH_TIME.as_micros() as u64
    } else {
        window
    };
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("writer-benchmark.sqlite");
    let store = Store::open_with_encryption(
        path.to_str().unwrap(),
        ServerKey::generate(),
        &EncryptionConfig::with_key(EncryptionMode::Required, DbKey::generate()),
    )
    .await
    .unwrap();
    let crate::Backend::Sqlite(live) = &store.backend else {
        panic!("SQLite required")
    };
    assert!(live.is_encrypted());
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "benchmark.invalid",
                port: 993,
                tls: "implicit",
                username: "benchmark",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "benchmark".into(),
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
            uidnext: MESSAGES as u32 + 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        })
        .await
        .unwrap();
    let mut ids = Vec::new();
    for uid in 1..=MESSAGES {
        ids.push(
            store
                .upsert_message(&MessageUpsert {
                    account_id: &account,
                    mailbox_id: &mailbox,
                    uid: uid as u32,
                    uidvalidity: 1,
                    message_id: None,
                    thread_id: None,
                    internaldate: Some("2026-11-01T00:00:00Z"),
                    size: 1024,
                    flags_json: "[]",
                    envelope: Some(br#"{"subject":"benchmark"}"#),
                    blob_ref: None,
                })
                .await
                .unwrap(),
        );
    }
    let ids = Arc::new(ids);
    for concurrency in [1usize, 8, 64, 256] {
        let stop_warmup = Instant::now() + WARMUP;
        let stop = stop_warmup + MEASURE;
        let writer_before = live.group_writer().map(|w| w.stats());
        let mut tasks = Vec::new();
        for worker in 0..concurrency {
            let (store, ids) = (store.clone(), ids.clone());
            tasks.push(tokio::spawn(async move {
                let id = &ids[worker % ids.len()];
                let mut samples = Vec::new();
                let mut seen = false;
                let mut measured_from = None;
                loop {
                    let now = Instant::now();
                    if now >= stop {
                        break;
                    }
                    seen = !seen;
                    let flags = if seen { r#"["Seen"]"# } else { "[]" };
                    store.set_flags(id, flags).await.unwrap();
                    if now >= stop_warmup {
                        measured_from.get_or_insert(now);
                        samples.push(now.elapsed().as_micros() as u64);
                    }
                }
                samples
            }));
        }
        // Writer counters at the end of warmup (group mode).
        tokio::time::sleep_until(tokio::time::Instant::from_std(stop_warmup)).await;
        let writer_start = live.group_writer().map(|w| w.stats()).or(writer_before);
        let mut samples: Vec<u64> = Vec::new();
        for task in tasks {
            samples.extend(task.await.unwrap());
        }
        let writer_end = live.group_writer().map(|w| w.stats());
        samples.sort_unstable();
        let pick = |p: usize| samples[(samples.len() - 1) * p / 100];
        let seconds = MEASURE.as_secs_f64();
        let writes = samples.len() as f64;
        let commits = match (writer_start, writer_end) {
            (Some(a), Some(b)) => (b.commits - a.commits) as f64,
            _ => writes,
        };
        println!(
            "SQLITE_WRITER_BENCH {}",
            serde_json::json!({
                "kind": "flag-write-ladder-v1",
                "mode": mode.as_str(),
                "batchWindowMicros": window,
                "concurrency": concurrency,
                "seconds": seconds,
                "writes": samples.len(),
                "writesPerSecond": writes / seconds,
                "commitsPerSecond": commits / seconds,
                "writesPerCommit": if commits > 0.0 { writes / commits } else { 0.0 },
                "p50Micros": pick(50),
                "p99Micros": pick(99),
                "maxMicros": samples[samples.len() - 1],
                "writer": writer_end,
                "profile": if cfg!(debug_assertions) { "debug_assertions_enabled" } else { "debug_assertions_disabled" },
                "scope": "owned_encrypted_tempfile_store_flag_writes_not_http_capacity",
            })
        );
    }
    // Every counter is still exact after the ladder.
    let audit = store.audit_message_projection(None).await.unwrap();
    assert!(audit.is_clean(), "{audit:?}");
    store.close().await;
}

// ---- t62 N14: grouped mutation capacity of one SQLite writer ------------------------------
//
// $env:MW_SQLITE_WRITER_BENCH='1'
// cargo test --locked --release -p mw-store --lib -- --ignored --exact
//   sqlite_live::sqlite_writer::benchmark::grouped_mutation_capacity --test-threads=1 --nocapture
//
// Encrypted (SQLCipher, verified: is_encrypted, cipher_version, file header is not
// "SQLite format 3" after close), all migrations incl. the 0125 projection triggers. A
// private writer with batch limits 1024 jobs / 60 s drives the same group-commit code as
// production; N tasks submit one job each, so a batch holds up to N jobs.
// Job bodies repeat the SQLite statements of the store paths (set_flags_batch,
// relocate_message_if_current, record_changes) so each statement can be timed; the
// store functions themselves are covered by the tests.

use crate::backend::{Backend, q, write_job};
use crate::sqlite_live::LiveSqlite;
use std::collections::BTreeMap;
use std::sync::Mutex;

const BENCH_MESSAGES: usize = 2048;
const CHANGE_SQL: &str = "INSERT INTO changes (account_id, type, state, stable_id, op, at)
     SELECT ?1, ?2, s.next, j.value, ?4, ?5
     FROM (SELECT COALESCE(MAX(state), 0) + 1 AS next FROM (
           SELECT MAX(state) AS state FROM changes WHERE account_id=?1 AND type=?2
           UNION ALL SELECT head_state AS state FROM change_log_checkpoint WHERE account_id=?1 AND type=?2
           ) heads) s,
          json_each(?3) j
     RETURNING state";

#[derive(Default)]
struct Split(Mutex<BTreeMap<&'static str, (u64, u64)>>);
impl Split {
    fn add(&self, label: &'static str, since: Instant) {
        let mut map = self.0.lock().unwrap();
        let entry = map.entry(label).or_default();
        entry.0 += 1;
        entry.1 += since.elapsed().as_micros() as u64;
    }
    fn take(&self) -> BTreeMap<&'static str, (u64, u64)> {
        std::mem::take(&mut *self.0.lock().unwrap())
    }
}

struct World {
    backend: Backend,
    writer: Arc<super::GroupWriter>,
    account: String,
    boxes: [String; 2],
    ids: Vec<String>,
    /// Current mailbox index per message (moves alternate).
    place: Mutex<Vec<usize>>,
    split: Split,
}

async fn flag_job(w: &World, i: usize, round: usize) -> Result<(), crate::StoreError> {
    let id = &w.ids[i];
    let flags = if round % 2 == 0 { r#"["Seen"]"# } else { "[]" };
    let now = "2026-11-25T00:00:00Z";
    write_job!(w.backend, super::Lane::Normal, |tx| async {
        let t = Instant::now();
        let rows = q("SELECT stable_id, account_id, mailbox_id, flags_json FROM messages
                 WHERE stable_id IN (SELECT value FROM json_each(?1))")
        .bind(format!("[\"{id}\"]"))
        .fetch_all_tx(tx)
        .await?;
        w.split.add("flag: SELECT old state", t);
        let row = &rows[0];
        let was = crate::cache::flags_json_is_unread(&row.get_string("flags_json"));
        let mailbox = row.get_string("mailbox_id");
        let t = Instant::now();
        q("UPDATE messages SET flags_json = j.f
                 FROM (SELECT json_extract(value, '$[0]') AS id, json_extract(value, '$[1]') AS f FROM json_each(?1)) j
                 WHERE messages.stable_id = j.id")
        .bind(serde_json::to_string(&[[id.as_str(), flags]]).unwrap())
        .execute_tx(tx)
        .await?;
        w.split.add("flag: UPDATE messages (+triggers)", t);
        let now_unread = crate::cache::flags_json_is_unread(flags);
        if was != now_unread {
            let t = Instant::now();
            q("UPDATE mailboxes SET unread = CASE WHEN unread + ?2 < 0 THEN 0 ELSE unread + ?2 END WHERE id = ?1")
                .bind(&mailbox)
                .bind(if now_unread { 1i64 } else { -1 })
                .execute_tx(tx)
                .await?;
            w.split.add("flag: UPDATE mailboxes unread", t);
        }
        let t = Instant::now();
        q(CHANGE_SQL).bind(&w.account).bind("Email").bind(format!("[\"{id}\"]")).bind("updated").bind(now).fetch_all_tx(tx).await?;
        w.split.add("flag: INSERT changes Email", t);
        let t = Instant::now();
        q(CHANGE_SQL).bind(&w.account).bind("Mailbox").bind(format!("[\"{mailbox}\"]")).bind("updated").bind(now).fetch_all_tx(tx).await?;
        w.split.add("flag: INSERT changes Mailbox", t);
        Ok::<_, crate::StoreError>(())
    })
}

async fn move_job(w: &World, i: usize, round: usize) -> Result<(), crate::StoreError> {
    let id = &w.ids[i];
    let from = w.place.lock().unwrap()[i];
    let to = 1 - from;
    let (source, destination) = (&w.boxes[from], &w.boxes[to]);
    let uid = 1_000_000 + (round * BENCH_MESSAGES + i) as i64;
    let now = "2026-11-25T00:00:00Z";
    let result = write_job!(w.backend, super::Lane::Normal, |tx| async {
        let t = Instant::now();
        crate::publication_mutations::lock_mailboxes(tx, &w.account, source, destination).await?;
        w.split.add("move: SELECT lock mailboxes", t);
        let t = Instant::now();
        let rows = q("UPDATE messages SET mailbox_id=?3, uid=?4 WHERE stable_id=?1 AND account_id=?2 RETURNING flags_json")
            .bind(id.as_str())
            .bind(&w.account)
            .bind(destination.as_str())
            .bind(uid)
            .fetch_all_tx(tx)
            .await?;
        w.split.add("move: UPDATE messages RETURNING (+triggers)", t);
        let t = Instant::now();
        q("UPDATE mailboxes SET total=CASE WHEN total>0 THEN total-1 ELSE 0 END WHERE id=?1 AND account_id=?2").bind(source.as_str()).bind(&w.account).execute_tx(tx).await?;
        q("UPDATE mailboxes SET total=total+1 WHERE id=?1 AND account_id=?2").bind(destination.as_str()).bind(&w.account).execute_tx(tx).await?;
        if crate::cache::flags_json_is_unread(&rows[0].get_string("flags_json")) {
            q("UPDATE mailboxes SET unread=unread-1 WHERE id=?1 AND account_id=?2 AND unread>0").bind(source.as_str()).bind(&w.account).execute_tx(tx).await?;
            q("UPDATE mailboxes SET unread=unread+1 WHERE id=?1 AND account_id=?2").bind(destination.as_str()).bind(&w.account).execute_tx(tx).await?;
        }
        w.split.add("move: UPDATE mailboxes counters (2-4)", t);
        let t = Instant::now();
        q(CHANGE_SQL).bind(&w.account).bind("Email").bind(format!("[\"{id}\"]")).bind("updated").bind(now).fetch_all_tx(tx).await?;
        w.split.add("move: INSERT changes Email", t);
        let t = Instant::now();
        q(CHANGE_SQL).bind(&w.account).bind("Mailbox").bind(format!("[\"{source}\",\"{destination}\"]")).bind("updated").bind(now).fetch_all_tx(tx).await?;
        w.split.add("move: INSERT changes Mailbox (2 rows)", t);
        Ok::<_, crate::StoreError>(())
    });
    if result.is_ok() {
        w.place.lock().unwrap()[i] = to;
    }
    result
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    Flag,
    Move,
}

async fn ladder(w: &Arc<World>, kind: Kind, label: &str) {
    for n in [1usize, 8, 64, 256, 1024] {
        // About 1,024 mutations per level (at least 2 batches).
        let rounds = (1024 / n).max(2);
        w.split.take();
        let before = w.writer.stats();
        let started = Instant::now();
        let mut message = 0usize;
        for round in 0..rounds {
            let mut tasks = Vec::with_capacity(n);
            for _ in 0..n {
                let i = message % BENCH_MESSAGES;
                message += 1;
                let w = w.clone();
                tasks.push(tokio::spawn(async move {
                    match kind {
                        Kind::Flag => flag_job(&w, i, round).await,
                        Kind::Move => move_job(&w, i, round).await,
                    }
                }));
            }
            for task in tasks {
                task.await.unwrap().unwrap();
            }
        }
        let wall = started.elapsed();
        let after = w.writer.stats();
        let jobs = after.jobs - before.jobs;
        let commits = after.commits - before.commits;
        let d = |f: fn(&super::SqliteWriterStats) -> u64| f(&after) - f(&before);
        let split: serde_json::Map<String, serde_json::Value> = w
            .split
            .take()
            .into_iter()
            .map(|(k, (count, micros))| {
                (k.to_string(), serde_json::json!({"count": count, "microsPerJob": micros as f64 / jobs as f64}))
            })
            .collect();
        println!(
            "SQLITE_WRITER_CAPACITY {}",
            serde_json::json!({
                "kind": "grouped-mutation-ladder-v1",
                "variant": label,
                "mutation": format!("{kind:?}"),
                "jobsPerBatchTarget": n,
                "jobs": jobs,
                "commits": commits,
                "jobsPerCommit": jobs as f64 / commits.max(1) as f64,
                "rowsPerSecond": jobs as f64 / wall.as_secs_f64(),
                "microsPerRow": wall.as_micros() as f64 / jobs as f64,
                "writerSplitMicrosPerJob": {
                    "jobHold": d(|s| s.job_hold_micros_total) as f64 / jobs as f64,
                    "savepoint": d(|s| s.savepoint_micros_total) as f64 / jobs as f64,
                    "begin": d(|s| s.begin_micros_total) as f64 / jobs as f64,
                    "commit": d(|s| s.commit_micros_total) as f64 / jobs as f64,
                    "commitPerBatch": d(|s| s.commit_micros_total) as f64 / commits.max(1) as f64,
                },
                "statementMicrosPerJob": split,
            })
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "explicit owned SQLCipher diagnostic; set MW_SQLITE_WRITER_BENCH=1"]
async fn grouped_mutation_capacity() {
    assert_eq!(std::env::var("MW_SQLITE_WRITER_BENCH").as_deref(), Ok("1"));
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("capacity.sqlite");
    let store = Store::open_with_encryption(
        path.to_str().unwrap(),
        ServerKey::generate(),
        &EncryptionConfig::with_key(EncryptionMode::Required, DbKey::generate()),
    )
    .await
    .unwrap();
    let crate::Backend::Sqlite(live) = &store.backend else { panic!("SQLite required") };
    assert!(live.is_encrypted());
    let cipher = q("PRAGMA cipher_version").fetch_one(&store.backend).await.unwrap().get_string_idx(0);
    assert!(!cipher.is_empty());
    assert!(store.message_projection_installed().await.unwrap(), "0125 triggers installed");
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "benchmark.invalid",
                port: 993,
                tls: "implicit",
                username: "capacity",
                sync_policy_json: "{}",
            },
            &Credentials { username: "capacity".into(), password: "fixture".into() },
        )
        .await
        .unwrap();
    let mut boxes = Vec::new();
    for name in ["INBOX", "Archive"] {
        boxes.push(
            store
                .upsert_mailbox(&MailboxUpsert {
                    account_id: &account,
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
                .unwrap(),
        );
    }
    let mut ids = Vec::new();
    for uid in 1..=BENCH_MESSAGES {
        ids.push(
            store
                .upsert_message(&MessageUpsert {
                    account_id: &account,
                    mailbox_id: &boxes[0],
                    uid: uid as u32,
                    uidvalidity: 1,
                    message_id: None,
                    thread_id: None,
                    internaldate: Some("2026-11-01T00:00:00Z"),
                    size: 1024,
                    flags_json: "[]",
                    envelope: Some(br#"{"subject":"capacity"}"#),
                    blob_ref: None,
                })
                .await
                .unwrap(),
        );
    }
    let options = (*live.pin().await.active_options()).clone();
    let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(8).connect_with(options).await.unwrap();
    let writer = super::GroupWriter::with_limits(super::Limits {
        max_jobs: 1024,
        max_time: Duration::from_secs(60),
        queue: 4096,
        // A 1,024-job move batch runs longer than the production 5 s turn wait.
        wait_limit: Duration::from_secs(120),
        // Every job through the batch path (N jobs per commit, not 1 + N-1).
        solo_when_idle: false,
        ..super::Limits::default()
    });
    let world = Arc::new(World {
        backend: Backend::Sqlite(LiveSqlite::unmanaged_with_writer(pool, writer.clone())),
        writer,
        account: account.clone(),
        boxes: [boxes[0].clone(), boxes[1].clone()],
        ids,
        place: Mutex::new(vec![0; BENCH_MESSAGES]),
        split: Split::default(),
    });
    let triggers: Vec<String> = q("SELECT name FROM sqlite_schema WHERE type='trigger' AND tbl_name IN ('messages','mailboxes','changes') ORDER BY name")
        .fetch_all(&world.backend)
        .await
        .unwrap()
        .iter()
        .map(|r| r.get_string_idx(0))
        .collect();
    println!(
        "SQLITE_WRITER_CAPACITY {}",
        serde_json::json!({"kind":"setup","cipherVersion":cipher,"messages":BENCH_MESSAGES,"triggersOnMessagesMailboxesChanges":triggers,
        "profile": if cfg!(debug_assertions) { "debug_assertions_enabled" } else { "debug_assertions_disabled" }})
    );
    ladder(&world, Kind::Flag, "all-triggers").await;
    ladder(&world, Kind::Move, "all-triggers").await;

    // (c) journal-style append: one ~10.9 KB row (136 records x 80 bytes) per commit.
    q("CREATE TABLE bench_journal(id INTEGER PRIMARY KEY, payload BLOB NOT NULL)")
        .execute(&world.backend)
        .await
        .unwrap();
    let payload: Vec<u8> = (0..136 * 80).map(|n| (n * 31 % 251) as u8).collect();
    let started = Instant::now();
    let mut commits = 0u64;
    while started.elapsed() < Duration::from_secs(5) {
        let mut tx = world.backend.begin().await.unwrap();
        q("INSERT INTO bench_journal(payload) VALUES(?1)").bind(payload.clone()).execute_tx(&mut tx).await.unwrap();
        tx.commit().await.unwrap();
        commits += 1;
    }
    let seconds = started.elapsed().as_secs_f64();
    println!(
        "SQLITE_WRITER_CAPACITY {}",
        serde_json::json!({"kind":"journal-append-v1","recordsPerRow":136,"bytesPerRecord":80,"rowBytes":payload.len(),
        "commitsPerSecond": commits as f64 / seconds, "megabytesPerSecond": commits as f64 * payload.len() as f64 / seconds / 1e6,
        "recordsPerSecond": commits as f64 * 136.0 / seconds})
    );

    // What the per-row cost is made of: the same flag ladder without the 0125
    // projection triggers, then without every trigger on messages.
    for name in triggers.iter().filter(|n| n.starts_with("message_projection")) {
        q(Box::leak(format!("DROP TRIGGER {name}").into_boxed_str())).execute(&world.backend).await.unwrap();
    }
    ladder(&world, Kind::Flag, "without-0125-projection-triggers").await;
    let rest: Vec<String> = q("SELECT name FROM sqlite_schema WHERE type='trigger' AND tbl_name='messages'")
        .fetch_all(&world.backend)
        .await
        .unwrap()
        .iter()
        .map(|r| r.get_string_idx(0))
        .collect();
    for name in &rest {
        q(Box::leak(format!("DROP TRIGGER {name}").into_boxed_str())).execute(&world.backend).await.unwrap();
    }
    ladder(&world, Kind::Flag, "without-any-messages-trigger").await;
    let Backend::Sqlite(private) = &world.backend else { unreachable!() };
    private.close().await;
    store.close().await;
    let header = std::fs::read(&path).unwrap();
    assert!(!header.starts_with(b"SQLite format 3"), "database file is encrypted");
    println!("SQLITE_WRITER_CAPACITY {}", serde_json::json!({"kind":"verified","fileHeaderPlaintext":false}));
}
