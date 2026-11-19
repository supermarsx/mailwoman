// V6 pluggable-backend seam (t6-e1; plan §1.1, §2.1). The `Store` façade holds a
// `Backend` and dispatches every query through the small helper layer below, so
// the *entire existing public API* runs identically on SQLite or Postgres without
// any `sqlx::Any` (its BLOB/chrono coverage is too narrow — plan §1.1).
//
// Design: queries are AUTHORED ONCE in the historical SQLite `?n` style. At exec
// time the [`Sql`] builder rewrites placeholders (`?n` → `$n`) for Postgres via
// [`crate::dialect`] and binds a homogeneous [`Arg`] vector against the concrete
// pool. Rows come back as a backend-tagged [`Row`] with typed getters. The two
// dialects only diverge in (a) placeholder style and (b) the DDL in `migrations`
// vs `migrations_pg` — every runtime SQL string here is textually valid on both
// (upserts use `ON CONFLICT … DO UPDATE`, which Postgres shares; integer/boolean
// columns are BIGINT on both so the i64 bind/read path is uniform; BLOB↔BYTEA
// both decode to `Vec<u8>`).

use sqlx::postgres::{PgArguments, PgPool, PgRow};
use sqlx::sqlite::{SqliteArguments, SqliteRow};
use sqlx::{Postgres, Row as _, Sqlite};

use crate::dialect;

/// The store backend behind the (unchanged) `Store` façade (plan §1.1, §2.1).
#[derive(Clone)]
pub(crate) enum Backend {
    Sqlite(crate::sqlite_live::LiveSqlite),
    Postgres(PgPool),
}

/// Which SQL dialect a query string must target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dialect {
    Sqlite,
    Postgres,
}

impl Backend {
    /// The dialect of this backend (drives placeholder rewriting).
    pub(crate) fn dialect(&self) -> Dialect {
        match self {
            Backend::Sqlite(_) => Dialect::Sqlite,
            Backend::Postgres(_) => Dialect::Postgres,
        }
    }

    /// Begin a transaction on the active backend.
    ///
    /// On SQLite the transaction takes the write lock at `BEGIN` (`BEGIN
    /// IMMEDIATE`). A deferred transaction starts as a reader and upgrades on its
    /// first write; when another connection has written in between, SQLite fails
    /// that upgrade at once with `SQLITE_BUSY` / `SQLITE_BUSY_SNAPSHOT` ("database
    /// is locked") and never consults `busy_timeout`, because waiting could not
    /// help a reader holding an old snapshot. Many store transactions read before
    /// they write (sign-in directory registration among them), and in engine mode
    /// the sync loop writes all the time, so logins failed with 503 under load
    /// (t33-e11). Taken up front, the lock is waited for like any other write.
    /// WAL readers outside a transaction are not blocked by it.
    pub(crate) async fn begin(&self) -> Result<Tx, sqlx::Error> {
        Ok(match self {
            Backend::Sqlite(p) => Tx::Sqlite(p.begin().await?),
            Backend::Postgres(p) => Tx::Postgres(p.begin().await?),
        })
    }
}

// ── parameters ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    use sqlx::TransactionManager;
    use std::{
        future::{Future, poll_fn},
        task::Poll,
        time::Duration,
    };

    #[tokio::test]
    async fn cancelling_immediate_begin_never_returns_an_open_transaction_to_pool() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let backend = Backend::Sqlite(crate::sqlite_live::LiveSqlite::unmanaged(pool.clone()));
        for polls in 1..=8 {
            for _ in 0..8 {
                let mut begin = Box::pin(backend.begin());
                for _ in 0..polls {
                    if let Poll::Ready(result) =
                        poll_fn(|cx| Poll::Ready(begin.as_mut().poll(cx))).await
                    {
                        drop(result.unwrap());
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
                drop(begin);
                let connection = tokio::time::timeout(Duration::from_secs(2), pool.acquire())
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    sqlx::sqlite::SqliteTransactionManager::get_transaction_depth(&connection),
                    0,
                    "cancelled BEGIN at poll {polls} left a transaction in the pool"
                );
            }
        }
    }
}

