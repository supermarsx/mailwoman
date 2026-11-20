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
        result["pool"] = self.database_pool_snapshot().await;
        match &self.backend {
            Backend::Sqlite(live) => {
                let pinned = live.pin().await;
                let pool = pinned.pool();
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

async fn sqlite_details(
    pool: crate::sqlite_live::LiveSqlite,
    budget: Duration,
) -> Result<Value, StoreError> {
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
    sqlite_details_with_scan_budgets(pool, budget, physical_budget, Duration::from_secs(7)).await
}

// Return measured prefix bytes, never extrapolate a biased prefix into a total.
async fn logical_sample(
    connection: &mut sqlx::SqliteConnection,
    name: &str,
    columns: &[String],
    deadline: Instant,
) -> Result<Value, sqlx::Error> {
    inspection_deadline(
        connection,
        deadline.min(Instant::now() + Duration::from_millis(500)),
    )
    .await?;
    let count = sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {}", quoted(name)))
        .fetch_one(&mut *connection)
        .await
        .ok();
    inspection_deadline(
        connection,
        deadline.min(Instant::now() + Duration::from_millis(750)),
    )
    .await?;
    let expressions = columns
        .iter()
        .enumerate()
        .map(|(i, col)| format!("octet_length({}) AS c{i}", quoted(col)))
        .collect::<Vec<_>>();
    let selected = if expressions.is_empty() {
        "1 AS present".into()
    } else {
        expressions.join(",")
    };
    let sums = columns
        .iter()
        .enumerate()
        .map(|(i, _)| format!(",CAST(COALESCE(SUM(c{i}),0) AS BIGINT) AS c{i}"))
        .collect::<String>();
    let statement = format!(
        "SELECT COUNT(*) AS n{sums} FROM (SELECT {selected} FROM {} LIMIT 512)",
        quoted(name)
    );
    let sampled = sqlx::query(&statement).fetch_one(&mut *connection).await;
    match sampled {
        Ok(row) => {
            let n = row.try_get::<i64, _>("n")?;
            let complete = n < 512 || count == Some(n);
            let values=columns.iter().enumerate().map(|(i,col)|Ok(json!({"name":col,"bytes":row.try_get::<i64,_>(format!("c{i}").as_str())?,"bytesLowerBound":!complete}))).collect::<Result<Vec<_>,sqlx::Error>>()?;
            let bytes = values
                .iter()
                .filter_map(|v| v["bytes"].as_i64())
                .fold(0i64, i64::saturating_add);
            Ok(
                json!({"rows":if n<512{Some(n)}else{count.or(Some(n))},"rowsEstimated":false,"rowsLowerBound":count.is_none()&&!complete,"payloadBytes":bytes,"payloadBytesLowerBound":!complete,"payloadSampleRows":n,"measurementSource":if complete{"bounded-complete-scan"}else{"sample-lower-bound"},"columns":values}),
            )
        }
        Err(_) => Ok(
            json!({"rows":count,"rowsEstimated":false,"measurementSource":"scan-budget-exceeded"}),
        ),
    }
}

async fn sqlite_details_with_scan_budgets(
    pool: sqlx::SqlitePool,
    budget: Duration,
    physical_budget: Duration,
    body_totals_budget: Duration,
) -> Result<Value, StoreError> {
    let mut connection = pool.acquire().await?;
    let deadline = Instant::now() + budget;
    inspection_deadline(&mut connection, deadline).await?;
    let work=async {
        let names=sqlx::query_scalar::<_,String>("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name LIMIT 513").fetch_all(&mut *connection).await?;
        let mut notes=Vec::new();
        let limited=names.len()>512;
        let mut bodies=json!({"count":null,"bytes":null,"orphanCount":null,"orphanBytes":null,"liveCount":null,"liveBytes":null});
        // COUNT may use the narrow primary-key index. Preserve it independently
        // from BLOB-table totals, then collect bounded metadata-only bytes first.
        let sample=logical_sample(&mut connection,"bodies",&["sealed_bytes".into()],deadline).await?;
        bodies["count"]=sample["rows"].clone();bodies["countLowerBound"]=sample["rowsLowerBound"].clone();
        bodies["bytes"]=sample["payloadBytes"].clone();bodies["bytesLowerBound"]=sample["payloadBytesLowerBound"].clone();bodies["sampleRows"]=sample["payloadSampleRows"].clone();
        // BLOB byte lengths are read from record metadata, never transferred or decrypted as mail.
        inspection_deadline(&mut connection,deadline.min(Instant::now()+body_totals_budget)).await?;
        match sqlx::query("SELECT COUNT(*) AS n,CAST(COALESCE(SUM(octet_length(sealed_bytes)),0) AS BIGINT) AS bytes FROM bodies").fetch_one(&mut *connection).await {
            Ok(row)=>{bodies["count"]=json!(row.try_get::<i64,_>("n")?);bodies["bytes"]=json!(row.try_get::<i64,_>("bytes")?);bodies["countLowerBound"]=json!(false);bodies["bytesLowerBound"]=json!(false);},
            Err(_)=>notes.push("body-totals-unavailable"),
        }
        let logical_deadline=deadline.min(Instant::now()+Duration::from_secs(12));
        let mut tables=Vec::new();
        for name in names.into_iter().take(512){
            let mut table=json!({"name":name,"bytes":null,"payloadBytes":null,"indexBytes":null,"rows":null,"rowsEstimated":false,"columns":[]});
            if name=="bodies"&&!bodies["bytes"].is_null(){
                table["payloadBytes"]=bodies["bytes"].clone();table["rows"]=bodies["count"].clone();
                table["payloadBytesLowerBound"]=bodies["bytesLowerBound"].clone();table["rowsLowerBound"]=bodies["countLowerBound"].clone();table["payloadSampleRows"]=bodies["sampleRows"].clone();
                table["measurementSource"]=json!(if bodies["bytesLowerBound"]==true{"sample-lower-bound"}else{"complete-scan"});
                table["columns"]=json!([{"name":"sealed_bytes","bytes":bodies["bytes"],"bytesLowerBound":bodies["bytesLowerBound"]}]);
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
                    Err(_)=>{
                        if !notes.contains(&"payload-scan-limited"){notes.push("payload-scan-limited");}
                        let fallback=logical_sample(&mut connection,&name,&columns,logical_deadline).await?;
                        if let Some(fields)=fallback.as_object(){for(key,value)in fields{table[key]=value.clone();}}
                    },
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
                if bodies["countLowerBound"]!=true {bodies["liveCount"]=bodies["count"].as_i64().map(|v|json!(v-n)).unwrap_or(Value::Null);}
                if bodies["bytesLowerBound"]!=true {bodies["liveBytes"]=bodies["bytes"].as_i64().map(|v|json!(v-bytes)).unwrap_or(Value::Null);}
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
    use sqlx::Connection;

    #[tokio::test]
    async fn interrupted_body_totals_keep_index_count_and_measured_prefix_bytes() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE bodies(blob_ref TEXT PRIMARY KEY,sealed_bytes BLOB);CREATE TABLE messages(blob_ref TEXT);WITH RECURSIVE n(i) AS (VALUES(1) UNION ALL SELECT i+1 FROM n WHERE i<800) INSERT INTO bodies SELECT printf('%08d',i),zeroblob(1024) FROM n").execute(&pool).await.unwrap();
        let report = sqlite_details_with_scan_budgets(
            pool.clone(),
            Duration::from_secs(5),
            Duration::ZERO,
            Duration::ZERO,
        )
        .await
        .unwrap();
        assert_eq!(report["bodies"]["count"], 800);
        assert_eq!(report["bodies"]["bytes"], 512 * 1024);
        assert_eq!(report["bodies"]["bytesLowerBound"], true);
        assert!(
            report["bodies"]["liveBytes"].is_null(),
            "a sampled total cannot be subtracted from exact orphan bytes"
        );
        let body = report["tables"]
            .as_array()
            .unwrap()
            .iter()
            .find(|table| table["name"] == "bodies")
            .unwrap();
        assert_eq!(body["rows"], 800);
        assert_eq!(body["payloadSampleRows"], 512);
        assert_eq!(body["payloadBytes"], 512 * 1024);
        assert_eq!(body["payloadBytesLowerBound"], true);
        assert!(body["bytes"].is_null(), "physical bytes were not scanned");
        assert_eq!(body["columns"][0]["bytesLowerBound"], true);
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bodies")
                .fetch_one(&pool)
                .await
                .unwrap(),
            800,
            "inspection must preserve all bodies"
        );
        pool.close().await;
    }
    #[tokio::test]
    async fn bounded_sample_supports_sqlcipher_and_never_exposes_values() {
        crate::encryption::init();
        let dir = tempfile::tempdir().unwrap();
        let key = crate::encryption::DbKey::generate();
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(dir.path().join("encrypted.db"))
            .create_if_missing(true)
            .pragma("key", crate::encryption::key_pragma(&key));
        let mut connection = sqlx::SqliteConnection::connect_with(&options)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE sample_log(detail TEXT,raw BLOB); INSERT INTO sample_log VALUES('NEVER-EXPOSE-THIS',zeroblob(2048)),('abc',zeroblob(1024))").execute(&mut connection).await.unwrap();
        let sample = logical_sample(
            &mut connection,
            "sample_log",
            &["detail".into(), "raw".into()],
            Instant::now() + Duration::from_secs(5),
        )
        .await
        .unwrap();
        assert_eq!(sample["rows"], 2);
        assert_eq!(sample["payloadBytes"], 3072 + 20);
        assert_eq!(sample["payloadBytesLowerBound"], false);
        assert_eq!(sample["columns"][1]["bytes"], 3072);
        assert!(!sample.to_string().contains("NEVER-EXPOSE-THIS"));
        connection
            .lock_handle()
            .await
            .unwrap()
            .remove_progress_handler();
        connection.close().await.unwrap();
    }

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
