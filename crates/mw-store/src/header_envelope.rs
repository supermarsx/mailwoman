//! Optional bounded list metadata. Full envelopes remain the source of truth.
use crate::{
    ServerKey, Store, StoreError,
    backend::{Dialect, Tx},
    q,
};
use serde::de::IgnoredAny;
use serde_json::{Map, Value};
use std::collections::HashMap;

pub const MAX_HEADER_ENVELOPE_BYTES: usize = 128 * 1024;
/// Bump when the metadata whitelist or encoding semantics change. Cache users
/// include this version independently of the full message's cache revision.
pub const HEADER_ENVELOPE_VERSION: u64 = 1;
const LIGHT_SQLITE: &str = "SELECT m.stable_id,m.cache_revision,CASE WHEN length(h.sealed_envelope)<=?3 THEN h.sealed_envelope ELSE NULL END AS light FROM messages m LEFT JOIN message_list_envelope h ON h.stable_id=m.stable_id AND h.account_id=m.account_id WHERE m.account_id=?2 AND m.stable_id IN(SELECT value FROM json_each(?1))";
const PURPOSE: &str = "message-list-envelope-v1";
const KEYS: &[&str] = &[
    "id",
    "blobId",
    "threadId",
    "mailboxIds",
    "keywords",
    "from",
    "to",
    "cc",
    "bcc",
    "replyTo",
    "messageId",
    "inReplyTo",
    "references",
    "subject",
    "receivedAt",
    "sentAt",
    "preview",
    "mailwomanPriority",
    "hasAttachment",
    "size",
    "attachments",
    "_mailwomanAttachmentSchema",
    "_mailwomanHeaderProjection",
];
const PART_KEYS: &[&str] = &[
    "partId",
    "blobId",
    "size",
    "type",
    "charset",
    "name",
    "cid",
    "disposition",
];
const READABLE_HEADERS: &[&str] = &[
    "delivered-to",
    "x-original-to",
    "envelope-to",
    "x-envelope-to",
    "x-forwarded-to",
    "resent-to",
    "expires",
    "expiry-date",
    "reply-by",
    "sensitivity",
    "supersedes",
];

