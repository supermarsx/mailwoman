//! `CalendarEvent/itip` (26.20 t28-e21): read the iTIP part of a received
//! message, say what applying it would do, and apply it only when asked.
//!
//! `{emailId, apply?, calendarId?}` → `{accountId, emailId, method, sender,
//! organizer, event, eventId, outcome, applied}`. The message is read from the
//! account's own store (an id of another account does not resolve), and the
//! sender is its `From` address: the client names the message, never who sent
//! it.
//!
//! The guards, in RFC 5546 terms:
//! - a REQUEST or CANCEL changes a stored event only when its sender is that
//!   event's ORGANIZER; a REPLY only when its sender is a stored ATTENDEE, and
//!   then only that attendee's answer;
//! - a message whose SEQUENCE is lower than the stored one, or equal with an
//!   older DTSTAMP, is stale and changes nothing (a replayed or reordered
//!   message);
//! - a REQUEST for a UID the account does not hold is a new invitation: anyone
//!   may send one, and it is added only by the user's action;
//! - COUNTER is shown, not applied.
//!
//! Nothing here sends mail. Answering an invitation is `CalendarEvent/respond`,
//! which sends through `send_itip` and its address checks.

use mail_parser::{MessageParser, MimeHeaders, PartType};
use serde_json::{Value, json};

use crate::account::AccountRuntime;
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;

use super::events::{method_name, normalize_event, set_json, set_str};
use super::{gen_id, server_fail};
use mw_ics::ItipMethod;
use mw_store::EventRow;

/// What applying a message would do. The wire value is the camelCase name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    /// A REQUEST for a UID the account does not hold: add the event.
    New,
    /// A newer REQUEST from the organizer: replace the event.
    Update,
    /// The message is the one already applied.
    Current,
    /// Older than what was applied; ignored.
    Stale,
    /// A CANCEL from the organizer: mark the event cancelled.
    Cancel,
    /// A REPLY from an attendee: record their answer.
    Reply,
    /// A REQUEST or CANCEL for a stored event, not sent by its organizer.
    NotOrganizer,
    /// A REPLY not sent by an attendee of the stored event (or answering for
    /// someone other than its sender).
    NotAttendee,
    /// A CANCEL or REPLY for an event the account does not hold.
    UnknownEvent,
    /// A COUNTER: shown to the user, never applied here.
    Unsupported,
}

impl Outcome {
    fn wire(self) -> &'static str {
        match self {
            Outcome::New => "new",
            Outcome::Update => "update",
            Outcome::Current => "current",
            Outcome::Stale => "stale",
            Outcome::Cancel => "cancel",
            Outcome::Reply => "reply",
            Outcome::NotOrganizer => "notOrganizer",
            Outcome::NotAttendee => "notAttendee",
            Outcome::UnknownEvent => "unknownEvent",
            Outcome::Unsupported => "unsupported",
        }
    }

    fn applies(self) -> bool {
        matches!(
            self,
            Outcome::New | Outcome::Update | Outcome::Cancel | Outcome::Reply
        )
    }
}

/// The scheduling content of a received message.
struct Invitation {
    /// The single `From` address, lower-cased.
    sender: String,
    /// The `text/calendar` (or `application/ics`) part, decoded.
    ics: String,
}

