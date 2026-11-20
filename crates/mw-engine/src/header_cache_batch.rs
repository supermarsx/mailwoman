//! Cache probes do not establish ownership. Only the caller's fresh scoped
//! Message rows and live request posture authorize these exact namespaces.
use super::{AccountPosture, CacheClass, PlaintextDerived, cache_err};
use crate::{
    Engine,
    backend::{EngineError, Result},
};

pub(super) fn key(message: &mw_store::Message) -> String {
    format!(
        "mail-v3:header-envelope:{}:{}",
        mw_store::HEADER_ENVELOPE_VERSION,
        serde_json::to_string(&(
            &message.account_id,
            &message.stable_id,
            &message.cache_revision
        ))
        .expect("string tuple")
    )
}

impl Engine {
    pub(crate) async fn cached_header_envelopes_with_posture(
        &self,
        account: &str,
        messages: &[Option<mw_store::Message>],
        posture: AccountPosture,
    ) -> Result<Vec<Option<Vec<u8>>>> {
        if messages.len() > crate::jmap::core::MAX_OBJECTS
            || messages
                .iter()
                .flatten()
                .any(|message| message.account_id != account)
        {
            return Err(EngineError::Protocol("invalid owned header batch".into()));
        }
        let cache = self.v6_hooks().cache;
        let mut output = vec![None; messages.len()];
        // Keep only namespace/index pairs, never a second cloned map of mail
        // values. Validate ALL request chunks before writing any source misses.
        let mut inserts = Vec::new();
        for (chunk_index, chunk) in messages.chunks(mw_cache::MAX_MAIL_BATCH_KEYS).enumerate() {
            let offset = chunk_index * mw_cache::MAX_MAIL_BATCH_KEYS;
            let owned = chunk
                .iter()
                .enumerate()
                .filter_map(|(index, row)| row.as_ref().map(|row| (offset + index, row)))
                .collect::<Vec<_>>();
            if owned.is_empty() {
                continue;
            }
            let keys = owned
                .iter()
                .map(|(_, message)| key(message))
                .collect::<Vec<_>>();
            let hits = if posture == AccountPosture::Standard {
                match &cache {
                    Some(cache) => cache
                        .probe_mail_bytes_batch(
                            CacheClass::HeaderWindows,
                            &keys,
                            posture,
                            mw_store::header_envelope::MAX_HEADER_ENVELOPE_BYTES,
                        )
                        .await
                        .map_err(cache_err)?
                        .into_inner(),
                    None => vec![None; owned.len()],
                }
            } else {
                vec![None; owned.len()]
            };
            let mut misses = Vec::new();
            for (position, hit) in hits.into_iter().enumerate() {
                if let Some(hit) = hit {
                    output[owned[position].0] = Some(hit);
                } else {
                    misses.push(position);
                }
            }
            if misses.is_empty() {
                continue;
            }
            let ids = misses
                .iter()
                .map(|position| owned[*position].1.stable_id.clone())
                .collect::<Vec<_>>();
            let loaded = self
                .store()
                .get_account_header_envelopes(account, &ids)
                .await?;
            if loaded.len() != misses.len() {
                return Err(EngineError::Protocol("invalid header batch result".into()));
            }
            for (position, loaded) in misses.into_iter().zip(loaded) {
                let Some(header) = loaded else {
                    continue;
                };
                let (index, message) = owned[position];
                if header.cache_revision != message.cache_revision {
                    return Err(EngineError::Protocol(
                        "message changed during header read; retry".into(),
                    ));
                }
                if header.projected {
                    if header.envelope.len() > mw_store::header_envelope::MAX_HEADER_ENVELOPE_BYTES
                    {
                        return Err(EngineError::Protocol(
                            "header projection byte limit exceeded".into(),
                        ));
                    }
                    if posture == AccountPosture::Standard && cache.is_some() {
                        inserts.push((index, keys[position].clone()));
                    }
                }
                // Full legacy/over-cap envelopes remain request-only; None is
                // never inserted. No request-local loader/coalescing slot is needed.
                output[index] = Some(header.envelope);
            }
        }
        if let Some(cache) = cache {
            for (index, key) in inserts {
                cache
                    .insert_mail_bytes(
                        CacheClass::HeaderWindows,
                        &key,
                        posture,
                        PlaintextDerived::new(
                            output[index]
                                .as_deref()
                                .expect("validated projected envelope"),
                        ),
                        mw_store::header_envelope::MAX_HEADER_ENVELOPE_BYTES,
                    )
                    .await
                    .map_err(cache_err)?;
            }
        }
        Ok(output)
    }
}
