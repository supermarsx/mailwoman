#![no_main]
//! `cargo-fuzz` target: raw bytes → the `mw-dav` shared-core DAV XML parsers
//! (plan §1.3, §1.9, SPEC §4.3).
//!
//! Invariant: parsing an untrusted `multistatus` / free-busy response must never
//! panic — only return `Ok`/`Err`. The harness feeds arbitrary bytes; any
//! panic/abort is a finding. Each parser is exercised (CalDAV + CardDAV).
//!
//! Run (nightly + cargo-fuzz):
//!   cargo +nightly fuzz run dav_xml
//! Build only:
//!   cargo +nightly fuzz build

use libfuzzer_sys::fuzz_target;
use mw_dav::request::DavKind;
use mw_dav::response;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    let _ = response::parse_sync_delta(s);
    let _ = response::parse_collections(s, DavKind::CalDav);
    let _ = response::parse_collections(s, DavKind::CardDav);
    let _ = response::parse_multiget(s, DavKind::CalDav);
    let _ = response::parse_multiget(s, DavKind::CardDav);
    let _ = response::parse_etag_list(s);
    let _ = response::parse_current_user_principal(s);
    let _ = response::parse_home_set(s, DavKind::CalDav);
    let _ = response::parse_free_busy(s);
});
