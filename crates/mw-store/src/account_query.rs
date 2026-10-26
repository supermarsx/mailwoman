//! Account-wide metadata listing, independent of plaintext search indexing.
use crate::{Store, StoreError, q};

impl Store {
    /// The default received-date order across every mailbox owned by this account.
    /// Only IDs/sort keys are read; encrypted bodies and envelopes stay unopened.
    pub async fn list_account_message_ids(
        &self,
        account: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<String>, StoreError> {
        if limit <= 0 {
            return Ok(Vec::new());
        }
        Ok(q("SELECT stable_id FROM messages WHERE account_id=?1
           ORDER BY internaldate DESC NULLS LAST, uid DESC, stable_id
           LIMIT ?2 OFFSET ?3")
        .bind(account)
        .bind(limit)
        .bind(offset.max(0))
        .fetch_all_scalar_string(&self.backend)
        .await?)
    }

    pub async fn count_account_messages(&self, account: &str) -> Result<u64, StoreError> {
        Ok(q("SELECT COUNT(*) FROM messages WHERE account_id=?1")
            .bind(account)
            .fetch_scalar_i64(&self.backend)
            .await?
            .max(0) as u64)
    }

    /// Anchor lookup uses the same total order and refuses foreign-account IDs.
    pub async fn message_position_in_account(
        &self,
        account: &str,
        stable_id: &str,
    ) -> Result<Option<u64>, StoreError> {
        let Some(anchor) =
            q("SELECT internaldate,uid FROM messages WHERE account_id=?1 AND stable_id=?2")
                .bind(account)
                .bind(stable_id)
                .fetch_optional(&self.backend)
                .await?
        else {
            return Ok(None);
        };
        let uid = anchor.get_i64("uid");
        let before=match anchor.get_opt_string("internaldate") {
            Some(date) => q("SELECT COUNT(*) FROM messages WHERE account_id=?1 AND internaldate IS NOT NULL
                AND (internaldate>?2 OR (internaldate=?2 AND (uid>?3 OR (uid=?3 AND stable_id<?4))))")
                .bind(account).bind(date).bind(uid).bind(stable_id).fetch_scalar_i64(&self.backend).await?,
            None => q("SELECT COUNT(*) FROM messages WHERE account_id=?1
                AND (internaldate IS NOT NULL OR uid>?2 OR (uid=?2 AND stable_id<?3))")
                .bind(account).bind(uid).bind(stable_id).fetch_scalar_i64(&self.backend).await?,
        };
        Ok(Some(before.max(0) as u64))
    }
}
