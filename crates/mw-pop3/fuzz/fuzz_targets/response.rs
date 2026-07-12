#![no_main]
//! Fuzz the POP3 server-response parsing surface (`mw_pop3::proto`).
//!
//! Run (nightly + Linux): `cargo +nightly fuzz run response`. The same entry
//! point is smoke-tested on every platform by the `proto` unit tests, so the
//! parser is checked for panics even where `cargo-fuzz` is unavailable.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    mw_pop3::fuzz_response_lines(data);
});
