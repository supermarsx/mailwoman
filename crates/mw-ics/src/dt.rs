//! Small date-time + ISO-8601 duration helpers shared by the iCalendar layer.
//!
//! Times are carried in the Mailwoman projection as JSCalendar `LocalDateTime`
//! wall-clock strings (`"2026-03-22T09:00:00"`) or date-only strings
//! (`"2026-03-22"`); the offset lives in a separate `timeZone` (TZID). These
//! helpers convert between that projection form and the compact RFC 5545
//! property value form (`20260322T090000` / `20260322`), and do nominal
//! (DST-independent) duration arithmetic in wall time.

use chrono::{NaiveDate, NaiveDateTime};

/// One parsed iCalendar date/date-time property value.
pub struct IcalDt {
    /// Wall-clock local form: `"2026-03-22T09:00:00"` or date-only `"2026-03-22"`.
    pub local: String,
    /// True when the value was a `VALUE=DATE` (date-only / floating all-day).
    pub is_date: bool,
    /// True when the value carried a trailing `Z` (UTC).
    pub is_utc: bool,
}

/// Parse a compact RFC 5545 date or date-time value into the projection form.
pub fn parse_ical_dt(raw: &str) -> Option<IcalDt> {
    let v = raw.trim();
    let is_utc = v.ends_with('Z');
    let body = v.strip_suffix('Z').unwrap_or(v);
    if body.contains('T') {
        let ndt = NaiveDateTime::parse_from_str(body, "%Y%m%dT%H%M%S").ok()?;
        Some(IcalDt {
            local: ndt.format("%Y-%m-%dT%H:%M:%S").to_string(),
            is_date: false,
            is_utc,
        })
    } else {
        let d = NaiveDate::parse_from_str(body, "%Y%m%d").ok()?;
        Some(IcalDt {
            local: d.format("%Y-%m-%d").to_string(),
            is_date: true,
            is_utc: false,
        })
    }
}

/// Projection local form (`2026-03-22T09:00:00` / `2026-03-22`) → compact RFC
/// 5545 value (`20260322T090000` / `20260322`), stripping the ISO separators.
pub fn to_compact(local: &str) -> String {
    local.replace(['-', ':'], "")
}

/// Parse a projection `LocalDateTime` into a `NaiveDateTime` (date-only values
/// are anchored to midnight).
pub fn local_to_naive(local: &str) -> Option<NaiveDateTime> {
    if local.contains('T') {
        NaiveDateTime::parse_from_str(local, "%Y-%m-%dT%H:%M:%S").ok()
    } else {
        NaiveDate::parse_from_str(local, "%Y-%m-%d")
            .ok()
            .and_then(|d| d.and_hms_opt(0, 0, 0))
    }
}

/// Format a nominal duration (in whole seconds) as an ISO-8601 duration.
/// `date_only` renders whole-day durations as `P<n>D` (all-day events).
pub fn secs_to_iso_duration(secs: i64, date_only: bool) -> String {
    if secs <= 0 {
        return if date_only {
            "P1D".into()
        } else {
            "PT0S".into()
        };
    }
    if date_only && secs % 86_400 == 0 {
        return format!("P{}D", secs / 86_400);
    }
    let days = secs / 86_400;
    let mut rem = secs % 86_400;
    let h = rem / 3600;
    rem %= 3600;
    let m = rem / 60;
    let s = rem % 60;
    let mut out = String::from("P");
    if days > 0 {
        out.push_str(&format!("{days}D"));
    }
    if h > 0 || m > 0 || s > 0 {
        out.push('T');
        if h > 0 {
            out.push_str(&format!("{h}H"));
        }
        if m > 0 {
            out.push_str(&format!("{m}M"));
        }
        if s > 0 {
            out.push_str(&format!("{s}S"));
        }
    }
    out
}

/// Parse an ISO-8601 duration (`P1D`, `PT1H30M`, `-PT15M`, …) into whole
/// seconds. Weeks (`P2W`) are supported; months/years are rejected as
/// non-nominal (they never appear in a VEVENT DURATION). Returns `None` on
/// malformed input.
pub fn iso_duration_to_secs(raw: &str) -> Option<i64> {
    let (sign, rest) = match raw.strip_prefix('-') {
        Some(r) => (-1i64, r),
        None => (1i64, raw.strip_prefix('+').unwrap_or(raw)),
    };
    let rest = rest.strip_prefix('P')?;
    if let Some(weeks) = rest.strip_suffix('W') {
        let w: i64 = weeks.parse().ok()?;
        return Some(sign * w * 7 * 86_400);
    }
    let (date_part, time_part) = match rest.split_once('T') {
        Some((d, t)) => (d, t),
        None => (rest, ""),
    };
    let mut total = 0i64;
    let mut num = String::new();
    for c in date_part.chars() {
        if c.is_ascii_digit() {
            num.push(c);
        } else {
            let n: i64 = num.parse().ok()?;
            num.clear();
            match c {
                'D' => total += n * 86_400,
                _ => return None, // Y / M are non-nominal, reject.
            }
        }
    }
    if !num.is_empty() {
        return None;
    }
    for c in time_part.chars() {
        if c.is_ascii_digit() {
            num.push(c);
        } else {
            let n: i64 = num.parse().ok()?;
            num.clear();
            match c {
                'H' => total += n * 3600,
                'M' => total += n * 60,
                'S' => total += n,
                _ => return None,
            }
        }
    }
    if !num.is_empty() {
        return None;
    }
    Some(sign * total)
}
