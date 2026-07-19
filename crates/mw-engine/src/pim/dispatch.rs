//! PIM method dispatch (frozen §2.2). Every family rides the existing JMAP
//! envelope; [`Engine::dispatch_pim`] is reached from `handle_jmap`'s dispatch
//! for any `Calendar/*`, `CalendarEvent/*`, `Task/*`, `Note/*`, `AddressBook/*`,
//! `ContactCard/*`, or `ContactGroup/*` method.
//!
//! The set of method names here IS the §2.2 contract the web client and the
//! parallel builders compile against — do not add/rename without a coordinator
//! re-broadcast.

use serde_json::{Value, json};

use crate::account::AccountRuntime;
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
    /// `handle_jmap` for any PIM-family method. `rt` is the connected account's
    /// runtime (submitter for iTIP, DAV config for CalDAV/CardDAV sync).
    pub(crate) async fn dispatch_pim(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        name: &str,
        args: &Value,
    ) -> Value {
        match name {
            // ── Calendars (§2.2) ──
            "Calendar/get" => self.calendar_get(account_id, args).await,
            "Calendar/set" => self.calendar_set(account_id, args).await,
            "Calendar/changes" => {
                self.pim_type_changes(account_id, crate::change::ChangeType::Calendar, args)
                    .await
            }
            "Calendar/freeBusy" => self.calendar_free_busy(account_id, rt, args).await,
            "Calendar/detectConflicts" => self.calendar_detect_conflicts(account_id, args).await,
            "Calendar/subscribe" => self.calendar_subscribe(account_id, args).await,
            "Calendar/refreshSubscription" => {
                self.calendar_refresh_subscription(account_id, args).await
            }
            // ── Calendar events (§2.2) ──
            "CalendarEvent/get" => self.event_get(account_id, args).await,
            "CalendarEvent/set" => self.event_set(account_id, rt, args).await,
            "CalendarEvent/query" => self.event_query(account_id, args).await,
            "CalendarEvent/queryChanges" => self.event_query_changes(account_id, args).await,
            "CalendarEvent/changes" => {
                self.pim_type_changes(account_id, crate::change::ChangeType::CalendarEvent, args)
                    .await
            }
            "CalendarEvent/expand" => self.event_expand(account_id, args).await,
            "CalendarEvent/parse" => self.event_parse(account_id, args).await,
            "CalendarEvent/import" => self.event_import(account_id, rt, args).await,
            "CalendarEvent/export" => self.event_export(account_id, args).await,
            "CalendarEvent/respond" => self.event_respond(account_id, rt, args).await,
            "CalendarEvent/quickAdd" => self.event_quick_add(account_id, rt, args).await,
            // ── Tasks (§2.2) ──
            "Task/get" => self.task_get(account_id, args).await,
            "Task/set" => self.task_set(account_id, rt, args).await,
            "Task/query" => self.task_query(account_id, args).await,
            "Task/queryChanges" => self.task_query_changes(account_id, args).await,
            "Task/changes" => {
                self.pim_type_changes(account_id, crate::change::ChangeType::Task, args)
                    .await
            }
            // ── Notes (§2.2) ──
            "Note/get" => self.note_get(account_id, args).await,
            "Note/set" => self.note_set(account_id, args).await,
            "Note/query" => self.note_query(account_id, args).await,
            "Note/export" => self.note_export(account_id, args).await,
            "Note/changes" => {
                self.pim_type_changes(account_id, crate::change::ChangeType::Note, args)
                    .await
            }
            // ── Contacts (§2.2) ──
            "AddressBook/get" => self.address_book_get(account_id, args).await,
            "AddressBook/set" => self.address_book_set(account_id, args).await,
            "AddressBook/changes" => {
                self.pim_type_changes(account_id, crate::change::ChangeType::AddressBook, args)
                    .await
            }
            "ContactCard/get" => self.contact_get(account_id, args).await,
            "ContactCard/set" => self.contact_set(account_id, rt, args).await,
            "ContactCard/query" => self.contact_query(account_id, args).await,
            "ContactCard/queryChanges" => self.contact_query_changes(account_id, args).await,
            "ContactCard/changes" => {
                self.pim_type_changes(account_id, crate::change::ChangeType::ContactCard, args)
                    .await
            }
            "ContactCard/import" => self.contact_import(account_id, rt, args).await,
            "ContactCard/export" => self.contact_export(account_id, args).await,
            "ContactCard/merge" => self.contact_merge(account_id, args).await,
            "ContactCard/autocomplete" => self.contact_autocomplete(account_id, args).await,
            "ContactGroup/get" => self.contact_group_get(account_id, args).await,
            "ContactGroup/set" => self.contact_group_set(account_id, args).await,
            "ContactGroup/changes" => {
                self.pim_type_changes(account_id, crate::change::ChangeType::ContactGroup, args)
                    .await
            }
            other => json!({
                "type": "unknownMethod",
                "description": format!("engine does not implement PIM method {other}")
            }),
        }
    }

    /// The generic PIM `*/changes` handler (frozen §2.1), sourced from the
    /// `pim_changes` log. Mirrors the mail `type_changes` shape.
    pub(crate) async fn pim_type_changes(
        &self,
        account_id: &str,
        kind: crate::change::ChangeType,
        args: &Value,
    ) -> Value {
        let since = args
            .get("sinceState")
            .and_then(Value::as_str)
            .unwrap_or("0");
        match self.build_pim_changes(account_id, kind, since).await {
            Ok(changes) => {
                let mut v = serde_json::to_value(&changes).unwrap_or_else(|_| json!({}));
                if let Some(obj) = v.as_object_mut() {
                    obj.insert("accountId".into(), json!(account_id));
                }
                v
            }
            Err(e) => super::server_fail(e),
        }
    }
}
