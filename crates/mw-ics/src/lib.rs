#![forbid(unsafe_code)]
//! `mw-ics` — iCalendar / vCard serialization + RRULE expansion + iTIP framing
//! for Mailwoman V3 (plan §0.1, §1.2, SPEC §6.2/§11/§13).
//!
//! This crate is the leaf serialization layer: it converts the frozen Mailwoman
//! PIM shapes (§2.1, carried here as `serde_json::Value` projections so the
//! crate stays decoupled from `mw-engine`) to and from wire bytes, and
//! hand-rolls the thin protocol layers on top. The `ical_raw`/`vcard_raw` bytes
//! are the round-trip source of truth (plan risk #13); the JSON `json`
//! projection is a focused, lossy view.
//!
//! ## Public surface (stable — e8 builds on this)
//! - [`parse_ical`] / [`emit_ical`] — VEVENT/VTODO ⇄ [`ParsedIcal`].
//! - [`expand_recurrence`] — TZID-aware, DST-correct RRULE/RDATE/EXDATE
//!   expansion → [`Instance`] UTC bounds (plan §1.12 gate).
//! - [`build_itip`] / [`parse_itip`] — iTIP `METHOD` framing ([`ItipMethod`]).
//! - [`parse_vcard`] / [`emit_vcard`] — vCard 3/4 ⇄ [`ParsedVcard`].
//! - [`parse_hol`] — Outlook `.hol` holiday packs → all-day events.
//! - [`aggregate_free_busy`] — VFREEBUSY merge → [`BusyInterval`].

mod dt;
mod freebusy;
mod hol;
mod ical;
mod itip;
mod recur;
mod vcard;

pub use freebusy::{BusyInterval, aggregate_free_busy};
pub use hol::parse_hol;
pub use ical::{ParsedIcal, emit_ical, parse_ical};
pub use itip::{ItipMethod, build_itip, parse_itip};
pub use recur::{Instance, expand_recurrence};
pub use vcard::{ParsedVcard, emit_vcard, parse_vcard};

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