impl Engine {
    pub(crate) async fn event_itip(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let Some(email_id) = args.get("emailId").and_then(Value::as_str) else {
            return invalid("CalendarEvent/itip requires emailId");
        };
        let apply = args.get("apply").and_then(Value::as_bool).unwrap_or(false);
        // A message id only: not one of its parts, not an upload.
        if email_id.is_empty() || email_id.contains('.') || email_id.starts_with('U') {
            return not_found(email_id);
        }
        let raw = match self.fetch_blob(account_id, email_id).await {
            Ok(Some(blob)) => blob.bytes,
            Ok(None) => return not_found(email_id),
            Err(e) => return server_fail(e),
        };
        let Some(inv) = read_invitation(&raw) else {
            return invalid("the message has no calendar part or no single sender address");
        };
        let (method, parsed) = match mw_ics::parse_itip(inv.ics.as_bytes()) {
            Ok(v) => v,
            Err(e) => return invalid(format!("the calendar part is not an iTIP message: {e}")),
        };
        let mut incoming = parsed.json;
        let uid = incoming
            .get("uid")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if uid.is_empty() {
            return invalid("the calendar part has no UID");
        }
        let dtstamp = dtstamp_of(&inv.ics);
        if let Some(d) = &dtstamp {
            set_str(&mut incoming, "itipDtstamp", d);
        }

        let stored = match self.event_by_uid(account_id, &uid).await {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        let stored_json = stored.as_ref().map(|row| self.event_row_to_json(row));
        let outcome = decide(method, &inv.sender, &incoming, stored_json.as_ref());

        let mut out = json!({
            "accountId": account_id,
            "emailId": email_id,
            "method": method_name(method),
            "sender": inv.sender,
            "organizer": organizer_of(&incoming),
            "event": incoming,
            "eventId": stored.as_ref().map(|r| r.id.clone()),
            "outcome": outcome.wire(),
            "applied": false,
        });
        if !apply || !outcome.applies() {
            return out;
        }
        let applied = match (outcome, stored, stored_json) {
            (Outcome::New, _, _) => {
                self.itip_add(account_id, rt, args, &uid, out["event"].clone())
                    .await
            }
            (o, Some(row), Some(current)) => {
                let next = match o {
                    Outcome::Update => updated(&current, out["event"].clone()),
                    Outcome::Cancel => cancelled(&current, &out["event"]),
                    _ => replied(&current, &out["event"], &inv.sender),
                };
                self.itip_replace(account_id, rt, &row, next).await
            }
            _ => return out,
        };
        match applied {
            Ok(id) => {
                out["eventId"] = json!(id);
                out["applied"] = json!(true);
                self.broadcast_state(account_id).await;
                out
            }
            Err(e) => server_fail(e),
        }
    }

    /// The account's event with `uid` in one of its own event calendars (a
    /// subscribed feed is not one), if any.
    async fn event_by_uid(
        &self,
        account_id: &str,
        uid: &str,
    ) -> crate::backend::Result<Option<EventRow>> {
        for cal in self.store().list_calendars(account_id).await? {
            if cal.component != "VEVENT" || cal.is_overlay {
                continue;
            }
            if let Some(row) = self
                .store()
                .list_events(&cal.id)
                .await?
                .into_iter()
                .find(|r| r.uid == uid)
            {
                return Ok(Some(row));
            }
        }
        Ok(None)
    }

    /// Store a new invitation in `calendarId` or the default calendar.
    async fn itip_add(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
        uid: &str,
        mut json: Value,
    ) -> crate::backend::Result<String> {
        let calendar_id = match args.get("calendarId").and_then(Value::as_str) {
            Some(c) => c.to_string(),
            None => self.ensure_default_calendar(account_id, "VEVENT").await?,
        };
        let id = gen_id("ev");
        normalize_event(&mut json);
        self.persist_event(account_id, &calendar_id, &id, uid, json, None, Some(rt))
            .await?;
        self.record_pim_change(
            account_id,
            ChangeType::CalendarEvent,
            &id,
            ChangeOp::Created,
        )
        .await?;
        Ok(id)
    }

    /// Replace a stored event with `json`, keeping its id, calendar and UID.
    async fn itip_replace(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        row: &EventRow,
        mut json: Value,
    ) -> crate::backend::Result<String> {
        normalize_event(&mut json);
        self.persist_event(
            account_id,
            &row.calendar_id,
            &row.id,
            &row.uid,
            json,
            row.etag.clone(),
            Some(rt),
        )
        .await?;
        self.record_pim_change(
            account_id,
            ChangeType::CalendarEvent,
            &row.id,
            ChangeOp::Updated,
        )
        .await?;
        Ok(row.id.clone())
    }
}

/// The guard: what `method` from `sender` may do, given the stored event.
fn decide(method: ItipMethod, sender: &str, incoming: &Value, stored: Option<&Value>) -> Outcome {
    let Some(stored) = stored else {
        return match method {
            ItipMethod::Request => Outcome::New,
            ItipMethod::Counter => Outcome::Unsupported,
            ItipMethod::Cancel | ItipMethod::Reply => Outcome::UnknownEvent,
        };
    };
    match method {
        ItipMethod::Counter => Outcome::Unsupported,
        ItipMethod::Request | ItipMethod::Cancel => {
            if organizer_of(stored).is_none_or(|o| !same_address(&o, sender)) {
                return Outcome::NotOrganizer;
            }
            match freshness(incoming, stored) {
                Freshness::Older => Outcome::Stale,
                Freshness::Same if method == ItipMethod::Request => Outcome::Current,
                _ if method == ItipMethod::Request => Outcome::Update,
                _ if status_of(stored) == "cancelled" => Outcome::Current,
                _ => Outcome::Cancel,
            }
        }
        ItipMethod::Reply => {
            let attends = |ev: &Value| {
                participant(ev, sender)
                    .is_some_and(|p| p.get("role").and_then(Value::as_str) != Some("organizer"))
            };
            if !attends(stored) || !attends(incoming) {
                return Outcome::NotAttendee;
            }
            if sequence(incoming) < sequence(stored) {
                return Outcome::Stale;
            }
            Outcome::Reply
        }
    }
}

enum Freshness {
    Older,
    Same,
    Newer,
}

/// RFC 5546 §2.1.5: SEQUENCE first, then DTSTAMP. A stored event with no
/// recorded DTSTAMP (made here, or imported from a plain file) is older than
/// any message of the same SEQUENCE.
fn freshness(incoming: &Value, stored: &Value) -> Freshness {
    match sequence(incoming).cmp(&sequence(stored)) {
        std::cmp::Ordering::Less => Freshness::Older,
        std::cmp::Ordering::Greater => Freshness::Newer,
        std::cmp::Ordering::Equal => {
            let stamp = |ev: &Value| {
                ev.get("itipDtstamp")
                    .and_then(Value::as_str)
                    .map(String::from)
            };
            match (stamp(incoming), stamp(stored)) {
                (Some(i), Some(s)) if i < s => Freshness::Older,
                (Some(i), Some(s)) if i == s => Freshness::Same,
                (None, Some(_)) => Freshness::Same,
                _ => Freshness::Newer,
            }
        }
    }
}

/// The organizer's projection replaced by the newer one, keeping what is the
/// user's own: their alerts and categories.
fn updated(current: &Value, mut next: Value) -> Value {
    for key in ["alerts", "categories"] {
        if let Some(v) = current.get(key).filter(|v| !v.is_null()) {
            set_json(&mut next, key, v.clone());
        }
    }
    next
}

/// The stored event marked cancelled, with the message's SEQUENCE and DTSTAMP.
fn cancelled(current: &Value, incoming: &Value) -> Value {
    let mut next = current.clone();
    set_str(&mut next, "status", "cancelled");
    set_json(
        &mut next,
        "sequence",
        json!(sequence(incoming).max(sequence(current))),
    );
    if let Some(d) = incoming.get("itipDtstamp").filter(|v| !v.is_null()) {
        set_json(&mut next, "itipDtstamp", d.clone());
    }
    next
}

/// The stored event with the sender's participation status from their reply.
fn replied(current: &Value, incoming: &Value, sender: &str) -> Value {
    let mut next = current.clone();
    let answer = participant(incoming, sender)
        .and_then(|p| p.get("participationStatus").cloned())
        .unwrap_or_else(|| json!("needs-action"));
    if let Some(parts) = next.get_mut("participants").and_then(Value::as_object_mut)
        && let Some(entry) = parts
            .iter_mut()
            .find(|(k, p)| {
                same_address(k, sender)
                    || p.get("email")
                        .and_then(Value::as_str)
                        .is_some_and(|e| same_address(e, sender))
            })
            .map(|(_, p)| p)
        && let Some(obj) = entry.as_object_mut()
    {
        obj.insert("participationStatus".into(), answer);
    }
    next
}

/// The participant entry for `address` (by key or `email`), any case.
fn participant<'a>(ev: &'a Value, address: &str) -> Option<&'a Value> {
    ev.get("participants")?
        .as_object()?
        .iter()
        .find_map(|(k, p)| {
            let email = p.get("email").and_then(Value::as_str).unwrap_or(k);
            (same_address(k, address) || same_address(email, address)).then_some(p)
        })
}

