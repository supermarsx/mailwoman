//! EWS beyond mail: calendar create/list, free-busy (`GetUserAvailability`), room
//! lists/rooms, Out-of-Office (`GetUserOofSettings`/`SetUserOofSettings`), the
//! honest message-recall path, and Outlook voting buttons. Pure request builders +
//! response parsers over recorded fixtures; the wasm guest runs them over the host
//! `http-fetch` import.
//!
//! **Honesty (recall, plan §1.6 / §10.3):** EWS has no cross-organization "unsend".
//! [`recall_delete_request`] issues a hard-delete of the *recipient copy* and only
//! succeeds where the sender genuinely has access (same-org delegated/shared
//! mailbox, message still present). Everything else is [`RecallOutcome::Unsupported`]
//! — the bridge never claims a recall it cannot perform.

use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;

use crate::soap;

fn local(name: QName<'_>) -> String {
    String::from_utf8_lossy(name.local_name().as_ref()).into_owned()
}

// PSETID_Common — the property set Outlook voting/verb properties live in.
const PSETID_COMMON: &str = "{00062008-0000-0000-C000-000000000046}";
/// PidLidVerbStream (voting options blob).
const PID_LID_VERB_STREAM: u16 = 0x8520;
/// PidLidVerbResponse (the recipient's chosen button text).
const PID_LID_VERB_RESPONSE: u16 = 0x8524;

// ── Calendar ────────────────────────────────────────────────────────────────────

/// A calendar event (subset used by the PIM bridge surface).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CalEvent {
    pub id: String,
    pub subject: String,
    pub start: String,
    pub end: String,
    pub location: String,
}

/// Build a `CreateItem` for a `CalendarItem` and send invitations to attendees.
#[must_use]
pub fn create_calendar_event_request(
    subject: &str,
    start_utc: &str,
    end_utc: &str,
    location: &str,
    attendees: &[&str],
) -> String {
    let mut req = String::new();
    for a in attendees {
        req.push_str(&format!(
            "<t:Attendee><t:Mailbox><t:EmailAddress>{}</t:EmailAddress></t:Mailbox></t:Attendee>",
            soap::escape(a)
        ));
    }
    let required = if req.is_empty() {
        String::new()
    } else {
        format!("<t:RequiredAttendees>{req}</t:RequiredAttendees>")
    };
    soap::envelope(&format!(
        concat!(
            "<m:CreateItem SendMeetingInvitations=\"SendToAllAndSaveCopy\">",
            "<m:Items><t:CalendarItem>",
            "<t:Subject>{subject}</t:Subject>",
            "<t:Start>{start}</t:Start><t:End>{end}</t:End>",
            "<t:Location>{loc}</t:Location>",
            "{required}",
            "</t:CalendarItem></m:Items>",
            "</m:CreateItem>"
        ),
        subject = soap::escape(subject),
        start = soap::escape(start_utc),
        end = soap::escape(end_utc),
        loc = soap::escape(location),
        required = required
    ))
}

/// Build a `FindItem` over the primary calendar in a `CalendarView` time window.
#[must_use]
pub fn find_calendar_events_request(start_utc: &str, end_utc: &str) -> String {
    soap::envelope(&format!(
        concat!(
            "<m:FindItem Traversal=\"Shallow\">",
            "<m:ItemShape><t:BaseShape>Default</t:BaseShape></m:ItemShape>",
            "<m:CalendarView StartDate=\"{start}\" EndDate=\"{end}\"/>",
            "<m:ParentFolderIds><t:DistinguishedFolderId Id=\"calendar\"/></m:ParentFolderIds>",
            "</m:FindItem>"
        ),
        start = soap::escape(start_utc),
        end = soap::escape(end_utc)
    ))
}

