#![no_main]
use libfuzzer_sys::fuzz_target;

// CAPA / UIDL / LIST bodies come off the wire from the upstream POP3 server.
// None of these line/body parsers may panic on arbitrary bytes.
fuzz_target!(|data: &[u8]| {
    let _ = mw_pop3::proto::parse_capa(data);
    let _ = mw_pop3::proto::parse_uidl_body(data);
    let _ = mw_pop3::proto::parse_list_body(data);
});
