//! Measurement M5 (t63 memory audit, F15): what the search index costs on
//! PostgreSQL. Seeds `M5_MESSAGES` (default 100,000) synthetic messages with
//! about 2 KB of varied body text through the store's own write path, then
//! builds the index twice from the store: in RAM (today's PostgreSQL default)
//! and in a sealed directory on disk. Prints one JSON line per build: seconds,
//! documents/s, index bytes, process memory before/after.
//!
//! Ignored: it needs `MW_TEST_PG` (a disposable database) and minutes of time.
//! `cargo test --release -p mw-engine --test m5_postgres_index_rebuild -- --ignored --nocapture`
use mw_engine::Engine;
use mw_store::{AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store};
use std::time::Instant;

/// Resident memory of this process in bytes (Linux `VmRSS`; Windows working set).
fn rss() -> Option<u64> {
    if cfg!(target_os = "linux") {
        let text = std::fs::read_to_string("/proc/self/status").ok()?;
        return text.lines().find_map(|l| l.strip_prefix("VmRSS:")).and_then(|v| v.trim().trim_end_matches("kB").trim().parse::<u64>().ok()).map(|k| k * 1024);
    }
    if cfg!(windows) {
        let out = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &format!("(Get-Process -Id {}).WorkingSet64", std::process::id())])
            .output()
            .ok()?;
        return String::from_utf8_lossy(&out.stdout).trim().parse().ok();
    }
    None
}

fn dir_bytes(dir: &std::path::Path) -> u64 {
    std::fs::read_dir(dir).map(|it| it.flatten().filter_map(|e| e.metadata().ok()).filter(|m| m.is_file()).map(|m| m.len()).sum()).unwrap_or(0)
}

/// Deterministic pseudo-random words: a 6,000-word synthetic vocabulary with a
/// skewed (roughly Zipf-like) choice, so postings look like mail text rather
/// than one repeated message.
struct Words(u64);
impl Words {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn word(&mut self) -> String {
        let r = self.next() % 1_000_000;
        // Skew: half the words from the 200 most common, the rest spread out.
        let rank = if r < 500_000 { r % 200 } else { 200 + (r * 7919) % 5_800 };
        let syllables = ["ka", "lo", "mi", "ner", "tu", "sa", "ri", "vo", "den", "pa", "qui", "ost"];
        let mut w = String::new();
        let mut n = rank + 7;
        for _ in 0..(2 + rank % 3) {
            w.push_str(syllables[(n % 12) as usize]);
            n /= 12;
        }
        w
    }
}

fn message(i: u64, words: &mut Words) -> Vec<u8> {
    let subject: Vec<String> = (0..6).map(|_| words.word()).collect();
    let mut body = String::new();
    while body.len() < 2_000 {
        body.push_str(&words.word());
        body.push(if words.next() % 12 == 0 { '\n' } else { ' ' });
    }
    format!(
        "Message-ID: <m5-{i}@example.test>\r\nFrom: Sender {s} <sender{s}@example.test>\r\nTo: me@example.test\r\nSubject: {subj}\r\nDate: Wed, 01 Jul 2026 09:00:00 +0000\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{body}\r\n",
        s = i % 997,
        subj = subject.join(" ")
    )
    .into_bytes()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "measurement: needs MW_TEST_PG and several minutes"]
