//! This crate is a copy of `crates/mw-server/tests/common/gate.rs`. This test is what
//! makes that safe.
//!
//! The copy exists because a `#[cfg(test)]` module inside a shipped library's `src/`
//! cannot `#[path]`-include a file from `mw-server/tests/` without breaking
//! `cargo package` for that library — see the crate docs. The cost of a copy is drift:
//! `MW_REQUIRE_LIVE` would then mean one thing in `mw-server`'s integration suite and
//! another in `mw-engine`/`mw-store`'s unit tests, and nothing would say so.
//!
//! So the canonical file is included **here**, in this crate's `tests/` directory
//! (where an out-of-crate include costs nothing, exactly as
//! `mw-store/tests/backend_parity.rs` already relies on), and every public function
//! that decides something is compared answer-for-answer over a matrix that reaches
//! every branch of each.
//!
//! **If this test is red:** the canonical file changed and `src/lib.rs` did not. Port
//! the change; the failure names the input the two now disagree on. Do not weaken the
//! matrix — a green test here is the only thing asserting that the two copies mean the
//! same thing.
//!
//! What is deliberately *not* compared: `skip` itself, `record_skip`, and
//! `Require::from_env`. All three read or write process-global state (the environment,
//! stderr, a log file), so calling both copies in one process would have each observe
//! the other's effects. Their bodies are plumbing; every decision they make is made
//! by one of the functions compared below.
//!
//! # The door this test used to leave open
//!
//! Comparing named functions can only see the **intersection** of the two copies. A
//! function added to canonical alone is invisible to it: there is nothing to call on
//! this side, so every comparison still passes while the two copies have stopped
//! meaning the same thing. That is not hypothetical — t24-e18 added `is_unmatched`,
//! `names_a_gate_variable` and `unmatched_line` to canonical and this file passed
//! against the change. They are ported now, but the door stayed open for the next one.
//!
//! [`every_public_item_of_canonical_is_accounted_for`] closes it by asserting the
//! **set**, not the intersection: it reads the canonical file as text, extracts every
//! public item, and fails unless each is listed in [`COMPARED`] or [`NOT_COMPARED`]
//! *and* actually named in this file's source. A new public item in canonical is then
//! red by default and has to be deliberately classified.
//!
//! Its own discriminating power is tested rather than assumed
//! ([`the_accounting_check_detects_a_canonical_only_addition`]), against the real
//! canonical text with one synthetic function appended. That is the "add a scratch
//! function and watch it go red" proof, done as an assertion instead of a one-off edit
//! to a file three other lanes are committing to — so it holds for every future
//! change, not just the one that motivated it.
//!
//! Worth knowing about the division of labour, because it is not what it looks like:
//! dropping a [`COMPARED`] item from the copy is caught by **rustc**, not by this
//! check — the matrix names it, so the crate stops compiling, which is stronger than a
//! failing assertion. What the accounting check uniquely reaches is the two cases that
//! compile cleanly: a public item added to canonical alone, and a [`NOT_COMPARED`]
//! item dropped from the copy, which nothing here calls.

// The canonical statement of the convention. Unused items are expected: this test
// calls the deciding functions and ignores the stateful plumbing around them.
#[allow(dead_code)]
#[path = "../../mw-server/tests/common/gate.rs"]
mod canonical;

/// Every input shape `MW_REQUIRE_LIVE` can arrive in, including the ones that decide
/// a branch: absent, present-but-empty, whitespace-only, `all` in either case, `all`
/// mixed into a list, and both separators.
const REQUIRE_VALUES: &[Option<&str>] = &[
    None,
    Some(""),
    Some("   "),
    Some(","),
    Some("all"),
    Some("ALL"),
    Some("All"),
    Some("MW_E14_PG_DSN"),
    Some("MW_E14_PG_DSN,DATABASE_URL_PG"),
    Some("MW_E14_PG_DSN DATABASE_URL_PG"),
    Some("MW_E14_PG_DSN,\tDATABASE_URL_PG\nMW_TEST_PG"),
    Some("MW_E14_PG_DSN,all"),
    Some("MW_TYPOED_NAME"),
    Some("MW_TYPOED_NAME,MW_E14_PG_DSN"),
    Some("allowance"),
];

