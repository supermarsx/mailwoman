//! The one wall clock every validity check reads (key expiry, revocation time,
//! certificate validity windows). `std::time::SystemTime` has no clock on
//! `wasm32-unknown-unknown`, so the wasm build reads the JS clock instead. Every
//! function that depends on "now" also has an `_at(now)` twin taking the time as an
//! argument, which is what the tests use to pin expiry and validity cases.

/// Seconds since the Unix epoch.
#[cfg(not(target_arch = "wasm32"))]
pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Seconds since the Unix epoch, from the JS clock.
#[cfg(target_arch = "wasm32")]
pub fn now_secs() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

/// RFC 3339 UTC text for a Unix time (`2026-10-25T09:00:00Z`).
pub fn rfc3339(secs: u64) -> String {
    let secs = i64::try_from(secs).unwrap_or(i64::MAX);
    chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0)
        .map(|d| d.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        .unwrap_or_default()
}

/// Parse RFC 3339 text back to a Unix time (`None` when it does not parse or is
/// before 1970).
pub fn parse_rfc3339(text: &str) -> Option<u64> {
    chrono::DateTime::parse_from_rfc3339(text)
        .ok()
        .and_then(|d| u64::try_from(d.timestamp()).ok())
}
