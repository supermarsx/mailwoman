//! iCalendar VEVENT/VTODO parse+emit against the frozen §2.1 Mailwoman
//! projection.
//!
//! Reading uses the `icalendar` crate's low-level generic parser (every
//! property is captured, so unknown X-properties survive into `ical_raw`);
//! writing rebuilds a generic `parser::Calendar` and lets the crate handle line
//! folding + CRLF. `ical_raw` is the round-trip source of truth (plan risk #13):
//! the `json` projection is a lossy view over a focused property set, the raw
//! bytes are not.

use icalendar::parser::{
    Calendar as PCalendar, Component as PComponent, Parameter as PParameter, Property as PProperty,
    read_calendar,
};
use serde_json::{Value, json};

use crate::dt;
use crate::{IcsError, Result};

/// One parsed calendar object: the Mailwoman-shape JSON projection plus the
/// verbatim (re-serialized) `ical_raw` round-trip source of truth.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedIcal {
    pub ical_raw: String,
    pub json: Value,
    /// `"VEVENT"` / `"VTODO"` / `"VJOURNAL"`.
    pub component: String,
}

// ── generic property helpers ────────────────────────────────────────────────

fn find<'a>(c: &'a PComponent<'a>, name: &str) -> Option<&'a PProperty<'a>> {
    c.properties
        .iter()
        .find(|p| p.name.as_str().eq_ignore_ascii_case(name))
}

fn find_all<'a>(c: &'a PComponent<'a>, name: &str) -> Vec<&'a PProperty<'a>> {
    c.properties
        .iter()
        .filter(|p| p.name.as_str().eq_ignore_ascii_case(name))
        .collect()
}

fn text(c: &PComponent, name: &str) -> String {
    find(c, name)
        .map(|p| p.val.as_str().to_string())
        .unwrap_or_default()
}

fn get_param<'a>(p: &'a PProperty<'a>, key: &str) -> Option<&'a str> {
    p.params
        .iter()
        .find(|pm| pm.key.as_str().eq_ignore_ascii_case(key))
        .and_then(|pm| pm.val.as_ref().map(|v| v.as_str()))
}

// ── owned-property builders (for emit) ──────────────────────────────────────

fn prop(name: &str, val: String) -> PProperty<'static> {
    PProperty {
        name: name.to_string().into(),
        val: val.into(),
        params: vec![],
    }
}

fn param(key: &str, val: &str) -> PParameter<'static> {
    PParameter {
        key: key.to_string().into(),
        val: Some(val.to_string().into()),
    }
}

fn prop_p(name: &str, val: String, params: Vec<PParameter<'static>>) -> PProperty<'static> {
    PProperty {
        name: name.to_string().into(),
        val: val.into(),
        params,
    }
}

/// RFC 5545 TEXT escaping (backslash, semicolon, comma, newline).
fn esc(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
}

/// Wrap components into a `VCALENDAR` string with the given extra top-level
/// properties (e.g. an iTIP `METHOD`). Shared with the iTIP framing layer.
pub(crate) fn wrap_calendar(
    components: Vec<PComponent<'static>>,
    extra: Vec<PProperty<'static>>,
) -> String {
    let mut properties = vec![
        prop("VERSION", "2.0".into()),
        prop("PRODID", "-//Mailwoman//mw-ics//EN".into()),
    ];
    properties.extend(extra);
    PCalendar {
        properties,
        components,
    }
    .to_string()
}

// ── DTSTART / DTEND / DUE encoding ───────────────────────────────────────────

/// Build a date/date-time property, attaching `TZID`/`VALUE=DATE`/`Z` as the
/// projection dictates.
fn date_prop(name: &str, local: &str, tz: Option<&str>, is_date: bool) -> PProperty<'static> {
    let compact = dt::to_compact(local);
    if is_date {
        return prop_p(name, compact, vec![param("VALUE", "DATE")]);
    }
    match tz {
        Some("UTC") | Some("Etc/UTC") => prop(name, format!("{compact}Z")),
        Some(z) => prop_p(name, compact, vec![param("TZID", z)]),
        None => prop(name, compact),
    }
}

// ── VEVENT / VTODO → JSON projection ─────────────────────────────────────────

fn read_datetime(c: &PComponent, name: &str) -> Option<(String, Option<String>, bool)> {
    let p = find(c, name)?;
    let parsed = dt::parse_ical_dt(p.val.as_str())?;
    let tz = if parsed.is_utc {
        Some("UTC".to_string())
    } else {
        get_param(p, "TZID").map(|s| s.to_string())
    };
    Some((parsed.local, tz, parsed.is_date))
}

