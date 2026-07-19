//! `CalendarEvent/*` (frozen §2.2): CRUD over VEVENTs, recurrence expansion +
//! `event_instances` materialization (plan §2.4), ICS/.hol parse/import/export,
//! and the iTIP invite flow (`respond` → iMIP REPLY/COUNTER via `MailSubmitter`,
//! plan §2.6). Local storage is the round-trip source of truth (`ical_raw`);
//! CalDAV-backed calendars also push through [`crate::pim::sync`].

use mw_store::{EventInstanceRow, EventRow};
use serde_json::{Value, json};

use crate::account::AccountRuntime;
use crate::backend::{EngineError, Result};
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;

use super::quickadd::parse_quick_add;
use super::{
    SetOutcome, gen_id, gen_token, get_response, materialize_window, query_response, server_fail,
    set_error, wanted_ids,
};

impl Engine {
    // ── CalendarEvent/get ────────────────────────────────────────────────────

    pub(crate) async fn event_get(&self, account_id: &str, args: &Value) -> Value {
        let state = self
            .pim_type_state(account_id, ChangeType::CalendarEvent)
            .await
            .unwrap_or_default();
        let ids = match wanted_ids(args) {
            Some(ids) => ids,
            None => match self.all_event_ids(account_id).await {
                Ok(v) => v,
                Err(e) => return server_fail(e),
            },
        };
        let mut list = Vec::new();
        let mut not_found = Vec::new();
        for id in &ids {
            match self.store().get_event(id).await {
                Ok(Some(row)) => list.push(self.event_row_to_json(&row)),
                Ok(None) => not_found.push(json!(id)),
                Err(e) => return server_fail(e),
            }
        }
        get_response(account_id, &state, list, not_found)
    }

    // ── CalendarEvent/set ────────────────────────────────────────────────────

