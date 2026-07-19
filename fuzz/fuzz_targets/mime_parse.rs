#![no_main]
use libfuzzer_sys::fuzz_target;

// Raw RFC822 bytes reach `mw_mime::parse` from untrusted, attacker-controlled
// mail inside the render jail. The parser is contracted to never panic; a crash
// here is a denial of service on message rendering.
fuzz_target!(|data: &[u8]| {
    let _ = mw_mime::parse(data);
});
