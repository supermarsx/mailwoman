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
    /// Delete several observed sources of one account in ONE transaction, each
    /// exactly as [`Store::delete_message_if_current`] does (a changed source
    /// is left alone and reported `false`), and write their change rows in the
    /// same transaction (W11): one Email `destroyed` row per deleted message at
    /// one state and, with `touch_mailboxes`, one Mailbox `updated` row per
    /// affected mailbox at one state. Sources are processed in mailbox, then id
    /// order, so mailbox locks are taken in a fixed order. The result is
    /// aligned with `items`.
    pub async fn delete_messages_if_current_recorded(
        &self,
        account_id: &str,
        items: &[Message],
        touch_mailboxes: bool,
    ) -> Result<Vec<bool>, StoreError> {
        let mut order: Vec<usize> = (0..items.len()).collect();
        order.sort_by(|a, b| {
            (&items[*a].mailbox_id, &items[*a].stable_id).cmp(&(&items[*b].mailbox_id, &items[*b].stable_id))
        });
        crate::backend::write_job!(self.backend, crate::backend::WriteLane::Normal, |tx| async {
            let mut deleted = vec![false; items.len()];
            let mut gone: Vec<String> = Vec::new();
            let mut boxes: Vec<String> = Vec::new();
            for &at in &order {
                let expected = &items[at];
                if expected.account_id != account_id {
                    continue;
                }
                if lock_mailboxes(tx, &expected.account_id, &expected.mailbox_id, &expected.mailbox_id).await? != 1 {
                    continue;
                }
                let row=q("DELETE FROM messages WHERE stable_id=?1 AND account_id=?2 AND mailbox_id=?3 AND uidvalidity=?4 AND uid=?5 AND cache_revision=?6 AND (blob_ref=?7 OR (blob_ref IS NULL AND ?7 IS NULL)) RETURNING flags_json")
                    .bind(&expected.stable_id).bind(&expected.account_id).bind(&expected.mailbox_id).bind(i64::from(expected.uidvalidity)).bind(i64::from(expected.uid)).bind(&expected.cache_revision).bind(expected.blob_ref.as_deref()).fetch_all_tx(tx).await?;
                let Some(row) = row.first() else { continue };
                q("DELETE FROM message_embeddings WHERE stable_id=?1 AND account_id=?2")
                    .bind(&expected.stable_id)
                    .bind(&expected.account_id)
                    .execute_tx(tx)
                    .await?;
                q("UPDATE mailboxes SET total=CASE WHEN total>0 THEN total-1 ELSE 0 END WHERE id=?1 AND account_id=?2").bind(&expected.mailbox_id).bind(&expected.account_id).execute_tx(tx).await?;
                if crate::cache::flags_json_is_unread(&row.get_string("flags_json")) {
                    q("UPDATE mailboxes SET unread=unread-1 WHERE id=?1 AND account_id=?2 AND unread>0")
                        .bind(&expected.mailbox_id)
                        .bind(&expected.account_id)
                        .execute_tx(tx)
                        .await?;
                }
                deleted[at] = true;
                gone.push(expected.stable_id.clone());
                if !boxes.contains(&expected.mailbox_id) {
                    boxes.push(expected.mailbox_id.clone());
                }
            }
            if !gone.is_empty() {
                // On PostgreSQL lock_mailboxes already holds the per-account
                // change-log lock (lock_body_account).
                let dialect = self.backend.dialect();
                crate::v2::record_changes_insert(dialect, account_id, "Email", &gone, "destroyed")
                    .fetch_all_tx(tx)
                    .await?;
                if touch_mailboxes {
                    crate::v2::record_changes_insert(dialect, account_id, "Mailbox", &boxes, "updated")
                        .fetch_all_tx(tx)
                        .await?;
                }
            }
            Ok::<_, StoreError>(deleted)
        })
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
        self.relocate_inner(expected, destination, uid, uidvalidity, false).await
    }
    /// [`Store::relocate_message_if_current`] with its change rows in the SAME
    /// transaction (W11): an Email row for the message and one Mailbox row each
    /// for the source and the destination (one state per type). Nothing is
    /// recorded when the source has changed (`Ok(false)`).
    pub async fn relocate_message_if_current_recorded(
        &self,
        expected: &Message,
        destination: &str,
        uid: u32,
        uidvalidity: u32,
    ) -> Result<bool, StoreError> {
        self.relocate_inner(expected, destination, uid, uidvalidity, true).await
    }
    async fn relocate_inner(
        &self,
        expected: &Message,
        destination: &str,
        uid: u32,
        uidvalidity: u32,
        record: bool,
    ) -> Result<bool, StoreError> {
        if self.backend.dialect() == crate::backend::Dialect::Postgres {
            return self.relocate_pg(expected, destination, uid, uidvalidity, record).await;
        }
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
        if record {
            // On PostgreSQL lock_mailboxes already holds the per-account
            // change-log lock (lock_body_account).
            let dialect = self.backend.dialect();
            crate::v2::record_changes_insert(dialect, &expected.account_id, "Email", std::slice::from_ref(&expected.stable_id), "updated")
                .fetch_all_tx(tx)
                .await?;
            let mut boxes = vec![expected.mailbox_id.clone()];
            if expected.mailbox_id != destination {
                boxes.push(destination.to_owned());
            }
            crate::v2::record_changes_insert(dialect, &expected.account_id, "Mailbox", &boxes, "updated")
                .fetch_all_tx(tx)
                .await?;
        }
        Ok::<_, StoreError>(true)
        })
    }
    /// PostgreSQL form of `relocate_inner`: the same locks (advisory, account,
    /// both mailboxes in id order) in one statement, then the CAS update, both
    /// mailbox counters and (with `record`) the change rows in one statement.
    /// The unread adjustment comes from `expected.flags_json`: the CAS includes
    /// `cache_revision`, which advances on every update, so a matching row
    /// still has exactly those flags.
    async fn relocate_pg(
        &self,
        expected: &Message,
        destination: &str,
        uid: u32,
        uidvalidity: u32,
        record: bool,
    ) -> Result<bool, StoreError> {
        let moves = expected.mailbox_id != destination;
        let unread = crate::cache::flags_json_is_unread(&expected.flags_json);
        let mut boxes = vec![expected.mailbox_id.clone()];
        if moves {
            boxes.push(destination.to_owned());
        }
        crate::backend::write_job!(self.backend, crate::backend::WriteLane::Normal, |tx| async {
            let lock = q("WITH adv AS MATERIALIZED (SELECT pg_advisory_xact_lock(hashtextextended(?1,719304))),
                 acc AS MATERIALIZED (SELECT id FROM accounts WHERE id=?1 AND (SELECT count(*) FROM adv) >= 0 FOR KEY SHARE)
                 SELECT CAST((SELECT count(*) FROM acc) AS BIGINT) AS a,
                        CAST(count(*) AS BIGINT) AS n
                 FROM (SELECT id FROM mailboxes WHERE account_id=?1 AND id IN (?2,?3) AND (SELECT count(*) FROM acc) >= 0 ORDER BY id FOR UPDATE) m")
                .bind(&expected.account_id)
                .bind(&expected.mailbox_id)
                .bind(destination)
                .fetch_all_tx(tx)
                .await?;
            let row = lock.first().ok_or(StoreError::NotFound)?;
            if row.get_i64("a") == 0 {
                return Err(StoreError::NotFound);
            }
            if row.get_i64("n") != if moves { 2 } else { 1 } {
                return Ok(false);
            }
            let now = chrono::Utc::now().to_rfc3339();
            let done = q("WITH upd AS (UPDATE messages SET mailbox_id=?8,uid=?9,uidvalidity=?10 WHERE stable_id=?1 AND account_id=?2 AND mailbox_id=?3 AND uidvalidity=?4 AND uid=?5 AND cache_revision=?6 AND (blob_ref=?7 OR (blob_ref IS NULL AND ?7 IS NULL)) RETURNING 1),
                 src AS (UPDATE mailboxes SET total=CASE WHEN total>0 THEN total-1 ELSE 0 END,
                                             unread=CASE WHEN ?12=1 AND unread>0 THEN unread-1 ELSE unread END
                         WHERE id=?3 AND account_id=?2 AND ?11=1 AND EXISTS(SELECT 1 FROM upd) RETURNING 1),
                 dst AS (UPDATE mailboxes SET total=total+1, unread=CASE WHEN ?12=1 THEN unread+1 ELSE unread END
                         WHERE id=?8 AND account_id=?2 AND ?11=1 AND EXISTS(SELECT 1 FROM upd) RETURNING 1),
                 ce AS (INSERT INTO changes (account_id, type, state, stable_id, op, at)
                        SELECT ?2, 'Email', s.next, ?1, 'updated', ?14
                        FROM (SELECT COALESCE(MAX(state), 0) + 1 AS next FROM (
                              SELECT MAX(state) AS state FROM changes WHERE account_id=?2 AND type='Email'
                              UNION ALL SELECT head_state AS state FROM change_log_checkpoint WHERE account_id=?2 AND type='Email') heads) s
                        WHERE ?13=1 AND EXISTS(SELECT 1 FROM upd) RETURNING 1),
                 cm AS (INSERT INTO changes (account_id, type, state, stable_id, op, at)
                        SELECT ?2, 'Mailbox', s.next, b.value, 'updated', ?14
                        FROM (SELECT COALESCE(MAX(state), 0) + 1 AS next FROM (
                              SELECT MAX(state) AS state FROM changes WHERE account_id=?2 AND type='Mailbox'
                              UNION ALL SELECT head_state AS state FROM change_log_checkpoint WHERE account_id=?2 AND type='Mailbox') heads) s,
                             json_array_elements_text(?15::json) b
                        WHERE ?13=1 AND EXISTS(SELECT 1 FROM upd) RETURNING 1)
                 SELECT CAST((SELECT count(*) FROM upd) AS BIGINT) AS n,
                        CAST((SELECT count(*) FROM src) + (SELECT count(*) FROM dst) + (SELECT count(*) FROM ce) + (SELECT count(*) FROM cm) AS BIGINT) AS side")
                .bind(&expected.stable_id).bind(&expected.account_id).bind(&expected.mailbox_id)
                .bind(i64::from(expected.uidvalidity)).bind(i64::from(expected.uid)).bind(&expected.cache_revision)
                .bind(expected.blob_ref.as_deref()).bind(destination).bind(i64::from(uid)).bind(i64::from(uidvalidity))
                .bind(i64::from(moves)).bind(i64::from(unread)).bind(i64::from(record)).bind(now)
                .bind(crate::cache::id_list_json(&boxes))
                .fetch_all_tx(tx)
                .await?;
            Ok::<_, StoreError>(done.first().is_some_and(|r| r.get_i64("n") == 1))
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
