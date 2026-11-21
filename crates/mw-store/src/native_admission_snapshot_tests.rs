use super::*;
use crate::{
    AccountKind, Credentials, DomainConfigRow, NewAccount, ServerKey,
    identity_principal::VerifiedPrimary,
};
async fn seed(s: &Store) -> (String, String) {
    let user = "person@example.test";
    let credentials = Credentials {
        username: user.into(),
        password: "fixture-secret-never-log".into(),
    };
    let account = s
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "mail.example.test",
                port: 993,
                tls: "implicit",
                username: user,
                sync_policy_json: "{}",
            },
            &credentials,
        )
        .await
        .unwrap();
    s.put_domain_config(&DomainConfigRow {
        name: "example.test".into(),
        upstream_json: "{}".into(),
        allowlist_json: "[]".into(),
        blocklist_json: "[]".into(),
        enabled: true,
        preferred: false,
        settings_json: "{}".into(),
        updated_at: 1,
    })
    .await
    .unwrap();
    s.remember_account_names(&account, &[user]).await.unwrap();
    let id = s
        .create_session(&account, user, "engine", "engine", &credentials)
        .await
        .unwrap();
    s.set_session_authentication(&id, "password").await.unwrap();
    s.bind_verified_primary(
        &id,
        VerifiedPrimary {
            account_id: &account,
            kind: "mailbox",
            authority: "fixture",
            subject: user,
        },
        1,
    )
    .await
    .unwrap()
    .unwrap();
    (account, id)
}
fn request() -> NativeAdmissionRequest {
    NativeAdmissionRequest::new(
        vec!["example.test".into()],
        vec!["fixture:exact:flag".into()],
    )
    .unwrap()
}
async fn contract(s: Store) {
    #[cfg(feature = "query-shape-diagnostics")]
    for dialect in [Dialect::Sqlite, Dialect::Postgres] {
        assert!(
            sql(dialect).len() <= crate::pool_telemetry::shapes::MAX_SQL_BYTES,
            "actual generated native template must fit the shared diagnostic bound"
        );
    }
    let (account, id) = seed(&s).await;
    #[cfg(feature = "query-shape-diagnostics")]
    {
        let (result, report) = crate::pool_telemetry::shapes::measure(
            s.native_cookie_admission_snapshot(&id, &request()),
        ).await;
        result.unwrap();
        report.require_valid().expect("real composite attribution must stay complete");
        assert_eq!(report.operations_completed, 1);
        assert_eq!(report.query_counts.read.calls, 1);
    }
    let witness=s.primary_session_policy_witness(&id).await.unwrap();
    let observed=s.native_cookie_admission_snapshot(&id,&request()).await.unwrap();
    let (same,counters)=crate::pool_telemetry::measure_queries(async {observed.matches_primary_policy_witness(&s,&witness)}).await;
    assert!(same.unwrap());assert_eq!(counters.read.calls+counters.write.calls+counters.other.calls,0,"auth witness comparison uses the LAST composite fields, not another SELECT");
    s.touch_session(&id).await.unwrap();
    assert!(s.native_cookie_admission_snapshot(&id,&request()).await.unwrap().matches_primary_policy_witness(&s,&witness).unwrap(),"last_seen is deliberately not authority");
    q("UPDATE sessions SET username='changed@example.test' WHERE id=?1").bind(&id).execute(&s.backend).await.unwrap();
    assert!(!s.native_cookie_admission_snapshot(&id,&request()).await.unwrap().matches_primary_policy_witness(&s,&witness).unwrap());
    q("UPDATE sessions SET username='person@example.test' WHERE id=?1").bind(&id).execute(&s.backend).await.unwrap();
    let (raw, count) =
        crate::pool_telemetry::measure_queries(s.native_cookie_admission_snapshot(&id, &request()))
            .await;
    let raw = raw.unwrap();
    assert_eq!(count.read.calls, 1);
    assert!(raw.bounded_shape());
    assert!(raw.revision(&s).unwrap().number().is_some());
    let source = raw.session(&s).unwrap().unwrap();
    let old = s.get_primary_session_with_activity(&id).await.unwrap();
    assert_eq!(source.session.id, old.0.id);
    assert_eq!(source.session.account_id, old.0.account_id);
    assert_eq!(source.session.username, old.0.username);
    assert_eq!(source.session.jmap_url, old.0.jmap_url);
    assert_eq!(source.session.api_url, old.0.api_url);
    assert_eq!(source.session.credentials, old.0.credentials);
    assert_eq!(source.last_seen, old.1);
    let account_source = raw.account_credentials(&s).unwrap().unwrap();
    let account_old = s.account_credentials_with_revision(&account).await.unwrap();
    assert_eq!(account_source.credentials(), &account_old.0);
    assert_eq!(account_source.revision(), account_old.1.as_slice());
    assert!(!format!("{account_source:?}").contains("fixture-secret"));
    assert_eq!(
        raw.account().unwrap().unwrap(),
        s.get_account(&account).await.unwrap()
    );
    assert_eq!(
        raw.clocks().unwrap(),
        s.session_activity_clock(&id).await.unwrap()
    );
    assert_eq!(
        raw.names().unwrap(),
        s.account_names(&account).await.unwrap()
    );
    assert_eq!(
        raw.linked_policy_required().unwrap(),
        s.linked_direct_login_policy_required(&id, &account)
            .await
            .unwrap()
    );
    let domains = raw.domains();
    let d = domains.get("example.test").unwrap().as_ref().unwrap();
    assert_eq!(
        d.configuration,
        s.get_domain_config("example.test").await.unwrap()
    );
    assert!(!d.second_alias);
    let p = raw.principal().unwrap().unwrap();
    assert_eq!(p.primary_account_id, account);
    assert_eq!(p.principal_id, p.binding_principal_id);
    assert_eq!(p.state, "active");
    assert!(p.retired_at.is_none());
    assert!(raw.held_addresses().is_empty());
    assert!(raw.exact_flags().is_empty());
    assert!(!raw.zero_access().unwrap());
    assert!(raw.restrictions().unwrap().is_empty());
    for debug in [
        format!("{raw:?}"),
        format!("{source:?}"),
        format!("{p:?}"),
        format!("{:?}", request()),
    ] {
        assert!(!debug.contains("person@example.test"));
        assert!(!debug.contains("fixture-secret"));
        assert!(!debug.contains(&id));
    }
    let other = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    assert!(raw.revision(&other).is_err());
    assert!(raw.session(&other).is_err());
    assert!(raw.account_credentials(&other).is_err());
    other.close().await;
    // Current SQL source, independent of any earlier result/revision, is returned.
    q("UPDATE sessions SET activity_at='2000-01-01T00:00:00Z' WHERE id=?1")
        .bind(&id)
        .execute(&s.backend)
        .await
        .unwrap();
    q("UPDATE identity_binding SET retired_at=2")
        .execute(&s.backend)
        .await
        .unwrap();
    s.set_setting("fixture:exact:flag", "{\"disabled\":true}")
        .await
        .unwrap();
    q("UPDATE domains SET enabled=0 WHERE name='example.test'")
        .execute(&s.backend)
        .await
        .unwrap();
    let next = s
        .native_cookie_admission_snapshot(&id, &request())
        .await
        .unwrap();
    assert_eq!(next.clocks().unwrap().unwrap().1, "2000-01-01T00:00:00Z");
    assert_eq!(next.principal().unwrap().unwrap().retired_at, Some(2));
    assert!(
        !next.domains()["example.test"]
            .as_ref()
            .unwrap()
            .configuration
            .as_ref()
            .unwrap()
            .enabled
    );
    assert_eq!(
        next.exact_flags()["fixture:exact:flag"].as_ref().unwrap(),
        "{\"disabled\":true}"
    );
    // Combined source payload must be masked INSIDE SQL before driver transfer.
    let large = "x".repeat(MAX_PAYLOAD / 6 + 1);
    s.set_setting("fixture:exact:flag", &large).await.unwrap();
    let over = s
        .native_cookie_admission_snapshot(&id, &request())
        .await
        .unwrap();
    assert!(!over.bounded_shape());
    assert!(
        over.rows
            .iter()
            .all(|r| r.try_get_opt_string("setting_value").unwrap().is_none())
    );
    assert!(
        over.rows
            .iter()
            .all(|r| r.try_get_opt_string("username").unwrap().is_none())
    );
    assert!(over.session(&s).is_err());
    assert!(over.account_credentials(&s).is_err());
    s.set_setting("fixture:exact:flag", "small").await.unwrap();
    q("UPDATE sessions SET linked_internal=2 WHERE id=?1")
        .bind(&id)
        .execute(&s.backend)
        .await
        .unwrap();
    let internal = s
        .native_cookie_admission_snapshot(&id, &request())
        .await
        .unwrap();
    assert!(internal.session(&s).unwrap().is_none());
    assert!(internal.account().unwrap().is_none());
    q("DELETE FROM sessions WHERE id=?1")
        .bind(&id)
        .execute(&s.backend)
        .await
        .unwrap();
    let deleted = s
        .native_cookie_admission_snapshot(&id, &request())
        .await
        .unwrap();
    assert!(deleted.session(&s).unwrap().is_none());
    s.close().await;
}
#[tokio::test]
async fn sqlite_composite_matches_existing_fresh_sources_and_bounds_payload() {
    contract(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn postgres_composite_matches_existing_fresh_sources_and_bounds_payload() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        eprintln!("SKIP: MW_TEST_PG absent; not actual PostgreSQL evidence");
        return;
    };
    use sqlx::Connection;
    let schema = format!("native_admission_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&mut admin)
        .await
        .unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    contract(
        Store::open(scoped.as_str(), ServerKey::generate())
            .await
            .unwrap(),
    )
    .await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&mut admin)
        .await
        .unwrap();
}
#[tokio::test]
async fn sqlite_section_corruption_stays_deferred_and_input_limits_are_before_sql() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (_, id) = seed(&s).await;
    let clocks = s.session_activity_clock(&id).await.unwrap();
    q("UPDATE sessions SET last_seen=?2 WHERE id=?1")
        .bind(&id)
        .bind("x".repeat(1025))
        .execute(&s.backend)
        .await
        .unwrap();
    assert!(
        s.get_primary_session_with_activity(&id)
            .await
            .unwrap()
            .1
            .is_some()
    );
    let telemetry = s
        .native_cookie_admission_snapshot(&id, &request())
        .await
        .unwrap();
    let source = telemetry.session(&s).unwrap().unwrap();
    assert!(source.last_seen.is_none());
    assert!(s.session_touch_may_be_due_at(source.last_seen.as_deref(), 60, chrono::Utc::now()));
    assert_eq!(telemetry.clocks().unwrap(), clocks);
    q("UPDATE accounts SET sync_policy_json=?1")
        .bind(vec![255u8])
        .execute(&s.backend)
        .await
        .unwrap();
    let raw = s
        .native_cookie_admission_snapshot(&id, &request())
        .await
        .unwrap();
    assert!(raw.session(&s).is_ok());
    assert!(raw.account().is_err());
    assert!(!raw.bounded_shape());
    assert!(NativeAdmissionRequest::new(vec!["x".repeat(1025)], vec![]).is_err());
    assert!(NativeAdmissionRequest::new(vec![], vec!["x".into(); 257]).is_err());
    let (result, count) = crate::pool_telemetry::measure_queries(
        s.native_cookie_admission_snapshot(&"x".repeat(1025), &request()),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(count.read.calls, 0);
    s.close().await;
}
#[tokio::test]
async fn independent_store_writer_names_identity_and_missing_revision_are_not_cached() {
    let path =
        std::env::temp_dir().join(format!("mw-native-composite-{}.db", uuid::Uuid::new_v4()));
    let key = ServerKey::generate();
    let reader = Store::open(&path.to_string_lossy(), key.clone())
        .await
        .unwrap();
    let writer = Store::open(&path.to_string_lossy(), key).await.unwrap();
    let (account, id) = seed(&writer).await;
    let before = reader
        .native_cookie_admission_snapshot(&id, &request())
        .await
        .unwrap();
    assert!(before.revision(&reader).unwrap().number().is_some());
    writer
        .remember_account_names(&account, &["other@example.test"])
        .await
        .unwrap();
    q("UPDATE identity_principal SET state='retired'")
        .execute(&writer.backend)
        .await
        .unwrap();
    q("UPDATE settings_revision SET n=0 WHERE id=1")
        .execute(&writer.backend)
        .await
        .unwrap();
    let after = reader
        .native_cookie_admission_snapshot(&id, &request())
        .await
        .unwrap();
    assert_eq!(after.revision(&reader).unwrap().number(), Some(0));
    assert!(
        after
            .names()
            .unwrap()
            .contains(&"other@example.test".into())
    );
    assert_eq!(after.principal().unwrap().unwrap().state, "retired");
    q("DELETE FROM settings_revision WHERE id=1")
        .execute(&writer.backend)
        .await
        .unwrap();
    let missing = reader
        .native_cookie_admission_snapshot(&id, &request())
        .await
        .unwrap();
    assert_eq!(missing.revision(&reader).unwrap().number(), None);
    writer.delete_session(&id).await.unwrap();
    assert!(
        reader
            .native_cookie_admission_snapshot(&id, &request())
            .await
            .unwrap()
            .session(&reader)
            .unwrap()
            .is_none()
    );
    reader.close().await;
    writer.close().await;
    let _ = std::fs::remove_file(path);
}
#[tokio::test]
async fn held_overflow_and_original_database_order_are_preserved_without_early_flag_decode() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (_, id) = seed(&s).await;
    // Populate via a recursive SQL statement: the snapshot must retain 5001 rows,
    // not truncate to an apparently allowed matching prefix or silently pass.
    q("WITH RECURSIVE nums(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM nums WHERE n<5001) INSERT INTO mail_user(address,domain,local_part,origin,status,created_at) SELECT 'hold-'||n||'@example.test','example.test','hold-'||n,'import','disabled',1 FROM nums").execute(&s.backend).await.unwrap();
    let raw = s
        .native_cookie_admission_snapshot(&id, &request())
        .await
        .unwrap();
    assert_eq!(raw.held_addresses().len(), 5001);
    let expected=q("SELECT address FROM mail_user WHERE status IN ('disabled','deleting','deleted') ORDER BY address LIMIT 5001").fetch_all(&s.backend).await.unwrap().iter().map(|r|r.get_string("address")).collect::<Vec<_>>();
    assert_eq!(
        raw.held_addresses()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap(),
        expected
    );
    s.close().await;
}

