// Included by authority_publication::tests, reusing its real SQLite and
// UUID-schema PostgreSQL fixtures. No runtime is claimed by authored source.
const GUARD_KEYS: [&str; 3] = [
    "security.inbound-allowlist.v1",
    "security.clientAccess.v1",
    "maintenance.mode.v1",
];
async fn guard_stamp(s: &Store) -> AuthorityStamp {
    current(
        s,
        &AuthorityScope::new(AuthorityKind::Protection, "").unwrap(),
    )
    .await
}
async fn guard_snapshot(s: &Store, cookie: &str) -> NativeGuardInputSnapshot {
    s.native_guard_input_source_snapshot(cookie, &["authority.test".into()], &[])
        .await
        .unwrap()
        .expect("complete native source")
}
async fn guard_key_coverage(s: Store) {
    let (_account, cookie) = seed(&s).await;
    for key in GUARD_KEYS {
        let start = guard_stamp(&s).await;
        q("INSERT INTO settings(key,value) VALUES(?1,'{}')")
            .bind(key)
            .execute(&s.backend)
            .await
            .unwrap();
        let inserted = guard_stamp(&s).await;
        assert!(inserted.version > start.version);
        q("UPDATE settings SET value=value WHERE key=?1")
            .bind(key)
            .execute(&s.backend)
            .await
            .unwrap();
        assert_eq!(
            guard_stamp(&s).await,
            inserted,
            "no-op cannot dirty publication"
        );
        let source = guard_snapshot(&s, &cookie).await;
        q("UPDATE settings SET value='changed raw config' WHERE key=?1")
            .bind(key)
            .execute(&s.backend)
            .await
            .unwrap();
        assert!(
            !source
                .same_source(
                    &s,
                    &s.native_publication_fence(&cookie, &["authority.test".into()], &[])
                        .await
                        .unwrap()
                        .unwrap()
                )
                .unwrap()
        );
        let changed = guard_stamp(&s).await;
        q("UPDATE settings SET key='fixture:unrelated' WHERE key=?1")
            .bind(key)
            .execute(&s.backend)
            .await
            .unwrap();
        assert!(
            guard_stamp(&s).await.version > changed.version,
            "rename away is covered"
        );
        let away = guard_stamp(&s).await;
        q("UPDATE settings SET key=?1 WHERE key='fixture:unrelated'")
            .bind(key)
            .execute(&s.backend)
            .await
            .unwrap();
        assert!(
            guard_stamp(&s).await.version > away.version,
            "rename into exact key is covered"
        );
        let renamed = guard_stamp(&s).await;
        q("DELETE FROM settings WHERE key=?1")
            .bind(key)
            .execute(&s.backend)
            .await
            .unwrap();
        assert!(guard_stamp(&s).await.version > renamed.version);
    }
    let telemetry = guard_stamp(&s).await;
    q("INSERT INTO settings(key,value) VALUES('telemetry:guard-fixture','1')")
        .execute(&s.backend)
        .await
        .unwrap();
    q("UPDATE settings SET value='2' WHERE key='telemetry:guard-fixture'")
        .execute(&s.backend)
        .await
        .unwrap();
    q("DELETE FROM settings WHERE key='telemetry:guard-fixture'")
        .execute(&s.backend)
        .await
        .unwrap();
    s.touch_session(&cookie).await.unwrap();
    assert_eq!(guard_stamp(&s).await, telemetry);
    let row=q("SELECT target_version,incarnation FROM authority_publication_outbox WHERE scope_kind='protection' AND scope_id=''").fetch_one(&s.backend).await.unwrap();
    assert_eq!(row.get_i64("target_version"), telemetry.version);
    assert_eq!(row.get_string("incarnation"), telemetry.incarnation);
    s.close().await;
}
async fn guard_deferred_source_and_payload_bounds(s: Store) {
    let (_account, cookie) = seed(&s).await;
    s.set_setting(GUARD_KEYS[0], "{}").await.unwrap();
    s.set_setting(GUARD_KEYS[1], "[malformed later JSON")
        .await
        .unwrap();
    let source = guard_snapshot(&s, &cookie).await;
    assert_eq!(
        source.raw(&s, GuardInputSection::InboundAllowlist).unwrap(),
        Some("{}")
    );
    assert_eq!(
        source.raw(&s, GuardInputSection::ClientAccess).unwrap(),
        Some("[malformed later JSON"),
        "owning guard parses later; source never invents defaults"
    );
    assert_eq!(
        source.raw(&s, GuardInputSection::Maintenance).unwrap(),
        None
    );
    let foreign = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    assert!(
        source
            .raw(&foreign, GuardInputSection::InboundAllowlist)
            .is_err()
    );
    let before = s
        .native_publication_fence(&cookie, &["authority.test".into()], &[])
        .await
        .unwrap()
        .unwrap();
    if s.backend.dialect() == Dialect::Sqlite {
        q("UPDATE settings SET value=CAST(X'7b7d' AS BLOB) WHERE key=?1")
            .bind(GUARD_KEYS[1])
            .execute(&s.backend)
            .await
            .unwrap();
        let corrupt = guard_snapshot(&s, &cookie).await;
        assert!(corrupt.raw(&s, GuardInputSection::ClientAccess).is_err());
        assert_eq!(
            corrupt
                .raw(&s, GuardInputSection::InboundAllowlist)
                .unwrap(),
            Some("{}")
        );
    }
    s.set_setting(GUARD_KEYS[1], &"x".repeat(1024 * 1024 + 1))
        .await
        .unwrap();
    let rows = q(super::guard_inputs::guard_source_sql_for_test(
        s.backend.dialect(),
    ))
    .bind(&cookie)
    .bind(before.account_id())
    .bind(before.principal_id())
    .bind(before.binding_id())
    .fetch_all(&s.backend)
    .await
    .unwrap();
    assert_eq!(rows.len(), 4);
    for row in rows {
        assert_eq!(row.get_i64("invalid"), 1);
        assert!(
            row.try_get_opt_string("value").unwrap().is_none(),
            "SQL must mask payload before driver transfer"
        );
    }
    let over = guard_snapshot(&s, &cookie).await;
    assert!(over.raw(&s, GuardInputSection::InboundAllowlist).is_err());
    assert!(over.raw(&s, GuardInputSection::ClientAccess).is_err());
    assert!(!format!("{over:?}").contains(&cookie));
    s.delete_session(&cookie).await.unwrap();
    assert!(
        s.native_guard_input_source_snapshot(&cookie, &["authority.test".into()], &[])
            .await
            .unwrap()
            .is_none()
    );
    foreign.close().await;
    s.close().await;
}
async fn guard_schema_missing_or_replaced_is_not_eligible(s: Store) {
    let (_account, cookie) = seed(&s).await;
    let source = guard_snapshot(&s, &cookie).await;
    assert!(
        s.validate_guard_input_publication_schema(source.source())
            .await
            .unwrap()
    );
    match s.backend.dialect() {
        Dialect::Sqlite => {
            q("DROP TRIGGER authority_guard_config_update")
                .execute(&s.backend)
                .await
                .unwrap();
        }
        Dialect::Postgres => {
            q("ALTER TABLE settings DISABLE TRIGGER authority_guard_config")
                .execute(&s.backend)
                .await
                .unwrap();
        }
    }
    assert!(
        !s.validate_guard_input_publication_schema(source.source())
            .await
            .unwrap()
    );
    assert!(
        s.native_guard_input_source_snapshot(&cookie, &["authority.test".into()], &[])
            .await
            .unwrap()
            .is_none()
    );
    s.close().await;
}
async fn guard_rollback_and_mid_build_writer(s: Store) {
    let (_account, cookie) = seed(&s).await;
    s.set_setting(GUARD_KEYS[0], "{}").await.unwrap();
    let start = guard_stamp(&s).await;
    let mut tx = s.backend.begin().await.unwrap();
    q("UPDATE settings SET value='rolled back' WHERE key=?1")
        .bind(GUARD_KEYS[0])
        .execute_tx(&mut tx)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    assert_eq!(guard_stamp(&s).await, start);
    assert_eq!(
        s.get_setting(GUARD_KEYS[0]).await.unwrap(),
        Some("{}".into())
    );
    let reject = match s.backend.dialect() {
        Dialect::Sqlite => {
            "CREATE TRIGGER guard_fixture_reject BEFORE UPDATE ON authority_publication_outbox WHEN NEW.scope_kind='protection' BEGIN SELECT RAISE(ABORT,'owned guard fixture'); END"
        }
        Dialect::Postgres => {
            "CREATE FUNCTION guard_fixture_reject() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.scope_kind='protection' THEN RAISE EXCEPTION 'owned guard fixture'; END IF; RETURN NEW; END; $$"
        }
    };
    q(reject).execute(&s.backend).await.unwrap();
    if s.backend.dialect() == Dialect::Postgres {
        q("CREATE TRIGGER guard_fixture_reject BEFORE UPDATE ON authority_publication_outbox FOR EACH ROW EXECUTE FUNCTION guard_fixture_reject()").execute(&s.backend).await.unwrap();
    }
    assert!(s.set_setting(GUARD_KEYS[0], "must rollback").await.is_err());
    assert_eq!(guard_stamp(&s).await, start);
    assert_eq!(
        s.get_setting(GUARD_KEYS[0]).await.unwrap(),
        Some("{}".into())
    );
    q(match s.backend.dialect() {
        Dialect::Sqlite => "DROP TRIGGER guard_fixture_reject",
        Dialect::Postgres => "DROP TRIGGER guard_fixture_reject ON authority_publication_outbox",
    })
    .execute(&s.backend)
    .await
    .unwrap();
    let pause = std::sync::Arc::new(super::guard_inputs::testing::Pause {
        entered: tokio::sync::Notify::new(),
        release: tokio::sync::Notify::new(),
    });
    let domains = vec!["authority.test".into()];
    let future = super::guard_inputs::testing::PAUSE.scope(
        pause.clone(),
        s.native_guard_input_source_snapshot(&cookie, &domains, &[]),
    );
    tokio::pin!(future);
    tokio::select! {_=pause.entered.notified()=>{},_=&mut future=>panic!("source reader did not reach after-source boundary")}
    // A separate transaction on the pool changes authoritative raw source after
    // its payload SELECT and before its AFTER fence. It cannot be installed.
    s.set_setting(GUARD_KEYS[0], "independent transaction")
        .await
        .unwrap();
    pause.release.notify_one();
    assert!(matches!(future.await, Err(StoreError::SourceChanged)));
    s.close().await;
}
#[tokio::test]
async fn sqlite_guard_exact_raw_key_epochs_and_hot_telemetry() {
    guard_key_coverage(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn postgres_guard_exact_raw_key_epochs_and_hot_telemetry() {
    postgres(guard_key_coverage).await;
}
#[tokio::test]
async fn sqlite_guard_deferred_owner_sections_and_driver_payload_bounds() {
    guard_deferred_source_and_payload_bounds(
        Store::open_in_memory(ServerKey::generate()).await.unwrap(),
    )
    .await;
}
#[tokio::test]
async fn postgres_guard_deferred_owner_sections_and_driver_payload_bounds() {
    postgres(guard_deferred_source_and_payload_bounds).await;
}
#[tokio::test]
async fn sqlite_guard_schema_missing_coverage_falls_back() {
    guard_schema_missing_or_replaced_is_not_eligible(
        Store::open_in_memory(ServerKey::generate()).await.unwrap(),
    )
    .await;
}
#[tokio::test]
async fn postgres_guard_schema_disabled_coverage_falls_back() {
    postgres(guard_schema_missing_or_replaced_is_not_eligible).await;
}
#[tokio::test]
async fn sqlite_guard_rollback_outbox_failure_and_mid_build_writer() {
    guard_rollback_and_mid_build_writer(
        Store::open_in_memory(ServerKey::generate()).await.unwrap(),
    )
    .await;
}
#[tokio::test]
async fn postgres_guard_rollback_outbox_failure_and_mid_build_writer() {
    postgres(guard_rollback_and_mid_build_writer).await;
}

#[tokio::test]
async fn sqlite_guard_separate_store_writer_and_restart_cannot_reuse_old_source() {
    let path = std::env::temp_dir().join(format!("mw-guard-source-{}.db", uuid::Uuid::new_v4()));
    let key = ServerKey::generate();
    let writer = Store::open(path.to_str().unwrap(), key.clone())
        .await
        .unwrap();
    let (_account, cookie) = seed(&writer).await;
    writer.set_setting(GUARD_KEYS[0], "{}").await.unwrap();
    let reader = Store::open(path.to_str().unwrap(), key.clone())
        .await
        .unwrap();
    let source = guard_snapshot(&reader, &cookie).await;
    q("UPDATE settings SET value='independent raw writer' WHERE key=?1")
        .bind(GUARD_KEYS[0])
        .execute(&writer.backend)
        .await
        .unwrap();
    let current = reader
        .native_publication_fence(&cookie, &["authority.test".into()], &[])
        .await
        .unwrap()
        .unwrap();
    assert!(!source.same_source(&reader, &current).unwrap());
    assert_eq!(
        guard_snapshot(&reader, &cookie)
            .await
            .raw(&reader, GuardInputSection::InboundAllowlist)
            .unwrap(),
        Some("independent raw writer")
    );
    writer.close().await;
    reader.close().await;
    let reopened = Store::open(path.to_str().unwrap(), key).await.unwrap();
    let after = reopened
        .native_publication_fence(&cookie, &["authority.test".into()], &[])
        .await
        .unwrap()
        .unwrap();
    assert!(
        source.same_source(&reopened, &after).is_err(),
        "old process Store scope is not reconstructed from persisted epochs"
    );
    reopened.close().await;
}
