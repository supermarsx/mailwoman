//! Bridge PIM routing (plan §2.2, §1.3; t10-e5).
//!
//! The engine PREFERS a bridge when a backend advertises the calendar/tasks
//! capability, else it takes the byte-unchanged CalDAV/standards path
//! ([`crate::pim::sync`]). The preference is expressed exactly like the existing
//! `bridge_reactions`/`bridge_voting`/`bridge_recall`/`bridge_focused_sync` accessor
//! pattern: [`Engine::bridge_calendar`] / [`Engine::bridge_tasks`] return
//! `Some(Arc<dyn …>)` only when a [`crate::v7::BridgeCapabilitySource`] advertises
//! it for the account — and that is `None` until e13 injects the mw-plugin-backed
//! adapters at mount. So a non-bridge account (plain IMAP/POP3/DAV, or nothing
//! attached) drives **every** method here into an early `None` return: no store
//! write, no side effect, the standards fallback runs unchanged (the §1.3 hard gate).
//!
//! ## The seam
//! - **Read (list/sync):** [`Engine::sync_pim_via_bridge`] is called first from
//!   [`Engine::sync_pim`]. When a bridge calendar/tasks cap is advertised it mirrors
//!   the bridge's collections into native store rows (no `caldav_url`, so the DAV
//!   pull never double-syncs them) and reconciles events/tasks over the bridge's
//!   opaque cursor. The ordinary `Calendar/get` / `CalendarEvent/*` / `Task/*` JMAP
//!   families then serve that landed data unchanged.
//! - **Write (complete):** [`Engine::maybe_bridge_complete_task`] mirrors a task
//!   completion upstream (best-effort, like the CalDAV push). The `calendar`/`tasks`
//!   WIT interfaces expose no create/update endpoint, so event/task *create* stays
//!   the local-native path; the bridge owns list/sync + completion.

use std::fmt::Write as _;

use mw_store::CalendarRow;
use serde_json::{Value, json};

use crate::backend::Result;
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;
use crate::v7::{BridgeCalendar, BridgeTasks};

use super::gen_id;

impl Engine {
    /// Pull PIM through the bridge when the account's backend advertises a bridge
    /// calendar/tasks capability. Returns `Ok(true)` when the bridge path was taken
    /// (at least one cap advertised), `Ok(false)` when nothing is advertised — in
    /// which case the caller ([`Engine::sync_pim`]) runs the standards fallback
    /// byte-for-byte unchanged. A `false` return performs **no** store write.
    pub(crate) async fn sync_pim_via_bridge(&self, account_id: &str) -> Result<bool> {
        let mut routed = false;
        if let Some(cal) = self.bridge_calendar(account_id) {
            self.pull_bridge_calendars(account_id, cal.as_ref()).await?;
            routed = true;
        }
        if let Some(tasks) = self.bridge_tasks(account_id) {
            self.pull_bridge_tasks(account_id, tasks.as_ref()).await?;
            routed = true;
        }
        Ok(routed)
    }

