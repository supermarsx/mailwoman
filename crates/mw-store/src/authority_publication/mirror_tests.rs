use super::*;
use crate::{
    AccountKind, Credentials, DomainConfigRow, NewAccount, ServerKey,
    identity_principal::VerifiedPrimary,
};

const BOUND: Duration = Duration::from_millis(100);
const DOMAINS: &[&str] = &["mirror.test"];

async fn seed(s: &Store) -> (String, String) {
    let user = "person@mirror.test";
    let creds = Credentials {
        username: user.into(),
        password: "fixture-secret-never-log".into(),
    };
    let account = s
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "mail.mirror.test",
                port: 993,
                tls: "implicit",
                username: user,
                sync_policy_json: "{}",
            },
            &creds,
        )
        .await
        .unwrap();
    s.put_domain_config(&DomainConfigRow {
        name: "mirror.test".into(),
        upstream_json: "{}".into(),
        allowlist_json: "[]".into(),
        blocklist_json: "[]".into(),
        enabled: true,
        preferred: false,
        settings_json: "{}".into(),
        updated_at: 1,
    })
    .await
    .unwrap();
    let session = s
        .create_session(&account, user, "engine", "engine", &creds)
        .await
        .unwrap();
    s.set_session_authentication(&session, "password")
        .await
        .unwrap();
    s.bind_verified_primary(
        &session,
        VerifiedPrimary {
            account_id: &account,
            kind: "mailbox",
            authority: "fixture",
            subject: user,
        },
        1,
    )
    .await
    .unwrap()
    .unwrap();
    (account, session)
}

fn domains() -> Vec<String> {
    DOMAINS.iter().map(|d| (*d).to_owned()).collect()
}

async fn fence(s: &Store, session: &str) -> NativeAuthorityFence {
    s.native_publication_fence(session, &domains(), &[])
        .await
        .unwrap()
        .expect("seeded session has a publication fence")
}

/// Stamp, then read: the order a request builder must follow.
async fn stamped(
    s: &Store,
    mirror: &EpochMirror,
    session: &str,
) -> (NativeAuthorityFence, MirrorGen) {
    let built = mirror.stamp_gen().expect("fresh mirror stamps");
    (fence(s, session).await, built)
}

async fn settle(mirror: &EpochMirror) {
    for _ in 0..8 {
        if !mirror.poll_now().await.unwrap() && mirror.stamp_gen().is_some() {
            return;
        }
    }
    panic!("mirror did not reach a fresh, unflushed state");
}

async fn exec(s: &Store, sql: &'static str) {
    q(sql).execute(&s.backend).await.unwrap();
}

#[tokio::test]
async fn sqlite_log_rows_carry_incarnations_not_scope_identifiers() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (account, session) = seed(&s).await;
    let rows = q("SELECT scope_kind,incarnation FROM authority_change_log ORDER BY seq")
        .fetch_all(&s.backend)
        .await
        .unwrap();
    assert!(!rows.is_empty(), "seeding writes authority sources");
    for row in rows {
        let kind = text(&row, "scope_kind").unwrap();
        let incarnation = text(&row, "incarnation").unwrap();
        if kind == SETTINGS_REVISION {
            assert_eq!(incarnation, "");
        } else {
            assert!(AuthorityKind::parse(&kind).is_some(), "unknown kind {kind}");
            assert!(
                incarnation.len() == 32 && incarnation.bytes().all(|b| b.is_ascii_hexdigit()),
                "log row must hold an epoch incarnation"
            );
            assert_ne!(incarnation, account);
            assert_ne!(incarnation, session);
        }
    }
    s.close().await;
}

#[tokio::test]
async fn sqlite_required_schema_includes_change_log_triggers() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (_, session) = seed(&s).await;
    let current = fence(&s, &session).await;
    assert!(
        s.validate_native_publication_schema(&current)
            .await
            .unwrap()
    );
    exec(&s, "DROP TRIGGER authority_change_log_epoch_update").await;
    let changed = fence(&s, &session).await;
    assert_ne!(changed.publication_schema(), current.publication_schema());
    assert!(
        !s.validate_native_publication_schema(&changed)
            .await
            .unwrap(),
        "a missing change-log trigger must disqualify published inputs"
    );
    s.close().await;
}

#[tokio::test]
async fn sqlite_source_writes_through_store_and_raw_sql_invalidate() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    source_writes_invalidate(&s, BOUND).await;
    s.close().await;
}

