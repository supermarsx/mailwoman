// t50hp-e26: `mailboxes.total` / `mailboxes.unread` equal a brute-force count
// of the cached rows after any interleaving of the row writers, a sync's STATUS
// never writes them, and 0122's one-time recount follows `flags_json_is_unread`.
// Included in cache::tests to reuse its account/mailbox/message helpers.

/// (total, unread) counted from the rows themselves.
async fn counted(s: &Store, mailbox: &str) -> (u32, u32) {
    let rows = q("SELECT flags_json FROM messages WHERE mailbox_id = ?1")
        .bind(mailbox)
        .fetch_all(s.backend())
        .await
        .unwrap();
    let unread = rows.iter().filter(|r| flags_json_is_unread(&r.get_string("flags_json"))).count();
    (rows.len() as u32, unread as u32)
}

async fn stored(s: &Store, mailbox: &str) -> (u32, u32) {
    let m = s.get_mailbox(mailbox).await.unwrap();
    (m.total, m.unread)
}

async fn assert_exact(s: &Store, boxes: &[String], context: &str) {
    for mailbox in boxes {
        assert_eq!(stored(s, mailbox).await, counted(s, mailbox).await, "{context}: mailbox {mailbox}");
    }
}

/// Deterministic xorshift, so a failing sequence replays from its seed.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

const FLAG_SETS: [&str; 7] = [
    "[]",
    r#"["Seen"]"#,
    r#"["Seen","Flagged"]"#,
    r#"["Flagged"]"#,
    r#"[{"Keyword":"Seen"}]"#,
    r#"[{"Keyword":"$Junk"},"Seen"]"#,
    "not json",
];

struct CounterWorld {
    account: String,
    boxes: Vec<String>,
    next_uid: std::sync::atomic::AtomicU32,
}

async fn counter_world(s: &Store) -> CounterWorld {
    let account = seed_account(s).await;
    let mut boxes = Vec::new();
    for name in ["INBOX", "Archive", "Trash"] {
        boxes.push(seed_mailbox(s, &account, name, 100).await);
    }
    CounterWorld { account, boxes, next_uid: std::sync::atomic::AtomicU32::new(1) }
}

async fn random_id(s: &Store, w: &CounterWorld, rng: &mut Rng) -> Option<String> {
    let ids = q("SELECT stable_id FROM messages WHERE account_id = ?1 ORDER BY stable_id")
        .bind(&w.account)
        .fetch_all(s.backend())
        .await
        .unwrap();
    (!ids.is_empty()).then(|| ids[rng.below(ids.len())].get_string("stable_id"))
}

