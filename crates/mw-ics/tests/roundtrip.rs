//! Acceptance tests for `mw-ics` (plan §3 e1): round-trip goldens for
//! events/tasks/contacts, the DST-boundary weekly recurrence gate (§1.12), iTIP
//! REPLY/COUNTER goldens, a `.hol` import, and a VFREEBUSY merge.

use mw_ics::{
    ItipMethod, aggregate_free_busy, build_itip, emit_ical, emit_vcard, expand_recurrence,
    parse_hol, parse_ical, parse_itip, parse_vcard,
};
use serde_json::{Value, json};

const EVENT_ICS: &str = include_str!("../../../fixtures/ics/event.ics");
const TASK_ICS: &str = include_str!("../../../fixtures/ics/task.ics");
const CONTACT_VCF: &str = include_str!("../../../fixtures/ics/contact.vcf");
const CONTACT3_VCF: &str = include_str!("../../../fixtures/ics/contact3.vcf");
const HOLIDAYS_HOL: &str = include_str!("../../../fixtures/ics/holidays.hol");

// ── round-trip goldens: parse → emit → parse yields an equal projection ──────

#[test]
fn event_roundtrip_projection_stable() {
    let p1 = parse_ical(EVENT_ICS.as_bytes()).unwrap();
    assert_eq!(p1.len(), 1);
    assert_eq!(p1[0].component, "VEVENT");

    let emitted = emit_ical(&p1[0].json).unwrap();
    let p2 = parse_ical(emitted.as_bytes()).unwrap();
    assert_eq!(p1[0].json, p2[0].json, "event projection must survive emit");
}

#[test]
fn event_projection_fields() {
    let p = parse_ical(EVENT_ICS.as_bytes()).unwrap();
    let e = &p[0].json;
    assert_eq!(e["uid"], "evt-1@mailwoman");
    assert_eq!(e["title"], "Team Sync");
    assert_eq!(e["start"], "2026-03-20T09:00:00");
    assert_eq!(e["timeZone"], "Europe/London");
    assert_eq!(e["duration"], "PT1H");
    assert_eq!(e["locations"][0]["name"], "Room 4");
    assert_eq!(e["recurrenceRules"][0]["rrule"], "FREQ=WEEKLY;COUNT=6");
    // organizer + attendee both projected
    assert_eq!(e["participants"]["alice@example.com"]["role"], "organizer");
    assert_eq!(e["participants"]["bob@example.com"]["expectReply"], true);
    // one display alarm
    assert_eq!(e["alerts"]["1"]["action"], "display");
    assert_eq!(e["alerts"]["1"]["trigger"]["offset"], "-PT15M");
}

#[test]
fn task_roundtrip_projection_stable() {
    let p1 = parse_ical(TASK_ICS.as_bytes()).unwrap();
    assert_eq!(p1[0].component, "VTODO");
    assert_eq!(p1[0].json["percentComplete"], 40);
    assert_eq!(p1[0].json["status"], "in-process");
    assert_eq!(p1[0].json["due"], "2026-03-25T17:00:00");

    let emitted = emit_ical(&p1[0].json).unwrap();
    let p2 = parse_ical(emitted.as_bytes()).unwrap();
    assert_eq!(p1[0].json, p2[0].json, "task projection must survive emit");
}

#[test]
fn contact_roundtrip_projection_stable() {
    let p1 = parse_vcard(CONTACT_VCF.as_bytes()).unwrap();
    assert_eq!(p1.len(), 1);
    let c = &p1[0].json;
    assert_eq!(c["name"]["surname"], "Jones");
    assert_eq!(c["name"]["given"], "Bob");
    assert_eq!(c["emails"][0]["context"], "work");
    assert_eq!(c["emails"][0]["pref"], 1);
    assert_eq!(c["phones"][0]["value"], "+15551234");
    assert_eq!(c["organizations"][0], "Acme;Engineering");
    assert_eq!(c["pgpKey"], "pgp-key-data");

    let emitted = emit_vcard(&p1[0].json).unwrap();
    let p2 = parse_vcard(emitted.as_bytes()).unwrap();
    assert_eq!(
        p1[0].json, p2[0].json,
        "contact projection must survive emit"
    );
}

