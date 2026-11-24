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
