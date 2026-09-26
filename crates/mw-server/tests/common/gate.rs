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
//!
//! That loudness still lands in a log nobody opens when the job is green, so a job
//! that *boots* the service it tests can opt out of skipping with
//! [`REQUIRE_LIVE_VAR`]. Its value names the gate variables that job has satisfied
//! (`MW_REQUIRE_LIVE=MW_E14_PG_DSN,DATABASE_URL_PG`), or the single token `all`.
//! A skip that cites one of those variables then fails the test instead of passing
//! it. Unset — every local run, and every job that does not boot the service —
//! behaviour is unchanged.
//!
//! The value is a list rather than a flag because the broad jobs run test binaries
//! whose other legs are gated on services they deliberately do not boot:
//! `store-dual-backend` runs the whole `mw-server` suite against Postgres, and its
//! Dovecot and OpenLDAP legs must still be free to skip. `all` is for a job that
//! runs one narrowly gated target, where any skip at all means the job did not do
//! its work.
//!
//! A list decides by matching the reason, so it cannot speak to a skip whose reason
//! names no variable — coverage held by convention in free text is coverage that
//! decays. Those skips get a second `UNMATCHED` line, so the gap is greppable in
//! exactly the job that asked to be strict (`grep -c '^UNMATCHED '`) instead of
//! silent. It is a marker and never a failure: the skips it fires for are build
//! preconditions rather than live-service gates.

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
