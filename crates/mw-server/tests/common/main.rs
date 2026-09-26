//! Test target `common` — the one place `mw-store`'s `test_db` helper is
//! proven, rather than merely used.
//!
//! Cargo auto-discovers `tests/<dir>/main.rs` as an integration-test target, so
//! this file is compiled exactly once even though its sibling `mod.rs` is
//! `#[path]`-included into every other `mw-server` test binary. Putting the
//! assertions in `test_db.rs` itself would have replicated them across ~35
//! binaries and inflated the suite's headline test count by a couple of hundred
//! duplicates, which is the sort of number that misleads a release report.
//!
//! What is asserted here is the property the whole test suite now leans on:
//! that two tests can never be handed the same SQLite path, whatever the host's
//! clock resolution and whatever `--test-threads` is set to.

#[path = "../../../mw-store/src/test_db.rs"]
mod test_db;

// `pg_dsn` and `skip` read the process environment; their logic is asserted here
// through `pg_dsn_from` and `skip_line`, which is why the two are unused.
#[allow(dead_code)]
#[path = "gate.rs"]
mod gate;

use std::collections::HashSet;

#[test]
fn tags_are_distinct_across_calls() {
    let tags: HashSet<String> = (0..1000).map(|_| test_db::unique_tag()).collect();
    assert_eq!(tags.len(), 1000, "unique_tag() repeated a value");
}

#[test]
fn tags_are_distinct_under_thread_contention() {
    // Exactly the case the old clock-based helpers got wrong: many test
    // threads asking at the same instant. Any residual dependence on timer
    // resolution shows up here as a duplicate.
    let hs: Vec<_> = (0..16)
        .map(|_| {
            std::thread::spawn(|| {
                (0..250)
                    .map(|_| test_db::unique_tag())
                    .collect::<Vec<String>>()
            })
        })
        .collect();
    let all: Vec<String> = hs.into_iter().flat_map(|h| h.join().unwrap()).collect();
    let distinct: HashSet<&String> = all.iter().collect();
    assert_eq!(
        distinct.len(),
        all.len(),
        "unique_tag() collided under contention"
    );
}

#[test]
fn every_tag_carries_pid_salt_and_counter() {
    let tag = test_db::unique_tag();
    let parts: Vec<&str> = tag.split('-').collect();
    assert_eq!(parts.len(), 3, "tag {tag} is not <pid>-<salt>-<seq>");
    assert_eq!(parts[0], std::process::id().to_string(), "pid component");
    assert!(!parts[1].is_empty(), "salt component of {tag} is empty");
    assert_ne!(parts[1], "0", "salt component of {tag} is degenerate");
    assert!(
        parts[2].parse::<u64>().is_ok(),
        "counter component of {tag} is not a number"
    );
}

#[test]
fn tags_do_not_consult_the_wall_clock() {
    // The counter is what makes the helper independent of the host's timer, so
    // it must advance across a sleep that a timestamp would have jumped over.
    // Sibling tests in this binary share the counter and run in parallel, so
    // the only safe claim is strict increase — not "by exactly one". (Asserting
    // the stronger form is itself a parallelism bug, and it flaked here before
    // this comment existed.)
    let a = test_db::unique_tag();
    std::thread::sleep(std::time::Duration::from_millis(20));
    let b = test_db::unique_tag();

    let (salt_a, seq_a) = split_tag(&a);
    let (salt_b, seq_b) = split_tag(&b);
    assert_eq!(salt_a, salt_b, "salt must be fixed for the process");
    assert!(seq_b > seq_a, "counter must advance: {a} then {b}");
}

/// `(salt, counter)` from a `<pid>-<salt>-<seq>` tag.
fn split_tag(tag: &str) -> (&str, u64) {
    let mut parts = tag.split('-');
    let _pid = parts.next().expect("pid");
    let salt = parts.next().expect("salt");
    let seq = parts.next().expect("seq").parse().expect("seq is numeric");
    (salt, seq)
}

