use super::*;
use mw_store::ServerKey;

#[tokio::test]
async fn shared_revision_batch_matches_individual_registry_resolution_and_locks() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let subject = SettingSubject {
        personal_owner: None,
        domain: "example.org",
        account_id: "account",
        address: "user@example.org",
    };
    store
        .put_setting_policy(&SettingPolicyRow {
            key: "privacy.remoteContent".into(),
            global_value: Some("\"ask\"".into()),
            allow_user_override: true,
            global_locked: false,
            updated_by: "fixture".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    for (scope, scope_id, value, locked) in [
        (SettingScope::Domain, "example.org", "\"block\"", true),
        (SettingScope::User, "account", "\"allow\"", false),
    ] {
        store
            .put_setting_value(&SettingValueRow {
                scope,
                scope_id: scope_id.into(),
                key: "privacy.remoteContent".into(),
                value: value.into(),
                locked,
                updated_by: "fixture".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
    }
    let revision = store.settings_read_revision().await.unwrap();
    let keys = ["privacy.remoteContent", "appearance.theme"];
    let all = effective_all_at_revision(&store, &subject, |d| keys.contains(&d.key), &revision)
        .await
        .unwrap();
    assert_eq!(all.len(), keys.len());
    for key in keys {
        let single = effective_for(&store, key, &subject).await.unwrap().unwrap();
        assert_eq!(all.iter().find(|v| v.key == key).unwrap(), &single);
    }
    let remote = all
        .iter()
        .find(|v| v.key == "privacy.remoteContent")
        .unwrap();
    assert_eq!(remote.value, serde_json::json!("block"));
    assert_eq!(remote.locked_by, Some(SettingSource::Domain));
    assert!(!remote.user_may_override);
    let other = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    assert!(
        effective_all_at_revision(&other, &subject, |_| true, &revision)
            .await
            .is_err()
    );
}
