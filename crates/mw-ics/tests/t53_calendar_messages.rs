//! t53: received calendar parts as the reader's event card reads them.
//!
//! `read_calendar_message` is what the engine applies to a `text/calendar` part
//! or an `.ics` attachment: every method (PUBLISH and a plain file included),
//! every component, bounded in size and count. `build_itip_reply` is the REPLY
//! an attendee sends: one ATTENDEE, the organizer's SEQUENCE, a DTSTAMP.

use mw_ics::{
    MAX_CALENDAR_BYTES, MAX_CALENDAR_ITEMS, ReplyStatus, build_itip_reply, parse_itip,
    read_calendar_message,
};
use serde_json::Value;

fn cal(method: Option<&str>, body: &str) -> String {
    let method = method
        .map(|m| format!("METHOD:{m}\r\n"))
        .unwrap_or_default();
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Test//EN\r\n{method}{body}END:VCALENDAR\r\n"
    )
}

const INVITE: &str = "BEGIN:VEVENT\r\nUID:weekly@example.com\r\nSEQUENCE:2\r\n\
    DTSTAMP:20261020T090000Z\r\nDTSTART;TZID=W. Europe Standard Time:20261103T100000\r\n\
    DTEND;TZID=W. Europe Standard Time:20261103T110000\r\nSUMMARY:Weekly sync\r\n\
    LOCATION:https://teams.microsoft.com/l/meetup-join/abc\r\n\
    DESCRIPTION:Agenda:\\n1. Status\r\n\
    RRULE:FREQ=WEEKLY;BYDAY=TU;UNTIL=20261203T235959Z\r\n\
    EXDATE;TZID=W. Europe Standard Time:20261110T100000\r\n\
    ORGANIZER;CN=Olga Organizer:mailto:olga@example.com\r\n\
    ATTENDEE;CN=Olga Organizer;PARTSTAT=ACCEPTED:mailto:olga@example.com\r\n\
    ATTENDEE;CN=Me;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:me@example.org\r\n\
    ATTENDEE;CN=Pat;PARTSTAT=TENTATIVE:mailto:pat@example.net\r\n\
    BEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:-PT15M\r\nEND:VALARM\r\n\
    END:VEVENT\r\n\
    BEGIN:VEVENT\r\nUID:weekly@example.com\r\nSEQUENCE:2\r\nDTSTAMP:20261020T090000Z\r\n\
    RECURRENCE-ID;TZID=W. Europe Standard Time:20261117T100000\r\n\
    DTSTART;TZID=W. Europe Standard Time:20261117T140000\r\n\
    DTEND;TZID=W. Europe Standard Time:20261117T150000\r\nSUMMARY:Weekly sync (moved)\r\n\
    ORGANIZER:mailto:olga@example.com\r\nEND:VEVENT\r\n";

const VTIMEZONE: &str = "BEGIN:VTIMEZONE\r\nTZID:W. Europe Standard Time\r\n\
    BEGIN:STANDARD\r\nDTSTART:16010101T030000\r\nTZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\n\
    RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10\r\nEND:STANDARD\r\n\
    BEGIN:DAYLIGHT\r\nDTSTART:16010101T020000\r\nTZOFFSETFROM:+0100\r\nTZOFFSETTO:+0200\r\n\
    RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=3\r\nEND:DAYLIGHT\r\nEND:VTIMEZONE\r\n";

fn first(method: Option<&str>, body: &str) -> (Option<String>, Value) {
    let msg = read_calendar_message(cal(method, body).as_bytes()).unwrap();
    (msg.method, msg.items[0].json.clone())
}

#[test]
fn every_method_is_read_and_named() {
    for m in ["REQUEST", "REPLY", "CANCEL", "COUNTER", "PUBLISH"] {
        let (method, ev) = first(Some(m), INVITE);
        assert_eq!(method.as_deref(), Some(m));
        assert_eq!(ev["title"], "Weekly sync");
    }
    let (method, _) = first(Some("publish"), INVITE);
    assert_eq!(
        method.as_deref(),
        Some("PUBLISH"),
        "METHOD is case-insensitive"
    );
    let (method, _) = first(None, INVITE);
    assert_eq!(method, None, "a plain calendar file has no method");
    // parse_itip still refuses PUBLISH: it is not a scheduling message.
    assert!(parse_itip(cal(Some("PUBLISH"), INVITE).as_bytes()).is_err());
}

#[test]
fn recurrence_overrides_exdates_and_the_dtstamp_are_projected() {
    let (_, ev) = first(Some("REQUEST"), INVITE);
    assert_eq!(
        ev["recurrenceRules"][0]["rrule"],
        "FREQ=WEEKLY;BYDAY=TU;UNTIL=20261203T235959Z"
    );
    assert_eq!(ev["excludedRecurrenceDates"][0], "2026-11-10T10:00:00");
    let overrides = ev["recurrenceOverrides"].as_object().unwrap();
    assert_eq!(overrides.len(), 1, "{overrides:?}");
    let moved = overrides.values().next().unwrap();
    assert_eq!(moved["title"], "Weekly sync (moved)");
    assert_eq!(ev["itipDtstamp"], "20261020T090000Z");
    assert_eq!(ev["sequence"], 2);
    assert_eq!(ev["description"], "Agenda:\n1. Status");
}