/// One operation from the writer set of the t50hp-e26 path table. Errors from
/// a lost race (`NotFound`, a refused compare-and-set) are part of the game.
async fn counter_op(s: &Store, w: &CounterWorld, rng: &mut Rng) -> &'static str {
    let mailbox = w.boxes[rng.below(w.boxes.len())].clone();
    let flags = FLAG_SETS[rng.below(FLAG_SETS.len())];
    match rng.below(12) {
        0 | 1 => {
            let uid = w.next_uid.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let mid = format!("<c{uid}@fixture>");
            let mut m = msg(&w.account, &mailbox, uid, 100, &mid, "2026-07-01T10:00:00Z");
            m.flags_json = flags;
            if rng.below(2) == 0 {
                s.upsert_message(&m).await.unwrap();
            } else {
                s.ingest_message_with_body(&m, format!("body {uid}").as_bytes()).await.unwrap();
            }
            "ingest"
        }
        2 => {
            // A sync replay of an existing row with new flags (sync flag path).
            if let Some(id) = random_id(s, w, rng).await
                && let Ok(cur) = s.get_message(&id).await
            {
                let mid = cur.message_id.clone().unwrap_or_default();
                let date = cur.internaldate.clone().unwrap_or_default();
                let mut m = msg(&w.account, &cur.mailbox_id, cur.uid, cur.uidvalidity, &mid, &date);
                m.flags_json = flags;
                let _ = s.upsert_message(&m).await;
            }
            "re-upsert"
        }
        3 => {
            if let Some(id) = random_id(s, w, rng).await {
                let _ = s.set_flags_batch(&[(id, flags.into())]).await;
            }
            "set_flags_batch"
        }
        4 => {
            if let Some(id) = random_id(s, w, rng).await
                && let Ok(cur) = s.get_message(&id).await
            {
                let _ = s.set_flags_if_current(&cur, flags).await;
            }
            "set_flags_if_current"
        }
        5 => {
            if let Some(id) = random_id(s, w, rng).await {
                let _ = s.relocate_message(&id, &mailbox, rng.below(1000) as u32 + 5000, 100).await;
            }
            "relocate"
        }
        6 => {
            if let Some(id) = random_id(s, w, rng).await
                && let Ok(cur) = s.get_message(&id).await
            {
                let _ = s.relocate_message_if_current(&cur, &mailbox, rng.below(1000) as u32 + 7000, 100).await;
            }
            "relocate_if_current"
        }
        7 => {
            if let Some(id) = random_id(s, w, rng).await {
                let _ = s.delete_message(&id).await;
            }
            "delete"
        }
        8 => {
            if let Some(id) = random_id(s, w, rng).await
                && let Ok(cur) = s.get_message(&id).await
            {
                let _ = s.delete_message_if_current(&cur).await;
            }
            "delete_if_current"
        }
        9 => {
            if let Some(id) = random_id(s, w, rng).await
                && let Ok(cur) = s.get_message(&id).await
            {
                let _ = s.delete_synced_message_if_source(&cur).await;
            }
            "delete_synced"
        }
        10 => {
            // A sync's LIST/STATUS: arbitrary server figures.
            let name = s.get_mailbox(&mailbox).await.unwrap().name;
            s.upsert_mailbox(&MailboxUpsert {
                account_id: &w.account,
                name: &name,
                role: Some("inbox"),
                uidvalidity: 100,
                uidnext: 9999,
                highestmodseq: 0,
                total: rng.below(50) as u32,
                unread: rng.below(50) as u32,
                parent_id: None,
            })
            .await
            .unwrap();
            "status"
        }
        _ => {
            s.update_mailbox_counts(&mailbox, 9999, 0, rng.below(50) as u32, rng.below(50) as u32)
                .await
                .unwrap();
            "update_counts"
        }
    }
}

async fn counter_sequence_contract(s: &Store, seed: u64, steps: usize) {
    let w = counter_world(s).await;
    let mut rng = Rng(seed);
    for step in 0..steps {
        let op = counter_op(s, &w, &mut rng).await;
        assert_exact(s, &w.boxes, &format!("seed {seed} step {step} after {op}")).await;
    }
}

/// Two connections writing the same folders at once; afterwards the counters
/// are exact and were never negative (a negative value would clamp and then
/// show as a mismatch).
async fn counter_concurrent_contract(s: &Store, steps: usize) {
    let w = counter_world(s).await;
    // A start population so moves, flags and deletes have something to race on.
    let mut rng = Rng(7);
    for _ in 0..12 {
        let uid = w.next_uid.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mid = format!("<p{uid}@fixture>");
        let mut m = msg(&w.account, &w.boxes[rng.below(3)], uid, 100, &mid, "2026-07-01T10:00:00Z");
        m.flags_json = FLAG_SETS[rng.below(2)];
        s.upsert_message(&m).await.unwrap();
    }
    let run = |seed: u64| {
        let w = &w;
        async move {
            let mut rng = Rng(seed);
            for _ in 0..steps {
                counter_op(s, w, &mut rng).await;
            }
        }
    };
    tokio::join!(run(0x9E37_79B9), run(0x85EB_CA6B));
    assert_exact(s, &w.boxes, "after two concurrent writers").await;
}