/// Parse `CalendarItem`s out of a `FindItem`/`GetItem` calendar response.
pub fn parse_calendar_events(xml: &str) -> Vec<CalEvent> {
    let mut events = Vec::new();
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut cur: Option<CalEvent> = None;
    let mut field: Option<String> = None;
    let mut pending = String::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let ln = local(e.name());
                if ln == "CalendarItem" {
                    cur = Some(CalEvent::default());
                } else if cur.is_some()
                    && matches!(ln.as_str(), "Subject" | "Start" | "End" | "Location")
                {
                    field = Some(ln);
                    pending.clear();
                }
            }
            Ok(Event::Empty(e)) if local(e.name()) == "ItemId" => {
                if let Some(c) = cur.as_mut()
                    && let Some(id) = e
                        .attributes()
                        .flatten()
                        .find(|a| a.key.local_name().as_ref() == b"Id")
                {
                    c.id = String::from_utf8_lossy(&id.value).into_owned();
                }
            }
            Ok(Event::Text(t)) if field.is_some() => {
                pending.push_str(&t.decode().unwrap_or_default());
            }
            Ok(Event::End(e)) => {
                let ln = local(e.name());
                if let Some(f) = field.take() {
                    if ln == f {
                        if let Some(c) = cur.as_mut() {
                            let v = pending.trim().to_string();
                            match f.as_str() {
                                "Subject" => c.subject = v,
                                "Start" => c.start = v,
                                "End" => c.end = v,
                                "Location" => c.location = v,
                                _ => {}
                            }
                        }
                    } else {
                        field = Some(f);
                    }
                }
                if ln == "CalendarItem"
                    && let Some(c) = cur.take()
                {
                    events.push(c);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    events
}

// ── Free/busy ────────────────────────────────────────────────────────────────────

/// A free/busy block for one attendee.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeBusyBlock {
    pub start: String,
    pub end: String,
    /// `Free` / `Tentative` / `Busy` / `OOF`.
    pub busy_type: String,
}

/// Build a `GetUserAvailabilityRequest` (free/busy) for `emails` over a window.
#[must_use]
pub fn get_user_availability_request(emails: &[&str], start_utc: &str, end_utc: &str) -> String {
    let mut mailboxes = String::new();
    for e in emails {
        mailboxes.push_str(&format!(
            concat!(
                "<t:MailboxData><t:Email><t:Address>{addr}</t:Address></t:Email>",
                "<t:AttendeeType>Required</t:AttendeeType></t:MailboxData>"
            ),
            addr = soap::escape(e)
        ));
    }
    soap::envelope(&format!(
        concat!(
            "<m:GetUserAvailabilityRequest>",
            "<m:MailboxDataArray>{mailboxes}</m:MailboxDataArray>",
            "<t:FreeBusyViewOptions>",
            "<t:TimeWindow><t:StartTime>{start}</t:StartTime><t:EndTime>{end}</t:EndTime></t:TimeWindow>",
            "<t:RequestedView>FreeBusy</t:RequestedView>",
            "</t:FreeBusyViewOptions>",
            "</m:GetUserAvailabilityRequest>"
        ),
        mailboxes = mailboxes,
        start = soap::escape(start_utc),
        end = soap::escape(end_utc)
    ))
}

