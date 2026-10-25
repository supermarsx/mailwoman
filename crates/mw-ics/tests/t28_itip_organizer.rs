//! 26.20 t28-e21 (f35): an organizer who is also listed as an attendee stays
//! the organizer.
//!
//! Participants are keyed by address, and RFC 5545 lets the ORGANIZER appear as
//! an ATTENDEE too (most clients write it that way, with the organizer's own
//! PARTSTAT). The ATTENDEE line used to replace the organizer's entry, so the
//! projection had no organizer: the engine could not tell who may update or
//! cancel the event, and a reply went to whichever participant came first.

use serde_json::Value;

const ICS: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Test//EN\r\nBEGIN:VEVENT\r\n\
    UID:org-also-attendee@example.com\r\nDTSTAMP:20261020T090000Z\r\n\
    DTSTART:20261110T100000Z\r\nDURATION:PT1H\r\nSUMMARY:Planning\r\n\
    ORGANIZER;CN=Org:mailto:org@example.com\r\n\
    ATTENDEE;PARTSTAT=ACCEPTED:mailto:org@example.com\r\n\
    ATTENDEE;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:me@example.org\r\n\
    END:VEVENT\r\nEND:VCALENDAR\r\n";

fn participants(ics: &str) -> serde_json::Map<String, Value> {
    let parsed = mw_ics::parse_ical(ics.as_bytes()).unwrap();
    parsed[0].json["participants"].as_object().unwrap().clone()
}

#[test]
fn the_organizer_listed_as_an_attendee_keeps_the_organizer_role() {
    let p = participants(ICS);
    assert_eq!(p["org@example.com"]["role"], "organizer", "{p:#?}");
    assert_eq!(p["org@example.com"]["name"], "Org");
    assert_eq!(p["me@example.org"]["role"], "attendee");
}

#[test]
fn the_organizer_survives_a_round_trip() {
    let first = mw_ics::parse_ical(ICS.as_bytes()).unwrap();
    let emitted = mw_ics::emit_ical(&first[0].json).unwrap();
    assert!(
        emitted.contains("ORGANIZER;CN=Org:mailto:org@example.com"),
        "{emitted}"
    );
    let p = participants(&emitted);
    assert_eq!(p["org@example.com"]["role"], "organizer", "{p:#?}");
}