#[tokio::test]
async fn alias_retarget_and_second_hop_are_fresh_data_not_a_prepared_identity_allow() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (_, id) = seed(&s).await;
    s.put_domain_alias("alias.test", "example.test")
        .await
        .unwrap();
    let request = NativeAdmissionRequest::new(vec!["alias.test".into()], vec![]).unwrap();
    let old = s
        .native_cookie_admission_snapshot(&id, &request)
        .await
        .unwrap();
    assert_eq!(
        old.domains()["alias.test"].as_ref().unwrap().canonical,
        "example.test"
    );
    let mut target = s.get_domain_config("example.test").await.unwrap().unwrap();
    target.name = "target.test".into();
    target.enabled = false;
    s.put_domain_config(&target).await.unwrap();
    q("UPDATE domain_alias SET domain='target.test' WHERE alias='alias.test'")
        .execute(&s.backend)
        .await
        .unwrap();
    let fresh = s
        .native_cookie_admission_snapshot(&id, &request)
        .await
        .unwrap();
    let domains = fresh.domains();
    let domain = domains["alias.test"].as_ref().unwrap();
    assert_eq!(domain.canonical, "target.test");
    assert!(!domain.configuration.as_ref().unwrap().enabled);
    q("INSERT INTO domain_alias(alias,domain) VALUES('target.test','example.test')")
        .execute(&s.backend)
        .await
        .unwrap();
    assert!(
        s.native_cookie_admission_snapshot(&id, &request)
            .await
            .unwrap()
            .domains()["alias.test"]
            .as_ref()
            .unwrap()
            .second_alias
    );
    s.close().await;
}

