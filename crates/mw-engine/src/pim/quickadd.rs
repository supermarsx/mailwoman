//! Natural-language quick-add parsing (P3): turn a one-line phrase like
//! `"Lunch with Sam tomorrow at 1pm @Cafe for 90m"` into a structured event spec
//! (title + local start + duration + optional location / all-day). Pure and
//! offline — `chrono` only, no new dependency, no network. The engine method
//! `CalendarEvent/quickAdd` (see [`crate::pim::events`]) feeds the result into the
//! normal event-create path, so recurrence/ICS emission stay unchanged.
//!
//! Recognized grammar (bounded, deterministic; unmatched words stay in the title):
//! - **day**: `today` | `tonight` | `tomorrow` | a weekday (`mon`..`sunday`, with an
//!   optional leading `next`) | ISO `YYYY-MM-DD` | `M/D` | month-name + day.
//! - **time**: `3pm` | `3:30pm` | `15:00` | `noon` | `midnight`, optionally after
//!   `at`; a range `3pm-4pm` / `3-4pm` / `3pm to 4pm` sets the duration.
//! - **duration**: `for 90m` | `for 2h` | `for 1h30m` (overrides a range/default).
//! - **location**: `@Place` (kept out of the title).
//! - **all-day**: a day with no time → an all-day event (`P1D`).

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc, Weekday};

/// A wall-clock `(hour, minute)` in 24-hour form.
type HourMin = (u32, u32);

/// A parsed quick-add phrase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickAdd {
    pub title: String,
    /// Local start: `"YYYY-MM-DDTHH:MM:SS"` for a timed event, `"YYYY-MM-DD"` for
    /// an all-day event, or `None` if no day/time was recognized.
    pub start: Option<String>,
    /// ISO-8601 duration (`"PT1H"` timed default, `"P1D"` all-day default).
    pub duration: String,
    pub all_day: bool,
    pub location: Option<String>,
}