/// A single bound parameter, backend-agnostic. Kept owned so the built query can
/// outlive the caller's borrows across the `await`.
#[derive(Debug, Clone)]
pub(crate) enum Arg {
    Text(String),
    OptText(Option<String>),
    Int(i64),
    OptInt(Option<i64>),
    Blob(Vec<u8>),
    OptBlob(Option<Vec<u8>>),
}

/// Convert a call-site value into an [`Arg`], mirroring the ergonomics of
/// `sqlx::query(..).bind(x)` so the per-call-site diff is minimal.
pub(crate) trait IntoArg {
    fn into_arg(self) -> Arg;
}

impl IntoArg for Arg {
    fn into_arg(self) -> Arg {
        self
    }
}
impl IntoArg for &str {
    fn into_arg(self) -> Arg {
        Arg::Text(self.to_string())
    }
}
impl IntoArg for String {
    fn into_arg(self) -> Arg {
        Arg::Text(self)
    }
}
impl IntoArg for &String {
    fn into_arg(self) -> Arg {
        Arg::Text(self.clone())
    }
}
impl IntoArg for Option<&str> {
    fn into_arg(self) -> Arg {
        Arg::OptText(self.map(|s| s.to_string()))
    }
}
impl IntoArg for Option<String> {
    fn into_arg(self) -> Arg {
        Arg::OptText(self)
    }
}
impl IntoArg for i64 {
    fn into_arg(self) -> Arg {
        Arg::Int(self)
    }
}
impl IntoArg for Option<i64> {
    fn into_arg(self) -> Arg {
        Arg::OptInt(self)
    }
}
impl IntoArg for Vec<u8> {
    fn into_arg(self) -> Arg {
        Arg::Blob(self)
    }
}
impl IntoArg for &Vec<u8> {
    fn into_arg(self) -> Arg {
        Arg::Blob(self.clone())
    }
}
impl IntoArg for &[u8] {
    fn into_arg(self) -> Arg {
        Arg::Blob(self.to_vec())
    }
}
impl IntoArg for Option<Vec<u8>> {
    fn into_arg(self) -> Arg {
        Arg::OptBlob(self)
    }
}
impl IntoArg for Option<&[u8]> {
    fn into_arg(self) -> Arg {
        Arg::OptBlob(self.map(|b| b.to_vec()))
    }
}

// ── query builder ───────────────────────────────────────────────────────────

/// A pending query authored in the SQLite `?n` style. Terminal methods dispatch
/// on the [`Backend`] (or [`Tx`]), rewriting placeholders for Postgres.
pub(crate) struct Sql {
    sql: &'static str,
    args: Vec<Arg>,
    origin: &'static std::panic::Location<'static>,
}

/// Author a query (SQLite `?n` placeholder style; translated per-backend).
#[track_caller]
pub(crate) fn q(sql: &'static str) -> Sql {
    Sql {
        sql,
        args: Vec::new(),
        origin: std::panic::Location::caller(),
    }
}

impl Sql {
    pub(crate) fn bind(mut self, v: impl IntoArg) -> Self {
        self.args.push(v.into_arg());
        self
    }

    pub(crate) async fn execute(self, backend: &Backend) -> Result<u64, sqlx::Error> {
        let input_bytes = crate::database_growth::input_bytes(&self.args);
        let result = match backend {
            Backend::Sqlite(p) => Ok(build_sqlite(self.sql, self.args)
                .execute(p)
                .await?
                .rows_affected()),
            Backend::Postgres(p) => {
                let sql = dialect::placeholders(self.sql, Dialect::Postgres);
                Ok(build_pg(&sql, &self.args).execute(p).await?.rows_affected())
            }
        };
        if let Ok(rows) = &result {
            crate::database_growth::record_write_bytes(self.sql, self.origin, input_bytes, *rows);
        }
        result
    }

    pub(crate) async fn fetch_optional(
        self,
        backend: &Backend,
    ) -> Result<Option<Row>, sqlx::Error> {
        let input_bytes = crate::database_growth::input_bytes(&self.args);
        let result = match backend {
            Backend::Sqlite(p) => Ok(build_sqlite(self.sql, self.args)
                .fetch_optional(p)
                .await?
                .map(Row::Sqlite)),
            Backend::Postgres(p) => {
                let sql = dialect::placeholders(self.sql, Dialect::Postgres);
                Ok(build_pg(&sql, &self.args)
                    .fetch_optional(p)
                    .await?
                    .map(Row::Postgres))
            }
        };
        if let Ok(rows) = &result {
            crate::database_growth::record_write_bytes(
                self.sql,
                self.origin,
                input_bytes,
                u64::from(rows.is_some()),
            );
        }
        result
    }

