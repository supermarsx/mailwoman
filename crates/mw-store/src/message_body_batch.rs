//! Bounded transactional message/body writes, sharing the scalar write path.
use super::*;

const MAX_ROWS: usize = 100;
const MAX_INPUT_BYTES: usize = 8 * 1024 * 1024;

#[cfg(test)]
tokio::task_local! {
    pub(super) static ROW_PAUSE: (std::sync::Arc<tokio::sync::Notify>, std::sync::Arc<tokio::sync::Notify>);
}

impl Store {
    /// Validate a message/body batch without SQL, encryption or a transaction.
    ///
    /// Returns its conservative charged byte count. Callers can use this same
    /// policy to choose bounded chunks BEFORE starting writes; a write failure
    /// must never be treated as a signal to split/retry an uncertain commit.
    pub fn validate_message_body_batch(
        inputs: &[(MessageUpsert<'_>, &[u8])],
    ) -> Result<usize, StoreError> {
        if inputs.len() > MAX_ROWS {
            return Err(StoreError::Corrupt("message body batch exceeds 100 rows".into()));
        }
        let Some((first, _)) = inputs.first() else { return Ok(0); };
        let mut bytes = 0usize;
        for (message, raw) in inputs {
            if message.account_id != first.account_id || message.mailbox_id != first.mailbox_id {
                return Err(StoreError::Corrupt("message body batch requires one account and mailbox".into()));
            }
            // Include source inputs, all prepared body ciphertext and one
            // envelope ciphertext. Header/SQL scratch is not a hard heap cap.
            let fields = [Some(message.account_id), Some(message.mailbox_id),
                message.message_id, message.thread_id, message.internaldate,
                Some(message.flags_json), message.blob_ref];
            let sizes = [raw.len(), raw.len(), 40, 64,
                message.envelope.map_or(0, <[u8]>::len),
                message.envelope.map_or(0, <[u8]>::len), 40];
            for size in sizes.into_iter().chain(fields.into_iter().flatten().map(str::len)) {
                bytes = bytes.checked_add(size)
                    .ok_or_else(|| StoreError::Corrupt("message body batch byte overflow".into()))?;
            }
            if bytes > MAX_INPUT_BYTES {
                return Err(StoreError::Corrupt("message body batch exceeds 8 MiB input budget".into()));
            }
        }
        Ok(bytes)
    }

    /// Atomically apply at most 100 message/body writes to one owned mailbox.
    ///
    /// Results retain input order, including duplicate coordinates. Like
    /// `upsert_message_with_body`, this preserves supplied mailbox counts and
    /// records no Email/Mailbox change-history rows. Body ownership, expiration,
    /// stable-ID resolution and encrypted header projections use the scalar
    /// implementation. IDs become available only after the transaction commits.
    ///
    /// The 8 MiB admission bound charges borrowed raw/envelope lengths plus
    /// worst-case prepared ciphertext and all variable text fields. It is not
    /// a parser or database heap limit. Inputs remain owned by the caller; this method never clones raw.
    pub async fn upsert_message_bodies_batch(
        &self,
        inputs: &[(MessageUpsert<'_>, &[u8])],
    ) -> Result<Vec<(String, bool)>, StoreError> {
        Self::validate_message_body_batch(inputs)?;
        if inputs.is_empty() { return Ok(Vec::new()); }
        // Sealing precedes the writer transaction. Expiration is checked per
        // row inside it, so repeated coordinates observe preceding batch rows.
        let mut prepared = Vec::with_capacity(inputs.len());
        for (message, raw) in inputs {
            prepared.push(self.prepare_body(message.account_id, raw).await?);
        }
        let mut tx = self.backend.begin().await?;
        let mut results = Vec::with_capacity(inputs.len());
        for ((message, raw), body) in inputs.iter().zip(prepared) {
            let expired = q(MESSAGE_BODY_EXPIRED_SQL)
                .bind(message.account_id).bind(message.mailbox_id)
                .bind(i64::from(message.uidvalidity)).bind(i64::from(message.uid))
                .bind(&body.0).fetch_all_tx(&mut tx).await?;
            let body = if expired.is_empty() { (Some(body.0), body.1) } else { (None, None) };
            let result = self.upsert_prepared_body_tx(message, raw, body, false, &mut tx).await?;
            results.push((result.stable_id, result.existed));
            #[cfg(test)]
            if results.len() == 1 {
                if let Ok((entered, release)) = ROW_PAUSE.try_with(Clone::clone) {
                    entered.notify_one();
                    release.notified().await;
                }
            }
        }
        tx.commit().await?;
        Ok(results)
    }
}
