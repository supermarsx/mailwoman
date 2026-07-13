#![no_main]
//! Fuzz the OpenPGP packet/armor parsers: arbitrary bytes must only ever produce
//! `Err`, never a panic (plan §4.3).
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = mw_crypto::pgp::parse_key(s, Vec::new());
        let _ = mw_crypto::pgp::parse_autocrypt_header(s);
        let _ = mw_crypto::pgp::export_public(s);
    }
    // Message parse via decrypt (an empty bundle short-circuits before crypto).
    let _ = mw_crypto::pgp::decrypt(data, "", "", None);
});