#[test]
fn vcard3_import_tolerant() {
    // vCard 3.0 must parse via the tolerant path (plan: vCard 3/4).
    let p = parse_vcard(CONTACT3_VCF.as_bytes()).unwrap();
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].json["name"]["surname"], "Smith");
    assert_eq!(p[0].json["emails"][0]["value"], "carol@example.com");
}

// ── the §1.12 DST-boundary weekly recurrence gate ────────────────────────────

#[test]
fn rrule_weekly_across_dst_spring_forward() {
    // Weekly 09:00 Europe/London from 2026-03-20; London springs forward on
    // 2026-03-29 (GMT→BST). Wall-clock 09:00 is held, so the UTC instant moves
    // from 09:00Z (GMT weeks) to 08:00Z (BST weeks).
    let event = json!({
        "start": "2026-03-20T09:00:00",
        "timeZone": "Europe/London",
        "duration": "PT1H",
        "recurrenceRules": [{ "rrule": "FREQ=WEEKLY;COUNT=6" }],
        "excludedRecurrenceDates": [],
    });
    let inst = expand_recurrence(&event, "2026-03-01T00:00:00Z", "2026-05-01T00:00:00Z").unwrap();
    assert_eq!(inst.len(), 6);
    assert_eq!(inst[0].start_utc, "2026-03-20T09:00:00Z"); // GMT
    assert_eq!(inst[1].start_utc, "2026-03-27T09:00:00Z"); // GMT
    assert_eq!(inst[2].start_utc, "2026-04-03T08:00:00Z"); // BST — DST-shifted
    assert_eq!(inst[5].start_utc, "2026-04-24T08:00:00Z"); // BST
    // Durations stay nominal 1h in local wall time.
    assert_eq!(inst[2].end_utc, "2026-04-03T09:00:00Z");
}

#[test]
fn rrule_exdate_excluded() {
    let event = json!({
        "start": "2026-06-01T10:00:00",
        "timeZone": "UTC",
        "duration": "PT30M",
        "recurrenceRules": [{ "rrule": "FREQ=DAILY;COUNT=4" }],
        "excludedRecurrenceDates": ["2026-06-02T10:00:00"],
    });
    let inst = expand_recurrence(&event, "2026-06-01T00:00:00Z", "2026-06-10T00:00:00Z").unwrap();
    let starts: Vec<&str> = inst.iter().map(|i| i.start_utc.as_str()).collect();
    assert_eq!(
        starts,
        vec![
            "2026-06-01T10:00:00Z",
            "2026-06-03T10:00:00Z",
            "2026-06-04T10:00:00Z",
        ]
    );
}

// ── iTIP REPLY / COUNTER goldens ─────────────────────────────────────────────

fn invite_event() -> Value {
    json!({
        "uid": "mtg-9@mailwoman",
        "calendarId": "",
        "title": "Project Kickoff",
        "start": "2026-07-15T14:00:00",
        "timeZone": "UTC",
        "duration": "PT1H",
        "status": "confirmed",
        "freeBusyStatus": "busy",
        "sequence": 1,
        "participants": {
            "org@example.com": {
                "name": "Org", "email": "org@example.com", "role": "organizer",
                "participationStatus": "accepted", "expectReply": false
            },
            "me@example.com": {
                "name": "Me", "email": "me@example.com", "role": "attendee",
                "participationStatus": "accepted", "expectReply": false
            }
        }
    })
}

