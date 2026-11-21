use super::*;
use crate::ServerKey;

fn scopes() -> Vec<AuthorityScope> {
    [AuthorityKind::Policy, AuthorityKind::Routing, AuthorityKind::Protection]
        .into_iter().map(|kind| AuthorityScope::new(kind, "").unwrap()).collect()
}
async fn seed(store: &Store) { store.seed_authority_scopes(&scopes()).await.unwrap(); }
async fn source(store: &Store) -> GlobalGuardConfigSource {
    store.global_guard_config_source().await.unwrap().expect("validated global source")
}
async fn changed(store: &Store, old: GlobalGuardConfigFence, sql: &'static str) -> GlobalGuardConfigFence {
    q(sql).execute(&store.backend).await.unwrap();
    let fresh = store.global_guard_config_fence(&old).await.unwrap().unwrap();
    assert!(!old.matches(&fresh).unwrap(), "raw configuration DML must invalidate before reuse");
    fresh
}
async fn hot_contract(store: &Store) {
    q("DELETE FROM authority_epoch WHERE scope_id='' AND scope_kind IN ('policy','routing','protection')")
        .execute(&store.backend).await.unwrap();
    let (absent, calls) = crate::pool_telemetry::measure_queries(store.global_guard_config_source()).await;
    assert!(absent.unwrap().is_none());
    assert_eq!(calls.read.calls + calls.other.calls, 1);
    assert_eq!(calls.write.calls, 0);
    assert_eq!(q("SELECT COUNT(*) FROM authority_epoch WHERE scope_id='' AND scope_kind IN ('policy','routing','protection')")
        .fetch_scalar_i64(&store.backend).await.unwrap(), 0, "missing source must not seed");
    seed(store).await;
    assert_eq!(q("SELECT COUNT(*) FROM sessions").fetch_scalar_i64(&store.backend).await.unwrap(), 0);
    assert_eq!(q("SELECT COUNT(*) FROM accounts").fetch_scalar_i64(&store.backend).await.unwrap(), 0);
    let input = source(store).await;
    let before = input.fence().clone();
    let (fresh, calls) = crate::pool_telemetry::measure_queries(store.global_guard_config_fence(&before)).await;
    assert!(before.matches(&fresh.unwrap().unwrap()).unwrap());
    assert_eq!(calls.read.calls + calls.other.calls, 1, "one complete hot statement");
    assert_eq!(calls.write.calls, 0);
    let foreign = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (result, calls) = crate::pool_telemetry::measure_queries(foreign.global_guard_config_fence(&before)).await;
    assert!(result.is_err()); assert_eq!(calls.read.calls + calls.other.calls + calls.write.calls, 0);
    assert!(input.raw(&foreign, GuardInputSection::Maintenance).is_err());
    foreign.close().await;
    let old = before;
    q("DELETE FROM authority_epoch WHERE scope_kind='protection' AND scope_id=''").execute(&store.backend).await.unwrap();
    let (missing, calls) = crate::pool_telemetry::measure_queries(store.global_guard_config_fence(&old)).await;
    assert!(missing.unwrap().is_none()); assert_eq!(calls.write.calls, 0);
    seed(store).await;
    let recreated = store.global_guard_config_fence(&old).await.unwrap().unwrap();
    assert!(!old.matches(&recreated).unwrap(), "delete/recreate must not ABA");
}
async fn dml_contract(store: &Store) {
    seed(store).await;
    let mut fence = source(store).await.fence().clone();
    for sql in [
        "INSERT INTO setting_policy(key,global_value,allow_user_override,global_locked,updated_by,updated_at) VALUES('global-guard-fixture','true',0,1,'test',1)",
        "UPDATE setting_policy SET global_value='false' WHERE key='global-guard-fixture'",
        "DELETE FROM setting_policy WHERE key='global-guard-fixture'",
    ] { fence = changed(store, fence, sql).await; }
    for (key, section) in [
        ("security.inbound-allowlist.v1", GuardInputSection::InboundAllowlist),
        ("security.clientAccess.v1", GuardInputSection::ClientAccess),
        ("maintenance.mode.v1", GuardInputSection::Maintenance),
        ("v6:admin:bans", GuardInputSection::SourceBans),
    ] {
        for value in ["{}", "{\"changed\":true}"] {
            q("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value")
                .bind(key).bind(value).execute(&store.backend).await.unwrap();
            let fresh = store.global_guard_config_fence(&fence).await.unwrap().unwrap();
            assert!(!fence.matches(&fresh).unwrap()); fence = fresh;
            let input = source(store).await;
            assert_eq!(input.raw(store, section).unwrap(), Some(value));
        }
        q("DELETE FROM settings WHERE key=?1").bind(key).execute(&store.backend).await.unwrap();
        let fresh = store.global_guard_config_fence(&fence).await.unwrap().unwrap();
        assert!(!fence.matches(&fresh).unwrap()); fence = fresh;
        assert_eq!(source(store).await.raw(store, section).unwrap(), None);
    }
    fence = changed(store, fence, "INSERT INTO domains(name,upstream_json,allowlist,blocklist,enabled,preferred,settings_json,updated_at) VALUES('global-guard.test','{}','[]','[]',1,0,'{}',1)").await;
    fence = changed(store, fence, "INSERT INTO domain_alias(alias,domain) VALUES('alias.global-guard.test','global-guard.test')").await;
    fence = changed(store, fence, "DELETE FROM domain_alias WHERE alias='alias.global-guard.test'").await;
    let _ = changed(store, fence, "DELETE FROM domains WHERE name='global-guard.test'").await;
    // Other subjects do not become shared GLOBAL inputs. These are real source
    // rows, not a fake resolver; missing account ownership is intentional DATA.
    q("INSERT INTO setting_policy(key,global_value,allow_user_override,global_locked,updated_by,updated_at) VALUES('global-only-fixture','true',1,0,'test',1)").execute(&store.backend).await.unwrap();
    let before = source(store).await.fence().clone();
    for scope in ["user", "admin_user", "domain"] {
        q("INSERT INTO setting_value(scope_kind,scope_id,key,value,locked,updated_by,updated_at) VALUES(?1,'other-scope','global-only-fixture','false',1,'test',1)")
            .bind(scope).execute(&store.backend).await.unwrap();
    }
    let fresh = store.global_guard_config_fence(&before).await.unwrap().unwrap();
    assert!(before.matches(&fresh).unwrap(), "global vector does not certify other-subject policy");
    let global = source(store).await.into_settings();
    assert!(global.values.is_empty());
    assert!(global.canonical.domain.is_empty() && global.canonical.account_id.is_empty() && global.canonical.address.is_empty());
    assert!(global.policies.iter().any(|row| row.key == "global-only-fixture" && row.global_value.as_deref() == Some("true")));
}
async fn schema_contract(store: &Store) {
    seed(store).await;
    let before = source(store).await.fence().clone();
    let sql = if store.uses_sqlite() { "DROP TRIGGER authority_guard_config_update" } else { "DROP TRIGGER authority_guard_config ON settings" };
    q(sql).execute(&store.backend).await.unwrap();
    assert!(store.global_guard_config_fence(&before).await.unwrap().is_none());
    assert!(store.global_guard_config_source().await.unwrap().is_none(), "cold validation must reject missing actual114 trigger");
}
async fn cold_race_contract(store: &Store, writer: &Store) {
    assert_ne!(store.cache_scope(), writer.cache_scope(), "independent Store source, not a cloned request cache");
    seed(store).await;
    let pause = std::sync::Arc::new(testing::Pause { entered: tokio::sync::Notify::new(), release: tokio::sync::Notify::new() });
    let pending = testing::PAUSE.scope(pause.clone(), store.global_guard_config_source());
    tokio::pin!(pending);
    tokio::select! { _=pause.entered.notified()=>{}, _=&mut pending=>panic!("actual cold extraction must reach after-source seam") }
    q("INSERT INTO settings(key,value) VALUES('maintenance.mode.v1','{}') ON CONFLICT(key) DO UPDATE SET value=excluded.value")
        .execute(&writer.backend).await.unwrap();
    pause.release.notify_one();
    assert!(matches!(pending.await, Err(StoreError::SourceChanged)), "concurrent raw source change cannot install old DATA");
}
async fn malformed_contract(store: &Store) {
    seed(store).await;
    let valid = source(store).await.fence().clone();
    // Corruption is intentionally injected only inside this disposable Store/
    // owned PG schema. Test the SQL projection guard independently of the cold
    // definition validator: neither malformed source can mint an observation.
    if store.uses_sqlite() {
        q("DROP TRIGGER authority_epoch_monotonic").execute(&store.backend).await.unwrap();
        q("DROP TRIGGER authority_epoch_outbox_update").execute(&store.backend).await.unwrap();
        q("PRAGMA ignore_check_constraints=ON").execute(&store.backend).await.unwrap();
    } else {
        q("ALTER TABLE authority_epoch DISABLE TRIGGER USER").execute(&store.backend).await.unwrap();
        q("ALTER TABLE authority_epoch DROP CONSTRAINT authority_epoch_version_check").execute(&store.backend).await.unwrap();
        q("ALTER TABLE authority_epoch DROP CONSTRAINT authority_epoch_incarnation_check").execute(&store.backend).await.unwrap();
    }
    q("UPDATE authority_epoch SET version=0 WHERE scope_kind='policy' AND scope_id=''").execute(&store.backend).await.unwrap();
    assert!(store.observe_global_guard_config().await.unwrap().is_none());
    q("UPDATE authority_epoch SET version=1,incarnation='malformed' WHERE scope_kind='policy' AND scope_id=''").execute(&store.backend).await.unwrap();
    assert!(store.observe_global_guard_config().await.unwrap().is_none());
    assert!(store.global_guard_config_fence(&valid).await.unwrap().is_none());
}
async fn bounds_contract(store: &Store) {
    seed(store).await;
    q("INSERT INTO settings(key,value) VALUES('maintenance.mode.v1',?1)")
        .bind("private-config-marker").execute(&store.backend).await.unwrap();
    let input = source(store).await;
    assert!(!format!("{input:?} {:?}", input.fence()).contains("private-config-marker"));
    q("UPDATE settings SET value=?1 WHERE key='maintenance.mode.v1'")
        .bind("x".repeat(RAW_BYTES + 1)).execute(&store.backend).await.unwrap();
    assert!(store.global_guard_config_source().await.unwrap().is_none(), "raw byte budget must reject before transferring configuration payload");
}