    /// Mirror the bridge's calendar collections + events into the local store,
    /// reconciling over the bridge's opaque per-collection cursor (persisted in the
    /// mirror row's `sync_token`, hex-encoded).
    async fn pull_bridge_calendars(
        &self,
        account_id: &str,
        cal: &dyn BridgeCalendar,
    ) -> Result<()> {
        for info in cal.list_calendars().await? {
            let local_id = bridge_local_id("bcal", account_id, &info.id);
            let existing = self.store().get_calendar(&local_id).await?;
            let is_new_collection = existing.is_none();
            let cursor = existing
                .as_ref()
                .and_then(|c| c.sync_token.as_deref())
                .map(decode_hex)
                .unwrap_or_default();

            let mut row = CalendarRow {
                id: local_id.clone(),
                account_id: account_id.to_string(),
                name: info.name.clone(),
                color: existing
                    .as_ref()
                    .map(|c| c.color.clone())
                    .unwrap_or_else(|| "#3366ff".to_string()),
                sort_order: existing.as_ref().map(|c| c.sort_order).unwrap_or(0),
                is_visible: existing.as_ref().map(|c| c.is_visible).unwrap_or(true),
                role: Some(info.role.clone()),
                caldav_url: None,
                sync_token: existing.as_ref().and_then(|c| c.sync_token.clone()),
                ctag: None,
                is_overlay: info.read_only,
                component: "VEVENT".to_string(),
            };
            self.store().upsert_calendar(&row).await?;
            if is_new_collection {
                self.record_pim_change(
                    account_id,
                    ChangeType::Calendar,
                    &local_id,
                    ChangeOp::Created,
                )
                .await?;
            }

            let delta = cal.sync_events(&info.id, &cursor).await?;
            for ev in delta.changed {
                let id = self
                    .event_id_for_uid(&local_id, &ev.id)
                    .await?
                    .unwrap_or_else(|| gen_id("ev"));
                let is_new = self.store().get_event(&id).await?.is_none();
                let projection = ical_to_projection(&ev.ical, &ev.id);
                // Best-effort per event — one malformed VEVENT never aborts the sync.
                match self
                    .persist_event(account_id, &local_id, &id, &ev.id, projection, None, None)
                    .await
                {
                    Ok(_) => {
                        self.record_pim_change(
                            account_id,
                            ChangeType::CalendarEvent,
                            &id,
                            if is_new {
                                ChangeOp::Created
                            } else {
                                ChangeOp::Updated
                            },
                        )
                        .await?;
                    }
                    Err(e) => tracing::warn!("bridge event {} persist failed: {e}", ev.id),
                }
            }
            for removed in &delta.removed {
                if let Some(id) = self.event_id_for_uid(&local_id, removed).await? {
                    self.store().delete_event(&id).await?;
                    self.record_pim_change(
                        account_id,
                        ChangeType::CalendarEvent,
                        &id,
                        ChangeOp::Destroyed,
                    )
                    .await?;
                }
            }

            // Advance + persist the opaque cursor for the next incremental pull.
            row.sync_token = Some(encode_hex(&delta.next_cursor));
            self.store().upsert_calendar(&row).await?;
        }
        Ok(())
    }

    /// Mirror the bridge's task lists + tasks into the local store. The `tasks` WIT
    /// interface has no list-collections method, so the lists are discovered from
    /// [`BridgeTasks::list_tasks`] (each task carries its `list_id`); the actual
    /// reconcile per list is cursor-driven via [`BridgeTasks::sync_tasks`].
    async fn pull_bridge_tasks(&self, account_id: &str, tasks: &dyn BridgeTasks) -> Result<()> {
        let mut list_ids: Vec<String> = Vec::new();
        for t in tasks.list_tasks().await? {
            if !list_ids.contains(&t.list_id) {
                list_ids.push(t.list_id);
            }
        }
        for list_id in &list_ids {
            let local_id = bridge_local_id("blist", account_id, list_id);
            let existing = self.store().get_calendar(&local_id).await?;
            let is_new_collection = existing.is_none();
            let cursor = existing
                .as_ref()
                .and_then(|c| c.sync_token.as_deref())
                .map(decode_hex)
                .unwrap_or_default();

            let mut row = CalendarRow {
                id: local_id.clone(),
                account_id: account_id.to_string(),
                name: existing
                    .as_ref()
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "Tasks".to_string()),
                color: existing
                    .as_ref()
                    .map(|c| c.color.clone())
                    .unwrap_or_else(|| "#8855ff".to_string()),
                sort_order: 0,
                is_visible: true,
                role: None,
                caldav_url: None,
                sync_token: existing.as_ref().and_then(|c| c.sync_token.clone()),
                ctag: None,
                is_overlay: false,
                component: "VTODO".to_string(),
            };
            self.store().upsert_calendar(&row).await?;
            if is_new_collection {
                self.record_pim_change(
                    account_id,
                    ChangeType::Calendar,
                    &local_id,
                    ChangeOp::Created,
                )
                .await?;
            }

            let delta = tasks.sync_tasks(list_id, &cursor).await?;
            for t in delta.changed {
                let id = self
                    .task_id_for_uid(&local_id, &t.id)
                    .await?
                    .unwrap_or_else(|| gen_id("task"));
                let is_new = self.store().get_task(&id).await?.is_none();
                let projection = ical_to_projection(&t.ical, &t.id);
                match self
                    .persist_task(account_id, &local_id, &id, &t.id, projection, None, None)
                    .await
                {
                    Ok(()) => {
                        self.record_pim_change(
                            account_id,
                            ChangeType::Task,
                            &id,
                            if is_new {
                                ChangeOp::Created
                            } else {
                                ChangeOp::Updated
                            },
                        )
                        .await?;
                    }
                    Err(e) => tracing::warn!("bridge task {} persist failed: {e}", t.id),
                }
            }
            for removed in &delta.removed {
                if let Some(id) = self.task_id_for_uid(&local_id, removed).await? {
                    self.store().delete_task(&id).await?;
                    self.record_pim_change(account_id, ChangeType::Task, &id, ChangeOp::Destroyed)
                        .await?;
                }
            }

            row.sync_token = Some(encode_hex(&delta.next_cursor));
            self.store().upsert_calendar(&row).await?;
        }
        Ok(())
    }

    /// Mirror a task completion upstream when the account's backend advertises a
    /// bridge tasks capability. Best-effort (like the CalDAV push): a bridge failure
    /// is logged, never fatal to the local mutation. No-op — hence byte-unchanged —
    /// when no bridge is advertised or the update does not complete the task.
    pub(crate) async fn maybe_bridge_complete_task(
        &self,
        account_id: &str,
        uid: &str,
        projection: &Value,
    ) {
        if !task_projection_is_complete(projection) {
            return;
        }
        if let Some(bridge) = self.bridge_tasks(account_id)
            && let Err(e) = bridge.complete(uid).await
        {
            tracing::warn!("bridge task complete for {uid} failed: {e}");
        }
    }
}

