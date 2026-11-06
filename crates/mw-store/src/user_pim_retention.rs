//! Retention for a person's calendar events and notes (table `user_pim_retention`).
//!
//! One table holds three kinds of row, all keyed by account:
//!
//! * `calendar` and `notes` rows carry the person's opt-in as `config_json`.
//! * `noteHold` rows hold a note a retention rule removed, for the undo window.
//!   The whole note is sealed into `payload`, so a held note is as protected as
//!   one in `notes`. `due_at` is the unix time the hold is purged.
//!
//! Nothing here decides what to delete: the server's job passes the cutoffs.

use crate::backend::q;
use crate::{NoteRow, Store, StoreError};

/// An event that ended before a cutoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndedEvent {
    pub id: String,
    pub calendar_id: String,
    pub end_utc: String,
}

/// A held note, without its content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldNote {
    pub note_id: String,
    pub title: String,
    pub held_at: i64,
    pub due_at: i64,
}

fn cfg_id(account: &str, kind: &str) -> String {
    format!("{kind}:{account}")
}

fn note_json(n: &NoteRow) -> serde_json::Value {
    serde_json::json!({
        "id": n.id, "accountId": n.account_id, "notebookId": n.notebook_id, "title": n.title,
        "tags": n.tags_json, "color": n.color, "pinned": n.pinned, "html": n.body_html,
        "text": n.body_text, "links": n.links_json, "created": n.created_at, "updated": n.updated_at,
    })
}

fn note_of(v: &serde_json::Value) -> Option<NoteRow> {
    let s = |k: &str| v[k].as_str().map(str::to_string);
    Some(NoteRow {
        id: s("id")?,
        account_id: s("accountId")?,
        notebook_id: s("notebookId"),
        title: s("title")?,
        tags_json: s("tags")?,
        color: s("color")?,
        pinned: v["pinned"].as_bool()?,
        body_html: s("html")?,
        body_text: s("text")?,
        links_json: s("links")?,
        created_at: s("created")?,
        updated_at: s("updated")?,
    })
}

