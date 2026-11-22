//! Explicit, owned-file diagnostic; never runs as part of the normal test suite.
use crate::{AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store};
use crate::encryption::{DbKey, EncryptionConfig, EncryptionMode};
use crate::pool_telemetry::{QuerySnapshot, measure_queries};
use std::time::Instant;

fn report(name: &str, operations: usize, rows: usize, elapsed: std::time::Duration,
          samples: &mut [u64], counters: QuerySnapshot) {
    samples.sort_unstable();
    let percentile = |p: usize| samples[(samples.len() - 1) * p / 100];
    println!("STORE_OPERATION_BENCH {}", serde_json::json!({
        "kind":"real-store-sqlcipher-v1", "cohort":name,
        "operations":operations,"rowOperations":rows,"elapsedMicros":elapsed.as_micros(),
        "sampleCount":samples.len(),"operationsPerSecond":operations as f64/elapsed.as_secs_f64(),
        "rowOperationsPerSecond":rows as f64/elapsed.as_secs_f64(),
        "profile":if cfg!(debug_assertions) {"debug_assertions_enabled"} else {"debug_assertions_disabled"},
        "configuredRows":50,"configuredRounds":20,"pooledReadConcurrency":5,
        "scope":"short_owned_tempfile_store_diagnostic_not_sustained_capacity",
        "operationP50Micros":percentile(50),"operationP95Micros":percentile(95),
        "qCounters":counters,"timingsOverlap":true,
        "sqlExecutionOnlyMicros":null,"cryptoCpuMicros":null,"httpRouteMicros":null
    }));
}

#[tokio::test]
#[ignore = "explicit owned SQLCipher diagnostic; set MW_STORE_OPERATION_BENCH=1"]
async fn real_sqlcipher_store_operation_breakdown() {
    assert_eq!(std::env::var("MW_STORE_OPERATION_BENCH").as_deref(), Ok("1"));
    const ROWS: usize = 50;
    const ROUNDS: usize = 20;
    const WORKERS: usize = 5;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("operation-benchmark.sqlite");
    let store = Store::open_with_encryption(path.to_str().unwrap(), ServerKey::generate(),
        &EncryptionConfig::with_key(EncryptionMode::Required, DbKey::generate())).await.unwrap();
    let crate::Backend::Sqlite(live) = &store.backend else { panic!("SQLite required") };
    assert!(live.is_encrypted());
    let cipher = crate::backend::q("PRAGMA cipher_version").fetch_one(&store.backend).await.unwrap();
    assert!(!cipher.get_string_idx(0).is_empty());
    let account = store.create_account(&NewAccount { kind:AccountKind::Imap,
        host:"benchmark.invalid",port:993,tls:"implicit",username:"benchmark",
        sync_policy_json:"{}" }, &Credentials { username:"benchmark".into(),password:"fixture".into() }).await.unwrap();
    let mailbox = store.upsert_mailbox(&MailboxUpsert {account_id:&account,name:"INBOX",
        role:Some("inbox"),uidvalidity:1,uidnext:51,highestmodseq:0,total:0,unread:0,parent_id:None}).await.unwrap();
    let mut ids = Vec::new();
    for uid in 1..=ROWS {
        ids.push(store.upsert_message(&MessageUpsert { account_id:&account,mailbox_id:&mailbox,
            uid:uid as u32,uidvalidity:1,message_id:None,thread_id:None,
            internaldate:Some("2026-11-01T00:00:00Z"),size:1024,flags_json:"[]",
            envelope:Some(br#"{"subject":"benchmark"}"#),blob_ref:None }).await.unwrap());
    }
    store.update_mailbox_counts(&mailbox,51,0,ROWS as u32,ROWS as u32).await.unwrap();
    // API warmup only; concurrent requests do not prove every handle was used.
    futures_util::future::join_all((0..WORKERS).map(|_| async {
        for id in &ids { assert_eq!(store.get_message(id).await.unwrap().size,1024); }
    })).await;
    let started = Instant::now();
    let (mut samples,counters) = measure_queries(async {
        let mut samples=Vec::new();
        for _ in 0..ROUNDS*ROWS {
            let at=Instant::now();
            assert_eq!(store.get_message(&ids[0]).await.unwrap().stable_id,ids[0]);
            samples.push(at.elapsed().as_micros() as u64);
        }
        samples
    }).await;
    assert_eq!(counters.read.calls,(ROUNDS*ROWS) as u64);
    report("warm_point_read",ROUNDS*ROWS,ROUNDS*ROWS,started.elapsed(),&mut samples,counters);
    let started=Instant::now();
    let (parts,counters)=measure_queries(futures_util::future::join_all((0..WORKERS).map(|worker| {
        let store=&store;let ids=&ids;
        async move {
            let mut samples=Vec::new();
            for n in 0..ROUNDS*ROWS/WORKERS {
                let at=Instant::now();
                let id=&ids[(n+worker)%ROWS];
                assert_eq!(store.get_message(id).await.unwrap().stable_id,*id);
                samples.push(at.elapsed().as_micros() as u64);
            }
            samples
        }
    }))).await;
    let mut samples:Vec<_>=parts.into_iter().flatten().collect();
    assert_eq!(counters.read.calls,(ROUNDS*ROWS) as u64);
    report("pooled_read_five_inflight",ROUNDS*ROWS,ROUNDS*ROWS,started.elapsed(),&mut samples,counters);
    for batch in [false,true] {
        let started=Instant::now();
        let (mut samples,counters)=measure_queries(async {
            let mut samples=Vec::new();
            for round in 0..ROUNDS {
                let flags=if round%2==0 {r#"["Seen"]"#} else {"[]"};
                if batch {
                    let updates:Vec<_>=ids.iter().map(|id|(id.clone(),flags.to_string())).collect();
                    let at=Instant::now();
                    assert!(store.set_flags_batch(&updates).await.unwrap().iter().all(|written|*written));
                    samples.push(at.elapsed().as_micros() as u64);
                } else {
                    for id in &ids {
                        let at=Instant::now();store.set_flags(id,flags).await.unwrap();
                        samples.push(at.elapsed().as_micros() as u64);
                    }
                }
            }
            samples
        }).await;
        let elapsed=started.elapsed();
        assert_eq!(counters.read.failures+counters.write.failures+counters.other.failures+counters.acquire.failures,0);
        for id in &ids { assert_eq!(store.get_message(id).await.unwrap().flags_json,"[]"); }
        let folder=store.get_mailbox(&mailbox).await.unwrap();
        assert_eq!(folder.unread,ROWS as u32);
        report(if batch {"batch_flags_transaction_50"} else {"scalar_flags_transaction"},
            if batch {ROUNDS} else {ROUNDS*ROWS},ROUNDS*ROWS,elapsed,&mut samples,counters);
    }
    store.close().await;
    // Header inspection occurs only after managed connections close (POSIX locks).
    use std::io::Read;
    let mut header=[0u8;16];
    std::fs::File::open(&path).unwrap().read_exact(&mut header).unwrap();
    assert_ne!(&header,crate::encryption::SQLITE_MAGIC);
}
