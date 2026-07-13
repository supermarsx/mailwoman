#![no_main]
//! Fuzz the CMS (S/MIME SignedData) parser via verify + cert harvest: arbitrary
//! bytes must only ever produce `Err`, never a panic (plan §4.3).
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = mw_crypto::smime::verify(data);
    let _ = mw_crypto::smime::harvest_certs(data);
});