async fn postgres(which: u8) {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else { mw_test_gate::skip("MW_TEST_PG absent; actual global guard PostgreSQL contract not executed"); return; };
    use sqlx::Connection;
    let schema = format!("global_guard_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&mut admin).await.unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped.query_pairs_mut().append_pair("options", &format!("-csearch_path={schema}"));
    let key = ServerKey::generate();
    let store = Store::open(scoped.as_str(), key.clone()).await.unwrap();
    match which {
        0=>hot_contract(&store).await, 1=>dml_contract(&store).await, 2=>schema_contract(&store).await,
        3=>{let writer=Store::open(scoped.as_str(), key).await.unwrap(); cold_race_contract(&store,&writer).await; writer.close().await;},
        4=>malformed_contract(&store).await, 5=>bounds_contract(&store).await, _=>unreachable!()
    }
    store.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE")).execute(&mut admin).await.unwrap();
}
#[tokio::test]
async fn sqlite_global_guard_hot_bound_no_session_no_seed_and_aba() { let s=Store::open_in_memory(ServerKey::generate()).await.unwrap(); hot_contract(&s).await; s.close().await; }
#[tokio::test]
async fn postgres_global_guard_hot_bound_no_session_no_seed_and_aba() { postgres(0).await; }
#[tokio::test]
async fn sqlite_global_guard_raw_dml_and_empty_subject_isolation() { let s=Store::open_in_memory(ServerKey::generate()).await.unwrap(); dml_contract(&s).await; s.close().await; }
#[tokio::test]
async fn postgres_global_guard_raw_dml_and_empty_subject_isolation() { postgres(1).await; }
#[tokio::test]
async fn sqlite_global_guard_schema_mutation_fails_closed() { let s=Store::open_in_memory(ServerKey::generate()).await.unwrap(); schema_contract(&s).await; s.close().await; }
#[tokio::test]
async fn postgres_global_guard_schema_mutation_fails_closed() { postgres(2).await; }
#[tokio::test]
async fn sqlite_global_guard_cold_raw_writer_race_rejects_install() {
    use crate::encryption::{DbKey, EncryptionConfig, EncryptionMode};
    let dir=tempfile::tempdir().unwrap(); let path=dir.path().join("independent-global-writer.db");
    let config=EncryptionConfig::with_key(EncryptionMode::Required,DbKey::generate()); let key=ServerKey::generate();
    let reader=Store::open_with_encryption(path.to_str().unwrap(),key.clone(),&config).await.unwrap();
    let writer=Store::open_with_encryption(path.to_str().unwrap(),key,&config).await.unwrap();
    assert!(q("PRAGMA cipher_version").fetch_opt_scalar_string(&reader.backend).await.unwrap().is_some_and(|version|!version.is_empty()));
    cold_race_contract(&reader,&writer).await; writer.close().await; reader.close().await;
}
#[tokio::test]
async fn postgres_global_guard_cold_raw_writer_race_rejects_install() { postgres(3).await; }
#[tokio::test]
async fn sqlite_global_guard_malformed_epochs_do_not_mint_fence() { let s=Store::open_in_memory(ServerKey::generate()).await.unwrap(); malformed_contract(&s).await; s.close().await; }
#[tokio::test]
async fn postgres_global_guard_malformed_epochs_do_not_mint_fence() { postgres(4).await; }
#[tokio::test]
async fn sqlite_global_guard_raw_budget_and_redacted_debug() { let s=Store::open_in_memory(ServerKey::generate()).await.unwrap(); bounds_contract(&s).await; s.close().await; }
#[tokio::test]
async fn postgres_global_guard_raw_budget_and_redacted_debug() { postgres(5).await; }
#[tokio::test]
async fn sqlite_global_guard_live_generation_swap_rejects_identical_source() {
    use crate::encryption::{DbKey, EncryptionConfig, EncryptionMode};
    let dir = tempfile::tempdir().unwrap();
    let config = EncryptionConfig::with_key(EncryptionMode::Required, DbKey::generate());
    let key = ServerKey::generate();
    let store = Store::open_with_encryption(dir.path().join("first.db").to_str().unwrap(), key.clone(), &config).await.unwrap();
    let replacement = Store::open_with_encryption(dir.path().join("replacement.db").to_str().unwrap(), key, &config).await.unwrap();
    seed(&store).await;
    let original = source(&store).await.fence().clone();
    // Duplicate all observed epoch values so the test cannot pass solely due
    // to different versions/incarnations or a new process-local Store scope.
    q("DELETE FROM authority_epoch WHERE scope_id='' AND scope_kind IN ('policy','routing','protection')").execute(&replacement.backend).await.unwrap();
    for value in original.versions.stamps.values() {
        q("INSERT INTO authority_epoch(scope_kind,scope_id,incarnation,version) VALUES(?1,'',?2,?3)")
            .bind(value.scope.kind.as_str()).bind(&value.incarnation).bind(value.version).execute(&replacement.backend).await.unwrap();
    }
    let Backend::Sqlite(other) = &replacement.backend else { unreachable!() };
    let pool = other.pin().await.pool().clone();
    let Backend::Sqlite(live) = &store.backend else { unreachable!() };
    let mut exclusive = live.exclusive().await;
    exclusive.close_old().await;
    exclusive.replace_with_page_cache(pool, None);
    drop(exclusive);
    let observed = store.observe_global_guard_config().await.unwrap().unwrap();
    assert_eq!(observed.schema, original.schema);
    assert!(observed.versions.stamps == original.versions.stamps);
    assert_ne!(observed.versions.generation, original.versions.generation);
    assert!(store.global_guard_config_fence(&original).await.unwrap().is_none());
    store.close().await; replacement.close().await;
}