async fn source_writes_invalidate(s: &Store, bound: Duration) {
    let s = s.clone();
    let (account, session) = seed(&s).await;
    let mirror = EpochMirror::manual(&s, bound);
    assert!(mirror.stamp_gen().is_none(), "never polled: not fresh");
    settle(&mirror).await;

    // A write that no authority trigger observes keeps the fence valid.
    let (current, built) = stamped(&s, &mirror, &session).await;
    s.set_setting("t34-unrelated-key", "1").await.unwrap();
    mirror.poll_now().await.unwrap();
    assert!(
        mirror.still_valid(&current, built),
        "unrelated write must not invalidate"
    );

    // Raw SQL that bypasses the Store API, on a session column the trigger covers.
    q("UPDATE sessions SET user_agent='raw-sql-agent' WHERE id=?1")
        .bind(session.as_str())
        .execute(&s.backend)
        .await
        .unwrap();
    assert!(mirror.still_valid(&current, built), "not yet polled");
    mirror.poll_now().await.unwrap();
    assert!(
        !mirror.still_valid(&current, built),
        "raw session write invalidates"
    );

    // Account update through the Store API.
    let (current, built) = stamped(&s, &mirror, &session).await;
    assert!(mirror.still_valid(&current, built));
    s.update_sync_policy(&account, r#"{"t34":true}"#)
        .await
        .unwrap();
    mirror.poll_now().await.unwrap();
    assert!(
        !mirror.still_valid(&current, built),
        "account update invalidates"
    );

    // Raw SQL on the identity chain the fence joins.
    let (current, built) = stamped(&s, &mirror, &session).await;
    q("UPDATE identity_principal SET state='disabled' WHERE id=?1")
        .bind(current.principal_id())
        .execute(&s.backend)
        .await
        .unwrap();
    mirror.poll_now().await.unwrap();
    assert!(
        !mirror.still_valid(&current, built),
        "principal state invalidates"
    );
    q("UPDATE identity_principal SET state='active' WHERE id=?1")
        .bind(current.principal_id())
        .execute(&s.backend)
        .await
        .unwrap();
    mirror.poll_now().await.unwrap();

    // Settings revision (primary_policy_revision in the fence).
    let (current, built) = stamped(&s, &mirror, &session).await;
    assert!(mirror.globals_unchanged(built));
    s.put_setting_policy(&crate::settings_policy::SettingPolicyRow {
        key: "t34-mirror-policy".into(),
        global_value: Some("true".into()),
        allow_user_override: false,
        global_locked: false,
        updated_by: "test".into(),
        updated_at: 1,
    })
    .await
    .unwrap();
    mirror.poll_now().await.unwrap();
    assert!(
        !mirror.still_valid(&current, built),
        "policy write invalidates"
    );
    assert!(!mirror.globals_unchanged(built));
    exec(&s, "UPDATE settings_revision SET n=n+1").await;
    let (current, built) = stamped(&s, &mirror, &session).await;
    exec(&s, "UPDATE settings_revision SET n=n+1").await;
    mirror.poll_now().await.unwrap();
    assert!(
        !mirror.still_valid(&current, built),
        "raw settings revision invalidates"
    );

    // Session delete through the Store API.
    let (current, built) = stamped(&s, &mirror, &session).await;
    s.delete_session(&session).await.unwrap();
    mirror.poll_now().await.unwrap();
    assert!(
        !mirror.still_valid(&current, built),
        "session delete invalidates"
    );
    let counters = mirror.counters();
    assert!(counters.hits >= 3 && counters.misses >= 6, "{counters:?}");
    assert_eq!(counters.failures, 0);
}

#[tokio::test]
async fn sqlite_change_committed_after_stamp_is_never_accepted_after_a_poll() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    change_after_stamp_is_refused(&s, BOUND).await;
    s.close().await;
}

