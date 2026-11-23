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
async fn stamped(s: &Store, mirror: &EpochMirror, session: &str) -> (NativeAuthorityFence, MirrorGen) {
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
    assert!(s.validate_native_publication_schema(&current).await.unwrap());
    exec(&s, "DROP TRIGGER authority_change_log_epoch_update").await;
    let changed = fence(&s, &session).await;
    assert_ne!(changed.publication_schema(), current.publication_schema());
    assert!(
        !s.validate_native_publication_schema(&changed).await.unwrap(),
        "a missing change-log trigger must disqualify published inputs"
    );
    s.close().await;
}

#[tokio::test]
async fn sqlite_source_writes_through_store_and_raw_sql_invalidate() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (account, session) = seed(&s).await;
    let mirror = EpochMirror::manual(&s, BOUND);
    assert!(mirror.stamp_gen().is_none(), "never polled: not fresh");
    settle(&mirror).await;

    // A write that no authority trigger observes keeps the fence valid.
    let (current, built) = stamped(&s, &mirror, &session).await;
    s.set_setting("t34-unrelated-key", "1").await.unwrap();
    mirror.poll_now().await.unwrap();
    assert!(mirror.still_valid(&current, built), "unrelated write must not invalidate");

    // Raw SQL that bypasses the Store API, on a session column the trigger covers.
    q("UPDATE sessions SET user_agent='raw-sql-agent' WHERE id=?1")
        .bind(session.as_str())
        .execute(&s.backend)
        .await
        .unwrap();
    assert!(mirror.still_valid(&current, built), "not yet polled");
    mirror.poll_now().await.unwrap();
    assert!(!mirror.still_valid(&current, built), "raw session write invalidates");

    // Account update through the Store API.
    let (current, built) = stamped(&s, &mirror, &session).await;
    assert!(mirror.still_valid(&current, built));
    s.update_sync_policy(&account, r#"{"t34":true}"#).await.unwrap();
    mirror.poll_now().await.unwrap();
    assert!(!mirror.still_valid(&current, built), "account update invalidates");

    // Raw SQL on the identity chain the fence joins.
    let (current, built) = stamped(&s, &mirror, &session).await;
    q("UPDATE identity_principal SET state='disabled' WHERE id=?1")
        .bind(current.principal_id())
        .execute(&s.backend)
        .await
        .unwrap();
    mirror.poll_now().await.unwrap();
    assert!(!mirror.still_valid(&current, built), "principal state invalidates");
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
    assert!(!mirror.still_valid(&current, built), "policy write invalidates");
    assert!(!mirror.globals_unchanged(built));
    exec(&s, "UPDATE settings_revision SET n=n+1").await;
    let (current, built) = stamped(&s, &mirror, &session).await;
    exec(&s, "UPDATE settings_revision SET n=n+1").await;
    mirror.poll_now().await.unwrap();
    assert!(!mirror.still_valid(&current, built), "raw settings revision invalidates");

    // Session delete through the Store API.
    let (current, built) = stamped(&s, &mirror, &session).await;
    s.delete_session(&session).await.unwrap();
    mirror.poll_now().await.unwrap();
    assert!(!mirror.still_valid(&current, built), "session delete invalidates");
    let counters = mirror.counters();
    assert!(counters.hits >= 3 && counters.misses >= 6, "{counters:?}");
    assert_eq!(counters.failures, 0);
    s.close().await;
}

