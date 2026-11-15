//! Same-installation store moves preserve configuration, identities and grants.
//! Unlike cross-installation migration, nothing is deliberately omitted. Schema
//! parity, an empty destination, and content parity are checked before commit.
use crate::{
    MigrationReport, Store, StoreError,
    backend::{Arg, Dialect, Row, Tx},
    q,
};
use futures_util::TryStreamExt;
use sha2::{Digest, Sha256};
use sqlx::{
    Row as _,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{collections::BTreeSet, str::FromStr};

fn identifier(value: &str) -> Result<String, StoreError> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(StoreError::Invalid("unsupported migration identifier"));
    }
    Ok(format!("\"{value}\""))
}

#[derive(Clone, Copy)]
enum Kind {
    Text,
    Int,
    Blob,
}
fn kind(value: &str) -> Result<Kind, StoreError> {
    match value.to_ascii_uppercase().as_str() {
        "TEXT" => Ok(Kind::Text),
        "INTEGER" | "BIGINT" => Ok(Kind::Int),
        "BLOB" | "BYTEA" => Ok(Kind::Blob),
        _ => Err(StoreError::Invalid("unsupported migration column type")),
    }
}
fn value(row: &Row, name: &str, kind: Kind) -> Result<Arg, sqlx::Error> {
    macro_rules! get {
        ($ty:ty) => {
            match row {
                Row::Sqlite(r) => r.try_get::<Option<$ty>, _>(name),
                Row::Postgres(r) => r.try_get::<Option<$ty>, _>(name),
            }
        };
    }
    Ok(match kind {
        Kind::Text => Arg::OptText(get!(String)?),
        Kind::Int => Arg::OptInt(match row {
            Row::Sqlite(r) => r.try_get::<Option<i64>, _>(name)?,
            Row::Postgres(r) => r
                .try_get::<Option<i64>, _>(name)
                .or_else(|_| r.try_get::<Option<i32>, _>(name).map(|v| v.map(i64::from)))
                .or_else(|_| r.try_get::<Option<i16>, _>(name).map(|v| v.map(i64::from)))?,
        }),
        Kind::Blob => Arg::OptBlob(get!(Vec<u8>)?),
    })
}
fn row_bytes(row: &Row, columns: &[(String, Kind)]) -> Result<Vec<u8>, sqlx::Error> {
    let mut bytes = Vec::new();
    for (name, kind) in columns {
        let data = match value(row, name, *kind)? {
            Arg::OptText(v) => v.map(String::into_bytes),
            Arg::OptInt(v) => v.map(|v| v.to_le_bytes().to_vec()),
            Arg::OptBlob(v) => v,
            _ => unreachable!(),
        };
        match data {
            None => bytes.push(0),
            Some(data) => {
                bytes.push(1);
                bytes.extend_from_slice(&(data.len() as u64).to_le_bytes());
                bytes.extend(data);
            }
        }
    }
    Ok(bytes)
}

async fn pg_rows(tx: &mut Tx, sql: &str) -> Result<Vec<Row>, sqlx::Error> {
    let Tx::Postgres(pg) = tx else { unreachable!() };
    Ok(sqlx::query(sql)
        .fetch_all(&mut **pg)
        .await?
        .into_iter()
        .map(Row::Postgres)
        .collect())
}

