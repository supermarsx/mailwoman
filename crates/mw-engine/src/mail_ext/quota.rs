//! `Quota/get` (RFC 9425). Two sources, reported side by side:
//!
//! - the upstream mail server's own quota, read through the backend
//!   (`AccountBackend::quota`, IMAP `GETQUOTAROOT INBOX` per RFC 9208). Ids
//!   start with `upstream-`. A backend that cannot report quota (POP3, plugins,
//!   IMAP without `QUOTA`) contributes nothing;
//! - the admin-configured `quotas` row (0007, `bytes_limit` / `msg_limit`)
//!   against usage summed from the sealed message cache. Ids `mail-storage` /
//!   `mail-count`.
//!
//! With neither, the list is empty (RFC 9425 permits an empty result), and the
//! client shows no meter.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::backend::{AccountBackend, EngineError, QuotaUsage};
use crate::change::ChangeType;
use crate::engine::Engine;

use super::{server_fail, wanted_ids};

/// The two admin-configured quota ids (octets + count).
const STORAGE_ID: &str = "mail-storage";
const COUNT_ID: &str = "mail-count";

/// How long one upstream reading is reused. Quota moves slowly and the IMAP
/// session is shared with sync, so the sidebar meter must not issue a command
/// on every folder refresh.
const UPSTREAM_TTL: Duration = Duration::from_secs(60);
/// How long `Quota/get` waits for the shared IMAP session before it answers
/// with the last reading (or none).
const UPSTREAM_WAIT: Duration = Duration::from_secs(5);

/// One upstream reading, tied to the backend it came from: a redialled or
/// re-registered account (new backend) never sees the old account's numbers.
struct Reading {
    backend: Weak<dyn AccountBackend>,
    at: Instant,
    usage: Vec<QuotaUsage>,
}

fn cache() -> &'static Mutex<HashMap<String, Reading>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Reading>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

impl Engine {
    /// `Quota/get` (RFC 9425 §2.2): the upstream quota resources plus the
    /// administrator's storage + message-count limits.
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

        let mut all = upstream_objects(&self.upstream_quota(account_id).await);
        if let Some(limits) = limits {
            let msgs = match self.account_messages(account_id).await {
                Ok(m) => m,
                Err(e) => return server_fail(e),
            };
            let used_octets: u64 = msgs.iter().map(|m| m.size).sum();
            let used_count = msgs.len() as u64;
            all.push(admin_obj(
                STORAGE_ID,
                "octets",
                used_octets,
                limits.bytes_limit,
            ));
            all.push(admin_obj(COUNT_ID, "count", used_count, limits.msg_limit));
        }

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

    /// The upstream server's quota for the account's INBOX, reused for
    /// [`UPSTREAM_TTL`]. An account that is not connected, a backend without
    /// quota support, or a failed read yields the last good reading or nothing:
    /// a missing meter is better than a made-up one.
    async fn upstream_quota(&self, account_id: &str) -> Vec<QuotaUsage> {
        let Some(rt) = self.runtime(account_id) else {
            return Vec::new();
        };
        let backend = rt.backend.clone();
        let previous = {
            let mut cache = cache().lock().unwrap_or_else(|e| e.into_inner());
            cache.retain(|_, r| r.backend.strong_count() > 0);
            cache
                .get(account_id)
                .filter(|r| {
                    r.backend
                        .upgrade()
                        .is_some_and(|b| Arc::ptr_eq(&b, &backend))
                })
                .map(|r| (r.at, r.usage.clone()))
        };
        if let Some((at, usage)) = &previous
            && at.elapsed() < UPSTREAM_TTL
        {
            return usage.clone();
        }
        let remember = |usage: &[QuotaUsage]| {
            cache().lock().unwrap_or_else(|e| e.into_inner()).insert(
                account_id.to_string(),
                Reading {
                    backend: Arc::downgrade(&backend),
                    at: Instant::now(),
                    usage: usage.to_vec(),
                },
            );
        };
        match tokio::time::timeout(UPSTREAM_WAIT, backend.quota()).await {
            Ok(Ok(usage)) => {
                remember(&usage);
                usage
            }
            // Remember "no quota" too, so a server without QUOTA is not asked
            // again on every call.
            Ok(Err(EngineError::Unsupported(_))) => {
                remember(&[]);
                Vec::new()
            }
            Ok(Err(error)) => {
                tracing::warn!(account = account_id, "upstream quota read failed: {error}");
                previous.map(|(_, u)| u).unwrap_or_default()
            }
            Err(_) => previous.map(|(_, u)| u).unwrap_or_default(),
        }
    }
}

/// Map upstream resources onto RFC 9425 `Quota` objects. Only `STORAGE`
/// (1024-octet units, RFC 9208 §5.1) and `MESSAGE` have a JMAP resource type;
/// the rest (`MAILBOX`, `ANNOTATION-STORAGE`, …) are left out. The first root
/// gets the plain ids; any further root gets a numeric suffix.
fn upstream_objects(reading: &[QuotaUsage]) -> Vec<Value> {
    let mut roots: Vec<&str> = Vec::new();
    let mut out = Vec::new();
    for q in reading {
        let index = match roots.iter().position(|r| *r == q.root) {
            Some(i) => i,
            None => {
                roots.push(&q.root);
                roots.len() - 1
            }
        };
        let (kind, resource_type, used, limit) = match q.resource.as_str() {
            "STORAGE" => (
                "storage",
                "octets",
                q.usage.saturating_mul(1024),
                q.limit.saturating_mul(1024),
            ),
            "MESSAGE" => ("count", "count", q.usage, q.limit),
            _ => continue,
        };
        let id = if index == 0 {
            format!("upstream-{kind}")
        } else {
            format!("upstream-{kind}-{index}")
        };
        let name = if q.root.is_empty() {
            id.clone()
        } else {
            q.root.clone()
        };
        out.push(json!({
            "id": id,
            "resourceType": resource_type,
            "used": used,
            "hardLimit": limit,
            "scope": "account",
            "name": name,
            "types": ["Mail"],
            "warnLimit": Value::Null,
            "softLimit": Value::Null,
            "description": "Reported by the mail server"
        }));
    }
    out
}

/// One RFC 9425 `Quota` object from the admin row. `hardLimit` clamps a
/// negative sentinel to 0; a stored `0` limit means the resource is capped at
/// zero (no headroom), which is exactly what the admin row encodes.
fn admin_obj(id: &str, resource_type: &str, used: u64, hard_limit: i64) -> Value {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn usage(root: &str, resource: &str, usage: u64, limit: u64) -> QuotaUsage {
        QuotaUsage {
            root: root.into(),
            resource: resource.into(),
            usage,
            limit,
        }
    }

    #[test]
    fn storage_is_converted_from_kib_and_unknown_resources_are_dropped() {
        let objects = upstream_objects(&[
            usage("User quota", "STORAGE", 2, 10),
            usage("User quota", "MESSAGE", 3, 100),
            usage("User quota", "MAILBOX", 1, 5),
            usage("Shared", "STORAGE", 1, 4),
        ]);
        assert_eq!(objects.len(), 3);
        assert_eq!(objects[0]["id"], "upstream-storage");
        assert_eq!(objects[0]["used"], 2048);
        assert_eq!(objects[0]["hardLimit"], 10240);
        assert_eq!(objects[0]["name"], "User quota");
        assert_eq!(objects[1]["id"], "upstream-count");
        assert_eq!(objects[1]["resourceType"], "count");
        assert_eq!(objects[2]["id"], "upstream-storage-1");
    }
}
