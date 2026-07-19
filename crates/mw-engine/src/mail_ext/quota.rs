//! `Quota/get` (RFC 9425). Reports the admin-configured `quotas` row (0007 —
//! `bytes_limit` / `msg_limit`) against live usage summed from the sealed
//! message cache. When no limit is configured there is nothing to enforce, so
//! the list is empty (RFC 9425 permits an empty result — no over-claiming).

use serde_json::{Value, json};

use crate::change::ChangeType;
use crate::engine::Engine;

use super::{server_fail, wanted_ids};

/// The two per-account quota ids (octets + count).
const STORAGE_ID: &str = "mail-storage";
const COUNT_ID: &str = "mail-count";

impl Engine {
    /// `Quota/get` (RFC 9425 §2.2): the account's storage + message-count quotas.
    pub(crate) async fn quota_get(&self, account_id: &str, args: &Value) -> Value {
        let state = self
            .type_state(account_id, ChangeType::Email)
            .await
            .unwrap_or_default();
        let limits = match self.store().get_quota(account_id).await {
            Ok(q) => q,
            Err(e) => return server_fail(e),
        };
        let wanted = wanted_ids(args);
        // No admin-configured limit ⇒ no enforced quota to report.
        let Some(limits) = limits else {
            return json!({
                "accountId": account_id,
                "state": state,
                "list": [],
                "notFound": wanted.unwrap_or_default(),
            });
        };

        let msgs = match self.account_messages(account_id).await {
            Ok(m) => m,
            Err(e) => return server_fail(e),
        };
        let used_octets: u64 = msgs.iter().map(|m| m.size).sum();
        let used_count = msgs.len() as u64;

        let all = [
            quota_obj(STORAGE_ID, "octets", used_octets, limits.bytes_limit),
            quota_obj(COUNT_ID, "count", used_count, limits.msg_limit),
        ];
        let mut list = Vec::new();
        let mut found = Vec::new();
        for q in all {
            let id = q["id"].as_str().unwrap_or_default().to_string();
            if wanted.as_ref().is_none_or(|ids| ids.contains(&id)) {
                found.push(id);
                list.push(q);
            }
        }
        let not_found: Vec<Value> = match &wanted {
            Some(ids) => ids
                .iter()
                .filter(|id| !found.contains(id))
                .map(|id| json!(id))
                .collect(),
            None => Vec::new(),
        };
        json!({
            "accountId": account_id,
            "state": state,
            "list": list,
            "notFound": not_found
        })
    }
}

/// One RFC 9425 `Quota` object. `hardLimit` clamps a negative sentinel to 0; a
/// stored `0` limit means the resource is capped at zero (no headroom), which is
/// exactly what the admin row encodes.
fn quota_obj(id: &str, resource_type: &str, used: u64, hard_limit: i64) -> Value {
    json!({
        "id": id,
        "resourceType": resource_type,
        "used": used,
        "hardLimit": hard_limit.max(0),
        "scope": "account",
        "name": id,
        "types": ["Mail"],
        "warnLimit": Value::Null,
        "softLimit": Value::Null,
        "description": Value::Null
    })
}