#[tokio::test]
async fn mailbox_counters_exact_over_random_sequences_sqlite() {
    for seed in [1u64, 2, 3, 0xDEAD_BEEF, 0x1234_5678_9ABC] {
        counter_sequence_contract(&store().await, seed, 200).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mailbox_counters_exact_under_concurrent_writers_sqlite() {
    let dir = tempfile::tempdir().unwrap();
    let s = Store::open(dir.path().join("c.db").to_str().unwrap(), ServerKey::generate())
        .await
        .unwrap();
    counter_concurrent_contract(&s, 150).await;
    s.close().await;
}

#[tokio::test]
async fn status_never_writes_cached_counters() {
    let s = store().await;
    let account = seed_account(&s).await;
    let mut up = MailboxUpsert {
        account_id: &account,
        name: "INBOX",
        role: Some("inbox"),
        uidvalidity: 1,
        uidnext: 1,
        highestmodseq: 0,
        total: 500,
        unread: 400,
        parent_id: None,
    };
    let inbox = s.upsert_mailbox(&up).await.unwrap();
    assert_eq!(stored(&s, &inbox).await, (0, 0), "a new folder holds no cached rows");
    let mut m = msg(&account, &inbox, 1, 1, "<s@fixture>", "2026-07-01T10:00:00Z");
    m.flags_json = "[]";
    s.upsert_message(&m).await.unwrap();
    up.total = 7;
    up.unread = 0;
    s.upsert_mailbox(&up).await.unwrap();
    s.update_mailbox_counts(&inbox, 2, 0, 9, 9).await.unwrap();
    assert_eq!(stored(&s, &inbox).await, (1, 1));
    let upstream = q("SELECT upstream_total, upstream_unseen FROM mailboxes WHERE id = ?1")
        .bind(&inbox)
        .fetch_all(s.backend())
        .await
        .unwrap();
    assert_eq!((upstream[0].get_i64("upstream_total"), upstream[0].get_i64("upstream_unseen")), (9, 9));
}

/// The part of 0122 after its `-- recount` marker, run again over rows whose
/// counters were damaged on purpose (the mixed values the old sync left).
async fn recount_contract(s: &Store) {
    let account = seed_account(s).await;
    let inbox = seed_mailbox(s, &account, "INBOX", 1).await;
    let empty = seed_mailbox(s, &account, "Empty", 1).await;
    for (uid, flags) in [
        "[]",
        r#"["Seen"]"#,
        r#" [ "Flagged" , "Seen" ] "#,
        r#"[{"Keyword":"Seen"}]"#,
        r#"{"Seen":"Seen"}"#,
        r#""Seen""#,
        "not json",
        r#"["Draft"]"#,
    ]
    .into_iter()
    .enumerate()
    {
        let mid = format!("<r{uid}@fixture>");
        let mut m = msg(&account, &inbox, uid as u32 + 1, 1, &mid, "2026-07-01T10:00:00Z");
        m.flags_json = flags;
        s.upsert_message(&m).await.unwrap();
    }
    q("UPDATE mailboxes SET total = 9999, unread = 4321 WHERE account_id = ?1")
        .bind(&account)
        .execute(s.backend())
        .await
        .unwrap();
    run_recount(s).await;
    assert_eq!(counted(s, &inbox).await, (8, 6), "brute force per flags_json_is_unread");
    assert_exact(s, &[inbox, empty], "after the 0122 recount").await;
}

async fn run_recount(s: &Store) {
    let sql = match s.backend.dialect() {
        Dialect::Sqlite => include_str!("../migrations/0122_mailbox_cached_counters.sql"),
        Dialect::Postgres => include_str!("../migrations_pg/0122_mailbox_cached_counters.sql"),
    };
    let recount = &sql[sql.find("-- recount").expect("0122 marks its recount")..];
    match &s.backend {
        crate::backend::Backend::Sqlite(pool) => {
            sqlx::raw_sql(recount).execute(pool).await.unwrap();
        }
        crate::backend::Backend::Postgres(pool) => {
            sqlx::raw_sql(recount).execute(pool).await.unwrap();
        }
    }
}

#[tokio::test]
async fn migration_0122_recount_matches_rows_sqlite() {
    recount_contract(&store().await).await;
}

/// Cost of the 0122 recount over one 100,000-message folder (30 % unread).
/// A measurement, not a gate: `--ignored --nocapture` prints the time.
async fn recount_cost(s: &Store) {
    let account = seed_account(s).await;
    let inbox = seed_mailbox(s, &account, "INBOX", 1).await;
    const ROWS: usize = 100_000;
    let mut tx = s.backend.begin().await.unwrap();
    if s.backend.dialect() == Dialect::Postgres {
        // Seeding only: skip the per-row publication triggers (a superuser
        // fixture). The recount below runs with every trigger in place.
        q("SET LOCAL session_replication_role = replica").execute_tx(&mut tx).await.unwrap();
    }
    for chunk in 0..(ROWS / 1000) {
        let mut sql = String::from(
            "INSERT INTO messages (stable_id, account_id, mailbox_id, uid, uidvalidity, internaldate, size, flags_json) VALUES ",
        );
        for i in 0..1000 {
            let n = chunk * 1000 + i;
            let flags = if n % 10 < 3 { "[]" } else { r#"["Seen"]"# };
            if i > 0 {
                sql.push(',');
            }
            sql.push_str(&format!("('cost{n}','{account}','{inbox}',{},1,'2026-07-01T10:00:00Z',100,'{flags}')", n + 1));
        }
        let statement: &'static str = Box::leak(sql.into_boxed_str());
        q(statement).execute_tx(&mut tx).await.unwrap();
    }
    tx.commit().await.unwrap();
    let started = std::time::Instant::now();
    run_recount(s).await;
    let elapsed = started.elapsed();
    assert_eq!(stored(s, &inbox).await, (ROWS as u32, (ROWS * 3 / 10) as u32));
    println!("0122 recount over {ROWS} messages ({:?}): {elapsed:?}", s.backend.dialect());
}

#[tokio::test]
#[ignore = "measurement: cargo test ... recount_cost -- --ignored --nocapture"]
async fn migration_0122_recount_cost_sqlite() {
    let dir = tempfile::tempdir().unwrap();
    let s = Store::open(dir.path().join("cost.db").to_str().unwrap(), ServerKey::generate())
        .await
        .unwrap();
    recount_cost(&s).await;
    s.close().await;
}

async fn counters_private_pg(which: u8) {
    use sqlx::Connection;
    let dsn = std::env::var("MW_TEST_PG").expect("this PostgreSQL contract requires the owned MW_TEST_PG fixture");
    let schema = format!("mailbox_counters_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&mut admin).await.unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped.query_pairs_mut().append_pair("options", &format!("-csearch_path={schema}"));
    let s = Store::open(scoped.as_str(), ServerKey::generate()).await.unwrap();
    match which {
        0 => {
            for seed in [1u64, 2, 0xDEAD_BEEF] {
                counter_sequence_contract(&s, seed, 150).await;
            }
        }
        1 => counter_concurrent_contract(&s, 150).await,
        2 => recount_contract(&s).await,
        3 => recount_cost(&s).await,
        _ => unreachable!(),
    }
    s.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE")).execute(&mut admin).await.unwrap();
}

#[tokio::test]
#[ignore = "requires owned MW_TEST_PG fixture; focused gate must explicitly include ignored tests"]
async fn mailbox_counters_exact_over_random_sequences_postgres() { counters_private_pg(0).await; }
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires owned MW_TEST_PG fixture; focused gate must explicitly include ignored tests"]
async fn mailbox_counters_exact_under_concurrent_writers_postgres() { counters_private_pg(1).await; }
#[tokio::test]
#[ignore = "requires owned MW_TEST_PG fixture; focused gate must explicitly include ignored tests"]
async fn migration_0122_recount_matches_rows_postgres() { counters_private_pg(2).await; }
#[tokio::test]
#[ignore = "measurement on MW_TEST_PG: -- --ignored --nocapture"]
async fn migration_0122_recount_cost_postgres() { counters_private_pg(3).await; }
