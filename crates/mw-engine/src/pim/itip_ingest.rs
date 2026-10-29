//! `CalendarEvent/itip` (26.20 t28-e21, t53): read the calendar part of a
//! received message, say what applying it would do, and apply it only when
//! asked.
//!
//! `{emailId, blobId?, apply?, calendarId?, uids?}` → `{accountId, emailId,
//! method, sender, organizer, me, event, items, todos, eventId, stored,
//! outcome, applied}`. The message is read from the account's own store (an id
//! of another account does not resolve), and the sender is its `From`
//! address: the client names the message, never who sent it. `blobId` names
//! one part of that message (`<emailId>.<n>`, an `.ics` attachment); without
//! it the first calendar part is read (`text/calendar`, `application/ics`, or
//! a file named `*.ics`). The part is bounded (`mw_ics::MAX_CALENDAR_BYTES`,
//! `MAX_CALENDAR_ITEMS`) and nothing it names is fetched.
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
//! - COUNTER is shown, not applied;
//! - PUBLISH, or a calendar file with no METHOD, is a set of events to add:
//!   outcome `publish` (or `current` when every one is already held), applied
//!   by the user's action as an import that never overwrites a held UID.
//!
//! Answering an invitation is `CalendarEvent/respond {emailId, action, …}`
//! ([`Engine::event_respond_to_message`]): it runs the same guards, stores
//! the answer, and sends the REPLY from the address that was invited.

use mail_parser::{MessageParser, MimeHeaders, PartType};
use serde_json::{Value, json};

use crate::account::AccountRuntime;
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;

use super::events::{
    attendee_view, method_name, normalize_event, reply_status, set_json, set_participation,
    set_str, title_of,
};
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
    /// PUBLISH or a plain calendar file with an event not yet held.
    Publish,
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
            Outcome::Publish => "publish",
        }
    }

    fn applies(self) -> bool {
        matches!(
            self,
            Outcome::New | Outcome::Update | Outcome::Cancel | Outcome::Reply | Outcome::Publish
        )
    }
}

/// The calendar content of a received message.
struct Received {
    /// The single `From` address, lower-cased.
    sender: String,
    /// The `METHOD`, upper-cased (`None` for a plain calendar file).
    method: Option<String>,
    /// The VEVENT projections, each with `itipDtstamp` when it has one.
    events: Vec<Value>,
    /// How many VTODOs the part also holds (shown, not imported).
    todos: usize,
}

impl Engine {
    /// Read the calendar part `args` names (`emailId`, optional `blobId`).
    async fn received_calendar(
        &self,
        account_id: &str,
        args: &Value,
    ) -> std::result::Result<Received, Value> {
        let Some(email_id) = args.get("emailId").and_then(Value::as_str) else {
            return Err(invalid("CalendarEvent/itip requires emailId"));
        };
        // A message id only: not one of its parts, not an upload.
        if email_id.is_empty() || email_id.contains('.') || email_id.starts_with('U') {
            return Err(not_found(email_id));
        }
        let part_blob = match args.get("blobId").and_then(Value::as_str) {
            None => None,
            // A part of this message, by its index: never another message's
            // part, never an upload.
            Some(b) => match b.strip_prefix(email_id).and_then(|r| r.strip_prefix('.')) {
                Some(idx) if !idx.is_empty() && idx.bytes().all(|c| c.is_ascii_digit()) => {
                    Some(b.to_string())
                }
                _ => return Err(not_found(b)),
            },
        };
        let raw = match self.fetch_blob(account_id, email_id).await {
            Ok(Some(blob)) => blob.bytes,
            Ok(None) => return Err(not_found(email_id)),
            Err(e) => return Err(server_fail(e)),
        };
        let Some(sender) = single_sender(&raw) else {
            return Err(invalid("the message has no single sender address"));
        };
        let bytes = match part_blob {
            Some(blob_id) => match self.fetch_blob(account_id, &blob_id).await {
                Ok(Some(blob)) => blob.bytes,
                Ok(None) => return Err(not_found(&blob_id)),
                Err(e) => return Err(server_fail(e)),
            },
            None => match first_calendar_part(&raw) {
                Some(b) => b,
                None => return Err(invalid("the message has no calendar part")),
            },
        };
        let msg = match mw_ics::read_calendar_message(&bytes) {
            Ok(m) => m,
            Err(e) => return Err(invalid(format!("the calendar part cannot be read: {e}"))),
        };
        let todos = msg.items.iter().filter(|i| i.component == "VTODO").count();
        let events: Vec<Value> = msg
            .items
            .into_iter()
            .filter(|i| i.component == "VEVENT")
            .map(|i| i.json)
            .collect();
        if events.is_empty() {
            return Err(invalid("the calendar part has no event"));
        }
        Ok(Received {
            sender,
            method: msg.method,
            events,
            todos,
        })
    }

