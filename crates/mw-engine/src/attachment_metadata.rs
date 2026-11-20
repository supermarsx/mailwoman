//! Version the persisted MIME attachment projection independently of raw mail.
use crate::backend::{EngineError, Result};
use serde_json::{Value, json};

/// Part descriptors survive cache expiry, but their full values do not. This is
/// inspected only for an actual reader request, never for header/prefetch work.
fn missing_body_values(email: &Value) -> bool {
    let Some(values) = email.get("bodyValues").and_then(Value::as_object) else {
        return true;
    };
    ["textBody", "htmlBody"].iter().any(|name| {
        email
            .get(*name)
            .and_then(Value::as_array)
            .is_some_and(|parts| {
                parts.iter().any(|part| {
                    part.get("partId").and_then(Value::as_str).is_none_or(|id| {
                        !values
                            .get(id)
                            .and_then(|value| value.get("value"))
                            .is_some_and(Value::is_string)
                    })
                })
            })
    })
}

/// Restore only derived body values (and missing part descriptors) from the
/// existing MIME parser, retaining unknown envelope/part metadata. On failure
/// the caller's envelope is unchanged and never marked as hydrated.
fn restore_body_values(email: &mut Value, raw: &[u8]) -> Result<bool> {
    if !missing_body_values(email) {
        return Ok(false);
    }
    let parsed = mw_mime::parse(raw)
        .map_err(|_| EngineError::Protocol("cached message body cannot be parsed".into()))?;
    let fresh = serde_json::to_value(parsed.email)
        .map_err(|_| EngineError::Protocol("cached body projection cannot be encoded".into()))?;
    let fresh_values = fresh
        .get("bodyValues")
        .and_then(Value::as_object)
        .ok_or_else(|| EngineError::Protocol("cached body projection has no values".into()))?;
    let mut hydrated = email.clone();
    let object = hydrated
        .as_object_mut()
        .ok_or_else(|| EngineError::Protocol("cached message envelope is not an object".into()))?;
    for name in ["textBody", "htmlBody"] {
        if object.get(name).is_none_or(Value::is_null) {
            object.insert(name.into(), fresh[name].clone());
        }
        let parts = object
            .get(name)
            .and_then(Value::as_array)
            .ok_or_else(|| EngineError::Protocol("cached body descriptors are invalid".into()))?;
        if parts.iter().any(|part| {
            part.get("partId")
                .and_then(Value::as_str)
                .is_none_or(|id| !fresh_values.contains_key(id))
        }) {
            return Err(EngineError::Protocol(
                "cached body descriptors do not match the raw source".into(),
            ));
        }
    }
    let mut values = object
        .get("bodyValues")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for (id, value) in fresh_values {
        let mut entry = values
            .get(id)
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        if let Some(fields) = value.as_object() {
            for (key, value) in fields {
                entry.insert(key.clone(), value.clone());
            }
        }
        values.insert(id.clone(), Value::Object(entry));
    }
    object.insert("bodyValues".into(), Value::Object(values));
    *email = hydrated;
    Ok(true)
}

pub(crate) const SCHEMA_KEY: &str = "_mailwomanAttachmentSchema";
pub(crate) const SCHEMA_VERSION: u64 = 1;
pub(crate) fn needs_repair(email: &Value) -> bool {
    email
        .get(SCHEMA_KEY)
        .and_then(Value::as_u64)
        .is_none_or(|version| version < SCHEMA_VERSION)
        || !email.get("attachments").is_some_and(Value::is_array)
        || !email.get("hasAttachment").is_some_and(Value::is_boolean)
}
pub(crate) fn stamp(email: &mut Value) {
    email[SCHEMA_KEY] = json!(SCHEMA_VERSION);
}
/// Encode the internal MIME envelope for sealed storage, using the same projections
/// for production ingestion and faithful integration fixtures.
pub fn encode_message_envelope(email: &mw_jmap::Email, raw: &[u8]) -> Vec<u8> {
    let mut value = serde_json::to_value(email).unwrap_or_else(|_| json!({}));
    stamp(&mut value);
    crate::header_projection::stamp(&mut value, raw);
    serde_json::to_vec(&value).unwrap_or_default()
}
/// Keep unrelated envelope fields, and never erase metadata when parsing fails.
pub(crate) fn repair(email: &mut Value, raw: &[u8]) -> bool {
    let Ok(parsed) = mw_mime::parse(raw) else {
        return false;
    };
    let Ok(fresh) = serde_json::to_value(parsed.email) else {
        return false;
    };
    if !email.is_object() || email.as_object().is_some_and(|o| o.is_empty()) {
        *email = fresh;
    } else {
        email["attachments"] = fresh["attachments"].clone();
        email["hasAttachment"] = fresh["hasAttachment"].clone();
    }
    stamp(email);
    true
}