/// Parse `input` relative to `now` (the reference "today", usually `Utc::now()`).
pub fn parse_quick_add(input: &str, now: DateTime<Utc>) -> QuickAdd {
    let today = now.date_naive();
    let raw_tokens: Vec<&str> = input.split_whitespace().collect();

    // `kept[i] == false` marks a token consumed by a date/time/location matcher.
    let mut kept = vec![true; raw_tokens.len()];
    let lower: Vec<String> = raw_tokens.iter().map(|t| t.to_lowercase()).collect();

    let mut date: Option<NaiveDate> = None;
    let mut start_time: Option<(u32, u32)> = None; // (hour, minute)
    let mut end_time: Option<(u32, u32)> = None;
    let mut explicit_duration: Option<String> = None;
    let mut location: Option<String> = None;

    let mut i = 0usize;
    while i < raw_tokens.len() {
        let w = trim_punct(&lower[i]);

        // ── location: @Place ──
        if let Some(stripped) = raw_tokens[i].strip_prefix('@')
            && !stripped.is_empty()
        {
            location = Some(stripped.to_string());
            kept[i] = false;
            i += 1;
            continue;
        }

        // ── "for <duration>" ──
        if w == "for"
            && i + 1 < raw_tokens.len()
            && let Some(dur) = parse_duration(trim_punct(&lower[i + 1]))
        {
            explicit_duration = Some(dur);
            kept[i] = false;
            kept[i + 1] = false;
            i += 2;
            continue;
        }

        // ── filler words that precede a date/time ──
        if matches!(w, "at" | "on" | "this" | "next") {
            // "next <weekday>" bumps a week; handle it with the weekday branch.
            if w == "next"
                && i + 1 < raw_tokens.len()
                && weekday_of(trim_punct(&lower[i + 1])).is_some()
            {
                let wd = weekday_of(trim_punct(&lower[i + 1])).unwrap();
                date = Some(next_weekday(today, wd, true));
                kept[i] = false;
                kept[i + 1] = false;
                i += 2;
                continue;
            }
            // Only drop the filler if the next token is actually a date/time.
            if i + 1 < raw_tokens.len() && is_datetime_token(trim_punct(&lower[i + 1])) {
                kept[i] = false;
            }
            i += 1;
            continue;
        }

        // ── time range across tokens: "<time> to <time>" ──
        if w == "to"
            && start_time.is_some()
            && i + 1 < raw_tokens.len()
            && let Some(t) = parse_time(trim_punct(&lower[i + 1]))
        {
            end_time = Some(t);
            kept[i] = false;
            kept[i + 1] = false;
            i += 1;
            continue;
        }

        // ── day keywords / weekday / dates ──
        if date.is_none() {
            if let Some(d) = parse_day(w, today) {
                date = Some(d);
                kept[i] = false;
                i += 1;
                continue;
            }
            // month-name + day: "jul 20" / "july 20th"
            if let Some(month) = month_of(w)
                && i + 1 < raw_tokens.len()
                && let Some(day) = parse_day_number(trim_punct(&lower[i + 1]))
            {
                let year = today.year();
                if let Some(d) = NaiveDate::from_ymd_opt(year, month, day) {
                    date = Some(if d < today {
                        NaiveDate::from_ymd_opt(year + 1, month, day).unwrap_or(d)
                    } else {
                        d
                    });
                    kept[i] = false;
                    kept[i + 1] = false;
                    i += 2;
                    continue;
                }
            }
        }

        // ── time / time-range single token ──
        if let Some((s, e)) = parse_time_range(w) {
            start_time = Some(s);
            end_time = e;
            kept[i] = false;
            i += 1;
            continue;
        }
        if start_time.is_none()
            && let Some(t) = parse_time(w)
        {
            start_time = Some(t);
            kept[i] = false;
            i += 1;
            continue;
        }

        i += 1;
    }

    // "tonight" with no explicit time → default to 19:00.
    if start_time.is_none() && lower.iter().any(|t| trim_punct(t) == "tonight") {
        start_time = Some((19, 0));
    }

    let title = raw_tokens
        .iter()
        .zip(&kept)
        .filter(|(_, k)| **k)
        .map(|(t, _)| *t)
        .collect::<Vec<_>>()
        .join(" ");

    let mut qa = build(title, date, start_time, end_time, explicit_duration);
    qa.location = location;
    qa
}

fn build(
    title: String,
    date: Option<NaiveDate>,
    start_time: Option<(u32, u32)>,
    end_time: Option<(u32, u32)>,
    explicit_duration: Option<String>,
) -> QuickAdd {
    match (date, start_time) {
        (Some(d), Some((h, m))) => {
            let start = format!("{}T{:02}:{:02}:00", d.format("%Y-%m-%d"), h, m);
            let duration = explicit_duration
                .or_else(|| end_time.map(|(eh, em)| range_duration(h, m, eh, em)))
                .unwrap_or_else(|| "PT1H".to_string());
            QuickAdd {
                title,
                start: Some(start),
                duration,
                all_day: false,
                location: None,
            }
        }
        (Some(d), None) => QuickAdd {
            title,
            start: Some(d.format("%Y-%m-%d").to_string()),
            duration: explicit_duration.unwrap_or_else(|| "P1D".to_string()),
            all_day: true,
            location: None,
        },
        (None, Some((h, m))) => {
            // Time but no day → assume today.
            let today = Utc::now().date_naive();
            let start = format!("{}T{:02}:{:02}:00", today.format("%Y-%m-%d"), h, m);
            QuickAdd {
                title,
                start: Some(start),
                duration: explicit_duration
                    .or_else(|| end_time.map(|(eh, em)| range_duration(h, m, eh, em)))
                    .unwrap_or_else(|| "PT1H".to_string()),
                all_day: false,
                location: None,
            }
        }
        (None, None) => QuickAdd {
            title,
            start: None,
            duration: explicit_duration.unwrap_or_else(|| "PT1H".to_string()),
            all_day: false,
            location: None,
        },
    }
}