impl Store {
    /// Move the complete installation to an empty PostgreSQL store. The source
    /// must be offline; the deployment script supplies a disposable volume copy.
    /// Every table except the backend's own migration ledger is copied. The
    /// destination transaction is rolled back on any schema, count or byte mismatch.
    pub async fn migrate_complete_from_sqlite(
        &self,
        source: &str,
    ) -> Result<MigrationReport, StoreError> {
        if self.backend.dialect() != Dialect::Postgres {
            return Err(StoreError::Invalid(
                "complete migration requires PostgreSQL",
            ));
        }
        let url = if source.starts_with("sqlite:") {
            source.to_owned()
        } else {
            format!("sqlite://{source}")
        };
        let mut options = SqliteConnectOptions::from_str(&url)?.read_only(true);
        if let Some(file) = crate::encryption::sqlite_file_from_dsn(source)
            && let Some(key) = crate::encryption::source_key(&file)?
        {
            options = options.pragma("key", crate::encryption::key_pragma(&key));
        }
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        let mut source_tx = pool.begin().await?;
        let mut names: Vec<String> = sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table' AND substr(name,1,7) <> 'sqlite_' AND name <> '_sqlx_migrations' ORDER BY name")
            .fetch_all(&mut *source_tx).await?;
        names.sort_by_key(|name| (name == "settings_revision", name.clone()));
        let target_names: BTreeSet<String> = q("SELECT table_name AS name FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE' AND table_name <> '_sqlx_migrations'")
            .fetch_all(&self.backend).await?.iter().map(|r| r.get_string("name")).collect();
        if names.iter().cloned().collect::<BTreeSet<_>>() != target_names {
            return Err(StoreError::Invalid(
                "source and destination schemas differ; upgrade the offline source copy first",
            ));
        }
        let mut tx = self.backend.begin().await?;
        q("SET CONSTRAINTS ALL DEFERRED")
            .execute_tx(&mut tx)
            .await?;
        let locks = names
            .iter()
            .map(|n| identifier(n))
            .collect::<Result<Vec<_>, _>>()?
            .join(",");
        let Tx::Postgres(ref mut pg) = tx else {
            unreachable!()
        };
        sqlx::query(&format!("LOCK TABLE {locks} IN ACCESS EXCLUSIVE MODE"))
            .execute(&mut **pg)
            .await?;
        // Check every table before writing. The serialization and revision singletons are seeded
        // by migrations; all other application tables must be empty.
        for name in &names {
            let table = identifier(name)?;
            let rows = pg_rows(&mut tx, &format!("SELECT * FROM {table} LIMIT 2")).await?;
            let seeded = rows.len() == 1
                && matches!(value(&rows[0], "id", Kind::Int), Ok(Arg::OptInt(Some(1))))
                && (name == "identity_serialization"
                    || (name == "settings_revision"
                        && matches!(value(&rows[0], "n", Kind::Int), Ok(Arg::OptInt(Some(0))))));
            if !rows.is_empty() && !seeded {
                return Err(StoreError::Invalid(
                    "complete migration requires an empty destination",
                ));
            }
        }
        q("DELETE FROM identity_serialization")
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM settings_revision")
            .execute_tx(&mut tx)
            .await?;
        let mut report = MigrationReport::default();
        for name in names {
            let table = identifier(&name)?;
            let schema = sqlx::query(&format!("PRAGMA table_info({table})"))
                .fetch_all(&mut *source_tx)
                .await?;
            let columns: Vec<(String, Kind)> = schema
                .iter()
                .map(|r| Ok((r.try_get("name")?, kind(r.try_get::<&str, _>("type")?)?)))
                .collect::<Result<_, StoreError>>()?;
            let target_columns: BTreeSet<String> = q("SELECT column_name AS name FROM information_schema.columns WHERE table_schema='public' AND table_name=?1")
                .bind(name.clone()).fetch_all_tx(&mut tx).await?.iter().map(|r| r.get_string("name")).collect();
            if columns
                .iter()
                .map(|(n, _)| n.clone())
                .collect::<BTreeSet<_>>()
                != target_columns
            {
                return Err(StoreError::Invalid("source and destination columns differ"));
            }
            let fields = columns
                .iter()
                .map(|(n, _)| identifier(n))
                .collect::<Result<Vec<_>, _>>()?
                .join(",");
            let select = format!("SELECT {fields} FROM {table}");
            let slots = (1..=columns.len())
                .map(|i| format!("?{i}"))
                .collect::<Vec<_>>()
                .join(",");
            let insert = format!("INSERT INTO {table}({fields}) VALUES({slots})");
            let insert_pg = crate::dialect::placeholders(&insert, Dialect::Postgres);
            let mut source_rows = sqlx::query(&select).fetch(&mut *source_tx);
            let mut count = 0;
            while let Some(source_row) = source_rows.try_next().await? {
                let row = Row::Sqlite(source_row);
                let Tx::Postgres(ref mut pg) = tx else {
                    unreachable!()
                };
                let mut query = sqlx::query(&insert_pg);
                for (column, kind) in &columns {
                    query = match value(&row, column, *kind)? {
                        Arg::OptText(v) => query.bind(v),
                        Arg::OptInt(v) => query.bind(v),
                        Arg::OptBlob(v) => query.bind(v),
                        _ => unreachable!(),
                    };
                }
                query.execute(&mut **pg).await?;
                count += 1;
            }
            drop(source_rows);
            report.tables.push((name, count));
        }
        // Verify after every insert and trigger has run, while both source and
        // destination are still held in their transactions.
        for (name, _) in &report.tables {
            let table = identifier(name)?;
            let schema = sqlx::query(&format!("PRAGMA table_info({table})"))
                .fetch_all(&mut *source_tx)
                .await?;
            let columns: Vec<(String, Kind)> = schema
                .iter()
                .map(|r| Ok((r.try_get("name")?, kind(r.try_get::<&str, _>("type")?)?)))
                .collect::<Result<_, StoreError>>()?;
            let select = format!("SELECT * FROM {table}");
            // Retain only row hashes for unordered comparison, never whole
            // message bodies or attachment payloads from the entire table.
            let mut before: Vec<[u8; 32]> = Vec::new();
            let mut source_rows = sqlx::query(&select).fetch(&mut *source_tx);
            while let Some(row) = source_rows.try_next().await? {
                before.push(Sha256::digest(row_bytes(&Row::Sqlite(row), &columns)?).into());
            }
            drop(source_rows);
            let mut after: Vec<[u8; 32]> = Vec::new();
            let Tx::Postgres(ref mut pg) = tx else {
                unreachable!()
            };
            let mut target_rows = sqlx::query(&select).fetch(&mut **pg);
            while let Some(row) = target_rows.try_next().await? {
                after.push(Sha256::digest(row_bytes(&Row::Postgres(row), &columns)?).into());
            }
            drop(target_rows);
            before.sort_unstable();
            after.sort_unstable();
            if before != after {
                return Err(StoreError::Invalid("migration content verification failed"));
            }
        }
        tx.commit().await?;
        source_tx.rollback().await?;
        Ok(report)
    }
}