/// Reasons spanning the three cases the list form distinguishes: cites one listed
/// variable, cites two, cites none (the reachability/platform class).
const REASONS: &[&str] = &[
    "[t15 upload] MW_E14_PG_DSN and DATABASE_URL_PG unset — live Postgres not driven.",
    "[mw-store] migrate-store: DATABASE_URL_PG and MW_TEST_PG unset — not driven.",
    "[e16] OpenLDAP unreachable at ldap://127.0.0.1:1389 — GAL legs not driven.",
    "",
    "MW_E14_PG_DSN",
];

/// The variables "this step exported", so the `missing` branch is reached for
/// `MW_TYPOED_NAME` and not for the others.
fn is_set(name: &str) -> bool {
    matches!(name, "MW_E14_PG_DSN" | "DATABASE_URL_PG" | "MW_TEST_PG")
}

#[test]
fn require_parse_agrees() {
    for value in REQUIRE_VALUES {
        let mine = mw_test_gate::Require::parse(*value);
        let theirs = canonical::Require::parse(*value);
        // The two enums are distinct types, so compare their debug spellings —
        // which carry both the variant and the parsed variable list.
        assert_eq!(
            format!("{mine:?}"),
            format!("{theirs:?}"),
            "MW_REQUIRE_LIVE={value:?} parses differently in mw-test-gate than in \
             mw-server/tests/common/gate.rs"
        );
    }
}

#[test]
fn require_violation_agrees() {
    let mut branches = (false, false, false, false);
    for value in REQUIRE_VALUES {
        for reason in REASONS {
            let mine = mw_test_gate::require_violation(
                &mw_test_gate::Require::parse(*value),
                is_set,
                reason,
            );
            let theirs =
                canonical::require_violation(&canonical::Require::parse(*value), is_set, reason);
            assert_eq!(
                mine, theirs,
                "MW_REQUIRE_LIVE={value:?} over reason {reason:?} is judged differently in \
                 mw-test-gate than in mw-server/tests/common/gate.rs"
            );

            match (mw_test_gate::Require::parse(*value), &mine) {
                (mw_test_gate::Require::Off, _) => branches.0 = true,
                (mw_test_gate::Require::All, _) => branches.1 = true,
                (mw_test_gate::Require::Vars(_), Some(why)) if why.contains("did not export") => {
                    branches.2 = true
                }
                (mw_test_gate::Require::Vars(_), _) => branches.3 = true,
            }
        }
    }
    // A matrix that agreed on nothing but `Off` would pass while proving nothing.
    assert_eq!(
        branches,
        (true, true, true, true),
        "the matrix no longer reaches all four branches of require_violation: \
         (Off, All, Vars/missing, Vars/cited-or-not)"
    );
}

#[test]
fn skip_line_agrees() {
    let cases: &[(Option<&str>, &str, u32)] = &[
        (None, "state.rs", 1),
        (Some("main"), "state.rs", 1448),
        (
            Some("live_pg_session_state_is_one_statement_per_group"),
            "state.rs",
            1448,
        ),
        (Some("t"), r"crates\mw-store\src\v2.rs", 1014),
        (Some("t"), "crates/mw-store/src/v2.rs", 1014),
        (Some("t"), "v2.rs", u32::MAX),
        (Some("t"), "", 0),
    ];
    let reasons = [
        "one line",
        "  leading and trailing  ",
        "wrapped\n across\t lines   with   runs",
        "",
    ];
    for (test, file, line) in cases {
        for reason in reasons {
            assert_eq!(
                mw_test_gate::skip_line(*test, file, *line, reason),
                canonical::skip_line(*test, file, *line, reason),
                "skip_line({test:?}, {file:?}, {line}, {reason:?}) differs between \
                 mw-test-gate and mw-server/tests/common/gate.rs"
            );
        }
    }
}

