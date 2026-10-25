use mw_store::{ServerKey, Store};

async fn exercise(store: Store) {
    let id = format!("role-description-{:032x}", rand::random::<u128>());
    let caps = vec!["users.read".into()];
    store
        .put_admin_role_description(
            &id,
            &id,
            &caps,
            Some("Mailbox support — no account changes"),
        )
        .await
        .unwrap();
    let value = store.get_admin_role(&id).await.unwrap().unwrap();
    assert_eq!(value.description, "Mailbox support — no account changes");
    assert_eq!(value.capabilities, caps);
    store.put_admin_role(&id, &id, &caps).await.unwrap();
    assert_eq!(
        store
            .get_admin_role(&id)
            .await
            .unwrap()
            .unwrap()
            .description,
        value.description,
        "old clients preserve the description"
    );
    store
        .put_admin_role_description(&id, &id, &caps, Some(""))
        .await
        .unwrap();
    assert!(
        store
            .get_admin_role(&id)
            .await
            .unwrap()
            .unwrap()
            .description
            .is_empty()
    );
    store.delete_admin_role(&id).await.unwrap();
}
#[tokio::test]
async fn role_description_sqlite() {
    exercise(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn role_description_postgres() {
    if let Ok(url) = std::env::var("MW_TEST_PG") {
        exercise(Store::open(&url, ServerKey::generate()).await.unwrap()).await;
    }
}
