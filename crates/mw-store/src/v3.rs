//! V3 (PIM) repository methods (plan §2.4) layered over [`Store`]: calendars,
//! events + materialized instances, tasks, sealed-at-rest notes, address books,
//! contacts, groups, and the per-account `pim_changes` log.
//!
//! Same seam discipline as [`crate::v2`]: every value crosses as an opaque
//! primitive; enum-like fields (`access`, `status`, `component`, change
//! `op`/`type`) are plain strings the engine owns. Note **bodies** are sealed
//! with the existing [`crate::ServerKey`] (`body_*_sealed` BLOBs) —
//! encrypted-at-rest, NOT zero-access (plan §1.6).
//!
//! ## Scaffolder note (e0)
//! e0 freezes the row DTOs + method signatures; **e8** fills every `todo!()`
//! (the CRUD, the sealed-note read/write, the instance regeneration, and the
//! `pim_changes` state counters). No SQL is executed yet.

#![allow(unused_variables)] // e8 consumes these when filling the query bodies.

use crate::{Store, StoreError};

/// A calendar or task-list collection row (`calendars`, plan §2.4).
/// `component` = `"VEVENT"` (calendar) | `"VTODO"` (task list).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarRow {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub color: String,
    pub sort_order: i64,
    pub is_visible: bool,
    pub role: Option<String>,
    pub caldav_url: Option<String>,
    pub sync_token: Option<String>,
    pub ctag: Option<String>,
    pub is_overlay: bool,
    pub component: String,
}

/// An event row (`events`). `ical_raw` is the fidelity source of truth; `json`
/// the parsed Mailwoman projection; `start_utc`/`end_utc` the expansion bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRow {
    pub id: String,
    pub calendar_id: String,
    pub uid: String,
    pub etag: Option<String>,
    pub ical_raw: String,
    pub start_utc: Option<String>,
    pub end_utc: Option<String>,
    pub tzid: Option<String>,
    pub rrule: Option<String>,
    pub status: String,
    pub json: Option<Vec<u8>>,
}

/// One materialized recurrence instance (`event_instances`), regenerated on
/// each event write; the range-query + conflict-detection index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventInstanceRow {
    pub event_id: String,
    pub instance_start_utc: String,
    pub instance_end_utc: String,
}

/// A task row (`tasks`, VTODO). `my_day_date` pins it to My Day / Today.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRow {
    pub id: String,
    pub list_id: String,
    pub uid: String,
    pub etag: Option<String>,
    pub due_utc: Option<String>,
    pub start_utc: Option<String>,
    pub priority: i64,
    pub percent_complete: i64,
    pub status: String,
    pub parent_id: Option<String>,
    pub my_day_date: Option<String>,
    pub ical_raw: String,
    pub json: Option<Vec<u8>>,
}

/// A note row with the body **decrypted** (`title`/`tags`/`color`/`pinned`
/// plaintext columns + the unsealed rich-text body). The store seals/unseals
/// `body_*` transparently at the repo boundary (plan §1.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteRow {
    pub id: String,
    pub account_id: String,
    pub notebook_id: Option<String>,
    pub title: String,
    /// JSON-encoded `[String]` tag array (plaintext).
    pub tags_json: String,
    pub color: String,
    pub pinned: bool,
    /// Rich-text body, **decrypted** for the caller (sealed as a BLOB at rest).
    pub body_html: String,
    pub body_text: String,
    /// JSON-encoded `[{type,id}]` cross-links.
    pub links_json: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A notebook row (`notebooks`) grouping notes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotebookRow {
    pub id: String,
    pub account_id: String,
    pub name: String,
}

/// An address-book row (`address_books`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressBookRow {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub is_default: bool,
    pub carddav_url: Option<String>,
    pub sync_token: Option<String>,
    pub ctag: Option<String>,
}

/// A contact row (`contacts`). `vcard_raw` is the fidelity source of truth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactRow {
    pub id: String,
    pub address_book_id: String,
    pub uid: String,
    pub etag: Option<String>,
    pub vcard_raw: String,
    pub json: Option<Vec<u8>>,
    pub full_name: String,
    pub is_favorite: bool,
    pub photo_blob_id: Option<String>,
    pub pgp_key: Option<String>,
    pub smime_cert: Option<String>,
}

/// A contact group / distribution list row (`contact_groups`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactGroupRow {
    pub id: String,
    pub address_book_id: String,
    pub name: String,
    /// JSON-encoded `[String]` member-id array.
    pub member_ids_json: String,
}

/// One row of the PIM change log for a `*/changes` diff (`pim_changes`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PimChangeRow {
    pub state: u64,
    pub object_id: String,
    /// `created` | `updated` | `destroyed` (opaque to the store).
    pub op: String,
}

impl Store {
    // ── calendars ───────────────────────────────────────────────────────────

    /// List an account's calendars + task lists (`calendars`).
    pub async fn list_calendars(&self, account_id: &str) -> Result<Vec<CalendarRow>, StoreError> {
        todo!("e8: SELECT * FROM calendars WHERE account_id = ?")
    }

    /// Insert or replace a calendar / task-list collection.
    pub async fn upsert_calendar(&self, row: &CalendarRow) -> Result<(), StoreError> {
        todo!("e8: INSERT ... ON CONFLICT(id) DO UPDATE for calendars")
    }

    // ── events + instances ──────────────────────────────────────────────────

    /// List a calendar's master events.
    pub async fn list_events(&self, calendar_id: &str) -> Result<Vec<EventRow>, StoreError> {
        todo!("e8: SELECT * FROM events WHERE calendar_id = ?")
    }

    /// Insert or replace an event (source of truth = `ical_raw`).
    pub async fn upsert_event(&self, row: &EventRow) -> Result<(), StoreError> {
        todo!("e8: INSERT ... ON CONFLICT(id) DO UPDATE for events")
    }