fn read_participants(c: &PComponent) -> Value {
    let mut map = serde_json::Map::new();
    let mut add = |p: &PProperty, role: &str| {
        let email = p
            .val
            .as_str()
            .trim_start_matches("mailto:")
            .trim_start_matches("MAILTO:");
        let name = get_param(p, "CN").unwrap_or("").to_string();
        let partstat = get_param(p, "PARTSTAT")
            .unwrap_or("NEEDS-ACTION")
            .to_ascii_lowercase();
        let rsvp = get_param(p, "RSVP")
            .map(|v| v.eq_ignore_ascii_case("TRUE"))
            .unwrap_or(false);
        map.insert(
            email.to_string(),
            json!({
                "name": name,
                "email": email,
                "role": role,
                "participationStatus": partstat,
                "expectReply": rsvp,
            }),
        );
    };
    if let Some(org) = find(c, "ORGANIZER") {
        add(org, "organizer");
    }
    for att in find_all(c, "ATTENDEE") {
        add(att, "attendee");
    }
    Value::Object(map)
}

fn read_alerts(c: &PComponent) -> Value {
    let mut map = serde_json::Map::new();
    let mut idx = 0u32;
    for sub in &c.components {
        if !sub.name.as_str().eq_ignore_ascii_case("VALARM") {
            continue;
        }
        idx += 1;
        let action = text(sub, "ACTION").to_ascii_lowercase();
        let trig_raw = text(sub, "TRIGGER");
        let trigger = if trig_raw.starts_with('P')
            || trig_raw.starts_with('-')
            || trig_raw.starts_with('+')
        {
            json!({ "offset": trig_raw })
        } else {
            json!({ "absolute": trig_raw })
        };
        map.insert(
            idx.to_string(),
            json!({
                "trigger": trigger,
                "action": if action.is_empty() { "display".into() } else { action },
            }),
        );
    }
    Value::Object(map)
}

fn event_to_json(c: &PComponent) -> Value {
    let uid = text(c, "UID");
    let (start, tz, is_date) = read_datetime(c, "DTSTART").unwrap_or_default();
    // duration = explicit DURATION, else DTEND − DTSTART (nominal wall time).
    let duration = if let Some(d) = find(c, "DURATION") {
        d.val.as_str().to_string()
    } else if let (Some(s), Some((e, _, _))) =
        (dt::local_to_naive(&start), read_datetime(c, "DTEND"))
    {
        match dt::local_to_naive(&e) {
            Some(en) => dt::secs_to_iso_duration((en - s).num_seconds(), is_date),
            None => {
                if is_date {
                    "P1D".into()
                } else {
                    "PT0S".into()
                }
            }
        }
    } else if is_date {
        "P1D".into()
    } else {
        "PT0S".into()
    };

    let rrules: Vec<Value> = find_all(c, "RRULE")
        .iter()
        .map(|p| json!({ "rrule": p.val.as_str() }))
        .collect();
    let mut exdates: Vec<Value> = vec![];
    for p in find_all(c, "EXDATE") {
        for part in p.val.as_str().split(',') {
            if let Some(d) = dt::parse_ical_dt(part) {
                exdates.push(Value::String(d.local));
            }
        }
    }

    let status = {
        let s = text(c, "STATUS").to_ascii_lowercase();
        if s.is_empty() { "confirmed".into() } else { s }
    };
    let free_busy = match text(c, "TRANSP").to_ascii_uppercase().as_str() {
        "TRANSPARENT" => "free",
        _ => "busy",
    };
    let priority: i64 = text(c, "PRIORITY").parse().unwrap_or(0);
    let sequence: i64 = text(c, "SEQUENCE").parse().unwrap_or(0);
    let location = text(c, "LOCATION");
    let locations = if location.is_empty() {
        vec![]
    } else {
        vec![json!({ "name": location })]
    };

    json!({
        "id": uid,
        "calendarId": "",
        "uid": uid,
        "title": text(c, "SUMMARY"),
        "description": text(c, "DESCRIPTION"),
        "locations": locations,
        "start": start,
        "timeZone": tz,
        "duration": duration,
        "showWithoutTime": is_date,
        "recurrenceRules": rrules,
        "recurrenceOverrides": {},
        "excludedRecurrenceDates": exdates,
        "status": status,
        "priority": priority,
        "freeBusyStatus": free_busy,
        "participants": read_participants(c),
        "alerts": read_alerts(c),
        "sequence": sequence,
        "etag": Value::Null,
    })
}