    pub(crate) async fn event_set(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let old_state = self
            .pim_type_state(account_id, ChangeType::CalendarEvent)
            .await
            .unwrap_or_default();
        let mut out = SetOutcome::default();

        if let Some(creates) = args.get("create").and_then(Value::as_object) {
            for (cid, spec) in creates {
                match self.event_create(account_id, rt, spec).await {
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
                match self.event_update(account_id, rt, id, patch).await {
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
                match self.event_destroy(account_id, rt, id).await {
                    Ok(()) => out.destroyed.push(json!(id)),
                    Err(e) => {
                        out.not_destroyed
                            .insert(id.to_string(), set_error("serverFail", e));
                    }
                }
            }
        }

        let new_state = self
            .pim_type_state(account_id, ChangeType::CalendarEvent)
            .await
            .unwrap_or_default();
        self.broadcast_state(account_id).await;
        out.into_response(account_id, &old_state, &new_state)
    }

    async fn event_create(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        spec: &Value,
    ) -> Result<String> {
        let calendar_id = match spec.get("calendarId").and_then(Value::as_str) {
            Some(c) => c.to_string(),
            None => self.ensure_default_calendar(account_id, "VEVENT").await?,
        };
        let id = gen_id("ev");
        let uid = spec
            .get("uid")
            .and_then(Value::as_str)
            .map(String::from)
            .unwrap_or_else(|| format!("{}@mailwoman.local", gen_token()));
        let mut json = spec.clone();
        normalize_event(&mut json);
        let row = self
            .persist_event(
                account_id,
                &calendar_id,
                &id,
                &uid,
                json.clone(),
                None,
                Some(rt),
            )
            .await?;
        self.record_pim_change(
            account_id,
            ChangeType::CalendarEvent,
            &row.id,
            ChangeOp::Created,
        )
        .await?;
        // iTIP REQUEST to attendees that expect a reply (organizer send, §2.6).
        self.maybe_send_request(rt, &json).await;
        Ok(id)
    }

    async fn event_update(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        id: &str,
        patch: &Value,
    ) -> Result<()> {
        let row = self
            .store()
            .get_event(id)
            .await?
            .ok_or_else(|| EngineError::Protocol(format!("unknown event {id}")))?;
        let mut json = self.event_row_to_json(&row);
        merge_patch(&mut json, patch);
        normalize_event(&mut json);
        self.persist_event(
            account_id,
            &row.calendar_id,
            id,
            &row.uid,
            json,
            row.etag.clone(),
            Some(rt),
        )
        .await?;
        self.record_pim_change(account_id, ChangeType::CalendarEvent, id, ChangeOp::Updated)
            .await?;
        Ok(())
    }

    async fn event_destroy(&self, account_id: &str, rt: &AccountRuntime, id: &str) -> Result<()> {
        let row = self
            .store()
            .get_event(id)
            .await?
            .ok_or_else(|| EngineError::Protocol(format!("unknown event {id}")))?;
        // Best-effort remote delete for CalDAV-backed calendars.
        if let Some(cal) = self.store().get_calendar(&row.calendar_id).await?
            && let (Some(base), Some(url)) = (rt.dav.clone(), cal.caldav_url.clone())
        {
            let href = resource_href(&url, &row.uid, "ics");
            let _ = self.dav_delete(&base, &href, row.etag.as_deref()).await;
        }
        self.store().delete_event(id).await?;
        self.record_pim_change(
            account_id,
            ChangeType::CalendarEvent,
            id,
            ChangeOp::Destroyed,
        )
        .await?;
        Ok(())
    }

    /// Emit + materialize + persist an event from its projection, pushing to
    /// CalDAV when the calendar is DAV-backed. Shared by create/update/import
    /// and the sync pull path. Returns the stored row.
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn persist_event(
        &self,
        _account_id: &str,
        calendar_id: &str,
        id: &str,
        uid: &str,
        mut json: Value,
        prior_etag: Option<String>,
        push_rt: Option<&AccountRuntime>,
    ) -> Result<EventRow> {
        // Stamp identity onto the projection so get/query are self-consistent.
        set_str(&mut json, "id", id);
        set_str(&mut json, "calendarId", calendar_id);
        set_str(&mut json, "uid", uid);

        // P4/P5: `categories` + `attachments` are engine-carried projection fields —
        // `mw-ics` does not round-trip CATEGORIES/ATTACH, so capture them before the
        // ICS canonicalization and re-apply below (get/query then see them).
        let carried: Vec<(&str, Value)> = ["categories", "attachments"]
            .iter()
            .filter_map(|k| {
                json.get(*k)
                    .filter(|v| !v.is_null())
                    .map(|v| (*k, v.clone()))
            })
            .collect();

        let ical_raw = mw_ics::emit_ical(&json).map_err(ics_err)?;
        // Re-parse so the stored json is the canonical round-trip projection.
        let mut canonical = mw_ics::parse_ical(ical_raw.as_bytes())
            .map_err(ics_err)?
            .into_iter()
            .next()
            .map(|p| p.json)
            .unwrap_or(json);
        set_str(&mut canonical, "id", id);
        set_str(&mut canonical, "calendarId", calendar_id);
        set_str(&mut canonical, "uid", uid);
        for (key, val) in carried {
            set_json(&mut canonical, key, val);
        }

        let tzid = canonical
            .get("timeZone")
            .and_then(Value::as_str)
            .map(String::from);
        let rrule = canonical
            .get("recurrenceRules")
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(|r| r.get("rrule"))
            .and_then(Value::as_str)
            .map(String::from);
        let status = canonical
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("confirmed")
            .to_string();

        // Materialize the recurrence into the range index.
        let (win_start, win_end) = materialize_window();
        let instances = mw_ics::expand_recurrence(&canonical, &win_start, &win_end)
            .map_err(ics_err)
            .unwrap_or_default();
        let (start_utc, end_utc) = instances
            .first()
            .map(|i| (Some(i.start_utc.clone()), Some(i.end_utc.clone())))
            .unwrap_or((None, None));

        // Push to CalDAV where backed; adopt the server etag.
        let mut etag = prior_etag.clone();
        if let Some(rt) = push_rt
            && let Some(cal) = self.store().get_calendar(calendar_id).await?
            && let (Some(base), Some(url)) = (rt.dav.clone(), cal.caldav_url.clone())
        {
            let href = resource_href(&url, uid, "ics");
            match self
                .dav_put(&base, &href, &ical_raw, prior_etag.as_deref())
                .await
            {
                Ok(new_etag) => etag = new_etag,
                Err(e) => tracing::warn!("CalDAV put for event {id} failed: {e}"),
            }
        }
        set_json(
            &mut canonical,
            "etag",
            etag.clone().map(Value::String).unwrap_or(Value::Null),
        );

        let json_bytes = serde_json::to_vec(&canonical).ok();
        let row = EventRow {
            id: id.to_string(),
            calendar_id: calendar_id.to_string(),
            uid: uid.to_string(),
            etag,
            ical_raw,
            start_utc,
            end_utc,
            tzid,
            rrule,
            status,
            json: json_bytes,
        };
        self.store().upsert_event(&row).await?;
        let inst_rows: Vec<EventInstanceRow> = instances
            .into_iter()
            .map(|i| EventInstanceRow {
                event_id: id.to_string(),
                instance_start_utc: i.start_utc,
                instance_end_utc: i.end_utc,
            })
            .collect();
        self.store().replace_event_instances(id, &inst_rows).await?;
        Ok(row)
    }

    // ── CalendarEvent/query ──────────────────────────────────────────────────

    pub(crate) async fn event_query(&self, account_id: &str, args: &Value) -> Value {
        let state = self
            .pim_type_state(account_id, ChangeType::CalendarEvent)
            .await
            .unwrap_or_default();
        match self.event_query_ids(account_id, args).await {
            Ok(ids) => query_response(account_id, &state, ids),
            Err(e) => server_fail(e),
        }
    }

    async fn event_query_ids(&self, account_id: &str, args: &Value) -> Result<Vec<String>> {
        let filter = args.get("filter").cloned().unwrap_or(Value::Null);
        let calendar_id = filter
            .get("calendarId")
            .or_else(|| filter.get("inCalendar"))
            .and_then(Value::as_str);
        let after = filter.get("after").and_then(Value::as_str);
        let before = filter.get("before").and_then(Value::as_str);
        // P4: optional category filter (`category: "x"` or `categories: [..]`).
        let wanted_categories: Vec<String> = filter
            .get("categories")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .or_else(|| {
                filter
                    .get("category")
                    .and_then(Value::as_str)
                    .map(|s| vec![s.to_string()])
            })
            .unwrap_or_default();

        let ids = if let (Some(after), Some(before)) = (after, before) {
            // Window path: distinct events with a materialized instance in range.
            let insts = self
                .store()
                .events_in_range(account_id, after, before)
                .await?;
            let mut ids: Vec<String> = Vec::new();
            for i in &insts {
                if !ids.contains(&i.event_id) {
                    ids.push(i.event_id.clone());
                }
            }
            if let Some(cid) = calendar_id {
                let mut kept = Vec::new();
                for id in ids {
                    if let Some(ev) = self.store().get_event(&id).await?
                        && ev.calendar_id == cid
                    {
                        kept.push(id);
                    }
                }
                ids = kept;
            }
            ids
        } else {
            // Full path: all events (optionally one calendar), start-ordered.
            let cals = self.store().list_calendars(account_id).await?;
            let mut ids = Vec::new();
            for cal in &cals {
                if cal.component != "VEVENT" {
                    continue;
                }
                if let Some(cid) = calendar_id
                    && cal.id != cid
                {
                    continue;
                }
                for ev in self.store().list_events(&cal.id).await? {
                    ids.push(ev.id);
                }
            }
            ids
        };

        self.filter_events_by_categories(ids, &wanted_categories)
            .await
    }

    pub(crate) async fn event_query_changes(&self, account_id: &str, args: &Value) -> Value {
        let since = args
            .get("sinceQueryState")
            .and_then(Value::as_str)
            .unwrap_or("0");
        let new_state = self
            .pim_type_state(account_id, ChangeType::CalendarEvent)
            .await
            .unwrap_or_default();
        let ids = self
            .event_query_ids(account_id, args)
            .await
            .unwrap_or_default();
        let removed = self
            .build_pim_changes(account_id, ChangeType::CalendarEvent, since)
            .await
            .map(|c| c.destroyed)
            .unwrap_or_default();
        let added: Vec<Value> = ids
            .iter()
            .enumerate()
            .map(|(i, id)| json!({ "id": id, "index": i }))
            .collect();
        json!({
            "accountId": account_id,
            "oldQueryState": since,
            "newQueryState": new_state,
            "total": ids.len(),
            "removed": removed,
            "added": added,
        })
    }

    // ── CalendarEvent/expand ─────────────────────────────────────────────────

    /// Expand masters into instances over an explicit window (§2.2). Input:
    /// `{ids?|calendarId?, start, end}`; output `list` of instances carrying the
    /// owning `eventId` + `start`/`end` (RFC3339 UTC) + the master projection.
    pub(crate) async fn event_expand(&self, account_id: &str, args: &Value) -> Value {
        let start = args.get("start").and_then(Value::as_str);
        let end = args.get("end").and_then(Value::as_str);
        let (Some(start), Some(end)) = (start, end) else {
            return server_fail("CalendarEvent/expand requires start + end (RFC3339 UTC)");
        };
        let ids = match wanted_ids(args) {
            Some(ids) => ids,
            None => match self.all_event_ids(account_id).await {
                Ok(v) => v,
                Err(e) => return server_fail(e),
            },
        };
        let mut list = Vec::new();
        for id in &ids {
            let Ok(Some(row)) = self.store().get_event(id).await else {
                continue;
            };
            let master = self.event_row_to_json(&row);
            let instances = mw_ics::expand_recurrence(&master, start, end).unwrap_or_default();
            for inst in instances {
                let mut obj = master.clone();
                set_str(&mut obj, "eventId", id);
                set_str(&mut obj, "start", &inst.start_utc);
                set_str(&mut obj, "timeZone", "UTC");
                set_json(&mut obj, "instanceStart", json!(inst.start_utc));
                set_json(&mut obj, "instanceEnd", json!(inst.end_utc));
                list.push(obj);
            }
        }
        json!({ "accountId": account_id, "list": list })
    }

    // ── CalendarEvent/parse + import + export ────────────────────────────────

    /// Parse an ICS / iTIP / `.hol` blob into event projections (no persist).
    pub(crate) async fn event_parse(&self, account_id: &str, args: &Value) -> Value {
        let blob = args.get("blob").and_then(Value::as_str).unwrap_or_default();
        match parse_calendar_blob(blob) {
            Ok(events) => json!({ "accountId": account_id, "parsed": events }),
            Err(e) => server_fail(e),
        }
    }

    /// Import an ICS / `.hol` blob into a calendar, persisting each event (§2.2).
    pub(crate) async fn event_import(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let blob = args.get("blob").and_then(Value::as_str).unwrap_or_default();
        let events = match parse_calendar_blob(blob) {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        let calendar_id = match args.get("calendarId").and_then(Value::as_str) {
            Some(c) => c.to_string(),
            None => match self.ensure_default_calendar(account_id, "VEVENT").await {
                Ok(c) => c,
                Err(e) => return server_fail(e),
            },
        };
        let created = self
            .persist_parsed_events(account_id, &calendar_id, events, Some(rt))
            .await;
        self.broadcast_state(account_id).await;
        json!({ "accountId": account_id, "imported": created.clone(), "count": created.len() })
    }

    /// Persist a batch of parsed event projections into `calendar_id`, recording a
    /// `Created` change per event. `push_rt` drives CalDAV push (None = local only,
    /// e.g. a webcal subscription overlay). Returns the created ids. Skips (logs)
    /// any single event that fails to persist. Shared by `CalendarEvent/import`
    /// and the webcal subscription path (P6).
    pub(crate) async fn persist_parsed_events(
        &self,
        account_id: &str,
        calendar_id: &str,
        events: Vec<Value>,
        push_rt: Option<&AccountRuntime>,
    ) -> Vec<String> {
        let mut created = Vec::new();
        for ev in events {
            let uid = ev
                .get("uid")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .unwrap_or_else(|| format!("{}@mailwoman.local", gen_token()));
            let id = gen_id("ev");
            match self
                .persist_event(account_id, calendar_id, &id, &uid, ev, None, push_rt)
                .await
            {
                Ok(_) => {
                    let _ = self
                        .record_pim_change(
                            account_id,
                            ChangeType::CalendarEvent,
                            &id,
                            ChangeOp::Created,
                        )
                        .await;
                    created.push(id);
                }
                Err(e) => tracing::warn!("event import skipped one: {e}"),
            }
        }
        created
    }

    /// Parse an ICS/iTIP/`.hol` blob into event projections (P6 subscription
    /// refresh helper; exposes the module-private feature-detecting parser).
    pub(crate) fn parse_calendar_document(&self, blob: &str) -> Result<Vec<Value>> {
        parse_calendar_blob(blob)
    }

    /// Export events (by id) to a single ICS document (§2.2).
    pub(crate) async fn event_export(&self, account_id: &str, args: &Value) -> Value {
        let ids = match wanted_ids(args) {
            Some(ids) => ids,
            None => self.all_event_ids(account_id).await.unwrap_or_default(),
        };
        let mut body =
            String::from("BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Mailwoman//EN\r\n");
        for id in &ids {
            if let Ok(Some(row)) = self.store().get_event(id).await {
                // Splice the VEVENT out of each event's single-component calendar.
                if let Some(inner) = extract_components(&row.ical_raw) {
                    body.push_str(&inner);
                }
            }
        }
        body.push_str("END:VCALENDAR\r\n");
        json!({ "accountId": account_id, "blob": body })
    }

    // ── CalendarEvent/respond (iTIP REPLY / COUNTER) ─────────────────────────

    pub(crate) async fn event_respond(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let event_id = args
            .get("eventId")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let action = args
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("accept");
        let row = match self.store().get_event(event_id).await {
            Ok(Some(r)) => r,
            Ok(None) => return server_fail(format!("unknown event {event_id}")),
            Err(e) => return server_fail(e),
        };
        let mut json = self.event_row_to_json(&row);

        let partstat = match action {
            "accept" => "accepted",
            "decline" => "declined",
            "tentative" => "tentative",
            "counter" => "tentative",
            other => return server_fail(format!("unknown respond action {other}")),
        };
        // Update our own participation status (participants keyed by email).
        let me = rt.identity.clone();
        if let Some(parts) = json.get_mut("participants").and_then(Value::as_object_mut)
            && let Some(p) = parts.get_mut(&me).and_then(Value::as_object_mut)
        {
            p.insert("participationStatus".into(), json!(partstat));
        }
        // A counter proposes a new start/duration.
        if action == "counter"
            && let Some(counter) = args.get("counter")
        {
            if let Some(start) = counter.get("start").and_then(Value::as_str) {
                set_str(&mut json, "start", start);
            }
            if let Some(dur) = counter.get("duration").and_then(Value::as_str) {
                set_str(&mut json, "duration", dur);
            }
        }
        // Bump sequence.
        let seq = json.get("sequence").and_then(Value::as_i64).unwrap_or(0) + 1;
        set_json(&mut json, "sequence", json!(seq));

        // Persist the local change.
        if let Err(e) = self
            .persist_event(
                account_id,
                &row.calendar_id,
                event_id,
                &row.uid,
                json.clone(),
                row.etag.clone(),
                Some(rt),
            )
            .await
        {
            return server_fail(e);
        }
        let _ = self
            .record_pim_change(
                account_id,
                ChangeType::CalendarEvent,
                event_id,
                ChangeOp::Updated,
            )
            .await;

        // Send the iMIP REPLY / COUNTER to the organizer.
        let method = if action == "counter" {
            mw_ics::ItipMethod::Counter
        } else {
            mw_ics::ItipMethod::Reply
        };
        if let Some(org) = organizer_email(&json) {
            self.send_itip(rt, &me, &[org], &json, method).await;
        }
        self.broadcast_state(account_id).await;
        json!({ "accountId": account_id, "updated": self.reload_event_json(event_id).await })
    }

    // ── CalendarEvent/quickAdd (P3 — natural-language create) ────────────────

    /// Parse a natural-language phrase (`{text, calendarId?}`) into an event and
    /// create it (P3). Returns `{created:{id}, parsed:{title,start,duration,allDay,
    /// location}}` so the UI can echo what was understood. A phrase with no
    /// recognizable day/time becomes an all-day event today.
    pub(crate) async fn event_quick_add(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let text = args.get("text").and_then(Value::as_str).unwrap_or_default();
        if text.trim().is_empty() {
            return server_fail("CalendarEvent/quickAdd requires a non-empty text");
        }
        let parsed = parse_quick_add(text, chrono::Utc::now());
        let title = if parsed.title.trim().is_empty() {
            text.trim().to_string()
        } else {
            parsed.title.clone()
        };

        let mut spec = json!({ "title": title });
        if let Some(cid) = args.get("calendarId").and_then(Value::as_str) {
            spec["calendarId"] = json!(cid);
        }
        match &parsed.start {
            Some(start) => {
                spec["start"] = json!(start);
                spec["showWithoutTime"] = json!(parsed.all_day);
                spec["duration"] = json!(parsed.duration);
            }
            None => {
                // No day/time recognized → an all-day event today.
                spec["start"] = json!(
                    chrono::Utc::now()
                        .date_naive()
                        .format("%Y-%m-%d")
                        .to_string()
                );
                spec["showWithoutTime"] = json!(true);
                spec["duration"] = json!("P1D");
            }
        }
        if let Some(loc) = &parsed.location {
            spec["locations"] = json!([{ "name": loc }]);
        }

        match self.event_create(account_id, rt, &spec).await {
            Ok(id) => {
                self.broadcast_state(account_id).await;
                json!({
                    "accountId": account_id,
                    "created": { "id": id },
                    "parsed": {
                        "title": title,
                        "start": parsed.start,
                        "duration": parsed.duration,
                        "allDay": parsed.all_day,
                        "location": parsed.location,
                    },
                })
            }
            Err(e) => server_fail(e),
        }
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    /// Restrict `ids` to events whose stored `categories` intersect `wanted`
    /// (case-insensitive). Used by `CalendarEvent/query` category filtering (P4).
    async fn filter_events_by_categories(
        &self,
        ids: Vec<String>,
        wanted: &[String],
    ) -> Result<Vec<String>> {
        if wanted.is_empty() {
            return Ok(ids);
        }
        let want: Vec<String> = wanted.iter().map(|s| s.to_lowercase()).collect();
        let mut kept = Vec::new();
        for id in ids {
            if let Some(row) = self.store().get_event(&id).await? {
                let cats: Vec<String> = self
                    .event_row_to_json(&row)
                    .get("categories")
                    .and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .filter_map(Value::as_str)
                            .map(str::to_lowercase)
                            .collect()
                    })
                    .unwrap_or_default();
                if cats.iter().any(|c| want.contains(c)) {
                    kept.push(id);
                }
            }
        }
        Ok(kept)
    }

    /// The stored projection for an event row, patched with its row identity.
    pub(crate) fn event_row_to_json(&self, row: &EventRow) -> Value {
        let mut json = row
            .json
            .as_ref()
            .and_then(|b| serde_json::from_slice::<Value>(b).ok())
            .or_else(|| {
                mw_ics::parse_ical(row.ical_raw.as_bytes())
                    .ok()
                    .and_then(|v| v.into_iter().next())
                    .map(|p| p.json)
            })
            .unwrap_or_else(|| json!({}));
        set_str(&mut json, "id", &row.id);
        set_str(&mut json, "calendarId", &row.calendar_id);
        set_str(&mut json, "uid", &row.uid);
        set_json(
            &mut json,
            "etag",
            row.etag.clone().map(Value::String).unwrap_or(Value::Null),
        );
        json
    }

    async fn reload_event_json(&self, id: &str) -> Value {
        match self.store().get_event(id).await {
            Ok(Some(row)) => self.event_row_to_json(&row),
            _ => Value::Null,
        }
    }

    /// Every event id across the account's calendars (start-ordered per calendar).
    pub(crate) async fn all_event_ids(&self, account_id: &str) -> Result<Vec<String>> {
        let cals = self.store().list_calendars(account_id).await?;
        let mut ids = Vec::new();
        for cal in &cals {
            if cal.component != "VEVENT" {
                continue;
            }
            for ev in self.store().list_events(&cal.id).await? {
                ids.push(ev.id);
            }
        }
        Ok(ids)
    }

    /// Send an iTIP REQUEST for a create/update when attendees expect a reply.
    async fn maybe_send_request(&self, rt: &AccountRuntime, json: &Value) {
        let recipients: Vec<String> = json
            .get("participants")
            .and_then(Value::as_object)
            .map(|m| {
                m.iter()
                    .filter(|(_, p)| p.get("expectReply").and_then(Value::as_bool) == Some(true))
                    .map(|(email, _)| email.clone())
                    .filter(|email| email.as_str() != rt.identity)
                    .collect()
            })
            .unwrap_or_default();
        if recipients.is_empty() {
            return;
        }
        let from = rt.identity.clone();
        self.send_itip(rt, &from, &recipients, json, mw_ics::ItipMethod::Request)
            .await;
    }

    /// Frame an iTIP method as a `text/calendar` message and submit it (§2.6).
    /// Best-effort — a submit failure is logged, never fatal to the mutation.
    async fn send_itip(
        &self,
        rt: &AccountRuntime,
        from: &str,
        to: &[String],
        event_json: &Value,
        method: mw_ics::ItipMethod,
    ) {
        let ics = match mw_ics::build_itip(event_json, method) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("iTIP build failed: {e}");
                return;
            }
        };
        let method_name = match method {
            mw_ics::ItipMethod::Request => "REQUEST",
            mw_ics::ItipMethod::Reply => "REPLY",
            mw_ics::ItipMethod::Counter => "COUNTER",
            mw_ics::ItipMethod::Cancel => "CANCEL",
        };
        let title = event_json
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Event");
        let raw = build_itip_mime(from, to, method_name, title, &ics);
        if let Err(e) = rt
            .submitter
            .submit(mw_smtp::Outgoing {
                mail_from: from.to_string(),
                rcpt_to: to.to_vec(),
                raw,
            })
            .await
        {
            tracing::warn!("iTIP submit failed: {e}");
        }
    }
}

