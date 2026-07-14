#![no_main]
//! `cargo-fuzz` target: arbitrary bytes → `mw_export::read_msg` (plan §3 e5, §1.7).
//!
//! Invariant (SPEC §7.5, plan §7): parsing an untrusted `.msg`/`.oft` CFB
//! (OLE2 compound-file) container must never panic. This is the hostile-parse
//! boundary flagged for the render jail (see `mw_export::msg` module docs) — the
//! parser is size-limited and panic-free so it is safe to lift into the
//! disposable `mw-render` child once that gains a CFB job frame (SEAM e14/e16).
//! `from_oft` shares the same reader. The harness feeds arbitrary bytes; any
//! panic/abort is a finding.
//!
//! Run (nightly + cargo-fuzz):
//!   cargo +nightly fuzz run parse_msg
//! Build only:
//!   cargo +nightly fuzz build

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = mw_export::read_msg(data);
    let _ = mw_export::from_oft(data);
});
