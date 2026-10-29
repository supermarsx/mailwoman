//! iTIP `METHOD` framing (RFC 5546) for iMIP scheduling messages.
//!
//! Hand-rolled over the `icalendar` component model (plan §1.2/§2.6): an
//! outbound organiser `REQUEST` / attendee `REPLY` / `COUNTER` / organiser
//! `CANCEL` is the event component wrapped in a `VCALENDAR` carrying a top-level
//! `METHOD`. Inbound, `parse_itip` reads the `METHOD` and the event projection —
//! the mail-ingest invite-detection hook the engine registers.

use icalendar::parser::{Property as PProperty, read_calendar};
use serde_json::Value;

use crate::ical::{self, ParsedIcal};
use crate::{IcsError, Result};

/// The iTIP scheduling method carried by a `text/calendar; method=…` part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ItipMethod {
    Request,
    Reply,
    Counter,
    Cancel,
}

impl ItipMethod {
    fn as_ical(self) -> &'static str {
        match self {
            ItipMethod::Request => "REQUEST",
            ItipMethod::Reply => "REPLY",
            ItipMethod::Counter => "COUNTER",
            ItipMethod::Cancel => "CANCEL",
        }
    }

    /// The method a `METHOD` value names; `None` for PUBLISH, an unknown
    /// method, or none.
    pub fn from_name(s: &str) -> Option<Self> {
        Self::from_ical(s)
    }

    fn from_ical(s: &str) -> Option<Self> {
        match s.to_ascii_uppercase().as_str() {
            "REQUEST" => Some(ItipMethod::Request),
            "REPLY" => Some(ItipMethod::Reply),
            "COUNTER" => Some(ItipMethod::Counter),
            "CANCEL" => Some(ItipMethod::Cancel),
            _ => None,
        }
    }
}

/// The largest calendar part read from a received message (1 MiB). An
/// invitation is a few kilobytes; a part larger than this is refused unread.
pub const MAX_CALENDAR_BYTES: usize = 1 << 20;
/// The most VEVENT/VTODO components read from one received calendar part.
pub const MAX_CALENDAR_ITEMS: usize = 100;

/// A calendar part read from a received message: its `METHOD` (upper-cased,
/// any token, `PUBLISH` included, or `None` for a plain calendar file) and
/// every VEVENT/VTODO in it. Each event projection carries `itipDtstamp` (the
/// DTSTAMP in UTC basic form) when it has one.
#[derive(Debug, Clone)]
pub struct CalendarMessage {
    pub method: Option<String>,
    pub items: Vec<ParsedIcal>,
}

/// Read a received calendar part, bounded by [`MAX_CALENDAR_BYTES`] and
/// [`MAX_CALENDAR_ITEMS`]. The content is untrusted: nothing in it is fetched
/// (`ATTACH`/`URL` values stay text) and VTIMEZONE rules are not evaluated (a
/// TZID resolves to a tz-database zone, see `dt::resolve_tzid`).
pub fn read_calendar_message(bytes: &[u8]) -> Result<CalendarMessage> {
    if bytes.len() > MAX_CALENDAR_BYTES {
        return Err(IcsError::Itip(format!(
            "the calendar part is larger than {MAX_CALENDAR_BYTES} bytes"
        )));
    }
    let text = ical::unfold(&String::from_utf8_lossy(bytes));
    let components = text
        .lines()
        .map(|l| l.trim_end_matches('\r'))
        .filter(|l| l.eq_ignore_ascii_case("BEGIN:VEVENT") || l.eq_ignore_ascii_case("BEGIN:VTODO"))
        .count();
    if components > MAX_CALENDAR_ITEMS {
        return Err(IcsError::Itip(format!(
            "the calendar part holds more than {MAX_CALENDAR_ITEMS} components"
        )));
    }
    let cal = read_calendar(&text).map_err(IcsError::Ical)?;
    let method = cal
        .properties
        .iter()
        .find(|p| p.name.as_str().eq_ignore_ascii_case("METHOD"))
        .map(|p| p.val.as_str().trim().to_ascii_uppercase())
        .filter(|m| !m.is_empty());
    // The DTSTAMP of each master component (no RECURRENCE-ID), by UID.
    let stamps: std::collections::HashMap<String, String> = cal
        .components
        .iter()
        .filter(|c| ical::find(c, "RECURRENCE-ID").is_none())
        .filter_map(|c| {
            let uid = ical::find(c, "UID")?.val.as_str().to_string();
            let stamp = ical::find(c, "DTSTAMP")?.val.as_str().trim().to_string();
            utc_basic(&stamp).then_some((uid, stamp))
        })
        .collect();
    let mut items = ical::parse_ical(text.as_bytes())?;
    for item in &mut items {
        if item.component != "VEVENT" {
            continue;
        }
        let uid = item.json.get("uid").and_then(Value::as_str).unwrap_or("");
        if let Some(stamp) = stamps.get(uid) {
            item.json["itipDtstamp"] = Value::String(stamp.clone());
        }
    }
    Ok(CalendarMessage { method, items })
}

/// `20261020T090000Z`: the one DTSTAMP form compared for freshness.
fn utc_basic(v: &str) -> bool {
    v.len() == 16
        && v.ends_with('Z')
        && v.as_bytes()[8] == b'T'
        && v[..8].bytes().all(|b| b.is_ascii_digit())
        && v[9..15].bytes().all(|b| b.is_ascii_digit())
}

