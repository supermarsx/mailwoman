//! Migration 0123: what moves the global settings revision. Session churn and
//! identity writes do not; the four administrator guard keys of the key-value
//! settings table do. A session's own identity changes are visible through
//! `Store::session_identity_witness` (the 0111 epochs) instead. Each contract
//! runs on SQLite (two handles on one file) and, with `MW_TEST_PG`, on
//! PostgreSQL (two handles on one database).
use crate::{Credentials, NewAccount, ServerKey, Store, backend::q, identity_principal::VerifiedPrimary};

const GUARD_KEYS: [&str; 4] = [
    "v6:admin:bans",
    "security.clientAccess.v1",
    "maintenance.mode.v1",
    "security.inbound-allowlist.v1",
];

async fn revision(store: &Store) -> i64 {
    store.settings_revision().await.unwrap().unwrap()
}

/// A primary engine session of a fresh account, bound to a verified principal
/// exactly as sign-in binds it.
async fn signed_in(store: &Store, account: Option<&str>, tag: &str) -> (String, String) {
    let user = format!("{tag}@scope.test");
    let credentials = Credentials { username: user.clone(), password: "fixture-only".into() };
    let account = match account {
        Some(account) => account.to_owned(),
        None => store
            .create_account(
                &NewAccount { kind: crate::AccountKind::Imap, host: "127.0.0.1", port: 9, tls: "none", username: &user, sync_policy_json: "{}" },
                &credentials,
            )
            .await
            .unwrap(),
    };
    let session = store.create_session(&account, &user, "engine", "engine", &credentials).await.unwrap();
    store.set_session_authentication(&session, "password").await.unwrap();
    store
        .bind_verified_primary(
            &session,
            VerifiedPrimary { account_id: &account, kind: "mailbox", authority: "scope-fixture", subject: &account },
            chrono::Utc::now().timestamp(),
        )
        .await
        .unwrap()
        .unwrap();
    (account, session)
}

async fn identity_writes_never_move_the_revision_and_guard_keys_do(writer: &Store, reader: &Store, tag: &str) {
    // Sign-in (session + session_principal; first sign-in also principal and
    // binding), a second sign-in of the same account, sign-out (cascade) and a
    // session_principal update: the revision stays where it is.
    let before = revision(reader).await;
    let (account, first) = signed_in(writer, None, &format!("{tag}-x")).await;
    let (_, second) = signed_in(writer, Some(&account), &format!("{tag}-x")).await;
    writer.delete_session(&second).await.unwrap();
    q("UPDATE session_principal SET auth_time=auth_time+1 WHERE session_id=?1").bind(&first).execute(&writer.backend).await.unwrap();
    q("UPDATE identity_principal SET state='disabled' WHERE id IN (SELECT principal_id FROM session_principal WHERE session_id=?1)").bind(&first).execute(&writer.backend).await.unwrap();
    q("UPDATE identity_principal SET state='active' WHERE id IN (SELECT principal_id FROM session_principal WHERE session_id=?1)").bind(&first).execute(&writer.backend).await.unwrap();
    writer.delete_session(&first).await.unwrap();
    assert_eq!(revision(reader).await, before, "{tag}: session churn and identity writes moved the revision");

    // Each guard key: insert, changed update and delete bump it once; an update
    // that keeps the value bumps nothing.
    for key in GUARD_KEYS {
        let _ = q("DELETE FROM settings WHERE key=?1").bind(key).execute(&writer.backend).await.unwrap();
        let start = revision(reader).await;
        writer.set_setting(key, "{\"t\":1}").await.unwrap();
        assert_eq!(revision(reader).await, start + 1, "{tag}: insert {key}");
        writer.set_setting(key, "{\"t\":1}").await.unwrap();
        assert_eq!(revision(reader).await, start + 1, "{tag}: same-value write {key}");
        writer.set_setting(key, "{\"t\":2}").await.unwrap();
        assert_eq!(revision(reader).await, start + 2, "{tag}: update {key}");
        q("DELETE FROM settings WHERE key=?1").bind(key).execute(&writer.backend).await.unwrap();
        assert_eq!(revision(reader).await, start + 3, "{tag}: delete {key}");
    }
    // Session watch state and any other key-value row bump nothing.
    let start = revision(reader).await;
    for key in [format!("sess-sec:v1:{tag}"), format!("sess-rot:v1:{tag}"), format!("t34-scope:{tag}")] {
        writer.set_setting(&key, "{}").await.unwrap();
        writer.set_setting(&key, "{\"changed\":1}").await.unwrap();
        q("DELETE FROM settings WHERE key=?1").bind(&key).execute(&writer.backend).await.unwrap();
    }
    assert_eq!(revision(reader).await, start, "{tag}: non-guard settings rows moved the revision");
}

