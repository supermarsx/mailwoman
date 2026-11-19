//! Logical SQLite generation copy with transaction-coupled, payload-free change capture.
//!
//! The host owns paths, file permissions, generation admission and the final source
//! quiescence. This module never renames a live database or publishes a generation.
//! Every source writer must use recursive_triggers=ON and ordinary SQL DML; direct
//! incremental-BLOB writes, DDL and external writers outside that contract are unsupported.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{
    Connection, Row, SqliteConnection,
    sqlite::{SqliteConnectOptions, SqliteRow, SqliteSynchronous},
};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

const PREFIX: &str = "__mw_copy_";
const CONTROL: &str = "__mw_copy_control";
const EVENTS: &str = "__mw_copy_events";
const CATALOG: &str = "__mw_copy_catalog";
const APPLIED: &str = "__mw_copy_applied";
#[derive(Debug, thiserror::Error)]
pub enum CopyError {
    #[error("SQLite shadow copy failed: {0}")]
    Database(#[from] sqlx::Error),
    #[error("unsupported SQLite copy schema: {0}")]
    Unsupported(String),
    #[error("SQLite copy source schema changed")]
    SchemaChanged,
    #[error("SQLite copy capture exceeded its limit; the source remains authoritative")]
    Invalidated,
    #[error("SQLite copy was cancelled")]
    Cancelled,
    #[error("a SQLite row exceeds the configured copy byte budget in table {0}")]
    RowTooLarge(String),
    #[error("SQLite copy needs its baseline and journal drained before finalization")]
    NotCaughtUp,
    #[error("SQLite shadow verification failed: {0}")]
    Verification(&'static str),
    #[error("SQLite copy connection is closed")]
    Closed,
}
pub type Result<T> = std::result::Result<T, CopyError>;
#[derive(Clone, Debug)]
pub struct CopyOptions {
    pub batch_rows: u32,
    pub batch_bytes: u64,
    pub journal_limit: u64,
    pub cancel: Arc<AtomicBool>,
}
impl Default for CopyOptions {
    fn default() -> Self {
        Self {
            batch_rows: 64,
            batch_bytes: 16 * 1024 * 1024,
            journal_limit: 100_000,
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyProgress {
    pub table_index: usize,
    pub table_name: Option<String>,
    pub table_count: usize,
    pub copied_rows: u64,
    pub copied_bytes: u64,
    pub replayed_keys: u64,
    pub pending: u64,
    pub seed_complete: bool,
    pub valid: bool,
}
#[derive(Clone, Debug)]
struct Object {
    kind: String,
    name: String,
    table: String,
    sql: String,
}
#[derive(Clone, Debug)]
struct Table {
    name: String,
    columns: Vec<String>,
}
#[derive(Debug, PartialEq)]
enum Cell {
    Null,
    Integer(i64),
    Real(f64),
    Text(Vec<u8>),
    Blob(Vec<u8>),
}
#[derive(Debug, PartialEq)]
struct Record {
    rowid: i64,
    cells: Vec<Cell>,
    bytes: u64,
}
pub struct CopySession {
    source: Option<SqliteConnection>,
    target: Option<SqliteConnection>,
    options: CopyOptions,
    objects: Vec<Object>,
    tables: Vec<Table>,
    fingerprint: String,
    cookie: i64,
    table_index: usize,
    cursor: Option<i64>,
    copied_rows: u64,
    copied_bytes: u64,
    replayed_keys: u64,
    triggers_installed: bool,
    finalized: bool,
    bulk_verified: bool,
}
fn ident(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
fn unsupported(value: impl Into<String>) -> CopyError {
    CopyError::Unsupported(value.into())
}
async fn pragma(
    connection: &mut SqliteConnection,
    sql: &'static str,
) -> std::result::Result<i64, sqlx::Error> {
    crate::database_observability::sqlite_pragma_i64(connection, sql).await
}
async fn objects(connection: &mut SqliteConnection) -> Result<Vec<Object>> {
    let rows=sqlx::query("SELECT type,name,tbl_name,sql FROM sqlite_schema WHERE type IN ('table','index','view','trigger') AND sql IS NOT NULL ORDER BY type,name").fetch_all(connection).await?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let name: String = row.get("name");
            if name.starts_with("sqlite_") || name.starts_with(PREFIX) {
                return None;
            }
            Some(Object {
                kind: row.get("type"),
                name,
                table: row.get("tbl_name"),
                sql: row.get("sql"),
            })
        })
        .collect())
}
async fn fingerprint(connection: &mut SqliteConnection) -> Result<String> {
    let mut hash = Sha256::new();
    for object in objects(connection).await? {
        for value in [&object.kind, &object.name, &object.table, &object.sql] {
            hash.update((value.len() as u64).to_le_bytes());
            hash.update(value.as_bytes());
        }
    }
    for value in [
        pragma(connection, "PRAGMA user_version").await?,
        pragma(connection, "PRAGMA application_id").await?,
    ] {
        hash.update(value.to_le_bytes());
    }
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
async fn tables(connection: &mut SqliteConnection, objects: &[Object]) -> Result<Vec<Table>> {
    let mut result = Vec::new();
    for object in objects.iter().filter(|o| o.kind == "table") {
        let flags =
            sqlx::query("SELECT type,wr FROM pragma_table_list WHERE schema='main' AND name=?1")
                .bind(&object.name)
                .fetch_one(&mut *connection)
                .await?;
        if flags.get::<String, _>("type") != "table" || flags.get::<i64, _>("wr") != 0 {
            return Err(unsupported(format!(
                "{} requires a rowid table",
                object.name
            )));
        }
        let columns = sqlx::query("SELECT name,hidden FROM pragma_table_xinfo(?1) ORDER BY cid")
            .bind(&object.name)
            .fetch_all(&mut *connection)
            .await?;
        let mut names = Vec::new();
        for column in columns {
            let name: String = column.get("name");
            if column.get::<i64, _>("hidden") != 0
                || ["rowid", "_rowid_", "oid"].contains(&name.to_ascii_lowercase().as_str())
            {
                return Err(unsupported(format!(
                    "{} has generated, hidden or shadowed rowid columns",
                    object.name
                )));
            }
            names.push(name);
        }
        if names.is_empty() || names.len() > 256 {
            return Err(unsupported(format!(
                "{} has unsupported column count",
                object.name
            )));
        }
        result.push(Table {
            name: object.name.clone(),
            columns: names,
        });
    }
    if result.is_empty() || result.len() > 512 {
        return Err(unsupported("unsupported table count"));
    }
    Ok(result)
}
fn trigger_sql(prefix: &str, index: usize, table: &str, op: char) -> (String, String) {
    let name = format!("{prefix}{index}_{op}");
    let (event, rows) = match op {
        'i' => ("INSERT", vec!["NEW._rowid_"]),
        'd' => ("DELETE", vec!["OLD._rowid_"]),
        _ => ("UPDATE", vec!["OLD._rowid_", "NEW._rowid_"]),
    };
    let mut actions = String::new();
    for row in rows {
        actions.push_str(&format!("UPDATE {CONTROL} SET valid=0 WHERE id=1 AND valid=1 AND (COALESCE((SELECT MAX(id) FROM {EVENTS}),0)-ack)>=event_limit;"));
        actions.push_str(&format!("INSERT INTO {EVENTS}(table_id,row_id) SELECT {index},{row} FROM {CONTROL} WHERE id=1 AND valid=1;"));
    }
    let sql = format!(
        "CREATE TRIGGER {} AFTER {event} ON {} BEGIN {actions} END",
        ident(&name),
        ident(table)
    );
    (name, sql)
}
impl CopySession {
    /// A new target must be absent or an operator-created empty file. Existing jobs
    /// must be explicitly recovered/cleaned before starting another; no silent resume.
    pub async fn prepare(
        source: SqliteConnectOptions,
        target: SqliteConnectOptions,
        mut options: CopyOptions,
    ) -> Result<Self> {
        crate::encryption::init();
        if options.cancel.load(Ordering::Relaxed) {
            return Err(CopyError::Cancelled);
        }
        let source_path = source.get_filename();
        let target_path = target.get_filename();
        if !source_path.is_file() || source_path == target_path {
            return Err(unsupported("distinct on-disk source and target required"));
        }
        if std::fs::metadata(target_path).is_ok_and(|meta| meta.len() > 0) {
            return Err(unsupported("target is not empty"));
        }
        options.batch_rows = options.batch_rows.clamp(1, 1024);
        options.batch_bytes = options.batch_bytes.clamp(64 * 1024, 256 * 1024 * 1024);
        options.journal_limit = options.journal_limit.clamp(1, 10_000_000);
        let mut from = SqliteConnection::connect_with(&source).await?;
        if pragma(&mut from, "PRAGMA recursive_triggers").await? != 1 {
            return Err(unsupported(
                "all source connections must enable recursive_triggers",
            ));
        }
        let reserved: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_schema WHERE name GLOB '__mw_copy_*'")
                .fetch_one(&mut from)
                .await?;
        if reserved != 0 {
            return Err(unsupported(
                "unfinished copy or reserved schema objects exist",
            ));
        }
        let list = objects(&mut from).await?;
        let table_list = tables(&mut from, &list).await?;
        let hash = fingerprint(&mut from).await?;
        let mut to = SqliteConnection::connect_with(
            &target
                .clone()
                .create_if_missing(true)
                .foreign_keys(false)
                .synchronous(SqliteSynchronous::Full),
        )
        .await?;
        let existing: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_schema WHERE type='table'")
                .fetch_one(&mut to)
                .await?;
        if existing != 0 {
            return Err(unsupported("target schema is not empty"));
        }
        sqlx::query("PRAGMA auto_vacuum=INCREMENTAL")
            .execute(&mut to)
            .await?;
        sqlx::query("VACUUM").execute(&mut to).await?;
        if pragma(&mut to, "PRAGMA auto_vacuum").await? != 2 {
            return Err(CopyError::Verification(
                "incremental vacuum was not enabled",
            ));
        }
        let user_version = pragma(&mut from, "PRAGMA user_version").await?;
        let application_id = pragma(&mut from, "PRAGMA application_id").await?;
        let mut target_tx = to.begin().await?;
        for object in list.iter().filter(|o| o.kind == "table") {
            sqlx::query(&object.sql).execute(&mut *target_tx).await?;
        }
        for object in list
            .iter()
            .filter(|o| o.kind == "index" || o.kind == "view")
        {
            sqlx::query(&object.sql).execute(&mut *target_tx).await?;
        }
        sqlx::query(&format!("CREATE TABLE {APPLIED}(id INTEGER PRIMARY KEY CHECK(id=1),watermark INTEGER NOT NULL);INSERT INTO {APPLIED} VALUES(1,0)")).execute(&mut *target_tx).await?;
        target_tx.commit().await?;
        sqlx::query(&format!("PRAGMA user_version={user_version}"))
            .execute(&mut to)
            .await?;
        sqlx::query(&format!("PRAGMA application_id={application_id}"))
            .execute(&mut to)
            .await?;
        let run = uuid::Uuid::new_v4().simple().to_string();
        let trigger_prefix = format!("{PREFIX}{run}_");
        let mut tx = from.begin().await?;
        if fingerprint(&mut tx).await? != hash {
            return Err(CopyError::SchemaChanged);
        }
        sqlx::query(&format!("CREATE TABLE {CONTROL}(id INTEGER PRIMARY KEY CHECK(id=1),run_id TEXT NOT NULL,valid INTEGER NOT NULL,ack INTEGER NOT NULL,event_limit INTEGER NOT NULL);CREATE TABLE {CATALOG}(id INTEGER PRIMARY KEY,name TEXT NOT NULL);CREATE TABLE {EVENTS}(id INTEGER PRIMARY KEY AUTOINCREMENT,table_id INTEGER NOT NULL,row_id INTEGER NOT NULL)")).execute(&mut *tx).await?;
        sqlx::query(&format!("INSERT INTO {CONTROL} VALUES(1,?1,1,0,?2)"))
            .bind(&run)
            .bind(options.journal_limit as i64)
            .execute(&mut *tx)
            .await?;
        for (index, table) in table_list.iter().enumerate() {
            sqlx::query(&format!("INSERT INTO {CATALOG} VALUES(?1,?2)"))
                .bind(index as i64)
                .bind(&table.name)
                .execute(&mut *tx)
                .await?;
            for op in ['i', 'u', 'd'] {
                sqlx::query(&trigger_sql(&trigger_prefix, index, &table.name, op).1)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        tx.commit().await?;
        let cookie = pragma(&mut from, "PRAGMA schema_version").await?;
        Ok(Self {
            source: Some(from),
            target: Some(to),
            options,
            objects: list,
            tables: table_list,
            fingerprint: hash,
            cookie,
            table_index: 0,
            cursor: None,
            copied_rows: 0,
            copied_bytes: 0,
            replayed_keys: 0,
            triggers_installed: false,
            finalized: false,
            bulk_verified: false,
        })
    }
    /// Bound SQLite VM work on both private connections during cutover. The host
    /// must still bound lock waits and filesystem I/O, which are not VM steps.
    pub async fn set_deadline(&mut self, deadline: std::time::Instant) -> Result<()> {
        for connection in [&mut self.source, &mut self.target] {
            if let Some(connection) = connection.as_mut() {
                let cancel = self.options.cancel.clone();
                connection
                    .lock_handle()
                    .await?
                    .set_progress_handler(1000, move || {
                        std::time::Instant::now() < deadline && !cancel.load(Ordering::Relaxed)
                    });
            }
        }
        Ok(())
    }
    pub async fn clear_deadline(&mut self) -> Result<()> {
        for connection in [&mut self.source, &mut self.target] {
            if let Some(connection) = connection.as_mut() {
                connection.lock_handle().await?.remove_progress_handler();
            }
        }
        Ok(())
    }

    fn check_cancel(&self) -> Result<()> {
        if self.options.cancel.load(Ordering::Relaxed) {
            Err(CopyError::Cancelled)
        } else {
            Ok(())
        }
    }
    async fn check(&mut self) -> Result<()> {
        self.check_cancel()?;
        let source = self.source.as_mut().ok_or(CopyError::Closed)?;
        if pragma(source, "PRAGMA schema_version").await? != self.cookie
            || fingerprint(source).await? != self.fingerprint
        {
            return Err(CopyError::SchemaChanged);
        }
        let valid: i64 = sqlx::query_scalar(&format!("SELECT valid FROM {CONTROL} WHERE id=1"))
            .fetch_one(source)
            .await?;
        if valid != 1 {
            return Err(CopyError::Invalidated);
        }
        Ok(())
    }
    pub async fn progress(&mut self) -> Result<CopyProgress> {
        let source = self.source.as_mut().ok_or(CopyError::Closed)?;
        let valid: i64 = sqlx::query_scalar(&format!("SELECT valid FROM {CONTROL} WHERE id=1"))
            .fetch_one(&mut *source)
            .await?;
        let pending: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {EVENTS}"))
            .fetch_one(source)
            .await?;
        Ok(CopyProgress {
            table_index: self.table_index,
            table_name: self.tables.get(self.table_index).map(|t| t.name.clone()),
            table_count: self.tables.len(),
            copied_rows: self.copied_rows,
            copied_bytes: self.copied_bytes,
            replayed_keys: self.replayed_keys,
            pending: pending as u64,
            seed_complete: self.table_index == self.tables.len(),
            valid: valid == 1,
        })
    }
    pub async fn seed_batch(&mut self) -> Result<CopyProgress> {
        self.check().await?;
        if self.triggers_installed {
            return Err(CopyError::NotCaughtUp);
        }
        let Some(table) = self.tables.get(self.table_index).cloned() else {
            return self.progress().await;
        };
        let source = self.source.as_mut().ok_or(CopyError::Closed)?;
        let mut tx = source.begin().await?;
        let size = size_expression(&table);
        let lower = if self.cursor.is_some() {
            "WHERE _rowid_>?1"
        } else {
            ""
        };
        let sql = format!(
            "SELECT _rowid_ AS rid,{size} AS bytes FROM {} {lower} ORDER BY _rowid_ LIMIT {}",
            ident(&table.name),
            self.options.batch_rows
        );
        let mut query = sqlx::query(&sql);
        if let Some(cursor) = self.cursor {
            query = query.bind(cursor);
        }
        let candidates = query.fetch_all(&mut *tx).await?;
        let mut records = Vec::new();
        let mut bytes = 0;
        let mut cursor = self.cursor;
        for row in &candidates {
            if self.options.cancel.load(Ordering::Relaxed) {
                return Err(CopyError::Cancelled);
            }
            let length = row.try_get::<i64, _>("bytes")?.max(0) as u64;
            if length > self.options.batch_bytes {
                return Err(CopyError::RowTooLarge(table.name.clone()));
            }
            if !records.is_empty() && bytes + length > self.options.batch_bytes {
                break;
            }
            let rowid: i64 = row.get("rid");
            let record =
                read_record(&mut tx, &table, rowid)
                    .await?
                    .ok_or(CopyError::Verification(
                        "row vanished inside a source snapshot",
                    ))?;
            bytes += record.bytes;
            cursor = Some(rowid);
            records.push(record);
        }
        tx.commit().await?;
        let target = self.target.as_mut().ok_or(CopyError::Closed)?;
        let mut write = target.begin().await?;
        for record in &records {
            write_record(&mut write, &table, record).await?;
        }
        write.commit().await?;
        self.copied_rows += records.len() as u64;
        self.copied_bytes += bytes;
        self.cursor = cursor;
        if records.len() == candidates.len() && candidates.len() < self.options.batch_rows as usize
        {
            self.table_index += 1;
            self.cursor = None;
        }
        self.check().await?;
        self.progress().await
    }
    pub async fn catch_up_batch(&mut self) -> Result<CopyProgress> {
        self.check().await?;
        if self.table_index != self.tables.len() || self.triggers_installed {
            return Err(CopyError::NotCaughtUp);
        }
        let source = self.source.as_mut().ok_or(CopyError::Closed)?;
        let mut tx = source.begin().await?;
        let events = sqlx::query(&format!(
            "SELECT id,table_id,row_id FROM {EVENTS} ORDER BY id LIMIT ?1"
        ))
        .bind(i64::from(self.options.batch_rows))
        .fetch_all(&mut *tx)
        .await?;
        let mut records = BTreeMap::new();
        let mut bytes = 0;
        let mut maximum = None;
        for event in events {
            if self.options.cancel.load(Ordering::Relaxed) {
                return Err(CopyError::Cancelled);
            }
            let table_id = event.try_get::<i64, _>("table_id")?;
            let rowid = event.try_get::<i64, _>("row_id")?;
            let key = (table_id, rowid);
            if !records.contains_key(&key) {
                let table = self
                    .tables
                    .get(
                        usize::try_from(table_id)
                            .map_err(|_| CopyError::Verification("invalid journal table"))?,
                    )
                    .ok_or(CopyError::Verification("invalid journal table"))?;
                let length = record_size(&mut tx, table, rowid).await?.unwrap_or(0);
                if length > self.options.batch_bytes {
                    return Err(CopyError::RowTooLarge(table.name.clone()));
                }
                if !records.is_empty() && bytes + length > self.options.batch_bytes {
                    break;
                }
                let record = read_record(&mut tx, table, rowid).await?;
                bytes += record.as_ref().map_or(0, |r| r.bytes);
                records.insert(key, record);
            }
            maximum = Some(event.try_get::<i64, _>("id")?);
        }
        tx.commit().await?;
        if let Some(maximum) = maximum {
            let target = self.target.as_mut().ok_or(CopyError::Closed)?;
            let mut write = target.begin().await?;
            for &(table_id, rowid) in records.keys() {
                sqlx::query(&format!(
                    "DELETE FROM {} WHERE _rowid_=?1",
                    ident(&self.tables[table_id as usize].name)
                ))
                .bind(rowid)
                .execute(&mut *write)
                .await?;
            }
            for (&(table_id, _), record) in &records {
                if let Some(record) = record {
                    write_record(&mut write, &self.tables[table_id as usize], record).await?;
                }
            }
            sqlx::query(&format!(
                "UPDATE {APPLIED} SET watermark=MAX(watermark,?1) WHERE id=1"
            ))
            .bind(maximum)
            .execute(&mut *write)
            .await?;
            write.commit().await?; // The target prefix is durable BEFORE source acknowledgement.
            let source = self.source.as_mut().ok_or(CopyError::Closed)?;
            let mut ack = source.begin().await?;
            sqlx::query(&format!("UPDATE {CONTROL} SET ack=MAX(ack,?1) WHERE id=1"))
                .bind(maximum)
                .execute(&mut *ack)
                .await?;
            sqlx::query(&format!("DELETE FROM {EVENTS} WHERE id<=?1"))
                .bind(maximum)
                .execute(&mut *ack)
                .await?;
            ack.commit().await?;
            self.replayed_keys += records.len() as u64;
        }
        self.check().await?;
        self.progress().await
    }
    /// Validate shadow B-trees while the application continues using the source.
    /// Later SQL replay transactions preserve this structural integrity. The host
    /// must run this before its bounded cutover gate, not during that gate.
    pub async fn verify_bulk(&mut self) -> Result<()> {
        self.check().await?;
        if self.table_index != self.tables.len() || self.triggers_installed {
            return Err(CopyError::NotCaughtUp);
        }
        let target = self.target.as_mut().ok_or(CopyError::Closed)?;
        let cancel = self.options.cancel.clone();
        target
            .lock_handle()
            .await?
            .set_progress_handler(1000, move || !cancel.load(Ordering::Relaxed));
        let result = sqlx::query("PRAGMA quick_check(100)")
            .fetch_all(&mut *target)
            .await;
        target.lock_handle().await?.remove_progress_handler();
        let rows = result?;
        if rows.len() != 1 || rows[0].try_get::<String, _>(0)? != "ok" {
            return Err(CopyError::Verification("shadow B-tree check failed"));
        }
        self.check().await?;
        self.bulk_verified = true;
        Ok(())
    }

    /// The caller must prevent ALL new source writes and drain old operations before
    /// calling this method, and retain that gate until the generation is published.
    /// Verification can be expensive; do not promise an unbounded queue wait.
    pub async fn finalize(&mut self) -> Result<CopyProgress> {
        self.check().await?;
        if !self.bulk_verified {
            return Err(CopyError::NotCaughtUp);
        }
        let progress = self.progress().await?;
        if !progress.seed_complete || progress.pending != 0 {
            return Err(CopyError::NotCaughtUp);
        }
        let source = self.source.as_mut().ok_or(CopyError::Closed)?;
        let has_sequence: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_schema WHERE name='sqlite_sequence'")
                .fetch_one(&mut *source)
                .await?;
        let sequences = if has_sequence != 0 {
            sqlx::query("SELECT name,seq FROM sqlite_sequence WHERE name NOT GLOB '__mw_copy_*'")
                .fetch_all(&mut *source)
                .await?
        } else {
            Vec::new()
        };
        let target = self.target.as_mut().ok_or(CopyError::Closed)?;
        if !sequences.is_empty() {
            let has: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_schema WHERE name='sqlite_sequence'",
            )
            .fetch_one(&mut *target)
            .await?;
            if has == 0 {
                sqlx::query("CREATE TABLE __mw_copy_seq_seed(id INTEGER PRIMARY KEY AUTOINCREMENT);DROP TABLE __mw_copy_seq_seed").execute(&mut *target).await?;
            }
            sqlx::query("DELETE FROM sqlite_sequence")
                .execute(&mut *target)
                .await?;
            for row in sequences {
                sqlx::query("INSERT INTO sqlite_sequence(name,seq) VALUES(?1,?2)")
                    .bind(row.try_get::<String, _>("name")?)
                    .bind(row.try_get::<i64, _>("seq")?)
                    .execute(&mut *target)
                    .await?;
            }
        }
        if sqlx::query("PRAGMA foreign_key_check")
            .fetch_optional(&mut *target)
            .await?
            .is_some()
        {
            return Err(CopyError::Verification("foreign key check failed"));
        }
        for table in &self.tables {
            let sql = format!("SELECT COALESCE(SUM(1),0) FROM {}", ident(&table.name));
            let source = self.source.as_mut().ok_or(CopyError::Closed)?;
            let expected: i64 = sqlx::query_scalar(&sql).fetch_one(source).await?;
            let actual: i64 = sqlx::query_scalar(&sql).fetch_one(&mut *target).await?;
            if expected != actual {
                return Err(CopyError::Verification(
                    "source and shadow row counts differ",
                ));
            }
        }
        if !self.triggers_installed {
            let mut tx = target.begin().await?;
            for object in self.objects.iter().filter(|o| o.kind == "trigger") {
                sqlx::query(&object.sql).execute(&mut *tx).await?;
            }
            sqlx::query(&format!("DROP TABLE {APPLIED}"))
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            self.triggers_installed = true;
        }
        sqlx::query("PRAGMA foreign_keys=ON")
            .execute(&mut *target)
            .await?;
        if pragma(target, "PRAGMA auto_vacuum").await? != 2
            || fingerprint(target).await? != self.fingerprint
        {
            return Err(CopyError::Verification("shadow schema differs"));
        }
        let checkpoint = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .fetch_one(&mut *target)
            .await?;
        if checkpoint.try_get::<i64, _>(0).unwrap_or(1) != 0 {
            return Err(CopyError::Verification("shadow checkpoint is busy"));
        }
        self.check().await?;
        let progress = self.progress().await?;
        if progress.pending != 0 {
            return Err(CopyError::NotCaughtUp);
        }
        self.finalized = true;
        Ok(progress)
    }
    pub async fn cleanup_source(&mut self) -> Result<()> {
        let source = self.source.as_mut().ok_or(CopyError::Closed)?;
        cleanup_connection(source).await.map(|_| ())
    }
    pub async fn close_target(&mut self) -> Result<()> {
        if let Some(connection) = self.target.take() {
            connection.close().await?;
        }
        Ok(())
    }
    pub async fn close_source(&mut self) -> Result<()> {
        if let Some(connection) = self.source.take() {
            connection.close().await?;
        }
        Ok(())
    }
    pub async fn close(mut self) -> Result<()> {
        self.close_target().await?;
        self.close_source().await
    }
    pub fn is_finalized(&self) -> bool {
        self.finalized
    }
}
fn size_expression(table: &Table) -> String {
    table.columns.iter().map(|c|{let c=ident(c);format!("CASE typeof({c}) WHEN 'null' THEN 0 WHEN 'integer' THEN 8 WHEN 'real' THEN 8 ELSE octet_length({c}) END")}).collect::<Vec<_>>().join("+")
}
async fn record_size(
    connection: &mut SqliteConnection,
    table: &Table,
    rowid: i64,
) -> Result<Option<u64>> {
    Ok(sqlx::query_scalar::<_, i64>(&format!(
        "SELECT {} FROM {} WHERE _rowid_=?1",
        size_expression(table),
        ident(&table.name)
    ))
    .bind(rowid)
    .fetch_optional(connection)
    .await?
    .map(|n| n.max(0) as u64))
}
async fn read_record(
    connection: &mut SqliteConnection,
    table: &Table,
    rowid: i64,
) -> Result<Option<Record>> {
    let columns = table
        .columns
        .iter()
        .map(|c| {
            let c = ident(c);
            format!("typeof({c}),{c}")
        })
        .collect::<Vec<_>>()
        .join(",");
    let row = sqlx::query(&format!(
        "SELECT _rowid_,{columns} FROM {} WHERE _rowid_=?1",
        ident(&table.name)
    ))
    .bind(rowid)
    .fetch_optional(connection)
    .await?;
    row.map(|row| decode_record(row, table.columns.len()))
        .transpose()
}
fn decode_record(row: SqliteRow, columns: usize) -> Result<Record> {
    let mut cells = Vec::new();
    let mut bytes = 0;
    for index in 0..columns {
        let kind: String = row.try_get(1 + 2 * index)?;
        let at = 2 + 2 * index;
        let value = match kind.as_str() {
            "null" => Cell::Null,
            "integer" => Cell::Integer(row.try_get_unchecked(at)?),
            "real" => Cell::Real(row.try_get_unchecked(at)?),
            "text" => Cell::Text(row.try_get_unchecked(at)?),
            "blob" => Cell::Blob(row.try_get_unchecked(at)?),
            _ => return Err(CopyError::Verification("unknown SQLite storage class")),
        };
        bytes += match &value {
            Cell::Null => 0,
            Cell::Integer(_) | Cell::Real(_) => 8,
            Cell::Text(v) | Cell::Blob(v) => v.len() as u64,
        };
        cells.push(value);
    }
    Ok(Record {
        rowid: row.try_get(0)?,
        cells,
        bytes,
    })
}
async fn write_record(
    connection: &mut SqliteConnection,
    table: &Table,
    record: &Record,
) -> Result<()> {
    let columns = table
        .columns
        .iter()
        .map(|c| ident(c))
        .collect::<Vec<_>>()
        .join(",");
    let values = record
        .cells
        .iter()
        .map(|c| {
            if matches!(c, Cell::Text(_)) {
                "CAST(? AS TEXT)"
            } else {
                "?"
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "INSERT OR REPLACE INTO {}(_rowid_,{columns}) VALUES(?,{values})",
        ident(&table.name)
    );
    let mut query = sqlx::query(&sql).bind(record.rowid);
    for value in &record.cells {
        query = match value {
            Cell::Null => query.bind(Option::<i64>::None),
            Cell::Integer(v) => query.bind(*v),
            Cell::Real(v) => query.bind(*v),
            Cell::Text(v) | Cell::Blob(v) => query.bind(v.as_slice()),
        };
    }
    query.execute(&mut *connection).await?;
    let copied = read_record(connection, table, record.rowid).await?;
    if copied.as_ref() != Some(record) {
        return Err(CopyError::Verification(
            "copied row storage classes or values differ",
        ));
    }
    Ok(())
}
/// Remove only a verified capture job left by a prior process. The caller owns
/// exclusive conversion admission; this does not erase any application rows.
pub async fn cleanup_abandoned(source: SqliteConnectOptions) -> Result<bool> {
    crate::encryption::init();
    let mut connection = SqliteConnection::connect_with(&source).await?;
    let result = cleanup_connection(&mut connection).await;
    connection.close().await?;
    result
}
async fn cleanup_connection(connection: &mut SqliteConnection) -> Result<bool> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_schema WHERE name=?1 AND type='table'")
            .bind(CONTROL)
            .fetch_one(&mut *connection)
            .await?;
    if count == 0 {
        return Ok(false);
    }
    let mut tx = connection.begin().await?;
    for (table, expected) in [
        (CONTROL, vec!["id", "run_id", "valid", "ack", "event_limit"]),
        (EVENTS, vec!["id", "table_id", "row_id"]),
        (CATALOG, vec!["id", "name"]),
    ] {
        let columns = sqlx::query("SELECT name FROM pragma_table_xinfo(?1) ORDER BY cid")
            .bind(table)
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|r| r.get::<String, _>("name"))
            .collect::<Vec<_>>();
        if columns != expected {
            return Err(unsupported("unrecognized capture schema; cleanup refused"));
        }
    }
    let run: String = sqlx::query_scalar(&format!("SELECT run_id FROM {CONTROL} WHERE id=1"))
        .fetch_one(&mut *tx)
        .await?;
    if run.len() != 32 || !run.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(unsupported("unrecognized capture owner"));
    }
    let prefix = format!("{PREFIX}{run}_");
    let catalog = sqlx::query(&format!(
        "SELECT id,name FROM {CATALOG} ORDER BY id LIMIT 513"
    ))
    .fetch_all(&mut *tx)
    .await?;
    if catalog.len() > 512 {
        return Err(unsupported("unrecognized capture catalog"));
    }
    let mut expected = BTreeMap::new();
    for table in catalog {
        let index: i64 = table.get("id");
        if index < 0 {
            return Err(unsupported("unrecognized capture table"));
        }
        let name: String = table.get("name");
        for op in ['i', 'u', 'd'] {
            let (name, sql) = trigger_sql(&prefix, index as usize, &name, op);
            expected.insert(name, sql);
        }
    }
    let actual = sqlx::query(
        "SELECT name,sql FROM sqlite_schema WHERE type='trigger' AND name GLOB '__mw_copy_*'",
    )
    .fetch_all(&mut *tx)
    .await?;
    for row in &actual {
        let name: String = row.get("name");
        let sql: String = row.get("sql");
        if expected.get(&name).map(|s| s.trim_end_matches(';').trim())
            != Some(sql.trim_end_matches(';').trim())
        {
            return Err(unsupported("capture trigger ownership mismatch"));
        }
    }
    for row in actual {
        sqlx::query(&format!(
            "DROP TRIGGER {}",
            ident(&row.get::<String, _>("name"))
        ))
        .execute(&mut *tx)
        .await?;
    }
    for table in [EVENTS, CATALOG, CONTROL] {
        sqlx::query(&format!("DROP TABLE {}", ident(table)))
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{path::PathBuf, time::Duration};
    struct Fixture {
        dir: PathBuf,
        source: SqliteConnectOptions,
        target: SqliteConnectOptions,
    }
    impl Fixture {
        async fn new(schema: &str) -> Self {
            crate::encryption::init();
            let dir = std::env::temp_dir().join(format!("mw-online-copy-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir(&dir).unwrap();
            let source = SqliteConnectOptions::new()
                .filename(dir.join("source.db"))
                .create_if_missing(true)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .pragma("recursive_triggers", "ON")
                .busy_timeout(Duration::from_secs(5));
            let target = source.clone().filename(dir.join("target.db"));
            let mut connection = SqliteConnection::connect_with(&source).await.unwrap();
            sqlx::raw_sql(schema)
                .execute(&mut connection)
                .await
                .unwrap();
            connection.close().await.unwrap();
            Self {
                dir,
                source,
                target,
            }
        }
        async fn writer(&self) -> SqliteConnection {
            SqliteConnection::connect_with(&self.source).await.unwrap()
        }
        async fn clean(self) {
            assert!(self.dir.starts_with(std::env::temp_dir()));
            assert!(
                self.dir
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("mw-online-copy-")
            );
            for _ in 0..40 {
                if std::fs::remove_dir_all(&self.dir).is_ok() {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            panic!("fixture cleanup failed");
        }
    }
    async fn seed(session: &mut CopySession) {
        for _ in 0..10000 {
            if session.seed_batch().await.unwrap().seed_complete {
                return;
            }
        }
        panic!("seed failed to finish");
    }
    async fn catch_up(session: &mut CopySession) {
        for _ in 0..10000 {
            if session.catch_up_batch().await.unwrap().pending == 0 {
                return;
            }
        }
        panic!("capture failed to drain");
    }
    async fn finish(session: &mut CopySession) {
        catch_up(session).await;
        session.verify_bulk().await.unwrap();
        session.finalize().await.unwrap();
        session.cleanup_source().await.unwrap();
        session.close_target().await.unwrap();
        session.close_source().await.unwrap();
    }
    #[tokio::test]
    async fn rowids_dynamic_values_cascades_and_revision_triggers_survive() {
        let fixture=Fixture::new("CREATE TABLE parent(id INTEGER PRIMARY KEY);CREATE TABLE child(id INTEGER PRIMARY KEY,parent_id INTEGER REFERENCES parent(id) ON DELETE CASCADE);INSERT INTO parent VALUES(1);INSERT INTO child VALUES(1,1);CREATE TABLE revision(id INTEGER PRIMARY KEY,n INTEGER);INSERT INTO revision VALUES(1,0);CREATE TABLE config(key TEXT PRIMARY KEY,value BLOB);CREATE TRIGGER config_revision AFTER UPDATE ON config BEGIN UPDATE revision SET n=n+1 WHERE id=1;END;INSERT INTO config VALUES('key',x'01');CREATE TABLE loose(value);INSERT INTO loose(rowid,value) VALUES(-7,NULL),(2,9223372036854775807),(9,3.125),(20,CAST(x'80FF0041' AS TEXT)),(42,x'00FF');CREATE TABLE nullable(key TEXT PRIMARY KEY,value BLOB);INSERT INTO nullable(rowid,key,value) VALUES(5,NULL,x'00FF'),(8,NULL,x'0000');").await;
        let mut copy = CopySession::prepare(
            fixture.source.clone(),
            fixture.target.clone(),
            CopyOptions {
                batch_rows: 1,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let mut writer = fixture.writer().await;
        assert_eq!(
            copy.seed_batch().await.unwrap().table_name.as_deref(),
            Some("child")
        );
        sqlx::raw_sql("BEGIN;DELETE FROM parent WHERE id=1;UPDATE config SET value=x'0200';UPDATE loose SET rowid=100 WHERE rowid=42;INSERT INTO loose(rowid,value) VALUES(3,'new');COMMIT;").execute(&mut writer).await.unwrap();
        seed(&mut copy).await;
        // Rollback does not leave capture records or visible content changes.
        let pending = copy.progress().await.unwrap().pending;
        sqlx::raw_sql("BEGIN;UPDATE config SET value=x'FFFFFFFF';ROLLBACK;")
            .execute(&mut writer)
            .await
            .unwrap();
        assert_eq!(copy.progress().await.unwrap().pending, pending);
        finish(&mut copy).await;
        let mut target = SqliteConnection::connect_with(&fixture.target)
            .await
            .unwrap();
        for table in ["loose", "nullable"] {
            let sql = format!(
                "SELECT _rowid_,typeof(value),hex(CAST(value AS BLOB)) FROM {table} ORDER BY _rowid_"
            );
            let a = sqlx::query_as::<_, (i64, String, String)>(&sql)
                .fetch_all(&mut writer)
                .await
                .unwrap();
            let b = sqlx::query_as::<_, (i64, String, String)>(&sql)
                .fetch_all(&mut target)
                .await
                .unwrap();
            assert_eq!(a, b);
        }
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM child")
                .fetch_one(&mut target)
                .await
                .unwrap(),
            0
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT n FROM revision")
                .fetch_one(&mut target)
                .await
                .unwrap(),
            1
        );
        sqlx::query("UPDATE config SET value=x'03'")
            .execute(&mut target)
            .await
            .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT n FROM revision")
                .fetch_one(&mut target)
                .await
                .unwrap(),
            2
        );
        writer.close().await.unwrap();
        target.close().await.unwrap();
        fixture.clean().await;
    }
    #[tokio::test]
    async fn baseline_uniqueness_churn_and_duplicate_replay_are_idempotent() {
        let fixture=Fixture::new("CREATE TABLE items(id INTEGER PRIMARY KEY,value TEXT UNIQUE);INSERT INTO items VALUES(1,'a'),(2,'b');").await;
        let mut copy = CopySession::prepare(
            fixture.source.clone(),
            fixture.target.clone(),
            CopyOptions {
                batch_rows: 1,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        copy.seed_batch().await.unwrap();
        let mut writer = fixture.writer().await;
        sqlx::raw_sql(
            "BEGIN;DELETE FROM items WHERE id=1;UPDATE items SET value='a' WHERE id=2;COMMIT;",
        )
        .execute(&mut writer)
        .await
        .unwrap();
        seed(&mut copy).await;
        // Simulate target commit followed by a crash before source ACK: replay again.
        let table = copy.tables[0].clone();
        let record = read_record(&mut writer, &table, 2).await.unwrap().unwrap();
        let target = copy.target.as_mut().unwrap();
        let mut tx = target.begin().await.unwrap();
        write_record(&mut tx, &table, &record).await.unwrap();
        sqlx::query(&format!("UPDATE {APPLIED} SET watermark=1"))
            .execute(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        finish(&mut copy).await;
        let mut target = SqliteConnection::connect_with(&fixture.target)
            .await
            .unwrap();
        assert_eq!(
            sqlx::query_as::<_, (i64, String)>("SELECT id,value FROM items")
                .fetch_all(&mut target)
                .await
                .unwrap(),
            vec![(2, "a".into())]
        );
        target.close().await.unwrap();
        writer.close().await.unwrap();
        fixture.clean().await;
    }
    #[tokio::test]
    async fn capture_overflow_invalidates_copy_without_rejecting_foreground_writes() {
        let fixture = Fixture::new("CREATE TABLE items(id INTEGER PRIMARY KEY,value BLOB)").await;
        let mut copy = CopySession::prepare(
            fixture.source.clone(),
            fixture.target.clone(),
            CopyOptions {
                journal_limit: 1,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let mut writer = fixture.writer().await;
        sqlx::query("INSERT INTO items VALUES(1,x'00'),(2,x'01'),(3,x'02')")
            .execute(&mut writer)
            .await
            .unwrap();
        let progress = copy.progress().await.unwrap();
        assert!(!progress.valid);
        assert_eq!(progress.pending, 1);
        assert!(matches!(
            copy.seed_batch().await,
            Err(CopyError::Invalidated)
        ));
        copy.cleanup_source().await.unwrap();
        copy.close().await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM items")
                .fetch_one(&mut writer)
                .await
                .unwrap(),
            3
        );
        writer.close().await.unwrap();
        fixture.clean().await;
    }
    #[tokio::test]
    async fn schema_changes_cancellation_and_unsupported_tables_fail_closed() {
        for schema in [
            "CREATE TABLE bad(rowid TEXT,value BLOB)",
            "CREATE TABLE bad(id TEXT PRIMARY KEY) WITHOUT ROWID",
            "CREATE TABLE bad(id INTEGER,value INTEGER GENERATED ALWAYS AS(id+1))",
        ] {
            let fixture = Fixture::new(schema).await;
            assert!(matches!(
                CopySession::prepare(
                    fixture.source.clone(),
                    fixture.target.clone(),
                    CopyOptions::default()
                )
                .await,
                Err(CopyError::Unsupported(_))
            ));
            fixture.clean().await;
        }
        let fixture = Fixture::new("CREATE TABLE items(id INTEGER PRIMARY KEY)").await;
        let cancel = Arc::new(AtomicBool::new(false));
        let mut copy = CopySession::prepare(
            fixture.source.clone(),
            fixture.target.clone(),
            CopyOptions {
                cancel: cancel.clone(),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        cancel.store(true, Ordering::Relaxed);
        assert!(matches!(copy.seed_batch().await, Err(CopyError::Cancelled)));
        cancel.store(false, Ordering::Relaxed);
        let mut writer = fixture.writer().await;
        sqlx::query("CREATE TABLE added(id INTEGER)")
            .execute(&mut writer)
            .await
            .unwrap();
        assert!(matches!(
            copy.seed_batch().await,
            Err(CopyError::SchemaChanged)
        ));
        copy.cleanup_source().await.unwrap();
        copy.close().await.unwrap();
        writer.close().await.unwrap();
        fixture.clean().await;
    }
    #[tokio::test]
    async fn cleanup_recovers_owned_capture_and_refuses_foreign_objects() {
        let fixture = Fixture::new("CREATE TABLE items(id INTEGER PRIMARY KEY)").await;
        let copy = CopySession::prepare(
            fixture.source.clone(),
            fixture.target.clone(),
            CopyOptions::default(),
        )
        .await
        .unwrap();
        copy.close().await.unwrap();
        assert!(cleanup_abandoned(fixture.source.clone()).await.unwrap());
        assert!(!cleanup_abandoned(fixture.source.clone()).await.unwrap());
        let mut writer = fixture.writer().await;
        sqlx::query("INSERT INTO items VALUES(5)")
            .execute(&mut writer)
            .await
            .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT id FROM items")
                .fetch_one(&mut writer)
                .await
                .unwrap(),
            5
        );
        writer.close().await.unwrap();
        fixture.clean().await;
    }
    #[tokio::test]
    async fn encrypted_copy_retains_sqlcipher_and_referenced_blob_bytes() {
        crate::encryption::init();
        let dir = std::env::temp_dir().join(format!("mw-online-copy-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let key = crate::encryption::DbKey::generate();
        let source = SqliteConnectOptions::new()
            .filename(dir.join("source.db"))
            .create_if_missing(true)
            .pragma("key", crate::encryption::key_pragma(&key))
            .pragma("recursive_triggers", "ON")
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
        let fixture = Fixture {
            dir,
            target: source
                .clone()
                .filename(source.get_filename().with_file_name("target.db")),
            source,
        };
        let mut writer = fixture.writer().await;
        sqlx::query("CREATE TABLE bodies(id TEXT PRIMARY KEY,payload BLOB);INSERT INTO bodies VALUES('kept',x'0001FF80')").execute(&mut writer).await.unwrap();
        let mut copy = CopySession::prepare(
            fixture.source.clone(),
            fixture.target.clone(),
            CopyOptions::default(),
        )
        .await
        .unwrap();
        seed(&mut copy).await;
        finish(&mut copy).await;
        assert_eq!(
            crate::encryption::probe_file(fixture.target.get_filename()).unwrap(),
            crate::encryption::FileState::NotPlaintext
        );
        let mut target = SqliteConnection::connect_with(&fixture.target)
            .await
            .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, Vec<u8>>("SELECT payload FROM bodies")
                .fetch_one(&mut target)
                .await
                .unwrap(),
            vec![0, 1, 255, 128]
        );
        assert_eq!(pragma(&mut target, "PRAGMA auto_vacuum").await.unwrap(), 2);
        target.close().await.unwrap();
        writer.close().await.unwrap();
        fixture.clean().await;
    }
    #[tokio::test]
    async fn entire_store_schema_including_security_and_migration_rows_is_preserved() {
        let fixture = Fixture::new("CREATE TABLE initial(id INTEGER);DROP TABLE initial;").await;
        let store = crate::Store::open_with_encryption(
            fixture.source.get_filename().to_str().unwrap(),
            crate::ServerKey::generate(),
            &crate::encryption::EncryptionConfig::off(),
        )
        .await
        .unwrap();
        store.close().await;
        let mut writer = fixture.writer().await;
        sqlx::query("PRAGMA auto_vacuum=NONE")
            .execute(&mut writer)
            .await
            .unwrap();
        sqlx::query("VACUUM").execute(&mut writer).await.unwrap();
        let mut copy = CopySession::prepare(
            fixture.source.clone(),
            fixture.target.clone(),
            CopyOptions::default(),
        )
        .await
        .unwrap();
        assert!(copy.tables.iter().any(|t| t.name == "_sqlx_migrations"));
        assert!(copy.tables.iter().any(|t| t.name == "admin_account"));
        assert!(copy.tables.iter().any(|t| t.name == "settings"));
        seed(&mut copy).await;
        finish(&mut copy).await;
        let mut target = SqliteConnection::connect_with(&fixture.target)
            .await
            .unwrap();
        let n = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&mut target)
            .await
            .unwrap();
        assert!(n >= 100);
        assert_eq!(pragma(&mut target, "PRAGMA auto_vacuum").await.unwrap(), 2);
        target.close().await.unwrap();
        writer.close().await.unwrap();
        fixture.clean().await;
    }
    #[tokio::test]
    async fn foreground_writer_continues_during_bounded_baseline_batches() {
        let fixture=Fixture::new("CREATE TABLE items(id INTEGER PRIMARY KEY,payload BLOB);WITH RECURSIVE n(i) AS(VALUES(1) UNION ALL SELECT i+1 FROM n WHERE i<100) INSERT INTO items SELECT i,zeroblob(1024) FROM n;").await;
        let mut copy = CopySession::prepare(
            fixture.source.clone(),
            fixture.target.clone(),
            CopyOptions {
                batch_rows: 2,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let options = fixture.source.clone();
        let writer = tokio::spawn(async move {
            let mut connection = SqliteConnection::connect_with(&options).await.unwrap();
            for id in 1..=100 {
                sqlx::query("UPDATE items SET payload=?1 WHERE id=?2")
                    .bind(vec![id as u8; 1024])
                    .bind(id)
                    .execute(&mut connection)
                    .await
                    .unwrap();
            }
            connection.close().await.unwrap();
        });
        seed(&mut copy).await;
        writer.await.unwrap();
        finish(&mut copy).await;
        let mut a = fixture.writer().await;
        let mut b = SqliteConnection::connect_with(&fixture.target)
            .await
            .unwrap();
        let original =
            sqlx::query_as::<_, (i64, Vec<u8>)>("SELECT id,payload FROM items ORDER BY id")
                .fetch_all(&mut a)
                .await
                .unwrap();
        let copied =
            sqlx::query_as::<_, (i64, Vec<u8>)>("SELECT id,payload FROM items ORDER BY id")
                .fetch_all(&mut b)
                .await
                .unwrap();
        assert_eq!(original, copied);
        a.close().await.unwrap();
        b.close().await.unwrap();
        fixture.clean().await;
    }
    #[tokio::test]
    async fn cutover_deadline_interrupts_vm_work_on_both_private_connections() {
        let fixture = Fixture::new("CREATE TABLE items(id INTEGER PRIMARY KEY)").await;
        let mut copy = CopySession::prepare(
            fixture.source.clone(),
            fixture.target.clone(),
            CopyOptions::default(),
        )
        .await
        .unwrap();
        let started = std::time::Instant::now();
        copy.set_deadline(started + Duration::from_millis(5))
            .await
            .unwrap();
        let sql = "WITH RECURSIVE n(i) AS(VALUES(1) UNION ALL SELECT i+1 FROM n WHERE i<1000000000) SELECT SUM(i) FROM n";
        assert!(
            sqlx::query(sql)
                .fetch_one(copy.source.as_mut().unwrap())
                .await
                .is_err()
        );
        assert!(
            sqlx::query(sql)
                .fetch_one(copy.target.as_mut().unwrap())
                .await
                .is_err()
        );
        assert!(started.elapsed() < Duration::from_secs(3));
        copy.clear_deadline().await.unwrap();
        copy.cleanup_source().await.unwrap();
        copy.close().await.unwrap();
        fixture.clean().await;
    }
}
