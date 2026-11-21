use super::*;
use crate::{
    AccountKind, Credentials, DomainConfigRow, NewAccount, ServerKey,
    identity_principal::VerifiedPrimary,
};

async fn seed(s: &Store) -> (String, String) {
    let user = "person@authority.test";
    let creds = Credentials {
        username: user.into(),
        password: "fixture-secret-never-log".into(),
    };
    let account = s
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "mail.authority.test",
                port: 993,
                tls: "implicit",
                username: user,
                sync_policy_json: "{}",
            },
            &creds,
        )
        .await
        .unwrap();
    s.put_domain_config(&DomainConfigRow {
        name: "authority.test".into(),
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
    let session = s
        .create_session(&account, user, "engine", "engine", &creds)
        .await
        .unwrap();
    s.set_session_authentication(&session, "password")
        .await
        .unwrap();
    s.bind_verified_primary(
        &session,
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
    (account, session)
}
async fn current(s: &Store, scope: &AuthorityScope) -> AuthorityStamp {
    s.authority_versions(&[scope.clone()])
        .await
        .unwrap()
        .stamps
        .get(scope)
        .unwrap()
        .clone()
}
async fn ack_all(s: &Store) {
    for _ in 0..4 {
        let claims = s.claim_authority_publications(32, 1000, 30).await.unwrap();
        if claims.is_empty() {
            return;
        }
        for claim in claims {
            assert!(s.acknowledge_authority_publication(&claim).await.unwrap());
        }
    }
    panic!("bounded fixture did not drain coalesced targets");
}
async fn transaction_contract(s: Store) {
    let (account, _) = seed(&s).await;
    let scope = AuthorityScope::new(AuthorityKind::Account, &account).unwrap();
    let before = current(&s, &scope).await;
    let mut tx = s.backend.begin().await.unwrap();
    q("UPDATE accounts SET host='rolled-back.authority.test' WHERE id=?1")
        .bind(&account)
        .execute_tx(&mut tx)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    assert_eq!(current(&s, &scope).await, before);
    assert_eq!(
        s.get_account(&account).await.unwrap().host,
        "mail.authority.test"
    );
    // A publication-write failure cannot leave the business source committed.
    match s.backend.dialect() {
        Dialect::Sqlite => {
            q("CREATE TRIGGER authority_fixture_reject BEFORE UPDATE ON authority_publication_outbox WHEN NEW.scope_kind='account' BEGIN SELECT RAISE(ABORT,'owned fixture outbox failure'); END").execute(&s.backend).await.unwrap();
        }
        Dialect::Postgres => {
            q("CREATE FUNCTION authority_fixture_reject() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.scope_kind='account' THEN RAISE EXCEPTION 'owned fixture outbox failure'; END IF; RETURN NEW; END; $$").execute(&s.backend).await.unwrap();
            q("CREATE TRIGGER authority_fixture_reject BEFORE UPDATE ON authority_publication_outbox FOR EACH ROW EXECUTE FUNCTION authority_fixture_reject()").execute(&s.backend).await.unwrap();
        }
    }
    assert!(
        q("UPDATE accounts SET host='must-not-commit.authority.test' WHERE id=?1")
            .bind(&account)
            .execute(&s.backend)
            .await
            .is_err()
    );
    assert_eq!(
        s.get_account(&account).await.unwrap().host,
        "mail.authority.test"
    );
    assert_eq!(current(&s, &scope).await, before);
    let drop_sql = match s.backend.dialect() {
        Dialect::Sqlite => "DROP TRIGGER authority_fixture_reject",
        Dialect::Postgres => {
            "DROP TRIGGER authority_fixture_reject ON authority_publication_outbox"
        }
    };
    q(drop_sql).execute(&s.backend).await.unwrap();
    // Both independent transaction streams publish their latest coalesced target.
    let first = s.clone();
    let second = s.clone();
    let a = account.clone();
    let b = account.clone();
    let (left, right) = tokio::join!(
        async move {
            q("UPDATE accounts SET host='writer-one.authority.test' WHERE id=?1")
                .bind(a)
                .execute(&first.backend)
                .await
        },
        async move {
            q("UPDATE accounts SET tls='starttls' WHERE id=?1")
                .bind(b)
                .execute(&second.backend)
                .await
        }
    );
    left.unwrap();
    right.unwrap();
    let after = current(&s, &scope).await;
    assert_eq!(after.incarnation, before.incarnation);
    assert!(after.version > before.version);
    let row=q("SELECT target_version,incarnation FROM authority_publication_outbox WHERE scope_kind='account' AND scope_id=?1").bind(&account).fetch_one(&s.backend).await.unwrap();
    assert_eq!(row.get_i64("target_version"), after.version);
    assert_eq!(row.get_string("incarnation"), after.incarnation);
    assert_eq!(q("SELECT COUNT(*) AS n FROM authority_publication_outbox WHERE scope_kind='account' AND scope_id=?1").bind(&account).fetch_one(&s.backend).await.unwrap().get_i64("n"),1);
    if s.backend.dialect() == Dialect::Postgres {
        // Triggers must update their own schema even when a raw writer changes
        // its search_path before touching the qualified business source table.
        let mut tx = s.backend.begin().await.unwrap();
        let schema = q("SELECT current_schema() AS name")
            .fetch_all_tx(&mut tx)
            .await
            .unwrap()[0]
            .get_string("name");
        assert!(
            schema.starts_with("authority_")
                && schema
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        );
        q("SET LOCAL search_path TO pg_catalog")
            .execute_tx(&mut tx)
            .await
            .unwrap();
        if let crate::backend::Tx::Postgres(ref mut tx) = tx {
            sqlx::query(&format!(
                "UPDATE \"{schema}\".accounts SET tls='implicit' WHERE id=$1"
            ))
            .bind(&account)
            .execute(&mut **tx)
            .await
            .unwrap();
        }
        tx.commit().await.unwrap();
        assert_eq!(s.get_account(&account).await.unwrap().tls, "implicit");
        assert!(current(&s, &scope).await.version > after.version);
    }
    // Signed BIGINT exhaustion fails the source mutation, never wraps to a hit.
    q("UPDATE authority_epoch SET version=?1 WHERE scope_kind='account' AND scope_id=?2")
        .bind(i64::MAX)
        .bind(&account)
        .execute(&s.backend)
        .await
        .unwrap();
    assert!(
        q("UPDATE accounts SET host='overflow.authority.test' WHERE id=?1")
            .bind(&account)
            .execute(&s.backend)
            .await
            .is_err()
    );
    assert_eq!(
        s.get_account(&account).await.unwrap().host,
        "writer-one.authority.test"
    );
    s.close().await;
}
async fn claim_aba_contract(s: Store) {
    let (account, session) = seed(&s).await;
    let scope = AuthorityScope::new(AuthorityKind::Account, &account).unwrap();
    let old = s
        .claim_authority_publications(32, 1000, 30)
        .await
        .unwrap()
        .into_iter()
        .find(|claim| claim.stamp.scope == scope)
        .unwrap();
    q("UPDATE accounts SET host='new.authority.test' WHERE id=?1")
        .bind(&account)
        .execute(&s.backend)
        .await
        .unwrap();
    assert!(!s.acknowledge_authority_publication(&old).await.unwrap());
    let current_claim = s
        .claim_authority_publications(32, 1001, 30)
        .await
        .unwrap()
        .into_iter()
        .find(|claim| claim.stamp.scope == scope)
        .unwrap();
    assert!(current_claim.stamp.version > old.stamp.version);
    // Lease time is reclaimability only; token + exact incarnation/target fence ACK.
    let reclaimed = s
        .claim_authority_publications(32, 1062, 30)
        .await
        .unwrap()
        .into_iter()
        .find(|claim| claim.stamp.scope == scope)
        .unwrap();
    assert!(
        !s.acknowledge_authority_publication(&current_claim)
            .await
            .unwrap()
    );
    assert!(
        s.acknowledge_authority_publication(&reclaimed)
            .await
            .unwrap()
    );
    assert!(
        !s.acknowledge_authority_publication(&reclaimed)
            .await
            .unwrap()
    );
    let original = current(&s, &scope).await;
    q("DELETE FROM authority_epoch WHERE scope_kind='account' AND scope_id=?1")
        .bind(&account)
        .execute(&s.backend)
        .await
        .unwrap();
    assert!(
        s.native_authority_fence(&session, &["authority.test".into()])
            .await
            .unwrap()
            .is_none()
    );
    s.seed_authority_scopes(&[scope.clone()]).await.unwrap();
    let recreated = current(&s, &scope).await;
    assert_ne!(recreated.incarnation, original.incarnation);
    assert_eq!(recreated.version, 1);
    assert!(!s.acknowledge_authority_publication(&old).await.unwrap());
    // Missing source cannot be manufactured into a live scope by the seed API.
    let absent =
        AuthorityScope::new(AuthorityKind::Account, "nonexistent-fixture-account").unwrap();
    s.seed_authority_scopes(&[absent.clone()]).await.unwrap();
    assert!(
        s.authority_versions(&[absent])
            .await
            .unwrap()
            .stamps
            .is_empty()
    );
    let session_scope = AuthorityScope::new(AuthorityKind::Session, &session).unwrap();
    let session_before = current(&s, &session_scope).await;
    s.delete_session(&session).await.unwrap();
    ack_all(&s).await;
    assert!(s.cleanup_obsolete_authority_scopes(128).await.unwrap() > 0);
    assert!(
        s.authority_versions(&[session_scope.clone()])
            .await
            .unwrap()
            .stamps
            .is_empty()
    );
    let creds = Credentials {
        username: "person@authority.test".into(),
        password: "fixture-secret-never-log".into(),
    };
    // Direct rawSQL recreate the exact same cookie ID, exercising actual ABA.
    let replacement = s
        .create_session(
            &account,
            "person@authority.test",
            "engine",
            "engine",
            &creds,
        )
        .await
        .unwrap();
    q("UPDATE sessions SET id=?1 WHERE id=?2")
        .bind(&session)
        .bind(&replacement)
        .execute(&s.backend)
        .await
        .unwrap();
    let session_after = current(&s, &session_scope).await;
    assert_ne!(session_before.incarnation, session_after.incarnation);
    s.close().await;
}
async fn fence_contract(s: Store) {
    let policy = AuthorityScope::new(AuthorityKind::Policy, "").unwrap();
    let policy_before = current(&s, &policy).await;
    let legacy_before = s.settings_revision().await.unwrap();
    let (account, session) = seed(&s).await;
    assert_eq!(
        current(&s, &policy).await,
        policy_before,
        "logins must not invalidate every published policy scope"
    );
    assert_ne!(
        s.settings_revision().await.unwrap(),
        legacy_before,
        "legacy counter remains independent"
    );
    let domains = ["authority.test".into()];
    let (read, queries) =
        crate::pool_telemetry::measure_queries(s.native_authority_fence(&session, &domains)).await;
    let before = read.unwrap().unwrap();
    assert_eq!(queries.read.calls, 1);
    assert_eq!(queries.write.calls + queries.other.calls, 0);
    assert_eq!(before.account_id(), account);
    let old_clock = before.clocks().1.to_owned();
    q("UPDATE sessions SET activity_at='2020-01-01T00:00:00Z' WHERE id=?1")
        .bind(&session)
        .execute(&s.backend)
        .await
        .unwrap();
    let after = s
        .native_authority_fence(&session, &domains)
        .await
        .unwrap()
        .unwrap();
    assert_ne!(after.clocks().1, old_clock);
    assert_eq!(after.clocks().1, "2020-01-01T00:00:00Z");
    assert!(before.same_published_source(&after).unwrap());
    // Telemetry/clock updates remain fresh inputs, not global or scoped
    // authority rebuilds. Sibling ordering is not certified by this fence.
    q("UPDATE sessions SET last_seen='2021-02-03T04:05:06Z',created_at='2019-01-01T00:00:00Z' WHERE id=?1")
        .bind(&session).execute(&s.backend).await.unwrap();
    let touched = s
        .native_authority_fence(&session, &domains)
        .await
        .unwrap()
        .unwrap();
    assert!(after.same_published_source(&touched).unwrap());
    assert_eq!(touched.last_seen(), Some("2021-02-03T04:05:06Z"));
    assert_eq!(touched.clocks().0, "2019-01-01T00:00:00Z");
    q("UPDATE sessions SET authentication_method='sso' WHERE id=?1")
        .bind(&session)
        .execute(&s.backend)
        .await
        .unwrap();
    let changed = s
        .native_authority_fence(&session, &domains)
        .await
        .unwrap()
        .unwrap();
    assert!(!touched.same_published_source(&changed).unwrap());
    assert_eq!(current(&s, &policy).await, policy_before);
    assert!(AuthorityScope::new(AuthorityKind::Domain, "nonascii-\u{e9}.test").is_err());
    assert!(
        s.native_authority_fence(&session, &["invalid\0tail".into()])
            .await
            .is_err()
    );
    assert!(
        s.native_authority_fence_with_subjects(&session, &domains, &["unseeded-subject".into()])
            .await
            .unwrap()
            .is_none()
    );
    // A cached account/principal reference cannot hide current reassignment.
    let credentials = Credentials {
        username: "other@authority.test".into(),
        password: "foreign-fixture-secret".into(),
    };
    let other = s
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "mail.authority.test",
                port: 993,
                tls: "implicit",
                username: &credentials.username,
                sync_policy_json: "{}",
            },
            &credentials,
        )
        .await
        .unwrap();
    q("UPDATE sessions SET account_id=?2 WHERE id=?1")
        .bind(&session)
        .bind(&other)
        .execute(&s.backend)
        .await
        .unwrap();
    assert!(
        s.native_authority_fence(&session, &domains)
            .await
            .unwrap()
            .is_none()
    );
    q("UPDATE sessions SET account_id=?2 WHERE id=?1")
        .bind(&session)
        .bind(&account)
        .execute(&s.backend)
        .await
        .unwrap();
    let restored = s
        .native_authority_fence(&session, &domains)
        .await
        .unwrap()
        .unwrap();
    assert!(!after.same_published_source(&restored).unwrap());
    let foreign_scope = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    assert!(
        restored
            .versions()
            .matches(&foreign_scope.authority_versions(&[]).await.unwrap())
            .is_err()
    );
    foreign_scope.close().await;
    // The actual scoped epoch SELECT stays under the optional shape tracer cap.
    for dialect in [Dialect::Sqlite, Dialect::Postgres] {
        assert!(super::fence::fence_sql(dialect).len() <= 16 * 1024);
    }
    assert!(!format!("{restored:?}").contains(&session));
    s.close().await;
}
// Scoped user/domain settings never dirty the shared global policy object.
// The old resolver also reads address-targeted user values, so those have a
// distinct subject epoch instead of pretending the account UUID covers them.
async fn scoped_policy_contract(s: Store) {
    let (account, session) = seed(&s).await;
    let policy = AuthorityScope::new(AuthorityKind::Policy, "").unwrap();
    let routing = AuthorityScope::new(AuthorityKind::Routing, "").unwrap();
    let shared_before = s
        .authority_versions(&[policy.clone(), routing.clone()])
        .await
        .unwrap();
    let address = "person@authority.test";
    q("INSERT INTO personal_setting_value(owner_subject,key,value,updated_at) VALUES(?1,'fixture.preference','opaque-ciphertext',1)")
        .bind(&account).execute(&s.backend).await.unwrap();
    let owner = AuthorityScope::new(AuthorityKind::Subject, &account).unwrap();
    let first = current(&s, &owner).await;
    q("UPDATE personal_setting_value SET value='new-opaque-nonce' WHERE owner_subject=?1")
        .bind(&account)
        .execute(&s.backend)
        .await
        .unwrap();
    assert!(current(&s, &owner).await.version > first.version);
    q("INSERT INTO setting_value(scope_kind,scope_id,key,value,locked,updated_by,updated_at) VALUES('user',?1,'fixture.flag','opaque',0,'fixture',1)")
        .bind(address).execute(&s.backend).await.unwrap();
    let address_scope = AuthorityScope::new(AuthorityKind::Subject, address).unwrap();
    let address_before = current(&s, &address_scope).await;
    let old = s
        .native_authority_fence_with_subjects(
            &session,
            &["authority.test".into()],
            &[account.clone(), address.into()],
        )
        .await
        .unwrap()
        .unwrap();
    q("UPDATE setting_value SET locked=1 WHERE scope_kind='user' AND scope_id=?1")
        .bind(address)
        .execute(&s.backend)
        .await
        .unwrap();
    let now = s
        .native_authority_fence_with_subjects(
            &session,
            &["authority.test".into()],
            &[account.clone(), address.into()],
        )
        .await
        .unwrap()
        .unwrap();
    assert!(!old.same_published_source(&now).unwrap());
    assert!(current(&s, &address_scope).await.version > address_before.version);
    assert!(
        shared_before
            .matches(
                &s.authority_versions(&[policy.clone(), routing])
                    .await
                    .unwrap()
            )
            .unwrap()
    );
    let domain = AuthorityScope::new(AuthorityKind::Domain, "authority.test").unwrap();
    let domain_before = current(&s, &domain).await;
    q("INSERT INTO setting_value(scope_kind,scope_id,key,value,locked,updated_by,updated_at) VALUES('domain','authority.test','fixture.flag','opaque',0,'fixture',1)")
        .execute(&s.backend).await.unwrap();
    assert!(current(&s, &domain).await.version > domain_before.version);
    assert_eq!(current(&s, &policy).await, shared_before.stamps[&policy]);
    s.close().await;
}
#[tokio::test]
async fn sqlite_scoped_policy_changes_preserve_shared_epochs() {
    scoped_policy_contract(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn postgres_scoped_policy_changes_preserve_shared_epochs() {
    postgres(scoped_policy_contract).await;
}
async fn erasure_contract(s: Store) {
    let (account, session) = seed(&s).await;
    let before = s
        .native_authority_fence(&session, &["authority.test".into()])
        .await
        .unwrap()
        .unwrap();
    let principal = before.principal_id().to_owned();
    let account_scope = AuthorityScope::new(AuthorityKind::Account, &account).unwrap();
    let account_before = current(&s, &account_scope).await;
    q("INSERT INTO setting_value(scope_kind,scope_id,key,value,locked,updated_by,updated_at) VALUES('user','person@authority.test','fixture.erase','opaque',0,'fixture',1)")
        .execute(&s.backend).await.unwrap();
    let address_scope =
        AuthorityScope::new(AuthorityKind::Subject, "person@authority.test").unwrap();
    assert_eq!(
        s.authority_versions(&[address_scope.clone()])
            .await
            .unwrap()
            .stamps
            .len(),
        1
    );
    q("INSERT INTO personal_setting_value(owner_subject,key,value,updated_at) VALUES(?1,'fixture.erase','opaque-owner',1)")
        .bind(&account).execute(&s.backend).await.unwrap();
    let owner_scope = AuthorityScope::new(AuthorityKind::Subject, &account).unwrap();
    assert_eq!(
        s.authority_versions(&[owner_scope.clone()])
            .await
            .unwrap()
            .stamps
            .len(),
        1
    );
    let person = crate::erasure::PersonIds {
        address: "person@authority.test".into(),
        account_ids: vec![account.clone()],
        owners: vec![account.clone()],
        principals: vec![principal.clone()],
    };
    let (_, errors) = s.dsr_purge_person(&person).await;
    assert!(errors.is_empty(), "actual registry erasure must complete");
    let removed = [
        account_scope.clone(),
        address_scope,
        owner_scope,
        AuthorityScope::new(AuthorityKind::Principal, &principal).unwrap(),
    ];
    assert!(
        s.authority_versions(&removed)
            .await
            .unwrap()
            .stamps
            .is_empty()
    );
    for scope in &removed {
        assert_eq!(q("SELECT COUNT(*) AS n FROM authority_publication_outbox WHERE scope_kind=?1 AND scope_id=?2")
            .bind(scope.kind.as_str()).bind(scope.id.as_str()).fetch_one(&s.backend).await.unwrap().get_i64("n"),0);
    }
    assert!(
        s.native_authority_fence(&session, &["authority.test".into()])
            .await
            .unwrap()
            .is_none()
    );
    let creds = Credentials {
        username: "person@authority.test".into(),
        password: "fixture-new-secret".into(),
    };
    let new_account = s
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "mail.authority.test",
                port: 993,
                tls: "implicit",
                username: &creds.username,
                sync_policy_json: "{}",
            },
            &creds,
        )
        .await
        .unwrap();
    // No dependent rows exist yet for this newly created account. Reuse the old
    // exact ID through raw SQL to prove deleted epoch counters cannot ABA.
    q("UPDATE accounts SET id=?1 WHERE id=?2")
        .bind(&account)
        .bind(&new_account)
        .execute(&s.backend)
        .await
        .unwrap();
    let recreated = current(&s, &account_scope).await;
    assert_ne!(account_before.incarnation, recreated.incarnation);
    assert!(
        !before
            .versions()
            .matches(&s.authority_versions(&removed).await.unwrap())
            .unwrap()
    );
    s.close().await;
}
#[tokio::test]
async fn sqlite_actual_person_erasure_removes_scopes_and_recreation_cannot_aba() {
    erasure_contract(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn postgres_actual_person_erasure_removes_scopes_and_recreation_cannot_aba() {
    postgres(erasure_contract).await;
}
async fn postgres(run: impl AsyncFnOnce(Store)) {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        mw_test_gate::skip("MW_TEST_PG absent; actual authority PostgreSQL contract not executed");
        return;
    };
    use sqlx::Connection;
    let schema = format!("authority_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&mut admin)
        .await
        .unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    run(Store::open(scoped.as_str(), ServerKey::generate())
        .await
        .unwrap())
    .await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&mut admin)
        .await
        .unwrap();
}
#[tokio::test]
async fn sqlite_source_epoch_and_outbox_commit_or_rollback_together() {
    transaction_contract(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn postgres_source_epoch_and_outbox_commit_or_rollback_together() {
    postgres(transaction_contract).await;
}
#[tokio::test]
async fn sqlite_claim_cas_recreation_and_bounded_cleanup_prevent_aba() {
    claim_aba_contract(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn postgres_claim_cas_recreation_and_bounded_cleanup_prevent_aba() {
    postgres(claim_aba_contract).await;
}
#[tokio::test]
async fn sqlite_active_parent_fence_has_fresh_clocks_and_independent_policy_scope() {
    fence_contract(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn postgres_active_parent_fence_has_fresh_clocks_and_independent_policy_scope() {
    postgres(fence_contract).await;
}

#[tokio::test]
async fn encrypted_restart_cross_store_and_mail_writes_preserve_scope_contracts() {
    use crate::encryption::{DbKey, EncryptionConfig, EncryptionMode};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("owned-authority.db");
    let config = EncryptionConfig::with_key(EncryptionMode::Required, DbKey::generate());
    let key = ServerKey::generate();
    let reader = Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &config)
        .await
        .unwrap();
    let writer = Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &config)
        .await
        .unwrap();
    let (account, session) = seed(&reader).await;
    let domains = ["authority.test".into()];
    assert!(
        q("PRAGMA cipher_version")
            .fetch_opt_scalar_string(&reader.backend)
            .await
            .unwrap()
            .is_some_and(|version| !version.is_empty())
    );
    let before = reader
        .native_authority_fence(&session, &domains)
        .await
        .unwrap()
        .unwrap();
    // Raw independent Store writer changes a legacy security namespace.
    q("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value").bind("v6:admin:flags:person@authority.test").bind(r#"{"disabled":true}"#).execute(&writer.backend).await.unwrap();
    let fresh = reader
        .native_authority_fence(&session, &domains)
        .await
        .unwrap()
        .unwrap();
    assert!(!before.same_published_source(&fresh).unwrap());
    assert!(
        before
            .versions()
            .matches(&writer.authority_versions(&[]).await.unwrap())
            .is_err()
    );
    let mailbox = reader
        .upsert_mailbox(&crate::MailboxUpsert {
            account_id: &account,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 1,
            uidnext: 2,
            highestmodseq: 0,
            total: 1,
            unread: 1,
            parent_id: None,
        })
        .await
        .unwrap();
    let message = reader
        .upsert_message(&crate::MessageUpsert {
            account_id: &account,
            mailbox_id: &mailbox,
            uid: 1,
            uidvalidity: 1,
            message_id: None,
            thread_id: None,
            internaldate: None,
            size: 0,
            flags_json: "[]",
            envelope: None,
            blob_ref: None,
        })
        .await
        .unwrap();
    let mail_before = reader
        .native_authority_fence(&session, &domains)
        .await
        .unwrap()
        .unwrap();
    q("UPDATE messages SET flags_json=?1 WHERE stable_id=?2")
        .bind(r#"["Seen"]"#)
        .bind(&message)
        .execute(&writer.backend)
        .await
        .unwrap();
    writer
        .put_body(&account, b"owned raw source cache fixture")
        .await
        .unwrap();
    let mail_after = reader
        .native_authority_fence(&session, &domains)
        .await
        .unwrap()
        .unwrap();
    assert!(
        mail_before.same_published_source(&mail_after).unwrap(),
        "mail flags/body writes must not invalidate authority scopes"
    );
    let scope = AuthorityScope::new(AuthorityKind::Account, &account).unwrap();
    let persisted = current(&reader, &scope).await;
    writer.close().await;
    drop(writer);
    reader.close().await;
    drop(reader);
    let reopened = Store::open_with_encryption(path.to_str().unwrap(), key, &config)
        .await
        .unwrap();
    assert_eq!(persisted, current(&reopened, &scope).await);
    let after_restart = reopened
        .native_authority_fence(&session, &domains)
        .await
        .unwrap()
        .unwrap();
    assert!(
        fresh.versions().matches(after_restart.versions()).is_err(),
        "a process-local Store scope is not restored by a DB token"
    );
    reopened.close().await;
    drop(reopened);
    // No raw file opens until all SQLite workers have closed (POSIX lock safety).
    use std::io::Read;
    let mut file = std::fs::File::open(&path).unwrap();
    let mut header = [0u8; 16];
    file.read_exact(&mut header).unwrap();
    assert_ne!(&header, crate::encryption::SQLITE_MAGIC);
}

include!("native_authority_seed_tests.rs");
