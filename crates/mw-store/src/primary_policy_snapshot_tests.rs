use super::*;
use crate::{Credentials, ServerKey};
async fn seed(store: &Store) -> String {
    store
        .create_session(
            "account-one",
            "person@example.test",
            "https://mail.example.test/session",
            "https://mail.example.test/api",
            &Credentials {
                username: "person@example.test".into(),
                password: "never-log-me".into(),
            },
        )
        .await
        .unwrap()
}
#[tokio::test]
async fn snapshot_is_one_fresh_read_and_activity_is_not_authority() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let id = seed(&store).await;
    let (snapshot, counts) =
        crate::pool_telemetry::measure_queries(store.primary_session_policy_snapshot(&id)).await;
    let snapshot = snapshot.unwrap();
    assert_eq!(counts.read.calls, 1);
    assert!(snapshot.witness.eligible());
    store.touch_session(&id).await.unwrap();
    let fresh = store.primary_session_policy_snapshot(&id).await.unwrap();
    assert!(snapshot.witness.matches(&fresh.witness));
    let (witness, counts) =
        crate::pool_telemetry::measure_queries(store.primary_session_policy_witness(&id)).await;
    assert_eq!(counts.read.calls, 1);
    assert!(snapshot.witness.matches(&witness.unwrap()));
    assert_eq!(snapshot.session.account_id, fresh.session.account_id);
    assert!(!format!("{snapshot:?}").contains("person@example.test"));
    assert!(!format!("{snapshot:?}").contains("never-log-me"));
    store.delete_session(&id).await.unwrap();
    assert!(matches!(
        store.primary_session_policy_snapshot(&id).await,
        Err(StoreError::NotFound)
    ));
}