    /// Regenerate an event's materialized recurrence instances (delete + insert).
    pub async fn replace_event_instances(
        &self,
        event_id: &str,
        instances: &[EventInstanceRow],
    ) -> Result<(), StoreError> {
        todo!("e8: DELETE + bulk INSERT into event_instances for one event")
    }

    /// Events whose materialized instances overlap `[start_utc, end_utc)` — the
    /// range-query + conflict-detection index (plan §2.4).
    pub async fn events_in_range(
        &self,
        account_id: &str,
        start_utc: &str,
        end_utc: &str,
    ) -> Result<Vec<EventInstanceRow>, StoreError> {
        todo!("e8: JOIN event_instances on idx_event_instances_range")
    }

    // ── tasks ───────────────────────────────────────────────────────────────

    /// List a task list's tasks.
    pub async fn list_tasks(&self, list_id: &str) -> Result<Vec<TaskRow>, StoreError> {
        todo!("e8: SELECT * FROM tasks WHERE list_id = ?")
    }

    /// Insert or replace a task.
    pub async fn upsert_task(&self, row: &TaskRow) -> Result<(), StoreError> {
        todo!("e8: INSERT ... ON CONFLICT(id) DO UPDATE for tasks")
    }

    // ── notebooks ───────────────────────────────────────────────────────────

    /// List an account's notebooks.
    pub async fn list_notebooks(&self, account_id: &str) -> Result<Vec<NotebookRow>, StoreError> {
        todo!("e8: SELECT * FROM notebooks WHERE account_id = ?")
    }

    /// Insert or replace a notebook.
    pub async fn upsert_notebook(&self, row: &NotebookRow) -> Result<(), StoreError> {
        todo!("e8: INSERT ... ON CONFLICT(id) DO UPDATE for notebooks")
    }

    // ── notes (sealed at rest) ──────────────────────────────────────────────

    /// Fetch one note with its body **unsealed** (plan §1.6).
    pub async fn get_note(&self, id: &str) -> Result<Option<NoteRow>, StoreError> {
        todo!("e8: SELECT note + self.key.unseal(body_*_sealed)")
    }

    /// List an account's notes (metadata + unsealed bodies for the small set).
    pub async fn list_notes(&self, account_id: &str) -> Result<Vec<NoteRow>, StoreError> {
        todo!("e8: SELECT notes WHERE account_id = ? (pinned first)")
    }

    /// Insert or replace a note, **sealing** the body columns at rest.
    pub async fn upsert_note(&self, row: &NoteRow) -> Result<(), StoreError> {
        todo!("e8: seal body_html/body_text via self.key.seal, then UPSERT")
    }

    // ── address books + contacts ────────────────────────────────────────────

    /// List an account's address books.
    pub async fn list_address_books(
        &self,
        account_id: &str,
    ) -> Result<Vec<AddressBookRow>, StoreError> {
        todo!("e8: SELECT * FROM address_books WHERE account_id = ?")
    }

    /// Insert or replace an address book.
    pub async fn upsert_address_book(&self, row: &AddressBookRow) -> Result<(), StoreError> {
        todo!("e8: INSERT ... ON CONFLICT(id) DO UPDATE for address_books")
    }

    /// List an address book's contacts.
    pub async fn list_contacts(
        &self,
        address_book_id: &str,
    ) -> Result<Vec<ContactRow>, StoreError> {
        todo!("e8: SELECT * FROM contacts WHERE address_book_id = ?")
    }

    /// Insert or replace a contact (source of truth = `vcard_raw`).
    pub async fn upsert_contact(&self, row: &ContactRow) -> Result<(), StoreError> {
        todo!("e8: INSERT ... ON CONFLICT(id) DO UPDATE for contacts")
    }

    /// Prefix-match contacts by name/email for Compose autocomplete
    /// (`ContactCard/autocomplete`, §2.2), ranked by favorite then name.
    pub async fn autocomplete_contacts(
        &self,
        account_id: &str,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<ContactRow>, StoreError> {
        todo!("e8: ranked prefix scan over idx_contacts_fullname/favorite")
    }

    /// List an address book's contact groups.
    pub async fn list_contact_groups(
        &self,
        address_book_id: &str,
    ) -> Result<Vec<ContactGroupRow>, StoreError> {
        todo!("e8: SELECT * FROM contact_groups WHERE address_book_id = ?")
    }

    /// Insert or replace a contact group.
    pub async fn upsert_contact_group(&self, row: &ContactGroupRow) -> Result<(), StoreError> {
        todo!("e8: INSERT ... ON CONFLICT(id) DO UPDATE for contact_groups")
    }

    // ── pim_changes (state tokens + `*/changes`) ────────────────────────────

    /// Append one PIM change and return the new `(account, type)` state.
    pub async fn record_pim_change(
        &self,
        account_id: &str,
        type_name: &str,
        object_id: &str,
        op: &str,
    ) -> Result<u64, StoreError> {
        todo!("e8: bump the (account,type) counter + INSERT into pim_changes")
    }

    /// The current `(account, type)` PIM state counter (0 when none).
    pub async fn current_pim_state(
        &self,
        account_id: &str,
        type_name: &str,
    ) -> Result<u64, StoreError> {
        todo!("e8: SELECT MAX(state) FROM pim_changes WHERE account_id=? AND type=?")
    }

    /// The PIM change rows for a datatype since `since_state` (the diff input).
    pub async fn pim_changes_since(
        &self,
        account_id: &str,
        type_name: &str,
        since: u64,
    ) -> Result<Vec<PimChangeRow>, StoreError> {
        todo!("e8: SELECT ... FROM pim_changes WHERE ... AND state > ?")
    }
}
