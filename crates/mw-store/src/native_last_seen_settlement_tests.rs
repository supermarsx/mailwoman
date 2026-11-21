// Opaque source CAS; the final fresh fence remains the only admission proof.
#[tokio::test]
async fn native_last_seen_settlement_updates_only_telemetry_and_accepts_concurrent_touch() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (_, session) = seed(&store).await;
    let recent = (chrono::Utc::now() - chrono::Duration::seconds(60)).to_rfc3339();
    q("UPDATE sessions SET created_at=?2,activity_at=?2,last_seen='2000-01-01T00:00:00+00:00' WHERE id=?1")
        .bind(&session).bind(&recent).execute(&store.backend).await.unwrap();
    let before = store
        .native_authority_fence(&session, &["authority.test".into()])
        .await
        .unwrap()
        .unwrap();
    let now = chrono::Utc::now();
    assert!(
        store
            .settle_native_last_seen(&before, 15, 3600, 7200)
            .await
            .unwrap()
    );
    let after = store
        .native_authority_fence(&session, &["authority.test".into()])
        .await
        .unwrap()
        .unwrap();
    assert!(before.same_published_source(&after).unwrap());
    assert_eq!(
        before.clocks(),
        after.clocks(),
        "idle/absolute authority cannot be renewed"
    );
    let touched = chrono::DateTime::parse_from_rfc3339(after.last_seen().unwrap())
        .unwrap()
        .with_timezone(&chrono::Utc);
    assert!(touched >= now && touched <= chrono::Utc::now());
    // A stale due predicate can lose to another legitimate touch. Zero rows is
    // not an actor failure; the NEW source proves the original actor and touch.
    assert!(
        !store
            .settle_native_last_seen(&before, 15, 3600, 7200)
            .await
            .unwrap()
    );
    assert!(!store.session_touch_may_be_due_at(after.last_seen(), 15, now));
    let foreign = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    assert!(
        foreign
            .settle_native_last_seen(&before, 15, 3600, 7200)
            .await
            .is_err()
    );
    foreign.close().await;
    store.close().await;
}
#[tokio::test]
async fn native_last_seen_settlement_rejects_revoked_sources_and_never_recreates_deleted_session() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (account, session) = seed(&store).await;
    q("UPDATE sessions SET last_seen='2000-01-01T00:00:00+00:00' WHERE id=?1")
        .bind(&session)
        .execute(&store.backend)
        .await
        .unwrap();
    let old = store
        .native_authority_fence(&session, &["authority.test".into()])
        .await
        .unwrap()
        .unwrap();
    q("UPDATE accounts SET host='changed.authority.test' WHERE id=?1")
        .bind(&account)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        !store
            .settle_native_last_seen(&old, 15, 3600, 7200)
            .await
            .unwrap()
    );
    let current = store
        .native_authority_fence(&session, &["authority.test".into()])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(current.last_seen(), old.last_seen());
    assert!(!old.same_published_source(&current).unwrap());
    store.delete_session(&session).await.unwrap();
    assert!(
        !store
            .settle_native_last_seen(&current, 15, 3600, 7200)
            .await
            .unwrap()
    );
    assert!(
        store
            .native_authority_fence(&session, &["authority.test".into()])
            .await
            .unwrap()
            .is_none()
    );
    store.close().await;
}

#[tokio::test]
async fn native_last_seen_settlement_never_touches_expired_future_or_unbounded_clocks() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (_, session) = seed(&store).await;
    let past = "2000-01-01T00:00:00+00:00";
    let future = (chrono::Utc::now() + chrono::Duration::days(1)).to_rfc3339();
    let recent = (chrono::Utc::now() - chrono::Duration::seconds(1)).to_rfc3339();
    for (created, activity, idle, absolute) in [
        (recent.as_str(), past, 60, 3600),
        (past, recent.as_str(), 3600, 60),
        (future.as_str(), future.as_str(), 3600, 7200),
        ("invalid", recent.as_str(), 3600, 7200),
    ] {
        q("UPDATE sessions SET created_at=?2,activity_at=?3,last_seen=?4 WHERE id=?1")
            .bind(&session)
            .bind(created)
            .bind(activity)
            .bind(past)
            .execute(&store.backend)
            .await
            .unwrap();
        let source = store
            .native_authority_fence(&session, &["authority.test".into()])
            .await
            .unwrap()
            .unwrap();
        assert!(
            !store
                .settle_native_last_seen(&source, 15, idle, absolute)
                .await
                .unwrap()
        );
        let after = store
            .native_authority_fence(&session, &["authority.test".into()])
            .await
            .unwrap()
            .unwrap();
        assert_eq!(after.last_seen(), Some(past));
        assert_eq!(after.clocks(), source.clocks());
    }
    let source = store
        .native_authority_fence(&session, &["authority.test".into()])
        .await
        .unwrap()
        .unwrap();
    assert!(
        store
            .settle_native_last_seen(&source, i64::MAX, 3600, 7200)
            .await
            .is_err()
    );
    assert!(
        store
            .settle_native_last_seen(&source, 15, u64::MAX, 0)
            .await
            .is_err()
    );
    assert!(
        store
            .settle_native_last_seen(&source, 0, 3600, 7200)
            .await
            .is_err()
    );
    store.close().await;
}
