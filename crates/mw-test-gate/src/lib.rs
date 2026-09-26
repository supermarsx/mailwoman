//! The skip/gate convention, in a crate a **library `src/` unit test** can reach.
//!
//! # Why this crate exists at all
//!
//! The convention itself is documented at length in
//! `crates/mw-server/tests/common/gate.rs`, which is where it was built (t23-e9-10,
//! t24-e18) and which remains the canonical statement of it. In one paragraph: a leg
//! that returns early because its live service is not configured reports `ok` to
//! libtest, exactly like a leg that ran, so [`skip`] writes a `SKIPPED` record to the
//! process's stderr **handle** (libtest's capture intercepts only the print macros,
//! so the line survives for a passing test), appends it to `$MW_TEST_SKIP_LOG` when
//! that is set, and — when [`REQUIRE_LIVE_VAR`] says the running job exists to run
//! this leg — panics instead of passing.
//!
//! What this crate adds is **reach**. `common/gate.rs` is a file inside
//! `mw-server/tests/`, consumed by `#[path = …]` includes. A test in another crate's
//! `tests/` directory can include it (`mw-store/tests/backend_parity.rs` does), but a
//! `#[cfg(test)]` module inside a *library's* `src/` cannot: the include would pull a
//! file from outside the crate into that crate's own build, and `cargo package` for a
//! crate that ships would then be packaging something it does not contain.
//!
//! So six Postgres legs — three in `mw-engine/src/state.rs`, three in
//! `mw-store/src/v2.rs` — were left outside `MW_REQUIRE_LIVE` when it landed, and
//! under CI's `store-dual-backend` job they could still skip invisibly while the job
//! stayed green. Four of them were worse than invisible to the switch: they reported
//! through `eprintln!`, which libtest captures for a passing test, so they said
//! nothing on either stream.
//!
//! This crate is taken as a **dev-dependency**, which a `src/` `#[cfg(test)]` module
//! may use freely and which Cargo strips from the published manifest (a `path`
//! dev-dependency with no `version`), so those legs get the gate without either
//! crate's package changing.
//!
//! # Relationship to `common/gate.rs`
//!
//! This is a **copy**, not a re-export, and that is a first step rather than a
//! preference: making the two share one implementation means editing
//! `common/gate.rs`, which another lane was editing when this landed. The end state
//! is for that file to become a thin re-export of this crate and for the equivalence
//! test below to be deleted as redundant.
//!
//! Until then the copy is not held together by a comment.
//! `tests/agrees_with_common_gate.rs` path-includes the canonical file and asserts
//! that every function that *decides* something — [`Require::parse`],
//! [`require_violation`], [`skip_line`], [`unmatched_line`], [`is_unmatched`],
//! [`names_a_gate_variable`], [`pg_dsn_from`], [`require_failure`] — returns the same
//! answer for the same input, across a matrix that reaches every branch of each. If
//! the canonical file moves and this copy does not, that test goes red and names the
//! input the two disagree on.
//!
//! # Using it
//!
//! `text` rather than `ignore` on both blocks below, deliberately: an `ignore` block
//! is still a doctest target, and the workspace's doctest inventory is a documented
//! figure (`docs/testing/coverage.md`) that a test-support crate has no business
//! moving. Neither block is runnable anyway — one is a manifest fragment.
//!
//! ```text
//! # crates/<crate>/Cargo.toml
//! [dev-dependencies]
//! mw-test-gate = { workspace = true }
//! ```
//!
//! ```text
//! let Some(dsn) = pg_dsn() else {
//!     mw_test_gate::skip("[mw-store] some leg: DATABASE_URL_PG and MW_TEST_PG unset — …");
//!     return;
//! };
//! ```
//!
//! The reason must **name the variables that gate the leg**. That is not decoration:
//! `MW_REQUIRE_LIVE`'s list form decides whether a skip is a failure by looking for
//! those names in the reason, so a reason that omits them describes a leg no job can
//! assert — and [`is_unmatched`] will say so on stderr rather than let it pass for
//! covered.

use std::fs::OpenOptions;
use std::io::Write;

