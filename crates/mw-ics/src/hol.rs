//! Outlook `.hol` holiday-pack parse → all-day VEVENT projections.
//!
//! The `.hol` format is an INI-like text file: section headers `[Region] <n>`
//! introduce `<n>` `Description,YYYY/MM/DD` holiday lines (plan §11, the bundled
//! / subscribable holiday feed). We parse tolerantly — inside a section, any
//! line with a trailing parseable date is a holiday and any other line is
//! skipped — and emit one all-day event per holiday.
//!
//! The section header is what tells a holiday pack from other text: a
//! comma-separated file whose last column is a date has the same lines. A text
//! with no section header, or with no holiday line inside a section, is not a
//! holiday pack and is an error, not a pack of zero holidays.

use serde_json::{Value, json};

use crate::ical::{ParsedIcal, to_component, wrap_calendar};
use crate::{IcsError, Result};

fn parse_hol_date(raw: &str) -> Option<String> {
    let t = raw.trim();
    let digits: String = t.chars().filter(|c| c.is_ascii_digit()).collect();
    // Accept YYYY/MM/DD, YYYY-MM-DD or compact YYYYMMDD (8 digits total).
    if t.contains('/') || t.contains('-') {
        let parts: Vec<&str> = t.split(['/', '-']).collect();
        if parts.len() == 3 {
            let y: u32 = parts[0].trim().parse().ok()?;
            let m: u32 = parts[1].trim().parse().ok()?;
            let d: u32 = parts[2].trim().parse().ok()?;
            if (1..=12).contains(&m) && (1..=31).contains(&d) && y > 1000 {
                return Some(format!("{y:04}-{m:02}-{d:02}"));
            }
        }
        return None;
    }
    if digits.len() == 8 {
        let y = &digits[0..4];
        let m = &digits[4..6];
        let d = &digits[6..8];
        return Some(format!("{y}-{m}-{d}"));
    }
    None
}

fn slug(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

/// A `[Region] <n>` section header: a line opening with a bracketed, non-empty
/// name. The count after it is not read.
fn is_section_header(line: &str) -> bool {
    line.strip_prefix('[')
        .and_then(|rest| rest.split_once(']'))
        .is_some_and(|(name, _)| !name.trim().is_empty())
}

/// Parse a `.hol` holiday pack into all-day VEVENT projections.
///
/// `Err(IcsError::Hol)` when the text is not a holiday pack: it has no section
/// header, or no section holds a holiday line. Lines before the first section
/// header are not read.
pub fn parse_hol(bytes: &[u8]) -> Result<Vec<ParsedIcal>> {
    let text = String::from_utf8_lossy(bytes);
    // `trim` does not take a UTF-8 byte order mark for whitespace, and one
    // would hide the first section header.
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let mut in_section = false;
    let mut out = vec![];
    for raw in text.lines() {
        let l = raw.trim();
        if is_section_header(l) {
            in_section = true;
            continue;
        }
        if !in_section || l.is_empty() || l.starts_with('[') || l.starts_with(';') {
            continue;
        }
        let Some((desc, date_part)) = l.rsplit_once(',') else {
            continue;
        };
        let Some(date) = parse_hol_date(date_part) else {
            continue;
        };
        let title = desc.trim().to_string();
        let uid = format!("{}-{}@holidays.mailwoman", date, slug(&title));
        let json = json!({
            "id": uid,
            "calendarId": "",
            "uid": uid,
            "title": title,
            "description": "",
            "locations": [],
            "start": date,
            "timeZone": Value::Null,
            "duration": "P1D",
            "showWithoutTime": true,
            "recurrenceRules": [],
            "recurrenceOverrides": {},
            "excludedRecurrenceDates": [],
            "status": "confirmed",
            "priority": 0,
            "freeBusyStatus": "free",
            "participants": {},
            "alerts": {},
            "sequence": 0,
            "etag": Value::Null,
        });
        let ical_raw = wrap_calendar(vec![to_component(&json)], vec![]);
        out.push(ParsedIcal {
            ical_raw,
            json,
            component: "VEVENT".into(),
        });
    }
    if !in_section {
        return Err(IcsError::Hol("no [section] line".into()));
    }
    if out.is_empty() {
        return Err(IcsError::Hol("no holiday lines".into()));
    }
    Ok(out)
}

/// Serialize all-day event projections into an Outlook `.hol` holiday pack.
///
/// The inverse of [`parse_hol`]: emits a single `[Holidays] <count>` section of
/// `Description,YYYY/MM/DD` lines (CRLF, the format Outlook expects). Only the
/// event's `title` and `start` date are used — the time and all other fields are
/// not representable in `.hol`. Titles are flattened (commas/newlines → spaces)
/// so each holiday stays on one parseable line.
pub fn write_hol(events: &[Value]) -> Result<String> {
    let mut body = Vec::new();
    for ev in events {
        let title = ev
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim();
        let start = ev.get("start").and_then(Value::as_str).unwrap_or_default();
        if title.is_empty() || start.is_empty() {
            continue;
        }
        // Take the date portion (drop any time) and switch to slash form.
        let date = start.split('T').next().unwrap_or(start);
        let parts: Vec<&str> = date.split('-').collect();
        if parts.len() != 3 {
            continue;
        }
        let safe_title = title.replace([',', '\r', '\n'], " ");
        body.push(format!(
            "{safe_title},{}/{}/{}",
            parts[0], parts[1], parts[2]
        ));
    }
    let mut lines = vec![format!("[Holidays] {}", body.len())];
    lines.extend(body);
    Ok(format!("{}\r\n", lines.join("\r\n")))
}
