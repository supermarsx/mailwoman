// Included in cache::tests to reuse the real account/mailbox/message contracts.
async fn flags_batch_rollback_contract(s: &Store) {
    let account = seed_account(s).await;
    let mailbox = seed_mailbox(s, &account, "INBOX", 100).await;
    let mut source = msg(&account, &mailbox, 1, 100, "<atomic@fixture>", "2026-07-01T10:00:00Z");
    source.flags_json = "[]";
    let id = s.upsert_message(&source).await.unwrap();
    s.update_mailbox_counts(&mailbox, 2, 0, 1, 1).await.unwrap();
    let before = s.get_message(&id).await.unwrap();
    let derived_before = q("SELECT data_revision FROM mail_data_rows WHERE stable_id=?1")
        .bind(&id).fetch_opt_scalar_string(s.backend()).await.unwrap();
    assert!(derived_before.is_some(), "real source publication trigger must be present");
    match s.backend.dialect() {
        Dialect::Sqlite => {
            q("CREATE TRIGGER flags_batch_counter_failure BEFORE UPDATE OF unread ON mailboxes WHEN NEW.unread<>OLD.unread BEGIN SELECT RAISE(ABORT,'synthetic counter failure'); END").execute(s.backend()).await.unwrap();
        }
        Dialect::Postgres => {
            q("CREATE FUNCTION flags_batch_counter_failure_fn() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.unread<>OLD.unread THEN RAISE EXCEPTION 'synthetic counter failure'; END IF; RETURN NEW; END $$").execute(s.backend()).await.unwrap();
            q("CREATE TRIGGER flags_batch_counter_failure BEFORE UPDATE OF unread ON mailboxes FOR EACH ROW EXECUTE FUNCTION flags_batch_counter_failure_fn()").execute(s.backend()).await.unwrap();
        }
    }
    assert!(s.set_flags_batch(&[(id.clone(), "[\"Seen\"]".into())]).await.is_err());
    assert!(s.set_flags(&id, "[\"Seen\"]").await.is_err(), "scalar writer must share the same atomic failure boundary");
    let after = s.get_message(&id).await.unwrap();
    assert_eq!(after.flags_json, before.flags_json, "counter failure must roll back the earlier message UPDATE");
    assert_eq!(after.cache_revision, before.cache_revision, "source-version triggers must roll back too");
    assert_eq!(s.get_mailbox(&mailbox).await.unwrap().unread, 1);
    assert_eq!(q("SELECT data_revision FROM mail_data_rows WHERE stable_id=?1")
        .bind(&id).fetch_opt_scalar_string(s.backend()).await.unwrap(), derived_before,
        "derived mail-data publication must roll back with the source");
    match s.backend.dialect() {
        Dialect::Sqlite => { q("DROP TRIGGER flags_batch_counter_failure").execute(s.backend()).await.unwrap(); }
        Dialect::Postgres => {
            q("DROP TRIGGER flags_batch_counter_failure ON mailboxes").execute(s.backend()).await.unwrap();
            q("DROP FUNCTION flags_batch_counter_failure_fn()").execute(s.backend()).await.unwrap();
        }
    }
    assert_eq!(s.set_flags_batch(&[(id.clone(), "[\"Seen\"]".into()), ("missing".into(), "[]".into())]).await.unwrap(), vec![true, false]);
    assert_eq!(s.get_message(&id).await.unwrap().flags_json, "[\"Seen\"]");
    assert_eq!(s.get_mailbox(&mailbox).await.unwrap().unread, 0);
}

async fn flags_batch_concurrent_contract(s: &Store) {
    let account = seed_account(s).await;
    let mailbox = seed_mailbox(s, &account, "INBOX", 100).await;
    let mut source = msg(&account, &mailbox, 1, 100, "<concurrent@fixture>", "2026-07-01T10:00:00Z");
    source.flags_json = "[]";
    let id = s.upsert_message(&source).await.unwrap();
    s.update_mailbox_counts(&mailbox, 2, 0, 1, 1).await.unwrap();
    // Two readers setting the same unread message must not both subtract one.
    // Start from a nonzero residual count so a floor-at-zero cannot hide a double subtraction.
    s.update_mailbox_counts(&mailbox, 2, 0, 3, 3).await.unwrap();
    let a = vec![(id.clone(), "[\"Seen\"]".into())];
    let (a, b) = tokio::join!(s.set_flags_batch(&a), s.set_flags(&id, "[\"Seen\"]"));
    assert_eq!(a.unwrap(), vec![true]);
    b.unwrap();
    assert!(matches!(s.set_flags("missing", "[]").await, Err(StoreError::NotFound)));
    assert_eq!(s.get_mailbox(&mailbox).await.unwrap().unread, 2);
    assert_eq!(s.get_message(&id).await.unwrap().flags_json, "[\"Seen\"]");
}

async fn flags_batch_private_pg(which: u8) {
    use sqlx::Connection;
    let dsn = std::env::var("MW_TEST_PG").expect("this PostgreSQL contract requires the owned MW_TEST_PG fixture");
    let schema = format!("flags_batch_atomic_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&mut admin).await.unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped.query_pairs_mut().append_pair("options", &format!("-csearch_path={schema}"));
    let s = Store::open(scoped.as_str(), ServerKey::generate()).await.unwrap();
    match which {
        0 => flags_batch_rollback_contract(&s).await,
        1 => assert_batch_flag_write(&s).await,
        2 => flags_batch_concurrent_contract(&s).await,
        _ => unreachable!(),
    }
    s.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE")).execute(&mut admin).await.unwrap();
}

#[tokio::test]
async fn flags_batch_atomic_counter_failure_sqlite() { flags_batch_rollback_contract(&store().await).await; }
#[tokio::test]
async fn flags_batch_atomic_semantics_sqlite() { assert_batch_flag_write(&store().await).await; }
#[tokio::test]
async fn flags_batch_atomic_concurrent_sqlite() { flags_batch_concurrent_contract(&store().await).await; }
#[tokio::test]
#[ignore = "requires owned MW_TEST_PG fixture; focused gate must explicitly include ignored tests"]
async fn flags_batch_atomic_counter_failure_postgres() { flags_batch_private_pg(0).await; }
#[tokio::test]
#[ignore = "requires owned MW_TEST_PG fixture; focused gate must explicitly include ignored tests"]
async fn flags_batch_atomic_semantics_postgres() { flags_batch_private_pg(1).await; }
#[tokio::test]
#[ignore = "requires owned MW_TEST_PG fixture; focused gate must explicitly include ignored tests"]
async fn flags_batch_atomic_concurrent_postgres() { flags_batch_private_pg(2).await; }