/// The address of the participant whose role is organizer; no fallback.
fn organizer_of(ev: &Value) -> Option<String> {
    ev.get("participants")?
        .as_object()?
        .iter()
        .find_map(|(k, p)| {
            (p.get("role").and_then(Value::as_str) == Some("organizer")).then(|| {
                p.get("email")
                    .and_then(Value::as_str)
                    .filter(|e| !e.is_empty())
                    .unwrap_or(k)
                    .to_string()
            })
        })
}

fn same_address(a: &str, b: &str) -> bool {
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

fn sequence(ev: &Value) -> i64 {
    ev.get("sequence").and_then(Value::as_i64).unwrap_or(0)
}

fn status_of(ev: &Value) -> &str {
    ev.get("status").and_then(Value::as_str).unwrap_or("")
}

/// The `From` address and the first calendar part of a message.
fn read_invitation(raw: &[u8]) -> Option<Invitation> {
    let message = MessageParser::default().parse(raw)?;
    let from = message.from()?;
    // One sender only: a list in `From` does not say who sent the message.
    if from.iter().count() != 1 {
        return None;
    }
    let sender = from.first()?.address()?.trim().to_ascii_lowercase();
    if sender.is_empty() {
        return None;
    }
    let ics = message.parts.iter().find_map(|part| {
        let ct = part.content_type()?;
        let kind = format!("{}/{}", ct.ctype(), ct.subtype().unwrap_or("")).to_ascii_lowercase();
        if kind != "text/calendar" && kind != "application/ics" {
            return None;
        }
        match &part.body {
            PartType::Text(t) => Some(t.to_string()),
            PartType::Binary(b) | PartType::InlineBinary(b) => {
                Some(String::from_utf8_lossy(b).into_owned())
            }
            _ => None,
        }
    })?;
    Some(Invitation { sender, ics })
}

/// The first VEVENT's DTSTAMP in UTC basic form (`20261020T090000Z`), or `None`
/// when it is absent or not in that form (then only SEQUENCE is compared).
fn dtstamp_of(ics: &str) -> Option<String> {
    // Unfold continuation lines (RFC 5545 §3.1) before reading properties.
    let unfolded = ics
        .replace("\r\n ", "")
        .replace("\r\n\t", "")
        .replace("\n ", "");
    let mut in_event = false;
    for line in unfolded.lines() {
        let line = line.trim_end_matches('\r');
        if line.eq_ignore_ascii_case("BEGIN:VEVENT") {
            in_event = true;
        } else if line.eq_ignore_ascii_case("END:VEVENT") {
            return None;
        } else if in_event {
            let Some((name, value)) = line.split_once(':') else {
                continue;
            };
            let prop = name.split(';').next().unwrap_or("");
            if prop.eq_ignore_ascii_case("DTSTAMP") {
                let v = value.trim();
                let ok = v.len() == 16
                    && v.ends_with('Z')
                    && v.as_bytes()[8] == b'T'
                    && v[..8].bytes().all(|b| b.is_ascii_digit())
                    && v[9..15].bytes().all(|b| b.is_ascii_digit());
                return ok.then(|| v.to_string());
            }
        }
    }
    None
}

fn invalid(msg: impl std::fmt::Display) -> Value {
    json!({ "type": "invalidArguments", "description": msg.to_string() })
}

fn not_found(email_id: &str) -> Value {
    json!({ "type": "notFound", "description": format!("no message {email_id:?} in this account") })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dtstamp_is_read_from_the_event_and_only_in_utc_basic_form() {
        let ics = "BEGIN:VCALENDAR\r\nMETHOD:REQUEST\r\nBEGIN:VEVENT\r\nUID:a\r\n\
                   DTSTAMP:20261020T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        assert_eq!(dtstamp_of(ics).as_deref(), Some("20261020T090000Z"));
        let local = ics.replace("20261020T090000Z", "20261020T090000");
        assert_eq!(dtstamp_of(&local), None);
        let folded = ics.replace("DTSTAMP:20261020T09", "DTSTAMP:20261020T09\r\n ");
        assert_eq!(dtstamp_of(&folded).as_deref(), Some("20261020T090000Z"));
        assert_eq!(dtstamp_of("BEGIN:VEVENT\r\nUID:a\r\nEND:VEVENT\r\n"), None);
    }

    #[test]
    fn addresses_compare_without_case_or_mailto() {
        assert!(same_address("Org@Example.com", "mailto:org@example.com"));
        assert!(!same_address("", ""));
        assert!(!same_address("a@example.com", "b@example.com"));
    }
}
