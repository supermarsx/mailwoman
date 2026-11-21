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


#[tokio::test]
async fn selected_strict_resolution_matches_individual_domain_profile_group_personal_and_locks() {
    use mw_store::feature_profiles::{FeatureProfile, ProfileValue, ProfileAssignment};
    use mw_store::groups::UserGroup;
    use std::collections::BTreeMap;
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    store.put_domain_config(&mw_store::DomainConfigRow { name:"example.test".into(), upstream_json:"{}".into(), allowlist_json:"[]".into(), blocklist_json:"[]".into(), enabled:true, preferred:false, settings_json:"{}".into(), updated_at:1 }).await.unwrap();
    store.put_domain_alias("alias.test", "example.test").await.unwrap();
    for (key, value) in [("privacy.remoteContent", "\"ask\""), ("attachments.downloadAllowed", "true"), ("appearance.theme", "\"system\"")] {
        store.put_setting_policy(&SettingPolicyRow { key:key.into(), global_value:Some(value.into()), allow_user_override:true, global_locked:false, updated_by:"fixture".into(), updated_at:1 }).await.unwrap();
    }
    store.put_feature_profile(&FeatureProfile { id:"selected-profile".into(), name:"Selected".into(), values:BTreeMap::from([("privacy.remoteContent".into(), ProfileValue { value:json!("allow"), locked:false })]), builtin:false, updated_at:1 }, "fixture").await.unwrap();
    store.put_user_group(&UserGroup { id:"selected-group".into(), name:"Selected".into(), domain:"example.test".into(), source:"manual".into(), ldap_dn:None, created_at:1 }).await.unwrap();
    store.replace_user_group_members("selected-group", &["personal-owner".into()]).await.unwrap();
    store.put_feature_profile_assignment(&ProfileAssignment { scope_kind:"group".into(), scope_id:"selected-group".into(), profile_id:"selected-profile".into(), priority:10 }).await.unwrap();
    for (scope,id,key,value,locked) in [
        (SettingScope::Domain,"alias.test","privacy.remoteContent","\"block\"",true),
        (SettingScope::Group,"selected-group","privacy.remoteContent","\"allow\"",false),
        (SettingScope::AdminUser,"account","appearance.theme","\"dark\"",true),
        (SettingScope::User,"account","privacy.remoteContent","\"ask\"",false),
        // This deliberately illegal user layer must not override admin-only GD.
        (SettingScope::User,"personal-owner","attachments.downloadAllowed","false",false),
    ] {
        store.put_setting_value(&SettingValueRow { scope, scope_id:id.into(), key:key.into(), value:value.into(), locked, updated_by:"fixture".into(), updated_at:1 }).await.unwrap();
    }
    store.put_personal_setting("personal-owner", "privacy.remoteContent", "\"allow\"").await.unwrap();
    store.put_personal_setting("personal-owner", "attachments.downloadAllowed", "false").await.unwrap();
    let legacy_subject = SettingSubject { personal_owner:None, domain:"alias.test", account_id:"account", address:"user@alias.test" };
    let held = effective_selected_for(&store,&["appearance.theme"],&legacy_subject,&[]).await.unwrap();
    assert_eq!(held[0],effective_for(&store,"appearance.theme",&legacy_subject).await.unwrap());
    assert_eq!(held[0].as_ref().unwrap().locked_by,Some(SettingSource::AdminUser));
    assert_eq!(held[0].as_ref().unwrap().value,json!("dark"));
    let subject = SettingSubject { personal_owner:Some("personal-owner"), domain:"alias.test", account_id:"account", address:"user@alias.test" };
    let keys = ["privacy.remoteContent","appearance.theme","attachments.downloadAllowed","fixture.unknown","privacy.remoteContent","attachments.maxBytes"];
    let selected = effective_selected_for(&store, &keys, &subject, &[]).await.unwrap();
    for (key,value) in keys.iter().zip(&selected) {
        assert_eq!(value, &effective_for(&store,key,&subject).await.unwrap());
    }
    assert_eq!(selected[0].as_ref().unwrap().value,json!("block"));
    assert_eq!(selected[0].as_ref().unwrap().locked_by,Some(SettingSource::Domain));
    assert_eq!(selected[2].as_ref().unwrap().value,json!(true));
    assert!(selected[3].is_none());
    assert_eq!(selected[0],selected[4]);
    // Real projected profile/group membership must have been included even when
    // a broader domain lock ultimately wins.
    let inputs=store.settings_resolution_inputs_selected(&subject,&keys).await.unwrap().unwrap();
    assert!(inputs.values.iter().any(|v|v.scope==SettingScope::Profile));
    assert!(inputs.values.iter().any(|v|v.scope==SettingScope::Group));
    assert!(inputs.values.iter().any(|v|v.scope==SettingScope::User&&v.scope_id=="personal-owner"));
}

#[tokio::test]
async fn selected_registry_preserves_extra_rows_missing_defaults_and_environment_precedence() {
    let store=Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let subject=SettingSubject { account_id:"account", ..SettingSubject::default() };
    let extras=[SettingValueRow { scope:SettingScope::User,scope_id:"account".into(),key:"appearance.theme".into(),value:"\"dark\"".into(),locked:false,updated_by:"fixture".into(),updated_at:1 }];
    let keys=["appearance.theme","attachments.previewAllowed","fixture.unknown"];
    let actual=effective_selected_for(&store,&keys,&subject,&extras).await.unwrap();
    for (key,value) in keys.iter().zip(actual) { assert_eq!(value,effective_for_with(&store,key,&subject,&extras).await.unwrap()); }
    store.put_setting_policy(&SettingPolicyRow {key:"attachments.previewAllowed".into(),global_value:Some("malformed-json".into()),allow_user_override:true,global_locked:false,updated_by:"fixture".into(),updated_at:1}).await.unwrap();
    assert_eq!(effective_selected_for(&store,&["attachments.previewAllowed"],&subject,&[]).await.unwrap()[0],effective_for(&store,"attachments.previewAllowed",&subject).await.unwrap());
    let keys=["realtime.recheckSecs","upstream.default"];
    for (key,value) in [(keys[0],"99"),(keys[1],"\"https://stored.example.test\"")] {
        store.put_setting_policy(&SettingPolicyRow {key:key.into(),global_value:Some(value.into()),allow_user_override:true,global_locked:false,updated_by:"fixture".into(),updated_at:1}).await.unwrap();
    }
    let input=store.settings_resolution_inputs_selected(&subject,&keys).await.unwrap().unwrap();
    let env=|key:&str|match key { "MW_REALTIME_RECHECK_SECS"=>Some("45".into()),"MW_UPSTREAM_DEFAULT"=>Some("https://environment.example.test".into()),_=>None };
    let actual=effective_selected_from_inputs(input,&keys,&[],&env);
    assert_eq!(actual[0].as_ref().unwrap().source,SettingSource::EnvPin);
    assert_eq!(actual[0].as_ref().unwrap().value,json!(45));
    assert_eq!(actual[1].as_ref().unwrap().value,json!("https://stored.example.test"));
}