#[test]
fn pg_dsn_order_agrees() {
    let table: &[&[(&str, &str)]] = &[
        &[],
        &[("MW_E14_PG_DSN", "a")],
        &[("DATABASE_URL_PG", "b")],
        &[("MW_E14_PG_DSN", "a"), ("DATABASE_URL_PG", "b")],
        &[("MW_E14_PG_DSN", "   "), ("DATABASE_URL_PG", "b")],
        &[("MW_E14_PG_DSN", ""), ("DATABASE_URL_PG", "")],
        &[("MW_TEST_PG", "c")],
    ];
    for env in table {
        let get = |name: &str| {
            env.iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| (*v).to_string())
        };
        assert_eq!(
            // `get` borrows `env`, so it is `Copy` and can be handed to both.
            mw_test_gate::pg_dsn_from(get),
            canonical::pg_dsn_from(get),
            "pg_dsn resolves {env:?} differently in mw-test-gate than in \
             mw-server/tests/common/gate.rs"
        );
    }
}

/// Reasons that separate "names a gate variable" from "does not", including the ones
/// `store-dual-backend` actually emits with no variable in them (build preconditions,
/// which must be marked and never failed) and the shapes a naive check gets wrong:
/// a lowercase `mw_`, a bare `MW_`, `DATABASE_URL` with no suffix, and a variable
/// glued to punctuation.
const UNMATCHED_REASONS: &[&str] = &[
    "[t12 IMAP] MW_IMAP_LIVE!=1 — real Dovecot SCRAM not driven.",
    "[t15 upload] MW_E14_PG_DSN and DATABASE_URL_PG unset — not driven.",
    "[mw-store] set DATABASE_URL_PG or MW_TEST_PG to a live postgres:16.",
    "[t17 TT] not built (no apps/web/dist/index.html) — build the web SPA.",
    "sanitize content assertions: a kernel jail is expected and no mw-render worker is built",
    "[e16] OpenLDAP unreachable at ldap://127.0.0.1:1389",
    "[t16 sandbox] non-Linux (windows) — the kernel jail is a Linux-only path.",
    "server does not advertise SORT",
    "mw_imap_live is lowercase and names nothing",
    "MW_ on its own is not a variable",
    "DATABASE_URL",
    "DATABASE_URL_PG,",
    "(MW_SSO_LIVE)",
    "",
];

#[test]
fn is_unmatched_and_its_marker_agree() {
    let mut saw = (false, false);
    for value in REQUIRE_VALUES {
        for reason in UNMATCHED_REASONS {
            let mine = mw_test_gate::is_unmatched(&mw_test_gate::Require::parse(*value), reason);
            let theirs = canonical::is_unmatched(&canonical::Require::parse(*value), reason);
            assert_eq!(
                mine, theirs,
                "is_unmatched under MW_REQUIRE_LIVE={value:?} for reason {reason:?} differs \
                 between mw-test-gate and mw-server/tests/common/gate.rs"
            );
            if mine { saw.0 = true } else { saw.1 = true }
        }
    }
    assert_eq!(
        saw,
        (true, true),
        "the matrix no longer produces both a marked and an unmarked skip"
    );

    // And the marker's own text, which is what CI counts with `grep -c '^UNMATCHED '`.
    for (test, file, line) in [
        (None, "state.rs", 1u32),
        (Some("main"), "v2.rs", 1014),
        (Some("a_leg"), r"crates\mw-store\src\v2.rs", 1014),
    ] {
        assert_eq!(
            mw_test_gate::unmatched_line(test, file, line),
            canonical::unmatched_line(test, file, line),
            "unmatched_line({test:?}, {file:?}, {line}) differs"
        );
    }
}

#[test]
fn names_a_gate_variable_agrees() {
    for reason in UNMATCHED_REASONS.iter().chain(REASONS) {
        assert_eq!(
            mw_test_gate::names_a_gate_variable(reason),
            canonical::names_a_gate_variable(reason),
            "names_a_gate_variable({reason:?}) differs between mw-test-gate and \
             mw-server/tests/common/gate.rs"
        );
    }
}