async fn change_after_stamp_is_refused(s: &Store, bound: Duration) {
    let s = s.clone();
    let (account, session) = seed(&s).await;
    let mirror = EpochMirror::manual(&s, bound);
    settle(&mirror).await;

    // Commit between stamp_gen and the fence read: the fence reflects it, and the
    // conservative mirror still refuses the pair once the change is polled.
    let built = mirror.stamp_gen().unwrap();
    let before = fence(&s, &session).await;
    s.update_sync_policy(&account, r#"{"between":1}"#)
        .await
        .unwrap();
    let after_change = fence(&s, &session).await;
    let account_scope = AuthorityScope::new(AuthorityKind::Account, &account).unwrap();
    let version =
        |f: &NativeAuthorityFence| f.versions().stamps.get(&account_scope).unwrap().version;
    assert!(version(&after_change) > version(&before));
    mirror.poll_now().await.unwrap();
    assert!(!mirror.still_valid(&after_change, built));

    // Commit after the fence read: the dangerous case.
    let (stale, built) = stamped(&s, &mirror, &session).await;
    s.update_sync_policy(&account, r#"{"after":1}"#)
        .await
        .unwrap();
    mirror.poll_now().await.unwrap();
    assert!(!mirror.still_valid(&stale, built));
    // A fence read after that poll validates again.
    let (current, built) = stamped(&s, &mirror, &session).await;
    assert!(mirror.still_valid(&current, built));
    // A stamp never validates a fence for another session's scopes either way,
    // and a stamp from another mirror instance never validates.
    let other = EpochMirror::manual(&s, bound);
    settle(&other).await;
    assert!(!other.still_valid(&current, built));
}

#[tokio::test]
async fn sqlite_background_mirror_refuses_a_change_by_the_bound() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    background_refuses_by_the_bound(&s, BOUND).await;
    s.close().await;
}

async fn background_refuses_by_the_bound(s: &Store, bound: Duration) {
    let s = s.clone();
    let (account, session) = seed(&s).await;
    let mirror = EpochMirror::for_store(&s, bound);
    assert!(mirror.is_live());
    assert!(Arc::ptr_eq(&mirror, &EpochMirror::for_store(&s, bound)));
    let deadline = Instant::now() + Duration::from_secs(10);
    let ready = || async {
        loop {
            if let Some(built) = mirror.stamp_gen() {
                let current = fence(&s, &session).await;
                if mirror.still_valid(&current, built) {
                    return (current, built);
                }
            }
            assert!(
                Instant::now() < deadline,
                "background mirror never became fresh"
            );
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    };
    let (current, built) = ready().await;
    s.update_sync_policy(&account, r#"{"bound":1}"#)
        .await
        .unwrap();
    let committed = Instant::now();
    tokio::time::sleep_until((committed + bound).into()).await;
    // Deterministic: either a poll that started after the commit applied it, or
    // no complete poll started after it and the mirror is not fresh.
    assert!(!mirror.still_valid(&current, built));
    let (_current, _built) = ready().await;
    assert!(mirror.counters().polls >= 2);
}

#[tokio::test]
async fn sqlite_expedite_latches_until_a_later_poll_completes() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    expedite_latches(&s, BOUND).await;
    s.close().await;
}

async fn expedite_latches(s: &Store, bound: Duration) {
    let s = s.clone();
    let (_, session) = seed(&s).await;
    let mirror = EpochMirror::manual(&s, bound);
    settle(&mirror).await;
    let (current, built) = stamped(&s, &mirror, &session).await;
    assert!(mirror.still_valid(&current, built));
    mirror.expedite();
    assert!(!mirror.still_valid(&current, built));
    assert!(!mirror.globals_unchanged(built));
    mirror.poll_now().await.unwrap();
    assert!(
        mirror.still_valid(&current, built),
        "no change: valid again after the poll"
    );
    assert_eq!(mirror.counters().expedites, 1);
}

#[tokio::test]
async fn sqlite_poll_failure_goes_stale_after_the_bound() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (_, session) = seed(&s).await;
    let mirror = EpochMirror::manual(&s, BOUND);
    settle(&mirror).await;
    let (current, built) = stamped(&s, &mirror, &session).await;
    exec(
        &s,
        "ALTER TABLE authority_change_log RENAME TO authority_change_log_away",
    )
    .await;
    assert!(mirror.poll_now().await.is_err());
    assert_eq!(mirror.counters().failures, 1);
    tokio::time::sleep(BOUND + Duration::from_millis(20)).await;
    assert!(mirror.poll_now().await.is_err());
    assert!(mirror.stamp_gen().is_none(), "failed polls: not fresh");
    assert!(!mirror.is_fresh());
    assert!(!mirror.still_valid(&current, built));
    exec(
        &s,
        "ALTER TABLE authority_change_log_away RENAME TO authority_change_log",
    )
    .await;
    settle(&mirror).await;
    assert!(
        !mirror.still_valid(&current, built),
        "schema changed: old fence stays cold"
    );
    let (current, built) = stamped(&s, &mirror, &session).await;
    assert!(mirror.still_valid(&current, built));
    s.close().await;
}

#[tokio::test]
async fn sqlite_schema_stamp_change_flushes() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (_, session) = seed(&s).await;
    let mirror = EpochMirror::manual(&s, BOUND);
    settle(&mirror).await;
    let (current, built) = stamped(&s, &mirror, &session).await;
    exec(&s, "CREATE TABLE t34_mirror_probe(x INTEGER)").await;
    mirror.poll_now().await.unwrap();
    assert_eq!(mirror.counters().schema_flushes, 1);
    assert!(!mirror.still_valid(&current, built));
    assert!(
        mirror.stamp_gen().is_none(),
        "flushed generation does not stamp"
    );
    settle(&mirror).await;
    let (current, built) = stamped(&s, &mirror, &session).await;
    assert!(mirror.still_valid(&current, built));
    s.close().await;
}

#[tokio::test]
async fn sqlite_overflow_and_pruned_horizon_flush() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (_, session) = seed(&s).await;
    let mirror = EpochMirror::manual(&s, BOUND);
    settle(&mirror).await;
    let (current, built) = stamped(&s, &mirror, &session).await;
    exec(&s, "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i+1 FROM n WHERE i<5000) INSERT INTO authority_change_log(scope_kind,incarnation,at) SELECT 'probe','probe',0 FROM n").await;
    assert!(
        mirror.poll_now().await.unwrap(),
        "full page flushes and jumps to the head"
    );
    assert_eq!(mirror.counters().overflow_flushes, 1);
    assert!(!mirror.still_valid(&current, built));
    assert!(mirror.stamp_gen().is_none());
    assert!(
        !mirror.poll_now().await.unwrap(),
        "no rows left after the jump"
    );
    assert_eq!(mirror.counters().overflow_flushes, 1);
    settle(&mirror).await;
    let (current, built) = stamped(&s, &mirror, &session).await;
    assert!(mirror.still_valid(&current, built));

    // More than the retained horizon between two polls: unread rows were pruned.
    exec(&s, "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i+1 FROM n WHERE i<70000) INSERT INTO authority_change_log(scope_kind,incarnation,at) SELECT 'probe','probe',0 FROM n").await;
    let retained = q("SELECT COUNT(*) AS n,MAX(seq)-MIN(seq)+1 AS span FROM authority_change_log")
        .fetch_one(&s.backend)
        .await
        .unwrap();
    assert!(number(&retained, "n").unwrap().unwrap() <= RETAINED_SEQ);
    assert!(number(&retained, "span").unwrap().unwrap() <= RETAINED_SEQ);
    mirror.poll_now().await.unwrap();
    assert!(mirror.counters().horizon_flushes >= 1);
    assert!(!mirror.still_valid(&current, built));
    settle(&mirror).await;

    // Rows deleted inside the horizon (outside the contract) also flush.
    let (current, built) = stamped(&s, &mirror, &session).await;
    let flushes = mirror.counters().horizon_flushes;
    exec(&s, "DELETE FROM authority_change_log").await;
    mirror.poll_now().await.unwrap();
    assert!(mirror.counters().horizon_flushes > flushes);
    assert!(!mirror.still_valid(&current, built));
    s.close().await;
}

