#![forbid(unsafe_code)]
//! `mw-ics` — iCalendar / vCard serialization + RRULE expansion + iTIP framing
//! for Mailwoman V3 (plan §0.1, §1.2, SPEC §6.2/§11/§13).
//!
//! This crate is the leaf serialization layer: it converts the frozen
//! Mailwoman PIM shapes (mirrored here as DTO seams so the crate stays
//! decoupled from `mw-engine`, same discipline as `mw-search`'s `IndexDoc`) to
//! and from wire bytes, and hand-rolls the thin protocol layers on top:
//!
//! - **iCalendar** VEVENT/VTODO/VJOURNAL/VALARM/VTIMEZONE parse+emit (via the
//!   `icalendar` crate), with `ical_raw` preserved as the round-trip source of
//!   truth (plan risk #13).
//! - **vCard 3/4** parse+emit (via `vcard4`).
//! - **RRULE expansion** (via `rrule` + `chrono-tz`), TZID-aware and
//!   DST-correct (plan §1.12 — the DST-boundary recurrence test is a DoD gate).
//! - **iTIP `METHOD` framing** (REQUEST/REPLY/COUNTER/CANCEL) for iMIP (§2.6).
//! - **`.hol`** (Outlook holiday pack) parse/emit.
//! - **VFREEBUSY** aggregation (free/busy merge).
//!
//! ## Scaffolder note (e0)
//! e0 freezes the module layout + the DTO seams + the public function
//! signatures below; **e1** fills every `todo!()` body, adds the golden
//! round-trip tests, and wires the `cargo-fuzz` targets (iCal + vCard parse,
//! plan §1.9). Nothing here carries logic yet.

use serde::{Deserialize, Serialize};

/// A recoverable serialization / expansion failure.
#[derive(Debug, thiserror::Error)]
pub enum IcsError {
    #[error("iCalendar parse error: {0}")]
    Ical(String),
    #[error("vCard parse error: {0}")]
    Vcard(String),
    #[error("recurrence expansion error: {0}")]
    Rrule(String),
    #[error("iTIP framing error: {0}")]
    Itip(String),
    #[error("holiday pack (.hol) error: {0}")]
    Hol(String),
}

/// The convenience result alias for this crate.
pub type Result<T> = std::result::Result<T, IcsError>;

// ── iCalendar (events / tasks) ──────────────────────────────────────────────

/// One parsed calendar object: the parsed Mailwoman-shape JSON projection plus
/// the verbatim `ical_raw` kept as the round-trip source of truth (plan §2.4,
/// risk #13). `json` is a projection, not the lossy authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedIcal {
    /// The original bytes, preserved so unknown X-properties re-emit verbatim.
    pub ical_raw: String,
    /// The parsed Mailwoman `CalendarEvent`/`Task` projection (§2.1 shapes).
    pub json: serde_json::Value,
    /// The top-level component kind (`"VEVENT"` / `"VTODO"` / `"VJOURNAL"`).
    pub component: String,
}

/// Parse an iCalendar document (one or more VEVENT/VTODO components) into the
/// Mailwoman projection, preserving the raw bytes (e1).
pub fn parse_ical(_bytes: &[u8]) -> Result<Vec<ParsedIcal>> {
    todo!("e1: iCalendar VEVENT/VTODO parse via the `icalendar` crate")
}

/// Emit an iCalendar document from a Mailwoman `CalendarEvent`/`Task` JSON
/// projection (e1). Unknown properties captured in `ical_raw` are preserved.
pub fn emit_ical(_event_json: &serde_json::Value) -> Result<String> {
    todo!("e1: iCalendar emit from the Mailwoman event/task shape")
}

// ── RRULE expansion (TZID-aware, DST-correct) ───────────────────────────────

/// A single expanded recurrence instance: start/end in UTC (the
/// `event_instances` materialization index bounds, plan §2.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instance {
    /// Instance start, RFC3339 UTC.
    pub start_utc: String,
    /// Instance end, RFC3339 UTC.
    pub end_utc: String,
}

/// Expand a master event's recurrence set within `[window_start, window_end)`
/// (RFC3339 UTC), honouring the event's TZID via `rrule` + `chrono-tz` so
/// DST boundaries are correct (plan §1.12). RDATE/EXDATE/recurrence overrides
/// are applied (e1).
pub fn expand_recurrence(
    _event_json: &serde_json::Value,
    _window_start: &str,
    _window_end: &str,
) -> Result<Vec<Instance>> {
    todo!("e1: RRULE/RDATE/EXDATE expansion in the event's TZID (rrule + chrono-tz)")
}

// ── iTIP / iMIP METHOD framing (§2.6) ───────────────────────────────────────

/// The iTIP scheduling method carried by a `text/calendar; method=…` part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItipMethod {
    Request,
    Reply,
    Counter,
    Cancel,
}

/// Build an iTIP `text/calendar` payload framing `event_json` with `method`
/// (REQUEST from an organizer, REPLY/COUNTER from an attendee, CANCEL) for iMIP
/// delivery over the account's `MailSubmitter` (§2.6, e1 frames / e8 sends).
pub fn build_itip(_event_json: &serde_json::Value, _method: ItipMethod) -> Result<String> {
    todo!("e1: iTIP METHOD framing over the icalendar component model")
}

/// Parse an inbound `text/calendar` part, returning its method + the event
/// projection (the mail-ingest invite-detection hook, §2.6).
pub fn parse_itip(_bytes: &[u8]) -> Result<(ItipMethod, ParsedIcal)> {
    todo!("e1: iTIP method detection + event parse")
}

// ── vCard (contacts) ────────────────────────────────────────────────────────

/// One parsed contact: the Mailwoman `ContactCard` projection plus the verbatim
/// `vcard_raw` (round-trip source of truth, plan §2.4/risk #13).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedVcard {
    pub vcard_raw: String,
    /// The parsed Mailwoman `ContactCard` projection (§2.1).
    pub json: serde_json::Value,
}

/// Parse a vCard 3/4 document (one or more cards) into the Mailwoman
/// projection, preserving the raw bytes (e1, via `vcard4`).
pub fn parse_vcard(_bytes: &[u8]) -> Result<Vec<ParsedVcard>> {
    todo!("e1: vCard 3/4 parse via the `vcard4` crate")
}

/// Emit a vCard from a Mailwoman `ContactCard` JSON projection (e1).
pub fn emit_vcard(_contact_json: &serde_json::Value) -> Result<String> {
    todo!("e1: vCard emit from the Mailwoman contact shape")
}

// ── `.hol` holiday packs + VFREEBUSY ────────────────────────────────────────

/// Parse an Outlook `.hol` holiday pack into a set of all-day VEVENT
/// projections (the bundled/subscribable holiday feed, §11, e1).
pub fn parse_hol(_bytes: &[u8]) -> Result<Vec<ParsedIcal>> {
    todo!("e1: .hol (Outlook holiday) parse")
}

/// A busy interval in a free/busy response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusyInterval {
    pub start_utc: String,
    pub end_utc: String,
    /// `"busy"` / `"tentative"` / `"free"`.
    pub status: String,
}

/// Aggregate a set of events into merged VFREEBUSY busy intervals over a window
/// (the `Calendar/freeBusy` engine helper, §2.2, e1).
pub fn aggregate_free_busy(
    _events_json: &[serde_json::Value],
    _window_start: &str,
    _window_end: &str,
) -> Result<Vec<BusyInterval>> {
    todo!("e1: VFREEBUSY aggregation / free-busy merge")
}
