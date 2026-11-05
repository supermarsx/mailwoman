//! The folder and message load metrics store and read back identically on SQLite
//! and PostgreSQL (the schema is the existing `client_timing_bucket`, whose
//! `metric` column carries the kind), and rows written before these metrics
//! existed are untouched. Postgres runs when `DATABASE_URL_PG` or `MW_TEST_PG`
//! is set; otherwise the SQLite assertions still run.

use mw_store::client_timing::{Hist, METRICS, TimingBucket};
use mw_store::{ServerKey, Store};

fn key() -> ServerKey {
    ServerKey::from_bytes(&[7u8; 32]).unwrap()
}

fn pg_dsn() -> Option<String> {
    std::env::var("DATABASE_URL_PG")
        .ok()
        .or_else(|| std::env::var("MW_TEST_PG").ok())
        .filter(|s| !s.trim().is_empty())
}

fn hist(values: &[u64]) -> Hist {
    let mut h = Hist::default();
    for v in values {
        h.record(*v);
    }
    h
}

/// A far-future hour no other test writes, so a shared database stays comparable.
const HOUR: i64 = 4_102_444_800 + 3600 * 17;

async fn run(store: &Store, instance: &str) -> Vec<(i64, String, String, u64, u64, Vec<u64>)> {
    let bucket = |metric: &str, device: &str, values: &[u64]| TimingBucket {
        start: HOUR,
        metric: metric.into(),
        device: device.into(),
        hist: hist(values),
    };
    let rows = vec![
        // A row of the first release's kind, as already stored.
        bucket("timeToUsable", "desktop", &[800, 1200]),
        bucket("folderLoadCold", "desktop", &[300, 450, 2000]),
        bucket("folderLoadCached", "desktop", &[15, 22]),
        bucket("messageLoadCold", "mobile", &[900]),
        bucket("messageLoadCached", "mobile", &[30, 31, 32]),
    ];
    let now = HOUR + 600;
    store.save_client_timing(instance, &rows, now, 30).await.unwrap();
    // Absolute snapshots: replaying the same snapshot adds nothing, a larger one wins.
    store.save_client_timing(instance, &rows, now, 30).await.unwrap();
    let grown = vec![bucket("folderLoadCold", "desktop", &[300, 450, 2000, 90])];
    store.save_client_timing(instance, &grown, now, 30).await.unwrap();
    let smaller = vec![bucket("folderLoadCold", "desktop", &[1])];
    store.save_client_timing(instance, &smaller, now, 30).await.unwrap();

    let mut got: Vec<_> = store
        .client_timing_history(HOUR, HOUR + 3600)
        .await
        .unwrap()
        .into_iter()
        .filter(|b| b.start == HOUR)
        .map(|b| (b.start, b.metric, b.device, b.hist.count, b.hist.sum_ms, b.hist.bins))
        .collect();
    got.sort_by(|a, b| (&a.1, &a.2).cmp(&(&b.1, &b.2)));
    got
}

#[tokio::test]
async fn load_metrics_round_trip_the_same_on_both_backends() {
    for m in ["folderLoadCold", "folderLoadCached", "messageLoadCold", "messageLoadCached", "timeToUsable"] {
        assert!(METRICS.contains(&m), "{m}");
    }
    let sqlite = Store::open_in_memory(key()).await.unwrap();
    let a = run(&sqlite, "parity-a").await;
    let cold = a.iter().find(|r| r.1 == "folderLoadCold").unwrap();
    assert_eq!((cold.3, cold.4), (4, 2840), "the larger snapshot won; the smaller was ignored");
    assert_eq!(a.len(), 5);

    // Unknown kinds are refused, as before.
    let bad = TimingBucket { start: HOUR, metric: "folderLoad:inbox".into(), device: "desktop".into(), hist: hist(&[1]) };
    assert!(sqlite.save_client_timing("parity-a", &[bad], HOUR, 30).await.is_err());

    match pg_dsn() {
        Some(dsn) => {
            let pg = Store::open_postgres(&dsn, key()).await.unwrap();
            let b = run(&pg, "parity-pg").await;
            assert_eq!(a, b, "SQLite vs Postgres client timing mismatch");
        }
        None => eprintln!("[mw-store] client-timing parity: no Postgres DSN set; SQLite only."),
    }
}
