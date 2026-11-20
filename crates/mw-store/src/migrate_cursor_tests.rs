use super::*;

#[tokio::test]
async fn postgres_cursor_copy_replaces_seed_with_saved_sqlite_position() {
    let Ok(dsn) = std::env::var("MW_BG_TEST_PG") else {
        eprintln!("SKIP: MW_BG_TEST_PG absent");
        return;
    };
    let source = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    q("UPDATE change_log_maintenance_cursor SET account_id='deleted-fixture-account',type='Mailbox',updated_at=123 WHERE id=1")
        .execute(&source.backend).await.unwrap();
    let target = Store::open_postgres(&dsn, crate::ServerKey::generate())
        .await
        .unwrap();
    let previous =
        q("SELECT account_id,type,updated_at FROM change_log_maintenance_cursor WHERE id=1")
            .fetch_one(&target.backend)
            .await
            .unwrap();
    let spec = TABLES
        .iter()
        .find(|table| table.name == "change_log_maintenance_cursor")
        .unwrap();
    let mut tx = target.backend.begin().await.unwrap();
    let rows = copy_table(&source.backend, &mut tx, spec).await.unwrap();
    assert_eq!(rows, 1);
    tx.commit().await.unwrap();
    let copied =
        q("SELECT account_id,type,updated_at FROM change_log_maintenance_cursor WHERE id=1")
            .fetch_one(&target.backend)
            .await
            .unwrap();
    assert_eq!(copied.get_string("account_id"), "deleted-fixture-account");
    assert_eq!(copied.get_string("type"), "Mailbox");
    assert_eq!(copied.get_i64("updated_at"), 123);
    assert_eq!(
        q("SELECT COUNT(*) FROM change_log_maintenance_cursor")
            .fetch_scalar_i64(&target.backend)
            .await
            .unwrap(),
        1
    );
    assert!(
        q("INSERT INTO change_log_maintenance_cursor VALUES(NULL,'','',0)")
            .execute(&target.backend)
            .await
            .is_err()
    );
    q("UPDATE change_log_maintenance_cursor SET account_id=?1,type=?2,updated_at=?3 WHERE id=1")
        .bind(previous.get_string("account_id"))
        .bind(previous.get_string("type"))
        .bind(previous.get_i64("updated_at"))
        .execute(&target.backend)
        .await
        .unwrap();
    target.close().await;
    source.close().await;
}
