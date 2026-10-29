//! Calendar content of a stored message (t78): the one detector behind the
//! Events view, its `hasCalendar` filter and its row hint.
//!
//! A message carries calendar content when its first calendar part — a
//! `text/calendar` or `application/ics` part, or a part whose file name ends in
//! `.ics` whatever type it was sent as (Outlook and some gateways send
//! `application/octet-stream`) — reads as a calendar with at least one VEVENT
//! under the same bounded reader the event card uses
//! (`mw_ics::read_calendar_message`, t53). So a file that only carries the
//! name, or a body that only mentions an `.ics`, is not an event; the card
//! would refuse it too.
//!
//! It runs on the raw message at ingest and at a search-index rebuild, never on
//! a read: the result is kept in the message's index document
//! ([`mw_search::CalendarHint`]), which `Email/query` filters on and `Email/get`
//! reads (`mailwomanCalendar`) without opening the message again.

use mail_parser::{MessageParser, MimeHeaders, PartType};
use serde_json::Value;

/// The bytes of the message's first calendar part (t53's detection): a
/// `text/calendar` or `application/ics` part, or a part named `*.ics`.
pub(crate) fn first_calendar_part(raw: &[u8]) -> Option<Vec<u8>> {
    let message = MessageParser::default().parse(raw)?;
    message.parts.iter().find_map(|part| {
        let kind = part
            .content_type()
            .map(|ct| format!("{}/{}", ct.ctype(), ct.subtype().unwrap_or("")).to_ascii_lowercase())
            .unwrap_or_default();
        let named_ics = part
            .attachment_name()
            .is_some_and(|n| n.trim().to_ascii_lowercase().ends_with(".ics"));
        if kind != "text/calendar" && kind != "application/ics" && !named_ics {
            return None;
        }
        match &part.body {
            PartType::Text(t) => Some(t.as_bytes().to_vec()),
            PartType::Binary(b) | PartType::InlineBinary(b) => Some(b.to_vec()),
            _ => None,
        }
    })
}