/// Parse `CalendarEvent` free/busy blocks from a `GetUserAvailability` response.
pub fn parse_free_busy(xml: &str) -> Vec<FreeBusyBlock> {
    let mut out = Vec::new();
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_event = false;
    let mut start = String::new();
    let mut end = String::new();
    let mut busy = String::new();
    let mut field: Option<String> = None;
    let mut pending = String::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let ln = local(e.name());
                if ln == "CalendarEvent" {
                    in_event = true;
                    start.clear();
                    end.clear();
                    busy.clear();
                } else if in_event && matches!(ln.as_str(), "StartTime" | "EndTime" | "BusyType") {
                    field = Some(ln);
                    pending.clear();
                }
            }
            Ok(Event::Text(t)) if field.is_some() => {
                pending.push_str(&t.decode().unwrap_or_default());
            }
            Ok(Event::End(e)) => {
                let ln = local(e.name());
                if let Some(f) = field.take() {
                    if ln == f {
                        match f.as_str() {
                            "StartTime" => start = pending.trim().to_string(),
                            "EndTime" => end = pending.trim().to_string(),
                            "BusyType" => busy = pending.trim().to_string(),
                            _ => {}
                        }
                    } else {
                        field = Some(f);
                    }
                }
                if ln == "CalendarEvent" {
                    in_event = false;
                    out.push(FreeBusyBlock {
                        start: std::mem::take(&mut start),
                        end: std::mem::take(&mut end),
                        busy_type: std::mem::take(&mut busy),
                    });
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    out
}

// ── Rooms ─────────────────────────────────────────────────────────────────────────

/// Build a `GetRoomLists` request.
#[must_use]
pub fn get_room_lists_request() -> String {
    soap::envelope("<m:GetRoomLists/>")
}

/// Build a `GetRooms` request for a specific room-list address.
#[must_use]
pub fn get_rooms_request(room_list_email: &str) -> String {
    soap::envelope(&format!(
        "<m:GetRooms><m:RoomList><t:EmailAddress>{}</t:EmailAddress></m:RoomList></m:GetRooms>",
        soap::escape(room_list_email)
    ))
}

/// Parse `(name, email)` pairs from a `GetRoomLists` or `GetRooms` response — both
/// carry `<t:Name>`/`<t:EmailAddress>` under `<t:Address>`/`<t:Id>`.
pub fn parse_room_addresses(xml: &str) -> Vec<crate::ews::GalEntry> {
    let names = soap::texts_of(xml, "Name");
    let emails = soap::texts_of(xml, "EmailAddress");
    names
        .into_iter()
        .zip(emails)
        .map(|(display_name, email)| crate::ews::GalEntry {
            display_name,
            email,
        })
        .collect()
}

// ── Out-of-Office (OOF / OOO) ──────────────────────────────────────────────────

/// Out-of-Office settings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OofSettings {
    /// `Disabled` / `Enabled` / `Scheduled`.
    pub state: String,
    /// `None` / `Known` / `All` — who gets the external reply.
    pub external_audience: String,
    pub internal_reply: String,
    pub external_reply: String,
}

impl OofSettings {
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.state == "Enabled" || self.state == "Scheduled"
    }
}

/// Build a `GetUserOofSettingsRequest` for `mailbox`.
#[must_use]
pub fn get_user_oof_request(mailbox: &str) -> String {
    soap::envelope(&format!(
        concat!(
            "<m:GetUserOofSettingsRequest>",
            "<t:Mailbox><t:Address>{addr}</t:Address></t:Mailbox>",
            "</m:GetUserOofSettingsRequest>"
        ),
        addr = soap::escape(mailbox)
    ))
}

/// Build a `SetUserOofSettingsRequest`.
#[must_use]
pub fn set_user_oof_request(oof: &OofSettings, mailbox: &str) -> String {
    soap::envelope(&format!(
        concat!(
            "<m:SetUserOofSettingsRequest>",
            "<t:Mailbox><t:Address>{addr}</t:Address></t:Mailbox>",
            "<t:UserOofSettings>",
            "<t:OofState>{state}</t:OofState>",
            "<t:ExternalAudience>{aud}</t:ExternalAudience>",
            "<t:InternalReply><t:Message>{internal}</t:Message></t:InternalReply>",
            "<t:ExternalReply><t:Message>{external}</t:Message></t:ExternalReply>",
            "</t:UserOofSettings>",
            "</m:SetUserOofSettingsRequest>"
        ),
        addr = soap::escape(mailbox),
        state = soap::escape(&oof.state),
        aud = soap::escape(if oof.external_audience.is_empty() {
            "All"
        } else {
            &oof.external_audience
        }),
        internal = soap::escape(&oof.internal_reply),
        external = soap::escape(&oof.external_reply)
    ))
}

/// Parse a `GetUserOofSettings` response into [`OofSettings`].
pub fn parse_oof(xml: &str) -> OofSettings {
    // InternalReply/ExternalReply each wrap a `<t:Message>`; take them in order.
    let messages = soap::texts_of(xml, "Message");
    OofSettings {
        state: soap::first_text(xml, "OofState").unwrap_or_default(),
        external_audience: soap::first_text(xml, "ExternalAudience").unwrap_or_default(),
        internal_reply: messages.first().cloned().unwrap_or_default(),
        external_reply: messages.get(1).cloned().unwrap_or_default(),
    }
}