fn method_prop(method: &str) -> PProperty<'static> {
    PProperty {
        name: "METHOD".to_string().into(),
        val: method.to_string().into(),
        params: vec![],
    }
}

fn now_stamp() -> String {
    chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

/// Build an iTIP `text/calendar` payload framing `event_json` with `method`.
/// The component carries a DTSTAMP of now (RFC 5545 requires one, and RFC 5546
/// §2.1.5 orders messages of one SEQUENCE by it).
pub fn build_itip(event_json: &Value, method: ItipMethod) -> Result<String> {
    let mut component = ical::to_component(event_json);
    component
        .properties
        .push(ical::prop("DTSTAMP", now_stamp()));
    Ok(ical::wrap_calendar(
        vec![component],
        vec![method_prop(method.as_ical())],
    ))
}

/// The answer an attendee gives in a REPLY (RFC 5546 §3.2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplyStatus {
    Accepted,
    Tentative,
    Declined,
}

impl ReplyStatus {
    /// The JSCalendar `participationStatus` value.
    pub fn as_json(self) -> &'static str {
        match self {
            ReplyStatus::Accepted => "accepted",
            ReplyStatus::Tentative => "tentative",
            ReplyStatus::Declined => "declined",
        }
    }
}

/// Build the iTIP REPLY that `attendee` sends for `event_json` (RFC 5546
/// §3.2.3): the ORGANIZER, that one ATTENDEE with its PARTSTAT, the
/// organizer's UID and SEQUENCE unchanged (an attendee never bumps it), a
/// DTSTAMP of now and an optional COMMENT. Other attendees, alarms and
/// per-instance overrides are left out. `Err` when the event has no organizer
/// or does not list `attendee`.
pub fn build_itip_reply(
    event_json: &Value,
    attendee: &str,
    status: ReplyStatus,
    comment: Option<&str>,
) -> Result<String> {
    let parts = event_json
        .get("participants")
        .and_then(Value::as_object)
        .ok_or_else(|| IcsError::Itip("the event has no participants".into()))?;
    let mut kept = serde_json::Map::new();
    let mut have_organizer = false;
    let mut have_attendee = false;
    for (key, p) in parts {
        let email = p
            .get("email")
            .and_then(Value::as_str)
            .filter(|e| !e.is_empty())
            .unwrap_or(key)
            .to_string();
        let mut entry = p.clone();
        if let Some(obj) = entry.as_object_mut() {
            obj.insert("email".into(), Value::String(email.clone()));
        }
        if p.get("role").and_then(Value::as_str) == Some("organizer") {
            have_organizer = true;
            kept.insert(key.clone(), entry);
        } else if !have_attendee && (same_address(key, attendee) || same_address(&email, attendee))
        {
            have_attendee = true;
            if let Some(obj) = entry.as_object_mut() {
                obj.insert(
                    "participationStatus".into(),
                    Value::String(status.as_json().into()),
                );
                obj.insert("expectReply".into(), Value::Bool(false));
            }
            kept.insert(key.clone(), entry);
        }
    }
    if !have_organizer {
        return Err(IcsError::Itip("the event has no organizer".into()));
    }
    if !have_attendee {
        return Err(IcsError::Itip(
            "the event does not list this attendee".into(),
        ));
    }
    let mut ev = event_json.clone();
    ev["participants"] = Value::Object(kept);
    ev["alerts"] = Value::Object(serde_json::Map::new());
    ev["recurrenceOverrides"] = Value::Object(serde_json::Map::new());
    let mut component = ical::to_component(&ev);
    component
        .properties
        .push(ical::prop("DTSTAMP", now_stamp()));
    if let Some(c) = comment.map(str::trim).filter(|c| !c.is_empty()) {
        // One TEXT value, bounded: a comment is a sentence, not a document.
        let c: String = c
            .chars()
            .filter(|ch| *ch == '\n' || !ch.is_control())
            .take(2000)
            .collect();
        component
            .properties
            .push(ical::prop("COMMENT", ical::esc(&c)));
    }
    Ok(ical::wrap_calendar(
        vec![component],
        vec![method_prop("REPLY")],
    ))
}

/// Two calendar addresses are the same mailbox: case and `mailto:` aside.
pub fn same_address(a: &str, b: &str) -> bool {
    let norm = |s: &str| {
        let s = s.trim();
        let s = s
            .strip_prefix("mailto:")
            .or_else(|| s.strip_prefix("MAILTO:"))
            .unwrap_or(s);
        s.to_ascii_lowercase()
    };
    !a.trim().is_empty() && norm(a) == norm(b)
}

/// Parse an inbound `text/calendar` part → its method + the event projection.
pub fn parse_itip(bytes: &[u8]) -> Result<(ItipMethod, ParsedIcal)> {
    let text = String::from_utf8_lossy(bytes);
    let cal = read_calendar(&text).map_err(IcsError::Ical)?;
    let method = cal
        .properties
        .iter()
        .find(|p| p.name.as_str().eq_ignore_ascii_case("METHOD"))
        .and_then(|p| ItipMethod::from_ical(p.val.as_str()))
        .ok_or_else(|| IcsError::Itip("missing or unknown METHOD".into()))?;

    let parsed = ical::parse_ical(bytes)?
        .into_iter()
        .next()
        .ok_or_else(|| IcsError::Itip("no schedulable component in iTIP message".into()))?;
    Ok((method, parsed))
}
