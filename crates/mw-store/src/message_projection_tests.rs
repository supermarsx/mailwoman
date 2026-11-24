// t50hp-e30: the 0125 projection equals its Rust derivation after every writer of
// `messages`, on SQLite and on PostgreSQL (MW_TEST_PG, one private schema per test).
use super::*;
use crate::backend::Backend;
use crate::{AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey};

async fn sqlite() -> Store {
    Store::open_in_memory(ServerKey::generate()).await.unwrap()
}

/// Run `body` against a store in a fresh schema of MW_TEST_PG; skipped without it.
async fn postgres(test: &str, body: impl AsyncFnOnce(Store)) {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        mw_test_gate::skip(format!("MW_TEST_PG absent; {test} not executed"));
        return;
    };
    use sqlx::Connection;
    let schema = format!("projection_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&mut admin)
        .await
        .unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    let store = Store::open(scoped.as_str(), ServerKey::generate())
        .await
        .unwrap();
    body(store.clone()).await;
    store.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&mut admin)
        .await
        .unwrap();
}

fn leak(sql: String) -> &'static str {
    Box::leak(sql.into_boxed_str())
}

async fn exec(s: &Store, sql: &str) -> u64 {
    q(leak(sql.to_owned())).execute(&s.backend).await.unwrap()
}

async fn scalar(s: &Store, sql: &str) -> i64 {
    q(leak(sql.to_owned()))
        .fetch_scalar_i64(&s.backend)
        .await
        .unwrap()
}

async fn account(s: &Store) -> String {
    let name = format!("projection-{}@example.test", uuid::Uuid::new_v4().simple());
    s.create_account(
        &NewAccount {
            kind: AccountKind::Imap,
            host: "projection.test",
            port: 993,
            tls: "implicit",
            username: &name,
            sync_policy_json: "{}",
        },
        &Credentials {
            username: name.clone(),
            password: "synthetic-only".into(),
        },
    )
    .await
    .unwrap()
}

async fn mailbox(s: &Store, account: &str, name: &str) -> String {
    s.upsert_mailbox(&MailboxUpsert {
        account_id: account,
        name,
        role: None,
        uidvalidity: 100,
        uidnext: 1,
        highestmodseq: 0,
        total: 0,
        unread: 0,
        parent_id: None,
    })
    .await
    .unwrap()
}

fn upsert<'a>(
    account: &'a str,
    mailbox: &'a str,
    uid: u32,
    mid: &'a str,
    date: &'a str,
    flags: &'a str,
) -> MessageUpsert<'a> {
    MessageUpsert {
        account_id: account,
        mailbox_id: mailbox,
        uid,
        uidvalidity: 100,
        message_id: Some(mid),
        thread_id: None,
        internaldate: Some(date),
        size: 100 + u64::from(uid % 50) * 10,
        flags_json: flags,
        envelope: None,
        blob_ref: None,
    }
}

/// The audit finds nothing, for every account at once.
async fn assert_clean(s: &Store, context: &str) {
    let audit = s.audit_message_projection(None).await.unwrap();
    assert!(
        audit.is_clean(),
        "{context}: projection drift {} rows, first {:?}",
        audit.drift,
        audit.sample
    );
}

