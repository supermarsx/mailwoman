//! The engine's PIM surface (plan §0, §1.1, §2.1/§2.2): the Mailwoman-native
//! calendar / tasks / notes / contacts method families, dispatched over the same
//! `handle_jmap` envelope the mail surface uses (result references, per-account
//! state, cookie auth, the WS/SSE push channel) but under Mailwoman capability
//! URNs (`urn:mailwoman:{calendars,tasks,notes,contacts}`).
//!
//! e0 froze the [`types`] (§2.1 object shapes) + the [`dispatch`] method arms;
//! e8 fills the families here:
//! - [`calendars`] — `Calendar/*`, `Calendar/freeBusy`, `Calendar/detectConflicts`.
//! - [`events`] — `CalendarEvent/*` incl. `expand`/`parse`/`import`/`export`/`respond`.
//! - [`tasks`] — `Task/*` (VTODO routing + My Day + `fromEmail`/`fromEvent`).
//! - [`notes`] — `Note/*` (sealed-at-rest CRUD + title/tag/body search).
//! - [`contacts`] — `AddressBook/*`, `ContactCard/*` (+ merge/import/export/
//!   autocomplete), `ContactGroup/*`.
//! - [`sync`] — the CalDAV/CardDAV pull/push orchestration driving
//!   `mw-dav`/`mw-carddav` (etag/sync-token/ctag reconcile, plan §2.3).
//! - [`bridge`] — the **bridge PIM routing** (plan §2.2, t10-e5): when a backend
//!   advertises a bridge calendar/tasks capability the engine pulls PIM through the
//!   bridge trait objects; absence ⇒ the [`sync`] standards path runs byte-unchanged.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value, json};

pub mod bridge;
pub mod calendars;
pub mod contacts;
pub mod dispatch;
pub mod events;
pub mod notes;
pub mod quickadd;
pub mod sync;
pub mod tasks;
pub mod types;

/// Materialization horizon for recurrence expansion (plan §2.4): events are
/// expanded into `event_instances` over `[now - BACK, now + FWD]` on each write,
/// bounding the row count while covering the range the UI queries. Recurrences
/// beyond the horizon are re-expanded on demand by `CalendarEvent/expand`.
const MATERIALIZE_BACK_DAYS: i64 = 2 * 365;
const MATERIALIZE_FWD_DAYS: i64 = 5 * 365;

/// The `[start, end)` UTC horizon (RFC3339) over which an event's recurrence is
/// materialized on write.
pub(crate) fn materialize_window() -> (String, String) {
    let now = chrono::Utc::now();
    let start = now - chrono::Duration::days(MATERIALIZE_BACK_DAYS);
    let end = now + chrono::Duration::days(MATERIALIZE_FWD_DAYS);
    (
        start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        end.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    )
}

static COUNTER: AtomicU64 = AtomicU64::new(1);

/// A monotonic-ish unique token for generated PIM ids / uids.
pub(crate) fn gen_token() -> String {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{t:x}{n:x}")
}

/// A prefixed stable id (e.g. `"ev-<token>"`).
pub(crate) fn gen_id(prefix: &str) -> String {
    format!("{prefix}-{}", gen_token())
}

/// An RFC3339 timestamp for `createdAt`/`updatedAt`.
pub(crate) fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// A JMAP method-level failure result (`serverFail`).
pub(crate) fn server_fail(msg: impl std::fmt::Display) -> Value {
    json!({ "type": "serverFail", "description": msg.to_string() })
}

/// A JMAP `SetError` for a failed create/update/destroy entry.
pub(crate) fn set_error(kind: &str, msg: impl std::fmt::Display) -> Value {
    json!({ "type": kind, "description": msg.to_string() })
}

/// The accumulator for a `*/set` response (frozen §2.1 shape), tracking the
/// created / updated / destroyed maps plus their failure counterparts.
#[derive(Default)]
pub(crate) struct SetOutcome {
    pub created: Map<String, Value>,
    pub updated: Map<String, Value>,
    pub destroyed: Vec<Value>,
    pub not_created: Map<String, Value>,
    pub not_updated: Map<String, Value>,
    pub not_destroyed: Map<String, Value>,
}

impl SetOutcome {
    /// Assemble the `{accountId,oldState,newState,created,updated,destroyed,…}`
    /// response object (omitting empty `notX` maps, like the mail surface).
    pub fn into_response(self, account_id: &str, old_state: &str, new_state: &str) -> Value {
        let mut resp = json!({
            "accountId": account_id,
            "oldState": old_state,
            "newState": new_state,
            "created": Value::Object(self.created),
            "updated": Value::Object(self.updated),
            "destroyed": Value::Array(self.destroyed),
        });
        if !self.not_created.is_empty() {
            resp["notCreated"] = Value::Object(self.not_created);
        }
        if !self.not_updated.is_empty() {
            resp["notUpdated"] = Value::Object(self.not_updated);
        }
        if !self.not_destroyed.is_empty() {
            resp["notDestroyed"] = Value::Object(self.not_destroyed);
        }
        resp
    }
}

/// Build a standard `*/get` response envelope (`{accountId,state,list,notFound}`).
pub(crate) fn get_response(
    account_id: &str,
    state: &str,
    list: Vec<Value>,
    not_found: Vec<Value>,
) -> Value {
    json!({
        "accountId": account_id,
        "state": state,
        "list": list,
        "notFound": not_found,
    })
}

/// Build a standard `*/query` response envelope.
pub(crate) fn query_response(account_id: &str, query_state: &str, ids: Vec<String>) -> Value {
    json!({
        "accountId": account_id,
        "queryState": query_state,
        "ids": ids.clone(),
        "total": ids.len(),
        "position": 0,
        "canCalculateChanges": true,
    })
}

/// Optionally restrict a get to the caller's `ids` array (returns `None` to mean
/// "all"), as a set of owned strings.
pub(crate) fn wanted_ids(args: &Value) -> Option<Vec<String>> {
    args.get("ids").and_then(Value::as_array).map(|a| {
        a.iter()
            .filter_map(Value::as_str)
            .map(String::from)
            .collect()
    })
}
