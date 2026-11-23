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
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

/// Distinct statements whose PostgreSQL text is kept. The crate authors about
/// two thousand static `q()` sites; a few build SQL at run time and leak it, so
/// the cache is bounded and falls back to a fresh rewrite once full.
const STATIC_REWRITE_CAPACITY: usize = 4096;

/// Process-wide interned rewrites, keyed by the SQLite text. Each distinct
/// statement is rewritten and leaked once.
static INTERNED: LazyLock<Mutex<HashMap<&'static str, &'static str>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

thread_local! {
    /// Lock-free fast path keyed by the static text's address and length.
    static LOCAL: RefCell<HashMap<(usize, usize), &'static str>> =
        RefCell::new(HashMap::new());
}

/// The PostgreSQL text of a static statement authored in the SQLite `?n`
/// style. Identical to [`placeholders`] with [`Dialect::Postgres`], but the
/// rewrite runs once per distinct statement instead of allocating per call.
pub(crate) fn postgres_static(sql: &'static str) -> std::borrow::Cow<'static, str> {
    let key = (sql.as_ptr() as usize, sql.len());
    if let Some(hit) = LOCAL.with(|local| local.borrow().get(&key).copied()) {
        return std::borrow::Cow::Borrowed(hit);
    }
    let interned = {
        let mut interned = INTERNED.lock().unwrap_or_else(|p| p.into_inner());
        match interned.get(sql) {
            Some(text) => Some(*text),
            None if interned.len() < STATIC_REWRITE_CAPACITY => {
                let text: &'static str =
                    Box::leak(placeholders(sql, Dialect::Postgres).into_boxed_str());
                interned.insert(sql, text);
                Some(text)
            }
            None => None,
        }
    };
    match interned {
        Some(text) => {
            LOCAL.with(|local| {
                let mut local = local.borrow_mut();
                if local.len() < STATIC_REWRITE_CAPACITY {
                    local.insert(key, text);
                }
            });
            std::borrow::Cow::Borrowed(text)
        }
        None => std::borrow::Cow::Owned(placeholders(sql, Dialect::Postgres)),
    }
}

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
    fn static_postgres_rewrite_is_cached_and_identical() {
        const SQL: &str = "SELECT ?1 FROM t WHERE a = ?2 AND b = ?1 AND c LIKE '?x' AND d = ?10";
        let first = postgres_static(SQL);
        assert_eq!(first, placeholders(SQL, Dialect::Postgres));
        assert!(matches!(first, std::borrow::Cow::Borrowed(_)));
        let second = postgres_static(SQL);
        // The same leaked text is returned: no rewrite and no allocation per call.
        assert_eq!(first.as_ptr(), second.as_ptr());
        // Another thread finds the process-wide interned text.
        let other = std::thread::spawn(|| postgres_static(SQL).as_ptr() as usize)
            .join()
            .unwrap();
        assert_eq!(other, first.as_ptr() as usize);
    }

    #[test]
    fn distinct_texts_with_equal_content_share_one_rewrite() {
        let a: &'static str = Box::leak("SELECT ?1 /*dialect-share*/".to_string().into_boxed_str());
        let b: &'static str = Box::leak("SELECT ?1 /*dialect-share*/".to_string().into_boxed_str());
        assert_ne!(a.as_ptr(), b.as_ptr());
        assert_eq!(postgres_static(a).as_ptr(), postgres_static(b).as_ptr());
        assert_eq!(postgres_static(a), "SELECT $1 /*dialect-share*/");
    }

    #[test]
    fn postgres_preserves_multi_digit_numbers() {
        assert_eq!(
            placeholders("VALUES (?10, ?11, ?12)", Dialect::Postgres),
            "VALUES ($10, $11, $12)"
        );
    }
}
