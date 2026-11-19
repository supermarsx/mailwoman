//! Read-only database measurements. Payload scans are explicit and time bounded.
use crate::{Store, StoreError, backend::Backend};
use serde_json::{Value, json};
use sqlx::Row;
use std::time::{Duration, Instant};

/// SQLCipher returns page_size as text; ordinary SQLite returns an integer.
pub(crate) async fn sqlite_pragma_i64(
    connection: &mut sqlx::SqliteConnection,
    statement: &'static str,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(statement).fetch_one(connection).await?;
    row.try_get::<i64, _>(0).or_else(|_| {
        let value = row.try_get::<String, _>(0)?;
        value
            .parse::<i64>()
            .map_err(|error| sqlx::Error::Decode(Box::new(error)))
    })
}

const BODY_USAGE: &str = "SELECT COUNT(*) AS body_count, CAST(COALESCE(SUM(length(sealed_bytes)),0) AS BIGINT) AS total_bytes,
 CAST(COALESCE(SUM(CASE WHEN blob_ref NOT IN (SELECT blob_ref FROM messages WHERE blob_ref IS NOT NULL) THEN 1 ELSE 0 END),0) AS BIGINT) AS orphan_count,
 CAST(COALESCE(SUM(CASE WHEN blob_ref NOT IN (SELECT blob_ref FROM messages WHERE blob_ref IS NOT NULL) THEN length(sealed_bytes) ELSE 0 END),0) AS BIGINT) AS orphan_bytes FROM bodies";

impl Store {
    /// The quick report does not scan message/body tables or read credentials.
    pub async fn database_observability(&self, detailed: bool) -> Result<Value, StoreError> {
        let basic = self.store_diagnostics().await?;
        let threshold = self
            .get_setting_policy("observability.database.growthMiBPerMinute")
            .await
            .ok()
            .flatten()
            .and_then(|p| p.global_value)
            .and_then(|v| serde_json::from_str::<u64>(&v).ok())
            .filter(|v| (1..=1048576).contains(v))
            .unwrap_or(64);
        let mut result = json!({
            "observedAt": chrono::Utc::now().to_rfc3339(),
            "database": self.database_info().await?,
            "driver": "SQLx",
            "diagnostics": basic,

            "migrations": self.migration_detail().await?,
            "details": null,
        });
        macro_rules! pool_info {
            ($pool:expr) => {{
                let options = $pool.options();
                json!({"open":$pool.size(),"idle":$pool.num_idle(),
                    "busy":$pool.size().saturating_sub($pool.num_idle() as u32),
                    "minimum":options.get_min_connections(),"maximum":options.get_max_connections(),
                    "acquireTimeoutMs":options.get_acquire_timeout().as_millis() as u64,
                    "idleTimeoutMs":options.get_idle_timeout().map(|v|v.as_millis() as u64),
                    "maxLifetimeMs":options.get_max_lifetime().map(|v|v.as_millis() as u64),
                    "closed":$pool.is_closed()})
            }};
        }
        match &self.backend {
            Backend::Sqlite(live) => {
                let pinned = live.pin().await;
                let pool = pinned.pool();
                result["pool"] = pool_info!(pool);
                let mut connection = pool.acquire().await?;
                let mut pragmas = serde_json::Map::new();
                for (name, statement) in [
                    ("pageSize", "PRAGMA page_size"),
                    ("pageCount", "PRAGMA page_count"),
                    ("freePages", "PRAGMA freelist_count"),
                    ("autoVacuum", "PRAGMA auto_vacuum"),
                    ("synchronous", "PRAGMA synchronous"),
                    ("busyTimeoutMs", "PRAGMA busy_timeout"),
                    ("cacheSize", "PRAGMA cache_size"),
                    ("foreignKeys", "PRAGMA foreign_keys"),
                    ("mmapBytes", "PRAGMA mmap_size"),
                ] {
                    if let Ok(value) = sqlite_pragma_i64(&mut connection, statement).await {
                        pragmas.insert(name.into(), json!(value));
                    }
                }
                let page = pragmas.get("pageSize").and_then(Value::as_u64).unwrap_or(0);
                let pages = pragmas
                    .get("pageCount")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                let free = pragmas
                    .get("freePages")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                pragmas.insert("freeBytes".into(), json!(free.saturating_mul(page)));
                pragmas.insert(
                    "occupiedBytes".into(),
                    json!(pages.saturating_sub(free).saturating_mul(page)),
                );
                result["storage"] = Value::Object(pragmas);
                drop(connection);
            }
            Backend::Postgres(pool) => {
                result["pool"] = pool_info!(pool);
                result["storage"] = json!({"tls":sqlx::query_scalar::<_,bool>(
                    "SELECT ssl FROM pg_stat_ssl WHERE pid=pg_backend_pid()"
                ).fetch_optional(pool).await.ok().flatten()});
            }
        }
        result["growth"] = crate::database_growth::observe(
            self.instance_id(),
            result["diagnostics"]["dbBytes"].as_u64(),
            result["diagnostics"]["walBytes"].as_u64(),
            result["storage"]["occupiedBytes"].as_u64(),
            threshold * 1024 * 1024,
        );
        if detailed {
            // Keep the cleanup running if an HTTP client disconnects. The SQLite
            // progress callback interrupts work and is removed before pool reuse.
            let backend = self.backend.clone();
            result["details"] = tokio::spawn(async move {
                match backend {
                    Backend::Sqlite(pool) => sqlite_details(pool, Duration::from_secs(30)).await,
                    Backend::Postgres(pool) => postgres_details(pool).await,
                }
            })
            .await
            .map_err(|error| StoreError::Corrupt(format!("database inspection task: {error}")))??;
        }
        if detailed {
            crate::database_growth::observe_tables(self.instance_id(), &mut result["details"]);
        }
        Ok(result)
    }
}

