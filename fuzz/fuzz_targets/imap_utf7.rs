#![no_main]
use libfuzzer_sys::fuzz_target;

// Mailbox names in modified UTF-7 come from the upstream server. Decoding must
// never panic, and decode/encode must round-trip exactly in both directions.
fuzz_target!(|data: &[u8]| {
    mw_imap::utf7::fuzz(data);
});
