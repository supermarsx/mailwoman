//! `cargo-fuzz` target over raw server-response bytes.
//!
//! Feeds arbitrary bytes through the same streaming parse loop the connection
//! read path uses ([`mw_imap::fuzz_parse_responses`]). The invariant: no input,
//! however malformed or truncated, may panic or hang the parser wrapper.
//!
//! Run (nightly): `cargo fuzz run parse_response`.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    mw_imap::fuzz_parse_responses(data);
});