    pub(crate) async fn event_itip(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let apply = args.get("apply").and_then(Value::as_bool).unwrap_or(false);
        let email_id = args
            .get("emailId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let received = match self.received_calendar(account_id, args).await {
            Ok(r) => r,
            Err(e) => return e,
        };
        let Some(method) = received.method.as_deref().and_then(ItipMethod::from_name) else {
            return self
                .itip_publish(account_id, rt, args, &email_id, received, apply)
                .await;
        };
        let mut incoming = received.events.into_iter().next().unwrap_or_default();
        let uid = incoming
            .get("uid")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if uid.is_empty() {
            return invalid("the calendar part has no UID");
        }
        if incoming.get("itipDtstamp").is_none() {
            set_json(&mut incoming, "itipDtstamp", Value::Null);
        }

        let stored = match self.event_by_uid(account_id, &uid).await {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        let stored_json = stored.as_ref().map(|row| self.event_row_to_json(row));
        let outcome = decide(method, &received.sender, &incoming, stored_json.as_ref());
        let me = match method {
            ItipMethod::Request | ItipMethod::Cancel => {
                self.invited_address(account_id, rt, &incoming).await
            }
            _ => None,
        };
        // For a REPLY: what its sender answered.
        let answer = (method == ItipMethod::Reply)
            .then(|| participant(&incoming, &received.sender))
            .flatten()
            .and_then(|p| p.get("participationStatus").cloned());

        let mut out = json!({
            "accountId": account_id,
            "emailId": email_id,
            "method": method_name(method),
            "sender": received.sender,
            "organizer": organizer_of(&incoming),
            "me": me,
            "answer": answer,
            "event": incoming,
            "items": [{
                "event": incoming,
                "eventId": stored.as_ref().map(|r| r.id.clone()),
                "calendarId": stored.as_ref().map(|r| r.calendar_id.clone()),
            }],
            "todos": received.todos,
            "eventId": stored.as_ref().map(|r| r.id.clone()),
            "stored": stored_json.as_ref().map(|s| stored_summary(s, me.as_deref())),
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
                    _ => replied(&current, &out["event"], &received.sender),
                };
                self.itip_replace(account_id, rt, &row, next).await
            }
            _ => return out,
        };
        match applied {
            Ok(id) => {
                out["eventId"] = json!(id);
                out["items"][0]["eventId"] = json!(id);
                out["applied"] = json!(true);
                if let Ok(Some(row)) = self.owned_event(account_id, &id).await {
                    let json = self.event_row_to_json(&row);
                    out["items"][0]["calendarId"] = json!(row.calendar_id);
                    out["stored"] = stored_summary(&json, me.as_deref());
                }
                self.broadcast_state(account_id).await;
                out
            }
            Err(e) => server_fail(e),
        }
    }