/// The six legs this crate exists for must each name a gate variable, or
/// `MW_REQUIRE_LIVE`'s list form cannot assert them and `store-dual-backend` would
/// mark them `UNMATCHED` instead of requiring them. The reasons are built in
/// `mw-engine/src/state.rs` and `mw-store/src/v2.rs`; these are their exact texts.
#[test]
fn the_reasons_this_crate_was_added_for_are_assertable() {
    for reason in [
        "[mw-engine t22-e0] session_state is one statement per group: MW_E14_PG_DSN and \
         DATABASE_URL_PG unset — the live Postgres statement-count legs are not driven. The \
         SQLite legs still asserted.",
        "[mw-store] t22-e1 record_changes: Postgres path (set DATABASE_URL_PG or MW_TEST_PG to \
         a live postgres:16 to run it). The SQLite path still asserted.",
    ] {
        assert!(
            mw_test_gate::names_a_gate_variable(reason),
            "this reason names no gate variable, so no job can require the leg: {reason:?}"
        );
        assert!(!mw_test_gate::is_unmatched(
            &mw_test_gate::Require::parse(Some("MW_E14_PG_DSN,DATABASE_URL_PG")),
            reason
        ));
    }
}

/// The two `pub const`s are what every message in both copies is formatted around, so
/// a silent rename on one side would change what CI is told to set.
#[test]
fn the_public_constants_agree() {
    assert_eq!(mw_test_gate::SKIP_LOG_VAR, canonical::SKIP_LOG_VAR);
    assert_eq!(mw_test_gate::REQUIRE_LIVE_VAR, canonical::REQUIRE_LIVE_VAR);
}

// ── the set check: a public item added to canonical alone must go red ───────────

/// Canonical's public items that this file compares, each paired with the spelling it
/// is compared through. The pairing is what stops the list from claiming a comparison
/// nobody wrote: `every_public_item_of_canonical_is_accounted_for` greps this very
/// file for the spelling.
const COMPARED: &[(&str, &str)] = &[
    ("SKIP_LOG_VAR", "canonical::SKIP_LOG_VAR"),
    ("REQUIRE_LIVE_VAR", "canonical::REQUIRE_LIVE_VAR"),
    // Compared through the Debug spelling of every variant `parse` can produce.
    ("Require", "canonical::Require"),
    ("parse", "canonical::Require::parse"),
    ("require_violation", "canonical::require_violation"),
    ("pg_dsn_from", "canonical::pg_dsn_from"),
    ("skip_line", "canonical::skip_line"),
    ("require_failure", "canonical::require_failure"),
    ("unmatched_line", "canonical::unmatched_line"),
    ("is_unmatched", "canonical::is_unmatched"),
    ("names_a_gate_variable", "canonical::names_a_gate_variable"),
];

/// Canonical's public items this file deliberately does **not** compare, because each
/// reads or writes process-global state and two copies called in one process would
/// observe each other. Every decision they make is made by something in [`COMPARED`].
/// They must still be **present** in the copy.
const NOT_COMPARED: &[&str] = &["from_env", "pg_dsn", "skip"];

/// Every `pub` item name declared in a Rust source text, in order of appearance.
fn public_items(src: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in src.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("pub ") else {
            continue;
        };
        for kw in [
            "fn ", "const ", "enum ", "struct ", "trait ", "type ", "mod ",
        ] {
            if let Some(tail) = rest.strip_prefix(kw) {
                let name: String = tail
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    found.push(name);
                }
                break;
            }
        }
    }
    found
}

