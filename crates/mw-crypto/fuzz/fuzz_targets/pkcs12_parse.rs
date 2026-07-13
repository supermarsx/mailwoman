#![no_main]
//! Fuzz the PKCS#12 (PFX) parser via import: arbitrary bytes must only ever produce
//! `Err`, never a panic (plan §4.3).
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = mw_crypto::smime::import_pkcs12(data, "password");
});
