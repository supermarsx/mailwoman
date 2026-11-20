use super::*;

#[tokio::test]
async fn absent_key_uses_one_cold_query_and_no_warm_query_at_same_revision() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let revision = store.settings_read_revision().await.unwrap();
    let (result, counts) = crate::pool_telemetry::measure_queries(async {
        assert!(
            !store
                .administrator_setting_present_at("fixture.absent", &revision)
                .await?
        );
        store
            .administrator_setting_present_at("fixture.absent", &revision)
            .await
    })
    .await;
    assert!(!result.unwrap());
    assert_eq!(counts.read.calls, 1);
    assert_eq!(counts.write.calls, 0);
}

#[tokio::test]
async fn raw_changes_missing_revision_and_store_identity_cannot_reuse_wrong_presence() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let other = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let first = store.settings_read_revision().await.unwrap();
    assert!(
        !store
            .administrator_setting_present_at("fixture.key", &first)
            .await
            .unwrap()
    );
    assert!(
        other
            .administrator_setting_present_at("fixture.key", &first)
            .await
            .is_err()
    );
    q("INSERT INTO setting_policy(key,global_value,allow_user_override,global_locked,updated_by,updated_at) VALUES('fixture.key',NULL,0,0,'raw',1)")
        .execute(&store.backend).await.unwrap();
    let next = store.settings_read_revision().await.unwrap();
    assert_ne!(first.number(), next.number());
    assert!(
        store
            .administrator_setting_present_at("fixture.key", &next)
            .await
            .unwrap()
    );
    q("DELETE FROM setting_policy WHERE key='fixture.key'")
        .execute(&store.backend)
        .await
        .unwrap();
    q("DELETE FROM settings_revision WHERE id=1")
        .execute(&store.backend)
        .await
        .unwrap();
    let none = store.settings_read_revision().await.unwrap();
    assert!(none.number().is_none());
    let (result, counts) = crate::pool_telemetry::measure_queries(async {
        assert!(
            !store
                .administrator_setting_present_at("fixture.key", &none)
                .await?
        );
        store
            .administrator_setting_present_at("fixture.key", &none)
            .await
    })
    .await;
    assert!(!result.unwrap());
    assert_eq!(counts.read.calls, 2);
}

#[tokio::test]
async fn only_original_administrator_scopes_activate_feature_guards() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    for scope in ["user", "domain", "profile", "group", "admin_user"] {
        q("DELETE FROM setting_value WHERE key='fixture.key'")
            .execute(&store.backend)
            .await
            .unwrap();
        q("INSERT INTO setting_value(scope_kind,scope_id,key,value,locked,updated_by,updated_at) VALUES(?1,'scope','fixture.key','false',0,'raw',1)")
            .bind(scope).execute(&store.backend).await.unwrap();
        let revision = store.settings_read_revision().await.unwrap();
        assert_eq!(
            store
                .administrator_setting_present_at("fixture.key", &revision)
                .await
                .unwrap(),
            scope != "user",
            "{scope}"
        );
    }
}