async fn credential_source_contract(s: Store) {
    let (account, id) = seed(&s).await;
    let before = s
        .native_cookie_admission_snapshot(&id, &request())
        .await
        .unwrap();
    let original = before.account_credentials(&s).unwrap().unwrap();
    let next = Credentials {
        username: "person@example.test".into(),
        password: "replaced-secret-do-not-log".into(),
    };
    let sealed = s.key.seal(&crate::encode_creds(&next)).unwrap();
    q("UPDATE accounts SET sealed_creds=?2 WHERE id=?1")
        .bind(&account)
        .bind(sealed.clone())
        .execute(&s.backend)
        .await
        .unwrap();
    // Simulate a writer committing during a connect/permit preparation wait:
    // the final SELECT must obtain replacement credentials, not initial hint.
    let fresh = s
        .native_cookie_admission_snapshot(&id, &request())
        .await
        .unwrap();
    let current = fresh.account_credentials(&s).unwrap().unwrap();
    assert_eq!(current.credentials(), &next);
    assert_eq!(current.revision(), sealed.as_slice());
    assert_ne!(original.revision(), current.revision());
    // Original CAS rejects only its exact source. A stale rejection cannot mark
    // the independent replacement, while a matching current rejection is seen.
    assert!(
        !s.reject_account_credential_revision(&account, original.revision())
            .await
            .unwrap()
    );
    assert!(
        s.reject_account_credential_revision(&account, current.revision())
            .await
            .unwrap()
    );
    let rejected = s
        .native_cookie_admission_snapshot(&id, &request())
        .await
        .unwrap();
    let policy: serde_json::Value =
        serde_json::from_str(&rejected.account().unwrap().unwrap().sync_policy_json).unwrap();
    assert_eq!(policy["credential_rejected"], true);
    assert_eq!(
        rejected
            .account_credentials(&s)
            .unwrap()
            .unwrap()
            .revision(),
        current.revision()
    );
    // Malformed account ciphertext is deferred until the connector reader. It
    // cannot make a good session magically authenticate or hide the old error.
    q("UPDATE accounts SET sealed_creds=?2 WHERE id=?1")
        .bind(&account)
        .bind(vec![1u8, 2, 3])
        .execute(&s.backend)
        .await
        .unwrap();
    let corrupt = s
        .native_cookie_admission_snapshot(&id, &request())
        .await
        .unwrap();
    assert!(corrupt.session(&s).unwrap().is_some());
    assert!(corrupt.account_credentials(&s).is_err());
    assert!(s.account_credentials_with_revision(&account).await.is_err());
    // Oversize is masked in SQL, before even the shared decoder sees ciphertext.
    q("UPDATE accounts SET sealed_creds=?2 WHERE id=?1")
        .bind(&account)
        .bind(vec![7u8; MAX_PAYLOAD + 1])
        .execute(&s.backend)
        .await
        .unwrap();
    let oversized = s
        .native_cookie_admission_snapshot(&id, &request())
        .await
        .unwrap();
    assert!(!oversized.bounded_shape());
    assert!(oversized.account_credentials(&s).is_err());
    use sqlx::Row as _;
    for row in &oversized.rows {
        let value: Option<Vec<u8>> = match row {
            Row::Sqlite(r) => r.try_get("account_sealed_creds").unwrap(),
            Row::Postgres(r) => r.try_get("account_sealed_creds").unwrap(),
        };
        assert!(value.is_none());
    }
    s.close().await;
}
#[tokio::test]
async fn sqlite_account_credentials_use_fresh_bound_source_and_original_rejection_cas() {
    credential_source_contract(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn postgres_account_credentials_use_fresh_bound_source_and_original_rejection_cas() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        eprintln!("SKIP: MW_TEST_PG absent; not actual PostgreSQL evidence");
        return;
    };
    use sqlx::Connection;
    let schema = format!("native_credential_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&mut admin)
        .await
        .unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    credential_source_contract(
        Store::open(scoped.as_str(), ServerKey::generate())
            .await
            .unwrap(),
    )
    .await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&mut admin)
        .await
        .unwrap();
}


