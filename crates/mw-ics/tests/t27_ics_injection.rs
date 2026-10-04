//! t27-e2b: no projection value can end the content line it is emitted into.
//!
//! Each case emits a control event first and checks that it round-trips, then
//! emits the same event with one hostile value and checks the *parsed* result:
//! the document holds the same components and the same properties as the
//! control, so the value added nothing.
//!
//! The reader used here is deliberately lenient about line ends — CRLF, a bare
//! LF and a bare CR each end a line — because a bare CR is a line break to some
//! consumers even though the `icalendar` parser does not treat it as one.

use icalendar::parser::read_calendar;
use mw_ics::{ItipMethod, build_itip, emit_ical, emit_vcard, parse_ical, parse_itip, parse_vcard};
use serde_json::{Value, json};

fn event() -> Value {
    json!({
        "uid": "mtg-27@mailwoman",
        "calendarId": "",
        "title": "Project Kickoff",
        "description": "Agenda, then lunch",
        "locations": [{ "name": "Room 4" }],
        "start": "2026-07-15T14:00:00",
        "timeZone": "Europe/London",
        "duration": "PT1H",
        "status": "confirmed",
        "freeBusyStatus": "busy",
        "sequence": 1,
        "recurrenceRules": [{ "rrule": "FREQ=WEEKLY;COUNT=3" }],
        "participants": {
            "org@example.com": {
                "name": "Org", "email": "org@example.com", "role": "organizer",
                "participationStatus": "accepted", "expectReply": false
            },
            "me@example.com": {
                "name": "Me", "email": "me@example.com", "role": "attendee",
                "participationStatus": "needs-action", "expectReply": true
            }
        },
        "alerts": { "1": { "trigger": { "offset": "-PT15M" }, "action": "display" } }
    })
}

/// The shape of an emitted document: one `component/…/PROPERTY` path per content
/// line (calendar-level properties included), read with every kind of line end
/// honoured.
fn shape(ics: &str) -> Vec<String> {
    let unfolded = ics.replace("\r\n ", "").replace("\r\n\t", "");
    let lines = unfolded.replace("\r\n", "\n").replace('\r', "\n");
    let cal = read_calendar(&lines).unwrap_or_else(|e| panic!("unparseable: {e}\n{ics:?}"));
    fn walk(c: &icalendar::parser::Component, path: &str, out: &mut Vec<String>) {
        let here = format!("{path}/{}", c.name.as_str());
        for p in &c.properties {
            out.push(format!("{here}/{}", p.name.as_str().to_ascii_uppercase()));
        }
        for sub in &c.components {
            walk(sub, &here, out);
        }
    }
    let mut out: Vec<String> = cal
        .properties
        .iter()
        .map(|p| format!("/{}", p.name.as_str().to_ascii_uppercase()))
        .collect();
    for c in &cal.components {
        walk(c, "", &mut out);
    }
    out
}

/// Every content line of the emitted text, split on CRLF only: none may hold a
/// control character other than HTAB.
fn assert_lines_clean(ics: &str) {
    for l in ics.split("\r\n") {
        assert!(
            !l.chars().any(|c| c.is_control() && c != '\t'),
            "control character inside a content line: {l:?}"
        );
    }
}

/// The addresses of the participants a reader finds in the emitted text.
fn participants(ics: &str) -> Vec<String> {
    let parsed = parse_ical(ics.as_bytes()).unwrap();
    assert_eq!(parsed.len(), 1, "one component expected");
    let mut keys: Vec<String> = parsed[0].json["participants"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    keys.sort();
    keys
}

/// The control: the unmodified event emits and reads back.
fn control() -> (String, Vec<String>) {
    let ics = emit_ical(&event()).unwrap();
    let parsed = parse_ical(ics.as_bytes()).unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].json["uid"], "mtg-27@mailwoman");
    assert_eq!(parsed[0].json["title"], "Project Kickoff");
    assert_eq!(
        participants(&ics),
        vec!["me@example.com".to_string(), "org@example.com".to_string()]
    );
    assert_eq!(
        parsed[0].json["participants"]["me@example.com"]["name"],
        "Me"
    );
    assert_lines_clean(&ics);
    let s = shape(&ics);
    (ics, s)
}

