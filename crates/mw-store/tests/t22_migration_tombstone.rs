//! t22-e12 — **migration `0024` is retired. It must never be filled.**
//!
//! There is no `0024` and there never was: `0025_message_paging_nulls_last.sql`
//! already exists in both dialects, so `0024` is a **hole inside an applied
//! sequence**, not the end of one. A lane computing "highest + 1" will not find it,
//! but a lane that notices the gap will helpfully close it — which is why this test
//! exists and why it says so in its failure message.
//!
//! **sqlx will not stop them.** `Migrator::run_direct` (sqlx 0.8.6) applies any
//! resolved-but-unapplied migration **regardless of version order**, and
//! `validate_applied_migrations` only raises `VersionMissing` for a migration that
//! was *applied* and is now absent from disk. So a file placed at `0024` runs
//! **after** `0025` on every existing database and **before** it on every fresh one:
//! two permanent, divergent orderings, and **no error on either path**. That is the
//! kind of difference that surfaces a year later as "works on my machine".
//!
//! The second assertion — that both dialects carry the identical set of version
//! numbers — catches the other half of the same class of mistake: a migration added
//! to one dialect and forgotten in the other, which passes every SQLite test and
//! fails only against real Postgres.
//!
//! **Deliberately NOT asserted: contiguity.** A general "no gaps" rule would fail
//! today, for exactly the reason this file exists, and a test that has to be weakened
//! on the day it lands teaches everyone that weakening tests is normal.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Version numbers present in a migration directory, parsed from the `NNNN_` prefix.
fn versions(dir: &Path) -> BTreeSet<u32> {
    std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{} is readable: {e}", dir.display()))
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let (prefix, _) = name.split_once('_')?;
            prefix.parse::<u32>().ok()
        })
        .collect()
}

fn migrations_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The retired version. See the module docs.
const TOMBSTONE: u32 = 24;

#[test]
fn version_0024_is_retired_in_both_dialects() {
    let root = migrations_root();
    for dialect in ["migrations", "migrations_pg"] {
        let found = versions(&root.join(dialect));

        // Asserted first: if the parse silently matched nothing, "0024 is absent"
        // would be trivially true and this test would guard nothing at all.
        assert!(
            found.len() > 20,
            "only {} migrations parsed from {dialect} — the version parse has stopped \
             matching and the tombstone below is vacuous",
            found.len()
        );

        assert!(
            !found.contains(&TOMBSTONE),
            "{dialect} now contains a migration 0024. It is RETIRED and must stay \
             empty: 0025 already exists, so 0024 is a hole INSIDE an applied \
             sequence. sqlx applies out-of-order migrations without complaint, so a \
             file here runs AFTER 0025 on existing databases and BEFORE it on fresh \
             ones — two permanent orderings, no error either way. Use the next number \
             above the highest, not the first gap."
        );
    }
}

#[test]
fn both_dialects_define_the_same_versions() {
    let root = migrations_root();
    let sqlite = versions(&root.join("migrations"));
    let postgres = versions(&root.join("migrations_pg"));

    let sqlite_only: Vec<_> = sqlite.difference(&postgres).collect();
    let postgres_only: Vec<_> = postgres.difference(&sqlite).collect();

    assert!(
        sqlite_only.is_empty() && postgres_only.is_empty(),
        "the two dialects have diverged — a migration present in one and missing from \
         the other passes every SQLite test and fails only against real Postgres.\n  \
         SQLite only: {sqlite_only:?}\n  Postgres only: {postgres_only:?}"
    );
}
