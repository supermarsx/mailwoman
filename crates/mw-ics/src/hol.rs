//! Outlook `.hol` holiday-pack parse → all-day VEVENT projections.
//!
//! The `.hol` format is an INI-like text file: section headers `[Region] <n>`
//! introduce `<n>` `Description,YYYY/MM/DD` holiday lines (plan §11, the bundled
//! / subscribable holiday feed). We parse tolerantly — any line with a trailing
//! parseable date is a holiday, section headers are skipped — and emit one
//! all-day event per holiday.

use serde_json::{Value, json};

use crate::Result;
use crate::ical::{ParsedIcal, to_component, wrap_calendar};

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

/// Parse a `.hol` holiday pack into all-day VEVENT projections.
pub fn parse_hol(bytes: &[u8]) -> Result<Vec<ParsedIcal>> {
    let text = String::from_utf8_lossy(bytes);
    let mut out = vec![];
    for raw in text.lines() {
        let l = raw.trim();
        if l.is_empty() || l.starts_with('[') || l.starts_with(';') {
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