// ── free helpers ─────────────────────────────────────────────────────────────

/// Fill sane defaults on an event projection so `emit_ical` always succeeds.
fn normalize_event(json: &mut Value) {
    let obj = match json.as_object_mut() {
        Some(o) => o,
        None => return,
    };
    obj.entry("status").or_insert_with(|| json!("confirmed"));
    obj.entry("freeBusyStatus").or_insert_with(|| json!("busy"));
    obj.entry("duration").or_insert_with(|| json!("PT1H"));
    obj.entry("showWithoutTime").or_insert_with(|| json!(false));
    obj.entry("title").or_insert_with(|| json!(""));
    obj.entry("sequence").or_insert_with(|| json!(0));
    obj.entry("priority").or_insert_with(|| json!(0));
}

/// Shallow-merge a JMAP patch object's top-level keys into a projection.
fn merge_patch(target: &mut Value, patch: &Value) {
    if let (Some(t), Some(p)) = (target.as_object_mut(), patch.as_object()) {
        for (k, v) in p {
            t.insert(k.clone(), v.clone());
        }
    }
}

fn set_str(json: &mut Value, key: &str, val: &str) {
    if let Some(obj) = json.as_object_mut() {
        obj.insert(key.to_string(), Value::String(val.to_string()));
    }
}