#[test]
fn itip_reply_roundtrips_with_partstat() {
    let payload = build_itip(&invite_event(), ItipMethod::Reply).unwrap();
    assert!(payload.contains("METHOD:REPLY"));
    assert!(payload.contains("PARTSTAT=ACCEPTED"));

    let (method, parsed) = parse_itip(payload.as_bytes()).unwrap();
    assert_eq!(method, ItipMethod::Reply);
    assert_eq!(parsed.json["uid"], "mtg-9@mailwoman");
    assert_eq!(
        parsed.json["participants"]["me@example.com"]["participationStatus"],
        "accepted"
    );
}

#[test]
fn itip_counter_carries_proposed_time() {
    // A COUNTER proposes a different start (attendee counter-proposal).
    let mut ev = invite_event();
    ev["start"] = json!("2026-07-15T16:00:00");
    ev["sequence"] = json!(2);
    let payload = build_itip(&ev, ItipMethod::Counter).unwrap();
    assert!(payload.contains("METHOD:COUNTER"));

    let (method, parsed) = parse_itip(payload.as_bytes()).unwrap();
    assert_eq!(method, ItipMethod::Counter);
    assert_eq!(parsed.json["start"], "2026-07-15T16:00:00");
    assert_eq!(parsed.json["sequence"], 2);
}

#[test]
fn itip_request_then_cancel() {
    let req = build_itip(&invite_event(), ItipMethod::Request).unwrap();
    assert_eq!(parse_itip(req.as_bytes()).unwrap().0, ItipMethod::Request);
    let cancel = build_itip(&invite_event(), ItipMethod::Cancel).unwrap();
    assert_eq!(parse_itip(cancel.as_bytes()).unwrap().0, ItipMethod::Cancel);
}

// ── .hol import ──────────────────────────────────────────────────────────────

#[test]
fn hol_import_yields_all_day_events() {
    let events = parse_hol(HOLIDAYS_HOL.as_bytes()).unwrap();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].json["title"], "New Year");
    assert_eq!(events[0].json["start"], "2026-01-01");
    assert_eq!(events[0].json["showWithoutTime"], true);
    assert_eq!(events[2].json["title"], "Christmas Day");
    // Emitted ICS is a valid all-day VEVENT.
    assert!(events[0].ical_raw.contains("VALUE=DATE"));
}

// ── VFREEBUSY merge ──────────────────────────────────────────────────────────

#[test]
fn freebusy_merges_overlapping_and_skips_free() {
    let busy_a = json!({
        "start": "2026-07-10T09:00:00", "timeZone": "UTC", "duration": "PT2H",
        "freeBusyStatus": "busy", "status": "confirmed", "recurrenceRules": [], "excludedRecurrenceDates": []
    });
    // Overlaps busy_a (10:00–11:30) → should coalesce into one 09:00–11:30 block.
    let busy_b = json!({
        "start": "2026-07-10T10:00:00", "timeZone": "UTC", "duration": "PT1H30M",
        "freeBusyStatus": "busy", "status": "confirmed", "recurrenceRules": [], "excludedRecurrenceDates": []
    });
    // Free events contribute nothing.
    let free_c = json!({
        "start": "2026-07-10T09:30:00", "timeZone": "UTC", "duration": "PT1H",
        "freeBusyStatus": "free", "status": "confirmed", "recurrenceRules": [], "excludedRecurrenceDates": []
    });
    let merged = aggregate_free_busy(
        &[busy_a, busy_b, free_c],
        "2026-07-10T00:00:00Z",
        "2026-07-11T00:00:00Z",
    )
    .unwrap();
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].start_utc, "2026-07-10T09:00:00Z");
    assert_eq!(merged[0].end_utc, "2026-07-10T11:30:00Z");
    assert_eq!(merged[0].status, "busy");
}

#[test]
fn parse_rejects_garbage_without_panicking() {
    assert!(parse_ical(b"not a calendar").is_err());
    // vCard parser tolerance: a bare non-vcard string errors, never panics.
    let _ = parse_vcard(b"\xff\xfe garbage");
}
