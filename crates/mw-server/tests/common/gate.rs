//! Env-gated legs: where a Postgres leg finds its DSN, and how a leg that did not
//! run says so.
//!
//! A leg that returns early because its live service is not configured reports
//! `ok` to libtest, the same as a leg that ran. The explanation those legs used to
//! print went through `eprintln!`, which libtest captures for a passing test, so it
//! was visible only under `--nocapture` and a gate log could not be read for what
//! did not run (t23-e9-10).
//!
//! [`skip`] writes to the process's stderr handle directly. libtest's capture only
//! intercepts the print macros, so the line reaches the gate log for a passing test
//! too. Grep a log for `^SKIPPED ` to list what a run did not cover. Set
//! `MW_TEST_SKIP_LOG` to a file path to also collect the same lines there, one per
//! skip, appended across test binaries.
//!
//! Skipping is still a pass. A missing live rig is not a failure of the code under
//! test; the defect was that a skip looked the same as a leg that ran.

use std::fs::OpenOptions;
use std::io::Write;

/// Names a file that [`skip`] appends each `SKIPPED` line to, in addition to stderr.
pub const SKIP_LOG_VAR: &str = "MW_TEST_SKIP_LOG";

/// The live Postgres DSN for a PG leg, or `None` when no database is configured.
///
/// `MW_E14_PG_DSN` first, then `DATABASE_URL_PG` — the variable CI's
/// `store-dual-backend` job sets. That is the order `t10_dcr`, `t11_dcr_admin`,
/// `t12_ews_auth` and `t19_e2e` already resolve in. An empty value counts as unset.
pub fn pg_dsn() -> Option<String> {
    pg_dsn_from(|name| std::env::var(name).ok())
}

/// [`pg_dsn`] over an arbitrary variable lookup, so the order can be tested
/// without mutating the process environment.
pub fn pg_dsn_from(get: impl Fn(&str) -> Option<String>) -> Option<String> {
    ["MW_E14_PG_DSN", "DATABASE_URL_PG"]
        .into_iter()
        .filter_map(get)
        .find(|v| !v.trim().is_empty())
}

/// Report that the calling leg is not running, and why. The caller still returns.
///
/// The line names the test (libtest names each test's thread after it) and the
/// call site, so it can be traced back without `--nocapture`.
#[track_caller]
pub fn skip(reason: impl std::fmt::Display) {
    let at = std::panic::Location::caller();
    let line = skip_line(
        std::thread::current().name(),
        at.file(),
        at.line(),
        &reason.to_string(),
    );
    let _ = writeln!(std::io::stderr(), "\n{line}");
    if let Some(path) = std::env::var_os(SKIP_LOG_VAR).filter(|p| !p.is_empty()) {
        // The variable was set on purpose; losing its lines silently would defeat it.
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap_or_else(|e| panic!("{SKIP_LOG_VAR}={path:?} cannot be opened: {e}"));
        writeln!(file, "{line}")
            .unwrap_or_else(|e| panic!("{SKIP_LOG_VAR}={path:?} cannot be written: {e}"));
    }
}

/// One `SKIPPED` line: `SKIPPED <test> (<file>:<line>): <reason>`, whitespace in
/// the reason collapsed so the whole record stays on one line.
pub fn skip_line(test: Option<&str>, file: &str, line: u32, reason: &str) -> String {
    let reason = reason.split_whitespace().collect::<Vec<_>>().join(" ");
    let file = file.rsplit(['/', '\\']).next().unwrap_or(file);
    match test.filter(|t| *t != "main") {
        Some(test) => format!("SKIPPED {test} ({file}:{line}): {reason}"),
        None => format!("SKIPPED ({file}:{line}): {reason}"),
    }
}