/// A deterministic, collision-free local collection id for a bridge collection —
/// namespaced by account so two accounts' identically-named bridge collections
/// never alias, and prefixed so it never collides with a native `cal-`/`list-` id.
fn bridge_local_id(prefix: &str, account_id: &str, bridge_id: &str) -> String {
    format!("{prefix}:{account_id}:{bridge_id}")
}

/// Whether a task projection represents a completed to-do (JMAP `progress:"completed"`,
/// iCalendar `status:"completed"`, or `percentComplete == 100`).
fn task_projection_is_complete(json: &Value) -> bool {
    let is = |k: &str| json.get(k).and_then(Value::as_str) == Some("completed");
    is("status")
        || is("progress")
        || json.get("percentComplete").and_then(Value::as_i64) == Some(100)
}

/// Parse a bridge's iCalendar component text (a bare `VEVENT`/`VTODO` or a full
/// `VCALENDAR`) into an engine projection. A wrapper is synthesized when the text is
/// a bare component; a parse failure degrades to a minimal `{uid}` projection.
fn ical_to_projection(ical: &str, fallback_uid: &str) -> Value {
    let wrapped;
    let body = if ical.contains("BEGIN:VCALENDAR") {
        ical
    } else {
        wrapped = format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Mailwoman//bridge//EN\r\n{}\r\nEND:VCALENDAR\r\n",
            ical.trim()
        );
        &wrapped
    };
    mw_ics::parse_ical(body.as_bytes())
        .ok()
        .and_then(|v| v.into_iter().next())
        .map(|p| p.json)
        .unwrap_or_else(|| json!({ "uid": fallback_uid }))
}