fn todo_to_json(c: &PComponent) -> Value {
    let uid = text(c, "UID");
    let (start, tz, _) = read_datetime(c, "DTSTART").unwrap_or((String::new(), None, false));
    let start_v = if start.is_empty() {
        Value::Null
    } else {
        Value::String(start)
    };
    let (due, due_tz, _) = read_datetime(c, "DUE").unwrap_or((String::new(), None, false));
    let due_v = if due.is_empty() {
        Value::Null
    } else {
        Value::String(due)
    };
    let tz = tz.or(due_tz);
    let rrules: Vec<Value> = find_all(c, "RRULE")
        .iter()
        .map(|p| json!({ "rrule": p.val.as_str() }))
        .collect();
    let status = {
        let s = text(c, "STATUS").to_ascii_lowercase();
        if s.is_empty() {
            "needs-action".into()
        } else {
            s
        }
    };
    let related = find_all(c, "RELATED-TO")
        .first()
        .map(|p| Value::String(p.val.as_str().to_string()))
        .unwrap_or(Value::Null);

    json!({
        "id": uid,
        "listId": "",
        "uid": uid,
        "title": text(c, "SUMMARY"),
        "description": text(c, "DESCRIPTION"),
        "start": start_v,
        "due": due_v,
        "timeZone": tz,
        "priority": text(c, "PRIORITY").parse::<i64>().unwrap_or(0),
        "percentComplete": text(c, "PERCENT-COMPLETE").parse::<i64>().unwrap_or(0),
        "status": status,
        "progress": "",
        "recurrenceRules": rrules,
        "parentId": related,
        "myDayDate": Value::Null,
        "etag": Value::Null,
    })
}

// ── JSON projection → VEVENT / VTODO component ───────────────────────────────

fn s(v: &Value, k: &str) -> String {
    v.get(k)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn opt_s(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(Value::as_str).map(str::to_string)
}

fn i(v: &Value, k: &str) -> i64 {
    v.get(k).and_then(Value::as_i64).unwrap_or(0)
}

fn participant_props(v: &Value) -> Vec<PProperty<'static>> {
    let mut out = vec![];
    let Some(map) = v.get("participants").and_then(Value::as_object) else {
        return out;
    };
    for part in map.values() {
        let email = s(part, "email");
        if email.is_empty() {
            continue;
        }
        let name = s(part, "name");
        let mut params = vec![];
        if !name.is_empty() {
            params.push(param("CN", &name));
        }
        if s(part, "role") == "organizer" {
            out.push(prop_p("ORGANIZER", format!("mailto:{email}"), params));
        } else {
            let ps = s(part, "participationStatus");
            if !ps.is_empty() {
                params.push(param("PARTSTAT", &ps.to_ascii_uppercase()));
            }
            if part
                .get("expectReply")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                params.push(param("RSVP", "TRUE"));
            }
            out.push(prop_p("ATTENDEE", format!("mailto:{email}"), params));
        }
    }
    out
}

fn alarm_components(v: &Value) -> Vec<PComponent<'static>> {
    let mut out = vec![];
    let Some(map) = v.get("alerts").and_then(Value::as_object) else {
        return out;
    };
    for alert in map.values() {
        let action = s(alert, "action");
        let action = if action.is_empty() {
            "display".into()
        } else {
            action
        };
        let trigger = alert.get("trigger");
        let trig_val = trigger
            .and_then(|t| t.get("offset").or_else(|| t.get("absolute")))
            .and_then(Value::as_str)
            .unwrap_or("-PT15M")
            .to_string();
        out.push(PComponent {
            name: "VALARM".to_string().into(),
            properties: vec![
                prop("ACTION", action.to_ascii_uppercase()),
                prop("TRIGGER", trig_val),
                prop("DESCRIPTION", "Reminder".into()),
            ],
            components: vec![],
        });
    }
    out
}

