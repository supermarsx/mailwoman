//! `Task/*` (frozen §2.2): VTODO-backed tasks routed through the task-list
//! collections (`component:"VTODO"`), with the engine-side **My Day / Today**
//! query and the `fromEmail` / `fromEvent` create conveniences (plan §1.4).

use mw_store::TaskRow;
use serde_json::{Value, json};

use crate::account::AccountRuntime;
use crate::backend::{EngineError, Result};
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;

use super::events::{ics_err, resource_href};
use super::{
    SetOutcome, gen_id, gen_token, get_response, query_response, server_fail, set_error, wanted_ids,
};

impl Engine {
    // ── Task/get ─────────────────────────────────────────────────────────────

    pub(crate) async fn task_get(&self, account_id: &str, args: &Value) -> Value {
        let state = self
            .pim_type_state(account_id, ChangeType::Task)
            .await
            .unwrap_or_default();
        let ids = match wanted_ids(args) {
            Some(ids) => ids,
            None => match self.all_task_ids(account_id).await {
                Ok(v) => v,
                Err(e) => return server_fail(e),
            },
        };
        let mut list = Vec::new();
        let mut not_found = Vec::new();
        for id in &ids {
            match self.store().get_task(id).await {
                Ok(Some(row)) => list.push(task_row_to_json(&row)),
                Ok(None) => not_found.push(json!(id)),
                Err(e) => return server_fail(e),
            }
        }
        get_response(account_id, &state, list, not_found)
    }

    // ── Task/set ─────────────────────────────────────────────────────────────

