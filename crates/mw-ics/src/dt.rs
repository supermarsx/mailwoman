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

/// Windows time-zone names (what Outlook and Exchange write as `TZID`) mapped
/// to their IANA zone, for the zones most invitations use. The list follows
/// CLDR `windowsZones.xml` (territory 001).
const WINDOWS_ZONES: &[(&str, &str)] = &[
    ("Dateline Standard Time", "Etc/GMT+12"),
    ("Hawaiian Standard Time", "Pacific/Honolulu"),
    ("Alaskan Standard Time", "America/Anchorage"),
    ("Pacific Standard Time", "America/Los_Angeles"),
    ("US Mountain Standard Time", "America/Phoenix"),
    ("Mountain Standard Time", "America/Denver"),
    ("Central America Standard Time", "America/Guatemala"),
    ("Central Standard Time", "America/Chicago"),
    ("Central Standard Time (Mexico)", "America/Mexico_City"),
    ("Canada Central Standard Time", "America/Regina"),
    ("SA Pacific Standard Time", "America/Bogota"),
    ("Eastern Standard Time", "America/New_York"),
    ("US Eastern Standard Time", "America/Indianapolis"),
    ("Atlantic Standard Time", "America/Halifax"),
    ("SA Western Standard Time", "America/La_Paz"),
    ("Pacific SA Standard Time", "America/Santiago"),
    ("Newfoundland Standard Time", "America/St_Johns"),
    ("E. South America Standard Time", "America/Sao_Paulo"),
    ("Argentina Standard Time", "America/Buenos_Aires"),
    ("SA Eastern Standard Time", "America/Cayenne"),
    ("Greenland Standard Time", "America/Godthab"),
    ("UTC-02", "Etc/GMT+2"),
    ("Azores Standard Time", "Atlantic/Azores"),
    ("Cape Verde Standard Time", "Atlantic/Cape_Verde"),
    ("UTC", "Etc/UTC"),
    ("GMT Standard Time", "Europe/London"),
    ("Greenwich Standard Time", "Atlantic/Reykjavik"),
    ("W. Europe Standard Time", "Europe/Berlin"),
    ("Central Europe Standard Time", "Europe/Budapest"),
    ("Romance Standard Time", "Europe/Paris"),
    ("Central European Standard Time", "Europe/Warsaw"),
    ("W. Central Africa Standard Time", "Africa/Lagos"),
    ("Morocco Standard Time", "Africa/Casablanca"),
    ("GTB Standard Time", "Europe/Bucharest"),
    ("E. Europe Standard Time", "Europe/Chisinau"),
    ("Egypt Standard Time", "Africa/Cairo"),
    ("FLE Standard Time", "Europe/Kiev"),
    ("Israel Standard Time", "Asia/Jerusalem"),
    ("South Africa Standard Time", "Africa/Johannesburg"),
    ("Turkey Standard Time", "Europe/Istanbul"),
    ("Arabic Standard Time", "Asia/Baghdad"),
    ("Arab Standard Time", "Asia/Riyadh"),
    ("Russian Standard Time", "Europe/Moscow"),
    ("E. Africa Standard Time", "Africa/Nairobi"),
    ("Iran Standard Time", "Asia/Tehran"),
    ("Arabian Standard Time", "Asia/Dubai"),
    ("Afghanistan Standard Time", "Asia/Kabul"),
    ("Pakistan Standard Time", "Asia/Karachi"),
    ("West Asia Standard Time", "Asia/Tashkent"),
    ("India Standard Time", "Asia/Calcutta"),
    ("Sri Lanka Standard Time", "Asia/Colombo"),
    ("Nepal Standard Time", "Asia/Katmandu"),
    ("Bangladesh Standard Time", "Asia/Dhaka"),
    ("Myanmar Standard Time", "Asia/Rangoon"),
    ("SE Asia Standard Time", "Asia/Bangkok"),
    ("China Standard Time", "Asia/Shanghai"),
    ("Singapore Standard Time", "Asia/Singapore"),
    ("Taipei Standard Time", "Asia/Taipei"),
    ("W. Australia Standard Time", "Australia/Perth"),
    ("Tokyo Standard Time", "Asia/Tokyo"),
    ("Korea Standard Time", "Asia/Seoul"),
    ("Cen. Australia Standard Time", "Australia/Adelaide"),
    ("AUS Central Standard Time", "Australia/Darwin"),
    ("E. Australia Standard Time", "Australia/Brisbane"),
    ("AUS Eastern Standard Time", "Australia/Sydney"),
    ("Tasmania Standard Time", "Australia/Hobart"),
    ("West Pacific Standard Time", "Pacific/Port_Moresby"),
    ("Central Pacific Standard Time", "Pacific/Guadalcanal"),
    ("New Zealand Standard Time", "Pacific/Auckland"),
    ("Fiji Standard Time", "Pacific/Fiji"),
    ("Tonga Standard Time", "Pacific/Tongatapu"),
];

/// The IANA zone a `TZID` parameter names, or the TZID unchanged when it is
/// already one or cannot be resolved.
///
/// Calendars write `TZID` three ways: an IANA name (`Europe/Lisbon`); a
/// Windows name (`W. Europe Standard Time`, Outlook and Exchange); or a
/// vendor path ending in an IANA name (`/mozilla.org/20050126_1/Europe/Berlin`,
/// `/softwarestudio.org/Olson_20011030_5/America/New_York`). The VTIMEZONE
/// rules that come with the message are not evaluated: every zone resolved
/// here is one the tz database already knows, and its rules are the current
/// ones, which is what the sending client meant.
pub fn resolve_tzid(raw: &str) -> String {
    let tzid = raw.trim().trim_matches('"');
    if tzid.parse::<chrono_tz::Tz>().is_ok() {
        return tzid.to_string();
    }
    if let Some((_, iana)) = WINDOWS_ZONES
        .iter()
        .find(|(win, _)| win.eq_ignore_ascii_case(tzid))
    {
        return (*iana).to_string();
    }
    // A vendor prefix: try the last two, then three, path segments.
    let parts: Vec<&str> = tzid.split('/').filter(|p| !p.is_empty()).collect();
    for take in [2usize, 3] {
        if parts.len() >= take {
            let candidate = parts[parts.len() - take..].join("/");
            if candidate.parse::<chrono_tz::Tz>().is_ok() {
                return candidate;
            }
        }
    }
    tzid.to_string()
}

#[cfg(test)]
mod tzid_tests {
    use super::resolve_tzid;

    #[test]
    fn tzids_resolve_to_iana_names() {
        assert_eq!(resolve_tzid("Europe/Lisbon"), "Europe/Lisbon");
        assert_eq!(resolve_tzid("W. Europe Standard Time"), "Europe/Berlin");
        assert_eq!(
            resolve_tzid("\"Pacific Standard Time\""),
            "America/Los_Angeles"
        );
        assert_eq!(
            resolve_tzid("/mozilla.org/20050126_1/Europe/Berlin"),
            "Europe/Berlin"
        );
        assert_eq!(
            resolve_tzid("/softwarestudio.org/Olson_20011030_5/America/Argentina/Buenos_Aires"),
            "America/Argentina/Buenos_Aires"
        );
        assert_eq!(resolve_tzid("Custom Zone"), "Custom Zone");
    }
}