    pub(crate) async fn fetch_one(self, backend: &Backend) -> Result<Row, sqlx::Error> {
        let input_bytes = crate::database_growth::input_bytes(&self.args);
        let result = match backend {
            Backend::Sqlite(p) => Ok(Row::Sqlite(
                build_sqlite(self.sql, self.args).fetch_one(p).await?,
            )),
            Backend::Postgres(p) => {
                let sql = dialect::placeholders(self.sql, Dialect::Postgres);
                Ok(Row::Postgres(
                    build_pg(&sql, &self.args).fetch_one(p).await?,
                ))
            }
        };
        if result.is_ok() {
            crate::database_growth::record_write_bytes(self.sql, self.origin, input_bytes, 1);
        }
        result
    }

    pub(crate) async fn fetch_all(self, backend: &Backend) -> Result<Vec<Row>, sqlx::Error> {
        let input_bytes = crate::database_growth::input_bytes(&self.args);
        let result = match backend {
            Backend::Sqlite(p) => Ok(build_sqlite(self.sql, self.args)
                .fetch_all(p)
                .await?
                .into_iter()
                .map(Row::Sqlite)
                .collect::<Vec<Row>>()),
            Backend::Postgres(p) => {
                let sql = dialect::placeholders(self.sql, Dialect::Postgres);
                Ok(build_pg(&sql, &self.args)
                    .fetch_all(p)
                    .await?
                    .into_iter()
                    .map(Row::Postgres)
                    .collect::<Vec<Row>>())
            }
        };
        if let Ok(rows) = &result {
            crate::database_growth::record_write_bytes(
                self.sql,
                self.origin,
                input_bytes,
                rows.len() as u64,
            );
        }
        result
    }

    /// A single-column `i64` scalar (`COUNT`/`MAX`/`RETURNING state`).
    pub(crate) async fn fetch_scalar_i64(self, backend: &Backend) -> Result<i64, sqlx::Error> {
        self.fetch_one(backend).await?.try_get_i64_idx(0)
    }

    /// An optional single-column text scalar (id lookups).
    pub(crate) async fn fetch_opt_scalar_string(
        self,
        backend: &Backend,
    ) -> Result<Option<String>, sqlx::Error> {
        Ok(self
            .fetch_optional(backend)
            .await?
            .map(|r| r.get_string_idx(0)))
    }

    /// A list of single-column text scalars (id lists, UIDL sets).
    pub(crate) async fn fetch_all_scalar_string(
        self,
        backend: &Backend,
    ) -> Result<Vec<String>, sqlx::Error> {
        Ok(self
            .fetch_all(backend)
            .await?
            .iter()
            .map(|r| r.get_string_idx(0))
            .collect())
    }

    // ---- transaction variants ------------------------------------------------

    pub(crate) async fn fetch_all_tx(self, tx: &mut Tx) -> Result<Vec<Row>, sqlx::Error> {
        let input_bytes = crate::database_growth::input_bytes(&self.args);
        let rows: Vec<Row> = match tx {
            Tx::Sqlite(t) => build_sqlite(self.sql, self.args)
                .fetch_all(&mut **t)
                .await?
                .into_iter()
                .map(Row::Sqlite)
                .collect(),
            Tx::Postgres(t) => {
                let sql = dialect::placeholders(self.sql, Dialect::Postgres);
                build_pg(&sql, &self.args)
                    .fetch_all(&mut **t)
                    .await?
                    .into_iter()
                    .map(Row::Postgres)
                    .collect()
            }
        };
        crate::database_growth::record_write_bytes(
            self.sql,
            self.origin,
            input_bytes,
            rows.len() as u64,
        );
        Ok(rows)
    }

