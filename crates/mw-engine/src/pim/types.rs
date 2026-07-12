//! Frozen §2.1 PIM object types (Mailwoman-native, JSCalendar/JSContact-aligned,
//! camelCase). These are the contract every parallel builder and the web client
//! agree on — field names match §2.1 EXACTLY so a future JMAP-Calendars/Contacts
//! migration is mechanical (plan §1.1). They mirror `apps/web/src/api/pim-types.ts`.
//!
//! DTOs only: `Serialize`/`Deserialize` shapes with no behaviour. e8 constructs
//! them from the store + `mw-ics`; the free-form JSCalendar corners
//! (recurrence-rule / patch / trigger objects) stay `serde_json::Value` so the
//! `ical_raw` round-trip source of truth (plan risk #13) is never lossily
//! over-frozen.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A wall-clock local date-time without offset (JSCalendar `LocalDateTime`,
/// e.g. `"2026-07-12T09:00:00"`); the offset is carried by `timeZone`.
pub type LocalDateTime = String;
/// An IANA time-zone id (e.g. `"Europe/Lisbon"`), or `null` for floating.
pub type Tzid = String;
/// An ISO 8601 duration (e.g. `"PT1H"`).
pub type Iso8601Duration = String;
/// A date-only value (e.g. `"2026-07-12"`).
pub type Date = String;

// ── Calendars ───────────────────────────────────────────────────────────────

/// A calendar collection (§2.1). Mailwoman-native calendars have a `null`
/// `caldavUrl`; overlay calendars pull-only with `isReadOnlyOverlay = true`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Calendar {
    pub id: String,
    pub name: String,
    pub color: String,
    pub order: i64,
    pub is_visible: bool,
    pub is_subscribed: bool,
    /// `"default"` for the primary calendar, else `null`.
    pub role: Option<String>,
    pub share_with: Vec<CalendarShare>,
    pub caldav_url: Option<String>,
    pub sync_token: Option<String>,
    pub is_read_only_overlay: bool,
}

/// One `shareWith` grant on a calendar (Mailwoman-native ACL sharing, §11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarShare {
    pub principal: String,
    /// `"read"` | `"readWrite"`.
    pub access: String,
}

/// A calendar event (§2.1, JSCalendar-aligned). The web receives expanded
/// instances for a queried window plus the master for editing; expansion is
/// engine-side (`mw-ics` + `rrule`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: String,
    pub calendar_id: String,
    pub uid: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub locations: Vec<EventLocation>,
    pub start: LocalDateTime,
    pub time_zone: Option<Tzid>,
    pub duration: Iso8601Duration,
    #[serde(default)]
    pub show_without_time: bool,
    /// `[RRuleJSON]` — free-form JSCalendar recurrence rules.
    #[serde(default)]
    pub recurrence_rules: Vec<serde_json::Value>,
    /// `{date: PatchObject}` — per-instance overrides.
    #[serde(default)]
    pub recurrence_overrides: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub excluded_recurrence_dates: Vec<Date>,
    /// `"confirmed"` | `"tentative"` | `"cancelled"`.
    pub status: String,
    #[serde(default)]
    pub priority: i64,
    /// `"free"` | `"busy"`.
    pub free_busy_status: String,
    /// `id → participant` (attendees / organizer).
    #[serde(default)]
    pub participants: BTreeMap<String, Participant>,
    /// `id → alert` (VALARM reminders).
    #[serde(default)]
    pub alerts: BTreeMap<String, Alert>,
    #[serde(default)]
    pub sequence: i64,
    pub etag: Option<String>,
}

/// A named event location (§2.1 `locations:[{name}]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventLocation {
    pub name: String,
}

/// An event participant (attendee/organizer), keyed by id in the event map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Participant {
    pub name: String,
    pub email: String,
    pub role: String,
    /// `"needs-action"` | `"accepted"` | `"declined"` | `"tentative"`.
    pub participation_status: String,
    #[serde(default)]
    pub expect_reply: bool,
}

/// A VALARM reminder (§2.1 `alerts`), keyed by id in the event map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    /// `{offset}` (relative) or `{absolute}` — free-form JSCalendar trigger.
    pub trigger: serde_json::Value,
    /// `"display"` | `"email"`.
    pub action: String,
}

// ── Tasks ───────────────────────────────────────────────────────────────────

