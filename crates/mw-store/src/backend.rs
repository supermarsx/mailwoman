// SCAFFOLD (t6-e0): the pluggable-backend SEAM stub (plan §1.1, §2.1). This is
// unwired scaffolding only — the live `Store` still holds its `SqlitePool`
// directly and every existing query runs unchanged, so the SQLite-default path +
// all existing tests stay byte-identical. e1 refactors `Store` to hold a
// `Backend`, adds the `Postgres(PgPool)` variant (enabling sqlx's `postgres` +
// `tls-rustls` features in this crate's Cargo.toml), ports every query to both
// dialects via the [`crate::dialect`] helper, and implements `Store::open` DSN
// dispatch + `Store::migrate_from_sqlite`.
#![allow(dead_code)]

use sqlx::sqlite::SqlitePool;

/// The store backend behind the (unchanged) `Store` façade (plan §1.1).
///
/// Frozen shape (§2.1): `Sqlite(SqlitePool) | Postgres(PgPool)`. e0 stubs the
/// SQLite arm only; e1 adds the `Postgres(PgPool)` variant once the `postgres`
/// feature is enabled. Kept a distinct enum (not `sqlx::Any`) because `Any`'s
/// type coverage is too narrow for the BLOB/chrono columns (plan §1.1).
pub(crate) enum Backend {
    Sqlite(SqlitePool),
    // Postgres(sqlx::postgres::PgPool),  // added by e1
}

/// Which SQL dialect a query string must target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dialect {
    Sqlite,
    Postgres,
}

impl Backend {
    /// The dialect of this backend (drives per-backend query selection in e1).
    pub(crate) fn dialect(&self) -> Dialect {
        match self {
            Backend::Sqlite(_) => Dialect::Sqlite,
            // Backend::Postgres(_) => Dialect::Postgres,  // added by e1
        }
    }
}

/// Parse a DSN scheme into the backend it selects (`sqlite://…` vs
/// `postgres://…`). STUB: e1 wires this into `Store::open`.
pub(crate) fn dialect_for_dsn(dsn: &str) -> Option<Dialect> {
    if dsn.starts_with("postgres://") || dsn.starts_with("postgresql://") {
        Some(Dialect::Postgres)
    } else if dsn.starts_with("sqlite:") {
        Some(Dialect::Sqlite)
    } else {
        None
    }
}
