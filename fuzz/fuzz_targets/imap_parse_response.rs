#![no_main]
use libfuzzer_sys::fuzz_target;

// IMAP server responses arrive from a potentially hostile or buggy upstream.
// `fuzz_parse_responses` drives the same response parser the live connection
// uses and must never panic on arbitrary bytes.
fuzz_target!(|data: &[u8]| {
    mw_imap::fuzz_parse_responses(data);
});