fn set_json(json: &mut Value, key: &str, val: Value) {
    if let Some(obj) = json.as_object_mut() {
        obj.insert(key.to_string(), val);
    }
}

/// The organizer's email from a projection (the participant with `role:organizer`).
fn organizer_email(json: &Value) -> Option<String> {
    let parts = json.get("participants").and_then(Value::as_object)?;
    parts
        .iter()
        .find(|(_, p)| p.get("role").and_then(Value::as_str) == Some("organizer"))
        .map(|(email, _)| email.clone())
        .or_else(|| parts.keys().next().cloned())
}

/// Parse an ICS / iTIP / `.hol` blob into event projections, feature-detecting
/// the format (plan §2.6 iMIP + §11 .hol import).
fn parse_calendar_blob(blob: &str) -> Result<Vec<Value>> {
    let bytes = blob.as_bytes();
    if blob.contains("METHOD:") && blob.contains("BEGIN:VCALENDAR") {
        // iTIP: unwrap the METHOD-framed component.
        if let Ok((_m, parsed)) = mw_ics::parse_itip(bytes) {
            return Ok(vec![parsed.json]);
        }
    }
    if blob.contains("BEGIN:VCALENDAR") {
        return mw_ics::parse_ical(bytes)
            .map(|v| v.into_iter().map(|p| p.json).collect())
            .map_err(ics_err);
    }
    // Fall back to an Outlook `.hol` holiday pack.
    mw_ics::parse_hol(bytes)
        .map(|v| v.into_iter().map(|p| p.json).collect())
        .map_err(ics_err)
}

