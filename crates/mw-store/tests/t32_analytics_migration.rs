use sqlx::{Connection, Executor, Row};

#[tokio::test]
async fn sqlite_analytics_upgrade_preserves_existing_scoped_destinations() {
    let mut db = sqlx::SqliteConnection::connect("sqlite::memory:")
        .await
        .unwrap();
    db.execute("CREATE TABLE egress_internal_allow (id TEXT PRIMARY KEY NOT NULL, traffic_class TEXT NOT NULL CHECK(traffic_class IN ('siem','webhooks')), destination TEXT NOT NULL, note TEXT NOT NULL DEFAULT '', created_by TEXT NOT NULL, created_at INTEGER NOT NULL)").await.unwrap();
    db.execute("INSERT INTO egress_internal_allow VALUES ('a','siem','10.1.0.0/16','audit','root',1),('b','webhooks','hooks.corp.test','hooks','operator',2)").await.unwrap();
    let mut tx = db.begin().await.unwrap();
    sqlx::raw_sql(include_str!("../migrations/0047_analytics_egress.sql"))
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let rows = sqlx::query("SELECT id,traffic_class,destination,note,created_by,created_at FROM egress_internal_allow ORDER BY id").fetch_all(&mut db).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].get::<String, _>("traffic_class"), "siem");
    assert_eq!(rows[0].get::<String, _>("note"), "audit");
    assert_eq!(rows[1].get::<String, _>("created_by"), "operator");
    db.execute("INSERT INTO egress_internal_allow VALUES ('c','analytics','stats.corp.test','counts','root',3)").await.unwrap();
    assert!(db.execute("INSERT INTO egress_internal_allow VALUES ('d','remote_content','stats.corp.test','','root',4)").await.is_err());
}

#[tokio::test]
async fn postgres_analytics_upgrade_preserves_existing_scoped_destinations() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        return;
    };
    let mut db = sqlx::PgConnection::connect(&dsn).await.unwrap();
    let mut tx = db.begin().await.unwrap();
    // A temporary table shadows the real table for this transaction only.
    tx.execute("CREATE TEMP TABLE egress_internal_allow (id TEXT PRIMARY KEY NOT NULL, traffic_class TEXT NOT NULL CHECK(traffic_class IN ('siem','webhooks')), destination TEXT NOT NULL, note TEXT NOT NULL DEFAULT '', created_by TEXT NOT NULL, created_at BIGINT NOT NULL) ON COMMIT DROP").await.unwrap();
    tx.execute("INSERT INTO egress_internal_allow VALUES ('a','siem','10.1.0.0/16','audit','root',1),('b','webhooks','hooks.corp.test','hooks','operator',2)").await.unwrap();
    sqlx::raw_sql(include_str!("../migrations_pg/0047_analytics_egress.sql"))
        .execute(&mut *tx)
        .await
        .unwrap();
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM egress_internal_allow WHERE traffic_class IN ('siem','webhooks')",
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert_eq!(count, 2);
    tx.execute("INSERT INTO egress_internal_allow VALUES ('c','analytics','stats.corp.test','counts','root',3)").await.unwrap();
    tx.rollback().await.unwrap();
}
