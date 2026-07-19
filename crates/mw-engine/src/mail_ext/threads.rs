//! `Thread/get` + `Thread/changes` (RFC 8621 §3) over the real JWZ `thread_id`s
//! the engine assigns at ingest (`thread.rs`). A Thread object is just its
//! ordered `emailIds`; nothing here re-runs the threading algorithm — it groups
//! the messages the store already keyed onto each thread.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::change::ChangeType;
use crate::engine::Engine;

use super::{server_fail, wanted_ids};

impl Engine {
    /// `Thread/get` (RFC 8621 §3.2): map each thread id to its member email ids,
    /// oldest-first (`receivedAt` ascending, stable-id tie-break). With no `ids`
    /// the whole thread set is returned.
    pub(crate) async fn thread_get(&self, account_id: &str, args: &Value) -> Value {
        let msgs = match self.account_messages(account_id).await {
            Ok(m) => m,
            Err(e) => return server_fail(e),
        };
        // Group message ids by thread; sort each group oldest-first.
        let mut by_thread: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
        for m in &msgs {
            if let Some(t) = &m.thread_id {
                by_thread.entry(t.clone()).or_default().push((
                    m.internaldate.clone().unwrap_or_default(),
                    m.stable_id.clone(),
                ));
            }
        }
        for rows in by_thread.values_mut() {
            rows.sort();
        }

        let wanted = wanted_ids(args);
        let state = self.thread_state(account_id).await;
        let ids: Vec<String> = match &wanted {
            Some(ids) => ids.clone(),
            None => by_thread.keys().cloned().collect(),
        };

        let mut list = Vec::new();
        let mut not_found = Vec::new();
        for id in &ids {
            match by_thread.get(id) {
                Some(rows) => {
                    let email_ids: Vec<String> = rows.iter().map(|(_, sid)| sid.clone()).collect();
                    list.push(json!({ "id": id, "emailIds": email_ids }));
                }
                None => not_found.push(json!(id)),
            }
        }
        json!({
            "accountId": account_id,
            "state": state,
            "list": list,
            "notFound": not_found
        })
    }

    /// `Thread/changes` (RFC 8621 §3.3): a best-effort delta derived from the
    /// Email change log — a thread changes when its mail does. Created/updated
    /// emails still exist, so their thread resolves and is reported as `updated`
    /// (the client re-fetches via `Thread/get`); a destroyed email is gone, so
    /// its thread cannot be resolved here and is left to the client's own
    /// reconciliation. The state tokens track the Email counter (see
    /// [`Engine::thread_state`]).
    pub(crate) async fn thread_changes(&self, account_id: &str, args: &Value) -> Value {
        let since = args
            .get("sinceState")
            .and_then(Value::as_str)
            .unwrap_or("0");
        let changes = match self
            .build_changes(account_id, ChangeType::Email, since)
            .await
        {
            Ok(c) => c,
            Err(e) => return server_fail(e),
        };
        let mut updated: Vec<String> = Vec::new();
        for id in changes.created.iter().chain(changes.updated.iter()) {
            if let Ok(m) = self.store().get_message(id).await
                && let Some(t) = m.thread_id
                && !updated.contains(&t)
            {
                updated.push(t);
            }
        }
        json!({
            "accountId": account_id,
            "oldState": changes.old_state,
            "newState": changes.new_state,
            "hasMoreChanges": false,
            "created": [],
            "updated": updated,
            "destroyed": []
        })
    }

    /// The Thread state token. Thread identity moves with the mail that composes
    /// it, so the token tracks the Email counter — the same tie the push layer
    /// makes in `state.rs::broadcast_state` (`thread: email`).
    async fn thread_state(&self, account_id: &str) -> String {
        self.type_state(account_id, ChangeType::Email)
            .await
            .unwrap_or_default()
    }
}