#[derive(Clone)]
pub struct HeaderEnvelope {
    pub envelope: Vec<u8>,
    pub cache_revision: String,
    /// False means the unchanged full envelope was required as a fallback.
    pub projected: bool,
}
impl std::fmt::Debug for HeaderEnvelope {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("HeaderEnvelope")
            .field("envelope_bytes", &self.envelope.len())
            .field("projected", &self.projected)
            .finish_non_exhaustive()
    }
}
#[cfg(test)]
#[path = "header_envelope_tests.rs"]
mod tests;
fn whitespace(bytes: &[u8], at: &mut usize) {
    while bytes.get(*at).is_some_and(u8::is_ascii_whitespace) {
        *at += 1;
    }
}
fn token_len(bytes: &[u8]) -> Option<usize> {
    let mut stream = serde_json::Deserializer::from_slice(bytes).into_iter::<IgnoredAny>();
    stream.next()?.ok()?;
    Some(stream.byte_offset())
}
fn normalize(key: &str, value: &mut Value) -> Option<()> {
    match key {
        "attachments" => {
            for part in value.as_array_mut()? {
                let part = part.as_object_mut()?;
                part.retain(|key, _| PART_KEYS.contains(&key.as_str()));
                if part.iter().any(|(key, value)| {
                    if key == "size" {
                        !value.is_u64()
                    } else {
                        !value.is_null() && !value.is_string()
                    }
                }) {
                    return None;
                }
            }
        }
        "from" | "to" | "cc" | "bcc" | "replyTo" => {
            if !value.is_null() {
                for address in value.as_array_mut()? {
                    let address = address.as_object_mut()?;
                    address.retain(|key, _| matches!(key.as_str(), "email" | "name"));
                    if address
                        .values()
                        .any(|value| !value.is_null() && !value.is_string())
                    {
                        return None;
                    }
                }
            }
        }
        "messageId" | "inReplyTo" | "references" => {
            if !value.is_null() && value.as_array()?.iter().any(|value| !value.is_string()) {
                return None;
            }
        }
        "mailboxIds" | "keywords" => {
            if value.as_object()?.values().any(|value| !value.is_boolean()) {
                return None;
            }
        }
        "hasAttachment" => {
            if !value.is_boolean() {
                return None;
            }
        }
        "size" | "_mailwomanAttachmentSchema" => {
            if !value.is_u64() {
                return None;
            }
        }
        "_mailwomanHeaderProjection" => {
            let fields = value.as_object()?;
            if fields.len() != 2
                || value["version"] != 1
                || serde_json::to_vec(value).ok()?.len() > 64 * 1024
            {
                return None;
            }
            for pair in value.get("pairs")?.as_array()? {
                let pair = pair.as_array()?;
                if pair.len() != 2
                    || !READABLE_HEADERS.contains(&pair[0].as_str()?)
                    || !pair[1].is_string()
                {
                    return None;
                }
            }
        }
        _ => {
            if !value.is_null() && !value.is_string() {
                return None;
            }
        }
    }
    Some(())
}
/// Skip unneeded JSON values without allocating their (possibly multi-MiB) strings.
fn project(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut at = 0;
    whitespace(bytes, &mut at);
    if bytes.get(at) != Some(&b'{') {
        return None;
    }
    at += 1;
    let mut fields = Map::new();
    let mut budget = 0usize;
    loop {
        whitespace(bytes, &mut at);
        if bytes.get(at) == Some(&b'}') {
            at += 1;
            break;
        }
        let key_len = token_len(bytes.get(at..)?)?;
        if key_len > 256 {
            return None;
        }
        let key: String = serde_json::from_slice(&bytes[at..at + key_len]).ok()?;
        at += key_len;
        whitespace(bytes, &mut at);
        if bytes.get(at) != Some(&b':') {
            return None;
        }
        at += 1;
        whitespace(bytes, &mut at);
        let length = token_len(bytes.get(at..)?)?;
        if KEYS.contains(&key.as_str()) {
            budget = budget.checked_add(key_len + length + 2)?;
            if budget > MAX_HEADER_ENVELOPE_BYTES {
                return None;
            }
            let mut value: Value = serde_json::from_slice(&bytes[at..at + length]).ok()?;
            normalize(&key, &mut value)?;
            fields.insert(key, value);
        }
        at += length;
        whitespace(bytes, &mut at);
        match bytes.get(at) {
            Some(b',') => {
                at += 1;
                let mut next = at;
                whitespace(bytes, &mut next);
                if bytes.get(next) == Some(&b'}') {
                    return None;
                }
            }
            Some(b'}') => {
                at += 1;
                break;
            }
            _ => return None,
        }
    }
    whitespace(bytes, &mut at);
    if at != bytes.len() {
        return None;
    }
    let result = serde_json::to_vec(&Value::Object(fields)).ok()?;
    (result.len() <= MAX_HEADER_ENVELOPE_BYTES).then_some(result)
}
fn seal_projection(
    key: &ServerKey,
    account: &str,
    id: &str,
    envelope: &[u8],
) -> Result<Option<Vec<u8>>, StoreError> {
    let Some(projected) = project(envelope) else {
        return Ok(None);
    };
    let payload=serde_json::to_vec(&serde_json::json!({"version":HEADER_ENVELOPE_VERSION,"email":serde_json::from_slice::<Value>(&projected).unwrap()})).unwrap();
    if payload.len() > MAX_HEADER_ENVELOPE_BYTES {
        return Ok(None);
    }
    Ok(Some(key.seal_bound(&payload, PURPOSE, &[account, id])?))
}
fn open_projection(key: &ServerKey, account: &str, id: &str, sealed: &[u8]) -> Option<Vec<u8>> {
    if sealed.len() > MAX_HEADER_ENVELOPE_BYTES + 44 {
        return None;
    }
    let bytes = key.open_bound(sealed, PURPOSE, &[account, id]).ok()?;
    if bytes.len() > MAX_HEADER_ENVELOPE_BYTES {
        return None;
    }
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    if value["version"] != HEADER_ENVELOPE_VERSION {
        return None;
    }
    let email = value.get("email")?.as_object()?;
    if email.keys().any(|key| !KEYS.contains(&key.as_str())) {
        return None;
    }
    let encoded = serde_json::to_vec(email).ok()?;
    // Reject a payload that would change under the descriptor whitelist.
    if serde_json::from_slice::<Value>(&project(&encoded)?).ok()? != Value::Object(email.clone()) {
        return None;
    }
    Some(encoded)
}

pub(crate) async fn refresh_tx(
    tx: &mut Tx,
    key: &ServerKey,
    account: &str,
    id: &str,
    envelope: Option<&[u8]>,
) -> Result<(), StoreError> {
    let Some(envelope) = envelope else {
        return Ok(());
    };
    let Some(sealed) = seal_projection(key, account, id, envelope)? else {
        q("DELETE FROM message_list_envelope WHERE stable_id=?1 AND account_id=?2")
            .bind(id)
            .bind(account)
            .execute_tx(tx)
            .await?;
        return Ok(());
    };
    q("INSERT INTO message_list_envelope(stable_id,account_id,sealed_envelope) VALUES(?1,?2,?3) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,sealed_envelope=excluded.sealed_envelope")
        .bind(id).bind(account).bind(sealed).execute_tx(tx).await?;
    Ok(())
}
pub(crate) async fn lock_parent(tx: &mut Tx, account: &str) -> Result<(), StoreError> {
    if matches!(tx, Tx::Postgres(_)) {
        if q("SELECT id FROM accounts WHERE id=?1 FOR KEY SHARE")
            .bind(account)
            .fetch_all_tx(tx)
            .await?
            .is_empty()
        {
            return Err(StoreError::NotFound);
        }
    }
    Ok(())
}

