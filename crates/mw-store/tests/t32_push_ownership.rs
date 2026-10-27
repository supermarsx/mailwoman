use mw_store::{PushSubscriptionRow, ServerKey, Store, StoreError};
async fn exercise(store: Store) {
    let row = |account: &str| PushSubscriptionRow {
        id: format!("{account}-sub"),
        account_id: account.into(),
        transport: "unifiedpush".into(),
        endpoint: "https://push.example/owned".into(),
        p256dh: None,
        auth: None,
        app_id: None,
        expires_at: None,
        created_at: "2026-01-01T00:00:00Z".into(),
        last_wake_at: None,
    };
    let a = row("a");
    let b = row("b");
    let (left, right) = tokio::join!(
        store.upsert_push_subscription(&a),
        store.upsert_push_subscription(&b)
    );
    assert_ne!(left.is_ok(), right.is_ok());
    assert!(matches!(left, Ok(()) | Err(StoreError::NotFound)));
    assert!(matches!(right, Ok(()) | Err(StoreError::NotFound)));
    let (owner, foreign) = if left.is_ok() { ("a", "b") } else { ("b", "a") };
    let saved = store
        .list_push_subscriptions(owner)
        .await
        .unwrap()
        .remove(0);
    store
        .delete_account_push_subscription(foreign, Some(&saved.endpoint), None)
        .await
        .unwrap();
    store
        .delete_account_push_subscription(foreign, None, Some(&saved.id))
        .await
        .unwrap();
    assert_eq!(store.list_push_subscriptions(owner).await.unwrap().len(), 1);
    store
        .delete_account_push_subscription(owner, None, Some(&saved.id))
        .await
        .unwrap();
    assert!(
        store
            .list_push_subscriptions(owner)
            .await
            .unwrap()
            .is_empty()
    );
    // A stale in-flight delivery's Gone response must not delete a replacement
    // endpoint registration, even when the original account registers it again.
    for replacement_owner in [foreign, owner] {
        let mut replacement = row(replacement_owner);
        replacement.id = format!("{replacement_owner}-replacement");
        store.upsert_push_subscription(&replacement).await.unwrap();
        store
            .delete_account_push_subscription(owner, None, Some(&saved.id))
            .await
            .unwrap();
        let remaining = store
            .list_push_subscriptions(replacement_owner)
            .await
            .unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, replacement.id);
        store
            .delete_account_push_subscription(replacement_owner, None, Some(&replacement.id))
            .await
            .unwrap();
    }
}
#[tokio::test]
async fn push_account_ownership_sqlite() {
    exercise(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn push_account_ownership_postgres() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        return;
    };
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    let name = format!("mw_push_{:032x}", rand::random::<u128>());
    sqlx::query(&format!("CREATE DATABASE {name}"))
        .execute(&admin)
        .await
        .unwrap();
    let mut url = url::Url::parse(&dsn).unwrap();
    url.set_path(&format!("/{name}"));
    exercise(
        Store::open(url.as_str(), ServerKey::generate())
            .await
            .unwrap(),
    )
    .await;
    sqlx::query(&format!("DROP DATABASE {name} WITH (FORCE)"))
        .execute(&admin)
        .await
        .unwrap();
}