/// Names a file that [`skip`] appends each `SKIPPED` line to, in addition to stderr.
pub const SKIP_LOG_VAR: &str = "MW_TEST_SKIP_LOG";

/// Names the gate variables the running job has satisfied; see the module docs.
pub const REQUIRE_LIVE_VAR: &str = "MW_REQUIRE_LIVE";

/// What [`REQUIRE_LIVE_VAR`] asks of a skip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Require {
    /// Unset or empty: a skip is a pass, as it has always been.
    Off,
    /// `all`: every skip in this run is a failure.
    All,
    /// A list of gate variables the job exported; a skip citing one is a failure.
    Vars(Vec<String>),
}

impl Require {
    /// Read [`REQUIRE_LIVE_VAR`] from the process environment.
    pub fn from_env() -> Self {
        Self::parse(std::env::var(REQUIRE_LIVE_VAR).ok().as_deref())
    }

    /// [`Require::from_env`] over an explicit value, so it can be tested without
    /// mutating the process environment. Entries split on commas and whitespace.
    pub fn parse(value: Option<&str>) -> Self {
        let vars: Vec<String> = value
            .unwrap_or("")
            .split([',', ' ', '\t', '\n'])
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        if vars.is_empty() {
            Require::Off
        } else if vars.iter().any(|v| v.eq_ignore_ascii_case("all")) {
            Require::All
        } else {
            Require::Vars(vars)
        }
    }
}

/// Why this skip is not allowed to pass, or `None` when it is.
///
/// `is_set` answers whether a named variable is present and non-empty in the
/// environment. A required variable that is *not* set is itself the failure: the
/// job claims to have exported it, so either the step dropped it or the name is
/// misspelled — and a misspelled name would otherwise match no reason and quietly
/// require nothing, which is the failure mode this switch exists to remove.
pub fn require_violation(
    req: &Require,
    is_set: impl Fn(&str) -> bool,
    reason: &str,
) -> Option<String> {
    match req {
        Require::Off => None,
        Require::All => Some(format!(
            "{REQUIRE_LIVE_VAR}=all: this job boots the service this test needs, so a skip \
             is a failure. Either the service did not come up, or the variable naming it \
             never reached this step."
        )),
        Require::Vars(vars) => {
            let missing: Vec<&str> = vars
                .iter()
                .map(String::as_str)
                .filter(|v| !is_set(v))
                .collect();
            if !missing.is_empty() {
                return Some(format!(
                    "{REQUIRE_LIVE_VAR} names {}, which this step did not export. A name that \
                     is set nowhere matches no skip and would require nothing at all — check \
                     the spelling against the job's `env:` block.",
                    missing.join(", ")
                ));
            }
            let cited: Vec<&str> = vars
                .iter()
                .map(String::as_str)
                .filter(|v| reason.contains(*v))
                .collect();
            (!cited.is_empty()).then(|| {
                format!(
                    "{REQUIRE_LIVE_VAR} names {}, and this skip cites {}: the job boots that \
                     service, so the leg was required to run. Either the service did not come \
                     up, or the value reaching this step is wrong.",
                    vars.join(", "),
                    cited.join(", ")
                )
            })
        }
    }
}

/// The live Postgres DSN for a PG leg, or `None` when no database is configured.
///
/// `MW_E14_PG_DSN` first, then `DATABASE_URL_PG` — the variable CI's
/// `store-dual-backend` job sets. An empty value counts as unset.
///
/// This is the order the `mw-server` integration legs and `mw-engine`'s
/// `state.rs` resolve in. It is **not** universal: `mw-store/src/v2.rs` reads
/// `DATABASE_URL_PG` then `MW_TEST_PG`, and keeps its own resolver, because
/// switching it to this one would silently change which variables configure it.
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

