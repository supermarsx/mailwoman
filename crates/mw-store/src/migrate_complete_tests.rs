//! Gate for the complete-move table classification (t54-e1c). It runs against a
//! freshly migrated PostgreSQL schema, so a new migration that seeds a row or
//! adds a trigger-maintained table fails here instead of making every fresh
//! destination refuse the move (as 0108 and 0111 did).
use super::{REGENERATED, SEEDED, TRIGGER_WRITTEN_COPIED};
use crate::{ServerKey, Store};
use std::collections::BTreeSet;

async fn fresh_schema() -> Option<(sqlx::PgPool, String, Store)> {
    let dsn = std::env::var("DATABASE_URL_PG")
        .ok()
        .or_else(|| std::env::var("MW_TEST_PG").ok())
        .filter(|dsn| !dsn.trim().is_empty());
    let Some(dsn) = dsn else {
        eprintln!(
            "[mw-store] complete-move classification: SKIPPED (set DATABASE_URL_PG or MW_TEST_PG)"
        );
        return None;
    };
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    let schema = format!("complete_move_{}", rand::random::<u64>());
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await
        .unwrap();
    let sep = if dsn.contains('?') { '&' } else { '?' };
    let scoped = format!("{dsn}{sep}options=-c%20search_path%3D{schema}");
    let store = Store::open_postgres(&scoped, ServerKey::generate())
        .await
        .unwrap();
    Some((admin, schema, store))
}

/// Table names following `INSERT INTO`, `UPDATE` or `DELETE FROM` in a
/// PL/pgSQL body (the caller keeps only names of real tables).
fn written_tables(body: &str) -> Vec<String> {
    let words: Vec<String> = body
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|word| !word.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    let mut out = Vec::new();
    for (i, word) in words.iter().enumerate() {
        let next = match word.as_str() {
            "insert" | "delete" if words.get(i + 1).is_some_and(|w| w == "into" || w == "from") => {
                words.get(i + 2)
            }
            "update" => words.get(i + 1),
            _ => None,
        };
        if let Some(next) = next {
            out.push(next.clone());
        }
    }
    out
}

#[test]
fn written_tables_reads_dml_targets() {
    assert_eq!(
        written_tables(
            "BEGIN INSERT INTO a(x) VALUES(1) ON CONFLICT DO UPDATE SET x=1; UPDATE b SET y=2; DELETE FROM c WHERE z; END"
        ),
        ["a", "set", "b", "c"]
    );
}

#[tokio::test]
async fn complete_move_classification() {
    let Some((admin, schema, store)) = fresh_schema().await else {
        return;
    };
    let crate::backend::Backend::Postgres(pool) = &store.backend else {
        unreachable!("opened via open_postgres")
    };
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT table_name::text FROM information_schema.tables
         WHERE table_schema=$1 AND table_type='BASE TABLE' AND table_name<>'_sqlx_migrations'",
    )
    .bind(&schema)
    .fetch_all(pool)
    .await
    .unwrap();
    let tables: BTreeSet<String> = tables.into_iter().collect();
    let regenerated: BTreeSet<&str> = REGENERATED.iter().copied().collect();
    let seeded: BTreeSet<&str> = SEEDED.iter().map(|(name, _)| *name).collect();
    let mut problems = Vec::new();

    for name in REGENERATED
        .iter()
        .chain(TRIGGER_WRITTEN_COPIED)
        .chain(&seeded)
    {
        if !tables.contains(*name) {
            problems.push(format!("{name}: listed but not in the schema"));
        }
    }

    // 1. A fresh database has rows only in seeded or regenerated tables, every
    //    seeded row matches its predicate, and every seeded entry still seeds.
    for name in &tables {
        let count: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {schema}.\"{name}\""))
            .fetch_one(pool)
            .await
            .unwrap();
        if let Some((_, seed)) = SEEDED.iter().find(|(seeded, _)| seeded == name) {
            let stray: i64 = sqlx::query_scalar(&format!(
                "SELECT count(*) FROM {schema}.\"{name}\" WHERE NOT COALESCE(({seed}),false)"
            ))
            .fetch_one(pool)
            .await
            .unwrap();
            if stray > 0 {
                problems.push(format!(
                    "{name}: {stray} fresh row(s) outside the seed predicate"
                ));
            }
            if count == 0 {
                problems.push(format!(
                    "{name}: listed in SEEDED but a fresh database has no row"
                ));
            }
        } else if count > 0 && !regenerated.contains(name.as_str()) {
            problems.push(format!(
                "{name}: a fresh database has {count} row(s); list it in SEEDED (with a predicate) \
                 or REGENERATED in migrate_complete.rs, or the complete move refuses every fresh destination"
            ));
        }
    }

    // 2. Every table a trigger function writes is classified.
    let bodies: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT p.prosrc FROM pg_trigger t
         JOIN pg_class c ON c.oid=t.tgrelid JOIN pg_namespace n ON n.oid=c.relnamespace
         JOIN pg_proc p ON p.oid=t.tgfoid
         WHERE NOT t.tgisinternal AND n.nspname=$1",
    )
    .bind(&schema)
    .fetch_all(pool)
    .await
    .unwrap();
    let mut trigger_targets = BTreeSet::new();
    for body in &bodies {
        for target in written_tables(body) {
            if tables.contains(&target) {
                trigger_targets.insert(target);
            }
        }
    }
    for target in &trigger_targets {
        if !regenerated.contains(target.as_str())
            && !TRIGGER_WRITTEN_COPIED.contains(&target.as_str())
        {
            problems.push(format!(
                "{target}: written by a trigger but neither REGENERATED nor TRIGGER_WRITTEN_COPIED \
                 in migrate_complete.rs; a verbatim copy would conflict with the trigger's rows"
            ));
        }
    }
    eprintln!(
        "[mw-store] complete-move classification: {} tables, trigger-written {:?}",
        tables.len(),
        trigger_targets
    );

    store.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&admin)
        .await
        .unwrap();
    assert!(
        problems.is_empty(),
        "complete-move classification:\n  - {}",
        problems.join("\n  - ")
    );
}
