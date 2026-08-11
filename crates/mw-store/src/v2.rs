//! V2 repository methods (plan §2.7) layered over [`Store`]: engine-local
//! per-message metadata (`message_meta`), the per-user tag registry (`tags`),
//! saved searches (`saved_searches`), the persisted submission queue
//! (`submissions`), sending identities (`identities`), and the per-account
//! change log (`changes`) that backs real JMAP state tokens + `*/changes`.
//!
//! Same seam discipline as [`crate::cache`] (plan §1.9): every value crosses as
//! an opaque primitive. Enum-like fields (`undo_status`, change `op`/`type`) are
//! plain strings the engine owns; the store never interprets them.

use std::collections::HashMap;

use crate::backend::Dialect;
use crate::cache::{id_list_json, project_in_order};
use crate::{Row, Store, StoreError, q};

// ---- message_meta ----------------------------------------------------------

/// Engine-local per-message metadata (plan §1.5): pin + snooze + follow-up,
/// keyed by `stable_id`. Surfaced by the engine as extra `Email` properties.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StoredMeta {
    pub pinned: bool,
    /// RFC3339 resurface time, or `None` when not snoozed.
    pub snoozed_until: Option<String>,
    /// RFC3339 follow-up reminder time, or `None`.
    pub follow_up_at: Option<String>,
}

/// A message whose snooze window has elapsed (the resurface scheduler input).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnoozeDue {
    pub account_id: String,
    pub mailbox_id: String,
    pub stable_id: String,
}

/// A per-user tag color/icon registry row (plan §1.5, §2.7 `tags`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagRow {
    pub id: String,
    pub user: String,
    pub name: String,
    pub color: String,
    pub icon: Option<String>,
}

/// A saved search surfaced as a virtual search folder (§2.1, §2.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedSearchRow {
    pub id: String,
    pub user: String,
    pub name: String,
    pub query_json: String,
    pub as_folder: bool,
}

/// A persisted submission (plan §1.3): the undo-send / send-later queue row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionRow {
    pub id: String,
    pub account_id: String,
    pub email_id: String,
    pub identity_id: Option<String>,
    pub send_at: Option<String>,
    /// `pending` | `final` | `canceled` (opaque to the store).
    pub undo_status: String,
    pub hold_seconds: u32,
    pub created_at: String,
}

/// A sending identity (plan §0.7): configured or server-pulled allowed-from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityRow {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub email: String,
    pub reply_to: Option<String>,
    pub signature_html: Option<String>,
    pub signature_text: Option<String>,
    /// Name of the signature TEMPLATE (see `signatures`) this identity applies.
    pub signature_name: Option<String>,
    pub sent_mailbox_id: Option<String>,
    /// `configured` | `pulled`.
    pub source: String,
}

/// One row of the change log for a `*/changes` diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRow {
    pub state: u64,
    pub stable_id: String,
    /// `created` | `updated` | `destroyed` (opaque to the store).
    pub op: String,
}

fn u32_col(row: &Row, col: &str) -> u32 {
    row.get_i64(col) as u32
}

impl Store {
    // ---- message_meta ------------------------------------------------------

