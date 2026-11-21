use super::*;
use crate::{DomainConfigRow, ServerKey};
fn subject() -> SettingSubject<'static> {
    SettingSubject {
        personal_owner: None,
        domain: "alias.test",
        account_id: "Account-ID",
        address: "User@Alias.Test",
    }
}
fn policy(value: String) -> SettingPolicyRow {
    SettingPolicyRow {
        key: "fixture.resolution".into(),
        global_value: Some(value),
        allow_user_override: true,
        global_locked: false,
        updated_by: "fixture".into(),
        updated_at: 1,
    }
}
async fn store() -> Store {
    Store::open_in_memory(ServerKey::generate()).await.unwrap()
}

#[tokio::test]
async fn revision_proofs_are_store_scoped_but_work_for_clones() {
    let first = store().await;
    let second = store().await;
    let revision = first.settings_read_revision().await.unwrap();
    assert_eq!(revision.scope(), first.cache_scope());
    assert!(
        second
            .settings_resolution_inputs_at(&SettingSubject::default(), &revision)
            .await
            .is_err()
    );
    assert!(
        first
            .clone()
            .settings_resolution_inputs_at(&SettingSubject::default(), &revision)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn raw_external_writes_change_next_resolution_and_missing_revision_disables_memos() {
    let store = store().await;
    store.put_setting_policy(&policy("1".into())).await.unwrap();
    let before = store.settings_read_revision().await.unwrap();
    let input = store
        .settings_resolution_inputs_at(&SettingSubject::default(), &before)
        .await
        .unwrap();
    assert_eq!(input.policies[0].global_value.as_deref(), Some("1"));
    // Bypass every Store policy writer: this models another process/SQL client.
    q("UPDATE setting_policy SET global_value='2' WHERE key='fixture.resolution'")
        .execute(&store.backend)
        .await
        .unwrap();
    let after = store.settings_read_revision().await.unwrap();
    assert!(after.number() > before.number());
    let input = store
        .settings_resolution_inputs_at(&SettingSubject::default(), &after)
        .await
        .unwrap();
    assert_eq!(input.policies[0].global_value.as_deref(), Some("2"));
    q("DELETE FROM settings_revision WHERE id=1")
        .execute(&store.backend)
        .await
        .unwrap();
    assert_eq!(store.settings_read_revision().await.unwrap().number(), None);
    q("UPDATE setting_policy SET global_value='3' WHERE key='fixture.resolution'")
        .execute(&store.backend)
        .await
        .unwrap();
    assert_eq!(
        store
            .settings_resolution_inputs(&SettingSubject::default())
            .await
            .unwrap()
            .policies[0]
            .global_value
            .as_deref(),
        Some("3")
    );
    q("UPDATE setting_policy SET global_value='4' WHERE key='fixture.resolution'")
        .execute(&store.backend)
        .await
        .unwrap();
    assert_eq!(
        store
            .settings_resolution_inputs(&SettingSubject::default())
            .await
            .unwrap()
            .policies[0]
            .global_value
            .as_deref(),
        Some("4")
    );
}

#[tokio::test]
async fn aliases_profiles_and_locks_match_existing_subject_helpers() {
    let store = store().await;
    store
        .put_domain_config(&DomainConfigRow {
            name: "canonical.test".into(),
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
    store
        .put_domain_alias("alias.test", "canonical.test")
        .await
        .unwrap();
    store.put_setting_policy(&policy("1".into())).await.unwrap();
    q("INSERT INTO feature_profile(id,name,values_json,builtin,updated_at) VALUES('resolution-profile','Resolution fixture','{}',0,1)")
        .execute(&store.backend).await.unwrap();
    q("INSERT INTO feature_profile_assignment(scope_kind,scope_id,profile_id,priority) VALUES('user','Account-ID','resolution-profile',20)")
        .execute(&store.backend).await.unwrap();
    for (scope, id, value, locked) in [
        (SettingScope::Domain, "alias.test", "10", true),
        (SettingScope::Profile, "resolution-profile", "20", false),
        (SettingScope::User, "Account-ID", "30", false),
    ] {
        store
            .put_setting_value(&SettingValueRow {
                scope,
                scope_id: id.into(),
                key: "fixture.resolution".into(),
                value: value.into(),
                locked,
                updated_by: "fixture".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
    }
    let subject = subject();
    let revision = store.settings_read_revision().await.unwrap();
    let inputs = store
        .settings_resolution_inputs_at(&subject, &revision)
        .await
        .unwrap();
    assert_eq!(
        inputs.canonical,
        store.canonical_setting_subject(&subject).await.unwrap()
    );
    assert_eq!(inputs.canonical.domain, "canonical.test");
    assert_eq!(inputs.canonical.address, "user@canonical.test");
    assert_eq!(
        inputs.values,
        store.setting_values_for_subject(&subject).await.unwrap()
    );
    assert!(
        inputs
            .values
            .iter()
            .any(|v| v.scope == SettingScope::Profile)
    );
    let values = inputs
        .values
        .iter()
        .filter(|v| v.key == "fixture.resolution")
        .collect::<Vec<_>>();
    let resolved = resolve(&ResolveInput {
        floor: "0",
        floor_is_env: false,
        pin: None,
        policy: inputs.policies.first(),
        values: &values,
    });
    assert_eq!(resolved.value, "10");
    assert_eq!(resolved.locked_by, Some(SettingSource::Domain));
    assert_eq!(resolved.user_value.as_deref(), Some("30"));
    assert_eq!(
        store
            .held_setting("fixture.resolution", "0", &subject)
            .await
            .unwrap(),
        Some(resolved.value)
    );
    let again = store
        .settings_resolution_inputs_at(&subject, &revision)
        .await
        .unwrap();
    assert_eq!(again.canonical, inputs.canonical);
    assert_eq!(again.policies, inputs.policies);
    assert_eq!(again.values, inputs.values);
}

#[tokio::test]
async fn oversized_all_policy_snapshot_is_resolved_without_memo_retention() {
    let store = store().await;
    let value = format!("\"{}\"", "x".repeat(2 * 1024 * 1024));
    store
        .put_setting_policy(&policy(value.clone()))
        .await
        .unwrap();
    let revision = store.settings_read_revision().await.unwrap();
    let inputs = store
        .settings_resolution_inputs_at(&SettingSubject::default(), &revision)
        .await
        .unwrap();
    assert_eq!(
        inputs.policies[0].global_value.as_deref(),
        Some(value.as_str())
    );
    assert!(
        ALL_POLICIES_MEMO
            .get(store.cache_scope(), "all", revision.number().unwrap())
            .is_none()
    );
}

#[tokio::test]
async fn single_key_resolution_never_loads_or_memoizes_unrelated_policies() {
    let store = store().await;
    store.put_setting_policy(&policy("1".into())).await.unwrap();
    let mut unrelated = policy(format!("\"{}\"", "x".repeat(2 * 1024 * 1024)));
    unrelated.key = "fixture.unrelated".into();
    store.put_setting_policy(&unrelated).await.unwrap();
    let subject = SettingSubject::default();
    let revision = store
        .settings_read_revision()
        .await
        .unwrap()
        .number()
        .unwrap();
    let inputs = store
        .settings_resolution_inputs_for(&subject, "fixture.resolution")
        .await
        .unwrap();
    assert_eq!(inputs.policies, vec![policy("1".into())]);
    assert!(
        ALL_POLICIES_MEMO
            .get(store.cache_scope(), "all", revision)
            .is_none()
    );
    assert!(
        POLICY_MEMO
            .get(store.cache_scope(), "fixture.resolution", revision)
            .is_some()
    );
    assert!(
        POLICY_MEMO
            .get(store.cache_scope(), "fixture.unrelated", revision)
            .is_none()
    );
    let missing = store
        .settings_resolution_inputs_for(&subject, "fixture.missing")
        .await
        .unwrap();
    assert!(missing.policies.is_empty());
    let bulk = store.settings_resolution_inputs(&subject).await.unwrap();
    assert_eq!(inputs.canonical, bulk.canonical);
    assert_eq!(inputs.values, bulk.values);
    q("UPDATE setting_policy SET global_value='2' WHERE key='fixture.resolution'")
        .execute(&store.backend)
        .await
        .unwrap();
    assert_eq!(
        store
            .settings_resolution_inputs_for(&subject, "fixture.resolution")
            .await
            .unwrap()
            .policies[0]
            .global_value
            .as_deref(),
        Some("2")
    );
    q("DELETE FROM settings_revision WHERE id=1")
        .execute(&store.backend)
        .await
        .unwrap();
    q("UPDATE setting_policy SET global_value='3' WHERE key='fixture.resolution'")
        .execute(&store.backend)
        .await
        .unwrap();
    assert_eq!(
        store
            .settings_resolution_inputs_for(&subject, "fixture.resolution")
            .await
            .unwrap()
            .policies[0]
            .global_value
            .as_deref(),
        Some("3")
    );
}


const SELECTED_KEYS: [&str; 9] = [
    "attachments.previewAllowed", "attachments.downloadAllowed", "attachments.maxBytes",
    "attachments.allowedExtensions", "attachments.blockedExtensions", "attachments.warnExtensions",
    "attachments.allowedMimeTypes", "attachments.blockedMimeTypes", "attachments.warnMimeTypes",
];
async fn seed_selected(store: &Store) {
    for key in SELECTED_KEYS {
        let mut row = policy("false".into()); row.key = key.into();
        store.put_setting_policy(&row).await.unwrap();
    }
}
async fn selected_reference_contract(store: &Store) -> u64 {
    seed_selected(store).await;
    let subject = SettingSubject::default();
    let (cold, cold_reads) = crate::pool_telemetry::measure_queries(
        store.settings_resolution_inputs_selected(&subject, &SELECTED_KEYS)
    ).await;
    let cold = cold.unwrap().unwrap();
    let (warm, warm_reads) = crate::pool_telemetry::measure_queries(
        store.settings_resolution_inputs_selected(&subject, &SELECTED_KEYS)
    ).await;
    let warm = warm.unwrap().unwrap();
    assert_eq!(warm.canonical, cold.canonical);
    assert_eq!(warm.values, cold.values);
    assert_eq!(warm_reads.read.calls, 2, "two fresh revision fences, no per-key/source reads");
    assert!(cold_reads.read.calls > warm_reads.read.calls);
    let (reference, reference_reads) = crate::pool_telemetry::measure_queries(async {
        let mut out = Vec::new();
        for key in SELECTED_KEYS { out.push(store.settings_resolution_inputs_for(&subject, key).await.unwrap()); }
        out
    }).await;
    assert_eq!(reference_reads.read.calls, 9);
    for (key, input) in SELECTED_KEYS.into_iter().zip(reference) {
        assert_eq!(input.canonical, warm.canonical);
        assert_eq!(input.values, warm.values);
        assert_eq!(input.policies.first(), warm.policies.iter().find(|row| row.key == key));
    }
    let revision = store.settings_revision().await.unwrap().unwrap();
    assert!(ALL_POLICIES_MEMO.get(store.cache_scope(), "all", revision).is_none());
    // A source write after the collection's first revision must reject the
    // entire frame. The private seam is exactly what the public reader calls.
    q("UPDATE setting_policy SET global_value='true' WHERE key='attachments.previewAllowed'")
        .execute(&store.backend).await.unwrap();
    assert!(store.collect_selected_settings_at(&subject, &SELECTED_KEYS, revision).await.unwrap().is_none());
    let current = store.settings_resolution_inputs_selected(&subject, &SELECTED_KEYS).await.unwrap().unwrap();
    assert_eq!(current.policies.iter().find(|p| p.key == SELECTED_KEYS[0]).unwrap().global_value.as_deref(), Some("true"));
    q("DELETE FROM settings_revision WHERE id=1").execute(&store.backend).await.unwrap();
    assert!(store.settings_resolution_inputs_selected(&subject, &SELECTED_KEYS).await.unwrap().is_none());
    // Missing revision keeps the original uncached reader available.
    assert_eq!(store.settings_resolution_inputs_for(&subject, SELECTED_KEYS[0]).await.unwrap().policies[0].global_value.as_deref(), Some("true"));
    println!("{{\"kind\":\"selected-settings-counts\",\"coldReads\":{},\"warmReads\":{},\"scalarWarmReads\":{}}}", cold_reads.read.calls, warm_reads.read.calls, reference_reads.read.calls);
    cold_reads.read.calls
}
#[tokio::test]
async fn selected_settings_sqlite_reference_counts_raw_invalidation_and_missing_revision() {
    let cold_batch = selected_reference_contract(&store().await).await;
    let reference = store().await;
    seed_selected(&reference).await;
    let (_, counts) = crate::pool_telemetry::measure_queries(async {
        for key in SELECTED_KEYS { reference.settings_resolution_inputs_for(&SettingSubject::default(), key).await.unwrap(); }
    }).await;
    assert!(cold_batch < counts.read.calls, "cold selected collector must reduce source roundtrips against fresh scalar readers");
}
#[tokio::test]
#[ignore = "requires explicit owned MW_TEST_PG fixture; must run with --ignored"]
async fn selected_settings_postgres_reference_counts_raw_invalidation_and_missing_revision() {
    use sqlx::Connection;
    let dsn = std::env::var("MW_TEST_PG").expect("explicit PostgreSQL test requires private owned fixture");
    let schema = format!("selected_settings_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&mut admin).await.unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped.query_pairs_mut().append_pair("options", &format!("-csearch_path={schema}"));
    let store = Store::open(scoped.as_str(), ServerKey::generate()).await.unwrap();
    // A test assertion/panic must not leave its owned schema behind.
    let task_store=store.clone();
    let outcome=tokio::spawn(async move { selected_reference_contract(&task_store).await }).await;
    store.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE")).execute(&mut admin).await.unwrap();
    outcome.unwrap();
}
#[tokio::test]
async fn selected_settings_bounds_empty_duplicates_unknown_and_store_scope() {
    let first = store().await;
    let second = store().await;
    seed_selected(&first).await;
    let subject = SettingSubject::default();
    let (_, empty) = crate::pool_telemetry::measure_queries(first.settings_resolution_inputs_selected(&subject, &[])).await;
    assert_eq!(empty.read.calls, 0);
    let (_, bounded) = crate::pool_telemetry::measure_queries(first.settings_resolution_inputs_selected(&subject, &["x";17])).await;
    assert_eq!(bounded.read.calls, 0);
    assert!(first.settings_resolution_inputs_selected(&subject, &["x";17]).await.is_err());
    assert!(first.settings_resolution_inputs_selected(&subject, &[&"x".repeat(4097)]).await.is_err());
    let keys = [SELECTED_KEYS[0], SELECTED_KEYS[0], "fixture.unknown"];
    assert_eq!(first.settings_resolution_inputs_selected(&subject, &keys).await.unwrap().unwrap().policies.len(), 1);
    assert!(second.settings_resolution_inputs_selected(&subject, &keys).await.unwrap().unwrap().policies.is_empty());
    q("UPDATE settings_revision SET n=-1 WHERE id=1").execute(&first.backend).await.unwrap();
    assert!(first.settings_resolution_inputs_selected(&subject, &keys).await.unwrap().is_none());
}
#[tokio::test]
async fn selected_settings_corrupt_alias_fails_like_strict_single_key() {
    let store = store().await;
    for name in ["first.test", "second.test"] {
        store.put_domain_config(&DomainConfigRow { name:name.into(), upstream_json:"{}".into(), allowlist_json:"[]".into(), blocklist_json:"[]".into(), enabled:true, preferred:false, settings_json:"{}".into(), updated_at:1 }).await.unwrap();
    }
    // Bypass the validated alias writer, as an independent SQL writer can.
    q("INSERT INTO domain_alias(alias,domain) VALUES('first.test','second.test')").execute(&store.backend).await.unwrap();
    q("INSERT INTO domain_alias(alias,domain) VALUES('second.test','first.test')").execute(&store.backend).await.unwrap();
    let subject = SettingSubject { domain:"first.test", address:"user@first.test", ..SettingSubject::default() };
    assert!(store.settings_resolution_inputs_for(&subject, SELECTED_KEYS[0]).await.is_err());
    assert!(store.settings_resolution_inputs_selected(&subject, &SELECTED_KEYS).await.is_err());
}


#[tokio::test]
async fn selected_settings_independent_encrypted_store_writer_and_reopen_never_reuse_old_policy() {
    use crate::encryption::{DbKey, EncryptionConfig, EncryptionMode};
    let dir=tempfile::tempdir().unwrap();
    let path=dir.path().join("selected-settings.db");
    let config=EncryptionConfig::with_key(EncryptionMode::Required,DbKey::generate());
    let key=ServerKey::generate();
    let reader=Store::open_with_encryption(path.to_str().unwrap(),key.clone(),&config).await.unwrap();
    let writer=Store::open_with_encryption(path.to_str().unwrap(),key.clone(),&config).await.unwrap();
    seed_selected(&reader).await;
    let subject=SettingSubject::default();
    let first=reader.settings_resolution_inputs_selected(&subject,&SELECTED_KEYS).await.unwrap().unwrap();
    assert_eq!(first.policies.iter().find(|p|p.key==SELECTED_KEYS[0]).unwrap().global_value.as_deref(),Some("false"));
    let revision=reader.settings_revision().await.unwrap().unwrap();
    q("UPDATE setting_policy SET global_value='true' WHERE key='attachments.previewAllowed'").execute(&writer.backend).await.unwrap();
    assert!(reader.collect_selected_settings_at(&subject,&SELECTED_KEYS,revision).await.unwrap().is_none());
    let fresh=reader.settings_resolution_inputs_selected(&subject,&SELECTED_KEYS).await.unwrap().unwrap();
    assert_eq!(fresh.policies.iter().find(|p|p.key==SELECTED_KEYS[0]).unwrap().global_value.as_deref(),Some("true"));
    writer.close().await;reader.close().await;
    let reopened=Store::open_with_encryption(path.to_str().unwrap(),key,&config).await.unwrap();
    let fresh=reopened.settings_resolution_inputs_selected(&subject,&SELECTED_KEYS).await.unwrap().unwrap();
    assert_eq!(fresh.policies.iter().find(|p|p.key==SELECTED_KEYS[0]).unwrap().global_value.as_deref(),Some("true"));
    reopened.close().await;
}
