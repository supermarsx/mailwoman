// Per-backend SQL dialect helper (t6-e1; plan §1.1, §2.1). Queries are AUTHORED
// once in the SQLite `?n` placeholder style; this module translates them to the
// Postgres `$n` style at exec time. The other dialect divergences are handled
// structurally rather than by string rewriting:
//
//   * upsert       — every runtime upsert uses `ON CONFLICT(col) DO UPDATE SET
//                    x = excluded.x`, which BOTH SQLite and Postgres accept
//                    verbatim (no `INSERT OR REPLACE` remains in the runtime path).
//   * blob columns — `BLOB` (SQLite) / `BYTEA` (Postgres); both decode to
//                    `Vec<u8>` via sqlx, so bind/read code is uniform.
//   * integers     — declared `BIGINT` in the Postgres schema so the i64
//                    bind/read path (incl. boolean-as-0/1) needs no branching.
//   * pragmas / autoincrement — live only in the migration DDL, never at runtime.
//
// The remaining case-sensitivity divergence (SQLite `LIKE` is ASCII
// case-insensitive; Postgres `LIKE` is not) is handled at the one call site that
// needs it (`autocomplete_contacts`), which selects a `LIKE`/`ILIKE` variant on
// the backend dialect.

use crate::backend::Dialect;

/// Rewrite positional placeholders in `sql` for `dialect`.
///
/// SQLite uses `?1, ?2, …`; Postgres uses `$1, $2, …`. A `?` is only rewritten
/// when immediately followed by a digit (our placeholders are always `?<n>`),
/// leaving any other `?` untouched. Placeholder *numbers* are preserved, so a
/// query that reuses `?1` twice becomes a query that reuses `$1` twice — bound
/// once, positionally, on both backends.
pub(crate) fn placeholders(sql: &str, dialect: Dialect) -> String {
    match dialect {
        Dialect::Sqlite => sql.to_string(),
        Dialect::Postgres => {
            let mut out = String::with_capacity(sql.len());
            let mut chars = sql.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '?' && chars.peek().is_some_and(|n| n.is_ascii_digit()) {
                    out.push('$');
                } else {
                    out.push(c);
                }
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_placeholders_unchanged() {
        assert_eq!(
            placeholders("SELECT ?1 WHERE x = ?2", Dialect::Sqlite),
            "SELECT ?1 WHERE x = ?2"
        );
    }

    #[test]
    fn postgres_placeholders_rewritten_and_reused() {
        assert_eq!(
            placeholders("INSERT INTO t VALUES (?1, ?2, ?1)", Dialect::Postgres),
            "INSERT INTO t VALUES ($1, $2, $1)"
        );
    }

    #[test]
    fn postgres_preserves_multi_digit_numbers() {
        assert_eq!(
            placeholders("VALUES (?10, ?11, ?12)", Dialect::Postgres),
            "VALUES ($10, $11, $12)"
        );
    }
}