/// The calendar summary of a raw message, or `None` when it carries no
/// readable calendar part with an event.
pub fn calendar_hint(raw: &[u8]) -> Option<mw_search::CalendarHint> {
    let part = first_calendar_part(raw)?;
    let msg = mw_ics::read_calendar_message(&part).ok()?;
    let events: Vec<&Value> = msg
        .items
        .iter()
        .filter(|i| i.component == "VEVENT")
        .map(|i| &i.json)
        .collect();
    let first = *events.first()?;
    let text = |key: &str| {
        first
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    let spans: Vec<(i64, i64)> = events
        .iter()
        .filter_map(|e| mw_ics::event_span_utc(e))
        .collect();
    let time_zone = first
        .get("timeZone")
        .and_then(Value::as_str)
        .filter(|z| !z.is_empty())
        .map(str::to_string);
    Some(mw_search::CalendarHint {
        method: msg.method,
        sequence: first.get("sequence").and_then(Value::as_i64).unwrap_or(0),
        title: text("title"),
        start: text("start"),
        time_zone,
        show_without_time: first
            .get("showWithoutTime")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        status: text("status"),
        start_utc: mw_ics::event_span_utc(first).map(|(s, _)| s),
        end_utc: spans.iter().map(|(_, e)| *e).max(),
        events: events.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invite(method: Option<&str>, content_type: &str, name: Option<&str>) -> String {
        let method_line = method
            .map(|m| format!("METHOD:{m}\r\n"))
            .unwrap_or_default();
        let ct_method = method.map(|m| format!("; method={m}")).unwrap_or_default();
        let disposition = name
            .map(|n| format!("Content-Disposition: attachment; filename={n}\r\n"))
            .unwrap_or_default();
        let ct_name = name.map(|n| format!("; name={n}")).unwrap_or_default();
        format!(
            "From: Olga <olga@example.com>\r\nTo: me@example.org\r\nSubject: Review\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=b\r\n\r\n--b\r\nContent-Type: text/plain\r\n\r\nSee you there.\r\n--b\r\nContent-Type: {content_type}{ct_method}{ct_name}\r\n{disposition}\r\nBEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t78//EN\r\n{method_line}BEGIN:VEVENT\r\nUID:t78-1@example.com\r\nDTSTAMP:20261020T090000Z\r\nDTSTART;TZID=Europe/Lisbon:20261103T100000\r\nDTEND;TZID=Europe/Lisbon:20261103T110000\r\nSEQUENCE:2\r\nSUMMARY:Design review\r\nORGANIZER:mailto:olga@example.com\r\nATTENDEE;PARTSTAT=NEEDS-ACTION:mailto:me@example.org\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n--b--\r\n"
        )
    }

    #[test]
    fn every_method_is_read_from_a_text_calendar_part() {
        for method in ["REQUEST", "REPLY", "CANCEL", "COUNTER", "PUBLISH"] {
            let raw = invite(Some(method), "text/calendar", None);
            let hint = calendar_hint(raw.as_bytes()).unwrap_or_else(|| panic!("{method}"));
            assert_eq!(hint.method.as_deref(), Some(method));
            assert_eq!(hint.title, "Design review");
            assert_eq!(hint.start, "2026-11-03T10:00:00");
            assert_eq!(hint.time_zone.as_deref(), Some("Europe/Lisbon"));
            assert_eq!(hint.sequence, 2);
            assert_eq!(hint.events, 1);
            // 10:00 Lisbon (WET, UTC+0 in November) = 10:00Z; ends an hour later.
            assert_eq!(hint.start_utc, Some(1_793_700_000));
            assert_eq!(hint.end_utc, Some(1_793_700_000 + 3600));
        }
    }

    #[test]
    fn application_ics_and_a_plain_calendar_file_count() {
        let raw = invite(None, "application/ics", Some("event.ics"));
        let hint = calendar_hint(raw.as_bytes()).expect("application/ics");
        assert_eq!(hint.method, None);
    }

    #[test]
    fn an_ics_sent_as_octet_stream_is_read_by_its_name() {
        let raw = invite(
            Some("REQUEST"),
            "application/octet-stream",
            Some("invite.ics"),
        );
        let hint = calendar_hint(raw.as_bytes()).expect("octet-stream .ics");
        assert_eq!(hint.method.as_deref(), Some("REQUEST"));
    }

    #[test]
    fn an_ics_mentioned_only_in_text_is_not_an_event() {
        let raw = "From: a@example.com\r\nTo: b@example.org\r\nSubject: the file\r\nContent-Type: text/plain\r\n\r\nI attached invite.ics yesterday; BEGIN:VCALENDAR is how it starts.\r\n";
        assert!(calendar_hint(raw.as_bytes()).is_none());
        let named_but_not_calendar = "From: a@example.com\r\nTo: b@example.org\r\nSubject: x\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=b\r\n\r\n--b\r\nContent-Type: text/plain\r\n\r\nsee invite.ics\r\n--b\r\nContent-Type: application/octet-stream; name=invite.ics\r\nContent-Disposition: attachment; filename=invite.ics\r\n\r\nnot a calendar at all\r\n--b--\r\n";
        assert!(calendar_hint(named_but_not_calendar.as_bytes()).is_none());
    }

    #[test]
    fn a_calendar_with_only_a_task_is_not_an_event() {
        let raw = "From: a@example.com\r\nTo: b@example.org\r\nSubject: t\r\nContent-Type: text/calendar\r\n\r\nBEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VTODO\r\nUID:x\r\nSUMMARY:todo\r\nEND:VTODO\r\nEND:VCALENDAR\r\n";
        assert!(calendar_hint(raw.as_bytes()).is_none());
    }

    #[test]
    fn a_recurring_event_never_ends() {
        let raw = invite(Some("REQUEST"), "text/calendar", None)
            .replace("SEQUENCE:2\r\n", "SEQUENCE:0\r\nRRULE:FREQ=WEEKLY\r\n");
        let hint = calendar_hint(raw.as_bytes()).expect("recurring");
        assert_eq!(hint.end_utc, Some(i64::MAX));
        assert_eq!(hint.sequence, 0);
    }
}