#[test]
fn dirs_are_fresh_empty_and_under_one_root() {
    let a = test_db::unique_dir("selftest-dir");
    let b = test_db::unique_dir("selftest-dir");
    assert_ne!(a, b);
    assert!(a.is_dir() && b.is_dir());
    assert_eq!(std::fs::read_dir(&a).unwrap().count(), 0, "dir not empty");
    assert!(a.starts_with(test_db::root()));
    std::fs::remove_dir_all(&a).ok();
    std::fs::remove_dir_all(&b).ok();
}

#[test]
fn each_db_path_gets_its_own_directory() {
    // The sidecar guarantee: `-wal` / `-shm` land beside the database, so two
    // databases must never share a parent.
    let a = test_db::unique_db_path("selftest-db");
    let b = test_db::unique_db_path("selftest-db");
    assert_ne!(a, b);
    assert_ne!(a.parent().unwrap(), b.parent().unwrap());
    assert!(a.parent().unwrap().is_dir());
    // Not created for us — `Store::open` does that.
    assert!(!a.exists(), "database file must not pre-exist");
    std::fs::remove_dir_all(a.parent().unwrap()).ok();
    std::fs::remove_dir_all(b.parent().unwrap()).ok();
}

#[test]
fn concurrent_dir_allocation_never_collides() {
    // `unique_dir` panics on AlreadyExists, so this test would fail loudly
    // rather than silently if the guarantee regressed.
    let hs: Vec<_> = (0..12)
        .map(|_| {
            std::thread::spawn(|| {
                (0..25)
                    .map(|_| test_db::unique_dir("selftest-race"))
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    let dirs: Vec<_> = hs.into_iter().flat_map(|h| h.join().unwrap()).collect();
    let distinct: HashSet<_> = dirs.iter().collect();
    assert_eq!(
        distinct.len(),
        dirs.len(),
        "unique_dir() handed out a repeat"
    );
    for d in &dirs {
        std::fs::remove_dir_all(d).ok();
    }
}

#[test]
fn pg_dsn_prefers_e14_then_falls_back_to_database_url_pg() {
    let env = |pairs: &'static [(&'static str, &'static str)]| {
        move |name: &str| {
            pairs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.to_string())
        }
    };
    assert_eq!(gate::pg_dsn_from(env(&[])), None);
    assert_eq!(
        gate::pg_dsn_from(env(&[("DATABASE_URL_PG", "pg://ci")])).as_deref(),
        Some("pg://ci"),
        "CI's store-dual-backend sets only DATABASE_URL_PG; a PG leg must find it"
    );
    assert_eq!(
        gate::pg_dsn_from(env(&[
            ("MW_E14_PG_DSN", "pg://e14"),
            ("DATABASE_URL_PG", "pg://ci")
        ]))
        .as_deref(),
        Some("pg://e14")
    );
    assert_eq!(
        gate::pg_dsn_from(env(&[
            ("MW_E14_PG_DSN", " "),
            ("DATABASE_URL_PG", "pg://ci")
        ]))
        .as_deref(),
        Some("pg://ci"),
        "an empty MW_E14_PG_DSN counts as unset"
    );
    assert_eq!(gate::pg_dsn_from(env(&[("DATABASE_URL_PG", "")])), None);
}

#[test]
fn skip_line_names_the_test_and_call_site_on_one_line() {
    assert_eq!(
        gate::skip_line(
            Some("leg_postgres"),
            "crates/mw-server/tests/t15_upload.rs",
            432,
            "\n[t15 upload] MW_E14_PG_DSN unset —\n   not driven.\n"
        ),
        "SKIPPED leg_postgres (t15_upload.rs:432): [t15 upload] MW_E14_PG_DSN unset — not driven."
    );
    assert_eq!(
        gate::skip_line(Some("main"), r"tests\t12_sasl.rs", 7, "why"),
        "SKIPPED (t12_sasl.rs:7): why"
    );
}

#[test]
fn require_live_is_off_unless_it_names_something() {
    use gate::Require;
    assert_eq!(Require::parse(None), Require::Off);
    assert_eq!(Require::parse(Some("")), Require::Off);
    assert_eq!(Require::parse(Some("  ")), Require::Off);
    assert_eq!(Require::parse(Some("all")), Require::All);
    assert_eq!(Require::parse(Some("ALL")), Require::All);
    assert_eq!(
        Require::parse(Some("MW_E14_PG_DSN,DATABASE_URL_PG")),
        Require::Vars(vec!["MW_E14_PG_DSN".into(), "DATABASE_URL_PG".into()])
    );
    assert_eq!(
        Require::parse(Some("MW_IMAP_LIVE MW_POP3_LIVE")),
        Require::Vars(vec!["MW_IMAP_LIVE".into(), "MW_POP3_LIVE".into()]),
        "a space-separated value is the shape a YAML folded scalar produces"
    );
}

#[test]
fn a_required_variable_turns_the_skip_that_cites_it_into_a_failure() {
    use gate::{Require, require_violation};
    let set = |_: &str| true;
    let pg = Require::parse(Some("MW_E14_PG_DSN,DATABASE_URL_PG"));

    let why = require_violation(
        &pg,
        set,
        "[t15 upload] MW_E14_PG_DSN and DATABASE_URL_PG unset — live Postgres upload \
         round-trip not driven.",
    )
    .expect("a skip citing a required variable must fail");
    assert!(why.contains("MW_E14_PG_DSN"), "names the variable: {why}");

    // The reason this is a list and not a flag: `store-dual-backend` runs the whole
    // mw-server suite with Postgres up and Dovecot deliberately down. Its Dovecot
    // legs must stay free to skip, or the switch manufactures failures.
    assert_eq!(
        require_violation(
            &pg,
            set,
            "[t13 ACL/METADATA] MW_T13_LIVE!=1 — real Dovecot not driven."
        ),
        None,
        "a service this job does not boot must still be allowed to skip"
    );

    // `all` is for a job running one narrowly gated target; there, a post-gate
    // failure to reach the booted service is exactly what must go red.
    assert!(
        require_violation(
            &Require::All,
            set,
            "[t12 IMAP] imap_scram_login_live: Dovecot-SASL unreachable at 127.0.0.1:2143 (timed out)."
        )
        .is_some(),
        "under `all` a booted-but-unreachable service is a failure, not a skip"
    );

    assert_eq!(
        require_violation(&Require::Off, set, "anything at all"),
        None
    );
}

#[test]
fn a_required_variable_that_is_set_nowhere_fails_on_its_own() {
    // The trap this switch could otherwise fall into: a job writes
    // MW_REQUIRE_LIVE=MW_E14_PG_DSNN, the name matches no skip reason, the switch
    // requires nothing, and the job stays green — a seventh check that cannot fail.
    use gate::{Require, require_violation};
    let why = require_violation(
        &Require::parse(Some("MW_E14_PG_DSNN")),
        |_| false,
        "[t13 ACL/METADATA] MW_T13_LIVE!=1 — real Dovecot not driven.",
    )
    .expect("a required name the step never exported must fail on the first skip");
    assert!(why.contains("MW_E14_PG_DSNN"), "names the typo: {why}");
    assert!(why.contains("spelling"), "says where to look: {why}");
}

#[test]
fn a_require_failure_carries_the_whole_skipped_record() {
    // Whatever the switch does, the reader gets what the gate log would have given
    // them: which test, which call site, which variable it was waiting for.
    let record = gate::skip_line(
        Some("leg_postgres"),
        "crates/mw-server/tests/t15_upload.rs",
        433,
        "[t15 upload] MW_E14_PG_DSN unset — not driven.",
    );
    let body = gate::require_failure(&record, "because the job boots Postgres.");
    assert!(body.starts_with("SKIPPED leg_postgres (t15_upload.rs:433):"));
    assert!(body.contains("MW_E14_PG_DSN"));
    assert!(body.ends_with("because the job boots Postgres."));
}

#[test]
fn a_gate_variable_is_recognised_by_shape_not_by_a_list() {
    use gate::names_a_gate_variable;
    // The reasons a list form decides on.
    assert!(names_a_gate_variable(
        "[t12 IMAP] MW_IMAP_LIVE!=1 — not driven."
    ));
    assert!(names_a_gate_variable(
        "[t15 upload] MW_E14_PG_DSN and DATABASE_URL_PG unset — not driven."
    ));
    assert!(
        names_a_gate_variable("… or set MW_E16_LDAP_URL."),
        "a guard added after this file was written must count too"
    );
    // The reasons it cannot decide on: build preconditions, platform, capability.
    assert!(!names_a_gate_variable(
        "[t17 TT] not built (no apps/web/dist/index.html) — build the web SPA."
    ));
    assert!(!names_a_gate_variable(
        "[t16 sandbox] non-Linux (windows) — kernel jail unavailable."
    ));
    assert!(!names_a_gate_variable(
        "[t12 SORT] server does not advertise SORT."
    ));
    assert!(
        !names_a_gate_variable("MW_ alone is not a variable"),
        "the bare prefix must not count"
    );
}

#[test]
fn only_a_list_marks_a_skip_it_could_not_have_spoken_to() {
    use gate::{Require, is_unmatched};
    let build_precondition = "[t17 TT] not built (no apps/web/dist/index.html).";
    let names_its_guard = "[t13 ACL/METADATA] MW_T13_LIVE!=1 — real Dovecot not driven.";
    let list = Require::parse(Some("MW_E14_PG_DSN,DATABASE_URL_PG"));

    assert!(
        is_unmatched(&list, build_precondition),
        "a list decides by matching the reason, so this skip is outside it in both \
         directions — that is the gap the marker exists to show"
    );
    assert!(
        !is_unmatched(&list, names_its_guard),
        "naming a DIFFERENT guard is not a gap: we can tell which guard this is, and \
         store-dual-backend's Dovecot legs are correctly skipping"
    );
    // Under `all` a variable-less reason is exactly what is being caught, so a marker
    // there would be noise on a working assertion.
    assert!(!is_unmatched(&Require::All, build_precondition));
    assert!(!is_unmatched(&Require::Off, build_precondition));
}

#[test]
fn an_unmatched_line_is_countable_and_says_which_leg() {
    let marker = gate::unmatched_line(
        Some("shipped_bundle_is_served_under_enforced_trusted_types"),
        "crates/mw-server/tests/t17_tt_shell.rs",
        74,
    );
    assert_eq!(
        marker,
        "UNMATCHED shipped_bundle_is_served_under_enforced_trusted_types \
         (t17_tt_shell.rs:74): reason names no gate variable; MW_REQUIRE_LIVE cannot assert it"
    );
    // Both tags lead their line, so a gate log stays countable per tag.
    assert!(marker.starts_with("UNMATCHED "));
    assert!(gate::skip_line(None, "x.rs", 1, "why").starts_with("SKIPPED "));
}

#[test]
fn skips_are_reported_through_the_shared_helper() {
    // The spelling every env-gated leg used before `common::gate::skip` existed. It went
    // through `eprintln!`, which libtest captures for a passing test. This catches a
    // new leg copying that pattern; it does not claim to recognise every early return.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("read tests/") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read test file");
        for (n, line) in text.lines().enumerate() {
            if line.contains("SKIP]") || line.contains("SKIPPED:") {
                offenders.push(format!("{}:{}", path.display(), n + 1));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "report a skipped leg with common::gate::skip(..), not eprintln!: {offenders:#?}"
    );
}

#[test]
fn named_files_land_in_their_own_directory() {
    let a = test_db::unique_file_path("selftest-file", "rules.json");
    let b = test_db::unique_file_path("selftest-file", "rules.json");
    assert_eq!(a.file_name(), b.file_name());
    assert_ne!(a.parent().unwrap(), b.parent().unwrap());
    std::fs::remove_dir_all(a.parent().unwrap()).ok();
    std::fs::remove_dir_all(b.parent().unwrap()).ok();
}