async fn sqlite_details(pool: crate::sqlite_live::LiveSqlite, budget: Duration) -> Result<Value, StoreError> {
    let pinned = pool.pin().await;
    sqlite_details_with_physical_budget(pinned.pool().clone(), budget, Duration::from_secs(8)).await
}
async fn inspection_deadline(
    connection: &mut sqlx::SqliteConnection,
    deadline: Instant,
) -> Result<(), sqlx::Error> {
    connection
        .lock_handle()
        .await?
        .set_progress_handler(100, move || Instant::now() < deadline);
    Ok(())
}
fn quoted(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
async fn sqlite_details_with_physical_budget(
    pool: sqlx::SqlitePool,
    budget: Duration,
    physical_budget: Duration,
) -> Result<Value, StoreError> {
    let mut connection = pool.acquire().await?;
    let deadline = Instant::now() + budget;
    inspection_deadline(&mut connection, deadline).await?;
    let work=async {
        let names=sqlx::query_scalar::<_,String>("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name LIMIT 513").fetch_all(&mut *connection).await?;
        let mut notes=Vec::new();
        let limited=names.len()>512;
        let mut bodies=json!({"count":null,"bytes":null,"orphanCount":null,"orphanBytes":null,"liveCount":null,"liveBytes":null});
        // BLOB byte lengths are read from record metadata, never transferred or decrypted as mail.
        inspection_deadline(&mut connection,deadline.min(Instant::now()+Duration::from_secs(7))).await?;
        match sqlx::query("SELECT COUNT(*) AS n,CAST(COALESCE(SUM(octet_length(sealed_bytes)),0) AS BIGINT) AS bytes FROM bodies").fetch_one(&mut *connection).await {
            Ok(row)=>{bodies["count"]=json!(row.try_get::<i64,_>("n")?);bodies["bytes"]=json!(row.try_get::<i64,_>("bytes")?);},
            Err(_)=>notes.push("body-totals-unavailable"),
        }
        let logical_deadline=deadline.min(Instant::now()+Duration::from_secs(12));
        let mut tables=Vec::new();
        for name in names.into_iter().take(512){
            let mut table=json!({"name":name,"bytes":null,"payloadBytes":null,"indexBytes":null,"rows":null,"rowsEstimated":false,"columns":[]});
            if name=="bodies"&&!bodies["bytes"].is_null(){
                table["payloadBytes"]=bodies["bytes"].clone();table["rows"]=bodies["count"].clone();
                table["columns"]=json!([{"name":"sealed_bytes","bytes":bodies["bytes"]}]);
            }else if Instant::now()<logical_deadline{
                inspection_deadline(&mut connection,logical_deadline.min(Instant::now()+Duration::from_millis(750))).await?;
                let metadata=match sqlx::query("SELECT name,type,hidden FROM pragma_table_xinfo(?1)").bind(&name).fetch_all(&mut *connection).await{
                    Ok(rows)=>rows,Err(_)=>{if !notes.contains(&"payload-scan-limited"){notes.push("payload-scan-limited");}tables.push(table);continue;}
                };
                let columns:Vec<String>=metadata.into_iter().filter_map(|row|{
                    let kind=row.try_get::<String,_>("type").ok()?.to_ascii_uppercase();
                    if row.try_get::<i64,_>("hidden").ok()?!=0||!(kind.contains("TEXT")||kind.contains("BLOB")||kind.contains("CHAR")||kind.is_empty()){return None}
                    row.try_get::<String,_>("name").ok()
                }).take(64).collect();
                let expressions=columns.iter().enumerate().map(|(i,col)|format!(",CAST(COALESCE(SUM(octet_length({})),0) AS BIGINT) AS c{i}",quoted(col))).collect::<String>();
                let statement=format!("SELECT COUNT(*) AS n{expressions} FROM {}",quoted(&name));
                match sqlx::query(&statement).fetch_one(&mut *connection).await{
                    Ok(row)=>{
                        let values:Vec<Value>=columns.iter().enumerate().map(|(i,col)|Ok(json!({"name":col,"bytes":row.try_get::<i64,_>(format!("c{i}").as_str())?}))).collect::<Result<_,sqlx::Error>>()?;
                        let total:i64=values.iter().filter_map(|v|v["bytes"].as_i64()).sum();
                        table["rows"]=json!(row.try_get::<i64,_>("n")?);table["payloadBytes"]=json!(total);table["columns"]=json!(values);
                    },
                    Err(_)=>{if !notes.contains(&"payload-scan-limited"){notes.push("payload-scan-limited");}},
                }
            }else if !notes.contains(&"payload-scan-limited"){notes.push("payload-scan-limited");}
            tables.push(table);
        }
        // Separate orphan classification: a slow join must not hide total body storage.
        inspection_deadline(&mut connection,deadline.min(Instant::now()+Duration::from_secs(5))).await?;
        match sqlx::query("SELECT COUNT(*) AS n,CAST(COALESCE(SUM(octet_length(sealed_bytes)),0) AS BIGINT) AS bytes FROM bodies WHERE blob_ref NOT IN (SELECT blob_ref FROM messages WHERE blob_ref IS NOT NULL)").fetch_one(&mut *connection).await{
            Ok(row)=>{
                let n=row.try_get::<i64,_>("n")?;let bytes=row.try_get::<i64,_>("bytes")?;
                bodies["orphanCount"]=json!(n);bodies["orphanBytes"]=json!(bytes);
                bodies["liveCount"]=bodies["count"].as_i64().map(|v|json!(v-n)).unwrap_or(Value::Null);
                bodies["liveBytes"]=bodies["bytes"].as_i64().map(|v|json!(v-bytes)).unwrap_or(Value::Null);
            },Err(_)=>notes.push("body-usage-unavailable"),
        }
        // Exact-name constraints avoid rescanning every B-tree once per table.
        // Keep page-mode SUM: aggregate dbstat can spend a long time inside one VM
        // step, delaying the progress callback on huge overflow chains.
        tables.sort_by_key(|v|std::cmp::Reverse(v["payloadBytes"].as_i64().unwrap_or(-1)));
        let physical_deadline=deadline.min(Instant::now()+physical_budget);
        for table in &mut tables{
            if Instant::now()>=physical_deadline{if !notes.contains(&"physical-scan-limited"){notes.push("physical-scan-limited");}break}
            let name=table["name"].as_str().unwrap_or_default().to_owned();
            inspection_deadline(&mut connection,physical_deadline).await?;
            let objects=match sqlx::query("SELECT name,type FROM sqlite_schema WHERE tbl_name=?1 AND type IN ('table','index') ORDER BY type").bind(&name).fetch_all(&mut *connection).await{
                Ok(rows)=>rows,Err(_)=>{if !notes.contains(&"physical-scan-limited"){notes.push("physical-scan-limited");}break;}
            };
            let mut total=0i64;let mut indexes=0i64;let mut complete=true;let mut index_complete=true;
            for object in objects{
                inspection_deadline(&mut connection,physical_deadline.min(Instant::now()+Duration::from_millis(250))).await?;
                let object_name=object.try_get::<String,_>("name")?;let is_index=object.try_get::<String,_>("type")?=="index";
                match sqlx::query_scalar::<_,i64>("SELECT COALESCE(SUM(pgsize),0) FROM dbstat WHERE name=?1").bind(object_name).fetch_one(&mut *connection).await{
                    Ok(bytes)=>{total+=bytes;if is_index{indexes+=bytes}},Err(_)=>{complete=false;if is_index{index_complete=false}},
                }
            }
            if complete{table["bytes"]=json!(total)}else if !notes.contains(&"physical-scan-limited"){notes.push("physical-scan-limited");}
            if index_complete{table["indexBytes"]=json!(indexes)}
        }
        Ok::<_,StoreError>(json!({"observedAt":chrono::Utc::now().to_rfc3339(),"tables":tables,"bodies":bodies,"limited":limited||!notes.is_empty(),"notes":notes}))
    }.await;
    connection.lock_handle().await?.remove_progress_handler();
    work
}

async fn postgres_details(pool: sqlx::PgPool) -> Result<Value, StoreError> {
    let mut transaction = pool.begin().await?;
    sqlx::query("SET TRANSACTION READ ONLY")
        .execute(&mut *transaction)
        .await?;
    sqlx::query("SET LOCAL statement_timeout='30s'")
        .execute(&mut *transaction)
        .await?;
    let rows = sqlx::query("SELECT relname,pg_table_size(relid) AS data_bytes,pg_indexes_size(relid) AS index_bytes,n_live_tup,n_dead_tup FROM pg_stat_user_tables WHERE schemaname=current_schema() ORDER BY pg_total_relation_size(relid) DESC LIMIT 513")
        .fetch_all(&mut *transaction).await?;
    let limited = rows.len() > 512;
    let tables: Vec<Value> = rows.into_iter().take(512).map(|row| Ok(json!({
        "name":row.try_get::<String,_>("relname")?,
        "bytes":row.try_get::<i64,_>("data_bytes")?.saturating_add(row.try_get::<i64,_>("index_bytes")?),
        "rows":row.try_get::<i64,_>("n_live_tup")?,"rowsEstimated":true,"deadRows":row.try_get::<i64,_>("n_dead_tup")?
    }))).collect::<Result<_,sqlx::Error>>()?;
    let row = sqlx::query(BODY_USAGE).fetch_one(&mut *transaction).await?;
    let bodies = json!({"count":row.try_get::<i64,_>("body_count")?,"bytes":row.try_get::<i64,_>("total_bytes")?,
        "orphanCount":row.try_get::<i64,_>("orphan_count")?,"orphanBytes":row.try_get::<i64,_>("orphan_bytes")?});
    transaction.commit().await?;
    Ok(
        json!({"observedAt":chrono::Utc::now().to_rfc3339(),"tables":tables,"bodies":bodies,"limited":limited,"notes":[]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccountKind, Credentials, NewAccount, ServerKey};

    #[tokio::test]
    async fn reports_payloads_and_old_bodies_even_without_physical_scan_time() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE bodies(blob_ref TEXT PRIMARY KEY,sealed_bytes BLOB);CREATE TABLE messages(blob_ref TEXT);CREATE TABLE sample_log(detail TEXT);INSERT INTO bodies VALUES('live',zeroblob(1048576)),('old',zeroblob(2097152));INSERT INTO messages VALUES('live');INSERT INTO sample_log VALUES('DO-NOT-EXPOSE-SENSITIVE-BOUND-VALUE')").execute(&pool).await.unwrap();
        let report = sqlite_details_with_physical_budget(
            pool.clone(),
            Duration::from_secs(5),
            Duration::ZERO,
        )
        .await
        .unwrap();
        assert_eq!(report["bodies"]["bytes"], 3145728);
        assert_eq!(report["bodies"]["liveBytes"], 1048576);
        assert_eq!(report["bodies"]["orphanBytes"], 2097152);
        let largest = &report["tables"][0];
        assert_eq!(largest["name"], "bodies");
        assert_eq!(largest["payloadBytes"], 3145728);
        assert!(largest["bytes"].is_null());
        assert_eq!(largest["columns"][0]["name"], "sealed_bytes");
        assert!(
            !report
                .to_string()
                .contains("DO-NOT-EXPOSE-SENSITIVE-BOUND-VALUE")
        );
        assert_eq!(report["limited"], true);
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bodies")
                .fetch_one(&pool)
                .await
                .unwrap(),
            2
        );
    }
    #[tokio::test]
    async fn database_reports_file_pages_pool_and_unreferenced_payload_without_changing_it() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let account = store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "example.test",
                    port: 993,
                    tls: "implicit",
                    username: "one@example.test",
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: "one@example.test".into(),
                    password: "diagnostic-fixture-secret-314159".into(),
                },
            )
            .await
            .unwrap();
        let blob = store.put_body(&account, b"retained body").await.unwrap();
        let quick = store.database_observability(false).await.unwrap();
        assert_eq!(quick["driver"], "SQLx");
        assert_eq!(quick["database"]["backend"], "sqlite");
        assert!(quick["pool"]["maximum"].as_u64().unwrap() > 0);
        assert!(quick["storage"]["pageSize"].as_u64().unwrap() > 0);
        assert!(quick["details"].is_null());
        let detailed = store.database_observability(true).await.unwrap();
        assert_eq!(detailed["details"]["bodies"]["count"], 1);
        assert_eq!(detailed["details"]["bodies"]["orphanCount"], 1);
        assert!(
            detailed["details"]["bodies"]["orphanBytes"]
                .as_u64()
                .unwrap()
                >= 13
        );
        assert_eq!(
            store.get_body(&blob).await.unwrap().unwrap(),
            b"retained body"
        );
        assert!(!detailed.to_string().contains("one@example.test"));
        assert!(
            !detailed
                .to_string()
                .contains("diagnostic-fixture-secret-314159")
        );
        assert!(!detailed.to_string().contains("retained body"));
        assert!(store.database_observability(false).await.is_ok());
        store
            .put_body(&account, &vec![b'x'; 1024 * 1024])
            .await
            .unwrap();
        let next = store.database_observability(true).await.unwrap();
        let table = next["details"]["tables"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["name"] == "bodies")
            .unwrap();
        assert!(
            table["growthBytes"].as_i64().unwrap() > 0,
            "new body allocation must be attributed to its table"
        );
    }
}
