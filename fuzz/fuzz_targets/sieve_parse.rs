#![no_main]
use libfuzzer_sys::fuzz_target;

// Sieve scripts are user-supplied and may be uploaded over ManageSieve. The
// parser must reject malformed scripts with an error, never a panic.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = mw_sieve::parse::parse(s);
    }
});
