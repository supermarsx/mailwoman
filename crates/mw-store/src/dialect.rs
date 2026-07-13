// SCAFFOLD (t6-e0): the per-backend SQL-dialect helper SKELETON (plan §1.1,
// §2.1). Unwired scaffolding only — no existing query is ported here (that is
// e1's job). e1 fills these to translate the SQLite-authored queries to Postgres
// at the points the two dialects diverge:
//
//   * placeholders   `?1`               ↔ `$1`
//   * upsert         `INSERT OR REPLACE` / `ON CONFLICT … DO UPDATE`
//   * blob columns   `BLOB`             ↔ `BYTEA`
//   * autoincrement  `INTEGER PK`       ↔ `BIGSERIAL`
//   * pragmas        `PRAGMA …`         ↔ (none)
//
// The design (plan §1.1) is per-backend query STRINGS via this helper, NOT
// `sqlx::Any` (whose type coverage is too narrow for our BLOB/chrono columns).
#![allow(dead_code)]

use crate::backend::Dialect;

/// Rewrite positional placeholders in `sql` for `dialect`.
///
/// SQLite uses `?1, ?2, …`; Postgres uses `$1, $2, …`. e1 fills the real
/// rewrite (queries are AUTHORED in the SQLite `?n` style and translated here for
/// the Postgres backend). STUB: returns the input unchanged.
pub(crate) fn placeholders(sql: &str, _dialect: Dialect) -> String {
    // e1: translate `?n` → `$n` when `dialect == Postgres`.
    sql.to_string()
}

/// The upsert clause for `dialect` given the conflict target column(s).
/// STUB: e1 returns the SQLite `ON CONFLICT(col) DO UPDATE …` vs the Postgres
/// equivalent (and handles `INSERT OR REPLACE` call-sites).
pub(crate) fn upsert_clause(_conflict_columns: &str, _dialect: Dialect) -> String {
    // e1: return the dialect-correct ON CONFLICT / DO UPDATE tail.
    String::new()
}
