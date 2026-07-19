//! `Calendar/*` (frozen §2.2): calendar + task-list collection CRUD, plus the
//! two engine-side calendar queries `Calendar/freeBusy` (busy-block aggregation
//! via `mw-ics`) and `Calendar/detectConflicts` (overlapping materialized
//! instances, plan §2.4).

use mw_store::CalendarRow;
use serde_json::{Value, json};

use crate::account::AccountRuntime;
use crate::backend::Result;
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;

use super::{SetOutcome, gen_id, get_response, server_fail, set_error, wanted_ids};

impl Engine {
    // ── Calendar/get ─────────────────────────────────────────────────────────

    pub(crate) async fn calendar_get(&self, account_id: &str, args: &Value) -> Value {
        if let Err(e) = self.seed_default_collections(account_id).await {
            return server_fail(e);
        }
        let state = self
            .pim_type_state(account_id, ChangeType::Calendar)
            .await
            .unwrap_or_default();
        let rows = match self.store().list_calendars(account_id).await {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        let wanted = wanted_ids(args);
        let mut list = Vec::new();
        let mut found = Vec::new();
        for row in &rows {
            if let Some(ids) = &wanted
                && !ids.contains(&row.id)
            {
                continue;
            }
            found.push(row.id.clone());
            list.push(self.calendar_row_to_json(row).await);
        }
        let not_found = match &wanted {
            Some(ids) => ids
                .iter()
                .filter(|id| !found.contains(id))
                .map(|id| json!(id))
                .collect(),
            None => Vec::new(),
        };
        get_response(account_id, &state, list, not_found)
    }

    // ── Calendar/set ─────────────────────────────────────────────────────────

    pub(crate) async fn calendar_set(&self, account_id: &str, args: &Value) -> Value {
        let old_state = self
            .pim_type_state(account_id, ChangeType::Calendar)
            .await
            .unwrap_or_default();
        let mut out = SetOutcome::default();

        if let Some(creates) = args.get("create").and_then(Value::as_object) {
            for (cid, spec) in creates {
                match self.calendar_create(account_id, spec).await {
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
                match self.calendar_update(account_id, id, patch).await {
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
                match self.store().delete_calendar(id).await {
                    Ok(()) => {
                        let _ = self
                            .record_pim_change(
                                account_id,
                                ChangeType::Calendar,
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
            .pim_type_state(account_id, ChangeType::Calendar)
            .await
            .unwrap_or_default();
        self.broadcast_state(account_id).await;
        out.into_response(account_id, &old_state, &new_state)
    }

    async fn calendar_create(&self, account_id: &str, spec: &Value) -> Result<String> {
        let component = spec
            .get("component")
            .and_then(Value::as_str)
            .unwrap_or("VEVENT")
            .to_string();
        let id = gen_id(if component == "VTODO" { "list" } else { "cal" });
        let row = CalendarRow {
            id: id.clone(),
            account_id: account_id.to_string(),
            name: spec
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("Calendar")
                .to_string(),
            color: spec
                .get("color")
                .and_then(Value::as_str)
                .unwrap_or("#3366ff")
                .to_string(),
            sort_order: spec.get("order").and_then(Value::as_i64).unwrap_or(0),
            is_visible: spec
                .get("isVisible")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            role: spec.get("role").and_then(Value::as_str).map(String::from),
            caldav_url: spec
                .get("caldavUrl")
                .and_then(Value::as_str)
                .map(String::from),
            sync_token: spec
                .get("syncToken")
                .and_then(Value::as_str)
                .map(String::from),
            ctag: None,
            is_overlay: spec
                .get("isReadOnlyOverlay")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            component,
        };
        self.store().upsert_calendar(&row).await?;
        self.apply_shares(&id, spec).await?;
        self.record_pim_change(account_id, ChangeType::Calendar, &id, ChangeOp::Created)
            .await?;
        Ok(id)
    }

    async fn calendar_update(&self, account_id: &str, id: &str, patch: &Value) -> Result<()> {
        let mut row = self.store().get_calendar(id).await?.ok_or_else(|| {
            crate::backend::EngineError::Protocol(format!("unknown calendar {id}"))
        })?;
        if let Some(v) = patch.get("name").and_then(Value::as_str) {
            row.name = v.to_string();
        }
        if let Some(v) = patch.get("color").and_then(Value::as_str) {
            row.color = v.to_string();
        }
        if let Some(v) = patch.get("order").and_then(Value::as_i64) {
            row.sort_order = v;
        }
        if let Some(v) = patch.get("isVisible").and_then(Value::as_bool) {
            row.is_visible = v;
        }
        if let Some(v) = patch.get("caldavUrl") {
            row.caldav_url = v.as_str().map(String::from);
        }
        self.store().upsert_calendar(&row).await?;
        if patch.get("shareWith").is_some() {
            self.apply_shares(id, patch).await?;
        }
        self.record_pim_change(account_id, ChangeType::Calendar, id, ChangeOp::Updated)
            .await?;
        Ok(())
    }

    /// P1: replace a calendar's ACL `shareWith` grants. Each grant is
    /// `{principal, access}` with `access ∈ {"read","readWrite"}` (an unknown or
    /// absent access is normalized to `"read"` — never silently escalated). A
    /// blank principal is dropped; duplicate principals collapse to the last grant.
    async fn apply_shares(&self, calendar_id: &str, spec: &Value) -> Result<()> {
        if let Some(shares) = spec.get("shareWith").and_then(Value::as_array) {
            let mut pairs: Vec<(String, String)> = Vec::new();
            for s in shares {
                let Some(principal) = s
                    .get("principal")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|p| !p.is_empty())
                else {
                    continue;
                };
                let access = match s.get("access").and_then(Value::as_str) {
                    Some("readWrite") => "readWrite",
                    _ => "read",
                };
                // Last grant for a principal wins (dedupe).
                pairs.retain(|(p, _)| p != principal);
                pairs.push((principal.to_string(), access.to_string()));
            }
            self.store()
                .replace_calendar_shares(calendar_id, &pairs)
                .await?;
        }
        Ok(())
    }

    // ── Calendar/subscribe + Calendar/refreshSubscription (P6 — webcal) ───────

    /// Subscribe to a remote iCalendar feed (P6). `{url, name?, blob?}`: registers a
    /// read-only overlay calendar for `url` (a `webcal://` URL is normalized to
    /// `https://`) and, when a `blob` (the fetched `.ics` body) is supplied,
    /// imports it immediately. The network fetch itself is performed by the caller /
    /// a sync driver (the engine holds no general HTTP client); a subsequent
    /// `Calendar/refreshSubscription {calendarId, blob}` re-imports fresh contents.
    pub(crate) async fn calendar_subscribe(&self, account_id: &str, args: &Value) -> Value {
        let Some(url) = args.get("url").and_then(Value::as_str).map(str::trim) else {
            return server_fail("Calendar/subscribe requires a url");
        };
        if url.is_empty() {
            return server_fail("Calendar/subscribe requires a non-empty url");
        }
        let normalized = normalize_webcal(url);
        let name = args
            .get("name")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or("Subscription")
            .to_string();
        let id = gen_id("cal");
        let row = CalendarRow {
            id: id.clone(),
            account_id: account_id.to_string(),
            name,
            color: args
                .get("color")
                .and_then(Value::as_str)
                .unwrap_or("#3366ff")
                .to_string(),
            sort_order: 0,
            is_visible: true,
            role: None,
            caldav_url: Some(normalized.clone()),
            sync_token: None,
            ctag: None,
            is_overlay: true,
            component: "VEVENT".to_string(),
        };
        if let Err(e) = self.store().upsert_calendar(&row).await {
            return server_fail(e);
        }
        if let Err(e) = self
            .record_pim_change(account_id, ChangeType::Calendar, &id, ChangeOp::Created)
            .await
        {
            return server_fail(e);
        }

        // Optional immediate import of the fetched feed body.
        let mut imported = 0usize;
        if let Some(blob) = args.get("blob").and_then(Value::as_str) {
            match self.parse_calendar_document(blob) {
                Ok(events) => {
                    imported = self
                        .persist_parsed_events(account_id, &id, events, None)
                        .await
                        .len();
                }
                Err(e) => return server_fail(e),
            }
        }
        self.broadcast_state(account_id).await;
        json!({
            "accountId": account_id,
            "created": { "id": id },
            "url": normalized,
            "imported": imported,
        })
    }

    /// Re-import a subscription overlay from a freshly-fetched feed body (P6).
    /// `{calendarId, blob}`: replaces the overlay's events with the parsed contents.
    pub(crate) async fn calendar_refresh_subscription(
        &self,
        account_id: &str,
        args: &Value,
    ) -> Value {
        let Some(calendar_id) = args.get("calendarId").and_then(Value::as_str) else {
            return server_fail("Calendar/refreshSubscription requires calendarId");
        };
        let blob = args.get("blob").and_then(Value::as_str).unwrap_or_default();
        if blob.is_empty() {
            return server_fail("Calendar/refreshSubscription requires the fetched blob");
        }
        // Confirm the target is an overlay owned by this account.
        match self.store().get_calendar(calendar_id).await {
            Ok(Some(cal)) if cal.account_id == account_id && cal.is_overlay => {}
            Ok(Some(_)) => return server_fail("target calendar is not a subscription overlay"),
            Ok(None) => return server_fail(format!("unknown calendar {calendar_id}")),
            Err(e) => return server_fail(e),
        }
        // Clear existing overlay events, then re-import.
        match self.store().list_events(calendar_id).await {
            Ok(existing) => {
                for ev in existing {
                    if let Err(e) = self.store().delete_event(&ev.id).await {
                        return server_fail(e);
                    }
                    let _ = self
                        .record_pim_change(
                            account_id,
                            ChangeType::CalendarEvent,
                            &ev.id,
                            ChangeOp::Destroyed,
                        )
                        .await;
                }
            }
            Err(e) => return server_fail(e),
        }
        let events = match self.parse_calendar_document(blob) {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        let imported = self
            .persist_parsed_events(account_id, calendar_id, events, None)
            .await
            .len();
        self.broadcast_state(account_id).await;
        json!({ "accountId": account_id, "calendarId": calendar_id, "imported": imported })
    }

    // ── Calendar/freeBusy (§2.2) ─────────────────────────────────────────────

    pub(crate) async fn calendar_free_busy(
        &self,
        account_id: &str,
        _rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let start = args.get("start").and_then(Value::as_str);
        let end = args.get("end").and_then(Value::as_str);
        let (Some(start), Some(end)) = (start, end) else {
            return server_fail("Calendar/freeBusy requires start + end (RFC3339 UTC)");
        };
        // Aggregate over the account's own events in the window (§2.2). Optional
        // `calendarIds` restricts the set.
        let wanted: Option<Vec<String>> =
            args.get("calendarIds").and_then(Value::as_array).map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            });
        let events = match self.gather_event_json(account_id, wanted.as_deref()).await {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        match mw_ics::aggregate_free_busy(&events, start, end) {
            Ok(intervals) => {
                let list: Vec<Value> = intervals
                    .into_iter()
                    .map(|b| json!({ "start": b.start_utc, "end": b.end_utc, "status": b.status }))
                    .collect();
                json!({ "accountId": account_id, "list": list })
            }
            Err(e) => server_fail(e),
        }
    }

    // ── Calendar/detectConflicts (§2.2) ──────────────────────────────────────

    pub(crate) async fn calendar_detect_conflicts(&self, account_id: &str, args: &Value) -> Value {
        let start = args
            .get("start")
            .and_then(Value::as_str)
            .unwrap_or("2000-01-01T00:00:00Z");
        let end = args
            .get("end")
            .and_then(Value::as_str)
            .unwrap_or("2100-01-01T00:00:00Z");
        let insts = match self.store().events_in_range(account_id, start, end).await {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        // Sorted by instance start: pair each with later instances that overlap.
        let mut conflicts = Vec::new();
        for i in 0..insts.len() {
            for j in (i + 1)..insts.len() {
                let a = &insts[i];
                let b = &insts[j];
                if b.event_id == a.event_id {
                    continue;
                }
                // Sorted by start: once b starts at/after a ends, no later b overlaps a.
                if b.instance_start_utc >= a.instance_end_utc {
                    break;
                }
                conflicts.push(json!({
                    "eventA": a.event_id,
                    "eventB": b.event_id,
                    "overlapStart": b.instance_start_utc,
                    "overlapEnd": if a.instance_end_utc < b.instance_end_utc {
                        a.instance_end_utc.clone()
                    } else {
                        b.instance_end_utc.clone()
                    },
                }));
            }
        }
        json!({ "accountId": account_id, "list": conflicts })
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    /// Ensure a default collection of `component` exists, returning its id (the
    /// role:default one, else the first, else a freshly-created native one).
    pub(crate) async fn ensure_default_calendar(
        &self,
        account_id: &str,
        component: &str,
    ) -> Result<String> {
        let cals = self.store().list_calendars(account_id).await?;
        if let Some(c) = cals.iter().find(|c| c.component == component) {
            return Ok(c.id.clone());
        }
        let (id, name, color) = if component == "VTODO" {
            (gen_id("list"), "Tasks", "#8855ff")
        } else {
            (gen_id("cal"), "Calendar", "#3366ff")
        };
        let row = CalendarRow {
            id: id.clone(),
            account_id: account_id.to_string(),
            name: name.to_string(),
            color: color.to_string(),
            sort_order: 0,
            is_visible: true,
            role: Some("default".to_string()),
            caldav_url: None,
            sync_token: None,
            ctag: None,
            is_overlay: false,
            component: component.to_string(),
        };
        self.store().upsert_calendar(&row).await?;
        self.record_pim_change(account_id, ChangeType::Calendar, &id, ChangeOp::Created)
            .await?;
        Ok(id)
    }

    /// Seed a default event calendar + task list on first access.
    async fn seed_default_collections(&self, account_id: &str) -> Result<()> {
        let cals = self.store().list_calendars(account_id).await?;
        if !cals.iter().any(|c| c.component == "VEVENT") {
            self.ensure_default_calendar(account_id, "VEVENT").await?;
        }
        if !cals.iter().any(|c| c.component == "VTODO") {
            self.ensure_default_calendar(account_id, "VTODO").await?;
        }
        Ok(())
    }

    /// The event projections in an account, optionally restricted to some
    /// calendars — the input to free/busy aggregation.
    async fn gather_event_json(
        &self,
        account_id: &str,
        calendar_ids: Option<&[String]>,
    ) -> Result<Vec<Value>> {
        let cals = self.store().list_calendars(account_id).await?;
        let mut out = Vec::new();
        for cal in &cals {
            if cal.component != "VEVENT" {
                continue;
            }
            if let Some(ids) = calendar_ids
                && !ids.contains(&cal.id)
            {
                continue;
            }
            for ev in self.store().list_events(&cal.id).await? {
                out.push(self.event_row_to_json(&ev));
            }
        }
        Ok(out)
    }

    /// The §2.1 `Calendar` JSON for a row (with its ACL shares + `component`).
    async fn calendar_row_to_json(&self, row: &CalendarRow) -> Value {
        let shares = self
            .store()
            .list_calendar_shares(&row.id)
            .await
            .unwrap_or_default();
        let share_with: Vec<Value> = shares
            .into_iter()
            .map(|(principal, access)| json!({ "principal": principal, "access": access }))
            .collect();
        json!({
            "id": row.id,
            "name": row.name,
            "color": row.color,
            "order": row.sort_order,
            "isVisible": row.is_visible,
            "isSubscribed": true,
            "role": row.role,
            "shareWith": share_with,
            "caldavUrl": row.caldav_url,
            "syncToken": row.sync_token,
            "isReadOnlyOverlay": row.is_overlay,
            "component": row.component,
        })
    }
}

/// Normalize a subscription URL for storage: `webcal://` and `webcals://` are the
/// calendar-subscription schemes for plain HTTP feeds — rewrite them to `https://`
/// (`webcals`) / `http://` is left as-is only when explicitly given, else `webcal`
/// maps to `https`. Any other scheme is returned unchanged.
fn normalize_webcal(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("webcals://") {
        format!("https://{rest}")
    } else if let Some(rest) = url.strip_prefix("webcal://") {
        format!("https://{rest}")
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_webcal;

    #[test]
    fn webcal_normalizes_to_https() {
        assert_eq!(
            normalize_webcal("webcal://example.com/cal.ics"),
            "https://example.com/cal.ics"
        );
        assert_eq!(
            normalize_webcal("webcals://example.com/cal.ics"),
            "https://example.com/cal.ics"
        );
        assert_eq!(
            normalize_webcal("https://example.com/cal.ics"),
            "https://example.com/cal.ics"
        );
    }
}
