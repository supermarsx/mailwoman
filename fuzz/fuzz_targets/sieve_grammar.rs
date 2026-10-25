#![no_main]
use libfuzzer_sys::fuzz_target;

// Any Sieve script a user saves goes through the full grammar, the validator,
// the server policy and the builder's managed-block merge before PUTSCRIPT.
// None of them may panic or recurse without bound on any input.
fuzz_target!(|data: &[u8]| {
    mw_sieve::fuzz_sieve_grammar(data);
});
