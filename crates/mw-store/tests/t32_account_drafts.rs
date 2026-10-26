use mw_store::{
    ServerKey, Store,
    account_drafts::{DraftClaim, DraftDiscard, DraftSave},
};
#[path = "../src/test_db.rs"]
mod test_db;
fn input<'a>(owner: &'a str, operation: &'a str, revision: Option<&'a str>) -> DraftSave<'a> {
    DraftSave {
        owner,
        account_id: "wire-1",
        id: "draft-1",
        expected_revision: revision,
        operation_id: operation,
        state: b"private body and partial recipient",
        summary: b"private subject",
        now: 1,
    }
}
async fn exercise(store: Store, key: ServerKey) {
    let a = input("realm-one", "operation-a", None);
    let b = input("realm-one", "operation-b", None);
    let (a, b) = tokio::join!(store.claim_account_draft(&a), store.claim_account_draft(&b));
    let winner = if matches!(a.unwrap(), DraftClaim::Claimed) {
        assert!(matches!(b.unwrap(), DraftClaim::Uncertain));
        "operation-a"
    } else {
        assert!(matches!(b.unwrap(), DraftClaim::Claimed));
        "operation-b"
    };
    assert!(
        store
            .account_draft("realm-two", "draft-1")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        !store
            .rebind_account_draft_state("realm-two", "draft-1", winner, b"stolen")
            .await
            .unwrap()
    );
    assert!(
        store
            .rebind_account_draft_state(
                "realm-one",
                "draft-1",
                winner,
                b"message-owned attachment references"
            )
            .await
            .unwrap()
    );
    let saved = store
        .complete_account_draft("realm-one", "draft-1", winner, "upstream-email", 2)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.state, b"message-owned attachment references");
    assert!(matches!(
        store
            .claim_account_draft_discard("realm-one", "draft-1", Some("stale"))
            .await
            .unwrap(),
        DraftDiscard::Conflict
    ));
    let next = input("realm-one", "next", saved.revision.as_deref());
    assert!(matches!(
        store.claim_account_draft(&next).await.unwrap(),
        DraftClaim::Claimed
    ));
    store
        .uncertain_account_draft("realm-one", "draft-1", "next")
        .await
        .unwrap();
    assert!(matches!(
        store.claim_account_draft(&next).await.unwrap(),
        DraftClaim::Uncertain
    ));
    let row = store
        .account_draft("realm-one", "draft-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.email_id.as_deref(), Some("upstream-email"));
    assert_eq!(row.state, b"private body and partial recipient");
    assert!(matches!(
        store
            .claim_account_draft_discard("realm-one", "draft-1", saved.revision.as_deref())
            .await
            .unwrap(),
        DraftDiscard::Uncertain
    ));
    let disposable = input("discard-owner", "save", None);
    assert!(matches!(
        store.claim_account_draft(&disposable).await.unwrap(),
        DraftClaim::Claimed
    ));
    let disposable = store
        .complete_account_draft("discard-owner", "draft-1", "save", "discard-email", 3)
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(
        store
            .claim_account_draft_discard("discard-owner", "draft-1", disposable.revision.as_deref())
            .await
            .unwrap(),
        DraftDiscard::Ready(_)
    ));
    assert!(matches!(
        store
            .claim_account_draft(&input(
                "discard-owner",
                "racing-save",
                disposable.revision.as_deref()
            ))
            .await
            .unwrap(),
        DraftClaim::CleanupPending
    ));
    assert!(
        !store
            .account_draft_cleanup_done("discard-owner", "draft-1", "discard-email")
            .await
            .unwrap(),
        "old-copy cleanup cannot release a reserved discard"
    );
    assert!(
        !store
            .complete_account_draft_discard(
                "other-owner",
                "draft-1",
                disposable.revision.as_deref().unwrap(),
                "discard-email"
            )
            .await
            .unwrap()
    );
    assert!(
        store
            .complete_account_draft_discard(
                "discard-owner",
                "draft-1",
                disposable.revision.as_deref().unwrap(),
                "discard-email"
            )
            .await
            .unwrap()
    );
    assert!(
        store
            .account_draft("discard-owner", "draft-1")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        matches!(
            store
                .claim_account_draft(&input("discard-owner", "save", None))
                .await
                .unwrap(),
            DraftClaim::Conflict
        ),
        "a delayed initial save cannot resurrect a discarded logical ID"
    );
    for index in 0..100 {
        let id = format!("quota-{index}");
        let mut claim = input("quota-owner", "save", None);
        claim.id = &id;
        assert!(matches!(
            store.claim_account_draft(&claim).await.unwrap(),
            DraftClaim::Claimed
        ));
        store
            .refuse_account_draft_before_append("quota-owner", &id, "save")
            .await
            .unwrap();
    }
    assert_eq!(
        store.account_drafts("quota-owner").await.unwrap().len(),
        100
    );
    let overflow = input("quota-owner", "overflow", None);
    assert!(matches!(
        store.claim_account_draft(&overflow).await.unwrap(),
        DraftClaim::Limit
    ));
    assert!(matches!(
        store
            .claim_account_draft_discard("quota-owner", "quota-0", None)
            .await
            .unwrap(),
        DraftDiscard::Gone
    ));
    assert_eq!(store.account_drafts("quota-owner").await.unwrap().len(), 99);
    assert!(matches!(
        store.claim_account_draft(&overflow).await.unwrap(),
        DraftClaim::Claimed
    ));
    // Copy a populated sealed pending claim with the same server key. Neither the
    // body nor its uncertainty is lost, and migration never retries the operation.
    let path = test_db::unique_file_path("draft-copy", "source.sqlite");
    let source = Store::open(path.to_str().unwrap(), key).await.unwrap();
    source
        .claim_account_draft(&input("copy-owner", "copy-operation", None))
        .await
        .unwrap();
    store
        .migrate_from_sqlite(path.to_str().unwrap())
        .await
        .unwrap();
    let copied = store
        .account_draft("copy-owner", "draft-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(copied.state, b"private body and partial recipient");
    assert_eq!(copied.status, "uncertain");
    assert!(matches!(
        store
            .claim_account_draft(&input("copy-owner", "copy-operation", None))
            .await
            .unwrap(),
        DraftClaim::Uncertain
    ));
}
#[tokio::test]
async fn sqlite_claim_rebind_and_populated_copy() {
    let key = ServerKey::generate();
    exercise(Store::open_in_memory(key.clone()).await.unwrap(), key).await;
}
#[tokio::test]
async fn postgres_claim_rebind_and_populated_copy() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        return;
    };
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    let name = format!("mw_draft_{:032x}", rand::random::<u128>());
    sqlx::query(&format!("CREATE DATABASE {name}"))
        .execute(&admin)
        .await
        .unwrap();
    let mut url = url::Url::parse(&dsn).unwrap();
    url.set_path(&format!("/{name}"));
    let key = ServerKey::generate();
    exercise(Store::open(url.as_str(), key.clone()).await.unwrap(), key).await;
    sqlx::query(&format!("DROP DATABASE {name} WITH (FORCE)"))
        .execute(&admin)
        .await
        .unwrap();
}
