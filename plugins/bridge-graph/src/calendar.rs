//! Calendar over Graph — calendars (incl. shared), event delta sync, room resources,
//! and free/busy. The frozen `mailwoman:plugin` WIT world exposes NO calendar export
//! interface, so these are not reachable across the plugin boundary today (see the
//! e10 report's WIT-ABI friction note for e11/e12). They ARE fully implemented and
//! fixture-tested here so the mapping is ready the moment a calendar WIT seam lands.

use crate::graph::{GraphClient, Result, Transport};
use crate::model::{CalendarsResponse, EventsDeltaResponse, GetScheduleResponse, RoomsResponse};
use crate::types::SyncCursor;

/// A calendar the account can see (own or shared).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarInfo {
    pub id: String,
    pub name: String,
    pub can_edit: bool,
    pub owner: Option<String>,
    /// True when the owner differs from the account (a shared/delegated calendar).
    pub shared: bool,
}

/// A room resource bookable in the tenant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomInfo {
    pub name: String,
    pub email: String,
    pub capacity: Option<u32>,
    pub building: Option<String>,
}

/// A single event as the bridge surfaces it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventInfo {
    pub id: String,
    pub subject: String,
    pub start: Option<String>,
    pub end: Option<String>,
    pub location: Option<String>,
    pub all_day: bool,
}

/// An event delta: live events + removed ids + the next opaque cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventDelta {
    pub events: Vec<EventInfo>,
    pub removed: Vec<String>,
    pub next_cursor: SyncCursor,
}

/// `GET /me/calendars` → the account's calendars, flagging shared ones.
pub fn list_calendars<T: Transport>(
    client: &GraphClient<'_, T>,
    account: &str,
) -> Result<Vec<CalendarInfo>> {
    let resp: CalendarsResponse = client.get_json("/me/calendars")?;
    Ok(resp
        .value
        .into_iter()
        .map(|c| {
            let owner = c.owner.and_then(|o| o.address);
            let shared = owner
                .as_deref()
                .map(|o| !o.eq_ignore_ascii_case(account))
                .unwrap_or(false);
            CalendarInfo {
                id: c.id,
                name: c.name.unwrap_or_default(),
                can_edit: c.can_edit.unwrap_or(false),
                owner,
                shared,
            }
        })
        .collect())
}

/// `GET /me/events/delta` (or the stored deltaLink) → an incremental event delta.
pub fn sync_events<T: Transport>(
    client: &GraphClient<'_, T>,
    cursor: &SyncCursor,
) -> Result<EventDelta> {
    let path = if cursor.opaque.is_empty() {
        "/me/events/delta?$select=subject,start,end,location,isAllDay".to_string()
    } else {
        String::from_utf8_lossy(&cursor.opaque).into_owned()
    };
    let page: EventsDeltaResponse = client.get_json(&path)?;
    let mut events = Vec::new();
    let mut removed = Vec::new();
    for e in page.value {
        if e.removed.is_some() {
            removed.push(e.id);
        } else {
            events.push(EventInfo {
                id: e.id,
                subject: e.subject.unwrap_or_default(),
                start: e.start.and_then(|d| d.date_time),
                end: e.end.and_then(|d| d.date_time),
                location: e.location.and_then(|l| l.display_name),
                all_day: e.is_all_day.unwrap_or(false),
            });
        }
    }
    let next = page
        .delta_link
        .or(page.next_link)
        .unwrap_or_default()
        .into_bytes();
    Ok(EventDelta {
        events,
        removed,
        next_cursor: SyncCursor { opaque: next },
    })
}

/// `GET /places/microsoft.graph.room` → bookable room resources.
pub fn find_rooms<T: Transport>(client: &GraphClient<'_, T>) -> Result<Vec<RoomInfo>> {
    let resp: RoomsResponse = client.get_json("/places/microsoft.graph.room")?;
    Ok(resp
        .value
        .into_iter()
        .map(|r| RoomInfo {
            name: r.display_name.unwrap_or_default(),
            email: r.email_address.unwrap_or_default(),
            capacity: r.capacity,
            building: r.building,
        })
        .collect())
}

/// `POST /me/calendar/getSchedule` → the per-attendee availability view strings.
pub fn get_schedule<T: Transport>(
    client: &GraphClient<'_, T>,
    schedules: &[String],
    start: &str,
    end: &str,
) -> Result<Vec<(String, String)>> {
    let body = serde_json::json!({
        "schedules": schedules,
        "startTime": { "dateTime": start, "timeZone": "UTC" },
        "endTime": { "dateTime": end, "timeZone": "UTC" },
        "availabilityViewInterval": 60,
    });
    let resp: GetScheduleResponse = client.post_json("/me/calendar/getSchedule", &body)?;
    Ok(resp
        .value
        .into_iter()
        .map(|s| {
            (
                s.schedule_id.unwrap_or_default(),
                s.availability_view.unwrap_or_default(),
            )
        })
        .collect())
}
