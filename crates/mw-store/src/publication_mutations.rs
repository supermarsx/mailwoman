//! Source-CAS mutations for operations that awaited an upstream action.
//! History remains with the existing caller; these methods do not append a
//! second Email/Mailbox change sequence.
use crate::{
    Message, Store, StoreError,
    backend::{Tx, q},
};

pub(crate) async fn lock_mailboxes(
    tx: &mut Tx,
    account: &str,
    source: &str,
    destination: &str,
) -> Result<usize, StoreError> {
    crate::background_maintenance::lock_body_account(tx, account).await?;
    crate::header_envelope::lock_parent(tx, account).await?;
    let sql = if matches!(tx, Tx::Postgres(_)) {
        "SELECT id FROM mailboxes WHERE account_id=?1 AND id IN (?2,?3) ORDER BY id FOR UPDATE"
    } else {
        "SELECT id FROM mailboxes WHERE account_id=?1 AND id IN (?2,?3) ORDER BY id"
    };
    Ok(q(sql)
        .bind(account)
        .bind(source)
        .bind(destination)
        .fetch_all_tx(tx)
        .await?
        .len())
}
impl Store {
    /// Delete exactly the source observed before the upstream action. A failed
    /// comparison changes no row, embedding or counter. No history is appended.
    pub async fn delete_message_if_current(&self, expected: &Message) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        if lock_mailboxes(
            &mut tx,
            &expected.account_id,
            &expected.mailbox_id,
            &expected.mailbox_id,
        )
        .await?
            != 1
        {
            tx.commit().await?;
            return Ok(false);
        }
        let row=q("DELETE FROM messages WHERE stable_id=?1 AND account_id=?2 AND mailbox_id=?3 AND uidvalidity=?4 AND uid=?5 AND cache_revision=?6 AND (blob_ref=?7 OR (blob_ref IS NULL AND ?7 IS NULL)) RETURNING flags_json")
            .bind(&expected.stable_id).bind(&expected.account_id).bind(&expected.mailbox_id).bind(i64::from(expected.uidvalidity)).bind(i64::from(expected.uid)).bind(&expected.cache_revision).bind(expected.blob_ref.as_deref()).fetch_all_tx(&mut tx).await?;
        let Some(row) = row.first() else {
            tx.commit().await?;
            return Ok(false);
        };
        q("DELETE FROM message_embeddings WHERE stable_id=?1 AND account_id=?2")
            .bind(&expected.stable_id)
            .bind(&expected.account_id)
            .execute_tx(&mut tx)
            .await?;
        q("UPDATE mailboxes SET total=CASE WHEN total>0 THEN total-1 ELSE 0 END WHERE id=?1 AND account_id=?2").bind(&expected.mailbox_id).bind(&expected.account_id).execute_tx(&mut tx).await?;
        if crate::cache::flags_json_is_unread(&row.get_string("flags_json")) {
            q("UPDATE mailboxes SET unread=unread-1 WHERE id=?1 AND account_id=?2 AND unread>0")
                .bind(&expected.mailbox_id)
                .bind(&expected.account_id)
                .execute_tx(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(true)
    }
    /// Apply flags only to the still-current original source, with the unread
    /// counter in the same transaction. Existing callers own change history.
    pub async fn set_flags_if_current(
        &self,
        expected: &Message,
        flags: &str,
    ) -> Result<bool, StoreError> {
        // One write job: a savepoint in the group writer's batch with
        // MW_SQLITE_WRITER=group, otherwise its own transaction, as before.
        crate::backend::write_job!(self.backend, crate::backend::WriteLane::Normal, |tx| async {
            if lock_mailboxes(
                tx,
                &expected.account_id,
                &expected.mailbox_id,
                &expected.mailbox_id,
            )
            .await?
                != 1
            {
                return Ok(false);
            }
            let n=q("UPDATE messages SET flags_json=?8 WHERE stable_id=?1 AND account_id=?2 AND mailbox_id=?3 AND uidvalidity=?4 AND uid=?5 AND cache_revision=?6 AND (blob_ref=?7 OR (blob_ref IS NULL AND ?7 IS NULL))")
                .bind(&expected.stable_id).bind(&expected.account_id).bind(&expected.mailbox_id).bind(i64::from(expected.uidvalidity)).bind(i64::from(expected.uid)).bind(&expected.cache_revision).bind(expected.blob_ref.as_deref()).bind(flags).execute_tx(tx).await?;
            if n == 0 {
                return Ok(false);
            }
            let old = crate::cache::flags_json_is_unread(&expected.flags_json);
            let new = crate::cache::flags_json_is_unread(flags);
            if old != new {
                let delta = if new { 1i64 } else { -1i64 };
                q("UPDATE mailboxes SET unread=CASE WHEN unread+?1<0 THEN 0 ELSE unread+?1 END WHERE id=?2 AND account_id=?3").bind(delta).bind(&expected.mailbox_id).bind(&expected.account_id).execute_tx(tx).await?;
            }
            Ok::<_, StoreError>(true)
        })
    }

    /// Mirror an upstream move only if its complete original source still
    /// exists. Both mailbox counters and the source update share one transaction.
    pub async fn relocate_message_if_current(
        &self,
        expected: &Message,
        destination: &str,
        uid: u32,
        uidvalidity: u32,
    ) -> Result<bool, StoreError> {
        // One write job: a savepoint in the group writer's batch with
        // MW_SQLITE_WRITER=group, otherwise its own transaction, as before.
        crate::backend::write_job!(self.backend, crate::backend::WriteLane::Normal, |tx| async {
        let count = lock_mailboxes(
            tx,
            &expected.account_id,
            &expected.mailbox_id,
            destination,
        )
        .await?;
        if count
            != if expected.mailbox_id == destination {
                1
            } else {
                2
            }
        {
            return Ok(false);
        }
        let row=q("UPDATE messages SET mailbox_id=?8,uid=?9,uidvalidity=?10 WHERE stable_id=?1 AND account_id=?2 AND mailbox_id=?3 AND uidvalidity=?4 AND uid=?5 AND cache_revision=?6 AND (blob_ref=?7 OR (blob_ref IS NULL AND ?7 IS NULL)) RETURNING flags_json")
            .bind(&expected.stable_id).bind(&expected.account_id).bind(&expected.mailbox_id).bind(i64::from(expected.uidvalidity)).bind(i64::from(expected.uid)).bind(&expected.cache_revision).bind(expected.blob_ref.as_deref()).bind(destination).bind(i64::from(uid)).bind(i64::from(uidvalidity)).fetch_all_tx(tx).await?;
        let Some(row) = row.first() else {
            return Ok(false);
        };
        if expected.mailbox_id != destination {
            q("UPDATE mailboxes SET total=CASE WHEN total>0 THEN total-1 ELSE 0 END WHERE id=?1 AND account_id=?2").bind(&expected.mailbox_id).bind(&expected.account_id).execute_tx(tx).await?;
            q("UPDATE mailboxes SET total=total+1 WHERE id=?1 AND account_id=?2")
                .bind(destination)
                .bind(&expected.account_id)
                .execute_tx(tx)
                .await?;
            if crate::cache::flags_json_is_unread(&row.get_string("flags_json")) {
                q("UPDATE mailboxes SET unread=unread-1 WHERE id=?1 AND account_id=?2 AND unread>0").bind(&expected.mailbox_id).bind(&expected.account_id).execute_tx(tx).await?;
                q("UPDATE mailboxes SET unread=unread+1 WHERE id=?1 AND account_id=?2")
                    .bind(destination)
                    .bind(&expected.account_id)
                    .execute_tx(tx)
                    .await?;
            }
        }
        Ok::<_, StoreError>(true)
        })
    }
    /// Delete a mailbox only while empty, after locking its parent row against
    /// concurrent ingest. Never cascade messages that arrived after enumeration.
    pub async fn delete_mailbox_if_empty(
        &self,
        account: &str,
        mailbox: &str,
    ) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        if lock_mailboxes(&mut tx, account, mailbox, mailbox).await? == 0 {
            tx.commit().await?;
            return Ok(true);
        }
        if q("SELECT stable_id FROM messages WHERE mailbox_id=?1 LIMIT 1")
            .bind(mailbox)
            .fetch_opt_scalar_string_tx(&mut tx)
            .await?
            .is_some()
        {
            tx.commit().await?;
            return Ok(false);
        }
        q("DELETE FROM sync_state WHERE mailbox_id=?1")
            .bind(mailbox)
            .execute_tx(&mut tx)
            .await?;
        q("UPDATE mailboxes SET parent_id=NULL WHERE parent_id=?1")
            .bind(mailbox)
            .execute_tx(&mut tx)
            .await?;
        let n=q("DELETE FROM mailboxes WHERE id=?1 AND account_id=?2 AND NOT EXISTS(SELECT 1 FROM messages WHERE mailbox_id=?1)").bind(mailbox).bind(account).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(n == 1)
    }
}

#[cfg(test)]
#[path = "publication_mutations_tests.rs"]
mod tests;
