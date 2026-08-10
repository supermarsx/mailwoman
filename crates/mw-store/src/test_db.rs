//! Collision-free temporary paths for tests — SQLite databases and scratch
//! directories that are unique **by construction**, not by luck and not by
//! serialising the suite.
//!
//! # Why this module exists
//!
//! Before 26.19 every test that needed a store rolled its own `unique()`
//! helper, and the twenty-odd copies did not agree on what "unique" meant.
//! Three shapes were in the tree:
//!
//! * `pid + atomic counter` — sound.
//! * `pid + atomic counter + wall-clock nanos` — sound, but only because of
//!   the counter; the timestamp contributes nothing.
//! * `pid + wall-clock nanos` with **no counter**
//!   (`crates/mw-server/tests/t12_ews_auth.rs`), and `wall-clock nanos +
//!   counter` with **no pid** (`crates/mw-server/tests/sso_e2e.rs`) — unsound.
//!
//! The clock-only shape is the one that bites. Two `#[tokio::test]` cases
//! starting together inside one binary can read the same `SystemTime`, derive
//! the same SQLite path, and both run `sqlx::migrate!` against it; the loser
//! fails with `UNIQUE constraint failed: _sqlx_migrations.version`. How likely
//! that is depends entirely on the host's clock granularity — measured at
//! 100 ns on this Windows 11 dev host (`GetSystemTimePreciseAsFileTime`), but
//! 15.6 ms on hosts that fall back to `GetSystemTimeAsFileTime`, where two
//! tests starting in the same tick collide outright. A suite whose isolation
//! depends on the runner's timer hardware is not isolated, and
//! `--test-threads=1` was papering over that rather than fixing it.
//!
//! Every name produced here carries three independent components:
//!
//! | component | rules out |
//! |---|---|
//! | process id | two test binaries running at the same moment |
//! | per-process random salt | a *recycled* pid meeting leftovers from an earlier run |
//! | atomic counter | two tests inside one binary, at any clock resolution |
//!
//! No wall clock is consulted, so no result depends on timer resolution.
//!
//! On top of that, [`unique_dir`] creates its directory with
//! [`std::fs::create_dir`] rather than `create_dir_all`, so a collision that
//! somehow survived all three components surfaces as a loud panic naming the
//! path instead of two tests silently sharing a database. That check runs in
//! every test in every binary, continuously, which is a stronger guarantee
//! than any one-off assertion.
//!
//! # How to reach it
//!
//! This module is deliberately **not** declared in
//! `crates/mw-store/src/lib.rs`: it is test-support code and has no business in
//! the shipped library. It depends on nothing but `std` and names no item of
//! its host crate, so it can be pulled into any test target with `#[path]`:
//!
//! ```ignore
//! // from crates/mw-store/tests/*.rs
//! #[path = "../src/test_db.rs"]
//! mod test_db;
//!
//! // from crates/mw-server/tests/*.rs — the `#[path]` is already written for
//! // you in crates/mw-server/tests/common/mod.rs, so those files just say:
//! mod common;
//! use common::test_db;
//! ```
//!
//! Its guarantees are exercised by the `common` test target,
//! `crates/mw-server/tests/common/main.rs`.
//!
//! # Cleanup
//!
//! Directories are left behind on purpose, as they were before. They now all
//! live under one [`root`] (`<system temp>/mailwoman-tests`), so a failed run's
//! databases stay available for inspection and the whole tree can be dropped in
//! one step instead of globbing a dozen `mw-*` prefixes. Nothing here deletes
//! anything: a `Drop` guard would run during unwinding and remove exactly the
//! evidence a failing test just produced.

#![allow(dead_code)] // No single test target uses every helper.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// Directory holding every temporary path this module hands out.
pub fn root() -> PathBuf {
    std::env::temp_dir().join("mailwoman-tests")
}

/// A token that is unique across processes and across calls within a process.
///
/// Use it when a *name* rather than a path needs to be unique — account ids,
/// mailbox names, client ids. For filesystem paths prefer [`unique_dir`] /
/// [`unique_db_path`], which also create the directory.
pub fn unique_tag() -> String {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    format!(
        "{}-{:x}-{}",
        std::process::id(),
        process_salt(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    )
}

/// A random value fixed for the lifetime of this process.
///
/// Operating systems recycle process ids, so `pid + counter` alone can still
/// collide with the leftovers of an earlier run that happened to draw the same
/// pid — the directories under [`root`] outlive the process that made them.
/// `RandomState` is seeded by the OS once per process and lives in `std`, so
/// this costs no dependency.
fn process_salt() -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    use std::sync::OnceLock;
    static SALT: OnceLock<u64> = OnceLock::new();
    *SALT.get_or_init(|| {
        let mut h = RandomState::new().build_hasher();
        h.write_u8(0);
        h.finish()
    })
}

/// Create and return a fresh, empty directory named `<prefix>-<unique tag>`
/// under [`root`].
///
/// Panics if the directory already exists — that would mean two tests were
/// about to share it, which is precisely the failure this module removes, and
/// it deserves a clear panic rather than a confusing migration error later.
pub fn unique_dir(prefix: &str) -> PathBuf {
    let root = root();
    std::fs::create_dir_all(&root)
        .unwrap_or_else(|e| panic!("create test temp root {}: {e}", root.display()));

    let dir = root.join(format!("{prefix}-{}", unique_tag()));
    match std::fs::create_dir(&dir) {
        Ok(()) => dir,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => panic!(
            "test temp dir {} already exists — two tests would have shared it. This should be \
             unreachable; if you are seeing it, the uniqueness guarantee of \
             mw-store::test_db is broken.",
            dir.display()
        ),
        Err(e) => panic!("create test temp dir {}: {e}", dir.display()),
    }
}

/// Path to a fresh SQLite database file, inside a directory of its own.
///
/// The enclosing directory is what makes this safe: SQLite writes `-wal` and
/// `-shm` sidecars next to the database, and tests read those sidecars back
/// (`t18_e2e_vacuum`, `t17_note_seal`, `t16_twofa`). One directory per database
/// keeps a test's sidecars unambiguously its own.
///
/// The file itself is **not** created — `Store::open` expects to create it.
/// `path.parent()` is the fresh directory, so call sites that derive a `web/`
/// directory from the database's parent keep working.
pub fn unique_db_path(prefix: &str) -> PathBuf {
    unique_dir(prefix).join(format!("{prefix}.db"))
}

/// Path to a fresh file named `file_name`, inside a directory of its own.
///
/// For tests that need a particular file name or extension rather than the
/// `<prefix>.db` that [`unique_db_path`] produces.
pub fn unique_file_path(prefix: &str, file_name: &str) -> PathBuf {
    unique_dir(prefix).join(file_name)
}