/// Extract the inner components (VEVENT…END:VEVENT) of a single-component
/// VCALENDAR string for splicing into an export bundle.
fn extract_components(ical: &str) -> Option<String> {
    let start = ical.find("BEGIN:VEVENT")?;
    let end = ical.find("END:VEVENT")? + "END:VEVENT".len();
    let mut out = ical[start..end].to_string();
    if !out.ends_with('\n') {
        out.push_str("\r\n");
    }
    Some(out)
}

/// Frame an iTIP payload as an RFC5322 message with a `text/calendar` body.
fn build_itip_mime(from: &str, to: &[String], method: &str, title: &str, ics: &str) -> Vec<u8> {
    let to_hdr = to.join(", ");
    let msg_id = format!("<{}@mailwoman.local>", gen_token());
    format!(
        "Message-ID: {msg_id}\r\n\
         From: {from}\r\n\
         To: {to_hdr}\r\n\
         Subject: {method}: {title}\r\n\
         MIME-Version: 1.0\r\n\
         Content-Type: text/calendar; method={method}; charset=utf-8\r\n\
         Content-Transfer-Encoding: 8bit\r\n\
         \r\n\
         {ics}"
    )
    .into_bytes()
}

/// The CalDAV resource href for a collection + uid (`<collection>/<uid>.<ext>`).
pub(crate) fn resource_href(collection: &str, uid: &str, ext: &str) -> String {
    format!("{}/{}.{}", collection.trim_end_matches('/'), uid, ext)
}

/// Adapt an `mw_ics` error onto the engine error type.
pub(crate) fn ics_err(e: mw_ics::IcsError) -> EngineError {
    EngineError::Protocol(format!("ics: {e}"))
}
