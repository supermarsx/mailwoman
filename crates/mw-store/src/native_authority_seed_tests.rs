// Source-authored contracts; the parent controls actual execution/PG fixture.
async fn publication_source(
    s: &Store,
    session: &str,
) -> crate::native_admission_snapshot::NativeAdmissionSnapshot {
    s.native_cookie_admission_snapshot(
        session,
        &crate::native_admission_snapshot::NativeAdmissionRequest::new(
            vec!["authority.test".into()],
            Vec::new(),
        )
        .unwrap(),
    )
    .await
    .unwrap()
}
async fn native_seed_contract(s: Store) {
    let (account, session) = seed(&s).await;
    let source = publication_source(&s, &session).await;
    let root = s
        .native_authority_fence(&session, &["authority.test".into()])
        .await
        .unwrap()
        .unwrap();
    let descriptor = s
        .prepare_native_authority_seed(&source, &root)
        .await
        .unwrap()
        .unwrap();
    assert!(descriptor.subjects().contains(&account));
    assert!(
        descriptor
            .subjects()
            .contains(&"person@authority.test".into())
    );
    assert!(
        s.native_authority_fence_with_subjects(
            &session,
            &["authority.test".into()],
            &descriptor.subjects()
        )
        .await
        .unwrap()
        .is_none()
    );
    // Global policy changes between canonical extraction and seed reject too;
    // own account/session unchanged is not a sufficient derivation fence.
    s.set_setting("v6:admin:flags:some-qualified-user", r#"{"disabled":true}"#)
        .await
        .unwrap();
    assert!(!s.seed_native_authority_subjects(&descriptor).await.unwrap());
    let root = s
        .native_authority_fence(&session, &["authority.test".into()])
        .await
        .unwrap()
        .unwrap();
    let descriptor = s
        .prepare_native_authority_seed(&source, &root)
        .await
        .unwrap()
        .unwrap();
    q("INSERT INTO domain_alias(alias,domain) VALUES('unused.authority.test','authority.test')")
        .execute(&s.backend)
        .await
        .unwrap();
    assert!(
        !s.seed_native_authority_subjects(&descriptor).await.unwrap(),
        "routing mutation cannot bless old canonical derivation"
    );
    assert!(
        s.authority_versions(&[AuthorityScope::new(
            AuthorityKind::Subject,
            "person@authority.test"
        )
        .unwrap()])
            .await
            .unwrap()
            .stamps
            .is_empty()
    );
    let root = s
        .native_authority_fence(&session, &["authority.test".into()])
        .await
        .unwrap()
        .unwrap();
    let fresh = s
        .prepare_native_authority_seed(&publication_source(&s, &session).await, &root)
        .await
        .unwrap()
        .unwrap();
    assert!(s.seed_native_authority_subjects(&fresh).await.unwrap());
    assert!(
        s.native_authority_fence_with_subjects(
            &session,
            &["authority.test".into()],
            &fresh.subjects()
        )
        .await
        .unwrap()
        .is_some()
    );
    // A freshly supplied vector must not certify an older opaque snapshot's
    // canonical input fields even when owner/principal IDs are unchanged.
    q("UPDATE sessions SET username='renamed@authority.test' WHERE id=?1")
        .bind(&session)
        .execute(&s.backend)
        .await
        .unwrap();
    let current = s
        .native_authority_fence(&session, &["authority.test".into()])
        .await
        .unwrap()
        .unwrap();
    let stale = s
        .prepare_native_authority_seed(&source, &current)
        .await
        .unwrap()
        .unwrap();
    assert!(!s.seed_native_authority_subjects(&stale).await.unwrap());
    let foreign = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    assert!(
        foreign
            .seed_native_authority_subjects(&fresh)
            .await
            .is_err()
    );
    foreign.close().await;
    s.close().await;
}
async fn publication_settings_contract(s: Store) {
    let (account, _session) = seed(&s).await;
    let subject = crate::settings_policy::SettingSubject {
        personal_owner: None,
        domain: "authority.test",
        account_id: &account,
        address: "person@authority.test",
    };
    let canonical = s.settings_publication_canonical(&subject).await.unwrap();
    let policy = crate::settings_policy::SettingPolicyRow {
        key: "fixture.publication".into(),
        global_value: Some("false".into()),
        allow_user_override: true,
        global_locked: false,
        updated_by: "fixture".into(),
        updated_at: 1,
    };
    s.put_setting_policy(&policy).await.unwrap();
    let old = s.settings_read_revision().await.unwrap();
    let legacy = s
        .settings_resolution_inputs_at(&subject, &old)
        .await
        .unwrap();
    q("UPDATE setting_policy SET global_value='true' WHERE key='fixture.publication'")
        .execute(&s.backend)
        .await
        .unwrap();
    let published = s.settings_publication_inputs(&canonical).await.unwrap();
    assert_eq!(
        legacy
            .policies
            .iter()
            .find(|p| p.key == "fixture.publication")
            .unwrap()
            .global_value
            .as_deref(),
        Some("false")
    );
    assert_eq!(
        published
            .policies
            .iter()
            .find(|p| p.key == "fixture.publication")
            .unwrap()
            .global_value
            .as_deref(),
        Some("true"),
        "publication must not reuse the old memo"
    );
    q("INSERT INTO user_group(id,domain,name,source,created_at) VALUES('publication-group','authority.test','Fixture','manual',1)").execute(&s.backend).await.unwrap();
    q("INSERT INTO user_group_member(group_id,account_id) VALUES('publication-group',?1)")
        .bind(&account)
        .execute(&s.backend)
        .await
        .unwrap();
    q("INSERT INTO feature_profile(id,name,values_json,builtin,updated_at) VALUES('publication-profile','Publication','{}',0,1)").execute(&s.backend).await.unwrap();
    q("INSERT INTO feature_profile_assignment(scope_kind,scope_id,profile_id,priority) VALUES('user',?1,'publication-profile',9)").bind(&account).execute(&s.backend).await.unwrap();
    for (scope, id, value) in [
        ("domain", "authority.test", "1"),
        ("profile", "publication-profile", "2"),
        ("group", "publication-group", "3"),
    ] {
        q("INSERT INTO setting_value(scope_kind,scope_id,key,value,locked,updated_by,updated_at) VALUES(?1,?2,'fixture.publication',?3,1,'fixture',1)").bind(scope).bind(id).bind(value).execute(&s.backend).await.unwrap();
    }
    let revision = s.settings_read_revision().await.unwrap();
    let old = s
        .settings_resolution_inputs_at(&subject, &revision)
        .await
        .unwrap();
    let new = s.settings_publication_inputs(&canonical).await.unwrap();
    assert_eq!(old.canonical, new.canonical);
    assert_eq!(old.policies, new.policies);
    assert_eq!(
        old.values, new.values,
        "profile/group order and lock rows are the existing reader's inputs"
    );
    s.close().await;
}
async fn publication_bounds_contract(s: Store) {
    let (account, session) = seed(&s).await;
    let canonical = s
        .settings_publication_canonical(&crate::settings_policy::SettingSubject {
            personal_owner: None,
            domain: "authority.test",
            account_id: &account,
            address: "person@authority.test",
        })
        .await
        .unwrap();
    s.put_setting_policy(&crate::settings_policy::SettingPolicyRow {
        key: "fixture.oversize".into(),
        global_value: Some("x".repeat(3 * 1024 * 1024)),
        allow_user_override: false,
        global_locked: true,
        updated_by: "fixture".into(),
        updated_at: 1,
    })
    .await
    .unwrap();
    assert!(s.settings_publication_inputs(&canonical).await.is_err());
    let (rows, bytes) = s
        .settings_publication_probe_for_test(&canonical)
        .await
        .unwrap();
    assert!(rows > 0);
    assert!(
        bytes < 4096,
        "oversized values must be masked inside SELECT before driver transfer"
    );
    s.delete_setting_policy("fixture.oversize").await.unwrap();
    let mut source = publication_source(&s, &session).await;
    let old_account = source.account().unwrap().unwrap();
    let old_domains = source.domains();
    source.split_publication_parts();
    assert_eq!(source.account().unwrap().unwrap().id, old_account.id);
    assert_eq!(source.domains().len(), old_domains.len());
    assert!(source.bounded_shape());
    assert!(!source.publication_parts().is_empty());
    use crate::native_admission_snapshot::{NativeAdmissionRequest, NativeAdmissionSharedKind};
    let account_index = source
        .publication_parts()
        .iter()
        .position(|part| part.kind() == NativeAdmissionSharedKind::Account)
        .unwrap();
    let own_part = source.publication_parts()[account_index].clone();
    source
        .replace_publication_part(account_index, own_part)
        .unwrap();
    let (_, other_session) = seed(&s).await;
    let mut other = publication_source(&s, &other_session).await;
    other.split_publication_parts();
    let foreign_part = other
        .publication_parts()
        .iter()
        .find(|part| part.kind() == NativeAdmissionSharedKind::Account)
        .unwrap()
        .clone();
    assert!(
        source
            .replace_publication_part(account_index, foreign_part)
            .is_err(),
        "same Store does not make another account's component compatible"
    );
    s.set_setting("fixture.publication.flag-a", "{}")
        .await
        .unwrap();
    s.set_setting("fixture.publication.flag-b", "{}")
        .await
        .unwrap();
    let mut flags = Vec::new();
    for key in ["fixture.publication.flag-a", "fixture.publication.flag-b"] {
        let mut snapshot = s
            .native_cookie_admission_snapshot(
                &session,
                &NativeAdmissionRequest::new(vec!["authority.test".into()], vec![key.into()])
                    .unwrap(),
            )
            .await
            .unwrap();
        snapshot.split_publication_parts();
        flags.push(snapshot);
    }
    let wrong_coverage = flags[1]
        .publication_parts()
        .iter()
        .find(|part| part.kind() == NativeAdmissionSharedKind::Flags)
        .unwrap()
        .clone();
    let flag_index = flags[0]
        .publication_parts()
        .iter()
        .position(|part| part.kind() == NativeAdmissionSharedKind::Flags)
        .unwrap();
    assert!(
        flags[0]
            .replace_publication_part(flag_index, wrong_coverage)
            .is_err(),
        "exact requested flag-key coverage is part of compatibility"
    );
    let foreign = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    assert!(source.session(&foreign).is_err());
    foreign.close().await;
    s.close().await;
}
#[tokio::test]
async fn sqlite_native_seed_fences_policy_routing_and_source() {
    native_seed_contract(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn postgres_native_seed_fences_policy_routing_and_source() {
    postgres(native_seed_contract).await;
}
#[tokio::test]
async fn sqlite_publication_settings_match_reader_without_legacy_memo() {
    publication_settings_contract(Store::open_in_memory(ServerKey::generate()).await.unwrap())
        .await;
}
#[tokio::test]
async fn postgres_publication_settings_match_reader_without_legacy_memo() {
    postgres(publication_settings_contract).await;
}
#[tokio::test]
async fn sqlite_publication_bounds_and_shared_sources_keep_scope() {
    publication_bounds_contract(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn postgres_publication_bounds_and_shared_sources_keep_scope() {
    postgres(publication_bounds_contract).await;
}

async fn publication_schema_contract(s: Store) {
    let (_account, session) = seed(&s).await;
    let (fence, counters) = crate::pool_telemetry::measure_queries(s.native_publication_fence(
        &session,
        &["authority.test".into()],
        &[],
    ))
    .await;
    let fence = fence.unwrap().unwrap();
    assert_eq!(counters.read.calls, 1);
    assert_eq!(counters.other.calls + counters.write.calls, 0);
    assert!(
        s.validate_native_publication_schema(&fence).await.unwrap(),
        "fresh required definitions must be recognized"
    );
    for d in [Dialect::Sqlite, Dialect::Postgres] {
        assert!(super::fence::publication_sql(d).len() < 16 * 1024);
    }
    q("ALTER TABLE accounts ADD COLUMN publication_schema_probe BIGINT")
        .execute(&s.backend)
        .await
        .unwrap();
    let changed = s
        .native_publication_fence(&session, &["authority.test".into()], &[])
        .await
        .unwrap()
        .unwrap();
    assert!(
        !fence.same_published_source(&changed).unwrap(),
        "relevant schema changes invalidate prepared component identity"
    );
    match s.backend.dialect() {
        Dialect::Sqlite => q("DROP TRIGGER authority_settings_update")
            .execute(&s.backend)
            .await
            .unwrap(),
        Dialect::Postgres => q("DROP TRIGGER authority_settings ON settings")
            .execute(&s.backend)
            .await
            .unwrap(),
    };
    let missing = s
        .native_publication_fence(&session, &["authority.test".into()], &[])
        .await
        .unwrap()
        .unwrap();
    assert!(
        !s.validate_native_publication_schema(&missing)
            .await
            .unwrap()
    );
    // A raw authority write with its new invalidation trigger absent cannot be
    // mistaken for the previous prepared input even when epoch counters match.
    s.set_setting(
        "v6:admin:flags:fixture-under-missing-trigger",
        r#"{"disabled":true}"#,
    )
    .await
    .unwrap();
    q(super::schema::fixture_restore_trigger(s.backend.dialect()))
        .execute(&s.backend)
        .await
        .unwrap();
    let restored = s
        .native_publication_fence(&session, &["authority.test".into()], &[])
        .await
        .unwrap()
        .unwrap();
    assert!(
        s.validate_native_publication_schema(&restored)
            .await
            .unwrap()
    );
    assert!(
        !fence.same_published_source(&restored).unwrap(),
        "drop/recreate identical DDL must not ABA into old input data"
    );
    match s.backend.dialect() {
        Dialect::Sqlite => {
            q("DROP TRIGGER authority_settings_update")
                .execute(&s.backend)
                .await
                .unwrap();
            q("CREATE TRIGGER authority_settings_update AFTER UPDATE ON settings BEGIN SELECT 1; END").execute(&s.backend).await.unwrap();
        }
        Dialect::Postgres => {
            q("DROP TRIGGER authority_settings ON settings")
                .execute(&s.backend)
                .await
                .unwrap();
            q("CREATE TRIGGER authority_settings AFTER INSERT OR UPDATE OR DELETE ON settings FOR EACH ROW WHEN(false) EXECUTE FUNCTION mw_authority_settings()").execute(&s.backend).await.unwrap();
        }
    }
    let noop = s
        .native_publication_fence(&session, &["authority.test".into()], &[])
        .await
        .unwrap()
        .unwrap();
    assert!(
        !s.validate_native_publication_schema(&noop).await.unwrap(),
        "name/timing/function alone cannot certify active coverage"
    );
    s.close().await;
}
async fn seed_anchored_subjects(s: &Store, session: &str) -> Vec<String> {
    let source = publication_source(s, session).await;
    let root = s
        .native_publication_fence(session, &["authority.test".into()], &[])
        .await
        .unwrap()
        .unwrap();
    assert!(s.validate_native_publication_schema(&root).await.unwrap());
    let seed = s
        .prepare_native_authority_seed(&source, &root)
        .await
        .unwrap()
        .unwrap();
    let subjects = seed.subjects();
    assert!(s.seed_native_authority_subjects(&seed).await.unwrap());
    subjects
}
async fn publication_anchor_contract(s: Store) {
    let (account, session) = seed(&s).await;
    let subjects = seed_anchored_subjects(&s, &session).await;
    let address = AuthorityScope::new(AuthorityKind::Subject, "person@authority.test").unwrap();
    let before = current(&s, &address).await;
    ack_all(&s).await;
    s.cleanup_obsolete_authority_scopes(128).await.unwrap();
    assert_eq!(
        before,
        current(&s, &address).await,
        "live absence anchor prevents premature cleanup"
    );
    seed_anchored_subjects(&s, &session).await;
    assert_eq!(
        before,
        current(&s, &address).await,
        "repeated warm setup never increments an epoch"
    );
    q("UPDATE sessions SET username='old-label@authority.test' WHERE id=?1")
        .bind(&session)
        .execute(&s.backend)
        .await
        .unwrap();
    seed_anchored_subjects(&s, &session).await;
    let old_label =
        AuthorityScope::new(AuthorityKind::Subject, "old-label@authority.test").unwrap();
    assert_eq!(
        s.authority_versions(&[old_label.clone()])
            .await
            .unwrap()
            .stamps
            .len(),
        1
    );
    q("UPDATE sessions SET username='new-label@authority.test' WHERE id=?1")
        .bind(&session)
        .execute(&s.backend)
        .await
        .unwrap();
    seed_anchored_subjects(&s, &session).await;
    assert!(
        s.authority_versions(&[old_label])
            .await
            .unwrap()
            .stamps
            .is_empty(),
        "subject rewrites replace the live set, not an append-only history"
    );
    assert!(
        q("SELECT COUNT(*) AS n FROM authority_subject_anchor WHERE session_id=?1")
            .bind(&session)
            .fetch_one(&s.backend)
            .await
            .unwrap()
            .get_i64("n")
            <= 8
    );
    let credentials = Credentials {
        username: "person@authority.test".into(),
        password: "second-owned-source".into(),
    };
    let second = s
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "other.authority.test",
                port: 993,
                tls: "implicit",
                username: &credentials.username,
                sync_policy_json: "{}",
            },
            &credentials,
        )
        .await
        .unwrap();
    let second_session = s
        .create_session(
            &second,
            &credentials.username,
            "engine",
            "engine",
            &credentials,
        )
        .await
        .unwrap();
    s.set_session_authentication(&second_session, "password")
        .await
        .unwrap();
    s.bind_verified_primary(
        &second_session,
        VerifiedPrimary {
            account_id: &second,
            kind: "mailbox",
            authority: "fixture.second",
            subject: &credentials.username,
        },
        1,
    )
    .await
    .unwrap()
    .unwrap();
    seed_anchored_subjects(&s, &second_session).await;
    let shared = current(&s, &address).await;
    let principals = s
        .dsr_resolve_principals(&[account.clone()], "person@alias.test")
        .await
        .unwrap();
    let person = crate::erasure::PersonIds {
        address: "person@alias.test".into(),
        account_ids: vec![account.clone()],
        owners: vec![account],
        principals,
    };
    let (_, errors) = s.dsr_purge_person(&person).await;
    assert!(errors.is_empty());
    assert_eq!(
        shared,
        current(&s, &address).await,
        "another live account's shared subject survives"
    );
    let principals = s
        .dsr_resolve_principals(&[second.clone()], "person@different-alias.test")
        .await
        .unwrap();
    let person = crate::erasure::PersonIds {
        address: "person@different-alias.test".into(),
        account_ids: vec![second.clone()],
        owners: vec![second],
        principals,
    };
    let (_, errors) = s.dsr_purge_person(&person).await;
    assert!(errors.is_empty());
    assert!(
        s.authority_versions(&[address])
            .await
            .unwrap()
            .stamps
            .is_empty(),
        "last anchor erases canonical PII even when the requested address was an alias"
    );
    assert!(
        s.native_publication_fence(&session, &["authority.test".into()], &subjects)
            .await
            .unwrap()
            .is_none()
    );
    s.close().await;
}
#[tokio::test]
async fn sqlite_publication_schema_stamp_and_required_definitions_fail_closed() {
    publication_schema_contract(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn postgres_publication_schema_stamp_and_required_definitions_fail_closed() {
    postgres(publication_schema_contract).await;
}
#[tokio::test]
async fn sqlite_live_absence_anchors_replace_bound_and_erase_shared_subjects() {
    publication_anchor_contract(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn postgres_live_absence_anchors_replace_bound_and_erase_shared_subjects() {
    postgres(publication_anchor_contract).await;
}
