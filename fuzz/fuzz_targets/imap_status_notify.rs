#![no_main]
use libfuzzer_sys::fuzz_target;

// Extended `* STATUS` replies (NOTIFY, STATUS=SIZE, APPENDLIMIT, MAILBOXID)
// and NOTIFY response codes come from the upstream server and are parsed by
// hand. Neither parser may panic on arbitrary bytes.
fuzz_target!(|data: &[u8]| {
    mw_imap::notify::fuzz(data);
});
