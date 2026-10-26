use sqlx::{Connection, Executor};

const TABLE: &str = "CREATE TABLE egress_internal_allow (id TEXT PRIMARY KEY NOT NULL, traffic_class TEXT NOT NULL CHECK(traffic_class IN ('siem','webhooks','analytics','nextcloud')), destination TEXT NOT NULL, note TEXT NOT NULL DEFAULT '', created_by TEXT NOT NULL, created_at BIGINT NOT NULL)";
const SEED: &str = "INSERT INTO egress_internal_allow VALUES ('a','siem','10.1.0.0/16','audit','root',1),('b','webhooks','hooks.corp.test','hooks','operator',2),('c','analytics','stats.corp.test','counts','root',3),('d','nextcloud','cloud.corp.test','files','root',4)";
const SELECT: &str = "SELECT id,traffic_class,destination,note,created_by,created_at FROM egress_internal_allow ORDER BY id";
type Rows = Vec<(String, String, String, String, String, i64)>;

#[tokio::test]
async fn sqlite_populated_upgrade_preserves_rows_and_constraints() {
    let mut db = sqlx::SqliteConnection::connect("sqlite::memory:")
        .await
        .unwrap();
    db.execute(TABLE).await.unwrap();
    db.execute(SEED).await.unwrap();
    let before: Rows = sqlx::query_as(SELECT).fetch_all(&mut db).await.unwrap();
    let mut tx = db.begin().await.unwrap();
    sqlx::raw_sql(include_str!("../migrations/0058_plugin_egress.sql"))
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let after: Rows = sqlx::query_as(SELECT).fetch_all(&mut db).await.unwrap();
    assert_eq!(before, after);
    db.execute("INSERT INTO egress_internal_allow VALUES ('p','plugins','ews.corp.test','bridge','root',5)").await.unwrap();
    assert!(
        db.execute(
            "INSERT INTO egress_internal_allow VALUES ('p','plugins','duplicate','','root',6)"
        )
        .await
        .is_err()
    );
    assert!(db.execute("INSERT INTO egress_internal_allow VALUES ('bad','remote_content','ews.corp.test','','root',6)").await.is_err());
    let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&mut db)
        .await
        .unwrap();
    assert_eq!(integrity, "ok");
}

#[tokio::test]
async fn postgres_populated_upgrade_preserves_rows_and_constraints() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        eprintln!("SKIP live PostgreSQL: MW_TEST_PG not configured");
        return;
    };
    let mut db = sqlx::PgConnection::connect(&dsn).await.unwrap();
    let mut tx = db.begin().await.unwrap();
    tx.execute(TABLE.replace("CREATE TABLE", "CREATE TEMP TABLE").as_str())
        .await
        .unwrap();
    tx.execute(SEED).await.unwrap();
    let before: Rows = sqlx::query_as(SELECT).fetch_all(&mut *tx).await.unwrap();
    sqlx::raw_sql(include_str!("../migrations_pg/0058_plugin_egress.sql"))
        .execute(&mut *tx)
        .await
        .unwrap();
    let after: Rows = sqlx::query_as(SELECT).fetch_all(&mut *tx).await.unwrap();
    assert_eq!(before, after);
    tx.execute("INSERT INTO egress_internal_allow VALUES ('p','plugins','ews.corp.test','bridge','root',5)").await.unwrap();
    // Savepoints allow checking both constraint failures without aborting this transaction.
    for invalid in [
        "INSERT INTO egress_internal_allow VALUES ('p','plugins','duplicate','','root',6)",
        "INSERT INTO egress_internal_allow VALUES ('bad','remote_content','ews.corp.test','','root',6)",
    ] {
        tx.execute("SAVEPOINT constraint_test").await.unwrap();
        assert!(tx.execute(invalid).await.is_err());
        tx.execute("ROLLBACK TO SAVEPOINT constraint_test")
            .await
            .unwrap();
    }
    tx.rollback().await.unwrap();
}