// ── token classifiers ────────────────────────────────────────────────────────

fn trim_punct(s: &str) -> &str {
    s.trim_matches(|c: char| matches!(c, ',' | '.' | ';' | '!' | '?'))
}

fn is_datetime_token(w: &str) -> bool {
    parse_time(w).is_some()
        || parse_time_range(w).is_some()
        || parse_day_keyword(w).is_some()
        || weekday_of(w).is_some()
        || is_iso_date(w)
        || parse_slash_date(w).is_some()
        || month_of(w).is_some()
}

fn parse_day(w: &str, today: NaiveDate) -> Option<NaiveDate> {
    if let Some(d) = parse_day_keyword(w).map(|off| today + Duration::days(off)) {
        return Some(d);
    }
    if let Some(wd) = weekday_of(w) {
        return Some(next_weekday(today, wd, false));
    }
    if is_iso_date(w) {
        return NaiveDate::parse_from_str(w, "%Y-%m-%d").ok();
    }
    if let Some(d) = parse_slash_date(w).and_then(|(mo, da)| resolve_md(today, mo, da)) {
        return Some(d);
    }
    None
}

/// `today`/`tomorrow`/`tonight` → day offset from today.
fn parse_day_keyword(w: &str) -> Option<i64> {
    match w {
        "today" | "tonight" => Some(0),
        "tomorrow" | "tmrw" => Some(1),
        _ => None,
    }
}

fn weekday_of(w: &str) -> Option<Weekday> {
    Some(match w {
        "monday" | "mon" => Weekday::Mon,
        "tuesday" | "tue" | "tues" => Weekday::Tue,
        "wednesday" | "wed" => Weekday::Wed,
        "thursday" | "thu" | "thurs" => Weekday::Thu,
        "friday" | "fri" => Weekday::Fri,
        "saturday" | "sat" => Weekday::Sat,
        "sunday" | "sun" => Weekday::Sun,
        _ => return None,
    })
}

/// The next date that is `wd`, strictly after today; `force_next_week` (from a
/// leading `next`) resolves to the following week's occurrence.
fn next_weekday(today: NaiveDate, wd: Weekday, force_next_week: bool) -> NaiveDate {
    let cur = today.weekday().num_days_from_monday() as i64;
    let target = wd.num_days_from_monday() as i64;
    let raw = (target - cur).rem_euclid(7); // 0..=6 (0 = same weekday as today)
    let delta = if force_next_week {
        raw + 7 // "next monday" → the Monday in the following week
    } else if raw == 0 {
        7 // a bare weekday name means the coming one, not today
    } else {
        raw
    };
    today + Duration::days(delta)
}

fn is_iso_date(w: &str) -> bool {
    NaiveDate::parse_from_str(w, "%Y-%m-%d").is_ok()
}

fn parse_slash_date(w: &str) -> Option<(u32, u32)> {
    let (mo, da) = w.split_once('/')?;
    Some((mo.parse().ok()?, da.parse().ok()?))
}

fn resolve_md(today: NaiveDate, mo: u32, da: u32) -> Option<NaiveDate> {
    let year = today.year();
    let d = NaiveDate::from_ymd_opt(year, mo, da)?;
    Some(if d < today {
        NaiveDate::from_ymd_opt(year + 1, mo, da).unwrap_or(d)
    } else {
        d
    })
}

fn month_of(w: &str) -> Option<u32> {
    Some(match w {
        "jan" | "january" => 1,
        "feb" | "february" => 2,
        "mar" | "march" => 3,
        "apr" | "april" => 4,
        "may" => 5,
        "jun" | "june" => 6,
        "jul" | "july" => 7,
        "aug" | "august" => 8,
        "sep" | "sept" | "september" => 9,
        "oct" | "october" => 10,
        "nov" | "november" => 11,
        "dec" | "december" => 12,
        _ => return None,
    })
}

