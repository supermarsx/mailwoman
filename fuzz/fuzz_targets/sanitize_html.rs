#![no_main]
use libfuzzer_sys::fuzz_target;

// The HTML sanitizer runs on untrusted email bodies before they are shown to a
// user. It must always return owned, safe HTML and never panic, no matter how
// malformed or hostile the input markup is.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = mw_sanitize::sanitize_email_html(s);
    }
});
