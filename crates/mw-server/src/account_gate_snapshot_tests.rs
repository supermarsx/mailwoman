use super::*;

fn domain(name: &str) -> mw_store::DomainConfigRow {
    mw_store::DomainConfigRow {
        name: name.into(),
        upstream_json: "unparsed".into(),
        allowlist_json: "unparsed".into(),
        blocklist_json: "[]".into(),
        enabled: true,
        preferred: false,
        settings_json: "{}".into(),
        updated_at: 1,
    }
}
fn session(name: &str) -> Session {
    Session {
        id: String::new(),
        account_id: "acct".into(),
        username: name.into(),
        jmap_url: String::new(),
        api_url: String::new(),
        credentials: mw_store::Credentials {
            username: name.into(),
            password: String::new(),
        },
    }
}
async fn oracle(store: &Store, session: &Session) -> Result<AccountGate, StoreError> {
    let names = session_names(store, session).await?;
    gate_for_names(store, &session.account_id, &names).await
}

#[tokio::test]
async fn final_session_gate_fuses_five_fresh_reads_to_two_and_matches_oracle() {
    let store = Store::open_in_memory(mw_store::ServerKey::generate())
        .await
        .unwrap();
    store
        .put_domain_config(&domain("example.test"))
        .await
        .unwrap();
    store
        .put_domain_alias("alias.test", "example.test")
        .await
        .unwrap();
    store
        .set_preferred_domain(Some("example.test"))
        .await
        .unwrap();
    let current = session("Alice+tag@example.test");
    oracle(&store, &current).await.unwrap();
    let (old, old_reads) =
        mw_store::pool_telemetry::measure_queries(oracle(&store, &current)).await;
    let (new, new_reads) =
        mw_store::pool_telemetry::measure_queries(for_session(&store, &current)).await;
    assert_eq!(new.unwrap(), old.unwrap());
    assert_eq!(old_reads.read.calls, 5);
    assert_eq!(new_reads.read.calls, 2);
    write_flags(
        &store,
        "alice@example.test",
        UserFeatureFlags {
            force_password_change: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    for name in [
        "Alice+tag@example.test",
        "Alice@alias.test",
        "alice",
        "NÉW@BÜCHER.TEST",
        "user@xn--bcher-kva.test",
    ] {
        let current = session(name);
        assert_eq!(
            oracle(&store, &current).await.unwrap(),
            for_session(&store, &current).await.unwrap(),
            "{name}"
        );
    }
    assert!(
        for_session(&store, &session("Alice+tag@example.test"))
            .await
            .unwrap()
            .password_change_required
    );
}

#[tokio::test]
async fn final_gate_never_reuses_an_earlier_domain_policy_or_flag_admission() {
    let dir = std::env::temp_dir().join(format!("mw-final-gate-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("gate.db");
    let key = mw_store::ServerKey::generate();
    let reader = Store::open(path.to_str().unwrap(), key.clone())
        .await
        .unwrap();
    let writer = Store::open(path.to_str().unwrap(), key).await.unwrap();
    writer
        .put_domain_config(&domain("example.test"))
        .await
        .unwrap();
    writer
        .put_domain_config(&domain("other.test"))
        .await
        .unwrap();
    writer
        .put_domain_alias("alias.test", "example.test")
        .await
        .unwrap();
    let current = session("alice@alias.test");
    oracle(&reader, &current).await.unwrap();
    assert!(!for_session(&reader, &current).await.unwrap().disabled);
    // This is the earlier domain-upgrade check, followed by a new independent
    // write before final admission. No earlier proof is passed to the gate.
    crate::domain_config::identity_snapshot(&reader, &current.username)
        .await
        .unwrap();
    let mut disabled = domain("example.test");
    disabled.enabled = false;
    writer.put_domain_config(&disabled).await.unwrap();
    assert!(for_session(&reader, &current).await.unwrap().disabled);
    writer
        .put_domain_config(&domain("example.test"))
        .await
        .unwrap();
    writer
        .delete_domain_alias("alias.test", "example.test")
        .await
        .unwrap();
    writer
        .put_domain_alias("alias.test", "other.test")
        .await
        .unwrap();
    write_flags(
        &writer,
        "alice@alias.test",
        UserFeatureFlags {
            force_password_change: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(
        for_session(&reader, &current)
            .await
            .unwrap()
            .password_change_required
    );
    writer
        .set_setting(&flags_key("Alice@ALIAS.TEST"), "ignored corrupt legacy row")
        .await
        .unwrap();
    assert!(
        for_session(&reader, &session("Alice@ALIAS.TEST"))
            .await
            .unwrap()
            .password_change_required
    );
    writer
        .set_setting(&flags_key("alice@alias.test"), "corrupt normalized row")
        .await
        .unwrap();
    assert!(for_session(&reader, &current).await.is_err());
    writer
        .set_setting(&flags_key("alice@alias.test"), "{}")
        .await
        .unwrap();
    writer
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "signin.domainMode".into(),
            global_value: Some("\"listed\"".into()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    assert!(
        for_session(&reader, &session("clear@unlisted.test"))
            .await
            .unwrap()
            .disabled
    );
    assert!(!for_session(&reader, &current).await.unwrap().disabled);
    writer
        .put_provisioned_user("alice@other.test", "import", None, None, "test", 1)
        .await
        .unwrap();
    let before = writer
        .user_lifecycle("alice@other.test")
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(
        writer
            .change_user_lifecycle(&before, "disabled", "test", 2)
            .await
            .unwrap(),
        mw_store::user_lifecycle::LifecycleChange::Applied { .. }
    ));
    assert!(for_session(&reader, &current).await.unwrap().disabled);
    reader.close().await;
    writer.close().await;
    drop(reader);
    drop(writer);
    for attempt in 0..100 {
        match std::fs::remove_dir_all(&dir) {
            Ok(()) => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(_) if attempt < 99 => {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await
            }
            Err(error) => panic!("fixture directory did not close: {error}"),
        }
    }
}

#[tokio::test]
async fn cold_and_unsupported_inputs_retain_original_reader_and_error_order() {
    let store = Store::open_in_memory(mw_store::ServerKey::generate())
        .await
        .unwrap();
    store
        .put_domain_config(&domain("example.test"))
        .await
        .unwrap();
    assert!(
        try_gate(&store, &["acct"], &["alice@example.test"])
            .await
            .unwrap()
            .is_none(),
        "cold policy must restart through original reader"
    );
    oracle(&store, &session("alice@example.test"))
        .await
        .unwrap();
    assert!(
        try_gate(&store, &["acct"], &["alice@example.test"])
            .await
            .unwrap()
            .is_some()
    );
    for name in [
        "alice",
        "NÉW@BÜCHER.TEST",
        "alice@xn--bcher-kva.test",
        "bad@@example.test",
    ] {
        assert!(Prepared::new(&["acct"], &[name]).is_none());
    }
    let huge = "x".repeat(1025);
    assert!(Prepared::new(&[&huge], &["alice@example.test"]).is_none());
    let many = (0..257)
        .map(|n| format!("user{n}@example.test"))
        .collect::<Vec<_>>();
    assert!(
        Prepared::new(
            &["acct"],
            &many.iter().map(String::as_str).collect::<Vec<_>>()
        )
        .is_none()
    );
    let boundary = Prepared::new(
        &["acct"],
        &many[..255].iter().map(String::as_str).collect::<Vec<_>>(),
    )
    .unwrap();
    assert_eq!(boundary.keys.len(), 256);
    // Validated domain APIs prevent chains. The Store contract separately
    // exercises a raw legacy shadow/chain; this checks its ineligible result.
    let prepared = Prepared::new(&["acct"], &["alice@example.test"]).unwrap();
    let mut chained = store
        .final_account_gate_snapshot(&prepared.domains, &prepared.keys)
        .await
        .unwrap();
    chained
        .domains
        .get_mut("example.test")
        .unwrap()
        .as_mut()
        .unwrap()
        .second_alias = true;
    assert!(finish_gate(prepared, chained, false).unwrap().is_none());
    let mut disabled = domain("example.test");
    disabled.enabled = false;
    store.put_domain_config(&disabled).await.unwrap();
    store
        .remember_account_names("acct", &["bad@@example.test"])
        .await
        .unwrap();
    assert!(
        for_session(&store, &session("alice@example.test"))
            .await
            .is_err(),
        "disabled first domain must not hide later malformed name"
    );
}

#[tokio::test]
async fn held_rows_keep_database_order_deferred_errors_and_global_cap() {
    let store = Store::open_in_memory(mw_store::ServerKey::generate())
        .await
        .unwrap();
    let prepare = || Prepared::new(&["acct"], &["a@example.test"]).unwrap();
    let mut snapshot = store
        .final_account_gate_snapshot(&["example.test".into()], &[])
        .await
        .unwrap();
    snapshot.held_addresses = vec![Ok("a@example.test".into()), Ok("z@bad/path".into())];
    assert!(
        finish_gate(prepare(), snapshot, false)
            .unwrap()
            .unwrap()
            .disabled,
        "first match suppresses later malformed hold exactly as original reader"
    );
    let mut snapshot = store
        .final_account_gate_snapshot(&["example.test".into()], &[])
        .await
        .unwrap();
    snapshot.held_addresses = vec![Ok("0@bad/path".into()), Ok("a@example.test".into())];
    assert!(finish_gate(prepare(), snapshot, false).is_err());
    let mut snapshot = store
        .final_account_gate_snapshot(&["example.test".into()], &[])
        .await
        .unwrap();
    snapshot.held_addresses = (0..5001).map(|_| Ok("a@example.test".into())).collect();
    assert!(
        finish_gate(prepare(), snapshot, false).is_err(),
        "5001 fails before early matching hold"
    );
}