async fn own_identity_changes_move_the_session_witness(writer: &Store, reader: &Store, tag: &str) {
    let (account, session) = signed_in(writer, None, &format!("{tag}-w")).await;
    let witness = || async { reader.session_identity_witness(&session).await.unwrap().unwrap() };
    let start = witness().await;
    // Another sign-in and sign-out of the same user, and anyone else's sign-in:
    // the witness (and the revision inside it) stays.
    let (_, other) = signed_in(writer, Some(&account), &format!("{tag}-w")).await;
    writer.delete_session(&other).await.unwrap();
    let (_, stranger) = signed_in(writer, None, &format!("{tag}-stranger")).await;
    assert_eq!(witness().await, start, "{tag}: another session's churn changed this witness");
    // Each change to THIS session's identity moves it, revision unchanged.
    for sql in [
        "UPDATE session_principal SET auth_time=auth_time+1 WHERE session_id=?1",
        "UPDATE identity_principal SET state='disabled' WHERE id IN (SELECT principal_id FROM session_principal WHERE session_id=?1)",
        "UPDATE identity_principal SET state='active' WHERE id IN (SELECT principal_id FROM session_principal WHERE session_id=?1)",
        "UPDATE identity_binding SET retired_at=1 WHERE principal_id IN (SELECT principal_id FROM session_principal WHERE session_id=?1)",
    ] {
        let before = witness().await;
        q(sql).bind(&session).execute(&writer.backend).await.unwrap();
        let after = witness().await;
        assert_eq!(after.0, before.0, "{tag}: identity write moved the revision: {sql}");
        assert_ne!(after.1, before.1, "{tag}: witness missed {sql}");
    }
    // Sign-out of this session is seen too.
    let before = witness().await;
    writer.delete_session(&session).await.unwrap();
    assert_ne!(witness().await.1, before.1, "{tag}: sign-out");
    writer.delete_session(&stranger).await.unwrap();
}

#[tokio::test]
async fn sqlite_settings_revision_scope() {
    let path = std::env::temp_dir().join(format!("mw-revision-scope-{}.db", uuid::Uuid::new_v4()));
    let name = path.to_string_lossy().into_owned();
    let key = ServerKey::generate();
    let writer = Store::open(&name, key.clone()).await.unwrap();
    let reader = Store::open(&name, key).await.unwrap();
    identity_writes_never_move_the_revision_and_guard_keys_do(&writer, &reader, "sqlite").await;
    own_identity_changes_move_the_session_witness(&writer, &reader, "sqlite").await;
    reader.close().await;
    writer.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn postgres_settings_revision_scope() {
    let Some(dsn) = std::env::var("MW_TEST_PG").ok().filter(|dsn| !dsn.is_empty()) else {
        eprintln!("[settings_revision_scope] MW_TEST_PG unset: Postgres leg not run");
        return;
    };
    let key = ServerKey::generate();
    let writer = Store::open(&dsn, key.clone()).await.unwrap();
    let reader = Store::open(&dsn, key).await.unwrap();
    let tag = format!("pg{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
    identity_writes_never_move_the_revision_and_guard_keys_do(&writer, &reader, &tag).await;
    own_identity_changes_move_the_session_witness(&writer, &reader, &tag).await;
    reader.close().await;
    writer.close().await;
}
