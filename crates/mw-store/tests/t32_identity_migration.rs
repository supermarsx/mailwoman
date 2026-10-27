use mw_store::{Credentials, ServerKey, Store, identity_principal::VerifiedPrimary};
#[path = "../src/test_db.rs"]
mod test_db;

async fn populated_copy(destination: Store, key: ServerKey) {
    let path = test_db::unique_file_path("principal-copy", "source.sqlite");
    let source = Store::open(path.to_str().unwrap(), key).await.unwrap();
    let session = source
        .create_session(
            "verified-account",
            "display@example.test",
            "engine",
            "engine",
            &Credentials {
                username: "backend-user".into(),
                password: "synthetic".into(),
            },
        )
        .await
        .unwrap();
    source
        .set_session_authentication(&session, "password+factor")
        .await
        .unwrap();
    let proof = || VerifiedPrimary {
        account_id: "verified-account",
        kind: "mailbox",
        authority: "configured-realm",
        subject: "backend-subject",
    };
    let principal = source
        .bind_verified_primary(&session, proof(), 123)
        .await
        .unwrap()
        .unwrap();
    let retired_session = source
        .create_session(
            "retired-account",
            "retired@example.test",
            "engine",
            "engine",
            &Credentials {
                username: "retired".into(),
                password: "synthetic".into(),
            },
        )
        .await
        .unwrap();
    source
        .set_session_authentication(&retired_session, "password")
        .await
        .unwrap();
    let retired_proof = || VerifiedPrimary {
        account_id: "retired-account",
        kind: "mailbox",
        authority: "configured-realm",
        subject: "retired-subject",
    };
    let retired = source
        .bind_verified_primary(&retired_session, retired_proof(), 124)
        .await
        .unwrap()
        .unwrap();
    source
        .retire_principal(&retired.principal_id, 125)
        .await
        .unwrap();
    let personal_session = source
        .create_session(
            "1",
            "same@example.test",
            "https://mail.test/a/session",
            "https://mail.test/a/api",
            &Credentials {
                username: "same".into(),
                password: "synthetic".into(),
            },
        )
        .await
        .unwrap();
    source
        .set_session_authentication(&personal_session, "header-auth")
        .await
        .unwrap();
    source
        .bind_verified_primary(
            &personal_session,
            VerifiedPrimary {
                account_id: "1",
                kind: "header",
                authority: "trusted-header",
                subject: "same",
            },
            123,
        )
        .await
        .unwrap()
        .unwrap();
    let owner = source
        .session_personal_subject(&personal_session)
        .await
        .unwrap()
        .unwrap();
    source
        .upsert_personal_signature(&mw_store::SignatureRow {
            account_id: owner.clone(),
            name: "copy".into(),
            body: "plain".into(),
            html: Some("<b>rich</b>".into()),
            text: Some("independent plain".into()),
            text_edited: true,
            is_default: true,
            rule_json: "{}".into(),
            updated_at: String::new(),
        })
        .await
        .unwrap();
    source
        .upsert_personal_saved_search(&mw_store::SavedSearchRow {
            id: "copied-search".into(),
            user: owner.clone(),
            name: "Copied".into(),
            query_json: "{}".into(),
            as_folder: true,
        })
        .await
        .unwrap();
    let report = destination
        .migrate_from_sqlite(path.to_str().unwrap())
        .await
        .unwrap();
    for (table, count) in [
        ("identity_principal", 3),
        ("identity_binding", 3),
        ("session_principal", 2),
    ] {
        assert_eq!(
            report
                .tables
                .iter()
                .find(|(name, _)| name == table)
                .unwrap()
                .1,
            count
        );
    }
    assert_eq!(
        destination.primary_principal(&session).await.unwrap(),
        Some(principal.clone())
    );
    assert_eq!(
        destination
            .get_session(&session)
            .await
            .unwrap()
            .credentials
            .password,
        "synthetic"
    );
    assert_eq!(
        Some(owner.clone()),
        destination
            .session_personal_subject(&personal_session)
            .await
            .unwrap()
    );
    let signatures = destination.list_personal_signatures(&owner).await.unwrap();
    assert_eq!(signatures.len(), 1);
    assert_eq!(signatures[0].html.as_deref(), Some("<b>rich</b>"));
    assert_eq!(signatures[0].text.as_deref(), Some("independent plain"));
    assert!(signatures[0].text_edited);
    let searches = destination
        .list_personal_saved_searches(&owner)
        .await
        .unwrap();
    assert_eq!(searches.len(), 1);
    assert!(searches[0].as_folder);
    assert_eq!(searches[0].id, "copied-search");
    let fresh = destination
        .create_session(
            "verified-account",
            "display@example.test",
            "engine",
            "engine",
            &Credentials {
                username: "backend-user".into(),
                password: "synthetic".into(),
            },
        )
        .await
        .unwrap();
    destination
        .set_session_authentication(&fresh, "password")
        .await
        .unwrap();
    assert_eq!(
        destination
            .bind_verified_primary(&fresh, proof(), 200)
            .await
            .unwrap()
            .unwrap()
            .principal_id,
        principal.principal_id
    );
    let replacement = destination
        .create_session(
            "retired-account",
            "retired@example.test",
            "engine",
            "engine",
            &Credentials {
                username: "retired".into(),
                password: "synthetic".into(),
            },
        )
        .await
        .unwrap();
    destination
        .set_session_authentication(&replacement, "password")
        .await
        .unwrap();
    assert!(
        destination
            .bind_verified_primary(&replacement, retired_proof(), 201)
            .await
            .unwrap()
            .is_none(),
        "migration preserves non-reassignment tombstones"
    );
}

#[tokio::test]
async fn populated_principal_copy_sqlite() {
    let key = ServerKey::generate();
    populated_copy(Store::open_in_memory(key.clone()).await.unwrap(), key).await;
}

#[tokio::test]
async fn populated_principal_copy_postgres() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        return;
    };
    // A new database avoids truncating any shared live test fixture.
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    let name = format!("mw_principal_copy_{:032x}", rand::random::<u128>());
    sqlx::query(&format!("CREATE DATABASE {name}"))
        .execute(&admin)
        .await
        .unwrap();
    let mut url = url::Url::parse(&dsn).unwrap();
    url.set_path(&format!("/{name}"));
    let key = ServerKey::generate();
    populated_copy(Store::open(url.as_str(), key.clone()).await.unwrap(), key).await;
    // Only the uniquely named database created by this test is removed.
    sqlx::query(&format!("DROP DATABASE {name} WITH (FORCE)"))
        .execute(&admin)
        .await
        .unwrap();
}
