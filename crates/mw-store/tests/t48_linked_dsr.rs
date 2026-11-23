//! A person who linked another person's mailbox: the linked mailbox's sessions
//! (the source session, linked_internal=1, and the per-link context clone,
//! linked_internal=2) carry the mailbox's own principal and account id, so the
//! data map never reaches the other person's rows through them. Built with the
//! real link and context calls, not hand-written session rows.
use mw_store::{
    Credentials, ServerKey, Store, data_map,
    erasure::PersonIds,
    identity_principal::VerifiedPrimary,
};
use sqlx::{Row, SqlitePool, sqlite::SqliteConnectOptions};
use std::collections::BTreeMap;

#[path = "../src/test_db.rs"]
mod test_db;

const ALFA: &str = "acct-alfa-linked";
const BRAVO: &str = "acct-bravo-linked";

/// A password+factor session with a verified primary binding.
async fn signed_in(store: &Store, account: &str, realm: &str, subject: &str) -> String {
    let session = store
        .create_session(
            account,
            &format!("{subject}@fixture.invalid"),
            &format!("https://fixture.invalid/{realm}/session"),
            &format!("https://fixture.invalid/{realm}/api"),
            &Credentials { username: format!("{subject}@fixture.invalid"), password: "fixture".into() },
        )
        .await
        .unwrap();
    store.set_session_authentication(&session, "password+factor").await.unwrap();
    store
        .bind_verified_primary(&session, VerifiedPrimary { account_id: account, kind: "mailbox", authority: realm, subject }, 100)
        .await
        .unwrap()
        .unwrap();
    session
}

async fn principal_of(pool: &SqlitePool, session: &str) -> String {
    sqlx::query("SELECT principal_id FROM session_principal WHERE session_id=?1")
        .bind(session)
        .fetch_one(pool)
        .await
        .unwrap()
        .get("principal_id")
}

/// The person as the server resolves them for a request (dsr.rs subj_of and
/// dsr_data::person_ids): bound accounts, owners from those accounts' sessions,
/// principals from dsr_resolve_principals.
async fn person(store: &Store, account: &str, address: &str) -> PersonIds {
    let accounts = vec![account.to_string()];
    let mut owners = Vec::new();
    if let Ok(Some(o)) = store.native_personal_subject(account).await {
        owners.push(o);
    }
    for s in store.sessions_by_account(account).await.unwrap() {
        for o in [store.session_personal_subject(&s.id).await, store.session_storage_authority(&s.id).await]
            .into_iter()
            .flatten()
            .flatten()
        {
            owners.push(o);
        }
    }
    owners.sort();
    owners.dedup();
    let principals = store.dsr_resolve_principals(&accounts, address).await.unwrap();
    PersonIds { address: address.into(), account_ids: accounts, owners, principals }
}

async fn export_text(store: &Store, p: &PersonIds) -> String {
    let mut all = String::new();
    for section in data_map::TABLE_SECTIONS {
        for row in store.dsr_export_table_section(section, p).await.unwrap() {
            all.push_str(&row.to_string());
            all.push('\n');
        }
    }
    all
}