#[tokio::test]
async fn sqlite_change_committed_after_stamp_is_never_accepted_after_a_poll() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (account, session) = seed(&s).await;
    let mirror = EpochMirror::manual(&s, BOUND);
    settle(&mirror).await;

    // Commit between stamp_gen and the fence read: the fence reflects it, and the
    // conservative mirror still refuses the pair once the change is polled.
    let built = mirror.stamp_gen().unwrap();
    let before = fence(&s, &session).await;
    s.update_sync_policy(&account, r#"{"between":1}"#).await.unwrap();
    let after_change = fence(&s, &session).await;
    let account_scope = AuthorityScope::new(AuthorityKind::Account, &account).unwrap();
    let version = |f: &NativeAuthorityFence| {
        f.versions().stamps.get(&account_scope).unwrap().version
    };
    assert!(version(&after_change) > version(&before));
    mirror.poll_now().await.unwrap();
    assert!(!mirror.still_valid(&after_change, built));

    // Commit after the fence read: the dangerous case.
    let (stale, built) = stamped(&s, &mirror, &session).await;
    s.update_sync_policy(&account, r#"{"after":1}"#).await.unwrap();
    mirror.poll_now().await.unwrap();
    assert!(!mirror.still_valid(&stale, built));
    // A fence read after that poll validates again.
    let (current, built) = stamped(&s, &mirror, &session).await;
    assert!(mirror.still_valid(&current, built));
    // A stamp never validates a fence for another session's scopes either way,
    // and a stamp from another mirror instance never validates.
    let other = EpochMirror::manual(&s, BOUND);
    settle(&other).await;
    assert!(!other.still_valid(&current, built));
    s.close().await;
}

#[tokio::test]
async fn sqlite_background_mirror_refuses_a_change_by_the_bound() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (account, session) = seed(&s).await;
    let mirror = EpochMirror::for_store(&s, BOUND);
    assert!(mirror.is_live());
    assert!(Arc::ptr_eq(&mirror, &EpochMirror::for_store(&s, BOUND)));
    let deadline = Instant::now() + Duration::from_secs(10);
    let ready = || async {
        loop {
            if let Some(built) = mirror.stamp_gen() {
                let current = fence(&s, &session).await;
                if mirror.still_valid(&current, built) {
                    return (current, built);
                }
            }
            assert!(Instant::now() < deadline, "background mirror never became fresh");
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    };
    let (current, built) = ready().await;
    s.update_sync_policy(&account, r#"{"bound":1}"#).await.unwrap();
    let committed = Instant::now();
    tokio::time::sleep_until((committed + BOUND).into()).await;
    // Deterministic: either a poll that started after the commit applied it, or
    // no complete poll started after it and the mirror is not fresh.
    assert!(!mirror.still_valid(&current, built));
    let (_current, _built) = ready().await;
    assert!(mirror.counters().polls >= 2);
    s.close().await;
}

#[tokio::test]
async fn sqlite_expedite_latches_until_a_later_poll_completes() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (_, session) = seed(&s).await;
    let mirror = EpochMirror::manual(&s, BOUND);
    settle(&mirror).await;
    let (current, built) = stamped(&s, &mirror, &session).await;
    assert!(mirror.still_valid(&current, built));
    mirror.expedite();
    assert!(!mirror.still_valid(&current, built));
    assert!(!mirror.globals_unchanged(built));
    mirror.poll_now().await.unwrap();
    assert!(mirror.still_valid(&current, built), "no change: valid again after the poll");
    assert_eq!(mirror.counters().expedites, 1);
    s.close().await;
}

#[tokio::test]
async fn sqlite_poll_failure_goes_stale_after_the_bound() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (_, session) = seed(&s).await;
    let mirror = EpochMirror::manual(&s, BOUND);
    settle(&mirror).await;
    let (current, built) = stamped(&s, &mirror, &session).await;
    exec(&s, "ALTER TABLE authority_change_log RENAME TO authority_change_log_away").await;
    assert!(mirror.poll_now().await.is_err());
    assert_eq!(mirror.counters().failures, 1);
    tokio::time::sleep(BOUND + Duration::from_millis(20)).await;
    assert!(mirror.poll_now().await.is_err());
    assert!(mirror.stamp_gen().is_none(), "failed polls: not fresh");
    assert!(!mirror.is_fresh());
    assert!(!mirror.still_valid(&current, built));
    exec(&s, "ALTER TABLE authority_change_log_away RENAME TO authority_change_log").await;
    settle(&mirror).await;
    assert!(!mirror.still_valid(&current, built), "schema changed: old fence stays cold");
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
    assert!(mirror.stamp_gen().is_none(), "flushed generation does not stamp");
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
    assert!(mirror.poll_now().await.unwrap(), "full page flushes and jumps to the head");
    assert_eq!(mirror.counters().overflow_flushes, 1);
    assert!(!mirror.still_valid(&current, built));
    assert!(mirror.stamp_gen().is_none());
    assert!(!mirror.poll_now().await.unwrap(), "no rows left after the jump");
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
    writer.update_sync_policy(&account, r#"{"other":1}"#).await.unwrap();
    mirror.poll_now().await.unwrap();
    assert!(!mirror.still_valid(&current, built), "other Store handle's write invalidates");

    // Live generation swap to a different file.
    let (current, built) = stamped(&reader, &mirror, &session).await;
    let replacement = Store::open_with_encryption(
        dir.path().join("replacement.db").to_str().unwrap(),
        key,
        &config,
    )
    .await
    .unwrap();
    let Backend::Sqlite(other) = &replacement.backend else { unreachable!() };
    let pool = other.pin().await.pool().clone();
    let Backend::Sqlite(live) = &reader.backend else { unreachable!() };
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
async fn invalid_bound_and_postgres_are_inert() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    for bound in [Duration::from_millis(99), Duration::from_millis(5001)] {
        let mirror = EpochMirror::for_store(&s, bound);
        assert!(!mirror.is_live());
        assert!(!mirror.poll_now().await.unwrap());
        assert!(mirror.stamp_gen().is_none());
    }
    assert!(valid_staleness(DEFAULT_STALENESS));
    s.close().await;
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        mw_test_gate::skip("MW_TEST_PG absent; PostgreSQL change-log contract not executed");
        return;
    };
    use sqlx::Connection;
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
    let pg = Store::open(scoped.as_str(), ServerKey::generate()).await.unwrap();
    let (account, session) = seed(&pg).await;
    let mirror = EpochMirror::for_store(&pg, DEFAULT_STALENESS);
    assert!(!mirror.is_live(), "PostgreSQL: no commit-ordered reader in t34");
    assert!(mirror.stamp_gen().is_none());
    let current = fence(&pg, &session).await;
    assert!(
        pg.validate_native_publication_schema(&current).await.unwrap(),
        "PostgreSQL required definitions include 0115"
    );
    let count = |pg: &Store| {
        let pg = pg.clone();
        async move {
            number(
                &q("SELECT COUNT(*) AS n FROM authority_change_log").fetch_one(&pg.backend).await.unwrap(),
                "n",
            )
            .unwrap()
            .unwrap()
        }
    };
    // 0118 (t34 S2): no change-log triggers on PostgreSQL.
    assert_eq!(count(&pg).await, 0);
    pg.update_sync_policy(&account, r#"{"pg":1}"#).await.unwrap();
    q("UPDATE settings_revision SET n=n+1").execute(&pg.backend).await.unwrap();
    assert_eq!(count(&pg).await, 0, "PostgreSQL authority writes no longer append");
    let triggers = q("SELECT COUNT(*) AS n FROM pg_catalog.pg_trigger WHERE tgname LIKE 'authority_change_log%' AND tgrelid IN (SELECT c.oid FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=current_schema())")
        .fetch_one(&pg.backend)
        .await
        .unwrap();
    assert_eq!(number(&triggers, "n").unwrap().unwrap(), 0);

    // The S2 interleaving: A bumps a session epoch, B bumps an account epoch
    // and commits while A is still open, then A bumps the same account. With the
    // 0115 per-insert prune and a log past the horizon, B waited on A's delete of
    // the oldest log row and A then waited on B: 40P01.
    q("INSERT INTO authority_change_log(scope_kind,incarnation,at) SELECT 'probe','probe',0 FROM generate_series(1,66000)")
        .execute(&pg.backend)
        .await
        .unwrap();
    let bump = "INSERT INTO authority_epoch(scope_kind,scope_id) VALUES($1,$2) ON CONFLICT(scope_kind,scope_id) DO UPDATE SET version=authority_epoch.version+1";
    let mut a = sqlx::PgConnection::connect(scoped.as_str()).await.unwrap();
    let mut b = sqlx::PgConnection::connect(scoped.as_str()).await.unwrap();
    sqlx::query("BEGIN").execute(&mut a).await.unwrap();
    sqlx::query(bump).bind("session").bind(session.as_str()).execute(&mut a).await.unwrap();
    let b_done = tokio::time::timeout(Duration::from_secs(5), async {
        sqlx::query("BEGIN").execute(&mut b).await?;
        sqlx::query(bump).bind("account").bind(account.as_str()).execute(&mut b).await?;
        sqlx::query("COMMIT").execute(&mut b).await
    })
    .await;
    assert!(
        matches!(b_done, Ok(Ok(_))),
        "B must not wait on A's open transaction: {b_done:?}"
    );
    sqlx::query(bump).bind("account").bind(account.as_str()).execute(&mut a).await.unwrap();
    sqlx::query("COMMIT").execute(&mut a).await.unwrap();
    a.close().await.unwrap();
    b.close().await.unwrap();
    pg.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&mut admin)
        .await
        .unwrap();
}