fn parse_day_number(w: &str) -> Option<u32> {
    let n: u32 = w
        .trim_end_matches(|c: char| !c.is_ascii_digit())
        .parse()
        .ok()?;
    (1..=31).contains(&n).then_some(n)
}

/// Parse a single time token: `3pm`, `3:30pm`, `15:00`, `noon`, `midnight`.
fn parse_time(w: &str) -> Option<HourMin> {
    match w {
        "noon" => return Some((12, 0)),
        "midnight" => return Some((0, 0)),
        _ => {}
    }
    let (body, meridiem) = if let Some(b) = w.strip_suffix("am") {
        (b, Some(false))
    } else if let Some(b) = w.strip_suffix("pm") {
        (b, Some(true))
    } else {
        (w, None)
    };
    if body.is_empty() {
        return None;
    }
    let (h_str, m_str) = match body.split_once(':') {
        Some((h, m)) => (h, m),
        None => (body, "0"),
    };
    let mut hour: u32 = h_str.parse().ok()?;
    let minute: u32 = m_str.parse().ok()?;
    if minute > 59 {
        return None;
    }
    match meridiem {
        Some(true) => {
            // pm
            if hour == 12 {
                // 12pm = noon
            } else if hour < 12 {
                hour += 12;
            } else {
                return None;
            }
        }
        Some(false) => {
            // am
            if hour == 12 {
                hour = 0;
            } else if hour > 12 {
                return None;
            }
        }
        None => {
            // Bare number: only accept a 24h value or an explicit `H:MM`.
            if hour > 23 {
                return None;
            }
            // A bare integer with no colon and no meridiem is NOT a time (avoid
            // eating quantities like "5 people"); require a colon or meridiem.
            if !body.contains(':') {
                return None;
            }
        }
    }
    if hour > 23 {
        return None;
    }
    Some((hour, minute))
}

/// Parse a single-token range `3pm-4pm` / `3-4pm` / `9:00-10:30`.
fn parse_time_range(w: &str) -> Option<(HourMin, Option<HourMin>)> {
    let (a, b) = w.split_once('-')?;
    if a.is_empty() || b.is_empty() {
        return None;
    }
    // The end usually carries the meridiem ("3-4pm"): if the start lacks am/pm,
    // borrow the end's.
    let end = parse_time(b)?;
    let start = parse_time(a).or_else(|| {
        let mer = if b.ends_with("pm") {
            "pm"
        } else if b.ends_with("am") {
            "am"
        } else {
            return None;
        };
        parse_time(&format!("{a}{mer}"))
    })?;
    Some((start, Some(end)))
}

/// Parse a duration token: `90m`, `2h`, `1h30m`, `45min`.
fn parse_duration(w: &str) -> Option<String> {
    let s = w.trim();
    if s.is_empty() {
        return None;
    }
    let mut hours = 0i64;
    let mut mins = 0i64;
    let mut num = String::new();
    let mut saw = false;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_ascii_digit() {
            num.push(c);
        } else {
            let n: i64 = num.parse().ok()?;
            num.clear();
            // consume a unit word (h/hr/hour(s), m/min(s))
            let mut unit = c.to_ascii_lowercase().to_string();
            while let Some(&nc) = chars.peek() {
                if nc.is_ascii_alphabetic() {
                    unit.push(nc.to_ascii_lowercase());
                    chars.next();
                } else {
                    break;
                }
            }
            match unit.as_str() {
                "h" | "hr" | "hrs" | "hour" | "hours" => hours += n,
                "m" | "min" | "mins" | "minute" | "minutes" => mins += n,
                _ => return None,
            }
            saw = true;
        }
    }
    // A trailing bare number defaults to minutes ("for 30").
    if !num.is_empty() {
        mins += num.parse::<i64>().ok()?;
        saw = true;
    }
    if !saw {
        return None;
    }
    Some(iso_duration(hours, mins))
}

