//! Data subject requests: the queue, its event trail, built export parts and the
//! per-table erasure of one person's rows.
//!
//! A request row keeps the filterable columns; everything else is in `body_json`
//! and is changed with a compare-and-set on `rev`. Export parts are one file per
//! name, split into chunks, optionally sealed with the server key.
use crate::{Store, StoreError, q};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// One request as stored.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DsrRow {
    pub id: String,
    pub address: String,
    pub domain: String,
    pub account_id: String,
    pub kind: String,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub due_at: i64,
    pub assignee: Option<String>,
    pub rev: i64,
    pub body_json: String,
}

/// One entry of a request's trail.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DsrEvent {
    pub id: String,
    pub request_id: String,
    pub ts: i64,
    pub actor: String,
    pub kind: String,
    pub detail: Value,
}

/// Upper bound on rows one listing reads (callers filter further in memory).
pub const MAX_LIST: i64 = 5000;
/// Size of one stored export chunk.
pub const CHUNK: usize = 1 << 20;

fn row_of(r: &crate::backend::Row) -> DsrRow {
    DsrRow {
        id: r.get_string("id"),
        address: r.get_string("address"),
        domain: r.get_string("domain"),
        account_id: r.get_string("account_id"),
        kind: r.get_string("kind"),
        status: r.get_string("status"),
        created_at: r.get_i64("created_at"),
        updated_at: r.get_i64("updated_at"),
        due_at: r.get_i64("due_at"),
        assignee: r.get_opt_string("assignee"),
        rev: r.get_i64("rev"),
        body_json: r.get_string("body_json"),
    }
}

/// Erasure statements by `account_id`: table name and its fixed DELETE.
const BY_ACCOUNT: &[(&str, &str)] = &[
    ("totp_secrets", "DELETE FROM totp_secrets WHERE account_id=?1"),
    ("webauthn_credentials", "DELETE FROM webauthn_credentials WHERE account_id=?1"),
    ("recovery_codes", "DELETE FROM recovery_codes WHERE account_id=?1"),
    ("remembered_device", "DELETE FROM remembered_device WHERE account_id=?1"),
    ("push_subscriptions", "DELETE FROM push_subscriptions WHERE account_id=?1"),
    ("signatures", "DELETE FROM signatures WHERE account_id=?1"),
    ("notification_rules", "DELETE FROM notification_rules WHERE account_id=?1"),
    ("signin_event", "DELETE FROM signin_event WHERE account_id=?1"),
    ("signin_identity", "DELETE FROM signin_identity WHERE account_id=?1"),
    ("user_activity", "DELETE FROM user_activity WHERE account_id=?1"),
    ("masked_email", "DELETE FROM masked_email WHERE account_id=?1"),
    ("api_keys", "DELETE FROM api_keys WHERE account_id=?1"),
    ("sieve_script", "DELETE FROM sieve_script WHERE account_id=?1"),
    ("identities", "DELETE FROM identities WHERE account_id=?1"),
    ("password_change_audit", "DELETE FROM password_change_audit WHERE account_id=?1"),
    ("account_draft", "DELETE FROM account_draft WHERE account_id=?1"),
    ("delegation_grant", "DELETE FROM delegation_grant WHERE owner_account_id=?1"),
    ("calendars", "DELETE FROM calendars WHERE account_id=?1"),
    ("address_books", "DELETE FROM address_books WHERE account_id=?1"),
    ("notes", "DELETE FROM notes WHERE account_id=?1"),
    ("notebooks", "DELETE FROM notebooks WHERE account_id=?1"),
];

/// Erasure statements by the personal owner subject.
const BY_OWNER: &[(&str, &str)] = &[
    ("personal_setting_value", "DELETE FROM personal_setting_value WHERE owner_subject=?1"),
    ("personal_signature", "DELETE FROM personal_signature WHERE owner_subject=?1"),
    ("personal_saved_search", "DELETE FROM personal_saved_search WHERE owner_subject=?1"),
    ("personal_identity", "DELETE FROM personal_identity WHERE owner_subject=?1"),
    ("personal_notification_rules", "DELETE FROM personal_notification_rules WHERE owner_subject=?1"),
    ("personal_push_subscription", "DELETE FROM personal_push_subscription WHERE owner_subject=?1"),
    ("focused_sender", "DELETE FROM focused_sender WHERE owner_subject=?1"),
];