    /// PUBLISH, or a calendar file with no METHOD: events to add, by the
    /// user's action, never over an event already held (any calendar).
    async fn itip_publish(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
        email_id: &str,
        received: Received,
        apply: bool,
    ) -> Value {
        let mut items = Vec::new();
        for ev in &received.events {
            let uid = ev.get("uid").and_then(Value::as_str).unwrap_or_default();
            let stored = if uid.is_empty() {
                None
            } else {
                match self.event_by_uid(account_id, uid).await {
                    Ok(v) => v,
                    Err(e) => return server_fail(e),
                }
            };
            items.push(json!({
                "event": ev,
                "eventId": stored.as_ref().map(|r| r.id.clone()),
                "calendarId": stored.as_ref().map(|r| r.calendar_id.clone()),
            }));
        }
        let all_held = items.iter().all(|i| !i["eventId"].is_null());
        let outcome = if all_held {
            Outcome::Current
        } else {
            Outcome::Publish
        };
        let mut out = json!({
            "accountId": account_id,
            "emailId": email_id,
            "method": received.method,
            "sender": received.sender,
            "organizer": organizer_of(&received.events[0]),
            "me": Value::Null,
            "answer": Value::Null,
            "event": received.events[0],
            "items": items,
            "todos": received.todos,
            "eventId": Value::Null,
            "stored": Value::Null,
            "outcome": outcome.wire(),
            "applied": false,
        });
        out["eventId"] = out["items"][0]["eventId"].clone();
        if !apply || outcome != Outcome::Publish {
            return out;
        }
        // Only the UIDs asked for (`uids`), when given; never one already held.
        let wanted: Option<Vec<String>> = args.get("uids").and_then(Value::as_array).map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect()
        });
        let mut picked = Vec::new();
        let mut positions = Vec::new();
        for (n, item) in out["items"].as_array().into_iter().flatten().enumerate() {
            let uid = item["event"]["uid"].as_str().unwrap_or_default();
            if !item["eventId"].is_null()
                || wanted.as_ref().is_some_and(|w| !w.iter().any(|u| u == uid))
            {
                continue;
            }
            let mut ev = item["event"].clone();
            if let Some(obj) = ev.as_object_mut() {
                obj.remove("itipDtstamp");
            }
            normalize_event(&mut ev);
            picked.push(ev);
            positions.push(n);
        }
        let calendar_id = match args.get("calendarId").and_then(Value::as_str) {
            Some(c) => c.to_string(),
            None => match self.ensure_default_calendar(account_id, "VEVENT").await {
                Ok(c) => c,
                Err(e) => return server_fail(e),
            },
        };
        let (created, _skipped) = self
            .persist_new_events(account_id, &calendar_id, picked, Some(rt))
            .await;
        // `persist_new_events` keeps the batch order and skips only failures,
        // so the created ids are matched back by UID, not by position.
        for id in &created {
            if let Ok(Some(row)) = self.owned_event(account_id, id).await
                && let Some(n) = positions
                    .iter()
                    .copied()
                    .find(|n| out["items"][*n]["event"]["uid"].as_str() == Some(row.uid.as_str()))
            {
                out["items"][n]["eventId"] = json!(row.id);
                out["items"][n]["calendarId"] = json!(row.calendar_id);
            }
        }
        out["eventId"] = out["items"][0]["eventId"].clone();
        out["imported"] = json!(created);
        out["applied"] = json!(!created.is_empty());
        if !created.is_empty() {
            self.broadcast_state(account_id).await;
        }
        out
    }

    /// `CalendarEvent/respond {emailId, blobId?, action, calendarId?,
    /// comment?, sendResponse?, counter?}`: answer the invitation in a
    /// received message.
    ///
    /// The message must be a REQUEST that `CalendarEvent/itip` would apply
    /// (`new`, `update` or `current`): a stale one is refused (`code:
    /// "stale"`), and so is one about a stored event its sender does not
    /// organize (`notOrganizer`). The answering attendee is the address the
    /// account was invited at ([`Engine::invited_address`]); an invitation
    /// listing none of the account's addresses is refused (`notInvited`) and
    /// nothing is sent. Accept and tentative store the event (a new one in
    /// `calendarId` or the default calendar) with the answer; decline records
    /// the answer on a stored event and adds nothing; counter proposes
    /// `counter.start`/`counter.duration` and changes nothing stored. The
    /// REPLY (or COUNTER) goes to the organizer unless `sendResponse` is
    /// false.
    pub(crate) async fn event_respond_to_message(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let action = args
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("accept");
        let partstat = match action {
            "accept" => "accepted",
            "decline" => "declined",
            "tentative" | "counter" => "tentative",
            other => return invalid(format!("unknown respond action {other}")),
        };
        let email_id = args
            .get("emailId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let received = match self.received_calendar(account_id, args).await {
            Ok(r) => r,
            Err(e) => return e,
        };
        if received.method.as_deref() != Some("REQUEST") {
            return refusal(
                "notRequest",
                "only an invitation (METHOD:REQUEST) is answered",
            );
        }
        let incoming = received.events.into_iter().next().unwrap_or_default();
        let uid = incoming
            .get("uid")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if uid.is_empty() {
            return invalid("the calendar part has no UID");
        }
        let stored = match self.event_by_uid(account_id, &uid).await {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        let stored_json = stored.as_ref().map(|row| self.event_row_to_json(row));
        let outcome = decide(
            ItipMethod::Request,
            &received.sender,
            &incoming,
            stored_json.as_ref(),
        );
        match outcome {
            Outcome::New | Outcome::Update | Outcome::Current => {}
            Outcome::Stale => {
                return refusal(
                    "stale",
                    "a newer version of this invitation is in the calendar",
                );
            }
            Outcome::NotOrganizer => {
                return refusal("notOrganizer", "the sender is not the event's organizer");
            }
            _ => return invalid("this invitation cannot be answered"),
        }
        let Some(me) = self.invited_address(account_id, rt, &incoming).await else {
            return refusal(
                "notInvited",
                "the invitation does not list an address this account sends as",
            );
        };
        let Some(organizer) = organizer_of(&incoming) else {
            return refusal("noOrganizer", "the invitation names no organizer");
        };

        // The event as the answer leaves it: the organizer's version (newer
        // or current) with this attendee's status.
        let mut answered = match (outcome, &stored_json) {
            (Outcome::Update, Some(current)) => updated(current, incoming.clone()),
            (Outcome::Current, Some(current)) => current.clone(),
            _ => incoming.clone(),
        };
        set_participation(&mut answered, &me, partstat);

        let mut event_id = stored.as_ref().map(|r| r.id.clone());
        let store_it = match action {
            "counter" => false,
            "decline" => stored.is_some(),
            _ => true,
        };
        if store_it {
            let result = match &stored {
                None => {
                    self.itip_add(account_id, rt, args, &uid, answered.clone())
                        .await
                }
                Some(row) => {
                    self.itip_replace(account_id, rt, row, answered.clone())
                        .await
                }
            };
            match result {
                Ok(id) => event_id = Some(id),
                Err(e) => return server_fail(e),
            }
        }

        let send = args
            .get("sendResponse")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let mut sent = false;
        if send {
            let comment = args.get("comment").and_then(Value::as_str);
            let built = if action == "counter" {
                let mut proposal = attendee_view(&answered, &me);
                let counter = args.get("counter").cloned().unwrap_or(Value::Null);
                if let Some(start) = counter.get("start").and_then(Value::as_str) {
                    set_str(&mut proposal, "start", start);
                }
                if let Some(dur) = counter.get("duration").and_then(Value::as_str) {
                    set_str(&mut proposal, "duration", dur);
                }
                if let Some(c) = comment {
                    set_str(&mut proposal, "description", c);
                }
                mw_ics::build_itip(&proposal, ItipMethod::Counter).map(|ics| ("COUNTER", ics))
            } else {
                mw_ics::build_itip_reply(&answered, &me, reply_status(partstat), comment)
                    .map(|ics| ("REPLY", ics))
            };
            match built {
                Ok((method, ics)) => {
                    sent = self
                        .submit_itip(
                            rt,
                            &me,
                            std::slice::from_ref(&organizer),
                            method,
                            &title_of(&answered),
                            &ics,
                        )
                        .await
                        .is_ok();
                }
                Err(e) => tracing::warn!("iTIP answer not built: {e}"),
            }
        }
        if store_it {
            self.broadcast_state(account_id).await;
        }
        let calendar_id = match &event_id {
            Some(id) => match self.owned_event(account_id, id).await {
                Ok(Some(row)) => Some(row.calendar_id),
                _ => None,
            },
            None => None,
        };
        json!({
            "accountId": account_id,
            "emailId": email_id,
            "action": action,
            "participationStatus": partstat,
            "eventId": event_id,
            "calendarId": calendar_id,
            "from": me,
            "to": organizer,
            "sendResponse": send,
            "sent": sent,
        })
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
            Some(c) => match self.owned_calendar(account_id, c).await? {
                Some(_) => c.to_string(),
                None => {
                    return Err(crate::backend::EngineError::Protocol(format!(
                        "unknown calendar {c}"
                    )));
                }
            },
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

/// What the card shows of a stored event: where it is, its status, and the
/// account's own answer (when `me` is known).
fn stored_summary(stored: &Value, me: Option<&str>) -> Value {
    let mine = me
        .and_then(|m| participant(stored, m))
        .and_then(|p| p.get("participationStatus").cloned())
        .unwrap_or(Value::Null);
    json!({
        "calendarId": stored.get("calendarId"),
        "status": stored.get("status"),
        "sequence": sequence(stored),
        "participationStatus": mine,
        "participants": stored.get("participants"),
    })
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

/// The single `From` address of a message, lower-cased. A list in `From`
/// does not say who sent the message.
fn single_sender(raw: &[u8]) -> Option<String> {
    let message = MessageParser::default().parse(raw)?;
    let from = message.from()?;
    if from.iter().count() != 1 {
        return None;
    }
    let sender = from.first()?.address()?.trim().to_ascii_lowercase();
    (!sender.is_empty()).then_some(sender)
}

/// The first calendar part of a message: `text/calendar`, `application/ics`,
/// or an attachment named `*.ics` (often sent as `application/octet-stream`).
fn first_calendar_part(raw: &[u8]) -> Option<Vec<u8>> {
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

fn invalid(msg: impl std::fmt::Display) -> Value {
    json!({ "type": "invalidArguments", "description": msg.to_string() })
}

/// An `invalidArguments` the card tells apart by its `code`.
fn refusal(code: &str, msg: &str) -> Value {
    json!({ "type": "invalidArguments", "code": code, "description": msg })
}

fn not_found(email_id: &str) -> Value {
    json!({ "type": "notFound", "description": format!("no message {email_id:?} in this account") })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MSG: &str = "From: Olga <olga@example.com>
To: me@example.org
Subject: x
        MIME-Version: 1.0
Content-Type: multipart/mixed; boundary=b

        --b
Content-Type: text/plain

hi
        --b
Content-Type: application/octet-stream; name=invite.ics
        Content-Disposition: attachment; filename=invite.ics

        BEGIN:VCALENDAR
BEGIN:VEVENT
UID:a
END:VEVENT
END:VCALENDAR
        --b--
";

    #[test]
    fn an_ics_attachment_sent_as_octet_stream_is_the_calendar_part() {
        let part = first_calendar_part(MSG.as_bytes()).unwrap();
        assert!(String::from_utf8_lossy(&part).contains("UID:a"));
        assert_eq!(
            single_sender(MSG.as_bytes()).as_deref(),
            Some("olga@example.com")
        );
    }

    #[test]
    fn a_message_from_two_senders_has_no_single_sender() {
        let two = MSG.replace(
            "From: Olga <olga@example.com>",
            "From: a@example.com, b@example.com",
        );
        assert_eq!(single_sender(two.as_bytes()), None);
    }

    #[test]
    fn addresses_compare_without_case_or_mailto() {
        assert!(same_address("Org@Example.com", "mailto:org@example.com"));
        assert!(!same_address("", ""));
        assert!(!same_address("a@example.com", "b@example.com"));
    }
}