/// Hex-encode an opaque bridge cursor for storage in the string `sync_token` column
/// (the cursor is arbitrary bytes, not guaranteed UTF-8).
fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Decode a hex-encoded cursor; any malformed input degrades to an empty cursor
/// (a full resync), never a panic.
fn decode_hex(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i + 1 < bytes.len() {
        match (hex_val(bytes[i]), hex_val(bytes[i + 1])) {
            (Some(hi), Some(lo)) => out.push((hi << 4) | lo),
            _ => return Vec::new(),
        }
        i += 2;
    }
    out
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

    use crate::Engine;
    use crate::v7::{
        BridgeCalendar, BridgeCalendarInfo, BridgeCapabilitySource, BridgeCaps, BridgeEventDelta,
        BridgeEventInfo, BridgeFocusedSync, BridgeReactions, BridgeRecall, BridgeRoomInfo,
        BridgeTaskDelta, BridgeTaskInfo, BridgeTasks, BridgeVoting, V7Hooks,
    };

    // ── Mock bridge calendar/tasks (one VEVENT + one VTODO, cursor-driven) ────────

    struct MockCal;
    #[async_trait]
    impl BridgeCalendar for MockCal {
        async fn list_calendars(&self) -> crate::backend::Result<Vec<BridgeCalendarInfo>> {
            Ok(vec![BridgeCalendarInfo {
                id: "C1".into(),
                name: "Work".into(),
                role: "calendar".into(),
                read_only: false,
            }])
        }
        async fn sync_events(
            &self,
            _calendar_id: &str,
            cursor: &[u8],
        ) -> crate::backend::Result<BridgeEventDelta> {
            // Only serve the event on the first (empty-cursor) sync; a resync with the
            // advanced cursor yields nothing — proving the cursor round-tripped.
            let changed = if cursor.is_empty() {
                vec![BridgeEventInfo {
                    id: "E1".into(),
                    calendar_id: "C1".into(),
                    ical: "BEGIN:VEVENT\r\nUID:E1\r\nSUMMARY:Standup\r\nDTSTART:20260714T090000Z\r\nDTEND:20260714T093000Z\r\nEND:VEVENT".into(),
                    start: Some("2026-07-14T09:00:00Z".into()),
                    end: Some("2026-07-14T09:30:00Z".into()),
                }]
            } else {
                Vec::new()
            };
            Ok(BridgeEventDelta {
                changed,
                removed: Vec::new(),
                next_cursor: b"graph-delta-XYZ\x00\xff".to_vec(),
            })
        }
        async fn find_rooms(&self) -> crate::backend::Result<Vec<BridgeRoomInfo>> {
            Ok(Vec::new())
        }
        async fn get_schedule(
            &self,
            _who: &str,
            _start: &str,
            _end: &str,
        ) -> crate::backend::Result<String> {
            Ok(String::new())
        }
    }

    struct MockTasks {
        completed: Arc<std::sync::Mutex<Vec<String>>>,
    }
    #[async_trait]
    impl BridgeTasks for MockTasks {
        async fn list_tasks(&self) -> crate::backend::Result<Vec<BridgeTaskInfo>> {
            Ok(vec![BridgeTaskInfo {
                id: "T1".into(),
                list_id: "L1".into(),
                ical: String::new(),
                completed: false,
            }])
        }
        async fn sync_tasks(
            &self,
            _list_id: &str,
            cursor: &[u8],
        ) -> crate::backend::Result<BridgeTaskDelta> {
            let changed = if cursor.is_empty() {
                vec![BridgeTaskInfo {
                    id: "T1".into(),
                    list_id: "L1".into(),
                    ical: "BEGIN:VTODO\r\nUID:T1\r\nSUMMARY:File report\r\nSTATUS:NEEDS-ACTION\r\nEND:VTODO".into(),
                    completed: false,
                }]
            } else {
                Vec::new()
            };
            Ok(BridgeTaskDelta {
                changed,
                removed: Vec::new(),
                next_cursor: b"todo-delta-1".to_vec(),
            })
        }
        async fn complete(&self, id: &str) -> crate::backend::Result<()> {
            self.completed.lock().unwrap().push(id.to_string());
            Ok(())
        }
    }

    struct MockSource {
        bridge_account: String,
        completed: Arc<std::sync::Mutex<Vec<String>>>,
    }
    impl BridgeCapabilitySource for MockSource {
        fn caps(&self, _account_id: &str) -> BridgeCaps {
            // calendar/tasks advertise via the `calendar()`/`tasks()` accessors, not
            // the coarse `BridgeCaps` bools (which cover reactions/voting/recall/focused).
            BridgeCaps::default()
        }
        fn reactions(&self, _a: &str) -> Option<Arc<dyn BridgeReactions>> {
            None
        }
        fn voting(&self, _a: &str) -> Option<Arc<dyn BridgeVoting>> {
            None
        }
        fn recall(&self, _a: &str) -> Option<Arc<dyn BridgeRecall>> {
            None
        }
        fn focused_sync(&self, _a: &str) -> Option<Arc<dyn BridgeFocusedSync>> {
            None
        }
        fn calendar(&self, account_id: &str) -> Option<Arc<dyn BridgeCalendar>> {
            (account_id == self.bridge_account)
                .then(|| Arc::new(MockCal) as Arc<dyn BridgeCalendar>)
        }
        fn tasks(&self, account_id: &str) -> Option<Arc<dyn BridgeTasks>> {
            (account_id == self.bridge_account).then(|| {
                Arc::new(MockTasks {
                    completed: self.completed.clone(),
                }) as Arc<dyn BridgeTasks>
            })
        }
    }

    async fn account(store: &Store, host: &str, user: &str) -> String {
        store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host,
                    port: 443,
                    tls: "implicit",
                    username: user,
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: user.into(),
                    password: "pw".into(),
                },
            )
            .await
            .unwrap()
    }

    /// An advertised calendar/tasks cap routes to the bridge: the mock's event +
    /// task land in the store as native rows, and the opaque cursor round-trips.
    #[tokio::test]
    async fn advertised_caps_route_to_bridge() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let bridge_account = account(&store, "graph.microsoft.com", "me@contoso.com").await;
        let engine = Engine::new(store);
        let completed = Arc::new(std::sync::Mutex::new(Vec::new()));
        engine.attach_v7(V7Hooks::new().with_bridge_caps(Arc::new(MockSource {
            bridge_account: bridge_account.clone(),
            completed: completed.clone(),
        })));

        // Accessors follow the same pattern as reactions/voting/recall/focused.
        assert!(engine.bridge_calendar(&bridge_account).is_some());
        assert!(engine.bridge_tasks(&bridge_account).is_some());

        let routed = engine.sync_pim_via_bridge(&bridge_account).await.unwrap();
        assert!(routed, "an advertised account takes the bridge path");

        // The bridge's collections + items landed as native store rows.
        let cals = engine
            .store()
            .list_calendars(&bridge_account)
            .await
            .unwrap();
        let vevent = cals.iter().find(|c| c.component == "VEVENT").unwrap();
        let vtodo = cals.iter().find(|c| c.component == "VTODO").unwrap();
        assert_eq!(
            engine.store().list_events(&vevent.id).await.unwrap().len(),
            1
        );
        assert_eq!(engine.store().list_tasks(&vtodo.id).await.unwrap().len(), 1);

        // Re-sync with the advanced cursor is idempotent (no duplicate rows) —
        // proving the opaque bridge cursor was persisted + handed back.
        engine.sync_pim_via_bridge(&bridge_account).await.unwrap();
        assert_eq!(
            engine.store().list_events(&vevent.id).await.unwrap().len(),
            1
        );
        assert_eq!(engine.store().list_tasks(&vtodo.id).await.unwrap().len(), 1);

        // The write path: completing a task mirrors the completion upstream via the
        // bridge (the mock records the uid it was asked to complete).
        engine
            .maybe_bridge_complete_task(
                &bridge_account,
                "T1",
                &serde_json::json!({ "status": "completed" }),
            )
            .await;
        assert_eq!(completed.lock().unwrap().as_slice(), ["T1"]);

        // A non-completing update does NOT touch the bridge.
        engine
            .maybe_bridge_complete_task(
                &bridge_account,
                "T1",
                &serde_json::json!({ "status": "in-process" }),
            )
            .await;
        assert_eq!(completed.lock().unwrap().len(), 1);
    }

    /// A non-advertising account performs NO bridge work and NO store write — the
    /// standards fallback stays byte-for-byte unchanged (the §1.3 hard gate).
    #[tokio::test]
    async fn non_advertising_account_is_byte_unchanged() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let plain = account(&store, "imap.example.com", "user@example.com").await;
        let engine = Engine::new(store);

        // Nothing attached: accessors are None, the router is a strict no-op.
        assert!(engine.bridge_calendar(&plain).is_none());
        assert!(engine.bridge_tasks(&plain).is_none());
        assert!(!engine.sync_pim_via_bridge(&plain).await.unwrap());
        assert!(
            engine
                .store()
                .list_calendars(&plain)
                .await
                .unwrap()
                .is_empty()
        );

        // A source attached but not advertising THIS account: still None ⇒ no-op.
        let completed = Arc::new(std::sync::Mutex::new(Vec::new()));
        engine.attach_v7(V7Hooks::new().with_bridge_caps(Arc::new(MockSource {
            bridge_account: "some-other-account".into(),
            completed,
        })));
        assert!(engine.bridge_calendar(&plain).is_none());
        assert!(engine.bridge_tasks(&plain).is_none());
        assert!(!engine.sync_pim_via_bridge(&plain).await.unwrap());
        assert!(
            engine
                .store()
                .list_calendars(&plain)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn cursor_hex_round_trips() {
        let raw = b"graph-delta\x00\xff\x10";
        assert_eq!(super::decode_hex(&super::encode_hex(raw)), raw);
        // Malformed input degrades to an empty cursor (full resync), never a panic.
        assert!(super::decode_hex("zz").is_empty());
        assert!(super::decode_hex("abc").is_empty());
    }
}
