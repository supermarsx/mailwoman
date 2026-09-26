//! Env-gated legs: where a Postgres leg finds its DSN, and how a leg that did not run
//! says so.
//!
//! **The implementation and the full statement of the convention now live in
//! `crates/mw-test-gate`.** This file is the seam that keeps `gate::…` working for the
//! ~70 test binaries that path-include it; it deliberately contains no logic.
//!
//! Why the move (t25-e1): the same convention is needed by `#[cfg(test)]` modules
//! inside `mw-engine/src/state.rs` and `mw-store/src/v2.rs`, and a library's `src/`
//! cannot `#[path]`-include a file from `mw-server/tests/` — that pulls a file from
//! outside the crate into its build and breaks `cargo package` for a crate that ships.
//! Six Postgres legs sat outside `MW_REQUIRE_LIVE` for that reason. A crate taken as a
//! dev-dependency is reachable from both places, and Cargo strips a path
//! dev-dependency with no `version` from the published manifest.
//!
//! It was briefly a copy, held in step by a parity test. That test compared the two
//! copies' answers, which could only ever see the **intersection** of their public
//! surfaces: when t24-e18 added `is_unmatched`, `names_a_gate_variable` and
//! `unmatched_line` to this file alone, the parity test passed. A re-export makes
//! divergence impossible rather than detectable, which is why the copy is gone.
//!
//! The glob is exactly equivalent to the file it replaces: a `#[path]`-included child
//! module only ever exposed its `pub` items to its parent, so there is nothing a caller
//! could previously reach that this does not re-export.

// Same reason `common/mod.rs` carries `#![allow(dead_code)]`: no single test binary
// uses every helper, and most of the ~70 that include `common` never gate on a live
// service at all. As a file of functions the unused ones read as dead code; as a
// re-export they read as an unused import. Same tolerance, different lint name.
#[allow(unused_imports)]
pub use mw_test_gate::*;
