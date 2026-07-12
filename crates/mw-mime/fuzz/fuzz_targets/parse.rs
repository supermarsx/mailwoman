#![no_main]
//! `cargo-fuzz` target: raw RFC822 bytes → `mw_mime::parse`.
//!
//! Invariant (SPEC §4.3, plan §7): parsing untrusted bytes must never panic.
//! The harness feeds arbitrary bytes; any panic/abort is a finding. We ignore
//! the `Ok`/`Err` result — only the absence of a crash matters.
//!
//! Run (nightly + cargo-fuzz):
//!   cargo +nightly fuzz run parse
//! Build only:
//!   cargo +nightly fuzz build

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = mw_mime::parse(data);
});
