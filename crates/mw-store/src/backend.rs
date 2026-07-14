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
use sqlx::sqlite::{SqliteArguments, SqlitePool, SqliteRow};
use sqlx::{Postgres, Row as _, Sqlite};

use crate::dialect;

/// The store backend behind the (unchanged) `Store` façade (plan §1.1, §2.1).
#[derive(Clone)]
pub(crate) enum Backend {
    Sqlite(SqlitePool),
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
    pub(crate) async fn begin(&self) -> Result<Tx, sqlx::Error> {
        Ok(match self {
            Backend::Sqlite(p) => Tx::Sqlite(p.begin().await?),
            Backend::Postgres(p) => Tx::Postgres(p.begin().await?),
        })
    }
}

// ── parameters ──────────────────────────────────────────────────────────────

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
}

/// Author a query (SQLite `?n` placeholder style; translated per-backend).
pub(crate) fn q(sql: &'static str) -> Sql {
    Sql {
        sql,
        args: Vec::new(),
    }
}

impl Sql {
    pub(crate) fn bind(mut self, v: impl IntoArg) -> Self {
        self.args.push(v.into_arg());
        self
    }

    pub(crate) async fn execute(self, backend: &Backend) -> Result<u64, sqlx::Error> {
        match backend {
            Backend::Sqlite(p) => Ok(build_sqlite(self.sql, &self.args)
                .execute(p)
                .await?
                .rows_affected()),
            Backend::Postgres(p) => {
                let sql = dialect::placeholders(self.sql, Dialect::Postgres);
                Ok(build_pg(&sql, &self.args).execute(p).await?.rows_affected())
            }
        }
    }

    pub(crate) async fn fetch_optional(
        self,
        backend: &Backend,
    ) -> Result<Option<Row>, sqlx::Error> {
        match backend {
            Backend::Sqlite(p) => Ok(build_sqlite(self.sql, &self.args)
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
        }
    }

    pub(crate) async fn fetch_one(self, backend: &Backend) -> Result<Row, sqlx::Error> {
        match backend {
            Backend::Sqlite(p) => Ok(Row::Sqlite(
                build_sqlite(self.sql, &self.args).fetch_one(p).await?,
            )),
            Backend::Postgres(p) => {
                let sql = dialect::placeholders(self.sql, Dialect::Postgres);
                Ok(Row::Postgres(
                    build_pg(&sql, &self.args).fetch_one(p).await?,
                ))
            }
        }
    }

    pub(crate) async fn fetch_all(self, backend: &Backend) -> Result<Vec<Row>, sqlx::Error> {
        match backend {
            Backend::Sqlite(p) => Ok(build_sqlite(self.sql, &self.args)
                .fetch_all(p)
                .await?
                .into_iter()
                .map(Row::Sqlite)
                .collect()),
            Backend::Postgres(p) => {
                let sql = dialect::placeholders(self.sql, Dialect::Postgres);
                Ok(build_pg(&sql, &self.args)
                    .fetch_all(p)
                    .await?
                    .into_iter()
                    .map(Row::Postgres)
                    .collect())
            }
        }
    }

    /// A single-column `i64` scalar (`COUNT`/`MAX`/`RETURNING state`).
    pub(crate) async fn fetch_scalar_i64(self, backend: &Backend) -> Result<i64, sqlx::Error> {
        Ok(self.fetch_one(backend).await?.get_i64_idx(0))
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

    pub(crate) async fn execute_tx(self, tx: &mut Tx) -> Result<u64, sqlx::Error> {
        match tx {
            Tx::Sqlite(t) => Ok(build_sqlite(self.sql, &self.args)
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
        }
    }

    pub(crate) async fn fetch_opt_scalar_string_tx(
        self,
        tx: &mut Tx,
    ) -> Result<Option<String>, sqlx::Error> {
        let row = match tx {
            Tx::Sqlite(t) => build_sqlite(self.sql, &self.args)
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
        Ok(row.map(|r| r.get_string_idx(0)))
    }
}

fn build_sqlite<'a>(
    sql: &'a str,
    args: &[Arg],
) -> sqlx::query::Query<'a, Sqlite, SqliteArguments<'a>> {
    let mut q = sqlx::query::<Sqlite>(sql);
    for a in args {
        q = match a {
            Arg::Text(s) => q.bind(s.clone()),
            Arg::OptText(s) => q.bind(s.clone()),
            Arg::Int(i) => q.bind(*i),
            Arg::OptInt(i) => q.bind(*i),
            Arg::Blob(b) => q.bind(b.clone()),
            Arg::OptBlob(b) => q.bind(b.clone()),
        };
    }
    q
}

fn build_pg<'a>(sql: &'a str, args: &[Arg]) -> sqlx::query::Query<'a, Postgres, PgArguments> {
    let mut q = sqlx::query::<Postgres>(sql);
    for a in args {
        q = match a {
            Arg::Text(s) => q.bind(s.clone()),
            Arg::OptText(s) => q.bind(s.clone()),
            Arg::Int(i) => q.bind(*i),
            Arg::OptInt(i) => q.bind(*i),
            Arg::Blob(b) => q.bind(b.clone()),
            Arg::OptBlob(b) => q.bind(b.clone()),
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

    /// Positional `i64` (for anonymous scalar columns like `COALESCE(MAX(..),0)`).
    /// Tolerant of Postgres returning `int4` for a bare integer literal.
    pub(crate) fn get_i64_idx(&self, idx: usize) -> i64 {
        match self {
            Row::Sqlite(r) => r.get::<i64, _>(idx),
            Row::Postgres(r) => r
                .try_get::<i64, _>(idx)
                .or_else(|_| r.try_get::<i32, _>(idx).map(i64::from))
                .expect("scalar column is an integer"),
        }
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
    Sqlite(sqlx::Transaction<'static, Sqlite>),
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