impl crate::engine::Engine {
    /// Reader-only restoration after expiry. Fresh source and revision fences
    /// prevent a derived body projection from overwriting concurrent metadata.
    pub(crate) async fn hydrate_reader_body_values(&self, account: &str, id: &str) -> Result<()> {
        for _ in 0..2 {
            let message = self.store().get_message(id).await?;
            if message.account_id != account {
                return Err(EngineError::Store(mw_store::StoreError::NotFound));
            }
            let mut envelope = match self.store().get_account_envelope(account, id).await? {
                Some(bytes) => serde_json::from_slice::<Value>(&bytes).map_err(|_| {
                    EngineError::Protocol("cached message envelope cannot be parsed".into())
                })?,
                None => json!({}),
            };
            if !missing_body_values(&envelope) {
                return Ok(());
            }
            let Some(blob) = message.blob_ref.as_deref() else {
                if self.message_raw(account, id).await?.is_none() {
                    return Err(EngineError::Protocol(
                        "message body is unavailable for this reader".into(),
                    ));
                }
                continue;
            };
            let Some(raw) = self.cached_body_for_message(&message).await? else {
                if self.message_raw(account, id).await?.is_none() {
                    return Err(EngineError::Protocol(
                        "message body is unavailable for this reader".into(),
                    ));
                }
                continue;
            };
            restore_body_values(&mut envelope, &raw)?;
            let encoded = serde_json::to_vec(&envelope).map_err(|_| {
                EngineError::Protocol("cached body projection cannot be encoded".into())
            })?;
            if self
                .store()
                .repair_message_envelope_revision(
                    account,
                    id,
                    blob,
                    &message.cache_revision,
                    &encoded,
                )
                .await?
            {
                self.invalidate_message_cache(id).await;
                return Ok(());
            }
        }
        Err(EngineError::Protocol(
            "message changed during body restoration; retry the read".into(),
        ))
    }
    /// Stage corrected fields without the version before updating the index.
    /// An index or persistence failure leaves a retryable old version, while
    /// this request still returns the correct metadata instead of failing mail.
    pub(crate) async fn persist_attachment_metadata(
        &self,
        account: &str,
        id: &str,
        blob: &str,
        email: &Value,
    ) {
        let mut ticket = match self.reserve_index_publication(account, 1).await {
            Ok(ticket) => ticket,
            Err(error) => {
                tracing::warn!("attachment index admission deferred: {error}");
                return;
            }
        };
        if ticket.track_attachment_ids(vec![id.into()]).is_err() {
            return;
        }
        let mut pending = email.clone();
        pending.as_object_mut().unwrap().remove(SCHEMA_KEY);
        let Ok(bytes) = serde_json::to_vec(&pending) else {
            return;
        };
        let publication = match self
            .store()
            .repair_message_envelope(account, id, blob, &bytes)
            .await
        {
            Ok(true) => ticket.submit(
                vec![id.into()],
                crate::index_publication::PublicationMode::AttachmentMetadata,
            ),
            Ok(false) => return, // The raw source changed before this repair.
            Err(error) => {
                tracing::warn!("attachment metadata persistence deferred: {error}");
                return;
            }
        };
        self.invalidate_message_cache(id).await;
        match publication {
            Ok(waiter) => {
                if let Err(error) = waiter.await {
                    tracing::warn!("attachment index publication deferred: {error}");
                    return;
                }
            }
            Err(error) => {
                tracing::warn!("attachment index handoff deferred: {error}");
                return;
            }
        }
        let Ok(bytes) = serde_json::to_vec(email) else {
            return;
        };
        match self
            .store()
            .repair_message_envelope(account, id, blob, &bytes)
            .await
        {
            Ok(true) => self.invalidate_message_cache(id).await,
            Ok(false) => {}
            Err(error) => {
                tracing::warn!("attachment metadata version persistence deferred: {error}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reader_restores_only_body_projection_and_preserves_unknown_metadata() {
        let raw = b"From: sender@example.invalid\r\nContent-Type: text/plain\r\n\r\nRecovered content.\r\n";
        let mut email = serde_json::to_value(mw_mime::parse(raw).unwrap().email).unwrap();
        let values = email["bodyValues"].clone();
        let descriptors = email["textBody"].clone();
        email.as_object_mut().unwrap().remove("bodyValues");
        email["futureMetadata"] = json!({"keep": true});
        assert!(restore_body_values(&mut email, raw).unwrap());
        assert_eq!(email["bodyValues"], values);
        assert_eq!(email["textBody"], descriptors);
        assert_eq!(email["futureMetadata"], json!({"keep":true}));
        assert!(
            !restore_body_values(&mut email, b"").unwrap(),
            "complete values avoid reparsing"
        );
    }
    #[test]
    fn reader_projection_failure_never_erases_or_reseals_metadata() {
        let mut email =
            json!({"textBody":[{"partId":"not-in-source"}],"htmlBody":[],"futureMetadata":true});
        let before = email.clone();
        assert!(
            restore_body_values(&mut email, b"From: sender@example.invalid\r\n\r\nBody.\r\n")
                .is_err()
        );
        assert_eq!(email, before);
        assert!(restore_body_values(&mut email, b"").is_err());
        assert_eq!(email, before);
    }
    #[test]
    fn malformed_current_projection_retries_but_newer_valid_versions_are_not_downgraded() {
        let mut email = json!({"hasAttachment":false,"attachments":[]});
        stamp(&mut email);
        assert!(!needs_repair(&email));
        email["attachments"] = Value::Null;
        assert!(needs_repair(&email));
        email["attachments"] = json!([]);
        email[SCHEMA_KEY] = json!(SCHEMA_VERSION + 1);
        assert!(!needs_repair(&email));
    }
    #[test]
    fn invalid_raw_never_erases_metadata_or_marks_it_repaired() {
        let mut email = json!({"hasAttachment":true,"attachments":[{"name":"remember.pdf"}]});
        let before = email.clone();
        assert!(!repair(&mut email, b""));
        assert_eq!(email, before);
        assert!(needs_repair(&email));
    }
}
