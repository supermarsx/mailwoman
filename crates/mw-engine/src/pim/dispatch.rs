//! PIM method dispatch (frozen §2.2). Every family rides the existing JMAP
//! envelope; [`Engine::dispatch_pim`] is reached from `handle_jmap`'s dispatch
//! for any `Calendar/*`, `CalendarEvent/*`, `Task/*`, `Note/*`, `AddressBook/*`,
//! `ContactCard/*`, or `ContactGroup/*` method.
//!
//! ## Scaffolder note (e0)
//! Every arm is a frozen contract entry with a `todo!()` body; e8 fills them.
//! The set of method names here IS the §2.2 contract the web client and the
//! parallel builders compile against — do not add/rename without a coordinator
//! re-broadcast.

use serde_json::{Value, json};

use crate::engine::Engine;

/// The Mailwoman PIM method families (frozen §2.2). A method name whose family
/// prefix is one of these routes to [`Engine::dispatch_pim`].
pub const PIM_FAMILIES: &[&str] = &[
    "Calendar/",
    "CalendarEvent/",
    "Task/",
    "Note/",
    "AddressBook/",
    "ContactCard/",
    "ContactGroup/",
];

/// Whether `method` belongs to a Mailwoman PIM family (§2.2).
pub fn is_pim_method(method: &str) -> bool {
    PIM_FAMILIES.iter().any(|fam| method.starts_with(fam))
}

impl Engine {
    /// Dispatch one resolved PIM method call (frozen §2.2). Reached from
    /// `handle_jmap` for any PIM-family method; e8 fills each `todo!()`.
    #[allow(unused_variables)] // e8 consumes account_id/args when filling bodies.
    pub(crate) async fn dispatch_pim(&self, account_id: &str, name: &str, args: &Value) -> Value {
        match name {
            // ── Calendars (§2.2) ──
            "Calendar/get" => todo!("e8: Calendar/get"),
            "Calendar/set" => todo!("e8: Calendar/set"),
            "Calendar/changes" => todo!("e8: Calendar/changes"),
            "Calendar/freeBusy" => {
                todo!("e8: Calendar/freeBusy (principals + window → busy blocks)")
            }
            "Calendar/detectConflicts" => {
                todo!("e8: Calendar/detectConflicts (window → overlapping-instance pairs)")
            }
            // ── Calendar events (§2.2) ──
            "CalendarEvent/get" => todo!("e8: CalendarEvent/get"),
            "CalendarEvent/set" => {
                todo!("e8: CalendarEvent/set (emits iTIP REQUEST when participants set)")
            }
            "CalendarEvent/query" => todo!("e8: CalendarEvent/query"),
            "CalendarEvent/queryChanges" => todo!("e8: CalendarEvent/queryChanges"),
            "CalendarEvent/changes" => todo!("e8: CalendarEvent/changes"),
            "CalendarEvent/expand" => {
                todo!("e8: CalendarEvent/expand (window + master → expanded instances via mw-ics)")
            }
            "CalendarEvent/parse" => todo!("e8: CalendarEvent/parse (ICS/.hol blob → events)"),
            "CalendarEvent/import" => todo!("e8: CalendarEvent/import (ICS/.hol blob → events)"),
            "CalendarEvent/export" => todo!("e8: CalendarEvent/export (→ ICS)"),
            "CalendarEvent/respond" => {
                todo!(
                    "e8: CalendarEvent/respond (iTIP accept/decline/tentative/counter → iMIP REPLY)"
                )
            }
            // ── Tasks (§2.2) ──
            "Task/get" => todo!("e8: Task/get"),
            "Task/set" => todo!("e8: Task/set (supports fromEmail/fromEvent convenience sources)"),
            "Task/query" => todo!("e8: Task/query (incl. My Day / Today filter)"),
            "Task/queryChanges" => todo!("e8: Task/queryChanges"),
            "Task/changes" => todo!("e8: Task/changes"),
            // ── Notes (§2.2) ──
            "Note/get" => todo!("e8: Note/get (decrypts sealed body)"),
            "Note/set" => todo!("e8: Note/set (seals body at rest)"),
            "Note/query" => todo!("e8: Note/query (tags/pinned/text — title + decrypt-scan)"),
            "Note/changes" => todo!("e8: Note/changes"),
            // ── Contacts (§2.2) ──
            "AddressBook/get" => todo!("e8: AddressBook/get"),
            "AddressBook/set" => todo!("e8: AddressBook/set"),
            "AddressBook/changes" => todo!("e8: AddressBook/changes"),
            "ContactCard/get" => todo!("e8: ContactCard/get"),
            "ContactCard/set" => todo!("e8: ContactCard/set"),
            "ContactCard/query" => todo!("e8: ContactCard/query"),
            "ContactCard/queryChanges" => todo!("e8: ContactCard/queryChanges"),
            "ContactCard/changes" => todo!("e8: ContactCard/changes"),
            "ContactCard/import" => todo!("e8: ContactCard/import (vCard/CSV blob)"),
            "ContactCard/export" => todo!("e8: ContactCard/export (→ vCard/CSV)"),
            "ContactCard/merge" => {
                todo!("e8: ContactCard/merge (duplicate resolution → merged + tombstones)")
            }
            "ContactCard/autocomplete" => {
                todo!("e8: ContactCard/autocomplete (prefix → ranked cards for Compose)")
            }
            "ContactGroup/get" => todo!("e8: ContactGroup/get"),
            "ContactGroup/set" => todo!("e8: ContactGroup/set"),
            "ContactGroup/changes" => todo!("e8: ContactGroup/changes"),
            other => json!({
                "type": "unknownMethod",
                "description": format!("engine does not implement PIM method {other}")
            }),
        }
    }
}