/// A task (§2.1, VTODO-aligned). A task list is a `Calendar`-like collection
/// with `component:"VTODO"`; subtasks link via `parentId` (RELATED-TO).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub list_id: String,
    pub uid: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub start: Option<LocalDateTime>,
    pub due: Option<LocalDateTime>,
    pub time_zone: Option<Tzid>,
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub percent_complete: i64,
    /// `"needs-action"` | `"in-process"` | `"completed"` | `"cancelled"`.
    pub status: String,
    #[serde(default)]
    pub progress: String,
    #[serde(default)]
    pub recurrence_rules: Vec<serde_json::Value>,
    /// Parent task id for subtasks (RELATED-TO), or `null`.
    pub parent_id: Option<String>,
    /// The date this task is pinned to My Day / Today, or `null`.
    pub my_day_date: Option<Date>,
    pub etag: Option<String>,
}

// ── Notes ───────────────────────────────────────────────────────────────────

/// A Mailwoman-native note (§2.1). `title`/`tags`/`color`/`pinned` are plaintext
/// (searchable/sortable); the body columns are sealed at rest (plan §1.6) and
/// transit the envelope in the clear — this DTO is the on-the-wire shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub notebook_id: String,
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub color: String,
    #[serde(default)]
    pub pinned: bool,
    /// Rich-text body (sealed at rest; plaintext over the same-origin channel).
    #[serde(default)]
    pub body_html: String,
    #[serde(default)]
    pub body_text: String,
    #[serde(default)]
    pub links: Vec<NoteLink>,
    pub created_at: String,
    pub updated_at: String,
}

/// A cross-link from a note to a message / event / contact (§2.1 `links`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteLink {
    /// `"email"` | `"event"` | `"contact"`.
    #[serde(rename = "type")]
    pub link_type: String,
    pub id: String,
}

// ── Contacts ────────────────────────────────────────────────────────────────

/// An address book (§2.1). CardDAV-backed books carry a `carddavUrl`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressBook {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub carddav_url: Option<String>,
    pub sync_token: Option<String>,
}

/// A contact card (§2.1, JSContact-aligned). `pgpKey`/`smimeCert` are opaque
/// placeholders — PGP/S-MIME wiring is V4 (plan §0/§13).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactCard {
    pub id: String,
    pub address_book_id: String,
    pub uid: String,
    /// `"individual"` | `"org"`.
    pub kind: String,
    pub name: ContactName,
    #[serde(default)]
    pub nicknames: Vec<String>,
    #[serde(default)]
    pub organizations: Vec<String>,
    #[serde(default)]
    pub titles: Vec<String>,
    #[serde(default)]
    pub emails: Vec<ContactEmail>,
    #[serde(default)]
    pub phones: Vec<ContactValue>,
    #[serde(default)]
    pub online_services: Vec<ContactValue>,
    #[serde(default)]
    pub addresses: Vec<serde_json::Value>,
    #[serde(default)]
    pub anniversaries: Vec<Anniversary>,
    #[serde(default)]
    pub notes: String,
    pub photo_blob_id: Option<String>,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub group_ids: Vec<String>,
    /// Opaque PGP public key placeholder (V4).
    pub pgp_key: Option<String>,
    /// Opaque S/MIME certificate placeholder (V4).
    pub smime_cert: Option<String>,
    pub etag: Option<String>,
}

/// A structured contact name (§2.1 `name:{full,given,surname,prefix,suffix}`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactName {
    #[serde(default)]
    pub full: String,
    #[serde(default)]
    pub given: String,
    #[serde(default)]
    pub surname: String,
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub suffix: String,
}

/// A contact email with context + preference (§2.1 `emails:[{context,value,pref}]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactEmail {
    #[serde(default)]
    pub context: String,
    pub value: String,
    #[serde(default)]
    pub pref: i64,
}

/// A generic contexted contact value (phones / online services).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactValue {
    #[serde(default)]
    pub context: String,
    pub value: String,
}

/// A birthday / anniversary (§2.1 `anniversaries:[{kind,date}]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Anniversary {
    /// `"birthday"` | `"anniversary"`.
    pub kind: String,
    pub date: Date,
}

/// A contact group / distribution list (§2.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactGroup {
    pub id: String,
    pub address_book_id: String,
    pub name: String,
    #[serde(default)]
    pub member_ids: Vec<String>,
}
