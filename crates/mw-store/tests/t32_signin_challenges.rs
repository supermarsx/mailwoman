use mw_store::{ServerKey, Store};
async fn exercise(store: Store) {
    let peer = "a".repeat(64);
    assert!(
        store
            .issue_signin_challenge("once", "mailbox", &peer, 100, 160)
            .await
            .unwrap()
    );
    assert!(
        !store
            .consume_signin_challenge("once", "admin", &peer, 101)
            .await
            .unwrap()
    );
    assert!(
        !store
            .consume_signin_challenge("once", "mailbox", &"b".repeat(64), 101)
            .await
            .unwrap()
    );
    let (a, b) = tokio::join!(
        store.consume_signin_challenge("once", "mailbox", &peer, 101),
        store.consume_signin_challenge("once", "mailbox", &peer, 101)
    );
    assert_eq!(u8::from(a.unwrap()) + u8::from(b.unwrap()), 1);
    assert!(
        !store
            .consume_signin_challenge("once", "mailbox", &peer, 102)
            .await
            .unwrap()
    );
    assert!(
        store
            .issue_signin_challenge("expiry", "mailbox", &peer, 100, 160)
            .await
            .unwrap()
    );
    assert!(
        !store
            .consume_signin_challenge("expiry", "mailbox", &peer, 160)
            .await
            .unwrap()
    );
    for n in 0..8 {
        assert!(
            store
                .issue_signin_challenge(&format!("limit{n}"), "admin", &peer, 160, 220)
                .await
                .unwrap()
        );
    }
    assert!(
        !store
            .issue_signin_challenge("ninth", "mailbox", &peer, 160, 220)
            .await
            .unwrap()
    );
    assert!(
        store
            .issue_signin_challenge("after-expiry", "mailbox", &peer, 220, 280)
            .await
            .unwrap()
    );
    assert!(
        !store
            .issue_signin_challenge("too-long", "mailbox", &peer, 220, 821)
            .await
            .unwrap()
    );
}
#[tokio::test]
async fn nonce_single_use_scope_expiry_and_capacity_sqlite() {
    exercise(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn nonce_single_use_scope_expiry_and_capacity_postgres() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        return;
    };
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    let name = format!("mw_challenge_{:032x}", rand::random::<u128>());
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