async fn m5_postgres_index_rebuild() {
    let url = std::env::var("MW_TEST_PG").expect("MW_TEST_PG");
    let total: u64 = std::env::var("M5_MESSAGES").ok().and_then(|v| v.parse().ok()).unwrap_or(100_000);
    let accounts: u64 = 50;
    // Fixed key: a re-run reads the bodies sealed by the first run.
    let key = ServerKey::from_bytes(&[71; 32]).unwrap();
    let store = Store::open_postgres(&url, key.clone()).await.unwrap();
    let pool = sqlx::postgres::PgPoolOptions::new().max_connections(1).connect(&url).await.unwrap();

    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages").fetch_one(&pool).await.unwrap();
    if (existing as u64) < total {
        assert_eq!(existing, 0, "partial seed in the database: use an empty database");
        let seeded = Instant::now();
        let mut tasks = Vec::new();
        for a in 0..accounts {
            let store = store.clone();
            let per = total / accounts;
            tasks.push(tokio::spawn(async move {
                let username = format!("m5-{a}@example.test");
                let account = store
                    .create_account(
                        &NewAccount { kind: AccountKind::Imap, host: "imap.example.test", port: 993, tls: "implicit", username: &username, sync_policy_json: "{}" },
                        &Credentials { username: username.clone(), password: "pw".into() },
                    )
                    .await
                    .unwrap();
                let mailbox = store
                    .upsert_mailbox(&MailboxUpsert { account_id: &account, name: "INBOX", role: Some("inbox"), uidvalidity: 1, uidnext: per as u32 + 1, highestmodseq: 0, total: 0, unread: 0, parent_id: None })
                    .await
                    .unwrap();
                let mut words = Words(a * 1_000_003 + 17);
                for n in 0..per {
                    let i = a * per + n;
                    let raw = message(i, &mut words);
                    let blob = store.put_body(&account, &raw).await.unwrap();
                    let mid = format!("m5-{i}@example.test");
                    store
                        .upsert_message(&MessageUpsert {
                            account_id: &account,
                            mailbox_id: &mailbox,
                            uid: n as u32 + 1,
                            uidvalidity: 1,
                            message_id: Some(&mid),
                            thread_id: None,
                            internaldate: Some("2026-07-01T09:00:00Z"),
                            size: raw.len() as u64,
                            flags_json: "[]",
                            envelope: None,
                            blob_ref: Some(&blob),
                        })
                        .await
                        .unwrap();
                }
            }));
        }
        for t in tasks {
            t.await.unwrap();
        }
        println!("{{\"seed\":{{\"messages\":{total},\"seconds\":{:.1}}}}}", seeded.elapsed().as_secs_f64());
    }
    let messages: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages").fetch_one(&pool).await.unwrap();

    // In RAM: today's PostgreSQL default. M5_SKIP_RAM=1 skips it on a re-run.
    if std::env::var("M5_SKIP_RAM").is_err() {
    let before = rss();
    let engine = Engine::new(store.clone());
    let started = Instant::now();
    let summary = engine.rebuild_search_index_all().await.unwrap();
    let seconds = started.elapsed().as_secs_f64();
    let searcher = engine.search().tantivy().reader().unwrap().searcher();
    let bytes = searcher.space_usage().unwrap().total().get_bytes();
    let after = rss();
    println!(
        "{{\"place\":\"ram\",\"messages\":{messages},\"indexed\":{},\"seconds\":{seconds:.1},\"documentsPerSecond\":{:.0},\"indexBytes\":{bytes},\"bytesPerMessage\":{:.0},\"rssBefore\":{before:?},\"rssAfter\":{after:?}}}",
        summary.indexed,
        summary.indexed as f64 / seconds,
        bytes as f64 / summary.indexed.max(1) as f64
    );
    let status_started = Instant::now();
    let status = engine.search_index_status().await.unwrap();
    println!("{{\"place\":\"ram\",\"statusCheckSeconds\":{:.2},\"documents\":{},\"storeMessages\":{}}}", status_started.elapsed().as_secs_f64(), status.documents, status.messages);
    drop(searcher);
    drop(engine);
    }

    // On disk, sealed (the SQLite default, and the proposed PostgreSQL default).
    let dir = std::env::temp_dir().join(format!("m5-index-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let before = rss();
    let engine = Engine::open_with_search_and_publication(store.clone(), &dir, mw_engine::PublicationOptions::default()).unwrap();
    let started = Instant::now();
    let summary = engine.rebuild_search_index_all().await.unwrap();
    let seconds = started.elapsed().as_secs_f64();
    let after = rss();
    let bytes = dir_bytes(&dir);
    println!(
        "{{\"place\":\"disk\",\"indexed\":{},\"seconds\":{seconds:.1},\"documentsPerSecond\":{:.0},\"directoryBytes\":{bytes},\"bytesPerMessage\":{:.0},\"rssBefore\":{before:?},\"rssAfter\":{after:?}}}",
        summary.indexed,
        summary.indexed as f64 / seconds,
        bytes as f64 / summary.indexed.max(1) as f64
    );
    drop(engine);
    // Reopen: what a restart with an existing directory costs before serving.
    // The dropped engine's writer releases the directory lock from its own
    // thread; wait for it (bounded) before reopening.
    let mut waited = 0.0;
    let engine = loop {
        let started = Instant::now();
        match Engine::open_with_search_and_publication(store.clone(), &dir, mw_engine::PublicationOptions::default()) {
            Ok(engine) => break (engine, started.elapsed().as_secs_f64()),
            Err(e) if waited < 60.0 && e.to_string().contains("lock") => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                waited += 0.5;
            }
            Err(e) => panic!("reopen: {e}"),
        }
    };
    let (engine, opened) = engine;
    println!("{{\"place\":\"disk-reopen\",\"waitedForLockSeconds\":{waited:.1}}}");
    let started = Instant::now();
    let reconciled = engine.reconcile_search_index().await.unwrap();
    println!(
        "{{\"place\":\"disk-reopen\",\"openSeconds\":{opened:.2},\"reconcileSeconds\":{:.2},\"rebuilt\":{},\"rss\":{:?}}}",
        started.elapsed().as_secs_f64(),
        reconciled.is_some(),
        rss()
    );
    drop(engine);
    let _ = std::fs::remove_dir_all(&dir);
    store.close().await;
}