async fn sibling_name_contract(s: Store) {
    let (account, id) = seed(&s).await;
    let creds = Credentials { username: "alias@example.test".into(), password: "same-owner-fixture-secret".into() };
    let sibling = s.create_session(&account, "clear-alias@example.test", "engine", "engine", &creds).await.unwrap();
    // Exactly the original reader semantics: a corrupt sealed credential loses
    // only its login name, not its clear name or all sibling-name evidence.
    let damaged = s.create_session(&account, "damaged@example.test", "engine", "engine", &creds).await.unwrap();
    q("UPDATE sessions SET sealed_creds=?2 WHERE id=?1").bind(&damaged).bind(vec![1u8,2,3]).execute(&s.backend).await.unwrap();
    let foreign_account = s.create_account(&NewAccount { kind:AccountKind::Imap,host:"mail.example.test",port:993,tls:"implicit",username:"foreign@foreign.test",sync_policy_json:"{}" },&creds).await.unwrap();
    let foreign = s.create_session(&foreign_account, "foreign@example.test", "engine", "engine", &creds).await.unwrap();
    q("UPDATE sessions SET sealed_creds=?2 WHERE id=?1").bind(&foreign).bind(vec![9u8; MAX_PAYLOAD+1]).execute(&s.backend).await.unwrap();
    let snapshot = s.native_cookie_admission_snapshot(&id, &request()).await.unwrap();
    assert!(snapshot.bounded_shape(), "unrelated account payload must not enter source budget");
    assert_eq!(snapshot.account_session_names(&s).unwrap().unwrap(), s.session_names_by_account(&account).await.unwrap());
    let names = snapshot.account_session_names(&s).unwrap().unwrap();
    assert!(names.iter().any(|name| name.username == "clear-alias@example.test" && name.login_name.as_deref() == Some("alias@example.test")));
    assert!(names.iter().any(|name| name.username == "damaged@example.test" && name.login_name.is_none()));
    assert!(names.iter().all(|name| name.username != "foreign@example.test"));
    assert_eq!(snapshot.native_subjects(&s).unwrap(), s.verified_session_subjects(&id).await.unwrap());
    let other_scope = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    assert!(snapshot.account_session_names(&other_scope).is_err());
    assert!(snapshot.native_subjects(&other_scope).is_err());
    other_scope.close().await;
    // Names and binding changes are fresh next observation, not an allow cache.
    q("UPDATE sessions SET username='new-clear@example.test' WHERE id=?1").bind(&sibling).execute(&s.backend).await.unwrap();
    let fresh = s.native_cookie_admission_snapshot(&id, &request()).await.unwrap();
    assert!(fresh.account_session_names(&s).unwrap().unwrap().iter().any(|name| name.username == "new-clear@example.test"));
    q("UPDATE identity_principal SET state='retired' WHERE id IN (SELECT principal_id FROM session_principal WHERE session_id=?1)").bind(&id).execute(&s.backend).await.unwrap();
    assert!(s.native_cookie_admission_snapshot(&id, &request()).await.unwrap().native_subjects(&s).unwrap().is_none());
    // Own oversized ciphertext is masked BEFORE any driver BLOB transfer.
    q("UPDATE sessions SET sealed_creds=?2 WHERE id=?1").bind(&sibling).bind(vec![3u8;MAX_PAYLOAD+1]).execute(&s.backend).await.unwrap();
    let oversized=s.native_cookie_admission_snapshot(&id,&request()).await.unwrap();
    assert!(!oversized.bounded_shape());assert!(oversized.account_session_names(&s).is_err());
    use sqlx::Row as _;
    for row in &oversized.rows {
        let bytes: Option<Vec<u8>>=match row { Row::Sqlite(row)=>row.try_get("sibling_sealed_creds").unwrap(),Row::Postgres(row)=>row.try_get("sibling_sealed_creds").unwrap() };
        assert!(bytes.is_none());
    }
    q("UPDATE sessions SET sealed_creds=?2 WHERE id=?1").bind(&sibling).bind(s.key.seal(&crate::encode_creds(&creds)).unwrap()).execute(&s.backend).await.unwrap();
    // A sentinel never silently drops a name that might carry a deny flag.
    for n in 3..33 {
        s.create_session(&account, &format!("sibling-{n}@example.test"), "engine", "engine", &creds).await.unwrap();
    }
    assert!(s.native_cookie_admission_snapshot(&id, &request()).await.unwrap().account_session_names(&s).unwrap().is_none());
    s.close().await;
}
#[tokio::test]
async fn sqlite_sibling_names_are_owned_bounded_fresh_and_match_original_decoder() {
    sibling_name_contract(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn postgres_sibling_names_are_owned_bounded_fresh_and_match_original_decoder() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else { eprintln!("SKIP: MW_TEST_PG absent; not actual PostgreSQL evidence"); return; };
    use sqlx::Connection;
    let schema = format!("native_siblings_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&mut admin).await.unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped.query_pairs_mut().append_pair("options", &format!("-csearch_path={schema}"));
    sibling_name_contract(Store::open(scoped.as_str(), ServerKey::generate()).await.unwrap()).await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE")).execute(&mut admin).await.unwrap();
}
#[tokio::test]
async fn sqlite_sibling_join_uses_account_index_in_actual_composite_plan() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (_, id) = seed(&s).await;
    static EXPLAIN: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let explain = EXPLAIN.get_or_init(|| format!("EXPLAIN QUERY PLAN {}", sql(Dialect::Sqlite)));
    let rows = q(explain).bind(id).bind("[]").bind("[]").fetch_all(&s.backend).await.unwrap();
    let details = rows.iter().map(|row| row.get_string("detail")).collect::<Vec<_>>();
    assert!(details.iter().any(|detail| detail.contains("sibling") && detail.contains("idx_sessions_account")), "owner predicate must use account index: {details:?}");
    assert!(!details.iter().any(|detail| detail.contains("SCAN sibling")), "no unindexed full session scan: {details:?}");
    s.close().await;
}