/// Build the VEVENT/VTODO `parser::Component` for a projection object. Shared
/// with the iTIP framing layer.
pub(crate) fn to_component(v: &Value) -> PComponent<'static> {
    let is_todo = v.get("listId").is_some() || v.get("due").is_some();
    let tz = opt_s(v, "timeZone");
    let mut props = vec![prop("UID", s(v, "uid"))];
    let title = s(v, "title");
    if !title.is_empty() {
        props.push(prop("SUMMARY", esc(&title)));
    }
    let desc = s(v, "description");
    if !desc.is_empty() {
        props.push(prop("DESCRIPTION", esc(&desc)));
    }

    if is_todo {
        if let Some(st) = opt_s(v, "start") {
            props.push(date_prop("DTSTART", &st, tz.as_deref(), false));
        }
        if let Some(due) = opt_s(v, "due") {
            props.push(date_prop("DUE", &due, tz.as_deref(), false));
        }
        let pc = i(v, "percentComplete");
        if pc > 0 {
            props.push(prop("PERCENT-COMPLETE", pc.to_string()));
        }
        if let Some(parent) = opt_s(v, "parentId") {
            props.push(prop_p(
                "RELATED-TO",
                parent,
                vec![param("RELTYPE", "PARENT")],
            ));
        }
        let status = s(v, "status");
        if !status.is_empty() {
            props.push(prop("STATUS", status.to_ascii_uppercase()));
        }
    } else {
        let start = s(v, "start");
        let is_date = v
            .get("showWithoutTime")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        props.push(date_prop("DTSTART", &start, tz.as_deref(), is_date));
        // Emit DTEND from start + duration for lossless round-trip.
        if let (Some(sn), Some(secs)) = (
            dt::local_to_naive(&start),
            dt::iso_duration_to_secs(&s(v, "duration")),
        ) {
            let end = sn + chrono::Duration::seconds(secs);
            let end_local = if is_date {
                end.format("%Y-%m-%d").to_string()
            } else {
                end.format("%Y-%m-%dT%H:%M:%S").to_string()
            };
            props.push(date_prop("DTEND", &end_local, tz.as_deref(), is_date));
        }
        if let Some(loc) = v
            .get("locations")
            .and_then(Value::as_array)
            .and_then(|a| a.first())
        {
            let name = s(loc, "name");
            if !name.is_empty() {
                props.push(prop("LOCATION", esc(&name)));
            }
        }
        let status = s(v, "status");
        if !status.is_empty() {
            props.push(prop("STATUS", status.to_ascii_uppercase()));
        }
        if s(v, "freeBusyStatus") == "free" {
            props.push(prop("TRANSP", "TRANSPARENT".into()));
        }
        let seq = i(v, "sequence");
        if seq > 0 {
            props.push(prop("SEQUENCE", seq.to_string()));
        }
        props.extend(participant_props(v));
    }

    let prio = i(v, "priority");
    if prio > 0 {
        props.push(prop("PRIORITY", prio.to_string()));
    }
    for rule in v
        .get("recurrenceRules")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(r) = rule.get("rrule").and_then(Value::as_str) {
            props.push(prop("RRULE", r.to_string()));
        }
    }
    for ex in v
        .get("excludedRecurrenceDates")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(d) = ex.as_str() {
            let is_date = !d.contains('T');
            props.push(date_prop("EXDATE", d, tz.as_deref(), is_date));
        }
    }

    let alarms = if is_todo { vec![] } else { alarm_components(v) };
    PComponent {
        name: if is_todo { "VTODO" } else { "VEVENT" }.to_string().into(),
        properties: props,
        components: alarms,
    }
}

// ── public API ───────────────────────────────────────────────────────────────

/// Parse an iCalendar document into per-component Mailwoman projections.
pub fn parse_ical(bytes: &[u8]) -> Result<Vec<ParsedIcal>> {
    let text = String::from_utf8_lossy(bytes);
    let cal = read_calendar(&text).map_err(IcsError::Ical)?;
    let mut out = vec![];
    for comp in &cal.components {
        let name = comp.name.as_str().to_ascii_uppercase();
        let json = match name.as_str() {
            "VEVENT" => event_to_json(comp),
            "VTODO" => todo_to_json(comp),
            _ => continue,
        };
        let ical_raw = wrap_calendar(vec![to_component(&json)], vec![]);
        out.push(ParsedIcal {
            ical_raw,
            json,
            component: name,
        });
    }
    Ok(out)
}

/// Emit a single-component `VCALENDAR` from a Mailwoman event/task projection.
pub fn emit_ical(event_json: &Value) -> Result<String> {
    Ok(wrap_calendar(vec![to_component(event_json)], vec![]))
}
