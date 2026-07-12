#![no_main]
//! `cargo-fuzz` target: arbitrary bytes → `mw_ics::parse_vcard` (plan §1.9).
//!
//! Invariant (SPEC §4.3, plan §7): parsing untrusted vCard bytes must never
//! panic. The harness feeds arbitrary bytes; any panic/abort is a finding.
//!
//! Run (nightly + cargo-fuzz):
//!   cargo +nightly fuzz run parse_vcard
//! Build only:
//!   cargo +nightly fuzz build

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = mw_ics::parse_vcard(data);
});