    pub(crate) async fn execute_tx(self, tx: &mut Tx) -> Result<u64, sqlx::Error> {
        let input_bytes = crate::database_growth::input_bytes(&self.args);
        let result = match tx {
            Tx::Sqlite(t) => Ok(build_sqlite(self.sql, self.args)
                .execute(&mut **t)
                .await?
                .rows_affected()),
            Tx::Postgres(t) => {
                let sql = dialect::placeholders(self.sql, Dialect::Postgres);
                Ok(build_pg(&sql, &self.args)
                    .execute(&mut **t)
                    .await?
                    .rows_affected())
            }
        };
        if let Ok(rows) = &result {
            crate::database_growth::record_write_bytes(self.sql, self.origin, input_bytes, *rows);
        }
        result
    }

    pub(crate) async fn fetch_opt_scalar_string_tx(
        self,
        tx: &mut Tx,
    ) -> Result<Option<String>, sqlx::Error> {
        let input_bytes = crate::database_growth::input_bytes(&self.args);
        let row = match tx {
            Tx::Sqlite(t) => build_sqlite(self.sql, self.args)
                .fetch_optional(&mut **t)
                .await?
                .map(Row::Sqlite),
            Tx::Postgres(t) => {
                let sql = dialect::placeholders(self.sql, Dialect::Postgres);
                build_pg(&sql, &self.args)
                    .fetch_optional(&mut **t)
                    .await?
                    .map(Row::Postgres)
            }
        };
        crate::database_growth::record_write_bytes(
            self.sql,
            self.origin,
            input_bytes,
            u64::from(row.is_some()),
        );
        Ok(row.map(|r| r.get_string_idx(0)))
    }
}

// SQLite's worker needs owned arguments. Transfer the payload allocations into
// SQLx instead of borrowing them (which SQLx would copy before worker dispatch).
fn build_sqlite<'a>(
    sql: &'a str,
    args: Vec<Arg>,
) -> sqlx::query::Query<'a, Sqlite, SqliteArguments<'a>> {
    let mut q = sqlx::query::<Sqlite>(sql);
    for a in args {
        q = match a {
            Arg::Text(s) => q.bind(s),
            Arg::OptText(s) => q.bind(s),
            Arg::Int(i) => q.bind(i),
            Arg::OptInt(i) => q.bind(i),
            Arg::Blob(b) => q.bind(b),
            Arg::OptBlob(b) => q.bind(b),
        };
    }
    q
}

// PostgreSQL encodes borrowed payloads directly into its argument buffer while
// the owning Sql remains alive through this terminal future.
fn build_pg<'a>(sql: &'a str, args: &'a [Arg]) -> sqlx::query::Query<'a, Postgres, PgArguments> {
    let mut q = sqlx::query::<Postgres>(sql);
    for a in args {
        q = match a {
            Arg::Text(s) => q.bind(s.as_str()),
            Arg::OptText(s) => q.bind(s.as_deref()),
            Arg::Int(i) => q.bind(*i),
            Arg::OptInt(i) => q.bind(*i),
            Arg::Blob(b) => q.bind(b.as_slice()),
            Arg::OptBlob(b) => q.bind(b.as_deref()),
        };
    }
    q
}

// ── rows ────────────────────────────────────────────────────────────────────

/// A backend-tagged row with typed getters. Column types are aligned across the
/// two schemas (TEXT/BIGINT/BYTEA) so the getters need no per-backend branching
/// beyond the enum match.
pub(crate) enum Row {
    Sqlite(SqliteRow),
    Postgres(PgRow),
}

impl Row {
    pub(crate) fn get_string(&self, col: &str) -> String {
        match self {
            Row::Sqlite(r) => r.get(col),
            Row::Postgres(r) => r.get(col),
        }
    }

    pub(crate) fn get_opt_string(&self, col: &str) -> Option<String> {
        match self {
            Row::Sqlite(r) => r.get(col),
            Row::Postgres(r) => r.get(col),
        }
    }

    pub(crate) fn get_i64(&self, col: &str) -> i64 {
        match self {
            Row::Sqlite(r) => r.get(col),
            Row::Postgres(r) => r.get(col),
        }
    }

    pub(crate) fn get_opt_i64(&self, col: &str) -> Option<i64> {
        match self {
            Row::Sqlite(r) => r.get(col),
            Row::Postgres(r) => r.get(col),
        }
    }

