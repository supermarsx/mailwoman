//! p95 latency gate (SPEC §23, plan §3 e1): build a synthetic 100k-document
//! index and assert p95 query latency < 50 ms.
//!
//! Marked `#[ignore]` so the default `cargo test -p mw-search` stays fast;
//! run the real number with:
//!
//! ```text
//! cargo test -p mw-search --release --test bench -- --ignored --nocapture
//! ```
//!
//! In release the whole harness (build 100k + 700 timed queries) runs in a few
//! seconds and prints p50/p95/max.

use std::time::Instant;

use mw_search::{Index, IndexDoc, parse_query};

const N: usize = 100_000;

/// A tiny deterministic LCG — reproducible corpus without a rand dependency.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 16
    }
    fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[(self.next() as usize) % xs.len()]
    }
}

fn synthetic_corpus() -> Vec<IndexDoc> {
    let first = [
        "alice", "bob", "carol", "dave", "erin", "frank", "grace", "heidi", "ivan", "judy",
    ];
    let domains = [
        "example.com",
        "acme.org",
        "vendor.net",
        "promo.example",
        "mail.io",
    ];
    let subjects = [
        "Quarterly report ready",
        "Lunch on Friday",
        "Invoice attached",
        "Weekend deals inside",
        "Project status update",
        "Meeting notes and action items",
        "Your receipt from the store",
        "Reminder about the deadline",
    ];
    let bodies = [
        "Please review the attached numbers before the deadline arrives.",
        "Want to grab lunch after the review meeting on Friday?",
        "Your monthly invoice is attached; total due is listed inside.",
        "Unbeatable weekend deals inside, unsubscribe anytime you like.",
        "Here is the latest project status with blockers and next steps.",
    ];
    let mailboxes = ["INBOX", "Archive", "Sent", "Trash", "Junk"];
    let tags = ["Work", "Personal", "Finance", "Travel", "Receipts"];

    let mut rng = Rng(0x1234_5678_9abc_def0);
    let mut docs = Vec::with_capacity(N);
    for i in 0..N {
        let from = format!("{}@{}", rng.pick(&first), rng.pick(&domains));
        let to = format!("{}@{}", rng.pick(&first), rng.pick(&domains));
        let has_attachment = rng.next().is_multiple_of(3);
        let mut keywords = vec![rng.pick(&tags).to_string()];
        if rng.next().is_multiple_of(2) {
            keywords.push("$seen".to_string());
        }
        let filenames = if has_attachment {
            vec![format!("file-{i}.pdf")]
        } else {
            vec![]
        };
        docs.push(IndexDoc {
            stable_id: format!("m{i}"),
            account_id: "acct".to_string(),
            mailbox_id: rng.pick(&mailboxes).to_string(),
            from,
            to,
            cc: String::new(),
            subject: format!("{} #{i}", rng.pick(&subjects)),
            body: rng.pick(&bodies).to_string(),
            date: 1_577_836_800 + (i as i64) * 60,
            has_attachment,
            keywords,
            size: 1_000 + (rng.next() % 500_000),
            filenames,
            pinned: rng.next().is_multiple_of(20),
        });
    }
    docs
}

fn percentile(sorted_us: &[u128], p: f64) -> u128 {
    if sorted_us.is_empty() {
        return 0;
    }
    let rank = (p / 100.0 * (sorted_us.len() as f64 - 1.0)).round() as usize;
    sorted_us[rank.min(sorted_us.len() - 1)]
}

#[test]
#[ignore = "heavy: builds a 100k-doc index; run explicitly for the p95 number"]
fn p95_under_50ms_over_100k() {
    let build_start = Instant::now();
    let idx = Index::open_in_ram().expect("open ram index");
    idx.upsert_batch(&synthetic_corpus()).expect("index 100k");
    assert_eq!(idx.num_docs(), N as u64);
    let build_ms = build_start.elapsed().as_secs_f64() * 1000.0;

    let queries = [
        "from:alice",
        "subject:report",
        "body:deadline",
        "has:attachment",
        "in:INBOX",
        "tag:Work",
        "is:unread",
        "larger:100000",
        "from:bob AND has:attachment",
        "tag:Finance OR tag:Travel",
        "subject:\"project status\"",
        "in:Archive NOT is:unread",
        "after:2020-06-01",
        "review meeting",
    ];

    // Warm up caches.
    for q in &queries {
        let parsed = parse_query(q).expect("parse");
        let _ = idx.search(&parsed, 50).expect("search");
    }

    let mut samples = Vec::with_capacity(queries.len() * 50);
    for _ in 0..50 {
        for q in &queries {
            let parsed = parse_query(q).expect("parse");
            let t = Instant::now();
            let _ = idx.search(&parsed, 50).expect("search");
            samples.push(t.elapsed().as_micros());
        }
    }
    samples.sort_unstable();

    let p50 = percentile(&samples, 50.0);
    let p95 = percentile(&samples, 95.0);
    let p99 = percentile(&samples, 99.0);
    let max = *samples.last().unwrap();

    println!("mw-search bench over {N} docs, {} queries:", samples.len());
    println!("  index build : {build_ms:.0} ms");
    println!("  p50         : {:.3} ms", p50 as f64 / 1000.0);
    println!("  p95         : {:.3} ms", p95 as f64 / 1000.0);
    println!("  p99         : {:.3} ms", p99 as f64 / 1000.0);
    println!("  max         : {:.3} ms", max as f64 / 1000.0);

    assert!(
        p95 < 50_000,
        "p95 {} us exceeds the 50 ms gate (SPEC §23)",
        p95
    );
}
