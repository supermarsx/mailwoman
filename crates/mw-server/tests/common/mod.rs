//! Shared support code for `mw-server`'s integration tests.
//!
//! Cargo does not build `tests/common/mod.rs` as a test target of its own, so
//! this file is only ever compiled into the binaries that say `mod common;`.
//!
//! It carries two things. The first is [`gate`]: `gate::pg_dsn`, the Postgres DSN
//! lookup the PG legs use, and `gate::skip`, which every env-gated leg calls when it
//! does not run so the gate log shows it. See `gate.rs` for why a skip has to bypass libtest's
//! output capture.
//!
//! The second is a bridge to one helper. `crates/mw-store/src/test_db.rs`
//! hands out temporary SQLite paths that are unique by construction; it lives
//! in `mw-store/src` so that `mw-store`'s own tests can use it too, but it is
//! deliberately not declared in `mw-store`'s `lib.rs` — test-support code has no
//! business in the shipped library. `#[path]` is what bridges the two: the
//! module depends on nothing but `std` and names no item of its host crate, so
//! it compiles cleanly wherever it is included.
//!
//! Usage from any `crates/mw-server/tests/*.rs`:
//!
//! ```ignore
//! mod common;
//! use common::test_db;
//!
//! let db = test_db::unique_db_path("mw-t19-example");
//!
//! let Some(dsn) = common::gate::pg_dsn() else {
//!     common::gate::skip("MW_E14_PG_DSN and DATABASE_URL_PG unset — Postgres leg not driven.");
//!     return;
//! };
//! ```
//!
//! The guarantees `test_db` and `gate` make are exercised once, by the sibling
//! `common` test target in `main.rs`.

#![allow(dead_code)] // No single test binary uses every helper.

pub mod gate;

#[path = "../../../mw-store/src/test_db.rs"]
pub mod test_db;