impl Store {
    pub async fn dsr_insert(&self, row: &DsrRow) -> Result<(), StoreError> {
        q("INSERT INTO dsr_request(id,address,domain,account_id,kind,status,created_at,updated_at,due_at,assignee,rev,body_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)")
            .bind(&row.id).bind(&row.address).bind(&row.domain).bind(&row.account_id)
            .bind(&row.kind).bind(&row.status).bind(row.created_at).bind(row.updated_at)
            .bind(row.due_at).bind(row.assignee.clone()).bind(row.rev).bind(&row.body_json)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    pub async fn dsr_get(&self, id: &str) -> Result<Option<DsrRow>, StoreError> {
        Ok(q("SELECT id,address,domain,account_id,kind,status,created_at,updated_at,due_at,assignee,rev,body_json FROM dsr_request WHERE id=?1")
            .bind(id)
            .fetch_optional(&self.backend)
            .await?
            .as_ref()
            .map(row_of))
    }

    /// Write `row` when the stored `rev` is still `row.rev`; the stored rev becomes
    /// `row.rev + 1`. `false` when another writer got there first.
    pub async fn dsr_update(&self, row: &DsrRow, now: i64) -> Result<bool, StoreError> {
        let n = q("UPDATE dsr_request SET status=?3,updated_at=?4,due_at=?5,assignee=?6,rev=?2+1,body_json=?7 WHERE id=?1 AND rev=?2")
            .bind(&row.id).bind(row.rev).bind(&row.status).bind(now).bind(row.due_at)
            .bind(row.assignee.clone()).bind(&row.body_json)
            .execute(&self.backend)
            .await?;
        Ok(n == 1)
    }

    /// Newest first, at most [`MAX_LIST`]; `address` narrows to one person.
    pub async fn dsr_list(&self, address: Option<&str>) -> Result<Vec<DsrRow>, StoreError> {
        let rows = match address {
            Some(a) => {
                q("SELECT id,address,domain,account_id,kind,status,created_at,updated_at,due_at,assignee,rev,body_json FROM dsr_request WHERE address=?1 ORDER BY created_at DESC,id LIMIT 5000")
                    .bind(a).fetch_all(&self.backend).await?
            }
            None => {
                q("SELECT id,address,domain,account_id,kind,status,created_at,updated_at,due_at,assignee,rev,body_json FROM dsr_request ORDER BY created_at DESC,id LIMIT 5000")
                    .fetch_all(&self.backend).await?
            }
        };
        Ok(rows.iter().map(row_of).collect())
    }

    /// Requests of one person created at or after `since` (the rate limit).
    pub async fn dsr_count_since(&self, address: &str, since: i64) -> Result<i64, StoreError> {
        Ok(q("SELECT COUNT(*) FROM dsr_request WHERE address=?1 AND created_at>=?2")
            .bind(address)
            .bind(since)
            .fetch_scalar_i64(&self.backend)
            .await?)
    }

    pub async fn dsr_event_add(
        &self,
        request_id: &str,
        actor: &str,
        kind: &str,
        detail: &Value,
        now: i64,
    ) -> Result<(), StoreError> {
        q("INSERT INTO dsr_event(id,request_id,ts,actor,kind,detail_json) VALUES(?1,?2,?3,?4,?5,?6)")
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(request_id)
            .bind(now)
            .bind(actor)
            .bind(kind)
            .bind(detail.to_string())
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    pub async fn dsr_events(&self, request_id: &str) -> Result<Vec<DsrEvent>, StoreError> {
        Ok(q("SELECT id,request_id,ts,actor,kind,detail_json FROM dsr_event WHERE request_id=?1 ORDER BY ts,id LIMIT 1000")
            .bind(request_id)
            .fetch_all(&self.backend)
            .await?
            .iter()
            .map(|r| DsrEvent {
                id: r.get_string("id"),
                request_id: r.get_string("request_id"),
                ts: r.get_i64("ts"),
                actor: r.get_string("actor"),
                kind: r.get_string("kind"),
                detail: serde_json::from_str(&r.get_string("detail_json")).unwrap_or(Value::Null),
            })
            .collect())
    }

    /// Store one export file, replacing a part of the same name. Content is split
    /// into [`CHUNK`]-sized rows; sealed with the server key when `seal`.
    pub async fn dsr_part_put(
        &self,
        request_id: &str,
        name: &str,
        data: &[u8],
        seal: bool,
    ) -> Result<(), StoreError> {
        q("DELETE FROM dsr_part WHERE request_id=?1 AND name=?2")
            .bind(request_id)
            .bind(name)
            .execute(&self.backend)
            .await?;
        let empty: [&[u8]; 1] = [&[]];
        let chunks: Vec<&[u8]> = if data.is_empty() {
            empty.to_vec()
        } else {
            data.chunks(CHUNK).collect()
        };
        for (seq, chunk) in chunks.into_iter().enumerate() {
            let stored = if seal {
                self.seal_blob(chunk)?
            } else {
                chunk.to_vec()
            };
            q("INSERT INTO dsr_part(request_id,name,seq,sealed,data) VALUES(?1,?2,?3,?4,?5)")
                .bind(request_id)
                .bind(name)
                .bind(seq as i64)
                .bind(i64::from(seal))
                .bind(stored)
                .execute(&self.backend)
                .await?;
        }
        Ok(())
    }

    /// Names of the stored parts, sorted.
    pub async fn dsr_part_names(&self, request_id: &str) -> Result<Vec<String>, StoreError> {
        Ok(q("SELECT DISTINCT name FROM dsr_part WHERE request_id=?1 ORDER BY name")
            .bind(request_id)
            .fetch_all_scalar_string(&self.backend)
            .await?)
    }

    /// One part's content, unsealed and joined.
    pub async fn dsr_part_read(
        &self,
        request_id: &str,
        name: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let rows = q("SELECT sealed,data FROM dsr_part WHERE request_id=?1 AND name=?2 ORDER BY seq")
            .bind(request_id)
            .bind(name)
            .fetch_all(&self.backend)
            .await?;
        if rows.is_empty() {
            return Ok(None);
        }
        let mut out = Vec::new();
        for r in rows {
            let data = r.get_blob("data");
            if r.get_i64("sealed") != 0 {
                out.extend(self.open_blob(&data)?);
            } else {
                out.extend(data);
            }
        }
        Ok(Some(out))
    }

    /// Delete every part of a request; the number of rows removed.
    pub async fn dsr_parts_delete(&self, request_id: &str) -> Result<u64, StoreError> {
        Ok(q("DELETE FROM dsr_part WHERE request_id=?1")
            .bind(request_id)
            .execute(&self.backend)
            .await?)
    }

    /// Delete a finished request and its trail (retention).
    pub async fn dsr_delete(&self, id: &str) -> Result<(), StoreError> {
        for sql in [
            q("DELETE FROM dsr_part WHERE request_id=?1"),
            q("DELETE FROM dsr_event WHERE request_id=?1"),
            q("DELETE FROM dsr_request WHERE id=?1"),
        ] {
            sql.bind(id).execute(&self.backend).await?;
        }
        Ok(())
    }

    /// Audit entries about a person: by actor (account id) or target.
    pub async fn dsr_audit_entries(
        &self,
        account_ids: &[String],
        limit: i64,
    ) -> Result<Vec<Value>, StoreError> {
        let mut out = Vec::new();
        for id in account_ids {
            let target = format!("account:{id}");
            let rows = q("SELECT id,ts,actor,actor_kind,action,target,detail_json FROM audit_log WHERE actor=?1 OR target=?2 ORDER BY ts DESC LIMIT ?3")
                .bind(id)
                .bind(&target)
                .bind(limit.clamp(1, 20000))
                .fetch_all(&self.backend)
                .await?;
            for r in rows {
                out.push(serde_json::json!({
                    "id": r.get_string("id"), "ts": r.get_string("ts"),
                    "actor": r.get_string("actor"), "actorKind": r.get_string("actor_kind"),
                    "action": r.get_string("action"), "target": r.get_opt_string("target"),
                    "detail": serde_json::from_str::<Value>(&r.get_string("detail_json")).unwrap_or(Value::Null),
                }));
            }
        }
        Ok(out)
    }

    /// User-scope setting rows of a person (`account` id scope or address scope).
    pub async fn dsr_user_settings(&self, scope_ids: &[String]) -> Result<Vec<Value>, StoreError> {
        let mut out = Vec::new();
        for id in scope_ids {
            for r in q("SELECT key,value,updated_at FROM setting_value WHERE scope_kind='user' AND scope_id=?1 ORDER BY key")
                .bind(id)
                .fetch_all(&self.backend)
                .await?
            {
                out.push(serde_json::json!({
                    "key": r.get_string("key"),
                    "value": serde_json::from_str::<Value>(&r.get_string("value")).unwrap_or(Value::Null),
                    "updatedAt": r.get_i64("updated_at"),
                }));
            }
        }
        Ok(out)
    }

    /// The directory row and bound account ids of a person.
    pub async fn dsr_profile(&self, address: &str) -> Result<Value, StoreError> {
        let row = q("SELECT address,domain,origin,status,display_name,created_at,status_changed_at FROM mail_user WHERE address=?1")
            .bind(address)
            .fetch_optional(&self.backend)
            .await?;
        Ok(match row {
            Some(r) => serde_json::json!({
                "address": r.get_string("address"), "domain": r.get_string("domain"),
                "origin": r.get_string("origin"), "status": r.get_string("status"),
                "displayName": r.get_opt_string("display_name"),
                "createdAt": r.get_opt_i64("created_at"),
                "statusChangedAt": r.get_opt_i64("status_changed_at"),
            }),
            None => serde_json::json!({ "address": address }),
        })
    }

    /// Delete one person's rows from every personal table. Returns rows removed
    /// per table; a table that fails is reported under `errors`, and the rest are
    /// still processed (the erasure is idempotent and is run again).
    pub async fn dsr_purge_personal(
        &self,
        account_ids: &[String],
        owners: &[String],
        address: &str,
    ) -> (BTreeMap<String, u64>, Vec<String>) {
        let mut counts = BTreeMap::new();
        let mut errors = Vec::new();
        for id in account_ids {
            for (table, sql) in BY_ACCOUNT {
                match self.dsr_exec(sql, id).await {
                    Ok(n) => *counts.entry((*table).to_string()).or_insert(0) += n,
                    Err(e) => errors.push(format!("{table}: {e}")),
                }
            }
        }
        for owner in owners {
            for (table, sql) in BY_OWNER {
                match self.dsr_exec(sql, owner).await {
                    Ok(n) => *counts.entry((*table).to_string()).or_insert(0) += n,
                    Err(e) => errors.push(format!("{table}: {e}")),
                }
            }
        }
        let mut scope_ids: Vec<&str> = account_ids.iter().map(String::as_str).collect();
        scope_ids.push(address);
        for id in scope_ids {
            match self
                .dsr_exec(
                    "DELETE FROM setting_value WHERE scope_kind='user' AND scope_id=?1",
                    id,
                )
                .await
            {
                Ok(n) => *counts.entry("setting_value".into()).or_insert(0) += n,
                Err(e) => errors.push(format!("setting_value: {e}")),
            }
        }
        (counts, errors)
    }

    /// Delete a native mail account row; the cascade removes its mail cache and
    /// PIM rows. The rows removed (1 or 0).
    pub async fn dsr_delete_native_account(&self, account_id: &str) -> Result<u64, StoreError> {
        self.dsr_exec("DELETE FROM accounts WHERE id=?1", account_id)
            .await
    }

    async fn dsr_exec(&self, sql: &'static str, bind: &str) -> Result<u64, StoreError> {
        Ok(q(sql).bind(bind).execute(&self.backend).await?)
    }

    /// Stable ids of the messages cached for an account, oldest first.
    pub async fn dsr_message_ids(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<String>, StoreError> {
        Ok(q("SELECT stable_id FROM messages WHERE mailbox_id IN (SELECT id FROM mailboxes WHERE account_id=?1) ORDER BY internaldate,stable_id LIMIT ?2")
            .bind(account_id)
            .bind(limit)
            .fetch_all_scalar_string(&self.backend)
            .await?)
    }

    /// Cookie consent records kept for the given account ids.
    pub async fn dsr_consent_records(
        &self,
        account_ids: &[String],
    ) -> Result<Vec<Value>, StoreError> {
        let mut out = Vec::new();
        for id in account_ids {
            for r in q("SELECT id,domain,policy_version,decision,categories_json,region,signal,created_at FROM cookie_consent WHERE account_id=?1 ORDER BY created_at")
                .bind(id)
                .fetch_all(&self.backend)
                .await?
            {
                out.push(serde_json::json!({
                    "id": r.get_string("id"), "domain": r.get_string("domain"),
                    "policyVersion": r.get_i64("policy_version"), "decision": r.get_string("decision"),
                    "categories": serde_json::from_str::<Value>(&r.get_string("categories_json")).unwrap_or(Value::Null),
                    "region": r.get_string("region"), "signal": r.get_string("signal"),
                    "createdAt": r.get_i64("created_at"),
                }));
            }
        }
        Ok(out)
    }

    /// Consent records of the given account ids are erased with the person.
    pub async fn dsr_delete_consent(&self, account_id: &str) -> Result<u64, StoreError> {
        self.dsr_exec("DELETE FROM cookie_consent WHERE account_id=?1", account_id)
            .await
    }

    pub fn seal_blob(&self, plain: &[u8]) -> Result<Vec<u8>, StoreError> {
        Ok(self.key.seal(plain)?)
    }

    pub fn open_blob(&self, sealed: &[u8]) -> Result<Vec<u8>, StoreError> {
        Ok(self.key.open(sealed)?)
    }
}