impl Store {
    /// The person's stored opt-in for `kind` (`calendar` or `notes`).
    pub async fn pim_retention_config(&self, account: &str, kind: &str) -> Result<Option<String>, StoreError> {
        let row = q("SELECT config_json FROM user_pim_retention WHERE id = ?1")
            .bind(cfg_id(account, kind))
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.map(|r| r.get_string("config_json")))
    }

    pub async fn pim_retention_config_put(&self, account: &str, kind: &str, config_json: &str, now: i64) -> Result<(), StoreError> {
        q("INSERT INTO user_pim_retention(id,account_id,kind,item_ref,config_json,payload,due_at,updated_at)
           VALUES(?1,?2,?3,'',?4,?5,0,?6)
           ON CONFLICT(id) DO UPDATE SET config_json=excluded.config_json, updated_at=excluded.updated_at")
            .bind(cfg_id(account, kind))
            .bind(account)
            .bind(kind)
            .bind(config_json)
            .bind(Vec::<u8>::new())
            .bind(now)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Non-recurring events of the account that ended before `cutoff_utc`
    /// (`YYYY-MM-DDTHH:MM:SSZ`), oldest first. Recurring events have no end.
    pub async fn events_ended_before(&self, account: &str, cutoff_utc: &str, limit: i64) -> Result<Vec<EndedEvent>, StoreError> {
        let rows = q("SELECT e.id AS id, e.calendar_id AS calendar_id, e.end_utc AS end_utc
             FROM events e JOIN calendars c ON c.id = e.calendar_id
             WHERE c.account_id = ?1 AND e.end_utc IS NOT NULL AND e.end_utc < ?2
               AND (e.rrule IS NULL OR e.rrule = '')
             ORDER BY e.end_utc LIMIT ?3")
            .bind(account)
            .bind(cutoff_utc)
            .bind(limit)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows
            .iter()
            .map(|r| EndedEvent { id: r.get_string("id"), calendar_id: r.get_string("calendar_id"), end_utc: r.get_string("end_utc") })
            .collect())
    }

    /// Ids of the account's notes last changed before `cutoff` (the `updated_at`
    /// text compares as RFC 3339), pinned notes excluded.
    pub async fn notes_untouched_since(&self, account: &str, cutoff: &str, limit: i64) -> Result<Vec<String>, StoreError> {
        let rows = q("SELECT id, updated_at FROM notes WHERE account_id = ?1 AND updated_at < ?2 ORDER BY updated_at LIMIT ?3")
            .bind(account)
            .bind(cutoff)
            .bind(limit * 4)
            .fetch_all(&self.backend)
            .await?;
        let mut out = Vec::new();
        for r in &rows {
            let id = r.get_string("id");
            // Pinned state is sealed, so it is read through the note.
            if let Some(n) = self.get_note(&id).await?
                && !n.pinned
            {
                out.push(id);
                if out.len() as i64 >= limit {
                    break;
                }
            }
        }
        Ok(out)
    }

    /// Save a sealed copy of a note in the hold. The caller removes the note
    /// afterwards (through the engine, so clients see the change), only once this
    /// has succeeded. `Ok(false)` when the note is gone or belongs to someone else.
    pub async fn note_hold_copy(&self, account: &str, note_id: &str, now: i64, due_at: i64) -> Result<bool, StoreError> {
        let Some(n) = self.get_note(note_id).await? else { return Ok(false) };
        if n.account_id != account {
            return Ok(false);
        }
        let payload = self.key.seal(note_json(&n).to_string().as_bytes())?;
        q("INSERT INTO user_pim_retention(id,account_id,kind,item_ref,config_json,payload,due_at,updated_at)
           VALUES(?1,?2,'noteHold',?3,?4,?5,?6,?7)
           ON CONFLICT(id) DO UPDATE SET payload=excluded.payload, due_at=excluded.due_at, updated_at=excluded.updated_at")
            .bind(format!("noteHold:{note_id}"))
            .bind(account)
            .bind(note_id)
            .bind(serde_json::json!({"held": now}).to_string())
            .bind(payload)
            .bind(due_at)
            .bind(now)
            .execute(&self.backend)
            .await?;
        Ok(true)
    }

    /// The account's held notes (titles only) for the undo list.
    pub async fn note_holds(&self, account: &str) -> Result<Vec<HeldNote>, StoreError> {
        let rows = q("SELECT item_ref, payload, due_at, updated_at FROM user_pim_retention WHERE account_id = ?1 AND kind = 'noteHold' ORDER BY updated_at DESC")
            .bind(account)
            .fetch_all(&self.backend)
            .await?;
        let mut out = Vec::new();
        for r in &rows {
            let plain = self.key.open(&r.get_opt_blob("payload").unwrap_or_default())?;
            let title = serde_json::from_slice::<serde_json::Value>(&plain).ok().and_then(|v| v["title"].as_str().map(str::to_string)).unwrap_or_default();
            out.push(HeldNote { note_id: r.get_string("item_ref"), title, held_at: r.get_i64("updated_at"), due_at: r.get_i64("due_at") });
        }
        Ok(out)
    }

    /// Put a held note back. `Ok(false)` when there is no such hold.
    pub async fn note_restore(&self, account: &str, note_id: &str) -> Result<bool, StoreError> {
        let id = format!("noteHold:{note_id}");
        let Some(r) = q("SELECT payload FROM user_pim_retention WHERE id = ?1 AND account_id = ?2")
            .bind(&id)
            .bind(account)
            .fetch_optional(&self.backend)
            .await?
        else {
            return Ok(false);
        };
        let plain = self.key.open(&r.get_opt_blob("payload").unwrap_or_default())?;
        let note = serde_json::from_slice::<serde_json::Value>(&plain).ok().and_then(|v| note_of(&v));
        let Some(note) = note else { return Ok(false) };
        self.upsert_note(&note).await?;
        q("DELETE FROM user_pim_retention WHERE id = ?1").bind(&id).execute(&self.backend).await?;
        Ok(true)
    }

    /// Delete holds past their due time. Returns how many.
    pub async fn note_holds_purge(&self, now: i64) -> Result<u64, StoreError> {
        Ok(q("DELETE FROM user_pim_retention WHERE kind = 'noteHold' AND due_at <= ?1").bind(now).execute(&self.backend).await?)
    }

    /// Remove every row of an account (account deletion).
    pub async fn pim_retention_forget_account(&self, account: &str) -> Result<(), StoreError> {
        q("DELETE FROM user_pim_retention WHERE account_id = ?1").bind(account).execute(&self.backend).await?;
        Ok(())
    }
}