#[tokio::test]
async fn independent_writer_changes_all_policy_namespaces_without_changing_session() {
    let path = std::env::temp_dir().join(format!("mw-policy-witness-{}.db", uuid::Uuid::new_v4()));
    let name = path.to_string_lossy().into_owned();
    let key = ServerKey::generate();
    let reader = Store::open(&name, key.clone()).await.unwrap();
    let writer = Store::open(&name, key).await.unwrap();
    // The dependency rows below name an account and a domain. Seed BOTH real
    // parents before inserting membership and alias rows; retain FK checks.
    let credentials = Credentials {
        username: "person@example.test".into(),
        password: "fixture-only".into(),
    };
    let generated = writer
        .create_account(
            &crate::NewAccount {
                kind: crate::AccountKind::Imap,
                host: "mail.example.test",
                port: 143,
                tls: "none",
                username: &credentials.username,
                sync_policy_json: "{}",
            },
            &credentials,
        )
        .await
        .unwrap();
    q("UPDATE accounts SET id='account-one' WHERE id=?1")
        .bind(&generated)
        .execute(&writer.backend)
        .await
        .unwrap();
    writer
        .put_domain_config(&crate::DomainConfigRow {
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
    let id = seed(&writer).await;
    // Every statement writes a real dependency covered by migration 0078.
    let mutations = [
        "INSERT INTO setting_policy(key,global_value,allow_user_override,global_locked,updated_by,updated_at) VALUES('access.protocols','[\"imap\"]',0,1,'test',1)",
        "INSERT INTO user_group(id,name,domain,source,created_at) VALUES('g','group','example.test','manual',1)",
        "INSERT INTO user_group_member(group_id,account_id) VALUES('g','account-one')",
        "INSERT INTO feature_profile(id,name,values_json,builtin,updated_at) VALUES('fp','profile','{}',0,1)",
        "INSERT INTO feature_profile_assignment(scope_kind,scope_id,profile_id,priority) VALUES('user','account-one','fp',1)",
        "INSERT INTO domain_alias(alias,domain) VALUES('alias.test','example.test')",
    ];
    for (index, sql) in mutations.into_iter().enumerate() {
        let before = reader.primary_session_policy_snapshot(&id).await.unwrap();
        q(sql)
            .execute(&writer.backend)
            .await
            .unwrap_or_else(|error| panic!("fixture dependency mutation {index} failed: {error}"));
        let after = reader.primary_session_policy_snapshot(&id).await.unwrap();
        assert!(!before.witness.matches(&after.witness));
        assert_eq!(before.session.username, after.session.username);
        assert_eq!(before.session.credentials, after.session.credentials);
    }
    // 0123: identity writes no longer move the settings revision. A principal
    // and binding that this session is not bound to leave its witness alone.
    for sql in [
        "INSERT INTO identity_principal(id,state,created_at) VALUES('principal','active',1)",
        "INSERT INTO identity_binding(id,principal_id,kind,authority,subject,created_at) VALUES('binding','principal','mailbox','fixture','one',1)",
    ] {
        let before = reader.primary_session_policy_snapshot(&id).await.unwrap();
        q(sql).execute(&writer.backend).await.unwrap();
        let after = reader.primary_session_policy_snapshot(&id).await.unwrap();
        assert!(before.witness.matches(&after.witness), "{sql}");
    }
    // Binding THIS session, and every later change to its identity row, does.
    let before = reader.primary_session_policy_snapshot(&id).await.unwrap();
    q("INSERT INTO session_principal(session_id,principal_id,binding_id,primary_account_id,auth_time,authentication_method) VALUES(?1,'principal','binding','account-one',1,'password')").bind(&id).execute(&writer.backend).await.unwrap();
    let after = reader.primary_session_policy_snapshot(&id).await.unwrap();
    assert!(!before.witness.matches(&after.witness));
    for sql in [
        "UPDATE session_principal SET auth_time=2",
        "UPDATE session_principal SET authentication_method='passkey'",
        "UPDATE identity_binding SET kind='sso'",
        "UPDATE identity_binding SET retired_at=2",
        "UPDATE identity_principal SET state='disabled'",
    ] {
        let before = reader.primary_session_policy_snapshot(&id).await.unwrap();
        q(sql).execute(&writer.backend).await.unwrap();
        let after = reader.primary_session_policy_snapshot(&id).await.unwrap();
        assert!(!before.witness.matches(&after.witness));
    }
    reader.close().await;
    writer.close().await;
    let _ = std::fs::remove_file(path);
}
#[tokio::test]
async fn credential_rotation_missing_revision_internal_session_and_cross_store_never_match() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let id = seed(&store).await;
    let before = store.primary_session_policy_snapshot(&id).await.unwrap();
    store
        .reseal_account_credentials("account-one", "rotated")
        .await
        .unwrap();
    let after = store.primary_session_policy_snapshot(&id).await.unwrap();
    assert!(!before.witness.matches(&after.witness));
    let other = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let second = seed(&other).await;
    let other = other
        .primary_session_policy_snapshot(&second)
        .await
        .unwrap();
    assert!(!after.witness.matches(&other.witness));
    q("DELETE FROM settings_revision WHERE id=1")
        .execute(&store.backend)
        .await
        .unwrap();
    let missing = store.primary_session_policy_snapshot(&id).await.unwrap();
    assert!(!missing.witness.eligible());
    assert!(!after.witness.matches(&missing.witness));
    q("UPDATE sessions SET linked_internal=1 WHERE id=?1")
        .bind(&id)
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(matches!(
        store.primary_session_policy_snapshot(&id).await,
        Err(StoreError::NotFound)
    ));
}

#[tokio::test]
async fn postgres_snapshot_and_final_witness_have_same_single_read_contract() {
    use sqlx::Connection;
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        eprintln!("SKIP: MW_TEST_PG absent");
        return;
    };
    let schema = format!("primary_policy_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&mut admin)
        .await
        .unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    let store = Store::open_postgres(scoped.as_str(), ServerKey::generate())
        .await
        .unwrap();
    let name = format!("account-{}", uuid::Uuid::new_v4());
    let credentials = Credentials {
        username: "person@example.test".into(),
        password: "fixture-only".into(),
    };
    let id = store
        .create_session(
            &name,
            &credentials.username,
            "https://mail.example.test/session",
            "https://mail.example.test/api",
            &credentials,
        )
        .await
        .unwrap();
    let (snapshot, counts) =
        crate::pool_telemetry::measure_queries(store.primary_session_policy_snapshot(&id)).await;
    assert_eq!(counts.read.calls, 1);
    let snapshot = snapshot.unwrap();
    let (witness, counts) =
        crate::pool_telemetry::measure_queries(store.primary_session_policy_witness(&id)).await;
    assert_eq!(counts.read.calls, 1);
    assert!(snapshot.witness.matches(&witness.unwrap()));
    store
        .reseal_account_credentials(&name, "rotated")
        .await
        .unwrap();
    let rotated = store.primary_session_policy_witness(&id).await.unwrap();
    assert!(!snapshot.witness.matches(&rotated));
    store.delete_session(&id).await.unwrap();
    assert!(matches!(
        store.primary_session_policy_witness(&id).await,
        Err(StoreError::NotFound)
    ));
    store.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&mut admin)
        .await
        .unwrap();
}