#[tokio::test]
async fn sqlite_tracked_scope_capacity_flushes() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (_, session) = seed(&s).await;
    let mirror = EpochMirror::manual(&s, BOUND);
    settle(&mirror).await;
    let (current, built) = stamped(&s, &mirror, &session).await;
    // Distinct scopes, applied page by page without a horizon gap.
    for _ in 0..17 {
        exec(&s, "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i+1 FROM n WHERE i<4000) INSERT INTO authority_change_log(scope_kind,incarnation,at) SELECT 'probe',lower(hex(randomblob(16))),0 FROM n").await;
        while mirror.poll_now().await.unwrap() {}
    }
    assert!(mirror.counters().capacity_flushes >= 1);
    assert!(!mirror.still_valid(&current, built));
    settle(&mirror).await;
    let (current, built) = stamped(&s, &mirror, &session).await;
    assert!(mirror.still_valid(&current, built));
    s.close().await;
}

#[tokio::test]
async fn sqlite_second_process_write_and_generation_swap_invalidate() {
    use crate::encryption::{DbKey, EncryptionConfig, EncryptionMode};
    let dir = tempfile::tempdir().unwrap();
    let config = EncryptionConfig::with_key(EncryptionMode::Required, DbKey::generate());
    let key = ServerKey::generate();
    let path = dir.path().join("mirror.db");
    let reader = Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &config)
        .await
        .unwrap();
    let writer = Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &config)
        .await
        .unwrap();
    let (account, session) = seed(&writer).await;
    let mirror = EpochMirror::manual(&reader, BOUND);
    settle(&mirror).await;
    let (current, built) = stamped(&reader, &mirror, &session).await;
    assert!(mirror.still_valid(&current, built));
    writer
        .update_sync_policy(&account, r#"{"other":1}"#)
        .await
        .unwrap();
    mirror.poll_now().await.unwrap();
    assert!(
        !mirror.still_valid(&current, built),
        "other Store handle's write invalidates"
    );

    // Live generation swap to a different file.
    let (current, built) = stamped(&reader, &mirror, &session).await;
    let replacement = Store::open_with_encryption(
        dir.path().join("replacement.db").to_str().unwrap(),
        key,
        &config,
    )
    .await
    .unwrap();
    let Backend::Sqlite(other) = &replacement.backend else {
        unreachable!()
    };
    let pool = other.pin().await.pool().clone();
    let Backend::Sqlite(live) = &reader.backend else {
        unreachable!()
    };
    let mut exclusive = live.exclusive().await;
    exclusive.close_old().await;
    exclusive.replace_with_page_cache(pool, None);
    drop(exclusive);
    mirror.poll_now().await.unwrap();
    assert_eq!(mirror.counters().generation_flushes, 1);
    assert!(!mirror.still_valid(&current, built));
    reader.close().await;
    writer.close().await;
    replacement.close().await;
}

#[tokio::test]
async fn invalid_bound_is_inert() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    for bound in [Duration::from_millis(99), Duration::from_millis(5001)] {
        let mirror = EpochMirror::for_store(&s, bound);
        assert!(!mirror.is_live());
        assert!(!mirror.poll_now().await.unwrap());
        assert!(mirror.stamp_gen().is_none());
    }
    assert!(valid_staleness(DEFAULT_STALENESS));
    s.close().await;
}

// ---------------------------------------------------------------------------
// PostgreSQL (t49pg). Each test opens its own schema on MW_TEST_PG.
// ---------------------------------------------------------------------------

/// Wide enough that the gap rule does not flush between the steps of a test on
/// a slow host; tests that need the bound to pass set their own.
const PG_BOUND: Duration = Duration::from_secs(3);

struct Pg {
    store: Store,
    dsn: String,
    admin: sqlx::PgConnection,
    schema: String,
}

async fn pg(test: &str) -> Option<Pg> {
    use sqlx::Connection;
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        mw_test_gate::skip(format!("MW_TEST_PG absent; {test} not executed"));
        return None;
    };
    let schema = format!("mirror_{}", uuid::Uuid::new_v4().simple());
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
    Some(Pg {
        store,
        dsn: scoped.to_string(),
        admin,
        schema,
    })
}

impl Pg {
    async fn conn(&self) -> sqlx::PgConnection {
        use sqlx::Connection;
        sqlx::PgConnection::connect(&self.dsn).await.unwrap()
    }
    async fn exec(&self, sql: &str) {
        let mut conn = self.conn().await;
        sqlx::query(sql).execute(&mut conn).await.unwrap();
    }
    async fn stamp(&self) -> String {
        q(super::schema::stamp_sql(Dialect::Postgres))
            .fetch_one(&self.store.backend)
            .await
            .unwrap()
            .try_get_opt_string("publication_schema")
            .unwrap()
            .expect("complete schema stamp")
    }
    async fn log_rows(&self) -> i64 {
        number(
            &q("SELECT COUNT(*) AS n FROM authority_change_log")
                .fetch_one(&self.store.backend)
                .await
                .unwrap(),
            "n",
        )
        .unwrap()
        .unwrap()
    }
    async fn done(mut self) {
        self.store.close().await;
        sqlx::query(&format!("DROP SCHEMA {} CASCADE", self.schema))
            .execute(&mut self.admin)
            .await
            .unwrap();
    }
}

const BUMP: &str = "INSERT INTO authority_epoch(scope_kind,scope_id) VALUES($1,$2) ON CONFLICT(scope_kind,scope_id) DO UPDATE SET version=authority_epoch.version+1";

