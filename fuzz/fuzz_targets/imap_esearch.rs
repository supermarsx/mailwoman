#![no_main]
use libfuzzer_sys::fuzz_target;

// `* ESEARCH` replies come from the upstream server and are parsed by hand
// (`imap-proto` cannot read them). The parser must never panic, and a complete
// reply must end at its CRLF.
fuzz_target!(|data: &[u8]| {
    mw_imap::esearch::fuzz(data);
});
