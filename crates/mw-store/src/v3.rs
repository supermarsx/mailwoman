//! V3 (PIM) repository methods (plan §2.4) layered over [`Store`]: calendars,
//! events + materialized instances, tasks, sealed-at-rest notes, address books,
//! contacts, groups, and the per-account `pim_changes` log.
//!
//! Same seam discipline as [`crate::v2`]: every value crosses as an opaque
//! primitive; enum-like fields (`access`, `status`, `component`, change
//! `op`/`type`) are plain strings the engine owns. Note **bodies** and their
//! sort/filter metadata (`title`/`tags`/`color`/`pinned`) are sealed with the
//! existing [`crate::ServerKey`] (`body_*_sealed` + the 0019 `*_sealed` BLOBs) —
//! encrypted-at-rest, NOT zero-access (plan §1.6).

use crate::{Row, Store, StoreError, q};

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

/// A note row with every user field **decrypted** for the caller. `title`,
/// `tags_json`, `color`, `pinned` and the rich-text body are all sealed at rest
/// (`*_sealed` BLOBs under the store [`crate::ServerKey`]) and unsealed here at
/// the repo boundary (0019, plan §1.6); the frozen plaintext columns are blanked
/// once a row is sealed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteRow {
    pub id: String,
    pub account_id: String,
    pub notebook_id: Option<String>,
    pub title: String,
    /// JSON-encoded `[String]` tag array (**decrypted**; sealed at rest).
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

/// Canonical sealed form of a note's `pinned` flag (0019): a single ASCII byte
/// `b"1"`/`b"0"`. [`Store::note_from_row`] unseals it back to a bool.
fn pinned_canonical(pinned: bool) -> &'static [u8] {
    if pinned { b"1" } else { b"0" }
}

fn calendar_from_row(r: &Row) -> CalendarRow {
    CalendarRow {
        id: r.get_string("id"),
        account_id: r.get_string("account_id"),
        name: r.get_string("name"),
        color: r.get_string("color"),
        sort_order: r.get_i64("sort_order"),
        is_visible: r.get_i64("is_visible") != 0,
        role: r.get_opt_string("role"),
        caldav_url: r.get_opt_string("caldav_url"),
        sync_token: r.get_opt_string("sync_token"),
        ctag: r.get_opt_string("ctag"),
        is_overlay: r.get_i64("is_overlay") != 0,
        component: r.get_string("component"),
    }
}

fn event_from_row(r: &Row) -> EventRow {
    EventRow {
        id: r.get_string("id"),
        calendar_id: r.get_string("calendar_id"),
        uid: r.get_string("uid"),
        etag: r.get_opt_string("etag"),
        ical_raw: r.get_string("ical_raw"),
        start_utc: r.get_opt_string("start_utc"),
        end_utc: r.get_opt_string("end_utc"),
        tzid: r.get_opt_string("tzid"),
        rrule: r.get_opt_string("rrule"),
        status: r.get_string("status"),
        json: r.get_opt_blob("json"),
    }
}

fn task_from_row(r: &Row) -> TaskRow {
    TaskRow {
        id: r.get_string("id"),
        list_id: r.get_string("list_id"),
        uid: r.get_string("uid"),
        etag: r.get_opt_string("etag"),
        due_utc: r.get_opt_string("due_utc"),
        start_utc: r.get_opt_string("start_utc"),
        priority: r.get_i64("priority"),
        percent_complete: r.get_i64("percent_complete"),
        status: r.get_string("status"),
        parent_id: r.get_opt_string("parent_id"),
        my_day_date: r.get_opt_string("my_day_date"),
        ical_raw: r.get_string("ical_raw"),
        json: r.get_opt_blob("json"),
    }
}

fn address_book_from_row(r: &Row) -> AddressBookRow {
    AddressBookRow {
        id: r.get_string("id"),
        account_id: r.get_string("account_id"),
        name: r.get_string("name"),
        is_default: r.get_i64("is_default") != 0,
        carddav_url: r.get_opt_string("carddav_url"),
        sync_token: r.get_opt_string("sync_token"),
        ctag: r.get_opt_string("ctag"),
    }
}

fn contact_from_row(r: &Row) -> ContactRow {
    ContactRow {
        id: r.get_string("id"),
        address_book_id: r.get_string("address_book_id"),
        uid: r.get_string("uid"),
        etag: r.get_opt_string("etag"),
        vcard_raw: r.get_string("vcard_raw"),
        json: r.get_opt_blob("json"),
        full_name: r.get_string("full_name"),
        is_favorite: r.get_i64("is_favorite") != 0,
        photo_blob_id: r.get_opt_string("photo_blob_id"),
        pgp_key: r.get_opt_string("pgp_key"),
        smime_cert: r.get_opt_string("smime_cert"),
    }
}

fn contact_group_from_row(r: &Row) -> ContactGroupRow {
    ContactGroupRow {
        id: r.get_string("id"),
        address_book_id: r.get_string("address_book_id"),
        name: r.get_string("name"),
        member_ids_json: r.get_string("member_ids_json"),
    }
}

impl Store {
    // ── calendars ───────────────────────────────────────────────────────────