#[test]
fn a_windows_tzid_with_its_vtimezone_resolves_to_an_iana_zone() {
    let (_, ev) = first(Some("REQUEST"), &format!("{VTIMEZONE}{INVITE}"));
    assert_eq!(ev["timeZone"], "Europe/Berlin");
    assert_eq!(ev["start"], "2026-11-03T10:00:00");
    assert_eq!(ev["duration"], "PT1H");
    // And so the event expands (it used to fail: "unknown time zone").
    let instances =
        mw_ics::expand_recurrence(&ev, "2026-11-01T00:00:00Z", "2026-12-31T00:00:00Z").unwrap();
    // Tuesdays 3, 17 (moved), 24 Nov and 1 Dec; 10 Nov is excluded.
    assert_eq!(instances.len(), 4, "{instances:?}");
}

#[test]
fn an_all_day_event_is_shown_without_time() {
    let body = "BEGIN:VEVENT\r\nUID:day@example.com\r\nDTSTAMP:20261020T090000Z\r\n\
        DTSTART;VALUE=DATE:20261224\r\nDTEND;VALUE=DATE:20261226\r\nSUMMARY:Holidays\r\nEND:VEVENT\r\n";
    let (_, ev) = first(Some("PUBLISH"), body);
    assert_eq!(ev["showWithoutTime"], true);
    assert_eq!(ev["start"], "2026-12-24");
    assert_eq!(ev["duration"], "P2D");
}

#[test]
fn a_file_with_several_events_and_a_task_yields_each() {
    let mut body = String::new();
    for n in 1..=3 {
        body.push_str(&format!(
            "BEGIN:VEVENT\r\nUID:e{n}@example.com\r\nDTSTAMP:20261020T090000Z\r\n\
             DTSTART:2026110{n}T090000Z\r\nDURATION:PT30M\r\nSUMMARY:Event {n}\r\nEND:VEVENT\r\n"
        ));
    }
    body.push_str("BEGIN:VTODO\r\nUID:t1@example.com\r\nSUMMARY:Bring slides\r\nDUE:20261101T170000Z\r\nEND:VTODO\r\n");
    let msg = read_calendar_message(cal(None, &body).as_bytes()).unwrap();
    let kinds: Vec<&str> = msg.items.iter().map(|i| i.component.as_str()).collect();
    assert_eq!(kinds, ["VEVENT", "VEVENT", "VEVENT", "VTODO"]);
    assert_eq!(msg.items[2].json["title"], "Event 3");
    assert_eq!(msg.items[3].json["title"], "Bring slides");
    assert!(msg.items[3].json.get("itipDtstamp").is_none());
}

#[test]
fn oversized_parts_and_too_many_components_are_refused() {
    let big = format!(
        "{}{}",
        cal(Some("REQUEST"), INVITE),
        " ".repeat(MAX_CALENDAR_BYTES)
    );
    assert!(read_calendar_message(big.as_bytes()).is_err());
    let mut body = String::new();
    for n in 0..=MAX_CALENDAR_ITEMS {
        body.push_str(&format!(
            "BEGIN:VEVENT\r\nUID:x{n}\r\nDTSTART:20261101T090000Z\r\nEND:VEVENT\r\n"
        ));
    }
    let err = read_calendar_message(cal(None, &body).as_bytes()).unwrap_err();
    assert!(err.to_string().contains("more than"), "{err}");
}

#[test]
fn a_reply_carries_only_the_replying_attendee_and_the_organizers_sequence() {
    let (_, ev) = first(Some("REQUEST"), INVITE);
    let reply = build_itip_reply(
        &ev,
        "ME@example.org",
        ReplyStatus::Accepted,
        Some("See you there; thanks"),
    )
    .unwrap();
    assert!(reply.contains("METHOD:REPLY"), "{reply}");
    assert!(
        reply.contains("SEQUENCE:2"),
        "the attendee never bumps SEQUENCE: {reply}"
    );
    assert!(reply.contains("DTSTAMP:"), "{reply}");
    assert!(reply.contains("COMMENT:See you there\\; thanks"), "{reply}");
    assert!(
        !reply.contains("pat@example.net"),
        "other attendees are left out: {reply}"
    );
    assert!(!reply.contains("BEGIN:VALARM"), "{reply}");
    assert!(!reply.contains("RECURRENCE-ID"), "{reply}");
    // It parses back as a REPLY whose one attendee accepted.
    let (method, parsed) = parse_itip(reply.as_bytes()).unwrap();
    assert_eq!(method, mw_ics::ItipMethod::Reply);
    let parts = parsed.json["participants"].as_object().unwrap();
    assert_eq!(parts.len(), 2, "{parts:#?}");
    assert_eq!(parts["me@example.org"]["participationStatus"], "accepted");
    assert_eq!(parts["olga@example.com"]["role"], "organizer");
}

#[test]
fn a_reply_needs_an_organizer_and_the_attendee() {
    let (_, ev) = first(Some("REQUEST"), INVITE);
    assert!(build_itip_reply(&ev, "stranger@example.org", ReplyStatus::Declined, None).is_err());
    let mut orphan = ev.clone();
    orphan["participants"]
        .as_object_mut()
        .unwrap()
        .remove("olga@example.com");
    assert!(build_itip_reply(&orphan, "me@example.org", ReplyStatus::Tentative, None).is_err());
}
