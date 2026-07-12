#![no_main]
//! `cargo-fuzz` target: arbitrary bytes → `mw_ics::parse_ical` (plan §1.9).
//!
//! Invariant (SPEC §4.3, plan §7): parsing untrusted iCalendar bytes must never
//! panic. Also exercises `parse_itip`, which layers on the same reader. The
//! harness feeds arbitrary bytes; any panic/abort is a finding.
//!
//! Run (nightly + cargo-fuzz):
//!   cargo +nightly fuzz run parse_ical
//! Build only:
//!   cargo +nightly fuzz build

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = mw_ics::parse_ical(data);
    let _ = mw_ics::parse_itip(data);
    let _ = mw_ics::parse_hol(data);
});
