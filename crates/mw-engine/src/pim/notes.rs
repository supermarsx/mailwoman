//! `Note/*` (frozen §2.2): Mailwoman-native notes whose rich-text body is sealed
//! at rest by the `mw-store` seal (plan §1.6) — the store transparently
//! seals on write / unseals on read. Title / tags / color / pinned are plaintext
//! columns so `Note/query` can filter on them; body search is a decrypt-scan
//! over the (small) note set.

use mw_store::NoteRow;
use serde_json::{Value, json};

use crate::backend::{EngineError, Result};
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;

use super::{
    SetOutcome, gen_id, get_response, now_rfc3339, query_response, server_fail, set_error,
    wanted_ids,
};

impl Engine {
    // ── Note/get ─────────────────────────────────────────────────────────────

    pub(crate) async fn note_get(&self, account_id: &str, args: &Value) -> Value {
        let state = self
            .pim_type_state(account_id, ChangeType::Note)
            .await
            .unwrap_or_default();
        let ids = match wanted_ids(args) {
            Some(ids) => ids,
            None => match self.store().list_notes(account_id).await {
                Ok(v) => v.into_iter().map(|n| n.id).collect(),
                Err(e) => return server_fail(e),
            },
        };
        let mut list = Vec::new();
        let mut not_found = Vec::new();
        for id in &ids {
            match self.store().get_note(id).await {
                Ok(Some(row)) if row.account_id == account_id => list.push(note_row_to_json(&row)),
                Ok(_) => not_found.push(json!(id)),
                Err(e) => return server_fail(e),
            }
        }
        get_response(account_id, &state, list, not_found)
    }

    // ── Note/set ─────────────────────────────────────────────────────────────

    pub(crate) async fn note_set(&self, account_id: &str, args: &Value) -> Value {
        let old_state = self
            .pim_type_state(account_id, ChangeType::Note)
            .await
            .unwrap_or_default();
        let mut out = SetOutcome::default();

        if let Some(creates) = args.get("create").and_then(Value::as_object) {
            for (cid, spec) in creates {
                match self.note_create(account_id, spec).await {
                    Ok(id) => {
                        out.created.insert(cid.clone(), json!({ "id": id }));
                    }
                    Err(e) => {
                        out.not_created
                            .insert(cid.clone(), set_error("invalidProperties", e));
                    }
                }
            }
        }
        if let Some(updates) = args.get("update").and_then(Value::as_object) {
            for (id, patch) in updates {
                match self.note_update(account_id, id, patch).await {
                    Ok(()) => {
                        out.updated.insert(id.clone(), Value::Null);
                    }
                    Err(e) => {
                        out.not_updated
                            .insert(id.clone(), set_error("serverFail", e));
                    }
                }
            }
        }
        if let Some(destroys) = args.get("destroy").and_then(Value::as_array) {
            for id in destroys.iter().filter_map(Value::as_str) {
                match self.store().delete_note(id).await {
                    Ok(()) => {
                        let _ = self
                            .record_pim_change(
                                account_id,
                                ChangeType::Note,
                                id,
                                ChangeOp::Destroyed,
                            )
                            .await;
                        out.destroyed.push(json!(id));
                    }
                    Err(e) => {
                        out.not_destroyed
                            .insert(id.to_string(), set_error("serverFail", e));
                    }
                }
            }
        }

        let new_state = self
            .pim_type_state(account_id, ChangeType::Note)
            .await
            .unwrap_or_default();
        self.broadcast_state(account_id).await;
        out.into_response(account_id, &old_state, &new_state)
    }

    async fn note_create(&self, account_id: &str, spec: &Value) -> Result<String> {
        let id = gen_id("note");
        let now = now_rfc3339();
        let row = NoteRow {
            id: id.clone(),
            account_id: account_id.to_string(),
            notebook_id: spec
                .get("notebookId")
                .and_then(Value::as_str)
                .map(String::from),
            title: spec
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            tags_json: array_str(spec, "tags"),
            color: spec
                .get("color")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            pinned: spec.get("pinned").and_then(Value::as_bool).unwrap_or(false),
            body_html: spec
                .get("bodyHtml")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            body_text: spec
                .get("bodyText")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            links_json: array_str(spec, "links"),
            created_at: now.clone(),
            updated_at: now,
        };
        self.store().upsert_note(&row).await?;
        self.record_pim_change(account_id, ChangeType::Note, &id, ChangeOp::Created)
            .await?;
        Ok(id)
    }

    async fn note_update(&self, account_id: &str, id: &str, patch: &Value) -> Result<()> {
        let mut row = self
            .store()
            .get_note(id)
            .await?
            .filter(|r| r.account_id == account_id)
            .ok_or_else(|| EngineError::Protocol(format!("unknown note {id}")))?;
        if let Some(v) = patch.get("title").and_then(Value::as_str) {
            row.title = v.to_string();
        }
        if let Some(v) = patch.get("color").and_then(Value::as_str) {
            row.color = v.to_string();
        }
        if let Some(v) = patch.get("pinned").and_then(Value::as_bool) {
            row.pinned = v;
        }
        if patch.get("tags").is_some() {
            row.tags_json = array_str(patch, "tags");
        }
        if patch.get("links").is_some() {
            row.links_json = array_str(patch, "links");
        }
        if let Some(v) = patch.get("notebookId") {
            row.notebook_id = v.as_str().map(String::from);
        }
        if let Some(v) = patch.get("bodyHtml").and_then(Value::as_str) {
            row.body_html = v.to_string();
        }
        if let Some(v) = patch.get("bodyText").and_then(Value::as_str) {
            row.body_text = v.to_string();
        }
        row.updated_at = now_rfc3339();
        self.store().upsert_note(&row).await?;
        self.record_pim_change(account_id, ChangeType::Note, id, ChangeOp::Updated)
            .await?;
        Ok(())
    }

