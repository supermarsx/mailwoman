// t62-n15 (review finding F4): the projection is trusted only while its triggers
// are the ones of the last migration or clean audit (0134 stamp), and on
// PostgreSQL they fire for replica-role writes too (ENABLE ALWAYS).
use super::*;
use crate::ServerKey;

fn leak(sql: String) -> &'static str {
    Box::leak(sql.into_boxed_str())
}
async fn exec(s: &Store, sql: &str) {
    q(leak(sql.to_owned())).execute(&s.backend).await.unwrap();
}
async fn seed(s: &Store) {
    let creds = if s.backend.dialect() == Dialect::Sqlite {
        "x'00'"
    } else {
        "'\\x00'::bytea"
    };
    exec(s, &format!("INSERT INTO accounts(id,kind,host,port,tls,username,sealed_creds) VALUES('st','imap','h',993,'tls','st',{creds})")).await;
    exec(
        s,
        "INSERT INTO mailboxes(id,account_id,name,role) VALUES('st_inbox','st','INBOX','inbox')",
    )
    .await;
    exec(s, "INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,flags_json) VALUES('st1','st','st_inbox',1,1,'[]')").await;
}

#[tokio::test]
async fn sqlite_replaced_projection_trigger_is_refused_until_a_clean_audit() {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    seed(&s).await;
    assert!(
        s.message_projection_installed().await.unwrap(),
        "fresh install"
    );
    // Same name, another body: the stamp no longer matches.
    let original: String =
        q("SELECT sql FROM sqlite_master WHERE name='message_projection_update'")
            .fetch_one(&s.backend)
            .await
            .unwrap()
            .get_string("sql");
    exec(&s, "DROP TRIGGER message_projection_update").await;
    exec(
        &s,
        "CREATE TRIGGER message_projection_update AFTER UPDATE ON messages BEGIN SELECT 1; END",
    )
    .await;
    assert!(
        !s.message_projection_installed().await.unwrap(),
        "replaced trigger"
    );
    exec(
        &s,
        "UPDATE messages SET flags_json='[\"Seen\"]' WHERE stable_id='st1'",
    )
    .await;
    assert!(
        !s.restamp_message_projection().await.unwrap(),
        "the audit finds the missed write"
    );
    assert!(!s.message_projection_installed().await.unwrap());
    // The right trigger back, the projection repaired by a write through it.
    exec(&s, "DROP TRIGGER message_projection_update").await;
    exec(&s, &original).await;
    exec(
        &s,
        "UPDATE messages SET flags_json='[\"Seen\",\"Flagged\"]' WHERE stable_id='st1'",
    )
    .await;
    assert!(
        s.restamp_message_projection().await.unwrap(),
        "clean audit re-stamps"
    );
    assert!(s.message_projection_installed().await.unwrap());
    s.close().await;
}

#[tokio::test]
async fn postgres_projection_triggers_fire_always_and_a_toggle_needs_a_clean_audit() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        mw_test_gate::skip("MW_TEST_PG absent; projection stamp test not executed".to_owned());
        return;
    };
    use sqlx::Connection;
    let schema = format!("stamp_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&mut admin)
        .await
        .unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    let s = Store::open(scoped.as_str(), ServerKey::generate())
        .await
        .unwrap();
    seed(&s).await;
    assert!(
        s.message_projection_installed().await.unwrap(),
        "fresh install"
    );
    let enabled = q("SELECT string_agg(tgenabled::text,'' ORDER BY tgname) AS e FROM pg_trigger WHERE tgname LIKE 'message_projection_%' AND tgrelid='messages'::regclass")
        .fetch_one(&s.backend)
        .await
        .unwrap()
        .get_string("e");
    assert_eq!(enabled, "AAA", "ENABLE ALWAYS");

    // A replica-role write (logical-replication apply, bulk tools) is projected.
    let mut conn = sqlx::PgConnection::connect(scoped.as_str()).await.unwrap();
    sqlx::raw_sql("BEGIN;SET LOCAL session_replication_role=replica;UPDATE messages SET flags_json='[\"Seen\"]' WHERE stable_id='st1';COMMIT")
        .execute(&mut conn)
        .await
        .unwrap();
    assert!(
        s.audit_message_projection(None).await.unwrap().is_clean(),
        "replica-role write projected"
    );
    assert!(s.message_projection_installed().await.unwrap());

    // Disable, write, enable again: the triggers are back, the tables are not.
    sqlx::raw_sql("ALTER TABLE messages DISABLE TRIGGER message_projection_update;UPDATE messages SET flags_json='[\"Flagged\"]' WHERE stable_id='st1';ALTER TABLE messages ENABLE ALWAYS TRIGGER message_projection_update")
        .execute(&mut conn)
        .await
        .unwrap();
    assert!(
        !s.message_projection_installed().await.unwrap(),
        "a toggle changes the stamp"
    );
    assert!(
        !s.restamp_message_projection().await.unwrap(),
        "the audit finds the missed write"
    );
    assert!(!s.message_projection_installed().await.unwrap());
    // Repaired by a write through the triggers; a clean audit re-stamps.
    exec(
        &s,
        "UPDATE messages SET flags_json='[]' WHERE stable_id='st1'",
    )
    .await;
    assert!(s.restamp_message_projection().await.unwrap());
    assert!(s.message_projection_installed().await.unwrap());

    // Enabled in origin mode only (the pre-0134 state) is not trusted either.
    exec(
        &s,
        "ALTER TABLE messages ENABLE TRIGGER message_projection_insert",
    )
    .await;
    assert!(
        !s.message_projection_installed().await.unwrap(),
        "origin-only is not enough"
    );
    assert!(
        !s.restamp_message_projection().await.unwrap(),
        "a clean audit does not accept origin-only"
    );
    exec(
        &s,
        "ALTER TABLE messages ENABLE ALWAYS TRIGGER message_projection_insert",
    )
    .await;
    assert!(s.restamp_message_projection().await.unwrap());

    drop(conn);
    s.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&mut admin)
        .await
        .unwrap();
}
