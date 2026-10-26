use mw_store::{ServerKey, Store, branding::BrandingUpdate, settings_policy::SettingPolicyRow};

async fn exercise(store: Store) {
    let key = "appearance.brandName";
    let before = SettingPolicyRow {
        key: key.into(),
        global_value: Some("\"Before\"".into()),
        allow_user_override: false,
        global_locked: true,
        updated_by: "fixture".into(),
        updated_at: 1,
    };
    store.put_setting_policy(&before).await.unwrap();
    let a = SettingPolicyRow {
        global_value: Some("\"A\"".into()),
        ..before.clone()
    };
    let b = SettingPolicyRow {
        global_value: Some("\"B\"".into()),
        ..before.clone()
    };
    let (left, right) = tokio::join!(
        store.compare_setting_policy(key, Some(&before), Some(&a)),
        store.compare_setting_policy(key, Some(&before), Some(&b))
    );
    assert_eq!(
        usize::from(left.unwrap()) + usize::from(right.unwrap()),
        1,
        "a stale conditional writer must not overwrite the winner"
    );
    assert!(
        !store
            .compare_setting_policy(key, Some(&before), None)
            .await
            .unwrap(),
        "stale deletion must not remove the winner"
    );
    let winner = store.get_setting_policy(key).await.unwrap().unwrap();
    assert!(
        store
            .compare_setting_policy(key, Some(&winner), None)
            .await
            .unwrap()
    );
    let (left, right) = tokio::join!(
        store.compare_setting_policy(key, None, Some(&a)),
        store.compare_setting_policy(key, None, Some(&b))
    );
    assert_eq!(
        usize::from(left.unwrap()) + usize::from(right.unwrap()),
        1,
        "only one absent-row creator wins"
    );
    assert!(
        !store.compare_setting_policy(key, None, None).await.unwrap(),
        "an absent-row delete must not erase a concurrent creation"
    );
    for _ in 0..8 {
        store.put_setting_policy(&before).await.unwrap();
        let branding = store.branding_snapshot().await.unwrap();
        let (policy, brand) = tokio::join!(
            store.compare_setting_policy(key, Some(&before), Some(&a)),
            store.put_branding(
                &branding,
                BrandingUpdate {
                    images: &branding.images,
                    name_json: "\"Editor\"",
                    notice_json: "null",
                    actor: "editor",
                    now: 2
                }
            )
        );
        assert_eq!(
            usize::from(policy.unwrap()) + usize::from(brand.unwrap()),
            1,
            "registry and branding writers share one conditional authority"
        );
    }
}
#[tokio::test]
async fn conditional_policy_writes_sqlite() {
    exercise(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn conditional_policy_writes_postgres() {
    if let Ok(url) = std::env::var("MW_TEST_PG") {
        exercise(Store::open(&url, ServerKey::generate()).await.unwrap()).await;
    }
}