    // ── Note/query (tags / pinned / text) ────────────────────────────────────

    pub(crate) async fn note_query(&self, account_id: &str, args: &Value) -> Value {
        let state = self
            .pim_type_state(account_id, ChangeType::Note)
            .await
            .unwrap_or_default();
        let notes = match self.store().list_notes(account_id).await {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        let filter = args.get("filter").cloned().unwrap_or(Value::Null);
        let want_pinned = filter.get("pinned").and_then(Value::as_bool);
        let want_tags: Vec<String> = filter
            .get("tags")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_lowercase)
                    .collect()
            })
            .unwrap_or_default();
        let text = filter
            .get("text")
            .and_then(Value::as_str)
            .map(str::to_lowercase);

        let ids: Vec<String> = notes
            .iter()
            .filter(|n| want_pinned.is_none_or(|p| n.pinned == p))
            .filter(|n| want_tags.is_empty() || note_has_any_tag(n, &want_tags))
            .filter(|n| match &text {
                None => true,
                Some(q) => {
                    n.title.to_lowercase().contains(q)
                        || n.body_text.to_lowercase().contains(q)
                        || n.body_html.to_lowercase().contains(q)
                }
            })
            .map(|n| n.id.clone())
            .collect();
        query_response(account_id, &state, ids)
    }

    // ── Note/export (P7 — VJOURNAL) ──────────────────────────────────────────

    /// Export notes as an iCalendar `VCALENDAR` of `VJOURNAL` components (RFC 5545
    /// §3.6.3), P7. `{ids?}` selects notes (default: the whole account). Each note
    /// maps to a VJOURNAL: `SUMMARY`=title, `DESCRIPTION`=plaintext body,
    /// `CATEGORIES`=tags. Returns `{blob}`.
    pub(crate) async fn note_export(&self, account_id: &str, args: &Value) -> Value {
        let ids = wanted_ids(args);
        let notes = match self.store().list_notes(account_id).await {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        let dtstamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let mut body =
            String::from("BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Mailwoman//EN\r\n");
        for note in &notes {
            if note.account_id != account_id {
                continue;
            }
            if let Some(want) = &ids
                && !want.contains(&note.id)
            {
                continue;
            }
            body.push_str(&vjournal_component(note, &dtstamp));
        }
        body.push_str("END:VCALENDAR\r\n");
        json!({ "accountId": account_id, "blob": body })
    }
}

/// Build one `VJOURNAL` component for a note (RFC 5545 §3.6.3 / P7).
fn vjournal_component(note: &NoteRow, dtstamp: &str) -> String {
    let mut out = String::from("BEGIN:VJOURNAL\r\n");
    out.push_str(&format!("UID:{}@mailwoman.local\r\n", note.id));
    out.push_str(&format!("DTSTAMP:{dtstamp}\r\n"));
    if !note.title.is_empty() {
        out.push_str(&format!("SUMMARY:{}\r\n", ics_escape(&note.title)));
    }
    if !note.body_text.is_empty() {
        out.push_str(&format!("DESCRIPTION:{}\r\n", ics_escape(&note.body_text)));
    }
    let tags: Vec<String> =
        serde_json::from_str::<Vec<String>>(&note.tags_json).unwrap_or_default();
    if !tags.is_empty() {
        let joined = tags
            .iter()
            .map(|t| ics_escape(t))
            .collect::<Vec<_>>()
            .join(",");
        out.push_str(&format!("CATEGORIES:{joined}\r\n"));
    }
    out.push_str("END:VJOURNAL\r\n");
    out
}

/// RFC 5545 TEXT escaping (backslash, semicolon, comma, newline).
fn ics_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
        .replace('\r', "")
}

// ── free helpers ─────────────────────────────────────────────────────────────

/// Serialize a spec's array field to a JSON string column (defaulting to `[]`).
fn array_str(spec: &Value, key: &str) -> String {
    spec.get(key)
        .filter(|v| v.is_array())
        .map(|v| v.to_string())
        .unwrap_or_else(|| "[]".to_string())
}

fn note_has_any_tag(n: &NoteRow, want: &[String]) -> bool {
    let tags: Vec<String> = serde_json::from_str::<Vec<String>>(&n.tags_json)
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.to_lowercase())
        .collect();
    want.iter().any(|w| tags.contains(w))
}

/// The §2.1 `Note` JSON for a decrypted row (body transits in the clear over the
/// same-origin channel; sealed only at rest, plan §1.6).
fn note_row_to_json(row: &NoteRow) -> Value {
    let tags: Value = serde_json::from_str(&row.tags_json).unwrap_or_else(|_| json!([]));
    let links: Value = serde_json::from_str(&row.links_json).unwrap_or_else(|_| json!([]));
    json!({
        "id": row.id,
        "notebookId": row.notebook_id,
        "title": row.title,
        "tags": tags,
        "color": row.color,
        "pinned": row.pinned,
        "bodyHtml": row.body_html,
        "bodyText": row.body_text,
        "links": links,
        "createdAt": row.created_at,
        "updatedAt": row.updated_at,
    })
}
