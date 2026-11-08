//! Version the persisted MIME attachment projection independently of raw mail.
use serde_json::{Value, json};

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
pub(crate) fn encode(email: &mw_jmap::Email) -> Vec<u8> {
    let mut value = serde_json::to_value(email).unwrap_or_else(|_| json!({}));
    stamp(&mut value);
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
        let mut pending = email.clone();
        pending.as_object_mut().unwrap().remove(SCHEMA_KEY);
        let Ok(bytes) = serde_json::to_vec(&pending) else {
            return;
        };
        match self
            .store()
            .repair_message_envelope(account, id, blob, &bytes)
            .await
        {
            Ok(true) => self.invalidate_message_cache(id).await,
            Ok(false) => return, // The raw source changed before this repair.
            Err(error) => {
                tracing::warn!("attachment metadata persistence deferred: {error}");
                return;
            }
        }
        if !self
            .repair_attachment_index(
                account,
                id,
                email["hasAttachment"].as_bool().unwrap_or(false),
            )
            .await
        {
            return;
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
    async fn repair_attachment_index(&self, account: &str, id: &str, has_attachment: bool) -> bool {
        if self.index_paused(account).await || !self.index_allowed(account).await {
            return true;
        }
        let result = self.search().fetch_doc(id).and_then(|doc| {
            if let Some(mut doc) = doc
                && doc.has_attachment != has_attachment
            {
                doc.has_attachment = has_attachment;
                self.search().upsert(&doc)?;
            }
            Ok(())
        });
        if let Err(error) = result {
            tracing::warn!("attachment index repair deferred: {error}");
            false
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