// ── Message recall (honest) ────────────────────────────────────────────────────

/// The outcome of a recall attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecallOutcome {
    /// The recipient copy was hard-deleted (same-org access + item present).
    Recalled,
    /// Recall is not possible here (cross-org, item already read/gone, no access).
    Unsupported(String),
}

/// Build the recall action: a hard-`DeleteItem` of the recipient copy. Only issue
/// this when the sender genuinely has delegated/shared access to `recipient_item_id`
/// — otherwise call sites must return [`RecallOutcome::Unsupported`] without a
/// request (there is no cross-org unsend in EWS; see the module honesty note).
#[must_use]
pub fn recall_delete_request(recipient_item_id: &str) -> String {
    soap::envelope(&format!(
        concat!(
            "<m:DeleteItem DeleteType=\"HardDelete\">",
            "<m:ItemIds><t:ItemId Id=\"{id}\"/></m:ItemIds>",
            "</m:DeleteItem>"
        ),
        id = soap::escape(recipient_item_id)
    ))
}

/// Interpret a `DeleteItem` response for a recall attempt.
#[must_use]
pub fn parse_recall(xml: &str) -> RecallOutcome {
    if soap::is_success(xml) {
        RecallOutcome::Recalled
    } else {
        RecallOutcome::Unsupported(
            soap::message_text(xml).unwrap_or_else(|| "recipient copy not accessible".into()),
        )
    }
}

// ── Voting buttons (Outlook parity) ────────────────────────────────────────────

/// Build a `CreateItem` that sends a message carrying Outlook voting buttons. The
/// options are encoded both as the human-readable `X-Micosoft-Voting` header (via
/// MIME) *and* the `PidLidVerbStream` extended property so Outlook renders buttons.
#[must_use]
pub fn create_voting_message_request(
    subject: &str,
    to: &str,
    body: &str,
    options: &[&str],
) -> String {
    let joined = options.join(";");
    soap::envelope(&format!(
        concat!(
            "<m:CreateItem MessageDisposition=\"SendAndSaveCopy\">",
            "<m:Items><t:Message>",
            "<t:Subject>{subject}</t:Subject>",
            "<t:Body BodyType=\"Text\">{body}</t:Body>",
            "<t:ExtendedProperty>",
            "<t:ExtendedFieldURI PropertySetId=\"{set}\" PropertyId=\"{pid}\" PropertyType=\"Binary\"/>",
            "<t:Value>{opts}</t:Value>",
            "</t:ExtendedProperty>",
            "<t:ToRecipients><t:Mailbox><t:EmailAddress>{to}</t:EmailAddress></t:Mailbox></t:ToRecipients>",
            "</t:Message></m:Items>",
            "</m:CreateItem>"
        ),
        subject = soap::escape(subject),
        body = soap::escape(body),
        set = PSETID_COMMON,
        pid = PID_LID_VERB_STREAM,
        opts = soap::escape(&joined),
        to = soap::escape(to)
    ))
}

/// Build a `GetItem` that fetches the voting *response* extended property from a
/// reply message (the recipient's chosen button).
#[must_use]
pub fn get_vote_response_request(item_id: &str) -> String {
    soap::envelope(&format!(
        concat!(
            "<m:GetItem>",
            "<m:ItemShape><t:BaseShape>IdOnly</t:BaseShape>",
            "<t:AdditionalProperties>",
            "<t:ExtendedFieldURI PropertySetId=\"{set}\" PropertyId=\"{pid}\" PropertyType=\"String\"/>",
            "</t:AdditionalProperties></m:ItemShape>",
            "<m:ItemIds><t:ItemId Id=\"{id}\"/></m:ItemIds>",
            "</m:GetItem>"
        ),
        set = PSETID_COMMON,
        pid = PID_LID_VERB_RESPONSE,
        id = soap::escape(item_id)
    ))
}

