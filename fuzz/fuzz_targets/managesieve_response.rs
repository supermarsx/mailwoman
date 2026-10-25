#![no_main]
use libfuzzer_sys::fuzz_target;

// ManageSieve responses come from a remote server before and after login: the
// response tokenizer, the response-code parser and the compile-error position
// parser must never panic on what it sends.
fuzz_target!(|data: &[u8]| {
    mw_sieve::managesieve::fuzz_response(data);
});
