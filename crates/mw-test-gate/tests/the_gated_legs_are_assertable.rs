//! `MW_REQUIRE_LIVE`'s list form decides whether a skip is a failure by looking for
//! the gate variable's **name in the skip's reason**. That makes the wording of a
//! reason load-bearing: a leg whose reason names no variable can never be required by
//! a job, and it reports as a pass forever.
//!
//! The six legs this crate was added for — three in `mw-engine/src/state.rs`, three in
//! `mw-store/src/v2.rs` — were exactly that kind of leg until t25-e1, four of them
//! invisibly (`eprintln!`, which libtest swallows for a passing test). This pins their
//! reasons so a reword cannot quietly undo it: `store-dual-backend` would go back to
//! marking them `UNMATCHED` and passing, which is the original defect returning
//! through the text rather than the code.
//!
//! The reasons below are the exact strings `state.rs`'s and `v2.rs`'s `skip` helpers
//! build. If one of those is reworded, copy the new text here — and if it no longer
//! names its variable, that is the bug, not this test.

/// The engine's three legs share one reason template, and the store's three share
/// another; both are pinned rather than one standing in for the other.
const REASONS: &[&str] = &[
    // crates/mw-engine/src/state.rs
    "[mw-engine t22-e0] session_state is one statement per group: MW_E14_PG_DSN and \
     DATABASE_URL_PG unset — the live Postgres statement-count legs are not driven. The \
     SQLite legs still asserted.",
    // crates/mw-store/src/v2.rs
    "[mw-store] t22-e1 record_changes: Postgres path (set DATABASE_URL_PG or MW_TEST_PG to \
     a live postgres:16 to run it). The SQLite path still asserted.",
];

#[test]
fn each_reason_names_a_gate_variable() {
    for reason in REASONS {
        assert!(
            mw_test_gate::names_a_gate_variable(reason),
            "this reason names no gate variable, so no job can ever require the leg and it \
             will pass forever: {reason:?}"
        );
    }
}

#[test]
fn store_dual_backends_list_requires_them_rather_than_marking_them() {
    // The value `ci.yml`'s store-dual-backend step actually sets.
    let req = mw_test_gate::Require::parse(Some("MW_E14_PG_DSN,DATABASE_URL_PG"));
    for reason in REASONS {
        assert!(
            !mw_test_gate::is_unmatched(&req, reason),
            "store-dual-backend would mark this UNMATCHED and pass it instead of requiring \
             the leg: {reason:?}"
        );
        assert!(
            mw_test_gate::require_violation(&req, |_| true, reason).is_some(),
            "with both guards exported, a skip citing them must be a failure: {reason:?}"
        );
    }
}