    pub(crate) async fn task_set(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let old_state = self
            .pim_type_state(account_id, ChangeType::Task)
            .await
            .unwrap_or_default();
        let mut out = SetOutcome::default();

        if let Some(creates) = args.get("create").and_then(Value::as_object) {
            for (cid, spec) in creates {
                match self.task_create(account_id, rt, spec).await {
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
                match self.task_update(account_id, rt, id, patch).await {
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
                match self.task_destroy(account_id, rt, id).await {
                    Ok(()) => out.destroyed.push(json!(id)),
                    Err(e) => {
                        out.not_destroyed
                            .insert(id.to_string(), set_error("serverFail", e));
                    }
                }
            }
        }

        let new_state = self
            .pim_type_state(account_id, ChangeType::Task)
            .await
            .unwrap_or_default();
        self.broadcast_state(account_id).await;
        out.into_response(account_id, &old_state, &new_state)
    }

    async fn task_create(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        spec: &Value,
    ) -> Result<String> {
        let list_id = match spec.get("listId").and_then(Value::as_str) {
            Some(l) => l.to_string(),
            None => self.ensure_default_calendar(account_id, "VTODO").await?,
        };
        let id = gen_id("task");
        let uid = spec
            .get("uid")
            .and_then(Value::as_str)
            .map(String::from)
            .unwrap_or_else(|| format!("{}@mailwoman.local", gen_token()));

        let mut json = spec.clone();
        // Mail→task / event→task conveniences (§2.2): seed the title/description.
        self.apply_from_sources(account_id, &mut json, spec).await;
        normalize_task(&mut json);
        self.persist_task(account_id, &list_id, &id, &uid, json, None, Some(rt))
            .await?;
        self.record_pim_change(account_id, ChangeType::Task, &id, ChangeOp::Created)
            .await?;
        Ok(id)
    }

    async fn task_update(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        id: &str,
        patch: &Value,
    ) -> Result<()> {
        let row = self
            .store()
            .get_task(id)
            .await?
            .ok_or_else(|| EngineError::Protocol(format!("unknown task {id}")))?;
        let mut json = task_row_to_json(&row);
        if let (Some(t), Some(p)) = (json.as_object_mut(), patch.as_object()) {
            for (k, v) in p {
                t.insert(k.clone(), v.clone());
            }
        }
        normalize_task(&mut json);
        self.persist_task(
            account_id,
            &row.list_id,
            id,
            &row.uid,
            json,
            row.etag.clone(),
            Some(rt),
        )
        .await?;
        self.record_pim_change(account_id, ChangeType::Task, id, ChangeOp::Updated)
            .await?;
        Ok(())
    }

    async fn task_destroy(&self, account_id: &str, rt: &AccountRuntime, id: &str) -> Result<()> {
        let row = self
            .store()
            .get_task(id)
            .await?
            .ok_or_else(|| EngineError::Protocol(format!("unknown task {id}")))?;
        if let Some(list) = self.store().get_calendar(&row.list_id).await?
            && let (Some(base), Some(url)) = (rt.dav.clone(), list.caldav_url.clone())
        {
            let href = resource_href(&url, &row.uid, "ics");
            let _ = self.dav_delete(&base, &href, row.etag.as_deref()).await;
        }
        self.store().delete_task(id).await?;
        self.record_pim_change(account_id, ChangeType::Task, id, ChangeOp::Destroyed)
            .await?;
        Ok(())
    }

    /// Emit + persist a task from its projection, pushing to CalDAV when the
    /// list is DAV-backed. Shared by create/update and the sync pull path.
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn persist_task(
        &self,
        _account_id: &str,
        list_id: &str,
        id: &str,
        uid: &str,
        mut json: Value,
        prior_etag: Option<String>,
        push_rt: Option<&AccountRuntime>,
    ) -> Result<()> {
        set_str(&mut json, "id", id);
        set_str(&mut json, "listId", list_id);
        set_str(&mut json, "uid", uid);

        let ical_raw = mw_ics::emit_ical(&json).map_err(ics_err)?;
        // Re-parse for the canonical VTODO projection.
        let mut canonical = mw_ics::parse_ical(ical_raw.as_bytes())
            .map_err(ics_err)?
            .into_iter()
            .next()
            .map(|p| p.json)
            .unwrap_or(json.clone());
        set_str(&mut canonical, "id", id);
        set_str(&mut canonical, "listId", list_id);
        set_str(&mut canonical, "uid", uid);
        // parse_ical drops the client-only My Day pin; carry it forward.
        if let Some(my_day) = json.get("myDayDate").cloned() {
            set_json(&mut canonical, "myDayDate", my_day);
        }

        let mut etag = prior_etag.clone();
        if let Some(rt) = push_rt
            && let Some(list) = self.store().get_calendar(list_id).await?
            && let (Some(base), Some(url)) = (rt.dav.clone(), list.caldav_url.clone())
        {
            let href = resource_href(&url, uid, "ics");
            match self
                .dav_put(&base, &href, &ical_raw, prior_etag.as_deref())
                .await
            {
                Ok(new_etag) => etag = new_etag,
                Err(e) => tracing::warn!("CalDAV put for task {id} failed: {e}"),
            }
        }
        set_json(
            &mut canonical,
            "etag",
            etag.clone().map(Value::String).unwrap_or(Value::Null),
        );

        let row = TaskRow {
            id: id.to_string(),
            list_id: list_id.to_string(),
            uid: uid.to_string(),
            etag,
            due_utc: canonical
                .get("due")
                .and_then(Value::as_str)
                .map(String::from),
            start_utc: canonical
                .get("start")
                .and_then(Value::as_str)
                .map(String::from),
            priority: canonical
                .get("priority")
                .and_then(Value::as_i64)
                .unwrap_or(0),
            percent_complete: canonical
                .get("percentComplete")
                .and_then(Value::as_i64)
                .unwrap_or(0),
            status: canonical
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("needs-action")
                .to_string(),
            parent_id: canonical
                .get("parentId")
                .and_then(Value::as_str)
                .map(String::from),
            my_day_date: canonical
                .get("myDayDate")
                .and_then(Value::as_str)
                .map(String::from),
            ical_raw,
            json: serde_json::to_vec(&canonical).ok(),
        };
        self.store().upsert_task(&row).await?;
        Ok(())
    }

    // ── Task/query (incl. My Day) ────────────────────────────────────────────

    pub(crate) async fn task_query(&self, account_id: &str, args: &Value) -> Value {
        let state = self
            .pim_type_state(account_id, ChangeType::Task)
            .await
            .unwrap_or_default();
        match self.task_query_ids(account_id, args).await {
            Ok(ids) => query_response(account_id, &state, ids),
            Err(e) => server_fail(e),
        }
    }

    async fn task_query_ids(&self, account_id: &str, args: &Value) -> Result<Vec<String>> {
        let filter = args.get("filter").cloned().unwrap_or(Value::Null);
        let list_id = filter.get("listId").and_then(Value::as_str);
        let my_day = filter
            .get("myDay")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let want_status = filter.get("status").and_then(Value::as_str);
        let today = chrono::Utc::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string();

        let cals = self.store().list_calendars(account_id).await?;
        let mut ids = Vec::new();
        for cal in &cals {
            if cal.component != "VTODO" {
                continue;
            }
            if let Some(l) = list_id
                && cal.id != l
            {
                continue;
            }
            for t in self.store().list_tasks(&cal.id).await? {
                if let Some(s) = want_status
                    && t.status != s
                {
                    continue;
                }
                if my_day && !is_my_day(&t, &today) {
                    continue;
                }
                ids.push(t.id);
            }
        }
        Ok(ids)
    }

    pub(crate) async fn task_query_changes(&self, account_id: &str, args: &Value) -> Value {
        let since = args
            .get("sinceQueryState")
            .and_then(Value::as_str)
            .unwrap_or("0");
        let new_state = self
            .pim_type_state(account_id, ChangeType::Task)
            .await
            .unwrap_or_default();
        let ids = self
            .task_query_ids(account_id, args)
            .await
            .unwrap_or_default();
        let removed = self
            .build_pim_changes(account_id, ChangeType::Task, since)
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

    // ── helpers ──────────────────────────────────────────────────────────────

    pub(crate) async fn all_task_ids(&self, account_id: &str) -> Result<Vec<String>> {
        let cals = self.store().list_calendars(account_id).await?;
        let mut ids = Vec::new();
        for cal in &cals {
            if cal.component != "VTODO" {
                continue;
            }
            for t in self.store().list_tasks(&cal.id).await? {
                ids.push(t.id);
            }
        }
        Ok(ids)
    }

    /// Seed a task title/description from a `fromEmail` / `fromEvent` source.
    async fn apply_from_sources(&self, _account_id: &str, json: &mut Value, spec: &Value) {
        let has_title = json
            .get("title")
            .and_then(Value::as_str)
            .is_some_and(|s| !s.is_empty());
        if let Some(email_id) = spec
            .get("fromEmail")
            .and_then(|f| f.get("emailId"))
            .and_then(Value::as_str)
        {
            if !has_title
                && let Ok(Some(bytes)) = self.store().get_envelope(email_id).await
                && let Ok(env) = serde_json::from_slice::<Value>(&bytes)
                && let Some(subject) = env.get("subject").and_then(Value::as_str)
            {
                set_str(json, "title", subject);
            }
            add_link(json, "email", email_id);
        }
        if let Some(event_id) = spec
            .get("fromEvent")
            .and_then(|f| f.get("eventId"))
            .and_then(Value::as_str)
        {
            if !has_title && let Ok(Some(row)) = self.store().get_event(event_id).await {
                let title = self
                    .event_row_to_json(&row)
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("Task")
                    .to_string();
                set_str(json, "title", &title);
            }
            add_link(json, "event", event_id);
        }
    }
}

// ── free helpers ─────────────────────────────────────────────────────────────

fn normalize_task(json: &mut Value) {
    let obj = match json.as_object_mut() {
        Some(o) => o,
        None => return,
    };
    obj.entry("status").or_insert_with(|| json!("needs-action"));
    obj.entry("title").or_insert_with(|| json!(""));
    obj.entry("percentComplete").or_insert_with(|| json!(0));
    obj.entry("priority").or_insert_with(|| json!(0));
    // A task must project to VTODO: ensure the emitter sees a task discriminator.
    if obj.get("due").is_none() && obj.get("listId").is_none() {
        obj.insert("listId".into(), json!(""));
    }
}

/// My Day membership (§1.4): pinned to today, or due/starting on-or-before today.
fn is_my_day(t: &TaskRow, today: &str) -> bool {
    if t.my_day_date.as_deref() == Some(today) {
        return true;
    }
    if let Some(due) = &t.due_utc
        && date_part(due) <= today
    {
        return true;
    }
    if let Some(start) = &t.start_utc
        && date_part(start) <= today
    {
        return true;
    }
    false
}

fn date_part(dt: &str) -> &str {
    dt.split('T').next().unwrap_or(dt)
}

fn add_link(json: &mut Value, link_type: &str, id: &str) {
    let entry = json!({ "type": link_type, "id": id });
    match json.get_mut("links").and_then(Value::as_array_mut) {
        Some(arr) => arr.push(entry),
        None => set_json(json, "links", json!([entry])),
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

/// The §2.1 `Task` JSON for a row (stored canonical projection, patched with the
/// row identity + the engine-owned columns).
fn task_row_to_json(row: &TaskRow) -> Value {
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
    set_str(&mut json, "listId", &row.list_id);
    set_str(&mut json, "uid", &row.uid);
    set_json(&mut json, "status", json!(row.status));
    set_json(&mut json, "percentComplete", json!(row.percent_complete));
    set_json(
        &mut json,
        "myDayDate",
        row.my_day_date
            .clone()
            .map(Value::String)
            .unwrap_or(Value::Null),
    );
    set_json(
        &mut json,
        "etag",
        row.etag.clone().map(Value::String).unwrap_or(Value::Null),
    );
    json
}
