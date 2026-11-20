//! Lazy repair of the bounded header projection, bound to an immutable source.
use serde_json::Value;

impl crate::engine::Engine {
    /// Preserve a concurrently updated envelope. A failed repair remains a
    /// correct raw-backed response and may be retried by a later request.
    pub(crate) async fn persist_header_projection(&self, source: &mw_store::Message, raw: &[u8]) {
        let Some(blob) = source.blob_ref.as_deref() else {
            return;
        };
        let mut projection = serde_json::json!({});
        crate::header_projection::stamp(&mut projection, raw);
        let Some(projection) = projection.get(crate::header_projection::KEY) else {
            return;
        };
        for _ in 0..2 {
            let rows = match self
                .store()
                .get_account_messages(&source.account_id, &[source.stable_id.clone()])
                .await
            {
                Ok(rows) => rows,
                Err(error) => {
                    tracing::warn!("header projection repair deferred: {error}");
                    return;
                }
            };
            let Some(Some(current)) = rows.first() else {
                return;
            };
            if current.blob_ref.as_deref() != Some(blob)
                || current.mailbox_id != source.mailbox_id
                || current.uid != source.uid
                || current.uidvalidity != source.uidvalidity
            {
                return;
            }
            let mut envelope = match self
                .store()
                .get_account_envelope(&source.account_id, &source.stable_id)
                .await
            {
                Ok(Some(bytes)) => match serde_json::from_slice::<Value>(&bytes) {
                    Ok(value) if value.is_object() => value,
                    _ => return,
                },
                Ok(None) => return,
                Err(error) => {
                    tracing::warn!("header projection repair deferred: {error}");
                    return;
                }
            };
            if crate::header_projection::read(&envelope).is_some() {
                return;
            }
            envelope[crate::header_projection::KEY] = projection.clone();
            let Ok(encoded) = serde_json::to_vec(&envelope) else {
                return;
            };
            match self
                .store()
                .repair_message_envelope_revision(
                    &source.account_id,
                    &source.stable_id,
                    blob,
                    &current.cache_revision,
                    &encoded,
                )
                .await
            {
                Ok(true) => {
                    self.invalidate_message_cache(&source.stable_id).await;
                    return;
                }
                Ok(false) => {}
                Err(error) => {
                    tracing::warn!("header projection repair deferred: {error}");
                    return;
                }
            }
        }
    }
}