#[tokio::test]
async fn a_linked_mailbox_never_puts_its_owner_into_the_linking_persons_export_or_erasure() {
    let path = test_db::unique_file_path("t48-linked-dsr", "store.sqlite");
    let path = path.to_str().unwrap().to_string();
    let store = Store::open(&path, ServerKey::from_bytes(&[7u8; 32]).unwrap()).await.unwrap();
    let pool = SqlitePool::connect_with(SqliteConnectOptions::new().filename(&path).foreign_keys(false)).await.unwrap();

    // Alfa signs in; Bravo's mailbox is signed into separately and linked to Alfa.
    let alfa = signed_in(&store, ALFA, "realm-alfa", "alfa").await;
    let bravo_own = signed_in(&store, BRAVO, "realm-bravo", "bravo").await;
    let bravo_source = signed_in(&store, BRAVO, "realm-bravo", "bravo").await;
    let link = store.attach_verified_link(&alfa, &bravo_source, "Second box", 5, 200).await.unwrap().expect("link");
    let context = store.linked_context(&alfa, &link.id).await.unwrap().expect("linked context session");
    let kinds: BTreeMap<String, Option<i64>> = [
        (alfa.clone(), store.linked_session_kind(&alfa).await.unwrap()),
        (bravo_source.clone(), store.linked_session_kind(&bravo_source).await.unwrap()),
        (context.id.clone(), store.linked_session_kind(&context.id).await.unwrap()),
    ]
    .into();
    assert_eq!(kinds[&bravo_source], Some(1), "{kinds:?}");
    assert_eq!(kinds[&context.id], Some(2), "{kinds:?}");
    assert_eq!(context.account_id, BRAVO, "the context session is a session of the linked mailbox");

    let alfa_principal = principal_of(&pool, &alfa).await;
    let bravo_principal = principal_of(&pool, &bravo_own).await;
    assert_ne!(alfa_principal, bravo_principal);
    for s in [&bravo_source, &context.id] {
        assert_eq!(principal_of(&pool, s).await, bravo_principal, "linked sessions carry the mailbox's principal");
    }

    // Each person's own rows, keyed the way the data map finds them.
    for (acct, principal, tag) in [(ALFA, &alfa_principal, "alfa"), (BRAVO, &bravo_principal, "bravo")] {
        for sql in [
            format!("INSERT INTO tags(id,\"user\",name,color) VALUES('{tag}-tag','{acct}','{tag}-tag-name','blue')"),
            format!("INSERT INTO webhooks(id,account_id,url,secret_sealed,event_filter,created_at) VALUES('{tag}-hook','{acct}','https://hooks.example.test/{tag}',x'00','{{}}','2026-01-01')"),
            format!("INSERT INTO oidc_consent(principal_id,client_id,client_revision,scopes_json,updated_at) VALUES('{principal}','{tag}-client','r1','[]',1)"),
            format!("INSERT INTO quotas(account_id,bytes_limit,msg_limit) VALUES('{acct}',1,1)"),
        ] {
            sqlx::query(&sql).execute(&pool).await.unwrap_or_else(|e| panic!("{sql}: {e}"));
        }
    }

    let alfa_person = person(&store, ALFA, "alfa@fixture.invalid").await;
    let bravo_person = person(&store, BRAVO, "bravo@fixture.invalid").await;
    assert_eq!(alfa_person.principals, vec![alfa_principal.clone()], "Alfa resolves only Alfa's principal");
    assert_eq!(bravo_person.principals, vec![bravo_principal.clone()], "Bravo resolves only Bravo's principal");
    assert!(alfa_person.owners.iter().all(|o| !bravo_person.owners.contains(o)), "owner keys are not shared: {alfa_person:?} {bravo_person:?}");

    // Alfa's export: Alfa's rows and Alfa's link, nothing of Bravo's own.
    let text = export_text(&store, &alfa_person).await;
    assert!(text.contains("alfa-tag-name") && text.contains("alfa-client") && text.contains("Second box"), "{text}");
    for leaked in ["bravo", BRAVO, bravo_principal.as_str()] {
        assert!(!text.contains(leaked), "{leaked} is in Alfa's export: {text}");
    }

    // Alfa's erasure removes Alfa's link (it names Bravo's principal as the
    // child, so it is in Bravo's footprint too) and nothing else of Bravo's.
    let before = store.dsr_verify_person(&bravo_person).await;
    for t in ["tags", "webhooks", "oidc_consent", "quotas", "sessions", "identity_binding", "session_principal", "linked_account"] {
        assert!(before.contains_key(t), "Bravo's {t} rows are seeded: {before:?}");
    }
    let (_, errors) = store.dsr_purge_person(&alfa_person).await;
    assert!(errors.is_empty(), "{errors:?}");
    assert!(store.dsr_verify_person(&alfa_person).await.is_empty());
    let after = store.dsr_verify_person(&bravo_person).await;
    let mut expected = before.clone();
    expected.remove("linked_account");
    assert_eq!(after, expected, "Alfa's erasure changed Bravo's rows other than Alfa's link");
    assert!(store.get_session(&bravo_own).await.is_ok(), "Bravo's own session survives");
    assert_eq!(export_text(&store, &bravo_person).await.matches("bravo-").count() >= 3, true);
}