/// Emit `ev` and require clean lines and the control's shape.
fn assert_adds_nothing(ev: &Value) -> String {
    let (_, want) = control();
    let ics = emit_ical(ev).unwrap();
    assert_lines_clean(&ics);
    assert_eq!(shape(&ics), want, "the value changed the document:\n{ics}");
    ics
}

/// A named edit that plants one hostile value in a projection.
type Case = (&'static str, Box<dyn Fn(&mut Value)>);

const INJECT: &str = "\r\nATTENDEE;PARTSTAT=ACCEPTED:mailto:victim@example.test";

#[test]
fn a_valid_event_emits_exactly_what_it_did_before() {
    // Byte-for-byte: values without control characters are not touched.
    let (ics, _) = control();
    assert_eq!(
        ics,
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         PRODID:-//Mailwoman//mw-ics//EN\r\n\
         BEGIN:VEVENT\r\n\
         UID:mtg-27@mailwoman\r\n\
         SUMMARY:Project Kickoff\r\n\
         DESCRIPTION:Agenda\\, then lunch\r\n\
         DTSTART;TZID=Europe/London:20260715T140000\r\n\
         DTEND;TZID=Europe/London:20260715T150000\r\n\
         LOCATION:Room 4\r\n\
         STATUS:CONFIRMED\r\n\
         SEQUENCE:1\r\n\
         ATTENDEE;CN=Me;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:me@example.com\r\n\
         ORGANIZER;CN=Org:mailto:org@example.com\r\n\
         RRULE:FREQ=WEEKLY;COUNT=3\r\n\
         BEGIN:VALARM\r\n\
         ACTION:DISPLAY\r\n\
         TRIGGER:-PT15M\r\n\
         DESCRIPTION:Reminder\r\n\
         END:VALARM\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n"
    );
}

#[test]
fn a_multi_line_description_keeps_its_line_breaks() {
    // LF, CRLF and a bare CR are all a TEXT line break (`\n`), not a raw byte.
    let mut ev = event();
    ev["description"] = json!("one\ntwo\r\nthree\rfour");
    let ics = assert_adds_nothing(&ev);
    assert!(
        ics.contains("DESCRIPTION:one\\ntwo\\nthree\\nfour\r\n"),
        "{ics}"
    );
}

#[test]
fn a_title_with_a_line_break_adds_no_property() {
    for brk in ["\r", "\n", "\r\n"] {
        let mut ev = event();
        ev["title"] = json!(format!(
            "Lunch{brk}ATTENDEE;PARTSTAT=ACCEPTED:mailto:victim@example.test"
        ));
        let ics = assert_adds_nothing(&ev);
        assert_eq!(participants(&ics).len(), 2, "{brk:?}");
    }
}

#[test]
fn a_location_with_a_line_break_adds_no_property() {
    let mut ev = event();
    ev["locations"] = json!([{ "name": format!("Room 4\r{}", INJECT.trim_start()) }]);
    let ics = assert_adds_nothing(&ev);
    assert_eq!(participants(&ics).len(), 2);
}

#[test]
fn a_participant_address_with_a_line_break_adds_no_attendee() {
    // The stored key and the email are both hostile; the organiser is valid.
    for brk in ["\r\n", "\n", "\r"] {
        let bad =
            format!("x@example.test{brk}ATTENDEE;PARTSTAT=ACCEPTED:mailto:victim@example.test");
        let mut ev = event();
        ev["participants"] = json!({
            "org@example.com": {
                "name": "Org", "email": "org@example.com", "role": "organizer",
                "participationStatus": "accepted", "expectReply": false
            },
            bad.clone(): {
                "name": "X", "email": bad, "role": "attendee",
                "participationStatus": "needs-action", "expectReply": true
            }
        });
        let ics = emit_ical(&ev).unwrap();
        assert_lines_clean(&ics);
        // A cal-address is refused, not repaired: the participant is left out
        // and nothing of its value is in the document.
        assert_eq!(participants(&ics), vec!["org@example.com".to_string()]);
        assert!(!ics.contains("victim"), "{brk:?}\n{ics}");
        let (_, want) = control();
        let without_attendee: Vec<String> = want
            .into_iter()
            .filter(|p| !p.ends_with("/ATTENDEE"))
            .collect();
        assert_eq!(shape(&ics), without_attendee, "{brk:?}\n{ics}");
    }
}

#[test]
fn an_organizer_address_with_a_line_break_is_left_out_of_an_itip_message() {
    let ok = build_itip(&event(), ItipMethod::Request).unwrap();
    let (method, parsed) = parse_itip(ok.as_bytes()).unwrap();
    assert_eq!(method, ItipMethod::Request);
    assert_eq!(
        parsed.json["participants"]["org@example.com"]["role"],
        "organizer"
    );

    let mut ev = event();
    ev["participants"]["org@example.com"]["email"] =
        json!("org@example.com\r\nMETHOD:CANCEL\r\nX-INJECTED:1");
    let ics = build_itip(&ev, ItipMethod::Request).unwrap();
    assert_lines_clean(&ics);
    let (method, parsed) = parse_itip(ics.as_bytes()).unwrap();
    assert_eq!(method, ItipMethod::Request);
    assert_eq!(participants_of(&parsed.json), vec!["me@example.com"]);
    let s = shape(&ics);
    assert_eq!(s.iter().filter(|p| p.ends_with("/METHOD")).count(), 1);
    assert!(!s.iter().any(|p| p.ends_with("/X-INJECTED")), "{ics}");
}

fn participants_of(json: &Value) -> Vec<&str> {
    json["participants"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect()
}

#[test]
fn a_display_name_cannot_leave_its_parameter() {
    // Valid: a name with a comma, a colon or a semicolon is one quoted CN.
    let mut ev = event();
    ev["participants"]["me@example.com"]["name"] = json!("Doe, Jane: QA; Ops");
    let ics = assert_adds_nothing(&ev);
    let parsed = parse_ical(ics.as_bytes()).unwrap();
    let me = &parsed[0].json["participants"]["me@example.com"];
    assert_eq!(me["name"], "Doe, Jane: QA; Ops");
    assert_eq!(me["participationStatus"], "needs-action");
    assert_eq!(me["expectReply"], true);

    // Hostile: a line break, and a quote that would close the parameter early.
    for name in [
        format!("Me{INJECT}"),
        "Me\rATTENDEE:mailto:victim@example.test".to_string(),
        "Me\";PARTSTAT=ACCEPTED:mailto:victim@example.test\r\nX-A:\"".to_string(),
        "Me:mailto:victim@example.test".to_string(),
    ] {
        let mut ev = event();
        ev["participants"]["me@example.com"]["name"] = json!(name);
        let ics = assert_adds_nothing(&ev);
        assert_eq!(
            participants(&ics),
            vec!["me@example.com".to_string(), "org@example.com".to_string()],
            "{name:?}\n{ics}"
        );
        let parsed = parse_ical(ics.as_bytes()).unwrap();
        let me = &parsed[0].json["participants"]["me@example.com"];
        assert_eq!(me["participationStatus"], "needs-action", "{name:?}");
    }
}

#[test]
fn values_written_without_text_escaping_cannot_add_a_line() {
    // UID, RRULE, TZID, STATUS and the alarm TRIGGER/ACTION are written as-is.
    let cases: Vec<Case> = vec![
        (
            "uid",
            Box::new(|ev| ev["uid"] = json!(format!("mtg-27@mailwoman{INJECT}"))),
        ),
        (
            "rrule",
            Box::new(|ev| {
                ev["recurrenceRules"] = json!([{ "rrule": format!("FREQ=WEEKLY;COUNT=3{INJECT}") }])
            }),
        ),
        (
            "timeZone",
            Box::new(|ev| ev["timeZone"] = json!(format!("Europe/London{INJECT}"))),
        ),
        (
            "status",
            Box::new(|ev| ev["status"] = json!(format!("confirmed{INJECT}"))),
        ),
        (
            "trigger",
            Box::new(|ev| {
                ev["alerts"]["1"]["trigger"]["offset"] = json!(
                    "-PT15M\r\nEND:VALARM\r\nATTENDEE:mailto:victim@example.test\r\nBEGIN:VALARM\r\nACTION:DISPLAY"
                )
            }),
        ),
        (
            "action",
            Box::new(|ev| ev["alerts"]["1"]["action"] = json!(format!("display{INJECT}"))),
        ),
        (
            "partstat",
            Box::new(|ev| {
                ev["participants"]["me@example.com"]["participationStatus"] =
                    json!(format!("needs-action{INJECT}"))
            }),
        ),
    ];
    for (what, mutate) in cases {
        let mut ev = event();
        mutate(&mut ev);
        let ics = assert_adds_nothing(&ev);
        assert_eq!(
            participants(&ics),
            vec!["me@example.com".to_string(), "org@example.com".to_string()],
            "{what}\n{ics}"
        );
    }
}

#[test]
fn a_task_parent_id_cannot_add_a_line() {
    let task = json!({
        "uid": "task-27@mailwoman", "listId": "", "title": "Write report",
        "due": "2026-03-25T17:00:00", "status": "needs-action", "parentId": "parent-1"
    });
    let ok = emit_ical(&task).unwrap();
    let parsed = parse_ical(ok.as_bytes()).unwrap();
    assert_eq!(parsed[0].component, "VTODO");
    assert_eq!(parsed[0].json["parentId"], "parent-1");
    let want = shape(&ok);

    let mut bad = task.clone();
    bad["parentId"] = json!("parent-1\r\nEND:VTODO\r\nBEGIN:VTODO\r\nUID:injected");
    let ics = emit_ical(&bad).unwrap();
    assert_lines_clean(&ics);
    assert_eq!(shape(&ics), want, "{ics}");
    assert_eq!(parse_ical(ics.as_bytes()).unwrap().len(), 1);
}

// ── vCard: the same rule for the hand-rolled emitter ─────────────────────────

fn contact() -> Value {
    json!({
        "uid": "card-27@mailwoman",
        "kind": "individual",
        "name": { "full": "Bob Jones", "given": "Bob", "surname": "Jones", "prefix": "", "suffix": "" },
        "nicknames": ["Bobby"],
        "organizations": ["Acme;Engineering"],
        "titles": ["Engineer"],
        "emails": [{ "context": "work", "value": "bob@acme.com", "pref": 1 }],
        "phones": [{ "context": "work", "value": "+15551234" }],
        "onlineServices": [],
        "notes": "A contact",
        "pgpKey": "pgp-key-data"
    })
}

/// The cards a reader finds, each as (email values, phone values).
fn cards(vcf: &str) -> Vec<(Vec<String>, Vec<String>)> {
    let values = |v: &Value, k: &str| -> Vec<String> {
        v[k].as_array()
            .unwrap()
            .iter()
            .map(|e| e["value"].as_str().unwrap().to_string())
            .collect()
    };
    parse_vcard(vcf.as_bytes())
        .unwrap_or_else(|e| panic!("unparseable: {e}\n{vcf:?}"))
        .iter()
        .map(|c| (values(&c.json, "emails"), values(&c.json, "phones")))
        .collect()
}

#[test]
fn a_valid_contact_emits_exactly_what_it_did_before() {
    let vcf = emit_vcard(&contact()).unwrap();
    assert_eq!(
        vcf,
        "BEGIN:VCARD\r\n\
         VERSION:4.0\r\n\
         UID:card-27@mailwoman\r\n\
         KIND:individual\r\n\
         FN:Bob Jones\r\n\
         N:Jones;Bob;;;\r\n\
         NICKNAME:Bobby\r\n\
         ORG:Acme;Engineering\r\n\
         TITLE:Engineer\r\n\
         EMAIL;TYPE=work;PREF=1:bob@acme.com\r\n\
         TEL;TYPE=work:+15551234\r\n\
         NOTE:A contact\r\n\
         KEY:pgp-key-data\r\n\
         END:VCARD\r\n"
    );
    assert_eq!(
        cards(&vcf),
        vec![(
            vec!["bob@acme.com".to_string()],
            vec!["+15551234".to_string()]
        )]
    );
}

#[test]
fn a_contact_value_with_a_line_break_adds_no_vcard_line() {
    const VC_INJECT: &str = "\r\nEMAIL:victim@example.test";
    let cases: Vec<Case> = vec![
        (
            "uid",
            Box::new(|c| c["uid"] = json!(format!("card-27@mailwoman{VC_INJECT}"))),
        ),
        (
            "kind",
            Box::new(|c| c["kind"] = json!(format!("individual{VC_INJECT}"))),
        ),
        (
            "full name, bare CR",
            Box::new(|c| c["name"]["full"] = json!("Bob Jones\rEMAIL:victim@example.test")),
        ),
        (
            "surname, bare CR",
            Box::new(|c| c["name"]["surname"] = json!("Jones\rEMAIL:victim@example.test")),
        ),
        (
            "organization",
            Box::new(|c| c["organizations"] = json!([format!("Acme;Engineering{VC_INJECT}")])),
        ),
        (
            "email value",
            Box::new(|c| c["emails"][0]["value"] = json!(format!("bob@acme.com{VC_INJECT}"))),
        ),
        (
            "email context",
            Box::new(|c| c["emails"][0]["context"] = json!(format!("work{VC_INJECT}"))),
        ),
        (
            "phone value",
            Box::new(|c| c["phones"][0]["value"] = json!(format!("+15551234{VC_INJECT}"))),
        ),
        (
            "notes, bare CR",
            Box::new(|c| c["notes"] = json!("A contact\rEMAIL:victim@example.test")),
        ),
        (
            "pgp key ends the card",
            Box::new(|c| {
                c["pgpKey"] = json!(
                    "pgp-key-data\r\nEND:VCARD\r\nBEGIN:VCARD\r\nVERSION:4.0\r\nFN:Victim\r\nEMAIL:victim@example.test"
                )
            }),
        ),
    ];
    let control = emit_vcard(&contact()).unwrap();
    assert_eq!(cards(&control).len(), 1);
    let want = vcard_shape(&control);
    assert_eq!(want.iter().filter(|n| *n == "EMAIL").count(), 1);
    for (what, mutate) in cases {
        let mut c = contact();
        mutate(&mut c);
        let vcf = emit_vcard(&c).unwrap();
        assert_lines_clean(&vcf);
        assert_eq!(vcard_shape(&vcf), want, "{what}\n{vcf}");
    }
}

/// The property name of every content line of an emitted vCard, in order, with
/// CRLF, a bare LF and a bare CR each taken as a line end. `BEGIN`/`END` lines
/// are included, so a second card shows up as extra names.
///
/// This reads the lines itself instead of going through `parse_vcard`: that
/// reader panics inside its URI parser on some single-line values the hostile
/// cases leave behind (a `UID` such as `a@b:c`, a `TEL` such as `+1:x`).
fn vcard_shape(vcf: &str) -> Vec<String> {
    vcf.replace("\r\n", "\n")
        .replace('\r', "\n")
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let end = l.find([':', ';']).unwrap_or(l.len());
            l[..end].to_ascii_uppercase()
        })
        .collect()
}

#[test]
fn a_multi_line_note_keeps_its_line_breaks() {
    let mut c = contact();
    c["notes"] = json!("one\ntwo\r\nthree\rfour");
    let vcf = emit_vcard(&c).unwrap();
    assert!(vcf.contains("NOTE:one\\ntwo\\nthree\\nfour\r\n"), "{vcf}");
    let parsed = parse_vcard(vcf.as_bytes()).unwrap();
    assert_eq!(parsed[0].json["notes"], "one\ntwo\nthree\nfour");
}