/// Report that the calling leg is not running, and why. The caller still returns —
/// unless [`REQUIRE_LIVE_VAR`] says this job exists to run it, in which case the
/// call panics and the leg fails.
///
/// The line names the test (libtest names each test's thread after it) and the
/// call site, so it can be traced back without `--nocapture`. The `SKIPPED` line is
/// written before any panic, so the skip log records what the run did not cover
/// whichever way the switch is set.
///
/// Under a [`Require::Vars`] requirement a second `UNMATCHED` line follows for a
/// skip the list could not have spoken to. It is a marker and never a failure; see
/// [`is_unmatched`].
#[track_caller]
pub fn skip(reason: impl std::fmt::Display) {
    let at = std::panic::Location::caller();
    let reason = reason.to_string();
    let test = std::thread::current().name().map(str::to_string);
    let req = Require::from_env();
    let line = skip_line(test.as_deref(), at.file(), at.line(), &reason);
    let _ = writeln!(std::io::stderr(), "\n{line}");
    record_skip(&line);

    if let Some(why) = require_violation(
        &req,
        |name| std::env::var_os(name).is_some_and(|v| !v.is_empty()),
        &reason,
    ) {
        panic!("{}", require_failure(&line, &why));
    }

    // Only once the requirement has cleared this skip: a misspelled variable fails
    // above, and pairing that failure with a marker would read as two problems.
    if is_unmatched(&req, &reason) {
        let marker = unmatched_line(test.as_deref(), at.file(), at.line());
        let _ = writeln!(std::io::stderr(), "{marker}");
        record_skip(&marker);
    }
}

/// The panic body for a skip that [`REQUIRE_LIVE_VAR`] does not allow.
///
/// It leads with the same `SKIPPED` record the log would have carried — test, call
/// site and reason — because the reader of a red job needs exactly what the reader
/// of the log needed, and then the reason that record is a failure here.
pub fn require_failure(record: &str, why: &str) -> String {
    format!("{record}\n  {why}")
}

/// Append one `SKIPPED` or `UNMATCHED` line to the file named by [`SKIP_LOG_VAR`],
/// if it is set. Each tag leads its line, so a log stays countable per tag.
fn record_skip(line: &str) {
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
    tagged_line("SKIPPED", test, file, line, reason)
}

/// One `UNMATCHED` line, marking a skip that a [`Require::Vars`] requirement could
/// not have spoken to either way. See [`is_unmatched`].
pub fn unmatched_line(test: Option<&str>, file: &str, line: u32) -> String {
    tagged_line(
        "UNMATCHED",
        test,
        file,
        line,
        &format!("reason names no gate variable; {REQUIRE_LIVE_VAR} cannot assert it"),
    )
}

/// `<TAG> <test> (<file>:<line>): <reason>`, whitespace in the reason collapsed so
/// the whole record stays on one line and a log can be counted with `grep -c`.
fn tagged_line(tag: &str, test: Option<&str>, file: &str, line: u32, reason: &str) -> String {
    let reason = reason.split_whitespace().collect::<Vec<_>>().join(" ");
    let file = file.rsplit(['/', '\\']).next().unwrap_or(file);
    match test.filter(|t| *t != "main") {
        Some(test) => format!("{tag} {test} ({file}:{line}): {reason}"),
        None => format!("{tag} ({file}:{line}): {reason}"),
    }
}

/// Whether a list-mode requirement is simply unable to speak to this skip.
///
/// [`Require::Vars`] decides by matching the reason against the named variables, so
/// a reason naming no gate variable at all is outside its reach in both directions:
/// it can never be asserted, and it can never be cleared. That is a gap worth
/// seeing, not a failure — the two skips it fires for in `store-dual-backend`
/// (`t17_tt_shell`, `integration`) are build preconditions rather than live-service
/// gates, and failing them is the false-failure class this switch was corrected
/// twice to avoid.
///
/// Never true under [`Require::All`], where a reason naming no variable is precisely
/// what is being caught, nor under [`Require::Off`].
pub fn is_unmatched(req: &Require, reason: &str) -> bool {
    matches!(req, Require::Vars(_)) && !names_a_gate_variable(reason)
}

/// Whether a reason names something shaped like a gate variable — `MW_…` or
/// `DATABASE_URL…`. Deliberately a shape test and not a list of known variables: a
/// guard added after this file was written must count too.
pub fn names_a_gate_variable(reason: &str) -> bool {
    reason
        .split(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
        .any(|token| {
            (token.starts_with("MW_") && token.len() > "MW_".len())
                || token.starts_with("DATABASE_URL")
        })
}