/// Everything wrong with the accounting between the two copies, as reader-facing
/// complaints. Pure over the three texts so its discriminating power can be tested.
fn accounting_complaints(canonical_src: &str, copy_src: &str, test_src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let canon = public_items(canonical_src);
    let copy = public_items(copy_src);

    // 1. The door this check exists for: canonical grew a public item nobody classified.
    for name in &canon {
        let known =
            COMPARED.iter().any(|(n, _)| n == name) || NOT_COMPARED.contains(&name.as_str());
        if !known {
            out.push(format!(
                "`{name}` is public in mw-server/tests/common/gate.rs and this file neither \
                 compares it nor records why not. Port it into mw-test-gate/src/lib.rs, then \
                 add it to COMPARED with a matrix that exercises it — or to NOT_COMPARED with \
                 the reason, in the module docs. Comparing only the functions both copies \
                 happen to share is what let t24-e18's three additions through."
            ));
        }
    }

    // 2. Classified but not actually ported: the two copies cannot agree about an item
    //    only one of them has.
    for name in COMPARED
        .iter()
        .map(|(n, _)| *n)
        .chain(NOT_COMPARED.iter().copied())
    {
        if canon.iter().any(|c| c == name) && !copy.iter().any(|c| c == name) {
            out.push(format!(
                "`{name}` is public in the canonical file and missing from \
                 mw-test-gate/src/lib.rs — the copies have diverged."
            ));
        }
    }

    // 3. A COMPARED entry this file never actually calls. Without this the table could
    //    silence complaint 1 by naming an item and comparing nothing.
    for (name, spelling) in COMPARED {
        if !test_src.contains(spelling) {
            out.push(format!(
                "COMPARED lists `{name}` but this file never names `{spelling}`, so nothing \
                 compares it. Write the comparison or move it to NOT_COMPARED."
            ));
        }
    }
    out
}

fn canonical_src() -> String {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../mw-server/tests/common/gate.rs");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

fn copy_src() -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

/// This file's own source, so a COMPARED entry cannot claim a comparison nobody wrote.
const TEST_SRC: &str = include_str!("agrees_with_common_gate.rs");

#[test]
fn every_public_item_of_canonical_is_accounted_for() {
    let complaints = accounting_complaints(&canonical_src(), &copy_src(), TEST_SRC);
    assert!(
        complaints.is_empty(),
        "the two copies of the gate convention are out of step:\n  {}",
        complaints.join("\n  ")
    );
}

/// The check above is only worth having if it can actually see the divergence it was
/// built for, so this drives it against the **real** canonical text with one extra
/// public function appended — the "add a scratch function to canonical and watch it go
/// red" proof, as a permanent assertion rather than a one-off edit to a file other
/// lanes are committing to.
#[test]
fn the_accounting_check_detects_a_canonical_only_addition() {
    let real = canonical_src();
    let copy = copy_src();
    assert!(
        accounting_complaints(&real, &copy, TEST_SRC).is_empty(),
        "precondition: the real pair must be clean before this proves anything"
    );

    let grown =
        format!("{real}\npub fn a_scratch_function_only_canonical_has() -> bool {{ true }}\n");
    let complaints = accounting_complaints(&grown, &copy, TEST_SRC);
    // Exactly one: the item is unclassified. The "classified but not ported" complaint
    // deliberately does not also fire — it only speaks about names the tables list, so
    // an unclassified addition is reported once and for the right reason, rather than
    // twice for reasons a reader would have to disentangle.
    assert_eq!(
        complaints.len(),
        1,
        "expected exactly the unclassified complaint, got: {complaints:#?}"
    );
    assert!(
        complaints[0].contains("a_scratch_function_only_canonical_has")
            && complaints[0].contains("neither compares it nor records why not"),
        "the unclassified complaint must name the new function: {:?}",
        complaints[0]
    );

    // And the mirror: an item classified and compared, but dropped from the copy.
    let shrunk = copy.replace("pub fn is_unmatched(", "fn is_unmatched(");
    let complaints = accounting_complaints(&real, &shrunk, TEST_SRC);
    assert!(
        complaints
            .iter()
            .any(|c| c.contains("`is_unmatched`") && c.contains("have diverged")),
        "un-porting a compared function must be caught too: {complaints:#?}"
    );
}

/// `require_failure` is two lines of `format!`, but it is the text a red CI job is
/// read through, so it is pinned too.
#[test]
fn require_failure_agrees() {
    let record = "SKIPPED some_leg (v2.rs:1014): DATABASE_URL_PG unset.";
    let why = "MW_REQUIRE_LIVE names DATABASE_URL_PG…";
    assert_eq!(
        mw_test_gate::require_failure(record, why),
        canonical::require_failure(record, why)
    );
}
