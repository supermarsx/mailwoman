//! `VacationResponse/get|set` (RFC 8621 §8). VacationResponse is a per-account
//! **singleton** whose id is the fixed string `"singleton"`; create/destroy are
//! forbidden. The object persists in the existing `settings` table under a
//! per-account key (its text is auto-replied to any sender, so it carries no
//! secret and is not sealed), alongside a small monotonic state counter so
//! `set` can return honest `oldState`/`newState` tokens.

use serde_json::{Map, Value, json};

use crate::backend::Result;
use crate::engine::Engine;

use super::server_fail;

/// The fixed singleton id (RFC 8621 §8).
const SINGLETON: &str = "singleton";

/// The mutable object fields a `set` update may touch (`id` is immutable).
const MUTABLE_FIELDS: &[&str] = &[
    "isEnabled",
    "fromDate",
    "toDate",
    "subject",
    "textBody",
    "htmlBody",
];

fn setting_key(account_id: &str) -> String {
    format!("vacationResponse:{account_id}")
}

impl Engine {
    /// `VacationResponse/get` (RFC 8621 §8.1). Only the `singleton` id exists.
    pub(crate) async fn vacation_response_get(&self, account_id: &str, args: &Value) -> Value {
        let (obj, state) = match self.load_vacation(account_id).await {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        let (list, not_found) = match args.get("ids").and_then(Value::as_array) {
            Some(arr) => {
                let mut list = Vec::new();
                let mut nf = Vec::new();
                for id in arr.iter().filter_map(Value::as_str) {
                    if id == SINGLETON {
                        list.push(obj.clone());
                    } else {
                        nf.push(json!(id));
                    }
                }
                (list, nf)
            }
            None => (vec![obj.clone()], Vec::new()),
        };
        json!({
            "accountId": account_id,
            "state": state.to_string(),
            "list": list,
            "notFound": not_found
        })
    }

    /// `VacationResponse/set` (RFC 8621 §8.2): only an `update` of the singleton
    /// is allowed; `create`/`destroy` are refused per the spec.
    pub(crate) async fn vacation_response_set(&self, account_id: &str, args: &Value) -> Value {
        let (mut obj, mut state) = match self.load_vacation(account_id).await {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        let old_state = state.to_string();
        let mut updated = Map::new();
        let mut not_updated = Map::new();
        let mut not_created = Map::new();
        let mut not_destroyed = Map::new();

        if let Some(creates) = args.get("create").and_then(Value::as_object) {
            for cid in creates.keys() {
                not_created.insert(cid.clone(), singleton_error("create"));
            }
        }
        if let Some(destroys) = args.get("destroy").and_then(Value::as_array) {
            for id in destroys.iter().filter_map(Value::as_str) {
                not_destroyed.insert(id.to_string(), singleton_error("destroy"));
            }
        }

        if let Some(updates) = args.get("update").and_then(Value::as_object) {
            for (id, patch) in updates {
                if id != SINGLETON {
                    not_updated.insert(
                        id.clone(),
                        json!({ "type": "notFound", "description": "only the singleton VacationResponse exists" }),
                    );
                    continue;
                }
                apply_patch(&mut obj, patch);
                match self.store_vacation(account_id, &obj, state + 1).await {
                    Ok(()) => {
                        state += 1;
                        updated.insert(id.clone(), Value::Null);
                    }
                    Err(e) => {
                        not_updated.insert(
                            id.clone(),
                            json!({ "type": "serverFail", "description": e.to_string() }),
                        );
                    }
                }
            }
        }

        let mut resp = json!({
            "accountId": account_id,
            "oldState": old_state,
            "newState": state.to_string(),
            "created": {},
            "updated": Value::Object(updated),
            "destroyed": []
        });
        if !not_created.is_empty() {
            resp["notCreated"] = Value::Object(not_created);
        }
        if !not_updated.is_empty() {
            resp["notUpdated"] = Value::Object(not_updated);
        }
        if !not_destroyed.is_empty() {
            resp["notDestroyed"] = Value::Object(not_destroyed);
        }
        resp
    }

    /// Load the stored singleton + its state counter, or a disabled default.
    async fn load_vacation(&self, account_id: &str) -> Result<(Value, u64)> {
        match self.store().get_setting(&setting_key(account_id)).await? {
            Some(s) => {
                let stored: Value = serde_json::from_str(&s).unwrap_or_else(|_| json!({}));
                let state = stored.get("state").and_then(Value::as_u64).unwrap_or(0);
                let obj = stored.get("obj").cloned().unwrap_or_else(default_vacation);
                Ok((obj, state))
            }
            None => Ok((default_vacation(), 0)),
        }
    }

    /// Persist the singleton + its new state counter.
    async fn store_vacation(&self, account_id: &str, obj: &Value, state: u64) -> Result<()> {
        let stored = json!({ "obj": obj, "state": state });
        self.store()
            .set_setting(&setting_key(account_id), &stored.to_string())
            .await?;
        Ok(())
    }
}

fn singleton_error(op: &str) -> Value {
    json!({
        "type": "singleton",
        "description": format!("VacationResponse is a singleton; {op} is not allowed")
    })
}

fn default_vacation() -> Value {
    json!({
        "id": SINGLETON,
        "isEnabled": false,
        "fromDate": Value::Null,
        "toDate": Value::Null,
        "subject": Value::Null,
        "textBody": Value::Null,
        "htmlBody": Value::Null
    })
}

/// Apply a `set` update patch to the singleton, honoring only the mutable fields
/// and keeping the id fixed.
fn apply_patch(obj: &mut Value, patch: &Value) {
    let (Some(patch), Some(map)) = (patch.as_object(), obj.as_object_mut()) else {
        return;
    };
    for key in MUTABLE_FIELDS {
        if let Some(v) = patch.get(*key) {
            map.insert((*key).to_string(), v.clone());
        }
    }
    map.insert("id".into(), json!(SINGLETON));
}