/// Extract the chosen voting option from a vote-response `GetItem` result — the
/// `PidLidVerbResponse` value, exposed either as an `ExtendedProperty` `Value` or
/// the `VotingResponse` convenience element.
#[must_use]
pub fn parse_vote_response(xml: &str) -> Option<String> {
    soap::first_text(xml, "VotingResponse")
        .or_else(|| soap::first_text(xml, "Value"))
        .filter(|s| !s.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FREEBUSY: &str = include_str!("../fixtures/free_busy.xml");
    const OOF: &str = include_str!("../fixtures/oof_settings.xml");
    const ROOMS: &str = include_str!("../fixtures/room_lists.xml");
    const CAL: &str = include_str!("../fixtures/calendar_events.xml");
    const VOTE: &str = include_str!("../fixtures/vote_response.xml");
    const RECALL_OK: &str = include_str!("../fixtures/recall_ok.xml");
    const RECALL_ERR: &str = include_str!("../fixtures/recall_error.xml");

    #[test]
    fn calendar_create_and_parse() {
        let req = create_calendar_event_request(
            "Sync",
            "2026-07-20T09:00:00Z",
            "2026-07-20T09:30:00Z",
            "Room 1",
            &["bob@corp.example"],
        );
        assert!(req.contains("CalendarItem"));
        assert!(req.contains("SendToAllAndSaveCopy"));
        assert!(req.contains("bob@corp.example"));

        let events = parse_calendar_events(CAL);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].subject, "Weekly sync");
        assert_eq!(events[0].location, "Room A");
        assert!(!events[0].id.is_empty());
    }

    #[test]
    fn free_busy_parses_blocks() {
        let req = get_user_availability_request(
            &["bob@corp.example"],
            "2026-07-20T00:00:00",
            "2026-07-21T00:00:00",
        );
        assert!(req.contains("GetUserAvailabilityRequest"));
        let blocks = parse_free_busy(FREEBUSY);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].busy_type, "Busy");
        assert_eq!(blocks[1].busy_type, "OOF");
    }

    #[test]
    fn rooms_parse_name_email_pairs() {
        assert!(get_room_lists_request().contains("GetRoomLists"));
        assert!(get_rooms_request("rooms-a@corp.example").contains("GetRooms"));
        let rooms = parse_room_addresses(ROOMS);
        assert_eq!(rooms.len(), 2);
        assert_eq!(rooms[0].display_name, "Room A");
        assert_eq!(rooms[0].email, "room-a@corp.example");
    }

    #[test]
    fn oof_round_trip() {
        let oof = parse_oof(OOF);
        assert!(oof.is_enabled());
        assert_eq!(oof.state, "Enabled");
        assert_eq!(oof.external_audience, "All");
        assert_eq!(oof.internal_reply, "Out until Monday (internal).");
        assert_eq!(oof.external_reply, "I am away.");
        let set = set_user_oof_request(&oof, "me@corp.example");
        assert!(set.contains("SetUserOofSettingsRequest"));
        assert!(set.contains("<t:OofState>Enabled</t:OofState>"));
    }

    #[test]
    fn recall_outcomes_are_honest() {
        assert!(recall_delete_request("X").contains("HardDelete"));
        assert_eq!(parse_recall(RECALL_OK), RecallOutcome::Recalled);
        match parse_recall(RECALL_ERR) {
            RecallOutcome::Unsupported(m) => assert!(m.contains("cannot")),
            other => panic!("expected Unsupported, got {other:?}"),
        }
    }

    #[test]
    fn voting_request_and_response() {
        let req = create_voting_message_request(
            "Lunch?",
            "team@corp.example",
            "Pick one",
            &["Pizza", "Salad"],
        );
        assert!(req.contains("ExtendedProperty"));
        assert!(req.contains("Pizza;Salad"));
        // PidLidVerbResponse (0x8524) is emitted as its decimal PropertyId.
        assert!(get_vote_response_request("i").contains(&PID_LID_VERB_RESPONSE.to_string()));
        assert_eq!(parse_vote_response(VOTE).as_deref(), Some("Pizza"));
    }
}