/// 0122's counters equal the projection: unread = COUNT(seen=0), total = rows.
async fn assert_counters(s: &Store, context: &str) {
    for r in q("SELECT b.id, b.total, b.unread,
            (SELECT CAST(COUNT(*) AS BIGINT) FROM message_state t WHERE t.mailbox_id = b.id) AS rows_total,
            (SELECT CAST(COUNT(*) AS BIGINT) FROM message_state t WHERE t.mailbox_id = b.id AND t.seen = 0) AS rows_unread
         FROM mailboxes b")
    .fetch_all(&s.backend)
    .await
    .unwrap()
    {
        assert_eq!(
            (r.get_i64("total"), r.get_i64("unread")),
            (r.get_i64("rows_total"), r.get_i64("rows_unread")),
            "{context}: folder {} counters vs projection",
            r.get_string("id")
        );
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

/// Elements for generated flag arrays: every system flag, case variants, a
/// keyword literally named "Seen", serde's unit-variant object form, escapes,
/// objects with extra or duplicate keys, wrong value types, non-strings.
const ELEMENTS: &[&str] = &[
    r#""Seen""#,
    r#""Seen""#,
    r#""Answered""#,
    r#""Flagged""#,
    r#""Deleted""#,
    r#""Draft""#,
    r#""Recent""#,
    r#""seen""#,
    r#""SEEN""#,
    r#""Keyword""#,
    r#""Seen""#,
    r#""""#,
    r#"{"Keyword":"Seen"}"#,
    r#"{"Keyword":"$seen"}"#,
    r#"{"Keyword":"$flagged"}"#,
    r#"{"Keyword":"$Junk"}"#,
    r#"{"Keyword":"Label"}"#,
    r#"{"Keyword":"label"}"#,
    r#"{"Keyword":"a b"}"#,
    r#"{"Keyword":"ünïcode"}"#,
    r#"{"Keyword":""}"#,
    r#"{"Keyword":"$mw-followup-active"}"#,
    r#"{"Keyword":"escaped-key"}"#,
    r#"{"Keyword":"x","extra":1}"#,
    r#"{"Keyword":"a","Keyword":"b"}"#,
    r#"{"Keyword":null}"#,
    r#"{"Keyword":5}"#,
    r#"{"Seen":null}"#,
    r#"{"Flagged":null}"#,
    r#"{"Deleted":null}"#,
    r#"{"Seen":1}"#,
    r#"{"Flagged":{}}"#,
    r#"{}"#,
    "5",
    "1.5",
    "null",
    "true",
    "[]",
    r#"["Seen"]"#,
];

/// Whole values that are not a flag array at all.
const WHOLE: &[&str] = &[
    "[]",
    "not json",
    "",
    "[",
    "null",
    "5",
    r#""Seen""#,
    r#"{"Seen":null}"#,
    r#"{"0":"Seen"}"#,
    r#" [ "Flagged" , "Seen" ] "#,
    "[\"Seen\"]\n",
    r#"["Seen"] trailing"#,
];

fn random_flags(rng: &mut Rng) -> String {
    if rng.below(8) == 0 {
        return WHOLE[rng.below(WHOLE.len())].to_owned();
    }
    let n = rng.below(5);
    let parts: Vec<&str> = (0..n)
        .map(|_| ELEMENTS[rng.below(ELEMENTS.len())])
        .collect();
    format!("[{}]", parts.join(","))
}

fn sql_text(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

// ---- derivation ----------------------------------------------------------------

#[test]
fn rust_derivation_follows_serde_and_the_counter_rule() {
    let k = |v: &str| projected_keywords(v).into_iter().collect::<Vec<_>>();
    assert_eq!(
        k(r#"["Seen","Flagged","Deleted","Recent",{"Keyword":"Label"}]"#),
        ["$flagged", "$seen", "Label"]
    );
    assert_eq!(
        k(r#"[{"Keyword":"Seen"},{"Keyword":"label"},{"Keyword":"Label"}]"#),
        ["Label", "Seen", "label"]
    );
    assert_eq!(k(r#"[{"Seen":null},{"Keyword":"x"}]"#), ["$seen", "x"]);
    // One element that is not a Flag: no keyword at all (flags_from_json).
    assert!(k(r#"["Seen",5]"#).is_empty());
    assert!(k(r#"["Seen",{"Keyword":"x","extra":1}]"#).is_empty());
    assert!(k(r#"[{"Keyword":"a","Keyword":"b"}]"#).is_empty());
    assert!(k(r#"["seen"]"#).is_empty());
    // seen is the counter rule: the JSON string "Seen" in any JSON array.
    assert!(projected_seen(r#"["Seen",5]"#));
    assert!(!projected_seen(r#"[{"Seen":null}]"#));
    assert!(!projected_seen(r#"[{"Keyword":"Seen"}]"#));
    assert!(!projected_seen("not json"));
}

/// The mirror enum and the keyword mapping are the engine's (mw-store cannot
/// depend on mw-engine, so this reads its source).
#[test]
fn flag_mirror_matches_the_engine_source() {
    let backend = include_str!("../../mw-engine/src/backend.rs");
    let squash = |s: &str| s.split_whitespace().collect::<String>();
    let start = backend
        .find("pub enum Flag {")
        .expect("mw-engine Flag enum");
    let body = &backend[start..start + backend[start..].find('}').unwrap()];
    let variants: Vec<String> = body
        .lines()
        .skip(1)
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .map(squash)
        .collect();
    assert_eq!(
        variants,
        [
            "Seen,",
            "Answered,",
            "Flagged,",
            "Deleted,",
            "Draft,",
            "Recent,",
            "Keyword(String),"
        ]
    );
    let derive = &backend[..start];
    let derive = derive[derive.rfind("#[derive(").unwrap()..].to_owned();
    assert!(
        derive.contains("Deserialize") && !derive.contains("serde("),
        "Flag uses serde's default externally tagged form: {derive}"
    );
    let mapping = squash(include_str!("../../mw-engine/src/mapping.rs"));
    for arm in [
        r#"Flag::Seen=>Some("$seen".to_string()),"#,
        r#"Flag::Answered=>Some("$answered".to_string()),"#,
        r#"Flag::Flagged=>Some("$flagged".to_string()),"#,
        r#"Flag::Draft=>Some("$draft".to_string()),"#,
        "Flag::Keyword(k)=>Some(k.clone()),",
        "Flag::Deleted|Flag::Recent=>None,",
        "serde_json::from_str(json).unwrap_or_default()",
    ] {
        assert!(
            mapping.contains(arm),
            "mw-engine mapping.rs no longer has `{arm}`"
        );
    }
}

async fn derivation_contract(s: Store) {
    let a = account(&s).await;
    let inbox = mailbox(&s, &a, "INBOX").await;
    let mut rng = Rng(0x5EED_F1A6);
    let mut values: Vec<String> = ELEMENTS.iter().map(|e| format!("[{e}]")).collect();
    values.extend(WHOLE.iter().map(|w| (*w).to_owned()));
    values.extend((0..300).map(|_| random_flags(&mut rng)));
    // INSERT path (raw, so nothing in Rust touches the value on the way).
    for (n, flags) in values.iter().enumerate() {
        exec(
            &s,
            &format!(
                "INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,internaldate,size,flags_json) VALUES('d{n}','{a}','{inbox}',{},1,'2026-07-01T10:00:00Z',{},{})",
                n + 1,
                n * 3,
                sql_text(flags)
            ),
        )
        .await;
    }
    assert_clean(&s, "after raw inserts").await;
    // Spot checks against the rule, independent of the audit's own derivation.
    let state = |id: &'static str| {
        q(leak(format!(
            "SELECT seen, flagged FROM message_state WHERE stable_id='{id}'"
        )))
    };
    let first = ELEMENTS
        .iter()
        .position(|e| *e == r#"{"Seen":null}"#)
        .unwrap();
    let r = state(leak(format!("d{first}")))
        .fetch_one(&s.backend)
        .await
        .unwrap();
    assert_eq!((r.get_i64("seen"), r.get_i64("flagged")), (0, 0));
    // UPDATE path: every row gets another value, twice.
    for round in 0..2 {
        for n in 0..values.len() {
            let flags = random_flags(&mut rng);
            exec(
                &s,
                &format!(
                    "UPDATE messages SET flags_json={} WHERE stable_id='d{n}'",
                    sql_text(&flags)
                ),
            )
            .await;
        }
        assert_clean(&s, &format!("after raw update round {round}")).await;
    }
    // Backfill path: the migration's own statements over the same rows.
    exec(&s, "DELETE FROM message_keyword").await;
    exec(&s, "DELETE FROM message_state").await;
    assert!(
        !s.audit_message_projection(None).await.unwrap().is_clean(),
        "an emptied projection must show as drift"
    );
    run_section(&s, "-- backfill", "-- end backfill").await;
    assert_clean(&s, "after the 0125 backfill").await;
}

fn migration(s: &Store) -> &'static str {
    match s.backend.dialect() {
        Dialect::Sqlite => include_str!("../migrations/0125_message_projection.sql"),
        Dialect::Postgres => include_str!("../migrations_pg/0125_message_projection.sql"),
    }
}

/// Run the part of 0125 between two marker lines.
async fn run_section(s: &Store, from: &str, to: &str) {
    let sql = migration(s);
    let start = sql.find(from).expect("0125 marker");
    let end = start + sql[start..].find(to).expect("0125 end marker");
    let section = &sql[start..end];
    match &s.backend {
        Backend::Sqlite(pool) => {
            sqlx::raw_sql(section).execute(pool).await.unwrap();
        }
        Backend::Postgres(pool) => {
            sqlx::raw_sql(section).execute(pool).await.unwrap();
        }
    }
}

#[tokio::test]
async fn sql_derivation_equals_rust_over_generated_flags_sqlite() {
    derivation_contract(sqlite().await).await;
}
#[tokio::test]
async fn sql_derivation_equals_rust_over_generated_flags_postgres() {
    postgres(
        "sql_derivation_equals_rust_over_generated_flags_postgres",
        derivation_contract,
    )
    .await;
}

// ---- every writer of messages --------------------------------------------------

struct World {
    account: String,
    other: String,
    boxes: Vec<String>,
    other_box: String,
    uid: std::sync::atomic::AtomicU32,
}

async fn world(s: &Store) -> World {
    let account = account(s).await;
    let other = self::account(s).await;
    let mut boxes = Vec::new();
    for name in ["INBOX", "Archive", "Trash"] {
        boxes.push(mailbox(s, &account, name).await);
    }
    let other_box = mailbox(s, &other, "INBOX").await;
    World {
        account,
        other,
        boxes,
        other_box,
        uid: std::sync::atomic::AtomicU32::new(1),
    }
}

async fn pick(s: &Store, w: &World, rng: &mut Rng) -> Option<crate::Message> {
    let ids = q("SELECT stable_id FROM messages WHERE account_id = ?1 ORDER BY stable_id")
        .bind(&w.account)
        .fetch_all_scalar_string(&s.backend)
        .await
        .unwrap();
    if ids.is_empty() {
        return None;
    }
    s.get_message(&ids[rng.below(ids.len())]).await.ok()
}

/// One write through a public Store function (the writers of the t50hp-e26 path
/// table plus the ones that do not change projected columns). Lost races
/// (`NotFound`, a refused compare-and-set) are part of the game.
async fn writer_op(s: &Store, w: &World, rng: &mut Rng) -> &'static str {
    let target = w.boxes[rng.below(w.boxes.len())].clone();
    let flags = random_flags(rng);
    let uid = w.uid.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let mid = format!("<w{uid}@projection.test>");
    let date = format!(
        "2026-07-{:02}T10:00:{:02}Z",
        1 + rng.below(28),
        rng.below(60)
    );
    match rng.below(17) {
        0 => {
            s.upsert_message(&upsert(&w.account, &target, uid, &mid, &date, &flags))
                .await
                .unwrap();
            "upsert_message"
        }
        1 => {
            s.ingest_message_with_body(
                &upsert(&w.account, &target, uid, &mid, &date, &flags),
                b"body",
            )
            .await
            .unwrap();
            "ingest_message_with_body"
        }
        2 => {
            s.upsert_message_with_body(
                &upsert(&w.account, &target, uid, &mid, &date, &flags),
                b"body",
            )
            .await
            .unwrap();
            "upsert_message_with_body"
        }
        3 => {
            let mid2 = format!("<w{uid}b@projection.test>");
            let first = upsert(&w.account, &target, uid + 100_000, &mid, &date, &flags);
            let second = upsert(
                &w.account,
                &target,
                uid + 200_000,
                &mid2,
                &date,
                r#"["Flagged"]"#,
            );
            s.upsert_message_bodies_batch(&[
                (first, b"one".as_slice()),
                (second, b"two".as_slice()),
            ])
            .await
            .unwrap();
            "upsert_message_bodies_batch"
        }
        4 => {
            // A sync replay of an existing row with new flags.
            if let Some(cur) = pick(s, w, rng).await {
                let mid = cur.message_id.clone().unwrap_or_default();
                let date = cur.internaldate.clone().unwrap_or_default();
                let mut m = upsert(&w.account, &cur.mailbox_id, cur.uid, &mid, &date, &flags);
                m.uidvalidity = cur.uidvalidity;
                let _ = s.upsert_message(&m).await;
            }
            "re-upsert"
        }
        5 => {
            if let Some(cur) = pick(s, w, rng).await {
                let other = pick(s, w, rng)
                    .await
                    .map(|m| m.stable_id)
                    .unwrap_or_default();
                let _ = s
                    .set_flags_batch(&[
                        (cur.stable_id, flags.clone()),
                        (other, r#"["Seen"]"#.into()),
                    ])
                    .await;
            }
            "set_flags_batch"
        }
        6 => {
            if let Some(cur) = pick(s, w, rng).await {
                let _ = s.set_flags(&cur.stable_id, &flags).await;
            }
            "set_flags"
        }
        7 => {
            if let Some(cur) = pick(s, w, rng).await {
                let _ = s.set_flags_if_current(&cur, &flags).await;
            }
            "set_flags_if_current (sync, rules, Sieve)"
        }
        8 => {
            if let Some(cur) = pick(s, w, rng).await {
                let _ = s
                    .relocate_message(&cur.stable_id, &target, uid + 300_000, 100)
                    .await;
            }
            "relocate_message"
        }
        9 => {
            if let Some(cur) = pick(s, w, rng).await {
                let _ = s
                    .relocate_message_if_current(&cur, &target, uid + 400_000, 100)
                    .await;
            }
            "relocate_message_if_current (move, Mailbox/empty, rules, Sieve)"
        }
        10 => {
            if let Some(cur) = pick(s, w, rng).await {
                let _ = s.delete_message(&cur.stable_id).await;
            }
            "delete_message"
        }
        11 => {
            if let Some(cur) = pick(s, w, rng).await {
                let _ = s.delete_message_if_current(&cur).await;
            }
            "delete_message_if_current"
        }
        12 => {
            if let Some(cur) = pick(s, w, rng).await {
                let _ = s.delete_synced_message_if_source(&cur).await;
            }
            "delete_synced_message_if_source"
        }
        13 => {
            // A raw IMAP INTERNALDATE, then the normaliser rewrites it.
            let raw_date = format!("{:02}-Jul-2026 10:00:00 +0200", 1 + rng.below(28));
            s.upsert_message(&upsert(&w.account, &target, uid, &mid, &raw_date, &flags))
                .await
                .unwrap();
            s.normalize_cached_imap_dates(&w.account).await.unwrap();
            "normalize_cached_imap_dates"
        }
        14 => {
            if let Some(cur) = pick(s, w, rng).await {
                s.set_thread(&cur.stable_id, &format!("thread-{uid}"))
                    .await
                    .unwrap();
                // The body-cache expiry and envelope repair statements touch only
                // blob_ref/envelope_json (body_cache_expiry.rs, cache.rs repair_*).
                exec(s, &format!("UPDATE messages SET blob_ref=NULL, envelope_json=NULL WHERE stable_id='{}'", cur.stable_id)).await;
            }
            "set_thread + body-expiry-shaped update"
        }
        15 => {
            // A folder with mail goes (delete_mailbox: rows, then the folder).
            let name = format!("Tmp{uid}");
            let tmp = mailbox(s, &w.account, &name).await;
            s.upsert_message(&upsert(&w.account, &tmp, uid, &mid, &date, &flags))
                .await
                .unwrap();
            if let Some(cur) = pick(s, w, rng).await {
                let _ = s
                    .relocate_message(&cur.stable_id, &tmp, uid + 500_000, 100)
                    .await;
            }
            s.delete_mailbox(&tmp).await.unwrap();
            "delete_mailbox"
        }
        _ => {
            // The other account writes too; it must never leak into this one.
            s.upsert_message(&upsert(&w.other, &w.other_box, uid, &mid, &date, &flags))
                .await
                .unwrap();
            "other account ingest"
        }
    }
}

async fn writers_contract(s: Store, seeds: &[u64], steps: usize) {
    for &seed in seeds {
        let w = world(&s).await;
        let mut rng = Rng(seed);
        for step in 0..steps {
            let op = writer_op(&s, &w, &mut rng).await;
            let context = format!("seed {seed} step {step} after {op}");
            assert_clean(&s, &context).await;
            assert_counters(&s, &context).await;
        }
    }
}

#[tokio::test]
async fn projection_has_no_drift_after_every_writer_sqlite() {
    writers_contract(sqlite().await, &[1, 2, 0xDEAD_BEEF], 220).await;
}
#[tokio::test]
async fn projection_has_no_drift_after_every_writer_postgres() {
    postgres(
        "projection_has_no_drift_after_every_writer_postgres",
        async |s| writers_contract(s, &[3, 0x1234_5678], 140).await,
    )
    .await;
}

/// Two connections writing the same folders at once.
async fn concurrent_contract(s: Store, steps: usize) {
    let w = world(&s).await;
    let mut rng = Rng(11);
    for _ in 0..20 {
        writer_op(&s, &w, &mut rng).await;
    }
    let run = |seed: u64| {
        let (s, w) = (&s, &w);
        async move {
            let mut rng = Rng(seed);
            for _ in 0..steps {
                writer_op(s, w, &mut rng).await;
            }
        }
    };
    tokio::join!(run(0x9E37_79B9), run(0x85EB_CA6B));
    assert_clean(&s, "after two concurrent writers").await;
    assert_counters(&s, "after two concurrent writers").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn projection_has_no_drift_under_two_concurrent_writers_sqlite() {
    let dir = tempfile::tempdir().unwrap();
    let s = Store::open(
        dir.path().join("p.db").to_str().unwrap(),
        ServerKey::generate(),
    )
    .await
    .unwrap();
    concurrent_contract(s.clone(), 150).await;
    s.close().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn projection_has_no_drift_under_two_concurrent_writers_postgres() {
    postgres(
        "projection_has_no_drift_under_two_concurrent_writers_postgres",
        async |s| concurrent_contract(s, 120).await,
    )
    .await;
}

/// Statements no Store function issues today: renames, owner changes, conflict
/// clauses, FK cascades, column writes outside the projection, no-op updates.
async fn raw_writers_contract(s: Store) {
    let w = world(&s).await;
    let (a, inbox, archive) = (&w.account, &w.boxes[0], &w.boxes[1]);
    let insert = |id: &str, uid: u32, flags: &str, mailbox: &str| {
        format!(
            "INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,internaldate,size,flags_json) VALUES('{id}','{a}','{mailbox}',{uid},1,'2026-07-01T10:00:00Z',10,{})",
            sql_text(flags)
        )
    };
    exec(&s, &insert("r1", 1, r#"["Seen",{"Keyword":"L"}]"#, inbox)).await;
    exec(&s, &insert("r2", 2, r#"["Flagged"]"#, inbox)).await;
    exec(&s, &insert("r3", 3, "[]", archive)).await;
    assert_clean(&s, "raw inserts").await;
    let steps: Vec<String> = vec![
        "UPDATE messages SET stable_id='r1-renamed' WHERE stable_id='r1'".into(),
        format!(
            "UPDATE messages SET account_id='{}' WHERE stable_id='r2'",
            w.other
        ),
        format!(
            "UPDATE messages SET account_id='{a}', flags_json='[\"Seen\"]' WHERE stable_id='r2'"
        ),
        "UPDATE messages SET internaldate=NULL, size=size+7, uid=uid+1000 WHERE stable_id='r3'"
            .into(),
        "UPDATE messages SET internaldate='2027-01-01T00:00:00Z' WHERE stable_id='r3'".into(),
        "UPDATE messages SET flags_json=flags_json".into(),
        "UPDATE messages SET thread_id='t', message_id='<m>', uidvalidity=uidvalidity+1".into(),
        "UPDATE messages SET cache_revision='', content_revision=''".into(),
        "UPDATE messages SET flags_json='not json' WHERE stable_id='r3'".into(),
        format!(
            "UPDATE messages SET mailbox_id='{archive}', flags_json='[{{\"Keyword\":\"M\"}}]' WHERE stable_id='r1-renamed'"
        ),
        format!("DELETE FROM mailboxes WHERE id='{archive}'"),
        insert(
            "r4",
            4,
            r#"["Seen","Seen",{"Keyword":"L"},{"Keyword":"L"}]"#,
            inbox,
        ),
    ];
    for sql in &steps {
        exec(&s, sql).await;
        assert_clean(&s, sql).await;
    }
    if s.backend.dialect() == Dialect::Sqlite {
        // REPLACE deletes the old row (delete trigger, recursive_triggers ON) and inserts.
        exec(&s, &format!("INSERT OR REPLACE INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,flags_json) VALUES('r4','{a}','{inbox}',4,1,'[\"Draft\"]')")).await;
        assert_clean(&s, "INSERT OR REPLACE").await;
        // An outer conflict clause cannot turn a projection insert into an error or a skip.
        exec(&s, &format!("INSERT OR IGNORE INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,flags_json) VALUES('r5','{a}','{inbox}',5,1,'[\"Seen\",\"Seen\"]')")).await;
        exec(&s, &format!("INSERT OR FAIL INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,flags_json) VALUES('r6','{a}','{inbox}',6,1,'[{{\"Keyword\":\"x\"}},{{\"Keyword\":\"x\"}}]')")).await;
        exec(&s, "UPDATE OR REPLACE messages SET flags_json='[\"Answered\",\"Answered\"]' WHERE stable_id='r6'").await;
        assert_clean(&s, "outer conflict clauses").await;
    } else {
        // Rows written with every trigger off (as the t61 seeding does) heal on
        // their next write; until then the audit reports them.
        let Backend::Postgres(pool) = &s.backend else {
            unreachable!()
        };
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SET LOCAL session_replication_role = replica")
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query(leak(insert("r7", 7, r#"["Seen"]"#, inbox)))
            .execute(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        let audit = s.audit_message_projection(Some(a)).await.unwrap();
        assert_eq!(
            audit.sample,
            [
                ProjectionDrift::MissingState {
                    stable_id: "r7".into()
                },
                ProjectionDrift::MissingKeyword {
                    stable_id: "r7".into(),
                    keyword: "$seen".into()
                }
            ]
        );
        exec(
            &s,
            "UPDATE messages SET flags_json='[\"Seen\",\"Flagged\"]' WHERE stable_id='r7'",
        )
        .await;
        exec(&s, "UPDATE messages SET size=size+1").await;
        assert_clean(&s, "after the trigger-less rows were written again").await;
    }
    // The account cascade removes the rest.
    exec(&s, &format!("DELETE FROM accounts WHERE id='{a}'")).await;
    assert_clean(&s, "account cascade").await;
    assert_eq!(
        scalar(&s, &format!("SELECT (SELECT COUNT(*) FROM message_state WHERE account_id='{a}')+(SELECT COUNT(*) FROM message_keyword WHERE account_id='{a}')")).await,
        0
    );
}

#[tokio::test]
async fn projection_follows_raw_sql_writers_sqlite() {
    raw_writers_contract(sqlite().await).await;
}
#[tokio::test]
async fn projection_follows_raw_sql_writers_postgres() {
    postgres(
        "projection_follows_raw_sql_writers_postgres",
        raw_writers_contract,
    )
    .await;
}

// ---- erasure ------------------------------------------------------------------------

async fn erasure_contract(s: Store) {
    let w = world(&s).await;
    let mut rng = Rng(77);
    for _ in 0..40 {
        writer_op(&s, &w, &mut rng).await;
    }
    for (n, b) in w.boxes.iter().enumerate() {
        s.upsert_message(&upsert(
            &w.account,
            b,
            900 + n as u32,
            "<e@x>",
            "2026-01-01T00:00:00Z",
            r#"["Seen",{"Keyword":"E"}]"#,
        ))
        .await
        .unwrap();
    }
    // Rows the triggers would never leave (as if written with triggers off): the
    // erasure registry removes them by account all the same.
    exec(&s, &format!("INSERT INTO message_state(stable_id,account_id,mailbox_id,internaldate,uid,size,seen,flagged) VALUES('ghost','{}','gone',NULL,1,1,0,0)", w.account)).await;
    exec(
        &s,
        &format!(
            "INSERT INTO message_keyword(stable_id,account_id,keyword) VALUES('ghost','{}','G')",
            w.account
        ),
    )
    .await;
    let count = |table: &'static str| {
        let s = &s;
        let a = w.account.clone();
        async move {
            scalar(
                s,
                &format!("SELECT COUNT(*) FROM {table} WHERE account_id='{a}'"),
            )
            .await
        }
    };
    assert!(count("message_state").await > 1 && count("message_keyword").await > 1);
    let other_before = s.audit_message_projection(Some(&w.other)).await.unwrap();
    let person = crate::erasure::PersonIds {
        address: "projection@example.test".into(),
        account_ids: vec![w.account.clone()],
        owners: Vec::new(),
        principals: Vec::new(),
    };
    let (removed, errors) = s.dsr_purge_person(&person).await;
    assert!(errors.is_empty(), "{errors:?}");
    assert!(
        removed.contains_key("message_state") && removed.contains_key("message_keyword"),
        "{removed:?}"
    );
    assert_eq!(
        (count("message_state").await, count("message_keyword").await),
        (0, 0)
    );
    let left = s.dsr_verify_person(&person).await;
    assert!(
        !left.contains_key("message_state") && !left.contains_key("message_keyword"),
        "{left:?}"
    );
    let other_after = s.audit_message_projection(Some(&w.other)).await.unwrap();
    assert!(
        other_after.is_clean()
            && other_after.messages == other_before.messages
            && other_after.messages > 0
    );
}

#[tokio::test]
async fn erasure_leaves_no_projection_rows_sqlite() {
    erasure_contract(sqlite().await).await;
}
#[tokio::test]
async fn erasure_leaves_no_projection_rows_postgres() {
    postgres(
        "erasure_leaves_no_projection_rows_postgres",
        erasure_contract,
    )
    .await;
}

// ---- guard --------------------------------------------------------------------------

fn rust_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            if !matches!(
                path.file_name().and_then(|n| n.to_str()),
                Some("tests" | "target" | "node_modules")
            ) {
                rust_files(&path, out);
            }
        } else if path.extension().is_some_and(|e| e == "rs")
            && !path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with("_tests.rs") || n == "tests.rs")
        {
            out.push(path);
        }
    }
}

/// The replacement for e12's "every mutable table gets bump triggers" (design
/// C5): nothing outside the 0125 trigger bodies and the erasure registry writes
/// the two tables, in any crate's shipped code and in any migration.
#[test]
fn only_the_triggers_and_erasure_write_the_projection() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    assert!(
        files.len() > 100,
        "the crates tree was not found from {root:?}"
    );
    let verbs = [
        "INSERT INTO",
        "INSERT OR IGNORE INTO",
        "INSERT OR REPLACE INTO",
        "REPLACE INTO",
        "UPDATE",
        "UPDATE OR REPLACE",
        "DELETE FROM",
        "TRUNCATE",
        "TRUNCATE TABLE",
    ];
    let mut found = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap_or_default();
        // A test module at the end of a shipped file is test code.
        let shipped = text
            .split("#[cfg(test)]")
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        for table in ["MESSAGE_STATE", "MESSAGE_KEYWORD"] {
            for verb in verbs {
                let needle = format!("{verb} {table}");
                for (at, _) in shipped.match_indices(&needle) {
                    let next = shipped[at + needle.len()..].chars().next().unwrap_or(' ');
                    if !(next.is_ascii_alphanumeric() || next == '_') {
                        found.push(format!("{}: {needle}", file.display()));
                    }
                }
            }
        }
        if file.ends_with("data_map.rs") {
            for table in ["message_state", "message_keyword"] {
                assert!(
                    text.contains(&format!("del!(\"{table}\", \"account_id\", Bind::Account)")),
                    "erasure rule for {table}"
                );
            }
        }
    }
    assert!(
        found.is_empty(),
        "projection written outside the 0125 triggers: {found:#?}"
    );
    for dir in ["migrations", "migrations_pg"] {
        for entry in std::fs::read_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(dir))
            .unwrap()
            .flatten()
        {
            let name = entry.file_name().to_string_lossy().into_owned();
            let text = std::fs::read_to_string(entry.path())
                .unwrap()
                .to_ascii_lowercase();
            if !name.starts_with("0125_") {
                assert!(
                    !text.contains("message_state") && !text.contains("message_keyword"),
                    "{dir}/{name} touches the projection"
                );
            }
        }
    }
}

async fn installed_contract(s: Store) {
    assert!(s.message_projection_installed().await.unwrap());
    let disable = match s.backend.dialect() {
        Dialect::Sqlite => "DROP TRIGGER message_projection_update",
        Dialect::Postgres => "ALTER TABLE messages DISABLE TRIGGER message_projection_update",
    };
    exec(&s, disable).await;
    assert!(
        !s.message_projection_installed().await.unwrap(),
        "a missing or disabled trigger must be reported"
    );
}

#[tokio::test]
async fn projection_triggers_are_reported_sqlite() {
    installed_contract(sqlite().await).await;
}
#[tokio::test]
async fn projection_triggers_are_reported_postgres() {
    postgres(
        "projection_triggers_are_reported_postgres",
        installed_contract,
    )
    .await;
}

// ---- mail-data schema stamp ---------------------------------------------------------

/// New triggers on `messages` change the mail-data schema stamp once: a fence
/// taken before no longer validates, a new one does, and validation never fails
/// because of the projection's own triggers (they are not in its required set).
async fn stamp_contract(s: Store) {
    let a = account(&s).await;
    let inbox = mailbox(&s, &a, "INBOX").await;
    let envelope = serde_json::to_vec(&serde_json::json!({"subject":"stamp","messageId":["s@x"],"inReplyTo":[],"references":[],"hasAttachment":false,"attachments":[]})).unwrap();
    let id = s
        .upsert_message(&MessageUpsert {
            envelope: Some(&envelope),
            ..upsert(&a, &inbox, 1, "<s@x>", "2026-01-01T00:00:00Z", "[]")
        })
        .await
        .unwrap();
    let descriptor = s
        .mail_data_descriptor(&a, std::slice::from_ref(&id))
        .unwrap();
    s.seed_mail_data(&descriptor).await.unwrap();
    let before = s.mail_data_snapshot(&descriptor).await.unwrap().unwrap();
    assert!(
        before.publishable(),
        "fence not publishable: {:?}",
        before.schema_stamp()
    );
    assert!(
        s.validate_mail_data_schema(&before).await.unwrap(),
        "fresh fence must validate"
    );
    // Re-create one projection trigger: what an upgrade that adds triggers does to the stamp.
    let sql = migration(&s);
    let (drop, create) = match s.backend.dialect() {
        Dialect::Sqlite => (
            "DROP TRIGGER message_projection_delete",
            &sql[sql
                .find("CREATE TRIGGER message_projection_delete")
                .unwrap()..],
        ),
        Dialect::Postgres => ("DROP TRIGGER message_projection_delete ON messages", {
            let at = sql
                .find("CREATE TRIGGER message_projection_delete")
                .unwrap();
            &sql[at..at + sql[at..].find(';').unwrap() + 1]
        }),
    };
    exec(&s, drop).await;
    match &s.backend {
        Backend::Sqlite(pool) => {
            sqlx::raw_sql(create).execute(pool).await.unwrap();
        }
        Backend::Postgres(pool) => {
            sqlx::raw_sql(create).execute(pool).await.unwrap();
        }
    }
    assert!(
        !s.validate_mail_data_schema(&before).await.unwrap(),
        "a fence from before the DDL must not validate"
    );
    let after = s.mail_data_snapshot(&descriptor).await.unwrap().unwrap();
    assert_ne!(after.schema_stamp(), before.schema_stamp());
    assert!(
        after.publishable() && s.validate_mail_data_schema(&after).await.unwrap(),
        "the new stamp validates"
    );
}

#[tokio::test]
async fn new_projection_triggers_restamp_mail_data_once_sqlite() {
    stamp_contract(sqlite().await).await;
}
#[tokio::test]
async fn new_projection_triggers_restamp_mail_data_once_postgres() {
    postgres(
        "new_projection_triggers_restamp_mail_data_once_postgres",
        stamp_contract,
    )
    .await;
}

// ---- query builders -----------------------------------------------------------------

/// Brute force over `messages` + `message_meta` for one query (the oracle).
async fn oracle(s: &Store, query: &ProjectionQuery<'_>) -> Vec<String> {
    let rows = q("SELECT m.stable_id, m.account_id, m.mailbox_id, m.internaldate, m.uid, m.size, m.flags_json,
            CAST(COALESCE(mm.pinned, 0) AS BIGINT) AS pinned, mm.follow_up_at,
            CAST(CASE WHEN b.account_id = m.account_id THEN 1 ELSE 0 END AS BIGINT) AS owned
         FROM messages m LEFT JOIN message_meta mm ON mm.stable_id = m.stable_id LEFT JOIN mailboxes b ON b.id = m.mailbox_id")
    .fetch_all(&s.backend)
    .await
    .unwrap();
    let date = |v: Option<i64>| {
        v.map(|v| {
            chrono::DateTime::from_timestamp(v, 0)
                .unwrap()
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        })
    };
    let (after, before) = (date(query.received_after), date(query.received_before));
    let mut hits: Vec<(Option<String>, i64, i64, String)> = Vec::new();
    for r in rows {
        let id = r.get_string("stable_id");
        let mailbox = r.get_string("mailbox_id");
        let flags = r.get_string("flags_json");
        let kws = projected_keywords(&flags);
        let has = |k: &str| match k {
            "$seen" => projected_seen(&flags),
            "$flagged" => kws.contains("$flagged"),
            k => kws.contains(k),
        };
        let in_scope = match query.mailboxes {
            MailboxScope::Account => true,
            MailboxScope::In(ids) => ids.contains(&mailbox) && r.get_i64("owned") == 1,
            MailboxScope::NotIn(ids) => !ids.contains(&mailbox),
        };
        let idate = r.get_opt_string("internaldate");
        let dated = match &idate {
            None => {
                query.received_after.is_none_or(|a| a <= 0)
                    && query.received_before.is_none_or(|b| b > 0)
            }
            Some(d) => {
                after.as_ref().is_none_or(|a| d >= a) && before.as_ref().is_none_or(|b| d < b)
            }
        };
        let size = r.get_i64("size");
        let ok = r.get_string("account_id") == query.account_id
            && in_scope
            && query.seen.is_none_or(|v| v == projected_seen(&flags))
            && query.flagged.is_none_or(|v| v == kws.contains("$flagged"))
            && query.keywords.iter().all(|k| has(k))
            && !query.not_keywords.iter().any(|k| has(k))
            && query.pinned.is_none_or(|p| p == (r.get_i64("pinned") != 0))
            && (!query.follow_up
                || ((r.get_opt_string("follow_up_at").is_some()
                    || kws.contains("$flagged")
                    || kws.contains("$mw-followup-active"))
                    && !kws.contains("mw-followup-completed")))
            && dated
            && query.size_larger.is_none_or(|v| size > v as i64)
            && query.size_smaller.is_none_or(|v| size < v as i64)
            && query.within.is_none_or(|ids| ids.contains(&id));
        if ok {
            hits.push((idate, r.get_i64("uid"), size, id));
        }
    }
    let desc = |x: &(Option<String>, i64, i64, String), y: &(Option<String>, i64, i64, String)| {
        match query.sort {
            ProjectionSort::ReceivedDesc | ProjectionSort::ReceivedAsc => {
                // NULLS LAST for descending dates.
                let d = match (&x.0, &y.0) {
                    (Some(a), Some(b)) => b.cmp(a),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                };
                d.then(y.1.cmp(&x.1)).then(x.3.cmp(&y.3))
            }
            ProjectionSort::SizeDesc | ProjectionSort::SizeAsc => {
                y.2.cmp(&x.2).then(y.1.cmp(&x.1)).then(x.3.cmp(&y.3))
            }
        }
    };
    hits.sort_by(desc);
    if matches!(
        query.sort,
        ProjectionSort::ReceivedAsc | ProjectionSort::SizeAsc
    ) {
        hits.reverse();
    }
    hits.into_iter().map(|h| h.3).collect()
}

async fn query_world(s: &Store) -> (World, Vec<String>) {
    let w = world(s).await;
    let mut ids = Vec::new();
    let mut rng = Rng(0xC0FFEE);
    let flag_sets = [
        "[]",
        r#"["Seen"]"#,
        r#"["Seen","Flagged"]"#,
        r#"["Flagged"]"#,
        r#"["Seen",{"Keyword":"L"}]"#,
        r#"[{"Keyword":"L"},{"Keyword":"M"}]"#,
        r#"[{"Keyword":"mw-followup-completed"},"Flagged"]"#,
        r#"[{"Keyword":"$mw-followup-active"}]"#,
        r#"["Seen",5]"#,
    ];
    for n in 0..120u32 {
        let b = &w.boxes[rng.below(w.boxes.len())];
        let flags = flag_sets[rng.below(flag_sets.len())];
        // Equal dates on purpose, so the uid and id tie-breakers decide.
        let date = format!("2026-07-{:02}T10:00:00Z", 1 + rng.below(6));
        let mid = format!("<q{n}@x>");
        let mut m = upsert(&w.account, b, n + 1, &mid, &date, flags);
        if n % 17 == 0 {
            m.internaldate = None;
        }
        ids.push(s.upsert_message(&m).await.unwrap());
    }
    // Same folders, flags and keywords in the other account: never returned.
    for n in 0..30u32 {
        let flags = flag_sets[n as usize % flag_sets.len()];
        let mid = format!("<o{n}@x>");
        s.upsert_message(&upsert(
            &w.other,
            &w.other_box,
            n + 1,
            &mid,
            "2026-07-02T10:00:00Z",
            flags,
        ))
        .await
        .unwrap();
    }
    for (n, id) in ids.iter().enumerate() {
        if n % 5 == 0 || n % 7 == 0 {
            let meta = crate::StoredMeta {
                pinned: n % 5 == 0,
                snoozed_until: None,
                follow_up_at: (n % 7 == 0).then(|| "2026-08-01T00:00:00Z".to_owned()),
            };
            s.upsert_message_meta(id, &meta).await.unwrap();
        }
    }
    (w, ids)
}

async fn builders_contract(s: Store) {
    let (w, ids) = query_world(&s).await;
    let one = std::slice::from_ref(&w.boxes[0]);
    let two = [w.boxes[0].clone(), w.boxes[1].clone(), w.other_box.clone()];
    let foreign = std::slice::from_ref(&w.other_box);
    let l = ["L".to_owned()];
    let lm = ["L".to_owned(), "M".to_owned()];
    let seen_kw = ["$seen".to_owned()];
    let flagged_kw = ["$flagged".to_owned()];
    let within: Vec<String> = ids.iter().step_by(3).cloned().collect();
    let base = ProjectionQuery::account(&w.account);
    let mut cases: Vec<(&str, ProjectionQuery<'_>)> = Vec::new();
    cases.push(("account", base.clone()));
    cases.push((
        "unread in folder",
        ProjectionQuery {
            mailboxes: MailboxScope::In(one),
            seen: Some(false),
            ..base.clone()
        },
    ));
    cases.push((
        "unread account",
        ProjectionQuery {
            seen: Some(false),
            ..base.clone()
        },
    ));
    cases.push((
        "notKeyword $seen",
        ProjectionQuery {
            mailboxes: MailboxScope::In(one),
            not_keywords: &seen_kw,
            ..base.clone()
        },
    ));
    cases.push((
        "flagged",
        ProjectionQuery {
            flagged: Some(true),
            ..base.clone()
        },
    ));
    cases.push((
        "hasKeyword $flagged in folder",
        ProjectionQuery {
            mailboxes: MailboxScope::In(one),
            keywords: &flagged_kw,
            ..base.clone()
        },
    ));
    cases.push((
        "not flagged",
        ProjectionQuery {
            flagged: Some(false),
            ..base.clone()
        },
    ));
    cases.push((
        "keyword",
        ProjectionQuery {
            keywords: &l,
            ..base.clone()
        },
    ));
    cases.push((
        "two keywords",
        ProjectionQuery {
            keywords: &lm,
            ..base.clone()
        },
    ));
    cases.push((
        "notKeyword in folder",
        ProjectionQuery {
            mailboxes: MailboxScope::In(one),
            not_keywords: &l,
            ..base.clone()
        },
    ));
    cases.push((
        "folder by date",
        ProjectionQuery {
            mailboxes: MailboxScope::In(one),
            ..base.clone()
        },
    ));
    cases.push((
        "folder ascending",
        ProjectionQuery {
            mailboxes: MailboxScope::In(one),
            sort: ProjectionSort::ReceivedAsc,
            ..base.clone()
        },
    ));
    cases.push((
        "folders incl. a foreign one",
        ProjectionQuery {
            mailboxes: MailboxScope::In(&two),
            seen: Some(true),
            ..base.clone()
        },
    ));
    cases.push((
        "foreign folder only",
        ProjectionQuery {
            mailboxes: MailboxScope::In(foreign),
            ..base.clone()
        },
    ));
    cases.push((
        "other than folder",
        ProjectionQuery {
            mailboxes: MailboxScope::NotIn(one),
            ..base.clone()
        },
    ));
    cases.push((
        "pinned",
        ProjectionQuery {
            pinned: Some(true),
            ..base.clone()
        },
    ));
    cases.push((
        "not pinned unread",
        ProjectionQuery {
            pinned: Some(false),
            seen: Some(false),
            ..base.clone()
        },
    ));
    cases.push((
        "follow-up",
        ProjectionQuery {
            follow_up: true,
            ..base.clone()
        },
    ));
    cases.push((
        "after",
        ProjectionQuery {
            received_after: Some(1_782_986_400),
            ..base.clone()
        },
    ));
    cases.push((
        "before (undated count as 0)",
        ProjectionQuery {
            received_before: Some(1_782_986_400),
            ..base.clone()
        },
    ));
    cases.push((
        "date window",
        ProjectionQuery {
            received_after: Some(1_782_900_000),
            received_before: Some(1_783_100_000),
            ..base.clone()
        },
    ));
    cases.push((
        "size window",
        ProjectionQuery {
            size_larger: Some(150),
            size_smaller: Some(400),
            ..base.clone()
        },
    ));
    cases.push((
        "size sort in folder",
        ProjectionQuery {
            mailboxes: MailboxScope::In(one),
            sort: ProjectionSort::SizeDesc,
            ..base.clone()
        },
    ));
    cases.push((
        "size ascending unread",
        ProjectionQuery {
            seen: Some(false),
            sort: ProjectionSort::SizeAsc,
            ..base.clone()
        },
    ));
    cases.push((
        "within",
        ProjectionQuery {
            within: Some(&within),
            seen: Some(false),
            ..base.clone()
        },
    ));
    cases.push((
        "contradiction",
        ProjectionQuery {
            keywords: &seen_kw,
            seen: Some(false),
            ..base.clone()
        },
    ));
    cases.push(("other account", ProjectionQuery::account(&w.other)));
    for (name, query) in &cases {
        let expected = oracle(&s, query).await;
        let got = s.projection_ids(query, 1000, 0).await.unwrap();
        assert_eq!(got, expected, "{name}: ids");
        assert_eq!(
            s.projection_count(query).await.unwrap(),
            expected.len() as u64,
            "{name}: count"
        );
        let page = s.projection_ids(query, 5, 3).await.unwrap();
        assert_eq!(
            page,
            expected.iter().skip(3).take(5).cloned().collect::<Vec<_>>(),
            "{name}: window"
        );
        if *name != "other account" {
            let other: BTreeSet<String> = s
                .projection_ids(&ProjectionQuery::account(&w.other), 1000, 0)
                .await
                .unwrap()
                .into_iter()
                .collect();
            assert!(
                got.iter().all(|id| !other.contains(id)),
                "{name}: another account's message returned"
            );
        }
    }
    let unread_folder = ProjectionQuery {
        mailboxes: MailboxScope::In(one),
        seen: Some(false),
        ..base.clone()
    };
    assert_eq!(
        s.projection_count(&unread_folder).await.unwrap(),
        u64::from(s.get_mailbox(&w.boxes[0]).await.unwrap().unread),
        "the unread list and the folder badge agree"
    );
    let filtered = s
        .projection_filter_ids(
            &ProjectionQuery {
                seen: Some(true),
                ..base.clone()
            },
            &within,
        )
        .await
        .unwrap();
    assert_eq!(
        filtered,
        oracle(
            &s,
            &ProjectionQuery {
                seen: Some(true),
                within: Some(&within),
                ..base.clone()
            }
        )
        .await
    );
    assert!(
        s.projection_ids(
            &ProjectionQuery {
                within: Some(&[]),
                ..base.clone()
            },
            10,
            0
        )
        .await
        .unwrap()
        .is_empty()
    );
    assert!(
        s.projection_ids(
            &ProjectionQuery {
                mailboxes: MailboxScope::In(&[]),
                ..base.clone()
            },
            10,
            0
        )
        .await
        .unwrap()
        .is_empty()
    );
}

#[tokio::test]
async fn query_builders_equal_the_oracle_sqlite() {
    builders_contract(sqlite().await).await;
}
#[tokio::test]
async fn query_builders_equal_the_oracle_postgres() {
    postgres(
        "query_builders_equal_the_oracle_postgres",
        builders_contract,
    )
    .await;
}

/// The plan of `query` (EXPLAIN QUERY PLAN on SQLite; EXPLAIN on PostgreSQL with
/// sequential scans off, and bitmap scans off for a page, so a listed index
/// proves it is usable).
async fn plan(s: &Store, query: &ProjectionQuery<'_>, op: ProjectionOp) -> String {
    let (sql, mut args) = projection_statement(query, op, s.backend.dialect())
        .unwrap()
        .unwrap();
    if op == ProjectionOp::Page {
        let n = args.len();
        args[n - 2] = Arg::Int(50);
        args[n - 1] = Arg::Int(0);
    }
    let bind = |mut st: crate::backend::Sql| {
        for a in args.clone() {
            st = st.bind(a);
        }
        st
    };
    match s.backend.dialect() {
        Dialect::Sqlite => bind(q(leak(format!("EXPLAIN QUERY PLAN {sql}"))))
            .fetch_all(&s.backend)
            .await
            .unwrap()
            .iter()
            .map(|r| r.get_string("detail"))
            .collect::<Vec<_>>()
            .join("\n"),
        Dialect::Postgres => {
            let mut tx = s.backend.begin().await.unwrap();
            q("SET LOCAL enable_seqscan = off")
                .execute_tx(&mut tx)
                .await
                .unwrap();
            if op == ProjectionOp::Page {
                // A bitmap scan cannot return index order; a count may use one.
                q("SET LOCAL enable_bitmapscan = off")
                    .execute_tx(&mut tx)
                    .await
                    .unwrap();
            }
            let rows = bind(q(leak(format!("EXPLAIN {sql}"))))
                .fetch_all_tx(&mut tx)
                .await
                .unwrap();
            tx.rollback().await.unwrap();
            rows.iter()
                .map(|r| r.get_string_idx(0))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

async fn plans_contract(s: Store) {
    let w = world(&s).await;
    let (a, inbox) = (&w.account, &w.boxes[0]);
    // 6,000 rows over the account's three folders (2,000 in INBOX), 30 % unread,
    // 2 % flagged, 10 % labelled.
    let mut values = Vec::new();
    for n in 0..6000u32 {
        let folder = &w.boxes[n as usize % 3];
        let flags = match n % 50 {
            0 => r#"["Flagged"]"#,
            1..=14 => "[]",
            15..=19 => r#"["Seen",{"Keyword":"L"}]"#,
            _ => r#"["Seen"]"#,
        };
        values.push(format!(
            "('p{n}','{a}','{folder}',{},1,'2026-07-{:02}T10:00:00Z',{},'{flags}')",
            n + 1,
            1 + n % 28,
            n * 7 % 5000
        ));
    }
    for chunk in values.chunks(500) {
        exec(&s, &format!("INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,internaldate,size,flags_json) VALUES {}", chunk.join(","))).await;
    }
    exec(&s, "ANALYZE").await;
    let one = std::slice::from_ref(inbox);
    let l = ["L".to_owned()];
    let base = ProjectionQuery::account(a);
    let pg = s.backend.dialect() == Dialect::Postgres;
    let sorted = |p: &str| {
        if pg {
            p.contains("Sort")
        } else {
            p.contains("TEMP B-TREE")
        }
    };
    let cases: Vec<(&str, ProjectionQuery<'_>, ProjectionOp, &str, bool)> = vec![
        (
            "unread in folder",
            ProjectionQuery {
                mailboxes: MailboxScope::In(one),
                seen: Some(false),
                ..base.clone()
            },
            ProjectionOp::Page,
            "message_state_unread",
            true,
        ),
        (
            "unread in folder, ascending",
            ProjectionQuery {
                mailboxes: MailboxScope::In(one),
                seen: Some(false),
                sort: ProjectionSort::ReceivedAsc,
                ..base.clone()
            },
            ProjectionOp::Page,
            "message_state_unread",
            true,
        ),
        (
            "unread count in folder",
            ProjectionQuery {
                mailboxes: MailboxScope::In(one),
                seen: Some(false),
                ..base.clone()
            },
            ProjectionOp::Count,
            "message_state_unread",
            false,
        ),
        (
            "unread account-wide",
            ProjectionQuery {
                seen: Some(false),
                ..base.clone()
            },
            ProjectionOp::Page,
            "message_state_account_unread",
            true,
        ),
        (
            "flagged account-wide",
            ProjectionQuery {
                flagged: Some(true),
                ..base.clone()
            },
            ProjectionOp::Page,
            "message_state_flagged",
            true,
        ),
        (
            "keyword",
            ProjectionQuery {
                keywords: &l,
                ..base.clone()
            },
            ProjectionOp::Page,
            "message_keyword_by_kw",
            false,
        ),
        (
            "size sort in folder",
            ProjectionQuery {
                mailboxes: MailboxScope::In(one),
                sort: ProjectionSort::SizeDesc,
                ..base.clone()
            },
            ProjectionOp::Page,
            "message_state_size",
            true,
        ),
        (
            "folder by date (notKeyword)",
            ProjectionQuery {
                mailboxes: MailboxScope::In(one),
                not_keywords: &l,
                ..base.clone()
            },
            ProjectionOp::Page,
            "idx_messages_mailbox_page",
            true,
        ),
    ];
    for (name, query, op, index, ordered) in cases {
        let p = plan(&s, &query, op).await;
        assert!(
            p.contains(index),
            "{name}: expected {index} in the plan:\n{p}"
        );
        if ordered {
            assert!(
                !sorted(&p),
                "{name}: the index order must serve ORDER BY, no sort step:\n{p}"
            );
        }
    }
}

#[tokio::test]
async fn query_plans_use_the_projection_indexes_sqlite() {
    plans_contract(sqlite().await).await;
}
#[tokio::test]
async fn query_plans_use_the_projection_indexes_postgres() {
    postgres(
        "query_plans_use_the_projection_indexes_postgres",
        plans_contract,
    )
    .await;
}

// ---- measurements (ignored: -- --ignored --nocapture) -------------------------------

/// Backfill of 0125 over one 100,000-message account (30 % unread, 2 % flagged,
/// 10 % with a label), with the five indexes built after it, as in the migration.
async fn backfill_cost(s: Store) {
    let a = account(&s).await;
    let inbox = mailbox(&s, &a, "INBOX").await;
    const ROWS: usize = 100_000;
    let mut tx = s.backend.begin().await.unwrap();
    if s.backend.dialect() == Dialect::Postgres {
        // Seeding only: every row trigger off (superuser fixture).
        q("SET LOCAL session_replication_role = replica")
            .execute_tx(&mut tx)
            .await
            .unwrap();
    }
    for chunk in 0..(ROWS / 1000) {
        let mut sql = String::from(
            "INSERT INTO messages (stable_id, account_id, mailbox_id, uid, uidvalidity, internaldate, size, flags_json) VALUES ",
        );
        for i in 0..1000 {
            let n = chunk * 1000 + i;
            let flags = match n % 50 {
                0 => r#"["Flagged","Seen"]"#,
                1..=15 => "[]",
                16..=20 => r#"["Seen",{"Keyword":"Label"}]"#,
                _ => r#"["Seen"]"#,
            };
            if i > 0 {
                sql.push(',');
            }
            sql.push_str(&format!(
                "('cost{n}','{a}','{inbox}',{},1,'2026-07-{:02}T10:00:00Z',{},'{flags}')",
                n + 1,
                1 + n % 28,
                1000 + n % 9000
            ));
        }
        q(leak(sql)).execute_tx(&mut tx).await.unwrap();
    }
    tx.commit().await.unwrap();
    for index in [
        "message_keyword_by_kw",
        "message_state_unread",
        "message_state_account_unread",
        "message_state_flagged",
        "message_state_size",
    ] {
        exec(&s, &format!("DROP INDEX {index}")).await;
    }
    exec(&s, "DELETE FROM message_keyword").await;
    exec(&s, "DELETE FROM message_state").await;
    let started = std::time::Instant::now();
    run_section(&s, "-- backfill", "-- end backfill").await;
    let filled = started.elapsed();
    run_section(&s, "-- Partial indexes", "-- Every statement").await;
    let total = started.elapsed();
    assert_eq!(
        scalar(&s, "SELECT COUNT(*) FROM message_state").await,
        ROWS as i64
    );
    assert_eq!(
        scalar(&s, "SELECT COUNT(*) FROM message_state WHERE seen=0").await,
        (ROWS * 15 / 50) as i64
    );
    assert_clean(&s, "after the measured backfill").await;
    println!(
        "0125 backfill over {ROWS} messages ({:?}): rows {filled:?}, rows + 5 indexes {total:?}, keyword rows {}",
        s.backend.dialect(),
        scalar(&s, "SELECT COUNT(*) FROM message_keyword").await
    );
}

#[tokio::test]
#[ignore = "measurement: cargo test ... backfill_cost -- --ignored --nocapture"]
async fn migration_0125_backfill_cost_sqlite() {
    let dir = tempfile::tempdir().unwrap();
    let s = Store::open(
        dir.path().join("cost.db").to_str().unwrap(),
        ServerKey::generate(),
    )
    .await
    .unwrap();
    backfill_cost(s.clone()).await;
    s.close().await;
}
#[tokio::test]
#[ignore = "measurement on MW_TEST_PG: -- --ignored --nocapture"]
async fn migration_0125_backfill_cost_postgres() {
    postgres("migration_0125_backfill_cost_postgres", backfill_cost).await;
}

/// `Trigger <name>: time=<ms> calls=<n>` lines of an EXPLAIN ANALYZE.
fn trigger_times(plan: &str) -> BTreeMap<String, (f64, u64)> {
    let mut out = BTreeMap::new();
    for line in plan.lines() {
        let Some(rest) = line.trim().strip_prefix("Trigger ") else {
            continue;
        };
        let Some((name, tail)) = rest.split_once(": time=") else {
            continue;
        };
        let Some((time, calls)) = tail.split_once(" calls=") else {
            continue;
        };
        if let (Ok(t), Ok(c)) = (time.trim().parse::<f64>(), calls.trim().parse::<u64>()) {
            out.insert(name.trim().to_owned(), (t, c));
        }
    }
    out
}

async fn explain_analyze(s: &Store, sql: String) -> BTreeMap<String, (f64, u64)> {
    let rows = q(leak(format!("EXPLAIN (ANALYZE) {sql}")))
        .fetch_all(&s.backend)
        .await
        .unwrap();
    trigger_times(
        &rows
            .iter()
            .map(|r| r.get_string_idx(0))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn values(a: &str, mailbox: &str, prefix: &str, rows: usize) -> String {
    (0..rows)
        .map(|n| {
            let flags = if n % 3 == 0 {
                "[]"
            } else {
                r#"["Seen",{"Keyword":"L"}]"#
            };
            format!(
                "('{prefix}{n}','{a}','{mailbox}',{},1,'2026-07-01T10:00:00Z',100,'{flags}')",
                n + 1
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Design part 5 gate: our triggers' per-row time at 8,000 rows in one statement
/// is at most 1.2x the 2,000 figure (INSERT and a set_flags_batch-shaped UPDATE).
/// Medians of three runs; every trigger's figure is printed for comparison.
async fn per_row_cost(s: Store) {
    const OURS: [&str; 3] = [
        "message_projection_insert",
        "message_projection_update",
        "message_projection_delete",
    ];
    let mut per_row: BTreeMap<(&str, usize), Vec<f64>> = BTreeMap::new();
    let mut last = BTreeMap::new();
    for round in 0..3 {
        let a = account(&s).await;
        for rows in [2000usize, 8000] {
            let b = mailbox(&s, &a, &format!("Cost{rows}-{round}")).await;
            let prefix = format!("c{round}-{rows}-");
            let ins = explain_analyze(&s, format!("INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,internaldate,size,flags_json) VALUES {}", values(&a, &b, &prefix, rows))).await;
            let upd = explain_analyze(&s, format!("UPDATE messages SET flags_json = j.f FROM (SELECT stable_id AS id, CASE WHEN flags_json='[]' THEN '[\"Seen\"]' ELSE '[\"Flagged\"]' END AS f FROM messages WHERE mailbox_id='{b}') j WHERE messages.stable_id = j.id")).await;
            let del =
                explain_analyze(&s, format!("DELETE FROM messages WHERE mailbox_id='{b}'")).await;
            for (label, times) in [("insert", &ins), ("update", &upd), ("delete", &del)] {
                for name in OURS {
                    if let Some((t, calls)) = times.get(name) {
                        assert_eq!(*calls as usize, rows, "{name} fired once per row");
                        per_row
                            .entry((label, rows))
                            .or_default()
                            .push(t / *calls as f64);
                    }
                }
                last.insert((label, rows), times.clone());
            }
        }
    }
    let median = |v: &Vec<f64>| {
        let mut v = v.clone();
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    for ((label, rows), times) in &last {
        println!("{label} {rows} rows, every trigger (ms total, calls): {times:?}");
    }
    for label in ["insert", "update", "delete"] {
        let small = median(&per_row[&(label, 2000)]);
        let large = median(&per_row[&(label, 8000)]);
        println!(
            "0125 {label}: {:.4} ms/row at 2,000, {:.4} ms/row at 8,000, ratio {:.2}",
            small,
            large,
            large / small
        );
        if label != "delete" {
            assert!(
                large <= small * 1.2,
                "{label}: per-row cost grows with the statement ({small} -> {large} ms/row)"
            );
        }
    }
}

#[tokio::test]
#[ignore = "measurement + gate on MW_TEST_PG: -- --ignored --nocapture"]
async fn projection_triggers_per_row_cost_is_flat_postgres() {
    postgres(
        "projection_triggers_per_row_cost_is_flat_postgres",
        per_row_cost,
    )
    .await;
}

/// What the triggers add inside the single SQLite writer: an 8,000-row flag
/// UPDATE and 1,000 single-message flag writes, with and without them.
#[tokio::test]
#[ignore = "measurement: -- --ignored --nocapture"]
async fn projection_trigger_cost_sqlite() {
    let dir = tempfile::tempdir().unwrap();
    let s = Store::open(
        dir.path().join("t.db").to_str().unwrap(),
        ServerKey::generate(),
    )
    .await
    .unwrap();
    let a = account(&s).await;
    let b = mailbox(&s, &a, "INBOX").await;
    let v = values(&a, &b, "t", 8000);
    exec(&s, &format!("INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,internaldate,size,flags_json) VALUES {v}")).await;
    let ids: Vec<String> = (0..1000).map(|n| format!("t{n}")).collect();
    let measure = async |s: &Store| {
        let mut bulk = Vec::new();
        let mut single = Vec::new();
        for round in 0..3 {
            let f = if round % 2 == 0 { "[\"Seen\"]" } else { "[]" };
            let t = std::time::Instant::now();
            exec(
                s,
                &format!("UPDATE messages SET flags_json='{f}' WHERE mailbox_id='{b}'"),
            )
            .await;
            bulk.push(t.elapsed());
            let t = std::time::Instant::now();
            for id in &ids {
                s.set_flags_batch(&[(
                    id.clone(),
                    if round % 2 == 0 {
                        "[\"Flagged\"]".into()
                    } else {
                        "[\"Seen\"]".into()
                    },
                )])
                .await
                .unwrap();
            }
            single.push(t.elapsed());
        }
        bulk.sort();
        single.sort();
        (bulk[1], single[1])
    };
    let with = measure(&s).await;
    assert_clean(&s, "after the measured writes").await;
    for name in [
        "message_projection_insert",
        "message_projection_update",
        "message_projection_delete",
    ] {
        exec(&s, &format!("DROP TRIGGER {name}")).await;
    }
    let without = measure(&s).await;
    // And back, so the order of the two runs cannot explain the difference.
    let sql = migration(&s);
    let Backend::Sqlite(pool) = &s.backend else {
        unreachable!()
    };
    sqlx::raw_sql(
        &sql[sql
            .find("CREATE TRIGGER message_projection_insert")
            .unwrap()..],
    )
    .execute(pool)
    .await
    .unwrap();
    let again = measure(&s).await;
    assert_clean(&s, "after the triggers came back").await;
    println!(
        "SQLite 8,000-row flag UPDATE: {:?} with the 0125 triggers, {:?} without, {:?} with again",
        with.0, without.0, again.0
    );
    println!(
        "SQLite 1,000 single-message set_flags_batch: {:?} with, {:?} without, {:?} with again",
        with.1, without.1, again.1
    );
    s.close().await;
}