    /// One column as JSON whatever its storage type (text, integer, real, boolean;
    /// a blob becomes `null`). Used by the table-driven export, which must not
    /// panic on a column whose declared type differs between the dialects.
    pub(crate) fn get_json(&self, col: &str) -> serde_json::Value {
        use serde_json::Value;
        match self {
            Row::Sqlite(r) => {
                if let Ok(v) = r.try_get::<Option<String>, _>(col) {
                    return v.map_or(Value::Null, Value::String);
                }
                if let Ok(v) = r.try_get::<Option<i64>, _>(col) {
                    return v.map_or(Value::Null, Value::from);
                }
                if let Ok(v) = r.try_get::<Option<f64>, _>(col) {
                    return v.map_or(Value::Null, Value::from);
                }
                Value::Null
            }
            Row::Postgres(r) => {
                if let Ok(v) = r.try_get::<Option<String>, _>(col) {
                    return v.map_or(Value::Null, Value::String);
                }
                if let Ok(v) = r.try_get::<Option<i64>, _>(col) {
                    return v.map_or(Value::Null, Value::from);
                }
                if let Ok(v) = r.try_get::<Option<i32>, _>(col) {
                    return v.map_or(Value::Null, Value::from);
                }
                if let Ok(v) = r.try_get::<Option<bool>, _>(col) {
                    return v.map_or(Value::Null, Value::from);
                }
                if let Ok(v) = r.try_get::<Option<f64>, _>(col) {
                    return v.map_or(Value::Null, Value::from);
                }
                Value::Null
            }
        }
    }

    pub(crate) fn get_blob(&self, col: &str) -> Vec<u8> {
        match self {
            Row::Sqlite(r) => r.get(col),
            Row::Postgres(r) => r.get(col),
        }
    }

    pub(crate) fn get_opt_blob(&self, col: &str) -> Option<Vec<u8>> {
        match self {
            Row::Sqlite(r) => r.get(col),
            Row::Postgres(r) => r.get(col),
        }
    }

    /// Positional `i64` (for anonymous scalar columns like `COALESCE(MAX(..),0)`),
    /// tolerant of Postgres returning `int4` for a bare integer literal.
    ///
    /// Fallible on purpose (t25-e3). This ran as `.expect("scalar column is an
    /// integer")`, which made it the one panic in this crate that fires at
    /// **query** time rather than at boot: a schema or migration drift that leaves
    /// a scalar column `NUMERIC` or `TEXT`, or a `COUNT` typed differently than
    /// assumed, turned a recoverable decode failure into a panic inside a request
    /// path. The `sqlx` decode error is a perfectly good `StoreError::Db`, so it
    /// is now returned. The SQLite arm was `r.get(..)`, which panics the same way,
    /// and is converted with it.
    pub(crate) fn try_get_i64_idx(&self, idx: usize) -> Result<i64, sqlx::Error> {
        match self {
            Row::Sqlite(r) => r.try_get::<i64, _>(idx),
            Row::Postgres(r) => r
                .try_get::<i64, _>(idx)
                .or_else(|_| r.try_get::<i32, _>(idx).map(i64::from)),
        }
    }

    /// Infallible wrapper kept for the **one** caller that cannot yet propagate:
    /// `v2.rs`'s `RETURNING state` read. That file was locked by another lane when
    /// this landed, so the `?` there is a one-line follow-up rather than part of
    /// this change; the panic is still reachable through that path alone.
    /// Everything routed through [`Q::fetch_scalar_i64`] is now fallible.
    pub(crate) fn get_i64_idx(&self, idx: usize) -> i64 {
        self.try_get_i64_idx(idx)
            .expect("scalar column is an integer")
    }

    /// Positional text (for anonymous single-column scalar selects).
    pub(crate) fn get_string_idx(&self, idx: usize) -> String {
        match self {
            Row::Sqlite(r) => r.get(idx),
            Row::Postgres(r) => r.get(idx),
        }
    }
}

// ── transactions ────────────────────────────────────────────────────────────

