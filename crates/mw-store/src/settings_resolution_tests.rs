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