fn iso_duration(hours: i64, mins: i64) -> String {
    let total = hours * 60 + mins;
    if total <= 0 {
        return "PT1H".to_string();
    }
    let h = total / 60;
    let m = total % 60;
    match (h, m) {
        (h, 0) => format!("PT{h}H"),
        (0, m) => format!("PT{m}M"),
        (h, m) => format!("PT{h}H{m}M"),
    }
}

fn range_duration(sh: u32, sm: u32, eh: u32, em: u32) -> String {
    let start = sh as i64 * 60 + sm as i64;
    let mut end = eh as i64 * 60 + em as i64;
    if end <= start {
        end += 24 * 60; // wraps past midnight
    }
    iso_duration(0, end - start)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(y: i32, m: u32, d: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, 9, 0, 0).unwrap()
    }

    #[test]
    fn tomorrow_at_1pm_with_location_and_duration() {
        // 2026-07-19 is a Sunday.
        let q = parse_quick_add(
            "Lunch with Sam tomorrow at 1pm @Cafe for 90m",
            at(2026, 7, 19),
        );
        assert_eq!(q.title, "Lunch with Sam");
        assert_eq!(q.start.as_deref(), Some("2026-07-20T13:00:00"));
        assert_eq!(q.duration, "PT1H30M");
        assert_eq!(q.location.as_deref(), Some("Cafe"));
        assert!(!q.all_day);
    }

    #[test]
    fn range_sets_duration() {
        let q = parse_quick_add("Team sync 3pm-4pm", at(2026, 7, 19));
        assert_eq!(q.title, "Team sync");
        assert_eq!(q.start.as_deref(), Some("2026-07-19T15:00:00"));
        assert_eq!(q.duration, "PT1H");
    }

    #[test]
    fn range_borrows_meridiem() {
        let q = parse_quick_add("Call 3-4pm", at(2026, 7, 19));
        assert_eq!(q.start.as_deref(), Some("2026-07-19T15:00:00"));
        assert_eq!(q.duration, "PT1H");
        assert_eq!(q.title, "Call");
    }

    #[test]
    fn iso_date_all_day() {
        let q = parse_quick_add("Conference on 2026-09-01", at(2026, 7, 19));
        assert_eq!(q.title, "Conference");
        assert_eq!(q.start.as_deref(), Some("2026-09-01"));
        assert!(q.all_day);
        assert_eq!(q.duration, "P1D");
    }

    #[test]
    fn weekday_next_week() {
        // 2026-07-19 is Sunday; "monday" → 2026-07-20.
        let q = parse_quick_add("Standup monday 9:30", at(2026, 7, 19));
        assert_eq!(q.start.as_deref(), Some("2026-07-20T09:30:00"));
        assert_eq!(q.title, "Standup");
    }

    #[test]
    fn month_name_day() {
        let q = parse_quick_add("Dentist jul 25 at 10am", at(2026, 7, 19));
        assert_eq!(q.start.as_deref(), Some("2026-07-25T10:00:00"));
        assert_eq!(q.title, "Dentist");
    }

    #[test]
    fn bare_number_is_not_a_time() {
        let q = parse_quick_add("Buy 5 apples tomorrow", at(2026, 7, 19));
        assert_eq!(q.title, "Buy 5 apples");
        assert!(q.all_day);
    }

    #[test]
    fn tonight_defaults_to_evening() {
        let q = parse_quick_add("Movie tonight", at(2026, 7, 19));
        assert_eq!(q.start.as_deref(), Some("2026-07-19T19:00:00"));
        assert_eq!(q.title, "Movie");
    }

    #[test]
    fn no_datetime_leaves_title_only() {
        let q = parse_quick_add("Think about roadmap", at(2026, 7, 19));
        assert_eq!(q.title, "Think about roadmap");
        assert_eq!(q.start, None);
    }
}