    /// Insert or replace a message's engine-local metadata.
    pub async fn upsert_message_meta(
        &self,
        stable_id: &str,
        meta: &StoredMeta,
    ) -> Result<(), StoreError> {
        q(
            "INSERT INTO message_meta (stable_id, pinned, snoozed_until, follow_up_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(stable_id) DO UPDATE SET
                 pinned = excluded.pinned,
                 snoozed_until = excluded.snoozed_until,
                 follow_up_at = excluded.follow_up_at",
        )
        .bind(stable_id)
        .bind(i64::from(meta.pinned))
        .bind(meta.snoozed_until.as_deref())
        .bind(meta.follow_up_at.as_deref())
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Fetch a message's metadata, or `None` if none was ever set.
    pub async fn get_message_meta(
        &self,
        stable_id: &str,
    ) -> Result<Option<StoredMeta>, StoreError> {
        let row =
            q("SELECT pinned, snoozed_until, follow_up_at FROM message_meta WHERE stable_id = ?1")
                .bind(stable_id)
                .fetch_optional(&self.backend)
                .await?;
        Ok(row.map(|r| StoredMeta {
            pinned: r.get_i64("pinned") != 0,
            snoozed_until: r.get_opt_string("snoozed_until"),
            follow_up_at: r.get_opt_string("follow_up_at"),
        }))
    }

    /// Fetch a whole page of message metadata in **one** statement (26.20
    /// t22-e1).
    ///
    /// The batch form of [`Store::get_message_meta`]: the returned vector is the
    /// same length as `stable_ids` and positionally aligned with it, `None`
    /// exactly where the single getter returns `None` (no metadata was ever set
    /// for that id — the common case, since a row exists only once a message has
    /// been pinned, snoozed or flagged for follow-up). Ordering, duplicates and
    /// holes are preserved, so it is interchangeable with
    /// `ids.map(get_message_meta)` as a sequence.
    ///
    /// This is the third of the three per-message reads `Email/get` used to make
    /// in a loop; see [`Store::get_messages`] for the id-list encoding and for
    /// the measurement that motivates the batch form. An empty slice issues
    /// **no** statement.
    pub async fn get_message_metas(
        &self,
        stable_ids: &[String],
    ) -> Result<Vec<Option<StoredMeta>>, StoreError> {
        if stable_ids.is_empty() {
            return Ok(Vec::new());
        }
        let sql = match self.backend.dialect() {
            Dialect::Sqlite => {
                "SELECT stable_id, pinned, snoozed_until, follow_up_at FROM message_meta
                 WHERE stable_id IN (SELECT value FROM json_each(?1))"
            }
            Dialect::Postgres => {
                "SELECT stable_id, pinned, snoozed_until, follow_up_at FROM message_meta
                 WHERE stable_id IN (SELECT value FROM json_array_elements_text(?1::json))"
            }
        };
        let rows = q(sql)
            .bind(id_list_json(stable_ids))
            .fetch_all(&self.backend)
            .await?;
        let found: HashMap<String, StoredMeta> = rows
            .iter()
            .map(|r| {
                (
                    r.get_string("stable_id"),
                    StoredMeta {
                        pinned: r.get_i64("pinned") != 0,
                        snoozed_until: r.get_opt_string("snoozed_until"),
                        follow_up_at: r.get_opt_string("follow_up_at"),
                    },
                )
            })
            .collect();
        Ok(project_in_order(stable_ids, found))
    }

    /// Snoozed messages whose resurface time is at/behind `now` (RFC3339),
    /// joined to their account + current mailbox (the scheduler input).
    pub async fn due_snoozed(&self, now: &str) -> Result<Vec<SnoozeDue>, StoreError> {
        let rows = q(
            "SELECT m.account_id AS account_id, m.mailbox_id AS mailbox_id, mm.stable_id AS stable_id
             FROM message_meta mm JOIN messages m ON m.stable_id = mm.stable_id
             WHERE mm.snoozed_until IS NOT NULL AND mm.snoozed_until <= ?1",
        )
        .bind(now)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| SnoozeDue {
                account_id: r.get_string("account_id"),
                mailbox_id: r.get_string("mailbox_id"),
                stable_id: r.get_string("stable_id"),
            })
            .collect())
    }

    // ---- tags --------------------------------------------------------------

    /// Insert or replace a tag registry entry.
    pub async fn upsert_tag(&self, tag: &TagRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO tags (id, \"user\", name, color, icon) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name, color = excluded.color, icon = excluded.icon",
        )
        .bind(&tag.id)
        .bind(&tag.user)
        .bind(&tag.name)
        .bind(&tag.color)
        .bind(tag.icon.as_deref())
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// List a user's tag registry.
    pub async fn list_tags(&self, user: &str) -> Result<Vec<TagRow>, StoreError> {
        let rows =
            q("SELECT id, \"user\", name, color, icon FROM tags WHERE \"user\" = ?1 ORDER BY name")
                .bind(user)
                .fetch_all(&self.backend)
                .await?;
        Ok(rows
            .iter()
            .map(|r| TagRow {
                id: r.get_string("id"),
                user: r.get_string("user"),
                name: r.get_string("name"),
                color: r.get_string("color"),
                icon: r.get_opt_string("icon"),
            })
            .collect())
    }

    /// Delete a tag registry entry.
    pub async fn delete_tag(&self, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM tags WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ---- saved_searches ----------------------------------------------------

    /// Insert or replace a saved search.
    pub async fn upsert_saved_search(&self, s: &SavedSearchRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO saved_searches (id, \"user\", name, query_json, as_folder)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name, query_json = excluded.query_json, as_folder = excluded.as_folder",
        )
        .bind(&s.id)
        .bind(&s.user)
        .bind(&s.name)
        .bind(&s.query_json)
        .bind(i64::from(s.as_folder))
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// List a user's saved searches.
    pub async fn list_saved_searches(&self, user: &str) -> Result<Vec<SavedSearchRow>, StoreError> {
        let rows = q(
            "SELECT id, \"user\", name, query_json, as_folder FROM saved_searches WHERE \"user\" = ?1 ORDER BY name",
        )
        .bind(user)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(saved_search_from_row).collect())
    }

    /// Fetch one saved search by id.
    pub async fn get_saved_search(&self, id: &str) -> Result<Option<SavedSearchRow>, StoreError> {
        let row =
            q("SELECT id, \"user\", name, query_json, as_folder FROM saved_searches WHERE id = ?1")
                .bind(id)
                .fetch_optional(&self.backend)
                .await?;
        Ok(row.as_ref().map(saved_search_from_row))
    }

    /// Delete a saved search.
    pub async fn delete_saved_search(&self, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM saved_searches WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ---- submissions -------------------------------------------------------

    /// Enqueue a submission (undo-send / send-later).
    pub async fn insert_submission(&self, s: &SubmissionRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO submissions
                 (id, account_id, email_id, identity_id, send_at, undo_status, hold_seconds, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(&s.id)
        .bind(&s.account_id)
        .bind(&s.email_id)
        .bind(s.identity_id.as_deref())
        .bind(s.send_at.as_deref())
        .bind(&s.undo_status)
        .bind(s.hold_seconds as i64)
        .bind(&s.created_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Fetch one submission by id.
    pub async fn get_submission(&self, id: &str) -> Result<Option<SubmissionRow>, StoreError> {
        let row = q(
            "SELECT id, account_id, email_id, identity_id, send_at, undo_status, hold_seconds, created_at
             FROM submissions WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(submission_from_row))
    }

    /// List an account's submissions newest-first — the Outbox
    /// (`EmailSubmission/query`).
    pub async fn list_submissions(
        &self,
        account_id: &str,
    ) -> Result<Vec<SubmissionRow>, StoreError> {
        let rows = q(
            "SELECT id, account_id, email_id, identity_id, send_at, undo_status, hold_seconds, created_at
             FROM submissions WHERE account_id = ?1 ORDER BY created_at DESC, id DESC",
        )
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(submission_from_row).collect())
    }

    /// Set a submission's lifecycle status (`pending`/`final`/`canceled`).
    pub async fn set_submission_status(&self, id: &str, status: &str) -> Result<(), StoreError> {
        q("UPDATE submissions SET undo_status = ?2 WHERE id = ?1")
            .bind(id)
            .bind(status)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Every still-`pending` submission across all accounts (the dispatcher scan).
    pub async fn pending_submissions(&self) -> Result<Vec<SubmissionRow>, StoreError> {
        let rows = q(
            "SELECT id, account_id, email_id, identity_id, send_at, undo_status, hold_seconds, created_at
             FROM submissions WHERE undo_status = 'pending' ORDER BY created_at ASC",
        )
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(submission_from_row).collect())
    }

    // ---- identities --------------------------------------------------------

    /// Insert or replace a sending identity.
    pub async fn upsert_identity(&self, i: &IdentityRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO identities
                 (id, account_id, name, email, reply_to, signature_html, signature_text, signature_name, sent_mailbox_id, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name, email = excluded.email, reply_to = excluded.reply_to,
                 signature_html = excluded.signature_html, signature_text = excluded.signature_text,
                 signature_name = excluded.signature_name,
                 sent_mailbox_id = excluded.sent_mailbox_id, source = excluded.source",
        )
        .bind(&i.id)
        .bind(&i.account_id)
        .bind(&i.name)
        .bind(&i.email)
        .bind(i.reply_to.as_deref())
        .bind(i.signature_html.as_deref())
        .bind(i.signature_text.as_deref())
        .bind(i.signature_name.as_deref())
        .bind(i.sent_mailbox_id.as_deref())
        .bind(&i.source)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// List an account's sending identities.
    pub async fn list_identities(&self, account_id: &str) -> Result<Vec<IdentityRow>, StoreError> {
        let rows = q(
            "SELECT id, account_id, name, email, reply_to, signature_html, signature_text, signature_name, sent_mailbox_id, source
             FROM identities WHERE account_id = ?1 ORDER BY source, email",
        )
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(identity_from_row).collect())
    }

    /// Fetch one identity by id.
    pub async fn get_identity(&self, id: &str) -> Result<Option<IdentityRow>, StoreError> {
        let row = q(
            "SELECT id, account_id, name, email, reply_to, signature_html, signature_text, signature_name, sent_mailbox_id, source
             FROM identities WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(identity_from_row))
    }

    /// Delete a sending identity by id.
    pub async fn delete_identity(&self, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM identities WHERE id = ?1")
            .bind(id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    // ---- changes -----------------------------------------------------------

    /// Append one change and return the new per-`(account,type)` monotonic
    /// state.
    ///
    /// A single atomic `INSERT … SELECT MAX+1 … RETURNING` rather than a
    /// read-then-write transaction: under WAL the lone writer is serialized, so
    /// the next state is computed against the latest committed row and concurrent
    /// appends can't collide. Crucially this takes the write lock directly, so a
    /// contended caller waits out `busy_timeout` instead of failing with a
    /// `SQLITE_BUSY_SNAPSHOT` "database is locked" (which a deferred transaction
    /// that reads-then-upgrades cannot avoid, timeout or not).
    pub async fn record_change(
        &self,
        account_id: &str,
        kind: &str,
        stable_id: &str,
        op: &str,
    ) -> Result<u64, StoreError> {
        let now = chrono::Utc::now().to_rfc3339();
        let next = q(
            "INSERT INTO changes (account_id, type, state, stable_id, op, at)
             VALUES (
                 ?1, ?2,
                 (SELECT COALESCE(MAX(state), 0) + 1 FROM changes WHERE account_id = ?1 AND type = ?2),
                 ?3, ?4, ?5
             )
             RETURNING state",
        )
        .bind(account_id)
        .bind(kind)
        .bind(stable_id)
        .bind(op)
        .bind(&now)
        .fetch_scalar_i64(&self.backend)
        .await?;
        Ok(next as u64)
    }

    /// The current (max) state for an `(account, type)`, `0` if none yet.
    pub async fn current_state(&self, account_id: &str, kind: &str) -> Result<u64, StoreError> {
        let n =
            q("SELECT COALESCE(MAX(state), 0) FROM changes WHERE account_id = ?1 AND type = ?2")
                .bind(account_id)
                .bind(kind)
                .fetch_scalar_i64(&self.backend)
                .await?;
        Ok(n as u64)
    }

    /// Change rows strictly newer than `since`, oldest-first — **unbounded**.
    ///
    /// Retained for the callers that genuinely want the whole tail. Anything
    /// answering a client (`*/changes`, `*/queryChanges`) should use
    /// [`Store::changes_since_limited`] instead: the `changes` table is
    /// append-only — nothing in `crates/` deletes from it — so "the whole tail"
    /// grows for the life of a deployment and this method's result set with it.
    pub async fn changes_since(
        &self,
        account_id: &str,
        kind: &str,
        since: u64,
    ) -> Result<Vec<ChangeRow>, StoreError> {
        Ok(self
            .changes_since_limited(account_id, kind, since, i64::MAX - 1)
            .await?
            .0)
    }

    /// Change rows strictly newer than `since`, oldest-first, capped at `limit`,
    /// plus whether more exist beyond the cap (26.20 t22-e1).
    ///
    /// This is what a JMAP `maxChanges` needs to be honest: the cap has to reach
    /// **SQL**, or the server materialises the entire tail and then throws most
    /// of it away — which is what `Email/queryChanges` does today, at a measured
    /// 1 749 125 bytes of response against a 3 580-byte page. Capping the array
    /// after the fact bounds the *response* while leaving the *work* unbounded.
    ///
    /// The `bool` is computed by asking for one row more than `limit` and
    /// reporting whether it arrived, so a caller gets JMAP's `hasMoreChanges` (or
    /// the trigger for `cannotCalculateChanges`) without a second statement and
    /// without a `COUNT(*)` over the tail. The extra row is never returned.
    ///
    /// `limit` is clamped to at least 1 row of lookahead, so `limit: 0` — legal
    /// in JMAP — answers with an empty page and a truthful `has_more`, rather
    /// than claiming there is nothing left.
    pub async fn changes_since_limited(
        &self,
        account_id: &str,
        kind: &str,
        since: u64,
        limit: i64,
    ) -> Result<(Vec<ChangeRow>, bool), StoreError> {
        let want = limit.max(0);
        let rows = q("SELECT state, stable_id, op FROM changes
             WHERE account_id = ?1 AND type = ?2 AND state > ?3 ORDER BY state ASC
             LIMIT ?4")
        .bind(account_id)
        .bind(kind)
        .bind(since as i64)
        .bind(want.saturating_add(1))
        .fetch_all(&self.backend)
        .await?;
        let has_more = rows.len() as i64 > want;
        Ok((
            rows.iter()
                .take(want as usize)
                .map(|r| ChangeRow {
                    state: r.get_i64("state") as u64,
                    stable_id: r.get_string("stable_id"),
                    op: r.get_string("op"),
                })
                .collect(),
            has_more,
        ))
    }
}

fn saved_search_from_row(r: &Row) -> SavedSearchRow {
    SavedSearchRow {
        id: r.get_string("id"),
        user: r.get_string("user"),
        name: r.get_string("name"),
        query_json: r.get_string("query_json"),
        as_folder: r.get_i64("as_folder") != 0,
    }
}

fn submission_from_row(r: &Row) -> SubmissionRow {
    SubmissionRow {
        id: r.get_string("id"),
        account_id: r.get_string("account_id"),
        email_id: r.get_string("email_id"),
        identity_id: r.get_opt_string("identity_id"),
        send_at: r.get_opt_string("send_at"),
        undo_status: r.get_string("undo_status"),
        hold_seconds: u32_col(r, "hold_seconds"),
        created_at: r.get_string("created_at"),
    }
}

fn identity_from_row(r: &Row) -> IdentityRow {
    IdentityRow {
        id: r.get_string("id"),
        account_id: r.get_string("account_id"),
        name: r.get_string("name"),
        email: r.get_string("email"),
        reply_to: r.get_opt_string("reply_to"),
        signature_html: r.get_opt_string("signature_html"),
        signature_text: r.get_opt_string("signature_text"),
        signature_name: r.get_opt_string("signature_name"),
        sent_mailbox_id: r.get_opt_string("sent_mailbox_id"),
        source: r.get_string("source"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
    };

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    async fn seed_msg(s: &Store) -> (String, String, String) {
        let account_id = s
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "h",
                    port: 993,
                    tls: "implicit",
                    username: "u",
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: "u".into(),
                    password: "p".into(),
                },
            )
            .await
            .unwrap();
        let mailbox_id = s
            .upsert_mailbox(&MailboxUpsert {
                account_id: &account_id,
                name: "INBOX",
                role: Some("inbox"),
                uidvalidity: 100,
                uidnext: 1,
                highestmodseq: 0,
                total: 0,
                unread: 0,
                parent_id: None,
            })
            .await
            .unwrap();
        let sid = s
            .upsert_message(&MessageUpsert {
                account_id: &account_id,
                mailbox_id: &mailbox_id,
                uid: 5,
                uidvalidity: 100,
                message_id: Some("<a@x>"),
                thread_id: None,
                internaldate: Some("2026-07-01T10:00:00Z"),
                size: 10,
                flags_json: "[]",
                envelope: None,
                blob_ref: None,
            })
            .await
            .unwrap();
        (account_id, mailbox_id, sid)
    }

    #[tokio::test]
    async fn message_meta_round_trip_and_snooze_due() {
        let s = store().await;
        let (_a, _m, sid) = seed_msg(&s).await;
        assert_eq!(s.get_message_meta(&sid).await.unwrap(), None);
        s.upsert_message_meta(
            &sid,
            &StoredMeta {
                pinned: true,
                snoozed_until: Some("2020-01-01T00:00:00Z".into()),
                follow_up_at: None,
            },
        )
        .await
        .unwrap();
        let m = s.get_message_meta(&sid).await.unwrap().unwrap();
        assert!(m.pinned);
        // A past snooze time is due; a future one is not.
        assert_eq!(
            s.due_snoozed("2026-07-11T00:00:00Z").await.unwrap().len(),
            1
        );
        s.upsert_message_meta(
            &sid,
            &StoredMeta {
                pinned: true,
                snoozed_until: Some("2999-01-01T00:00:00Z".into()),
                follow_up_at: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(
            s.due_snoozed("2026-07-11T00:00:00Z").await.unwrap().len(),
            0
        );
    }

    /// 26.20 t22-e1. Same contract as [`Store::get_messages`]: the batch form is
    /// the loop **as a sequence**. Metadata is the getter where the hole is the
    /// normal case — most messages are never pinned or snoozed — so a batch
    /// method that returned only the rows it found would misalign every result
    /// after the first unpinned message.
    #[tokio::test]
    async fn get_message_metas_equals_the_loop_as_a_sequence() {
        let s = store().await;
        let (account_id, mailbox_id, first) = seed_msg(&s).await;

        let mut ids = vec![first.clone()];
        for uid in 6..=8u32 {
            ids.push(
                s.upsert_message(&MessageUpsert {
                    account_id: &account_id,
                    mailbox_id: &mailbox_id,
                    uid,
                    uidvalidity: 100,
                    message_id: Some("<m@x>"),
                    thread_id: None,
                    internaldate: Some(&format!("2026-07-0{uid}T10:00:00Z")),
                    size: 10,
                    flags_json: "[]",
                    envelope: None,
                    blob_ref: None,
                })
                .await
                .unwrap(),
            );
        }
        // Only two of the four carry metadata.
        s.upsert_message_meta(
            &ids[1],
            &StoredMeta {
                pinned: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        s.upsert_message_meta(
            &ids[3],
            &StoredMeta {
                pinned: false,
                snoozed_until: Some("2027-01-01T00:00:00Z".into()),
                follow_up_at: None,
            },
        )
        .await
        .unwrap();

        let asked: Vec<String> = vec![
            ids[3].clone(),
            ids[0].clone(),
            "no-such-message".into(),
            ids[1].clone(),
            ids[3].clone(),
        ];
        let mut one_by_one = Vec::new();
        for id in &asked {
            one_by_one.push(s.get_message_meta(id).await.unwrap());
        }
        assert_eq!(s.get_message_metas(&asked).await.unwrap(), one_by_one);

        let batch = s.get_message_metas(&asked).await.unwrap();
        assert_eq!(
            batch[0].as_ref().unwrap().snoozed_until.as_deref(),
            Some("2027-01-01T00:00:00Z")
        );
        assert_eq!(batch[1], None, "a message with no metadata row is a hole");
        assert_eq!(batch[2], None, "so is an id that names no message");
        assert!(batch[3].as_ref().unwrap().pinned);
        assert_eq!(batch[4], batch[0], "a repeated id repeats its answer");

        assert!(s.get_message_metas(&[]).await.unwrap().is_empty());
    }

    /// 26.20 t22-e1. `maxChanges` is only honest if the cap reaches SQL; this
    /// pins the store half of that — the page is capped, `has_more` is truthful
    /// on both sides of the boundary, and the lookahead row never leaks into the
    /// result.
    #[tokio::test]
    async fn changes_since_limited_caps_the_page_and_reports_more() {
        let s = store().await;
        let (account_id, _m, _sid) = seed_msg(&s).await;
        for i in 0..10 {
            s.record_change(&account_id, "Email", &format!("e{i}"), "created")
                .await
                .unwrap();
        }

        let (page, more) = s
            .changes_since_limited(&account_id, "Email", 0, 4)
            .await
            .unwrap();
        assert_eq!(
            page.len(),
            4,
            "the cap is the page size, lookahead excluded"
        );
        assert_eq!(page[0].state, 1);
        assert_eq!(page[3].state, 4);
        assert!(more);

        // Exactly at the boundary there is nothing more, and one row short of it
        // there is — the off-by-one `hasMoreChanges` gets wrong.
        let (page, more) = s
            .changes_since_limited(&account_id, "Email", 0, 10)
            .await
            .unwrap();
        assert_eq!(page.len(), 10);
        assert!(!more);
        let (page, more) = s
            .changes_since_limited(&account_id, "Email", 0, 9)
            .await
            .unwrap();
        assert_eq!(page.len(), 9);
        assert!(more);

        // `since` still means "strictly newer than", and the cap applies to what
        // is left.
        let (page, more) = s
            .changes_since_limited(&account_id, "Email", 8, 4)
            .await
            .unwrap();
        assert_eq!(
            page.iter().map(|c| c.state).collect::<Vec<_>>(),
            vec![9, 10]
        );
        assert!(!more);

        // A zero cap is legal and must not claim the tail is empty.
        let (page, more) = s
            .changes_since_limited(&account_id, "Email", 0, 0)
            .await
            .unwrap();
        assert!(page.is_empty());
        assert!(
            more,
            "there are ten rows waiting — saying otherwise loses them"
        );

        // And the unbounded method still returns everything, unchanged.
        assert_eq!(
            s.changes_since(&account_id, "Email", 0)
                .await
                .unwrap()
                .len(),
            10
        );
    }

    #[tokio::test]
    async fn meta_cascades_on_message_delete() {
        let s = store().await;
        let (_a, _m, sid) = seed_msg(&s).await;
        s.upsert_message_meta(
            &sid,
            &StoredMeta {
                pinned: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        s.delete_message(&sid).await.unwrap();
        assert_eq!(s.get_message_meta(&sid).await.unwrap(), None);
    }

    #[tokio::test]
    async fn relocate_preserves_stable_id_and_meta() {
        let s = store().await;
        let (account_id, _m, sid) = seed_msg(&s).await;
        s.upsert_message_meta(
            &sid,
            &StoredMeta {
                pinned: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let dest = s
            .upsert_mailbox(&MailboxUpsert {
                account_id: &account_id,
                name: "Archive",
                role: Some("archive"),
                uidvalidity: 100,
                uidnext: 1,
                highestmodseq: 0,
                total: 0,
                unread: 0,
                parent_id: None,
            })
            .await
            .unwrap();
        s.relocate_message(&sid, &dest, 42, 100).await.unwrap();
        let loc = s.message_location(&sid).await.unwrap().unwrap();
        assert_eq!((loc.mailbox_id.as_str(), loc.uid), (dest.as_str(), 42));
        // Meta keyed on stable_id survived the move.
        assert!(s.get_message_meta(&sid).await.unwrap().unwrap().pinned);
        assert!(matches!(
            s.relocate_message("nope", &dest, 1, 100).await,
            Err(StoreError::NotFound)
        ));
    }

    #[tokio::test]
    async fn submissions_lifecycle() {
        let s = store().await;
        let (account_id, _m, sid) = seed_msg(&s).await;
        s.insert_submission(&SubmissionRow {
            id: "sub1".into(),
            account_id: account_id.clone(),
            email_id: sid,
            identity_id: None,
            send_at: None,
            undo_status: "pending".into(),
            hold_seconds: 10,
            created_at: "2026-07-01T10:00:00Z".into(),
        })
        .await
        .unwrap();
        assert_eq!(s.pending_submissions().await.unwrap().len(), 1);
        assert_eq!(s.list_submissions(&account_id).await.unwrap().len(), 1);
        s.set_submission_status("sub1", "canceled").await.unwrap();
        assert_eq!(s.pending_submissions().await.unwrap().len(), 0);
        assert_eq!(
            s.get_submission("sub1").await.unwrap().unwrap().undo_status,
            "canceled"
        );
    }

    #[tokio::test]
    async fn identities_and_tags_and_saved_searches() {
        let s = store().await;
        let (account_id, _m, _sid) = seed_msg(&s).await;
        s.upsert_identity(&IdentityRow {
            id: "id1".into(),
            account_id: account_id.clone(),
            name: "Me".into(),
            email: "me@x".into(),
            reply_to: None,
            signature_html: None,
            signature_text: Some("--\nMe".into()),
            signature_name: Some("work".into()),
            sent_mailbox_id: None,
            source: "configured".into(),
        })
        .await
        .unwrap();
        assert_eq!(s.list_identities(&account_id).await.unwrap().len(), 1);
        let got = s.get_identity("id1").await.unwrap().unwrap();
        assert_eq!(got.email, "me@x");
        // The named signature template round-trips (0020 signature_name column).
        assert_eq!(got.signature_name.as_deref(), Some("work"));

        s.upsert_tag(&TagRow {
            id: "t1".into(),
            user: account_id.clone(),
            name: "Work".into(),
            color: "#0af".into(),
            icon: None,
        })
        .await
        .unwrap();
        assert_eq!(s.list_tags(&account_id).await.unwrap().len(), 1);

        s.upsert_saved_search(&SavedSearchRow {
            id: "ss1".into(),
            user: account_id.clone(),
            name: "Big".into(),
            query_json: "larger:1000".into(),
            as_folder: true,
        })
        .await
        .unwrap();
        assert!(s.get_saved_search("ss1").await.unwrap().unwrap().as_folder);
        assert_eq!(s.list_saved_searches(&account_id).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn change_log_states_and_diff() {
        let s = store().await;
        let (account_id, _m, _sid) = seed_msg(&s).await;
        assert_eq!(s.current_state(&account_id, "Email").await.unwrap(), 0);
        let s1 = s
            .record_change(&account_id, "Email", "e1", "created")
            .await
            .unwrap();
        let s2 = s
            .record_change(&account_id, "Email", "e2", "created")
            .await
            .unwrap();
        let s3 = s
            .record_change(&account_id, "Email", "e1", "updated")
            .await
            .unwrap();
        assert_eq!((s1, s2, s3), (1, 2, 3));
        assert_eq!(s.current_state(&account_id, "Email").await.unwrap(), 3);
        let diff = s.changes_since(&account_id, "Email", 1).await.unwrap();
        assert_eq!(diff.len(), 2); // states 2 and 3
        // A different type has its own counter.
        assert_eq!(
            s.record_change(&account_id, "Mailbox", "m1", "updated")
                .await
                .unwrap(),
            1
        );
    }
}