/// A backend-tagged transaction (pool-derived, hence `'static`).
pub(crate) enum Tx {
    Sqlite(crate::sqlite_live::LiveTransaction),
    Postgres(sqlx::Transaction<'static, Postgres>),
}

impl Tx {
    pub(crate) async fn commit(self) -> Result<(), sqlx::Error> {
        match self {
            Tx::Sqlite(t) => t.commit().await,
            Tx::Postgres(t) => t.commit().await,
        }
    }
}

#[cfg(test)]
mod binding_tests {
    use super::*;
    async fn exercise(backend: Backend, sqlite: bool) {
        q("CREATE TEMP TABLE binding_probe(id BIGINT PRIMARY KEY,text_value TEXT,optional_text TEXT,blob_value BYTEA,optional_blob BYTEA,integer_value BIGINT,optional_integer BIGINT)").execute(&backend).await.unwrap();
        let payload: Vec<u8> = (0..131_072).map(|n| (n % 256) as u8).collect();
        let insert = {
            let text = "owned café 😀".to_string();
            let bytes = payload.clone();
            q("INSERT INTO binding_probe VALUES(?1,?2,?3,?4,?5,?6,?7)")
                .bind(1i64)
                .bind(text)
                .bind(None::<String>)
                .bind(bytes)
                .bind(None::<Vec<u8>>)
                .bind(i64::MIN)
                .bind(None::<i64>)
        };
        insert.execute(&backend).await.unwrap();
        let mut tx = backend.begin().await.unwrap();
        q("INSERT INTO binding_probe VALUES(?1,?2,?3,?4,?5,?6,?7)")
            .bind(2i64)
            .bind(String::new())
            .bind(Some(String::new()))
            .bind(Vec::<u8>::new())
            .bind(Some(vec![0u8, 255, 17]))
            .bind(i64::MAX)
            .bind(Some(0i64))
            .execute_tx(&mut tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        // The consumed query and its owned input strings/vectors are already gone.
        let rows = q("SELECT * FROM binding_probe ORDER BY id")
            .fetch_all(&backend)
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get_string("text_value"), "owned café 😀");
        assert_eq!(rows[0].get_opt_string("optional_text"), None);
        assert_eq!(rows[0].get_blob("blob_value"), payload);
        assert_eq!(rows[0].get_opt_blob("optional_blob"), None);
        assert_eq!(rows[0].get_i64("integer_value"), i64::MIN);
        assert_eq!(rows[0].get_opt_i64("optional_integer"), None);
        assert_eq!(rows[1].get_string("text_value"), "");
        assert_eq!(rows[1].get_opt_string("optional_text"), Some(String::new()));
        assert!(rows[1].get_blob("blob_value").is_empty());
        assert_eq!(
            rows[1].get_opt_blob("optional_blob"),
            Some(vec![0u8, 255, 17])
        );
        assert_eq!(rows[1].get_i64("integer_value"), i64::MAX);
        assert_eq!(rows[1].get_opt_i64("optional_integer"), Some(0));
        let nul = q("INSERT INTO binding_probe(id,text_value) VALUES(?1,?2)")
            .bind(3i64)
            .bind("before\0after".to_string())
            .execute(&backend)
            .await;
        if sqlite {
            nul.unwrap();
            let text = q("SELECT text_value FROM binding_probe WHERE id=3")
                .fetch_opt_scalar_string(&backend)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(text.as_bytes(), b"before\0after");
        } else {
            // PostgreSQL TEXT forbids NUL; borrowing must preserve that refusal,
            // while BYTEA above retains all 256 byte values unchanged.
            assert!(nul.is_err());
        }
        match backend {
            Backend::Sqlite(pool) => pool.close().await,
            Backend::Postgres(pool) => pool.close().await,
        };
        // Owned result rows stay usable after all connections have closed.
        assert_eq!(rows[0].get_string("text_value"), "owned café 😀");
    }
    #[tokio::test]
    async fn driver_bindings_preserve_sqlite_nulls_bytes_and_owned_rows() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        exercise(Backend::Sqlite(pool.into()), true).await;
    }
    #[tokio::test]
    async fn driver_bindings_preserve_postgres_nulls_bytes_and_owned_rows() {
        let url = std::env::var("MW_TEST_PG")
            .ok()
            .or_else(|| std::env::var("DATABASE_URL_PG").ok());
        let Some(url) = url else {
            mw_test_gate::skip(
                "backend binding PostgreSQL parity: DATABASE_URL_PG/MW_TEST_PG not configured",
            );
            return;
        };
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await
            .unwrap();
        exercise(Backend::Postgres(pool), false).await;
    }
}