impl Store {
    pub async fn get_account_header_envelope(
        &self,
        account: &str,
        id: &str,
    ) -> Result<Option<HeaderEnvelope>, StoreError> {
        Ok(self
            .get_account_header_envelopes(account, &[id.to_owned()])
            .await?
            .pop()
            .flatten())
    }
    /// Current light rows never SELECT the full envelope or raw message body.
    /// Missing/invalid rows receive bounded source-CAS lazy repair, not a backfill sweep.
    pub async fn get_account_header_envelopes(
        &self,
        account: &str,
        ids: &[String],
    ) -> Result<Vec<Option<HeaderEnvelope>>, StoreError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let sql = match self.backend.dialect() {
            Dialect::Sqlite => LIGHT_SQLITE,
            Dialect::Postgres => {
                "SELECT m.stable_id,m.cache_revision,CASE WHEN length(h.sealed_envelope)<=?3 THEN h.sealed_envelope ELSE NULL END AS light FROM messages m LEFT JOIN message_list_envelope h ON h.stable_id=m.stable_id AND h.account_id=m.account_id WHERE m.account_id=?2 AND m.stable_id IN(SELECT value FROM json_array_elements_text(?1::json))"
            }
        };
        let rows = q(sql)
            .bind(serde_json::to_string(ids).unwrap())
            .bind(account)
            .bind((MAX_HEADER_ENVELOPE_BYTES + 44) as i64)
            .fetch_all(&self.backend)
            .await?;
        let mut result = HashMap::new();
        for row in rows {
            let id = row.get_string("stable_id");
            if let Some(envelope) = row
                .get_opt_blob("light")
                .and_then(|sealed| open_projection(&self.key, account, &id, &sealed))
            {
                result.insert(
                    id,
                    HeaderEnvelope {
                        envelope,
                        cache_revision: row.get_string("cache_revision"),
                        projected: true,
                    },
                );
            } else if let Some(envelope) = self.header_fallback(account, &id).await? {
                result.insert(id, envelope);
            }
        }
        Ok(ids.iter().map(|id| result.get(id).cloned()).collect())
    }
    async fn header_fallback(
        &self,
        account: &str,
        id: &str,
    ) -> Result<Option<HeaderEnvelope>, StoreError> {
        for attempt in 0..2 {
            let row=q("SELECT envelope_json,cache_revision,blob_ref FROM messages WHERE account_id=?1 AND stable_id=?2")
                .bind(account).bind(id).fetch_optional(&self.backend).await?;
            let Some(row) = row else { return Ok(None) };
            let Some(sealed) = row.get_opt_blob("envelope_json") else {
                return Ok(None);
            };
            let envelope = self.key.open(&sealed)?;
            let revision = row.get_string("cache_revision");
            let original_blob = row.get_opt_string("blob_ref");
            let Some(projected) = seal_projection(&self.key, account, id, &envelope)? else {
                return Ok(Some(HeaderEnvelope {
                    envelope,
                    cache_revision: revision,
                    projected: false,
                }));
            };
            let mut tx = self.backend.begin().await?;
            crate::background_maintenance::lock_body_account(&mut tx, account).await?;
            lock_parent(&mut tx, account).await?;
            let source_sql = if matches!(tx, Tx::Postgres(_)) {
                "SELECT stable_id FROM messages WHERE account_id=?1 AND stable_id=?2 AND cache_revision=?3 AND (blob_ref=?4 OR (blob_ref IS NULL AND ?4 IS NULL)) FOR NO KEY UPDATE"
            } else {
                "SELECT stable_id FROM messages WHERE account_id=?1 AND stable_id=?2 AND cache_revision=?3 AND (blob_ref=?4 OR (blob_ref IS NULL AND ?4 IS NULL))"
            };
            if q(source_sql)
                .bind(account)
                .bind(id)
                .bind(&revision)
                .bind(original_blob)
                .fetch_all_tx(&mut tx)
                .await?
                .is_empty()
            {
                tx.commit().await?;
                if attempt == 0 {
                    continue;
                }
                return Err(StoreError::SourceChanged);
            }
            q("INSERT INTO message_list_envelope(stable_id,account_id,sealed_envelope) VALUES(?1,?2,?3) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,sealed_envelope=excluded.sealed_envelope")
                .bind(id).bind(account).bind(&projected).execute_tx(&mut tx).await?;
            tx.commit().await?;
            return Ok(Some(HeaderEnvelope {
                envelope: open_projection(&self.key, account, id, &projected).unwrap(),
                cache_revision: revision,
                projected: true,
            }));
        }
        Err(StoreError::SourceChanged)
    }
}