    /// List an account's calendars + task lists (`calendars`).
    pub async fn list_calendars(&self, account_id: &str) -> Result<Vec<CalendarRow>, StoreError> {
        let rows = q(
            "SELECT id, account_id, name, color, sort_order, is_visible, role, caldav_url,
                    sync_token, ctag, is_overlay, component
             FROM calendars WHERE account_id = ?1 ORDER BY sort_order, name",
        )
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(calendar_from_row).collect())
    }

    /// Fetch one calendar / task-list collection by id.
    pub async fn get_calendar(&self, id: &str) -> Result<Option<CalendarRow>, StoreError> {
        let row = q(
            "SELECT id, account_id, name, color, sort_order, is_visible, role, caldav_url,
                    sync_token, ctag, is_overlay, component
             FROM calendars WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(calendar_from_row))
    }

    /// Insert or replace a calendar / task-list collection.
    pub async fn upsert_calendar(&self, row: &CalendarRow) -> Result<(), StoreError> {
        q("INSERT INTO calendars
                 (id, account_id, name, color, sort_order, is_visible, role, caldav_url,
                  sync_token, ctag, is_overlay, component)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name, color = excluded.color, sort_order = excluded.sort_order,
                 is_visible = excluded.is_visible, role = excluded.role,
                 caldav_url = excluded.caldav_url, sync_token = excluded.sync_token,
                 ctag = excluded.ctag, is_overlay = excluded.is_overlay,
                 component = excluded.component")
        .bind(&row.id)
        .bind(&row.account_id)
        .bind(&row.name)
        .bind(&row.color)
        .bind(row.sort_order)
        .bind(i64::from(row.is_visible))
        .bind(row.role.as_deref())
        .bind(row.caldav_url.as_deref())
        .bind(row.sync_token.as_deref())
        .bind(row.ctag.as_deref())
        .bind(i64::from(row.is_overlay))
        .bind(&row.component)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Delete a calendar / task list (cascades events, tasks, instances, shares).
    pub async fn delete_calendar(&self, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM calendars WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Replace a calendar's ACL shares (delete + bulk insert).
    pub async fn replace_calendar_shares(
        &self,
        calendar_id: &str,
        shares: &[(String, String)],
    ) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        q("DELETE FROM calendar_shares WHERE calendar_id = ?1")
            .bind(calendar_id)
            .execute_tx(&mut tx)
            .await?;
        for (principal, access) in shares {
            q("INSERT INTO calendar_shares (calendar_id, principal, access) VALUES (?1, ?2, ?3)")
                .bind(calendar_id)
                .bind(principal)
                .bind(access)
                .execute_tx(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// List a calendar's ACL shares as `(principal, access)` pairs.
    pub async fn list_calendar_shares(
        &self,
        calendar_id: &str,
    ) -> Result<Vec<(String, String)>, StoreError> {
        let rows = q(
            "SELECT principal, access FROM calendar_shares WHERE calendar_id = ?1 ORDER BY principal",
        )
        .bind(calendar_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| (r.get_string("principal"), r.get_string("access")))
            .collect())
    }

    // ── events + instances ──────────────────────────────────────────────────

    /// List a calendar's master events.
    pub async fn list_events(&self, calendar_id: &str) -> Result<Vec<EventRow>, StoreError> {
        let rows = q(
            "SELECT id, calendar_id, uid, etag, ical_raw, start_utc, end_utc, tzid, rrule, status, json
             FROM events WHERE calendar_id = ?1 ORDER BY start_utc",
        )
        .bind(calendar_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(event_from_row).collect())
    }

    /// Fetch one event by id.
    pub async fn get_event(&self, id: &str) -> Result<Option<EventRow>, StoreError> {
        let row = q(
            "SELECT id, calendar_id, uid, etag, ical_raw, start_utc, end_utc, tzid, rrule, status, json
             FROM events WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(event_from_row))
    }

    /// Insert or replace an event (source of truth = `ical_raw`).
    pub async fn upsert_event(&self, row: &EventRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO events
                 (id, calendar_id, uid, etag, ical_raw, start_utc, end_utc, tzid, rrule, status, json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
                 calendar_id = excluded.calendar_id, uid = excluded.uid, etag = excluded.etag,
                 ical_raw = excluded.ical_raw, start_utc = excluded.start_utc,
                 end_utc = excluded.end_utc, tzid = excluded.tzid, rrule = excluded.rrule,
                 status = excluded.status, json = excluded.json",
        )
        .bind(&row.id)
        .bind(&row.calendar_id)
        .bind(&row.uid)
        .bind(row.etag.as_deref())
        .bind(&row.ical_raw)
        .bind(row.start_utc.as_deref())
        .bind(row.end_utc.as_deref())
        .bind(row.tzid.as_deref())
        .bind(row.rrule.as_deref())
        .bind(&row.status)
        .bind(row.json.as_deref())
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Delete an event (cascades its materialized instances).
    pub async fn delete_event(&self, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM events WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Regenerate an event's materialized recurrence instances (delete + insert).
    pub async fn replace_event_instances(
        &self,
        event_id: &str,
        instances: &[EventInstanceRow],
    ) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        q("DELETE FROM event_instances WHERE event_id = ?1")
            .bind(event_id)
            .execute_tx(&mut tx)
            .await?;
        for inst in instances {
            q(
                "INSERT INTO event_instances (event_id, instance_start_utc, instance_end_utc)
                 VALUES (?1, ?2, ?3)",
            )
            .bind(event_id)
            .bind(&inst.instance_start_utc)
            .bind(&inst.instance_end_utc)
            .execute_tx(&mut tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Events whose materialized instances overlap `[start_utc, end_utc)` for an
    /// account — the range-query + conflict-detection index (plan §2.4). Rows are
    /// ordered by instance start so conflict pairing is a linear scan.
    pub async fn events_in_range(
        &self,
        account_id: &str,
        start_utc: &str,
        end_utc: &str,
    ) -> Result<Vec<EventInstanceRow>, StoreError> {
        let rows = q("SELECT ei.event_id AS event_id,
                    ei.instance_start_utc AS instance_start_utc,
                    ei.instance_end_utc AS instance_end_utc
             FROM event_instances ei
             JOIN events e ON e.id = ei.event_id
             JOIN calendars c ON c.id = e.calendar_id
             WHERE c.account_id = ?1
               AND ei.instance_start_utc < ?3
               AND ei.instance_end_utc > ?2
             ORDER BY ei.instance_start_utc")
        .bind(account_id)
        .bind(start_utc)
        .bind(end_utc)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| EventInstanceRow {
                event_id: r.get_string("event_id"),
                instance_start_utc: r.get_string("instance_start_utc"),
                instance_end_utc: r.get_string("instance_end_utc"),
            })
            .collect())
    }

    // ── tasks ───────────────────────────────────────────────────────────────

    /// List a task list's tasks.
    pub async fn list_tasks(&self, list_id: &str) -> Result<Vec<TaskRow>, StoreError> {
        let rows = q(
            "SELECT id, list_id, uid, etag, due_utc, start_utc, priority, percent_complete,
                    status, parent_id, my_day_date, ical_raw, json
             FROM tasks WHERE list_id = ?1 ORDER BY due_utc IS NULL, due_utc, id",
        )
        .bind(list_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(task_from_row).collect())
    }

    /// Fetch one task by id.
    pub async fn get_task(&self, id: &str) -> Result<Option<TaskRow>, StoreError> {
        let row = q(
            "SELECT id, list_id, uid, etag, due_utc, start_utc, priority, percent_complete,
                    status, parent_id, my_day_date, ical_raw, json
             FROM tasks WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(task_from_row))
    }

    /// Insert or replace a task.
    pub async fn upsert_task(&self, row: &TaskRow) -> Result<(), StoreError> {
        q("INSERT INTO tasks
                 (id, list_id, uid, etag, due_utc, start_utc, priority, percent_complete,
                  status, parent_id, my_day_date, ical_raw, json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(id) DO UPDATE SET
                 list_id = excluded.list_id, uid = excluded.uid, etag = excluded.etag,
                 due_utc = excluded.due_utc, start_utc = excluded.start_utc,
                 priority = excluded.priority, percent_complete = excluded.percent_complete,
                 status = excluded.status, parent_id = excluded.parent_id,
                 my_day_date = excluded.my_day_date, ical_raw = excluded.ical_raw,
                 json = excluded.json")
        .bind(&row.id)
        .bind(&row.list_id)
        .bind(&row.uid)
        .bind(row.etag.as_deref())
        .bind(row.due_utc.as_deref())
        .bind(row.start_utc.as_deref())
        .bind(row.priority)
        .bind(row.percent_complete)
        .bind(&row.status)
        .bind(row.parent_id.as_deref())
        .bind(row.my_day_date.as_deref())
        .bind(&row.ical_raw)
        .bind(row.json.as_deref())
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Delete a task.
    pub async fn delete_task(&self, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM tasks WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ── notebooks ───────────────────────────────────────────────────────────

    /// List an account's notebooks.
    pub async fn list_notebooks(&self, account_id: &str) -> Result<Vec<NotebookRow>, StoreError> {
        let rows =
            q("SELECT id, account_id, name FROM notebooks WHERE account_id = ?1 ORDER BY name")
                .bind(account_id)
                .fetch_all(&self.backend)
                .await?;
        Ok(rows
            .iter()
            .map(|r| NotebookRow {
                id: r.get_string("id"),
                account_id: r.get_string("account_id"),
                name: r.get_string("name"),
            })
            .collect())
    }

    /// Insert or replace a notebook.
    pub async fn upsert_notebook(&self, row: &NotebookRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO notebooks (id, account_id, name) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name",
        )
        .bind(&row.id)
        .bind(&row.account_id)
        .bind(&row.name)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    // ── notes (sealed at rest) ──────────────────────────────────────────────

    /// Fetch one note with its body **unsealed** (plan §1.6).
    pub async fn get_note(&self, id: &str) -> Result<Option<NoteRow>, StoreError> {
        let row = q(
            "SELECT id, account_id, notebook_id, title, tags_json, color, pinned,
                    body_html_sealed, body_text_sealed, links_json, created_at, updated_at,
                    title_sealed, tags_json_sealed, color_sealed, pinned_sealed
             FROM notes WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
        .await?;
        match row {
            Some(r) => Ok(Some(self.note_from_row(&r)?)),
            None => Ok(None),
        }
    }

    /// List an account's notes (unsealed metadata + bodies), pinned first.
    pub async fn list_notes(&self, account_id: &str) -> Result<Vec<NoteRow>, StoreError> {
        let rows = q(
            "SELECT id, account_id, notebook_id, title, tags_json, color, pinned,
                    body_html_sealed, body_text_sealed, links_json, created_at, updated_at,
                    title_sealed, tags_json_sealed, color_sealed, pinned_sealed
             FROM notes WHERE account_id = ?1 ORDER BY updated_at DESC, id",
        )
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        let mut notes = rows
            .iter()
            .map(|r| self.note_from_row(r))
            .collect::<Result<Vec<NoteRow>, StoreError>>()?;
        // 0019 (26.17): `pinned` is sealed, so ordering can no longer be a SQL
        // `ORDER BY pinned DESC`. Re-home the pinned-first ordering to a STABLE
        // sort after decrypt — it preserves the SQL `updated_at DESC, id` order
        // within each pinned group, reproducing the pre-seal visual order exactly.
        // Notes are few per account, so decrypt-then-sort is cheap. `sort_by_key`
        // is a STABLE sort; `Reverse` puts pinned (`true`) first.
        notes.sort_by_key(|n| std::cmp::Reverse(n.pinned));
        Ok(notes)
    }

    /// Unseal a note row's sealed columns into a [`NoteRow`]. The body and (0019)
    /// the `title`/`tags_json`/`color`/`pinned` metadata come from the `*_sealed`
    /// BLOBs; each falls back to its frozen plaintext column only for a
    /// not-yet-backfilled row (`*_sealed IS NULL`) — belt-and-braces for the
    /// store-open backfill window.
    fn note_from_row(&self, r: &Row) -> Result<NoteRow, StoreError> {
        let body_html = self.unseal_opt(r.get_opt_blob("body_html_sealed"))?;
        let body_text = self.unseal_opt(r.get_opt_blob("body_text_sealed"))?;
        let title = match r.get_opt_blob("title_sealed") {
            Some(sealed) => self.unseal_opt(Some(sealed))?,
            None => r.get_string("title"),
        };
        let tags_json = match r.get_opt_blob("tags_json_sealed") {
            Some(sealed) => self.unseal_opt(Some(sealed))?,
            None => r.get_string("tags_json"),
        };
        let color = match r.get_opt_blob("color_sealed") {
            Some(sealed) => self.unseal_opt(Some(sealed))?,
            None => r.get_string("color"),
        };
        let pinned = match r.get_opt_blob("pinned_sealed") {
            Some(sealed) => self.unseal_opt(Some(sealed))? == "1",
            None => r.get_i64("pinned") != 0,
        };
        Ok(NoteRow {
            id: r.get_string("id"),
            account_id: r.get_string("account_id"),
            notebook_id: r.get_opt_string("notebook_id"),
            title,
            tags_json,
            color,
            pinned,
            body_html,
            body_text,
            links_json: r.get_string("links_json"),
            created_at: r.get_string("created_at"),
            updated_at: r.get_string("updated_at"),
        })
    }

    /// Unseal an optional sealed BLOB into a UTF-8 string (empty when NULL).
    fn unseal_opt(&self, sealed: Option<Vec<u8>>) -> Result<String, StoreError> {
        match sealed {
            None => Ok(String::new()),
            Some(bytes) if bytes.is_empty() => Ok(String::new()),
            Some(bytes) => {
                let plain = self.key.open(&bytes)?;
                String::from_utf8(plain).map_err(|e| StoreError::Corrupt(e.to_string()))
            }
        }
    }

    /// Insert or replace a note, **sealing** the body and (0019) the
    /// `title`/`tags_json`/`color`/`pinned` metadata into the `*_sealed` BLOBs.
    /// The frozen plaintext columns are written neutral placeholders
    /// (`''`/`'[]'`/`''`/`0`) so no plaintext note metadata survives at rest;
    /// [`Self::note_from_row`] reads back from the sealed columns.
    pub async fn upsert_note(&self, row: &NoteRow) -> Result<(), StoreError> {
        let body_html_sealed = self.key.seal(row.body_html.as_bytes())?;
        let body_text_sealed = self.key.seal(row.body_text.as_bytes())?;
        let title_sealed = self.key.seal(row.title.as_bytes())?;
        let tags_json_sealed = self.key.seal(row.tags_json.as_bytes())?;
        let color_sealed = self.key.seal(row.color.as_bytes())?;
        let pinned_sealed = self.key.seal(pinned_canonical(row.pinned))?;
        q("INSERT INTO notes
                 (id, account_id, notebook_id, title, tags_json, color, pinned,
                  body_html_sealed, body_text_sealed, links_json, created_at, updated_at,
                  title_sealed, tags_json_sealed, color_sealed, pinned_sealed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
             ON CONFLICT(id) DO UPDATE SET
                 notebook_id = excluded.notebook_id, title = excluded.title,
                 tags_json = excluded.tags_json, color = excluded.color, pinned = excluded.pinned,
                 body_html_sealed = excluded.body_html_sealed,
                 body_text_sealed = excluded.body_text_sealed,
                 links_json = excluded.links_json, updated_at = excluded.updated_at,
                 title_sealed = excluded.title_sealed, tags_json_sealed = excluded.tags_json_sealed,
                 color_sealed = excluded.color_sealed, pinned_sealed = excluded.pinned_sealed")
        .bind(&row.id)
        .bind(&row.account_id)
        .bind(row.notebook_id.as_deref())
        // Plaintext metadata columns blanked to neutral defaults — the sealed
        // columns are authoritative.
        .bind("")
        .bind("[]")
        .bind("")
        .bind(0_i64)
        .bind(body_html_sealed)
        .bind(body_text_sealed)
        .bind(&row.links_json)
        .bind(&row.created_at)
        .bind(&row.updated_at)
        .bind(title_sealed)
        .bind(tags_json_sealed)
        .bind(color_sealed)
        .bind(pinned_sealed)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Idempotent one-shot backfill (JWZ-maintenance pattern) that seals the 0019
    /// Note metadata for any pre-0019 row: a note carrying the frozen plaintext
    /// columns but no sealed counterpart (`title_sealed IS NULL`) is sealed under
    /// the [`crate::ServerKey`] and its plaintext columns blanked, so no plaintext
    /// note metadata survives at rest. Invoked at [`Store::open`]; re-running is a
    /// no-op once every row is sealed.
    ///
    /// Returns the number of rows sealed this run. A non-zero count means plaintext
    /// was just blanked in place, so the prior bytes still sit in SQLite free/overflow
    /// pages (and WAL) or Postgres dead tuples until a `VACUUM` reclaims them — the
    /// caller uses this to decide whether to run [`Store::reclaim_note_metadata_residue`]
    /// (R2).
    pub(crate) async fn seal_note_metadata_backfill(&self) -> Result<usize, StoreError> {
        let rows =
            q("SELECT id, title, tags_json, color, pinned FROM notes WHERE title_sealed IS NULL")
                .fetch_all(&self.backend)
                .await?;
        for r in &rows {
            let id = r.get_string("id");
            let title_sealed = self.key.seal(r.get_string("title").as_bytes())?;
            let tags_json_sealed = self.key.seal(r.get_string("tags_json").as_bytes())?;
            let color_sealed = self.key.seal(r.get_string("color").as_bytes())?;
            let pinned_sealed = self.key.seal(pinned_canonical(r.get_i64("pinned") != 0))?;
            q("UPDATE notes SET
                   title_sealed = ?2, tags_json_sealed = ?3, color_sealed = ?4, pinned_sealed = ?5,
                   title = '', tags_json = '[]', color = '', pinned = 0
               WHERE id = ?1")
            .bind(&id)
            .bind(title_sealed)
            .bind(tags_json_sealed)
            .bind(color_sealed)
            .bind(pinned_sealed)
            .execute(&self.backend)
            .await?;
        }
        Ok(rows.len())
    }

    /// Delete a note.
    pub async fn delete_note(&self, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM notes WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ── address books + contacts ────────────────────────────────────────────

    /// List an account's address books.
    pub async fn list_address_books(
        &self,
        account_id: &str,
    ) -> Result<Vec<AddressBookRow>, StoreError> {
        let rows = q(
            "SELECT id, account_id, name, is_default, carddav_url, sync_token, ctag
             FROM address_books WHERE account_id = ?1 ORDER BY is_default DESC, name",
        )
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(address_book_from_row).collect())
    }

    /// Fetch one address book by id.
    pub async fn get_address_book(&self, id: &str) -> Result<Option<AddressBookRow>, StoreError> {
        let row = q(
            "SELECT id, account_id, name, is_default, carddav_url, sync_token, ctag
             FROM address_books WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(address_book_from_row))
    }

    /// Insert or replace an address book.
    pub async fn upsert_address_book(&self, row: &AddressBookRow) -> Result<(), StoreError> {
        q("INSERT INTO address_books
                 (id, account_id, name, is_default, carddav_url, sync_token, ctag)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name, is_default = excluded.is_default,
                 carddav_url = excluded.carddav_url, sync_token = excluded.sync_token,
                 ctag = excluded.ctag")
        .bind(&row.id)
        .bind(&row.account_id)
        .bind(&row.name)
        .bind(i64::from(row.is_default))
        .bind(row.carddav_url.as_deref())
        .bind(row.sync_token.as_deref())
        .bind(row.ctag.as_deref())
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Delete an address book (cascades contacts + groups).
    pub async fn delete_address_book(&self, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM address_books WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// List an address book's contacts.
    pub async fn list_contacts(
        &self,
        address_book_id: &str,
    ) -> Result<Vec<ContactRow>, StoreError> {
        let rows = q(
            "SELECT id, address_book_id, uid, etag, vcard_raw, json, full_name, is_favorite,
                    photo_blob_id, pgp_key, smime_cert
             FROM contacts WHERE address_book_id = ?1 ORDER BY full_name, id",
        )
        .bind(address_book_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(contact_from_row).collect())
    }

    /// Fetch one contact by id.
    pub async fn get_contact(&self, id: &str) -> Result<Option<ContactRow>, StoreError> {
        let row = q(
            "SELECT id, address_book_id, uid, etag, vcard_raw, json, full_name, is_favorite,
                    photo_blob_id, pgp_key, smime_cert
             FROM contacts WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(contact_from_row))
    }

    /// Insert or replace a contact (source of truth = `vcard_raw`).
    pub async fn upsert_contact(&self, row: &ContactRow) -> Result<(), StoreError> {
        q("INSERT INTO contacts
                 (id, address_book_id, uid, etag, vcard_raw, json, full_name, is_favorite,
                  photo_blob_id, pgp_key, smime_cert)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
                 address_book_id = excluded.address_book_id, uid = excluded.uid,
                 etag = excluded.etag, vcard_raw = excluded.vcard_raw, json = excluded.json,
                 full_name = excluded.full_name, is_favorite = excluded.is_favorite,
                 photo_blob_id = excluded.photo_blob_id, pgp_key = excluded.pgp_key,
                 smime_cert = excluded.smime_cert")
        .bind(&row.id)
        .bind(&row.address_book_id)
        .bind(&row.uid)
        .bind(row.etag.as_deref())
        .bind(&row.vcard_raw)
        .bind(row.json.as_deref())
        .bind(&row.full_name)
        .bind(i64::from(row.is_favorite))
        .bind(row.photo_blob_id.as_deref())
        .bind(row.pgp_key.as_deref())
        .bind(row.smime_cert.as_deref())
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Delete a contact.
    pub async fn delete_contact(&self, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM contacts WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Prefix-match contacts by name/email for Compose autocomplete
    /// (`ContactCard/autocomplete`, §2.2), ranked by favorite then name. The
    /// email match scans `vcard_raw` (the `EMAIL:` lines) since emails are a
    /// projected column set, not a top-level searchable field.
    pub async fn autocomplete_contacts(
        &self,
        account_id: &str,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<ContactRow>, StoreError> {
        let like = format!("%{}%", prefix.replace('%', "\\%").replace('_', "\\_"));
        // SQLite `LIKE` is ASCII case-insensitive; Postgres needs `ILIKE` for the
        // same behaviour (plan §1.1). Two frozen static strings keep the authored
        // SQL a `&'static str` while matching each backend's semantics.
        let sql = match self.backend.dialect() {
            crate::backend::Dialect::Sqlite => {
                "SELECT c.id AS id, c.address_book_id AS address_book_id, c.uid AS uid, c.etag AS etag,
                        c.vcard_raw AS vcard_raw, c.json AS json, c.full_name AS full_name,
                        c.is_favorite AS is_favorite, c.photo_blob_id AS photo_blob_id,
                        c.pgp_key AS pgp_key, c.smime_cert AS smime_cert
                 FROM contacts c
                 JOIN address_books ab ON ab.id = c.address_book_id
                 WHERE ab.account_id = ?1
                   AND (c.full_name LIKE ?2 ESCAPE '\\' OR c.vcard_raw LIKE ?2 ESCAPE '\\')
                 ORDER BY c.is_favorite DESC, c.full_name, c.id
                 LIMIT ?3"
            }
            crate::backend::Dialect::Postgres => {
                "SELECT c.id AS id, c.address_book_id AS address_book_id, c.uid AS uid, c.etag AS etag,
                        c.vcard_raw AS vcard_raw, c.json AS json, c.full_name AS full_name,
                        c.is_favorite AS is_favorite, c.photo_blob_id AS photo_blob_id,
                        c.pgp_key AS pgp_key, c.smime_cert AS smime_cert
                 FROM contacts c
                 JOIN address_books ab ON ab.id = c.address_book_id
                 WHERE ab.account_id = ?1
                   AND (c.full_name ILIKE ?2 ESCAPE '\\' OR c.vcard_raw ILIKE ?2 ESCAPE '\\')
                 ORDER BY c.is_favorite DESC, c.full_name, c.id
                 LIMIT ?3"
            }
        };
        let rows = q(sql)
            .bind(account_id)
            .bind(&like)
            .bind(limit.max(0))
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(contact_from_row).collect())
    }

    /// List an address book's contact groups.
    pub async fn list_contact_groups(
        &self,
        address_book_id: &str,
    ) -> Result<Vec<ContactGroupRow>, StoreError> {
        let rows = q("SELECT id, address_book_id, name, member_ids_json
             FROM contact_groups WHERE address_book_id = ?1 ORDER BY name")
        .bind(address_book_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(contact_group_from_row).collect())
    }

    /// Fetch one contact group by id.
    pub async fn get_contact_group(&self, id: &str) -> Result<Option<ContactGroupRow>, StoreError> {
        let row = q(
            "SELECT id, address_book_id, name, member_ids_json FROM contact_groups WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(contact_group_from_row))
    }

    /// Insert or replace a contact group.
    pub async fn upsert_contact_group(&self, row: &ContactGroupRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO contact_groups (id, address_book_id, name, member_ids_json)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                 address_book_id = excluded.address_book_id, name = excluded.name,
                 member_ids_json = excluded.member_ids_json",
        )
        .bind(&row.id)
        .bind(&row.address_book_id)
        .bind(&row.name)
        .bind(&row.member_ids_json)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Delete a contact group.
    pub async fn delete_contact_group(&self, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM contact_groups WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ── pim_changes (state tokens + `*/changes`) ────────────────────────────

    /// Append one PIM change and return the new `(account, type)` state. A single
    /// atomic `INSERT … SELECT MAX+1 … RETURNING` (same WAL-safe discipline as the
    /// mail change log, [`Store::record_change`]).
    pub async fn record_pim_change(
        &self,
        account_id: &str,
        type_name: &str,
        object_id: &str,
        op: &str,
    ) -> Result<u64, StoreError> {
        let now = chrono::Utc::now().to_rfc3339();
        let next = q(
            "INSERT INTO pim_changes (account_id, type, state, object_id, op, at)
             VALUES (
                 ?1, ?2,
                 (SELECT COALESCE(MAX(state), 0) + 1 FROM pim_changes WHERE account_id = ?1 AND type = ?2),
                 ?3, ?4, ?5
             )
             RETURNING state",
        )
        .bind(account_id)
        .bind(type_name)
        .bind(object_id)
        .bind(op)
        .bind(&now)
        .fetch_scalar_i64(&self.backend)
        .await?;
        Ok(next as u64)
    }

    /// The current `(account, type)` PIM state counter (0 when none).
    pub async fn current_pim_state(
        &self,
        account_id: &str,
        type_name: &str,
    ) -> Result<u64, StoreError> {
        let n = q(
            "SELECT COALESCE(MAX(state), 0) FROM pim_changes WHERE account_id = ?1 AND type = ?2",
        )
        .bind(account_id)
        .bind(type_name)
        .fetch_scalar_i64(&self.backend)
        .await?;
        Ok(n as u64)
    }

    /// The PIM change rows for a datatype since `since_state` (the diff input),
    /// oldest-first.
    pub async fn pim_changes_since(
        &self,
        account_id: &str,
        type_name: &str,
        since: u64,
    ) -> Result<Vec<PimChangeRow>, StoreError> {
        let rows = q("SELECT state, object_id, op FROM pim_changes
             WHERE account_id = ?1 AND type = ?2 AND state > ?3 ORDER BY state ASC")
        .bind(account_id)
        .bind(type_name)
        .bind(since as i64)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| PimChangeRow {
                state: r.get_i64("state") as u64,
                object_id: r.get_string("object_id"),
                op: r.get_string("op"),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccountKind, Credentials, NewAccount, ServerKey, Store};

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    async fn account(s: &Store) -> String {
        s.create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "h",
                port: 993,
                tls: "implicit",
                username: "u",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "u".into(),
                password: "p".into(),
            },
        )
        .await
        .unwrap()
    }

    fn calendar(id: &str, account_id: &str, component: &str) -> CalendarRow {
        CalendarRow {
            id: id.into(),
            account_id: account_id.into(),
            name: "Personal".into(),
            color: "#3366ff".into(),
            sort_order: 0,
            is_visible: true,
            role: Some("default".into()),
            caldav_url: None,
            sync_token: None,
            ctag: None,
            is_overlay: false,
            component: component.into(),
        }
    }

    #[tokio::test]
    async fn calendars_and_events_round_trip() {
        let s = store().await;
        let a = account(&s).await;
        s.upsert_calendar(&calendar("cal1", &a, "VEVENT"))
            .await
            .unwrap();
        assert_eq!(s.list_calendars(&a).await.unwrap().len(), 1);
        assert_eq!(
            s.get_calendar("cal1").await.unwrap().unwrap().name,
            "Personal"
        );

        let ev = EventRow {
            id: "ev1".into(),
            calendar_id: "cal1".into(),
            uid: "uid-1".into(),
            etag: Some("\"e1\"".into()),
            ical_raw: "BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n".into(),
            start_utc: Some("2026-07-11T09:00:00Z".into()),
            end_utc: Some("2026-07-11T10:00:00Z".into()),
            tzid: Some("UTC".into()),
            rrule: None,
            status: "confirmed".into(),
            json: Some(b"{}".to_vec()),
        };
        s.upsert_event(&ev).await.unwrap();
        s.replace_event_instances(
            "ev1",
            &[EventInstanceRow {
                event_id: "ev1".into(),
                instance_start_utc: "2026-07-11T09:00:00Z".into(),
                instance_end_utc: "2026-07-11T10:00:00Z".into(),
            }],
        )
        .await
        .unwrap();

        assert_eq!(s.list_events("cal1").await.unwrap().len(), 1);
        let in_range = s
            .events_in_range(&a, "2026-07-11T00:00:00Z", "2026-07-12T00:00:00Z")
            .await
            .unwrap();
        assert_eq!(in_range.len(), 1);
        // Out-of-window returns nothing.
        assert_eq!(
            s.events_in_range(&a, "2026-08-01T00:00:00Z", "2026-08-02T00:00:00Z")
                .await
                .unwrap()
                .len(),
            0
        );
        s.delete_event("ev1").await.unwrap();
        assert_eq!(s.list_events("cal1").await.unwrap().len(), 0);
        // Instances cascade with the event.
        assert_eq!(
            s.events_in_range(&a, "2026-07-11T00:00:00Z", "2026-07-12T00:00:00Z")
                .await
                .unwrap()
                .len(),
            0
        );
    }

    #[tokio::test]
    async fn notes_seal_at_rest_and_decrypt() {
        let s = store().await;
        let a = account(&s).await;
        let note = NoteRow {
            id: "n1".into(),
            account_id: a.clone(),
            notebook_id: None,
            title: "Groceries CONFIDENTIAL".into(),
            tags_json: "[\"home SECRETTAG\"]".into(),
            color: "#ffcc00".into(),
            pinned: true,
            body_html: "<p>milk SUPERSECRET eggs</p>".into(),
            body_text: "milk SUPERSECRET eggs".into(),
            links_json: "[]".into(),
            created_at: "2026-07-11T00:00:00Z".into(),
            updated_at: "2026-07-11T00:00:00Z".into(),
        };
        s.upsert_note(&note).await.unwrap();

        // Every user field decrypts on read (title/tags/color/pinned + body).
        let got = s.get_note("n1").await.unwrap().unwrap();
        assert_eq!(got.title, "Groceries CONFIDENTIAL");
        assert_eq!(got.tags_json, "[\"home SECRETTAG\"]");
        assert_eq!(got.color, "#ffcc00");
        assert!(got.pinned);
        assert_eq!(got.body_text, "milk SUPERSECRET eggs");

        // The plaintext metadata columns are BLANKED to neutral defaults — the
        // sealed columns are authoritative, no plaintext survives at rest.
        let row = q("SELECT title, tags_json, color, pinned,
                            title_sealed, tags_json_sealed, color_sealed, pinned_sealed,
                            body_text_sealed
                     FROM notes WHERE id = ?1")
        .bind("n1")
        .fetch_one(s.backend())
        .await
        .unwrap();
        assert_eq!(row.get_string("title"), "");
        assert_eq!(row.get_string("tags_json"), "[]");
        assert_eq!(row.get_string("color"), "");
        assert_eq!(row.get_i64("pinned"), 0);

        // The sealed BLOBs are ciphertext — no plaintext secret is present.
        let scan = |col: &str| {
            let blob = row.get_blob(col);
            assert!(
                !blob.windows(11).any(|w| w == b"SUPERSECRET")
                    && !blob.windows(12).any(|w| w == b"CONFIDENTIAL")
                    && !blob.windows(9).any(|w| w == b"SECRETTAG"),
                "{col} must be sealed at rest (no plaintext leak)"
            );
        };
        scan("title_sealed");
        scan("tags_json_sealed");
        scan("color_sealed");
        scan("pinned_sealed");
        scan("body_text_sealed");
    }

    #[tokio::test]
    async fn list_notes_pinned_first_then_updated_at() {
        let s = store().await;
        let a = account(&s).await;
        let mk = |id: &str, pinned: bool, updated_at: &str| NoteRow {
            id: id.into(),
            account_id: a.clone(),
            notebook_id: None,
            title: id.into(),
            tags_json: "[]".into(),
            color: "".into(),
            pinned,
            body_html: "".into(),
            body_text: "".into(),
            links_json: "[]".into(),
            created_at: "2026-07-01T00:00:00Z".into(),
            updated_at: updated_at.into(),
        };
        // Insert in shuffled order; the sealed pinned flag means ordering is a
        // Rust STABLE sort over the SQL `updated_at DESC, id` order.
        s.upsert_note(&mk("na", false, "2026-07-03T00:00:00Z"))
            .await
            .unwrap();
        s.upsert_note(&mk("nb", true, "2026-07-01T00:00:00Z"))
            .await
            .unwrap();
        s.upsert_note(&mk("nc", false, "2026-07-02T00:00:00Z"))
            .await
            .unwrap();
        s.upsert_note(&mk("nd", true, "2026-07-04T00:00:00Z"))
            .await
            .unwrap();

        // Pinned first (updated_at DESC within group), then the rest — the exact
        // order the pre-seal `ORDER BY pinned DESC, updated_at DESC, id` produced.
        let ids: Vec<String> = s
            .list_notes(&a)
            .await
            .unwrap()
            .into_iter()
            .map(|n| n.id)
            .collect();
        assert_eq!(ids, vec!["nd", "nb", "na", "nc"]);
    }

    #[tokio::test]
    async fn seal_note_metadata_backfill_idempotent() {
        let s = store().await;
        let a = account(&s).await;
        // Simulate a pre-0019 row: plaintext metadata, all `*_sealed` NULL.
        q(
            "INSERT INTO notes (id, account_id, title, tags_json, color, pinned,
                              links_json, created_at, updated_at)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, '[]', ?7, ?7)",
        )
        .bind("legacy1")
        .bind(&a)
        .bind("Legacy LEAKME")
        .bind("[\"old\"]")
        .bind("#123456")
        .bind(1_i64)
        .bind("2026-07-01T00:00:00Z")
        .execute(s.backend())
        .await
        .unwrap();

        // The backfill reports the number of rows it sealed this run.
        assert_eq!(s.seal_note_metadata_backfill().await.unwrap(), 1);

        // Now sealed + decryptable, plaintext blanked.
        let got = s.get_note("legacy1").await.unwrap().unwrap();
        assert_eq!(got.title, "Legacy LEAKME");
        assert_eq!(got.tags_json, "[\"old\"]");
        assert_eq!(got.color, "#123456");
        assert!(got.pinned);

        let after = q("SELECT title, pinned, title_sealed FROM notes WHERE id = ?1")
            .bind("legacy1")
            .fetch_one(s.backend())
            .await
            .unwrap();
        assert_eq!(after.get_string("title"), "");
        assert_eq!(after.get_i64("pinned"), 0);
        let sealed_after_first = after.get_blob("title_sealed");
        assert!(!sealed_after_first.is_empty());
        assert!(
            !sealed_after_first.windows(6).any(|w| w == b"LEAKME"),
            "backfilled title must be sealed at rest"
        );

        // Re-running touches nothing (WHERE title_sealed IS NULL matches no rows) →
        // count 0, which is what gates the store-open auto-reclaim off.
        assert_eq!(s.seal_note_metadata_backfill().await.unwrap(), 0);
        let sealed_after_second = q("SELECT title_sealed FROM notes WHERE id = ?1")
            .bind("legacy1")
            .fetch_one(s.backend())
            .await
            .unwrap()
            .get_blob("title_sealed");
        assert_eq!(
            sealed_after_first, sealed_after_second,
            "backfill must be idempotent — a sealed row is not re-sealed"
        );
    }

    #[tokio::test]
    async fn reclaim_note_metadata_residue_vacuums_and_notes_survive() {
        // R2: the dialect-aware reclaim (SQLite whole-DB VACUUM here) must run clean
        // and leave every note fully readable afterward.
        let s = store().await;
        let a = account(&s).await;
        let note = NoteRow {
            id: "n1".into(),
            account_id: a.clone(),
            notebook_id: None,
            title: "Kept After Vacuum".into(),
            tags_json: "[\"t\"]".into(),
            color: "#abcdef".into(),
            pinned: true,
            body_html: "<p>body</p>".into(),
            body_text: "body".into(),
            links_json: "[]".into(),
            created_at: "2026-07-11T00:00:00Z".into(),
            updated_at: "2026-07-11T00:00:00Z".into(),
        };
        s.upsert_note(&note).await.unwrap();

        // Runs unconditionally (mirrors the `maintenance vacuum` CLI) and succeeds.
        s.reclaim_note_metadata_residue().await.unwrap();

        // The note still round-trips after the VACUUM rewrite.
        let got = s.get_note("n1").await.unwrap().unwrap();
        assert_eq!(got.title, "Kept After Vacuum");
        assert_eq!(got.tags_json, "[\"t\"]");
        assert_eq!(got.color, "#abcdef");
        assert!(got.pinned);
        assert_eq!(got.body_text, "body");

        // Re-running is safe (idempotent operator remedy).
        s.reclaim_note_metadata_residue().await.unwrap();
    }

    #[tokio::test]
    async fn contacts_and_autocomplete_ranking() {
        let s = store().await;
        let a = account(&s).await;
        s.upsert_address_book(&AddressBookRow {
            id: "ab1".into(),
            account_id: a.clone(),
            name: "Contacts".into(),
            is_default: true,
            carddav_url: None,
            sync_token: None,
            ctag: None,
        })
        .await
        .unwrap();

        let mk = |id: &str, name: &str, email: &str, fav: bool| ContactRow {
            id: id.into(),
            address_book_id: "ab1".into(),
            uid: id.into(),
            etag: None,
            vcard_raw: format!("BEGIN:VCARD\r\nFN:{name}\r\nEMAIL:{email}\r\nEND:VCARD\r\n"),
            json: None,
            full_name: name.into(),
            is_favorite: fav,
            photo_blob_id: None,
            pgp_key: None,
            smime_cert: None,
        };
        s.upsert_contact(&mk("c1", "Ada Lovelace", "ada@x.test", false))
            .await
            .unwrap();
        s.upsert_contact(&mk("c2", "Alan Turing", "alan@x.test", true))
            .await
            .unwrap();

        // Prefix "Al" matches Alan; favorite ranks first among ties.
        let hits = s.autocomplete_contacts(&a, "Al", 10).await.unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "c2");

        // Email substring match works via the vCard scan.
        let by_email = s.autocomplete_contacts(&a, "ada@", 10).await.unwrap();
        assert_eq!(by_email.len(), 1);
        assert_eq!(by_email[0].id, "c1");
    }

    #[tokio::test]
    async fn pim_change_log_states_and_diff() {
        let s = store().await;
        let a = account(&s).await;
        assert_eq!(s.current_pim_state(&a, "CalendarEvent").await.unwrap(), 0);
        let s1 = s
            .record_pim_change(&a, "CalendarEvent", "ev1", "created")
            .await
            .unwrap();
        let s2 = s
            .record_pim_change(&a, "CalendarEvent", "ev2", "created")
            .await
            .unwrap();
        assert_eq!((s1, s2), (1, 2));
        assert_eq!(s.current_pim_state(&a, "CalendarEvent").await.unwrap(), 2);
        let diff = s.pim_changes_since(&a, "CalendarEvent", 1).await.unwrap();
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].object_id, "ev2");
        // A different type keeps its own counter.
        assert_eq!(
            s.record_pim_change(&a, "Note", "n1", "created")
                .await
                .unwrap(),
            1
        );
    }
}