#[tokio::test]
async fn pg_poll_snapshot_is_the_transaction_snapshot() {
    let Some(pg) = pg("pg_poll_snapshot_is_the_transaction_snapshot").await else {
        return;
    };
    // The poll's guarantee: inside REPEATABLE READ, pg_current_snapshot() is
    // the snapshot every statement of the transaction reads.
    let mut reader = pg.conn().await;
    sqlx::query("BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut reader)
        .await
        .unwrap();
    let probe = "SELECT CAST(pg_catalog.pg_current_snapshot() AS TEXT),(SELECT COUNT(*) FROM authority_change_log),(SELECT bool_and(pg_catalog.pg_visible_in_snapshot(xid,pg_catalog.pg_current_snapshot())) FROM authority_change_log)";
    let first: (String, i64, Option<bool>) =
        sqlx::query_as(probe).fetch_one(&mut reader).await.unwrap();
    pg.exec("UPDATE settings_revision SET n=n+1").await;
    let second: (String, i64, Option<bool>) =
        sqlx::query_as(probe).fetch_one(&mut reader).await.unwrap();
    sqlx::query("COMMIT").execute(&mut reader).await.unwrap();
    assert_eq!(first.0, second.0, "one snapshot for the whole transaction");
    assert_eq!(first.1, second.1, "the commit is not visible to the scan");
    assert_ne!(first.2, Some(false));
    let (visible, now): (bool, bool) = sqlx::query_as(
        "SELECT pg_catalog.pg_visible_in_snapshot(xid,CAST(CAST($1 AS TEXT) AS pg_catalog.pg_snapshot)),pg_catalog.pg_visible_in_snapshot(xid,pg_catalog.pg_current_snapshot()) FROM authority_change_log ORDER BY seq DESC LIMIT 1",
    )
    .bind(&first.0)
    .fetch_one(&mut reader)
    .await
    .unwrap();
    assert!(
        !visible && now,
        "the later row is outside the poll snapshot"
    );
    pg.done().await;
}

#[tokio::test]
async fn pg_change_log_rows_and_required_schema() {
    let Some(pg) = pg("pg_change_log_rows_and_required_schema").await else {
        return;
    };
    let (account, session) = seed(&pg.store).await;
    let mirror = EpochMirror::for_store(&pg.store, DEFAULT_STALENESS);
    assert!(mirror.is_live(), "PostgreSQL mirrors are live (t49pg)");
    let current = fence(&pg.store, &session).await;
    assert!(
        pg.store
            .validate_native_publication_schema(&current)
            .await
            .unwrap(),
        "required definitions: 0078, 0111, 0112, 0120"
    );
    let before = pg.log_rows().await;
    assert!(before > 0, "seeding writes authority sources");
    pg.store
        .update_sync_policy(&account, r#"{"pg":1}"#)
        .await
        .unwrap();
    exec(&pg.store, "UPDATE settings_revision SET n=n+1").await;
    assert!(pg.log_rows().await >= before + 2);
    let rows =
        q("SELECT scope_kind,incarnation,CAST(xid AS TEXT) AS xid FROM authority_change_log")
            .fetch_all(&pg.store.backend)
            .await
            .unwrap();
    for row in rows {
        let kind = text(&row, "scope_kind").unwrap();
        let incarnation = text(&row, "incarnation").unwrap();
        assert!(text(&row, "xid").unwrap().parse::<u64>().unwrap() > 0);
        if kind == SETTINGS_REVISION {
            assert_eq!(incarnation, "");
        } else {
            assert!(AuthorityKind::parse(&kind).is_some(), "unknown kind {kind}");
            assert!(incarnation.len() == 32 && incarnation.bytes().all(|b| b.is_ascii_hexdigit()));
            assert_ne!(incarnation, account);
            assert_ne!(incarnation, session);
        }
    }
    // Append-only: two change-log triggers, no prune trigger.
    let triggers = q("SELECT COUNT(*) AS n FROM pg_catalog.pg_trigger WHERE tgname LIKE 'authority_change_log%' AND tgrelid IN (SELECT c.oid FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=current_schema())")
        .fetch_one(&pg.store.backend)
        .await
        .unwrap();
    assert_eq!(number(&triggers, "n").unwrap().unwrap(), 2);
    pg.done().await;
}

#[tokio::test]
async fn pg_source_writes_through_store_and_raw_sql_invalidate() {
    let Some(pg) = pg("pg_source_writes_through_store_and_raw_sql_invalidate").await else {
        return;
    };
    source_writes_invalidate(&pg.store, PG_BOUND).await;
    pg.done().await;
}

#[tokio::test]
async fn pg_change_committed_after_stamp_is_never_accepted_after_a_poll() {
    let Some(pg) = pg("pg_change_committed_after_stamp_is_never_accepted_after_a_poll").await
    else {
        return;
    };
    change_after_stamp_is_refused(&pg.store, PG_BOUND).await;
    pg.done().await;
}

#[tokio::test]
async fn pg_expedite_latches_until_a_later_poll_completes() {
    let Some(pg) = pg("pg_expedite_latches_until_a_later_poll_completes").await else {
        return;
    };
    expedite_latches(&pg.store, PG_BOUND).await;
    pg.done().await;
}

#[tokio::test]
async fn pg_background_mirror_refuses_a_change_by_the_bound() {
    let Some(pg) = pg("pg_background_mirror_refuses_a_change_by_the_bound").await else {
        return;
    };
    background_refuses_by_the_bound(&pg.store, Duration::from_millis(500)).await;
    pg.done().await;
}

#[tokio::test]
async fn pg_out_of_order_commit_is_applied() {
    let Some(pg) = pg("pg_out_of_order_commit_is_applied").await else {
        return;
    };
    let (_, session) = seed(&pg.store).await;
    let mirror = EpochMirror::manual(&pg.store, PG_BOUND);
    settle(&mirror).await;
    // A inserts its log row first (lower seq) and commits after B.
    let mut a = pg.conn().await;
    sqlx::query("BEGIN").execute(&mut a).await.unwrap();
    sqlx::query(BUMP)
        .bind("session")
        .bind(session.as_str())
        .execute(&mut a)
        .await
        .unwrap();
    let (seq_a,): (i64,) = sqlx::query_as("SELECT MAX(seq) FROM authority_change_log")
        .fetch_one(&mut a)
        .await
        .unwrap();
    let mut b = pg.conn().await;
    sqlx::query("BEGIN").execute(&mut b).await.unwrap();
    sqlx::query("UPDATE settings_revision SET n=n+1")
        .execute(&mut b)
        .await
        .unwrap();
    let (seq_b,): (i64,) = sqlx::query_as("SELECT MAX(seq) FROM authority_change_log")
        .fetch_one(&mut b)
        .await
        .unwrap();
    sqlx::query("COMMIT").execute(&mut b).await.unwrap();
    assert!(seq_a < seq_b, "A's row has the lower seq");
    mirror.poll_now().await.unwrap();
    // The fence reflects B but not A.
    let (current, built) = stamped(&pg.store, &mirror, &session).await;
    assert!(mirror.still_valid(&current, built));
    mirror.poll_now().await.unwrap();
    assert!(mirror.still_valid(&current, built), "A is still open");
    sqlx::query("COMMIT").execute(&mut a).await.unwrap();
    mirror.poll_now().await.unwrap();
    assert!(
        !mirror.still_valid(&current, built),
        "a row with a lower seq that commits later is applied"
    );
    let (current, built) = stamped(&pg.store, &mirror, &session).await;
    assert!(mirror.still_valid(&current, built));
    pg.done().await;
}

#[tokio::test]
async fn pg_long_open_transaction_is_applied_at_commit_without_repeated_flushes() {
    let Some(pg) =
        pg("pg_long_open_transaction_is_applied_at_commit_without_repeated_flushes").await
    else {
        return;
    };
    let (account, session) = seed(&pg.store).await;
    let mirror = EpochMirror::manual(&pg.store, PG_BOUND);
    settle(&mirror).await;
    let mut long = pg.conn().await;
    sqlx::query("BEGIN").execute(&mut long).await.unwrap();
    sqlx::query(BUMP)
        .bind("session")
        .bind(session.as_str())
        .execute(&mut long)
        .await
        .unwrap();
    // Other writers commit while the long transaction pins the snapshot xmin.
    for i in 0..5 {
        pg.store
            .update_sync_policy(&account, &format!(r#"{{"long":{i}}}"#))
            .await
            .unwrap();
        mirror.poll_now().await.unwrap();
    }
    let (current, built) = stamped(&pg.store, &mirror, &session).await;
    for _ in 0..3 {
        assert!(!mirror.poll_now().await.unwrap());
        assert!(
            mirror.still_valid(&current, built),
            "rows already applied are not applied again"
        );
    }
    // More than a page committed at once while the xmin is pinned: one flush,
    // then the next polls page past it.
    exec(&pg.store, "INSERT INTO authority_change_log(scope_kind,incarnation,at) SELECT 'probe','probe',0 FROM generate_series(1,5000)").await;
    assert!(mirror.poll_now().await.unwrap());
    assert_eq!(mirror.counters().overflow_flushes, 1);
    assert!(!mirror.still_valid(&current, built));
    settle(&mirror).await;
    let (current, built) = stamped(&pg.store, &mirror, &session).await;
    assert!(!mirror.poll_now().await.unwrap());
    assert!(mirror.still_valid(&current, built));
    assert_eq!(mirror.counters().overflow_flushes, 1);
    sqlx::query("COMMIT").execute(&mut long).await.unwrap();
    mirror.poll_now().await.unwrap();
    assert!(
        !mirror.still_valid(&current, built),
        "applied when it commits"
    );
    assert_eq!(mirror.counters().failures, 0);
    pg.done().await;
}

#[tokio::test]
async fn pg_truncate_of_each_stamped_table_flushes() {
    let Some(pg) = pg("pg_truncate_of_each_stamped_table_flushes").await else {
        return;
    };
    seed(&pg.store).await;
    let mirror = EpochMirror::manual(&pg.store, PG_BOUND);
    let tables = super::schema::stamped_tables();
    assert!(tables.len() >= 28);
    for table in tables {
        settle(&mirror).await;
        let built = mirror.stamp_gen().unwrap();
        assert!(mirror.globals_unchanged(built));
        let flushes = mirror.counters().schema_flushes;
        // No row trigger fires; the stamp's relfilenode changes.
        pg.exec(&format!("TRUNCATE {table} CASCADE")).await;
        assert!(mirror.poll_now().await.unwrap(), "{table}");
        assert_eq!(mirror.counters().schema_flushes, flushes + 1, "{table}");
        assert!(!mirror.globals_unchanged(built), "{table}");
    }
    pg.done().await;
}

#[tokio::test]
async fn pg_disabled_dropped_or_replaced_required_definitions_change_the_stamp() {
    let Some(pg) =
        pg("pg_disabled_dropped_or_replaced_required_definitions_change_the_stamp").await
    else {
        return;
    };
    let (_, session) = seed(&pg.store).await;
    let valid = |pg: &Pg| {
        let s = pg.store.clone();
        let session = session.clone();
        async move {
            let current = fence(&s, &session).await;
            s.validate_native_publication_schema(&current)
                .await
                .unwrap()
        }
    };
    assert!(valid(&pg).await);
    let required = super::schema::required_triggers(true);
    assert!(
        required
            .iter()
            .any(|(name, _)| name == "settings_revision_setting_policy")
    );
    assert!(
        required
            .iter()
            .any(|(name, _)| name == "authority_change_log_epoch")
    );
    for (name, table) in &required {
        let base = pg.stamp().await;
        pg.exec(&format!("ALTER TABLE {table} DISABLE TRIGGER {name}"))
            .await;
        let disabled = pg.stamp().await;
        assert_ne!(disabled, base, "DISABLE {name}");
        assert!(!valid(&pg).await, "disabled {name} disqualifies");
        pg.exec(&format!("ALTER TABLE {table} ENABLE TRIGGER {name}"))
            .await;
        assert_ne!(pg.stamp().await, disabled, "ENABLE {name}");
        assert!(valid(&pg).await, "re-enabled {name}");
    }
    // The mirror flushes on the next poll.
    let mirror = EpochMirror::manual(&pg.store, PG_BOUND);
    settle(&mirror).await;
    let (current, built) = stamped(&pg.store, &mirror, &session).await;
    pg.exec("ALTER TABLE settings_revision DISABLE TRIGGER authority_change_log_revision")
        .await;
    assert!(mirror.poll_now().await.unwrap());
    assert_eq!(mirror.counters().schema_flushes, 1);
    assert!(!mirror.still_valid(&current, built));
    pg.exec("ALTER TABLE settings_revision ENABLE TRIGGER authority_change_log_revision")
        .await;
    // A replaced function body changes the stamp and fails validation.
    let base = pg.stamp().await;
    pg.exec("CREATE OR REPLACE FUNCTION settings_revision_bump() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$")
        .await;
    assert_ne!(pg.stamp().await, base, "replaced function");
    assert!(!valid(&pg).await);
    // DROP of a 0120 trigger and of a 0078 trigger.
    for (name, table) in [
        ("authority_change_log_epoch", "authority_epoch"),
        // 0123 retired the identity-table ones; a remaining 0078 trigger.
        ("settings_revision_setting_policy", "setting_policy"),
    ] {
        let base = pg.stamp().await;
        pg.exec(&format!("DROP TRIGGER {name} ON {table}")).await;
        assert_ne!(pg.stamp().await, base, "DROP {name}");
        assert!(!valid(&pg).await);
    }
    pg.done().await;
}

#[tokio::test]
async fn pg_prune_never_removes_an_unread_row() {
    let Some(pg) = pg("pg_prune_never_removes_an_unread_row").await else {
        return;
    };
    let (account, session) = seed(&pg.store).await;
    let mirror = EpochMirror::manual(&pg.store, PG_BOUND);
    // Test timing: mark on every poll, prune below a mark at once.
    mirror
        .set_prune_timing(Duration::ZERO, Duration::ZERO)
        .await;
    settle(&mirror).await;
    let mut a = pg.conn().await;
    sqlx::query("BEGIN").execute(&mut a).await.unwrap();
    sqlx::query(BUMP)
        .bind("session")
        .bind(session.as_str())
        .execute(&mut a)
        .await
        .unwrap();
    let (a_xid,): (String,) =
        sqlx::query_as("SELECT CAST(pg_catalog.pg_current_xact_id() AS TEXT)")
            .fetch_one(&mut a)
            .await
            .unwrap();
    let (current, built) = stamped(&pg.store, &mirror, &session).await;
    for i in 0..4 {
        pg.store
            .update_sync_policy(&account, &format!(r#"{{"prune":{i}}}"#))
            .await
            .unwrap();
        mirror.poll_now().await.unwrap();
    }
    assert!(mirror.counters().pruned_rows > 0, "{:?}", mirror.counters());
    assert_eq!(mirror.counters().prune_failures, 0);
    sqlx::query("COMMIT").execute(&mut a).await.unwrap();
    let survived = q(
        "SELECT COUNT(*) AS n FROM authority_change_log WHERE xid=CAST(CAST(?1 AS TEXT) AS xid8)",
    )
    .bind(a_xid.as_str())
    .fetch_one(&pg.store.backend)
    .await
    .unwrap();
    assert_eq!(number(&survived, "n").unwrap(), Some(1), "unread row kept");
    mirror.poll_now().await.unwrap();
    assert!(
        !mirror.still_valid(&current, built),
        "and applied after its commit"
    );

    // A poller that stopped for longer than the bound flushes when it resumes,
    // because the rows it did not read may be gone.
    let paused = EpochMirror::manual(&pg.store, Duration::from_millis(200));
    settle(&paused).await;
    let (current, built) = stamped(&pg.store, &paused, &session).await;
    tokio::time::sleep(Duration::from_millis(250)).await;
    pg.store
        .update_sync_policy(&account, r#"{"paused":1}"#)
        .await
        .unwrap();
    for _ in 0..3 {
        mirror.poll_now().await.unwrap();
    }
    assert!(paused.poll_now().await.unwrap());
    assert_eq!(paused.counters().gap_flushes, 1);
    assert!(!paused.still_valid(&current, built));
    pg.done().await;
}

#[tokio::test]
async fn pg_poll_failure_is_not_fresh_and_reconnects() {
    let Some(mut pg) = pg("pg_poll_failure_is_not_fresh_and_reconnects").await else {
        return;
    };
    let bound = Duration::from_millis(500);
    let (_, session) = seed(&pg.store).await;
    let mirror = EpochMirror::manual(&pg.store, bound);
    settle(&mirror).await;
    assert_eq!(mirror.counters().connects, 1);
    let (current, built) = stamped(&pg.store, &mirror, &session).await;
    let pid = mirror.poll_backend_pid().await.unwrap();
    sqlx::query("SELECT pg_catalog.pg_terminate_backend($1)")
        .bind(pid)
        .execute(&mut pg.admin)
        .await
        .unwrap();
    assert!(mirror.poll_now().await.is_err(), "lost poll connection");
    assert_eq!(mirror.counters().failures, 1);
    tokio::time::sleep(bound + Duration::from_millis(50)).await;
    assert!(mirror.stamp_gen().is_none(), "failed polls: not fresh");
    assert!(!mirror.is_fresh());
    assert!(!mirror.still_valid(&current, built));
    // The next poll opens a new connection; the gap flushes everything stamped
    // before the failure.
    assert!(mirror.poll_now().await.unwrap());
    assert_eq!(mirror.counters().connects, 2);
    assert_eq!(mirror.counters().gap_flushes, 1);
    settle(&mirror).await;
    assert!(!mirror.still_valid(&current, built));
    let (current, built) = stamped(&pg.store, &mirror, &session).await;
    assert!(mirror.still_valid(&current, built));
    pg.done().await;
}

#[tokio::test]
async fn pg_concurrent_epoch_writers_do_not_wait_on_each_other() {
    let Some(pg) = pg("pg_concurrent_epoch_writers_do_not_wait_on_each_other").await else {
        return;
    };
    let (account, session) = seed(&pg.store).await;
    // Past the 0115 horizon that made the per-insert prune contend (t34 S2).
    exec(&pg.store, "INSERT INTO authority_change_log(scope_kind,incarnation,at) SELECT 'probe','probe',0 FROM generate_series(1,66000)").await;
    let mirror = EpochMirror::manual(&pg.store, PG_BOUND);
    mirror
        .set_prune_timing(Duration::ZERO, Duration::ZERO)
        .await;
    settle(&mirror).await;

    // The S2 interleaving: A bumps a session epoch, B bumps an account epoch and
    // commits while A is open, then A bumps the same account.
    let mut a = pg.conn().await;
    let mut b = pg.conn().await;
    sqlx::query("BEGIN").execute(&mut a).await.unwrap();
    sqlx::query(BUMP)
        .bind("session")
        .bind(session.as_str())
        .execute(&mut a)
        .await
        .unwrap();
    let b_done = tokio::time::timeout(Duration::from_secs(5), async {
        sqlx::query("BEGIN").execute(&mut b).await?;
        sqlx::query(BUMP)
            .bind("account")
            .bind(account.as_str())
            .execute(&mut b)
            .await?;
        sqlx::query("COMMIT").execute(&mut b).await
    })
    .await;
    assert!(
        matches!(b_done, Ok(Ok(_))),
        "B must not wait on A: {b_done:?}"
    );
    sqlx::query(BUMP)
        .bind("account")
        .bind(account.as_str())
        .execute(&mut a)
        .await
        .unwrap();
    sqlx::query("COMMIT").execute(&mut a).await.unwrap();

    // 16 writers, each holding its transaction open across a barrier while the
    // mirror polls and prunes.
    let writers = 16;
    let barrier = Arc::new(tokio::sync::Barrier::new(writers + 1));
    let mut tasks = Vec::new();
    for i in 0..writers {
        let dsn = pg.dsn.clone();
        let barrier = barrier.clone();
        tasks.push(tokio::spawn(async move {
            use sqlx::Connection;
            let mut c = sqlx::PgConnection::connect(&dsn).await?;
            sqlx::query("BEGIN").execute(&mut c).await?;
            sqlx::query(BUMP)
                .bind("account")
                .bind(format!("writer-{i}-a"))
                .execute(&mut c)
                .await?;
            barrier.wait().await;
            sqlx::query(BUMP)
                .bind("account")
                .bind(format!("writer-{i}-b"))
                .execute(&mut c)
                .await?;
            sqlx::query("COMMIT").execute(&mut c).await?;
            Ok::<_, sqlx::Error>(())
        }));
    }
    let polled = tokio::time::timeout(Duration::from_secs(20), async {
        barrier.wait().await;
        for _ in 0..5 {
            mirror.poll_now().await.unwrap();
        }
    })
    .await;
    assert!(polled.is_ok(), "writers reached the barrier");
    let done = tokio::time::timeout(Duration::from_secs(20), async {
        let mut results = Vec::new();
        for task in tasks {
            results.push(task.await.unwrap());
        }
        results
    })
    .await
    .expect("no writer waits on another");
    for result in done {
        result.expect("no 40P01 and no lock wait");
    }
    // The pruner catches up batch by batch, outside every writer.
    for _ in 0..40 {
        if mirror.counters().pruned_rows >= 66_000 {
            break;
        }
        mirror.poll_now().await.unwrap();
    }
    settle(&mirror).await;
    let counters = mirror.counters();
    assert_eq!(counters.failures, 0, "{counters:?}");
    assert_eq!(counters.prune_failures, 0, "{counters:?}");
    assert!(counters.pruned_rows >= 66_000, "{counters:?}");
    pg.done().await;
}
